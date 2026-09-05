# Push report: mnemonic-engrave master → 81ecec3

Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)
Tip pushed: `81ecec380495879c35db7bea21c82885b20ccc55`
Prior `origin/master`: `a382c14` (9 commits ahead; push report + FOLLOWUPS rulings + a new spec + briefs/continuity — no crate code)

## Preconditions (verified before running the ritual)

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
81ecec380495879c35db7bea21c82885b20ccc55

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)
```

## Command run (foreground, no timeout hit)

```
$ ./scripts/push-via-staging.sh master
```

## Verbatim tail (full stdout/stderr of the run)

```
== staging 81ecec380495879c35db7bea21c82885b20ccc55 (branch master, 9 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33976938456; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   a382c14..81ecec3  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (macos-x86_64): success
build me-preview (all targets): success
build me (macos-aarch64): success
build me (linux-aarch64): success
assemble + sign + release: skipped
== OK: 81ecec380495879c35db7bea21c82885b20ccc55 is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output. The script completed the whole ritual on its own (staged, waited for the required context, pushed master, deleted `ci/staging`) — no separate `gh run view` polling loop was needed after the script exited.

## Run details

- Workflow run: `33976938456` — https://github.com/bg002h/mnemonic-engrave/actions/runs/33976938456
- `gh run view 33976938456 --repo bg002h/mnemonic-engrave --json databaseId,status,conclusion,headSha,url`:
  ```json
  {"conclusion":"success","databaseId":33976938456,"headSha":"81ecec380495879c35db7bea21c82885b20ccc55","status":"completed","url":"https://github.com/bg002h/mnemonic-engrave/actions/runs/33976938456"}
  ```
- Per-job conclusions (`gh run view 33976938456 --repo bg002h/mnemonic-engrave --json jobs`):
  - `test (rust + go)` (required context): **success**
  - `build me (windows-x86_64)`: success
  - `build me (linux-x86_64)`: success
  - `build me (macos-x86_64)`: success
  - `build me-preview (all targets)`: success
  - `build me (macos-aarch64)`: success
  - `build me (linux-aarch64)`: success
  - `assemble + sign + release`: skipped (gated on `refs/tags/v*`, expected for a non-tag push)

## Post-push verification

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
81ecec380495879c35db7bea21c82885b20ccc55
```

`origin/master` matches the intended tip. `ci/staging` was deleted by the script (confirmed in the tail above: `- [deleted]  ci/staging`).

## Outcome

SUCCESS — no bypass, required context earned on the SHA before the branch push, `origin/master` = `81ecec380495879c35db7bea21c82885b20ccc55`.
