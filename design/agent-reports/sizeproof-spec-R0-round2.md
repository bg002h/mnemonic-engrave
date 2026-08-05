# R0 architect review — SPEC_sizeproof.md — round 2 (fold check)

Two independent reviewers dispatched 2026-08-05 against the R2 spec @ `9680a64`,
scoped to "did the fold fix each round-1 finding, and did it introduce a new
defect" — not a fresh audit. Lanes per `CLAUDE.md`'s tiering table: an **opus**
design-level adversarial pass and a **sonnet** mechanical fold-vs-findings +
number-reproduction pass, merged by an independent **opus** synthesiser.
Persisted VERBATIM before folding.

VERDICT: **RED** (0 Critical / 5 Important / 5 Minor / 3 Nit)

---

## MERGED GATE VERDICT (synthesiser)

VERDICT: RED — 13 findings

### 1. Important — SPEC_sizeproof.md §3 ("The edit path — stated precisely") and §7.13, vs gui/freetext_flow.go:107-135

**§3's edit-path decision rests on a false description of `ftPlan.Blocks`: it does not collapse to one block when the text has fewer parts than the plan has runs, and the one case where it does collapse yields a block that (per §3's own table) carries a size — so neither of the "two different things" §3 enumerates is what the code produces.**

*Failure:* Traced against freetext_flow.go:107-135 with the FRONT plan's 4 runs [{sh,1},{const,1},{sh,1},{const,last}]: `Blocks` emits min(parts, runs) blocks, the last run absorbing the remainder. 4 parts -> 4 blocks; 3 parts -> 3 blocks [sh,const,sh]; 1 part -> 1 block. (a) MIDDLE CASE, undescribed and untested: the operator loads SIZEPROOF!BACK (6 runs), deletes one newline, presses OK. `Blocks` returns 5 blocks, §3's table stamps each with its own run's size, so every block carries a size, `ftFitAt` routes to `FitSized`, and permanent steel is cut as a FIVE-rung ladder with const@3.0 — the smallest, most load-bearing rung — silently absent. §3 says "Two different things follow"; this is a third, and §7.13 tests only the two. (b) COLLAPSE CASE: §3 asserts "The collapsed block carries no size, so `ftFitAt` routes to `FitBlocks`", but the same section's table says `ftPlan.Blocks` "stamps each block with its run's size", so the single surviving block carries run 0's size (5.0mm front / 4.4mm back) and routes to `FitSized`, which refuses or mis-cuts the 380-character text at 5.0mm instead of giving the promised auto-fit plate. Root cause: freetext_flow.go:107's own doc comment ("collapses to a single block in the first run's face") is TRUE for the shipped 2-run plans and FALSE for 4- and 6-run plans; the spec extrapolated it without re-checking.

*Fix:* State the real rule (`Blocks` yields min(parts, runs) blocks, last run absorbing the remainder) and decide the block-count != run-count case explicitly — e.g. `ftPlan.Blocks` clears `SizeMM` on every block whenever `len(out) != len(p.Runs)`, so any non-exact-shape edit reverts to uniform auto-fit. Add the partial-edit case (5 blocks of 6) to §7.13, and fix the stale doc comment while you are there.

### 2. Important — SPEC_sizeproof.md §2.4 vs backup/fit.go:280 and fit.go:302

**§2.4 redefines `wrapBlocks`' `start` from a row index to a device-unit y but supplies the translation only for the fitter; `AdmissibleBlocks` passes the literal `1` and `rowFaces` the literal `0`, both of which stay type-correct and silently come to mean "0.00016 mm / 0 mm from the plate top".**

*Failure:* Verified analytically at sh2.Params() (MM=6400, outerMargin=3mm=19200, innerMargin=10mm=64000, F(3.0)=19200). TODAY `AdmissibleBlocks` calls `wrapBlocks(..., 1, math.MaxInt)` with baseY=margin and row index 1, so rows land at y=38400, 57600, 76800...: exactly two rows fall in the top screw-hole band (y < 64000). The correct §2.4 translation is `start = params.I(outerMargin) + params.F(size)` = 38400, which reproduces that. The literal `1` carried over gives y=1, 19201, 38401, 57601...: FOUR rows in the band. At 3.0mm sh, charWidth=11462, holeChars=4, charPerLine=44, so each extra banded row drops from 44 to 36 columns — two extra narrowed rows at the top of every admission computation. `linesUsed` over-reports, and since `!f.ok` gates OK on both the text and the confirm step, an ordinary operator plate (seeds and passphrases go through this path) is refused with a line count it does not actually need. The same defect hits `rowFaces` (literal `0` vs the correct `params.I(outerMargin)`), shifting the face-boundary rows `MaxCharsAtBlocks` sums over. §2.4's only formula, `start = margin + F(titleSizeMM)`, does not apply to `AdmissibleBlocks`, which deliberately never reads the title. Nothing forces a compile error, because only the `sizes`/`qrp` parameters change type.

*Fix:* In §2.4 give both translations explicitly: `AdmissibleBlocks` passes `params.I(outerMargin) + params.F(size)` (its unconditional title-row reservation) and `rowFaces` passes `params.I(outerMargin)`. Add a test pinning `AdmissibleBlocks`' linesUsed at 3.0mm for a text sitting exactly at capacity, so the verdict cannot flip unnoticed.

### 3. Important — SPEC_sizeproof.md §3 (GUI table) vs gui/freetext_flow.go:484-489

**§3's table — introduced as "Every one of these must change" — misses `ftConfirmSummary`, which formats `f.plate.SizeMM` with its own `%.1fmm` and is the line on the screen the operator actually approves.**

*Failure:* `ftConfirmSummary` (freetext_flow.go:485) builds `fmt.Sprintf("%.1fmm  %d lines  QR: %s  font: %s", f.plate.SizeMM, ...)` independently of `ftSizeLabel` (freetext_flow.go:218), which is the only readout §3 lists. On a ladder plate `SizeMM` is invalid by §2.3 ("valid only when !Mixed"), so with only the changes §3 enumerates the confirm screen for SIZEPROOF!FRONT reads "0.0mm  16 lines  QR: no  font: sh 4 + constant 5 + sh 3 + constant 4" — precisely the "reader that prints `0.0mm` is a defect, not a fallback" §3 bolds two paragraphs later, on the one screen §3 names as what the operator approves. §7.17 only asserts that line FITS the panel by measuring rectangles, and a "0.0mm" line fits, so nothing in the test plan catches it. Same incomplete-enumeration class as R0's C6 and round 1's I4. This finding also gates finding 1: the per-run summary on this screen is the operator's only pre-engraving warning that a rung went missing.

*Fix:* Add a `ftConfirmSummary (gui/freetext_flow.go:485)` row to §3: it must print the ladder's rungs (or a range) rather than `SizeMM`, reusing whatever `ftSizeLabel` is changed to. Extend §7.17 to assert the rendered summary names the front's rungs and never contains "0.0mm".

### 4. Important — SPEC_sizeproof.md §2.1 + §2.3 + §7.7(c) vs backup/freetext.go:76-113

**The `qrPlacement` fix has no channel from the fit to `EngraveFitted` — §2.3's `Fitted` gains no placement field — so §2.1's "computed ONCE ... so the two cannot drift" is unenforced and §7.7(c)'s "asserted by construction" cannot be written.**

*Failure:* §2.1 requires the placement to be "read by BOTH the layout that narrows the lines and the engraver that draws the code", and §7.7(c) asserts "freetext.go's and backup.go's QR offsets both read `qrPlacement.Y` — asserted by construction, since neither may compute a y of its own". `EngraveText` can hold the placement in a local because it builds its own layout; `EngraveFitted` receives only `Fitted`, and §2.3 adds `Mixed`/`Sizes`/`TitleSizeMM`/`FooterSizeMM` but no placement. So freetext.go:81-85 MUST call `qrPlaceAt(...)` again — exactly the second derivation §2.1 forbids — and §7.7(c) is a test item that cannot be implemented as worded. Secondary hazard the missing field exposes: §2.5 removes `fontSize` from `EngraveFitted`, so the only size left to re-derive from is `f.SizeMM`, which §2.3 declares invalid when `Mixed`; neither §2.3's guards nor §2.7's validation list states `QR != nil => !Mixed`, so a hand-built `Fitted{Mixed:true, SizeMM:0, QR:code}` makes `params.F(0)==0` and `qrLines = (qrsz+2*qrBorder+fontSize-1)/fontSize` an integer divide by zero inside `ftBuildPlate`.

*Fix:* Add the resolved placement to `Fitted` (populated by `fitBlocksAt`, nil from `FitSized`), have `EngraveFitted` read the stored field rather than re-derive it, and list `QR != nil => !Mixed` beside the Faces / Sizes / size-string guards in §2.3. Then §7.7(c) can assert the stored value is what is drawn.

### 5. Important — SPEC_sizeproof.md §3 (GUI table) vs gui/preview.go:149-178

**§3 lists `proofPreview` and `previewBuilders` but not `fittedPreviewAt`, which is the only fit call on the preview path and calls `backup.FitBlocks` / `FitBlocksAt` directly — both of which ignore `Block.SizeMM` by §2.2/§2.7.**

*Failure:* `fittedPreviewAt(params, plan, text, title, footer, qr, size)` (preview.go:149) does `fit := backup.FitBlocks; if size != 0 { fit = FitBlocksAt(...) }`. With only the changes §3 lists — routing `proofPreview` through `ftProofOutcomeFor`, plus two `previewBuilders` entries — `plateview -plate sizeproof-front` (no `-size`) fits the 4-block composition UNIFORMLY: `FitBlocks` succeeds at some single rung, `describe` prints one size, and the pre-engraving preview of a permanent-steel plate shows a plate the device will not cut. §3's `previewBuilders` row only covers the `-size != 0` case ("`-size` must not re-fit a ladder plate at one rung"); the default `size == 0` path is uncovered. §7.18's stated assertions are about the resolver and the footer, so they pass while the sizes are wrong. `BuildPreview` is host-only (cmd/plateview/main.go:63), so no wrong steel results directly — but the tool exists to verify the plate before it is cut, and §3 asserts its enumeration is complete.

*Fix:* Add a `fittedPreviewAt (gui/preview.go:149)` row: it must route through the same `ftFitAt` the device uses, so a composition carrying sizes reaches `FitSized` in the preview too. Extend §7.18 to compare the preview's per-row sizes against the device's `Fitted.Sizes`, not just the footer.

### 6. Minor — SPEC_sizeproof.md §2.6

**"out by about 0.73 mm in the wrong direction" does not reproduce — the inset delta is 0.119 mm — and the understatement hides the larger half of the C4 hazard, which is band mis-classification rather than inset error.**

*Failure:* Re-derived by hand at sh2.Params() (MM=6400, sh advance 4000, Metrics.Height 6700, innerMargin-outerMargin = 44800): sh@5.0 -> charWidth = 4000*32000/6700 = 19104, holeChars = ceil(44800/19104) = 3, inset = 57312 = 8.955 mm. sh@3.8 -> charWidth = 4000*24320/6700 = 14519, holeChars = ceil(44800/14519) = 4, inset = 58076 = 9.074 mm. Delta 0.119 mm, not 0.73 mm — and the 5.0mm inset is SMALLER, so "the wrong direction" is also unqualified. Separately, the cached 5.0mm layout evaluates `holeLine := baseY+i*fontSize < innerMargin || ...` at the 5.0mm pitch, so it decides WHICH rows are screw-hole rows wrongly: a row in the 3.8mm block can receive the full ~9.07 mm inset when it should get none, or none when it should get ~9.07 mm — two orders of magnitude larger than the inset delta the spec quotes. The preamble's "Every number in this document was re-measured for R2" is falsified by this one figure. No decision turns on it: dropping `faceLayouts` is right on the re-keying argument alone.

*Fix:* Replace 0.73 mm with the measured 0.119 mm inset delta and add the band-predicate mis-classification as the larger half of the C4 hazard — or drop the number and keep the qualitative statement.

### 7. Minor — SPEC_sizeproof.md §1.2 (last paragraph)

**"with one, `sh@5.0` starts below the top band" is contradicted by §1.1's own measured table: the titled block starts at 6.800 mm, inside the 10 mm band, and its first row IS narrowed.**

*Failure:* §1.1's titled-front budgets for sh@5.0 are `[20 26 26 26]`: the first row is 20 columns against the unobstructed 26, i.e. narrowed by 2*holeChars, and the block's y-range starts at 6.800 mm while the top band is y < 10 mm. Both reviewers reproduced that table exactly. What the title actually buys is that the SECOND row's top moves from 8.000 mm to 11.800 mm and clears the band, which is why the block takes 4 rows rather than 5. The tables are right; the sentence a reader uses to judge whether the front's 3.600 mm of spare is structural is not.

*Fix:* Restate: with a title only the FIRST row of `sh@5.0` falls in the top band; untitled, the first TWO do, which costs the fifth row and 2.400 mm of the spare.

### 8. Minor — SPEC_sizeproof.md §2.3 vs backup/freetext.go:97-113

**§2.3 mandates panics on `len(Sizes) != len(Lines)` and on the size/string invariant, but `EngraveFreeText` — the constructor §2.3 itself cites as the load-bearing goldens' path — builds a `Fitted` with none of the three new fields set.**

*Failure:* `EngraveFreeText` returns `EngraveFitted(params, Fitted{SizeMM, Lines, Faces, QR, Title, Footer, TitleFace, FooterFace})` (freetext.go:105-113). With §2.3's guards implemented literally, `len(f.Sizes) == 0 != len(f.Lines)` panics on the very first golden, and every call passing a non-empty title panics on `TitleSizeMM == 0`. §2.3 cites `EngraveFreeText` as evidence the `Mixed` zero value is safe without saying the constructor must fill the three new fields. Caught immediately by §7.1 rather than shipped, hence Minor.

*Fix:* Add an `EngraveFreeText (backup/freetext.go:97)` row to §3's table, or state in §2.3 that the legacy constructor populates Sizes / TitleSizeMM / FooterSizeMM from its single `fontMM`.

### 9. Minor — SPEC_sizeproof.md §2.4 (pseudocode block)

**The no-footer branch of the `limit` formula appears only as a trailing comment, not as a conditional, so the pseudocode read in isolation recomputes round-1's I2 divide-by-zero.**

*Failure:* The block computes `footerY = margin + (LinesPerPlate(params, footerSizeMM)-1)*F(footerSizeMM)` unconditionally and then writes `limit = footerY   // no footer -> plateHeight - margin`. Read as literal Go on the no-footer path §5 mandates for BOTH ladder plates, `LinesPerPlate(params, 0)` is `height / params.F(0)` = height / 0 (backup.go:81-84). The guard exists only in §2.3's prose ("§2.4 and §2.5 both branch on the STRING being empty"), one section away. An implementer working from the code block reproduces I2.

*Fix:* Show the branch in §2.4's pseudocode: `if footer == "" { limit = plateHeight - margin } else { limit = margin + (LinesPerPlate(params, footerSizeMM)-1)*F(footerSizeMM) }`.

### 10. Minor — SPEC_sizeproof.md §7 items 2 and 14

**Two test-plan items name verification mechanisms that are not directly assertable as worded.**

*Failure:* Item 2 ("decode the engraving and assert row i's glyph height matches `Sizes[i]`") states no mechanism; the repo's bspline-bounds geometry helpers in backup/*_test.go make it plausible but nothing pins how, so it can be satisfied by a weaker proxy. Item 14's second half ("`LinesPerPlate` is never called with 0 on the no-footer path") has no built-in Go expression — asserting that an unexported function was not invoked with a given argument needs instrumentation the spec does not call for; the checkable property is "the no-footer path does not panic and `limit == plateHeight - margin`". Given §7.19 mutation-tests everything, an unfalsifiable item is a false-pass slot.

*Fix:* Reword item 2 to name the geometry-bounds or golden-decode helper it uses, and item 14's second half to "`FitSized`/`EngraveFitted` on the no-footer path does not panic and `limit` equals `plateHeight - margin`".

### 11. Nit — SPEC_sizeproof.md §2.1 (`qrPlacement`)

**`qrPlacement` carries Top/Bottom/X/Y/Size but not the QR border, which `charPerQRLine` needs.**

*Failure:* `textLayout` computes `l.charPerQRLine = (width - 2*qrBorder - qrsz) / charWidth` (wrap.go:181). With `(qrc, qrScale)` replaced by `*qrPlacement`, `qrBorder` is no longer derivable from the struct — inverting `X = plateW - Size - margin - qrBorder` needs it. It works only because `qrBorder` is the constant `params.I(2)`, which the spec never states.

*Fix:* Carry the horizontal keep-out (`2*qrBorder + Size`) on `qrPlacement`, or state that `qrBorder = params.I(2)` remains a shared constant both sides read.

### 12. Nit — SPEC_sizeproof.md §3 (`ftFitAt` row) vs gui/freetext_flow.go:204-209

**§3 says `ftFitAt` "routes to `FitSized` when every block carries a size" without fixing the order against the existing `if size != 0` branch.**

*Failure:* `ftFitAt` currently tests `size != 0` first and calls `FitBlocksAt`, which ignores `Block.SizeMM` (§2.2/§2.7). `ftFitAt` is shared with the `BOTHPROOF!<rung>` path, which does set a non-zero rung. If the FitSized test is appended second, a caller that sets both a rung and per-block sizes engraves the ladder at one uniform rung — R0's C6 in a third hat. Unreachable today only because §3 has `ftProofOutcome` carry the plan rather than a rung.

*Fix:* State that the per-block-size test comes FIRST, and that a non-zero `size` together with sized blocks is an error rather than a silent uniform fit.

### 13. Nit — SPEC_sizeproof.md §3, citation `gui/preview.go:111-133`

**Off-by-one line range for `proofPreview`.**

*Failure:* `proofPreview` spans lines 111-132 (the closing `}` of the outer function is line 132); line 133 is blank before `freeTextPreview` at 134. Verified by inspection. Cosmetic only.

*Fix:* Cite `gui/preview.go:111-132`.

### Notes

MERGE SUMMARY. The two reviews had ZERO overlapping findings — the design-adversarial pass worked the spec-vs-code surface (five gaps between what the spec says the code does and what it does), the mechanical pass worked fold coverage, citations and number reproduction (and found the spec's numbers overwhelmingly sound). All 5 opus Importants survive adjudication; I re-verified each one independently against the source rather than taking the report on trust (details below). Nothing was killed as out of scope: no finding re-opens the composition, the title/footer/6.0mm decisions, or round 0's six Criticals. One severity change from the inputs: the mechanical reviewer's `gui/preview.go:111-133` item is demoted Minor -> Nit (a stale line number is a Nit). Two opus Minors were considered for Important and held at Minor because no decision turns on them (§2.6's 0.73mm, §1.2's band sentence). Opus finding 1 (edit path) is the closest call to Critical — a 6-run BACK edit cuts a 5-rung ladder onto permanent steel — held at Important only because finding 3's confirm-screen fix keeps the run list visible before approval; if the controller folds finding 1 WITHOUT finding 3, treat finding 1 as Critical, because the operator's sole pre-engraving warning is the line that would then read "0.0mm".

(a) NUMERIC REPRODUCTIONS. Succeeded, and independently corroborated by BOTH reviewers with separate probes: §1.1's front table (sh@5.0 4 rows [20 26 26 26] 6.800->26.800; const@5.0 5 rows [23 x5] ->51.800; sh@3.8 3 rows [34 34 34] ->63.200; const@3.8 4 rows [31 31 31 25] ->78.400, spare 3.600) and back table ([24 30 30 30], [26 x4], [38 38 38], [34 34 34], [44 44 44], [39 31 31], end 79.600, spare 2.400), each row count confirmed MINIMAL; §1.2's corrected I5 (titled 0 of 10 pairs exceed ceil(95/CharsPerLine), untitled exactly 1 — sh@5.0 [20 20 26 26 26]) and the front-only inversion (untitled front ends 79.600/spare 2.400, untitled back 76.600/spare 5.400); §2.1's band equivalence (0 disagreements at all six rungs) and its QR figures (89 modules, holeLines 3, qrLines 20, band [12.000,72.000) mm, 12 vs 36 columns); §2.4's "24 of 24 identical" and the six-row overlap table (6.0/1.000, 5.0/4.000, 4.4/4.200, 3.8/3.000, 3.4/0.800, 3.0/1.000); §2.5's per-row at(0) equivalence; §5's inset spans (13<=26 at sh 3.8mm, 16<=36 at sh 3.0mm) and footer shortfalls (3.200mm front, 1.600mm back); §1's 380+570=950.
FAILED TO REPRODUCE — exactly one number in the document: §2.6's "about 0.73 mm". I re-derived it by hand from backup/backup.go:90-96 and backup/wrap.go:167-176 rather than relying on either report: sh@5.0 charWidth 19104, holeChars 3, inset 8.955 mm; sh@3.8 charWidth 14519, holeChars 4, inset 9.074 mm; delta 0.119 mm, and the 5.0mm inset is the smaller one. That falsifies the preamble's "Every number in this document was re-measured for R2", which is why it is recorded rather than waived.
NOT INDEPENDENTLY MEASURED BY ME: §2.1's 89-module/700-character QR figures and §6's admission-reachability estimate. The mechanical reviewer reproduced the former and flags a reproducibility fragility worth carrying into §7.7: QR mode selection depends on the CHARACTER SET, not just length — an all-uppercase probe text gives 77 modules, qrLines 17, band [12,63) — so whoever writes that test must pin a byte-mode text that actually hits 89 modules, not merely "700 characters".
CITATIONS: every file:line in the spec was checked against 3c3a2ad by the mechanical reviewer and spot-checked by me (freetext_flow.go:56/107/146/204/218/485, fit.go:150/280/297/302/316/344, freetext.go:34/71/97/105, backup.go:81/359/385/390, preview.go:111/149, plateview:98); all correct except the preview.go:111-133 off-by-one.
Finding 2's magnitude I derived analytically rather than by probe: today's `AdmissibleBlocks` start (row 1, baseY=margin) puts exactly 2 rows in the top band at 3.0mm; the correct §2.4 translation margin+F(3.0)=38400 reproduces that exactly; the carried-over literal `1` puts 4 rows in the band, each losing 2*holeChars = 8 of 44 columns.

(b) FOR THE CONTROLLER TO DECIDE, NOT MECHANICALLY FIX. (i) Finding 1's policy: does a shape-mismatched edit revert the WHOLE plate to uniform auto-fit (clear SizeMM on every block when len(out) != len(p.Runs)), or is a partial ladder legitimate provided the confirm screen names the surviving rungs? The spec currently implies the first and the code delivers the second; either is defensible, but the choice must be written down and tested. (ii) Finding 4's shape: add a placement field to `Fitted`, or weaken §7.7(c) — the re-derivation is deterministic so no plate is wrong today, but §2.1's "cannot drift" guarantee is unenforced either way unless the field is added. (iii) Finding 5 is the weakest of the five Importants (host-only tooling, one table row); a controller wanting a single-round fold can take it as a table addition without further debate. (iv) Related repo-level item, not a spec finding: gui/freetext_flow.go:107's doc comment on `ftPlan.Blocks` is accurate only for 2-run plans and is what the spec extrapolated from; it wants correcting in the fork whether or not this spec lands.

(c) REPO STATE. /scratch/code/shibboleth/seedhammer is CLEAN. `git status --short` returned nothing at both the start and the end of my pass, HEAD is 3c3a2ad, and I created no files — all my verification was reading source and hand-deriving arithmetic. For the record: the design-adversarial reviewer reported the repo was NOT clean when it started — four untracked probe files (zzprobe_test.go .. zzprobe4_test.go) left by an earlier review session — and deleted them along with its own two probes; that cleanup is confirmed effective. Worth a standing habit: reviewers should `git status --short` on entry, not only on exit.

---

## Lane 1 — design-level adversarial (opus)

VERDICT: RED — 10 findings

### 1. Important — SPEC_sizeproof.md §3 ("The edit path — stated precisely") vs gui/freetext_flow.go:107-135

**§3's edit-path decision is built on a false description of `ftPlan.Blocks`: it does not collapse to one block on a shorter text, and when it does collapse the block carries a size, so neither stated outcome is what the code produces.**

*Failure:* Measured by running the real `ftPlan.Blocks` against a 4-run ladder plan (runs [1,1,1,·]): 4 parts -> 4 blocks; 3 parts -> 3 blocks [sh,const,sh]; 2 parts -> 2 blocks; only exactly 1 part -> 1 block. (a) THE MIDDLE CASE: operator loads SIZEPROOF!FRONT, deletes one newline, presses OK — `Blocks` returns 3 blocks, each stamped with its own run's size per §3's own table row, so every block carries a size, `ftFitAt` routes to `FitSized`, and the plate is cut as a THREE-rung ladder with const@3.8 silently absent. §3 says "Two different things follow" and this is a third; §7.13 tests only the two. (b) THE COLLAPSE CASE: §3 says "The collapsed block carries no size, so `ftFitAt` routes to `FitBlocks` and the plate is an ordinary free-text plate", but the same table says `ftPlan.Blocks` "stamps each block with its run's size", so the single surviving block carries run 0's size (5.0mm front, 4.4mm back) — `ftFitAt` routes to `FitSized`, which either refuses the 380-character text at 5.0mm or cuts it at 5.0mm. The operator never gets the promised auto-fit plate. Two statements in one section contradict each other, which is round 1's I1 class.

*Fix:* State the real rule (`Blocks` yields min(parts, runs) blocks, last run absorbing the remainder) and decide the block-count ≠ run-count case explicitly — e.g. `ftPlan.Blocks` clears SizeMM on every block whenever `len(out) != len(p.Runs)`, so anything but an exact-shape edit reverts to uniform auto-fit. Add the partial-edit case (3 of 4 blocks) to §7.13.

### 2. Important — SPEC_sizeproof.md §2.4 / §6 vs backup/fit.go:280 and fit.go:302

**§2.4 redefines `wrapBlocks`' `start` from a row index to a device-unit y but gives the translation only for the fitter; `AdmissibleBlocks` passes literal `1` and `rowFaces` literal `0`, and both still compile unchanged under the new meaning.**

*Failure:* `wrapBlocks(params, blocks, size, qrc, 1, math.MaxInt)` (fit.go:280) and `wrapBlocks(..., 0, math.MaxInt)` (fit.go:302) pass int ROW INDICES. After the signature change the same literals mean y = 1 and y = 0 DEVICE units — 0.00016mm and 0mm from the plate top, instead of `margin + F(3.0)` and `margin`. Nothing forces a compile error (int -> int), and §2.4's only formula (`start = margin + F(titleSizeMM)`) does not apply to `AdmissibleBlocks`, which deliberately does not read the title. Measured at 3.0mm (sh, no QR), `start = margin + F(3.0)` vs `start = 1`: for 8 text lengths in 200..1200 the admission VERDICT flips — a 933-character text is admitted today (24 lines used, 24 available) and would be refused with "the text needs 25 lines and a plate holds 24"; `linesUsed` in the live readout differs for ~12% of lengths (168 of 1400 for a single-token text). `!f.ok` gates OK on both the text and the confirm step, so this is a wrong refusal on an ordinary operator plate.

*Fix:* In §2.4 state both translations explicitly: `AdmissibleBlocks` passes `params.I(outerMargin) + params.F(size)` (its unconditional title-row reservation), `rowFaces` passes `params.I(outerMargin)`. Add a test pinning `AdmissibleBlocks`' linesUsed at 3.0mm for a text at exactly capacity.

### 3. Important — SPEC_sizeproof.md §2.1 + §2.3 + §7.7(c) vs backup/freetext.go:76-87

**The `qrPlacement` fix has no channel from the fit to `EngraveFitted`, so §7.7(c)'s "asserted by construction" is false, and the only size `EngraveFitted` could re-derive the placement from is the one §2.3 declares invalid.**

*Failure:* §2.1 requires the placement to be "computed ONCE per plate ... read by BOTH the layout that narrows the lines and the engraver that draws the code, so the two cannot drift", and §7.7(c) asserts "freetext.go's and backup.go's QR offsets both read `qrPlacement.Y` — asserted by construction, since neither may compute a y of its own". `EngraveText` can hold the placement in a local (it builds its own layout), but `EngraveFitted` receives only `Fitted`, and §2.3's `Fitted` gains no placement field. So freetext.go MUST re-derive `qrPlaceAt(...)` — exactly the second derivation §2.1 forbids — and §7.7(c) can only be a vacuous assertion. Worse, §2.5 removes `fontSize` from `EngraveFitted`, so the only size left for that re-derivation is `f.SizeMM`, which §2.3 defines as "valid only when !Mixed"; nothing in §2.3's guards or §2.7's validation states or enforces `QR != nil => !Mixed`. A `Fitted{Mixed: true, SizeMM: 0, QR: code}` gives `params.F(0) == 0`, so `qrLines = (qrsz + 2*qrBorder + fontSize - 1) / fontSize` is an integer divide by zero and `holeLines = ceil(x/0)` overflows — R0's C5 panic arriving through a second door, in `ftBuildPlate`, with a plate clamped in the machine.

*Fix:* Add the resolved placement to `Fitted` (populated by `fitBlocksAt`, nil from `FitSized`), have `EngraveFitted` read it rather than re-derive it, and list `QR != nil => !Mixed` beside the Faces / Sizes / size-string guards in §2.3. Then §7.7(c) can assert the stored field is what is drawn.

### 4. Important — SPEC_sizeproof.md §3 (GUI table) vs gui/freetext_flow.go:485-489

**§3's enumeration of every size-reading site misses `ftConfirmSummary`, which formats `f.plate.SizeMM` with its own `%.1fmm` and is the line on the screen the operator approves.**

*Failure:* `ftConfirmSummary` builds `fmt.Sprintf("%.1fmm  %d lines  QR: %s  font: %s", f.plate.SizeMM, ...)` independently of `ftSizeLabel` (the only readout §3 lists, at freetext_flow.go:218). On a ladder plate `SizeMM` is invalid — §2.3: "valid only when !Mixed" — so the confirm screen reads "0.0mm  16 lines  QR: no  font: sh 4 + constant 5 + sh 3 + constant 4". That is precisely the "reader that prints `0.0mm` is a defect, not a fallback" §3 bolds two paragraphs later, on the one screen §3 names as what the operator approves. §7.17 only asserts that line FITS the panel by measuring rectangles — a "0.0mm" line fits — so nothing in the test plan catches it. Same incomplete-enumeration class as R0's C6 and round 1's I4.

*Fix:* Add a `ftConfirmSummary (gui/freetext_flow.go:485)` row to §3: it must print the ladder's rungs (or a range) rather than `SizeMM`. Extend §7.17 to assert the rendered summary names the front's rungs and never contains "0.0mm".

### 5. Important — SPEC_sizeproof.md §3 (GUI table) vs gui/preview.go:149-178

**§3 lists `proofPreview` and `previewBuilders` but not `fittedPreviewAt`, which is the only fit call on the preview path and calls `backup.FitBlocks`/`FitBlocksAt` directly rather than `ftFitAt`.**

*Failure:* `fittedPreviewAt(params, plan, text, title, footer, qr, size)` does `fit := backup.FitBlocks; if size != 0 { fit = FitBlocksAt(..., size) }`. Both ignore `Block.SizeMM` by §2.2/§2.7, so with only the changes §3 lists (routing `proofPreview` through `ftProofOutcomeFor`, and the two `previewBuilders` entries) `plateview -plate sizeproof-front` fits the 4-block composition UNIFORMLY: `FitBlocks` succeeds at some single rung, `describe` prints one size, and the preview of the ladder shows a plate the device will not cut. That is what §7.18 asserts against, and the site that must change is absent from a table §3 introduces with "Every one of these must change".

*Fix:* Add a `fittedPreviewAt (gui/preview.go:149)` row: it must route through the same `ftFitAt` the device uses, so a composition carrying sizes reaches `FitSized` in the preview as well.

### 6. Minor — SPEC_sizeproof.md §2.6

**"putting the left inset of every screw-hole row in that block out by about 0.73 mm in the wrong direction" does not reproduce; measured it is 0.119 mm, and the real C4 hazard is a different and larger one.**

*Failure:* Measured at production params: sh@5.0 has holeChars 3, charWidth 19104 -> inset 8.955mm; sh@3.8 has holeChars 4, charWidth 14519 -> inset 9.074mm. Delta 0.119mm, not 0.73mm. The understatement also hides the bigger half: the cached 5.0mm layout evaluates the band predicate at the 5.0mm pitch, so it decides WHICH rows are screw-hole rows wrongly — a row can receive the full 9.07mm inset when it should get none, or none when it should get 9.07mm. The spec's preamble states "Every number in this document was re-measured for R2", which this number falsifies (round 1's I5 class, though no decision turns on it — dropping faceLayouts is right either way).

*Fix:* Replace the figure with the measured 0.119mm inset delta and add the band-predicate mis-classification as the larger half of the C4 hazard, or drop the number and keep the qualitative statement.

### 7. Minor — SPEC_sizeproof.md §1.2

**"with one, `sh@5.0` starts below the top band" is false — the titled block starts at 6.800 mm, inside the 10 mm band, and its first row IS narrowed.**

*Failure:* §1.1's own measured budgets for the titled front are `[20 26 26 26]`: the first row is 20 columns against the unobstructed 26, i.e. narrowed by 2*holeChars. The block starts at y = 6.800 mm and the top band is y < 10 mm, so it starts INSIDE the band. What the title actually buys is that the SECOND row's top moves from 8.000 mm to 11.800 mm and clears the band, which is why the block takes 4 rows rather than 5. The tables are right; the sentence a reader uses to judge whether the front's 3.600 mm of spare is structural is not.

*Fix:* Restate: with a title only the FIRST row of `sh@5.0` falls in the top band; untitled, the first TWO do, which costs the fifth row and 2.400 mm of the spare.

### 8. Minor — SPEC_sizeproof.md §2.3 vs backup/freetext.go:97-113

**§2.3 mandates panics on `len(Sizes) != len(Lines)` and on the size/string invariant, but `EngraveFreeText` — the constructor §2.3 itself names as the load-bearing goldens' path — builds a `Fitted` with none of the three new fields set.**

*Failure:* `EngraveFreeText` returns `EngraveFitted(params, Fitted{SizeMM, Lines, Faces, QR, Title, Footer, TitleFace, FooterFace})`. With §2.3's guards implemented literally, `len(f.Sizes) == 0 != len(f.Lines)` panics on the very first golden, and every call passing a non-empty title panics on `TitleSizeMM == 0`. §2.3 cites `EngraveFreeText` as evidence that the `Mixed` zero value is safe without saying the constructor must be updated to fill the three new fields. Caught immediately by §7.1 rather than shipped, hence Minor.

*Fix:* Add `EngraveFreeText (backup/freetext.go:97)` to §3's table, or state in §2.3 that the legacy constructor populates Sizes / TitleSizeMM / FooterSizeMM from its single `fontMM`.

### 9. Nit — SPEC_sizeproof.md §2.1 (`qrPlacement`)

**`qrPlacement` carries Top/Bottom/X/Y/Size but not the QR border, while `charPerQRLine` needs `2*qrBorder + qrsz`.**

*Failure:* `textLayout` today computes `l.charPerQRLine = (width - 2*qrBorder - qrsz) / charWidth` (wrap.go:182). With `(qrc, qrScale)` replaced by `*qrPlacement`, `qrBorder` is no longer derivable from the struct — `X = plateW - Size - margin - qrBorder` needs qrBorder to invert. It works only because `qrBorder` is the constant `params.I(2)`, which the spec never states.

*Fix:* Carry the horizontal keep-out (`2*qrBorder + Size`) on `qrPlacement`, or state that `qrBorder = params.I(2)` remains a shared constant both sides read.

### 10. Nit — SPEC_sizeproof.md §3 (`ftFitAt` row) vs gui/freetext_flow.go:204-209

**§3 says `ftFitAt` "routes to `FitSized` when every block carries a size" without fixing the order against the existing `if size != 0` branch.**

*Failure:* `ftFitAt` currently tests `size != 0` first and calls `FitBlocksAt`, which ignores `Block.SizeMM` (§2.2/§2.7). `ftFitAt` is shared with the `BOTHPROOF!<rung>` path, which does set a non-zero rung. If the FitSized test is appended second, a caller that sets both a rung and per-block sizes engraves the ladder at one rung — R0's C6 in a third hat. Not reachable today only because `ftProofLoader` always writes `*size = out.SizeMM`.

*Fix:* State that the per-block-size test comes FIRST, and that a non-zero `size` together with sized blocks is an error rather than a silent uniform fit.

### Notes

VERDICT RED: 0 Critical, 5 Important, 3 Minor, 2 Nits. Scope held to \"does the R2 fold work, and did it introduce a new defect\"; R0 Criticals not revisited, composition not re-opened, no scope changes proposed.

WHAT I VERIFIED INDEPENDENTLY (probe tests in package `backup` at `sh2.Params()`, all deleted afterwards):

1. §1.1's tables REPRODUCE EXACTLY. Titled FRONT: sh@5.0 4 rows [20 26 26 26] 6.800->26.800; const@5.0 5 rows [23 x5] ->51.800; sh@3.8 3 rows [34 34 34] ->63.200; const@3.8 4 rows [31 31 31 25] ->78.400, spare 3.600. Titled BACK: [24 30 30 30], [26 x4], [38 38 38], [34 34 34], [44 44 44], [39 31 31], end 79.600, spare 2.400. Each block was also checked NOT to fit in one fewer row, so the counts are minimal rather than padded.
2. §1.2's corrected I5 claim HOLDS: titled, 0 of 10 (face,rung) pairs exceed ceil(95/CharsPerLine); untitled over the real block order exactly 1 does (front sh@5.0, [20 20 26 26 26]). Untitled front ends 79.600 (spare 2.400), untitled back 76.600 (spare 5.400) — the front-only inversion is real.
3. §2.4's "24 of 24 identical" HOLDS: recomputed start/limit/admitted-row-count against `bodyRows` for all 6 rungs x title x footer, 0 mismatches. "bottom <= limit" is right at both ends (with a footer the last admitted row's bottom lands exactly on footerY; without one, margin + rows*fontSize <= plateHeight - margin, and row `rows` is refused). It also survives title-size != body-size != footer-size, since `start` reads TitleSizeMM and `limit` reads FooterSizeMM independently of the body rows.
4. §2.4's overlap table REPRODUCES EXACTLY (6.0/1.000, 5.0/4.000, 4.4/4.200, 3.8/3.000, 3.4/0.800, 3.0/1.000).
5. §2.1's band equivalence HOLDS: `Top <= y < Bottom` and `holeLines <= i < holeLines+qrLines` agree on every row at every rung, 0 disagreements — algebraically identical since Top = baseY + holeLines*fontSize and y = baseY + i*fontSize. That identity holds for the DESCRIPTOR anchor too (anchorY = paragraph offy), so EngraveText is preserved and its QR-ONLY centring override at backup.go:390-393 is untouched by the change.
6. §2.5's per-row `at(0)` rewrite IS equivalent on a uniform plate: with baseY = y_i the band predicate reduces to the same comparison, the QR predicate is plate-absolute either way, and offx = holeChars*charWidth is baseY-independent. Same for MaxCharsAtBlocks' at(row) -> per-row layout at at(0).
7. §5's title measurements HOLD: inset span = charPerLine - 2*holeChars = 34-8 = 26 at sh 3.8mm (13-char title) and 44-8 = 36 at sh 3.0mm (16-char title).
8. Grepped every consumer of holeLines/qrLines: only freetext.go:85, backup.go:385 and wrap.go itself in non-test code — the fold forgot none. Test consumers (freetext_test.go:195/289, engravetext_test.go:165-190, which builds a lineLayout literal) need rewriting; that is churn, not a finding.
9. §3's citations all check out: ftFaceRun:56, ftPlan.Blocks:107, ftFaceSummary:146, ftFitAt:204, ftSizeLabel:218, ftProof:367, ftProofOutcome:511, ftRungLabel:531, ftProofReplaces:538, fit.go MaxCharsAtBlocks:344 / rowFaces:297, preview.go:111-133, plateview sizeLabel:98. §2.7's TextQR:"" -> NeedsWholePlate -> ftProofLoader clearing *useQR before ftProofOutcomeFor is implementable exactly as described.
10. §6's admission claim is reachable: at 3.0mm the front's 4 sweeps take ~12 rows and the back's 6 take ~18-19 against linesAvail 24, so both triggers pass AdmissibleBlocks.

REPO HYGIENE: /scratch/code/shibboleth/seedhammer/backup/ contained four UNTRACKED probe files left by an earlier review session (zzprobe_test.go, zzprobe2_test.go, zzprobe3_test.go, zzprobe4_test.go) — the repo was NOT clean when I started. I ran them (they independently corroborate §1.1, §1.2 and the overlap table), then deleted them along with my own two probes. `git status --short` now returns nothing, HEAD is 3c3a2ad, and no tracked file was modified.

SEVERITY NOTE FOR THE CONTROLLER: Important 1 (the edit path) is the one I came closest to calling Critical — it engraves a ladder with a rung missing onto permanent steel. I held it at Important because the confirm screen still lists the runs before the operator approves — but only IF Important 4 is also folded. Today's confirm summary prints its own "%.1fmm" from SizeMM and would read "0.0mm", and the per-run sizes come from `ftFaceSummary`, which §3 does require to read `Fitted.Sizes`. Fold Important 4 and Important 1 stays visible-before-approval; fold neither, or only 1, and the operator's sole warning disappears.

---

## Lane 2 — mechanical fold-vs-findings + number reproduction (sonnet)

VERDICT: GREEN — 3 findings

### 1. Minor — SPEC_sizeproof.md §2.4 (pseudocode block)

**The no-footer branch of the `limit` formula is only stated as an inline comment, not as an explicit conditional, even though §2.3 promises §2.4 'branches' on the string being empty.**

*Failure:* The three-line pseudocode block computes `footerY = margin + (LinesPerPlate(params, footerSizeMM)-1)*F(footerSizeMM)` unconditionally and then comments '// no footer -> plateHeight - margin' beside `limit = footerY`. Read as literal Go, this is round-1 I2's exact divide-by-zero (`LinesPerPlate(params, 0)`) reappearing, since nothing in the code block itself shows the `if footer == ""` guard — it's asserted only in §2.3's prose cross-reference. An implementer who works from the pseudocode block in isolation (skimming past the §2.3 paragraph) could reproduce I2.

*Fix:* Show the branch explicitly in §2.4's pseudocode, e.g. `if footer == "" { limit = plateHeight - margin } else { limit = margin + (LinesPerPlate(params, footerSizeMM)-1)*F(footerSizeMM) }`, rather than relying on a same-line comment plus a forward reference to §2.3.

### 2. Minor — SPEC_sizeproof.md §3 table, citation `gui/preview.go:111-133`

**Off-by-one line-range citation for `proofPreview`.**

*Failure:* `proofPreview` (the outer function plus its returned closure) spans lines 111-132 in the current source; line 133 is a blank line before `freeTextPreview` begins at 134. Not load-bearing, but citations decay and an implementer following the range literally lands one line past the function.

*Fix:* Cite `gui/preview.go:111-132`.

### 3. Minor — SPEC_sizeproof.md §7 items 2 and 14

**Two test-plan items describe verification mechanisms that are not directly assertable as stated.**

*Failure:* Item 2 says to 'decode the engraving and assert row i's glyph height matches Sizes[i]' with no stated mechanism; the repo has bspline.Bounds-based geometry helpers used elsewhere in backup/*_test.go, so it is plausible, but nothing pins how. Item 14 says 'LinesPerPlate is never called with 0 on the no-footer path' — Go has no built-in way to assert a specific unexported function was or wasn't invoked with a given argument without instrumentation; the practical test is 'FitSized/EngraveFitted with no footer does not panic and produces the right limit,' which is a proxy for, not a literal check of, the stated property.

*Fix:* Reword item 2 to name the geometry-bounds mechanism (or the golden-decode helper) it will use; reword item 14's second half to 'FitSized/EngraveFitted on the no-footer path does not panic and limit equals plateHeight-margin,' which is what is actually checkable.

### Notes

SCOPE A (fold coverage against round-1's report): every item — the 1 Critical, all 6 Importants (I1-I6), both Minors (Uniform->Mixed; 83.4mm/1.4mm), both Nits (475->950; footer/sizeLabel), the two prose nits (title face/inset-span; PASSPROOF! in LEXICON), the two citation nits (limit/maxY; ftFaceSummary line), and both round-0 residues (I4-residue GUI-surface gaps; I8(e) missing test) — is addressed in R2 with specific, quoted spec text. I independently traced the fix mechanism (not just the presence of a sentence) for the Critical (§2.1's qrPlaceAt/anchorY design correctly separates the plate-absolute free-text anchor from the paragraph-relative descriptor anchor, and lineLayout's isQRLine predicate becomes anchor-agnostic) and for I1/I2/I3/I6 by reading the actual current source (wrap.go, fit.go, freetext.go, backup.go) alongside the spec's claims. No round-1 item is unaddressed.

SCOPE B (citations): checked every file:line and symbol cited in SPEC_sizeproof.md against seedhammer @ 3c3a2ad (clean, matches repo HEAD). All are correct except one off-by-one (gui/preview.go:111-133, function actually ends at 132) — reported as a Minor above. freetext_flow.go:146 for ftFaceSummary (previously a round-1 nit citing the wrong line/file) is now correct. All other citations (wrap.go:133-158, :140-141, :135; freetext.go:42-46, :81-85, :34-39, :71; backup.go:359,385, :383-385, :390-393; fit.go:150, :276-280, :297, :302, :316-331, :344; freetext_flow.go:56,107,204,146,218; freetext_proof.go:367,511,531,538; cmd/plateview/main.go:98) verified byte-exact against the real source.

SCOPE C (numbers reproduced with a Go probe under backup/, package-internal so it could call textLayout/WrapText directly; probe files created and then deleted, git status --short confirmed clean before finishing):
- §1.1 FRONT and BACK tables (block, rows, per-row budgets, y-ranges): reproduced EXACTLY, including the [20 26 26 26]/[31 31 31 25] and [24 30 30 30]/[39 31 31] budget vectors and every y boundary (6.800->78.400 front, 6.000->79.600 back). Confirmed each given row count is the true MINIMUM (rows-1 does not fit).
- §1.2 'zero of ten (titled) / exactly one (untitled, sh@5.0, [20 20 26 26 26])': reproduced exactly using the REAL cumulative front/back layouts (not per-pair isolated tests, which give a different, irrelevant number — my first attempt at this made that isolation mistake and got 2/10 and 6/10 before correcting the method to match what '(face,rung) pair' means in context: its actual position within the real FRONT/BACK plate). Titled: 0/10 exceed naive. Untitled: 1/10 exceeds (sh@5.0, 5 rows vs naive 4). Untitled FRONT ends 79.600mm/spare 2.400mm; untitled BACK ends 76.600mm/spare 5.400mm — both exact matches, confirming the 'inversion is front-only' claim.
- §2.1 QR-window numbers: modules=89, holeLines=3, qrLines=20, band=[12.000,72.000)mm, row-index-3 budget=12, row-index-2 (same layout object) budget=36 — ALL reproduce exactly, but only once I used a realistic mixed-case 700-char descriptor-like string (byte-mode QR encoding); an all-uppercase-A probe text gives materially different numbers (77 modules, qrLines=17, band [12,63), 16/44) purely because QR mode selection depends on the character set, not the length. This is worth the controller knowing: the spec's '700-character descriptor text' is underspecified as to WHAT text (byte-mode vs alphanumeric-mode QR encoding changes module count substantially for the same length) — not a defect in the spec's stated numbers (which do reproduce with a plausible descriptor string), but a reproducibility fragility for whoever writes the actual §7.7 test: they need a text that hits 89 modules, not merely '700 characters.' I did not file this as a formal finding since the STATED numbers are internally consistent and reproducible, but flagging it here so it doesn't get re-litigated as a fold-vs-findings gap.
- §2.4 24/24 equivalence claim and the six-row overlap table (6.0->1.000, 5.0->4.000, 4.4->4.200, 3.8->3.000, 3.4->0.800, 3.0->1.000): both reproduce EXACTLY.
- §5: title face sh on both sides confirmed (blocks[0].Face by construction). Inset-span check 13<=26 at sh@3.8mm and 16<=36 at sh@3.0mm reproduces exactly (charPerLine - 2*holeChars = 34-8=26 and 44-8=36; title strings are literally 13 and 16 characters). Footer shortfalls 3.200mm (front, 78.400-75.200) and 1.600mm (back, 79.600-78.000) reproduce exactly, closing round-1's '1.6mm not 1.4mm' minor correctly.
- §1 character counts: 4*95=380, 6*95=570, total 950 — trivial arithmetic, correct.

SCOPE D (test plan, §7's 19 items): 17 of 19 are concretely falsifiable as written (each names a specific mechanism or existing precedent — e.g. item 7's three-part QR-window test explicitly targets the round-1 Critical with a two-block-plus-QR fixture that is 'unreachable by any single-block fixture,' item 17 reuses ftProofBody's proven measure-rectangles methodology). Items 2 and 14 have a wording gap between the claim and what is mechanically assertable in Go without instrumentation — reported as a Minor above, not gating.

Overall: R2 is a materially careful fold. Every number I re-derived from the real source matched the spec's stated value once I used the right input construction (my two false starts, the isolated-pair naive-count method and the plain-ASCII QR probe text, were my own methodology errors, not spec errors — corrected and reproduced exactly on the second attempt in both cases). Zero Critical/Important findings survive; three Minors recorded, none gating.
