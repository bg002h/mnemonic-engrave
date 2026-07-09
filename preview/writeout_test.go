package main

import (
	"io"
	"os"
	"path/filepath"
	"testing"
)

// F10 (R0 L2): a file written by writeOut for a real path must be owner-only —
// no group/other permission bits. This exercises the Go writeOut change directly
// (the Rust shell-fake e2e cannot, since it is not Go writeOut).
func TestWriteOutPermIsOwnerOnly(t *testing.T) {
	dir := t.TempDir()
	outPath := filepath.Join(dir, "plate.svg")
	if err := writeOut(outPath, []byte("<svg/>"), io.Discard); err != nil {
		t.Fatalf("writeOut: %v", err)
	}
	info, err := os.Stat(outPath)
	if err != nil {
		t.Fatalf("stat: %v", err)
	}
	if perm := info.Mode().Perm(); perm&0o077 != 0 {
		t.Fatalf("writeOut created a group/other-accessible file: mode %#o, want no bits in 0o077", perm)
	}
}
