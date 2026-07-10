package main

import (
	"bytes"
	"image"
	"image/color"
	"image/png"
	"testing"

	"seedhammer.com/bezier"
	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
)

// B1 (F13): discRadius maps a machine-unit->px scale to the integer disc radius
// that reproduces the SVG's physical stroke width. radius == max(1,
// round(strokeWidth*scale/2)): strokeWidth is the FULL stroke width, so the
// radius is half of strokeWidth*scale; the max(1,...) floor guarantees strokes
// never vanish on a heavily downscaled render.
func TestDiscRadiusMapping(t *testing.T) {
	cases := []struct {
		name  string
		scale float64
		want  int
	}{
		// strokeWidth*scale/2 == 3.0 -> 3
		{"exact-3", 6.0 / strokeWidth, 3},
		// strokeWidth*scale/2 == 5.0 -> 5
		{"exact-5", 10.0 / strokeWidth, 5},
		// Default MD1_REF render: scale = pngMaxPx/wantDx -> 2.226 -> round 2.
		{"md1-default", float64(pngMaxPx) / float64(wantDx), 2},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if got := discRadius(tc.scale); got != tc.want {
				t.Fatalf("discRadius(%v) = %d, want %d", tc.scale, got, tc.want)
			}
		})
	}
}

// B1 (F13): the 1px floor — at a tiny scale where strokeWidth*scale/2 < 0.5,
// discRadius must return 1, never 0 (a radius-0 disc paints nothing).
func TestDiscRadiusFloor(t *testing.T) {
	cases := []struct {
		name  string
		scale float64
	}{
		{"quarter", 0.5 / strokeWidth}, // strokeWidth*scale/2 == 0.25 -> round 0
		{"zero", 0.0},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if got := discRadius(tc.scale); got != 1 {
				t.Fatalf("discRadius(%v) = %d, want 1 (floor)", tc.scale, got)
			}
		})
	}
}

// --- test-only baseline: the pre-B1 1px hairline stroke, kept reachable per
// spec §B1 / R0 L2 so the pixel-mass test can compare the disc-brush black mass
// against a live hairline baseline. Coexists with the disc-brush drawLine in
// render_png.go; it is NOT production code. ---

// strokeHairline is a verbatim copy of the pre-B1 drawLine: a 1px Bresenham
// line (SetRGBA per step, bounds-checked), used only to compute the hairline
// baseline pixel-mass.
func strokeHairline(img *image.RGBA, x0, y0, x1, y1 int, c color.RGBA) {
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

// renderHairline mirrors renderPNG's walk EXACTLY but strokes the polyline with
// strokeHairline (1px) on the ungrown base canvas, to produce the hairline
// baseline black-pixel count. White margin does not affect the black count, so
// the ungrown canvas is equivalent for mass comparison.
func renderHairline(eng engrave.Engraving) *image.RGBA {
	bounds := bspline.Measure(engrave.PlanEngraving(params.StepperConfig, eng)).Bounds
	dx, dy := bounds.Dx(), bounds.Dy()
	if dx <= 0 {
		dx = 1
	}
	if dy <= 0 {
		dy = 1
	}
	scale := float64(pngMaxPx) / float64(max(dx, dy))
	if scale > 1 {
		scale = 1
	}
	w := int(float64(dx)*scale) + 1
	h := int(float64(dy)*scale) + 1
	img := image.NewRGBA(image.Rect(0, 0, w, h))
	for i := range img.Pix {
		img.Pix[i] = 0xff
	}
	black := color.RGBA{R: 0, G: 0, B: 0, A: 0xff}
	toPx := func(p bezier.Point) (int, int) {
		return int(float64(p.X-bounds.Min.X) * scale), int(float64(p.Y-bounds.Min.Y) * scale)
	}
	spacing := int(1.0 / scale)
	if spacing < 1 {
		spacing = 1
	}
	var seg bspline.Segment
	for k := range engrave.PlanEngraving(params.StepperConfig, eng) {
		c, dt, line := seg.Knot(k)
		if dt == 0 || !line {
			continue
		}
		pts := []bezier.Point{c.C0}
		pts = bezier.Sample(pts, c, spacing)
		for i := 1; i < len(pts); i++ {
			x0, y0 := toPx(pts[i-1])
			x1, y1 := toPx(pts[i])
			strokeHairline(img, x0, y0, x1, y1, black)
		}
	}
	return img
}

// countBlack returns the number of fully-black (R==G==B==0) pixels in img.
func countBlack(img *image.RGBA) int {
	n := 0
	for i := 0; i+3 < len(img.Pix); i += 4 {
		if img.Pix[i] == 0 && img.Pix[i+1] == 0 && img.Pix[i+2] == 0 {
			n++
		}
	}
	return n
}

// decodeRGBA decodes PNG bytes and asserts the result is *image.RGBA (opaque
// truecolor PNGs decode to *image.RGBA under the Go stdlib).
func decodeRGBA(t *testing.T, b []byte) *image.RGBA {
	t.Helper()
	im, err := png.Decode(bytes.NewReader(b))
	if err != nil {
		t.Fatalf("png.Decode: %v", err)
	}
	rgba, ok := im.(*image.RGBA)
	if !ok {
		t.Fatalf("decoded PNG is %T, want *image.RGBA", im)
	}
	return rgba
}

// B1 (F13): pixel-mass regression — the disc-brush stroke must carry >=2x the
// black mass of the 1px hairline baseline over the same MD1_REF centerlines
// (measured ratio ~5x; >=2x is a robust non-flaky floor). RED before B1 (the
// current output IS the hairline, ratio ~1.0); GREEN after the disc-brush lands.
func TestPNGStrokePixelMass(t *testing.T) {
	eng, _, err := engraveBest(MD1_REF)
	if err != nil {
		t.Fatalf("engraveBest: %v", err)
	}
	b, err := renderPNG(eng)
	if err != nil {
		t.Fatalf("renderPNG: %v", err)
	}
	discCount := countBlack(decodeRGBA(t, b))

	hairCount := countBlack(renderHairline(eng))
	if hairCount == 0 {
		t.Fatal("hairline baseline has zero black pixels")
	}
	ratio := float64(discCount) / float64(hairCount)
	if discCount < 2*hairCount {
		t.Fatalf("pixel-mass too low: disc=%d hairline=%d ratio=%.2f, want >=2x", discCount, hairCount, ratio)
	}
	t.Logf("pixel-mass: disc=%d hairline=%d ratio=%.2fx", discCount, hairCount, ratio)
}
