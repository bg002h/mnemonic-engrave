#!/usr/bin/env bash
# plan-wiring-check.sh — every field and constant a plan DECLARES must be used
# somewhere outside its own declaration. Declaring is not wiring.
#
# WHY THIS EXISTS. Twice in one cycle a value was added to a design and never
# connected to anything:
#
#   * `sawUnaccounted` — the sticky bit for the sixth status. The arm that SET it
#     was written; the arm that READ it was not, so the status it existed for was
#     UNPRODUCIBLE and every path routed into it fell through to a milder line.
#   * `suppliedCosigners` — added to fix a Critical about single-sig and multisig
#     being indistinguishable. It appeared three times in the whole document, all
#     inside its own declaration. It would have held Go's zero value on every run,
#     so the clause it existed for would render nowhere, and the Critical would
#     have survived INSIDE ITS OWN FIX.
#
# Both were found by a reviewer noticing a name appeared suspiciously few times.
# That is a count, and counts belong in a script.
#
# USAGE
#   ./scripts/plan-wiring-check.sh <doc.md> [more.md ...]
#
# EXIT
#   0  every declared name is used outside its declaration
#   1  at least one is declared and never used — READ EACH, then either wire it
#      or delete it
#
# ─── WHAT THIS DOES NOT COVER ────────────────────────────────────────────────
#   * It proves a name is MENTIONED elsewhere, never that it is correctly wired.
#     `suppliedCosigners` needed THREE things — written, read, and tested — and a
#     single mention anywhere would satisfy this script. Mention is a weak signal;
#     it is simply much better than zero, which is what preceded it.
#   * It only sees `type X struct { ... }` and `const ( ... )` blocks written in
#     the plan's indented-code style. A field described only in prose is invisible.
#   * It cannot tell a deliberately-reserved name (scaffolding whose emptiness is
#     tested) from a forgotten one. Such names need a comment saying so, and this
#     script will still flag them — which is the correct nuisance.

set -uo pipefail

if [ "$#" -eq 0 ]; then
  echo "usage: $0 <doc.md> [doc.md ...]" >&2
  exit 2
fi

python3 - "$@" <<'PY'
import re, sys

bad = 0
checked = 0
for path in sys.argv[1:]:
    print(f"═══ {path}")
    try:
        text = open(path, encoding="utf-8").read()
    except OSError as e:
        print(f"MISSING DOC: {e}", file=sys.stderr); bad += 1; continue
    lines = text.split("\n")

    # Collect (name, declaration-line-range) for struct fields and const members.
    decls = []
    i = 0
    while i < len(lines):
        l = lines[i]
        if re.search(r'\btype\s+\w+\s+struct\s*\{', l) or re.search(r'\bconst\s*\(', l):
            start = i
            depth_open = l.count("{") + l.count("(") - l.count("}") - l.count(")")
            j = i + 1
            while j < len(lines) and depth_open > 0:
                depth_open += (lines[j].count("{") + lines[j].count("(")
                               - lines[j].count("}") - lines[j].count(")"))
                j += 1
            block = lines[start:j]
            for b in block[1:]:
                b2 = b.split("//")[0]
                m = re.match(r'^\s+([a-z][A-Za-z0-9_]*)\s+\S', b2) or \
                    re.match(r'^\s+([a-z][A-Za-z0-9_]*)\s*$', b2)
                if m:
                    decls.append((m.group(1), start, j))
            i = j
            continue
        i += 1

    for name, lo, hi in decls:
        checked += 1
        outside = sum(1 for n, l in enumerate(lines) if not (lo <= n < hi) and name in l)
        if outside == 0:
            bad += 1
            print(f"DECLARED, NEVER USED   {name!r}   (declared around line {lo+1})")
        else:
            print(f"wired  {name:<24} {outside} mention(s) outside its declaration")

print()
print(f"─── declared names checked: {checked} ; never used elsewhere: {bad}")
print("─── NOT covered: whether a name is CORRECTLY wired (written AND read AND")
print("─── tested); prose-only fields; deliberately-reserved names, which this")
print("─── flags on purpose.")
sys.exit(1 if bad else 0)
PY
