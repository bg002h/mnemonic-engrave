---
description: Resume the operator-journeys phase; the simulator overlay is DONE — pick the next functionality item
---

Resume the **operator journeys** phase for the SeedHammer II constellation.

Read these first, in order, and do not re-derive what they already record:

1. `design/PREP_journeys.md` — the map: the nine top-level programs (read from
   `gui/gui.go`'s `program` enum, not the title switch), the constraints any
   operator-facing document inherits, and the first task **with its outcome**.
   Read the task section to its END before acting on the heading; the heading
   names the task, the body records that it is finished.
2. `design/CONTINUITY_2026-08-11b.md` — the state that prep note builds on.
   Note it supersedes `CONTINUITY_2026-08-11.md`, which has an error in it.

## The first task is DONE — do not rebuild it

**The simulator plate overlay shipped 2026-08-11**, fork `77774ec` ("emu: draw
the plate's layout while it cuts, and home as the machine does"). The layout is
drawn from the plan the moment the cut begins and the decoded step stream marks
progress on top of it. Six tests in `cmd/emu/plate_test.go` cover the Go half,
`planPath` matches `internal/golden.Vectorize` byte for byte, and it was
confirmed in the browser — abort mid-"hello", hold to resume, and the resumed
strokes land on the plan.

**F-121 is CLOSED**, fixed as a prerequisite rather than a neighbour: the
emulator now homes, because the overlay cannot register without it. It is no
longer a trap to design around.

Two things that will otherwise cost a session:

- **`qaProgram` is unreachable in `cmd/emu`** — `NFCReader()` returns nil, so
  `shNFC.present("FOREVERLAURA!")` queues a record nothing consumes. Driving a
  real engrave in the emulator means going through a normal program, not QA.
- Everything above is already verified. **Re-verifying it is the exact
  over-investment the operator ruled against on 2026-08-12** (see the Phase
  policy at the top of `design/FOLLOWUPS.md`): test infrastructure is polish
  owned by v0.0.1, and functionality comes first.

## What is actually next

No task is pre-selected — the operator picks. The live candidates, functionality
before polish:

- **F-150 — the on-device wallet-descriptor builder.** Operator's words: *needs
  major attention*. It dead-ends (blank screen after Next), assumes the user has
  exactly one key, and offers none of taproot, `after()`, `older()`, or
  `hash()`. Filed as needing its own brainstorm. This is the largest known
  functionality gap on the device.
- **F-152 — selecting "from payload" when a payload is PRESENT BUT NOT LOADED
  should launch the loader.** Agreed as a feature; deliberately not implemented
  freehand. Needs a spec §3.1 state and one plan stage.
- **The journey documents themselves**, which are what this phase is named for.
  Two exist in `design/journeys/`; the programs none of them covers are the gap.

Before proposing an approach, confirm the current state yourself: both repos
clean and pushed, and whether anything on fork `main` is unflashed — behaviour
that has never run on the machine is not verified, however green the suite is.
