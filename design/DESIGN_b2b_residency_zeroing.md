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

### C1 (round 0) — the scrub cannot reach what the buffer outgrew

**Folded. The round-0 Critical is not a flaw in the proposed fix; it is a defect
in code B2b already shipped.**

`op.Buffer` has no pre-sizing, so `args` grows from nil by doubling, and `Scrub`
zeroes `b.args[:cap(b.args)]` — **the current array only**. Every reallocation
orphans an array still holding every rune written before it. The reviewer
instrumented it and pulled the words back out of a 24-word seed frame:

```
seed frame len=2387 cap=3392 | reallocated=true
Residue() after Scrub = (0 args, 0 refs)
orphan text: "1: ABAN$D2O?NO5<2: ABILI(T-Y85Q3: ABLE#5f4: ABOU(T65…"
```

Thirteen words, verbatim and in index order, in an array §10.2.2 declares wiped,
scored 0 by `Residue()`. **`run_flow.go:245`'s existing wipe-path `Scrub` has the
same hole**, so moving the call was never going to deliver the property — it
would have made the normal exit equal to the wipe exit, an equality of two
incomplete wipes.

**The fix: record what the buffer outgrows, and scrub that too.** Prototyped and
measured, not proposed:

```go
type Buffer struct {
	// orphans are backing arrays this Buffer has outgrown. append REPLACES the
	// array on reallocation and the old one becomes unreachable -- still
	// holding every rune written into it.
	orphanArgs [][]uint32
	orphanRefs [][]any
	...
}

// growArgs records the outgoing array before append can replace it.
func (b *Buffer) growArgs(n int) {
	if cap(b.args)-len(b.args) < n && cap(b.args) > 0 {
		b.orphanArgs = append(b.orphanArgs, b.args[:cap(b.args)])
	}
}
```

`Scrub` then clears the orphans as well as the current arrays and drops the list;
`Residue` counts them, or a reallocated buffer keeps scoring 0 while holding the
words.

**The growth surface is bounded and measured** — 9 `args` appends and 3 `refs`
appends, all in `gui/op/op.go`, so routing them through the recorder is a local
change and not a discipline anyone must remember.

**Cost:** the orphaned arrays stay reachable until the next `Scrub` instead of
becoming garbage. That is a deliberate trade — memory retained in exchange for
memory that can be *wiped* — and it is bounded by the doubling series, ~2× the
high-water mark.

**Measured on the prototype:** a warm buffer at cap 128 grown to 10,240 records
**16 orphaned arrays holding 25,054 non-zero words**; after `Scrub`, zero.

### The scrub's position — F-107's original fix, still needed

Round 0 confirmed the placement is **safe** (§(a)): `unlockSecretSession`'s defer
runs strictly between frames, so no op still to be drawn is zeroed. It is
necessary but not sufficient — without the orphan fix above it scrubs one array
out of seventeen.

Round 0 also found the placement **insufficient in scope** (I1): §8's twelve-word
passphrase is rendered by `unlockPassphraseFlow`, **outside** this bracket, and on
the give-up routes nothing scrubs at all. So the scrub belongs on **both**
brackets — the passphrase flow's own defer (which Task 9 already installs for the
wipe guard) and the secret session's.

And I4: `run_flow.go:245` is **not** subsumed and stays. It covers the wipe path,
where the Context is abandoned rather than returned through.

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

### RE-RESOLVED after round 0 — I WAS WRONG TWICE, and the buffer IS clearable

My first filing said "zero the spline after the cut". I then re-scoped it to
"impossible, because `Curve = iter.Seq[Knot]` is a closure". **Round 0's I2 shows
the second answer was as wrong as the first**, and for a worse reason: I stopped
at the type and never looked at what the closure closes over.

`Curve = iter.Seq[Knot]` is true and **irrelevant**. A closure over an
already-materialised slice is still a clearable slice, and `engrave.PlanEngraving`
builds exactly that. Measured by the reviewer: **9 non-zero knots left in
`knotBuf` after a full cut; `clear(buf[:cap(buf)])` drives it to 0.**
`planEngraving(knotBuf, conf, e)` already exists as a caller-supplies-the-buffer
seam, with a doc comment saying so.

**Three ownable, zeroable buffers**, none of which the previous draft named:

| buffer | where | today |
| --- | --- | --- |
| `knotBuf` | `engrave.go:1016-1021` | never zeroed |
| `SafePointer.history` | `engrave.go:1637` | trimmed by `copy`+reslice at `:1675-1676`, so the tail is never zeroed |
| `splineResumer.catchup` | `gui/engraver.go:222` | never zeroed |

**The genuinely unownable part** is `appendLine`'s per-segment
`make([]bspline.Knot, len(sc))` (`engrave.go:1146`) — a fresh allocation per
segment that no caller can reach. *That* is the real argument for accepting a
residual limitation, and it is the argument the previous draft should have made
instead of claiming the whole thing was impossible.

**So both earlier framings are withdrawn:**

- **Option 1 as written would have amended the spec to assert an impossibility
  that is measurably not impossible** — on a funds path, in a document future
  work treats as settled. That is the worst outcome available here.
- **Option 2 was dismissed on a memory cost that does not exist.** The 100-knot
  allocation is already made on every `toPlate`: ~1.6 KB on the 32-bit target
  (`bezier.Point{X,Y int}` = 8 B, `Knot` = 16 B padded).

**Re-scoped fix:** zero `knotBuf`, `SafePointer.history` and
`splineResumer.catchup` at the end of the cut, and amend §2.2 to name **only**
the per-segment allocations as residual. The severity grading stands at
Important rather than Critical: the measured residue is 9 knots of the final
stroke of the last glyph, which is not seed-recoverable. **The defect is a
decision taken on false facts, not a live leak** — but the decision was about to
be written into the spec.

### The ordering hazard — proposed resolution

Round 0 left this open and it is the whole remaining difficulty: `Engrave`
returning and the engrave loop finishing with the knot buffer are **not the same
instant**. `gui.go:2651-2656` calls `Stop()` and keeps rendering, and the job
iterates `e.spline` on its own goroutine. Zeroing under a live loop corrupts a
cut — turning the machine's most ordinary recovery into a ruined plate.

**Resolution: zero inside the engrave goroutine, as its last act, immediately
before it signals completion.** The signal already exists and is unambiguous:

```
gui/engraver.go:131-144   Status() reads e.errs, and ONLY then moves the state
                          to engraveStopped / engraveDone / engraveFailed
```

The goroutine sends on `e.errs` when it is done with the spline. Anything zeroed
**before that send** is provably no longer read by the loop, on every exit —
completion, `Stop()`, and error alike, since all three converge on that send.

**Why not have the caller join and then zero.** `unlockSecretPlate` would have to
wait for a terminal state after `Engrave` returns, which means either a spin or a
timeout. A spin risks never terminating on a wedged goroutine, and **a hang on a
watchdog-less device is a brick** — the same reasoning that already guards
`WipeSecretAt`'s bounds check three lines from this code. Putting the zeroing
where the loop's completion is *locally known* removes the synchronisation
question entirely rather than answering it.

**Cost:** it places a residency concern inside the engraver, which is a layering
smell. Worth it — the alternative is a cross-goroutine lifetime invariant
enforced by a comment, which is precisely the failure mode
[[comments-outlive-their-conditions]] catalogues and which produced F-107 and
F-108 in the first place.

**What R0 must judge:** whether `e.errs` is genuinely the last read of the
spline on ALL exits — including `Resume`/`catchup` after an interruption, where
`splineResumer.catchup` is a second buffer with its own lifetime, and the
`Status()` restart path at `engraver.go:146-148` which calls `e.Start()` again.
**A restart that re-reads a zeroed buffer would cut a wrong plate**, which is
worse than leaving the bytes resident.

**Superseded framing:** the ordering hazard is unchanged and is now the
whole difficulty. `Engrave` returning and the engrave loop finishing with
`knotBuf` are not the same instant (`gui.go:2651-2656` calls `Stop()` and keeps
rendering; the job iterates on its own goroutine). Zeroing under a live loop
corrupts a cut. Round 0's M3 sharpens this further: `bspline.Measure` fills the
knot buffer at **build** time, so "for the duration of the cut" was never the
right lifetime bound in the first place.

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
