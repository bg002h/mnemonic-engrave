package main

import (
	"fmt"
	"strings"

	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
)

// renderSVG mirrors seedhammer's OWN SVG renderer (internal/golden/golden.go
// Vectorize, ~lines 175-194) (resolves plan-R0 C-1): a SINGLE <path>
// accumulating commands —
//
//	pen-UP segment  (!line) -> "M C3.x C3.y"   (reposition cursor to next run start)
//	pen-DOWN segment (line) -> "C C1 C2 C3"     (C0 is the IMPLICIT cursor = prior C3;
//	                                             NO M, NO C0 — preserves B-spline G1 continuity)
//	skip dt==0 (zero-duration) segments (NOT just pen-up).
//
// Emitting "M C0 C ..." per cubic (a naive reading) re-specifies C0 and breaks
// continuity; that was the rejected-and-fixed bug.
func renderSVG(eng engrave.Engraving) string {
	bounds := bspline.Measure(engrave.PlanEngraving(params.StepperConfig, eng)).Bounds

	var d strings.Builder
	var seg bspline.Segment
	for k := range engrave.PlanEngraving(params.StepperConfig, eng) {
		c, dt, line := seg.Knot(k)
		if dt == 0 {
			continue // zero-duration (incl. window-priming) — skip
		}
		if line {
			fmt.Fprintf(&d, " C %d %d, %d %d, %d %d", c.C1.X, c.C1.Y, c.C2.X, c.C2.Y, c.C3.X, c.C3.Y)
		} else {
			fmt.Fprintf(&d, " M %d %d", c.C3.X, c.C3.Y) // pen-up: cursor jump
		}
	}

	var b strings.Builder
	fmt.Fprintf(&b, `<svg xmlns="http://www.w3.org/2000/svg" viewBox="%d %d %d %d">`+"\n",
		bounds.Min.X, bounds.Min.Y, bounds.Dx(), bounds.Dy())
	fmt.Fprintf(&b, `<path fill="none" stroke="black" stroke-width="%d" stroke-linecap="round" stroke-linejoin="round" d="%s"/>`+"\n",
		strokeWidth, strings.TrimSpace(d.String()))
	b.WriteString("</svg>\n")
	return b.String()
}
