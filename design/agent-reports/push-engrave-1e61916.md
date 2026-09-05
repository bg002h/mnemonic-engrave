# Push report: mnemonic-engrave master → 1e61916 via ci/staging

Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)
Date: 2026-09-04

## Preconditions (checked before running the script)

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
1e6191602faaed8e88faf45ad587edac167be53d

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty — no stale staging ref)

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/master
f94c9034497ea35cf917f8f583d45d1389480ded	refs/heads/master

$ ls -la /scratch/code/shibboleth/mnemonic-engrave/scripts/push-via-staging.sh
-rwxr-xr-x 1 bcg bcg 2433 Aug 28 21:36 ... (present)

$ ls -la /scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/push-engrave-1e61916.md
No such file or directory (did not pre-exist, as required)
```

`master` tip matched the requested `1e61916...`; `origin/master` was `f94c903`;
`git rev-list --count f94c903..1e6191916` (run post-push, see below) reports
**9** commits ahead, not the 14 mentioned in the dispatch brief — recorded as
observed, not corrected.

## Command run (foreground, from repo root)

```
$ ./scripts/push-via-staging.sh master
```

## Verbatim script output

```
== staging 1e6191602faaed8e88faf45ad587edac167be53d (branch master, 9 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33947660078; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   f94c903..1e61916  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
build me-preview (all targets): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (macos-x86_64): success
test (rust + go): success
build me (linux-aarch64): success
assemble + sign + release: skipped
== OK: 1e6191602faaed8e88faf45ad587edac167be53d is on master with the required check earned
```

No "Bypassed rule violations" line appeared at any point — the push was not a
bypass; the required `test (rust + go)` context was earned on the exact SHA
via `ci/staging` before the push to `master`, per the standing ritual.

## Independent verification (post-push, separate from the script's own report)

```
$ gh run view 33947660078 --repo bg002h/mnemonic-engrave --json databaseId,headSha,status,conclusion,event,headBranch
{"conclusion":"success","databaseId":33947660078,"event":"push","headBranch":"ci/staging",
 "headSha":"1e6191602faaed8e88faf45ad587edac167be53d","status":"completed"}

$ gh run view 33947660078 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | "\(.name): \(.conclusion)"'
build me-preview (all targets): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (macos-x86_64): success
test (rust + go): success
build me (linux-aarch64): success
assemble + sign + release: skipped

$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
(no new output — already up to date after the script's push)

$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
1e6191602faaed8e88faf45ad587edac167be53d

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty — staging ref deleted as expected)

$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-list --count f94c9034497ea35cf917f8f583d45d1389480ded..1e6191602faaed8e88faf45ad587edac167be53d
9
```

Per-job conclusions match between the script's own straggler report and the
independent `gh run view` query. `assemble + sign + release` correctly
reported `skipped` (it is gated on `refs/tags/v*`, and this was a branch
push) — consistent with the repo's documented release-workflow gating, not a
failure.

## Outcome

`origin/master` for `bg002h/mnemonic-engrave` now points at
`1e6191602faaed8e88faf45ad587edac167be53d`, matching local `master`. The
required `test (rust + go)` check (run 33947660078) reports `success` against
that exact SHA, confirmed independently via `gh run view`. `ci/staging` was
deleted from origin, restoring the empty precondition state. No bypass
occurred.
