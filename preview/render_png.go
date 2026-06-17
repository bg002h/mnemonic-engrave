package main

import (
	"bytes"
	"image"
	"image/color"
	"image/png"

	"seedhammer.com/bezier"
	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
)

// pngMaxPx is the target size (px) for the longest side of the rasterized
// preview; machine-unit bounds are scaled down to fit.
const pngMaxPx = 1000

// renderPNG rasterizes the SAME pen-down cubics as renderSVG onto an RGBA
// canvas: each pen-down cubic is sampled with bezier.Sample into a polyline,
// and consecutive sample points are joined with 1px black line segments.
// Pen-down (line) segments only; pen-up jumps and zero-duration knots are
// skipped (matching the SVG walk). The machine-unit bounds are scaled down by
// an integer-ish factor so the longest side is about pngMaxPx.
func renderPNG(eng engrave.Engraving) ([]byte, error) {
	bounds := bspline.Measure(engrave.PlanEngraving(params.StepperConfig, eng)).Bounds

	dx, dy := bounds.Dx(), bounds.Dy()
	// Guard against degenerate (empty) bounds.
	if dx <= 0 {
		dx = 1
	}
	if dy <= 0 {
		dy = 1
	}

	// scale converts machine units -> pixels. Pick it so the longest side is
	// about pngMaxPx; never upscale beyond 1:1.
	scale := float64(pngMaxPx) / float64(max(dx, dy))
	if scale > 1 {
		scale = 1
	}

	w := int(float64(dx)*scale) + 1
	h := int(float64(dy)*scale) + 1
	img := image.NewRGBA(image.Rect(0, 0, w, h))
	// White background.
	for i := range img.Pix {
		img.Pix[i] = 0xff
	}

	black := color.RGBA{R: 0, G: 0, B: 0, A: 0xff}

	// toPx maps a machine-unit point into pixel space (origin at bounds.Min).
	toPx := func(p bezier.Point) (int, int) {
		px := int(float64(p.X-bounds.Min.X) * scale)
		py := int(float64(p.Y-bounds.Min.Y) * scale)
		return px, py
	}

	// spacing controls sample density in machine units. Aim for ~1px chords.
	spacing := int(1.0 / scale)
	if spacing < 1 {
		spacing = 1
	}

	var seg bspline.Segment
	for k := range engrave.PlanEngraving(params.StepperConfig, eng) {
		c, dt, line := seg.Knot(k)
		if dt == 0 {
			continue
		}
		if !line {
			continue // pen-up: no stroke
		}
		// Sample the cubic into a polyline, seeded with its own start point.
		pts := []bezier.Point{c.C0}
		pts = bezier.Sample(pts, c, spacing)
		for i := 1; i < len(pts); i++ {
			x0, y0 := toPx(pts[i-1])
			x1, y1 := toPx(pts[i])
			drawLine(img, x0, y0, x1, y1, black)
		}
	}

	var buf bytes.Buffer
	if err := png.Encode(&buf, img); err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

// drawLine draws a 1px line between (x0,y0) and (x1,y1) using Bresenham's
// algorithm, clipped to the image bounds.
func drawLine(img *image.RGBA, x0, y0, x1, y1 int, c color.RGBA) {
	dx := abs(x1 - x0)
	dy := -abs(y1 - y0)
	sx := 1
	if x0 >= x1 {
		sx = -1
	}
	sy := 1
	if y0 >= y1 {
		sy = -1
	}
	err := dx + dy
	b := img.Bounds()
	for {
		if x0 >= b.Min.X && x0 < b.Max.X && y0 >= b.Min.Y && y0 < b.Max.Y {
			img.SetRGBA(x0, y0, c)
		}
		if x0 == x1 && y0 == y1 {
			break
		}
		e2 := 2 * err
		if e2 >= dy {
			err += dy
			x0 += sx
		}
		if e2 <= dx {
			err += dx
			y0 += sy
		}
	}
}

func abs(v int) int {
	if v < 0 {
		return -v
	}
	return v
}
