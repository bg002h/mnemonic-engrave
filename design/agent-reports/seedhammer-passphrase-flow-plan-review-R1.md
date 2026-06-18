<!--
Persisted verbatim from the opus-architect R1 gate review of the Slice 3
implementation plan (IMPLEMENTATION_PLAN_seedhammer_passphrase_flow.md @ 841db66).
Reviewer agentId: a47244a9a4fff00af. Round: R1 (re-dispatch after the I-1 fold).
Verdict: GREEN — 0 Critical / 0 Important. Cleared for implementation.
Four carried Minors (M-1..M-4) are non-blocking notes the plan already carries.
The text below is the agent's report exactly as returned; do not edit.
-->

# R1 GATE REVIEW — Slice 3 Implementation Plan

**Reviewer:** opus architect (adversarial, read-only R1 re-dispatch)
**Plan:** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_passphrase_flow.md` (HEAD `841db66`)
**Spec:** `design/SPEC_seedhammer_passphrase_flow.md` (R1 GREEN)
**Base:** fork `main` `e990f0b` (`Merge feat/passphrase-keyboard … (Slice 2)`)
**Date:** 2026-06-18

---

## Verification Results

### 1. I-1 fold confirmed (the one finding from R0)

R0's sole Important: `TestEngraveFingerprintChoiceMapping` uses `fmt.Sprintf` but the plan added `"fmt"` only to `gui/gui.go`, not `gui/gui_test.go` → compile error `undefined: fmt`.

**The fold is correct and complete in TWO places:**

- **Step 3** (plan line 148): *"Add the `"fmt"` import to the import block of **BOTH** `gui/gui.go` **AND** `gui/gui_test.go` (both currently lack `"fmt"`; `gui.go` needs it for the fingerprint-choice labels in `backupWalletFlow`, and `gui_test.go` needs it for `fmt.Sprintf("%.8X", …)` in `TestEngraveFingerprintChoiceMapping` — Step 8)."*
- **File-structure table** (plan lines 21-22): `gui.go` row → *"add `"fmt"` import"*; `gui/*_test.go` row → *"add `"fmt"` import (used by `TestEngraveFingerprintChoiceMapping`)."*

**Live source confirms both files lack `"fmt"` at `e990f0b`:**

- `git show e990f0b:gui/gui.go` import block (lines 4-39) — no `"fmt"`; `git show e990f0b:gui/gui.go | grep -c 'fmt\.'` = `0` (so it is a genuinely new, used import, no conflict).
- `git show e990f0b:gui/gui_test.go` import block (lines 3-26) — imports `"bytes" … "strings" … "unicode"` but **no `"fmt"`**.

The compile error is resolved. ✓

### 2. Type / signature consistency across all tasks — verified against live bytes

| Symbol | Plan claim | Live `e990f0b` evidence | Match |
|---|---|---|---|
| `deriveMasterKey` | gains `password string`; body `MnemonicSeed(m, password)` | `gui.go:187` `(m bip39.Mnemonic, net *chaincfg.Params) (*hdkeychain.ExtendedKey, bool)`; `:188` `seed := bip39.MnemonicSeed(m, "")` | ✓ |
| `masterFingerprintFor` | gains `, password string`; `→(uint32, error)` | `gui.go:482` `(m bip39.Mnemonic, network *chaincfg.Params) (uint32, error)`; calls `deriveMasterKey(m, network)` at `:483` | ✓ |
| `engraveSeed(params, m, mfp uint32)` | new 3-arg sig; delete first 4 lines; `qrc, err := qr.Encode(...)` becomes first `err` decl | `gui.go:454` `(params engrave.Params, m bip39.Mnemonic) (Plate, error)`; `:455-458` are the `mfp, err := masterFingerprintFor…/if err…/return/}`; `:459` `qrc, err := qr.Encode(...)` | ✓ |
| `passphraseFlow(ctx, th)→(string,bool)` | spec §4.2 body | all referenced symbols verified (row below) | ✓ |
| `showSeedError(ctx, th, ss, m, err)` | spec §4.3 helper | `ErrorScreen.Layout(ctx,th,dims)(op.Op,bool)` `gui.go:205`; `SeedScreen.Draw(ctx,th,dims,mnemonic) op.Op` `gui.go:2127`; `NewErrorScreen(err)` `gui.go:384` | ✓ |
| `ChoiceScreen.Choose→(int,bool)` | index 0=bare, 1=pass | `gui.go:1296` `Choose(ctx,th)(int,bool)`; struct `gui.go:1282` has `Title/Lead/Choices []string` | ✓ |

`passphraseFlow` internals all resolve at `e990f0b`: `NewPassphraseKeyboard` (`passphrase_keyboard.go:62`), `Update→bool` (`:200`), `Layout→(op.Op, image.Point)` (`:339`), `Fragment string` (`:48`); `leadingSize` used in the exact `screen.CutTop(leadingSize)` idiom (`gui.go:616/694/801`); `layout.Rectangle.S(sz image.Point) image.Point` (`layout.go:55`), `CutTop`/`CutBottom`; `layoutNavigation(buf, th, dims, ...NavButton)` variadic (`gui.go:1662`); `layoutTitle(ctx, width int, col color.RGBA, title string)` (`gui.go:1633`).

### 3. Caller completeness (threading blast radius)

`git grep -n` across `gui/*.go` at `e990f0b` enumerates **every** caller — exactly matches the plan, nothing missed:

- `deriveMasterKey`: only `gui.go:483` (in `masterFingerprintFor`) + `gui.go:2071` (in `SeedScreen.Confirm`, kept `""` — validates words). ✓
- `masterFingerprintFor`: sole caller `gui.go:455` (in `engraveSeed`; Task 1 passes `""`, Task 2 removes it). ✓
- `engraveSeed`: sole caller `gui.go:1894` (in `backupWalletFlow`; rewritten atomically in Task 2). ✓
- `backupWalletFlow`: unchanged signature; called from `gui.go:1809` — **unaffected by the rewrite**. ✓

### 4. `backupWalletFlow` control flow

Live body at `gui.go:1888-1913` is exactly `for { Confirm→return-false; engraveSeed; inline-error-loop(break/continue); Engrave→return-true }` — **no `qaProgram`/`backupWallet` branch**, confirming the plan's wholesale-replace drops nothing (Step 7's verify-before-replace caveat is satisfied by the actual bytes). The inline error loop (`errScr.Layout` + `ss.Draw` + `op.Layer(d, main)`) is byte-for-byte the `showSeedError` helper body — the factoring is semantics-preserving (inner `break` → helper `return`, caller `continue`s after).

Control-flow properties of the rewrite hold against `ChoiceScreen.Choose` semantics (`gui.go:1296-1346`, which I read): fresh `ChoiceScreen` per iteration → `s.choice` zero-value = 0 = safe default; `Down`+`Button3` → `(1,true)`; `Button1` → `(0,false)`. Therefore:
- **Exactly one engrave path per iteration** (single `engraveSeed` call). ✓
- **Terminates**: `Confirm`→false `return`; `Engrave`→true `return`; all other exits `continue`. ✓
- **Back-semantics**: Skip/Back from `ppChoice` (`!ok` or `sel==0`) → falls through to bare-`mfp` engrave (Skip≡Back≡bare); Back/empty from `passphraseFlow` → bare; Back from `fpChoice` (`!ok`) → `continue` → re-Confirm; passphrase discarded on re-loop. Matches spec §4.3 verbatim. ✓

### 5. Atomicity / build order

Task 1 compiles standalone (all callers pass `""`, `engraveSeed` stays 2-arg). Task 2 is atomic: the `engraveSeed` 3-arg signature change + its sole caller `backupWalletFlow` + `passphraseFlow` + `showSeedError` + `"fmt"` import land in one commit. No intermediate non-compiling state; Go has no intra-package forward-reference issue. ✓

### 6. Golden-guard claim (no regen)

`backup.Seed.MasterFingerprint` is `uint32` (`backup.go:21,28`) — structurally unchanged. `frontSideSeed` (`backup.go:181-188`) renders `fmt.Sprintf("%.8X", plate.MasterFingerprint)` when `!=0` — **render path untouched**. `seedqr.QR(m bip39.Mnemonic) []byte` (`seedqr.go:24`) is words-only (formats word indices `%04d`), passphrase-independent → engraved SeedQR + words byte-identical regardless. The no-passphrase path computes the bare `mfp` (passphrase `""`, identical seed to today) and engraves identically → `backup` golden `TestSeed*` (which build `backup.Seed` directly, not via gui funcs) stay green. **No golden regen needed.** ✓ The plan's `%.8X` label format matches `backup.go:182` exactly.

### 7. Passphrase never reaches `backup.Seed`/engrave/NFC

The passphrase flows only into `masterFingerprintFor(mnemonic, …, pass)` → ephemeral key → `uint32` fingerprint. The new `engraveSeed` takes `mfp uint32` (not a passphrase); `backup.Seed` carries only `MasterFingerprint uint32`. `bip39.MnemonicSeed` salt is `"mnemonic"+password` (`bip39.go:226`, PBKDF2-2048/64/SHA512, pure, no persistence) — empty vs non-empty → distinct seeds → distinct fingerprints, so the `bare != pass` test assertions are protocol-sound. ✓

### 8. Test-harness symbols + vectors

`runUI` (`gui_test.go:466`), `click(r *EventRouter, bs ...Button)` (`gui_test.go:42`), `runes(r *EventRouter, str string)` (`gui_test.go:68`), `newPlatform` (`:451`), `descriptorTheme`, button constants `Up/Down/Center/Button1/Button3` (`event.go:23-30`) — all present; the plan's `click(&ctx.Router, …)`/`runes(&ctx.Router, …)` arg form matches. `emptyBIP39Mnemonic(nwords int)` exists at `gui.go:511` (so the dead-loop draft compiles). The "abandon…about" vector is a valid 12-word BIP-39 mnemonic (`bip39_test.go:114`). `engraveObjectFlow` direct-call test pattern (`codex32_polish_test.go:197`) confirms the plan's "mirrors `TestEngraveCodex32BackoutNotUnknown`" claim. `bip39.ClosestWord(word)(Word,bool)` (`bip39.go:95`) supports the implementer-authored `bip39FromWords`.

### 9. Drift check — fold introduced no inconsistency

The fold touched only Step 3 (line 148) and the file-structure table (lines 21-22). Cross-checking: the Self-Review (line 309 "`"fmt"` import added"), the §4.3 spec reference, and the final-review focus list (line 300 "`fmt` is the only new import and is used") are all consistent with adding `"fmt"` to both files. No type/signature/line-number claim elsewhere in the plan was altered or contradicted by the fold. No new drift.

---

## Findings

### CRITICAL
None.

### IMPORTANT
None. (R0's I-1 is folded and verified resolved against live `e990f0b` bytes in both `gui.go` and `gui_test.go`.)

### MINOR
- **M-1 (carried, non-blocking):** Task 1 Step 1's `TestMasterFingerprintPassphrase` draft opens with a dead `emptyBIP39Mnemonic(12)` + discard loop; the plan's own NOTE (line 67) instructs removing it for the direct builder. Compiles either way; implementer must heed the NOTE.
- **M-2 (carried, non-blocking):** `bip39FromWords` is not shown as a body and does not exist anywhere in `gui/*_test.go` at `e990f0b` (`git grep` = NONE). The description (split + `bip39.ClosestWord`, exact) is unambiguous and `strings` is already imported; implementer authors it once and reuses in Task 2.
- **M-3 (carried, non-blocking):** Task 2 Step 1 shows a `runUI`/package-var `TestPassphraseFlow` (references undeclared `passphraseFlowResult`/`passphraseFlowOK`) before the NOTE (line 144) discards it in favor of the direct-call form. Implementer must use only the direct-call form.
- **M-4 (carried from spec, layout QA, non-blocking):** fingerprint-choice labels ("No passphrase " + 8-hex ≈ 23 chars) fit-on-480px is a render QA item deferred to the final review (plan line 300), not a correctness blocker.

These four are exactly the non-blocking notes R0 already dispositioned; each is explicitly covered by an in-plan NOTE. None gates implementation.

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

The single R0 Important (I-1, missing `"fmt"` in `gui_test.go`) is correctly folded into BOTH Step 3 and the file-structure table, and verified against live `e990f0b` bytes (neither `gui.go` nor `gui_test.go` imports `"fmt"` today; the fold adds it to both). The fold introduced no drift. Every cited file:line, symbol, signature, caller set, control-flow property, the golden-guard/no-regen claim, and the passphrase-never-engraved invariant are confirmed against the authoritative live fork source and BIP-39 seed/fingerprint semantics. The plan is cleared for implementation; the four remaining Minors are non-blocking notes the plan already carries.
