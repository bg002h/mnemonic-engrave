#!/usr/bin/env python3
"""fake_picotool.py -- a row-level RP2350 simulator for scripts/test/run-e2e-otp.sh.

It stands in for picotool 2.3.1 (built against pico-sdk 2.3.1) so that
scripts/refugium-otp.sh can be exercised end to end with no board attached.
It is NOT the old scripts/test/fake-picotool (which drives the rehearsal
script's own e2e and stays as it is).

What it models, and where each behaviour comes from
---------------------------------------------------
Every output line it prints is taken from the 2.3.1 source, compiled with fake
OTP contents (design/agent-reports/e3a-g-facts.md, "G-facts"); the main.cpp line
of each is cited next to the code below.

  version [-s]                       F17
  info [--ser S]                     G5 (one board, several boards, none)
  otp list [-n] <selector...>        the fingerprint names only (MAC0, EXEC2)
  otp get [-c N] [-r] [-e] [-n] [--ser S] <selector...>
                                     G1: vote, RAW_VALUE, WARNING, the flipping
                                     note, ECC branch, unnamed rows, -c N, fields
  otp set [-c N] [-r] [-e] [-s] <selector> <value> [--ser S]
                                     F11/F19/F21: value from copy 0, rows written
                                     in order, a later copy holding a bit outside
                                     the value fails AFTER earlier rows burned
  otp load <file.json> [--ser S]     bootkeyN arrays only
  erase -r <from> <to> [--ser S]     G3: 4 KiB rounding, sector-by-sector,
                                     refused beyond CS0 when FLASH_DEVINFO_ENABLE
  save -r <from> <to> <f.bin> [--ser S]
                                     G3: exact range, file truncated BEFORE the
                                     read, so a refused read leaves a partial file
  load [-v] <f.bin> [-o <off>] [--ser S]
                                     G3: guess_flash_size refusal, verify text

stdout is word-wrapped at 80 columns exactly as picotool does when stdout is
not a terminal (F18, clipp's formatting_ostream): see class Fos.

It ENFORCES picotool's argument order (F14). picotool's parser matches option
groups in declaration order and a misplaced option silently becomes a selector;
this fake refuses instead: any option after a later group, any selector that is
not a row number, a register name or REG.FIELD, and any subcommand or flag it
does not implement exits 99 "fake-picotool: ...". OTP bits only ever set.

State: $FAKE_PT_STATE, a JSON file written by fake_otp_state.py. Each device
has a serial, a dict of raw 24-bit rows and a flash image file. The argv of every
call is appended, tab-separated, to $FAKE_PT_ARGV_LOG when that is set.

Fault modes (environment)
-------------------------
  SUPPRESS_WARNING=1       a vote that finds unequal copies prints only VALUE:
                           no RAW_VALUE and no WARNING (an output regression)
  SUPPRESS_RAW_VALUE=1     the WARNING is printed, the RAW_VALUE list is not
  COPIES_IGNORED=1         `-c N` on a named register is ignored: VALUE is the
                           register's full vote, printed in the equal-copies
                           shape (F-619's old reading of `-c 1`)
  FAIL_SET_AFTER=N         `otp set` burns N rows, then fails with F21's
                           mid-write text and exit 248
  FAIL_READ_AFTER_WRITE=1  once an `otp set` has succeeded on this state,
                           every later `otp get` fails (exit 247)
  DEVICES=2                a second board (other serial, blank OTP) is attached
  FAKE_REQUIRE_SER=1       a device-touching call without --ser exits 99. The
                           bare `info` that counts boards is exempt: it is the
                           one call the tool makes on purpose without --ser
                           (plan section 3.1), and it reads no OTP and writes
                           nothing.
  FAIL_LOAD_VERIFY=1       `load -v` reports a verify failure (exit 245)
  ERASE_SKIP_BYTE=<off>    `erase` leaves the byte at flash offset <off> as it was
  FAIL_GET_AFTER=N         the first N `otp get` calls succeed, every later one
                           fails (exit 247); the count lives in <state>.getcount
  NOISY_COPY_READS=1       `-c N` on a named register prints a RAW_VALUE list and
                           the WARNING (F-619's noisy shape), whatever the copies
                           hold; the VALUE is still the one-copy read
  NOISY_BARE_READS=1       a bare read of an unnamed copy row (0x049, 0x04a, ...)
                           prints a RAW_VALUE and the WARNING the same way
  SAVE_SHORT=1             `save` exits 0 but leaves the file one byte short
  SAVE_NO_FILE=1           `save` exits 0 but leaves no file (a local I/O loss)
"""
import json
import os
import sys

FLASH_BASE = 0x10000000
FLASH_WINDOW = 32 * 1024 * 1024
SECTOR = 4096

# ---------------------------------------------------------------------------
# The register table: the subset of picotool 2.3.1's table (generated from
# pico-sdk 2.3.1 otp_data.h; `picotool otp list -n` on the real binary) that
# the tools read. Rows absent here are unnamed, as 0x039-0x03f, 0x041-0x047,
# 0x049, 0x04a, 0x04c, 0x04d, 0x05a, 0x05b and the user rows are in the real
# table. kind: 'ecc', 'crit' or 'rbit'.
# ---------------------------------------------------------------------------


class Reg:
    def __init__(self, row, name, kind, redundancy, fields=(), seq=None):
        self.row = row
        self.name = 'OTP_DATA_' + name
        self.kind = kind
        self.redundancy = redundancy
        self.fields = list(fields)  # (name, lo, hi), high bits first
        self.seq = seq  # (index 1-based, length)

    @property
    def ecc(self):
        return self.kind == 'ecc'

    @property
    def crit(self):
        return self.kind == 'crit'


def _bf0_fields(sdk):
    f = [('DISABLE_SRAM_WINDOW_BOOT', 21, 21), ('DISABLE_XIP_ACCESS_ON_SRAM_ENTRY', 20, 20),
         ('DISABLE_BOOTSEL_UART_BOOT', 19, 19), ('DISABLE_BOOTSEL_USB_PICOBOOT_IFC', 18, 18),
         ('DISABLE_BOOTSEL_USB_MSD_IFC', 17, 17), ('DISABLE_WATCHDOG_SCRATCH', 16, 16),
         ('DISABLE_POWER_SCRATCH', 15, 15), ('ENABLE_OTP_BOOT', 14, 14), ('DISABLE_OTP_BOOT', 13, 13),
         ('DISABLE_FLASH_BOOT', 12, 12), ('ROLLBACK_REQUIRED', 11, 11), ('HASHED_PARTITION_TABLE', 10, 10),
         ('SECURE_PARTITION_TABLE', 9, 9), ('DISABLE_AUTO_SWITCH_ARCH', 8, 8), ('SINGLE_FLASH_BINARY', 7, 7),
         ('OVERRIDE_FLASH_PARTITION_SLOT_SIZE', 6, 6), ('FLASH_DEVINFO_ENABLE', 5, 5),
         ('FAST_SIGCHECK_ROSC_DIV', 4, 4), ('FLASH_IO_VOLTAGE_1V8', 3, 3),
         ('ENABLE_BOOTSEL_NON_DEFAULT_PLL_XOSC_CFG', 2, 2), ('ENABLE_BOOTSEL_LED', 1, 1)]
    if sdk != '2.3.1':
        # pico-sdk 2.2.0 still has bit 0; 2.3.1 removed it (PT D2). This is
        # what the `otp list` fingerprint tells apart.
        f.append(('DISABLE_BOOTSEL_EXEC2', 0, 0))
    return f


def register_table(sdk):
    regs = {}
    for i in range(4):
        regs[i] = Reg(i, 'CHIPID%d' % i, 'ecc', 1, seq=(i + 1, 4))
    regs[0x038] = Reg(0x038, 'CRIT0', 'crit', 8, [('RISCV_DISABLE', 1, 1), ('ARM_DISABLE', 0, 0)])
    regs[0x040] = Reg(0x040, 'CRIT1', 'crit', 8, [
        ('GLITCH_DETECTOR_SENS', 5, 6), ('GLITCH_DETECTOR_ENABLE', 4, 4), ('BOOT_ARCH', 3, 3),
        ('DEBUG_DISABLE', 2, 2), ('SECURE_DEBUG_DISABLE', 1, 1), ('SECURE_BOOT_ENABLE', 0, 0)])
    regs[0x048] = Reg(0x048, 'BOOT_FLAGS0', 'rbit', 3, _bf0_fields(sdk))
    regs[0x04b] = Reg(0x04b, 'BOOT_FLAGS1', 'rbit', 3, [
        ('DOUBLE_TAP', 19, 19), ('DOUBLE_TAP_DELAY', 16, 18), ('KEY_INVALID', 8, 11), ('KEY_VALID', 0, 3)])
    regs[0x054] = Reg(0x054, 'FLASH_DEVINFO', 'ecc', 1, [
        ('CS1_SIZE', 12, 15), ('CS0_SIZE', 8, 11), ('D8H_ERASE_SUPPORTED', 7, 7), ('CS1_GPIO', 0, 5)])
    regs[0x059] = Reg(0x059, 'USB_BOOT_FLAGS', 'rbit', 3, [
        ('DP_DM_SWAP', 23, 23), ('WHITE_LABEL_ADDR_VALID', 22, 22),
        ('WL_INFO_UF2_TXT_BOARD_ID_STRDEF_VALID', 15, 15), ('WL_INFO_UF2_TXT_MODEL_STRDEF_VALID', 14, 14),
        ('WL_INDEX_HTM_REDIRECT_NAME_STRDEF_VALID', 13, 13), ('WL_INDEX_HTM_REDIRECT_URL_STRDEF_VALID', 12, 12),
        ('WL_SCSI_INQUIRY_VERSION_STRDEF_VALID', 11, 11), ('WL_SCSI_INQUIRY_PRODUCT_STRDEF_VALID', 10, 10),
        ('WL_SCSI_INQUIRY_VENDOR_STRDEF_VALID', 9, 9), ('WL_VOLUME_LABEL_STRDEF_VALID', 8, 8),
        ('WL_USB_CONFIG_ATTRIBUTES_MAX_POWER_VALUES_VALID', 7, 7),
        ('WL_USB_DEVICE_SERIAL_NUMBER_STRDEF_VALID', 6, 6), ('WL_USB_DEVICE_PRODUCT_STRDEF_VALID', 5, 5),
        ('WL_USB_DEVICE_MANUFACTURER_STRDEF_VALID', 4, 4), ('WL_USB_DEVICE_LANG_ID_VALUE_VALID', 3, 3),
        ('WL_USB_DEVICE_SERIAL_NUMBER_VALUE_VALID', 2, 2), ('WL_USB_DEVICE_PID_VALUE_VALID', 1, 1),
        ('WL_USB_DEVICE_VID_VALUE_VALID', 0, 0)])
    regs[0x05c] = Reg(0x05c, 'USB_WHITE_LABEL_ADDR', 'ecc', 1)
    if sdk == '2.3.1':
        for i in range(3):
            regs[0x062 + i] = Reg(0x062 + i, 'MAC%d' % i, 'ecc', 1, seq=(i + 1, 3))
    for n in range(4):
        for i in range(16):
            r = 0x080 + 16 * n + i
            regs[r] = Reg(r, 'BOOTKEY%d_%d' % (n, i), 'ecc', 1, seq=(i + 1, 16))
    lock0 = [('R2', 16, 23), ('R1', 8, 15), ('NO_KEY_STATE', 6, 6), ('KEY_R', 3, 5), ('KEY_W', 0, 2)]
    lock1 = [('R2', 16, 23), ('R1', 8, 15), ('LOCK_BL', 4, 5), ('LOCK_NS', 2, 3), ('LOCK_S', 0, 1)]
    for page, base in ((1, 0xf82), (2, 0xf84)):
        regs[base] = Reg(base, 'PAGE%d_LOCK0' % page, 'rbit', 1, lock0)
        regs[base + 1] = Reg(base + 1, 'PAGE%d_LOCK1' % page, 'rbit', 1, lock1)
    return regs


DESCRIPTIONS = {
    # otp set has no -n, so it prints the register description (PT:9654-9660).
    'OTP_DATA_BOOT_FLAGS0': 'Disable/Enable boot paths/features in the RP2350 mask ROM. Disables always '
                            'supersede enables. Enables are provided where there are other configurations '
                            'in OTP that must be valid. (RBIT-3)',
    'OTP_DATA_BOOT_FLAGS1': 'Disable/Enable boot paths/features in the RP2350 mask ROM. Disables always '
                            'supersede enables. Enables are provided where there are other configurations '
                            'in OTP that must be valid. (RBIT-3)',
}


# ---------------------------------------------------------------------------
# ECC: a line-for-line port of picotool's otp_calculate_ecc (PT:5469-5486).
# ---------------------------------------------------------------------------
def _even_parity(x):
    return bin(x).count('1') & 1


def otp_calculate_ecc(x):
    x &= 0xffff
    p0 = _even_parity(x & 0b1010110101011011)
    p1 = _even_parity(x & 0b0011011001101101)
    p2 = _even_parity(x & 0b1100011110001110)
    p3 = _even_parity(x & 0b0000011111110000)
    p4 = _even_parity(x & 0b1111100000000000)
    p5 = _even_parity(x) ^ p0 ^ p1 ^ p2 ^ p3 ^ p4
    p = p0 | (p1 << 1) | (p2 << 2) | (p3 << 3) | (p4 << 4) | (p5 << 5)
    return x | (p << 16)


# ---------------------------------------------------------------------------
# Word wrapping: picotool prints through clipp's formatting_ostream on stdout.
# When stdout is not a terminal the last column is 80 (PT:10180-10200). A run
# longer than the space left breaks at the last space at or before the limit,
# the space is dropped and the continuation starts at first_column + hanging
# indent. A token with no space is never broken: it overflows, and clipp does
# not advance its column, so the next text is treated as being at the start of
# the line and loses its leading space (G-facts section 1). That is why an
# 8-copy RAW_VALUE line runs into its warning: `...;0x000001(WARNING - ...`.
# ---------------------------------------------------------------------------
class Fos:
    def __init__(self, width=80):
        self.width = width
        self.lines = []
        self.cur = None   # None: no line started
        self.col = 0
        self.indent = 0
        self.first = 0
        self.hang = 0

    def set(self, first, hang):
        self.first, self.hang = first, hang

    def first_column(self, first):
        self.first = first

    def hard(self):
        self.lines.append(self.cur if self.cur is not None else '')
        self.cur = None
        self.col = 0

    def write(self, s):
        parts = s.split('\n')
        for k, p in enumerate(parts):
            if k > 0:
                self.hard()
            self._run(p)

    def _start(self, indent):
        self.indent = indent
        self.cur = ' ' * indent
        self.col = indent

    def _run(self, t):
        while t:
            if self.cur is None:
                t = t.lstrip(' ')
                if not t:
                    return
                self._start(self.first)
            elif self.col == self.indent:
                t = t.lstrip(' ')
                if not t:
                    return
            avail = self.width - self.col
            if len(t) <= avail:
                self.cur += t
                self.col += len(t)
                return
            idx = t.rfind(' ', 0, avail + 1)
            if idx >= 0:
                self.cur += t[:idx]
                self.lines.append(self.cur)
                self._start(self.first + self.hang)
                t = t[idx + 1:]
                continue
            j = t.find(' ')
            if j < 0:
                self.cur += t          # overflow; the column is NOT advanced
                return
            self.cur += t[:j]
            self.lines.append(self.cur)
            self._start(self.first + self.hang)
            t = t[j + 1:]

    def text(self):
        out = '\n'.join(self.lines)
        if self.lines:
            out += '\n'
        if self.cur is not None:
            out += self.cur
        return out


def out_fos(text, first=0, hang=0):
    f = Fos()
    f.set(first, hang)
    f.write(text)
    sys.stdout.write(f.text())


# ---------------------------------------------------------------------------
# State
# ---------------------------------------------------------------------------
def die99(msg):
    sys.stdout.write('fake-picotool: %s\n' % msg)
    sys.exit(99)


def load_state():
    path = os.environ.get('FAKE_PT_STATE')
    if not path:
        die99('FAKE_PT_STATE not set')
    with open(path) as f:
        st = json.load(f)
    st['_path'] = path
    return st


def save_state(st):
    path = st['_path']
    data = {k: v for k, v in st.items() if not k.startswith('_')}
    tmp = path + '.tmp'
    with open(tmp, 'w') as f:
        json.dump(data, f, indent=1, sort_keys=True)
        f.write('\n')
    os.replace(tmp, path)


def devices(st):
    devs = list(st.get('devices', []))
    if os.environ.get('DEVICES') == '2':
        devs.append({'serial': 'E3A0000000000002', 'rows': {}, 'flash': None, 'flash_size': 4 * 1024 * 1024,
                     '_synthetic': True})
    elif os.environ.get('DEVICES') == '0':
        devs = []
    return devs


def row_get(dev, r):
    return int(dev['rows'].get('0x%03x' % r, '0x0'), 16)


def row_set(dev, r, v):
    dev['rows']['0x%03x' % r] = '0x%06x' % v


def written_marker(st):
    return st['_path'] + '.written'


# ---------------------------------------------------------------------------
# argv helpers
# ---------------------------------------------------------------------------
def get_int(s):
    try:
        if s.lower().startswith('0x'):
            return int(s, 16)
        if s.isdigit():
            return int(s, 10)
    except ValueError:
        pass
    return None


def take_ser(args):
    """Strip a trailing/embedded `--ser S` (device-selection). Other
    device-selection options are not implemented."""
    for a in args:
        if a in ('--bus', '--address', '--vid', '--pid', '-f', '-F', '--rp2040'):
            die99('device-selection option %s not implemented' % a)
    return args


def no_device_msg(otp, ser):
    kind = 'RP2350' if otp else 'RP-series'
    msg = 'No accessible %s devices in BOOTSEL mode were found' % kind
    if ser is not None:
        msg += ' with serial number %s.' % ser
    else:
        msg += '.'
    out_fos(msg + '\n')
    sys.exit(249)


def select_device(st, ser, otp, touching=True):
    if touching and ser is None and os.environ.get('FAKE_REQUIRE_SER') == '1':
        die99('device-touching call without --ser (FAKE_REQUIRE_SER=1)')
    devs = devices(st)
    if ser is not None:
        devs = [d for d in devs if d['serial'] == ser]   # strcmp, case-sensitive
    if not devs:
        no_device_msg(otp, ser)
    if len(devs) > 1:
        sys.stdout.write('ERROR: Command requires a single RP-series device to be targeted.\n')
        sys.exit(248)
    return devs[0]


# ---------------------------------------------------------------------------
# Selectors (filter_otp, PT:7598-7665), strict
# ---------------------------------------------------------------------------
def find_reg(regs, name):
    u = name.upper()
    for r in regs.values():
        if r.name == u or r.name == 'OTP_DATA_' + u:
            return r
    return None


def parse_selector(regs, sel):
    """-> (row, mask, reg, field) or exits 99."""
    reg_sel, _, field_sel = sel.partition('.')
    row = get_int(reg_sel)
    reg = None
    if row is not None:
        if row < 0 or row >= 0x1000:
            die99('row out of range in selector %s' % sel)
        reg = regs.get(row)
    else:
        reg = find_reg(regs, reg_sel)
        if reg is None:
            die99('argv out of order or unknown selector: %r is not a row number, a register name or '
                  'REG.FIELD' % sel)
        row = reg.row
    if not field_sel:
        return (row, 0xffffff if reg else 0xffffffff, reg, None)
    if reg is None:
        die99('field selector on an unnamed row: %s' % sel)
    for f in reg.fields:
        if f[0] == field_sel.upper():
            lo, hi = f[1], f[2]
            return (row, ((2 << hi) - (1 << lo)) & 0xffffff, reg, f)
    die99('unknown field in selector %s' % sel)


def parse_ordered(args, groups, cmd):
    """Parse options in declaration order. groups: list of (flag, takes_value).
    Returns (opts dict, positionals list, ser). Exits 99 on any misorder."""
    opts = {}
    pos = []
    ser = None
    last = -1
    order = {g[0]: i for i, g in enumerate(groups)}
    i = 0
    while i < len(args):
        a = args[i]
        if a == '--ser':
            if i + 1 >= len(args):
                die99('--ser without a value')
            if ser is not None:
                die99('--ser given twice')
            if pos:
                die99('argv out of order: --ser after a selector in `%s`' % cmd)
            idx = order['--ser']
            if idx <= last:
                die99('argv out of order: --ser before an earlier option group in `%s`' % cmd)
            last = idx
            ser = args[i + 1]
            i += 2
            continue
        if a.startswith('-') and get_int(a) is None:
            if a not in order:
                die99('flag %s not implemented for `%s`' % (a, cmd))
            if pos:
                die99('argv out of order: option %s after a selector in `%s`' % (a, cmd))
            idx = order[a]
            if idx <= last:
                die99('argv out of order: %s after a later option group in `%s`' % (a, cmd))
            last = idx
            if groups[idx][1]:
                if i + 1 >= len(args):
                    die99('%s without a value' % a)
                opts[a] = args[i + 1]
                i += 2
            else:
                opts[a] = True
                i += 1
            continue
        pos.append(a)
        i += 1
    return opts, pos, ser


# ---------------------------------------------------------------------------
# otp get (PT:9162-9330)
# ---------------------------------------------------------------------------
def cmd_otp_get(st, args, regs):
    groups = [('-c', True), ('-r', False), ('-e', False), ('-n', False), ('--ser', False)]
    opts, pos, ser = parse_ordered(args, groups, 'otp get')
    if not pos:
        die99('otp get with no selector')
    if '-n' not in opts:
        die99('otp get without -n (descriptions) is not implemented')
    dev = select_device(st, ser, otp=True)
    if os.environ.get('FAIL_READ_AFTER_WRITE') == '1' and os.path.exists(written_marker(st)):
        sys.stdout.write('ERROR: Communication with RP2350 device failed\n')
        sys.exit(247)
    after = os.environ.get('FAIL_GET_AFTER')
    if after is not None:
        cpath = st['_path'] + '.getcount'
        n = int(open(cpath).read()) if os.path.exists(cpath) else 0
        with open(cpath, 'w') as f:
            f.write('%d\n' % (n + 1))
        if n >= int(after):
            sys.stdout.write('ERROR: Communication with RP2350 device failed\n')
            sys.exit(247)
    copies_opt = int(opts['-c'], 0) if '-c' in opts else -1
    raw_flag = '-r' in opts
    ecc_flag = '-e' in opts
    matches = {}
    for s in pos:
        m = parse_selector(regs, s)
        matches.setdefault((m[0], m[1]), m)
    fos = Fos()
    first = True
    last_row = None
    for key in sorted(matches):
        row, mask, reg, field = matches[key]
        do_ecc = ecc_flag
        redundancy = copies_opt
        # COPIES_IGNORED: the vote, in the equal-copies shape (F20: no RAW_VALUE
        # and no WARNING under -c), i.e. what -c 1 would print if it read the vote.
        quiet_vote = False
        if reg is not None and os.environ.get('COPIES_IGNORED') == '1':
            quiet_vote = copies_opt >= 0
            redundancy = -1
        # NOISY_COPY_READS / NOISY_BARE_READS: the one-copy reads the tool makes
        # (`-c N` on a named register; a bare read of an unnamed copy row) print
        # a RAW_VALUE list and the WARNING, whatever the copies hold.
        noisy_rows = 0
        if reg is not None and copies_opt >= 0 and os.environ.get('NOISY_COPY_READS') == '1':
            noisy_rows = max(reg.redundancy, 1)
        if reg is None and os.environ.get('NOISY_BARE_READS') == '1' and any(
                r.redundancy > 1 and r.row < row < r.row + r.redundancy for r in regs.values()):
            noisy_rows = 1
        corrected = 0
        if row != last_row:
            last_row = row
            fos.set(0, 7)
            if not first:
                fos.hard()
            first = False
            hdr = 'ROW 0x%04x' % row
            if reg is not None:
                hdr += ': ' + reg.name
                if reg.ecc:
                    hdr += ' (ECC)'
                elif reg.crit:
                    hdr += ' (CRIT)'
                elif reg.redundancy:
                    hdr += ' (RBIT-%d)' % reg.redundancy
                if reg.seq:
                    hdr += ' (Part %d/%d)' % reg.seq
                do_ecc = do_ecc or (reg.ecc and not raw_flag)
                if redundancy < 0:
                    redundancy = reg.redundancy
            fos.write(hdr)
            fos.write('\n')
            fos.set(4, 10)
            raw = row_get(dev, row)
            raw_buf = 'RAW_VALUE=0x%06x' % raw
            for i in range(1, max(redundancy, 1)):
                raw = row_get(dev, row + i)
                raw_buf += ';0x%06x' % raw
                if 3 == (raw >> 22):
                    raw ^= 0xffffff
                    fos.write('(flipping raw value to 0x%08x)' % raw)
            if noisy_rows:
                fos.write('RAW_VALUE=' + ';'.join('0x%06x' % row_get(dev, row + i)
                                                  for i in range(noisy_rows)))
                fos.write(" (WARNING - REDUNDANT ROWS AREN'T EQUAL)")
            if do_ecc:
                corrected = otp_calculate_ecc(raw & 0xffff)
                fos.write('\nVALUE 0x%04x\n' % (corrected & 0xffff))
                if corrected != raw:
                    fos.write(raw_buf)
                    fos.write(" (WARNING - ECC IS INVALID)")
            elif redundancy > 0:
                sets = [0] * 24
                clears = [0] * 24
                diff = False
                crit = reg.crit if reg is not None else False
                for i in range(redundancy):
                    v = row_get(dev, row + i)
                    for b in range(24):
                        if v & (1 << b):
                            sets[b] += 1
                        else:
                            clears[b] += 1
                for b in range(24):
                    if sets[b] >= clears[b] or (crit and sets[b] >= 3):
                        corrected |= (1 << b)
                    if sets[b] and clears[b]:
                        diff = True
                if diff and not quiet_vote and not noisy_rows and os.environ.get('SUPPRESS_WARNING') != '1':
                    if os.environ.get('SUPPRESS_RAW_VALUE') == '1':
                        fos.write("(WARNING - REDUNDANT ROWS AREN'T EQUAL)")
                    else:
                        fos.write(raw_buf)
                        fos.write(" (WARNING - REDUNDANT ROWS AREN'T EQUAL)")
                fos.write('\nVALUE 0x%06x\n' % corrected)
            else:
                corrected = raw
                fos.write('\nVALUE 0x%06x\n' % corrected)
            fos.write('\n')
        else:
            corrected = None
        if reg is not None:
            if corrected is None:
                corrected = 0
            for (fname, lo, hi) in reg.fields:
                fmask = ((2 << hi) - (1 << lo)) & 0xffffff
                if fmask & mask:
                    fos.set(4, 10)
                    fos.write('field ' + fname)
                    if lo == hi:
                        fos.write(' (bit %d)' % lo)
                    else:
                        fos.write(' (bits %d-%d)' % (lo, hi))
                    fos.write(' = %x\n' % ((corrected & fmask) >> lo))
    sys.stdout.write(fos.text())
    return 0


# ---------------------------------------------------------------------------
# otp set (PT:9600-9740; boot ROM BR varm_otp.c:392-407)
# ---------------------------------------------------------------------------
def cmd_otp_set(st, args, regs):
    # Synopsis: otp set [-c N] [-r] [-e] [-s] [-i f] [-z] <selector> <value> [device-selection]
    ser = None
    if '--ser' in args:
        k = args.index('--ser')
        if k != len(args) - 2:
            die99('argv out of order: `otp set` takes --ser only after <selector> <value>')
        ser = args[k + 1]
        args = args[:k]
    groups = [('-c', True), ('-r', False), ('-e', False), ('-s', False)]
    order = {g[0]: i for i, g in enumerate(groups)}
    opts = {}
    last = -1
    i = 0
    while i < len(args) and args[i].startswith('-'):
        a = args[i]
        if a not in order:
            die99('flag %s not implemented for `otp set`' % a)
        idx = order[a]
        if idx <= last:
            die99('argv out of order: %s after a later option group in `otp set`' % a)
        last = idx
        if groups[idx][1]:
            opts[a] = args[i + 1]
            i += 2
        else:
            opts[a] = True
            i += 1
    pos = args[i:]
    if len(pos) != 2:
        die99('argv out of order: `otp set` wants exactly <selector> <value> after its options, got %r' % pos)
    for p in pos:
        if p.startswith('-') and get_int(p) is None:
            die99('argv out of order: option %s after the selector in `otp set`' % p)
    row, mask, reg, field = parse_selector(regs, pos[0])
    value = get_int(pos[1])
    if value is None:
        die99('otp set value %r is not a number' % pos[1])
    dev = select_device(st, ser, otp=True)
    old = row_get(dev, row)
    redundancy = int(opts['-c'], 0) if '-c' in opts else -1
    ecc = '-e' in opts
    raw_flag = '-r' in opts
    hdr = 'ROW 0x%04x  OLD_VALUE=0x%06x' % (row, old)
    if reg is not None:
        hdr += ': ' + reg.name
        ecc = ecc or (reg.ecc and not raw_flag)
        if redundancy < 0:
            redundancy = reg.redundancy
    out_fos(hdr + '\n', 0, 7)
    if reg is not None:
        out_fos('"%s"\n' % DESCRIPTIONS.get(reg.name, reg.name + ' (fake description)'), 8, 0)
    if field is not None:
        lo, hi = field[1], field[2]
        fmask = ((2 << hi) - (1 << lo)) & 0xffffff
        out_fos('field %s %s\n' % (field[0], '(bit %d)' % lo if lo == hi else '(bits %d-%d)' % (lo, hi)), 4, 10)
        if value & (~fmask >> lo) & 0xffffffff:
            sys.stdout.write('ERROR: Value to set does not fit in field: value %06x, mask %06x\n\n'
                             % (value, fmask >> lo))
            sys.exit(248)
        value = ((value << lo) & fmask) | (old & ~fmask & 0xffffff)
    if '-s' in opts:
        value |= old
    if ~value & old:
        sys.stdout.write('ERROR: Cannot clear bits in OTP row(s): current value %06x, new value %06x\n\n'
                         % (old, value))
        sys.exit(248)
    use_ecc = ecc and not raw_flag
    if old and use_ecc:
        sys.stdout.write('ERROR: Cannot modify OTP ECC row(s)\n\n')
        sys.exit(248)
    if use_ecc:
        rows = [(row, otp_calculate_ecc(value))]
    elif redundancy > 0:
        rows = [(row + k, value) for k in range(redundancy)]
    else:
        rows = [(row, value)]
    fail_after = os.environ.get('FAIL_SET_AFTER')
    burned = 0
    for (r, v) in rows:
        if fail_after is not None and burned >= int(fail_after):
            save_state(st)
            sys.stdout.write('ERROR: Attempted to clear bits in OTP row(s)\n\n')
            sys.exit(248)
        cur = row_get(dev, r)
        if cur & ~v & 0xffffff:
            # BR varm_otp.c:401-407: the boot ROM stops at this row, after the
            # earlier rows were programmed.
            save_state(st)
            sys.stdout.write('ERROR: Attempted to clear bits in OTP row(s)\n\n')
            sys.exit(248)
        row_set(dev, r, cur | v)
        burned += 1
    save_state(st)
    with open(written_marker(st), 'w') as f:
        f.write('1\n')
    return 0


def cmd_otp_load(st, args, regs):
    ser = None
    if '--ser' in args:
        k = args.index('--ser')
        if k != len(args) - 2:
            die99('argv out of order: `otp load` takes --ser after <filename>')
        ser = args[k + 1]
        args = args[:k]
    if len(args) != 1 or args[0].startswith('-'):
        die99('only `otp load <file.json> [--ser S]` is implemented')
    dev = select_device(st, ser, otp=True)
    with open(args[0]) as f:
        j = json.load(f)
    for k, v in j.items():
        if not k.startswith('bootkey') or len(v) != 32:
            die99('otp load: only bootkeyN (32 bytes) is implemented')
        n = int(k[len('bootkey'):])
        for i in range(16):
            r = 0x080 + 16 * n + i
            data = v[2 * i] | (v[2 * i + 1] << 8)
            if row_get(dev, r):
                sys.stdout.write('ERROR: Cannot modify OTP ECC row(s)\n\n')
                sys.exit(248)
            row_set(dev, r, otp_calculate_ecc(data))
    save_state(st)
    return 0


def cmd_otp_list(st, args, regs):
    groups = [('-p', False), ('-n', False), ('-f', False)]
    opts, pos, ser = parse_ordered(args, groups + [('--ser', False)], 'otp list')
    if ser is not None:
        die99('otp list takes no device selection')
    if '-n' not in opts:
        die99('otp list without -n is not implemented')
    fos = Fos()
    for s in pos:
        reg_sel, _, field_sel = s.partition('.')
        reg = find_reg(regs, reg_sel) if get_int(reg_sel) is None else regs.get(get_int(reg_sel))
        if reg is None:
            continue   # a selector matching nothing prints nothing, exit 0
        fields = reg.fields
        if field_sel:
            fields = [f for f in fields if f[0] == field_sel.upper()]
            if not fields:
                continue
        hdr = 'ROW 0x%04x: %s' % (reg.row, reg.name)
        hdr += ' (ECC)' if reg.ecc else (' (CRIT)' if reg.crit else ' (RBIT-%d)' % reg.redundancy)
        if reg.seq:
            hdr += ' (Part %d/%d)' % reg.seq
        fos.set(0, 7)
        fos.write(hdr + '\n')
        fos.set(4, 10)
        if not fields:
            fos.write('(row has no sub-fields)\n')
        for (fname, lo, hi) in fields:
            fos.write('field %s %s\n' % (fname, '(bit %d)' % lo if lo == hi else '(bits %d-%d)' % (lo, hi)))
    sys.stdout.write(fos.text())
    return 0


# ---------------------------------------------------------------------------
# Flash (G3)
# ---------------------------------------------------------------------------
def flash_limit(dev):
    """Bytes the boot ROM permits from FLASH_BASE: CS0 size when
    FLASH_DEVINFO_ENABLE (BOOT_FLAGS0 bit 5, 2-of-3 vote) is set, else the
    16 MiB default. None means unlimited within the window (default)."""
    votes = sum(1 for k in range(3) if row_get(dev, 0x048 + k) & (1 << 5))
    if votes >= 2:
        data = row_get(dev, 0x054) & 0xffff
        code = (data >> 8) & 0xf
        return 0 if code == 0 else (4096 << code)
    return 16 * 1024 * 1024


def flash_path(st, dev):
    p = dev.get('flash')
    if not p:
        die99('device has no flash image in the state')
    if not os.path.isabs(p):
        p = os.path.join(os.path.dirname(st['_path']), p)
    return p


def permission_failure():
    sys.stdout.write('ERROR: The RP2350 device returned an error: permission failure\n')
    sys.exit(157)


def parse_flash_args(args, cmd, allowed):
    ser = None
    if '--ser' in args:
        k = args.index('--ser')
        if k != len(args) - 2:
            die99('argv out of order: `%s` takes --ser last' % cmd)
        ser = args[k + 1]
        args = args[:k]
    return args, ser


def cmd_erase(st, args):
    args, ser = parse_flash_args(args, 'erase', None)
    if len(args) != 3 or args[0] != '-r':
        die99('only `erase -r <from> <to> [--ser S]` is implemented')
    frm, to = get_int(args[1]), get_int(args[2])
    if frm is None or to is None:
        die99('erase -r wants two numbers')
    dev = select_device(st, ser, otp=False)
    frm &= ~(SECTOR - 1)
    to = (to + SECTOR - 1) & ~(SECTOR - 1)
    if frm < FLASH_BASE or to > FLASH_BASE + FLASH_WINDOW or to <= frm:
        sys.stdout.write('ERROR: Erase range not all in flash\n')
        sys.exit(248)
    limit = flash_limit(dev)
    path = flash_path(st, dev)
    size = os.path.getsize(path)
    skip = os.environ.get('ERASE_SKIP_BYTE')
    skip = int(skip, 0) if skip else None
    with open(path, 'r+b') as f:
        for a in range(frm, to, SECTOR):
            off = a - FLASH_BASE
            if off >= limit:
                sys.stdout.write('Erasing:              [==============================]  50%\n')
                permission_failure()
            phys = off % size
            f.seek(phys)
            if skip is not None and phys <= skip < phys + SECTOR:
                old = f.read(SECTOR)
                buf = bytearray(b'\xff' * SECTOR)
                buf[skip - phys] = old[skip - phys]
                f.seek(phys)
                f.write(bytes(buf))
            else:
                f.write(b'\xff' * SECTOR)
    sys.stdout.write('Erasing:              [==============================]  100%%\nErased %d bytes\n'
                     % (to - frm))
    return 0


def cmd_save(st, args):
    args, ser = parse_flash_args(args, 'save', None)
    if len(args) != 4 or args[0] != '-r':
        die99('only `save -r <from> <to> <file.bin> [--ser S]` is implemented')
    frm, to, fname = get_int(args[1]), get_int(args[2]), args[3]
    if frm is None or to is None:
        die99('save -r wants two numbers')
    if not fname.endswith('.bin'):
        die99('save to a non-.bin file is not implemented')
    dev = select_device(st, ser, otp=False)
    if to <= frm:
        sys.stdout.write('ERROR: Save range is invalid/empty\n')
        sys.exit(255)
    limit = flash_limit(dev)
    path = flash_path(st, dev)
    size = os.path.getsize(path)
    with open(fname, 'wb') as out, open(path, 'rb') as f:   # truncated BEFORE the read (PT:5044)
        a = frm
        while a < to:
            off = a - FLASH_BASE
            if off < 0 or off >= FLASH_WINDOW:
                sys.stdout.write('ERROR: Save range crosses unmapped memory\n')
                sys.exit(248)
            if off >= limit:
                out.flush()
                permission_failure()
            n = min(SECTOR - (off % SECTOR), to - a)
            phys = off % size
            f.seek(phys)
            out.write(f.read(n))
            a += n
    if os.environ.get('SAVE_SHORT') == '1':
        with open(fname, 'r+b') as out:
            out.truncate(max(to - frm - 1, 0))
    if os.environ.get('SAVE_NO_FILE') == '1':
        os.remove(fname)
    sys.stdout.write('Saving file: [==============================]  100%%\nWrote %d bytes to %s\n'
                     % (to - frm, fname))
    return 0


def cmd_load(st, args):
    args, ser = parse_flash_args(args, 'load', None)
    verify = False
    if args and args[0] == '-v':
        verify = True
        args = args[1:]
    if not args or args[0].startswith('-'):
        die99('only `load [-v] <file.bin> [-o <offset>] [--ser S]` is implemented')
    fname = args[0]
    rest = args[1:]
    offset = FLASH_BASE
    if rest:
        if len(rest) != 2 or rest[0] != '-o' or get_int(rest[1]) is None:
            die99('argv out of order or unimplemented in `load`: %r' % rest)
        offset = get_int(rest[1])
    if not fname.endswith('.bin'):
        die99('load of a non-.bin file is not implemented')
    dev = select_device(st, ser, otp=False)
    with open(fname, 'rb') as f:
        data = f.read()
    limit = flash_limit(dev)
    path = flash_path(st, dev)
    size = os.path.getsize(path)
    with open(path, 'r+b') as f:
        # guess_flash_size probes +8 MiB unless pages 0 and 1 are identical (G3).
        f.seek(0)
        p0 = f.read(256)
        p1 = f.read(256)
        if p0 != p1 and limit <= 8 * 1024 * 1024:
            permission_failure()
        off = offset - FLASH_BASE
        if off < 0 or off + len(data) > limit:
            permission_failure()
        f.seek(off % size)
        f.write(data)
    sys.stdout.write('Loading into Flash:   [==============================]  100%\n')
    if verify:
        sys.stdout.write('Verifying Flash:      [==============================]  100%\n')
        if os.environ.get('FAIL_LOAD_VERIFY') == '1':
            sys.stdout.write('  FAILED\nERROR: The device contents did not match the file\n')
            sys.exit(245)
        sys.stdout.write('  OK\n')
    return 0


def cmd_info(st, args):
    ser = None
    if args:
        if len(args) == 2 and args[0] == '--ser':
            ser = args[1]
        else:
            die99('only `info [--ser S]` is implemented for a device')
    devs = devices(st)
    if ser is not None:
        devs = [d for d in devs if d['serial'] == ser]
    if not devs:
        no_device_msg(False, ser)
    if len(devs) > 1:
        # G5, PT:4777-4791. info never refuses and exits 0.
        sys.stdout.write('Multiple RP-series devices in BOOTSEL mode found:\n')
        for k, d in enumerate(devs):
            name = 'RP2350 device at bus 1, address %d:' % (7 + k)
            sys.stdout.write('\n%s\n%s\n' % (name, '-' * (len(name) + 1)))
            sys.stdout.write('Program Information\n none\n')
        return 0
    sys.stdout.write('Program Information\n none\n')
    return 0


def main(argv):
    log = os.environ.get('FAKE_PT_ARGV_LOG')
    if log:
        with open(log, 'a') as f:
            f.write('\t'.join(argv) + '\n')
    if not argv:
        die99('no command')
    cmd = argv[0]
    if cmd == 'version':
        st = load_state()
        v = st.get('picotool_version', '2.3.1')
        if argv[1:] == ['-s']:
            sys.stdout.write(v + '\n')
        elif argv[1:] == []:
            sys.stdout.write('picotool v%s (Linux, GNU-16.2.0, Release)\n' % v)
        else:
            die99('version flags %r not implemented' % argv[1:])
        return 0
    st = load_state()
    regs = register_table(st.get('sdk', '2.3.1'))
    if cmd == 'info':
        return cmd_info(st, argv[1:])
    if cmd == 'otp':
        if len(argv) < 2:
            die99('otp with no subcommand')
        sub = argv[1]
        if sub == 'get':
            return cmd_otp_get(st, argv[2:], regs)
        if sub == 'set':
            return cmd_otp_set(st, argv[2:], regs)
        if sub == 'load':
            return cmd_otp_load(st, argv[2:], regs)
        if sub == 'list':
            return cmd_otp_list(st, argv[2:], regs)
        die99('otp %s not implemented' % sub)
    if cmd == 'erase':
        return cmd_erase(st, argv[1:])
    if cmd == 'save':
        return cmd_save(st, argv[1:])
    if cmd == 'load':
        return cmd_load(st, argv[1:])
    die99('subcommand %r not implemented' % cmd)


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
