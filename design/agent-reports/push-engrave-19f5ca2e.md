# Push report — mnemonic-engrave master to 19f5ca2e

## SHA pushed
`19f5ca2eb48f003db24ebe5f388f138ad85ba0e9` (master tip, verified via `git rev-parse master` before running; 4 commits ahead of prior `origin/master` `1aee9db9e7396b3a6436d9b7175bcf748de8498b`). Tracked working tree was clean before and after (`git status --porcelain=v1 --untracked-files=no` empty both times).

## Method
Ran `scripts/push-via-staging.sh master` from the repo root in the foreground (no backgrounding), full run to completion, no manual fallback steps needed.

## Staging run
- Run ID: `34046366917`
- URL: https://github.com/bg002h/mnemonic-engrave/actions/runs/34046366917
- Required context: `test (rust + go)`

## Per-job conclusions (verbatim from `gh run view 34046366917 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | .name + ": " + (.conclusion // .status)'`)
```
build me-preview (all targets): success
build me (macos-x86_64): success
build me (macos-aarch64): success
test (rust + go): success
build me (linux-aarch64): success
build me (windows-x86_64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
```
The required job (`test (rust + go)`) concluded `success`. All non-required `build me (*)` jobs also concluded `success` this run — no macOS artifact-upload timeout observed this time. `assemble + sign + release` is `skipped`, as expected (gated on `refs/tags/v*`, and this was a branch push, not a tag).

## Final script output (verbatim)
```
== staging 19f5ca2eb48f003db24ebe5f388f138ad85ba0e9 (branch master, 4 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34046366917; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   1aee9db9..19f5ca2e  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
build me-preview (all targets): success
build me (macos-x86_64): success
build me (macos-aarch64): success
test (rust + go): success
build me (linux-aarch64): success
build me (windows-x86_64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
== OK: 19f5ca2eb48f003db24ebe5f388f138ad85ba0e9 is on master with the required check earned
```

No "Bypassed rule violations" line appeared in the final push output (`1aee9db9..19f5ca2e HEAD -> master`) — the required-context check was earned, not bypassed.

## Post-push verification
- `git fetch origin` run; `git rev-parse origin/master` = `19f5ca2eb48f003db24ebe5f388f138ad85ba0e9` (matches the pushed tip).
- `ci/staging` ref deleted by the script (confirmed in output above).
- Tracked working tree remained clean throughout (no source files modified, no commits made by this agent).

## What was not done (as instructed)
- No tag created, no version bump, no publish/release step — `assemble + sign + release` correctly stayed `skipped`.
- No source files modified, no commits made.
- This report was written but not committed (per instructions).
