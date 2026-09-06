# Push report: mnemonic-engrave master via ci/staging ritual

- Repo: `/scratch/code/shibboleth/mnemonic-engrave`
- GitHub: `bg002h/mnemonic-engrave`
- TIP recorded before push: `c011196d2b0f4788444d33432219922b1cd36cb4`
- origin/master before push: `4e71a8c4` (6 commits behind TIP: a push report, a brief, two agent reports, a spec STATUS line, and a continuity commit -- no crate code)

## Precondition check

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
```
Output: empty (no stale ci/staging ref). Precondition satisfied.

## Commands run

1. Record tip and precondition (above).
2. Inspected `scripts/push-via-staging.sh` (unmodified from repo) to confirm it stages `HEAD` to `ci/staging`, polls for the run, waits specifically on job `test (rust + go)`, verifies the local tip hasn't moved, then pushes `HEAD:master`, checks the output for a "Bypassed rule violations" string, and deletes the `ci/staging` ref.
3. Ran in the foreground, no background/watcher stopgap needed (script completed on its own):

```
$ ./scripts/push-via-staging.sh master
```

### Verbatim script output

```
== staging c011196d2b0f4788444d33432219922b1cd36cb4 (branch master, 6 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34012817228; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   4e71a8c4..c011196d  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
build me-preview (all targets): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-aarch64): success
test (rust + go): success
build me (linux-x86_64): success
build me (macos-x86_64): success
assemble + sign + release: skipped
== OK: c011196d2b0f4788444d33432219922b1cd36cb4 is on master with the required check earned
```

**No "Bypassed rule violations" line present anywhere in the output.** The final push (`4e71a8c4..c011196d HEAD -> master`) landed after the required context reported success -- the branch-protection rule was satisfied, not bypassed.

Run id: `34012817228`.

## Post-push verification (independent of the script)

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
c011196d2b0f4788444d33432219922b1cd36cb4          # MATCH with recorded TIP

$ gh run view 34012817228 --repo bg002h/mnemonic-engrave --json status,conclusion
{"conclusion":"success","status":"completed"}

$ gh run view 34012817228 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | .name + ": " + (.conclusion // .status)'
build me-preview (all targets): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-aarch64): success
test (rust + go): success
build me (linux-x86_64): success
build me (macos-x86_64): success
assemble + sign + release: skipped

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty -- ref deleted as expected)
```

`assemble + sign + release` is gated on `refs/tags/v*` and correctly reported `skipped` for this branch push -- consistent with the repo's documented CI trigger design, not a failure.

## Outcome

SUCCESS. `origin/master` on `bg002h/mnemonic-engrave` now points at `c011196d2b0f4788444d33432219922b1cd36cb4`, matching the pre-push local tip exactly. The required `test (rust + go)` context, and every non-required build job, concluded `success`; no bypass occurred; `ci/staging` was cleaned up. Nothing was committed as part of this push task.
