# Opus architect — R0 round 1 (fold check), Engrave Text spec rev 1

Scope: did the fold fix each round-0 finding, and did it introduce a new defect?
Baseline suite green (exit 0); all probing in a scratch copy at `abb7458`.

## VERDICT
**NOT GREEN (1C / 4I)** + 4 Minor, 1 Nit.

## FOLD CHECK
- **C1 FIXED.** §5.1's inset table verified against `backup/backup.go:330-331` at production
  params — **all six rungs match exactly**, as do `holeChars` 2/3/3/4/4/4 and the grids.
- **C2 FIXED.** §5.3 forbids the `backup.Paragraph` mapping; §11 pins it. 3.0mm grid bottom is
  exactly 81.0mm against the 82mm bound.
- **C3 PARTIAL.** QR first ✓, rows reserved ✓, and one row is exactly enough per rung
  (`widthAt(0) == widthAt(last) ==` 18/20/24/26/30/36). But title/footer **width** stayed out of
  the fit test while that limit varies 18→36 → **new C1**.
- **I1 FIXED (scale-2 half).** `198 278 384 519 678 897` reproduces exactly at 37 modules,
  `QRScale=2`, production stroke, cumulative. The `mm/3`+33-module false-PASS also reproduces, so
  §11's prohibition is warranted. Residues → M1, new I1.
- **I2 PARTIAL.** Encoder-not-table mandated ✓; 106 lower→37 / upper→33 / digit→29 confirmed. But
  "114-char uppercase is 37 modules" is **wrong — measured 33** → new I2.
- **I3 FIXED.** One space rule; the trailing-space defect is gone. Residue at M2.
- **I4 FIXED.** Refactor authorized, unbounded path preserved, golden identity pinned and
  achievable — all three `text-*` inputs are space-free, so greedy fill reproduces `txt[:n]`.
- **I5 FIXED.** Verified `engrave.go:1517-1537`. Also confirmed `Decode('\n')` is false but
  `String` short-circuits before `Decode`, so §3 and §5 are consistent.
- **I6 NOT FIXED (test claim).** Design fix landed; the measurement did not → new I3.
- **I7 FIXED.** Adopts the sibling program's reviewed decision; "lines used / available".
- **I8 FIXED.** All lockstep line references verified real. Nit at N1.
- **I9 PARTIAL.** Plate-absolute ✓, inset-span centering ✓, verbatim ✓. Rung dependence → C1;
  footer alignment → M4.
- **I10 FIXED.** Sibling at new I4.
- **M1/M2/M3 fixed** (Plain grid confirmed identical at both stroke values).
  **N1/N2 recorded** — 81/95 decode, missing set exactly §10's 14, every advance 4000.

## NEW C1 — the title/footer limit is rung-dependent (18…36) but §2 states one number, and no fit test constrains it
`charsPerLine − 2·holeChars` = 18/20/24/26/30/36 down the ladder. §6 anchors admission at
**3.0mm**, where it is 36 — but the engraved size is the *largest* rung that fits, and the fit
test counts rows only. A short text plus a long title auto-fits at 6.0mm with a title that row
cannot hold. **Measured, centered per §8 at 6.0mm:** 18 chars → ink `x[10.709, 74.380]`mm, clear
of both screw-hole bands (`[3,10]`, `[75,82]`); **20 chars → `x[7.127, 77.962]` — crosses both
bands, and `toPlate` does NOT reject it.** Silently wrong plate: the confirm screen shows a
fitting title, the machine cuts through the screw holes. 26 chars is off-plate (post-confirm
refusal); 36 gives `x[-21.529, 106.618]`. Enforcing at the *current* rung instead is no better —
deleting text raises the rung and retroactively invalidates an entered title, the exact case
§6/§12.1 exist to rule out.
**Fix:** cap title and footer at the 6.0mm figure (**18**) unconditionally, or make their width
part of the per-rung fit test. Pin: a title at the stated limit sits inside
`[innerMargin, plateSize − innerMargin]` at *every* rung.

## NEW I1 — §4's QR column is unattainable: the QR encodes the Text, so 37 modules contradicts the column's own character counts
37 modules is v5-L: ≤106 bytes / 154 alphanumeric. §2 requires the QR to encode the Text, so a
text of the column's length can never yield a 37-module QR. Measured true maxima (fixed point
text↔QR, production stroke, scale 2, rows reserved):

| rung | §4 says | real (lower) | real (upper) |
|---|---|---|---|
| 6.0 | 198 | 178 | 195 |
| 5.0 | 278 | 230 | 255 |
| 4.4 | 384 | 284 | 318 |
| 3.8 | 519 | 367 | 398 |
| 3.4 | 678 | 429 | 501 |
| 3.0 | **897** | **520** | **616** |

§6's live-encode rule still yields the right answer, so this is not a wrong plate — but §6's
**refusal message is built from these figures and is badly wrong when it fires**: measured at
3.0mm with QR on, a 700-char text encodes to an 89-module QR leaving 392 columns beside it, so
dropping the QR frees **640** characters; the spec tells the implementer to say **135**.
**Fix:** relabel the column as geometry at a hypothetical 37-module QR, state the real per-rung
maxima (or that they must be solved by iteration), and derive the refusal number live.

## NEW I2 — "a 114-char uppercase string is 37 modules" is wrong; measured **33**
Contradicts §6's own boundary list two lines earlier (115 is the first length needing 37).
33 → 41 is still two versions, so the narrative holds; the number does not. §11 makes this a
required test, so an implementer coding it from §6 gets a red test and must guess who is
authoritative. **Fix:** `33 → 41`.

## NEW I3 — §7.1 names two tests as breaking; under its own opt-in design **neither** does, and "updating" one removes the anti-leak guard
Round 0 measured with an *unconditional* key; rev 1 adopted the opt-in fix but kept the old
measurement. Implemented the folded design in a scratch copy (`newPPKeyboard(ctx, newline bool)`,
`NewPassphraseKeyboard`→false, `NewTextKeyboard`→true): **`go test ./gui/` exit 0** — both named
tests pass, because both call `NewPassphraseKeyboard`, unchanged. Appending unconditionally at
index 4 fails **only** `TestPassphraseKeyboardConstruction`; both fail only when inserted before
index 2, a position §7.1 never specifies.
`TestPassphraseKeyboardConstruction:24` asserts the shared function row has **exactly 4 keys** —
precisely the guard against an accidental unconditional newline key leaking into
`NewAddressKeyboard` and BIP-85. §7.1 instructs the implementer to disable it, for a change that
does not require it. **Fix:** no existing keyboard test changes; add *new* tests for the
free-text variant; keep "exactly 4" as the anti-leak guard; state the newline key's function-row
position (the reveal key's index is asserted at `passphrase_keyboard_test.go:200`).

## NEW I4 — `WrapText`'s `widthAt` index base is ambiguous; for free text the two readings differ by one
The contract calls `i` *plate-absolute* while asserting only `i < maxLines`. `WrapText` iterates
its own output lines, so literally output line 0 lands on plate row 0 — the title's row — and the
assertion never covers the last text row. The same ambiguity hits `EngraveText`'s QR predicate
`holeLines <= lineno < holeLines+qrLines`, and the descriptor callers use a third base
(per-paragraph `offy`). Implemented literally: the first text line collides with the title, or fit
and engraving disagree by one row — the approved-vs-engraved class §5 exists to prevent, and §11
pins no offset. **Fix:** index by *output* line with the caller supplying the plate-row offset in
the closure (free text `i+1`; descriptor the paragraph's `offy`), or add an explicit `firstRow`
parameter; match the `>= 1` assertion range to whichever is chosen.

## MINOR / NIT
- **M1** — §4's scale-3 footnote `197 268 357 488 636 831` is not the same column: measured
  cumulative-with-reservations at scale 3 is **`161 228 309 436 576 759`**. `197…831` is the
  geometry *without* the two reserved rows — a round-0 figure the new unconditional reservation
  invalidated. §6's `1104`/`969`/`831` share that stale basis.
- **M2** — §5.2 step 5's clauses collide: a line whose entire content is a preserved *leading*
  space run either ends in a space (violating one clause) or loses the run (violating the other).
  §11 pins both properties in one test.
- **M3** — `EngraveText` hardcodes `fontSize := params.F(plateFontSizeUR)` (`backup.go:284`).
  §10 imports `Text.FontSize` but never says the **zero value must fall back to
  `plateFontSizeUR`** — which is exactly what keeps the three `text-*` goldens byte-identical,
  since every existing caller constructs `Text` without it.
- **M4** — the footer's alignment is never stated; §12.2 covers only the title.
- **N1** — §7.2 over-states site 2: `npage`/`npages` are `int(bip85Derive)+1`, derived, so they
  need no hand edit. Only the enum and `layoutMainPlates`' case list do.

## VERIFIED BY MEASUREMENT
§5.1's table on all six rungs; §4's Plain (274…1104) and +title+footer (238…1032) exact; the QR
column exact at 37 modules/scale 2/production stroke and the `mm/3`+33 false-PASS reproduced;
scale-3 cumulative = 161…759; the QR fixed-point maxima above; 106 lower/upper/digit → 37/33/29
and 114 upper → 33, +1 lowercase → 41; §4.1's bounds hold conservatively (grid bottoms
81.0/78.0/77.8/79.0/81.2/81.0mm, right edge 81.8047mm at 6.0mm, true ink maxima x=81.622,
y=80.337 inside `toPlate`'s [3,82] box); the keyboard claim three ways; font 81/95 with exactly
§10's 14 missing and all advances 4000; `Measure()` returns last-line width and one lheight;
title overflow at 6.0mm (18 clear, 20 crossing both bands with `toPlate` passing).
