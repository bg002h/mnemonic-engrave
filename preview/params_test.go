package main

import (
	"testing"

	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
)

// MD1_REF is a fixed reference string; its engraved geometry bbox is a stable
// proxy for "params unchanged". Use the Phase A vector.
const MD1_REF = "md1yqpqqxqq8xtwhw4xwn4qh"

// Golden bbox (machine units) for MD1_REF under the replicated SH2 params,
// captured from the first green run against seedhammer v1.4.2 (713aee2).
// These are a geometry proxy for the replicated engrave.Params: a param drift
// (StrokeWidth / Millimeter / StepperConfig) re-weights the B-spline timing
// and changes these bounds, so a mismatch flags stale replicated params.
const (
	wantDx = 431224 // ~67.4 mm at 6400 units/mm
	wantDy = 200868 // ~31.4 mm at 6400 units/mm
)

func TestParamsGeometryGolden(t *testing.T) {
	eng, mode, err := engraveBest(MD1_REF) // layout.go (Task 3)
	if err != nil {
		t.Fatalf("engrave: %v", err)
	}
	_ = mode
	b := bspline.Measure(engrave.PlanEngraving(params.StepperConfig, eng)).Bounds
	if b.Dx() != wantDx || b.Dy() != wantDy {
		t.Fatalf("geometry drift: got %dx%d want %dx%d (replicated params may be stale)", b.Dx(), b.Dy(), wantDx, wantDy)
	}
}
