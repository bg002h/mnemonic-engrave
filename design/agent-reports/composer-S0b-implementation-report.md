# Composer S0b implementation report — `md compose --preset` + six archetype vectors (F-453)

**Implementer:** single agent, executing `mnemonic-engrave/design/IMPLEMENTATION_PLAN_composer_S0b_presets.md` at mnemonic-engrave master `373436d69f3b7eeea39d283f9fce5af05d31645b` (STATUS: R0 GREEN). Verified before starting: `373436d6` is an ancestor of engrave master `85b59f2`, and `git diff 373436d6..85b59f2 -- design/IMPLEMENTATION_PLAN_composer_S0b_presets.md` is **empty** — the plan file is byte-identical to the GREEN revision.

**Worktree:** `/scratch/code/shibboleth/wt-composer-s0b`
**Branch:** `composer-s0b`, created off descriptor-mnemonic `main` = `66bdf2f47e7fc703d5fb09120122b3e98cab5528` (the plan's declared baseline; `main` was verified to still be at exactly that SHA).
**Toolchain:** `rustc 1.85.0 (4d91de4e4 2025-02-17)` — the repo's `rust-toolchain.toml` pin.
**Result:** all three tasks complete, three commits, working tree clean, whole workspace green at **1340 tests run: 1340 passed, 3 skipped** — the plan's exact predicted number.

```
$ git log --oneline main..HEAD
87bc10ff md-codec/md-cli: preset vector corpus regenerated; release notes (F-453 composer S0b task 3)
5002ebac md-cli: md compose --preset -- the six archetypes over presets::*, mutually exclusive with --path, --json preset field (F-453 composer S0b task 2)
4793619b md-codec: six preset MANIFEST vectors -- one per archetype, built by calling presets::*, singular preset:<name> tags (F-453 composer S0b task 1)

$ git diff --stat main..HEAD | tail -1
 36 files changed, 2134 insertions(+), 28 deletions(-)
```

No design decision was made by the implementer. Every code block, fragment and commit message was applied verbatim from the plan (extracted mechanically with `sed` from the plan's fenced blocks, never retyped). Nothing was left undone; nothing forced a stop.

---

## Task 1 — the six preset vectors (`family()`, tags, `MANIFEST`, exporter)

### Step 1 — family rows without the MANIFEST entries: the pinned red

`crates/md-codec/tests/compose_support.rs` was replaced in full from the plan's block (lines 77–429). Before applying, the extracted file was diffed against the committed one; **the only differences were the intended ones** — the `use md_codec::compose::presets;` import, the six new `family()` rows with their comment header, and the `SINGULAR_TAGS` expansion (`spine:0` + `head:hashed` + the six `preset:<name>` tags). No pre-existing row, literal or tag was touched.

```
$ cargo nextest run --locked -p md-codec --test compose_vectors --test compose_crosscheck
    thread 'every_compose_vector_in_the_manifest_is_exactly_what_compose_renders' panicked at
      crates/md-codec/tests/compose_vectors.rs:45:32:
    MANIFEST lacks keyed_compose_preset_plain_multisig
     Summary [  38.883s] 11 tests run: 10 passed, 1 failed, 0 skipped
        FAIL [   0.014s] ( 7/11) md-codec::compose_vectors every_compose_vector_in_the_manifest_is_exactly_what_compose_renders
```

**Matches the plan's Expected line exactly**, including the vector name (`keyed_compose_preset_plain_multisig`) and the pinned-red shape (`MANIFEST lacks`). The other ten — including `every_family_entry_renders_as_listed`, `every_tag_appears_in_at_least_two_vectors`, `every_family_entry_passes_the_5b_cross_check` and `every_preset_passes_the_5b_cross_check` — passed, so the six hand-typed expected-template literals and the six new `SINGULAR_TAGS` entries were right on the first run.

### Step 2 — the six MANIFEST entries: green

Plan lines 440–469 inserted verbatim after the last existing entry (`compose_tr_thirty_two_slots`, ending at `test_vectors.rs:445`) and before the closing `];`. Insertion point re-read afterwards to confirm placement.

```
$ cargo nextest run --locked -p md-codec
     Summary [  38.789s] 519 tests run: 519 passed, 2 skipped
```

Compose-binary count, obtained without a second suite run:

```
$ cargo nextest list --locked -p md-codec -E 'binary(/^compose_/)' | wc -l
52
```

**52 — the plan's Expected number**, unchanged from S0 as predicted (the six new `family()` tuples are data consumed by existing `#[test]` functions, not new test functions).

### Step 3 — exporter file-count delta

```
$ cargo run --locked -p md-cli --bin md -- vectors --out /tmp/compose-vectors-f453 >/dev/null \
    && ls /tmp/compose-vectors-f453 | grep -c 'keyed_compose_preset_.*conformance.json'
6
$ ls /tmp/compose-vectors-f453 | grep -c 'keyed_compose_preset_'
30
```

**6 conformance records, 30 files total — exactly the plan's Expected `6`** and the stated 6 × 5 delta.

Then the md-cli suite with the corpus not yet regenerated:

```
$ cargo nextest run --locked -p md-cli
     Summary [   2.040s] 761 tests run: 760 passed, 1 failed, 1 skipped
        FAIL [   0.107s] (756/761) md-cli::vector_corpus vectors_output_matches_committed_corpus
    thread 'vectors_output_matches_committed_corpus' panicked at crates/md-cli/tests/vector_corpus.rs:41:5:
    vectors corpus drift detected
```

Drift-line audit (the plan's stop condition):

```
$ grep -c 'Only in' <run output>            → 30
$ grep 'Only in' <run> | grep -v 'keyed_compose_preset_' | wc -l   → 0
```

**30 "Only in" lines, every one a `keyed_compose_preset_*` file; zero touching any pre-existing vector.**

> **Reading note, so a later reader does not misread the plan's stop condition.** A naive `grep -c 'differ'` over that run returns **22**, not 0. All 22 are *test names* containing the word (`the_default_is_multipath_and_differs_from_both_single_paths`, `r3_origins_differ_pair_is_still_spend_equal`, `synthetic_for_0_and_1_differ`, …), each on a `PASS` line. **Zero** are corpus-diff `differ` lines. The plan's "any 'differ' line is a finding, stop" is satisfied; the raw count is not the right measurement.

### Step 4 — fmt, clippy, commit

`cargo fmt --all` produced **no change** to Task 1's diff (`git diff --stat` identical before and after). `cargo clippy --locked -p md-codec --all-targets --all-features -- -D warnings` finished clean, zero warnings. Committed with the plan's message via `git commit -F`, trailers intact.

---

## Task 2 — `md compose --preset`

### Step 1 + Step 2 — the failing CLI tests

`crates/md-cli/tests/cli_compose_preset.rs` created from plan lines 549–1051: **503 lines, 21 `#[test]` functions** — the plan's stated 21.

```
$ cargo nextest run --locked -p md-cli --test cli_compose_preset
     Summary [   0.038s] 21 tests run: 2 passed, 19 failed, 0 skipped
    error: unexpected argument '--preset' found
    Usage: md compose --wrapper <WRAPPER> --path <PATH>
```

**DEVIATION from the plan's Expected line (Minor, recorded not fixed).** The plan says "every test using it fails immediately with clap's own `error: unexpected argument '--preset' found`, exit 2". Nineteen did. **Two passed before any implementation existed:**

- `path_json_names_no_preset` — it asserts `v["preset"].is_null()`, and `serde_json::Value`'s `Index` returns `Null` for an **absent** key, so the assertion holds identically in the world where the field was never added. It passes in both worlds.
- `compose_refuses_when_neither_path_nor_preset_given` — it asserts only `.failure().code(2)`, which the pre-change `--path required = true` already produced for a different reason. It too passes in both worlds.

Neither is wrong after the change (both still pass, and `path_json_names_no_preset` does exercise the real field once it exists — the JSON now carries an explicit `"preset": null`), but neither could have failed for the intended reason, so neither carries fail-then-pass evidence. Nothing in the plan's design depends on this; recorded as a test-strength observation for the whole-diff reviewer, not treated as a blocker.

### Step 3 — `compose.rs` replaced

Replaced in full from plan lines 1064–1654. Diffed against the committed file before applying: the only **removals** were the 22 lines the plan's architecture describes — the `use` line's import list, the inline sha256 arm now factored into `parse_sha256_hex`, and `run`'s old path-list construction. Verified the factored-out helper reproduces `--path`'s message byte-for-byte (`ctx` is `format!("path \`{s}\`")`, so `path \`…\`: sha256 needs 64 hex characters, lowercase` is unchanged).

### Step 4 — `main.rs` fragments

Both regions were **re-read from the working tree and confirmed byte-identical to the plan's quoted "currently reads" text** before replacement: the `Compose` clap variant at `:282-297` and the dispatch arm at `:993-998`. Both fragments applied with an asserting script (it asserts the first and last line of each target range before splicing), not by hand-editing.

### Step 5 — the CLI tests pass

```
$ cargo nextest run --locked -p md-cli --no-fail-fast \
    -E "binary(/^cli_compose/) + test(every_preset_name_parses_with_some_valid_parameters)"
     Summary [   0.017s] 31 tests run: 31 passed, 753 skipped
        PASS [   0.003s] (21/31) md-cli::bin/md cmd::compose::tests::every_preset_name_parses_with_some_valid_parameters
```

**31/31 — the plan's Expected number**, including the `#[cfg(test)]` unit test that reaches `PRESET_NAMES`/`parse_preset` from inside the crate.

```
$ cargo nextest run --locked -p md-cli
     Summary [   3.473s] 783 tests run: 782 passed, 1 failed, 1 skipped
        FAIL [   0.121s] (778/783) md-cli::vector_corpus vectors_output_matches_committed_corpus
```

**782/783 with only the not-yet-regenerated corpus test red — exactly the plan's Expected line**, and 761 + 22 = 783 confirms the stated 22-test addition with zero regressions.

### Step 6 — fmt, clippy, commit

`cargo fmt --all` reformatted `compose.rs` and `cli_compose_preset.rs` (the plan predicted this: line-wrapping normalisation); `cargo fmt --all -- --check` then clean. `cargo clippy --locked -p md-cli --all-targets --all-features -- -D warnings` clean, zero warnings. Committed with the plan's message.

---

## Task 3 — whole-workspace gate, corpus regeneration, release notes

### DEVIATION: Step 1's Expected line cannot hold in the plan's own step order

**Recorded, worked around, not a stop.** Task 3 Step 1 runs the whole-workspace gate and expects "every test PASS … 1340 passed", but it is written **before** Step 2, which regenerates the corpus that makes `vectors_output_matches_committed_corpus` green. The plan states this contradiction itself elsewhere — Task 2 Step 5's Expected line says 783/783 holds "once Task 1's corpus is regenerated (Task 3)". I ran the steps **in the written order anyway** and then re-ran the gate after Step 2, so both halves of the fail-then-pass are on the record.

Step 1 as written (pre-regeneration):

```
$ cargo fmt --all --check                                                  → clean
$ cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
    Finished `dev` profile … in 4.41s     (exit 0, zero 'warning' lines)
$ cargo nextest run --locked --workspace --all-features
     Summary [  52.985s] 1219/1340 tests run: 1218 passed, 1 failed, 3 skipped
        FAIL [   0.157s] (1196/1340) md-cli::vector_corpus vectors_output_matches_committed_corpus
    warning: 121/1340 tests were not run due to test failure
```

The workspace total is already **1340** here, confirming the plan's arithmetic (1318 baseline + 22 new) before a single test was re-run.

### Step 2 — regeneration and the corpus diff

```
$ cargo run --locked -p md-cli --bin md -- vectors --out crates/md-codec/tests/vectors
$ git status --short crates/md-codec/tests/vectors | wc -l          → 30
$ git status --short crates/md-codec/tests/vectors | grep -c '^??'  → 30
$ git status --short crates/md-codec/tests/vectors | grep -v '^??' | wc -l → 0
```

**Exactly 30 new files, ZERO modified pre-existing files** — the plan's Expected outcome, and the check that mattered (a changed pre-existing vector would have meant the six additions altered something upstream of them).

**Fork-side numbers in the plan's hand-off note, independently machine-checked here** (they are the S3 plan's inputs, so being wrong would be expensive later):

```
$ ls crates/md-codec/tests/vectors | grep -cE '^(keyed_)?compose_'                         → 156
$ ls … | grep -E '^keyed_compose_' | sed 's/\..*//' | sort -u | wc -l                      → 28
$ ls … | grep -E '^compose_'       | sed 's/\..*//' | sort -u | wc -l                      → 4
```

**156 vendored files and 32 vector names (28 keyed + 4 unkeyed) — both of the plan's stated fork-side targets (126 → 156, 26 → 32) confirmed against the real regenerated corpus.**

### Step 1, re-run after regeneration — the real gate

```
$ cargo fmt --all --check                                     → FMT_CLEAN
$ cargo nextest run --locked --workspace --all-features
     Summary [  34.122s] 1340 tests run: 1340 passed, 3 skipped        (exit 0)
     (zero FAIL lines)
$ cargo test --locked --workspace --all-features --doc
     test result: ok. 0 passed; 0 failed; …                   (exit 0)
```

**Whole-workspace test count: 1340 passed / 3 skipped — identical to the plan's Expected 1340, no deviation.** Doctests 0, as the plan states this crate ships none. The plan's R0 tests-lens N-2 caveat (a spurious `display_grouping_conformance::conformance_vectors_pass` red in an under-populated scratch copy) **did not reproduce**, as predicted — this is a real worktree, not a gate scratch copy.

### EXTRA CHECK beyond the plan: CI's actual runner, not nextest

The plan's Step 1 gate uses `cargo nextest`, but the branch-protection context is `cargo test (ubuntu-latest)`, and `.github/workflows/ci.yml:48` runs `cargo test --workspace --all-targets --all-features`. Nextest's process-per-test isolation has previously hidden a shared-state bug in this constellation that CI's threaded `cargo test` exposed, so I ran CI's exact command too:

```
$ cargo test --workspace --all-targets --all-features
    cargo_test_exit=0
    (sum of all `test result: ok. N passed` lines)  TOTAL PASSED: 1340
    zero FAILED, zero `error`
```

**No nextest-vs-`cargo test` divergence: 1340 either way, both exit 0.**

### Step 3 — CHANGELOG and commit

Both bullets inserted verbatim from the plan's blocks: the md-cli bullet directly after the existing `md compose` bullet under `## md-cli [Unreleased]` → `### Added`, and the md-codec bullet directly after the existing "28 tagged compose vectors" bullet under `## md-codec [Unreleased]` → `### Added`. Both placements re-read after insertion.

The md-codec bullet's counting claim was machine-checked rather than trusted:

```
$ grep -cE '^        \("(keyed_)?compose_' crates/md-codec/tests/compose_support.rs   → 34
$ grep -cE '^    Vector \{ name: "(keyed_)?compose_' crates/md-codec/src/test_vectors.rs → 32
```

**"the family and the MANIFEST grow to 34 tagged / 32 in MANIFEST" is exactly right.**

Committed with the plan's message; 31 files, 1167 insertions. **Working tree clean afterwards.**

---

## Live spot-check of the shipped surface (beyond the plan's steps)

```
$ ./target/debug/md compose --wrapper tr --preset kofn-recovery,2of3,older=26280
tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*),and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(26280))})
exit=0
```

Byte-identical to `keyed_compose_preset_kofn_recovery`'s `MANIFEST` template. `md compose --help` now shows the group in its usage line — `Usage: md compose [OPTIONS] --wrapper <WRAPPER> <--path <PATH>|--preset <PRESET>>` — and the `--preset` help text carries the R0 fidelity M-1 `older1`/`older2` sentence.

---

## Anything I had to decide, could not do, or stopped on

**Nothing.** No step was blocked, no fragment failed to apply, no plan text turned out to be wrong about the tree. Every "Expected" line was met except the two recorded above (Task 2 Step 2's two vacuously-passing tests; Task 3 Step 1's ordering), neither of which required a design call.

Three process observations for the whole-diff reviewer, in descending order of usefulness:

1. **Two of the 21 new CLI tests pass in both worlds** (Task 2 Step 2, above). They are not false PASSes of the *feature* — the feature is covered by the other 19 plus the in-crate unit test — but they carry no fail-then-pass evidence of their own, and `path_json_names_no_preset` in particular would keep passing if the `"preset"` JSON field were deleted entirely. A stronger form would assert the key is *present and null* (e.g. `v.as_object().unwrap().contains_key("preset")`). Filed here as an observation; **not changed**, because changing a plan-specified test is authorship the plan did not grant.
2. **`sed -i 'Nr FILE'` silently no-ops when `FILE` does not exist** — GNU sed's `r` does not error, and the command still exits 0. My first CHANGELOG insertion attempt hit this (a missing `/` in a path) and reported success while changing nothing; it was caught only because I diffed the result instead of trusting the exit code. Everything in this report that claims a file changed was verified by reading the file back or by `git diff`, never by exit status alone.
3. **Every count in this report is a command's output**, never a hand count: `52`, `519`, `761`, `783`, `31`, `1340`, `30`, `156`, `28`, `4`, `34`, `32`, `21`, `503`.

**Not done here, by design (the plan's Task 3 Step 4 hand-off):** the whole-diff independent execution review (`composer-S0b-exec-review-r0.md`), and the merge to `main` via the `ci/staging` ritual. The branch is left un-pushed and un-merged, exactly as briefed. The fork (`seedhammer`) was not touched; the S3-plan corrections the plan's hand-off names (A10's `126` → `156`, its 26-name list → 32, its wrapper-loop narrowing for `kofn_recovery`'s `Tr` vector, and its wrong `preset_*` prefix) remain the controller's to schedule.
