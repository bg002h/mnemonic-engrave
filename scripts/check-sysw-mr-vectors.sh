#!/usr/bin/env bash
# check-sysw-mr-vectors.sh — the oracle check for design/vectors/sysw_mr_v1_vectors.json
# (SPEC_systemwide_payloads §14, spec S1 2026-11-22).
#
# WHAT IT CHECKS
#   1. The generator's self-test passes and re-running it reproduces the
#      committed JSON byte for byte (the file is the generator's, not hand-edited).
#   2. Every string and set vector is run through `mr decode` (mnemonic-refugium,
#      the mr1 format's primary implementation): its verdict (exit 0 = accept,
#      non-zero = refuse) and a substring of its first line must match the
#      vector's `mr` field. Where the container and mr disagree on purpose
#      (upper case: mr reads it, the container refuses it; framing-only: the
#      container admits it, mr refuses it) the vector says so in `why`.
#   3. The md1/mk1 strings of every kit vector are packed with `me sysw pack` and
#      read back with `me sysw show`, and their confirmation states must be the
#      ones the vector's verdict relies on.
#   4. RED (optional, ME_RED=1): the `me` named by ME_BIN (default: me on PATH)
#      is run on the single-card vector; before §14 is built it must refuse it
#      with exit 4 and "not a form this container can place" — the behaviour
#      §14 replaces.
#
# WHAT IT DOES NOT COVER: the device (Go) behaviour, the screens, the plate
# count M, and the me refusal TEXTS of §14 (they do not exist until built).
#
# Usage: scripts/check-sysw-mr-vectors.sh            (needs python3, mr, me on PATH)
#        ME_RED=1 ME_BIN=/path/to/me scripts/check-sysw-mr-vectors.sh
set -u
here=$(cd "$(dirname "$0")/.." && pwd)
vec="$here/design/vectors/sysw_mr_v1_vectors.json"
gen="$here/scripts/gen-sysw-mr-vectors.py"
work=$(mktemp -d "${TMPDIR:-/tmp}/sysw-mr-check.XXXXXX")
trap 'rm -r -- "$work"' EXIT
fail=0
bad() { echo "FAIL  $*"; fail=$((fail + 1)); }

echo "mr: $(mr --version 2>&1)"
echo "me: $(${ME_BIN:-me} --version 2>&1)"

python3 -I "$gen" --out "$work/regen.json" >/dev/null || bad "generator self-test or run"
cmp -s "$vec" "$work/regen.json" || bad "committed vectors differ from a fresh generator run"

# 2. mr's verdict on every string and set vector
python3 -I - "$vec" > "$work/mr-cases.tsv" <<'EOF'
import json, sys
d = json.load(open(sys.argv[1]))
for k in ("string_vectors", "set_vectors"):
    for v in d[k]:
        print("\t".join([v["name"], v["mr"]["verdict"], v["mr"]["says"]] + v["mr"]["args"]))
EOF
n=0
while IFS=$'\t' read -r name verdict says args; do
  n=$((n + 1))
  read -r -a argv <<<"$(printf '%s' "$args" | tr '\t' ' ')"
  out=$(mr --network signet decode "${argv[@]}" 2>&1)
  rc=$?
  first=$(printf '%s\n' "$out" | grep -m1 -v '^$')
  if [ "$verdict" = accept ] && [ $rc -ne 0 ]; then bad "$name: mr refused ($rc): $first"; continue; fi
  if [ "$verdict" = refuse ] && [ $rc -eq 0 ]; then bad "$name: mr accepted, vector says refuse"; continue; fi
  case "$first" in *"$says"*) ;; *) bad "$name: mr said '$first', vector expects '$says'";; esac
done < "$work/mr-cases.tsv"
echo "mr oracle: $n vectors checked"

# 3. md1/mk1 confirmation states the kit vectors rely on
python3 -I - "$vec" "$work" <<'EOF'
import json, sys
d = json.load(open(sys.argv[1]))
for v in d["kit_vectors"]:
    mdmk = [r for r in v["records"] if r[:3].lower() in ("md1", "mk1")]
    if mdmk:
        open(f"{sys.argv[2]}/kit-{v['name']}.txt", "w").write("\n".join(mdmk) + "\n")
EOF
for f in "$work"/kit-*.txt; do
  name=$(basename "$f" .txt); name=${name#kit-}
  ${ME_BIN:-me} sysw pack --no-passphrase --in "$f" --out "$work/$name.bin" >/dev/null 2>&1 \
    || { bad "$name: me sysw pack refused the md1/mk1 strings"; continue; }
  show=$(${ME_BIN:-me} sysw show "$work/$name.bin" 2>&1)
  unconf=$(printf '%s\n' "$show" | grep -c 'md1/mk1 — unconfirmed')
  case "$name" in
    kit-mk1-alone) [ "$unconf" -ge 1 ] || bad "$name: the lone mk1 string confirmed; KIT_UNCONFIRMED_MDMK would not fire" ;;
    *)         [ "$unconf" -eq 0 ] || bad "$name: $unconf md1/mk1 string(s) unconfirmed; the vector relies on all confirming" ;;
  esac
done
echo "me confirmation: $(ls "$work"/kit-*.txt | wc -l) kit vectors checked"

# 4. RED: today's me refuses an mr1 record (the behaviour §14 replaces)
if [ "${ME_RED:-0}" = 1 ]; then
  single=$(python3 -I -c "import json,sys;print([v for v in json.load(open(sys.argv[1]))['string_vectors'] if v['name']=='single-card'][0]['record'])" "$vec")
  out=$(${ME_BIN:-me} sysw pack --no-passphrase --out "$work/red.bin" "$single" 2>&1); rc=$?
  echo "RED me sysw pack <single-card>: exit $rc: $(printf '%s' "$out" | head -1 | cut -c1-120)"
  [ $rc -eq 4 ] || bad "RED: expected exit 4 from the pre-§14 me, got $rc"
  case "$out" in *"not a form this container can place"*) ;; *) bad "RED: expected the Unrecognised refusal";; esac
fi

if [ $fail -ne 0 ]; then echo "$fail failure(s)"; exit 1; fi
echo "OK"
