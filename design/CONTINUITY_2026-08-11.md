# Continuity — 2026-08-11, after the release

Supersedes `CONTINUITY_2026-08-10.md`. Written for a fresh context that must
resume at **the post-release fable whole-Phase-2 review** and **post-merge polish
and hardening**. Carries only what cannot be re-derived from the repo.

## 1. Where everything is

| thing | state |
| --- | --- |
| `mnemonic-engrave` `master` | `d0c0a9d`, pushed |
| **released** | **`v0.5.0`**, public, 7 signed assets + `SHA256SUMS.minisig` |
| fork `bg002h/seedhammer` `main` | `93ee004`, pushed — `b2b` merged in (29 commits) |
| fork tag | `fork-v0.0.0-g93ee004`, pushed |
| hardware last flashed | `v0.0.0-g747cf48` (pre-merge), validated 2026-08-10 |

`b2b`, `b2b-residency`, `b2b-f106`, `b2b-int`, `f113-ms1-engraveable`,
`emu-test-payload`, `emu-toolpath` are all merged and can be deleted.

**The release ships with the wipe incomplete, by explicit operator decision.**
That is stated in `README.md` (a section above "What it does"), in
`SPEC_encrypted_payload_delivery.md` §2.2 item 16, and in the fork tag message.

## 2. The fable review — its mandate, decided 2026-08-10

**Order was deliberately merge → tag → release → fable.** The earlier "do not tag
with F-109 open" ruling was superseded; the strike-through is preserved in
`FOLLOWUPS.md` rather than deleted.

**Scope, as ruled:**

- the **whole Phase 2 diff** — B2a-i + B2a-ii + B2b, read by one reviewer at
  once. That is the thing no single phase's context could do, and it is the
  reason the review exists.
- **every deferred follow-up**, with a mandate to **suggest closures**.

**Suggest, not perform.** The reviewer proposes which deferred items its
whole-diff reading shows to be already satisfied, subsumed, or not defects; the
operator decides. This matters: several items in `FOLLOWUPS.md` turned out to be
*unrecorded rather than undone*, and a whole-diff reading is exactly the vantage
point that can tell those apart — but a reviewer closing its own findings is
marking its own homework.

**One advantage of this ordering worth using:** fable reads the code *after* it
has shipped and run on hardware, so the hardware evidence is available to it —
`HARDWARE_RESULT_2026-08-10c_b2b_gate.md`, the F-106 readings, the abort→resume
observation, `TOOLPATH_EQUIVALENCE_2026-08-10.md`.

**Fable's own prior involvement is on the record and should be in its brief:**
it designed §10.2.4's idle wipe in a single consult
(`design/CONSULT_b2b_idle_timer_design.md`, 2026-08-09) that the B2b plan was
written against, and it reviewed the spec's cryptographic core
(`agent-reports/encrypted-payload-spec-v2-R0-round1-fable.md`). The idle timer
shipped two defects: F-106 (found only on hardware, fixed) and F-103 (open). A
brief that does not say so invites the same blind spot.

## 3. The one question the review process never asked

**"What are all the copies?"**

The wipe was specified against the *record*. Every copy made downstream of it was
found afterwards, one at a time, by review and measurement — F-88, F-90, F-94,
F-104, and the ~35 KB in F-109 that still has no name. Architect loops reached
0C/0I on every design document without anyone enumerating the inventory.

That is the generalisable failure, and it is now a rule in §2.2 item 16: **a wipe
requirement must carry an inventory of what must be wiped.** Put this question in
the fable brief explicitly — it is the highest-value thing a whole-diff reading
can answer.

## 4. Post-merge polish and hardening — 13 items, three groups

**Seed residue (inside the payload flow, all traced and confirmed binding):**
F-88, F-90, F-94, F-104, and F-87 (one missing test on `unlockEngraveMnemonic`,
the same path F-88 covers — do them together).

**The wipe's own reliability:**
- **F-103** — `gui/run_flow.go:251` refreshes `a.idle.start` on raw
  `len(evts) > 0`, with no requirement that an event resolve to *effective*
  input. Spurious touch readings keep the machine permanently non-idle; the
  warning branch is nested inside the idle branch, so there is no countdown, no
  wipe, and no indication. **Host-testable and tested:** 100,000 spurious polls
  over ~1000 s → zero wipes; control → 3:00. Smallest fix: refresh only on input
  that resolves to a state change.
- **F-109** — ~35 KB across ~81 reachable objects, unidentified. Assigned to
  the fable review to identify, not merely observe.

**Motion:** F-114. **Severity undetermined and worth settling first.**
`SafePointer.Resume` synthesises its approach line from `bezier.Point{}`
(`engrave/engrave.go:1664`), so the head tracks toward the origin before running
to the safe point. Needle up, right destination — but `stepper.Driver.fill()`
clamps to one step per tick, so the *unplanned* move from the head's real
position to that line's start may run at max rate with no acceleration ramp.
If so it is plate integrity, not efficiency. **Checkable in the simulator**
(`window.shToolpath`) on a plate placed far from the origin — vertex spacing
right after the resume gives it away.

**Font and rendering:** F-78, F-86, F-95, F-119. F-78 (`·` has no glyph on four
shipped screens) and F-86 (`%` renders as zero pixels during the ~31 s
derivation) are **visible on shipped screens today** — most likely day-one user
reports.

**Post-release features:** F-117 (seed plate QR 33→37, closes the 91–93 band by
engraving it; the text path already runs at 37 in production), F-118 (long codes
need QR v6, past `bitmapForQRStatic`'s 21/25/29/33/37 table, whose author wrote
"raise both together or not at all").

## 5. Process facts that cost real rounds, and will again

**Empty output is not evidence of absence.** Four instances in two days: a
`go list` that failed and printed nothing, read as "dependency absent"; a
`_arm_test.go` suffix that hid a file in `IgnoredGoFiles` with the suite green
and zero tests run; a `sed` that never applied, reported as a surviving mutant;
and `plan-cite-gate.sh` resolving a citation to a stale `target/` artefact. Pair
every negative check with a positive control. Saved as the
`empty-output-is-not-absence` memory.

**Classify follow-ups by tracing code, never by their prose.** This produced two
opposite errors in one day: F-112 was called a split when it was a close, and
F-88/F-90/F-104 were called accepted when all nine sub-items bind. The §2.2 item
12 ruling is about where code *runs*; only the call graph says that.

**Describing your own edit is not reading it.** Three review findings in one loop
were claims about my own work — one asserted fixed that wasn't, one asserted true
that wasn't, one asserted done that wasn't. And the `EngraveText` mechanics I
wrote into the spec were fabricated from reading `ConstantQR`'s cap and
attributing it to a path that calls `engrave.QR`.

**A no-op mutant is indistinguishable from a surviving one.** Verify the diff
applied before testing.

## 6. Instruments now available that did not exist before

- **`scripts/release-scan-firmware.sh`** — builds the real firmware and searches
  ELF + reassembled UF2 for the emulator's test payload, with a positive control.
  Run against `93ee004`: CLEAN.
- **`window.shToolpath`** in `cmd/emu` — decodes the driver's step stream into
  actual head motion. `reset/summary/path/svg`. This is what makes F-114
  checkable without a plate, and it is stronger than `cmd/plateview`, which
  renders the *plan* rather than what the driver emits.
- **`sh-sim`** — runs any firmware ref in a browser. Readings 1–3 of a hardware
  gate run there; only steel needs the machine.
- **The emulator carries a real sealed payload and its passphrase, deliberately**
  (operator ruling). Confinement is enforced by two mutation-tested guards plus
  the firmware scan — do not "fix" it by removing it.
