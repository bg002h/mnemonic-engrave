#!/usr/bin/env python3
"""fake_otp_state.py -- build and poke the state file fake_picotool.py serves.

Test-only. Used by scripts/test/run-e2e-otp.sh; never by the tools themselves.

  new <state> --serial S --chipid 0xHEX [--flash-size N]
                                  CHIPID rows (ECC), every other row 0, a flash
                                  image <state>.flash filled with a non-0xFF
                                  pattern (so an erase is observable)
  shape <state> retail|rehearsal [--slot0 HASH] [--slot1 HASH] [--kv N]
                                  the profile's expected rows (plan section 3.2)
  set-row <state> <row> <raw24>   one raw row
  set-copies <state> <row0> <n> <raw24>
  set-ecc <state> <row> <data16>  one ECC row (raw = data | ECC parity)
  set-slot <state> <n> <hash64>   boot-key slot n (16 ECC rows, low byte first)
  get-row <state> <row>
  set <state> <key> <value>       picotool_version, sdk
  white-label <state> <addr-row> <usb_boot_flags> <entries-json>
                                  write a white-label table and its strings
  flash-byte <state> <offset>     print one flash byte (hex)
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fake_picotool import otp_calculate_ecc  # noqa: E402


def load(p):
    with open(p) as f:
        return json.load(f)


def save(p, st):
    with open(p + '.tmp', 'w') as f:
        json.dump(st, f, indent=1, sort_keys=True)
        f.write('\n')
    os.replace(p + '.tmp', p)


def setrow(st, r, v):
    st['devices'][0]['rows']['0x%03x' % r] = '0x%06x' % (v & 0xffffff)


def getrow(st, r):
    return int(st['devices'][0]['rows'].get('0x%03x' % r, '0x0'), 16)


def setslot(st, n, h):
    b = bytes.fromhex(h)
    assert len(b) == 32
    for i in range(16):
        setrow(st, 0x080 + 16 * n + i, otp_calculate_ecc(b[2 * i] | (b[2 * i + 1] << 8)))


def main(a):
    cmd, p = a[0], a[1]
    if cmd == 'new':
        opts = dict(zip(a[2::2], a[3::2]))
        cid = int(opts['--chipid'], 16)
        size = int(opts.get('--flash-size', str(4 * 1024 * 1024)), 0)
        st = {'picotool_version': '2.3.1', 'sdk': '2.3.1',
              'devices': [{'serial': opts['--serial'], 'rows': {}, 'flash': os.path.basename(p) + '.flash',
                           'flash_size': size}]}
        for i in range(4):
            setrow(st, i, otp_calculate_ecc((cid >> (16 * i)) & 0xffff))
        save(p, st)
        pat = bytes((i * 37 + 11) & 0xff for i in range(4096))
        with open(p + '.flash', 'wb') as f:
            for _ in range(size // 4096):
                f.write(pat)
        return 0
    st = load(p)
    if cmd == 'shape':
        kind = a[2]
        opts = dict(zip(a[3::2], a[4::2]))
        kv = int(opts.get('--kv', '3'), 0)
        for r in range(0x040, 0x048):
            setrow(st, r, 0x000001)            # CRIT1: SECURE_BOOT_ENABLE in all 8
        for r in range(0x038, 0x040):
            setrow(st, r, 0)                   # CRIT0
        for r in (0x04b, 0x04c, 0x04d):
            setrow(st, r, kv | (0x080000 if kind == 'retail' else 0))
        for r in (0x048, 0x049, 0x04a):
            setrow(st, r, 0x000020 if kind == 'retail' else 0)
        if kind == 'retail':
            setrow(st, 0x054, otp_calculate_ecc(0x0c00))   # FLASH_DEVINFO: CS0 16 MiB
        for r in (0xf82, 0xf84):
            setrow(st, r, 0)
        for r in (0xf83, 0xf85):
            setrow(st, r, 0x040404)
        if '--slot0' in opts:
            setslot(st, 0, opts['--slot0'])
        if '--slot1' in opts:
            setslot(st, 1, opts['--slot1'])
    elif cmd == 'set-row':
        setrow(st, int(a[2], 16), int(a[3], 16))
    elif cmd == 'set-copies':
        r0, n, v = int(a[2], 16), int(a[3]), int(a[4], 16)
        for k in range(n):
            setrow(st, r0 + k, v)
    elif cmd == 'set-ecc':
        setrow(st, int(a[2], 16), otp_calculate_ecc(int(a[3], 16)))
    elif cmd == 'set-slot':
        setslot(st, int(a[2]), a[3])
    elif cmd == 'get-row':
        print('0x%06x' % getrow(st, int(a[2], 16)))
        return 0
    elif cmd == 'set':
        st[a[2]] = a[3]
    elif cmd == 'white-label':
        # entries-json: {"<index>": <data16>, ...} for VALUE entries and
        # {"<index>": "string"} for ASCII STRDEF entries; strings are laid out
        # right after the 16-row table.
        addr = int(a[2], 16)
        ubf = int(a[3], 16)
        entries = json.loads(a[4])
        setrow(st, 0x05c, otp_calculate_ecc(addr))
        for r in (0x059, 0x05a, 0x05b):
            setrow(st, r, ubf)
        nxt = 16
        for k in range(16):
            setrow(st, addr + k, 0)
        for idx, val in sorted(entries.items(), key=lambda kv: int(kv[0])):
            idx = int(idx)
            if isinstance(val, str):
                data = val.encode('ascii')
                setrow(st, addr + idx, otp_calculate_ecc(((nxt & 0xff) << 8) | len(data)))
                rows = (len(data) + 1) // 2
                for j in range(rows):
                    lo = data[2 * j]
                    hi = data[2 * j + 1] if 2 * j + 1 < len(data) else 0
                    setrow(st, addr + nxt + j, otp_calculate_ecc(lo | (hi << 8)))
                nxt += rows
            else:
                setrow(st, addr + idx, otp_calculate_ecc(int(val)))
    elif cmd == 'flash-byte':
        off = int(a[2], 0)
        with open(p + '.flash', 'rb') as f:
            f.seek(off)
            print('%02x' % f.read(1)[0])
        return 0
    else:
        print('unknown command %s' % cmd, file=sys.stderr)
        return 2
    save(p, st)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
