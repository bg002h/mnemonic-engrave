# Opus architect — post-implementation execution review, Engrave Text

Whole diff `main..free-text`, 11 commits, 31 files, +3892/−69. Read-only; all probing on copies.

## VERDICT
**NOT GREEN (0 Critical / 1 Important)** + 2 Minor, 1 Nit. **No defect reaches steel.** The one
blocking finding is a coverage gap on correct code.
**30 mutations run, 26 killed, 4 survived — all four genuinely equivalent or already reported.**

## I1 — Nothing binds the QR choice's LABEL to the boolean it produces *(FIXED, `6d47d96`)*
`gui/freetext_flow.go:66-84`. Every test selects by *index*, using the same convention the code
uses, so the label↔semantics binding is asserted in neither direction and default-off is never
asserted. **Proof (mutation G9):** swapping only the two strings, leaving `sel == 1` alone, leaves
`go test ./gui/` **green**. Under it, an operator tapping the displayed **"Add QR"** gets **no
QR**, and one tapping **"No QR"** gets a plate carrying a machine-readable copy of their text —
on the one field spec §9 names a privacy hazard. Also deviates from house convention
(`TestMultisigScriptChoices` is order-locked; `bip85_test.go:142` pins element-by-element).

## M1 — Both provenance records misattribute the redrawn `!`/`?` dot to `period` *(FIXED)*
`font/sh/import-check.md:101`, `PROVENANCE.md:109`. `period` (`sh.svg:178`) draws
`C266,5,266,4.9,266.1,5` — the form the docs attribute to *upstream*. **Measured:** reverting
`!`/`?` to `period`'s exact form returns both to **2638.22 mm/s³** and lifts the whole-alphabet max
from 2610.91 to 2638.22, past `TestFonts`' 1% slack (2626) — red. Following the sentence
reintroduces the failure the record documents.

## M2 — The title/footer newline guard had no test *(FIXED, `6d47d96`)*
`gui/freetext_flow.go:212-215`. **Proof (G4b):** disabling `strings.ContainsRune(…, '\n')` leaves
the gui suite green. `engrave.String` treats `'\n'` as a line break, so a two-line title engraves
its tail onto the body's first row, and `StringCmd.Measure` returns only the last segment's
advance, so centering is computed from the wrong width.

## N1 — `ftBuildPlate` re-runs `Fit` rather than reusing the confirm evaluation
Harmless: `qr.Encode` is deterministic and `TestFTBuiltPlateIsTheFittedComposition` pins equality.

## CHECKED AND CLEAN
- **Glyph artwork.** The 2638.22 → 2600.79 figures reproduce exactly. **The 2600 "limit" is not
  the gate** — `TestFonts` allows 1% (2626) and **13 pre-existing glyphs already exceed 2600**
  (`0` at 2610.91, `i`/`j` at 2605.49). The import did **not** raise the face's worst case: max is
  2610.91, driven by `0`, before and after. Ink bounds **byte-identical** for `!`, `?`, `.`; only
  3 of 34 control points move, ≤287 units ≈ **2.7 µm at 6.0mm, ~1% of the stroke**. `sh.bin`
  reproduces byte-for-byte from the committed `sh.svg`.
- **All three "equivalent" survivors confirmed equivalent**, with reasons.
- **All four previously-surviving mutations now die.**
- **Inset-span centering is arithmetically identical to full-width** at all six rungs — the
  implementer's claim is true and honestly documented. **The 18-char cap is the only protection
  and is exactly right:** 18 chars ink `x[10.709, 74.380]` inside `[10,75]`; **19 already escapes**
  at `x[8.918, 76.171]`. Tested the unstated `'W'`-is-worst-case assumption across all 95 glyphs —
  only `C` inks further left, by 0.009mm, still inside. Cap enforced on both fields at the only
  entry path.
- **Both upstream fixes correct and side-effect free.** `Bounds.Empty` had no production caller;
  reverting `uiFlow` makes the gui suite **hang** (confirmed under `-timeout 60s`).
- **`Fit`'s nil-`*qr.Code`**: both call sites handle it; no path dereferences nil.
- **Admission monotonicity fuzzed** (1–1400 chars × 4 alphabets × QR on/off; never decreases) and
  **`Admissible ⟹ Fit` over ~19,200 compositions with no counterexample** — which is why removing
  the confirm-step re-check survives: genuinely unreachable defensive code, not a gap.
- **Capacity tables independently re-derived** without reading the tests: plain true maxima
  `[214 268 322 408 474 564]` / `[220 279 357 450 535 667]`, refusal figure 640 — all matching.
- **QR never leaves the plate** (swept 1–1400 chars; largest accepted is 81 modules at n=521).
- **Secret hygiene clean:** counts and field names only; no field content in any log or error.
- **Spec §11 line by line:** every bullet non-vacuous; §4/§5.1 numbers are literals, not
  re-derived; capacity tests all at production `0.3*mm`, guarded by
  `TestProductionStrokeIsWhatWeMeasure`; `gui/passphrase_keyboard_test.go` byte-identical to
  `main`; the six touched test files changed only navigation indices, no assertion weakened.
- **Goldens:** three `text-*` compared not regenerated; two new `freetext-*` die under 6 mutations.

## FOLD RECORD (`6d47d96`)
I1 and M2 fixed with mutation-verified tests — labels swapped and newline guard disabled each turn
the suite red. M1 corrected in both records with the measurement, so the note cannot mislead the
next reader into reverting the fix it documents. N1 recorded, not actioned. Suite exit 0; no golden
moved.
