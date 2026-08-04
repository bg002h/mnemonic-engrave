# SPEC — Engrave Text (free-text plate), SeedHammer II fork

Status: **DRAFT rev 2 — R0 rounds 0 (3C/10I) and 1 (1C/4I) folded, awaiting re-review.**
No code before 0C/0I. Date: 2026-08-04. Target: `bg002h/seedhammer`, program slot **3**.

---

## 1. Purpose

A plate carrying arbitrary user text: a label, a note, an instruction to an heir,
a URL. Not a backup format — nothing here is validated, derived, or checksummed.

**Not constant-time.** Unlike the seed and passphrase programs, engraving
duration varies with content (user directive 2026-08-04). §9 states what that
costs.

## 2. Fields

| Field | Required | Limit | In QR? |
|---|---|---|---|
| **Title** | no | 1 line, centered | **no** |
| **Text** | yes | multi-line, capacity per §6 | **yes — and only this** |
| **Footer** | no | 1 line | **no** |
| **QR** | no (opt-in) | encodes Text verbatim | — |

The QR encodes **the Text field and nothing else**. A scanner must return exactly
what the operator believes it will.

Title and footer occupy plate-absolute rows 0 and `LinesPerPlate-1`. Both are
screw-hole rows (§5.1), so the geometric limit is `charsPerLine − 2·holeChars` —
but that evaluates to **18/20/24/26/30/36** down the ladder, and the engraved
rung is not known while the title is being typed.

**Therefore title and footer are capped at 18 characters, unconditionally** — the
6.0mm figure, the tightest rung. A rung-relative cap is rejected: anchoring it at
3.0mm (36) lets a short text auto-fit at 6.0mm carrying a title that row cannot
hold, and **`toPlate` does not catch it**. Measured, a centered 20-character
title at 6.0mm inks `x[7.127, 77.962]`mm, crossing both screw-hole bands
(`[3,10]` and `[75,82]`) while every check passes — a silently wrong plate.
Enforcing at the *current* rung is no better: deleting text raises the rung and
retroactively invalidates an already-entered title, the case §6 and §12.1 exist
to rule out.

§11 pins that a title at the cap sits inside `[innerMargin, plateSize −
innerMargin]` at **every** rung.

The title is engraved **verbatim**. `backup.TitleString`
(`backup/backup.go:49-61`) MUST NOT be used: it silently upper-cases and
truncates at `MaxTitleLen = 18`, which would engrave something the operator never
approved.

## 3. Font

`font/sh` — the SeedHammer engraving face, already used upstream for descriptor
plates. Extended to the full 95 printable ASCII by importing 14 glyphs (§10).

Verified: `sh.bin` currently decodes 81 of the 95, the missing set is exactly the
14 imported, and every advance is 4000 — the face is genuinely monospace, so
`charWidth` derived from `'W'` is exact for all glyphs.

`engrave.String` **panics** on a rune absent from the face
(`engrave/engrave.go:1531`). The glyph import MUST land before the keyboard is
wired, and §11 requires a test that every rune the keyboard can emit decodes.

Non-ASCII is refused. Newline is a control character, never a glyph (§5).

## 4. Size ladder and auto-fit

`FontSizes = {6.0, 5.0, 4.4, 3.8, 3.4, 3.0}` mm, descending. **The engraved size
is the first entry at which the whole composition fits.**

| Size | Grid | Plain | +title+footer | +title+footer+QR¹ |
|---|---|---|---|---|
| 6.0 | 22×13 | 274 | 238 | 198 |
| 5.0 | 26×15 | 372 | 332 | 278 |
| 4.4 | 30×17 | 492 | 444 | 384 |
| 3.8 | 34×20 | 648 | 596 | 519 |
| 3.4 | 38×23 | 834 | 774 | 678 |
| 3.0 | 44×26 | 1104 | 1032 | 897 |

¹ **Geometry only, at a hypothetical 37-module QR — NOT an attainable capacity.**
Cumulative with title+footer, `QRScale = 2` (§8). At scale 3 the same geometry is
`161 228 309 436 576 759`.

**The QR encodes the Text (§2), so this column is unreachable**: 37 modules is
v5-L, at most 106 bytes, and no text of the column's own length produces one.
Measured true maxima, solving the fixed point text↔QR:

| Size | lowercase | uppercase |
|---|---|---|
| 6.0 | 178 | 195 |
| 5.0 | 230 | 255 |
| 4.4 | 284 | 318 |
| 3.8 | 367 | 398 |
| 3.4 | 429 | 501 |
| 3.0 | 520 | 616 |

These vary with content, so the implementation MUST solve for them by iteration
(§6), never read them from this table.

`charWidth` derives from the font, not the stroke, so the Plain grid is identical
at `0.3*mm` and at `mm/3`. **Only the QR columns move with stroke width** — and
those must be computed at the production value, `strokeWidth = 0.3 * mm`
(`cmd/controller/platform_sh2.go`). A capacity test written with
`backup/backup_test.go`'s `mm/3` params reproduces this same column with a
**33**-module QR, which production does not satisfy — §11 forbids that.

### 4.1 Vertical headroom

Every rung's bottom edge lands inside `toPlate`'s 82mm bound
(`gui/gui.go:2833-2841`, `safetyMargin = 3`): max 81.2mm at 3.4mm. The widest
line is 81.8mm at 6.0mm — 0.2mm of horizontal headroom, tight but real. §11 pins
both.

## 5. Wrapping — ONE function, three callers

> **The load-bearing requirement of the feature.**

```go
// WrapText lays out s into engraved lines at a fixed pitch.
//
// widthAt is indexed by OUTPUT line (0 = the first line WrapText emits), NOT by
// plate row. The caller supplies the plate-row offset inside the closure: the
// free-text plate passes i+1 when a title occupies row 0; the descriptor callers
// pass their paragraph's own base. Getting this wrong puts the first text line
// on the title's row, or makes the fit check and the engraving disagree by one.
// widthAt MUST return >= 1 for every output line 0 <= i < maxLines; WrapText
// asserts this.
// Returns ok=false if the composition needs more than maxLines lines; callers
// MUST NOT engrave a false result.
func WrapText(s string, widthAt func(line int) int, maxLines int) (lines []string, ok bool)
```

**The screen renders these lines, the engraver engraves these lines, and the fit
check counts these lines** — one function called three times, never three
implementations that ought to agree.

Output lines contain **no `'\n'`**. Each is engraved by its own
`engrave.String` call at its own `(offx, offy)`. `StringCmd.Measure` MUST NOT be
used for fit: measured, it returns the *last* line's width and *one* em of
height regardless of line count (`engrave/engrave.go:1537`).

### 5.1 `widthAt` — the screw-hole inset is a BAND, not two rows

Line `i` loses `holeChars` on **each** side iff

```
offy + i*fontSize < innerMargin  ||  offy + (i+1)*fontSize > plateHeight - innerMargin
```

Measured inset rows, which is what §4's table was computed from:

| Size | inset rows | holeChars | lines |
|---|---|---|---|
| 6.0 | 0, 1, 12 | 2 | 13 |
| 5.0 | 0, 1, 14 | 3 | 15 |
| 4.4 | 0, 1, 16 | 3 | 17 |
| 3.8 | 0, 1, 18, 19 | 4 | 20 |
| 3.4 | 0, 1, 2, 21, 22 | 4 | 23 |
| 3.0 | 0, 1, 2, 24, 25 | 4 | 26 |

**At 3.0mm five rows are inset, not two.** Lines beside the QR use the QR width
instead and are not additionally inset on the QR side — reproducing
`EngraveText`'s `n` bit-for-bit, **including its `if n < 1 { n = 1 }` clamp**
(`backup/backup.go:341-343`).

### 5.2 Line breaking

1. Split `s` on `'\n'` into blocks. **A block boundary is a wrap-time concept
   only** — it never becomes a `backup.Paragraph` (§5.3). An empty block emits
   one empty line occupying a full `fontSize` row.
2. Within a block, split on `' '` (U+0020). No other character is a break
   opportunity — not `-`, not `/`.
3. Greedily fill line `i` to `widthAt(i)`.
4. **Overlong-token fallback.** A word longer than `widthAt(i)` that is alone at
   the start of a line is character-broken at exactly `widthAt(i)`; the remainder
   continues on the next line. An xpub or URL must not deadlock the wrap.
5. **The space rule, stated once, in precedence order.** (a) A break consumes
   exactly the run of spaces at the break point. (b) Runs *not* at a break —
   including a block's leading indent — are preserved verbatim, so a plate may be
   a table. (c) Trailing spaces are then stripped from every emitted line, which
   resolves the collision: a line whose whole content would be a space run is
   emitted **empty**, and rule (c) always wins over (b) at end of line.
6. Return `(partial, false)` the moment the line count would exceed `maxLines`,
   including mid-token.

### 5.3 A single flat line list

The free-text plate renders **one flat list of lines at a uniform `fontSize`
pitch, with no inter-paragraph gap**. It MUST NOT map blocks onto
`backup.Paragraph` values. Measured, that mapping is wrong twice over
(`backup/backup.go:363-367`): a blank paragraph advances `offy` by **1mm, not a
full row**, so a blank line the operator saw in the preview becomes a 1mm gap and
everything below shifts up; and every `'\n'` costs an extra 1mm that a uniform
model does not account for — at 3.0mm, where the bottom line already sits at
81mm against the 82mm bound, **two newlines overflow the plate** after the
confirm screen.

## 6. Capacity, admission, and refusal

**The QR's module count is obtained by calling `qr.Encode(text, qr.L)` and
reading `.Size`. Never from a length table.** `qr.Encode` is mode-adaptive:
measured, 106 lowercase chars → 37 modules, 106 uppercase → 33, 106 digits → 29;
alphanumeric boundaries are 26/48/78/115/155. A 114-char uppercase string is **33**
modules and **one appended lowercase letter takes it to 41 — two version steps
from one keystroke.** A byte-boundary table is simultaneously too pessimistic for
uppercase and blind to the double step.

Module count is non-decreasing as characters are appended (measured over 1–260
chars, lower and upper), so capacity is monotone — *provided the encoder is
called rather than a table consulted*.

**Admission anchor.** A composition is admissible iff it fits at **3.0mm with the
QR as chosen and one row reserved for each of title and footer, whether or not
they are used**. Reserving unconditionally is what makes the anchor monotone:
without it, entering a title after the text would retroactively invalidate text
already accepted (§7 orders the QR choice first for the same reason).

**Displayed size** is the largest rung that fits, recomputed live, so the
operator watches it drop 6.0 → 5.0 → … as they type.

**Over-capacity is shown, not silently dropped.** Keystrokes are accepted; the
readout shows the over-capacity state; OK refuses with a message naming the field
and that it is already at the smallest size. This follows the sibling program's
reviewed decision (`gui/passphrase_flow.go:113-118`): *"Over-length is shown
rather than clamped: silently dropping keystrokes would leave the operator
believing a longer passphrase had been entered."*

The readout is **"lines used / lines available"**, not "characters remaining":
under word wrap no scalar character count is correct, since appending `x` to the
last word can cost a whole line while appending ` x` does not.

**Text that does not fit one plate is refused, not split** (user directive). The
refusal names how many characters removing the QR would free — measured
1104 → 969 at scale 2, 831 at scale 3 — but **never drops the QR automatically**,
since that silently changes what a scanner returns from the plate. Offer it as an
explicit choice, in the shape `validateDescriptor` already uses
(`gui/gui.go:432-444`).

Note `EngraveText`'s `for len(txt) > 0` loop is unbounded, but `toPlate` is
fail-closed against the 82mm bound and **both existing callers depend on that
refusal** for their TEXT+QR → TEXT-ONLY → QR-ONLY fallback. The fit gate is
therefore a UX improvement over "a variant silently disappears", not a
correctness fix.

## 7. Flow

1. **QR** — opt-in, default No QR (`ChoiceScreen`, mirroring the passphrase
   program). **First**, so the admission anchor is fixed before text entry.
2. **Text** (required) — keyboard with newline key. Live: fitted size, lines
   used / available.
3. **Title** (optional, skippable) — single line.
4. **Footer** (optional, skippable) — single line.
5. **Confirm** — the wrapped lines exactly as they will be cut, fitted size, line
   count, QR yes/no, and §9's warnings.
6. **Engrave**.

Back from any step preserves every entered value.

### 7.1 Keyboard

Reuse `PassphraseKeyboard` — four pages, 95 characters, touch-driven, already
covered by reachability and panel-fit tests — plus a newline key on the function
row of **all four pages**.

The newline key MUST be a **per-instance opt-in field, default off**.
`PassphraseKeyboard` is shared with `NewAddressKeyboard` (address verification)
and BIP-85 index entry, and `ValidatePassphrase` rejects `'\n'` with
`ErrNonASCII` — an unconditional key would give the passphrase program a key that
silently fails at OK.

**No existing keyboard test changes.** Measured on the opt-in design
(`newPPKeyboard(ctx, newline bool)`, `NewPassphraseKeyboard`→false,
`NewTextKeyboard`→true): the whole `gui` suite exits 0, because every existing
test constructs `NewPassphraseKeyboard`, which is unchanged.

In particular **`TestPassphraseKeyboardConstruction` MUST NOT be modified.** Its
assertion that the shared function row has *exactly four* keys is precisely the
guard that catches a newline key leaking into `NewAddressKeyboard` or BIP-85
index entry. Rev 1 told the implementer to update it — that would disable the
guard for a change that does not need it. (Round 0's measurement was taken under
the *unconditional* design this section rejects: appended at index 4 only
`TestPassphraseKeyboardConstruction` fails; both fail only if inserted before
index 2.)

Instead, add **new** construction and reachability tests for the free-text
variant. The newline key is **appended to the end of the function row**, so the
reveal key keeps index 2 — asserted at `passphrase_keyboard_test.go:200`.

### 7.2 Menu integration

Insert after `engravePassphrase`. `bip85Derive` MUST remain last. **Two sites
need hand-editing** (`gui/gui.go:146-168`, `:1891-1899`):

1. the program enum,
2. `layoutMainPlates`' case list — which **panics** on a program it does not
   list.

`npage` / `npages` are `int(bip85Derive)+1` (`:1868`, `:1902`) and update
themselves, provided `bip85Derive` stays last.

The pager goes from 7 to 8 dots, widening `(sz.X+space)*npages-space`. §11
requires a start-screen panel-fit assertion at 8 dots.

## 8. Layout

- **Title** — plate row 0, centered within the **inset span**
  `[holeChars*charWidth, width − holeChars*charWidth]`, not the full 79mm: row 0
  is a screw-hole row, and full-width centering pushes a long title into the
  screw-hole band.
- **Text** — the remaining rows.
- **Footer** — plate row `LinesPerPlate-1`, absolute, not "after the text";
  **centered in the same inset span as the title**, for symmetry.
- **QR** — right-hand side, 2mm border, text reflowing beside it; upstream
  `EngraveText` placement. **`QRScale = 2`**, normative: 0.6mm modules against the
  0.9mm every other plate uses, chosen because §4's capacity column depends on it
  and free text needs the room.
- Screw-hole insets per §5.1.

## 9. Safety

- **A free-text box is where someone will type a seed phrase.** It bypasses the
  wordlist, the checksum, and the verify flow. The confirm screen MUST warn that
  this plate is not a validated backup and nothing has been checked. (A
  damaged-backup-format heuristic, as the Gangleri fork implements, is a
  follow-up, not in scope.)
- **Engraving is not constant-time.** Duration varies with content, so an
  observer who can watch or time the machine learns about the text. Fine for a
  label, wrong for a secret. State it on the confirm screen.
- **A QR makes the text trivially machine-readable** from a photograph. Default
  off, opt-in only.
- No field is logged; no error message quotes field content.

## 10. Import and provenance

From `Gangleri42/seedhammer` (Unlicense; verified separately):

1. **14 glyphs** into `font/sh/sh.svg`: `& \ | ^ $ = ! \` % + ? " ~ _`
2. **`FontSizes`, `CharsPerLine`, `LinesPerPlate`, `fixedCharWidth`,
   `const plateSize`, `Text.FontSize`** in `backup/backup.go`.
   **`Text.FontSize == 0` MUST fall back to `plateFontSizeUR`** — every existing
   caller constructs `Text` without the field, so this fallback is exactly what
   keeps the three `text-*` goldens byte-identical.

**MUST NOT be taken** — the same file's diff also reworks *seed* plates
(`stringColumn` signature, `.SourceOrder()` on titles, a `largeN` rebalance) and
deletes `if qrc.Size > 33 { return nil, errors.New("seed too long to engrave QR") }`
(`backup/backup.go:82-84`), a live fail-closed guard on a seed path.

**`EngraveText` is refactored** to consume pre-wrapped lines, which §5 requires
and the original draft failed to authorize. The descriptor and mdmk callers keep
an unbounded path so their variant fallback is unchanged. Descriptor and UR text
contains no `' '`, so wrapping reduces to the existing character slice — the
three `text-*` goldens must come out **byte-identical**, which §11 pins.

Record the exact upstream commit in `PROVENANCE.md` under *Imports*, naming the
symbols and glyphs taken.

## 11. Test requirements

- **The single-wrap-function invariant is the primary target.** Assert the
  confirm screen's extracted lines equal `WrapText`'s output for the same input
  and size. Each line must be rendered as its **own unwrapped** label — a
  width-bounded `widget.Labelw` would re-wrap in a proportional screen face and
  break the very invariant the test protects.
- **The three existing `text-*` goldens must be byte-identical** after the
  `EngraveText` refactor. No existing golden may be updated; new goldens are fine.
- §5.1's inset-row table pinned per rung; the `n < 1 → n = 1` clamp reproduced.
- Wrap: word boundaries, overlong-token fallback, the single space rule (leading
  runs preserved, break runs consumed, no trailing space), varying `widthAt`,
  explicit `\n`, empty blocks, `maxLines` refusal mid-token, and the
  `widthAt >= 1` assertion.
- §5.3: a blank line occupies a full row, and N newlines do not add N millimetres.
- QR: size from the encoder, not a table; an **uppercase** and a **digit** string;
  the mode-flip double-step (114 uppercase + one lowercase); monotonicity over a
  long append run; module-level assertion that the QR encodes Text **only**.
- Capacity tests run at `StrokeWidth = 0.3*mm`; a test at `mm/3` with a 33-module
  QR reproduces §4's column falsely and is forbidden.
- §4.1's 82mm vertical and 81.8mm horizontal bounds.
- **A title and a footer at the 18-character cap sit inside
  `[innerMargin, plateSize − innerMargin]` at EVERY rung** — the 6.0mm rung is the
  binding one, where 20 characters already cross both screw-hole bands while
  `toPlate` passes.
- The refusal's "dropping the QR frees N characters" is computed from a live
  encode, not a constant: at 3.0mm with a 700-character text the true figure is
  640, where §4's geometry column would suggest ~135.
- Keyboard: **no existing test modified** — `TestPassphraseKeyboardConstruction`'s
  "exactly four function-row keys" stays as the anti-leak guard. New construction
  and reachability tests for the free-text variant; the newline key absent from
  address verification and BIP-85; the reveal key still at index 2; every rune the
  keyboard can emit decodes in `font/sh`.
- Start screen fits at 8 pager dots.
- **Every fix verified by mutation.** A green suite proves nothing until the code
  is broken and the suite goes red.

## 12. Decisions (were open in rev 0)

1. **Size is always automatic**, never user-picked. An override would re-anchor
   admission to the current size — exactly the retroactive-invalidation §6 rules
   out. If "bigger letters" is wanted later, express it as an optional max-lines
   cap, which composes with the anchor.
2. **Title centered**, matching every existing plate — within the inset span (§8).
3. **No automatic QR drop.** Refusal names the characters it would free and
   offers it as an explicit choice.
4. **Newline on all four pages** of the free-text keyboard, on **no** page of any
   other, via the per-instance flag (§7.1).
