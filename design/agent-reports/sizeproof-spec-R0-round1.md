# R0 architect review — SPEC_sizeproof.md — round 1 (fold check)

Reviewer: independent opus subagent. Dispatched 2026-08-05 against the R1 spec
@ `cb6e093`. Scoped to "did the fold fix each round-0 finding, and did it
introduce a new defect" — not a fresh audit. Persisted VERBATIM before folding.

VERDICT: RED (1 Critical / 6 Important new; 2 round-0 Importants partially closed)

## Round-0 findings

All six round-0 CRITICALS **FIXED**: C1 (§2.1 names the plate-absolute
`isQRLine` and requires a device-unit y-range), C2 (§1's table independently
reproduced: front 78.400 spare 3.600, back 79.600 spare 2.400, limit 82.000),
C3 (§2.4 states start/limit; §5 mandates no footer), C4 (§2.6 re-keys
`faceLayouts`), C5 (§2.3's `Uniform bool`; §2.5 removes fontSize/rows/bodyRows),
C6 (§3 enumerates the GUI surface; §7.10 is the end-to-end assertion).

Importants: I1, I2, I3, I5, I6(signature), I7 FIXED. **I4 PARTIALLY** — the
`MaxCharsAtBlocks`/`rowFaces` rows restate the status quo without prescribing a
change, and two sites are still absent: `gui/preview.go:130` hardcodes
`ftProofFooter` for every proof preview, and `cmd/plateview` `sizeLabel` still
prints "fixed layout" for `SizeMM == 0`. **I8 PARTIALLY** — (e) is missing: no
§7 item exercises the `len(Sizes) != len(Lines)` guard.

Minors M1-M5 and Nits N1-N2 FIXED, with residue noted below.

## NEW — CRITICAL

**§2.1 + §2.4 — the `qrTop`/`qrBottom` ANCHOR is unspecified, and the two
existing consumers require OPPOSITE anchors; the branch an implementer will
naturally take re-creates C1's exact failure.** `EngraveText` (descriptor/seed,
backup.go:385) draws its QR at `lay.baseY + lay.holeLines*lay.fontSize` — a
BASEY-relative window. `EngraveFitted` (freetext.go:85) draws at
`margin + lay.holeLines*fontSize` — MARGIN-relative, correct today only because
`wrapBlocks` passes `outerMargin` as `baseY` for every block. §2.4 changes that
to the running `y`. §2.1 never says which anchor, and never says what replaces
freetext.go:85 once §2.5 removes `fontSize`.
*Failure:* implementer computes `qrTop = baseY + holeLines*fontSize` (the field
is right there, and it preserves the descriptor path). Operator loads
`BOTHPROOF!` with the QR forced off, presses Back to the QR screen — `plan` and
`text` survive Back by design — chooses "Add QR", trims until it fits. Block 1
gets the right window; block 2's window sits at its own `baseY`, so its rows
beside the QR are wrapped at the full 44 columns and **engraved straight across
the QR**, on a plate whose QR is a machine-readable copy of the text.
*No test catches it:* §7.7 is a single-block title+QR plate; `freetext-0-plain`
and `freetext-1-qr` are both one block, and every shipped descriptor plate is a
single paragraph, so the margin-anchored branch is equally unpinned.
*Fix:* the QR y-range is PLATE-ABSOLUTE for the free-text fitter and
BLOCK-RELATIVE for the paragraph fitter — `textLayout` takes the QR's own top y
as a parameter rather than deriving it from `baseY`. Pin freetext.go:85 and
backup.go:385 as reading that same field, and add a two-block-plus-QR test
asserting no body ink enters the code box.

## NEW — IMPORTANT

1. **§2.4's `limit` and §2.5's footer y are mutually inconsistent**, so §7.8's
   own property fails on any mixed plate with a footer. `limit = plateHeight -
   margin - F(footer)` reserves the bottom strip; §2.5 places the footer at
   `margin + (LinesPerPlate(footer)-1)*F(footer)`, higher by the remainder. At
   3.8 mm: limit 78.200, footer ink 75.200–79.000 → a body ending legally at the
   limit **overlaps the footer by 3.0 mm**. Uniform plates are safe by
   arithmetic accident, so no golden moves. Round 0 handed over both formulas
   and they do not compose; the spec folded both verbatim. *Fix:* limit is the
   footer's TOP y; exercise §7.8 on a mixed plate with a 3.8 mm footer.
2. **§2.5's footer y divides by zero on the no-footer case §5 mandates.**
   `LinesPerPlate(params, 0)` is `height / 0`. Every ladder plate has
   `FooterSizeMM == 0`, as does every operator plate that skips the Footer
   field; neither golden covers it. Symmetrically `Title != "" &&
   TitleSizeMM == 0` makes `centerInset` call `textLayout` at fontSize 0.
   *Fix:* state the invariant that the sizes are non-zero exactly when the
   strings are non-empty, enforced at construction.
3. **§2.5 never says how `EngraveFitted` obtains the per-row screw-hole inset
   (`offx`) once `rows`/`start` are gone.** freetext.go:71 is
   `lays.at(...).at(start + i)`, a plate-absolute index. The fit computes `offx`
   block-relative; the natural rewrite keeps `start + i`, which agrees on a
   uniform plate and diverges on a mixed one — fit/engrave drift invisible to
   any assertion on size, lines or code. *Fix:* `EngraveFitted` builds each
   row's layout at `baseY = y_i` and reads `at(0)`.
4. **§2.7 drops `useQR` from `FitSized`, but nothing on the caller side stops
   the operator's QR choice being silently dropped.** `ftStepQR` runs first, so
   `useQR` can be true when the trigger is typed. Today the drop is prompted,
   and only via `NeedsWholePlate()` (`TextQR == ""`); §3/§4 never mention it.
   "FitSized refuses a QR outright" is not implementable with no parameter.
   *Fix:* require the ladder proofs to set `TextQR == ""` so the existing
   prompted drop applies.
5. **§1's headline rationale is false, in the section presented as MEASURED.**
   "Six of the ten (face, rung) pairs need a row more than the naive count" does
   not reproduce: measured, ZERO of ten exceed `ceil(95/CharsPerLine)` in the
   titled configuration, and exactly ONE does untitled (`sh@5.0`, budgets
   `[20 20 26 26 26]`). RECON §3's per-pair counts are IDENTICAL to the measured
   ones; the whole delta between RECON's 71.6/73.6 and §1's 78.40/79.60 is the
   outer margin plus the title row. "Every figure in the previous draft was
   wrong" is itself wrong. The number was inherited unverified from the round-0
   controller's note. The table is right; the sentence telling a reader to trust
   it is not.
6. **§2.2/§2.4/§6 leave undecided whether `wrapBlocks` honours `Block.SizeMM`
   over the passed `fontMM`**, and §6 asserts one side as settled. If it does,
   `AdmissibleBlocks` stops being anchored at 3.0 mm for ladder plates and §6's
   "the readout will report the 3.0 mm anchor" is false. If it does not,
   `FitSized` needs another channel. *Fix:* pick one and restate §6.

## NEW — MINOR / NIT

- `Uniform bool` has the unsafe zero value: `false` means "mixed", so every
  hand-built `Fitted` literal defaults to the dangerous branch — including
  `EngraveFreeText`, the only constructor the load-bearing goldens use. Prefer
  `Mixed bool`.
- §5's "83.4 mm > 82" is the front WITHOUT a title; with it the figure is 82.2
  and under §2.5's footer y it is a 3.0 mm overlap, not an overflow. The back's
  overlap is 1.6 mm, not 1.4.
- §1's "475 characters" — ten (face, rung) blocks x 95 = 950.
- `gui/preview.go:130` hardcodes `ftProofFooter` for every proof preview.
- `cmd/plateview` `sizeLabel` prints "fixed layout" for `SizeMM == 0`.
- §5/§7.12 never state the title's FACE (`blocks[0].Face` = sh on both sides) or
  the inset-span fit; measured, both fit (13 <= 26 at sh 3.8, 16 <= 36 at sh 3.0).
- LEXICON's inventory lists `PASSPROOF!` as live; the shipped root is
  `FONTPROOF!` and the rename is Open at the bottom of the same file.
- §2.4 names one quantity `limit` and `maxY`. §3 cites `ftFaceSummary` at
  flow.go:487; it is at flow.go:146.
