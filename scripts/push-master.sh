#!/usr/bin/env bash
# push-master.sh — the ci/staging ritual as ONE command, with tiny output.
#
# WHY THIS EXISTS. Branch protection requires the `test (rust + go)` context,
# and a status check binds to a COMMIT SHA, not a branch. A commit pushed
# straight to master therefore carries no check when the rule is evaluated,
# reports "expected", and is BYPASSED. `strict: false` is what makes it
# fixable: GitHub asks only whether the commit carries a passing context, so
# the SHA has to earn one first — hence the staging branch.
#
# This was previously run by dispatching a subagent, which cost ~45k tokens to
# execute four commands. A script costs one Bash call and prints one line.
#
# It also enforces the FREEZE that the ritual assumes: master must not move
# between the staging push and the final push. A prior incident staged one SHA,
# had two more commits land while CI ran, and pushed a tip two commits past the
# gated one — `strict: false` accepted it against the older gated ancestor and
# printed "Bypassed rule violations". Two commits reached origin/master with
# zero CI signal.
#
# Usage:  ./scripts/push-master.sh            # run the ritual
#         ./scripts/push-master.sh --verbose  # show CI progress too
set -uo pipefail

REPO="bg002h/mnemonic-engrave"
CTX="test (rust + go)"
STAGE="ci/staging"
VERBOSE=0
[ "${1:-}" = "--verbose" ] && VERBOSE=1

say()  { printf '%s\n' "$*" >&2; }
die()  { printf 'PUSH FAILED: %s\n' "$*" >&2; cleanup; exit 1; }
cleanup() { git push origin --delete "$STAGE" >/dev/null 2>&1 || true; }

cd "$(git rev-parse --show-toplevel)" || exit 1

# --- preconditions --------------------------------------------------------
[ -z "$(git status --porcelain)" ] || die "working tree is dirty; commit or stash first"
[ "$(git rev-parse --abbrev-ref HEAD)" = "master" ] || die "not on master"

SHA="$(git rev-parse HEAD)"                      # full 40-char; gh needs it
N="$(git rev-list --count origin/master..HEAD 2>/dev/null || echo '?')"
[ "$N" = "0" ] && { say "nothing to push"; exit 0; }
say "staging $SHA ($N commit(s))"

# --- stage: let the SHA earn its context ----------------------------------
git push -f origin "master:refs/heads/$STAGE" >/dev/null 2>&1 \
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

# --- judge PER-JOB, never the run-level status ----------------------------
CONC="$(gh run view "$RUNID" --repo "$REPO" --json jobs \
        --jq ".jobs[] | select(.name == \"$CTX\") | .conclusion" 2>/dev/null)"
[ -n "$CONC" ] || die "job '$CTX' not found in run $RUNID (empty != absent — check manually)"
[ "$CONC" = "success" ] || die "'$CTX' concluded '$CONC' — see https://github.com/$REPO/actions/runs/$RUNID"

# --- FREEZE check: the tip must not have moved ----------------------------
NOW="$(git rev-parse HEAD)"
[ "$NOW" = "$SHA" ] || die "master moved during CI ($SHA -> $NOW); re-run to stage the new tip"

# --- push for real; a bypass line is a FAILURE, not a success -------------
OUT="$(git push origin master 2>&1)"
if printf '%s' "$OUT" | grep -qi 'bypass'; then
  printf '%s\n' "$OUT" >&2
  die "push printed a bypass line — the staged context did not apply"
fi

cleanup
say "PUSHED $N commit(s) — $CTX: success — run $RUNID — no bypass"
