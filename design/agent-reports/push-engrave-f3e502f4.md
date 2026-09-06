# Push report — mnemonic-engrave master → f3e502f4

## SHA pushed
`f3e502f485d2e92aff56ced2a60c7c01540a9f96` (full 40-char SHA; matches `git -C /scratch/code/shibboleth/mnemonic-engrave rev-parse master` both before and after the push).

Pre-push state: `origin/master` was `86568d0c8800ac2ea49d94d63b9e9586855f6e69` (an ancestor, 40 commits behind — the H6 host stage: `h6-b` merge with ms-codec 0.9.0 / `--pack-preimage` / the `phrase:` record and its classes, the `h6-records` branch, and design records). `git status --short` was clean (no tracked-file modifications, no untracked files listed).

## Staging run
Run id: `34033407106` (`ci/staging`, `--repo bg002h/mnemonic-engrave`), triggered by `scripts/push-via-staging.sh master` run in the foreground.

Final per-job conclusions (`gh run view 34033407106 --repo bg002h/mnemonic-engrave --json jobs`):

```
test (rust + go): success
build me (linux-x86_64): success
build me-preview (all targets): success
build me (macos-x86_64): success
build me (linux-aarch64): success
build me (windows-x86_64): success
build me (macos-aarch64): success
assemble + sign + release: skipped
```

The required context `test (rust + go)` was `success` on this exact SHA. `assemble + sign + release` is gated on `refs/tags/v*` and correctly `skipped` for a non-tag push (per `.github/workflows/release.yml`).

Note: the script's own post-push straggler report (captured below) printed `build me (windows-x86_64):` with no conclusion because that job was still `in_progress` at the moment the script queried it, immediately after the required-context wait finished. It was polled separately afterward and completed `success` (job still running is expected — those non-required `build me (*)` jobs are informational only and do not gate the push). No zsh-related `history_purge` test failures appeared in CI (F-500 is a local-box-only issue per the brief; CI has zsh).

## Final push output (verbatim, `scripts/push-via-staging.sh master`)
```
== staging f3e502f485d2e92aff56ced2a60c7c01540a9f96 (branch master, 40 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]        HEAD -> ci/staging
== run 34033407106; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   86568d0c..f3e502f4  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]           ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (linux-x86_64): success
build me-preview (all targets): success
build me (macos-x86_64): success
build me (linux-aarch64): success
build me (windows-x86_64): 
build me (macos-aarch64): success
== OK: f3e502f485d2e92aff56ced2a60c7c01540a9f96 is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output (verified with `grep -i bypass` over the captured log — no match).

## Post-push verification
`git fetch origin && git rev-parse origin/master` → `f3e502f485d2e92aff56ced2a60c7c01540a9f96` — matches the pushed tip exactly. `ci/staging` ref was deleted by the script (confirmed by the `[deleted]` line above).

## Anything not done
Nothing outstanding from the brief. No tag, version bump, or publish was performed (none was requested). No source file was modified; no commit was made by this agent.
