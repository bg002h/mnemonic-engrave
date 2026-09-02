# composer-S1 push report — repos 1+2

Executed per `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/702b37c9-e041-404f-8220-2456ff9c6bf3/scratchpad/push-brief-s1.md`.

## Repo 1: mnemonic-engrave — push `master`, tag `v0.8.0`

**Pre-check.** `git rev-parse master` = `38e3ed13eb0d903ae2d24e64edc830a9484dcc6e` (matched dispatch TIP). `git status --short` empty.

**Staging ritual** via `scripts/push-via-staging.sh master`:
- Staged to `ci/staging`, run id **33620279843**.
- Required context `test (rust + go)`: **success**.
- Final push output (verbatim):
  ```
  To github.com:bg002h/mnemonic-engrave.git
     b8e19eb..38e3ed1  HEAD -> master
  ```
  No "Bypassed rule violations" line. `ci/staging` deleted by the script.
- Straggler report (informational, non-required jobs), all **success**: `build me (windows-x86_64)`, `build me (linux-aarch64)`, `build me-preview (all targets)`, `build me (macos-x86_64)`, `build me (linux-x86_64)`, `build me (macos-aarch64)`; `assemble + sign + release`: `skipped` (expected — tag-gated).

**Post-push verify.** `git fetch origin && git rev-parse origin/master` = `38e3ed13eb0d903ae2d24e64edc830a9484dcc6e` (= TIP).

**Tag.** `git tag -a v0.8.0 38e3ed13eb0d903ae2d24e64edc830a9484dcc6e -F tag-me-v0.8.0.msg` → tag object `7868e97534dc76751a40247b9644a7ba6636f733`, `object 38e3ed13eb0d903ae2d24e64edc830a9484dcc6e` (confirmed via `git cat-file -p`). Push output:
```
To github.com:bg002h/mnemonic-engrave.git
 * [new tag]         v0.8.0 -> v0.8.0
```

**Release run** (triggered by the tag push), id **33620586356**, on SHA `38e3ed13eb0d903ae2d24e64edc830a9484dcc6e`. `gh run view … --json status,conclusion` → `status: completed`, `conclusion: success`. Per-job conclusions (all `success`): `build me-preview (all targets)`, `test (rust + go)`, `build me (macos-x86_64)`, `build me (linux-aarch64)`, `build me (windows-x86_64)`, `build me (macos-aarch64)`, `build me (linux-x86_64)`, `assemble + sign + release`.

**Release.** `gh release view v0.8.0 --json url,assets`:
- url: `https://github.com/bg002h/mnemonic-engrave/releases/tag/v0.8.0`
- assets: `mnemonic-engrave-v0.8.0-linux-amd64.tar.gz`, `mnemonic-engrave-v0.8.0-linux-arm64.tar.gz`, `mnemonic-engrave-v0.8.0-macos-amd64.tar.gz`, `mnemonic-engrave-v0.8.0-macos-arm64.tar.gz`, `mnemonic-engrave-v0.8.0-windows-amd64.zip`, `SHA256SUMS`, `SHA256SUMS.minisig`

No `cargo publish` performed. No version bump performed (none was in scope for this push).

## Repo 2: mnemonic-secret — push `master`, tag `ms-cli-v0.17.0`

**Pre-check.** `git rev-parse master` = `1068f389116928e4cd22e5b0658749d09b06611d` (matched dispatch tip, 3 commits ahead of `origin/master`). `git status --short` empty.

**Staging ritual (manual, no script here).**
1. `git push origin master:refs/heads/ci/staging` → `* [new branch] master -> ci/staging`.
2. Runs on SHA `1068f389116928e4cd22e5b0658749d09b06611d` (via `gh run list --commit`), both completed **success**:
   - `rust` — run id **33620942528**
   - `vendor-freshness` — run id **33620942655**
   - Per-job conclusions inside the `rust` run (`gh run view 33620942528 --json jobs`), all `success`: `miri (mlock unsafe)`, `fmt (pinned 1.95.0)`, `test (release, ubuntu-latest, mlock einval)`, `g6 invariant (cross-repo mlock.rs)`, `test (macos-latest)`, `musl compile/test (x86_64-unknown-linux-musl)`, `test (ms-codec)`, `freebsd compile-gate (whole-crate)`, `clippy`, `clippy (ms-codec)`, `test (ubuntu-latest)`, `musl compile/test (aarch64-unknown-linux-musl)`, `history purge (recipes RUN under real shells)`.
   - The four **required contexts** (`test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)`) confirmed `success` individually.
3. `git push origin master` output (verbatim):
   ```
   To github.com:bg002h/mnemonic-secret.git
      22d1869..1068f38  master -> master
   ```
   No "Bypassed rule violations" line.
4. `git push origin --delete ci/staging` → `- [deleted] ci/staging`.

**Post-push verify.** `git fetch origin && git rev-parse origin/master` = `1068f389116928e4cd22e5b0658749d09b06611d`.

**Tag.** `git tag -a ms-cli-v0.17.0 1068f389116928e4cd22e5b0658749d09b06611d -F tag-ms-cli-v0.17.0.msg` → tag object `7bbd810b6376e7a14eecf18739b316d17c282908`, `object 1068f389116928e4cd22e5b0658749d09b06611d` (confirmed). Push output:
```
To github.com:bg002h/mnemonic-secret.git
 * [new tag]         ms-cli-v0.17.0 -> ms-cli-v0.17.0
```

**`man-release.yml` trigger check.** Read the file header: `on: push: tags: ["ms-cli-v*"]` plus `workflow_dispatch` — fires on this tag.

**man-release run**, id **33621228397**, on SHA `1068f389116928e4cd22e5b0658749d09b06611d`. Overall `status: completed`, `conclusion: failure`. Per-job conclusions (`gh run view 33621228397 --json jobs`):
- `ms-man.tar.gz release asset`: **success**
- `repro / build-container (resolve BUILT-DIGEST)`: **success**
- `repro / repro-aarch64-musl (aarch64-unknown-linux-musl)`: `skipped`
- `repro / repro-substrate (x86_64-unknown-linux-musl)`: **failure**
- `repro / repro-x86_64-musl (x86_64-unknown-linux-musl)`: **failure**
- `musl-binary (${{ matrix.target }})`: `skipped`

First error (both failing jobs, identical cause; `gh run view 33621228397 --log-failed`), verbatim from `repro-substrate`:
```
error: failed to get `mnemonic-io-lib` as a dependency of package `ms-cli v0.17.0 (/__w/mnemonic-secret/mnemonic-secret/crates/ms-cli)`

Caused by:
  failed to load source for dependency `mnemonic-io-lib`

Caused by:
  Unable to update https://github.com/bg002h/mnemonic-engrave?rev=6c24e62823e6c1ac02aa3862cd6020674bf58544#6c24e628

Caused by:
  can't checkout from 'https://github.com/bg002h/mnemonic-engrave': you are in the offline mode (--offline)
##[error]Process completed with exit code 101.
```
This is the reproducibility gate's cached Docker image (`ghcr.io/bg002h/repro-musl-mnemonic-secret@sha256:77fc3d4c8e43a15ebccf9b8670fa4a56dde2fcd1ae7bfc6ee9fe4699ae7569da`) attempting `cargo build --locked --offline` against vendored sources that do not contain the git dependency `mnemonic-io-lib` pinned to the new rev `6c24e62823e6c1ac02aa3862cd6020674bf58544` in `mnemonic-engrave` — the container's vendor cache is stale relative to this commit's `Cargo.lock`. **This job is not one of the four required branch-protection contexts** (`test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)`), all of which had already passed on the staged SHA before `master` was pushed. The `master` push itself is unaffected and already verified above.

The release-asset job (`ms-man.tar.gz release asset`) succeeded independently of the repro gate and did publish. `gh release view ms-cli-v0.17.0 --json url,assets`:
- url: `https://github.com/bg002h/mnemonic-secret/releases/tag/ms-cli-v0.17.0`
- assets: `ms-man.tar.gz`

Per brief instruction (red release run → do not delete the tag, do not retry, record and return): **the tag was not deleted and no retry was attempted.**

A `fuzz-smoke` workflow (run id 33621227764) also fired on this tag push and concluded `success` — it is not named in the brief and not a required context; noted here for completeness only.

No `cargo publish` performed (ms-codec unchanged at 0.7.0, per brief).

## What could not be done
- The `man-release.yml` reproducibility gate (`repro-substrate`, `repro-x86_64-musl`) is red on `ms-cli-v0.17.0` due to a stale vendored dependency cache in the repro Docker image for the git dependency `mnemonic-io-lib` at rev `6c24e62823e6c1ac02aa3862cd6020674bf58544`. Not fixed, not retried — flagged for the controller. The release itself (man-page tarball) still published successfully; only the reproducibility verification is red.
