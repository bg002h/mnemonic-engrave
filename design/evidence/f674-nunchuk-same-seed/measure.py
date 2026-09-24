#!/usr/bin/env python3
"""F-674 item 1: libnunchuk (a7cfb49, Nunchuk 2.1.1) on the composer's same-seed
Liana-key wallets.

Input:  wallets.tsv  (name, preset, slot pubkeys, keyed md1 chunks), written by
        the scratch Go test zz_f674_measure_test.go through the fork's composer.
Steps:  1. md descriptor <chunks>  -> the concrete descriptor (md 0.20.1)
        2. a PR-1746 control per wallet: the same leaves with Nunchuk's own
           internal key (sha256 over the SORTED UNIQUE leaf pubkeys)
        3. every descriptor through fableharness (HOME sandboxed)
        4. for each ACCEPT of a composer wallet: Nunchuk receive/change 0..2
           must equal `md address` for the same cards
Output: descriptors.tsv, harness.out, verdicts.txt; exits non-zero on any
        mismatch between a wallet's measured verdict and the sorted-order rule.
"""
import hashlib, os, re, subprocess, sys
HERE = os.path.dirname(os.path.abspath(__file__))
HARNESS = os.environ.get('NUNCHUK_HARNESS',
    '/scratch/code/shibboleth/.tmp/fable-nunchuk-lib/build/fableharness')
B58 = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
NUMS = bytes.fromhex('0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0')
def b58c(b):
    b = b + hashlib.sha256(hashlib.sha256(b).digest()).digest()[:4]
    n = int.from_bytes(b, 'big'); s = ''
    while n: n, r = divmod(n, 58); s = B58[r] + s
    return '1' * (len(b) - len(b.lstrip(b'\0'))) + s
def unsp(pks):  # Liana's recipe over pks in the order given
    return b58c(bytes.fromhex('0488b21e') + b'\0' * 9 + hashlib.sha256(b''.join(pks)).digest() + NUMS)
def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode: sys.exit(f'FAILED {cmd[:2]}: {r.stderr}')
    return r.stdout

rows = [l.rstrip('\n').split('\t') for l in open(os.path.join(HERE, 'wallets.tsv'))]
descs, meta = [], {}
for name, preset, pks, chunks in rows:
    pk = [bytes.fromhex(h) for h in pks.split(',')]
    desc = run(['md', 'descriptor', *chunks.split()]).strip().splitlines()[-1]
    ik = re.match(r'tr\((xpub[1-9A-HJ-NP-Za-km-z]+)/<0;1>/\*,', desc).group(1)
    # the composer's key path IS Liana's recipe over the leaves in slot order
    assert ik == unsp(pk), f'{name}: internal key is not the Liana recipe'
    srt = pk == sorted(pk)
    recv = run(['md', 'address', *chunks.split(), '--count', '3']).split()
    chg = run(['md', 'address', *chunks.split(), '--count', '3', '--change']).split()
    meta[name] = (preset, srt, recv, chg)
    descs.append((name, desc.split('#')[0]))
    # control: Nunchuk's own PR-1746 key path over the same leaves (a DIFFERENT
    # wallet; md1 cannot encode it). Accepting it shows a refusal of `name`
    # came from the key-path re-render, not from the repeated seed.
    descs.append((name + '+pr1746', desc.split('#')[0].replace(ik, unsp(sorted(set(pk))))))
with open(os.path.join(HERE, 'descriptors.tsv'), 'w') as f:
    for n, d in descs: f.write(f'{n}\t{d}\n')
home = os.path.join(HERE, 'nunhome'); os.makedirs(home, exist_ok=True)
out = subprocess.run([HARNESS], input=''.join(f'{n}\t{d}\n' for n, d in descs),
                     capture_output=True, text=True, env={**os.environ, 'HOME': home})
open(os.path.join(HERE, 'harness.out'), 'w').write(out.stdout)
blocks = {}
for blk in out.stdout.split('### ')[1:]:
    n, _, body = blk.partition('\n'); blocks[n] = body
def field(body, key):
    m = re.search(r'  ' + key + r'=(.*)', body); return m.group(1).strip() if m else None
bad = 0; lines = []
for name, (preset, srt, recv, chg) in meta.items():
    body = blocks[name]; acc = 'ParseWalletDescriptor=ACCEPT' in body
    ctl = 'ParseWalletDescriptor=ACCEPT' in blocks[name + '+pr1746']
    addr = '-'
    if acc:
        nr, nc = field(body, 'receive').split(), field(body, 'change').split()
        addr = 'EQUAL' if (nr, nc) == (recv, chg) else 'DIFFER'
        if addr != 'EQUAL': bad += 1
    ok = acc == srt and ctl
    if not ok: bad += 1
    lines.append(f'{name:22} {preset:16} leaves={"sorted" if srt else "unsorted":8} '
                 f'nunchuk={"ACCEPT" if acc else "REFUSE"} addresses_vs_md={addr:6} '
                 f'pr1746_control={"ACCEPT" if ctl else "REFUSE"} rule_holds={ok}')
lines.append(f'mismatches: {bad}')
open(os.path.join(HERE, 'verdicts.txt'), 'w').write('\n'.join(lines) + '\n')
print('\n'.join(lines)); sys.exit(1 if bad else 0)
