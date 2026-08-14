# Emulator pace-1 baseline measurement — INCOMPLETE

Agent: emulator speedup measurement. Date: 2026-08-14. Machine-local wall clock (MST).

## STATUS

**INCOMPLETE. The pace-1 baseline walk never ran to completion, and no
six-plate pace-1 total was measured.** Nothing in this file may be read as a
finished measurement of the pace-1 walk.

- What ran: `run({ pace: 1, plates: 2, pollMs: 150 })` from
  `/scratch/code/shibboleth/seedhammer/cmd/emu/walk_trace_a.js`, started
  **16:02:20** against a freshly served emulator at
  `http://127.0.0.1:8473/index.html`.
- Where it got to: **plate 1 complete, plate 2 roughly half cut**, at the last
  reading **16:35:4x** — about **33 minutes** of wall clock for 1.5 of 6 plates.
- Why it stopped: the MCP `browser_evaluate` call carrying the walk hit the
  1800 s idle timeout at 16:32, and the run was then wound down by the
  coordinator rather than restarted. The walk was holding the single shared
  browser session that every stage gate in the current plan needs, and the
  precise pace-1 ratio was not worth another hour of that.
- What finishing it would take: the same call with `plates: 6`, and an
  estimated **1.5–1.8 h more wall clock** beyond the 33 min already spent
  (see the estimate section — that figure is derived, not measured). It also
  needs an invocation that survives the MCP idle timeout: either a per-server
  `timeout` raise, or a fire-and-forget launch (`run(...).then(r => {
  window.__walk = r; })`) that returns immediately and is polled from later
  calls. The second form is strongly preferred and is the reason this run
  produced only two data points.

## MEASURED THIS SESSION

Two direct readings of `window.shToolpath` on the live page. Both timestamps are
tool-call boundaries, not instrumentation, so each carries roughly ±15–20 s.

| # | time (approx) | `strings.length` | `announced` | `unattributed` | plate-2 `steps` |
| --- | --- | --- | --- | --- | --- |
| A | 16:32:5x | 1 | 6 | 0 | 77,484,720 |
| B | 16:35:4x | 1 | 6 | 0 | 102,478,698 |

- `shPace()` read back **1** at reading B, so the override was in force (the
  emulator's own default is `defaultPace = 2048`, `cmd/emu/pace.go:85`).
- `unattributed == 0` at both readings — no unnamed motion (the F-160 class of
  failure did not occur in what did run).
- The step counter is **per plate**, not cumulative: `beginPlate` calls
  `rec.Reset()` at the start of a genuinely new plate
  (`cmd/emu/plate.go:172-179`). So both readings are plate 2 alone. This was
  verified in source, not assumed.
- Screen text at both readings: `12:54EngravingplateEngravePlate` — identical
  across the ~3 min gap, i.e. a stale frame. Not interpreted further here.

**Step rate at pace 1** — the one clean number this run produced:

```
Δsteps = 102,478,698 − 77,484,720 = 24,993,978
Δt     ≈ 165 s (16:32:5x → 16:35:4x, ±35 s)
rate   ≈ 1.5e5 steps/s        (1.25e5 at Δt=200 s, 1.9e5 at Δt=130 s)
```

**Not measured:** plate-1 elapsed time (the walk was unattended between 16:02
and 16:32, so no boundary was captured), any per-plate digest, any six-plate
total, and any pace-1 vs pace-2048 ratio.

## ESTIMATE (derived, NOT measured)

Everything in this section is arithmetic on the two rows above. It is an
estimate and must not be quoted as a measurement.

Taking the central rate 1.51e5 steps/s:

```
plate 2 elapsed at reading B = 102,478,698 / 1.51e5   ≈ 679 s ≈ 11.3 min
wall clock, walk start → B                            ≈ 33.3 min
plate 1 + nav/gather (~1 min)                         ≈ 33.3 − 11.3 − 1.0
plate 1                                               ≈ 21 min  (≈ 1.9e8 steps)
```

Scaling by chunk length (measured from `CARDS` in `walk_trace_a.js`: three
chunks of 111 chars and three of 80, so plate 1 is a 111-char chunk):

```
per-char cost      = 21 min / 111 chars      ≈ 0.189 min/char
six-plate total    = 3×(111+80) × 0.189      ≈ 108 min ≈ 1.8 h
naive 6 × plate 1                            ≈ 126 min ≈ 2.1 h
ratio vs the 186 s pace-2048 walk            ≈ 108×60/186 ≈ 35×
ratio vs the ~165 s default-pace walk        ≈ 108×60/165 ≈ 39×
```

So the pace-1 baseline is **plausibly 1.8–2.1 h, i.e. of order 35–40× the
default-pace walk** — an order-of-magnitude claim at best. The linear-in-chars
assumption is untested, plate-boundary prompt time is folded into the ~1 min
nav allowance rather than measured, and the rate itself has a ±25 % band.

One inconsistency a finisher should resolve rather than inherit: ~1.9e8 steps
per plate implies ~1.1e9 steps for six plates, which at pace 2048 would have to
be consumed inside a 186 s walk that is already overhead-dominated (2048→186 s
vs 8192→183 s). That is ~3e7 steps/s. It may well be real — a step is a counter
increment — but it has not been checked, and if it is wrong then one of the two
figures is.

## WHAT IS ALREADY KNOWN

Inherited from the prior session's notes. **Not measured by this agent** — they
are restated here so the file is useful at zero measured rows, and they should
be re-verified before being leaned on.

- A six-plate Trace A walk at the default pace takes **~165 s**.
- **pace 2048 → 186 s** vs **pace 8192 → 183 s**: a 4× pace increase buys ~3 s,
  so at 2048 the walk is already dominated by driver overhead (`pollMs`,
  `settleMs`, `chunkGapMs`, and the 1.3 s hold), not by cut speed. This is why
  `walk_trace_a.js` documents those three knobs as the lever on a big bundle
  rather than the pace.
- The **step stream is pace-independent**: the same six toolpath digests were
  produced at pace 64 and at pace 512. Pace changes how often the engrave
  goroutine yields, not what it cuts — which is what makes a pace-1 baseline a
  timing question only, with no correctness content.

## SESSION HYGIENE

The browser page was **closed and the Playwright session released** at the end
of this report, so the shared session is free for the next user. The background
timer this agent had set (a 2-minute tick loop) was stopped explicitly; no
timers or background tasks remain. No files outside this report were modified,
and no commit was made.
