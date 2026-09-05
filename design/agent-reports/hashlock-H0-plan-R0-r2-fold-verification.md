# Hashlock H0 plan — R0 round 2, fold verification

**Reviewer:** independent sonnet fold-verification reviewer (targeted, scoped to
the capture step and the gate claim).
**Artifact under review:** fold commit `64a6e0da4c8498d0c70abb3596717b108d77f813`
over `fdfb040`, responding to round 1's persisted report
`design/agent-reports/hashlock-H0-plan-R0-r1-fold-verification.md` (`97dab8c`:
8/8 C+I fixed, 1 new Important — the round-0 fold's whole-crate claim did not
reproduce because Task 1 Step 1's four seam rows break `tests/record_corpus.rs`'s
pre-S2 capture, a file Task 1 never mentioned).

**ONE QUESTION:** does the r2 fold fix the r1 Important — so that following
Task 1 exactly, in order, leaves the whole crate green except for the three
box-local `history_purge` failures — without a new contradiction or a false
claim?

**Answer: yes.** Task 1 (Steps 1, 1b, 3, 5, 7, plus Step 8's plain `cargo fmt`,
which the plan's own sequence runs before Step 9's `--check`) was applied to a
fresh detached worktree at the fold commit and executed end to end. The r1
Important is fixed: `record_corpus.rs` is 6/6 with 37 records. The whole crate
is `613 passed, 3 failed, 2 skipped` out of 616, all three failures
`history_purge` — reproduced identically on untouched `master`. Every number
the fold's STATUS line, Global Constraint, File Structure row, Step 1b, and
Step 9 claim reproduces exactly. One pre-existing (not r2-introduced) defect
was found by literal execution and is recorded below as out of this round's
scope.

**Closing counts: 0 Critical / 0 Important / 0 Minor beyond wording — GREEN**
(one out-of-scope observation recorded, not counted).

---

## Method

Detached worktree `/scratch/code/shibboleth/me-worktrees/h0-verify2` at
`64a6e0d` (own `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h0-verify2-target`),
never the controller's `me-worktrees/h0-gate2`. Applied Task 1 Steps 1, 1b, 3,
5, 7 verbatim (byte-for-byte from the plan's own code blocks), then Step 8's
`cargo fmt -p mnemonic-engrave` (the plan's own Step 9 `-- --check` depends on
it having run; the brief's item 1 names only the modification steps, and Step
8's first command is the one that actually reformats the pasted blocks — this
is applying Task 1 "exactly, in order," not a scope expansion). Worktree and
both target dirs removed at the end; nothing committed anywhere.

---

## 1. Verify (brief item 1) — record_corpus after Step 1b

```
$ cargo nextest run --locked -p mnemonic-engrave --test record_corpus
        PASS [0.003s] (1/6) record_corpus the_capture_is_the_whole_corpus
        PASS [0.003s] (2/6) record_corpus the_descriptor_gate_stays_shut_on_every_corpus_record
        PASS [0.003s] (3/6) record_corpus the_descriptor_gate_stays_shut_on_every_corpus_document
        PASS [0.003s] (4/6) record_corpus the_capture_covers_every_class_s2_must_not_move
        PASS [0.004s] (5/6) record_corpus every_corpus_record_classifies_as_it_did_before_s2
        PASS [0.004s] (6/6) record_corpus expect_resolves_before_the_descriptor_gate
     Summary [0.004s] 6 tests run: 6 passed, 0 skipped
```

`testdata/record_corpus_pre_s2.json` has 37 records (confirmed via `json.load`
+ `len()`), and the four new entries (`codex32_seam/preimage-plate-0x03`,
`codex32_seam/bip93-plain-payload-0x03`, `codex32_seam/bip93-share-payload-0x03`,
`codex32_seam/preimage-shape-entr-id`) sit at indices 19-22, directly after
`codex32_seam/bip93-bad-checksum` (index 18) and before the first literal —
exactly the seam file's row order.

**`class`/`consult` are what `sysw::classify` and `descriptor::consult` actually
answer, not asserted values.** `every_corpus_record_classifies_as_it_did_before_s2`
calls `class_name(mnemonic_engrave::sysw::classify(record))` against the
capture's `class` field for every one of the 37 records and PASSED;
`the_descriptor_gate_stays_shut_on_every_corpus_record` calls
`mnemonic_engrave::descriptor::consult(...).class()` against `consult` for the
same 37 and PASSED. Since both tests compare the JSON's stated value against a
live function call and both PASS, the four new rows' `"class": "Unknown"` /
`"consult": "record-refusal"` are proven correct by the test run itself, not
merely asserted by the plan.

## 2. Verify (brief item 2) — whole crate

```
$ cargo nextest run --locked -p mnemonic-engrave --no-fail-fast
     Summary [0.547s] 616 tests run: 613 passed, 3 failed, 2 skipped
        FAIL [0.004s] (438/616) history_purge editing_the_file_alone_is_the_trap_the_message_warns_about
        FAIL [0.004s] (441/616) history_purge the_emitted_zsh_recipe_actually_purges_the_entry
        FAIL [0.004s] (442/616) history_purge the_harness_records_history_at_all
```

Matches the claim under test exactly: `616 tests run: 613 passed, 3 failed, 2
skipped`, all 3 `history_purge`. Confirmed stable (reran after the `cargo fmt`
pass with the identical result). Confirmed on untouched `master` (`d389e84`,
clean tree apart from one unrelated untracked brief file) with a scoped run:
`cargo nextest run --locked -p mnemonic-engrave --test history_purge` → same
three tests FAIL, same root cause (`/usr/bin/zsh is required` panic) — per the
brief, not investigated further.

`cargo clippy --locked -p mnemonic-engrave --all-targets -- -D warnings` fails
on exactly the one lint the plan names: `manual implementation of
.is_multiple_of()` at `sysw/composer_records.rs:114` — untouched by Task 1,
matching the plan's claim "the local nightly's, green in CI at `917d4e3`."
`cargo fmt -p mnemonic-engrave -- --check` is clean after Step 8's plain
`cargo fmt` (which only rewrapped long lines in the pasted `Step 5`/`Step 7`
blocks — no logic change).

## 3. Verify (brief item 3) — the Step 1b argument, four clauses

| Clause | Verdict | Evidence |
| --- | --- | --- |
| "added, not moved" | **TRUE** | `the_capture_is_the_whole_corpus` does a positional `Vec<(String,String)>` equality between the capture and a freshly-enumerated, dedup-on-record corpus; it PASSED, which is only possible if the four new records are genuinely new (not colliding with, and not displacing, any existing entry) and sit at the exact position `corpus()` generates for them. Independently: none of the four new record strings match any pre-existing capture record (checked the four against the other 33 origins/records — no collision). |
| "all four Unknown on the host at 0.7" | **TRUE** | Same test run: `class_name(sysw::classify(record))` for all four new rows compared equal to the JSON's `"class": "Unknown"` and PASSED. `host_admits` is `false` for all four in `codex32_seam_vectors.json` (checked directly, below). |
| "the descriptor gate refuses each as a record" | **TRUE** | `the_descriptor_gate_stays_shut_on_every_corpus_record` calls `descriptor::consult(...).class()` for all four and compared equal to `"record-refusal"`; PASSED. |
| "no record that was placeable changes class" | **TRUE** | The same `every_corpus_record_classifies_as_it_did_before_s2` run covers all 37 records, including the 33 pre-existing ones; it PASSED, which is only possible if none of them moved class. |

`host_admits` direct check (`codex32_seam_vectors.json`, last 4 rows):
```
preimage-plate-0x03        host_admits=False device_admits=False
bip93-plain-payload-0x03   host_admits=False device_admits=True
bip93-share-payload-0x03   host_admits=False device_admits=True
preimage-shape-entr-id     host_admits=False device_admits=False
```

**Does invariant 2's doc comment allow this kind of change, or forbid growth?**
It requires an argument, not a fixed size: *"A change to that file is a change
to invariant 2 and has to be argued for in the diff, which is the point of
capturing it rather than recomputing it."* Nothing in the doc comment states
or implies the capture must stay at a fixed count — the file's own
non-vacuity check is a floor, `assert!(want.len() >= 30, ...)`, with no
ceiling. Step 1b's argument (four records added, none moved, no existing
record's class disturbed) is exactly the shape of argument the invariant asks
for, and the passing test suite is the mechanical proof that the argument's
factual claims hold. **The doc comment permits this change; it does not
forbid growth.**

## 4. Verify (brief item 4) — new contradictions, read as a hostile implementer

- **STATUS line.** Claims "fixed below in Task 1 Step 1b" (confirmed above)
  and "whole crate re-run with `--no-fail-fast` in a worktree with its own
  target dir, output in the r1 fold commit's message" — the commit message's
  own gate output (`6/6 PASS, 37 records`; `616 tests run: 613 passed, 3
  failed, 2 skipped`, the 3 `history_purge`) reproduces exactly, independently,
  above. No contradiction.
- **New Global Constraint** ("whole-crate numbers measured with
  `--no-fail-fast`, own `CARGO_TARGET_DIR`..."). Consistent with r1's own
  finding ("615/616" for a tree that was 610/616) and with the commit
  message's stated root cause (a shared target dir handing a run compiled
  test binaries from another tree). Not independently re-triggered (would
  require deliberately sharing a target dir with a stale worktree), but
  nothing in it contradicts anything reproduced this round. Wording only
  ("And touch a file restored from a backup, or cargo reuses the mutated
  build" is grammatically loose) — Nit, not Important.
- **File Structure row** for `record_corpus_pre_s2.json` ("append 4 entries...
  class `Unknown`, consult `record-refusal`") — matches exactly what landed
  in the file (§1). No contradiction.
- **Step 1b** — fully verified above (§1, §3).
- **Step 9's Expected and `git add`** — the Expected text's whole-crate claim
  reproduces exactly (§2); the `git add` file list (`codex32_seam_vectors.json`,
  `record_corpus_pre_s2.json`, `codex32_seam.rs`, `preimage_plate_is_not_a_seed.rs`,
  `seal/record.rs`, `sysw/mod.rs`, `main.rs`) covers every file Task 1's File
  Structure table names as Modify/Create — no file is edited but left
  unstaged.
- **Self-review's r1 paragraph** ("the round-0 fold's 'whole crate 615/616'
  was false... Step 1b now extends the capture...") — matches r1's report and
  this round's own findings; no contradiction.

**Out-of-scope observation (not counted toward this round's severity, found by
literally executing Task 1's own Step 7 code, unchanged by the r2 diff):** the
hardcoded operator-facing string in `sysw/mod.rs`'s `U::PreimagePlate` arm
reads `"...hashlock PREIMAGE plate (kind 0x03, id \`hash\`)..."` with a literal
`` `hash` ``. The `preimage_plate()` predicate that selects this arm keys only
on the ms-codec decode error (`ReservedPrefixViolation { got: 0x03 }`), not on
the record's actual codex32 id — so the `preimage-shape-entr-id` row (id
`entr`, not `hash`) also takes this arm and produces the same message,
misnaming its id. Reproduced directly:

```
$ echo "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9" | me sysw pack
me: record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03, id `hash`), not a seed record; ...
```

(the record's actual codex32 id is `entr`, not `hash`)

This is not the r1 Important, and Step 7 (where this string lives) is
untouched by the r2 diff under review — it was already present, reviewed, and
signed off in round 0 (fidelity I-3) and round 1 (§3, which tested only the
canonical `id hash` plate through this same code path and so did not exercise
this row). Per this round's brief and severity rule, a defect neither
reopening the r1 Important nor introduced by this fold's diff does not block
this round's GREEN. Recording it here for a future fold/round rather than
silently dropping it.

## Closing counts

- **Critical: 0.**
- **Important: 0.** The r1 Important is fixed and reproduces exactly; no new
  contradiction found in the r2 diff.
- **Minor: 0 beyond wording** (the Global Constraint's one awkward sentence).
- **Out-of-scope observation: 1** (the `id \`hash\`` mislabelling on the
  `preimage-shape-entr-id` row's operator-facing message — pre-existing since
  round 0, not part of this round's diff, does not gate this round).

**GREEN / NOT GREEN: GREEN.**
