# E3a recon: which picotool to pin (2.3.1 vs 2.2.0-a4)

Date: 2026-10-05. Read-only research; source-only (no docs relied on).

Sources (cloned over git):

- `raspberrypi/picotool` tags `2.2.0-a4` = `25aa087b` (2025-08-07) and `2.3.1` = `2041936441` (2026-09-04). 60 commits between them. Citations are `A4 main.cpp:N` and `231 main.cpp:N`.
- `raspberrypi/pico-sdk` tags `2.2.0` (`a1438df`), `2.3.0` (`98a542c`), `2.3.1` (`079c6f3`). Citations: `SDK220 otp_data.h:N`, `SDK231 otp_data.h:N` (`src/rp2350/hardware_regs/include/hardware/regs/otp_data.h`).
- `NixOS/nixpkgs` branches nixos-25.05 `ac62194c3`, nixos-25.11 `b6018f87d`, nixos-26.05 `825e2028c` (newest stable), nixos-unstable `a7868a727`.

## Recommendation: pin **2.3.1**, built against **pico-sdk 2.3.1**

The plan's test is whether 2.3.1 changes field names, `otp set` copy behaviour or erase semantics in a way the runbook does not cover. From the source, it does not:

- **Field names.** No field or row that this repo or the E3a scope uses was renamed. Across the whole OTP table, the only differences are that one field was removed (`BOOT_FLAGS0.DISABLE_BOOTSEL_EXEC2`, bit 0, which nothing here uses) and that three rows were added (`MAC0`..`MAC2` at rows 0x62-0x64).
- **`otp set` and `otp load`.** The copy behaviour is the same code in both tags. The only difference is an extra error message for the A2 erratum E15.
- **`erase` and `save`.** `guess_flash_size`, the refusal on erased flash, and the arithmetic for `-r` ranges are the same code in both. 2.3.1 adds a call to `exit_xip()` before erasing. That is a safety fix (errata E10 and #302) and does not change which bytes get erased.

There is one output change a parser could notice. Under `-e`, or on an ECC row read without `-r`, the `otp get` VALUE line is now 16-bit (`0x%04x`). Before, it was the 24-bit ECC re-encoding. Every parser in this repo already masks with `& 0xffff` or takes field lines, so all of them still work. 2.3.1 also fixes a real bug that `scripts/pico2-bootkey-rehearsal.sh` currently works around: a single-row `otp get` of row 1 (CHIPID1) printed nothing. That bug is the cause of the "CHIPID1 prints NOTHING" observation at R:199-202.

Conditions on the pin (none of them blocks it):

1. Build picotool 2.3.1 against **pico-sdk 2.3.1**, and against mbedtls 3.x so that `seal` is present (see §3). The OTP name table is generated at build time from the SDK's `otp_data.h`, so the names follow the SDK the build uses, not the picotool tag.
2. **`seal` output changes in a way outside the runbook's scope. Re-test the signing chain once before relying on it.** 2.3.1 sets `PICOBIN_IMAGE_TYPE_EXE_EXTRA_SECURITY_BITS` (bit 11 of IMAGE_DEF flags) and inserts a VECTOR_TABLE item when signing an Arm image (`231 main.cpp:5534`, `:5621`; commit 699049f, not in A4). As a result:
   - signed images will not be byte-identical to those 2.2.0-a4 produces;
   - the fork's Go `picosign` must parse the extra item. `sign-firmware.sh` only uses `picotool seal` to create the SIGNATURE structure and then re-signs with `picosign`.
   - The OTP JSON that `seal` emits is the same in both (`bootkey0`, `crit1.secure_boot_enable`, `boot_flags1.key_valid`; A4 `main.cpp:5780-5785` = 231 `:6263-6268`).
3. Update two small things that encode 2.2.0 output: the comment at R:166-171, and `scripts/test/fake-picotool`. The stub emits ECC rows as `VALUE 0x%06x` (`:19`, `:83`) and its `version` line says `v2.2.0-a4` (`:22`). Real 2.3.1 prints `0x%04x` for ECC rows. The parsers accept both, but the stub should mirror the pinned tool.
4. A rule for new E3a code, true in both versions: any raw 24-bit read of an ECC row (FLASH_DEVINFO 0x54, USB_WHITE_LABEL_ADDR 0x5c, USB_BOOT_FLAGS 0x59 and the white-label data rows) must pass `-r`. Without `-r` picotool auto-applies ECC (`A4 :7699` / `231 :9227`). Under ECC, the VALUE line in A4 is `otp_calculate_ecc(raw & 0xffff)` and not the raw row (A4 `:7726-7727`). In 2.3.1 it is the 16-bit data only (`231 :9255`). On 2.3.1, rows 0x62-0x64 become named ECC rows (MAC0-2). If H0's white-label structure ever sits there, a read without `-r` changes format.
5. Never put a *warning* trap around `picotool info` output on a device. 2.3.1 adds a "WARNING: Device has no partition table loaded…" paragraph (`231 main.cpp:4297-4310`). The repo's `info` probes use only the exit status or grep for specific keys, so they are unaffected today.

## 1. Every difference found that is relevant here

| # | Area | 2.2.0-a4 | 2.3.1 | Script-visible? | Covered by runbook/scripts? |
|---|---|---|---|---|---|
| D1 | SDK pairing (OTP table source) | nixpkgs builds it against pico-sdk 2.2.0 (26.05/25.11 `pico-sdk/package.nix:20`). SDK 2.2.0 `tools/CMakeLists.txt:134` requires picotool 2.1.1. picotool `MODULE.bazel:7` = pico-sdk 2.1.0. | SDK 2.3.1 `tools/CMakeLists.txt:146` requires picotool 2.3.0, and `tools/Findpicotool.cmake:42-44` fetches the picotool tag equal to the SDK version. picotool `MODULE.bazel:7` = pico-sdk 2.3.0. `otp_data.h` is identical in 2.3.0 and 2.3.1 (empty diff). | n/a | n/a. Table generated by `otp_header_parse` over the SDK `otp_data.h` (picotool `CMakeLists.txt:179-198` at both tags). Minimum SDK 2.1.0 at both (`CMakeLists.txt:22`). |
| D2 | OTP field removed | `BOOT_FLAGS0.DISABLE_BOOTSEL_EXEC2` bit 0 (SDK220 `otp_data.h:654`); `BOOT_FLAGS0_BITS 0x3fffff` (`:447`) | absent; `BOOT_FLAGS0_BITS 0x3ffffe` (SDK231 `:447`) | Only if a script names that field; none does. A whole-row `otp get BOOT_FLAGS0` simply omits the bit-0 field line. | Not used anywhere in the repo. |
| D3 | OTP rows added | 0x62-0x64 unnamed | `MAC0/1/2` (ECC) at 0x62-0x64 (SDK231 `:1443-1452` ff.) | A read of 0x62-0x64 without `-r` becomes auto-ECC with a header naming the row | Not used today; see condition 4. |
| D4 | Names used by repo/E3a | `CHIPID0..3` rows 0-3 (`:30-60`); `CRIT1` 0x40 RBIT-8 (`:312-315`), `.SECURE_BOOT_ENABLE` bit 0 (`:366`); `BOOT_FLAGS0` 0x48 RBIT-3 (`:441-446`) with `DISABLE_OTP_BOOT`, `ENABLE_OTP_BOOT`, `FLASH_DEVINFO_ENABLE`, `ROLLBACK_REQUIRED` etc.; `BOOT_FLAGS1` 0x4b (`:684`), `.KEY_VALID` 0x00f (`:762`), `.KEY_INVALID` 0xf00 (`:736`); `BOOTKEY0_0` 0x80 ECC (`:1450-1452`) .. `BOOTKEY3_15` 0xbf (`:2082`); `FLASH_DEVINFO` 0x54 (`:851`); `USB_BOOT_FLAGS` 0x59 (`:1127`); `USB_WHITE_LABEL_ADDR` 0x5c (`:1366`); `PAGE1_LOCK0/1` 0xf82/0xf83 (`:2919`/`:2978`), `PAGE2_LOCK0/1` 0xf84/0xf85 (`:3069`/`:3128`), `LOCK_S` 0x3, `LOCK_BL` 0x30 | **identical names, rows and masks** (SDK231 `:30-60`, `:314`, `:366`, `:446`, `:677`, `:755`, `:729`, `:1490`, `:2120`, `:844`, `:1120`, `:1359`, `:2958`, `:3017`, `:3109`, `:3168`). Only the header line numbers moved. Of the 1331 `_ROW`/`_BITS` names in 2.2.0, all are present in 2.3.1 except D2 (machine diff). | no | yes |
| D5 | How redundancy/ECC is attached to names | `otp_header_parse.cpp:137-139`: `(ECC)` and `(RBIT-n)` in descriptions, and `_Rn` rows folded into the base register | same file, same lines (only `BUILD.bazel` changed in `otp_header_parser/`) | no. `0x04b` still resolves to `OTP_DATA_BOOT_FLAGS1 (RBIT-3)` and `0x04c`/`0x04d` still print bare (F-619 holds) | yes (R:370-376, R:835-845) |
| D6 | `otp get` ECC VALUE line | `"\nVALUE 0x%06x\n", corrected_val`, where `corrected_val = otp_calculate_ecc(raw&0xffff)` is the 24-bit re-encoding (A4 `:7726-7727`) | `"\nVALUE 0x%04x\n", corrected_val & 0xffff` (231 `:9254-9255`; commit af47f2d "Only print 16-bit ECC value") | **Yes: format width.** `read_rows` (R:216-218) takes `0x[0-9a-f]+` and then `& 0xffff`, so it works with both. | Partly. R:166-171 documents the 2.2.0 form; fake-picotool emits the 2.2.0 form. |
| D7 | `otp get` row-1 bug | `last_reg_row = 1 // invalid` (A4 `:7640`, `otp list` `:7989`). When the first match is row 1, the ROW header and VALUE are suppressed, so a lone `otp get -n CHIPID1` prints nothing and exits 0. | `UINT32_MAX` (231 `:9168`, `:9517`; commit 3003f8d "Fix OTP get/list for row 1") | Yes, as a fix. The batching in `read_rows` (R:199-210) still works and is now belt-and-braces. | Yes (worked around) |
| D8 | `otp get` redundant-row vote and WARNING | Majority vote `sets>=clears` (crit `sets>=3`), `RAW_VALUE=` line plus `(WARNING - REDUNDANT ROWS AREN'T EQUAL)` printed **before** VALUE; `(WARNING - ECC IS INVALID)` for ECC (A4 `:7715`, `:7733`, `:7738-7751`) | identical (231 `:9243`, `:9261`, `:9266-9279`) | no | yes (every reader traps `*warning*`) |
| D9 | `otp get` flags | `-c/--copies`, `-r/--raw`, `-e/--ecc`, `-n/--no-descriptions`, `-i`, `-z/--fuzzy` (A4 `:1124-1134`). ECC is applied automatically for `ecc` rows unless `-r` (`:7699`). | identical CLI block (A4 `:1116-1250` vs 231 `:1370-1504`: byte-identical diff) and same auto-ECC (231 `:9227`) | no | yes |
| D10 | `otp get` field line | `field NAME (bit n)`/`(bits n-m)` then `" = %x"` bare hex (A4 `:7775`) | identical (231 `:9303`) | no | yes (`otp_field`, R:176-193) |
| D11 | `otp set` copies | One raw read of the **named row only** (`old_raw_value`, A4 `:8099`). With a field selector it merges `old_raw_value & ~mask`. `-s` ORs `old_raw_value` (`:8182`). It refuses to clear bits (`:8185`) and refuses any ECC row that is already non-zero (`:8190`). For a redundant row it writes **all `redundancy` copies in one PICOBOOT write**, every copy with the same value computed from copy 0 (`:8197`). It does not read back. | identical logic (231 `:9628`, `:9711`, `:9714`, `:9719`, `:9726`). The only change is `check_otp_write_error(e,&otp_cmd,model)`, which adds an E15 message for A2 page-lock writes to pages 32-63. | no | yes (runbook "`otp set` writes all three redundant `BOOT_FLAGS1` copies in **one** PICOBOOT command"). `BOOT_FLAGS0.DISABLE_OTP_BOOT` will likewise write rows 0x48-0x4a. |
| D12 | `otp set` flags | `-c/--copies`, `-r/--raw`, `-e/--ecc`, `-s/--set-bits`, `-i`, `-z` (A4 `:1219-1226`) | identical | no | yes |
| D13 | `otp load` (JSON) | `process_otp_json` (A4 `:7472`): a named register writes `seq_length` rows for a sequence such as `bootkeyN`, or `redundancy` rows; `bEcc = reg->ecc` (`:7513`). A single value is replicated to every copy. | function byte-identical (231 `:9000`, `:9041`) | no | yes |
| D14 | `otp load` (BIN), `otp dump`, `otp permissions` | — | `otp permissions` no longer accumulates lock bits across pages (commit 2006299). Command order in `otp` help changed. | no (not used) | n/a |
| D15 | `erase` | `-a` (default) or `-p N` or `-r from to`. `-r` rounds out to 4 KiB (A4 `:880-886`). With no `-r`, `guess_flash_size` is used; 0 means "Cannot determine the flash size… try --range." (`:4715-4717`). | Same CLI (231 `:1129-1135`) and same logic (`:5164-5166`). Adds `exit_xip()` on A2 at entry (`:5124`) and **unconditionally** before the erase loop (`:5179`; commits d745d64, 29e6e3f). | no | yes |
| D16 | `guess_flash_size` | Returns 0 if the first two 256-byte pages are identical (erased flash). Otherwise probes aliases at 8 MiB, 4 MiB, ... down to 4 KiB and returns `2*size` (A4 `:2840-2860`). | byte-identical (231 `:3176-3196`) | no | yes |
| D17 | `save` | `-a`/`-r`, same `guess_flash_size` refusal (A4 `:4543-4545`) | byte-identical function (231 `:4988-4990`) | no | yes |
| D18 | `load` | `-v/--verify`, `-x`, `-o/--offset`, `-t`, `-p`, `-n/-N`, `-u` (A4 `:841-872`) | identical except the help text for `-x` (231 `:1106`). Partition tuple became a struct (`.start/.end`). | no | yes |
| D19 | `reboot` | `-a`, `-u`, `-c`, `-g` (A4 `:1470-1480`) | Adds the global `--bootsel-led <gpio>` / `--bootsel-led-active-low` (231 `:717-725`), which only matter with `-u`. A plain `reboot` uses the same `dParam0 = diagnostic partition` (231 `reboot_command::execute` `:10059`). | no | yes |
| D20 | Global device selection | `--bus`, `--address`, `--vid`, `--pid`, `--ser`, `-f`, `-F` (A4 `:605-614`) | Adds `--rp2040` (231 `:714`). `--ser` is still a **case-sensitive `strcmp` against the USB iSerialNumber** (`picoboot_connection.c:211-215` at 2.3.1 = `:205-209` at A4). An RP2350 with a non-standard VID/PID whose chip-info query fails is now `dr_vidpid_unknown` rather than assumed RP2350. | no | yes (runbook "`--ser` wants the picotool form, uppercased, without `0x`") |
| D21 | `info` chipid / signature | `info_pair("chipid", hex_string(data[1] \| data[2]<<32, 16))` (A4 `:3776`); `signature: verified\|incorrect` (`:3499`); `public key:` (`:3512`) | identical strings (231 `:4153`, `:3868`, `:3881`) | no | yes |
| D22 | `info` on a device | — | New WARNING paragraph when a PT is at the start of flash but none is loaded, or when it is badly hashed or signed (231 `:4297-4310`, commit f3602dc) | Only for a script that greps `info` output for "warning" | n/a today (condition 5) |
| D23 | `seal --sign` output | IMAGE_DEF flags unchanged; no VECTOR_TABLE item added when there is no entry point (A4 `sign_guts_bin` `:5126`) | Sets `EXTRA_SECURITY` bit 11 and adds VECTOR_TABLE (231 `:5534`, `:5621`). ELF inputs are also squashed unless `--no-squash` (231 `seal_command::execute` `:6122` ff.). Adds `--pin-xip-sram` and `--no-squash` (231 `:1204-1206`). | **Yes: signed bytes differ** | **Not covered** (condition 2). The OTP JSON written by `seal` is unchanged. |
| D24 | udev rule shipped | `udev/60-picotool.rules` with `TAG+="uaccess"` and GROUP plugdev in one rule | Split into plugdev rules and separate `TAG+="uaccess"` rules; still `60-` | no | yes (runbook requires < 73) |

## 2. Every picotool invocation in this repo's scripts

R = `scripts/pico2-bootkey-rehearsal.sh`, F = `scripts/sh2-flash` (a single file, not a directory), S = `scripts/sign-firmware.sh`, E = `scripts/test/run-e2e.sh` (which runs against `fake-picotool`).

| Where | Invocation | 2.2.0-a4 | 2.3.1 |
|---|---|---|---|
| R:176 `otp_field` | `otp get -n <ROW.FIELD>` (CRIT1.SECURE_BOOT_ENABLE, BOOT_FLAGS1.KEY_VALID, BOOT_FLAGS1.KEY_INVALID) | OK | OK (D4, D8, D10 unchanged) |
| R:210 `read_rows` | `otp get -n -e CHIPID0..3` / `BOOTKEYn_0..15` | OK. VALUE is 24-bit, masked to 16. Lone CHIPID1 is lost (D7), avoided by batching. | OK. VALUE is 16-bit (D6) and the parser accepts it. The CHIPID1 bug is fixed. |
| R:222-236 | the `^ROW 0x…: OTP_DATA_<NAME>` order check | OK (ascending, deduped) | OK (filter_otp unchanged) |
| R:367 `read_row_raw24` | `otp get -n PAGE{1,2}_LOCK{0,1}` / `0x040..0x047` / `0x04b..0x04d` | OK (non-ECC rows: raw or voted 24-bit VALUE) | OK (same paths). **Do not reuse it for ECC rows without adding `-r`** (condition 4). |
| R:462 | `seal --sign --quiet blinky.uf2 out.uf2 key.pem otp.json` (only the JSON is kept) | OK, needs an mbedtls build | OK. JSON identical; the discarded UF2 differs (D23). |
| R:509 / 504 | `load --verify <img>` | OK | OK |
| R:512 / 504 | `reboot` | OK | OK |
| R:555 | `info` (exit status only) | OK | OK (new warning goes to stdout; exit status unaffected) |
| R:691 (printed hint) / 960 / 1074 | `otp load <json>` (bootkeyN only) | OK, writes 16 ECC rows | OK, identical (D13) |
| R:814, 822 (printed hint) / 969 / 1082 | `otp set -s BOOT_FLAGS1.KEY_VALID 0x…` | OK, writes 3 copies | OK, identical (D11) |
| R:970 | `otp set -s CRIT1.SECURE_BOOT_ENABLE 0x1` | OK, writes 8 copies | OK, identical |
| R:900-901 | `otp get CRIT1.SECURE_BOOT_ENABLE`, `otp get BOOT_FLAGS1.KEY_VALID` (display only) | OK | OK |
| R:1104, 1110 | `info -a <file.uf2>`, grep `public key:` / `signature: *verified` | OK, needs mbedtls | OK (D21) |
| F:190 | `nix develop --command picotool version` | OK | OK |
| F:290 | `info -a "$SIGNED"`, grep verified | OK, needs mbedtls | OK |
| F:326 | `load --verify "$SIGNED"` | OK | OK |
| F:327 | `reboot` | OK | OK |
| S:85, 173 | `info -a "$IMG"` | OK, needs mbedtls | OK |
| S:104 | `seal --sign --clear --quiet in.uf2 out.uf2 seal.pem` | OK | **Output differs** (EXTRA_SECURITY bit and VECTOR_TABLE item). Re-test that `picosign hash/sign/extract` accept it. |
| S:188 (hint), demo/sh2/build-payload.sh:125 (hint) | `load --verify [-t bin -o 0x10D00000] …` + `reboot` | OK | OK (D18) |
| E:143, 164, 187 | `otp load`, `otp set -s BOOT_FLAGS1.KEY_VALID` against fake-picotool | n/a (stub) | n/a. The stub's ECC VALUE width and version line should be updated to 2.3.1 (condition 3). |

The repo has no `erase` or `save` invocation today. E3a's explicit-range `erase -r`/`save -r` will behave the same under both versions (D15-D17).

## 3. nixpkgs and the overlay

| Branch | picotool | pico-sdk | mbedtls patched in (so `seal` and signature verification exist) | udev rule installed |
|---|---|---|---|---|
| nixos-25.05 | 2.1.1 (`picotool/package.nix:16`) | 2.1.1 | yes, mbedtls_2 (`:26-30`) | no |
| nixos-25.11 | 2.2.0-a4 (`:16`) | 2.2.0 | **no postPatch**: `mbedtls` is an argument but unused, so without the SDK's mbedtls submodule picotool builds **without `seal`** and without signature verification in `info` | no |
| nixos-26.05 (newest stable) | 2.2.0-a4 (`:16`) | 2.2.0 | yes, mbedtls 3.6.6 src (`:25-30`) | yes (`:45-46`) |
| nixos-unstable | **2.3.1** (`:16`, hash `sha256-D5zsAQlDAv9hVMjAOTzXalwqYKgnBmqL3dkKlyNdizo=`) | **2.3.1** (`pico-sdk/package.nix:20`, no-submodule hash `sha256-UWWrZT7Mg2yei/LlDxgei3yXWaP9ETMlgiVOFFAEMZ8=`) | yes, mbedtls 3.6.7 src | yes |

So the runbook's "nix develop provides 2.2.0-a4" can only be a working toolchain if the fork's flake follows nixos-26.05 or later, or patches mbedtls in itself. On 25.11, `seal` does not exist. I could not check this here because the fork is not in this sandbox.

An overlay pinning 2.3.1 needs **no patches beyond what nixos-unstable already carries**:

- `pico-sdk` → version 2.3.1, `fetchFromGitHub` tag 2.3.1, the hash above. Its default `withSubmodules = false` is fine.
- `picotool` → version 2.3.1, the hash above, `cmakeFlags = [ "-DPICO_SDK_PATH=${pico-sdk}/lib/pico-sdk" ]` pointing at the 2.3.1 SDK, plus the `postPatch` that substitutes `${PICO_SDK_PATH}/lib/mbedtls` → `${mbedtls.src}` in `lib/CMakeLists.txt`. That needs mbedtls 3.x: SDK 2.3.1's own submodule is at `0bebf8b8`, and 3.6.6/3.6.7 from nixpkgs both work for this. Keep `postInstall` installing `../udev/60-picotool.rules`.
- The simplest form is to take both `package.nix` files from nixos-unstable `a7868a727` via an `inputs.nixpkgs-unstable` and `callPackage` them with the stable `mbedtls`. Verify with `picotool version` → `2.3.1` and with `picotool help` listing `seal`.

Do not build 2.3.1 against SDK 2.2.0. It compiles (2.2.0 has `pico/usb_reset_interface.h`), but the OTP table would then be the 2.2.0 one (D2/D3), giving a hybrid that matches neither tag.
