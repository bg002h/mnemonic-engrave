# F-103 mechanism trace — the screen-film / idle-timer coupling

**Read-only investigation, sonnet.** Tree: `seedhammer-b2b` @ `75233b8` (branch
`b2b`). Nothing committed there; a scratch host test was written, run, and
deleted (`git status --short` clean at the end — verified).

## The claim (F-103, `design/FOLLOWUPS.md:1468-1521`)

The protective screen film resting on the panel generates a continuous stream
of touch events, which silently and permanently disables §10.2.4's wipe, the
screensaver, and every idle-driven behaviour, because the idle clock is keyed
on `len(evts) > 0` — any event, not effective input.

## The trace

`gui/run_flow.go`'s session loop, the single refresh line (line 251):

```go
if len(evts) > 0 || (ctx.keepAwake && !armed) {
    a.idle.start = now
}
```

`evts` comes from `evts = pl.AppendEvents(wakeup, evts[:0])` (line 221), called
once per loop iteration. On the real device, `platform_sh2.go:369`'s
`AppendEvents` has a "don't starve touch input" fast path (lines 371-378) that,
whenever the touch IRQ has fired, calls `processTouch` and — if it returns
`ok` — returns immediately with exactly one event appended, before ever
touching the deadline/timer/wakeup select. `processTouch` (lines 398-417)
dedupes only on **exact equality**: `touching == inp.last && tp == inp.lastPos`.
A capacitive panel (`driver/ft6x36`) with something resting on it that
produces jittery, non-identical successive readings (position drift, or the
touch/no-touch boundary flickering) never hits that equality check, so every
poll yields a "new" event.

**Point 3 confirmed directly from the code.** The warning branch (`if
a.idle.active { if armed { ... } }`, lines 277-282) is reached only when `idle
:= now.Sub(idleWakeup) >= 0` computes true (line 270), which requires
`a.idle.start` to have gone `idleTimeout` (3 min) without a refresh. If
`len(evts) > 0` is satisfied on effectively every iteration, `a.idle.active`
never becomes true, so the warning branch — and the wipe inside it — is never
even entered. No draw, no log: exactly the "silent" claim.

**Point 4 confirmed.** `(ctx.keepAwake && !armed)` is unconditionally `false`
whenever `armed == true` (Cut/Skip, per F-103, has `ctx.wipe = &wipeGuard{}`
with no job, so `armed()` is true — `gui/wipe_guard.go:49-60`). `ctx.KeepAwake()`
has exactly one caller in the whole tree, `gui/unlock_kdf.go:327` (the KDF
derivation, which has already finished before Cut/Skip). So on the Cut/Skip
screen `keepAwake` cannot be the refresh source under any value; `len(evts) >
0` is the only remaining term, exactly as the entry argues.

## Host test — built, run, and it reproduces the coupling

The existing harness in `gui/run_harness_test.go` (`deadlinePlatform`, comment
at lines 49-53) already documents this exact hazard by name: *"a platform that
appended a synthetic event every tick would refresh the idle clock forever and
NO timer could ever fire — the test would pass by never arming anything."* I
built the platform that comment warns against.

**Sketch** (written to `gui/f103_scratch_test.go`, run, then deleted — not
committed):

- `spuriousTouchPlatform` embeds `*deadlinePlatform` and overrides
  `AppendEvents` to `time.Sleep(p.tickFloor)` (so the `synctest` fake clock
  still advances one tick per poll) then unconditionally
  `append(evts, PointerEvent{Pressed:true, Pos: <alternating>}.Event())` —
  modelling jitter, matching `processTouch`'s equality-only dedupe.
- Flow: `ctx.wipe = &wipeGuard{}` (Cut/Skip's exact "armed, no job" shape),
  then `for !ctx.Done { ctx.Frame(op.Layer()) }`.
- Driven directly through `runWithFlow` under `synctest.Test`, capped at
  `maxRunFrames` (100,000 ticks × 10 ms floor ≈ 1000 s fake time — 4.8× past
  the 3:30 warning+wipe deadline), asserting the loop parks (`ctx.Done` never
  set — no wipe) and no frame ever contains `"decrypted seed material"` (no
  warning).

**Result, actually run** under
`/nix/var/nix/profiles/default/bin/nix develop /scratch/code/shibboleth/seedhammer --command go test ./gui/ -run TestF103 -v`:

```
=== RUN   TestF103SpuriousTouchNeverGoesIdle
    f103_scratch_test.go:82: calls=100000 frames=100001 parked=true warned=false
--- PASS: TestF103SpuriousTouchNeverGoesIdle (7.52s)
=== RUN   TestF103ControlNoSpuriousTouchWipesNormally
    f103_scratch_test.go:133: frames=32 parked=false warned=true
--- PASS: TestF103ControlNoSpuriousTouchWipesNormally (0.00s)
PASS
ok  	seedhammer.com/gui	7.530s
```

100,000 spurious-but-distinct touch polls, spanning ~1000 s of fake time —
zero warnings, zero wipes, flow parked forever. The control (identical flow,
ordinary `deadlinePlatform`, no spurious events) warns at ~3:00 and does not
park, showing the harness/flow shape itself is sound and the divergence is
caused by the spurious events, not a test bug. This turns the hardware
anecdote into a reproducible regression test; it is `TestF103...` in the
sketch above, ready to be written for real when B2c picks up F-103.

## A necessary reframing — do not stop at "screen film"

F-103's own diagnosis (screen film → continuous touch) was inferred logically
from the code on 2026-08-09, not measured with an event counter on that
session. The *next day's* investigation of a superficially identical symptom
— "nothing happens for several minutes" — is `design/HARDWARE_RESULT_2026-
08-10b_f106_ROOT_CAUSE.md`, and it is instructive: on that occasion the event
counter (`e`) was **flat** across the whole window (162 → 162 → 162, zero
events), which **refutes** phantom input for that specific measurement. The
true cause there was a completely different, unrelated bug (F-106: an armed
edge sampled only after a blocking `AppendEvents` call lands on the very
wakeup that was supposed to be the idle deadline, doubling the window to
6:00/6:30) — now fixed and closed 2026-08-10.

So "the wipe didn't fire in time" has **at least two independent causes** in
this codebase's history: F-106 (fixed, edge-timing, zero events involved) and
the `len(evts) > 0` coupling this report confirms (a live code hazard,
unfixed). F-103's specific 2026-08-09 incident was never re-instrumented with
an event counter the way F-106's was, so which of the two (or both, since
F-106's bug pre-dated its 2026-08-10 fix and was present during F-103's
session too) actually produced that day's "4:05, unchanged" is not fully
settled by measurement. What **is** settled, independent of that historical
attribution: the `len(evts) > 0` hazard is real, present in code today, and
now independently reproduced on a host. The reframing the task asked for:
**the risk is not "screen film," it is any source of continuous or
non-identical-reading touch noise** — moisture, ESD, panel debris, a marginal
connector, or a driver bug would trip the identical silent, permanent
disablement of the screensaver and the §10.2.4 wipe. The film is one instance
of the hazard class, not the hazard.

## Answers

**Mechanism, three sentences.** `run_flow.go:251` refreshes `a.idle.start` on
`len(evts) > 0` with no requirement that an event resolve to effective input,
so continuous or jittery touch-panel readings (film, moisture, debris, a
driver bug — anything `processTouch`'s exact-equality dedupe fails to
suppress) keep the machine perpetually "not idle." Because the §10.2.4 warning
is nested inside `if a.idle.active`, which is gated on the machine having gone
genuinely idle, a machine that never goes idle never enters that branch at
all — no draw, no log, total silence, and the screensaver (same clock, same
gate) is disabled identically. `ctx.keepAwake` is provably not the mechanism
on the armed Cut/Skip screen: its term is ANDed with `!armed`, and its one
caller (`unlock_kdf.go:327`) runs before Cut/Skip is ever reached.

**Entry's accuracy.** Accurate as a description of a real, unfixed code
hazard — confirmed by direct trace and by a host test that reproduces total,
silent, permanent disablement. Its attribution of *that specific 2026-08-09
incident* to the screen film specifically is a plausible, code-consistent
inference but was not measured with an event counter the way the superficially
similar F-106 incident was the next day (and F-106 turned out to have a wholly
different, event-free cause) — so treat the mechanism as confirmed and the
specific historical attribution as unconfirmed-but-plausible.

**Host-testable.** Yes — built and run this session (`gui/f103_scratch_test.go`,
not committed, deleted after use); sketch above. `100,000` distinct spurious
touch polls over `~1000s` fake time under `synctest`, zero warnings, zero
wipes; a same-shape control platform without spurious events warns at ~3:00
correctly, confirming the divergence is caused by the events, not the harness.

**Smallest fix.** The entry's own Option 2 (`design/FOLLOWUPS.md:1512-1521`):
key `a.idle.start`'s refresh on *effective* input — a resolved press/release
or a router-consumed event — rather than raw `len(evts) > 0`. That is a
normative change to §10.2.4 and needs the R0 loop. A smaller, defensive-only
supplement (Option 3) is a cap: refuse to let raw events refresh the clock for
more than some bounded duration of continuous "touch," or require at least one
intervening event-free tick — which would contain the blast radius of any
future spurious-input source without changing what a genuine tap does today.

Report persisted to:
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/2026-08-10-f103-screen-film-mechanism.md`
