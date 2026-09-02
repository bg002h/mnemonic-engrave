# Push report — mnemonic-engrave master via ci/staging — 2026-09-02b

## TIP
`88b4a4aa971274e10aea8eded04d3716426ae371`

Pre-push state: `git status --short` empty; `master` was 22 commits ahead of
`origin/master` (`38e3ed13eb0d903ae2d24e64edc830a9484dcc6e`).

## Staging run
- Run id: `33627354744` (repo `bg002h/mnemonic-engrave`)
- `headSha`: `88b4a4aa971274e10aea8eded04d3716426ae371` (confirmed via
  `gh run view 33627354744 --repo bg002h/mnemonic-engrave --json headSha`)
- Required job **`test (rust + go)`** conclusion, verbatim: `success`

Full per-job conclusions from the same run (informational, non-required jobs
included):

```
test (rust + go): success
build me (macos-x86_64): success
build me (linux-aarch64): success
build me-preview (all targets): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (windows-x86_64): success
assemble + sign + release: skipped
```

`assemble + sign + release` is gated on `refs/tags/v*`; it correctly stayed
`skipped` since no tag was pushed.

## Ritual used
`scripts/push-via-staging.sh master`, run in full, no manual fallback needed.

## Final push output, verbatim (via the script)

```
== staging 88b4a4aa971274e10aea8eded04d3716426ae371 (branch master, 22 ahead)
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-engrave/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-engrave.git
 * [new branch]      HEAD -> ci/staging
== run 33627354744; waiting for required context: test (rust + go)
To github.com:bg002h/mnemonic-engrave.git
   38e3ed1..88b4a4a  HEAD -> master
To github.com:bg002h/mnemonic-engrave.git
 - [deleted]         ci/staging
== post-push straggler report (non-required jobs, informational):
test (rust + go): success
build me (macos-x86_64): success
build me (linux-aarch64): success
build me-preview (all targets): success
build me (linux-x86_64): success
build me (macos-aarch64): success
build me (windows-x86_64): success
assemble + sign + release: skipped
== OK: 88b4a4aa971274e10aea8eded04d3716426ae371 is on master with the required check earned
```

No "Bypassed rule violations" line appeared anywhere in the output — the
staging step was not bypassed.

## Post-push verification
- `git fetch origin && git rev-parse origin/master` → `88b4a4aa971274e10aea8eded04d3716426ae371`
  (equals TIP)
- `ci/staging` ref deleted (confirmed in script output above)
- No tag was pushed; `assemble + sign + release` remained `skipped` — this
  push published nothing

## What I could not do
Nothing. The scripted ritual ran to completion on the first attempt; no
manual fallback, no CI failure, no tip movement during the window.
