# SLIP-39 recovery — Trezor-routing fork Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development.
> Small (S) follow-up on a seed-bearing flow — still runs the full gated pipeline.
> **Design authority:** the architect consult `design/agent-reports/seedhammer-slip39-verbatim-hex-design-consult.md` (rescoped the `-verbatim-hex` follow-up). This AMENDS the
> shipped recovery-engrave design (`SPEC_seedhammer_slip39_recovery.md` §5.7's one-way
> acknowledgement → a two-way fork). Base: fork `main` `9db3fd2` (Cycle D shipped).

**Goal:** Remove the post-recovery dead-end for a non-constellation (e.g. Trezor) SLIP-39 user.
Today, after recovery, `engraveRecoveredSLIP39` shows a one-way hold-to-confirm ("…a Trezor
backup would engrave the WRONG seed…") whose only non-proceed action is abort. Replace it with
a **two-way fork**: "Engrave as BIP-39 seed (this toolkit)" → today's path (fingerprint →
`backupWalletFlow`); "Trezor / other — engrave my shares verbatim" → the existing
`engraveSLIP39Verbatim` on the first share, with **NO** BIP-39 fingerprint (it's
convention-specific and would mislead on that arm). Plus a user-facing doc line.

**Why NOT hex:** the architect verified (Trezor docs) that no consumer wallet restores from a
raw seed / master-secret hex — the ecosystem restores by re-entering **share words**. So the
correct artifact for the Trezor user is their **shares on steel** (already engravable
verbatim, all lengths since D2), not a hex plate. This plan builds the routing to that
existing artifact; it does NOT add a hex/`SeedString` artifact.

**Architecture:** one function changes (`engraveRecoveredSLIP39`) + its one caller
(`engraveSLIP39` passes `scan` through). All in `gui/slip39_polish.go`. No crypto, no new
artifact, no new screen type (reuse `ChoiceScreen`). The `engraveSLIP39Verbatim`,
`confirmSLIP39Fingerprint`, `backupWalletFlow`, and `masterFingerprintFor` bodies are unchanged.

**Tech stack:** Go/TinyGo. Test: `/home/bcg/.local/go/bin/go test ./gui/ ./slip39/ ./bip39/`
+ `go vet ./gui/` + `gofmt -l gui/`.

**Commit hygiene:** explicit paths; SSH-signed + DCO (`git commit -S -s`, author Brian Goss);
`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

---

## File structure

| File | Change |
|---|---|
| `gui/slip39_polish.go` | `engraveRecoveredSLIP39(ctx, th, m)` → `engraveRecoveredSLIP39(ctx, th, scan slip39words.Share, m bip39.Mnemonic)`: replace the one-way `ConfirmWarningScreen` hold-ack with a two-arm `ChoiceScreen` fork; verbatim arm → `engraveSLIP39Verbatim(ctx, th, scan)`. Caller `engraveSLIP39` passes `scan`. |
| `gui/slip39_polish_test.go` | Update the existing recover-to-backup test to the fork (BIP-39 arm); add verbatim-arm + Back-at-fork tests. |
| `README.md` *(fork)* — "About this fork" section | One doc line: Trezor/other SLIP-39 backups → engrave shares verbatim here, or recover with the `mnemonic-toolkit` CLI. |

**Unchanged (must stay green):** `engraveSLIP39Verbatim`, `confirmSLIP39Fingerprint`,
`backupWalletFlow`, `masterFingerprintFor`, all of `slip39/` (D1 crypto), `codex32/`, `backup/`,
`bip39/`.

---

## Task 0: Worktree

- [ ] **Step 1:** `git -C /scratch/code/shibboleth/seedhammer worktree add /scratch/code/shibboleth/seedhammer-wt-slip39-trezor -b feat/slip39-trezor-routing 9db3fd2`
- [ ] **Step 2:** Baseline green: `cd …-trezor && /home/bcg/.local/go/bin/go test ./gui/ ./slip39/ ./bip39/`.

---

## Task 1: Two-way fork in `engraveRecoveredSLIP39`

**Files:** `gui/slip39_polish.go`, `gui/slip39_polish_test.go`.

- [ ] **Step 1: Update + add the failing tests.** The shipped `TestEngraveSLIP39RecoverToBackup`
  drives the hold-ack ("WRONG seed") → fingerprint → backup; rewrite it to drive the **BIP-39
  arm** of the new fork. Add:

```go
func TestEngraveSLIP39RecoverForkVerbatim(t *testing.T) {
	// After recovery, choose the "engrave shares verbatim" arm (sel==1).
	// Drive: ...confirm Recover → driveShare(2nd share) → Skip passphrase →
	// at the fork ChoiceScreen select the verbatim choice (Down→Button3)...
	// POSITIVE assertion (C2 fold): the verbatim path reaches the EngraveScreen —
	// pumpUntil a frame renders "Insert a blank plate" (or "Engrave Plate").
	// NOTE: the verbatim share Title (id #m/t) is engraving GEOMETRY inside
	// backup.Seed, NOT rendered as on-screen label text, so do NOT assert it.
	// NEGATIVE assertions (these uniquely separate the verbatim arm from the
	// BIP-39 arm — both eventually reach an EngraveScreen): the run NEVER renders
	// "Recovered Fingerprint" AND never the BIP-39 SeedScreen ("Word 1 of" / the
	// recovered words), confirming engraveSLIP39Verbatim ran, not backupWalletFlow.
}

func TestEngraveSLIP39RecoverForkBIP39(t *testing.T) {
	// Choose "BIP-39 seed (this toolkit)" → the fingerprint screen ("Recovered
	// Fingerprint" / "%.8X") IS shown, then backupWalletFlow is reached.
}

func TestEngraveSLIP39RecoverForkBack(t *testing.T) {
	// Back at the fork ChoiceScreen → engraveRecoveredSLIP39 returns false →
	// engraveSLIP39 continues back to the original confirm (recognized; no engrave).
}
```

- [ ] **Step 2:** Run → FAIL (signature mismatch / fork not implemented).

- [ ] **Step 3: Implement.**
  1. In `engraveSLIP39` (the dispatch loop), change the `slip39Recover` arm's call to
     `engraveRecoveredSLIP39(ctx, th, scan, m)` (pass the first share `scan`).
  2. Rewrite `engraveRecoveredSLIP39` signature to
     `func engraveRecoveredSLIP39(ctx *Context, th *Colors, scan slip39words.Share, m bip39.Mnemonic) bool`
     and replace the `ConfirmWarningScreen`/`holdToConfirm` block with a fresh `ChoiceScreen`
     fork (allocate fresh per call, like `backupWalletFlow`):

```go
choice := &ChoiceScreen{
	Title: "Recovered Seed",
	// The Lead IS width-wrapped (widget.Labelw) — put the explanation here.
	Lead: "How was this backup made? A BIP-39 phrase / this toolkit recovers as a " +
		"seed. A Trezor or other SLIP-39 wallet should engrave its shares verbatim.",
	// I1 fold: ChoiceScreen choice buttons are SINGLE-LINE (widget.Label, NOT
	// wrapped), so keep them short — comparable to the shipped slip39LengthPick /
	// passphrase choices. Detail lives in the Lead above.
	Choices: []string{
		"BIP-39 seed",    // sel == 0 (default)
		"Engrave shares", // sel == 1
	},
}
sel, ok := choice.Choose(ctx, th)
if !ok {
	return false // Back → caller continues to the original confirm
}
if sel == 1 {
	// Not a constellation backup: engrave the share verbatim (convention-agnostic,
	// restorable). NO BIP-39 fingerprint here — it would be a misleading
	// "verification" of a number unrelated to a non-BIP-39 wallet.
	engraveSLIP39Verbatim(ctx, th, scan)
	return true
}
// BIP-39 arm: the existing records cross-check + native seed engrave.
mfp, err := masterFingerprintFor(m, &chaincfg.MainNetParams, "")
if err != nil {
	showError(ctx, th, "Recovery failed", "could not derive the fingerprint")
	return false
}
if !confirmSLIP39Fingerprint(ctx, th, mfp) {
	return false
}
backupWalletFlow(ctx, th, m)
return true
```
  The short choice strings above fit the 480px single-line button width (only the `Lead`
  wraps). **C1 fold — dead-ref cleanup is narrow:** remove ONLY the local `ack`
  `ConfirmWarningScreen` literal inside `engraveRecoveredSLIP39` (it's replaced by the
  `ChoiceScreen`). Do **NOT** remove `holdToConfirm`, the `ConfirmWarningScreen` type, or the
  `assets` import — all remain in use (the high-iteration-exponent gate in `recoverSLIP39Flow`
  at `slip39_polish.go:256-261`, and `assets` at `:121,432`). After the edit, `go vet ./gui/`
  + `gofmt -l gui/` must be clean with NO import removal.

- [ ] **Step 4:** Run → PASS; `go vet ./gui/`, `gofmt -l gui/` clean. The shipped guards
  (`TestConfirmSLIP39*`, `TestRecoverSLIP39*`, `TestEngraveSLIP39BackoutRecognized`, codex32,
  BIP-39, backup goldens) stay green.
- [ ] **Step 5: Commit** → `feat: slip39 recovery — two-way BIP-39-vs-verbatim engrave fork`.

---

## Task 2: Doc line (fork README)

**Files:** `README.md` (fork) — the existing "About this fork" section.

- [ ] **Step 1:** The fork README "About this fork" currently lists only CODEX32 and md1/mk1
  (no SLIP-39 bullet yet, MINOR-1). Add a SLIP-39-recovery line as its OWN item under that
  section:
  > *Recovering a SLIP-39 backup: this device engraves the recovered seed as BIP-39 words for
  > backups made from a BIP-39 phrase / the `mnemonic` toolkit. For a Trezor or other
  > SLIP-39 wallet backup, choose "engrave shares" to engrave your share words verbatim, or
  > use the `mnemonic-toolkit` CLI to recover off-device.*
- [ ] **Step 2: Commit** → `docs: note SLIP-39 recover BIP-39-vs-verbatim choice for non-toolkit backups`.

---

## Self-review checklist

- The fork is a `ChoiceScreen` (two explicit arms + Back), NOT a one-way ack; Back returns
  `false` so the caller continues to the confirm (recognized, no engrave).
- The verbatim arm calls `engraveSLIP39Verbatim(ctx, th, scan)` on the FIRST share and shows
  **no** BIP-39 fingerprint; the BIP-39 arm keeps the `confirmSLIP39Fingerprint` records-check
  before `backupWalletFlow`.
- `engraveRecoveredSLIP39` gained `scan`; the one caller passes it; no other caller exists.
- No hex / `SeedString` artifact added (architect: no restore path). `engraveSLIP39Verbatim`/
  `confirmSLIP39Fingerprint`/`backupWalletFlow`/`masterFingerprintFor` bodies unchanged.
- ONLY the local `ack` literal is removed; `holdToConfirm` / `ConfirmWarningScreen` / the
  `assets` import STAY (still used by the high-e gate at `:256-261` and `:121,432`); vet/gofmt
  clean with no import removal; no new `gui.go` import.
- The verbatim-arm test's positive assertion targets the EngraveScreen text ("Insert a blank
  plate"/"Engrave Plate"), NOT the share Title (which is engraving geometry, never rendered);
  negatives assert "Recovered Fingerprint" and the BIP-39 SeedScreen are never seen.
- Choice buttons are single-line and short ("BIP-39 seed" / "Engrave shares"); the explanation
  is in the wrapped Lead.
- Signed + DCO + Brian Goss; existing guards green.
