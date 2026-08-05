# SPEC — `BOTHPROOF!ALL`: per-row sizing and the two-sided survey plate

Status: **R0 not yet run.** Author: controller session, 2026-08-05. Supersedes
nothing; builds on `RECON_bothproof_all.md`, whose measurements are taken as
settled and are not re-derived here.

Risk-set work under the project rule: it changes **plate admission and layout**,
and `font/constant` plates carry seeds. No code before R0 is 0C/0I.

---

## 1. Goal

One plate, engraved on both sides, carrying the complete 95-character sweep in
**both faces** at **five rungs**. It answers one question a render cannot:
which glyphs stay legible as the size drops.

| side | content | height | spare |
|---|---|---|---|
| **A (front)** | `5.0 @95` + `3.8 @95` | 71.6 mm | 7.4 mm |
| **B (back)** | `4.4 @95` + `3.4 @95` + `3.0 @95` | 73.6 mm | 5.4 mm |

475 characters of proof on one piece of steel. No QR, no confusable table, no
prose or pangrams. 6.0 mm is deliberately absent — see RECON §4.

The two sides are **two independent plate programs and an operator flip.** The
firmware gains no concept of a "side"; nothing in this spec knows one exists.

## 2. The one real change — per-row sizing

Today a plate has a single size. `backup/freetext.go:42-49`:

```go
fontSize := params.F(f.SizeMM)
rowY := func(row int) int { return margin + row*fontSize }
```

**The key structural fact, which makes this far smaller than it looks: size is
uniform WITHIN a block, and varies only BETWEEN blocks.** A block is already a
run of text in one face; it becomes a run in one face at one size. So the
per-line grid (`lineLayout`, the screw-hole band predicate, the QR narrowing)
needs no new arithmetic at all — it is already computed per block and already
takes a `baseY`.

What changes is that `baseY` must become each block's **actual y**, accumulated
in device units, rather than the plate margin with a plate-wide row index doing
the work.

### 2.1 `Block` gains a size

```go
type Block struct {
    Face   *vector.Face
    Text   string
    SizeMM float32 // 0 = the size the plate is fitted at
}
```

Zero keeps every existing caller and every existing golden unchanged.

### 2.2 `Fitted` gains per-row sizes

```go
type Fitted struct {
    SizeMM float32     // the uniform size, or 0 when the plate is mixed
    Sizes  []float32   // parallel to Lines: the size row i is cut at
    Lines  []string
    Faces  []*vector.Face
    ...
}
```

`Sizes` is **always** populated, including for uniform plates, where every entry
is equal. That is deliberate: `EngraveFitted` then has ONE path rather than a
uniform path and a mixed path, and the existing goldens are the proof the
general path reproduces the special case exactly. A second code path is how the
two would drift.

`Sizes` is parallel to `Lines` for the same reason `Faces` is, and carries the
same prohibition: **nothing downstream may re-derive it.**

### 2.3 `wrapBlocks` accumulates height, not rows

Today it counts rows against `end-row` and passes `params.I(outerMargin)` as
`baseY` for every block, letting `widthFor(lay, row)` supply a plate-wide row
index. With mixed sizes `margin + i*fontSize` is wrong for any block after the
first.

It must instead track a running `y` in device units, pass that as the block's
`baseY`, index rows from 0 **within** the block, and refuse when `y` would pass
the bottom margin. The budget stops being a row count and becomes a height.

`lineLayout.at`, `textLayout`, `widthFor` and `WrapText` are otherwise unchanged.

### 2.4 `EngraveFitted` walks a running y

`rowY` becomes a prefix sum over `Sizes` rather than `margin + row*fontSize`.
Title and footer keep their existing behaviour and sit at the size of the block
they border, as `TitleFace`/`FooterFace` already do for faces.

### 2.5 The new entry point

```go
// FitSized lays out a composition whose every block states its own size.
func FitSized(params engrave.Params, blocks []Block, title, footer string, useQR bool) (Fitted, error)
```

No ladder walk: the composition is fully specified, so this either lays it out
or refuses. `FitBlocks` (auto-fit) and `FitBlocksAt` (one rung) are untouched.

**Every block must carry a non-zero `SizeMM`, and it must be a rung in
`FontSizes`** — the same guard `FitBlocksAt` already applies, for the same
reason: every capacity number in the package is measured at those rungs.

## 3. Consumers of `SizeMM` — all seven, and what each does

A mixed plate has no single size, so `SizeMM` is 0 and every reader must be
visited. Enumerated so none is missed:

| site | today | with a mixed plate |
|---|---|---|
| `backup/freetext.go:42,45` | row pitch, row count | reads `Sizes` instead |
| `backup/fit.go` (`Fit`) | returns it | unchanged; auto-fit only |
| `gui/freetext_flow.go` readout | `"%.1fmm"` | shows the range, `"3.0-5.0mm"` |
| `gui/freetext_flow.go` confirm | `"%.1fmm"` | same |
| `gui/freetext_proof.go` | prompt copy | the ALL prompt names both sides |
| `cmd/plateview` `sizeLabel` | `"%.1fmm"` | already handles 0; must show the range |
| `gui/preview.go` | passes it through | unchanged |

**A reader that silently prints `0.0mm` is a defect, not a fallback.** The
readout and the confirm screen are what the operator approves.

## 4. Triggers

Two programs, so two triggers. Proposed: **`BOTHPROOF!ALLA`** and
**`BOTHPROOF!ALLB`**.

Both are exact matches in `ftProofForTrigger`, which today accepts only an exact
trigger or a suffix parsing as a rung in `FontSizes` — so `ALLA`/`ALLB` match
nothing at present and are free. They are the same length and differ in the last
character, which breaks the existing "triggers differ from their first
character" convention; that convention exists so a mistyped trigger matches
nothing, and exact-match lookup already guarantees that, but **R0 should confirm
this rather than take my word for it.**

The prompt must name which side it is loading and say the other exists,
because a plate cut from only one of them is half a proof.

## 5. Identification — OPEN, and the weakest part of this spec

With no title, a side found in a drawer has no record of which rungs it carries.
Adding one costs a row at the size of the block it borders:

| | body | + title at the side's smallest rung | spare |
|---|---|---|---|
| A | 71.6 | 75.4 mm | 3.6 mm |
| B | 73.6 | 76.6 mm | 2.4 mm |

Both fit, but 2.4 mm on side B is **under one row at 3.0 mm**. This pattern is
rebuilt whenever a glyph changes, and if `font/constant` ever drops from 39 to
31 columns at 3.0 mm the sweep needs a fourth row and side B is refused
outright.

Three options, no recommendation — this is the decision R0 should force:

1. **Titles at the smallest rung.** Self-documenting, 2.4 mm of margin.
2. **No titles.** Maximum margin; sides identified by which rungs they carry,
   which is legible only in relation to each other.
3. **Titles, and drop `3.4` from side B** (back becomes 4.4+3.0, 53.2 mm).
   Comfortable margin, four rungs instead of five.

## 6. Invariants — what must not break

- **No existing golden moves.** Uniform plates take the same path with every
  `Sizes` entry equal; if a golden moves, the general path is not reproducing
  the special case and the change is wrong.
- **Run counts and per-run timing quantisation are untouched.** This spec adds
  no glyph change.
- **`AdmissibleBlocks` keeps anchoring free text at 3.0 mm.** The ALL plates do
  not go through admission — they are fixed compositions, not operator text.
- **No wire format, NDEF, codec, validation or identity behaviour changes.**
- **`font/sh` and `font/constant` are unmodified.**

## 7. Test plan

Behavioural, not incidental:

1. **The uniform path is unchanged** — every existing plate golden, byte for
   byte. This is the load-bearing test of the whole change.
2. **A mixed plate's rows are cut at the sizes it claims** — decode the
   engraving and assert row i's glyph height matches `Sizes[i]`, not merely that
   `Sizes` was populated.
3. **The running y is right** — the last row's baseline plus its size is inside
   the bottom margin, and no row overlaps its neighbour. This is what a prefix
   sum gets wrong.
4. **The screw-hole band is respected per block** — a block starting inside the
   top band is inset; one starting below it is not. Mixed sizes are exactly
   where a plate-wide row index would silently mis-inset.
5. **Both sides fit at the stated heights**, and each carries all 95 characters
   in both faces — assert the character SET, not the count.
6. **Off-ladder or zero sizes are refused** by `FitSized`.
7. **The readout and confirm screen never show `0.0mm`** for a mixed plate.
8. Every one of the above **mutation-tested**: revert the behaviour and the test
   must fail. Two false-passing tests reached the pre-ship review in the last
   cycle; assume more will try.

## 8. Non-goals

- The 6.0 mm rung. RECON §4.
- Generalising auto-fit to mixed sizes. `FitBlocks` walks the ladder for a
  uniform composition and stays that way.
- Any notion of plate sides in firmware.
- A QR on these plates.
- Rendering both sides as one image in `cmd/plateview`; two invocations.
