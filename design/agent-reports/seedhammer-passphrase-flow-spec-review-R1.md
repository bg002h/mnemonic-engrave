# Slice 3: passphrase-flow — SPEC R0 GATE REVIEW — R1

- **Stage:** spec gate round R1 (after folding R0's 2 Critical + 3 Important + minors). FINAL.
- **Spec reviewed:** `design/SPEC_seedhammer_passphrase_flow.md` (committed `b93e964`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `ab5d45fcb3ce0814e`), read-only vs fork `e990f0b`.
- **Outcome:** **GREEN — 0 Critical / 0 Important.** Spec cleared to the plan phase.

> NOTE: verbatim architect output, recovered from the agent transcript; a working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

# R1 ARCHITECT REVIEW — SPEC_seedhammer_passphrase_flow.md (Slice 3)

**Reviewer:** opus architect
**Round:** R1 (post-R0 fold)
**Base:** fork `main` `e990f0b`, `/scratch/code/shibboleth/seedhammer/gui/gui.go`

---

## Fold Verification

### C-1 — `showSeedError` helper (formerly phantom `showSeedErr`)

**Verified GREEN.**

The real inline error loop at `gui.go:1896-1905` is exactly:

```go
errScr := NewErrorScreen(err)
for !ctx.Done {
    dims := ctx.Platform.DisplaySize()
    d, dismissed := errScr.Layout(ctx, th, dims)
    if dismissed { break }
    main := ss.Draw(ctx, th, dims, mnemonic)
    ctx.Frame(op.Layer(d, main))
}
continue
```

The spec's `showSeedError` helper faithfully captures this pattern. Signature accuracy confirmed against actual function signatures:
- `NewErrorScreen(err error) *ErrorScreen` — `gui.go:384` ✓
- `errScr.Layout(ctx *Context, th *Colors, dims image.Point) (op.Op, bool)` — `gui.go:205` ✓
- `ss.Draw(ctx *Context, th *Colors, dims image.Point, mnemonic bip39.Mnemonic) op.Op` — `gui.go:2127` ✓
- `op.Layer(ops ...Op) Op` — `gui/op/op.go:161` ✓

Behavior equivalence: the existing code does `break` then `continue`; the helper callers do `showSeedError(…); continue`. With `dismissed`, the helper returns and caller `continue`s — semantically identical. With `ctx.Done`, same: helper exits the inner loop and returns, caller `continue`s, `ss.Confirm` returns `false` (it also guards on `ctx.Done`), outer loop terminates via `return`. No regression.

All three error sites in the new `backupWalletFlow` (bare-mfp error, passphrase-mfp error, `engraveSeed` error) route through `showSeedError`. DRY, no behavior change. ✓

### C-2 — `ChoiceScreen` state reuse

**Verified GREEN.**

`ChoiceScreen.choice` is a private `int` field (`gui.go:1287`) initialized to zero by the Go runtime on struct allocation. The spec's `backupWalletFlow` declares both `ppChoice` and `fpChoice` with `:=` inside the outer `for` body — brand-new heap allocations each iteration. No `ChoiceScreen` is hoisted above the loop or reused. `choice` reliably defaults to index 0 (the safe default: "Skip" for pp-choice; "No passphrase" for fp-choice) at the start of each backup attempt. ✓

### I-1 — `"fmt"` import

**Verified GREEN.**

`"fmt"` is confirmed absent from `gui.go`'s import block (grep found zero matches). The only new import needed is `"fmt"` for `fmt.Sprintf("%.8X", mfp)` / `fmt.Sprintf("%.8X", passFp)`. The format string `%.8X` matches the existing `backup/backup.go:120,182` usage exactly. No other new stdlib imports are required by this slice (`strings`, `image`, `errors`, `chaincfg` are all already imported). ✓

### I-3 — Compilable `backupWalletFlow`

**Verified GREEN.**

Every API cross-checked against actual source:

- `ChoiceScreen.Choose(ctx *Context, th *Colors) (int, bool)` — `gui.go:1296` ✓
- `passphraseFlow(ctx *Context, th *Colors) (string, bool)` — new function defined in spec §4.2 ✓
- `masterFingerprintFor(m bip39.Mnemonic, net *chaincfg.Params, password string) (uint32, error)` — new signature (S1 threading) ✓
- `engraveSeed(params engrave.Params, m bip39.Mnemonic, mfp uint32) (Plate, error)` — new signature (S1 threading) ✓
- `NewEngraveScreen(ctx *Context, plate Plate) *EngraveScreen` — `gui.go:2309` ✓
- `.Engrave(ctx *Context, th *Colors) bool` — `gui.go:2321` ✓
- `&engraveTheme` is `*Colors` (`engraveTheme` declared as `Colors` in `theme.go:39`) ✓
- `ctx.Platform.EngraverParams()` — established pattern at `gui.go:1894` ✓

Variable shadowing analysis: the `sel, ok :=` in the `if` init scope (ppChoice) and the `sel, ok :=` inside the passphrase branch (fpChoice) are in separate scopes; the inner `sel` is a new declaration that correctly governs `if sel == 1 { mfp = passFp }`. No unreachable code, no type mismatch, no unintended shadowing. ✓

### I-4 / M-7 — Back-semantics coherence and non-regression

**Verified GREEN.**

Control flow analysis of the `if sel, ok := ppChoice.Choose(ctx, th); ok && sel == 1 { … }` gate:

| Event | Choose returns | `ok && sel==1` | Falls through to `engraveSeed`? | Re-Confirm? |
|-------|---------------|----------------|---------------------------------|-------------|
| Skip (index 0 selected, Button3) | `(0, true)` | false | YES — bare `mfp` | No |
| Back (Button1) | `(0, false)` | false | YES — bare `mfp` | No |
| Add passphrase (index 1, Button3) | `(1, true)` | true | enters branch | — |

Inside the passphrase branch, Back from `passphraseFlow` returns `("", false)`: `ok && pass != ""` is false, falls through to `engraveSeed` (bare `mfp`). Empty passphrase confirmed: `ok && pass != ""` is false → skip degenerate choice, use bare `mfp`. Back from `fpChoice.Choose` → `!ok` → `continue` outer loop → re-Confirm. No infinite loop (each `Confirm` call loops until user action). Exactly one path to `engraveSeed` per outer-loop iteration (not reachable from both branches). Regression-free: the no-passphrase path is byte-identical to today's behavior — same bare mfp, same `engraveSeed` call, same engrave path. ✓

### M-2 — `runUI` existence

**Verified GREEN.**

`runUI` is defined at `gui_test.go:466`:
```go
func runUI(ctx *Context, ui func()) (frame func() (string, bool), close func()) {
```
R0's "no runUI" finding was indeed a reviewer error. The function exists and is the established harness for all flow tests. ✓

---

## Full Fresh Read — Regression and Completeness

**Threading (S1) — complete and correct:**

All call sites of `deriveMasterKey` and `masterFingerprintFor` accounted for:
- `gui.go:187` — definition — gains `password string` param ✓
- `gui.go:483` — `masterFingerprintFor` calls `deriveMasterKey(m, network)` → becomes `deriveMasterKey(m, network, password)` ✓
- `gui.go:2071` — `Confirm` validity check — `deriveMasterKey(mnemonic, &chaincfg.MainNetParams)` → `deriveMasterKey(mnemonic, &chaincfg.MainNetParams, "")`. Validates WORDS only; passphrase irrelevant here. ✓
- `gui.go:455` — `engraveSeed` calls `masterFingerprintFor` — this call is ELIMINATED; `engraveSeed` now receives `mfp uint32` directly ✓
- `gui.go:1894` — `backupWalletFlow` calls `engraveSeed(params, mnemonic)` → `engraveSeed(params, mnemonic, mfp)` ✓

No other callers of these functions exist in `gui/` or its tests.

**`passphraseFlow` compile-accuracy:**

- `NewPassphraseKeyboard(ctx *Context) *PassphraseKeyboard` — `passphrase_keyboard.go:62` ✓
- `kbd.Update(ctx *Context) bool` — `passphrase_keyboard.go:200` ✓
- `kbd.Layout(ctx *Context, th *Colors) (op.Op, image.Point)` — `passphrase_keyboard.go:339` ✓ (returns combined readout+grid extent; no separate fragment box needed)
- `kbd.Fragment string` — public field, `passphrase_keyboard.go:48` ✓
- `layout.Rectangle{Max: dims}` — `gui/layout/layout.go:7` ✓
- `screen.CutTop(leadingSize)` — returns `(top Rectangle, bottom Rectangle)` — `layout.go:96` ✓
- `content.CutBottom(8)` — returns `(top Rectangle, bottom Rectangle)` — `layout.go:101` ✓
- `content.S(kbdsz image.Point) image.Point` — `layout.go:55` ✓
- `kbdOp.Offset(image.Point) op.Op` — `op/op.go:182` ✓
- `layoutNavigation(&ctx.B, th, dims, []NavButton{…}...)` — `gui.go:1662`, variadic splat valid Go ✓
- `layoutTitle(ctx, dims.X, th.Text, "Enter Passphrase")` — `gui.go:1633`, `th.Text` is `color.RGBA` (`theme.go:30`) ✓
- `op.Color(&ctx.B, th.Background)` — `op/op.go:66`, `th.Background` is `color.RGBA` ✓
- `op.Layer(kbdOp, nav, title, op.Color(…))` — all `op.Op` ✓
- `ctx.Frame(…)` — established pattern throughout ✓

Pattern matches `inputCodex32Flow` at `gui.go:672` exactly (the canonical flow template). ✓

**Both-fingerprints math:**

`masterFingerprintFor(mnemonic, &chaincfg.MainNetParams, "")` → bare fp via `bip39.MnemonicSeed(m, "")` → PBKDF2-2048 → BIP-32 master → `bip32.Fingerprint(pubkey)` → `uint32`. Same derivation path as today. `masterFingerprintFor(mnemonic, &chaincfg.MainNetParams, pass)` → `bip39.MnemonicSeed(m, pass)` → different seed → different master → different fingerprint. The two are guaranteed distinct when `pass != ""` (BIP-39 PBKDF2 with different salts). The spec correctly gates on `pass != ""` before entering the choice screen. `fmt.Sprintf("%.8X", mfp)` produces the 8-char uppercase hex label matching `backup.go:182`. ✓

**Scope/security:** passphrase lives only in `passphraseFlow`'s `kbd.Fragment` (a `string` on the heap) and the local `pass string` in `backupWalletFlow`. Never written to `backup.Seed`, never in any NFC/engrave path. The `backup.Seed.MasterFingerprint` field carries only the chosen `uint32` — a public wallet identifier, not secret material. Consistent with stated posture. ✓

**Unchanged packages:** `backup/`, `bip39/`, `bip32/`, `gui/passphrase_keyboard.go` — no changes required or implied by any part of the spec. ✓

**Golden test stability:** `backup/backup_test.go` at line 347 calls `bip39.MnemonicSeed(m, "")` directly — completely bypasses `deriveMasterKey`. Unaffected by the signature change. The `TestSeed*` tests build `backup.Seed` directly and do not call any `gui` package functions. ✓

**No new issues introduced by the fold:** The fold added the `showSeedError` helper (DRY refactor, zero behavior change), corrected the `ChoiceScreen` allocation pattern, added the `"fmt"` import note, and wrote a correct compilable `backupWalletFlow`. None of these introduce new defects. The `passphraseFlow` spec was already present in R0 draft and its compile-accuracy is confirmed above.

---

## CRITICAL

None.

## IMPORTANT

None.

## MINOR

**M-4 (label-width, carried forward, still open):** The spec correctly flags this as a QA item. Labels `"No passphrase AABBCCDD"` (23 chars) and `"Passphrase AABBCCDD"` (19 chars) render via `widget.Label` at `ctx.Styles.button` in `ChoiceScreen.Draw`. Whether they fit the 480-px display without truncation depends on the button font metrics — this is unverifiable statically from the spec. The plan to shorten if needed (e.g., "Bare AABBCCDD" / "Pass AABBCCDD") is sound and does not require a spec change. This is a layout QA item for the implementation phase, not a spec defect.

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

Both R0 Criticals (C-1, C-2) and all three R0 Importants (I-1, I-3, I-4/M-7) are correctly and completely folded. M-2 was correctly identified as a reviewer error and confirmed. The spec is compile-accurate, semantically sound, non-regressive, and complete. Implementation may proceed.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 2C/3I/7m | C-1 phantom showSeedErr; C-2 ChoiceScreen state; I-1 fmt; I-3 invalid switch; I-4 back-state. All folded. |
| R1 | **GREEN — 0C/0I** | All folds verified compile-accurate vs `e990f0b`: showSeedError matches the real inline loop (NewErrorScreen/Layout/ss.Draw/op.Layer); fresh ChoiceScreens; fmt-only new import; complete backupWalletFlow valid Go (Choose/passphraseFlow/engraveSeed sigs, no shadow, single engrave path); back-semantics non-regressive (Skip≡Back≡bare, fp-Back→re-Confirm); threading blast radius complete (Confirm keeps ""); passphraseFlow compile-accurate; no golden regen; M-2 confirmed a reviewer error (runUI exists). M-4 label-width = impl QA item. |

GATE PASSED at R1. Proceed to the implementation plan.
