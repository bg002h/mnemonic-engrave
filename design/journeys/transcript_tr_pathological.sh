#!/usr/bin/env bash
# The host half of the TAPROOT pathological-wallet journey.
#
# The wallet is the constellation's four-tier degrading vault, expressed as a
# depth-3 taproot tree: two sha256 hashlocks, both Bitcoin timelock flavours,
# multi_a at thresholds 3/2/2/1, and a NUMS internal key so there is no
# key-path spend at all. Eleven cosigners across THREE masters.
#
# THE ROUND TRIP THIS HALF ESTABLISHES (E1): the cards re-encode to the policy
# they came from, checked with `md verify` — which exists for exactly that,
# returns 0/1, and compares PAYLOADS rather than rendered text.
#
# NOT a `md decode` diff, and that is F-219: decode renders per-key origins
# AWAY, so the decoded text is well-formed, missing the field a signer needs,
# and re-encodes to a DIFFERENT card. A journey asserting decode-equality here
# would either fail on a correct card or be quietly weakened to make it pass.
set -u

W="$(cd "$(dirname "$0")" && pwd)"
C=/scratch/code/shibboleth
MD=$C/descriptor-mnemonic/target/release/md
MK=$C/mnemonic-key/target/release/mk
ME=$C/mnemonic-engrave/target/release/me
MS=$C/mnemonic-secret/target/release/ms
IN="$W/inputs-pathological"
# ITS OWN OUTPUT SUBTREE. The two wallet journeys previously shared out/, so one
# could overwrite the other's intermediates and the second would "pass" against
# the first's artifacts.
OUT="$W/out/tr-pathological"
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
    FATAL=1
  fi
  rm -f "$sout" "$serr"
  echo "[exit $rc]"; echo
}

POLICY="$(cat "$IN/wallet-policy-tr.txt")"

# PROVENANCE. The PDF builder reads a COMMITTED SNAPSHOT of this script's
# output, not a live run. `section()` already fails on a RENAMED header, but a
# snapshot can go stale with every header still matching -- which is exactly
# what happened to the wsh journey: the F-127 fix corrected an obstacle, the
# .txt was never re-captured, and the document kept publishing the old failure
# as current. Stamping this script's hash lets the builder refuse a stale
# snapshot. Regenerate with:  bash <this script> > <the .txt>
echo "# transcript-generator-sha256: $(sha256sum "$0" | cut -d" " -f1)"

echo "=== 0. Versions ==="
echo
run "$MD" --version

echo "=== 1. The wallet ==="
echo
run cat "$IN/wallet-policy-tr.txt"

# G1 — EVERY SLOT'S TEMPLATE ORIGIN MUST MATCH ITS OWN KEY FILE'S HEADER.
# The origins are written into the template by hand; the key files declare them
# independently. If the two ever disagree the card names a path the key does not
# live at, and nothing downstream would notice -- addresses come from the xpub,
# never from the declared origin (F-217's exact lesson).
echo "=== 2. Every slot's origin matches its key file (G1) ==="
echo
python3 - "$IN" "$POLICY" <<'PY'
import glob, re, sys
inp, policy = sys.argv[1], sys.argv[2]
bad = 0
for f in sorted(glob.glob(inp + "/keys/key-*.xpub")):
    txt = open(f).read()
    slot = int(re.search(r"@(\d+)", txt).group(1))
    path = re.search(r"\[[0-9a-f]{8}/([^\]]+)\]", txt).group(1)
    want = "@%d/%s/<0;1>/*" % (slot, path)
    if want not in policy:
        print("  MISMATCH @%d: key file says %s, template does not carry it" % (slot, path))
        bad += 1
    else:
        print("  @%-2d %s ok" % (slot, path))
print("\n%d slot(s) disagree with their key file" % bad)
sys.exit(1 if bad else 0)
PY
if [ $? -ne 0 ]; then fatal "G1: a slot's template origin disagrees with its key file"; fi
echo

echo "=== 3. The engraved artifact: the KEYLESS TEMPLATE card set ==="
echo
# No --path: the origins ride in the template. --path would FLATTEN eleven
# distinct origins onto one shared path, which over eleven different keys is the
# impossible card F-217 refuses.
# F-227: DECLARE THE FINGERPRINTS IN THE KEYLESS TEMPLATE.
#
# Without them this engraved set cannot be RESTORED, only verified. The
# template names its slots by origin, and eleven slots share only four
# distinct origins -- so a gathered mk1 card matches several slots at once, and
# a device that will not guess must refuse the whole set
# (errSeatSlotContested, gui/key_card_seating.go). `md encode` warns about this
# now; before F-227 nothing did, which is how it survived a journey with a
# restore test.
#
# All eleven (fingerprint, path) pairs ARE unique across the three masters, so
# declaring the fingerprints closes it outright. Same shape as the origin fix
# above and the same price: about one extra chunk, and the template-id does not
# move, so every address, wallet id and mk1 stub below is unchanged.
#
# Slot assignment comes from each key file's own @N marker -- the same source
# the keyed card's arguments use, rather than a second hand-maintained list.
FPARGS=()
for f in "$IN"/keys/key-*.xpub; do
  slot=$(grep -oE '@[0-9]+' "$f" | head -1 | tr -d '@')
  fp=$(grep -oE '\[[0-9a-f]{8}/' "$f" | head -1 | tr -d '[/')
  [ -n "$slot" ] && [ -n "$fp" ] || { echo "FATAL: no @N or fingerprint in $f" >&2; exit 1; }
  FPARGS+=(--fingerprint "@$slot=$fp")
done
# TWO array elements per key -- the flag and its value -- so eleven keys is 22.
# Asserted because a silently short list would engrave a template that is still
# unseatable while looking fixed, which is the exact failure mode F-227 is about.
if [ ${#FPARGS[@]} -ne 22 ]; then
  echo "FATAL: expected 11 fingerprints (22 argv elements), built ${#FPARGS[@]}" >&2; exit 1
fi

runcap "$OUT/md1-template.txt" '^md1' \
  "$MD" encode "$POLICY" "${FPARGS[@]}" --group-size 0 --force-chunked

echo "The engraved set is $(wc -l < "$OUT/md1-template.txt") md1 chunk(s)."
echo

echo "=== 4. E1: the cards re-encode to the policy they came from ==="
echo
mapfile -t TMPL < "$OUT/md1-template.txt"
# The fingerprints go to VERIFY too. Verify re-encodes the template from the
# arguments and compares payloads, so omitting them here compares a
# fingerprinted card against an unfingerprinted expectation and fails -- which
# is what happened the first time this ran. Same rule as `--experimental`:
# verify must be given whatever encode was given, or a card authored with a
# flag becomes unverifiable, which is worse than not authoring it.
run "$MD" verify --template "$POLICY" "${FPARGS[@]}" "${TMPL[@]}"
if ! "$MD" verify --template "$POLICY" "${FPARGS[@]}" "${TMPL[@]}" >/dev/null 2>&1; then
  fatal "E1: the keyless card set does not re-encode to its own policy"
fi

echo "=== 5. G3: the committed input file is what this script produces ==="
echo
run diff "$OUT/md1-template.txt" "$IN/backup-strings-tr.txt"
if ! diff -q "$OUT/md1-template.txt" "$IN/backup-strings-tr.txt" >/dev/null; then
  fatal "G3: backup-strings-tr.txt is stale -- regenerate it from this script"
fi

echo "=== 6. The device's card set: the KEYED full policy ==="
echo
KEYARGS=()
for f in "$IN"/keys/key-*.xpub; do
  slot=$(grep -oE '@[0-9]+' "$f" | head -1 | tr -d '@')
  fp=$(grep -oE '\[[0-9a-f]{8}/' "$f" | head -1 | tr -d '[/')
  xp=$(grep -E '^xpub' "$f" | head -1)
  KEYARGS+=(--key "@$slot=$xp" --fingerprint "@$slot=$fp")
done
runcap "$OUT/md1-keyed.txt" '^md1' \
  "$MD" encode "$POLICY" "${KEYARGS[@]}" --group-size 0 --force-chunked
echo "The device gathers $(wc -l < "$OUT/md1-keyed.txt") md1 chunk(s)."
echo

echo "=== 7. THE BIND: both card sets are the same wallet ==="
echo
# Without this the journey could engrave one wallet and prove another. The
# TEMPLATE id is key-STABLE, so it is the same across the keyless and keyed
# forms -- and that is exactly what makes it the right thing to compare.
mapfile -t KEYED < "$OUT/md1-keyed.txt"
run "$MD" inspect "${TMPL[@]}"
run "$MD" inspect "${KEYED[@]}"
TID_T=$("$MD" inspect "${TMPL[@]}" 2>/dev/null | grep '^wallet-descriptor-template-id:' | awk '{print $2}')
TID_K=$("$MD" inspect "${KEYED[@]}" 2>/dev/null | grep '^wallet-descriptor-template-id:' | awk '{print $2}')
echo "keyless template-id: $TID_T"
echo "keyed   template-id: $TID_K"
if [ -z "$TID_T" ] || [ "$TID_T" != "$TID_K" ]; then
  fatal "the two card sets are NOT the same wallet"
else
  echo "SAME WALLET."
fi
echo

runcap "$OUT/tr.id.txt" '^wallet-policy-id:' "$MD" inspect "${KEYED[@]}"

echo "=== 8. What the device must prove to ==="
echo
# Derived from the CARDS, not from the argument list: this proves what the cards
# carry, not what was typed to make them.
runcap "$OUT/tr.receive.txt" '^bc1' "$MD" address "${KEYED[@]/#/}" --chain 0 --count 2
runcap "$OUT/tr.change.txt"  '^bc1' "$MD" address "${KEYED[@]}" --chain 1 --count 2

echo "=== 9. The eleven key cards ==="
echo
# Until now this journey engraved a DESCRIPTOR and nothing else. A descriptor
# card alone is not a backup: it names eleven @N slots and carries no way for a
# cosigner to prove which slot is theirs. The mk1 key cards are that half.
#
# They bind to the KEYLESS TEMPLATE card, not the keyed one. The stub is
# form-aware (SPEC_mk_v0_1.md 3.3), so a card minted against the keyed policy
# carries a DIFFERENT stub and is refused against the template -- one wallet,
# two stubs. The template is what gets engraved here, so the template is what
# the cards must bind to.
KEYFILE="$OUT/keys.txt"
KEYMETA="$OUT/keys-meta.txt"
: > "$KEYFILE"; : > "$KEYMETA"
for f in "$IN"/keys/key-*.xpub; do
  KX=$(grep '^xpub' "$f")
  KFP=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$f" | head -1)
  KPATH=$(sed -n 's/.*origin \[[0-9a-f]*\/\([^]]*\)\].*/\1/p' "$f" | head -1)
  if [ -z "$KX" ] || [ -z "$KFP" ] || [ -z "$KPATH" ]; then
    fatal "$f is missing an xpub or an origin header"; continue
  fi
  printf '[%s/%s]%s\n' "$KFP" "$KPATH" "$KX" >> "$KEYFILE"
  ki=$(basename "$f" .xpub | sed 's/key-0*//'); [ -z "$ki" ] && ki=0
  printf '%s %s %s\n' "$ki" "$KFP" "$KPATH" >> "$KEYMETA"
done
run head -2 "$KEYFILE"

# ONE invocation for all eleven, and each card derives its own binding from the
# template card rather than from a hand-copied stub.
FROM_MD1=(); for c in "${TMPL[@]}"; do FROM_MD1+=(--from-md1 "$c"); done
CARDS_JSON="$OUT/mk-encode.json"
if ! "$MK" encode --keys "$KEYFILE" "${FROM_MD1[@]}" --json > "$CARDS_JSON" 2>"$OUT/mk.err"; then
  cat "$OUT/mk.err"; fatal "mk encode --keys failed for the taproot template"
fi
cat "$OUT/mk.err"; rm -f "$OUT/mk.err"
run "$W/mk_cards_from_json.py" "$CARDS_JSON" "$OUT/mk-encode-raw.txt" \
  || fatal "could not read the batch JSON"

# The stub every card carries must be the TEMPLATE id, not the policy id.
TID=$("$MD" inspect "${TMPL[@]}" 2>/dev/null | sed -n 's/^wallet-descriptor-template-id: //p' | cut -c1-8)
PID=$("$MD" inspect "${TMPL[@]}" 2>/dev/null | sed -n 's/^wallet-policy-id: //p' | cut -c1-8)
# Exactly the FIRST card's chunks -- `head -3` spanned two cards, and an
# incomplete chunk set decodes to nothing, which read as an empty stub rather
# than as the mistake it was.
N0=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["cards"][0]["chunk_count"])' "$CARDS_JSON")
STUB=$("$MK" decode $(head -n "$N0" "$OUT/mk-encode-raw.txt" | tr '\n' ' ') 2>/dev/null | sed -n 's/^policy_id_stubs: *//p' | head -1)
echo "template-id prefix: $TID   policy-id prefix: $PID   stub mk embedded: $STUB"
if [ "$STUB" != "$TID" ]; then
  fatal "the key cards bind to $STUB, not the template id $TID"
elif [ "$STUB" = "$PID" ]; then
  fatal "the key cards bind to the POLICY id -- wrong form for a keyless card"
else
  echo "The cards bind to the keyless template. Same wallet, correct form."
fi
echo

echo "=== 10. me bundle: the plate checklist ==="
echo
# What actually gets cut. The descriptor chunks and every key card in one set,
# so the plate count is the real one rather than the descriptor's alone.
cat "$OUT/md1-template.txt" "$OUT/mk-encode-raw.txt" > "$OUT/backup-strings.txt"
echo "The engraved set: $(grep -c '^md1' "$OUT/backup-strings.txt") md1 + $(grep -c '^mk1' "$OUT/backup-strings.txt") mk1 = $(grep -c . "$OUT/backup-strings.txt") strings."
rm -rf "$OUT/plates" && mkdir -p "$OUT/plates"
run "$ME" bundle --in "$OUT/backup-strings.txt" --preview "$OUT/plates" --png --manifest "$OUT/manifest.json"
if [ ! -s "$OUT/manifest.json" ]; then
  fatal "me bundle produced no manifest"
fi
echo

echo "=== 11. The seed, and the refusal that still holds ==="
echo
# The one plate that is NOT public. `me` refuses an ms1 outright -- it will not
# render, preview or transmit a secret -- so the seed plate is listed in the
# manifest and never rendered. That refusal is the reason the bundle above can
# be previewed at all.
run cat "$IN/seeds/master-A.seed"
run "$MS" encode --phrase "$(cat "$IN/seeds/master-A.seed")"
MS1=$("$MS" encode --phrase "$(cat "$IN/seeds/master-A.seed")" --no-engraving-card 2>/dev/null | tr -d ' ')
run "$ME" --in <(printf '%s\n' "$MS1") --hex
echo

echo "=== 12. THE RESTORE TEST: the plates plus the seeds give the wallet back ==="
echo
# Section 4 proves the card RE-ENCODES to its own policy -- a transcription
# check on the bytes. This asks the different question: hold these plates and
# the three seeds, and do you get the wallet back?
#
# Taproot makes that question harder than it is for wsh. A tr() address commits
# to the TAPTREE SHAPE as well as the keys, so a restore can recover all eleven
# keys, rebuild the tree wrongly, and produce a wallet that looks right and is
# not. The test therefore ends on ADDRESSES, not on keys.
run "$W/restore_test_tr_pathological.py"
if ! "$W/restore_test_tr_pathological.py" >/dev/null 2>&1; then
  fatal "a card-only-plus-seeds restore does NOT reproduce this wallet"
fi
echo

echo "=== Artifacts ==="
run ls -la "$OUT"

if [ "$FATAL" -ne 0 ]; then
  echo "TRANSCRIPT FAILED: at least one gate is red (see FATAL above)."
  exit 1
fi
echo "All gates green."
