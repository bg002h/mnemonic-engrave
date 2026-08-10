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

## Not yet observed

- [ ] **8.1** — the walk-away wipe (incl. **8.1a**, the §7.1 KDF measurement)
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
