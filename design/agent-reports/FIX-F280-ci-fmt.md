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

---

## Addendum: F-361 closed (coordinator dispatch, same branch/worktree)

The coordinator reproduced the same `rc 101` independently and directed
closing F-361 rather than deferring it, since the `clippy` step this branch
wires in makes the branch tip RED on the required `test (rust + go)` check
until it does. Two more commits, in order, both on `fix/f280-ci-fmt`:

1. `4728109` — **the unknown-lint guard.** Both `#[allow(clippy::
   manual_is_multiple_of)]` sites (`record.rs`, `wire.rs`) name a lint that
   exists on the repo's default nightly clippy (0.1.97) but not on the
   pinned clippy (0.1.85) — under `-D warnings` on the pinned toolchain,
   the `#[allow]` attribute itself is an unknown-lint error. Fixed by
   stacking `#[allow(unknown_lints)]` above each one: the pinned toolchain
   stops erroring on the unrecognised name, and the newer toolchain's
   `allow(clippy::manual_is_multiple_of)` still works, since
   `unknown_lints` only applies where the named lint actually is unknown.
   **Both exit 0 for this class of error immediately after this commit**
   (verified before moving on, per the coordinator's acceptance test):
   ```
   cargo +1.85.0 clippy --all-targets --locked -- -D warnings  -> exit 101 (11 findings remained, was 13)
   cargo        clippy --all-targets --locked -- -D warnings  -> exit 0
   ```
2. `813ad62` — **the mechanical fixes**, described below.
3. `bc2465c` — **followups: F-361 CLOSED**, with the corrected count.

### The masking discovery — the count in this file's first half was wrong

Fixing the two unknown-lint sites let `cargo +1.85.0 clippy` progress past
the `lib` target for the first time, and it immediately surfaced **1 more
finding** (`tests/sysw_cli.rs:2209`, a `format_collect`) that the first
pass never reached because the `lib` target's own errors had been aborting
the build before clippy could even attempt the integration-test targets.
Fixing that one let the build progress into `bin "me"`, which surfaced
**5 more** (`main.rs`: 3 `clippy::precedence`, 2 `clippy::format_collect`).
Each round was re-verified with a fresh `cargo +1.85.0 clippy --all-targets
--locked -- -D warnings` before declaring the next round the last one; the
round that finally returned exit 0 confirmed nothing further was hiding
behind a target that still hadn't compiled.

**So the original count in this report and in F-361 (13 findings / 8
files) was an undercount, not the true state** — it was only what the
build got far enough to show. True total: **18 lint instances across 11
files** (2 unknown-lint in commit 1 above; 16 more — 4 `precedence`, 2
`nonminimal_bool`/`bool_comparison` on one line, 10 `format_collect` — in
commit 2). This is now corrected in F-361's closure text; the original
entry text was left in place and the correction appended, per this repo's
own fold discipline (append, don't silently rewrite a number a future
reader might `git blame`).

### The mechanical fixes (commit `813ad62`)

- **`clippy::precedence` (4 sites), checked against Rust's actual operator
  table before touching anything — not merely applying clippy's suggested
  diff:** `sysw/record.rs`'s `hi << 4 | lo` and `main.rs`'s three
  `(n >> N) & 63` base64 terms. `<<`/`>>` bind tighter than `&`/`|` in
  Rust, so every one of these already parsed as intended; the fix is
  explicit parens for a human reader, not a behaviour change. **None of
  these is a real bug** — the one place this mattered most,
  `record.rs:294`, already carries a comment from a prior cycle explaining
  the intended semantics via a cargo-mutants equivalent-mutant note, and
  the added parens match that stated intent exactly.
- **`clippy::format_collect` (10 sites):** `main.rs` ×2, `seal/{crypto,mod,
  pubhash}.rs`, `sysw/{pubhash,record,tx}.rs` (`tx.rs` ×2),
  `sysw/vectors.rs`, `tests/sysw_cli.rs`. Each `.iter().map(|x|
  format!(...)).collect()` rewritten to clippy's own suggested `.fold(
  String::new(), |mut acc, x| { let _ = write!(acc, ...); acc })`, with a
  local `use std::fmt::Write as _;` added per file. Two of the ten
  (`seal/crypto.rs`, `seal/mod.rs`) needed the import placed *inside*
  `mod tests` rather than at the file top: the helper function there is
  test-only, so a module-level import would be unused (and therefore
  itself a `-D warnings` error) in a non-test `lib` build — the exact same
  masking mechanism as the section above, caught before it could produce
  a second round-trip.
- **`clippy::nonminimal_bool` + `clippy::bool_comparison` (1 site, 2
  lints):** `sysw/coverage.rs`'s `assert!(... == !unbuilt.is_empty(), ...)`
  → `!= unbuilt.is_empty()`. Verified the rewrite is boolean-equivalent
  (`X == !Y` ⟺ `X != Y` for all bool `X`, `Y`) before applying it, not
  merely trusted to clippy's suggestion.

Diffs are minimal and local throughout — no restructuring, no renames, no
shared helper extracted even though the `format!`-to-hex pattern repeats
across 8 files (a consolidation would have been the "nicer" fix and was
explicitly out of scope per the coordinator's instruction).

### Final verification, all in one clean run after the last commit

```
cargo +1.85.0 fmt --check                                    -> exit 0
cargo +1.85.0 clippy --all-targets --locked -- -D warnings    -> exit 0
cargo        clippy --all-targets --locked -- -D warnings    -> exit 0
cargo nextest run --locked                                    -> 430 passed, 1 skipped
actionlint .github/workflows/release.yml                      -> exit 0
```

`cargo nextest run --locked`'s 430/1 is byte-identical to every prior
measurement in this report — none of the 18 fixes changed behaviour,
including the one hand-edited test (`argv_refuses_every_secret_class_too`
in `tests/sysw_cli.rs`, confirmed individually PASS in the nextest log).

### Nothing found that was a real defect rather than a style finding

Checked deliberately for this, per the coordinator's instruction to stop
and report separately rather than fold in a genuine bug found by a lint:
every `precedence` site already parsed as intended (verified against
Rust's operator table, not assumed), and `format_collect` /
`bool_comparison` / `nonminimal_bool` are all lints clippy itself
documents as non-behavioural rewrites. Nothing to split out.

### Scope (addendum)

This round additionally touched: `crates/me-cli/src/main.rs`,
`crates/me-cli/src/sysw/{record,wire,coverage,pubhash,tx}.rs`,
`crates/me-cli/src/seal/{crypto,mod,pubhash}.rs`,
`crates/me-cli/src/sysw/vectors.rs`, `crates/me-cli/tests/sysw_cli.rs`,
`design/FOLLOWUPS.md` (F-361 closure only), and this report. Still not
pushed; still only F-280/F-361 on `fix/f280-ci-fmt`.
