# f434-fit-measure

F-434's real-fix measurement (`design/FOLLOWUPS.md` F-433/F-434): if
`backup.EngraveText` advanced a paragraph by `max(textLines, qrLines)`
instead of text lines only (the fix F-434 names but has not built), how many
(string + QR) pairs would fit one 85mm SeedHammer plate side, for md1- and
mk1-sized strings, TEXT+QR and QR-ONLY? `go run .` prints the arithmetic
(real QR module count via the fork's own `qr.Encode`, the QR band height via
`qrPlaceAt`'s own formula replicated in device units, the text line count
via `backup.CharsPerLine`, both packing budgets) and then trial-fits every
N=1 configuration through the real `backup.EngraveText` -> `toPlate`
mechanism (`fitCheck`, reused verbatim from `scripts/f423-fit-measure`),
plus confirms `backup.ErrMultiParagraphQR` still refuses the N=2 case at
this fork rev. Full output is pasted verbatim in
`design/agent-reports/MEASURE-F434.md`.

`go.mod` pins the fork worktree via a local `replace` directive:

```
replace seedhammer.com => /scratch/code/shibboleth/seedhammer
```

measured at fork rev `5f02773` (branch `main`), mnemonic-engrave rev
`5f02773`. Re-run against a different fork rev by updating that path (or
vendoring); the numbers in the persisted report are only valid as of that
rev.

## Why the multi-pair (N>=2) number is arithmetic-only

`backup.EngraveText` refuses outright (`ErrMultiParagraphQR`,
backup/backup.go:378) any plate with more than one paragraph where a
paragraph carries a QR -- the CHEAP HALF of F-434, already shipped. So an
N>=2 arrangement cannot be constructed through the real API at all: the
program's own trial confirms this refusal fires on N=2 for every string
tested. The N>=2 packing numbers are therefore computed directly from the
same primitives `EngraveText` itself reads (`qrPlaceAt`'s formula,
backup/wrap.go:196-212; `WrapText`'s line count via `backup.CharsPerLine`),
not from a call into `EngraveText`. N=1 (both TEXT+QR and QR-ONLY, both
string sizes) IS trial-fit through the real mechanism, since a single
QR-carrying paragraph is exactly what `validateMdmkStrings`/
`validateDescriptor` construct and offer in production today.

## Two budgets, one binding

The program reports the packing arithmetic against two different vertical
budgets:

- **The F-435 body budget (68.4mm)**, `(LinesPerPlate-2)*fontMM` -- the same
  worst-case title+footer mark (`bundlePlateFitMark`, 18 `W`s) that
  `bundlePlateTextFits` (gui/bundle_flow.go, called from `bundleCardPlates`)
  already packs EVERY bundle plate against, marked or not, by its own stated
  design ("THE PLATE COUNT MAY NOT DEPEND ON THE MARKING... packing against
  the worst case makes the answer the same for all three readers",
  gui/bundle_flow.go:443). This is the number that actually governs how many
  strings/pairs the packer puts on one plate today, and would continue to
  under the real fix.
- **The raw content height (79.0mm)**, `plateSize - 2*outerMargin`, no
  title/footer reserved -- reported because the dispatch brief asks for the
  arithmetic against both "the plate's content height and the F-435 body
  budget", and because most bundle callers DO ship an unmarked plate
  (`bundleEngrave(ctx, th, ..., "", "")` at every call site but
  `singlesig.go:261`, confirmed by grep). It is NOT what decides the
  packer's plate count, for the reason above.

`fitCheck` (gui/gui.go:3515-3528's bounds check, reproduced over its own
exported dependencies -- `gui.SquarePlate`/`.Dims`, `gui.ErrTooLarge`,
`engrave.PlanEngraving`, `bspline.Measure`/`.Bounds`, `bezier.Pt`/`.Sub`,
`safetyMargin=3` inlined as a bare numeral) is identical to
`scripts/f423-fit-measure`'s shim -- a minimal reproduction of `toPlate`'s
own public-surface primitives, not a fork of private logic.

## Real strings

`design/journeys/out/pathological/backup-strings.txt` (this repo) carries
19 mk1 rows at exactly 111 chars and md1 rows topping out at 81 chars (no
85-char row). Line 1 (81-char md1) and line 7 (111-char mk1) are used
verbatim. Because the dispatch brief and F-439 (`design/FOLLOWUPS.md`) both
name an exact "85-char md1" figure, that exact length is cross-checked
separately against the fork's own 85-char md1 fixture
(`gui/md1_gather_test.go:17`, `wshSortedmultiChunks[0]` -- the same fixture
`scripts/f423-fit-measure`'s predecessor program used, real and not
synthesized, confirmed 85 chars by direct `len()`). Both md1 lengths (81 and
85 chars) land on the identical QR module count (37) and text line count (3)
at this font/CharsPerLine, so the cross-check changes nothing about the
answer.
