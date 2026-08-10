# Residency zeroing — the rendered seed and the cut plate

**Status:** design, pre-R0. Written 2026-08-10.
**Owns:** F-107 (`ctx.B` scrubbed only on the wipe path) and F-108 (`plate.Spline`
never zeroed after the cut). Both Critical, both B2b.

Two defects, one shape: **a copy of the seed that a comment says is bounded, with
nothing in the code enforcing the bound.** They are designed together because
they share a hazard — zeroing a buffer that is still in use — and because fixing
one and not the other leaves §10.2.2's wipe-by-any-route guarantee just as false.

## F-107 — the rendered seed survives a normal exit

`ctx.B.Scrub()` has exactly one call site, measured:

```
$ grep -rn "\.Scrub()" --include="*.go" . | grep -v _test.go
gui/run_flow.go:245:			ctx.B.Scrub()
```

and it sits inside `if !wiping { return }` — the §10.2.4 wipe path only.

**The other branch is unreachable in production.** `uiFlow` loops `for !ctx.Done`
(`gui.go:1612`); on the device `ctx.Done` is set by the wipe alone, because the
`!yield()` route needs the consumer to stop ranging and
`cmd/controller/main.go:34` is `for range gui.Run(p, ver) {}`. So `runWithFlow`
never returns on hardware, and a UI-level "normal exit" is the flow walking back
to the start screen **with the same `Context` and the same `op.Buffer`**.

`Buffer.Reset()` runs per frame and does not zero:

```go
func (b *Buffer) Reset() {
	b.args = b.args[:0]     // TRUNCATE
	clear(b.refs)
	b.refs = b.refs[:0]
}
```

`op.Glyph` encodes every rendered rune into `args`, so — in `Scrub`'s own words —
*"on the SeedScreen path the twelve words come back VERBATIM AND IN ORDER from
the backing array."* On a normal exit nothing zeroes them; they persist until
later frames overwrite those indices, and a start screen writes far fewer args
than a seed screen, so the later words survive longest.

**The exposed path is the common one.** Read your words, press back. The
protected path — walk away for 3:30 — is the rare one.

### The fix

Scrub when the **secret session** ends, not only when the process-level wipe
fires. `unlockSecretSession` already brackets exactly that lifetime — it is where
§10.2.4's guard is installed and removed (`unlock_session.go:87-90`, quoted
verbatim after an earlier draft misquoted it as two lines and the patch failed to
apply — the anchor is three):

```go
prev := ctx.wipe
g := &wipeGuard{}
ctx.wipe = g
defer func() { ctx.wipe = prev }()
```

Add the scrub to that same defer, **after** the guard is restored:

```go
defer func() {
	ctx.wipe = prev
	// §10.2.2 wipe-by-ANY-route. The records are cleared by their own
	// defers; this is the RENDERED copy -- op.Glyph writes every drawn rune
	// into ctx.B.args, and Buffer.Reset only truncates. Until 2026-08-10
	// this ran solely on the §10.2.4 wipe path (run_flow.go:245), so an
	// operator who read their seed and pressed Back left the twelve words in
	// the backing array. That is the COMMON exit; the wipe is the rare one.
	ctx.B.Scrub()
}()
```

**The hazard, and why this is safe here.** `Scrub` zeroes to capacity, so it must
not run while an op built into `ctx.B` is still going to be drawn.
`Context.Frame` resets `ctx.B` *after* the frame callback returns
(`gui.go:88`), and this defer runs when `unlockSecretSession` returns — strictly
between frames, with the next frame's content not yet built. Same position in the
frame cycle as the existing `run_flow.go:245` call, which has been running there
since B2b.

**What it deliberately does not do:** scrub on *every* screen exit. The seed is
only ever rendered inside the secret session, and a per-screen scrub would zero
the buffer under ordinary navigation for no gain.

## F-108 — the plate's geometry survives the cut

`unlock_session.go:239` states the limitation and its bound:

> `LIVE    plate.Spline, for the duration of the cut. It IS the seed rendered as
> geometry and must exist while the needle moves. F-83, accepted.`

**"For the duration of the cut" is the justification; nothing ends it.** There is
no `clear` of a plate or spline anywhere in `gui/` — the three matches are two
comments and a constructor. When `scr.Engrave(...)` returns, `plate` goes out of
scope, becomes garbage, and is never zeroed. TinyGo does not zero on free.

The same function already demonstrates the correct pattern for the *record*, and
explains why the timing matters (`unlock_session.go:195-203`): `clear(rec)` runs
**before** `Engrave`, precisely so the seed is not resident for the ~21-minute
cut. That reasoning was never carried to the geometry, which is the other copy of
the same secret.

### RESOLVED BEFORE REVIEW: the spline cannot be zeroed at all

The previous draft sent R0 a question a tool answers in one grep, which is the
exact division of labour this project forbids. Answered here instead:

```
$ grep -rn "type Curve" bspline/bspline.go
bspline/bspline.go:22:type Curve = iter.Seq[Knot]
```

`Plate.Spline` is `bspline.Curve` = **`iter.Seq[Knot]`, a closure**. There is no
buffer to `clear`. **`clearSpline(plate)` cannot be written**, and F-108's fix as
I first filed it does not exist.

**So the finding changes shape.** What is true:

- The **reference** is already dropped promptly — `plate` is a local, and
  `unlockSecretPlate` returns immediately after `Engrave`. The lifetime is
  bounded by the function, not by the session.
- What is **not** true is that the memory is zeroed. TinyGo does not zero on
  free, so the geometry — control points that encode the seed — lingers in the
  heap as garbage until the allocation is reused. That is real residency, and it
  is exactly F-83's original point: *"a `[]byte` pipeline would relocate the
  secret rather than remove it, because the spline still encodes it."*
- The record is separately fine: `clear(rec)` runs **before** `Engrave`, and the
  spline does not read `rec` during the cut — the geometry was computed into the
  closure beforehand, which is why that early clear is sound.

**So the operator's correction still stands, but its remedy is not a `clear`.**
The exemption genuinely is time-boxed to the cut; what expires at the end of the
cut is the *justification*, and no code needs to change for the reference to
drop. What remains is that **the bytes are unzeroable by construction**.

**Three honest options, none of them a one-liner:**

1. **Accept and document.** The geometry is unzeroable garbage after the cut,
   bounded by heap reuse. This is what F-83 already accepts *during* the cut; the
   post-cut window is the part §2.2 never named. Cost: a spec amendment (F-85),
   not code.
2. **Materialise the geometry into an ownable buffer** so it CAN be zeroed —
   plan the knots into a slice the caller owns, iterate that, zero it after. This
   is the fix that actually removes the secret, and it is a real design change to
   the engrave pipeline with a memory cost on a device with 283 K free.
3. **Force reuse.** Allocate the next plate over the same memory. Fragile and
   unverifiable; listed to be dismissed.

**Recommendation: 1 now, 2 evaluated for a later phase.** Option 1 is honest and
costs nothing; option 2 is the only one that makes the guarantee true, and it
deserves its own cycle rather than being smuggled into a residency fix.

**F-108 is therefore re-scoped from "add a missing clear" to "the spec claims a
wipe-by-any-route guarantee the geometry cannot satisfy, and never says so".**

## Tests that can fail

1. **F-107, `gui`.** Drive `runWithFlow` through a real unlock to the seed screen,
   exit **normally** (no wipe), then assert `ctx.B.Residue()` reports no seed
   material — the existing `Residue()` accessor exists for exactly this. **Must
   fail before the fix**: today that path never scrubs. Mutation row: delete the
   `ctx.B.Scrub()` from the session defer.
2. **F-107 mutation row.** `Scrub` → no-op must be killed by (1). If it is not,
   the assertion is looking at the wrong buffer.
3. **F-108, `gui` — a LIFETIME assertion, since there are no bytes to check.**
   A finalizer canary on the plate: engrave under a test platform, let the job
   finish, force collection, assert the plate is unreachable. That pins the one
   property option 1 actually promises — the reference does not outlive the cut —
   and it would catch a future change that parks a finished plate in a field.
4. **Neither test may assert on a buffer it also holds a reference to.** The
   `op` canary work established the trap: `runtime.KeepAlive` the holder, use two
   `GC()` calls plus a timeout, and choose the canary so it enters the structure
   under test.

## What R0 should attack

1. **F-108's ordering.** Is there any path where the engrave loop still reads
   `e.spline` after `Engrave` returns? The abort path is the obvious one; are
   there others — a paused plate resumed from the spline, a job that outlives its
   screen?
2. **Is option 1 acceptable?** RESOLVED that the spline is unzeroable; NOT
   resolved is whether documenting that is good enough for a funds path, or
   whether option 2 (materialise into an ownable buffer) must be scheduled before
   the release tag. This is the judgement call the note most needs reviewed.
3. **F-107's scrub position.** Does any op built into `ctx.B` outlive
   `unlockSecretSession`'s return? The claim is that the defer runs strictly
   between frames; verify it against `Context.Frame`'s reset point rather than
   accepting it.
4. **Does F-107's fix subsume `run_flow.go:245`?** Both would then run on a wipe.
   `Scrub` is idempotent, so this is a question about clarity, not correctness.
5. **What else renders a secret?** The fix is scoped to the secret session because
   that is where the seed is drawn. If any other screen can render seed material —
   the BIP-39 password flow, the plate list — the scope is wrong.

## Gate coverage

The Go blocks here are modifications, so `plan-build-gate-go.sh` reaches them at
**TIER 2 (syntax) only**. Unlike the previous design, the code is **not** yet
applied to a scratch fork and type-checked, because F-108's fix cannot be written
until the `plate.Spline` question is answered — an invented `clearSpline` would
compile-check nothing real. **F-107's fix IS gated** — applied to a
scratch fork and run through both toolchains:

```
go build ./gui/...                       clean
go test ./gui/ ./gui/op/ ./seal/         ok / ok / ok
tinygo -target pico-plus2 -gc precise    1313768 flash / 60584 ram
```

Note what that does NOT prove: no existing test covers this property, so green
means "nothing broke", not "the fix works". Test 1 below is what must fail first.
**F-108 no longer has code to gate** — its remedy is a spec
amendment, not a patch. Stated so the brief does not imply coverage that does not
exist.

Resolved against the source and quoted, not paraphrased: the single `Scrub` call
site, `Buffer.Reset`'s body, `uiFlow`'s loop condition, `main.go:34`,
`unlock_session.go`'s guard bracket and its `LIVE plate.Spline` inventory line.
