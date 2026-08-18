# Push record: seedhammer fork `main` — S6b pre-flash cycle merge

**Repo**: `/scratch/code/shibboleth/seedhammer`, branch `main`, remote `origin` = `git@github.com:bg002h/seedhammer.git`
**Date**: 2026-08-18 (push landed 2026-08-18T15:34:xxZ; local session started 2026-08-17)

## Pre-push state

- `git rev-parse HEAD` before push: `b4bbe613c6d669424e23f5a7b8ac69c9c4881977`
- `git status --porcelain`: empty (clean working tree)
- `origin/main` (after `git fetch origin`): `b1479a1b38f6b045d27443764c858906e4e6e122` — matches the expected pre-push tip from the brief.
- `main` branch protection: not re-verified this session (brief states 404/unprotected; no `ci/staging` ritual used, consistent with that).

### Commit-count note (discrepancy vs. brief)

The brief described the push as "22 commits ahead ... a `--no-ff` merge of `s6b-pre-flash` (21 commits) plus one CI commit." The actual count is:

```
$ git rev-list --count origin/main..HEAD
23
```

Breakdown, confirmed via `git log --merges --oneline origin/main..HEAD`:
- 21 commits from the `s6b-pre-flash` branch (`c95dd23` .. `1cec141`)
- 1 merge commit itself: `df75314` ("S6b: the pre-flash cycle -- the device stops saying things that are not true")
- 1 CI commit: `b4bbe61` ("ci: raise the test timeout to 20m -- gui was at 79% of Go's 600s default")

21 + 1 (merge) + 1 (CI) = 23. The brief's "22" appears to have not counted the merge commit as a commit in its own right. This is a description discrepancy only — `origin/main`'s pre-push SHA, the working tree, and the commit content all matched expectations; the push proceeded.

## Push

```
$ git push origin main
To github.com:bg002h/seedhammer.git
   b1479a1..b4bbe61  main -> main
```

No "Bypassed rule violations" message — consistent with `main` being unprotected on this fork.

## Landing confirmation

```
$ git ls-remote origin refs/heads/main
b4bbe613c6d669424e23f5a7b8ac69c9c4881977	refs/heads/main
```

Matches the recorded pre-push `HEAD` exactly.

## CI runs triggered on `b4bbe613c6d669424e23f5a7b8ac69c9c4881977`

| Workflow | Run ID | URL | Status | Conclusion |
|---|---|---|---|---|
| Test | 32155309524 | https://github.com/bg002h/seedhammer/actions/runs/32155309524 | completed | success |
| Build image | 32155309240 | https://github.com/bg002h/seedhammer/actions/runs/32155309240 | completed | success |

### Per-job conclusions (via `gh api repos/bg002h/seedhammer/commits/<sha>/check-runs`, filtered to `status == "completed"`)

```
tests                | status=completed conclusion=success | started=2026-08-18T15:34:51Z completed=2026-08-18T15:42:23Z
tinygo-device-build  | status=completed conclusion=success | started=2026-08-18T15:34:53Z completed=2026-08-18T15:41:07Z
build                | status=completed conclusion=success | started=2026-08-18T15:34:52Z completed=2026-08-18T15:39:52Z
```

3 of 3 check-runs completed with conclusion `success`. No job was skipped, cancelled, or failed. Cross-checked against `gh run view --json jobs` for each run individually — same three jobs, same conclusions, same timestamps.

- `tests` job (Test workflow): 15:34:51Z → 15:42:23Z = 7m32s wall time.
- `tinygo-device-build` job (Test workflow): 15:34:53Z → 15:41:07Z = 6m14s wall time.
- `build` job (Build image workflow): 15:34:52Z → 15:39:52Z = 5m00s wall time.

## `gui` package duration (the item under specific scrutiny)

Extracted from the `tests` job's raw log (`gh run view --job 95770973262 --repo bg002h/seedhammer --log`), the `go test -timeout 20m ./...` step:

```
ok  	seedhammer.com/gui	386.065s
```

**386.065s** on this runner, against the 20-minute (1200s) timeout raised in commit `b4bbe61`. No timeout occurred; the job completed with margin (1200s − 386s ≈ 814s / 68% headroom against the `gui` package alone, and the whole `tests` job finished in 7m32s total). This run's 386.065s is somewhat lower than the 476.3s measurement cited in the brief as motivating the timeout bump — consistent with runner-to-runner variance, not a discrepancy requiring action.

No `FAIL`, `panic`, or timeout-triggered abort anywhere in the `tests` job log (checked via `grep -iE "FAIL|panic|timeout|error"`, the only "timeout" hits being the literal `-timeout 20m` flag in the step's own command echo). Log ends cleanly at `Complete job` / `Cleaning up orphan processes`.

## Verdict

**GREEN**

- Push landed at the exact recorded SHA (`ls-remote` match).
- No "Bypassed rule violations" (none expected — `main` unprotected on this fork).
- Both triggered workflows (`Test`, `Build image`) completed, run-level conclusion `success`.
- All 3 underlying jobs (`tests`, `tinygo-device-build`, `build`) independently confirmed `completed` / `success` — no job-level skip or failure hidden behind a run-level pass.
- The `gui` package, the specific risk this cycle's CI change was meant to cover, ran to completion in 386.065s, well inside the new 20-minute timeout, with no failure.
