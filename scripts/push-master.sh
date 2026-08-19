#!/usr/bin/env bash
# push-master.sh — the ci/staging ritual as ONE command, with tiny output.
#
# WHY THIS EXISTS. Branch protection requires status contexts, and a status
# check binds to a COMMIT SHA, not a branch. A commit pushed straight to the
# default branch therefore carries no check when the rule is evaluated, reports
# "expected", and is BYPASSED. `strict: false` is what makes it fixable:
# GitHub asks only whether the commit carries a passing context, so the SHA has
# to earn one first — hence the staging branch.
#
# This was previously run by dispatching a subagent, which cost ~45k tokens to
# execute four commands. A script costs one Bash call and prints one line.
#
# It also enforces the FREEZE the ritual assumes: the branch must not move
# between the staging push and the final push. A prior incident staged one SHA,
# had two more commits land while CI ran, and pushed a tip two commits past the
# gated one — `strict: false` accepted it against the older gated ancestor and
# printed "Bypassed rule violations". Two commits reached the remote with zero
# CI signal.
#
# REPO-AGNOSTIC. Repo, branch and the REQUIRED CONTEXTS are all discovered from
# the remote and the protection API, so this works from any constellation repo
# that builds `ci/**` — `mnemonic-engrave` and `descriptor-mnemonic` both do,
# and both document the ritual in their own workflow. Nothing here is
# hardcoded to one of them.
#
# Usage:  ./scripts/push-master.sh            # run the ritual
#         ./scripts/push-master.sh --verbose  # show CI progress too
set -uo pipefail

STAGE="ci/staging"
VERBOSE=0
[ "${1:-}" = "--verbose" ] && VERBOSE=1

say()  { printf '%s\n' "$*" >&2; }
die()  { printf 'PUSH FAILED: %s\n' "$*" >&2; cleanup; exit 1; }
cleanup() { git push origin --delete "$STAGE" >/dev/null 2>&1 || true; }

cd "$(git rev-parse --show-toplevel)" || exit 1

# --- discover repo + branch ----------------------------------------------
ORIGIN="$(git remote get-url origin 2>/dev/null)" || die "no origin remote"
REPO="$(printf '%s' "$ORIGIN" | sed -E 's#\.git$##; s#^.*[:/]([^/]+/[^/]+)$#\1#')"
case "$REPO" in
  */*) : ;;
  *)   die "could not derive owner/repo from $ORIGIN (got '$REPO')" ;;
esac
BRANCH="$(git rev-parse --abbrev-ref HEAD)"

# --- preconditions --------------------------------------------------------
[ -z "$(git status --porcelain)" ] || die "working tree is dirty; commit or stash first"

# --- CITATION GATE on changed docs ---------------------------------------
# Decided 2026-08-18 (design/agent-reports/decision-item8-item9.md, item 8).
#
# The gate lives HERE rather than in CI for a semantic reason, not a cost one:
# CI would resolve citations against sibling repos' ORIGIN HEAD, while authors
# write docs against their LOCAL sibling state — often in the same session as
# unpushed sibling work. Push-ordering races would then turn the gate red on
# CORRECT docs, and a gate that reds on correct work trains the reader to
# ignore it exactly as fast as one that is always green. The local roots are,
# by construction, the state the doc was written against.
#
# Scope is CHANGED DOCS ONLY — "a fold is authorship and re-earns the gate".
# The 201 + 633 doc corpus is not cleaned in a campaign; it converges one doc
# at a time as docs are touched.
#
# `design/agent-reports/` is excluded PERMANENTLY. Reports are persisted
# verbatim and never edited afterwards, so a red gate on a report would demand
# a forbidden edit — a dangling citation inside a report is information about
# the review, not a defect to fix.
if [ -x scripts/plan-cite-check.sh ]; then
  DOCS="$(git diff --name-only --diff-filter=d "origin/$BRANCH..HEAD" -- \
            'design/*.md' 'design/**/*.md' ':(exclude)design/agent-reports/**' 2>/dev/null)"
  if [ -n "$DOCS" ]; then
    say "citation gate: $(printf '%s\n' "$DOCS" | wc -l | tr -d ' ') changed doc(s) (agent-reports excluded)"
    # shellcheck disable=SC2086
    ./scripts/plan-cite-check.sh $DOCS >&2 \
      || die "citation gate failed on changed docs — fix the citations or qualify the paths"
  fi
fi

SHA="$(git rev-parse HEAD)"                      # full 40-char; gh needs it
N="$(git rev-list --count "origin/$BRANCH..HEAD" 2>/dev/null || echo '?')"
[ "$N" = "0" ] && { say "nothing to push"; exit 0; }

# --- which contexts does protection actually require? ---------------------
# Asked, not assumed: the two repos require different ones.
mapfile -t CONTEXTS < <(gh api "repos/$REPO/branches/$BRANCH/protection" \
  --jq '.required_status_checks.contexts[]?' 2>/dev/null)
# gh prints a JSON error BODY on 404 and still exits 0 here, so "non-empty" is
# not the same as "these are contexts" — an earlier version of this script
# happily staged a push with `{"message":"Not Found"...}` as its context list.
[ "${#CONTEXTS[@]}" -gt 0 ] \
  || die "could not read required contexts for $REPO@$BRANCH (gh returns empty silently — check auth/repo)"
for c in "${CONTEXTS[@]}"; do
  case "$c" in
    '{'*|'['*|'') die "protection API did not return contexts for $REPO@$BRANCH; got: $c" ;;
  esac
done

say "staging $SHA ($N commit(s)) → $REPO@$BRANCH; requires: ${CONTEXTS[*]}"

# --- stage: let the SHA earn its contexts ---------------------------------
git push -f origin "$BRANCH:refs/heads/$STAGE" >/dev/null 2>&1 \
  || die "could not push $STAGE"

# --- find the run for THIS sha (gh returns empty silently; retry) ---------
RUNID=""
for _ in $(seq 1 40); do
  RUNID="$(gh run list --repo "$REPO" --commit "$SHA" \
            --json databaseId --jq '.[0].databaseId' 2>/dev/null)"
  [ -n "$RUNID" ] && [ "$RUNID" != "null" ] && break
  sleep 5
done
[ -n "$RUNID" ] && [ "$RUNID" != "null" ] || die "no CI run appeared for $SHA"

if [ "$VERBOSE" = "1" ]; then
  gh run watch "$RUNID" --repo "$REPO" >&2 || true
else
  gh run watch "$RUNID" --repo "$REPO" >/dev/null 2>&1 || true
fi

# --- judge EVERY required context, PER JOB, never the run-level status ----
# A repo may require several contexts and they may live in different runs, so
# each is resolved across all runs for this SHA rather than assumed present.
for ctx in "${CONTEXTS[@]}"; do
  conc=""
  for rid in $(gh run list --repo "$REPO" --commit "$SHA" --json databaseId --jq '.[].databaseId' 2>/dev/null); do
    c="$(gh run view "$rid" --repo "$REPO" --json jobs \
          --jq ".jobs[] | select(.name == \"$ctx\") | .conclusion" 2>/dev/null)"
    [ -n "$c" ] && { conc="$c"; break; }
  done
  [ -n "$conc" ] || die "required context '$ctx' not found in any run for $SHA (empty != absent — check manually)"
  [ "$conc" = "success" ] || die "'$ctx' concluded '$conc' — see https://github.com/$REPO/actions/runs/$RUNID"
done

# --- FREEZE check: the tip must not have moved ----------------------------
NOW="$(git rev-parse HEAD)"
[ "$NOW" = "$SHA" ] || die "$BRANCH moved during CI ($SHA -> $NOW); re-run to stage the new tip"

# --- push for real; a bypass line is a FAILURE, not a success -------------
OUT="$(git push origin "$BRANCH" 2>&1)"
if printf '%s' "$OUT" | grep -qi 'bypass'; then
  printf '%s\n' "$OUT" >&2
  die "push printed a bypass line — the staged contexts did not apply"
fi

cleanup
say "PUSHED $N commit(s) to $REPO@$BRANCH — ${#CONTEXTS[@]} context(s) success — run $RUNID — no bypass"
