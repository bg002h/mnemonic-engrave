# Hardware result 2026-08-10c — B2b's merge gate

**Build:** `v0.0.0-g747cf48` (`b2b-int` = `b2b` @ `3de8aa1` + `b2b-residency` +
`b2b-f106` + the `-race` fix), sha256 `f5a71e19f79bb51a2d5b3ea6d0241c76029fd9ca3e26d147e690f5be77b60b27`.
Signature verified before flash; flash verify OK.
**Procedure:** `design/HARDWARE_PROCEDURE_2026-08-10c.md`.
**Operator:** bg. Readings as reported at the machine.

## 1 — F-106, the original shape — **PASS**

Sealed payload unlocked, Cut/Skip, untouched.

| moment | want | got |
| --- | --- | --- |
| warning | 3:00 | **3:00** |
| wipe | 3:30 | **3:30** |

Against the defect's own reading on `b2b-idleprobe3` (`256b38c`): warning 6:00,
wipe 6:30. **The 2x is gone.**

## 2 — F-106, the `engraveStopping` park — **PASS**

Cut started, aborted with Back at ~1:00, head allowed to stop, then untouched.

| moment | want | got |
| --- | --- | --- |
| warning | 3:00 after the head stops | **3:00 after the head stops** |
| wipe | +30 s | **+30 s** |

This is the case R0 round 0 raised as I2 and the one no `syncArmed` call covers:
`EngraveScreen` installs its 500 ms poll only while the job is `engraveRunning`,
so a job moved to `engraveStopping` parks to the idle deadline and **only**
`pl.Wakeup()` (`gui/engraver.go:110`, received at
`cmd/controller/platform_sh2.go:384`) ends the park. Confirmed on the device.

## 3 — F-105, the passphrase bracket (Task 9.5) — **PASS**

Two words typed at the passphrase keyboard, then left.

| moment | want | got |
| --- | --- | --- |
| warning | 3:00 from the last keypress | **3:00** |
| wipe | 3:30 | **3:30** |

Row 4's window starts from a real event, as designed. **Task 9.5 is closed.**

## 4 — F-107/F-108, the abort→resume seam — **PASS**

### 4a — "Engrave Text", abort mid-cut then hold to resume — PASS

> "head was near origin (top left) but upon resuming went a short distance
> towards top left and then directly to where it left off. Tracked perfectly
> with completion of the letter that was interrupted mid-engraving."

**The letter completing perfectly IS the result.** The failure reading 4 exists
to catch is resume state zeroed while a restart is still reachable, and that
produces WRONG GEOMETRY, not a residue. Exact tracking through the interruption
proves `safePoint` and `history` survived F-107/F-108's zeroing.

**The detour toward the origin is correct, pre-existing behaviour**, and it
corrected a belief of mine rather than revealing a defect —
`engrave/engrave.go:1664`:

```go
move = appendLine(move, conf, false, bezier.Point{}, s.safePoint)
```

`Resume` synthesises its approach FROM the origin, and `appendLine`
interpolates in absolute coordinates, so the head tracks toward (0,0) before
running out to the safe point. Needle up, nothing cut. Short here only because
the work sat near the origin. See **F-114**.

### 4b — abandoned job releases its resume state — PASS

Back out of a paused cut, re-enter the plate: it offers "Insert a blank
plate… Hold button to start", not "Engraving paused". `releaseResumeState` is
firing on the abandoned job.

### 4c — abort→resume INSIDE the secret session — PASS, all 5 steps

A real record plate from the sealed payload: start, Back at ~1:00, hold to
resume, run on, abandon. Session remained coherent, the next plate offered
normally, and the §10.2.4 window still fired on leaving.

This is the reading 4a/4b could not give: the wipe guard installed around a cut
with secrets resident, across an abort and a resume.

## What this closes

| item | status |
| --- | --- |
| **F-106** | CLOSED — both shapes green (readings 1, 2) |
| **F-105** | CLOSED — Task 9.5 (reading 3) |
| **F-107 / F-108** | hardware half CLOSED (reading 4) |
| **F-110** | abandoned-job release confirmed operator-visible (4b) |

`b2b` fast-forwarded to `747cf48` — **the exact SHA that was flashed and
tested**, bit for bit, rather than a rebuild of it.

## One correction this session produced

The `cutsThroughOrigin` detector in `cmd/emu` was written as "the path returns
to the origin", because I believed a dive home was the F-108 signature. Reading
4a showed that describes the HEALTHY path. No host test could have caught it —
every test I wrote encoded the same wrong belief. Fixed in `seedhammer`
`c38cb6b`: the flag now requires the needle to be DOWN.
