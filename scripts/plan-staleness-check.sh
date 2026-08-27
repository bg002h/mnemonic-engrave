#!/usr/bin/env bash
# plan-staleness-check.sh — has the code moved under a plan since it was written?
#
# WHY THIS EXISTS. A plan's R0 GREEN is earned against a tree at one moment.
# Every later commit — including the plan's OWN earlier rows — can move the lines
# it cites, and nothing else notices. `plan-cite-check.sh` cannot: it asks
# whether a cited line EXISTS, never what is on it, and says so in its own
# "NOT covered" block.
#
# Measured 2026-08-27 on IMPLEMENTATION_PLAN_P1_mt_adopts.md, after its own first
# four rows landed: 14 of 15 mt-cli citations had drifted, and the citation gate
# reported 41/41 resolved with 0 dangling. One drifted citation had come to point
# at a real function the plan cites elsewhere — plausible, wrong, and reachable by
# no lexical or referential check.
#
# USAGE
#   scripts/plan-staleness-check.sh <plan.md> <repo-path> <baseline-rev> [path-prefix]
#
#   plan.md       the plan to check
#   repo-path     the git repo its citations resolve against
#   baseline-rev  the revision the plan was written against
#   path-prefix   optional: only citations starting with this (default: all that
#                 resolve inside repo-path)
#
# EXIT: 0 = no drift. 1 = at least one cited line's CONTENT changed.
#
# WHAT THIS DOES NOT COVER, and a gate that hides its blind spot is worse than no
# gate:
#   * It compares the line at a fixed NUMBER across two revisions. A citation that
#     drifted and happens to land on an identical line (a lone `}`, a blank, a
#     repeated attribute) reads as clean. Prefer citing a symbol beside the number.
#   * It says nothing about whether the ORIGINAL citation was right — only whether
#     it still says what it used to.
#   * Prose that is stale without citing anything is invisible here. That is the
#     "what did this diff falsify elsewhere" lens, and it still needs a reader.
#   * It does not check the plan's CLAIMS about the line, only the line's bytes.
set -uo pipefail

if [ $# -lt 3 ]; then
  sed -n '2,32p' "$0" | sed 's/^# \?//'
  exit 2
fi

PLAN="$1"
REPO="$2"
BASE="$3"
PREFIX="${4:-}"

[ -f "$PLAN" ] || { echo "no such plan: $PLAN" >&2; exit 2; }
[ -d "$REPO/.git" ] || [ -f "$REPO/.git" ] || { echo "not a git repo: $REPO" >&2; exit 2; }

if ! git -C "$REPO" rev-parse --verify --quiet "$BASE^{commit}" >/dev/null; then
  echo "no such revision in $REPO: $BASE" >&2
  exit 2
fi

echo "═══ $PLAN"
echo "─── against $REPO at $BASE .. $(git -C "$REPO" rev-parse --short HEAD)"

# Citations look like path/to/file.ext:1234 . Take the distinct set: a plan
# repeats the same site, and reporting it five times trains a reader to skim.
mapfile -t CITES < <(
  grep -ohE '[A-Za-z0-9_./-]+\.[a-z]+:[0-9]+' "$PLAN" \
    | sort -u
)

same=0; drift=0; skipped=0
for c in "${CITES[@]}"; do
  file="${c%%:*}"
  line="${c##*:}"

  [ -n "$PREFIX" ] && case "$file" in "$PREFIX"*) ;; *) continue ;; esac

  # Only citations that resolve inside THIS repo are ours to judge. A plan cites
  # several repos, and a path missing here is another repo's business, not drift.
  [ -f "$REPO/$file" ] || { skipped=$((skipped + 1)); continue; }

  before=$(git -C "$REPO" show "$BASE:$file" 2>/dev/null | sed -n "${line}p")
  after=$(sed -n "${line}p" "$REPO/$file")

  # A line past the end of the file at EITHER revision is a citation problem, not
  # a drift problem -- plan-cite-check.sh owns that. Do not double-report it.
  if [ -z "$before" ] && [ -z "$after" ]; then
    skipped=$((skipped + 1))
    continue
  fi

  if [ "$before" = "$after" ]; then
    same=$((same + 1))
  else
    drift=$((drift + 1))
    printf 'DRIFT %s\n' "$c"
    printf '   %s: %s\n' "$BASE" "$(printf '%s' "$before" | cut -c1-88)"
    printf '   %s: %s\n' "now  " "$(printf '%s' "$after" | cut -c1-88)"
  fi
done

echo "─── unchanged: $same ; DRIFTED: $drift ; not in this repo: $skipped"
echo "─── NOT covered: whether the citation was ever RIGHT, drift onto an"
echo "───              identical line, and stale prose that cites nothing."
echo "───              Cite the SYMBOL beside the number; this checks bytes."

[ "$drift" -eq 0 ]
