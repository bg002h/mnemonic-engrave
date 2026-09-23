# F-449 stage 4a — implementation report (single implementer)

Plan: `design/IMPLEMENTATION_PLAN_f449_stage4a_me.md` (GREEN at 434d94c3).
Worktree: `/scratch/code/shibboleth/me-worktrees/f449-stage4a`, branch `f449-stage4a`.
Toolchain 1.85.0; `CARGO_TARGET_DIR=/scratch/code/shibboleth/me-worktrees/f449-stage4a-target`
(the brief's path; the plan's `.tmp/f449-4a-target` is superseded by the brief).

## Task 0 — re-validation

- `scripts/plan-staleness-check.sh … . 8aea0d36`: unchanged 14, DRIFTED 0, not in repo 25.
  No commit touches `crates/`, `Cargo.toml` or `Cargo.lock` in `8aea0d36..434d94c3`.
- `descriptor-mnemonic-md-cli-v0.19.0^{commit}` = `cf35d61af0f058029df841f4882945e53f86cd4b` (matches).
- Baseline suite (`.tmp/f449-4a-base.log`): **664 run, 664 passed, 2 skipped** (matches plan).

## Task 1 — unpin + patch + repairs — commit `a16da983`

- Red first: both new tests failed on the unpinned tree exactly as the plan measured
  (`left: Number(0)` / `right: 2`; `left: [0]` / `right: []`) — this is **M1, killed**.
- Pin + patch applied; patch alone not verified in isolation (plan F3 already measured it).
  `cargo update -p miniscript`: lock diff is exactly the two stanzas (md-codec 0.42.0 → 0.47.0 git,
  miniscript 13.1.0 → 13.0.0 git). `cargo check --all-targets`: 0 "was not used" warnings;
  errors exactly 2 × E0559 at `md1.rs:354`, `:355` (F4).
- Descriptor-mnemonic's own patch rev at `cf35d61a` confirmed `ff4732e5…` (`git show cf35d61a:Cargo.toml`, line 44).
- Gate: nextest **666 run, 666 passed, 2 skipped**; fmt clean; clippy `-D warnings` clean.
- Mutations:
  - **M9** (`false && matches!(…)`): KILLED — panic at rust-miniscript `ff4732e/src/lib.rs:355`, "cannot be larger than 520 bytes, but got 547 bytes".
  - **M10** (`admit(…).is_err()`): KILLED — assertion "the twin is skipped on exactly the key-count-exceeded rows".
  - Reverted: test green.

## Task 2 — F-635, unchunked path refuses — commit `d25eee36`

- Red first: all five new tests failed before the change (four exit 0 with "backup needs N public plate(s)";
  the chunk test exit 4 with only "unsupported md1 wire version").
- `parse_line_md1_wrong_wire_version_rejected`'s fixture was re-read: `payload[0] = 0x08` → version 0, so
  the `(_, 0)` pattern is correct.
- Gate: nextest **671 run, 671 passed, 2 skipped**; fmt clean; clippy clean.
- Mutations (logs in `.tmp/f449-mut/M{2,3,4}.log`):
  - **M2** (swallow via `if let Ok(d) = d`): KILLED — 4 tests, each at `assert_eq!(r.code, 4, …)`
    (lines 86, 97, 113, 124).
  - **M3** (drop the `WireVersionMismatch` arm in `run_bundle`): KILLED — `bundle_refuses_…_unsupported_version`,
    at line 89 `contains("unsupported md1 wire version")`. **Plan-prose note (not a defect):** the plan says
    the `!contains("does not decode")` assertion is what catches M3; in fact the earlier
    `contains("unsupported md1 wire version")` assertion fires first. Either one would kill it.
  - **M4** (`Md1WireVersion(s.to_string(), 0)` in `parse_line`): KILLED — `bundle_names_the_version_…`,
    line 136 `contains(V12_NAMED)`.
  - Each reverted by restoring the file; test green.

## Task 3 — `me sysw` names the version — commit `57822135`

- Red first: `the_walk_…` failed to compile (E0432, `mdmk_unconfirmed_why`/`Unconfirmed` missing). With record.rs
  done and main.rs/expect.rs not yet changed, the three CLI tests failed (8 passed, 3 failed) and the walk test passed.
- Gate: nextest **675 run, 675 passed, 2 skipped**. The named guards all ran and PASSED:
  `the_descriptor_show_block_leaves_every_other_container_byte_identical`,
  `pack_warns_once_per_unconfirmed_record_and_still_succeeds`, `the_two_walks_agree_wherever_both_have_an_answer`,
  `sysw::vectors::tests::the_implementation_still_matches_the_recorded_vectors` and the other vectors tests. fmt and clippy clean.
- Mutations (logs in `.tmp/f449-mut/`):
  - **M5** (`WireVersionMismatch` → `Undecodable` in `md_verdict`): KILLED, all four. Walk at line 145 (`assert_eq!`),
    pack at 161, show at 191 (the `contains(format!(…))`), expect at 222 (`contains(V12_NAMED)`).
  - **M6** (`version_note().filter(|_| false)` in `report_unconfirmed`): KILLED, `pack_…` only (line 161).
  - **M7** (the same in `print_records`): KILLED, `show_…` only (line 191).
  - **M8** (`UnsupportedWireVersion(_) => broken.push(i)`): KILLED, `expect_…` only (line 222 `contains(V12_NAMED)`).
  - **Harness defect, found and fixed:** my first M6 and M7 runs were CONTAMINATED. The restore step
    (`shutil.move` of a backup) left the file's mtime OLDER than the mutated build, so cargo kept M5's compiled lib, and
    M6 and M7 showed M5's four failures. I fixed it with `os.utime` on restore, touched every restored file, and re-ran
    both. The results above are from the clean re-run. M2 to M5, M8, M9 and M10 were unaffected: each was the first write
    after a clean build, or it edited a lib file, which forced the lib to rebuild from the restored sources. The Task 3
    suite was re-run from a clean state (touched sources) before the commit.
- Step 6b: the doc comment moved to `mdmk_unconfirmed_why`. I re-derived its caller list by grep (`report_unconfirmed`,
  `print_records`, `expect::check`, and the projection for `sysw/vectors.rs`). The OLD doc named
  `print_mdmk_confirmation` for `show`, a function that does not exist. `expect.rs`'s module doc now names
  `mdmk_unconfirmed_why`. The projection has the one-line doc the plan specifies.
- **Pre-existing finding (not introduced here; out of scope, NOT fixed):** the doc's "one walk call per invocation" rule
  is false for `me sysw pack --expect …` whenever every expectation is met. `expect::check` calls the walk (at `434d94c3`,
  `expect.rs:202`), and then `report_unconfirmed` calls it again. Measured on this branch with sysw_cli.rs's
  `MK1_A MK1_B` (pinned csid mismatch): `me sysw pack --no-passphrase --expect cosigner MK1_A MK1_B --out …` prints
  the R2/R6 "was not derived from its content" warning **twice**, exit 0. `tests/sysw_cli.rs:113-139` pins `pack` and
  `show` without `--expect` only. The new doc comment states this double call rather than hiding it. Suggest a
  follow-up (ownerless; stderr noise only, no wrong result).
- Minor: `cargo doc` flags the expect.rs module doc's intra-doc links as unresolved. This includes the pre-existing
  `super::record::card_hrp` and `super::mt::mt_unconfirmed`, and the new links follow the same pattern. No CI gate runs rustdoc.

## Task 4 — records — commit `6be9016d`

- F-635 → `**Status:** CLOSED 2026-09-23 in mnemonic-engrave d25eee36 …`. `scripts/followups-status.sh`: OK.
- F-651 (FOLLOWUPS.md:19746), F-652 (:19767), F-653 (:19778) verified present, each `**Status:** OPEN` with an owning
  phase. `descriptor_seam.rs:1144` cites F-651.
- Spec edits: §8.9 rows DONE (`d25eee36`, `57822135`); §8b "latent today" replaced with "live at v4 too" (F7), plus a
  Status line; §9 stage-4a row Status clause (with `--expect` as the third reader); §9a item 1 (git rev, F3); §9a item 2
  (blast radius, F-651); the "me is downstream" paragraph put in the past tense, with a Status note.
- **Ruling:** pre-fix line citations in the touched sections (`bundle.rs:371`, `sysw/record.rs:251-252`,
  `Cargo.toml:26`) are now marked "at me 0.10.0" rather than re-pointed — they cite the DEFECT's location, and on this
  branch `Cargo.toml:26` resolves to the new comment block and `bundle.rs:371` to `let mut hashlock_kinds` — so left
  unqualified they would read as current and wrong. *Cost if wrong:* a reader wanting the current line must grep.
- `plan-cite-check.sh` on the spec: exit 1, 40 ok / 48 DANGLING, **identical DANGLING count to the pre-edit spec at
  434d94c3** (basename-relative false alarms, as the plan predicted); every line in touched sections was read.
- Steps 6 (whole-diff review) and 7 (merge) belong to the controller, and were not run.

## Task 5 — release 0.11.0 — commit `e8034282` (branch only; NOT tagged, NOT installed)

- **Step 0: FAILS, as expected.**
  1. Ancestry: **no stage-3 fork merge SHA is recorded.** `design/CONTINUITY_f449_stage2.md` has none, there is no
     successor continuity file, and engrave master (`e8451be2`) has only `plan: F-449 stage 3 (Go port) draft`.
  2. Behavioural: after `git fetch`, fork `origin/main` = `7b6f2fb`. In a throwaway detached worktree
     (`.tmp/f449-4a-fork-precond`, removed afterwards; `git worktree list` confirms it is gone):
     ```
     === RUN   TestStage4aPreconditionDecodesV8
         zz_stage4a_precondition_test.go:7: fork md/ cannot decode a version-8 card: md: wire version mismatch
     --- FAIL: TestStage4aPreconditionDecodesV8 (0.00s)
     FAIL	seedhammer.com/md	0.002s
     ```
  → Release **blocked on stage 3**. No tag, no push, no `cargo install`, no me-preview copy (Steps 6 to 9 not run).
- Steps 1-3: version 0.11.0. CHANGELOG: `[0.10.0] - 2026-09-16` inserted first. The date is the tagger date of
  `v0.10.0` (tag object `1fa8dd99`). Its bullets come from `git show -s acbfcc93`, plus the NOT BREAKING note. F-493/F-504
  moved under it: both CHANGELOG commits (`1d10db17`, `ea3229f1`) are ancestors of `v0.10.0`. The `[0.11.0]` section is
  as Step 2 specifies. `1cbecbfd` is confirmed NOT an ancestor of `v0.10.0`, and its quoted message was grep-verified in
  `composer_records.rs:274`. `Cargo.lock` diff: the `mnemonic-engrave` version line only.
- **Ruling:** `[0.11.0]` is dated 2026-09-23 (the release commit's date) — the plan's `<release date>` — because no
  tag date exists yet. *Cost if wrong:* when stage 3 lands and the tag is cut later, the controller must re-date the
  heading (a one-line edit before tagging).
- Step 4 gate: workspace nextest **684 run, 684 passed, 2 skipped** (`mnemonic-engrave` alone is 675; the other 9
  come from the workspace's second crate). `cargo test --locked`: 684 passed, 0 failed, 2 ignored. fmt clean; clippy
  clean; `cargo metadata --locked` resolves; `me --version` → `me 0.11.0`.
- Preview of Step 7's three behaviour checks on the BRANCH debug binary (not a release):
  1. v8 template `bundle`: exit 0, `this md1 is a TEMPLATE` ✓, `declares 2 key slots` ✓.
  2. v12 `bundle`: exit 4, stderr `me: unsupported md1 wire version: wire-format version mismatch: got 12; accepted
     versions: 4, 8 -- this build of me cannot read this plate, so it states no plate count for it`; no `backup needs` ✓.
  3. v12 `sysw pack`: exit 0, names `got 12`, no `could not decode` ✓.

## Summary for the controller

- Branch `f449-stage4a` HEAD `e8034282`: 5 commits on `434d94c3` (a16da983, d25eee36, 57822135, 6be9016d, e8034282).
- All 10 plan mutations were KILLED (M1 on the baseline; M2 to M10 applied → red, reverted → green). No survivors.
- master has moved since the branch point (`90805e31` persist F-642, `e8451be2` stage-3 plan draft). Both touch only
  design/ files and neither was merged here.
- Concerns: (1) the release is blocked on stage 3 (Step 0 FAIL, above). (2) PRE-EXISTING: `me sysw pack --expect …`
  calls the walk twice and double-prints the R2/R6 csid warning; the one-walk-call rule is false for that path (see
  Task 3). This is a follow-up candidate, not filed. (3) Plan-prose nit: M3 is caught by the `contains("unsupported md1
  wire version")` assertion, not the `!contains("does not decode")` the plan names. (4) The `[0.11.0]` date may need
  moving at tag time.

## Fix round — whole-branch review M1 — commit `ed4b7792`

- (a) New `BundleError::Md1MissingOrigin(String, u8)`, mapped from `MissingExplicitOrigin` on the unchunked path AND on
  the chunked path. The chunked path had said "md1 set … is incomplete/inconsistent" for a whole set; I measured that
  with `md1ffxweqqpqggqps8zjs4qqyhaq7eqm6qq6k`. The message now says the plate carries no key origin, so `me` cannot
  count what a restore needs, and that `md decode` still reads it. It names the remedy: `md encode --path <PATH>`, or
  inline origins, or engrave the complete set. **I ran the remedy:** `md encode --path bip48` and inline origins each give
  exit 0 plus a TEMPLATE note, for the tr fixture and for the wsh/older(144) example.
  - **Ruling:** the remedy names `--path` and inline origins, not `--fingerprint`. md-cli's help says `--fingerprint`
    supplies a master fingerprint, not an origin PATH, and `--path` alone was measured to fix the refusal. *Cost if
    wrong:* none; the remedy text is correct as measured.
  - **Ruling:** the chunked path is included. It is the same class, and its "incomplete" wording was equally false.
    *Cost if wrong:* one extra arm the controller could ask to be split out.
- Tests: the v4 origin-less test now asserts the new wording and the remedy, and forbids "does not decode". Its old
  `"requires explicit origin"` assertion is removed. New test: `a_chunked_origin_less_template_is_not_called_incomplete`.
- (b) The CHANGELOG `[0.11.0]` now carries a separate "Newly refused" bullet. It covers every template shape with no
  canonical derivation path encoded without origins (wsh miniscript, hashlocks, timelocks, tr script trees; 12 of 20 in
  the review), and gives the remedy. `Md1MissingOrigin` is added to the library breaks. M3's date is untouched.
- Gate: nextest `-p mnemonic-engrave` **676 run, 676 passed, 2 skipped**; fmt and clippy clean.
- Mutations: **M11** (drop the unchunked arm) KILLED `a_version_4_origin_less_…`, whose stderr fell back to "does not
  decode". **M12** (drop the chunked arm) KILLED `a_chunked_origin_less_…` at the `!contains("incomplete")` assertion.
  Both reverted → green.
