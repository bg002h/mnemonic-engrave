# Hardware result 2026-08-10 — F-106 DID NOT REPRODUCE on a fresh boot

**Build:** `v0.0.0-g1da54e1` (branch `b2b-idleprobe`, DIAGNOSTIC — never merge),
signed, flashed and verified; sha256 `a78f8324…`.
**Instrument:** the idle-clock overlay, `gui/idleprobe.go`. Format:
`idle <n>s w<site> t<ticks> e<evtTicks> <A|-> <lastEvent>`.

## The readings, in order

| moment | overlay | reading |
| --- | --- | --- |
| boot, start screen | `idle 0s w48 t2 e0 -` | two ticks, no events; frozen (see "the probe's blind spot") |
| start screen, ~3:00 untouched | *(screensaver — no overlay)* | **the screensaver activated on schedule** |
| after dismissing it | `idle 0s w151 t1390 e2 - Pu0,0` | 1390 ticks, **2** event-ticks — the operator's own press+release |
| Cut/Skip, straight after unlock | `idle 0s w151 t2122 e133 - Pu0,0` | `e133` ≈ the twelve words' own taps |
| warning appears, ~3:00 later | `idle …s w170 t2123 e133 A Pu0,0` | **one tick later. One 3-minute block, served.** |
| last frame before the wipe | `idle 209s t2153 e113 A Pu0,0` | 30 frames at 1 Hz; **`e` constant throughout** |
| wipe | — | fired at `idle 210s`, instantly to the home screen |

## What is now established

**The whole §10.2.4 cycle ran correctly with ZERO input.** `e` did not move
between the Cut/Skip frame and the wipe. The window opened, the warning drew, the
countdown ran and the wipe fired, with nothing touching the screen.

- **`t2122 → t2123`** across the three-minute wait. Exactly **one**
  `AppendEvents` call: the platform blocked on the deadline and its own timer
  released it. The reused-`*time.Timer` pattern in `platform_sh2.go:369` is not
  dropping long deadlines.
- **`t2123 → t2153`**, +1 per second, with `idle` +1 per second. The warning
  branch's `ctx.WakeupAt(now.Add(time.Second))` is served just as accurately.
  **The platform serves 1-second and 3-minute deadlines alike.**
- **`w170` and `A`** — the armed edge fired and row 2 wrote a fresh window at the
  arm. Task 9's guard installs. §10.2.4 arms.
- **`idle 210s` at the wipe** is exactly `start + idleTimeout + wipeWarningDelay`.
  The arithmetic is right to the second.
- **No phantom input anywhere.** 2 event-ticks across 1390 on the start screen,
  both the operator's; 133 across the unlock, all the operator's typing; **zero**
  across the entire three-and-a-half-minute window.

**So family A1 is dead, the platform is exonerated, and so is `gui/`** — which the
host diagnostic had already argued (`DESIGN_f106_idle_timer_never_starts.md`).

## What this does NOT establish

**F-106 is not explained — it is now known to be conditional.** Yesterday the same
screen, on the same machine, with the protective film already removed, sat for
**4:00 and then 4:15** with no warning and no wipe. Today it fired at 3:00 on the
first try.

The difference that survives: **session history.** Yesterday's failing runs came
after earlier unlocks and at least one wipe; today's was the **first unlock after
a cold boot**. That is also the state in which the post-wipe re-entry hang lives,
so the next experiment tests both at once: re-enter Sealed Payload in session 2,
unlock again, and touch nothing.

An intermittent funds-safety timer is **not** closable by one green run. F-106
stays open and still gates the phase.

## Two defects in the instrument, both worth carrying forward

1. **The overlay reports state one tick stale.** It is drawn *before* the tick
   that evaluates `armed` and writes `a.idle.start`, so on a screen that draws
   once and then blocks, `A/-` and `w` are the *previous* tick's values. This
   made the Cut/Skip frame read `-` and `w151` when the arm fired microseconds
   later — briefly misread here as "the guard never installed". The readout
   belongs *after* the evaluation, and a v2 should move it.
2. **`n` freezes when nothing redraws.** A quiet, correct device draws no frames
   for three minutes, so the counter is a photograph, not a clock. Every long
   quiet stretch is therefore unobservable exactly where it matters.

One reading remains unexplained: the **first** warning frame was transcribed as
`idle 0s` where the model requires `180s`, while the last read `209s` and the
wipe fired at `210s` — self-consistent. Operator has video. Most likely a
transcription of the still-frozen previous frame; recorded rather than dismissed
because if it is real, something wrote the clock at that tick.
