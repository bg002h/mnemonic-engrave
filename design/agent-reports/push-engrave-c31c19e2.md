# Push report — mnemonic-engrave master via ci/staging

## SHA pushed
`c31c19e2821534004c5f7ca6ccf76f8912113b4f` (full 40-char)

Pre-push verification: `git rev-parse master` returned this exact SHA before the
ritual ran; `origin/master` was `a7357d68ed530e6e801ea50e0bcdfe2ef6401f7f` (an
ancestor, 7 commits behind); `git status --porcelain=v1 --untracked-files=all`
was empty (no tracked or untracked files) both before and after the run.

## Method
Ran `./scripts/push-via-staging.sh master` in the foreground (no backgrounding),
timeout 600000ms. No source file modified, no commit made.

## Staging run
- Run ID: `34052072286`
- Repo: `bg002h/mnemonic-engrave`
- `headSha`: `c31c19e2821534004c5f7ca6ccf76f8912113b4f`
- Run `status`: `completed`; run `conclusion`: `success`

### Per-job conclusions (verbatim, independent `gh run view --json jobs` query, not just the script's own report)

```
build me (windows-x86_64): success
build me (macos-aarch64): success
build me (macos-x86_64): success
build me (linux-x86_64): success
test (rust + go): success
build me (linux-aarch64): success
build me-preview (all targets): success
assemble + sign + release: skipped
```

`test (rust + go)` — the required context — is `success`. `assemble + sign +
release` is `skipped`, as expected: that job gates on `refs/tags/v*`, and this
was a branch push, not a tag.

## Final push output (verbatim)

```
== staging c31c19e2821534004c5f7ca6ccf76f8912113b4f (branch master, 7 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34052072286; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   a7357d68..c31c19e2  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
build me (windows-x86_64): success
build me (macos-aarch64): success
build me (macos-x86_64): success
build me (linux-x86_64): success
test (rust + go): success
build me (linux-aarch64): success
build me-preview (all targets): success
assemble + sign + release: skipped
== OK: c31c19e2821534004c5f7ca6ccf76f8912113b4f is on master with the required check earned
```

No "Bypassed rule violations" line appears anywhere in the output — confirmed
both in the script's stdout above and by rereading it specifically for that
string.

## Post-push verification (independent of the script)
- `git fetch origin` then `git rev-parse origin/master` → `c31c19e2821534004c5f7ca6ccf76f8912113b4f`
- `git rev-parse master` (local) → `c31c19e2821534004c5f7ca6ccf76f8912113b4f` (unchanged — no commits made)
- `git ls-remote origin refs/heads/ci/staging` → empty (ref deleted, as the ritual requires)
- `gh run view 34052072286 --repo bg002h/mnemonic-engrave --json headSha,status,conclusion` → `headSha` matches, `status: completed`, `conclusion: success`

## Outcome
SUCCESS. `origin/master` now equals `c31c19e2821534004c5f7ca6ccf76f8912113b4f`,
earned via the required `test (rust + go)` context on that exact SHA (not
bypassed). No manual fallback steps were needed — the script ran end to end.

## What I could not do / did not do
- Did not tag, bump versions, or publish (out of scope, as instructed).
- Did not modify any source file or make any commit.
- Nothing else was left undone; the ritual completed cleanly on the first attempt.
