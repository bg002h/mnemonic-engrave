# R0 review — IMPLEMENTATION_PLAN_sizeproof.md — round 2 (final) — **GREEN**

Single opus lane dispatched 2026-08-05 against the folded plan @ `e4c08a5`,
scoped to two questions: did the fold close round 1's six findings, and did it
introduce a new one. Declared the last scheduled round. Persisted VERBATIM.

VERDICT: **GREEN — 0 Critical / 0 Important.** The gate closes;
implementation may begin on the plan as written.

---

## Findings

*(none)*

## Notes

SCOPE HELD: only the two questions asked. Spec, coverage tables, phase-boundary compilability, P6-last, P2's 7.7(a) survival and the untested multi-size QR guard were taken as settled and not re-derived.

Q1 — ALL SIX ROUND-1 FINDINGS CLOSED. Checked against the fold diff cc78b75..e4c08a5 of /scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_sizeproof.md:
(1) I — both halves present. P3 now states "7.7(b) is a REGRESSION pin and cannot be RED before this phase's implementation - 0's TDD rule does not apply to it", followed by the pulled-forward mutation ("temporarily index block 2's rows block-relative against baseY = outerMargin, confirm 7.7(b) FAILS, then revert - from a COPY made first") and the escalation clause ("a 7.7(b) that stays green under that mutation is a blocking finding"). P2 now carries the four numbered shape constraints, including constraint 4's rectangle intersection [qrAt.X, X+Size) x [qrAt.Y, Y+Size) with the explicit "never a plate-wide max-x bound".
(2) m — "What P1's gate CANNOT see" paragraph added. Its three load-bearing claims verified in source: textLayout still computes holeLines/qrLines internally (backup/wrap.go:172,183) so nothing consumes qrAt in P1; EngraveFitted still derives the code y from lay.holeLines/lay.qrLines (backup/freetext.go:81-85); qrPlaceAt's signature carries no face, so it cannot delegate to textLayout.
(3) m — rowFaces (fit.go:302, the wrapBlocks call site) named; "four more sites" replaced by "more sites"; the fixture count is now six and names blocks_test.go:30 (rowBudget), :393 (insetOf) and fit_test.go:15 (layAt). Every added citation resolves: wrapBlocks is at fit.go:147 (the fold silently corrected the earlier :150), rowFaces at :297 with its wrapBlocks call at :302, faceLayouts.at at :321 with its textLayout call at :327, and all three test call sites are textLayout callers.
(4) m — the fill is now specified: Sizes = len(lines) copies of size, equal to SizeMM, Mixed false, TitleSizeMM/FooterSizeMM 0, with the reason (the insetOf reference is plate-absolute).
(5) n — P1 now names "both its enforcement points": the fitBlocksAt ERROR return plus the defensive re-assert as a panic in EngraveFitted. Matches SPEC 2.1.1 row 3.
(6) n — Status is "R2" with an explicit note that revisions are Rn and Pn is a phase.

Q2 — NO NEW DEFECT. Three attacks, all clean.

(a) MEASUREMENT CONFIRMED EXACTLY. Probe backup/zz_final_probe_test.go (written, run, DELETED; repo clean at 6d57681 on exit, git status --short empty). prodParams, 3.0 mm, sh.Font, an 89-module code (qr.Encode of a 645-byte text): LinesPerPlate = 26, holeLines = 3, qrLines = 20, charPerLine = 44, charPerQRLine = 12, holeChars = 4, charWidth = 11462, fontSize = baseY = 19200. Per-row at(i): rows 0-2 n=36 offx=45848, rows 3-22 n=12 offx=0, ROW 23 n=44 offx=0, rows 24-25 n=36 offx=45848. The plan's "band is plate rows 3-22 at 12 columns against 44 unobstructed, and row 23 sits below the band and legitimately inks all 44 columns" is correct verbatim. The rectangle-vs-max-x argument also checks out arithmetically: X = 544000-341760-19200-12800 = 170240, Y = 97920, Y+Size = 439680; row 23's ink spans x to 504328 (well past X) but y in [460800, 480000), i.e. BELOW the code box - so the rectangle assertion passes on a correct plate where a max-x bound fails, exactly as constraint 4 claims.

(b) CONSTRAINTS 2 AND 3 ARE COMPATIBLE, AND THE MUTATION DISCRIMINATES. Satisfiability: fitBlocksAt(P, blocks, "", "", qrc89, 3.0) with block 1 a spaceless run filling 1, 2 or 3 rows and block 2 filling the remainder returns 26 lines in all three cases, every line exactly equal to its row's budget (36/36/36, 12x20, 44, 36/36). wrapBlocks starts each block on the row the previous one ended on (no gap, fit.go:151-158), so "block 1 ends above the band's first row" puts block 2's start at plate row s in {1,2,3} - satisfiable, and not contradictory with block 2 filling every row it spans, because a spaceless run fills every row but its last by construction. Necessity: with block 1 at zero rows the shift is zero and the mutation is invisible, which is precisely what constraint 2 forbids. Discrimination: simulating the mutation (WrapText over widthFor(lay, 0) instead of widthFor(lay, s)) with s=2 yields 36-character lines engraved at plate rows 3 and 4 where the correct budget is 12; row 4's ink spans y [96000,115200) and x to 412632, intersecting the code rectangle on both axes, so BOTH halves of 7.7(b) fail. For every admissible s the budget half fails regardless (s=1: at(2)=36 vs at(3)=12; s=2: at(1)=36 vs at(3)=12; s=3: at(0)=36 vs at(3)=12). No fixture satisfying the four constraints survives the mutation. Not a Critical - the opposite: the constraint set is exactly strong enough.

(c) NO CROSS-PHASE CONTRADICTION. P1's added EngraveFitted defensive panic compiles in P1 because qrAt lands in P1 and both hand-built Fitted literals (blocks_test.go:298, :414) carry QR nil and qrAt nil, so nil-consistency holds. P2's new claim that rowFaces and MaxCharsAtBlocks become placement producers in P2 agrees with P3's caller-translation list (rowFaces -> params.I(outerMargin), which is the correct device-unit translation of its current start row 0) and with P5's "AdmissibleBlocks/rowFaces/MaxCharsAtBlocks in P2-P3". P1's "mk keeps its per-row-inset assertions" and P2's "blocks_test.go:393 changes type only" are consistent - both are nil-code layouts, so nil qrc becomes nil qrp. P3's TDD carve-out is an explicit, justified exception stated at the point of use rather than a silent conflict with 0.

NON-GATING OBSERVATIONS (recorded, not filed; none of these blocks implementation):
- An 89-module code cannot be produced BY a composition that fits at 3.0 mm: the plate holds 464 characters beside such a code, and an 89-module code needs roughly 645 bytes. The fixture will therefore either inject the code into the unexported fitBlocksAt (legal in-package, and the shape I verified) or pin whatever module count the exported path yields. The plan never mandates 89 modules - constraint 1 says pin by module count, and constraint 4's measurement is offered as the reason the max-x formulation is wrong, which holds for every realizable code size (rows below any band ink full width). No action needed.
- fillRows computes its budgets with qrc = nil, so used verbatim it sizes text for a no-QR plate. This does not weaken constraint 3, because spaceless text fills every row but its last whatever the budgets are; it only changes how many rows block 2 spans, which is visible immediately (the fit either lands at 3.0 mm or errors).
- Constraint 2's rationale sentence ("otherwise a block-relative index shifts block 2's window by zero rows") strictly justifies only the "at least one row" half; "end above the band's first row" is stronger than its stated reason. It is satisfiable and sufficient, so the over-specification costs nothing.
- freetext.go is cited as 80-85 in one place; the derivation is at 81-85 (line 80 is the last comment line). Immaterial.

READY TO EXECUTE. The gate closes: 0 Critical / 0 Important. Implementation may begin on the plan as written.
