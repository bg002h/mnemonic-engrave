#!/usr/bin/env bash
# fix-sec-canonical-keeps-xpub-depth-gate.sh — the build gate for
# design/FIX_sec_canonical_keeps_xpub_depth.md (security sweep unit
# canonical-keeps-xpub-depth; FOLLOWUPS sh2-me-canonical-depth).
#
#   bash scripts/fix-sec-canonical-keeps-xpub-depth-gate.sh --static  # A only (seconds)
#   bash scripts/fix-sec-canonical-keeps-xpub-depth-gate.sh           # A to D
#   FORK_WT=<seedhammer worktree> bash scripts/...-gate.sh            # A to E
#
# What it runs:
#   A. static: the seam file's sha256 and both me pins; the 16-key row's canonical
#      is its input plus `#thg3nw30`; rows.py no longer carries `canonical_from`
#      on that row nor `origin_serialised`; the spec, gate.rs and CHANGELOG carry
#      the FIX's new sentences and no longer carry the old ones.
#   B. CLI row: builds `me` (--locked) and packs the depth-4 promotion fixture;
#      exit 0, stderr carries the FIX §4 lines whole, and `me sysw show` prints the
#      record the device measured.
#   C. the full nextest suite ONCE to a log (Go on PATH, ME_REQUIRE_GO=1, as CI);
#      each named FIX test must PASS exactly once, and no FAIL line may appear.
#   D. doc-tests, cargo fmt --check, the code-docs check, and both clippy runs of the CI `test` job on
#      the CI-pinned toolchain (1.85.0, release.yml RUST_TOOLCHAIN).
#   E. (only with FORK_WT) the fork's vendored seam file is byte-identical to me's,
#      its pin equals me's, and `go test ./nonstandard -run DescriptorSeam` passes.
# NOT covered: re-running scripts/descriptor-seam-vectors/gen.py (the builder runs
# it; this gate checks its output's sha and the one changed value); the fork's CI
# (E is a local run of one package); CI jobs this change does not touch (me-preview
# Go tests, OTP e2e, shellcheck, cross builds); wording quality (the new text is
# PROPOSED in the FIX doc); the mutants named in FIX §5 (run by hand at the RED).
set -uo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
LOG="${GATE_LOG_DIR:-target}/fix-sec-canonical-keeps-xpub-depth-gate"
mkdir -p "$LOG"
fail=0
bad() { echo "FAIL $*"; fail=1; }
has() { grep -qF -- "$2" "$1" && echo "ok   $1 has: ${2:0:60}" || bad "$1 lacks: $2"; }
lacks() { grep -qF -- "$2" "$1" && bad "$1 still has: $2" || echo "ok   $1 lacks: ${2:0:60}"; }
# Whitespace-collapsed variants for hard-wrapped prose (the spec).
inw() { python3 -I -c 'import sys; c=lambda t: " ".join(t.split()); sys.exit(0 if c(sys.argv[2]) in c(open(sys.argv[1]).read()) else 1)' "$1" "$2"; }
hasw() { inw "$1" "$2" && echo "ok   $1 has: ${2:0:60}" || bad "$1 lacks: $2"; }
lacksw() { inw "$1" "$2" && bad "$1 still has: $2" || echo "ok   $1 lacks: ${2:0:60}"; }

# Re-pinned by the constellation exit-code change (the file's exit_code values moved;
# design/FIX_sec_canonical_keeps_xpub_depth.md records the earlier 71bba6f2 pin).
SHA=ec4b6e216bb9e064319ec5c66fcdac47f64055ce4089e2058ed211f911bb5ade
J=crates/me-cli/testdata/descriptor_seam_vectors.json
SPEC=design/SPEC_descriptor_input.md

echo "== A. static"
got=$(sha256sum "$J" | cut -d' ' -f1)
[ "$got" = "$SHA" ] && echo "ok   $J sha256" || bad "A: $J sha256 $got, want $SHA"
has crates/me-cli/tests/descriptor_seam.rs "\"$SHA\""
python3 -I - "$J" <<'PY' || bad "A: 16-key canonical"
import json, sys
v = {r["name"]: r for r in json.load(open(sys.argv[1]))["vectors"]}
r = v["accepted/sh-wsh-sortedmulti-16-keys"]
sys.exit(0 if r["canonical"] == r["input"] + "#thg3nw30" else 1)
PY
lacks scripts/descriptor-seam-vectors/rows.py "origin_serialised"
lacks scripts/descriptor-seam-vectors/rows.py "sh2-me-canonical-depth"
python3 -I - scripts/descriptor-seam-vectors/rows.py <<'PY' || bad "A: rows.py 16-key row still has canonical_from"
import sys
src = open(sys.argv[1]).read()
i = src.index('R(name="accepted/sh-wsh-sortedmulti-16-keys"')
j = src.index("\nR(", i + 1)
sys.exit(1 if "canonical_from" in src[i:j] else 0)
PY
lacks crates/me-cli/src/descriptor/gate.rs "depth bytes rebuilt from"
has crates/me-cli/src/descriptor/gate.rs "The key's version bytes may be normalised"
lacksw "$SPEC" "The canonical re-encoding rebuilds an xpub's"
lacksw "$SPEC" "depth/child-number bytes are rebuilt from the origin path"
hasw "$SPEC" "The canonical re-encoding keeps the key's own depth and child-number bytes"
hasw "$SPEC" "measured three ways (R0's C5):"
hasw crates/me-cli/CHANGELOG.md "keeps an extended key's own depth and child number"

[ "${1:-}" = "--static" ] && { [ $fail = 0 ] && echo "gate --static: PASS" || echo "gate --static: FAIL"; exit $fail; }

echo "== B. CLI row"
cargo build --locked -p mnemonic-engrave --bin me >"$LOG/build.log" 2>&1 || { bad "build (see $LOG/build.log)"; exit 1; }
ME="${CARGO_TARGET_DIR:-target}/debug/me"
D4=xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan
WANT="pkh([4bbaa801/44h/0h/0h]$D4)#57c9a380"
printf '[4bbaa801/44h/0h/0h]%s\n' "$D4" >"$LOG/deep.txt"
rm -f "$LOG/deep.bin"
"$ME" sysw pack --no-passphrase --as descriptor --in "$LOG/deep.txt" --out "$LOG/deep.bin" >"$LOG/deep.out" 2>"$LOG/deep.err"
rc=$?
[ "$rc" = 0 ] && echo "ok   pack exit 0" || bad "B: pack exit $rc"
for line in \
    "      descriptor: $WANT" \
    "      key as supplied: [4bbaa801/44h/0h/0h]$D4" \
    "      inferred wallet: $WANT" \
    "      The key's version bytes may be normalised (\`zpub\` becomes \`xpub\`); its depth, child number and key material are unchanged."; do
    grep -qxF -- "$line" "$LOG/deep.err" && echo "ok   stderr: ${line:6:50}" || bad "B: stderr lacks the line: $line"
done
"$ME" sysw show "$LOG/deep.bin" >"$LOG/show.out" 2>&1 || bad "B: show exit $?"
grep -qxF -- "      descriptor: $WANT" "$LOG/show.out" && echo "ok   show carries the device's canonical" || bad "B: show lacks descriptor: $WANT"

echo "== C. nextest (whole workspace, once)"
[ -f third_party/seedhammer/go.mod ] || { bad "C: run 'git submodule update --init third_party/seedhammer' first"; exit 1; }
GO=/scratch/code/shibboleth/.toolchain/go/bin
[ -d "$GO" ] && export PATH="$GO:$PATH"
ME_REQUIRE_GO=1 cargo nextest run --locked --no-fail-fast --build-jobs 4 -j 4 >"$LOG/nextest.log" 2>&1
echo "nextest exit $?; summary: $(grep -E '^ *Summary' "$LOG/nextest.log" | tail -1)"
grep -E '^ *(FAIL|SIGSEGV|TIMEOUT|LEAK)' "$LOG/nextest.log" | sed 's/^/    /' && bad "C: failing tests (above)"
for t in \
    canonical_keeps_a_promoted_keys_own_depth_and_child \
    canonical_keeps_a_coldcard_keys_own_depth_and_child \
    canonical_keeps_the_keys_own_child_number_when_the_origin_ends_elsewhere \
    canonical_keeps_the_keys_own_depth_with_no_origin_block \
    promotion_announces_both_the_supplied_key_and_the_inferred_wallet \
    promotion_of_a_depth_4_key_packs_and_announces_the_key_as_supplied \
    the_encoder_produces_every_canonical_the_file_carries; do
    c=$(grep -cE "^ *PASS .*[ :]$t\$" "$LOG/nextest.log")
    [ "$c" = 1 ] && echo "ok   $t" || bad "C: $t passed $c times, want 1"
done

echo "== D. doc-tests and lints"
cargo test --locked --doc >"$LOG/doctest.log" 2>&1 || bad "D: doc-tests (see $LOG/doctest.log)"
cargo fmt --check >"$LOG/fmt.log" 2>&1 || bad "D: cargo fmt --check (see $LOG/fmt.log)"
bash ci/code-docs-check.sh >"$LOG/code-docs.log" 2>&1 || bad "D: code-docs check (see $LOG/code-docs.log)"
cargo +1.85.0 clippy --all-targets --locked -- -D warnings >"$LOG/clippy.log" 2>&1 || bad "D: clippy (see $LOG/clippy.log)"
cargo +1.85.0 clippy -p mnemonic-engrave --lib --no-default-features --locked -- -D warnings >"$LOG/clippy-nodef.log" 2>&1 \
    || bad "D: no-default-features clippy (see $LOG/clippy-nodef.log)"

if [ -n "${FORK_WT:-}" ]; then
    echo "== E. fork seam ($FORK_WT)"
    cmp -s "$J" "$FORK_WT/nonstandard/testdata/descriptor_seam_vectors.json" \
        && echo "ok   vendored file identical" || bad "E: fork's vendored seam file differs from me's"
    has "$FORK_WT/nonstandard/descriptor_seam_test.go" "\"$SHA\""
    (cd "$FORK_WT" && go test ./nonstandard -run DescriptorSeam -count=1) >"$LOG/fork.log" 2>&1 \
        && echo "ok   fork DescriptorSeam tests" || bad "E: fork seam tests (see $LOG/fork.log)"
fi

[ $fail = 0 ] && echo "gate: PASS" || echo "gate: FAIL"
exit $fail
