#!/usr/bin/env python3
"""Check that every mutation-table anchor in a plan matches EXACTLY ONCE,
in the file the row names.

SPEC §11.3: "a silently-failing `sed` reads exactly like a surviving mutation."
A mutation row naming a token that occurs 0 times never runs; one naming a token
that occurs N>1 times mutates the wrong site. Both report as a clean run, and a
clean run is what gates the phase.

Usage:  python3 scripts/plan-mutation-anchors.py <plan.md>...

Rows are markdown table rows of the shape

    | `file.go` | `anchor` | replacement | killed by |

TWO RULES, both learned from this script's own first version, which passed a
duplicate anchor and so had the exact false-PASS it exists to prevent:

  1. The anchor is resolved against the ```go block for the file named in the
     FIRST cell -- not against every block concatenated. Version 1 searched all
     fences at once and so never noticed a row whose file cell named the wrong
     file.
  2. The anchor cell must contain EXACTLY ONE code span. Version 1 took the
     longest span, which silently graded a parenthetical *context* note instead
     of the anchor, reporting `ok` for an anchor that matched twice. Context
     belongs in the "killed by" column; the anchor cell is machine input.

Blocks are associated with files by the same anchor convention
plan-build-gate-go.sh uses: a line matching `create|new file|add to|insert
into|in|modify` + a backticked *.go path binds the following fences until the
next markdown heading.

WHAT THIS DOES NOT COVER
  - Anchors targeting FORK files the plan does not quote in a ```go block.
    Their uniqueness holds against the fork tree after the task's edits, which
    does not exist yet. Reported UNRESOLVED, never passed.
  - Whether the replacement is a real mutation rather than an equivalent
    implementation. A semantically identical substitution survives every run and
    is not a mutant at all.
  - Whether the named test would in fact kill the mutant.
  - Indentation-sensitive intent: an anchor is compared literally, so a row
    meaning "the indented one" must make the line itself unique.
"""
import re
import sys
import pathlib
import collections

CODE_SPAN = re.compile(r"`([^`]+)`")
ANCHOR_VF = re.compile(r"(create|new file|add to|insert into|in|modify)\s+`([^`]*\.go)`", re.I)
ANCHOR_PF = re.compile(r"`([^`]*\.go)`[\s,—–-]*\(?(new file|new)\b", re.I)
FENCE = re.compile(r"^```go\s*$")


def blocks_by_file(text):
    """path -> concatenated ```go source, using plan-build-gate-go.sh's anchors."""
    out = collections.defaultdict(list)
    whole = set()   # paths introduced as WHOLE files (create/new file)
    cur = None
    lines = text.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.startswith("#"):
            cur = None
        m = ANCHOR_VF.search(line)
        if m:
            cur = m.group(2)
            if m.group(1).lower() in ("create", "new file"):
                whole.add(cur)
        else:
            m = ANCHOR_PF.search(line)
            if m:
                cur = m.group(1)
                whole.add(cur)
        if FENCE.match(line):
            j = i + 1
            body = []
            while j < len(lines) and not lines[j].startswith("```"):
                body.append(lines[j])
                j += 1
            if cur:
                out[cur].append("\n".join(body))
            i = j
        i += 1
    return ({k: "\n".join(v) for k, v in out.items()}, whole)


def rows(text):
    for n, line in enumerate(text.split("\n"), 1):
        s = line.strip()
        if not s.startswith("|") or "| --- |" in s or s.startswith("| ---"):
            continue
        cells = [c.strip() for c in s.strip("|").split("|")]
        if len(cells) < 4:
            continue
        f = CODE_SPAN.findall(cells[0])
        if not f:
            continue
        yield n, f[0], cells[1]


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    ok = bad = unresolved = 0
    for path in argv[1:]:
        text = pathlib.Path(path).read_text()
        byfile, whole = blocks_by_file(text)
        print(f"== {path} ==")
        for p, src in sorted(byfile.items()):
            print(f"   block {p}: {len(src.splitlines())} lines")
        print()
        for lineno, fname, anchorcell in rows(text):
            spans = CODE_SPAN.findall(anchorcell)
            if len(spans) != 1:
                bad += 1
                print(f"   BAD ANCHOR   line {lineno}  {fname}: anchor cell has "
                      f"{len(spans)} code spans, must have exactly 1. Move context to "
                      f"the 'killed by' column -> {anchorcell[:56]}")
                continue
            anchor = spans[0]
            # match the block whose path ENDS with the named file
            src = None
            is_whole = False
            for p, body in byfile.items():
                if p.endswith(fname) or fname.endswith(p):
                    src, is_whole = body, p in whole
                    break
            if src is None:
                unresolved += 1
                print(f"   UNRESOLVED   line {lineno}  {fname}: no ```go block for this "
                      f"file (fork-file anchor?) -> {anchor[:46]}")
                continue
            n = src.count(anchor)
            if n == 1:
                ok += 1
                print(f"   ok           line {lineno}  {fname}: {anchor[:56]}")
            elif n == 0 and not is_whole:
                # The plan quotes this file only as a FRAGMENT, which is a partial
                # view -- absence from it proves nothing about the real file.
                unresolved += 1
                print(f"   UNRESOLVED   line {lineno}  {fname}: the plan quotes this file "
                      f"only as a fragment, so absence proves nothing -> {anchor[:40]}")
            elif n == 0:
                bad += 1
                print(f"   NOT IN FILE  line {lineno}  {fname}: 0 matches in the WHOLE file "
                      f"the row names -> {anchor[:46]}")
            else:
                bad += 1
                print(f"   DUPLICATE    line {lineno}  {fname}: {n} matches -> {anchor[:46]}")
        print()

    print("== RESULT ==")
    print(f"   {ok} unique, {bad} BAD, {unresolved} unresolved")
    tail = ("   NOT covered: fork-file anchors (unresolved above); whether a\n"
            "   replacement is a real mutation rather than an equivalent; whether the\n"
            "   named test would actually kill it; and indentation-sensitive intent --\n"
            "   anchors are compared literally.")
    if bad:
        print("   FAIL: an anchor that is duplicated, absent, or ambiguous cannot be")
        print("   applied mechanically, and reads as a clean run when it silently fails.")
        print(tail)
        return 1
    print("   every checkable anchor matches exactly once, in the file its row names")
    print(tail)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
