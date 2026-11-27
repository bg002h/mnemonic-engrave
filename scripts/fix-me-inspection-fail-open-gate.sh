#!/usr/bin/env bash
# fix-me-inspection-fail-open-gate.sh — the build gate for
# design/FIX_me_inspection_fail_open.md (bughunt-1119 bh-me-I7, bh-me-I8).
#
#   bash scripts/fix-me-inspection-fail-open-gate.sh --cli   # CLI rows R3-R8 only (fast; the RED demonstration)
#   bash scripts/fix-me-inspection-fail-open-gate.sh         # everything below
#
# What it runs:
#   A. CLI rows (the FIX doc's R3-R8): builds `me` (--locked), packs three containers
#      (`text:6162` unsealed; `text:6162` + `pass:6869` sealed; `pass:6869` sealed, no public
#      section), flips one public-section byte with dd, and runs `me sysw show` on each file.
#      Each row requires the exit code; a refusal row requires its T-UTF8 line as one whole
#      stderr line, no `identity:` on stdout and no `digest:` on stderr; a control row
#      requires exit 0 and its identity/digest lines.
#   B. the full nextest suite ONCE to a log (Go on PATH, ME_REQUIRE_GO=1, as CI); each
#      named FIX test must PASS exactly once, and no FAIL line may appear.
#   C. doc-tests, cargo fmt --check, ci/code-docs-check.sh and both clippy runs of the CI `test` job on the
#      CI-pinned toolchain (1.85.0, release.yml RUST_TOOLCHAIN).
# NOT covered: R1/R2 (library rows) run only in B, as unit tests; the device side
# (firmware reads of a damaged public section are unchanged by this FIX); CI jobs this
# change does not touch (me-preview Go tests, OTP e2e, shellcheck, cross builds) — the
# PR's CI runs them. Wording quality is not machine-checkable: T-UTF8 is PROPOSED.
# The sealed rows print a freshly generated passphrase for a throwaway test container
# to $LOG/*.pack.err; it protects nothing.
set -uo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
LOG="${GATE_LOG_DIR:-target}/fix-inspection-fail-open-gate"
mkdir -p "$LOG"
fail=0
bad() { echo "FAIL $*"; fail=1; }

echo "== A. CLI rows"
cargo build --locked -p mnemonic-engrave --bin me >"$LOG/build.log" 2>&1 || { bad "build (see $LOG/build.log)"; exit 1; }
ME="${CARGO_TARGET_DIR:-target}/debug/me"

# pack <name> <args...>: a fresh container at $LOG/<name>.bin.
pack() {
    local n=$1; shift
    rm -f "$LOG/$n.bin"
    "$ME" sysw pack "$@" --out "$LOG/$n.bin" >"$LOG/$n.pack.out" 2>"$LOG/$n.pack.err" \
        || { bad "pack $n: exit $? (see $LOG/$n.pack.err)"; return 1; }
}
# flip <src> <dst> <offset> <octal byte>: copy src to dst with one byte replaced.
flip() {
    cp "$LOG/$1.bin" "$LOG/$2.bin"
    printf "\\$4" | dd of="$LOG/$2.bin" bs=1 seek="$3" conv=notrunc status=none
}
# refuse <row> <file> <offset>: show must exit 2 with T-UTF8 naming <offset>.
refuse() {
    "$ME" sysw show "$LOG/$2.bin" >"$LOG/$2.show.out" 2>"$LOG/$2.show.err"
    local rc=$?
    local want="me: this file's public section is not UTF-8 text (where UTF-8 decoding first fails, at file offset $3); it is damaged, or it was not written by \`me sysw pack\`. No identity or digest is shown, because either would be a number for records that cannot be read."
    [ "$rc" = 2 ] || bad "$1: exit $rc, want 2"
    grep -qxF -- "$want" "$LOG/$2.show.err" || { bad "$1: stderr lacks the T-UTF8 line for offset $3"; sed 's/^/    got: /' "$LOG/$2.show.err" | head -3; }
    grep -q '^identity:' "$LOG/$2.show.out" && bad "$1: printed an identity"
    grep -q '^digest:' "$LOG/$2.show.err" && bad "$1: printed a digest"
    grep -q '^public record' "$LOG/$2.show.out" && bad "$1: listed records"
    [ "$rc" = 2 ] && echo "ok   $1"
}
# accept <row> <file> <digest-grep>: show must exit 0, print an identity and the digest line.
accept() {
    "$ME" sysw show "$LOG/$2.bin" >"$LOG/$2.show.out" 2>"$LOG/$2.show.err"
    local rc=$?
    [ "$rc" = 0 ] || bad "$1: exit $rc, want 0"
    grep -q '^identity: [0-9a-f]\{64\}$' "$LOG/$2.show.out" || bad "$1: no identity line"
    grep -q -- "$3" "$LOG/$2.show.err" || bad "$1: stderr lacks '$3'"
    [ "$rc" = 0 ] && echo "ok   $1"
}

printf 'text:6162\npass:6869\n' >"$LOG/sealed.in"
printf 'pass:6869\n' >"$LOG/secrets-only.in"
pack plain --no-passphrase text:6162 \
    && pack sealed --passphrase-words 2 --in "$LOG/sealed.in" \
    && pack secrets-only --passphrase-words 2 --in "$LOG/secrets-only.in" \
    || exit 1
flip plain plain-last 60 377
flip plain plain-first 52 200
flip sealed sealed-last 60 377
refuse R3 plain-last 60
refuse R4 plain-first 52
refuse R5 sealed-last 60
accept R6 plain '^digest:   1ad5 a49b 17a1 2dbb 7ba1 093b e9cf 09bb$'
accept R7 secrets-only '^digest:   none — this payload has no public section$'
# R8 (order control): truncation is decided before UTF-8, so a short file whose public
# section is also bad gets the existing truncation sentence, not T-UTF8.
n=$(stat -c %s "$LOG/sealed-last.bin")
head -c $((n - 1)) "$LOG/sealed-last.bin" >"$LOG/sealed-last-short.bin"
"$ME" sysw show "$LOG/sealed-last-short.bin" >"$LOG/sealed-last-short.show.out" 2>"$LOG/sealed-last-short.show.err"
rc=$?
if [ "$rc" = 2 ] && grep -q "but its header declares $n; it is truncated" "$LOG/sealed-last-short.show.err" \
    && ! grep -q 'is not UTF-8 text' "$LOG/sealed-last-short.show.err"; then
    echo "ok   R8"
else
    bad "R8: exit $rc, want 2 with the truncation sentence and no T-UTF8"
fi

[ "${1:-}" = "--cli" ] && { [ $fail = 0 ] && echo "gate --cli: PASS" || echo "gate --cli: FAIL"; exit $fail; }

echo "== B. nextest (whole workspace, once)"
[ -f third_party/seedhammer/go.mod ] || { bad "B: run 'git submodule update --init third_party/seedhammer' first"; exit 1; }
GO=/scratch/code/shibboleth/.toolchain/go/bin
[ -d "$GO" ] && export PATH="$GO:$PATH"
ME_REQUIRE_GO=1 cargo nextest run --locked --no-fail-fast --build-jobs 2 -j 2 >"$LOG/nextest.log" 2>&1
echo "nextest exit $?; summary: $(grep -E '^ *Summary' "$LOG/nextest.log" | tail -1)"
grep -E '^ *(FAIL|SIGSEGV|TIMEOUT|LEAK)' "$LOG/nextest.log" | sed 's/^/    /' && bad "B: failing tests (above)"
for t in \
    region_image_refuses_a_blob_missing_its_last_public_byte \
    region_image_refuses_a_sealed_blob_missing_its_last_tag_byte \
    region_image_pads_with_erased_flash_and_refuses_what_bound_refuses \
    show_refuses_a_public_section_that_is_not_utf8 \
    show_names_the_first_bad_byte_of_the_public_section \
    show_refuses_a_sealed_public_section_that_is_not_utf8 \
    show_prints_the_same_digest_pack_did \
    a_secrets_only_payload_reports_no_digest \
    show_on_a_truncated_container_fails_cleanly_without_printing_a_digest \
    show_reports_truncation_before_an_unreadable_public_section; do
    c=$(grep -cE "^ *PASS .*[ :]$t\$" "$LOG/nextest.log")
    [ "$c" = 1 ] && echo "ok   $t" || bad "B: $t passed $c times, want 1"
done

echo "== C. doc-tests and lints"
cargo +1.85.0 test --locked --doc >"$LOG/doctest.log" 2>&1 || bad "C: doc-tests (see $LOG/doctest.log)"
cargo +1.85.0 fmt --check >"$LOG/fmt.log" 2>&1 || bad "C: cargo fmt --check (see $LOG/fmt.log)"
cargo +1.85.0 clippy --all-targets --locked -- -D warnings >"$LOG/clippy.log" 2>&1 || bad "C: clippy (see $LOG/clippy.log)"
cargo +1.85.0 clippy -p mnemonic-engrave --lib --no-default-features --locked -- -D warnings >"$LOG/clippy-nodef.log" 2>&1 \
    || bad "C: no-default-features clippy (see $LOG/clippy-nodef.log)"
bash ci/code-docs-check.sh >"$LOG/code-docs.log" 2>&1 || bad "C: ci/code-docs-check.sh (see $LOG/code-docs.log)"

[ $fail = 0 ] && echo "gate: PASS" || echo "gate: FAIL"
exit $fail
