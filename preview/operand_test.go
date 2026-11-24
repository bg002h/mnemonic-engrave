package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

const bhMeI18MD1 = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3\n"

const bhMeI18Usage = "usage: me-preview [--version] | render --format <svg|png> --out <FILE|-> [--mode <text+qr|text|qr>]\n"

// readTracker records whether stdin was read.
type readTracker struct {
	r    *strings.Reader
	read bool
}

func (t *readTracker) Read(p []byte) (int, error) {
	t.read = true
	return t.r.Read(p)
}

func bhMeI18Refusal(t *testing.T, build func(out string) []string) {
	t.Helper()
	out := filepath.Join(t.TempDir(), "plate.svg")
	in := &readTracker{r: strings.NewReader(bhMeI18MD1)}
	var so, se bytes.Buffer
	code := run(build(out), in, &so, &se)
	if code != 1 {
		t.Fatalf("exit %d, want 1; stderr %q", code, se.String())
	}
	want := "me-preview: unexpected operand \"second-card\"\n" + bhMeI18Usage
	if se.String() != want {
		t.Fatalf("stderr %q, want %q", se.String(), want)
	}
	if in.read {
		t.Fatalf("stdin was read before the operand was refused")
	}
	if _, err := os.Stat(out); err == nil {
		t.Fatalf("%s exists after a refusal", out)
	}
	if so.Len() != 0 {
		t.Fatalf("stdout not empty: %q", so.String())
	}
}

func TestBhMeI18StrayOperandRefused(t *testing.T) {
	bhMeI18Refusal(t, func(out string) []string {
		return []string{"render", "--format", "svg", "--out", out, "second-card"}
	})
}

func TestBhMeI18StrayOperandBeforeModeRefused(t *testing.T) {
	bhMeI18Refusal(t, func(out string) []string {
		return []string{"render", "--format", "svg", "--out", out, "second-card", "--mode", "qr"}
	})
}

func TestBhMeI18ModeQrControl(t *testing.T) {
	out := filepath.Join(t.TempDir(), "plate.svg")
	var so, se bytes.Buffer
	code := run([]string{"render", "--format", "svg", "--out", out, "--mode", "qr"}, strings.NewReader(bhMeI18MD1), &so, &se)
	if code != 0 {
		t.Fatalf("exit %d; stderr %q", code, se.String())
	}
	if strings.TrimSpace(so.String()) != "mode qr" {
		t.Fatalf("stdout %q, want \"mode qr\"", so.String())
	}
	fi, err := os.Stat(out)
	if err != nil || fi.Size() == 0 {
		t.Fatalf("output missing or empty: %v", err)
	}
}
