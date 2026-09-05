# Push report: mnemonic-engrave master → a382c14

## Preconditions (checked before running)

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
a382c14dddb3aa341064c97b294566d488746f5e
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
e77b09752022512910d3f829a3cf9b5de7d02418
$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)
```

Master tip matched `a382c14`; no stale `ci/staging` ref present. Confirmed
`scripts/push-via-staging.sh` exists before running it.

## Command run (foreground, cwd = /scratch/code/shibboleth/mnemonic-engrave)

```
$ ./scripts/push-via-staging.sh master 2>&1 | tee /tmp/push-engrave-a382c14.log
```

Ran to completion within the tool's single foreground call (timeout 600000ms) —
no background execution, no separate polling was required.

## Verbatim script output (full tail, `/tmp/push-engrave-a382c14.log`)

```
== staging a382c14dddb3aa341064c97b294566d488746f5e (branch master, 4 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33975448094; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   e77b097..a382c14  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (macos-x86_64): success
build me (linux-aarch64): success
build me-preview (all targets): success
assemble + sign + release: skipped
== OK: a382c14dddb3aa341064c97b294566d488746f5e is on master with the required check earned
```

No "Bypassed rule violations" line appears anywhere in the log (grepped for
`bypass`, case-insensitive: no match).

## Run details

- Run ID: `33975448094`
- Repo: `bg002h/mnemonic-engrave`
- `gh run view 33975448094 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha`:
  `{"conclusion":"success","headSha":"a382c14dddb3aa341064c97b294566d488746f5e","status":"completed"}`

Per-job conclusions (`gh run view 33975448094 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | .name + ": " + (.conclusion // .status)'`):

| Job | Conclusion |
| --- | --- |
| test (rust + go) *(required)* | success |
| build me (macos-aarch64) | success |
| build me (windows-x86_64) | success |
| build me (linux-x86_64) | success |
| build me (macos-x86_64) | success |
| build me (linux-aarch64) | success |
| build me-preview (all targets) | success |
| assemble + sign + release | skipped *(gated on `refs/tags/v*`, expected)* |

## Post-push verification

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
a382c14dddb3aa341064c97b294566d488746f5e
$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)
```

`origin/master` matches the staged tip; `ci/staging` was deleted by the script
(confirmed independently, not just from the script's own claim).

## Outcome

Success. `origin/master` = `a382c14dddb3aa341064c97b294566d488746f5e`, the
required `test (rust + go)` context earned it (not bypassed), staging ref
cleaned up, no code committed by this task.
