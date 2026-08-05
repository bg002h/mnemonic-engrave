# SPEC — `SIZEPROOF!`: per-row sizing and the two-sided size-ladder plate

Status: **R1, folding R0 round 0 (RED, 6C/8I).** Author: controller session,
2026-08-05. Review persisted at
`design/agent-reports/bothproof-all-spec-R0-round0.md`.

Risk-set work: changes plate **layout and admission**, and `font/constant`
plates carry seeds and passphrases. No code before R0 is 0C/0I.

Renamed from `BOTHPROOF!ALL` per `LEXICON_proof_triggers.md`: the slot after
`BOTHPROOF!` already means "rung", so a content-and-side value there would be a
second trigger wearing the first one's name.

---

## 1. Goal

One plate, engraved both sides, carrying the complete 95-character sweep in
**both faces** at **five rungs**. It answers what a render cannot: which glyphs
survive as the size drops.

**All heights below are MEASURED by simulating the real layout** — accumulating
each block's `baseY` and reading `lineLayout.at` per row — not computed from
`CharsPerLine`. That distinction is R0's C2 and is the reason every figure in
the previous draft was wrong: `CharsPerLine` is the UNOBSTRUCTED width, and the
screw-hole band narrows the top rows and forces more of them. **Six of the ten
(face, rung) pairs need a row more than the naive count.**

| side | title | blocks (face@rung: rows) | body ends | spare |
|---|---|---|---|---|
| **FRONT** | `FRONT 5.0+3.8` @3.8 | sh@5.0:4, const@5.0:5, sh@3.8:3, const@3.8:4 | 78.40 mm | **3.60 mm** |
| **BACK** | `BACK 4.4+3.4+3.0` @3.0 | sh@4.4:4, const@4.4:4, sh@3.4:3, const@3.4:3, sh@3.0:3, const@3.0:3 | 79.60 mm | **2.40 mm** |

Limit is 82.00 mm (`plateSize - outerMargin`). 475 characters. No QR, no
confusable table, no prose, **no footer** (§5). 6.0 mm is absent by decision.

**A block's height depends on where it starts.** The front's `sh@5.0` block
takes 5 rows with no title and 4 with one, because the title pushes it below the
screw-hole band. That is why a title makes the front *roomier* (3.60 mm) than no
title (2.40 mm) — the inversion R0 found, and the reason "no titles = maximum
margin" was false.

The two sides are **two independent plate programs and an operator flip.** The
firmware gains no concept of a side.

## 2. Per-row sizing

Today a plate has one size (`backup/freetext.go:42-49`). Size becomes uniform
**within** a block and variable **between** blocks.

### 2.1 What the previous draft got wrong

It claimed the per-line grid needs "no new arithmetic at all". **Half true, and
the false half engraves over the QR.** `lineLayout.at(i)` uses `i` for two jobs:

- the band predicate, `baseY+i*fontSize` (wrap.go:140-141) — `baseY`-relative,
  and does survive block-relative indexing;
- `isQRLine := holeLines <= i && i < holeLines+qrLines` (wrap.go:135) — a
  **plate-absolute row index**, which does not.

Re-indexing rows from 0 within a block shifts the QR window by the number of
rows above the block. Measured at production params (3.0 mm, QR size 73):
`holeLines=3`, `qrLines=16`; with a title, body line 2 is plate row 3 and is a
QR line at 17 columns, but becomes index 2 and is wrapped at the full 44 — **the
line is engraved straight across the QR**, on an ordinary operator free-text
plate, not on these proof plates at all.

**Required fix:** `holeLines`/`qrLines` become a device-unit y-range `qrTop`,
`qrBottom` on `lineLayout`, tested against `baseY+i*fontSize` exactly as the
band predicate already is. The QR narrowing needs *precisely this* new
arithmetic.

### 2.2 `Block` gains a size

```go
type Block struct {
    Face   *vector.Face
    Text   string
    SizeMM float32 // 0 = the size the plate is fitted at
}
```

Zero preserves every existing caller and golden.

### 2.3 `Fitted` gains per-row sizes and an explicit title size

```go
type Fitted struct {
    Uniform    bool        // false when the plate mixes sizes
    SizeMM     float32     // valid only when Uniform
    Sizes      []float32   // parallel to Lines: the size row i is cut at
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
(freetext.go:34-39).

`Uniform` exists because R0's C5 showed `SizeMM == 0` is not a safe sentinel:
`params.F(0) == 0`, so `LinesPerPlate` divides by zero and `fixedCharWidth`
returns 0, making `textLayout`'s `width / charWidth` a second divide-by-zero.
The device would **panic in `ftBuildPlate` mid-flow with a plate clamped in the
machine.** A named boolean cannot be dereferenced by accident; a zero float can.

`TitleSizeMM` is explicit because the title sits at the side's **smallest** rung,
not the first block's size — R0's I1. The first block's rule would give the back
a 4.4 mm title and 1.00 mm of spare.

### 2.4 `wrapBlocks` takes an explicit y budget

Today it counts rows against `end-row`, passes `params.I(outerMargin)` as
`baseY` for every block, and lets `widthFor(lay, row)` supply a plate-wide row
index. It must instead carry a running `y` in **device units**, pass that as the
block's `baseY`, and index rows from 0 within the block.

The budget is stated explicitly at both ends, because R0's C3 showed the naive
"refuse at the bottom margin" reserves nothing for the footer and lays the last
body block over it:

```
start = margin + F(titleSizeMM)          // 0 when there is no title
limit = plateHeight - margin - F(footerSizeMM)   // no footer -> just the margin
```

`maxY` is a parameter with a documented **unbounded sentinel**, because
`AdmissibleBlocks` (fit.go:280) and `rowFaces` (fit.go:302) deliberately pass
`math.MaxInt` to get an untruncated count — fit.go:277-279 states why: *"a
refusal that reported '26 / 26' for a text needing 300 lines would tell the
operator nothing about how much to cut."* A height budget with no unbounded
representation regresses that.

### 2.5 `EngraveFitted` walks a running y in device units

`fontSize`, `rows` and `bodyRows` are **removed** from `EngraveFitted` — they
have no meaning on a mixed plate and are how C5's panic arrives. Replaced by:

- `y` accumulated as `y += params.F(f.Sizes[i])` — **device units**. Accumulating
  float32 millimetres and converting once at the end drifts (20 additions of
  3.8f can land on 486399 instead of 486400) and moves every golden. For a
  uniform plate the running sum and `margin + row*fontSize` are **exactly
  equal**, because every rung converts exactly at MM=6400.
- the title at `margin`, in `TitleFace` at `TitleSizeMM`;
- the footer at `margin + (LinesPerPlate(params, FooterSizeMM)-1)*F(FooterSizeMM)`.
  A naive bottom anchor differs by the `LinesPerPlate` remainder — 1.0 mm at
  3.0 mm, 3.0 mm at 3.8 mm — and moves every existing golden.

### 2.6 `faceLayouts` must be keyed on more than the face

`faceLayouts` (fit.go:316-331) caches per face and hardcodes
`baseY = params.I(outerMargin)`. The front is `sh@5.0, const@5.0, sh@3.8,
const@3.8` — the third block asks for `sh` and gets the cached **5.0 mm** grid
while being cut at 3.8 mm, putting the left inset of every screw-hole row in
that block out by about 0.73 mm in the wrong direction.

Key it on `(face, fontSize, baseY)`, or drop the cache — a mixed plate has a
handful of blocks.

### 2.7 The new entry point

```go
// FitSized lays out a composition whose every block states its own size.
func FitSized(params engrave.Params, blocks []Block, title, footer string,
    titleSizeMM, footerSizeMM float32) (Fitted, error)
```

No ladder walk and **no `useQR`**: the QR box is placed from a single
`fontSize` (freetext.go:81-85) and has no meaning without one. `FitSized`
refuses a QR outright rather than accepting a parameter it cannot honour.

Every block must carry a non-zero `SizeMM` that is a rung in `FontSizes`, the
same guard `FitBlocksAt` applies. `FitBlocks` and `FitBlocksAt` are untouched.

## 3. The GUI surface — the whole path, not just the prompt

R0's C6: nothing carries per-block sizes from the trigger to the fit, so the
previous draft would have **silently engraved a uniform plate**. Six sweeps do
fit uniformly at 3.0 mm, so `FitBlocks` would have succeeded, the confirm screen
would have read "3.0mm 18 lines", and the operator would have approved a
five-rung survey and received one rung. Every one of these must change:

| site | change |
|---|---|
| `ftFaceRun` | gains `SizeMM float32` |
| `ftPlan.Blocks` | returns blocks carrying their run's size |
| `ftProofOutcome` | carries the composition, not a single rung |
| `ftFitAt` / `ftEvaluate` / `ftBuildPlate` | route a fixed composition to `FitSized` |
| `ftProof` | gains a `Side` field the prompt reads directly |
| `ftFaceSummary` (flow.go:487) | also prints the SIZE runs, read from `Fitted.Sizes` |
| readout / confirm (`"%.1fmm"`) | show a range when `!Uniform`; never `0.0mm` |
| `MaxCharsAtBlocks` (fit.go:344) | takes one `fontMM` and indexes by plate row |
| `rowFaces` (fit.go:297) | takes one `fontMM` |
| `gui.Preview` | gains `Sizes`; `describe` prints per-row size beside per-row face |
| `previewBuilders` | two entries; `-size` must not re-fit a ladder plate at one rung |

**A reader that prints `0.0mm` is a defect, not a fallback.** The readout and
the confirm screen are what the operator approves.

**The edit path** (R0 M4): the proof loads into an EDITABLE field, and
`ftPlan.Blocks` collapses to one block if the operator deletes a line. An edited
ladder composition **reverts to ordinary uniform auto-fit** — stated, and tested.

## 4. Triggers

**`SIZEPROOF!FRONT`** and **`SIZEPROOF!BACK`**, per `LEXICON_proof_triggers.md`.

`SIZEPROOF!` bare is not a trigger: the ladder has no default half, and
defaulting would let a slip cut the wrong side onto steel already engraved.
Neither entry may ever be marked `Sizeable`, or `SIZEPROOF!FRONT4.4` becomes
ambiguous against the rung suffix parser.

The prompt must name the **side and its rungs** from the `Side` field, not from
`Plan.Name()`. R0's I5: both sides prove identical faces, so the plan name gives
near-identical prompts, and a mis-pick is engraved where a mistype is refused.

`AdmissibleBlocks` (§6) is what makes the two triggers reachable at all; the
previous draft's claim that ladder plates bypass admission was false.

## 5. Identification — DECIDED

**Titles yes, at each side's smallest rung. No footer on either side.**

- `FRONT 5.0+3.8` (13 chars) at 3.8 mm — spare 3.60 mm
- `BACK 4.4+3.4+3.0` (16 chars) at 3.0 mm — spare 2.40 mm

Both under `MaxTitleLen` (18). `EngraveFitted` engraves `Title` **verbatim**, not
through `TitleString`, so a test must validate both against `MaxTitleLen` and
against `TitleFace.Decode`.

**No footer**, because with a title a footer overflows the front outright
(83.4 mm > 82) and overlaps the back's last body row by 1.4 mm of ink. The
ladder proofs therefore pass an EMPTY footer and **must not inherit
`ftProofFooter` from `ftProofOutcomeFor`.**

On the back's 2.40 mm: acceptable **only because §7 pins each side's per-block
row count and total height**. The hazard is discrete — a glyph change costing
`font/constant` a row at 3.0 mm moves the total by 3.0 mm, more than any margin
under discussion — so margin does not mitigate it and a test does. Without that
test 5.40 mm would not be enough either.

## 6. Invariants

- **No existing golden moves.** Uniform plates take the general path with every
  `Sizes` entry equal. `-update` is **not** run in this change; a moved golden
  means the general path does not reproduce the special case.
- **The unbounded `wrapBlocks` callers stay unbounded** (§2.4).
- **`AdmissibleBlocks` still runs on ladder plates.** They load into the text
  field, `ftEvaluate` calls it, and `!f.ok` gates OK on both the text and
  confirm steps. The readout will report the 3.0 mm anchor's line count, which
  differs from the rows actually cut — accepted, and stated here because the
  previous draft claimed the opposite.
- Run counts and per-run timing quantisation untouched; no glyph changes.
- No wire format, NDEF, codec, validation or identity change.
- `outerMargin` and `toPlate`'s safety margin are both 3 mm. They are
  **coincidentally equal and are not independent checks**; the front at 3.60 mm
  clears both by the same number.

## 7. Test plan

1. **Every existing plate golden, byte for byte** — the load-bearing test.
2. **Rows are cut at the sizes claimed** — decode the engraving and assert row
   i's glyph height matches `Sizes[i]`.
3. **Per side, a table pinning per block: face, size, row count, device-unit
   y-range**, plus the total. This is what makes 2.40 mm safe, and what a font
   change must fail against.
4. **Character SET per `(rung, face)` pair** — not per plate. "95 characters in
   both faces" is satisfiable by a full sweep at one rung and a truncated one at
   another.
5. **A block starting inside the top band is inset; one below it is not** —
   mixed sizes are exactly where a plate-wide row index mis-insets.
6. **The same face at two different sizes on one plate** (the front is one):
   each row's grid and left inset computed at that row's own size. Catches C4.
7. **The QR window is a y-range** — a title + QR plate wraps the same as today.
   Catches C1 without relying on a golden being re-baselined.
8. **Footer and last body row are disjoint**, and a composition one row too tall
   for its title+footer is refused. Replaces the previous draft's "no row
   overlaps its neighbour", which was a theorem, not a property.
9. **`EngraveFitted` on a `!Uniform` plate does not panic.** Catches C5.
10. **End-to-end through `freetextPlateHook`**: `SIZEPROOF!FRONT`/`BACK` produce
    a `Fitted` with the expected `Sizes` vector and `Uniform == false`. The only
    thing that catches C6.
11. **Unbounded `wrapBlocks` callers still report untruncated counts.**
12. **Titles fit `MaxTitleLen` and decode in their face.**
13. **An edited ladder composition reverts to uniform auto-fit.**
14. **Every one mutation-tested.** Four false-passing tests reached review across
    the last two cycles; assume more will try.

## 8. Non-goals

- The 6.0 mm rung.
- Generalising auto-fit to mixed sizes; `FitBlocks` stays as it is.
- Any notion of plate sides in firmware.
- A QR on these plates — `FitSized` refuses one.
- Rendering both sides as one image in `cmd/plateview`; two invocations.
- The `FONTPROOF!` → `PASSPROOF!` rename; its own change, operator's call.
