#!/usr/bin/env bash
# push-via-staging.sh -- the ci/staging push ritual as a command.
#
# The branch-protection context binds to a SHA, so the SHA earns its check on
# ci/staging first, then the branch push satisfies the rule instead of
# bypassing it. This script IS the ritual; running it is the discipline.
#
#   scripts/push-via-staging.sh            # pushes current branch
#   scripts/push-via-staging.sh master     # explicit branch
#
# Gates the final push on the REQUIRED context only (operator direction
# 2026-08-28); non-required jobs keep running and are reported afterwards.
# FREEZE: no commits to the branch between invocation and completion.
set -euo pipefail
BRANCH="${1:-$(git rev-parse --abbrev-ref HEAD)}"
REQUIRED_CONTEXT="${REQUIRED_CONTEXT:-test (rust + go)}"
TIP=$(git rev-parse HEAD)
echo "== staging $TIP (branch $BRANCH, $(git rev-list --count "origin/$BRANCH"..HEAD 2>/dev/null || echo '?') ahead)"
git push origin "HEAD:refs/heads/ci/staging"
RUN_ID=""
for _ in $(seq 1 30); do
  RUN_ID=$(gh run list --commit "$TIP" --json databaseId -q '.[0].databaseId' 2>/dev/null || true)
  [ -n "$RUN_ID" ] && break
  sleep 10   # an empty gh result can be a race, never a conclusion
done
[ -n "$RUN_ID" ] || { echo "FATAL: no workflow run appeared for $TIP"; exit 1; }
echo "== run $RUN_ID; waiting for required context: $REQUIRED_CONTEXT"
CONC=""
for _ in $(seq 1 120); do
  CONC=$(gh run view "$RUN_ID" --json jobs -q ".jobs[] | select(.name==\"$REQUIRED_CONTEXT\") | .conclusion // empty" 2>/dev/null || true)
  case "$CONC" in
    success) break ;;
    failure|cancelled|timed_out|action_required)
      echo "FATAL: required job concluded '$CONC' -- NOT pushing $BRANCH"; exit 1 ;;
    *) sleep 10 ;;
  esac
done
[ "$CONC" = "success" ] || { echo "FATAL: timed out waiting for the required context"; exit 1; }
[ "$(git rev-parse HEAD)" = "$TIP" ] || { echo "FATAL: tip moved during the window -- re-stage the new tip"; exit 1; }
OUT=$(git push origin "HEAD:$BRANCH" 2>&1); echo "$OUT"
if echo "$OUT" | grep -qi "bypassed rule violations"; then
  echo "FATAL: bypass message detected -- staging ref left in place for forensics"; exit 1
fi
git push origin --delete ci/staging
echo "== post-push straggler report (non-required jobs, informational):"
gh run view "$RUN_ID" --json jobs -q '.jobs[] | .name + ": " + (.conclusion // .status)' || true
echo "== OK: $TIP is on $BRANCH with the required check earned"
