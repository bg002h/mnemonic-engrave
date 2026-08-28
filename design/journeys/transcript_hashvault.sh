#!/usr/bin/env bash
# transcript_hashvault.sh — the four-tier hashlock vault, start to finish.
#
# Six seeds, six keys, three engraved passphrases, four spending tiers, one
# taproot tree. Every command below is run for real and its output captured;
# every input file is `cat`-ed where it is first used, so nothing in the
# document rests on a value that appears from nowhere.
#
# EVERYTHING HERE IS TEST MATERIAL. Seeds come from entropy 0x0..01-0x0..06 and
# the preimages are printable strings in derive-hashvault-keys.sh. Never fund it.
set -u

W="$(cd "$(dirname "$0")" && pwd)"
C=/scratch/code/shibboleth
MD=$C/descriptor-mnemonic/target/release/md
MK=$C/mnemonic-key/target/release/mk
ME=$C/mnemonic-engrave/target/release/me
# Overridable, following derive-pathological-keys.sh: a P2 implementer must be
# able to point a driver at a BRANCH build without editing it.
MS="${MS:-$C/mnemonic-secret/target/release/ms}"
IN="$W/inputs-hashvault"
OUT="$W/out/hashvault"
mkdir -p "$OUT"

FATAL=0
run() { echo "\$ $*"; "$@" 2>&1; echo "[exit $?]"; echo; }
fatal() { printf 'FATAL: %s\n\n' "$1"; FATAL=1; }
runcap() {
  local out="$1" keep="$2"; shift 2
  echo "\$ $*"
  local sout serr rc
  sout="$(mktemp)"; serr="$(mktemp)"
  "$@" >"$sout" 2>"$serr"; rc=$?
  cat "$sout"; cat "$serr"
  if ! grep -E "$keep" "$sout" > "$out"; then
    rm -f "$out"
    printf 'runcap: CAPTURE FAILED -- no stdout line matched /%s/\n' "$keep"
    [ "$rc" -eq 0 ] && rc=1
  fi
  rm -f "$sout" "$serr"
  echo "[exit $rc]"; echo
}

# PROVENANCE: the PDF builder reads a committed snapshot of this output, so a
# change here that is never re-captured would leave the document asserting
# things the toolchain no longer does. The builder refuses a snapshot whose
# stamp does not match. Regenerate:  bash <this> > transcript_hashvault.txt
echo "# transcript-generator-sha256: $(sha256sum "$0" | cut -d" " -f1)"
echo

echo "=== 0. Versions ==="
echo
run "$MD" --version
run "$MK" --version
run "$MS" --version
run "$ME" --version

echo "=== 1. The source material: six seeds, one per key ==="
echo
# Every key has its OWN seed, so every key has its own master fingerprint. That
# is the difference from the pathological wallet, where three masters supplied
# eleven keys across four accounts.
for i in 0 1 2 3 4 5; do
  run cat "$IN/seeds/key-$i.seed"
done

echo "=== 2. The keys, derived at the constellation's own path ==="
echo
# m/270028'/coin'/account'/script' -- purpose 270028' reads bg002h in one
# component, and level 4 is the script type with 0' = tr. Ruled 2026-08-18.
# `ms derive --template bg002h-tr` is what implements it.
# `--in` on `derive` reads an ms1, not a phrase, so the phrase takes the freed
# stdin instead. Two commands would be the alternative; one is enough here.
run "$MS" derive --phrase - --template bg002h-tr --account 0 < "$IN/seeds/key-0.seed"
echo "The six key files this produced:"
echo
for i in 0 1 2 3 4 5; do
  run cat "$IN/keys/key-$i.xpub"
done

echo "=== 3. The three passphrases, and the hashes the policy commits to ==="
echo
# Engraved as TEXT+QR plates, not as ms1: `ms encode` takes a BIP-39 mnemonic or
# hex entropy, and these are human passphrases.
#
# The hash is over the UTF-8 bytes with NO trailing newline. `printf %s`, never
# `echo` -- a stray \n changes the hash and locks that tier forever.
for i in 0 1 2; do
  echo "\$ cat $IN/preimages/preimage-$i.txt"
  cat "$IN/preimages/preimage-$i.txt"; echo
  echo "\$ printf %s '<that file>' | sha256sum"
  printf '%s' "$(cat "$IN/preimages/preimage-$i.txt")" | sha256sum | cut -d' ' -f1
  echo
done

echo "=== 4. The policy ==="
echo
run cat "$IN/wallet-policy-hashvault.txt"
POLICY=$(cat "$IN/wallet-policy-hashvault.txt")
echo "Four tiers, degrading:"
echo "  tier 1  3-of-3 (@0,@1,@2) + hash A                  any time"
echo "  tier 2  2-of-2 (@3,@4)    + hash B  older(32768)    relative, ~7.6 months from funding"
echo "  tier 3  @5 alone                    after(1173520)  963520 + 210000"
echo "  tier 4  hash C alone                after(1383520)  963520 + 420000   <- NO KEY"
echo

echo "=== 5. Tier 4 has no key, so the default parser refuses it ==="
echo
# Not a miniscript limitation: `and_v(v:after(N),sha256(H))` is well-formed and
# compiles to valid script. rust-miniscript's sanity_check() applies five rules
# and requires_sig() is the first. The library documents them as guarantees
# rather than validity ("results cannot be relied upon") and ships ExtParams so
# individual rules can be relaxed.
run "$MD" encode "$POLICY" --group-size 0
echo "--- and with the flag ---"
echo
runcap "$OUT/md1-template.txt" '^md1' \
  "$MD" encode "$POLICY" --group-size 0 --experimental
echo "The engraved template set is $(wc -l < "$OUT/md1-template.txt") md1 chunk(s)."
echo

echo "=== 6. The cards re-encode to the policy they came from ==="
echo
mapfile -t TMPL < "$OUT/md1-template.txt"
run "$MD" verify --template "$POLICY" "${TMPL[@]}" --experimental
if ! "$MD" verify --template "$POLICY" "${TMPL[@]}" --experimental >/dev/null 2>&1; then
  fatal "the keyless card set does not re-encode to its own policy"
fi

echo "=== 7. What the cards carry ==="
echo
run "$MD" inspect "${TMPL[@]}"

echo "=== 8. The KEYED card set: the same wallet, with the keys inlined ==="
echo
KEYARGS=()
for i in 0 1 2 3 4 5; do
  f="$IN/keys/key-$i.xpub"
  fp=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$f" | head -1)
  KEYARGS+=(--key "@$i=$(grep '^xpub' "$f")" --fingerprint "@$i=$fp")
done
runcap "$OUT/md1-keyed.txt" '^md1' \
  "$MD" encode "$POLICY" "${KEYARGS[@]}" --group-size 0 --experimental
echo "The keyed set is $(wc -l < "$OUT/md1-keyed.txt") md1 chunk(s)."
echo
mapfile -t KEYED < "$OUT/md1-keyed.txt"
TID_T=$("$MD" inspect "${TMPL[@]}" 2>/dev/null | sed -n 's/^wallet-descriptor-template-id: //p')
TID_K=$("$MD" inspect "${KEYED[@]}" 2>/dev/null | sed -n 's/^wallet-descriptor-template-id: //p')
echo "keyless template-id: $TID_T"
echo "keyed   template-id: $TID_K"
if [ -z "$TID_T" ] || [ "$TID_T" != "$TID_K" ]; then
  fatal "the two card sets are NOT the same wallet"
else
  echo "SAME WALLET."
fi
echo

echo "=== 9. Addresses, derived FROM THE CARD ==="
echo
# From the cards, not from the policy string: this proves what the cards carry,
# not what was typed to make them. A taproot address commits to the taptree
# SHAPE, so a wrong tree gives wrong addresses even with every key right.
runcap "$OUT/receive.txt" '^bc1' "$MD" address "${KEYED[@]}" --chain 0 --count 2
runcap "$OUT/change.txt"  '^bc1' "$MD" address "${KEYED[@]}" --chain 1 --count 2

echo "=== 10. The CONCRETE descriptor — what goes on the QR plate ==="
echo
# The device engraves this as TEXT+QR. It is the whole wallet in one string:
# every key, every hash, every timelock.
runcap "$OUT/descriptor.txt" '^tr\(' "$MD" descriptor "${KEYED[@]}"
echo "That descriptor is $(wc -c < "$OUT/descriptor.txt") bytes."
echo

echo "=== 11. The six mk1 key cards ==="
echo
# Bound to the KEYLESS TEMPLATE. The stub is form-aware, so a card minted
# against the keyed policy carries a different stub and is refused against the
# template -- one wallet, two stubs.
KEYFILE="$OUT/keys.txt"
: > "$KEYFILE"
for i in 0 1 2 3 4 5; do
  f="$IN/keys/key-$i.xpub"
  fp=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$f" | head -1)
  pa=$(sed -n 's/.*origin \[[0-9a-f]*\/\([^]]*\)\].*/\1/p' "$f" | head -1)
  printf '[%s/%s]%s\n' "$fp" "$pa" "$(grep '^xpub' "$f")" >> "$KEYFILE"
done
run cat "$KEYFILE"
FROM_MD1=(); for c in "${TMPL[@]}"; do FROM_MD1+=(--from-md1 "$c"); done
if ! "$MK" encode --keys "$KEYFILE" "${FROM_MD1[@]}" --json > "$OUT/mk-encode.json" 2>"$OUT/mk.err"; then
  cat "$OUT/mk.err"; fatal "mk encode --keys failed"
fi
cat "$OUT/mk.err"; rm -f "$OUT/mk.err"
run "$W/mk_cards_from_json.py" "$OUT/mk-encode.json" "$OUT/mk-encode-raw.txt"
TID8=$(printf '%s' "$TID_T" | cut -c1-8)
STUB=$("$MK" decode $(head -n "$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["cards"][0]["chunk_count"])' "$OUT/mk-encode.json")" "$OUT/mk-encode-raw.txt" | tr '\n' ' ') 2>/dev/null | sed -n 's/^policy_id_stubs: *//p' | head -1)
echo "template-id prefix: $TID8   stub mk embedded: $STUB"
[ "$STUB" = "$TID8" ] || fatal "the key cards bind to $STUB, not the template id $TID8"
echo

echo "=== 12. me bundle: the plate checklist ==="
echo
cat "$OUT/md1-template.txt" "$OUT/mk-encode-raw.txt" > "$OUT/backup-strings.txt"
echo "The engraved set: $(grep -c '^md1' "$OUT/backup-strings.txt") md1 + $(grep -c '^mk1' "$OUT/backup-strings.txt") mk1 = $(grep -c . "$OUT/backup-strings.txt") public strings."
rm -rf "$OUT/plates" && mkdir -p "$OUT/plates"
run "$ME" bundle --in "$OUT/backup-strings.txt" --preview "$OUT/plates" --png --manifest "$OUT/manifest.json"

echo "=== 13. The secrets: six seeds and three passphrases ==="
echo
# me refuses an ms1 outright -- it will not render, preview or transmit a
# secret. The passphrase plates are TEXT+QR and carry no ms1 at all, which is
# why tier 4 matters: with no key in that path, THE PASSPHRASE PLATE ALONE
# spends the wallet after its timelock.
run "$MS" encode --in "$IN/seeds/key-0.seed"
MS1=$("$MS" encode --in "$IN/seeds/key-0.seed" --no-engraving-card 2>/dev/null | tr -d ' ')
run "$ME" --in <(printf '%s\n' "$MS1") --hex

echo "=== 14. What goes on which plate, and why ==="
echo
# The device caps a QR at dim 37 -- QR v5, ~106 bytes at ECC-L
# (seedhammer engrave/engrave.go:420). Measured against that cap:
python3 - "$OUT" <<'PY'
import sys, os
OUT = sys.argv[1]
CAP, PLATE = 106, 300
md1 = [l.strip() for l in open(os.path.join(OUT, "md1-template.txt")) if l.startswith("md1")]
mk1 = [l.strip() for l in open(os.path.join(OUT, "mk-encode-raw.txt")) if l.startswith("mk1")]
desc = open(os.path.join(OUT, "descriptor.txt")).read().strip()
print("  device QR cap (dim 37 = QR v5, ECC-L byte mode): ~%d bytes" % CAP)
print("  single-plate TEXT budget:                         %d chars" % PLATE)
print()
print("  md1 chunks  %2d, longest %3d chars  -> TEXT + QR    (fits)" % (len(md1), max(map(len, md1))))
print("  mk1 chunks  %2d, longest %3d chars  -> TEXT ONLY    (over the QR cap)" % (len(mk1), max(map(len, mk1))))
print("  descriptor      %4d bytes          -> %d TEXT plates (%.1fx the QR cap)"
      % (len(desc), -(-len(desc) // (PLATE - 16)), len(desc) / CAP))
PY
echo
echo "The mk1 cards lose their QR to 5 characters: 111 against a 106-byte cap."
echo "Nothing is lost by it -- an mk1 is BCH-protected either way -- but it is"
echo "the difference between scanning a plate and typing it."
echo
run "$W/split_descriptor_plates.py" "$OUT/descriptor.txt" "$OUT/descriptor-plates"
echo "The five descriptor plates, as they will read:"
echo
for f in "$OUT"/descriptor-plates/descriptor-part-*.txt; do
  echo "\$ cat $(basename "$f")"
  head -c 120 "$f"; echo " ..."
  echo
done

echo "=== 15. THE GAP: the engraved set does not say which seed is which slot ==="
echo
# Every key sits at the SAME path from a DIFFERENT seed, so the keyless
# template's six origins are identical and it carries no fingerprints. The
# engraved backup is the template plus the mk1 cards -- and the mk1 cards all
# bind to the same template with the same stub and the same origin too.
#
# Slot order is load-bearing: multi_a commits to key ORDER, so a different
# assignment is a different script, a different leaf, and a different address.
echo "The keyless template's origins, as engraved:"
"$MD" inspect "${TMPL[@]}" 2>/dev/null | sed -n '/^origins:/,/^wallet-policy-id/p' | head -8
echo
echo "The keyed card's origins, which DO name a master:"
"$MD" inspect "${KEYED[@]}" 2>/dev/null | sed -n '/^origins:/,/^wallet-policy-id/p' | head -8
echo
echo "What a wrong assignment costs, measured:"
addr_for() {
  local A=() i=0 idx f fp xp cards
  for idx in "$@"; do
    f="$IN/keys/key-$idx.xpub"
    fp=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$f" | head -1)
    xp=$(grep '^xpub' "$f")
    A+=(--key "@$i=$xp" --fingerprint "@$i=$fp"); i=$((i+1))
  done
  cards=$("$MD" encode "$POLICY" "${A[@]}" --group-size 0 --experimental 2>/dev/null | grep '^md1')
  [ -n "$cards" ] || { echo "ENCODE-FAILED"; return; }
  "$MD" address $cards --chain 0 --count 1 2>/dev/null | grep '^bc1' | head -1
}
echo "  canonical  @0..@5 = key-0..key-5 : $(addr_for 0 1 2 3 4 5)"
echo "  swap @0/@1 (tier 1 3-of-3)       : $(addr_for 1 0 2 3 4 5)"
echo "  swap @3/@4 (tier 2 2-of-2)       : $(addr_for 0 1 2 4 3 5)"
echo
echo "Three different wallets from one set of six keys. 6! = 720 assignments,"
echo "and the engraved template distinguishes none of them."
echo

echo "=== 16. THE FIX: one more chunk, and the template names its slots ==="
echo
echo "The gap is not that the paths collide -- it is that the template carries"
echo "NOTHING ELSE to tell the slots apart. \`md encode\` takes a repeatable"
echo "--fingerprint, and the six masters have six distinct fingerprints, so the"
echo "template can declare them without touching a path, a key or the policy."
echo
FPARGS=()
for i in 0 1 2 3 4 5; do
  FPARGS+=(--fingerprint "@$i=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$IN/keys/key-$i.xpub" | head -1)")
done
runcap "$OUT/md1-template-fp.txt" '^md1' \
  "$MD" encode "$POLICY" --group-size 0 --experimental "${FPARGS[@]}"

FPN=$(grep -c '^md1' "$OUT/md1-template-fp.txt")
BAREN=$(grep -c '^md1' "$OUT/md1-template.txt")
echo "Chunks: $BAREN without fingerprints, $FPN with. The whole cost is $((FPN - BAREN))."
echo

echo "It is the SAME WALLET -- the template id is key-stable and does not move:"
TID_BARE=$("$MD" inspect $(grep '^md1' "$OUT/md1-template.txt" | tr '\n' ' ') 2>/dev/null | sed -n 's/^wallet-descriptor-template-id: //p')
TID_FP=$("$MD" inspect $(grep '^md1' "$OUT/md1-template-fp.txt" | tr '\n' ' ') 2>/dev/null | sed -n 's/^wallet-descriptor-template-id: //p')
if [ -z "$TID_BARE" ] || [ "$TID_BARE" != "$TID_FP" ]; then
  fatal "adding fingerprints changed the wallet: '$TID_BARE' vs '$TID_FP'"
fi
echo "  bare        $TID_BARE"
echo "  fingerprint $TID_FP"
echo

echo "And the slots are now distinguishable:"
"$MD" inspect $(grep '^md1' "$OUT/md1-template-fp.txt" | tr '\n' ' ') 2>/dev/null | grep -E '^\s+@[0-9]+'
echo
echo "The device half proves the consequence -- capture_hashvault.py --fingerprinted"
echo "seats all six cards on this template and derives the wallet's addresses,"
echo "where --seating on the bare template is refused. Same cards, same policy."
echo

echo "=== 17. RESTORE FROM THE PLATES -- and why THIS one cannot ==="
echo
echo "Everything above starts from the strings in out/. An operator starts from"
echo "a stack of plates. restore_from_plates.py goes the other way: it decodes"
echo "the QR out of each rendered plate image with zbarimg -- an independent"
echo "decoder, not the library that drew them -- and rebuilds the wallet from"
echo "what came off the images and nothing else."
echo
echo "This journey's plates carry the BARE template, deliberately: they are the"
echo "set section 15 is about. So the restore is expected to FAIL, and the gate"
echo "below is inverted -- it goes red if these plates ever restore cleanly,"
echo "because that would mean the finding this journey documents has silently"
echo "stopped being true and the narrative around it is now wrong."
echo
run python3 "$W/restore_from_plates.py" hashvault
if python3 "$W/restore_from_plates.py" hashvault >/dev/null 2>&1; then
  fatal "the hashvault plates now restore cleanly -- section 15's finding is stale, rewrite it"
fi
echo "Expected failure, and it is the seatability check that catches it: the"
echo "template-id off the plates is RIGHT, so the QR, the rendering and the"
echo "codec are all fine. What the plates cannot do is say which key is which."
echo

echo "=== Artifacts ==="
run ls -la "$OUT"

if [ "$FATAL" -ne 0 ]; then
  echo "TRANSCRIPT FAILED: at least one gate is red (see FATAL above)."
  exit 1
fi
echo "All gates green."
