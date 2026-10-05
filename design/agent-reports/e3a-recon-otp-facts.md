# E3a recon: RP2350 OTP and boot-ROM facts, checked against source

*Research agent, 2026-10-05. Read-only, apart from writing this report. The datasheet host is blocked, so every
fact below comes from these primary sources:*

| key | source | revision |
| --- | --- | --- |
| **OD** | `raspberrypi/pico-sdk` `src/rp2350/hardware_regs/include/hardware/regs/otp_data.h` (generated from the same source as the datasheet) | master `079c6f39023649b154152db30f1d781e884879bc` (2026-09-04) |
| **OH** | same repo, `.../regs/otp.h` | same |
| **BR** | `raspberrypi/pico-bootrom-rp2350` (boot ROM source) | tag **A4** = `c6cdb1711f32c3e34faaebd58618a6d096dbd52e`; the load-bearing lines were re-checked on tag **A2** (`fd610445`). Semantics are identical; only line numbers differ. |
| **PT** | `raspberrypi/picotool` `main.cpp` | 2.3.1 / master `2041936441b48a3cc53ae3da9e805229fe8f4e18`; the same logic was cross-checked on tag **2.2.0** (`a7eb3988`), the installed version per the runbook. |

## Table

"Repo agrees?" is judged against RUNBOOK_custom_boot_key.md (RB), scripts/pico2-bootkey-rehearsal.sh (SC),
agent-reports/bootkey-round5-lens-external-facts.md (L2), HARDWARE_INVENTORY.md (HI) and, where it is relevant,
FOLLOWUPS.md (FU). "n/a" means the repo does not state the fact.

| # | fact | value | citation | repo agrees? |
| --- | --- | --- | --- | --- |
| 1a | CHIPID0..3 rows | 0x000..0x003. ECC, 16 bits each. CHIPID0 = bits 15:0 (LSW) of the 64-bit ID. | OD:17-18, 30, 33, 40, 50, 60 | Yes. HI's "word-reversed" script form `6f463e8d0bf609f5` is rows 0,1,2,3 printed in row order, and `0x09f50bf63e8d6f46` is CHIPID3:2:1:0. |
| 1b | CRIT1 + copies | 0x040, with R1..R7 at 0x041..0x047 (RBIT-8, raw 24-bit) | OD:313-317, 373-433 | Yes (SC:746, L2 §5) |
| 1c | BOOT_FLAGS0, R1, R2 | 0x048, 0x049, 0x04a (RBIT-3, raw 24-bit) | OD:442-449, 654, 664 | n/a (not referenced anywhere in the repo) |
| 1d | BOOT_FLAGS1, R1, R2 | 0x04b, 0x04c, 0x04d (RBIT-3, raw 24-bit) | OD:673-680, 762, 772 | Yes (SC:759, 846; L2 §5) |
| 1e | BOOTKEY0..3 | BOOTKEYn_0 = 0x080 / 0x090 / 0x0a0 / 0x0b0. 16 ECC rows each (last row 0x0bf). Page 2. | OD:1489-1493, 1650, 1810, 1970, 2120. BR `varm_blocks.c:235-237` static-asserts the 16-row spacing. | Yes (L2 §5) |
| 1f | KEY_VALID / KEY_INVALID location | Both live in BOOT_FLAGS1 (0x04b ×3). There are no separate rows. | OD:729, 755 | Yes |
| 1g | FLASH_DEVINFO | 0x054. ECC, 16 bits. ECC pair with FLASH_PARTITION_SLOT_SIZE at 0x055. | OD:841-847, 979; BR `varm_boot_path.c:724` (`check_ecc_pair`) | n/a |
| 1h | USB_BOOT_FLAGS, R1, R2 | 0x059, 0x05a, 0x05b (RBIT-3, raw 24-bit) | OD:1119-1123, 1288, 1298 | Approximate only ("around 0x059-0x05c", per the e3a repo map). The exact values are in this table. |
| 1i | USB_WHITE_LABEL_ADDR | 0x05c. ECC, 16 bits. Holds the row index of a 16-row ECC table. | OD:1307-1362 | Approximate only, as 1h |
| 1j | PAGE1_LOCK0 / LOCK1 | 0xf82 / 0xf83 (cover rows 0x040-0x07f) | OD:2958, 3017 | Yes (by name in SC:402) |
| 1k | PAGE2_LOCK0 / LOCK1 | 0xf84 / 0xf85 (cover rows 0x080-0x0bf) | OD:3109, 3168 | Yes (by name) |
| 2a | BOOT_FLAGS0 DISABLE_OTP_BOOT | bit 13 (0x002000) | OD:525-527 | n/a |
| 2b | BOOT_FLAGS0 ENABLE_OTP_BOOT | bit 14 (0x004000) | OD:519 | n/a |
| 2c | BOOT_FLAGS0 FLASH_DEVINFO_ENABLE | bit 5 (0x000020) | OD:598 | n/a |
| 2d | other BOOT_FLAGS0 bits | Defined-bits mask 0x3ffffe (bit 0 unused). 1 ENABLE_BOOTSEL_LED; 2 ENABLE_BOOTSEL_NON_DEFAULT_PLL_XOSC_CFG; 3 FLASH_IO_VOLTAGE_1V8; 4 FAST_SIGCHECK_ROSC_DIV; 5 FLASH_DEVINFO_ENABLE; 6 OVERRIDE_FLASH_PARTITION_SLOT_SIZE; 7 SINGLE_FLASH_BINARY; 8 DISABLE_AUTO_SWITCH_ARCH; 9 SECURE_PARTITION_TABLE; 10 HASHED_PARTITION_TABLE; 11 ROLLBACK_REQUIRED (the boot ROM sets this itself, BR `varm_launch_image.c:128`); 12 DISABLE_FLASH_BOOT; 13 DISABLE_OTP_BOOT; 14 ENABLE_OTP_BOOT; 15 DISABLE_POWER_SCRATCH; 16 DISABLE_WATCHDOG_SCRATCH; 17 DISABLE_BOOTSEL_USB_MSD_IFC; 18 DISABLE_BOOTSEL_USB_PICOBOOT_IFC; 19 DISABLE_BOOTSEL_UART_BOOT; 20 DISABLE_XIP_ACCESS_ON_SRAM_ENTRY; 21 DISABLE_SRAM_WINDOW_BOOT | OD:447-650 | n/a |
| 2e | BOOT_FLAGS1 KEY_VALID | bits 3:0 (bit k = slot k) | OD:755 | Yes |
| 2f | BOOT_FLAGS1 KEY_INVALID | bits 11:8 (bit 8+k = slot k). **0xC sets bits 10 and 11, which invalidates slots 2 and 3 only.** | OD:729. BR `varm_blocks.c:241` (`valid = flags & ~(flags >> 8)`), `:274` (`1u << (k + 8)`) | Yes (the repo map says "slots 2 and 3") |
| 2g | other BOOT_FLAGS1 bits | Defined-bits mask 0x0f0f0f: 16-18 DOUBLE_TAP_DELAY, 19 DOUBLE_TAP | OD:678, 696-711 | n/a. HI's retail value 0x000003 is consistent with this. |
| 3 | **How the boot ROM reads BOOT_FLAGS0/1 (and USB_BOOT_FLAGS)** | **Bitwise 3-way MAJORITY vote** of the raw rows `row, row+1, row+2`: `(a&b)|(b&c)|(c&a)`. **Not** an OR. Each bit is set if it is set in ≥2 of the 3 copies. | BR `varm_otp.c:429-434` (C) and `:437-479` (asm). Comment at `bootrom_otp.h:143-146`. Used for BOOT_FLAGS0 at `hardening.h:158-164`, BOOT_FLAGS1 at `varm_blocks.c:238-239`, USB_BOOT_FLAGS at `varm_nsboot.c:258` and `nsboot/native/usb_device.c:1756-1758` | Yes. RB:375-382 and SC say "majority". |
| 3b | CRIT1 vote (contrast) | **NOT a majority vote.** SECURE_BOOT_ENABLE is on if **any 3 of 8** copies have it set, and the boot ROM asserts that this matches the hardware `OTP_CRITICAL` register. picotool uses the same rule for CRIT rows (`crit && sets>=3`). | BR `varm_boot_path.c:497-512` (comment at :498). PT `main.cpp:9273` | **Disagrees (wording).** SC:753 says "The hardware majority-votes these" (CRIT1 ×8). The rule is a 3-of-8 threshold, so 3 stray copies already set the bit. The script's conclusion (require all 8 equal) is still right. |
| 4 | DISABLE_OTP_BOOT vs ENABLE_OTP_BOOT | DISABLE takes precedence ("Disables always supersede enables"). Its only effect: in the normal (BOOTSEL-not-pressed) boot path, the OTP-boot attempt (`s_varm_crit_ram_trash_try_otp_boot`) is skipped and ENABLE_OTP_BOOT is never even read. Flash boot, BOOTSEL/PICOBOOT and every other path are untouched. It is the only reference to either bit in the boot ROM. | OD:443, 525. BR `varm_boot_path.c:909-921` | n/a |
| 5 | KEY_INVALID overrides KEY_VALID | Yes. Effective valid = `KEY_VALID & ~KEY_INVALID` per slot, re-asserted at match time. | OD:742-744. BR `varm_blocks.c:241, 274` | Yes (L2 §4) |
| 6a | Lock-row encoding | One raw 24-bit row holds 3 copies of an 8-bit value: bits 7:0 primary, 15:8 R1, 23:16 R2, decoded by "3-way majority vote". The rows are always readable, and each is write-protected by **its own page's** permissions: the boot ROM maps a row in page 62/63 to the page index `(row & 0x7f) >> 1`, so PAGE1_LOCKx is governed by page 1's lock. | OD:2951-2974, 3010-3033. BR `varm_otp.c:352`. The BL-field majority decode is at BR `varm_misc.S:417-500`. | Yes (SC:388-412) |
| 6b | LOCK0 fields | KEY_W bits 2:0, KEY_R bits 5:3 (key index 1-6, 0 = none). NO_KEY_STATE bit 6 (0 = read-only, 1 = inaccessible, when a key is registered but not entered). | OD:2985-3005, 3136-3156 | Yes (SC:425-435) |
| 6c | LOCK1 fields | LOCK_S bits 1:0, LOCK_NS bits 3:2, LOCK_BL bits 5:4. Each: 0 = RW, 1 = RO, 2 = "do not use, behaves as inaccessible", 3 = inaccessible. | OD:3025-3093, 3176-3244 | Yes (SC:390-394) |
| 6d | Meaning of LOCK1 = 0x040404, LOCK0 = 0 (pages 1, 2) | All 3 copies = 0x04, so LOCK_S = 0, LOCK_NS = 1 (Non-secure read-only), LOCK_BL = 0, and no page keys. Secure and bootloader are fully RW. | decoded from 6c | Yes (RB:209-214, SC:396-398, HI) |
| 6e | Which settings bar picotool | In BOOTSEL the boot ROM ORs the majority-voted **LOCK_BL** into the hardware Secure soft-lock for every page (and makes NS inaccessible). It then serves PICOBOOT OTP access as Secure: **write refused if (LOCK_S \| LOCK_BL) != 0; read refused if bit 1 of (LOCK_S \| LOCK_BL) is set (value 2 or 3).** Separately, a read whose raw MSB comes back set (hardware key check failed, i.e. KEY_R/KEY_W registered with NO_KEY_STATE) is refused. So KEY_W != 0 bars writes, and KEY_R != 0 bars reads if NO_KEY_STATE = 1. | BR `varm_nsboot.c:280-304`, `varm_otp.c:305, 347-382`. OH:17-23 (SW_LOCK is initialised from the OTP lock rows at reset; writes only OR in) | Yes. SC dies on LOCK_S≠0, LOCK_BL≠0, KEY_R≠0 or KEY_W≠0, which is conservative and correct for a write procedure. Read-only access survives LOCK_S/BL = 1. |
| 7a | FLASH_DEVINFO layout | CS1_SIZE 15:12, CS0_SIZE 11:8, D8H_ERASE_SUPPORTED bit 7, bit 6 reserved, CS1_GPIO 5:0. A size code n≠0 means `4 KiB << n` (0xa = 4 MiB, 0xb = 8 MiB, 0xc = 16 MiB; 0 = no device). Default when ENABLE is clear: CS0 = 0xc (16 MiB), CS1 = 0, D8H = 0. | OD:844-962. BR `native_generic_flash.h:98-113` (`FLASH_DEFAULT_DEVINFO`, `0x1000 << size_bits`) | n/a |
| 7b | Boot ROM behaviour with FLASH_DEVINFO_ENABLE + smaller CS0 | The value is read once at boot (ECC read of 0x054) into bootram and then used for: (i) the **flash-boot image / partition-table search window = CS0 size** (BR `varm_flash_boot.c:72`); (ii) **bounds checks on every checked flash op**, so an op whose offset ≥ CS0 size returns NOT_PERMITTED. That covers the `flash_op` API and the BOOTSEL/PICOBOOT/UF2 program, erase and read path, which all go through `s_varm_api_checked_flash_op` (BR `varm_generic_flash.c:722-737`, `varm_flash_permissions.h:130-133`, `varm_checked_flash.c:96`, `varm_s_from_ns_hardened_buffer_wrappers.S:187-251`); (iii) `get_sys_info(FLASH_DEV_INFO)` (BR `varm_apis.c:316-318`). Raw XIP reads are **not** bounded by it: hardware aliasing is outside the boot ROM. | BR `varm_boot_path.c:721-728` | n/a. HI's 4 MB aliasing note concerns XIP, which is consistent. |
| 8 | ECC vs raw | **Raw 24-bit (non-ECC):** CRIT1 ×8, BOOT_FLAGS0 ×3, BOOT_FLAGS1 ×3, USB_BOOT_FLAGS ×3, PAGEn_LOCK0/1. **ECC (16-bit data):** CHIPID0-3, BOOTKEYn_0..15, FLASH_DEVINFO, USB_WHITE_LABEL_ADDR and its 16-row table. Raw rows must be read raw (picotool without `-e`). An ECC read of a lock row returns only 16 bits, so the R2 copy (bits 23:16) is lost. | OD WIDTH 24 vs 16 and "(ECC)" tags: :33, :317, :449, :680, :847, :1123, :1362, :1493, :2961, :3020 | Yes (RB:207-209) |

## picotool facts that bear on "injected unequal copy" and read-back

| # | fact | citation | repo agrees? |
| --- | --- | --- | --- |
| P1 | `otp get` on a NAMED redundant register: picotool reads the whole page raw and votes per bit, `sets >= clears` (crit: `sets >= 3`). If any bit differs it prints `RAW_VALUE=0x..;0x..;0x..` with **every copy's raw value**, then `(WARNING - REDUNDANT ROWS AREN'T EQUAL)`. So a single named read already exposes all 3 copies whenever they differ. | PT `main.cpp:9240-9283` (2.2.0: `:7574-7675`) | Partly. FU/SC rely on the WARNING trap, which is correct. The per-copy RAW_VALUE line is not used anywhere. |
| P2 | **`-c/--copies N` is not a no-op in source.** `redundancy = settings.otp.redundancy` (default -1) and only falls back to `reg->redundancy` when < 0. With `-c 1` the vote runs over one row, so VALUE is the named row's own raw content. On a board whose copies all agree, `-c 1` output is byte-identical to the default **by construction**. | PT `main.cpp:584`, `:9178`, `:9228`, `:9263-9283` | **Disagrees.** FU F-619 (~l.18846) and the e3a repo map (l.139) conclude "`-c 1` does nothing … NO picotool flag reads one physical copy independently". That came from an identical-output measurement on a healthy board, which cannot distinguish the two cases. By source, `-c 1` on `0x04b` returns copy 0 raw. **UNVERIFIED on hardware.** The rehearsal-profile injected-unequal-copy test is exactly what would settle it. |
| P3 | `otp set` on a redundant register reads **only copy 0** (raw, 1 row) as OLD_VALUE, computes the new value from it, and writes that same value to **all N copies in one PICOBOOT command**. The boot ROM processes the rows one at a time: each row is checked, then burned (`varm_otp.c:344, 403-407`). If a later copy already has a bit set that the new value lacks, that row fails `BOOTROM_ERROR_UNSUPPORTED_MODIFICATION` **after** the earlier copies were burned. | PT `main.cpp:9628-9629, 9706-9729`. BR `varm_otp.c:344, 402-407` | Yes (RB:375; L2 §3 per-row nuance). The extra-bit-in-R1/R2 failure mode is not stated anywhere in the repo. |

## Implications for E3a (derived, not new facts)

1. **What an "unequal copy" means.** The boot ROM majority-votes BOOT_FLAGS0/1 and USB_BOOT_FLAGS (fact 3). One odd copy therefore never changes behaviour. Two copies with DISABLE_OTP_BOOT set make it effective. Read-back must check each raw copy individually (all three = expected) **and** the vote. Checking the vote alone hides a 2-of-3 write, as RB:375-382 already explains for BOOT_FLAGS1.
2. **An unequal copy can only be healed upward.** OTP bits only set. Re-running `otp set -s` writes the full OR-ed value to all three copies, which heals a copy missing a bit. If a copy carries an *extra* bit (the injected case), every rewrite fails on that row (P3), and the only "fix" is to set that bit in the other copies too. So the check should refuse rather than offer a repair.
3. **DISABLE_OTP_BOOT is safe to set on SeedHammer-class boards.** It only removes the OTP-image boot path (fact 4) and touches neither flash boot nor BOOTSEL. Its row (BOOT_FLAGS0, page 1) is governed by the same page-1 lock that already permits the BOOT_FLAGS1 writes (fact 6).
4. **KEY_INVALID 0xC** makes slots 2 and 3 permanently unusable, and nothing else. Slots 0 and 1 are unaffected (fact 2f, 5).
5. **A FLASH_DEVINFO burn with too small a CS0_SIZE would hide flash above that size** from flash boot and from BOOTSEL/picotool flash writes (7b). On a 16 MB SH2, any value other than 0xc truncates. Because CS0 = 0xc is already the default, burning FLASH_DEVINFO_ENABLE with CS0 = 0xc changes nothing about sizing. FLASH_DEVINFO is ECC, so it is written once and then permanent.

## UNVERIFIED (could not be settled from source)

- **Exact hardware decode of the lock rows and of CRIT1 bits other than SECURE_BOOT_ENABLE.** The 3-way majority for locks is stated in OD:2951-2953. The boot ROM only implements the LOCK_BL vote in software; the LOCK_S/LOCK_NS/KEY decode is in hardware RTL, which is not public. The 3-of-8 rule for the other CRIT1 bits rests on the boot ROM's assertion that its 3-of-8 count matches `OTP_CRITICAL` (for SECURE_BOOT_ENABLE) and on picotool's generic `crit && sets>=3`.
- **USB_BOOT_FLAGS bits 22 vs 15.** OD's description strings for `WHITE_LABEL_ADDR_VALID` (bit 22) and `WL_INFO_UF2_TXT_BOARD_ID_STRDEF_VALID` (bit 15) appear swapped (OD:1135-1145). The boot ROM code gates reading USB_WHITE_LABEL_ADDR on bit 22 (`usb_device.c:1759`), so **bit 22 = WHITE_LABEL_ADDR_VALID** is code-verified. Bits 0-15 are the per-entry valid bits, indexed as listed in OD:1307-1378.
- **P2 (`-c 1` semantics) on hardware.** It follows from source. It has not been observed on a board with unequal copies.
- Retail SeedHammer white-label and USB_BOOT_FLAGS values: these are a hardware measurement, not a source fact.

## Repo cross-check summary

- **RUNBOOK_custom_boot_key.md:** agrees on every checkable fact: lock decode (l.207-217), ECC-vs-raw (l.207-209), majority vote and the 2-of-3 hazard (l.375-382), KEY_VALID bit per slot (l.363). Nothing found that contradicts source.
- **pico2-bootkey-rehearsal.sh:** row addresses (0x040-0x047, 0x04b-0x04d), lock field positions and decode (l.388-435) all agree. **One wording error:** l.753 says the hardware "majority-votes" CRIT1's 8 copies. It is 3-of-8 for SECURE_BOOT_ENABLE (BR `varm_boot_path.c:498`). This does not affect behaviour.
- **bootkey-round5-lens-external-facts.md:** §3, §4 and §5 agree with source on every point checked here.
- **HARDWARE_INVENTORY.md:** retail values (KEY_VALID 0x1→0x3, KEY_INVALID 0, PAGE1/2_LOCK0 = 0, LOCK1 = 0x040404, BOOT_FLAGS1 copies 0x000003) are all consistent with the field definitions. CHIPID byte and word order is consistent.
- **FOLLOWUPS F-619 / e3a-recon-repo-map l.139:** the "`-c 1` is a measured no-op" conclusion **conflicts with picotool source** (P2). See above.
