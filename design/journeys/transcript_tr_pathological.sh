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
runcap "$OUT/md1-template.txt" '^md1' \
  "$MD" encode "$POLICY" --group-size 0 --force-chunked

echo "The engraved set is $(wc -l < "$OUT/md1-template.txt") md1 chunk(s)."
echo

echo "=== 4. E1: the cards re-encode to the policy they came from ==="
echo
mapfile -t TMPL < "$OUT/md1-template.txt"
run "$MD" verify --template "$POLICY" "${TMPL[@]}"
if ! "$MD" verify --template "$POLICY" "${TMPL[@]}" >/dev/null 2>&1; then
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

echo "=== Artifacts ==="
run ls -la "$OUT"

if [ "$FATAL" -ne 0 ]; then
  echo "TRANSCRIPT FAILED: at least one gate is red (see FATAL above)."
  exit 1
fi
echo "All gates green."
