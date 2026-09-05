# Push mnemonic-engrave master via ci/staging — ac6a19c

Repo: `/scratch/code/shibboleth/mnemonic-engrave`
GitHub: `bg002h/mnemonic-engrave`

## Preconditions

```
$ TIP=$(git rev-parse master) && echo "TIP=$TIP"
TIP=ac6a19c21e03080f10e7885287e8e59e7d0d57d4

$ git ls-remote origin refs/heads/ci/staging
(empty — no pre-existing staging ref)

$ git status --short
(clean)

$ git fetch origin master && git rev-parse origin/master
26690c19ccf7f3c7218a5881f8e91fed96f5090d
```

origin/master before the push: `26690c19ccf7f3c7218a5881f8e91fed96f5090d` (matches the
`26690c1` the operator cited). 14 commits ahead on local `master` (reports, a plan,
briefs, a script, continuity — no crate code, per the operator's framing).

## Staging run

Command (foreground, no backgrounding, no watcher wait):

```
$ ./scripts/push-via-staging.sh master
```

Verbatim tail (captured to `/tmp/push-via-staging-engrave.log`):

```
== staging ac6a19c21e03080f10e7885287e8e59e7d0d57d4 (branch master, 14 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33981988573; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   26690c1..ac6a19c  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-aarch64): success
build me-preview (all targets): success
build me (linux-x86_64): success
build me (macos-x86_64): success
test (rust + go): success
assemble + sign + release: skipped
== OK: ac6a19c21e03080f10e7885287e8e59e7d0d57d4 is on master with the required check earned
```

The script ran to completion in the foreground within the tool timeout; no
poll-the-run fallback was needed. No "Bypassed rule violations" line appeared
anywhere in the output.

Run id: `33981988573`.

## Independent verification

```
$ gh run view 33981988573 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha,displayTitle
{"conclusion":"success","displayTitle":"continuity: H5 plan round 0 persisted; fold author dispatched","headSha":"ac6a19c21e03080f10e7885287e8e59e7d0d57d4","status":"completed"}

$ gh run view 33981988573 --repo bg002h/mnemonic-engrave --json jobs --jq '.jobs[] | {name, conclusion}'
{"conclusion":"success","name":"build me (macos-aarch64)"}
{"conclusion":"success","name":"build me (windows-x86_64)"}
{"conclusion":"success","name":"build me (linux-aarch64)"}
{"conclusion":"success","name":"build me-preview (all targets)"}
{"conclusion":"success","name":"build me (linux-x86_64)"}
{"conclusion":"success","name":"build me (macos-x86_64)"}
{"conclusion":"success","name":"test (rust + go)"}
{"conclusion":"skipped","name":"assemble + sign + release"}
```

`test (rust + go)` (the required context) = success. `assemble + sign + release` is
`skipped`, consistent with `.github/workflows/release.yml` gating that job on
`refs/tags/v*` — a `ci/**`/master push cannot sign or publish.

```
$ git fetch origin
$ git rev-parse origin/master
ac6a19c21e03080f10e7885287e8e59e7d0d57d4
```

`origin/master` == `TIP` (`ac6a19c21e03080f10e7885287e8e59e7d0d57d4`). Confirmed match.

```
$ git ls-remote origin refs/heads/ci/staging
(empty)
```

Staging ref cleaned up as expected.

## Outcome

SATISFIED, not bypassed. `origin/master` now points at `ac6a19c21e03080f10e7885287e8e59e7d0d57d4`,
carrying the required `test (rust + go)` check earned on that exact SHA (run
33981988573). No commits landed in this session; nothing else was staged or
folded.
