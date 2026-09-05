# Push report: mnemonic-engrave master via ci/staging ritual

- Repo: `/scratch/code/shibboleth/mnemonic-engrave`
- GitHub: `bg002h/mnemonic-engrave`
- TIP (recorded before push): `1b1e73ee2e767526a12e454ccef26e91b020a9fc`
- origin/master before: `03295d07c645637929359193ca6e75a896ecc6fb`
- origin/master after: `1b1e73ee2e767526a12e454ccef26e91b020a9fc` (= TIP, confirmed)

## Precondition check

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
```
Output: empty (no stale staging ref). Precondition satisfied.

## Commands run

```
TIP=$(git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master)
# TIP=1b1e73ee2e767526a12e454ccef26e91b020a9fc
```

```
./scripts/push-via-staging.sh master
```
(run from `/scratch/code/shibboleth/mnemonic-engrave`, foreground, no backgrounding)

## Script tail (verbatim)

```
== staging 1b1e73ee2e767526a12e454ccef26e91b020a9fc (branch master, 7 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 33998721051; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   03295d07..1b1e73ee  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (linux-x86_64): success
build me (linux-aarch64): success
build me-preview (all targets): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (macos-x86_64): success
assemble + sign + release: skipped
== OK: 1b1e73ee2e767526a12e454ccef26e91b020a9fc is on master with the required check earned
```

No "Bypassed rule violations" line anywhere in the output (grepped the captured log independently — none found). The script completed the entire ritual (stage → wait for required check → push master → delete ci/staging) in a single foreground run; no manual polling of an in-flight run was needed since the script did not exit early.

## Independent verification (post-script)

```
$ gh run view 33998721051 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha
{"conclusion":"success","status":"completed","headSha":"1b1e73ee2e767526a12e454ccef26e91b020a9fc"}
```

Per-job conclusions:
```
$ gh run view 33998721051 --repo bg002h/mnemonic-engrave --json jobs --jq '.jobs[] | {name, conclusion}'
{"conclusion":"success","name":"test (rust + go)"}
{"conclusion":"success","name":"build me (linux-x86_64)"}
{"conclusion":"success","name":"build me (linux-aarch64)"}
{"conclusion":"success","name":"build me-preview (all targets)"}
{"conclusion":"success","name":"build me (macos-aarch64)"}
{"conclusion":"success","name":"build me (windows-x86_64)"}
{"conclusion":"success","name":"build me (macos-x86_64)"}
{"conclusion":"skipped","name":"assemble + sign + release"}
```
`assemble + sign + release` is tag-gated (`refs/tags/v*`) and correctly `skipped` on a branch push — not evidence of a bypass.

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
1b1e73ee2e767526a12e454ccef26e91b020a9fc
```
Matches TIP.

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
```
Output: empty (staging ref cleaned up as expected).

## Outcome

SUCCESS — no bypass. `origin/master` = `1b1e73ee2e767526a12e454ccef26e91b020a9fc`, gated by run 33998721051 (`test (rust + go)`: success).
