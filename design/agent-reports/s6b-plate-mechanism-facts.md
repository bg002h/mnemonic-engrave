# S6b plate-text mechanism facts

Read-only factual recon. Repo: `/scratch/code/shibboleth/seedhammer`, `main` =
`b1479a1b38f6b045d27443764c858906e4e6e122`, clean working tree (confirmed via
`git status` / `git rev-parse HEAD` at the start of this recon). No design
recommendation is made here.

---

## PART 1 — the four plate-text mechanisms

### Mechanism 1 — `Fitted.Title` / `Fitted.Footer`

**Defined:** `backup/fit.go:100-144` (the `Fitted` struct). `Title, Footer
string` and `TitleFace, FooterFace *vector.Face` at `fit.go:120-121`;
`TitleSizeMM, FooterSizeMM float32` at `fit.go:129`. Constructors:
`fitBlocksAt` (`fit.go:332-367`, the one-rung case both `FitBlocks` and
`FitBlocksAt` share), and `FitSized` (`fit.go:430-513`, per-block sizes).
`Fit` (`fit.go:518-524`) is `FitBlocks` for a single face/block.

**Render path:** `backup/freetext.go:48` `EngraveFitted(params, f Fitted)`.
Also `EngraveFreeText` (`freetext.go:179-217`), the single-face convenience
wrapper that builds a `Fitted` and calls `EngraveFitted`.

**What it actually renders, read from the code, not the name:** Body lines
(`f.Lines`, one row per string, each cut in its own `f.Faces[i]`/`f.Sizes[i]`)
plus, when non-empty, a title row and a footer row. The comment at
`fit.go:117-119` states the placement and the render code confirms it:
`EngraveFitted` calls `centerInset(f.Title, f.TitleFace, f.TitleSizeMM,
margin)` at `freetext.go:130` (`margin = params.I(outerMargin)`, i.e. the
plate's very top) and, only `if f.Footer != ""`,
`centerInset(f.Footer, f.FooterFace, f.FooterSizeMM, limit)` at
`freetext.go:136`, where `limit` comes from `yBudget` (`fit.go:192-202`) and
equals `footerRowY`, the footer's own top y — i.e. the last row. Quote
(`fit.go:117-121`):

```go
// The screw-hole rows. Title takes plate row 0 and Footer the last row,
// each when non-empty. Their faces are the faces of the blocks they border
// -- see FitBlocks.
Title, Footer         string
TitleFace, FooterFace *vector.Face
```

Title and footer are engraved **verbatim** — never through `TitleString`
(see mechanism-1/mechanism-4 comparison question below). Quote
(`freetext.go:14-19`):

```go
// f.Title and f.Footer are engraved VERBATIM -- never through TitleString,
// which upper-cases and truncates at 18, so it would engrave something the
// operator never approved. Row 0 and the last row are both screw-hole rows, and
// what keeps their ink clear of the holes is the 18-character CAP, measured at
// every rung by TestTitleCapFitsAtEveryRung...
```

**Length/capacity rule, MEASURED:** `MaxTitleLen = 18` is defined at
`backup/backup.go:58`, but **nothing in the `backup` package itself enforces
it on `Fitted.Title`/`Fitted.Footer`.** `fitBlocksAt`, `FitSized`, and
`EngraveFitted` never compare `len(title)` or `len(footer)` to
`MaxTitleLen`/18 anywhere — confirmed by reading `fitBlocksAt` (`fit.go:332-367`),
`FitSized` (`fit.go:430-513`), and the whole of `EngraveFitted`
(`freetext.go:48-171`): its panics guard shape invariants (`len(Faces) ==
len(Lines)`, `len(Sizes) == len(Lines)`, size-vs-string-emptiness, QR
guards) — none of them bound `len(f.Title)` or `len(f.Footer)`. If a title
longer than the plate can hold reached `EngraveFitted` directly (bypassing the
UI), `centerInset` (`freetext.go:116-128`) would still compute an offset and
engrave it — no panic, no truncation, no error, just an ink run that can
extend past the margin/into a screw-hole band.

The 18-char cap is enforced **only at the UI input layer**, in
`gui/freetext_flow.go`, as a hard **reject**, not a truncation:

```go
// gui/freetext_flow.go:1174-1179
if len(kbd.Fragment) > ftMaxLineLen {
    showError(ctx, th, what, fmt.Sprintf(
        "The %s holds %d characters and %d were entered. It sits on a screw-hole row at every size.",
        strings.ToLower(what), ftMaxLineLen, len(kbd.Fragment)))
    continue
}
```

`ftMaxLineLen = backup.MaxTitleLen` (`gui/freetext_flow.go:28`). So: at the
UI layer, overflow is a **hard error dialog that blocks proceeding** (the
operator must shorten the text); at the render layer (`backup` package),
overflow is **unchecked** — nothing there refuses, truncates, wraps, or clips.

**Callers/flows (grep, both bare and prefixed forms checked):**

```
$ grep -rn "FitBlocks(\|FitBlocksAt(\|FitSized(\|EngraveFitted(\|EngraveFreeText(" --include="*.go" . | grep -v _test.go
backup/fit.go:279:func FitBlocks(...)
backup/fit.go:310:func FitBlocksAt(...)
backup/fit.go:430:func FitSized(...)
backup/freetext.go:48:func EngraveFitted(...)
backup/freetext.go:179:func EngraveFreeText(...)
backup/freetext.go:203:  return EngraveFitted(params, Fitted{...})   // EngraveFreeText's own body
gui/freetext_flow.go:424:  return backup.FitSized(...)
gui/freetext_flow.go:427:  return backup.FitBlocksAt(...)
gui/freetext_flow.go:429:  return backup.FitBlocks(...)
gui/freetext_flow.go:1449: return toPlate(backup.EngraveFitted(params, fitted), params)
gui/preview.go:183:  p, err := toPlate(backup.EngraveFitted(params, fitted), params)
gui/freetext_proof.go:325:  if _, err := backup.FitBlocksAt(...); err != nil {
```

All non-test call sites are inside **one flow**: the free-text plate program
(`engraveTextFlowFrom`, `gui/freetext_flow.go:1485`, reached from
`gui/gui.go:2263-2268` for the `freeTextScan` record class), plus its preview
(`gui/preview.go`) and its proof/self-check tooling
(`gui/freetext_proof.go`). **Count: 1 flow, 3 files, ~7 call sites.**
`Fit`/`Admissible`/`AdmissibleBlocks`/`AdmissibleSized` (fit-only, no
engrave) have no production callers outside this same flow either
(`gui/freetext_flow.go:345,347`).

**Physical placement:** Title = plate row 0 (top screw-hole row); Footer =
last row (bottom screw-hole row) — both explicit in code (`fit.go:117-119`,
`freetext.go:130,136`, `yBudget`/`footerRowY` at `fit.go:166-202`).

---

### Mechanism 2 — `Seed.Title` / `SeedString.Title`

**Defined:** `backup/backup.go:16-24` (`Seed.Title`, line 17) and
`backup/backup.go:26-31` (`SeedString.Title`, line 27).

**Render path:** `engraveSeedString` (`backup/backup.go:168-232`, title block
at `backup.go:222-230`) and `frontSideSeed` (`backup/backup.go:234-320`,
title block at `backup.go:310-318`). Both render the SAME shape: title is
**upper-cased in place** (`title := strings.ToUpper(plate.Title)`,
`backup.go:223` / `backup.go:311`), engraved at the fixed
`plateSmallFontSize` (3.0mm, `backup.go:164`), and **horizontally centred**
on the plate at a y computed from the content block's own height —
`offy := (plateDims.Y+col1Height)/2 + metaMargin` — **not** a fixed row-0/
last-row screw-hole position the way mechanism 1 is. There is no separate
"footer": only a `Title` field exists on both types, and a
`MasterFingerprint` band (when non-zero) sits symmetrically ABOVE the
content block (`offy := (plateDims.Y-col1Height)/2 - metaMargin`,
`backup.go:194` / `backup.go:271`).

**Length/capacity rule, MEASURED:** No cap is enforced in `engraveSeedString`
or `frontSideSeed` — `title` is engraved verbatim (after `ToUpper`) with no
length check, no truncation, no error return keyed to title length.
Production callers keep it short by construction/comment rather than by
code: `gui/slip39_polish.go:492` sets `Title:
fmt.Sprintf("%d #%d/%d", scan.Identifier, scan.MemberIndex+1,
scan.MemberThreshold)` with an inline comment `// max "32767 #16/16" = 12 <=
MaxTitleLen 18` — a human-verified bound, not a checked one.

**Callers, grepped (bare `Seed{`/`SeedString{` inside `backup` itself: none
outside tests):**

```
$ grep -rn "backup\.Seed{" --include="*.go" . | grep -v _test.go
gui/slip39_polish.go:488   (Title set, from a SLIP-39 share identifier/index)
gui/gui.go:627             (Title NOT set -- the ordinary BIP-39 seed plate carries no title today)

$ grep -rn "backup\.SeedString{" --include="*.go" . | grep -v _test.go
gui/unlock_session.go:196  (Title = id, the codex32 identifier)
gui/codex32_polish.go:232  (Title = id, same pattern, different flow -- backupSeedStringFlow)
```

**Count: 4 production call sites across 4 flows** (BIP-39 seed engrave,
SLIP-39 verbatim-share engrave, codex32 unlock-session engrave, codex32
"Engrave" choice) — only 3 of the 4 actually populate `Title`.

**Physical placement:** centred, immediately below the mnemonic/seed
columns (title) or immediately above them (fingerprint), at a y that moves
with content height — not screw-hole-row-anchored.

---

### Mechanism 3 — the passphrase plate's `topLines` / `bottomLines` banding

**Defined:** `backup/passphrase.go`. `passphraseLayout.topLines,
bottomLines []string` at `passphrase.go:133-134`. Populated in
`passphraseLayoutFor` (`passphrase.go:160-215`):

```go
// passphrase.go:176-187
if plate.SeedFP != "" {
    l.topLines = append(l.topLines, "SEED FP: "+passphrase.GroupFingerprint(plate.SeedFP))
}
if plate.CombinedFP != "" {
    l.topLines = append(l.topLines, "EXPECTED COMB FP: "+passphrase.GroupFingerprint(plate.CombinedFP))
}
if strings.ContainsRune(plate.Passphrase, ' ') {
    l.bottomLines = append(l.bottomLines, passphraseLegend)
}
if plate.SeedFP != "" || plate.CombinedFP != "" {
    l.bottomLines = append(l.bottomLines, passphraseFooter)
}
```

`passphraseFooter = "FINGERPRINTS TYPED, NOT VERIFIED"` (`passphrase.go:156`).

**Discrepancy from the task's background text, measured directly:** the
task background states the combined-fingerprint line as `"COMB FP: FC60
C6DF"` (18 chars). The current source's actual label is **`"EXPECTED COMB
FP: "`**, not `"COMB FP: "`. Measured lengths (`printf '%s' … | wc -c`):

| line | text | length |
|---|---|---|
| `"SEED FP: A1B2 C3D4"` | matches the task's stated form | **18** |
| `"EXPECTED COMB FP: A1B2 C3D4"` | current source, NOT `"COMB FP: …"` | **27** |
| `"FINGERPRINTS TYPED, NOT VERIFIED"` | `passphraseFooter` | **32** |

`FingerprintLen = 8` (`passphrase/passphrase.go:45`) is fixed, and
`SeedFP`/`CombinedFP` "arrive canonical from `fingerprintEntryFlow`, which
is the precondition `backup.Passphrase` documents but does not check"
(`gui/passphrase_flow.go:539-540`) — so these band strings are effectively
**fixed-format** (constant label + 8 hex digits), not free text, which is
why nothing in `passphrase.go` bounds their length: the format itself is
the bound, by convention, not by a runtime check.

**Render path (the `band` closure), quoted in full**
(`passphrase.go:227-235`):

```go
// band engraves centred metadata lines downwards from y.
band := func(t engrave.Transform, y int, lines []string) {
    for i, line := range lines {
        s := engrave.String(plate.Font, l.smallEm, line)
        w, _ := s.Measure()
        t.Offset((plateX-w)/2, y+i*l.smallEm)
        s.Engrave(t.Yield)
    }
}
```

called as `band(t, l.topY, l.topLines)` then `band(t, l.bottomY,
l.bottomLines)` (`passphrase.go:238,249`), around the body text block. It
takes an arbitrary `[]string` (not a single title/footer string) and stacks
each line downward by `l.smallEm` (`plateSmallFontSize`, 3.0mm) — genuinely
a multi-line band, unlike mechanisms 1/2/4 which each carry at most one
title line and one footer line.

**Length/capacity rule, MEASURED:** No hard cap constant, no truncation, no
error on overflow anywhere in `band` or `passphraseLayoutFor`. The **only**
statement of a line budget is a **comment**, not a check
(`passphrase.go:171-174`):

```go
// The bands are the 10mm margins, matching existing practice: the master
// fingerprint and title already sit in exactly those bands. At most two
// lines fit -- a band offers innerMargin 10 - outerMargin 3 = 7mm, and
// three 3mm lines need 9mm and run off the plate edge (spec 4.3).
```

This is asserted nowhere in code for `topLines`/`bottomLines` counts —
`band` will happily render a 3rd line at `y+2*l.smallEm` past the stated 7mm
budget with no refusal. (It stays safe in practice only because
`topLines`/`bottomLines` are populated from at most 2 fixed sources each,
per the `if` list above — never from unbounded user text.)

**Callers:** `backup.Passphrase{` literal — one production call site,
`gui/passphrase_flow.go:550` (`ppBuildPlate`), one flow
(`engravePassphraseFlow`).

**Physical placement:** `topY = params.F(outerMargin)` (top margin, i.e.
row 0 in device units — `passphrase.go:169`); `bottomY = params.F(85 -
innerMargin)` = 75mm from the top (`passphrase.go:175`) — a fixed offset
from the bottom edge, explicitly said (in a comment, not shared code) to
match "the master fingerprint and title" bands mechanism 2 uses.

---

### Mechanism 4 — `backup.Text` / `Text.Paragraphs` (the one `md1`/`mk1` use)

**Defined:** `backup/backup.go:33-41`:

```go
type Text struct {
    Paragraphs []Paragraph
    Font       *vector.Face
    // FontSize is the text size in millimeters. Zero means
    // plateFontSizeUR, which is what every descriptor and mdmk caller
    // constructs...
    FontSize float32
}
```

**No `Title` or `Footer` field exists on `Text` at all** — see the three-part
question below.

**Render path:** `backup.EngraveText` (`backup/backup.go:350-446`), read in
full. It iterates `plate.Paragraphs`, and for each: resolves an optional QR
placement (`qrPlaceAt`), builds a `textLayout` (the SAME helper mechanism 1
uses — see the structural-closeness question below), wraps `p.Text` with
`WrapText`, and engraves the wrapped lines starting at a running `offy` that
begins at `params.I(outerMargin)` (`backup.go:361`) and, between paragraphs,
advances by `params.I(1)` (1mm gap, `backup.go:440-443`). **There is no
title row and no footer row rendered anywhere in this function** — every
line comes from `WrapText(p.Text, …)`; nothing reserves a row for a fixed
label.

**Length/capacity rule, MEASURED:** No `MaxTitleLen`/18 cap applies at all
— `Text`/`Paragraph` text is explicitly UNBOUNDED at this layer. Quote
(`backup.go:386-392`, inside `EngraveText`):

```go
// The descriptor and mdmk callers (validateDescriptor,
// validateMdmk) keep an UNBOUNDED path here: they offer
// whichever of TEXT+QR / TEXT-ONLY / QR-ONLY fit, which
// depends on toPlate rejecting overflow, so a maxLines
// refusal here would silently change which variants they
// offer.
```

Overflow behaviour: `WrapText` produces as many lines as needed (bounded
only by `math.MaxInt` passed as the max, `backup.go:417`); if the composed
plate doesn't fit, the caller's `toPlate(plan, params)` call is what
refuses it (an **error**, not a truncation/wrap/clip at this layer) — that
refusal is measured elsewhere in the same doc comment: "TEXT+QR fails first
(works through 268 chars, fails at 269), then QR-ONLY (641, fails at 642),
… TEXT-ONLY fails LAST (645, fails at 646)" (`backup.go:397-403`).

**Callers, grepped (bare `Text{` inside `backup` package: none in
production):**

```
$ grep -rn "backup\.Text{" --include="*.go" . | grep -v _test.go
gui/gui.go:546   (validateDescriptor)
gui/gui.go:2308  (validateMdmk)
```

**Count: 2 production call sites, 2 flows** — `validateDescriptor`
(`gui/gui.go:515-559`) and `validateMdmk` (`gui/gui.go:2288-2336`, the
target of this recon). Both build the same 3-variant TEXT+QR/TEXT-ONLY/
QR-ONLY set from a single `Paragraph`.

**Physical placement:** paragraphs stack top-to-bottom starting at
`outerMargin` from the top; no row is reserved anywhere for metadata — a
title/footer/status line would have to either become a `Paragraph` itself
(competing for wrap budget with the body) or the mechanism would need new
fields.

---

### The three specific questions

**Q1 — Does `backup.Text` have ANY unused or partially-implemented
title/footer capability already?**

**None.** `Text` (`backup.go:33-41`) has exactly three fields:
`Paragraphs`, `Font`, `FontSize`. `EngraveText`
(`backup.go:350-446`, read in full) has no `Title`/`Footer` local, no dead
branch, no commented-out band code, and no unused struct field. Confirmed
by reading the whole function body, not by grepping for the word "title" —
the string "title"/"Title" does not appear anywhere in `backup.go`'s `Text`/
`Paragraph`/`EngraveText` code path (it appears only in the unrelated
`Seed`/`SeedString`/`TitleString` code in the same file). This is stated as
"I found none" after reading the full definitions and render function, not
"I did not check."

**Q2 — How close are mechanisms 3 and 4 structurally? Could `Text` adopt the
passphrase plate's banding without a new rendering primitive?**

Mechanisms **1 and 4 are close**, mechanism **3 is independent of both**:

- `EngraveText` (mechanism 4) calls `textLayout` at `backup.go:384` and
  `qrPlaceAt` (via the paragraph's own `qrPlaceAt(...)` call at
  `backup.go:375`) — the exact same helpers `fitBlocksAt`/`wrapBlocks`
  (mechanism 1, `fit.go:231`) and `EngraveFitted`'s `centerInset`
  (mechanism 1, `freetext.go:121`) use. Both are defined once, in
  `backup/wrap.go` (`textLayout` at `wrap.go:224`, `qrPlaceAt` at
  `wrap.go:196`, `lineLayout`/`qrPlacement` types at `wrap.go:118,176`).
  Mechanisms 1 and 4 **share** the screw-hole/QR-narrowing layout engine and
  `WrapText` (`wrap.go`, used by both `wrapBlocks` and `EngraveText`).

- The passphrase plate's `band` closure (mechanism 3,
  `passphrase.go:228-235`, quoted above) calls **none** of
  `textLayout`/`qrPlaceAt`/`WrapText`. It is a self-contained ~8-line
  closure: fixed small font size, plain centred `engrave.String` +
  `Measure()`, no screw-hole clamp, no QR-narrowing, no wrap — because its
  inputs are fixed-format label+hex strings that never need wrapping or a
  hole-avoidance budget the way free-running body text does.

So: **`Text` (mechanism 4) could reuse mechanism 1's `textLayout`/
`qrPlaceAt`/`WrapText` primitives directly** (it already imports and calls
two of the three) to build a title/footer-like band, since they are the
same package-private helpers. It **cannot** "adopt the passphrase plate's
banding" as such, because that banding is not a shared primitive at all —
it is a private, independent implementation local to `passphrase.go` that
`Text`/`EngraveText` does not call and was not written to be reusable
(no exported function, takes a `passphraseLayout`-shaped y/lines pair
inline in a closure, not a general one).

**Q3 — Is `MaxTitleLen`'s truncation genuinely silent? Does anything else
call `TitleString` besides the mechanism-1 path?**

`TitleString`, read in full (`backup/backup.go:98-110`):

```go
func TitleString(face *vector.Face, s string) string {
    s = strings.ToUpper(s)
    res := ""
    for _, r := range s {
        if _, _, valid := face.Decode(r); valid {
            res += string(r)
        }
        if len(res) == MaxTitleLen {
            break
        }
    }
    return res
}
```

Yes — genuinely silent: **no error return** (`func` returns only
`string`), **no log call**, **no panic**. Two silent behaviours, both
unconditional: (a) any rune the face cannot decode is dropped, not
substituted or reported; (b) once `len(res)` hits 18, the loop `break`s
with no signal that truncation occurred.

**And it is not even used by mechanism 1** — the doc comment on
`EngraveFitted` says so explicitly (quoted above: "never through
`TitleString`") and grepping every call site confirms it has **zero
production callers anywhere in the repository**:

```
$ grep -rn "TitleString" --include="*.go" .
backup/freetext.go:14      -- doc comment explaining mechanism 1 does NOT call it
backup/backup.go:98        -- the function definition
backup/backup_test.go:339  -- TestTitleString, a test of the function itself
backup/passphrase.go:18    -- doc comment (unrelated: explains SpaceMark's codepoint choice)
cmd/vectorfont/main.go:795 -- doc comment, not a call
gui/freetext_sizeproof_table_test.go:566 -- test-only differential check ("and it survives TitleString unchanged")
backup/freetext_test.go:138,146,158 -- TestTitleAndFooterAreEngravedVerbatim, asserting mechanism 1 does NOT go through it
```

Every one of the 8 hits is either the definition, a doc comment, or a test.
**`TitleString` has no production caller at all** — not in mechanism 1, 2,
3, or 4. The three production `Title` producers that exist today
(`gui/slip39_polish.go:492`, `gui/unlock_session.go:196`,
`gui/codex32_polish.go:232`, all mechanism 2) each construct their title
string directly (`fmt.Sprintf`, or a codex32 `id`) and never call
`TitleString`.

---

## PART 2 — golden/snapshot test inventory

### The two golden mechanisms

**1. `golden.CompareBSpline`** (`internal/golden/golden.go:19-70`) — compares
an engraved plate's B-spline output against a `.bin` file
(gzip + varint-delta encoded knot list, `encodeBSpline`/`decodeBSpline`,
`golden.go:84-151`). Regeneration: an `update bool` parameter, sourced in
every calling package from a package-local `flag.Bool("update", false, …)`
var — **not** an env var, **not** a script (except see the oracle case
below). Confirmed call sites (grep, `CompareBSpline` and its `flag.Bool`
declarations):

```
backup/backup_test.go:29    var update = flag.Bool("update", false, "update golden files")
backup/backup_test.go:393   golden.CompareBSpline(p, *update, t.ArtifactDir(), sw, bounds, spline)
backup/freetext_test.go:240 golden.CompareBSpline(p, *update, t.ArtifactDir(), prodParams.StrokeWidth, bounds, spline)
engrave/engrave_test.go:28  var update = flag.Bool(...)
engrave/engrave_test.go:177,253  golden.CompareBSpline(...)
gui/freetext_sizeproof_golden_test.go:16  var update = flag.Bool(...)
gui/freetext_sizeproof_golden_test.go:113 golden.CompareBSpline(...)
```

**2. Package-local PNG image goldens** — `gui/op/draw_test.go:22`
(`var update = flag.Bool("update", false, "update golden images")`),
`testGolden` (`draw_test.go:138-182`) writes/reads `testdata/<name>.png`
directly (no shared helper); same pattern independently in
`bspline/bspline_test.go:21` / `compareImages` (`bspline_test.go:304`).

**3. `s2_md1_golden.expect.json`** (`gui/testdata/`, referenced at
`gui/multisig_build_oracle_test.go:48`) — a **string-content** golden (the
device-assembled `md1` chunks compared byte-for-byte against a committed
oracle-produced JSON), regenerated by a dedicated script, not a `-update`
flag on the test binary:

```
scripts/oracle-live.sh:6    #   ./scripts/oracle-live.sh -update    re-mint the S2 md1 golden
scripts/oracle-live.sh:101  -run '^TestAssembledMd1MatchesThePrimaryByteForByte$' ./gui/ -update
```

This is a wire-format/content golden, not a plate-**layout** golden —
orthogonal to S6b, which is about the rendered plate, not the `md1` string
itself.

### Inventory by location

| location | count | files |
|---|---|---|
| `backup/testdata/*.bin` | **16** (`ls backup/testdata/*.bin \| wc -l` = 16) | see table below |
| `gui/testdata/sizeproof-{front,back}.bin` | **2** | `sizeproof-front.bin`, `sizeproof-back.bin` |
| `gui/op/testdata/*.png` | **4** | `alpha-mask.png`, `image-mask.png`, `paletted.png`, `rounded-rect.png` |
| `bspline/testdata/curves.png` | 1 | curve-rendering utility golden (unrelated to plates) |
| `engrave/testdata/*.bin` | 2 | `font-constant.bin`, `font-sh.bin` — per-glyph identity goldens for the two faces, driven by `engrave/engrave_test.go`; pin **glyph shape**, not paragraph/band layout |
| `gui/testdata/s2_md1_golden.expect.json` | 1 | string-content golden, see above |

Everything under `address/testdata`, `uf2/testdata`, `seedxor/testdata`,
`md/testdata`, `picobin/testdata`, `slip39/testdata`, `seal/testdata`,
`sysw/testdata` is protocol/codec conformance vectors (JSON/binary), not
visual/plate-layout goldens — checked, not merely assumed absent (each
directory listed; contents are `vectors.json`, `*.uf2`, `README.md`, etc.,
consistent with wire-format fixtures).

### `backup/testdata`'s 16, mapped to the driving test and mechanism

| golden | driving test | mechanism |
|---|---|---|
| `text-0-shards-1.bin`, `text-1-shards-1.bin`, `text-2-shards-1.bin` | `TestText` (`backup/backup_test.go:150-179`) | **Mechanism 4** (`backup.Text`/`EngraveText`) — directly what an `md1`/`mk1` status-line change would churn |
| `seed-0-words-24.bin`, `seed-1-words-12.bin` | `TestSeed` (`backup_test.go:181-201`) | Mechanism 2 (`Seed`/`frontSideSeed`) |
| `slip39-0.bin`, `slip39-23-words.bin`, `slip39-33-words.bin` | `TestSLIP39` and friends (`backup_test.go:203+`) | Mechanism 2 (`Seed`/`frontSideSeed`, SLIP-39 path) |
| `codex32-0.bin`, `codex32-1.bin` | `TestCodex32` (`backup_test.go:283+`) | Mechanism 2 (`SeedString`/`engraveSeedString`) |
| `passphrase-0-plain.bin`, `passphrase-1-qr.bin`, `passphrase-2-no-metadata.bin`, `passphrase-3-max-qr.bin` | `TestPassphraseGolden` (`backup/passphrase_test.go:669-708`) | Mechanism 3 (`Passphrase`/`engravePassphrase`) |
| `freetext-0-plain.bin`, `freetext-1-qr.bin` | `TestFreeTextGoldens` (`backup/freetext_test.go:248+`), via `EngraveFreeText` | Mechanism 1 |

3 + 2 + 3 + 2 + 4 + 2 = 16, matching the measured `ls` count exactly.

**Regeneration for these 16, quoted policy** (`gui/freetext_sizeproof_golden_test.go:14-16,63-64`):

```
// update re-records the goldens in this package. Scope it with -run: a bare
// `go test ./... -update` also rewrites backup's sixteen, and those are frozen.
...
// Those sixteen goldens are FROZEN: a moved byte is a finding, and -update has
// never been run on them.
```

This "frozen" status is **policy stated in a comment, not enforced by
code** — grepped for "FROZEN"/"frozen" across `backup/*.go` and the golden
test file; the only hits are the comment above and its two restatements
(`gui/freetext_sizeproof_golden_test.go:15,63,126`). Nothing refuses `go
test ./backup -update`; a change to `EngraveText` that intentionally moves
the 3 `text-*.bin` goldens would re-record them the same way any other
golden is re-recorded (`go test ./backup -run TestText -update`), and the
project's own stated review posture treats each moved byte there as a
finding to justify, not as routine.

### `gui/testdata/sizeproof-{front,back}.bin` — the OTHER kind

Driven by `TestSizeLadderGoldens` (`gui/freetext_sizeproof_golden_test.go:85-133`),
over `EngraveFitted` (mechanism 1) via `ftLadderPlate`. Unlike the 16
above, these are **designed to move**: quoted (`freetext_sizeproof_golden_test.go:62-67`):

```
// THE CONTRACT HERE IS NOT backup/testdata's, and the difference matters.
// Those sixteen goldens are FROZEN... These two exist in order to move.
// A glyph edit is supposed to move them; what they buy is that the movement
// is SEEN, attributed to the glyph that caused it, and re-recorded in the
// same commit as the edit
```

Regeneration: `go test ./gui -run TestSizeLadderGoldens -update`
(`freetext_sizeproof_golden_test.go:82`). These pin every glyph both shipped
faces can cut, at the production plate scale, on the free-text size ladder
— a change to shared layout code (`textLayout`/`qrPlaceAt`/`EngraveFitted`)
would very likely churn these two, since mechanism 1 is exactly what they
exercise.

### `gui/op/` — the pre-existing go-vet note, and what those goldens are

Confirmed present, not investigated further (out of scope per the task):
`gui/freetext_sizeproof_golden_test.go:111` (`dir := t.ArtifactDir()`) and
`gui/op/draw_test.go:176` (`n := filepath.Join(t.ArtifactDir(), name +
"-mismatch.png")`) — both lines read exactly as cited.

`gui/op/testdata/*.png` (4 files: `alpha-mask.png`, `image-mask.png`,
`paletted.png`, `rounded-rect.png`) are driven by `TestClip`, `TestImageMask`,
`TestPaletted`, `TestRoundedRect` (`gui/op/draw_test.go:90,107,125,50`) via
the shared `testGolden` helper (`draw_test.go:138-182`). Read in full: these
test the `op`/`Buffer`/`Drawer` **rasterizer primitives** (rounded-rect
fill, alpha clipping, image masking, palette drawing) at small synthetic
sizes (100x60, 80x50, 150x150) — they are neither plate-layout goldens nor
screen/UI-content goldens; they pin the low-level drawing library
independent of what is drawn with it. A plate-text layout change (adding a
band to `backup.Text`) would not touch these unless it also changed how
`op`'s primitives themselves draw.

### Plate-layout-sensitive vs not, summary

- **Directly plate-layout sensitive to an `EngraveText`/mechanism-4 band
  change:** `text-0/1/2-shards-1.bin` (3 files, `backup/testdata`).
- **Sensitive only if shared layout code (`textLayout`/`qrPlaceAt`/`WrapText`
  in `backup/wrap.go`) changes:** `freetext-0/1-*.bin` (2, mechanism 1),
  `sizeproof-front/back.bin` (2, mechanism 1, designed to move).
- **Not sensitive unless their own package changes:** `seed-*`, `slip39-*`,
  `codex32-*` (7, mechanism 2, independent render code), `passphrase-*` (4,
  mechanism 3, independent `band` closure).
- **Not plate-layout at all:** `engrave/testdata/font-{constant,sh}.bin`
  (glyph identity, not composition), `gui/op/testdata/*.png` (rasterizer
  primitives), `bspline/testdata/curves.png` (curve-drawing utility),
  `s2_md1_golden.expect.json` (md1 string content, not rendering).

---

*Method note: every claim above was read from the file, not inferred from a
name or doc comment; every count is a pasted command result; every symbol
search was run both bare and with package-qualified prefixes
(`backup.Text{` and bare `Text{`, `FitBlocks(` and its `At`/`Sized`
siblings, `TitleString` bare) to catch prefixed/sibling variants per the
task's stated grep-prefix-blindness concern. No design recommendation is
made; Part 1's `centerInset`/`band` comparison and Part 2's sensitivity
table report what each mechanism IS, not which is preferable.*
