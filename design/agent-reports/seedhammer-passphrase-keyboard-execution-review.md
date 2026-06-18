# Slice 2: passphrase-keyboard-widget — WHOLE-DIFF ADVERSARIAL EXECUTION REVIEW

- **Stage:** mandatory post-implementation execution review over the whole diff (ultracode phase 4). Read-only; ran toolchain + adversarial scratch probes.
- **Diff reviewed:** branch `feat/passphrase-keyboard`, base `06b57f3` … head `fd8714e` (2 implementation commits). 2 new files.
- **Reviewer:** `feature-dev:code-reviewer` (agentId `afa93262866c49714`).
- **Outcome:** **SHIP-READY — 0 Critical / 0 Important.** M-1 folded; M-2 declined (see fold note); M-3/M-4 coverage notes.

> NOTE: verbatim reviewer output, recovered from the agent transcript; a working-notes preamble omitted.

---

## VERBATIM REVIEW OUTPUT

## ADVERSARIAL EXECUTION REVIEW — Slice 2 Passphrase Keyboard Widget

**Reviewing:** `feat/passphrase-keyboard` diff vs `06b57f3`, files `gui/passphrase_keyboard.go` + `gui/passphrase_keyboard_test.go` only.

**Scope check:** `git diff 06b57f3..HEAD --name-status` shows exactly 2 new files. Confirmed.

---

### CRITICAL

None.

---

### IMPORTANT

None.

---

### MINOR

**M-1: Reveal key cell width measured from "show" only; "hide" label is unchecked**
- File: `/scratch/code/shibboleth/seedhammer-wt-passphrase/gui/passphrase_keyboard.go:133-141`
- `ppKeyExtent` is called at construction time with `{label: "show", action: ppReveal}`. It returns `widthOf("show")` as `key.size.X`. At render (Layout lines 366-371), the label dynamically becomes `"hide"` when `k.revealed`. If `"hide"` were wider than `"show"` in poppins.Bold25, the glyph overflows the cell and visually overlaps the adjacent backspace key; the touch-clip stays at `widthOf("show")`, so only the inner portion of the "hide" label is tappable. In poppins Bold 25pt, `"show"` contains `'w'` (wide) vs `"hide"` containing `'i'` (narrow), so `"show"` is empirically wider and this is benign today. But it is structurally fragile if labels change. Fix: measure `max(widthOf("show"), widthOf("hide"))` in `ppKeyExtent`. No test covers the "hide" render width. Confidence: 80 (real structural issue, not a current crash, depends on font metrics the test suite doesn't prove).

**M-2: `Measure(math.MaxInt, "%s", lbl)` deviates from codebase convention**
- File: `/scratch/code/shibboleth/seedhammer-wt-passphrase/gui/passphrase_keyboard.go:140`
- Every other `Measure` call in the codebase passes the string directly as the format argument: `Measure(math.MaxInt, "W")`, `Measure(math.MaxInt, "24: ")`. The new code uses `Measure(math.MaxInt, "%s", lbl)`. Functionally equivalent for the current label set (none of "ABC", "?123", "abc", "space", "show", "hide" contain `%`). The plan document at Task 1 Step 3 line 236 shows the direct form `Measure(math.MaxInt, lbl)`. The implementer flagged this explicitly. Not a vet or runtime issue. Fix: `ctx.Styles.keyboard.Measure(math.MaxInt, lbl).X` (consistent with all other sites). Confidence: 82 (real deviation from project convention, explicitly flagged by implementer, directly referenced in the plan).

**M-3: `TestPassphraseDpadCommit` uses byte-length check instead of rune-count check**
- File: `/scratch/code/shibboleth/seedhammer-wt-passphrase/gui/passphrase_keyboard_test.go:99`
- `len(k.Fragment) != len(before)+1` counts bytes, not runes. All keys on all three pages are ASCII (1 byte each), so this is correct for the current widget. The assertion would silently mis-report if a multi-byte character were ever added. Low risk but worth noting for Slice 3 evolution. Not a current functional bug.

**M-4: D-pad navigation into the function row is not directly tested**
- File: `/scratch/code/shibboleth/seedhammer-wt-passphrase/gui/passphrase_keyboard_test.go` (gap, no single line)
- No test drives `Down` or `Up` via `press()` to navigate from the bottom letter row into the function row and then commits a function-row key via Center. The `moveRow` path for crossing into the function row exercises `adjustCol` on a row containing `ppBackspace` (invalid when Fragment is empty), but that's the only invalid-key scenario in `adjustCol`. `TestPassphraseActions` covers `commit(ppPageCycle)` directly but not the D-pad path to it. Coverage gap acceptable for the widget-only slice; noting for Slice 3 integration.

---

### POSITIVE CONFIRMATIONS (things the green suite does NOT disprove but the review confirms are correct)

- **Case preservation end-to-end:** commit path at line 173 uses `string(key.r)` with no transform; render path at line 375 uses `"%c", key.r` with no transform. No `unicode.ToUpper` or `unicode.ToLower` anywhere in the file. ✓
- **Cross-page RuneEvent rune uniqueness:** lower/UPPER/symbols page alphabets are disjoint char sets. Space rune (' ') appears in the function row of all three pages but is deterministically matched on page 0 first — the commit result is identical regardless of which page's space key is matched (all have `r=' '`). ✓
- **Panic-safety:** `moveCol` terminates after one full loop (`next == k.col` detection); `moveRow` terminates when target row has valid keys (always true — every row has ≥1 ppRune key) or when it wraps back to original row. `adjustCol` is safe. `adjust` handles the "no valid key" case via the `!found` fallback. `commit(ppBackspace)` guards `if k.Fragment != ""`. No panic paths found. ✓
- **Layout geometry:** `key.pos` for letter rows is `x + j*letterW + keyPadX` (glyph origin inside the cell). Touch-clip is `{Max: key.size}` = glyph extent (same pattern as reference `Keyboard.Layout`). Function row key positions are advanced by `cw = size.X + 2*keyPadX + margin` per key, consistent with `ppRowWidth`. Grid + readout offsets are non-overlapping. ✓
- **Mask/reveal:** `Layout` computes `shown = strings.Repeat("*", utf8.RuneCountInString(k.Fragment))` when `!k.revealed`. `Clear()` resets `revealed = false`. The reveal key's label correctly tracks `k.revealed` at render time (lines 366-370). ✓
- **`TestPassphraseMaskReveal` non-vacuousness:** `uiContains(c, "****")` checks for consecutive `****` in the extracted text — this can only appear from the readout (the grid keys on page 0 don't produce consecutive `****`). `!uiContains(c, "ab1!")` checks that `"ab1!"` does not appear consecutively — it cannot, since page 0 has no '1' or '!' keys and the masked readout shows "****" not the cleartext. Both assertions would fail if the feature were broken. ✓
- **Scope:** shared `Keyboard`, `gui.go`, `theme.go`, codex32, SLIP-39, BIP-39 flows untouched. The new type has no `MnemonicSeed`/flow/fingerprint. ✓
- **Imports:** exactly `image`, `math`, `strings`, `unicode/utf8`, `seedhammer.com/gui/assets`, `seedhammer.com/gui/op`, `seedhammer.com/gui/widget`. No `fmt`, no `layout`. ✓
- **Signed-off-by** present in final commit message: `Signed-off-by: Brian Goss <goss.brian@gmail.com>`. ✓

---

### VERDICT

**SHIP-READY — 0 Critical / 0 Important**

Two minor findings: M-1 (reveal key width structurally unguarded against label swap, benign for current font/labels) and M-2 (convention deviation in `Measure` call form, flagged by implementer, functionally correct). Neither is a correctness defect at the current label set. M-3 and M-4 are coverage observations. The implementation is faithful to the plan and spec, case-preservation is confirmed end-to-end, no panics, no scope violations, no rune-uniqueness ambiguity, no geometry misalignment. Recommend fix of M-2 (`Measure(math.MaxInt, lbl)`) as a cleanup and consideration of M-1 (max of both label widths) as a defensive measure before Slice 3 integration, but neither blocks ship.


---

## Fold note (main session)
Verdict **SHIP-READY (0C/0I)** — gate passed. Disposition of the Minors:
- **M-1 (FOLDED, commit `67ceb95`):** the reveal key's cell was sized from "show" only; a future-wider "hide" would overflow/clip the tap target. Fixed `ppKeyExtent` to size `ppReveal` to `max(width("show"), width("hide"))` + a regression test `TestPassphraseRevealKeyFitsBothLabels`. Full suite re-run ALL PASS.
- **M-2 (DECLINED — reviewer mistaken):** the reviewer suggested reverting `Measure(math.MaxInt, "%s", lbl)` to `Measure(math.MaxInt, lbl)` for "convention." But `lbl` is a VARIABLE, and `Measure` is printf-style — `go1.26 go vet` flags a non-constant format string in that direct form (the codebase's direct `.Measure` calls all pass CONSTANT literals like `"W"`). The implementer's `"%s", lbl` is the correct vet-clean form; reverting would re-introduce the vet failure. Kept as-is.
- **M-3 (byte-len in TestPassphraseDpadCommit) / M-4 (no D-pad-into-function-row test):** accepted coverage notes — ASCII-only pages make byte==rune; `commit(ppPageCycle)` is directly tested; touch-tap is harness-blocked (consistent with Slice-1's deferral). No change.

Post-fold head: `67ceb95` (3 commits). The reviewer independently confirmed case-preservation end-to-end (no ToUpper/ToLower), cross-page rune uniqueness (disjoint page charsets), panic-safety/termination, layout/touch-clip coincidence, mask non-leak, and scope (2 files only).
