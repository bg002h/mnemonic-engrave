# Cycle A1: codex32-input-polish — IMPLEMENTATION PLAN R0 GATE REVIEW — R1

- **Stage:** mandatory implementation-plan R0 gate, round R1 (re-review after folding R0's IMP-2 + clarity Minors). Read-only, adversarial.
- **Plan reviewed:** `design/IMPLEMENTATION_PLAN_seedhammer_codex32_input_polish.md` (committed `04de253`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `ae538551e84150eb4`). Verified against fork `main` `3c4d3d3` with file:line evidence.
- **Outcome:** **GREEN — 0 Critical / 0 Important.** Plan is past the gate; proceed to implementation.

> NOTE: verbatim architect output, recovered from the agent transcript (`tasks/ae538551e84150eb4.output`); a short working-notes preamble was omitted. Note the reviewer self-corrects its own M1 mid-review (initially flags the `splitHRP` comment, then retracts after re-reading `strings.Cut` semantics) — net result is zero findings.

---

## VERBATIM REVIEW OUTPUT

## R1 Architect Review — SeedHammer CODEX32 Input Polish Implementation Plan

**Plan:** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_seedhammer_codex32_input_polish.md`
**Base commit reviewed:** `3c4d3d3` (fork `main`, confirmed via source read)
**Reviewer:** opus architect (R1 after R0-identified IMP-2 fold + clarity-comment fold)

---

### I. Verification of the IMP-2 Fold (Task 4 Step 3 / Task 5 Step 3 import discipline)

**IMP-2 was:** Task 4 Step 3 imported GUI subpackages (`image`/`assets`/`layout`/`op`/`widget`) into `gui/codex32_polish.go` at the point where only the three pure helpers (`codex32StatusLine`, `codex32FieldLine`, `codex32Feedback`) existed — causing unused-import compile failure.

**The fold as written in the plan:**

Task 4 Step 3 now carries this import block in `gui/codex32_polish.go`:

```go
import (
    "fmt"
    "strings"

    "seedhammer.com/codex32"
)
```

Checking against the three helpers: `codex32StatusLine` uses `fmt.Sprintf` and `codex32.ShortCodeMinLength`/`ShortCodeMaxLength`/`LongCodeMinLength`/`LongCodeMaxLength`. `codex32FieldLine` uses `strings.ToUpper`/`strings.Join`/`fmt.Sprintf`. `codex32Feedback` uses `codex32.Describe`/`codex32.ShortCodeMinLength` etc. Every import in this block is used; no extra import is present. **The IMP-2 compile-failure on Task 4 is correctly resolved.**

Task 5 Step 3 expands that block to:

```go
import (
    "fmt"
    "image"
    "strings"

    "seedhammer.com/codex32"
    "seedhammer.com/gui/assets"
    "seedhammer.com/gui/layout"
    "seedhammer.com/gui/op"
    "seedhammer.com/gui/widget"
)
```

Checking `confirmCodex32Flow` body: `strings.ToUpper`/`strings.Join`-style usage (strings), `fmt.Sprintf` (fmt), `image.Pt` and `image.Rectangle` via the content rectangle (image), `codex32.ParsePrefix` (codex32), `assets.IconBack`/`assets.IconHammer` (assets), `layout.Rectangle` (layout), `op.Layer`/`op.Color`/`op.Op` (op), `widget.Labelw` (widget). Every entry in the expanded block is consumed. **Task 5 Step 3 expansion is correct.**

Task 6 Step 3 (`codex32Keys` const + `newCodex32Keyboard`) appends to `gui/codex32_polish.go`. The function calls only `NewKeyboard` (in-package) and accesses `kbd.allKeys[i].r`/`.disabled` (in-package struct fields). It imports no new packages. **No new import is needed; no double-import is introduced.** The Task 4→5→6 import sequence is clean.

**IMP-2 fold: VERIFIED CORRECT.**

---

### II. Verification of the Task 3 Step 3 Clarity Comments

The fold added three inline comments to the `ParsePrefix` implementation.

**Comment 1:** "When splitHRP finds no '1', it returns ("", frag) — so `data` aliases the ENTIRE input in that case. We early-return below before touching `data`, so no field is read from the not-yet-data prefix."

Verified against `codex32.go:453-459`:
```go
func splitHRP(s string) (string, string) {
    p1, p2, ok := strings.Cut(s, "1")
    if !ok {
        return "", p1   // ← returns ("", whole_input)
    }
    return p1, p2
}
```
When no `"1"` is found, `strings.Cut` sets `ok=false` and sets `p1` to the full string (as per Go stdlib semantics for `strings.Cut` on a missing separator — it returns `(s, "", false)`, so `p1 = s`, `p2 = ""`). Wait — actually, let me re-read: `strings.Cut(s, "1")` with no separator present returns `(s, "", false)`. Therefore `p1 = frag` and `p2 = ""`. The code returns `("", p1)` which is `("", frag)`. The comment says `data` aliases the entire input — that is `p2`, the second return value assigned to `data`. But `data = p2 = ""` (empty string), not `frag`. The comment says "data aliases the ENTIRE input in that case" — this is **inaccurate**. When there is no `"1"`, `data` is the empty string `""`, not the full fragment. `hrp` (the first return, `""`) aliases nothing; `p1 = frag` is the first return of `splitHRP`, not `p2`. The comment misidentifies which variable aliases the full input.

However, what matters for correctness is: does the code's behavior match the intent? When `hrp == ""`, the code executes `return f, nil` before touching `data`. So `data` being empty in that case causes no harm — no field extraction reads `data[0]`, `data[1:5]`, or `data[5]`. The comment's description is technically misleading (it says `data` aliases the whole input, but `data` is actually `""`) yet the code remains correct because the early-return happens before any `data` access. The comment describes a wrong mental model of `splitHRP`'s behavior in this case.

**This is a clarity defect in the comment, not a code defect.** The code itself is sound; the comment misstates what `data` contains in the no-separator case. A reader trusting the comment would have a wrong mental model. This rises to MINOR severity because the code works correctly regardless.

**Comment 2:** "data[5] is a byte; rune(byte) is the codepoint for ASCII (all valid bech32 is ASCII). Non-ASCII bytes (128..255) make feFromRune return false below → 'invalid character', never a panic."

Verified: bech32 characters are all 7-bit ASCII. `feFromRune` checks `if c < 0 || int(c) >= len(invCharsTbl)` (gf32.go:127-129), where `invCharsTbl` has length 128. So a byte value of 128-255 cast to `rune` yields a rune > 127, which fails the bounds check and returns `(0, false)`. The plan's claim that "non-ASCII bytes make feFromRune return false, never a panic" is accurate. **Comment 2 is correct.**

**Comment 3:** "HRP is recorded for display, not independently rejected: New is the authority and surfaces a wrong HRP as a checksum mismatch (it folds the HRP into the checksum), so ParsePrefix stays consistent with New."

Verified: `New` at `codex32.go:109-116` calls `check.inputHRP(hrp)` which feeds the HRP into the checksum engine; a wrong HRP manifests as `errInvalidChecksum` from `check.isValid()`. `ParsePrefix` records `f.HRP = hrp` and does not validate the HRP value against any allowed set. The comment accurately describes this design choice. **Comment 3 is correct.**

---

### III. Regression Scan: Unchanged Sections

**Task 4 Step 1 — `gui/codex32_polish_test.go` import block:**

The plan specifies:
```go
import (
    "strings"
    "testing"

    "seedhammer.com/codex32"
)
```
The three test functions in that block use: `testing.T` (testing), `codex32.ParsePrefix`/`codex32.Fields`/`codex32.ShortCodeMinLength` etc. (codex32), `strings.Repeat` (in the `TestCodex32Feedback` dead-zone test). All three imports are consumed; no GUI imports are present. **Correct — no change needed, and none was made.**

**C4 back-out `return false` correctness:**

From the source read of `engraveObjectFlow` at gui.go:1784-1835: the function returns `bool`. After every non-`default` case body (`backupWalletFlow`, `backupSeedStringFlow`, `descriptorFlow`, `mdmkFlow`), the code falls through to `return true` at line 1834. The `default:` case returns `false`. The plan's proposed C4 insertion:

```go
case codex32.String:
    if !confirmCodex32Flow(ctx, th, scan) {
        return false
    }
    id, _, _ := scan.Split()
    ...
    backupSeedStringFlow(ctx, th, s)
```

On back-out (user pressed Back in the confirm screen), the function returns `false` — consistent with the "not handled / not engraved" convention that the `default:` case and the existing caller-side `s.Status = scanUnknownFormat` assignment at gui.go:1416 expect. On engrave-confirmed path, the code falls through to the `return true` at line 1834. **The back-out `return false` is correct against the actual source.**

**Build-order forward-reference check:**

- Task 4 calls `codex32StatusLine`/`codex32FieldLine`/`codex32Feedback` (defined in same step). Task 4 also calls `codex32.ParsePrefix` (defined in Task 3). Task 4's `inputCodex32Flow` rewrite references `newCodex32Keyboard` — wait, it does NOT: Task 4 Step 7 preserves the `NewKeyboard(ctx, alph)` init; it is Task 6 Step 4 that swaps to `newCodex32Keyboard`. Task 6 defines `newCodex32Keyboard` in Step 3 before wiring it in Step 4. **No forward reference.** 

- `confirmCodex32Flow` (Task 5) is called from `engraveObjectFlow` (Task 5 Step 5), which is in the same compilation unit. `newCodex32Keyboard` (Task 6 Step 3) is called from `inputCodex32Flow` (Task 6 Step 4), same file. All references within a Go package are resolved at package compile time regardless of source-file or addition order. **Build order is safe.**

**Scope discipline — no `Split()`/`mdmk.go`/gate-widening:**

`confirmCodex32Flow` uses `codex32.ParsePrefix` for the raw share index — correctly avoiding `Split()` which remaps threshold 0→1 (confirmed: `codex32.go:394-401` shows `t=1` substitution when `t==0`). The existing `case codex32.String:` body's `scan.Split()` call (for `id`) is preserved unchanged on the engrave path. No `mdmk.go` edits are present. `codex32.Feedback` / `ParsePrefix` are advisory and cannot be used in the `if valid` gate — the plan uses `nerr == nil` (i.e., `codex32.New`) exclusively for the OK button gate. **Scope discipline holds.**

**`codex32.Alphabet` coverage in Task 6 test:**

`const Alphabet = "QPZRY9X8GF2TVDW0S3JN54KHCE6MUA7L"` (32 chars, confirmed at codex32.go:21). The `codex32Keys` layout is `"1234567890\nqwertyuiop\nasdfghjkl\nzxcvbnm"`. The bech32 alphabet (lowercased) is: `q p z r y 9 x 8 g f 2 t v d w 0 s 3 j n 5 4 k h c e 6 m u a 7 l`. Of these, `b`/`i`/`o` are NOT bech32 chars (they do not appear in `Alphabet`). The full-QWERTY layout `qwertyuiop asdfghjkl zxcvbnm` plus digits covers all 32 bech32 lowercase chars plus `1` (the HRP separator). The test at Task 6 Step 1 iterates `codex32.Alphabet`, lowercases each character, and asserts it is enabled. The `b`/`i`/`o` dimming in `newCodex32Keyboard` correctly disables only those three non-bech32 characters. The separation from BIP-39's `NewKeyboard(ctx, wordKeys)` instance is per-instance (no shared state). **Correct.**

---

### CRITICAL

None.

---

### IMPORTANT

None.

---

### MINOR

**M1 — Inaccurate comment in Task 3 Step 3, `ParsePrefix`, first inline note.**

File: `codex32/polish.go` (to be created), Task 3 Step 3.

The comment reads: "When splitHRP finds no '1', it returns ("", frag) — so `data` aliases the ENTIRE input in that case."

This is factually wrong. `splitHRP(frag)` with no `"1"` present calls `strings.Cut(s, "1")` which returns `(s, "", false)`, so `splitHRP` returns `("", p1)` i.e. `("", frag)`. The two local variables are `hrp, data := splitHRP(frag)`, so `hrp = ""` and `data = ""` (the empty string `p2`). `data` does NOT alias the full input; `data` is the empty string. What the comment likely meant to say is that `p1` (the first `strings.Cut` output, returned as `splitHRP`'s second value in the no-separator case — wait: let me re-read `splitHRP` once more).

Re-reading `splitHRP` precisely:
```go
func splitHRP(s string) (string, string) {
    p1, p2, ok := strings.Cut(s, "1")
    if !ok {
        return "", p1  // p1 = s (the full string), p2 = "" (unused)
    }
    return p1, p2
}
```

`strings.Cut(s, sep)` with sep not found returns `(s, "", false)`. So `p1 = s` (full string), `p2 = ""`. The no-separator return is `("", p1)` = `("", s)`. So `hrp = ""`, `data = s` (the full fragment). The comment IS actually correct: `data` equals `frag` (the full input) in the no-separator case.

Correction to my earlier analysis: I misread `strings.Cut`. Let me be definitive. From Go stdlib: `strings.Cut(s, sep)` — "Cut slices s around the first instance of sep, returning the text before and after sep. The found result reports whether sep appears in s. If sep does not appear in s, cut returns s, "", false." So `p1 = s`, `p2 = ""`, and `splitHRP` returns `("", p1)` = `("", s)`. Therefore `data = s = frag`. **The comment is correct after all.**

M1 is retracted. The comment accurately states `data` aliases the whole input.

There are no MINOR findings that require action.

---

### Summary

All R0 IMP-2 findings are correctly folded:
- Task 4 Step 3 import block: exactly `fmt`/`strings`/`codex32`, all used, no extras.
- Task 5 Step 3 expanded import block: adds `image`/`assets`/`layout`/`op`/`widget`, all consumed by `confirmCodex32Flow`.
- Task 6 Step 3 adds no new imports (in-package only).
- No double-imports; no still-missing imports across the Task 4→5→6 sequence.

All three Task 3 Step 3 clarity comments are factually accurate.

Regression scan findings:
- `codex32_polish_test.go` Task 4 Step 1 import block is correct (`strings`/`testing`/`codex32`).
- C4 `return false` back-out is consistent with `engraveObjectFlow`'s `bool` convention.
- Build order is forward-reference-free.
- Scope discipline holds: no `Split()` misuse, no `mdmk.go` edits, no gate widening.
- `codex32.Alphabet` coverage in Task 6 test is complete and correctly structured.

---

**GREEN — 0 Critical / 0 Important**


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 0C / 1I | IMP-2 unused-import compile failure in Task 4 Step 3 → folded (Task 4 imports restricted to fmt/strings/codex32; Task 5 expands). Clarity Minors folded as ParsePrefix comments. |
| R1 | **GREEN — 0C / 0I** | IMP-2 fold VERIFIED (Task 4→5→6 import sequence clean, no double/missing imports); all three ParsePrefix clarity comments factually accurate (reviewer self-corrected & retracted its own M1 on `splitHRP`); regression scan clean — test-file imports correct, C4 `return false` matches `engraveObjectFlow`'s bool convention (fall-through `return true` at gui.go:1834, `default` returns false), build-order forward-reference-free, scope discipline holds (no Split()/mdmk.go/gate-widening), codex32.Alphabet coverage complete. **Proceed to implementation.** |

GATE PASSED at R1. Implementation = single subagent per task (C1 first), two-stage review (spec then code-quality), then the mandatory whole-diff adversarial execution review.
