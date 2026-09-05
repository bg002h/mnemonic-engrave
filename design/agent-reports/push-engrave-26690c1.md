# Push report: mnemonic-engrave master 26690c1 via ci/staging

Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)
Target tip: `26690c19ccf7f3c7218a5881f8e91fed96f5090d`
Prior `origin/master`: `81ecec380495879c35db7bea21c82885b20ccc55`

## Preconditions checked

```
$ git rev-parse master
26690c19ccf7f3c7218a5881f8e91fed96f5090d

$ git ls-remote origin refs/heads/ci/staging
(empty)

$ git ls-remote origin refs/heads/master
81ecec380495879c35db7bea21c82885b20ccc55	refs/heads/master

$ ls -la scripts/push-via-staging.sh
-rwxr-xr-x 1 bcg bcg 2433 Aug 28 21:36 scripts/push-via-staging.sh
```

Both preconditions satisfied: local `master` starts with `26690c1`; `ci/staging` absent on origin.

## Command run (foreground, no backgrounding, no separate watcher)

```
$ ./scripts/push-via-staging.sh master 2>&1 | tee /tmp/push-staging-engrave-26690c1.log
```

## Verbatim script output (tail)

```
== staging 26690c19ccf7f3c7218a5881f8e91fed96f5090d (branch master, 20 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33979013525; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   81ecec3..26690c1  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me-preview (all targets): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (macos-x86_64): success
build me (linux-aarch64): success
assemble + sign + release: skipped
== OK: 26690c19ccf7f3c7218a5881f8e91fed96f5090d is on master with the required check earned
```

No "Bypassed rule violations" line anywhere in the output. The script completed within the single foreground invocation — no need to fall back to polling `gh run view` separately, since the script itself waited for the required context and reported the outcome before exiting.

## Independent verification (gh, full SHAs, --repo on every call)

```
$ gh run view 33979013525 --repo bg002h/mnemonic-engrave --json databaseId,status,conclusion,headSha,event
{"conclusion":"success","databaseId":33979013525,"event":"push","headSha":"26690c19ccf7f3c7218a5881f8e91fed96f5090d","status":"completed"}

$ gh run view 33979013525 --repo bg002h/mnemonic-engrave --json jobs --jq '.jobs[] | {name, conclusion}'
{"conclusion":"success","name":"test (rust + go)"}
{"conclusion":"success","name":"build me-preview (all targets)"}
{"conclusion":"success","name":"build me (linux-x86_64)"}
{"conclusion":"success","name":"build me (macos-aarch64)"}
{"conclusion":"success","name":"build me (windows-x86_64)"}
{"conclusion":"success","name":"build me (macos-x86_64)"}
{"conclusion":"success","name":"build me (linux-aarch64)"}
{"conclusion":"skipped","name":"assemble + sign + release"}
```

`test (rust + go)` (the required context) concluded `success`, matching the target `headSha`. `assemble + sign + release` correctly `skipped` (gated on `refs/tags/v*`, not applicable to a `ci/staging`/`master` push).

## Post-push confirmation

```
$ git fetch origin
(no output — up to date after push)

$ git rev-parse origin/master
26690c19ccf7f3c7218a5881f8e91fed96f5090d

$ git rev-parse master
26690c19ccf7f3c7218a5881f8e91fed96f5090d

$ git ls-remote origin refs/heads/ci/staging
(empty)
```

`origin/master` == local `master` == `26690c19ccf7f3c7218a5881f8e91fed96f5090d`. `ci/staging` deleted from origin as expected.

## Outcome

SUCCESS — `mnemonic-engrave` `origin/master` advanced from `81ecec380495879c35db7bea21c82885b20ccc55` to `26690c19ccf7f3c7218a5881f8e91fed96f5090d` via the `ci/staging` ritual. The required `test (rust + go)` context was earned on the exact pushed SHA (run `33979013525`, `success`) before `master` was updated; no bypass occurred. `ci/staging` was deleted afterward per the script's own cleanup.
