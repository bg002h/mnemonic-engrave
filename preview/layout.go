package main

import (
	"fmt"

	qr "github.com/seedhammer/kortschak-qr"
	"seedhammer.com/backup"
	"seedhammer.com/bezier"
	"seedhammer.com/bspline"
	"seedhammer.com/engrave"
	"seedhammer.com/font/sh"
)

const qrScale = 3

func paragraphFor(mode string, s string, qrc *qr.Code) (backup.Paragraph, bool) {
	switch mode {
	case "text+qr":
		return backup.Paragraph{Text: s, QR: qrc, QRScale: qrScale}, true
	case "text":
		return backup.Paragraph{Text: s}, true
	case "qr":
		return backup.Paragraph{QR: qrc, QRScale: qrScale}, true
	}
	return backup.Paragraph{}, false
}

func fits(eng engrave.Engraving) bool {
	b := bspline.Measure(engrave.PlanEngraving(params.StepperConfig, eng)).Bounds
	lo, hi := safetyMarginMM*mm, (plateSizeMM-safetyMarginMM)*mm
	return b.In(bspline.Bounds{Min: bezier.Pt(lo, lo), Max: bezier.Pt(hi, hi)})
}

// engrave_ builds an Engraving for the given mode. backup.EngraveText returns
// a closure-backed iter.Seq that recomputes from scratch on every range, so it
// is safely re-rangeable (verified in backup.go: the returned func re-derives
// all state per call) — fits() and the caller can both range it independently.
func engrave_(mode, s string, qrc *qr.Code) (engrave.Engraving, error) {
	p, ok := paragraphFor(mode, s, qrc)
	if !ok {
		return nil, fmt.Errorf("unknown mode %q", mode)
	}
	return backup.EngraveText(params, backup.Text{Paragraphs: []backup.Paragraph{p}, Font: sh.Font}), nil
}

// engraveBest renders the first fitting mode (text+qr > text > qr), like validateMdmk.
func engraveBest(s string) (engrave.Engraving, string, error) {
	qrc, err := qr.Encode(s, qr.L)
	if err != nil {
		return nil, "", fmt.Errorf("qr encode: %w", err)
	}
	for _, mode := range []string{"text+qr", "text", "qr"} {
		eng, err := engrave_(mode, s, qrc)
		if err != nil {
			return nil, "", err
		}
		if fits(eng) {
			return eng, mode, nil
		}
	}
	return nil, "", fmt.Errorf("string does not fit any plate mode")
}

func engraveMode(mode, s string) (engrave.Engraving, error) {
	qrc, err := qr.Encode(s, qr.L)
	if err != nil {
		return nil, err
	}
	eng, err := engrave_(mode, s, qrc)
	if err != nil {
		return nil, err
	}
	if !fits(eng) {
		return nil, fmt.Errorf("mode %q does not fit a plate", mode)
	}
	return eng, nil
}
