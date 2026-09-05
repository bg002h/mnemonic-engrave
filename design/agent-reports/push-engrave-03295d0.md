# Push report: mnemonic-engrave master via ci/staging ritual

Repo: `/scratch/code/shibboleth/mnemonic-engrave`
GitHub: `bg002h/mnemonic-engrave`
Date: 2026-09-05

## Precondition and tip

```
$ TIP=$(git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master); echo "TIP=$TIP"
TIP=03295d07c645637929359193ca6e75a896ecc6fb

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)

$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/master
119688cc848571259d517745d8a2b2ba4fdfbd5c	refs/heads/master
```

origin/master before push: `119688cc848571259d517745d8a2b2ba4fdfbd5c` (matches the value given in the dispatch brief). Local `master` was 9 commits ahead. Precondition (`ci/staging` empty) satisfied.

## Command run

```
$ ./scripts/push-via-staging.sh master
```
Run in the foreground, no backgrounding, no separate watcher.

## Verbatim script output (tail, in full)

```
== staging 03295d07c645637929359193ca6e75a896ecc6fb (branch master, 9 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 33997308260; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   119688cc..03295d07  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
build me-preview (all targets): success
build me (linux-x86_64): success
build me (linux-aarch64): success
test (rust + go): success
build me (macos-x86_64): success
build me (windows-x86_64): success
build me (macos-aarch64): success
assemble + sign + release: skipped
== OK: 03295d07c645637929359193ca6e75a896ecc6fb is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output. The script's own final-push line (`OUT` variable, echoed verbatim) is the `119688cc..03295d07  HEAD -> master` block above -- a clean fast-forward push, no bypass message.

Run id: **33997308260**.

The script completed end-to-end within its own polling loop; no separate `gh run watch` / manual poll was needed because the run concluded before the script's internal wait loop timed out.

## Per-job conclusions (independently re-queried post-hoc)

```
$ gh run view 33997308260 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | .name + ": " + (.conclusion // .status)'
build me-preview (all targets): success
build me (linux-x86_64): success
build me (linux-aarch64): success
test (rust + go): success
build me (macos-x86_64): success
build me (windows-x86_64): success
build me (macos-aarch64): success
assemble + sign + release: skipped

$ gh run view 33997308260 --repo bg002h/mnemonic-engrave --json status,conclusion,headSha
{"conclusion":"success","headSha":"03295d07c645637929359193ca6e75a896ecc6fb","status":"completed"}
```

`test (rust + go)` (the required context) = **success**. `assemble + sign + release` = skipped, as expected for a `ci/**` ref (that job gates on `refs/tags/v*`).

## Post-push verification

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave fetch origin
$ git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse origin/master
03295d07c645637929359193ca6e75a896ecc6fb
```

`origin/master` == `TIP` (`03295d07c645637929359193ca6e75a896ecc6fb`). Confirmed match.

```
$ git -C /scratch/code/shibboleth/mnemonic-engrave ls-remote origin refs/heads/ci/staging
(empty)
```

`ci/staging` ref confirmed deleted post-push (script's own `git push origin --delete ci/staging` step, independently reconfirmed).

## Outcome

**SUCCESS.** `master` advanced `119688cc` -> `03295d07` on `bg002h/mnemonic-engrave` via the staging ritual; the required `test (rust + go)` context earned the check on the exact SHA before the branch push, no bypass. No commits were made to `master` during the window (single foreground script invocation, no interleaved commands touching the branch).
