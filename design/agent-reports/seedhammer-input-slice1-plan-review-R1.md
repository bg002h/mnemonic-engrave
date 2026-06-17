# SeedHammer input Slice 1 (BIP-39 word-entry polish) — PLAN review — R1 (convergence)

- **Stage:** plan R1 convergence gate (verify R0's 0C/1I/5m fold).
- **Date:** 2026-06-17
- **Reviewer:** opus `feature-dev:code-architect` (read-only; re-verified every amended block against `gui/gui.go`, `gui/event*.go`, `gui/widget*`, `gui/op/op.go`, `gui/layout/layout.go`, `gui/theme.go`, `gui/gui_test.go`, `gui/codex32_input_test.go`, `bip39/bip39.go`, `bip39/bip39_test.go`, `bip39/wordlist.go`).
- **Verdict:** **GREEN — 0 Critical / 0 Important.** (1 Minor doc-table nit, non-blocking — since folded.)

---

## VERBATIM REVIEW OUTPUT

### R0-finding resolution
**I1 (RESOLVED) — 12-word last-word test.** `TestWordFlowLastWord12` added (`validMnemonic(12)`, `m[11]=-1`, `selected=11`); entry sub-test asserts `"128 matches"`, commit sub-test types `LabelFor(want)` and asserts `m[11]==want`. **No-hang confirmed:** the target T is itself a candidate, so `updateValidCandidateKeys` keeps T's next letter enabled at every prefix (T matches all its own prefixes); the top-of-loop `updateKeys("")` applies the mask before the first `kbd.Update`; after runes drain, `okBtn.Clicked` (Button3, freed in Task 2) fires, `completeCandidateWord` returns `(T,true)` via its exact-label loop, `selected++ == len` → return. Deterministic. Step 7/9 run-commands + self-review §6 updated.
**M1 (RESOLVED) — I2 assertion.** Now `if w, ok := completeCandidateWord(...); ok && w == nonCand` — fires only when a non-candidate completes to ITSELF (the exact hole); completing a prefix to a *different* candidate (`w != nonCand`) or no completion (`ok false`) does not false-fail. Robust regardless of wordlist prefix pairs (ADD⊂ADDICT etc.); the hardcoded `nvalid=1` is now harmless.
**M2 (RESOLVED) — count clamp.** `countY := wordOff.Y + longest.Y + 8; if lim := top.Max.Y - csz.Y; countY > lim { countY = lim }`. `top` in scope; `top.Max.Y` is the keyboard top edge (`content.Max.Y - kbdsz.Y`). Guarantees no keyboard overlap and on-screen on the 240px display; arithmetic correct.
**M3 (RESOLVED) — codex32 edit explicit.** Task 2 Step 1 now states "change ONLY the keypad-confirm line (:31) + comment (:29); leave the menu line :28 (ChoiceScreen Button3) unchanged," quoting the post-edit block. Verified against the file.
**M4 (RESOLVED) — gpgsign.** `git config --get commit.gpgsign  # must print: true` + idempotent `git config commit.gpgsign true`.
**M5 (RESOLVED) — upstream guard.** `git remote get-url upstream || git remote add upstream https://github.com/seedhammer/seedhammer.git` before fetch.

### Fresh-eyes re-scan — Critical: none. Important: none.
Cleared: added-test ↔ candidate-path interaction (direct-call no-`runUI` pattern like the passing `TestWordKeyboardScreen`; `ctx.Frame` a no-op without `FrameCallback`; commit terminates); `TestWordKeyboardScreen` regression (`make(Mnemonic,1)` → guard fires but `LastWordCandidates` nil on `len%3==1` → full-keyboard fallback, all 2048 words committable on Button3); clamp arithmetic (cannot go negative/onto word box; tests assert text presence not coordinates); signature/scope drift (all helper sigs + closures consistent with call sites; `widget.Labelf` matches `label.go:20`; `Word` is `int` so `-1`/`>=NumWords` well-typed; three `okBtn` sites `gui.go:543,628,688`; `editBtn` `:1922` left on Button2); imports (`gui.go` has `strings`/`unicode`/`image`, no `fmt` needed; `gui_test.go` adds `unicode` per plan; `bip39.go` avoids `slices` via make+copy; `bip39_test.go` internal package, no new imports). Spec §6 now fully covered (both 12- and 24-word last-word flows + I2 + mask + LastWordCandidates unit + both Button2→Button3 + title + match-count). §4.1-4.4, §5 backstop, §7 all map. No placeholders; no hang risk.

### Minor (non-blocking)
**m1 — File-structure table omitted `TestWordFlowLastWord12`** (the test body, run-commands, and self-review §6 all include it, so an executing agent writes it regardless). *Folded by main session: appended to the table.*

### Final verdict
**GREEN — 0 Critical / 0 Important.** All R0 findings (I1 + M1–M5) correctly folded and verified against the real code; amendments introduce no new Critical/Important. **Cleared to proceed to implementation.**

---

## Loop summary
| Round | Verdict | Findings |
|---|---|---|
| R0 | NOT-GREEN (0C/1I/5m) | I1 missing 12-word flow test; M1-M5 hardening |
| **R1** | **GREEN (0C/0I)** | I1 + M1–M5 verified folded; no new C/I; 1 cosmetic table nit (m1) folded. **Cleared to implementation.** |
