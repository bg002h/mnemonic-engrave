# Opus architect — R0 round 0, Engrave Text implementation plan

Scope: the PLAN only (spec is closed). All probing in `/tmp/shexp`; real tree untouched at
`abb7458`; baseline suite exit 0.

## VERDICT
**NOT GREEN (2C / 6I)** + 4 Minor, 2 Nit.

## C1 — `WrapText("")` returns one empty line; `EngraveText`'s empty-text paragraph must produce ZERO. Golden `text-2-shards-1` moves.
`backup/backup.go:322`, `:356`; callers `gui/gui.go:443`, `:1959`. Both callers build a QR-ONLY
variant with `Text: ""`, and it is golden-covered (`TestText` case 3). Today empty text emits zero
lines and triggers the `len(p.Text) == 0` branch that **centers** the QR. Spec §5.2 rule 1 makes
`WrapText("")` emit **one** empty line, so the literal refactor gives `len(lines) == 1`, the
centering branch never fires, and the QR moves to the right-hand side.
**Measured, both readings implemented:** centering keyed to the original `p.Text` → full suite
**exit 0**, all three `text-*` byte-identical. Centering keyed to `len(lines)` →
`TestText/2-shards-1` fails, `45281/45282 knot mismatches`; QR displaced (3.005, 6.100)mm at test
params, (6.450, 2.300)mm at production stroke.
**Fix:** B3.2 must special-case `len(p.Text) == 0` before wrapping and keep the centering test
keyed to the original text. §5.2's empty-block rule serves the free-text plate only.

## C2 — No task implements the free-text plate engraving (title, footer, QR at the fitted size)
Phase B yields `WrapText` + refactor, C yields `Fit`, D yields keyboard/flow/menu/copy. **Nothing
puts ink on the plate.** `backup.Text` has no `Title`/`Footer` field (`backup/backup.go:33-42`);
`EngraveText` has no title concept; `Text.FontSize` is added in B1.3 and never set by any later
task. C1.7 asks for a bounds test with no code under test, so it can only re-derive the formula it
is meant to check.
**Why it bites:** the only title pattern in the repo (`backup/backup.go:153-160`, `:242-249`)
centers on the **full plate width** and calls `strings.ToUpper` — exactly the silently-wrong plate
spec §2 measured (20-char title at 6.0mm inking `x[7.127, 77.962]`, crossing both screw-hole bands
while every check passes). An implementer told "everything else is assembly" copies the nearest
pattern and lands the failure the spec exists to prevent. It is also the one place the
confirm↔plate binding is unenforced: D2.1 binds the text lines, nothing binds title, footer, QR
placement, or the fitted size.
**Fix:** add a task producing `backup/freetext.go`:
`EngraveFreeText(params, fontMM float32, title string, lines []string, footer string, qrc *qr.Code)`,
laying out §8 (title row 0, footer row `LinesPerPlate-1`, both centered in the **inset span**,
verbatim — not `TitleString`; `QRScale = 2`), with its own new golden. Move C1.7's bounds test onto it.

## I1 — D3 lists one test file; six existing tests across six files break
Measured: inserting `engraveText` after `engravePassphrase` → `go test ./gui/` exit 1 with
`TestBip85DeriveProgramNavigable`, `TestEngraveBundleProgramNavigable`,
`TestEngraveXpubProgramNavigable`, `TestEngraveMultisigProgramNavigable`,
`TestEngraveSingleSigProgramNavigable`, `TestStartScreenPagerTouchReachesEveryProgram`. Five files
unlisted; each hardcodes a Right-press count. D1.4 sets the norm "if an existing test needs
changing, stop" — the implementer hits five red files with no guidance.
**Fix:** list all six and state the only permitted edit is the navigation index/expected title.

## I2 — The enum has FOUR keyed sites, not two; the flow-dispatch site is untested
`gui/gui.go:147-158` (enum), `:1506-1533` (flow dispatch, **no default**), `:1676-1691` (title
switch, no default), `:1893` (`layoutMainPlates`). Without a `case engraveText:` the selection
falls through with `obj == nil` into `engraveObjectFlow(ctx, th, nil)`. D3.1 tests carousel
reachability only; D2 drives the flow directly. **Nothing presses OK on the new menu entry**, so
the feature can ship with a dead menu item and a green suite.
**Fix:** list four sites; add a test that selecting the program by touch actually enters the flow.

## I3 — B3 gives `EngraveText` no signature; the two readings put golden-critical geometry in different places
`charWidth`, `holeChars`, `holeLines`, `charPerLine`, `charPerQRLine`, `qrLines`, `offy` are
private locals (`backup/backup.go:288-318`). Reading (a) keeps them and calls `WrapText`
internally; reading (b) — which B3.2's wording points at — changes the signature to accept
`lines []string` and forces every derivation to be reconstructed at `gui/gui.go:455` and `:1969`.
Only (a) is safe, and (a) is what my measured pass used.
**Fix:** state the signature; wrapping stays inside `EngraveText`, geometry stays private.

## I4 — `Fit` cannot express a `qr.Encode` failure its own over-capacity policy makes reachable
Measured: `qr.Encode` errors `text too long to encode as QR` at **2954** bytes (fine at 2953). D2.2
accepts keystrokes without limit and no task caps the Text field, so that input is constructible.
`Fit` has no error return and the plan is silent → nil `*qr.Code` and a nil dereference in a live
per-keystroke path.
**Fix:** give `Fit` an `error`, or specify `ok=false` on encode failure; add a Text cap in D2 with a test.

## I5 — Admission, "lines available", and the refusal figure have no entry point
`Fit` answers "largest rung that fits". Admission (§6) is a different question — does it fit at
3.0mm with both rows reserved *unconditionally*. C1.8's refusal figure needs a capacity solver;
D2.3's "lines used / available" needs a line count for text that does **not** fit, and `Fit`
returning `ok=false` says nothing about `lines`. Three tasks test functions no task defines.
**Fix:** add `Admissible(params, text, title, footer string, qr bool) (linesUsed, linesAvail int, ok bool)`
at 3.0mm with both rows always reserved, and `MaxCharsAt(params, fontMM, text, qr)` for the
refusal figure; point C1.6/C1.8/D2.3/D2.7 at them.

## I6 — A1.7's test body is empty; as written it passes on any font
The task ships a comment and no code, and an empty Go test passes. No capture mechanism is given,
and `vector.Face` exposes only `Decode` and `Metrics`, with `UniformBSpline` a consuming iterator.
A1.7 is the sole stated guard against an existing glyph silently changing every descriptor plate.
(A1.5's golden run does fence this in practice — but that is not what A1.7 claims.)
**Fix:** delete it and say A1.5's golden run is the guard, or specify the before/after dump.

## MINOR / NIT
- **M1** — `fixedCharWidth` has no value, type or semantics anywhere; B1.1 pins the other three.
- **M2** — the newline key's label is unspecified and `"↵"` measures **0 px** in
  `ctx.Styles.keyboard`, giving an 8 px target that D1.2's synthetic touch test would still pass.
  Any ASCII label fits: `"nl"` 285, `"enter"` 329, `"return"` 342, `"new line"` 366 px against 480.
- **M3** — A1.6's command does not run: `-dump` is a `flag.String` output path, so
  `vectorfont -dump sh.svg sh` consumes `sh.svg` as the dump path and fails `flag.NArg() != 2`.
- **M4** — B3.3 pins only "no spaces"; the equivalence also needs **no `'\n'`**. Measured, both
  golden inputs have 0 of each. Rename and assert both.
- **N1** — D3.4's symptom is wrong: inserting after `bip85Derive` trips the compile-time guard at
  `gui/gui.go:168` — a build failure, not pager drift.
- **N2** — for paragraphs after the first, `offy` advances by `lineno*fontSize + 1mm`, so the
  closure must capture `offy` in **device units**, not a row index.

## VERIFIED BY MEASUREMENT
Baseline exit 0. B3 achievable under reading (a) only — implemented `wrap.go` + the refactor with a
`widthAt` closure reproducing `n` incl. the clamp: faithful reading exit 0 with all three `text-*`
byte-identical; naive reading fails with knot mismatches. QR displacement figures above. Menu
insertion → 6 failures across 6 files. `font/sh` 81/95 with exactly the 14 missing, all advances
4000, `Metrics{Ascent:5000, Height:6700}`. `qr.Encode` boundary 2953/2954. Keyboard row 249-256 px,
5th key labels all fit, `"↵"` 0 px. `engrave.String("")` emits 0 commands, so C1's golden move comes
from the centering branch, not stray ink. Spec §4's grid and §5.1's inset rows reproduce exactly
from the real integer arithmetic — C1.1 is achievable as written.
