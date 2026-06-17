# SeedHammer input Slice 1 (BIP-39 word-entry polish) — PLAN review — R0

- **Stage:** plan R0 gate (mandatory; must reach 0C/0I before implementation).
- **Date:** 2026-06-17
- **Reviewer:** opus `feature-dev:code-architect` (read-only; verified every code block/test against `gui/gui.go`, `gui/event.go`, `gui/widget.go`, `gui/widget/label.go`, `gui/op/op.go`, `gui/layout/layout.go`, `gui/theme.go`, the three gui `*_test.go`, `bip39/bip39.go`, `bip39/bip39_test.go`, `bip39/wordlist.go`).
- **Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_input_slice1.md` (committed `b5e8d73`).
- **Verdict:** **NOT-GREEN — 0 Critical / 1 Important / 5 Minor.**

---

## VERBATIM REVIEW OUTPUT

### Critical
None. Every code block compiles against the real code; the event-routing analysis holds; core logic correct. Verified: Task 1 `LastWordCandidates` (guard-before-`Valid()`, `len==0` checked before slice, make+copy clone, 8/128 bit-math, `FixChecksum` fixture validity, `foundLast`); Task 2 Button3 routing (event model `gui/event.go:266-279`; exactly three `okBtn := &Clickable{Button: Button2}` at `gui/gui.go:543,628,688`; `editBtn` Button2 `:1922` untouched; both tests pass post-edit incl. the direct-call no-`runUI` path; no other Button3-keyboard consumer; `TestScan`/`TestMdmk*` don't drive the keyboard; codex32 menu `click(...,Button3)` `:28` is the ChoiceScreen choose-button, correctly left alone); Task 3 title (`layoutTitlef` sig `:1520`; `uiContains` space-strip + `ExtractText` glyph-drop proven via passing `TestEngraveScreenError`); Task 4 count (`widget.Labelf` sig `label.go:20`; zero `op.Op{}` layers harmlessly; on-screen on 240px; guard-extension ordering sound); Task 5 helpers+wiring (`completeCandidateWord` closes I2 hole; `updateValidCandidateKeys` mirrors `updateValidBIP39Keys` incl. `unicode.ToLower(...)-'a'`; closures capture by reference; memoization recomputes only on `selected` change; `make(Mnemonic,1)` falls back via nil guard; one-frame full-keyboard window harmless; commit sub-test terminates).

### Important
**I1 — Spec §6 requires a 12-word last-word test the plan does not provide.** The spec enumerates the 12-word last-word case co-equally with 24-word (candidate-prefix key enabled / non-candidate disabled, `128 matches` shown, completed word `Valid()`, I2 non-candidate check). The plan only adds 24-word coverage (`TestWordFlowLastWord24`, `TestUpdateValidCandidateKeys`, `TestCompleteCandidateWord` all use `validMnemonic(24)`); 12-word is covered only indirectly (the `len==128` math + length-agnostic helpers). A fresh engineer won't write the spec-mandated 12-word flow test, leaving a listed GREEN requirement unmet — and only a 12-word flow test catches a `"128 matches"` label width/centering regression the 24-word `"8 matches"` test can't. **Fix:** add `TestWordFlowLastWord12` mirroring `TestWordFlowLastWord24` with `validMnemonic(12)`/`m[11]=-1`/`selected=11`, asserting `"128 matches"` on entry + correct last word commits; or explicitly document a conscious deviation. Adding the test is the cleaner route to 0I.

### Minor
**M1 — `TestCompleteCandidateWord` I2 check is robust only by fixture luck.** It hardcodes `nvalid=1`; if `nonCand`'s label were a proper prefix of a candidate label (wordlist has ADD⊂ADDICT, ART⊂ARTIST, KIT⊂KITTEN), the `nvalid==1` branch returns that candidate → `ok==true` → false failure. For the `[0..23].FixChecksum()` fixture the first non-candidate is a low-index word that is no word's prefix, so it passes by data luck. **Fix:** assert the hole precisely — `if w, ok := completeCandidateWord(...); ok && w == nonCand { t.Error(...) }` (completing to a *different* candidate is correct; only completing to `nonCand` itself is the bug), or pass the real per-fragment count.
**M2 — On-device count placement may overlap word box/keyboard.** `image.Pt((dims.X-csz.X)/2, wordOff.Y+longest.Y+8)` lands at Y≈98 on the 240px test display (keyboard top ≈114) so tests pass, but on 480×320 the `+8` could crowd the keyboard. Cosmetic; no test affected. **Fix:** clamp Y to `min(wordOff.Y+longest.Y+8, top.Max.Y - csz.Y)` + note manual QA.
**M3 — Task 2 Step 1 codex32 edit must not touch line 28.** The snippet correctly targets only the keypad confirm (:31) + comment (:29) and leaves the menu `click(...,Down,Down,Button3)` (:28) intact, but the prose could be misread. **Fix:** state "edit ONLY line 31 + comment :29; leave :28 unchanged."
**M4 — Task 0 `git config commit.gpgsign` is a read, not a set.** Bare command only prints the value. **Fix:** annotate as read-back `git config --get commit.gpgsign  # must print: true` + fallback `git config commit.gpgsign true` if unset.
**M5 — Task 0 assumes an `upstream` remote.** `git fetch -q upstream` presupposes the remote exists. **Fix:** guard `git remote get-url upstream || git remote add upstream https://github.com/seedhammer/seedhammer.git`.

### Spec coverage
§4.1→Task 3 ✓; §4.2→Task 4 + 5b guard-extension ✓; §4.3→Task 2 (3 flows + both tests) ✓; §4.4→Task 1 + Task 5 ✓; §5 backstop unchanged ✓; §6 → **gap at 12-word last word (I1)**, all others mapped; §7 no bump ✓. Types/signatures consistent; `"unicode"` import for `gui_test.go` flagged by plan; no placeholders; no hang risk.

**To converge to GREEN:** resolve I1 (add the 12-word flow test, or document the deviation). M1–M5 are recommended hardening, non-blocking.

---

## Loop summary (added by main session)
| Round | Verdict | Findings |
|---|---|---|
| R0 | NOT-GREEN (0C/1I/5m) | I1 missing 12-word last-word flow test (spec §6); M1 fixture-luck I2 assertion; M2 count clamp; M3 codex32 edit explicitness; M4 gpgsign read-vs-set; M5 upstream-remote guard. Folding I1 + all 5 Minors. |
