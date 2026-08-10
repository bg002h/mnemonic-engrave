# Continuity — 2026-08-10 (supersedes CONTINUITY_2026-08-09b.md)

Written at the end of a long bench-and-review day. Everything below is on disk
and committed; this file exists so none of it has to be re-derived.

## The headline: the post-wipe HANG IS CLOSED

Three post-wipe re-entries on hardware, three successes, against a build that
froze deterministically. **Fix C did it** — `seal.boundBlob` bounds the payload
read to what the header declares (~1.4 KB typical) instead of the 64 KiB region,
so the fragmented post-wipe heap can serve it. Shipped in b2b **`3de8aa1`** with
fix D (`op.Drawer.Release` + a self-maintaining `Draw`).

**Fix D is NOT what fixed it** — measured, its entire ceiling is ~12 KB
(`buf 2048/512` on the device). R0 round 0's C1 predicted exactly that and was
conservative at ~24 KB. Fix B (reuse the Context) was retired, un-retired on that
Critical, and is now moot: the hang is closed without it.

## State

| repo / branch | HEAD | state |
| --- | --- | --- |
| `mnemonic-engrave` | `3e78295` | ~15 commits today; nothing pushed |
| `seedhammer-b2b` (`b2b`) | `3de8aa1` | fixes C+D; suite green; clean |
| `seedhammer-idleprobe3` | `256b38c` | **DIAGNOSTIC** — idle overlay ON the fixes. Built, green, NOT flashed |
| `seedhammer-heapprobe2` | `64dbf6c` | **DIAGNOSTIC** — forced GC before readout. **Currently flashed** |
| `seedhammer` (main) | `a01b666` | untouched |

**Never merge any `*probe*` branch.** Heap numbers taken with a forced GC are not
comparable with ones taken without it — at baseline the un-forced count
overstated live objects by 3.5× (688 vs 193), which is what made the original
"1,567 stranded objects" figure unusable.

## Open, in priority order

**1. The residency design is RED and needs a real fold.**
`design/DESIGN_b2b_residency_zeroing.md`, reports round 0 and round 1 in
`design/agent-reports/2026-08-10-r0-residency-round{0,1}.md`.

Round 1: 2 RESOLVED, 6 PARTIAL, 6 NOT ADDRESSED, plus **1 new Critical and 3
Important introduced by my own fold**. The worklist is the round-1 report, all
fourteen findings — not the interesting ones.

**The new Critical, and it is physical:** `e.errs` ends **one goroutine's** reads,
not the job's. `Status()` nils `errs`, which re-arms `Start()`, and
`gui/gui.go:2747`'s `s.job.Start()` is the operator's hold-to-resume after Back or
a stall. Zeroing `SafePointer.history` before that send drives the resume's
catch-up motion to the origin at `T:0` — **a wrecked plate, worse than
residency**. `knotBuf` zeroing at that point IS safe and stays. The two buffers
have different lifetimes and I treated them as one.

**2. F-107 — the rendered seed survives a normal exit, and the shipped wipe is
incomplete too.** `ctx.B.Scrub()` has one caller, inside `if !wiping`, and that
branch is unreachable in production (`uiFlow` loops `for !ctx.Done`; `ctx.Done` is
set on the device by the wipe alone; `main.go:34` ranges forever). Worse, round
0's C1: `Scrub` zeroes only the **current** array, and every reallocation orphans
one still holding what came before — the reviewer read **thirteen of twenty-four
seed words back out**, verbatim and in index order, from an array `Residue()`
scores 0. **`run_flow.go:245` has this hole today**, so B2b does not deliver the
wipe it documents.

Fix prototyped and mutation-tested (see below), not yet applied to `b2b`.

**3. F-108 — I was wrong twice; the buffer IS clearable.** First "zero the
spline", then "impossible, `Curve` is a closure". Both wrong: a closure over an
already-materialised slice is still a clearable slice, and `PlanEngraving` builds
exactly that. 9 non-zero knots survive a cut in `knotBuf`; `clear` drives it to 0.
Also ownable: `SafePointer.history`, `splineResumer.catchup`. Genuinely
unownable: `appendLine`'s per-segment `make` — *that* is the real argument for a
residual limitation. **Option 1 as first written would have amended the spec to
assert an impossibility that is measurably not impossible.**

**4. F-106 — a deterministic 2× window, not "the timer never starts".** Timed from
video over three cycles: Cut/Skip → warning **6:00**, warning → wipe **29–30 s**.
The second half is exact; the first is consistently 2 × `idleTimeout`, which is
what an armed edge landing at +3:00 produces. Yesterday's "4:15 and nothing" is
what a 6:00 window looks like if you stop waiting. **`b2b-idleprobe3` is built and
ready to flash** — its `w` field decides it: `w170` means the armed edge rewrote
the clock, `w151` with `e` climbing means an event did.

**5. F-109 — ~35 K in ~81 REACHABLE objects survives every wipe, unidentified.**
Plateaus (a274 / a276 / a276 across three cycles), so bounded, not compounding.
**Not closable as a memory nit** — operator, correctly: *"for all we know that
missing 35 K is unwiped secret data."* Name the objects with the finalizer
technique, at `gui` level, no hardware needed.

## The prototype waiting to be applied

In `/tmp/claude-1000/c1-proto` (recreate rather than trust it): `op.Buffer` records
outgrown arrays in `orphanArgs`/`orphanRefs` via `growArgs`/`growRefs` before the
appends that reallocate; `Scrub` clears them and drops the list; `Residue` counts
them. Growth surface is exactly **9 `args` appends and 3 `refs` appends, all in
`gui/op/op.go`**. Measured: warm cap 128 → cap 10,240 records **16 orphaned arrays
holding 25,054 non-zero words**; zero after `Scrub`. Suite green.

## Traps that cost real time today — do not re-pay them

1. **`runtime.KeepAlive` the holder**, or Go's precise liveness collects it and a
   reachability test passes vacuously. Measured: canary provably still in
   `maskStack[2]` and `collected()` returned true.
2. **Two `runtime.GC()` calls plus a timeout**, never one — the finalizer
   goroutine has to be scheduled.
3. **A canary must land where a later frame will not overwrite it.** `Compose`'s
   masks stack in REVERSE, so a canary passed last sits at index 0.
4. **Do not let bookkeeping be the witness.** The orphan test passed under its own
   mutant because `Scrub` truncates the orphan list and `Residue` then sees
   nothing. Hold your own references and inspect the memory.
5. **Apply the patch before quoting the anchor.** The design misquoted
   `unlock_session.go`'s guard bracket as two lines (it is three), so the patch
   silently failed — and the first suite run after that was the UNPATCHED tree.
6. **`e` never decrements.** A reading where it drops is a transcription slip.
7. **Check the arithmetic of a reported line:** in-use + free must equal sys. One
   "anomaly" today was a typo I chased instead of adding up.

## Next action

Fold round 1 — **all fourteen findings**, starting with splitting `knotBuf` from
`SafePointer.history` in the ordering resolution. Then re-gate, re-dispatch, and
only then apply the prototype to `b2b`.

Nothing is pushed. Push `master` via `ci/staging` — see `CLAUDE.md`.
