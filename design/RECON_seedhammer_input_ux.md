# Recon — SeedHammer II on-device input UX (seed words, passphrase, CODEX32)

**Date:** 2026-06-17
**Repo under investigation:** the SeedHammer II firmware fork `bg002h/seedhammer`
(`/scratch/code/shibboleth/seedhammer`), Go/TinyGo, RP2350, 480×320 touchscreen +
3 side buttons + D-pad, single-touch capacitive (FT6x36).
**Motivation:** upstream PR #34 (re-enable on-device CODEX32 seed entry) was closed
with the reason *"the codex32 option is disabled for a reason — the UI flow is not
sufficiently polished for general use."* This recon investigates the on-device
secret-input UX and proposes how to make it polished, focused on **seed-word entry**
and **passphrase-on-seed** (the explicit asks), with **CODEX32** as the third pillar
since all three share one keyboard widget.
**Method:** four parallel read-only `code-explorer` agents (BIP-39 word flow; CODEX32
flow; passphrase gap; shared widget + peer-device survey). This doc consolidates them.

---

## 1. Headline

**This is a UX problem, not a crypto problem.** The cryptographic plumbing is already
correct and present; the GUI simply never exercises it. Almost all the work lives in
one file, `gui/gui.go` (~2,600 lines).

- `bip39.MnemonicSeed(m, password)` (`bip39/bip39.go:189`) fully implements the BIP-39
  passphrase derivation. It is only ever called with `""` (`gui/gui.go:188`).
- `codex32.Interpolate(...)` (`codex32/codex32.go:188`) fully implements k-of-n share
  reconstruction. It is **never called from the GUI**.

Three input flows exist at very different polish levels:

| Flow | Function | Status |
|---|---|---|
| BIP-39 word entry | `inputWordsFlow` (`gui/gui.go:539`) | enabled; the polish *baseline* (but has gaps) |
| CODEX32 entry | `inputCodex32Flow` (`gui/gui.go:623`) | the #34 subject; least polished; missing its core feature |
| SLIP-39 entry | `inputSLIP39Flow` (`gui/gui.go:684`) | commented out / disabled |
| Passphrase | — | **does not exist** (zero matches for "passphrase") |

All three call the shared `Keyboard` widget (`NewKeyboard`, `gui/gui.go:790`), which
takes an alphabet string with `\n` row separators and auto-appends backspace.

---

## 2. Cross-cutting "not polished" themes

These recur across flows and are the substance of the maintainer's objection:

1. **No live entry feedback** — no "word N of 24", no match-count, no candidate list.
   BIP-39 computes `nvalid` (`updateValidBIP39Keys:869`) but hides it; CODEX32 shows
   nothing until the whole 48–127-char string validates.
2. **Inconsistent confirm button** — accept-word is on **Button2 (middle)**
   (`inputWordsFlow:543,606`), while every other screen uses **Button3 (bottom)** for
   the primary action. Invites accidental Back presses.
3. **No verification anchor** — the master fingerprint (`masterFingerprintFor:482`) is
   computed and engraved but never shown to the user; CODEX32 engraves with no review.
4. **Weak error recovery** — can't step back one word mid-flow (Back exits the whole
   screen, `inputWordsFlow:558`); CODEX32 is backspace-only across a 127-char string.
5. **No checksum assistance** — the last word is fully constrained (exactly 1 valid word
   for 24-word seeds; 128 for 12-word), yet any word can be typed and the error only
   surfaces at confirm time (`SeedScreen.Confirm:1973`).
6. **Infra debt** — small touch targets (~3.9 mm vs ~6 mm recommended); the three theme
   alphas `overlayMask`/`activeMask`/`inactiveMask` are all identically `0x55`
   (`theme.go:65-67`, no visual hierarchy); `fadeClip` is a no-op (`gui.go:521`);
   nav/progress code is copy-pasted across all three flows.

---

## 3. Pillar 1 — Seed-word entry polish (BIP-39)

| # | Suggestion | Cx | Anchor |
|---|-----------|----|--------|
| 1 | "Word N of 24" progress in the title (data already in scope) | S | `inputWordsFlow:612` |
| 2 | Show match count / candidate list (Krux/SeedSigner) — surface the already-computed `nvalid` | S–M | `updateValidBIP39Keys:869` |
| 3 | Tap the predicted word to accept; move accept to **Button3** for consistency | S | `inputWordsFlow:543,606` |
| 4 | Last-word checksum shortlist — present only valid final words (Coldcard); eliminates the "invalid seed" error | M | new `bip39` helper + `inputWordsFlow` |
| 5 | In-flow "back one word" edit instead of exiting the screen | M | `inputWordsFlow:558` |
| 6 | Show master fingerprint on the confirm screen ("verify this matches your wallet") — Jade/Coldcard | M | `SeedScreen.Confirm`, `masterFingerprintFor:482` |

---

## 4. Pillar 2 — Passphrase on seed (net-new)

The plumbing is ready; this is a clean additive flow injected **after** the seed is
confirmed (after `SeedScreen.Confirm` returns true, ~`gui.go:1958`).

Injection points: thread a `passphrase string` through `deriveMasterKey` (`gui.go:187`)
and `masterFingerprintFor` (`gui.go:482`); the keyboard force-uppercases in `rune()`
(`gui.go:1030`) and has no symbols, so a full-ASCII keyboard needs a shift layer +
symbol page.

| # | Suggestion | Cx | Note |
|---|-----------|----|------|
| 1 | Thread `passphrase` through `deriveMasterKey`/`masterFingerprintFor` (drop the hardcoded `""`) | S | unblocks everything |
| 2 | "Skip / Add passphrase" preamble with a security warning ("not engraved — store separately") | S | reuse `ChoiceScreen` |
| 3 | Full-ASCII keyboard — shift layer + symbols page (add `preserveCase` + page switch to `Keyboard`) | M | |
| 4 | Masked display + timed reveal + character count | M–L | reuse `ConfirmDelay` for reveal |
| 5 | Double-entry confirmation (re-enter, compare, error on mismatch) | M | new `confirmPassphraseFlow` |
| 6 | Fingerprint verification screen — passphrase changes the fingerprint, so show it to cross-check | L | |
| — | **Invariant: passphrase is NEVER engraved** (not on plate, not in SeedQR, never over NFC) | — | matches the constellation secret/public split |

Optional (Jade pattern): a **word-list passphrase mode** reusing the BIP-39 autocomplete
widget for memorable passphrases.

---

## 5. Pillar 3 — CODEX32 (the #34 subject)

| # | Suggestion | Cx | Severity |
|---|-----------|----|----------|
| 1 | **Multi-share k-of-n entry + `Interpolate`** — currently unimplemented; the GUI never calls `codex32.Interpolate`, so a k-of-n share is **engraved verbatim instead of reconstructing the seed** (actively wrong for recovery) | L | Must-fix |
| 2 | Parsed-field confirmation screen (id / threshold / share index) before engraving | S | Must-fix |
| 3 | Surface `codex32.New` error class (bad checksum vs length vs char) instead of just hiding the OK button | S | High |
| 4 | Per-position key filtering + char counter + segmented display to match BIP-39 polish | M | High |

Note: the CODEX32 keyboard alphabet (`gui.go:624`) is character-complete for bech32 but
non-standard (drops `b/i/o`, reorders) — restore QWERTY + dim non-bech32 keys, or order
by the bech32 alphabet.

---

## 6. Shared-infra improvements (benefit all flows)

- Extract a shared `inputFlowNav(back, ok, condition)` + `layoutWordProgress(...)` helper
  (removes ~15 duplicated lines per flow; fixes inconsistent `widestWord` constants). (M)
- Touch-target sizing floor + give `activeMask`/`inactiveMask` distinct values for a real
  visual hierarchy. (S)
- Haptic feedback on keystroke via a new `Platform.Haptic()` method (Trezor Safe 5
  pattern); hardware work, breaking interface change. (L)

---

## 7. Peer-device patterns worth adopting

- **Krux / SeedSigner:** progressive letter disabling + visible match count / candidate
  list.
- **Coldcard Mk4/Q, Krux Tinyseed:** last-word presented as a shortlist of valid
  checksum words.
- **Jade, Coldcard:** wallet fingerprint shown after seed/passphrase entry for
  verification.
- **Trezor Safe 5 / Model T:** tap the auto-suggested word to accept; haptic confirm.
- **Jade:** passphrase-as-BIP-39-word-list mode.
- **SeedSigner / Jade:** QR (SeedQR) as an alternative seed-input path.

---

## 8. Proposed first slice (recommended scope)

High-visibility, mostly S/M, directly answers "input UX isn't polished":

- **Seed words:** progress indicator + match count + accept-on-primary/tap-to-accept +
  last-word checksum shortlist (Pillar 1 #1–4).
- **Passphrase MVP:** thread-through + "skip/add" preamble + full-ASCII masked keyboard +
  double-entry confirm (Pillar 2 #1–5).

**CODEX32 multi-share (Pillar 3 #1)** is the single heaviest item and the most important
*correctness* fix — treat as its own follow-on so it gets dedicated design attention.

---

## 9. Upstream-contribution path

1. The fork is public ⇒ upstream can already fetch/cherry-pick any branch with no action
   from us.
2. PRs are the formal channel (#35 open; #34 closed but reopenable).
3. This work is the *strong* contribution: it's **generic** (helps every BIP-39 user;
   adds broadly-wanted passphrase support) and directly addresses the #34 polish
   objection — unlike the niche md1/mk1 support. Build on the fork → make it genuinely
   polished → open a clean, focused, signed+DCO PR rebased on current `upstream/main`.
   Small one-feature PRs review better than a mega-PR.

---

## 10. Process note

Per project standard ([[iterative-architect-review-standard]]): any implementation goes
**brainstorm → spec → plan → architect R0 gate (converge to 0C/0I) → subagent-driven
implementation**. This recon feeds the brainstorm; it is not a green light to code.

## 11. Key files

- `gui/gui.go` — all input flows, `Keyboard`/`NewKeyboard`, `completeBIP39Word`,
  `updateValidBIP39Keys`, `inputWordsFlow`/`inputCodex32Flow`/`inputSLIP39Flow`,
  `deriveMasterKey`, `masterFingerprintFor`, `engraveSeed`, `SeedScreen.Confirm`,
  `newInputFlow`, `layoutNavigation`, `ConfirmWarningScreen`
- `bip39/bip39.go` — `MnemonicSeed(m, password)`, `ClosestWord`, `LabelFor`,
  `Mnemonic.Valid`, `ChecksumWord`
- `codex32/codex32.go` — `New`, `Interpolate`, `Split`, `NewSeed`, `Alphabet`
- `codex32/checksum.go` — checksum engine + case enforcement
- `gui/theme.go` — `Styles`, alphas (`inactiveMask=0x55` etc.), fonts
- `gui/widget.go`, `gui/event.go`, `gui/op/op.go`, `gui/layout/layout.go` — widget/event/render/layout primitives
- `cmd/controller/platform_sh2.go` — 480×320 display, touch wiring, button mapping
- `gui/codex32_input_test.go` — the only CODEX32 GUI test (notably thin)
