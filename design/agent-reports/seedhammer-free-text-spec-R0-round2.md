# Sonnet architect — R0 round 2 (fold check), Engrave Text spec rev 2

Narrow fold check of round 1's findings plus a hunt for regressions in rev 2's new
normative numbers. All work in a /tmp copy; real tree untouched; suite green throughout.

## VERDICT
**GREEN (0C / 0I)** — no new Critical or Important. **This closes the R0 gate.**

## FOLD CHECK — all fixed
- **C1 fixed.** §2 caps title and footer at 18 unconditionally with the rejected alternatives
  restated; §11 pins the every-rung containment. **Measured with the worst-case glyph** (search
  over all 95 printable ASCII found `'W'` tightest) via true ink `bspline.Measure`: 6.0mm →
  `x[10.709, 74.380]`mm against band `[10,75]` — **0.62–0.71mm slack, the tightest rung**; the
  other five clear by 7mm+. A 20-char title at 6.0mm still crosses both bands
  (`x[7.127, 77.873]`), so 18 is correct and genuinely tight.
- **I1 fixed.** §4's QR column is footnoted as geometry at a hypothetical 37-module QR, not a
  capacity, with a separate true-maxima table and "solve by iteration, never read from this
  table". **Measured, reproduces exactly at all six rungs**: lowercase 178/230/284/367/429/520,
  uppercase 195/255/318/398/501/616.
- **I2 fixed.** 114 uppercase = **33** modules, +1 lowercase = 41. Measured exact, along with
  106 lower/upper/digit → 37/33/29.
- **I3 fixed.** §7.1 now forbids modifying `TestPassphraseKeyboardConstruction` and keeps its
  "exactly four" as the anti-leak guard. **Implemented the opt-in design in a scratch copy**
  (`newPPKeyboard(ctx, newline bool)`, `NewTextKeyboard`→true, key appended at the end of the
  function row): `go test ./gui/...` **exit 0 with zero existing test files touched**; a probe
  confirms `NewPassphraseKeyboard`/`NewAddressKeyboard` keep 4 function-row keys with reveal at
  index 2, while `NewTextKeyboard` has 5 with reveal still at index 2.
- **I4 fixed.** `widthAt` indexed by output line with a caller-supplied plate-row offset, and it
  is self-consistent with §5.1's plate-row inset formula — the geometry replica using the row
  index literally reproduces both §4's grid and §5.1's inset table. Because title/footer rows are
  reserved *unconditionally*, the `+1` offset is unconditional too, resolving the docstring's
  apparent conditionality.
- **M1 fixed** — scale-3 geometry `161 228 309 436 576 759` measured exact at all six rungs.
- **M2 fixed** — step 5 is now a deterministic pipeline (wrap → preserve → trim trailing) with no
  input where two clauses assign different outputs; the space-run line is explicitly emitted empty.
- **M3 fixed** — the `Text.FontSize == 0 → plateFontSizeUR` fallback is stated, consistent with
  `EngraveText` hardcoding it today and every existing caller omitting the field.
- **M4 fixed** — footer centered in the same inset span; since `holeChars`/`charWidth`/`margin`
  are row-independent and the footer row is inset at every rung, the title's verified geometry
  transfers directly.
- **N1 fixed** — two hand-edited sites; `npage`/`npages` verified as literally
  `int(bip85Derive)+1` at `gui.go:1868` and `:1902`, and `layoutMainPlates` panics at `:1898`.

## NEW FINDINGS
**None.**

## VERIFIED BY MEASUREMENT
Grid and inset-row tables reproduced from first principles at all six rungs before trusting the
probe for anything else; the 18-char cap at every rung by both advance-sum and true-ink bounds;
the true-maxima table; the scale-3 column; the 700-char/3.0mm refusal figure — live QR 89
modules, 392 columns beside it, capacity without QR 1032, **freed = 640 exactly**, against the
naive geometry answer of 1032−897 = 135; QR module counts; the keyboard opt-in design end to end;
the menu line citations read from source. `go build ./...` and
`go test ./backup/... ./gui/... ./engrave/...` exit 0 throughout.
