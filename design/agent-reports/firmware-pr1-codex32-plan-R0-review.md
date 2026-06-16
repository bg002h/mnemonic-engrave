# Firmware PR1 plan-R0 review — re-enable on-device CODEX32 entry

- **Stage:** implementation-plan R0 gate (pre-code)
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (traced the full event path against the fork at `/scratch/code/shibboleth/seedhammer`)
- **Plan:** `design/IMPLEMENTATION_PLAN_firmware_pr1_codex32.md`
- **Verdict:** **NOT-GREEN** — 1 Critical, 1 Important, 4 Minor (mechanics verified OK). Folded below; plan-R1 required.

---

## VERBATIM REVIEW OUTPUT

## Adversarial Architect Review — plan-R0 gate

I traced the full event path against the real harness. The selection/confirm mechanics are correct; the test has one fatal assertion bug.

### CRITICAL

**C1 — The assertion `s.String() != share` fails even WITH the change (case mismatch).** The keypad appends every typed rune as **uppercase**: `k.Fragment = k.Fragment + string(unicode.ToUpper(r))` (`gui/gui.go:1036`). So `kbd.Fragment` becomes `MS10TESTSXXXX…4NZVCA9CMCZLW`. `codex32.New` accepts a *consistently* uppercase string — `setCase` (`codex32/checksum.go:132-153`) only rejects *mixed* case (see `TestBIPInvalidCase`, `codex32/codex32_test.go:264-280`), and `New` stores the input verbatim as `String{s}` (`codex32/codex32.go:119`); `String()` returns it verbatim (`codex32/codex32.go:390-391`). The test's `share` constant is **lowercase** (`ms10tests…`), so `got != share` is TRUE → `t.Errorf` fires → **the test FAILS after the one-line change too**. The plan's "passes only after the uncomment" claim is wrong. Fix: assert against the uppercase form, e.g. `if got := s.String(); got != strings.ToUpper(share)` (and import `strings`), or compare case-insensitively, or seed `share` uppercase.

### IMPORTANT

**I1 — The "fails without change" path panics; it does NOT return a `bip39.Mnemonic`.** Plan Step 2 / Self-Review claim the without-CODEX32 run routes to 24-word entry and returns a mnemonic so the type assertion fails. Actual trace: `Down,Down` caps at index 1 → `case 0,1` → `inputWordsFlow` (24 words). The first rune `m` is typed (uppercase "M"), `kbd.Update` returns true, then `updateValidBIP39Keys("M", …)` (`gui/gui.go:552`) calls `bip39.ClosestWord("M")` which returns `valid=false` (uppercase "M" < lowercase "abandon", `HasPrefix` false; `bip39/bip39.go:95-104`) → **`panic("invalid fragment")`** (`gui/gui.go:873`). A panic is still a failure, but the plan's stated failure *reason* is incorrect, which undermines the "fails for the right reason" gate. The iteration note loosely covers this, but the documented expectation should be corrected to "panics / does not return `codex32.String`."

### MINOR

- **M1 — Event mechanics are otherwise CORRECT.** `click` injects press+release (`gui/event_test.go:42-55`); `runes` injects `RuneEvent`s (68-76). With nil `FrameCallback` (matching `TestWordKeyboardScreen`, `gui_test.go:276-288`), `ChoiceScreen.Choose` (`gui/gui.go:1179`) drains both `Down` pairs in one frame via the `inp.Next(Up,Down)` loop (clamped at `len(Choices)-1`, line 1212), then consumes the `Button3` press/release across two frames via `chooseBtn` (`Button3`/`AltButton: Center`, line 1182) → returns choice 2. Remaining queue `[runes…, Button2 pair]` flows to `inputCodex32Flow`, whose `okBtn` is `Button2` (`gui/gui.go:628`). Buttons confirmed: ChoiceScreen=`Button3`, codex32 OK=`Button2`. Ordering is sound.
- **M2 — Vector + keypad coverage OK.** `ms10tests…4nzvca9cmczlw` is a clean positive vector (`codex32/codex32_test.go:11`, `mustFromString`). All chars are on alphabet `1234567890qwertyupasdfghjklzxcvnm` (`gui/gui.go:624`); no b/i/o. Codex32 flow never calls `updateValidKeys`, so no keys are disabled.
- **M3 — Regression risk: none.** `newInputFlow` is referenced only at `gui/gui.go:1313`; no test asserts exactly two Input-Seed choices.
- **M4 — One-line edit, DCO, PR mechanics all correct.** Comment stays well-formed with SLIP-39 commented (`gui/gui.go:1806`); `case 2:` already wired (1820-1824); `git commit -s` and `gh pr create --repo seedhammer/seedhammer --base main --head bg002h:feat/enable-codex32-input` are right (origin must be `bg002h/seedhammer`).

### Verdict: NOT-GREEN (1 Critical / 1 Important)

Open list to clear before code:
1. **C1:** Fix the assertion to expect the uppercase string the keypad actually produces (`strings.ToUpper(share)` or case-insensitive compare), plus the `strings` import.
2. **I1:** Correct the documented without-change expectation to "panics / returns non-`codex32.String`," not "returns `bip39.Mnemonic`."

Everything else (event injection, button choices, vector, keypad, regression, edit syntax, DCO/PR) is verified GREEN.

---

## FOLD DISPOSITION (added by main session; not part of verbatim review)

| Finding | Sev | Disposition |
|---|---|---|
| C1 — keypad uppercases; assert against `strings.ToUpper(share)` (+ import) | **Critical** | FOLDED — test now imports `strings`, types lowercase `share`, asserts `s.String() == strings.ToUpper(share)`; comment explains the keypad uppercases. |
| I1 — without-change path panics, not returns Mnemonic | **Important** | FOLDED — Step 2 expectation + self-review + test comment corrected to "panics in 24-word entry / does not return codex32.String". |
| M1–M4 | Minor | No change (verified correct). |

plan-R1 re-dispatch follows.
