# Firmware PR1 plan-R1 review — re-enable on-device CODEX32 entry

- **Stage:** implementation-plan R1 (re-dispatch after plan-R0 fold) — convergence check
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect`
- **Plan:** `design/IMPLEMENTATION_PLAN_firmware_pr1_codex32.md` (post plan-R0 fold)
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Plan cleared its pre-implementation R0 gate.

---

## VERBATIM REVIEW OUTPUT

## plan-R1 Convergence Check — Firmware PR1 (codex32 keypad)

**C1 (Critical) — keypad uppercases; assertion must compare to uppercase: RESOLVED**
- (a) `strings` imported and used exactly once: `want := strings.ToUpper(share)`. No shadowing — `want` is a fresh local.
- (b) Keypad stores uppercase: `k.Fragment + string(unicode.ToUpper(r))` at `gui/gui.go:1036`. All-uppercase input is ACCEPTED by `codex32.New`: `setCase` (`codex32/checksum.go:132-153`) rejects only *mixed* case — a consistent `upperCase` run passes; and `inputHRP`/`inputChar` lowercase via `unicode.ToLower` before deriving the field element (`checksum.go:82,90,104` → `feFromRune`), so checksum math is case-insensitive. Live validation runs `codex32.New(kbd.Fragment)` every keystroke (`gui/gui.go:633-634`); on the complete share `valid=true`, and `click(Button2)` → `okBtn.Clicked` returns `share`. Uppercase input is NOT rejected — no flag.
- (c) `String()` returns the stored string verbatim (`codex32/codex32.go:390-392`) and `New` stores `s` as-is (`codex32.go:119`), so `s.String() == strings.ToUpper(share)` holds.

**I1 (Important) — without-change path panics, not "returns a Mnemonic": RESOLVED**
- Wording now accurate. Step 2 and Self-Review both say the 24-WORDS path **panics** (uppercase "M" not a valid BIP-39 fragment) and "never returns a codex32.String." Confirmed: `updateValidBIP39Keys` → `bip39.ClosestWord(frag)` false → `panic("invalid fragment")` at `gui/gui.go:873`.

**New C/I from the folds:** none.
- Test compiles: `strings` used once, no unused imports, no shadowing.
- `runes(share)` drives the keypad correctly: `RuneEvent` handler (`gui/gui.go:1014-1022`) lowercases `e.Rune`, matches the lowercase alphabet keys (`gui/gui.go:624`), registers only when `k.Valid(key)` (gated by live codex32 validation), building exactly the valid fragment. Then `click(Button2)` fires `okBtn` (`Button2`, `gui/gui.go:628`). Sequence sound.
- Minor (non-blocking, already an "Open item"): the 47-char vector's validity and per-key acceptance are execution-time checks, not plan defects.

**Verdict: GREEN (0C/0I).**

---

## Loop summary (added by main session)

| Round | Verdict | Folded |
|---|---|---|
| plan-R0 | NOT-GREEN | 1C (keypad uppercases → assert ToUpper) + 1I (without-change panics, not Mnemonic) |
| **plan-R1** | **GREEN** | none — converged |
