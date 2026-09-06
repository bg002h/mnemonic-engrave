# Push report: mnemonic-engrave master via ci/staging

- Repo: `/scratch/code/shibboleth/mnemonic-engrave`
- GitHub: `bg002h/mnemonic-engrave`
- origin/master before push: `c011196d` (5 commits ahead locally)
- TIP staged and pushed: `9045b5781c2c025c9dcce2a76d05d925e6525715`

## Commands run

```
TIP=$(git rev-parse master)
# TIP=9045b5781c2c025c9dcce2a76d05d925e6525715

git ls-remote origin refs/heads/ci/staging
# (empty output -- precondition satisfied: no stale ci/staging ref)

git status --short
# (empty -- clean tree)

./scripts/push-via-staging.sh master
```

## Verbatim final output of push-via-staging.sh

```
== staging 9045b5781c2c025c9dcce2a76d05d925e6525715 (branch master, 5 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34013154184; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   c011196d..9045b578  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
build me-preview (all targets): success
build me (macos-x86_64): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (macos-aarch64): success
test (rust + go): success
build me (linux-aarch64): success
assemble + sign + release: skipped
== OK: 9045b5781c2c025c9dcce2a76d05d925e6525715 is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output. The script ran to completion in the foreground within a single invocation (timeout 600000ms) -- no separate polling of `gh run view` was needed.

## Run details

- Run ID: `34013154184`
- Repo: `bg002h/mnemonic-engrave`
- Head SHA: `9045b5781c2c025c9dcce2a76d05d925e6525715`
- Overall run status/conclusion (via `gh run view 34013154184 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha,jobs`): `status: completed`, `conclusion: success`

Per-job conclusions:

| Job | Conclusion |
| --- | --- |
| build me-preview (all targets) | success |
| build me (macos-x86_64) | success |
| build me (windows-x86_64) | success |
| build me (linux-x86_64) | success |
| build me (macos-aarch64) | success |
| test (rust + go) [REQUIRED context] | success |
| build me (linux-aarch64) | success |
| assemble + sign + release | skipped (correct: gated on `refs/tags/v*`, not applicable to a branch push) |

## Post-push verification

```
git fetch origin
git rev-parse origin/master
# 9045b5781c2c025c9dcce2a76d05d925e6525715
```

`origin/master` == `9045b5781c2c025c9dcce2a76d05d925e6525715` == recorded TIP. Match confirmed.

The `ci/staging` ref was deleted by the script itself after the final push succeeded (`git push origin --delete ci/staging`), consistent with the ritual.

## Outcome

SUCCESS. `master` pushed to `9045b5781c2c025c9dcce2a76d05d925e6525715` via the ci/staging ritual; required context `test (rust + go)` earned (not bypassed); origin/master confirmed at the recorded tip.
