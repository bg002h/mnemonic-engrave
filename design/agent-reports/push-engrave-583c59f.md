# Push report: mnemonic-engrave master via ci/staging ritual

- Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)
- Prior `origin/master`: `ac6a19c` (per task brief)
- Local `master` tip (TIP), recorded before any action:
  `583c59fbfef55826806785954b1569c1a1d29666` (12 commits ahead of `ac6a19c`;
  reports, plan/spec folds, briefs, FOLLOWUPS and continuity -- no crate code)

## Precondition check

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
583c59fbfef55826806785954b1569c1a1d29666

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty -- no pre-existing ci/staging ref)
```

## Ritual run

Command (foreground, no backgrounding, no watcher wait):

```
$ ./scripts/push-via-staging.sh master
```

Verbatim tail (full run captured to /tmp/push-engrave-583c59f.log, not committed):

```
== staging 583c59fbfef55826806785954b1569c1a1d29666 (branch master, 12 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33985937652; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   ac6a19c..583c59f  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (macos-x86_64): success
build me (windows-x86_64): success
build me-preview (all targets): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (linux-aarch64): success
assemble + sign + release: skipped
== OK: 583c59fbfef55826806785954b1569c1a1d29666 is on master with the required check earned
```

**No "Bypassed rule violations" line appeared anywhere in the run output** --
the final `master` push was satisfied by the earned required check, not
bypassed. The script completed in one foreground invocation; no separate
polling loop was needed (the script itself waited on the required context to
conclude before pushing to `master`).

## Independent verification (full SHAs, `--repo` on every `gh` call)

```
$ gh run view 33985937652 --repo bg002h/mnemonic-engrave \
    --json databaseId,headSha,status,conclusion,jobs \
    -q '{databaseId, headSha, status, conclusion, jobs: [.jobs[] | {name, conclusion}]}'
{
  "conclusion": "success",
  "databaseId": 33985937652,
  "headSha": "583c59fbfef55826806785954b1569c1a1d29666",
  "status": "completed",
  "jobs": [
    {"name": "test (rust + go)",               "conclusion": "success"},
    {"name": "build me (macos-x86_64)",         "conclusion": "success"},
    {"name": "build me (windows-x86_64)",       "conclusion": "success"},
    {"name": "build me-preview (all targets)",  "conclusion": "success"},
    {"name": "build me (linux-x86_64)",         "conclusion": "success"},
    {"name": "build me (macos-aarch64)",        "conclusion": "success"},
    {"name": "build me (linux-aarch64)",        "conclusion": "success"},
    {"name": "assemble + sign + release",       "conclusion": "skipped"}
  ]
}
```

`assemble + sign + release` is gated on `refs/tags/v*` (per this repo's
`.github/workflows/release.yml`), so `skipped` on a plain branch push is
expected, not a gap.

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
583c59fbfef55826806785954b1569c1a1d29666

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty -- ci/staging ref deleted after the ritual)
```

`origin/master` == TIP == `583c59fbfef55826806785954b1569c1a1d29666`. Matches.

## Outcome

**SUCCESS.** `master` pushed via `ci/staging` (run `33985937652`, run URL
`https://github.com/bg002h/mnemonic-engrave/actions/runs/33985937652`); required
context `test (rust + go)` concluded `success` before the branch push; all
build jobs `success`; no bypass; `origin/master` confirmed at TIP
`583c59fbfef55826806785954b1569c1a1d29666`; `ci/staging` ref removed. No
commits were made by this task.
