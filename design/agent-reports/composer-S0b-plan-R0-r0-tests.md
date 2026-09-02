# R0 TESTS lens — `design/IMPLEMENTATION_PLAN_composer_S0b_presets.md` (round 0)

**Reviewer:** independent TESTS-lens pass. Did not author the plan.
**Question:** can every test in the plan FAIL when the code it guards is wrong, and is every `Expected:` line observable and true? Answered by mutation in a private copy, never by reading.
**Scope:** the plan's own tests and Expected-lines only — no design/spec audit.

## Setup

Copied the controller's wired scratch `/tmp/plan-build-gate-md` (read-only) to
`/scratch/code/shibboleth/.s0b-tests-lens` via `cp -r`; built with
`CARGO_TARGET_DIR=/scratch/code/shibboleth/.s0b-tests-lens-target` (toolchain
1.85.0 from the copy's `rust-toolchain.toml`, matches the pin). Baseline
confirmed green before any mutation:
`cargo nextest run --locked -p md-cli -E 'binary(/^cli_compose/)'` → 23/23;
`cargo nextest run --locked -p md-codec -E 'binary(/^compose_/)'` → 52/52.
Every mutation below was applied to a single file, run, then reverted and
diff-verified byte-identical to the pre-mutation copy before the next
mutation — no two mutations were ever live at once.

## Mutation table

All 12 mutations were caught. None produced a false PASS.

| # | Mutation | Test(s) that failed | Exact match to plan's claim? |
|---|---|---|---|
| 1a | `tiered-recovery`: swap `ofs[0]`/`ofs[1]` (tier 2 consumed first) | `cli_compose_preset::preset_tiered_recovery_and_decaying_multisig_and_hashlock_gated_compose` | yes — stdout diff shows `multi(1,...)`/`multi(2,...)` swapped |
| 1b | Drop the legacy-wrapper shape check in `compose::validate` (simulates the refusal breaking for a non-plain preset under `sh`) | `cli_compose_preset::preset_kofn_recovery_refuses_under_legacy_wrappers_with_the_spec_4d_shape` **and** md-codec's own `compose_lowering::compose_refuses_legacy_wrappers_outside_the_single_sorted_multi_shape` (pre-existing) | yes — the plan's claimed test fires, plus a pre-existing codec test as a second line of defense |
| 1c | `named_only` always returns `Ok(())` (accepts any unknown `<param>=`) | `cli_compose_preset::preset_refuses_an_extra_parameter` | yes |
| 1d | `need_u32` defaults to `Ok(1)` on a missing key instead of erroring | `cli_compose_preset::preset_refuses_a_missing_parameter` | yes |
| 1e | `presets::blocks()` builds `Lock::OlderUnits` instead of `Lock::OlderBlocks` (this helper is shared S0 code every preset's `older=` value passes through) | `cli_compose_preset::{preset_kofn_recovery_matches_the_equivalent_path_list_under_tr, preset_tiered_recovery_and_decaying_multisig_and_hashlock_gated_compose}` **and** pre-existing md-codec `compose_lowering::{presets_compose_and_carry_the_documented_shapes, presets_lower_to_their_pinned_templates}` | yes — four tests, two of them this plan's own |
| 1f | `parse_sha256_hex`: length check `!= 64` → `!= 63` (rejects valid 64-hex) | `cli_compose_preset::preset_tiered_recovery_and_decaying_multisig_and_hashlock_gated_compose` **and** pre-existing `cli_compose::compose_refuses_a_keyless_path_without_experimental_and_admits_it_with` (shared helper) | yes |
| 2a | One-character edit to a MANIFEST template (`sortedmulti(2,...` → `sortedmulti(3,...` on `keyed_compose_preset_plain_multisig`) | md-codec `compose_vectors::every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` (names the vector in the assertion) **and** md-cli `vector_corpus::vectors_output_matches_committed_corpus` (names the file, shows the exact 1-line diff) | yes, names the vector both times |
| 2b | `family()` row for `keyed_compose_preset_plain_multisig`: `presets::plain_multisig(Wrapper::Wsh, 2, 3)` → `(..., 2, 2)`, expected text left at the 3-key form | md-codec `compose_vectors::{every_family_entry_renders_as_listed, every_compose_vector_in_the_manifest_is_exactly_what_compose_renders}` (both name the vector) | yes. Notably NOT caught by `every_preset_passes_the_5b_cross_check` or `every_family_entry_passes_the_5b_cross_check` — correct, those check a row's internal self-consistency (PathList vs its own literal), not against MANIFEST; the two-layer design works as the plan describes |
| 3 | `--json`'s `"preset": preset_json` field dropped from the object | `cli_compose_preset::preset_json_names_the_preset_and_its_resolved_parameters` | yes |
| 4 | `ArgGroup::new("path_source")` gets `.multiple(true)` (accepts `--path` and `--preset` together) | `cli_compose_preset::preset_and_path_are_mutually_exclusive` | yes — reproduced verbatim: exit 2, stderr contains "cannot be used with" |
| 5a | Add a 7th `preset:*` entry to `SINGULAR_TAGS` with no matching vector | md-codec `compose_vectors::every_tag_appears_in_at_least_two_vectors` — "a singular tag has exactly one vector: preset:nonexistent-archetype" | yes, names the tag |
| 5b | Remove `"preset:plain-multisig"` from that vector's tag list | same test — "a singular tag has exactly one vector: preset:plain-multisig" | yes, names the tag |
| 6 | `main()`'s catch-all `Err(e)` arm: exit code 1 → 2 | 9/23 `cli_compose`/`cli_compose_preset` tests fail immediately (`preset_refuses_an_unknown_name`, `preset_refuses_a_missing_parameter`, `preset_refuses_an_extra_parameter`, `preset_propagates_a_parameter_the_constructor_rejects`, `preset_kofn_recovery_refuses_under_legacy_wrappers_with_the_spec_4d_shape`, `preset_decaying_multisig_propagates_preset_shape_refusals`, plus 3 pre-existing `--path`/encode-gate tests) | yes, broad and immediate. Only tested the 1→2 direction (the natural single-line mutation); did not separately force an exit-0 refusal since the finding was already conclusive and that direction requires a structurally different edit (turning an `Err` into an `Ok`) |

Mutation 1b note on methodology: the brief asked to "drop the legacy-wrapper
refusal for **one** non-plain preset." The refusal lives entirely in shared
`compose::validate` (S0 code, not this plan's diff) with no per-preset branch
to target narrowly, so the mutation disabled the check globally and observed
which tests fired — a superset of "one preset," but it directly answers
whether the plan's own claim ("no CLI-side special case needed... the SAME
`ComposeError::LegacyWrapperShape`") is actually exercised by a real test.
It is.

## Expected-line audit (7 checked, brief asked for 6)

| # | Plan location | Claim | Result |
|---|---|---|---|
| 1 | Task 1 Step 3 | Exporter writes exactly 6 `keyed_compose_preset_*.conformance.json` | **TRUE** — ran `md vectors --out <tmp>`, counted 6 |
| 2 | Task 2 Step 5 | Full `-p md-cli` suite is 775/775 | **TRUE** — `775 tests run: 775 passed, 1 skipped` |
| 3 | Task 3 Step 1 | `cargo fmt --all --check` clean | **TRUE** |
| 4 | Task 3 Step 1 | `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean | **TRUE** — zero warnings |
| 5 | Task 3 Step 1 | Whole-workspace `cargo nextest run --workspace --all-features` = "1332 tests run: 1332 passed, 3 skipped" | **PARTIALLY TRUE**: total count and skip count match exactly (1332 tests, 3 skipped), but one failure is present: `md-codec::display_grouping_conformance::conformance_vectors_pass`. Per the controller's addendum this is the SAME pre-existing failure already seen in the controller's own wired scratch (518/519 md-codec) and is being triaged separately as a probable scratch artefact unrelated to this plan — confirmed: it fails identically in this independent copy too. `display_grouping_conformance.rs` is not one of this plan's Task-1/2/3 files, so this is not attributed to the plan. No further budget spent on it per the brief. |
| 6 | Task 1 Step 1 | Pre-Step-2 state: `every_family_entry_renders_as_listed`, `every_tag_appears_in_at_least_two_vectors`, `every_family_entry_passes_the_5b_cross_check` PASS; `every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` FAILS with `MANIFEST lacks keyed_compose_preset_plain_multisig` | **TRUE, exact wording** — reproduced by stripping the six MANIFEST entries and re-running; got the identical panic message and PASS/FAIL pattern (including `every_preset_passes_the_5b_cross_check`, not named in the plan's list, also passing) |
| 7 | Task 2 Step 2 | "Run to verify the tests fail **to compile**" | **FALSE AS WORDED** (N-1 below) — reproduced by swapping in the true pre-Task-2 baseline `compose.rs`/`main.rs` from the primary repo at `66bdf2f4` alongside the new `cli_compose_preset.rs`: the crate and test binary **compile successfully**; 12/14 tests instead fail **at runtime** with clap's own parse error (`error: unexpected argument '--preset' found`, exit 2), because `--preset` is simply not a recognised flag on the old `Compose` variant. The RED state the step wants is still achieved — just via a runtime CLI-parse rejection, not a `rustc` compile error. |

## Also — the fork-side questions

- **Exporter file kinds:** confirmed exactly `bytes.hex`, `conformance.json`, `descriptor.json`, `phrase.txt`, `template` — 5 kinds × 6 vectors = 30 files, no more, no fewer (`find ... -iname 'keyed_compose_preset_*' | wc -l` = 30; grouped by extension, 6 of each).
- **156-file fork math:** machine-counted the regenerated `crates/md-codec/tests/vectors/` in the wired copy directly (not from the plan's prose): 28 `keyed_compose_*` vectors × 5 files = 140, 4 unkeyed `compose_*` vectors × 4 files = 16, **total 156** — matches the plan's Task 3 claim exactly.
- **Fork-side ownership + a real gating test:** read the actual fork at `/scratch/code/shibboleth/seedhammer` (`321acb5`, matches the plan's citation). `md/compose_vectors_pin_test.go`'s `composeVectorNames` has exactly 26 entries (machine-counted) and `TestComposeVectorsMatchTheirProvenancePin` hardcodes `if len(p.Files) != 126 { t.Fatalf(...) }` plus `if p.Vectors != len(composeVectorNames) { t.Fatalf(...) }`. This test WOULD fail once the corpus grows to 32/156 and the vendoring script's pin file is regenerated without a matching update to `composeVectorNames`/`126→156` — exactly the two one-line edits the plan names. `scripts/vendor-compose-vectors.sh:16`'s glob `^(keyed_)?compose_` matches the new `keyed_compose_preset_*` names with no script change, confirmed by reading the script. The plan's Task 3 closing paragraph states this ownership explicitly ("Stage 3 (or a dedicated F-453-follow-on) territory, not this plan's") — so yes to both halves of the question: stated as owned, and a real test gates it.

## Findings

**C (Critical): none.** All 12 mutations were caught; zero false-PASS paths found in the sweep.

**I (Important): none.** All six §4d refusal-table rows (unknown name, missing param, extra param, constructor-rejected param, legacy-wrapper shape, `--path`/`--preset` both-or-neither) have direct test coverage, confirmed by mutation for five of the six and by direct test reading for the sixth (decaying-multisig's two `PresetShape` propagation cases, which exercise pre-existing S0 guard logic through this plan's new message-propagation path).

**M (Minor): none.**

**N-1 (Nit):** Task 2 Step 2's header, "Run to verify the tests fail to compile," is inaccurate. Reproduced: the workspace compiles cleanly at that point; the 12 tests that exercise `--preset` fail at **runtime** via clap's own arg-parse rejection (exit 2, "unexpected argument '--preset' found"), not a `rustc` compile error. The TDD discipline itself is intact (RED before GREEN, verified) — this is a wording/precision defect in the step's own description, not a defect in what the tests catch or a gap a build gate should have caught (the build gate doesn't run at this intermediate step). No fold action needed to unblock implementation; worth a one-word wording fix at fold time if convenient.

**N-2 (Nit):** Expected-line 5 (Task 3 Step 1's whole-workspace count) states "1332 tests run: 1332 passed, 3 skipped" as an unqualified PASS claim; in practice one pre-existing, plan-unrelated test (`display_grouping_conformance::conformance_vectors_pass`) fails, matching the controller's own already-flagged and separately-triaged observation. Recorded for completeness; not attributed to this plan's diff.

## Closing counts

**0 Critical / 0 Important / 0 Minor / 2 Nit.**

Mutation coverage: 12/12 mutations caught (100%), spanning `parse_preset`
grammar, the MANIFEST/family cross-check (both directions), the `--json`
`preset` field, the `--path`/`--preset` `ArgGroup`, the singular-tag
exemption (both directions), and exit-code discipline. Expected-line
coverage: 7/7 checked were either exactly true or precisely characterized
where imprecise (both departures are wording nits, not test-quality defects,
and one is a pre-existing failure outside this plan's file set). This
plan's test suite has no false-PASS path found by this lens.
