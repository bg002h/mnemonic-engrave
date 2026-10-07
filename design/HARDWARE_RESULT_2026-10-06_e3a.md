# E3a bench rehearsal — PASSED, 2026-10-06 (plan `IMPLEMENTATION_PLAN_e3a_refugium_otp.md` §7)

**Result: R0-R10 pass on a Pico 2 (CHIPID `66D3D60FF20ABF2F`); R5 and R7 refused as designed; R8
confirms F10 on silicon. `check --profile rehearsal … --disable-otp-boot 1 --key-invalid c` prints
`RESULT: REHEARSAL PROFILE PASS — not a retail check` (exit 0). R11 (optional) not run.** No SeedHammer
was attached or written to.

Bench: Brian at 13764k, 2026-10-06 18:03-18:33 local (2026-10-07 01:03-01:33 UTC). Master `864fe6c`.
picotool 2.3.1 from `nix develop .#otp`, store path
`/nix/store/ri9krl08293dxl0w36765hcy45da1klp-picotool-2.3.1` (the patched path in
`design/PICOTOOL_PIN.md`). Transcripts: `rehearsal-work/e3a-66D3D60FF20ABF2F/` on 13764k (gitignored);
the replay fixtures are committed under `scripts/test/fixtures/transcripts/`.

## Deviation: the board was already sealed (Brian's ruling)

§7 assumes a fresh Pico 2. The board attached is the 2026-08-03 rehearsal board
(`design/REHEARSAL_RESULT_2026-08-03.md`; HARDWARE_INVENTORY row `0x66d3d60ff20abf2f`), already through
R's phases 0-6: SECURE_BOOT_ENABLE 1, slot 0 = August `factory-key.pem` (`cd027c2c…`), slot 1 = August
`my-key.pem` (`17644fb6…`), KEY_VALID 0x3, slots 2-3 empty. Not a SeedHammer (slot 0 ≠
`SH_SIGNKEY_HASH`), the only device attached. Brian chose to reuse it, then picked option A in
writing: "go A on 66D3D60FF20ABF2F, revoking slots 2 and 3". Consequences:

| step | on this board |
|---|---|
| R0 | as written |
| R1 | adapted: the capture was taken before any write, but every cell already held its **post-R2** value (CRIT1 0x000001 ×8, BOOT_FLAGS1 0x000003 ×3); every other R1 cell matched (CRIT0 0 ×8, BOOT_FLAGS0 0 ×3, FLASH_DEVINFO 0, slots 2-3 zero, LOCK1 0x040404, LOCK0 0, copies equal) |
| R2 | not run: R's phases 0-4, including the phase-3 reject / phase-5 accept A/B, passed on this board 2026-08-03. The August keys and CHIPID pin were reused from the archive |
| R3-R10 | as written |
| R11 | not run (optional) |

The board's two spare key slots are now permanently revoked; it still boots images signed by either
August key.

## Steps

| step | command / action | result |
|---|---|---|
| R0 | `picotool version -s` | 2.3.1, patched store path |
| R0 | R's old e2e (`scripts/test/run-e2e.sh`) | 48 passed, 0 failed |
| R0 | fresh blinky (R's tinygo command) → `sign-firmware.sh` | step 7: `signature: verified` and `load map entry 0: Clear 0x20000000->0x20082000` |
| R0 | `seal-clear-test.sh "$(command -v picotool)"` | 11 passed, 0 failed |
| R1 | `capture` | exit 0; capture sha256 `4054cee3…ba9d` |
| R3 | `check … --disable-otp-boot 0 --key-invalid 0` | REHEARSAL PROFILE PASS, exit 0 |
| R4 | `erase-range --probe-only --execute` | exit 2: `REFUSED: the marker reappears at +4 MiB` (the positive control) |
| R4 | `erase-range --execute` | exit 0: 0x10000000-0x10400000 erased, every byte 0xFF |
| R4 | R phase 5, `ACCEPT_BLINKY_ONLY=1` | blinky blinked (Brian); 5b also ran (below) |
| R5 | `inject-copy --case bf0-copy3 --execute` | exit 0; BOOT_FLAGS0 0x000000/0x000000/0x002000 |
| R5 | `check` (R3's flags) | exit 2: `FAIL  BOOT_FLAGS0  copies differ: 0x000000/0x000000/0x002000 (copy 0 from -c 1)` |
| R5 | `capture --out r5-capture.json` | exit 0; `0x048_named.txt` holds `RAW_VALUE=0x000000;0x000000;0x002000 (WARNING - REDUNDANT ROWS AREN'T EQUAL)` |
| R6 | `disable-otp-boot --execute` | exit 0: `healing an unequal copy: copies 0x000000/0x000000/0x002000 -> 0x002000 in every copy`; post-write check PASS |
| R6 | R phase 5 | blinky blinked; 5b accepted (F5 holds) |
| R7 | `inject-copy --case bf1-copy0 --execute` | exit 0; BOOT_FLAGS1 0x000803/0x000003/0x000003 (the `-c 1` verification agreed: no exit 3) |
| R7 | `check` (`--disable-otp-boot 1 --key-invalid 0`) | exit 2: `FAIL  BOOT_FLAGS1  copies differ: 0x000803/0x000003/0x000003 (copy 0 from -c 1)` (also `FAIL SLOT1 KEY_VALID unknown`, same cause) |
| R7 | `capture --out r7-capture.json` | exit 0; `0x04b_named.txt` holds RAW_VALUE |
| R8 | F10 read (below) | F10 holds |
| R9 | `invalidate-spare-keys --execute` | exit 0: `healing an unequal copy: copies 0x000803/0x000003/0x000003 -> 0x000c03 in every copy`; post-write check PASS |
| R9 | R phase 5 | blinky blinked; 5b accepted |
| R10 | `check … --disable-otp-boot 1 --key-invalid c` | exit 0, all 15 rows PASS, CANNOT PROVE list printed, `RESULT: REHEARSAL PROFILE PASS — not a retail check` |

Every `refugium-otp.sh` call passed `--ser 66D3D60FF20ABF2F`. R's phase 5 takes no `--ser` (unchanged
R behaviour); one RP2350 was attached throughout.

Final OTP state (R10 capture, sha256 `afa63358…72ed`): BOOT_FLAGS0 0x002000 ×3, BOOT_FLAGS1 0x000c03
×3, CRIT1 0x000001 ×8, CRIT0 0 ×8, slots 2-3 zero, page locks unchanged.

## R8 — F10 on silicon

After R7's injection (copy 0 of BOOT_FLAGS1 = 0x000803):

```
$ picotool otp get -c 1 -n --ser 66D3D60FF20ABF2F 0x04b
ROW 0x004b: OTP_DATA_BOOT_FLAGS1 (RBIT-3)
    VALUE 0x000803
$ picotool otp get -n --ser 66D3D60FF20ABF2F BOOT_FLAGS1
ROW 0x004b: OTP_DATA_BOOT_FLAGS1 (RBIT-3)
    RAW_VALUE=0x000803;0x000003;0x000003 (WARNING - REDUNDANT ROWS AREN'T EQUAL)
    VALUE 0x000003
```

`-c 1` reads copy 0 raw (0x000803), the named read votes 0x000003, and RAW_VALUE[0] equals the `-c 1`
read. Both exit 0. Recorded in F-619.

## The seal-patch HOLD

R4's phase 5 booted `blinky-mykey.signed.uf2` (sha256 `bb50f9dd…7e9b`). It was built fresh with no
SIGNATURE section and sealed and signed by `sign-firmware.sh` under `.#otp`, i.e. by the patched 2.3.1
(`--clear`; step 1: "no SIGNATURE section; sealing … SIGNATURE section created"). This is the hardware
boot of a 2.3.1-sealed image that `design/PICOTOOL_PIN.md`'s HOLD waits on, repeated after R6 and R9.
This PR does not edit the HOLD; lifting it is a separate decision.

Phase 5b also found the fork's `seedhammerii-66d3121691ecf325b35e44285c1b2e7cf5250cce.uf2`, re-signed
it with my-key (`fw-mykey.signed.uf2`, sha256 `b6e2928e…2417`) and the bootrom accepted it three times.
The fork build had already sealed that image ("SIGNATURE section already present"), so 5b is evidence
for re-signing, not for 2.3.1 sealing.

## Notes (Minor, non-blocking)

- `inject-copy --case bf0-copy3` prints "copy 2 of BOOT_FLAGS0" (0-based) while the case name counts
  from 1. Cosmetic.
- R's phase 5 prints "phase-3 image is intact and signed by your key (true A/B)". On this board the
  image was re-signed today; the A/B was the 2026-08-03 run with a different file. The message's
  claim does not hold when R2 is skipped. Cosmetic for this run.
- The auto-mode classifier refused to let the controller pipe the typed `BURN … <CHIPID>` confirmations,
  so Brian ran every confirmed step in his own terminal and the controller ran the read-only ones. The
  confirmation gate worked as designed.
- After a 5b boot the fork firmware enumerates as `2e8a:000f` ("Pimoroni Pico Plus2" in `lsusb`), the
  same VID:PID as BOOTSEL; picotool correctly reports no board. Re-enter BOOTSEL by hand before the next
  step.
