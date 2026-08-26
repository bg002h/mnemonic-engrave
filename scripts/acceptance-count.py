#!/usr/bin/env python3
"""Count design/ACCEPTANCE_engrave_transaction.md's verdicts and gates.

WHY THIS IS A SCRIPT. The sheet's §4.5 counts were hand-entered, and the first
recount after P3a got three of five rows wrong before this existed. A table
that nobody can regenerate is a table that drifts from the rows above it, and
those rows are what a reviewer reads.

Run it after any edit to §4 or §6 and paste the numbers.
"""
import re
import sys
import pathlib

DOC = pathlib.Path(__file__).resolve().parent.parent / "design" / "ACCEPTANCE_engrave_transaction.md"
s = DOC.read_text()

BOUNDS = [
    ("4.1 refusals", "### 4.1 The refusals", "### 4.2 The NORMATIVE"),
    ("4.2 NORMATIVE", "### 4.2 The NORMATIVE", "### 4.3 The rulings"),
    ("4.3 rulings", "### 4.3 The rulings", "### 4.4 What must be true"),
    ("4.4 close conditions", "### 4.4 What must be true", "### 4.5 Counts"),
]
HEADERS = ("| ---", "| # |", "| § |", "| ruling |", "| condition |", "| |")

def verdict(row):
    # ORDER MATTERS: "MET-DIFF" and "NOT-MET" both contain "MET".
    for v in ("MET-DIFF", "NOT-MET", "SUPERSEDED"):
        if v in row:
            return v
    return "MET" if "MET" in row else None

totals = dict.fromkeys(("MET", "MET-DIFF", "NOT-MET", "SUPERSEDED"), 0)
rows_total = 0
bad = 0
print(f"{'section':24} {'MET':>4} {'DIFF':>5} {'NOT':>4} {'SUP':>4} {'rows':>5}")
for name, a, b in BOUNDS:
    block = s[s.index(a):s.index(b)]
    rows = [l for l in block.splitlines()
            if l.startswith("| ") and not any(l.startswith(h) for h in HEADERS)]
    c = dict.fromkeys(totals, 0)
    for r in rows:
        v = verdict(r)
        if v is None:
            print(f"  UNCLASSIFIED in {name}: {r[:70]}", file=sys.stderr)
            bad += 1
            continue
        c[v] += 1
    for k in totals:
        totals[k] += c[k]
    rows_total += len(rows)
    print(f"{name:24} {c['MET']:>4} {c['MET-DIFF']:>5} {c['NOT-MET']:>4} "
          f"{c['SUPERSEDED']:>4} {len(rows):>5}")
print(f"{'spec items':24} {totals['MET']:>4} {totals['MET-DIFF']:>5} "
      f"{totals['NOT-MET']:>4} {totals['SUPERSEDED']:>4} {rows_total:>5}")

gates = s[s.index("## 6. THE GATES"):s.index("## 7. WHAT P3a FOUND")]
ids = re.findall(r"\|\s*\**(G-P\d+\.\d+)", gates)
closed = re.findall(r"\|\s*\*\*(G-P\d+\.\d+) — CLOSED", gates)
print()
print(f"gates listed in §6: {len(ids)}  (unique {len(set(ids))})")
print(f"gates marked CLOSED: {len(closed)} -> {' '.join(sorted(set(closed)))}")
print(f"gates still open:    {len(set(ids)) - len(set(closed))} -> "
      f"{' '.join(sorted(set(ids) - set(closed)))}")
sys.exit(1 if bad else 0)
