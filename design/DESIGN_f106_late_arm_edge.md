# F-106 — the idle window runs 2×, because the arm edge is processed one wakeup late

**Status:** design, pre-R0. Written 2026-08-10.
**Owns:** F-106 (owning phase B2b, *CRITICAL, gates the phase*).
**Supersedes** the original filing's title, "the timer never starts unless the
operator touches the screen", which is wrong: it starts, and then gets restarted.

## The measurement, on hardware

Build `b2b-idleprobe3` = `256b38c`. Full readings in
`design/HARDWARE_RESULT_2026-08-10b_f106_ROOT_CAUSE.md`.

| moment | annotation | reading |
| --- | --- | --- |
| Cut/Skip, untouched | `idle 0s w151 t770 e162 **A!**` | armed TRUE live, tracked value disagrees — **edge pending** |
| 3:00 (from video) | `idle 0s w170 t771 e162 **A**` | site **170** wrote the clock; `!` cleared |
| 6:00 | warning animates | a fresh window elapsed |
| 6:30 | wipe fires | +30 s, the genuine warning→wipe gap |

Two numbers carry the diagnosis:

- **`t` advanced by exactly ONE across three minutes** (770 → 771). `AppendEvents`
  blocks, so the loop was parked the whole time and woke **once** — on the 3:00
  idle deadline — and that wakeup was consumed processing the pending edge.
- **`e` never moved** (162 at `t770`, `t771` and `t803`). **Zero events arrived.**
  So `Pu0,0` is a *stale* record of the last real touch, and the phantom-input
  hypothesis — the probe's own first decision row, which requires `e` to climb —
  is **refuted**. That hypothesis was mine and the operator's reading killed it.

## The mechanism

`gui/run_flow.go`, the inner loop, with the ordering that matters:

```go
for {
	if ctx.Done || !yield() { return }        // (1) the FLOW runs here
	wakeup := ctx.Wakeup                      // (2) read the deadline set at (6)
	evts = pl.AppendEvents(wakeup, evts[:0])  // (3) BLOCKS until wakeup or event
	now := time.Now()
	armed := ctx.wipe.armed()                 // (4) edge sampled ONLY here
	if len(evts) > 0 || (ctx.keepAwake && !armed) {
		a.idle.start = now
	}
	if armed != a.armed {
		a.armed = armed
		if armed {
			a.idle.start = now                // (5) row 2: fresh window at cut end
		}
	}
	// ...
	idleWakeup := a.idle.start.Add(idleTimeout)
	// ...
	ctx.WakeupAt(idleWakeup)                  // (6) next deadline, from a.idle.start
	break
}
```

1. The wipe guard is installed **inside `yield()` at (1)** — `ctx.wipe = g` in the
   unlock arms — so `armed()` flips true while the loop is between samples. The
   frame drawn at that moment reads `A!`: armed true, `a.armed` still false.
2. **(6) already scheduled the next wakeup from the OLD `a.idle.start`.** Nothing
   recomputes it when arming changes, and nothing wakes the loop on an arming
   change.
3. So (3) sleeps until the **3:00 idle deadline**.
4. Only then does (4) sample `armed`, and (5) stamps `a.idle.start = now` — **at
   the exact instant the wipe should have fired**.
5. (6) schedules 3:00 further out. Warning 6:00, wipe 6:30.

**Deterministic 2×**, and it matches the hardware to the second.

### The edge is also spurious

`wipeGuard.armed()` (`gui/wipe_guard.go:49-59`) is true whenever `g != nil` and no
job is running. At session open `g.job` is nil, so the guard reads **armed the
moment it is installed**. That first transition is **guard installation, not a
finished cut** — yet row 2's "a finished cut starts a FRESH window" is applied to
it.

Processed on arrival it would have been a harmless reset at t≈0. **The damage is
entirely that it lands 3:00 late.** That is why the fix targets the *lateness*
rather than the transition.

## The fix

**Process the arm edge before the loop blocks, as well as after it.**

```go
// syncArmed processes a change in ctx.wipe.armed(). Idempotent, so it is safe
// to call twice per iteration -- and it MUST be called twice, for two different
// reasons.
syncArmed := func(now time.Time) bool {
	armed := ctx.wipe.armed()
	if armed != a.armed {
		a.armed = armed
		if armed {
			// §10.2.4 row 2: a finished cut starts a FRESH window.
			a.idle.start = now
		}
	}
	return armed
}
```

called in two places:

```go
for {
	if ctx.Done || !yield() { return }
	// BEFORE the block. The guard is installed inside yield() above, so an
	// edge is already pending here -- and ctx.Wakeup below was computed from
	// the PRE-edge clock. Leaving it until after AppendEvents means the edge is
	// processed at whatever the next wakeup happens to be, and when that wakeup
	// IS the idle deadline, row 2 restarts the window at the instant the wipe
	// should have fired. Measured on hardware: a 6:00 warning and a 6:30 wipe
	// against a 3:00 spec (F-106).
	syncArmed(time.Now())
	wakeup := ctx.Wakeup
	evts = pl.AppendEvents(wakeup, evts[:0])
	now := time.Now()
	// AFTER the block too, and NOT instead: arming can change DURING the sleep,
	// when the engrave goroutine finishes a cut on the other side of e.errs.
	// That is row 2's actual subject, and only this call can see it.
	armed := syncArmed(now)
	if len(evts) > 0 || (ctx.keepAwake && !armed) {
		a.idle.start = now
	}
	// ... unchanged ...
}
```

**Both calls are load-bearing and neither subsumes the other:**

- the **pre-block** call catches an edge created by the *flow* (guard
  installation, and any bracket change) — the F-106 case;
- the **post-block** call catches an edge created by the *engrave goroutine*
  finishing a cut while the loop slept — row 2's actual subject.

**One consequence to state plainly:** the pre-block call can reset
`a.idle.start` *after* `ctx.Wakeup` was computed, so the loop may sleep to a now-
stale (earlier) deadline, wake, find itself not idle, and reschedule. That is
**one extra wakeup, not a missed window** — the recomputation at (6) uses the
corrected clock. It costs a wakeup on the frame where the guard is installed and
nothing else.

### Alternatives considered and rejected

1. **Seed `a.armed` at guard installation.** `run_flow` cannot see installation —
   `ctx.wipe = g` happens in the unlock arms, and the accumulator `a` is a
   closure local of `runWithFlow`. It would need a callback or an exported seam
   into the frame loop, which is more surface for less coverage: it fixes the
   installation edge and not a late edge from any other source.
2. **Recompute `ctx.Wakeup` on an arming change.** Strictly weaker than the
   above and more code: it still samples `armed` only after the block, so it
   cannot know the edge happened until it has already slept through it.
3. **Make row 2 not fire on installation** (distinguish "a job existed and
   stopped" from "no job yet"). This treats the symptom. Installation *should*
   start the window — the secret becomes resident then — so a reset at t≈0 is
   correct behaviour, and the defect is purely that it lands at t=3:00.

## Tests that can fail

**The trap, stated first because it invalidates the obvious test.**
`gui/idle_realclock_diag_test.go` on `b2b` already reproduces
`platform_sh2.go`'s timer structure line for line, and the warning lands at
**3m0s with `ticks=2`, `evtTicks=0`** — i.e. **the host harness does not
reproduce this bug**, because its loop is woken often enough that the edge is
never pending across the deadline. A passing host test proves nothing here
unless it reproduces the **parked loop**: a single `AppendEvents` call spanning
the whole window.

1. **The regression test — a PARKED loop with an edge installed during the
   park.** Drive `runWithFlow` under `synctest` with a platform whose
   `AppendEvents` blocks until the requested wakeup and returns no events, so
   `t` advances once per window exactly as on hardware. Install the wipe guard
   from inside the flow (as `unlockSecretSession` does), then assert **the
   warning is drawn at `idleTimeout`, not at `2 × idleTimeout`**.
   **Must fail before the fix**, and on the unfixed tree it fails by producing
   the warning at exactly double.
   *Mutation row:* delete the pre-block `syncArmed` call.
2. **The post-block call still earns its place.** Arm the guard by finishing a
   job *during* the block (set the job terminal from the fake engraver while the
   loop is parked), and assert row 2 starts a fresh window from the cut's end.
   *Mutation row:* delete the post-block `syncArmed` call — test 1 must stay
   green and this one must fail, which is what proves the two calls are not
   redundant.
3. **Idempotence.** Calling `syncArmed` twice with no change between must not
   move `a.idle.start`. Without this, the two-call structure could silently
   become a per-iteration clock reset, which would mean the window NEVER fires —
   a strictly worse failure than 2×.
4. **The warning→wipe gap is unchanged at 30 s.** It was exact on hardware
   across three cycles and must not regress.

## Gate coverage

Applied to a scratch worktree off `b2b` (`seedhammer-f106`), built and measured.
Numbers are outputs, not projections.

```
go build ./gui/...    clean
go test ./gui/        ok
gofmt                 clean (5 pre-existing files unchanged)
```

**Mutation rows, executed:**

| mutant | test 1 (late edge) | test 2 (mid-park edge) |
| --- | --- | --- |
| delete the **pre-block** `syncArmed` | **KILLED** — *"the warning appeared 5m59.99s after the guard was installed, want ~3m0s"* | KILLED |
| delete the **post-block** `syncArmed` | **SURVIVED** | **SURVIVED** |

**Test 1 reproduces F-106 on the host, exactly.** 5m59.99s against a 3m0s spec is
the operator's measured 6:00 warning, to the tick. That is the first host
reproduction of this defect — the pre-existing `idle_realclock_diag_test.go`
lands at a correct 3m0s and cannot see it.

**MUTANT B SURVIVES, AND THAT IS AN OPEN PROBLEM WITH THIS DESIGN, NOT A
FOOTNOTE.** The design above asserts both calls are load-bearing. Measurement
says the post-block call is either **redundant** or **untested**, and I cannot
currently tell which:

- The argument for keeping it: with only the pre-block call, a cut-end edge that
  lands mid-park is read with a STALE `armed` on wake, so the loop takes the
  screensaver branch instead of the warning branch, and the clock is reset one
  frame later by the pre-block call — the same 2× shape, for cut ends.
- Why test 2 does not discriminate: the flow draws a frame per iteration, so the
  pre-block call runs every iteration and catches the edge within one tick. With
  `tickFloor` at 2 s and a 5 s tolerance, the difference is inside the noise. A
  discriminating test needs a park with **no frame drawn** between the edge and
  the deadline, which this harness cannot currently express.

**Do not read the two-call structure as verified.** One of these is true and R0
should say which: the post-block call is load-bearing and needs a harness that
can express a frameless park, or it is dead and should be deleted.

**A test that never ran, found en route.** The regression test was first written
as `idle_late_arm_test.go`. Go reads the trailing `_arm` as an implicit GOARCH
constraint, so the file landed in `IgnoredGoFiles` and **never compiled on this
host** — `go vet` clean, suite green, zero tests run. Renamed to
`idle_late_arm_edge_test.go`. Worth recording as its own hazard: this project
already treats "a test that cannot fail" as the dominant defect class, and this
is the strictly worse version — a test that does not exist. `go list -f
'{{.IgnoredGoFiles}}'` is the check.

**No host test can prove the fix on the device**, because the defect is a
function of when `platform_sh2.go`'s event source actually returns. Hardware
re-verification — Cut/Skip untouched, warning at 3:00 — is required, and is
already scheduled alongside the abort→resume seam for F-107/F-108.

## What R0 should attack

1. **THE PRIMARY QUESTION: is the post-block `syncArmed` call load-bearing or
   dead?** Deleting it kills no test (§Gate coverage). Either produce the case
   that needs it — a park with no frame drawn between a cut-end edge and the
   deadline — or conclude it is redundant and delete it. Shipping an unpinned
   line while the design calls it load-bearing is the worse outcome. Related:
   can arming change between the post-block call and the bottom of the loop,
   where `idleWakeup` is computed?
2. **Does the pre-block call introduce a way for the window to never fire?** If
   some caller flips `ctx.wipe` on every frame, the pre-block call would reset
   the clock every iteration. `armed != a.armed` should make that impossible;
   verify it, because the failure mode is worse than the bug.
3. **Is the extra wakeup acceptable?** It costs one wake on the installation
   frame. Confirm it cannot compound — e.g. a flow that reinstalls its guard
   every frame.
4. **Is `a.armed = false` at session start (`run_flow.go:53`) still correct**
   given the pre-block call now runs before the first block of a new session?
5. **Row 4's passphrase bracket.** `unlockPassphraseFlow` installs and restores
   its own guard, so a session produces disarm→arm transitions that are not cut
   ends. Does the fix give the right window across that handover, or does it
   restart the clock when the passphrase bracket closes?
