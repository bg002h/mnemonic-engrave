#!/usr/bin/env bash
# plan-fold-sweep.sh — after a fold, find terms the fold REMOVED that are still
# standing somewhere else in the document.
#
# WHY THIS EXISTS. Measured over one plan: SEVEN consecutive folds carried a
# defect, and the dominant class every time was INCOMPLETE PROPAGATION — the
# facts were corrected where the reviewer pointed and left standing three
# sections away. The worst instance renamed a verdict in the design section and
# not in the test table, so the test billed as "the sharpest test in this plan"
# ended up asserting the exact OPPOSITE of the property the same fold introduced.
#
# READING THE DIFF CANNOT FIND THIS. By construction the defect lives in the text
# the diff did NOT touch. Only a sweep of the whole file finds it, and a sweep
# that depends on remembering to sweep is what failed seven times.
#
# USAGE — the PRIMARY mode is explicit, because inference does not work here
#   ./scripts/plan-fold-sweep.sh <doc> --terms 'old term' 'another'
#   ./scripts/plan-fold-sweep.sh <doc> [base-ref]     # inference; SEE THE WARNING
#
# THE EXPLICIT MODE IS THE ONE THAT WORKS. The fold author knows what they
# superseded; naming it is the discipline, and the script makes the check
# unskippable and its output unreadable-past.
#
# EXIT
#   0  nothing the fold removed still survives elsewhere
#   1  at least one removed term still appears — READ EACH ONE, they are
#      candidates, not verdicts (a term may survive legitimately)
#
# ─── WHAT THIS DOES NOT COVER ────────────────────────────────────────────────
#   * It flags CANDIDATES, not defects. A removed term can legitimately survive
#     (quoting today's code, or naming the defect being fixed). Every hit needs a
#     human read — the point is that you cannot skip looking.
#   * It only sees terms it can tokenise: backticked identifiers and quoted
#     phrases. A superseded idea expressed in different prose each time is
#     invisible to it.
#   * It cannot judge SEMANTIC staleness — a sentence that is now false without
#     containing any removed token will pass clean.
#   * INFERENCE MODE IS WEAK, and this was MEASURED, not assumed. Run against the
#     R4 fold -- which demonstrably left `verifyFailed` standing in the test table
#     -- it reported "nothing to sweep". The fold's own new section DISCUSSED
#     `verifyFailed` at length, so the token appeared on both removed and added
#     lines and was subtracted out. A fold whose subject IS the renamed term is
#     exactly the case inference cannot see, and exactly the case that matters.
#     Use --terms. Inference is a backstop for folds that rename in passing.

set -uo pipefail

doc="${1:-}"
shift || true

if [ -z "$doc" ] || [ ! -f "$doc" ]; then
  echo "usage: $0 <doc.md> --terms 'old' ['old2' ...]" >&2
  echo "       $0 <doc.md> [base-ref]      # inference mode, weak -- see header" >&2
  exit 2
fi

# ─── EXPLICIT MODE ───────────────────────────────────────────────────────────
if [ "${1:-}" = "--terms" ]; then
  shift
  [ "$#" -gt 0 ] || { echo "--terms needs at least one term" >&2; exit 2; }
  echo "═══ fold sweep (explicit): $doc"
  echo "─── $# superseded term(s) named by the fold author"
  echo
  hits=0
  for bare in "$@"; do
    c=$(grep -cF -- "$bare" "$doc" || true); : "${c:=0}"
    if [ "$c" -gt 0 ]; then
      hits=$((hits + 1))
      printf 'STILL PRESENT  %-46s  %s occurrence(s)\n' "$bare" "$c"
      grep -nF -- "$bare" "$doc" | sed 's/^/                 /' | cut -c1-118
    else
      printf 'gone           %-46s\n' "$bare"
    fi
  done
  echo
  if [ "$hits" -eq 0 ]; then
    echo "─── clean: no named superseded term survives"
  else
    echo "─── $hits term(s) still present. READ EVERY LINE ABOVE."
    echo "─── They are CANDIDATES, not verdicts: a term may legitimately survive by"
    echo "─── quoting today's code or by naming the defect being fixed."
  fi
  echo "─── NOT covered: terms you did not name, prose-only supersessions,"
  echo "─── and semantic staleness that uses none of the named tokens."
  [ "$hits" -eq 0 ]
  exit
fi

base="${1:-HEAD~1}"
echo "!!! INFERENCE MODE IS WEAK — it misses any fold whose subject IS the renamed"
echo "!!! term (measured: it reported clean on a fold that left one standing)."
echo "!!! Prefer:  $0 $doc --terms 'old term' ..."
echo

diff_out=$(git diff "$base" -- "$doc" 2>/dev/null)
if [ -z "$diff_out" ]; then
  echo "no changes to $doc since $base — nothing to sweep"
  exit 0
fi

echo "═══ fold sweep: $doc (vs $base)"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# Tokens on REMOVED lines and on ADDED lines. A token that left the added set
# entirely is what the fold superseded.
printf '%s\n' "$diff_out" | grep '^-' | grep -v '^---' \
  | grep -oE '`[^`]{3,60}`' | sort -u > "$TMP/removed"
printf '%s\n' "$diff_out" | grep '^+' | grep -v '^+++' \
  | grep -oE '`[^`]{3,60}`' | sort -u > "$TMP/added"

comm -23 "$TMP/removed" "$TMP/added" > "$TMP/superseded"

n=$(wc -l < "$TMP/superseded" | tr -d ' ')
if [ "$n" -eq 0 ]; then
  echo "─── the fold superseded no tokenisable terms; nothing to sweep"
  echo "─── NOT covered: prose-only supersessions, semantic staleness."
  exit 0
fi

echo "─── $n term(s) removed by this fold; checking whether each still survives"
echo

hits=0
while IFS= read -r term; do
  [ -z "$term" ] && continue
  bare=${term//\`/}
  # Count survivors in the CURRENT file.
  c=$(grep -cF -- "$bare" "$doc" || true)
  : "${c:=0}"
  if [ "$c" -gt 0 ]; then
    hits=$((hits + 1))
    printf 'STILL PRESENT  %-46s  %s occurrence(s)\n' "$term" "$c"
    grep -nF -- "$bare" "$doc" | head -4 | sed 's/^/                 /' | cut -c1-118
  fi
done < "$TMP/superseded"

echo
if [ "$hits" -eq 0 ]; then
  echo "─── clean: no superseded term survives elsewhere in the document"
else
  echo "─── $hits superseded term(s) still present. READ EVERY ONE."
  echo "─── They are CANDIDATES, not verdicts: a term may legitimately survive by"
  echo "─── quoting today's code or by naming the defect being fixed."
fi
echo "─── NOT covered: prose-only supersessions, semantic staleness, and renames"
echo "─── where the old and new spellings both appear on changed lines."

[ "$hits" -eq 0 ]
