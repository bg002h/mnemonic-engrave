# PLAN: mnemonic-io-lib 0.1.2 safe write (bughunt-1119 row 7, ruling T5)

Author: Claude Sonnet 5.5 (claude-sonnet-5-5), 2026-11-22, SLIM plan (no code blocks). Spec:
`design/SPEC_io_lib_safe_write.md` (this branch; spec head e98d809a closed GREEN after r3, plus the commit
`spec: r3 M1 + r2 M2 exact texts`). The spec decides every holder-visible text, refusal and expected output;
this plan decides only order, commits, commands and gates. Where the spec is silent the plan says STOP.

TRACE rows: the Rule cell and its plan legs are binding. The Rule cells are the "expected after the fix"
column of spec section 8. This plan adds no plan leg: every case the spec decides is already pinned by a
spec row, and a case it does not decide is listed under "Spec sentences with no row" below, not given a leg.

## Machine-verified before review

`scripts/io-lib-safe-write-gate.sh plan` (run, output in the report): every coverage-map quote occurs in the
spec; every row id the map names exists in this plan; every spec section-8 row name is in this plan with the
same id; the plan has no fenced code block; the three spec texts of the `spec: r3` commit are present.
Modes `red1`, `red2`, `green` run at the build commits (steps T2 to T5) and each was shown going RED on the
tree that exists today (the report pastes it). NOT covered: whether a mutant (table below) fails its row
(run by hand, pasted into the build report); anything on Windows or macOS beyond compiling (`cargo check`
only: no row runs there); the release steps R1 to R4 and the consumer steps C1 to C5 (other repos, other
people); the correctness of the wording of any test name (the names come from the spec, byte for byte).

## Step order (phase 1: mnemonic-engrave)

Base and branch: the coordinator names them in the build brief. Worktree not under `/tmp`; `$HOME/.cargo/bin`
first in PATH; `export RUSTUP_TOOLCHAIN=1.85.0` (the rustup default here is nightly; the gate exports it for its own steps only) so that `cargo --version` says 1.85.0 for every other cargo call, mutant runs included (the repo's CI pin; `scripts/lint-gate.sh` uses +1.85.0);
every cargo call `--locked`; nextest `--build-jobs 6 -j 6`; never `--release`; a private `CARGO_TARGET_DIR`.

- [T1] Read `CLAUDE.md` and `AGENTS.md` of the repo. Run `gate red1` BEFORE writing anything: it must fail with
  none of W1-W4 found (it runs only the 2 existing rows, so the summary line and the failing-set check FAIL; this is the gate going RED on a tree without the rows).
- [T2] **RED-1 commit.** In `crates/mnemonic-io-lib/src/write.rs` `mod tests` only: the module attribute
  becomes `#[cfg(test)]`; `use std::os::unix::fs::PermissionsExt`, `mode_of` and the two existing rows get
  `#[cfg(unix)]`; add the `child()` harness and rows W1-W4, W5-W11 and W17 exactly as spec section 8 names
  them. Gate `red1` must show: 15 tests run (2 existing, W5-W11, W17, `child`, W1-W4), exactly W1-W4 FAIL and
  11 pass. Commit message pastes each of W1-W4's failure line next to the spec's "measured today" cell. If a
  row's measured failure differs from the spec's cell, STOP and report (the spec's number is the claim).
- [T3] **RED-2 commit.** Add S1-S10. `gate red2` must show the unit-test build failing with `E0425` naming
  `same_file`; paste the whole compiler output in the commit message (the spec says "record that output";
  the spec's r3 nit N2 notes an `E0433` on `Endpoint::` may appear too: record it, it is not a STOP).
  S1-S10 reach the new names through `use super::*;`, added to `mod tests` in this commit (not
  `use super::{same_file, Endpoint}`: that gives E0432 only and fails gate `red2`).
- [T4] **GREEN commit.** Add the private `Step` enum, the thread-local seam, the temp-name function and rows
  W12-W16; implement `write_private` (spec section 3, steps 1 to 11), `Endpoint` and `same_file` (sections 2
  and 5, Windows differences per section 6); make the doc edits of spec section 2 item 5 (module doc, the
  sentence at `lib.rs:73`, `channel.rs` lines that say "tightening the OPEN file"). `crates/mnemonic-io-lib/Cargo.toml` `[dependencies]` stays
  empty. Gate `IOLIB_PRE_BUMP=1 scripts/io-lib-safe-write-gate.sh green` must pass (it skips only the two
  version checks; 30 rows in `write::tests::`, all pass; whole workspace nextest; doctests;
  lint-gate; Windows and macOS compile checks). The mutant table is then run by hand.
- [T5] **Version commit** (separate, last; plan decision, the spec only says the PR carries it):
  `crates/mnemonic-io-lib/Cargo.toml` 0.1.1 to 0.1.2 and the one `Cargo.lock` entry. Gate plain `green` (no `IOLIB_PRE_BUMP`) again.
- [T6] The PR (full track through `ci/staging`, coordinator lands it). FOLLOWUPS lines are listed in the
  build report, not added in the PR.

Child-process harness (decided by spec section 8; the plan adds only the checks): the child re-executes
`current_exe()` with `--exact write::tests::child --nocapture`; under nextest the row ids are
`write::tests::<name>` and `child` is itself a nextest test that returns at once without
`MNEMONIC_IO_LIB_CHILD`, so it counts as one passing row in every mode. W1 and W2 run the child through
`sh -c` with `ulimit -f 1`; the gate refuses to run as root (W4 and S10 panic on their precondition then, as
the spec says).

Windows and macOS behaviour: the me CI Rust job is ubuntu only, so no row runs on Windows or macOS. The module
attribute `#[cfg(test)]` makes the non-unix rows (W5, W11, W12, W13 in its non-unix form, S1, S3 to S6) compile
on Windows; the gate proves that by `cargo check --tests` for `x86_64-pc-windows-gnu` and `x86_64-apple-darwin`
(both installed here). The release workflow's Windows and macOS builds compile the library part. md's and mk's
own Windows/macOS legs exercise `write_private` after their pin bump (steps C1, C2).

## Rows (ids; names come from spec section 8, checked by the gate)

State at RED and the wrong implementation each row catches. "Mutant" = a scratch edit of the finished GREEN
code that the row must turn red; the build report pastes each mutant's failing row name. Precondition rows
(W4, S10) are tested violated by the root refusal and met by the normal run.

| id | RED state | mutant (plausible wrong implementation) the row catches |
| --- | --- | --- |
| [W1] | RED-1 fails, 512 bytes of `N` | rename first, then write into the final name |
| [W2] | RED-1 fails, no leftover | temp created with default mode and not chmodded; or a temp name without the pid |
| [W3] | RED-1 fails, both names new | keep DIRECT (open in place) for any existing file; or `fs::copy` over the final name |
| [W4] | RED-1 fails, `Ok` | on a failed temp create fall back to DIRECT |
| [W5] | RED-1 passes (guard) | leave the temp behind after success |
| [EX1] `an_existing_world_readable_target_is_tightened_not_inherited` | passes (guard) | temp created 0644 and step 6's chmod deleted |
| [EX2] `a_shrinking_overwrite_leaves_no_stale_tail` | passes (guard) | REPLACE sent to DIRECT, and DIRECT's `set_len(0)` deleted |
| [W6] | RED-1 passes (guard) | skip the preflight, so the rename silently replaces a read-only file |
| [W7] | RED-1 passes (guard) | rename over the link name, replacing the link by a file |
| [W8] | RED-1 passes (guard) | create the absent target at the link name |
| [W9] | RED-1 passes (guard) | classify a FIFO as REPLACE (run the row under `timeout 60`: it can hang) |
| [W10] | RED-1 passes (guard) | ATOMIC for every non-regular destination |
| [W11] | RED-1 passes (guard) | always suffix the name, so 250 bytes exceeds NAME_MAX |
| [W17] | RED-1 passes (guard) | no stdio-alias rule: ATOMIC on `/dev/stdout` replaces the redirected file's inode |
| [W19] | GREEN | prefix rule off: ATOMIC on `/dev/fd/1` replaces the redirected file's inode |
| [W20] | 0.1.3 RED | 0.1.2 DIRECT on an alias |
| [W21] | 0.1.3 RED | `/dev/fd/` prefix off |
| [W22] | 0.1.3 RED | O_APPEND reopen instead of the stream; or no stdout flush |
| [W23] | 0.1.3 RED | alias checked on the given path only |
| [W24] | 0.1.3 RED | no canonical-parent check |
| [W12] | GREEN | limit 201 instead of 200 bytes; or the fallback name changed |
| [W13] | GREEN | skip `remove_file` after one step; or swallow the step's error |
| [W14] | GREEN | propagate the directory-sync error |
| [W15] | GREEN | open the temp with truncate instead of `create_new`, overwriting a stale leftover |
| [W16] | GREEN | loop 101 times, or return `NotFound` after the last name |
| [W18] | GREEN | no fallback on 45 (the plain `fsync` is never called); or fall back on any error; or fall back on every platform, not Apple only; or ignore the 45 instead of calling the plain `fsync` |
| [S1] | RED-2 compile error | always `None` |
| [S2] | RED-2 compile error | compare canonical paths only (misses a hard link) |
| [S3] | RED-2 compile error | `symlink_metadata` instead of `metadata` |
| [S4] | RED-2 compile error | compare length or bytes |
| [S5] | RED-2 compile error | compare the path text without canonicalising the directory (`dir/./x` vs `dir/x`) |
| [S6] | RED-2 compile error | report the last match, or always `Some(0)` |
| [S7] | RED-2 compile error | treat `Stdout` as `Unknown` |
| [S8] | RED-2 compile error | let `NonRegular` match `NonRegular` (two pipes would match) |
| [S9] | RED-2 compile error | compare by path for non-regular files (`/dev/null` matches itself) |
| [S10] | RED-2 compile error | let `Unknown` match, or panic on a lookup error |
| [S11] | GREEN | `Endpoint::Stdin` as `Identity::Unknown`; or Stdin reading fd 1 instead of fd 0 |

A mutant that no row turns red is a STOP: paste it and report.

Counts the gate checks: 24 W rows + 11 S rows + 2 existing rows + `child` = 38 tests in `write::tests::` at
GREEN (0.1.3); 15 at RED-1 (W1-W11, W17, the 2 existing, `child`), 4 failing.

## Existing tests and files that change (rule per file)

- `crates/mnemonic-io-lib/src/write.rs` `mod tests`: attribute and `#[cfg(unix)]` edits, in RED-1 (the same
  commit that adds the first new row; the two existing rows keep their bodies).
- `crates/mnemonic-io-lib/src/{lib.rs,channel.rs,write.rs}` docs: in GREEN.
- me-cli tests that reach `write_private` (`crates/me-cli/tests/bughunt.rs` `bughunt_me_01_...`,
  `cli.rs` `manifest_overwrite_shrink_no_trailing_bytes`, `world_readable_output.rs`): expected to pass
  UNCHANGED (spec section 9, me: call sites compile unchanged). They run in the whole-workspace nextest of
  gate `green`. If one fails, STOP and report: the spec orders no edit of them.
- `crates/mnemonic-io-lib/tests/fish_history_purge.rs`: unrelated, unchanged.
- No golden is re-blessed. Consumer-side changes (tk `tests/p3_declined_crate_items.rs` and
  `tests/p3_io_lib_pin.rs`, ms `ci/repro/vendor-git-sources.txt`, ms `man-release.yml`, the manual lines of
  spec section 9) belong to the consumer steps below.

## Test seams

Spec section 8 decides: a `#[cfg(test)]` thread-local `Cell<Option<Step>>`, read at the start of each step;
only W13 and W14 set it; no release code carries it. Children get env vars `MNEMONIC_IO_LIB_CHILD` and
`MNEMONIC_IO_LIB_TARGET`. W9 shells out to `mkfifo`. Nothing touches a network, a node or a real device
(no row writes `/dev/null`, spec section 8).

## Phase 2: the release of mnemonic-io-lib 0.1.2 (coordinator; Brian 2026-11-22 "Go for any release")

The spec (section 10 step 2) puts the crates.io publish of mnemonic-io-lib in this release, so ms and mt can
move from git to the registry. Who runs what: the coordinator runs all four steps; the builder runs none.
`cut-release.sh` covers the tag only (its spec section 1 item 9: crates.io publish is out of scope), so the
publish is a separate command.

- [R1] On the merge commit of the T6 PR (local `master` equal to `origin/master`), from inside the me
  checkout, with `$HOME/.cargo/bin` first: `bash scripts/cut-release.sh --dry-run mnemonic-io-lib-v0.1.2`
  (path: this repo's script; spec `design/SPEC_cut_release.md` section 3 has the `mnemonic-io-lib-v` row).
  Expected: `== mnemonic-io-lib is already 0.1.2 (crates/mnemonic-io-lib/Cargo.toml); no version edit` and
  `DRY RUN: nothing to commit; would stage HEAD <12> as it is`. A refusal here is a STOP for the coordinator.
- [R2] `bash scripts/cut-release.sh --push mnemonic-io-lib-v0.1.2`: pushes the annotated tag through staging.
- [R3] `cargo publish --locked -p mnemonic-io-lib --dry-run`, then `cargo publish --locked -p
  mnemonic-io-lib` (irreversible; the registry token stays in the operator's environment and is written to no
  file). Tag first, publish second, so the published crate matches a tag.
- [R4] From a directory outside any Cargo workspace (for example `cd /scratch/cache`):
  `cargo info mnemonic-io-lib@0.1.2` prints `version: 0.1.2` with no `(from ./...)` suffix, and
  `cargo info mnemonic-io-lib@0.1.1` refuses `could not find`; and
  `curl -sf -A 'crate-version-check' https://crates.io/api/v1/crates/mnemonic-io-lib | jq -r '.versions[].num'`
  prints `0.1.2` and `0.1.0` and no `0.1.1` (the spec: "0.1.1 is skipped on crates.io; 0.1.2 carries it").

## Phase 3: one pin-bump step per consumer (each is the FIRST commit of that repo's bughunt FIX PR)

The consumer's FIX plan owns everything else in its PR; this plan owns only the pin commit. Order among C1-C4
is free (spec section 10 step 4); the master table order is md, mk, ms, mt, tk. C1-C4 start after R4. C5 starts
only after C1, C2, C3 and C4 have all merged: the toolkit-manual lines it edits describe behaviour each of those
consumers ships, so C5 lands last. The
toolkit-manual lines of spec section 9 (`42-md.md:30` and `:219`, `43-ms.md:44-45`, `44-mk-cli.md:42`) exist only in
mnemonic-toolkit: the C5 tk PR edits all of them, for md, mk, ms, mt and tk. C1-C3 edit no toolkit-manual line (they
have none), and say so in their descriptions. The source comments of spec section 9 stay with their repo: md-cli
`cmd/mod.rs:105-108` in C1, mt-cli `main.rs:844-847` in C4.

- [C1] md (`descriptor-mnemonic`): `mnemonic-io-lib = "0.1.2"`; `cargo update -p mnemonic-io-lib --precise
  0.1.2`; refresh `vendor/` (`cargo vendor --locked vendor/`); pass `ci/repro/vendor-freshness.sh`. Unblocks #9 FIX_bh1119_md_out_writes (md-C4).
- [C2] mk (`mnemonic-key`): same recipe (including `cargo vendor --locked vendor/`). Unblocks #10 F-B output writes (mk-I1; mk-I6 through `same_file`).
- [C3] ms (`mnemonic-secret`): replace the git dependency by `"0.1.2"`; `cargo update -p mnemonic-io-lib` with
  no `--precise`; the lock shows `version = "0.1.2"` with the crates.io source; refresh `vendor/` (`cargo vendor --locked vendor/`); delete the
  `mnemonic-io-lib|https://github.com/bg002h/mnemonic-engrave|crates/mnemonic-io-lib` line of
  `ci/repro/vendor-git-sources.txt`; pass `ci/repro/vendor-freshness.sh`; then `git add -A vendor/ ci/repro/vendor-git-sources.txt` (the vendored crate gains a new untracked `Cargo.lock`; commit it, a clean checkout fails freshness without it) and run `bash ci/repro/vendor-freshness.sh` once more from a fresh clone of the branch; edit `.github/workflows/man-release.yml`
  as spec section 10 step 4 lists (delete the `pins` job and `needs: pins`; delete `git_source_url` and
  `git_source_rev`; in both musl steps delete the `IO_LIB_REV` and `IO_LIB_SRC` lines and the three
  `--config "source...` lines, and the `-e IO_LIB_REV="$IO_LIB_REV" \` line in the x86_64 `docker run` step).
  `grep -n 'mnemonic-engrave\|IO_LIB\|pins' .github/workflows/*.yml` must print only comment lines. This PR
  edits `.github/`; say so in its description (RULES allows it inside a reviewed PR). Unblocks #11
  FIX_bughunt1119_ms_out_writes (ms-I13; ms-I1 through `same_file`).
- [C4] mt (`mnemonic-transaction`): replace the git dependency by `"0.1.2"`; `cargo update -p
  mnemonic-io-lib`; the lock shows `0.1.2` from crates.io; mt is not vendored. The spec lists no
  `.github/` edit for mt; if mt's workflows reference the git source, that is a STOP for the mt builder.
  Unblocks #12 FIX-B output writes (mt-C3; mt-I4 through `same_file`).
- [C5] tk (`mnemonic-toolkit`): `"0.1.2"`; `cargo update -p mnemonic-io-lib --precise 0.1.2`; refresh `vendor/` (`cargo vendor --locked vendor/`);
  pass `ci/repro/vendor-freshness.sh`. Start only after C1-C4 have merged (see above). Unblocks #14 FIX-tk-secret-file-writes (tk-I3, tk-C1). The two tk tests
  `crates/mnemonic-toolkit/tests/p3_declined_crate_items.rs` and `crates/mnemonic-toolkit/tests/p3_io_lib_pin.rs` change with #14 (spec section 9), and the
  tk pin PR must keep them green or change them in the same PR: that choice is the tk builder's STOP if the
  tests refuse `0.1.2`.
- #13 FIX-me-output-integrity (me-I10 through `same_file`): the me path dependency needs no bump and no
  release; it can land with T6 or after (spec section 10 step 3). Not this plan's.

## Spec sentences with no row (the spec orders no test; verified by the review reading the code)

Spec section 3 steps 2 and 3 (the 40-hop limit, the race re-check), step 5's "Operation not permitted" for a
file owned by another user, step 10's skip of directory fsync on Windows, and the section 5 Windows
identities have no TRACE row; neither do step 1's
stdio-alias list other than `/dev/stdout` and its `/dev/fd/` and `/proc/` prefixes, step 1's other-`Err` DIRECT,
step 2's stdio-alias hop, and section 2's no-library-error-text rule. The coverage map marks them `CODE`. They are not given plan legs (no TRACE row
quotes them). Proposed FOLLOWUPS (Minor) are in the report.

## Coverage map

`design/io-lib-safe-write-coverage.tsv`: columns `id`, `section`, `spec quote`, `owner`. Owner is a row or
step id of this plan, `CODE` (above), or `OUT:<who>` (not this plan's). The gate checks every quote occurs
verbatim in the spec and every owner id occurs in this plan as `[id]`.
