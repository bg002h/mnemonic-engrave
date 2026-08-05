# SPEC — `SIZEPROOF!`: per-row sizing and the two-sided size-ladder plate

Status: **R2, folding R0 round 1 (RED, 1C/6I).** Author: controller session,
2026-08-05. Reviews persisted at
`design/agent-reports/bothproof-all-spec-R0-round0.md` (6C/8I) and
`design/agent-reports/sizeproof-spec-R0-round1.md` (1C/6I). Round 0's six
Criticals are confirmed fixed and are not revisited here.

Risk-set work: changes plate **layout and admission**, and `font/constant`
plates carry seeds and passphrases. No code before R0 is 0C/0I.

Renamed from `BOTHPROOF!ALL` per `LEXICON_proof_triggers.md`: the slot after
`BOTHPROOF!` already means "rung", so a content-and-side value there would be a
second trigger wearing the first one's name.

**Every number in this document was re-measured for R2** by simulating the
proposed layout against the real fonts and the real `sh2.Params()` — running
`y` in device units, one `textLayout` per block at that block's own `baseY` and
size, rows indexed from 0 within the block. Raw probe output is quoted in §1.1
and §2.4. Round 1's I5 exists because the previous draft asserted a number in a
section labelled MEASURED that nobody had measured; that sentence is corrected
in §1.2 and the correction is itself measured.

---

## 1. Goal

One plate, engraved both sides, carrying the complete 95-character sweep in
**both faces** at **five rungs**. It answers what a render cannot: which glyphs
survive as the size drops.

| side | title | blocks (face@rung: rows) | body ends | spare |
|---|---|---|---|---|
| **FRONT** | `FRONT 5.0+3.8` @3.8 | sh@5.0:4, const@5.0:5, sh@3.8:3, const@3.8:4 | 78.400 mm | **3.600 mm** |
| **BACK** | `BACK 4.4+3.4+3.0` @3.0 | sh@4.4:4, const@4.4:4, sh@3.4:3, const@3.4:3, sh@3.0:3, const@3.0:3 | 79.600 mm | **2.400 mm** |

Limit is 82.000 mm (`plateSize - outerMargin`, §2.4). The front carries 4 blocks
x 95 = **380** characters, the back 6 x 95 = **570**, so the pair is **950**. No
QR, no confusable table, no prose, **no footer** (§5). 6.0 mm is absent by
decision.

The two sides are **two independent plate programs and an operator flip.** The
firmware gains no concept of a side.

### 1.1 The measurement

Front, title at 3.8 mm, body starting at y = 6.800 mm:

| block | rows | per-row character budgets | y from → to |
|---|---|---|---|
| sh@5.0 | 4 | `[20 26 26 26]` | 6.800 → 26.800 |
| const@5.0 | 5 | `[23 23 23 23 23]` | 26.800 → 51.800 |
| sh@3.8 | 3 | `[34 34 34]` | 51.800 → 63.200 |
| const@3.8 | 4 | `[31 31 31 25]` | 63.200 → **78.400** |

Back, title at 3.0 mm, body starting at y = 6.000 mm:

| block | rows | per-row character budgets | y from → to |
|---|---|---|---|
| sh@4.4 | 4 | `[24 30 30 30]` | 6.000 → 23.600 |
| const@4.4 | 4 | `[26 26 26 26]` | 23.600 → 41.200 |
| sh@3.4 | 3 | `[38 38 38]` | 41.200 → 51.400 |
| const@3.4 | 3 | `[34 34 34]` | 51.400 → 61.600 |
| sh@3.0 | 3 | `[44 44 44]` | 61.600 → 70.600 |
| const@3.0 | 3 | `[39 31 31]` | 70.600 → **79.600** |

### 1.2 What the bands actually do — corrected

**Both bands bite, and the previous draft named only one.** The top band
narrows rows whose top is above y = 10 mm; the **bottom** band narrows rows
whose bottom passes y = 75 mm. On both sides the bottom band lands inside the
LAST block: `const@3.8` on the front is `[31 31 31 25]` and `const@3.0` on the
back is `[39 31 31]`. Neither narrowing changes a row count here — but that is a
**measured coincidence, not a structural guarantee**, and it is one of the
things §7.3 pins. The claim that "everything below the top band takes
`ceil(95/CharsPerLine)` rows exactly" is false as a rule and true only as an
outcome.

**Round 1's I5, corrected.** The previous draft claimed "six of the ten (face,
rung) pairs need a row more than the naive count". Measured: in the titled
configuration **zero of ten** exceed `ceil(95/CharsPerLine)`, and untitled
**exactly one** does — `sh@5.0`, whose budgets are `[20 20 26 26 26]`, five rows
against a naive four. RECON §3's per-pair counts were right all along; the whole
delta between RECON's 71.600/73.600 mm and this table's 78.400/79.600 mm is the
outer margin plus the title row.

**A block's height still depends on where it starts,** which is the reason the
table is measured rather than computed. On the FRONT a title makes the plate
*roomier*: with one, `sh@5.0` starts below the top band and takes 4 rows for
3.600 mm spare; without one it starts inside the band, takes 5, and spare falls
to 2.400 mm. **The inversion is front-only** — the untitled back ends at
76.600 mm for 5.400 mm of spare, more than the titled 2.400 mm, because its
first block is 4.4 mm and clears the band either way.

## 2. Per-row sizing

Today a plate has one size (`backup/freetext.go:42-46`). Size becomes uniform
**within** a block and variable **between** blocks.

### 2.1 The QR window is a y-range, and it has an ANCHOR

The previous draft claimed the per-line grid needs "no new arithmetic at all".
**Half true, and the false half engraves over the QR.** `lineLayout.at(i)`
(`backup/wrap.go:133-158`) uses `i` for two jobs:

- the band predicate, `baseY+i*fontSize` (`wrap.go:140-141`) — `baseY`-relative,
  and does survive block-relative indexing;
- `isQRLine := holeLines <= i && i < holeLines+qrLines` (`wrap.go:135`) — a
  **row index counted from the layout's own baseY**, which does not.

Measured at 3.0 mm against a 700-character descriptor text (code 89 modules,
`holeLines` 3, `qrLines` 20, band `[12.000, 72.000)` mm): a line wrapped at
plate row 3 gets **12** columns; the same line re-indexed to block-relative 2
gets **36**. Three times the width, engraved straight across the code — on an
ordinary operator plate, not on these proof plates at all.

**Round 1's Critical: the two existing consumers of that window are anchored
differently, and the spec never said which anchor survives.**

- `EngraveText` (`backup/backup.go:359, 385`) builds its layout at
  `baseY = offy` — the PARAGRAPH's top — and draws the code at
  `lay.baseY + lay.holeLines*lay.fontSize + …`. The descriptor's QR belongs to a
  paragraph and moves with it. Internally consistent.
- `EngraveFitted` (`backup/freetext.go:81-85`) draws at
  `margin + lay.holeLines*fontSize + …` — the PLATE's top margin. The free-text
  QR is one object on one plate and does not belong to any block.

They agree today only because `wrapBlocks` (`fit.go:150`) hands
`params.I(outerMargin)` to every block as `baseY`. §2.4 replaces that with the
running `y`. An implementer reaching for the field that is already there —
`baseY + holeLines*fontSize` — preserves the descriptor path and silently
re-creates the original defect on any multi-BLOCK free-text plate.

That case is reachable on shipped firmware: `BOTHPROOF!` is two blocks and drops
the QR when it loads, but `plan` and `text` survive Back by design, so the
operator can go Back to the QR screen, enable it, and trim until it fits. Block
1 would get the right window and block 2's would sit at its own `baseY` — its
rows beside the code wrapped at the full width and cut across a QR that is a
machine-readable copy of the text.

**Required fix — one placement, computed once, read by everyone.**

```go
// qrPlacement is where a code sits, in DEVICE units, and the band of y the text
// must keep out of. It is computed ONCE per plate (free text) or per paragraph
// (descriptor) and is read by BOTH the layout that narrows the lines and the
// engraver that draws the code, so the two cannot drift.
//
// fontSize is the size the band is quantised to. A plate that MIXES sizes has
// no single value here and carries no QR; see FitSized.
type qrPlacement struct {
    Top, Bottom int // the narrowed band, [Top, Bottom), PLATE-ABSOLUTE
    X, Y        int // the code's own top-left corner
    Size        int // the code's side
}

func qrPlaceAt(params engrave.Params, qrc *qr.Code, qrScale, fontSize, anchorY int) qrPlacement
```

with `Top = anchorY + holeLines*fontSize`, `Bottom = Top + qrLines*fontSize`,
`Y = Top + (qrLines*fontSize - Size)/2`, `X = plateW - Size - margin - qrBorder`
— the arithmetic that is in `textLayout`, `freetext.go:81-85` and
`backup.go:383-385` today, moved to the one place all three read.

`textLayout` takes a `*qrPlacement` in place of `(qrc, qrScale)` and keeps
`Top`/`Bottom` on `lineLayout` instead of `holeLines`/`qrLines`. The predicate
becomes, in the same form as the band predicate beside it:

```go
y := l.baseY + i*l.fontSize
isQRLine := l.qrTop <= y && y < l.qrBottom
```

**The anchor is the CALLER's decision and each keeps the one it has:**

| caller | `anchorY` | why |
|---|---|---|
| free text — `wrapBlocks`, `EngraveFitted`, `MaxCharsAtBlocks`, `rowFaces` | `params.I(outerMargin)` | one code on one plate, at the plate's top margin, whatever block a row belongs to |
| descriptor — `EngraveText` | that paragraph's `offy` | the code belongs to the paragraph and moves with it |

`EngraveText`'s QR-ONLY special case (`backup.go:390-393`: empty text centres the
code) is unchanged and stays in `EngraveText` — it is a placement override, not a
band question, and `text-2-shards-1.bin` depends on it.

**Measured equivalence (§7.7 pins it):** for every rung, with
`anchorY = outerMargin`, the plate-absolute predicate and the row-index predicate
agree on every row of the plate — 0 disagreements at 6.0, 5.0, 4.4, 3.8, 3.4 and
3.0 mm. No golden moves.

### 2.2 `Block` gains a size

```go
type Block struct {
    Face   *vector.Face
    Text   string
    SizeMM float32 // 0 = the size the plate is fitted at
}
```

Zero preserves every existing caller and golden.

**`SizeMM` is data the composition carries; the wrap never reads it** (§2.4).
Round 1's I6 asked which of `Block.SizeMM` and the passed `fontMM` wins inside
`wrapBlocks`; the answer is that the question is removed. `FitSized` is the one
function that resolves `SizeMM` into the per-block sizes it passes, and every
other entry point passes its own single rung for every block. That is what keeps
§6's admission anchor true.

### 2.3 `Fitted` gains per-row sizes and explicit title and footer sizes

```go
type Fitted struct {
    Mixed        bool      // true when the plate does NOT cut everything at one size
    SizeMM       float32   // valid only when !Mixed
    Sizes        []float32 // parallel to Lines: the size row i is cut at
    TitleSizeMM  float32   // explicit, NOT inherited from blocks[0]
    FooterSizeMM float32
    ...
}
```

`Sizes` is **always** populated, uniform plates included, so `EngraveFitted` has
one path and the existing goldens prove the general path reproduces the special
case. `Sizes` is parallel to `Lines` for the same reason `Faces` is, with the
same prohibition: **nothing downstream may re-derive it.** A
`len(Sizes) != len(Lines)` panic mirrors the existing `Faces` guard
(`freetext.go:34-39`) and is exercised by §7.15.

**`Mixed`, not `Uniform`** (round 1 Minor). The zero value has to be the safe,
legacy branch: every hand-built `Fitted` literal — including `EngraveFreeText`,
the constructor the load-bearing goldens run through — leaves it false, and false
must mean "one size everywhere", which is what those literals are.

`Mixed` is true when `Sizes`, `TitleSizeMM` (when there is a title) and
`FooterSizeMM` (when there is a footer) are **not all the same value**. So
`!Mixed` means literally every glyph on the plate is one size, which is exactly
what makes `SizeMM` printable, and a plate with a smaller title than body reports
a range rather than a number that is true of most of it.

A named boolean cannot be dereferenced by accident; a zero float can. R0's C5:
`params.F(0) == 0`, so `LinesPerPlate` divides by zero and `fixedCharWidth`
returns 0, making `textLayout`'s `width / charWidth` a second divide-by-zero —
a **panic in `ftBuildPlate` mid-flow with a plate clamped in the machine.**

**The size/string invariant** (round 1's I2). `TitleSizeMM` is non-zero **exactly
when** `Title != ""`, and `FooterSizeMM` non-zero exactly when `Footer != ""`;
every entry in `Sizes` is non-zero. `FitSized` returns an error on any violation
and `EngraveFitted` panics on it beside the `Faces` and `Sizes` guards. Without
it `LinesPerPlate(params, 0)` is `height / 0` on the **no-footer case §5
mandates** — every ladder plate, and every operator plate that skips the Footer
field — and `Title != "" && TitleSizeMM == 0` puts `centerInset` through
`textLayout` at fontSize 0. Neither is covered by any golden. §2.4 and §2.5 both
branch on the STRING being empty, so `LinesPerPlate` is never reached with 0
even before the guard fires.

`TitleSizeMM` is explicit because the title sits at the side's **smallest** rung,
not the first block's size — R0's I1. The first block's rule would give the back
a 4.4 mm title and 1.000 mm of spare.

### 2.4 `wrapBlocks` takes an explicit y budget

Today it counts rows against `end-row`, passes `params.I(outerMargin)` as
`baseY` for every block, and lets `widthFor(lay, row)` supply a plate-wide row
index. It must instead carry a running `y` in **device units**, pass that as the
block's `baseY`, and index rows from 0 within the block:

```go
func wrapBlocks(params engrave.Params, blocks []Block, sizes []float32,
    qrp *qrPlacement, start, limit int) (lines []string, faces []*vector.Face,
    rowSizes []float32, ok bool)
```

`sizes` is parallel to `blocks` and is the ONLY channel a per-block size travels
on (§2.2). `start` and `limit` are device-unit y, not row indices.

**The budget, and how the two ends compose.** R0's C3 showed the naive "refuse at
the bottom margin" reserves nothing for the footer and lays the last body block
over it. Round 1's I1 showed that the two formulas R0 handed over do not compose
either: `limit = plateHeight - margin - F(footerSizeMM)` sits BELOW the footer's
own ink. The fix is to stop computing the limit and read it off the footer:

```
start = margin + F(titleSizeMM)                 // no title -> just the margin
footerY = margin + (LinesPerPlate(params, footerSizeMM)-1)*F(footerSizeMM)
limit = footerY                                 // no footer -> plateHeight - margin
```

A block's row is admitted iff its BOTTOM is `<= limit`. **`limit` is the only
name for this quantity; `maxY` is not used.**

Measured, at every rung and every title/footer combination, against the window
`bodyRows` produces today (`start*fontSize` and `end*fontSize` off the margin) —
**24 of 24 identical, zero mismatches.** The rule is not merely safe, it is the
same rule stated in y instead of in rows, which is why no golden moves.

What the previous formula would have done, measured — the last body row's ink
against the footer's ink:

| rung | spec-as-written `limit` | footer ink top | overlap |
|---|---|---|---|
| 6.0 | 76.000 | 75.000 | 1.000 |
| 5.0 | 77.000 | 73.000 | 4.000 |
| 4.4 | 77.600 | 73.400 | **4.200** |
| 3.8 | 78.200 | 75.200 | 3.000 |
| 3.4 | 78.600 | 77.800 | 0.800 |
| 3.0 | 79.000 | 78.000 | 1.000 |

Uniform plates escape only because `bodyRows` counts rows and never consults
either formula, so no golden would have moved and nothing would have caught it.
§7.8 exercises the property on a MIXED plate with a 3.8 mm footer, where it does
not cancel.

**The unbounded sentinel survives.** `limit` is a parameter with a documented
unbounded value, because `AdmissibleBlocks` (`fit.go:280`) and `rowFaces`
(`fit.go:302`) deliberately pass `math.MaxInt` to get an untruncated count —
`fit.go:276-279` states why: *"a refusal that reported '26 / 26' for a text
needing 300 lines would tell the operator nothing about how much to cut."* A
height budget with no unbounded representation regresses that.

### 2.5 `EngraveFitted` walks a running y in device units

`fontSize`, `rows` and `bodyRows` are **removed** from `EngraveFitted` — they
have no meaning on a mixed plate and are how C5's panic arrives. Replaced by:

- `y` accumulated as `y += params.F(f.Sizes[i])` — **device units**. Accumulating
  float32 millimetres and converting once at the end drifts (20 additions of
  3.8f can land on 486399 instead of 486400) and moves every golden. For a
  uniform plate the running sum and `margin + row*fontSize` are **exactly
  equal**, because every rung converts exactly at MM = 6400.
- the title at `margin`, in `TitleFace` at `TitleSizeMM`;
- the footer at `footerY` (§2.4), in `FooterFace` at `FooterSizeMM` — the SAME
  expression the limit is read from, so the row the body is refused above and the
  row the footer is engraved on cannot be two different rows. A naive bottom
  anchor differs by the `LinesPerPlate` remainder — 1.000 mm at 3.0 mm, 3.000 mm
  at 3.8 mm — and moves every existing golden.

**The per-row screw-hole inset** (round 1's I3). `freetext.go:71` is
`lays.at(...).at(start + i)` — a plate-absolute row index into a layout built at
`baseY = margin`. Once `rows` and `start` are gone the natural rewrite keeps
`start + i`, which agrees on a uniform plate and diverges on a mixed one: the fit
computed `offx` block-relative and the engraver would compute it plate-relative,
a drift no assertion on the size, the lines or the code can see. **`EngraveFitted`
builds row i's layout at `(f.Faces[i], params.F(f.Sizes[i]), baseY = y_i)` and
reads `at(0)`.** With `y_i = margin + i*fontSize` that is byte-identical to
today's `at(i)`: the band predicate reduces to the same comparison and the QR
band is plate-absolute either way (§2.1).

### 2.6 `faceLayouts` is removed

`faceLayouts` (`fit.go:316-331`) caches per face and hardcodes
`baseY = params.I(outerMargin)`. The front is `sh@5.0, const@5.0, sh@3.8,
const@3.8` — the third block asks for `sh` and gets the cached **5.0 mm** grid
while being cut at 3.8 mm, putting the left inset of every screw-hole row in that
block out by about 0.73 mm in the wrong direction (R0's C4).

Re-keying it on `(face, fontSize, baseY)` would make `baseY` part of the key, and
§2.5 gives every ROW its own `baseY` — so the cache would never hit. **Drop it.**
`EngraveFitted` and `MaxCharsAtBlocks` build one `lineLayout` per row instead.
That is a value struct and one `Face.Decode('W')` per row, at most 26 rows on a
live per-keystroke path; `MaxCharsAtBlocks` already pays a `qr.Encode` on that
same path.

### 2.7 The new entry point

```go
// FitSized lays out a composition whose every block states its own size.
func FitSized(params engrave.Params, blocks []Block, title, footer string,
    titleSizeMM, footerSizeMM float32) (Fitted, error)
```

No ladder walk and **no `useQR`**: the QR box is placed from a single `fontSize`
(§2.1) and has no meaning without one, so `FitSized` has no parameter for it and
sets `Fitted.QR` to nil.

That is not by itself enough to stop the operator's choice being discarded
(round 1's I4). `ftStepQR` runs first, so `useQR` can be true when the trigger is
typed, and today the QR drop is prompted only via `NeedsWholePlate()`, which is
`TextQR == ""`. **Both ladder proofs therefore carry `TextQR: ""`**, so the
existing prompted drop applies verbatim: `ftProofReplaces` says *"It also REMOVES
THE QR"* before the operator can accept, and `ftProofLoader` clears `*useQR`
before resolving the outcome. §7.16 pins it.

Validation, all of it returning an error rather than laying out:

- every block carries a non-zero `SizeMM` that is a rung in `FontSizes`, the same
  guard `FitBlocksAt` applies;
- `len(blocks) > 0`;
- the §2.3 size/string invariant for the title and the footer;
- the composition fits `[start, limit]` (§2.4).

`TitleFace` is `blocks[0].Face` and `FooterFace` is `blocks[len-1].Face`, exactly
as `fitBlocksAt` does. `FitBlocks` and `FitBlocksAt` are untouched and continue
to ignore `Block.SizeMM` (§2.2).

## 3. The GUI surface — the whole path, not just the prompt

R0's C6: nothing carries per-block sizes from the trigger to the fit, so the
previous draft would have **silently engraved a uniform plate**. Six sweeps do
fit uniformly at 3.0 mm, so `FitBlocks` would have succeeded, the confirm screen
would have read "3.0mm 18 lines", and the operator would have approved a
five-rung survey and received one rung. Every one of these must change:

| site | change |
|---|---|
| `ftFaceRun` (`gui/freetext_flow.go:56`) | gains `SizeMM float32` |
| `ftPlan.Blocks` (`freetext_flow.go:107`) | stamps each block with its run's size |
| `ftProof` (`gui/freetext_proof.go:367`) | gains `Side string`; both ladder entries carry `TextQR: ""` |
| `ftProofOutcome` (`freetext_proof.go:511`) | carries the plan (which now carries the sizes), not a single rung |
| `ftRungLabel` (`freetext_proof.go:531`) | must NOT print "3.0mm" for a ladder — reads the plan's rungs |
| `ftProofReplaces` (`freetext_proof.go:538`) | names the side and its rungs, not `Plan.Name()` (§4) |
| `ftFitAt` (`freetext_flow.go:204`) | routes to `FitSized` when every block carries a size |
| `ftEvaluate` / `ftBuildPlate` | unchanged in shape; both go through `ftFitAt` |
| `ftFaceSummary` (`freetext_flow.go:146`) | groups by `(face, size)` and prints the SIZE of each run, read from `Fitted.Sizes` |
| `ftSizeLabel` (`freetext_flow.go:218`) | shows a range when `Mixed`; never `0.0mm` |
| `MaxCharsAtBlocks` (`fit.go:344`) | takes one `fontMM`, builds a layout per row (§2.6) |
| `rowFaces` (`fit.go:297`) | takes one `fontMM` |
| `gui.Preview` | gains `Sizes`; `describe` prints per-row size beside per-row face |
| `proofPreview` (`gui/preview.go:111-133`) | goes through `ftProofOutcomeFor` instead of hardcoding `ftProofFooter` |
| `previewBuilders` | two entries; `-size` must not re-fit a ladder plate at one rung |
| `sizeLabel` (`cmd/plateview/main.go:98`) | prints the range for a mixed plate; `0.0mm` stays a defect |

**A reader that prints `0.0mm` is a defect, not a fallback.** The readout and the
confirm screen are what the operator approves.

**`proofPreview` hardcodes the footer** (round 1's I4-residue). `gui/preview.go`
ends in `fittedPreviewAt(params, p.Plan, p.For(qr), p.Title, ftProofFooter, qr,
o.SizeMM)` — a literal footer for every proof, which is already wrong for the
`ftProofFooterFaceMap` case and would put a footer on a ladder plate that must
not have one (§5). `ftProofOutcomeFor` exists precisely so the prompt and the
loader cannot disagree; the preview is a third derivation of the same answer and
must use the same resolver.

**The edit path — stated precisely** (R0's M4, refined). The proof loads into an
EDITABLE field, and `ftPlan.Blocks` collapses to a single block when the text has
fewer `'\n'`-blocks than the plan has runs. Two different things follow, and only
one of them is a revert:

- **Collapse → uniform auto-fit.** The collapsed block carries no size, so
  `ftFitAt` routes to `FitBlocks` and the plate is an ordinary free-text plate.
  There is no ladder left to cut.
- **Same shape, edited characters → the ladder is KEPT.** The runs still match,
  every block still carries its size, and the plate is still cut at those sizes —
  or refused if the edit no longer fits. Reverting here would be C6 wearing a
  different hat: an operator fixing a typo would silently receive a one-rung
  plate. A refusal is visible; a size change is not.

Both are tested (§7.13).

## 4. Triggers

**`SIZEPROOF!FRONT`** and **`SIZEPROOF!BACK`**, per `LEXICON_proof_triggers.md`.

`SIZEPROOF!` bare is not a trigger: the ladder has no default half, and
defaulting would let a slip cut the wrong side onto steel already engraved.
Neither entry may ever be marked `Sizeable`, or `SIZEPROOF!FRONT4.4` becomes
ambiguous against the rung suffix parser.

The prompt must name the **side and its rungs** from the `Side` field, not from
`Plan.Name()`. R0's I5: both sides prove identical faces, so the plan name gives
near-identical prompts — and worse under the ladder, where the front's plan is
four runs and names itself `SH+CONST+SH+CONST`. A mis-pick is engraved where a
mistype is refused.

`AdmissibleBlocks` (§6) is what makes the two triggers reachable at all; the
previous draft's claim that ladder plates bypass admission was false.

## 5. Identification — DECIDED

**Titles yes, at each side's smallest rung. No footer on either side.**

- `FRONT 5.0+3.8` — 13 characters, `sh` at 3.8 mm — spare 3.600 mm
- `BACK 4.4+3.4+3.0` — 16 characters, `sh` at 3.0 mm — spare 2.400 mm

The title's FACE is `blocks[0].Face`, which is `sh` on both sides (§2.7). Both
titles are under `MaxTitleLen` (18) **and** fit the INSET span they are centred
in — measured: 13 <= 26 columns at `sh` 3.8 mm, 16 <= 36 at `sh` 3.0 mm.
`EngraveFitted` engraves `Title` **verbatim**, not through `TitleString`, so
§7.12 validates both against `MaxTitleLen`, against the inset span, and against
`TitleFace.Decode`.

**No footer**, because under §2.4's limit a footer refuses BOTH sides outright:

| side | body ends | footer top y at the title's rung | short by |
|---|---|---|---|
| FRONT | 78.400 | 75.200 (3.8 mm) | 3.200 mm |
| BACK | 79.600 | 78.000 (3.0 mm) | 1.600 mm |

The ladder proofs therefore pass an EMPTY footer and **must not inherit
`ftProofFooter`** — not from `ftProofOutcomeFor`, and not from `proofPreview`'s
hardcoded literal (§3).

On the back's 2.400 mm: acceptable **only because §7.3 pins each side's per-block
row count, per-row budgets and total height**. The hazard is discrete — a glyph
change costing `font/constant` a row at 3.0 mm moves the total by 3.000 mm, more
than any margin under discussion — so margin does not mitigate it and a test
does. Without that test 5.400 mm would not be enough either.

## 6. Invariants

- **No existing golden moves.** Uniform plates take the general path with every
  `Sizes` entry equal; §2.1's band and §2.4's limit are measured to be the same
  rules restated in y. `-update` is **not** run in this change; a moved golden
  means the general path does not reproduce the special case.
- **The unbounded `wrapBlocks` callers stay unbounded** (§2.4).
- **`AdmissibleBlocks` still runs on ladder plates, and still at 3.0 mm.** They
  load into the text field, `ftEvaluate` calls it, and `!f.ok` gates OK on both
  the text and confirm steps. It passes a uniform `sizes` at its own anchor rung
  and never reads `Block.SizeMM` (§2.2), so the readout reports the 3.0 mm
  anchor's line count — which differs from the rows actually cut. Accepted, and
  stated here because the previous draft claimed the opposite and round 1 found
  the claim unsettled.
- Run counts and per-run timing quantisation untouched; no glyph changes.
- No wire format, NDEF, codec, validation or identity change.
- `outerMargin` and `toPlate`'s safety margin are both 3 mm. They are
  **coincidentally equal and are not independent checks**; the front at 3.600 mm
  clears both by the same number.

## 7. Test plan

1. **Every existing plate golden, byte for byte** — the load-bearing test.
2. **Rows are cut at the sizes claimed** — decode the engraving and assert row
   i's glyph height matches `Sizes[i]`.
3. **Per side, a table pinning per block: face, size, row count, per-row
   character budgets and device-unit y-range**, plus the total — §1.1's two
   tables, verbatim. This is what makes 2.400 mm safe, what a font change must
   fail against, and what pins the bottom-band narrowing that §1.2 shows is a
   coincidence rather than a rule.
4. **Character SET per `(rung, face)` pair** — not per plate. "95 characters in
   both faces" is satisfiable by a full sweep at one rung and a truncated one at
   another.
5. **A block starting inside the top band is inset; one below it is not** —
   mixed sizes are exactly where a plate-wide row index mis-insets.
6. **The same face at two different sizes on one plate** (the front is one):
   each row's grid and left inset computed at that row's own size. Catches C4.
7. **The QR window is a plate-absolute y-range.** Three assertions, because one
   is not enough: (a) at every rung, the band predicate and the old row-index
   predicate agree on every row of a uniform plate — the no-golden-moves proof;
   (b) a TWO-BLOCK plate with a QR wraps block 2's rows at the code's own budget,
   and no body ink enters the code box — the round-1 Critical, unreachable by any
   single-block fixture; (c) `freetext.go`'s and `backup.go`'s QR offsets both
   read `qrPlacement.Y` — asserted by construction, since neither may compute a
   y of its own.
8. **Footer and last body row are disjoint on a MIXED plate with a 3.8 mm
   footer**, where §2.4's two formulas do not cancel, and a composition one row
   too tall for its title+footer is refused. Replaces the previous draft's "no
   row overlaps its neighbour", which was a theorem, not a property.
9. **`EngraveFitted` on a `Mixed` plate does not panic.** Catches C5.
10. **End-to-end through `freetextPlateHook`**: `SIZEPROOF!FRONT`/`BACK` produce
    a `Fitted` with the expected `Sizes` vector and `Mixed == true`. The only
    thing that catches C6.
11. **Unbounded `wrapBlocks` callers still report untruncated counts.**
12. **Titles fit `MaxTitleLen`, fit the inset span of their own face and size,
    and decode in `TitleFace`.**
13. **The edit path, both halves** (§3): a collapsed composition reverts to
    uniform auto-fit; a same-shape edit keeps the ladder and is refused rather
    than re-fitted when it no longer fits.
14. **The size/string invariant** (§2.3): `FitSized` refuses a title with a zero
    size and a footer with a zero size, and `LinesPerPlate` is never called with
    0 on the no-footer path.
15. **The `len(Sizes) != len(Lines)` guard panics** — round 0's I8(e), still
    open after round 1.
16. **Both ladder proofs report `NeedsWholePlate()`**, the prompt says the QR is
    removed, and loading one with the QR on clears `useQR` before the outcome is
    resolved.
17. **The confirm screen fits.** `ftFaceSummary` for the front is four
    `(face, size)` runs; assert the rendered line fits the real panel by
    MEASURING RECTANGLES, as `ftProofBody`'s test does — `ExtractText` collects
    runes regardless of occlusion, so a label drawn off the panel reads as
    present.
18. **`proofPreview` and `cmd/plateview` agree with the device** — the preview
    resolves through `ftProofOutcomeFor`, and no proof preview carries a footer
    the device would not engrave.
19. **Every one mutation-tested.** Four false-passing tests reached review across
    the last two cycles; assume more will try.

## 8. Non-goals

- The 6.0 mm rung.
- Generalising auto-fit to mixed sizes; `FitBlocks` stays as it is.
- Any notion of plate sides in firmware.
- A QR on these plates — `FitSized` has no parameter for one (§2.7).
- Rendering both sides as one image in `cmd/plateview`; two invocations.
- The `FONTPROOF!` → `PASSPROOF!` rename; its own change, operator's call.
