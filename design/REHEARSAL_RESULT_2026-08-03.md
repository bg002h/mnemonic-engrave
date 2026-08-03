# Pico 2 boot-key rehearsal — EXECUTED AND PASSED, 2026-08-03

Board: Raspberry Pi Pico 2, CHIPID `bf2ff20ad60f66d3`. **Consumed as designed** —
2 of 4 boot-key slots burned, secure boot permanently enabled.

## Result: all phases 0 → 6 passed on real silicon

| Phase | What it proved | Result |
|---|---|---|
| 0 | stock board, all 4 slots empty, page permissions writable, CHIPID pinned | PASS |
| 1 | seal: `otp load` 16 rows + `otp set -s` on RBIT-3 and CRIT rows | PASS |
| 2 | sealed state matches a retail SeedHammer II | PASS |
| 3 | **negative control** — your-key-signed AND unsigned images both REJECTED | PASS |
| 4 | burn your key to slot 1; `KEY_VALID` 0x1 → 0x3 | PASS |
| 5 | **positive control** — the SAME file from 3a now ACCEPTED | PASS |
| 5b | the real 2.4 MB fork firmware, signed, ACCEPTED | PASS |
| 6 | **fallback control** — factory-key-signed image STILL boots | PASS |

## What is now proven, not inferred

- **The A/B holds.** `blinky-mykey.signed.uf2` was rejected in 3a and accepted in
  5a, byte-for-byte identical, with exactly one OTP write between them. A blink
  means signature acceptance, because rejection was first shown to be possible.
- **The OTP write path works on real silicon** — `otp load` of a 16-row ECC slot,
  and `otp set -s` on both a 3-way-redundant row (`BOOT_FLAGS1`) and a crit row
  (`CRIT1`). Open since round 5; this was the largest untested surface.
- **The 16-row readback gate works**, catching what it reads and matching an
  independently openssl-derived hash both times (slot 0 `cd027c2c…`, slot 1
  `17644fb6…`).
- **The signing chain works end to end, on both branches** — the seal path (fresh
  blinky, no SIGNATURE section) in 3a/6, and the already-present path (real
  `build-firmware` output) in 5b.
- **Acceptance generalises past the toy image.** F11(d) closed: the real 2.4 MB
  firmware was accepted, verdict taken from picotool rather than by eye.
- **The recovery path is real.** Dual trust works: after adding a second key, a
  factory-key-signed image still boots. This is what runbook step 7 rests on.
- **No high-`s` problem.** Every signature was accepted first try, consistent
  with the round-5 finding that the RP2350 bootrom performs no canonicality
  check.

## Bugs this exercise found (all in our code, none in the plan's design)

Four surfaced on first hardware contact, before any OTP was burned:

1. `check_page_locks` demanded all-zero page-lock rows. Every RP2350 — the
   SeedHammer *and* a factory-fresh Pico — reads `0x040404`. It would have
   declared the procedure impossible on a working machine.
2. picotool prints nothing (exit 0) for a single-row query whose first row is
   `0x001`; `CHIPID1` silently vanished. Fixed by batching all rows into one call
   with a parsed-count assertion.
3. `die` inside `$(...)` exits only the subshell, so the fail-closed readers were
   not actually closed. Readers now set globals.
4. RP2350 detection grepped `picotool info` for the chip name, which only appears
   when flash holds a program — it rejected a brand-new Pico. Now probes CHIPID.

A fifth (`ask_blink`'s verdict could be silently skipped) was found by the
round-6 fable review and fixed before phase 3 was trusted.

## Still unproven for the SeedHammer II

- That the SH2's own write path behaves identically. Same silicon, same starting
  state (verified by `--sh2-precheck`), but not the same physical device.
- That the fork firmware **runs** on the SH2. No Pico can show that.
- The `(UNLOCKED)` display change after a second key becomes valid.
- Behaviour under an interrupted `otp load` — still the one unrecoverable window.
