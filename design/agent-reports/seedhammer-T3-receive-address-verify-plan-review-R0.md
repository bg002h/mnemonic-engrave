<!--
Persisted verbatim. opus-architect R0 GATE review of IMPLEMENTATION_PLAN_seedhammer_T3_receive_address_verify.md
(commit f943d28) BEFORE code. Reviewer agentId ac938f003697aadf5. Method: materialized the plan
task-by-task in a detached worktree off d334861, built, ran every test. Verdict: NOT GREEN — 1C/3I. The
headless address.Find core is CORRECT (embedded addresses cross-verified vs address_test.go table;
keyless-no-panic, case-preservation, scan-recognizer, alloc-gate, read-only all hold). But the plan does
NOT build-and-pass its own load-bearing tests: C1 (runVerify's "Verifying…" via showMessage BLOCKS like
showError → Find never runs, flow hangs — the central GUI flow is non-functional as written); I1
(showMessage undefined + unsupplied); I2 (existing T1 TestDescriptorConfirmAddressAffordance regresses
from the Show/Verify ChoiceScreen, no plan task to fix it, touches spec §2.6); I3 (spec §2.1b mandated
derivation-error-propagation test absent — code is correct, test missing); M1 (xpubs function-local →
TestFind won't compile, hoist it); M2 (TestScanRecognizesAddress single-call returns errScanInProgress +
placeholder xpub); M3 (scan-recognizer `address.DecodeAddress` collides with gui's seedhammer.com/address
import → alias btcaddr); M4 (NewAddressKeyboard must set revealed=true AFTER NewPassphraseKeyboard's
Clear()). Worktree removed; fork clean at d334861; nothing committed/pushed. Disposition: fold all →
re-dispatch R1. Text below verbatim (entities un-escaped: &lt;→<, &gt;→>, &amp;→&).
-->

# R0 GATE REVIEW — IMPLEMENTATION_PLAN_seedhammer_T3_receive_address_verify.md (`f943d28`)

## Verification Results (materialized at `d334861`, go1.26.4)

I transcribed the plan task-by-task into a throwaway worktree, built it, and ran every test. **Worktree removed; fork clean at `d334861`; nothing committed/pushed.**

| Check | As-written by the plan | Result |
|---|---|---|
| `address.Find` (Task 1 code) | compiles, logic correct | **PASS** |
| `TestFind` (all 7 subcases) | **does NOT compile** (`xpubs` undefined at pkg scope) | **RED → fixed (hoist `xpubs`) → PASS** |
| `TestFindKeylessNoPanic` | correct | **PASS** (ErrUnsupported, no panic) |
| §2.1b derivation-error propagation | code correct; **test absent from plan** | code **PASS** (verified ad-hoc) / test MISSING |
| `TestRunVerifyResult` | **FAILS** (stuck on "Verifying…" forever) | **RED → required 3 fixes → PASS** |
| `TestTypeAddressCasePreserved` | correct | **PASS** (`bc1Q3` preserved, unmasked) |
| `TestScanRecognizesAddress` | **FAILS** (`errScanInProgress`, single-call) | **RED → fixed (drive loop) → PASS** |
| `TestDescriptorConfirmAddressAffordance` (existing) | **REGRESSION — FAILS** | **RED → required test update → PASS** |
| `go build ./...` / `go vet` / `gofmt -l` | clean (after fixes) | **PASS** |
| `TestAllocs` | 0-alloc preserved (ChoiceScreen inside click branch) | **PASS** |
| Full `go test ./...` | green after the regression fix | **PASS** |

Cross-checked embedded addresses against `address/address_test.go`: `bc1qkwl5...nm8` = wpkh receive[2] ✓, `bc1qvwlscf...x7x` = wpkh change[1] ✓, `bc1qt77623...0k0` = multi receive[0] ✓, `bc1qwh9lh...zlp` = multi change[1] ✓. All correct.

## Findings

### CRITICAL

**C1 — `runVerify`'s "Verifying…" frame is non-functional as written; the flow hangs and `Find` never runs.** (Plan Task 2 Step 3a, lines 221–244.) The plan calls `showMessage(ctx, th, "Verify address", "Verifying…")` as the first statement, then claims "The 'Verifying…' first call renders one frame then returns so `Find` runs." But `showMessage` is specified to **mirror `showError`**, which is a **blocking `for !ctx.Done` loop** (`gui/slip39_polish.go:22`). A blocking modal never returns until the user dismisses it — so `address.Find` is never reached and the user is frozen on "Verifying…". I reproduced this exactly: `TestRunVerifyResult` rendered `"VerifyingVerifyaddress"` ×8 and never showed Receive/Not-found/Invalid. **Fix:** the "Verifying…" indicator must be a single, non-blocking `ctx.Frame(...)` render (not a `showMessage`/`showError`-style loop), THEN compute, THEN a blocking result modal. (Confirmed working once changed to a one-shot `scr.Layout`+`ctx.Frame`.) The plan's own load-bearing `TestRunVerifyResult` does not pass against the plan's own `runVerify`.

### IMPORTANT

**I1 — `showMessage` does not exist and the plan does not supply it.** (Task 2 Step 3a parenthetical, line 244.) Only `showError` exists (`gui/slip39_polish.go:22`); grep confirms no `showMessage`/`showSuccess`/`showInfo`. The plan hand-waves ("if a suitable helper exists, reuse it — else add a small one here") without giving the implementation. Combined with C1, the result-rendering primitive is under-specified. **Fix:** the plan must include the `showMessage` body (blocking result modal mirroring `showError`, dismiss on **Button3**) AND specify the dismiss button explicitly.

**I2 — Existing T1 test `TestDescriptorConfirmAddressAffordance` regresses; the plan has no task to update it.** (Task 2 Step 3b; spec §2.6.) The existing test (`address_polish_test.go:139`) drives `click(Button2)` and expects the address view to open directly. The interposed Show/Verify `ChoiceScreen` now requires a second selection, so the address view never appears → test fails. This is observable behavior change to a prior cycle. The plan's File-manifest claims "show-addresses preserved as choice 0" but omits the mandatory test update. **Fix:** add an explicit task to update `TestDescriptorConfirmAddressAffordance` to drive through the new ChoiceScreen (select "Show addresses"), and reconcile spec §2.6's "behaviorally unchanged" against spec §4.2's deliberate Button2-becomes-ChoiceScreen change (the latter is intended; the spec wording should acknowledge the interaction now has one extra step).

**I3 — Spec §2.1b mandates a derivation-error-propagation test; the plan's Task 1 test code omits it.** (Spec §2.1b/§6; plan Task 1 Step 1 only has `TestFind`+`TestFindKeylessNoPanic`.) The `Find` code propagates correctly (I verified ad-hoc with a `<5;7>` range → `address: unsupported range path element` propagated, not swallowed), but the mandated test is absent. **Fix:** add the injection test to Task 1 (e.g. a `<5;7>` range descriptor).

### MINOR

**M1 — `TestFind` does not compile: `xpubs` is function-local to `TestAddresses`.** (Task 1 Step 1, lines 49/51.) The plan references `xpubs[0]` at `TestFind` scope and parenthetically says "reuse the file's `xpubs`" — impossible without hoisting. **Fix:** hoist `xpubs` to a package-level `var` (one-line change; verified).

**M2 — `TestScanRecognizesAddress` does not pass as written.** (Task 3 Step 1, lines 296–308.) `s.Scan(strings.NewReader(...))` returns `(nil, errScanInProgress)` on the first call (data-available read), so `obj.(addressText)` fails immediately. **Fix:** drive `Scan` in a loop until past `errScanInProgress` (the existing `TestScan` idiom). Also the negative case uses `"wpkh(" + /* a valid xpub */ ")"` — a non-compilable placeholder needing a real xpub.

**M3 — Scan-recognizer import collides with the existing `seedhammer.com/address`.** (Task 3 Step 3c, lines 314–319.) In package `gui`, `address` already names `seedhammer.com/address` (`gui/gui.go:23`), which has no `DecodeAddress`. The plan's `address.DecodeAddress(...)` won't compile. **Fix:** import the btcd parser under an alias (e.g. `btcaddr "github.com/btcsuite/btcd/address/v2"`) plus `chaincfg/v2`; call `btcaddr.DecodeAddress`. (Verified.) Minor sub-point: the plan says "Try mainnet then testnet" — I implemented both branches; works and keeps the descriptor's own re-check authoritative.

**M4 — `NewAddressKeyboard` must set `revealed=true` AFTER construction.** (Task 3 Step 3a.) `NewPassphraseKeyboard` ends with `Clear()`, which sets `revealed=false` (`passphrase_keyboard.go:164`). A constructor that relies on a "default" would be reset. **Fix (already minimal):** `k := NewPassphraseKeyboard(ctx); k.revealed = true; return k`. (Verified — case-preservation test passes.)

## Verdict

`NOT GREEN — 1C/3I`

The headless `address.Find` core is correct and its addresses verify against the live table; the keyless-no-panic, case-preservation, scan-recognizer, alloc-gate, and read-only/no-secret invariants all hold. **But the plan as written does not build-and-pass its own load-bearing tests:** C1 (the `runVerify`/"Verifying…" sequence hangs and never runs `Find` — the central GUI flow is non-functional as specified), I1 (`showMessage` undefined and unsupplied), I2 (an existing T1 test regresses with no plan task to fix it, touching spec §2.6), and I3 (spec §2.1b's mandated propagation test is missing). M1–M4 are mechanical compile/test defects. Fold C1+I1 (specify a non-blocking "Verifying…" frame + a concrete `showMessage` body with Button3 dismiss), I2 (add the affordance-test-update task + reconcile §2.6), I3 (add the derivation-error test), and M1–M4, persist this review verbatim, and re-dispatch.
