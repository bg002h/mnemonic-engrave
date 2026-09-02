# Push report: mnemonic-engrave master → 1c7aac4

- **SHA pushed:** `1c7aac4a5a301d5776b64333e1b1d95247fc3316` (was 8 ahead of `origin/master` = `50aa76d73ad763abc2965f31b6ead03d2aec27cd`, 0 behind, per controller freeze).
- **Pre-push verification:** `git rev-parse master` = `1c7aac4a5a301d5776b64333e1b1d95247fc3316` (matched brief). `git status --short` = empty (clean tree, no tracked modifications).
- **Method:** `scripts/push-via-staging.sh master` (unmodified, ran as-is).

## Staging run
- **Run ID:** `33646121156` (`gh run list --commit 1c7aac4a5a301d5776b64333e1b1d95247fc3316`)
- **Per-job conclusions** (`gh run view 33646121156 --repo bg002h/mnemonic-engrave --json jobs`), verbatim:
```
test (rust + go): success
build me (linux-x86_64): success
build me (linux-aarch64): success
build me (macos-x86_64): success
build me (windows-x86_64): success
build me (macos-aarch64): success
build me-preview (all targets): success
assemble + sign + release: skipped
```
`assemble + sign + release` is gated on `refs/tags/v*` and correctly skipped for a `ci/staging` push (per CLAUDE.md note on `.github/workflows/release.yml`).

## Final push output (verbatim, from script stdout)
```
== staging 1c7aac4a5a301d5776b64333e1b1d95247fc3316 (branch master, 8 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33646121156; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   50aa76d..1c7aac4  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (linux-x86_64): success
build me (linux-aarch64): success
build me (macos-x86_64): success
build me (windows-x86_64): success
build me (macos-aarch64): success
build me-preview (all targets): success
assemble + sign + release: skipped
== OK: 1c7aac4a5a301d5776b64333e1b1d95247fc3316 is on master with the required check earned
```
No "Bypassed rule violations" line appeared anywhere in the output.

## Post-push verification
- `git fetch origin && git rev-parse origin/master` → `1c7aac4a5a301d5776b64333e1b1d95247fc3316` — matches the pushed tip exactly.
- `ci/staging` ref deleted by the script.

## Anything not done
Nothing outstanding. No tag, no version bump, no publish were performed (none requested). CI was green throughout; the "if CI is red" branch of the brief was not triggered.
