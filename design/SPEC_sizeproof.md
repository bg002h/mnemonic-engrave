# SPEC — `SIZEPROOF!`: per-row sizing and the two-sided size-ladder plate

Status: **R6 — R0 GREEN (0 Critical / 0 Important) at round 4; one test-plan
item corrected in flight at P3.** Author: controller session, 2026-08-05.

**§7.20's QR half was a measured false PASS** and is rewritten. Four review
rounds and six reviewers read it and none caught that the case it prescribed
cannot fail against the defect it names; P3's implementer found it by sweeping
900 text lengths against a simulated defect. The correction adds a third case and
changes no design, so it does not re-open this gate — but it is the clearest
evidence in this document that **a test item is a claim, and reading one proves
nothing about whether it can fail.**

The gate ran five times. Reviews persisted verbatim, each before its fold:

| round | verdict | reviewers |
|---|---|---|
| 0 (`bothproof-all-spec-R0-round0.md`) | RED 6C/8I | opus |
| 1 (`sizeproof-spec-R0-round1.md`) | RED 1C/6I | opus |
| 2 (`sizeproof-spec-R0-round2.md`) | RED 0C/5I | opus + sonnet, opus synthesis |
| 3 (`sizeproof-spec-R0-round3.md`) | RED 0C/2I | opus + sonnet, opus synthesis |
| 4 (`sizeproof-spec-R0-round4.md`) | **GREEN 0C/0I** | opus + sonnet, opus synthesis — **both lanes GREEN independently** |

Every finding from rounds 0-3 is folded. Round 4's six Minors and Nits are folded
inline in this revision; a 0C/0I re-review closes the loop and does not earn
another round. **Nothing here is re-opened by later work** — the next artifact is
an implementation plan, which carries its own R0 gate.

Risk-set work: changes plate **layout and admission**, and `font/constant`
plates carry seeds and passphrases. No code before R0 is 0C/0I.

Renamed from `BOTHPROOF!ALL` per `LEXICON_proof_triggers.md`: the slot after
`BOTHPROOF!` already means "rung", so a content-and-side value there would be a
second trigger wearing the first one's name.

**Every number in this document has been measured against the real fonts and the
real `sh2.Params()`** — running `y` in device units, one `textLayout` per block
at that block's own `baseY` and size, rows indexed from 0 within the block —
and every one of them has been **independently reproduced by two reviewers**
(round 2, both lanes). Raw measurements are quoted in §1.1, §2.4 and §2.6.

That sentence is stated carefully because R2 made the same mistake twice over.
Round 1's I5 was a number asserted in a section labelled MEASURED that nobody
had measured; R2 corrected it — and then carried §2.6's "about 0.73 mm" over
from R1 unmeasured, under a preamble claiming everything had been re-measured.
Round 2 measured it: **0.119 mm, and in the opposite direction.** The lesson is
not "measure more", it is that a blanket claim of having measured is itself a
claim, and inherited numbers are the ones that survive re-drafting unchecked.
§2.6 now carries the measurement and the derivation.

**A second-order lesson from rounds 2 and 3, which every Important since round 1
shares:** the spec repeatedly described what the code does from a doc comment,
from an adjacent function, or from a partial trace, rather than from the code —
and was wrong each time. `ftPlan.Blocks` does not collapse the way R2's §3
claimed; `wrapBlocks`' existing callers do not translate the way R2's §2.4
assumed; two size-reading sites were missing from a table introduced as complete;
and R3's replacement predicate for the first of those was traced only over
DELETED newlines, so it was blind to inserted ones. Where this document now
states what shipped code does, it cites the line, and the behaviours that decide
something are given as measured tables (§1.1, §3.1) rather than as prose.

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
firmware gains no RELATIONSHIP between them — no pairing, no flip prompt, no
ordering, no notion that one plate has another side. It does gain a `Side` LABEL
the prompt prints (§4), which is a string, not a model.

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
*roomier* — but not for the reason the previous draft gave. Both configurations
start `sh@5.0` INSIDE the top band, and its first row is narrowed either way
(`[20 …]` in both). What the title buys is the SECOND row: untitled it starts at
8.000 mm and is still inside the band, giving `[20 20 26 26 26]` and five rows;
titled it starts at 11.800 mm and clears, giving `[20 26 26 26]` and four. One
row, and with it 3.600 mm of spare instead of 2.400 mm.

**The inversion is front-only** — the untitled back ends at 76.600 mm for
5.400 mm of spare, more than the titled 2.400 mm, because its first block is
4.4 mm and takes the same four rows either way.

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
    KeepOutX    int // horizontal space the code denies a line: Size + 2*qrBorder
}

func qrPlaceAt(params engrave.Params, qrc *qr.Code, qrScale, fontSize, anchorY int) qrPlacement
```

with `Top = anchorY + holeLines*fontSize`, `Bottom = Top + qrLines*fontSize`,
`Y = Top + (qrLines*fontSize - Size)/2`, `X = plateW - Size - margin - qrBorder`
— the arithmetic that is in `textLayout`, `freetext.go:81-85` and
`backup.go:383-385` today, moved to the one place all three read.

`KeepOutX` is carried rather than re-derived because `charPerQRLine` is
`(width - 2*qrBorder - qrsz) / charWidth` (`wrap.go:181-182`), and once
`(qrc, qrScale)` are replaced by the placement, `qrBorder` is recoverable only
by inverting `X`. It happens to be the constant `params.I(2)` today, which is
exactly the kind of coincidence this spec has already been caught leaning on.

`textLayout` takes a `*qrPlacement` in place of `(qrc, qrScale)` and keeps
`Top`/`Bottom` on `lineLayout` instead of `holeLines`/`qrLines`. The predicate
becomes, in the same form as the band predicate beside it:

```go
y := l.baseY + i*l.fontSize
isQRLine := l.qrTop <= y && y < l.qrBottom
```

**The anchor is the CALLER's decision and each keeps the one it has:**

**Placement PRODUCERS** — the functions that call `qrPlaceAt` and choose an
anchor:

| producer | `anchorY` | why |
|---|---|---|
| free text — `fitBlocksAt`, `EngraveFreeText`, `MaxCharsAtBlocks`, `rowFaces` | `params.I(outerMargin)` | one code on one plate, at the plate's top margin, whatever block a row belongs to |
| **`AdmissibleBlocks`** (`fit.go:262-281`) | **`params.I(outerMargin)`** | **the same. It encodes its own code (`fit.go:269`) and so builds its own placement — see the warning below** |
| `FitSized` | — | no QR at all (§2.7); leaves `qrAt` nil |
| descriptor — `EngraveText` | that paragraph's `offy` | the code belongs to the paragraph and moves with it |

**Placement CONSUMERS** — `EngraveFitted` reads `f.qrAt` and calls `qrPlaceAt`
**never**. It is not in the table above and must not choose an anchor: it
receives only `Fitted`, and re-deriving there is the second derivation this
section exists to abolish (see below).

**`AdmissibleBlocks`' `start` and its `anchorY` are deliberately DIFFERENT
values, and this is the one place in the change where that is true.** §2.4 gives
it `start = params.I(outerMargin) + params.F(size)`, because it reserves a title
ROW unconditionally; its `anchorY` is `params.I(outerMargin)`, because the title
row moves the first row of TEXT, not the CODE. Reusing `start` — the value §2.4
hands this exact function, and the nearest one in scope — shifts the whole QR
band down by one row, so the rows at both band edges swap `charPerLine` for
`charPerQRLine`, `len(l)` moves, and `linesUsed`/`ok` change. That verdict gates
OK on the text step *and* the confirm step (`freetext_flow.go:195`) for ordinary
QR-carrying operator plates — the path seeds and passphrases take — so an
admissible plate is refused, or an inadmissible one accepted, with the readout
disagreeing with the fit. §7.20 pins it with a QR and without.

Checked: with `anchorY = params.I(outerMargin)` the new predicate reduces to
`holeLines <= 1+j < holeLines+qrLines`, term for term today's `lay.at(1+i)`.

`EngraveText`'s QR-ONLY special case (`backup.go:390-393`: empty text centres the
code) is unchanged and stays in `EngraveText` — it is a placement override, not a
band question, and `text-2-shards-1.bin` depends on it.

**The placement needs a channel from the fit to the engraver, and R2 did not give
it one** (round 2's I4). `EngraveText` can hold the placement in a local, because
it builds its own layout. `EngraveFitted` cannot: it receives only `Fitted`
(`freetext.go:33`), so with no field to read it would have to call `qrPlaceAt`
again — the second derivation this section exists to abolish — and §7.7(c)'s
"asserted by construction" would be unwritable. Worse, §2.5 removes `fontSize`
from `EngraveFitted`, so the only size left to re-derive from is `SizeMM`, which
§2.3 declares invalid when `Mixed`: a `Fitted{Mixed: true, SizeMM: 0, QR: code}`
makes `params.F(0) == 0` and `qrLines = (qrsz + 2*qrBorder + fontSize - 1) /
fontSize` an integer divide by zero — R0's C5 panic arriving through a second
door, in `ftBuildPlate`, with a plate clamped in the machine.

So `Fitted` carries the resolved placement in an unexported field (§2.3), set by
`fitBlocksAt` and by `EngraveFreeText`, left nil by `FitSized`. Unexported is
right: the only `Fitted` literals outside the package are three GUI test fixtures
(`gui/freetext_flow_test.go:564, 893, 928`) that feed the readout and never
engrave — including `:893`, which sets `QR` with no placement — and a literal
that set `QR` without a placement and then engraved it must fail loudly rather
than draw a code at a y nobody computed.

### 2.1.1 The QR guards

Three invariants. **Where each is enforced matters as much as what it says**,
because `EngraveFitted` is reached from `ftBuildPlate`
(`freetext_flow.go:643-651`) only AFTER the confirm screen: a check that belongs
at the fit but lives in the engraver is a panic mid-flow with a plate clamped in
the machine — C5's failure mode, arriving from the guard meant to prevent it.

| invariant | enforced at | why there |
|---|---|---|
| `(QR == nil) == (qrAt == nil)` | `EngraveFitted`, panic | a caller bug with no operator-facing meaning; there is no correct plate to fall back to |
| **`QR != nil` implies `!Mixed`** | `EngraveFitted`, panic | same; and `FitSized` (§2.7) makes it structurally true rather than checked |
| `qrAt.Bottom <= plateHeight - margin` | **`fitBlocksAt`/`FitSized`, error return**; re-asserted in `EngraveFitted` as a defensive panic | it is a property of the COMPOSITION, so it is refusable before the operator approves anything |

The second is the load-bearing one: the band is quantised by a single `fontSize`
via `qrLines`, so a plate that mixes sizes has no single band. Without it the
mixed-size and QR features compose into the divide-by-zero above.

The third is unreachable from any shipped fit — measured, violating it at 3.0 mm
needs roughly a 108-module code, i.e. a text over 1000 bytes, on a plate that
holds about 247 characters beside such a code, so `FitBlocks` refuses first. It
is stated anyway because "unreachable" is a property of today's rungs and today's
`freeTextQRScale`, not of the arithmetic.

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
    qrAt         *qrPlacement // §2.1: resolved once, never re-derived. nil iff QR is nil
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

**Both legacy constructors must populate the new fields** (round 2 and round 3
Minors). `fitBlocksAt` (`fit.go:224-241`) is the one every `FitBlocks` and
`FitBlocksAt` plate goes through — every ordinary operator plate — so with §2.3's
guards implemented literally and its `Fitted` literal left as it is,
`len(Sizes) == 0 != len(Lines)` panics on the first golden and any titled plate
panics on the `TitleSizeMM` invariant. It fills `Sizes` with `len(lines)` copies
of `size`, sets `TitleSizeMM`/`FooterSizeMM` to `size` when the corresponding
string is non-empty and 0 otherwise, leaves `Mixed` false, and sets `qrAt` from
`qrPlaceAt` at `anchorY = outerMargin` when there is a code.

`EngraveFreeText` needs the same treatment. It builds a
`Fitted` literal (`freetext.go:103-113`) and is the path every load-bearing
golden takes, so with §2.3's guards implemented literally and the literal left as
it is, `len(Sizes) == 0 != len(Lines)` panics on the first golden and any
non-empty title panics on `TitleSizeMM == 0`. It fills `Sizes` with `len(lines)`
copies of its single `fontMM`, sets `TitleSizeMM`/`FooterSizeMM` to that same
value when the corresponding string is non-empty and 0 when it is not, leaves
`Mixed` false, and sets `qrAt` from `qrPlaceAt` at `anchorY = outerMargin` when
`qrc != nil` and nil when it is not — §2.1.1's first guard, satisfied at the
constructor. That is what makes it the one-size case of the general path rather
than a second path.

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
`textLayout` at fontSize 0. Neither is covered by any golden.

**The guard is LOAD-BEARING, not defence in depth — corrected at P4.** An earlier
draft closed this paragraph with "§2.4 and §2.5 both branch on the STRING being
empty, so `LinesPerPlate` is never reached with 0 even before the guard fires."
That holds only for `Footer == ""`. It is **false for `Footer != "" &&
FooterSizeMM == 0`** — which is precisely the input §7.14 orders `FitSized` to
refuse. There the string branch IS taken, and `footerRowY` divides by the zero
size. Measured with the guard deleted: `FitSized` panics with an integer divide
by zero at `yBudget → footerRowY → LinesPerPlate`, **before laying anything
out.** A reader who took the old sentence at face value could have removed the
guard as redundant.

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

```go
start := margin
if title != "" {
    start += params.F(titleSizeMM)
}
limit := plateHeight - margin
if footer != "" {
    // footerY is the footer's OWN top y, and §2.5 engraves the footer at this
    // same expression. LinesPerPlate is unreachable with a 0 size because this
    // branch tests the STRING; see §2.3's invariant.
    footerY := margin + (LinesPerPlate(params, footerSizeMM)-1)*params.F(footerSizeMM)
    limit = footerY
}
```

A block's row is admitted iff its BOTTOM is `<= limit`. **`limit` is the name of
the budget's lower end; `maxY` is not used.** `footerY` names the footer's own
top y, which the footer branch assigns to `limit` and §2.5 engraves at — one
expression, two readers, so the row the body is refused above and the row the
footer is cut on cannot be two different rows. With no footer there is no
`footerY`, and `limit` is the bottom margin. The branches are shown as branches
rather than as a formula with a trailing comment, because the no-footer path is
the one §5 mandates for BOTH ladder plates and a reader who takes the formula
literally recomputes round 1's I2 divide-by-zero.

**Every existing caller needs its translation stated, and two of them do not
appear in the formula above** (round 2's I2). `start` changes from a ROW INDEX to
a device-unit y, and the existing literals stay type-correct — `int` to `int` —
so nothing forces a compile error and the wrong value is silent:

| caller | today | after |
|---|---|---|
| `fitBlocksAt` | `bodyRows(rows, title, footer)` | the block above |
| `AdmissibleBlocks` (`fit.go:280`) | `1` — one row reserved for the title, **unconditionally** | `params.I(outerMargin) + params.F(size)` |
| `rowFaces` (`fit.go:302`) | `0` — the plate top | `params.I(outerMargin)` |

`AdmissibleBlocks` reserves the title row whether or not there is a title, which
is what makes admission monotone (`fit.go:255-261`), so its translation is *not*
`start = margin + F(titleSizeMM)` — it deliberately never reads the title.

Measured at 3.0 mm, `sh`, no QR: today's `start = 1` against `baseY = margin`
puts rows at y = 38400, 57600, 76800 …, so exactly **two** fall in the top band
(y < 64000). `params.I(outerMargin) + params.F(3.0)` = 38400 reproduces that. The
literal `1` carried over gives y = 1, 19201, 38401, 57601 …, so **four** do —
each losing `2*holeChars` = 8 of 44 columns. `linesUsed` over-reports, `!f.ok`
gates OK on the text step and the confirm step, and an ordinary operator plate —
the path seeds and passphrases take — is refused needing a line count it does not
need. §7.20 pins it.

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
while being cut at 3.8 mm (R0's C4). Two things go wrong, and the previous draft
named only the smaller one, with a number it had not measured:

- **The inset is wrong by 0.119 mm.** Measured: `sh@5.0` has `charWidth`
  `4000*32000/6700 = 19104`, `holeChars = ceil(44800/19104) = 3`, inset
  `57312` = 8.955 mm; `sh@3.8` has `charWidth = 14519`, `holeChars = 4`, inset
  `58076` = 9.074 mm. The delta is 0.119 mm and the 5.0 mm inset is the
  **smaller**, so "out by about 0.73 mm in the wrong direction" was wrong in the
  magnitude and unqualified in the direction.
- **Which rows are screw-hole rows is decided at the wrong pitch, and that is the
  larger hazard.** `holeLine` is `baseY+i*fontSize < innerMargin || …`
  (`wrap.go:140-141`), evaluated at the cached 5.0 mm `fontSize` for a row cut at
  3.8 mm. A row in the 3.8 mm block can take the full ~9.07 mm inset when it
  should take none, or none when it should take ~9.07 mm — two orders of
  magnitude past the inset delta above.

Neither decides anything: dropping the cache is right on the re-keying argument
alone. The numbers are corrected because a spec that states a wrong magnitude
teaches the implementer the wrong thing to test for.

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
leaves both `Fitted.QR` and `Fitted.qrAt` nil — which is §2.1.1's
`QR != nil ⇒ !Mixed` guard satisfied structurally rather than by a check.

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

`FitSized` sets `Sizes` from the resolved per-block sizes, computes `Mixed` by
§2.3's rule, and sets `SizeMM` to the common value when `!Mixed` and 0 otherwise.
Every ladder composition is genuinely mixed, so the degenerate case never arises
in this flow — but `FitSized` is a public entry point and all-3.0 blocks with a
3.0 title are legal, whereupon hardcoding `Mixed: true, SizeMM: 0` would put
`0.0mm` on the readout, the defect §3 bolds.

## 3. The GUI surface — the whole path, not just the prompt

R0's C6: nothing carries per-block sizes from the trigger to the fit, so the
previous draft would have **silently engraved a uniform plate**. Six sweeps do
fit uniformly at 3.0 mm, so `FitBlocks` would have succeeded, the confirm screen
would have read "3.0mm 18 lines", and the operator would have approved a
five-rung survey and received one rung. Every one of these must change:

| site | change |
|---|---|
| `ftFaceRun` (`gui/freetext_flow.go:56`) | gains `SizeMM float32` |
| `ftPlan.Blocks` (`freetext_flow.go:107`) | stamps each block with its run's size, and **clears every `SizeMM` unless the text's PART count equals the plan's declared count** (§3.1); stale doc comment corrected |
| `ftProof` (`gui/freetext_proof.go:367`) | gains `Side string`; both ladder entries carry `TextQR: ""` |
| `ftProofOutcome` (`freetext_proof.go:511`) | carries the plan (which now carries the sizes), not a single rung |
| `ftRungLabel` (`freetext_proof.go:531`) | must NOT print "3.0mm" for a ladder — reads the plan's rungs |
| `ftProofReplaces` (`freetext_proof.go:538`) | names the side and its rungs, not `Plan.Name()` (§4) |
| `ftFitAt` (`freetext_flow.go:204`) | routes to `FitSized` when every block carries a size — **tested FIRST**, see below |
| `ftEvaluate` / `ftBuildPlate` | unchanged in shape; both go through `ftFitAt` |
| `ftFaceSummary` (`freetext_flow.go:146`) | groups by `(face, size)` and prints the SIZE of each run, read from `Fitted.Sizes` |
| `ftSizeLabel` (`freetext_flow.go:218`) | shows a range when `Mixed`; never `0.0mm` |
| **`ftConfirmSummary` (`freetext_flow.go:485`)** | **its own `%.1fmm` off `f.plate.SizeMM` — must print the rungs, not `SizeMM`** |
| `MaxCharsAtBlocks` (`fit.go:344`) | takes one `fontMM`, builds a layout per row (§2.6) |
| `rowFaces` (`fit.go:297`) | takes one `fontMM`; passes `params.I(outerMargin)` as `start` (§2.4) |
| `AdmissibleBlocks` (`fit.go:262`) | passes `params.I(outerMargin) + params.F(size)` as `start` (§2.4) |
| `EngraveFreeText` (`freetext.go:97`) | populates `Sizes`, `TitleSizeMM`, `FooterSizeMM`, `qrAt` (§2.3) |
| `gui.Preview` | gains `Sizes`; `describe` prints per-row size beside per-row face |
| `proofPreview` (`gui/preview.go:111-132`) | goes through `ftProofOutcomeFor` instead of hardcoding `ftProofFooter` at line 130 |
| **`fittedPreviewAt` (`gui/preview.go:149`)** | **routes through `ftFitAt`, not `backup.FitBlocks`/`FitBlocksAt` directly** |
| `previewBuilders` | two entries; `-size` must not re-fit a ladder plate at one rung |
| `ftProofLoader` (`freetext_proof.go:656-672`) | writes `*size = out.SizeMM`; the ladder proofs are not `Sizeable`, so this is **0** — see below |
| **`ftProofOutcomeFor` (`freetext_proof.go:519-526`)** | **its fallback hardcodes `Footer: ftProofFooter` for every non-`Sizeable` proof, which is both ladders. `ftProof` gains `Footer string`, defaulting to `ftProofFooter` and EMPTY for the two ladder entries; the resolver returns `p.Footer`** |
| `ftProofReplaces` (`freetext_proof.go:538`) | with an empty footer it renders "Footer becomes ." — say "the Footer is CLEARED" instead |
| `sizeLabel` (`cmd/plateview/main.go:98-103`) | its zero branch prints **"fixed layout"**, not `0.0mm`; a `Mixed` plate must print the range instead |

### 3.0 INVARIANT: a `SIZEPROOF!` plate NEVER carries a QR

**Operator directive, 2026-08-05: "Sizeproof must always be without a QR code."**
This is a hard invariant, not a default, and it holds at two levels that must not
be confused:

1. **The plate cannot carry a code.** Structural, and already true: `FitSized`
   has no QR parameter (§2.7) and leaves `QR`/`qrAt` nil, so no code exists to
   engrave whatever any flag says. **Nothing may weaken this** — a future
   `FitSized` that accepted a code would break the invariant at its root.
2. **The operator's `useQR` flag must not silently diverge from it.** *Not* true
   today, and this is the gap that produced the whole-diff review's Important.
   `ftQRChoiceFlow` (`freetext_flow.go:457`) is a bare two-choice screen that
   knows nothing about what is loaded, and it deliberately preserves a prior
   opt-in across Back (`if prior { cs.choice = 1 }`). So the operator can load a
   ladder — whose loader clears the flag, with a prompt saying so — then press
   Back and set it again. The plate still carries no code, but the flag now says
   otherwise, and everything reading the flag instead of the fit goes wrong: the
   confirm screen did (fixed at P5, §3.2), admission did (fixed after the
   whole-diff review).

**Ignoring the flag is not sufficient.** Silently discarding a choice the
operator made is the substitution this program exists to avoid — the same
reasoning `ftRefuse` already states for never dropping a QR automatically, and
`ftProofReplaces` for saying "It also REMOVES THE QR" out loud before the
operator accepts.

**Required:** the QR step must not offer a choice it will not honour. When the
composition in the text field needs the whole plate, that screen states the QR is
unavailable for this pattern rather than presenting "Add QR" and discarding it.
Any code reading `useQR` where the answer is a property of the PLATE must read
the fit instead — `f.plate.QR != nil` — because the fit is the one object that
knows.

Filed as a follow-up owned by the phase that next touches the QR step; it is not
folded into the whole-diff fix, which is deliberately narrow.

### 3.2 The confirm screen reads the QR off the FIT, never off `useQR`

**Found at P5, and it is a lie on the screen the operator approves.** §2.7 handles
round 1's I4 entirely at the load moment — both ladder proofs carry
`TextQR: ""`, so the prompted drop applies and `ftProofLoader` clears `useQR`.
That is not the end of it. **Back from the Text step returns to the QR screen
with the plan and the text intact** — the very path §2.1 cites as reachable for
`BOTHPROOF!` — and re-enabling the QR there leaves `useQR` true while `FitSized`
still produces no code.

`ftConfirmSummary` printed `ppYesNo(useQR)`, so the operator would approve a
screen reading **`QR: yes` on a plate that carries none**, together with the
privacy warning for a code that does not exist. On permanent steel, a confirm
screen describing a different plate than the one that will be cut is the failure
this whole program is built to avoid.

**`ftConfirmSummary`, `ftConfirmBody` and `ftConfirmFlow` therefore take no
`useQR` at all; they read `f.plate.QR`.** This is the same rule the codebase
already applies to `Faces` and `Sizes` — the fit is the one object, and nothing
downstream re-derives what it already answered. §7.17 covers it.

Two smaller P5 corrections to this section:

- **`ftProofOutcomeFor`'s rung branch is gated on `Sizeable`.** §3's table names
  its hardcoded footer but not that the branch calls `ftBothAt` for **any** proof
  when `rung != 0`. Safe on the device, where only a `Sizeable` proof ever gets a
  non-zero rung — but `cmd/plateview`'s `-size` is a flag, and §3 routes
  `proofPreview` through this resolver, so `-plate sizeproof-front -size 4.4`
  would have returned `BOTHPROOF!`'s plate under the ladder's trigger.
- **The `sizeLabel` row was half-stale.** Its zero branch already printed
  "fixed layout"; the real work was the `Mixed` range, which needs
  `Preview.Sizes` to reach the tool at all. Without that, the zero branch calls a
  ladder a "fixed layout" — which is `0.0mm` in that tool's own wording.

**A reader that prints `0.0mm` is a defect, not a fallback** — at `ftSizeLabel`
and `ftConfirmSummary`, where that is the literal symptom. At `cmd/plateview`'s
`sizeLabel` the same defect shows as **"fixed layout"**, because its zero branch
has its own string; a test written against §7.18 must assert on the string that
site actually produces. The readout and the confirm screen are what the operator
approves.

**The ladder's rung is 0, and both routing rules depend on it.** The ladder
proofs are not `Sizeable` (§4), so `ftProofForTrigger` returns rung 0,
`ftProofOutcome.SizeMM` is 0, and `ftProofLoader` writes 0 into the flow's `size`
(`freetext_proof.go:670`). That is what makes §3.1's revert reach `FitBlocks` —
"no block carries a size" is only half of it — and what keeps the UN-edited
`SIZEPROOF!FRONT`/`BACK` path from tripping the rule below, under which a
non-zero `size` together with sized blocks is an error rather than a plate.

**`ftFitAt`'s routing order matters** (round 2 Nit). It tests `size != 0` first
today and calls `FitBlocksAt`, which ignores `Block.SizeMM` by §2.2/§2.7 — and it
is shared with the `BOTHPROOF!<rung>` path, which does set a non-zero rung. If
the `FitSized` test is appended second, a caller that sets both a rung and
per-block sizes engraves the ladder at one uniform rung: R0's C6 in a third hat.
**The per-block-size test comes first, and a non-zero `size` together with sized
blocks is an error, not a silent uniform fit.**

**Two preview sites, not one** (round 1's I4-residue, and round 2's I5).
`proofPreview` ends at `gui/preview.go:130` in `fittedPreviewAt(params, p.Plan,
p.For(qr), p.Title, ftProofFooter, qr, o.SizeMM)` — a literal footer for every
proof, already wrong for the `ftProofFooterFaceMap` case and one that would put a
footer on a ladder plate that must not have one (§5). `ftProofOutcomeFor` exists
precisely so the prompt and the loader cannot disagree; the preview is a third
derivation and must use the same resolver.

Routing `proofPreview` through the resolver is **not sufficient**, because the
fit itself is one level down. `fittedPreviewAt` (`preview.go:149-155`) does
`fit := backup.FitBlocks; if size != 0 { fit = FitBlocksAt(…) }` — both of which
ignore `Block.SizeMM`. So `plateview -plate sizeproof-front` with no `-size` fits
the four-block composition UNIFORMLY: `FitBlocks` succeeds at some single rung,
`describe` prints one size, and the preview of a permanent-steel plate shows a
plate the device will not cut. `BuildPreview` is host-only, so no wrong steel
results directly — but the tool exists to check the plate *before* it is cut,
which is worse, not better. `fittedPreviewAt` routes through `ftFitAt`.

### 3.1 The edit path — what `ftPlan.Blocks` ACTUALLY does

R2 described this from `ftPlan.Blocks`' doc comment, which says the text
"collapses to a single block in the first run's face". That comment is true for
the two-run plans shipped today and **false for the four- and six-run ladder
plans** (round 2's I1). Read from the code (`freetext_flow.go:107-135`):

> Each NON-FINAL run takes `min(Blocks, remaining)` parts and the walk stops as
> soon as the parts run out; the FINAL run takes whatever is left. So
> `len(out) == len(p.Runs)` only once every non-final run's declared share has
> been satisfied.

`min(parts, runs)` is **not** the rule — that generalisation, and "it collapses
to ONE block only at one part", hold for the all-`Blocks: 1` ladders and are
false for the shipped two-run `ftPlanBoth`, whose first run declares
`ftProofBothSplit` and so swallows everything at any part count at or below it
(pinned already by `gui/freetext_flow_test.go:1134-1141`). The measured table
below is the normative statement; the prose above is its summary, and
`freetext_flow.go:107`'s corrected doc comment takes the table's rule, not the
summary. Writing a tidy generalisation into that comment is how R2 was misled in
the first place.

Measured against the BACK plan's six runs — the numbers are the probe's, not a
reading:

| parts | blocks | `len(out) == len(Runs)`? | what comes out |
|---|---|---|---|
| 1 | 1 | no | run 0's face |
| 2 | 2 | no | runs 0-1 |
| 5 | 5 | no | runs 0-4; **run 5 never emitted** |
| 6 | 6 | **yes** | the ladder |
| 7 | 6 | **yes** | runs 0-4 one part each, **run 5 absorbs two** |
| 8 | 6 | **yes** | runs 0-4 one part each, **run 5 absorbs three** |

So the previous draft's two cases were both wrong, and it missed the dangerous
one entirely:

- **The middle case it did not describe.** The operator loads `SIZEPROOF!BACK`,
  deletes one newline, presses OK. `Blocks` returns five blocks, each stamped
  with its own run's size, so every block carries a size, `ftFitAt` routes to
  `FitSized`, and permanent steel is cut as a **five-rung ladder with `const@3.0`
  — the smallest and most load-bearing rung — silently absent**, under a title
  reading `BACK 4.4+3.4+3.0` that the operator did not edit and that now lies
  about what the plate proves.
- **The collapse case it described backwards.** It claimed the collapsed block
  "carries no size", but §3's own table stamps every block with its run's size,
  so the survivor carries run 0's (5.0 mm front, 4.4 mm back) and routes to
  `FitSized`, which refuses or mis-cuts the text at 5.0 mm instead of giving the
  promised auto-fit plate. Two statements in one section contradicting each
  other — round 1's I1 class, in the section that folded round 1's I1.

**Decided: any shape-mismatched edit reverts the WHOLE plate to uniform
auto-fit.** The decision stands; R3's *predicate* for it did not.

R3 wrote that predicate as `len(out) != len(p.Runs)`, which detects a DELETED
newline and is blind to an INSERTED one: at 7 and 8 parts the table above shows
`len(out) == len(p.Runs)` still holds, so the guard reports "exact shape" and the
ladder is kept (round 3's I1). What the operator then gets is worse than the
partial ladder this section rejects. Split the first sweep across two lines —
the text keyboard has a newline key — and every part lands one run late:
`sh-a` in `sh`@4.4 correctly, `sh-b` in **`const`**@4.4, the const@4.4 sweep in
**`sh`**@3.4, and so on, with run 5 absorbing both remaining sweeps so the
**`sh`@3.0 rung is absent entirely**. A plate titled `BACK 4.4+3.4+3.0` whose
bands are engraved in the faces they do not name. The confirm screen cannot
expose it: it reads the rungs from `Fitted.Sizes` (§3), and all six rungs are
present, exactly as expected.

**The predicate is on the PART COUNT, not on the block count.** A plan whose runs
carry sizes must declare `Blocks` on every run (the ladders declare 1 each), and
`ftPlan.Blocks` clears `SizeMM` on every block it emits unless

```go
len(strings.Split(text, "\n")) == declaredParts   // sum of every run's Blocks
```

Block count cannot carry this, because the final run absorbs the remainder
(`freetext_flow.go:116-117`) and so pins `len(out)` at `len(p.Runs)` for every
part count at or above it. The part count is the quantity that actually has to
match, and it is the one the operator can see in the field.

The clearing is scoped to `SizeMM` alone, so face behaviour is untouched:
`ftPlanBoth` and `ftBothPlanFor` carry no sizes today, which makes this a no-op
for every shipped plan.

So:

- **Exact part count → the ladder is KEPT.** The plate is cut at those sizes, or
  refused if the edit no longer fits. Reverting here would be C6 wearing a
  different hat: an operator fixing a typo would silently receive a one-rung
  plate. A refusal is visible; a size change is not.
- **Any other part count, fewer OR more → uniform auto-fit.** No block carries a
  size, `ftFitAt` routes to `FitBlocks`, and the plate is an ordinary free-text
  plate.

The alternative — cut the partial ladder and rely on the confirm screen naming
the surviving rungs — was rejected. On a DELETED newline the rung that goes
missing is always the LAST run, which on both sides is the smallest and the one
the proof exists to answer. On an INSERTED one the confirm screen does not even
have the information: six rungs are reported and six rungs are cut, in the wrong
faces. The title is a separate field the operator has no reason to have edited,
and steel is permanent while the confirm screen is a glance. A plate that proves
something other than what its own title claims is worse than a plate that reverts
visibly.

All four shapes are tested (§7.13).

`freetext_flow.go:107`'s doc comment is stale for any plan with more than two
runs and is corrected as part of this change, whether or not its wording is what
misled this spec.

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
`ftProofFooter`** — not from `ftProofOutcomeFor`'s fallback, and not from
`proofPreview`'s hardcoded literal. §3's table names the mechanism for both: a
per-proof `Footer` field, empty on the two ladder entries. Getting this wrong is
a visible refusal (`FitSized` rejects both plates), not wrong steel.

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
2. **Rows are cut at the sizes claimed** — take the engraving's per-row ink
   BOUNDS with the b-spline bounds helper the geometry tests in `backup/` already
   use, and assert row i's height and baseline match `Sizes[i]` and the running
   `y` of §2.5. Naming the mechanism matters: "assert the glyph height matches"
   is satisfiable by re-reading `Sizes[i]`, which asserts nothing.
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
7. **The QR window is a plate-absolute y-range.** Four assertions, because one is
   not enough: (a) at every rung, the band predicate and the old row-index
   predicate agree on every row of a uniform plate — the no-golden-moves proof;
   (b) a TWO-BLOCK plate with a QR wraps block 2's rows at the code's own budget,
   and no body ink enters the code box — the round-1 Critical, unreachable by any
   single-block fixture. **Pin the fixture by MODULE COUNT, not by text length:**
   QR mode selection depends on the character SET, so 700 uppercase characters
   give a 77-module code and 700 mixed-case ones give 89. A test that says "700
   characters" measures a different plate the day the fixture text changes case;
   (c) `EngraveFitted` draws at `f.qrAt.Y` and `EngraveText` at its local
   placement's `Y` — now assertable, because §2.3 gives `Fitted` the field to
   carry and neither engraver computes a y of its own; (d) `qrAt.Bottom >
   plateHeight - margin` comes back as an ERROR from `fitBlocksAt`/`FitSized`,
   not as a panic — a test that accepts a panic here would pass against exactly
   the mid-flow crash §2.1.1 exists to avoid.

   **Not tested, by decision (operator, 2026-08-05): the QR on a multi-size
   plate.** `QR != nil` with `Mixed` is unreachable through every entry point —
   `FitSized` has no QR parameter and `fitBlocksAt` never sets `Mixed` — so
   testing it means hand-building an illegal `Fitted` to watch a panic fire on a
   state no caller can produce. §2.1.1's guard STAYS as a defensive assertion for
   a future constructor; it is deliberately unpinned, which is recorded here
   rather than left to look like an oversight. Every QR assertion above is on a
   UNIFORM-size plate, including (b)'s two-block fixture — that one is the
   mixed-FACE shipped-plate regression from round 1, not a multi-size test.
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
13. **The edit path, all FOUR shapes** (§3.1), against the six-run BACK plan:
    (a) 6 parts — exact — keeps the ladder, and is refused rather than re-fitted
    when the edit no longer fits; (b) **5 parts — a deleted newline — reverts,
    and does NOT cut a five-rung plate missing `const@3.0`**; (c) **7 parts — an
    inserted newline — reverts, and does NOT cut a six-rung plate with every
    band one run late and `sh@3.0` absent**; (d) 1 part — full collapse —
    reverts. (c) is the one `len(out) != len(p.Runs)` was blind to, so a test
    that omits it re-admits round 3's I1. Assert on the resolved `Fitted.Sizes`
    and `Fitted.Faces`, not on the block list: the block list is what looked
    right in the failing case.
    **Plus a synthetic sized plan with runs `[2, 1, 1]`**, where
    `declaredParts == 4` but `len(p.Runs) == 3`: sizes survive at 4 parts and
    clear at 3 and 5. Both ladders have one block per run, so
    `declaredParts == len(p.Runs)` for every real fixture and none of (a)-(d)
    can tell the specified predicate from a `len(p.Runs)`-based one — including
    under §7.19's mutation pass. Assert too that every run of every SIZED plan
    declares `Blocks >= 1` and that a sized plan has more than one run, since
    `ftPlan.Blocks`' single-run early return (`freetext_flow.go:109-111`)
    bypasses the split and would never clear a one-run sized plan's sizes.
14. **The size/string invariant** (§2.3), **all FOUR combinations** — the
    invariant is a biconditional, and an earlier draft of this item named only
    two of them. `FitSized` refuses a title with a zero size, a footer with a
    zero size, **a title size with no title, and a footer size with no footer**.
    The second pair is the direction where a size was resolved for a row that
    does not exist; a test written to the two-case form misses it entirely.

    And the no-footer path lays out, engraves without panicking, and refuses one
    more row. **Assert that, not `limit == plateHeight - margin`** — this item
    used to demand the latter while banning its own form in the same breath:
    `limit` is a local of the unexported `yBudget`, which neither named function
    returns or exposes, so pinning it means calling `yBudget` directly rather
    than observing either function "produce" anything. Legal in-package, but it
    is not the observable outcome the item asks for. An unfalsifiable item is a
    false-pass slot, and §7.19 mutation-tests every item here.
15. **The `len(Sizes) != len(Lines)` guard panics** — round 0's I8(e), still
    open after round 1.
16. **Both ladder proofs report `NeedsWholePlate()`**, the prompt says the QR is
    removed, and loading one with the QR on clears `useQR` before the outcome is
    resolved.
17. **The confirm screen says what the plate is, and fits.** Three assertions:
    (a) the rendered `ftConfirmSummary` for the front NAMES the rungs and
    **never contains "0.0mm"** — the defect §3 bolds, on the one screen the
    operator approves, which a fits-the-panel assertion passes happily;
    (a2) **it reports the QR from the FIT, not from `useQR`** (§3.2): load a
    ladder, go Back to the QR screen, re-enable the QR, and the confirm screen
    must still read `QR: no` and must not carry the privacy warning. This is
    reachable on shipped firmware and is the one screen the operator approves;
    (b) it fits the real panel, asserted by MEASURING RECTANGLES as
    `ftProofBody`'s test does — `ExtractText` collects runes regardless of
    occlusion, so a label drawn off the panel reads as present.
18. **The preview agrees with the device, in sizes and not only in fields** —
    `proofPreview` resolves through `ftProofOutcomeFor`, no proof preview carries
    a footer the device would not engrave, and `plateview -plate sizeproof-front`
    **with no `-size`** yields per-row sizes equal to the device's
    `Fitted.Sizes`. The bare case is the one `fittedPreviewAt` silently fitted
    uniformly.
19. **Every one mutation-tested.** Four false-passing tests reached review across
    the last two cycles; assume more will try.
20. **`AdmissibleBlocks`' verdict does not move** (§2.4, §2.1). **THREE cases at
    3.0 mm in `sh`, not two** — the two-case form this item carried until P3 was
    a measured false PASS, corrected below.
    - **At capacity and one line over, no QR** — pins the `start` translation.
      Measured: `linesAvail` 24; 1032 `a` → `(24, true)`; 1033 → `(25, false)`.
      Carrying `start` over as the literal `1` gives `(25, false)` at capacity.
    - **At capacity and one line over, with a QR** — keeps the verdict pinned on
      the QR path. Measured: 616 `A` (69 modules) → `(24, true)`; 617 →
      `(25, false)`.
    - **A text INSIDE the band, with a QR** — and this is the only case that
      catches the placement being anchored at `start` instead of `outerMargin`.
      Measured: 375 `A`, 57 modules → 16 rows.

    **Why the third case is required.** This item used to say "the QR case is the
    one that fails if the placement is anchored at `start`", prescribing only a
    text at capacity. That does not hold. Anchoring at `start` slides the band
    down exactly one row, and at 3.0 mm both edge rows are plain — so the band
    trades one plain row in at the top for one plain row out at the bottom and
    **the 24-row total is unchanged**. At capacity both anchors report
    `(24, true)`; one over, both report `(25, false)`. Swept over 1-900
    characters in both cases, the two anchors disagree on **207 lengths, none of
    them at capacity**. Written as it was, the QR half of this item was
    decorative: it would have passed against the defect it was written to catch.

    Also pin `MaxCharsAtBlocks` for a mixed-block composition whose face boundary
    falls on a screw-hole row, so `rowFaces`' `0` → `params.I(outerMargin)`
    translation cannot be skipped silently: measured **1094** characters for an
    `sh` block filling plate rows 0-23 plus a `constant` block, boundary on row
    24. `TestMaxCharsAtBlocksCountsEachRowInItsOwnFace` asserts only a loose
    bracket and passes with the boundary row shifted.

    **Every pin above was verified by mutation, not by passing.**

## 8. Non-goals

- The 6.0 mm rung.
- Generalising auto-fit to mixed sizes; `FitBlocks` stays as it is.
- Any RELATIONSHIP between the two sides in firmware — no pairing, no flip
  prompt, no ordering, no "the other side of this plate". `ftProof.Side` is a
  label the prompt prints (§4) and nothing more; dropping it in the name of this
  non-goal re-admits R0's I5, since both sides prove identical faces and
  `Plan.Name()` cannot tell them apart.
- A QR on these plates — `FitSized` has no parameter for one (§2.7).
- Rendering both sides as one image in `cmd/plateview`; two invocations.
- The `FONTPROOF!` → `PASSPROOF!` rename; its own change, operator's call.
