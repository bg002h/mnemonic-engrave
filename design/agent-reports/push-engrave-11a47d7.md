# Push report: mnemonic-engrave master 11a47d7 via staging ritual

Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)
Target tip: `11a47d7d4d18c4792afc3de25870dea65345bdd8`
Prior `origin/master`: `1dbe5d3`

## Preconditions checked

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
11a47d7d4d18c4792afc3de25870dea65345bdd8

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty output)
```

Both preconditions satisfied: local `master` tip matched the required SHA, and no
stale `ci/staging` ref existed on origin before starting.

## Command run (foreground, no backgrounding)

```
$ ./scripts/push-via-staging.sh master
```

## Verbatim script output (tail; full output captured, nothing elided)

```
== staging 11a47d7d4d18c4792afc3de25870dea65345bdd8 (branch master, 5 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33972116843; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   1dbe5d3..11a47d7  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me-preview (all targets): success
build me (macos-x86_64): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (linux-aarch64): success
assemble + sign + release: skipped
== OK: 11a47d7d4d18c4792afc3de25870dea65345bdd8 is on master with the required check earned
```

The script ran to completion in the foreground within the tool timeout; no
separate poll of `gh run view` was needed (its own wait loop covered it).

**No "Bypassed rule violations" line appears anywhere in the output** — checked
by grep against the full captured log (`grep -i bypass`, zero matches).

## Independent verification (post-script)

Run id: `33972116843`

```
$ gh run view 33972116843 --repo bg002h/mnemonic-engrave --json status,conclusion
{"conclusion":"success","status":"completed"}

$ gh run view 33972116843 --repo bg002h/mnemonic-engrave --json jobs --jq '.jobs[] | {name, conclusion}'
{"conclusion":"success","name":"test (rust + go)"}
{"conclusion":"success","name":"build me-preview (all targets)"}
{"conclusion":"success","name":"build me (macos-x86_64)"}
{"conclusion":"success","name":"build me (windows-x86_64)"}
{"conclusion":"success","name":"build me (linux-x86_64)"}
{"conclusion":"success","name":"build me (macos-aarch64)"}
{"conclusion":"success","name":"build me (linux-aarch64)"}
{"conclusion":"skipped","name":"assemble + sign + release"}
```

`assemble + sign + release` is `skipped` as expected — that job is gated on
`refs/tags/v*` and this run was triggered by a branch push (`ci/staging`), not
a tag.

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
11a47d7d4d18c4792afc3de25870dea65345bdd8

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty output)
```

`origin/master` now equals the target tip `11a47d7d4d18c4792afc3de25870dea65345bdd8`,
and the `ci/staging` staging ref no longer exists on origin.

## Outcome

SUCCESS. `master` was pushed to `11a47d7d4d18c4792afc3de25870dea65345bdd8` on
`bg002h/mnemonic-engrave` with the required `test (rust + go)` check earned
against that exact commit SHA (no bypass), via run `33972116843`. No commits
were made to this repo as part of this task; this report was written as a new,
uncommitted file.
