package main

import "seedhammer.com/engrave"

// Replicated VERBATIM from seedhammer v1.4.2 cmd/controller/platform_sh2.go
// (//go:build tinygo && rp — not host-importable) and cross-checked against
// the host-compilable gui/gui_test.go. Re-verify on any third_party/seedhammer bump.
const mm = 6400          // Millimeter in machine units (200/8 * 256 microsteps)
const strokeWidth = 1920 // 0.3 * mm

// NB: TicksPerSecond == Speed == topSpeed (30*mm) is a real SH2 hardware
// equality on the SH2, not a coincidence.
var params = engrave.Params{
	StrokeWidth: strokeWidth,
	Millimeter:  mm,
	StepperConfig: engrave.StepperConfig{
		Speed:          30 * mm,   // topSpeed = 192000
		EngravingSpeed: 8 * mm,    // 51200
		Acceleration:   250 * mm,  // 1600000
		Jerk:           2600 * mm, // 16640000
		TicksPerSecond: 30 * mm,   // == topSpeed
	},
}

// Plate geometry (replicated from gui toPlate): 85x85 mm, 3 mm safety margin.
const plateSizeMM = 85
const safetyMarginMM = 3
