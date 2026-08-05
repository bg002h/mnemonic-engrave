# R0 review — IMPLEMENTATION_PLAN_sizeproof.md — round 0

Two lanes dispatched 2026-08-05 against the P0 plan @ `7533c02`: opus
sequencing-adversarial + sonnet spec-vs-plan coverage, merged by an independent
opus synthesiser. The SPEC was declared closed in the brief; both lanes stayed
on the plan and neither raised a spec finding. Persisted VERBATIM before folding.

VERDICT: **RED** — 0C / 4I / 3 Minor / 2 Nit

Synthesiser's judgement: *"NOT ready to execute, but very close — a good plan
with four small, local defects and no structural fault."* 19 of 20 spec test
items and every §3 site already have an owner.

---

## MERGED GATE VERDICT (synthesiser)

VERDICT: RED — 9 findings

### 1. Important — §1 P2 gate / P3 gate — spec §7 item 7(b)

**Spec §7.7(b) — the two-block-plus-QR fixture that regression-pins the round-1 Critical — is scheduled by no phase, and the phase whose change re-opens that defect (P3) gates on nothing that can see it.**

*Failure:* Both lanes found this independently; I verified it. Grepping every gate line in the plan: P1 = §7.1, §7.15; P2 = §7.1, §7.7(a), §7.7(c), §7.7(d); P3 = §7.1, §7.2, §7.8, §7.11, §7.20; P4 = §7.5, §7.6, §7.9, §7.14; P5 = §7.10, §7.13, §7.16, §7.17, §7.18; P6 = §7.3, §7.4, §7.12, §7.19. P2 enumerates §7.7's sub-items and skips exactly (b). It appears nowhere else. Spec §7 item 7(b) reads: "a TWO-BLOCK plate with a QR wraps block 2's rows at the code's own budget, and no body ink enters the code box — the round-1 Critical, unreachable by any single-block fixture." The gap is not cosmetic. In P2 the band is plate-absolute while every block still lays out at baseY = params.I(outerMargin), so block 2's window is right by construction and (b) would pass vacuously if written there. P3 is the phase that gives each block its own running y and re-indexes rows from 0 within a block — precisely the combination in which a block-relative index re-creates the measured 12-vs-36-column defect and engraves body ink across the code. No shipped golden is a multi-block QR plate, the compiler cannot see it (int to int), and none of P3's five gate items exercises a QR on a two-block plate. P3 can therefore close green with the original Critical reintroduced: a gate that cannot catch what its own phase changed.

*Fix:* Write the §7.7(b) fixture in P2 (pin it by MODULE COUNT, per the spec's own warning) and add "§7.7(b) a two-block plate with a QR wraps block 2 at the code's budget; no body ink in the code box" to P3's gate list. P3 is where it must go RED-then-green; P2 is where it is cheap to build.

### 2. Important — §1 P1 content vs §1 P2 content

**P1 declares the `qrAt *qrPlacement` field, populates it in two constructors, and adds §2.1.1's panics — but the plan puts the `qrPlacement` type and `qrPlaceAt` in P2. P1 as written does not compile.**

*Failure:* Opus lane; verified against the spec. Plan P1: "`Fitted` gains `Mixed`, `Sizes`, `TitleSizeMM`, `FooterSizeMM`, `qrAt` (§2.3); **both** legacy constructors populate them ... Guards added: ... and §2.1.1's two panics." Spec §2.3 declares the field as `qrAt *qrPlacement`; §2.3's constructor rules for both `fitBlocksAt` and `EngraveFreeText` are "sets `qrAt` from `qrPlaceAt` at `anchorY = outerMargin` when there is a code"; §2.1.1's first guard is `(QR == nil) == (qrAt == nil)` and its third is `qrAt.Bottom <= plateHeight - margin`. All of that needs the type and the constructor function. Plan P2 states "`qrPlacement` + `qrPlaceAt` (§2.1)" — one phase later. The contradiction resolves only by pulling them into P1, because the alternative (defer `qrAt` to P2) strips P1 of one of its "two panics" and of half of §2.3's constructor obligation — the exact property P1 exists to establish. A phase-boundary error in the document whose stated job is fixing phase boundaries.

*Fix:* Move `qrPlacement` and `qrPlaceAt` into P1's content line; leave P2 as "`textLayout` takes a `*qrPlacement`; `lineLayout` carries `qrTop`/`qrBottom`; `EngraveFitted` reads `f.qrAt`, `EngraveText` its own local; the producer table's anchors". Also state in P1 that `fitBlocksAt` gains the `qrAt.Bottom <= plateHeight - margin` ERROR return (§2.1.1 row 3), since P2's gate already tests it via §7.7(d); "two panics" is the wrong count for what P1 actually lands.

### 3. Important — §1 P1 — "Nothing reads the new fields yet" vs backup/blocks_test.go:406-418

**P1's new `len(Sizes) != len(Lines)` guard trips an existing test that hand-builds a `Fitted` outside both legacy constructors and does NOT expect a panic, so P1 cannot close on the whole-suite-green criterion the plan sets.**

*Failure:* Both lanes; I read the source. `backup/blocks.go`'s `EngraveFitted` is where the existing `len(f.Faces) != len(f.Lines)` panic lives, so §2.3's mirror guard lands there too — in the engraver, not the constructors. `backup/blocks_test.go:406-416` (`mk`, inside `TestEngraveFittedInsetsEachRowInItsOwnFace`) returns `Fitted{SizeMM: size, Lines: lines, Faces: faces, TitleFace: faces[0], FooterFace: constant.Font}` with `len(Lines) == rows` and no `Sizes`, and lines 417-418 call `EngraveFitted(P, mk(...))` twice with no `recover`. P1's guard fires and the test fails outright. Plan §2 requires `go test ./...` green for a phase to close, and P1's own rationale ("pure addition", "Nothing reads the new fields yet", "a moved golden here means the constructors were filled wrong, with nothing else in the diff to hide behind") is false as stated. No phase is assigned the fixture update. This test is also the only existing pin on the per-row inset — the exact property §2.5's `at(0)` rewrite endangers in P3 — so it must be repaired, not weakened. (The three GUI literals the spec names, gui/freetext_flow_test.go:564/893/928, are confirmed safe: they feed ftConfirmBody/ftConfirmSummary only and never reach EngraveFitted.)

*Fix:* Add an explicit P1 task: "backup/blocks_test.go:414's `mk` helper and :298's literal gain `Sizes`; audit for any other hand-built `Fitted{}` in the backup package outside the two constructors. The per-row-inset assertions in :414 keep their exact form."

### 4. Important — §0 line 31 vs §3 heading (line 126)

**The plan contradicts itself on when the mandatory, non-deferrable two-lane whole-diff review runs: §0 says "after P5", §3 is titled "After P6" and places the same review there.**

*Failure:* Sonnet lane; verified by reading both lines. Line 31: "**Review:** opus + sonnet two-lane on the whole diff after P5, per the operator's 2026-08-05 choice." Line 126: "## 3. After P6", whose item 1 is "Two-lane adversarial review over the whole diff (opus + sonnet) ... Non-deferrable". P6 is a real phase with substantial new content — §7.3's two composition tables, §7.4, §7.12, and the §7.19 mutation pass that is the plan's own answer to false-passing tests. Dispatched literally after P5, the one gate the plan calls non-deferrable never sees the phase that decides whether the back's 2.400 mm of spare is proven or asserted. The two statements cannot both be executed; the plan must say one thing, and this is the gate protecting a permanent-steel change.

*Fix:* Change line 31 to "after P6" to match §3. If a P5 checkpoint review is genuinely wanted in addition, §3 must say so and say which one is the gate.

### 5. Minor — §1 P1 — backup/blocks_test.go:296-309

**`TestEngraveFittedRefusesAFaceMapThatDoesNotCoverTheLines` also omits `Sizes`; it survives P1 only because it already expects a panic, and then passes for the wrong reason.**

*Failure:* The literal at :298 is `Fitted{SizeMM: 3.0, Lines: []string{"one","two"}, Faces: []*vector.Face{sh.Font}}` and asserts only `recover() != nil`. If the Sizes guard is evaluated before the Faces guard, the test stays green while no longer exercising the invariant it names. §7.19's mutation pass is scoped to "every test added in P1-P6", so this pre-existing test is outside it and the silent coverage loss would not be caught.

*Fix:* Give :298 a `Sizes` of the right length and check the Faces guard before the Sizes guard, so it keeps isolating what it names.

### 6. Minor — §1 P2 — unnamed forced call sites

**P2 names `textLayout`, `lineLayout`, `EngraveFitted` and `EngraveText`, but four other call sites and three test fixtures are compiler-forced to move in the same commit — including two the plan attributes to P3 and P5.**

*Failure:* `textLayout`'s callers are backup/fit.go:150 (`wrapBlocks`), backup/fit.go:327 (`faceLayouts.at`), backup/backup.go:359 (`EngraveText`), plus backup/blocks_test.go:30, :393 and backup/fit_test.go:15. `wrapBlocks` and `faceLayouts.at` must thread a `*qrPlacement` in P2 even though the plan books `wrapBlocks`' signature change in P3 and deletes `faceLayouts` there; `MaxCharsAtBlocks` becomes a placement producer in P2, a phase earlier than its §3 table row (P5). Removing `holeLines`/`qrLines` from `lineLayout` also breaks backup/engravetext_test.go:159-196 (a lineLayout literal with `holeLines: 2, qrLines: 19`, the current pin for the n<1 clamp) and backup/freetext_test.go:195-197, :289. All compiler-forced, so nothing goes silently wrong — but the clamp pin is being rewritten in the commit that changes its subject, with no instruction to preserve it.

*Fix:* Extend P2's content line to name `wrapBlocks`, `faceLayouts.at` and `MaxCharsAtBlocks` as forced placement plumbing (still one placement per plate at `anchorY = outerMargin`), and add: "engravetext_test.go's lineLayout literal and freetext_test.go's holeLines/qrLines derivations are re-expressed in qrTop/qrBottom with assertions and numbers unchanged."

### 7. Minor — §1 P3 — "both round-2's I2 and round-3's I2 live here"

**Round-3's I2 is split: its design half is P2's content by the plan's own P2 text; only its confirming test (§7.20 with `useQR`) is P3's.**

*Failure:* design/agent-reports/sizeproof-spec-R0-round3.md:29-37 has two parts — (1) add `AdmissibleBlocks` to §2.1's anchorY table, stating `anchorY` and `start` are deliberately different, which is exactly what plan lines 58-59 already put in P2; (2) extend §7.20 to pin with `useQR = true`, correctly P3's gate item. Both halves are covered by some phase, so nothing is lost; the narrative attribution is imprecise for a future reader reconciling phases.

*Fix:* Reword to "round-3's I2's PIN closes here; its design half is P2's."

### 8. Nit — §1 P2 gate — §7.7(d)

**P2's gate cites §7.7(d) whole, but the `FitSized` half cannot exist until P4 and is vacuous there.**

*Failure:* Spec §7.7(d) is worded against `fitBlocksAt`/`FitSized`. `FitSized` does not exist in P2, and P4's gate (§7.5, §7.6, §7.9, §7.14) never picks the remainder up. It costs nothing in practice — §2.7 gives `FitSized` no QR parameter and leaves `qrAt` unconditionally nil, so the Bottom check is structurally satisfied — but half a named item has no owning phase.

*Fix:* Write it as "§7.7(d), `fitBlocksAt` half; the `FitSized` half is vacuous by §2.7 (qrAt always nil) and is recorded as such rather than scheduled."

### 9. Nit — §1 P5 — "Every row of §3's table"

**Four rows of §3's table are already owned by P1 and P3, so the blanket claim makes follow-up reconciliation ambiguous.**

*Failure:* §3's table includes `EngraveFreeText` (assigned to P1), and `AdmissibleBlocks`, `rowFaces`, `MaxCharsAtBlocks` (assigned to P3 by name), then re-claims all of them for P5. Harmless for one implementer working in order, but the plan mandates per-phase follow-up reconciliation ("anything filed against a phase is burned down in or before that phase"), which a double-owned row makes ambiguous.

*Fix:* Say "every GUI/preview row of §3's table; the backup-package rows (EngraveFreeText, AdmissibleBlocks, rowFaces, MaxCharsAtBlocks) close in P1/P3."

### Notes

MERGE: opus raised 2I/2m/2n, sonnet 3I/2m. After dedup and adjudication: 4 Important, 3 Minor, 2 Nit. Verdict RED.

Dedup/adjudication log:
- §7.7(b) orphan — BOTH lanes, same defect. Merged into one Important. The lanes disagreed on the fix (sonnet: gate it in P2; opus: gate it in P3). I verified opus is right about WHERE it bites: in P2 every block still lays out at baseY = outerMargin, so the item passes vacuously; P3 is the phase that opens the window. Merged fix = write in P2, gate at P3.
- blocks_test.go:414 — BOTH lanes, split severity (opus Minor, sonnet Important). Adjudicated IMPORTANT. I read backup/blocks.go: the existing Faces guard lives in EngraveFitted, so the mirrored Sizes guard does too; :417-418 call EngraveFitted with no recover, so the package test fails and P1 cannot close on the plan's own whole-suite-green criterion. That is squarely "a phase cannot close as specified", not a missing sentence.
- blocks_test.go:298 wrong-reason pass — both lanes, kept separate as Minor (it stays green; the loss is coverage, and §7.19's mutation pass is scoped to tests ADDED in P1-P6 so it will not catch it).
- P1/P2 qrPlacement ordering — opus only; verified against spec §2.3 and §2.1.1. Kept Important, not Critical: it does not produce a wrong plate and the fix is to move two declarations, but P1 literally does not compile as written.
- "after P5" vs "After P6" — sonnet only; verified both lines. Kept Important: it is a direct self-contradiction about the timing of the one gate the plan calls non-deferrable, not a missing sentence.
- Opus's P2-call-sites Minor, opus's two Nits, sonnet's round-3-I2 Minor: carried through unchanged.
- DROPPED FOR SPEC SCOPE: nothing. Neither lane raised a finding against the spec's design; both stayed on the plan. Opus's §7.7(d) Nit brushes spec wording but is framed as a scheduling gap, so it stays as a Nit.

(a) COVERAGE TABLES

§7 item -> phase (from the plan's six gate lines, read verbatim):
 1 -> P1, P2, P3 (redundant, harmless) | 2 -> P3 | 3 -> P6 | 4 -> P6 | 5 -> P4 | 6 -> P4
 7(a) -> P2 | 7(b) -> **ORPHAN** | 7(c) -> P2 | 7(d) -> P2 (fitBlocksAt half only; FitSized half unowned, Nit)
 8 -> P3 | 9 -> P4 | 10 -> P5 | 11 -> P3 | 12 -> P6 | 13 -> P5 | 14 -> P4 | 15 -> P1
 16 -> P5 | 17 -> P5 | 18 -> P5 | 19 -> P6 | 20 -> P3
 Orphans: §7.7(b) only. 19 of 20 items cleanly owned.

§3 site -> phase (23 rows):
 EngraveFreeText -> P1. AdmissibleBlocks (own QR anchor) -> P2; AdmissibleBlocks (start translation), rowFaces, MaxCharsAtBlocks -> P3 (MaxCharsAtBlocks is additionally forced into P2 as placement plumbing, Minor). FitSized -> P4.
 P5 (blanket "every row of §3's table" plus named call-outs): ftFaceRun, ftPlan.Blocks, ftProof, ftProofOutcome, ftRungLabel, ftProofReplaces (both rows), ftFitAt, ftEvaluate/ftBuildPlate, ftFaceSummary, ftSizeLabel, ftConfirmSummary, gui.Preview, proofPreview, fittedPreviewAt, previewBuilders, ftProofLoader, ftProofOutcomeFor, sizeLabel.
 Orphans: none. Double-owned: 4 rows (P5 Nit).

(b) P1 BUILDABILITY — do the new guards trip an existing caller? YES, two, both in backup/blocks_test.go, both bypassing the two legacy constructors:
 - :406-416 `mk` builds Fitted with Lines/Faces and no Sizes, :417-418 engrave it with no recover -> the len(Sizes)!=len(Lines) panic FAILS the test. P1 cannot close. (Important above.)
 - :298 builds Fitted{Lines:2, Faces:1} with no Sizes but already defers a recover -> stays green, loses its Faces-guard coverage. (Minor above.)
 The three GUI literals the spec names (gui/freetext_flow_test.go:564, 893, 928) are SAFE — they flow only into ftConfirmBody/ftConfirmSummary and never reach EngraveFitted, so :893's QR with no placement does not trip §2.1.1's first panic either. gui/freetext_proof_test.go's Fitted mutation goes through proofFit, a production helper, so it is unaffected. No non-test caller builds a Fitted literal outside fitBlocksAt/EngraveFreeText.
 P1's rationale sentence "Nothing reads the new fields yet ... pure addition" is therefore false as written and needs the fixture task attached.

(c) CONTROLLER MUST DECIDE
 1. §7.7(b): confirm write-in-P2 / gate-at-P3 (my recommendation) versus gating it in P2 only. Gating only in P2 does not close the hole.
 2. The review timing: "after P5" or "after P6". Recommend P6 — P6 carries §7.3's composition pins and the §7.19 mutation pass, which is exactly the content a whole-diff review should see.
 3. P1 boundary: pull qrPlacement/qrPlaceAt into P1 (recommended) versus deferring qrAt to P2. The latter is coherent but costs P1 one of its two panics and half of §2.3's constructor obligation — decide explicitly rather than letting the implementer pick.
 4. Whether the §7.7(d) FitSized half is recorded as vacuous or scheduled into P4 (Nit, but it is a named test item with no owner).

(d) REPO CLEAN: yes. /scratch/code/shibboleth/seedhammer at 6d57681 ("gui(passphrase): rename the FONTPROOF! trigger to PASSPROOF!"); `git status --short` empty on entry and on exit. No probe files written by this lane; nothing to delete. Both upstream lanes also reported clean entry/exit with no probes.

(e) JUDGEMENT: NOT ready to execute, but very close — this is a good plan with four small, local defects and no structural fault. All four fixes are edits of at most three lines each: one gate item added to P3 (and its fixture booked in P2), two declarations moved from P2 to P1, one fixture task added to P1, one word changed on line 31. No phase needs re-cutting, no ordering changes, nothing is re-designed, and every §3 site and every §7 item but one already has an owner.
 Independently confirmed sound and NOT re-litigated: every phase boundary is compilable and whole-suite-closable apart from the P1 defects above (P4 leaves FitSized uncalled, which Go permits; P3 changes no public signature so gui and cmd/plateview compile untouched through P1-P4); P3 combining wrapBlocks' signature with EngraveFitted's row walk hides no golden move, because a wrong start/limit translation changes the line breaks and a wrong running y moves the ink, with no compensating pair that leaves the testdata bins byte-identical; P2's §7.7(a) survives P3 without rewriting, since for a uniform plate layout(baseY=margin).at(row) and layout(baseY=margin+row*fontSize).at(0) are the same comparison; P3 is correctly the phase carrying §7.20, because neither translation defect is expressible before it; P6-last is safe, since P5's §7.10 drives both proof sides end-to-end and FitSized errors rather than returning a Fitted when the composition overruns [start, limit], so the back's 2.400 mm of spare is proven non-negative at P5 and §7.3 only pins it against future font changes. The plan also correctly records the deliberately-untested multi-size QR guard by scheduling no test for it.
 Fold the four Importants and re-dispatch a scoped re-review ("did the fold fix each finding, did it introduce a new one") — not a fresh audit; the coverage tables above are settled and need not be re-derived.

---

## Lane 1 — sequencing-adversarial (opus)

VERDICT: RED — 6 findings

### 1. Important — §1 P3 gate (and the §7 cross-check)

**§7.7(b) — the two-block-plus-QR test that pins the change's original Critical — is assigned to no phase, and the phase it must gate is P3.**

*Failure:* Walking the plan's six gate lists against the spec's 20 test items, every item lands somewhere except §7.7(b). P2's gate names §7.1, §7.7(a), §7.7(c), §7.7(d) — an explicit enumeration of §7.7's sub-items with (b) skipped. P3's gate names §7.1, §7.2, §7.8, §7.11, §7.20. §7.7(b) appears in neither, nor in P4/P5/P6.

This is not a harmless omission, because (b) is the only pin for the regression window P3 opens. Today wrapBlocks (backup/fit.go:147-162) builds every block's layout at baseY = params.I(outerMargin) and hands WrapText widthFor(lay, row) with a PLATE-WIDE row index, so lineLayout.at's isQRLine (backup/wrap.go:135) is correct for block 2 by construction. In P2 the band becomes plate-absolute [qrTop, qrBottom) while baseY is still outerMargin for every block, so a second block's window is still right no matter what the implementer does — §7.7(b) written in P2 passes vacuously. P3 is the phase that gives each block its own running-y baseY AND indexes rows from 0 within the block; that is precisely the combination in which reaching for a block-relative row index re-creates spec §2.1's measured 12-columns-vs-36-columns defect, engraving body ink across the code. Neither a golden (no shipped golden is a multi-block QR plate) nor the compiler sees it, and none of P3's five listed gate items exercises a QR on a two-block plate: §7.2 is row sizes, §7.8 is the footer band, §7.11 is the unbounded callers, §7.20 is AdmissibleBlocks/MaxCharsAtBlocks counts. So P3 can close green with the round-1 Critical reintroduced.

*Fix:* Add §7.7(b) to P3's gate list (it may additionally be written in P2, where it passes by construction, but P3 is where it must be RED-then-green). Keep §7.7(a)/(c)/(d) where they are.

### 2. Important — §1 P1 vs P2

**P1 adds Fitted.qrAt and requires both legacy constructors to populate it, but the qrPlacement type and qrPlaceAt are introduced in P2 — P1 as written does not compile.**

*Failure:* P1's content is stated as: "`Fitted` gains `Mixed`, `Sizes`, `TitleSizeMM`, `FooterSizeMM`, `qrAt` (§2.3); **both** legacy constructors populate them — `fitBlocksAt` and `EngraveFreeText` (§2.3)", plus "§2.1.1's two panics". Spec §2.3 declares the field as `qrAt *qrPlacement`, and §2.3's constructor rules are "sets `qrAt` from `qrPlaceAt` at `anchorY = outerMargin` when there is a code". §2.1.1's first panic is `(QR == nil) == (qrAt == nil)`. All three need the `qrPlacement` type, and the constructor rule needs `qrPlaceAt`.

P2's content is stated as "`qrPlacement` + `qrPlaceAt` (§2.1)". So the type and its constructor are declared to arrive one phase after the phase that declares a field of that type, populates it in two constructors, and panics on it. The contradiction resolves in exactly one direction — the type and qrPlaceAt must be in P1 — because the alternative (defer qrAt to P2) also strips P1 of the first of its "two panics" and of half of §2.3's constructor obligation, which is the property P1 exists to establish. This is a phase-boundary error in a document whose stated job is fixing phase boundaries.

*Fix:* Move `qrPlacement` and `qrPlaceAt` into P1's content line and delete them from P2's, leaving P2 as "textLayout takes a *qrPlacement; lineLayout carries qrTop/qrBottom; EngraveFitted reads f.qrAt and EngraveText its own local". Also state explicitly in P1 that fitBlocksAt gains the `qrAt.Bottom <= plateHeight - margin` error return (§2.1.1 row 3), since P2's gate already tests it via §7.7(d).

### 3. Minor — §1 P1 — "Nothing reads the new fields yet"

**P1's guards fire on two existing in-package tests, so P1 is not the inert pure-addition the plan describes; the plan does not say those fixtures must move with it.**

*Failure:* Two backup-package tests build a Fitted literal and engrave it, and neither can carry Sizes:
- backup/blocks_test.go:414 (`TestEngraveFittedInsetsEachRowInItsOwnFace`): `mk` returns `Fitted{SizeMM: size, Lines: lines, Faces: faces, TitleFace: ..., FooterFace: ...}` with `len(Lines) == rows` and no Sizes, then engraves it twice (:417, :418). P1's `len(Sizes) != len(Lines)` panic fires and the test FAILS. This is the one existing test that pins the per-row inset — the exact property §2.5's `at(0)` rewrite endangers in P3 — so it must not be quietly weakened.
- backup/blocks_test.go:298 (`TestEngraveFittedRefusesAFaceMapThatDoesNotCoverTheLines`): `Fitted{SizeMM: 3.0, Lines: 2, Faces: 1}` with no Sizes. It only asserts `recover() != nil`, so it stays green — but if the Sizes guard is evaluated before the Faces guard it now passes for the wrong reason and no longer tests the Faces guard at all. §7.19's mutation pass is scoped to "every test added in P1-P6", so this pre-existing test is outside it.
The three GUI literals the spec names (gui/freetext_flow_test.go:564, 893, 928) are confirmed safe — all three flow only into ftConfirmBody/ftConfirmSummary and never reach EngraveFitted; :893's `QR` with no placement therefore does not trip §2.1.1's first panic.

*Fix:* Add to P1: "backup/blocks_test.go:298 and :414 build Fitted literals that EngraveFitted now rejects; both gain `Sizes`. The Faces guard is checked before the Sizes guard so :298 keeps testing what it names."

### 4. Minor — §1 P2

**P2 names textLayout, lineLayout, EngraveFitted and EngraveText, but not the other four call sites of textLayout/lineLayout that must move in the same commit.**

*Failure:* textLayout's callers are backup/fit.go:150 (wrapBlocks), backup/fit.go:327 (faceLayouts.at), backup/backup.go:359 (EngraveText), and three tests: backup/blocks_test.go:30, backup/blocks_test.go:393, backup/fit_test.go:15. wrapBlocks and faceLayouts.at both currently thread a `*qr.Code` and must instead thread a `*qrPlacement` — wrapBlocks in P2, even though the plan attributes wrapBlocks' signature change to P3, and faceLayouts in P2 even though the plan deletes it in P3. MaxCharsAtBlocks (fit.go:344-360) is the one faceLayouts consumer with a `qrc` and no placement, so it becomes a placement producer in P2, a phase earlier than the §3 table row the plan assigns to P5.
Separately, removing holeLines/qrLines from lineLayout (§2.1) breaks three tests that read them: backup/engravetext_test.go:159-196 constructs a lineLayout literal with `holeLines: 2, qrLines: 19` and is the current pin for the n<1 clamp and the QR-line predicate; backup/freetext_test.go:195-197 and :289 recompute the QR's y from lay.holeLines/lay.qrLines.
All of this is compiler-forced, so nothing can go silently wrong — but engravetext_test.go's clamp test is being rewritten in the same commit that changes its subject, which is where a pin gets weakened, and the plan gives no instruction to preserve it.

*Fix:* Extend P2's content line to name wrapBlocks, faceLayouts.at and MaxCharsAtBlocks as forced placement plumbing (still at anchorY = outerMargin, still one placement per plate), and add a sentence: "engravetext_test.go's lineLayout literal and freetext_test.go's holeLines/qrLines derivations are re-expressed in qrTop/qrBottom with their assertions unchanged; the n<1 clamp test keeps its exact numbers."

### 5. Nit — §1 P2 gate — §7.7(d)

**P2's gate cites §7.7(d) as "an ERROR from the fit", but the FitSized half of that item cannot exist in P2 and is vacuous in P4.**

*Failure:* Spec §7.7(d) is worded against `fitBlocksAt`/`FitSized`. FitSized does not exist until P4, so P2 can only satisfy the fitBlocksAt half. P4's gate (§7.5, §7.6, §7.9, §7.14) never picks the remainder up. In practice this costs nothing — §2.7 gives FitSized no QR parameter and leaves qrAt nil unconditionally, so the Bottom check is structurally satisfied there — but as written the plan leaves half a named test item with no owning phase.

*Fix:* Write P2's gate item as "§7.7(d), fitBlocksAt half — the FitSized half is vacuous by §2.7 (qrAt is always nil) and is recorded as such rather than scheduled."

### 6. Nit — §1 P5

**P5 claims "every row of §3's table", but four of that table's rows are owned by P1 and P3.**

*Failure:* §3's site table includes `MaxCharsAtBlocks` (§2.6), `rowFaces` (§2.4 start translation), `AdmissibleBlocks` (§2.4 start translation) and `EngraveFreeText` (§2.3). The plan assigns EngraveFreeText to P1 and the other three to P3 by name, then re-claims all of them for P5. Harmless for a single implementer working the phases in order, but it makes the follow-up reconciliation the plan mandates ("anything filed against a phase is burned down in or before that phase") ambiguous about which phase owns those rows.

*Fix:* Say "every GUI/preview row of §3's table; the backup-package rows (EngraveFreeText, AdmissibleBlocks, rowFaces, MaxCharsAtBlocks) are already closed in P1/P3."

### Notes

Reviewed against the real tree at /scratch/code/shibboleth/seedhammer @ 6d57681. `git status --short` was clean on entry and clean on exit; no probe files written.

Verdict RED on 2 Important. Both are phase-boundary errors with small, local fixes — no phase needs to be re-cut and no ordering has to change.

What I checked and found SOUND, so it is not re-litigated:

- **Every phase boundary is compilable and whole-suite-closable**, with the two exceptions above. P4 leaves FitSized uncalled (Go permits unused package functions; ftFitAt routes to it only in P5). P3's public signatures (AdmissibleBlocks, MaxCharsAtBlocks, Admissible, MaxCharsAt, FitBlocks, FitBlocksAt, EngraveFitted, EngraveFreeText) are all unchanged, so `gui` and `cmd/plateview` compile untouched across P1-P4 and only move in P5. `bodyRows` and `widthFor` becoming unused in P3 is not a compile error and both retain test callers.
- **P3 combining wrapBlocks' signature with EngraveFitted's internals does not hide a golden move.** The two halves are independently golden-visible on a uniform plate: a wrong `start`/`limit` translation changes which rows are hole-rows and therefore the line breaks, and a wrong running y moves the ink outright. There is no compensating pair that leaves backup/testdata/freetext-*.bin, text-*.bin, seed-*.bin, passphrase-*.bin byte-identical. Splitting P3 would buy nothing the goldens do not already give.
- **P2's §7.7(a) survives P3 without rewriting.** It is a property of `lineLayout.at` at baseY = outerMargin, not of EngraveFitted's row walk, so P3's `at(0)` rewrite does not touch it. Likewise backup/freetext_test.go:283-296's independent plate-row re-derivation stays valid because for a uniform plate `layout(baseY=margin).at(row)` and `layout(baseY=margin+row*fontSize).at(0)` are the same comparison.
- **P3 is correctly the phase that carries §7.20.** Both translation defects it pins (`AdmissibleBlocks`' literal `1` and anchoring the placement at `start` rather than `outerMargin`) only become expressible in P3, since P2 has no device-unit `start` for either to be confused with. Nothing earlier could have caught them and nothing later needs to.
- **P6-last is safe.** Deferring §7.3's composition pins does not let P4/P5 close without anyone checking the plate fits: P5's §7.10 drives SIZEPROOF!FRONT/BACK end-to-end through freetextPlateHook, and FitSized returns an error rather than a Fitted when the composition exceeds `[start, limit]`, so the back's 2.400 mm of spare is proven non-negative at P5. §7.3 pins the per-block row counts, budgets and y-ranges — a guard against a future font change, not against this implementation — and it genuinely cannot be written before P5 produces the composition. §7.19's placement at P6 is likewise defensible; per-phase mutation would be cheaper but its absence does not let a phase close on a gate that cannot see its own change (with the §7.7(b) exception above, which is a coverage hole, not a mutation-strength one).
- **No other §7 item and no §3 table row is orphaned.** Full cross-check: 1→P1/P2/P3, 2→P3, 3→P6, 4→P6, 5→P4, 6→P4, 7a/c/d→P2, 7b→NONE (finding 1), 8→P3, 9→P4, 10→P5, 11→P3, 12→P6, 13→P5, 14→P4, 15→P1, 16→P5, 17→P5, 18→P5, 19→P6, 20→P3. All 21 §3 rows land in a phase (four of them in two phases — the Nit above).
- **No contradiction with the spec's design** beyond finding 2's internal ordering error. The plan's standing constraints, the no-`-update` rule, the one-worktree/one-implementer shape and the opus+sonnet post-P5 review all match the operator's settled decisions, and the plan correctly records the deliberately-untested multi-size QR guard by not scheduling a test for it.

---

## Lane 2 — spec-vs-plan coverage (sonnet)

VERDICT: RED — 5 findings

### 1. Important — P2 gate (plan lines 65-67) vs spec §7 item 7(b)

**Spec test item 7(b) — the two-block QR-wrap regression test ("the round-1 Critical, unreachable by any single-block fixture") — is not gated by any phase.**

*Failure:* P2's gate line reads '§7.1 · §7.7(a) the band equivalence at all six rungs · §7.7(c) both engravers read the stored placement · §7.7(d) the Bottom check is an ERROR from the fit, not a panic' — (b) is absent. Grepping the whole plan for '§7.' confirms 7.7(b) appears in no phase's gate anywhere (P1-P6). Since P2 is exactly the phase that extracts qrPlacement and is the natural place for a new two-block-plus-QR fixture, its omission means the plan could be executed end-to-end, close all six phases green, and never build the fixture that regression-tests the original Round-1 Critical (body ink wrapping across the code box on a multi-block free-text plate). This is checkable today, independent of the spec's content.

*Fix:* Add '§7.7(b) a two-block plate with a QR wraps block 2's rows at the code's own budget, no body ink in the code box' to P2's gate list, alongside 7.7(a)/(c)/(d).

### 2. Important — P1 (plan lines 39-52) — "no behaviour change" claim vs backup/blocks_test.go

**P1's claim that P1 is pure addition with 'nothing reads the new fields yet' is false: an existing test bypasses both 'legacy constructors' and will panic against P1's new len(Sizes)!=len(Lines) guard, so go test ./... will not be green at P1's close as the plan requires — and no phase is assigned to fix it.**

*Failure:* backup/blocks_test.go:406-416 (the `mk` helper inside TestEngraveFittedInsetsEachRowInItsOwnFace) hand-builds `Fitted{SizeMM: size, Lines: lines, Faces: faces, TitleFace: faces[0], FooterFace: constant.Font}` directly — it does not go through fitBlocksAt or EngraveFreeText, the only two constructors P1 says are updated to populate Sizes/TitleSizeMM/FooterSizeMM/qrAt. `Sizes` is left nil while `Lines` has `LinesPerPlate(P, 3.0)` entries, so P1's own new guard fires. The test does not expect a panic (no recover/defer) — it calls `EngraveFitted(P, mk(sh.Font))` directly and inspects ink bounds, so it will fail outright once the guard is added literally as §2.3 specifies. Plan §2 states 'A phase is green when ... go test ./... [is] green — the whole surface,' so P1 as written cannot close without an unplanned edit to this pre-existing test.

*Fix:* Add an explicit P1 task: update the `mk` helper in backup/blocks_test.go (and audit for any other hand-built `Fitted{}` literals in the backup package outside the two constructors) to populate `Sizes` before P1's guard lands.

### 3. Important — Plan §0 line 31 ("after P5") vs §3 heading "After P6" (line 126)

**The plan contradicts itself on when the mandatory two-lane review runs: the standing-constraints section says 'after P5', but the numbered post-phase procedure is titled and scoped 'After P6' — the actual last phase.**

*Failure:* Line 31: '**Review:** opus + sonnet two-lane on the whole diff after P5, per the operator's 2026-08-05 choice.' Line 126: '## 3. After P6', whose item 1 places the same non-deferrable whole-diff review there. P6 (composition pins §7.3/§7.4/§7.12 plus the §7.19 mutation pass) is the last phase and contains substantial new test and pin content. If 'after P5' in §0 is followed literally, the mandatory whole-diff review would be dispatched before P6 exists and would never cover it — directly undermining the review's own stated purpose ('the whole diff ... catches implementation-introduced regressions TDD misses'). This is a plain textual contradiction, checkable by grep alone.

*Fix:* Change line 31 to 'after P6' to match §3, or explicitly state a rationale if a P5 checkpoint review is genuinely intended in addition to the P6 one (in which case §3 needs to say so too).

### 4. Minor — backup/blocks_test.go:296-309 (TestEngraveFittedRefusesAFaceMapThatDoesNotCoverTheLines) vs P1

**A second pre-existing test also omits `Sizes`, but since it already expects a panic (via recover), P1's new guard will make it pass for the wrong reason, silently losing its stated coverage of the Faces-length guard.**

*Failure:* The literal at line 298 sets `Lines` (2 entries) and `Faces` (1 entry) to test the Faces-length mismatch panic, but never sets `Sizes`. After P1, `len(Sizes)==0 != len(Lines)==2` will very likely panic before/instead of the Faces check, and the test's `recover() == nil` assertion doesn't distinguish which panic fired, so it stays green while no longer exercising its named invariant. P6's mutation pass is scoped to 'every test added in P1-P6' per plan line 112, so it will not catch this pre-existing test's silent coverage loss.

*Fix:* When P1 lands the Sizes guard, also set `Sizes: []float32{3.0, 3.0}` on this test's literal so it continues to isolate the Faces-length guard specifically.

### 5. Minor — P3 phase text (lines 82-84) vs sizeproof-spec-R0-round3.md finding 2

**The plan's claim that 'both round-2's I2 and round-3's I2 live here [P3]' is only half right for round-3's I2 — its design-side fix (adding AdmissibleBlocks to §2.1's anchorY table) is P2's content per the plan's own P2 text; only the confirming test (§7.20 with useQR) is P3's, as the plan separately and correctly states.**

*Failure:* Round-3's I2 (design/agent-reports/sizeproof-spec-R0-round3.md:29-37) has two parts: (1) add AdmissibleBlocks to §2.1's anchorY table, stating anchorY and start are deliberately different — this is exactly what P2's own text already captures ('including AdmissibleBlocks, whose anchorY is outerMargin while §2.4 will give it a different start', plan line 58-59); (2) extend §7.20 to pin with useQR=true — this is P3's gate item, correctly cited. So attributing the whole of round-3's I2 to 'living' in P3 is imprecise; the design fix is P2's, only the regression test is P3's. Not blocking since both halves are in fact covered by some phase, just misattributed in the narrative.

*Fix:* Reword the P3 note to 'round-3's I2's test (§7.20 with useQR) closes here; its design fix is P2's,' or leave as-is if the intent was always 'the pin, not the code, lives here' (in which case a slightly clearer phrase would help future readers).

### Notes

Scope respected: reviewed the PLAN only; no findings raised against SPEC content. Repo verified at commit 6d57681 with the PASSPROOF! rename (git log -1 confirms). git status --short was clean on entry and exit in /scratch/code/shibboleth/seedhammer; no probe files were written.

Part A (test-item coverage): built the full item -> phase map for all 20 items (with §7.7's four sub-parts). 19 of 20 items are cleanly gated exactly once each (or, for item 1, redundantly at P1/P2/P3, which is harmless). The single gap is §7.7(b), reported above. No phase cites a §7 item number that doesn't exist in the spec (checked 1-20 plus 7(a)-(d) all resolve).

Part B (site coverage, spec §3): all 22 named sites are accounted for. `fitBlocksAt`, `EngraveFreeText` -> P1; `AdmissibleBlocks` (its own QR anchor) -> P2, `AdmissibleBlocks`/`rowFaces` (wrapBlocks start translation) -> P3, `faceLayouts`/`MaxCharsAtBlocks` (§2.6) -> P3; `FitSized` -> P4; every remaining GUI-layer row (`ftFaceRun`, `ftPlan.Blocks`, `ftProof`, `ftProofOutcome`, `ftRungLabel`, `ftProofReplaces`, `ftFitAt`, `ftEvaluate`/`ftBuildPlate`, `ftFaceSummary`, `ftSizeLabel`, `ftConfirmSummary`, `gui.Preview`, `proofPreview`, `fittedPreviewAt`, `previewBuilders`, `sizeLabel`, `ftProofLoader`, `ftProofOutcomeFor`) -> P5, via P5's explicit blanket 'Every row of §3's table' plus named call-outs for several. No site is unclaimed.

Part C (citations): repo HEAD is 6d57681, `PASSPROOF!` rename commit confirmed by `git log -1`. Every spec section the plan cites (§2.1-§2.7, §3, §3.1, §4, §7.x, §8) exists and says what the plan claims it says — read the full 978-line spec and cross-checked each citation. Round-2's I2 matches P3 cleanly. Round-3's I2 is split across P2 (design) and P3 (test) — plan's attribution is slightly imprecise, reported as a Minor. §7.7's multi-size-QR-untested decision text and date match the plan's settled-decisions section verbatim (both say operator, 2026-08-05).

Part D (buildability spot-check, P1): grepped every `Fitted{` construction and every `EngraveFitted(`/`backup.EngraveFitted(` call in the repo. Two backup-package test literals build `Fitted{}` directly, bypassing both legacy constructors: blocks_test.go:298 (expects a panic anyway — silently loses its specific Faces-guard coverage, Minor) and blocks_test.go:414 (does NOT expect a panic — will break P1's own close criterion, Important, detailed above). The three GUI fixtures the spec names (freetext_flow_test.go:564/893/928) were confirmed to never call EngraveFitted (they only feed ftConfirmBody/ftConfirmSummary/ftConfirmRows), matching the spec's own claim. gui/freetext_proof_test.go's Fitted-mutation test goes through `proofFit` (a real production-path helper), not a hand-built literal, so it is unaffected.

Part E (internal consistency): found one genuine contradiction (line 31 'after P5' vs §3 'After P6', reported above as Important). No phase-count mismatch (P1-P6 consistently used throughout) and no dangling section references found elsewhere.
