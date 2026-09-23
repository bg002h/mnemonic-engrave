# F-449 stage 1b — Task 9: the mutation pass (§8.10)

**Worktree:** `/scratch/code/shibboleth/dm-worktrees/f449-stage1b`, branch `f449-stage1b`.
**Tree re-run against:** tip `e45ad51b640eabb26d06a58865a7806cc15799b0` (unmoved across the whole pass — `git log --oneline -1` before and after every mutation confirms this).
**Environment:**

```
export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH
export CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/f449-1b-target
cargo nextest run --locked --all-features --no-fail-fast
```

Baseline (unmutated, confirmed before the pass began): **1495 tests run, 1495 passed, 3 skipped**, 22.5s.

## Method — Ruling 1 (re-run against THIS tree)

All seventeen mutations were applied and run fresh against the tip above, not re-cited
from any earlier task report. Five had been proven once before, each against an
earlier tree (M16 at Task 3's tip, M17 at Task 6's, row 2's override half at Task 7's,
the leak mutation at Task 7's fold, mutation 6 at Task 8's) — all five folds since then
made this re-run non-optional, and all five reproduced identically here.

## Method — Ruling 3 (prove the mutation APPLIED before trusting any result)

For every mutation: a `panic!()` was inserted at (or immediately guarding) the mutated
branch first, the **full** suite (`cargo nextest run --locked --all-features
--no-fail-fast`) was run once and captured to a file, and the panic's occurrence
count was grepped to confirm the branch actually executes under the existing suite.
Only then was the panic replaced with the real mutation and the full suite re-run for
the authoritative RED/GREEN result. Every occurrence count is reported below,
per-mutation.

Two mutations (6 and 16) initially produced a **masking artifact** worth recording
explicitly rather than silently working around: when both of a mutation's two call
sites were panic-guarded simultaneously, the earlier-executing site's panic aborted
the one test that would have reached the second site before it ever got there,
making the second site's probe read `0` — which would misread as "unreachable" if
taken at face value. Both were re-checked with **only the second site guarded** (the
first left at its real, unmutated behavior) to confirm genuine reachability
independent of the masking. Both sites are confirmed independently reachable; see
mutations 6 and 16 below for the isolated counts.

## Method — a housekeeping correction mid-pass

Mutations 7 and 16 each caused a large cascade of test failures, and two kinds of
test-generated artifacts were left behind as an automatic side effect of the FAILING
runs (not of the mutation itself): a `proptest`-saved regression seed file
(`*.proptest-regressions`, modified or newly created) and `insta` `.snap.new` files.
`git diff --stat` alone does not show untracked files, so these were caught only by
`git status --porcelain` after each revert and deleted before continuing:

- `crates/md-codec/tests/proptest_to_miniscript.proptest-regressions` (reverted via
  `git checkout --`, after mutation 7)
- `crates/md-cli/tests/snapshots/json_snapshots__decode@nums_taproot.snap.new`,
  `crates/md-cli/tests/snapshots/json_snapshots__inspect@nums_taproot.snap.new`
  (deleted, untracked, after mutation 7)
- `crates/md-codec/tests/proptest_roundtrip.proptest-regressions` (deleted,
  untracked, after mutation 7 AND again after mutation 16 — proptest regenerated it
  independently both times)

`git status --porcelain` (not `git diff --stat`) was used as the between-mutation
cleanliness check for the remainder of the pass. None of these files reached the
final state or the report commit.

## Result summary

**17 of 17 mutations CAUGHT. Zero survivors — nothing to classify under Ruling 2.**

| # | mutation | panic pre-check | observed result | caught by |
| --- | --- | --- | --- | --- |
| 1 | reverse the concat, `nums.rs:52-54` (`for pk in leaf_pubkeys` → `for pk in leaf_pubkeys.iter().rev()`) | CONFIRMED — 17 panics across the full suite | RED — 4 failed | `md-codec::liana_unspendable the_recipe_reproduces_every_golden_xpub` (golden byte mismatch) + 3 downstream consumers (`liana_to_miniscript` ×2, `md-cli::liana_input_side decompose_recognises_a_real_liana_descriptor_and_gives_it_no_slot`) |
| 2 | `sort()` the concat, `nums.rs:52-54` (leaves sorted before hashing) | CONFIRMED — 16 panics | RED — 5 failed | `md-codec::liana_unspendable sorting_or_deduplicating_produces_a_different_xpub` + `the_recipe_reproduces_every_golden_xpub` + 3 downstream consumers |
| 3 | `dedup()` the concat, `nums.rs:52-54` (leaves deduplicated before hashing) | CONFIRMED — 2 panics (`a_synthetic_duplicate_leaf_changes_the_xpub_when_removed`, `derive_address_kind_1_liana_unspendable_internal_key_derives`) | RED — 1 failed | `md-codec::liana_unspendable a_synthetic_duplicate_leaf_changes_the_xpub_when_removed` only (as designed — no vendored case has adjacent duplicates, only the synthetic pin can see this) |
| 4 | `wire_version()` returns 4 always, `encode.rs:63-77` (`if needs_v8(...) { 8 } else { 4 }` → unconditional `4`) | CONFIRMED — 21 panics | RED — 16 failed | `md-codec::wire_version_8 a_kind_1_tree_round_trips_at_v8`, `kind_0_and_kind_1_over_the_same_tree_encode_to_different_bytes`, `wire_version_is_derived_from_the_tree_not_assumed` + `liana_unspendable`'s Task-8 loops + `mint_policy_does_not_reach_decode` + CLI-level tests |
| 5 | render kind 1 as the NUMS hex, `render.rs:202-210` (`InternalKey::NumsPoint => NUMS hex; InternalKey::LianaUnspendable => marker` → collapsed to `NumsPoint \| LianaUnspendable => NUMS hex`) — **same physical edit as mutation 12; see note below** | CONFIRMED — 5 panics | RED — 4 failed | `md-codec::liana_render kind_0_and_kind_1_templates_differ`, `kind_1_emits_the_marker_in_abstract_mode`, `kind_1_emits_the_marker_in_literal_mode` + `md-cli::liana_render_reparse_fixpoint item_7_the_render_reparse_fixpoint_covers_tr_kind_1` |
| 6 | constant `4` at `identity.rs:90` and `:200` (both `d.wire_version()` arguments to `write_node` replaced with `4u8`, together) | CONFIRMED — combined probe: site `:200` fired (1 panic); site `:90` read `0` under the combined probe, re-isolated separately (write-side untouched) and confirmed independently reachable (1 panic) — see masking note above | RED — 1 failed | `md-codec::liana_unspendable kind_0_and_kind_1_get_different_ids_and_a_different_phrase`, via `write_node`'s own `debug_assert!` at `tree.rs:216` ("LianaUnspendable has no wire representation at version 4…"), fired from the `:200` call site (`compute_wallet_policy_id` executes first in that test, before `compute_wallet_descriptor_template_id` at `:90` is ever reached) |
| 7 | write the kind bit at version 4, `tree.rs:192-220` (the `if wire_version == WF_UNSPENDABLE_VERSION {...} else {debug_assert}` guard removed — kind bit written unconditionally) | CONFIRMED — 1064 panics (fires for essentially every version-4 write of a `NumsPoint` key, i.e. nearly the whole corpus) | RED — 37 failed | `md-codec::internal_key_refactor every_vendored_wire_vector_re_encodes_to_the_same_bytes` (stage 1a's byte-equality gate) + 36 others (an extra bit shifts every downstream byte) |
| 8 | drop §6 row 1 (`UnspendableWithSortedMultiA`), `validate.rs:596-598` (`if contains_sortedmulti_a(t) {...}` → `if false && contains_sortedmulti_a(t) {...}`) | CONFIRMED — 6 panics | RED — 3 failed | `md-codec::liana_unspendable kind_1_with_a_sortedmulti_a_leaf_is_refused_at_mint`, `encode::unspendable_shape_tests::a_refused_shape_still_decodes_so_existing_cards_never_stop_reading`, `mint_policy_does_not_reach_decode minting_a_kind_1_sortedmulti_a_card_is_still_refused` |
| 9 | drop §6 row 2 (`UnspendableUseSiteNotCanonical`), `validate.rs:594-603` (both the shared-field check and the per-key-override check `&& false`-guarded together) | CONFIRMED — shared-field probe 1 panic, override probe 1 panic (both halves independently reachable) | RED — 2 failed | `md-codec::liana_unspendable kind_1_off_the_canonical_use_site_is_refused` (shared-field half) + `kind_1_with_a_noncanonical_per_key_override_is_refused` (override half) — one test per half, exactly as fix round 1 established |
| 10 | drop §6 row 4 (`UnspendableNotRootTr`), `validate.rs:617` (`if *internal_key == LianaUnspendable && !is_root {...}` → `if false && ...`) | CONFIRMED — 1 panic | RED — 1 failed | `md-codec::liana_unspendable kind_1_nested_under_wsh_is_refused` |
| 11 | drop §6 row 6 (`NonMinimalWireVersion`), `validate.rs:708-716` (the version/minimal comparison `&& false`-guarded) | CONFIRMED — 1 panic, in `encode::unspendable_shape_tests::version_8_with_no_kind_1_node_is_refused_at_encode` | RED — 1 failed | same test, `encode::unspendable_shape_tests::version_8_with_no_kind_1_node_is_refused_at_encode` |
| 12 | collapse G-1 site 1, `render.rs:202-210` — **identical physical mutation to row 5** (see note below) | see row 5 | see row 5 | see row 5 |
| 13 | collapse G-1 site 2, `to_miniscript.rs:424-429` (`node_to_descriptor`'s `match ik { NumsPoint => build_nums_internal_key()?, LianaUnspendable => {derive xpub}, ... }` → collapsed to `NumsPoint \| LianaUnspendable => build_nums_internal_key()?`) | CONFIRMED — 8 panics | RED — 5 failed | `md-codec::liana_to_miniscript a_tpub_wallet_renders_a_tpub_internal_key`, `the_keyed_multipath_descriptor_embeds_the_derived_literal_xpub_matching_liana_evidence`, `the_keyed_single_path_descriptor_collapses_the_internal_key_to_the_chosen_chain`, `a_multi_key_leaf_contributes_every_key_in_order` + `md-cli::liana_kind1_cmd_descriptor md_descriptor_network_flag_reaches_the_derived_internal_key` |
| 14 | collapse G-1 site 3, `policy_shape.rs:283-286` (`match internal_key { Slot(_) => Xpub, NumsPoint => Nums, LianaUnspendable => LianaUnspendable }` → `NumsPoint \| LianaUnspendable => Nums`) | CONFIRMED — 3 panics | RED — 1 failed | `md-codec::policy_shape liana_unspendable_taproot_is_its_own_key_path_kind` |
| 15 | collapse G-1 site 4, `format/json.rs:390-393` (`match internal_key { Slot(i) => (false,i,None), NumsPoint => (true,0,None), LianaUnspendable => (true,0,Some("liana_unspendable")) }` → `NumsPoint \| LianaUnspendable => (true,0,None)`) | CONFIRMED — 1 panic | RED — 1 failed | `md-cli::bin/md format::json::descriptor_json_tests::liana_unspendable_gets_its_own_json_state_distinct_from_nums` |
| 16 | invert the kind bit's polarity on BOTH sides, `tree.rs:199` (write: `== LianaUnspendable` → `!= LianaUnspendable`) and `tree.rs:377` (read: `r.read_bits(1)? != 0` → `== 0`), together | CONFIRMED — combined probe: write side fired (13 panics); read side read `0` under the combined probe (masked — write panic aborts before any byte is produced to read), re-isolated separately (write side untouched) and confirmed independently reachable (107 panics) — see masking note above | RED — 1 failed, **every round-trip test stayed GREEN** | `md-codec::wire_version_8 the_kind_bit_polarity_is_pinned_on_the_wire_not_just_round_tripped` ONLY — confirms by direct measurement that a symmetric inversion is invisible to round-trip testing by construction, exactly as the brief states, and that only the raw-bit golden test closes it |
| 17 | weaken the recogniser, `decompose/walk.rs:235` (`if actual_xpub == recomputed` → `if actual_xpub.public_key == recomputed.public_key && actual_xpub.depth == 0`) | CONFIRMED — 5 panics | RED — 1 failed | `md-cli::liana_input_side decompose_does_not_recognise_lianas_own_recipe_computed_over_the_wrong_leaves` ONLY — every positive-path test in the suite stayed green under this weakening (as the brief predicts: a pattern-match accepts every input the real check accepts), confirming this near-miss test is the ONLY thing standing between the weakened recogniser and a false `UNSPENDABLE(liana)` relabelling of a different wallet's key |

**Note on rows 5 and 12.** The brief's table lists "render kind 1 as the NUMS hex"
(row 5, gated by Task 5) and "collapse G-1 site 1 (`render.rs`)" (row 12, part of the
four G-1 sites) separately, but they name the exact same code location and the exact
same edit (`render.rs`'s `Tag::Tr` internal-key match, collapsing the
`LianaUnspendable` arm into `NumsPoint`'s). One execution answers both rows; running
it twice would reproduce byte-identical results. Reported once, against both row
numbers, rather than padded with a duplicate run.

## Every mutation's revert, confirmed

After each mutation (panic-probe and real alike), the file was reverted with `git
checkout --` and `git status --porcelain` was checked empty before the next mutation
began — including catching and deleting three test-tool-generated artifacts
(`*.proptest-regressions`, `*.snap.new`) that are not covered by `git diff --stat`,
per the housekeeping note above. `git log --oneline -1` was checked unmoved
(`e45ad51b640eabb26d06a58865a7806cc15799b0`) throughout.

## Final state

```
$ git status --porcelain
(empty)
$ git diff --stat
(empty)
$ ./scripts/phase-gate.sh
     Summary [  22.613s] 1495 tests run: 1495 passed, 3 skipped
=== cargo test --workspace --doc --all-features === ok (0 tests)
=== cargo clippy --locked --all-targets --all-features -- -D warnings === clean
=== cargo fmt --check === clean
=== cargo doc --workspace --no-deps --document-private-items --all-features === clean
=== design/display-grouping-vectors.tsv.sha256 === OK
phase-gate: all six steps passed
```

## Conclusion

Seventeen mutations, all re-derived and re-run against the current tip (not cited
from earlier tasks' trees), all confirmed to actually execute via a panic pre-check
before being trusted, all caught RED by a named test. Zero survivors — Ruling 2's
(a)/(b) classification does not apply to this run because there is nothing left
un-caught to classify. The two mutations most likely to hide a gap by construction —
#16 (symmetric bit-polarity inversion, invisible to every round-trip test) and #17
(a pattern-match weakening that accepts every input the real check accepts) — are
each caught by exactly one purpose-built test and nothing else, which is the
intended, minimal shape of a tight gate, not evidence of fragility.
