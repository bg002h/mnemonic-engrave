#!/usr/bin/env bash
# build-payload.sh -- build the SH2 demo payload: three cosigner `key:` records
# in a cleartext SYSTEMWIDE container, ready to write at 0x10D00000.
#
# WHY A PAYLOAD AT ALL. A visitor should be able to engrave a TEMPLATE plate or a
# CONCRETE one without first typing three seed phrases into a scroll wheel. The
# device's Wallet Policy composer already reads `key:` records from the payload
# (gui/composer_sources.go composerKeySources) and already offers the
# template-vs-concrete choice at engrave time (gui/composer_engrave.go). This is
# payload AUTHORING -- zero firmware changes. See design/SPEC_sh2_demo_payload.md.
#
# ---------------------------------------------------------------------------
# TWO CORRECTIONS TO THAT SPEC, both measured here on 2026-09-18. Read them
# before "simplifying" this script back to what §8/§9 say.
#
#   1. It is `me sysw pack`, NOT `me seal`. SPEC §9 step 2 prescribes
#      `me seal --plaintext key:... --out demo.uf2`. That does not work, on
#      either channel:
#          me: unrecognised record: unrecognized HRP 'key:6' (expected md, mk,
#          ms, or mt)
#      `me seal` carries constellation STRINGS (md1/mk1/ms1/mt1) only. Composer
#      records (`key:`/`hash:`/`now:`) are a `sysw pack` surface. §8 measured the
#      seal route with mk1 chunks and generalised to `key:` records without
#      running one.
#
#   2. The region is 0x10D00000, NOT 0x10E00000. SPEC §8 carries a paragraph
#      "correcting" the review's 0x10D00000 to 0x10E00000 as "the normative
#      constant". That correction is itself wrong HERE: 0x10E00000 is the SEALED
#      PAYLOAD region (seal.PayloadAddr), and the composer reads the SYSTEMWIDE
#      container -- `ctx.sysw.takeAll(sysw.ClassKey)`, a different container in a
#      different region read by different programs. Flashing these records to
#      0x10E00000 would put them where the composer never looks.
#      SPEC_systemwide_payloads.md §4: systemwide is 0x10D00000-0x10D10000.
# ---------------------------------------------------------------------------
#
# WHY THESE THREE SEEDS. Every one is self-labelling: a person reading the words
# knows immediately it is a test vector, and any wallet derived from one is
# drained by bots the moment it is funded. SPEC §3's invariant is <= 2 DISTINCT
# WORDS per phrase -- abandon/about = 2, zoo/wrong = 2, beef = 1. A real 12-word
# seed with <=2 distinct words has probability ~(2/2048)^12. It does not happen.
# The invariant is PROVEN by the literals below being visible in source (operator
# ruling 2026-09-17: "We don't need to enforce the rule, we can have it be
# proven") and re-checked by check_distinct_words below.
#
# "beef" MUST be spelled as a PHRASE, never as entropy. 0xbeefbeef... decodes to
# "same law room lava winner jelly wing water use wash use teach", which looks
# exactly like a real seed and destroys the self-labelling property (SPEC §2).
set -euo pipefail

OUT="${1:-demo-payload.bin}"

S0="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
S1="zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"
S2="beef beef beef beef beef beef beef beef beef beef beef beef"

check_distinct_words() {
  local n
  n=$(printf '%s\n' $1 | sort -u | wc -l)
  if [ "$n" -gt 2 ]; then
    echo "REFUSING: a demo seed has $n distinct words (SPEC §3 allows <= 2)." >&2
    echo "  It would not be self-labelling, which is the whole safety story." >&2
    exit 1
  fi
  printf '  ok: %d distinct word(s)\n' "$n"
}
echo "SPEC §3 -- <= 2 distinct words per seed:"
check_distinct_words "$S0"; check_distinct_words "$S1"; check_distinct_words "$S2"

# --- derive the three cosigner account keys ---------------------------------
# BIP-48 native-segwit multisig, account 0: m/48'/0'/0'/2'. That is
# md.DefaultOrigin(wsh, 0) on the device (ComposeWrapper.ScriptType() = 2 for
# wsh), so these keys seat into the composer's own wsh presets.
#
# The keys are read off `bundle`'s mk1 CARDS, never off its printed descriptor.
# A card carries the key's REAL BIP-32 header -- depth 4, real parent
# fingerprint, child 2' -- and the device REFUSES a record whose header
# disagrees with its origin (sysw/composer_records.go:399-408):
#
#     if depth != 3 && depth != 4        { return ErrKeyRecord }
#     if len(origin) != depth            { return ErrKeyRecord }
#     if origin[len(origin)-1] != child  { return ErrKeyRecord }
echo
echo "cosigner keys (BIP-48 account 0, m/48'/0'/0'/2'):"
BUNDLE=$(mnemonic bundle --allow-argv-secret \
  --template wsh-sortedmulti --threshold 2 \
  --multisig-path-family bip48 --account 0 --md1-form policy --network mainnet \
  --slot "@0.phrase=$S0" --slot "@1.phrase=$S1" --slot "@2.phrase=$S2" \
  </dev/null 2>/dev/null)

mapfile -t MK1 < <(printf '%s\n' "$BUNDLE" | grep '^mk1' | tr -d ' ')
[ "${#MK1[@]}" -eq 6 ] || { echo "expected 6 mk1 chunks, got ${#MK1[@]}" >&2; exit 1; }

: > "$OUT.records"
for i in 0 1 2; do
  a=$((i*2)); b=$((i*2+1))
  info=$(mnemonic inspect --mk1 "${MK1[$a]}" --mk1 "${MK1[$b]}" </dev/null 2>/dev/null)
  fp=$(printf '%s\n'   "$info" | awk '/^origin_fingerprint:/{print $2}')
  path=$(printf '%s\n' "$info" | awk '/^origin_path:/{print $2}')
  xpub=$(printf '%s\n' "$info" | awk '/^xpub:/{print $2}')
  [ -n "$fp" ] && [ -n "$path" ] && [ -n "$xpub" ] || { echo "slot $i: unreadable card" >&2; exit 1; }
  # `key:<hex of "[fingerprint/path]xpub">`. The path drops its leading "m/":
  # ParseKeyRecord cuts the origin on its FIRST "/" to split off the fingerprint
  # (sysw/composer_records.go:383).
  text="[$fp/${path#m/}]$xpub"
  echo "  @$i  [$fp/${path#m/}]"
  printf 'key:%s\n' "$(printf '%s' "$text" | od -An -tx1 -v | tr -d ' \n')" >> "$OUT.records"
done

# --- pack (cleartext: nothing here is secret) -------------------------------
# No passphrase ceremony, by construction: these are PUBLIC keys, so `me` reports
# "NOT SEALED -- no record in this payload is secret material". A visitor-facing
# demo must not ask a stranger for a passphrase.
#
# --region pads to the full 64 KiB with 0xFF (erased NOR), so the file is
# byte-for-byte what the sector looks like with only this container written.
echo
me sysw pack --in "$OUT.records" --no-passphrase --region --out "$OUT" </dev/null
rm -f "$OUT.records"

echo
echo "verify:"
me sysw show "$OUT" </dev/null | sed 's/^/  /'
echo
echo "flash it (machine in BOOTSEL, laptop power):"
echo "  picotool load --verify -t bin -o 0x10D00000 $OUT"
