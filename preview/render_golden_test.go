package main

import (
	"crypto/sha256"
	"encoding/hex"
	"flag"
	"strings"
	"testing"
)

// update regenerates the golden constants below. It is registered ONCE at the
// package level (shared by the SVG-hash and PNG-pixel-hash assertions). Regen:
//
//	go test ./preview -run TestRenderGoldens -update
//
// then paste the logged values into the *Golden constants and re-run without
// -update to confirm the round-trip is green.
var update = flag.Bool("update", false, "regenerate render goldens (logs the fresh values)")

// Golden values for MD1_REF's engraveBest render (mode text+qr), pinned over the
// B1 disc-brush output. The PNG golden is the SHA-256 of the DECODED RGBA pixel
// buffer (img.Pix), NOT the compressed PNG bytes: image/png + compress/flate
// output can drift across Go toolchains for identical pixels, but the decoded
// pixels are toolchain-stable and carry the same regression teeth (any
// stroke/coord/canvas change flips the hash). The SVG golden is the SHA-256 of
// the whole SVG string (catches viewBox/stroke-width regressions, not just the
// d path). mCountGolden/cCountGolden pin the exact M/C command counts so a
// pen-up/pen-down swap flips even if a -update masked the d hash.
// blackCountGolden pins the total black-pixel mass of the default render, a
// drift-guard like wantDx/wantDy (a third_party/seedhammer submodule bump that
// moves the geometry legitimately changes it and forces a re-baseline).
const (
	svgGolden        = "e1d0311fd361e6adede4e185e7deff7ca6f35e31e83f0f7db8df0b6d75f9499b"
	pngPixGolden     = "5d7d153b2c7dd00704550eded53bff4ab9ae99dbf8ed0a305bafb8f333db368a"
	mCountGolden     = 2132
	cCountGolden     = 2578
	blackCountGolden = 62702
)

func TestRenderGoldens(t *testing.T) {
	eng, mode, err := engraveBest(MD1_REF)
	if err != nil {
		t.Fatalf("engraveBest: %v", err)
	}
	if mode != "text+qr" {
		t.Fatalf("MD1_REF render mode drifted: got %q want text+qr", mode)
	}

	// --- SVG golden (whole-SVG hash + M/C command counts) ---
	svg := renderSVG(eng)
	svgSum := sha256.Sum256([]byte(svg))
	svgHash := hex.EncodeToString(svgSum[:])
	// Every C/M command is emitted as "C %d"/"M %d", so "C "/"M " (letter+space)
	// count the commands exactly and unambiguously (coordinates are digits/'-'
	// only; no other uppercase C/M appears in the document).
	mCount := strings.Count(svg, "M ")
	cCount := strings.Count(svg, "C ")

	// --- PNG golden (decoded RGBA pixel hash, Route A) ---
	pngBytes, err := renderPNG(eng)
	if err != nil {
		t.Fatalf("renderPNG: %v", err)
	}
	img := decodeRGBA(t, pngBytes)
	pixSum := sha256.Sum256(img.Pix)
	pixHash := hex.EncodeToString(pixSum[:])
	blackCount := countBlack(img)

	if *update {
		t.Logf("regenerated goldens — paste into the const block:")
		t.Logf("  svgGolden        = %q", svgHash)
		t.Logf("  pngPixGolden     = %q", pixHash)
		t.Logf("  mCountGolden     = %d", mCount)
		t.Logf("  cCountGolden     = %d", cCount)
		t.Logf("  blackCountGolden = %d", blackCount)
		return
	}

	if svgHash != svgGolden {
		t.Errorf("SVG hash drift: got %s want %s (run with -update if intentional)", svgHash, svgGolden)
	}
	if mCount != mCountGolden {
		t.Errorf("SVG M-command count drift: got %d want %d", mCount, mCountGolden)
	}
	if cCount != cCountGolden {
		t.Errorf("SVG C-command count drift: got %d want %d", cCount, cCountGolden)
	}
	if pixHash != pngPixGolden {
		t.Errorf("PNG decoded-pixel hash drift: got %s want %s (run with -update if intentional)", pixHash, pngPixGolden)
	}
	if blackCount != blackCountGolden {
		t.Errorf("PNG black-pixel mass drift: got %d want %d", blackCount, blackCountGolden)
	}
}
