<!--
Persisted verbatim. opus-architect R1 re-dispatch of the SLIP-39 Trezor-routing plan R0 gate
(@ 3f5b374). Reviewer agentId a43673ec047d81361. Verdict: GREEN — 0C/0I. All three R0 blockers
(C1 dead-ref narrowing, C2 unrenderable-Title assertion, I1 single-line choice overflow) +
MINOR-1 folded correctly, verified against shipped source 9db3fd2; no drift; safety routing
undisturbed. Cleared for implementation. The text below is the agent's report exactly as returned; do not edit.
-->

# R1 GATE REVIEW — SLIP-39 Trezor-routing plan

**Reviewer:** opus architect (adversarial R1 re-dispatch of the R0 gate, read-only).
**Plan under review:** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_slip39_trezor_routing.md` (commit `3f5b374` — "design: fold trezor-routing plan-R0 (2C/1I+1m)").
**R0 review folded:** `design/agent-reports/seedhammer-slip39-trezor-routing-plan-review-R0.md` (C1/C2/I1 + MINOR-1).
**Design authority:** `design/agent-reports/seedhammer-slip39-verbatim-hex-design-consult.md` (agent `a89706fc9955e8479`).
**Base verified against:** fork `seedhammer` `main` @ `9db3fd2` (confirmed via `git rev-parse`/`git log`: "Merge feat/slip39-recovery-gui … Cycle D, Phase D2").
**Date:** 2026-06-18.

---

## Verification Results

### C1 — Dead-ref cleanup narrowed to the local `ack` only — FOLDED (PASS)

Plan now reads (Step 3, lines 139-144):
> **C1 fold — dead-ref cleanup is narrow:** remove ONLY the local `ack` `ConfirmWarningScreen` literal inside `engraveRecoveredSLIP39` (it's replaced by the `ChoiceScreen`). Do **NOT** remove `holdToConfirm`, the `ConfirmWarningScreen` type, or the `assets` import — all remain in use (the high-iteration-exponent gate in `recoverSLIP39Flow` at `slip39_polish.go:256-261`, and `assets` at `:121,432`).

Verified against shipped `gui/slip39_polish.go`:
- `holdToConfirm` is called at `:261`, inside the high-iteration-exponent gate in `recoverSLIP39Flow`. **Still used — must keep.**
- `ConfirmWarningScreen` is instantiated at `:256` in that same gate. **Still used — must keep.**
- `assets` import is used at `:116` (`IconBack`), `:121` (`IconHammer`), `:259` (`IconInfo`), `:391` (the `ack` block being removed), `:431` (`IconBack`), `:432` (`IconHammer`). After removing only the `ack` block, `assets` remains referenced at `:121,432` (and others). **Import must NOT be removed.**

Removing any of these would break `go build`/`go vet` of the same file, exactly as R0 warned. The fold's narrowing is correct, and the no-import-removal assertion (line 144, checklist 178-180) matches reality.

### C2 — Verbatim-arm test targets EngraveScreen text, not the share Title — FOLDED (PASS)

Plan now reads (Step 1, lines 71-77):
> POSITIVE assertion (C2 fold): the verbatim path reaches the EngraveScreen — pumpUntil a frame renders "Insert a blank plate" (or "Engrave Plate"). NOTE: the verbatim share Title (id #m/t) is engraving GEOMETRY inside backup.Seed, NOT rendered as on-screen label text, so do NOT assert it. NEGATIVE assertions … the run NEVER renders "Recovered Fingerprint" AND never the BIP-39 SeedScreen …

Verified against shipped source:
- The share Title is `Title: fmt.Sprintf("%d #%d/%d", scan.Identifier, scan.MemberIndex+1, scan.MemberThreshold)` (`slip39_polish.go:458`), passed into `backup.Seed.Title` → `backup.EngraveSeed` (`:454-462`) — i.e. engraving geometry, never a UI label. The `scan` fields exist on `slip39words.Share` (`slip39/share.go:22-31`: `Mnemonic`, `Identifier`, `MemberIndex`, `MemberThreshold`).
- `EngraveScreen.draw` (`gui.go:2452-2504`) renders only fixed strings: title `"Engrave Plate"` (`:2498`) and idle body `"Insert a blank plate and close the lock…"` (`:2468`). The realizable positive assertion now targets exactly these.
- The negative `"Recovered Fingerprint"` is the title at `confirmSLIP39Fingerprint` (`gui.go:434`) — uniquely present on the BIP-39 arm, absent on the verbatim arm.

The new assertion is realizable; the old (share Title) was not. Correct fold.

### I1 — Choice strings shortened, explanation moved to wrapped Lead — FOLDED (PASS)

Plan now uses (lines 105-113): `Lead: "How was this backup made? A BIP-39 phrase / this toolkit recovers as a seed. A Trezor or other SLIP-39 wallet should engrave its shares verbatim."` and `Choices: []string{"BIP-39 seed", "Engrave shares"}`, with the inline note (lines 107-109) "ChoiceScreen choice buttons are SINGLE-LINE (widget.Label, NOT wrapped), so keep them short."

Verified against `ChoiceScreen.Draw` (`gui.go:1389-1412`):
- `Lead` is laid out with `widget.Labelw(... dims.X-2*8 ...)` at `:1392` — **width-wrapped.** Correct home for the explanation.
- Choices are laid out with `widget.Label(...)` at `:1408` (single-line, not `Labelw`), centered with `maxW` after `content.Shrink(16,0,16,0)` (`:1396`). **Single-line, must stay short** — matches the fold's note.
- The two short labels ("BIP-39 seed" = 11 chars; "Engrave shares" = 14 chars) are comparable to the shipped single-line choices ("Skip", "Enter passphrase" at `:274`), well within a 480px button width.

`sel == 0` is the zero-value default (`s.choice` per `Choose`, `gui.go:1347`) = "BIP-39 seed"; `sel == 1` = "Engrave shares" → verbatim, matching the plan's branch logic (lines 110-125). Correct fold.

### MINOR-1 — README line added as its own item under "About this fork" — FOLDED (PASS)

Plan Task 2 (lines 157-163) now states the README "currently lists only CODEX32 and md1/mk1 (no SLIP-39 bullet yet, MINOR-1)" and adds the SLIP-39-recovery line "as its OWN item under that section." Verified against fork `README.md:10-28` ("About this fork"): it lists only CODEX32 (`:16`) and BCH-validated md1/mk1 (`:19`) — there is no SLIP-39 bullet. The fold's premise and placement are accurate.

### No-drift check — PASS

- **Sole caller / signature:** `grep engraveRecoveredSLIP39 gui/` returns exactly the call at `:372`, the def at `:384`, and the doc comment at `:380`. The plan's caller update (`engraveSLIP39` passes `scan`, Step 3.1) is correct and complete; `scan` is the `engraveSLIP39` parameter (`:359`). New signature types (`slip39words.Share`, `bip39.Mnemonic`, `chaincfg`) are all already imported (`slip39_polish.go:8,9,16`).
- **Safety routing intact (R0-confirmed sound, undisturbed by the fold):** verbatim arm = `engraveSLIP39Verbatim(ctx, th, scan)` with NO fingerprint (plan lines 119-125; consult line 65); BIP-39 arm = `masterFingerprintFor` → `confirmSLIP39Fingerprint` → `backupWalletFlow`, byte-identical to shipped `:398-406` (plan lines 127-136); Back → `return false` → caller `continue`s to the original confirm (plan lines 116-118; shipped `:372-374`). No hex / `SeedString` (lines 17-21, 162). Unchanged.
- **Folds touched only intended regions:** Step 1 test block (C2), Step 3 code+instruction (C1 + I1), the self-review checklist (lines 168-186, consistent with all three folds), and Task 2 (MINOR-1).
- **Stale-residue grep:** no "drop the dead refs", no long choice strings, no "These shares can be read two ways", no "this toolkit / from a phrase", no "under the SLIP-39 feature note". The only "share Title" hits (plan lines 72, 182) are the C2 NOTE/checklist lines stating it must NOT be asserted — that is the fold, not residue.
- Test helpers referenced by the plan all exist: `pumpUntil` (`gui/slip39_polish_test.go:329`), `uiContains` (`gui/gui_test.go:480`), `driveShare`/`driveRecover` (`:217,228`).
- No new contradiction introduced; design fidelity / signatures / safety unchanged from the R0-confirmed-sound state.

---

## Findings

### CRITICAL
None.

### IMPORTANT
None.

### MINOR
None.

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

All three R0 blockers (C1 dead-ref narrowing, C2 unrenderable-Title assertion, I1 single-line choice overflow) and MINOR-1 (README placement) are folded correctly and verified against shipped fork source at `9db3fd2`. The folds touched only the intended regions, introduced no drift, and left the R0-confirmed design fidelity, signature/caller wiring, and safety routing (BIP-39 arm = fingerprint→`backupWalletFlow`; verbatim arm = `engraveSLIP39Verbatim(scan)`, no fingerprint; Back→false→caller continues) undisturbed. The plan is cleared for implementation.
