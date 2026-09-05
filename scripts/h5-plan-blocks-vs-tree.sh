#!/usr/bin/env bash
# h5-plan-blocks-vs-tree.sh -- prove that every code block in
# design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md is byte-for-byte the
# text that was actually built and tested, by diffing each block against the
# GATED SCRATCH TREE the plan's build gate left behind.
#
# WHY IT IS A WRAPPER AND NOT A COPY. scripts/h2-plan-blocks-vs-tree.sh already
# takes (plan, tree) as arguments and its parser is generic: it keys on the
# ```<lang> file=<path> mode=whole|fragment convention, not on anything H2. A
# second copy of 153 lines would be a second thing to fix when the convention
# moves, and this repo has been bitten by hand-maintained duplicates before.
# So this file is the H5 DEFAULTS plus this comment, and the engine stays in
# one place.
#
# WHAT THE ENGINE CHECKS AND WHAT IT DOES NOT is printed by every run -- read
# the tail of the output, not this header, for the blind spots. In summary:
# whole blocks are diffed against <tree>/<path>, fragments must appear as an
# exact byte substring, and NOTHING here runs a command, checks a prose claim,
# or says whether the tree is green.
#
# USAGE
#   scripts/h5-plan-blocks-vs-tree.sh [plan.md] [tree]
# Defaults: design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md and
#           /scratch/code/shibboleth/.tmp/h5-gate

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PLAN="${1:-$HERE/design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md}"
TREE="${2:-/scratch/code/shibboleth/.tmp/h5-gate}"
ENGINE="$HERE/scripts/h2-plan-blocks-vs-tree.sh"

[ -x "$ENGINE" ] || { echo "no such checker engine: $ENGINE" >&2; exit 2; }

exec "$ENGINE" "$PLAN" "$TREE"
