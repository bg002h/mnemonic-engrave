# Toolpath equivalence — the residency change does not move the needle

**Question:** does F-108's zeroing alter the geometry the machine cuts?

`cmd/plateview` renders a plate from **the same `FitBlocks`/`EngraveFitted`/
`PlanEngraving` calls the firmware makes**, stroked at the production 0.3 mm cut
width — "what you see is the cut". So a byte-identical render is a byte-identical
toolpath.

Rendered every named plate on both trees: `seedhammer-b2b` @ `3de8aa1`
(baseline) and `seedhammer-gate-orphan` @ `231a222` (branch `b2b-residency`).

| plate | baseline sha256 (16) | residency | verdict |
| --- | --- | --- | --- |
| `seed` (carries the QR) | `f95cdb7fbca1b9e4` | `f95cdb7fbca1b9e4` | IDENTICAL |
| `bothproof` | `d366e479deacbee7` | `d366e479deacbee7` | IDENTICAL |
| `textproof` | `f03dc9bf415d0cfa` | `f03dc9bf415d0cfa` | IDENTICAL |
| `passphrase` | `f055c6e38ad18894` | `f055c6e38ad18894` | IDENTICAL |
| `constproof` | `022cfcbd5aba6307` | `022cfcbd5aba6307` | IDENTICAL |

`plateview` ranges the real `bspline.Curve`, so the `defer` added inside
`planEngraving`'s closure **does fire** on every one of these runs. The geometry
is unchanged with the zeroing live.

## What this proves

**Item (1) — zeroing `knotBuf` at iterator exit — does not alter the toolpath any
caller reads.** That was the largest of the remaining hardware questions, and it
is now answered on the host across five plates including the seed plate.

## What this does NOT prove, and hardware is still required for

1. **The RESUME path.** `plateview` ranges the curve once and never resumes, so it
   exercises neither `SafePointer.history` nor `splineResumer.catchup`. The
   wrong-plate risk lives exactly there: abort mid-plate, hold to resume, and the
   catch-up motion must still land on the safe point. Host tests cover the knot
   values (`TestPlanEngravingRematerialisesAfterZeroing`,
   `TestReleaseResumeStateOnlyClearsAnAbandonedJob`); nothing covers real
   stepper motion.
2. **Physical cutting** — steps, timing, jerk limits, and whether the needle
   actually tracks the commanded path.

**So the hardware session narrows to one scenario rather than a general
regression hunt: engrave a plate, abort mid-cut, resume, and inspect the seam.**
