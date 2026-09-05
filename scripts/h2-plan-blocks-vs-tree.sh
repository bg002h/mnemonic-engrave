#!/usr/bin/env bash
# h2-plan-blocks-vs-tree.sh -- prove that every code block in
# design/IMPLEMENTATION_PLAN_hashlock_H2_device.md is byte-for-byte the text that
# was actually built and tested, by diffing each block against the GATED SCRATCH
# TREE the build gate left behind.
#
# WHY THIS EXISTS. The build gate (design/agent-reports/hashlock-H2-plan-build-gate.md)
# hand-wired the plan into /scratch/code/shibboleth/.tmp/h2-gate and needed 12 fixes
# to reach green. A fold that transcribes those fixes back into the plan is
# authorship, and a transcription defect there is invisible: the plan still LOOKS
# green, and the implementer following it writes code nobody compiled. This script
# makes "the plan describes the gated tree" a command instead of a promise.
#
# THE CONVENTION IT PARSES. A fenced block that carries file content opens with
#
#     ```<lang> file=<path> mode=whole
#     ```<lang> file=<path> mode=fragment
#
# where <path> is relative to the fork root. Markdown renderers use only the first
# word of an info string as the language, so highlighting is unaffected.
#
#   mode=whole     the block IS the file: diffed against <tree>/<path>.
#   mode=fragment  the block is an excerpt: must appear VERBATIM (exact byte
#                  substring, indentation included) somewhere in <tree>/<path>.
#
# WHAT IT DOES NOT COVER -- printed at the end of every run, because a gate that
# hides its blind spot is worse than no gate:
#   * blocks with NO file= header: ```bash command blocks (the commit/vendor/size
#     recipes) and any illustrative snippet. Nothing checks that those commands run.
#   * every PROSE claim in the plan: expected test names, mutation outcomes, headroom
#     numbers, firmware sizes, spec section references, file:line citations. Those
#     are the reviewer's and the report's job.
#   * whether the tree itself is green. This script compares TEXT. `go test` is what
#     says the text works, and the gate report is where that result lives.
#   * files the plan changes but carries no block for (e.g. a rename applied "consistently").
#
# USAGE
#   scripts/h2-plan-blocks-vs-tree.sh [plan.md] [tree]
# Defaults: design/IMPLEMENTATION_PLAN_hashlock_H2_device.md and
#           /scratch/code/shibboleth/.tmp/h2-gate
# Exits non-zero if any block FAILs or any referenced tree file is missing.

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PLAN="${1:-$HERE/design/IMPLEMENTATION_PLAN_hashlock_H2_device.md}"
TREE="${2:-/scratch/code/shibboleth/.tmp/h2-gate}"

[ -f "$PLAN" ] || { echo "no such plan: $PLAN" >&2; exit 2; }
[ -d "$TREE" ] || { echo "no such tree: $TREE" >&2; exit 2; }

echo "plan: $PLAN"
echo "tree: $TREE"
echo

PLAN="$PLAN" TREE="$TREE" python3 - <<'PY'
import os, re, sys, difflib

plan = os.environ["PLAN"]
tree = os.environ["TREE"]

lines = open(plan, encoding="utf-8").read().split("\n")
open_re = re.compile(r"^```(\w+)\s+file=(\S+)\s+mode=(whole|fragment)\s*$")

blocks, i, unheadered = [], 0, []
while i < len(lines):
    m = open_re.match(lines[i])
    if m:
        lang, path, mode = m.group(1), m.group(2), m.group(3)
        start = i + 1
        j = start
        while j < len(lines) and not lines[j].startswith("```"):
            j += 1
        blocks.append((i + 1, lang, path, mode, "\n".join(lines[start:j]) + "\n"))
        i = j + 1
        continue
    if lines[i].startswith("```"):
        # an opening fence with no file= header -- a bash recipe, a captured
        # command output, or an illustration. Recorded so the blind spot is named.
        unheadered.append((i + 1, lines[i] if len(lines[i]) > 3 else "``` (no info string)"))
        j = i + 1
        while j < len(lines) and not lines[j].startswith("```"):
            j += 1
        i = j + 1
        continue
    i += 1

fails = 0
cache = {}


def read(path):
    if path not in cache:
        full = os.path.join(tree, path)
        if not os.path.exists(full):
            cache[path] = None
        else:
            cache[path] = open(full, encoding="utf-8").read()
    return cache[path]


for lineno, lang, path, mode, body in blocks:
    tag = "%s:%d  %-14s %-44s" % (os.path.basename(plan), lineno, mode, path)
    content = read(path)
    if content is None:
        print("FAIL %s  -- no such file in the tree" % tag)
        fails += 1
        continue
    if mode == "whole":
        if body == content:
            print("PASS %s  (%d lines, identical)" % (tag, body.count("\n")))
        else:
            print("FAIL %s  -- differs from the tree file:" % tag)
            d = list(difflib.unified_diff(body.split("\n"), content.split("\n"),
                                          "plan block", "tree file", lineterm="", n=1))
            for ln in d[:40]:
                print("       " + ln)
            if len(d) > 40:
                print("       ... %d more diff lines" % (len(d) - 40))
            fails += 1
    else:
        needle = body.rstrip("\n")
        if needle in content:
            print("PASS %s  (%d lines, verbatim substring)" % (tag, body.count("\n")))
        else:
            print("FAIL %s  -- not a verbatim substring of the tree file." % tag)
            # Name the first block line that is absent from the file, so the
            # failure points at a line rather than at the whole block.
            haystack = set(content.split("\n"))
            missing = [ln for ln in needle.split("\n") if ln not in haystack]
            if missing:
                print("       first line absent from the file: %r" % missing[0])
                print("       (%d of %d block lines absent)" % (len(missing), needle.count("\n") + 1))
            else:
                print("       every line exists, but not contiguously/in this order")
            fails += 1

print()
print("%d blocks checked, %d FAIL" % (len(blocks), fails))
print()
print("NOT COVERED by this script:")
print("  * %d fenced blocks carry no file= header (bash recipes, illustrative" % len(unheadered))
print("    snippets); nothing here runs or checks them:")
for lineno, fence in unheadered:
    print("      %s:%d  %s" % (os.path.basename(plan), lineno, fence))
print("  * every PROSE claim: expected test names, mutation outcomes, headroom and")
print("    firmware numbers, spec references, file:line citations.")
print("  * whether the tree is GREEN -- this compares TEXT only; `go test` and the")
print("    gate report are what say the text works.")
print("  * files the plan modifies without carrying a block for them.")

sys.exit(1 if fails else 0)
PY
