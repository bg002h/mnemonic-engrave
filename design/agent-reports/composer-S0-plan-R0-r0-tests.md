# Composer Stage 0 plan — R0 review, lens: mutation-tested tests + claims

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md` @ `3a799fafc9db2c1bb2c9036b67a2e0686b39a978`
**Target repo:** `/scratch/code/shibboleth/descriptor-mnemonic` @ `3b0944fb8e3c1d3d44056a689505383b3982d555`
**Lens:** for every test, what mutation of the implementation would make it fail, proved by actually mutating the assembled scratch copy and re-running; plus verification of the plan's per-step "Expected" claims, its counts, its `file:line` citations into descriptor-mnemonic, and static tracing of the CLI tests (never executed in the gate) against `run`/`parse_path` as written.

Already-settled facts (build gate result, F-219, `CliError` exits 1 in general, Stage-0-is-Rust-only) were taken as given and not re-derived, **except** where a mutation or a direct source read produced hard evidence against one of them (see I-1, M-1).

## VERDICT: 0C/2I/4M/2N

## Mutation table

Scratch copy: `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/702b37c9-e041-404f-8220-2456ff9c6bf3/scratchpad/lensB/plan-build-gate-md`. All 48 compose tests were the universe for every run (`cargo nextest run -p md-codec --locked --no-fail-fast -E 'binary(/^compose_/)'`); baseline is 47 passed / 1 pinned-red (`every_compose_vector_in_the_manifest_is_exactly_what_compose_renders`, "MANIFEST lacks", unrelated to any mutation). Every mutation below was applied, tested, and reverted individually (single-occurrence exact-string edits, verified reverted).

| # | mutation | file:line (scratch copy) | tests that failed | verdict |
|---|---|---|---|---|
| 1 | `or_d`/`or_i` swapped for a bare-multi head | `compose/lowering.rs:76` | `a_single_key_head_uses_or_i_not_or_d`, `a_locked_multi_path_is_unsorted_multi_without_the_experimental_mark`, `a_keyless_wsh_path_is_admitted_and_marked_experimental`, `eight_paths_chain_right_associatively_and_the_last_stands_alone`, `conjunct_order_is_keys_hash_lock`, `every_family_entry_renders_as_listed`, `two_path_wsh_with_a_bare_multi_head_uses_or_d` | CAUGHT |
| 2 | drop `sole &&` so a non-sole bare-multi head can get `sortedmulti` | `compose/lowering.rs:71` | `keyed_wsh_two_path_is_sane_and_derives_addresses`, `two_path_wsh_with_a_bare_multi_head_uses_or_d`, `every_compose_vector...`, `every_family_entry_renders_as_listed` | CAUGHT |
| 3 | conjunct order: hash and lock swapped | `compose/lowering.rs:50-56` | `a_keyless_wsh_path_is_admitted_and_marked_experimental`, `conjunct_order_is_keys_hash_lock`, `every_family_entry_renders_as_listed`, `every_compose_vector...` | CAUGHT |
| 4 | internal key = LAST unlocked single key, not first (`.position` → `.rposition`) | `compose/tr.rs:11` | `only_the_first_listed_unlocked_single_key_is_extracted`, `every_compose_vector...` | CAUGHT ONLY BY `only_the_first_listed_unlocked_single_key_is_extracted` — see I-note below |
| 5 | spine left-nested instead of right | `compose/tr.rs:16-22` | `four_leaves_form_a_right_spine`, `every_compose_vector...`, `every_family_entry_renders_as_listed` | CAUGHT |
| 6 | `is_nums` inverted (`ik.is_none()` → `ik.is_some()`) | `compose/tr.rs:35` | `the_first_listed_unlocked_single_key_becomes_the_internal_key_and_slot_zero`, `two_path_taproot_with_no_single_key_uses_nums_and_two_leaves`, `every_compose_vector...`, `every_family_entry_renders_as_listed` | CAUGHT |
| 7 | default account = slot index, not lowest-free | `compose/lowering.rs:136-149` | `compose_with_uses_declared_origins_and_fills_unseated_slots_with_the_lowest_free_account`, `every_compose_vector...` | CAUGHT ONLY BY `compose_with_uses_declared_origins_and_fills_unseated_slots_with_the_lowest_free_account` |
| 8 | invariant check (`IndistinguishableSlots`) removed | `compose/lowering.rs:150-161` | `compose_with_refuses_two_slots_at_one_origin_unless_both_fingerprints_differ`, `every_compose_vector...` | CAUGHT ONLY BY that one test |
| 9a | `all_same` forced `true` (always `Shared`) | `compose/lowering.rs:164` | `compose_with_uses_declared_origins_and_fills_unseated_slots_with_the_lowest_free_account`, `unseated_slots_take_ascending_default_accounts_under_the_wrapper_script_type`, `every_compose_vector...` | CAUGHT |
| 9b | `all_same` forced `false` (always `Divergent`) | `compose/lowering.rs:164` | `unseated_slots_take_ascending_default_accounts_under_the_wrapper_script_type`, `every_compose_vector...` | CAUGHT ONLY BY `unseated_slots_take_ascending_default_accounts_under_the_wrapper_script_type` |
| 10 | `OlderUnits` operand: `0x400000+u` → `u` | `compose/mod.rs:92` | `lock_operand_bands_are_inclusive_at_both_ends`, `a_time_lock_of_one_unit_encodes_as_0x400001`, `every_family_entry_renders_as_listed`, `every_compose_vector...` | CAUGHT |
| 11 | `AfterHeight` upper bound off-by-one (admits `h == 500_000_000`) | `compose/mod.rs:93` | `compose_refuses_lock_operands_outside_the_consensus_bands`, `every_compose_vector...` | CAUGHT ONLY BY that one test (confirmed: it is the exact `Lock::AfterHeight(500_000_000)` case that flips from `Err` to `Ok`) |
| 12 | `MAX_SLOTS` 32 → 33 | `compose/mod.rs:35` | `compose_refuses_a_thirty_third_slot`, `every_compose_vector...` | CAUGHT |
| 13 | keyless path under `tr` admitted (refusal branch deleted) | `compose/mod.rs:322-324` | `compose_refuses_a_keyless_path_under_tr`, `every_compose_vector...` | CAUGHT ONLY BY that one test |
| 14 | `Experimental::UnsortedKeys` never emitted | `compose/lowering.rs:112-124` | `sole_unsorted_multi_path_under_wsh_is_multi_and_experimental`, `every_compose_vector...` | CAUGHT ONLY BY that one test (wsh-only; see I-2 — no test independently checks this for `tr`, and the CLI never exercises it at all) |
| 15 | `pkh` → `pk` (`Tag::PkK`) in wsh single-key leaf | `compose/lowering.rs:46` | `single_key_under_wsh_is_pkh`, `two_path_wsh_with_a_bare_multi_head_uses_or_d`, `every_compose_vector...`, `every_family_entry_renders_as_listed` | CAUGHT |
| 16 | `template_with_origins`: drop the trailing `/` from the match key (prefix-insensitive replace, garbles slots ≥10 e.g. `@1` matching inside `@10`) | `compose/mod.rs:403` | *(none — identical to baseline: 47 passed, only the pinned-red MANIFEST failure)* | **NOT CAUGHT** — see I-1 |
| 17 | tr numbering ignores the internal key (`number(list, ik)` → `number(list, None)`) | `compose/tr.rs:25-26` | `only_the_first_listed_unlocked_single_key_is_extracted`, `the_first_listed_unlocked_single_key_becomes_the_internal_key_and_slot_zero`, `every_family_entry_renders_as_listed`, `every_compose_vector...` | CAUGHT |
| 18 | presets: `kofn_recovery` locks the primary path instead of the recovery path | `compose/presets.rs:38-42` | `presets_compose_and_carry_the_documented_shapes`, `every_compose_vector...` | CAUGHT ONLY BY that one test |
| 19 | legacy-wrapper shape: sortedness requirement dropped (`!(sole && sorted)` → `!sole`) | `compose/mod.rs:341` | `compose_refuses_legacy_wrappers_outside_the_single_sorted_multi_shape`, `every_compose_vector...` | CAUGHT |
| 20 | `MAX_KEYS_PER_PATH` 9 → 10 | `compose/mod.rs:33` | `compose_refuses_bad_thresholds`, `every_compose_vector...` | CAUGHT |

19 of 20 mutations were caught; several by exactly one test (noted above), which is fine where that test is squarely aimed at the rule (8, 11, 13, 18) but worth flagging where a same-named "obvious" test turns out **not** to be the one carrying the signal (4, 7, 9b — see I-note under the table and I-2).

## Findings

### I-1: `template_with_origins`'s slot-inlining has no test that would catch a regression before Task 5 lands, and none at all for the exact bug class named in the review brief

**Plan location:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md:658-659` (Task 1 Step 3, the `template_with_origins` body and its own defending comment: *"`@1/` never occurs inside `@10/` or `@11/`, and a slot appears once."*).

**Evidence:** Mutation 16 (drop the trailing `/` from both sides of the `replace` call at `compose/mod.rs:403`, scratch copy) produces **byte-identical test output to the unmutated baseline** — 47 passed, 1 pinned red, same failing test, same message. No test in the 48-test gate-runnable suite calls `template_with_origins` on a descriptor with 10 or more slots and checks the result against an independently-computed string. The two family entries that DO have ≥10 slots (`compose_wsh_thirty_two_slots`, `compose_tr_thirty_two_slots`, plan lines 1730-1735) are checked only via `descriptor_to_template` (origin-less rendering) in `every_family_entry_renders_as_listed` — never via `template_with_origins`.

The as-written code is **correct** (the trailing `/` is exactly the guard that defeats the `@1` ⊂ `@10` collision), so this is a coverage gap, not a live bug. The gap is real but narrower once Task 5 lands: the two 32-slot vectors' MANIFEST `template` field is generated **by the same printer that would encode the bug**, so a defect present *at generation time* ships silently in the pasted vectors (the round-trip check `every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` only catches a **regression introduced after** the vectors are pasted, never a bug baked in from the start). A Go port implementer reading a 32-slot compose vector would copy whatever spelling is there, correct or not.

**Why it matters:** this is exactly the "byte-for-byte, the Go port must reproduce it" function (plan's own docstring, `compose/mod.rs` module doc). No independent (non-self-generated) assertion exists anywhere in the plan's tests for the ≥10-slot inlining case.

**Remedy** (one sentence, not authoritative): add one hand-written (not printer-generated) assertion comparing `template_with_origins` against a literal expected string for a ≥10-slot descriptor, the way the ≤4-slot cases already are in `compose_lowering.rs`.

### I-2: Task 2's own test calls into taproot before taproot exists — the plan's Task 2 Step 4 "Expected: every test PASSES" is false

**Plan location:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md:1181` (Task 2, Step 4: *"Expected: every test PASSES (no taproot test exists yet)."*) contradicted by the test the SAME task adds at `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md:737-755`, `unseated_slots_take_ascending_default_accounts_under_the_wrapper_script_type`.

**Evidence:** that test's last four lines exercise **all four wrappers** to check the §4f `script_type` values, including:
```rust
let c = compose(&list(Wrapper::Tr, vec![keys(2, 2)])).unwrap();
assert_eq!(origins(&c), vec![hardened(&[48, 0, 0, 3]), hardened(&[48, 0, 1, 3])]);
```
At Task 2's checkpoint, `tr.rs` is still the Task-1/2 stub (`unimplemented!("the taproot lowering lands with its tests")`), and `Wrapper::Tr` is a structurally-valid, fully-keyed 2-of-2 list — it does NOT hit any `validate()` refusal, so `compose()` dispatches to `lower_tr` and panics. Reproduced directly: reverted `tr.rs` to the Task-1/2 stub in the otherwise-complete scratch copy and ran only this one test —
```
thread 'unseated_slots_take_ascending_default_accounts_under_the_wrapper_script_type' panicked at crates/md-codec/src/compose/tr.rs:7:5:
not implemented: the taproot lowering lands with its tests
```
By contrast, Task 1's own `compose_refuses_a_keyless_path_under_tr` test (plan line ~156) also uses `Wrapper::Tr`, but its list is structurally invalid (a keyless path under `tr`), so it's rejected by `validate()` **before** reaching the stub — that test's Task 1 checkpoint claim (line 687) is fine.

**Why it matters:** an implementer following the plan literally at Task 2 Step 4 would see a real, unexplained test failure at a checkpoint the plan promises is green, and could waste time debugging what looks like a Task-2 regression rather than recognizing it as an artifact of test/task sequencing.

**Remedy:** either move the `Wrapper::Tr` sub-case of that test to Task 3, or restate Task 2 Step 4's Expected line to name this one exception.

### M-1: The plan tells the implementer to write a doc-comment claiming "exits 1 like every other `CliError`," which is false for `BadArg` — the variant right next to it

**Plan location:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md:2175-2177` (Task 6, the `Compose(String)` variant's doc comment, to be pasted verbatim into `crates/md-cli/src/error.rs` directly after `BadArg(String),`); the same claim appears at plan line 1869.

**Evidence:** `crates/md-cli/src/main.rs` at `3b0944fb` (lines 787-800):
```rust
match dispatch(cli.command) {
    Ok(code) => ExitCode::from(code),
    Err(CliError::BadArg(m)) => {
        eprintln!("md: {m}");
        ExitCode::from(2)
    }
    Err(e) => {
        eprintln!("md: {e}");
        ExitCode::from(1)
    }
}
```
`CliError::BadArg` — the sibling variant the plan anchors `Compose` right after — exits **2**, not 1. `Compose` itself is fine (it falls into the generic `Err(e)` arm and correctly exits 1; the CLI test `compose_refuses_a_keyless_path_without_experimental_and_admits_it_with`'s `.code(1)` assertion is correct as traced), so **no test breaks**. The comment being committed into the codebase is simply an inaccurate statement about `CliError`'s actual behavior.

**Why it matters:** low — doesn't affect this plan's shipped behavior, but plants a wrong claim in a doc-comment a later reader (or agent) could rely on when adding a new `CliError` variant that isn't supposed to exit 1.

### M-2: Two more tests are vacuous in the gate besides the one the plan names, and the plan's "what it covers" section doesn't call them out

**Plan location:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md:2446` ("What the build gate covers, and does not" — names only `every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` as the pinned red).

**Evidence:** `every_compose_manifest_entry_is_in_the_family` and `keyed_compose_vectors_carry_four_journey_keys_or_fewer` (plan lines 1765-1803) both iterate `MANIFEST.iter().filter(...)`, which is empty in the gate assembly (`test_vectors.rs` is a fragment) — both PASS vacuously (0 iterations), confirmed in the baseline run (`PASS` at gate time). Task 5's paste of the 28 entries (per the task brief's explicit question) makes **both** real: the first would then catch an untagged/misnamed compose vector, the second would catch a keyed vector minted without keys or with a keys/fingerprints length mismatch.

**Why it matters:** minor — this doesn't invalidate anything, but the plan is precise about naming its ONE known vacuous-red test and silent about these two vacuous-PASS ones; a reader checking "did the gate actually verify X" for either of these two tests would wrongly conclude it did.

### M-3: `compose_refuses_lock_operands_outside_the_consensus_bands` wildcards the refusal reason, so a boundary case could return the wrong `why` and still pass

**Plan location:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md:226` (`assert!(matches!(err, ComposeError::LockOutOfRange { path: 0, .. }), "{why}: {err:?}");` — six band-violation cases share this one pattern with `why` wildcarded via `..`).

**Evidence:** by inspection (not mutated separately, since the actual boundary *values* are independently pinned by `lock_operand_bands_are_inclusive_at_both_ends` and mutation 11 above): this test only confirms *that* a `LockOutOfRange` fired for `path: 0`, never that the returned `why` string names the band actually violated. Swapping which `why` string a given branch of `Lock::operand()` returns (e.g. giving `AfterHeight(0)` the `"after time needs..."` message) would not be caught here.

**Why it matters:** low — the numeric boundaries themselves are well covered elsewhere; only the operator-facing wording could drift silently.

### M-4: A test named for "the first" doesn't independently prove firstness

**Plan location:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md:1230-1251`, `the_first_listed_unlocked_single_key_becomes_the_internal_key_and_slot_zero`.

**Evidence:** mutation 4 (`.position` → `.rposition`, i.e. LAST instead of FIRST unlocked single key) does **not** fail this test — its fixture (`[locked 2-of-2, unlocked 1-of-1, locked 1-of-1]`) has only ONE unlocked-single-key candidate, so "first" and "last" coincide. The mutation is caught only by the differently-named `only_the_first_listed_unlocked_single_key_is_extracted` (plan lines 1286-1295), which does have two candidates. Not a coverage gap (the rule IS tested), just a test whose name overstates what it independently demonstrates.

## N-1 (Nit): three preset-refusal assertions check `.is_err()` without the `ComposeError` kind

`design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md:2294-2296` (`presets_refuse_parameters_the_grammar_refuses`) — traced by hand against `presets.rs`/`mod.rs::validate`: all three do fail for the stated reason (`blocks()`'s u16 bound, and `validate`'s `BadThreshold`), so this is a style nit, not a live gap.

## N-2 (Nit): the codebase already has an unrelated `compose` — `crates/md-cli/src/seat/compose.rs`

Confirmed at `3b0944fb`: `seat::compose` handles seating a key assignment into a keyless policy card (an entirely different operation — filling placeholders, not lowering a path list). No Rust namespace collision (different parent modules, and `Compose` is not yet in the `Commands` enum — confirmed only `Compile` exists at `main.rs:266`), but two unrelated concepts share the English word "compose" in the same crate family; worth a one-line disambiguating mention somewhere for future readers.

## Claims checked

| # | claim | verdict | evidence |
|---|---|---|---|
| 1 | Task 1 Step 4: 8 refusal tests + `lock_operand_bands_are_inclusive_at_both_ends` PASS on the stub; `compose_admits_exactly_thirty_two_slots` FAILS with the stub panic | TRUE | traced: every refusal test hits a `validate()` error before `lower()`; the bands test never calls `compose`/`lower` at all; the 32-slot test is the only Task-1 test with a structurally-valid list, so it alone reaches the stub. |
| 2 | Task 2 Step 2: every new test except `compose_with_refuses_a_declaration_slice_of_the_wrong_length` fails with the stub panic | TRUE (not independently re-derived beyond confirming `WrongSlotCount` is returned before `lower()` is ever called — `compose_with` checks `declared.len() != n` first) | plan line 930; code trace of `compose_with` in Task 1's `mod.rs`. |
| 3 | Task 2 Step 4: every test PASSES (no taproot test exists yet) | **FALSE** | see I-2. |
| 4 | Task 3 Step 2: taproot tests FAIL with the stub, wsh tests still PASS | TRUE, with the same caveat as I-2 (the Task-2 `unseated_slots_...` test is *already* failing before Task 3, so it isn't a NEW failure at Task 3's checkpoint, but the plan's Task 2 promise was already broken by then) | reproduced empirically: reverted `tr.rs` to the stub in the full scratch copy and ran the whole compose suite; only tr-touching tests failed, all wsh-only tests passed. |
| 5 | Task 5 Step 2: `every_family_entry_renders_as_listed` and `every_tag_appears_in_at_least_two_vectors` PASS; `every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` FAILS with `MANIFEST lacks keyed_compose_wsh_sole_sortedmulti` | TRUE | reproduced exactly: baseline run's panic message is byte-identical to the plan's claim, at `compose_vectors.rs:162:77`. |
| 6 | Task 7 Step 2: FAILS with `could not find presets in compose` | TRUE (uncontroversial — module doesn't exist before Task 7 Step 3) | not independently reconstructed; trivial compile-error claim. |
| 7 | 28 family entries, 22 keyed / 6 unkeyed | TRUE | counted by hand from `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md:1647-1735`: 22 `keyed_compose_*` entries, 6 `compose_*` (unkeyed) entries = 28. |
| 8 | Every required tag present; every non-singular tag ≥ 2 | TRUE | `every_tag_appears_in_at_least_two_vectors` (a real test — it iterates the hardcoded `family()`, not `MANIFEST`, so it is NOT vacuous in the gate) PASSED in every run of the 48-test suite, including the final restored baseline. |
| 9 | `crates/md-codec/src/encode.rs:99-121` — `encode_payload` canonicalises and validates | TRUE | `git show 3b0944fb:crates/md-codec/src/encode.rs`: `pub fn encode_payload` starts line 99; `validate_no_duplicate_key_slots(d)?;` (last validation call) is line 120; bit-encoding (`BitWriter::new()`) starts line 122. Span is exact. |
| 10 | `crates/md-codec/src/error.rs:57-59` — `KeyCountOutOfRange` | TRUE | `git show 3b0944fb:...error.rs`, lines 57-59 are exactly the doc comment, `#[error(...)]` attribute, and `KeyCountOutOfRange {` opening. |
| 11 | `crates/md-codec/src/render.rs:17` — states the "pure wire/decode taxonomy" rule for `RenderError` | TRUE | `git show 3b0944fb:...render.rs:17` is exactly `//! \`md_codec::Error\` stays a pure wire/decode taxonomy.` |
| 12 | `crates/md-cli/src/cmd/vectors.rs:16` — the exporter that writes `.conformance.json` for every KEYED vector | TRUE (citation points to the `run` function's signature, not the specific `if !v.keys.is_empty()` gate a few lines further down; loose but not wrong) | `git show 3b0944fb:...vectors.rs:16` is `pub fn run(...)`; the keyed-only gate is at line 100 of the same file (`if !v.keys.is_empty() { ... write .conformance.json }`), inside that function. |
| 13 | "Every `CliError` exits 1" (task brief's settled fact / plan lines 1869, 2175-2177) | FALSE as a literal, general claim; TRUE for the specific `Compose` variant this plan adds | see M-1: `CliError::BadArg` exits 2 via its own `main.rs` match arm; `Compose` correctly lands in the generic exit-1 arm, so no test in this plan is affected. |
| 14 | No pre-existing `compose` module in `md-codec` or CLI-subcommand-name collision | TRUE, with the N-2 naming-overlap nit | `git show 3b0944fb:crates/md-codec/src/lib.rs` has no `compose` mod; `Commands` enum has no `Compose` variant (only `Compile` at `main.rs:266`); `crates/md-cli/src/seat/compose.rs` exists but is an unrelated module (seating, not lowering). |
| 15 | CLI tests in `cli_compose.rs` (never executed in the gate) match `run`/`parse_path` as written | TRUE for all 5 tests, traced by hand line-by-line (wrapper/path parsing, slot numbering, JSON field names, exit codes, the `older=4194305` case, the two "structural defect" wording checks) | see full trace in-session; no discrepancy found between the plan's CLI test assertions and the plan's own `run`/`parse_path` code. |

## What I ran

- `TMPDIR=.../scratchpad/lensB CARGO_TARGET_DIR=/scratch/code/shibboleth/.plan-gate-target-lensB bash scripts/plan-build-gate-md.sh design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md` — reproduced the settled 47/1/clean result.
- For each of 20 mutations: an exact single-occurrence string edit (via a small Python `mutate.py` helper enforcing count==1 both ways) to a file under `.../plan-build-gate-md/crates/md-codec/src/compose/{mod,lowering,tr,presets}.rs`, then `cargo nextest run -p md-codec --locked --no-fail-fast -E 'binary(/^compose_/)'` with the same `CARGO_TARGET_DIR`, then an exact revert, confirmed by the python helper's success message (one-occurrence match) each way.
- One additional reconstruction: reverted `compose/tr.rs` to the Task-1/2 stub inside the otherwise-complete scratch copy, ran the whole compose suite, then ran the single test `unseated_slots_take_ascending_default_accounts_under_the_wrapper_script_type` in isolation (`-E 'test(...)'`) to get an unambiguous panic-site confirmation, then restored `tr.rs` (`diff` confirmed byte-identical to the pre-reconstruction file) and reran the full suite to confirm the 47/1 baseline was intact.
- `git show 3b0944fb:<path>` for every cited file in descriptor-mnemonic (`encode.rs`, `error.rs`, `render.rs`, `crates/md-cli/src/cmd/vectors.rs`, `crates/md-cli/src/main.rs`, `crates/md-codec/src/lib.rs`, `crates/md-cli/src/parse/template.rs`, `crates/md-cli/src/seat/compose.rs`) plus `git ls-tree -r 3b0944fb --name-only` for the naming-collision check.
- Static line-by-line trace of all 5 tests in the plan's `crates/md-cli/tests/cli_compose.rs` against the plan's own `crates/md-cli/src/cmd/compose.rs` `run`/`parse_path`, since `main.rs` is a fragment the gate cannot execute.
- Manual count of the 28 `family()` entries in Task 5 (22 keyed / 6 unkeyed) against the plan's claimed counts.
- Did not read any `.jsonl` file.
