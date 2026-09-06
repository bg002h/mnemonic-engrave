#!/usr/bin/env bash
# h6-plan-blocks-vs-tree.sh -- prove that every code block in
# design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md is byte-for-byte the
# text that was actually built and tested, by diffing each block against the
# GATED SCRATCH TREES the plan's build gate left behind.
#
# WHY IT IS A WRAPPER AND NOT A COPY, and what H6 adds. H5's wrapper simply
# supplied its own defaults to scripts/h2-plan-blocks-vs-tree.sh, whose parser
# keys on the ```<lang> file=<path> mode=whole|fragment convention rather than on
# anything H2. H6 spans THREE repositories -- mnemonic-secret (the Rust primary
# for the phrase rule and the QR text), mnemonic-engrave (the `me` host half) and
# the seedhammer fork (the device half) -- so one tree root is not enough.
#
# THE ONE CONVENTION CHANGE, and it needs no engine change: an H6 block's path is
# ROOTED AT A REPO NAME.
#
#     ```go   file=fork/engrave/engrave.go            mode=fragment
#     ```rust file=ms/crates/ms-codec/src/hashlock.rs mode=fragment
#     ```rust file=me/crates/me-cli/src/sysw/mod.rs   mode=fragment
#
# This script builds a symlink farm whose `fork`, `ms` and `me` entries point at
# the three gated trees, and hands THAT to the engine as a single root. The
# parser, the diffing and the blind-spot report are unchanged and stay in one
# place.
#
# WHAT THE ENGINE CHECKS AND WHAT IT DOES NOT is printed by every run -- read the
# tail of the output, not this header. In summary: whole blocks are diffed
# against <tree>/<path>, fragments must appear as an exact byte substring, and
# NOTHING here runs a command, checks a prose claim, or says whether any tree is
# green. H6 adds one blind spot of its own, named here because a gate that hides
# it is worse than no gate: **the fork tree's gui/ work for Tasks 8b-12 was
# SPECIFIED and not wired** by the plan author (see the plan's `## Build gate`
# and the author report), so those tasks' blocks deliberately carry NO file=
# header and this script does not check them. Every block it does check is one
# that was built and run.
#
# USAGE
#   scripts/h6-plan-blocks-vs-tree.sh [plan.md] [fork-tree] [ms-tree] [me-tree]
# Defaults: design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md
#           /scratch/code/shibboleth/.tmp/h6-gate
#           /scratch/code/shibboleth/.tmp/h6-ms
#           /scratch/code/shibboleth/.tmp/h6-me

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PLAN="${1:-$HERE/design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md}"
FORK="${2:-/scratch/code/shibboleth/.tmp/h6-gate}"
MS="${3:-/scratch/code/shibboleth/.tmp/h6-ms}"
ME="${4:-/scratch/code/shibboleth/.tmp/h6-me}"
ENGINE="$HERE/scripts/h2-plan-blocks-vs-tree.sh"

[ -x "$ENGINE" ] || { echo "no such checker engine: $ENGINE" >&2; exit 2; }
for d in "$FORK" "$MS" "$ME"; do
	[ -d "$d" ] || { echo "no such tree: $d" >&2; exit 2; }
done

FARM="$(mktemp -d "${TMPDIR:-/tmp}/h6-blocks-XXXXXX")"
trap 'rm -rf "$FARM"' EXIT
ln -s "$(cd "$FORK" && pwd)" "$FARM/fork"
ln -s "$(cd "$MS" && pwd)" "$FARM/ms"
ln -s "$(cd "$ME" && pwd)" "$FARM/me"

echo "fork: $FORK"
echo "ms:   $MS"
echo "me:   $ME"
echo

exec "$ENGINE" "$PLAN" "$FARM"
