<!--
Persisted verbatim from the opus-architect R0 gate review of the Slice 3
implementation plan (IMPLEMENTATION_PLAN_seedhammer_passphrase_flow.md @ b7e7fd3).
Reviewer agentId: aca4dc508f0d07d6e. Round: R0.
Verdict: NOT GREEN — 1 Important (I-1) open.
Disposition: I-1 FOLDED into the plan (Step 3 + file-structure table now add
"fmt" to BOTH gui/gui.go AND gui/gui_test.go). Minors M-1/M-2/M-3 are
non-blocking notes already covered by the plan's own NOTEs (dead-loop removal,
bip39FromWords description, direct-call-form-only). Re-dispatched as R1.
The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — Slice 3 Implementation Plan

**Reviewer:** opus architect (adversarial, read-only)
**Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_passphrase_flow.md` (committed `b7e7fd3`)
**Base:** fork `main` `e990f0b`
**Date:** 2026-06-18

---

## Verification Results

### 1. Task-1 threading completeness & compile-accuracy

Live source confirmed:

- `deriveMasterKey` at `gui.go:187-196`: signature `(m bip39.Mnemonic, net *chaincfg.Params)`, body `seed := bip39.MnemonicSeed(m, "")`. Plan's edit — add `password string` param, change to `bip39.MnemonicSeed(m, password)` — is compile-accurate.
- `masterFingerprintFor` at `gui.go:482-492`: signature `(m bip39.Mnemonic, network *chaincfg.Params)`, calls `deriveMasterKey(m, network)`. Plan's edit — add `, password string`, thread to `deriveMasterKey(m, network, password)` — compile-accurate.
- ALL THREE callers verified:
  - `gui.go:483`: `deriveMasterKey(m, network)` (inside `masterFingerprintFor`) → updated to pass `password`.
  - `gui.go:455`: `masterFingerprintFor(m, &chaincfg.MainNetParams)` (inside `engraveSeed`) → updated to `("", ...)`. Correct (Task 1 only; Task 2 removes this call entirely).
  - `gui.go:2071`: `deriveMasterKey(mnemonic, &chaincfg.MainNetParams)` (inside `SeedScreen.Confirm`) → updated to `("", ...)`. Correct (validates words only).
- No other callers of either function anywhere in the `gui/` package (Grep confirmed).
- Behavior unchanged: `bip39.MnemonicSeed(m, "")` is identical before and after. Package compiles standalone after Task 1. ✓

### 2. Task-1 test validity

- `emptyBIP39Mnemonic` exists at `gui.go:511` (not `gui_test.go`; it's in the main package file). The function returns a `bip39.Mnemonic` of `-1` sentinel words. The dead loop in the plan's test draft (`for i := range m { w, _ := bip39.ClosestWord(...); _ = w }`) compiles but does nothing — the plan explicitly instructs removing it. Not a compile error.
- `bip39FromWords` does NOT exist anywhere in the `gui/` package. The implementer must add it. The description ("splits `s` and maps each via `bip39.ClosestWord` (exact)") is sufficient; `strings` is already imported in `gui_test.go`. This is a construction task left implicit but clearly described.
- "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" — confirmed valid 12-word BIP-39 mnemonic at `bip39/bip39_test.go:114` (standard test vector, entropy=`00`×16). `deriveMasterKey` will return `true`.
- `masterFingerprintFor(m, &chaincfg.MainNetParams, "")` vs `("TREZOR")`: `bip39.MnemonicSeed` at `bip39.go:217-226` — PBKDF2-2048 with `salt = "mnemonic"+password`. Empty vs "TREZOR" produce distinct seeds → distinct keys → distinct fingerprints. Test assertion `bare != pass` is sound. ✓

### 3. Task-2 `engraveSeed` edit

Current body at `gui.go:454-480`:
```
func engraveSeed(params engrave.Params, m bip39.Mnemonic) (Plate, error) {
    mfp, err := masterFingerprintFor(m, &chaincfg.MainNetParams)   ← line 455 (DELETE)
    if err != nil {                                                   ← line 456 (DELETE)
        return Plate{}, err                                           ← line 457 (DELETE)
    }                                                                 ← line 458 (DELETE)
    qrc, err := qr.Encode(...)                                       ← line 459 (first err decl, now correct)
```

After deleting the first 4 lines and adding `mfp uint32` param, `qrc, err := qr.Encode(...)` becomes the first `err` declaration. No "declared but not used", no "redeclared" issue. `seedqr.QR(m)` is words-only (no passphrase involvement — it encodes the word indices directly). ✓

### 4. Task-2 `backupWalletFlow` rewrite

Current body at `gui.go:1888-1913` confirmed as exactly `for{Confirm; engraveSeed; inline-error-loop; Engrave}` — no `qaProgram`, no `backupWallet` branch, no extra state. The wholesale replace drops nothing that exists.

Plan's replacement: verified compile-accurate:
- `ppChoice := &ChoiceScreen{Title: ..., Lead: ..., Choices: ...}` — all fields exist (`Title string`, `Lead string`, `Choices []string` at `gui.go:1283-1286`). ✓
- `ppChoice.Choose(ctx, th)` returns `(int, bool)`. ✓
- Nested `if sel, ok :=` / `if pass, ok :=` / `sel, ok :=` — no shadowing; each `:=` introduces at least one new variable in its scope. ✓
- `engraveSeed(ctx.Platform.EngraverParams(), mnemonic, mfp)` — matches new Task-2 signature. ✓
- Single engrave path per iteration (`engraveSeed` called once per outer loop). ✓
- Termination: `ss.Confirm` returning false → `return`; `NewEngraveScreen.Engrave` returning true → `return`. All back paths `continue`. ✓
- Back-semantics: Back/Skip from ppChoice → falls through to `engraveSeed` with bare `mfp`. Back from `passphraseFlow` → same. Back from fpChoice → `continue` → re-Confirm. ✓

### 5. `showSeedError` accuracy

The existing inline error loop at `gui.go:1896-1905` uses `break` out of the inner loop + `continue` on the outer loop. `showSeedError` uses `return` instead — semantically equivalent from the caller's perspective (the caller `continue`s after `showSeedError` returns in all error paths). `ErrorScreen.Layout(ctx, th, dims) (op.Op, bool)` at `gui.go:205` confirmed. `SeedScreen.Draw(ctx, th, dims, mnemonic) op.Op` at `gui.go:2127` confirmed — 4 params, returns `op.Op`. ✓

### 6. `passphraseFlow` compile-accuracy

- `NewPassphraseKeyboard(ctx *Context) *PassphraseKeyboard` at `passphrase_keyboard.go:62`. ✓
- `Update(ctx *Context) bool` at `passphrase_keyboard.go:200`. ✓
- `Layout(ctx *Context, th *Colors) (op.Op, image.Point)` at `passphrase_keyboard.go:339`. ✓
- `Fragment string` field at `passphrase_keyboard.go:48`. ✓
- `layout.Rectangle{Max: dims}`, `CutTop(leadingSize)`, `CutBottom(8)`, `content.S(kbdsz)` — all verified against `layout/layout.go`. `S(sz image.Point) image.Point` at `layout.go:55`: returns bottom-center offset. ✓
- `kbdOp.Offset(content.S(kbdsz))` — `Offset` takes `image.Point`; `S` returns `image.Point`. ✓
- `layoutNavigation(&ctx.B, th, dims, []NavButton{...}...)` — spread form used throughout `gui.go` (confirmed at lines 652, 654, 740, 742, 822, 824). ✓
- `assets.IconBack`, `assets.IconCheckmark` — used throughout `gui.go`. ✓
- `layoutTitle(ctx, dims.X, th.Text, "Enter Passphrase")` at `gui.go:1633` — `(ctx *Context, width int, col color.RGBA, title string)`. ✓
- `op.Color(&ctx.B, th.Background)`, `op.Layer(kbdOp, nav, title, op.Color(...))` — `op.Layer` is variadic `(...Op) Op`. ✓
- Direct-call test pattern: events pre-queued → first frame processes `kbd.Update` (runes), then `okBtn.Clicked` fires (Button3) → returns. Verified via trace of EventRouter queue. Does NOT loop forever for test cases. ✓

### 7. `"fmt"` import

Confirmed: `gui.go` imports block (lines 4-41) does NOT contain `"fmt"`. Plan's Task 2, Step 3 adds it. `fmt.Sprintf("%.8X", mfp)` matches `backup.go:182`'s format `%.8X`. ✓

**IMPORTANT:** `gui_test.go` (lines 3-27) does NOT import `"fmt"`. `TestEngraveFingerprintChoiceMapping` (Task 2, Step 8) uses `fmt.Sprintf("%.8X", bare)` and `fmt.Sprintf("%.8X", pass)`. The plan specifies adding `"fmt"` only to `gui.go`, not to `gui_test.go`. This will produce a compile error: `undefined: fmt`. **The plan is missing a `"fmt"` import addition to `gui_test.go`.**

### 8. No golden regen / scope

`backup.Seed.MasterFingerprint` is a `uint32` field — unchanged structurally. `frontSideSeed` at `backup.go:181-188` reads it unchanged. `backup/`, `bip39/`, `bip32/`, `PassphraseKeyboard` untouched. The bare-fp golden `TestSeed*` in `backup/` call `backup.EngraveSeed` directly with `MasterFingerprint` set explicitly — they do not go through `gui.go` functions. ✓

`TestEngraveFingerprintChoiceMapping` is non-vacuous: it verifies (a) `bare != pass` for a known mnemonic+passphrase, (b) `ChoiceScreen` index 1 is returned for Down+Button3, (c) the `fmt.Sprintf` format produces the correct label string. ✓ (subject to the `"fmt"` import fix)

### 9. Atomicity / build order

Task 1 compiles standalone: all callers updated to pass `""`, behavior unchanged. The `engraveSeed` signature stays 2-arg until Task 2. ✓

Task 2 must land atomically: `engraveSeed` signature change + its sole caller `backupWalletFlow` + `passphraseFlow` + `showSeedError` + `"fmt"` import all in one commit. This is stated in the plan. The `backupWalletFlow` calls `engraveSeed(ctx.Platform.EngraverParams(), mnemonic, mfp)` (3-arg) which requires the new `engraveSeed` signature. No intermediate non-compiling state within a task. ✓

No forward references: all new functions (`passphraseFlow`, `showSeedError`) are called from `backupWalletFlow` — Go is not order-dependent for functions within a package. ✓

---

## Issues

### IMPORTANT

**I-1: `"fmt"` not added to `gui_test.go`**

`TestEngraveFingerprintChoiceMapping` (Task 2, Step 8, `gui_test.go`) uses `fmt.Sprintf("%.8X", bare)` and `fmt.Sprintf("%.8X", pass)`. The plan specifies adding `"fmt"` to `gui.go` (Task 2, Step 3) but never mentions adding `"fmt"` to `gui_test.go`. Since `gui_test.go` currently has no `"fmt"` import, this test will produce a compile error: `undefined: fmt`.

**Fix:** Add `"fmt"` to the import block of `gui_test.go` in Task 2, Step 3 (alongside the `gui.go` import addition) or as a separate substep.

---

### MINOR

**M-1: Dead code in Task 1, Step 1 test draft**

The plan presents a `TestMasterFingerprintPassphrase` draft that opens with `m := emptyBIP39Mnemonic(12)` and a for-range loop that discards all results, immediately followed by the NOTE instructing to "Remove the dead `emptyBIP39Mnemonic` loop above." The draft as written compiles (dead code only), but the NOTE and the "direct builder" form instruction are in the right place. Implementer must heed the NOTE; the dead block should not appear in the committed test.

**M-2: `bip39FromWords` implementation not fully specified**

The plan says "add a tiny `bip39FromWords(t, s)` that splits `s` and maps each via `bip39.ClosestWord` (exact)" without showing the function body. The description is unambiguous and `strings` is already imported in `gui_test.go`. No blocker, but the implementer must write it from the description. Standard pattern; low risk.

**M-3: `runUI`-based `TestPassphraseFlow` shown before being discarded**

Task 2, Step 1 presents the `runUI`/package-var version first (which references undeclared `passphraseFlowResult`/`passphraseFlowOK` package vars — would not compile as written), then a NOTE explicitly says "Drop the `runUI`/package-var version above. Use the direct-call form." The direct-call form is correct. Implementer must use ONLY the direct-call form and ignore the first block. The NOTE is present and explicit; no compile error if the implementer follows it.

---

## Verdict

**NOT GREEN — 1 Important open.**

**I-1** (missing `"fmt"` in `gui_test.go`) must be folded into the plan before implementation begins.

**Fix required:** In Task 2, Step 3 (or as a new substep), explicitly add `"fmt"` to the `gui_test.go` import block. The one-line fix: the plan's Step 3 text should read "Add `"fmt"` to the import block of BOTH `gui/gui.go` AND `gui/gui_test.go`."

All other elements — threading completeness, `engraveSeed` sig change, `backupWalletFlow` control flow, `showSeedError` accuracy, `passphraseFlow` compile-accuracy, direct-call test validity, ChoiceScreen index semantics, atomicity, back-navigation semantics, golden guard, `backup`/`bip39`/`bip32`/`PassphraseKeyboard` scope — are VERIFIED CORRECT against the live source.
