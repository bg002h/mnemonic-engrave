package main

import (
	"bytes"
	"image/png"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestVersionFlag(t *testing.T) {
	version = "9.9.9-test"
	t.Cleanup(func() { version = "" })

	var out, errBuf bytes.Buffer
	code := run([]string{"--version"}, strings.NewReader(""), &out, &errBuf)
	if code != 0 {
		t.Fatalf("--version exit = %d, want 0 (stderr: %s)", code, errBuf.String())
	}
	got := strings.TrimSpace(out.String())
	if got != "me-preview 9.9.9-test" {
		t.Fatalf("--version printed %q, want %q", got, "me-preview 9.9.9-test")
	}
}

func TestRenderSVGToFile(t *testing.T) {
	dir := t.TempDir()
	outPath := filepath.Join(dir, "plate.svg")

	var out, errBuf bytes.Buffer
	code := run(
		[]string{"render", "--format", "svg", "--out", outPath},
		strings.NewReader(MD1_REF),
		&out, &errBuf,
	)
	if code != 0 {
		t.Fatalf("render exit = %d, want 0 (stderr: %s)", code, errBuf.String())
	}
	// stdout reports the chosen mode.
	stdout := strings.TrimSpace(out.String())
	if !strings.HasPrefix(stdout, "mode ") {
		t.Fatalf("stdout = %q, want it to start with %q", stdout, "mode ")
	}
	b, err := os.ReadFile(outPath)
	if err != nil {
		t.Fatalf("read out file: %v", err)
	}
	if !strings.Contains(string(b), "<svg") {
		t.Fatalf("output file is not an SVG:\n%s", string(b))
	}
}

func TestRenderSVGToStdout(t *testing.T) {
	var out, errBuf bytes.Buffer
	code := run(
		[]string{"render", "--format", "svg", "--out", "-"},
		strings.NewReader(MD1_REF),
		&out, &errBuf,
	)
	if code != 0 {
		t.Fatalf("render exit = %d, want 0 (stderr: %s)", code, errBuf.String())
	}
	if !strings.Contains(out.String(), "<svg") {
		t.Fatalf("stdout is not an SVG:\n%s", out.String())
	}
}

func TestRenderPNGToFile(t *testing.T) {
	dir := t.TempDir()
	outPath := filepath.Join(dir, "plate.png")

	var out, errBuf bytes.Buffer
	code := run(
		[]string{"render", "--format", "png", "--out", outPath},
		strings.NewReader(MD1_REF),
		&out, &errBuf,
	)
	if code != 0 {
		t.Fatalf("render exit = %d, want 0 (stderr: %s)", code, errBuf.String())
	}
	b, err := os.ReadFile(outPath)
	if err != nil {
		t.Fatalf("read out file: %v", err)
	}
	if _, err := png.Decode(bytes.NewReader(b)); err != nil {
		t.Fatalf("output file is not a PNG: %v", err)
	}
}

func TestRenderModeForced(t *testing.T) {
	var out, errBuf bytes.Buffer
	code := run(
		[]string{"render", "--mode", "text", "--format", "svg", "--out", "-"},
		strings.NewReader(MD1_REF),
		&out, &errBuf,
	)
	if code != 0 {
		t.Fatalf("render --mode text exit = %d, want 0 (stderr: %s)", code, errBuf.String())
	}
	// With --out -, the SVG payload streams to stdout and the mode line goes
	// to stderr (so stdout stays a clean, pipeable SVG). The forced mode is
	// reported as "mode text".
	if !strings.Contains(out.String(), "<svg") {
		t.Fatalf("expected SVG on stdout, got:\n%s", out.String())
	}
	if !strings.Contains(errBuf.String(), "mode text") {
		t.Fatalf("expected 'mode text' on stderr, got:\n%s", errBuf.String())
	}
}

func TestRenderOversizeExitsNonZero(t *testing.T) {
	huge := strings.Repeat(MD1_REF, 200)
	var out, errBuf bytes.Buffer
	code := run(
		[]string{"render", "--format", "svg", "--out", "-"},
		strings.NewReader(huge),
		&out, &errBuf,
	)
	if code == 0 {
		t.Fatalf("oversize render exit = 0, want non-zero")
	}
	if errBuf.Len() == 0 {
		t.Fatalf("oversize render wrote nothing to stderr")
	}
}

func TestRenderUnknownModeExitsNonZero(t *testing.T) {
	var out, errBuf bytes.Buffer
	code := run(
		[]string{"render", "--mode", "bogus", "--format", "svg", "--out", "-"},
		strings.NewReader(MD1_REF),
		&out, &errBuf,
	)
	if code == 0 {
		t.Fatalf("unknown mode exit = 0, want non-zero")
	}
}

func TestRenderBadFormatExitsNonZero(t *testing.T) {
	var out, errBuf bytes.Buffer
	code := run(
		[]string{"render", "--format", "gif", "--out", "-"},
		strings.NewReader(MD1_REF),
		&out, &errBuf,
	)
	if code == 0 {
		t.Fatalf("bad format exit = 0, want non-zero")
	}
}
