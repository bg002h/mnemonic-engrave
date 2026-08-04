# Implementation Plan — Engrave Text (free-text plate)

> **For agentic workers:** implement task-by-task, test-first. Steps use `- [ ]`.

**Spec:** `design/SPEC_seedhammer_engrave_free_text.md` — **GREEN, R0 closed (rev 2)**.
Read §5 before writing anything; the whole feature turns on it.

**Goal:** a free-text plate program at menu slot 3, using `font/sh`, auto-fitting
6.0→3.0mm, with an optional QR encoding the text only, and optional 18-character
title and footer.

**Architecture:** one wrapping function (`backup.WrapText`) serves three callers
— the confirm screen, the engraver, and the fit check. `EngraveText` is refactored
to consume pre-wrapped lines. Everything else is assembly.

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
- [ ] **A1.6 — render and eyeball.** `cmd/vectorfont -dump` the result and confirm
      the 14 new glyphs look like the rest of the face.
- [ ] **A1.7 — assert no existing glyph moved.** New test:

```go
// The import must be purely additive: an existing glyph's outline changing
// would silently alter every descriptor plate ever engraved.
func TestSHExistingGlyphsUnchanged(t *testing.T) {
	// Pin a sample of pre-import advances/segment counts captured before A1.3.
	// (Capture them in A1.2 and paste here — do not compute from the new font.)
}
```

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
      `LinesPerPlate`, `fixedCharWidth` exactly as spec §10 lists. Add
      `Text.FontSize float32`.
- [ ] **B1.4 — `FontSize == 0` falls back to `plateFontSizeUR`.** Test it
      explicitly: this is what keeps the `text-*` goldens identical.
- [ ] **B1.5 — run; expect PASS**, full suite exit 0, goldens unmoved.
- [ ] **B1.6 — commit.**

### Task B2 — `WrapText`

**Files:** `backup/wrap.go` (new), `backup/wrap_test.go` (new)

Signature and semantics are spec §5 verbatim. `widthAt` is indexed by **output
line**; the caller supplies the plate-row offset.

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
- [ ] **B3.2 — refactor.** `EngraveText` takes pre-wrapped lines. The descriptor
      and mdmk callers (`gui/gui.go:455`, `:1969`) keep an **unbounded** path —
      their TEXT+QR → TEXT-ONLY → QR-ONLY fallback depends on `toPlate`
      rejecting overflow, and bolting a `maxLines` refusal in would silently
      change it.
- [ ] **B3.3 — the line-producing path must reproduce `n` bit-for-bit**, including
      `if n < 1 { n = 1 }` (`backup/backup.go:341-343`). Descriptor text contains
      no spaces, so greedy fill plus break-at-exactly-`widthAt(i)` reduces to the
      existing `txt[:n]` — that equivalence is why the goldens survive. Pin it:

```go
// The refactor is only safe because descriptor text has no spaces. If that ever
// changes, the goldens move and this test says why.
func TestDescriptorTextHasNoSpaces(t *testing.T)
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
- [ ] **C1.2 — implement** `Fit(params, text, title, footer string, qr bool) (fontMM float32, lines []string, ok bool)`,
      trying `FontSizes` largest-first and returning the first that fits.
      `widthAt` composes §5.1's band predicate with the QR narrowing and the
      caller's plate-row offset.
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
- [ ] **C1.7 — the 18-character cap holds at every rung:** a title and a footer at
      the cap sit inside `[innerMargin, plateSize − innerMargin]`. Use true ink
      bounds and the worst-case glyph `W`. At 6.0mm the slack is ~0.62mm — this
      test is tight by design.
- [ ] **C1.8 — refusal figure computed live**, not from a constant: at 3.0mm with
      a 700-character text, dropping the QR frees **640**.
- [ ] **C1.9 — mutation-verify:** anchor admission at the fitted size instead of
      3.0mm; drop the unconditional row reservation; read QR size from a byte
      table. Each must turn the suite red.
- [ ] **C1.10 — commit.**

**Exit criteria:** every number in spec §4 and §5.1 pinned by a test at production
stroke; mutations killed.

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

### Task D3 — menu integration

**Files:** `gui/gui.go` (modify), `gui/start_screen_touch_test.go` (modify)

- [ ] **D3.1 — failing test:** eight programs reachable **by touch**, the new one
      third, and the start screen fits at 8 pager dots at `sh2DisplaySize`.
      Touch, not synthesized `ButtonEvent`s — five of six programs were once
      unreachable on hardware while every button-driven test passed.
- [ ] **D3.2 — two sites need hand-editing** (§7.2): the program enum, and
      `layoutMainPlates`' case list, which **panics** on a program it does not
      list. `npage`/`npages` are `int(bip85Derive)+1` and update themselves —
      `bip85Derive` MUST remain last.
- [ ] **D3.3 — run; expect PASS**, full suite exit 0.
- [ ] **D3.4 — mutation-verify:** omit the program from `layoutMainPlates` → panic;
      insert it after `bip85Derive` → pager/enum drift.
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
