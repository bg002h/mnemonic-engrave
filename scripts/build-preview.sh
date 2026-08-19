#!/usr/bin/env bash
# build-preview.sh — build the `me-preview` Go sidecar at the EXACT crate
# version, into `target/release/` beside `me`.
#
# WHY THIS EXISTS. `me` refuses a sidecar whose `--version` is not a byte-exact
# match for its own crate version (`crates/me-cli/src/main.rs`, the lockstep
# pin from SPEC_me_bundle_phaseB_preview.md §4.1), and `me bundle --preview`
# exits 2 when it mismatches. That version is injected at LINK time via
# `-ldflags -X main.version=` — `preview/main.go` declares a bare
# `var version string` with no default, so a plain `go build` produces a
# sidecar that prints `me-preview ` and can never satisfy the gate.
#
# Until now that link flag existed only inside `.github/workflows/release.yml`.
# So every `me` version bump silently broke `me bundle --preview` for anyone
# working from a checkout — including the journey transcripts, which invoke
# `target/release/me-preview --version` directly — and the fix was a command
# nobody could find. That is the reproduction-path decay the journeys README
# warns about, one layer down: a build step whose only copy lives in CI is a
# build step a developer cannot run.
#
# Usage:  bash scripts/build-preview.sh          # build at the crate version
#         bash scripts/build-preview.sh --check  # verify the built one matches
#
# Set GO= to pick a toolchain when `go` is not on $PATH, e.g.
#   GO=/nix/store/…-go-1.25.10/bin/go bash scripts/build-preview.sh
# 1.25.10 is what release CI pins.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/target/release/me-preview"

# The crate version is the single source of truth; parse it from the manifest
# rather than `cargo metadata` so this runs without a toolchain warm-up.
VERSION="$(sed -n '0,/^version = /s/^version = "\(.*\)"/\1/p' "$ROOT/crates/me-cli/Cargo.toml")"
[ -n "$VERSION" ] || { echo "FATAL: could not read version from crates/me-cli/Cargo.toml" >&2; exit 1; }

if [ "${1:-}" = "--check" ]; then
  [ -x "$OUT" ] || { echo "MISSING: $OUT — run without --check"; exit 1; }
  got="$("$OUT" --version 2>&1 || true)"
  if [ "$got" = "me-preview $VERSION" ]; then
    echo "ok: $got matches the crate"
    exit 0
  fi
  echo "DRIFT: sidecar reports '$got', crate is $VERSION — run without --check"
  exit 1
fi

GO="${GO:-go}"
command -v "$GO" >/dev/null || { echo "FATAL: go not found (set GO=/path/to/go)" >&2; exit 1; }

cd "$ROOT/preview"
CGO_ENABLED=0 "$GO" build -trimpath \
  -ldflags="-s -w -X main.version=$VERSION" \
  -o "$OUT" .

# Verify the stamp actually landed. `-X` against a missing symbol is silently
# ignored by the Go linker, so a successful build proves nothing on its own.
got="$("$OUT" --version 2>&1)"
[ "$got" = "me-preview $VERSION" ] || {
  echo "FATAL: built sidecar reports '$got', expected 'me-preview $VERSION'" >&2
  exit 1
}
echo "built $OUT ($got)"
