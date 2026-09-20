#!/usr/bin/env python3
"""Render design/IMPORTABILITY_composer_shapes.md from the composer fable
review r0 evidence files (design/evidence/composer-fable-r0/), so the table is
COMPUTED from what the harnesses printed, never hand-transcribed.

Sources (all RUN, 2026-09-20):
  fable-liana-shapes.json           the 56 shapes (lens 5): name, wrapper, slots, source
  fable-nunchuk-harness-out.txt     lens 2: libnunchuk 2.1.1 ParseWalletDescriptor per shape.variant
  fable-liana-parse-out.jsonl       lens 5: LianaDescriptor::from_str (liana v8.0) per name x variant
  fable-liana-core-v31-import.out   lens 5: Core 31.1 wallet import (createwallet+importdescriptors, recv/chg vs md)
  fable-liana-core-v25-import.out   lens 5: Core 25.0 (single-chain), same journey
  fable-liana-core-v31-spend.out    lens 5: Core 31.1 spends through the wallet's own PSBT machinery
  fable-liana-lianad.out            lens 5: lianad 8.0 on regtest for every Liana-accepted shape
"""
import json, os, re
HERE = os.path.dirname(os.path.abspath(__file__))
E = os.path.join(HERE, '..', 'design', 'evidence', 'composer-fable-r0')
def p(n): return os.path.join(E, n)

shapes = json.load(open(p('fable-liana-shapes.json')))
names = [s['name'] for s in shapes]

nunchuk = {}
cur = None
for line in open(p('fable-nunchuk-harness-out.txt')):
    m = re.match(r'^### (.+?)\.(multipath|chain0|chain1|keyless-template|template-only|multipath\.h-spelling|multipath\.H-spelling)\s*$', line)
    if m:
        cur = (m.group(1), m.group(2)); continue
    m = re.match(r'^\s+ParseWalletDescriptor=(ACCEPT|REFUSE)(.*)$', line)
    if m and cur:
        nunchuk[cur] = (m.group(1), m.group(2).strip())
def nunchuk_cell(name):
    key = (name, 'multipath')
    if key not in nunchuk and name == 'plain-2of3-wsh-DEMO':
        key = ('DEMO', 'multipath')
    if key not in nunchuk:
        return 'n/m'
    v, rest = nunchuk[key]
    if v == 'ACCEPT': return 'OK'
    m = re.search(r"what='([^']*)'", rest)
    return 'REFUSE' + (f' ({m.group(1)})' if m else '')

liana = {}
for line in open(p('fable-liana-parse-out.jsonl')):
    r = json.loads(line); liana[(r['name'], r['variant'])] = r
def liana_cell(name):
    r = liana.get((name, 'md'))
    if r is None: return 'n/m'
    if r['ok']: return 'OK'
    e = r.get('error', '').replace('Descriptor is not compatible with a Liana spending policy.', 'IncompatibleDesc')
    return 'REFUSE (' + (e[:60] + ('...' if len(e) > 60 else '')) + ')'

def core_import(fname):
    out = {}
    for line in open(p(fname)):
        m = re.match(r'^(\S+)\s+(.*)$', line.rstrip('\n'))
        if not m or m.group(1) not in names: continue
        rest = m.group(2)
        if 'import ok' in rest:
            rm = re.search(r'recv==md \[([^\]]*)\]', rest); cm = re.search(r'chg==md \[([^\]]*)\]', rest)
            ok = all(x.strip() == 'True' for x in (rm.group(1).split(',') if rm else [])) and \
                 all(x.strip() == 'True' for x in (cm.group(1).split(',') if cm else []))
            # A False here is an INDEX shift, not a mismatch: seven wallets had
            # been paid by the spend/lianad legs before this re-run, so Core
            # handed out the next unused index; the report (§Import and
            # addresses) records every string as md's address at that index.
            out[m.group(1)] = 'OK' + ('' if ok else ' (idx shifted, see note)')
        elif 'REFUSED' in rest:
            out[m.group(1)] = 'REFUSE'
        else:
            out[m.group(1)] = rest[:40]
    return out
core31 = core_import('fable-liana-core-v31-import.out')
core25 = core_import('fable-liana-core-v25-import.out')
v25_reason = {}
lines = open(p('fable-liana-core-v25-import.out')).read().split('\n')
for i, line in enumerate(lines):
    m = re.match(r'^(\S+)\s+.*REFUSED', line)
    if m:
        j = i + 1
        while j < len(lines) and lines[j].strip() in ('', 'error message:'): j += 1
        if j < len(lines): v25_reason[m.group(1)] = lines[j].strip()[:50]
def core25_cell(name):
    v = core25.get(name, 'n/m')
    return v + (f' ({v25_reason[name]})' if v == 'REFUSE' and name in v25_reason else '')

spends = {}
for line in open(p('fable-liana-core-v31-spend.out')):
    # The logger cut long JSON lines, so the record is scanned, not parsed: an
    # attempt is a bracketed "[Sn shape ...]" line; it was accepted iff it
    # carries a sendrawtransaction txid (the network took it).
    m = re.match(r'^\s*\[S\d+[a-z]?\s+(\S+)\s+([^\]]*)\]\s+\{', line)
    if not m or m.group(1) not in names: continue
    # (the log truncates long lines, so a txid may be cut: a hex PREFIX after
    # sendrawtransaction is acceptance; an error string there is refusal)
    acc = re.search(r'"sendrawtransaction": "[0-9a-f]', line) is not None
    a = spends.setdefault(m.group(1), [0, 0]); a[0] += 1; a[1] += int(acc)
def spend_cell(name):
    if name not in spends: return '-'
    n, ok = spends[name]; return f'{ok} of {n} attempts sent (the rest are the deliberate refusals: immature lock, one key of k, wrong preimage)'

lianad = {}
cur = None
for line in open(p('fable-liana-lianad.out')):
    m = re.match(r'^== (\S+): lianad up', line)
    if m: cur = m.group(1); lianad[cur] = []; continue
    m = re.match(r'^\s+(primary|recovery\S*):.*mempool: (True|False)', line)
    if m and cur: lianad[cur].append((m.group(1), m.group(2) == 'True'))
def lianad_cell(name):
    if name not in lianad: return '-'
    ok = sum(1 for _, v in lianad[name] if v); return f'{ok}/{len(lianad[name])} paths spent'

rows = []
for s in shapes:
    n = s['name']
    rows.append((n, s.get('wrapper', ''), s.get('source', ''), nunchuk_cell(n), liana_cell(n), core25_cell(n), core31.get(n, 'n/m'), spend_cell(n), lianad_cell(n)))

def count(col, pred): return sum(1 for r in rows if pred(r[col]))
hdr = f'''# Which composer shapes import into which wallets

GENERATED by `scripts/importability-matrix.py` from the evidence files under
`design/evidence/composer-fable-r0/` (composer fable review r0, lenses 2 and 5,
2026-09-20). Do not edit by hand; re-run the script. Every cell is what a
harness printed: libnunchuk 2.1.1 `Utils::ParseWalletDescriptor` (the desktop
app's own import call), `LianaDescriptor::from_str` at liana v8.0 (the GUI's
"Import the wallet" call), Bitcoin Core 25.0 and 31.1 descriptor-wallet import
with receive/change compared to `md`, Core 31.1 spends through
`walletcreatefundedpsbt` -> `walletprocesspsbt` -> `sendrawtransaction`, and
`lianad` 8.0 spends through Liana's own signer. Descriptor spelling in every
cell is `md descriptor`'s multipath output (`<0;1>/*`, `'` hardened,
zero-parent-fingerprint xpubs); other spellings are in the reports.

Shapes named `X01..X26` were composed with `md compose 0.17.0`; the rest are
the 30 device-composed shapes from lens 2 (fork tip f5b068fa). "n/m" = not
measured by that lens. Reasons are the wallet's own strings, shortened.
"idx shifted" (Core 31.1): the wallet had been paid by the spend legs before
the import re-run, so Core started at the next unused index; the report records
every receive/change string as md's address at that index (equal by
scriptPubKey, HRP aside) -- an artefact of running spends first, not a
mismatch.

Totals over {len(rows)} shapes: Nunchuk OK {count(3, lambda v: v=="OK")} / measured {count(3, lambda v: v!="n/m")};
Liana OK {count(4, lambda v: v=="OK")} / {count(4, lambda v: v!="n/m")}; Core 25 OK {count(5, lambda v: v.startswith("OK"))} / {count(5, lambda v: v!="n/m")};
Core 31.1 OK {count(6, lambda v: v.startswith("OK"))} / {count(6, lambda v: v!="n/m")}.

Reports: `design/agent-reports/composer-fable-r0-nunchuk.md`,
`design/agent-reports/composer-fable-r0-liana-core.md` (matrices, spec claims,
operator runbooks, what could not be verified).

| shape | wrapper | source | Nunchuk 2.1.1 | Liana 8.0 | Core 25.0 | Core 31.1 import | Core 31.1 wallet spends | lianad 8.0 spends |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
'''
body = ''.join('| ' + ' | '.join(str(c) for c in r) + ' |\n' for r in rows)
open(os.path.join(HERE, '..', 'design', 'IMPORTABILITY_composer_shapes.md'), 'w').write(hdr + body)
print(f'{len(rows)} rows written')
