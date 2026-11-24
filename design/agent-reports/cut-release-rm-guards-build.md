# cut-release rm guards build

Gate: green. Head 008048be. No STOPs.

Commits: 4e13e50a (RED, H1+H7+H8), 008048be (GREEN: S1-S8, C1-C3, H2-H6, F1).

RED (/scratch/cache/cut-release-rm-guards/red.log): seven FAIL lines, "0 passed, 7 failed", exit 1; /ms-cli-v0.20.2 absent.
GREEN (green.log): seven ok, "7 passed, 0 failed", "ran 7 rows, expected 77", exit 1.
Gate: bash -n ok on cut-release.sh, cut-release.test.sh, fake-gh. Offline (offline.log): 77 passed, 0 failed, exit 0. Full (full.log, network reachable): 85 passed, 0 failed.
FOLLOWUPS: the FIX names no owed-to-build line; none closed.

Deviations:
- RED applied only H1, H7, H8 (FIX section 3 and its build order), not all harness entries as the brief's step 2 reads; applying H2-H6 would have made rows 75/76 not fail as section 3 says.
- Commit trailer uses "Claude Sonnet 5.5" (the session attribution) instead of the brief's "Claude Opus 5.5".
- Kept test dirs run-2731565 and run-2734165 (mine) removed; older run-* dirs in ~/.cache/cut-release-test belong to others, untouched.
