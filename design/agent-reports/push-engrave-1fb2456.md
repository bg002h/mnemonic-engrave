# Push report: mnemonic-engrave master 1fb24566 via ci/staging

Date: 2026-09-05
Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)

## Preconditions (checked before running)

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
1fb245668ea624e5b05e5e39bac7041c215711c7

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)
```

`origin/master` before the push was `dca22b9...` (per the dispatch brief); master tip
`1fb245668ea624e5b05e5e39bac7041c215711c7` starts with `1fb24566` as expected, and no
stale `ci/staging` ref existed on origin.

## Command run

```
$ ./scripts/push-via-staging.sh master 2>&1 | tee /tmp/push-staging-1fb2456.log
```

Run in the foreground (not backgrounded), no timeout hit — the script ran to completion
on its own.

## Script tail (verbatim)

```
== staging 1fb245668ea624e5b05e5e39bac7041c215711c7 (branch master, 7 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 33988590572; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   dca22b93..1fb24566  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me-preview (all targets): success
build me (linux-x86_64): success
build me (windows-x86_64): success
build me (linux-aarch64):
build me (macos-x86_64): success
build me (macos-aarch64): success
== OK: 1fb245668ea624e5b05e5e39bac7041c215711c7 is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output — the required
`test (rust + go)` check was earned by the staged SHA before the final push to `master`,
per the staging ritual in CLAUDE.md.

Note: at the moment the script printed its straggler report, `build me (linux-aarch64)`
was still `in_progress` (hence the blank conclusion in that line) — a non-required,
informational job, not the gating check. Polled to completion below.

## Run verification via `gh` (full SHA, `--repo` on every call)

Immediately after the script exited:

```
$ gh run view 33988590572 --repo bg002h/mnemonic-engrave \
    --json databaseId,status,conclusion,headSha,jobs \
    --jq '{databaseId,status,conclusion,headSha,jobs: [.jobs[] | {name, status, conclusion}]}'

{"conclusion":"","databaseId":33988590572,"headSha":"1fb245668ea624e5b05e5e39bac7041c215711c7","status":"in_progress",
 "jobs":[
   {"name":"test (rust + go)","status":"completed","conclusion":"success"},
   {"name":"build me-preview (all targets)","status":"completed","conclusion":"success"},
   {"name":"build me (linux-x86_64)","status":"completed","conclusion":"success"},
   {"name":"build me (windows-x86_64)","status":"completed","conclusion":"success"},
   {"name":"build me (linux-aarch64)","status":"in_progress","conclusion":""},
   {"name":"build me (macos-x86_64)","status":"completed","conclusion":"success"},
   {"name":"build me (macos-aarch64)","status":"completed","conclusion":"success"}
 ]}
```

Polled (foreground, no background/watcher) until the whole run concluded:

```
$ until gh run view 33988590572 --repo bg002h/mnemonic-engrave --json status --jq '.status' | grep -q completed; do sleep 5; done
$ gh run view 33988590572 --repo bg002h/mnemonic-engrave \
    --json databaseId,status,conclusion,headSha,jobs \
    --jq '{databaseId,status,conclusion,headSha,jobs: [.jobs[] | {name, status, conclusion}]}'

{"conclusion":"success","databaseId":33988590572,"headSha":"1fb245668ea624e5b05e5e39bac7041c215711c7","status":"completed",
 "jobs":[
   {"name":"test (rust + go)","status":"completed","conclusion":"success"},
   {"name":"build me-preview (all targets)","status":"completed","conclusion":"success"},
   {"name":"build me (linux-x86_64)","status":"completed","conclusion":"success"},
   {"name":"build me (windows-x86_64)","status":"completed","conclusion":"success"},
   {"name":"build me (linux-aarch64)","status":"completed","conclusion":"success"},
   {"name":"build me (macos-x86_64)","status":"completed","conclusion":"success"},
   {"name":"build me (macos-aarch64)","status":"completed","conclusion":"success"},
   {"name":"assemble + sign + release","status":"completed","conclusion":"skipped"}
 ]}
```

Per-job conclusions, final: 7 builds/tests all `success`; `assemble + sign + release`
`skipped` (expected — that job is gated on `refs/tags/v*`, not a branch push).
Run-level `conclusion`: `success`.

## Post-push confirmation

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
1fb245668ea624e5b05e5e39bac7041c215711c7

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)
```

`origin/master` is now `1fb245668ea624e5b05e5e39bac7041c215711c7`, matching the intended
tip. The `ci/staging` branch was deleted by the script (confirmed empty on origin, and
also visible as `[deleted] ci/staging` in the script's own output).

## Outcome

SUCCESS. `bg002h/mnemonic-engrave` `master` is now `1fb245668ea624e5b05e5e39bac7041c215711c7`,
pushed via the staging ritual with the required `test (rust + go)` check earned on that
exact SHA before landing on `master` (run 33988590572, run-level conclusion `success`, no
"Bypassed rule violations"). No commits were made by this task; this report was written
by the agent as its final action and is not committed here.
