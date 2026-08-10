# F-106 ROOT CAUSE — measured on the SeedHammer II, 2026-08-10

**Build:** `b2b-idleprobe3` = `256b38c`, flashed as `v0.0.0-g256b38c`.
**Result: the 2× idle window is a LATE ARM EDGE, and it is deterministic.**

## The readings

| moment | annotation | reading |
| --- | --- | --- |
| Cut/Skip, untouched | `idle 0s w151 t770 e162 A!` `Pu0,0` | armed TRUE live, tracked value disagrees — **edge pending** |
| 3:00 (from video) | `idle 0s w170 t771 e162 A` `Pu0,0` | **site 170 wrote the clock**; `!` gone — edge processed |
| 6:00 | wipe-warning screen animates | fresh 3:00 window elapsed |
| 6:30 | wipe fires | +30 s, the genuine warning→wipe gap |
| Home, post-wipe | `idle 0s w48 t803 e162 -` `Pu0,0` | session-start clock, disarmed |

## What the numbers prove

**`t` advanced by exactly ONE across three minutes** (770 → 771). `AppendEvents`
blocks, so the loop was parked the entire time and woke once — at the 3:00 idle
deadline. That single wakeup was consumed processing the pending arm edge.

**`e` never moved: 162 at t770, at t771, and still 162 at t803.** So **zero events
arrived** across the whole window. `Pu0,0` is a *stale* record of the last real
touch, not a live event.

This **refutes the phantom-input hypothesis** that the probe's own decision table
lists first ("n sticks near 0, site 151, e climbing → PHANTOM INPUT"). The
signature requires `e` to climb. It did not. The controller-noise reading of
`processTouch`'s dedup (`cmd/controller/platform_sh2.go:398-402`, which compares
`tp` even when `touching` is false) is a real latent fragility but is **NOT**
what causes F-106 — nothing was generating events at all.

## The mechanism

`gui/run_flow.go`, inner loop:

```go
for {
	if ctx.Done || !yield() { return }
	wakeup := ctx.Wakeup
	evts = pl.AppendEvents(wakeup, evts[:0])   // BLOCKS until wakeup or event
	now := time.Now()
	armed := ctx.wipe.armed()                  // edge sampled only AFTER the block
	if len(evts) > 0 || (ctx.keepAwake && !armed) {
		a.idle.start = now
	}
	if armed != a.armed {
		a.armed = armed
		if armed {
			a.idle.start = now                 // row 2: fresh window at cut end
		}
	}
	...
}
```

1. The wipe guard is installed **during the flow's own execution** (`ctx.wipe = g`
   in the unlock arms), so `armed()` flips true while the loop is between
   `AppendEvents` calls. The frame drawn at that moment reads `A!` — armed true,
   `a.armed` still false.
2. `armed` is sampled **only after** `AppendEvents` returns. Nothing wakes the
   loop on an arming change, so the edge stays pending.
3. The loop's next wakeup **is the 3:00 idle deadline**.
4. On that tick the edge is finally processed, and row 2 stamps
   `a.idle.start = now` — restarting the window at the exact instant the wipe
   should have fired.
5. A full fresh window runs: warning at 6:00, wipe at 6:30.

**Deterministic 2×, exactly as predicted, and for the predicted reason.**

## Why the edge is spurious in the first place

`wipeGuard.armed()` returns true whenever `g != nil` and no job is running
(`gui/wipe_guard.go:49-59`). At Cut/Skip **there is no job yet** — `g.job` is set
only around the `Engrave` call — so the guard reads armed the moment it is
installed. That first transition is **guard installation, not a finished cut**,
yet row 2's "a finished cut starts a FRESH window" is applied to it.

Had the edge been processed on arrival it would have been a harmless reset at
t≈0. **The damage is entirely that it lands 3:00 late.**

## Fix direction (not yet implemented, needs its own R0 pass)

The edge must be processed **without waiting on the blocking read**, and the
initial installation should not be treated as a cut-end. Candidates:

1. **Sample and process the arm edge before `AppendEvents` blocks**, in addition
   to after it (arming can also change during the block, when a job finishes on
   the engrave goroutine — so both are needed, not either).
2. **Seed `a.armed` when the guard is installed**, so installation is not an
   edge at all. Narrower, and it leaves row 2 meaning only what it says.
3. Recompute `ctx.Wakeup` on an arming change so the loop does not sleep through
   a pending edge.

(1) and (2) are complementary and probably both correct. **Any of these changes
§10.2.4's timing on a secrets-residency control**, so it is risk-set work: it
needs a test that can fail — drive a session to an armed state, advance a fake
clock by exactly `idleTimeout`, and assert the wipe fires at 1× and not 2× —
plus a mutation row that reverts the edge processing.

## Caution for whoever writes the test

`gui/idle_realclock_diag_test.go` on `b2b` already reproduces
`platform_sh2.go`'s timer structure line for line and the warning lands at
**3m0s with ticks=2, evtTicks=0** — i.e. **the host harness does NOT reproduce
this bug**, because its loop is woken often enough that the edge is never
pending across the deadline. A host test that passes proves nothing here unless
it reproduces the *parked loop*: one `AppendEvents` call spanning the whole
window.
