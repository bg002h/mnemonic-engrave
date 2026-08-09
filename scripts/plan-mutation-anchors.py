#!/usr/bin/env python3
"""Check that every mutation-table anchor in a plan matches EXACTLY ONCE.

SPEC §11.3: "a silently-failing `sed` reads exactly like a surviving mutation."
A mutation row that names a token appearing 0 times never runs, and one naming a
token appearing N>1 times mutates the wrong site -- both of which report as a
clean run. The B2b plan shipped nine such rows past two review rounds; `break`
alone matched 6 sites in its own code blocks, one of them the `pl.NextChunk()`
chunk walk, where substituting `return` silently truncates every frame drawn.

This is the check that makes "each row names a unique anchor" a command instead
of a claim.

Usage:  python3 scripts/plan-mutation-anchors.py <plan.md>...

Anchors are read from markdown table rows of the shape

    | `file.go` | `anchor` | replacement | killed by |

taking the SECOND backticked cell as the anchor. Rows whose anchor cell has no
backticked code span (prose-only anchors, e.g. "the three lines X / Y / Z") are
reported as UNCHECKABLE rather than passed: a human-readable anchor is exactly
what a runner cannot apply.

WHAT THIS DOES NOT COVER
  - Anchors targeting FORK files that the plan does not quote in a ```go block
    (e.g. `unlock_session.go`). Their uniqueness holds against the fork tree
    after the task's edits, which does not exist yet. Those are listed
    separately as UNRESOLVED, not passed.
  - Whether the replacement is a real mutation rather than an equivalent
    implementation. A semantically identical substitution survives every run and
    is not a mutant at all; only a reviewer or an actual run catches that.
  - Whether the named test would in fact kill the mutant.
"""
import re
import sys
import pathlib

CODE_SPAN = re.compile(r"`([^`]+)`")


def go_blocks(text):
    return "\n".join(re.findall(r"```go\n(.*?)```", text, re.S))


def rows(text):
    """Yield (lineno, file_cell, anchor_cell) for 4+-column mutation rows."""
    for n, line in enumerate(text.split("\n"), 1):
        s = line.strip()
        if not s.startswith("|") or s.startswith("| ---") or "| --- |" in s:
            continue
        cells = [c.strip() for c in s.strip("|").split("|")]
        if len(cells) < 4:
            continue
        if not CODE_SPAN.search(cells[0]):
            continue  # header row, or not a file cell
        yield n, cells[0], cells[1]


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    bad = unresolved = uncheckable = checked = 0
    for path in argv[1:]:
        text = pathlib.Path(path).read_text()
        blocks = go_blocks(text)
        print(f"== {path} ==")
        print(f"   {len(blocks.splitlines())} lines of ```go across the plan\n")
        for lineno, filecell, anchorcell in rows(text):
            spans = CODE_SPAN.findall(anchorcell)
            fname = CODE_SPAN.findall(filecell)[0]
            if not spans:
                print(f"   UNCHECKABLE  line {lineno}  {fname}: anchor is prose, "
                      f"a runner cannot apply it -> {anchorcell[:60]}")
                uncheckable += 1
                continue
            anchor = max(spans, key=len)
            n = blocks.count(anchor)
            if n == 1:
                checked += 1
                print(f"   ok           line {lineno}  {fname}: {anchor[:58]}")
            elif n == 0:
                unresolved += 1
                print(f"   UNRESOLVED   line {lineno}  {fname}: not in the plan's own "
                      f"blocks (fork-file anchor?) -> {anchor[:48]}")
            else:
                bad += 1
                print(f"   DUPLICATE    line {lineno}  {fname}: matches {n}x -> {anchor[:48]}")
        print()

    print("== RESULT ==")
    print(f"   {checked} unique, {bad} DUPLICATE, {unresolved} unresolved, "
          f"{uncheckable} uncheckable")
    if bad or uncheckable:
        print("   FAIL: a duplicate or prose anchor cannot be applied mechanically.")
        print("   NOT covered: fork-file anchors (listed unresolved), whether a")
        print("   replacement is a real mutation rather than an equivalent, and")
        print("   whether the named test would actually kill it.")
        return 1
    print("   every checkable anchor matches exactly once")
    print("   NOT covered: fork-file anchors (listed unresolved above), whether a")
    print("   replacement is a real mutation rather than an equivalent, and")
    print("   whether the named test would actually kill it.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
