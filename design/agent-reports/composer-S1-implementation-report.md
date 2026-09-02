# Composer Stage 1 — implementation report

**Implementer:** single agent, UC off, executing
`design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md` at mnemonic-engrave
`4a0fd02` (the plan file is byte-identical at `4a0fd02` and at the branch base
`59e6f12` — verified with `git diff 4a0fd02 HEAD -- <plan>`, empty).

**Outcome: all six implementer tasks (1–5, 7) executed and committed. Both
repos green. Task 6 skipped as instructed (already DONE by the controller).**

Nothing was left undone, and nothing required a decision the plan did not
already make. Three deviations from the plan's *Expected* lines are recorded
below; all three are wording slips in the plan, not defects in the code, and
none changed what was built.

---

## Worktrees and branches

| repo | worktree | branch | base |
| --- | --- | --- | --- |
| mnemonic-engrave | `/scratch/code/shibboleth/wt-composer-s1` | `composer-s1` | `59e6f12` |
| mnemonic-secret | `/scratch/code/shibboleth/wt-ms-bip48-p2tr` | `bip48-p2tr` | `5f37b43` (ms-cli 0.16.0, the plan's baseline) |

Every cargo command in the engrave worktree ran with `RUSTUP_TOOLCHAIN=1.85.0`
(no `rust-toolchain.toml`; CI pins it in `.github/workflows/release.yml`).
mnemonic-secret used its own `rust-toolchain.toml` (`channel = "1.85.0"`). Both
worktrees built in their own on-disk `target/` under `/scratch` (`/dev/nvme2n1p6`,
5.1T free — not tmpfs). Neither main checkout was touched; nothing was pushed,
tagged or published.

### `git log --oneline 59e6f12..HEAD` (composer-s1)

```
90560cb me: changelog for the composer's record classes; the three-prefix sentences corrected (N9, the Unrecognised message, pack --help, record.rs) (composer S1 task 7)
26e10e5 me sysw pack: --now/--no-now, the pack time appended when a key:/hash: record is present, show lines for key:/hash:/now: (composer S1 task 4)
b474a31 me sysw: record_class_vectors.json -- the composer classes' lockstep fixture, 40 rows (one per 6a rule), sha256-pinned (composer S1 task 3)
e27d659 me sysw: Class::{Key,Hash,Now}, classifier arms, 8n refusals, the single-now rule (composer S1 task 2)
d01e7a1 me sysw: composer_records -- key:/hash:/now: prefixes, body rules, 8n lines, constructors (composer S1 task 1)
```

### `git log --oneline 5f37b43..HEAD` (bip48-p2tr)

```
7f979e5 ms derive: --template bip48-p2tr (m/48'/coin'/account'/3'), stale taproot justifications rewritten, negative test flipped and renamed (composer S1 task 5)
```

Both trees are clean (`git status --short` empty in each). Every commit carries
the plan's message verbatim with both trailer lines, applied via `git commit -F`.

---

## Task 1 — `composer_records`: prefixes, parser, §8n lines

**Expected (Step 2):** FAIL to compile, `could not find composer_records in sysw`.
**Got — matches:**

```
error[E0432]: unresolved import `mnemonic_engrave::sysw::composer_records`
 --> crates/me-cli/tests/sysw_composer_records.rs:5:29
5 | use mnemonic_engrave::sysw::composer_records::{
  |                             ^^^^^^^^^^^^^^^^ could not find `composer_records` in `sysw`
```

**Expected (Step 4):** all PASS. **Got — matches, 9/9:**

```
     Summary [   0.016s] 9 tests run: 9 passed, 0 skipped
```

No fix to the code was needed on account of `bitcoin` 0.32 parsing behaviour:
the module compiled and every assertion held as written, including the `h`
hardened spelling, the empty-path case and the `DerivationPath`/`Fingerprint`
`from_str` semantics. `cargo fmt --all` reflowed the new files; clippy
`-D warnings` clean (exit 0).

**Deviation: none.**

---

## Task 2 — `Class::{Key,Hash,Now}`, classifier arms, refusals, single-`now:`

**Expected (Step 2):** FAIL to compile (`Class::Key`, `UnknownReason::Composer`,
`SyswError::SecondNow` do not exist). **Got — matches exactly, and nothing else:**

```
      3 error[E0599]: no variant or associated item named `Composer` found for enum `UnknownReason` in the current scope
      2 error[E0599]: no variant or associated item named `Hash` found for enum `Class` in the current scope
      2 error[E0599]: no variant or associated item named `Key` found for enum `Class` in the current scope
      2 error[E0599]: no variant or associated item named `Now` found for enum `Class` in the current scope
      2 error[E0599]: no variant or associated item named `SecondNow` found for enum `SyswError` in the current scope
```

**Step 3, exhaustive-match claim CHECKED, not assumed.** The plan states the
repo has exactly TWO exhaustive matches on `Class` (`main.rs`'s `class_name`
and `tests/record_corpus.rs`), and that `sysw/expect.rs` needs no change. After
adding the three arms to both, `cargo build --locked --all-targets` exited 0
with no error output — so no third site exists. The plan's claim is true.

**Expected (Step 4):** all PASS, no pre-existing test changes verdict.
**Got — matches:**

```
     Summary [   0.409s] 600 tests run: 600 passed, 1 skipped
```

(600, not 612, because `tests/sysw_composer_cli.rs` and the Task 3 fixture tests
do not exist yet at this commit. The 612 arrives at Task 4, matching the plan's
gate measurement exactly.) `record_corpus` tests ran (6 lines in the capture)
and passed — no record moved class. fmt + clippy clean.

**Deviation: none.**

---

## Task 3 — the lockstep fixture `record_class_vectors.json`

**Expected (Step 2):** FAIL to compile (`CASES`, `fixture_rows`, `FixtureRow`
missing). **Got — matches:**

```
error[E0432]: unresolved imports `mnemonic_engrave::sysw::composer_records::fixture_rows`, `mnemonic_engrave::sysw::composer_records::FixtureRow`, `mnemonic_engrave::sysw::composer_records::CASES`
```

The `CASES` table was **extracted programmatically from the plan file** (the
fenced ```rust block following "Add to
`crates/me-cli/src/sysw/composer_records.rs` (append at the end):", plan lines
853–944) and appended byte-for-byte — no hex body was retyped. The extractor
counted **40** `Case` rows, matching the plan's stated 40.

**Expected (Step 4, first run):** the two content tests PASS (40 rows), the
consumer test FAILS (no file). **Got — matches:**

```
    thread 'the_committed_fixture_is_what_the_table_generates_and_carries_the_pinned_digest' panicked at crates/me-cli/tests/sysw_composer_records.rs:440:47:
    testdata/record_class_vectors.json exists; run the regenerate test: Os { code: 2, kind: NotFound, message: "No such file or directory" }
     Summary [   0.034s] 17 tests run: 16 passed, 1 failed, 1 skipped
```

**The fixture was GENERATED by the regenerate test. Nothing was pasted by hand,
and nothing was re-pinned.** Its printed digest equals the plan's pinned value:

```
wrote 40 rows to /scratch/code/shibboleth/wt-composer-s1/crates/me-cli/testdata/record_class_vectors.json
sha256 a894e619580db8ca0e06ebfe45576cc45722f695913bf46e9285201c95f146c3
```

Independently re-measured with the system tool, after `cargo fmt` had reflowed
the source that generates it (the digest is a function of the row data, not the
formatting):

```
a894e619580db8ca0e06ebfe45576cc45722f695913bf46e9285201c95f146c3  crates/me-cli/testdata/record_class_vectors.json
```

Both equal the plan's `a894e619580db8ca0e06ebfe45576cc45722f695913bf46e9285201c95f146c3`.
Re-run after generation: `17 tests run: 17 passed, 1 skipped`. fmt + clippy clean.

**Deviation: none.**

---

## Task 4 — `--now`/`--no-now`, the auto-append, `me sysw show`

The CLI test file was likewise extracted verbatim from the plan (163 lines).

**Expected (Step 2), per the plan's own enumeration:** `no_now_suppresses…` and
`now_forces_…` FAIL (unknown flags); `pack_appends_…` FAILS; `a_payload_without_a_composer_record_packs_byte_identically_to_before`
PASSES already; the refusal tests PASS already; `show_prints_each_class_legibly`
FAILS.

**Got — 3 passed, 6 failed:**

```
PASS  two_operator_supplied_now_records_are_refused_naming_the_second
PASS  malformed_records_are_refused_with_the_8n_lines
PASS  a_payload_without_a_composer_record_packs_byte_identically_to_before
FAIL  no_now_suppresses_the_auto_append_so_a_fixture_is_a_pure_function_of_its_inputs
FAIL  now_forces_the_append_onto_any_payload_and_conflicts_with_no_now
FAIL  pack_appends_the_pack_time_when_a_composer_record_is_present_and_says_so
FAIL  an_operator_supplied_now_wins_silently_and_nothing_is_appended
FAIL  show_prints_each_class_legibly
FAIL  a_second_now_is_refused_before_the_passphrase_ceremony
```

and each failure had the mechanism the plan predicts — e.g.
`error: unexpected argument '--now' found` for the flag tests.

### DEVIATION 1 (plan wording, not a defect) — Task 4 Step 2

The plan's Expected line says, verbatim:

> the refusal tests PASS already (Task 2)

`a_second_now_is_refused_before_the_passphrase_ceremony` is a refusal test and
it **FAILED** at Step 2, with exactly the symptom the plan's own Step 3 comment
anticipates — the passphrase ceremony printed before the refusal:

```
    thread 'a_second_now_is_refused_before_the_passphrase_ceremony' panicked at crates/me-cli/tests/sysw_composer_cli.rs:135:5:
    the ceremony ran before the refusal: sealing:  SEALED — this payload holds secret material (record 0 (BIP-39 mnemonic)), so it is encrypted
    passphrase — write this down and store it APART from the machine:
        review fortune season negative turtle lamp garden inherit valley corn panic meadow
```

This is the failure the plan's hoisted pre-check exists to fix ("A first draft
put this beside the auto-append below, AFTER the ceremony; the plan's own test
caught it"). Nothing was changed in response — the plan's Step 3 already
prescribes the fix, and it works. **The Expected line is imprecise; the design
is not.** No action taken beyond recording it.

**Step 3** applied exactly the RULED default: append only when a `key:`/`hash:`
record is present and no `now:` is supplied; `--now` forces; `--no-now`
suppresses; `conflicts_with = "no_now"`. The second-`now:` pre-check was placed
**directly after the `sysw::admit_check` block and before `decide_sealing` /
the passphrase ceremony**, as instructed. `print_composer_confirmation` was
added as its own helper with its own call site after
`print_descriptor_confirmation`, not inside `print_mt_confirmation`.

**Expected (Step 4):** all PASS under both runners with NO pre-existing test
changed. **Got — matches, and equals the plan's gate measurement exactly:**

```
     Summary [   0.341s] 612 tests run: 612 passed, 2 skipped
```

Threaded runner (`cargo test --locked -p mnemonic-engrave`): every binary
`test result: ok`, 0 failed.

**No pre-existing test was edited.** The seven tests the plan names as the ones
a wrong predicate would move all pass untouched — grepped from the same capture:

```
PASS ( 368/612) descriptor_as item_1_every_format_packs_one_descriptor_record
PASS ( 372/612) descriptor_as item_2_every_format_packs_reads_back_and_derives_the_device_address
PASS ( 474/612) sysw_cli a_payload_past_the_old_8191_cap_packs_and_reads_back
PASS ( 492/612) sysw_cli an_incomplete_set_still_packs_and_is_readable
PASS ( 505/612) sysw_cli a_secrets_only_payload_reports_no_digest
PASS ( 555/612) sysw_cli the_descriptor_show_block_leaves_every_other_container_byte_identical
PASS ( 579/612) sysw_cli show_reports_exactly_one_descriptor_record_for_each_of_the_four_formats
```

`tempfile` was already a dev-dependency (`crates/me-cli/Cargo.toml:82`), so the
plan's contingency did not apply. fmt + clippy clean; composer CLI file
`9 tests run: 9 passed`.

**Deviation: 1 (above), wording only.**

---

## Task 5 — `ms derive --template bip48-p2tr` (mnemonic-secret)

**Expected (Step 2):** the two new tests FAIL (`bip48-p2tr` is not a valid
value); every other test PASSES. **Got — matches:**

```
     Summary [   0.045s] 16 tests run: 14 passed, 2 failed, 0 skipped
        FAIL  bip48_p2tr_derives_the_composer_taproot_origin
        FAIL  bip48_p2tr_json_names_the_path_and_no_assumption

    assertion `left == right` failed: error: invalid value 'bip48-p2tr' for '--template <TEMPLATE>'
      [possible values: bip44, bip49, bip84, bip86, bip48-p2wsh, bip48-p2sh-p2wsh, bip48, bg002h-tr, bg002h-wsh]
```

All edits landed as the plan specifies: the module-doc sentences in
`cli_derive_bip48.rs`, the renamed positive test replacing
`an_unregistered_script_type_is_refused`, the type-level doc paragraph, the
`Bip48P2tr` variant, and arms in **all three** of `purpose` (`=> 48`),
`script_type` (`Some(3)`) and `script_type_label`, plus the `Bg002hTr` doc
sentence.

**After Step 3, both new tests pass against the plan's oracle xpubs unchanged
—** i.e. the two-implementation oracle (Go `hdkeychain`, Python BIP-32)
reproduces here:

```
     Summary [   0.038s] 16 tests run: 16 passed, 0 skipped
```

**Expected (Step 4):** all PASS; `the_single_sig_template_names_are_unchanged`,
`bg002h_templates_derive_the_ruled_path` and
`bg002h_wsh_is_not_labelled_as_nested_segwit` still PASS; no snapshot changes.
**Got — matches, and BETTER than the plan's scratch-copy 308/310:**

```
     Summary [   0.105s] 310 tests run: 310 passed, 11 skipped

PASS ( 47/310) cli_derive_bip48 bg002h_wsh_is_not_labelled_as_nested_segwit
PASS ( 56/310) cli_derive_bip48 bg002h_templates_derive_the_ruled_path
PASS ( 89/310) cli_derive_bip48 the_single_sig_template_names_are_unchanged
```

The plan predicted its two scratch-copy failures
(`format::conformance::conformance_vectors_pass`,
`the_display_grouping_conformance_pin_is_untouched`) were artefacts that "pass in
the repo itself" — **confirmed: they pass here.** `gen_man` and
`gui_schema_emits_spec_v7_json` all pass — no snapshot moved, as predicted.
Whole-workspace `cargo test --locked`: every binary `ok`, 0 failed.

### DEVIATION 2 (plan artifact, mechanical fix) — Task 5 Step 4

The plan's Step 4 runs `cargo fmt --all --check` and expects clean. **It was
not clean**: the plan's own test source for Task 5 is not rustfmt-formatted.

```
Diff in .../crates/ms-cli/tests/cli_derive_bip48.rs:187:
-    assert!(!err(&o).contains("ASSUMED"), "an explicit script type is a choice, not an assumption");
-    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip48-p2tr", "--account", "1"]);
+    assert!(
+        !err(&o).contains("ASSUMED"),
...
Diff in .../crates/ms-cli/tests/cli_derive_bip48.rs:196:
-    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip48-p2tr", "--json"]);
+    let o = ms(&[
...
fmt-exit=1
```

Only two `assert!`/`ms(&[…])` call sites in the new test are affected; my
`derive.rs` edits were already fmt-clean. **Action: ran `cargo fmt --all`
(mechanical reflow only — no assertion, constant or argument changed), then
`cargo fmt --all --check` → exit 0.** This is the same treatment the engrave
tasks give (they run `cargo fmt --all` before checking), and the repo's phase
gate requires a fmt-clean tree, so leaving it red was not an option. No design
decision was involved. Recording it because the plan's Expected line asserts
cleanliness that the plan's own text does not satisfy.

**Step 5:** CHANGELOG `## ms-cli [Unreleased]` added above `## ms-cli [0.16.0] —
2026-08-15`; the follow-up `ms-derive-taproot-justifications-stale` status line
changed to CLOSED **with today's date filled in as instructed**:

```
- **Status:** CLOSED 2026-09-02 by composer Stage 1 (bip48-p2tr added; both comments rewritten). **Tier:** was docs + feature.
```

and the "What to do" section carries the note that `bg002h-tr` KEEPS its
purpose (composer spec §4f rules `48'/…/3'` for seed-derived slots; nothing in
this stage removes `bg002h-tr`).

**Deviation: 1 (above), mechanical.**

---

## Task 6 — SKIPPED

Already DONE per the plan's own STATUS line (fold `12e0659`, R0 closed at
`fdf7671`). Nothing under `design/` was touched by the implementer except the
one edit Task 7 explicitly assigns (`SPEC_sh2_sysw_consumption.md` N9) and this
report.

---

## Task 7 — whole-repo gates and the me changelog

### Step 1 — the repo the way CI does (run on the FINAL tree, after Step 3's edits)

```
=== fmt ===
fmt-exit=0
=== clippy (--all-targets --locked -- -D warnings) ===
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.06s
clippy-exit=0
=== nextest (cargo nextest run --locked, whole workspace) ===
     Summary [  32.189s] 621 tests run: 621 passed, 2 skipped
=== cargo test --locked (threaded, as CI) ===
28 binaries: 621 passed, 0 failed, 2 ignored
```

Both runners agree at 621. The mnemonic-secret worktree's final gate:

```
fmt-exit=0
clippy-exit=0
     Summary [   0.125s] 477 tests run: 477 passed, 11 skipped
```

### Step 2 — the sysw vector fixture is byte-stable

```
$ git status --short crates/me-cli/testdata/
(empty)

$ git diff --stat 59e6f12..HEAD -- crates/me-cli/testdata/
 crates/me-cli/testdata/record_class_vectors.json | 242 +++++++++++++++++++++++
 1 file changed, 242 insertions(+)

$ cargo test --locked -p mnemonic-engrave --lib sysw::vectors
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 281 filtered out; finished in 0.04s
```

**`record_class_vectors.json` is the ONLY file changed under `testdata/` across
the whole branch — `sysw_vectors.json` is untouched**, confirming the library
appends nothing. Expected, matched.

### Step 3 — the four record edits and the two spec gates

All four applied: N9 in `design/SPEC_sh2_sysw_consumption.md`; the
`U::Unrecognised` message in `main.rs`; the `SyswCmd::Pack` `--help` paragraph
after the `tx:` one; the reserved-prefix doc sentence in `sysw/record.rs`.

**`scripts/plan-glyph-check.sh design/SPEC_sh2_sysw_consumption.md`:**

```
═══ design/SPEC_sh2_sysw_consumption.md
WOULD NOT DRAW  line 358: '…' in
                'This payload holds no transaction.\n\nIt holds: <n class, n class…>.'
WOULD NOT DRAW  line 492: '—…' in
                'and amounts (verified — it printed `bc1qc80qm4p…  0.05000000 BTC` during this'

─── operator strings scanned: 40 ; undrawable: 2
─── NOT covered: prose-embedded strings, line-fit, the Go source itself.
glyph-exit=0
```

**`scripts/plan-cite-check.sh design/SPEC_sh2_sysw_consumption.md`:**

```
DANGLING  record.rs:74-108                                      (no such file under any root)
─── citations resolved: 17 / 18 ; dangling: 1 ; ambiguous: 0
─── resolved against fork root: /scratch/code/shibboleth/seedhammer
cite-exit=0
```

### DEVIATION 3 (pre-existing, NOT introduced) — Task 7 Step 3 gate output

The plan's Task 6 Step 2 expects these gates "all clean". They exit 0 but print
2 undrawable glyphs and 1 dangling citation. **I verified mechanically that all
three are pre-existing and untouched by the N9 edit**, by running both gates on
the base version of the file (`git show 59e6f12:…` copied into the repo under a
scratch name, then deleted):

```
--- glyph on BASE ---
WOULD NOT DRAW  line 357: '…' in
WOULD NOT DRAW  line 491: '—…' in
─── operator strings scanned: 40 ; undrawable: 2
--- cite on BASE ---
DANGLING  record.rs:74-108                                      (no such file under any root)
─── citations resolved: 17 / 18 ; dangling: 1 ; ambiguous: 0
```

Identical findings, identical counts; the two glyph line numbers differ by
exactly +1 because the N9 sentence now wraps to three lines instead of two. **My
edit introduced no new gate finding, and I fixed none of the pre-existing ones**
— they are outside this task's four named edits, and the plan does not assign
them. Flagging them here rather than silently acting: the `record.rs:74-108`
dangling citation in `SPEC_sh2_sysw_consumption.md` is a real (pre-existing)
loose end for whoever owns that spec — the cite-checker resolves against the
fork root, where no `record.rs` exists, so the citation names no reachable file.

**Post-edit suite re-run** (the plan warns the message edits may touch test
expectations):

```
     Summary [   0.399s] 612 tests run: 612 passed, 2 skipped
```

**No test expectation needed changing** — no test asserted the old three-prefix
wording.

### Step 4 — changelog and commit

`## [Unreleased] → ### Added` in `crates/me-cli/CHANGELOG.md` gained the plan's
entry verbatim. Committed with the plan's message.

**Deviation: 1 (above), pre-existing gate noise, no action.**

---

## Summary of everything I decided, could not do, or stopped on

**I stopped on nothing. No task was left partial, and no plan step was skipped
other than Task 6, which was instructed.**

Three judgement calls, all mechanical, all recorded above:

1. **Task 5 fmt (DEVIATION 2).** Ran `cargo fmt --all` on the mnemonic-secret
   worktree because the plan's own test text is not rustfmt-clean and the plan's
   Step 4 gate demands `cargo fmt --all --check` pass. Reflow only.
2. **Task 4 Step 2 Expected line (DEVIATION 1).** One test the plan groups under
   "the refusal tests PASS already" in fact fails at Step 2 — by design, since
   Step 3 is what hoists the check ahead of the ceremony. No code change; the
   plan's prescription works as written.
3. **Task 7 gate noise (DEVIATION 3).** Two glyph findings and one dangling
   citation in `SPEC_sh2_sysw_consumption.md` are pre-existing; proved by running
   both gates against the base file. Left alone rather than opportunistically
   fixed.

Things I deliberately did **not** do, per the brief: no push, no tag, no
publish, no touching of either main checkout, no edits under
`/scratch/code/shibboleth/mnemonic-engrave` except this report, no `.jsonl` read,
no re-pinning of the fixture digest (it matched on the first generation), and no
editing of any pre-existing test.

**What remains for the controller** (plan Task 7 Step 5, explicitly not the
implementer's): the whole-diff independent execution review over each repo's
diff, persisted to `design/agent-reports/composer-S1-exec-review-r0.md`, to 0C/0I;
then the releases; then Stage 2, which vendors
`record_class_vectors.json` at sha256
`a894e619580db8ca0e06ebfe45576cc45722f695913bf46e9285201c95f146c3`.
