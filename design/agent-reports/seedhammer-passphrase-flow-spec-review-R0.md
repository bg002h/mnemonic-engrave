# Slice 3: passphrase-flow — SPEC R0 GATE REVIEW — R0

- **Stage:** mandatory spec R0 gate (0C/0I before any code).
- **Spec reviewed:** `design/SPEC_seedhammer_passphrase_flow.md` (committed `34b3eac`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `af720e759c5b15c66`), read-only vs fork `e990f0b`.
- **Outcome:** **RED — 2 Critical (C-1,C-2) + 3 Important (I-1,I-3,I-4) + minors.** All folded; re-dispatched R1.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

## R0 Spec Review — SPEC_seedhammer_passphrase_flow.md (Slice 3)

**Reviewer:** opus architect (adversarial, read-only)
**Tree verified against:** fork `main` @ `e990f0b`, `/scratch/code/shibboleth/seedhammer`
**Date:** 2026-06-18

---

### CRITICAL findings

**C-1 — `showSeedErr` does not exist; the spec invokes a phantom helper**

Spec §4.3, line 107:
```go
showSeedErr(ctx, th, ss, mnemonic, err) // existing ErrorScreen pattern
```

`showSeedErr` does not exist anywhere in the codebase. Grep over `gui/**/*.go` finds no such symbol. The real `backupWalletFlow` error path (gui.go:1896-1905) is an inline anonymous loop, not a named helper:
```go
errScr := NewErrorScreen(err)
for !ctx.Done {
    dims := ctx.Platform.DisplaySize()
    d, dismissed := errScr.Layout(ctx, th, dims)
    if dismissed { break }
    main := ss.Draw(ctx, th, dims, mnemonic)
    ctx.Frame(op.Layer(d, main))
}
```
The only named helper patterns are `showErr` (a closure defined locally within `SeedScreen.Confirm` at gui.go:2044 and within `descriptorFlow` at gui.go:2218) and `showError` (in `slip39_polish.go`, different signature). None match the spec's `showSeedErr(ctx, th, ss, mnemonic, err)` signature.

The spec's control-flow skeleton is thus non-compiling as written. The implementer must either inline the error loop (copy the existing `backupWalletFlow` pattern) or define a new helper. The spec must specify which. The signature shown also passes `ss *SeedScreen` for background rendering — that matches the existing inline pattern — but none of this is spelled out. This is a blocking compile error if taken literally.

**Fix:** Replace the `showSeedErr` call with the explicit inline pattern (copying gui.go:1895-1906), or define and spec a concrete `showSeedErr` helper with its exact signature and body. Mark it as new code, not "existing."

---

**C-2 — `ChoiceScreen.s.choice` state bleeds across re-entries; the "Add passphrase?" choice defaults to index 1 on re-loop**

`ChoiceScreen` is a struct with a `choice int` field (gui.go:1287). The spec allocates a `&ChoiceScreen{...}` at the top of each `backupWalletFlow` iteration, which is correct for the first call. However the spec's skeleton shows only one `ChoiceScreen` literal for the "passphrase?" step (§4.3, line 111). When the user: (a) picks "Add passphrase", (b) backs out of `passphraseFlow`, (c) the spec says "re-show the 'Passphrase' choice (or loop to Confirm)" — the spec is ambiguous here but one natural reading is to re-call `Choose` on the SAME `ChoiceScreen` instance. Since `s.choice` was left at index 1 (the user's last pick), the re-presented "Add passphrase?" screen will default-highlight "Add passphrase" rather than "Skip". This is not necessarily wrong UX, but the spec says nothing about it.

More critically, the fingerprint-choice `ChoiceScreen` is also re-entrant if the user backs out at the engrave step and loops again. If the `ChoiceScreen` structs are declared once outside the inner loops, `s.choice` persists. If declared fresh each iteration, they reset. The spec skeleton at §4.3 does not make the allocation site unambiguous (it shows a single expression, not a `var` declaration with explicit scope).

**Fix:** Spec must explicitly state that both `ChoiceScreen` instances are allocated fresh per outer loop iteration (zero-initialised `choice` = index 0 = safe default each time), or justify persisting the prior selection. The fingerprint choice screen's `s.choice` persistence is especially risky — after a back-from-engrave loop, index 1 (passphrase fp) would be pre-selected.

---

### IMPORTANT findings

**I-1 — The `"fmt"` import addition is mentioned but not committed to; the spec leaves it as a parenthetical**

Spec §4.3 line 131: `"fmt" IS already imported in gui.go? (Recon: gui.go imports "strings" but NOT "fmt". So the %.8X formatting needs "fmt" — add it to gui.go's imports, OR use strconv/the existing engrave.String path. The plan picks one; cleanest is to add "fmt".)`

Confirmed from source: gui.go imports list (lines 4-41) does NOT include `"fmt"`. The spec ends with "The plan picks one" without actually picking. This is an open decision that will cause a compile error if the implementer writes `fmt.Sprintf("%.8X", mfp)` without noticing the missing import, or wastes time picking a path.

Confirmed: `backup/backup.go` imports `"fmt"` and uses `fmt.Sprintf("%.8X", ...)` for the same fingerprint (backup.go:182). The correct and cleanest fix is to add `"fmt"` to gui.go's import block. The spec must lock this in as a stated requirement, not a parenthetical.

**Fix:** State explicitly: "Add `\"fmt\"` to gui.go imports. Use `fmt.Sprintf(\"%.8X\", mfp)` for consistency with backup.go:182."

---

**I-2 — `passphraseFlow` hardwires `&chaincfg.MainNetParams` implicitly via `masterFingerprintFor` but the spec does not make the network parameter explicit at the call sites in `backupWalletFlow`**

The spec at §4.3 line 105 shows:
```go
mfp, err := masterFingerprintFor(mnemonic, &chaincfg.MainNetParams, "") // bare
```

This is fine. But `engraveSeed` today (gui.go:455) also hardwires `&chaincfg.MainNetParams`:
```go
mfp, err := masterFingerprintFor(m, &chaincfg.MainNetParams)
```

After the refactor, `engraveSeed(params, m, mfp uint32)` receives the already-chosen `mfp` and no longer calls `masterFingerprintFor`. The network is therefore hardwired in `backupWalletFlow` for BOTH fingerprint calls. This is consistent with today's behavior and with the rest of the codebase (no multi-network support), and the spec correctly shows `&chaincfg.MainNetParams` in both calls.

However, the spec does not explicitly say that `chaincfg` is already imported in gui.go. Verify: yes, gui.go line 20 imports `"github.com/btcsuite/btcd/chaincfg/v2"`. Not a blocker but worth noting.

**Actually: no issue.** This is not a blocking gap. Downgrading this observation — see MINOR below.

---

**I-3 — The `backupWalletFlow` control structure skeleton in §4.3 is incomplete and contradictory**

The spec provides a partial Go skeleton (lines 99-122) and then a separate prose description (lines 124-129). The skeleton has an incomplete `switch` statement that is not valid Go:

```go
switch (&ChoiceScreen{...}).Choose(ctx, th); ... {
// index 0 (Skip) or Back → keep the bare mfp.
// index 1 (Add passphrase) →
}
```

`switch` over `(int, bool)` does not work in Go — `Choose` returns two values and a `switch` expression must be a single value. This is a Go compile error. The correct pattern used elsewhere for `ChoiceScreen` (e.g. gui.go:2246) is:
```go
choice, ok := cs.Choose(ctx, th)
if ok { ... }
```

The spec comments on "the plan nails the exact control structure" (line 124) but then only provides the prose description (lines 124-129) as the real specification. The skeleton is misleading because it is not compilable Go. An implementer trusting the skeleton literally will write broken code.

**Fix:** Remove the broken `switch` skeleton or replace it with compilable Go. The prose description (steps 1-5) is clear and should be the authoritative control structure. Add an explicit note that the skeleton is schematic only. Better: write the complete compilable `backupWalletFlow` body in the spec.

---

**I-4 — Back-navigation from the fingerprint-choice screen is under-specified: re-shows `passphraseFlow` but `PassphraseKeyboard` state is gone**

Spec §4.3, step 4: "If `Back` → re-show passphrase entry."

When the user backs out of the 2-row fingerprint choice, the spec says to re-show `passphraseFlow`. But `passphraseFlow` calls `NewPassphraseKeyboard(ctx)` at the start, which creates a fresh keyboard (Fragment=""). The previously entered passphrase is lost. This is likely correct behavior (don't pre-fill the passphrase for security and simplicity), but the spec is silent on it — an implementer might try to pass the old `pass` string back in to pre-populate, which `PassphraseKeyboard` has no API for. The spec should state explicitly: "on Back from the fingerprint choice, `passphraseFlow` is called fresh (prior passphrase entry is not preserved — the user re-types)."

This is important because if an implementer attempts to preserve the passphrase in a local variable and re-populate the keyboard (no API exists for this), they will either fail to compile or introduce a confusing workaround.

**Fix:** Add a sentence: "The `pass` string is discarded on Back from the fingerprint-choice screen; `passphraseFlow` is re-entered fresh (`NewPassphraseKeyboard` zeros Fragment). The user must re-type."

---

### MINOR findings

**M-1 — `chaincfg` import already present in gui.go; no action needed**

gui.go:20: `"github.com/btcsuite/btcd/chaincfg/v2"` already imported. The I-2 concern above is not a real gap. No action needed.

**M-2 — Test section (§6) refers to `runUI` but no such helper is described**

§6, S2: "via `runUI` — drive `NewPassphraseKeyboard`...". Grep shows no `runUI` in the test files; the existing pattern for driving flows is direct: instantiate the context, push events via `runes`/`click`/`press`, call the flow function. The existing `gui_test.go` tests (e.g. `TestWordKeyboardScreen`) drive flows directly without a `runUI` wrapper. This is a stale reference (may have been copied from another spec) that will confuse the TDD implementer.

**Fix:** Remove `runUI`; describe the test pattern as "drive via `runes`/`click` helper functions as in `TestWordKeyboardScreen`."

**M-3 — Spec §3 says `SeedScreen.Confirm` validity check is at `gui.go:2071` — actual line is 2071, confirmed. But the line cited for `return true` is "`:2078`" — actual is gui.go:2078. Both accurate.**

No action needed.

**M-4 — The fingerprint label width concern: `"No passphrase  XXXXXXXX"` (two spaces) may overflow `ChoiceScreen` if the fingerprint is the max 8 hex digits**

The spec proposes labels like `"No passphrase  " + hex(mfp)` (§4.3 line 128). `ChoiceScreen.Draw` renders these using `widget.Label` with `ctx.Styles.button` style and no explicit width limit (gui.go:1367). The device display is fixed-size. At the firmware's font sizes, "No passphrase  XXXXXXXX" (26 chars) may or may not overflow — impossible to verify without measuring. This is a minor layout risk, not a correctness bug. The spec should note it as a layout concern to be validated in the TDD phase by checking that `ChoiceScreen` renders the full label within screen width, and give the implementer permission to shorten labels (e.g. "No passphrase" + newline + hex, or just the hex with a prefix glyph).

**M-5 — `backup_test.go:genSeed` and `gui_test.go:fillDescriptor` call `bip39.MnemonicSeed(m, "")` directly (not via `deriveMasterKey`) — they are immune to the `deriveMasterKey` signature change and need no update. Confirmed.**

No action needed.

**M-6 — Spec asserts `ChoiceScreen` is at `gui.go:1282` — actual: `type ChoiceScreen struct` at gui.go:1282, `Choose` at gui.go:1296. Both correct.**

No action needed.

**M-7 — The spec says "re-loop/re-Confirm" on Back from the "Add passphrase?" choice (§4.3 step 2, `!ok`)**

The spec's step 2 says Back from the "Passphrase?" choice → loop (re-Confirm). This means the user must re-confirm ALL their words again just because they pressed Back on the optional "add passphrase?" screen. This is a UX regression compared to the non-passphrase path. The existing flow loops only on engrave-not-completed (the user backed out of the engraver). Adding a "re-Confirm all words" penalty for pressing Back on an optional intermediate screen is likely unintended. The correct behavior is probably: Back from "Add passphrase?" → jump back to the `ChoiceScreen` (i.e. loop the choice, not re-Confirm). 

This is a UX/correctness concern but since the spec explicitly says "loop (re-Confirm)" and the R0 process gates code, not UX intent, I flag it as MINOR with a strong recommendation to reconsider. If it is intentional, state it explicitly.

---

### Threading correctness summary (Questions 1-2, verified)

- `deriveMasterKey` callers: gui.go:483 (via `masterFingerprintFor`), gui.go:2071 (Confirm validity check). Both confirmed. The `Confirm` validity check MUST keep `password=""` — confirmed correct (it validates the mnemonic words, not a passphrased wallet). No test callers call `deriveMasterKey` directly.
- `masterFingerprintFor` caller: gui.go:455 only. Confirmed sole caller.
- `engraveSeed` caller: gui.go:1894 only. Confirmed sole caller.
- Test helpers `backup_test.go:347` and `gui_test.go:299` call `bip39.MnemonicSeed` directly with `""` — they bypass `deriveMasterKey` entirely and are unaffected by the signature change. Confirmed.
- `seedqr.QR(m)` (gui.go:459): takes only `m bip39.Mnemonic`, no password. Words are passphrase-independent. Confirmed.
- `bip39.MnemonicSeed` salt confirmed as `"mnemonic"+password` (bip39.go:225). Empty vs non-empty → different 64-byte seed → different fingerprint. Confirmed.

### Security and scope (Questions 6-7, verified)

- `backup.Seed.MasterFingerprint uint32` field: unchanged. `frontSideSeed` render: unchanged (backup.go:181-188). Golden tests: unaffected. `backup`, `bip39`, `bip32`, `PassphraseKeyboard` packages: all untouched by this slice. Confirmed.
- Passphrase in RAM only. `MnemonicSeed` pure. No NFC path. No `SeedString` changes. Confirmed.

---

### Verdict

**RED — 2 Critical / 4 Important open**

- **C-1:** `showSeedErr` phantom helper → compile error
- **C-2:** `ChoiceScreen.s.choice` state persistence across re-loops → silent UX/correctness bug (pre-selected "Add passphrase" / passphrase-fp on re-entry)
- **I-1:** `"fmt"` import decision left open → compile error risk
- **I-2:** (downgraded to M-1, no action)
- **I-3:** Broken `switch (int,bool)` skeleton → compile error if followed literally
- **I-4:** Back-from-fingerprint-choice state loss unspecified → implementation ambiguity

No code may be written until C-1, C-2 are resolved and I-1, I-3, I-4 are addressed. Fold findings into the spec, persist this review verbatim to `design/agent-reports/`, and re-dispatch for R0 round 2.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 2C/3I/7m | **C-1** phantom `showSeedErr` → folded: define a real `showSeedError` helper factoring the existing inline error loop (also DRYs the engrave-error path). **C-2** reused `ChoiceScreen` `choice` persists across re-loops → folded: allocate both ChoiceScreens FRESH each iteration (default index 0 = safe). **I-1** `fmt` import left open → folded: add `"fmt"`, use `fmt.Sprintf("%.8X")`. **I-3** invalid `switch (int,bool)` skeleton → folded: complete compilable `backupWalletFlow` written. **I-4** back-from-fp-choice state loss → folded: explicit back-semantics (Skip≡Back-at-add-passphrase≡bare; fp-choice Back → re-Confirm, passphrase re-typed). Minors: M-4 label-width QA note; **M-2 was a reviewer ERROR** (runUI DOES exist at gui_test.go:466 — kept); M-7 back-semantics corrected (no needless re-Confirm). Architect confirmed threading blast radius (deriveMasterKey:188 sole injection; Confirm validity check keeps ""; engraveSeed sole caller; seedqr.QR words-only), no golden regen, security/scope clean. |

Re-dispatched R1 after the fold.
