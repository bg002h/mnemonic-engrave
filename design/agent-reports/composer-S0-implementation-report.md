# Composer Stage 0 — implementation report

**Plan:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md` at mnemonic-engrave `3533638` (R0 GREEN, 9 tasks).
**Worktree:** `/scratch/code/shibboleth/wt-composer-s0`
**Branch:** `composer-s0` (from `descriptor-mnemonic` `b19dca7b`)
**Toolchain:** 1.85.0 (`rust-toolchain.toml`), `cargo-nextest`.
**Outcome:** all 9 tasks implemented and committed. Whole-workspace gate GREEN. No STOP clause triggered.

## Commits (`git log --oneline b19dca7b..HEAD`)

```
9820e618 md-codec/md-cli: compose vector corpus regenerated; release note (composer S0 task 9)
99600923 md-cli: encode runs the miniscript sanity gate for every wrapper, not only tr; --experimental relaxes the signature rule alike (composer S0 task 8)
b18e9ce4 md-codec: compose presets -- plain k-of-n and the five toolkit archetypes as path lists, pinned templates, 5b-checked (composer S0 task 7)
ce4d03f9 md-cli: md compose -- fixed lowering from a path DSL, --experimental gate, --json (composer S0 task 6)
aff5281f md-codec: compose vector family -- 28 tagged vectors (26 in MANIFEST), every required tag twice, the 5b cross-check over all of them, keyed ones export conformance.json (composer S0 task 5)
1dd351a9 md-codec: compose cross-check -- reusable 5b check (sanity, address, lift equality vs the compiler), top_unsafe for keyless wsh (composer S0 task 4)
17a96d0c md-codec: compose -- taproot internal-key extraction, right spine, NUMS, declared origins (composer S0 task 3)
8eca997f md-codec: compose -- path body, wsh or_d/or_i chain, legacy wrap, numbering, default origins + invariant (composer S0 task 2)
ebdd898e md-codec: compose module -- types, structural refusals, lock bands (composer S0 task 1)
```

`git diff --stat b19dca7b..HEAD` → **145 files changed, 8231 insertions(+), 13 deletions(-)**. Working tree clean at HEAD.

Every code block was extracted from the plan with `sed -n '<a>,<b>p'` on the plan file and written verbatim — no code in this branch was retyped by hand. The one exception is the Task 5 `MANIFEST` block, which was generated mechanically from the printer test's own stdout (see Task 5 below).

---

## Task 1 — types, structural refusals, lock bands

**Step 2 (fail first):**
```
error[E0432]: unresolved import `md_codec::compose`
error: could not compile `md-codec` (test "compose_lowering") due to 1 previous error
```
Matches the plan's Expected.

**Step 4 (after the types + stub):**
```
     Summary [   0.005s] 11 tests run: 10 passed, 1 failed, 0 skipped
        FAIL [   0.003s] ( 3/11) md-codec::compose_lowering compose_admits_exactly_thirty_two_slots

    thread 'compose_admits_exactly_thirty_two_slots' panicked at crates/md-codec/src/compose/lowering.rs:7:5:
    not implemented: the wsh lowering lands with its tests
```

**DEVIATION 1 (Nit, count only).** The plan's Expected says *"the **eight** refusal tests and `lock_operand_bands_are_inclusive_at_both_ends` PASS"*. There are **nine** refusal tests in the file, so **10 passed**, not 9. Substance matches exactly: everything except `compose_admits_exactly_thirty_two_slots` passes, and that one fails on the stub with the expected message.

**DEVIATION 2 (Important-shaped for the plan, harmless for the code): Task 1 Step 5's `Expected: clean` clippy line is unattainable at Task 1, and Task 2's is unattainable at Task 2.** The plan's build gate assembles Tasks 1-3 and 7's files *together*, so it never sees the intermediate states that a per-task commit sequence creates. Measured:

- After Task 1 (`cargo clippy --locked -p md-codec --all-targets -- -D warnings`):
```
error: method `is_bare_single` is never used
   --> crates/md-codec/src/compose/mod.rs:135:8
    = note: `-D dead-code` implied by `-D warnings`
```
plus six unused-import warnings in `tests/compose_lowering.rs` (`canonicalize_placeholder_indices`, `descriptor_to_template`, `PathDeclPaths`, `Composed`/`Experimental`/`SlotOrigin`/`compose_with`/`template_with_origins`, `encode_payload`/`encode_md1_string`, `reassemble`/`split`) — every one of them consumed by the tests Task 2 adds.
- After Task 2: `method is_bare_single is never used` **and** `field path_index is never read` — both consumed by `tr.rs`, which Task 3 creates.

**Resolution taken:** no unplanned code was added (no `#[allow(dead_code)]`, no reordering). The deviation is recorded here and the gate was verified to close on its own at the point the plan's assembled state exists — **clippy is clean from Task 3 onward and at every later task**, including `--workspace --all-targets --all-features` in Task 9. `cargo fmt --all --check` was clean at every task including 1 and 2.

Commit `ebdd898e`.

---

## Task 2 — path body, wsh chain, numbering, origins

**Step 2 (fail first):** `25/28 tests run: 11 passed, 14 failed` — every new test except `compose_with_refuses_a_declaration_slice_of_the_wrong_length` failed with
```
    thread 'slots_are_numbered_by_first_appearance_and_canonicalisation_is_identity' panicked at crates/md-codec/src/compose/lowering.rs:10:5:
    not implemented: the wsh lowering lands with its tests
```
Matches Expected.

**Step 4 (after the lowering):**
```
     Summary [   0.023s] 28 tests run: 28 passed, 0 skipped
```
Matches Expected. **No rendered string differed from any expectation** — the renderer-authority escape hatch in Step 4 was not needed and no test string was edited.

Commit `8eca997f`.

---

## Task 3 — taproot

**Step 2 (fail first):** `27/38 tests run: 21 passed, 6 failed`, the six being the taproot tests, each with
```
    thread '...' panicked at crates/md-codec/src/compose/tr.rs:10:5:
    not implemented: the taproot lowering lands with its tests
```
(Five more taproot tests were not run due to fail-fast; every wsh test still passed.) Matches Expected.

**Step 4 (after `tr.rs`):**
```
     Summary [   0.021s] 38 tests run: 38 passed, 0 skipped
```
Matches Expected. No lowering fix and no expectation edit was needed.

**Step 5:** `cargo fmt --all --check` clean; `cargo clippy --locked -p md-codec --all-targets -- -D warnings` → `Finished` with no diagnostics. This is where Deviations 1-2's transient dead-code disappears, as predicted.

Commit `17a96d0c`.

---

## Task 4 — the §5b cross-check

**Step 2:**
```
     Summary [   0.188s] 3 tests run: 3 passed, 0 skipped
        PASS a_keyless_wsh_path_is_admitted_with_top_unsafe_and_refused_by_the_default_sanity
        PASS the_cross_check_notices_a_wrong_lowering
        PASS the_reference_two_path_wallets_pass_the_cross_check
```
Matches Expected. **`Cargo.lock` did not change** after adding the `compiler` dev-feature (`git status --short Cargo.lock` → empty), exactly as the plan predicted, so `--locked` accepted it and nothing was committed to the lock.

**Step 3:** fmt clean; `clippy --all-targets --all-features -- -D warnings` clean.

Commit `1dd351a9`.

---

## Task 5 — the vector family

**Step 2 (fail first), the plan's predicted single red:**
```
     Summary [  30.222s] 9 tests run: 8 passed, 1 failed, 0 skipped
        FAIL md-codec::compose_vectors every_compose_vector_in_the_manifest_is_exactly_what_compose_renders

    thread '...' panicked at crates/md-codec/tests/compose_vectors.rs:35:77:
    MANIFEST lacks keyed_compose_wsh_sole_sortedmulti
```
`every_family_entry_renders_as_listed`, `every_tag_appears_in_at_least_two_vectors` and `every_family_entry_passes_the_5b_cross_check` all PASSED. Matches Expected exactly.

**Step 3 (the printer):** `cargo nextest run --locked -p md-codec --test compose_vectors print_family --no-capture` emitted **26** `name<TAB>template` lines (22 `keyed_compose_*` + 4 unkeyed `compose_*`; the two `no-corpus` keyless-wsh entries were skipped) — the plan's expected count. Those 26 lines were piped to a file and the `Vector { .. }` entries generated from it by script, so **no template was retyped**; keys and fingerprints were bound by the rules in the plan (journey xpubs in emitted slot order, `[0x73,0xc5,0xda,0x0a]` per slot, `[0x11;4]`/`[0x22;4]`/`[0x33;4]`/`[0x44;4]` on the two `*_distinct_fingerprints` entries, `keys: &[]`/`fingerprints: &[]` on the four unkeyed ones, `path: None` throughout). `XPUB_JOURNEY_0..3` were declared above `MANIFEST`, their values copied from `compose_support.rs::XPUB`.

**DEVIATION 3 (Minor, plan command wrong): `cargo run --locked -p md-cli -- vectors` does not run.**
```
error: `cargo run` could not determine which binary to run. Use the `--bin` option to specify a binary, or the `default-run` manifest key.
available binaries: gen_ap2_grind, md
```
`md-cli` has two binaries at this revision. Used `cargo run --locked -p md-cli --bin md -- vectors --out ...` throughout (Task 5 Step 4 and Task 9 Step 2). No behaviour change.

**DEVIATION 4 (Minor, plan under-specified): every one of the 26 compose vectors needs `force_chunked: true`, not `false`.** The plan says *"Use `force_chunked: false` except where the exporter reports `PayloadTooLongForSingleString`, in which case set it `true`."* Measured — the exporter refuses each in turn:
```
md: codec error: payload is 152 data symbols; the codex32 regular code caps single strings at 80 (use chunked encoding / --force-chunked)   [compose_tr_seven_leaves]
md: codec error: payload is 304 data symbols; ...                                                                                            [compose_tr_thirty_two_slots]
md: codec error: payload is 164 data symbols; ...                                                                                            [compose_wsh_eight_paths]
md: codec error: payload is 286 data symbols; ...                                                                                            [compose_wsh_thirty_two_slots]
```
and for the keyed class, measured on the **smallest** member (`keyed_compose_tr_key_path_only`, one xpub) by flipping it back to `false` and re-running:
```
md: codec error: payload is 131 data symbols; the codex32 regular code caps single strings at 80 (use chunked encoding / --force-chunked)
```
131 > 80 with a single key, so every larger keyed entry exceeds it too — consistent with `test_vectors.rs`'s own standing note that *"ALL KEYED ENTRIES ARE `force_chunked`, and not as a style choice: real xpubs push the payload past the codex32 regular code's 80-data-symbol cap."* All 26 were set `true`; the exporter then exits 0.

**Step 4 (md-codec):**
```
     Summary [  34.901s] 515 tests run: 515 passed, 2 skipped
```
Exporter: **22** `keyed_compose_*.conformance.json` files written — the plan's expected count (26 compose entries in MANIFEST, 28 in `family()`).

**DEVIATION 5 (Important-shaped for the plan, resolved by Task 9): Task 5 Step 4's "`cargo nextest run --locked -p md-cli` must also PASS" cannot hold until Task 9 regenerates the committed corpus.**
```
     Summary [   2.101s] 752 tests run: 751 passed, 1 failed, 1 skipped
        FAIL [   0.111s] md-cli::vector_corpus vectors_output_matches_committed_corpus
    thread 'vectors_output_matches_committed_corpus' panicked at crates/md-cli/tests/vector_corpus.rs:41:5:
    vectors corpus drift detected
```
The drift is **126 `Only in <tmpdir>` lines and ZERO `differ` lines** — i.e. purely the new compose files missing from `crates/md-codec/tests/vectors`, with no pre-existing vector file changed. That regeneration is Task 9 Step 2's job, and the test PASSES after it (see Task 9). The plan's own Task 9 Step 2 shows it knew this; only the Task 5 Expected line is out of order. `template_roundtrip::reencode_round_trip_each_manifest_entry` and `corpus_origin_consistency::no_vector_declares_one_origin_for_two_different_keys` — the two md-cli tests that iterate `MANIFEST` through the real parser and encoder — **PASSED at Task 5**, which is the substance of what that step was checking.

**SELF-INFLICTED SLIP, found and fixed inside the task (recorded because it was mine, not the plan's).** My first paste put the `XPUB_JOURNEY_*` consts between `MANIFEST`'s doc comment and `#[rustfmt::skip] pub const MANIFEST`, detaching the doc and producing `warning: missing documentation for a constant --> crates/md-codec/src/test_vectors.rs:77:1`. Moved the consts *above* the doc block; a forced recompile then shows **0** occurrences of that warning under `cargo build --locked -p md-codec --all-targets` (baseline `HEAD:test_vectors.rs` also 0), and `clippy ... -D warnings` is clean. Sightings of that warning after the repair were cargo replaying cached diagnostics from the pre-repair artifact, not a live diagnostic.

**Step 5:** fmt clean; clippy clean. Commit `aff5281f`.

---

## Task 6 — `md compose`

**Step 2 (fail first):**
```
     Summary [   0.004s] 6 tests run: 0 passed, 6 failed, 0 skipped
```
(all six `cli_compose` tests, `md compose` being an unrecognized subcommand). Matches Expected.

**Step 4 (after the subcommand):**
```
     Summary [   1.704s] 758 tests run: 757 passed, 1 failed, 1 skipped
        FAIL [   0.100s] md-cli::vector_corpus vectors_output_matches_committed_corpus
```
All six `cli_compose` tests PASS. The single failure is Deviation 5's corpus drift, unchanged. **No pre-existing test needed its expectation updated** — the plan warned that `gui-schema`/`gen-man` might pin a subcommand count or flag set; none did, so nothing was touched.

**Step 5:** fmt clean; `clippy --locked -p md-cli --all-targets --all-features -- -D warnings` clean. Commit `ce4d03f9`.

---

## Task 7 — presets

**Step 2 (fail first):**
```
695 | use md_codec::compose::presets;
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ no `presets` in `compose`
error[E0432] ... could not compile `md-codec` (test "compose_lowering")
```
Matches Expected.

**Step 4 (after `presets.rs`):**
```
     Summary [  36.628s] 519 tests run: 519 passed, 2 skipped
```
All three preset tests (`presets_compose_and_carry_the_documented_shapes`, `presets_lower_to_their_pinned_templates`, `presets_refuse_parameters_the_grammar_refuses`) and `every_preset_passes_the_5b_cross_check` pass. No pinned template needed adjusting. fmt clean; clippy clean.

Commit `b18e9ce4`.

---

## Task 8 — `md encode` gates a signature-free path under every wrapper

**Step 2 (fail first):**
```
     Summary [   0.011s] 3 tests run: 1 passed, 2 failed, 0 skipped
        FAIL md-cli::cli_compose_encode_gate encode_refuses_a_sigless_wsh_path_unkeyed_unless_experimental
        FAIL md-cli::cli_compose_encode_gate encode_refuses_a_sigless_wsh_path_keyed_unless_experimental
```
The regression guard `encode_still_admits_a_signed_wsh_policy_without_the_flag` PASSED. Matches Expected exactly, and reproduces the follow-up's premise: the unflagged encode exits 0 today.

**Step 4 (after the `parse_template_ext` change) — the STOP clause was NOT triggered.** No pre-existing test and no corpus vector became refused by `sanity_check`.
```
     Summary [   1.683s] 761 tests run: 760 passed, 1 failed, 1 skipped
        FAIL md-cli::vector_corpus vectors_output_matches_committed_corpus     [Deviation 5, unchanged]
```
The four tests the plan names by hand, all PASS:
```
        PASS md-cli::cli_compose_encode_gate encode_refuses_a_sigless_wsh_path_unkeyed_unless_experimental
        PASS md-cli::cli_compose_encode_gate encode_refuses_a_sigless_wsh_path_keyed_unless_experimental
        PASS md-cli::cmd_encode experimental_admits_a_keyless_spend_path
        PASS md-cli::n1_admission_taxonomy r_n1a_card_verifies_at_exit_0_with_a_warning
        PASS md-cli::n1_admission_taxonomy verify_template_warns_and_completes_on_a_refused_shape
```
(the last two being the reading-verb tests that justify the minting-only placement). md-codec re-run unchanged: `519 tests run: 519 passed, 2 skipped`. The plan's own hand-check figure — 761 in the md-cli suite — matches the 761 seen here.

**Step 5:** fmt clean; clippy clean. Commit `99600923`.

---

## Task 9 — whole-stage gate, corpus, release note

**ORDERING NOTE (deliberate, recorded).** Step 2 (regenerate the corpus) was run **before** Step 1's test leg, because Step 1 cannot be green while Deviation 5's drift stands. `cargo fmt --all --check` and the workspace clippy were run first, as written; the drift evidence for Step 1's pre-regeneration state is the Task 8 run above.

**Step 1 — the workspace the way CI does it:**
```
$ cargo fmt --all --check
FMT CLEAN

$ cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
    Finished `dev` profile [optimized + debuginfo] target(s) in 2.97s        (exit 0, no diagnostics)

$ cargo nextest run --locked --workspace --all-features
     Summary [  33.419s] 1318 tests run: 1318 passed, 3 skipped              (exit 0)
        PASS [   0.135s] (1100/1318) md-cli::vector_corpus vectors_output_matches_committed_corpus

$ cargo test --locked --workspace --all-features --doc
   Doc-tests md_codec
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out    (exit 0)
```
Also run, per the constellation note that CI's threaded runner has exposed shared-state bugs nextest's process isolation hides:
```
$ cargo test --locked --workspace --all-features
exit 0    (no failing `test result:` line)
```
Of the 1318, **52** are compose tests in md-codec (`compose_lowering` + `compose_crosscheck` + `compose_vectors`) and **9** are the md-cli compose tests (`cli_compose` 6 + `cli_compose_encode_gate` 3).

**Step 2 — corpus regeneration.**
```
$ cargo run --locked -p md-cli --bin md -- vectors --out crates/md-codec/tests/vectors
(exit 0)

$ git status --short crates/md-codec/tests/vectors
126 lines, ALL of them `?? ` (untracked-new)
  non-`??` lines (modified or deleted pre-existing vector files): 0
  new files not matching `compose_`:                             0
  breakdown: 110 `keyed_compose_*` + 16 unkeyed `compose_*` = 22x5 + 4x4
  distinct new vectors: 26
  `*.conformance.json`: 22
```
This is the plan's Expected precisely: only NEW `compose_*`/`keyed_compose_*` files, 26 vectors' worth, and **no existing vector file changed**.

**Step 3 — release note.** `CHANGELOG.md` at the repo root: the `md compose` subcommand added under the existing `## md-cli [Unreleased]` → `### Added`, the `md encode` behaviour change under a new `### Changed` in that same section (naming the closed follow-up `md-encode-keyless-template-sigless-path-not-gated`), and a **new `## md-codec [Unreleased]` section** (none existed; the newest md-codec heading was `[0.42.0]`) carrying `md_codec::compose`, the 28 compose vectors / 26 in MANIFEST / 22 conformance records, and the miniscript `compiler` dev-dependency. **No version was bumped and nothing was published.**

Commit `9820e618`.

---

## Things I decided, and things I did not do

1. **Deviations 1-5 above are the complete list of divergences from the plan's Expected lines.** None required a design decision; each is recorded with the exact output that produced it. No STOP clause fired: Task 8 Step 4's (a pre-existing test or corpus vector refused by `sanity_check`) did not occur, and Task 5 Step 4's (the exporter refusing an entry) occurred only as the anticipated `force_chunked` case the plan itself tells the implementer to resolve, not as a refusal of an entry.
2. **The one judgment call: Deviation 2.** The plan's Task 1 and Task 2 clippy gates are unattainable in a per-task commit sequence, and I chose to record that and continue rather than stop at Task 1 or add an unplanned `#[allow(dead_code)]`. Reasoning: the failing lint is `dead-code` on `is_bare_single`/`path_index`, both of which the plan's *own* Task 3 file consumes; it is an artifact of the plan's build gate assembling Tasks 1-3+7 at once, not a defect in any code the plan specifies. Stopping at Task 1 for it would have left eight tasks unbuilt over a lint that clears itself two commits later. Every later gate, including the Task 9 workspace clippy, is clean. **If the reviewer disagrees, the fix is a plan edit (Task 1/2 Expected lines), not a code edit.**
3. **Not done, by instruction and by plan:** no push, no tag, no publish, no version bump; the fork (`third_party/seedhammer`, `md/testdata/vectors/`) was not touched — the plan assigns that vendoring to Stage 2's implementer on a separate fork commit. The main `descriptor-mnemonic` checkout was not modified; all work is in the `composer-s0` worktree. Nothing under `mnemonic-engrave` was modified except this report.
4. **Still unproven by anything here** (carried from the plan's own "what the gate does not cover"): the Go port. The 22 `keyed_compose_*.conformance.json` records now exist in `crates/md-codec/tests/vectors` for Stage 2 to measure against, and the two `no-corpus` keyless-wsh vectors remain reachable only through `family()` and the §5b cross-check, as designed.
5. **Next per the plan's Task 9 Step 4:** the whole-diff independent review (opus execution review over `git diff b19dca7b..HEAD`) persisted to `design/agent-reports/composer-S0-exec-review-r0.md`, to 0C/0I, before the stage is complete.
