# F-106 — §10.2.4's timer never starts unless the screen is touched

**Status:** analysis, pre-fix. Written 2026-08-10.
**Owns:** the hardware Critical recorded in `HARDWARE_RESULT_2026-08-09_phaseB2b.md`
and carried at the top of `CONTINUITY_2026-08-09b.md`.

## The observation

| on the machine, armed and parked on Cut/Skip | result |
| --- | --- |
| unlock, then **do not touch the screen** | 4:15 elapsed — no warning, no wipe, no screensaver |
| then touch a blank area | warning at **exactly 3:00**, wipe at **exactly 3:30** |

Pre-existing, not a B2b regression — see "Not a regression" below.

## What the second row proves

The post-touch run exercises the entire mechanism end to end: `time.Now()`, the
deadline handed to `AppendEvents`, the platform's timer, the idle comparison,
the warning draw, and the unwind. All of it works, and works to the second.

So the defect cannot be in any of those. The only state that differs between
the two rows is **`a.idle.start`**, and `run_flow.go` reduces the whole question
to two lines:

```go
idleWakeup := a.idle.start.Add(idleTimeout)
idle := now.Sub(idleWakeup) >= 0
```

For nothing to fire across 4:15, `now - (a.idle.start + 3:00)` must have stayed
negative the whole time. That admits **exactly two** families, and no third:

- **(A) `a.idle.start` was continuously refreshed** — dragged forward to `now`
  on every tick.
- **(B) `a.idle.start` was poisoned into the FUTURE** — one bad value, and the
  window opens at some far-off time instead of never.

Both are consistent with a touch producing a correct 3:00, because a touch
assigns `a.idle.start = now` unconditionally and repairs either one.

### Family A — the refresh sites

`a.idle.start` is assigned in exactly **three** places, and A has to come from
one of them:

```
gui/run_flow.go:48   a.idle.start = time.Now()   // session start
gui/run_flow.go:151  a.idle.start = now          // the refresh term
gui/run_flow.go:170  a.idle.start = now          // row 2's armed edge
```

**A1 — the refresh term.** Two conditions feed it:

```go
if len(evts) > 0 || (ctx.keepAwake && !armed) {
    a.idle.start = now
}
```

`ctx.KeepAwake()` has exactly **one** caller in the tree — `unlock_kdf.go:327`,
the derivation — and it is gated on `!armed` besides, which is false on
Cut/Skip. Measured, not read off a doc comment:

```
$ grep -rn "KeepAwake()" --include="*.go" . | grep -v _test.go
gui/unlock_kdf.go:327:		ctx.KeepAwake()
```

So on Cut/Skip the surviving term is `len(evts) > 0` alone. A1 therefore says:
**the panel is delivering events with nobody touching it.** That is not idle
speculation on this machine — the protective film generated exactly this class
of phantom input, and removing it navigated the UI by itself.

**A2 — the armed edge.** `run_flow.go:170` refreshes the clock on every
false→true transition of `ctx.wipe.armed()`. One transition is the design (row
2: a finished cut starts a fresh window); an **oscillating** `armed` would
refresh forever, and would look identical from outside.

```go
func (g *wipeGuard) armed() bool {
	if g == nil { return false }
	if j := g.job; j != nil {
		switch j.Status().State {
		case engraveRunning, engraveStopping: return false
		}
	}
	return true
}
```

On Cut/Skip `g.job` should be nil — nothing is cutting — so this should be
constant `true`. "Should be" is the reason A2 is listed rather than dismissed:
it is the one refresh site whose input is a *live status read* from another
goroutine, and it is the only one that could flip without any hardware
misbehaving at all. A probe that reports only the clock's age cannot tell A1
from A2, so it must also report **which site last wrote it**.

### Family B — a clock read from the future

`a.idle.start` is only ever assigned `time.Now()` (or `now`, read from it). A
single glitched read anywhere in the session — the arming edge, the session
start, any of the KDF's ~200 ticks — parks the window that far out and stays
parked until the next input.

## The one measurement that separates them

`time.Since(a.idle.start)`, on screen — **plus the site that last wrote it**,
which is what separates A1 from A2.

| reading | family | meaning |
| --- | --- | --- |
| near 0, last write = `:151` | **A1** | events are arriving; find their source |
| near 0, last write = `:170` | **A2** | `armed()` is oscillating |
| **negative** | **B** | a clock read landed in the future |
| counts past 3:00 and nothing fires | neither | the evaluation is wrong after all |

The third row is listed because a diagnostic that cannot report "my own
reasoning was wrong" is not a diagnostic.

## The zero-firmware bench check to run first

**Leave the device on the main screen, untouched, for 3:30. Does the
screensaver appear?**

No flash, no payload, no unlock — and it discriminates:

- **Screensaver appears** → the idle clock advances normally when nothing else
  is going on, so the refresh (or the poisoning) is specific to the unlock
  path. That narrows the hunt to a bounded stretch of code.
- **Screensaver never appears** → the clock never advances *anywhere*, on
  upstream's own code (see below). That is family A on a global scale, and the
  next step is the panel, not `run_flow.go`.

Worth the three minutes before building anything: it is the cheapest experiment
available and it halves the search either way.

## The bench card — three experiments, two of them needing no flash

Run in order. Each one is independently informative, and the first two cost
nothing but a stopwatch. The machine currently runs the **heap probe** build, so
experiments 1 and 2 are valid on it as they stand — the idle clock is untouched
by that build.

**1. Does the screensaver EVER appear?**
Main screen, no payload flow, do not touch for 3:30.

- *Appears* → the clock advances normally when nothing else is happening. The
  refresh (or the poisoning) is specific to the unlock path, which bounds the
  hunt to a short stretch of code.
- *Never appears* → the clock never advances anywhere, on upstream's own
  condition. That is A1 at global scale and the next step is the panel.

**2. Does the PASSPHRASE screen's own timer work?**
Enter Sealed Payload, type two or three words, then stop touching for 3:30.
Row 4's guard is armed for the whole of `unlockPassphraseFlow`, and the last
keystroke is a real event, so the window should open 3:00 after it.

- *Warning appears* (it will say **"partly typed passphrase"**, not "seed
  material") → the clock runs correctly when a genuine touch set it. Phantom
  input is then not continuous, which weakens A1 and promotes **B**.
- *No warning* → the clock is held forever on that screen too, and A1 is the
  answer. This is also, in itself, an F-105 defect worth its own entry.

**3. Flash `b2b-idleprobe` and read the overlay.**
Only if 1 and 2 have not already settled it. Unlock, do not touch, and read the
top-left line: `idle <n>s w<site> t<ticks> e<evtTicks> <A|-> <lastEvent>`.

A word on experiment 2 that matters for reading the result: F-106 barely touches
row 4, because entering a passphrase *is* touching the screen. The operator
cannot reach that keyboard without generating events, so its window is always set
by a real touch. Row 1 — the post-unlock walk-away — is the one with no
guaranteed touch anywhere near it, which is exactly why the defect surfaced there
and nowhere else.

## Not a regression, and the base commit proves it

The refresh condition at the base commit `a01b666` — upstream's own `Run`, before
B2b touched anything:

```go
evts = pl.AppendEvents(wakeup, evts[:0])
now := time.Now()
if len(evts) > 0 {
    a.idle.start = now
}
```

B2b's version adds `|| (ctx.keepAwake && !armed)`, which is strictly **more**
refresh and never less. Whatever holds the clock at `now` today held it at
`a01b666` too — and it governed the **screensaver** there, which is why "has the
screensaver ever appeared on this machine?" is a question about upstream, not
about this phase.

## Host reproduction: what is already covered, and what is not

**Correction to `CONTINUITY_2026-08-09b.md`**, which said F-106 was
host-reproducible and told the next session to write the failing test first.
That test already exists and it **passes**:

```
$ nix develop … go test ./gui/ -run TestRunSealedPayloadReentryAfterWipe -v
--- PASS: TestRunSealedPayloadReentryAfterWipe/F_idle-wipe_nfc (0.15s)
```

`F idle-wipe nfc` drives the **real** `uiFlow` through a real unlock of vector F,
parks on Cut/Skip, **delivers no further events**, and observes the warning at
3:00 and the wipe at 3:30. `TestRunWarningThenWipe` covers the same property
synthetically. Both are green, so the park-with-no-input case is not what is
broken.

That is a useful negative: the defect is **below** the harness, in one of the
two things every host test replaces.

1. **The clock.** Every test runs in a `synctest` bubble.
2. **The event loop.** `deadlinePlatform.AppendEvents` is a `time.Sleep`; the
   SH2's is a `select` over a reused `*time.Timer`, a wakeup channel and a pin
   interrupt (`cmd/controller/platform_sh2.go:369`).

`gui/idle_realclock_diag_test.go` removes both substitutions at once — real wall
time, and an `AppendEvents` whose structure is the SH2's line for line, timer
reuse included. It is opt-in (`SH2_REALCLOCK=1`) because it costs 3.5 minutes of
real time, and it reports the tick and event counts either way.

**Result — it does not reproduce, and the numbers are worth reading:**

```
$ SH2_REALCLOCK=1 nix develop … go test ./gui/ -run TestIdleTimerUnderSH2ShapedEventLoop -v -timeout 12m
    idle_realclock_diag_test.go:142: warning drawn at 3m0s (ticks=2 evtTicks=0)
    idle_realclock_diag_test.go:165: elapsed=3m30s sessions=2 ticks=32 evtTicks=0 longestDeadline=3m0s
--- PASS: TestIdleTimerUnderSH2ShapedEventLoop (210.09s)
```

- `evtTicks=0` — not one event, the whole run. The clock was never refreshed and
  the window opened anyway.
- `ticks=2` to reach the warning — the platform sat inside a **single** blocking
  `AppendEvents` for the full three minutes and was released by its own timer.
- `longestDeadline=3m0s` — the 3-minute deadline was both requested and served,
  so the reused-`time.Timer` pattern is not dropping it.
- `sessions=2` at `3m30s` — the wipe fired on schedule and restarted the session.

So: real clock, real timer-reuse, zero input, correct behaviour. **Everything
above the panel is exonerated**, which is why the next measurement has to happen
on the machine.

A note on the first cut of this test, because it is the reason to distrust a
green diagnostic: it reported `sessions=2` and a zero wipe timestamp in the same
breath. Session 2 returns **without drawing**, so it never yields and the range
body never observes it — the assertion was unobservable, not satisfied. Timing
the wipe from inside the flow fixed it. A diagnostic that cannot fail is worth
exactly as much as a test that cannot fail.

## Why this matters beyond a missing screensaver

§10.2.4 is a **funds-safety** guarantee: walk away from an unlocked machine and
the secrets are erased. A timer that only starts once the operator touches the
screen inverts the guarantee exactly where it is needed — the operator who walks
away *without* touching anything is the one the clause is written for.
