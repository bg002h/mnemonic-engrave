# Push report — mnemonic-engrave master via ci/staging

- SHA pushed: `a7357d68ed530e6e801ea50e0bcdfe2ef6401f7f`
- Pre-push tip check: `git rev-parse master` = `a7357d68ed530e6e801ea50e0bcdfe2ef6401f7f` (matched expected); working tree had no modified tracked files (`git status --porcelain=v1 --branch` showed only `## master...origin/master [ahead 6]`); `origin/master` was `aa572b122e65e84f2ae322da6316f3595af74f04` before this push.
- Staging run id: `34050389453`
- Repo: `bg002h/mnemonic-engrave`

## Per-job conclusions (verbatim, `gh run view 34050389453 --repo bg002h/mnemonic-engrave --json jobs`)

```
build me (linux-x86_64): success
build me-preview (all targets): success
build me (windows-x86_64): success
build me (macos-aarch64): success
build me (linux-aarch64): success
test (rust + go): success
build me (macos-x86_64): success
assemble + sign + release: skipped
```

Independent confirmation of the run's head SHA (`gh run view 34050389453 --repo bg002h/mnemonic-engrave --json headSha,status,conclusion`):

```
headSha=a7357d68ed530e6e801ea50e0bcdfe2ef6401f7f status=completed conclusion=success
```

Required context `test (rust + go)`: **success**. `assemble + sign + release` is correctly `skipped` (gated on `refs/tags/v*`; this was a branch push, not a tag).

## Script output (`scripts/push-via-staging.sh master`, verbatim)

```
== staging a7357d68ed530e6e801ea50e0bcdfe2ef6401f7f (branch master, 6 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34050389453; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   aa572b12..a7357d68  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
build me (linux-x86_64): success
build me-preview (all targets): success
build me (windows-x86_64): success
build me (macos-aarch64): success
build me (linux-aarch64): success
test (rust + go): success
build me (macos-x86_64): success
assemble + sign + release: skipped
== OK: a7357d68ed530e6e801ea50e0bcdfe2ef6401f7f is on master with the required check earned
```

### Final push output specifically (the `git push origin HEAD:master` line inside the script)

```
To github.com:bg002h/mnemonic-engrave.git
   aa572b12..a7357d68  HEAD -> master
```

No "Bypassed rule violations" line present — the required check was earned, not bypassed.

## Post-push independent verification

- `git fetch origin` run; `git rev-parse origin/master` = `a7357d68ed530e6e801ea50e0bcdfe2ef6401f7f` — matches the pushed tip.
- `git ls-remote origin refs/heads/ci/staging` returned empty — staging ref confirmed deleted.

## What could not be done / was not attempted

- Nothing failed. No tag, version bump, or release was created or attempted (out of scope per brief).
- No source file was modified and no commit was made by this agent; this report file itself was written and is not committed.
