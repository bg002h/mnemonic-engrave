# Push report — mnemonic-engrave master via ci/staging

- SHA pushed: `d118c5731375fbd9d56f2cbe6428ecc27a290954`
- Pre-push local tip (`git rev-parse master`): `d118c5731375fbd9d56f2cbe6428ecc27a290954` (matched expected)
- Pre-push `origin/master` (cached): `19f5ca2eb48f003db24ebe5f388f138ad85ba0e9` (7 commits behind, ancestor)
- Working tree: clean per `git status --porcelain=v1 --branch` (`## master...origin/master [ahead 7]`, no modified tracked files)

## Staging run

- Command: `./scripts/push-via-staging.sh master` (repo root), run in foreground, no timeout hit.
- Run id: `34047229373`
- Required context: `test (rust + go)`

Full script output, verbatim:

```
== staging d118c5731375fbd9d56f2cbe6428ecc27a290954 (branch master, 7 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34047229373; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   19f5ca2e..d118c573  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me-preview (all targets): success
build me (macos-x86_64): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-aarch64): success
assemble + sign + release: skipped
== OK: d118c5731375fbd9d56f2cbe6428ecc27a290954 is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output.

## Per-job conclusions (run 34047229373, re-queried independently after the fetch below)

Query: `gh run view 34047229373 --repo bg002h/mnemonic-engrave --json jobs -q '.jobs[] | .name + ": " + (.conclusion // .status)'`

```
test (rust + go): success
build me-preview (all targets): success
build me (macos-x86_64): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (windows-x86_64): success
build me (linux-aarch64): success
assemble + sign + release: skipped
```

`assemble + sign + release` is gated on `refs/tags/v*` and correctly `skipped` for this `ci/**`/branch push — expected, not a failure.

## Post-fetch verification (independent of the script)

- `git fetch origin` — ran clean.
- `git rev-parse origin/master` → `d118c5731375fbd9d56f2cbe6428ecc27a290954` (equals the pushed tip).
- `git rev-parse master` (local) → `d118c5731375fbd9d56f2cbe6428ecc27a290954` (unmoved, as expected under freeze).

## Outcome

SUCCESS. `d118c573` is on `origin/master`, the required `test (rust + go)` context reports `success` for this exact SHA, and the final push carried no bypass. `ci/staging` was deleted by the script. No tags, version bumps, or releases were touched. No source file was modified and no commit was made by this agent.

## Could not do / not applicable

Nothing was left undone. The freeze window held (no commits landed on `master` during the run), so no re-staging was required.
