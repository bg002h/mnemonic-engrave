#!/usr/bin/env bash
# fix-me-descriptor-input-strict-gate.sh — the build gate for
# design/FIX_me_descriptor_input_strict.md (bughunt-1119 bh-me-I2, bh-me-I15; ruling T31).
#
#   bash scripts/fix-me-descriptor-input-strict-gate.sh --cli   # CLI rows only (fast; the RED demonstration)
#   bash scripts/fix-me-descriptor-input-strict-gate.sh         # everything below
#
# What it runs:
#   A. CLI rows: builds `me` (--locked) and runs every row of
#      design/evidence/fix-me-descriptor-input-strict/cases.txt as
#      `me sysw pack --no-passphrase [--as <mode>] --out <tmp> --in <fixture>`.
#      Each row requires the exit code, and (when the row names one) that stderr
#      carries want/<key>.err as one whole line. Plus one container check: the
#      2147483647 control packs a container different from base.txt's.
#   B. the full nextest suite ONCE to a log (Go on PATH, ME_REQUIRE_GO=1, as CI);
#      each named FIX test must PASS exactly once, and no FAIL line may appear.
#   C. doc-tests, cargo fmt --check, and both clippy runs of the CI `test` job on
#      the CI-pinned toolchain (1.85.0, release.yml RUST_TOOLCHAIN).
# NOT covered: the device side (the fork's Go encoding/json and bip32.ParsePath are
# unchanged by this FIX; the author measured them with a host Go probe, see the FIX
# doc); CI jobs this change does not touch (me-preview Go tests, OTP e2e, shellcheck,
# cross builds) — the PR's CI runs them. Wording quality is not machine-checkable:
# the new text is PROPOSED in the FIX doc.
set -uo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
E=design/evidence/fix-me-descriptor-input-strict
LOG="${GATE_LOG_DIR:-target}/fix-descriptor-input-strict-gate"
mkdir -p "$LOG"
fail=0
bad() { echo "FAIL $*"; fail=1; }

echo "== A. CLI rows"
cargo build --locked -p mnemonic-engrave --bin me >"$LOG/build.log" 2>&1 || { bad "build (see $LOG/build.log)"; exit 1; }
ME="${CARGO_TARGET_DIR:-target}/debug/me"
n=0
while IFS='|' read -r fx mode want_rc key; do
    case "$fx" in ''|'#'*) continue ;; esac
    n=$((n + 1))
    tag="$fx[$mode]"
    args=(sysw pack --no-passphrase)
    [ "$mode" = none ] || args+=(--as "$mode")
    rm -f "$LOG/out.bin"
    "$ME" "${args[@]}" --out "$LOG/out.bin" --in "$E/$fx" >"$LOG/row.out" 2>"$LOG/row.err"
    rc=$?
    [ "$rc" = "$want_rc" ] || { bad "$tag: exit $rc, want $want_rc"; sed 's/^/    /' "$LOG/row.err" | head -3; continue; }
    if [ "$key" != - ]; then
        grep -qxF -f "$E/want/$key.err" "$LOG/row.err" || {
            bad "$tag: stderr lacks want/$key.err"; sed 's/^/    got: /' "$LOG/row.err" | grep -v 'read as\|descriptor:\|script:' | head -3
            continue
        }
    fi
    [ "$mode" = descriptor ] && [ "$rc" = 0 ] && cp "$LOG/out.bin" "$LOG/$fx.bin"
    echo "ok   $tag"
done <"$E/cases.txt"
[ "$n" = 66 ] || bad "A: cases.txt has $n rows, want 66"
if [ -f "$LOG/base.txt.bin" ] && [ -f "$LOG/i15-origin-2147483647.txt.bin" ]; then
    cmp -s "$LOG/base.txt.bin" "$LOG/i15-origin-2147483647.txt.bin" \
        && bad "A: unhardened 2147483647 packed the same container as /48h/" || echo "ok   2147483647 container differs from base"
else
    bad "A: a control container is missing"
fi

[ "${1:-}" = "--cli" ] && { [ $fail = 0 ] && echo "gate --cli: PASS" || echo "gate --cli: FAIL"; exit $fail; }

echo "== B. nextest (whole workspace, once)"
[ -f third_party/seedhammer/go.mod ] || { bad "B: run 'git submodule update --init third_party/seedhammer' first"; exit 1; }
GO=/scratch/code/shibboleth/.toolchain/go/bin
[ -d "$GO" ] && export PATH="$GO:$PATH"
ME_REQUIRE_GO=1 cargo nextest run --locked --no-fail-fast --build-jobs 4 -j 4 >"$LOG/nextest.log" 2>&1
echo "nextest exit $?; summary: $(grep -E '^ *Summary' "$LOG/nextest.log" | tail -1)"
grep -E '^ *(FAIL|SIGSEGV|TIMEOUT|LEAK)' "$LOG/nextest.log" | sed 's/^/    /' && bad "B: failing tests (above)"
for t in \
    json_duplicate_members_are_refused_exact_or_case_variant \
    json_duplicate_detection_folds_like_go_encoding_json \
    json_duplicate_check_runs_before_field_lookup \
    json_nested_duplicates_are_not_inspected \
    path_element_refuses_an_index_above_2_pow_31_minus_1 \
    path_element_admits_2147483647_unhardened_and_hardened \
    json_duplicate_member_gets_its_own_sentence \
    out_of_range_path_index_is_refused_not_rewritten; do
    c=$(grep -cE "^ *PASS .*[ :]$t\$" "$LOG/nextest.log")
    [ "$c" = 1 ] && echo "ok   $t" || bad "B: $t passed $c times, want 1"
done

echo "== C. doc-tests and lints"
cargo test --locked --doc >"$LOG/doctest.log" 2>&1 || bad "C: doc-tests (see $LOG/doctest.log)"
cargo fmt --check >"$LOG/fmt.log" 2>&1 || bad "C: cargo fmt --check (see $LOG/fmt.log)"
cargo +1.85.0 clippy --all-targets --locked -- -D warnings >"$LOG/clippy.log" 2>&1 || bad "C: clippy (see $LOG/clippy.log)"
cargo +1.85.0 clippy -p mnemonic-engrave --lib --no-default-features --locked -- -D warnings >"$LOG/clippy-nodef.log" 2>&1 \
    || bad "C: no-default-features clippy (see $LOG/clippy-nodef.log)"

[ $fail = 0 ] && echo "gate: PASS" || echo "gate: FAIL"
exit $fail
