#!/usr/bin/env bash
# fix-me-checklist-partial-keys-gate.sh — the build gate for
# design/FIX_me_recovery_checklist_partial_keys.md (bughunt-1119 bh-me-C1).
#
#   bash scripts/fix-me-checklist-partial-keys-gate.sh --cli   # CLI rows only (fast; the RED demonstration)
#   bash scripts/fix-me-checklist-partial-keys-gate.sh         # everything below
#
# What it runs:
#   A. CLI rows: builds `me` (--locked), feeds each case in
#      design/evidence/fix-me-checklist-partial-keys/cases.txt to `me bundle`,
#      and requires exit 0, every `me: NOTE` stderr line equal to <case>.notes
#      (same lines, same order, nothing extra), and the manifest's
#      `missing_keys` equal to <case>.missing.json (`null` = field absent).
#   B. the fixtures' producer in --check mode (needs `md` and `mk` on PATH;
#      skipped with a SKIP line if either is missing).
#   C. every fixture string occurs in crates/me-cli/tests/partial_keys.rs.
#   D. the full nextest suite ONCE to a log (Go on PATH, ME_REQUIRE_GO=1, as
#      CI), then each named FIX test must PASS exactly once in that log, and
#      the log must show no FAIL line.
#   E. doc-tests, cargo fmt --check, and both clippy runs of the CI `test`
#      job on the CI-pinned toolchain (1.85.0, release.yml RUST_TOOLCHAIN).
#      Not scripts/lint-gate.sh: its nightly leg fails on master 2987f388
#      already (f449_stage4a.rs:179 clippy::cloned_ref_to_slice_refs), a
#      lint CI does not run.
# NOT covered: the CI jobs this change does not touch (Go tests of the
# me-preview sidecar and ndef oracle, the OTP e2e, shellcheck, SH2 install
# links, picotool-probe, cross builds); the PR's CI runs them. Wording quality
# is not machine-checkable: the texts are PROPOSED in the FIX doc.
set -uo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
E=design/evidence/fix-me-checklist-partial-keys
LOG="${GATE_LOG_DIR:-target}/fix-partial-keys-gate"
mkdir -p "$LOG"
fail=0
bad() { echo "FAIL $*"; fail=1; }

echo "== A. CLI rows"
cargo build --locked -p mnemonic-engrave --bin me >"$LOG/build.log" 2>&1 || { bad "build (see $LOG/build.log)"; exit 1; }
ME="${CARGO_TARGET_DIR:-target}/debug/me"
while IFS=: read -r case inputs; do
    [ -z "$case" ] && continue
    : >"$LOG/$case.in"
    IFS=, read -r -a parts <<<"$inputs"
    for p in "${parts[@]}"; do cat "$E/$p.txt" >>"$LOG/$case.in"; done
    "$ME" bundle <"$LOG/$case.in" >"$LOG/$case.out" 2>"$LOG/$case.err"
    rc=$?
    [ $rc = 0 ] || { bad "$case: exit $rc, want 0"; continue; }
    grep '^me: NOTE' "$LOG/$case.err" >"$LOG/$case.notes"
    if ! cmp -s "$LOG/$case.notes" "$E/$case.notes"; then
        bad "$case: NOTE lines differ"; diff "$E/$case.notes" "$LOG/$case.notes" | sed 's/^/    /'
    fi
    if ! python3 -I -c 'import json,sys
got=json.load(open(sys.argv[1])).get("missing_keys")
want=json.load(open(sys.argv[2]))
sys.exit(0 if got==want else 1)' "$LOG/$case.out" "$E/$case.missing.json"; then
        bad "$case: missing_keys differs from $E/$case.missing.json"
    else
        echo "ok   $case"
    fi
done <"$E/cases.txt"

[ "${1:-}" = "--cli" ] && { [ $fail = 0 ] && echo "gate --cli: PASS" || echo "gate --cli: FAIL"; exit $fail; }

echo "== B. fixture producer --check"
if command -v md >/dev/null && command -v mk >/dev/null; then
    bash "$E/make-fixtures.sh" --check || bad "fixtures do not reproduce"
else
    echo "SKIP B: md or mk not on PATH"
fi

echo "== C. fixtures occur in the test file"
TF=crates/me-cli/tests/partial_keys.rs
if [ -f "$TF" ]; then
    for f in "$E"/*.txt; do
        case "$f" in */cases.txt) continue ;; esac
        while read -r s; do grep -qF "$s" "$TF" || bad "C: $TF lacks $(basename "$f") line ${s:0:24}..."; done <"$f"
    done
else
    bad "C: $TF does not exist"
fi

echo "== D. nextest (whole workspace, once)"
# The Go oracles build against the submodule; an uninitialised one fails two
# unrelated tests (cross_lang, preview_cross_lang), so refuse up front.
[ -f third_party/seedhammer/go.mod ] || { bad "D: run 'git submodule update --init third_party/seedhammer' first"; exit 1; }
GO=/scratch/code/shibboleth/.toolchain/go/bin
[ -d "$GO" ] && export PATH="$GO:$PATH"
ME_REQUIRE_GO=1 cargo nextest run --locked --no-fail-fast --build-jobs 6 -j 6 >"$LOG/nextest.log" 2>&1
echo "nextest exit $?; summary: $(grep -E '^ *Summary' "$LOG/nextest.log" | tail -1)"
grep -E '^ *(FAIL|SIGSEGV|TIMEOUT|LEAK)' "$LOG/nextest.log" | sed 's/^/    /' && bad "D: failing tests (above)"
for t in \
    partial_keys_one_of_three_names_two_missing_slots \
    partial_keys_middle_slot_only_names_slots_zero_and_two \
    partial_keys_two_of_three_names_the_one_missing_slot \
    partial_keys_missing_fingerprint_renders_bare_slot \
    fully_keyed_policy_has_no_missing_keys \
    keyless_template_lists_every_slot_and_keeps_the_template_note \
    template_and_partial_policies_are_reported_in_plate_order \
    partial_keys_note_fires_with_the_mk1_card_present \
    partial_keys_note_single_plate_form \
    partial_keys_taproot_key_on_second_slot_names_slot_zero \
    a_keyless_template_says_the_cosigner_cards_are_missing \
    bundle_manifest_golden; do
    n=$(grep -cE "^ *PASS .*[ :]$t\$" "$LOG/nextest.log")
    [ "$n" = 1 ] && echo "ok   $t" || bad "D: $t passed $n times, want 1"
done

echo "== E. doc-tests and lints"
cargo test --locked --doc >"$LOG/doctest.log" 2>&1 || bad "E: doc-tests (see $LOG/doctest.log)"
cargo fmt --check >"$LOG/fmt.log" 2>&1 || bad "E: cargo fmt --check (see $LOG/fmt.log)"
cargo +1.85.0 clippy --all-targets --locked -- -D warnings >"$LOG/clippy.log" 2>&1 || bad "E: clippy (see $LOG/clippy.log)"
cargo +1.85.0 clippy -p mnemonic-engrave --lib --no-default-features --locked -- -D warnings >"$LOG/clippy-nodef.log" 2>&1 \
    || bad "E: no-default-features clippy (see $LOG/clippy-nodef.log)"

[ $fail = 0 ] && echo "gate: PASS" || echo "gate: FAIL"
exit $fail
