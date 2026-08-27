#!/usr/bin/env bash
# plan-table-check.sh — every markdown table row must have the same cell count as
# its header. A dropped cell does not look broken; it silently shifts meaning.
#
# WHY THIS EXISTS. In this plan the tables ARE the design: the 2x2 of recorded
# booleans, the adverse/benign return-site split, the status-to-line map, the
# membership test. A row that loses a cell shifts every value left, so a
# "verdict" column silently reads a "reason" column, and the rendered table looks
# entirely plausible.
#
# It was written after a review found exactly that: a fold editing the membership
# table dropped one cell from the "VERIFIED vs on-repeat" row, and the defect was
# found only because the reviewer thought to count pipes. Nothing else would have
# caught it -- not the citation gate, not the glyph gate, not the fold sweep, and
# not reading it, because a four-column row rendered as three still reads fine.
#
# USAGE
#   ./scripts/plan-table-check.sh <doc.md> [more.md ...]
#
# EXIT
#   0  every table row matches its header's cell count
#   1  at least one row is malformed
#
# ─── WHAT THIS DOES NOT COVER ────────────────────────────────────────────────
#   * It checks SHAPE, never CONTENT. A row with the right number of cells and
#     the wrong values in them passes.
#   * It cannot tell an intentionally empty cell from a lost one -- both are
#     well-formed. It only catches cells that vanished entirely.
#   * A table with no separator row is invisible to it (it takes the separator as
#     the width declaration, which is what markdown does).
#   * Pipes inside code spans or escaped as \| will miscount. This plan has none;
#     a doc that does needs a real markdown parser rather than this.

set -uo pipefail

if [ "$#" -eq 0 ]; then
  echo "usage: $0 <doc.md> [doc.md ...]" >&2
  exit 2
fi

python3 - "$@" <<'PY'
import re, sys

bad = 0
rows = 0
for path in sys.argv[1:]:
    print(f"═══ {path}")
    try:
        lines = open(path, encoding="utf-8").read().split("\n")
    except OSError as e:
        print(f"MISSING DOC: {e}", file=sys.stderr); bad += 1; continue

    width = 0      # pipe count declared by the current table's separator row
    at = 0
    ind = None     # indent of the current table; a change ends it (F-261)
    for n, l in enumerate(lines, 1):
        # INDENTED TABLES COUNT. Markdown nests a table inside a list item by
        # indenting it, and this document does that deliberately to keep a table
        # with the bullet it belongs to. Matching on a bare `|` skipped every
        # such row -- silently, so "0 malformed" was reported over a set that
        # excluded them, and a five-row table added to this repo's own P0 plan
        # moved the count by zero.
        stripped = l.lstrip()
        if not stripped.startswith("|"):
            width, ind = 0, None          # table ended
            continue
        lead = len(l) - len(stripped)
        if width and lead != ind:         # indent shifted: a different table
            width, ind = 0, None
        pipes = stripped.count("|")
        if re.match(r'^\|[\s:\-|]+\|$', stripped):   # the |---|---| separator
            width, at, ind = pipes, n, lead
            continue
        if width:
            rows += 1
            if pipes != width:
                bad += 1
                print(f"MALFORMED  line {n}: {pipes} cells vs {width} declared "
                      f"(table header at line {at})")
                print(f"           {l[:96]}")

print()
print(f"─── table rows checked: {rows} ; malformed: {bad}")
print("─── NOT covered: cell CONTENT, intentionally-empty vs lost cells,")
print("─── tables with no separator row, and pipes inside code spans.")
print("─── COVERED since F-261: indented tables (nested in a list item).")
sys.exit(1 if bad else 0)
PY
