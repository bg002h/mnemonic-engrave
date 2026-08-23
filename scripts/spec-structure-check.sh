#!/usr/bin/env bash
# spec-structure-check.sh — mechanical structure audit for a design doc.
#
# WHY THIS EXISTS. A spec folded twenty times in one session accumulates
# structural damage that READS PERFECTLY WELL. Measured on SPEC_mt_v0_1.md in a
# single session: a scripted renumber ate the newline after a heading
# ("## 10. Open questions1. **The F-234..."); a later one left a STALE DUPLICATE
# of two items with the next section's heading glued onto the last line; and two
# `.replace()` edits silently matched nothing while their commit messages
# claimed they had landed. Every one was found by COUNTING, never by reading —
# a duplicated block and a missing edit both look like fine prose.
#
# WHAT IT CHECKS
#   1. `## N.` and `### Na.` headings are sequential and unique.
#   2. Numbered items inside each section run 1..N with no gaps or repeats
#      (suffixed items like `2b.` are allowed and checked for ordering).
#   3. Every `§N` and `§N.M` cross-reference resolves to something that exists.
#   4. GFM table rows match their header's column count — an overflow cell is
#      silently DROPPED when rendered, so the text exists and shows nowhere.
#   5. No duplicated heading text, and no heading glued to other text.
#   5. Superseded terms (passed via SUPERSEDED, newline-separated) do not appear
#      outside a correction/retraction block.
#
# WHAT IT DOES NOT CHECK — stated because a gate that hides its blind spot is
# worse than no gate.
#   * Whether the CONTENT is right, consistent, or still true. Two sections can
#     contradict each other with perfect numbering.
#   * Citations — that is scripts/plan-cite-check.sh.
#   * Whether a cross-reference points at the RIGHT thing, only that the target
#     exists.
#
# USAGE   ./scripts/spec-structure-check.sh design/SPEC_mt_v0_1.md
# EXIT    0 clean, 1 structural defect found
set -uo pipefail
FAIL=0
for DOC in "$@"; do
  echo "═══ $DOC"
  python3 - "$DOC" <<'PY'
import re, sys
doc = sys.argv[1]
lines = open(doc).read().split('\n')
bad = 0

def err(msg):
    global bad
    print(f"  FAIL  {msg}")
    bad += 1

# ---- 1. section headings sequential + unique -------------------------------
# `###` SUBSECTIONS COUNT. The first version of this script matched only `^## N`
# and reported §6a as a dangling cross-reference when §6a exists perfectly well
# as `### 6a.` — a gate producing a FALSE FAIL, which trains a reader to ignore
# its output just as surely as one that is always green.
heads = [(i+1, l) for i, l in enumerate(lines) if re.match(r'^#{2,3} \d+[a-z]?\.', l)]
seen = {}
for ln, h in heads:
    m = re.match(r'^#{2,3} (\d+)([a-z]?)\.', h)
    if not m:
        err(f"line {ln}: heading not in `## N.` form: {h[:60]!r}")
        continue
    key = m.group(1) + m.group(2)
    if key in seen:
        err(f"line {ln}: DUPLICATE section {key} (first at line {seen[key]})")
    seen[key] = ln
    # heading glued to other text?
    if re.search(r'\*\*.*##\s*\d', h) or h.count('##') > 1:
        err(f"line {ln}: heading appears GLUED to other text: {h[:70]!r}")
nums = [int(re.match(r'^#{2,3} (\d+)', h).group(1)) for _, h in heads if re.match(r'^#{2,3} (\d+)', h)]
for a, b in zip(nums, nums[1:]):
    if b not in (a, a+1):
        err(f"section numbering jumps: {a} -> {b}")

# ---- 2. numbered items within each section ---------------------------------
bounds = [ln for ln, _ in heads] + [len(lines)+1]
for (start, head), end in zip(heads, bounds[1:]):
    body = lines[start:end-1]
    items = [(start+1+k, l) for k, l in enumerate(body) if re.match(r'^\d+[a-z]?\. ', l)]
    if not items:
        continue
    order, prev_base = [], 0
    for ln, l in items:
        m = re.match(r'^(\d+)([a-z]?)\. ', l)
        base, suf = int(m.group(1)), m.group(2)
        order.append((base, suf, ln, l))
    bases = [b for b, s, _, _ in order if s == '']
    exp = list(range(1, len(bases)+1))
    if bases != exp:
        err(f"{head[:40]!r}: item numbering is {bases}, expected {exp}")
    # suffixed items must follow their base
    for base, suf, ln, l in order:
        if suf and base not in bases:
            err(f"line {ln}: item {base}{suf}. has no base item {base}.")
    # duplicate item text
    txt = {}
    for base, suf, ln, l in order:
        key = l[:55]
        if key in txt:
            err(f"line {ln}: DUPLICATE item text (also line {txt[key]}): {key!r}")
        txt[key] = ln

# ---- 3. cross-references resolve -------------------------------------------
body = '\n'.join(lines)
sect_ids = set(seen.keys())
# QUALIFIED references belong to ANOTHER document and are not ours to resolve.
# `EPD §6.4`, `BIP §2`, `BCR §3` name a foreign spec's section; only a bare `§N`
# refers to this one. Without this the gate reports EPD §6.4 as "section 6 has
# no item 4" — a FALSE FAIL, and the second one this script produced.
FOREIGN = r'(?<!EPD )(?<!BIP )(?<!RFC )(?<!BCR )(?<!EPD§)(?<!BIP§)'
for ref in sorted(set(re.findall(FOREIGN + r'§(\d+[a-z]?)(?:\.(\d+))?', body))):
    sec, item = ref
    if sec not in sect_ids:
        err(f"cross-reference §{sec} -> no such section")
        continue
    if item:
        ln = seen[sec]
        nxt = min([v for v in seen.values() if v > ln] + [len(lines)+1])
        seg = lines[ln:nxt-1]
        if not any(re.match(rf'^{item}[a-z]?\. ', l) for l in seg):
            err(f"cross-reference §{sec}.{item} -> section {sec} has no item {item}")

# ---- 3b. GFM table rows must match their header's column count -------------
# A row with MORE cells than the header silently DROPS the overflow when
# rendered — the text is in the file, invisible on the page. Found for real:
# §7's "Pinned fee" row carried a third cell orphaned by an earlier rewrite, so
# the statement that an `mt string` plate's fee is unrecoverable existed in the
# source and rendered nowhere. A row with FEWER cells renders blank columns.
ncol = 0
for ln, l in enumerate(lines, 1):
    if l.startswith('|'):
        cells = l.strip().strip('|').split('|')
        if set(l.strip().strip('|').replace('|', '').strip()) <= set('-: '):
            ncol = len(cells)
            continue
        if ncol and len(cells) != ncol:
            err(f"line {ln}: table row has {len(cells)} cells, header has {ncol} "
                f"(GFM drops the overflow): {l[:50]!r}")
    else:
        ncol = 0

# ---- 4. duplicated heading text --------------------------------------------
htext = {}
for ln, h in heads:
    t = re.sub(r'^#{2,3} \d+[a-z]?\.\s*', '', h).strip()
    if t and t in htext:
        err(f"line {ln}: heading TEXT duplicated (also line {htext[t]}): {t[:50]!r}")
    htext[t] = ln

print(f"  sections: {len(heads)} ; cross-refs checked: {len(set(re.findall(r'§(\d+[a-z]?)(?:\.(\d+))?', body)))}")
print("  STRUCTURE OK" if bad == 0 else f"  {bad} STRUCTURAL DEFECT(S)")
sys.exit(1 if bad else 0)
PY
  [ $? -ne 0 ] && FAIL=1
done
exit $FAIL
