# Push report: mnemonic-engrave master via ci/staging

- Repo: `/scratch/code/shibboleth/mnemonic-engrave`
- GitHub: `bg002h/mnemonic-engrave`
- Prior `origin/master`: `e638434a`
- TIP recorded before push: `c6969eaeab4b5a503733cc9cf0ea44c0b9a1bcbe`
- Commits pushed: 5 ahead of `e638434a` (push report, gate report, H6 plan fold, a brief, and continuity -- no crate code)

## Commands run

```
TIP=$(git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master)
git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
```
Precondition check: `ls-remote origin refs/heads/ci/staging` returned empty (exit 0) -- no pre-existing staging ref.

```
./scripts/push-via-staging.sh master
```
Run in the foreground from `/scratch/code/shibboleth/mnemonic-engrave`, no backgrounding, ran to completion without needing a manual poll.

```
git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
gh run view 34010721913 --repo bg002h/mnemonic-engrave --json databaseId,headSha,status,conclusion,jobs -q '{databaseId, headSha, status, conclusion, jobs: [.jobs[] | {name, conclusion}]}'
```

## Run ID and per-job conclusions

Run id: `34010721913` (GitHub Actions run for commit `c6969eaeab4b5a503733cc9cf0ea44c0b9a1bcbe`)

```json
{
  "databaseId": 34010721913,
  "headSha": "c6969eaeab4b5a503733cc9cf0ea44c0b9a1bcbe",
  "status": "completed",
  "conclusion": "success",
  "jobs": [
    {"name": "test (rust + go)", "conclusion": "success"},
    {"name": "build me (linux-aarch64)", "conclusion": "success"},
    {"name": "build me-preview (all targets)", "conclusion": "success"},
    {"name": "build me (macos-x86_64)", "conclusion": "success"},
    {"name": "build me (macos-aarch64)", "conclusion": "success"},
    {"name": "build me (windows-x86_64)", "conclusion": "success"},
    {"name": "build me (linux-x86_64)", "conclusion": "success"},
    {"name": "assemble + sign + release", "conclusion": "skipped"}
  ]
}
```

Required context `test (rust + go)`: **success**. `assemble + sign + release` is gated on `refs/tags/v*` and correctly reported `skipped` for this `ci/**` staging push (not a tag).

## Verbatim script output (`scripts/push-via-staging.sh master`)

```
== staging c6969eaeab4b5a503733cc9cf0ea44c0b9a1bcbe (branch master, 5 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34010721913; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   e638434a..c6969eae  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (linux-aarch64): success
build me-preview (all targets): success
build me (macos-x86_64): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
== OK: c6969eaeab4b5a503733cc9cf0ea44c0b9a1bcbe is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output -- push satisfied the required check rather than bypassing it.

## Post-push verification

- `git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin` then `rev-parse origin/master` => `c6969eaeab4b5a503733cc9cf0ea44c0b9a1bcbe` -- **matches** the recorded TIP.
- `ls-remote origin refs/heads/ci/staging` after completion => empty -- staging ref cleaned up as expected (script's own `git push origin --delete ci/staging` succeeded, consistent with no bypass/forensics path taken).

## Outcome

SUCCESS. `master` on `bg002h/mnemonic-engrave` now points at `c6969eaeab4b5a503733cc9cf0ea44c0b9a1bcbe`, earned via the `ci/staging` ritual with the required `test (rust + go)` context passing on that exact SHA. No commits were made by this report-writing task.
