#!/usr/bin/env bash
#
# fold-propagation-check.sh -- after folding a review finding, prove the
# superseded claim is gone from the WHOLE artifact, not just the sentence the
# reviewer pointed at.
#
# WHY THIS EXISTS
#   Three of six folds in the multisig-build-repair cycle were defective, all
#   the same way: the facts were RIGHT and the propagation was INCOMPLETE. A
#   claim was corrected where the reviewer pointed and left standing elsewhere.
#
#   The worst instance: S3's gate was rewritten to require `grep TYPED-ONLY`
#   return 0, while the Implementation bullet ONE LINE ABOVE it still said "the
#   four TYPED-ONLY comments" and named 4 of 9 real sites. An implementer
#   following the bullet would fix four, leave five, and fail the gate written
#   directly beneath. Unbuildable as specified -- and it took a review round to
#   find something a grep finds instantly.
#
#   Operator ruling (2026-08-13): folds stay with the author, with a mandatory
#   build gate AND this cheap propagation check every time. This script is the
#   second half of that ruling, so it is a command rather than a thing to
#   remember.
#
# USAGE
#   scripts/fold-propagation-check.sh <artifact> <superseded-pattern>...
#
#   Each pattern is an ERE matched case-insensitively against the artifact.
#   Give it the phrasings the OLD claim used -- not the new one.
#
#   $ scripts/fold-propagation-check.sh design/PLAN.md \
#         'four .?TYPED-ONLY' 'with BIP-382 vectors'
#
# EXIT
#   0  no superseded phrasing survives
#   1  at least one does; every hit is printed with its line number
#
# WHAT IT DOES NOT DO
#   It cannot invent the patterns for you. It checks the phrasings you thought
#   of, so a claim you did not think to search for survives silently -- the same
#   blind spot plan-cite-gate.sh names in its own header. Two habits close most
#   of it: search for the OLD VALUE (`four`, `382`) rather than the topic, and
#   search for the topic word alone (`TYPED-ONLY`, `BIP-38`) to see every site
#   that mentions it, then read those sites rather than trusting the count.
#
#   It also cannot tell you a surviving mention is WRONG. A historical note
#   ("an earlier draft said 382; that was incorrect") is a legitimate hit. Read
#   what it prints; do not just count.
set -u

ART="${1:?usage: fold-propagation-check.sh <artifact> <superseded-pattern>...}"
shift
[ $# -gt 0 ] || { echo "no patterns given -- nothing to check" >&2; exit 2; }
[ -f "$ART" ] || { echo "no such artifact: $ART" >&2; exit 2; }

fail=0
echo "== propagation check: $(basename "$ART") =="
for pat in "$@"; do
  hits=$(grep -nEi -- "$pat" "$ART" || true)
  if [ -z "$hits" ]; then
    printf '  gone   %s\n' "$pat"
  else
    fail=1
    printf '  LEFT   %s\n' "$pat"
    printf '%s\n' "$hits" | sed 's/^/           /'
  fi
done

echo
if [ "$fail" -eq 0 ]; then
  echo "   no superseded phrasing survives"
  echo "   NOT covered: a phrasing you did not think to search for, and whether"
  echo "   a surviving mention is a legitimate historical note. Read, don't count."
else
  echo "   SUPERSEDED PHRASING SURVIVES -- the fold is not finished."
  echo "   Each hit above is either a site that still needs the correction, or a"
  echo "   deliberate historical note. Decide which, per hit, before committing."
fi
exit "$fail"
