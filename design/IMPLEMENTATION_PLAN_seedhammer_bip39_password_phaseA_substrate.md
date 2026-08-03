# BIP-39 Password — Phase A: Engraving Substrate — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the SeedHammer II able to engrave any printable-ASCII string, constant-time, without changing a single byte of existing plate output.

**Architecture:** Extend `font/constant` with 44 new glyphs at codepoints that currently have none, teach `cmd/vectorfont` to address a control codepoint for the visible-space mark, and add a **separate** passphrase alphabet plus its own `ConstantStringer` instance — never widening the shared one. Separately, determine whether `ConstantQR` can reach version 6 by deriving module maxima from the fuzz corpus.

**Tech Stack:** Go 1.25 (TinyGo for firmware), Nix devshell (`nix develop`), single-stroke SVG font source compiled by `go:generate`.

**Spec:** `design/SPEC_seedhammer_engrave_bip39_password.md` — **R0 GREEN (0C/0I)**, three rounds, reports in `design/agent-reports/seedhammer-engrave-bip39-password-spec-R0-round{0,1,2}.md`.

**Repo:** all paths are relative to `/scratch/code/shibboleth/seedhammer` unless stated.

## Global Constraints

- **Run everything inside the devshell:** `cd /scratch/code/shibboleth/seedhammer && nix develop --command <cmd>`. `go`, `tinygo`, `picotool` are not on the host PATH.
- **Existing plate output must not change.** Goldens for seed, SLIP-39, codex32 and text plates stay byte-identical. `-update` is **forbidden** in this phase — if a golden changes, that is a finding, not a chore.
- **The shared `constantAlphabet` (`engrave/engrave.go:750`) MUST NOT be modified.** Spec §3.5.1. Widening it moves `center` and `startEndDist` for every constant-time string on the machine.
- **No new glyph may begin at (0,0).** `paddedString` uses `inf.Start != (bezier.Point{})` as a sentinel (`engrave/engrave.go:1294-1296`).
- **Every glyph keeps advance 6 units.** `NewConstantStringer` panics `"variable width font"` on any variance (`engrave/engrave.go:1216-1218`).
- **Any alphabet string must be in ascending codepoint order.** `NewConstantStringer` panics `"unsorted alphabet"` (`engrave/engrave.go:1208-1210`), because lookup is a `sort.Find` binary search.
- Font metrics stay `height=9`, `baseline=8`, `advance=6` (spec D5). Do not touch the `height`/`advance`/`baseline` markers in `constant.svg`.
- Commits: stage paths explicitly, never `git add -A`. Sign off (`git commit -s`) per fork convention.

## Staging note — why glyphs and alphabet can be separate commits here

Spec §3.5.1.1 requires glyphs and alphabet to land together *when the shared alphabet is touched*, because `NewConstantStringer` validates its alphabet against the face at construction and the shared instance is built for seed plates. Phase A adds a **separate** alphabet whose `ConstantStringer` nothing constructs until Phase B. Task 3 (glyphs) may therefore precede Task 4 (alphabet) safely. This relaxation is a direct dividend of the round-1 I3 decision.

---

### Task 1: Lock in the current font coverage as a regression test

Establishes the baseline the whole phase is measured against, and turns the ad-hoc probe used during specification into a permanent test.

**Files:**
- Create: `font/constant/coverage_test.go`

**Interfaces:**
- Consumes: nothing.
- Produces: `TestPrintableASCIICoverage` — the test Task 3 must flip from failing to passing.

- [ ] **Step 1: Write the test asserting FULL printable-ASCII coverage**

```go
package constant

import "testing"

// TestPrintableASCIICoverage asserts the engraving face can render every
// printable ASCII character. A BIP-39 passphrase is case-sensitive free text,
// so anything the face cannot decode either panics at engrave time
// (engrave/engrave.go:1365) or forces a refusal at entry.
func TestPrintableASCIICoverage(t *testing.T) {
	var missing []rune
	for r := rune(0x20); r <= 0x7E; r++ {
		if _, _, ok := Font.Decode(r); !ok {
			missing = append(missing, r)
		}
	}
	if len(missing) > 0 {
		t.Errorf("face cannot decode %d of 95 printable ASCII runes: %q",
			len(missing), string(missing))
	}
}
```

- [ ] **Step 2: Run it and confirm it FAILS with exactly the expected 43**

Run: `nix develop --command go test ./font/constant/ -run TestPrintableASCIICoverage -v`

Expected: FAIL, reporting 43 missing runes: `! " $ % & + ; < = > ? \ ^ _ ` a-z | ~`

If the count is not 43, stop — the spec's §3.1 baseline is wrong and Phase A rests on it.

- [ ] **Step 3: Mark the test as a known-failing baseline**

Add `t.Skip` guarded by an env var so CI stays green until Task 3 lands:

```go
	if os.Getenv("SH_FONT_COVERAGE_STRICT") == "" && len(missing) == 43 {
		t.Skipf("known baseline: %d runes missing, pending glyph authoring (Task 3)", len(missing))
	}
```

Import `"os"`. Task 3's final step deletes this skip.

- [ ] **Step 4: Verify it now skips rather than fails**

Run: `nix develop --command go test ./font/constant/ -v`
Expected: SKIP with the baseline message. Then run with `SH_FONT_COVERAGE_STRICT=1` and confirm it still FAILS.

- [ ] **Step 5: Commit**

```bash
git add font/constant/coverage_test.go
git commit -s -m "test(font): assert full printable-ASCII coverage, skipped at the known 43-rune baseline

A BIP-39 passphrase is case-sensitive free text; the engraving face currently
decodes 52 of 95 printable ASCII with zero lowercase. This test is the gate the
glyph authoring must flip. Skips at exactly the 43-rune baseline so CI stays
green; SH_FONT_COVERAGE_STRICT=1 forces the real assertion."
```

---

### Task 2: Teach `cmd/vectorfont` to address a control codepoint

The visible-space mark lives at a control codepoint so that `0x20` stays a blank advance and existing titles/paragraphs are untouched (spec §3.3). The generator cannot currently name one.

**Files:**
- Modify: `cmd/vectorfont/main.go` — `mapChar`, around `:704-771`
- Create: `cmd/vectorfont/mapchar_test.go`

**Interfaces:**
- Consumes: nothing.
- Produces: SVG id `space_mark` maps to rune `0x1F`. Task 3 relies on this id.

- [ ] **Step 1: Read the existing name table**

Run: `nix develop --command sed -n '704,772p' cmd/vectorfont/main.go`

Signature is `func mapChar(id string) (rune, bool)`. Single-character ids map to
themselves; named ids go through an inner `switch` that assigns `r` and falls
through to a shared `return r, true`, with `default: return 0, false`. Note that
`lt` (`<`) and `gt` (`>`) are already **named** here even though the SVG has no
glyph for them — the name table and the glyph set are independent.

- [ ] **Step 2: Write the failing test**

```go
package main

import "testing"

func TestMapCharSpaceMark(t *testing.T) {
	got, ok := mapChar("space_mark")
	if !ok {
		t.Fatal("space_mark not recognised")
	}
	if got != 0x1F {
		t.Errorf("space_mark = %#x, want 0x1F", got)
	}
	// 0x1F must be below 0x20 so a validated ASCII passphrase can never
	// contain it, and so it sorts FIRST in an ascending alphabet.
	if got >= 0x20 {
		t.Errorf("space_mark %#x must be a control codepoint", got)
	}
}
```

- [ ] **Step 3: Run it and confirm it fails**

Run: `nix develop --command go test ./cmd/vectorfont/ -run TestMapCharSpaceMark -v`
Expected: FAIL — `space_mark not recognised`.

- [ ] **Step 4: Add the case**

In `mapChar`'s named-glyph switch, alongside `case "apostrophe":` and friends:

```go
		case "space_mark":
			// Visible-space mark. Lives at a control codepoint so 0x20 stays a
			// blank advance -- backup.TitleString and EngraveText paragraphs
			// already engrave literal spaces, and remapping 0x20 would silently
			// change existing plates (spec 3.3).
			r = 0x1F
```

Assign `r` rather than returning directly — the inner switch falls through to the
shared `return r, true` at `:771`.

- [ ] **Step 5: Run the test and the full generator suite**

Run: `nix develop --command go test ./cmd/vectorfont/ -v`
Expected: PASS.

- [ ] **Step 6: Confirm the font still regenerates byte-identically**

```bash
cd font/constant && cp constant.bin /tmp/constant.bin.orig
nix develop --command go generate ./...
cmp constant.bin /tmp/constant.bin.orig && echo "IDENTICAL"
```
Expected: `IDENTICAL` — no glyph has been added yet, so the binary must not move.

- [ ] **Step 7: Commit**

```bash
git add cmd/vectorfont/main.go cmd/vectorfont/mapchar_test.go
git commit -s -m "feat(vectorfont): map SVG id space_mark to control codepoint 0x1F

The passphrase plate engraves spaces as a visible mark. It cannot live at 0x20:
TitleString and EngraveText already engrave literal spaces, so remapping 0x20
would silently change existing plate output. 0x1F is free (the face indexes
0..126), sorts first in an ascending alphabet, and can never appear in a
validated ASCII passphrase. Font binary regenerates byte-identically."
```

---

### Task 3: Author the 44 glyphs

**This is craft work, not mechanical transcription.** It is the one task in this plan whose output cannot be fully specified in advance — glyph shapes must be drawn, rendered, looked at, and adjusted. What follows is the harness, the house style, the acceptance criteria, and worked examples; the coordinates themselves come out of the loop.

**Files:**
- Modify: `font/constant/constant.svg`
- Modify: `font/constant/constant.bin`, `font/constant/constant.go` (regenerated, do not hand-edit)
- Modify: `font/constant/coverage_test.go` (remove the skip)
- Create: `font/constant/glyph_rules_test.go`

**Interfaces:**
- Consumes: `space_mark` → `0x1F` from Task 2.
- Produces: a face decoding all 95 printable ASCII plus `0x1F`, every glyph advance 600, none starting at the origin.

#### House style (read before drawing)

The source is `font/constant/constant.svg`, `viewBox="0 0 306 9"` — glyphs laid out left to right on a 9-unit-tall grid, each as `<polyline>` (or `<g>` of `<line>`) with `fill:none`, i.e. **centerlines**, not outlines. Metrics: `height=9`, `baseline=8`, `advance=6`.

Existing uppercase occupies **x-width 4 with 1 unit side bearing, y2→y8** (cap height 6):

```xml
<polyline id="A" class="st0" points="1,5 5,5 5,8 5,3 4,2 2,2 1,3 1,8 "/>
```

Note the pen never lifts — a single polyline is one continuous stroke, and doubling back (`5,5 5,8 5,3`) is how the existing font draws a crossbar without a second path. Match that idiom.

For the new glyphs:
- **Lowercase x-height: y4 → y8** (4 units).
- **Ascenders** (`b d f h k l t`): up to y2, matching cap height.
- **Descenders** (`g j p q y`): down to **y9 only** — 1 unit. This is D5, and it is deliberate.
- **Symbols:** follow the vertical extent of the nearest existing symbol; `#` spans y2→y8, `,` and `.` sit at the baseline.

#### Reference letterforms — do not start from a blank page

Single-stroke lowercase is a solved design problem. The **Hershey fonts**
(Dr. A. V. Hershey, US Naval Weapons Laboratory, c. 1967) are public-domain
vector fonts built for plotters and engravers, with full lowercase and symbol
coverage, and they are available as SVG through the Inkscape *Hershey Text*
extension and several SVG/JSON repackagings. Hershey Sans / Simplex is the
closest in spirit to `constant`.

**Use them as a visual reference and redraw in house style — do not import
coordinate data.** Three reasons, in order of importance:

1. **Monospace.** Hershey is proportional. This font requires a single uniform
   advance (`engrave.go:1216-1218`), and the plate's "position implies index"
   property (spec §4) depends on it. Every glyph would need re-spacing into a
   6-unit cell regardless.
2. **Metrics.** Hershey uses its own coordinate space; everything needs rescaling
   to em 9 / cap 6 / x-height 4 / baseline y8.
3. **Provenance.** This repository is released under the **Unlicense** — public
   domain. The original Hershey data is public domain, but common redistributions
   attach a "credit the author, do not sell the data" note. Redrawing from visual
   reference avoids inheriting any such term into an Unlicense codebase. **Do not
   copy glyph coordinates from a source whose licence is anything other than
   public domain** — SIL OFL fonts in particular are incompatible, since OFL
   requires derivatives stay OFL.

If any reference material is consulted closely enough that redrawing feels like
transcription, stop and record what was used, so the licence question can be
settled deliberately rather than by accident.

#### Acceptance criteria (all enforced by tests in this task)

1. All 95 printable ASCII plus `0x1F` decode.
2. Every glyph advance is exactly 600 (post-`-scale 100`).
3. No glyph's first engraved point is (0,0).
4. Confusable pairs are visually distinct — checked by eye, see below.

#### Confusable pairs — the requirement that makes this hard

The plate is case-sensitive free text with **no checksum**, unlike wordlist-checked seeds or bech32 strings. A misread character silently opens a different wallet.

- **Cross-character:** `l`/`1`/`I`, `0`/`O`/`o`, `'`/`` ` ``, `;`/`:`, `,`/`.`, `8`/`B`, `5`/`S`, `2`/`Z`, `9`/`g`, `u`/`v`, `rn`/`m`
- **Case-only — the class this feature creates:** `C/c O/o S/s U/u V/v W/w X/x Z/z K/k`. These have the *same stroke path* at different heights. Under D5's fixed metrics they are same-shape-by-construction unless deliberately differentiated.

Suggested conventions (author's discretion, but pick and apply consistently): slashed or dotted zero; serifed `I`; flagged `1`; based `l`; and for the case-only class, give the lowercase a distinguishing feature rather than relying on size — e.g. single-storey `a`, curled-tail `u`, or a shortened crossbar.

- [ ] **Step 1: Set up the render-and-inspect loop**

The generator can round-trip to SVG for visual checking:

```bash
cd /scratch/code/shibboleth/seedhammer
nix develop --command go run seedhammer.com/cmd/vectorfont \
  -package constant -scale 100 -dump /tmp/font-dump.svg \
  font/constant/constant.svg constant
```

Open `/tmp/font-dump.svg`. This is the loop: edit `constant.svg` → regenerate → look → adjust. Run it once now against the unmodified font to confirm the loop works before drawing anything.

- [ ] **Step 2: Write the mechanical glyph-rule tests first**

```go
package constant

import (
	"testing"

	"seedhammer.com/bezier"
)

// runes iterates the space mark plus every printable ASCII rune.
func runes(yield func(rune) bool) {
	if !yield(0x1F) {
		return
	}
	for r := rune(0x20); r <= 0x7E; r++ {
		if !yield(r) {
			return
		}
	}
}

// Every glyph must share one advance -- NewConstantStringer panics
// "variable width font" otherwise (engrave/engrave.go:1216-1218), and the
// passphrase plate's "position implies index" property depends on it.
func TestUniformAdvance(t *testing.T) {
	const want = 600
	for r := range runes {
		adv, _, ok := Font.Decode(r)
		if !ok {
			continue // coverage_test.go reports missing glyphs
		}
		if adv != want {
			t.Errorf("advance(%q) = %d, want %d", r, adv, want)
		}
	}
}

// paddedString uses inf.Start != (bezier.Point{}) as its sentinel for "this
// glyph has a leading move segment" (engrave/engrave.go:1294-1296). A glyph
// whose first control point is exactly the origin takes the wrong branch.
// Plausible for '_', which naturally starts at the lower left.
func TestNoGlyphStartsAtOrigin(t *testing.T) {
	for r := range runes {
		_, spline, ok := Font.Decode(r)
		if !ok {
			continue
		}
		k, ok := spline.Next() // vector.UniformBSpline.Next() (Knot, bool)
		if !ok {
			continue // no knots (space is a blank advance)
		}
		if k.Ctrl == (bezier.Point{}) {
			t.Errorf("glyph %q starts at the origin; paddedString's sentinel will misfire", r)
		}
	}
}
```

`Font.Decode` returns `(advance int, spline vector.UniformBSpline, ok bool)`; `spline.Next()` returns `(vector.Knot, bool)` where `Knot` has fields `Line bool` and `Ctrl bezier.Point` (`font/vector/font.go:34-47`, `:72-85`). `Decode` is a value receiver on the spline, so iterating does not disturb the face.

- [ ] **Step 3: Run the rule tests against the current font**

Run: `nix develop --command go test ./font/constant/ -run 'TestUniformAdvance|TestNoGlyphStartsAtOrigin' -v`
Expected: PASS (52 existing glyphs already satisfy both). This proves the tests work before they have new glyphs to judge.

- [ ] **Step 4: Author the 26 lowercase glyphs**

Append to `constant.svg` before the metrics markers (`height`/`advance`/`baseline` at the end). Place each glyph at the next free x-slot and widen the `viewBox` width accordingly.

Two worked examples in house style, as **starting points to refine in the loop** — not final coordinates:

```xml
<!-- o : x-height bowl, y4-y8. Octagonal to keep stroke count low. -->
<polyline id="o" class="st0" points="289,5 290,4 292,4 293,5 293,7 292,8 290,8 289,7 289,5 "/>

<!-- p : descender to y9 (1 unit, per D5), bowl y4-y7. -->
<polyline id="p" class="st0" points="295,9 295,4 298,4 299,5 299,6 298,7 295,7 "/>
```

Regenerate and inspect after every few glyphs rather than all 26 at once.

- [ ] **Step 5: Author the 17 symbols**

`! " $ % & + ; < = > ? \ ^ _ ` | ~`

`_` is the origin trap from Step 2 — it sits at the baseline and its natural start is the far left. Keep its x ≥ 1.

- [ ] **Step 6: Author the visible-space mark**

```xml
<polyline id="space_mark" class="st0" points="..."/>
```

Requirements: must resemble **no** real character (spec O3), must be unmistakable at ~2 mm x-height, and must read as "something is here" rather than as punctuation. A open-bottomed bracket (`⎵`) is the conventional choice and does not collide with `_` if drawn with visible risers.

- [ ] **Step 7: Regenerate and run the whole font suite**

```bash
nix develop --command go generate ./font/constant/...
SH_FONT_COVERAGE_STRICT=1 nix develop --command go test ./font/constant/ -v
```
Expected: `TestPrintableASCIICoverage` now reports 0 missing; `TestUniformAdvance` and `TestNoGlyphStartsAtOrigin` PASS.

- [ ] **Step 8: Remove the skip from Task 1's test**

Delete the `SH_FONT_COVERAGE_STRICT` guard and the `"os"` import — the baseline is gone and the assertion is now unconditional.

- [ ] **Step 9: PROVE existing plate output is unchanged**

```bash
nix develop --command go test ./backup/ ./engrave/ ./gui/ -v 2>&1 | tail -30
```

Expected: **all goldens pass without `-update`.** This is the single most important check in Phase A — it is the evidence for spec D5's claim that existing output is provably unaffected. If any golden moves, stop and investigate: the shared `ConstantStringer` should not have been touched.

- [ ] **Step 10: Visual inspection of confusables**

Regenerate the dump SVG and check, by eye, every pair in the list above. Record which convention you used for each in the commit message. Note that final judgement is deferred to O1 (engraved metal) — this is the screen-level pre-check.

- [ ] **Step 11: Commit**

```bash
git add font/constant/constant.svg font/constant/constant.bin font/constant/constant.go \
        font/constant/coverage_test.go font/constant/glyph_rules_test.go
git commit -s -m "feat(font): add 26 lowercase, 17 symbol and 1 visible-space glyph

Brings the engraving face to full printable-ASCII coverage plus a visible-space
mark at 0x1F, so a case-sensitive BIP-39 passphrase can be engraved verbatim.

Metrics are unchanged (height 9, baseline 8, advance 6): descenders take the
single unit below the baseline rather than raising the em box, which would have
rescaled every existing glyph and changed the geometry of every plate this
machine makes. All existing goldens pass unmodified.

Confusable pairs differentiated: <record conventions used>."
```

---

### Task 4: Separate passphrase alphabet and `ConstantStringer`

**Files:**
- Modify: `engrave/engrave.go` — add a constant next to `constantAlphabet` (`:750`) and a constructor
- Create: `engrave/passphrase_alphabet_test.go`

**Interfaces:**
- Consumes: the extended face from Task 3.
- Produces: `const passphraseAlphabet string` and `func NewPassphraseStringer(face *vector.Face, params Params, em int) *ConstantStringer`. Phase B's plate layout consumes both.

- [ ] **Step 1: Write the failing test**

```go
package engrave

import (
	"testing"

	"seedhammer.com/font/constant"
)

// The passphrase alphabet must cover every printable ASCII rune plus the
// visible-space mark, be in ascending codepoint order (NewConstantStringer
// binary-searches it, engrave.go:1208-1210), and construct without panicking.
func TestPassphraseAlphabet(t *testing.T) {
	if passphraseAlphabet[0] != 0x1F {
		t.Errorf("alphabet must start with the space mark 0x1F, got %#x", passphraseAlphabet[0])
	}
	var last rune = -1
	for _, r := range passphraseAlphabet {
		if r <= last {
			t.Fatalf("alphabet not ascending at %q", r)
		}
		last = r
	}
	for r := rune(0x20); r <= 0x7E; r++ {
		found := false
		for _, a := range passphraseAlphabet {
			if a == r {
				found = true
				break
			}
		}
		if !found {
			t.Errorf("alphabet missing %q", r)
		}
	}
	if got, want := len([]rune(passphraseAlphabet)), 96; got != want {
		t.Errorf("alphabet has %d runes, want %d (95 printable + mark)", got, want)
	}
	// Must not panic: exercises the ascending-order check (engrave.go:1210),
	// the alphabet-subset-of-face check (:1215) and uniform advance (:1218).
	NewPassphraseStringer(constant.Font, params(), 4*mm)
}
```

`params()` and `mm` already exist in `engrave/engrave_test.go` (`:112`, `:131-137`) — same package, no import needed.

- [ ] **Step 2: Run it and confirm it fails to compile**

Run: `nix develop --command go test ./engrave/ -run TestPassphraseAlphabet -v`
Expected: FAIL — `undefined: passphraseAlphabet`.

- [ ] **Step 3: Add the alphabet and constructor**

```go
// passphraseAlphabet is the alphabet for engraving BIP-39 passphrases: the
// visible-space mark followed by every printable ASCII rune, in ascending
// codepoint order (NewConstantStringer binary-searches it).
//
// It is deliberately SEPARATE from constantAlphabet. NewConstantStringer
// derives runeDuration, startEndDist and center from whichever alphabet it is
// given, accumulating bounds over every glyph. The lowercase descenders push
// bounds.Max.Y positive, which would move center and startEndDist for every
// constant-time string the machine engraves -- changing the goldens for seed,
// SLIP-39 and codex32 plates. Widening constantAlphabet is forbidden.
const passphraseAlphabet = "\x1f !\"#$%&'()*+,-./0123456789:;<=>?@" +
	"ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`" +
	"abcdefghijklmnopqrstuvwxyz{|}~"

// NewPassphraseStringer builds a ConstantStringer over passphraseAlphabet.
func NewPassphraseStringer(face *vector.Face, params Params, em int) *ConstantStringer {
	return newConstantStringer(face, params, em, passphraseAlphabet)
}
```

`NewConstantStringer` currently reads the package-level `constantAlphabet` directly. Refactor it into an unexported `newConstantStringer(face, params, em, alphabet string)` and have `NewConstantStringer` call it with `constantAlphabet` — a pure extraction, no behaviour change.

- [ ] **Step 4: Run the test**

Run: `nix develop --command go test ./engrave/ -run TestPassphraseAlphabet -v`
Expected: PASS.

- [ ] **Step 5: Prove the shared stringer is untouched**

```bash
nix develop --command go test ./backup/ ./engrave/ ./gui/ 2>&1 | tail -20
```
Expected: all pass, no golden updates. The extraction in Step 3 must be behaviour-preserving.

- [ ] **Step 6: Record the timing cost (spec O5)**

Add a benchmark comparing both stringers' `runeDuration`:

```go
func TestPassphraseStringerTiming(t *testing.T) {
	shared := NewConstantStringer(constant.Font, testParams(), 4*mmTest)
	pass := NewPassphraseStringer(constant.Font, testParams(), 4*mmTest)
	t.Logf("shared runeDuration=%d  passphrase runeDuration=%d", shared.runeDuration, pass.runeDuration)
	t.Logf("shared center=%v  passphrase center=%v", shared.center, pass.center)
}
```

Run it and **record both numbers in the commit message**. This closes O5: the shared values must be identical to their pre-change values, and the passphrase ones are simply reported.

- [ ] **Step 7: Commit**

```bash
git add engrave/engrave.go engrave/passphrase_alphabet_test.go
git commit -s -m "feat(engrave): separate passphrase alphabet and ConstantStringer

A BIP-39 passphrase is secret material, so it must be engraved constant-time --
but the shared constantAlphabet is 36 characters (uppercase and digits) and
ConstantStringer panics on anything else WITHOUT consulting the face.

The alphabet is separate rather than widened. NewConstantStringer derives
runeDuration, startEndDist and center from its alphabet by accumulating bounds
over every glyph; the new lowercase descenders push bounds.Max.Y positive, which
would move center for every constant-time string on the machine and change the
goldens for seed, SLIP-39 and codex32 plates. Separate instance, zero effect on
existing output.

O5 measurements: shared runeDuration <N> (unchanged), passphrase <M>."
```

---

### Task 5: Determine whether `ConstantQR` can reach version 6 (spec O6)

**This task's deliverable is an answer, not necessarily a feature.** It decides whether Phase B's passphrase cap is 100 characters with QR, or 78.

**Files:**
- Modify: `engrave/engrave_test.go` — `FuzzConstantQR` (`:427-454`)
- Modify (only if the answer is yes): `engrave/engrave.go` at `:349-365`, `:384-401`, `:406-413`
- Create: `design/agent-reports/` — no; record the finding in the plan's Phase B input instead

**Interfaces:**
- Consumes: nothing.
- Produces: either working v5/v6 constant QR, or a recorded decision that Phase B caps the passphrase at 78 characters when QR is enabled.

- [ ] **Step 1: Understand what the constant is**

Run: `nix develop --command sed -n '349,365p' engrave/engrave.go`

`constantTimeQRModules` returns a per-dimension **maximum module count**, documented as *"maximum numbers found through fuzzing… Add a bit more to account for outliers not yet found"* with `const extra = 5`. It sets both the failure threshold (`:479`) and the engraving duration (`:641`, `:649`). Too small → content-dependent failures. Too large → every QR of that size engraves slower.

- [ ] **Step 2: Extend the fuzz target to reach versions 5 and 6**

`FuzzConstantQR` currently truncates entropy to 40 bytes (`:432-434`), which never produces a 37-module code. Raise the cap and record observed module counts:

```go
	if m := 120; len(entropy) > m {
		entropy = entropy[:m]
	}
```

and inside, after each `qr.Encode`, log `qrc.Size` and the module count so a corpus run reports the distribution.

- [ ] **Step 3: Run the fuzzer long enough to be meaningful**

```bash
nix develop --command go test ./engrave/ -run FuzzConstantQR -fuzz FuzzConstantQR -fuzztime 10m
```

Record the maximum module count observed for dims 37 and 41.

- [ ] **Step 4: Decide, and record the decision explicitly**

- **If the maxima converge** (the fuzzer stops finding higher counts well before the time limit): extend all three sites — the `dim > 33` guard, `bitmapForQRStatic`'s switch (v5/v6 take one alignment marker each at `(dim-9, dim-9)`; `newBitmap` is safe to width 64), and `constantTimeQRModules` with the derived maxima plus `extra`.
- **If they do not converge**, or the derived duration makes QR engraving unacceptably slow: **stop**. Phase B caps the passphrase at 78 characters when QR is enabled — QR v4-L's exact byte capacity. Falling back to non-constant-time `engrave.QR` is **forbidden** (spec §3.5.2): it would engrave a secret with content-dependent timing.

Write the outcome into `design/SPEC_seedhammer_engrave_bip39_password.md` §9 as O6's resolution, with the observed numbers.

- [ ] **Step 5: If extending — verify fail-closed behaviour survives**

Add a test asserting a 41-module QR either engraves correctly or returns an error, and **never** truncates:

```go
func TestConstantQRLargeVersionsFailClosed(t *testing.T) {
	long := strings.Repeat("Xy7#", 30) // 120 bytes -> v6-ish
	c, err := qr.Encode(long, qr.L)
	if err != nil {
		t.Fatal(err)
	}
	cmd, err := ConstantQR(c)
	if err != nil {
		t.Skipf("v%d not supported, Phase B uses the 78-char cap: %v", c.Size, err)
	}
	if cmd == nil {
		t.Fatal("ConstantQR returned nil without an error -- truncation, not fail-closed")
	}
}
```

- [ ] **Step 6: Run the full suite and commit**

```bash
nix develop --command go test ./... 2>&1 | grep -v "no test files"
git add engrave/engrave.go engrave/engrave_test.go \
        ../mnemonic-engrave/design/SPEC_seedhammer_engrave_bip39_password.md
git commit -s -m "feat(engrave): <extend constant QR to v5/v6 | record O6 as the 78-char cap>

<observed fuzz maxima and the reasoning>"
```

---

## What Phase A does NOT cover

Deliberately out of scope, and the subject of Phase B (written once O5/O6 are
answered, because O6 decides whether the passphrase caps at 100 or 78 characters
when QR is enabled):

| Spec section | Deferred to Phase B |
|---|---|
| §3.4 | `ValidatePassphrase` / `ValidateFingerprint` and the five-check table |
| §4 | `backup.Passphrase` plate type, both layouts, metadata bands, legend |
| §5.0 | Keyboard extension to all 32 symbols |
| §5 / §5.1 / §5.3 | Flow, warnings, confirm-screen space surfacing, `[]byte` wiping |
| §6 | Menu insertion at position 2 of 7 |
| §7 | Three-way alignment, QR byte-exactness, worst-case fit, touch, canonicalisation |

## Phase A exit criteria

Phase B may not begin until all of these hold:

- [ ] `TestPrintableASCIICoverage` passes unconditionally — 95/95.
- [ ] `TestUniformAdvance` and `TestNoGlyphStartsAtOrigin` pass.
- [ ] `nix develop --command go test ./...` is green **without any `-update`**.
- [ ] Every confusable pair inspected on screen, conventions recorded.
- [ ] O5 answered: shared `runeDuration`/`center` provably unchanged, passphrase values recorded.
- [ ] O6 answered: either v5/v6 constant QR works, or Phase B's cap is 78 characters with QR.
- [ ] Mandatory post-implementation adversarial execution review over the whole Phase A diff (risk-set work, `CLAUDE.md`) — **non-deferrable**.

**Still open after Phase A, by design:** O1, the hardware legibility check. Lowercase has never been engraved on this machine, and screen inspection is not a substitute for looking at cut metal. That gate belongs to the feature, not the substrate.
