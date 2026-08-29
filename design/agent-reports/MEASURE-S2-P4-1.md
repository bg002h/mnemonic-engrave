# MEASURE-S2-P4-1 — F-423 plate-packing measurement

S2 plan P4.1. Measured against engrave worktree `impl/descriptor-s2` @
`781d10d`, fork worktree `s2/descriptor-arm` @ `fe9475c`. Program:
`scripts/f423-fit-measure/` (this commit).

## Analytic upper bound

`backup.CharsPerLine` / `backup.LinesPerPlate` (backup/backup.go:88-97), at
`plateSize = 85` mm (backup/backup.go:77), production geometry
(`Millimeter = 6400`, `StrokeWidth = 0.3 * 6400`, matching
`cmd/controller/platform_sh2.go:206-213`), and the shipped font size
`fontMM = 3.8` (`backup.plateFontSizeUR`, backup/backup.go:176 — the value
`backup.Text.fontMM()` resolves to when `FontSize` is left zero, which is
what every descriptor/mdmk caller does; gui/gui.go:459,
gui/gui.go:2563-2569). Hand-verified: width = 85·6400 − 2·3·6400 = 505,600
units; `fixedCharWidth` at fontSize 24,320 units = 14,519 units;
505,600 / 14,519 = 34 (int); 505,600 / 24,320 = 20 (int).

```
CharsPerLine = 34
LinesPerPlate = 20
plate char capacity (CharsPerLine * LinesPerPlate) = 680
```

## Test strings

Real md1 strings, not synthesized, per the plan's instruction to grep the
fork's own fixtures:

- `singleStringBoundary` — `md/testdata/vectors/single_string_boundary.phrase.txt`,
  a single-line (non-chunked) md1 string; its name marks it as sitting at the
  boundary of what fits one md1 string before chunking is forced. 95 chars.
- `chunkA`, `chunkB` — `gui/md1_gather_test.go:17-18`, two of the five
  distinct chunks of `wshSortedmultiChunks`, a real chunked md1 set. Each
  chunk is independently a complete, valid md1 string of the exact kind
  `bundlePlate.str` carries verbatim (`bundlePlatePlan`,
  gui/bundle_flow.go:384-402). 85 chars each.

(Several fork `.phrase.txt` vectors in the 90-120 char band render with
internal space-grouping — e.g. `gap_tr_leaf_pkh.phrase.txt` — which
`codex32.MDDataSymbols`'s bech32 charset check rejects; those are a
display/documentation format, not literal wire strings, so they were not
used. The three strings above are literal, single-line, directly
`g.offer()`-able md1 strings, confirmed against `gui/md1_gather_test.go` and
`gui/bundle_test.go`'s own use of the same literals.)

```
singleStringBoundary: len=95 chars, ceil(len/CharsPerLine)=3 lines (analytic estimate)
chunkA: len=85 chars, ceil(len/CharsPerLine)=3 lines (analytic estimate)
chunkB: len=85 chars, ceil(len/CharsPerLine)=3 lines (analytic estimate)
```

(These per-string line counts are the simple analytic estimate
`ceil(len/CharsPerLine)`; `backup.CharsPerLine`'s own doc comment notes real
wrapping needs more lines wherever a line crosses a screw-hole band. The
trial below runs the real wrap/layout code, not this estimate.)

## Trial fit — mechanism and reachability note

The mechanism is `backup.EngraveText(params, plate)` → the same fit-or-error
bounds check `toPlate` applies, which is what the `validateDescriptor` loop
(gui/gui.go:722-736) and `validateMdmk` (gui/gui.go:2543-2593, the function
`bundleEngrave` actually calls per plate) both use.

`toPlate` itself (gui/gui.go:3515-3528) is unexported (`package gui`) and
unreachable from an external module — this was hit and confirmed, not
assumed. Every primitive `toPlate` is built from is exported:
`gui.SquarePlate`/`.Dims`, `gui.ErrTooLarge`, `engrave.PlanEngraving`,
`bspline.Measure`, `bspline.Bounds`/`.In`, `bezier.Pt`/`.Sub`. The one
non-exported value it needs, `safetyMargin = 3` (mm, gui/gui.go:52), is a
bare numeral. `fitCheck` in `scripts/f423-fit-measure/main.go` reproduces
`toPlate`'s five-line bounds check over that exported surface — a minimal
shim, not a fork of private logic — so the trial ran as directed rather than
stopping at the analytic bound alone.

Each `N` builds `N` `backup.Paragraph{Text: <string>}` entries in one
`backup.Text.Paragraphs` slice (no `FontSize` override — shipped font). This
is the plate's native mechanism for visually distinct units: `EngraveText`
inserts a 1mm gap between paragraphs and lays out each paragraph's lines
independently (gui's own comment at backup/backup.go:487-490,
"Space UR sections") — no reflowing of one string into another.

## Trial output (verbatim, `go run .`)

```
=== ANALYTIC UPPER BOUND (backup.CharsPerLine / backup.LinesPerPlate, fontMM=3.8) ===
CharsPerLine = 34
LinesPerPlate = 20
plate char capacity (CharsPerLine * LinesPerPlate) = 680

=== TEST STRINGS ===
singleStringBoundary: len=95 chars, ceil(len/CharsPerLine)=3 lines (analytic estimate)
chunkA: len=85 chars, ceil(len/CharsPerLine)=3 lines (analytic estimate)
chunkB: len=85 chars, ceil(len/CharsPerLine)=3 lines (analytic estimate)

=== TRIAL FIT (backup.EngraveText -> fitCheck, shipped font, FontSize=0) ===
N=1 strings ([singleStringBoundary]): FITS
N=2 strings ([singleStringBoundary chunkA]): FITS
N=3 strings ([singleStringBoundary chunkA chunkB]): FITS
```

## QR-side constraint

`bundlePlate` (gui/bundle_flow.go:371-379) carries fields `cardIdx`,
`cardTotal`, `plateIdx`, `plateTotal`, `str`, `label`, `kind` — no QR field.
`bundlePlatePlan` (gui/bundle_flow.go:384-402) emits one `bundlePlate` per
gathered chunk string, verbatim, with no QR data attached. QR variants
(`TEXT + QR`, `QR ONLY`) are generated downstream, per plate, inside
`validateMdmk` (gui/gui.go:2543-2593) — called once per `bundlePlate.str` by
`bundleEngrave` (gui/bundle_flow.go:456-459) — and offered to the operator as
alternative engravings of that ONE string via a `ChoiceScreen`, not stored on
the plan. `bundlePlatePlan`'s plates are text-only (the gathered string); the
QR-side constraint does not bind earlier and was not measured.

## Verdict

**3 md1 strings fit one plate side at the shipped font** (measured: all of
N=1, N=2, N=3 pass `toPlate`'s bounds check; the plan's directed trial scope
was 1-3 strings, so this is the trial's ceiling, not a claim that N=4+
fails). Trial and analysis agree: the analytic capacity (680 chars / 20
lines) has wide headroom over N=3's usage (265 chars / ~9-11 lines including
paragraph gaps), consistent with all three trial points passing.
