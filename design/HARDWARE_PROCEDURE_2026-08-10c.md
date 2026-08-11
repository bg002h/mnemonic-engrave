# Hardware session — B2b's remaining gate (F-106, F-105, F-107/F-108 seam)

**Status:** written 2026-08-10, **not yet executed**.
**Blocks:** merging `b2b-residency` and `b2b-f106` into `b2b`.
**Precondition:** F-106 R0 round 1 returns GREEN. Do not flash a build carrying
an open Critical — the whole point of the gate is that hardware time is spent on
what tools cannot reach, not on defects a review would have caught.

## Why a session at all

Three of B2b's findings are closed on the host and cannot be closed there:

- **F-106** is a function of when `cmd/controller/platform_sh2.go`'s event source
  actually returns. The host harness models it; the device *is* it.
- **F-105**'s bracket is proven by reading the shipped code and by a host test;
  what is owed is one operator-visible reading.
- **F-107/F-108** zero seed-derived geometry. Zeroing *too early* wrecks a plate,
  and no host test can prove the head went where the plan said. Toolpath equality
  is already proven byte-identical across 5 plates
  (`design/TOOLPATH_EQUIVALENCE_2026-08-10.md`) — what is left is the **resume**
  path, which rebuilds the geometry the zeroing threw away.

## Build

The two branches are independent (`b2b-f106` touches `gui/run_flow.go` and the
Run harness; `b2b-residency` touches `gui/op/`, `engrave/`, `gui/engraver.go`,
`gui/gui.go`, the unlock brackets). Merge both into a **scratch integration
worktree** — not into `b2b` — so a failed reading costs no history:

```sh
git -C /scratch/code/shibboleth/seedhammer worktree add \
    /scratch/code/shibboleth/seedhammer-b2bint -b b2b-int b2b
cd /scratch/code/shibboleth/seedhammer-b2bint
git merge --no-ff b2b-residency        # two sequential merges, not an octopus:
git merge --no-ff b2b-f106             # a conflict then names ONE culprit
/nix/var/nix/profiles/default/bin/nix develop /scratch/code/shibboleth/seedhammer \
    --command go test ./...          # must be green before anything is flashed
SH2_REPO=/scratch/code/shibboleth/seedhammer-b2bint ~/bin/sh/sh2-flash -y
```

Always `sh2-flash`, never `picotool` by hand — the build output is unsigned and a
laptop port cannot boot the machine. If the screen stays dark after flashing,
**wait** before reflashing: PD negotiation is sometimes slow, and a slow
negotiation looks exactly like a rejected signature. Do **not** burn another OTP
slot for any boot failure.

Record the firmware id (`v0.0.0-g<sha>`) shown on the version screen against
every reading below.

## Run it in the simulator FIRST

Three of the four readings run in `sh-sim` (`cmd/emu`, a `GOOS=js` build of the
same `seedhammer.com/gui` the firmware ships), and doing so costs no flash cycle:

```sh
sh-sim b2b-int        # build cmd/emu from the integration branch, serve it
```

| reading | simulator | why |
| --- | --- | --- |
| 1 — Cut/Skip untouched | **yes** | same `run_flow.go`, and `cmd/emu/platform.go:150` is a real park on `{timer, wakeups, events}` — the same shape as `platform_sh2.go:369-396` |
| 2 — Back mid-cut | **yes** | `emuEngraver.Write` sleeps 1 ms per write, so a cut has duration to abort, and the mechanism under test — `defer e.pl.Wakeup()` reaching the platform's select — is implemented |
| 3 — passphrase bracket | **yes** | GUI and timing only |
| 4 — abort → resume | **motion, yes; steel, no** | `emuEngraver` now DECODES the step stream (`emu-toolpath` branch, `4b68c18`) — see below |

Shrink `idleTimeout` in the simulator build to iterate in seconds rather than
waiting 3:00 per attempt — there is no `synctest` in a browser, the clock is real.

**Reading 4 in the simulator.** `window.shToolpath` reconstructs where the head
would actually have gone, decoded from the DMA words the driver emits — so the
resume comparison is a digest, not an inspection:

```js
shToolpath.reset()                              // then cut straight through
a = JSON.parse(shToolpath.summary())
shToolpath.reset()                              // then Back mid-cut, hold to resume
b = JSON.parse(shToolpath.summary())
a.digest === b.digest                           // same motion?
b.returnsToOrigin                               // must be FALSE
```

`returnsToOrigin` is the F-108 failure by name: a zeroed `SafePointer.history`
makes `Resume` feed `appendLine` from a cleared `safePoint`, and the catch-up
drives home at `T:0`. `shToolpath.svg()` draws it if you want to see it.

This is what the motion *would* be, not what the steel *is*. Stroke depth, burr,
smear and anything mechanical still need reading 4 on the machine.

**This does not remove the hardware session.** It changes what a hardware failure
*means*: with the simulator green, a device that disagrees is telling you
something about `platform_sh2.go`, TinyGo's runtime, or the machine — which is
the only residue hardware time is worth spending on. The emulator uses a
different platform implementation, so a green simulator is strong evidence and
not proof.

Never settled by the simulator, at any effort: `platform_sh2.go` itself,
TinyGo `-gc precise` zeroing and every 32-bit memory measurement, and anything
physical — stepper motion, load, stalls, plate mechanics.

## Readings

Timings are wall-clock from the stated instant. `idleTimeout` = 3:00,
`wipeWarningDelay` = 30 s, so a correct window is **warning at 3:00, wipe at
3:30**. F-106's signature is exactly double: 6:00 / 6:30.

### 1 — F-106, the original shape (**do this first**)

Unlock the sealed payload, advance to **Cut/Skip**, then **do not touch the
machine**.

| want | fail |
| --- | --- |
| warning animates at **3:00** | 6:00 → F-106 not fixed |
| wipe fires at **3:30** | 6:30 → same |

*Why first:* F-105's reading is confounded while F-106 is open — a doubled window
changes what "the passphrase was still there" means.

### 2 — F-106, the `engraveStopping` park (R0 round 0, I2)

Start a cut, let it run ~1:00, press **Back** to abort, and let the head stop.
Then do not touch the machine.

| want | fail |
| --- | --- |
| warning **3:00 after the head stops** | anything later → the arm edge waited for the idle deadline |

*Why this case specifically:* `EngraveScreen` installs its 500 ms poll only while
`Status().State == engraveRunning` (`gui/gui.go:2766-2769`). `Stop()` moves the
job to `engraveStopping`, for which `armed()` is still false and the poll is
**not** set — so the loop parks to the idle deadline and **only `pl.Wakeup()`**
ends the park. It is also §10.2.2's most ordinary recovery, not an exotic path.

### 3 — F-105, the passphrase bracket (Task 9.5)

At the passphrase keyboard, type **two words**, then stop and wait.

| want | fail |
| --- | --- |
| warning at **3:00** from the last keypress | no warning at all → an F-105 defect in its own right, not F-106 spilling over |

### 4 — F-107/F-108, the abort→resume seam

Use a **single-character** plate (top-left, uncentred): ~2 s per cut against
~21 min for a full plate, and it exercises the same code path.

1. Start the cut, press **Back** mid-cut, let the head stop.
2. **Hold to resume.**
3. Inspect the plate.

| want | fail |
| --- | --- |
| the resumed cut continues from where it stopped | head drives to the origin → resume state was zeroed while a restart was still reachable (R0 round 1's Critical, regressed) |
| the finished character is correct and unsmeared | any doubled or offset stroke → the catch-up array was zeroed before the fast-forward consumed it |

4. Then press **Back** on the *finished* plate and start a new cut. It must begin
   from the **start**, not resume — that is `releaseResumeState` doing its job on
   an abandoned job.

## What closes what

| reading | closes |
| --- | --- |
| 1 + 2 green | **F-106** — then `b2b-f106` merges |
| 3 green | **F-105** Task 9.5 |
| 4 green | the hardware half of **F-107/F-108** — then `b2b-residency` merges |

All four green → merge both branches into `b2b`, and push via the `ci/staging`
ref so the commit **earns** its required check instead of bypassing it:

```sh
git push origin b2b:refs/heads/ci/staging
gh run watch <id>                  # wait for `test (rust + go)`
git push origin b2b
git push origin --delete ci/staging
```

A push that prints "Bypassed rule violations" means the staging step was missed.

## Record the result

Write the readings to `design/HARDWARE_RESULT_2026-08-10c_b2b_gate.md` — the
actual annotations, not a summary of them. Past sessions have been decided by a
single field in the diagnostic overlay (`w`, `t`, `e`, `A!`), and the phantom-input
hypothesis for F-106 died on one number that had been transcribed rather than
read. If a reading disagrees with what this document expects, the reading wins.
