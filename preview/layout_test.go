package main

import (
	"strings"
	"testing"
)

func TestEngraveBestReturnsModeAndEngraving(t *testing.T) {
	eng, mode, err := engraveBest(MD1_REF)
	if err != nil {
		t.Fatalf("engraveBest: %v", err)
	}
	if eng == nil {
		t.Fatal("engraveBest returned nil engraving")
	}
	switch mode {
	case "text+qr", "text", "qr":
		// ok
	default:
		t.Fatalf("unexpected mode %q", mode)
	}
	// The engraving must yield at least one command.
	n := 0
	for range eng {
		n++
		if n > 0 {
			break
		}
	}
	if n == 0 {
		t.Fatal("engraving yielded no commands")
	}
}

func TestEngraveBestOversizeErrors(t *testing.T) {
	// A very long string cannot fit any plate mode.
	huge := strings.Repeat("md1yqpqqxqq8xtwhw4xwn4qh", 200)
	_, _, err := engraveBest(huge)
	if err == nil {
		t.Fatal("expected oversize string to fail, got nil error")
	}
}

func TestEngraveModeForcesMode(t *testing.T) {
	eng, err := engraveMode("text", MD1_REF)
	if err != nil {
		t.Fatalf("engraveMode(text): %v", err)
	}
	if eng == nil {
		t.Fatal("engraveMode returned nil engraving")
	}
}

func TestEngraveModeUnknownErrors(t *testing.T) {
	_, err := engraveMode("nonsense", MD1_REF)
	if err == nil {
		t.Fatal("expected unknown mode to error")
	}
}
