# F-280 close-out: wire `fmt`/`clippy` into CI, reformat the drift

Worktree: `/scratch/code/shibboleth/_work/f280/mnemonic-engrave`, branch
`fix/f280-ci-fmt`, starting commit `3609b0c`. Not pushed (per instruction —
the dispatcher runs the `ci/staging` merge ritual).

## Commits, in order

1. `1fbc732` — **F-280 step 1/2: wire cargo fmt and clippy into CI, before
   the reformat.** Adds `rustfmt, clippy` components to the pinned-toolchain
   install step, then `fmt` and `clippy --all-targets --locked -- -D
   warnings` steps in `.github/workflows/release.yml`'s `test (rust + go)`
   job, positioned right after toolchain install and before Go setup
   (matches `mnemonic-transaction`'s `ci.yml` ordering: fail fast and
   cheaply before the heavier steps). Committed while the tree is RED —
   deliberately, per F-280's own "add the gate first" ordering.
2. `06c2c81` — **F-280 step 2/2: reformat the 13 files.** Pure `cargo
   +1.85.0 fmt`, nothing else touched.
3. `2dcaf94` — **followups: F-361.** Records a second, independent gap
   found while closing this one (below).

## `cargo fmt --check` — before / after

```
before (commit 1fbc732, and unchanged tree back through 3609b0c):
  cargo +1.85.0 fmt --check  ->  exit 1
  13 files, 76 hunks ("Diff in ..." lines)

  F-280's own text records 14 files / 77 hunks, measured at ba1f3ec (before
  the P1 crate rows). Re-measuring here on 3609b0c shows 13/76 -- one hunk's
  worth of drift resolved itself somewhere in the P1 rows between those two
  commits. Not investigated further; not this branch's scope.

after (commit 06c2c81):
  cargo +1.85.0 fmt --check  ->  exit 0
```

Files touched by the reformat (13, matches the fmt-check file list exactly):
`crates/me-cli/src/main.rs`, `crates/me-cli/src/sysw/{expect,mod,mt,tx}.rs`,
`crates/me-cli/tests/{argv_secret_guard,cli,expect_kinds,seal_cli,sysw_cli,
terminal_destination,world_readable_output}.rs`,
`crates/mnemonic-io-lib/src/remedy.rs`. 474 insertions / 154 deletions.
Spot-checked several hunks by hand: all are rustfmt line-wrap / trailing-comma
reflow, plus one match-arm gaining braces because the single-expression form
no longer fit the line (`main.rs:1566`) — no logic change. `third_party/
seedhammer` (submodule) was not touched; nothing outside the 13 files above
was reformatted.

## Behavioural-equivalence check (the trap named in the brief)

- `cargo nextest run --locked` after the reformat: **430 passed, 1 skipped**
  — identical to the baseline the brief specified.
- `cargo +1.85.0 clippy --all-targets --locked -- -D warnings`: **exit 101**,
  both before and after the reformat, and the finding list is byte-identical
  (diffed the `-->` / `error:` lines from both runs). So the reformat itself
  introduced nothing; the clippy redness is pre-existing and unrelated to
  fmt. See F-361 below — this is the one thing in this task that did not go
  as the brief assumed.

## Deliberate-violation test (fmt gate can actually fail)

Inserted `let    x = 1;` (irregular spacing) into
`crates/mnemonic-io-lib/src/remedy.rs`, uncommitted:

```
cargo +1.85.0 fmt --check  ->  exit 1
  Diff in .../crates/mnemonic-io-lib/src/remedy.rs:151:
```

Names the exact mutated file. Reverted (`git checkout --
crates/mnemonic-io-lib/src/remedy.rs`), re-ran:

```
cargo +1.85.0 fmt --check  ->  exit 0
```

Confirmed the working tree was clean again before proceeding (`git status
--short` empty).

## `actionlint`

`actionlint .github/workflows/release.yml` → **exit 0**, checked after each
commit that touched the workflow file.

## Toolchain check the brief asked for — and the finding it turned up

The brief said: use the toolchain CI pins (`+1.85.0`), and if the pinned
toolchain and local default disagree, report it rather than picking the
smaller diff. For `fmt` they agree in shape (nightly rustfmt was not used;
`+1.85.0` was used throughout, matching F-280's own measurement toolchain).

For **clippy** they do not agree, and the disagreement is not cosmetic:

```
cargo +1.85.0 clippy --all-targets --locked -- -D warnings  ->  exit 101
cargo        clippy --all-targets --locked -- -D warnings  ->  exit 0
```

`+1.85.0` is clippy 0.1.85 (2025-02-17, matches `RUST_TOOLCHAIN`); the
repo's default toolchain here is nightly, clippy 0.1.97 (2026-04-27). 13
distinct findings across 8 files under the pinned version, all pre-existing
(confirmed unrelated to the fmt reformat: identical finding list before and
after it). Filed as **F-361** in `design/FOLLOWUPS.md`, with each file/line,
what the two "unknown lint" cases reveal (an `#[allow(clippy::
manual_is_multiple_of)]` naming a lint that doesn't exist yet on the pinned
clippy — the same pinned-toolchain-never-exercised pattern F-280 describes,
just via clippy instead of fmt), and a check that the one lint that could
plausibly be a real bug (`record.rs:294`'s operator-precedence warning on
`hi << 4 | lo`) is not one — `<<` already binds tighter than `|`, matching
the intent documented in the comment immediately above it.

**Left unfixed here, deliberately — this is the one scope call this agent
made rather than following the brief literally.** The brief's verification
recipe assumed `clippy --all-targets --locked -- -D warnings` would exit 0
after the reformat; it does not, for reasons the reformat has no part in.
Fixing the 13 findings is real code editing across 8 files (rewriting
`map(...).collect()` string-building patterns, one `#[allow]` fix, added
parens, a bool simplification) — not the mechanical, machine-verified
reformat this branch's scope covers, and not something F-280's own text
(which measured fmt only) asked for. The `clippy` CI step is still wired in,
per the explicit instruction and to match the sibling repo's shape, so it is
a real, exercised gate from the moment this branch lands — which also means
**the branch tip is RED on the required `test (rust + go)` check** until
F-361 closes. Flagging this prominently rather than either (a) silently
omitting the clippy step, or (b) silently expanding scope to fix 8 files of
lint debt the dispatch brief never anticipated or bounded.

**Recommendation:** close F-361 before merging `fix/f280-ci-fmt` to
`master`, or explicitly decide to merge with a known-red required check and
fix forward immediately.

## Scope

Touched only `.github/workflows/release.yml`, the 13 reformatted files,
`design/FOLLOWUPS.md` (F-361 only), and this report. No other follow-ups
worked, no plan documents touched, nothing pushed.
