# BIP-39 Password — Phase C: The Plate — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn a validated passphrase into engraving commands — both layouts, the metadata bands, the legend — with the space substitution and the QR proven byte-exact.

**Architecture:** A new `backup.Passphrase` plate type and `EngravePassphrase`, modelled on the existing `engraveSeedString` but deliberately a separate type (spec D6): `SeedString` calls `strings.ToUpper` on its content, and one wrong argument would destroy a passphrase. No GUI — Phase C produces engraving commands and is testable through goldens and layout assertions alone.

**Tech Stack:** Go 1.25 (TinyGo for firmware), Nix devshell.

**Spec:** `design/SPEC_seedhammer_engrave_bip39_password.md` — R0 GREEN, amendment GREEN. Relevant: §3.3 (space mark), §4 (layouts, bands, legend), §7 (tests).

**Depends on:**
- **Phase A Task 4** — `NewPassphraseStringer` with per-run quantization. Phase C cannot be finished without it.
- **Phase B** — `passphrase.ValidateFingerprint` and `passphrase.GroupFingerprint`.

**Repo:** paths relative to `/scratch/code/shibboleth/seedhammer` unless stated.

## Global Constraints

- Devshell only: `nix develop --command <cmd>` from the repo root.
- **`-update` is FORBIDDEN on any *existing* golden.** Phase C adds new goldens for the new plate type; those may be created. Any movement in `seed-*`, `slip39-*`, `codex32-*` or `text-*` is a bug — STOP and report.
- **Use `NewPassphraseStringer`, never `NewConstantStringer`.** The shared instance's alphabet is 36 characters and panics on lowercase (spec §3.5.1).
- **Call `String`, never `PaddedString` with `shortest != longest`.** Phase A adds a `hasMultiRun` guard that panics; `String` is `PaddedString(yield, txt, n, n)` and is fine.
- **Never `strings.ToUpper` the passphrase.** That is the whole reason this is a separate type from `SeedString`.
- Stage paths explicitly. `git commit -s`.

---

### Task 1: `backup.Passphrase` and the no-QR layout (§4.1)

**Files:**
- Create: `backup/passphrase.go`
- Create: `backup/passphrase_test.go`

**Interfaces:**
- Consumes: `engrave.NewPassphraseStringer` (Phase A Task 4).
- Produces: `type Passphrase struct{...}`, `func EngravePassphrase(params engrave.Params, plate Passphrase) (engrave.Engraving, error)`, and `const SpaceMark = '\x1f'`. Phase D constructs the struct.

- [ ] **Step 1: Write the failing test — case preservation and the space mark**

```go
package backup

import (
	"strings"
	"testing"

	"seedhammer.com/engrave"
	"seedhammer.com/font/constant"
)

// The passphrase must be engraved verbatim. SeedString uppercases its content;
// doing that here would silently destroy the secret.
func TestPassphrasePreservesCase(t *testing.T) {
	got := passphraseGlyphs("Hunter2")
	if got != "Hunter2" {
		t.Errorf("case not preserved: %q", got)
	}
}

// Every space becomes the visible mark; nothing else changes.
func TestPassphraseSpaceSubstitution(t *testing.T) {
	tests := []struct{ in, want string }{
		{"ab", "ab"},
		{"a b", "a\x1fb"},
		{"a  b", "a\x1f\x1fb"},
		{" ab", "\x1fab"},
		{"ab ", "ab\x1f"},
		{"  ", "\x1f\x1f"},
	}
	for _, tc := range tests {
		if got := passphraseGlyphs(tc.in); got != tc.want {
			t.Errorf("%q -> %q, want %q", tc.in, got, tc.want)
		}
	}
}

// The engraved stream must contain no literal 0x20. A real space would be
// invisible on metal, which is the entire reason the mark exists.
func TestPassphraseEngravesNoLiteralSpace(t *testing.T) {
	if strings.ContainsRune(passphraseGlyphs("a b c"), ' ') {
		t.Error("engraved text contains a literal 0x20")
	}
}
```

- [ ] **Step 2: Run and confirm it fails**

Run: `nix develop --command go test ./backup/ -run Passphrase`
Expected: FAIL — `undefined: passphraseGlyphs`.

- [ ] **Step 3: Implement the type and the substitution**

```go
// SpaceMark is the codepoint of the visible-space glyph. A space is invisible
// on metal -- one space and two look identical, and leading or trailing spaces
// cannot be seen at all -- while "hunter2 " is a different wallet from
// "hunter2". Every space is therefore engraved as this mark (spec 3.3).
//
// It lives at a control codepoint, NOT at 0x20: TitleString and EngraveText
// paragraphs already engrave literal spaces, so remapping 0x20 would silently
// change existing plate types.
const SpaceMark = '\x1f'

type Passphrase struct {
	// Passphrase is engraved VERBATIM. Never uppercase it.
	Passphrase string
	// SeedFP and CombinedFP are canonical 8-hex-digit fingerprints, or empty.
	// Both are user-typed and unverified (spec D1).
	SeedFP     string
	CombinedFP string
	// QR includes a machine-readable copy of the passphrase. Opt-in (spec D8).
	QR   bool
	Font *vector.Face
}

// passphraseGlyphs maps the passphrase to the glyph sequence that gets
// engraved: every space becomes SpaceMark, everything else is verbatim.
func passphraseGlyphs(s string) string {
	if !strings.ContainsRune(s, ' ') {
		return s
	}
	var b strings.Builder
	b.Grow(len(s))
	for _, r := range s {
		if r == ' ' {
			r = SpaceMark
		}
		b.WriteRune(r)
	}
	return b.String()
}
```

- [ ] **Step 4: Run the tests**

Run: `nix develop --command go test ./backup/ -run Passphrase -v`
Expected: PASS.

- [ ] **Step 5: Add the no-QR layout**

Follow `engraveSeedString` (`backup/backup.go:99`) as the model, with these differences, each of which matters:

- `engrave.NewPassphraseStringer`, not `NewConstantStringer`
- **no `strings.ToUpper`**
- `rowLen = 10`, `pfs = params.F(passphraseFontSize)` where `passphraseFontSize = 6.0` (spec §4.1 — 10 rows fits ~60 mm of the ~65 mm usable, giving lowercase x-height ≈ 9 stroke widths)
- rows come from `passphraseGlyphs(plate.Passphrase)`, not the raw string

Reuse `stringColumn` — it already issues one `String` call per `groupLen` group, which is exactly what §3.5.0's disclosure is stated against.

- [ ] **Step 6: Assert the layout fits**

```go
func TestPassphraseLayoutFitsNoQR(t *testing.T) {
	p := Passphrase{Passphrase: strings.Repeat("a", 100), Font: constant.Font}
	eng, err := EngravePassphrase(params(), p)
	if err != nil {
		t.Fatal(err)
	}
	bounds := measure(t, eng) // existing helper; match backup_test.go's idiom
	if bounds.Max.X > params().F(85) || bounds.Max.Y > params().F(85) {
		t.Errorf("100-char plate exceeds 85x85mm: %v", bounds)
	}
}
```

Read `backup/backup_test.go` for the existing measurement idiom and match it rather than inventing one.

- [ ] **Step 7: Commit**

```bash
git add backup/passphrase.go backup/passphrase_test.go
git commit -s -m "feat(backup): Passphrase plate type and the no-QR layout

A separate type from SeedString, deliberately: SeedString calls strings.ToUpper
on its content, and one wrong argument would silently destroy a passphrase.
Separate types make that unrepresentable.

Every space becomes the visible mark at 0x1f. A space is invisible on metal --
one space and two look identical, leading and trailing ones cannot be seen at
all -- and 'hunter2 ' is a different wallet from 'hunter2'."
```

---

### Task 2: The QR layout (§4.2), and prove the QR is byte-exact

**Files:**
- Modify: `backup/passphrase.go`, `backup/passphrase_test.go`

**Interfaces:**
- Consumes: Task 1.
- Produces: the `plate.QR == true` path.

- [ ] **Step 1: Write the byte-exactness test first — it is the highest-leverage test in this phase**

```go
// Two variants of the secret are in flight: the RAW string (QR, confirm screen)
// and the MARK-TRANSLATED one (engraver). Swapping them either engraves
// invisible real spaces or QR-encodes the control codepoint, which a scanner
// hands to a wallet as different bytes. Silent either way.
func TestPassphraseQRIsByteExact(t *testing.T) {
	cases := []string{
		"hunter2",
		"correct horse battery staple",
		" leading",
		"trailing ",
		"double  space",
		allPrintableASCII(),
		strings.Repeat("a", 100),
	}
	for _, in := range cases {
		code := passphraseQR(t, in)          // build via the real path
		out := decodeQR(t, code)             // decode back
		if out != in {
			t.Errorf("QR round-trip changed the passphrase:\n in: %q\nout: %q", in, out)
		}
		if strings.ContainsRune(out, SpaceMark) {
			t.Errorf("QR encoded the visible-space mark instead of a real space: %q", out)
		}
	}
}
```

The QR carries the passphrase **exactly as entered** — real `0x20`, never the mark (spec §4.2). The mark is a rendering device for the text block only.

- [ ] **Step 2: Run, confirm it fails, then implement the QR path**

Encode `plate.Passphrase` (raw, untranslated) at **ECC-L**, and engrave via `engrave.ConstantQR` — never `engrave.QR`, which is content-timing-dependent and would leak a secret (spec §3.5.2).

Layout per §4.2: **5 rows × 20 characters**, `passphraseFontSizeQR = 4.5`, QR beneath, centred in a **reserved 37-module envelope** (QR size is variable 33/37 — do not assume 37).

- [ ] **Step 3: Assert the QR layout fits and the size varies**

```go
func TestPassphraseQRLayoutFits(t *testing.T) { /* 100 chars + QR within 85x85 */ }

func TestPassphraseQRSizeVariable(t *testing.T) {
	// alphanumeric-subset content encodes at dim 33, byte-mode at 37.
	// Both must lay out correctly; the envelope is reserved for 37.
}
```

- [ ] **Step 4: Commit**

---

### Task 3: Metadata bands, legend and the worst-case fit

**Files:** modify `backup/passphrase.go`, `backup/passphrase_test.go`

- [ ] **Step 1: Write the worst-case fit test FIRST**

The partial case is what let the metadata overflow go unnoticed twice during specification. Test the full one:

```go
func TestPassphrasePlateWorstCase(t *testing.T) {
	p := Passphrase{
		Passphrase: strings.Repeat("a b", 33) + "a", // 100 chars, spaces -> legend required
		SeedFP:     "A1B2C3D4",
		CombinedFP: "5E6F7A8B",
		QR:         true,
		Font:       constant.Font,
	}
	// assert: nothing exceeds the usable area; <= 2 lines per margin band;
	// no metadata line wider than 64mm; blocks do not overlap; nothing lands
	// in a corner screw-hole band.
}
```

- [ ] **Step 2: Implement the bands per §4.3**

| Band | Contents | Lines |
|---|---|---|
| Top (10 mm) | `SEED FP:` , `EXPECTED COMB FP:` | ≤ 2 |
| Bottom (10 mm) | `<mark> = SPACE` legend, `FINGERPRINTS TYPED, NOT VERIFIED` | ≤ 2 |

Normative and test-asserted: **≤2 lines per band** (a band offers `innerMargin` 10 − `outerMargin` 3 = **7 mm**; three 3 mm lines need 9 and run off the plate) and **no metadata line wider than 64 mm** (the longest, `FINGERPRINTS TYPED, NOT VERIFIED`, is 32 chars × 2.0 mm = 64 mm and clears the corner bands by 0.5 mm).

Fingerprint lines are **omitted entirely when blank**, and rendered grouped 4-and-4 via `passphrase.GroupFingerprint` — separator a **plain space, never the mark**.

The legend is required **whenever the passphrase contains a space**, engraved with the real mark glyph so the reader matches shapes, not descriptions.

- [ ] **Step 3: Space-fidelity and legend-presence tests**

```go
func TestLegendPresence(t *testing.T) {
	// legend emitted iff the passphrase contains at least one space
}
func TestSpaceFidelity(t *testing.T) {
	// leading / trailing / interior / repeated spaces each produce the
	// corresponding count of marks in the engraved stream
}
```

- [ ] **Step 4: Commit**

---

### Task 4: Goldens and existing-output invariance

**Files:** `backup/passphrase_test.go`, `backup/testdata/passphrase-*.bin`

- [ ] **Step 1: Add goldens for both layouts** using `golden.CompareBSpline`, matching `backup_test.go`'s idiom. Creating *new* goldens is allowed; `-update` on existing ones is not.
- [ ] **Step 2: Prove existing output is unaffected**

```bash
nix develop --command go test -count=1 ./... 2>&1 | grep -v "no test files"
git diff --stat main..HEAD -- backup/testdata engrave/testdata gui/testdata
```
Every pre-existing golden must pass unmodified. Only `passphrase-*.bin` may be new.

- [ ] **Step 3: Commit**

---

## Phase C exit criteria

- [ ] Case preserved; no literal `0x20` in any engraved stream.
- [ ] QR round-trips **byte-exactly**, with real spaces and no mark.
- [ ] Both layouts fit 85×85 mm at 100 characters, worst case including QR + both fingerprints + legend + footer.
- [ ] ≤2 lines per margin band; no metadata line wider than 64 mm.
- [ ] Legend emitted iff a space is present.
- [ ] Fingerprints grouped 4-and-4 with a plain-space separator.
- [ ] Existing goldens byte-identical; new `passphrase-*` goldens added.
- [ ] Full suite green with no `-update` on an existing golden.
- [ ] Mandatory post-implementation adversarial review over the whole Phase C diff — **non-deferrable**.

**Still open after Phase C:** O1 (hardware legibility — lowercase has never been cut on this machine, and no amount of green tests substitutes) and O4 (final legend/footer wording measured at 3 mm).

## Not in Phase C

The GUI flow, warnings, confirm-screen space rendering, `[]byte` secret wiping, and menu wiring — all **Phase D**.
