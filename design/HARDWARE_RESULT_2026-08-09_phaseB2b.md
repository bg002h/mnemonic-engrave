# Hardware result — Plan B Phase B2b (§10.2.4's residency-keyed idle wipe)

**Date:** 2026-08-09 · **Operator:** bg · **Machine:** SeedHammer II (RP2350B)

**IN PROGRESS.** Recorded as observed, step by step, rather than written up at
the end. Anything not yet observed says so.

## What was flashed

| | |
| --- | --- |
| firmware | `v0.0.0-ge8e78f0` — B2b Tasks 1–7 complete, plus the `Buffer.Scrub` fix |
| signed image sha256 | `8842b5076f9a0f924177c43be1ed8fd0b90caf8847dd68c41894ebf3d858fe8a` |
| build header read | `e8e78f0 gui/op: SCRUB the abandoned frame buffer` — **not** `a01b666` |
| payload | vector F — 15 secret records (3 `ms1`, 6 `mk1`, 6 `md1`), **0 public** |
| iterations | 300,000 (the default) |
| device budget | 1312808 flash / 60584 ram — **RAM unchanged from the `a01b666` baseline** |

**Order was payload first, firmware second**, deliberately, so that the payload
surviving the reflash could be observed. `picotool load --verify` reported OK for
both.

## Observed

### Boot

- [x] **Boots on machine power.** Signature accepted by the bootrom against the
      slot-1 key burned 2026-08-03. Judged on the 20–28 V PD supply, not a laptop
      port — `Init()` checks for the PD contract before configuring the LCD, so a
      dark screen on USB is not a signature rejection.

### F-100 / SPEC §11.5 — **CLOSED**

- [x] **The payload survived a firmware reflash.** Start screen shows Sealed
      Payload present with **9 pager dots** — B1's recorded baseline is 8 dots
      absent, 9 present.

      This is the first time §11.5's *"confirm firmware reflash preserves the
      blob"* has been executed by anything. B1's hardware run covered four things
      and this was not among them; its closest statement was the **converse**
      ("only the 64 KB payload region was cleared; B1's firmware was untouched").
      Filed as F-100 by the residue sweep on 2026-08-09 because it was owned by
      nobody; closed the same day, for free, by ordering the setup correctly.

### 8.1 — the walk-away wipe — **PASSES, on every observation**

- [x] **Warning at 3:00.** `WIPING SECRET DATA` appeared on the *Cut this plate /
      Skip* screen, which is armed (guard installed, no engrave job registered).
- [x] **The transition is INSTANTANEOUS, with no blank screen.** Straight back to
      the home screen.

      **This is the observation the whole design turns on, and it cannot be
      recovered after the fact.** Instantaneous means the flow *unwound*:
      `ctx.Done` was set, control returned through every parked `ctx.Frame`, each
      deferred wipe ran on the way out, and `Run` re-entered the UI with a fresh
      `Context`. A blank interval would have meant a **reboot**, which looks
      similar to an operator and guarantees nothing about the defers.

      So "the unwind IS the wipe" — the claim the phase was built on — holds on
      real hardware.
- [x] **The wipe fired at EXACTLY 3:30.** Operator's stopwatch, restarted at the
      last touch after the screen film was removed.

      So both deadlines landed on their nominal values with no drift over the
      3½-minute window: warning at `idleWakeup` = 3:00, wipe at
      `idleWakeup + wipeWarningDelay` = 3:30. The arithmetic is one clock
      (`a.idle.start`) plus two constants, and it holds on `time.Now()` with no
      RTC.
- [ ] countdown's first number (expect 30) — not separately recorded; the 3:30
      wipe implies a 30 s warning window, but the *displayed* first number was
      not read. Minor, and only worth a re-run if the countdown text is ever
      suspected.

### 8.1a — the KDF, and a CORRECTION to my own analysis

| | |
| --- | --- |
| iterations | 300,000 |
| **wall clock, stopwatch** | **40.2 s ± 1.0** |
| **device's own on-screen estimate** | **40 s** — independent instrument, agrees to within tenths |
| §7.1's `cmd/kdfbench` figure | 9,715 it/s → **30.9 s of DERIVATION** |

**These do not conflict, and my first reading that they did was WRONG.** §7.1's
9,715 it/s is a harness measurement of PBKDF2 alone — the time inside `d.Step`.
Its "300,000 → 30.9 s" is 300,000 ÷ 9,715, i.e. **derivation time**. The
stopwatch measures **wall clock**, which additionally covers ~600 full-panel
repaints. Two different quantities.

`gui/unlock_kdf.go:217-229` distinguishes them deliberately and warns about this
exact mistake:

> *"§7.1 is closed by … reading the log line below off the real machine … so the
> line must report the DERIVATION, not the wall clock. Wall time here also covers
> ~600 full-panel repaints … §7.1's own history is a rate estimate wrong by 1.54×
> that set the default to 450,000; a parked wall-clock reading would repeat that
> error larger, and this time with the number 'measured on the real part'."*

I took a wall reading, computed 7,463 it/s, declared §7.1 falsified, and changed
`me seal`'s default to 230,000 — reducing the KDF work 23% on a category error.
**Reverted (`7c4a7b4`); the default is 300,000, as §7.1 specifies.**

**What the measurement DOES establish, and it is worth having:**

1. **The operator experiences ~40 s, not 30.9 s — established twice, independently.**
   The stopwatch gave 40.2 s ± 1.0; the device's own self-calibrating estimate
   displayed **40 s**. That estimate is computed from *this* derivation
   (`unlockKDFLead`, `gui/unlock_kdf.go:187-204`) rather than read off §7.1, and
   it extrapolates `elapsed` — which includes the repaints — so it is a **wall**
   figure too. Two instruments, same quantity, same answer. The spec's headline is
   derivation time, and the ~9.3 s of repaint overhead (~15.5 ms across 600
   frames) is real and not documented anywhere an operator would look. The code
   comment says §7.1 "separately asks for" this number — *"the number the
   operator actually experiences"* — and this is it, now measured twice by
   independent instruments.
2. **§7.1's in-situ DERIVATION rate is still not measured.** Closing it needs the
   `log.Printf("seal: kdf %d iterations in %s derived (%s wall)")` line read off
   the machine, which this build has no console for. **§7.1 remains open.**
3. **F-93's threshold should use the WALL rate, and that recalculation stands** —
   parking is a wall-clock phenomenon, since `idleTimeout` is wall time. At
   ~7,463 it/s wall the park threshold is 180 × 7,463 = **1,343,284** iterations,
   so **34.6%** of the legal 100k–2M range would have hung pre-Task-5 firmware,
   not the 13.2% recorded — which was computed from the *derivation* rate and is
   the same conflation, in the plan, in the other direction.

## CRITICAL — the device HANGS re-entering the sealed payload after a wipe

**Observed 2026-08-09, immediately after 8.1's successful wipe.** This is the
first time the post-wipe restart path has run on hardware, and it is a **hang on
a watchdog-less device**.

### What happened

1. 8.1's wipe fired at 3:30 and the UI restarted to the home screen — correctly,
   instantaneously, no blank.
2. The operator navigated the program carousel to **Sealed Payload (dot #9
   active)** — so **input and drawing were both alive**.
3. Pressed the **checkmark** to enter the flow. **Nothing happened.**
4. **No screensaver at 3:00. None at 3:30. Still unresponsive at 4:00.**

### Screen state, captured before the power-cycle

- Program carousel with navigation arrows and the checkmark button
- **Program nav dot #9 active** (Sealed Payload)
- Version line **present and correct** — `v0.0.0-ge8e78f0`, hardware 1.5
  `(UNLOCKED)`, bottom right; "SeedHammer" bottom left
- **Nothing responds** — no button, no region of the panel
- **First** unlock attempt after the wipe

### Why the missing screensaver is the diagnostic, again

The saver, the §10.2.4 timer and touch handling all live in **one goroutine**.
All three stopped together. That is not three features failing; it is that
goroutine being gone or blocked. The film explanation does not apply — it was
removed, and 8.1 subsequently ran perfectly on the same session.

### NARROWED BY EXPERIMENT — the wipe is required

| sequence | result |
| --- | --- |
| enter → exit Sealed Payload, repeatedly | **works** |
| full unlock → **normal** exit → re-enter | **works** |
| full unlock → **wipe** → re-enter | **HANGS** |

Both control experiments were run on the machine. They **exonerate the NFC
shutdown deadlock** as the primary cause: `StartScreen.Flow`'s exit defer runs on
*every* program entry and completed many times without hanging. They also show a
single 64 KiB alloc/free cycle is not sufficient — entering and exiting the
payload flow does exactly that.

**So the trigger is what the wipe does that a normal exit does not.** A normal
exit reuses the same `Context`. **A wipe abandons it and allocates a fresh one**,
stranding the old `ctx.B` — grown across a full session including the seed
screen — alongside `a.warnBuf` (grown during the 30 s warning) and `a.mask`.
Re-entry then asks for **64 KiB contiguous** under `-gc precise`, which is
**non-moving**.

**This is a design decision of this phase, not an upstream bug.** The plan says:
*"A fresh `Context`, not a scrubbed one … a wipe is rare enough that the
allocation is irrelevant."* Round 0 accepted it. That reasoning weighed the
allocation's **cost** and never considered the **garbage it strands**, nor that a
non-moving collector plus a 64 KiB contiguous demand makes fragmentation a
**correctness** issue rather than a performance one.

### The number that reframes the fix

`seal/wire.go:192` records the format's own arithmetic: the largest payload the
wire format admits is **52 + 8191 + 8191 + 16 = 16,450 bytes**. `XIPReader.Read`
allocates **65,536** unconditionally — **about 4× more than anything legal can
occupy**. The 64 KiB contiguous demand is not justified by the format; it is the
flash *region* size used as an allocation size.

### Superseded hypothesis

An earlier reading here was that `Run` had returned. The **pressed-button frame**
refutes it: a returned `Run` could not have drawn it.

### Leading hypothesis, NOT yet confirmed

`Run` **returned** instead of looping. If `ctx.Done` goes true in the restarted
session while `wiping` is false, then `if !wiping { return }` exits `runWithFlow`,
`cmd/controller`'s `for range gui.Run(p, ver) {}` ends, `main` returns — and the
device is left with the last frame frozen and nothing servicing input. That
matches every symptom including the frozen-but-correct carousel.

It is consistent with the `FrameCallback` early return added by Task 3: once
`ctx.Done` is true, `ctx.Frame` returns without yielding, so no further frame is
ever drawn.

**What is not yet known:** why `ctx.Done` would become true on that transition,
and whether it reproduces. Recorded as a hypothesis, not a cause.

### What this cost the review process

Four R0 rounds, a 4-lens pre-hardware preflight with a dedicated **brick/hang
lens**, and 16/16 mutation kills did not find this. The post-wipe restart is a
path **no test exercises on hardware**, and Task 3's own tests drive it only
through the `wipeNowHook` seam in a host harness.

## Not yet observed
- [ ] **8.2** — the touch reset, with a duration
- [ ] **8.3** — the mid-cut plate
- [ ] **8.4** — remaining payload-survival observations (re-unlock, power-cycle)
- [ ] **8.5** — the full record

## Known gap in the procedure, recorded before it is hit

**Task 8.4's "confirm the §6.6 public-data hash matches" cannot be satisfied with
this payload.** Vector F has **zero public records**, so `me seal` printed no
public-data hash line and there is no baseline to compare against. The
observation is dropped rather than fudged; the plate-list comparison at re-unlock
still stands.

## Procedure defects found during setup, both of the same class

Both would have cost the trip, and both are "a bare command name resolved to the
wrong artifact":

1. **`sh2-flash` with no `SH2_REPO`** builds `/scratch/code/shibboleth/seedhammer`
   — this phase's **parent** at `a01b666`, where `run_flow.go`, `wipe_warning.go`
   and `wipe_guard.go` do not exist. Caught by the pre-hardware preflight and
   fixed in the plan before flashing; the `== Build ==` header now carries a pass
   condition.
2. **`me` on `PATH` is v0.3.0 and has no `seal` subcommand at all** — it failed
   with `unrecognized subcommand 'seal'`. The repo is v0.4.0. Found by running it.
   Task 8 should name the build, not the command. **Not yet folded into the plan.**
