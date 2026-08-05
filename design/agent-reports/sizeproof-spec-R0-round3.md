# R0 architect review — SPEC_sizeproof.md — round 3 (fold check)

Two independent lanes dispatched 2026-08-05 against the R3 spec @ `38b7a84`,
scoped to "did the fold fix each round-2 finding, and did it introduce a new
defect". Opus design-adversarial + sonnet mechanical, merged by an independent
opus synthesiser. Rounds 0 and 1 declared closed in the brief; round 2's
confirmed measurements declared settled so they would not be re-derived.
Persisted VERBATIM before folding.

VERDICT: **RED** (0C / 2I / 4 Minor / 3 Nit)

All thirteen round-2 items (5I/5M/3N) are confirmed folded. Both Importants
below are NEW gaps opened by the R3 fold itself.

---

## MERGED GATE VERDICT (synthesiser)

VERDICT: RED — 9 findings

### 1. Important — §3.1 (the edit-path decision) + §7.13

**The revert predicate `len(out) != len(p.Runs)` detects only DELETED newlines. An INSERTED newline (parts > runs) still yields exactly `len(p.Runs)` blocks, so the guard reports "exact shape", the ladder is KEPT, and every part after the insertion is stamped with the NEXT run's face and size. §7.13 therefore says "all THREE shapes" when there are four.**

*Failure:* Traced `(*ftPlan).Blocks` (gui/freetext_flow.go:107-135) by hand against a six-run BACK ladder plan (Blocks:1 per non-final run). parts=7 -> i=0..4 each consume one part, i=5 is last so n=len(parts)=2 -> out has exactly 6 blocks, len(out)==len(p.Runs). Operator loads SIZEPROOF!BACK and splits the first sweep across two lines (the text keyboard has a newline key, gui/passphrase_keyboard.go:78). Result: block0 = first half of the sh@4.4 sweep in sh; block1 = SECOND half of that sh sweep cut in font/constant @4.4; block2 = the const@4.4 sweep cut in sh @3.4; ... block5 = the sh@3.0 sweep AND the const@3.0 sweep joined, both cut in font/constant @3.0 -- the sh@3.0 rung is absent entirely. Every block carries a size, so ftFitAt routes to FitSized and permanent steel is cut under a title reading `BACK 4.4+3.4+3.0` whose bands are engraved in the wrong faces. The confirm screen cannot expose it: it reads rungs from Fitted.Sizes (§3) and six rungs are present, exactly as expected. This is strictly worse than the 5-part partial ladder §3.1 explicitly rejects ("a plate that proves less than its own title claims"), and §3.1's stated guarantee -- "any shape-mismatched edit reverts the WHOLE plate to uniform auto-fit" -- is not met by the predicate it names.

*Fix:* Strengthen the predicate: keep the ladder only when `len(out) == len(p.Runs)` AND no run absorbed more than its declared share -- for the ladder plans (Blocks:1 per non-final run) that is `strings.Count(text, "\n")+1 == len(p.Runs)`. Keep the clearing scoped to SizeMM so ftPlanBoth/ftBothPlanFor face behaviour is untouched (both carry no sizes today, so this is a no-op for every shipped plan). Restate §7.13 as FOUR shapes and add: 7 parts (an inserted newline) reverts to uniform auto-fit and does NOT cut a ladder with the faces shifted one run up.

### 2. Important — §2.1 (the anchorY table) + §7.20

**`AdmissibleBlocks` is a QR-carrying `wrapBlocks` caller and is missing from §2.1's `anchorY` table, which is written as an exhaustive per-caller enumeration ("The anchor is the CALLER's decision and each keeps the one it has"). §2.4 meanwhile drills a DIFFERENT value -- `params.I(outerMargin) + params.F(size)` -- into the reader as its `start`, and §7.20's pin has no QR case, so nothing catches the confusion.**

*Failure:* After §2.4, `wrapBlocks` receives `qrp *qrPlacement` built by the caller. `AdmissibleBlocks` (backup/fit.go:262-281) encodes its own code via `qrFor(CompositionText(blocks), useQR)` and calls `wrapBlocks(..., 1, math.MaxInt)`, so it must construct a placement -- but §2.1's table names only wrapBlocks / EngraveFitted / MaxCharsAtBlocks / rowFaces, and §2.3 names only fitBlocksAt and EngraveFreeText as qrAt setters. The nearest value in scope, and the one §2.4 just handed this exact function, is `start`. Checked the arithmetic: with anchorY = params.I(outerMargin), Top = margin + holeLines*fs and rows at y = start + j*fs reduce the new predicate to `holeLines <= 1+j < holeLines+qrLines`, term-for-term today's `lay.at(1+i)`; with anchorY = start the whole band shifts down exactly one row, so the rows at both band edges swap charPerLine for charPerQRLine, `len(l)` moves and `linesUsed`/`ok` change. That verdict gates OK on BOTH the text step and the confirm step (gui/freetext_flow.go:195 -> f.ok), on ordinary QR-carrying operator plates -- the path seeds and passphrases take -- so an admissible plate is refused (or an inadmissible one accepted) with the readout disagreeing with the fit. §7.20 is specified as "3.0 mm in `sh`" with no QR, where the placement is nil and the error is invisible.

*Fix:* Add the row to §2.1's table: `AdmissibleBlocks` -> `params.I(outerMargin)`, and state explicitly that its `start` (§2.4) and its `anchorY` (§2.1) are deliberately different values -- the title-row reservation moves the first ROW, not the CODE. Extend §7.20 to pin `linesUsed`/`ok` with `useQR=true` as well as without.

### 3. Minor — §2.1.1 (third guard) vs its own preamble and §7.7(d)

**§2.1.1 says all three invariants are "enforced in `EngraveFitted`" and §7.7(d) tests them as panics, but the third bullet claims the effect is that an oversized code "is refused at the fit rather than by `toPlate` after the operator has approved it". A panic in EngraveFitted is neither a refusal nor before approval.**

*Failure:* EngraveFitted is reached from ftBuildPlate (gui/freetext_flow.go:643-651) only AFTER the confirm screen, so implementing the guard where the preamble and §7.7(d) put it gives a panic mid-flow with a plate clamped in the machine -- the exact C5 failure mode §2.3 exists to abolish -- and is strictly worse than today's toPlate refusal, not better. Held to Minor only because it is unreachable from a shipped fit: violating `Bottom <= plateHeight - margin` at 3.0 mm needs roughly a 108-module code (>1000-byte text) on a plate that holds ~247 characters beside such a code, so FitBlocks refuses first.

*Fix:* Pick one and make §7.7(d) match: either state that `Bottom <= plateHeight - margin` is validated in `fitBlocksAt`/`FitSized` as an error return and only re-asserted as a panic in EngraveFitted, or drop the "refused at the fit" clause and call it a defensive assertion.

### 4. Minor — §2.3 / §3 table

**`fitBlocksAt` (backup/fit.go:224-241) is never told to populate `Sizes`, `TitleSizeMM` and `FooterSizeMM`. §2.3 spells the fill rule out for `EngraveFreeText` and §3's table gives it a row; the constructor every FitBlocks/FitBlocksAt plate -- i.e. every ordinary operator plate -- goes through gets neither.**

*Failure:* With §2.3's guards implemented literally and fit.go's Fitted construction left as it is, `len(Sizes) == 0 != len(Lines)` panics on the first golden, and any titled plate panics on the `TitleSizeMM != 0 iff Title != ""` invariant. §2.3's general rule ("`Sizes` is **always** populated") covers it in spirit, but the R3 fold added an explicit instruction for one constructor and not the other, which reads as "only EngraveFreeText needs changing".

*Fix:* Add a §3-table row (or a sentence in §2.3): `fitBlocksAt` fills `Sizes` with `len(lines)` copies of `size`, sets `TitleSizeMM`/`FooterSizeMM` to `size` when the corresponding string is non-empty and 0 otherwise, leaves `Mixed` false, and sets `qrAt` from `qrPlaceAt` at `anchorY = outerMargin` when there is a code.

### 5. Minor — §3 table / §3.1

**Nothing in R3 states that a ladder `ftProofOutcome` carries `SizeMM == 0`, which both §3.1's revert path and §3's ftFitAt routing depend on; `ftProofLoader` (gui/freetext_proof.go:~665-670), which writes `*size = out.SizeMM` into the flow's rung variable, has no row in the §3 table.**

*Failure:* §3.1's revert reads "No block carries a size, `ftFitAt` routes to `FitBlocks`" -- true only if the flow's `size` is 0. If a ladder outcome carried a rung, §3's own rule ("a non-zero `size` together with sized blocks is an error") would turn the UN-edited SIZEPROOF!FRONT/BACK path into an error rather than a plate. The dependency is load-bearing in both directions and is left implicit in a table §3 presents as "the whole path, not just the prompt".

*Fix:* Add an `ftProofLoader` row and one sentence: the ladder proofs are not `Sizeable`, so `ftProofForTrigger` returns rung 0, `ftProofOutcome.SizeMM` is 0 and `*size` is written 0 -- which is what makes §3.1's revert reach `FitBlocks` and the un-edited path reach `FitSized`.

### 6. Minor — §7 test plan (item 20) vs §2.4

**§2.4 fixes BOTH broken `start` translations (`AdmissibleBlocks` and `rowFaces`), but §7's item 20 pins a regression test for `AdmissibleBlocks` only. `rowFaces` (fit.go:302) has the identical defect class: the literal `0` meant "baseY = margin" under row-index semantics and means "plate absolute top, no margin" under §2.4's device-unit y.**

*Failure:* If an implementer updates one call site and not the other, no test in §7 fails. The one existing test that touches rowFaces' output, TestMaxCharsAtBlocksCountsEachRowInItsOwnFace (backup/blocks_test.go:347), asserts only a loose bracket (mixed strictly between allSH and allConst), not an exact value, so a face-boundary row shifted by a wrong baseY passes. Minor rather than Important because rowFaces feeds MaxCharsAtBlocks, which is the advisory "dropping the QR frees ~N characters" figure in a refusal message, not the gating admission decision.

*Fix:* Extend item 20 (or add a companion): pin `MaxCharsAtBlocks`/`rowFaces` output at a fixed rung for a mixed-block composition whose face boundary falls on a screw-hole row, so the figure cannot silently move if the `0` -> `params.I(outerMargin)` translation is skipped.

### 7. Nit — §2.1 (final paragraph)

**"the only `Fitted` literals outside the package are two GUI test fixtures (`gui/freetext_flow_test.go:564, 893, 928`)" -- that is three literals at three distinct lines, in three distinct test functions.**

*Failure:* `grep -rn 'backup\.Fitted{' gui/ cmd/` returns exactly three hits: freetext_flow_test.go:564, 893, 928. The conclusion is unaffected -- all three feed only the readout and never engrave, so the `(QR == nil) == (qrAt == nil)` guard breaks no fixture, including :893, which sets `QR` with no placement.

*Fix:* "three GUI test fixtures".

### 8. Nit — §2.5 vs §2.4

**§2.5 engraves "the footer at `footerY` (§2.4)" while §2.4 states "**`limit` is the only name for this quantity; `maxY` is not used.**" -- §2.5 immediately introduces the third name §2.4 forbids.**

*Failure:* The two quantities also only coincide when `footer != ""`; with no footer `limit` is `plateHeight - margin`, which is not any footer's y, so the cross-reference is loose as well as off-name.

*Fix:* Either name the footer-branch expression `footerY` inside §2.4 and define `limit = footerY` in that branch, or have §2.5 say "at the same expression §2.4's footer branch computes for `limit`".

### 9. Nit — §3 table, `sizeLabel` row

**"`sizeLabel` (`cmd/plateview/main.go:98`) | prints the range for a mixed plate; `0.0mm` stays a defect" -- that site's zero branch prints "fixed layout", not "0.0mm" (main.go:99-101, verified).**

*Failure:* A Mixed plate makes `Preview.SizeMM` 0 (gui/preview.go copies `fitted.SizeMM`), so the observable defect at this site is a free-text ladder described as "fixed layout", not as "0.0mm". The prescribed fix (print the range) is right; only the named symptom is wrong, which matters because a test written against §7.18 would assert on the wrong string.

*Fix:* Say the zero branch currently reports "fixed layout" and that a Mixed plate must print the range instead; keep "0.0mm" as the symptom named for `ftSizeLabel`/`ftConfirmSummary`, where it is accurate.

### Notes

VERDICT RED on two Importants; everything else is recorded and does not gate.

(a) §2.6's 0.119 mm — REPRODUCED. Not re-run by me, but both reviewers ran independent throwaway probes in package backup and returned byte-identical device-unit integers on sh2.Params() (MM=6400): sh@5.0 fontSize 32000, charWidth 19104, holeChars 3, inset 57312 = 8.955 mm; sh@3.8 fontSize 24320, charWidth 14519, holeChars 4, inset 58076 = 9.0744 mm; delta 764 = 0.119375 mm. Direction confirmed too: the 5.0 mm inset IS the smaller, as §2.6 states. Two independent reproductions of the same four constants is stronger evidence than a third run, so I did not re-derive it. The magnitude, derivation and direction of the ONE new number in R3 are sound.

(b) Fold coverage of round 2 (5 Important / 5 Minor / 3 Nit) — all 13 addressed in R3, verified against quoted spec text and live source: I1 -> §3.1 rewritten with the correct min(parts,runs) rule (I re-traced ftPlan.Blocks by hand: 6->6, 5->5 missing the LAST run, 1->1 in run 0's face; the "always the last run" claim holds by construction since the loop consumes runs left-to-right and breaks on exhaustion); I2 -> §2.4's caller table now gives both AdmissibleBlocks and rowFaces their translations, and I independently re-derived the 2-vs-4 top-band row figures at 3.0 mm; I3 -> dedicated §3 row + §7.17(a); I4 -> unexported `qrAt *qrPlacement` on Fitted + the QR!=nil => !Mixed guard; I5 -> `fittedPreviewAt` row; M1 -> 0.73 replaced with the correct 0.119; M2 -> §1.2 rewritten consistent with §1.1's [20 26 26 26]; M3 -> EngraveFreeText fill rule spelled out; M4 -> §2.4's no-footer branch is now a real `if`; M5 -> §7.2/§7.14 given checkable mechanisms; N1 -> KeepOutX; N2 -> ftFitAt routing order; N3 -> preview.go:111-132 corrected. The ONLY residual fold gaps are the two Importants above, and both are new-in-R3 gaps rather than unfolded round-2 items: finding 1 is a hole in the predicate R3 chose when folding I1 (the shape it never considers is INSERTION, which R3's own "last run absorbing the remainder" sentence supplies the mechanism for); finding 2 is the caller R3 added to §2.4's table but not to §2.1's.

(c) Controller must DECIDE, not merely fix:
  - Finding 1: what an inserted-newline edit SHOULD do. §3.1's decision text ("any shape-mismatched edit reverts") already answers it -- revert to uniform auto-fit -- so the cheapest close is to make the predicate say what §3.1 already decided (`strings.Count(text,"\n")+1 == len(p.Runs)`) and add the fourth case to §7.13. But refusing outright is a defensible alternative and is a decision, not an edit.
  - Finding 3 (Minor): pick ONE home for the `Bottom <= plateHeight - margin` check -- error return at the fit, or panic in EngraveFitted -- and make §2.1.1's prose and §7.7(d) agree with the choice.

(d) Repo hygiene: /scratch/code/shibboleth/seedhammer was clean on entry (`git status --short` empty, HEAD 3c3a2ad) and clean on exit (empty, HEAD 3c3a2ad). I created no files and modified nothing; my work was read-only plus hand-tracing. Both upstream reviewers report deleting their temporary probes and leaving the repo clean, which matches what I observed on entry.

DROPPED / NOT PROMOTED:
  - Nothing was dropped for re-opening a settled fact -- neither review re-litigated the composition, rounds 0-1, or the confirmed measured numbers. Both explicitly honoured the settled-facts list.
  - The two reviews' "verified sound" material (band equivalence, §2.4's translations, the guards' choke point, the unbounded-sentinel soundness, the revert-vs-partial-ladder decision itself) is corroboration, not findings, and is not carried as entries.
  - Duplicate merged: the "two vs three GUI test fixtures" miscount was raised by both reviewers (opus against §2.1, sonnet against §2.3); it is one Nit against one sentence.
  - Nothing was inflated: the fixture miscount, the footerY naming and the sizeLabel symptom are prose errors with no effect on any plate, and stay Nits. The rowFaces test gap stays Minor because rowFaces feeds only the advisory refusal-message figure, not the admission verdict.
  - Neither Important was manufactured to justify a round: I verified both against the source myself. Finding 1 is a hand-trace of gui/freetext_flow.go:107-135 that anyone can rerun in a minute; finding 2 is confirmed by fit.go:262-281 building its own qrc, gui/freetext_flow.go:195 gating `f.ok` on it with useQR, and §7.20 pinning only the no-QR case. Finding 1 alone is dispositive; finding 2 rides along and would not on its own have been worth a round if the fold were otherwise clean -- but it is real, cheap to close (one table row plus a QR variant of §7.20), and should be folded in the same pass.
  - Neither Important was rated Critical. Finding 1's blast radius is a permanently wrong PROOF plate under a title that lies, reachable only via an operator edit -- an unmet guarantee of §3.1's own decision, but no seed, key or address is mis-engraved. Finding 2 is a wrong admission verdict (over- or under-reported line count) on ordinary QR plates -- a visible refusal or a fit that disagrees with the readout, not silent wrong ink.

---

## Lane 1 — design-level adversarial (opus)

VERDICT: RED — 8 findings

### 1. Important — §3.1 (the edit-path decision) and §7.13

**`len(out) != len(p.Runs)` detects only DELETED newlines. An INSERTED newline gives parts > runs, which yields exactly `len(p.Runs)` blocks — so the guard reports "exact shape", the ladder is kept, and every part after the insertion is stamped with the NEXT run's face and rung. §7.13 calls its three shapes "all THREE shapes" when there are four.**

*Failure:* Ran gui/freetext_flow.go:107-135 verbatim against a six-run ladder plan (Blocks:1 each). parts=8 -> 6 blocks, parts=7 -> 6 blocks, both with len(out)==len(p.Runs). Operator loads SIZEPROOF!BACK and splits the first sweep across two lines (the text keyboard has a newline key — passphrase_keyboard.go:78). parts becomes 7: block0=first half of the sh sweep @ sh/4.4 (right), block1=second half of the SH sweep cut in font/constant @ 4.4, block2=the const@4.4 sweep cut in font/sh @ 3.4, ... block5 = the sh@3.0 sweep AND the const@3.0 sweep joined, both cut in font/constant @ 3.0. Every block carries a size, so ftFitAt routes to FitSized and permanent steel is cut as a plate titled `BACK 4.4+3.4+3.0` on which the band labelled as one face is engraved in the other and the smallest sh rung is absent entirely. This is strictly worse than the 5-part partial ladder §3.1 rejects ("a plate that proves less than its own title claims"), and it is the one shape §3.1 never considers even though §3.1 itself derives the mechanism ("the last run absorbing the remainder").

*Fix:* Strengthen the predicate: clear SizeMM unless `len(out) == len(p.Runs)` AND no run absorbed more than its declared share — for the ladder plans (Blocks:1 per run) that is `strings.Count(text, "\n")+1 == len(p.Runs)`. Keep the clearing scoped to SizeMM so ftPlanBoth/ftBothPlanFor face behaviour is untouched. Add a fourth case to §7.13: 7 parts (an inserted newline) reverts to uniform auto-fit and does NOT cut a ladder with the faces shifted one run up; and restate §7.13 as four shapes.

### 2. Important — §2.1 (the anchor table) vs §2.4

**`AdmissibleBlocks` is a QR-carrying `wrapBlocks` caller and is absent from §2.1's per-caller `anchorY` table, which is written as exhaustive ("The anchor is the CALLER's decision and each keeps the one it has"). Nothing in R3 says what anchor it passes, while §2.4 emphatically drills `params.I(outerMargin) + params.F(size)` into the reader as its value.**

*Failure:* After §2.4, `wrapBlocks` takes `qrp *qrPlacement` built by the caller. `AdmissibleBlocks` (fit.go:269-280) encodes its own qrc and passes it, so it must build a placement — but §2.1's table names only wrapBlocks / EngraveFitted / MaxCharsAtBlocks / rowFaces, and §2.3 names only fitBlocksAt and EngraveFreeText as qrAt setters. The natural (and wrong) reuse is the `start` value §2.4 just gave it. Checked the equivalence: with anchorY = params.I(outerMargin) the new predicate reduces to `holeLines <= i+1 < holeLines+qrLines`, byte-identical to today's `lay.at(1+i)`; with anchorY = start the whole QR band shifts down exactly one row, so rows at the band edges get charPerLine instead of charPerQRLine (or vice versa) and `len(l)` changes. That is `linesUsed` and `ok` on an ordinary QR-carrying operator plate — the path seeds and passphrases take — refused or admitted wrongly, with the fit disagreeing with the readout. §7.20's fixture is specified as "3.0 mm in `sh`" with no QR, so it does not catch this.

*Fix:* Add the row to §2.1's table: `AdmissibleBlocks` -> `params.I(outerMargin)` (one code on one plate, the same anchor every other free-text caller keeps), and state that its `start` and its `anchorY` are deliberately different values. Extend §7.20 to pin `linesUsed`/`ok` with useQR=true as well as without.

### 3. Minor — §2.1.1 (third guard) vs its own preamble and §7.7(d)

**The section says all three invariants are "enforced in `EngraveFitted`", and §7.7(d) says "§2.1.1's three guards each panic when violated" — but the third bullet claims the effect is that an oversized code "is refused at the fit rather than by `toPlate` after the operator has approved it". A panic in EngraveFitted is neither.**

*Failure:* EngraveFitted is reached from ftBuildPlate (gui/freetext_flow.go:643-651) only AFTER the confirm screen, so implementing the guard where the preamble and §7.7(d) put it gives a panic mid-flow with a plate clamped in the machine — the exact C5 failure mode §2.3 exists to abolish — and is strictly worse than today's toPlate refusal, not better. Reachability is what keeps this Minor: I computed the threshold at 3.0 mm (holeLines*fs + qrLines*fs <= 79 mm => qrsz <= ~65 mm => ~108 modules, i.e. a >1000-byte text), and such a text needs ~1100 characters against a plate that holds ~247 with that code, so FitBlocks refuses first. The guard as written can never fire from a shipped fit.

*Fix:* Pick one: either state that `Bottom <= plateHeight - margin` is validated in `fitBlocksAt`/`FitSized` (error return) and only re-asserted as a panic in EngraveFitted, or drop the "refused at the fit" clause. Make §7.7(d) match whichever is chosen.

### 4. Minor — §2.3 / §3 table

**`fitBlocksAt` (fit.go:224-241) is never told to populate `Sizes`, `TitleSizeMM` and `FooterSizeMM`. §2.1 tells it to set `qrAt`, and §2.3 spells the requirement out for `EngraveFreeText` (a round-2 Minor fold) but not for the constructor every FitBlocks/FitBlocksAt plate — i.e. every ordinary operator plate — goes through.**

*Failure:* With §2.3's guards implemented literally and fit.go:231-240 left as it is, `len(Sizes) == 0 != len(Lines)` panics on the first golden and any titled plate panics on the `TitleSizeMM == 0` invariant. The general rule in §2.3 ("Sizes is always populated") covers it in spirit, but the R3 fold added an explicit instruction for one constructor and not the other, which is the asymmetry that reads as "only EngraveFreeText needs changing".

*Fix:* Add a §3-table row (or a sentence in §2.3): `fitBlocksAt` fills `Sizes` with `len(lines)` copies of `size`, sets `TitleSizeMM`/`FooterSizeMM` to `size` when the corresponding string is non-empty and 0 otherwise, and leaves `Mixed` false.

### 5. Minor — §3 table / §3.1

**Nothing in R3 states that a ladder `ftProofOutcome` carries `SizeMM == 0`, which both §3.1's revert path and §3's ftFitAt routing depend on; `ftProofLoader` (`gui/freetext_proof.go:670`), which writes `*size = out.SizeMM` into the flow's rung variable, has no row in the §3 table.**

*Failure:* §3.1's revert says "No block carries a size, ftFitAt routes to FitBlocks" — true only if `size == 0`. If a ladder outcome carried a rung, §3's own rule ("a non-zero size together with sized blocks is an error") would make the un-edited SIZEPROOF!FRONT/BACK path an error rather than a plate. The dependency is load-bearing in both directions and is left implicit in a table that §3 presents as "the whole path, not just the prompt".

*Fix:* Add a `ftProofLoader (freetext_proof.go:665-670)` row and one sentence: the ladder proofs are not `Sizeable`, so `ftProofForTrigger` returns rung 0, `ftProofOutcome.SizeMM` is 0 and `*size` is written 0 — which is what makes §3.1's revert reach FitBlocks and the un-edited path reach FitSized.

### 6. Nit — §2.5 vs §2.4

**§2.5 engraves "the footer at `footerY` (§2.4)" while §2.4 states "**`limit` is the only name for this quantity; `maxY` is not used.**"**

*Failure:* §2.4 defines exactly one name and forbids alternates; §2.5 immediately introduces a third. The quantities also only coincide when `footer != ""` (with no footer `limit` is `plateHeight - margin`, which is not any footer's y), so the cross-reference is loose as well as off-name.

*Fix:* Either name the footer-branch expression `footerY` in §2.4 and define `limit = footerY` in that branch, or have §2.5 say "at the same expression §2.4's footer branch computes for `limit`".

### 7. Nit — §2.1 (final paragraph)

**"the only `Fitted` literals outside the package are two GUI test fixtures (`gui/freetext_flow_test.go:564, 893, 928`)" — that is three literals, at three distinct lines.**

*Failure:* Counting error; the enumeration itself is correct (I grepped `Fitted{` repo-wide: those three plus in-package fit.go/freetext.go/blocks_test.go sites). None of the three reaches EngraveFitted, so the `(QR == nil) == (qrAt == nil)` guard breaks no fixture — including :893, which sets `QR` with no placement but only feeds the readout.

*Fix:* "three GUI test fixtures".

### 8. Nit — §3 table, `sizeLabel` row

**"`sizeLabel` (`cmd/plateview/main.go:98`) | prints the range for a mixed plate; `0.0mm` stays a defect" — that site's zero branch prints "fixed layout", not "0.0mm" (main.go:99-101).**

*Failure:* A Mixed plate makes `Preview.SizeMM` 0 (preview.go:167 copies `fitted.SizeMM`), so the observable defect at this site is a free-text plate described as "fixed layout", not as "0.0mm". The prescribed fix (print the range) is right; only the named symptom is wrong, which matters because §7.18's assertion would be written against the wrong string.

*Fix:* Say the zero branch currently reports "fixed layout" and that a Mixed plate must print the range instead; keep "0.0mm" as the symptom named for `ftSizeLabel`/`ftConfirmSummary`, where it is accurate.

### Notes

Entry and exit `git status --short` in /scratch/code/shibboleth/seedhammer were both empty; the one temp file (backup/zz_r3check_test.go) and the scratch trace program were deleted.

VERIFIED SOUND (do not re-derive next round):

- §2.6's new numbers reproduce EXACTLY on sh2.Params() (MM=6400): sh@5.0 fontSize 32000, charWidth 19104, holeChars 3, inset 57312 = 8.9550 mm; sh@3.8 fontSize 24320, charWidth 14519, holeChars 4, inset 58076 = 9.0744 mm; delta 764 device = 0.1194 mm, and the 5.0 mm inset is the SMALLER. §2.6's derivation, magnitude and direction are all correct.
- §3.1's description of `ftPlan.Blocks` is now RIGHT. Ran gui/freetext_flow.go:107-135 verbatim against a six-run Blocks:1 plan: 8->6, 7->6, 6->6, 5->5, 2->2, 1->1 (run 0's face), last run absorbing the remainder. The "min(parts, runs)" claim and all three traced cases (6, 5, 1) hold. The `n <= 0` continue can only shorten `out`, so it can never masquerade as the exact shape; a run with Blocks > 1 (ftPlanBoth, ftBothPlanFor) also cannot — verified 5/4/3/2/1 parts against a 2-run Blocks:3 plan. The one hole is parts > runs (finding 1).
- Clearing SizeMM is a no-op for every plan shipping today: ftPlanSH/ftPlanConst are single-run (early return), and ftPlanBoth/ftBothPlanFor carry no sizes. No existing behaviour changes.
- §2.4's two caller translations are correct. `AdmissibleBlocks` passes `wrapBlocks(..., 1, math.MaxInt)` (fit.go:280) against a layout built at `params.I(outerMargin)` (fit.go:150), so today's first wrapped row sits at y = margin + fontSize = 38400 at 3.0 mm — exactly `params.I(outerMargin) + params.F(size)`; both halves of `lay.at`'s holeLine predicate reduce term-for-term. `rowFaces` passes 0 (fit.go:302) -> `params.I(outerMargin)`. The `linesAvail = rows - 2` reservation is independent of `start` (the wrap is unbounded), so there is no double-reservation. The `math.MaxInt` sentinel stays sound as a y: `(MaxInt - y)/fontSize` with y ~19200 cannot overflow and cannot truncate.
- §2.1.1's guards fire at the one choke point: every production engrave goes through `backup.EngraveFitted` (gui/freetext_flow.go:651, gui/preview.go:161), so no Fitted reaches the engraver having skipped them. `EngraveFreeText` has no non-test caller, so §2.3's instruction to populate its literal is golden-facing only, as the spec says.
- `Bottom <= plateHeight - margin` refuses nothing that ships: violating it at 3.0 mm needs a ~108-module code (>1000-byte text) on a plate that holds ~247 characters beside such a code, so the fit refuses first. Guard is defensive only (see finding 3).
- The revert-vs-partial-ladder decision itself is defensible as argued for the DELETION case: the confirm screen reads rungs from Fitted.Sizes (§3), so a revert shows one rung where the ladder showed several, whereas a five-rung partial plate differs from a six-rung one only in the bottom band. I attacked it and did not find a reason to overturn it — the defect is in the predicate's coverage (finding 1), not in the choice.

Verdict is RED on findings 1 and 2 only. Everything else is recorded and does not gate.

---

## Lane 2 — mechanical fold-vs-findings (sonnet)

VERDICT: GREEN — 2 findings

### 1. Minor — §2.4 / §7 test plan

**§2.4's table fixes both AdmissibleBlocks' and rowFaces' broken start-translation, but §7's new item 20 pins a regression test only for AdmissibleBlocks, not for rowFaces.**

*Failure:* rowFaces (fit.go:302) has the identical defect class as AdmissibleBlocks (fit.go:280): under row-index semantics literal `0` correctly meant baseY=margin (since old wrapBlocks always used params.I(outerMargin) as baseY internally); under §2.4's new device-unit-y semantics the same literal `0` means baseY=0, i.e. the plate's absolute top with no margin at all. If an implementer forgets to update this one call site the same way they might forget AdmissibleBlocks', §7 has no item that would catch it -- item 20 only pins AdmissibleBlocks' linesUsed/ok. The one existing test that touches rowFaces' output indirectly, TestMaxCharsAtBlocksCountsEachRowInItsOwnFace (backup/blocks_test.go:347), only asserts a loose bracket (mixed strictly between allSH and allConst), not an exact value, so it is not guaranteed to catch a face-boundary-row shift caused by a wrong baseY. Held to Minor rather than Important because the value rowFaces feeds (MaxCharsAtBlocks) is purely the advisory 'dropping the QR frees ~N characters' refusal-message figure, not a gating admission decision -- the actual cut/refuse decision goes through AdmissibleBlocks, which item 20 does cover.

*Fix:* Add a companion assertion (or extend item 20) pinning MaxCharsAtBlocks'/rowFaces' output at a fixed rung for a mixed-block composition whose face boundary falls on a screw-hole row, so the verdict cannot silently move if the `0` -> `params.I(outerMargin)` translation is skipped.

### 2. Nit — §2.3 (Fitted's qrAt field discussion)

**"the only `Fitted` literals outside the package are two GUI test fixtures (`gui/freetext_flow_test.go:564, 893, 928`)" undercounts: those three line numbers are three separate `backup.Fitted{...}` literals in three different test functions (TestConfirmLinesAreOwnUnwrappedLabels, TestFTConfirmReservesRoomForTheWarnings, TestFTConfirmPagesEveryRowExactlyOnce), not two.**

*Failure:* grep -rn "backup\.Fitted{" . (outside backup/) returns exactly three hits, at freetext_flow_test.go:564, 893 and 928, each inside a distinct top-level test function. "Two" does not match the three cited lines/functions. Does not affect the conclusion (all three literals do feed only the readout and never engrave, so 'unexported is right' still holds), but the sentence miscounts its own citation list.

*Fix:* Change "two GUI test fixtures" to "three" (or name the three test functions).

### Notes

SCOPE A (fold coverage, round-2's 5 Important/5 Minor/3 Nit) -- ALL 13 addressed, each verified with quoted R3 text and cross-checked against source:
I1 (edit path) -> §3.1 entirely rewritten with the correct min(parts,runs) rule and the 'clears SizeMM on shape mismatch' decision; verified myself by hand-tracing the REAL `ftPlan.Blocks` (gui/freetext_flow.go:107-135) against 6/5/1-part inputs -- results match §3.1 exactly (6->6 blocks, 5->5 blocks missing run 5/const@3.0, 1->1 block carrying run 0's face/size), and the claim 'the rung that goes missing is always the LAST run' holds by construction (the loop consumes runs left-to-right and breaks the moment parts run out, so any missing runs are always a trailing suffix that includes the last run).
I2 (start translations) -> §2.4's table now states `AdmissibleBlocks` (fit.go:280) -> `params.I(outerMargin)+params.F(size)` and `rowFaces` (fit.go:302) -> `params.I(outerMargin)`; §7's new item 20 pins AdmissibleBlocks' verdict. Verified fit.go:280/297/302 citations exact, and independently re-derived the '2 vs 4 rows in the top band at 3.0mm' numbers by hand (baseY=1 device-unit -> rows at y=1,19201,38401,57601, four <64000; baseY=38400 -> rows at 38400,57600,76800, two <64000) -- matches spec exactly.
I3 (ftConfirmSummary) -> now a dedicated §3 table row plus §7.17(a) asserting no '0.0mm'. Verified freetext_flow.go:485 is exactly `func ftConfirmSummary(...)` and it does format `f.plate.SizeMM` with `%.1fmm` today, confirming the original hazard and that the fix targets the real line.
I4 (qrPlacement channel) -> `Fitted` gains an unexported `qrAt *qrPlacement` field (§2.3), populated by fitBlocksAt/EngraveFreeText, and §2.1.1 adds the `QR!=nil => !Mixed` guard; §7.7(c) rewritten to be assertable. Verified freetext.go:76-89 (today's EngraveFitted draws the QR via a locally re-derived lay, exactly the defect described) and freetext.go:33/97/103-113 citations.
I5 (fittedPreviewAt) -> new §3 table row 'fittedPreviewAt (gui/preview.go:149) | routes through ftFitAt, not FitBlocks/FitBlocksAt directly'. Verified preview.go:149-155 today does exactly `fit := backup.FitBlocks; if size != 0 { fit = FitBlocksAt(...) }`, confirming the defect and citation.
M1 (0.73mm wrong) -> replaced with 0.119mm plus the band-mis-classification hazard called out as 'the larger hazard'. VERIFIED BY PROBE (see below): reproduces exactly.
M2 (§1.2 contradiction) -> rewritten to 'both configurations start sh@5.0 INSIDE the top band ... What the title buys is the SECOND row'. Matches §1.1's own [20 26 26 26] table exactly.
M3 (EngraveFreeText populate fields) -> §2.3 spells out the fill rule (Sizes = len(lines) copies of fontMM, TitleSizeMM/FooterSizeMM set from the same value when the string is non-empty) plus a new §3 table row 'EngraveFreeText (freetext.go:97) | populates Sizes, TitleSizeMM, FooterSizeMM, qrAt'. Verified freetext.go:97/103-113 citations exact.
M4 (§2.4 pseudocode no-footer branch) -> now an explicit `if footer != "" { ... }` conditional, not a trailing comment. Confirmed by reading the current §2.4 code block.
M5 (§7 items 2/14 mechanism) -> item 2 now names 'the b-spline bounds helper the geometry tests in backup/ already use'; item 14's second half reworded to the checkable 'do not panic and produce limit == plateHeight - margin'. Both match round-2's suggested fix text verbatim.
N1 (qrPlacement qrBorder) -> struct gained `KeepOutX int // Size + 2*qrBorder`, satisfying the suggested fix's first alternative (carry the keep-out rather than qrBorder itself); algebraically sufficient to reconstruct charPerQRLine = (width-KeepOutX)/charWidth.
N2 (ftFitAt routing order) -> new paragraph 'ftFitAt's routing order matters ... The per-block-size test comes first, and a non-zero size together with sized blocks is an error'. Matches suggested fix.
N3 (preview.go:111-133 off-by-one) -> citation corrected to 111-132 everywhere it appears in §3. Verified proofPreview spans exactly lines 111-132 in current source (closing brace at 132, blank line 133).

SCOPE B (the one new number, §2.6) -- REPRODUCED EXACTLY by a throwaway probe in package backup (created and deleted; repo clean before/after, see below): sh@5.0 fontSize=32000, charWidth=19104, holeChars=3, inset=57312 device units = 8.955mm; sh@3.8 fontSize=24320, charWidth=14519, holeChars=4, inset=58076 = 9.074mm. Delta = 764 device units = 0.119mm exactly (58076-57312=764; 764/6400=0.119375, rounds to the stated 0.119mm). The 5.0mm inset (8.955) IS the smaller one, confirming the spec's 'so the 5.0mm inset is the smaller' claim. All matches §2.6 verbatim.

SCOPE C (citations) -- every file:line the task listed was checked against the live source at HEAD (seedhammer main, working tree clean). ALL CORRECT except the fixture-count Nit above: freetext_flow.go:56/107-135/146/204/218/485 all land exactly on the named declarations; fit.go:150/255-261/262/280/297/302/316-331/344 all exact; freetext.go:33/34-39/71/97/103-113 all exact; backup.go:81/90-96/359/383-385/390-393 all exact; wrap.go:133-158/135/140-141/167-176/181-182 all exact (181-182's formula is technically on line 182 alone but 181 supplies qrsz used in it -- not a real error); preview.go:111-132/130/149-155 all exact (the R3 fold corrected round-2's 111-133 nit); plateview/main.go:98 exact (func sizeLabel); freetext_flow_test.go:564/893/928 exact line numbers (only the '"two" fixtures' prose miscounts, see Nit above); freetext_proof.go:367/511/531/538 all exact.

SCOPE D (§3.1 trace) -- CONFIRMED by manual trace of the real `(*ftPlan).Blocks` algorithm (gui/freetext_flow.go:107-135) against a 6-run BACK-shaped plan with each non-final run's Blocks=1: 6 parts -> 6 blocks (one per run); 5 parts -> 5 blocks, always missing the LAST run (const@3.0) regardless of which newline was deleted, because the loop consumes runs left-to-right and `break`s the instant parts are exhausted, so any deficit is always a trailing suffix; 1 part -> 1 block carrying run 0's face (and, per §3's stamping rule, run 0's size). Also confirmed the doc-comment claim: for a 2-run plan (today's shipped BOTHPROOF! shape) with 1 part, the code does collapse to exactly one block in run 0's face -- so the stale doc comment ('collapses to a single block in the first run's face') is accurate for 2-run plans and provably inaccurate for plans with more runs, exactly as §3.1 states.

SCOPE E (test plan, 20 items) -- items 2, 7, 13, 14, 17, 18, 20 (R3-rewritten/added) all individually reviewed for falsifiability: each names a concrete mechanism or observable outcome and would fail if the defect it targets were present (item 2 names the b-spline bounds helper; item 7(a)-(d) separately covers the no-golden-moves proof, the round-1 QR-anchor Critical, the now-writable §7.7(c) placement assertion, and the three §2.1.1 guard panics; item 13 asserts on `Fitted.Sizes` post-`ftFitAt` rather than on the block list, so it would fail if routing silently sent a sizeless composition to FitSized; item 14 is reworded to the checkable panic/limit-value form; item 17(a) specifically forbids '0.0mm' in the rendered ftConfirmSummary, closing the exact gap I3 found; item 18 compares the plateview preview's per-row Sizes against the device's Fitted.Sizes for the bare `-size`-less case, closing I5's gap; item 20 pins AdmissibleBlocks' verdict at exactly the boundary I2's bug would flip). No item in this set is untestable, true-by-construction, or a false-pass slot as worded. One coverage gap noted above (rowFaces, Minor).

REPO HYGIENE: /scratch/code/shibboleth/seedhammer was clean (`git status --short` empty) on entry and on exit. I created exactly two temporary files during this review -- backup/zzprobe_r3_test.go (§2.6 number reproduction) and a throwaway constant-lookup probe -- both deleted before finishing; no tracked file was modified. HEAD unchanged throughout.

OVERALL: 0 Critical, 0 Important found against R3. Two non-gating observations recorded (1 Minor test-coverage gap, 1 Nit prose miscount). This closes the SIZEPROOF! spec R0 gate per the proportional re-review rule -- a clean mechanical result on a proportionally-scoped re-review should close the loop rather than trigger another round.
