# F-106 — the idle window runs 2×, because the arm edge is processed one wakeup late

**Status:** R0 round 0 folded (was RED, 1C/2I). Written 2026-08-10.
Awaiting re-review, then hardware re-verification.
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

### The edge is row 1, and it is not spurious

*(Corrected by R0 round 0, M1. The first draft called this transition "spurious"
and attributed it to row 2. Both were wrong, and the wrong version invites this
design's own rejected alternative 3.)*

`wipeGuard.armed()` (`gui/wipe_guard.go:49-59`) is true whenever `g != nil` and no
job is running. At session open `g.job` is nil, so the guard reads **armed the
moment it is installed**. That first transition is **guard installation**, which
is **row 1**, not row 2: §10.2.4 as amended defines "resident" as a *lifetime*,
implemented as the secret session's lifetime, and `gui/wipe_guard.go:3-14` says
the guard **is** that seam. So the window opening at installation is row 1
working correctly.

Both rows reset the same clock, and the reset is right in both cases. Processed
on arrival it is a correct window origin at t≈0. **The damage is entirely that it
lands 3:00 late.** That is why the fix targets the *lateness* rather than the
transition.

## The fix

**Process the arm edge before the loop blocks, as well as after it.**

```go
// syncArmed processes a change in ctx.wipe.armed(). Idempotent by the
// `armed != a.armed` guard, so calling it twice per iteration cannot move the
// clock on its own.
syncArmed := func(now time.Time) bool {
	armed := ctx.wipe.armed()
	if armed != a.armed {
		a.armed = armed
		if armed {
			// Both §10.2.4 rows land here: row 1 when the guard is
			// INSTALLED (the residency begins, and its window opens with
			// it), row 2 when a job finishes (a finished cut starts a
			// FRESH window).
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
	// BEFORE the block -- this is the fix. The guard is installed inside
	// yield() above, so an edge is already pending here, and ctx.Wakeup below
	// was computed from the PRE-edge clock. Leaving it until after
	// AppendEvents means the edge is processed at whatever the next wakeup
	// happens to be, and when that wakeup IS the idle deadline, row 2 restarts
	// the window at the instant the wipe should have fired. Measured on
	// hardware: a 6:00 warning and a 6:30 wipe against a 3:00 spec (F-106).
	syncArmed(time.Now())
	wakeup := ctx.Wakeup
	evts = pl.AppendEvents(wakeup, evts[:0])
	now := time.Now()
	// AFTER the block, secondary: this advances an engrave-goroutine edge by
	// one loop turnaround, not by a wakeup, and no test pins it. It must stay
	// syncArmed(now) -- the bare ctx.wipe.armed() it replaced is one character
	// away and NOT equivalent (a fresh sample with no stamp warns on the
	// pre-edge clock).
	armed := syncArmed(now)
	if len(evts) > 0 || (ctx.keepAwake && !armed) {
		a.idle.start = now
	}
	// ... unchanged ...
}
```

**What each call buys** *(rewritten after R0 round 0, I1 — the first draft said
"both are load-bearing and neither subsumes the other", and measurement says
otherwise):*

- the **pre-block** call catches an edge created by the *flow* — guard
  installation, and any bracket handover — on the frame it happens. It is worth
  a **wakeup**: without it the edge waits for the next one, and when that wakeup
  *is* the idle deadline the window doubles. This is F-106, and this call is the
  fix.
- the **post-block** call catches an edge created by the *engrave goroutine* at
  the end of the block instead of at the start of the next iteration. It is worth
  **one loop turnaround, not one wakeup**: there is no blocking call between that
  sample point and the next iteration's pre-block one, so the pre-block call
  subsumes it. Deleting it leaves the whole `./gui/` suite green (measured), and
  costs at most one screensaver frame drawn where a warning frame belonged, on
  the single tick an async edge lands.

**Decision: keep the post-block call**, for that tick's branch consistency, and
say plainly in the code that it carries no §10.2.4 guarantee of its own and that
no test pins it. The alternative — deleting it — is defensible but must then use
`armed := a.armed`; see the trap below.

**The trap, one character wide.** The post-block call must stay
`syncArmed(now)`. Substituting the bare `ctx.wipe.armed()` it replaced is *not*
equivalent: a fresh sample with no stamp enters the warning branch on a tick
whose `a.idle.start` is still the pre-edge clock, so the warning draws at the
edge and the wipe follows `wipeWarningDelay` later instead of a fresh
`idleTimeout`. Measured: the substitution leaves the pre-existing suite green,
because in every scenario it reached the pre-block call had already stamped.
`TestCutEndingAfterTheDeadlineStartsAFreshWindow` was written to close that.

**Engrave-side edges are covered by NEITHER call**, and the design has to say so:
the pre-block call runs before the sleep, the post-block call after it returns,
so an edge created *inside* `AppendEvents` is invisible to both. What un-parks
the loop for those is **`pl.Wakeup()`** — `engraveJob.Start`'s goroutine calls it
on the way out (`gui/engraver.go:110`) and `platform_sh2.go:384` returns
early on it. §10.2.4 row 2's promptness for real cut ends rests entirely on that
one `defer`, which is why `TestCutEndingDuringTheParkStartsAFreshWindow` now pins
it and why `deadlinePlatform` models the wakeup channel (R0 round 0, I2).

**One consequence to state plainly:** the pre-block call can reset
`a.idle.start` *after* `ctx.Wakeup` was computed, so the loop may sleep to a now-
stale (earlier) deadline, wake, find itself not idle, and reschedule. That is
**one extra wakeup, not a missed window** — the recomputation at (6) uses the
corrected clock. It costs a wakeup on the frame where the guard is installed and
nothing else.

**The structure cannot extend a window.** It does not change the *set* of clock
stamps, only their timing, and only ever **earlier** — so it can make the wipe
fire sooner, never later. That, rather than the two-call symmetry, is the
argument that the fix is safe.

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
2. **A cut that ends while the loop is parked.**
   `TestCutEndingDuringTheParkStartsAFreshWindow`. A **real** job — `p.engraver`
   set, a spline that sleeps 1:30 inside the bubble and returns of its own
   accord — so the cut end arrives from the engrave goroutine while the loop is
   blocked in `AppendEvents` and the screensaver has not yet activated. Assert
   the warning lands `idleTimeout` after the cut's end.
   *Mutation row:* delete `defer e.pl.Wakeup()` from `engraveJob.Start`.

   *R0 round 0 rewrote this test.* It was written to pin the post-block
   `syncArmed` call and did not; worse, it never set `p.engraver`, so
   `runEngraving` failed at its first statement with `"engraver unavailable"`,
   `close(cutting)` was a no-op on a dead job, and the timing it measured was a
   startup failure observed at the next wakeup. It passed by coincidence (C1).
   Two structural changes made it honest: the cut now ends on the **engrave
   goroutine's** schedule (while the loop is parked the flow is suspended inside
   `yield()` and cannot run at all, so a flow-driven `close(cutting)` can only
   fire once something has *already* un-parked the loop — the very thing under
   test), and the job's terminal state is asserted **before** the timing
   assertion, so a job that never ran is INCONCLUSIVE rather than PASS.

3. **A cut that ends AFTER the loop has already gone idle.**
   `TestCutEndingAfterTheDeadlineStartsAFreshWindow`. The spline sleeps
   `idleTimeout + 10s`, so the arm edge lands on a tick where `a.idle.active` is
   already true. Assert row 2 still starts a **fresh** window rather than
   warning on the pre-edge deadline.
   *Mutation row:* post-block `syncArmed(now)` → `ctx.wipe.armed()`.

4. **Idempotence — already covered, no new test.** Calling `syncArmed` twice
   with no change between must not move `a.idle.start`; without it the two-call
   structure becomes a per-iteration clock reset and the window NEVER fires, a
   strictly worse failure than 2×. Measured: dropping the `armed != a.armed`
   guard fails **12** tests across `./gui/` (list in §Gate coverage). Writing a
   thirteenth would add nothing.

5. **The warning→wipe gap is unchanged at 30 s — already covered, no new test.**
   Pinned by `gui/idle_realclock_diag_test.go:182` and `gui/run_flow_test.go:346`.
   It was exact on hardware across three cycles and must not regress.

*(Items 4 and 5 were listed as tests to write in the first draft and were not
written — R0 round 0, M2. They are pre-existing coverage, and saying so is the
honest version.)*

## Gate coverage

Applied to a scratch worktree off `b2b` (`seedhammer-f106`), built and measured.
Numbers are outputs, not projections.

```
go test ./...         ok        (all packages; cmd/kdfbench and cmd/sealread
                                 fail [setup failed] on b2b too -- TinyGo-only,
                                 pre-existing, untouched)
go test ./gui/        ok  38.3s
gofmt -l gui/         5 pre-existing files, identical to b2b; the four files
                      this change touches are clean
go vet ./gui/         one pre-existing go1.25/go1.26 diagnostic, identical to b2b
go list IgnoredGoFiles ./gui/ -> [debug.go]   (no test file hidden by an
                                 implicit GOOS/GOARCH constraint)
```

**Mutation rows, executed** — T1 = `TestIdleWindowIsNotDoubledByALateArmEdge`,
T2 = `TestCutEndingDuringTheParkStartsAFreshWindow`,
T3 = `TestCutEndingAfterTheDeadlineStartsAFreshWindow`:

| mutant | T1 | T2 | T3 | whole `./gui/` |
| --- | --- | --- | --- | --- |
| delete the **pre-block** `syncArmed` | **KILLED** — *"the warning appeared 5m59.99s after the guard was installed, want ~3m0s"* | **KILLED** | ok | FAIL |
| delete `defer e.pl.Wakeup()` (`engraver.go:110`) | ok | **KILLED** | ok | FAIL |
| post-block `syncArmed(now)` → `ctx.wipe.armed()` | ok | ok | **KILLED** | FAIL |
| delete the **post-block** `syncArmed` (`armed := a.armed`) | ok | ok | ok | **SURVIVED** |
| drop the `armed != a.armed` idempotence guard | KILLED | KILLED | KILLED | FAIL — **12** tests |

The 12: `TestIdleWindowIsNotDoubledByALateArmEdge`,
`TestCutEndingDuringTheParkStartsAFreshWindow`,
`TestCutEndingAfterTheDeadlineStartsAFreshWindow`, `TestRunWarningThenWipe`,
`TestRunWarningCountdownIsReal`, `TestRunTapDuringWarningResetsAndReturnsContent`,
`TestRunPostCutWindowRestartsFromCutEnd`, `TestRunWarningBufferDoesNotGrow`,
`TestRunKeepAwakeCannotPostponeAnArmedWipe`, `TestRunSealedPayloadReentryAfterWipe`,
`TestUnlockPassphraseWarningShowsTheRow4Subject`,
`TestWipeZeroesEveryPinnedBufferAtRunLevel`.

**Test 1 reproduces F-106 on the host, exactly.** 5m59.99s against a 3m0s spec is
the operator's measured 6:00 warning, to the tick. That is the first host
reproduction of this defect — the pre-existing `idle_realclock_diag_test.go`
lands at a correct 3m0s and cannot see it.

**The post-block `syncArmed` is unpinned, deliberately, and that is now a
recorded fact rather than an open question.** R0 round 0 answered it: the call is
not load-bearing (§The fix). It is kept for branch consistency on the tick an
async edge lands, the code says so, and no test claims otherwise.

**A no-op mutant reads exactly like a surviving one.** The first run of this
table reported the idempotence guard as SURVIVED. The `sed` had the wrong
indentation depth and never edited the file — five tabs where the source has
four. Re-run with the edit verified by `git diff` before testing, it fails 12
tests. Any mutation row in this project must show the applied diff, not just the
result; a silently unapplied mutant is a false PASS wearing the costume of a
finding.

**A test that never ran, found en route.** The regression test was first written
as `idle_late_arm_test.go`. Go reads the trailing `_arm` as an implicit GOARCH
constraint, so the file landed in `IgnoredGoFiles` and **never compiled on this
host** — `go vet` clean, suite green, zero tests run. Renamed to
`idle_late_arm_edge_test.go`. Worth recording as its own hazard: this project
already treats "a test that cannot fail" as the dominant defect class, and this
is the strictly worse version — a test that does not exist. `go list -f
'{{.IgnoredGoFiles}}'` is the check.

**The harness diverged from the device on exactly the path row 2 depends on, and
that is fixed.** `deadlinePlatform.AppendEvents` did not model
`Platform.AppendEvents`' wakeup channel, so a cut ending mid-park was never
un-parked by `pl.Wakeup()` and the harness reproduced **F-106's own doubled
window — a 6m0s warning against a 3m0s spec, with the fix applied**. It now
selects on `testPlatform.wakeups` alongside the deadline timer, matching
`cmd/controller/platform_sh2.go:384`. Verified: that change alone leaves
every pre-existing test green and breaks exactly one — the false-PASS test C1
identified.

**No host test can prove the fix on the device**, because the defect is a
function of when `platform_sh2.go`'s event source actually returns. Hardware
re-verification is required, and is scheduled alongside the abort→resume seam for
F-107/F-108:

1. **Cut/Skip untouched → warning at 3:00, wipe at 3:30** (the F-106 reading
   itself, previously 6:00/6:30).
2. **Back mid-cut → warning 3:00 after the head stops** (R0 round 0, I2). This
   is the `engraveStopping` park: `EngraveScreen` installs its 500 ms poll only
   while `Status().State == engraveRunning`, and `Stop()` moves the job to
   `engraveStopping`, for which `armed()` is still false and the poll is *not*
   set — so the loop parks to the idle deadline and **only `pl.Wakeup()`** ends
   the park when the goroutine finishes. It is also §10.2.2's "most ordinary
   recovery" (`gui/unlock_session.go:199-203`), not an exotic path.

## R0 round 0 — outcome

**RED, 1 Critical / 2 Important**, persisted verbatim at
`design/agent-reports/2026-08-10-r0-f106-round0.md` (commit `e29d61b`). Folded
above. Summary of what changed, so a re-review can scope itself:

| finding | disposition |
| --- | --- |
| **C1** `TestArmEdgeDuringTheParkStillStartsAFreshWindow` is a false PASS — no `p.engraver`, so the job died at `Engraver()` and the test measured a startup failure | test rewritten and renamed `TestCutEndingDuringTheParkStartsAFreshWindow`; real job, cut ends on the engrave goroutine, terminal state asserted **before** the timing assertion; mutation table re-measured and replaced |
| **I1** code and design both assert a protection the post-block call does not provide | §The fix rewritten to what is measured; both comment blocks in `run_flow.go` replaced; keep-vs-delete decided (keep) and recorded with its reason |
| **I2** coverage of engrave-side edges rests entirely on `pl.Wakeup()`, unstated and unmodelled | dependency named in the design and beside the call; `deadlinePlatform` now models the wakeup channel; `defer e.pl.Wakeup()` is now a **killed** mutation row; Back-mid-cut added to the hardware run |
| **M1** the installation edge is row 1, not row 2, and is not "spurious" | §"The edge is also spurious" retitled and corrected, in the design and in the code comment |
| **M2** the design promised four tests and shipped two | items 4 and 5 restated as pre-existing coverage, with the 12-test measurement that backs item 4 |
| **M3** `unarmedAt` is misnamed — it records the moment the guard **arms** | renamed `cutEndAt` |
| **N1** `armed()` called twice per tick drains `e.progress` twice | no action, as recommended |

**Answered, no action needed** (recorded so a re-review does not re-derive them):
the pre-block call cannot make the window never fire; `a.armed = false` at
session start is still correct; the extra wakeup is exactly one and cannot
compound; row 4's passphrase-bracket handover produces the correct window origin
and does not restart the clock when the bracket closes; arming cannot change
between the post-block call and the `idleWakeup` computation.

**Beyond the findings:** `TestCutEndingAfterTheDeadlineStartsAFreshWindow` was
added to pin the one-character `ctx.wipe.armed()` trap the reviewer named in
prose. A trap that only exists in a comment is one refactor from being sprung.
