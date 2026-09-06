# Push report: mnemonic-engrave master via ci/staging ritual

Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)
Pre-push `origin/master`: `f6723516`
Recorded local tip (`TIP`): `75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c`
Contents beyond `f6723516`: 6 commits (push report, agent reports, H6 spec GREEN status, briefs, continuity -- no crate code).

## Commands run

```
TIP=$(git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master)
# => 75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c

git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
# => (empty) -- precondition satisfied

./scripts/push-via-staging.sh master
```
(run in the foreground from `/scratch/code/shibboleth/mnemonic-engrave`, no backgrounding, no separate watcher; the script completed on its own without needing manual `gh run view` polling)

```
git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
# => 75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c  (matches TIP)

gh run view 34001649602 --repo bg002h/mnemonic-engrave \
  --json status,conclusion,headSha,jobs \
  -q '{status:.status, conclusion:.conclusion, headSha:.headSha, jobs: [.jobs[] | {name:.name, conclusion:.conclusion}]}'
```

## Run id and per-job conclusions

Run: `34001649602` (workflow triggered by the `ci/staging` push of `75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c`)
Overall: `status: completed`, `conclusion: success`, `headSha: 75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c`

| Job | Conclusion |
| --- | --- |
| build me-preview (all targets) | success |
| build me (macos-aarch64) | success |
| test (rust + go) *(required context)* | success |
| build me (linux-x86_64) | success |
| build me (linux-aarch64) | success |
| build me (macos-x86_64) | success |
| build me (windows-x86_64) | success |
| assemble + sign + release | skipped (gated on `refs/tags/v*`, expected) |

## Verbatim script output (`scripts/push-via-staging.sh master`)

```
== staging 75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c (branch master, 6 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34001649602; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   f6723516..75f00b56  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
build me-preview (all targets): success
build me (macos-aarch64): success
test (rust + go): success
build me (linux-x86_64): success
build me (linux-aarch64): success
build me (macos-x86_64): success
build me (windows-x86_64): success
assemble + sign + release: skipped
== OK: 75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output -- the final push (`f6723516..75f00b56  HEAD -> master`) was earned by the required `test (rust + go)` context on `ci/staging`, not bypassed. The `ci/staging` ref was deleted immediately after the successful branch push, as shown above.

## Post-push verification

```
git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
```
=> `75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c`, matching the recorded `TIP` exactly. `origin/master` advanced `f6723516` -> `75f00b56` with no bypass.

## Outcome

SUCCESS. `mnemonic-engrave` `master` pushed to `75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c` via the `ci/staging` ritual; required context `test (rust + go)` earned on run `34001649602` before the branch push; no "Bypassed rule violations"; `origin/master` confirmed at `75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c` post-fetch.
