#!/usr/bin/env bash
# verify-returnsite-sweep.sh — every verdict return site in the verify flows must
# appear in the plan's observation table. P5's row-coverage half, as a command.
#
# WHY THIS EXISTS. Three times the observation table was extended to cover exactly
# the return paths a reviewer had named, and three times the next reviewer found
# paths nobody had named. The author performed an exhaustive per-site analysis for
# ONE verdict — because a review named it — and never repeated it for the others;
# `verifyIncomplete` turned out to have three non-uniform sites, unexamined, two
# of which printed a confident line on an adverse observation.
#
# "Check every return site" is a discipline, and disciplines are what failed. This
# makes it a command.
#
# It is P5's FIRST half only. The second — that an unrecognised observation maps
# to the fewest-claims line — is enforced by the tested-unreachable default arm
# (plan T17), not by this script. Both are needed: this one proves the table SAW
# every site; the default arm makes being wrong about one of them SAFE.
#
# USAGE
#   ./scripts/verify-returnsite-sweep.sh <plan.md>
#
# EXIT
#   0  every verdict return site appears in the plan
#   1  at least one site is unrowed — READ EACH, then either add a row or state
#      in the plan why it needs none
#
# ─── WHAT THIS DOES NOT COVER ────────────────────────────────────────────────
#   * It proves a site is MENTIONED, never that its row is CORRECT. The world-set
#     W per row is a human judgement and this script cannot check it. R7 found a
#     row whose |W| was wrong while the site was present and cited.
#   * It only sees `return verify*` in the files it scans. A verdict produced by
#     assignment, or in a file not listed, is invisible.
#   * It cannot see paths that reach the document WITHOUT a verdict.
#   * A site can be listed in the plan for an unrelated reason and still count as
#     covered here. Mention is a weak signal; it is simply a much better one than
#     nothing, which is what preceded it.

set -uo pipefail

plan="${1:-}"
if [ -z "$plan" ] || [ ! -f "$plan" ]; then
  echo "usage: $0 <plan.md>" >&2
  exit 2
fi

FORK="/scratch/code/shibboleth/seedhammer"
FILES=("gui/multisig_verify.go" "gui/singlesig_verify.go")

echo "═══ verdict return-site sweep vs $plan"
echo

total=0
missing=0

for f in "${FILES[@]}"; do
  path="$FORK/$f"
  [ -f "$path" ] || { echo "SKIP (no such file): $f"; continue; }
  # Every line returning a verify* verdict.
  while IFS= read -r hit; do
    n=${hit%%:*}
    text=$(printf '%s' "${hit#*:}" | sed 's/^[[:space:]]*//')
    total=$((total + 1))
    # Covered if the plan cites this file:line anywhere.
    if grep -qF -- "$f:$n" "$plan"; then
      printf 'rowed    %-34s  %s\n' "$f:$n" "$(printf '%s' "$text" | cut -c1-58)"
    else
      missing=$((missing + 1))
      printf 'UNROWED  %-34s  %s\n' "$f:$n" "$(printf '%s' "$text" | cut -c1-58)"
    fi
  done < <(grep -nE 'return[[:space:]]+verify[A-Z]' "$path" || true)
done

echo
echo "─── verdict return sites: $total ; unrowed in the plan: $missing"
echo "─── NOT covered: whether each row's WORLD-SET is right (a human judgement,"
echo "─── and R7 found a wrong one on a site that WAS cited); verdicts produced by"
echo "─── assignment; files not scanned; and paths reaching the document with no"
echo "─── verdict at all."

[ "$missing" -eq 0 ]
