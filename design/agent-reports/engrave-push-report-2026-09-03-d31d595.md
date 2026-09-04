# Push report -- mnemonic-engrave master @ d31d595

Repo: `/scratch/code/shibboleth/mnemonic-engrave`, remote `bg002h/mnemonic-engrave`, branch `master`.

## SHA pushed

`d31d595f81198b91ef35c35085615ffbe0558e21` (40-char, verified via `git rev-parse master` before running the script; matched the brief).

Pre-push state: `origin/master` was `04be1112b68a7afc62ee2821dd604c1d2a850f8e` (an ancestor, six commits behind -- design records only). `git status --short` showed only the untracked brief file `design/agent-briefs/engrave-push-brief-d31d595.md`; no tracked file was modified.

## Staging run

Ran `scripts/push-via-staging.sh master` in the foreground from the repo root (not backgrounded).

- Staging push: `HEAD:refs/heads/ci/staging` -> new branch created.
- Run id: **33834385328**
- Required context: `test (rust + go)`

Per-job conclusions, verbatim (`gh run view 33834385328 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | .name + ": " + (.conclusion // .status)'`, re-queried independently after the script exited):

```
test (rust + go): success
build me (macos-x86_64): success
build me-preview (all targets): success
build me (windows-x86_64): success
build me (linux-aarch64): success
build me (macos-aarch64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
```

`assemble + sign + release` is gated on `refs/tags/v*` and correctly `skipped` for a branch push -- not a failure.

## Final push output (verbatim, from the script's own stdout)

```
== staging d31d595f81198b91ef35c35085615ffbe0558e21 (branch master, 6 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33834385328; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   04be111..d31d595  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (macos-x86_64): success
build me-preview (all targets): success
build me (windows-x86_64): success
build me (linux-aarch64): success
build me (macos-aarch64): success
build me (linux-x86_64): success
assemble + sign + release: skipped
== OK: d31d595f81198b91ef35c35085615ffbe0558e21 is on master with the required check earned
```

No "Bypassed rule violations" line appears anywhere in the output. `ci/staging` was deleted by the script (`[deleted] ci/staging` line above).

## Post-push verification

`git fetch origin && git rev-parse origin/master`:

```
d31d595f81198b91ef35c35085615ffbe0558e21
```

Equals the tip pushed.

## What I could not do / did not do

Nothing outstanding. No file was modified, no commit was made, no tag was created, nothing was published. Did not read any `.jsonl` file. `assemble + sign + release` did not run (`skipped`) because this was a branch push, not a tag push -- expected and out of scope for this brief.
