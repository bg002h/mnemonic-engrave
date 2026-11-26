#!/usr/bin/env bash
# fix-me-cli-arg-contract-gate.sh — the build gate for design/FIX_me_cli_arg_contract.md
# (bughunt-1119 bh-me-I3, bh-me-I11, bh-me-I18; ruling T32).
#
#   bash scripts/fix-me-cli-arg-contract-gate.sh --cli   # CLI rows only (fast; the RED demonstration)
#   bash scripts/fix-me-cli-arg-contract-gate.sh         # everything below
#
# What it runs:
#   A. CLI rows: builds `me` (--locked) and `me-preview` (go build, preview/), then
#      runs every row of FIX §5 in a fresh directory under target/ and checks exit
#      code, the exact stderr line(s) FIX §4 decides, and the files written or not
#      written. On master every RED row fails here; the controls pass.
#   B. the full nextest suite ONCE to a log (Go on PATH, ME_REQUIRE_GO=1, as CI);
#      each named FIX test must PASS exactly once, and no FAIL line may appear.
#   C. the me-preview Go tests (`go test ./...` in preview/, as CI) with each
#      named FIX test passing, and gofmt -l preview/ empty.
#   D. doc-tests, cargo fmt --check, and both clippy runs of the CI `test` job on
#      the CI-pinned toolchain (1.85.0, release.yml RUST_TOOLCHAIN).
# NOT covered: shells other than the host `sh` for the I11 re-run rows (the quoting
# rule is POSIX single-quoting, which bash, dash and zsh read the same way; fish
# reads \\ and \' inside single quotes as escapes, so a path with a backslash
# prints the placeholder; only `sh` is executed here); CI jobs this change does not touch (OTP
# e2e, shellcheck, cross builds, the ndef-roundtrip oracle) — the PR's CI runs
# them. Wording quality is not machine-checkable: the new texts are PROPOSED in
# the FIX doc. Rows A5 and A5b assert the STOP-1 recommendation; if the architect
# rules otherwise, those two rows change with the ruling.
set -uo pipefail
cd "$(dirname "$0")/.."
ROOT=$PWD
export PATH="$HOME/.cargo/bin:$PATH"
GO=/scratch/code/shibboleth/.toolchain/go/bin
[ -d "$GO" ] && export PATH="$GO:$PATH"
export GOTOOLCHAIN=local
LOG="${GATE_LOG_DIR:-$ROOT/target}/fix-cli-arg-contract-gate"
mkdir -p "$LOG"
fail=0
bad() { echo "FAIL $*"; fail=1; }
ok() { echo "ok   $*"; }

MD1=md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3
MD1B=md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374
MD1C=md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz
MS1=ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw
USAGE='usage: me-preview [--version] | render --format <svg|png> --out <FILE|-> [--mode <text+qr|text|qr>]'

echo "== A. CLI rows"
cargo build --locked -p mnemonic-engrave --bin me >"$LOG/build.log" 2>&1 || { bad "build me (see $LOG/build.log)"; exit 1; }
ME="${CARGO_TARGET_DIR:-$ROOT/target}/debug/me"
case "$ME" in /*) ;; *) ME="$ROOT/$ME" ;; esac
[ -f third_party/seedhammer/go.mod ] || { bad "A: run 'git submodule update --init third_party/seedhammer' first"; exit 1; }
MP="$LOG/me-preview"
(cd preview && GOCACHE="${GOCACHE:-$ROOT/target/gocache}" go build -o "$MP" .) >"$LOG/go-build.log" 2>&1 || { bad "build me-preview (see $LOG/go-build.log)"; exit 1; }

D=$(mktemp -d "$LOG/run.XXXXXX"); cd "$D" || exit 1
umask 077
printf 'text:6162\n' >given
printf 'text:7172\n' >other
printf '%s\n' "$MD1" >md1.txt
for r in 6162 6869; do "$ME" sysw pack --no-passphrase --out "ref$r.bin" "text:$r" 2>/dev/null; done
has_line() { grep -qxF -- "$1" "$2"; }
exists() { [ -e "$1" ]; }

# A1 bh_me_i3_root_in_honoured_by_sysw_pack
printf 'text:6364\n' | "$ME" --in given sysw pack --no-passphrase --out a1.bin 2>a1.err; rc=$?
[ $rc = 0 ] && cmp -s a1.bin ref6162.bin && ok A1 || bad "A1: exit $rc, a1.bin is not text:6162's container"

# A2 bh_me_i3_root_out_honoured_by_sysw_pack
"$ME" --out a2.bin sysw pack --no-passphrase text:6869 >a2.stdout 2>a2.err; rc=$?
[ $rc = 0 ] && cmp -s a2.bin ref6869.bin && [ ! -s a2.stdout ] && ok A2 \
    || bad "A2: exit $rc, a2.bin $(exists a2.bin && echo present || echo absent), stdout $(wc -c <a2.stdout) bytes"

# A3 bh_me_i3_root_in_conflict_refused_by_sysw_pack
"$ME" --in given sysw pack --in other --no-passphrase --out a3.bin 2>a3.err; rc=$?
[ $rc = 64 ] && [ "$(cat a3.err)" = "me: --in was given before 'sysw pack' and again after it, with a different value; give it once, after 'sysw pack'" ] && ! exists a3.bin \
    && ok A3 || { bad "A3: exit $rc"; head -2 a3.err | sed 's/^/    got: /'; }

# A4 bh_me_i3_root_in_same_value_honoured
"$ME" --in given sysw pack --in given --no-passphrase --out a4.bin 2>a4.err; rc=$?
[ $rc = 0 ] && cmp -s a4.bin ref6162.bin && ok A4 || bad "A4: exit $rc"

# A5 bh_me_i3_root_converter_option_refused_by_sysw_pack (STOP-1 recommendation)
"$ME" --hex sysw pack --no-passphrase --out a5.bin text:6162 2>a5.err; rc=$?
[ $rc = 64 ] && [ "$(cat a5.err)" = "me: --hex is a converter option (me with no subcommand); 'sysw pack' does not take it" ] && ! exists a5.bin \
    && ok A5 || { bad "A5: exit $rc"; head -2 a5.err | sed 's/^/    got: /'; }

# A5b bh_me_i3_first_misplaced_option_is_named (STOP-1 recommendation)
"$ME" --echo --in given bundle --in other >/dev/null 2>a5b.err; rc=$?
[ $rc = 64 ] && [ "$(cat a5b.err)" = "me: --echo is a converter option (me with no subcommand); 'bundle' does not take it" ] \
    && ok A5b || { bad "A5b: exit $rc"; head -2 a5b.err | sed 's/^/    got: /'; }

# A5c bh_me_i3_first_misplaced_option_is_named (P-2: command-line order, a conflict before a not-taken option)
"$ME" --in given --echo bundle --in other >/dev/null 2>a5c.err; rc=$?
[ $rc = 64 ] && [ "$(cat a5c.err)" = "me: --in was given before 'bundle' and again after it, with a different value; give it once, after 'bundle'" ] \
    && ok A5c || { bad "A5c: exit $rc"; head -2 a5c.err | sed 's/^/    got: /'; }

# A5d bh_me_i3_misplaced_option_after_valued_option_is_named (review I1: a value-taking root option does not end the scan)
"$ME" --in given --hex sysw pack --no-passphrase --out a5d.bin >/dev/null 2>a5d.err; rc=$?
[ $rc = 64 ] && [ "$(cat a5d.err)" = "me: --hex is a converter option (me with no subcommand); 'sysw pack' does not take it" ] && ! exists a5d.bin \
    && ok A5d || { bad "A5d: exit $rc"; head -2 a5d.err | sed 's/^/    got: /'; }

# A6 bh_me_i3_root_out_honoured_by_seal
"$ME" --out a6.uf2 seal --plaintext "$MD1" --plaintext "$MD1B" --plaintext "$MD1C" </dev/null 2>a6.err; rc=$?
[ $rc = 0 ] && exists a6.uf2 && has_line "load:  picotool load --verify a6.uf2   (machine in BOOTSEL)" a6.err \
    && ok A6 || { bad "A6: exit $rc"; head -3 a6.err | sed 's/^/    got: /'; }

# A6b bh_me_i3_root_in_honoured_by_seal (seal's --in is the field in_path: match by long name, not by id)
printf '%s\n%s\n%s\n' "$MD1" "$MD1B" "$MD1C" >recs.txt
"$ME" --in recs.txt seal --out a6b.uf2 </dev/null 2>a6b.err; rc=$?
[ $rc = 0 ] && exists a6b.uf2 && has_line "load:  picotool load --verify a6b.uf2   (machine in BOOTSEL)" a6b.err \
    && ok A6b || { bad "A6b: exit $rc"; head -2 a6b.err | sed 's/^/    got: /'; }

# A6c bh_me_i3_root_out_honoured_by_sysw_wipe
"$ME" sysw wipe --fill zeros --out a6c.ref 2>/dev/null
"$ME" --out a6c.bin sysw wipe --fill zeros >a6c.stdout 2>a6c.err; rc=$?
[ $rc = 0 ] && cmp -s a6c.bin a6c.ref && [ ! -s a6c.stdout ] && ok A6c \
    || bad "A6c: exit $rc, a6c.bin $(exists a6c.bin && echo present || echo absent), stdout $(wc -c <a6c.stdout) bytes"

# A7 bh_me_i3_root_in_conflict_refused_by_bundle
"$ME" --in given bundle --in other >/dev/null 2>a7.err; rc=$?
[ $rc = 64 ] && [ "$(cat a7.err)" = "me: --in was given before 'bundle' and again after it, with a different value; give it once, after 'bundle'" ] \
    && ok A7 || { bad "A7: exit $rc"; head -2 a7.err | sed 's/^/    got: /'; }

# A8 bh_me_i3_root_allow_world_readable_honoured_by_sysw_pack
(umask 022; "$ME" --allow-world-readable sysw pack --no-passphrase text:6162 >a8.bin 2>a8.err; echo $? >a8.rc)
[ "$(cat a8.rc)" = 0 ] && cmp -s a8.bin ref6162.bin && ok A8 || { bad "A8: exit $(cat a8.rc)"; head -1 a8.err | sed 's/^/    got: /'; }

# A9 control: the argv guard still runs first (exit 3) with a converter option before `seal`
"$ME" --out a9.uf2 seal --allow-argv-secret "$MS1" </dev/null 2>a9.err; rc=$?
[ $rc = 8 ] && head -1 a9.err | grep -qF 'me: argument 5 on ARGV' && ! exists a9.uf2 && ok "A9 control" || bad "A9: exit $rc"

# A10 control: seal with --out in neither place keeps clap's text byte for byte
"$ME" seal --plaintext "$MD1" </dev/null 2>a10.err; rc=$?
cat >a10.want <<'EOF'
error: the following required arguments were not provided:
  --out <OUT>

Usage: me seal --out <OUT> --plaintext <PLAINTEXT> [PAYLOAD]...

For more information, try '--help'.
EOF
[ $rc = 64 ] && cmp -s a10.err a10.want && ok "A10 control" || { bad "A10: exit $rc"; diff a10.want a10.err | head -5; }

# A11 control: the converter with no subcommand is unchanged
"$ME" --in md1.txt --hex >a11.out 2>a11.err; rc=$?
[ $rc = 0 ] && [ "$(tr -d '\n' <a11.out | wc -c)" = 150 ] && ok "A11 control" || bad "A11: exit $rc"

# A12 bh_me_i11_printed_show_command_runs_as_printed
i=0
for pair in "my copy.bin|me sysw show 'my copy.bin'" "it's.bin|me sysw show 'it'\\''s.bin'" "-dash.bin|me sysw show ./-dash.bin"; do
    i=$((i + 1)); name=${pair%%|*}; want=${pair#*|}
    "$ME" sysw pack --no-passphrase --out="$name" text:6162 2>a12.$i.err; rc=$?
    got=$(sed -n 's/^          re-print it with: //p' a12.$i.err)
    [ $rc = 0 ] && [ "$got" = "$want" ] || { bad "A12[$name]: exit $rc, printed [$got], want [$want]"; continue; }
    mkdir -p bin && ln -sf "$ME" bin/me
    PATH="$D/bin:$PATH" sh -c "$got" >a12.$i.show 2>&1; rc=$?
    [ $rc = 0 ] && [ "$(grep '^digest:' a12.$i.show)" = "$(grep '^digest:' a12.$i.err)" ] \
        && ok "A12[$name]" || bad "A12[$name]: printed command exit $rc or digest differs"
done

# A13 control: a plain path prints unquoted, as today
"$ME" sysw pack --no-passphrase --out plain.bin text:6162 2>a13.err; rc=$?
[ $rc = 0 ] && has_line "          re-print it with: me sysw show plain.bin" a13.err && ok "A13 control" || bad "A13: exit $rc"

# A14 bh_me_i11_printed_picotool_path_is_one_shell_word
"$ME" seal --plaintext "$MD1" --plaintext "$MD1B" --plaintext "$MD1C" --out='my copy.uf2' </dev/null 2>a14.err; rc=$?
line=$(grep '^load:' a14.err)
words=$(sh -c "set -- $(printf '%s' "$line" | sed -e 's/^load:  picotool load --verify //' -e 's/   (machine in BOOTSEL)$//'); echo \$#")
[ $rc = 0 ] && [ "$line" = "load:  picotool load --verify 'my copy.uf2'   (machine in BOOTSEL)" ] && [ "$words" = 1 ] \
    && ok A14 || bad "A14: exit $rc, printed [$line], words $words"

# A15 TestBhMeI18StrayOperandRefused
printf '%s\n' "$MD1" | "$MP" render --format svg --out a15.svg second-card >a15.out 2>a15.err; rc=$?
printf 'me-preview: unexpected operand "second-card"\n%s\n' "$USAGE" >a15.want
[ $rc = 1 ] && cmp -s a15.err a15.want && ! exists a15.svg && [ ! -s a15.out ] && ok A15 || { bad "A15: exit $rc"; head -2 a15.err | sed 's/^/    got: /'; }

# A16 TestBhMeI18StrayOperandBeforeModeRefused
printf '%s\n' "$MD1" | "$MP" render --format svg --out a16.svg second-card --mode qr >a16.out 2>a16.err; rc=$?
[ $rc = 1 ] && cmp -s a16.err a15.want && ! exists a16.svg && [ ! -s a16.out ] && ok A16 || { bad "A16: exit $rc"; head -2 a16.err | sed 's/^/    got: /'; }

# A17 control: TestBhMeI18ModeQrControl
printf '%s\n' "$MD1" | "$MP" render --format svg --out a17.svg --mode qr >a17.out 2>a17.err; rc=$?
[ $rc = 0 ] && [ "$(cat a17.out)" = "mode qr" ] && [ -s a17.svg ] && ok "A17 control" || bad "A17: exit $rc"

cd "$ROOT" || exit 1
[ "${1:-}" = "--cli" ] && { [ $fail = 0 ] && echo "gate --cli: PASS" || echo "gate --cli: FAIL"; exit $fail; }

echo "== B. nextest (whole workspace, once)"
ME_REQUIRE_GO=1 cargo nextest run --locked --no-fail-fast --build-jobs 2 -j 2 >"$LOG/nextest.log" 2>&1
echo "nextest exit $?; summary: $(grep -E '^ *Summary' "$LOG/nextest.log" | tail -1)"
grep -E '^ *(FAIL|SIGSEGV|TIMEOUT|LEAK)' "$LOG/nextest.log" | sed 's/^/    /' && bad "B: failing tests (above)"
for t in \
    bh_me_i3_root_in_honoured_by_sysw_pack \
    bh_me_i3_root_out_honoured_by_sysw_pack \
    bh_me_i3_root_in_conflict_refused_by_sysw_pack \
    bh_me_i3_root_in_same_value_honoured \
    bh_me_i3_root_converter_option_refused_by_sysw_pack \
    bh_me_i3_first_misplaced_option_is_named \
    bh_me_i3_misplaced_option_after_valued_option_is_named \
    bh_me_i3_root_out_honoured_by_seal \
    bh_me_i3_root_in_honoured_by_seal \
    bh_me_i3_root_out_honoured_by_sysw_wipe \
    bh_me_i3_root_in_conflict_refused_by_bundle \
    bh_me_i3_root_allow_world_readable_honoured_by_sysw_pack \
    bh_me_i3_controls_guard_seal_text_converter_unchanged \
    bh_me_i11_printed_show_command_runs_as_printed \
    bh_me_i11_printed_show_plain_path_unchanged \
    bh_me_i11_printed_picotool_path_is_one_shell_word; do
    c=$(grep -cE "^ *PASS .*[ :]$t\$" "$LOG/nextest.log")
    [ "$c" = 1 ] && ok "$t" || bad "B: $t passed $c times, want 1"
done

echo "== C. me-preview Go tests"
(cd preview && GOCACHE="${GOCACHE:-$ROOT/target/gocache}" go test -v ./...) >"$LOG/go-test.log" 2>&1 || bad "C: go test (see $LOG/go-test.log)"
for t in TestBhMeI18StrayOperandRefused TestBhMeI18StrayOperandBeforeModeRefused TestBhMeI18ModeQrControl; do
    c=$(grep -cE "^--- PASS: $t " "$LOG/go-test.log")
    [ "$c" = 1 ] && ok "$t" || bad "C: $t passed $c times, want 1"
done
[ -z "$(gofmt -l preview)" ] || bad "C: gofmt -l preview is not empty"

echo "== D. doc-tests and lints"
cargo test --locked --doc >"$LOG/doctest.log" 2>&1 || bad "D: doc-tests (see $LOG/doctest.log)"
cargo fmt --check >"$LOG/fmt.log" 2>&1 || bad "D: cargo fmt --check (see $LOG/fmt.log)"
cargo +1.85.0 clippy --all-targets --locked -- -D warnings >"$LOG/clippy.log" 2>&1 || bad "D: clippy (see $LOG/clippy.log)"
cargo +1.85.0 clippy -p mnemonic-engrave --lib --no-default-features --locked -- -D warnings >"$LOG/clippy-nodef.log" 2>&1 \
    || bad "D: no-default-features clippy (see $LOG/clippy-nodef.log)"

[ $fail = 0 ] && echo "gate: PASS" || echo "gate: FAIL"
exit $fail
