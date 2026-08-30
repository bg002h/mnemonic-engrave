// Command f434-fit-measure answers F-434's real-fix measurement: how many
// (string + QR) pairs fit one 85mm SeedHammer plate side under the
// advance-by-full-paragraph-height fix (max(textLines, qrLines) per
// paragraph, backup/backup.go:373), for md1- and mk1-sized strings, in both
// TEXT+QR and QR-ONLY configurations.
//
// TODAY, backup.EngraveText REFUSES any multi-paragraph plate where a
// paragraph carries a QR (ErrMultiParagraphQR, backup/backup.go:378) -- the
// CHEAP HALF of F-434, already shipped. So the multi-pair (N>=2) geometry
// cannot be exercised through the real API: this program computes it
// ARITHMETICALLY, from the same primitives EngraveText itself reads
// (qrPlaceAt's formula, backup/wrap.go:196-212; WrapText's line count via
// backup.CharsPerLine), and separately TRIAL-FITS every N=1 configuration
// through the real backup.EngraveText -> toPlate path (both configurations
// are what validateMdmkStrings/validateDescriptor offer in production
// today), plus confirms the N=2 refusal is still live. Which numbers are
// arithmetic and which are trial-fit is stated at each printout.
package main

import (
	"fmt"
	"math"

	qr "github.com/seedhammer/kortschak-qr"
	"seedhammer.com/backup"
	"seedhammer.com/bezier"
	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
	"seedhammer.com/font/sh"
	"seedhammer.com/gui"
)

// Production geometry, IDENTICAL to scripts/f423-fit-measure/main.go's block
// (same citations: cmd/controller/platform_sh2.go, mm=6400, strokeWidth=0.3mm;
// gui_test.go/backup_test.go's own golden tests use the same mm=6400).
const (
	mm             = 6400
	strokeWidth    = 0.3 * mm
	topSpeed       = 30 * mm
	engravingSpeed = 4 * mm
	acceleration   = 250 * mm
	jerk           = 2600 * mm
)

var params = engrave.Params{
	StrokeWidth: strokeWidth,
	Millimeter:  mm,
	StepperConfig: engrave.StepperConfig{
		TicksPerSecond: topSpeed,
		Speed:          topSpeed,
		EngravingSpeed: engravingSpeed,
		Acceleration:   acceleration,
		Jerk:           jerk,
	},
}

// plateFontSizeUR is backup.plateFontSizeUR (backup/backup.go:176,
// unexported) -- the shipped default every descriptor/mdmk/bundle caller
// resolves to when FontSize is left zero (backup.Text.fontMM,
// backup/backup.go:56-63). Quoted as a literal per the same rule f423's
// program states: the S2/F-434 measurements never lower this floor.
const plateFontSizeUR = 3.8

// safetyMargin mirrors gui.go:52's unexported const of the same name and
// value, toPlate's own safety inset in millimeters on all four sides.
const safetyMargin = 3

// qrScale=3 is what BOTH real QR-carrying callers pass to backup.Paragraph
// today: validateDescriptor (gui/gui.go:733, "const qrScale = 3") and
// validateMdmkStrings (gui/gui.go:2617, "const qrScale = 3"). It is the
// scale a packed plate's QR pairs would use too -- the same paragraph
// mechanism, same caller family.
const qrScale = 3

// qrLevel = qr.L is what both real callers encode at (gui/gui.go:734,
// gui/gui.go:2623 -- "qr.Encode(strs[0], qr.L)" / "qr.Encode(desc..., qr.L)").
const qrLevel = qr.L

// qrBorder mirrors wrap.go:199's qrPlaceAt, "qrBorder := params.I(2)" -- a
// bare numeral (2mm), not outerMargin, confirmed by reading the source
// rather than assumed.
const qrBorderMM = 2

// outerMargin mirrors backup.go:73's unexported const of the same name and
// value: the plate's margin on all four sides, in millimeters.
const outerMarginMM = 3

func devF(valMM float64) int {
	return int(math.Round(valMM * mm))
}

// fitCheck reproduces gui.toPlate's fit-or-error bounds check
// (gui/gui.go:3515-3528) over its own exported dependencies, identical to
// f423-fit-measure/main.go's fitCheck (same citation).
func fitCheck(plan engrave.Engraving) error {
	sz := gui.SquarePlate.Dims(params.Millimeter)
	spline := engrave.PlanEngraving(params.StepperConfig, plan)
	attrs := bspline.Measure(spline)
	margin := bezier.Pt(safetyMargin*params.Millimeter, safetyMargin*params.Millimeter)
	if !attrs.Bounds.In(bspline.Bounds{Min: margin, Max: sz.Sub(margin)}) {
		return gui.ErrTooLarge
	}
	return nil
}

// Real strings. md1PathLine1 and mk1PathLine7 are grepped verbatim from
// design/journeys/out/pathological/backup-strings.txt (mnemonic-engrave
// repo) -- the file this program's dispatch brief names. That file's md1
// rows top out at 81 chars (5 rows) and 73 (1 row); it carries no 85-char
// md1 row, so the exact "85-char md1" figure F-439 and this dispatch brief
// both name is cross-checked separately (md1Chunk85 below) against the
// fork's OWN 85-char md1 fixture (gui/md1_gather_test.go:17,
// wshSortedmultiChunks[0] -- the same fixture scripts/f423-fit-measure's
// predecessor program used, real, not synthesized, confirmed 85 chars by
// direct len()).
const (
	// md1PathLine1 -- backup-strings.txt line 1, 81 chars, the longest md1
	// row the pathological journey's file carries.
	md1PathLine1 = "md1fkl3czs9zjtvyyy5jmpprjjtvyy49ykcgfw2fdsssjjtvyyw2fdssj55jmpp9ef9kvs543f3zlncdz"
	// mk1PathLine7 -- backup-strings.txt line 7, 111 chars, an exact hit on
	// the "mk1-sized (111-char)" target this program measures.
	mk1PathLine7 = "mk1qpd8cwpqqsq4kj90x4eutks2q5zg3vs7rnefw94m5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5z8lphut2hvvpp5av5muuc0cmfrjw2"
	// md1Chunk85 -- gui/md1_gather_test.go:17, wshSortedmultiChunks[0], 85
	// chars. Cross-check fixture for the exact "85-char md1" figure.
	md1Chunk85 = "md1f9k2szspqjtvyyy4qqxppcgsc97v95zqyudm486mm4xav6hqptc0rd7sr9mfc8yrzcx7sju0ra3jh8llnx"
)

// pairGeometry is one string's arithmetic under the real fix.
type pairGeometry struct {
	label        string
	strLen       int
	qrModules    int
	qrSizeMM     float64
	qrLinesN     int
	qrBandMM     float64
	textLinesN   int
	textHeightMM float64
	// pairHeightMM is max(textHeightMM, qrBandMM) -- the paragraph's
	// advance under the real fix (backup.go:373's "advance by
	// max(textLines, qrLines)"), for a TEXT+QR paragraph.
	pairHeightMM float64
}

// measure computes one string's real-fix geometry: the QR module count via
// the fork's OWN qr package (qr.Encode, never assumed), the QR band height
// via qrPlaceAt's own formula (wrap.go:196-212, replicated in device units
// since qrPlaceAt itself is unexported), and the text line count via
// backup.CharsPerLine (real exported call) -- WrapText's own analytic
// estimate, cross-checked below against the ONE real repo citation available
// (backup.go / bundle_flow.go's "measured: paragraph 0's code spans y
// 67840..311040" comment on this exact 85-char fixture).
func measure(label, s string, cpl int) pairGeometry {
	qrc, err := qr.Encode(s, qrLevel)
	if err != nil {
		panic(err)
	}
	fontSizeDev := devF(plateFontSizeUR)
	qrBorderDev := devF(qrBorderMM)
	qrszDev := qrc.Size * params.StrokeWidth * qrScale
	// qrLines: EXACTLY wrap.go:202's formula, "(qrsz + 2*qrBorder + fontSize
	// - 1) / fontSize" -- integer ceiling division, replicated verbatim in
	// device units rather than approximated in mm.
	qrLinesDev := (qrszDev + 2*qrBorderDev + fontSizeDev - 1) / fontSizeDev

	textLines := (len(s) + cpl - 1) / cpl // ceil(len/CharsPerLine)
	textHeightMM := float64(textLines) * plateFontSizeUR
	qrBandMM := float64(qrLinesDev) * plateFontSizeUR
	pairHeightMM := math.Max(textHeightMM, qrBandMM)

	return pairGeometry{
		label:        label,
		strLen:       len(s),
		qrModules:    qrc.Size,
		qrSizeMM:     float64(qrszDev) / mm,
		qrLinesN:     qrLinesDev,
		qrBandMM:     qrBandMM,
		textLinesN:   textLines,
		textHeightMM: textHeightMM,
		pairHeightMM: pairHeightMM,
	}
}

// stackN is the packing arithmetic: N pairs of height h, stacked with the
// SAME 1mm inter-paragraph gap EngraveText already applies between
// paragraphs today (backup.go:505-508, "Space UR sections", params.I(1)),
// against budgetMM. N*h + (N-1)*1mm <= budget  =>  N <= (budget+1)/(h+1).
func stackN(hMM, budgetMM float64) int {
	if hMM <= 0 {
		return 0
	}
	n := int(math.Floor((budgetMM + 1) / (hMM + 1)))
	if n < 0 {
		n = 0
	}
	return n
}

func main() {
	fnt := sh.Font

	cpl := backup.CharsPerLine(params, fnt, plateFontSizeUR)
	lpp := backup.LinesPerPlate(params, plateFontSizeUR)
	fmt.Printf("=== PLATE GEOMETRY (fontMM=%.1f, qrScale=%d, qrLevel=L) ===\n", plateFontSizeUR, qrScale)
	fmt.Printf("CharsPerLine (backup.CharsPerLine, real call) = %d\n", cpl)
	fmt.Printf("LinesPerPlate (backup.LinesPerPlate, real call) = %d\n", lpp)

	// F-435 body budget: the SAME worst-case title+footer mark
	// (bundlePlateFitMark, "WWWWWWWWWWWWWWWWWW") bundlePlateTextFits packs
	// bundle plates against (gui/bundle_flow.go:443, 538-541), replicated
	// here since yBudget/footerRowY are unexported (fit.go:166-202):
	//   start = margin + fontSize            (title reserves row 0)
	//   limit = margin + (LinesPerPlate-1)*fontSize   (footerRowY)
	//   budget = limit - start = (LinesPerPlate-2) * fontMM
	bodyRows := lpp - 2
	bodyBudgetMM := float64(bodyRows) * float64(plateFontSizeUR)
	fmt.Printf("F-435 body budget (title+footer marked, matching bundlePlateTextFits's own trial-fit convention): (LinesPerPlate-2)*fontMM = %d*%.1f = %.1fmm\n",
		bodyRows, plateFontSizeUR, bodyBudgetMM)

	// Footerless raw content height: yBudget's OTHER branch (fit.go:192-202)
	// -- no title, no footer -- start=outerMargin, limit=plateSize-outerMargin.
	// Reported for completeness (the dispatch brief asks for the packing
	// arithmetic against BOTH "the plate's content height and the F-435 body
	// budget"), but it is NOT what decides how many pairs a real packed
	// plate gets: bundleCardPlates packs every plate against the WORST-CASE
	// mark (bundlePlateFitMark, gui/bundle_flow.go:443, "THE PLATE COUNT MAY
	// NOT DEPEND ON THE MARKING... packing against the worst case makes the
	// answer the same for all three readers") regardless of whether that
	// plate ends up visually marked -- so the packer's own convention binds
	// every packed plate to the budget above, not this one.
	rawContentBudgetMM := float64(85 - 2*outerMarginMM) // 85 - 2*3
	fmt.Printf("Plate raw content height (no title/footer, yBudget's footerless branch): plateSize-2*outerMargin = 85-%d = %.1fmm -- NOT the packer's bound; see note above\n",
		2*outerMarginMM, rawContentBudgetMM)
	fmt.Println()

	strs := []struct {
		label, s string
	}{
		{"md1 (backup-strings.txt:1, 81 chars)", md1PathLine1},
		{"md1 (gui/md1_gather_test.go:17, 85 chars, cross-check)", md1Chunk85},
		{"mk1 (backup-strings.txt:7, 111 chars)", mk1PathLine7},
	}

	geoms := make(map[string]pairGeometry)
	fmt.Println("=== PER-STRING ARITHMETIC (real qr.Encode module count; qrLines/textLines by qrPlaceAt/WrapText's own formulas) ===")
	for _, e := range strs {
		g := measure(e.label, e.s, cpl)
		geoms[e.label] = g
		fmt.Printf("%s: len=%d, QR modules=%d (real qr.Encode), QR size=%.2fmm, QR band=%d lines=%.1fmm, text=%d lines=%.1fmm, pair advance under real fix=max(text,qr)=%.1fmm\n",
			g.label, g.strLen, g.qrModules, g.qrSizeMM, g.qrLinesN, g.qrBandMM, g.textLinesN, g.textHeightMM, g.pairHeightMM)
	}
	fmt.Println()

	fmt.Println("=== PACKING ARITHMETIC (ARITHMETIC ONLY for N>=2 -- see refusal confirmation below) ===")
	fmt.Println("N pairs of height h, 1mm inter-paragraph gap: N <= floor((budget+1)/(h+1))")
	fmt.Println("-- against the F-435 body budget (68.4mm), the packer's ACTUAL bound (bundlePlateTextFits packs every plate against this, marked or not):")
	for _, e := range strs {
		g := geoms[e.label]
		nTextQR := stackN(g.pairHeightMM, bodyBudgetMM)
		nQROnly := stackN(g.qrBandMM, bodyBudgetMM)
		fmt.Printf("%s: TEXT+QR pairs/plate = %d (pair height %.1fmm) | QR-ONLY pairs/plate = %d (pair height %.1fmm)\n",
			g.label, nTextQR, g.pairHeightMM, nQROnly, g.qrBandMM)
	}
	fmt.Println("-- against the raw content height (79.0mm, footerless -- informational only, NOT the packer's bound, see note above):")
	for _, e := range strs {
		g := geoms[e.label]
		nTextQR := stackN(g.pairHeightMM, rawContentBudgetMM)
		nQROnly := stackN(g.qrBandMM, rawContentBudgetMM)
		fmt.Printf("%s: TEXT+QR pairs/plate = %d (pair height %.1fmm) | QR-ONLY pairs/plate = %d (pair height %.1fmm)\n",
			g.label, nTextQR, g.pairHeightMM, nQROnly, g.qrBandMM)
	}
	fmt.Println()

	fmt.Println("=== TRIAL FIT (real backup.EngraveText -> fitCheck; the mechanism validateMdmkStrings/validateDescriptor use today) ===")
	trial := func(label string, paras []backup.Paragraph) {
		plate := backup.Text{Paragraphs: paras, Font: fnt}
		plan, err := backup.EngraveText(params, plate)
		if err != nil {
			fmt.Printf("%s: backup.EngraveText refuses -- %v\n", label, err)
			return
		}
		if err := fitCheck(plan); err != nil {
			fmt.Printf("%s: FAILS toPlate's bounds check -- %v\n", label, err)
			return
		}
		fmt.Printf("%s: FITS\n", label)
	}

	for _, e := range strs {
		qrc, err := qr.Encode(e.s, qrLevel)
		if err != nil {
			panic(err)
		}
		trial(e.label+" N=1 TEXT+QR", []backup.Paragraph{{Text: e.s, QR: qrc, QRScale: qrScale}})
		trial(e.label+" N=1 QR-ONLY", []backup.Paragraph{{QR: qrc, QRScale: qrScale}})
		// N=2, one paragraph carrying a QR: confirms F-434's cheap-half
		// refusal is still live at this fork rev, which is WHY the N>=2
		// packing numbers above are arithmetic-only rather than trial-fit --
		// the real API refuses to construct the arrangement at all.
		trial(e.label+" N=2 (one paragraph QR-carrying)", []backup.Paragraph{
			{Text: e.s, QR: qrc, QRScale: qrScale},
			{Text: e.s},
		})
	}
}
