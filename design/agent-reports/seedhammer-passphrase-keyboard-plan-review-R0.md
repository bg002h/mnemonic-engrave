# Slice 2: passphrase-keyboard-widget — IMPLEMENTATION PLAN R0 GATE REVIEW — R0

- **Stage:** mandatory implementation-plan R0 gate (0C/0I before any code).
- **Plan reviewed:** `design/IMPLEMENTATION_PLAN_seedhammer_passphrase_keyboard.md` (committed `50f8874`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `af3758a3917db0d2f`), read-only vs fork `06b57f3`.
- **Outcome:** **RED — 2 Critical + 2 Important + minors.** All C/I + cheap minors folded; re-dispatched R1.

> NOTE: verbatim architect output, recovered from the agent transcript; a working-notes preamble (detailed trace) omitted; the report proper is verbatim.

---

## VERBATIM REVIEW OUTPUT

**IMPLEMENTATION_PLAN R0 REVIEW**
**Plan:** `design/IMPLEMENTATION_PLAN_seedhammer_passphrase_keyboard.md` (committed `50f8874`)
**Reviewer:** opus architect (adversarial, read-only)
**Date:** 2026-06-18
**Base:** fork `main` `06b57f3`

---

## CRITICAL

**C-1: `"fmt"` import is unused — compile-fatal**
File: `gui/passphrase_keyboard.go` (Task 1 import block, plan line 100)
Evidence: The import block includes `"fmt"`. Searching every code block across Tasks 1-3, there is no `fmt.` call anywhere. The plan's justification ("fmt in Layout's reveal label") is incorrect — Layout uses `widget.Labelf(...)` (a seedhammer internal, not the stdlib `fmt` package). `gui.go` itself does not import `"fmt"`. `go build` will fail with `"fmt" imported and not used`.
Fix: Remove `"fmt"` from the import block in Task 1's code.

**C-2: `"seedhammer.com/gui/layout"` import is unused — compile-fatal path**
File: `gui/passphrase_keyboard.go` (Task 1 import block + Task 3 `var _` placeholder, plan lines 100-110 and 672)
Evidence: The `layout` package is imported. The ONLY usage in any code block is `var _ = layout.Rectangle{}` — a placeholder the plan itself says "remove if unused" (plan line 675). No actual code in Tasks 1-3 calls any `layout.*` symbol. The plan's Task 3 note says to remove the `var _` line if `layout` is genuinely unused — it IS genuinely unused. If the implementer follows the guidance and removes `var _`, the import becomes `"seedhammer.com/gui/layout" imported and not used`, a compile error. The plan is internally contradictory: it imports `layout` in the code block and then says to remove it. The correct fix is to remove the `layout` import from the import block entirely and not add the `var _` placeholder.

---

## IMPORTANT

**I-1: `ppKeyExtent` drops `assets.KeyBackspace.Bounds().Min.X` margin — rendering defect**
File: `gui/passphrase_keyboard.go`, `ppKeyExtent` function (plan lines 225-237)
Evidence: `assets.KeyBackspace` is declared with `Rect: alpha4.Rectangle{MinX: 2, MinY: 0, MaxX: 19, MaxY: 11}` (`gui/assets/embed.go:127`). So `assets.KeyBackspace.Bounds()` returns `image.Rect(2, 0, 19, 11)`: `Dx()=17`, `Min.X=2`. The shared `NewKeyboard` (gui.go:868) computes `bsWidth := bsb.Min.X*2 + bsb.Dx()` = 4 + 17 = 21 to center the icon within a cell that accounts for its visual margins. The plan's `ppKeyExtent` returns `image.Pt(b.Dx(), cell.Y)` = `image.Pt(17, cell.Y)` — omitting the `Min.X*2` margin. This makes the backspace cell 4px narrower than visually needed, causing the icon to appear clipped or off-center at render time.
Fix: In `ppKeyExtent`, for `ppBackspace`: `b := assets.KeyBackspace.Bounds(); return image.Pt(b.Min.X*2 + b.Dx(), cell.Y)`.

**I-2: Unused `allowBackspace bool` parameter in `adjust` — dead code and misleading**
File: `gui/passphrase_keyboard.go`, `adjust` function (plan line 451)
Evidence: `func (k *PassphraseKeyboard) adjust(allowBackspace bool)` — the parameter `allowBackspace` is declared but the function body never reads it (confirmed by inspection of all lines 451-473). The plan acknowledges this on line 499: "keep the param for signature parity but it is unused." In Go an unused parameter is not a compile error, but `go vet` with `go vet ./gui/...` (Step 7, plan line 717) may flag it. More importantly it is actively misleading: `Update` calls `k.adjust(true)` but the `true` has no effect. The spec's `Valid` already excludes empty-Fragment backspace; the parameter is truly vestigial and should be removed. Both call sites (`Update` line 362, `moveCol` line 423, `moveRow` line 438) pass `true`; changing to `adjust()` is a one-line cleanup.
Fix: Remove the `allowBackspace bool` parameter and update the three call sites to `k.adjust()`.

---

## MINOR

**M-1: `op` local variable in `passphraseFrame` shadows the `op` package import**
File: `gui/passphrase_keyboard_test.go`, `passphraseFrame` function (plan line 548)
Evidence: `op, _ := k.Layout(ctx, &descriptorTheme)` declares a local variable named `op` of type `op.Op`. If `passphrase_keyboard_test.go` imports `"seedhammer.com/gui/op"` for any reason (e.g. for `op.Op` in a type assertion), this shadowing would cause a `"op" declared and not used` or silent package-reference break. Even if `op` the package is not imported in that file (since the test file is `package gui` and doesn't need to explicitly import `op` for the return type), the variable name collision is a code-quality hazard. Fix: rename the local: `fop, _ := k.Layout(ctx, &descriptorTheme)` and `ctx.Frame(fop)`.

**M-2: `_ = dims` is dead code in `passphraseFrame`**
File: `gui/passphrase_keyboard_test.go` (plan lines 547, 549)
Evidence: `dims := ctx.Platform.DisplaySize()` followed immediately by `_ = dims`. The `dims` variable is never used. Remove both lines.

**M-3: `passphraseFrame` returns `string` but the `TestPassphrasePageCycleRender` caller ignores the return of the FIRST call**
File: `gui/passphrase_keyboard_test.go` (plan lines 686-690)
Evidence: `c := passphraseFrame(t, func(r *EventRouter) { ... })` is assigned, then immediately `_ = c`. The drive function inside is a no-op comment-only stub. This test body provides effectively zero coverage of page-cycle rendering via the `passphraseFrame` path — it only tests the second block (direct `k.page = 2` render). This is not a compile or runtime error but is a coverage gap / misleading test structure. The `c := passphraseFrame(...)` call at lines 686-689 is entirely useless. Recommendation: remove the dead `passphraseFrame` call and just keep the direct-render block.

**M-4: `moveCol` and `moveRow` infinite-loop guard uses `next == k.col` / `next == k.row` but `adjustCol` mutates `k.row` mid-loop**
File: `gui/passphrase_keyboard.go`, `moveRow` function (plan lines 431-445)
Evidence: `moveRow` starts with `next := k.row`. The loop guard `if next == k.row { return }` checks if `next` has wrapped back. BUT `k.adjustCol(next)` (when it returns false) may have a side effect: looking at `adjustCol`, if `found = false`, `k.row` is NOT mutated (the only `k.row = row` assignment is inside `if !k.Valid(key)` ... no wait: `found = true; k.row = row` is set whenever a valid key is found). If `adjustCol` returns false (no valid key in that row), `k.row` is unchanged (the `k.row = row` line at plan line 485 is only executed when `!k.Valid(key)` is false... wait let me re-read:

```go
func (k *PassphraseKeyboard) adjustCol(row int) bool {
    ...
    for i, key := range rows[row] {
        if !k.Valid(key) {
            continue
        }
        found = true
        k.row = row   // ← This MUTATES k.row for every valid key found
```

So `k.row = row` is set the FIRST time a valid key in `row` is found (and again for subsequent valid keys in the same row, but always to the same value `row`). If `found = false` (no valid keys), `k.row` is NOT changed. So in `moveRow`, if `adjustCol(next)` returns false, `k.row` remains unchanged, and the guard `next == k.row` (original `k.row`) is still comparing correctly. No infinite loop. My earlier analysis holds. This is actually fine — I was being overly cautious. Not a real issue. Disregard this minor.

**M-4 (revised): `ppPageSymbols` row 3 contains `'` (U+0027 apostrophe) and `"` (U+0022 quotation mark) — confirm font presence**
File: `gui/passphrase_keyboard.go`, `ppPageSymbols` const (plan line 115)
Evidence: The spec states all printable ASCII glyphs are in the font (`cmd/bitmapfont/main.go:32`). U+0027 and U+0022 are printable ASCII. No font regen needed. Not an error — confirming the plan's claim is correct. No action needed.

**M-5: `TestPassphraseDpadCommit` uses byte-length comparison**
File: `gui/passphrase_keyboard_test.go` (plan line 514)
Evidence: `len(k.Fragment)` is byte length, not rune count. For all-ASCII page 0 keys this is numerically equal to rune count, so the test works. However, if future pages contained multi-byte glyphs, the test would fail incorrectly. The correct idiom is `utf8.RuneCountInString(k.Fragment) != utf8.RuneCountInString(before) + 1`. For this widget (ASCII-only pages) this is not a bug, but it is fragile. Low priority — note for implementer awareness.

---

## Verdict

**RED — 2 Critical / 2 Important**

Open Critical issues:
- **C-1:** `"fmt"` unused import → compile error
- **C-2:** `"seedhammer.com/gui/layout"` unused import → compile error (when `var _` placeholder is removed as directed)

Open Important issues:
- **I-1:** `ppKeyExtent` backspace width drops `Min.X*2` margin → visible rendering defect
- **I-2:** `adjust(allowBackspace bool)` — parameter declared, never read, misleads callers, should be removed

**Required before any code:** Fold all C and I findings into the plan, re-persist the updated plan, and re-dispatch the R0 gate. The Criticals are compile-fatal; C-1 and C-2 together mean Task 1's file will not compile at all. I-1 would produce a visually broken backspace key. I-2 is a maintainability/correctness hazard. No implementation may begin until this review reaches 0C/0I.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 2C/2I/5m | **C-1** unused `"fmt"` import (Layout uses widget.Labelf, not stdlib fmt) → removed. **C-2** unused `"seedhammer.com/gui/layout"` import + the `var _ = layout.Rectangle{}` smell → both removed (final imports pinned: image/math/strings/utf8/assets/op/widget). **I-1** `ppKeyExtent` backspace width dropped the `Min.X*2` centering margin → `b.Min.X*2+b.Dx()` (matches NewKeyboard gui.go:868). **I-2** vestigial `adjust(allowBackspace bool)` param → dropped to `adjust()`, all call sites updated. Minors folded: M-1 `op`→`fop` local (no package shadow), M-2 dead `dims` removed, M-3 useless `passphraseFrame` call removed from TestPassphrasePageCycleRender. M-5 (len vs RuneCount) left — ASCII-only pages. Architect independently verified the case-preservation invariant, cross-page RuneEvent, geometry, panic-safety, and mask/reveal testability all sound. |

Re-dispatched R1 after the fold.
