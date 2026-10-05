#!/usr/bin/env python3
"""capture_to_entry.py <capture.json> <capture-sha256> <capture-path-in-entry>

Test-only: turn a `refugium-otp.sh capture` file into one retail-otp.json entry
(plan section 3.2: the cells the expected-state table marks "recorded entry").
Never used to write design/hardware/retail-otp.json in the repository: real
entries are added by a reviewed PR after H0.
"""
import json
import sys


def h(v):
    return '0x%06x' % v


def main(a):
    cap = json.load(open(a[0]))
    rows = {k: int(v, 16) for k, v in cap['rows'].items()}
    wl = cap.get('white_label', {})
    wl_rows = {}
    for r in wl.get('table_rows', []) + [r for s in wl.get('strings', []) for r in s['rows']]:
        wl_rows[r] = h(rows[r])
    entry = {
        'id': 'fixture-' + cap['chipid'].lower(),
        'capture': a[2],
        'capture_sha256': a[1],
        'date': '2026-10-05',
        'history': 'test fixture built from the fake; not a real unit',
        'crit0': h(rows['0x038']),
        'crit1': h(rows['0x040']),
        'boot_flags0': h(rows['0x048'] & ~0x006800 & 0xffffff),
        'boot_flags1': h(rows['0x04b'] & ~0x000f0f & 0xffffff),
        'flash_devinfo': h(rows['0x054']),
        'usb_boot_flags': h(rows['0x059']),
        'usb_white_label_addr': h(rows['0x05c']),
        'white_label_rows': wl_rows,
    }
    json.dump(entry, sys.stdout, indent=1, sort_keys=True)
    sys.stdout.write('\n')


if __name__ == '__main__':
    main(sys.argv[1:])
