**Haiku transcription fold: STOPPED before editing (no edits, no commit; worktree clean at f7ed8339).**

(Recorded verbatim by the coordinator from the agent's return; the agent did not write this file.)

Checks that passed:
- `s3-me.diff` (sha256 5f719fba…da612) applies with `git apply --check` rc=0 in the me worktree.
- `s3-fork.diff` (sha256 b9d95f1d…8fb8) applies forward with rc=0 in `/scratch/code/shibboleth/sh-worktrees/bughunt-triage-s3`.
- The probe dir `/scratch/cache/bughunt/sh2/architect-stops-probe/fx3-s3` is not a git repo, so applying there fails.
- `gui/composer_selfcheck_test.go:150` in `/scratch/code/shibboleth/sh-worktrees/bughunt-triage-s2` (head 1df27de) contains "without two distinct fingerprints"; the ruling's S2 anchor is `:167`.

STOP points:
1. §3 row for `cmd/controller/main_host.go` has no ruling.
2. §2.14 insertion point is not given.
3. §1 line 89 wording is not given.
4. Dangling STOP references in the FIX doc at lines 21, 64, 95, 158 ("STOP 3(b)") and 217 are not covered by S1–S4.
5. M-S2a disposition is not given.
6. The §2.5 lead-in and the S2 line reference (`:150` or `:167`) need a ruling.
7. The §2.13 30d replacement for "Relative repeated `s` is not covered (STOP 1)." and the S1 row/subtest text need exact wording checked against the ruling (mapping not finished).
8. The patch copies to `design/FIX_bughunt_sh2-s3-me.diff` and `design/FIX_bughunt_sh2-s3-fork.diff` are not done.

counts: 0 C, 0 I, 8 STOP
