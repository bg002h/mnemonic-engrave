# Push report — mnemonic-engrave master `1dbe5d3` via ci/staging ritual

Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)
Date: 2026-09-04/05

## Preconditions (checked before running the ritual)

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
1dbe5d33a89eb57be18eafa254f514f6d8749e4f

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)

$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
279d7318c03e1a7e7039a2adb7d7bcc96fc8b672
```

Master tip confirmed `1dbe5d3...`, matches the four commits described (a push
report, two briefs, and continuity — no crate code) ahead of `origin/master`
`279d731`. `ci/staging` was clear on origin before starting.

## Command run

```
$ cd /scratch/code/shibboleth/mnemonic-engrave
$ bash scripts/push-via-staging.sh master
```

Run in the background (log redirected to `/tmp/push-via-staging-1dbe5d3.log`),
then watched to completion via `gh run watch 33970656647 --repo
bg002h/mnemonic-engrave --exit-status` (exit code `0`) per the coordinator's
direction, rather than idly polling the background task.

## Script's own log (verbatim, from `/tmp/push-via-staging-1dbe5d3.log`)

```
== staging 1dbe5d33a89eb57be18eafa254f514f6d8749e4f (branch master, 4 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33970656647; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   279d731..1dbe5d3  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
build me-preview (all targets): success
build me (linux-aarch64): success
build me (windows-x86_64): success
test (rust + go): success
build me (macos-aarch64): success
build me (macos-x86_64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
== OK: 1dbe5d33a89eb57be18eafa254f514f6d8749e4f is on master with the required check earned
EXIT_CODE=0
```

**No "Bypassed rule violations" line appears anywhere in the log.** The final
push (`279d731..1dbe5d3 HEAD -> master`) is the verbatim git output for the
branch-protected push — it satisfied the required check rather than bypassing
it.

## Run id and per-job conclusions

Run id: **33970656647** (workflow `release`, event `push`, commit
`1dbe5d33a89eb57be18eafa254f514f6d8749e4f`, triggered on `ci/staging`).

```
$ gh run view 33970656647 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | .name + ": " + (.conclusion // .status)'
build me-preview (all targets): success
build me (linux-aarch64): success
build me (windows-x86_64): success
test (rust + go): success
build me (macos-aarch64): success
build me (macos-x86_64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
```

The required context, `test (rust + go)`, concluded `success`.
`assemble + sign + release` is gated on `refs/tags/v*` and correctly
`skipped` for a `ci/**` push (per repo convention — it never signs or
publishes off a staging ref).

`gh run watch 33970656647 --repo bg002h/mnemonic-engrave --exit-status` also
concluded with exit code `0` (whole-run success), confirming no job failed.

## Post-push verification

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
1dbe5d33a89eb57be18eafa254f514f6d8749e4f

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)
```

`origin/master` now equals the pushed tip; `ci/staging` is deleted (matches
the script's own `[deleted] ci/staging` line).

## Outcome

**SUCCESS.** `origin/master` is `1dbe5d33a89eb57be18eafa254f514f6d8749e4f`,
earned via the required `test (rust + go)` check on run `33970656647`
(commit staged first on `ci/staging`, per the branch-protection ritual). No
bypass. `ci/staging` cleaned up. Nothing else was committed by this task.
