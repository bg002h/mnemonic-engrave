# Coordinator-compat plan 1b — implementation report

Implementer: single agent (opus). Plan:
`design/IMPLEMENTATION_PLAN_coordinator_compat_1b_verdicts.md` (GREEN, R0/R1).
Toolchain 1.85.0 on PATH; `CARGO_TARGET_DIR=/scratch/code/shibboleth/dm-worktrees/cc-1b-target`.
Gate = the plan's four commands, each captured once to
`/scratch/code/shibboleth/dm-worktrees/cc-1b-gate/t<N>/`.
Mutations run with a helper that asserts the mutation site occurs exactly once,
runs nextest `--no-fail-fast`, restores, and touches the file.

## Task 0 — mnemonic-engrave (worktree `me-worktrees/cc-1b`, branch `cc-1b-records`)

- `7c8def7c` evidence: Nunchuk kind-1 probe (py/tsv/out) + R0 Liana v15.0 probes
  (6 in / 6 out rows). Byte-identical (cmp) to the plan's mirror
  `.tmp/cc1b-engrave-mirror` inputs.
- `7ca9d2f8` records: F-661..F-670 filed (next free IDs re-grepped: last was
  F-660); F-644 md-cli half re-owned to plan 1b Task 8; F-655 to Task 2.
  `scripts/followups-status.sh`: OK. Citations spot-checked at their SHAs
  (md-cli `walk.rs:259-282`, ms-cli `hashlock.rs:525`, fork test at `2c9eed3`
  — its `func` is at :136, doc comment from :131).
- Ruling: the plan's "commit the working-tree edit to
  `scripts/policy-differential.py`" — nothing to commit — the `--md-only` retry
  is already in `aa7f0ffa` (the R0 fold commit). Cost if wrong: none; grep shows
  the retry at `scripts/policy-differential.py:381-384`.

## descriptor-mnemonic (worktree `dm-worktrees/cc-1b`, branch `cc-1b-verdicts`)

| task | commit | nextest | doc/clippy/fmt | mutations |
| --- | --- | --- | --- | --- |
| 1 | `534de037` | 1537 / 4 skipped | 0/0/0 | guard absent (test-first red: `(2, 3)`) — killed |
| 2 | `a3b04f5c` | 1538 / 4 | 0/0/0 | recogniser→false (reds near-miss + `chunks_and_descriptor_yield_the_same_key`); compare only `public_key`; `want == xkey`→`true` — all killed |
| 3 | `ae10d478` | 1539 / 4 | 0/0/0 | `pk <= p`→`pk < p` — killed |
| 4 | `c497da9a` | 1540 / 4 | 0/0/0 | Core span 29.4-31.1→29.4-30.3 — killed |
| 5 | `3af03f15` | 1540 / 4 | 0/0/0 | `--check` non-vacuity: one-byte input edit in a scratch clone → `STALE`, rc 1 |

Every task was test-first: the new test was applied alone and seen to fail
(assertion or E0432/E0609 compile error) before the implementation.

| 6 | `b1dce4d3` | 1549 / 4 | 0/0/0 | see below — 11 killed |
| 7 | `0826b426` | 1552 / 4 | 0/0/0 | U+001F printed as spaces — killed (reds `shape_key_agrees_with_the_library_on_every_evidence_descriptor`) |
| 8 | `5cd3e5bf` | 1556 / 4 | 0/0/0 | see below — 4 killed |
| 9 | `9196b13c` | 1556 / 4 | 0/0/0 | — (release commit) |

The nextest counts match the plan's measured sequence exactly
(1537 → 1538 → 1539 → 1540 → 1540 → 1549 → 1552 → 1556 → 1556).

Every task was test-first: the new tests were applied alone and seen to fail
(an assertion, E0432/E0609 compile errors, or 3/3 and 10/26 failing CLI tests)
before the implementation.

Task 5: 532 rows from 23 inputs; `evidence.meta.json` records
`engrave_commit` `7ca9d2f8` (the engrave worktree HEAD). `--check`: fresh.

Task 6: `cargo xtask verdicts` → `532 rows: 230 cells stored, 206 refusals
the rules explain, 2 unkeyable`, zero disagreements; `--check` fresh.
`table.rs` byte-identical to the plan's staged T6 tree; the whole worktree
matched staged T6 except `evidence.meta.json` (`engrave_commit` only).
Mutations (all killed; the test that reds them in parentheses):
- recogniser → `false` (`every_evidence_row_keys_identically_through_chunks_and_text`, plus 2 others)
- R-1 clause `Some(false)` → `Some(true)` (`r1_leaf_order_splits_one_key_into_two_nunchuk_verdicts`)
- drop `CORE_MULTIPATH` from 26.0-28.4 (`core_verdicts_follow_the_measured_boundary`)
- delete `at_version`'s `keys_present` return (`a_template_is_never_claimed_to_import`, + R-1 test)
- `none_imports` `all` → `any` (`none_imports_only_when_every_run_refuses`)
- drop the D5 comparison (`the_build_names_every_disagreement_class`)
- class 9 `k >= 2` → `k >= 1`, and → `k >= 3` (`liana_class_9_refuses_a_second_threshold_only_from_k_2`, both)
- drop the `OlderBlocks > 0xFFFF` disjunct (xtask `table_is_fresh`; `cargo xtask verdicts --check` names
  `D2 missed-refusal coord-compat-1b/liana-r0-probes-out.jsonl:5 ... Timelock value '70000'`)
- delete one `Cell` line from `table.rs` (xtask `table_is_fresh`)
- `liana_reads_as_built` → `true`: `table_is_fresh` reds; after regenerating,
  `imports_altered_is_derived_from_the_coordinators_own_reading` (and the
  class-9 test) red. Table and registry restored; `--check` fresh after.
- (the Task 4 span mutation, re-run in this scope: killed)

Harness note: an early M10 run combined `-p xtask` with `--test coordinator`,
which filters xtask's unit test out, so it read SURVIVED; re-run on `-p xtask`
alone it is killed. Not a finding about the code.

Task 8 mutations (all killed): remove `verdict::notice` from compose (5 red,
incl. `compose_names_liana_refusal_for_every_f644_shape`); drop `&& !md_only`
(5 red, incl. `compose_refuses_the_none_case_and_md_only_proceeds`); route
`md descriptor` through a none-case refusal (2 red, incl.
`descriptor_names_the_form_and_never_refuses`) — **plan listed as unmeasured,
now measured**; restore the retired hashlock warning (reds
`liana_policy_verdict_comes_from_the_registry_by_shape_not_name`'s
never-prints assertion) — **also now measured**. The `KeysAbsent` mutation is
the md-codec twin measured in Task 6. `omitting_the_flag_is_byte_identical_to_the_previous_release`
passes.

Task 9: md-codec 0.48.0, md-cli 0.20.0, exact pin `=0.48.0`, `Cargo.lock`
moves exactly the two versions, two CHANGELOG entries.
`vendor-coord-evidence.sh --check <engrave worktree>` → fresh;
`cargo xtask verdicts --check` → fresh. The final tree equals the plan's
staged T9 tree except `evidence.meta.json` (`engrave_commit`) and the two
CHANGELOG dates.

## Rulings

- Ruling: CHANGELOG dates set to 2026-09-23 in the release commit, not left
  `UNRELEASED` — the plan says "set when the release is cut", and the previous
  release (`972da724`, 0.47.0/0.19.0) dated its entries in the release commit
  itself, so an `UNRELEASED` marker in a release commit would be a defect in
  the PR. Cost if wrong: re-date both lines at merge if it slips past today.
- Ruling: `--check` run against the engrave WORKTREE (`cc-1b-records`), per
  the dispatch, not engrave `master`. Against `master` today it fails loudly
  (`not found: .../coord-compat-1b/liana-r0-probes-in.jsonl`) until
  `cc-1b-records` merges. `--check` compares input hashes and rows, not
  `engrave_commit`, so after a merge it reads fresh against master.
  Cost if wrong: none to the build; `engrave_commit` in the meta names a
  branch commit, which stays reachable only if the engrave branch is merged
  (not squashed/rebased).

## Concerns for the controller

1. Merge engrave `cc-1b-records` (`7c8def7c`, `7ca9d2f8`) into engrave master
   with a merge or fast-forward — not a squash/rebase — so the vendored
   `engrave_commit` `7ca9d2f8` stays reachable, and before anyone runs
   `--check` against the main engrave checkout.
2. F-669: do not `cargo install` md 0.20.0 until the fork test gains `--md-only`.
3. Nothing pushed; no PR opened; no tag.
