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

## 4. The resolution — DEAL the alphabet out, do not repeat it

The operator's insight (2026-08-05): rather than showing all 95 characters at
each rung, split the alphabet **across** rungs — first N characters at 6.0, the
next N at 5.0, and so on. Each character then appears once, and every rung is
still represented.

Measured, both faces, characters per rung:

| composition | height | characters |
|---|---|---|
| 32/32/31 at 6.0/5.0/4.4 only (as proposed) | 61.6 mm | 95 |
| **big 16/16/16 + small 32/32/31** | **55.0 mm** | **143** |
| 24 per rung, all six | 68.2 mm | 144 |
| 16 per rung, all six (alphabet exactly once) | 51.2 mm | 95 |
| 8 per rung, all six | 51.2 mm | 48 |
| 32 per rung, all six | 85.8 mm | OVER |
| 32/32/31 big + small rungs dealt the same | 85.8 mm | OVER |

Note the two rows that both read 51.2 mm: at 8 characters per rung every rung
still costs two rows (one per face), so halving the characters buys nothing.
**Row count, not character count, is what consumes the plate** — which is why
the big rungs are so expensive and why giving the SMALL rungs the larger share
is nearly free.

**Recommended: big 16/16/16 + small 32/32/31, 55.0 mm.** It puts the most
characters where legibility is actually in doubt, leaves 24 mm of margin against
the auto-fit-is-all-or-nothing fragility, and covers all six rungs.

## 5. Open questions

1. **Which characters go to which rung?** Codepoint order is the obvious deal,
   but it clusters punctuation at the big end and letters at the small end. An
   interleaved deal would put a mix at every rung — worth deciding deliberately,
   since the plate exists to compare glyphs.
2. **Does each rung need a label?** The size is visually obvious once the rungs
   differ, but a plate found later has no other record. The mixed-plate rule
   already learned this: see `ftProofFooterFaceMap`.
3. **Trigger name.** `BOTHPROOF!ALL` collides with nothing — the rung suffix
   parser accepts only strings that parse as a rung in `FontSizes`, so `ALL`
   currently matches nothing and stays ordinary text.
