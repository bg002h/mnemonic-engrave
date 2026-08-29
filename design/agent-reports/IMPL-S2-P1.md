# IMPL-S2-P1 — implementer report, P1 of `IMPLEMENTATION_PLAN_descriptor_input_S2.md`

Worktree `/scratch/code/shibboleth/me-worktrees/impl-descriptor-s2`, branch
`impl/descriptor-s2`, base `0144f02`. Five commits, `b8f0538..6efd7b5`. Nothing
pushed; the main checkout and the fork were not touched.

**P1 GATE: GREEN.** 575 tests run, 575 passed, 1 skipped (the pre-existing
deliberate regenerator ignore at `crates/me-cli/src/sysw/vectors.rs:132`);
`cross_lang` RAN; `lint-gate.sh` PASS on all three legs; the matrix witness
green at every one of the five commit boundaries; zero new `#[ignore]`.
Invariant 1 holds: `crates/me-cli/testdata/descriptor_seam_vectors.json` is
byte-unchanged from base (`sha256 542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974`,
the pinned value).

---

## What landed, per task

| task | commit | what |
| --- | --- | --- |
| P1.0 | `b8f0538` | the pack-path restructure + the consult-first negative sweep + its capture fixture |
| P1.1 | `282a071` | the `Descriptor` arm in `sysw::classify`, delegating to `descriptor::host_admits` |
| P1.2 | `928d167` | the derived classification rule, exhaustive over the vector file, both bases |
| P1.3 | `5d027e5` | `Kind::Descriptor` widens to two carriers; description, module doc, `card_hrp` doc |
| — | `6efd7b5` | the text P1.1's arm falsified elsewhere (see DEVIATIONS 3) |

### P1.0 — `b8f0538`

`crates/me-cli/src/main.rs`: `descriptor::consult` moves out of the
`admit_check` error branch and runs immediately before `admit_check`, on the
`--as`-omitted path, after `--expect` resolution — whose position did not move.
The four `Outcome` arms are unchanged; `RecordRefusal` now falls through to a
plain `admit_check`, which reports the shipped record refusal.

New: `crates/me-cli/tests/record_corpus.rs` (6 tests) and
`crates/me-cli/testdata/record_corpus_pre_s2.json` (33 records).

**The corpus**, enumerated from the data files the shipped classify tests
already use, plus the literals no data file in this crate carries, each cited
at its entry: `testdata/sysw_vectors.json` records (11 unique),
`testdata/codex32_seam_vectors.json` strings (8), and 14 literals from
`src/sysw/mt.rs:256` (the six-chunk `mt1` even set),
`tests/argv_secret_guard.rs:55,59` (`pass:`/`tx:`) and
`tests/ms_remedy_runs.rs:162` (the eight pack-path shapes). Class census of the
capture: `Unknown` 12, `MdMk` 8, `Mt` 6, `FreeText` 2, `Codex32Secret` 2,
`Mnemonic` 1, `Tx` 1, `Passphrase` 1 — every class invariant 2 names, with
`Unknown` densest because that is the class the arm takes records FROM.

**The consult-first sweep answers cleanly: 33 of 33 records are
`record-refusal`, at record scope AND at document scope** (the 8 sysw vectors'
records joined on LF, because `gate_opens` applies T1–T3 per line and T4 to the
whole input, so a document is not the sum of its records). Zero gate/classify
collisions, measured rather than assumed — which is the premise that makes P1.0
behaviour-preserving pre-arm.

**The whole existing suite stayed green with no test-expectation changed in
this commit**: 562 → 568 tests, all passing.

### P1.1 — `282a071`

`crates/me-cli/src/sysw/mod.rs:265`, LAST arm, in the `Err(_)` leg of the
seal-record match:

```
Err(_) if crate::descriptor::host_admits(record) => Class::Descriptor,
Err(_) => Class::Unknown,
```

Implemented ONCE, by delegating to the shipped predicate
(`crates/me-cli/src/descriptor/admit.rs:417`) rather than restating §5.2. The
position makes invariant 2 structural: only records that already fell through
to `Unknown` can move.

Also corrected in the same commit: `classify`'s doc claimed *"Descriptor and
Address are deliberately absent"*.

### P1.2 — `928d167`

`crates/me-cli/tests/descriptor_seam.rs`, two derived-rule tests plus two
named ones from P1.1:

* `every_single_line_input_classifies_by_the_admission_column` — for each of the
  **58** single-line rows, `classify(input) == Descriptor` iff `host_admits`,
  `== Unknown` otherwise, EXACT equality; asserted counts 15 admitted / 43 not,
  so the rule is satisfiable in both directions and cannot go one-sided;
* `every_admitted_rows_canonical_classifies_as_a_descriptor_record` — for each
  of the **19** `host_admits: true` rows, `classify(canonical) == Descriptor`,
  with `checked == POP.host_admits_true` so the loop cannot iterate zero rows;
* `a_canonical_descriptor_classifies_as_a_descriptor_record` and
  `a_multi_policy_the_cascade_parses_is_not_a_descriptor_record` (the latter
  pins `format == "bip380"` and `host_admits == false` on `neither/wsh-multi`,
  so the test states WHY the predicate is not "the cascade parsed it").

Two new consts, `SINGLE_LINE_ROWS = 58` and `SINGLE_LINE_ADMITTED = 15`,
measured from the file (`python3` over `vectors`), not read off it — and both
are asserted by the test that uses them. They are deliberately NOT fields of
`Pop`, which P2.6 regenerates from `gen.py`.

### P1.3 — `5d027e5`

`crates/me-cli/src/sysw/expect.rs:112` widens:

```
Kind::Descriptor => {
    card_hrp(record) == Some('d')
        || super::classify_with(record, adm) == Class::Descriptor
}
```

`describes()` → `"an md1 descriptor card, or a descriptor record (\`--as descriptor\`)"`.
The module doc's resolution table, its "must not resolve through `Class`"
paragraph and its "`Class::Address` and `Class::Descriptor` are never produced
by `classify`" claim are corrected, as is `record::card_hrp`'s parallel doc.
`check()`'s completeness walk is unchanged and a comment now says why: it names
only `Class::MdMk` records, so §5.2's record cannot appear in it — correctly,
because that record is not chunked and presence IS completeness for it.

Three tests in `crates/me-cli/tests/expect_kinds.rs`; the third named test in
the brief is covered at a different layer — see DEVIATIONS 2.

---

## The classify-cost measurement

Harness: a scratch integration test (not committed), best of five runs of 2000
reps per population, `std::hint::black_box` on input and output, test profile
(`opt-level = 2`, `debug_assertions` ON — no `--release`). Pre-arm numbers taken
at `b8f0538`, post-arm at `282a071`.

| population | pre-arm | post-arm | delta |
| --- | --- | --- | --- |
| the 33-record corpus | 615.6 ns/record | 722.6 ns/record | +107 ns (+17%) |
| `Unknown`-only subset (n=12) | 364.2 ns/record | 573.4 ns/record | +209 ns (+57%) |
| the 58 single-line seam inputs | 675.8 ns/record | 23 055 ns/record | ×34 |

Post-arm re-runs for stability: corpus 694.2 / 811.9 ns, Unknown-only 552.2 /
553.5 ns, seam 23 699 / 25 205 ns.

Reading it: **only the third row is the arm doing real work.** Those 58 inputs
are actual descriptors, and 23 µs is what it costs to run §4's cascade plus
secp point validation on a string that previously failed a bech32 check and
stopped. The record corpus — the population invariant 2 protects, and the shape
of a real payload — pays +107 ns/record; a 20-record payload pays about 2 µs
in total. Nothing that used to be cheap became expensive. The plan's bound
("the cascade on a non-descriptor record fails at the gate cheaply") holds:
the `Unknown`-only delta of +209 ns is the gate rejecting a non-descriptor, not
a parse.

---

## Mutation evidence — every new gate was shown to fail

Not "the tests pass". Each of these was applied, measured, and reverted.

1. **Hoisting `consult` above `--expect`** (the r2 N2 hazard):
   `expect_resolves_before_the_descriptor_gate` goes red, `left: Some(2)`,
   `right: Some(4)` — the choice block overtaking a stated expectation.
2. **Removing the `r#as.is_none()` guard**: 6 red in the first 23 of
   `descriptor_as` + `descriptor_refusals`, including
   `item_2_every_format_packs_reads_back_and_derives_the_device_address` and
   `as_md1_is_a_flag_the_binary_knows` — the flag path answering "`--as`
   decides" to an operator who just decided.
3. **Corrupting one `class` cell in the capture** (`Mt` → `Unknown`):
   `every_corpus_record_classifies_as_it_did_before_s2` goes red. The capture is
   asserted, not decorative.
4. **Re-keying the arm on the cascade parse** (`format_of(record) != "none"`
   instead of `host_admits`): `every_single_line_input_classifies_by_the_admission_column`
   goes red at `narrowed/tr-sortedmulti`, and
   `a_multi_policy_the_cascade_parses_is_not_a_descriptor_record` with it.
5. **P1.0 is load-bearing, measured under the arm**: with `282a071`'s arm
   present and `main.rs` reverted to `b8f0538^`,
   `item_5_the_five_case_matrix` AND `row_as_omitted` both go red — exactly the
   collapse R0 r1 recorded as C1.
6. **P1.3's widening**: `expect_descriptor_is_satisfied_by_a_descriptor_record`
   was red before the widening and green after; the other two P1.3 tests were
   green either way, by design (they pin what must NOT move).

---

## Test counts

| point | tests run | passed | skipped |
| --- | --- | --- | --- |
| base `0144f02` | 562 | 562 | 1 |
| P1.0 `b8f0538` | 568 | 568 | 1 |
| P1.1 `282a071` | 570 | 570 | 1 |
| P1.2 `928d167` | 572 | 572 | 1 |
| P1.3 `5d027e5` | 575 | 575 | 1 |
| HEAD `6efd7b5` | 575 | 575 | 1 |

+13 tests. The one skip is the pre-existing regenerator at
`crates/me-cli/src/sysw/vectors.rs:132`, untouched.

Matrix witness at every commit boundary, run per-commit on a detached checkout:

```
b8f0538  Summary [0.008s] 1 test run: 1 passed, 568 skipped
282a071  Summary [0.036s] 1 test run: 1 passed, 570 skipped
928d167  Summary [0.009s] 1 test run: 1 passed, 572 skipped
5d027e5  Summary [0.008s] 1 test run: 1 passed, 575 skipped
6efd7b5  Summary [0.008s] 1 test run: 1 passed, 575 skipped
```

---

## DEVIATIONS from the plan's text

**1. P1.2's classify-unchanged sweep landed at P1.0, not P1.2 — deliberate,
and strictly stronger.** The plan requires the capture to be *generated* at
P1.0 so its `class` column is pre-S2; it schedules the *assertion* at P1.2. I
committed both at P1.0. Reason: with the assertion parked at P1.2, the arm
commit (P1.1) would have had no guard on invariant 2 at its own boundary. As
landed, P1.1 could not have moved a corpus record's class without going red in
its own gate. The capture's provenance is unaffected — it was generated and
committed before `282a071` existed.

**2. P1.3's first named test is asserted at a different invocation than the
brief spells, because the brief's spelling is unsatisfiable in this build.**
The brief names `--as descriptor --expect descriptor` exits 0. `--as
descriptor` cannot exit 0 at P1: `DESCRIPTOR_PATH_SHIPPED` is `false` until
P2.1, so `descriptor_follower` returns the `WindowNotInBuild` refusal at exit
3. Pinning that invocation now would mean asserting the parked exit 3, which is
the opposite of the ruling. **The same defect is reachable today** by the route
that made the ruling necessary — `--expect` resolves before §5.1's gate, so
`me sysw pack --in <canonical descriptor> --expect descriptor` ran the kind
test over the descriptor record itself and, before the widening, reported it
ABSENT at exit 4. That is what
`expect_descriptor_is_satisfied_by_a_descriptor_record` asserts: no "was not
met", and exit 2 with the choice block, because `--expect` is met and the run
proceeds to §5.1. **P2.1 owns the exit-0 CLI form**; it is a one-line addition
to this test file once the flag packs, and it should be added there. The other
two named tests are exactly as briefed.

**3. A fifth commit, `6efd7b5`, was not in the brief.** P1.1's arm falsified
text elsewhere in the tree: `sysw_error`'s operator-facing `Unrecognised`
message said *"Descriptors and addresses are not yet classifiable here"*, and
`UnknownReason::Unrecognised`'s doc said the same. Both are now true and
narrower — an inadmissible descriptor DOES still land there, because the arm
places only what §4.7 admits. `tests/sysw_cli.rs:457`'s negative assertion
tracked the old wording and would have gone vacuously true, so it moved with
it. Committed separately so the response to my own diff is reviewable apart
from the diff.

**No other deviation.** The arm is `host_admits` verbatim, not reimplemented;
arm order is last; the vector file is byte-unchanged; `--expect` resolution did
not move; `Kind::Descriptor`'s vocabulary gained no word.

---

## For the controller — two things P1 found that P1 does not own

**A. A gap in P0.1's SPEC-FALSIFICATION enumeration, which the plan calls "the
COMPLETE enumeration".** `design/SPEC_descriptor_input.md:101` (§2.1, "The host
refuses every descriptor form there is") quotes the `Unrecognised` message
verbatim inside a transcript, and it is not listed as a falsification member.
It is a *historical* transcript section and it was **already stale before S2** —
that invocation has exited 2 with the choice block since S1 shipped, not 4 — so
it is not P1's diff to correct, and I did not touch the spec. `6efd7b5` changes
the code half of that quote, so the pair now differs. **P2.7 should carry it**,
and the sweep term that reaches it is `not yet classifiable`, which is not among
P2.7's listed terms.

**B. P2.1 inherits one named test.** See DEVIATIONS 2: `--as descriptor
--expect descriptor` exits 0, added to
`crates/me-cli/tests/expect_kinds.rs` beside the three that landed here. It is
the funds-path invocation the P1.3 ruling was written for, and P1 cannot run it.

---

## Gate output tails

```
$ ME_REQUIRE_GO=1 PATH=/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH \
  cargo nextest run --locked
        PASS [  32.010s] (575/575) mnemonic-io-lib::fish_history_purge history_delete_prefix_purges_nothing_however_it_fails
────────────
     Summary [  32.192s] 575 tests run: 575 passed, 1 skipped
exit=0

  (cross_lang RAN, not skipped:)
        PASS [   0.277s] (569/575) mnemonic-engrave::cross_lang rust_ndef_parses_in_seedhammer_go_reader
        PASS [   0.155s] (567/575) mnemonic-engrave::preview_cross_lang real_sidecar_renders_public_plates_only

$ ./scripts/lint-gate.sh
== cargo fmt --check
== clippy (CI-pinned 1.85.0)
== clippy (nightly)
lint-gate: PASS
exit=0

$ grep -rn '#\[ignore' crates/
crates/me-cli/src/sysw/vectors.rs:132:    #[ignore = "regenerates the fixture; run deliberately"]

$ git diff --stat 0144f02..HEAD -- crates/me-cli/testdata/descriptor_seam_vectors.json
  (empty)
$ sha256sum crates/me-cli/testdata/descriptor_seam_vectors.json
542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974
```

Whole-branch diffstat:

```
 crates/me-cli/src/main.rs                        |  61 +++--
 crates/me-cli/src/sysw/expect.rs                 |  44 ++--
 crates/me-cli/src/sysw/mod.rs                    |  25 ++-
 crates/me-cli/src/sysw/record.rs                 |  19 +-
 crates/me-cli/testdata/record_corpus_pre_s2.json | 211 ++++++++++++++++++
 crates/me-cli/tests/descriptor_seam.rs           | 124 +++++++++++
 crates/me-cli/tests/expect_kinds.rs              |  78 +++++++
 crates/me-cli/tests/record_corpus.rs             | 271 +++++++++++++++++++++++
 crates/me-cli/tests/sysw_cli.rs                  |   2 +-
 9 files changed, 785 insertions(+), 50 deletions(-)
```

## One environment note

The worktree's `third_party/seedhammer` submodule was uninitialised, so
`cross_lang` failed at base for a reason unrelated to any code
(`reading ../../third_party/seedhammer/go.mod: no such file or directory`).
`git submodule update --init third_party/seedhammer` (→ `713aee2`, `v1.4.2`)
fixed it, and the base suite then ran 562/562 green. Any later worktree cut
from this repo needs the same step before `ME_REQUIRE_GO=1` means anything.
