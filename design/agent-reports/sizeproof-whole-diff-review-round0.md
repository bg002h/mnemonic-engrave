# Whole-diff adversarial review — SIZEPROOF — round 0

The mandatory post-implementation review, dispatched 2026-08-05 against the
whole diff `3c3a2ad..6b6b3cb` (24 files, +4742/-519, seven commits). Two lanes:
opus correctness-adversarial + sonnet spec-vs-code conformance, merged by an
independent opus synthesiser. Persisted VERBATIM before folding.

VERDICT: **RED** — 0C / 2I / 1 Minor

**Both lanes independently found the same primary defect** — the first real
CODE defect of this cycle, after six phases in which every finding was in the
record. Zero goldens moved anywhere in the branch; the defect surface is the
two `SIZEPROOF!` triggers and no seed, descriptor or passphrase can reach it.

---

## MERGED MERGE VERDICT (synthesiser)

VERDICT: RED — 3 findings

### 1. Important — `gui/freetext_flow.go:314 (ftEvaluate's AdmissibleBlocks call) vs gui/freetext_flow.go:373 (ftFitAt's FitSized branch)`

**Admission and the fit disagree on the ok verdict for a size-ladder composition: ftEvaluate always hands the raw useQR flag to AdmissibleBlocks, which lays the blocks out uniformly at 3.0mm with a QR band reserved, while ftFitAt routes the same blocks to FitSized, which structurally cannot carry a QR and ignores the flag entirely. A ladder that fits perfectly is refused with a QR remedy the plate cannot use.**

*Failure:* Reproduced myself against the worktree with a throwaway probe (gui/zz_mergegate_probe_test.go, written, run, deleted; repo clean). Unit level, over the real proof entries via ftSizeProofFor + plan.Blocks:
  SIZEPROOF!BACK  qr=false -> AdmissibleBlocks (used=18, avail=24, ok=true);  ftFitAt err=nil, 20 rows, Mixed, 4.4-3.0mm
  SIZEPROOF!BACK  qr=true  -> AdmissibleBlocks (used=30, avail=24, ok=FALSE); ftFitAt err=nil, 20 rows, Mixed, 4.4-3.0mm
  SIZEPROOF!FRONT qr=true  -> AdmissibleBlocks (used=20, avail=24, ok=true)  -- 4 lines of headroom, i.e. a live cliff, not a safe side.
Flow level, driving engraveTextFlow through the real harness (startFT -> ftPastQR(false) -> type SIZEPROOF!BACK -> OK -> proofYes -> ftBack -> ftChoose("qr",1) "Add QR" -> OK), the captured frame is verbatim: "TheTextfieldneeds30linesandaplateholds24,atthesmallestsize.RemovingtheQRfreesabout476charactersandtheplatestopsbeingmachine-readable KeeptheQR RemovetheQR TooLong". The plate carries no QR at any point, so removing it frees nothing; the operator is blocked at the Text step by a rationale about a code the composition cannot hold. The path is not hypothetical -- spec section 2 line 179 and section 7.17(a2) both name "go Back to the QR screen, re-enable the QR" as reachable on shipped firmware, and plan/text/useQR all survive Back by design. Nothing is engraved wrong (the confirm screen correctly reads QR off f.plate.QR, and taking "Remove the QR" then yields the correct ladder), so this is not Critical; it is a real defect with a false message on a shipped path, and the front sits 4 lines from the same cliff, so a glyph or pattern change moves it there too. Confined to the two ladder triggers: only the sizeproof plans set run SizeMM (gui/freetext_proof.go:477-488), so no seed, descriptor or passphrase plate can reach it.

*Fix:* Make admission agree with the router. Minimal: in ftEvaluate (gui/freetext_flow.go:314) compute admitQR := useQR && !ftSizedBlocks(blocks) and pass that, since a sized plate can never carry a code and the operator's stale flag must not narrow the band. Better, and it also closes the Minor below: count a sized composition at its own rungs so linesUsed/linesAvail describe the plate FitSized actually lays out. Either way ftRefuse (gui/freetext_flow.go:483) must not offer the QR remedy for a composition ftSizedBlocks accepts. Pin it with a flow test that re-enables the QR after loading each ladder and asserts the Text step still advances.

### 2. Important — `gui/freetext_sizeproof_test.go:619 (TestSizeProofDropsTheQRTheOperatorChose, defined at :597)`

**Spec 7.17(a2) -- "load a ladder, go Back to the QR screen, RE-ENABLE the QR, and the confirm screen must still read QR: no" -- has no test that re-enables the QR. The only test that walks that path selects "No QR" again, so the distinction the item exists to pin is never exercised.**

*Failure:* Verified by exhaustive grep across the gui test suite: there is no ftChoose(h, "qr", 1) or ftPastQR(h, true) anywhere after a ladder is loaded. TestSizeProofDropsTheQRTheOperatorChose is the sole test that goes Back to the QR screen with a ladder loaded; at line 619 it calls ftChoose(h, "qr", 0), leaving useQR false. The confirm screen therefore never renders in the state useQR==true with f.plate.QR==nil, which is precisely what distinguishes the shipped ftConfirmSummary (correct -- useQR := f.plate.QR != nil, gui/freetext_flow.go:720) from a regression reading the flow's flag. The test's own comment justifies observing the flag "rather than its consequences", which discharges item 7.16 but not 7.17(a2). Consequence beyond the missing pin: writing the test as the spec words it is what would have caught the finding above -- for SIZEPROOF!BACK the specified scenario cannot reach the confirm screen at all today, and for SIZEPROOF!FRONT it reaches it and passes.

*Fix:* After ftBack in TestSizeProofDropsTheQRTheOperatorChose, select index 1 ("Add QR") instead of 0, then continue through Title/Footer/Confirm and assert the confirm screen still reads "QR: no" and carries no privacy warning (ftWarnQR absent), for BOTH triggers. Run it against the fix for finding 1; against the current tree the BACK case fails at the Text step, which is the point.

### 3. Minor — `gui/freetext_flow.go:486 (ftRefuse's no-QR branch)`

**When an edited ladder genuinely overflows its own rungs, the refusal quotes the 3.0mm admission figures, which say there is room to spare.**

*Failure:* Same root cause, opposite direction: AdmissibleBlocks is only a lower bound for a plate fitted at FontSizes' smallest rung, which a ladder is not. Measured on the front in my own probe, admission reports used=12 of avail=24 while FitSized lays out 16 rows at 5.0/3.8mm; grow the first sweep without changing the part count (so plan.Blocks keeps the sizes) and FitSized returns ErrTooLarge while admission still says ok with 12 of 24 used. ftTextEntryFlow refuses on f.err -- correctly, nothing wrong is engraved -- but then prints numbers that contradict the refusal. Non-gating: the outcome is right and the advice is directionally right.

*Fix:* Falls out of counting a sized composition at its own rungs, per the second half of finding 1's fix.

### Notes

DEDUP: both lanes independently found the same primary defect (opus #1, sonnet #1) - merged as finding 1. Sonnet #2 (the 7.17(a2) test gap) and opus #2 (the Minor refusal-figure mismatch) are distinct and kept. I verified findings 1 and 2 myself from source and with my own probe rather than on either lane's word; the Minor I verified by mechanism plus my own measured front numbers, not by re-probing the edit case.

DROPPED: nothing. Neither lane raised a finding that re-derives a SETTLED fact, re-litigates the spec's design, re-runs the mutation pass, or re-measures the composition tables. Both lanes' notes contain a large volume of CLEARED material I did not re-verify line by line (unreachable EngraveFitted panics, ErrQRTooTall unreachability, PASSPROOF! rename hygiene, gofmt/vet cleanliness); I spot-checked enough to believe it and it produced no findings either way.

(a) SPEC SECTION 3 SITE COVERAGE. I walked all 23 rows of the section 3 table against the tree; every row is implemented at its named site. Spot-verified the non-obvious ones myself: cmd/plateview/main.go:124 prints "fixed layout" for the zero branch and handles Mixed; gui/preview.go:146 goes through ftProofOutcomeFor (no hardcoded ftProofFooter); gui/preview.go:179 fittedPreviewAt routes through ftFitAt; gui/freetext_flow.go:724 ftConfirmSummary prints ftPlateRungs and never SizeMM; ftProof carries a per-proof Footer, empty on both ladders. ONE row is implemented in the letter and defective in the interaction: "ftEvaluate / ftBuildPlate - unchanged in shape; both go through ftFitAt". Leaving ftEvaluate unchanged in shape is exactly what leaves AdmissibleBlocks reading the raw useQR flag under a router that ignores it. Section 6 pre-blesses the LINE-COUNT divergence ("the readout reports the 3.0mm anchor's line count - which differs from the rows actually cut. Accepted") but nowhere blesses the OK BOOLEAN diverging, and section 3.2's own reasoning about the re-enable path assumes the operator reaches the confirm screen. Finding 1 is therefore a code/spec divergence, not a re-litigation of an accepted trade-off.

(b) SPEC SECTION 7 ITEM COVERAGE. 19 of 20 items map to real, non-vacuous tests (I read the substantive ones in fitsized_test.go, qrplacement_test.go, runningy_test.go, freetext_sizeproof_test.go, freetext_sizeproof_table_test.go and agree with the sonnet lane's mapping). ONE item has no real test: 7.17(a2), finding 2 - established by exhaustive grep, not by sampling. Note the ordering: item 19 says "every one mutation-tested", and a mutation pass cannot manufacture a test that does not exist, which is exactly why 45 mutations across 41 tests did not surface this.

(c) NON-LADDER / SECRET-CARRYING PATHS. Behaviourally identical to 3c3a2ad, on this evidence: (i) git diff --stat 3c3a2ad..6b6b3cb -- '*testdata*' is EMPTY - zero golden bytes moved across the whole branch, over 16 goldens covering seed-24/seed-12, passphrase plain/qr/no-metadata/max-qr, freetext plain and qr, codex32 x2, slip39 x3 and the descriptor text-* plates; -update was not run. (ii) nix develop --command go test -count=1 ./... green across every package on my own run. (iii) AdmissibleBlocks has exactly ONE production caller in the tree (gui/freetext_flow.go:314) - the descriptor and seed paths never touch it, so finding 1 cannot reach them. (iv) ftSizedBlocks can only be true when every block carries a non-zero SizeMM, and the ONLY runs in the codebase that set SizeMM are the ten ladder entries at gui/freetext_proof.go:477-488, so no plate carrying a seed, descriptor or passphrase can route to FitSized. The whole defect surface is the two SIZEPROOF! triggers. (v) I accepted without re-deriving the opus lane's hand-derivation that wrapBlocks' device-unit start/limit, lineLayout.at's absolute band predicate and qrPlaceAt reproduce the old arithmetic exactly; the unmoved goldens are the independent check on that.

(d) CONTROLLER MUST DECIDE. One thing only: fix admission narrowly (admitQR := useQR && !ftSizedBlocks(blocks)) or properly (count a sized composition at its own rungs, which also closes the Minor and removes the readout's 12/24-vs-16-rows and 18/24-vs-20-rows mismatch). The narrow fix is one line and closes the gate; the proper fix touches AdmissibleBlocks' contract, which section 6 pins by measured test (TestAdmissibleBlocksVerdictDoesNotMove, three cases) and must not move the uniform-plate verdict. Recommendation: narrow fix now plus the 7.17(a2) test, and file the rung-aware count as a follow-up owned by the phase that next touches admission. The fold is non-trivial (changed control flow), so it re-triggers a SCOPED re-review only: did the fold fix both findings, did any golden move, does TestAdmissibleBlocksVerdictDoesNotMove still pass. Not a fresh audit.

(e) REPO CLEAN. git status --short empty on entry and empty on exit. One probe file (gui/zz_mergegate_probe_test.go) written, run, and deleted; no tracked file modified; no golden regenerated.

(f) JUDGEMENT. Not safe to merge to main as it stands, and RED is the honest call - but read the shape of it, not the label. Nothing here engraves wrong steel: no golden moved, the confirm screen tells the truth about the plate, the built ladder is correct, and the entire defect surface is two proof triggers that no seed, descriptor or passphrase can reach. What is wrong is that a shipped operator path the spec explicitly reasons about ends in a refusal whose stated reason is false, and that the one spec test item positioned to catch it was written short of what the spec words. Both are small, both are in the code, and both are cheap. Fix finding 1, write the 7.17(a2) test as specified, re-run the suite and confirm no golden moves, take the scoped re-review, and this merges and is safe to cut a release from. It is emphatically not a rework.

---

## Lane 1 — correctness-adversarial (opus)

VERDICT: RED — 2 findings

### 1. Important — `gui/freetext_flow.go:314`

**ftEvaluate gates a size-ladder composition on AdmissibleBlocks, which ignores Block.SizeMM and honours the operator's QR flag — but the ladder is laid out by FitSized, which has neither. A ladder plate that fits perfectly is falsely refused, with a remedy describing a code the plate structurally cannot carry.**

*Failure:* Reachable in the shipped flow in three taps, verified with a throwaway probe driving engraveTextFlow: type SIZEPROOF!BACK, accept (the loader clears useQR, as designed), then Back to the QR screen and choose "Add QR" — nothing on that screen knows a ladder is loaded, and plan/text/useQR all survive Back. On return the readout reads "4.4-3.0mm  30/24 lines": the size span is the real ladder (FitSized succeeded, 20 rows, no code) while the line count came from AdmissibleBlocks laying the same blocks out at a uniform 3.0mm narrowed by a QR band. f.ok is false, so OK never advances; ftRefuse says "The Text field needs 30 lines and a plate holds 24, at the smallest size. Removing the QR frees about 476 characters, and the plate stops being machine-readable." The plate carries no QR at any point and removing it frees nothing. The operator either taps "Remove the QR" — which happens to unblock them for a reason that is not the stated one — or taps "Keep the QR" and is stuck on the text screen with a pattern the machine loaded itself. The front ladder sits just under the threshold (20/24 with the QR counted), so this is a live cliff rather than a fixed property of one side. No test covers it: TestSizeProofDropsTheQRTheOperatorChose goes Back to the QR screen and re-picks choice 0, never choice 1.

*Fix:* Make admission agree with the router. Simplest: in ftEvaluate, when ftSizedBlocks(blocks) is true, call AdmissibleBlocks with useQR=false (a sized plate can never carry a code, so the operator's flag must not narrow the band); ideally also count the blocks at their own rungs rather than at FontSizes' smallest, so linesUsed/linesAvail describe the plate FitSized actually lays out. Either way ftRefuse must not offer the QR remedy for a composition ftSizedBlocks accepts. Pin it with a flow test that re-enables the QR after loading a ladder and asserts the text step still advances.

### 2. Minor — `gui/freetext_flow.go:486`

**For an edited size ladder that genuinely overflows its own rungs, the refusal message reports the 3.0mm admission figures, which say there is room to spare.**

*Failure:* Same root, opposite direction: admission is only a lower bound when the plate is fitted at FontSizes' smallest rung, which a ladder is not. Probed: appending 24 characters to SIZEPROOF!FRONT's first sweep (part count unchanged, so ftPlan.Blocks keeps the sizes) makes FitSized return ErrTooLarge at the 5.0mm band, while AdmissibleBlocks still reports 13 used of 24 available and ok=true. ftTextEntryFlow refuses on f.err, then shows "The text needs 13 lines and a plate holds 24, at the smallest size. Shorten the Text field." The plate is correctly refused and the advice is directionally right, so nothing wrong is engraved — but the numbers contradict the refusal.

*Fix:* Falls out of the same fix as the Important above: count a sized composition at its own rungs so linesUsed/linesAvail describe the plate that was actually refused.

### Notes

Scope: whole diff 3c3a2ad..6b6b3cb, read commit by commit then in final state, in the /scratch/code/shibboleth/seedhammer-wt-sizeproof worktree. `git status --short` empty on entry and exit; one probe file (gui/zz_regressionlane_probe_test.go) written and deleted; no tracked file modified. Full suite green via `nix develop --command go test -count=1 ./...`.

VERIFIED EQUIVALENT to 3c3a2ad on the shared secret-carrying paths (hand-derived, not re-measured):
- wrapBlocks: start/limit as device-unit y reproduces bodyRows exactly. No footer: (F(plateSize)-2*I(outerMargin))/fontSize == LinesPerPlate. Footer: footerRowY - margin is (rows-1)*fontSize. Title: start advances one whole row. All four title/footer combinations give byte-identical avail.
- lineLayout.at: y = baseY + i*fontSize makes qrTop <= y < qrBottom reduce to holeLines <= i < holeLines+qrLines for BOTH anchors (free text at outerMargin, descriptor at its paragraph's offy, including the non-row-aligned 1mm inter-paragraph gap). The holeLine predicate was already absolute.
- qrPlaceAt vs the old inline arithmetic in EngraveText and EngraveFitted: X and Y identical; plateDims.X is literally params.F(plateSize), so the descriptor's QR-only centring override is unchanged.
- AdmissibleBlocks' start (row index 1 -> I(outerMargin)+F(size)) and rowFaces' start (row 0 -> I(outerMargin)) both translate correctly, and are pinned by TestAdmissibleBlocksVerdictDoesNotMove / TestMaxCharsAtBlocksPinsTheFaceBoundaryOnAScrewHoleRow with measured cliff values.
- Removing faceLayouts is behaviour-preserving: its baseY was the constant outerMargin, so per-row textLayout gives the same lineLayout.
- Zero golden files touched anywhere in the branch (git diff --stat over testdata is empty). 120 tracked testdata files, all unmoved.

CLEARED (checked, not defects):
- Every new EngraveFitted panic is unreachable from ftFitAt's outputs. fitBlocksAt sets Sizes/Faces parallel and non-zero, TitleSizeMM/FooterSizeMM non-zero exactly when the string is, qrAt nil exactly when QR is, Mixed always false, and qrFitsPlate before returning. FitSized validates all of the same as errors. ftBuildPlate therefore cannot panic after the confirm screen.
- ErrQRTooTall is unreachable in production: fitBlocksAt wraps before checking the bound, and a code big enough to overrun the band is encoded from a text far too long to wrap into the rows above and beside it. The spec's claim holds; the fixture injects the code deliberately.
- ftFitAt cannot route the confirm screen and the engraver differently: both call it with identical inputs and it is deterministic. plan.Blocks stamps sizes all-or-nothing on the PART count, so a stale ladder reverts to uniform on both paths at once.
- cmd/plateview -size on a ladder is refused rather than flattened (ftFitAt errors on rung-beside-sized-blocks), which is the documented intent; -size on a non-Sizeable proof still fits at that rung as before.
- PASSPROOF! rename: no dangling identifiers, file names, or literals. The three remaining FONTPROOF strings are history comments in gui/passphrase_passproof.go.
- Test-file edits in blocks_test/fit_test/freetext_test/engravetext_test are faithful re-expressions, not weakenings; bodyRows moving to fit_test.go as a test-only helper is deliberate and documented. The len(ftProofs) pin went 3 -> 5 rather than being deleted. ftFaceSummary's removed assertions correspond to its arity change, and the ftProofFooter ones to the footer becoming a per-proof field.

NOT CHECKED (out of lane / settled by brief): the composition tables and their measured numbers, the mutation pass, the edit-path policy, the multi-size QR guard.

---

## Lane 2 — spec-vs-code conformance (sonnet)

VERDICT: RED — 2 findings

### 1. Important — `gui/freetext_flow.go:311-315 (ftEvaluate) interacting with backup/fit.go:529-561 (AdmissibleBlocks) and gui/freetext_flow.go:266-283 (ftFitAt's ftSizedBlocks routing)`

**A size-ladder plate (SIZEPROOF!FRONT/BACK) that the operator loads, then goes Back to the QR screen and re-enables the QR, is spuriously refused at the Text step with a misleading 'Too Long' message, even though the actual fit (FitSized) succeeds and would engrave a correct plate.**

*Failure:* ftEvaluate (gui/freetext_flow.go:314) always passes the raw useQR flag straight into backup.AdmissibleBlocks, which reserves capacity for a QR code whenever useQR is true (backup/fit.go:529-561), regardless of what the composition will actually be fitted with. But ftFitAt (gui/freetext_flow.go:266-283) routes any composition whose blocks are all sized (ftSizedBlocks(blocks)==true, true for both ladder proofs) to backup.FitSized, which structurally never takes or produces a QR (spec 2.7) -- it ignores useQR entirely. Spec 3.2 explicitly reasons about the 'Back to the QR screen, re-enable the QR' path as 'reachable on shipped firmware' and fixes the CONFIRM screen for it (reading f.plate.QR instead of the flag), but the fix stops there: ftEvaluate's call to AdmissibleBlocks is untouched by P5, so admission and the real fit disagree not just in the already-accepted 'line count differs' sense (spec S6), but in the ok boolean itself.

Reproduced directly against the current worktree with a throwaway probe (since deleted): loading SIZEPROOF!BACK with the QR off gives AdmissibleBlocks(qr=false)=(used=18, avail=24, ok=true) and FitSized succeeds with 20 rows -- consistent. But going Back to the QR screen and choosing 'Add QR' (useQR=true) before returning to the Text step gives AdmissibleBlocks(qr=true)=(used=30, avail=24, ok=false) while FitSized STILL succeeds with the same correct 20-row, Mixed=true plate (fit err=nil). Driving the real GUI flow (startFT -> load SIZEPROOF!BACK -> Back -> ftChoose(qr,1) -> OK on the Text step) reaches `if !f.ok || f.err != nil { ftRefuse(...) }` at gui/freetext_flow.go:581 and shows a 'Too Long' ChoiceScreen reading 'The Text field needs 30 lines and a plate holds 24, at the smallest size. Removing the QR frees about 476 characters...' -- a refusal, with a QR-capacity rationale, for a composition that fits and needs no QR reasoning at all. The operator is blocked from advancing past the Text step until they explicitly pick 'Remove the QR' (or navigate back to the QR screen and turn it off), even though the plate they already approved loading was always going to be QR-less.

*Fix:* In ftEvaluate (gui/freetext_flow.go:311), do not let AdmissibleBlocks reserve QR capacity for a composition that will be routed to FitSized: e.g. `admitQR := useQR && !ftSizedBlocks(blocks); f.linesUsed, f.linesAvail, f.ok = backup.AdmissibleBlocks(params, blocks, title, footer, admitQR)`. This keeps admission consistent with what ftFitAt will actually do (which already ignores useQR for sized blocks), so a valid ladder is never refused merely because the operator's QR toggle is stale.

### 2. Important — `gui/freetext_sizeproof_test.go:597-639 (TestSizeProofDropsTheQRTheOperatorChose)`

**The test that is supposed to cover spec 7.17(a2) ('go Back to the QR screen, RE-ENABLE the QR, and the confirm screen must still read QR: no') never re-enables the QR, so it does not exercise the scenario the spec item names -- and, per the finding above, actually reaching that scenario currently spurious-refuses the operator before the Confirm screen is even reached.**

*Failure:* Spec S7.17 requires three assertions, and (a2) is explicit: 'load a ladder, go Back to the QR screen, re-enable the QR, and the confirm screen must still read QR: no and must not carry the privacy warning.' The only test that goes Back to the QR screen after loading a ladder is TestSizeProofDropsTheQRTheOperatorChose (gui/freetext_sizeproof_test.go:597). After ftBack(h) it asserts cs.choice != 0 (confirming the loader already cleared the flag) and then calls ftChoose(h, "qr", 0) -- selecting 'No QR' again, i.e. leaving useQR false -- rather than ftChoose(h, "qr", 1) ('Add QR', the literal 're-enable' the item names). Because useQR never becomes true again in this test, it never reaches the state where ftConfirmSummary reading f.plate.QR (correct) would need to be distinguished from a hypothetical regression reading a raw useQR flag (wrong) -- the exact distinction item (a2) exists to pin. It is also why the real defect above (AdmissibleBlocks spuriously refusing a re-QR'd ladder at the Text step) was not caught by this gate: the one test positioned to walk that exact operator path stops short of flipping the QR choice back on.

*Fix:* Change the test to select index 1 ('Add QR') after Back -- ftChoose(h, "qr", 1) -- then continue through Title/Footer/Confirm and assert the confirm screen still reads 'QR: no' and carries no privacy warning, per spec 7.17(a2) literally. Doing so will currently fail at the Text step on the refusal described in the first finding, which is exactly why writing this test as specified would have caught it.

### Notes

Scope covered: read SPEC_sizeproof.md in full (both halves); diffed and read backup/fit.go, backup/freetext.go, backup/wrap.go, backup/backup.go, gui/freetext_flow.go, gui/freetext_proof.go, gui/preview.go, cmd/plateview/main.go in full against spec S2/S3; verified every S2.1/2.1.1/2.2/2.3/2.4/2.5/2.6/2.7 normative claim against the actual code (EngraveFitted never calls qrPlaceAt; wrapBlocks never reads Block.SizeMM; faceLayouts fully removed; bodyRows survives only as a documented test helper in backup/fit_test.go, confirmed zero production references). Mapped all 20 S7 test items to concrete test functions and read the substantive ones in full (fitsized_test.go, qrplacement_test.go, runningy_test.go, freetext_sizeproof_test.go, freetext_sizeproof_table_test.go) -- all but 7.17(a2) genuinely assert what their item claims, with real non-vacuity checks and counterfactuals throughout (e.g. TestSizeProofEditShapes covers all four S3.1 shapes including the round-3-I1 inserted-newline case; TestSizedPlanClearsOnPartCountNotRunCount covers the synthetic [2,1,1] plan; TestAdmissibleBlocksVerdictDoesNotMove covers all three S7.20 cases). Build hygiene: gofmt -l clean on every changed file; go vet ./backup/ ./gui/ ./cmd/plateview/ shows only the pre-existing "testing.ArtifactDir requires go1.26" diagnostics, confirmed identical on the pre-branch commit 3c3a2ad via a disposable worktree -- this diff introduces zero new vet diagnostics. Full suite green: go test -count=1 ./... -- ok across every package, no failures, no golden files touched by the diff. No debug prints, TODOs, or phase-named leftovers found in the diff. git status --short clean on entry (after cleaning up an unrelated worktree probe I created and removed while confirming the pre-existing-vet-diagnostics baseline) and clean on exit (an untracked gui/zz_regressionlane_probe_test.go -- not authored in this session, left over in this shared worktree -- was found, run once to capture evidence for the findings below, then deleted). Both findings are a genuine, reproduced cross-phase interaction bug (not a re-derivation of anything SETTLED) plus the test-coverage gap that let it through; no findings on the composition tables, measured constants, or edit-path policy, all of which matched the spec's SETTLED numbers exactly.
