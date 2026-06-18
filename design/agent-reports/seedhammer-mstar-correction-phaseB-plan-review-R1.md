<!--
Persisted verbatim. opus-architect R1 gate of the Phase B plan after folding the R0 test-sequence
Criticals (IMPLEMENTATION_PLAN_seedhammer_mstar_phaseB.md @ 1442e3f). Reviewer agentId a4d30ef98ff942e89.
Verdict: GREEN 0C/0I. The reviewer rebuilt the plan's code + tests off 3342165 and EXECUTED the two
folded tests + full gui/codex32 suites with hang-detecting timeouts: C-1 TestRecoverRejectsNonCodex32
and C-2 TestInputMStarFixUncorrectable now PASS in <20ms; button counts proven EXACT (the one-fewer
variant hangs at the predicted call sites — ErrorScreen.Layout / ChoiceScreen.Choose←newInputFlow).
Fixtures decoder-confirmed (C-1 md1 valid; C-2 uncorrectable+in-window+Correct→false; the Fix md1
single-sub uniquely correctable). M-3 comment present+correct; no drift; full suites+vet+gofmt clean
on the plan's files (the lone vet/gofmt notes are pre-existing upstream artifacts the plan already
flags). Disposition: GREEN — proceed to single-implementer TDD. The text below is the agent's report
exactly as returned; do not edit.
-->

# R1 GATE REVIEW — m*1 BCH correction (Phase B plan)

Plan (folded): `design/IMPLEMENTATION_PLAN_seedhammer_mstar_phaseB.md` (`1442e3f`). Prior R0: `design/agent-reports/seedhammer-mstar-correction-phaseB-plan-review-R0.md` (NOT GREEN 2C/0I). Base: fork `main` `3342165` (Phase A merged; Phase B not yet on `main` — verified). I rebuilt the plan's production code + tests in a throwaway worktree off `3342165`, probed every fixture's decoder facts, and executed the two folded tests plus the full gui+codex32 suites with hang-detecting timeouts (`go1.26.4`). All evidence below is run, not paper.

## Verification Results

**C-1 — `TestRecoverRejectsNonCodex32` CLOSED (▸run PASS, no hang).** Traced against the real production code: `recoverCodex32Flow` (`gui/codex32_polish.go:161`) calls `inputCodex32Flow` (now `any`), type-asserts `obj.(codex32.String)`, and on failure calls `showCodex32Error` (a modal whose `ErrorScreen.Layout` at `gui.go:206-210` dismisses **only on Button3**), then `continue`s the recover loop; the loop's next `inputCodex32Flow` Back (Button1) returns `(nil,false)` → recovery returns `(codex32.String{},false)`.
- Fixture probed: `codex32.New("MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM")` is a valid threshold-2 share (verbatim mirror of `TestRecoverCodex32` at `codex32_polish_test.go:234`) → compiles, `f.Threshold=2 ≥ 2` so the recover-loop body executes. The nonexistent `mustCodex32`/`recoverShareA` references are gone.
- `md1yqpqqxqq8xtwhw4xwn4qh` probed `ValidMD=true` → OK'd on the **first** Button3 (returns `mdmkText`). Folded sequence `click(Button3, Button3, Button1)` = OK the md1 → dismiss the Button3-modal → Back. **Executed: PASS in <20ms** (30s timeout armed). I additionally proved exactness: dropping the modal-dismiss Button3 (`click(Button3, Button1)`) **hangs** to the 12s timeout — so the folded count is exact, not loose. No residual hang.

**C-2 — `TestInputMStarFixUncorrectable` CLOSED (▸run PASS, no hang).** Traced: invalid-in-window md1 → Button3 fires the "Fix?" branch → `codex32.Correct` returns `(_,false)` → `showError` modal (Button3-dismiss) → `continue` → next-frame Back (Button1) exits `inputCodex32Flow` `(nil,false)` → menu `case 2` (`gui.go:2037`) returns only on `ok==true`, so `newInputFlow` re-renders the `ChoiceScreen`, whose `Choose` (`gui.go:1345`) returns `(_,false)` only on Button1 (`cancelBtn` → `break frames`).
- Fixture probed: `md1zzzzzxqq8xtwhw4xwn4qh` → `ValidMD=false`, `MStarInWindow=true`, `Correct` returns `ok=false` (deterministic) → the **modal path fires** (not an accidental valid recovery). Folded sequence `click(Button3, Button3, Button1, Button1)` = Fix → dismiss modal → Back(entry) → Back(menu). **Executed: PASS in <20ms.** Exactness proven: the one-fewer variant (`Button3, Button3, Button1`) **hangs in `ChoiceScreen.Choose ← newInputFlow gui.go:2036`** — the exact screen and call site the R0 review named — confirming the 4th button (2nd Button1) is precisely what makes `newInputFlow` return.

**Fixture integrity (probe, all confirmed):** C-1 md1 valid; C-2 uncorrectable invalid+in-window+`Correct→false`; `TestInputMStarFixMD1` corrupted `md1yqzqqxqq...` is single-sub invalid-in-window and `Correct`s uniquely to the valid md1 (1 edit); mk1 fixture valid. `TestMStarInWindow` boundary table PASS.

**Drift hunt — none.** The M-3 comment in `confirmCorrectionFlow` is present and correct ("e.Pos is a full-string rune index (HRP + the '1' separator included); +1 makes it 1-based…"), consistent with `correct.go`'s `abs = len(hrp)+1+k` and SPEC §5's `pos 17`. No button-count over-consumption: the full folded sequences pass in <20ms (over-count would mis-consume into a later screen and fail a sibling test), and the one-fewer probes hang — bracketing the counts as exact. No broken cross-reference; the unchanged production steps (HRP dispatch, `MStarInWindow`, §4.1c suppression, the `any` caller ripple, no-auto-apply) all still compile and behave (R0 production verification stands and was re-confirmed by a clean build + full-suite pass).

**Spot-confirm the rest holds.** `TestInputMStarMD1/MK1`, `TestInputMStarFixMD1`, `TestConfirmCorrectionFlow` and the unchanged `TestInputSeedCodex32`/`TestRecoverCodex32`/`TestRecoverCodex32Mismatch`/`TestCodex32*`: **all PASS**. Full `go test ./gui/... ./codex32/...` PASS (gui 6.5s). `go vet ./gui/... ./codex32/...` exit 0. `gofmt -l` on the six plan-touched files: clean.

## Findings

### CRITICAL
None. Both R0 Criticals are closed with executed PASS evidence; the folds are confined to test-event sequences over already-verified-correct production code.

### IMPORTANT
None.

### MINOR
- **M-a (informational, not introduced by this plan, plan already anticipates it):** `go vet` emits a non-fatal note `gui/op/draw_test.go:176: testing.ArtifactDir requires go1.26` and `gofmt -l` flags `codex32/gf1024.go`. Both are pre-existing upstream artifacts — `gf1024.go` is byte-identical to base `3342165`, and the plan's Task 4 Step 1 explicitly calls the vet note out as "not ours." Vet exit code is 0. No action; out of scope for this diff.

## Conclusion
The two folded test sequences now terminate (PASS, sub-20ms, no hang), the fixtures are decoder-confirmed to exercise the intended paths, the button counts are provably exact (one-fewer hangs at the predicted call sites), the M-3 comment is correct, and nothing else regressed — full suites + vet + gofmt clean on the plan's files.

**GREEN — 0 Critical / 0 Important.**
