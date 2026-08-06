# R0 review — IMPLEMENTATION_PLAN_sizeproof.md — round 1 (fold check)

Two lanes dispatched 2026-08-05 against the folded plan @ `cc78b75`, scoped to
"did the fold fix each round-0 finding, and did it introduce a new defect".
The spec and round 0's settled facts were declared out of scope; neither lane
re-derived them. Persisted VERBATIM before folding.

VERDICT: **RED** — 0C / 1 Important / 3 Minor / 2 Nit.
Lanes split: sonnet GREEN with 0 findings (all nine round-0 items confirmed
folded, every source fact verified); opus RED on one Important the coverage
lane was not looking for. **All nine round-0 findings are CLOSED** — the
Important is a NEW defect the fold introduced.

---

## MERGED GATE VERDICT (synthesiser)

VERDICT: RED — 6 findings

### 1. Important — §1 P3 ("Written in P2 where it is cheap, RED-then-green here") vs §1 P2 ("passes vacuously in P2") and §0's TDD rule

**The fold gave §7.7(b) an owner and a gate but no phase in which its discriminating power is ever demonstrated: the plan says it is green by construction in P2 and simultaneously instructs "RED-then-green" in P3, which cannot be performed — only DEFECTIVE P3 code turns it red — so P3's gate cannot establish that it catches the round-1 Critical it is the sole pin for.**

*Failure:* Plan L26 requires "the phase's gating tests are written and RED before its implementation". L115 says §7.7(b) "passes vacuously in P2". Nothing changes between P2 closing whole-suite-green and P3's implementation starting, so at P3's start the fixture is green; correct P3 code leaves it green. "RED-then-green here" (L144) is therefore literally unexecutable, and the only constraint the plan places on the fixture is "pin its code by MODULE COUNT" — nothing that forces a shape capable of failing. This matters because L142-144 itself enumerates that nothing else in P3's gate can see the defect (§7.2 row sizes, §7.8 footer band, §7.11 unbounded counts, §7.20 admission counts; no shipped golden is a multi-block QR plate). MEASURED at prodParams / 3.0 mm / sh.Font / an 89-module code (probe run and deleted): holeLines=3, qrLines=20, band = plate rows 3..22, charPerQRLine=12 vs charPerLine=44, and ROW 23 IS A FULL-WIDTH 44-COLUMN ROW BELOW THE BAND. So the natural reading of "no body ink enters the code box" as a max-x bound over the plate FAILS on a correct plate, and the implementer's natural repair is to weaken it to something the block-relative-index defect also satisfies. Failure scenario: P3 lands with block 2's rows indexed block-relative against baseY = outerMargin; every other P3 gate item passes, no golden moves, the compiler sees int-to-int, and §7.7(b) passes because it was authored in a phase where nothing could tell a good fixture from a bad one. P3 closes green on a plate that engraves body ink across the QR. §7.19's mutation pass at P6 is the backstop — three committed phases later, forcing rework of P3 under P4/P5.

*Fix:* Two sentences, no phase re-cut. (1) In P3, replace "RED-then-green here" with: "§7.7(b) is a REGRESSION pin and cannot be RED before this phase's implementation. Its power is demonstrated by pulling §7.19's mutation forward for this ONE item: after P3 is green, temporarily index block 2's rows block-relative against baseY = outerMargin and confirm §7.7(b) FAILS, then revert. A §7.7(b) that stays green under that mutation is a blocking finding, not a passing gate." (2) In P2's §7.7(b) paragraph, add the shape constraints the fixture needs, since it is authored where nothing can check it: block 1 must wrap to at least one row and end ABOVE the band's first row (so a block-relative index shifts block 2's window by a non-zero number of rows); block 2's text must FILL its budget on every row it spans (a spaceless run, as `mixedBlocks`/`fillRows` already do) so a wrong budget inks rather than merely permitting; and "no body ink enters the code box" must be asserted as an intersection with the code's RECTANGLE — [qrAt.X, X+Size) x [qrAt.Y, Y+Size) — never as a plate-wide max-x bound (measured: row 23 legitimately inks all 44 columns below the band).

### 2. Minor — §1 P1 — the `qrPlacement`/`qrPlaceAt` content line vs P1's Gate (§7.1, §7.15) and its "Why first" rationale

**Pulling `qrPlaceAt` into P1 lands a second copy of textLayout's holeLines/qrLines arithmetic plus a new error return, and no P1 gate item can observe a wrong VALUE in either; detection is deferred to P2.**

*Failure:* Verified: in P1 `textLayout` still computes holeLines/qrLines internally (backup/wrap.go:172,183) and `EngraveFitted` still derives the code's y from `lay.holeLines`/`lay.qrLines` (backup/freetext.go:80-85), so nothing consumes qrAt's values. The P1 guards that do fire check nil-consistency and !Mixed; §7.15 checks the Sizes length. §2.1.1's third row is measured unreachable from any shipped fit (spec §2.1.1), so the new ERROR return has no P1 exercise either — the plan books its test at P2's §7.7(d). A wrong Top/Bottom/X/Y/Size/KeepOutX (an off-by-one ceil, qrBorder omitted) closes P1 green on every golden byte-for-byte and first shows up at P2. Rework is bounded to the next phase, hence Minor — but P1's "a moved golden here means the constructors were filled wrong" reads as though the goldens validate the placement fill, and for the placement values they cannot.

*Fix:* Add to P1: "P1's gate cannot see a wrong VALUE in `qrPlaceAt` or in the new error return — no consumer reads `qrAt` this phase (`EngraveFitted` still derives the code's y from `lay.holeLines`/`lay.qrLines`, freetext.go:80-85), and §2.1.1's third row is unreachable from any shipped fit. Both are first exercised at P2 (§7.1, §7.7(a)/(c)/(d)); P1's guards prove only nil-consistency and the `Sizes` length."

### 3. Minor — §1 P2 — "The signature change forces four more sites into this same commit"

**`rowFaces` (fit.go:302) and the three direct `textLayout` test callers are compiler-forced into P2 and go unnamed, so a paragraph whose stated purpose is "so the diff is not a surprise" undercounts the diff.**

*Failure:* Verified against source: once `wrapBlocks` (fit.go:147) takes a `*qrPlacement`, all three of its callers must produce one — `fitBlocksAt`, `AdmissibleBlocks` (covered by the preceding sentence) and `rowFaces` (fit.go:297-302, which calls `wrapBlocks(params, blocks, fontMM, qrc, 0, math.MaxInt)`). `rowFaces` is a §2.1 producer named by P3, not P2. Separately, three tests call `textLayout` directly and must be re-expressed: backup/blocks_test.go:30 (`rowBudget`), backup/blocks_test.go:393 (`insetOf` — inside the very test P1 says must keep its exact form) and backup/fit_test.go:15. P2's "Three test fixtures move too" names only the lineLayout-FIELD readers (engravetext_test.go:159-196, freetext_test.go:195-197, :289). Everything here is compiler-forced so nothing goes silently wrong; the defect is an inaccurate diff preview, one item of which is a pin P1 was just told to preserve verbatim.

*Fix:* In P2: "...`wrapBlocks` (fit.go:147), `faceLayouts.at` (fit.go:327) and `rowFaces` (fit.go:302) thread a `*qrPlacement` instead of a `*qr.Code`, which makes `MaxCharsAtBlocks` and `rowFaces` placement producers in P2 rather than P3..." and extend the fixture sentence with "...plus the three direct `textLayout` callers — blocks_test.go:30, blocks_test.go:393 (`insetOf`, the P1 pin) and fit_test.go:15 — which change type only and keep their arguments and assertions identical."

### 4. Minor — §1 P1 — "It gains `Sizes`; its per-row-inset assertions keep their exact form" (backup/blocks_test.go:406-416)

**Adding `Sizes` to `mk` is assertion-preserving only for one specific value, which the plan does not state; a non-uniform `Sizes` would pass the length guard and every assertion while silently measuring a different row once P3 lands.**

*Failure:* Verified: `mk` (blocks_test.go:405-416) builds `Fitted{SizeMM: 3.0, Lines: 26 entries, Faces, TitleFace, FooterFace}` with Title and Footer empty. Its reference `insetOf` (blocks_test.go:392-396) is `textLayout(P, fnt, P.F(3.0), P.I(outerMargin), nil, freeTextQRScale).at(rows-1)` — a UNIFORM, plate-absolute layout at one size — and the test self-guards on `insetOf(constant.Font) != 0` ("row 25 is a screw-hole row") from that same reference, not from the engraved y. If an implementer satisfies `len(Sizes) == len(Lines)` with anything non-uniform or != SizeMM, then in P3 `EngraveFitted`'s running y for row 25 is no longer margin + 25*F(3.0); row 25 may cease to be a screw-hole row while the reference keeps saying it is, `mixed` and `alone` shift identically so the equality still holds, and the test passes measuring nothing — the same silent-coverage-loss class the plan flags for :296-309, on the ONLY existing pin for the property §2.5's at(0) rewrite endangers.

*Fix:* State the value in P1: "blocks_test.go:414's `mk` gains `Sizes` = len(lines) copies of `size` (3.0), equal to `SizeMM`, with `Mixed` false and `TitleSizeMM`/`FooterSizeMM` 0 — Title and Footer are empty, so §2.3's invariant requires 0. Uniform-at-SizeMM is the only fill that keeps the test's plate-absolute `insetOf` reference valid once §2.5's running y lands in P3."

### 5. Nit — Header line 3 — "Status: **P1, folding plan R0 round 0 (RED, 0C/4I).**"

**The fold advanced the revision label to "P1", which collides with the document's own phase namespace (P1 = "the data carriers"), reads as "implementation has reached phase P1", and states a finding count that drops the 3 Minor / 2 Nit also folded.**

*Failure:* Round 0 reviewed "the P0 plan", so P<n> is this document's revision counter — but P1..P6 are also its implementation phases. A reader arriving at "Status: P1" reasonably reads it as implementation-in-progress, which would be an R0-gate violation; in fact no `sizeproof` branch or worktree exists and no code has started. "folding" is also present-tense for completed work.

*Fix:* "Status: **R1** — plan R0 round 0 (RED, 0C/4I/3m/2n) FOLDED; awaiting re-review. No implementation has begun."

### 6. Nit — §1 P1 — "§2.1.1's **two panics** *and* its third row, the `qrAt.Bottom <= plateHeight - margin` **ERROR return** from `fitBlocksAt`"

**Spec §2.1.1 row 3 has two enforcement points — the error return AND a defensive re-assert panic in `EngraveFitted` — and only the error return is assigned to any phase.**

*Failure:* Verified against SPEC_sizeproof.md §2.1.1's table: row 3 reads "`fitBlocksAt`/`FitSized`, error return; re-asserted in `EngraveFitted` as a defensive panic". P1 names the `fitBlocksAt` error return (FitSized's half correctly waits for P4); the defensive `EngraveFitted` panic is named nowhere in P1-P6. It costs nothing today — the condition is measured unreachable from any shipped fit and no `Fitted` literal in the tree carries a QR into `EngraveFitted` — so it can neither break a phase nor be caught by one. It is simply an unassigned line of the spec.

*Fix:* Extend P1's clause to "...and its third row: the `qrAt.Bottom <= plateHeight - margin` ERROR return from `fitBlocksAt`, plus its defensive re-assert as a panic in `EngraveFitted`."

### Notes

MERGED VERDICT: RED — 0 Critical / 1 Important / 3 Minor / 2 Nit. The two lanes agree completely on question 1 (all nine round-0 findings folded) and on every source fact; they diverge only on question 2, where the opus lane found one Important the sonnet lane did not look for (sonnet verified citations and coverage; it did not interrogate whether a gate item authored in a vacuous phase can bite). I verified the Important myself against source and it holds. Nothing was deduplicated away — the lanes' findings do not overlap.

(a) FOLD STATUS — ALL NINE ROUND-0 FINDINGS CLOSED. Verified line-by-line in /scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_sizeproof.md, both lanes independently concurring:
 I1 §7.7(b) orphan — CLOSED as scheduling: written in P2 (L112-115, module-count pin), gated at P3 (L130-132). The mechanism gap is a NEW defect (the Important), not a failure to fold.
 I2 P1 did not compile — CLOSED: `qrPlacement`/`qrPlaceAt` moved to P1 (L45-46), removed from P2, §2.1.1's third row named as an ERROR return (L50-51). Confirmed compilable: `fitBlocksAt` already returns (Fitted, error) so the error return forces no caller change.
 I3 blocks_test.go:406-417 — CLOSED (L59-65).
 I4 "after P5" vs "After P6" — CLOSED: L33 now "after P6 (§3)" plus "There is no P5 checkpoint review"; no "after P5" remains anywhere.
 m5 blocks_test.go:296-309 — CLOSED (L66-70), including "the Faces guard is evaluated before the Sizes guard".
 m6 unnamed P2 sites — CLOSED but incomplete (Minor 3: rowFaces and the three textLayout test callers still unnamed).
 m7 round-2/round-3 I2 split — CLOSED (L146-149), exactly as adjudicated.
 n8 §7.7(d) FitSized half — CLOSED, recorded vacuous (L118-120).
 n9 P5 double-owned rows — CLOSED (L163-165); "P2-P3" is more accurate than round 0's suggested "P1/P3".

(b) SOURCE CHECKS CONFIRM THE PLAN'S FIXTURE CLAIMS — yes, every one, at /scratch/code/shibboleth/seedhammer @ 6d57681. I re-verified the load-bearing ones myself: blocks_test.go:405-416 `mk` builds a Fitted with no `Sizes` and :417-418 engrave it twice with no recover in the whole function (386-430); blocks_test.go:296-309 builds Fitted{3.0, 2 lines, 1 face} and DOES defer a recover; backup/freetext.go:80-85 still derives the QR y from lay.holeLines/lay.qrLines (basis of Minor 2); backup/fit.go:302 `rowFaces` is a third `wrapBlocks` caller (basis of Minor 3); blocks_test.go:30 and :393 and fit_test.go:15 call `textLayout` directly. Sonnet additionally verified engravetext_test.go:159-198's n<1 clamp pin, freetext_test.go:195/197/289 as the only holeLines/qrLines readers, and gui/freetext_flow_test.go:564/893/928 never reaching EngraveFitted. And I reproduced the opus lane's measurement exactly (probe `backup/zz_merge_probe_test.go`, run and DELETED): prodParams, 3.0 mm, sh.Font, 89-module code → holeLines 3, qrLines 20, band = plate rows 3..22, charPerQRLine 12 vs charPerLine 44, and row 23 inks the full 44 columns below the band. That measurement is what makes the Important's fix wording load-bearing: a max-x formulation of "no body ink enters the code box" fails on a CORRECT plate.

(c) CONTROLLER MUST DECIDE — one thing only: the Important's fix adds a fixture-SHAPE constraint (rectangle-intersection assertion; block 1 ends above the band; block 2 fills its budget) to a plan whose stated rule is "it adds no design: every what lives in the spec". Decide whether to (i) record the shape in the PLAN as a sequencing consequence of authoring the fixture a phase early — recommended, it re-opens no spec gate and the spec's §7.7(b) already says "no body ink enters the code box"; or (ii) reopen the GREEN/CLOSED spec to add it there. Option (i) keeps SPEC_sizeproof.md closed. Everything else in this report is either a two-sentence plan edit or non-gating.

(d) REPO CLEAN — yes. /scratch/code/shibboleth/seedhammer @ 6d57681, `git status --short` empty on entry and empty on exit; my probe (backup/zz_merge_probe_test.go) was deleted and the clean status re-confirmed after deletion. Both source lanes also reported clean entry/exit. No `sizeproof` branch or worktree exists — no code has started ahead of the gate.

(e) READY TO EXECUTE — NOT YET, but it is one small edit away. The plan's ORDER, phase boundaries, item-to-phase map and fixture inventory are sound and complete; the single blocker is that P3 instructs a step ("RED-then-green here") that cannot be performed on a regression pin authored vacuously in P2, leaving P3's named catcher of the round-1 Critical unvalidated until P6. Fold the Important (two sentences in §1, no phase re-cut) plus, opportunistically, the three Minors and two Nits — none of which requires a re-review on its own — and this plan is executable. The re-fold is comment/wording-scale plus one added mutation step, so a scoped "did the fold fix it, did it break anything" pass closes the gate.

DROPPED — nothing. Neither lane re-derived a settled fact: coverage tables, phase-boundary compilability, the P3-combining / P2-§7.7(a)-survives-P3 / P6-last arguments, and the deliberately-untested multi-size QR guard were all taken as settled by both lanes and by me. No finding from either lane targets SPEC_sizeproof.md, so no spec findings were dropped.

---

## Lane 1 — fold-adversarial (opus)

VERDICT: RED — 6 findings

### 1. Important — §1 P3 (gate line + "Written in P2 where it is cheap, RED-then-green here") vs §0's TDD rule and §1 P2's §7.7(b) paragraph

**The fold gave §7.7(b) an owner but not a phase in which its power is demonstrated: it is green by construction in P2, it cannot be RED before P3's implementation, and the plan constrains its shape only by module count — so P3's gate may not be able to catch the round-1 Critical it is now the sole pin for.**

*Failure:* §0 requires "the phase's gating tests are written and RED before its implementation", and P3 says of §7.7(b) "Written in P2 where it is cheap, RED-then-green here." That instruction cannot be executed. §7.7(b) is a REGRESSION pin, not a driver: at the start of P3 the tree is exactly P2's tree, where the plan itself says the item "passes vacuously", so it is green; correct P3 code leaves it green; only DEFECTIVE P3 code turns it red. There is therefore no phase in which the fixture is ever observed failing, and the only constraint the plan places on it is "pin its code by MODULE COUNT" — nothing that forces it into a shape capable of failing. Three concrete shapes satisfy the plan's text and still pass under the exact defect P3 opens (block 2 indexed block-relative against baseY = outerMargin): (a) block 1 wraps to zero rows or to rows that already cover the band, so shifting block 2's window changes no row's classification; (b) block 2's text is short words that never fill their budget, so a 44-column budget where 12 was correct produces no extra ink; (c) "no body ink enters the code box" asserted as a max-x bound over the plate. (c) is not hypothetical — I measured it: at 3.0 mm, sh, an 89-module code, the band is plate rows 3..22 with charPerQRLine = 12 against charPerLine = 44, and row 23 sits BELOW the band and legitimately inks the full 44 columns. A max-x assertion therefore fails on a correct plate, and the implementer's natural repair is to weaken it to something the defect also satisfies. Failure scenario: P3 lands with block 2's rows indexed block-relative; §7.2 (row sizes), §7.8 (footer band), §7.11 (unbounded counts) and §7.20 (admission counts) all pass, no golden moves because no shipped golden is a multi-block QR plate, the compiler sees int-to-int, and §7.7(b) passes because it was written in a phase where nothing could tell a good fixture from a bad one. P3 closes green with body ink engraved across a QR that is a machine-readable copy of the plate. That is the failure round-0's finding 1 exists to prevent; the fold moved it from "unowned" to "owned but unvalidated at the gate that depends on it". §7.19 at P6 would eventually catch it, three phases of committed work later.

*Fix:* Two edits, both in §1. (1) In P3, replace "RED-then-green here" with the mutation it actually needs: "§7.7(b) is a regression pin and cannot be RED before this phase's implementation. Its power is demonstrated instead by pulling §7.19's mutation for THIS ONE item forward from P6: after P3 is green, temporarily index block 2's rows block-relative against baseY = outerMargin and confirm §7.7(b) FAILS, then revert. A §7.7(b) that stays green under that mutation is a blocking finding, not a passing gate." (2) In P2's §7.7(b) paragraph, add the shape the fixture must have, since it is being authored where nothing can check it: block 1 must wrap to at least one row and must END ABOVE the band's first row, so a block-relative index shifts block 2's window by a non-zero number of rows; block 2's text must be a run that FILLS its budget on every row it spans (a long unbroken token stream, not short words), so a wrong budget inks rather than merely permitting; and "no body ink enters the code box" must be asserted as an intersection with the code's RECTANGLE — [qrAt.X, X+Size) x [qrAt.Y, Y+Size) — never as a max-x bound, because rows below the band ink the full 44 columns on a correct plate (measured at 3.0 mm, sh, 89 modules: band = plate rows 3..22, charPerQRLine 12 vs charPerLine 44, row 23 at full width).

### 2. Minor — §1 P1 — content line ("qrPlacement and qrPlaceAt", the ERROR return) vs P1's Gate (§7.1, §7.15) and its "Why first" rationale

**Pulling qrPlaceAt into P1 lands a second, independent copy of textLayout's holeLines/qrLines arithmetic plus a new error return, and none of P1's gate items can see a wrong value in either; detection is deferred one phase to P2.**

*Failure:* In P1 textLayout still computes holeLines/qrLines internally (wrap.go:172,183) and EngraveFitted still draws the code from lay.holeLines/lay.qrLines (freetext.go:81-85), so nothing consumes qrAt's VALUES. qrPlaceAt has no face parameter, so it cannot delegate to textLayout and must duplicate the derivation. The guards that do fire in P1 check only nil-consistency ((QR==nil)==(qrAt==nil), exercised by the existing QR goldens) and !Mixed; §7.15 tests the Sizes length guard. §2.1.1's third row is measured unreachable from any shipped fit, so the new ERROR return has no P1 exercise either — the plan books its test at P2's §7.7(d). Failure scenario: qrPlaceAt computes Top, Bottom, X, Y, Size or KeepOutX wrongly — an off-by-one in the ceil, qrBorder omitted from KeepOutX — and P1 closes green on every golden byte-for-byte. It first shows up in P2, where §7.1 and §7.7(a)/(c) do catch it. No rework beyond the immediately following phase, hence Minor, but P1's "a moved golden here means the constructors were filled wrong, with nothing else in the diff to hide behind" reads as though the goldens validate the fill, and for the placement values they cannot.

*Fix:* Add one sentence to P1: "P1's gate cannot see a wrong VALUE in qrPlaceAt or in the new error return — no consumer reads qrAt in this phase, and §2.1.1's third row is unreachable from any shipped fit. Both are first exercised at P2 (§7.1, §7.7(a)/(c)/(d)); the P1 guards prove only nil-consistency and the Sizes length." Optionally soften the "Why first" clause to name what the goldens actually pin in P1 (the constructor fills that feed the existing engraving path), not the placement arithmetic.

### 3. Minor — §1 P2 — "The signature change forces four more sites into this same commit"

**The sentence promises four forced sites and names three; rowFaces (fit.go:302) and the three tests that call textLayout directly are also compiler-forced in P2 and go unnamed, so the list does not do the job it states.**

*Failure:* P2 names wrapBlocks (fit.go:150), faceLayouts.at (fit.go:327) and MaxCharsAtBlocks — three, under a heading that says four. Once wrapBlocks takes a *qrPlacement, all three of its callers must produce one: fitBlocksAt (fit.go:227, covered), AdmissibleBlocks (fit.go:280, covered by the preceding sentence) and rowFaces (fit.go:302, unnamed) — and rowFaces is a §2.1 producer in the spec's own table. Separately, three tests call textLayout directly and must be re-expressed: backup/blocks_test.go:30, backup/blocks_test.go:393 (the insetOf helper inside the very test P1 says must keep its exact form) and backup/fit_test.go:15 (which builds its layout from a qrc and a scale). P2's "Three test fixtures move too" names only engravetext_test.go:159-196, freetext_test.go:195-197 and :289 — the lineLayout-FIELD readers, not the textLayout callers. All of it is compiler-forced, so nothing goes silently wrong; the defect is that a paragraph whose stated purpose is "so the diff is not a surprise" undercounts the diff, and one of the unnamed sites is a pin P1 has just been told to preserve verbatim.

*Fix:* In P2, change "four more sites" to match what is listed and add the missing ones: "...wrapBlocks (fit.go:150), faceLayouts.at (fit.go:327) and rowFaces (fit.go:302) thread a *qrPlacement instead of a *qr.Code, which makes MaxCharsAtBlocks and rowFaces placement producers in P2...". Extend the fixture sentence to "...plus the three direct textLayout callers — blocks_test.go:30, blocks_test.go:393 (insetOf, the P1 pin) and fit_test.go:15 — which change type only and must keep their arguments and assertions identical."

### 4. Minor — §1 P1 — "It gains Sizes; its per-row-inset assertions keep their exact form" (backup/blocks_test.go:406-415)

**Adding Sizes to mk is assertion-preserving only for one specific value; the plan does not state it, and a non-uniform Sizes would silently change what the test measures in P3 without any assertion noticing.**

*Failure:* mk builds Fitted{SizeMM: size, Lines: lines (26 entries), Faces: faces, TitleFace: faces[0], FooterFace: constant.Font} with Title and Footer both empty. The test's reference, insetOf (blocks_test.go:392-396), is textLayout(P, fnt, P.F(3.0), P.I(outerMargin), nil, freeTextQRScale).at(row) with row = rows-1 = 25 — a UNIFORM, plate-absolute layout at 3.0 mm. It also self-guards on insetOf(constant.Font) != 0, i.e. "row 25 is a screw-hole row", computed from that same uniform reference rather than from the engraved y. Failure scenario: an implementer satisfies len(Sizes) == len(Lines) with anything non-uniform (or with a value != SizeMM). The length guard passes, Mixed stays false in the literal, and in P3 EngraveFitted's running y for row 25 is no longer margin + 25*F(3.0). Row 25 may cease to be a screw-hole row, the reference insetOf keeps saying it is, both mixed and alone shift by the same amount so mixed == alone still holds, and the test passes while measuring nothing — the same silent-coverage-loss class the plan itself flags for :296-309, and this one is the ONLY existing pin on the property §2.5's at(0) rewrite endangers. Only one value keeps the test measuring what it names: 26 entries of 3.0, equal to SizeMM, with Mixed false and TitleSizeMM/FooterSizeMM left 0 (Title and Footer are "", so §2.3's size/string invariant requires exactly 0 there).

*Fix:* State the value in P1: "backup/blocks_test.go:414's mk gains Sizes = len(lines) copies of size (3.0), equal to SizeMM, with Mixed false and TitleSizeMM/FooterSizeMM 0 — Title and Footer are empty, so §2.3's invariant requires 0. Uniform-at-SizeMM is the only fill that leaves the test's insetOf reference (a plate-absolute layout at one size) valid once §2.5's running y lands in P3; a non-uniform Sizes passes the length guard and every assertion while measuring a different row."

### 5. Nit — Header line 3 — "Status: P1, folding plan R0 round 0 (RED, 0C/4I)"

**The fold advanced the plan's own revision label from P0 to P1, which now collides with the document's phase namespace, and the status still reads as mid-fold.**

*Failure:* Round 0 reviewed "the P0 plan @ 7533c02", so P<n> is this document's revision counter — but P1..P6 are also its implementation phases, and P1 is titled "the data carriers". A reader arriving at "Status: P1" reasonably reads it as "implementation is at phase P1", which would be an R0-gate violation (no code before GREEN; there is in fact no sizeproof branch or worktree, so nothing has started). "folding" is also present-tense for work that is complete, and the finding count (0C/4I) drops the 3 Minor / 2 Nit that were also folded.

*Fix:* Rename the revision counter out of the phase namespace and state the gate position: "Status: R1 — plan R0 round 0 (RED, 0C/4I/3m/2n) FOLDED; awaiting re-review. No implementation has begun."

### 6. Nit — §1 P1 — "§2.1.1's two panics and its third row, the qrAt.Bottom <= plateHeight - margin ERROR return from fitBlocksAt"

**§2.1.1 row 3 has two enforcement points — the error return AND a defensive re-assert panic in EngraveFitted — and only the error return is named by any phase.**

*Failure:* Spec §2.1.1's third row reads "fitBlocksAt/FitSized, error return; re-asserted in EngraveFitted as a defensive panic". P1 names the error return in fitBlocksAt (FitSized's half correctly waits for P4). The defensive panic in EngraveFitted is named nowhere in P1-P6. It costs nothing today — the condition is measured unreachable from any shipped fit and no Fitted literal in the tree carries a QR into EngraveFitted (blocks_test.go:298 and :414 have no QR; the three gui literals never engrave) — so it can neither break a phase nor be caught by one. It is simply an unassigned line of the spec.

*Fix:* Extend P1's clause to "...and its third row: the qrAt.Bottom <= plateHeight - margin ERROR return from fitBlocksAt, plus its defensive re-assert as a panic in EngraveFitted."

### Notes

SCOPE: fold-vs-findings + new-defect only. Coverage tables, phase-boundary compilability, the P3-combining / P2-§7.7(a)-survives / P6-last arguments, and the untested multi-size QR guard were taken as settled and not re-derived. No spec findings.

QUESTION 1 — did the fold fix each round-0 finding? YES, all nine, verified line by line against /scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_sizeproof.md:
- I1 (§7.7(b) orphan): written in P2 (L112-115, with the module-count pin), gated at P3 (L130-132). Closed as scheduling; see the Important for the mechanism.
- I2 (P1 does not compile): qrPlacement + qrPlaceAt moved to P1 (L45-46), removed from P2 (L84-90), and §2.1.1's third row named as an ERROR return (L50-51). Verified compilable: fitBlocksAt ALREADY returns (Fitted, error) at backup/fit.go:224, so the error return forces no caller change — FitBlocks (fit.go:186) and FitBlocksAt (fit.go:219) already handle it.
- I3 (blocks_test.go:406-417): booked in P1 at L59-65 with the assertions preserved.
- I4 (after P5 vs After P6): L35 now reads "after P6 (§3)" and adds "There is no P5 checkpoint review"; §3's heading unchanged. No remaining occurrence of "after P5".
- m5 (:296-309): L66-70, including "the Faces guard is evaluated before the Sizes guard".
- m6 (unnamed P2 sites): L92-106. Partially — see the Minor.
- m7 (round-3's I2): L147-149, split exactly as adjudicated.
- n8 (§7.7(d) FitSized half): L119-120, recorded vacuous.
- n9 (P5 double-owned rows): L164-166; "P2-P3" is more accurate than round 0's suggested "P1/P3" given the P2 forcing.

QUESTION 2 — new defects: one Important, three Minor, two Nit. The Important is in the §7.7(b) MECHANISM the fold introduced, not in its scheduling: a regression pin authored in a phase where it is green by construction, gated in a phase where correct code also leaves it green, under an instruction ("RED-then-green here") that cannot be executed, with no shape constraint beyond module count. The fix is two sentences and re-cuts no phase.

MEASURED (probe run and deleted, backup/zz_foldreview_probe_test.go): at prodParams, 3.0 mm, sh.Font, an 89-module code — holeLines 3, qrLines 20, band = plate rows 3..22, charPerQRLine 12 vs charPerLine 44, rows 0-2 and 24-25 are screw-hole rows (inset 45848, n=36), and ROW 23 IS A FULL-WIDTH 44-COLUMN ROW BELOW THE BAND. That last fact is why §7.7(b)'s "no body ink enters the code box" must be a rectangle-intersection assertion, not a max-x bound: the max-x formulation fails on a correct plate and invites the implementer to weaken it into something the defect also satisfies.

ALSO VERIFIED INDEPENDENTLY (cheap, load-bearing for P1): grepped every `Fitted{` in the tree — backup/blocks_test.go:298 and :414 are the only hand-built literals in backup, gui/freetext_flow_test.go:564/893/928 the only ones outside it. Matches round 0 exactly; P1's fixture task is complete. Also confirmed no `sizeproof` branch or worktree exists, so no code has started ahead of the gate.

Repo: /scratch/code/shibboleth/seedhammer @ 6d57681, `git status --short` empty on entry and on exit, probe deleted.

---

## Lane 2 — fold coverage (sonnet)

VERDICT: GREEN — 0 findings

*(no findings)*

### Notes

GREEN. All nine round-0 findings FIXED; no new defect found; all added citations verified against /scratch/code/shibboleth/seedhammer @ 6d57681; internal consistency restored.

PER-FINDING VERDICT (quoting the fold):

1. (I) §7.7(b) orphan — FIXED. P2: "Also written here, but gated at P3: §7.7(b), the two-block-plus-QR fixture. It is cheap to build now and passes vacuously in P2 ... Pin its code by MODULE COUNT, not text length (§7.7(b))." P3 gate: "§7.7(b) a two-block plate with a QR wraps block 2 at the code's own budget, and no body ink enters the code box." Matches spec §7.7(b) wording (incl. "MODULE COUNT, not by text length") verbatim in intent.

2. (I) P1 didn't compile (qrPlacement/qrPlaceAt in P2) — FIXED. P1 now: "`Block.SizeMM` (§2.2). **`qrPlacement` and `qrPlaceAt` (§2.1)** — the type must land here, not in P2, because `Fitted` declares a field of it in this phase." P2's line no longer declares qrPlacement/qrPlaceAt (diff confirms removal). Also lands §2.1.1's third row: "Guards added: ... §2.1.1's **two panics** *and* its third row, the `qrAt.Bottom <= plateHeight - margin` **ERROR return** from `fitBlocksAt`." Verified against spec §2.1.1's table (rows: two panics + one error-return row) — accurate, 3 rows total.

3. (I) blocks_test.go:406-417 no recover — FIXED. P1: "backup/blocks_test.go:406-417 — `mk` returns a `Fitted` with `Lines` and `Faces` and no `Sizes`, then engraves it twice **with no `recover`**. ... It gains `Sizes`; its per-row-inset assertions keep their exact form." Verified against source: `mk` (lines 406-416) builds `Fitted{SizeMM, Lines, Faces, TitleFace, FooterFace}` with no `Sizes`; lines 417-418 call `EngraveFitted(P, mk(...))` twice with no recover/defer anywhere in the function (confirmed by reading through line 430). Accurate.

4. (I) §0 "after P5" vs §3 "After P6" — FIXED. Line 33: "**Review:** opus + sonnet two-lane on the whole diff **after P6** (§3) ... There is no P5 checkpoint review; P6 carries §7.3's composition pins and the §7.19 mutation pass." §3 heading unchanged: "## 3. After P6". No remaining "after P5" text anywhere in the file (grepped).

5. (m) blocks_test.go:296-309 wrong-reason pass — FIXED. P1: "backup/blocks_test.go:296-309 — same omission, but it already defers a `recover` ... It gains a correctly-sized `Sizes`, and **the `Faces` guard is evaluated before the `Sizes` guard** so it keeps isolating the face map." Verified: lines 296-309 build `Fitted{SizeMM:3.0, Lines:[2 entries], Faces:[1 entry]}` and defer a recover checking only `recover() != nil` — matches.

6. (m) P2 unnamed forced call sites — FIXED. P2: "The signature change forces four more sites into this same commit ... `wrapBlocks` (`fit.go:150`) and `faceLayouts.at` (`fit.go:327`) thread a `*qrPlacement` instead of a `*qr.Code`, which makes `MaxCharsAtBlocks` a placement producer in P2 rather than P3 ... Three test fixtures move too, and their assertions and numbers are preserved exactly: `backup/engravetext_test.go:159-196` ... `backup/freetext_test.go:195-197` and `:289`." All citations verified (see check A below).

7. (m) round-2/round-3 I2 split — FIXED. P3: "Round-2's I2 also lands here whole. Round-3's I2 is split: its design half is P2's (the `AdmissibleBlocks` anchor), and only its PIN — §7.20 with `useQR` — closes here." Matches the merged report's exact recommended rewording.

8. (n) §7.7(d) FitSized half unowned — FIXED. P2 gate: "§7.7(d), **`fitBlocksAt` half only** — the `FitSized` half is vacuous by §2.7 (`qrAt` is always nil there) and is recorded as such rather than scheduled into P4." Verified against spec §2.7/producer table: "`FitSized` | no QR at all (§2.7); leaves `qrAt` nil" — accurate.

9. (n) P5 "every row" double-owned — FIXED. P5: "Every **GUI and preview** row of §3's table — the backup-package rows close earlier: `EngraveFreeText` in P1, `AdmissibleBlocks`/`rowFaces`/`MaxCharsAtBlocks` in P2-P3."

CHECK A — fixture/source-fact verification (repo @ 6d57681, git status clean before and after; I wrote no probe files):
- blocks_test.go:406-417: `mk` (406-416) builds `Fitted{SizeMM, Lines, Faces, TitleFace, FooterFace}`, no `Sizes`; :417-418 call `EngraveFitted` twice, no recover in the whole test function (386-430 read in full). Confirmed as claimed.
- blocks_test.go:296-309: literal at 298-302 `Fitted{SizeMM:3.0, Lines:[]string{"one","two"}, Faces:[]*vector.Face{sh.Font}}`; defer/recover block at 303-307 checking `recover() == nil`. Confirmed defers a recover.
- engravetext_test.go:159-198 (plan cites 159-196; core literal 161-172 and clamp assertion 174-177 both fall inside 159-196): `lineLayout{... holeLines: 2, qrLines: 19 ...}` at lines 165-166, and `n, offx := lay.at(18); if n != 1 {...}` — this is the n<1 clamp pin, confirmed by the test's own doc comment ("TestLineLayoutClampsBudgetToOne ... the n < 1 clamp").
- freetext_test.go: `grep -n "holeLines\|qrLines"` returns exactly lines 195, 197, 289 — matches "195-197 and :289" precisely; both use `lay.holeLines`/`lay.qrLines` to recompute a QR y-coordinate.
- wrapBlocks (fit.go:150), faceLayouts.at (fit.go:327), EngraveText (backup.go:359) — grepped every `textLayout(` call site in backup/*.go: exactly these three non-test callers plus three test callers (blocks_test.go:30,393, fit_test.go:15) plus the definition (wrap.go:164). List is complete — no fourth non-test caller exists.
- gui/freetext_flow_test.go:564/893/928 (cited unchanged from round 0, carried into P1's audit sentence) — verified: all three build `backup.Fitted{...}` literals feeding `ftConfirmBody`/`ftConfirmRows`; `grep -n EngraveFitted gui/freetext_flow_test.go` shows only comment references (line 28), never a call — confirmed safe as claimed.

CHECK B — every file:line/§x.y citation the fold added resolves and matches: confirmed for all of the above, plus §7.7(a)-(d) and §2.1.1's three-row table cross-checked directly against SPEC_sizeproof.md (lines 275-287, 873-897) — wording (MODULE COUNT pin, ERROR-not-panic, qrAt always nil in FitSized) matches spec verbatim in substance.

CHECK C — internal consistency: no remaining "after P5"/"P0, awaiting" text (grepped); Status line updated to "P1, folding plan R0 round 0 (RED, 0C/4I)"; P1-P6 phase count consistent throughout; full §7-item-to-phase gate map post-fold is exhaustive with no orphan (1,2,3,4,5,6,7a,7b,7c,7d,8-20 all present exactly once across P1-P6, matching round 0's recommended target). No new self-contradiction introduced by the fold.

Repo /scratch/code/shibboleth/seedhammer: git status --short clean on entry and exit (6d57681, "gui(passphrase): rename the FONTPROOF! trigger to PASSPROOF!"); no probe files created by this lane.
