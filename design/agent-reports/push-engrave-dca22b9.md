# Push report: mnemonic-engrave master via ci/staging

- Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)
- Date: 2026-09-04/05 session
- TIP recorded before push: `dca22b93f5741b2a30e474a0b3f22bf5dd935fcf`
- origin/master before push: `583c59fbfef55826806785954b1569c1a1d29666` (9 commits behind local master; reports/briefs/continuity only, no crate code)

## Commands run, in order

```
$ TIP=$(git rev-parse master) && echo "TIP=$TIP"
TIP=dca22b93f5741b2a30e474a0b3f22bf5dd935fcf

$ git ls-remote origin refs/heads/ci/staging
(empty — precondition satisfied)

$ git ls-remote origin refs/heads/master
583c59fbfef55826806785954b1569c1a1d29666	refs/heads/master

$ ./scripts/push-via-staging.sh master
(see verbatim tail below)

$ gh run view 33987031203 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha,displayTitle
$ gh run view 33987031203 --repo bg002h/mnemonic-engrave --json jobs --jq '.jobs[] | {name, conclusion}'
$ git fetch origin
$ git rev-parse origin/master
$ git rev-parse master
```

## Run id and per-job conclusions

Run: `33987031203` (workflow run for SHA `dca22b93f5741b2a30e474a0b3f22bf5dd935fcf`), overall `status: completed`, `conclusion: success`.

| job | conclusion |
| --- | --- |
| test (rust + go) | success |
| build me (linux-aarch64) | success |
| build me-preview (all targets) | success |
| build me (macos-x86_64) | success |
| build me (macos-aarch64) | success |
| build me (windows-x86_64) | success |
| build me (linux-x86_64) | success |
| assemble + sign + release | skipped (expected — gated on `refs/tags/v*`, not a `ci/**` push) |

## Verbatim final tail of `scripts/push-via-staging.sh master`

```
== staging dca22b93f5741b2a30e474a0b3f22bf5dd935fcf (branch master, 9 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33987031203; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   583c59f..dca22b9  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (linux-aarch64): success
build me-preview (all targets): success
build me (macos-x86_64): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
== OK: dca22b93f5741b2a30e474a0b3f22bf5dd935fcf is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the script's output — the push was gated by the earned `test (rust + go)` check, not bypassed.

## Post-push verification

- `git fetch origin` then `git rev-parse origin/master` → `dca22b93f5741b2a30e474a0b3f22bf5dd935fcf`
- Compared against recorded `TIP` (`dca22b93f5741b2a30e474a0b3f22bf5dd935fcf`): **MATCH**.
- Local `master` unchanged at `dca22b93f5741b2a30e474a0b3f22bf5dd935fcf` throughout (no commits landed on master during the staging window).
- `ci/staging` branch deleted by the script (confirmed by the `[deleted]` line in its output); no leftover remote branch.

## Outcome

Success, no bypass. `origin/master` = `dca22b93f5741b2a30e474a0b3f22bf5dd935fcf`, earned via the required `test (rust + go)` check on run `33987031203`.
