# REVIEW-S2-P1P2-r1 — proportional adversarial execution review, P1+P2 of `IMPLEMENTATION_PLAN_descriptor_input_S2.md`

## Header

**Target diff:** `git diff 0144f02..dbcd6b0` in
`/scratch/code/shibboleth/me-worktrees/impl-descriptor-s2`, branch
`impl/descriptor-s2`.
P1 = `b8f0538..5cf5c34` (5 commits + report), P2 = `70f566e..dbcd6b0`
(6 commits + report). 27 files, **+2641 / −460**.

**Trees at review time**

| tree | rev | state |
| --- | --- | --- |
| engrave worktree `impl-descriptor-s2` | `dbcd6b0` | clean before and after; `git status --porcelain -uall` EMPTY at exit |
| engrave main `/scratch/code/shibboleth/mnemonic-engrave` | `8a74d02` | clean; moved one commit past the brief's `21ef965` (`continuity: P2 done+verified`) — `git diff 21ef965..8a74d02 -- design/IMPLEMENTATION_PLAN_descriptor_input_S2.md design/SPEC_descriptor_input.md` is EMPTY, so the review basis is unchanged |
| fork worktree `sh-worktrees/s2-descriptor-arm` | `0abbf81` | clean; READ ONLY (its parser built and run as the measuring instrument) |

**Question answered:** does P1+P2 faithfully build what the GREEN plan's P1 and
P2 direct, and did IMPLEMENTATION introduce defects the suite cannot see? Plan
design was NOT re-reviewed.

**Counts: 0 Critical / 1 Important / 3 Minor / 2 Nit.**

**VERDICT: RED** — one Important. It is a false claim in an S2 spec amendment
plus a mis-named test, both closable in one edit; nothing in the behavioural
surface is wrong. Everything else measured true, including every number in
the regeneration and both device-measured booleans, re-derived independently.

---

## Execution evidence — what I ran, observed vs spec

Binary: `target/debug/me` at `dbcd6b0` (`cargo build --locked --bin me`,
already current). Inputs extracted from the regenerated vector file's own
`input` column.

### §11 item 1 — the four formats through `--as descriptor`

`me sysw pack --no-passphrase --as descriptor --in <row input> --out <bin>`
then `me sysw show <bin>`:

| row (format) | pack | records | `show` | canonical packed |
| --- | --- | --- | --- | --- |
| `formats-happy/bluewallet-sh-fixture` (bluewallet) | exit 0 | 1 | `public record 0: descriptor — complete in one record` + §5.4 block | `wsh(sortedmulti(2,…))#tk50fvpm` |
| `formats-happy/bip380-sortedmulti-multipath` (bip380) | exit 0 | 1 | same | `…#ud8uyjz3` |
| `formats-happy/json-label-descriptor` (json) | exit 0 | 1 | same | `…/0/*…#hfwurrvt` |
| `promotion/01-bare-xpub` (promoted-key) | exit 0 | 1 | same | **`pkh(xpub6C9j4…GnX)#775scpf7`** — a canonical descriptor, never raw bytes |

§5.4's identification block printed on the pack path for all four (read-as /
descriptor / script / wallet-id / address 0 / compare-prompt / watch-only), and
`show` re-prints the same block from the packed record. §5.3(b)'s label warning
fired on exactly the two exemplars carrying a label (`sh`,
`Test Multisig 2-of-3`) and on neither of the other two. Matches §5.2
("canonical re-encoded descriptor string … as one record of class
`Descriptor`") and §11 item 1's amended host half.

Childless input (`md1-split/childless`) packs the **childless** canonical
`wsh(sortedmulti(2,K0,K1))#n2cpznut` under `--as descriptor` — §5.3(a′)'s
`<0;1>/*` materialisation stays md1's, correctly. Its reported `address 0`
matches the Go probe's byte-for-byte.

### §11 item 5 — the matrix, invoked by hand

| case | invocation | observed | spec |
| --- | --- | --- | --- |
| 1 carried | `--as` omitted, bip380 happy | **2**, §5.1 block, 0 stdout bytes | §5.1 ✓ |
| 2 inadmissible, `--as` omitted | `narrowed/threshold-zero` | **3**, *"threshold 0 means NO signature is required … treat them as at risk now"* | §6 / §5.4 carriage ✓ |
| 3 carried by NEITHER | `neither/wsh-multi-fixed-path` | **3**, *"No `me` path engraves this file as written, in any build."*, no choice block | §5.4 carriage ✓ |
| 3b the flip S2 made | `md1-split/fixed-index`, `--as` omitted | **2** + block (was 3 pre-S2) | inventory member 2 ✓ |
| 4 inadmissible + `--as descriptor` | `narrowed/threshold-zero` | **3**, admission refusal, **no window text** | §5.1 ordering ✓ |
| 5 `multi` + `--as descriptor` | `neither/wsh-multi` | **3**, *"the device's descriptor parser accepts `sortedmulti` and not `multi`"* — **permanent, never a wait** | §4.7 conjunct 1 ✓ |

Additional inadmissible inputs under `--as descriptor`, all exit 3, none
printing a window text:
`bluewallet/short-fingerprint` → *"a master fingerprint is exactly 8 hex
characters (4 bytes)"*; `promotion/15-bare-tpub-host-refused` → *"this is a
testnet key … Supply the descriptor with its real origin"*;
`version-gap/full-origin-ypub` → *"the device admits exactly `xpub`, `tpub`,
`zpub`, `Ypub`, `Zpub` …"* (the `refusal.rs:583` text P3.5 owns, correctly
UNTOUCHED).

`grep 'not available in this build'` over every observed stderr: **0 hits**.
`me sysw pack --help`: **0 hits** — the amendment's "marks nothing" is true.

### §5.1's choice block, rendered

Padding fix verified with `cat -A`: `      --as descriptor   ` and
`      --as md1          `, both 24 columns, descriptions INLINE, every
continuation line at column 24. This is the M2 defect closed.

**But the rendered block is not byte-equal to §5.1's NORMATIVE block** — see
Important I-1 below.

### `--expect` surfaces

- `--as descriptor --expect descriptor --in <bip380 happy>` → **exit 0**, container
  written. This is the funds-path invocation P1.3's ruling exists for.
- `--expect descriptor` on a mnemonic-only container → **exit 4**,
  *"--expect descriptor was not met: NO record of that kind is in the stream.
  Looking for an md1 descriptor card, or a descriptor record (`--as
  descriptor`)."* — the widened description, both carriers named.
- `--in <descriptor> --expect mnemonic` → **exit 4** naming the mnemonic, not
  exit 2 about a choice: `--expect` still resolves ahead of the gate (r2 N2).

### Independent re-measurement of the device column — the highest-value check

I built `goprobe` from the branch's own source into scratch and ran it over
**all 72 rows** against the fork worktree at `0abbf81`:

```
device_admits mismatches: 0 of 72
```

Every boolean in the regenerated file reproduces, including the three the
implementer measured:

| row | file | my re-run @ `0abbf81` |
| --- | --- | --- |
| `bluewallet/short-fingerprint` | `false`, no `device_probe` | `false`, `parse_err: "nonstandard: unrecognized output descriptor format"` — **no panic**; the whole 72-row run exited 0 |
| `version-gap/full-origin-ypub` | `true` | `true`, `Nested Segwit (P2SH-P2WPKH)`, canonical `sh(wpkh([4bbaa801/49h/0h/0h]xpub6C9j4…/<0;1>/*))#ve5r4lwl`, `fixed_point: true` |
| `neither/wsh-multi-fixed-path` (NEW) | `false` | `false` (the parser rejects `multi`) |

`canonical` agrees on all 19 rows that carry one. `sysw.Classify` at `0abbf81`
answers `Unknown` on 72/72 — the Go arm is P3's, as scheduled.

### Regenerated-file content, recomputed from the file

```
rows 72 | device_admits absent 0 | sysw_class present 0 | device_probe {panic:encode: 2}
tag slots 89 | MANIFEST gate 37, promotion-near-miss 15, narrowed-4.7 14, md1-splits 6,
              narrowed-4.2 5, formats-happy 4, whitespace 3, neither 3,
              accepted-extreme 1, version-gap 1   (sums to 89)
refusal_rows 35 | single-line rows 59 | single-line admitted 15
device_admits TRUE & host_admits FALSE: 22 of 72, 18 of them single-line
§6 live data rows in the spec: 35 (36 table rows, 1 struck)
Row::ALL = 35 | grep -c '^fn row_' = 35 | POP.rows = 72
_comment in the shipped JSON == scripts/…/comment.json  →  True
```

Every number the S2 amendments state is confirmed by recomputation: 35, 72,
89, `89 − 17 = 72`, `22 of 72`, `18 single-line`, `("version-gap", 1)`,
`("neither", 3)`, `SECOND_TAGGED`/`THIRD_TAGGED` unmoved at 15/2.

The §5.2 amendment's exemplar list for the 18 rows is accurate — the set
contains `narrowed/threshold-zero` (anyone-can-spend),
`narrowed/threshold-exceeds-keys` (k > n),
`narrowed/wsh-sortedmulti-21-keys` (21 keys), `narrowed/mixed-network`,
`narrowed/use-site-hardened`, and the two conjunct-8 rows
(`gate/colliding-origin-sortedmulti`, `gate/duplicate-key-same-use-site`).

### The NEW witness row's content (invariant 1's payload list)

`neither/wsh-multi-fixed-path` =
`wsh(multi(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNn…/0/*,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8…/0/*))`
— two REAL fixture keys with plausible BIP-48 origins, `/0/*` on **every**
key, `host_admits=false` (conjunct 1), `md1_admits=false` (F-417),
`device_admits=false` (re-measured), `format: "bip380"` (its cascade branch
kept), single-tagged `neither`. `me` derives `address 0` for it, so the keys
parse. Exactly what the plan specified.

`version-gap/full-origin-ypub`'s `source` annotation was rewritten and is
now true of `me` rather than of the device:
*"`me` admits five versions and `ypub` is not one — refused even with a full
explicit origin. The DEVICE's parser accepts it after P3.4's `ypubVer` case
(F-426 device half) …"* — confirmed by execution (exit 3 host-side) and by
probe (`device_admits: true`).

`comment.json`'s manifest block, header block and PROVENANCE paragraph all
moved together, and the shipped `_comment` is byte-identical to the
generator input. The `refusal_rows` renumbering is **verified aligned**: I
sorted the 35 entries by their `S6 row N` prefix and zipped them against
§6's live table rows in document order — all 35 line up, 1..35 contiguous,
no gaps.

### Suite and gates, re-run by me

```
$ ME_REQUIRE_GO=1 PATH=<go-1.26.3>/bin:$PATH cargo nextest run --locked --no-fail-fast
     Summary [ 32.216s] 579 tests run: 579 passed, 1 skipped
  cross_lang RAN:  PASS mnemonic-engrave::cross_lang rust_ndef_parses_in_seedhammer_go_reader
                   PASS mnemonic-engrave::preview_cross_lang real_sidecar_renders_public_plates_only
$ grep -rn '#\[ignore' crates/
  crates/me-cli/src/sysw/vectors.rs:132  (the pre-existing regenerator)
```

### Fence verification (P2.7 vs P3.5)

**NOT touched, correctly** — confirmed by hunk arithmetic against the base
file and by direct read:

| P3.5-owned site | evidence |
| --- | --- |
| §4.3's DEVICE clauses (base `:453-461`) | no spec hunk between base `:401` and base `:810`; text byte-identical |
| §4.5's promotion prose (base `:566-578`) | same; *"has no case in the switch"* intact |
| §9 item 2 (base `:1808+`) | nearest hunk ends at base `:1807`; item 2 byte-identical |
| `refusal.rs:583/584` "the device admits exactly" | `git diff … -- refusal.rs \| grep -c "device admits exactly"` = **0** |
| §7 requirement 3 "the Go test asserts the device column" (head `:1582`) | byte-identical to base |
| spec `:411` "PANICS the Go parser" | `grep -c` in the spec diff = **0** |
| `cascade.rs:55-66` host comment asserting the device's five | file absent from the diffstat |

**Touched, correctly** — §2.1's transcript (the gap P1 found and P2.7
adopted), §4.2 defect 4's FILE half only, §5.1's gate trigger, §5.1's window
section (marked RETIRED, kept), §5.2's Go-arm implementation sentence (the
predicate sentence NOT narrowed), §5.3's window-substitution rule (marked,
condition unreachable), §5.5's firmware row, §6's table (row struck + two
remedy rows + the `--as` omitted row), §7's `sysw_class` / `device_probe` /
`gate_open` definitions / `neither` bullet + referent disambiguation /
derivation / floor table / NEW `version-gap` bullet, §8's parked sentence,
§11 items 1, 4 and 5.

Whole-repo sweep for retired phrasings in `crates/` + `scripts/`:
`not yet classifiable` → 0, `record classification fails` → 0,
`WindowNotInBuild` / `window-not-in-build` → retirement notes only.

### Mutation evidence — mine, not the implementer's

Four mutations applied, measured, reverted; the tree is byte-identical to
`dbcd6b0` at exit.

1. **Classifier answers `Descriptor` on everything**
   (`Err(_) if host_admits(record) => …` → `Err(_) => Class::Descriptor`).
   RED ×3: `every_single_line_input_classifies_by_the_admission_column`,
   `a_multi_policy_the_cascade_parses_is_not_a_descriptor_record`,
   `every_corpus_record_classifies_as_it_did_before_s2`
   (`codex32_seam/bip93-secret-128: class moved under S2, left "Descriptor"
   right "Unknown"`). **The derived-rule tests do assert, in both
   directions.**
2. **Un-pad the choice-block head** (`head()` returns `lead`). RED ×2, and
   `row_as_omitted` + `item_5_the_five_case_matrix` stayed GREEN — the M2
   blind spot is real and is now covered by exactly the two new tests.
3. **Pack a de-normalised canonical** (`d.encode().replace("h]", "']")`).
   RED ×5 including `item_1_every_format_packs_one_descriptor_record` and
   `as_descriptor_with_expect_descriptor_packs`. The packed record is pinned
   to the file's measured `canonical`, not to itself.
4. **Re-key the gate on classification failure** — the r1 C1 shape, with the
   arm present (`if r#as.is_none()` → `if r#as.is_none() &&
   admit_check(&recs, admission).is_err()`). RED ×3 with `left: 0, right: 2`:
   **the descriptor packs RAW at exit 0**, exactly the collapse P1.0 exists
   to prevent. P1.0 is load-bearing and guarded.

Test quality spot-checks: the `show` byte-identity test is an
`assert_eq!(got, want)` over a committed 5-container capture — bytes, not
vibes; the four-format `show` test pins the canonical string and the exact
`public record N:` line vector; the verbatim block test asserts the literal
AND re-derives column 24 independently so a transcription error in its own
literal cannot be pinned as spec.

---

## Deviations judged

| # | deviation (from IMPL-S2-P1 / IMPL-S2-P2) | verdict | evidence |
| --- | --- | --- | --- |
| P1-1 | classify-unchanged capture + assertion both landed at P1.0, not P1.2 | **SOUND, strictly stronger** | the capture predates `282a071`, so P1.1's arm could not move a class without reddening its own commit |
| P1-2 | P1.3's `--as descriptor --expect descriptor` exit-0 form deferred to P2.1 | **SOUND** | at P1 `DESCRIPTOR_PATH_SHIPPED == false`, so the briefed spelling would have pinned the parked exit 3; the same defect was pinned through the pack-free route, and P2.1 landed the exit-0 form (`e54a1b9`, verified exit 0 by hand) |
| P1-3 | a fifth, unbriefed commit `6efd7b5` (the `Unrecognised` message P1.1 falsified) | **SOUND** | the old text *"Descriptors and addresses are not yet classifiable here"* became false the moment the arm landed; `sysw_cli.rs:457`'s negative assertion tracked the old wording and would have gone vacuously true. New text contains the asserted substring, so the replacement is non-vacuous |
| P2-1 | P2.1+P2.2+P2.4+P2.6 in ONE commit | **SOUND — the coupling is forced, and I verified the mechanism** | `descriptor_seam.rs::the_refusal_row_vocabulary_is_the_same_set_on_both_sides` asserts SET EQUALITY between the file's `refusal_rows` and `Row::ALL`, so retiring the row demands a file byte change; invariant 1 allows exactly one. And `row_window_not_in_build` runs the binary, so it cannot survive the flip. **Everything the four tasks promised is in it** — inventory members 1 (`SHIPPED = true`), 2 (`carriage()` → `Outcome::AsDecides`; the four `md1-split/*` rows now exit 2, verified by hand on `md1-split/fixed-index`), 3 (`window_remedy()` → §6's own remedy; both row texts verified by execution: *"Use `--as descriptor`, which carries `/0/*` exactly."* and *"… carries `/<0;1>` exactly."*), 4 (padding — split out to `df00632`, see P2-6), 5 (clap help un-marks — 0 hits), 6 (`identify::window_refusal` deleted), 7 (`Row::WindowNotInBuild` gone from enum + `ALL` + `slug()`, both producers gone), 8 (§11 item 5's matrix is the full-build truth table with case 3's new witness and a sixth case pinning the 3 → 2 flip), 10 (comments reworded) |
| P2-2 | `refusal_rows.json`'s 35 surviving entries RENUMBERED | **SOUND, and verified** | I zipped the renumbered entries against §6's live table in document order: 35/35 align, 1..35 contiguous. Leaving the old numbers would have made every entry after row 9 a false annotation |
| P2-3 | `goprobe/go.mod` committed pointing at `/scratch/…/sh-worktrees/s2-descriptor-arm` | **SOUND for now, decay risk REAL** — see Minor M-3 | it is the rev the corpus was measured at, which is what makes the reproduction command reproduce; but the worktree does not survive P3's merge, whereas the previous target `_work/seam-fork` still exists. Failure mode is LOUD (missing path → build error), not silent |
| P2-4 | `gen.py` edited (not only `rows.py` + `comment.json`) | **SOUND and necessary** | "a retired column a generator can still emit is a column that comes back" — the `sysw_class` emission at `gen.py:192-194` is removed; the other two edits repoint measurement-rev comments. F-428's fix was already in the generator's scope |
| P2-5 | §11 item 4 amended, not on the brief's P2.7 list | **SOUND** | item 4 said *"the `--as descriptor`-only rows among them are S2's"* — a set S2 measured as EMPTY while subtracting a row. Machine-checkable and machine-checked (35). Leaving it is the "a diff falsifies text it never touches" class |
| P2-6 | the four-format `show` test landed at P2.5, not P2.3 | **SOUND** | the brief names it under both; it needs P2.5's code. P2.3 (`2e88da4`) carries the distinct pack-side half — one record, byte-equal to the measured canonical, label warning named two-sidedly |
| P2-7 | F-428 is "one annotation + the fork's copy", not two annotations in one repo | **CORRECT, verified** | `parse.go:151` at base occurs exactly twice in this repo: the vector file's `source` and `rows.py`'s generator line for it (`gen.py` never carried it). Both now say `:158`; the third site is the fork's copy of the same JSON, which P3.3 takes with P2.6's bytes |
| P2-8 | `:158` vs `:161` | **NOT a defect — the annotation is rev-qualified** — see Nit N-2 | measured: the count-mismatch `return` is `parse.go:158` at fork `main` `a5e29b4`, `:161` at `0abbf81`, and **`:158` at `1f09537`, the rev the annotation itself names** (*"measured at fork 1f09537 … parse.go:158 fires before the Title gate at :37"*; `:37` confirmed). Because the citation carries its own rev, it does not go stale when P3 merges |

---

## Findings

### Critical — none.

---

### IMPORTANT

#### I-1 — §5.1's S2 amendment asserts the choice block is printed "verbatim"; it is not, and the test that claims to pin §5.1's normative text pins the code's text instead

**Where.** `design/SPEC_descriptor_input.md` §5.1 (branch `dbcd6b0`, head
lines 848-851 — the P2.7 amendment inserted by `0c130ba`), and
`crates/me-cli/tests/descriptor_refusals.rs:200-294`
(`the_choice_block_is_section_5_1s_normative_text_verbatim`).

**The amendment, added by P2.7:**

> … (R0 r9's M6, **and since S2 no build state reaches it: both values ship and
> the block, and `me sysw pack --help` with it, marks nothing**; the block above
> is what S2 prints, **verbatim**, with each description INLINE on a head padded
> to the description column), …

**Counterexample.** Extract §5.1's fenced NORMATIVE block from the branch spec,
strip the `me: ` prefix, and diff it against what the binary actually writes:

```
$ me sysw pack --no-passphrase --in <formats-happy/bip380-sortedmulti-multipath> --out /dev/null
--- SPEC §5.1 NORMATIVE block
+++ me actual output
@@ -10,8 +10,8 @@
                         (substitutions -- a missing or extra strike is not
-                        correctable), so it can even be hand-stamped. Carries policies
-                        --as descriptor cannot. Restoring needs an md1
-                        decoder (an open spec; the tooling today is this
+                        correctable), so it can even be hand-stamped. Carries
+                        policies --as descriptor cannot. Restoring needs an
+                        md1 decoder (an open spec; the tooling today is this
                         project's).
-    They are not interchangeable — `me sysw pack --help` has the comparison.
+    They are not interchangeable -- `me sysw pack --help` has the comparison.
```

**Four lines differ** — three wrap positions in the `--as md1` arm and an
em dash where the code emits `--`.

**Why it is a defect and not a nit.**

1. §5 is headed **NORMATIVE**, and this is the one sentence in the document
   that promotes the illustrative block to a byte-exact claim. It was not
   there before S2 — `git show 0144f02:design/SPEC_descriptor_input.md` has the
   same block text with no verbatim claim — so P2 introduced the falsehood,
   not the divergence. (The divergent lines are pre-existing in the `--as md1`
   arm, which S2 never touched; the `format!` diff at `gate.rs:608-627` changes
   only the descriptor head's line join.)
2. The test added by the same phase is named
   `the_choice_block_is_section_5_1s_normative_text_verbatim` and documented
   *"**§5.1's NORMATIVE block, VERBATIM — the whole thing, not its first
   line.**"* Its `BLOCK` literal is the **code's** text. So the test cannot
   detect the divergence it is named for, and the spec cannot either: a future
   author who "fixes" §5.1's block reds nothing, and one who reads §5.1 as the
   authority produces a renderer this suite rejects. This is precisely the
   shape P2.2 existed to remove (a claim nothing can falsify), reintroduced one
   layer up.
3. It is cheap to close, in either direction, and the choice is an editorial
   one for the controller: **(a)** re-wrap §5.1's `--as md1` arm and change the
   final line's `—` to `--` so the block IS byte-equal, and the test's name and
   literal both become true; or **(b)** drop the word "verbatim" from the
   amendment, rename the test to
   `the_choice_block_lays_its_descriptions_out_in_section_5_1s_column`, and say
   in its doc that §5.1's fence is illustrative of layout, not of wrapping. (a)
   is stronger and is what the plan's M2 intent reads as.

**Not Critical because:** no wrong result reaches an operator — the block's
content, exit code, column layout and build-marking are all correct and
measured — and no downstream consumer (the Go port, the device) renders this
block. Under the brief's literal rubric ("a spec amendment asserting something
false") it reads Critical; I place it at Important because what S2 introduced
is the *claim*, while the divergence is in untouched pre-existing text. It
gates either way.

---

### MINOR (recorded, not gating)

#### M-1 — IMPL-S2-P2's sweep table states a guard that does not exist; the retirement is nonetheless guarded, by a different mechanism

`design/agent-reports/IMPL-S2-P2.md`, propagation-sweep table, `sysw_class`
row: *"The `KNOWN_ROW_KEYS` entry was REMOVED, so a returning column also reds
as an unknown key."*

Measured: `crates/me-cli/tests/descriptor_seam.rs:91` still lists
`"sysw_class"` in `KNOWN_ROW_KEYS`, and the diff never touches that array. A
returning `sysw_class` column would **not** red as an unknown key.

It would still red — `POP.sysw_class = 0` is asserted at
`descriptor_seam.rs:527` (`assert_eq!(count("sysw_class"), POP.sysw_class,
"sysw_class")`), and the sha pin at `:163` reds first on any regeneration. So
the retirement holds; only the report's account of *why* is wrong. Recorded
under the standing "records are the weak half" rule. Fix is one word in the
report, or actually remove the key so both guards exist.

#### M-2 — two S2 amendments describe the Go half in the present tense, one phase early

`design/SPEC_descriptor_input.md:1619` and `comment.json:54` (shipped verbatim
into the vector file's `_comment`): *"**Both suites now assert** the classifier
EXHAUSTIVELY …"*. §11 item 1's amendment likewise: *"the device side is
exercised by the Go test's DERIVED rule"*.

At `dbcd6b0` the fork's `nonstandard/descriptor_seam_test.go` still has
`wantRows = 71`, `wantDeviceAbsent = 1`, `wantSyswClass = 4` and
`TestDescriptorSeamSyswClass` counting the retired sample against its own
unchanged copy (`sha256 542cd492…`). The Rust suite asserts the derived rule;
the Go suite does not yet.

This is scheduled, not smuggled: the plan assigns the `sysw_class` paragraph to
P2.7 (r2 I1) and lands P2.7 at P2's close, and invariant 1 states the two
copies "transiently differ". P3.3 closes it. Worth a hedge ("both suites assert
… — the Go half lands at P3.3") only if the controller wants the branch to be
readable standalone; P3's gate makes it true either way.

#### M-3 — `goprobe/go.mod`'s `replace` names a worktree that P3's merge deletes

`scripts/descriptor-seam-vectors/goprobe/go.mod:28` →
`/scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm`. The previous target,
`/scratch/code/shibboleth/_work/seam-fork`, still exists on this box; the new
one is a scratch worktree with a defined end of life, so this is a net increase
in decay risk even though it is the *truthful* pointer for the corpus as
measured (and it is what let me reproduce all 72 booleans today).

Mitigations already present: the README's Baselines section and
`comment.json`'s PROVENANCE both name fork `0abbf81` and
`descriptor-mnemonic 6c4a56fd`, and the failure mode is a loud build error
rather than a silent regeneration against the wrong tree.

**Owning phase: P3.** Re-point at fork `main` once P3.1 and P3.4 merge, and
update the two provenance paragraphs in the same commit. The implementer
flagged this; it needs a follow-up entry so it is a grep rather than a memory.

---

### NIT

#### N-1 — one new assertion is self-referential

`crates/me-cli/tests/expect_kinds.rs`,
`expect_descriptor_still_refuses_a_mnemonic_only_container` asserts
`r.err.contains(Kind::Descriptor.describes())`. A `describes()` returning `""`
would satisfy it. The test's load-bearing assertions (`code == 4`,
`!out_exists`) do bind, and the description string is pinned by execution
elsewhere, so this changes nothing — but the clause reads as a text pin and is
not one.

#### N-2 — what P5.1 should record about F-428

Do **not** re-base the `:158` citation on merged fork `main`. The annotation
reads *"measured at fork `1f09537` … `parse.go:158` fires before the Title gate
at `:37`"*, and I confirmed 158 is the count-mismatch `return` **at
`1f09537`** (it is also 158 at `a5e29b4`, and 161 at `0abbf81` because P3.1
adds three comment lines above it). A rev-qualified citation does not decay.
P5.1 should close F-428 as *fixed, rev-qualified to `1f09537`*, and note the
`:161` value only as the reason a naive re-check disagrees.

---

## What this review did NOT cover

- The plan's design (7 R0 rounds; out of scope by the brief).
- The fork half: P3.1's parse fix and P3.4's `ypubVer` arm sit unreviewed at
  `0abbf81` and are P3's review, as the plan says. I ran them as an
  instrument, and their two outputs reproduce; I did not read them as code.
- The Go seam test's un-skip (P3.3) and the fork's byte-mirror of the vector
  file — both still pending, both P3.
- §11 item 6 (a `ClassDescriptor` record displayed on the real device) —
  operator-gated, unchanged.
- The P1 report's classify-cost table (+107 ns/record on the corpus, ×34 on
  descriptor inputs). Not re-measured; the shape is plausible and the
  conclusion — a real payload pays microseconds — is not load-bearing on
  anything in this diff.

---

## Worktree state at exit

```
$ cd /scratch/code/shibboleth/me-worktrees/impl-descriptor-s2
$ git rev-parse HEAD
dbcd6b09325952d27b4fec978aeca7391620ddf0
$ git status --porcelain --untracked-files=all
   (empty)
$ git diff HEAD --stat
   (empty)
$ cargo nextest run --locked -E '<the six tests I mutated against>'
   6 tests run: 6 passed, 574 skipped
```

All four mutations reverted. The fork worktree is unchanged at `0abbf81`
(`git status --porcelain` empty) and was never written; `goprobe` was copied to
scratch and built there. Nothing pushed. Nothing committed.
