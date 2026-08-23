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
#   3. Suffixed items (`2b.`, `7c.`) are unique and alphabetically ordered after
#      their base — a duplicate or an out-of-order suffix reads perfectly well.
#   4. Every `§N` and `§N.M` cross-reference resolves to something that exists.
#   4. GFM table rows match their header's column count — an overflow cell is
#      silently DROPPED when rendered, so the text exists and shows nowhere.
#   5. Superseded terms (design/SUPERSEDED_TERMS.txt) appear only inside a
#      correction context — matched with WHITESPACE NORMALISED so a phrase
#      wrapping across a line break cannot hide from the check.
#   6. No duplicated heading text, and no heading glued to other text.
#   5. Superseded terms (passed via SUPERSEDED, newline-separated) do not appear
#      outside a correction/retraction block.
#
# WHAT IT DOES NOT CHECK — stated because a gate that hides its blind spot is
# worse than no gate.
#   * Whether the CONTENT is right, consistent, or still true. Two sections can
#     contradict each other with perfect numbering.
#   * Citations — that is scripts/plan-cite-check.sh.
#   * Whether a cross-reference points at the RIGHT thing, only that the target
#     exists. MITIGATED, not closed: every target's first line is printed, so a
#     wrong-but-existing citation (measured: four to §10.20 meaning §1.1) is a
#     read away rather than a lookup away. Reading them is still a human's job.
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
    # MALFORMED item labels: `1a0.` or `2e0.` look like items and are INVISIBLE
    # to every check below, because the strict pattern allows one optional
    # letter. I produced both, an hour apart, and the gate passed both times.
    for k, l in enumerate(body):
        if re.match(r'^\d+[a-z0-9]+\. ', l) and not re.match(r'^\d+[a-z]?\. ', l):
            err(f"line {start+1+k}: MALFORMED item label {l.split('.')[0]!r} — "
                f"a label must be digits plus at most one letter, or it is "
                f"invisible to every numbering check")
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
    # SUFFIXED items must be unique and in alphabetical order after their base.
    # The gate passed a spec carrying TWO `1b.` items and a `7c.` printed before
    # `7b.` — it checked that base numbers ran 1..N and that a suffixed item had
    # a base, but never that the suffixes themselves were unique or ordered.
    by_base = {}
    for base, suf, ln, l in order:
        if suf:
            by_base.setdefault(base, []).append((suf, ln))
    for base, sufs in by_base.items():
        seen_suf = {}
        for suf, ln in sufs:
            if suf in seen_suf:
                err(f"line {ln}: DUPLICATE item {base}{suf}. (also line {seen_suf[suf]})")
            seen_suf[suf] = ln
        letters = [suf for suf, _ in sufs]
        if letters != sorted(letters):
            err(f"§ item {base}: suffixes out of order — {' '.join(letters)}")

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

# ---- WHAT IS AT THE TARGET, not merely that a target exists -----------------
#
# This check used to answer only "does §N.M exist", and said so at the top of
# the file. That honesty did not stop it costing a defect: four citations to
# `§10.20` for the `inspect`-consults-a-node design resolved CLEAN across three
# commits, because §10.20 exists — it is "legacy inputs are txid-malleable".
# The design lives at §1.1. A gate that is right about its own blind spot is
# still green while the blind spot is being walked into.
#
# So the target's first line is now PRINTED for every distinct reference, which
# is `plan-cite-check.sh`'s pattern: it does not make aboutness machine-checked
# — nothing here can — but it turns it from a LOOKUP into a READ. An author who
# writes §10.20 meaning §1.1 sees "legacy inputs are txid-malleable" in the gate
# output next to their own citation, in the run they were going to do anyway.
def _gist(line, n=68):
    t = re.sub(r'^#{2,3} ', '', line).strip()
    t = re.sub(r'^\d+[a-z]?\.\s*', '', t)
    t = re.sub(r'[*`~_]', '', t)
    return (t[:n] + '…') if len(t) > n else t

targets = []
for ref in sorted(set(re.findall(FOREIGN + r'§(\d+[a-z]?)(?:\.(\d+))?', body))):
    sec, item = ref
    if sec not in sect_ids:
        err(f"cross-reference §{sec} -> no such section")
        continue
    if item:
        ln = seen[sec]
        nxt = min([v for v in seen.values() if v > ln] + [len(lines)+1])
        seg = lines[ln:nxt-1]
        hit = [l for l in seg if re.match(rf'^{item}[a-z]?\. ', l)]
        if not hit:
            err(f"cross-reference §{sec}.{item} -> section {sec} has no item {item}")
        else:
            targets.append((f"§{sec}.{item}", _gist(hit[0])))
    else:
        targets.append((f"§{sec}", _gist(lines[seen[sec]-1])))

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

# ---- 3c. superseded terms must not appear as LIVE text ----------------------
# WHITESPACE-NORMALISED, because the spec is hard-wrapped at ~78 columns and a
# superseded phrase that straddles a line break is INVISIBLE to a line-based
# grep. Four incomplete sweeps in one cycle each reported success for exactly
# that reason. A hit is allowed only inside a blockquote or on a line carrying
# retraction vocabulary — the spec keeps its history, not its mistakes.
import os
terms_path = os.path.join(os.path.dirname(doc), 'SUPERSEDED_TERMS.txt')
if os.path.exists(terms_path):
    terms = [t.strip() for t in open(terms_path)
             if t.strip() and not t.strip().startswith('#')]
    RETRACT = re.compile(r'CORRECTION|RETRACTION|previous draft|earlier draft|'
                         r'earlier version|removed|overrule|are gone|no longer|'
                         r'moot|~~|superseded|was wrong|REVERSED|reversal|'
                         r'used to|had claimed|first version|no longer|rejected', re.I)
    # normalise the whole document, keeping a map back to line numbers
    norm_lines = []
    for k, l in enumerate(lines):
        norm_lines.append((k + 1, l, re.sub(r'\s+', ' ', l).strip()))
    joined = ' '.join(n for _, _, n in norm_lines)
    for t in terms:
        tn = re.sub(r'\s+', ' ', t).strip()
        if tn not in joined:
            continue
        # report EVERY live occurrence, not just the first: an earlier version
        # of this check broke after one hit per term and would have reported a
        # clean sweep with later instances still live.
        seen_at = set()
        for idx, (ln, raw, _) in enumerate(norm_lines):
            if any(abs(idx - p) < 3 for p in seen_at):
                continue
            window = ' '.join(n for _, _, n in norm_lines[idx:idx + 3])
            if tn not in window:
                continue
            # NARROW window and TIGHT vocabulary, both deliberately. An earlier
            # version used +/-8 lines and words like "regenerated" and
            # "candidate" — common enough in ordinary prose that an injected
            # live `base45` at the end of the file landed inside "correction
            # context" and the POSITIVE CONTROL DID NOT FIRE. A gate loosened
            # until it stops failing is a gate that cannot fail. If a legitimate
            # mention trips this, fix the SENTENCE to say it is historical —
            # which a human reader needs anyway — rather than widening the
            # exemption.
            ctx = ' '.join(r for _, r, _ in norm_lines[max(0, idx - 2):idx + 3])
            quoted = raw.lstrip().startswith('>') or RETRACT.search(ctx)
            if not quoted:
                err(f"line {ln}: SUPERSEDED term {t!r} appears as LIVE text")
                seen_at.add(idx)

# ---- 4. duplicated heading text --------------------------------------------
htext = {}
for ln, h in heads:
    t = re.sub(r'^#{2,3} \d+[a-z]?\.\s*', '', h).strip()
    if t and t in htext:
        err(f"line {ln}: heading TEXT duplicated (also line {htext[t]}): {t[:50]!r}")
    htext[t] = ln

if targets:
    print("  ── cross-reference targets (READ THESE: the gate proves they EXIST,")
    print("     never that they say what the citing sentence claims) ──")
    for ref, gist in sorted(set(targets), key=lambda r: [int(x) if x.isdigit() else x
                                                         for x in re.findall(r'\d+|[a-z]+', r[0])]):
        print(f"     {ref:<9} {gist}")
    print()

print(f"  sections: {len(heads)} ; cross-refs checked: {len(set(re.findall(r'§(\d+[a-z]?)(?:\.(\d+))?', body)))}")
print("  STRUCTURE OK" if bad == 0 else f"  {bad} STRUCTURAL DEFECT(S)")
sys.exit(1 if bad else 0)
PY
  [ $? -ne 0 ] && FAIL=1
done
exit $FAIL
