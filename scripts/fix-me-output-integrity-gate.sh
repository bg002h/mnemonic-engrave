#!/usr/bin/env bash
# fix-me-output-integrity-gate.sh — the build gate for
# design/FIX_me_output_integrity.md (bughunt-1119 bh-me-I4, bh-me-I10).
#
#   bash scripts/fix-me-output-integrity-gate.sh --cli   # step A only (fast; the RED demonstration)
#   bash scripts/fix-me-output-integrity-gate.sh         # A, B and C
#   ME_BIN=/path/to/me bash scripts/fix-me-output-integrity-gate.sh --cli   # A against another binary (the base)
#
# What it runs:
#   A. CLI rows: builds `me` (--locked; skipped when ME_BIN is set) and runs the FIX's rows
#      w1, w2, w4, w5, w11, w12, w14, o1, o2, o7, p1, p4 and p7 as printed commands against it
#      (exit code, stdout, the exact stderr line (the last stderr line for p1, p4 and p7) and the
#      file bytes compared to the fixed column; FSIZE0 by `ulimit -f 0; trap '' XFSZ`).
#   B. the full nextest suite ONCE to a log (Go on PATH, ME_REQUIRE_GO=1, as CI); each named
#      row PASS exactly once and no FAIL line.
#   C. doc-tests, cargo fmt --check, ci/code-docs-check.sh and both clippy runs of the CI `test`
#      job on 1.85.0 (release.yml RUST_TOOLCHAIN).
# NOT covered: macOS/Windows legs (the /dev/full, ulimit and fcntl rows are Linux/unix), the device
# side (unchanged), the release workflow (CI only), the WORDING_REVIEW rows for the W2 and P1 texts.
# Every cargo call runs with the caller's environment (CARGO_BUILD_JOBS=6 and the jobserver prefix
# are the caller's job; see the coordinator's cargo-env standard).
set -uo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
LOG="${GATE_LOG_DIR:-target}/fix-output-integrity-gate"
mkdir -p "$LOG"
fail=0
rowbad=0
bad() { echo "FAIL $*"; fail=1; rowbad=1; }

echo "== A. CLI rows"
if [ -z "${ME_BIN:-}" ]; then
    cargo +1.85.0 build --locked -p mnemonic-engrave --bin me >"$LOG/build.log" 2>&1 || { bad "build (see $LOG/build.log)"; exit 1; }
    ME_BIN="${CARGO_TARGET_DIR:-target}/debug/me"
fi
ME="$(cd "$(dirname "$ME_BIN")" && pwd)/$(basename "$ME_BIN")"
export ME
MD1=md1yqpqqxqq8xtwhw4xwn4qh
MK1_A=mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf
MK1_B=mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x
NOSPC='me: cannot write the output: No space left on device (os error 28)'
F0="ulimit -f 0; trap '' XFSZ; "
WORK="$(cd "$(mktemp -d "$LOG/work.XXXXXX")" && pwd)"

# newrow <name>: a fresh directory with the fixtures; sets D.
newrow() {
    D="$WORK/$1"
    rowbad=0
    mkdir -p "$D"
    printf '%s\n' "$MD1" >"$D/m.txt"
    printf '%s\n%s\n%s\n' "$MD1" "$MK1_A" "$MK1_B" >"$D/b.txt"
    (cd "$D" && umask 077 && "$ME" sysw pack --no-passphrase text:6869 --out c.bin >/dev/null 2>&1) || bad "$1: fixture pack"
    printf '%s\n' '#!/bin/sh' \
        'if [ "$1" = "--version" ]; then echo "me-preview '"$(sed -n 's/^version = "\(.*\)"/\1/p' crates/me-cli/Cargo.toml | head -1)"'"; exit 0; fi' \
        'if [ "$1" = "render" ]; then out=""; while [ "$#" -gt 0 ]; do if [ "$1" = "--out" ]; then out="$2"; fi; shift; done; cat >/dev/null; printf "<svg/>" >"$out"; echo "mode text"; exit 0; fi' \
        'exit 1' >"$D/fake-me-preview"
    chmod 755 "$D/fake-me-preview"
}
# go <name> <cmd>: run cmd in $D under umask 077 (stdin /dev/null); sets rc, $D/out, $D/err.
go() {
    # stderr goes through a pipe, never a file: a size limit on a stderr file would make the
    # shell fail for the wrong reason (triage t1 note (a)).
    (cd "$D" && ME_PREVIEW_BIN="${PB:-}" bash -c "umask 077; $2" 2>&1 >"$D/out" </dev/null | cat >"$D/err"; exit "${PIPESTATUS[0]}")
    rc=$?
}
last() { tail -n 1 "$D/err"; }
# check <row> <want rc>
rcis() { [ "$rc" = "$2" ] || bad "$1: exit $rc, want $2 (stderr: $(tail -n 2 "$D/err" | tr '\n' '|'))"; }
lastis() { [ "$(last)" = "$2" ] || bad "$1: last stderr line is '$(last)', want '$2'"; }
errexact() { [ "$(cat "$D/err")" = "$2" ] || bad "$1: stderr is '$(tr '\n' '|' <"$D/err")', want exactly '$2'"; }
nocard() { grep -q '^digest:' "$D/err" && bad "$1: a digest card was printed"; return 0; }
done_row() { [ "$rowbad" = 0 ] && echo "ok   $1"; return 0; }

newrow w1; go w1 "${F0}\"\$ME\" sysw pack --no-passphrase text:6869 > sink.bin"
rcis w1 32; lastis w1 'me: cannot write the output: File too large (os error 27)'; nocard w1; done_row w1
newrow w2; go w2 "\"\$ME\" sysw pack --no-passphrase text:6869 > /dev/full"
rcis w2 32; lastis w2 "$NOSPC"; nocard w2; done_row w2
newrow w4; go w4 "\"\$ME\" --in m.txt --stdout > /dev/full"
rcis w4 32; errexact w4 "$NOSPC"; done_row w4
newrow w5; go w5 "\"\$ME\" --in m.txt --hex > /dev/full"
rcis w5 32; errexact w5 "$NOSPC"; done_row w5
newrow w11; go w11 "\"\$ME\" sysw wipe > /dev/full"
rcis w11 32; errexact w11 "$NOSPC"; done_row w11
newrow w12; go w12 "\"\$ME\" --help > /dev/full"
rcis w12 32; errexact w12 "$NOSPC"; done_row w12
newrow w14; go w14 "(sleep 0.5; exec \"\$ME\" sysw show c.bin) | true; exit \${PIPESTATUS[0]}"
rcis w14 0; grep -q panicked "$D/err" && bad "w14: a panic"; grep -q 'cannot write' "$D/err" && bad "w14: a cannot-write line"; done_row w14

newrow o1; printf 'OLD BACKUP\n' >"$D/F.bin"; go o1 "${F0}\"\$ME\" sysw pack --no-passphrase text:6869 --out F.bin"
rcis o1 32; lastis o1 'me: cannot write the output: F.bin: File too large (os error 27). F.bin is unchanged.'
[ "$(cat "$D/F.bin")" = 'OLD BACKUP' ] || bad "o1: F.bin changed"; done_row o1
newrow o2; go o2 "\"\$ME\" sysw pack --no-passphrase text:6869 --out nodir/x.bin"
rcis o2 32; lastis o2 'me: cannot write the output: nodir/x.bin: No such file or directory (os error 2). Nothing was written.'; nocard o2; done_row o2
newrow o7; printf 'OLD BACKUP\n' >"$D/s.txt"; go o7 "${F0}\"\$ME\" sysw pack --no-passphrase text:6869 --out /dev/stdout >> s.txt"
rcis o7 32; lastis o7 'me: cannot write the output: /dev/stdout: File too large (os error 27)'
[ "$(stat -c %s "$D/s.txt")" = 11 ] || bad "o7: s.txt is $(stat -c %s "$D/s.txt") bytes, want 11"; done_row o7

p1text() { echo "me: --manifest $1 is the same file as the plate preview $2; write the manifest to a new name"; }
prow() { # prow <name> <args> <M> <P> <mkdir-dir>
    newrow "$1"; mkdir "$D/$5"
    [ "$1" = p4 ] && ln -s p/plate-2.svg "$D/L"
    PB="$D/fake-me-preview"
    [ "$1" = p7 ] && PB=""
    go "$1" "\"\$ME\" bundle --in b.txt --preview $5 --manifest $3"
    rcis "$1" 8; lastis "$1" "$(p1text "$3" "$4")"
    grep -E 'rendered plate|wrote manifest|preview skipped|backup needs' "$D/err" >/dev/null && bad "$1: a line that must not print"
    [ -s "$D/out" ] && bad "$1: stdout is not empty"
    [ -z "$(ls -A "$D/$5")" ] || bad "$1: $5 is not empty"
    done_row "$1"
}
prow p1 x p/plate-1.svg p/plate-1.svg p
prow p4 x L p/plate-2.svg p
prow p7 x p9/plate-1.svg p9/plate-1.svg p9

[ "${1:-}" = "--cli" ] && { [ $fail = 0 ] && echo "gate --cli: PASS" || echo "gate --cli: FAIL"; exit $fail; }

echo "== B. nextest (whole workspace, once)"
[ -f third_party/seedhammer/go.mod ] || { bad "B: run 'git submodule update --init third_party/seedhammer' first"; exit 1; }
GO=/scratch/code/shibboleth/.toolchain/go/bin
[ -d "$GO" ] && export PATH="$GO:$PATH"
ME_REQUIRE_GO=1 cargo +1.85.0 nextest run --locked --workspace --all-targets --no-fail-fast --build-jobs 6 -j 6 >"$LOG/nextest.log" 2>&1
echo "nextest exit $?; summary: $(grep -E '^ *Summary' "$LOG/nextest.log" | tail -1)"
grep -E '^ *(FAIL|SIGSEGV|TIMEOUT|LEAK)' "$LOG/nextest.log" | sed 's/^/    /' && bad "B: failing tests (above)"
for t in \
    w1_pack_stdout_rejected_by_fsize_limit_is_not_success w2_pack_stdout_full_exits_1 \
    w3_pack_read_only_stdout_exits_1 w4_converter_raw_stdout_full_exits_1 w5_converter_hex_full_exits_1 \
    w6_converter_base64_full_exits_1 w7_converter_hex_read_only_stdout_exits_1 \
    w8_bundle_stdout_full_exits_1_without_checklist w9_show_stdout_full_exits_1 w10_hash_stdout_full_exits_1 \
    w11_wipe_stdout_full_exits_1 w12_help_stdout_full_exits_1 w13_version_stdout_full_exits_1 \
    w14_show_closed_pipe_keeps_exit_0 w15_converter_raw_closed_pipe_keeps_exit_0 w16_pack_stdout_success_is_unchanged \
    o1_pack_failed_overwrite_says_unchanged o2_pack_missing_dir_says_nothing_was_written \
    o3_pack_failed_new_file_leaves_nothing o4_pack_read_only_dir_says_unchanged o5_converter_out_missing_dir \
    o6_bundle_manifest_missing_dir o7_stdio_alias_onto_a_file_names_no_state o8_seal_out_missing_dir \
    o9_pack_out_device_names_no_state p1_manifest_is_svg_plate_refused p2_manifest_is_png_plate_refused \
    p3_manifest_dot_slash_spelling_refused p4_manifest_symlink_to_plate_refused \
    p5_manifest_through_dir_symlink_refused p6_manifest_hard_link_at_plate_name_refused \
    p7_manifest_on_plate_refused_without_sidecar p8_distinct_manifest_still_written \
    p9_unplanned_plate_name_still_written p10_manifest_alias_made_during_render_refused \
    a_failed_write_is_exit_one; do
    c=$(grep -cE "^ *PASS .*[ :]$t\$" "$LOG/nextest.log")
    [ "$c" = 1 ] && echo "ok   $t" || bad "B: $t passed $c times, want 1"
done

echo "== C. doc-tests and lints"
cargo +1.85.0 test --locked --doc >"$LOG/doctest.log" 2>&1 || bad "C: doc-tests (see $LOG/doctest.log)"
cargo +1.85.0 fmt --all --check >"$LOG/fmt.log" 2>&1 || bad "C: cargo fmt --check (see $LOG/fmt.log)"
cargo +1.85.0 clippy --all-targets --locked -- -D warnings >"$LOG/clippy.log" 2>&1 || bad "C: clippy (see $LOG/clippy.log)"
cargo +1.85.0 clippy -p mnemonic-engrave --lib --no-default-features --locked -- -D warnings >"$LOG/clippy-nodef.log" 2>&1 \
    || bad "C: no-default-features clippy (see $LOG/clippy-nodef.log)"
bash ci/code-docs-check.sh >"$LOG/code-docs.log" 2>&1 || bad "C: ci/code-docs-check.sh (see $LOG/code-docs.log)"

[ $fail = 0 ] && echo "gate: PASS" || echo "gate: FAIL"
exit $fail
