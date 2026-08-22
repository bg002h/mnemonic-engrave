# Push round — mnemonic-toolkit (Bitcoin Core addresses) + mnemonic-engrave (Phase 1b reports)

Two-repo round, run one at a time as instructed: mnemonic-toolkit first,
then mnemonic-engrave.

## 1. mnemonic-toolkit

### Clean-tree check

`git status --porcelain | grep -vE '^\?\?'` → empty (0 tracked changes).
38 untracked pre-existing files present (unchanged in count from the
previous round), as briefed. Treated as clean per standing instruction for
this repo.

### Tip

- Branch `master`. Tip SHA: `e95e80a8e13b45528daef8ff01f5bed2f41b5522` (40
  chars confirmed via `wc -c`). One commit since `8a564a08`:
  `e95e80a8` — "feat(export-wallet): --format bitcoin-core-addresses, the
  one Core route".

### Why this round's ritual was structured differently

The coordinator flagged in advance that `manual` was **RED on
`origin/master`** at the previously-pushed `8a564a08` (an MD012 lint
failure from Phase 1's `docs/manual/` additions), and explained the root
cause: only `examples.yml` and `rust.yml` in this repo carry `'ci/**'` in
their push-trigger branches. **Seven other workflows — including
`manual.yml` and `vendor-freshness.yml` — trigger only on `[main,
master]`**, so `ci/staging` structurally cannot exercise them; they only
run once code actually lands on `master`. This round's commit was
represented as the fix for that lint failure, with an explicit instruction
to confirm `manual`'s conclusion on the new SHA after the real push, not
assume it from a green `ci/staging` result.

### Staging (`ci/staging`) — what it could see

`git push origin master:refs/heads/ci/staging --force` →
`8a564a08..e95e80a8 master -> ci/staging`. Verified via fresh `git fetch
origin` that `git rev-parse master` == `git rev-parse origin/ci/staging`
== `e95e80a8e13b45528daef8ff01f5bed2f41b5522`.

Four workflows fired on `ci/staging` for this SHA (no `manual`, no
`vendor-freshness`, no `fuzz-smoke` — expected, per the trigger-scope
explanation above and confirmed below for the latter two):

- `examples` (`databaseId 32583963677`) → **success**
- `rust` (`databaseId 32583963651`) → **success**, 13/13 jobs green,
  including `test (ubuntu-latest)` and `clippy` (both required contexts).
  First `gh run watch` attempt exceeded a single 10-minute blocking call
  (this suite includes an 11-minute `test (ubuntu-latest)` job and a
  10-minute `musl build+test` job); re-issued the same watch in the
  foreground per the standing correction against ending the turn on a
  background wait, and it completed.
- `bitcoind-differential` (`databaseId 32583963652`) → **success**
- `sibling-pin-check` (`databaseId 32583963681`) → **success**

All three required contexts (`examples`, `test (ubuntu-latest)`, `clippy`)
confirmed green by name.

### Tip-movement check, before the real push

`git rev-parse master` == `e95e80a8e13b45528daef8ff01f5bed2f41b5522`
(exact match to staged/tested SHA); fresh `git fetch origin` showed
`origin/master` unchanged at `8a564a08dc3b27fa209102b0168bf0f1cf3f8b18`,
confirmed an ancestor of local `master` (clean fast-forward guaranteed);
re-confirmed 0 tracked changes. No movement observed.

### `git push origin master`

```
To github.com:bg002h/mnemonic-toolkit.git
   8a564a08..e95e80a8  master -> master
```
No "Bypassed rule violations" text present. Genuine fast-forward.

### Post-push verification — the workflows `ci/staging` could NOT see

This push to the real `master` triggered a second, fuller wave of runs.
Watched and confirmed each by name:

- **`manual` (`databaseId 32585012834`) → `success`.** This is the
  workflow that was RED at `8a564a08`. Job `build` includes "Audit manual
  (lint + verify-examples with real mnemonic binary)", which completed
  green. **Explicitly confirmed rather than assumed, as instructed: the
  fix landed and `manual` is now green on `master`.**
- `technical-manual` (`databaseId 32585012793`) → `success` (a workflow
  not previously seen on `ci/staging` either — also `[main, master]`-only)
- `examples` (`databaseId 32585012818`) → `success`
- `bitcoind-differential` (`databaseId 32585012836`) → `success`
- `sibling-pin-check` (`databaseId 32585012870`) → `success`
- `rust` (`databaseId 32585012928`) → `success`, 13/13 jobs green again

**`vendor-freshness` and `fuzz-smoke` still did not appear at all**, on
either `ci/staging` or the real `master` push. Checked why rather than
assuming, per this session's established practice: `git diff --stat
8a564a08..e95e80a8` touches `crates/mnemonic-toolkit/src/{cmd/
export_wallet.rs, cmd/restore.rs, derive_address.rs,
descriptor_builder/allow.rs, wallet_export/*}`, `tests/*`, fixture files,
`CHANGELOG.md`, `docs/manual/*`, and `.github/workflows/
bitcoind-differential.yml` — **none of `Cargo.lock`, `Cargo.toml`,
`crates/**/Cargo.toml`, or `vendor/**`** (checked
`vendor-freshness.yml`'s path filter directly), and none of `fuzz/**`,
`crates/mnemonic-toolkit/src/parse_descriptor.rs`, `src/lib.rs`, or
`crates/wc-codec/**` (checked `fuzz-smoke.yml`'s path filter directly).
**Both skips are correct path-filter behavior for this specific commit** —
not evidence the structural gap the coordinator described (both being
`[main, master]`-only, unreachable from `ci/staging`) is fixed. That gap
is still real; this commit simply didn't touch anything either workflow
cares about, so it can't demonstrate the gap either way.

`git push origin --delete ci/staging` → `- [deleted] ci/staging`. Fresh
fetch confirmed `origin/master` ==
`e95e80a8e13b45528daef8ff01f5bed2f41b5522`.

**VERDICT: PUSHED.** All 10 workflow runs across both waves (`ci/staging`
+ real `master`) green, including explicit confirmation that `manual` is
now green (was RED before this push). No tip movement. No bypass text.

---

## 2. mnemonic-engrave

### Clean-tree check

`git status --porcelain` → **empty** (fully clean, not just 0-tracked —
the coordinator had removed two agent worktrees that were previously
showing as untracked). Confirmed independently.

### Tip

- Branch `master`. Tip SHA: `305efd0e147dbc0f7f248565996b5324f2d034e3` (40
  chars confirmed via `wc -c`). Four commits since `28e6ff5`: `bf5fec7`
  ("reports: Phase 1 implemented — and two places the plan was wrong"),
  `9557077` ("reports: whole-diff review of Phase 1 — 0C/1I/2M, and a hole
  a green suite hid"), `dfa16a9` ("plan: the status line said 'no code'
  after the code shipped"), `305efd0` ("reports: Phase 1b implemented —
  and it caught a gate I never ran").

### What actually changed, checked before writing anything below

`git diff --stat 28e6ff5..305efd0` touches exactly 4 files —
`design/PLAN_wallet_file_export.md`, `design/agent-reports/
IMPL_export_phase1.md`, `IMPL_export_phase1b.md`,
`R1_export_phase1_wholediff.md`. **All markdown, no `.rs` or `.go`
anywhere.** Matches the coordinator's description exactly. Consequently, a
green `test (rust + go)` for this SHA confirms no regression rather than
validating new source.

### Staging

`git push origin master:refs/heads/ci/staging --force` →
`* [new branch] master -> ci/staging`. Verified via fresh `git fetch
origin` that `git rev-parse master` == `git rev-parse origin/ci/staging`
== `305efd0e147dbc0f7f248565996b5324f2d034e3`.

### CI

Workflow `release` (`databaseId 32586152050`), `event: push`, `headSha`
confirmed matching. Watched actively via `gh run watch` (blocked in the
foreground). Overall `conclusion: success`. Per-job, via `gh run view
--json headSha,status,conclusion,jobs`: `test (rust + go)` → success,
`build me (linux-aarch64/macos-aarch64/macos-x86_64/linux-x86_64/
windows-x86_64)` → all success, `build me-preview (all targets)` →
success, `assemble + sign + release` → `skipped` (expected, tag-gated).
8 of 8 jobs green.

### Tip-movement check, before the real push

`git rev-parse master` == `305efd0e147dbc0f7f248565996b5324f2d034e3`
(exact match); fresh `git fetch origin` showed `origin/master` unchanged
at `28e6ff5de9c1e260838c47fb49c0445fa8decb06`, confirmed ancestor of local
`master`; `git status --porcelain` fully empty. No movement observed.

### `git push origin master`

```
To github.com:bg002h/mnemonic-engrave.git
   28e6ff5..305efd0  master -> master
```
No "Bypassed rule violations" text present. Genuine fast-forward.

`git push origin --delete ci/staging` → `- [deleted] ci/staging`. Fresh
fetch confirmed `origin/master` ==
`305efd0e147dbc0f7f248565996b5324f2d034e3`.

**VERDICT: PUSHED.**

---

## Summary

| Repo | SHA | Verdict |
| --- | --- | --- |
| mnemonic-toolkit | `e95e80a8e13b45528daef8ff01f5bed2f41b5522` | PUSHED — `manual` confirmed green (was RED before), all 10 workflow runs across both waves green |
| mnemonic-engrave | `305efd0e147dbc0f7f248565996b5324f2d034e3` | PUSHED — markdown-only, 8/8 jobs green |

No bitcoind processes touched. No tip movement observed in either repo's
CI-wait window.
