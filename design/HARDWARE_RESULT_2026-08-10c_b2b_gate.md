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

## 4 — F-107/F-108, the abort→resume seam — **NOT YET RUN**

See the procedure. Note the correction recorded there: a **finished** plate
(`engraveDone`) renders a single FORWARD button, not Back
(`gui/gui.go:2895-2901`), so the earlier instruction to "press Back on the
finished plate" was wrong.

## What this closes

- **F-106** — both shapes green on hardware. The follow-up's stated condition
  for closing was exactly these two readings.
- **F-105** — Task 9.5, the only part still owed.

`b2b-f106` is clear to merge. `b2b-residency` still waits on reading 4.
