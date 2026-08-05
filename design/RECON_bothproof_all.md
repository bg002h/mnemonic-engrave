# RECON — `BOTHPROOF!ALL`: every character, across the size range

Status: **designed enough to build, not built.** Captured 2026-08-05 so the
measurements below are not re-derived. Deferred deliberately: it needs a change
to plate layout, and the release at `fork-v0.0.0-g3c3a2ad` was already at the
gate when it was proposed.

## 1. What it is

A survey plate: every printable character, seen across the engraving size range,
in **both faces**. No QR, no confusable table, no prose or pangrams — the sweep
alone (operator, 2026-08-05).

## 2. The blocker — a plate has ONE size today

`backup/freetext.go` takes a single `fontSize` from `Fitted.SizeMM` and lays
every row on a uniform pitch:

```go
fontSize := params.F(f.SizeMM)          // :42
rowY := func(row int) int { return margin + row*fontSize }   // :49
```

So a plate carrying several sizes needs **per-row sizing and a variable row
pitch**. That is new layout code, not a new pattern string, and it touches how
every free-text plate is laid out — hence its own design pass and review rather
than a bolt-on.

Knock-on effects to think through when building it:

- `LinesPerPlate` and `bodyRows` are computed from one size; with mixed sizes,
  "rows" stops being a meaningful unit and the fit becomes a HEIGHT budget.
- The screw-hole band predicate in `wrap.go`'s `lineLayout.at` tests
  `baseY + i*fontSize` against `innerMargin`. With variable pitch it must take
  the row's own y, not an index times a constant.
- `AdmissibleBlocks` anchors admission at 3.0mm; a mixed plate has no single
  anchor.
- `Fitted.SizeMM` is read by the readout, the confirm screen and the engraver.
  A mixed plate needs either a per-row size list or an explicit "mixed" marker,
  and every consumer must be visited — the same "one value, three consumers"
  property `Fitted` was built for.

## 3. The measurement that shapes the design

The whole alphabet at EVERY rung does not fit, by a wide margin. Rows needed for
the 95-character sweep, and the height they cost (usable plate height is
**79 mm**: 85 less two 3 mm margins):

| rung | sh cols → rows | const cols → rows | height |
|---|---|---|---|
| 6.0 | 22 → 5 | 19 → 5 | 60.0 mm |
| 5.0 | 26 → 4 | 23 → 5 | 45.0 mm |
| 4.4 | 30 → 4 | 26 → 4 | 35.2 mm |
| 3.8 | 34 → 3 | 31 → 4 | 26.6 mm |
| 3.4 | 38 → 3 | 34 → 3 | 20.4 mm |
| 3.0 | 44 → 3 | 39 → 3 | 18.0 mm |

**205.2 mm needed against 79 mm available — 2.6x over.** Even a single face at
all six rungs is 98.2 mm (sh) or 107.0 mm (const), still over. Dropping the QR,
confusables and prose does not help: none of them are in this arithmetic.

## 4. DECIDED — two sides of one plate, five rungs, full sweep each

Settled with the operator 2026-08-05, after working through the options in §5.

| side | content | height | spare |
|---|---|---|---|
| **front** | `5.0 @95` + `3.8 @95` | 71.6 mm | 7.4 mm |
| **back** | `4.4 @95` + `3.4 @95` + `3.0 @95` | 73.6 mm | 5.4 mm |

**475 characters on one plate**: the complete 95-character sweep, in both faces,
at every rung from 3.0 to 5.0. No QR, no confusable table, no prose.

Two decisions carry the design:

**Use both sides.** The firmware needs no concept of a "side" -- it is two plate
programs and an operator flip. This is what turns an impossible one-sided
problem into a comfortable two-sided one, and it doubles the proof yield per
piece of steel, which matters because plates are scarce.

**Drop the 6.0 mm rung.** Its full sweep alone costs 60 mm of the 79 available,
so it consumes a whole side by itself and forces every other rung to be trimmed.
Dropping it is what lets the remaining five carry the FULL sweep with real
margin. 6.0 mm is also the rung whose legibility was never in doubt.

### Deliberately NOT taken

- **A 6.0 mm header doubling as the big-size sample.** A header is one row, so a
  6.0 mm one costs 6 mm and would give a 6.0 mm specimen without spending 60. It
  fits -- front goes to 77.6 mm -- but that leaves **1.4 mm of spare, under half
  a row at that size**. This pattern is rebuilt whenever a glyph changes, and
  auto-fit is all-or-nothing: a side sitting at 77.6 of 79 is refused outright by
  the next edit rather than engraved slightly smaller. Revisit once the layout
  code exists and it can be rendered. If it is taken, make the header text carry
  interesting glyphs (`SH+CONST ALL 0O1lI| =f`) so it is a specimen and not just
  a label.
- **All six rungs.** Needs three sides across two plates: `5.0 @95` + `4.4 @88`
  (75.8 mm), `3.8/3.4/3.0 @95` (65.0 mm), and `6.0 @95` alone (60.0 mm). Costs a
  second plate and trims 4.4 to 88 of 95 characters, for one rung nobody doubts.

## 5. The options that were measured, and why they lost

Rather than showing all 95 characters at each rung, the alphabet can be DEALT
across rungs -- first N characters at 6.0, the next N at 5.0, and so on -- so
each character appears once. Measured, both faces:

| composition | height | characters |
|---|---|---|
| 32/32/31 at 6.0/5.0/4.4 only | 61.6 mm | 95 |
| big 16/16/16 + small 32/32/31 | 55.0 mm | 143 |
| 24 per rung, all six | 68.2 mm | 144 |
| 16 per rung, all six | 51.2 mm | 95 |
| 8 per rung, all six | 51.2 mm | 48 |
| 32 per rung, all six | 85.8 mm | OVER |

All of these lost to the two-sided plan, which carries **475** characters rather
than 95-144. Dealing the alphabet out is only necessary while the plate is
one-sided.

Two measurements worth keeping, because they are counter-intuitive:

**8 characters cost exactly what 16 do.** At every rung, 8 characters still needs
one row per face, and so does 16 -- 51.2 mm either way. **Row count consumes the
plate, not character count.** Below one row per face you are paying for empty
row. At 4.4 mm you get 26 characters for the price of 8, because 26 is where
`font/constant` needs a second row.

**Removing the title and footer saves nothing here.** The 79 mm budget is
`85 - 2x3 mm` outer margin and every figure counts sweep rows only. A title and
footer each cost one row at the size of the block they border (`bodyRows` in
`backup/fit.go`), so the question is whether to ADD them, not whether to remove
them.

## 6. Open questions

1. **Which characters go to which side?** Both sides carry the complete sweep at
   every rung they hold, so this is not a dealing question any more -- but the
   two sides need to be distinguishable once cut. Codepoint order is the obvious
   sweep order and is what the shipped patterns use.
2. **How is each side identified?** With no header, a side found later has no
   record of which rungs it carries. The rung sizes are visually obvious in
   relation to each other but not absolutely. See the "deliberately not taken"
   note above.
3. **Trigger names.** One trigger per side is needed, since they are two plate
   programs: `BOTHPROOF!ALL` and something for the second side. The rung suffix
   parser accepts only strings parsing as a rung in `FontSizes`, so `ALL` matches
   nothing today and stays ordinary text.
4. **Does engraving the back risk the front?** Unmeasured, and not a firmware
   question -- whether a cut face is marked or misregistered by the holder when
   flipped is something only a real plate answers.
