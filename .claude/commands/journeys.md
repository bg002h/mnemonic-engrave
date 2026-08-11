---
description: Resume the operator-journeys phase; first task is the simulator plate-layout overlay
---

Resume the **operator journeys** phase for the SeedHammer II constellation.

Read these first, in order, and do not re-derive what they already record:

1. `design/PREP_journeys.md` — the map: the nine top-level programs (read from
   `gui/gui.go`'s `program` enum, not the title switch), the constraints any
   operator-facing document inherits, and the first task with its traps.
2. `design/CONTINUITY_2026-08-11b.md` — the state that prep note builds on.
   Note it supersedes `CONTINUITY_2026-08-11.md`, which has an error in it.

**First task, by operator ruling:** extend the **simulator** to display the
**final layout of a cut plate at the beginning of the engrave**, and to
**indicate on that layout what is currently being engraved**.

Both halves already exist — `cmd/plateview` renders the plan from the same
calls the firmware makes, and `cmd/emu`'s `toolpathRecorder` decodes the
driver's real step stream as `window.shToolpath`. The work is joining them.
**The trap: F-121 — the emulator does not home while the device does**, so an
overlay aligning recorded motion onto planned geometry will render a resumed
cut offset unless it accounts for that.

Before proposing an approach, confirm the current state yourself: both repos
clean and pushed, and check whether the flashed firmware's boot has been judged
on machine power yet (it had not been, as of the prep note).
