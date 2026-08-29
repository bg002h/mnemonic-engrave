#!/usr/bin/env bash
# lint-gate.sh — the S2 plan's lint gate as ONE command (P0.3; F-430's durable fix).
#
# F-430's lesson: a lint rule that CI enforces but no local command reproduces
# gets broken by the obvious command. This script IS the local command: clippy
# on the CI-pinned toolchain AND nightly, plus rustfmt, all --locked. Every
# fold and every phase gate in IMPLEMENTATION_PLAN_descriptor_input_S2.md that
# says "lint-gate" means exactly this.
#
# NOT covered (stated because a gate that hides its blind spot is worse than no
# gate): tests (nextest is the suite gate, run separately), the Go/fork side
# (go vet / gofmt / TinyGo build live in the fork's own gates), and the plan
# cite/staleness checks (scripts/plan-cite-check.sh, plan-staleness-check.sh).
set -euo pipefail
cd "$(dirname "$0")/.."
echo "== cargo fmt --check"
cargo fmt --check
echo "== clippy (CI-pinned 1.85.0)"
cargo +1.85.0 clippy --locked --workspace --all-targets -- -D warnings
echo "== clippy (nightly)"
cargo +nightly clippy --locked --workspace --all-targets -- -D warnings
echo "lint-gate: PASS"
