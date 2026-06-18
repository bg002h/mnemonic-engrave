<!--
Persisted verbatim. opus-architect R0 gate of the Phase B implementation plan
(IMPLEMENTATION_PLAN_seedhammer_mstar_phaseB.md @ b529dc1). Reviewer agentId aba69750fff48e6eb.
Verdict: NOT GREEN 2C/0I (3 minor). The reviewer BUILT the plan's production code + tests in a
throwaway worktree and EXECUTED them with hang-detecting timeouts against fork main 3342165. The
PRODUCTION code was verified correct end-to-end (HRP dispatch, MStarInWindow boundaries, §4.1c
ParsePrefix suppression, the Button3 dual OK/Fix event flow with always-drain, the any-return
caller ripple compiling at both sites + reaching engraveObjectFlow, all §2/§4 safety invariants,
Edit.Pos full-string-rune +1). BOTH Criticals are defects in the PLAN'S OWN TEST event sequences
(not production): C-1 TestRecoverRejectsNonCodex32 — the md1 is VALID so it's OK'd first, the
type-assert modal dismisses on Button3 not Button1 → hang; also cited nonexistent helpers
mustCodex32/recoverShareA. C-2 TestInputMStarFixUncorrectable — after Back-out-of-entry returns
(nil,false) the menu loops on its ChoiceScreen, needs a second Back to exit → hang. Reviewer
verified the corrected sequences pass. Disposition: folded C-1 (rewrote with codex32.New inline
share + click(Button3,Button3,Button1)), C-2 (click(...,Button1,Button1)), M-3 (full-string-pos
comment). Test-only folds over verified-correct production code; re-dispatching the gate.
The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — m*1 BCH correction (Phase B plan)

Plan: `design/IMPLEMENTATION_PLAN_seedhammer_mstar_phaseB.md` (`b529dc1`). Spec: `design/SPEC_seedhammer_mstar_correction.md` (GREEN R1). Source verified against fork `main` `3342165` at `/scratch/code/shibboleth/seedhammer`. I built the plan's production code + tests in a throwaway worktree and executed them with hang-detecting timeouts; findings below are evidence-backed, not paper analysis.

## Verification Results

**1. HRP dispatch correctness (`validateMStar`, §4.1a) — CORRECT.** `New` for ms / `ValidMD` for md / `ValidMK` for mk matches the verifiers and their self-gating. `ValidMD` (`mdmk.go:124`) has no upper bound (data ≥13); `ValidMK` (`mdmk.go:136-148`) switches on data-part length (14..93 regular / 96..108 long, 94..95 rejected); `New` (`codex32.go:98-107`) rejects out-of-window total length. The keypad uppercases (`gui.go:1202`); `ValidMD`/`ValidMK`/`New` are case-tolerant via `splitHRP`+engine case-state (`mdmk.go:94-117`), and `strings.EqualFold` in `validateMStar` matches "MD"/"MK"/"MS". `mdmkText(frag)` is the right type — `engraveObjectFlow` (`gui.go:1865`) routes `case mdmkText:` → `mdmkFlow`. Verified live: `md1yqpqqxqq8xtwhw4xwn4qh` and the mk1 literal both validate and return `mdmkText`; `TestInputMStarMD1`/`TestInputMStarMK1` PASS.

**2. `MStarInWindow` boundaries (§4.1b) — CORRECT.** Every bracket in `codex32/mstar.go` matches `codex32.go:41-44` (`shortCodeMin/Max`=48/93, `longCodeMin/Max`=125/127) and `mdmk.go:41,47-50` (`mdmkShortSyms`=13, `mkRegular`=14..93, `mkLong`=96..108). `pad("ms",45)`=total 48 confirmed. `splitHRP` (`codex32.go:453`) data = chars after the first `1`; `1` is not in the bech32 alphabet so this is unambiguous. The plan's full boundary table test PASSES.

**3. §4.1c suppression — CORRECT (executed).** For an md/mk fragment `ParsePrefix` returns `errInvalidThreshold` the instant `data[0]` is a non-threshold char (`polish.go:102-109`). `mstarFeedback` ignores `perr` for md/mk entirely and the field line is gated to `ms`/empty. Verified on rendered frames: `md1y` shows neither "bad threshold" nor "bad checksum"; corrupted md1 (in window) shows "bad checksum" only; `ms12name` still shows "id NAME · thr 2"; `ms11` still shows "bad threshold". Correct at every length class.

**4. Event-consumption / hang — production code CORRECT; two PLAN TESTS HANG.** `clicked3 := okBtn.Clicked(ctx)` is always called, draining Button3 even when invalid-not-in-window (no queue-head block). `confirmCorrectionFlow` drains Button2 every frame (mirrors the `confirmCodex32Flow` R0-C1 idiom). The full `TestInputMStarFixMD1` sequence (runes→Fix→accept→OK) terminates with `mdmkText` — PASS, no cross-frame Button3 mis-consumption (each `click` is a distinct press/release pair and each `Clicked` loop consumes exactly one pair). **However, two of the plan's own test event sequences hang** (see CRITICAL-1/2) — both are test-event defects, not production defects (corrected sequences pass).

**5. Caller ripple — COMPILES + behavior preserved.** `inputCodex32Flow` → `any` compiles at both sites: menu `case 2` (`gui.go:2037`, into `(any,bool)`) and `recoverCodex32Flow` (now `obj.(codex32.String)`). The `any` reaches `engraveObjectFlow` via `newInputFlow`→`obj`→`engraveObjectFlow(ctx,th,obj)` (`gui.go:1479-1486`). Full existing suite (`TestInputSeedCodex32`, `TestRecoverCodex32`, `TestRecoverCodex32Mismatch`, all `TestCodex32*`) PASSES unchanged.

**6. Safety invariants (§2) — SATISFIED.** No auto-apply: `kbd.Fragment = res.Corrected` only after `confirmCorrectionFlow` returns true; the corrected string then re-validates through `validateMStar` before OK (next-frame gate). `codex32.Correct`'s mandatory `reverify` (`correct.go:132`) is the backstop. The per-position diff is the universal anchor; `id·thr·share` is gated `hrp=="ms"`. md/mk reach engrave via `mdmkFlow`'s `ChoiceScreen` (the md/mk review) with no unintended extra confirm — consistent with spec.

**7. Test fidelity — mixed (two defects, one helper gap).** Happy/Fix/confirm tests are faithful and pass. `TestInputMStarFixMD1`'s `corrupted` is a single data-part substitution (Pos 5 = data index 2, one edit `Z→P`, corrects uniquely — verified). The 5-error uncorrectable literal returns `(_,false)` deterministically. Imports are satisfied. But `TestInputMStarFixUncorrectable` and `TestRecoverRejectsNonCodex32` hang as written, and the latter references nonexistent `mustCodex32`/`recoverShareA`.

**8. `Edit.Pos` display — CORRECT.** `Edit.Pos` is a full-string rune index (`correct.go:5-12,114,129`: `abs = len(hrp)+1+k`), so `e.Pos+1` is 1-based full-string position (HRP + `1` counted). For the verified md1 fix, Pos 5 → "pos 6". No off-by-one; consistent with SPEC §5's `pos 17` example.

## Findings

### CRITICAL

**C-1 — `TestRecoverRejectsNonCodex32` hangs the suite (plan Task 2 Step 1, lines 235-247).** The md1 literal `md1yqpqqxqq8xtwhw4xwn4qh` is a *valid* md1 (verified). Inside `recoverCodex32Flow`→`inputCodex32Flow`, the first Button3 is consumed as **OK** (valid → returns `mdmkText`). The type-assert fails → `showCodex32Error`, whose `ErrorScreen` dismisses only on **Button3** (`gui.go:206-210`). The plan supplies `click(Button3, Button1)`: the lone remaining Button1 never dismisses the modal, and in a direct-call (no `FrameCallback`→no `ctx.Reset`, `ctx.Done` never set) the modal loop spins forever. **Confirmed: 15 s timeout in `ErrorScreen.Layout`.** Required fix: `click(&ctx.Router, Button3, Button3, Button1)` (OK the md1 → dismiss modal → Back). Also resolve the `mustCodex32`/`recoverShareA` references (they do not exist; `TestRecoverCodex32` uses an inline `codex32.New(...)` literal — mirror that). Corrected sequence verified PASS.

**C-2 — `TestInputMStarFixUncorrectable` hangs the suite (plan Task 3 Step 5, lines 614-627).** After Fix→"no fix" modal→dismiss→Back, `inputCodex32Flow` returns `(nil,false)`. The menu `case 2` only returns on `ok==true` (`gui.go:2038-2040`), so `newInputFlow` loops back and re-renders the `ChoiceScreen`, which then blocks waiting for input the test never supplies. **Confirmed: 15 s timeout in `ChoiceScreen.Choose`.** The test must also dismiss the menu, e.g. append a second `click(Button1)` so the `ChoiceScreen` returns `ok=false` and `newInputFlow` returns `(nil,false)`. Corrected sequence verified PASS. (Note: the existing `TestInputSeedCodex32` never hits this because its OK path returns `true` and exits the menu immediately — the gap is unique to the not-returning paths through `newInputFlow`.)

These are Critical because, left as written, `go test ./gui/...` (plan Task 2 Step 7, Task 3 Step 6, Task 4 Step 1) never completes — it dies on the test timeout, masking real results and blocking the GREEN gate. Both are pure test-event-sequence defects; the production `inputCodex32Flow`/`confirmCorrectionFlow`/`recoverCodex32Flow`/`showError` are correct.

### IMPORTANT

None. The production code (the actual subject of the gate) compiles, vets clean, is gofmt-clean, preserves all existing behavior, satisfies every §2/§4 invariant, and its event flow terminates correctly on all designed paths.

### MINOR

- **M-1 (Task 2 Step 4 note, line 415):** the optional temporary `confirmCorrectionFlow` stub `return false` would make `TestInputMStarFixMD1` fail if shipped; the plan already recommends implementing the real body. Keep the recommendation; do not ship the stub.
- **M-2 (Task 3 Step 5 footnote, line 629):** the "5 substitutions" worry is moot for the chosen literal — `Correct` returns `(_,false)` deterministically (verified), so the modal path fires. Fine as-is once C-2's event sequence is fixed.
- **M-3 (cosmetic):** `confirmCorrectionFlow` diff shows `pos N` counting the HRP + `1` separator (full-string index). Acceptable and matches SPEC §5, but a one-line code comment noting "position is full-string, HRP included" would prevent future off-by-one confusion. Non-blocking.

## Conclusion

The architecture is sound and the production implementation the plan specifies is correct end-to-end — verified by building and running it. The only defects are two of the plan's own test event sequences, which omit terminating events and consequently hang the test binary (one also cites nonexistent helpers). Because those tests are mandated steps that gate GREEN and would wedge CI, they block the gate until their event sequences are corrected (both fixes verified to pass).

**NOT GREEN — 2 Critical / 0 Important.**
