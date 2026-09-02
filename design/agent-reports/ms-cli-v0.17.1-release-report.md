# ms-cli 0.17.1 release report

Repo: `bg002h/mnemonic-secret`. Starting state verified: `git status --short` empty,
`git rev-parse master` = `a069c77c9ec98f6d0b6972d295c4da548db9d2fc` (matched expected),
`origin/master` in sync, `rust-toolchain.toml` channel `1.85.0`.

## Step 1 — branch, edits, gates, commit

```
git checkout -b release-0.17.1
```

`crates/ms-cli/Cargo.toml` line 3: `version = "0.17.0"` → `version = "0.17.1"`.

```
cargo check --offline -q -p ms-cli
```
Exit 0, no output.

```
grep -A1 'name = "ms-cli"' Cargo.lock
```
```
name = "ms-cli"
version = "0.17.1"
```

```
cargo build --locked -q
```
Exit 0, no output.

`CHANGELOG.md`: inserted the 0.17.1 entry directly above the existing
`## ms-cli [0.17.0] — 2026-09-02` heading, verbatim as specified in the dispatch
brief. Verified with `sed -n '1,25p' CHANGELOG.md` after the edit — text matches
the brief exactly, byte for byte.

### Gates

```
cargo fmt --all --check
```
Exit 0, no output.

```
cargo clippy --locked --all-targets -- -D warnings
```
Exit 0. Tail of output: `Finished \`dev\` profile [optimized + debuginfo] target(s) in 3.81s`.

```
cargo nextest run --locked
```
Summary line: `Summary [ 0.136s] 477 tests run: 477 passed, 11 skipped` — matches
the expected 477.

```
bash ci/repro/vendor-freshness.sh
```
```
vendor-freshness: resolving Cargo.lock against committed vendor/ (offline, locked; mnemonic-io-lib rev 6c24e62823e6c1ac02aa3862cd6020674bf58544) ...
vendor-freshness: OK — vendor/ satisfies Cargo.lock.
```

### Commit

`git status --short` before staging showed exactly the three intended files:
```
 M CHANGELOG.md
 M Cargo.lock
 M crates/ms-cli/Cargo.toml
```

```
git add CHANGELOG.md Cargo.lock crates/ms-cli/Cargo.toml
git commit -s -F <msgfile>
```
Commit message body:
```
ms-cli 0.17.1 -- release infrastructure only (F-324): the reproducible musl gate can see the mnemonic-io-lib git source; 0.17.0 shipped without binaries

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
```
Git appended `Signed-off-by: bg <goss.brian@gmail.com>` via `-s`.

Result: `[release-0.17.1 3bf9fba]  3 files changed, 14 insertions(+), 2 deletions(-)`.
Full commit SHA: `3bf9fba457b0245c41b35d3aaab0f18fcd4cd2c3`.

## Step 2 — staging ritual

```
git push origin release-0.17.1:refs/heads/ci/staging
```
```
remote:
remote: Create a pull request for 'ci/staging' on GitHub by visiting:
remote:      https://github.com/bg002h/mnemonic-secret/pull/new/ci/staging
remote:
To github.com:bg002h/mnemonic-secret.git
 * [new branch]      release-0.17.1 -> ci/staging
```

Runs on SHA `3bf9fba457b0245c41b35d3aaab0f18fcd4cd2c3`
(`gh run list --repo bg002h/mnemonic-secret --commit 3bf9fba457b0245c41b35d3aaab0f18fcd4cd2c3 --json databaseId,name,status,conclusion`),
both completed `success`:

| run | databaseId | conclusion |
|---|---|---|
| `vendor-freshness` | 33625665452 | success |
| `rust` | 33625665425 | success |

Per-job conclusions, `gh run view <id> --json jobs`:

`rust` (33625665425):

| job | databaseId | conclusion |
|---|---|---|
| clippy (ms-codec) | 100232689904 | success |
| musl compile/test (aarch64-unknown-linux-musl) | 100232690114 | success |
| test (ms-codec) | 100232690198 | success |
| freebsd compile-gate (whole-crate) | 100232690210 | success |
| fmt (pinned 1.95.0) | 100232690219 | success |
| g6 invariant (cross-repo mlock.rs) | 100232690265 | success |
| miri (mlock unsafe) | 100232690280 | success |
| history purge (recipes RUN under real shells) | 100232690288 | success |
| test (macos-latest) | 100232690303 | success |
| test (release, ubuntu-latest, mlock einval) | 100232690327 | success |
| test (ubuntu-latest) | 100232690373 | success |
| musl compile/test (x86_64-unknown-linux-musl) | 100232690388 | success |
| clippy | 100232690426 | success |

Required contexts `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`,
`clippy (ms-codec)` all `success` — confirmed.

### Merge to master

```
git checkout master
git fetch origin --quiet
git rev-parse origin/master
```
→ `a069c77c9ec98f6d0b6972d295c4da548db9d2fc` (unchanged since session start —
freeze held).

```
git merge --ff-only release-0.17.1
git push origin master
```
```
Updating a069c77..3bf9fba
Fast-forward
 CHANGELOG.md             | 12 ++++++++++++
 Cargo.lock               |  2 +-
 crates/ms-cli/Cargo.toml |  2 +-
 3 files changed, 14 insertions(+), 2 deletions(-)
To github.com:bg002h/mnemonic-secret.git
   a069c77..3bf9fba  master -> master
```
No "Bypassed rule violations" line — the required check on this exact SHA was
satisfied, not bypassed.

```
git push origin --delete ci/staging
```
```
To github.com:bg002h/mnemonic-secret.git
 - [deleted]         ci/staging
```

```
git fetch && git rev-parse origin/master
```
→ `3bf9fba457b0245c41b35d3aaab0f18fcd4cd2c3` = REL_SHA, matching the tested commit.

## Step 3 — tag

```
git tag -a ms-cli-v0.17.1 3bf9fba457b0245c41b35d3aaab0f18fcd4cd2c3 -F <msgfile>
```
Tag message (verified via `git show ms-cli-v0.17.1 --no-patch`) matches the
brief's text verbatim. Tag is annotated (`-a`) per the spec, not GPG-signed
(`git tag -v` correctly reports "no signature found" — expected, `-s` was not
requested).

Tag object SHA: `de9060c45fb61aeb9bf826b391ec115ed836c61e` (points to commit
`3bf9fba457b0245c41b35d3aaab0f18fcd4cd2c3`).

```
git push origin ms-cli-v0.17.1
```
```
To github.com:bg002h/mnemonic-secret.git
 * [new tag]         ms-cli-v0.17.1 -> ms-cli-v0.17.1
```

### man-release.yml run for the tag

```
gh run list --repo bg002h/mnemonic-secret --workflow man-release.yml --json databaseId,headSha,event,conclusion,status --limit 5
```
Identified run `33626008575`, event `push`, headSha
`3bf9fba457b0245c41b35d3aaab0f18fcd4cd2c3`.

Final overall result: `{"conclusion":"success","status":"completed"}`.

Per-job conclusions (`gh run view 33626008575 --json jobs`):

| job | databaseId | conclusion |
|---|---|---|
| ms-man.tar.gz release asset | 100233793746 | success |
| derive git-source pins | 100233793874 | success |
| repro / build-container (resolve BUILT-DIGEST) | 100233836541 | success |
| repro / repro-aarch64-musl (aarch64-unknown-linux-musl) | 100233837924 | **skipped** (by design, per brief) |
| repro / repro-substrate (x86_64-unknown-linux-musl) | 100234085308 | success |
| repro / repro-x86_64-musl (x86_64-unknown-linux-musl) | 100234085379 | success |
| musl-binary (x86_64-unknown-linux-musl) | 100234822188 | success |
| musl-binary (aarch64-unknown-linux-musl) | 100234822254 | success |

Every job `success` except `repro-aarch64-musl`, `skipped` exactly as the brief
predicted.

### Release assets

```
gh release view ms-cli-v0.17.1 --repo bg002h/mnemonic-secret --json url,assets --jq '{url, assets: [.assets[].name]}'
```
```
{
  "assets": [
    "ms-0.17.1-aarch64-linux-musl.tar.gz",
    "ms-0.17.1-x86_64-linux-musl.tar.gz",
    "ms-man.tar.gz",
    "PROVENANCE.aarch64.txt",
    "PROVENANCE.x86_64.txt",
    "SHA256SUMS.aarch64",
    "SHA256SUMS.x86_64"
  ],
  "url": "https://github.com/bg002h/mnemonic-secret/releases/tag/ms-cli-v0.17.1"
}
```
Seven asset names, all matching the expected list from the brief.

## Step 4 — annotate ms-cli-v0.17.0

Before (`gh release view ms-cli-v0.17.0 --json body --jq .body`):
```
**Full Changelog**: https://github.com/bg002h/mnemonic-secret/compare/ms-cli-v0.16.0...ms-cli-v0.17.0
```

Appended line:
```
NOTE (2026-09-02): this release carries no musl binaries (F-324, release-infrastructure defect); use ms-cli-v0.17.1, which is the same code with the binaries, PROVENANCE and SHA256SUMS.
```

```
gh release edit ms-cli-v0.17.0 --notes-file <file>
```

After (re-read to confirm):
```
**Full Changelog**: https://github.com/bg002h/mnemonic-secret/compare/ms-cli-v0.16.0...ms-cli-v0.17.0

NOTE (2026-09-02): this release carries no musl binaries (F-324, release-infrastructure defect); use ms-cli-v0.17.1, which is the same code with the binaries, PROVENANCE and SHA256SUMS.
```

## Summary of key identifiers

- Release commit (master tip after merge): `3bf9fba457b0245c41b35d3aaab0f18fcd4cd2c3`
- Tag object: `ms-cli-v0.17.1` → `de9060c45fb61aeb9bf826b391ec115ed836c61e` → commit `3bf9fba4...`
- Staging runs: `rust` 33625665425 (success), `vendor-freshness` 33625665452 (success)
- Tag release run: `man-release.yml` 33626008575 (success, one job skipped by design)
- Release URL: https://github.com/bg002h/mnemonic-secret/releases/tag/ms-cli-v0.17.1
- CI was never red at any step; no `ci/staging` branch or tag was deleted for
  failure recovery — none was needed.

Local branch `release-0.17.1` deleted after the fast-forward merge (fully
merged into `master`, no data loss).
