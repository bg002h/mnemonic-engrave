# MEASURE-F434 — F-434 real-fix packing measurement

Measured against fork `seedhammer` `main` @ `5f02773`, mnemonic-engrave
`master` @ `5f02773`. Program: `scripts/f434-fit-measure/` (this commit).

## The question

F-433/F-434 (`design/FOLLOWUPS.md`): if `backup.EngraveText` advanced a
paragraph by `max(textLines, qrLines)` instead of text lines only (the real
fix F-434 names but has not built), how many (string + QR) pairs would fit
one 85mm plate side, for md1- and mk1-sized strings, TEXT+QR and QR-ONLY?
The plan's rule: **N>=2 in any scannable configuration → the fix proceeds;
N=1 everywhere → it does not.** This report states the measured N values
only; it makes no recommendation.

## Method summary

- **QR module count**: real, from the fork's own `qr.Encode(text, qr.L)`
  (`github.com/seedhammer/kortschak-qr`), never assumed. `qr.L` and
  `qrScale=3` are what both real QR-carrying callers use today
  (`validateDescriptor`, gui/gui.go:733-734; `validateMdmkStrings`,
  gui/gui.go:2617,2623).
- **QR band height (`qrLines`)**: `qrPlaceAt`'s own formula
  (backup/wrap.go:196-212), replicated in device units since `qrPlaceAt` is
  unexported: `qrLines = ceil((qrsz + 2*qrBorder) / fontSize)`,
  `qrBorder = 2mm` (wrap.go:199, a bare numeral, confirmed by reading the
  source rather than `outerMargin`).
- **Text line count**: `ceil(len(s) / backup.CharsPerLine(...))`
  (`backup.CharsPerLine` is a real exported call). Cross-checked against the
  one real repo citation available at this exact string size:
  `gui/bundle_flow.go`'s own comment ("measured: with three 85-char chunks,
  paragraph 0's code spans y 67840..311040") gives a QR band of
  `(311040-67840)/6400 = 38.0mm` for an 85-char md1 string at this font/scale
  — an EXACT match to this program's own computed 38.0mm QR band for the
  same string size, and the same real module count F-439 recorded (37
  modules md1, 41 modules mk1) — both independently confirmed here via
  `qr.Encode`, not copied.
- **Pair height under the real fix**: `max(textLines, qrLines) * fontMM` mm
  (TEXT+QR); `qrLines * fontMM` (QR-ONLY, no text row to max against).
- **Stacking**: N pairs of height `h`, the SAME 1mm inter-paragraph gap
  `EngraveText` already applies (backup/backup.go:505-508), against a
  budget: `N <= floor((budget+1)/(h+1))`.
- **Budget**: two are reported (see `scripts/f434-fit-measure/README.md`'s
  "Two budgets, one binding" section for the full citation trail):
  - **F-435 body budget, 68.4mm** — `(LinesPerPlate-2)*fontMM = 18*3.8`.
    This is the packer's OPERATIVE bound: `bundlePlateTextFits` packs every
    bundle plate against the worst-case title+footer mark
    (`bundlePlateFitMark`) regardless of whether that plate ends up
    genuinely marked, by its own stated design ("THE PLATE COUNT MAY NOT
    DEPEND ON THE MARKING", gui/bundle_flow.go:443).
  - **Raw content height, 79.0mm** (`plateSize - 2*outerMargin`, footerless)
    — informational only, NOT the packer's bound (most bundle callers DO
    ship an unmarked plate — `bundleEngrave(..., "", "")` at every call site
    but `singlesig.go:261` — but the packer's plate-count decision does not
    depend on that).

## Numbers table (real modules, real trial-fit for N=1; N>=2 arithmetic-only — refusal confirmed live)

| String | Config | QR modules (real) | QR band | Text lines | Pair height (real-fix advance) | N @ 68.4mm (packer's bound) | N @ 79.0mm (footerless, informational) | N=1 trial-fit |
|---|---|---|---|---|---|---|---|---|
| md1, 81 chars (backup-strings.txt:1) | TEXT+QR | 37 | 38.0mm (10 lines) | 3 lines / 11.4mm | 38.0mm | **1** | 2 | FITS (real `EngraveText`→`toPlate`) |
| md1, 81 chars | QR-ONLY | 37 | 38.0mm | — | 38.0mm | **1** | 2 | FITS |
| md1, 85 chars (gui/md1_gather_test.go:17, cross-check) | TEXT+QR | 37 | 38.0mm | 3 lines / 11.4mm | 38.0mm | **1** | 2 | FITS |
| md1, 85 chars (cross-check) | QR-ONLY | 37 | 38.0mm | — | 38.0mm | **1** | 2 | FITS |
| mk1, 111 chars (backup-strings.txt:7) | TEXT+QR | 41 | 41.8mm (11 lines) | 4 lines / 15.2mm | 41.8mm | **1** | 1 | FITS |
| mk1, 111 chars | QR-ONLY | 41 | 41.8mm | — | 41.8mm | **1** | 1 | FITS |

N=2 (one paragraph QR-carrying, one text-only) was attempted through the
real `backup.EngraveText` for every string above and refused every time —
`backup.ErrMultiParagraphQR` ("a QR belongs to a plate of its own; this
plate has more than one paragraph") — confirming the N>=2 packing numbers
above are geometry computed from `EngraveText`'s own primitives, not
something the real API can be made to construct today.

## Verbatim program output (`go run .`, fork `5f02773`)

```
=== PLATE GEOMETRY (fontMM=3.8, qrScale=3, qrLevel=L) ===
CharsPerLine (backup.CharsPerLine, real call) = 34
LinesPerPlate (backup.LinesPerPlate, real call) = 20
F-435 body budget (title+footer marked, matching bundlePlateTextFits's own trial-fit convention): (LinesPerPlate-2)*fontMM = 18*3.8 = 68.4mm
Plate raw content height (no title/footer, yBudget's footerless branch): plateSize-2*outerMargin = 85-6 = 79.0mm -- NOT the packer's bound; see note above

=== PER-STRING ARITHMETIC (real qr.Encode module count; qrLines/textLines by qrPlaceAt/WrapText's own formulas) ===
md1 (backup-strings.txt:1, 81 chars): len=81, QR modules=37 (real qr.Encode), QR size=33.30mm, QR band=10 lines=38.0mm, text=3 lines=11.4mm, pair advance under real fix=max(text,qr)=38.0mm
md1 (gui/md1_gather_test.go:17, 85 chars, cross-check): len=85, QR modules=37 (real qr.Encode), QR size=33.30mm, QR band=10 lines=38.0mm, text=3 lines=11.4mm, pair advance under real fix=max(text,qr)=38.0mm
mk1 (backup-strings.txt:7, 111 chars): len=111, QR modules=41 (real qr.Encode), QR size=36.90mm, QR band=11 lines=41.8mm, text=4 lines=15.2mm, pair advance under real fix=max(text,qr)=41.8mm

=== PACKING ARITHMETIC (ARITHMETIC ONLY for N>=2 -- see refusal confirmation below) ===
N pairs of height h, 1mm inter-paragraph gap: N <= floor((budget+1)/(h+1))
-- against the F-435 body budget (68.4mm), the packer's ACTUAL bound (bundlePlateTextFits packs every plate against this, marked or not):
md1 (backup-strings.txt:1, 81 chars): TEXT+QR pairs/plate = 1 (pair height 38.0mm) | QR-ONLY pairs/plate = 1 (pair height 38.0mm)
md1 (gui/md1_gather_test.go:17, 85 chars, cross-check): TEXT+QR pairs/plate = 1 (pair height 38.0mm) | QR-ONLY pairs/plate = 1 (pair height 38.0mm)
mk1 (backup-strings.txt:7, 111 chars): TEXT+QR pairs/plate = 1 (pair height 41.8mm) | QR-ONLY pairs/plate = 1 (pair height 41.8mm)
-- against the raw content height (79.0mm, footerless -- informational only, NOT the packer's bound, see note above):
md1 (backup-strings.txt:1, 81 chars): TEXT+QR pairs/plate = 2 (pair height 38.0mm) | QR-ONLY pairs/plate = 2 (pair height 38.0mm)
md1 (gui/md1_gather_test.go:17, 85 chars, cross-check): TEXT+QR pairs/plate = 2 (pair height 38.0mm) | QR-ONLY pairs/plate = 2 (pair height 38.0mm)
mk1 (backup-strings.txt:7, 111 chars): TEXT+QR pairs/plate = 1 (pair height 41.8mm) | QR-ONLY pairs/plate = 1 (pair height 41.8mm)

=== TRIAL FIT (real backup.EngraveText -> fitCheck; the mechanism validateMdmkStrings/validateDescriptor use today) ===
md1 (backup-strings.txt:1, 81 chars) N=1 TEXT+QR: FITS
md1 (backup-strings.txt:1, 81 chars) N=1 QR-ONLY: FITS
md1 (backup-strings.txt:1, 81 chars) N=2 (one paragraph QR-carrying): backup.EngraveText refuses -- backup: a QR belongs to a plate of its own; this plate has more than one paragraph (paragraph 1 of 2 carries one)
md1 (gui/md1_gather_test.go:17, 85 chars, cross-check) N=1 TEXT+QR: FITS
md1 (gui/md1_gather_test.go:17, 85 chars, cross-check) N=1 QR-ONLY: FITS
md1 (gui/md1_gather_test.go:17, 85 chars, cross-check) N=2 (one paragraph QR-carrying): backup.EngraveText refuses -- backup: a QR belongs to a plate of its own; this plate has more than one paragraph (paragraph 1 of 2 carries one)
mk1 (backup-strings.txt:7, 111 chars) N=1 TEXT+QR: FITS
mk1 (backup-strings.txt:7, 111 chars) N=1 QR-ONLY: FITS
mk1 (backup-strings.txt:7, 111 chars) N=2 (one paragraph QR-carrying): backup.EngraveText refuses -- backup: a QR belongs to a plate of its own; this plate has more than one paragraph (paragraph 1 of 2 carries one)
```

## Notes, not recommendations

- Text height never governs the pair height at either string size: the QR
  band (38.0mm md1, 41.8mm mk1) exceeds the text height (11.4mm md1, 15.2mm
  mk1) by a wide margin, so TEXT+QR and QR-ONLY land on the IDENTICAL pair
  height and IDENTICAL N at both budgets — adding the text costs nothing
  once the QR is present.
- The two md1 lengths tested (81 chars, real; 85 chars, cross-check fixture)
  produce identical QR module counts (37) and identical text line counts
  (3), so the 81-vs-85 gap in `backup-strings.txt` does not change any
  number in this report.
- Against the packer's actual bound (68.4mm), N=1 in all six rows. Against
  the footerless raw content height (79.0mm, not the packer's bound), md1
  reaches N=2 in both configurations while mk1 stays at N=1.
