# H6 plan-round fold — spec + plan, after the build gate

**Agent:** fold author (opus). **Brief:** `design/agent-briefs/hashlock-H6-plan-gate-fold-brief.md`.
**Artifacts edited:** `design/SPEC_hashlock_H6_preimage_plates.md` (R0 GREEN at
engrave `a67a3924`) and `design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md`
(engrave master `55950604`, working tree). **Inputs:**
`design/agent-reports/hashlock-H6-plan-author-report.md` §4 (nine findings) and
`design/agent-reports/hashlock-H6-plan-gate-8b-12.md` §6 (seventeen fixes), plus
the three gated trees `/scratch/code/shibboleth/.tmp/h6-{gate,ms,me}`.

**VERDICT: FOLDED, 26 of 26, none declined.** All nine author findings and all
seventeen gate fixes are folded — twenty-one as normative spec text, five as
plan-only block corrections that touch nothing normative, and the gate's two
deliberate non-changes as recorded follow-ups with their measurements.
`./scripts/h6-plan-blocks-vs-tree.sh` reports **86 blocks checked, 0 FAIL**. Six
mutations were re-run in private copies of the trees and every one went RED.
**Nothing was committed. No repo checkout and no gated tree was edited.**

---

## 1. What was measured, and how

**Every number written into either document is this author's own measurement**,
taken in private copies (`/scratch/code/shibboleth/.tmp/h6-fold-fork` from
`h6-gate`, `h6-fold-me` from `h6-me`, and a fresh `git ls-files | tar` copy of
fork `fb0dd04` at `h6-fold-fork-base`). Nothing was carried from either report.

| what | instrument | result |
| --- | --- | --- |
| the four new modal bodies | `assertModalBodyFits` on the production renderer | §5.1 preimage confirm **274 / 186**; §5.3 plate refusal **99 / 455**; §5.2 flow abort **80 / 476**; §5.2 empty-payload refusal **46 / 513** |
| every carried modal figure | the same | 75/476, 86/476, 185/360, 186/360, 107/455, 126/378, 165/397, 204/302, 184/378 — **all re-confirmed to the character** |
| `Which hash?` rows | `widget.Labelw` in `composerTextBand` | `(in payload)` **41 chars / 343 px / 1 line**; preimage row 31/276; phrase-record row 43/336; band **411 px**, label **23 px**, content box **224 px** |
| `Which hash?` page capacity | `plateHitPoints` on the drawn screen | six rows → **page 1 draws 5 touch targets**, `No hash lock` on page 2 |
| the pick step | the same | rows **128 / 138 / 180 / 196 px**, all one line; **4 of 4 rows on page 1** with the two-line lead |
| the census geometry | `composerRowSize` at both wraps | panel **480**, nav column **427**, `confirmReviewScreen` wrap **464**, centred-safe **374**, band **411**, shipped completeness line **459**; the five new rows **419 / 385 / 434 / 442 / 441** at the wrap and **387 / 385 / 404 / 383 / 387** in the band |
| census paging | `composerPageLines` | **11 lines over 3 pages**; §8.3's heading on page 2, the apart-storage line on page 3 |
| the plate | `CharsPerLine` / `LinesPerPlate` / `fixedCharWidth` / `hashlockLayoutFor` | rungs 19/23/26/31/34/39 chars, advances 4.0000/3.3333/2.9333/2.5333/2.2666/2.0000 mm; budget **416,000 units = 65.00 mm**; QR envelope **31.80 mm** at scale 2, **47.70** at scale 3; method line **73 chars**; worst case **10 rows / 30.00 mm text / 63.80 mm total / 1.20 mm spare** |
| the body-row ORDER | `hashlockSegments` + `hashlockLayoutFor` | method (2 rows), blank, three locator rows, blank, phrase (3) = **2 + 1 + 3 + 1 + 3 = 10** |
| the budget arms | `constantTimeQRModules` | **843 / 1013 / 1199 / 1399**; shipped 171/266/391/547/684 untouched |
| the corpora | `sha256sum`, `json.load` | class `3575ccb0…a1c4abaf` / **68 rows** (was `5b3960ca…b312` / 47), pinned identically in `sysw_composer_records.rs:399` and the fork's provenance file; ms `4f1819cd…aba21d4` identical in both copies (fork pin at `fb0dd04` is `a46c197a…11d30`); seam `2c2fbb3f…6bd541b` **unchanged** |
| the copy bodies | set difference of `^func composerCopy*` against `fb0dd04` | **20 added, 13 blockquoted, 7 not** |
| the firmware | `nix develop -c tinygo build -size short …` | pristine `fb0dd04` **1,599,208 / 62,856**; wired **1,643,580 / 63,272**; delta **+44,372 B / +416 B**; the plate layout **4,560 B** of it (stubbed: 1,639,020 B) |
| the join guard | `TestComposerEveryScreenFunctionHasAProductionCaller` | **PASSES** on the fully wired tree, logging only the pre-existing `composerDescriptorCeilingChars` exemption |

**One number is cited rather than re-run and it is labelled as such in the
spec:** the four 32-minute fuzzing campaigns (43,458,059 payloads). Re-running
them is 128 minutes of wall clock and would not make them more true; what this
round measured instead is the outcome the spec now states — the four arms in the
gated tree, and that the in-suite fuzzed row passes against fresh content.

---

## 2. The mutation re-runs — six, all RED

Each was applied to a private copy, run, and reverted; the tree was re-run green
afterwards and `diff -rq` against the gated tree reports no difference.

| # | mutation | tree | failure, verbatim |
| --- | --- | --- | --- |
| M1 | grow the method line to **79 characters** (spec §11.4's replacement) | fork | `3.0mm: fits = false (11 rows, 427520 units against 416000), want true` … `EngraveHashlock(the worst case): backup: the hashlock plate does not fit at any font size: 11 rows at 3.0mm need 427520 units against a budget of 416000` |
| M1b | the CONTROL for M1: 74, 75, 76, 77, 78 characters | fork | **five runs, five `ok`** — the spec's original *"add one character"* mutation demonstrably cannot fail |
| M2 | `return 1379 + 20` → `return 1378` (spec §11.3's new PIN) | fork | `constantTimeQRModules(53) = 1378, want 1399 (1379 observed + 20 buffer). The original was derived by fuzzing: 7,759,282 in 32 min, converged with 22.6 min of quiet. A different number needs its own campaign.` |
| M2b | the CONTROL for M2: the same mutated budget under the FUZZED row | fork | `ok seedhammer.com/engrave 1.472s` — the in-suite sample cannot carry §11.3's original mutation, which is the finding |
| M3 | drop `preimage_plate_admissible`'s `33 bytes beginning 0x03` conjunct | me | `a_preimage_plate_is_named_not_misdiagnosed` … `left: Err(PreimageNotAdmitted(0, Preimage))` / `right: Err(Unclassifiable(0, PreimagePlate))` |
| M4 | compare the id with `eq_ignore_ascii_case` | me | the same assertion, same values — the UPPERCASE plate becomes a CLASS |
| M5 | choose §10.1's fourth arm on `at least one` phrase | fork | `a mixed composition claims to hold the phrase for each path: "HASH ON EVERY PATH\nEvery way to spend this wallet needs a hashlock preimage. This composition holds the phrase and method for each one …"` |
| M6 | draw the census through `confirmReviewScreen` | fork | `composerEngraveStep does not draw its census through composerReadScreen, which is the only paged confirm that wraps inside composerTextBand` … `every census row over 374 px has its right edge under a navigation button (W-3)` |
| M7 | give the pick lead FOUR lines | fork | `the pick screen draws 3 tappable rows on page 1` … `page 1 draws 3 of the four rows; the lead is spending them` |

Post-mutation green: `go test ./engrave/ ./backup/ ./codex32/ ./sysw/ ./hashlock/`
— **all ok**; `go test -run 'TestComposer|TestWhichHash|TestHashlockPlates|TestSyswWarn|TestSyswNotice' ./gui/`
— **ok**; `cargo nextest run --locked -p mnemonic-engrave -E 'test(a_preimage_plate_is_named_not_misdiagnosed)'`
— **1 passed**.

---

## 3. The nine author findings, per document

Numbering is the author report's §4, which the spec's `## Plan-round fold` and
the plan's `## Build gate folded here` now both use — the plan's own list was
**renumbered** to match, so the same finding carries the same number in all three
documents.

| # | SPEC | PLAN |
| --- | --- | --- |
| 1 the phrase rule is unreachable from `me` | new §1 item 3b; §3.1 gains a three-part NORMATIVE block (the move into `ms-codec`; a PUBLISHED **0.9.0** via `RELEASE_PROCESS.md`, whose item 1 forces `0.X+1.0` on a corpus-sha change, with the H1 `## ms-codec [0.8.0]` entry as precedent; then the `Cargo.toml:53` bump); new §12 item **0** | STATUS + `## Build gate folded here` index; Task 1/Task 2 unchanged in substance |
| 2 the method-line mutation cannot fail | §11.4's mutation replaced by the **79-character** threshold, with the 74–78 control stated | Task 6 Step 7 reworded from "this plan replaces it" to "the spec has since folded it" |
| 3 the budget mutation cannot fire | §11.3 splits into the fuzzed row (bounds fresh content) and a **PIN** (proves the entry was derived); §7.2 item 4 records the campaign; §16's "not settled" item **closed** | Task 4 Step 3 reworded the same way |
| 4 `preimage_plate_admissible` needs three conjuncts | §4.3 gains a conjunct table and the Rust-primary-in-reverse reasoning; §11.1 gains both mutations | Task 2 Step 4 reworded |
| 5 the classes are BEARER → argv | **new §3.6**; §12 item 3 rewritten to `--in` | Task 3 Step 10 reworded; its `main.rs:566` citation corrected to `:551` for the function, `:566` for the arm it replaces |
| 6 two shipped tests falsified | §14 gains a row naming both with their v10 expectations and the third rewritten record | Global Constraints reworded; `passphrase_test.go:775-791` corrected to **`:774-791`** (re-grepped) |
| 7 the join guard | **new §2.2 item 6** — one commit, and why the exemption table is not the instrument; §11.6 states it as a gate | Task 8a unchanged (it already carried the ruling) |
| 8 the body-row order | §6.5 states ONE order with the layout's segment list and `2 + 1 + 3 + 1 + 3 = 10` | Task 6 Step 2's arithmetic corrected from `3 + 1 + 2 + 1 + 3` (the losing order's) to the drawn one |
| 9 the seam corpus row | §4.2 gains a NORMATIVE corpus table with both repos' pins | Task 2 Step 7 unchanged |

**Finding 9 was refined by measurement, and the refinement matters.** The
controller's decision said "re-pin the corpus sha in both repos". Measured:
- the row that moves lives in `crates/me-cli/testdata/record_corpus_pre_s2.json`,
  a **`me`-only capture with no fork twin**;
- what IS re-pinned on both sides is the CLASS corpus (`3575ccb0…`, 68 rows) and
  the MS corpus (`4f1819cd…`), and §4.2 now tables both;
- the two-repo `codex32_seam_vectors.json` **does NOT move**, because its
  `device_admits` column is `Classify(s) == ClassCodex32Secret` and a preimage
  plate is `ClassPreimage`, so both columns stay `false`. Its sha
  `2c2fbb3f…6bd541b` is a literal in `crates/me-cli/tests/codex32_seam.rs:25`
  and `sysw/codex32_seam_test.go:30`. **The spec now says so explicitly**, so an
  implementer reading "re-pin the seam corpus" does not edit a file whose only
  effect would be to red both suites.

---

## 4. The gate's seventeen fixes, per document

Five changed only the plan and nothing normative — **F1** (`sysw.Record` does not
exist), **F2** (an uncompilable Step-3 sketch), **F7** (two signatures), **F16**
(three test paths that do not exist), **F17** (an annotation inside a cited
fragment). The other twelve are spec text now:

| # | spec section | what it says now |
| --- | --- | --- |
| F3 | §5.1 | the payload preimage record gets its OWN confirm body; **274 / 186** |
| F4 | §5.1 | `composerHashRows` gains `st *composerState`, answered out of `hashlockHeld` |
| F5 | §5.1 | the `(in payload)` annotation is exact for a preimage record, true for a `phrase:` record once derived |
| F6 | §5.1, §11.5, §13 | **five rows**, 29 px pitch, 224 px box; filed as a follow-up |
| F8 | §8.6 rule **4a** | `hashlock.MethodLine` / `hashlock.QRText` are deliverables, built from the constants, pinned against the corpus; the hardened line is **73 characters** |
| F9 | §5.4 | §8.4b's cut counter, without which the arm cannot fire |
| F10 | §5.3 (B), §8.3's heading, §11.5 | `composerReadScreen`, not `confirmReviewScreen`; the gate is an AST assertion |
| F11 | §5.3 item 2, §8.3, §16 | the widths are digest-dependent; 405/423/441/447/448 **withdrawn** (and the withdrawal annotated in §16's own record of round 0) |
| F12 | §10.1, §11.5 | the fourth arm's predicate is `every`, not `at least one` |
| F13 | §5.3 step (A), §11.5 | the masked lead is TWO lines |
| F14 | §5.3, §11.5 | the plate order is deterministic |
| F15 | §5.2 | the predicate takes the SESSION; the door's caller `gui/wallet_policy.go:46-58` must dispatch; the `:93-97` citation corrected to `:86-90` |

**The gate's two deliberate non-changes are recorded, not reversed.** §8.3's
`plate(s)` stays verbatim in both documents and is filed. The bodies with no §8
blockquote are tabulated in a **new §8.9** — measured as **SEVEN of the twenty**
this stage adds (four modals with their headroom, three picker leads) — and
filed. The gate's own note said "four items"; the set difference against
`fb0dd04` makes it seven, and the door's preimage-count lead is the one the
narrative list missed.

**Also folded, each measured:** §7.2 item 4 and §16 (the campaign result, the
"not settled" item closed); §11.6 (the firmware table and the layout probe);
§11.7 (the walk's second keyed path, because `md.Compose` refuses a wholly
key-less composition — `md/compose.go:93` — and the census's 3-page shape).

---

## 5. Gates run

| gate | before | after |
| --- | --- | --- |
| `./scripts/h6-plan-blocks-vs-tree.sh` | 86 blocks, 0 FAIL | **86 blocks, 0 FAIL** |
| `plan-glyph-check.sh` (plan) | 4 undrawable, all pre-existing | **4**, the same four |
| `plan-glyph-check.sh` (spec) | 8 undrawable | **8**, the same eight |
| `plan-table-check.sh` (plan) | 1 malformed (the 8a boundary row; pipes in a code span, the checker's named blind spot) | **1**, the same row |
| `plan-table-check.sh` (spec) | 0 malformed | **0** (one row of my own tripped it and was rewritten) |
| fork sweep in the fold copy | — | `engrave backup codex32 sysw hashlock` **all ok** |
| the touched `gui` tests | — | **ok** |
| `a_preimage_plate_is_named_not_misdiagnosed` | — | **1 passed** |

`git status` shows exactly two modified files: the spec and the plan.
**Nothing committed.** The checker script needed no change — the header
convention was already right.

---

## 6. What this fold did NOT do, stated because a gate that hides its blind spot is worse than no gate

- **The fuzzing campaign was not re-run** (§1 above). Cited, labelled in the spec.
- **The retention-only join-guard red was not reproduced.** Reverting Task 8b out
  of the tree is a large edit with its own defect surface; the red was measured
  twice already (author, then gate) and what this round measured is the other
  half — that the guard PASSES on the wired tree. The spec's §2.2 item 6 says
  which half is whose.
- **No hardware.** Every screen number is `sh2DisplaySize` in the harness;
  §12 item 8's QR scan is still unrun and still named in §16.
- **The emulator walk was not re-run.** Task 12 ran it four times; this round
  only added the two spec facts the run exposed (the second keyed path, the
  paging).
- **The `ms` tree was read, never mutated.** Task 1's release is a publish, so the
  scratch tree still reads `ms-codec 0.8.0` and `me-cli` still reads
  `ms-codec = "0.8"` behind a `[patch.crates-io]`; the spec now states the bump
  as §12 item 0 rather than assuming it.

## 7. Trees left in place

| tree | what it is |
| --- | --- |
| `/scratch/code/shibboleth/.tmp/h6-fold-fork` | private copy of `h6-gate`; `diff -rq` against it is EMPTY (all mutations reverted) |
| `/scratch/code/shibboleth/.tmp/h6-fold-me` | private copy of `h6-me`; `diff -rq` against it is EMPTY |
| `/scratch/code/shibboleth/.tmp/h6-fold-fork-base` | a tracked-files-only copy of fork `fb0dd04`, unmodified — the firmware baseline |
| `/scratch/code/shibboleth/.tmp/h6-fold-fw-{base,wired,stub}.log` | the three `tinygo -size short` outputs |

The stubbed-layout probe tree was **deleted** after its measurement, so no
deliberately-broken tree is left sitting beside the gated ones.

**No `.jsonl` was read. No sub-agent was dispatched. No repo checkout and no
gated tree was edited. Nothing was committed.**
