# E3a pre-dispatch facts G1 / G3 / G5 (+ F15, F17, fingerprint) — picotool 2.3.1

Date: 2026-10-05. Author: research subagent (opus). Plan: `design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md` (draft 5) §1.

## 0. Sources and method

| tag | what | identity |
|---|---|---|
| PT | picotool source | `raspberrypi/picotool` tag `2.3.1`, commit `2041936441b48a3cc53ae3da9e805229fe8f4e18` (all three scratchpad checkouts `src/picotool`, `e3a-picotool/picotool`, `picotool` are this commit). `main.cpp` lines cited as `PT:<line>`. |
| CL | picotool's vendored clipp | `PT clipp/clipp.h` (the `formatting_ostream` that does all wrapping) |
| SDK | pico-sdk | tag `2.3.1` (scratchpad `pico-sdk`, `src/sdk`) |
| BR | RP2350 boot ROM | `raspberrypi/pico-bootrom-rp2350` tag `A4`, commit `c6cdb1711f32c3e34faaebd58618a6d096dbd52e` (scratchpad `br`) |
| BIN | built binary | `/nix/store/5cxc3bjswwjpgw9i3sdlp5xw1y4v82hc-picotool-2.3.1/bin/picotool`, run with no device attached, stdout redirected to a file (i.e. not a tty) |
| HX | **verbatim-source harness** | see below |

**HX — how the literal `otp get` / `otp set` blocks below were produced.** No device was available, so
the output-producing code was compiled verbatim and fed fake OTP contents:

- `g1.cpp` = a small prelude (settings struct, `fos` = `clipp::formatting_ostream<std::ostream>` on
  `std::cout` exactly as `PT:1837-1844`, `fail()` as `PT errors/errors.cpp`, a fake `con.otp_read`
  that copies raw 32-bit rows from an array) + **verbatim line ranges** of `main.cpp`:
  `PT:5469-5486` (`otp_calculate_ecc`), `PT:7528-7665` (`otp_match`, `get_mask`, `init_matches`,
  `filter_otp`), `PT:9167-9327` (the whole body of `otp_get_command::execute` after the connection
  set-up). `fos.last_column(80)` as `PT:10199` does when stdout is not a tty.
- `gset.cpp` = same prelude + `PT:8976-8996` (`check_otp_write_error`, `settings_select_ecc`,
  `otp_cmd_max_bits`) + `PT:9601-9740` (whole body of `otp_set_command::execute`) + the
  `main()` catch formats `PT:10436-10441`; the fake `otp_write` emulates `BR varm_otp.c:392-407`
  (rows in order; `current & ~new` → `BOOTROM_ERROR_UNSUPPORTED_MODIFICATION` → PICOBOOT status 17,
  after earlier rows are programmed).
- The register table: the 2.3.1 Nix binary was **not** built from the committed
  `otp_header_parser/rp2350.json.h` (that one still has `DISABLE_BOOTSEL_EXEC2`); it was regenerated
  from SDK 2.3.1 `otp_data.h`. HX therefore uses JSON produced by compiling `PT otp_header_parser/otp_header_parse.cpp`
  and running it on `SDK src/rp2350/hardware_regs/include/hardware/regs/otp_data.h`.
  **Cross-check against BIN:** all 1005 `ROW`/`field` lines of `picotool otp list -n` are identical to
  the HX table; all 51 register and field description strings for rows 0x040/0x048/0x04b/0x054/0x059
  (`otp list -f`) are identical; and the 80-column wrapping of every description line BIN printed is
  byte-identical to HX's wrapping (`diff` empty). So HX reproduces BIN's text and wrapping engine.
- Harness files: scratchpad `g1/{g1.cpp,gset.cpp,otp231.json,ohp}` (ephemeral; recipe above suffices
  to rebuild).

What HX does **not** cover: the USB/PICOBOOT transport and the boot ROM's actual responses (G3 and
the mid-write failure are traced in BR source, not observed), and device enumeration (G5 traced in PT
source). Those are marked accordingly.

Exit status mapping (all paths): `main` returns `rc` (`PT:10202-10476`); `rc` is a negative
`ERROR_*` from `PT errors/errors.h`, so the process exit status is `rc & 0xff`:

| constant | value | exit |
|---|---|---|
| ERROR_ARGS (also CLI parse errors, `PT:10203`) | -1 | 255 |
| ERROR_FORMAT | -2 | 254 |
| ERROR_INCOMPATIBLE | -3 | 253 |
| ERROR_USB | -6 | 250 |
| ERROR_NO_DEVICE | -7 | **249** |
| ERROR_NOT_POSSIBLE | -8 | **248** |
| ERROR_CONNECTION (`picoboot::connection_error`, `PT:10442-10444`) | -9 | 247 |
| ERROR_CANCELLED | -10 | 246 |
| ERROR_VERIFICATION_FAILED | -11 | **245** |
| ERROR_UNKNOWN (every `picoboot::command_failure`, i.e. a boot-ROM-reported PICOBOOT error, `PT:10439-10441`) | -99 | **157** |

Error text goes to **stdout** via raw `std::cout` (`PT:10437, 10440, 10443`), *not* through `fos`, so
`ERROR:` lines are never word-wrapped. A `fail()` message that itself ends in `\n` (all the `otp set`
ones do) produces an extra empty line. Nothing in these paths writes to stderr (BIN: stderr empty in
every run).

## 1. F18 — the wrapping engine (applies to everything below)

- Stream: `fos` wraps `std::cout` (`PT:1837-1844`) = **stdout**. Progress bars (`PT` `struct
  progress_bar`, `\r`-updated) and `ERROR:` lines are raw `std::cout`, unwrapped.
- Width: `get_terminal_size` does `ioctl(fileno(stdout), TIOCGWINSZ)`; on failure (stdout is a
  pipe/file) width = 80 (`PT:10180-10186`); `main` then sets `fos.last_column(max(tw,40))`
  (`PT:10197-10200`). If stdout **is** a tty, the terminal width is used. Edge: if the ioctl succeeds
  but reports 0 columns, `last_column` is never set and clipp's default **100** applies
  (`CL:79`, `lastCol_{100}`).
- Rule (`CL:327-372`, `write_line`): a text run longer than the columns left is broken at the **last
  whitespace at or before** the limit; the break whitespace is dropped; the continuation starts at
  `first_column + hanging_indent`. Max printed width is 80 characters.
- Indents in `otp get`: the `ROW` header runs at first_column 0, hanging 7 (`PT:9203-9204`);
  `RAW_VALUE`/`VALUE`/`field` lines at first_column 4, hanging **10** → continuation lines start at
  **column 14** (`PT:9238-9239, 9292-9293`); descriptions at first_column 8, hanging 0 (`PT:9232-9233`).
  The missing-device message is at first_column 0, hanging 0 → continuation at column 0.
- **A single whitespace-free token longer than the space left is NOT broken; it overflows the
  line** (`CL:337-342`: no whitespace before → search after → none → whole token written). And clipp
  does not advance `curCol_` in that branch (`CL:343-347` vs `CL:356`), so the very next text is
  treated as "at beginning of line" and **its leading whitespace is discarded** (`CL:330-334`).
  Consequence for CRIT1/CRIT0 (8 copies): `RAW_VALUE=` + 7×`;0x……` is one 81-char token, so the
  whole RAW_VALUE line, warning included, prints on **one physical line of 124 characters**, and the
  warning is glued on **without its space**: `…;0x000001(WARNING - REDUNDANT ROWS AREN'T EQUAL)`.
  When a `(flipping …)` note precedes it, `curCol_` differs and the space survives (§2.6).
- What *does* span lines in practice: descriptions; the `--ser` miss message (§5); a 3-copy
  `RAW_VALUE` line when a `(flipping …)` note precedes it (§2.6). The plain 3-copy line
  `    RAW_VALUE=0x000803;0x000003;0x000003 (WARNING - REDUNDANT ROWS AREN'T EQUAL)` is exactly
  80 characters and does **not** wrap.

**Contradiction with plan F18:** "a CRIT1 `RAW_VALUE=` list with 8 copies spans lines" is false at
width 80 (it overflows on one line, and may lose the space before `(WARNING`). The rest of F18 holds.
Parser consequence: join continuation lines (lines starting with ≥ 14 spaces after a
RAW_VALUE/VALUE/field line) *and* accept the warning with or without a preceding space; do not split
the RAW_VALUE list on whitespace.

## 2. G1 — `otp get` output, exact text

### 2.1 Format strings and conditions (`otp_get_command::execute`, `PT:9162-9330`)

Per selected row (rows sorted by `(row, mask)` from a `std::map`, `PT:7598-7601`, so output order is
row order, not argv order, and duplicates are merged):

1. Separator: from the second row on, `fos.wrap_hard()` → **one empty line** before the header
   (`PT:9202`). (When the previous row ended with the empty line after `VALUE`, i.e. a row with no
   field lines, two consecutive empty lines appear — §2.5 CHIPID.)
2. Header `ROW 0x%04x` — 4 lowercase hex digits (`PT:9205`). If the row resolves to a named register
   (by name, or by number via `otp_regs.find`, `PT:7554-7558`): `: <NAME>` with the full
   `OTP_DATA_` name (`PT:9212`). With `-n` only: ` (ECC)` / else ` (CRIT)` / else ` (RBIT-<n>)`
   (`PT:9213-9221`); sequences add ` (Part i/n)` (`PT:9223-9225`). Unnamed rows (0x049, 0x04a, 0x04c,
   0x04d, 0x039-0x03f, 0x041-0x047, 0x05a, 0x05b — BIN `otp list` prints nothing for them) get no
   name and are treated as raw (redundancy stays -1).
3. Without `-n`, a named register's description line `"…"` at column 8 (`PT:9231-9237`). The
   description ends with e.g. ` (RBIT-3)` as part of its own text.
4. `RAW_VALUE=0x%06x` is built for copy 0, then `;0x%06x` per extra copy (`PT:9243-9246`),
   **but is printed only** (a) in the vote branch when any bit differs between copies (`PT:9277-9281`),
   followed by ` (WARNING - REDUNDANT ROWS AREN'T EQUAL)`; or (b) in the ECC branch when the ECC check
   fails, followed by ` (WARNING - ECC IS INVALID)` (`PT:9258-9262`). Never for raw/unnamed rows
   without `-c`. Hex is lowercase, 6 digits.
5. `(flipping raw value to 0x%08x)` (`PT:9247-9251`): printed **immediately, before the RAW_VALUE
   text and on the same line, with no separator**, once for each copy **i ≥ 1** whose raw bits 23:22
   are both 1; the printed value is that copy XOR 0xffffff, 8 lowercase hex digits. Copy 0 never
   triggers it. It does not change the vote or the RAW_VALUE list (the list was already formatted;
   the vote re-reads raw, `PT:9268-9270`). It is printed whether or not the copies differ.
6. `VALUE`:
   - ECC branch (`do_ecc` = `-e`, or named ECC register and not `-r`, `PT:9227`): `VALUE 0x%04x`
     (4 digits) of `otp_calculate_ecc(raw & 0xffff) & 0xffff` (`PT:9254-9256`), where `raw` is the
     **last** copy read (copy 0 unless `-c N>1`). Then on ECC mismatch the RAW_VALUE+warning line,
     and in that case **no** empty line follows before the fields (the warning text has no `\n`).
   - Vote branch (`redundancy > 0`: named RBIT/CRIT register with no `-c`, any row with `-c N`, or a
     named ECC register under `-r`, whose redundancy is 1): per bit, set if `sets >= clears`, or for
     CRIT registers also if `sets >= 3` (`PT:9272-9274`; note: a 4/4 tie sets the bit);
     `VALUE 0x%06x` (6 digits).
   - Else (unnamed row, no `-c`): `VALUE 0x%06x` of copy 0 (`PT:9285-9288`).
   - `VALUE` is preceded by an **empty line** whenever no RAW_VALUE line was printed (the format is
     `"\nVALUE …\n"` and clipp emits the leading `\n` as a hard wrap, `CL:298-302, 157-180`). When a
     RAW_VALUE line was printed, `VALUE` is on the next line with no empty line.
   - After `VALUE`, one empty line (`PT:9290`), except the ECC-invalid case above.
7. Field lines (named registers only, fields whose mask intersects the selection, in JSON order =
   high bits first): `field <NAME> (bit <n>) = %x` or `field <NAME> (bits <lo>-<hi>) = %x`
   (`PT:9297-9304`) — value **lowercase hex, no `0x`, no padding** (e.g. `= a`, `= 8`), decoded from
   the voted/ECC-corrected value. Without `-n`, each field's description follows at column 8.
   A field selector (`0x04b.KEY_INVALID`) prints only that field.
8. Selectors that match nothing print nothing and **exit 0** (`filter_otp` returns empty; no error,
   `PT:7598-7665`). BIN confirms the same for `otp list -n NOPE` (rc 0, no output).

`-c N`: sets `settings.otp.redundancy` (`PT:1378`), which wins over the register's redundancy
(`PT:9228`). For a named register **`-c 1` gives a 1-copy vote = copy 0's raw value, with no
RAW_VALUE line ever** (confirms F10, source level). `-c N` on an unnamed row reads N consecutive rows
and votes like a named register (non-CRIT rule).

`-r`: only suppresses ECC decoding for named ECC rows (`PT:9227`); on a named ECC register the
redundancy is 1 (otp231.json: FLASH_DEVINFO `redundancy 1, ecc true`), so it falls into the vote
branch and prints the raw 24-bit `VALUE 0x%06x`. On RBIT/CRIT/unnamed rows `-r` changes nothing.

`-e`: forces the ECC branch on any row. On an RBIT register this is wrong-shaped output (VALUE is
the ECC-corrected **last copy**, fields decoded from a value that includes parity bits 16-21 — HX
shows `DOUBLE_TAP_DELAY = 6` for a 0x000003 row). Do not use `-e` on non-ECC rows.

Register facts from the table (otp231.json = BIN): CRIT0 0x038 and CRIT1 0x040 `crit, redundancy 8`;
BOOT_FLAGS0 0x048, BOOT_FLAGS1 0x04b, USB_BOOT_FLAGS 0x059 `redundancy 3`; FLASH_DEVINFO 0x054
`ecc, redundancy 1`; PAGE1/2_LOCK0/1 0xf82-0xf85 `redundancy 1` (shown `(RBIT-1)` under `-n`).

### 2.2 Literal blocks (HX, width 80, `cat -A` checked; `⏎`-free here, trailing spaces: none)

**(a) BOOT_FLAGS0, three equal copies 0x002000 — `otp get -n --ser S 0x048`**
```
ROW 0x0048: OTP_DATA_BOOT_FLAGS0 (RBIT-3)

    VALUE 0x002000

    field DISABLE_SRAM_WINDOW_BOOT (bit 21) = 0
    field DISABLE_XIP_ACCESS_ON_SRAM_ENTRY (bit 20) = 0
    field DISABLE_BOOTSEL_UART_BOOT (bit 19) = 0
    field DISABLE_BOOTSEL_USB_PICOBOOT_IFC (bit 18) = 0
    field DISABLE_BOOTSEL_USB_MSD_IFC (bit 17) = 0
    field DISABLE_WATCHDOG_SCRATCH (bit 16) = 0
    field DISABLE_POWER_SCRATCH (bit 15) = 0
    field ENABLE_OTP_BOOT (bit 14) = 0
    field DISABLE_OTP_BOOT (bit 13) = 1
    field DISABLE_FLASH_BOOT (bit 12) = 0
    field ROLLBACK_REQUIRED (bit 11) = 0
    field HASHED_PARTITION_TABLE (bit 10) = 0
    field SECURE_PARTITION_TABLE (bit 9) = 0
    field DISABLE_AUTO_SWITCH_ARCH (bit 8) = 0
    field SINGLE_FLASH_BINARY (bit 7) = 0
    field OVERRIDE_FLASH_PARTITION_SLOT_SIZE (bit 6) = 0
    field FLASH_DEVINFO_ENABLE (bit 5) = 0
    field FAST_SIGCHECK_ROSC_DIV (bit 4) = 0
    field FLASH_IO_VOLTAGE_1V8 (bit 3) = 0
    field ENABLE_BOOTSEL_NON_DEFAULT_PLL_XOSC_CFG (bit 2) = 0
    field ENABLE_BOOTSEL_LED (bit 1) = 0
```
(Without `-n`: header has no ` (RBIT-3)`; a 3-line description follows the header, then the empty
line and `VALUE`; each field with a description is followed by it at column 8. There is no
`DISABLE_BOOTSEL_EXEC2` field in 2.3.1-as-built — §6.)

Same, without `-n` (head only):
```
ROW 0x0048: OTP_DATA_BOOT_FLAGS0
        "Disable/Enable boot paths/features in the RP2350 mask ROM. Disables
        always supersede enables. Enables are provided where there are other
        configurations in OTP that must be valid. (RBIT-3)"

    VALUE 0x000000

    field DISABLE_SRAM_WINDOW_BOOT (bit 21) = 0
    field DISABLE_XIP_ACCESS_ON_SRAM_ENTRY (bit 20) = 0
        "Disable all access to XIP after entering an SRAM binary. Note that this
        will cause bootrom APIs that access XIP to fail, including APIs that
        interact with the partition table."
```

**(b) BOOT_FLAGS1 copies 0x000803 / 0x000003 / 0x000003 — `otp get -n --ser S 0x04b`**
```
ROW 0x004b: OTP_DATA_BOOT_FLAGS1 (RBIT-3)
    RAW_VALUE=0x000803;0x000003;0x000003 (WARNING - REDUNDANT ROWS AREN'T EQUAL)
    VALUE 0x000003

    field DOUBLE_TAP (bit 19) = 0
    field DOUBLE_TAP_DELAY (bits 16-18) = 0
    field KEY_INVALID (bits 8-11) = 0
    field KEY_VALID (bits 0-3) = 3
```
(The RAW_VALUE line is exactly 80 characters.) Without `-n`, the header is
`ROW 0x004b: OTP_DATA_BOOT_FLAGS1`, then the same 3-line description as BOOT_FLAGS0, then the
RAW_VALUE line directly (no empty line), then `VALUE`, empty line, fields with descriptions.

**(c) the same board — named read with `-c 1`: `otp get -c 1 -n --ser S 0x04b`**
```
ROW 0x004b: OTP_DATA_BOOT_FLAGS1 (RBIT-3)

    VALUE 0x000803

    field DOUBLE_TAP (bit 19) = 0
    field DOUBLE_TAP_DELAY (bits 16-18) = 0
    field KEY_INVALID (bits 8-11) = 8
    field KEY_VALID (bits 0-3) = 3
```
(No RAW_VALUE, no WARNING, ever, under `-c 1`. The header still says `(RBIT-3)`.)

**(d) CRIT1, 8 copies, copy 5 odd (0x000001 ×7, copy 5 = 0x000000) — `otp get -n --ser S 0x040`**
```
ROW 0x0040: OTP_DATA_CRIT1 (CRIT)
    RAW_VALUE=0x000001;0x000001;0x000001;0x000001;0x000001;0x000000;0x000001;0x000001(WARNING - REDUNDANT ROWS AREN'T EQUAL)
    VALUE 0x000001

    field GLITCH_DETECTOR_SENS (bits 5-6) = 0
    field GLITCH_DETECTOR_ENABLE (bit 4) = 0
    field BOOT_ARCH (bit 3) = 0
    field DEBUG_DISABLE (bit 2) = 0
    field SECURE_DEBUG_DISABLE (bit 1) = 0
    field SECURE_BOOT_ENABLE (bit 0) = 1
```
The RAW_VALUE line is ONE line of 124 characters with **no space before `(WARNING`** (§1). Other
CRIT votes from HX: copy 0 odd (0, then 1×7) → VALUE 0x000001; four 1 + four 0 → VALUE 0x000001
(tie sets); one 1 + seven 0 → VALUE 0x000000. `-c 1 -n 0x040` → empty line, `VALUE 0x000001`
(copy 0), no RAW_VALUE. Field selector `0x040.SECURE_BOOT_ENABLE` → same RAW_VALUE/VALUE lines and
only `    field SECURE_BOOT_ENABLE (bit 0) = 1`.

**(e) unnamed row 0x04a, raw — `otp get -n --ser S 0x04a`** (row holds 0x002000)
```
ROW 0x004a

    VALUE 0x002000

```
(the last line is an empty line). No name, no fields, no RAW_VALUE. `-n` makes no difference.
With `-c 3 -n 0x04c` (rows 0x04c, 0x04d, 0x04e) differing:
```
ROW 0x004c
    RAW_VALUE=0x000003;0x000003;0x000000 (WARNING - REDUNDANT ROWS AREN'T EQUAL)
    VALUE 0x000003

```
With `-e -n 0x04a` on a raw (non-ECC) 0x002000:
```
ROW 0x004a

    VALUE 0x2000
    RAW_VALUE=0x002000 (WARNING - ECC IS INVALID)
```

**(f) ECC row FLASH_DEVINFO 0x054 holding data 0x0a00 with valid ECC (raw 0x3f0a00)**

`otp get -n --ser S 0x054`:
```
ROW 0x0054: OTP_DATA_FLASH_DEVINFO (ECC)

    VALUE 0x0a00

    field CS1_SIZE (bits 12-15) = 0
    field CS0_SIZE (bits 8-11) = a
    field D8H_ERASE_SUPPORTED (bit 7) = 0
    field CS1_GPIO (bits 0-5) = 0
```
`otp get -r -n --ser S 0x054` (the plan's "ECC rows read raw"):
```
ROW 0x0054: OTP_DATA_FLASH_DEVINFO (ECC)

    VALUE 0x3f0a00

    field CS1_SIZE (bits 12-15) = 0
    field CS0_SIZE (bits 8-11) = a
    field D8H_ERASE_SUPPORTED (bit 7) = 0
    field CS1_GPIO (bits 0-5) = 0
```
Raw row 0x000a00 (ECC bits missing), no `-r`:
```
ROW 0x0054: OTP_DATA_FLASH_DEVINFO (ECC)

    VALUE 0x0a00
    RAW_VALUE=0x000a00 (WARNING - ECC IS INVALID)
    field CS1_SIZE (bits 12-15) = 0
    field CS0_SIZE (bits 8-11) = a
    field D8H_ERASE_SUPPORTED (bit 7) = 0
    field CS1_GPIO (bits 0-5) = 0
```
An all-zero ECC row is valid (ECC of 0 is 0) → `VALUE 0x0000`, no warning.

**(g) the flipping note** (BOOT_FLAGS1 copies 0x000003 / 0xc00003 / 0x000003, `-n 0x04b`):
```
ROW 0x004b: OTP_DATA_BOOT_FLAGS1 (RBIT-3)
    (flipping raw value to 0x003ffffc)RAW_VALUE=0x000003;0xc00003;0x000003
              (WARNING - REDUNDANT ROWS AREN'T EQUAL)
    VALUE 0x000003

    field DOUBLE_TAP (bit 19) = 0
    field DOUBLE_TAP_DELAY (bits 16-18) = 0
    field KEY_INVALID (bits 8-11) = 0
    field KEY_VALID (bits 0-3) = 3
```
Here the warning **wraps** to column 14. All three copies 0xc00003 (equal, so no RAW_VALUE):
```
ROW 0x004b: OTP_DATA_BOOT_FLAGS1 (RBIT-3)
    (flipping raw value to 0x003ffffc)(flipping raw value to 0x003ffffc)
    VALUE 0xc00003
```
(the note line ends with a hard newline from `"\nVALUE"`, so there is no empty line before
`VALUE` in this case). CRIT1 with copy 3 = 0xc00001: one physical line
`    (flipping raw value to 0x003ffffe)RAW_VALUE=0x000001;…;0xc00001;…;0x000001 (WARNING - REDUNDANT ROWS AREN'T EQUAL)`
— here the space before `(WARNING` **is** kept (curCol state differs, §1).
A parser anchored at `^\s*RAW_VALUE=` misses these lines; match `RAW_VALUE=` anywhere after
stripping any number of leading `(flipping raw value to 0x[0-9a-f]{8})` groups.

**(h) multiple rows in one call** (`-n 0x048 0x04b BOOT_FLAGS1`): rows printed once each, in row
order, separated by exactly one empty line after the last field line; a row with no field lines
(CHIPIDn, unnamed rows) is followed by **two** empty lines before the next `ROW`.

**(i) CHIPID** (`-n OTP_DATA_CHIPID0 CHIPID1 0x002 0x003`, CHIPID 0x09f50bf63e8d6f46):
```
ROW 0x0000: OTP_DATA_CHIPID0 (ECC) (Part 1/4)

    VALUE 0x6f46


ROW 0x0001: OTP_DATA_CHIPID1 (ECC) (Part 2/4)

    VALUE 0x3e8d


ROW 0x0002: OTP_DATA_CHIPID2 (ECC) (Part 3/4)

    VALUE 0x0bf6


ROW 0x0003: OTP_DATA_CHIPID3 (ECC) (Part 4/4)

    VALUE 0x09f5

```
Name selectors match `NAME` or `OTP_DATA_NAME` exactly (`PT:7656-7660`); numeric selectors resolve to
the named register when one exists at that row (`PT:7554-7558`).

### 2.3 `otp set` (`otp_set_command::execute`, `PT:9600-9740`)

Synopsis (BIN `help otp set`): `otp set [-c <copies>] [-r] [-e] [-s] [-i <filename>] [-z] <selector>
<value> [device-selection]`. **There is no `-n`**, so a named register always prints its description.

Text, in order:
1. `ROW 0x%04x` + `  OLD_VALUE=0x%06x` (two spaces; OLD_VALUE = **copy 0's raw row only**, one-row
   raw read, `PT:9623-9633`) + `: <NAME>` for named rows, then `\n` (`PT:9629-9653`).
2. Named row: the description at column 8 (`PT:9654-9660`).
3. Field selector: `    field <NAME> (bit n)` / `(bits lo-hi)` with **no** `= value` (`PT:9686-9697`).
4. Nothing else on success. **Exit 0.** No VALUE/verification output.

Value computation (confirms F19/F16): field → `value <<= low; &= mask; |= old & ~mask`
(`PT:9702-9704`); `-s` → `|= old` (`PT:9706-9709`); then refuse if `~value & old`
(`PT:9713-9715`). Copies written = `-c N` if given, else the register's redundancy, else (unnamed)
one row (`PT:9663-9664, 9722-9731`); ECC only if `ecc && !raw` (`PT:8990-8992`), automatically for
named ECC registers (`PT:9663`).

Failure texts (stdout; each `fail` message ends in `\n` so an empty line follows):

| case | text | exit |
|---|---|---|
| value lacks a bit copy 0 holds, no `-s` | `ERROR: Cannot clear bits in OTP row(s): current value %06x, new value %06x` (no `0x`, 6 digits), e.g. `ERROR: Cannot clear bits in OTP row(s): current value 000020, new value 002000` | 248 |
| ECC row already non-zero (after the clear-bits check passes) | `ERROR: Cannot modify OTP ECC row(s)` | 248 |
| field value too wide | `ERROR: Value to set does not fit in field: value 00001c, mask 00000f` | 248 |
| selector matches nothing | `ERROR:  no OTP rows matched for writing.` (two spaces) | 255 |
| >1 row / >1 field matched | `ERROR:  multiple OTP rows matched, so write is not allowed.` / `… fields …` | 255 |
| **row-write failure mid-way** (a later copy holds a bit outside the value computed from copy 0) | boot ROM programs rows in order and stops at the bad row with `BOOTROM_ERROR_UNSUPPORTED_MODIFICATION` (`BR varm_otp.c:401-407`) → PICOBOOT status 17 (`BR nsboot_async_task.c:110`, `SDK boot/picoboot.h:82`) → `check_otp_write_error` (`PT:8977-8979`) → `ERROR: Attempted to clear bits in OTP row(s)` | 248 |
| any other boot-ROM refusal (e.g. page lock → NOT_PERMITTED) | `ERROR: The RP2350 device returned an error: permission failure` (`PT:10440`, strings `PT picoboot_connection/picoboot_connection_cxx.cpp:24-44`) | 157 |

Literal mid-way failure (HX; copies 0x000003 / 0x000103 / 0x000003, `otp set -s 0x04b 0x000c00 --ser S`):
```
ROW 0x004b  OLD_VALUE=0x000003: OTP_DATA_BOOT_FLAGS1
        "Disable/Enable boot paths/features in the RP2350 mask ROM. Disables
        always supersede enables. Enables are provided where there are other
        configurations in OTP that must be valid. (RBIT-3)"
ERROR: Attempted to clear bits in OTP row(s)

```
exit 248; afterwards row 0x04b = 0x000c03 (burned), 0x04c = 0x000103, 0x04d = 0x000003 untouched
(F11 confirmed). A copy that is a subset is healed: copies 0x000803/0x000003/0x000003 with
`-s 0x04b 0x000c00` → all three become 0x000c03, exit 0 (note: copy 0's 0x800 is already inside
0xc00 here; per F19 every bit copy 0 holds lands in every copy).

Success example (`otp set -s BOOT_FLAGS1.KEY_INVALID 0xc --ser S`, copies 0x000003):
```
ROW 0x004b  OLD_VALUE=0x000003: OTP_DATA_BOOT_FLAGS1
        "Disable/Enable boot paths/features in the RP2350 mask ROM. Disables
        always supersede enables. Enables are provided where there are other
        configurations in OTP that must be valid. (RBIT-3)"
    field KEY_INVALID (bits 8-11)
```
exit 0, all three rows 0x000c03. Unnamed row `otp set -s 0x04a 0x002000`: just
`ROW 0x004a  OLD_VALUE=0x000000`, exit 0, one row written raw. `-c 1 -s 0x04b 0x000803`: header +
description, one row (0x04b) written.

## 3. G3 — refused flash ranges and failed verify

Boot ROM side (BR, A4). All three BOOTSEL flash ops go through the checked-flash API at NSBOOT
security level: erase `BR varm_s_from_ns_hardened_buffer_wrappers.S:148-167` (falls through to
`s_varm_api_checked_flash_op`, asserted `BR bootrom_arm.template.ld:125`), read/program
`…wrappers.S:169-251`. With storage addressing it calls `s_varm_checked_flash_op_notranslate`
(`BR varm_checked_flash.c:143-146`), whose permission check first requires the span in bounds of the
FLASH_DEVINFO chip-select size (`BR varm_flash_permissions.h:130-134` →
`varm_generic_flash.c:722-738, 740-…`, size `4 KiB << CS0_SIZE`, `BR native_generic_flash.h:104-113`)
and otherwise returns `BOOTROM_ERROR_NOT_PERMITTED` (`BR varm_checked_flash.c:96-99`). NSBOOT maps
that to `PICOBOOT_NOT_PERMITTED` (`BR nsboot_async_task.c:96, 250-255`). The cached devinfo is
`FLASH_DEFAULT_DEVINFO` = 0xc00 (16 MiB CS0) unless FLASH_DEVINFO_ENABLE, in which case the OTP row
(`BR varm_boot_path.c:722-728`, `122`). The NSBOOT pre-checks only reject addresses beyond
XIP_BASE+32 MiB (`BR nsboot_usb_client.h:16-21`, `nsboot_async_task.c:201-206, 219-226`) with
`PICOBOOT_INVALID_ADDRESS`.

picotool side: `connection::wrap_call` turns the non-zero PICOBOOT status into
`picoboot::command_failure(status)` (`PT picoboot_connection_cxx.cpp:55-74`); nothing in
erase/save/load catches it, so `main` prints
`ERROR: The RP2350 device returned an error: permission failure` and returns ERROR_UNKNOWN → **exit
157** (`PT:10439-10441`).

| command | behaviour on a range beyond CS0 (ENABLE set, CS0 smaller) | exit |
|---|---|---|
| `erase -r <from> <to>` | range rounded out to 4 KiB (`PT:5151-5153`); picotool only checks the range is in its flash window (`PT:5170-5174`, else `ERROR: Erase range not all in flash` 248); then erases **sector by sector** (`PT:5178-5184`) — sectors below CS0 are erased before the first refused sector fails. Output: progress bar `Erasing:              [===   ]  N%\r` then `\n` (bar destructor), then `ERROR: The RP2350 device returned an error: permission failure`. `Erased N bytes` is printed only on success (`PT:5186`). | **157** |
| `save -r <from> <to> f.bin` | for a BIN target the range is used **exactly, not rounded** (`PT:4954-4958`; UF2 rounds to 256 B, `PT:4950-4952`); `end <= start` → `ERROR: Save range is invalid/empty` 255; type mismatch → `ERROR: Save range crosses unmapped memory` 248 (`PT:4995-4999`). The output file is **opened/truncated before reading** (`PT:5044`); the refused read throws, the file is closed and left partial (`PT:5065-5068`); no `Wrote …` line. | **157** |
| `load -v <bin> -o 0x10000000` failing verify | after the per-range progress bars: `  FAILED` then `ERROR: The device contents did not match the file` (`PT:5372-5375`) | **245** |
| `load` with flash refused | erase/program throws → `…returned an error: permission failure` | 157 |

Additional finding (not in F1-F19): every flash `load`, `erase -a` and `save -a` calls
`guess_flash_size` (`PT:5277, 5164, 4988`), which reads flash at +8 MiB, +4 MiB, … (`PT
guess_flash_size`) **unless pages 0 and 1 are identical (erased)**. With FLASH_DEVINFO_ENABLE and
CS0 ≤ 8 MiB those probe reads are refused, so such a command dies with exit 157 *before writing*.
Not an issue under retail (CS0 must be 0xc) or rehearsal (ENABLE 0 → default 16 MiB), but the fake
should model "load refused" as 157 as well. Also: with ENABLE 0 on a 4 MiB part the boot ROM does
**not** refuse +4..16 MiB — it aliases (that is what the plan's alias probe exists for).

Also note for the plan's "non-zero exit = the boot ROM refused this range": other non-zero exits
reach the same branch (249 no device / `--ser` miss, 248 single-device refusal, 247 USB comms, 255
arg errors). Only 157 with `permission failure` is the boot-ROM refusal; the tool keys on non-zero,
which is conservative (condemns), so this is a wording point, not a safety hole.

## 4. G5 — more than one RP2350 in BOOTSEL

Traced in PT source (not observed; needs two boards):

- Enumeration: every libusb device is opened; with `--ser S`, an RP2350 is admitted only if its USB
  serial string `strcmp`-equals S (`PT picoboot_connection/picoboot_connection.c:200-218`);
  non-matching ones become `dr_vidpid_unknown` and are ignored (`PT:10246-10252`).
- `info` is `zero_or_more` (`PT:832-836`): it never refuses multiple devices and **exits 0**. With ≥ 2
  admitted devices it prints (`PT:4777-4791`):
```
Multiple RP-series devices in BOOTSEL mode found:

RP2350 device at bus 1, address 7:
----------------------------------
<info body for this device>

RP2350 device at bus 1, address 8:
----------------------------------
<info body>
```
  (`bus_device_string` = `RP2350 device at bus <n>, address <m>`, `PT:232-236`; dashes = its length
  + 1; the `"\n"` before each device name is a hard wrap → one empty line.) The per-device body is
  `info_guts` output — exact body text UNVERIFIED (not traced; the plan should not parse it).
  So **an `info`-based "exactly one device" gate must look for the `Multiple RP-series devices in
  BOOTSEL mode found:` line; the exit code does not distinguish 1 from 2 devices.**
- `info --ser S` with two boards attached: only the matching board is admitted → normal single-device
  output (no `Multiple…` line), exit 0. If neither matches → exit 249 and the §5 text.
- Every `one`-device command (`otp get`/`set`, `erase`, `save`, `load`; default
  `get_device_support() = one`, `PT:525`) with ≥ 2 admitted devices:
  `ERROR: Command requires a single RP-series device to be targeted.` (`PT:10327-10331`) → **exit
  248**. With `--ser` matching exactly one, it proceeds on that one.

## 5. F15 / F17 / no-device texts (BIN, observed)

All with stdout redirected (width 80), stderr empty:

| argv | stdout | exit |
|---|---|---|
| `version -s` | `2.3.1` | 0 |
| `version` | `picotool v2.3.1 (Linux, GNU-16.2.0, Release)` | 0 |
| `info` | `No accessible RP-series devices in BOOTSEL mode were found.` | 249 |
| `info --ser ZZPROBE00000000` | `No accessible RP-series devices in BOOTSEL mode were found with serial number`⏎`ZZPROBE00000000.` | 249 |
| `otp get -c 1 -n --ser ZZPROBE00000000 0x04b` | `No accessible RP2350 devices in BOOTSEL mode were found with serial number`⏎`ZZPROBE00000000.` | 249 |
| `otp set -s 0x04a 0x002000 --ser ZZPROBE00000000` | same two lines as above (RP2350) | 249 |
| `otp get 0x048` | `No accessible RP2350 devices in BOOTSEL mode were found.` | 249 |
| `erase -r 0x10000000 0x10001000` | `No accessible RP-series devices in BOOTSEL mode were found.` | 249 |
| `save -r 0x10000000 0x10001000 f.bin` | same; **no file created** (device check precedes `fopen`) | 249 |
| `erase -r 0x10000000` | empty line, `ERROR: missing <to>`, empty line | 255 |

`otp` commands say `RP2350` (`requires_rp2350`, `PT:1373`), others `RP-series` (`PT:4436-4458`).
The message is printed through `fos` (`PT:10277`), so it **wraps**: the break is before the serial,
continuation at column 0, for any serial (a 16-char CHIPID serial also wraps: line 1 is 74/77 chars).
Confirms F15 (exit 249, text) and F18 for this message. Argument-order probe (F14): `otp get 0x04b -c 1 --ser Z` and `otp get -n -c 1 --ser Z 0x04b` both printed the
message **without** "with serial number" — `--ser` was not parsed — while `otp get --ser Z -c 1
0x04b` and `otp set 0x04a 0x002000 -s --ser Z` did parse it; consistent with F14.

## 6. `otp list` fingerprint (BIN, observed)

- `otp list -n MAC0` → `ROW 0x0062: OTP_DATA_MAC0 (ECC) (Part 1/3)` / `    (row has no sub-fields)`,
  exit 0. **MAC0 present.**
- `otp list -n BOOT_FLAGS0.DISABLE_BOOTSEL_EXEC2`, `otp list -n .DISABLE_BOOTSEL_EXEC2`,
  `otp list | grep -c EXEC2` → no output / 0, exit 0. **DISABLE_BOOTSEL_EXEC2 absent.** (The committed
  `PT otp_header_parser/rp2350.json.h` at tag 2.3.1 *does* contain it; the Nix binary regenerated the
  table from SDK 2.3.1, which does not. So the fingerprint really is "built against SDK 2.3.1".)
- A non-matching `otp list` selector exits **0** with empty output — the fingerprint must test
  output, not exit status.
- `otp list -n` (whole table) sha256 `c9507615467323b3cc8460ca5b6359bd9916437afdd3d80284ae168b5d72e94f`,
  2205 lines without `-n` (informational).

## 7. Status against plan facts

- F9: confirmed, with precision: RAW_VALUE is printed only when copies differ (vote branch) or the ECC
  check fails (ECC branch); never on equal copies, never under `-c 1`.
- F10: confirmed at source/harness level (`PT:9178, 9228, 9263-9276`); silicon reading remains R8.
- F11, F16, F19: confirmed (§2.3; `PT:9702-9715, 8990-8992`; `BR varm_otp.c:401-407`); mid-way failure
  text is `ERROR: Attempted to clear bits in OTP row(s)`, exit 248.
- F12: ECC `VALUE` is 4 hex digits (`0x%04x`), confirmed; under `-r` it is 6 digits.
- F13: **partly wrong**: `erase -r` rounds to 4 KiB; `save -r` to a `.bin` does **not** round
  (exact `from`/`to`), to `.uf2` rounds to 256 B. The `.bin`-name claim was not re-checked here.
- F14: consistent with BIN probes (§5).
- F15: confirmed (exit 249; message wraps before the serial).
- F17: confirmed (`2.3.1`).
- **F18: partly wrong** (§1): the 8-copy CRIT `RAW_VALUE` list does not span lines; it overflows one
  line and may lose the space before `(WARNING`. What wraps is descriptions, the `--ser` miss
  message, and a 3-copy RAW_VALUE line preceded by a `(flipping …)` note (warning moves to a
  continuation line indented 14). Wrapping is on stdout; `ERROR:` lines and progress bars are never
  wrapped; width is the tty width if stdout is a tty (100 if it reports 0 columns).

## 8. UNVERIFIED

1. Everything in §2 is from verbatim source compiled with fake OTP data (HX), cross-checked against
   BIN for table and wrapping, **not observed on a device**. Confirm at bench R1 by capturing
   `otp get -n` of 0x040, 0x048, 0x04b, 0x054 (with and without `-r`, and `-c 1`) on the Pico 2
   before any write, and diff against §2.2's shapes.
2. G3 refusal (exit 157, `permission failure`) is traced in BR A4 + PT source only. It cannot be
   observed on the rehearsal Pico 2 without burning FLASH_DEVINFO_ENABLE with a small CS0
   (irreversible; not recommended). Observation would need a sacrificial board.
3. G5 is traced in PT source only; needs two RP2350 boards in BOOTSEL on one host. The per-device
   `info` body text is not established.
4. Whether the shipping SeedHammer/Pico 2 boot ROM revision behaves as BR tag A4 (the source checked)
   was not verified here.
5. The `-s`/selector order behaviour of `otp set` beyond the `--ser` probe (F14) was not re-measured.
