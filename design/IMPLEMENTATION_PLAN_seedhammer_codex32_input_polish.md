# SeedHammer CODEX32 Input Polish (Cycle A1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Polish the on-device CODEX32 single-share entry UX in the SeedHammer fork — error-class feedback, a window-aware char counter + live field parse, a pre-engrave confirmation screen, and a full-QWERTY keypad with `b/i/o` statically dimmed.

**Architecture:** Five changes. **C1** adds a small fail-soft API to the `codex32` package (`Describe`, `ParsePrefix`/`Fields`, exported length constants) — pure, host-tested first; it is the one-way-door change everything else consumes. **C2/C3** rewrite `inputCodex32Flow`'s readout (one `New`+`ParsePrefix` per frame → status/field/feedback lines, clamped). **C4** inserts a bespoke pre-engrave confirm screen in `engraveObjectFlow`'s `case codex32.String:` block, branching on the RAW share index (never `Split()`). **C5** swaps the codex32 keypad to the BIP-39 full-QWERTY layout and statically dims `b/i/o`. The OK button stays gated solely on `codex32.New(...) == nil`; `Describe`/`ParsePrefix` are advisory (display only) and can never widen acceptance.

**Tech Stack:** Go/TinyGo (host tests run with the standard Go toolchain at `/home/bcg/.local/go/bin/go`). GUI test harness: `runUI` + `(*op.Drawer).ExtractText` + `uiContains` (asserts rendered text), `runes`/`click`/`press`, `descriptorTheme`, `newPlatform()`. codex32 package tests are white-box (`package codex32`) with in-package BIP-93 vectors and `errors.Is` assertions.

**Base:** fork `main` `3c4d3d3` (A0 — Slice 1's BIP-39 polish — already merged). Branch `feat/codex32-input-polish`. Fork-side only; **no upstream PR**.

**Spec:** `design/SPEC_seedhammer_codex32_input_polish.md` (R0 GREEN — `design/agent-reports/seedhammer-codex32-polish-spec-review-R0.md`).

**PLAN R0 GATE: PASSED (GREEN — 0C/0I at R1).** R0 caught a real build bug (Task 4 unused-import compile failure, IMP-2) → folded; R1 verified the fold + zero new findings. Reviews persisted verbatim to `design/agent-reports/seedhammer-codex32-polish-plan-review-R{0,1}.md`. Cleared for implementation.

**Build order (architect-recommended):** C1 (Tasks 1–3) → C2/C3 (Task 4) → C4 (Task 5) → C5 (Task 6). C5 is last so it and C2/C3 don't both edit `inputCodex32Flow` in overlapping tasks.

---

## File Structure

| File | Responsibility | Tasks |
|---|---|---|
| `codex32/polish.go` *(new)* | Exported length consts; `Describe(err) string`; `Fields` + `ParsePrefix(frag) (Fields, error)`; `checkCase` helper. Pure, no GUI deps. | 1,2,3 |
| `codex32/polish_test.go` *(new)* | White-box table tests for the above. | 1,2,3 |
| `gui/gui.go` *(modify)* | Rewrite `inputCodex32Flow` body (C2/C3); insert confirm gate in `engraveObjectFlow` `case codex32.String:` (C4); swap codex32 keyboard init (C5). | 4,5,6 |
| `gui/codex32_polish.go` *(new)* | GUI helpers: `codex32StatusLine`, `codex32FieldLine`, `codex32Feedback` (C2/C3); `confirmCodex32Flow` (C4); `codex32Keys` const + `newCodex32Keyboard` (C5). | 4,5,6 |
| `gui/codex32_polish_test.go` *(new)* | Pure-helper unit tests + `runUI` integration tests for C2/C3/C4/C5. | 4,5,6 |
| `gui/codex32_input_test.go` *(unchanged — must stay green)* | Existing `TestInputSeedCodex32` (Button3-accept). | guard |
| `gui/gui_test.go` *(unchanged — must stay green)* | `TestWordKeyboardScreen`, `TestWordFlow*` (BIP-39 no-contamination guards). | guard |

**Commit hygiene:** stage explicit paths (no `git add -A`). Commits signed + DCO sign-off, author Brian Goss: `git commit -S -s -m "…"`.

---

## Task 0: Isolated worktree + clean baseline

**Files:** none (workspace setup).

- [ ] **Step 1: Create the worktree off fork main**

```bash
cd /scratch/code/shibboleth/seedhammer
git rev-parse --short HEAD            # expect 3c4d3d3 (A0 base)
git worktree add /scratch/code/shibboleth/seedhammer-wt-codex32 -b feat/codex32-input-polish 3c4d3d3
cd /scratch/code/shibboleth/seedhammer-wt-codex32
git config user.name "Brian Goss"    # if not inherited
```

- [ ] **Step 2: Verify clean baseline**

Run: `/home/bcg/.local/go/bin/go test ./codex32/... ./gui/...`
Expected: PASS (the A0 base is green, including `TestInputSeedCodex32`, `TestWordKeyboardScreen`, `TestWordFlowMatchCount`).

If the baseline fails, STOP and report — do not build on a red base.

---

## Task 1: C1a — export codex32 length constants

**Files:**
- Create: `codex32/polish.go`
- Create: `codex32/polish_test.go`

- [ ] **Step 1: Write the failing test** — `codex32/polish_test.go`

```go
package codex32

import (
	"errors"
	"testing"
)

func TestExportedLengthConstants(t *testing.T) {
	cases := []struct {
		name      string
		got, want int
	}{
		{"ShortCodeMinLength", ShortCodeMinLength, 48},
		{"ShortCodeMaxLength", ShortCodeMaxLength, 93},
		{"LongCodeMinLength", LongCodeMinLength, 125},
		{"LongCodeMaxLength", LongCodeMaxLength, 127},
	}
	for _, c := range cases {
		if c.got != c.want {
			t.Errorf("%s = %d, want %d", c.name, c.got, c.want)
		}
	}
	if ShortCodeMinLength != shortCodeMinLength ||
		ShortCodeMaxLength != shortCodeMaxLength ||
		LongCodeMinLength != longCodeMinLength ||
		LongCodeMaxLength != longCodeMaxLength {
		t.Error("exported length consts diverge from private originals")
	}
}

var _ = errors.Is // keep errors imported for Tasks 2-3 in this file
```

- [ ] **Step 2: Run it to verify it fails** — Run: `/home/bcg/.local/go/bin/go test ./codex32/ -run TestExportedLengthConstants`
Expected: FAIL — `undefined: ShortCodeMinLength` (compile error).

- [ ] **Step 3: Write the minimal implementation** — `codex32/polish.go`

```go
// Package codex32 polish helpers (Cycle A1): a fail-soft partial parser and an
// error classifier for on-device input feedback, plus exported length bounds.
// These are advisory: New remains the sole validity authority.
package codex32

// Exported codex32 total-length bounds (BIP-93 / firmware gate). A valid string
// is in [ShortCodeMinLength, ShortCodeMaxLength] (short checksum) or
// [LongCodeMinLength, LongCodeMaxLength] (long checksum). 94..124 is never valid
// (BIP-93: "a data part of 94 or 95 characters is never legal"); the firmware's
// long window is the conservative subset 125..127.
const (
	ShortCodeMinLength = shortCodeMinLength // 48
	ShortCodeMaxLength = shortCodeMaxLength // 93
	LongCodeMinLength  = longCodeMinLength  // 125
	LongCodeMaxLength  = longCodeMaxLength  // 127
)
```

- [ ] **Step 4: Run it to verify it passes** — Run: `/home/bcg/.local/go/bin/go test ./codex32/ -run TestExportedLengthConstants`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add codex32/polish.go codex32/polish_test.go
git commit -S -s -m "codex32: export length-bound constants for the GUI window model"
```

---

## Task 2: C1b — `Describe(err) string`

**Files:**
- Modify: `codex32/polish.go`
- Modify: `codex32/polish_test.go`

**Context:** `New` wraps every error with `fmt.Errorf("codex32: %w", …)`, so `errors.Is` matches the private sentinels (proven by the existing `TestBIPBadChecksums` at `codex32_test.go:185`). Map only the seven sentinels `New` can return; the `Interpolate`-only sentinels (`errInsufficientShares`, `errMismatched*`, `errInvalidIDLength`, `errRepeatedIndex`) and the one non-sentinel `fmt.Errorf("invalid character: %c")` in `inputHRP`'s second loop fall through to "invalid".

- [ ] **Step 1: Write the failing test** — append to `codex32/polish_test.go`

```go
func TestDescribe(t *testing.T) {
	if got := Describe(nil); got != "" {
		t.Errorf("Describe(nil) = %q, want \"\"", got)
	}
	sentinels := []struct {
		in   error
		want string
	}{
		{errInvalidChecksum, "bad checksum"},
		{errInvalidLength, "wrong length"},
		{errInvalidCharacter, "invalid character"},
		{errInvalidCase, "mixed case"},
		{errInvalidThreshold, "bad threshold"},
		{errInvalidShareIndex, "bad share index"},
		{errIncompleteGroup, "incomplete group"},
		{errInsufficientShares, "invalid"}, // Interpolate-only → fallback
		{errors.New("other"), "invalid"},
	}
	for _, c := range sentinels {
		if got := Describe(c.in); got != c.want {
			t.Errorf("Describe(%v) = %q, want %q", c.in, got, c.want)
		}
	}
	// Real New errors (wrapped) classify correctly.
	if _, err := New("tooshort"); Describe(err) != "wrong length" {
		t.Errorf("Describe(New short) = %q, want \"wrong length\"", Describe(err))
	}
	if _, err := New("ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxve740yyge2ghp"); Describe(err) != "bad checksum" {
		t.Errorf("Describe(New bad-checksum) = %q, want \"bad checksum\"", Describe(err))
	}
}
```

- [ ] **Step 2: Run it to verify it fails** — Run: `/home/bcg/.local/go/bin/go test ./codex32/ -run TestDescribe`
Expected: FAIL — `undefined: Describe`.

- [ ] **Step 3: Write the minimal implementation** — append to `codex32/polish.go`

Add `import "errors"` at the top of `polish.go` (a fresh import block under the `package codex32` clause), then:

```go
// Describe returns a short human-readable label for an error returned by New,
// suitable for on-device display, or "" for a nil error. Unknown non-nil errors
// map to "invalid".
func Describe(err error) string {
	switch {
	case err == nil:
		return ""
	case errors.Is(err, errInvalidChecksum):
		return "bad checksum"
	case errors.Is(err, errInvalidLength):
		return "wrong length"
	case errors.Is(err, errInvalidCharacter):
		return "invalid character"
	case errors.Is(err, errInvalidCase):
		return "mixed case"
	case errors.Is(err, errInvalidThreshold):
		return "bad threshold"
	case errors.Is(err, errInvalidShareIndex):
		return "bad share index"
	case errors.Is(err, errIncompleteGroup):
		return "incomplete group"
	default:
		return "invalid"
	}
}
```

Remove the `var _ = errors.Is` placeholder line from `polish_test.go` (the real test now uses `errors`).

- [ ] **Step 4: Run it to verify it passes** — Run: `/home/bcg/.local/go/bin/go test ./codex32/ -run TestDescribe`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add codex32/polish.go codex32/polish_test.go
git commit -S -s -m "codex32: add Describe(err) error classifier for UI feedback"
```

---

## Task 3: C1c — `ParsePrefix` + `Fields`

**Files:**
- Modify: `codex32/polish.go`
- Modify: `codex32/polish_test.go`

**Context:** `ParsePrefix` is a FRESH fail-soft parser — it must NOT reuse `partsInner` (which indexes `res[0]`/`res[5]`/`res[1:5]` unconditionally and `panic("unreacable")` on malformed input). Use `splitHRP` (returns `("", whole)` until a `1` separator) + the non-panicking `feFromRune` (`(fe, bool)`). Data part = everything after the first `1`. Offsets (from `partsInner`): threshold `data[0]`, identifier `data[1:5]`, share index `data[5]`. Determinability/timing (spec §4.1c): before the `1`, nothing is `…Known`; the threshold-0 ⇒ index-`s`/`S` rule is only enforceable at len≥6. Errors are the same sentinels `New`/`Describe` use, wrapped with `%w`.

- [ ] **Step 1: Write the failing test** — append to `codex32/polish_test.go`

```go
func TestParsePrefix(t *testing.T) {
	// No separator yet: HRP-candidate chars, nothing Known, no error.
	f, err := ParsePrefix("ms")
	if err != nil {
		t.Fatalf("ParsePrefix(ms) err=%v", err)
	}
	if f.HRP != "" || f.ThresholdKnown || f.IdentifierKnown || f.ShareIndexKnown {
		t.Errorf("ParsePrefix(ms) = %+v, want all-unknown", f)
	}

	// Threshold known at len>=1 after the separator; id not yet.
	f, _ = ParsePrefix("ms12")
	if !f.ThresholdKnown || f.Threshold != 2 {
		t.Errorf("ParsePrefix(ms12) threshold: %+v", f)
	}
	if f.IdentifierKnown {
		t.Errorf("ParsePrefix(ms12) id should be unknown: %+v", f)
	}

	// Threshold '1' is forbidden.
	if _, err := ParsePrefix("ms11"); !errors.Is(err, errInvalidThreshold) {
		t.Errorf("ParsePrefix(ms11) err=%v, want errInvalidThreshold", err)
	}

	// Identifier known at len>=5.
	f, _ = ParsePrefix("ms12name")
	if !f.IdentifierKnown || f.Identifier != "name" {
		t.Errorf("ParsePrefix(ms12name) id: %+v", f)
	}

	// Share index + Unshared at len>=6 (BIP-93 vector 1 prefix).
	f, err = ParsePrefix("ms10tests")
	if err != nil {
		t.Fatalf("ParsePrefix(ms10tests) err=%v", err)
	}
	if !f.ShareIndexKnown || f.ShareIndex != 's' || !f.Unshared {
		t.Errorf("ParsePrefix(ms10tests) share: %+v", f)
	}

	// threshold-0 with a non-S index at len>=6 → determinable error.
	if _, err := ParsePrefix("ms10testa"); !errors.Is(err, errInvalidShareIndex) {
		t.Errorf("ParsePrefix(ms10testa) err=%v, want errInvalidShareIndex", err)
	}

	// threshold-0 leading but len<6 → NOT yet determinable (no error).
	if _, err := ParsePrefix("ms10te"); err != nil {
		t.Errorf("ParsePrefix(ms10te) err=%v, want nil (not determinable yet)", err)
	}

	// Non-bech32 char in the identifier ('b' is excluded from bech32).
	if _, err := ParsePrefix("ms12bbbb"); !errors.Is(err, errInvalidCharacter) {
		t.Errorf("ParsePrefix(ms12bbbb) err=%v, want errInvalidCharacter", err)
	}

	// Mixed case is determinable.
	if _, err := ParsePrefix("Ms10tests"); !errors.Is(err, errInvalidCase) {
		t.Errorf("ParsePrefix(Ms10tests) err=%v, want errInvalidCase", err)
	}

	// A full New-valid string parses cleanly with full fields.
	f, err = ParsePrefix("ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw")
	if err != nil {
		t.Fatalf("ParsePrefix(full) err=%v", err)
	}
	if !f.Unshared || f.Identifier != "test" || f.HRP != "ms" {
		t.Errorf("ParsePrefix(full) = %+v", f)
	}

	// Uppercase (keypad-form) parses identically.
	f, err = ParsePrefix("MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM")
	if err != nil {
		t.Fatalf("ParsePrefix(MS12NAME…) err=%v", err)
	}
	if f.Threshold != 2 || f.Identifier != "NAME" || f.ShareIndex != 'A' || f.Unshared {
		t.Errorf("ParsePrefix(MS12NAME…) = %+v", f)
	}
}
```

- [ ] **Step 2: Run it to verify it fails** — Run: `/home/bcg/.local/go/bin/go test ./codex32/ -run TestParsePrefix`
Expected: FAIL — `undefined: ParsePrefix` / `undefined: Fields`.

- [ ] **Step 3: Write the minimal implementation** — append to `codex32/polish.go`

Add `"fmt"` to `polish.go`'s import block (alongside `"errors"`), then:

```go
// Fields holds the codex32 header fields determinable from an in-progress
// fragment. Each XxxKnown flag is true once that field is present and valid.
type Fields struct {
	HRP             string // "" until the '1' separator is seen
	Threshold       int    // 0,2..9; valid only if ThresholdKnown
	ThresholdKnown  bool
	Identifier      string // up to 4 chars; valid only if IdentifierKnown
	IdentifierKnown bool
	ShareIndex      rune // valid only if ShareIndexKnown
	ShareIndexKnown bool
	Unshared        bool // ShareIndexKnown && ShareIndex is 's'/'S'
}

// ParsePrefix fail-soft-parses the determinable header fields of an in-progress
// codex32 fragment without panicking. The returned error is non-nil ONLY for a
// determinable violation (mixed case, non-bech32 character, bad threshold digit,
// or threshold-0 without index S); a merely-too-short fragment returns
// (partialFields, nil). It never splits payload/checksum — their boundary
// depends on the final total length. Errors are the same sentinels New uses
// (wrapped with %w), so Describe maps them. Advisory only: New stays the sole
// validity authority.
func ParsePrefix(frag string) (Fields, error) {
	var f Fields
	// When splitHRP finds no '1', it returns ("", frag) — so `data` aliases the
	// ENTIRE input in that case. We early-return below before touching `data`,
	// so no field is read from the not-yet-data prefix.
	hrp, data := splitHRP(frag)
	// Case consistency (HRP + data) is determinable at any length.
	if err := checkCase(frag); err != nil {
		return f, fmt.Errorf("codex32: %w", err)
	}
	if hrp == "" {
		// No '1' separator yet: the typed chars are HRP candidates, not data.
		return f, nil
	}
	// HRP is recorded for display, not independently rejected: New is the
	// authority and surfaces a wrong HRP as a checksum mismatch (it folds the
	// HRP into the checksum), so ParsePrefix stays consistent with New.
	f.HRP = hrp

	// Threshold: data[0] at len>=1 (∈ {0,2..9}; '1' and non-digits are invalid).
	if len(data) >= 1 {
		switch data[0] {
		case '0', '2', '3', '4', '5', '6', '7', '8', '9':
			f.Threshold = int(data[0] - '0')
			f.ThresholdKnown = true
		default:
			return f, fmt.Errorf("codex32: %w", errInvalidThreshold)
		}
	}

	// Identifier: data[1:5] at len>=5; each char must be bech32.
	if len(data) >= 5 {
		for _, c := range data[1:5] {
			if _, ok := feFromRune(c); !ok {
				return f, fmt.Errorf("codex32: %w", errInvalidCharacter)
			}
		}
		f.Identifier = data[1:5]
		f.IdentifierKnown = true
	}

	// Share index: data[5] at len>=6; bech32; threshold-0 ⇒ index s/S.
	if len(data) >= 6 {
		// data[5] is a byte; rune(byte) is the codepoint for ASCII (all valid
		// bech32 is ASCII). Non-ASCII bytes (128..255) make feFromRune return
		// false below → "invalid character", never a panic.
		idx := rune(data[5])
		if _, ok := feFromRune(idx); !ok {
			return f, fmt.Errorf("codex32: %w", errInvalidCharacter)
		}
		f.ShareIndex = idx
		f.ShareIndexKnown = true
		f.Unshared = idx == 's' || idx == 'S'
		if f.ThresholdKnown && f.Threshold == 0 && !f.Unshared {
			return f, fmt.Errorf("codex32: %w", errInvalidShareIndex)
		}
	}
	return f, nil
}

// checkCase returns errInvalidCase if frag mixes upper- and lower-case ASCII
// letters (digits are case-neutral) — matching the engine's case rule, for
// display honesty. Moot on the force-uppercasing keypad, but the package API
// should not silently accept mixed case.
func checkCase(frag string) error {
	hasUpper, hasLower := false, false
	for _, c := range frag {
		switch {
		case c >= 'a' && c <= 'z':
			hasLower = true
		case c >= 'A' && c <= 'Z':
			hasUpper = true
		}
	}
	if hasUpper && hasLower {
		return errInvalidCase
	}
	return nil
}
```

- [ ] **Step 4: Run it to verify it passes** — Run: `/home/bcg/.local/go/bin/go test ./codex32/...`
Expected: PASS (all of Task 1–3 plus the pre-existing codex32 tests).

- [ ] **Step 5: Commit**

```bash
git add codex32/polish.go codex32/polish_test.go
git commit -S -s -m "codex32: add fail-soft ParsePrefix + Fields for live input parsing"
```

---

## Task 4: C2/C3 — error-class feedback + window-model counter + live field parse

**Files:**
- Create: `gui/codex32_polish.go`
- Create: `gui/codex32_polish_test.go`
- Modify: `gui/gui.go` (rewrite `inputCodex32Flow`, currently `:672-731`)

**Context:** `inputCodex32Flow` runs `for !ctx.Done { for kbd.Update(ctx) { … }; …; ctx.Frame(…) }`. The shared keyboard force-uppercases typed runes, so `kbd.Fragment` is uppercase. The test harness reads rendered text: `(*op.Drawer).ExtractText` concatenates text runs, and `uiContains(content, needle)` lowercases both sides and strips spaces from the needle (`uiContains(c, "1 match")` actually searches for `"1match"`). Assertions below use needles that match space-insensitively. The `inputWordsFlow` clamp precedent (`gui.go:645-648`): `countY := …; if lim := top.Max.Y - csz.Y; countY > lim { countY = lim }`.

**MIN-3 timing:** field errors show eagerly (any length); a checksum/structure error from `New` shows only inside a valid-length window, because `New` returns `errInvalidLength` for 1–47 and 94–124 (length is checked first) — that band is "keep typing", not an error.

- [ ] **Step 1: Write the failing pure-helper tests** — `gui/codex32_polish_test.go`

```go
package gui

import (
	"strings"
	"testing"

	"seedhammer.com/codex32"
)

func TestCodex32StatusLine(t *testing.T) {
	cases := []struct {
		n    int
		want string
	}{
		{0, "0 chars"},
		{47, "47 chars"},
		{48, "short · 48 chars"},
		{93, "short · 93 chars"},
		{94, "94 chars — keep typing"},
		{124, "124 chars — keep typing"},
		{125, "long · 125 chars"},
		{127, "long · 127 chars"},
		{128, "too long"},
	}
	for _, c := range cases {
		if got := codex32StatusLine(c.n); got != c.want {
			t.Errorf("codex32StatusLine(%d) = %q, want %q", c.n, got, c.want)
		}
	}
}

func TestCodex32FieldLine(t *testing.T) {
	f, _ := codex32.ParsePrefix("ms12name")
	if got := codex32FieldLine(f); got != "id NAME · thr 2" {
		t.Errorf("codex32FieldLine(ms12name) = %q", got)
	}
	f, _ = codex32.ParsePrefix("ms10tests")
	if got := codex32FieldLine(f); got != "id TEST · thr 0 · share S" {
		t.Errorf("codex32FieldLine(ms10tests) = %q", got)
	}
	var empty codex32.Fields
	if got := codex32FieldLine(empty); got != "" {
		t.Errorf("codex32FieldLine(empty) = %q, want \"\"", got)
	}
}

func TestCodex32Feedback(t *testing.T) {
	// Eager field error (bad threshold), regardless of length.
	_, perr := codex32.ParsePrefix("MS11")
	if got := codex32Feedback("MS11", perr, nil); got != "bad threshold" {
		t.Errorf("feedback(MS11) = %q, want \"bad threshold\"", got)
	}
	// Dead zone (94..124): no determinable error → suppressed.
	keep := "MS10TESTS" + strings.Repeat("X", 91) // 100 chars
	_, perr = codex32.ParsePrefix(keep)
	_, nerr := codex32.New(keep)
	if got := codex32Feedback(keep, perr, nerr); got != "" {
		t.Errorf("feedback(deadzone) = %q, want \"\"", got)
	}
	// Full-length bad checksum → shown.
	bad := "MS10FAUXSXXXXXXXXXXXXXXXXXXXXXXXXXXVE740YYGE2GHP"
	_, perr = codex32.ParsePrefix(bad)
	_, nerr = codex32.New(bad)
	if got := codex32Feedback(bad, perr, nerr); got != "bad checksum" {
		t.Errorf("feedback(badchecksum) = %q, want \"bad checksum\"", got)
	}
}
```

- [ ] **Step 2: Run to verify they fail** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestCodex32StatusLine|TestCodex32FieldLine|TestCodex32Feedback'`
Expected: FAIL — `undefined: codex32StatusLine` (compile error).

- [ ] **Step 3: Write the helpers** — `gui/codex32_polish.go`

> **Import discipline (R0 IMP-2):** this step's helpers use ONLY `fmt`, `strings`, and `seedhammer.com/codex32`. Use exactly the import block below — Go rejects unused imports, so do NOT add the GUI subpackage imports here. Task 5 Step 3 expands this block (adding `image`, `assets`, `layout`, `op`, `widget`) when `confirmCodex32Flow` is appended.

```go
package gui

import (
	"fmt"
	"strings"

	"seedhammer.com/codex32"
)

// codex32StatusLine returns the window-aware length readout for an in-progress
// codex32 fragment of length n. There is no single target: BIP-93 short totals
// are 48..93, the firmware long window is 125..127, and 94..124 is a dead zone
// that is not (yet) an error.
func codex32StatusLine(n int) string {
	switch {
	case n < codex32.ShortCodeMinLength:
		return fmt.Sprintf("%d chars", n)
	case n <= codex32.ShortCodeMaxLength:
		return fmt.Sprintf("short · %d chars", n)
	case n < codex32.LongCodeMinLength:
		return fmt.Sprintf("%d chars — keep typing", n)
	case n <= codex32.LongCodeMaxLength:
		return fmt.Sprintf("long · %d chars", n)
	default:
		return "too long"
	}
}

// codex32FieldLine renders the parsed header fields as "id NAME · thr 2 · share C",
// each segment appearing once its field is known. Returns "" if nothing is known.
func codex32FieldLine(f codex32.Fields) string {
	var segs []string
	if f.IdentifierKnown {
		segs = append(segs, "id "+strings.ToUpper(f.Identifier))
	}
	if f.ThresholdKnown {
		segs = append(segs, fmt.Sprintf("thr %d", f.Threshold))
	}
	if f.ShareIndexKnown {
		segs = append(segs, "share "+strings.ToUpper(string(f.ShareIndex)))
	}
	return strings.Join(segs, " · ")
}

// codex32Feedback returns an error label to show under the entry, or "" if the
// fragment is fine so far. Field errors (from ParsePrefix) show eagerly; a
// checksum/structure error from New shows only once the fragment reaches a valid
// length window (so a half-typed string isn't flagged "wrong length").
func codex32Feedback(frag string, perr, nerr error) string {
	if perr != nil {
		return codex32.Describe(perr)
	}
	n := len(frag)
	inWindow := (n >= codex32.ShortCodeMinLength && n <= codex32.ShortCodeMaxLength) ||
		(n >= codex32.LongCodeMinLength && n <= codex32.LongCodeMaxLength)
	if inWindow && nerr != nil {
		return codex32.Describe(nerr)
	}
	return ""
}
```

- [ ] **Step 4: Run to verify the helpers pass** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestCodex32StatusLine|TestCodex32FieldLine|TestCodex32Feedback'`
Expected: PASS. (`gui/codex32_polish.go` at this point imports only `fmt`/`strings`/`codex32` — no unused-import build error.)

- [ ] **Step 5: Write the failing integration test** — append to `gui/codex32_polish_test.go`

```go
// codex32Frame runs inputCodex32Flow, types `typed` (uppercased by the keypad),
// and returns the first rendered frame's extracted text.
func codex32Frame(t *testing.T, typed string) string {
	t.Helper()
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() {
		inputCodex32Flow(ctx, &descriptorTheme)
	})
	defer quit()
	if typed != "" {
		runes(&ctx.Router, typed)
	}
	content, ok := frame()
	if !ok {
		t.Fatal("no frame")
	}
	return content
}

func TestCodex32FlowReadout(t *testing.T) {
	if c := codex32Frame(t, ""); !uiContains(c, "0 chars") {
		t.Errorf("empty: want \"0 chars\"; got %q", c)
	}
	if c := codex32Frame(t, "ms12name"); !uiContains(c, "id NAME") || !uiContains(c, "thr 2") {
		t.Errorf("fields: want id NAME + thr 2; got %q", c)
	}
	if c := codex32Frame(t, "ms11"); !uiContains(c, "bad threshold") {
		t.Errorf("bad threshold: got %q", c)
	}
	keep := "ms10tests" + strings.Repeat("x", 91) // 100 chars → dead zone
	if c := codex32Frame(t, keep); !uiContains(c, "keep typing") {
		t.Errorf("keep typing: got %q", c)
	}
	bad := "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxve740yyge2ghp" // valid len, bad checksum
	if c := codex32Frame(t, bad); !uiContains(c, "bad checksum") {
		t.Errorf("bad checksum: got %q", c)
	}
}
```

- [ ] **Step 6: Run to verify it fails** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run TestCodex32FlowReadout`
Expected: FAIL — the current `inputCodex32Flow` renders no status/field/feedback lines, so the assertions miss.

- [ ] **Step 7: Rewrite `inputCodex32Flow`** — replace the entire function body (`gui/gui.go:672-731`) with:

```go
func inputCodex32Flow(ctx *Context, th *Colors) (codex32.String, bool) {
	const alph = "1234567890\nqwertyup\nasdfghjk\nlzxcvnm"

	kbd := NewKeyboard(ctx, alph)
	backBtn := &Clickable{Button: Button1}
	okBtn := &Clickable{Button: Button3}
	for !ctx.Done {
		for kbd.Update(ctx) {
		}
		// Parse once per frame (MIN-3): New gates acceptance; ParsePrefix drives
		// the advisory readout.
		share, nerr := codex32.New(kbd.Fragment)
		parsed, perr := codex32.ParsePrefix(kbd.Fragment)
		valid := nerr == nil

		if backBtn.Clicked(ctx) {
			break
		}
		if valid && okBtn.Clicked(ctx) {
			return share, true
		}
		dims := ctx.Platform.DisplaySize()

		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		content, _ = content.CutBottom(8)

		kbdOp, kbdsz := kbd.Layout(ctx, th)
		kbdOp = kbdOp.Offset(content.S(kbdsz))

		word, frgSize := widget.Labelw(&ctx.B, ctx.Styles.word, dims.X-50, th.Background, kbd.Fragment)
		frgSize.X = max(frgSize.X, 100)
		r := image.Rectangle{Max: frgSize}
		r.Min.Y -= 3
		r.Max.Y += buttonPadY
		r.Min.X -= buttonPadX
		r.Max.X += buttonPadX
		top, _ := content.CutBottom(kbdsz.Y)
		wordOff := top.Center(frgSize)
		word = op.Layer(
			word,
			op.Compose(
				op.Color(&ctx.B, th.Text),
				op.RoundedRect2(&ctx.B, r, cornerRadius),
			),
		).Offset(wordOff)

		// Status line + (feedback | field line), stacked below the fragment box,
		// each clamped so it never overlaps the keyboard (mirrors inputWordsFlow).
		var infoOps []op.Op
		lineY := wordOff.Y + frgSize.Y + 8
		addLine := func(s string) {
			if s == "" {
				return
			}
			lbl, sz := widget.Label(&ctx.B, ctx.Styles.body, th.Text, s)
			y := lineY
			if lim := top.Max.Y - sz.Y; y > lim {
				y = lim
			}
			infoOps = append(infoOps, lbl.Offset(image.Pt((dims.X-sz.X)/2, y)))
			lineY = y + sz.Y + 4
		}
		addLine(codex32StatusLine(len(kbd.Fragment)))
		if fb := codex32Feedback(kbd.Fragment, perr, nerr); fb != "" {
			addLine(fb)
		} else {
			addLine(codex32FieldLine(parsed))
		}

		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack}}...)
		if valid {
			nav2, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{{Clickable: okBtn, Style: StylePrimary, Icon: assets.IconCheckmark}}...)
			nav = op.Layer(nav, nav2)
		}
		title, _ := layoutTitle(ctx, dims.X, th.Text, "Input Codex32 Share")

		frameOps := []op.Op{kbdOp, word}
		frameOps = append(frameOps, infoOps...)
		frameOps = append(frameOps, nav, title, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
	return codex32.String{}, false
}
```

If `gui.go` does not already import `"fmt"`/`"strings"`, they are NOT needed here (the helpers live in `codex32_polish.go`); `inputCodex32Flow` uses only already-imported packages (`image`, `op`, `widget`, `layout`, `assets`, `codex32`). Confirm no new imports are required in `gui.go`.

- [ ] **Step 8: Run the targeted + guard tests** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestCodex32|TestInputSeedCodex32|TestWordFlow|TestWordKeyboardScreen'`
Expected: PASS — the new readout tests pass AND the existing `TestInputSeedCodex32` (Button3-accept) and BIP-39 guards stay green.

- [ ] **Step 9: Full package test** — Run: `/home/bcg/.local/go/bin/go test ./codex32/... ./gui/...`
Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git add gui/codex32_polish.go gui/codex32_polish_test.go gui/gui.go
git commit -S -s -m "gui(codex32): live error feedback + window-model char counter + field parse"
```

---

## Task 5: C4 — pre-engrave confirmation screen

**Files:**
- Modify: `gui/codex32_polish.go` (add `confirmCodex32Flow`)
- Modify: `gui/gui.go` (`engraveObjectFlow` `case codex32.String:`, currently `:1819-1826`)
- Modify: `gui/codex32_polish_test.go`

**Context:** Today the `case codex32.String:` block engraves immediately (no confirm). Insert a bespoke confirm screen modeled on `SeedScreen.Confirm`/`mdmkFlow`: own `Clickable`s (Back=Button1, Engrave=Button3+Center), a `for !ctx.Done` loop, `layoutNavigation` + labels, `ctx.Frame(op.Layer(…))`, returning `bool`. Branch on the RAW share index from `ParsePrefix` — NOT `Split()` (which remaps threshold 0→1 and would mislabel the unshared secret). `assets.IconHammer` is the engrave icon.

- [ ] **Step 1: Write the failing tests** — append to `gui/codex32_polish_test.go`

```go
func TestConfirmCodex32Unshared(t *testing.T) {
	s, err := codex32.New("ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw")
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() { confirmCodex32Flow(ctx, &descriptorTheme, s) })
	defer quit()
	c, ok := frame()
	if !ok {
		t.Fatal("no frame")
	}
	if !uiContains(c, "Unshared secret") {
		t.Errorf("unshared: want \"Unshared secret\"; got %q", c)
	}
	if !uiContains(c, "id TEST") {
		t.Errorf("unshared id: got %q", c)
	}
}

func TestConfirmCodex32Share(t *testing.T) {
	s, err := codex32.New("MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM")
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := NewContext(newPlatform())
	frame, quit := runUI(ctx, func() { confirmCodex32Flow(ctx, &descriptorTheme, s) })
	defer quit()
	c, ok := frame()
	if !ok {
		t.Fatal("no frame")
	}
	if !uiContains(c, "Share A") {
		t.Errorf("share: want \"Share A\"; got %q", c)
	}
	if !uiContains(c, "not a recovered seed") {
		t.Errorf("share note: got %q", c)
	}
}
```

- [ ] **Step 2: Run to verify they fail** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestConfirmCodex32'`
Expected: FAIL — `undefined: confirmCodex32Flow`.

- [ ] **Step 3: Implement `confirmCodex32Flow`** — append to `gui/codex32_polish.go`

First, **expand the import block** at the top of `gui/codex32_polish.go` (R0 IMP-2) to add the GUI imports `confirmCodex32Flow` needs — the block becomes:

```go
import (
	"fmt"
	"image"
	"strings"

	"seedhammer.com/codex32"
	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
)
```

Then append:

```go
// confirmCodex32Flow shows a pre-engrave review of a (New-valid) codex32 share
// and returns true to engrave, false to go back. It branches on the RAW share
// index from ParsePrefix (NOT Split(), which remaps an unshared secret's
// threshold 0→1, mislabeling it). The codex32 string is engraved verbatim;
// multi-share recovery is a separate cycle.
func confirmCodex32Flow(ctx *Context, th *Colors, scan codex32.String) bool {
	f, _ := codex32.ParsePrefix(scan.String()) // scan is New-valid → no error
	lines := []string{"id " + strings.ToUpper(f.Identifier)}
	if f.Unshared {
		lines = append(lines, "Unshared secret (S)")
	} else {
		lines = append(lines,
			"Share "+strings.ToUpper(string(f.ShareIndex))+" of a k-of-n set",
			"engraves THIS share, not a recovered seed",
		)
	}
	lines = append(lines, fmt.Sprintf("%d chars", len(scan.String())))

	backBtn := &Clickable{Button: Button1}
	engraveBtn := &Clickable{Button: Button3, AltButton: Center}
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return false
		}
		if engraveBtn.Clicked(ctx) {
			return true
		}
		dims := ctx.Platform.DisplaySize()
		nav, _ := layoutNavigation(&ctx.B, th, dims, []NavButton{
			{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack},
			{Clickable: engraveBtn, Style: StylePrimary, Icon: assets.IconHammer},
		}...)
		title, _ := layoutTitle(ctx, dims.X, th.Text, "Confirm Codex32 Share")

		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		content, _ = content.CutBottom(leadingSize)
		body := make([]op.Op, 0, len(lines))
		y := content.Min.Y + 8
		for _, ln := range lines {
			lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, dims.X-2*8, th.Text, ln)
			body = append(body, lbl.Offset(image.Pt((dims.X-sz.X)/2, y)))
			y += sz.Y + 6
		}
		frameOps := append([]op.Op{nav, title}, body...)
		frameOps = append(frameOps, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
	return false
}
```

- [ ] **Step 4: Run to verify the confirm tests pass** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestConfirmCodex32'`
Expected: PASS.

- [ ] **Step 5: Gate the engrave on the confirm** — in `gui/gui.go`, `engraveObjectFlow`'s `case codex32.String:` block (`:1819-1826`), insert the confirm gate:

```go
	case codex32.String:
		if !confirmCodex32Flow(ctx, th, scan) {
			return false
		}
		id, _, _ := scan.Split()
		s := backup.SeedString{
			Title: id,
			Seed:  scan.String(),
			Font:  constant.Font,
		}
		backupSeedStringFlow(ctx, th, s)
```

> Verify the back-out return matches `engraveObjectFlow`'s "not engraved" convention: read the function's existing return value(s) for the codex32/other cases. If the function returns `true` after `backupSeedStringFlow` (engrave handled), keep that path unchanged and use `return false` on back-out (as above). If the function's not-handled sentinel differs, match it. Do not change any other case.

- [ ] **Step 6: Run guard + full tests** — Run: `/home/bcg/.local/go/bin/go test ./codex32/... ./gui/...`
Expected: PASS (incl. `TestInputSeedCodex32` — note that test drives `newInputFlow` and inspects the returned value; it does not exercise `engraveObjectFlow`, so the confirm gate does not affect it).

- [ ] **Step 7: Commit**

```bash
git add gui/codex32_polish.go gui/gui.go gui/codex32_polish_test.go
git commit -S -s -m "gui(codex32): pre-engrave confirm screen (raw-index, share vs unshared)"
```

---

## Task 6: C5 — full-QWERTY keypad with `b/i/o` statically dimmed

**Files:**
- Modify: `gui/codex32_polish.go` (add `codex32Keys` + `newCodex32Keyboard`)
- Modify: `gui/gui.go` (`inputCodex32Flow` keyboard init)
- Modify: `gui/codex32_polish_test.go`

**Context:** `NewKeyboard` builds per-instance `allKeys`/`keys` from the alphabet string; the per-row `keys` slices share the `allKeys` backing array, so mutating `kbd.allKeys[i].disabled` also affects `kbd.keys`. Keys store `r` exactly as written (lowercase here); output uppercases via `unicode.ToUpper`. `Clear()` does NOT reset `disabled`, and `inputCodex32Flow` never calls `updateValidKeys`, so a one-time static dim survives. `Valid()`/`adjust()`/`adjustCol()` skip disabled keys; `Layout` renders them dimmed. `b/i/o` are exactly the QWERTY letters bech32 excludes; `codex32.Alphabet = "QPZRY9X8GF2TVDW0S3JN54KHCE6MUA7L"` (all its letters/digits, plus the `1` separator, are present in the digit row + full QWERTY).

- [ ] **Step 1: Write the failing tests** — append to `gui/codex32_polish_test.go`

```go
func TestCodex32KeyboardDimsBIO(t *testing.T) {
	ctx := NewContext(newPlatform())
	kbd := newCodex32Keyboard(ctx)
	dimmed := map[rune]bool{'b': true, 'i': true, 'o': true}
	for _, k := range kbd.allKeys {
		if dimmed[k.r] && !k.disabled {
			t.Errorf("codex32 key %q should be disabled", k.r)
		}
		if k.r >= 'a' && k.r <= 'z' && !dimmed[k.r] && k.disabled {
			t.Errorf("codex32 key %q should be enabled", k.r)
		}
	}
	// Every codex32.Alphabet char (lowercased) + the '1' separator is present and enabled.
	enabled := map[rune]bool{}
	for _, k := range kbd.allKeys {
		if !k.disabled {
			enabled[k.r] = true
		}
	}
	for _, c := range codex32.Alphabet {
		lc := []rune(strings.ToLower(string(c)))[0]
		if !enabled[lc] {
			t.Errorf("codex32 Alphabet char %q (lc %q) missing/disabled on keypad", c, lc)
		}
	}
	if !enabled['1'] {
		t.Error("codex32 keypad must keep '1' (HRP separator) enabled")
	}
}

func TestBIP39KeyboardNotDimmed(t *testing.T) {
	// Regression: dimming the codex32 instance must not affect the BIP-39 keyboard.
	ctx := NewContext(newPlatform())
	kbd := NewKeyboard(ctx, wordKeys)
	for _, k := range kbd.allKeys {
		switch k.r {
		case 'b', 'i', 'o':
			if k.disabled {
				t.Errorf("BIP-39 key %q must NOT be disabled (no cross-contamination)", k.r)
			}
		}
	}
}
```

- [ ] **Step 2: Run to verify they fail** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestCodex32KeyboardDimsBIO|TestBIP39KeyboardNotDimmed'`
Expected: FAIL — `undefined: newCodex32Keyboard`.

- [ ] **Step 3: Implement the keyboard constructor** — append to `gui/codex32_polish.go`

```go
// codex32Keys is the on-screen codex32 keypad: digit row + the BIP-39
// full-QWERTY letter rows. b/i/o are present (for visual familiarity) but
// statically dimmed by newCodex32Keyboard, since bech32 excludes them.
const codex32Keys = "1234567890\nqwertyuiop\nasdfghjkl\nzxcvbnm"

// newCodex32Keyboard builds the codex32 keypad and statically disables the
// never-valid b/i/o keys. Per-instance, so the BIP-39 keyboard is unaffected.
// Disabling via allKeys also disables the same elements in the per-row keys
// slices (shared backing array); Clear() does not reset disabled.
func newCodex32Keyboard(ctx *Context) *Keyboard {
	kbd := NewKeyboard(ctx, codex32Keys)
	for i := range kbd.allKeys {
		switch kbd.allKeys[i].r {
		case 'b', 'i', 'o':
			kbd.allKeys[i].disabled = true
		}
	}
	return kbd
}
```

- [ ] **Step 4: Wire it into `inputCodex32Flow`** — in `gui/gui.go`, replace the two keyboard-init lines at the top of `inputCodex32Flow`:

```go
	const alph = "1234567890\nqwertyup\nasdfghjk\nlzxcvnm"

	kbd := NewKeyboard(ctx, alph)
```

with:

```go
	kbd := newCodex32Keyboard(ctx)
```

(Delete the now-unused `const alph`.)

- [ ] **Step 5: Run keyboard + guard tests** — Run: `/home/bcg/.local/go/bin/go test ./gui/ -run 'TestCodex32KeyboardDimsBIO|TestBIP39KeyboardNotDimmed|TestInputSeedCodex32|TestWordKeyboardScreen|TestCodex32FlowReadout'`
Expected: PASS — b/i/o dimmed on codex32, enabled on BIP-39; `TestInputSeedCodex32` still types its vector (no b/i/o in it) and accepts on Button3.

- [ ] **Step 6: Full test suite** — Run: `/home/bcg/.local/go/bin/go test ./...`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add gui/codex32_polish.go gui/gui.go gui/codex32_polish_test.go
git commit -S -s -m "gui(codex32): full-QWERTY keypad with b/i/o statically dimmed"
```

---

## Final: whole-diff adversarial execution review (mandatory, non-deferrable)

After all tasks: per the refined ultracode policy, dispatch an independent opus adversarial execution review over the **entire diff** vs `3c4d3d3` (R0 reviewed the plan; this catches implementation-introduced regressions TDD misses). Persist the verbatim report to `design/agent-reports/seedhammer-codex32-polish-execution-review.md`. Fold any Critical/Important findings and re-review until clean.

Review focus:
- C1: `ParsePrefix` cannot panic on any prefix of any BIP-93 vector (incl. the long/512-bit one) or on adversarial junk; `Describe`/`ParsePrefix` are advisory-only and never widen `New`-gated acceptance.
- C2/C3: the once-per-frame `New`+`ParsePrefix` refactor preserves the original accept/back behavior; the two info lines clamp and never overlap the keyboard; `uiContains` assertions are space-insensitive-correct.
- C4: the confirm gate's back-out return matches `engraveObjectFlow`'s convention; `Split()` is untouched; the engrave still uses only `id` from `Split()`.
- C5: `b/i/o` dimming is per-instance (no BIP-39 contamination); every `codex32.Alphabet` char + `1` stays enabled; `TestWordKeyboardScreen` + `TestInputSeedCodex32` green.
- Whole diff: `go test ./...` green; no `mdmk.go` / `Split()` changes; commits signed + DCO, author Brian Goss; staged paths explicit.

Then use **superpowers:finishing-a-development-branch** — but per the user's post-#36 directive, **no upstream PR**: merge `feat/codex32-input-polish` into fork `main` locally (or keep the branch), fork-side only.

---

## Self-Review (author)

- **Spec coverage:** C1 → Tasks 1–3; C2/C3 → Task 4; C4 → Task 5; C5 → Task 6. All four spec polish items + the C1 API are covered; A0/Cycle-B/out-of-scope items correctly excluded.
- **Placeholder scan:** every code/test step shows real code; commands include the explicit `go` path and `-run` filters with expected PASS/FAIL. The one soft spot (C4 back-out return value) is a precise verify-and-match instruction, not a placeholder, because the exact post-switch return wasn't extracted.
- **Type consistency:** `Fields` field names (`ThresholdKnown`/`IdentifierKnown`/`ShareIndexKnown`/`Unshared`) are used identically in Tasks 3/4/5; helper names (`codex32StatusLine`/`codex32FieldLine`/`codex32Feedback`/`newCodex32Keyboard`/`confirmCodex32Flow`) match across tasks and tests; `codex32.Alphabet`/`ShortCodeMinLength`/… match the exported C1 symbols.
- **Build-order safety:** Task 4 keeps the original keyboard init; Task 6 swaps it to `newCodex32Keyboard` — no forward reference. The shared `gui/codex32_polish.go` import note flags the unused-import pitfall if tasks are built out of order.
