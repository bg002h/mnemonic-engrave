# Implementation Plan — Engrave Text (free-text plate)

> **For agentic workers:** implement task-by-task, test-first. Steps use `- [ ]`.

**Status:** rev 2 — plan R0 rounds 0 (2C/6I) and 1 (0C/3I) folded. Awaiting re-review.
**Spec:** `design/SPEC_seedhammer_engrave_free_text.md` — **GREEN, R0 closed (rev 2)**.
Read §5 before writing anything; the whole feature turns on it.

**Goal:** a free-text plate program at menu slot 3, using `font/sh`, auto-fitting
6.0→3.0mm, with an optional QR encoding the text only, and optional 18-character
title and footer.

**Architecture:** one wrapping function (`backup.WrapText`) serves three callers
— the confirm screen, the engraver, and the fit check. `EngraveText` is refactored
to wrap internally. A **new** `backup.EngraveFreeText` lays out the free-text
plate: title, wrapped body, footer, QR, at the fitted size.

**"Everything else is assembly" was wrong** and round 0 caught it: there is no
existing code that engraves a title on a text plate. The nearest pattern
(`backup/backup.go:153-160`, `:242-249`) centers on the **full plate width** and
calls `strings.ToUpper` — precisely the silently-wrong plate spec §2 measured. Do
not copy it.

**Tech stack:** Go (TinyGo target), `nix develop --command go test ./...`.

## Global constraints

Every task inherits these. They are not suggestions.

- **Check `$?`.** Never `go test | grep FAIL` — a build error goes to a different
  stream and has produced a false GREEN on this project.
- **No existing golden may be updated.** New goldens are fine. `-update` is
  forbidden.
- **The three `text-*` goldens must be byte-identical** after Phase B.
- **Verify every fix by mutation.** Break the code; if the suite stays green, the
  test is decoration. Report surviving mutations honestly.
- **Measure, don't assume.** Numbers in the spec were measured; re-measure rather
  than re-derive. `charWidth` comes from the font, `StrokeWidth` from production
  (`0.3 * mm`) — a capacity test at `backup_test.go`'s `mm/3` reproduces §4's QR
  column falsely and is forbidden.
- **`font/sh` is a B-spline face.** SVG points are control points, not the drawn
  curve. Render through `cmd/vectorfont -dump` before believing a glyph edit.
- **`vectorfont` writes into the current directory.** Run font work in a
  throwaway worktree.
- Work in a worktree on branch `free-text`. Stage paths explicitly.

---

# Phase A — the font (must land first)

`engrave.String` **panics** on a rune absent from the face
(`engrave/engrave.go:1531`), so the glyphs must exist before any keyboard can
emit them.

### Task A1 — import the 14 glyphs

**Files:** `font/sh/sh.svg` (modify), `font/sh/sh.bin` (regenerated),
`PROVENANCE.md` (modify)

- [ ] **A1.1 — failing test first.** In `font/sh/sh_coverage_test.go` (new):

```go
// Every printable ASCII rune must decode, because engrave.String panics on a
// rune the face lacks and the free-text keyboard can emit all 95.
func TestSHCoversPrintableASCII(t *testing.T) {
	var missing []rune
	for r := rune(0x20); r <= 0x7E; r++ {
		if r == ' ' {
			continue // space inks nothing; advance-only
		}
		if _, _, ok := Font.Decode(r); !ok {
			missing = append(missing, r)
		}
	}
	if len(missing) > 0 {
		t.Fatalf("font/sh is missing %d printable ASCII glyphs: %q", len(missing), missing)
	}
}
```

- [ ] **A1.2 — run it; expect FAIL** listing exactly 14: `!"$%&+=?\^_` `` ` `` `|~`
- [ ] **A1.3 — import.** Fetch `font/sh/sh.svg` from `Gangleri42/seedhammer`
      (record the exact commit). Copy **only** the 14 missing `<path id=…>`
      elements into our `sh.svg`, appended in the same document order they appear
      in theirs. Do not touch existing glyphs — verify with
      `git diff` that only additions appear.
- [ ] **A1.4 — regenerate** in a throwaway worktree:
      `cd font/sh && nix develop --command go generate ./...`
- [ ] **A1.5 — run A1.1; expect PASS.** Then `nix develop --command go test ./...`; expect exit 0.
- [ ] **A1.6 — render and eyeball.** `-dump` takes an OUTPUT path and the tool
      still needs its two positional args:
      `nix develop --command go run seedhammer.com/cmd/vectorfont -package sh -scale 1000 -dump /tmp/sh-dump.svg sh.svg sh`
      then `rsvg-convert` it. Confirm the 14 new glyphs match the face.
- [ ] **A1.7 — assert no existing glyph moved, by DUMP COMPARISON.** An empty Go
      test passes, so do not ship a stub. Before A1.3, dump every existing glyph:

```sh
# capture BEFORE the import
nix develop --command go run seedhammer.com/cmd/vectorfont -package sh -scale 1000 \
  -dump /tmp/sh-before.svg font/sh/sh.svg sh
```

      After A1.4, dump again to `/tmp/sh-after.svg` and assert that every
      `<path id=…>` present in *before* is byte-identical in *after* — additions
      only. A shell/py check committed as `font/sh/import-check.md` with both
      dumps' hashes is sufficient; the standing guard thereafter is A1.5's golden
      run, since all three `text-*` goldens engrave with `sh.Font`.

- [ ] **A1.8 — PROVENANCE.md.** Add under *Imports*: source repo, exact commit,
      the 14 glyph ids, licence (Unlicense), obligations (none). State that the
      base glyphs were verified byte-identical to upstream.
- [ ] **A1.9 — commit** `font/sh/sh.svg font/sh/sh.bin font/sh/sh_coverage_test.go PROVENANCE.md`

**Exit criteria:** all 95 decode; no existing glyph changed; full suite exit 0;
no golden moved.

---

# Phase B — wrapping and the `EngraveText` refactor

The riskiest phase: it touches descriptor plates, which have goldens.

### Task B1 — import the size-ladder API

**Files:** `backup/backup.go` (modify), `backup/sizes_test.go` (new)

- [ ] **B1.1 — failing tests** pinning §4's grid at production params
      (`StrokeWidth: 0.3 * mm`), all six rungs:
      `CharsPerLine` → 22/26/30/34/38/44, `LinesPerPlate` → 13/15/17/20/23/26.
- [ ] **B1.2 — run; expect FAIL** (symbols undefined).
- [ ] **B1.3 — add** `const plateSize = 85`, `FontSizes`, `CharsPerLine`,
      `LinesPerPlate`, and `Text.FontSize float32`. `fixedCharWidth` is the
      private helper the other two share — spec §10 names it but gives no body:

```go
// fixedCharWidth is the character advance at fontSize machine units, assuming a
// fixed-width face. Verified: every font/sh advance is 4000 with
// Metrics{Ascent:5000, Height:6700}, so 'W' is exact for all glyphs.
func fixedCharWidth(fnt *vector.Face, fontSize int) int {
	w, _, ok := fnt.Decode('W')
	if !ok {
		panic("W not in font")
	}
	return int(float32(w*fontSize) / float32(fnt.Metrics().Height))
}
```

- [ ] **B1.4 — `FontSize == 0` falls back to `plateFontSizeUR`.** Test it
      explicitly: this is what keeps the `text-*` goldens identical.
- [ ] **B1.5 — run; expect PASS**, full suite exit 0, goldens unmoved.
- [ ] **B1.6 — commit.**

### Task B2 — `WrapText`

**Files:** `backup/wrap.go` (new), `backup/wrap_test.go` (new)

Signature and semantics are spec §5 verbatim. `widthAt` is indexed by **output
line**; the caller supplies the plate-row offset. For the descriptor path that
offset is **not row-aligned** — paragraphs after the first advance `offy` by
`lineno*fontSize + 1mm` (`backup/backup.go:363-367`) — so the closure must
capture `offy` in **device units**, not as a row index.

- [ ] **B2.1 — failing tests, one per rule.** At minimum:

```go
func TestWrapWordBoundaries(t *testing.T)      // greedy fill, break on U+0020 only
func TestWrapOverlongToken(t *testing.T)       // alone-on-empty-line -> break at exactly widthAt(i)
func TestWrapSpacePrecedence(t *testing.T)     // (a) break run consumed, (b) leading indent kept,
                                               // (c) trailing stripped; all-space line -> empty
func TestWrapVaryingWidth(t *testing.T)        // widthAt returning different values per line
func TestWrapExplicitNewlines(t *testing.T)    // blocks; empty block -> one empty line
func TestWrapRefusesPastMaxLines(t *testing.T) // (partial, false), including mid-token
func TestWrapAssertsWidthAtPositive(t *testing.T) // widthAt returning 0 must panic, not hang
```

The last one matters: §5's fallback slices `w[:widthAt(i)]`, so a zero width
consumes nothing and appends forever on a device with no OOM killer.

- [ ] **B2.2 — run; expect FAIL.**
- [ ] **B2.3 — implement** per §5.2, precedence order (a) → (b) → (c).
- [ ] **B2.4 — run; expect PASS.**
- [ ] **B2.5 — mutation-verify.** At minimum: break on `-` as well as space;
      drop the trailing-space strip; return `true` when over `maxLines`; skip the
      `widthAt >= 1` assertion. **Each must turn the suite red.** Any survivor is
      a test gap — fix the test, not the report.
- [ ] **B2.6 — commit.**

### Task B3 — refactor `EngraveText` to consume lines

**Files:** `backup/backup.go` (modify), `backup/backup_test.go` (unchanged —
this is the point)

- [ ] **B3.1 — failing test:** the three `text-*` goldens must be byte-identical
      after the refactor. They already exist; the test is that they still pass
      **without `-update`**.
- [ ] **B3.2 — refactor, keeping the geometry PRIVATE.** `EngraveText` keeps its
      current signature and its private locals (`charWidth`, `holeChars`,
      `holeLines`, `charPerLine`, `charPerQRLine`, `qrLines`, `offy`) and calls
      `WrapText` **internally** via a `widthAt` closure. Do **not** change the
      signature to accept `lines []string`: that forces every one of those
      golden-critical derivations to be reconstructed at `gui/gui.go:455` and
      `:1969`, with no home for the arithmetic. Round 0 measured both readings —
      only this one keeps the goldens.
      The descriptor and mdmk callers keep an **unbounded** path; their
      TEXT+QR → TEXT-ONLY → QR-ONLY fallback depends on `toPlate` rejecting
      overflow, and bolting a `maxLines` refusal in would silently change it.

- [ ] **B3.2a — EMPTY TEXT MUST ENGRAVE ZERO LINES.** Both callers build a
      QR-ONLY variant with `Text: ""` (`gui/gui.go:443`, `:1959`) and it is
      golden-covered (`TestText` case 3, `text-2-shards-1.bin`). Today zero lines
      triggers the `len(p.Text) == 0` branch that **centers** the QR. Spec §5.2's
      empty-block rule makes `WrapText("")` return **one** empty line — that rule
      serves the free-text plate **only**. So: special-case `len(p.Text) == 0`
      before wrapping, and keep the centering test keyed to the **original text**,
      never to `len(lines)`.
      Measured: keying it to `len(lines)` fails `TestText/2-shards-1` with
      45281/45282 knot mismatches and displaces the QR by (6.450, 2.300)mm at
      production stroke. Add a test asserting the QR-ONLY paragraph still centers.
- [ ] **B3.3 — the line-producing path must reproduce `n` bit-for-bit**, including
      `if n < 1 { n = 1 }` (`backup/backup.go:341-343`). Descriptor text contains
      no spaces, so greedy fill plus break-at-exactly-`widthAt(i)` reduces to the
      existing `txt[:n]` — that equivalence is why the goldens survive. Pin it:

```go
// The refactor is only safe because descriptor text contains neither spaces NOR
// newlines: a space would change the greedy fill, and a '\n' would split into
// blocks under spec 5.2 rule 1, whereas today engrave.String handles '\n'
// inside a sliced chunk. Measured: both golden inputs have zero of each.
func TestDescriptorTextHasNoSpacesOrNewlines(t *testing.T)
```

- [ ] **B3.4 — run full suite; expect exit 0 and zero golden churn**
      (`git status --porcelain -- '*testdata*'` must be empty).
- [ ] **B3.5 — mutation-verify the equivalence:** change the clamp to `n < 2`,
      or drop it. The goldens must go red.
- [ ] **B3.6 — commit.**

**Exit criteria:** `text-*` goldens byte-identical; `WrapText` mutation-verified;
suite exit 0.

---

# Phase C — fit, capacity, admission

**Files:** `backup/fit.go` (new), `backup/fit_test.go` (new)

### Task C1 — the fit model

- [ ] **C1.1 — failing tests** pinning §5.1's inset rows per rung
      (6.0 → `[0 1 12]` … 3.0 → `[0 1 2 24 25]`), `holeChars` 2/3/3/4/4/4, and
      §4's Plain (274…1104) and +title+footer (238…1032) columns at production
      stroke.
- [ ] **C1.2 — implement THREE entry points.** `Fit` alone cannot answer the
      questions C1.6/C1.8/D2.3/D2.7 ask:

```go
// Fit: the largest rung whose layout holds the composition.
func Fit(params engrave.Params, text, title, footer string, qr bool) (fontMM float32, lines []string, err error)

// Admissible: spec 6's anchor -- 3.0mm, QR as chosen, BOTH title and footer rows
// reserved whether or not they are used. linesAvail is defined even when ok is
// false, so the UI can show "lines used / lines available" over capacity.
func Admissible(params engrave.Params, text, title, footer string, qr bool) (linesUsed, linesAvail int, ok bool)

// MaxCharsAt: capacity solver behind the refusal message's live figure.
func MaxCharsAt(params engrave.Params, fontMM float32, text string, qr bool) int
```

      `widthAt` composes §5.1's band predicate with the QR narrowing and the
      caller's plate-row offset.

- [ ] **C1.2a — ONE encoding call site**, shared by all four consumers, so the
      failure mode is defined once and the engraved artifact is the same object
      that was measured:

```go
// qrFor returns the code the plate will carry, or nil when want is false.
// qr.Encode fails at 2954 bytes and above (measured; fine at 2953), and the Text
// field is deliberately uncapped (D2.2), so that input is REACHABLE on a live
// per-keystroke path. Every caller must handle err; none may dereference a nil
// *qr.Code.
func qrFor(text string, want bool) (*qr.Code, error)
```

      `Fit` returns an `error`. **`Admissible` returns `ok=false` on an encode
      failure** — with `linesAvail` still meaningful so the readout keeps working
      — and **`MaxCharsAt` returns 0**. Neither may panic. Test all three at 2954
      bytes with the QR on.
- [ ] **C1.3 — QR size comes from `qr.Encode(text, qr.L).Size`.** Never a length
      table. Tests: 106 lower→37, 106 upper→33, 106 digit→29, 114 upper→**33**,
      114 upper + one lowercase→**41** (two versions from one keystroke).
- [ ] **C1.4 — monotonicity property test:** module count never decreases as
      characters are appended, over a long run in both cases.
- [ ] **C1.5 — true maxima** match spec §4's second table (520 lowercase / 616
      uppercase at 3.0mm). Solve by iteration; do not read the table.
- [ ] **C1.6 — admission is anchored at 3.0mm** with both rows reserved
      unconditionally and the QR as chosen. Test that entering a title afterwards
      never invalidates already-accepted text.
- [ ] **C1.8 — refusal figure computed live**, not from a constant: at 3.0mm with
      a 700-character text, dropping the QR frees **640**.
- [ ] **C1.9 — mutation-verify:** anchor admission at the fitted size instead of
      3.0mm; drop the unconditional row reservation; read QR size from a byte
      table. Each must turn the suite red.
- [ ] **C1.10 — commit.**

### Task C2 — the free-text plate engraver *(added: round 0 found nothing engraved it)*

**Files:** `backup/freetext.go` (new), `backup/freetext_test.go` (new),
`backup/testdata/freetext-*.bin` (new goldens)

```go
// EngraveFreeText lays out the free-text plate per spec 8. title and footer are
// engraved VERBATIM -- never through TitleString, which upper-cases and
// truncates at 18 -- and are centered in the INSET SPAN, not the full width.
func EngraveFreeText(params engrave.Params, fontMM float32,
	title string, lines []string, footer string, qrc *qr.Code) engrave.Engraving
```

- [ ] **C2.1 — failing tests.** Title on plate row 0, footer on row
      `LinesPerPlate-1`, body between them, QR right-hand side at `QRScale = 2`.
- [ ] **C2.2 — centering is in the inset span** `[holeChars*charWidth,
      width − holeChars*charWidth]`. **Do not copy `backup/backup.go:153-160`** —
      it centers on the full plate width, which spec §2 measured as the
      silently-wrong plate: a 20-char title at 6.0mm inks `x[7.127, 77.962]`mm,
      crossing both screw-hole bands while every check passes.
- [ ] **C2.3 — title and footer engraved verbatim.** Mutation: route either
      through `TitleString` — a lowercase title must go red.
- [ ] **C2.4 — the 18-char cap holds at EVERY rung** (moved here from C1.7, which
      had no code under test): a title and footer at the cap sit inside
      `[innerMargin, plateSize − innerMargin]`. Use true ink bounds via
      `bspline.Measure` and the worst-case glyph `W`. Slack at 6.0mm is ~0.62mm —
      tight by design.
- [ ] **C2.5 — new goldens** for a representative plate with and without a QR.
      New goldens are permitted; existing ones are not to move.
- [ ] **C2.6 — mutation-verify:** center on full width (C2.4 must fire), drop the
      footer row, place the QR at `QRScale = 3`.
- [ ] **C2.7 — commit.**

**Exit criteria:** every number in spec §4 and §5.1 pinned by a test at production
stroke; the free-text plate has its own engraver and goldens; mutations killed.

---

# Phase D — GUI

### Task D1 — newline key, opt-in

**Files:** `gui/passphrase_keyboard.go` (modify), `gui/text_keyboard_test.go` (new)

- [ ] **D1.1 — NO existing keyboard test may be modified.** In particular
      `TestPassphraseKeyboardConstruction`'s "exactly four function-row keys" is
      the guard against a newline key leaking into `NewAddressKeyboard` and BIP-85
      index entry. Leave it alone.
- [ ] **D1.2 — failing tests** for a new `NewTextKeyboard`: five function-row
      keys, **newline appended last**, reveal still at index 2, every key
      reachable by touch on every page at `sh2DisplaySize`.
- [ ] **D1.3 — implement** `newPPKeyboard(ctx, newline bool)`;
      `NewPassphraseKeyboard` passes `false`, `NewTextKeyboard` passes `true`.
      **Label the key `"nl"`** (measured 285px row, against a 480px panel;
      `"enter"` 329 and `"return"` 342 also fit). **Do not use `"↵"`** — it
      measures **0px** in `ctx.Styles.keyboard`, giving an 8px tap target that a
      synthetic touch test would still pass.
- [ ] **D1.4 — run; expect PASS and the whole existing gui suite exit 0 with zero
      existing test files touched.** If any existing test needed changing, the
      design is wrong — stop and re-read §7.1.
- [ ] **D1.5 — mutation-verify:** make the flag unconditional. The anti-leak guard
      must fire.
- [ ] **D1.6 — commit.**

### Task D2 — the flow

**Files:** `gui/freetext_flow.go` (new), `gui/freetext_flow_test.go` (new)

Order is **QR → Text → Title → Footer → Confirm → Engrave** (spec §7); the QR
choice comes first so the admission anchor is fixed before typing.

- [ ] **D2.1 — failing test:** the single-wrap-function invariant.

```go
// The confirm screen's lines MUST equal WrapText's output for the same input
// and size. Each line is rendered as its OWN unwrapped label -- a width-bounded
// widget.Labelw would re-wrap in a proportional screen face and break the very
// invariant this test protects.
func TestConfirmLinesEqualWrapText(t *testing.T)
```

- [ ] **D2.2 — over-capacity is shown, not dropped.** Keystrokes are accepted; the
      readout shows the over-capacity state; **OK refuses**, naming the field.
      Follows `gui/passphrase_flow.go:113-118`'s reviewed decision.
- [ ] **D2.3 — readout is "lines used / lines available"**, never "characters
      remaining" — no scalar character count is correct under word wrap.
- [ ] **D2.4 — title/footer capped at 18**, refusal message on overflow.
- [ ] **D2.5 — QR encodes the Text only.** Assert at **module level**, as
      `TestPassphraseQRIgnoresFingerprints` does: a decoder ignoring trailing data
      would pass while the modules differed.
- [ ] **D2.6 — Back preserves every entered value**, driven through the real flow
      — not by setting a field and asserting the field. (That mistake shipped once
      on this project.)
- [ ] **D2.7 — refusal never drops the QR automatically**; it offers the choice and
      names the live figure.
- [ ] **D2.8 — mutation-verify** each of the above.
- [ ] **D2.9 — commit.**

### Task D2a — build the plate *(added: round 1 found `EngraveFreeText` had no caller)*

**Files:** `gui/freetext_flow.go` (modify), `gui/freetext_flow_test.go` (modify)

Nothing in Phase D called `EngraveFreeText`, and nothing produced its `*qr.Code`.
Mirror the established wiring: `gui/passphrase_flow.go`'s `ppBuildPlate` (`:532`)
feeding `NewEngraveScreen(ctx, plate).Engrave(...)` (`:645`).

- [ ] **D2a.1 — failing test:** the engrave step produces a plate whose lines are
      exactly `WrapText`'s output at the size the **confirm screen displayed** —
      binding the approved layout to the engraved one end to end, not just on
      screen.
- [ ] **D2a.2 — implement** `ftBuildPlate(params, text, title, footer string, qr bool) (Plate, error)`:
      call `Fit` for `(fontMM, lines)`, call **`qrFor`** for the code (the same
      helper the fit used — never a second, differently-parameterised encode),
      then `backup.EngraveFreeText(params, fontMM, title, lines, footer, qrc)`,
      then `toPlate`. Propagate every error; the flow shows the §6 refusal rather
      than engraving.
- [ ] **D2a.3 — mutation-verify:** re-encode the QR at a different ECC level in
      the builder than the fit measured — D2a.1 must go red, because the module
      count and therefore the line widths change.
- [ ] **D2a.4 — the QR carries the Text only**, asserted at module level here too:
      the fit path and the build path must produce byte-identical codes.
- [ ] **D2a.5 — commit.**

### Task D3 — menu integration

**Files:** `gui/gui.go` (modify) and **six** existing test files, measured — not
one. Inserting the program renumbers hardcoded Right-press counts in:
`gui/start_screen_touch_test.go`, plus the files holding
`TestBip85DeriveProgramNavigable`, `TestEngraveBundleProgramNavigable`,
`TestEngraveXpubProgramNavigable`, `TestEngraveMultisigProgramNavigable`,
`TestEngraveSingleSigProgramNavigable`.
**The only permitted edit is the navigation index / expected title — never the
assertion.** This is the one sanctioned exception to D1.4's "if an existing test
needs changing, stop".

- [ ] **D3.1 — failing test:** eight programs reachable **by touch**, the new one
      third, and the start screen fits at 8 pager dots at `sh2DisplaySize`.
      Touch, not synthesized `ButtonEvent`s — five of six programs were once
      unreachable on hardware while every button-driven test passed.
- [ ] **D3.2 — FOUR sites need hand-editing**, measured (spec §7.2 undercounted):
      1. the program enum (`gui/gui.go:147-158`),
      2. the flow dispatch `switch act.prog` (`:1506-1533`) — **no default**, so a
         missing case falls through with `obj == nil` into
         `engraveObjectFlow(ctx, th, nil)`,
      3. the title switch (`:1676-1691`) — no default, so a missing case renders a
         blank title,
      4. `layoutMainPlates`' case list (`:1893`), which **panics** on a program it
         does not list.
      `npage`/`npages` are `int(bip85Derive)+1` and update themselves —
      `bip85Derive` MUST remain last.

- [ ] **D3.2a — press OK on the new entry.** D3.1 proves the program is
      *reachable in the carousel*; D2 drives the flow directly. Neither selects it
      from the menu. Without this test the feature can ship with a **dead menu
      item and a fully green suite**. Assert on a string only the free-text flow
      renders.
- [ ] **D3.3 — run; expect PASS**, full suite exit 0.
- [ ] **D3.4 — mutation-verify:** omit the program from `layoutMainPlates` →
      panic; omit the flow-dispatch case → D3.2a fails (not a panic — it silently
      enters `engraveObjectFlow` with a nil object); omit the title case → blank
      title. Note inserting after `bip85Derive` trips the compile-time guard at
      `gui/gui.go:168` — a **build failure**, not pager drift.
- [ ] **D3.5 — commit.**

### Task D4 — safety copy

- [ ] **D4.1** Confirm screen states: this plate is **not a validated backup** and
      nothing has been checked; engraving is **not constant-time**, so duration
      leaks content; and if a QR is present, that the text is **machine-readable
      from a photograph**.
- [ ] **D4.2** Test the copy is present and gates correctly (QR warning only when
      a QR was chosen).
- [ ] **D4.3 — commit.**

---

# Exit criteria for the whole feature

- [ ] Full suite exit 0, `go build ./...` exit 0
- [ ] **Zero golden churn** — `git status --porcelain -- '*testdata*'` empty
- [ ] Every spec §11 requirement has a test
- [ ] Mutation report: what was mutated, what died, and **any survivor stated
      honestly** rather than quietly dropped
- [ ] `PROVENANCE.md` records the import with its exact commit
- [ ] Mandatory post-implementation adversarial review over the whole diff
      (project rule; non-deferrable) before merge to `main`

# Notes for the implementer

- The spec's numbers were all measured, and three review rounds found that **every
  Critical was a number or rule written down wrongly, never a design flaw**. If a
  test disagrees with the spec, measure before assuming which is right — then say
  so.
- `uiContains` lowercases **and strips spaces from its needle**, which has made
  negative assertions vacuously green three times. Prefer needles that cannot
  appear in the content under test.
- A rendered space inks nothing, so `ExtractText` yields `_=SPACE`, never `_ = SPACE`.
- `op.Drawer.ExtractText` collects runes **regardless of occlusion**. Fit is a
  rectangle question, never a text question.
