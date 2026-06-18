<!--
Persisted verbatim. opus-architect R0 gate of the SLIP-39 Trezor-routing plan
(@ ced6d71). Reviewer agentId a61fd1df1a71f3eaa. Verdict: NOT GREEN 2C/1I+1m. C1 my dead-ref
cleanup instruction would break the build (holdToConfirm/ConfirmWarningScreen/assets still used
by the high-e gate; only local `ack` is unused). C2 verbatim-arm test asserts the share Title
which is engraving geometry, never rendered. I1 ChoiceScreen choice buttons are single-line
(only Lead wraps) — long labels risk 480px overflow. Design fidelity/signatures/safety sound.
Disposition: all folded. The text below is the agent's report exactly as returned; do not edit.
-->

# R0 GATE REVIEW — SLIP-39 Trezor-routing plan

**Reviewer:** opus architect (adversarial R0 gate, read-only).
**Plan under review:** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_slip39_trezor_routing.md` (commit `ced6d71`).
**Design authority:** `design/agent-reports/seedhammer-slip39-verbatim-hex-design-consult.md` (agent `a89706fc9955e8479`).
**Base verified against:** fork `seedhammer` `main` @ `9db3fd2` (confirmed via `git log`/`git rev-parse` — "Merge feat/slip39-recovery-gui … Cycle D, Phase D2").
**Date:** 2026-06-18.

---

## Verification Results

### Fidelity to the consult — PASS
The consult's bottom line (§Q2/Q4/Q5, lines 40–44, 62–68, 94) is: convert the one-way ack into a **two-way fork**; BIP-39 arm = fingerprint→`backupWalletFlow`; "not mine" arm = `engraveSLIP39Verbatim(scan)` with **NO** BIP-39 fingerprint (line 65: "`confirmSLIP39Fingerprint` … must **not** run on that arm"); **no hex / no `SeedString`** (lines 26, 64, 88). The plan implements exactly this:
- Plan lines 110–116: `if sel == 1 { engraveSLIP39Verbatim(ctx, th, scan); return true }` with the comment "NO BIP-39 fingerprint here."
- Plan lines 118–127: BIP-39 arm keeps `masterFingerprintFor`→`confirmSLIP39Fingerprint`→`backupWalletFlow`.
- Plan lines 17–21, 161: explicitly no hex/`SeedString`.
- Back path: plan lines 106–109 `if !ok { return false }`, and `engraveSLIP39` (`slip39_polish.go:372-374`) does `continue` on `false` → loops to the original confirm, never `scanUnknownFormat`. The shipped `TestSLIP39FingerprintBackRecognized` guard (test:344) confirms this "recognized, no engrave" contract. **Faithful.**

### Compile / signature reality — PASS (with one caveat in §IMPORTANT-2)
- `engraveRecoveredSLIP39` has exactly **one** caller: `slip39_polish.go:372` (grep confirmed; the only other hits are the def at :384 and the doc comment at :380). The plan's caller update (`engraveSLIP39` passes `scan` through, plan Step 3.1) is correct and complete — `scan` is the `engraveSLIP39` parameter (:359) and the recover arm already holds it.
- New signature `func engraveRecoveredSLIP39(ctx *Context, th *Colors, scan slip39words.Share, m bip39.Mnemonic) bool` — both `slip39words.Share` and `bip39.Mnemonic` are already imported (`slip39_polish.go:9,16`). `chaincfg.MainNetParams` already imported (:8) and already used at :398. **Compiles.**
- `ChoiceScreen{Title, Lead, Choices}` is the real struct (`gui.go:1323-1329`); `Choose(ctx, th) (int, bool)` is real (`gui.go:1337`). `sel == 1` → second choice (verbatim) — confirmed: `Choose` returns `s.choice` which is the 0-based index into `Choices`. **Correct.**
- The `showError`/`masterFingerprintFor`/`confirmSLIP39Fingerprint`/`backupWalletFlow` calls in the BIP-39 arm are byte-identical to the shipped body (:398-406). **Correct.**

### Safety — PASS
- Verbatim arm engraves the **first** share (`scan`) — the same value `engraveSLIP39` already passes to `engraveSLIP39Verbatim` on the lone-Engrave path (:365), and `engraveSLIP39Verbatim` (:453) renders that share's own words/identifier. No BIP-39 fingerprint is shown on this arm (the `confirmSLIP39Fingerprint` call is only in the `sel != 1` branch). **Matches consult line 65.**
- BIP-39 arm retains the records cross-check (`confirmSLIP39Fingerprint`) before `backupWalletFlow`. A recovered seed cannot reach `backupWalletFlow` without passing the fork **and** the fingerprint check (the only path into `backupWalletFlow` from recovery is through this function). **No bypass.**
- Deliberate-action safety: dropping the hold-to-confirm in favor of a `ChoiceScreen` (single Button3 click) does slightly lower the friction vs. the held confirm. But the consult explicitly endorses this (it is UX polish over an already-closed loss-of-funds gate, lines 78, 94), and the BIP-39 arm still has two downstream deliberate gates: the fingerprint confirm **and** `SeedScreen.Confirm` inside `backupWalletFlow` (`gui.go:~1932`). The verbatim arm's deliberate gate is the per-share `id`/index review already shown upstream in `confirmSLIP39Flow` (:83) plus the `EngraveScreen` hold-to-start. **Acceptable; consistent with the design authority.**

### Dead-ref cleanup claim — INCORRECT (see CRITICAL-1)
Grep results contradict the plan's cleanup instruction (plan lines 130-131, 163-164):
- `holdToConfirm` — **still used** at `slip39_polish.go:261` (the high-iteration-exponent slow-recovery gate in `recoverSLIP39Flow`). Must NOT be removed.
- `ConfirmWarningScreen` — **still used** at `:256` (same gate) and `gui.go:2083,216,312`. Must NOT be removed.
- `assets.IconHammer` / the `assets` import — **used pervasively**: `slip39_polish.go:121, 432`, plus `codex32_polish.go:122`, `gui.go:2523`. Must NOT be removed.
Only the local `ack` variable inside `engraveRecoveredSLIP39` becomes unused (and it's being deleted with the block). Removing `holdToConfirm`, `ConfirmWarningScreen`, or the `assets` import as the plan's checklist suggests would **break compilation** of the same file.

### Test soundness — one unrealizable assertion (see CRITICAL-2)
- Harness confirmed: `runUI`/`synctest`/`click`/`press`/`runes`/`driveShare`/`driveRecover`/`pumpUntil`/`uiContains` all exist (`gui_test.go:467`, `event_test.go:42,57,68`, `slip39_polish_test.go:217,228,329`). `ChoiceScreen` navigation via `Down`+`Button3` is the established pattern (`slip39_polish_test.go:122,236`; `Choose` reads `Down` at `gui.go:1368` and acts on `Button3` at :1346).
- **BIP-39 arm test** (`TestEngraveSLIP39RecoverForkBIP39`): realizable. After the fork ChoiceScreen, default index is 0 (`s.choice` zero-value) = the BIP-39 choice, so a single `Button3` selects it → "Recovered Fingerprint"/`%.8X` frame (`confirmSLIP39Fingerprint`, render text `gui.go:434`, value `BDDBDA4F` per shipped test:389) → `backupWalletFlow` ("GIFT" words). Matches existing `TestEngraveSLIP39RecoverToBackup` mechanics minus the hold-ack.
- **Back-at-fork test** (`TestEngraveSLIP39RecoverForkBack`): realizable via `Button1` at the ChoiceScreen (`Choose` cancel at `gui.go:1344` → returns `(0,false)`).
- **Verbatim-arm test** (`TestEngraveSLIP39RecoverForkVerbatim`): the navigation is realizable (`Down`→`Button3` selects `sel==1`), and the *negative* assertion (never render "Recovered Fingerprint") is sound. **But the plan's positive assertion is NOT realizable:** plan lines 72-73 say "Assert a frame shows the verbatim share Title (id #m/t)." The share Title (`fmt.Sprintf("%d #%d/%d", …)`, `:458`) is passed into `backup.Seed`/`EngraveSeed` as **engraving geometry**, not rendered as on-screen label text. `EngraveScreen.draw` (`gui.go:2452-2504`) only renders the fixed strings "Engrave Plate" (title) and "Insert a blank plate and close the lock…" (body). `uiContains` matches rendered label text only, so `"#"`/the identifier will never appear in a frame. The realizable positive assertion is reaching the verbatim engrave screen ("Engrave Plate" / "Insert a blank plate") **while** "GIFT"/"Word 1 of 24" (the BIP-39 `SeedScreen`) and "Recovered Fingerprint" are never seen — that uniquely distinguishes the verbatim arm from the BIP-39 arm.
- Existing guards stay green: `TestEngraveSLIP39BackoutRecognized` calls `engraveObjectFlow` (test:86), not `engraveRecoveredSLIP39` — unaffected by the signature change. `TestSLIP39FingerprintBackRecognized`, `TestConfirmSLIP39*`, `TestRecoverSLIP39*` call lower-level functions directly — unaffected. Only `TestEngraveSLIP39RecoverToBackup` drives through `engraveRecoveredSLIP39` and is being rewritten as planned.

### Scope / doc — see MINOR-1
README target exists (`README.md:10` "About this fork"), but it currently lists only CODEX32 and md1/mk1 features — there is **no existing "SLIP-39 feature note"** to add the line "under" (plan line 144). The doc line is appropriate content; the placement instruction is slightly inaccurate. No forbidden bodies are touched (`engraveSLIP39Verbatim`, `confirmSLIP39Fingerprint`, `backupWalletFlow`, `masterFingerprintFor` all unchanged — confirmed).

---

## Findings

### CRITICAL

**CRITICAL-1 — Dead-ref cleanup instruction would break the build.** Plan lines 130-131 and checklist lines 163-164 instruct removing `holdToConfirm`, the `ack` `ConfirmWarningScreen`, and `assets.IconHammer`/unused `assets` if they "become unused." They do **not** become unused: `holdToConfirm` and `ConfirmWarningScreen` are still called by the slow-recovery gate at `slip39_polish.go:256-261`, and `assets.IconHammer`/the `assets` import are used at `slip39_polish.go:121,432` (and elsewhere). Only the local `ack` variable is removed (with its block). Following the instruction literally deletes still-referenced symbols and fails `go build`/`go vet`.
**Required fix:** Reword Step 3 / checklist to: "Remove only the local `ack` `ConfirmWarningScreen` literal in `engraveRecoveredSLIP39`. Do NOT remove `holdToConfirm`, the `ConfirmWarningScreen` type, or the `assets` import — all remain used by `recoverSLIP39Flow`'s slow-recovery gate (`:256-261`) and other call sites (`:121,432`). Confirm `go vet`/`gofmt` clean after the edit (no import should need removal)."

**CRITICAL-2 — Verbatim-arm test asserts an unrenderable string.** Plan lines 72-73 direct the test to "Assert a frame shows the verbatim share Title (id #m/t)." That title is engraving geometry inside `backup.Seed`, never rendered as UI label text; `EngraveScreen.draw` (`gui.go:2452-2504`) renders only "Engrave Plate" / "Insert a blank plate…". The assertion can never pass via `uiContains`, so the test as specified is unrealizable (it would fail or be silently weakened to only the negative check).
**Required fix:** Respecify the verbatim-arm test's positive assertion to reach the verbatim **EngraveScreen** — e.g. `pumpUntil(frame, "Insert a blank plate", …)` (or "Engrave Plate") after the `Down`,`Button3` selection — combined with asserting "Recovered Fingerprint" and "GIFT"/"Word 1 of 24" are **never** seen (the latter uniquely separates the verbatim arm from the BIP-39 `backupWalletFlow` arm, since both eventually reach an `EngraveScreen`).

### IMPORTANT

**IMPORTANT-1 — Lead/Title text mismatch between plan body and consult intent, and label-fit not actually verified.** The plan's code block (lines 99-105) sets `Title: "Recovered Seed"`, `Lead: "These shares can be read two ways — pick the wallet that made them:"` and choices `"BIP-39 seed (this toolkit / from a phrase)"` / `"Trezor / other — engrave shares"`. This is fine, but Step 3 (line 129) only says "Verify the labels fit the 480px display" without committing to it. `ChoiceScreen.Draw` lays choices as single `widget.Label` (not `Labelw`-wrapped) buttons centered with `maxW` (`gui.go:1402-1412`) — a long single-line choice like "BIP-39 seed (this toolkit / from a phrase)" risks overflowing the `content.Shrink(16,0,16,0)` width on a 480px panel, unlike the `Lead` which **is** wrapped (`Labelw`, :1392). The plan asserts "the `ChoiceScreen` renders multi-line" (line 130) — that is true for the **Lead** but the **choice buttons are single-line `Label`s**. This needs an explicit width check or shortened choice strings, not a hand-wave.
**Required fix:** Either shorten the choice strings to known-fitting lengths (compare against the shipped `slip39LengthPick`/passphrase choices, which are short) or add a concrete render-width assertion/manual check step. State that choice buttons are single-line (`widget.Label`, not wrapped) so the author sizes them accordingly.

### MINOR

**MINOR-1 — README placement instruction references a non-existent "SLIP-39 feature note."** Plan line 144 says add the line "under the SLIP-39 feature note," but the fork README "About this fork" (lines 10-28) lists only CODEX32 and md1/mk1 — there is no SLIP-39 bullet. Recommend: add the SLIP-39 recovery doc line as its own item under "About this fork" (or note that no SLIP-39 bullet exists yet and add one). Non-blocking.

---

## Verdict

**NOT GREEN — 2 Critical / 1 Important.**

Required fixes before implementation:
1. **(CRITICAL-1)** Correct the dead-ref instruction: remove only the local `ack` literal; keep `holdToConfirm`, `ConfirmWarningScreen`, and the `assets` import (all still used at `slip39_polish.go:256-261, 121, 432`). Removing them breaks the build.
2. **(CRITICAL-2)** Respecify the verbatim-arm test's positive assertion to reach the verbatim `EngraveScreen` ("Insert a blank plate" / "Engrave Plate") plus the negative assertions (never "Recovered Fingerprint", never "GIFT"/"Word 1 of 24"). The `id #m/t` share title is engraving geometry and is never rendered as frame text.
3. **(IMPORTANT-1)** Commit to a concrete 480px label-fit check (or shorten the choice strings); note that `ChoiceScreen` choice buttons are single-line (`widget.Label`), only the `Lead` wraps.

The design fidelity, signatures, caller wiring, safety routing, and Back-recognition contract are all sound and faithful to the consult; the failures are in the cleanup instruction and one test assertion, both mechanically fixable. Re-dispatch after folding.
