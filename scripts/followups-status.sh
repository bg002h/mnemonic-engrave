#!/usr/bin/env bash
# followups-status.sh -- make "what is open?" a command instead of archaeology.
#
# WHY IT EXISTS (F-618, 2026-09-16). design/FOLLOWUPS.md had accumulated FOUR
# closure notations -- `- CLOSED`, `~~strikethrough~~ **CLOSED**`, `SHIPPED`,
# `DOWNGRADED` -- and 319 of its 419 entries carried no status at all. No
# predicate could read the file. Three separate items were fixed and still read
# as open in one session, and a block of fourteen findings was folded into code
# without ever being filed, because nothing could show they were missing.
#
# A prose convention already said "status lives in the heading". It had been in
# the file since 2026-08-12 and was ignored by three quarters of the entries,
# which is what a convention with no gate is worth.
#
# THE RULE: every `### F-NNN` heading is followed by exactly one
# `**Status:** OPEN|CLOSED|WITHDRAWN|SUPERSEDED` line as the first line of its
# body, and a CLOSED entry cites the commit that closed it.
#
#   scripts/followups-status.sh            # check + summary, non-zero on failure
#   scripts/followups-status.sh --open     # list the open entries, nothing else
set -euo pipefail
F="$(dirname "$0")/../design/FOLLOWUPS.md"
[ -f "$F" ] || { echo "no FOLLOWUPS.md at $F"; exit 2; }

python3 - "$F" "${1:-}" <<'PY'
import re, sys
from collections import Counter
path, mode = sys.argv[1], sys.argv[2]
lines = open(path, encoding='utf-8').read().split('\n')
VALID = ('OPEN', 'CLOSED', 'WITHDRAWN', 'SUPERSEDED')

entries, problems = [], []
for i, l in enumerate(lines):
    m = re.match(r'### (F-\d+)\b(.*)', l)
    if not m:
        continue
    num, title = m.group(1), m.group(2).lstrip(' —-').strip()
    body = [x for x in lines[i+1:i+5] if x.strip()]
    first = body[0] if body else ''
    if not first.startswith('**Status:**'):
        problems.append(f"{num} (line {i+1}): no **Status:** line as its first body line")
        continue
    st = first[len('**Status:**'):].strip()
    word = st.split()[0] if st.split() else ''
    if word not in VALID:
        problems.append(f"{num} (line {i+1}): status {word!r} is not one of {'/'.join(VALID)}")
        continue
    entries.append((num, word, st, title, i))

# A closure with no commit is the shape that made the old file unreadable: the
# fact lived only in a commit message nobody could find from here. Searched over
# the WHOLE entry, not just the status line -- older entries record the sha in
# their prose, and rewriting 400 entries to move it would be churn.
#
# 28 entries predate the practice and cite nothing at all. Rather than failing
# forever or fabricating history, that count is PINNED: it may shrink, never
# grow. If it shrinks, lower the pin in the same commit -- a stale pin hides the
# next defect.
LEGACY_UNCITED = 28
bounds = [e[4] for e in entries] + [len(lines)]
uncited = []
for k, (num, word, st, title, i) in enumerate(entries):
    if word != 'CLOSED':
        continue
    body = '\n'.join(lines[i:bounds[k+1]])
    if not re.search(r'`?\b[0-9a-f]{7,40}\b`?', body):
        uncited.append(num)
if len(uncited) > LEGACY_UNCITED:
    problems.append(
        f"{len(uncited)} CLOSED entries cite no commit, above the pinned "
        f"{LEGACY_UNCITED}: {', '.join(uncited[LEGACY_UNCITED:])}")

seen = {}
for num, word, _, _, _ in entries:
    seen.setdefault(num, []).append(word)
dupes = {n: w for n, w in seen.items() if len(w) > 1}

if mode == '--open':
    for num, word, st, title, _ in entries:
        if word == 'OPEN':
            print(f"{num}  {title[:96]}")
    sys.exit(0)

tally = Counter(w for _, w, _, _, _ in entries)
print(f"design/FOLLOWUPS.md: {len(entries)} entries")
for k in VALID:
    print(f"  {k:<11} {tally.get(k, 0)}")
if dupes:
    print(f"\n  note: {len(dupes)} number(s) used by more than one entry: "
          + ', '.join(f'{n} ({"/".join(w)})' for n, w in sorted(dupes.items())))
    print("        (original)/(historical) variants are kept on purpose; two LIVE")
    print("        entries must not share a number.")
    live = {n: w for n, w in dupes.items() if w.count('OPEN') > 1}
    if live:
        problems.append("two OPEN entries share a number: " + ', '.join(sorted(live)))

if problems:
    print(f"\nFAIL ({len(problems)}):")
    for p in problems:
        print("  " + p)
    sys.exit(1)
print(f"\n  closures citing no commit: {len(uncited)} (pinned ceiling {LEGACY_UNCITED})")
if len(uncited) < LEGACY_UNCITED:
    print(f"  ^ below the pin -- LOWER LEGACY_UNCITED to {len(uncited)} in this script.")
print("\nOK: every entry carries a valid status, and no closure is newly uncited.")
PY
