# Push report: mnemonic-engrave master → 279d731 via ci/staging

- Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)
- Tip pushed: `279d7318c03e1a7e7039a2adb7d7bcc96fc8b672`
- origin/master before: `0a4acef891f6868ba568014c160d7ab78af929a2`
- Required context: `test (rust + go)`
- Run: https://github.com/bg002h/mnemonic-engrave/actions/runs/33968156129 (id `33968156129`)

## Preconditions checked

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
279d7318c03e1a7e7039a2adb7d7bcc96fc8b672

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)

$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
0a4acef891f6868ba568014c160d7ab78af929a2
```

## Command run

```
$ cd /scratch/code/shibboleth/mnemonic-engrave
$ ./scripts/push-via-staging.sh master
```

## Verbatim script output

```
== staging 279d7318c03e1a7e7039a2adb7d7bcc96fc8b672 (branch master, 4 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33968156129; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   0a4acef..279d731  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (macos-x86_64): success
build me (linux-x86_64): success
build me (windows-x86_64): success
build me-preview (all targets): success
build me (linux-aarch64): success
build me (macos-aarch64): success
assemble + sign + release: skipped
== OK: 279d7318c03e1a7e7039a2adb7d7bcc96fc8b672 is on master with the required check earned
```

**No "Bypassed rule violations" line appeared anywhere in the output.** The final push line
`0a4acef..279d731  HEAD -> master` shows a fast-forward push, not a bypass.

## Post-push verification

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
279d7318c03e1a7e7039a2adb7d7bcc96fc8b672
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
279d7318c03e1a7e7039a2adb7d7bcc96fc8b672
$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)
```

`origin/master` and local `master` both resolve to `279d7318c03e1a7e7039a2adb7d7bcc96fc8b672`.
The `ci/staging` ref no longer exists on the remote.

## Run job conclusions (per job, from `gh run view --repo bg002h/mnemonic-engrave`)

```
$ gh run view 33968156129 --repo bg002h/mnemonic-engrave --json headSha,status,conclusion,url -q '{headSha,status,conclusion,url}'
{"conclusion":"success","headSha":"279d7318c03e1a7e7039a2adb7d7bcc96fc8b672","status":"completed","url":"https://github.com/bg002h/mnemonic-engrave/actions/runs/33968156129"}

$ gh run view 33968156129 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | .name + " | " + .status + " | " + (.conclusion // "null")'
test (rust + go) | completed | success
build me (macos-x86_64) | completed | success
build me (linux-x86_64) | completed | success
build me (windows-x86_64) | completed | success
build me-preview (all targets) | completed | success
build me (linux-aarch64) | completed | success
build me (macos-aarch64) | completed | success
assemble + sign + release | completed | skipped
```

`assemble + sign + release` is gated on `refs/tags/v*` and correctly reports `skipped` for a
branch push (per repo convention documented in CLAUDE.md).

## Outcome

SUCCESS. `master` on `bg002h/mnemonic-engrave` is now `279d7318c03e1a7e7039a2adb7d7bcc96fc8b672`,
pushed with the required `test (rust + go)` context earned on the SHA via `ci/staging` — no bypass.
Nothing was committed as part of this task; this report file was written directly and is not
committed.
