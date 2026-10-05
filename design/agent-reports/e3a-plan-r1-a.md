# E3a plan R0 round 2, lens A (correctness and external facts)

Reviewed: `design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md` draft 2 at mnemonic-engrave `33d4c11`.
Reviewer: independent opus agent, 2026-10-05. Proportional re-review: did the fold fix each lens-A
finding, are draft 2's new facts true, does the §7 bench walk hold, and did the fold add a defect.
Read-only apart from this file.

**Counts: 0 Critical / 1 Important / 8 Minor / 1 Nit**

## Sources checked (primary)

| key | source | revision |
|---|---|---|
| PT | `raspberrypi/picotool` `main.cpp`, `otp_header_parser/otp_header_parse.cpp` | tag 2.3.1 (fresh shallow clone) |
| A4 | picotool 2.2.0-a4 `main.cpp` (from round 1's recon copy) | — |
| OD | `raspberrypi/pico-sdk` `otp_data.h` | tag 2.3.1 = `079c6f39` |
| BIN | the real picotool 2.3.1, `nix build 'git+https://github.com/NixOS/nixpkgs?ref=nixos-unstable&shallow=1&rev=a7868a72…#picotool'` → `/nix/store/5cxc3bjs…-picotool-2.3.1`, run in this container (no `/dev/bus/usb` at all) | — |
| R | `scripts/pico2-bootkey-rehearsal.sh` | `33d4c11` |

## Round-1 finding → fixed?

| finding | fixed? | evidence (draft 2 text) |
|---|---|---|
| A-C1 argv order | **Yes** | F14 states the declared order for get and set (BIN/PT confirm: `otp get` group `-c -r -e -n -i`, then device selection, then `-z`/selectors; `otp set` selectors, value, then device selection, PT `main.cpp:1370-1495`); §3.0 "One argv builder … emits every picotool call in F14's canonical order … No call site writes picotool argv by hand"; §6.1 "It enforces F14's argument order … exits 99"; §6.3 "builder emits `-n` before `-c` → fake exit 99"; §6.4 probe; §5 F-619 correction "an argument-order artefact". Probe text-matching caveat: A2-M1. |
| A-C2 heal vs equality | **Yes, with a new defect in the rule** | §3.2 "`check` takes the whole expected state explicitly"; §3.3 "Heal rule (the exact F11 safety condition …)"; "The read-only `check` never heals". R5→R6 and R7→R9 now go through the heal path and work (walked below). The rule's bound is too loose for multi-bit fields: A2-I1. |
| A-I1 phase 3 skipped | **Yes** | R2 "R's phases 0, 1, 2, 3 (`--execute`), 4 — the A/B proof needs phase 3 (R:1099)"; R4 "`ACCEPT_BLINKY_ONLY=1` … Re-enter BOOTSEL by hand after every phase 5". |
| A-I2 no shell; picosign | **Yes** (one sequencing nit, A2-M8) | §2 "`devShells.<system>.otp` with picotool, openssl, jq, python3, xxd, **tinygo and go**"; "R is **not** gated to 2.3.1: it accepts `2.2.0-a4` or `2.3.1`"; R0 "run `sign-firmware.sh` on the blinky and require `picotool info -a` → `signature: verified`". |
| A-I3 gates cannot run | **Yes** | §6.4 "R's existing e2e (`run-e2e.sh`, old fake with its version line updated to 2.3.1) is not in CI … runs in `.#otp` at §8 step 1 and at the bench (R0)"; old case 9 dropped (§10). |
| A-I4 R10 vs R11 | **Yes** | R10 is the PASS; R11 "Optional, after R10 … A fresh key generated outside `rehearsal-work/`; `--make-otp-json --slot 2` with `ALLOW_UNCHECKED_KEY=1` … Then `check` must refuse naming slot 2. Recorded as evidence, not proof". `ALLOW_UNCHECKED_KEY` short-circuits `reject_rehearsal_key` (R:635-637); `--slot 2` passes `require_spare_slot`. |
| A-I5 white-label strings | **Yes** | §3.2 row "USB_WHITE_LABEL_ADDR, its 16-row table, every valid STRDEF's string rows \| raw 24-bit \| equal the recorded entry"; F7 extended; §3.5 capture "the white-label table and every valid string's rows". |
| A-M1 rule (d) | **Mostly** | Copy rule (a) "the raw reads of the unnamed copies and the `-c 1` read of the first row are all equal", applied to BOOT_FLAGS0/1, USB_BOOT_FLAGS, CRIT1. CRIT0 is listed ×8 but only "`-c 1` named read of 0x040" is named: A2-M2. |
| A-M2 F10 reading | **Yes** | R8 after R7's copy-0 injection: "`otp get -c 1 -n --ser S 0x04b` must show `0x000803` while the named vote shows `0x000003`". PT `main.cpp:9264-9277`: with `-c 1`, redundancy = 1 and VALUE is copy 0's raw. |
| A-M3 CRIT whole | **Yes** | §3.2 "values equal the recorded retail entry (SECURE_BOOT_ENABLE 1, DEBUG_DISABLE clear)"; rehearsal "CRIT1 = 0x000001 in all 8, CRIT0 = 0". |
| A-M4 stages | **Yes** | stages removed; `--slot1 empty\|key\|valid` joint table covers the spec's step-1 state (`key`, KEY_VALID 0x1); writes derive their own pre-state (§3.3). |
| A-M5 `--ser` on check | **Yes** | §3.1 "`--ser` is required everywhere"; §3.0 "R uses the builder too, with an empty serial". |
| A-M6 flake URL | **Yes** | §2 "`git+https://github.com/NixOS/nixpkgs?ref=nixos-unstable&shallow=1`". Re-fetched here: works. |
| A-M7 one-row write | **Yes** | F16; §3.6 uses `-s` and `-c 1`. Verified, see "new facts". |
| A-M8 `.bin`, top sector | **Yes** | F13 "`save` writes raw only to a `.bin` name"; §3.4 "(16 MB, which includes the top sector, so the top sector is erased: UI spec step 2's question answered)". |
| A-M9 ROLLBACK_REQUIRED | **Yes** | §3.2 "bit 11 (ROLLBACK_REQUIRED) either value". |
| A-M10 flipping note | **Yes** | G1 and §6.1 "the flipping note". |
| A-N1 version | **Yes** | F17; BIN `version -s` → `2.3.1`. |
| A-N2 save default | **Yes** | F13. |
| A-N3 serial vs white-label | **Yes** | F15 "The USB serial is the CHIPID unless white-label entry 6 is valid". |

## Draft 2's new facts, checked

| claim | verdict | evidence |
|---|---|---|
| F14 canonical `otp get` / `otp set` order | TRUE | PT `main.cpp:1370-1495`; same declaration order in 2.2.0-a4 (A4), so R's builder use is safe on both versions. BIN: `otp get -c 1 -r -n --ser Z 0x04b`, `otp get -r -n --ser Z 0x04a`, `otp get -n --ser Z BOOT_FLAGS0`, `otp set -s 0x04a 0x002000 --ser Z`, `otp set -c 1 -s 0x04b 0x000800 --ser Z`, `otp set -s BOOT_FLAGS0.DISABLE_OTP_BOOT 0x1 --ser Z`, `otp set -s BOOT_FLAGS1.KEY_INVALID 0xc --ser Z`, `otp load f.json --ser Z`, `erase -r 0x10000000 0x10400000 --ser Z`, `save -r A B f.bin --ser Z`, `load -v m.bin -o 0x10000000 --ser Z`, `info [-a] --ser Z` all reach device selection with the serial (exit 249, serial named). Misordered `otp get -n -c 1 --ser Z 0x048` → 249 with **no** serial named (detected). `otp get --ser Z -c 1 0x048` → serial named but `-c` lost; the canonical order never emits it, and because `--ser` follows `-c/-r/-e/-n` in canonical order, a misparse of any earlier option swallows `--ser` and the probe goes red. |
| F15 miss → 249 "…found with serial number X." | TRUE, text wraps | PT `main.cpp:4455`. BIN with stdout not a tty prints `…were found with serial number` / newline / `ZZPROBE00000000.` (A2-M1). Device noun differs per command: "RP2350" for `otp *`, "RP-series" for `erase/save/load/info`. |
| No-USB CI runner behaves the same | TRUE | This container has no `/dev/bus/usb`; `libusb_init` succeeds and every shape exits 249 with the serial message. A failing `libusb_init` would exit with "Failed to initialise libUSB" (PT `main.cpp:10231-10233`), which turns the probe red, i.e. fails safe. |
| F16 `otp set -s 0x04a 0x002000` writes one raw row, no ECC | TRUE | `otp list -n` (BIN) has no entry at 0x039-0x03f, 0x041-0x047, 0x049/0x04a, 0x04c/0x04d, 0x05a/0x05b (the `_R1.._R7` registers are folded into the base register by `otp_header_parse.cpp:250-268`). With no register, `settings.otp.redundancy` stays -1 and `bEcc = ecc && !raw` = 0 (PT `main.cpp:8990-8992, 9716-9732`): one raw 24-bit write of `0x002000 \| old`. **G2 is therefore answered** (A2-N1). |
| F16 `otp set -c 1 -s 0x04b <v>` writes copy 0 only | TRUE | `-c 1` sets `settings.otp.redundancy = 1`; the register only fills it when `< 0` (PT `main.cpp:9656`); `wRowCount = 1` (9725). `-s` ORs copy 0 (9709-9712). |
| F16 0x040 writes all copies | TRUE | CRIT1 redundancy 8, `crit` (`otp_header_parse.cpp:262-267`). |
| F17 `version -s` | TRUE | BIN prints `2.3.1`. |
| `otp list` fingerprint | TRUE | BIN 2.3.1: `OTP_DATA_MAC0` present, `EXEC2` absent in `otp list -n`; 2.2.0 name list has EXEC2 and no MAC0. |
| `-c 1` on a named read returns copy 0 | TRUE | PT `main.cpp:9264-9277`: vote loop over `redundancy` = 1 rows; no RAW_VALUE/WARNING line. |
| alias probe: `load` a raw `.bin` at 0x10000000 with `--ser` | TRUE (spell `-o`) | PT `main.cpp:1108-1113, 5438`: `-o` sets `offset_set`, which skips the UF2-target-partition query. Aliasing of +4 MiB on a 4 MB part is already measured on this repo's Pico (HARDWARE_INVENTORY: `0x10E00000` aliases to `0x10200000`). |
| `save -r` to `.bin` is raw | TRUE | settled F13. |
| Rehearsal page locks "same" (LOCK1 0x040404) | TRUE | REHEARSAL_RESULT_2026-08-03 l.46: "a factory-fresh Pico — reads `0x040404`"; FOLLOWUPS: RP2350 factory default. |
| CRIT0 = 0 on a stock Pico 2 | assumption, not measured | OD:219-238: CRIT0 holds only ARM_DISABLE/RISCV_DISABLE (mask 0x3), so 0 is expected on a stock part, but no capture of the bench Pico exists. See A2-M3. |

## §7 walk (R0-R11), against R and §3.2/§3.3/§3.6

- **R2 → R3.** Phase 1 loads a bootkey-only JSON (R:466-472 strips crit1/boot_flags1), then `otp set -s BOOT_FLAGS1.KEY_VALID 0x1` (3 copies) and `otp set -s CRIT1.SECURE_BOOT_ENABLE 0x1` (CRIT1 is redundancy 8, so 8 copies of `old | 1`). Phase 4 adds `KEY_VALID 0x2`. On a stock part (all zero) that gives CRIT1 = 0x000001 ×8 and BOOT_FLAGS1 = 0x000003 ×3, bits 16-19 zero; R never writes BOOT_FLAGS0, USB_BOOT_FLAGS, FLASH_DEVINFO or page locks. R3 PASS follows **if** the stock values are zero (A2-M3).
- **R4.** `erase-range` (0x10000000-0x10400000), then phase 5: `flash_image` is `picotool load --verify <uf2>` + `reboot` (R:501-512), which rewrites the sectors it needs; KEY_VALID 3 (R:1097) and the phase-3 image (R:1099-1113) still hold. Works.
- **R5.** inject `bf0-copy3`: pre (copies equal, bit 13 clear) holds; `0x04a` becomes `old | 0x2000`. `check` refuses on copy rule (a). Correct.
- **R6.** Copies X, X, X|0x2000. Heal: each copy ⊆ copy0|0x2000, differ only in bit 13. The write is `copy0 | 0x2000` to all three; row 0x04a already equals it, so the boot ROM accepts it. Post PASS. Phase 5 needs BOOTSEL re-entry, which R4 states.
- **R7.** BOOT_FLAGS1 = 0x000003 ×3, bit 11 clear: the precondition holds (the board must be re-entered into BOOTSEL after R6's phase 5, which R4's rule covers). Copy 0 → 0x000803; the others are untouched.
- **R8.** `-c 1` → `VALUE 0x000803`; the named read → `RAW_VALUE=0x000803;0x000003;0x000003 (WARNING …)`, `VALUE 0x000003`. Follows from source.
- **R9.** Write value = `(0xc<<8) | (copy0 & ~0xf00) | copy0` = 0xC03; every copy ⊆ 0xC03; the boot ROM sets bit 10 on row 0x04b and bits 10-11 on the others. Post PASS; phase 5 KEY_VALID check still 3.
- **R10.** PASS. **R11.** Commands are executable as written. After it, KEY_VALID 0x7 and slot 2 non-zero, so `check` refuses on slot 2 and KEY_VALID.

The walk is sound. The two open points are the unrecorded stock values (A2-M3) and R0's blinky (A2-M8).

---

## Important

### A2-I1 — the heal rule admits a copy-0 bit inside the target field that the target value excludes, so `invalidate-spare-keys` can burn KEY_INVALID 0xD/0xE/0xF into all three copies (§3.3 "Heal rule", §6.2 case 5, §6.3)

**What is wrong.** §3.3 (1) bounds every copy by "the value the write will produce (copy 0 | target bits)". Copy 0 trivially satisfies this, so (1) never constrains copy 0. (2) "the copies differ only inside the target field's bits" and (3) "every bit outside the target field equals the expected value" do not constrain it either when the stray bit is **inside** the field but **outside** the target value. KEY_INVALID is a 4-bit field (bits 8-11, OD:729) and the target is 0xC (bits 10-11). Take a unit whose BOOT_FLAGS1 copy 0 = 0x000203 (KEY_INVALID bit 9 = slot 1, in copy 0 only) and copies 1-2 = 0x000003:

- vote KEY_INVALID = 0, copies unequal → the heal path;
- (1): copies 1-2 ⊆ 0x203|0xC00 = 0xE03 ✓, copy 0 ✓; (2): they differ only in bit 9, inside KEY_INVALID ✓; (3): bits outside the field are 0x003 = expected ✓;
- the write: `otp set -s BOOT_FLAGS1.KEY_INVALID 0xc` computes `0xC00 | (old & ~0xF00) | old` = **0xE03** from copy 0 (PT `main.cpp:9704-9712`) and writes it to all three rows. The boot ROM accepts each row, because each row's current value is a subset (F11).

Result: KEY_INVALID = 0xE in every copy, so **slot 1 (the fork key, Refugium's only boot key) is permanently invalid**. The post-check then exits 3 and tells the operator to re-run, which refuses. With bit 8 in copy 0 instead, slot 0 (SeedHammer's recovery key) is lost. This is an irreversible wrong write on a retail unit, admitted by a rule the plan calls "the exact F11 safety condition". That is accurate for "the write will not fail" but not for "the write produces the intended state". The read-only `check` would refuse the unit first, but write commands derive their own pre-state (§3.3) and do not require a prior `check`. DISABLE_OTP_BOOT is unaffected (1-bit field).

**Fix.** In §3.3, define **E** = the expected post-write value of the row: the expected bits outside the field, OR the field set to its post value (for KEY_INVALID, the bits found in the pre-state ∪ 0xC, which must itself be 0xC). Admit an unequal row only if **every** copy, copy 0 (`-c 1`) included, is ⊆ E **and** copy 0 | target = E (the value `otp set -s` will actually write). Drop "(copy 0 | target bits)" as the bound. In §6.2 case 5, add copy 0 holding KEY_INVALID bit 8, and separately bit 9, with every other copy clean → exit 2, state byte-identical. In §6.3, add the mutation "bound copies by copy0|target instead of E", killed by that case. R9 still heals under E (copies 0x803/0x003/0x003 ⊆ E = 0xC03).

---

## Minor

- **A2-M1 (§6.4 probe, F15, §6.1 fake text).** When stdout is not a tty, picotool word-wraps at 80 columns (PT `main.cpp:10178-10186`). BIN in CI-like conditions prints `…found with serial number` / `ZZPROBE00000000.` on **two lines**, so a `grep 'with serial number ZZPROBE00000000'` fails every shape. That is red rather than false-green, but the gate has never run. The same wrap affects `otp get` output: a CRIT1 `RAW_VALUE=` line with 8 copies (10 + 8×9 − 1 = 81 characters before the WARNING) wraps mid-list. **Fix:** the probe joins lines (`tr '\n' ' ' | tr -s ' '`) before matching, and matches the device-agnostic text (`with serial number ZZPROBE00000000.`), since the noun is "RP2350" for `otp` and "RP-series" for `erase/save/load/info`. G1 should include the 80-column wrap, and the fake should emit wrapped lines for long RAW_VALUE/WARNING output so the parsers are tested on them.
- **A2-M2 (§3.2 CRIT0 row).** CRIT0 is a named CRIT register with redundancy 8 (`otp list` "OTP_DATA_CRIT0 (CRIT)"). A bare read of 0x038 is the crit vote, not copy 0. The table names only "`-c 1` named read of 0x040", so copy rule (a) for CRIT0 lacks its first-copy read: an odd copy 0 of CRIT0 hides behind the vote (WARNING is the only guard, the F-619 class). **Fix:** add `-c 1` of 0x038, and a case-3 variant for CRIT0 copy 0.
- **A2-M3 (§7 R1 vs R3).** The rehearsal profile assumes stock CRIT0 = 0, CRIT1 = 0, BOOT_FLAGS0 = 0 (outside bit 11), BOOT_FLAGS1 bits 16-23 = 0, USB_BOOT_FLAGS copies equal and FLASH_DEVINFO_ENABLE = 0. None of these is recorded for a Pico 2. If one is false, R3 fails **after** R2's irreversible phases, and the rehearsal profile then cannot pass on that board. **Fix:** R1 checks these from the `capture` before R2 and stops if any differs (or records it and the profile is amended), not only the line-format diff.
- **A2-M4 (§3.1/§3.2 rehearsal slot-1 key).** Under `rehearsal`, `--slot1 key|valid` compares slot 1 with "R's `my-key.pem` hash", but the synopsis has only `--rehearsal-key` (slot 0's key). The source of the slot-1 key is unspecified: an implicit `rehearsal-work/my-key.pem`, or a flag. **Fix:** add `--rehearsal-slot1-key K` (required under `rehearsal`), or state the fixed path and print the hash used. Also state in R3 that `--rehearsal-key` is R's `factory-key.pem`.
- **A2-M5 (§6.3 "drop `--ser` from the builder").** The named killer, case 4 "`--ser` mismatch", is refused by the tool's own CHIPID-vs-`--ser` comparison (§3.1) whether or not the argv carries `--ser`. With `DEVICES=2`, the expected refusal also happens either way. The mutation can survive. **Fix:** the fake logs every argv, and a case asserts that every recorded device call carries `--ser <S>` in its canonical position. Make that assertion the mutation's killer.
- **A2-M6 (§3.2 BOOT_FLAGS1 retail).** The retail row fixes KEY_INVALID, KEY_VALID (joint table) and "bits 16-19". Bits 4-7, 12-15 and 20-23 are left unstated, and heal rule (3) needs an expected value for every bit outside the field. **Fix:** "every bit outside KEY_VALID and KEY_INVALID equals the recorded entry", as BOOT_FLAGS0's row already says.
- **A2-M7 (§3.4 alias probe).** Specify (a) the marker is random per run and not all-0xFF, so stale content at +4/8/12 MiB cannot match; (b) `load <marker>.bin -o 0x10000000 --ser S`, since `-o` sets `offset_set` and bypasses picotool's UF2-target-partition query (PT `main.cpp:5438`), plus `-v`; (c) a non-zero exit from the probe's `load`/`save` (the boot ROM refusing beyond CS0, F6) is "condemned", exit 2, the same as erase/save.
- **A2-M8 (§7 R0).** R0 runs `sign-firmware.sh` "on the blinky … with no board attached", but no blinky exists before R's `build_blinky` (R:443-449, reached in phase 0, which pins a board). **Fix:** at R0, build it with the same `tinygo build -target pico2 -opt 2` that `build_blinky` uses, into a scratch path (not `rehearsal-work/`, so phase 0 still builds its own). Or move the picosign check to just after R2's phase 0 and before phase 1, which is the first OTP write.

## Nit

- **A2-N1 (§1 G2).** G2 is answered from source. `otp set` takes `-r`. For an unnamed row the write is raw unless `-e` is given, because `bEcc = ecc && !raw` and there is no register to enable ECC (PT `main.cpp:8990-8992, 9716`). `-r` only disables ECC on a named ECC row. Move G2 into F16 and drop it from the open list.
