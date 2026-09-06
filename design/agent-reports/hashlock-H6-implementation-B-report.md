# H6 implementer B — Tasks 2 then 3 (`me`: the `phrase:` record, the two classes, `--pack-preimage`)

**Branch** `h6-b`, worktree `/scratch/code/shibboleth/me-worktrees/h6-b`, off engrave `master`
`f82b8c11` (which carries the plan's stated baseline `aec48835` as an ancestor — verified with
`git merge-base --is-ancestor`; `f82b8c11` is the controller's own continuity commit, added
between the brief being written and the worktree being created).

| task | commit |
| --- | --- |
| Task 2 — the `phrase:` record and the two classes | `4d00fbbf2b4012a533d060161bef72e3c7b547c1` |
| Task 3 — `--pack-preimage`, the refusals and the four warnings | `8cf2a7f98e5f5bfa535200b260a514cdae33f087` |

**Branch tip:** `8cf2a7f98e5f5bfa535200b260a514cdae33f087`. Nothing pushed. Working tree clean.

**Precondition, checked before starting:** `cargo info ms-codec` reported `version: 0.8.0 (latest
0.9.0)` — 0.9.0 published. `cargo update -p ms-codec --precise 0.9.0` moved `Cargo.lock` by exactly
one package (`git diff Cargo.lock` is the `ms-codec` stanza and nothing else), and
`cargo fetch --locked` resolved it. The 0.9.0 source carries every symbol Tasks 2 and 3 consume —
`validate_phrase`, `preimage_hardened`, `preimage_sha256`, `digest`,
`HASHLOCK_PHRASE_MAX_CHARS`, `PhraseRefusal`, `looks_like_ms1`, `qr_text` — grepped out of
`~/.cargo/registry/src/index.crates.io-*/ms-codec-0.9.0/src/hashlock.rs`. **Neither of my tasks
calls `qr_text`**, so the post-GREEN `Zeroizing<String>` change the dispatch flagged does not reach
this branch (grepped: zero hits in `crates/me-cli/`).

**Files touched — exactly the twelve the brief lists, and no others** (`git diff --stat
f82b8c11..HEAD`):

```
 Cargo.lock                                       |   4 +-
 crates/me-cli/Cargo.toml                         |   2 +-
 crates/me-cli/src/main.rs                        | 337 ++++++++++++++++-
 crates/me-cli/src/seal/record.rs                 |  46 +++
 crates/me-cli/src/sysw/composer_records.rs       | 130 ++++++-
 crates/me-cli/src/sysw/mod.rs                    |  94 ++++-
 crates/me-cli/src/sysw/record.rs                 |  20 +-
 crates/me-cli/testdata/record_class_vectors.json | 126 +++++++
 crates/me-cli/testdata/record_corpus_pre_s2.json |   2 +-
 crates/me-cli/tests/record_corpus.rs             |  15 +
 crates/me-cli/tests/sysw_composer_records.rs     |   2 +-
 crates/me-cli/tests/sysw_pack_preimage.rs        | 448 +++++++++++++++++++++++
 12 files changed, 1197 insertions(+), 29 deletions(-)
```

**Independent cross-check against the plan author's gate tree.** All ten source/test/data files
are **byte-identical** to `/scratch/code/shibboleth/.tmp/h6-me/`'s. The only two deltas are
`crates/me-cli/Cargo.toml` (`ms-codec = "0.9"` here, `"0.8"` there) and `Cargo.lock` — which is
exactly right: Task 2 Step 0's real bump is what replaces the scratch tree's
`[patch.crates-io]` device, and that device was never to be committed.

---

## Task 2 — commit `4d00fbbf`

### RED (quoted)

The 21 `CASES` rows and the one moved `record_corpus_pre_s2.json` row were applied FIRST, before
any implementation. `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast -E
'binary(sysw_composer_records) + binary(record_corpus)'`:

```
Summary [   0.019s] 24 tests run: 20 passed, 4 failed, 1 skipped
  FAIL mnemonic-engrave::record_corpus the_capture_covers_every_class_s2_must_not_move
  FAIL mnemonic-engrave::record_corpus every_corpus_record_classifies_as_it_did_before_s2
  FAIL mnemonic-engrave::sysw_composer_records every_case_classifies_as_its_row_says_and_refuses_with_its_line
  FAIL mnemonic-engrave::sysw_composer_records the_committed_fixture_is_what_the_table_generates_and_carries_the_pinned_digest
```

```
panicked at crates/me-cli/tests/sysw_composer_records.rs:410:9:
assertion `left == right` failed: phrase-hardened: phrase:68617264656e65642c636f727265637420686f727365206261747465727920737461706c65
  left: "Unknown"
 right: "Phrase"

panicked at crates/me-cli/tests/record_corpus.rs:147:9:
assertion `left == right` failed: codex32_seam/preimage-plate-0x03: class moved under S2
  left: "Unknown"
 right: "Preimage"

panicked at crates/me-cli/tests/record_corpus.rs:224:22:
testdata/record_corpus_pre_s2.json: unexpected class Preimage
```

### Step 6, RUN and independently re-derived

```
wrote 68 rows to /scratch/code/shibboleth/me-worktrees/h6-b/crates/me-cli/testdata/record_class_vectors.json
sha256 3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf
```

That is **the plan's own quoted RUN output, character for character** — 68 rows and the same
digest. Re-derived from the committed file rather than trusted from the test:
`sha256sum crates/me-cli/testdata/record_class_vectors.json` →
`3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf`, and
`json.load(...)` counts **68** rows. `FIXTURE_SHA256` re-pinned to that value.

### Task 2 boundary gate

- `cargo fmt --all -- --check` — **exit 0, clean**.
- `cargo clippy --locked --all-targets -p mnemonic-engrave` — **exit 0**, one warning, the
  pre-existing `manual implementation of .is_multiple_of()`.
- `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` —
  **621 tests run: 616 passed, 5 failed, 2 skipped.**

### DEVIATION 2-A — Task 2 cannot build without a block the plan schedules in Task 3

The plan's File Structure table assigns `print_composer_confirmation`'s `ComposerRecord::Phrase(_)`
arm to **Task 3 Step 8**. But Task 2 Step 1 adds the `ComposerRecord::Phrase` variant, and that
match is exhaustive:

```
error[E0004]: non-exhaustive patterns: `ComposerRecord::Phrase(_)` not covered
    --> crates/me-cli/src/main.rs:2215:15
error: could not compile `mnemonic-engrave` (bin "me") due to 1 previous error
```

`cargo test --test sysw_composer_records regenerate` could not even run. **Minimal correct thing:**
the plan's own Task 3 Step 8 block was applied verbatim in Task 2, and Task 3 does not re-add it.
Recorded in the Task 2 commit message.

### DEVIATION 2-B — the plan's Task 2 boundary number is the Task-2+3 tree's

The plan states `621 tests run: 618 passed, 3 failed` (the box-local `history_purge` trio) for
Task 2. **Measured: 5 failed.** The trio, plus two SHIPPED tests that Task 2 falsifies and only
Task 3 repairs:

| test | why it is red after Task 2 alone |
| --- | --- |
| `sysw::tests::a_preimage_plate_is_named_not_misdiagnosed` | Task 3 Step 9 rewrites it — the plan says so outright, but does not carry the consequence into Task 2's gate number |
| `preimage_plate_is_not_a_seed::sysw_pack_names_a_preimage_plate_and_never_echoes_it` (`:61`, *"sysw pack accepted a preimage plate"*) | **Named by no task at all.** Task 2 gives the plate a class; `admit_check`'s second refusal rule (Task 3 Step 2) is what refuses it again |

Evidence it is transient rather than a defect: the plan author's fully-applied gate tree leaves
`crates/me-cli/tests/preimage_plate_is_not_a_seed.rs` **untouched** (`diff -q` is silent), i.e. the
final tree passes it unmodified. **Both are PASS in the Task 3 gate run below.**

This is the `a-gate-that-wires-all-tasks-at-once-cannot-see-order` class exactly: the gate tree
had every task applied simultaneously, so no per-task boundary was ever built, and both per-task
gate figures in the plan are the final tree's.

### Task 2 mutations

| mutation | result |
| --- | --- |
| **cut on the LAST comma** (`split_once` → `rsplit_once`) | **RED.** `every_case_classifies_as_its_row_says_and_refuses_with_its_line`: `phrase-containing-a-comma: phrase:68617264656e65642c6f6e652c2074776f2c207468726565 / left: "Unknown" / right: "Phrase"`. Reverted. **The plan predicted the wrong catcher**: it says *"`ComposerRecord::Phrase.phrase` is `" three"` … and `sysw_composer_records`'s value assertions red"*. There is **no value assertion on a parsed phrase anywhere in that file** (grepped: `ComposerRecord::` appears at `:31 :46 :55 :64 :166 :260`, none of them `Phrase`). What actually catches it is the CLASS row — `rsplit_once` makes the method text `hardened,one, two`, which is not a known selector, so the record classifies `Unknown`. The mutation IS caught; the plan's stated mechanism is not the one that fires. |
| **accept an unknown method** | **RED**, exactly as predicted. `phrase-unknown-method: phrase:7363727970742c… / left: "Phrase" / right: "Unknown"`. Reverted. |
| **gate the classifier on admission** | **NOT RUNNABLE AT THIS BOUNDARY** — `error[E0609]: no field pack_preimage on type Admission` at `sysw/mod.rs:290` and `:314`; the field is Task 3 Step 1. **Deferred to the Task 3 boundary and RUN there** (mutation I below). |

---

## Task 3 — commit `8cf2a7f9`

`crates/me-cli/tests/sysw_pack_preimage.rs` is 448 lines and `grep -c '^#\[test\]'` = **12**, the
plan's own figure; all twelve names match the plan's table (the table's warning that three earlier
names — `the_flag_admits_both`, `warning_order_against_the_ceremony`,
`the_orphan_warning_covers_both_carriers` — do not exist is respected).

### RED, in two stages

**Stage 1** — the test file alone, before any Task 3 source change:

```
error[E0560]: struct `Admission` has no field named `pack_preimage`
   --> crates/me-cli/tests/sysw_pack_preimage.rs:148:17
```

**Stage 2** — after Step 1 (the `Admission` field) and Step 3 (the clap flag and its wiring into
`Admission`) ONLY, so the flag exists and reaches `Admission` but nothing reads it. This is the
behavioural RED:

```
Summary [   0.014s] 12 tests run: 2 passed, 10 failed, 0 skipped
```

| test | assertion, verbatim |
| --- | --- |
| `no_flag_refuses_both_carriers_by_index` | `carrier 0 packed without the flag` |
| `the_three_ids_each_get_their_own_refusal` | `not §8.1.2: me: record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), not a seed record; this container cannot place one yet. …` |
| `the_warnings_print_in_the_f246_order` | `§8.2.1 was not printed` |
| `the_flag_seals_by_default_and_names_the_class` | `§8.2.4 was not printed` |
| `the_sealed_transit_note_does_not_point_the_wrong_way` | `§8.2.4 was not printed` |
| `the_no_hash_record_note_prints_once_per_payload` | `assertion left == right failed: the payload-wide note printed 0 times` |
| `no_hash_record_at_all_is_a_note_not_a_warning` | `the incomplete case did not draw the NOTE` |
| `a_space_after_the_comma_derives_a_different_preimage_and_warns` | `the space-after-the-comma row did not warn` |
| `the_flag_over_nothing_is_a_warning` | `sealing:  NOT SEALED — no record in this payload is secret material, so there …` |
| `the_no_op_warning_is_silent_when_a_carrier_shaped_record_is_present` | `the no-op warning went silent over a payload with no carrier shape at all` |

The two that PASSED are Task 2's work: `classification_is_unconditional` and
`the_flag_admits_both_carriers` (trivially true while nothing refuses).

### Task 3 boundary gate

- `cargo fmt --all -- --check` — **exit 0, clean**.
- `cargo clippy --locked --all-targets -p mnemonic-engrave` — **exit 0**, only the pre-existing
  `is_multiple_of` warning.
- `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` —
  **633 tests run: 630 passed, 3 failed, 2 skipped.**

That is **the plan's Task 3 figure exactly**, and the three failures are exactly the box-local
`history_purge` trio (`the_harness_records_history_at_all`,
`editing_the_file_alone_is_the_trap_the_message_warns_about`,
`the_emitted_zsh_recipe_actually_purges_the_entry`; each panics with the file's own
*"/usr/bin/zsh is required"*). **No fourth failure.** All 12 `sysw_pack_preimage` tests PASS.

**Both of Task 2's transient reds are closed in this same run**, as its commit message said they
would be:

```
PASS (222/633) mnemonic-engrave sysw::tests::a_preimage_plate_is_named_not_misdiagnosed
PASS (450/633) mnemonic-engrave::preimage_plate_is_not_a_seed sysw_pack_names_a_preimage_plate_and_never_echoes_it
```

### Two Step deliverables no test covers, verified by RUNNING the built binary

Step 8b and Step 10 are operator-surface changes with no row in the test table, so both were
executed against `target/debug/me` rather than read:

- **Step 8b** — `me sysw pack --help` now documents the `phrase:` body: *"`phrase:<hex of
  "<method>,<phrase>">` is a hashlock PHRASE and the method that derives its preimage — `hardened`
  or `sha256` — cut on the FIRST comma …"*. Before this it had a single `phrase:` hit, inside
  `--pack-preimage`'s own flag text; it now has two.
- **Step 10** — a plate on argv draws the new wording, not the transaction wording:
  *"me: argument 4 on ARGV … is a HASHLOCK PREIMAGE -- the plate string, or the phrase a `phrase:`
  record carries. For a key-less hashlock path it alone spends the coins."*

### Mutations — all five of the plan's, plus the three R0-round-0 `RUN` rows, plus Task 2's deferred one. Each applied, run once, and reverted.

| # | mutation | measured failure (verbatim) | vs. the plan |
| --- | --- | --- | --- |
| A | drop `Preimage`/`Phrase` from `Class::is_secret` | `the_warnings_print_in_the_f246_order`: **`the passphrase ceremony did not run`**. Also reds `the_flag_seals_by_default_and_names_the_class` (`not sealed:`), `the_sealed_transit_note_does_not_point_the_wrong_way`, `classification_is_unconditional` (`… is not secret`). 12 run: 8 passed, 4 failed | **exact match**. This is the funds-relevant row: NOT SEALED means the payload ships bearer material in cleartext |
| B | drop `admit_check`'s second rule | `no_flag_refuses_both_carriers_by_index`: **`carrier 0 packed without the flag`**. 12 run: 11 passed, 1 failed | **exact match** |
| C | drop the phrase arm from `preimage_digest_of` | `a_space_after_the_comma_derives_a_different_preimage_and_warns`: **`the space-after-the-comma row did not warn`**. 12 run: 11 passed, 1 failed | **exact match** |
| D | make the no-`hash:` case a `WARNING` | `no_hash_record_at_all_is_a_note_not_a_warning`: **`the incomplete case did not draw the NOTE`**; also `the_no_hash_record_note_prints_once_per_payload` (`printed 0 times`). 12 run: 10 passed, 2 failed | **exact match**, and caught by one more row than the plan claimed |
| E | drop the id test from `preimage_plate_admissible` | `the_three_ids_each_get_their_own_refusal`: **`not §8.1.2: me: record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), not a seed record; this payload did not ask for one. … Re-run with --pack-preimage if that is what you intend.`** Also `the_no_op_warning_is_silent_…` (`the wrong-id plate packed`), `a_preimage_plate_is_named_not_misdiagnosed`, `every_corpus_record_classifies_as_it_did_before_s2`. 633 run: 626 passed, 7 failed | **exact match**, plus three more catchers |
| F | restore the unconditional emptiness test (R0 r0) | `the_no_op_warning_is_silent_when_a_carrier_shaped_record_is_present`: **`§8.2.2 fired above the refusal for the wrong-id plate, and the two contradict each other`** | **exact match** to the plan's quoted RUN |
| G | move the no-`hash:` branch back inside the loop (R0 r0) | `the_no_hash_record_note_prints_once_per_payload`: **`assertion left == right failed: the payload-wide note printed 2 times`** | **exact match** to the plan's quoted RUN |
| H | restore *"the passphrase above"* (R0 r0) | `the_sealed_transit_note_does_not_point_the_wrong_way`: **`§8.2.4 points ABOVE at a passphrase that is printed BELOW it`** | **exact match** to the plan's quoted RUN |
| I | **gate the classifier on admission** (Task 2's deferred mutation) | 633 run: **618 passed, 15 failed** — 12 beyond the baseline trio. `every_corpus_record_classifies_as_it_did_before_s2`: `codex32_seam/preimage-plate-0x03: class moved under S2 / left: "Unknown"`. `the_flag_seals_by_default_and_names_the_class`: `not sealed: sealing:  NOT SEALED — no record in this payload is secret material`. Also `classification_is_unconditional`, `every_case_classifies_as_its_row_says_and_refuses_with_its_line`, `sysw_pack_names_a_preimage_plate_and_never_echoes_it`, and seven more | The plan predicted the device-side consequence (*"the corpus row says `Unknown` while the device's `Classify` says `ClassPhrase`, and Task 7's vendored lockstep reds"*). **The corpus row does move to `Unknown` here** — so the Rust half of that prediction is confirmed on this branch, and Task 7's lockstep will see the same. The NOT SEALED row is §3.2's whole design justification, executed |

Working tree is clean at the tip; every mutation was reverted with `git checkout <file>` and
`git status --porcelain` is empty, so `8cf2a7f9` is byte-for-byte the content the green gate ran
against.

---

## Findings for the controller

1. **`preimage_plate_is_not_a_seed::sysw_pack_names_a_preimage_plate_and_never_echoes_it` is
   falsified by Task 2 and repaired by Task 3, and NO task names it.** Task 13 (records) may want
   it written down: the test's `assert!(!out.status.success())` depends on `admit_check`'s second
   rule, so anyone who ever splits Task 2 from Task 3 again meets it. Not a code defect — the final
   tree passes it unmodified.
2. **Both per-task boundary gate figures in the plan are the final tree's, not the boundary's.**
   Task 2's is off by two failures; Task 3's is exact only because Task 3 is the last task in this
   group. Worth folding into the plan if it is re-run.
3. **The plan's stated catcher for Task 2's first mutation is wrong** (it names value assertions
   that do not exist in `sysw_composer_records.rs`). The mutation is caught, by the class row
   instead. A reviewer taking the plan at its word would look for a test that is not there.
4. Nothing else deviates. No secret or preimage bytes were written to any log kept under
   `/scratch/code/shibboleth/.tmp/h6-b-logs/` beyond the `phrase:` hex the plan's own `CASES` table
   publishes (all of which encode the public anchor phrase `correct horse battery staple` and its
   variants, already in the committed corpus).

**Every count above comes from a run captured to a file under
`/scratch/code/shibboleth/.tmp/h6-b-logs/` and is quoted from there**, not from memory:
`task2-red.txt`, `task2-regen.txt`, `task2-fmt.txt`, `task2-clippy.txt`, `task2-gate.txt`,
`task2-mut1.txt`, `task2-mut2.txt`, `task2-mut3.txt`, `task3-red1.txt`, `task3-red2.txt`,
`task3-green.txt`, `task3-fmt.txt`, `task3-clippy.txt`, `mutA.txt` … `mutI.txt`.
