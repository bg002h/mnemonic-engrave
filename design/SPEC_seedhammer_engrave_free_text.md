# SPEC — Engrave Text (free-text plate), SeedHammer II fork

Status: **DRAFT — awaiting R0 review.** No code before 0C/0I.
Date: 2026-08-04. Target: `bg002h/seedhammer`, program slot **3 of 8**.

---

## 1. Purpose

A plate carrying arbitrary user text: a label, a note, an instruction to a heir,
a URL. Not a backup format — nothing here is validated, derived, or checksummed.

**Not constant-time.** Unlike the seed and passphrase programs, engraving
duration varies with content (user directive 2026-08-04). This buys a prettier
face and a simpler engine; §9 states what it costs.

## 2. Fields

| Field | Required | Limit | In QR? |
|---|---|---|---|
| **Title** | no | 1 line, centered, first line | **no** |
| **Text** | yes | multi-line, capacity per §6 | **yes — and only this** |
| **Footer** | no | 1 line, last line | **no** |
| **QR** | no (opt-in) | encodes Text verbatim | — |

The QR encodes **the Text field and nothing else** — not the title, not the
footer, not a separator. Same rule as the passphrase plate, for the same reason:
a scanner must return exactly what the operator believes it will.

Title and footer occupy the first and last text rows, which are also the rows the
screw-hole inset shortens, so their own limit is `charsPerLine − 2·holeChars`,
materially less than a middle line. The UI must enforce the real number, not
`charsPerLine`.

## 3. Font

`font/sh` — the SeedHammer engraving face (cubic Bézier, proportional-looking
monospace), already used upstream for descriptor plates. Extended to the full 95
printable ASCII by importing 14 glyphs (§10).

Non-ASCII input is refused, as in the passphrase program. Newline is a control
character, not a glyph, and is handled by §5.

## 4. Size ladder and auto-fit

`FontSizes = {6.0, 5.0, 4.4, 3.8, 3.4, 3.0}` mm, descending. **The engraved size
is the first entry at which the whole composition fits.** Measured grids, which
match the Gangleri fork's documented numbers independently:

| Size | Grid | Plain | +title+footer | +106-byte QR |
|---|---|---|---|---|
| 6.0 | 22×13 | 274 | 238 | 198 |
| 5.0 | 26×15 | 372 | 332 | 278 |
| 4.4 | 30×17 | 492 | 444 | 384 |
| 3.8 | 34×20 | 648 | 596 | 519 |
| 3.4 | 38×23 | 834 | 774 | 678 |
| 3.0 | 44×26 | 1104 | 1032 | 897 |

Figures are at the **production** stroke width, `strokeWidth = 0.3 * mm`
(`cmd/controller/platform_sh2.go`). Note `backup/backup_test.go` uses `mm/3`,
11% fatter; any test asserting a millimetre figure must say which it means.

## 5. Wrapping — ONE function, three callers

> **This is the load-bearing requirement of the feature.**

A single exported function lays out text:

```go
// WrapText lays out s into engraved lines. widthAt gives the usable character
// count of line i, which VARIES: lines beside the QR are narrower, and the
// first and last lines lose 2*holeChars to the screw-hole inset.
func WrapText(s string, widthAt func(line int) int, maxLines int) (lines []string, ok bool)
```

**The screen renders these lines, the engraver engraves these lines, and the fit
check counts these lines.** Not three implementations that ought to agree — one
function, called three times. A preview that wraps differently from the plate
would let an operator approve a layout that is not what gets cut, on a permanent
medium. This is the same seam class as the counter-occlusion defect, which shipped
because two paths agreed in tests and disagreed in reality.

**Algorithm, stated so it can be reproduced exactly:**

1. Split `s` on `'\n'` into paragraphs. An empty paragraph emits one empty line.
2. Within a paragraph, split on `' '` (U+0020) into words. No other character is
   a break opportunity — not `-`, not `/`.
3. Greedily fill line `i` to `widthAt(i)`: append a word if
   `len(current) + 1 + len(word) <= widthAt(i)` (the `+1` is the separating
   space, omitted when the line is empty).
4. **Overlong-token fallback.** A word longer than `widthAt(i)` on an *empty*
   line is character-broken: take exactly `widthAt(i)` characters, continue the
   remainder on the next line. An xpub or URL must not deadlock the wrap.
5. The space at a break is **consumed**, never leading the next line and never
   trailing the previous one.
6. Stop and return `ok=false` if the line count would exceed `maxLines`.

Trailing whitespace on a paragraph is stripped before wrapping. A run of N spaces
inside a line is preserved as typed — a free-text plate may be a table.

## 6. Capacity and refusal

Capacity is a **step function that can move against the operator**: as Text grows
it crosses QR version boundaries (17 / 32 / 53 / 78 / 106 bytes at ECC-L), the QR
grows, and the lines beside it get narrower — reducing room while the text that
caused it is still present.

Therefore:

- **Admission is decided at the smallest size (3.0mm).** A keystroke is accepted
  iff the composition still fits at 3.0mm. This is the only monotone anchor;
  testing against the *current* fitted size would let a QR boundary retroactively
  invalidate already-accepted text.
- **The displayed size is the largest that fits** — recomputed live, so the
  operator sees it drop 6.0 → 5.0 → … as they type.
- **Text that does not fit on one plate is refused, not split.** No second plate,
  no truncation (user directive). The refusal must name the reason: which field,
  and that it is already at the smallest size.
- Upstream's `EngraveText` loop is `for len(txt) > 0` with **no line bound** — it
  emits lines past the plate edge. The fit gate is therefore covering a real
  existing gap, not only a UX nicety.

## 7. Flow

1. **Text** (required) — keyboard, newline key. Live: fitted size, lines used,
   remaining characters.
2. **Title** (optional, skippable) — single line.
3. **Footer** (optional, skippable) — single line.
4. **QR** (opt-in, default **No QR**) — `ChoiceScreen`, mirroring the passphrase
   program so the two behave the same way.
5. **Confirm** — the wrapped lines exactly as they will be cut, the fitted size,
   line count, QR yes/no, and the §9 warnings.
6. **Engrave**.

Back from any step preserves every entered value. Keyboard: reuse
`PassphraseKeyboard` (four pages, 95 characters, touch-driven, already covered by
reachability and panel-fit tests) plus a **newline key**. Adding a key changes
page geometry, so `TestPassphraseKeyboardStaysOnPanel` and the all-keys-reachable
test must be re-run and must still pass for the passphrase program.

## 8. Layout

- Title: first line, **centered**, at the fitted size.
- Text: from line 1 (or 0 without a title) to the last available line.
- Footer: last line.
- QR: right-hand side, 2mm border, text reflowing beside it — upstream
  `EngraveText` placement, unchanged.
- Screw-hole insets on any line outside `innerMargin`, unchanged.

## 9. Safety

- **A free-text box is where someone will type a seed phrase.** It bypasses the
  wordlist, the checksum, and the verify flow the seed programs enforce. The
  confirm screen MUST warn that this plate is not a validated backup and that
  nothing here has been checked. (The Gangleri fork reached the same conclusion
  independently and warns when text resembles a damaged backup format; a
  heuristic detector is **out of scope here** and is recorded as a follow-up.)
- **Engraving is not constant-time.** Duration varies with content, so an
  observer who can watch or time the machine learns about the text. Acceptable
  for a label; wrong for a secret. State it on the confirm screen.
- **A QR makes the text trivially machine-readable** from any photograph of the
  plate. Default off, opt-in only.
- No field is logged, and no error message quotes field content.

## 10. Import and provenance

From `Gangleri42/seedhammer` (Unlicense — verified: repo LICENSE is the
Unlicense; `font/sh/` has no per-directory LICENSE; their base glyphs are
byte-identical to upstream's, so it is a genuine extension of SeedHammer's own
public-domain face, with no third-party attribution markers):

1. **14 glyphs** into `font/sh/sh.svg`: `& \ | ^ $ = ! \` % + ? " ~ _`
2. **`FontSizes`, `CharsPerLine`, `LinesPerPlate`, `fixedCharWidth`,
   `const plateSize`, and the `Text.FontSize` field** in `backup/backup.go`.

**MUST NOT be taken** — the same file's diff also reworks *seed* plates:
`stringColumn` signature changes, `.SourceOrder()` on titles, a `largeN` layout
rebalance, and the deletion of

```go
if qrc.Size > 33 { return nil, errors.New("seed too long to engrave QR") }
```

Those would move seed goldens and remove a fail-closed check on a seed path.
Nothing about free text needs them.

Record the exact upstream commit in `PROVENANCE.md` under *Imports*, naming the
symbols and glyphs taken.

## 11. Test requirements

- **The single-wrap-function invariant is the primary test target.** A test must
  fail if the screen and the plate could disagree — e.g. by asserting the confirm
  screen's extracted lines equal `WrapText`'s output for the same input and size.
- Wrap: word boundaries, the overlong-token fallback, consumed spaces, varying
  `widthAt`, explicit `\n`, empty paragraphs, and the `maxLines` refusal.
- Fit: the QR step function — a string one byte over a version boundary must
  shrink capacity; admission at 3.0mm; refusal messages.
- Title/footer limited by `charsPerLine − 2·holeChars`, not `charsPerLine`.
- QR encodes Text **only** — asserted at module level, as
  `TestPassphraseQRIgnoresFingerprints` does, since a decoder that ignores
  trailing data would pass while the modules differed.
- Passphrase program regression: keyboard geometry after adding the newline key.
- **Every fix verified by mutation**, per project standard. A green suite proves
  nothing until the code is broken and the suite goes red.
- **No existing golden may be updated.** New goldens are fine.

## 12. Open questions for R0

1. Should the fitted size be **user-overridable** (pick 6.0mm and accept less
   text), or always automatic?
2. Is a **centered** title right, or should it be left-aligned like body text?
3. When text is refused at 3.0mm, should the UI offer to **drop the QR**
   automatically to recover ~10–15% capacity, or simply refuse?
4. Does the newline key belong on **every** keyboard page or only the letter
   pages? (The Gangleri fork puts it on the letter layers only.)
