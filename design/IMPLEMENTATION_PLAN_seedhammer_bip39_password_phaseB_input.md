# BIP-39 Password — Phase B: Input Layer — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every character the spec promises both **typeable** and **validated**, and prove the three definitions of "allowed" cannot drift apart.

**Architecture:** A new `passphrase` package holding the two validators, plus a fourth page on the existing `PassphraseKeyboard`. No plate output, no GUI flow, no engraving — Phase B is testable entirely in isolation, which is why it is separated from the plate (Phase C) and flow (Phase D).

**Tech Stack:** Go 1.25 (TinyGo for firmware), Nix devshell.

**Spec:** `design/SPEC_seedhammer_engrave_bip39_password.md` — **R0 GREEN**, amendment GREEN. Relevant: §3.4 (validation), §5.0 (keyboard), §7 (three-way alignment).

**Depends on:** Phase A. The face must already decode all 95 printable ASCII plus the `0x1F` mark.

**Repo:** paths relative to `/scratch/code/shibboleth/seedhammer` unless stated.

## Global Constraints

- Run everything in the devshell: `nix develop --command <cmd>` from the repo root. `go` is not on the host PATH.
- **`-update` on any golden is forbidden.** Phase B touches no engraving path, so *any* golden movement is a bug.
- Stage paths explicitly, never `git add -A`. `git commit -s`.
- **TinyGo target.** This code runs on the firmware. No `regexp`, no `fmt.Sprintf` in hot paths, no reflection. Keep allocations obvious.
- **The passphrase is secret material.** Validators must not log it, must not include it in error strings, and must not retain it. An error message may state *what* was wrong, never *the offending content*.

---

### Task 1: The `passphrase` package — `ValidatePassphrase`

**Files:**
- Create: `passphrase/passphrase.go`
- Create: `passphrase/passphrase_test.go`

**Interfaces:**
- Consumes: nothing.
- Produces: `func ValidatePassphrase(s string) error` and the exported sentinel errors below. Phase D's flow consumes both.

- [ ] **Step 1: Write the failing test**

```go
package passphrase

import (
	"errors"
	"strings"
	"testing"
)

func TestValidatePassphrase(t *testing.T) {
	long := strings.Repeat("a", 100)
	tests := []struct {
		name string
		in   string
		want error
	}{
		{"empty", "", ErrEmpty},
		{"single", "a", nil},
		{"exactly 100", long, nil},
		{"101 chars", long + "a", ErrTooLong},
		{"space is legal", "correct horse", nil},
		{"leading space", " x", nil},
		{"trailing space", "x ", nil},
		{"all printable ascii", allASCII(), nil},
		{"non-ascii accent", "café", ErrNonASCII},
		{"non-ascii cjk", "日本", ErrNonASCII},
		{"emoji", "a\U0001F600", ErrNonASCII},
		{"invalid utf8", "a\xff", ErrNonASCII},
		{"tab", "a\tb", ErrNonASCII},
		{"newline", "a\nb", ErrNonASCII},
		{"del", "a\x7f", ErrNonASCII},
		{"nul", "a\x00", ErrNonASCII},
	}
	for _, tc := range tests {
		got := ValidatePassphrase(tc.in)
		if !errors.Is(got, tc.want) {
			t.Errorf("%s: got %v, want %v", tc.name, got, tc.want)
		}
	}
}

// allASCII returns every printable ASCII rune once.
func allASCII() string {
	var b strings.Builder
	for r := rune(0x20); r <= 0x7E; r++ {
		b.WriteRune(r)
	}
	return b.String()
}

// An error must never quote the passphrase back.
func TestErrorsDoNotLeakContent(t *testing.T) {
	const secret = "hunter2é"
	err := ValidatePassphrase(secret)
	if err == nil {
		t.Fatal("expected an error")
	}
	if strings.Contains(err.Error(), "hunter2") {
		t.Errorf("error leaks passphrase content: %q", err.Error())
	}
}
```

- [ ] **Step 2: Run it and confirm it fails to compile**

Run: `nix develop --command go test ./passphrase/`
Expected: FAIL — `undefined: ErrEmpty` (and the rest).

- [ ] **Step 3: Implement**

```go
// Package passphrase validates BIP-39 passphrases for engraving.
//
// The device accepts only printable ASCII. That is NOT a BIP-39 rule -- the
// standard permits any string -- it is the boundary within which this
// codebase's derivation is provably conformant: bip39.MnemonicSeed performs no
// NFKD normalization, which is identity on ASCII and divergent otherwise. See
// SPEC_seedhammer_engrave_bip39_password.md D3.
package passphrase

import "errors"

// MaxLen is a plate-capacity limit chosen for legibility, not a BIP-39 rule.
const MaxLen = 100

var (
	ErrEmpty    = errors.New("a passphrase is required")
	ErrTooLong  = errors.New("too long for one plate")
	ErrNonASCII = errors.New("this device can only engrave printable ASCII")
)

// ValidatePassphrase reports whether s can be engraved. It never includes s in
// its error, because s is secret.
func ValidatePassphrase(s string) error {
	if s == "" {
		return ErrEmpty
	}
	n := 0
	for _, r := range s {
		// A malformed UTF-8 byte decodes to U+FFFD, which is > 0x7E and so is
		// rejected here rather than needing a separate check.
		if r < 0x20 || r > 0x7E {
			return ErrNonASCII
		}
		n++
	}
	if n > MaxLen {
		return ErrTooLong
	}
	return nil
}
```

- [ ] **Step 4: Run the tests**

Run: `nix develop --command go test ./passphrase/ -v`
Expected: PASS.

Note the ordering the tests pin: a 101-character string of ASCII returns
`ErrTooLong`, while a 5-character string containing `é` returns `ErrNonASCII`.
Charset is checked per rune during the count, so a long *and* non-ASCII string
reports non-ASCII. Either is defensible; the tests fix it so it cannot drift.

- [ ] **Step 5: Commit**

```bash
git add passphrase/passphrase.go passphrase/passphrase_test.go
git commit -s -m "feat(passphrase): ValidatePassphrase — printable ASCII, 1..100

The ASCII restriction is this device's limit, not BIP-39's. It is the boundary
within which derivation is provably conformant: bip39.MnemonicSeed performs no
NFKD normalization, which is identity on ASCII and divergent otherwise.

Errors never quote the passphrase back -- it is secret material, and an error
string is the easiest place to leak it."
```

---

### Task 2: `ValidateFingerprint` and canonicalisation

**Files:**
- Modify: `passphrase/passphrase.go`
- Modify: `passphrase/passphrase_test.go`

**Interfaces:**
- Consumes: Task 1's package.
- Produces: `func ValidateFingerprint(s string) (canonical string, err error)` plus `ErrBadFingerprint`. Phase C engraves the canonical form grouped 4-and-4; Phase D displays it.

- [ ] **Step 1: Write the failing test**

```go
func TestValidateFingerprint(t *testing.T) {
	tests := []struct {
		name, in, want string
		wantErr        error
	}{
		{"empty is allowed", "", "", nil},
		{"lowercase", "a1b2c3d4", "A1B2C3D4", nil},
		{"uppercase", "A1B2C3D4", "A1B2C3D4", nil},
		{"grouped 4-4", "A1B2 C3D4", "A1B2C3D4", nil},
		{"odd spacing", " a1 b2c3 d4 ", "A1B2C3D4", nil},
		{"too short", "A1B2C3D", "", ErrBadFingerprint},
		{"too long", "A1B2C3D4E", "", ErrBadFingerprint},
		{"non-hex", "A1B2C3DG", "", ErrBadFingerprint},
		{"non-ascii", "A1B2C3Dé", "", ErrBadFingerprint},
	}
	for _, tc := range tests {
		got, err := ValidateFingerprint(tc.in)
		if !errors.Is(err, tc.wantErr) {
			t.Errorf("%s: err %v, want %v", tc.name, err, tc.wantErr)
			continue
		}
		if got != tc.want {
			t.Errorf("%s: got %q, want %q", tc.name, got, tc.want)
		}
	}
}

// The canonical form is what is stored and compared; grouping is presentation.
func TestFingerprintCanonicalIsStable(t *testing.T) {
	forms := []string{"a1b2c3d4", "A1B2C3D4", "A1B2 C3D4", "a1b2 c3d4", " A1B2C3D4 "}
	want := "A1B2C3D4"
	for _, f := range forms {
		got, err := ValidateFingerprint(f)
		if err != nil {
			t.Fatalf("%q: %v", f, err)
		}
		if got != want {
			t.Errorf("%q canonicalised to %q, want %q", f, got, want)
		}
	}
}
```

- [ ] **Step 2: Run it and confirm it fails**

Run: `nix develop --command go test ./passphrase/ -run Fingerprint`
Expected: FAIL — `undefined: ValidateFingerprint`.

- [ ] **Step 3: Implement**

```go
// FingerprintLen is the canonical length: a BIP-32 master fingerprint is the
// first 4 bytes of RIPEMD160(SHA256(pubkey)), i.e. exactly 8 hex digits. This
// is the WHOLE fingerprint, not a truncation.
const FingerprintLen = 8

var ErrBadFingerprint = errors.New("fingerprint must be 8 hex digits")

// ValidateFingerprint accepts an empty string (the field is optional) or 8 hex
// digits with optional internal whitespace, and returns the canonical form:
// whitespace stripped, uppercased.
//
// The canonical form is the ONLY value stored or compared. The 4-and-4 grouping
// used on the plate and in the UI is presentation only -- see spec 4.3.
func ValidateFingerprint(s string) (string, error) {
	var buf [FingerprintLen]byte
	n := 0
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c == ' ' || c == '\t' {
			continue
		}
		switch {
		case c >= '0' && c <= '9':
		case c >= 'a' && c <= 'f':
			c -= 'a' - 'A'
		case c >= 'A' && c <= 'F':
		default:
			return "", ErrBadFingerprint
		}
		if n == FingerprintLen {
			return "", ErrBadFingerprint
		}
		buf[n] = c
		n++
	}
	if n == 0 {
		return "", nil
	}
	if n != FingerprintLen {
		return "", ErrBadFingerprint
	}
	return string(buf[:]), nil
}

// GroupFingerprint renders a canonical fingerprint for display and engraving,
// as "A1B2 C3D4". The separator is a plain space, NEVER the visible-space mark:
// the mark means "a literal space in the passphrase", and hex is 0-9A-F so a
// gap cannot be misread as a digit.
func GroupFingerprint(canonical string) string {
	if len(canonical) != FingerprintLen {
		return canonical
	}
	return canonical[:4] + " " + canonical[4:]
}
```

- [ ] **Step 4: Add the grouping test and run**

```go
func TestGroupFingerprint(t *testing.T) {
	if got := GroupFingerprint("A1B2C3D4"); got != "A1B2 C3D4" {
		t.Errorf("got %q, want %q", got, "A1B2 C3D4")
	}
	if got := GroupFingerprint(""); got != "" {
		t.Errorf("empty should pass through, got %q", got)
	}
}
```

Run: `nix develop --command go test ./passphrase/ -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add passphrase/passphrase.go passphrase/passphrase_test.go
git commit -s -m "feat(passphrase): ValidateFingerprint + canonical/grouped split

8 hex digits is the WHOLE BIP-32 fingerprint, not a truncation -- it is the
first 4 bytes of RIPEMD160(SHA256(pubkey)).

Canonical form (stripped, uppercased) is the only value stored or compared;
4-and-4 grouping is presentation. Stating that split explicitly because it is
where an off-by-one or a double-normalisation would hide. The separator is a
plain space, never the visible-space mark."
```

---

### Task 3: Extend the keyboard to all 32 ASCII symbols

Thirteen characters the spec promises cannot currently be typed: `% * < > [ \ ] ^ \` { | } ~`. Without this, D3's charset guarantee is hollow at the input stage.

**Files:**
- Modify: `gui/passphrase_keyboard.go`
- Modify: `gui/passphrase_keyboard_test.go`

**Interfaces:**
- Consumes: nothing.
- Produces: a 4-page keyboard whose union of pages is exactly the 95 printable ASCII.

- [ ] **Step 1: Write the failing test**

```go
// Every printable ASCII character the spec promises must be typeable.
func TestKeyboardCoversPrintableASCII(t *testing.T) {
	ctx := NewContext(newPlatform())
	k := NewPassphraseKeyboard(ctx)
	seen := map[rune]bool{}
	for p := range k.pages {
		for _, row := range k.pages[p] {
			for _, key := range row {
				if key.action == ppRune {
					seen[key.r] = true
				}
			}
		}
	}
	var missing []rune
	for r := rune(0x20); r <= 0x7E; r++ {
		if !seen[r] {
			missing = append(missing, r)
		}
	}
	if len(missing) > 0 {
		t.Errorf("keyboard cannot type %d of 95 printable ASCII: %q",
			len(missing), string(missing))
	}
}
```

- [ ] **Step 2: Run it and confirm it fails with exactly 13**

Run: `nix develop --command go test ./gui/ -run TestKeyboardCoversPrintableASCII -v`
Expected: FAIL listing 13 runes: `% * < > [ \ ] ^ \` { | } ~`

If the count is not 13, stop — §5.0's baseline is wrong.

- [ ] **Step 3: Add the fourth page**

**Six sites move in lockstep.** I verified this against the source rather than
counting the obvious ones — the two struct fields are easy to miss because they
are twenty lines from the constants:

| # | Site | Change |
|---|---|---|
| 1 | `ppPages` (`:24`) | `[3]string` → `[4]string` |
| 2 | `ppPageCycleLabel` (`:27`) | `[3]string` → `[4]string` |
| 3 | `PassphraseKeyboard.pages` (`:52`) | `[3][][]ppKey` → `[4][][]ppKey` |
| 4 | `PassphraseKeyboard.size` (`:53`) | `[3]image.Point` → `[4]image.Point` |
| 5 | build loop (`:70`) | `p < 3` → `p < len(ppPages)` |
| 6 | page cycle (`:199`) | `(k.page + 1) % 3` → `% len(ppPages)` |

Sites 1–4 fail the build if missed. **Sites 5 and 6 do not** — a stale `3` leaves
the fourth page built-but-unreachable, or unbuilt-but-cycled-to. That is what
Step 5's cycle test exists to catch.

```go
const (
	ppPageLower    = "qwertyuiop\nasdfghjkl\nzxcvbnm"
	ppPageUpper    = "QWERTYUIOP\nASDFGHJKL\nZXCVBNM"
	ppPageSymbols  = "1234567890\n-/:;()&$@\"\n.,?!'+=_#"
	// The 13 printable-ASCII symbols the first three pages omit.
	ppPageSymbols2 = "%*<>[]{}\n\\^`|~"
)

var ppPages = [4]string{ppPageLower, ppPageUpper, ppPageSymbols, ppPageSymbols2}

// ppPageCycleLabel[p] is the cap shown on page p (it names the NEXT page).
var ppPageCycleLabel = [4]string{"ABC", "?123", "#+=", "abc"}
```

Note `\\` is a single backslash and the backtick sits inside a double-quoted
string; neither can be written in a raw string literal.

- [ ] **Step 4: Run the coverage test and the existing keyboard tests**

Run: `nix develop --command go test ./gui/ -run 'Keyboard|Passphrase' -v`
Expected: PASS, 0 missing. Existing keyboard tests must still pass — the first
three pages are unchanged.

- [ ] **Step 5: Assert the cycle reaches every page**

```go
func TestKeyboardPageCycleVisitsAllPages(t *testing.T) {
	ctx := NewContext(newPlatform())
	k := NewPassphraseKeyboard(ctx)
	start := k.page
	seen := map[int]bool{start: true}
	for i := 0; i < len(ppPages)*2; i++ {
		k.commit(ppKey{action: ppPageCycle})
		seen[k.page] = true
	}
	if len(seen) != len(ppPages) {
		t.Errorf("cycle visited %d of %d pages", len(seen), len(ppPages))
	}
	if k.page != start {
		t.Errorf("after %d cycles page is %d, want %d", len(ppPages)*2, k.page, start)
	}
}
```

Run and expect PASS. This is what catches a `%3` left behind at one of the four sites.

- [ ] **Step 6: Commit**

```bash
git add gui/passphrase_keyboard.go gui/passphrase_keyboard_test.go
git commit -s -m "feat(gui): fourth keyboard page — all 32 ASCII symbols typeable

13 characters the spec promises could not be typed: % * < > [ \\ ] ^ \` { | } ~.
Extending the font without extending the keyboard left D3's charset guarantee
hollow at the input stage -- a user whose passphrase contains ~ simply could not
back it up.

Six sites move together (two arrays, two struct fields, the build loop bound and
the cycle modulus); the two that do NOT fail the build are the loop and the
modulus, so
the page-cycle test is what catches a %3 left behind at any of them."
```

---

### Task 4: The three-way alignment test

The load-bearing test of Phase B. Three definitions of "allowed" now exist — the validator, the engraving face, and the keyboard — and nothing yet stops them drifting apart. Drift is silent: a rune the validator accepts but the face cannot decode panics at engrave time; one the keyboard cannot type is simply unreachable.

**Files:**
- Create: `passphrase/alignment_test.go`

**Interfaces:**
- Consumes: Tasks 1–3.
- Produces: nothing; it is a guard.

- [ ] **Step 1: Write the test**

```go
package passphrase_test

import (
	"testing"

	"seedhammer.com/font/constant"
	"seedhammer.com/passphrase"
)

// For EVERY rune ValidatePassphrase accepts, the engraving face must be able to
// decode it. A rune that validates but cannot be decoded panics at engrave time
// -- mid-plate, on a permanent medium.
//
// (The keyboard half of the three-way check lives in gui, which cannot be
// imported here without a cycle; see gui.TestKeyboardCoversPrintableASCII.)
func TestValidatorAgreesWithFace(t *testing.T) {
	for r := rune(0x20); r <= 0x7E; r++ {
		accepted := passphrase.ValidatePassphrase(string(r)) == nil
		_, _, decodable := constant.Font.Decode(r)
		if accepted != decodable {
			t.Errorf("rune %q: validator accepts=%v, face decodes=%v", r, accepted, decodable)
		}
	}
}

// And nothing outside printable ASCII is accepted, so the loop above is the
// whole domain.
func TestValidatorRejectsEverythingElse(t *testing.T) {
	for _, r := range []rune{0x00, 0x09, 0x0A, 0x1F, 0x7F, 0x80, 0xE9, 0x65E5} {
		if err := passphrase.ValidatePassphrase(string(r)); err == nil {
			t.Errorf("rune %#x was accepted", r)
		}
	}
	// 0x1F in particular: it is the visible-space mark's codepoint. A validated
	// passphrase must never contain it, or it would collide with the
	// substitution in spec 3.3.
}
```

- [ ] **Step 2: Run it**

Run: `nix develop --command go test ./passphrase/ -v`
Expected: PASS. If it fails, either Phase A's font work regressed or the
validator's bounds are wrong — investigate, do not adjust the test to match.

- [ ] **Step 3: Run the whole suite**

Run: `nix develop --command go test ./...`
Expected: all green, **no golden updated**. Phase B touches no engraving path, so
any golden movement is a bug.

- [ ] **Step 4: Commit**

```bash
git add passphrase/alignment_test.go
git commit -s -m "test(passphrase): pin validator/face agreement over all printable ASCII

Three definitions of 'allowed' now exist -- validator, engraving face, keyboard
-- and drift between them is silent. A rune the validator accepts but the face
cannot decode panics at ENGRAVE time, mid-plate. This asserts the first two
agree across the whole domain; gui.TestKeyboardCoversPrintableASCII covers the
third (a package cycle prevents doing all three in one place).

Also pins that 0x1F is rejected: it is the visible-space mark's codepoint, and a
validated passphrase containing it would collide with the spec 3.3 substitution."
```

---

## Phase B exit criteria

- [ ] `ValidatePassphrase` and `ValidateFingerprint` implemented, with the ordering between `ErrTooLong` and `ErrNonASCII` pinned by test.
- [ ] Errors never contain the passphrase.
- [ ] Keyboard types all 95 printable ASCII; page cycle reaches every page and returns to start.
- [ ] Validator and face agree across the whole domain; `0x1F` rejected.
- [ ] `nix develop --command go test ./...` green **with no `-update`**.
- [ ] Mandatory post-implementation adversarial review over the whole Phase B diff (risk-set work) — **non-deferrable**.

## Not in Phase B

`backup.Passphrase`, the two plate layouts, metadata bands and legend (**Phase C**); the flow, warnings, confirm-screen space rendering, `[]byte` wiping and menu wiring (**Phase D**). Phase B deliberately produces no plate output and no GUI flow, so it can be reviewed on its own.
