# Push report: mnemonic-engrave master via ci/staging ritual — SHA 364b864

## Pre-push verification
- Tree: clean (`git status --short` empty).
- Local tip before push: `364b864e738ed3d19c66a9d3e854b5bb4ba26ba8` (matched expected).
- Unpushed commit count: `git log --oneline origin/master..master | wc -l` = **23**.
- Merge check: `--no-ff` merge present — `024dd08 merge hashlock-h0: H0 reader guards, host half -- seam corpus rows, record capture, pin test, PreimagePlate diagnosis (me seal + sysw pack)`.

## Ritual script used
`scripts/push-via-staging.sh master`, invoked as:
```
PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp scripts/push-via-staging.sh master 2>&1 | tee /scratch/code/shibboleth/.tmp/push-engrave-364b864.log
```
Log: `/scratch/code/shibboleth/.tmp/push-engrave-364b864.log`.

## Run and job conclusions (required-context run, on `ci/staging`)
- Run id: **33936257564** (workflow `release`, event `push`, headBranch `ci/staging`, conclusion `success`).
- Per-job conclusions (via `gh run view 33936257564 --json jobs` and independently via `gh api .../commits/364b864e738ed3d19c66a9d3e854b5bb4ba26ba8/check-runs`):
  - `test (rust + go)`: **success** (the required context)
  - `build me-preview (all targets)`: success
  - `build me (windows-x86_64)`: success
  - `build me (linux-x86_64)`: success
  - `build me (macos-aarch64)`: success
  - `build me (linux-aarch64)`: success
  - `build me (macos-x86_64)`: success
  - `assemble + sign + release`: skipped (expected — gated on `refs/tags/v*`, not applicable to a branch push)

No job failed or required a follow-up log pull.

## Bypass check
"Bypassed rule violations" did **NOT** appear anywhere in the push output. Verbatim last lines of the log:
```
== run 33936257564; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   423b276..364b864  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me-preview (all targets): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (linux-aarch64): success
build me (macos-x86_64): success
assemble + sign + release: skipped
== OK: 364b864e738ed3d19c66a9d3e854b5bb4ba26ba8 is on master with the required check earned
```

## Post-push independent verification
- `git fetch origin && git rev-parse origin/master` = `364b864e738ed3d19c66a9d3e854b5bb4ba26ba8` — matches local tip exactly.
- `git ls-remote origin refs/heads/ci/staging` printed nothing — staging ref deleted, confirmed.
- Note (informational, not part of the required gate): the push to `master` itself triggered a second workflow run on the same SHA — run id `33936422385` (event `push`, headBranch `master`), observed `in_progress` at verification time. This is expected: the workflow triggers on pushes to `master` in addition to `ci/staging`; branch protection had already been satisfied by the completed, successful `ci/staging` run (33936257564) before the `master` push happened, per the script's design (`strict: false`). This second run is not gating and was not waited on.

## Verdict

**SUCCESS**
