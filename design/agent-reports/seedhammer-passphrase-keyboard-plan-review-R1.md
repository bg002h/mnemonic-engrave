# Slice 2: passphrase-keyboard-widget — IMPLEMENTATION PLAN R0 GATE REVIEW — R1

- **Stage:** plan gate round R1 (after folding R0's 2 Critical + 2 Important + minors).
- **Plan reviewed:** `design/IMPLEMENTATION_PLAN_seedhammer_passphrase_keyboard.md` (committed `735b402`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `a856fc3b902c6bef1`), read-only vs fork `06b57f3`.
- **Outcome:** **GREEN — 0 Critical / 0 Important.** Plan cleared for implementation.

> NOTE: verbatim architect output, recovered from the agent transcript; a working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

## R1 ARCHITECT REVIEW — Slice 2 Passphrase Keyboard Widget Implementation Plan

**Reviewer:** opus architect, R1 gate
**Date:** 2026-06-18
**Plan commit:** 735b402

---

### R0 Fold Verification

**C-1 (unused `fmt`):** CONFIRMED CLEAN. The Task 1 import block (`gui/passphrase_keyboard.go`) contains exactly `image`, `math`, `strings`, `unicode/utf8`, `seedhammer.com/gui/assets`, `.../op`, `.../widget` — no `"fmt"` import. The `Layout` implementation uses `widget.Labelf(...)` throughout; no `fmt.*` call appears in any code block across Tasks 1-3. The fold is complete and correct.

**C-2 (unused `layout` + `var _` smell):** CONFIRMED CLEAN. No `"seedhammer.com/gui/layout"` import appears in the implementation file. No `var _ = layout.Rectangle{}` line appears anywhere in the plan. No `layout.*` reference exists in any code block. The Task 3 comment explicitly documents the exclusion: `// "seedhammer.com/gui/layout" — this widget references no layout.* symbol.` Fold is complete and correct.

**I-1 (backspace width):** CONFIRMED CLEAN. `ppKeyExtent` for `ppBackspace` returns `image.Pt(b.Min.X*2+b.Dx(), cell.Y)` (line matching the comment `// R0 I-1: include the Min.X margin (matches NewKeyboard gui.go:868)`). This matches `NewKeyboard`'s `bsWidth := bsb.Min.X*2 + bsb.Dx()` at `gui.go:868`. The fix is present and matches the upstream formula exactly.

**I-2 (adjust param):** CONFIRMED CLEAN. `adjust()` has no parameter. All call sites are `k.adjust()` — in `Update` (start of function), `moveCol` (after finding valid key), and `moveRow` (after `adjustCol` succeeds). No stray `k.adjust(true)` or `k.adjust(false)` call exists anywhere. The note in Task 2 explicitly documents this. Fold is complete and correct.

**M-1 (fop not op):** CONFIRMED CLEAN. The `passphraseFrame` helper uses `fop, _ := k.Layout(ctx, &descriptorTheme)` and `ctx.Frame(fop)`. No package-shadowing of `op`.

**M-2 (dead `dims`):** CONFIRMED CLEAN. No `dims` variable appears anywhere in the plan.

**M-3 (TestPassphrasePageCycleRender):** CONFIRMED CLEAN. The Task 3 Step 5 test constructs its own `ctx`/`k`, sets `k.page = 2`, drives `runUI` directly, and calls `frame()` once. There is no vestigial `passphraseFrame(...)` call.

---

### Regression / Completeness Check

**Type/signature verification against 06b57f3:**

All primitives verified against live source files:
- `widget.Labelw(buf, st, width, col, txt)` — matches `label.go:16`. ✓
- `widget.Labelf(buf, st, col, fmt, args...)` — matches `label.go:20`. ✓
- `op.Input(&ctx.B, tag).Clip(r)` — `op.Input` at `op.go:154` returns `Op`; `Op.Clip` at `op.go:224`. ✓
- `mulAlpha(col, theme.inactiveMask)` — package-local at `gui.go:1274`. ✓
- `theme.inactiveMask` set to `0x55` in `theme.go:67`. ✓
- `assets.KeyBackspace` is `*alpha4.Image`; `.Bounds()` returns `image.Rectangle`; `.Min.X` and `.Dx()` are standard `image.Rectangle` methods. ✓
- `InputTracker.Next(ctx, filters...)` at `gui.go:105`. ✓
- `Event.AsButton() (ButtonEvent, bool)` at `event.go:236`. ✓
- `Event.AsRune() (RuneEvent, bool)` at `event.go:257`. ✓
- `Up/Down/Left/Right/Center` at `event.go:22-26`. ✓
- `Clickable.Clicked(ctx *Context) bool` at `widget.go:35`. ✓
- `ctx.Styles.keyboard` / `.word` are `text.Style` with `.Measure(maxWidth int, format string, args...)`. ✓
- Test helpers `NewContext`, `runUI`, `uiContains`, `descriptorTheme`, `runes`, `press` all verified in `gui_test.go` and `event_test.go`. ✓

**Case-preservation invariant:** The `commit` function uses `k.Fragment += string(key.r)` with no `unicode.ToUpper`. The `Layout` function uses `widget.Labelf(..., "%c", key.r)` with no `unicode.ToUpper`. Both `ToUpper` sites from the shared `Keyboard` are absent. The `RuneEvent` handler matches `key.r == e.Rune` directly (no `ToLower`). Case-preservation is structurally guaranteed.

**Cross-page RuneEvent:** The `Update` RuneEvent handler iterates `for _, page := range k.pages` (all 3 pages), then all rows, then all keys. It commits the first match and returns without modifying `k.page`. `TestPassphraseRuneEntryCrossPage` validates this for 'A' (page 1), '1'/'!' (page 2), 'b'/' ' (page 0).

**Geometry soundness:** Letter rows use uniform `letterW = cell.X + 2*keyPadX + margin`. Function row uses per-key widths from `ppKeyExtent`. Both are centered within `maxw`. The `gridY` offset in `Layout` correctly positions the grid below the readout. `k.size[p]` stores the grid extent; `combined` is `max(readoutSz.X, k.size[k.page].X)` × `gridY + k.size[k.page].Y`.

**adjust / adjustCol / moveCol / moveRow termination:** All loops have valid termination conditions. `moveCol` terminates because the row length is finite and the guard `if next == k.col` detects a full wraparound. `moveRow` terminates because `adjustCol` sets `k.row` only on success (which immediately returns), and the guard `if next == k.row` uses the pre-loop value of `k.row`. `adjust` is a finite double loop with no early exit.

**Panic safety:** `adjust` has the `!found` fallback that sets `k.row` / `k.col` to the last key, preventing OOB. `moveCol` and `moveRow` guard on `len(row)` and `len(rows)`. `commit` guards `k.Fragment != ""` before backspace. No nil dereferences possible in construction (all pages have 4 rows by construction).

**Shared Keyboard / 3 flows untouched:** The new file is `gui/passphrase_keyboard.go` with `gui/passphrase_keyboard_test.go`. No modifications to `gui.go`, `theme.go`, or any existing test. The guard-test step explicitly runs `./gui/...` to confirm.

**Import correctness:** Every import in the final file is used; no unused import exists. No `color` package import is needed (all `color.RGBA` values come from type inference through `th.Text` and function returns). No `text` package import is needed in the implementation file (no explicit type annotation). The test file only needs `"testing"` (no `op` import required because `op.Op` values are passed without explicit type annotation).

**Minor concern (non-blocking):** The `ppPageCycleLabel` `"?123"` contains a `?`, and the symbols row also has `?` as a key. If the `ppPageCycle` button's label `"?123"` were ever rendered through a `%s` format string where the renderer tried to parse format verbs, it would be a problem — but `widget.Labelf(..., "%s", key.label)` passes `key.label` as an argument, not as the format string. The `%s` is the literal format, and `key.label` is the argument. No issue.

---

### Verdict

GREEN — 0 Critical / 0 Important

All 6 R0 findings (C-1, C-2, I-1, I-2, M-1, M-2, M-3) are correctly and completely folded. No new defects were introduced by the fold. All type signatures, import sets, nav logic, geometry, and test helpers are verified against the live `06b57f3` codebase. The plan is consistent and implementable as written.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | RED — 2C/2I/5m | C-1/C-2 unused imports; I-1 backspace margin; I-2 adjust param; minors. All folded. |
| R1 | **GREEN — 0C/0I** | All folds verified clean (imports exactly image/math/strings/utf8/assets/op/widget; backspace `Min.X*2+Dx`; `adjust()` no param; `fop` not `op`; dead `dims`/useless test call gone). No new defects; every signature re-verified vs `06b57f3`; case-preservation + cross-page RuneEvent + geometry + panic-safety + termination + scope all confirmed. |

GATE PASSED at R1. Proceed to single-implementer TDD.
