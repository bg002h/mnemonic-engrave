#!/usr/bin/env bash
# fork-vet-gate.sh -- run `go vet` over the seedhammer fork and fail on anything
# except the ONE known, explained gap.
#
# WHY THIS EXISTS. `go vet ./...` on the fork has been red since before the
# hashlock work (F-494), and because vet is not in the fork's CI gate nothing
# looked at it -- so a genuinely new vet finding would have landed in a stream of
# noise nobody reads. Keying the `bezier.Point` literals removed 33 of the 43
# warnings; this script is what keeps the other 10 from hiding the next one.
#
# THE KNOWN GAP, and why it is not simply fixed. Ten test files call
# `testing.ArtifactDir`, a Go 1.26 API, while `go.mod` declares `go 1.25.10`, so
# vet reports "requires go1.26 or later". The obvious remedy -- raise the
# directive -- BREAKS THE FIRMWARE, measured 2026-09-06 on a probe worktree:
#
#   $ sed -i 's/^go 1.25.10$/go 1.26.0/' go.mod
#   $ nix develop -c tinygo build ... ./cmd/controller
#   cannot compile with Go toolchain version go1.26
#   (TinyGo was built using toolchain version go1.25.10)
#
# TinyGo 0.41.1 in this flake is built against go1.25.10, and the directive is
# what it checks. So the gap closes when the pinned TinyGo moves, not before,
# and until then the tests run only because the HOST toolchain is 1.26 -- legal
# by luck rather than by declaration, which is worth knowing and not worth
# breaking the device build to fix.
#
# Usage: fork-vet-gate.sh [fork-worktree]   (default /scratch/code/shibboleth/seedhammer)
set -euo pipefail

FORK="${1:-/scratch/code/shibboleth/seedhammer}"
KNOWN='requires go1\.26 or later'

cd "$FORK"
out="$(go vet ./... 2>&1 || true)"

# Package headers (`# seedhammer.com/gui`) are not findings; drop them, then
# split what is left into the known class and everything else.
findings="$(printf '%s\n' "$out" | grep -vE '^#' | grep -vE '^\s*$' || true)"
known="$(printf '%s\n' "$findings" | grep -cE "$KNOWN" || true)"
other="$(printf '%s\n' "$findings" | grep -vE "$KNOWN" || true)"

printf 'fork: %s\n' "$FORK"
printf 'known gap (testing.ArtifactDir vs the go directive TinyGo pins): %s\n' "$known"

if [ -n "$other" ]; then
  printf '\nUNEXPECTED vet findings -- this is what the gate is for:\n%s\n' "$other"
  exit 1
fi

printf 'no other vet findings\n'
