package main

import (
	"bytes"
	"image/png"
	"strings"
	"testing"
)

func TestRenderSVGContainsExpectedStructure(t *testing.T) {
	eng, _, err := engraveBest(MD1_REF)
	if err != nil {
		t.Fatalf("engraveBest: %v", err)
	}
	svg := renderSVG(eng)
	if !strings.Contains(svg, "<svg") {
		t.Errorf("SVG missing <svg root:\n%s", svg)
	}
	if !strings.Contains(svg, "viewBox=") {
		t.Errorf("SVG missing viewBox:\n%s", svg)
	}
	if !strings.Contains(svg, "<path") {
		t.Errorf("SVG missing <path:\n%s", svg)
	}
	// Exactly one accumulated <path> (mirrors seedhammer's own single-path
	// renderer), not one <path> per cubic.
	if n := strings.Count(svg, "<path"); n != 1 {
		t.Errorf("expected exactly one <path>, got %d:\n%s", n, svg)
	}
	// The accumulated d string must contain at least one cubic command.
	if !strings.Contains(svg, " C ") {
		t.Errorf("SVG path has no cubic commands:\n%s", svg)
	}
}

func TestRenderPNGValidHeader(t *testing.T) {
	eng, _, err := engraveBest(MD1_REF)
	if err != nil {
		t.Fatalf("engraveBest: %v", err)
	}
	b, err := renderPNG(eng)
	if err != nil {
		t.Fatalf("renderPNG: %v", err)
	}
	// PNG magic header.
	if !bytes.HasPrefix(b, []byte("\x89PNG\r\n\x1a\n")) {
		t.Fatalf("renderPNG did not produce a PNG header, got % x", b[:min(8, len(b))])
	}
	// The bytes must decode as a real PNG image with a non-empty canvas.
	img, err := png.Decode(bytes.NewReader(b))
	if err != nil {
		t.Fatalf("png.Decode: %v", err)
	}
	if r := img.Bounds(); r.Dx() <= 0 || r.Dy() <= 0 {
		t.Fatalf("decoded PNG has empty bounds: %v", r)
	}
}
