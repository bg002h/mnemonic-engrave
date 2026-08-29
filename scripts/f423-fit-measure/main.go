// Command f423-fit-measure answers F-423's question (S2 plan P4.1): does more
// than one md1 string fit on one SeedHammer plate side, at the SHIPPED font
// (no FontSize reduction -- the S2 plan's standing ruling)?
//
// It measures two ways, as the plan directs:
//
//  1. ANALYTIC UPPER BOUND: backup.CharsPerLine / backup.LinesPerPlate, the
//     fork's own arithmetic (backup/backup.go:88-97), at plateSize=85mm
//     (backup/backup.go:77) and the shipped font metrics (backup.Text.fontMM,
//     backup/backup.go:58-63).
//  2. TRIAL FIT: drives the REAL mechanism validateDescriptor itself uses
//     (gui/gui.go:722-736) -- backup.EngraveText(params, plate), then the
//     same fits-or-errors bounds check toPlate applies (gui/gui.go:3515-3528).
//
// toPlate itself is unexported (package gui) and unreachable from an external
// module. Every primitive it is built from IS exported --
// gui.SquarePlate/.Dims, gui.ErrTooLarge, engrave.PlanEngraving,
// bspline.Measure, bspline.Bounds, bezier.Pt/.Sub -- so fitCheck below is a
// minimal shim reproducing toPlate's five lines over that public surface,
// not a fork of private logic. The one non-exported value it needs,
// safetyMargin=3 (mm, gui/gui.go:52), is inlined with its citation; it is a
// bare numeral, not logic.
package main

import (
	"fmt"

	"seedhammer.com/backup"
	"seedhammer.com/bezier"
	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
	"seedhammer.com/font/sh"
	"seedhammer.com/gui"
)

// Production geometry, mirrored from cmd/controller/platform_sh2.go (mm=6400,
// strokeWidth=0.3mm) via the SAME values gui_test.go/backup_test.go use for
// their own golden tests (mm=6400) -- confirmed by hand: fullStepsPerRevolution
// (200) / mmPerRevolution (8) * tmc2209.Microsteps = mm, and 200/8*256=6400.
// The StepperConfig speeds below are irrelevant to FIT (they affect only
// Duration, not Bounds) but cannot be left zero -- PlanEngraving divides by
// them (see the StepperConfig doc comment below), so they are set to
// production's own values anyway.
const (
	mm             = 6400
	strokeWidth    = 0.3 * mm
	topSpeed       = 30 * mm
	engravingSpeed = 4 * mm
	acceleration   = 250 * mm
	jerk           = 2600 * mm
)

// StepperConfig mirrors cmd/controller/platform_sh2.go's production
// engraverConf exactly (platform_sh2.go:227-233). It has no effect on the
// Bounds fitCheck reads (only on Duration/timing), but PlanEngraving divides
// by these fields internally (engrave.go:1139's timeScaler.Scale), so a zero
// StepperConfig panics rather than measuring anything -- confirmed by trying
// it first.
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

// plateFontSizeUR is backup.plateFontSizeUR (backup/backup.go:176, unexported),
// the value backup.Text.fontMM() resolves to whenever FontSize is left zero --
// which is what EVERY descriptor/mdmk caller does (gui/gui.go:459,
// gui/gui.go:2563-2569 construct backup.Text with no FontSize field set). It
// is quoted here as a literal, not reimplemented logic: the S2 plan's ruling
// is that this floor may not be lowered, so this is the ONE font size this
// program ever measures at.
const plateFontSizeUR = 3.8

// safetyMargin mirrors gui.go:52's unexported const of the same name and
// value -- toPlate's own safety inset in millimeters, applied on all four
// sides before the bounds check.
const safetyMargin = 3

// fitCheck reproduces gui.toPlate's fit-or-error bounds check
// (gui/gui.go:3515-3528) using only toPlate's own exported dependencies.
// toPlate itself cannot be called from outside package gui.
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

// Real md1 chunk strings, grepped from the fork's own test fixtures (per the
// plan's instruction), not synthesized:
//   - singleStringBoundary: md/testdata/vectors/single_string_boundary.phrase.txt
//     (single-line, non-chunked -- its name says it sits at the boundary of
//     what fits ONE md1 string before chunking is required).
//   - chunkA, chunkB: gui/md1_gather_test.go:17-18, two of the five distinct
//     chunks of wshSortedmultiChunks (a real chunked md1 set; each chunk is
//     independently a complete, valid md1 string of the kind bundlePlate.str
//     carries verbatim).
const (
	singleStringBoundary = "md1ygpqqxppc2qpydzk0qxvsqqypqxpqsyqcyq5srqszsvvzq2ps8gpgxquy9qcrssztqwzqfpfcgpy9qk3s85sn9eadh98"
	chunkA               = "md1f9k2szspqjtvyyy4qqxppcgsc97v95zqyudm486mm4xav6hqptc0rd7sr9mfc8yrzcx7sju0ra3jh8llnx"
	chunkB               = "md1f9k2szsguxj4ln63stmuuq6kgrvtxn9uedgysqk5mrsqw5njj30rf8ejcf6w954djz5pse9uf467htrhv9"
)

func main() {
	fnt := sh.Font

	cpl := backup.CharsPerLine(params, fnt, plateFontSizeUR)
	lpp := backup.LinesPerPlate(params, plateFontSizeUR)
	fmt.Printf("=== ANALYTIC UPPER BOUND (backup.CharsPerLine / backup.LinesPerPlate, fontMM=%.1f) ===\n", plateFontSizeUR)
	fmt.Printf("CharsPerLine = %d\n", cpl)
	fmt.Printf("LinesPerPlate = %d\n", lpp)
	fmt.Printf("plate char capacity (CharsPerLine * LinesPerPlate) = %d\n", cpl*lpp)
	fmt.Println()

	strs := []string{singleStringBoundary, chunkA, chunkB}
	names := []string{"singleStringBoundary", "chunkA", "chunkB"}

	fmt.Println("=== TEST STRINGS ===")
	for i, s := range strs {
		lines := (len(s) + cpl - 1) / cpl // ceil(len/CharsPerLine), analytic estimate
		fmt.Printf("%s: len=%d chars, ceil(len/CharsPerLine)=%d lines (analytic estimate)\n", names[i], len(s), lines)
	}
	fmt.Println()

	fmt.Println("=== TRIAL FIT (backup.EngraveText -> fitCheck, shipped font, FontSize=0) ===")
	for n := 1; n <= 3; n++ {
		var paras []backup.Paragraph
		for i := 0; i < n; i++ {
			paras = append(paras, backup.Paragraph{Text: strs[i]})
		}
		plate := backup.Text{
			Paragraphs: paras,
			Font:       fnt,
		}
		plan := backup.EngraveText(params, plate)
		err := fitCheck(plan)
		names := names[:n]
		if err != nil {
			fmt.Printf("N=%d strings (%v): FAILS toPlate's bounds check -- %v\n", n, names, err)
		} else {
			fmt.Printf("N=%d strings (%v): FITS\n", n, names)
		}
	}
}
