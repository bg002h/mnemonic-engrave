# Push mnemonic-engrave master 0a4acef via ci/staging ritual

Repo: `/scratch/code/shibboleth/mnemonic-engrave` (GitHub `bg002h/mnemonic-engrave`)
Target: master tip `0a4acef891f6868ba568014c160d7ab78af929a2`
Prior origin/master: `1e6191602faaed8e88faf45ad587edac167be53d` (15 commits behind)

## Preconditions (checked before running the script)

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master
0a4acef891f6868ba568014c160d7ab78af929a2

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty -- no ci/staging ref present)

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/master
1e6191602faaed8e88faf45ad587edac167be53d	refs/heads/master
```

Both preconditions satisfied: local master tip matches `0a4acef`, and `ci/staging`
did not already exist on the remote.

## Command run

```
$ /scratch/code/shibboleth/mnemonic-engrave/scripts/push-via-staging.sh master
```

Run in the foreground, full output captured to `/tmp/push-engrave-0a4acef.log` via `tee`.

## Verbatim script output

```
== staging 0a4acef891f6868ba568014c160d7ab78af929a2 (branch master, 15 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33967600299; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   1e61916..0a4acef  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
build me-preview (all targets): success
build me (macos-x86_64): success
test (rust + go): success
build me (windows-x86_64): success
build me (linux-aarch64): success
build me (linux-x86_64): success
build me (macos-aarch64): success
assemble + sign + release: skipped
== OK: 0a4acef891f6868ba568014c160d7ab78af929a2 is on master with the required check earned
```

No "Bypassed rule violations" text appears anywhere in the output. The final
`git push origin HEAD:master` line is `1e61916..0a4acef  HEAD -> master`
(a clean fast-forward; git prints no other summary lines for this push).

## Run details (verified via `gh`, full SHA + `--repo`)

Workflow run: `33967600299` (`https://github.com/bg002h/mnemonic-engrave/actions/runs/33967600299`)

```
$ gh run view 33967600299 --repo bg002h/mnemonic-engrave --json databaseId,headSha,status,conclusion,jobs
```

- `headSha`: `0a4acef891f6868ba568014c160d7ab78af929a2`
- `status`: `completed`
- overall `conclusion`: `success`

Per-job conclusions:

| Job | Conclusion |
| --- | --- |
| build me-preview (all targets) | success |
| build me (macos-x86_64) | success |
| test (rust + go) (**required context**) | success |
| build me (windows-x86_64) | success |
| build me (linux-aarch64) | success |
| build me (linux-x86_64) | success |
| build me (macos-aarch64) | success |
| assemble + sign + release | skipped (tag-gated on `refs/tags/v*`; expected for a non-tag push) |

## Post-push verification

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
0a4acef891f6868ba568014c160d7ab78af929a2
```

`origin/master` now equals the intended tip `0a4acef891f6868ba568014c160d7ab78af929a2`.
`ci/staging` was deleted by the script (confirmed in the script's own output:
`- [deleted]  ci/staging`).

## Outcome

SUCCESS. `master` on `bg002h/mnemonic-engrave` is now `0a4acef891f6868ba568014c160d7ab78af929a2`,
earned via the `ci/staging` SHA-first ritual, required context `test (rust + go)`
satisfied (not bypassed), no "Bypassed rule violations" message at any point,
tip did not move during the window (script's own tip-check passed silently),
and `origin/master` fetch-confirmed to match.
