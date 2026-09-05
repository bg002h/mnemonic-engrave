# Push report: mnemonic-engrave master @ 1b0ec7e (2026-09-04)

## Pre-push check (step 1)
- `git status --short`: clean (no output)
- Tip SHA (full): `1b0ec7e328bdbd6c41d8c1911f730707df27c5f5`
- Unpushed commits (`git log --oneline origin/master..master | wc -l`): 5

## Ritual run (step 2)
Command: `PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp scripts/push-via-staging.sh master`
Run id: `33941671822`

Push output's last lines (verbatim, from `/scratch/code/shibboleth/.tmp/push-engrave-1b0ec7e.log`):

```
== staging 1b0ec7e328bdbd6c41d8c1911f730707df27c5f5 (branch master, 5 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33941671822; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   364b864..1b0ec7e  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me-preview (all targets): success
build me (linux-aarch64): success
build me (windows-x86_64): success
build me (macos-aarch64): success
build me (macos-x86_64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
== OK: 1b0ec7e328bdbd6c41d8c1911f730707df27c5f5 is on master with the required check earned
```

## Per-job conclusions on the gated SHA (step 3, via `gh api .../check-runs`)
(check-runs API returns each job's in-progress and completed attempt; completed conclusions shown)

| job | conclusion |
| --- | --- |
| test (rust + go) | success |
| build me-preview (all targets) | success |
| build me (linux-x86_64) | success |
| build me (linux-aarch64) | success |
| build me (macos-x86_64) | success |
| build me (macos-aarch64) | success |
| build me (windows-x86_64) | success |
| assemble + sign + release | skipped (tag-gated, expected on a non-tag push) |

Required context `test (rust + go)`: **success**. No job failed, timed out, or was cancelled.

## Bypass check
`grep -i "bypass" /scratch/code/shibboleth/.tmp/push-engrave-1b0ec7e.log` → no match. No "Bypassed rule violations" text anywhere in the push output.

## Post-push state
- `git fetch origin && git rev-parse origin/master` → `1b0ec7e328bdbd6c41d8c1911f730707df27c5f5`
- `git rev-parse master` (local) → `1b0ec7e328bdbd6c41d8c1911f730707df27c5f5` (matches)
- `git ls-remote origin refs/heads/ci/staging` → empty (staging ref deleted, as the script's final step does)

## Verdict

**SUCCESS**
