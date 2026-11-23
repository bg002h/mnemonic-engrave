#!/usr/bin/env bash
# Gate for spec S1b (MNEMBLOB v2, design/SPEC_encrypted_payload_delivery.md §14).
# Runs the independent vector generator against the spec and fails if any printed
# vector value (shape row, header hex, derived key, tag, blob sha256, §6.6 hash,
# titles record) differs from the computation.
# NOT covered: the Rust and Go implementations (none exist yet), the spec's prose,
# refusal texts, test-row names, line citations and UF2 bytes; R0 reads those.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
python3 -I "$here/scripts/gen-mnemblob-v2-vectors.py" --check "$here/design/SPEC_encrypted_payload_delivery.md"
echo "spec-s1b-gate: OK (not covered: implementations, prose, texts, row names, citations, UF2)"
