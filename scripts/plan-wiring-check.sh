#!/usr/bin/env bash
#
# plan-wiring-check.sh -- referential integrity for a plan's WIRING SITES and
# VECTORS, so a retracted row cannot keep being referenced and a live row cannot
# go unbuilt.
#
# WHY THIS EXISTS (R0 P1 rounds 4 and 5)
#   plan-fold-sweep.sh matches LITERAL TOKENS a fold retracted. That catches a
#   fact corrected in prose and left standing in a table -- but it cannot catch a
#   reference that shares no token with what it falsified. Both Criticals in R0
#   round 4 were exactly that shape, and both cost a full opus round:
#
#     C1  v5 added W11-W13 and never retracted the W6/W7 rows they replace, so
#         section 4's step 6 still said "the wiring -- W1-W10" and section 6 still
#         asserted W6 and W7. An implementer would have built the RETRACTED
#         version and PASSED the gate, because W7's assertion was "the variant
#         exists and is Copy" -- which nothing has to produce.
#     C2  A vector was redefined from a record into a whole PAYLOAD and the step
#         that consumes it was carried across unexamined, leaving that step's
#         gate unable to go green at all.
#
#   Neither shared a token with the text it broke. Both are one grep away IF you
#   ask the structural question instead of the lexical one.
#
# WHAT IT DOES
#   1. Parses the wiring table into LIVE rows and RETRACTED rows (a row is
#      retracted when its site cell is struck with ~~...~~).
#   2. Flags every reference to a retracted W outside a line that also says
#      RETRACTED -- i.e. every place still telling a reader to build it.
#   3. Flags every reference to a W with no row at all.
#   4. Flags every LIVE W that no step in the TDD-order table builds.
#   4b. Flags a STRUCK rule or vector still referenced as LIVE somewhere else.
#   5. Parses the vector table and flags any vector no step's test column names,
#      and any vector named by more than one step.
#
# WHAT IT DOES NOT DO -- a gate that hides its blind spot is worse than no gate
#   * It checks that a W is REFERENCED by a step, never that the step builds the
#     right thing, or in a feasible ORDER. C2's real defect was ordering -- a
#     gate placed before the code that lets it pass -- and this script would NOT
#     have caught it. Only rule 5's "named by exactly one step" half applies.
#   * It cannot judge prose. "these ten sites" is invisible to it; only an
#     explicit W-token is seen. Pair it with plan-fold-sweep.sh, which is the
#     lexical half.
#   * Its table detection is regex over markdown. A row that does not match the
#     expected shape is silently not a row, so a PASS on a malformed table means
#     little -- run plan-table-check.sh first.
#
# USAGE
#   ./scripts/plan-wiring-check.sh <plan.md>
# EXIT
#   0 = clean, 1 = findings, 2 = usage/parse failure

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "usage: $0 <plan.md>" >&2
  exit 2
fi
DOC="$1"
if [ ! -f "$DOC" ]; then
  echo "$0: no such file: $DOC" >&2
  exit 2
fi

python3 - "$DOC" <<'PYEOF'
import re, sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()
lines = text.split("\n")

# ---------- 1. the wiring table: live rows vs retracted rows ----------
# A wiring row starts with | W<n> | or | **W<n>** | or | ~~W<n>~~ |
ROW = re.compile(r"^\|\s*(~~)?\*{0,2}(W\d+)\*{0,2}(~~)?\s*\|(.*)$")
live, retracted, row_line = {}, {}, {}
for i, l in enumerate(lines, 1):
    m = ROW.match(l)
    if not m:
        continue
    w = m.group(2)
    # Retraction is decided ONLY by strike-through on the W token itself. Reading
    # the word "RETRACTED" out of the cell body misfires: a LIVE row that explains
    # why its neighbour was struck ("NOT a TxRule arm: that variant is W7's and W7
    # is RETRACTED") would classify itself as dead. Measured on v7 -- W8.
    struck = bool(m.group(1))
    # a closure-table row (indented, no site cell) is an assertion, not a definition
    if l.startswith("  |"):
        continue
    (retracted if struck else live)[w] = i
    row_line[w] = i

if not live:
    print("plan-wiring-check: found no wiring rows -- refusing to report a pass", file=sys.stderr)
    sys.exit(2)

# ---------- 2. the step table: which steps name which W and which V ----------
# SCOPED to the TDD-order section. Reading numbered rows document-wide is a
# FALSE-PASS path: any other numbered table (a bit-index table, a numbered list
# of stale statements) contributes text, and a W or V named there would count as
# "built by a step" when no step builds it. Measured 2026-08-25 -- three bit rows
# and five stale-statement rows were being counted as steps.
STEP = re.compile(r"^\|\s*\*{0,2}(\d+)\*{0,2}\s*\|(.*)$")
tdd_start = tdd_end = None
for i, l in enumerate(lines):
    if re.match(r"^##+\s*4\.\s", l) and tdd_start is None:
        tdd_start = i
    elif tdd_start is not None and re.match(r"^##+\s*(4\.\d|5\.)", l):
        tdd_end = i
        break
if tdd_start is None:
    print("plan-wiring-check: no section 4 heading found -- refusing to report a pass", file=sys.stderr)
    sys.exit(2)
if tdd_end is None:
    tdd_end = len(lines)

steps = {}
for i, l in enumerate(lines, 1):
    if not (tdd_start < i <= tdd_end):
        continue
    m = STEP.match(l)
    if m and not l.startswith("  |"):
        steps.setdefault(m.group(1), []).append((i, m.group(2)))

step_text = {n: " ".join(t for _, t in rows) for n, rows in steps.items()}

def ws_in(s):
    out = set()
    for a, b in re.findall(r"W(\d+)\s*[–-]\s*W(\d+)", s):     # ranges: W1-W5
        out |= {f"W{k}" for k in range(int(a), int(b) + 1)}
    out |= set(re.findall(r"\bW\d+\b", s))
    return out

built_by = {}
for n, t in step_text.items():
    for w in ws_in(t):
        built_by.setdefault(w, []).append(n)

findings = []

# ---------- rule 2: a retracted W still PRESCRIBED by a step or asserted in closure ----------
# Deliberately narrow. Prose may discuss a retraction freely -- "W6's prescribed
# edit is actively wrong" is the plan explaining itself and is not a defect. What
# IS a defect is a place that tells an implementer to BUILD it or a closure row
# that asserts it, because those are the two surfaces that get executed.
# Scoped this way after the first version reported 16 findings on v7, of which
# every single one was prose. A gate that is red for non-defects trains a reader
# to ignore it just as surely as one that is green for everything.
prescriptive = {}
for n, rows in steps.items():
    for i, t in rows:
        prescriptive[i] = (f"step {n}", t)
for i, l in enumerate(lines, 1):
    if l.startswith("  |") and re.match(r"^\s*\|\s*\*{0,2}W\d+", l):
        prescriptive[i] = ("closure table", l)

# A cell that names a retracted site IN ORDER TO SAY SO ("NOT W6/W7, which are
# RETRACTED", "replacing v4's W6/W7 rows") is the fix, not the defect. Suppress it.
# BLIND SPOT, stated because a gate that hides one is worse than no gate: this
# also suppresses a cell that says "RETRACTED" about one site while genuinely
# prescribing another. If a step's cell carries any of these words, this rule is
# off for that cell -- rules 3, 4 and 5 still apply to it.
EXEMPT = re.compile(r"RETRACTED|retracted|replacing|superseded|SUPERSEDED", re.I)

for w, defline in sorted(retracted.items(), key=lambda kv: int(kv[0][1:])):
    for i, (where, t) in sorted(prescriptive.items()):
        if w in ws_in(t) and not EXEMPT.search(t):
            findings.append((f"{w} is RETRACTED (row at line {defline}) but {where} "
                             f"still names it", i, t.strip()[:110]))

# ---------- rule 3: a W referenced with no row at all ----------
all_refs = {}
for i, l in enumerate(lines, 1):
    for w in re.findall(r"\bW\d+\b", l):
        all_refs.setdefault(w, []).append(i)
for w, where in sorted(all_refs.items(), key=lambda kv: int(kv[0][1:])):
    if w not in live and w not in retracted:
        findings.append((f"{w} is referenced but has NO row in the wiring table",
                         where[0], f"{len(where)} reference(s), first at line {where[0]}"))

# ---------- rule 4: a live W no step builds ----------
for w, defline in sorted(live.items(), key=lambda kv: int(kv[0][1:])):
    if w not in built_by:
        findings.append((f"{w} is a LIVE wiring site that NO step builds",
                         defline, "an implementer following section 4 never writes it"))

# ---------- rule 4b: a STRUCK rule or vector still referenced as LIVE ----------
# Added 2026-08-25. Striking a row in place leaves every OTHER mention of it
# reading as live -- measured at 48 references after one simplification pass, of
# which several were load-bearing (a field table citing a deleted rule, a
# near-miss pair citing a deleted vector). A reviewer reporting these is a
# reviewer paid design-review rates to act as a grep.
STRIKE_LANG = re.compile(r"STRUCK|DELETED|struck|RETRACTED|retracted|SIMPLIFICATION|inexpressible|collapse", re.I)
# A RANGE label ("E1-E20", "V1-V26") names a numbering span, not a claim that
# every member is live. Flagging those buried the three real defects under 16
# false ones on the first run. The span is stated once in the plan instead.
RANGE = re.compile(r"[EV]\d+\s*[\u2013-]\s*[EV]\d+")

def struck_names(prefix):
    out = {}
    for i, l in enumerate(lines, 1):
        m = re.match(rf"^\|\s*~~\*{{0,2}}({prefix}\d+[a-z]?)\*{{0,2}}~~\s*\|", l)
        if m:
            out[m.group(1)] = i
    return out

for prefix, what in (("E", "rule"), ("V", "vector")):
    dead = struck_names(prefix)
    for nm, defline in sorted(dead.items(), key=lambda kv: int(re.sub(r"\D", "", kv[0]))):
        for i, l in enumerate(lines, 1):
            if i == defline or STRIKE_LANG.search(l):
                continue
            if l.lstrip().startswith("| ~~"):
                continue
            if RANGE.search(l):
                continue
            if re.search(rf"\b{nm}\b(?![a-z0-9])", l):
                findings.append((f"{nm} is a STRUCK {what} (row at line {defline}) but line {i} "
                                 f"references it as live", i, l.strip()[:110]))

# ---------- rule 5: vectors named by exactly one step ----------
VROW = re.compile(r"^\|\s*\*{0,2}(V\d+[a-z]?)\*{0,2}\s*\|")
vectors = []
for l in lines:
    m = VROW.match(l)
    if m and m.group(1) not in vectors:
        vectors.append(m.group(1))

def vs_in(s):
    """Vectors a step's cell names, expanding V-ranges like V5-V6 and V16-V17b."""
    out = set()
    for a, _, b, _ in re.findall(r"V(\d+)([a-z]?)\s*[–-]\s*V(\d+)([a-z]?)", s):
        for k in range(int(a), int(b) + 1):
            out |= {v for v in vectors if re.fullmatch(rf"V{k}[a-z]?", v)}
    out |= {v for v in vectors if re.search(rf"\b{v}\b", s)}
    return out

named_by = {}
for n, t in step_text.items():
    for v in vs_in(t):
        named_by.setdefault(v, []).append(n)

for v in vectors:
    who = sorted(set(named_by.get(v, [])), key=int)
    if not who:
        findings.append((f"{v} is in the vector table but NO step names it",
                         0, "it would be constructed and never asserted"))

# ---------- report ----------
print(f"--- wiring rows: {len(live)} live, {len(retracted)} retracted")
print(f"--- steps parsed: {len(step_text)}   vectors parsed: {len(vectors)}")
live_list = " ".join(sorted(live, key=lambda w: int(w[1:])))
ret_list = " ".join(sorted(retracted, key=lambda w: int(w[1:]))) or "(none)"
print(f"--- live:      {live_list}")
print(f"--- retracted: {ret_list}")
print()
if not findings:
    print("--- PASS: every retracted site is referenced only as retracted, every live")
    print("---       site is built by a step, and every vector is named by a step.")
else:
    for msg, ln, detail in findings:
        print(f"  FINDING  {msg}")
        if detail:
            print(f"           {detail}")
print()
print("--- NOT covered: whether a step builds the RIGHT thing, whether the step")
print("---              ORDER is feasible (R0 r4-C2 was an ordering defect and")
print("---              this script would NOT have caught it), and prose counts")
print("---              like \"these ten sites\" that name no W token.")
sys.exit(1 if findings else 0)
PYEOF
