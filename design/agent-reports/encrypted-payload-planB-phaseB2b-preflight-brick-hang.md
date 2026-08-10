# Pre-flight review — B2b, one lens: CAN THIS BRICK OR HANG THE MACHINE?

- **Diff:** `git -C /scratch/code/shibboleth/seedhammer-b2b diff a01b666..b2b` (6 commits, 7 tasks)
- **Plan:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md`
- **Spec:** `design/SPEC_encrypted_payload_delivery.md` §10.2.4
- **Reviewer:** independent context, read-only on both repos, nothing committed
- **Question answered:** only "can the operator end up at BOOTSEL / a power cycle / a ruined
  plate". Plan correctness, mutation rows, the build, and the residency/funds lens were
  declared settled in the brief and were NOT re-derived.

## Verdict

**Nothing in this diff can brick or hang the SeedHammer II on the paths the operator will
run tonight.** The four things the brief asked me to hunt — range-over-func panics, a
spinning session-restart loop, a busy-spin in the warning/saver branches, and a blocking
UI goroutine — are each closed, and the reasons are structural rather than incidental.
The proofs are in "What I cleared, and how" below.

One **Important** finding is a *latent* hazard, not a live one: this diff makes `ctx.Done`
a real production signal for the first time, and the invariant the whole unwind rests on —
"every loop that can be on the stack while `Done` is true tests `Done`" — is enforced by
nothing. Two counter-examples exist in the same package. Both are provably out of reach of
the only code that can set `Done` today, so they do not block the flash; they block the
next widening of `armed()`.

The remaining findings are Minor/Nit.

---

## Findings

### F1 — Important — two `for { … Engrave() … }` loops have no `ctx.Done` test and become 100 %-CPU hangs the moment `Done` is reachable on their stack

**Location**

- `gui/gui.go:2251-2256` — `backupSeedStringFlow`
- `gui/slip39_polish.go:506-510` — `engraveSLIP39Verbatim`

```go
// gui/gui.go:2251
for {
	completed := NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)
	if completed {
		return
	}
}
```

**The checkable claim.** `EngraveScreen.Engrave` (`gui/gui.go:2714`) is `for !ctx.Done { … }`
followed by `return false` (`gui/gui.go:2777`). With `ctx.Done` true it therefore returns
`false` without drawing, without yielding and without touching the clock. Both call sites
loop until it returns `true`. Each iteration allocates a fresh `EngraveScreen` +
`engraveJob` (`NewEngraveScreen`, `gui/gui.go:2700`) and returns instantly. The result is a
tight loop on the single UI goroutine that never calls `yield()`, never reaches
`pl.AppendEvents`, and never calls `ctx.WakeupAt` — no frame is ever drawn again and the
CPU never sleeps.

Contrast the shapes that are safe, and note how narrow the difference is: every other
engrave-retry loop in the package is gated on a screen that *reports* cancellation
(`cs.Choose` returning `ok == false`), which `Done` also produces —
`gui/unlock_platelist.go:228`, `gui/bundle_flow.go:344`, `gui/derive_xpub.go:279`,
`gui/gui.go:2158`, `gui/gui.go:2196`, `gui/gui.go:2259`, `gui/bip85.go:299`. These two are
the only ones that loop on `Engrave`'s completion boolean alone.

**Reachability today: NOT reachable. The operator can flash.** `ctx.Done` is assigned in
exactly three places, all in `gui/run_flow.go`:

| line | setter | live in production? |
| --- | --- | --- |
| `run_flow.go:77` | `if !yield(o) { ctx.Done = true }` | no — `cmd/controller/main.go:34` is `for range gui.Run(p, ver) {}` with an empty body, so the generated yield never returns false |
| `run_flow.go:185` | `wipeNowHook` | no — the var has no non-test assignment (`grep -rn "wipeNowHook\s*=" gui/*.go` outside `_test.go` is empty) |
| `run_flow.go:205` | §10.2.4's timer | yes — but only when `armed` |

`armed` is `ctx.wipe.armed()` (`run_flow.go:140`), and `ctx.wipe` has exactly one non-test
assignment: `gui/unlock_session.go:83`, scoped by `defer func() { ctx.wipe = nil }()` on
line 84 to the body of `unlockSecretSession`. Neither `backupSeedStringFlow` nor
`engraveSLIP39Verbatim` is in that call tree (the secret session reaches only
`ChoiceScreen.Choose`, `SeedScreen.Confirm`, `showModal`, `EngraveScreen.Engrave` — and
`unlockEngraveCodex32`'s own doc comment records that it deliberately does not delegate to
`backupSeedStringFlow`). And `uiFlow`'s dispatch loop is `for !ctx.Done`
(`gui/gui.go:1612`), so after the unwind it exits *before* dispatching another program —
no flow is ever *entered* with `Done` already true except the tail of the unlock flow,
which is clean (see below).

**Operator impact if it ever does become reachable.** Frozen screen, no repaint, touch
dead, no watchdog. Power cycle at best; mid-cut it is a ruined plate. This is the exact
failure class the brief names, and it is one `armed()` widening away — e.g. arming the
timer for the typed-seed path, which holds the same twelve words in the same `SeedScreen`.

**Suggested fix.** `for !ctx.Done {` at both sites (2 lines), plus a test or lint that
asserts no engrave-retry loop is unguarded, so the invariant the unwind depends on is
checked rather than remembered.

---

### F2 — Minor — `wipeGuard.armed()` is a *mutating* call into the engrave state machine, executed on every Run tick (up to 25 Hz under the screensaver)

**Location** `gui/wipe_guard.go:44-51` → `gui/engraver.go:126-146`.

**The checkable claim.** `armed()` reads like a predicate and calls `j.Status()`, which is
not one. `engraveJob.Status()` (`gui/engraver.go:126`) drains `e.progress` into
`e.status.Completed`, drains `e.errs`, sets `e.errs = nil`, transitions `e.status.State`,
and ends with:

```go
if e.status.State == engraveRunning {
	// Restart if requested.
	e.Start()
}
```

Before this diff, `Status()` had exactly one caller — `EngraveScreen.Engrave` — and while
the screensaver had the flow parked it was not called at all. It now also runs from Run's
own tick, on the same goroutine but at a cadence the engrave screen does not control.

**Why it is benign today, verified rather than assumed.** `e.status.State` and `e.errs` are
assigned only at `engraver.go:82-113` (`Stop`, `Start`) and `engraver.go:132-146`
(`Status`'s errs drain). `Start` sets `errs` and `State = engraveRunning` together and
returns early when `errs != nil`; `Status`'s drain sets `errs = nil` and `State` to
`engraveStopped`/`engraveDone`/`engraveFailed` together. So `State == engraveRunning`
implies `errs != nil`, which makes the restart branch dead code — `e.Start()` returns on
its first line.

**Impact if that invariant is ever broken.** Run's *idle timer* would call `e.Start()` and
spawn a second engrave goroutine, with no operator action and the needle down. That is the
one failure mode in this diff that ruins a plate and possibly the head. The invariant is
currently held by two functions in one file agreeing, with nothing naming it.

**Suggested fix.** Have `armed()` read a cached state (the engrave screen already calls
`Status()` every frame) rather than calling `Status()`, or move the restart branch out of
`Status()` so a predicate cannot reach it.

---

### F3 — Minor — a hung engraver leaves `armed()` false forever, so §10.2.4's timer silently never arms

**Location** `gui/wipe_guard.go:46-50`; `gui/engraver.go:84-93` (`Stop`), `:157-163` (the
worker's deferred `Close`); `cmd/controller/platform_sh2.go:612-628`.

**The checkable claim.** `armed()` returns false for `engraveRunning` **and**
`engraveStopping`. `engraveStopping` is set by `Stop()` and only ends when `e.errs`
delivers, which happens after `runEngraving`'s deferred `d := <-e.lock; d.Close()`. On the
SH2, `Close` is `homingEngraver.Close` — `Dev.Reset()`, `Dev.Flush()`, then `home()`, i.e.
real head motion with no timeout in that path. If the axis stalls or the cable is pulled
such that `Flush`/`home` never returns, the state stays `engraveStopping` indefinitely,
`armed()` stays false, and the 3-minute timer never starts — while the decrypted seed's
plate spline and the `SeedScreen` state are still resident and the screensaver has the flow
parked.

**Operator impact.** Not a brick: the UI still ticks at the saver's 40 ms cadence, a touch
still refreshes `a.idle.start` and un-parks the flow (`run_flow.go:145`), and Back still
leaves the screen. What is lost is the guarantee: "3 minutes and it is gone" silently does
not hold in exactly the failure mode where the operator is most likely to walk away to
fetch tools. Worth knowing before the first real cut; not worth holding the flash.

---

### F4 — Minor — the diff's own comment claims the abandoned Context's buffer "is already zeroed"; that is true of `refs` only, not of `args`

**Location** `gui/run_flow.go:235-245` (the session loop's closing comment), against
`gui/op/op.go:374-378`.

**The checkable claim.** The comment reads: *"clear(b.refs) (gui/op/op.go:376) runs on the
last frame drawn … The abandoned Context's buffer is already zeroed by the time control
reaches this line."* But:

```go
// gui/op/op.go:374
func (b *Buffer) Reset() {
	b.args = b.args[:0]   // re-sliced, NOT cleared
	clear(b.refs)
	b.refs = b.refs[:0]
}
```

and `op.Glyph` puts the **rune itself** in `args`:

```go
// gui/op/op.go:132
return MaskOp{encodeOp(b, opMask, 0, []any{glyphImage, face}, uint32(r))}
```

So after `Reset()` the backing `[]uint32` still holds the last drawn frame's glyph stream
in draw order. For a frame parked on `SeedScreen.Confirm` — which is precisely the frame
this comment is reasoning about, and which `run_flow.go:20-27` names by hand as "the twelve
words" — that is the words' letters. The same applies to `a.warnBuf`, which outlives every
session.

Not a brick, not a behaviour change (`op.Buffer` has always worked this way), and outside
the funds lens I was scoped to. The reportable part is that the **claim** is new and a
future reader will rely on it when deciding there is nothing to scrub — which is the
failure mode this project's own "records are the weak half" rule exists to catch.

**Suggested fix.** Either `clear(b.args)` in `Reset` (measure the per-frame cost first) or
narrow the comment to "refs are cleared; the arg stream is re-sliced, not zeroed".

---

### F5 — Nit — the armed window covers `SeedScreen.Confirm`, where a 3-minute pause is the normal operator behaviour

`ctx.wipe` is live for all of `unlockSecretSession` and `g.job` is nil until
`scr.Engrave` (`gui/unlock_session.go:200-205`, `:309-314`), so the twelve/twenty-four-word
confirm screen is **armed**. Comparing 24 words against a written copy without touching the
panel for 3 minutes is entirely plausible; the operator then gets 30 seconds of warning and
loses the session. Recovery is twelve words plus a ~31 s KDF, with the sealed blob untouched
in flash — fully recoverable, and the plan records this as deliberate (row 2 as amended
arms the walk-away states on purpose). Flagged only because it is the first thing the
operator will actually hit on the real machine, and the warning copy — "Touch the screen to
keep it" — is the only thing standing between them and a re-unlock.

---

## What I cleared, and how

Stated so the next reader does not re-derive it.

**Range-over-func panic — closed, structurally.** Go panics if `yield` is called after it
returns false. `yield` is held only by `ctx.FrameCallback` (`run_flow.go:56-81`), which
early-returns on `ctx.Done` before calling it (`:73`). It returns false only when the range
body executes `return` (`:130`), which returns from `runWithFlow` entirely; the wipe uses
`break` (`:186`, `:206`), after which the range body completes normally and yield returns
**true**. All 54 `ctx.Frame` sites reached during the unwind therefore hit
`gui/gui.go:79`'s nil-safe callback, take the early return, and still run `c.B.Reset()`.
No path calls yield twice.

**Session restart loop — cannot spin.** `if !wiping { return }` (`run_flow.go:233`) and
`wiping` is reset per session (`:53`). `wiping` is only set inside the range body, which
requires a `ctx.Frame` from the flow, which requires the flow to run. A flow that returns
without ever drawing leaves `wiping` false and Run exits. A second wipe needs another
3 min 30 s of idle. Also checked the other direction: `ctx.Done ⟹ wiping` in production
(the only other setter, `:77`, is unreachable — see F1's table), so `Run` returning →
`run()` returning → `main` returning → a dead machine is not reachable either.

**Busy-spin — closed.** `pl.AppendEvents` always receives a future deadline: the warning
sets `now+1s` (`:219`), the saver `now+40ms` (`:225`), the normal exit `idleWakeup`
(`:228`). The one way `WakeupAt` can be defeated is `ctx.Reset()` pinning `Wakeup` to
`time.Now()` when `Router.Reset()` returns true (`gui/gui.go:110-116`) — but
`EventRouter.Reset` (`gui/event.go:280-292`) clears `r.filters` each call and discards
every event no filter matches, and while the flow is parked no new filters are ever
registered. So it returns false from the second parked tick onward: at most one extra
immediate iteration, then a real sleep. Verified against both branches.

**Blocking the UI goroutine — closed.** The only blocking join in the flow is
`StartScreen.Flow`'s `defer { close(closer); r.Close(); <-closed }` (`gui/gui.go:1697`),
and `StartScreen.Flow` is never on the stack while `ctx.wipe != nil`. The engrave worker is
never joined (`Stop()` closes a channel and returns, `gui/engraver.go:84`). The KDF is
sliced across frames and runs strictly *before* `unlockSecretSession`
(`gui/unlock_flow.go:106-114`), so `armed` is false for its whole duration and Task 5's
`ctx.KeepAwake()` (`gui/unlock_kdf.go:302`) can never interact with the wipe.

**Nil derefs — closed.** `ctx.wipe.armed()` has a nil-receiver guard
(`gui/wipe_guard.go:45`); `g.job` is nil-checked (`:47`) and is only ever assigned
`scr.job`, which `NewEngraveScreen` always populates; `a.warnBuf` is a struct field, so
`&a.warnBuf` is always valid. `wipeWarningOp` (`gui/wipe_warning.go:41-61`) does no slice
indexing of its own, clamps `secs` at 0, and is called only when `wipeAt.Sub(now) > 0`;
its widths are `480-32` and `480-16` at `lcdWidth`.

**`panic(err)` on `pl.Dirty` — unreachable on SH2.** `draw` panics on a `Dirty` error
(`run_flow.go:98`) and the warning now takes that path once per second. But
`Platform.Dirty` (`cmd/controller/platform_sh2.go:638-657`) returns only
`p.lcdDev.BeginFrame(r)`, and `ili9488.Device.BeginFrame`
(`driver/ili9488/ili9488.go:207-219`) returns `nil` unconditionally. There is no error to
panic on.

**No mid-cut wipe.** `armed()` is false for `engraveRunning`/`engraveStopping`, it is
evaluated once per tick (`:140`) with the flow parked so nothing can start a job between
that read and the `break`, and `s.job.Start()` sets `State = engraveRunning` synchronously
before spawning the worker (`gui/engraver.go:104-113`). Further, whenever a wipe *can*
fire, the engraver hardware is already closed: `engraveDone`/`Stopped`/`Failed` are only
reached when `e.errs` delivers, and `errs` is sent after `runEngraving`'s deferred
`d.Close()`; `engraveIdle` never opened it.

**The unwind and the post-unwind path terminate.** Traced every loop from the parked frame
out: `ChoiceScreen.Choose` (`gui/gui.go:1465`), `SeedScreen.Confirm` (`:2353`, plus its
nested `:2369` and `:2397`), `showModal` (`gui/slip39_polish.go:25`),
`EngraveScreen.Engrave` (`:2720`, plus `:2744`) — all `for !ctx.Done`.
`unlockSecretSession`'s `for _, i := range at` is bounded by ≤ 24 records and each
iteration is straight-line with its `WipeSecretAt` defer, so the unwind *finishes* the wipe
rather than skipping it. `unlockPlatesOrNotice` is then **entered** with `Done` already true
(`gui/unlock_flow.go:115`) — `unlockPlates` is pure slice building, `unlockPlateListFlow` is
`for !ctx.Done` (`gui/unlock_platelist.go:103`), `unlockEngraveFlow`'s `for {`
(`:228`) exits via `cs.Choose` returning `!ok`, and `showNotice` returns immediately.
`uiFlow` then exits at `gui/gui.go:1612`.

---

## Not examined (scope)

Plan/spec correctness, the mutation table, the build gate, §10.2.4's design choices
(which screens are armed, the 3:00/0:30 values, arming only for the secret session and not
the public plate list), and the residency/funds lens — all declared settled in the brief.
F4 touches residency only because the *claim* it corrects was introduced by this diff.
