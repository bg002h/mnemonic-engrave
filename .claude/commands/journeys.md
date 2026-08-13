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

## THE TASK: write the Load Payload journey, folding the phase's open findings in

Decided 2026-08-12 with the operator ("fold as you write"). Not a menu — this is
the work.

**Write `design/journeys/SeedHammer-II-load-payload-journey.pdf`**, following the
convention the two existing documents set: `transcript_payload.sh` +
`build_pdf_payload.py` + `inputs-payload/` committed, `out/` and `shots/` not.
**Nothing illustrative** — every CLI block is real stdout+stderr with its true
exit code, every screenshot the emulator's own 480x320 framebuffer via
`shot_server.py`. Failures are RESULTS: the pathological transcript reports the
three places the toolchain refuses, and this one must do the same.

The prerequisite is DONE — `cmd/emu` carries a systemwide test payload (fork
`20e99a6`), so the whole flow is walkable in a browser. Verified: boot offers
LOAD/SKIP, and the device shows `55ad b800 6ec6 a066 94f3 6a0e 900a c8d5`,
byte-for-byte what `me sysw show` prints on the host.

**Fold these in while writing**, rather than as a later sweep — they are
documentation defects in the EXISTING journeys, and this is the phase that owns
them (all twelve are `#mnemonic`, owning phase *operator journeys*, and per the
burndown rule they are not deferrable past it):

| item | what is wrong |
| --- | --- |
| F-131 | the engraving checklist's recovery rule is false in BOTH directions |
| F-132 | the hashlock preimage is required to spend, absent from the backup, unmentioned by it |
| F-133 | the relative tiers are INVERTED — the weakest key-set matures ~90 days before the stronger |
| F-134 | plate count ranges 26 -> 58 on an md1-form flag nobody is told about |
| F-130 | restored xpubs lose depth/parent/child, so the descriptor and its checksum change |
| F-127 F-128 F-136 F-137 F-139 F-140 | encoder/stub/cost-comparison defects the earlier runs surfaced |
| F-147 | `cmd && echo OK` prints nothing on failure — the habit, not a doc defect |

Two corrections to make while in these files:

- **The spec's delivery step is no longer unrehearsed.** §"How the image reaches
  `0x10D00000`" says the `picotool load --verify -t bin -o 0x10D00000` line is
  transcribed rather than performed, and forbids it reaching operator
  documentation until rehearsed. Payloads are on the machine, so it has been.
  Correct the paragraph, then the journey may name the command. Note the actor:
  this is the HOST writing through the bootrom in BOOTSEL — the firmware never
  writes flash (D10), and nothing here changes that.
- **`me sysw pack` requires lowercase HEX bodies** for `text:`/`pass:` records
  (spec §5.3.1); a non-hex body is ClassUnknown and refused with exit 4. It is
  the first thing anyone will get wrong, so it belongs in the document.

## Other candidates, NOT selected

These are NOT the next task — the Load Payload journey above is. Listed so a
later phase can pick them up without re-deriving their state:

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
