# Push report — mnemonic-engrave master via ci/staging — 2026-09-02

**SHA pushed:** `329e8c8e0a9b971df18c8eb25a9b941f207432ca`

**Pre-push state:** `git rev-parse master` = `329e8c8e0a9b971df18c8eb25a9b941f207432ca`; `git status --porcelain=v1` empty (clean tree). `origin/master` before fetch = `401697f7e9e394a385a65b8af4084514d82e1093` (ancestor, 11 commits behind).

**Command run (foreground):** `./scripts/push-via-staging.sh master`

**Staging run:** id `33711988277`

**Per-job conclusions** (from `gh run view 33711988277 --repo bg002h/mnemonic-engrave --json jobs`, independently re-queried after the script's own report — verbatim, identical both times):

```
build me-preview (all targets): success
test (rust + go): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (macos-x86_64): success
build me (linux-aarch64): success
assemble + sign + release: skipped
```

Required context `test (rust + go)`: **success**. `assemble + sign + release` is gated on `refs/tags/v*` and correctly `skipped` for a branch push — no tag/version/publish action occurred, none was requested.

**Final push output (verbatim, full script stdout/stderr, `tee`d to scratchpad):**

```
== staging 329e8c8e0a9b971df18c8eb25a9b941f207432ca (branch master, 11 ahead)
remote: 
remote: Create a pull request for 'ci/staging' on GitHub by visiting:        
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging        
remote: 
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33711988277; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   401697f..329e8c8  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
build me-preview (all targets): success
test (rust + go): success
build me (windows-x86_64): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (macos-x86_64): success
build me (linux-aarch64): success
assemble + sign + release: skipped
== OK: 329e8c8e0a9b971df18c8eb25a9b941f207432ca is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output (checked with `grep -i bypass` over the captured transcript — no match).

**Post-push verification (`git fetch origin && git rev-parse origin/master`):** `329e8c8e0a9b971df18c8eb25a9b941f207432ca` — matches the pushed SHA and matches local `master`.

**`ci/staging` ref:** deleted by the script (`git push origin --delete ci/staging` succeeded, shown above).

**What could not be done / was not attempted (as instructed):** no tag, no version bump, no publish — `assemble + sign + release` correctly skipped rather than run. No source files modified, no commits made, no sub-agents spawned.
