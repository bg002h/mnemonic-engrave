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

### 8.1 — the walk-away wipe — **PASSES**

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
- [ ] countdown's first number (expect 30) — *pending*
- [ ] the reading when the screen changed (expect 3:30) — *pending*

### 8.1a — the §7.1 in-situ KDF rate — **MEASURED, and it disagrees with the spec**

| | |
| --- | --- |
| iterations | 300,000 |
| elapsed | **40.2 s ± 1.0** (stopwatch; the film-affected first attempt gave 36 s ± 8 and could not discriminate) |
| **in-situ rate** | **7,463 it/s** (7,282–7,653) |
| §7.1 records | 9,715 it/s → predicts 30.9 s |
| verdict | **9,715 is OUTSIDE the error bar.** 23% slower in situ. |

**Explained, not merely observed.** The KDF runs `kdfStepIterations = 500` per
frame, so 300,000 is 600 frames at **67.0 ms** each. Raw PBKDF2 at 9,715 it/s
accounts for 51.5 ms, leaving **~15.5 ms per frame of GUI overhead** — the
progress screen, event handling, `WakeupAt`. §7.1's figure is a **raw bench**
number; the in-situ rate carries ~30% on top. This is the last open item in
§12.1, and closing it is what Task 8.1a existed for.

**Consequences:**

1. **§7.1's "300,000 = 30.9 s on device" is falsified in situ** — it is ~40 s.
   The claim appears in the spec, the plan, and the CLI's help text.
2. **F-93's severity was UNDERSTATED.** The park threshold at the real rate is
   180 × 7,463 = **1,343,284** iterations, so **34.6%** of the legal 100k–2M
   range would have hung the pre-Task-5 firmware — not the 13.2% recorded. Task 5
   fixes it either way, but the affected range is 2.6× wider than believed.
3. **`me seal`'s default changed 300,000 → 230,000** (`8106f56`) to restore
   §7.1's ~30 s intent. Security cost is negligible: the passphrase is a
   *generated* 128-bit BIP-39 mnemonic, and §7.1's own reasoning for the KDF's
   ~20 bits is about *human-chosen* passphrases worth 25–35 bits.

**Caveat: one timed sample.** The remaining unlock can corroborate.

**SPEC DRIFT, open:** §7.1 still records 9,715 it/s and a 300,000 default. It is
GREEN and normative, so the amendment awaits operator approval; until then the
CLI and the spec disagree by exactly this measurement.

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
