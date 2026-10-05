# IMPLEMENTATION PLAN — E3a: OTP tooling for Refugium, the rehearsal profile, and one picotool (F-701)

*Draft 5, 2026-10-05. Author: thread "mnemonic-engrave for Refugium". Baseline: mnemonic-engrave
`fde7841` (master after PR 6). Risk set (a) irreversible OTP writes and (b) keys: R0 to 0 C / 0 I
before code, a single implementer, then a mandatory adversarial review of the whole diff.*

Draft 2 folded R0 round 1: `design/agent-reports/e3a-plan-r0-a.md` (lens A, facts: 2 C / 5 I / 10 M /
3 N) and `e3a-plan-r0-b.md` (lens B, failure states: 1 C / 7 I / 10 M / 2 N). Draft 3 folds round 2:
`e3a-plan-r1-a.md` (0 C / 1 I / 8 M / 1 N) and `e3a-plan-r1-b.md` (0 C / 2 I / 10 M / 2 N). Draft 4
folds round 3: `e3a-plan-r2-a.md` (0 C / 3 I / 5 M / 2 N). Draft 5 folds round 4: `e3a-plan-r3-a.md`
(0 C / 1 I / 3 M / 2 N). Fold tables: §10-§13.

Source of the ask: `bg002h/refugium-wallet` `design/IMPLEMENTATION_PLAN_mr_gui_v1.md` at `f929084`,
phase E3a (and E3b, which this plan prepares but does not run); UI spec
`design/SPEC_ui_refugium_wallet.md` §4.4 ("Expected OTP state", "Engraver provisioning sitting").
Recon: `design/agent-reports/e3a-recon-otp-facts.md` (**OF**), `e3a-recon-picotool.md` (**PT**),
`e3a-recon-repo-map.md` (**RM**). The rehearsal script `scripts/pico2-bootkey-rehearsal.sh` is **R**.

Build-gate coverage: this plan carries no ```rust blocks, so `scripts/plan-build-gate.sh` covers
nothing. The gates are the e2e suite (§6), the argv probe against the real binary (§6.4) and the
bench rehearsal (§7). Lens A already ran the real picotool 2.3.1 (built here from nixpkgs-unstable
`a7868a72`) for the argv and `--ser` facts; §8 step 1 runs the e2e skeleton and the probe once before
dispatch.

## 0. Outcome

1. **One picotool**: 2.3.1 built against pico-sdk 2.3.1, from nixpkgs-unstable through a small
   `flake.nix` here (`packages.picotool`, `devShells.otp`). The fork's `nix develop` and the Sitting
   image (S1) take it as an input. `refugium-otp.sh` refuses any other picotool.
2. **`scripts/refugium-otp.sh`**, a new tool for the Refugium expected OTP state (UI spec §4.4): a
   read-only `check` of an explicit expected state, a read-only `capture` for H0, two write steps
   (`disable-otp-boot`, `invalidate-spare-keys`), explicit-range `erase-range`/`save-range`, and a
   rehearsal-only `inject-copy`. Every device call is bound to the CHIPID with `--ser`, through one
   argv builder.
3. **Two profiles**, `retail` and `rehearsal`, each verified against the board (slot 0's key) before
   any device write, never trusted from the command line alone.
4. **`KEY_INVALID` 0xC accepted** by R's `--sh2-verify-valid` and `--sh2-precheck` when asked for.
5. **A row-level picotool fake in Python** that enforces picotool's argument order, an e2e suite
   with named fault modes, CI, and a CI probe of the argv shapes against the real 2.3.1 binary.
6. **A bench procedure** for a 4 MB Pico 2 (no per-board go-ahead, plan §9 item 5) with two injected
   unequal copies the check must refuse, and an E3b runbook section for SeedHammer #1 that runs only
   on Brian's typed go-ahead naming the CHIPID and the step.

Not in E3a: running E3b; any write to a SeedHammer; the key model (E4); the Sitting image's
provisioning flow (lane S); the retail values themselves (H0 measures them with `capture`).

## 1. Facts (from source; lens A re-verified F1-F13 on picotool 2.3.1, pico-sdk 2.3.1, boot ROM A4)

| id | fact | source |
|---|---|---|
| F1 | BOOT_FLAGS0 rows 0x048/0x049/0x04a; BOOT_FLAGS1 0x04b/0x04c/0x04d; USB_BOOT_FLAGS 0x059/0x05a/0x05b; CRIT1 0x040-0x047; all raw 24-bit | OF 1b-1d, 1h |
| F2 | DISABLE_OTP_BOOT = BOOT_FLAGS0 bit 13; ENABLE_OTP_BOOT bit 14; FLASH_DEVINFO_ENABLE bit 5; ROLLBACK_REQUIRED bit 11 is set by the boot ROM itself | OF 2a-2d |
| F3 | KEY_VALID BOOT_FLAGS1 bits 3:0; KEY_INVALID bits 11:8; effective = KEY_VALID & ~KEY_INVALID; 0xC invalidates slots 2 and 3 only | OF 2e, 2f, 5 |
| F4 | BOOT_FLAGS0/1 and USB_BOOT_FLAGS are read by per-bit 2-of-3 vote; CRIT1 SECURE_BOOT_ENABLE is any-3-of-8 | OF 3, 3b |
| F5 | DISABLE_OTP_BOOT supersedes ENABLE_OTP_BOOT and only skips OTP boot | OF 4 |
| F6 | FLASH_DEVINFO 0x054 (ECC); CS0_SIZE bits 11:8, `4 KiB << n`, 0xc = 16 MiB; with ENABLE set and a smaller CS0, BOOTSEL flash ops beyond it are refused | OF 7a, 7b |
| F7 | USB_WHITE_LABEL_ADDR 0x05c (ECC) = row index of a 16-row ECC table; each STRDEF entry points to string rows outside the table (length low 7 bits, ×2 if bit 7); USB_BOOT_FLAGS bit 22 enables the table, bits 0-15 are per-entry valid bits | OF 1i; lens A A-I5 |
| F8 | Page locks PAGE1_LOCK0/1 0xf82/0xf83, PAGE2_LOCK0/1 0xf84/0xf85; retail LOCK1 0x040404, LOCK0 0; writes refused if LOCK_S or LOCK_BL ≠ 0 or KEY_W set | OF 6a-6e |
| F9 | `otp get` on a named redundant register votes per bit; when copies differ it prints `RAW_VALUE=` with every copy and `(WARNING - REDUNDANT ROWS AREN'T EQUAL)` | OF P1 |
| F10 | `-c 1` makes a named read return the first row's own raw content — **only when `-c` is in its declared position** (F14) | lens A A-C1 |
| F11 | `otp set` computes the new value from copy 0 and writes it to all copies, row by row; a copy holding a bit outside the new value fails that row after earlier rows are burned; a copy that is a subset is healed | OF P3; BR `varm_otp.c:402-407` |
| F12 | 2.3.1 vs 2.2.0-a4: names unchanged; ECC `VALUE` prints 16-bit; lone CHIPID1 read works; `seal --sign` adds EXTRA_SECURITY and a VECTOR_TABLE item | PT |
| F13 | `erase` (default `-a`) and `save -a` guess the flash size and refuse erased flash; plain `save` defaults to `-p`; `erase -r <from> <to>` rounds the range to 4 KiB, `save -r` to a `.bin` uses the exact range; `save` writes raw only to a `.bin` name and creates/truncates it before reading (a refused read leaves a partial file) | PT D15-D17; lens A A-N2, A-M8; G-facts G3 |
| F14 | picotool's `cli.h` matches option groups **in declaration order**. `otp get` must be spelled `otp get [-c N] [-r] [-e] [-n] [--ser S] <selector…>`; `otp set` as `otp set [-c N] [-r] [-e] [-s] <selector> <value> [--ser S]`. A misplaced option silently becomes a selector that matches no row, and the call then runs on whatever device is attached. (This is also what F-619 measured: `-n -c 1` added CHIPID1 as a selector.) | lens A A-C1, measured on the 2.3.1 binary |
| F15 | `--ser` must be CHIPID3..0 in uppercase (e.g. `09F50BF63E8D6F46`); picotool `strcmp`s it; a miss exits 249 with "…found with serial number X." (2.3.1). The USB serial is the CHIPID unless white-label entry 6 is valid | HARDWARE_INVENTORY; lens A G4, A-N3 |
| F16 | `0x049`, `0x04a`, `0x04c`, `0x04d` name no register, so `otp set -s 0x04a <v>` writes that one row; `0x048`, `0x04b`, `0x040` resolve to the named register and write all copies unless `-c 1` comes first. Without `-s`, a value lacking existing bits fails ("Cannot clear bits"). An unnamed row is written raw (no ECC) unless `-e` is given (`bEcc = ecc && !raw`) | lens A A-M7, A2-N1; PT `main.cpp:8990-8992, 9715-9732` |
| F17 | `picotool version -s` prints the bare version (`2.3.1`) | lens A A-N1 |
| F18 | When stdout is not a terminal, picotool word-wraps **stdout** at 80 columns (terminal width otherwise); `ERROR:` lines and progress bars are never wrapped. A token with no spaces is never broken: CRIT0/CRIT1's 8-copy `RAW_VALUE=` list is one 124-character line, often run together with its warning (`…;0x000001(WARNING - REDUNDANT ROWS AREN'T EQUAL)`). What does wrap: descriptions, the `--ser` miss message (it breaks before the serial), and a 3-copy `RAW_VALUE` line carrying the `(flipping raw value to 0x%08x)` note, whose warning moves to a continuation line indented 14 spaces. Parsers join continuation lines and match the warning with or without a preceding space | lens A A2-M1; G-facts G1, PT `main.cpp:10178-10186` |
| F20 | **G1, `otp get` layout** (literal blocks in `e3a-g-facts.md`): `RAW_VALUE=` prints only when copies differ or an ECC row fails its check (`(WARNING - ECC IS INVALID)`), never under `-c 1`; with no RAW_VALUE line an empty line precedes `VALUE`; ECC `VALUE` is 4 hex digits, 6 under `-r`; field values are bare lowercase hex (`= a`); rows print in row order with duplicates merged; a selector matching nothing prints nothing and exits 0 (so the `otp list` fingerprint and every read test the output, never the exit code alone) | G-facts G1, compiled from 2.3.1 source; `otp list -n` matched the binary line for line |
| F21 | **`otp set` exits**: success prints `ROW 0x%04x  OLD_VALUE=0x%06x` (copy 0), the description and a `field` line; "Cannot clear bits in OTP row(s): current value %06x, new value %06x" exits 248; a write failing part-way prints `ERROR: Attempted to clear bits in OTP row(s)` and exits 248 with earlier rows written; any other boot-ROM refusal exits 157 | G-facts G1 |
| F22 | **G3, flash refusals**: a range beyond CS0 prints `ERROR: The RP2350 device returned an error: permission failure` and exits 157; `erase` goes sector by sector, so sectors below CS0 are already erased when it fails; `load -v` failing verify prints `  FAILED` and `ERROR: The device contents did not match the file`, exit 245; every flash `load`, `erase -a` and `save -a` first probes flash at +8 MiB, so with FLASH_DEVINFO enabled, CS0 ≤ 8 MiB and unerased flash they exit 157 before writing. The tool treats any non-zero exit as a refusal (more conservative than keying on 157) | G-facts G3, boot ROM A4 source |
| F23 | **G5, several boards**: `info` never refuses and exits 0; with two or more boards it prints `Multiple RP-series devices in BOOTSEL mode found:` and a header and dashed line per device; `--ser` matching exactly one board gives normal output. `otp get/set`, `erase`, `save`, `load` with two or more boards print `ERROR: Command requires a single RP-series device to be targeted.` and exit 248 | G-facts G5 |
| F19 | `otp set -s` writes `(value << shift & mask) \| (old & ~mask) \| old`, with `old` = copy 0's raw row: **every bit copy 0 holds is written to every copy**, including bits inside the target field that the target value lacks | lens B B2-I1, PT `main.cpp:9693-9712` |

G1, G3 and G5 are answered (F20-F23) from 2.3.1 source compiled with fake OTP contents and the boot
ROM A4 source: `design/agent-reports/e3a-g-facts.md`, which cites `main.cpp:<line>` for each line the
parsers and the fake must match. Still unverified on silicon: the G1 blocks (compared at R1, which
captures `otp get -n` of 0x040, 0x048, 0x04b, 0x054 with and without `-r` and `-c 1`); the G3 refusal
(needs a board with FLASH_DEVINFO set; not reproduced on the rehearsal Pico); G5's per-device `info`
body (needs two boards); the boot ROM revision of the real boards.

## 2. The picotool pin

- `flake.nix` + `flake.lock` at the repo root. The nixpkgs input uses the URL form that fetches
  through this sandbox's proxy: `git+https://github.com/NixOS/nixpkgs?ref=nixos-unstable&shallow=1`,
  locked to a rev that carries picotool 2.3.1 and pico-sdk 2.3.1 (lens A built `a7868a72` here;
  nixpkgs already applies the mbedtls substitution and installs the udev rule, so no overlay).
  Outputs: `packages.<system>.picotool`; `devShells.<system>.otp` with picotool, openssl, jq,
  python3, xxd, **tinygo and go** (R's phases need them, R:131-141). Systems: x86_64-linux,
  aarch64-linux; darwin is listed but unverified.
- `refugium-otp.sh` requires `picotool version -s` = `2.3.1` exactly, and an `otp list` fingerprint
  of the SDK it was built with (`MAC0` present, `DISABLE_BOOTSEL_EXEC2` absent, PT). No override.
- R is **not** gated to 2.3.1: it accepts `2.2.0-a4` or `2.3.1` (its parsers already mask ECC to 16
  bits) and prints which, so its SeedHammer modes keep working from the fork's shell until the fork
  moves to this flake. R's format comment (R:166-171) is updated for both.
- `design/PICOTOOL_PIN.md`: version, nixpkgs rev, why 2.3.1, the `seal --sign` change, how the fork
  and the Sitting image consume the flake (`inputs.mnemonic-engrave.packages.${system}.picotool`).
  The fork thread gets the pin; its devshell change is its own PR and is **not** a prerequisite of
  this plan (§7 uses `.#otp` with `SEEDHAMMER_DIR` pointing at the fork checkout).

## 3. `scripts/refugium-otp.sh`

A new script: R's header promises its SH2 modes have no write path (R:48, 58-60, 149-152, 583-585),
and that stays true. Shared code moves to `scripts/lib/otp-read.sh`, sourced by both.

### 3.0 Shared library and the error mechanism

- **One argv builder**, `pt_otp_get`/`pt_otp_set`/`pt_flash`, emits every picotool call in F14's
  canonical order with `--ser` in its declared place. No call site writes picotool argv by hand.
  R uses the builder too, with an empty serial (unchanged behaviour: no `--ser`).
- **Non-dying readers.** Each helper (`otp_field`, `read_rows`, `read_row_raw24`, `read_slot`,
  `chipid`, `check_page_locks`) becomes `try_<name>`: returns 0/1, sets its globals, and on failure
  sets `ERR` to the message. R keeps its names as thin wrappers that die on failure and otherwise
  print what R's call sites capture (`otp_field() { try_otp_field "$@" || die "$ERR"; printf '%s'
  "$OTP_FIELD_VAL"; }`), so R's call sites (`KV="$(otp_field …)"`, R:348, 704, 791) and messages are
  unchanged; R's old e2e checks it. `refugium-otp.sh` never calls a reader inside `$(...)`.
- `refugium-otp.sh` uses only the `try_` forms. `check` records each row as PASS or FAIL in arrays in
  the main shell (`ROW_NAMES`, `ROW_RESULTS`), never in a subshell. RESULT is PASS only if FAIL count
  is 0 **and** the set of row names compared equals the required set, which is derived from the
  profile and the matched retail entry, never from what the board reports; a required row never
  compared is a FAIL, and a row compared twice is a tool error.
- Exit codes: 0 pass; 1 usage or environment (bad flags, wrong picotool, no device); 2 state refused
  (any FAIL, any unreadable row); 3 a write was issued and anything after it failed: a non-zero exit
  of the write itself, any read failure, WARNING or mismatch in the post-write check. Exit 3 is used
  only when `otp set` was actually invoked; a branch that wrote nothing exits 2. Exit 3 prints: "The
  write may be partial. Re-run this same command once: it only adds bits and heals a missing copy. If
  the re-run exits non-zero for any reason, stop and do not use this board for seeds."
  (`inject-copy` has its own exit-3 text, §3.6.)
- `--log <file>`: the tool writes its own full transcript, so the operator need not pipe into `tee`
  (which would hide the exit status).

### 3.1 Command line

```
refugium-otp.sh check   --profile P --ser S [--rehearsal-key K --rehearsal-slot1-key K1]
                        --slot1 empty|key|valid --disable-otp-boot 0|1 --key-invalid 0|c
refugium-otp.sh capture --ser S --out <file>
refugium-otp.sh disable-otp-boot      --profile P --ser S [--rehearsal-key K] [--execute]
refugium-otp.sh invalidate-spare-keys --profile P --ser S [--rehearsal-key K] [--execute]
refugium-otp.sh erase-range --profile P --ser S [--rehearsal-key K] [--probe-only] [--execute]
refugium-otp.sh save-range  --profile P --ser S [--rehearsal-key K] --out <file.bin>
refugium-otp.sh inject-copy --profile rehearsal --ser S --rehearsal-key K --case bf0-copy3|bf1-copy0 [--execute]
```

- `--ser` is required everywhere, must be 16 hex characters and is uppercased; it must equal the
  CHIPID read from the board (CHIPID3..0, F15). Exactly one RP2350 in BOOTSEL: `info` is run without `--ser`, its output
  must not contain `Multiple RP-series devices in BOOTSEL mode found:` and must show one device (F23:
  the exit code cannot tell one board from two).
- Under `rehearsal`, `--rehearsal-key` (slot 0's key: R's `rehearsal-work/factory-key.pem`) is
  required on every device command, and `--rehearsal-slot1-key` (R's `my-key.pem`) on every command
  that judges slot 1 (`check`, `disable-otp-boot`, `invalidate-spare-keys`); no defaults; the hashes used
  are printed. Both are refused under `retail`. (The two write lines take `--rehearsal-slot1-key` too;
  omitted above for width.)
- **Profile identity gate**, run before anything else touches the device, for every command except
  `capture`: under `retail`, slot 0 = `SH_SIGNKEY_HASH`; under `rehearsal`, slot 0 = the hash of
  `--rehearsal-key`, slot 0 ≠ `SH_SIGNKEY_HASH`, and the CHIPID is not one of the SeedHammer CHIPIDs
  listed in `otp-read.sh` (from HARDWARE_INVENTORY), stored in `--ser` form and compared after the
  same conversion as `--ser`. A failure exits 2 before any write or erase.
- Write commands are dry-run unless `--execute`; then a typed confirmation `BURN <step> <CHIPID>`.

### 3.2 `check` — the expected state

`check` takes the whole expected state explicitly (no stage, no defaults): `--slot1`,
`--disable-otp-boot`, `--key-invalid`. Each row is read and listed PASS/FAIL; nothing short-circuits
after the identity gate.

Slot 1 and KEY_VALID are judged together:

| `--slot1` | slot 1 | KEY_VALID |
|---|---|---|
| `empty` | all zero | 0x1 |
| `key` | the fork key (retail: `846aa289…`, moved from `sh2-flash:142` into `otp-read.sh`; rehearsal: R's `my-key.pem` hash) | 0x1 |
| `valid` | the fork key | 0x3 |

| row(s) | read | retail | rehearsal |
|---|---|---|---|
| CHIPID0-3 | `read_rows` | = `--ser` | same |
| CRIT0 ×8, CRIT1 ×8 | copy rule; `-c 1` named reads of 0x038 and 0x040 | equal the recorded entry (SECURE_BOOT_ENABLE 1, DEBUG_DISABLE clear) | CRIT1 = 0x000001 in all 8, CRIT0 = 0 in all 8 |
| slot 0 | `read_slot` | identity gate | identity gate |
| slot 1 + KEY_VALID | above | joint table | joint table |
| slots 2, 3 | `read_slot` | all zero | all zero |
| BOOT_FLAGS1 ×3 | copy rule | KEY_INVALID = `--key-invalid`; every bit outside KEY_VALID and KEY_INVALID equals the recorded entry | KEY_INVALID = `--key-invalid`; every bit outside KEY_VALID and KEY_INVALID is 0 |
| BOOT_FLAGS0 ×3 | copy rule | ENABLE_OTP_BOOT 0; DISABLE_OTP_BOOT = flag; bit 11 (ROLLBACK_REQUIRED) either value; every other bit equals the recorded entry | ENABLE_OTP_BOOT 0; DISABLE_OTP_BOOT = flag; bit 11 either; other bits 0 |
| FLASH_DEVINFO | raw 24-bit | equals the recorded entry; if ENABLE is set, CS0_SIZE = 0xc | FLASH_DEVINFO_ENABLE 0 |
| USB_BOOT_FLAGS ×3 | copy rule | equals the recorded entry | copies equal; value not compared (CANNOT PROVE) |
| USB_WHITE_LABEL_ADDR, its 16-row table, every valid STRDEF's string rows | raw 24-bit | equal the recorded entry | not compared (CANNOT PROVE) |
| PAGE1/2_LOCK0/1 | raw | LOCK1 0x040404, LOCK0 0 exactly | same |

**Copy rule** (BOOT_FLAGS0/1, USB_BOOT_FLAGS ×3; CRIT0, CRIT1 ×8): (a) the raw reads of the unnamed
copies and the `-c 1` read of the first row are all equal; (b) the named read printed no WARNING; (c)
the named vote equals the expected value; (d) if the named read printed `RAW_VALUE=`, its copies must
equal the bare and `-c 1` reads copy for copy, or the row is refused (this keeps the rule honest if
F10 were false on silicon). (a) is what catches the first copy being the odd one, the case F-619
called blind. ECC rows are always read raw (24-bit) so a corrected single-bit change is visible.

**Retail values** live in `design/hardware/retail-otp.json`: a list of entries, each holding the
cells the table marks "recorded entry" (CRIT0, CRIT1, BOOT_FLAGS0 bits outside DISABLE_OTP_BOOT,
ENABLE_OTP_BOOT and bit 11, BOOT_FLAGS1 bits outside KEY_VALID and KEY_INVALID, FLASH_DEVINFO,
USB_BOOT_FLAGS, the white-label address, table and strings), the capture file it came from (committed
under `design/hardware/captures/`), that file's sha256, the date and the unit's history (e.g. "has run
fork firmware"). CHIPID, slots, KEY_VALID, KEY_INVALID and DISABLE_OTP_BOOT are never taken from an
entry. A unit passes only if **the same** entry matches every recorded cell. Each entry is schema-checked before use (every row present,
hex shape, capture file present with matching sha256); a malformed file is exit 1. The path can be
overridden only by `REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY`: with it set, every write command and
`--execute` are refused, and `check`'s RESULT reads `RESULT: TEST ENTRY — not a retail check`, never
PASS.
With no entries, a retail `check` refuses: "no recorded retail values; run `capture` on a retail
unit (H0)". Test fixtures live under `scripts/test/fixtures/`, never in `design/hardware/`.

CANNOT PROVE (rehearsal only, printed above RESULT): slot 0 holds SeedHammer's key; the retail
white-label, USB_BOOT_FLAGS and FLASH_DEVINFO values; flash above 4 MB; SeedHammer's on-device sealing
(bootkey-rehearsal-fidelity-residue (b)). The rehearsal RESULT reads `RESULT: REHEARSAL PROFILE PASS —
not a retail check`.

### 3.3 Write steps and the heal rule

Each write command reads the board, works out its own pre-state and post-state, and never needs the
operator to state them.

- **Pre-state is the full §3.2 check**, with slot 1 `valid` and the other row's flag derived from the
  board, restricted to its legal values: `disable-otp-boot` accepts KEY_INVALID 0 or 0xC only;
  `invalidate-spare-keys` accepts DISABLE_OTP_BOOT 0 or 1 only. Any other value (e.g. KEY_INVALID 0x1
  in all copies) is exit 2 with no write. The derived row is judged by the copy rule; the write's
  target row is judged as below. A retail unit whose CRIT1, page locks or white-label rows mismatch is
  refused before any write.
- **E and T.** T = the bits the write sets. E = the expected post-write value of the target row,
  computed from the expected state, never from copy 0: for BOOT_FLAGS0, the recorded entry's bits
  (retail) or 0 (rehearsal), DISABLE_OTP_BOOT 1, ENABLE_OTP_BOOT 0, and bit 11 equal to the value
  every copy agrees on (copies disagreeing on bit 11 is exit 2); for BOOT_FLAGS1, KEY_VALID 0x3,
  KEY_INVALID 0xC, every other bit the recorded entry's (retail) or 0 (rehearsal).
- **Reading the target row's copies**, in every branch: the bare reads of the unnamed copies, the
  `-c 1` read of the first row, and the named read. The copies count as **equal** only if the named
  read printed neither a WARNING nor a `RAW_VALUE=` line and all three per-copy reads agree. Otherwise
  the row is **unequal** and only the heal rule can admit it.
- **Branches, in this order:**
  1. copies equal and equal to E → no write; run the post-check; a failure there is exit 2 (no write
     was issued), never exit 3;
  2. copies equal and equal to E & ~T, or copies unequal and the heal rule admits them → write;
  3. anything else → exit 2, no write.
  Exit 3 is used only when `otp set` was invoked.
- **`disable-otp-boot`.** T = bit 13 of BOOT_FLAGS0. Write: `otp set -s BOOT_FLAGS0.DISABLE_OTP_BOOT
  0x1 --ser S` (F14 order). Post: the full check with `--disable-otp-boot 1`.
- **`invalidate-spare-keys`.** T = bits 10-11 of BOOT_FLAGS1 (KEY_INVALID 0xC). Extra pre-state: slots
  2 and 3 zero; KEY_VALID bits 2-3 clear. Write: `otp set -s BOOT_FLAGS1.KEY_INVALID 0xc --ser S`.
  Post: the full check with `--key-invalid c`. The confirmation adds: "Slots 2 and 3 can never hold a
  key after this."
- **Heal rule** (the exact F11/F19 safety condition, and the admission rule for a re-run after exit 3).
  A row whose copies are unequal is admitted for this write only if all of these hold:
  1. every copy, copy 0 included, is a subset of E (no bit outside E, so no stray bit inside the
     target field either: a KEY_INVALID bit 8 or 9 is refused);
  2. every copy & ~T equals E & ~T (the copies differ only in target bits; with T ⊆ E this implies
     condition 1, which is kept as the stated bound);
  3. copy 0 | T equals E (picotool computes the write from copy 0, F19, so this is what lands in every
     copy);
  4. the named read printed `RAW_VALUE=`, and its copies equal the bare reads and the `-c 1` read copy
     for copy (F9); a missing RAW_VALUE or any disagreement (which means F10 does not hold on this
     board) refuses the row (exit 2).
  Conditions 1-3 use the per-copy reads (bare reads and `-c 1`), never RAW_VALUE's list; RAW_VALUE
  enters only through condition 4. The tool prints the copies and the words "healing an unequal copy". The read-only `check` never heals:
  it refuses any unequal copy.
- The two writes are order-independent; UI spec §4.4's order (precheck, erase, key, image, re-check,
  DISABLE_OTP_BOOT, optional KEY_INVALID, check) is the Sitting image's job (lane S).

### 3.4 Flash range

- Range from the verified profile: retail `0x10000000`-`0x11000000` (16 MB, which includes the top
  sector, so the top sector is erased: UI spec step 2's question answered); rehearsal
  `0x10000000`-`0x10400000`.
- `erase-range`: identity gate; `picotool erase -r <from> <to> --ser S`; then `save -r <from> <to>
  <tmp>.bin --ser S`; require the file size equals the range and every byte is 0xFF (counted with `tr
  -d '\377'` into a file and `stat`, no pipe into `grep -q`, F-695). Under `retail`, an **alias probe**
  runs first, and only after `--execute` and the typed confirmation (it writes flash): a 4 KiB marker of
  random bytes, generated per run and printed by sha256, written with `load -v <marker>.bin -o
  0x10000000 --ser S`; then 4 KiB read at `+4 MiB`, `+8 MiB`, `+12 MiB`; any copy of the marker means the
  flash is smaller than 16 MB (refuse, condemned); then erase as above. A non-zero exit from the `load`
  is condemned too. `erase-range --probe-only` runs just the probe (any profile, same gating), for the
  bench (§7 R4). A dry run records no `load` argv (case 8). A non-zero
  exit from `erase` or `save` is "the boot ROM refused this range: treat the engraver as an unknown
  image (condemned)", exit 2.
- A `save` that exits non-zero leaves a partial file (F13): the tool deletes it and never reads it.
- `save-range`: identity gate; the same read into `--out` (must end `.bin`); prints its sha256.

### 3.5 `capture`

Read-only, no profile. Writes JSON: CHIPID, `picotool version -s`, the raw 24-bit value of every row
in §3.2 (all copies, both page-lock pages, CRIT0, CRIT1 ×8), the white-label table and every valid
string's rows, decoded strings for reading, and the `otp list` fingerprint. Refuses to overwrite
`--out`. H0 runs it on SeedHammer #1; a PR then adds the capture file and its entry.

### 3.6 `inject-copy` (rehearsal only)

Exactly two cases, no free `--bits`: `bf0-copy3` writes DISABLE_OTP_BOOT into row 0x04a only
(`otp set -s 0x04a 0x002000 --ser S`, F16); `bf1-copy0` writes KEY_INVALID slot 3 into the first
BOOT_FLAGS1 copy only (`otp set -c 1 -s 0x04b <copy0|0x000800> --ser S`, F14, F16). Pre: identity gate
(rehearsal), the target copies equal and the bit clear. After the write it reads every copy of the row
(unnamed rows bare, the first row with `-c 1`, cross-checked with RAW_VALUE[0]) and requires: the target
copy = old | bit, every other copy unchanged. Anything else is exit 3 with inject-copy's own text: "stop
the rehearsal; do not re-run".

## 4. R changes

- Source `scripts/lib/otp-read.sh` and the argv builder; R's own messages and exits are unchanged.
- `--sh2-verify-valid` and `--sh2-precheck` take `--expect-key-invalid 0|c` (default 0); R:717 and
  R:829 compare with it; KEY_INVALID bits 0-1 must be 0 either way.
- Picotool version: 2.2.0-a4 or 2.3.1 (§2).
- R:753 comment: CRIT1 SECURE_BOOT_ENABLE is any-3-of-8 (OF 3b). R:166-171 comment: both formats.
- shellcheck warnings in R fixed (SC2010 at R:1134, SC2006 at R:851).

## 5. Docs

- RUNBOOK: prerequisites name `nix develop .#otp` with `SEEDHAMMER_DIR`; a section "Refugium steps
  (E3b)": `check --profile retail --ser S --slot1 valid --disable-otp-boot 0 --key-invalid 0`,
  `disable-otp-boot`, the optional `invalidate-spare-keys` (only if plan §9 item 14 allows it on a
  test board), the final `check`; each IRREVERSIBLE step needs Brian's typed go-ahead naming the
  CHIPID and the step; exit 3 means re-run the same command once, and that one re-run is
  pre-authorised by the step's go-ahead; a second re-run needs a new go-ahead, and a re-run that exits
  non-zero for any reason means stop and do not use the board for seeds; no SeedHammer write before E4.
- HARDWARE_INVENTORY: a "Retail OTP rows" section pointing at `design/hardware/retail-otp.json`.
- FOLLOWUPS: F-619's correction (the `-c 1` measurement was an argument-order artefact, F14); F-701
  progress. The F10 hardware reading (§7 R9) is written to the result file and to F-619.

## 6. Tests

### 6.1 The fake (`scripts/test/fake_picotool.py`)

Row-level RP2350 simulator. State: a JSON file of raw rows (ECC rows as 24-bit raw), a flash image of
a configured physical size, a device list. Implements, with G1's exact text and F14-F17: `version
[-s]`, `info` (one or more devices, G5), `otp list` (the fingerprint names), every output
line word-wrapped at 80 columns as picotool does when stdout is not a tty (F18), `otp get` (named
registers with vote, RAW_VALUE, WARNING, the flipping note; numeric rows resolving to named registers
for 0x040/0x048/0x04b/0x059; `-c`, `-r`, `-e`), `otp set` (all copies row by row with F11's failure
after earlier rows; `-c 1`; unnamed raw rows; "Cannot clear bits"), `otp load`, `erase -r`/`save -r`
(refused beyond CS0 when FLASH_DEVINFO_ENABLE is set; aliasing when the physical size is below the
range), `load` of a marker, `--ser` (uppercase match, exit 249 and F15's text on a miss).

**It enforces F14's argument order**: an option after a later group, or a selector that is not a row
number, a register name or `REG.FIELD`, exits 99 "fake-picotool: argv out of order". Any unimplemented
subcommand or flag exits 99. OTP bits only set.

Fault modes (env): `SUPPRESS_WARNING=1` (the named read prints only the vote: no WARNING and no
`RAW_VALUE=`, a picotool output regression); `SUPPRESS_RAW_VALUE=1` (WARNING printed, no
`RAW_VALUE=`); `COPIES_IGNORED=1` (`-c 1` returns the vote, F-619's old reading); `FAIL_SET_AFTER=N`
(`otp set` burns N rows, then exits 248 with F21's text); `FAIL_READ_AFTER_WRITE=1`; `DEVICES=2` (F23 output and exits);
`FAKE_REQUIRE_SER=1` (any device-touching call without `--ser` exits 99; `run-e2e-otp.sh` always sets
it). The fake appends every argv to an argv log the cases can assert on. R's old fake stub is updated
to answer `version -s` with `2.3.1`.

### 6.2 `scripts/test/run-e2e-otp.sh` cases (each asserts exit code, a message regex, and for refusals
that the state file is byte-identical afterwards)

1. Retail-shaped state with a fixture entry, `check` PASS; rehearsal-shaped state, `check` PASS.
2. Each §3.2 row wrong, one at a time → exit 2 naming the row; two rows wrong → both named. The two
   bad slot-1/KEY_VALID combinations (empty slot 1 with 0x3; fork key with 0x1 under `--slot1 valid`).
3. Unequal copies: each of the 3 copies of BOOT_FLAGS0, BOOT_FLAGS1, USB_BOOT_FLAGS, and copies 0 and
   7 of CRIT1, odd one at a time → `check` exit 2. The same with `SUPPRESS_WARNING=1` → still exit 2. CRIT0 copy 0
   odd → exit 2.
   3c. Copy 0 odd with `COPIES_IGNORED=1` and `SUPPRESS_RAW_VALUE=1` (only the WARNING can refuse)
   → exit 2.
4. Writes: dry-run leaves state identical; `--execute` with the right confirmation writes and passes;
   wrong confirmation, `--ser` mismatch (exit 1 before any write), `DEVICES=2`, retail with no entries →
   refused, state identical. Both orders of the two writes pass.
5. Heal: each single-copy subset partial (for both rows) + the write → heal message, post PASS; a copy
   holding a bit outside the target → exit 2, state identical. A stray KEY_INVALID bit 8, and
   separately bit 9, in copy 0 only → `invalidate-spare-keys` exit 2, state identical. With
   `COPIES_IGNORED=1`, copies 0x103/0x003/0x003 (copy 0 holds a bit the others lack) →
   `invalidate-spare-keys` exit 2, state identical (the RAW_VALUE cross-check is the only refusal).
   5d. Copies 0x903/0x103/0x103 (a stray bit in every copy, T partial) → `invalidate-spare-keys` exit 2,
   state identical. 5e. KEY_INVALID 0x1 in all three copies → `disable-otp-boot` exit 2, state
   identical; DISABLE_OTP_BOOT copies disagreeing → `invalidate-spare-keys` exit 2. With `COPIES_IGNORED=1`,
   copies 0x803/0x003/0x003 → `invalidate-spare-keys` exit 2, state identical. 5f. BOOT_FLAGS0 bit 11
   (ROLLBACK_REQUIRED) set in all three copies → `disable-otp-boot --execute` writes, post PASS, bit 11
   still set in every copy; the same after `bf0-copy3` (bit 13 in copy 3 only) → heal, post PASS. Bit 11 in one copy only →
   `disable-otp-boot` exit 2, state identical. Bit 11 set in all three copies →
   `invalidate-spare-keys --execute` writes, post PASS. A no-write branch whose
   post-check fails → exit 2, not 3.
6. Interrupted write: `FAIL_SET_AFTER=1` and `=2` → exit 3 with the re-run text; re-run → heal → PASS.
   `FAIL_READ_AFTER_WRITE=1` → exit 3.
7. Identity gate: `erase-range --profile rehearsal` and `inject-copy` on a retail-shaped state → exit 2
   before any erase/set argv is recorded; rehearsal on a CHIPID in the SeedHammer list → exit 2 (7b: also with `--rehearsal-slot1-key`
   given, on `check` and the two write commands).
8. Flash: argv carries `-r` and `--ser`; FLASH_DEVINFO CS0 8 MB under retail → condemned exit 2; a byte
   left non-0xFF → exit 2; physical flash 4 MB under retail → alias probe refuses; a dry run records no `load` argv; the
   probe's `load` failing → condemned.
9. picotool 2.2.0-a4, or 2.3.1 with the wrong `otp list` fingerprint → exit 1 before device access.
10. `inject-copy`: both cases produce exactly the intended copy; with the builder mutated to drop `-c 1`
    (all copies written) → post-injection check exit 3.
11. CHIPID test vector: rows of `0x09f50bf63e8d6f46` → `--ser 09F50BF63E8D6F46`; lowercase `--ser`
    accepted and uppercased.
12. Replay: the real 2.3.1 transcripts committed in `scripts/test/fixtures/transcripts/` (from §7 R1)
    are fed through the parsers and must parse to the recorded values. Until the bench PR the directory
    is empty and the case passes vacuously; once `scripts/test/fixtures/transcripts/BENCH_RUN` exists,
    an empty or partial directory (R1 captures, R5 and R7 refusals) fails.
13. Argv log: in every case, every device-touching argv the fake recorded carries `--ser <S>`.

### 6.3 Mutations (run once by the implementer; the report shows each mutation's red run: case id and
failing assertion)

| mutation | killed by |
|---|---|
| drop copy rule (a) | case 3, copy 2 odd, with `SUPPRESS_WARNING=1` |
| drop the `-c 1` read from (a) | case 3, copy 0 odd, with `SUPPRESS_WARNING=1` |
| drop the WARNING trap (b) | case 3c (`SUPPRESS_RAW_VALUE` leaves only (b)) |
| drop `--ser` from the builder | `FAKE_REQUIRE_SER=1` (exit 99) and case 13 |
| builder emits `-n` before `-c` | fake exit 99 (argv order) |
| drop the post-write check | case 6 `FAIL_READ_AFTER_WRITE` |
| E replaced by copy 0 \| T in conditions 1-3 | case 5d |
| drop condition 4 alone (conditions 1-3 still on the per-copy reads) | case 5, 0x803/0x003/0x003 under `COPIES_IGNORED=1` |
| E's bit 11 fixed at 0 instead of the agreed value | case 5f |
| drop the RAW_VALUE cross-check (cond 4 and the equal-copy test) | case 5, 0x103/0x003/0x003 under `COPIES_IGNORED=1` |
| derived flag accepts any equal value | case 5e |
| identity gate skipped for erase | case 7 |

Equivalent mutants, with no row: "heal accepts a superset copy" and "heal judged by field". Because T ⊆ E,
condition 2 implies condition 1, so dropping or loosening condition 1 alone changes no outcome; case 5's
superset and stray-bit cases still run as behaviour tests.

### 6.4 CI

- `run-e2e-otp.sh` and `shellcheck -x -S warning` (on `refugium-otp.sh`, `lib/otp-read.sh` and R) run
  as steps inside the existing required job `test (rust + go)`, so they gate merges.
- Job `picotool argv probe`, with job-level `permissions: contents: read`: installs nix (an install
  action pinned by commit SHA), `nix build .#picotool`, then runs every argv shape the builder emits
  (generated by the builder's print mode, not written by hand) with `--ser ZZPROBE00000000` and no
  device attached, and requires the 249 exit and, after joining wrapped lines, "with serial number
  ZZPROBE00000000." (proving `--ser` was parsed, F14/F15/F18); plus `version -s` = 2.3.1 and the `otp
  list` fingerprint. The e2e asserts that every argv shape in the fake's argv log is in the probed
  set; a shape is the argv with the serial, every hex or decimal value and every file path replaced by
  placeholders (`<S>`, `<N>`, `<F>`), so the builder's print mode and the argv log normalise the same
  way. The thread asks Brian, through the Merging PRs thread, to add this job as a required check.
- R's existing e2e (`run-e2e.sh`, old fake answering `version -s` with 2.3.1) is not in CI (it needs
  TinyGo and the fork); it runs in `.#otp` at §8 step 1 and at the bench (R0).

## 7. Bench rehearsal (plain Pico 2, 4 MB, consumable; Brian at 13764k)

No per-board go-ahead (plan §9 item 5); every write is still bound to the CHIPID. Use `--log` for
every step under `rehearsal-work/e3a-<CHIPID>/`; summary in `design/HARDWARE_RESULT_<date>_e3a.md`.

- R0. `nix develop .#otp` with `SEEDHAMMER_DIR` set; `picotool version -s` = 2.3.1. Run R's old e2e.
  Then, with no board attached: build the blinky to a scratch path with R's own tinygo command
  (`build_blinky`), run `sign-firmware.sh` on it and require `picotool info -a` →
  `signature: verified` (picosign on 2.3.1's `seal` output, F12) and
  `load map entry 0: Clear 0x20000000->0x20082000` (both asserted by `sign-firmware.sh` step 7 since
  the seal-patch plan, `IMPLEMENTATION_PLAN_e3a_picotool_seal_patch.md`). Also run
  `scripts/test/seal-clear-test.sh "$(command -v picotool)"` and require the `.#otp` picotool's store
  path to be the patched one recorded in `design/PICOTOOL_PIN.md`. Any failure stops here.
- R1. Board in BOOTSEL: `capture` (read-only) before any write; commit its `otp get` transcripts as the
  replay fixtures (§6.2 case 12) and diff the line formats against the fake. Before R2, check from the
  capture every cell the §3.2 rehearsal column fixes, at its pre-R2 value: CRIT0 0 and CRIT1 0 in all
  copies (R2 sets CRIT1 later); BOOT_FLAGS0 0 outside bit 11; BOOT_FLAGS1 0; FLASH_DEVINFO_ENABLE 0;
  slots 0-3 zero; page locks LOCK1 0x040404, LOCK0 0; every copied row's copies equal. Any difference stops the rehearsal and
  returns to the implementer.
- R2. R's phases 0, 1, 2, 3 (`--execute`), 4 — the A/B proof needs phase 3 (R:1099).
- R3. `check --profile rehearsal --ser S --rehearsal-key rehearsal-work/factory-key.pem
  --rehearsal-slot1-key rehearsal-work/my-key.pem --slot1 valid --disable-otp-boot 0 --key-invalid 0`:
  PASS. (R keeps both keys at those paths, R:67, R:1066.) Every later step passes the same `--ser`
  and `--rehearsal-key`, and `check`, `disable-otp-boot` and `invalidate-spare-keys` also pass
  `--rehearsal-slot1-key` (§3.1).
- R4. `erase-range --probe-only --execute` (4 MB board: the probe must find the marker and refuse
  with exit 2; that refusal is the probe's positive control, and any other result stops the
  rehearsal), then `erase-range`, then R's phase 5 with `ACCEPT_BLINKY_ONLY=1` (positive control: blinky boots).
  Re-enter BOOTSEL by hand after every phase 5.
- R5. `inject-copy --case bf0-copy3`. `check` (same flags as R3) must refuse naming BOOT_FLAGS0 copies
  (`FAIL BOOT_FLAGS0 copies differ`, not `unreadable`). Then `capture --out …/r5-capture.json`: its
  transcripts are the silicon RAW_VALUE/WARNING lines the replay fixtures need (case 12). *(Added by
  the execution review r1 fold, M9.)*
- R6. `disable-otp-boot --execute`: heal message, post PASS. Phase 5 again: still boots (F5).
- R7. `inject-copy --case bf1-copy0`. `check` (R3's flags with `--disable-otp-boot 1 --key-invalid 0`)
  must refuse naming BOOT_FLAGS1 copies (`FAIL BOOT_FLAGS1 copies differ`). Then `capture --out
  …/r7-capture.json` as in R5. *(Added by the execution review r1 fold, M9.)*
- R8. F10 on silicon: `otp get -c 1 -n --ser S 0x04b` must show `0x000803` while the named vote shows
  `0x000003`, and the named read's RAW_VALUE[0] must equal it. F10 false on silicon shows up first in
  R7: `inject-copy`'s verification exits 3 because `-c 1` disagrees with RAW_VALUE[0]. In that case run
  R8's read anyway, record both readings, stop the rehearsal there, and re-open the plan to replace the
  `-c 1` read; R9 and R10 do not run.
- R9. `invalidate-spare-keys --execute`: heal, post PASS. Phase 5 again: still boots. Write R8's reading
  into the result file and F-619.
- R10. `check --profile rehearsal --ser S --rehearsal-key … --rehearsal-slot1-key … --slot1 valid --disable-otp-boot 1 --key-invalid c`: PASS, with the
  CANNOT PROVE list. This is E3a's pass.
- R11. Optional, after R10: hardening evidence. A fresh key generated outside `rehearsal-work/`;
  `--make-otp-json --slot 2` with `ALLOW_UNCHECKED_KEY=1`; `otp load … --ser S`; `otp set -s
  BOOT_FLAGS1.KEY_VALID 0x4 --ser S`; flash an image signed by that key: it must not boot. Then `check`
  must refuse naming slot 2. Recorded as evidence, not proof (no positive control for slot 2).

## 8. Order of work

1. Pre-dispatch (author): G1, G3, G5 established (F20-F23, `e3a-g-facts.md`); `nix build .#picotool` here;
   run R's old e2e in `.#otp` if the fork is reachable (else at R0); run the e2e skeleton once.
2. Implementer (one agent, worktree, TDD): fake and cases first (red); `otp-read.sh` extraction with
   R's old e2e still green; `refugium-otp.sh`; R changes; flake; docs; CI jobs; mutations.
3. Adversarial review of the whole diff (opus); fold; repeat to 0 C / 0 I.
4. PR; merge through the Merging PRs thread on Brian's per-PR go-ahead.
5. Bench rehearsal §7 with Brian; result file.
6. Bench-result PR: the result file, the R1 transcripts, the R5 and R7 refusal transcripts, and the
   `BENCH_RUN` marker (case 12 activates); F-701's E3a part closes when it merges.
7. E3b separately, after E4 and H0's `capture`, on Brian's typed go-ahead naming SeedHammer #1's CHIPID
   and each step.

## 9. Done when

- `run-e2e-otp.sh` passes locally and in CI; every §6.3 mutation is killed by its named case.
- The argv probe job passes in CI.
- R's old e2e passes in `.#otp`.
- `nix build .#picotool` gives 2.3.1 here and on Brian's box.
- §7 R0-R10 pass on a Pico 2, R5 and R7 refused, transcripts committed.
- No SeedHammer was written to.

## 10. Fold table (R0 round 1)

| finding | fold |
|---|---|
| A-C1 argv order | F14; one argv builder §3.0; fake enforces order §6.1; CI probe §6.4; mutation §6.3; F-619 correction §5 |
| A-C2 / B-C1 heal vs equality; stage | explicit expected state for `check` §3.2; writes derive pre/post §3.3; heal rule §3.3; cases 4-5 |
| A-I1 / B-I6.1 phase 3 skipped | R2 runs 0-4; `ACCEPT_BLINKY_ONLY` and BOOTSEL re-entry R4 |
| A-I2 / B-I6.2-3 no shell; picosign | `.#otp` has tinygo, go, xxd; `SEEDHAMMER_DIR`; R0 picosign check before any write; R not gated to 2.3.1 |
| A-I3 / B-I4 gates cannot run | old fake version line 2.3.1; R's old e2e out of CI, run in `.#otp`; case 9 dropped |
| A-I4 / B-M1 R10 vs R11 | R11 after R10, commands spelled out, evidence not proof |
| A-I5 white-label strings | §3.2 row, `capture` §3.5 |
| B-I1 interrupted write | exit 3 contract and text §3.0; heal rule admits the re-run; case 6 |
| B-I2 profile trusted | identity gate for every device command §3.1; case 7 |
| B-I3 inject writes all copies | fixed cases, `-c 1`, post-injection verification §3.6; case 10 |
| B-I5 mutations unsatisfiable | fault modes §6.1; mapping §6.3 |
| B-I7 exit codes vs dying helpers | `try_` readers, main-shell arrays, compared-row count §3.0 |
| A-M1 / B-M2 rule (d) | copy rule restated with `-c 1` inside (a), applied to CRIT1 and USB_BOOT_FLAGS |
| A-M2 F10 reading | R8 after the copy-0 injection |
| A-M3 CRIT0/CRIT1 whole | §3.2 |
| A-M4 stages | explicit state; slot-1 joint table covers the spec's interrupted-run state |
| A-M5 `--ser` on check | required everywhere §3.1; R passes empty serial |
| A-M6 flake URL | `git+https` shallow form §2 |
| A-M7 one-row write | F16, §3.6 |
| A-M8 `.bin`, top sector | F13, §3.4 |
| A-M9 / B-M8 ROLLBACK_REQUIRED | bit 11 either value; capture records unit history |
| A-M10 flipping note | G1, fake emits it |
| A-N1 version | F17 `version -s` |
| A-N2 save default | F13 |
| A-N3 serial vs white-label | F15 |
| B-M3 joint states | slot-1/KEY_VALID table; case 2 |
| B-M4 rehearsal key optional | required under `rehearsal` §3.1 |
| B-M5 G1 drift late | R1 capture before any write; transcripts replayed in CI (case 12) |
| B-M6 CHIPID spelling | case 11 |
| B-M7 test entry in retail JSON | fixtures path, `_TEST_ONLY` override, schema check §3.2 |
| B-M9 ECC rows read raw | §3.2 |
| B-M10 tee, shellcheck, SDK, small flash | `--log`; `shellcheck -x -S warning` and R's two warnings fixed; `otp list` fingerprint; alias probe |
| B-N1 where R6's result goes | R9 writes it to the result file and F-619 |
| B-N2 `--board-rev` | dropped: a unit must equal one recorded entry |

## 11. Fold table (R0 round 2: A 0C/1I, B 0C/2I; reports e3a-plan-r1-a.md, e3a-plan-r1-b.md)

| finding | fold |
|---|---|
| A2-I1 / B2-I1 heal admits stray target-field bits | heal rule bounds every copy by E, differs only in T, copy0 \| T = E §3.3; case 5 stray bit 8/9; mutation row |
| B2-I2 two mutations survive | case 3c (`COPIES_IGNORED`); `FAKE_REQUIRE_SER` + case 13 argv log; red runs in the report §6.3 |
| A2-M1 80-column wrap | F18; fake wraps §6.1; probe joins lines §6.4 |
| A2-M2 CRIT0 is a CRIT register | §3.2 copy rule with `-c 1` of 0x038; case 3 CRIT0 |
| A2-M3 stock values assumed | R1 checks them before R2 §7 |
| A2-M4 rehearsal slot-1 key flag | `--rehearsal-slot1-key` §3.1; R3 names it |
| A2-M5 / B2-I2.2 `--ser` killer | as B2-I2 |
| A2-M6 BOOT_FLAGS1 other bits | bits outside KV/KI equal the entry (retail) or 0 (rehearsal) §3.2 |
| A2-M7 alias probe detail | random marker, `load -v`, gating, `--probe-only` §3.4; R4; case 8 |
| A2-M8 no blinky at R0 | R0 builds it §7 |
| A2-N1 G2 answered | F16 absorbs G2 |
| B2-M1 F10 before R8 | RAW_VALUE[0] cross-check in heal and inject-copy; inject-copy's own exit-3 text; case 5 |
| B2-M2 pre-state scope, branches | full check pre-state; ordered branches; exit 3 only after `otp set` §3.3 |
| B2-M3 probe writes flash | probe only after `--execute` + confirmation §3.4 |
| B2-M4 compared-row count | compared-row set §3.0 |
| B2-M5 retail entry vs table | entry cells listed, same-entry rule §3.2 |
| B2-M6 deny-list spelling | `--ser` form §3.1 |
| B2-M7 case 12 vacuous | `BENCH_RUN` marker; bench-result PR §8 with R5/R7 transcripts |
| B2-M8 CI not gating | steps in `test (rust + go)`; probe `contents: read`, builder-generated argv, ⊆ check, required-check request §6.4 |
| B2-M9 `_TEST_ONLY` on writes | refused with `--execute`; RESULT "TEST ENTRY" §3.2 |
| B2-M10 exit-3 guidance | "non-zero for any reason, stop" §3.0; one pre-authorised re-run §5 |
| B2-N1 wrapper prints | `printf` form §3.0 |
| B2-N2 old fake `version -s` | old fake stub answers 2.3.1 §6.1, §6.4 |

## 12. Fold table (R0 round 3: 0C/3I/5M/2N; report e3a-plan-r2-a.md)

| finding | fold |
|---|---|
| R3A-I1 pre-state loosened; E ambiguous | derived flags restricted to legal values; E defined from the expected state, bit 11 from the agreed copies §3.3; case 5e |
| R3A-I2 three mutations survive | `SUPPRESS_WARNING` drops RAW_VALUE too, new `SUPPRESS_RAW_VALUE` §6.1; case 3c under it; case 5d 0x903/0x103/0x103; `COPIES_IGNORED` case pinned to 0x103/0x003/0x003; field half noted as equivalent §6.3 |
| R3A-I3 cross-check only inside heal | equal-copy test in every branch needs no WARNING, no RAW_VALUE and agreeing per-copy reads; cond 4 requires RAW_VALUE §3.3 |
| R3A-M1 R8 unreachable if F10 false | R8 runs after an R7 exit 3, records both readings, stops §7 |
| R3A-M2 bench flags, key path | R3/R10 carry `--ser` and both keys at R's paths §7 |
| R3A-M3 R1 stock list | every rehearsal-column cell at its pre-R2 value §7 |
| R3A-M4 `--probe-only` | in the synopsis §3.1; R4 expects exit 2 |
| R3A-M5 shape normalisation | placeholders `<S>`, `<N>`, `<F>` §6.4 |
| R3A-N1 fold-table pointer | header names §10-§12 |
| R3A-N2 slot-1 key scope | the three commands that judge slot 1 §3.1 |

## 13. Fold table (R0 round 4: 0C/1I/3M/2N; report e3a-plan-r3-a.md)

| finding | fold |
|---|---|
| R4A-I1 condition 2 implies 1; superset row unkillable | equivalent mutants listed without rows; bound row respelled "E replaced by copy 0 \| T in conditions 1-3" §6.3; note in heal rule §3.3 |
| R4A-M1 source of the per-copy values | conditions 1-3 use per-copy reads §3.3; case 0x803/0x003/0x003 under `COPIES_IGNORED`; mutation row §6.3 |
| R4A-M2 bit 11 untested | case 5f; mutation row §6.3 |
| R4A-M3 slot-1 key on other commands | R3 says which steps pass it §7 |
| R4A-N1 table broken by the note | note moved below the table §6.3 |
| R4A-N2 runbook and R7 flags | `--ser S` in the runbook line; R7 spelled out §5, §7 |

Round 5 (`e3a-plan-r4-a.md`, Sonnet fold check): 0 C / 0 I / 2 M / 1 N, which closes the R0 gate.
R5-M1 (blank line splitting the §6.3 table) removed; R5-M2 case 5f wording fixed and its two missing
cases added; R5-N1 case 7b scoped to the commands that take the flag.
