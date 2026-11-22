#!/usr/bin/env bash
# spec-s1-gate.sh — the build gate for amendment S1 of SPEC_systemwide_payloads.md
# (§14, 2026-11-22). A fold of §14 is authorship and re-runs this.
#
# WHAT IT RUNS
#   1. scripts/check-sysw-mr-vectors.sh (generator self-test, committed JSON
#      reproduced, `mr decode` oracle on every string/set vector, `me sysw show`
#      on every kit vector's md1/mk1 strings; with ME_RED=1 also the RED row).
#   2. scripts/spec-check.py and scripts/spec-structure-check.sh on the spec
#      AND on its base revision (BASE, default origin/master): the amendment may
#      add no problem line the base does not have (line numbers stripped).
#   3. Quote checks: every reason code and every vector name in the JSON
#      appears in §14; every §14g reason row appears once; §8.3 lists tests
#      24 to 32; each fixed screen text of §14e/§14f appears in §14.
#
# WHAT IT DOES NOT COVER: whether the rules are RIGHT (a reviewer's job);
# device behaviour (no Go code exists for §14 yet); the plate count M; any
# claim about the fork beyond the cited names, which a reviewer resolves at
# seedhammer origin/main.
set -u
here=$(cd "$(dirname "$0")/.." && pwd)
spec="$here/design/SPEC_systemwide_payloads.md"
vec="$here/design/vectors/sysw_mr_v1_vectors.json"
base=${BASE:-origin/master}
work=$(mktemp -d "${TMPDIR:-/tmp}/s1-gate.XXXXXX")
trap 'rm -r -- "$work"' EXIT
fail=0
bad() { echo "FAIL  $*"; fail=$((fail + 1)); }

echo "== 1. vectors"
"$here/scripts/check-sysw-mr-vectors.sh" || bad "check-sysw-mr-vectors.sh"

echo "== 2. spec-check and structure check, against $base"
git -C "$here" show "$base:design/SPEC_systemwide_payloads.md" > "$work/base.md" || bad "cannot read the base spec"
norm() { sed -E 's/line [0-9]+/line N/g; s/first at line N/first at line N/g; s/:[0-9]+(-[0-9]+)?/:N/g' | grep -E 'FAIL|WARN' | sort; }
for tool in spec-check structure; do
  for which in base new; do
    f=$([ $which = base ] && echo "$work/base.md" || echo "$spec")
    if [ $tool = spec-check ]; then python3 "$here/scripts/spec-check.py" "$f" > "$work/$tool-$which.txt" 2>&1
    else bash "$here/scripts/spec-structure-check.sh" "$f" > "$work/$tool-$which.txt" 2>&1; fi
    norm < "$work/$tool-$which.txt" > "$work/$tool-$which.norm"
  done
  added=$(comm -13 "$work/$tool-base.norm" "$work/$tool-new.norm")
  echo "$tool: base $(wc -l < "$work/$tool-base.norm") problem line(s), amended $(wc -l < "$work/$tool-new.norm")"
  [ -z "$added" ] || { bad "$tool: problem lines the amendment adds:"; printf '%s\n' "$added"; }
done

echo "== 3. quote checks"
s14=$(sed -n '/^## 14\. /,$p' "$spec")
[ -n "$s14" ] || bad "no §14"
for code in $(python3 -I -c "
import json,sys;d=json.load(open(sys.argv[1]))
c=set()
for k in ('string_vectors','set_vectors','kit_vectors'):
  for v in d[k]:
    if v.get('reason'): c.add(v['reason'])
print(' '.join(sorted(c)))" "$vec"); do
  printf '%s' "$s14" | grep -q "\`$code\`" || bad "reason $code is not in §14"
  n=$(printf '%s\n' "$s14" | grep -c "^| \`$code\` | ")
  [ "$n" -ge 1 ] || bad "reason $code has no row in §14's tables"
done
for code in MR_CASE MR_CHARACTER MR_LENGTH MR_CHECKSUM MR_RESERVED_SYMBOL MR_CHUNK_SHORT MR_CHUNK_PADDING SET_COUNT SET_INDEX SET_DUPLICATE SET_MISSING SET_SLICES SET_ID CARD_SPLIT KIT_OTHER_RECORD KIT_UNCONFIRMED_MDMK; do
  n=$(sed -n '/^### 14g\. /,/^### 14h\. /p' "$spec" | grep -c "^| \`$code\` | ")
  [ "$n" -eq 1 ] || bad "§14g has $n sentence rows for $code (want 1)"
done
for name in $(python3 -I -c "
import json,sys;d=json.load(open(sys.argv[1]))
print(' '.join(v['name'] for k in ('string_vectors','set_vectors','kit_vectors') for v in d[k]))" "$vec"); do
  sed -n '/^### 14i\. /,/^### 14j\. /p' "$spec" | grep -q "\`$name\`" || bad "vector $name is not in §14i's table"
done
for t in 24 25 26 27 28 29 30 31 32; do
  sed -n '/^### 8\.3 /,/^## 9\. /p' "$spec" | grep -q "^$t\. " || bad "§8.3 has no test $t"
done
while IFS= read -r q; do
  printf '%s' "$s14" | tr '\n' ' ' | sed 's/  */ /g' | grep -qF "$q" || bad "screen text not in §14: $q"
done <<'EOF'
Engrave kit now (N cards, M plates)?
Kit Refused
Nothing from this payload can be engraved as a kit. Write the kit payload again.
N cards, M plates. Start at:
Card k of N: <kind>, begins <begins>, ends <ends>
Card k of N | Plate p of q
Choose engraving
Stopped before plate p of q of card k of N. To finish this card, choose card k in the list again: it starts again at its plate 1.
Cards k to N of this kit are cut.
Card N of this kit is cut.
Card k of N, plate p of q does not fit on a plate, so this kit cannot be engraved on this machine.
Refugium build: secrets travel only in the Sealed Payload, never in this one. Unload this payload unless you mean to engrave its secret now.
me: this payload is not a kit the device will engrave:
EOF

if [ $fail -ne 0 ]; then echo "GATE RED: $fail failure(s)"; exit 1; fi
echo "GATE GREEN"
