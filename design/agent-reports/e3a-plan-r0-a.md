# E3a plan R0, lens A (correctness and external facts)

Reviewed: `design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md` draft 1 at mnemonic-engrave `6b0d1ba`.
Reviewer: independent opus agent, 2026-10-05. Read-only apart from this file.

**Counts: 2 Critical / 5 Important / 10 Minor / 3 Nit**

## Sources checked (primary, cloned over git)

| key | source | revision |
|---|---|---|
| PT | `raspberrypi/picotool` `main.cpp`, `cli.h`, `model/model.h` | tag 2.3.1 = `2041936441b48a3cc53ae3da9e805229fe8f4e18` |
| OD | `raspberrypi/pico-sdk` `src/rp2350/hardware_regs/include/hardware/regs/otp_data.h` | tag 2.3.1 = `079c6f39` |
| BR | `raspberrypi/pico-bootrom-rp2350` | tag A4 = `c6cdb171` |
| NX | `NixOS/nixpkgs` `pkgs/by-name/pi/picotool/package.nix`, `pico-sdk/package.nix` | nixos-unstable `a7868a727837f3c09cee2ce0ca671c76b1589fed` |
| BIN | the real picotool 2.3.1, built here from NX: `nix build 'git+https://github.com/NixOS/nixpkgs?ref=nixos-unstable&shallow=1&rev=a7868a72…#picotool'` -> `/nix/store/5cxc3bjs…-picotool-2.3.1` (cache hit, 23 s) | — |
| H | a parse harness: picotool 2.3.1's own `cli.h` compiled with the exact `otp get` / `otp set` / `device_selection` groups copied from `main.cpp:705-727, 1370-1495`, printing what each argv parses to | — |

## F1-F13 re-verification

| id | verdict | evidence |
|---|---|---|
| F1 | TRUE | OD:446, 654, 664 (0x048/49/4a); 677, 762, 772 (0x04b/4c/4d); 1120, 1288, 1298 (0x059/5a/5b); WIDTH 24 |
| F2 | TRUE | OD:527 (0x2000), 519 (0x4000), 598 (0x20) |
| F3 | TRUE | OD:755 (0x00f), 729 (0xf00); BR `varm_blocks.c:241` |
| F4 | TRUE | BR `varm_otp.c:429-434` (`(a&b)|(b&c)|(c&a)`); BR `varm_boot_path.c:497-512` (any 3 of 8) |
| F5 | TRUE | BR `varm_boot_path.c:909-921` |
| F6 | TRUE | OD:844, 917, 933; BR `varm_boot_path.c:721-727`, `varm_generic_flash.c:730` |
| F7 | TRUE, with the OD description swap noted | OD:1138 (bit 22 = 0x400000), OD:1307-1378; BR `usb_device.c` gating as OF says |
| F8 | TRUE | BR `varm_otp.c:347-382` (`lock_fail = page_locks & (is_write ? 0x3 : 0x2)`) |
| F9 | TRUE | PT `main.cpp:9263-9283` |
| F10 | TRUE **only when `-c` is the first option** — see A-C1. As argv is usually written (`-n -c 1`), `-c 1` is silently a selector | PT `cli.h` group matching; H; BIN |
| F11 | TRUE | PT `main.cpp:9628-9729`; BR `varm_otp.c:402-407` (`current_val & ~full_val` -> UNSUPPORTED_MODIFICATION, row by row) |
| F12 | TRUE | PT `main.cpp:9168` (`UINT32_MAX`), `:9253-9255` (`0x%04x`); NX build contains `OTP_DATA_MAC0` (SDK 2.3.1 table) and `seal` |
| F13 | TRUE for `erase` (default `-a` guesses) and `save -a`; plain `save` defaults to `-p` (program only, binary end), not to a guess | PT `main.cpp:4947-4992`, `5162-5166` |

G4 is already answerable: BIN prints `No accessible RP2350 devices in BOOTSEL mode were found with serial number X.` and exits **249** on a `--ser` miss (2.3.1). G2 is answerable too (A-M7).

---

## Critical

### A-C1 — picotool's argument parser is order-sensitive; trailing `--ser`, a late `-c 1` and a late `-r` are silently turned into selectors on `otp get` (§1 F10, §3.1 "every picotool call … passes `--ser` … reads included", §3.2 rule (d), §6, §7 R6)

**What is wrong.** picotool does not use clipp; `cli.h` is its own "hastily hacked together" parser (PT `cli.h:25-32`), and its groups match **in declaration order**. For `otp get` the declared order is `-c -r -e -n -i`, then device-selection (`--ser`), then `-z` and the repeatable `<selector>` (PT `main.cpp:1370-1385`). Anything written after a later element is swallowed by the repeatable selector value. Measured with the harness (H) and confirmed on the real 2.3.1 binary (BIN):

| argv | parsed as |
|---|---|
| `otp get -n -c 1 0x04b` | copies **unset**, selectors `['-c','1','0x04b']` |
| `otp get -c 1 -n 0x04b` | copies 1, selectors `['0x04b']` |
| `otp get -n BOOT_FLAGS0 --ser X` | ser **unset**, selectors `['BOOT_FLAGS0','--ser','X']`; BIN prints "…were found." with no "with serial number" |
| `otp get --ser X -c 1 0x048` | ser X, copies **unset**, selectors `['-c','1','0x048']` |
| `otp get -n -r 0x054` | raw **unset** (ECC applied), selectors `['-r','0x054']` |
| `otp get -n -e CHIPID0 …` (R's `read_rows`, R:210) | `-e` is a selector; harmless only because ECC rows are auto-ECC |
| `otp get -c 1 -n --ser X 0x048` | copies 1, ser X, selectors `['0x048']` — the only safe shape |
| `otp set -s SEL 0x1 --ser X`, `otp set -c 1 -s 0x04b 0x800 --ser X` | correct (device selection is last for `set`, PT `main.cpp:1490-1493`) |
| `otp set -s -c 1 0x04b 0x800`, `otp set --ser X -s …` | parse error (fails safe) |

An unmatched selector (`-c`, `--ser`, `X`) matches no row and is dropped without a message (PT `main.cpp:7598-7660`, `init_matches`), so the read runs **against whatever single device is attached**, with default redundancy. This also fully explains F-619's measurement, which OF P2 and the plan attribute to semantics: `otp get -n -c 1 0x04b` on 2.2.0-a4 added selector `1` = CHIPID1, whose output the row-1 bug suppressed (PT D7), so the output was byte-identical; `-c 3` added CHIPID3 — exactly the "unrelated row (OTP_DATA_CHIPID3)" F-619 recorded (`design/FOLLOWUPS.md:18846`). Neither "`-c 1` is a no-op" nor "`-c 1` returns copy 0" is the whole fact; the fact is "`-c 1` works only in first position".

**Why Critical.** Three of the plan's safety mechanisms — reads bound to the CHIPID, rule (d), and raw (`-r`) reads of ECC rows (PT condition 4) — are silently voided by the natural argv spelling, and the plan's Python fake, written from `execute()` semantics, would accept options in any order. The e2e suite and the "`--ser` pass-through" mutation check would be green while silicon ignores `--ser` and `-c 1`. R6 would then re-record F-619's false conclusion. The WARNING trap still catches an unequal copy, so this is a false-assurance defect on an irreversible-write tool, the class CLAUDE.md names.

**Fix.**
1. Add F14 to §1: "picotool 2.3.1 `cli.h` matches options in declaration order; `otp get` must be spelled `otp get [-c N] [-r] [-e] [-n] [--ser S] <selectors…>`; `otp set` as `otp set [-c N] [-r] [-e] [-s] <selector> <value> [--ser S]`; a misplaced option becomes a selector with no error", citing PT `cli.h` and `main.cpp:1370-1385, 1464-1495`.
2. One bash builder in `otp-read.sh` produces every picotool argv in canonical order; no call site writes argv by hand.
3. The fake enforces the same order and exits 99 on any option seen after a later group, or on any selector that does not parse as a row, name or `ROW.FIELD`.
4. Add a no-hardware CI probe against the **real** pinned binary: `picotool otp get -c 1 -n --ser ZZPROBE 0x048` must print `with serial number ZZPROBE` and exit 249. Do the same for each argv shape the tool emits, so a parser change in a future picotool turns CI red.
5. In §5, rewrite F-619's correction around argument order, not OF P2.
6. Add an e2e case: misordered argv is refused by the fake. Add a mutation: the builder emits `-n` before `-c`, and that must turn a case red.

### A-C2 — the bench sequence cannot pass: the write commands' own pre-check refuses the unequal copies R5/R8 inject, which R7/R9 rely on writes to heal (§3.1, §3.3, §7 R5-R9, §9)

**What is wrong.** §3.1 says "a full `check --stage pre` must pass in the same run immediately before the write". §3.3 says `disable-otp-boot` pre-checks "with equal copies", and that the partial-burn case cannot start because "the pre-check has already refused it (copies unequal)". §7 R5 injects DISABLE_OTP_BOOT into copy 3 only, then requires `check --stage pre` to **refuse**. R7 then runs `disable-otp-boot --execute` on that same board and says "the write succeeds and heals the copy". By the plan's own rules R7 exits 2 at the pre-check. R8 and R9 repeat this for BOOT_FLAGS1. So §9's "the §7 rehearsal passes" is unsatisfiable as written. That is a gate which has never run, and which cannot pass.

The physical claim is right: BR `varm_otp.c:402-405` refuses a row only if `current_val & ~full_val`, and `otp set` computes the value from copy 0 (PT `main.cpp:9628, 9711-9719`), so a copy that is a **subset** of the new value is healed. The obvious way to make R7 pass is to relax copy equality in the write pre-check. Doing that in general removes the guard against F11's partial burn on a retail unit, so the plan has to make this decision, not the implementer.

**Fix.** State the precise precondition for each write: "every raw copy ⊆ the value `otp set` will write (copy 0 | new bits), and copy 0 ⊆ that value". That is the exact F11 condition, not equality. Choose one of two designs:
- (a) Under `retail`, keep equal-copies as the write pre-check (no heal path on a SeedHammer). Under `rehearsal`, allow a flag such as `--heal-subset-copies`, which accepts unequal copies only when each copy is a subset of the target, prints them, and requires the post-check to show all copies equal.
- (b) Keep equality everywhere, and in §7 run the injections after the writes, in the opposite direction: inject a bit the target lacks, and assert refusal plus no write (the fake already models F11's failure).

Either way, add the matching e2e cases: subset copy plus write gives a heal and post PASS; superset copy gives a refusal and the state file unchanged.

---

## Important

### A-I1 — §7 R1 skips R's phase 3, so R4 (R's phase 5) dies, and the A/B proof R exists for is lost (§7 R1, R4)

R's phase 5 dies unless phase 3 produced `blinky-mykey.signed.uf2`: "no phase-3 SIGNED image found -- run phase 3 --execute first" (R:1099). R's header says "Do not skip phase 3" (R:22-23). R1 lists "phases 0-2 and 4". **Fix:** R1 = phases 0, 1, 2, 3 (`--execute`), 4. Also state that each later "phase 5 again" (R7, R9) needs the board re-entered into BOOTSEL first, and needs either the real `seedhammerii-*.uf2` or `ACCEPT_BLINKY_ONLY=1` (R:1137-1182). Phase 5 leaves the board running an image, so R5's `inject-copy` (BOOTSEL) needs a manual re-entry too.

### A-I2 — no environment can run R under the pin: `.#otp` lacks tinygo/go/the fork, the fork's shell keeps 2.2.0-a4 (which R will refuse), and R's signing depends on an unsequenced `picosign` re-test against 2.3.1 `seal` (§2, §4, §5, §7 R0-R1, §8)

R requires `tinygo` and `go` for phases 0, 3, 4, 5 and 6 and for `--make-otp-json` (R:131-141). It requires `SEEDHAMMER_DIR` to be the fork (R:145-146), because `sign_image` runs `sign-firmware.sh`, which runs `picotool seal --sign` and then `go run seedhammer.com/cmd/picosign` in the fork (`scripts/sign-firmware.sh` ~:64, :104-108). §2 moves "R's existing bench users" to `nix develop .#otp`, whose package list is picotool, openssl, jq, python3, bash and coreutils: no tinygo and no go. The runbook's route (`nix develop --command ../mnemonic-engrave/scripts/…` from the fork, RUNBOOK:16, 195, 275, 346, 371, 414) gives picotool 2.2.0-a4 until the fork's own PR lands, and `require_pinned_picotool` (no override, §2) then refuses. Separately, 2.3.1's `seal --sign` adds EXTRA_SECURITY and a VECTOR_TABLE item (PT `main.cpp:5534, 5621`). §2 hands the `picosign` re-test to the fork thread, and §8 does not order it before §7. So R3/R4 depend on a deliverable that has no position in the order of work.

**Fix:** either (a) add tinygo and go to `devShells.otp`, and document `SEEDHAMMER_DIR` as a prerequisite, or (b) make §8 step 5 depend on the fork's devshell PR having merged with the flake's picotool. In both cases add a §8 step before the bench: "re-run `sign-firmware.sh` on the blinky under 2.3.1 and confirm `picosign hash/sign/extract` and `picotool info -a` → `signature: verified`". Also scope `require_pinned_picotool` in R: SH2 read-only modes and phases may still need to run under the fork shell in the interim, or the runbook is dead between the two PRs.

### A-I3 — the test plan contradicts itself: the old fake reports 2.2.0-a4 (R will refuse it), and case 9 cannot run "without TinyGo" (§4, §6 case 9, §9)

`scripts/test/fake-picotool:22` prints `picotool v2.2.0-a4 (stub)`. With `require_pinned_picotool` at R's start (§4), every old-fake run dies, yet §9 requires "R's existing e2e passes under both fakes". Case 9 promises R's sections A-E with "a fixture UF2 so TinyGo is not needed". But R dies on `command -v tinygo`/`go` for phases 0 and 3-6 before it looks for `blinky.uf2` (R:131-135). Phases 3/5/6 sign through `go run …picosign` in the fork. And `run-e2e.sh:36-37` itself exits 2 without real picotool and tinygo. So the CI job cannot run case 9 on bare ubuntu. **Fix:** update the old fake's version line to `picotool v2.3.1 (stub)` (PT condition 3), and say so. For case 9, either run it only in a fork-devshell job (and say CI covers cases 1-8 only), or specify stubs (a `tinygo` and a `go` stub on PATH, with the signing chain faked in the new fake's `seal`/`info -a`) and state which of R's assertions they then cover.

### A-I4 — R10 makes R11 fail, and R10 cannot be run with R's `--make-otp-json` as described (§7 R10-R11)

After R10 burns slot 2 and sets KEY_VALID bit 2, R11's `check --profile rehearsal` must fail: slots 2 and 3 must be "all zero" and KEY_VALID 0x3 (§3.2). R10 also says "a third rehearsal key" via `--make-otp-json`. That mode calls `reject_rehearsal_key`, which refuses any key inside `rehearsal-work/` and any key equal to factory, my or third-party (R:633-680), so it dies unless the key lives outside WORKDIR or `ALLOW_UNCHECKED_KEY=1` is set. It also needs tinygo (R:139). There is no R phase that flashes and judges a slot-2-signed image. **Fix:** move R10 after R11, record R11's PASS first, and have R10 end with an expected `check` **refusal** naming slot 2 and KEY_VALID. Spell out R10's commands: a fresh key outside WORKDIR, `--make-otp-json --slot 2`, `otp load … --ser`, `otp set -s BOOT_FLAGS1.KEY_VALID 0x4 --ser`, then `sign_image`/flash/`bootsel_present` by hand.

### A-I5 — the retail check compares the white-label table but not the strings it points to, which the UI spec requires (§3.2 USB row, §3.5)

UI spec §4.4 "Expected OTP state" (refugium-wallet `319d28d`, SPEC_ui_refugium_wallet.md ~l.350-356): "`USB_WHITE_LABEL_ADDR` and the white-label structure it points to hold their retail values … Retail units write them on the device, one string being the board revision". OD:1307-1330: the 16 table rows are `_VALUE`s or `_STRDEF`s, and each STRDEF points to string data at "USB_WHITE_LABEL_ADDR value + msb_byte", **outside** the 16 rows. The plan reads the revision string, so it reads string rows, and uses it to choose the expected entry, yet it compares only "the 16-row table". The strings drive the USB identity BOOTSEL shows (BR `nsboot_usb_client.c:268-276` serves manufacturer, product and serial through `white_label_copy_string`). **Fix:** for every entry whose USB_BOOT_FLAGS valid bit is set and which is a STRDEF, read the string rows raw and with ECC (length from the low 7 bits, ×2 if bit 7). Record them in `capture` and in `retail-otp.json`, and compare them in the retail `check`. Add a fake case where one string row differs, which must exit 2.

---

## Minor

- **A-M1 (§3.2 rule (d)).** A numeric read of 0x048, 0x04b, 0x040 or 0x059 resolves to the named register and returns the **vote**, not copy 0 (PT `init_matches`, `main.cpp:9216`; F-619). So "the first copy's raw read" does not exist without `-c 1`. Restate (d) as "`otp get -c 1 -n --ser S <first row>` equals the 3-copy read of that row's raw value". Apply (d) to CRIT1 (0x040) and USB_BOOT_FLAGS (0x059) as well, not only BOOT_FLAGS0/1. Under `rehearsal`, still require USB_BOOT_FLAGS copies to be equal (E3a "raw reads of all three copies of each boot-flag row", build plan l.371-372), even though the values are not compared.
- **A-M2 (§7 R6).** At R6 the odd copy is copy 3 (0x04a), so `-c 1` on 0x048 returns the same value as the vote whether or not it isolates copy 0. That is F-619's uninformative measurement again. Take the F10 reading after R8, while copy 0 of 0x04b differs: `otp get -c 1 -n --ser S 0x04b` must show 0x000803 while the vote shows 0x000003.
- **A-M3 (§3.2 CRIT1).** Only SECURE_BOOT_ENABLE is compared. The UI spec has DEBUG_DISABLE "stays clear" (spec l.~362). Compare the whole CRIT1 value (and CRIT0, which carries ARM/RISCV_DISABLE) against the recorded retail and stock values.
- **A-M4 (§3.2 vs UI spec provisioning).** The pre-stage of `disable-otp-boot` accepts "slot 1 empty, KEY_VALID 0x1". UI spec step 5 sets DISABLE_OTP_BOOT only after the key (step 3) and the image (step 4). Define `--stage pre` per write: for `disable-otp-boot` that is slot 1 = fork key, KEY_VALID 0x3. Keep the spec's step-1 precheck (which accepts "slot 1 holds the fork's key with its valid bit clear", A12-M2) as a separate `--stage precheck`, or say that E3a does not cover it.
- **A-M5 (§3.1 / §3.2).** `check` has no `--ser` in its synopsis, yet the table requires "CHIPID equals `--ser`", and every call must pass `--ser`. Moving `read_rows` and the other helpers into `otp-read.sh` while adding `--ser` changes R's argv, which is not "behaviour-preserving". Give the helpers an optional serial parameter that R leaves empty, and add `--ser` to `check`/`capture` (required, or taken from a one-device CHIPID read and then pinned).
- **A-M6 (§2, §6, §8 nix).** The flake is feasible as described: NX already carries 2.3.1, pico-sdk 2.3.1, the mbedtls 3.x substitution and the udev rule, so no overlay is needed. But in this container `github:` and `https://github.com/…/archive/…tar.gz` both return 403 (GitHub API, proxy). Only `git+https://github.com/NixOS/nixpkgs?ref=nixos-unstable&shallow=1` fetches. Use that URL form for the input (or document `nix flake lock` from Brian's box), or §8 step 1's in-container `nix build` cannot run.
- **A-M7 (§1 G2).** G2 is answerable now. `0x049`, `0x04a`, `0x04c` and `0x04d` resolve to no register, so `otp set -s 0x04a 0x2000 --ser S` writes that one raw row (PT `main.cpp:9726-9728`). `0x048` and `0x04b` resolve to the named RBIT-3 register and write all three copies unless `-c 1` comes first: `otp set -c 1 -s 0x04b 0x800 --ser S` (H). Without `-s`, a value lacking existing bits fails at PT `main.cpp:9715` ("Cannot clear bits"). `inject-copy` must use `-s`, and must add `-c 1` for 0x048 and 0x04b. Write this into §3 so that the fake models it, and so that it is not left to rediscovery.
- **A-M8 (§3.4).** `save -r` writes a raw image only for a `.bin` name (file type is taken from the extension, PT `main.cpp:3263-3265`; uf2 pads and wraps). The "size equals the range" check depends on that, so state that the temp file is named `*.bin`. Also, UI spec step 2 asks "whether the top sector stays", and the plan does not answer it.
- **A-M9 (§3.2 BOOT_FLAGS0 "every other bit").** The boot ROM sets BOOT_FLAGS0.ROLLBACK_REQUIRED (bit 11) itself when it boots an image carrying a rollback version (BR `varm_launch_image.c:114-129`). An image install between pre and post could then change "other bits". Record that the fork image and the blinky carry no rollback version, or exempt bit 11 explicitly with a reason.
- **A-M10 (§3.2 USB_BOOT_FLAGS parsing).** USB_BOOT_FLAGS bits 22 and 23 are both defined (OD:1121, 0xc0ffff). If retail sets both, picotool prints `(flipping raw value to 0x…)` while listing copies 1 and 2 (PT `main.cpp:9237-9241`), and the parser must not read that text as a value or a warning. The fake should emit it.

## Nit

- **A-N1 (§2).** "`picotool version` must print exactly 2.3.1" — it prints `picotool v2.3.1 (Linux, GNU-16.2.0, Release)` (BIN). Use `picotool version -s`, which prints `2.3.1`. Note that `picotool version 2.3.1` is a compatibility check that accepts a newer patch, not an equality test (PT `main.cpp:1727-1741`). The reference to "format per G1" is wrong, since G1 covers `otp get`.
- **A-N2 (§1 F13).** Plain `save` defaults to `-p` (program only, `Cannot determine the binary size…`), not a whole-flash guess. Only `save -a` and `erase` (default `-a`) guess.
- **A-N3 (§3.1).** `--ser` equals the USB iSerialNumber, which is CHIPID3..0 (BR `nsboot_usb_client.c:647-648`) **unless** white-label entry 6 (SERIAL_NUMBER_STRDEF) is valid (BR `:268-276`). Retail #2 matched (HARDWARE_INVENTORY l.91-93), so this is fine today. The retail compare (A-I5) is what keeps it so; say that.

## Things checked and found right

- The R line references: R:48, 58-60, 149-152, 583-585 (no write path), 166-171 (format comment), 717 and 829 (KEY_INVALID == 0), 753 ("majority-votes" CRIT1, wrong per BR 3-of-8), and `sh2-flash:142` (`846aa289…`) all point where the plan says.
- `erase -r 0x10000000 0x11000000` is in range: `flash_end` is 0x12000000, inclusive (PT `model/model.h:152-155, 303-306`). `erase -r`/`save -r <from> <to> <file>` and trailing `--ser` parse correctly (BIN).
- The OTP JSON written by `seal` is unchanged in 2.3.1, so R's `make_otp_json` and its assertions hold. 2.3.1 fixes the lone CHIPID1 read, and R's batching keeps working.
- UI spec items present in the plan: KEY_VALID 0x3, slots 2/3 empty and not valid, secure boot on, DISABLE_OTP_BOOT in all 3 copies, page locks 0x040404/0, KEY_INVALID 0xC only with hardening, FLASH_DEVINFO(+ENABLE) retail values, explicit 16 MB range with a boot-ROM refusal counting as condemned, an unreadable row counting as a mismatch, and the rehearsal profile with its CANNOT PROVE list. Build plan E3a items present: one pinned picotool, the DISABLE write and read-back, KEY_INVALID in R's `--sh2-verify-valid`, explicit-range erase/save, and the rehearsal profile. Gaps are listed above (A-I5, A-M1, A-M3, A-M4, A-M8).
