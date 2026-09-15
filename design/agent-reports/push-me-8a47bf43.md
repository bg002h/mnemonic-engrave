# Push round — mnemonic-engrave master (8a47bf43, hashlock-kinds cycle, 55 commits)

## Pre-flight

- Repo: `/scratch/code/shibboleth/mnemonic-engrave`, GitHub `bg002h/mnemonic-engrave`.
- `git status --porcelain` → empty (clean tree).
- `git rev-parse master` → `8a47bf435b9593d95a8220b529b4493b72a95d34` (matches
  briefed SHA; controller had frozen the repo for the whole window).
- `git rev-list --count origin/master..master` → **55**, matching the brief.
- Content: one session's design record for the hashlock-kinds cycle —
  verbatim agent reports, spec/follow-up edits, two new scripts, a
  continuity document, and one code change (`crates/me-cli/src/main.rs` +
  `crates/me-cli/tests/sysw_composer_cli.rs`, commit `63d7d87c`).

### Required status checks

`gh api repos/bg002h/mnemonic-engrave/branches/master/protection --jq '.required_status_checks.contexts'`
→ `["test (rust + go)"]`.

### Workflow trigger confirmation

Only one workflow file exists: `.github/workflows/release.yml`. Its `on:`
block:

```yaml
on:
  push:
    tags:
      - 'v*'
    branches:
      - master
      - 'ci/**'
  pull_request:
```

`ci/**` is present, so a `ci/staging` push builds this exact SHA and can earn
the `test (rust + go)` context before it reaches `master`. Confirmed `RUST_TOOLCHAIN: '1.85.0'` in the same file's `env:` block.

## Local gate — run on the CI toolchain (1.85.0), NOT the repo's rustup default (1.97.0-nightly)

`export PATH=/home/bcg/.cargo/bin:$PATH`, all commands run with `cargo +1.85.0`:

- `cargo +1.85.0 nextest run --locked --all-targets` → **647 passed, 2
  skipped**, 32.204s wall. Matches the brief's expected numbers exactly.
- `cargo +1.85.0 clippy --workspace --all-targets --locked -- -D warnings` →
  clean build, **0 warnings**, exit 0.
- `cargo +1.85.0 fmt --all -- --check` → exit 0 (clean).

Re-verified immediately after the gate, before pushing anything: `git status
--porcelain` still empty, `git rev-parse master` still
`8a47bf435b9593d95a8220b529b4493b72a95d34` — no drift during the local gate.

(Not independently re-run: the repo's default nightly toolchain reportedly
shows 3 clippy errors CI never sees, per the brief. Not reproduced here since
the CI-pinned toolchain is what gates the push; the brief already establishes
those 3 are pre-existing on `origin/master` and not to be "fixed".)

## Staging (`ci/staging`)

`git push origin master:refs/heads/ci/staging` → `* [new branch] master ->
ci/staging`.

`gh run list --repo bg002h/mnemonic-engrave --commit 8a47bf435b9593d95a8220b529b4493b72a95d34`
(full SHA, per the gh-queries-fail-silently-empty lesson) → run id
**34996582364**, workflow `release`, event `push`, ref `ci/staging`.

`gh run watch 34996582364 --repo bg002h/mnemonic-engrave --interval 15` ran to
completion (~3 minutes; this repo also runs Go tests against the
`third_party/seedhammer` submodule per the brief, so it is slower than a
Rust-only repo — no hang, just normal wall time).

Final conclusions (`gh run view 34996582364 --json conclusion,status,jobs`):

```json
{"conclusion":"success","jobs":[
  {"conclusion":"success","name":"test (rust + go)"},
  {"conclusion":"success","name":"build me (linux-x86_64)"},
  {"conclusion":"success","name":"build me-preview (all targets)"},
  {"conclusion":"success","name":"build me (macos-x86_64)"},
  {"conclusion":"success","name":"build me (windows-x86_64)"},
  {"conclusion":"success","name":"build me (macos-aarch64)"},
  {"conclusion":"success","name":"build me (linux-aarch64)"},
  {"conclusion":"skipped","name":"assemble + sign + release"}
],"status":"completed"}
```

`test (rust + go)` — the required context — is **success**, with every
sub-step (fmt, clippy, Rust test suite, Go tests (me-preview sidecar), Go
build+tests (ndef-roundtrip oracle)) green inside it. `assemble + sign +
release` correctly shows **skipped**: it is tag-gated (`refs/tags/v*`) and
this was a branch push, exactly as the repo's CLAUDE.md describes.

One informational annotation only, not a failure: Node.js 20 deprecation
notice on `actions/checkout@v4` / `actions/upload-artifact@v4`, forced to
Node 24 via `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`. Pre-existing repo config,
unrelated to this push.

## Tip-movement re-check before the real push

Immediately before `git push origin master`: `git status --porcelain` empty,
`git rev-parse master` still `8a47bf435b9593d95a8220b529b4493b72a95d34`. No
movement during the ~3-minute CI wait.

## `git push origin master`

```
To github.com:bg002h/mnemonic-engrave.git
   9524a1e9..8a47bf43  master -> master
```

Exit code 0. **No "Bypassed rule violations" text anywhere in the output.**
The push satisfied the branch-protection rule via the SHA's already-passing
`test (rust + go)` context (`strict: false`), rather than bypassing it.

## Cleanup and final verification

`git push origin --delete ci/staging` → `- [deleted] ci/staging`.

`git fetch origin master` then `git rev-parse origin/master` →
`8a47bf435b9593d95a8220b529b4493b72a95d34` — matches local `master` exactly.

## VERDICT: PUSHED, CLEAN.

All required and non-required jobs green on the staged SHA before the real
push; the real push carried no bypass; `origin/master` now sits at
`8a47bf43`, 55 commits ahead of where it started. `ci/staging` deleted.
