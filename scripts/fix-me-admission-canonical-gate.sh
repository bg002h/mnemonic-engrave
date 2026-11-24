#!/usr/bin/env bash
# fix-me-admission-canonical-gate.sh — the build gate for
# design/FIX_me_admission_canonical.md (bughunt-1119 bh-me-I1, I6, I9, I17; closes F-245).
#
#   bash scripts/fix-me-admission-canonical-gate.sh --cli   # CLI rows only (fast; the RED demonstration)
#   bash scripts/fix-me-admission-canonical-gate.sh         # everything below
#
# What it runs:
#   A. CLI rows R1-R16 and controls C1-C7 of the FIX doc §5: builds `me` (--locked), runs
#      `me sysw pack` once per row in a private scratch dir (umask 077). A refusal row requires
#      exit 4, its PROPOSED sentence as one whole stderr line (T-SEP, T-LF, T-PAD, T-UTF8BODY;
#      the existing T-UNREC first line for C5/C6), and no output file. An accept row requires
#      exit 0 and the container's records (bytes 52.. of the unsealed file) equal to the
#      expected bytes exactly.
#   B. the full nextest suite ONCE to a log (Go on PATH, ME_REQUIRE_GO=1, as CI); each named
#      FIX test must PASS exactly once, and no FAIL line may appear.
#   C. doc-tests, cargo fmt --check, ci/code-docs-check.sh and both clippy runs of the CI
#      `test` job on the CI-pinned toolchain (1.85.0, release.yml RUST_TOOLCHAIN).
# NOT covered: the library rows L1-L8 run only in B (as tests); the device side (the fork's
# classifier is unchanged by this FIX; the F1 residue is a separate FOLLOWUPS line); CI jobs
# this change does not touch (me-preview Go tests, OTP e2e, shellcheck, cross builds) — the
# PR's CI runs them. Wording quality is not machine-checkable: every new sentence is PROPOSED.
# Test material only: the BIP-39 all-zero test vector (abandon x11 about) and the public README
# card md1yqpqqxqq8xtwhw4xwn4qh. R16 prints a freshly generated passphrase only if the FIX is
# missing; it protects nothing.
set -uo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
LOG="${GATE_LOG_DIR:-target}/fix-admission-canonical-gate"
mkdir -p "$LOG"
fail=0
bad() { echo "FAIL $*"; fail=1; }

echo "== A. CLI rows"
cargo build --locked -p mnemonic-engrave --bin me >"$LOG/build.log" 2>&1 || { bad "build (see $LOG/build.log)"; exit 1; }
ME="$(cd "${CARGO_TARGET_DIR:-target}" && pwd)/debug/me"
W="$LOG/run"; mkdir -p "$W"; chmod 700 "$W"; umask 077
D=md1yqpqqxqq8xtwhw4xwn4qh
M="$(printf 'abandon %.0s' 1 2 3 4 5 6 7 8 9 10 11)about"
M2="${M/ /  }"; MT="${M/ /$'\t'}"; MN="${M/ /$' '}"
W6="${M:0:47}"$'\n'"${M:48}"
BOM=$'\xef\xbb\xbf'
export ME D M M2 MT MN W6 BOM

T_SEP='me: record 0, as given (records count from 0) is a BIP-39 mnemonic whose words are not separated by single spaces (a doubled space, a tab or another space character). The device reads a seed only in that spelling and would not recognise this record as one. Separate the words with one ordinary space each and re-run.'
T_LF='me: record 0, as given (records count from 0) contains a line break (LF or CR). A record is one line, and records are stored separated by LF, so this one would be read back as several records. Give each record its own line (--in or stdin) or its own argument; to pack a whole descriptor document, pass it with --as.'
T_PAD='me: record 0, as given (records count from 0) begins or ends with whitespace. Records are stored exactly as given, so the whitespace would be engraved as part of this one. Remove it and re-run.'
t_utf8() { printf 'me: record 0, as given (records count from 0) begins `%s` and its body is lowercase hex, but the bytes it decodes to are not UTF-8 text (§5.3.1). Encode the text'"'"'s own UTF-8 bytes:' "$1"; }
T_UNREC_HEAD='me: record 0, as given (records count from 0) is not a form this container can place'

# run <row> <snippet>: run the snippet in $W with o.bin removed first.
run() { (cd "$W" && rm -f o.bin && bash -c "$2" >"$1.out" 2>"$1.err"); echo $? >"$W/$1.rc"; }
# refuse <row> <whole-line|--prefix head> <snippet>
refuse() {
    local row=$1 want=$2 mode=x; shift 2
    if [ "$want" = --prefix ]; then mode=prefix; want=$1; shift; fi
    run "$row" "$1"
    local rc; rc=$(cat "$W/$row.rc")
    [ "$rc" = 4 ] || bad "$row: exit $rc, want 4"
    if [ $mode = x ]; then
        grep -qxF -- "$want" "$W/$row.err" || { bad "$row: stderr lacks its sentence as a whole line"; grep '^me: record' "$W/$row.err" | head -2 | cut -c1-200 | sed 's/^/    got: /'; }
    else
        grep -q "^$(printf '%s' "$want" | sed 's/[][\.*^$/]/\\&/g')" "$W/$row.err" || bad "$row: stderr lacks '$want'"
    fi
    [ -e "$W/o.bin" ] && bad "$row: an output file was written"
    grep -q 'write this down' "$W/$row.err" && bad "$row: the passphrase ceremony ran before the refusal"
    [ "$rc" = 4 ] && echo "ok   $row"
}
# accept <row> <expected records, printf format> <snippet>
accept() {
    local row=$1 want=$2; shift 2
    run "$row" "$1"
    local rc; rc=$(cat "$W/$row.rc")
    [ "$rc" = 0 ] || bad "$row: exit $rc, want 0"
    printf "$want" >"$W/$row.want"
    if [ -s "$W/o.bin" ] && tail -c +53 "$W/o.bin" | cmp -s - "$W/$row.want"; then
        [ "$rc" = 0 ] && echo "ok   $row"
    else
        bad "$row: records on the wire differ from the expected bytes"
    fi
}

refuse R1  "$T_SEP"  'printf "%s\n" "$M2" | "$ME" sysw pack --no-passphrase --expect mnemonic --out o.bin'
refuse R2  "$T_SEP"  'printf "%s\n" "$MT" | "$ME" sysw pack --no-passphrase --expect mnemonic --out o.bin'
refuse R3  "$T_SEP"  'printf "%s\n" "$MN" | "$ME" sysw pack --no-passphrase --expect mnemonic --out o.bin'
accept C1  "$M"      'printf "%s\n" "$M" | "$ME" sysw pack --no-passphrase --expect mnemonic --out o.bin'
refuse R4  "$T_LF"   '"$ME" sysw pack --no-passphrase --allow-argv-secret --expect mnemonic --out o.bin "$W6"'
refuse R5  "$(t_utf8 text:)" 'printf "text:ff\n" | "$ME" sysw pack --no-passphrase --out o.bin'
refuse R6  "$(t_utf8 pass:)" 'printf "pass:ff\n" | "$ME" sysw pack --no-passphrase --out o.bin'
accept C2  'text:c3a9' 'printf "text:c3a9\n" | "$ME" sysw pack --no-passphrase --out o.bin'
refuse R7  "$T_PAD"  '"$ME" sysw pack --no-passphrase --out o.bin "$D "'
refuse R8  "$T_PAD"  '"$ME" sysw pack --no-passphrase --out o.bin " $D"'
refuse R9  "$T_PAD"  'printf "%s \n" "$D" | "$ME" sysw pack --no-passphrase --out o.bin'
refuse R10 "$T_PAD"  'printf " %s\n" "$D" | "$ME" sysw pack --no-passphrase --out o.bin'
refuse R11 "$T_LF"   'printf "%s\r\r\n" "$D" | "$ME" sysw pack --no-passphrase --out o.bin'
accept R12 "$D"      'printf "%s%s\n" "$BOM" "$D" | "$ME" sysw pack --no-passphrase --out o.bin'
accept R13 "$D"      'printf "%s%s\n" "$BOM" "$D" >in.txt && "$ME" sysw pack --no-passphrase --in in.txt --out o.bin'
accept R14 "$D\ntext:6869" 'printf "%s%s\n" "$BOM" "$D" | "$ME" sysw pack --no-passphrase --out o.bin - text:6869'
refuse R15 "$T_PAD"  'printf "%s \n" "$M2" | "$ME" sysw pack --no-passphrase --out o.bin'
refuse R16 "$T_SEP"  'printf "%s\n" "$M2" | "$ME" sysw pack --passphrase-words 6 --out o.bin'
accept C3  "$D"      'printf "%s\r\n" "$D" | "$ME" sysw pack --no-passphrase --out o.bin'
accept C4  "$D"      '"$ME" sysw pack --no-passphrase --out o.bin "$D"'
refuse C5  --prefix "$T_UNREC_HEAD" 'printf "%s%s%s\n" "$BOM" "$BOM" "$D" | "$ME" sysw pack --no-passphrase --out o.bin'
refuse C6  --prefix "$T_UNREC_HEAD" '"$ME" sysw pack --no-passphrase --out o.bin "$BOM$D"'
accept C7  "$D\n$D"  'printf "%s\r\n\r\n%s\r\n" "$D" "$D" | "$ME" sysw pack --no-passphrase --out o.bin'

[ "${1:-}" = "--cli" ] && { [ $fail = 0 ] && echo "gate --cli: PASS" || echo "gate --cli: FAIL"; exit $fail; }

echo "== B. nextest (whole workspace, once)"
[ -f third_party/seedhammer/go.mod ] || { bad "B: run 'git submodule update --init third_party/seedhammer' first"; exit 1; }
GO=/scratch/code/shibboleth/.toolchain/go/bin
[ -d "$GO" ] && export PATH="$GO:$PATH"
ME_REQUIRE_GO=1 cargo nextest run --locked --no-fail-fast --build-jobs 2 -j 2 >"$LOG/nextest.log" 2>&1
echo "nextest exit $?; summary: $(grep -E '^ *Summary' "$LOG/nextest.log" | tail -1)"
grep -E '^ *(FAIL|SIGSEGV|TIMEOUT|LEAK)' "$LOG/nextest.log" | sed 's/^/    /' && bad "B: failing tests (above)"
for t in \
    admission_refuses_a_record_with_a_line_break \
    admission_refuses_a_padded_record \
    admission_refuses_a_mnemonic_without_single_space_separators \
    admission_refuses_a_text_or_pass_body_that_is_not_utf8 \
    admission_rule_order_is_line_break_padding_separator_utf8 \
    pack_refuses_a_multi_line_descriptor_document \
    pack_refuses_a_padded_card_and_writes_the_plain_one \
    classification_of_non_canonical_spellings_is_unchanged \
    pack_refuses_a_mnemonic_without_single_space_separators \
    pack_refuses_a_wrapped_mnemonic_on_argv \
    pack_refuses_text_and_pass_bodies_that_are_not_utf8 \
    pack_refuses_padded_records_on_every_channel \
    pack_refuses_a_lone_carriage_return \
    pack_strips_one_leading_byte_order_mark_from_a_stream \
    pack_refuses_before_the_passphrase_ceremony_on_a_spelling_fault \
    strip_leading_bom_removes_exactly_one; do
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
