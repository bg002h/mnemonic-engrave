package main

import (
	"bytes"
	"image"
	"image/color"
	"image/png"
	"math"

	"seedhammer.com/bezier"
	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
)

// pngMaxPx is the target size (px) for the longest side of the rasterized
// preview; machine-unit bounds are scaled down to fit.
const pngMaxPx = 1000

// discRadius maps a machine-unit->px scale to the integer radius of the disc
// brush stamped along each stroke, so the PNG stroke reproduces the SVG's
// physical stroke width. strokeWidth is the FULL stroke width, so the radius is
// half of strokeWidth*scale; the max(1,...) floor guarantees a stroke never
// vanishes on a heavily downscaled render.
func discRadius(scale float64) int {
	r := int(math.Round(strokeWidth * scale / 2))
	if r < 1 {
		return 1
	}
	return r
}

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

	// The disc brush extends `radius` px beyond the centerline in every
	// direction, so grow the canvas by +2*radius per axis and shift the toPx
	// origin by +radius; a stroke at the bounds edge is then never clipped and
	// no write is ever negative-indexed (stampDisc is bounds-checked too).
	radius := discRadius(scale)
	w := int(float64(dx)*scale) + 1 + 2*radius
	h := int(float64(dy)*scale) + 1 + 2*radius
	img := image.NewRGBA(image.Rect(0, 0, w, h))
	// White background.
	for i := range img.Pix {
		img.Pix[i] = 0xff
	}

	black := color.RGBA{R: 0, G: 0, B: 0, A: 0xff}

	// toPx maps a machine-unit point into pixel space (origin at bounds.Min,
	// shifted by +radius so the disc margin fits inside the grown canvas).
	toPx := func(p bezier.Point) (int, int) {
		px := int(float64(p.X-bounds.Min.X)*scale) + radius
		py := int(float64(p.Y-bounds.Min.Y)*scale) + radius
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
			drawLine(img, x0, y0, x1, y1, radius, black)
		}
	}

	var buf bytes.Buffer
	if err := png.Encode(&buf, img); err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

// drawLine strokes the line between (x0,y0) and (x1,y1) by stamping a disc of
// the given radius at EACH Bresenham step (not per polyline sample point). This
// yields round caps AND round joins for free, matching the SVG's
// stroke-linecap/linejoin: round, and is gap-free by construction regardless of
// the sample spacing. Fully integer, no anti-aliasing (determinism is
// load-bearing: B2 pins the decoded-pixel hash).
func drawLine(img *image.RGBA, x0, y0, x1, y1, radius int, c color.RGBA) {
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
	for {
		stampDisc(img, x0, y0, radius, c)
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

// stampDisc paints a filled disc of the given radius centered at (cx,cy). Every
// pixel is written via the bounds-checked SetRGBA (never raw img.Pix indexing),
// so a disc straddling the canvas edge cannot panic. The disc is a plain
// integer set (dx*dx+dy*dy <= radius*radius) — no anti-aliasing.
func stampDisc(img *image.RGBA, cx, cy, radius int, c color.RGBA) {
	b := img.Bounds()
	r2 := radius * radius
	for dy := -radius; dy <= radius; dy++ {
		for dx := -radius; dx <= radius; dx++ {
			if dx*dx+dy*dy > r2 {
				continue
			}
			x, y := cx+dx, cy+dy
			if x >= b.Min.X && x < b.Max.X && y >= b.Min.Y && y < b.Max.Y {
				img.SetRGBA(x, y, c)
			}
		}
	}
}

func abs(v int) int {
	if v < 0 {
		return -v
	}
	return v
}
