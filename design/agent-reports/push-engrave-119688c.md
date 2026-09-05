# Push report -- mnemonic-engrave master `119688cc` via ci/staging

Repo: `/scratch/code/shibboleth/mnemonic-engrave`
GitHub: `bg002h/mnemonic-engrave`
Date: 2026-09-05 (session started 2026-09-04)

## Preconditions (checked before running the ritual)

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
119688cc848571259d517745d8a2b2ba4fdfbd5c

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty -- no ci/staging ref existed on origin)

$ git -C /scratch/code/shibboleth/mnemonic-engrave status --short
(clean)

$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master  # before the push
1fb24566...  (confirmed via the script's own "11 ahead" line; not queried separately)
```

master tip matched `119688cc` exactly, `ci/staging` did not exist, and the working
tree was clean. Proceeded.

## Command run (foreground, no backgrounding, no waiting on a separate watcher)

```
$ ./scripts/push-via-staging.sh master 2>&1 | tee /tmp/push-via-staging-119688cc.log
```

Ran to completion in the foreground inside a single Bash call (timeout budget
600000ms); the script itself did all the polling and exited 0 without needing
a manual `gh run view` poll loop.

## Verbatim script output (full, unedited)

```
== staging 119688cc848571259d517745d8a2b2ba4fdfbd5c (branch master, 11 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 33993812226; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   1fb24566..119688cc  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-aarch64): success
build me (linux-x86_64): success
build me-preview (all targets): success
build me (macos-x86_64): success
assemble + sign + release: skipped
== OK: 119688cc848571259d517745d8a2b2ba4fdfbd5c is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output (confirmed
by `grep -i bypass` over the captured log -- no match).

## Run id and per-job conclusions

Run id: `33993812226`

Independently re-queried after the script finished (not trusting the script's
own straggler report alone):

```
$ gh run view 33993812226 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha,jobs \
    -q '{status,conclusion,headSha,jobs:[.jobs[]|{name,conclusion}]}'
{
  "status": "completed",
  "conclusion": "success",
  "headSha": "119688cc848571259d517745d8a2b2ba4fdfbd5c",
  "jobs": [
    {"name": "test (rust + go)",              "conclusion": "success"},
    {"name": "build me (macos-aarch64)",       "conclusion": "success"},
    {"name": "build me (windows-x86_64)",      "conclusion": "success"},
    {"name": "build me (linux-aarch64)",       "conclusion": "success"},
    {"name": "build me (linux-x86_64)",        "conclusion": "success"},
    {"name": "build me-preview (all targets)", "conclusion": "success"},
    {"name": "build me (macos-x86_64)",        "conclusion": "success"},
    {"name": "assemble + sign + release",      "conclusion": "skipped"}
  ]
}
```

`headSha` matches the target tip exactly. Required context `test (rust + go)`
concluded `success`. `assemble + sign + release` is `skipped`, as expected --
it is gated on `refs/tags/v*` and this was a branch push, not a tag.

## Post-push verification

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
119688cc848571259d517745d8a2b2ba4fdfbd5c

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty -- ci/staging deleted as the script's final cleanup step)
```

`origin/master` now equals `119688cc848571259d517745d8a2b2ba4fdfbd5c`, matching
the target tip exactly. The `ci/staging` ref is gone from origin, confirming the
script's own delete step took effect (not just claimed in its output).

## Outcome

SUCCESS. `master` was pushed to `origin/master` at `119688cc848571259d517745d8a2b2ba4fdfbd5c`
via the `ci/staging` ritual. The required `test (rust + go)` check was earned
on that exact SHA before the branch push landed -- no bypass message, and all
build/test jobs (including the merge's Rust change in
`crates/me-cli/src/seal/mod.rs`) reported `success`. `assemble + sign + release`
correctly `skipped` (tag-gated, not a bypass). No commits were made by this
task; nothing was read as `.jsonl`.
