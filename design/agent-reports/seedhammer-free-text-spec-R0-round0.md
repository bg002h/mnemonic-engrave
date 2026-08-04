# Opus architect — R0 round 0, Engrave Text spec

- **Spec:** `design/SPEC_seedhammer_engrave_free_text.md` (DRAFT)
- **Brief:** if implemented exactly as written, what would be wrong, missing, or under-specified such that an operator gets a plate differing from what they approved, or the implementer must guess?
- **Date:** 2026-08-04. Repo verified clean at `abb7458`; all probing in a scratch copy; baseline suite exit 0.

## VERDICT
**NOT GREEN (3C / 10I)** + 3 Minor, 2 Nit.

## C1 — §5's `widthAt` contract is factually wrong: the screw-hole inset is a BAND, not "first and last line"
`backup/backup.go:330-331`. The real predicate is geometric —
`offy+lineno*fontSize < innerMargin || offy+(lineno+1)*fontSize > plateDims.Y-innerMargin`.
Measured inset line indices: 6.0mm `[0 1 12]`, 5.0 `[0 1 14]`, 4.4 `[0 1 16]`,
3.8 `[0 1 18 19]`, 3.4 `[0 1 2 21 22]`, 3.0 `[0 1 2 24 25]`. **At 3.0mm five rows are
inset, not two.** §4's capacity table was computed with the CORRECT model — an independent
replica reproduces 274/372/492/648/834/1104 exactly only when all those rows are debited.
**So §4 and §5 contradict each other inside one document.** Implemented literally, lines 1-2
and the second-to-last run into the screw-hole bands, and because §5 mandates the same
`widthAt` feed the preview, the confirm screen shows them fitting.

## C2 — §5's "paragraph" ≠ `backup.Paragraph`; blank lines collapse and every `\n` costs 1mm
`backup/backup.go:322`, `:363-367`. Measured y-span at 3.8mm: `[{A},{B}]` 7.012mm;
`[{A},{},{B}]` **8.012mm** — a blank paragraph buys **1mm, not a 3.8mm line**;
`[{A},{},{},{B}]` 9.012mm. Separately `Paragraph{Text:"AAA\nBBB"}` spans 6.012mm vs
`"AAABBB"` 2.212mm — `engrave.String` broke the line itself AND the `\n` consumed a column
from `n`. Two approved-vs-engraved mismatches: a blank line becomes a 1mm gap shifting
everything below up 2.8mm; and every `\n` adds 1mm the uniform-pitch model ignores. At 3.0mm
the bottom line already sits at 81mm against `toPlate`'s 82mm bound (`gui/gui.go:2838`,
`safetyMargin = 3`), so **two newlines overflow it** — admitted by the fit gate, refused
after the confirm screen.

## C3 — §6's monotone-admission guarantee is unmet by §7's step order
Text is entered at step 1; Title, Footer and QR arrive at steps 2-4, and §4's own table shows
each REDUCES capacity (6.0mm: 274 → 238 → 198). Text admitted at step 1 is routinely
invalidated by the operator's own later choices. The spec never says which side gives, and
the live "remaining characters" readout at step 1 has no defined value because the divisor is
not yet known.
**Fix:** move the QR choice BEFORE text entry (`ppQRChoiceFlow`, `gui/passphrase_flow.go:388`
is already this shape) and reserve one row each for title and footer unconditionally.

## I1 — `QRScale` never stated; §4's QR column reproducible only at scale 2
Exactly one production-stroke configuration matches `198 278 384 519 678 897`: **37 modules,
QRScale=2, cumulative with title+footer**. The codebase's actual precedent is scale 3
(`gui/gui.go:434`, `:1953`), which yields `197 268 357 488 636 831` — up to 12% less.
**False-PASS trap:** the identical column also falls out of the TEST params (`mm/3`) with a
**33**-module QR, so a test written with `backup_test.go` params would "confirm" a table
production does not satisfy.

## I2 — The QR step function is content-dependent, not length-dependent
`qr.Encode` is mode-adaptive. Measured: 106 lowercase → 37 modules; 106 **uppercase** → 33;
106 digits → **29**. Alphanumeric boundaries are 26/48/78/115/155, nothing like the byte list.
A 114-char uppercase string is 37 modules; **appending one lowercase letter takes it to 41 —
two version steps from one keystroke.** §6/§11 push toward a hardcoded byte table that is too
pessimistic for uppercase and misses the double-step. **Fix:** obtain size by calling
`qr.Encode(text, qr.L)` and reading `.Size`, never a table.

## I3 — §5 is internally contradictory about spaces
Implemented literally under both readings of "the line is empty", width 12, input
`"NAME    QTY\n  bolt    12\n  nut      4"`: reading A destroys indentation, reading B
preserves it, and nothing in §5 selects B — contradicting the "may be a table" promise. Both
readings also violate step 5: `"aaaa    bbbb"` at width 10 emits `"aaaa   "`, three trailing
spaces, eating 3 of 10 columns in the accounting.

## I4 — The single-wrap-function invariant requires changing `EngraveText`, which the spec never authorizes
`backup/backup.go:322-352` slices at `charPerLine` inside the emit loop; §5 is unachievable
without replacing it with a `[]string` consumer, yet §10 lists only data imports and §8 says
placement is "unchanged". Worse, the unbounded `for len(txt) > 0` is **relied upon**:
`validateDescriptor` and `validateMdmk` build three variants and let `toPlate` reject the
overflowing ones — that is how "TEXT + QR" degrades to "TEXT ONLY"/"QR ONLY".
**Good news:** descriptor/UR text contains no `' '`, so the overlong-token fallback reduces to
the existing char slice — the three `text-*.bin` goldens CAN survive, but only if `widthAt(i)`
reproduces `n` bit-for-bit including the `n < 1 → n = 1` clamp (`:341-343`).

## I5 — `engrave.String` owns `'\n'` too, and `Measure()` is wrong for multi-line strings
`engrave/engrave.go:1524-1528`, `:1537`. Its `\n` resets `dot.X = 0` to the String command's
OWN origin, ignoring per-line `offx` inset and the QR column. Measured: `Measure()` on
`"ABC\nDEFGH"` returns the **last** line's width and **one** em of height regardless of line
count. Three `\n` mechanisms are in play and the spec claims none of them.

## I6 — The newline key leaks into three flows; §7 names the wrong regression surface
Patched an `"enter"` key into the shared function row and ran the suite:
`TestPassphraseKeyboardStaysOnPanel` **PASS** (the test §7 names is not the constraint);
`TestPassphraseKeyboardConstruction` **FAIL** (asserts exactly 4 function-row keys, reveal at
index 2); `TestPassphraseRevealKeyFitsBothLabels` **FAIL**. Neither is named in §7.
`PassphraseKeyboard` is shared with `NewAddressKeyboard` and BIP-85 index entry, and
`ValidatePassphrase` rejects `'\n'` with `ErrNonASCII` — an unconditional newline key gives
the passphrase program a key that silently fails at OK.

## I7 — Per-keystroke rejection contradicts a reviewed decision in this codebase
`gui/passphrase_flow.go:113-118` deliberately does the opposite, with the rationale in source:
*"Over-length is shown rather than clamped: silently dropping keystrokes at 100 would leave
the operator believing a longer passphrase had been entered."* The spec neither adopts nor
rebuts it, never says what the operator SEES on rejection, and "remaining characters" is not
well-defined under word wrap — appending `x` to the last word can cost a whole line while
appending ` x` does not.

## I8 — Menu integration entirely unspecified, and the codebase flags it as a lockstep hazard
`gui/gui.go:146-168`, `:1868`, `:1891-1899`, `:1902`. Seven navigable programs today; the enum
warns `bip85Derive` must stay last and that `npage`/`npages` and `layoutMainPlates`' case list
are keyed in lockstep. `layoutMainPlates` **panics** on a program missing from its case list.
8 pager dots widens `(sz.X+space)*npages-space`. The compile-time guard catches none of this.

## I9 — Title and footer semantics under-specified in three ways
(a) "last line" — plate-absolute or after the text? (b) "centered" — in the full 79mm or in the
inset span? The title sits on a hole line, so full-width centering pushes a long title into the
screw-hole band. (c) The spec never says the title is engraved **verbatim**; the obvious symbol
`backup.TitleString` (`backup/backup.go:49-61`) silently `ToUpper`s and truncates at
`MaxTitleLen = 18`.

## I10 — `widthAt` has no lower bound; §5 step 4 can never terminate
`EngraveText` clamps `if n < 1 { n = 1 }` (`:341-343`) precisely because QR-plus-inset
arithmetic can drive usable width to zero. §5's fallback slices `w[:widthAt(i)]` with no guard:
at `widthAt(i) <= 0` it consumes nothing and appends forever, on a device with no OOM killer.

## M1 — §6 overstates the existing gap
`EngraveText`'s loop is unbounded, but `toPlate` (`gui/gui.go:2833-2841`) is fail-closed against
`safetyMargin = 3mm`, and both callers depend on that refusal for variant fallback. The fit gate
is a UX improvement over "silently drop a variant", not a correctness fix.

## M2 — §4's stroke-width note is misleading
Measured: the Plain grid is IDENTICAL at `0.3*mm` and `mm/3` — `charWidth` derives from the
font, not the stroke. Only the QR columns move.

## M3 — §4's third column is cumulative (includes title+footer) but reads as QR-only.

## N1 — Glyph import verified
`font/sh/sh.bin` decodes to exactly 81 of 95 printable ASCII; missing set is exactly §10's 14.
All advances are 4000 — the face is genuinely monospace, so `charWidth` from `'W'` is exact.

## N2 — `engrave.String` PANICS on a rune absent from the face (`engrave/engrave.go:1531`)
The glyph import must land before the keyboard is wired, and a test should assert every rune
the keyboard can produce decodes in `font/sh`.

## §12 ANSWERS
1. **Overridable size? No — always automatic.** An override re-anchors admission to the current
   size, which is exactly the retroactive-invalidation case §6 rules out. If the want is bigger
   letters, express it as an optional max-lines cap.
2. **Centered title — but say what it is centered in.** Every existing plate centers its title
   (`backup/backup.go:159`, `:247`). The title line is a screw-hole line, so center within
   `[holeChars*charWidth, width-holeChars*charWidth]`, not the full 79mm.
3. **No automatic QR drop — offer it explicitly.** Silently removing the QR changes what a
   scanner returns, the exact property §2 protects. `validateDescriptor` already presents
   "TEXT + QR / TEXT ONLY / QR ONLY" as a ChoiceScreen (`gui/gui.go:432-444`). Name the number:
   measured 1104 → 969 at scale 2, → 831 at scale 3.
4. **Newline on every page of the free-text keyboard, no page of any other.** Panel fit is not
   the constraint (measured PASS); the shared widget is. Per-instance flag, default off.

## WHAT I CHECKED AND FOUND SOUND
- §4's Plain and +title+footer columns reproduced EXACTLY by an independent replica
  (274/372/492/648/834/1104; deltas 36/40/48/52/60/72). `LinesPerPlate = floor(79/fs)` confirmed.
  Every rung's bottom edge lands inside `toPlate`'s 82mm bound (max 81.2mm at 3.4mm); widest line
  81.8mm at 6.0mm — 0.2mm horizontal headroom, tight but real.
- **The circularity worry in the brief: there is none.** `qrsz = QR.Size * StrokeWidth * QRScale`
  depends only on the encoded text and scale, never on `fontSize`. Text → module count →
  `widthAt(fontSize)` → line count → fits? is a straight evaluation per rung.
- `TestPassphraseKeyboardStaysOnPanel` genuinely tap-tests every key on every page in both reveal
  states via real hit rectangles. `TestPassphraseQRIgnoresFingerprints` is the correct
  module-level exemplar.
- §10's MUST-NOT list is real and live: `if qrc.Size > 33` at `backup/backup.go:82-84`.
- `qr.L` is consistent with both existing text-plate callers.
- **Could not break:** no case where `qr.Encode` shrinks on an appended character (module count
  non-decreasing across 1-260 chars, lower and upper) — so §6's monotonicity premise holds, but
  only if the implementer calls the encoder rather than a table. Could not break panel fit with a
  fifth function-row key. Could not reach `widthAt(i) <= 0` at the specified ladder, which is why
  I10 is Important rather than Critical.
