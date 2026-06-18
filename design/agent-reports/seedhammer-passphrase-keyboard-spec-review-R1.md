# Slice 2: passphrase-keyboard-widget — SPEC R0 GATE REVIEW — R1

- **Stage:** spec gate round R1 (after folding R0's 2 Important + 6 Minor). FINAL.
- **Spec reviewed:** `design/SPEC_seedhammer_passphrase_keyboard.md` (committed `444cc36`).
- **Reviewer:** opus `feature-dev:code-architect` (agentId `a0d66d2a28b6f4965`), read-only vs fork `06b57f3`.
- **Outcome:** **GREEN — 0 Critical / 0 Important.** Spec cleared to the plan phase.

> NOTE: verbatim architect output, recovered from the agent transcript; a short working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

## R1 Architect Review — SPEC_seedhammer_passphrase_keyboard.md (post-R0 fold)

**Reviewer:** Opus architect, R1
**Spec commit:** 444cc36
**Fork base:** 06b57f3
**Scope:** verify all R0 folds landed correctly; full fresh read for regressions and completeness.

---

### R0 Fold Verification

**I-1 (RuneEvent model — cross-page, case-honoring, no auto-switch)**

The fold is present and coherent. §4.5 commits to: (a) no `ToLower`/`ToUpper` on the typed rune; (b) scan all three pages' `ppRune` keys for a literal match; (c) append the matched rune as-is; (d) do not switch the active page. The model is internally consistent: 'A' lives only on the UPPER page, '1' only on symbols, 'a' only on lowercase — each rune is unambiguous across pages, so the cross-page scan is well-defined and produces no collision. `runes(&ctx.Router, "Ab1!")` will yield Fragment == "Ab1!" because 'A' matches page 1, 'b' matches page 0, '1' matches page 2, '!' matches page 2, all appended as-is. This is coherent with the touch/D-pad model being page-scoped (clicking only reaches visible-page keys). No contradiction. FOLD CORRECT.

**I-2 (function-row key model — ppKey/ppAction + per-action dispatch + D-pad integration + cell sizing)**

§4.3 defines `ppAction`, `ppKey`, `Valid`, `Layout`-dispatch, and `Update`-dispatch explicitly. The function row is the last row of each page's `keys [][]ppKey`, so D-pad Up/Down wraps into it exactly as the shared `Keyboard` does (the existing D-pad code iterates by row index, wraps via modulo — same pattern will apply). `ppBackspace` validity is `Fragment != ""`. Function-row cell widths are per-label via `ctx.Styles.keyboard.Measure(math.MaxInt, label) + keyPadX`. Reuse of `keyPadX`/`keyPadY`/`keyCornerRadius`/`keyLineWidth` at `gui.go:49-57` is confirmed (those are package-level constants, accessible in the same package). `assets.KeyBackspace` used as the backspace icon (confirmed present in `gui/assets/embed.go`). `Clickable` is in `widget.go`, accessible. FOLD CORRECT.

**M-1 (Clear resets revealed=false)**

§4.1 Clear() signature comment reads: `reset Fragment="", page=0, cursor to center, AND revealed=false (re-mask — R0 M-1)`. Present and explicit. FOLD CORRECT.

**M-2/M-3 (function-row cells sized to labels)**

§4.3 final paragraph: "sized to each label (`ctx.Styles.keyboard.Measure(math.MaxInt, label)` + `keyPadX`)". Explicit. FOLD CORRECT.

**M-4 (space glyph / uiContains note)**

§6 last bullet: "the space glyph (U+0020) renders with no pixels, so `ExtractText` does not collect it; `uiContains` also strips spaces from its needle." The `uiContains` implementation at `gui_test.go:479-484` confirms: it applies `strings.ReplaceAll(..., " ", "")` only to the **needle** (`str`), not to `txt` (the haystack). The note says "uiContains strips spaces from its needle" — that is accurate. Fragment content that contains spaces will still be detected by a needle that omits spaces. FOLD CORRECT.

**M-5 (keyPad consts cited)**

§4.3: "Reuse `keyPadX`/`keyPadY`/`keyCornerRadius`/`keyLineWidth` (`gui.go:871-876`)." The spec cites `gui.go:871-876` but the constants are defined at `gui.go:49-57`. The cited lines (871-876) are inside `NewKeyboard` where those constants are *used*. The spec author likely cited the usage site (as a pointer to where these are applied) rather than the declaration site. This is imprecise — a plan author following the cite literally would find the constants applied there but not declared there, and would need to grep to find the declaration at `:49-57`. However, the constant names are unambiguous, and `NewKeyboard` at that range does clearly show them in action, so a competent implementer will not be blocked. This is a documentation imprecision, not an architectural defect.

**M-6 (Layout returns combined readout+grid extent)**

§4.1 `Layout` signature comment: "the returned image.Point is the COMBINED extent (readout height + grid), so the Slice-3 flow places the whole widget as one block". Present. FOLD CORRECT.

---

### Fresh Full-Spec Read — Regression and Completeness

**Page-cycle cursor re-seed**

§4.3 Update dispatch: "`ppPageCycle`→`page=(page+1)%3` + re-seed the cursor." The spec mentions re-seeding but does not say what "re-seed" means. Looking at the shared `Keyboard.Clear()` at `gui.go:916-920`: it sets `row = len(k.keys)/2`, `col = len(k.keys[k.row])/2` — i.e., center of the grid. The spec does not specify whether the cursor is reset to center (mimicking `Clear`), kept at the same logical position on the new page, or clamped to a valid key. For a 3-page keyboard where all pages share an identical function row at the bottom and have similar row counts, "re-seed to center" is the natural interpretation — but it is not stated. A plan author must invent this detail. This is a genuine gap.

Assessment: the shared `Keyboard.Clear()` center-reset is the obvious default, and the function-row row count is identical across pages, so the gap is very narrow. A competent plan author will trivially pick "reset to center row/col of the new page" or "reset to row=0, col=0". It does not affect correctness (any valid starting cursor position is fine). This is a MINOR gap, not an Important.

**Three separate page grids vs one grid with swapped rows**

§4.3: "All three page grids are `[][]ppKey` sharing the SAME function row as their last row, stored in the active page's `keys [][]ppKey` slice." The spec says "three page grids" sharing one function row. It does not specify the storage layout: does the struct hold three `[][]ppKey` slices (one per page, each terminating in the shared function row), or one function row slice plus three page-letter-row slices, assembled on demand? The sentence "stored in the active page's `keys [][]ppKey` slice" implies three full slices, each ending in the same function-row slice. This is implementable and unambiguous enough. A plan author can choose the concrete layout (e.g., `[3][][]ppKey` with the function row as the final `[]ppKey` appended to each, or shared via a single pointer). The spec does not need to go deeper here.

Assessment: Sufficient. Not a defect.

**Function-row geometry vs letter rows**

§4.3 specifies letter-row cells use the fixed-width `bgsz` (from `Measure("W") + keyPadX/Y`), while function-row cells use per-label widths. The spec says function-row cell widths are `Measure(math.MaxInt, label) + keyPadX` — but does not specify the height. Looking at the shared `Keyboard`: height comes from `widest.Y` (the `Measure("W").Y`), which is the glyph height from `poppins.Bold25`. For the function row it is natural to use the same `keyPadY`-padded height as letter rows (same font, same style). The spec does not state this explicitly, but it is the only sensible choice. Not a defect — implementer cannot be confused here.

**Layout readout placement**

§4.4: readout rendered "above the key grid" via `widget.Labelw`. §4.1: Layout returns the COMBINED extent. The spec does not specify the vertical gap between the readout and the key grid, or the width constraint for the readout (the codex32 flow uses `dims.X - 50` at `gui.go:700`). The plan author will need to choose this. Minor detail, not a blocker.

**`ppKey.label` for `ppRune` keys**

§4.3: `ppKey` has both `r rune` and `label string`. For `ppRune` keys, `label` is unused (the Layout dispatch uses `"%c", k.r`). The struct is fine; an implementer will leave `label` empty for `ppRune` keys. No defect.

**Page-cycle key label consistency check**

§4.3: "cap shows the *target* page: `"ABC"` (on the lowercase page) / `"?123"` (on UPPER) / `"abc"` (on symbols)." So: page 0 shows `"ABC"` (target=UPPER), page 1 shows `"?123"` (target=symbols), page 2 shows `"abc"` (target=lowercase). The label `"?123"` on UPPER page pointing to symbols (which has digits and symbols) is sensible. `"abc"` on symbols pointing to lowercase is sensible. This is internally consistent and the exact labels are called out for the plan. GOOD.

**`InputTracker` — spec mentions it in §4.1 struct comment but does not name it in the `ppKey`/struct definition**

§4.1 struct comment says "plus row/col cursor, InputTracker, layout extents — reusing the keyboardKey cell model." `InputTracker` is referenced by name in the struct comment but not shown as a field name in the struct body. The shared `Keyboard` has `inp InputTracker` at `gui.go:849`. An implementer will add `inp InputTracker` as a field — this is unambiguous. Not a defect.

**Slice-2 boundary — MnemonicSeed/flow/fingerprint excluded**

§1, §2, §7, §8: all consistent. `gui.go:188` `MnemonicSeed(m, "")` explicitly noted as untouched. Slice-3 boundary is clean. GOOD.

**Shared Keyboard / three consumers untouched**

§2: explicit. `TestWordKeyboardScreen`, `TestInputSeedCodex32`, SLIP-39 tests must stay green. The new type adds a new file, touches nothing in `Keyboard`/`keyboardKey`/existing flows. GOOD.

**`widget.Labelw` for the readout (§4.4)**

`widget.Labelw` is confirmed at `widget/label.go:16` — it takes `(buf, style, width, col, txt)`. The call in §4.4 will be something like `widget.Labelw(&ctx.B, ctx.Styles.word, <width>, col, maskedStr)` — consistent with how `gui.go:700` uses it. The width constraint for the readout is not specified (a minor plan-authoring detail).

**Symbol page charset (§4.2)**

The 30 symbols listed: `1234567890`, `-/:;()&$@"`, `.,?!'+=_#`. Spec notes "printable-ASCII; the exact glyph set is finalized in the plan — all are font-present." The font is `poppins.Bold25`, generated from the full printable-ASCII range (`cmd/bitmapfont/main.go:32` cited in §3). This is consistent and deferred to the plan. No defect.

**`runes` test helper in `event_test.go:68-76`**

Confirmed: `runes(r *EventRouter, str string)` fires one `RuneEvent` per rune, with the rune as typed. So `runes(&ctx.Router, "Ab1!")` fires runes 'A', 'b', '1', '!' in order. Under the cross-page model, each is found on its respective page and appended as-is → Fragment "Ab1!". The test assertion in §6 is achievable. GOOD.

**`uiContains` and mask test (§6)**

The mask test asserts `uiContains(content, "****")`. `uiContains` strips spaces from the needle but NOT from the haystack. `ExtractText` collects rendered glyphs — `*` is printable ASCII and will be rendered by `poppins.Bold25`, so it will be collected. `uiContains("****", "****")` → needle `"****"` (no spaces), searches in haystack. If the readout renders 4 asterisks, `ExtractText` collects `"****"` and `strings.Contains(txt, "****")` is true. The test is feasible. GOOD.

**M-5 cite imprecision (constants declared at :49-57, cited as :871-876)**

This is an editorial imprecision — the plan author will find the constants quickly, and the values are already quoted in the spec. This does not impede a compile-accurate implementation.

---

### Open Defect Summary

No Critical issues.
No Important issues.

The only residual items are minor editorial/detail gaps that do not block a plan author from writing compile-accurate code:

- MINOR (§4.3): cursor re-seed strategy on page-cycle is not specified (center vs retain vs clamp). Trivially resolved by plan author following the `Keyboard.Clear()` pattern.
- MINOR (§4.4): readout width constraint and vertical gap to the grid are not specified. Plan detail.
- MINOR (§4.3): `keyPad*` constants are cited at their usage site (:871-876) rather than their declaration site (:49-57). Imprecision only.

None of these rise to Important because a competent plan author can resolve each with a single-line decision, and there is no risk of a structurally incorrect implementation resulting from the ambiguity.

---

**VERDICT: GREEN — 0 Critical / 0 Important**

All R0 Important findings (I-1, I-2) are correctly and completely folded. All six Minors (M-1 through M-6) landed correctly. The spec is internally consistent: the `ppKey`/`ppAction` naming is used uniformly throughout §4.3; the cross-page RuneEvent model does not contradict the page-scoped touch/D-pad model; the masked readout, `Clear` semantics, `Layout` return contract, and function-row geometry are coherent. Slice-2 scope boundaries are clean. The shared `Keyboard` and its three live consumers are untouched by design. A competent plan author can now write compile-accurate code against `06b57f3` with no unresolved architectural fork.


---

## Loop summary (added by main session)
| Round | Verdict | Notes |
|---|---|---|
| R0 | NOT GREEN — 0C/2I/6m | I-1 RuneEvent cross-page; I-2 ppKey model; + 6 minors. All folded. |
| R1 | **GREEN — 0C/0I** | All R0 folds verified correct (cross-page RuneEvent coherent, ppKey/ppAction + per-action dispatch + same-slice D-pad + function-row sizing implementable against real primitives, Clear re-masks, Layout combined extent). 3 residual Minors (page-cycle cursor re-seed; readout width/gap; keyPad consts cite at usage not decl) — 2 fixed post-R1 as doc-precision (cursor→center per Clear; cite corrected to :49-57); readout geometry left to the plan. |

GATE PASSED at R1 (2 doc-precision fixes folded after, no code/C-I impact). Proceed to the implementation plan.
