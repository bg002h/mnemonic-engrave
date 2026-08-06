# RECON — the wiggle is dot pile-up at commanded cusps

**2026-08-06.** Written the session the accel/jerk plate came back null. Every
number below is measured from the fork at `e39ec30` by planning the glyph through
`engrave.PlanEngraving` — the same planner the device runs — not read off a doc
comment.

## The operator's observation, which is what redirected the investigation

> The constant `~` is more normal at larger size and far worse as size gets
> smaller, while the sh `~` looks more like a straight line at larger size and
> gets better at smaller size. Sh `~` is curvy while constant is cuspy with
> straight lines.

Both `~`s sit on the **same plate, same machine, same motion parameters**, and
their defects run in **opposite directions with size**. No motion parameter can
do that. That single sentence moves the cause out of the machine and into the
commanded geometry.

## 1. The two `~`s are different primitives

```
font/constant/constant.svg:139   <polyline id="asciitilde" points="559,6 560,4 562,6 563,4"/>
font/sh/sh.svg:249               <path     id="asciitilde" d="M372.9,3.16c0.3-0.8,0.75-0.75,1.05-0.1
                                                              c0.3,0.65,0.75,0.6,1.05-0.2"/>
```

In `cmd/vectorfont/main.go`, a polyline vertex becomes a **tripled control
point** (`spline = append(spline, k, k, k)`, `main.go:238-239`). Tripling a
control point in a uniform cubic B-spline drags the curve exactly through it with
zero tangent length — **a cusp**. A `<path>` cubic instead goes `appendBezier` →
`bezier.Sample` → `bspline.InterpolatePoints` (`main.go:186`), producing a smooth
interpolated spline.

So constant's `~` is not a tilde. It is a **3-segment zigzag with 3 sharp
corners**.

**Correction worth recording:** the `Line` flag on a `vector.Knot` means
**pen-down**, not "straight" — `ControlPoint(k.Line, …)` maps it to
`lineCmd`/`moveCmd` and `bspline.Knot` calls the same field `Engrave`. Both faces
are mostly `Line:true`. The cusps live in the *tripling*, not the flag. This was
misread on a first pass and caught by measuring.

## 2. `font/constant` contains no curves at all

A knot count that is an exact multiple of 3 means every control point is tripled.
Measured over all 94 glyphs of each face:

```
font/constant   94 glyphs   94 all-polygon    0 curved
font/sh         94 glyphs   63 all-polygon   31 curved  &2369?BCDGOPQSabcdfhjnopqstu{}~
```

```
constant.svg   104 <polyline>    7 <line>     0 <path>
sh.svg          25 <polyline>   53 <line>    66 <path>
```

`O` in `font/constant` is a **9-sided polygon**. `o` is a **pentagon**. `8` is a
16-gon. This is why the sixteen problem glyphs are exactly the round ones,
`aeszOo8@*&<>(){}`.

## 3. The physical mechanism: a fixed-rate needle over a varying-speed path

`cmd/controller/platform_sh2.go:154` — `needlePeriod = 25 * time.Millisecond`.
The solenoid is driven by its own PIO state machine
(`driver/mjolnir2/mjolnir2.go`) on a **fixed period, independent of motion**.

**The needle fires at exactly 40 Hz. Motion decides only where each strike
lands.** So

```
dot spacing = feed rate x 25ms        8 mm/s -> 0.20mm      4 mm/s -> 0.10mm
```

against a **0.30 mm** stroke. That is the whole "the machine is a hammer" note in
one equation, and it is why hard@4 looked so much better than hard@8.

## 4. Measured speed profile — `~` at 3.0 mm, stock params

Per engraved B-spline segment, from `bspline.Segment.Knot`'s cubic and tick
count, at `sh2.Params()`:

```
const '~': 12 engraved segments, path 2.433mm, 605.1ms, ~24 dots
  seg    len(mm)  time(ms)      mm/s   dot gap
  0       0.0621    50.427     1.231    0.0308   <- cusp
  1       0.3104    50.427     6.155    0.1539
  2       0.3107    50.427     6.161    0.1540
  3       0.0619    50.427     1.228    0.0307   <- cusp
  4       0.0785    50.427     1.557    0.0389   <- cusp
  5       0.3931    50.427     7.795    0.1949
  6       0.3927    50.427     7.788    0.1947
  7       0.0788    50.427     1.562    0.0390   <- cusp
  8       0.0621    50.427     1.231    0.0308   <- cusp
  9       0.3104    50.427     6.155    0.1539
  10      0.3107    50.427     6.161    0.1540
  11      0.0619    50.427     1.228    0.0307   <- cusp

sh '~': 9 engraved segments, path 1.349mm, 433.5ms, ~17 dots
  speeds 1.58 - 3.97 mm/s, dot gaps 0.0394 - 0.0994mm, no cusp
```

**Every segment takes the same 50.427 ms.** The planner spends equal time per
knot, so where the segment is short the tool crawls. At a cusp the tool runs at
**1.2 mm/s laying dots 0.031 mm apart** — a 0.30 mm dot struck two or three times
inside 0.06 mm — while the straights run at **7.8 mm/s with dots 0.195 mm apart**.
A **6.3x** swing in dot density inside one 1.33 mm glyph.

## 5. The model predicts both things the operator already saw

| case | segs | path mm | ms | vmin | vmax | ratio | gap min | gap max | dots |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| const `~` 3.0mm stock | 12 | 2.434 | 605.1 | 1.231 | 7.791 | **6.3x** | 0.0308 | 0.1948 | 24.2 |
| const `~` 6.0mm stock | 15 | 4.867 | 832.8 | 1.490 | 11.315 | 7.6x | 0.0373 | 0.2829 | 33.3 |
| const `~` 3.0mm **halved a/j** | 12 | 2.434 | 762.4 | 0.977 | 6.184 | **6.3x** | 0.0244 | 0.1546 | 30.5 |
| sh `~` 3.0mm stock | 9 | 1.349 | 433.5 | 1.576 | 3.974 | **2.5x** | 0.0394 | 0.0994 | 17.3 |
| sh `~` 6.0mm stock | 9 | 2.697 | 546.2 | 2.500 | 6.311 | **2.5x** | 0.0625 | 0.1578 | 21.8 |
| const `\|` 3.0mm stock | 5 | 2.667 | 444.3 | 1.332 | 8.000 | 6.0x | 0.0333 | 0.2000 | 17.8 |
| const `O` 3.0mm stock | 34 | 5.886 | 1599.0 | 0.977 | 8.001 | **8.2x** | 0.0244 | 0.2000 | 64.0 |

**a. The null accel/jerk result was predictable.** Halving acceleration and jerk
leaves the ratio at **6.3x, unchanged to one decimal**. Everything got 26% slower
and every dot gap shrank by the same 21%. The *unevenness* — which is the visible
defect — did not move at all. Halving a motion parameter cannot fix a *ratio*; it
only rescales it. **This model would have predicted the null before the plate was
cut.**

**b. The size dependence.** The pile at each cusp is **~2 dots of 0.3 mm inside
~0.06-0.08 mm, at both rungs** — its absolute size is set by the needle period
and the dot diameter, neither of which scales with the glyph. The glyph does
scale: `~` is 1.33 x 0.67 mm at 3.0 mm and 2.67 x 1.33 mm at 6.0 mm. So the same
absolute blob is **half the glyph at 3.0 mm and a quarter of it at 6.0 mm**.
Worse as size shrinks, exactly as observed.

**c. `\|` is the control that proves the refinement.** It has the same tripled
knots and nearly the same 6.0x speed ratio, and it is **clean** — the fact
already recorded in `CONTINUITY_2026-08-06.md` §3. Uneven dot spacing along a
*straight* line is invisible, because every dot lands on the same line. **The
defect is not uneven dot spacing; it is uneven dot spacing AT A DIRECTION
CHANGE**, where the pile lands off to one side and deforms the corner.

**d. `O` is the worst glyph in the face and has not been drawn yet.** As a
9-sided polygon it has 9 cusps, an 8.2x ratio, and takes 1.6 s. `O`, `o` and `8`
are the three glyphs still open of the sixteen.

## 6. What is measured and what is not

Measured: the primitives, the tripling, the face-wide polygon fact, the needle
period, the per-segment speed and dot-gap profiles, the invariance of the ratio
under halved acceleration, and the `|` control.

**Not yet measured:** that the piled dots are what the eye reads as the wiggle on
steel. The geometry and timing say the ink must pile at the corners; only a plate
says the pile is the artefact the operator has been looking at. That is what the
single-character test plates are for.

## 7. The single-character test-plate protocol

**Operator directive, 2026-08-06:** engrave **one character at a time**, at the
**top-left-most position**, and **do not centre it**.

No code change is needed. In `backup/freetext.go` the *title* is centred
(`centerInset`, `freetext.go:125`) but a *body row* is left-aligned at
`margin + offx` (`freetext.go:153`), and with no title the first body row sits at
the top margin. So:

```
go run ./cmd/plateview -plate freetext -face const -text '~' -size 3.0 -o /tmp/p.png
```

renders one glyph at the top-left, left-aligned, **~2 s to engrave**. Pass no
`-title` and no `-footer`. `-face` selects `const` or `sh`; `-size` pins the rung
instead of letting auto-fit choose 6.0 mm.

Why it is the right instrument: at ~2 s a comparison costs nothing against the
~21 minutes a full plate costs, the glyph is isolated so no neighbour or retrace
can be blamed, and a fixed left-aligned origin means successive cuts are
positionally comparable — a centred title moves with the string's width and
would not be.

## 8. The fix this points at, and its one real gate

`cmd/vectorfont`'s `<path>` parsing is face-agnostic shared code, so
`constant.svg` **can carry cubics today**; nothing forbids it and
`font/constant/glyph_rules_test.go` does not constrain the primitive.
`maxSplineKnots = 100` (`engrave/engrave.go:1017,1267`) is a preallocation, not a
cap — exceeding it costs one allocation on TinyGo, not a failure.

**The gate is the constant-time property.** Curving changes path length and
therefore duration, and equalising duration across `constantAlphabet` is the
face's entire reason to exist. Run count `k` is unaffected by curvature (it counts
pen-lifts), and **max k = 2 is a security property** pinned by
`TestPassphraseRunPartition`. Verify duration equalisation still holds before
curving anything.

Order matters: `O`, `o` and `8` are the worst possible candidates to draw as
polygons, and they are the three still unwritten. **Settle whether the face gets
curves before they are drawn, not after.**
