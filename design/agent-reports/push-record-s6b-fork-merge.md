# Push record: seedhammer fork `main` — S6b pre-flash cycle merge

**Repo**: `/scratch/code/shibboleth/seedhammer`, branch `main`, remote `origin` = `git@github.com:bg002h/seedhammer.git`
**Date**: 2026-08-18 (push landed 2026-08-18T15:34:xxZ)

## Pre-push state

- `git rev-parse HEAD` before push: `b4bbe613c6d669424e23f5a7b8ac69c9c4881977`
- `git status --porcelain`: empty (clean working tree)
- `origin/main` (after `git fetch origin`): `b1479a1b38f6b045d27443764c858906e4e6e122` — matches the expected pre-push tip from the brief.
- `main` branch protection: brief states 404/unprotected; no `ci/staging` ritual used, consistent with that.

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
tests                | status=completed conclusion=success | started=2026-08-18T15:34:51Z completed=2026-08-18T15:42:23Z  (7m32s)
tinygo-device-build  | status=completed conclusion=success | started=2026-08-18T15:34:53Z completed=2026-08-18T15:41:07Z  (6m14s)
build                | status=completed conclusion=success | started=2026-08-18T15:34:52Z completed=2026-08-18T15:39:52Z  (5m00s)
```

3 of 3 check-runs completed with conclusion `success`. No job was skipped, cancelled, or failed. Cross-checked against `gh run view --json jobs` for each run individually — same three jobs, same conclusions, same timestamps.

## `gui` package duration — CORRECTION, not a success story

Extracted from the `tests` job's raw log (`gh run view --job 95770973262 --repo bg002h/seedhammer --log`), the `go test -timeout 20m ./...` step:

```
ok  	seedhammer.com/gui	386.065s
```

**386.065s** on the GitHub Actions runner, against the 20-minute (1200s) timeout raised in commit `b4bbe61`.

**This is not vindication of the timeout raise — it is a wrong prediction, and the record must say so plainly.** Commit `b4bbe61`'s own justification was that the runner is slower per core than the maintainer's 24-core box, where `gui` measured 476.309s (79% of Go's 600s per-package default), making the 600s default "odds-on to expire" on CI. That reasoning had the sign backwards:

- Runner measured: 386.065s
- Maintainer's box measured: 476.309s
- **The runner is ~90 seconds FASTER for this workload, not slower.**
- Against the original 600s default, the actual runner run would have finished with **~214s (600 − 386.065) to spare** — the default would have held. No timeout was averted by raising it to 20m; there was never a failure in flight.
- 386.065s / 600s ≈ 64% of the original default (vs. the 79% figure that was used to justify raising it).

A follow-up commit, **`5bfc118`** ("ci: correct the timeout commit's own justification -- the runner is FASTER"), exists locally on `main` and corrects the workflow comment in place — it does **not** revert the 20m timeout, and does **not** delete the bad prediction from history; it amends the justification going forward. Confirmed present and unpushed as of this report: `git cat-file -t 5bfc118` → `commit`; `git branch --contains 5bfc118` → `main` only; `git branch -r --contains 5bfc118` → empty (no remote-tracking branch carries it, i.e. it has not been pushed to `origin`). This report does not include its push — that is a separate, later action.

No `FAIL`, `panic`, or timeout-triggered abort anywhere in the `tests` job log (checked via `grep -iE "FAIL|panic|timeout|error"`, the only "timeout" hits being the literal `-timeout 20m` flag in the step's own command echo). Log ends cleanly at `Complete job` / `Cleaning up orphan processes`.

## Controller-side corroboration (run locally on the merged tree, BEFORE the push)

These three surfaces are **not** covered by host `go build ./...` or by the `tests` / `tinygo-device-build` / `build` CI jobs above, and had not been exercised by any phase gate earlier in this cycle. Recorded as reported by the controller; not independently re-run by this agent per the controller's explicit instruction not to re-verify already-observed facts:

- `GOOS=js GOARCH=wasm go vet ./cmd/emu/` — exit 0.
- `./scripts/test-32bit.sh` — green (386 tests exit 0, arm build exit 0).
- Nix TinyGo build of `./cmd/controller` for `pico-plus2` — exit 0 (flash 1412404 bytes, ram 61972 bytes).

## Verdict

**GREEN**

- Push landed at the exact recorded SHA (`ls-remote` match).
- No "Bypassed rule violations" (none expected — `main` unprotected on this fork).
- Both triggered workflows (`Test`, `Build image`) completed, run-level conclusion `success`.
- All 3 underlying jobs (`tests`, `tinygo-device-build`, `build`) independently confirmed `completed` / `success` — no job-level skip or failure hidden behind a run-level pass.
- The `gui` package ran to completion in 386.065s, well inside the new 20-minute timeout, with no failure — but the number is recorded here as evidence the timeout-raise's own justification was wrong (runner faster, not slower), not as a near-miss that the raise averted. A local, unpushed follow-up commit (`5bfc118`) already corrects the justification text.
- Three additional build/test surfaces outside CI's coverage (wasm vet, 32-bit test script, TinyGo pico-plus2 build) were exercised locally before the push and reported green by the controller.
