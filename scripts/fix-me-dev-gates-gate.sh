#!/usr/bin/env bash
# fix-me-dev-gates-gate.sh -- the row gate for FIX_me_dev_gates (bughunt-1119
# bh-me-I12, I13, I14, I19). Run from anywhere in the worktree; needs python3,
# go and the pinned cargo on PATH. Each row prints `ok` or `FAIL`; the script
# exits 1 on any FAIL. On the base (before the fix) the test rows FAIL: that
# is the RED run.
#
# NOT COVERED here (the builder runs them separately): the rest of CI (cargo
# fmt, both clippy runs, the full nextest suite, doc-tests, go test), shellcheck
# of the changed shell scripts, and the two-transaction regtest byte comparison
# of the generator (needs a regtest node; the generation code path is unchanged).
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT" || exit 1
LOG="${GATE_LOG_DIR:-$ROOT/target/fix-me-dev-gates}"
mkdir -p "$LOG"
fails=0
row() { if [ "$2" = 0 ]; then echo "ok   $1"; else echo "FAIL $1"; fails=$((fails + 1)); fi; }

# G1 (I12, I13): the generator's Python tests, 7 of them.
python3 -I scripts/test/gen_mt1_vectors_test.py > "$LOG/py.txt" 2>&1
grep -q '^Ran 7 tests' "$LOG/py.txt" && tail -1 "$LOG/py.txt" | grep -qx 'OK'
row "G1 gen_mt1_vectors_test.py: 7 tests OK" $?

# G2 (I12): the default corpus lives in this repo and is the pinned copy.
! grep -n 'mnemonic-key/' scripts/gen-mt1-vectors.py > "$LOG/g2.txt"
row "G2 gen-mt1-vectors.py names no path in a sibling checkout" $?
echo "7591561f61863f91a65f00f8b004d958ee03eb4a2272f593fb48c54c2e5c6caa  scripts/test/fixtures/mk-codec-v0.1-vectors.json" \
  | sha256sum -c --quiet > /dev/null 2>&1
row "G3 vendored mk corpus is mk-codec v0.1.json at mnemonic-key 6faf8a60" $?

# G4 (I13): the printed usage line names --tx.
grep -qF 'gen-mt1-vectors.py --from-regtest <datadir> --rpcport <port> --tx <label>=<hexfile>:<note> [--tx ...] --out design/vectors' \
  scripts/gen-mt1-vectors.py
row "G4 usage line names --tx" $?

# G5 (I14): the demo build republishes after an embedded-only change.
TMPDIR="$LOG" bash scripts/test/demo-sh2-embed-stale.test.sh > "$LOG/sh.txt" 2>&1
grep -qx 'PASS: second dist wasm contains fixture-bravo' "$LOG/sh.txt"
row "G5 demo-sh2-embed-stale.test.sh PASS" $?

# G6 (I19): the Rust probe tests, 11 of them.
cargo nextest run --locked --build-jobs 2 -j 2 -p mnemonic-engrave --test preview_version_probe \
  --no-fail-fast > "$LOG/rs.txt" 2>&1
grep -q 'Summary .* 11 tests run: 11 passed' "$LOG/rs.txt"
row "G6 preview_version_probe: 11 passed" $?

# G7: the new tests and the CI step name no docs file.
bash ci/code-docs-check.sh > "$LOG/cdc.txt" 2>&1
row "G7 ci/code-docs-check.sh ok" $?

# G8: CI runs the two script tests.
grep -qF 'python3 -I scripts/test/gen_mt1_vectors_test.py' .github/workflows/release.yml \
  && grep -qF 'bash scripts/test/demo-sh2-embed-stale.test.sh' .github/workflows/release.yml
row "G8 release.yml test job runs both script tests" $?

echo "gate: $fails FAIL (logs in $LOG)"
[ "$fails" = 0 ]
