# IMPL-S2-P2 — implementer report, P2 of `IMPLEMENTATION_PLAN_descriptor_input_S2.md`

Worktree `/scratch/code/shibboleth/me-worktrees/impl-descriptor-s2`, branch
`impl/descriptor-s2`, base `5cf5c34` (P1's HEAD). Six commits,
`70f566e..0c130ba`, plus this report. Nothing pushed; the main checkouts and the
fork were not touched — the fork worktree `sh-worktrees/s2-descriptor-arm` was
READ (its parser run as the measuring instrument) and not written.

**P2 GATE: GREEN.** 579 tests run, 579 passed, 1 skipped (the pre-existing
regenerator ignore at `crates/me-cli/src/sysw/vectors.rs:132`); `cross_lang`
RAN; `lint-gate.sh` PASS on all three legs; zero NEW `#[ignore]`;
`item_5_the_five_case_matrix` green at every one of the six commit boundaries.
Invariant 1 holds: `descriptor_seam_vectors.json` changed bytes EXACTLY ONCE,
in `70f566e`, `542cd492…` → `e7a4160c…`.

---

## What landed, per task

| task | commit | what |
| --- | --- | --- |
| P2.1 + P2.2 + P2.4 + P2.6 | `70f566e` | the flip, the pack, the row retirement, and THE single regeneration — forced atomic, see DEVIATIONS 1 |
| P2.1 (inherited test) | `e54a1b9` | `--as descriptor --expect descriptor` exits 0 — the form P1 deferred |
| P2.2 (M2 half) | `df00632` | the choice block's padding fix + the verbatim block test that did not exist |
| P2.5 | `cde5c8b` | `me sysw show`'s per-record `Class::Descriptor` confirmation block |
| P2.3 | `2e88da4` | §11 item 1's host half — one record per format, and it is the measured canonical |
| P2.7 | `0c130ba` | the spec amendments, plus `gate.rs`'s module doc, which the sweep caught |

### `70f566e` — P2.1/P2.2/P2.4/P2.6

`DESCRIPTOR_PATH_SHIPPED = true`; `descriptor_follower` returns
`Decision::Pack(vec![d.encode()])` after §4.7 admission, so conjunct 1's `multi`
refusal still precedes everything. `carriage()`'s dead `WindowNotInBuild` arm
became `Outcome::AsDecides` (with both paths shipped, `md1_carries` IS
`md1_admits.is_ok() && representable.is_ok()`, both established by the two
returns above it — so the arm is unreachable, and a total function beats a panic
on an operator path). `identify::window_refusal` and `Row::WindowNotInBuild`
retired. The four `md1-split/*` rows flip exit 3 → 2 on the `--as`-omitted path;
`window_remedy()`'s two §6 rows now print §6's own remedy; the clap help
un-marks; §11 item 5's matrix became the full-build truth table.

`§5.4`'s identification block needed no work — `as_flag::run` builds it before
the follower `match` (`as_flag.rs:79`), verified.

### `df00632` — the padding fix

§5.1's NORMATIVE block puts each description INLINE on a head padded to column
24; the shipped code rendered `      --as descriptor` on its own line. One
`head(flag, shipped)` helper now lays out BOTH arms, because the marked arm has
a different layout and no build state reaches it — an untested dead layout is
how a comment outlives its condition.

### `cde5c8b` — `show`

Per public `Class::Descriptor` record, one confirmation block reusing
`descriptor::identification_block(record, None)`. Additive: the guard is
classification, and `testdata/show_public_records_pre_s2.txt` — captured at
`df00632`, before the block existed — is compared by EQUALITY over five
containers covering md1, mk1, a complete mt1 set with its set line, a text
record and a mnemonic.

---

## MEASURED numbers

Nothing below was predicted. `gen.py` was first run at BASELINE (unmodified
generator, unmodified `rows.py`, `_work/seam-fork` at fork `main` `a5e29b4`,
and a `md` debug binary REBUILT at `descriptor-mnemonic` `6c4a56fd`) and
reproduced the pinned sha `542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974`
byte-for-byte — so the reproduction path is alive, not merely documented, and
every number below is a delta against a verified starting point.

### The post-retirement §6 row count: **35**

Measured three ways, all asserted in the suite: `descriptor::Row::ALL` = 35, the
vector file's `refusal_rows` = 35, and `descriptor_refusals.rs`'s own
`fn row_*` set = 35 (`grep -c '^fn row_'`). The plan's expected 35 = 36 − 1 is
confirmed. S2 adds none.

### The two measured `device_admits` booleans

Measured by running `goprobe` against
`/scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm` @ `0abbf81` (P3.1's
`!= 4` fingerprint guard + P3.4's `ypubVer` case), never predicted:

| row | before | MEASURED after |
| --- | --- | --- |
| `bluewallet/short-fingerprint` | `device_probe: "panic:parse"`, `device_admits` ABSENT | **`false`** — `nonstandard: unrecognized output descriptor format`, a clean error, no panic |
| `neither/full-origin-ypub` → `version-gap/full-origin-ypub` | `false` | **`true`** — parses, `Nested Segwit (P2SH-P2WPKH)`, canonical `sh(wpkh([4bbaa801/49h/0h/0h]xpub6C9j4…/<0;1>/*))#ve5r4lwl`, fixed point |

A third measurement, the NEW row `neither/wsh-multi-fixed-path`: `device_admits`
**`false`** (the device parser rejects `multi`). r5 N1's attribution is
confirmed by run, not by reading.

### Final guard values

| guard | before | after |
| --- | --- | --- |
| `SEAM_VECTORS_SHA256` | `542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974` | `e7a4160ce064a6cb7ca31dc530e079c861cf2c8a075d75f793ef0d935f583758` |
| `TAG_SLOTS` | 88 | **89** |
| `ROW_FLOOR` | 71 | **72** |
| `SECOND_TAGGED` / `THIRD_TAGGED` | 15 / 2 | **15 / 2 — did NOT move** (both touched rows are single-tagged, as r5 I1a predicted) |
| `MANIFEST` | 9 tags | **10** — `("version-gap", 1)` added, `("neither", 3)` unchanged |
| `SINGLE_LINE_ROWS` / `SINGLE_LINE_ADMITTED` | 58 / 15 | **59 / 15** |

`POP`, every field measured from the regenerated file:

```
rows 72          host_admits_true 19     md1_admits_true 15
device_admits_true 38   device_admits_false 34   device_admits_absent 0
canonical 19     address_0 20    address_1 5     wallet_id 4
md_descriptor_contains 1        sysw_class 0    device_probe 2
gate_fields 37   refusal_row 18  both_routes_address_0 11
```

Two further counts the file forced, both measured and both now asserted:
`the_md1_column_matches_the_representability_rules`'s `cited` 5 → **6** (the new
row's md1 refusal is §5.3(a)'s too), and the §5.2 widening count in the spec —
`device_admits` true with `host_admits` false is **22 of 72 rows, 18 of them
single-line** (R0 r1's "17" was the pre-S2 file, before the `ypub` boolean
flipped; the spec now states the number WITH its provenance rather than
inheriting it).

### Test counts

| point | run | passed | skipped |
| --- | --- | --- | --- |
| base `5cf5c34` | 575 | 575 | 1 |
| `70f566e` | 573 | 573 | 1 |
| `e54a1b9` | 574 | 574 | 1 |
| `df00632` | 576 | 576 | 1 |
| `cde5c8b` | 578 | 578 | 1 |
| `2e88da4` | 579 | 579 | 1 |
| `0c130ba` | 579 | 579 | 1 |

575 → 573 at the flip is the retirement: `row_window_not_in_build` and
`the_window_refusal_has_two_variants` both lost their trigger. Net +4.

Matrix witness, per commit, on a detached checkout:

```
70f566e  Summary [0.024s] 1 test run: 1 passed, 573 skipped
e54a1b9  Summary [0.013s] 1 test run: 1 passed, 574 skipped
df00632  Summary [0.007s] 1 test run: 1 passed, 576 skipped
cde5c8b  Summary [0.008s] 1 test run: 1 passed, 578 skipped
2e88da4  Summary [0.012s] 1 test run: 1 passed, 579 skipped
0c130ba  Summary [0.029s] 1 test run: 1 passed, 579 skipped
```

---

## Mutation evidence — every new gate was shown to fail

Applied, measured, reverted. Not "the tests pass".

1. **Narrowing `Kind::Descriptor` back to `card_hrp(record) == Some('d')`** reds
   `as_descriptor_with_expect_descriptor_packs` — the funds-path invocation
   fails in exactly the world P1.3's widening exists to rule out.
2. **Un-padding the choice block's head** (`head()` returns `lead`) reds BOTH
   new block tests — and `row_as_omitted` and `item_5_the_five_case_matrix`
   stay GREEN. That is the plan's M2 blind spot, measured: a `contains` on one
   sentence cannot see a column.
3. **Dropping the `print_descriptor_confirmation` call** reds
   `show_reports_exactly_one_descriptor_record_for_each_of_the_four_formats`
   and nothing else — so that test, not a bystander, holds the surface.
4. **Widening the `show` block's classification guard** so a non-descriptor
   record reaches it reds the byte-identity capture AND the shipped
   `show_states_confirmed_or_unconfirmed_beside_each_mdmk_record`. "Additive" is
   a measurement here, not a claim about a `continue`.
5. **Packing `d.encode()` with the BIP-380 checksum stripped** reds
   `item_1_every_format_packs_one_descriptor_record` and NOTHING else in
   `descriptor_as` or `sysw_cli` — P2.5's `show` test re-parses and re-encodes,
   so it prints the right canonical from a wrong record. The two tests are
   complementary, and that is measured rather than asserted.

---

## The propagation sweep — survivors and dispositions

Whole repo, excluding `design/agent-reports/` and `third_party/`. **One real
finding**, then survivors.

**FOUND AND FIXED (in `0c130ba`):** `crates/me-cli/src/descriptor/gate.rs:3` —
the module doc still opened *"When `--as` is absent and record classification
fails, `me` consults this gate"*. That is the precondition P1.0 abolished, in
the very file whose behaviour P1.0 changed, and no test reads a doc comment.
Term: `record classification fails`. It is exactly the class the sweep exists
for — a diff falsifying text it never touched.

Every other surviving hit, by disposition:

| term | survivors | disposition |
| --- | --- | --- |
| `sysw_class` | `descriptor_seam.rs:91,129,152,154,527` | **DELIBERATE.** `POP.sysw_class = 0` and its assertion are what keep the column retired — a re-added column reds. The `KNOWN_ROW_KEYS` entry was REMOVED, so a returning column also reds as an unknown key. |
| `sysw_class` | `goprobe/main.go:45` | **DELIBERATE.** The probe still computes `sysw.Classify` per input; `gen.py` no longer reads it. Kept as a diagnostic the next measurement may want, and it emits nothing into the file. |
| `sysw_class` | `comment.json:53`, the vector file's `_comment` | **RECORD OF THE RETIREMENT** — the block now says the column is retired and what replaced it. |
| `sysw_class` | `CONTINUITY_2026-08-29-s2.md`, `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md`, the S2 plan itself | **RECORDS.** |
| `panic:parse` | `descriptor_seam.rs:110,277,286,287,436,464` | **DELIBERATE.** The marker VOCABULARY survives (`KNOWN_PROBES`) with the schema clause that binds `device_admits` presence to it; no row carries it, and `POP.device_admits_absent = 0` asserts that. Retiring the vocabulary as well is a fork-side shape change P3.3 owns. |
| `panic:parse` | `gen.py:80,88,101`, `goprobe/main.go:23` | **DELIBERATE.** The generator keeps the capability to mark a future panicking row; no row uses it. |
| `panic:parse` | spec `:420`, `:1643`, `:1706`, `comment.json` | **RECORDS OF THE RETIREMENT** (my own amendment text quoting the retired clause). |
| `PANICS the Go parser` | spec `:411` | **P3.5's, deliberately untouched** — it describes fork `main`, which has no `!= 4` guard. The plan assigns it to P3.5 with P3.1 as the falsifying diff. |
| `record classification fails` | spec `:46`, `:903` | **AMENDMENT TEXT** quoting the abolished precondition. |
| `gate_open` | 59 hits, all the COLUMN and the `gate_opens` function | **UNFALSIFIED.** The column exists and means the same thing; only its definition SENTENCE carried the precondition, and that sentence is amended (spec `:1663` region). |
| `ypub` | `cascade.rs:58-62` | **P3.5's, deliberately untouched** — the host comment asserting the device's five. Named in the plan as P3.5's. |
| `ypub` | `cascade.rs:110,490`, `refusal.rs:538`, `keytool-main.go` | **UNFALSIFIED** — the version spelling and the per-version remedy, both about `me`, whose admission S2 does not change. |
| `ypub` | `admit.rs:23`, `descriptor_refusals.rs` `vector_input` | **RENAMED** to `version-gap/full-origin-ypub` in `70f566e`, as the plan's r6 M2 requires. |
| `tag-slots` | `descriptor_seam.rs:69`, `comment.json:115`, spec `:52`, `:1849` | **UPDATED to 89.** The one remaining old value is in `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md` — a completed sibling plan, a RECORD. |
| `not yet classifiable` | spec `:120` (marked HISTORY), `:135` (the amendment), `FOLLOWUPS.md:5212`, `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md:422`, `design/journeys/build_pdf_payload.py:207` | **RECORDS**, every one: two are transcripts of the pre-fix message, and the journey page quotes it while SAYING it was fixed. |
| `not available in this build` | `gate.rs:583`, `main.rs:361` | **DELIBERATE.** The build-marking machinery §5.1 defines as conditional. Both constants are true, so neither marking renders; `gate.rs`'s own unit test exercises the marked arm directly so the dead layout is not untested. |
| `not available in this build` | `descriptor_as.rs:945,955`, `descriptor_refusals.rs:280,907,931,939,947` | **ASSERTIONS OF ABSENCE** — they are what would catch a marking that outlived its condition. |
| `scannable-plate path is not in this build` | `gate.rs:280` | **DELIBERATE**, and the one survivor worth arguing about: §5.3's window SUBSTITUTION is defined by the spec as build-conditional, so the code keeps the conditional and the spec now says the condition cannot occur. Deleting the branch would put the code out of step with a rule that still stands. |
| `scannable-plate path is not in this build` | spec `:1209` | **MARKED** — the rule stands, its condition cannot occur. |

---

## DEVIATIONS from the brief

**1. P2.1, P2.2, P2.4 and P2.6 are ONE commit, and the coupling is forced by the
code, not chosen.** The brief asks for one commit per task with the suite green
at every boundary; these four cannot be separated while both hold. The
mechanism, measured rather than argued:

* `descriptor_seam.rs`'s `the_refusal_row_vocabulary_is_the_same_set_on_both_sides`
  asserts SET EQUALITY between the vector file's `refusal_rows` map and
  `descriptor::Row::ALL`, and `descriptor_refusals.rs`'s
  `the_file_carries_one_named_test_per_section_6_row` asserts the row-test set
  equals the same vocabulary. So **P2.4's retirement cannot land without a
  vector-file byte change** — and invariant 1 allows exactly one, P2.6's.
* **P2.1's flip leaves `Row::WindowNotInBuild` with no producer**, so
  `row_window_not_in_build` (which runs the binary) cannot pass after the flip
  and the retirement cannot land after it either.
* The alternative — a hand edit to `refusal_rows` at P2.4 and a regeneration at
  P2.6 — is two byte changes and two sha bumps, which invariant 1 forbids in
  terms ("ONE sha bump per repo").

What I DID separate: P2.2's padding fix and its verbatim test are not coupled to
the flip and landed alone (`df00632`), as did P2.1's inherited `--expect` test,
P2.5, P2.3 and P2.7.

**2. `refusal_rows.json`'s surviving 35 entries are RENUMBERED.** Each
description opens `"S6 row N -- …"`, naming a position in §6's table. Retiring
row 9 would have left every later number pointing one row past its own text —
a false annotation of exactly the class this project keeps paying for. Nothing
binds to the strings (only the key set is asserted), so the change is data and
mechanical. It is the reason that file's diff is 27 lines wide.

**3. `goprobe/go.mod`'s `replace` is committed pointing at
`/scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm`.** The README
documents it as a knob you point at your fork worktree; committing it at the rev
the corpus was MEASURED against is what makes the reproduction command
reproduce. `comment.json`'s PROVENANCE paragraph and the README's baselines both
name fork `0abbf81` and `descriptor-mnemonic` `6c4a56fd` now. **P3 should
re-point it at fork `main` once P3.1/P3.4 merge** — the file is otherwise
truthful but transiently names a worktree.

**4. `gen.py` was edited, not only `rows.py` and `comment.json`.** Three
changes: the `sysw_class` emission removed (a retired column that a generator
can still emit is a column that comes back), and two comments repointed at the
measurement rev. The brief named `rows.py` and `comment.json`; F-428's fix is in
the generator too, so the file was already in scope.

**5. §11 item 4 was amended, and the brief's P2.7 list does not name it.** It
says *"the `--as descriptor`-only rows among them are S2's"* — a set S2 measured
as EMPTY while SUBTRACTING a row. Leaving it is the "a diff falsifies text it
never touches" class, and the count is machine-checkable. Amended with the
measured 35.

**6. The four-format `show` test landed at P2.5, not P2.3.** The brief names it
under both; it is P2.5's own named test ("the four §11 item 1 containers each
report exactly ONE descriptor record") and it needs P2.5's code. P2.3's commit
carries the pack-side half — one record, classifying `Descriptor`, byte-equal to
the measured canonical, with §5.3(b)'s label warning named.

**7. `gen.py`'s device-column pipeline matches the README**; no divergence to
report. The README's "point `replace` at your fork worktree, build `rsprobe`,
run `gen.py`" is exactly what runs, `rsprobe` built offline from the vendored
registry, and nothing in the generator reaches the network.

**No other deviation.** The vector file changed bytes once; the fork's copy was
NOT touched and diverges transiently until P3.3, by design; §4.3's device
clauses, §4.5's promotion prose, §9 item 2 and `refusal.rs:583` are untouched.

---

## Gate output tails

```
$ ME_REQUIRE_GO=1 PATH=/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH \
  cargo nextest run --locked --no-fail-fast
        PASS [  32.011s] (579/579) mnemonic-io-lib::fish_history_purge history_delete_prefix_purges_nothing_however_it_fails
────────────
     Summary [  32.186s] 579 tests run: 579 passed, 1 skipped
exit=0

  (cross_lang RAN, not skipped:)
        PASS [   0.268s] (573/579) mnemonic-engrave::cross_lang rust_ndef_parses_in_seedhammer_go_reader
        PASS [   0.156s] (571/579) mnemonic-engrave::preview_cross_lang real_sidecar_renders_public_plates_only

$ ./scripts/lint-gate.sh
== cargo fmt --check
== clippy (CI-pinned 1.85.0)
== clippy (nightly)
lint-gate: PASS
exit=0

$ grep -rn '#\[ignore' crates/
crates/me-cli/src/sysw/vectors.rs:132:    #[ignore = "regenerates the fixture; run deliberately"]

$ sha256sum crates/me-cli/testdata/descriptor_seam_vectors.json
e7a4160ce064a6cb7ca31dc530e079c861cf2c8a075d75f793ef0d935f583758

$ git log --oneline --name-only 5cf5c34..HEAD -- crates/me-cli/testdata/descriptor_seam_vectors.json
70f566e S2 P2.1+P2.2+P2.4+P2.6: --as descriptor ships, and the file S2 regenerates once
crates/me-cli/testdata/descriptor_seam_vectors.json
  (ONE commit — invariant 1)
```

`scripts/plan-staleness-check.sh design/IMPLEMENTATION_PLAN_descriptor_input_S2.md . 5cf5c34`:

```
─── unchanged: 18 ; DRIFTED: 45 ; not in this repo: 36
```

The 45 drifted citations are **P2's own diff** — 30 into `crates/`, 14 into
`design/SPEC_descriptor_input.md`, 1 into the generator README — every one a
line the plan cited and P2 moved or deleted (`gate.rs:42`, `as_flag.rs:126-138`,
`refusal.rs:43/83/124`, the §7 line numbers, `README.md:9`). None is a citation
that was wrong when written; the script's own footer states it checks bytes, not
whether a cite was ever right. The 36 "not in this repo" are the fork's.

`scripts/spec-structure-check.sh design/SPEC_descriptor_input.md` reports **30
findings both before and after** P2.7 — diffed line-for-line with line numbers
normalised, the sets are IDENTICAL, so the amendments introduced no structural
regression. (They are pre-existing: the script's `§N.M` cross-reference
heuristic does not model `### 5.2`-style subsections, and one table row contains
an escaped pipe.)

Whole-branch diffstat:

```
 crates/me-cli/src/descriptor/admit.rs              |   2 +-
 crates/me-cli/src/descriptor/as_flag.rs            |  29 +--
 crates/me-cli/src/descriptor/gate.rs               | 120 ++++++---
 crates/me-cli/src/descriptor/identify.rs           |  38 +--
 crates/me-cli/src/descriptor/refusal.rs            |  13 +-
 crates/me-cli/src/main.rs                          |  36 +++
 crates/me-cli/testdata/descriptor_seam_vectors.json| 145 ++++++-----
 crates/me-cli/testdata/show_public_records_pre_s2.txt| 38 +++
 crates/me-cli/tests/descriptor_as.rs               | 228 +++++++++++++++--
 crates/me-cli/tests/descriptor_refusals.rs         | 249 +++++++++++-------
 crates/me-cli/tests/descriptor_seam.rs             |  51 ++--
 crates/me-cli/tests/expect_kinds.rs                |  27 ++
 crates/me-cli/tests/sysw_cli.rs                    | 128 ++++++++++
 design/SPEC_descriptor_input.md                    | 279 ++++++++++++++++-----
 scripts/descriptor-seam-vectors/README.md          |   7 +-
 scripts/descriptor-seam-vectors/comment.json       |  59 +++--
 scripts/descriptor-seam-vectors/gen.py             |  12 +-
 scripts/descriptor-seam-vectors/goprobe/go.mod     |   2 +-
 scripts/descriptor-seam-vectors/refusal_rows.json  |  55 ++--
 scripts/descriptor-seam-vectors/rows.py            |  43 +++-
 20 files changed, 1150 insertions(+), 411 deletions(-)
```

---

## For the controller — three things P2 found that P2 does not own

**A. `goprobe/go.mod` names a transient worktree.** See DEVIATIONS 3. Once
P3.1 and P3.4 merge to fork `main`, the `replace` (and the two provenance
paragraphs naming `0abbf81`) should be repointed, so a future regeneration is
against a rev that still exists. It is not a P2 defect — the corpus WAS measured
there — but it is a decay path.

**B. The plan's "the two `source` annotations" for F-428 is one annotation plus
one copy.** `parse.go:151` appears exactly ONCE in the engrave vector file and
once in `rows.py`; the "second" annotation is the FORK's copy of the same file,
which P3.3 updates when it takes P2.6's bytes. Nothing was missed — the count
just reads as two sites in one repo and is not.

**C. F-428's target line is `:158` at fork `main` and `:161` at `0abbf81`.**
Measured: the count-mismatch `return` is line 158 at `a5e29b4` and 161 at
`0abbf81`, because the parse fix adds three comment lines. I used **`:158`**, the
value F-428 and the plan both name and the value true at fork `main` — the rev
the corpus's other citations are written against. Worth a controller decision if
P3 wants every cite re-based on the merged fork.
