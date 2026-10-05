#!/usr/bin/env bash
#
# run-e2e-otp.sh -- end-to-end tests of scripts/refugium-otp.sh against
# scripts/test/fake_picotool.py (plan E3a, design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md
# section 6.2). No board, no real picotool, no TinyGo: python3, jq, openssl and
# coreutils only, so it runs in CI.
#
# Every case asserts an exit code and a message, and every refusal also asserts
# that the fake's state (OTP rows and flash image) is byte-identical afterwards.
# A failing assertion prints `FAIL [<case>] <what> -- <assertion>`; the case ids
# are the plan's (1 .. 13, 3c, 5d, 5e, 5f, 7b), which is what the mutation table
# in the plan (section 6.3) names.
#
#   scripts/test/run-e2e-otp.sh            # all cases
#   E2E_KEEP=1 scripts/test/run-e2e-otp.sh # keep the scratch directory
#
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
TOOL="$REPO/scripts/refugium-otp.sh"
LIB="$REPO/scripts/lib/otp-read.sh"
STATE_PY="$HERE/fake_otp_state.py"

for t in python3 jq openssl sha256sum; do
  command -v "$t" >/dev/null || { echo "run-e2e-otp: $t not found"; exit 2; }
done

TMP="$(mktemp -d "${E2E_TMPDIR:-${TMPDIR:-/tmp}}/e3a-e2e.XXXXXX")"
[ "${E2E_KEEP:-0}" = "1" ] || trap 'rm -rf "$TMP"' EXIT

PASS=0; FAIL=0; FAILED_CASES=()
ok()  { printf '\033[32m  ok  \033[0m [%s] %s\n' "$1" "$2"; PASS=$((PASS+1)); }
bad() {
  printf '\033[31m FAIL \033[0m [%s] %s -- %s\n' "$1" "$2" "$3"; FAIL=$((FAIL+1)); FAILED_CASES+=("$1")
  printf '%s\n' "$OUT" | tail -15 | sed 's/^/        | /'
}
hdr() { printf '\n\033[1m== %s ==\033[0m\n' "$*"; }

# --- the fake on PATH -------------------------------------------------------
BIN="$TMP/bin"; mkdir -p "$BIN"
cat > "$BIN/picotool" <<EOF
#!/usr/bin/env bash
exec python3 "$HERE/fake_picotool.py" "\$@"
EOF
chmod +x "$BIN/picotool"
export PATH="$BIN:$PATH"
export FAKE_REQUIRE_SER=1
export FAKE_PT_ARGV_LOG="$TMP/argv.log"
: > "$FAKE_PT_ARGV_LOG"
# A case must never inherit a fault mode from the shell that launched it.
unset SUPPRESS_WARNING SUPPRESS_RAW_VALUE COPIES_IGNORED FAIL_SET_AFTER FAIL_READ_AFTER_WRITE \
      DEVICES FAIL_LOAD_VERIFY ERASE_SKIP_BYTE FAIL_GET_AFTER REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY

# --- keys ---------------------------------------------------------------------
KEYS="$TMP/keys"; mkdir -p "$KEYS"
for k in factory my third; do
  openssl ecparam -name secp256k1 -genkey -noout -out "$KEYS/$k.pem" 2>/dev/null
done
khash() { openssl ec -in "$1" -pubout -conv_form uncompressed -outform DER 2>/dev/null \
            | tail -c 64 | sha256sum | cut -d' ' -f1; }
FACT="$(khash "$KEYS/factory.pem")"; MINE="$(khash "$KEYS/my.pem")"; THIRD="$(khash "$KEYS/third.pem")"
SH_HASH="c8314536d6af61ac2e62e5991e3e4711629c54696ba8c4af08965a1d319a473b"
FORK_FP="846aa289f2f317e55ff03f90555132302842cff2f68ee45712834a25d64cabb4"
K0="$KEYS/factory.pem"; K1="$KEYS/my.pem"

RSER="66D3D60FF20ABF2F"; RCHIP="66d3d60ff20abf2f"      # rehearsal Pico 2
TSER="09F50BF63E8D6F46"; TCHIP="09f50bf63e8d6f46"      # retail-shaped (plan case 11 vector)

st() { python3 "$STATE_PY" "$@"; }

# --- base states ----------------------------------------------------------------
# Each base is a directory holding state.json + state.json.flash; a case copies
# the directory, so cases never share state.
mk_rehearsal() { # mk_rehearsal <dir> [chipid serial]
  local d="$1" chip="${2:-$RCHIP}" ser="${3:-$RSER}"
  mkdir -p "$d"
  st new "$d/state.json" --serial "$ser" --chipid "$chip" --flash-size $((4*1024*1024))
  st shape "$d/state.json" rehearsal --slot0 "$FACT" --slot1 "$MINE" --kv 3
}
mk_retail() { # mk_retail <dir> [flash-size]
  local d="$1" size="${2:-$((16*1024*1024))}"
  mkdir -p "$d"
  st new "$d/state.json" --serial "$TSER" --chipid "$TCHIP" --flash-size "$size"
  st shape "$d/state.json" retail --slot0 "$SH_HASH" --slot1 "$FORK_FP" --kv 3
  # A white-label table at row 0x100: VID, manufacturer "SH", product "SHII",
  # SCSI vendor "SH"; USB_BOOT_FLAGS bit 22 plus each entry's valid bit.
  st white-label "$d/state.json" 100 400231 '{"0": 11914, "4": "SH", "5": "SHII", "9": "SH"}'
}
BASE_REH="$TMP/base-rehearsal"; mk_rehearsal "$BASE_REH"
BASE_RET="$TMP/base-retail"; mk_retail "$BASE_RET"

N=0
fresh() { # fresh <base> -> sets S (state path), D (dir)
  N=$((N+1)); D="$TMP/c$N"; cp -r "$1" "$D"; S="$D/state.json"; export FAKE_PT_STATE="$S"
}
snap() { cat "$S" "$S.flash" | sha256sum | cut -d' ' -f1; }
row() { st get-row "$S" "$1"; }

# run <stdin> <cmd...>  -> OUT, RC; the argv log of this run alone -> RUNLOG
run() {
  local input="$1"; shift
  local before; before="$(wc -l < "$FAKE_PT_ARGV_LOG")"
  OUT="$(printf '%s' "$input" | "$@" 2>&1)"; RC=$?
  RUNLOG="$(tail -n +"$((before+1))" "$FAKE_PT_ARGV_LOG")"
}
# expect <case> <desc> <rc> <regex> [same]   ("same": state must be byte-identical to $SNAP0)
expect() {
  local c="$1" desc="$2" want="$3" re="$4" same="${5:-}"
  if [ "$RC" -ne "$want" ]; then bad "$c" "$desc" "exit $RC, want $want"; return; fi
  if [ -n "$re" ] && ! grep -qE -- "$re" <<<"$OUT"; then bad "$c" "$desc" "output did not match /$re/"; return; fi
  if [ "$same" = "same" ] && [ "$(snap)" != "$SNAP0" ]; then bad "$c" "$desc" "state changed on a refusal"; return; fi
  ok "$c" "$desc"
}
no_argv() { # no_argv <case> <desc> <regex-of-argv-that-must-not-appear>
  if grep -qE -- "$3" <<<"$RUNLOG"; then bad "$1" "$2" "argv matching /$3/ was issued"; else ok "$1" "$2"; fi
}
has_argv() {
  if grep -qE -- "$3" <<<"$RUNLOG"; then ok "$1" "$2"; else bad "$1" "$2" "no argv matching /$3/"; fi
}
eqv() { # eqv <case> <desc> <got> <want>
  if [ "$3" = "$4" ]; then ok "$1" "$2"; else OUT="got $3 want $4"; bad "$1" "$2" "got $3, want $4"; fi
}

[ -x "$TOOL" ] || { echo "run-e2e-otp: $TOOL missing or not executable"; FAIL=$((FAIL+1)); FAILED_CASES+=("setup"); }

REH=(--profile rehearsal --ser "$RSER" --rehearsal-key "$K0")
REH1=("${REH[@]}" --rehearsal-slot1-key "$K1")
chk_reh() { run "" "$TOOL" check "${REH1[@]}" --slot1 "${1:-valid}" --disable-otp-boot "${2:-0}" --key-invalid "${3:-0}"; }

# --- a retail tree: the tool copied with a retail-otp.json holding one entry ----
# The tool finds design/hardware/retail-otp.json relative to itself, so a copy
# of the scripts beside a generated entry exercises the real retail path (not
# the _TEST_ONLY override, which refuses every write).
TREE="$TMP/tree"
mk_tree() { # mk_tree <tree> <capture-json>
  local t="$1" cap="$2"
  rm -rf "$t"; mkdir -p "$t/scripts/lib" "$t/design/hardware/captures"
  cp "$TOOL" "$t/scripts/"; cp "$LIB" "$t/scripts/lib/"
  cp "$cap" "$t/design/hardware/captures/fixture-retail.json"
  local sha; sha="$(sha256sum "$t/design/hardware/captures/fixture-retail.json" | cut -d' ' -f1)"
  python3 "$HERE/capture_to_entry.py" "$cap" "$sha" captures/fixture-retail.json > "$t/entry.json"
  jq -n --slurpfile e "$t/entry.json" \
     '{schema: "refugium-retail-otp/1", entries: $e}' > "$t/design/hardware/retail-otp.json"
}
RET=(--profile retail --ser "$TSER")

########################################################################
hdr "capture (feeds the retail fixture)"
fresh "$BASE_RET"; SNAP0="$(snap)"
run "" "$TOOL" capture --ser "$TSER" --out "$TMP/cap-retail.json"
expect 1 "capture of a retail-shaped board (read-only)" 0 "wrote" same
run "" "$TOOL" capture --ser "$TSER" --out "$TMP/cap-retail.json"
expect 1 "capture refuses to overwrite --out" 1 "exists"
mk_tree "$TREE" "$TMP/cap-retail.json"
TTOOL="$TREE/scripts/refugium-otp.sh"

########################################################################
hdr "1. a board in the expected state passes"
fresh "$BASE_RET"; SNAP0="$(snap)"
run "" "$TTOOL" check "${RET[@]}" --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 1 "retail-shaped state with a recorded entry: check PASS" 0 "RESULT: PASS" same
fresh "$BASE_REH"; SNAP0="$(snap)"
chk_reh valid 0 0
expect 1 "rehearsal-shaped state: check PASS" 0 "RESULT: REHEARSAL PROFILE PASS — not a retail check" same
grep -q "CANNOT PROVE" <<<"$OUT" && ok 1 "rehearsal RESULT carries the CANNOT PROVE list" \
  || bad 1 "rehearsal RESULT carries the CANNOT PROVE list" "no CANNOT PROVE"
fresh "$BASE_RET"; SNAP0="$(snap)"
REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY="$TREE/design/hardware/retail-otp.json" \
  run "" "$TOOL" check "${RET[@]}" --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 1 "_TEST_ONLY override: RESULT is TEST ENTRY, never PASS, exit 4" 4 "RESULT: TEST ENTRY — not a retail check" same
grep -q "RESULT: PASS" <<<"$OUT" && bad 1 "_TEST_ONLY never prints RESULT: PASS" "it did" \
  || ok 1 "_TEST_ONLY never prints RESULT: PASS"

########################################################################
hdr "2. each expected-state row wrong, one at a time"
rehrow() { # rehrow <desc> <row-name-regex> <state-edit...>  (@S in the edit = this case's state)
  local desc="$1" re="$2"; shift 2
  fresh "$BASE_REH"; "${@//@S/$S}"; SNAP0="$(snap)"; chk_reh valid 0 0
  expect 2 "rehearsal: $desc" 2 "FAIL +$re" same
}
rehrow "CRIT0 not 0" CRIT0 st set-copies @S 038 8 000001
rehrow "CRIT1 not 0x000001" CRIT1 st set-copies @S 040 8 000005
rehrow "slot 2 not empty" SLOT2 st set-slot @S 2 "$THIRD"
rehrow "slot 3 not empty" SLOT3 st set-slot @S 3 "$THIRD"
rehrow "KEY_INVALID not as stated" BOOT_FLAGS1 st set-copies @S 04b 3 000c03
rehrow "BOOT_FLAGS1 other bit (DOUBLE_TAP)" BOOT_FLAGS1 st set-copies @S 04b 3 080003
rehrow "ENABLE_OTP_BOOT set" BOOT_FLAGS0 st set-copies @S 048 3 004000
rehrow "DISABLE_OTP_BOOT not as stated" BOOT_FLAGS0 st set-copies @S 048 3 002000
rehrow "BOOT_FLAGS0 other bit" BOOT_FLAGS0 st set-copies @S 048 3 000002
rehrow "FLASH_DEVINFO_ENABLE set" FLASH_DEVINFO st set-copies @S 048 3 000020
rehrow "PAGE1_LOCK1 not 0x040404" PAGE1_LOCK1 st set-row @S f83 000000
rehrow "PAGE2_LOCK0 not 0" PAGE2_LOCK0 st set-row @S f84 000001
rehrow "slot 1 is not the rehearsal slot-1 key" SLOT1 st set-slot @S 1 "$THIRD"
fresh "$BASE_REH"; st set-copies "$S" 038 8 000001; st set-row "$S" f83 000000; SNAP0="$(snap)"
chk_reh valid 0 0
expect 2 "two rows wrong: CRIT0 named" 2 "FAIL +CRIT0" same
expect 2 "two rows wrong: PAGE1_LOCK1 named" 2 "FAIL +PAGE1_LOCK1" same
# The joint slot-1 / KEY_VALID table.
fresh "$BASE_REH"; st set-slot "$S" 1 "$(printf '0%.0s' $(seq 1 64))"; SNAP0="$(snap)"
chk_reh empty 0 0
expect 2 "--slot1 empty with KEY_VALID 0x3 refused" 2 "FAIL +SLOT1" same
fresh "$BASE_REH"; st set-copies "$S" 04b 3 000001; SNAP0="$(snap)"
chk_reh valid 0 0
expect 2 "--slot1 valid with the key but KEY_VALID 0x1 refused" 2 "FAIL +SLOT1" same
chk_reh key 0 0
expect 2 "--slot1 key with KEY_VALID 0x1 passes" 0 "RESULT: REHEARSAL PROFILE PASS" same
# Retail cells against the recorded entry.
retrow() { # retrow <desc> <row-name-regex> <state-edit...>  (@S in the edit = this case's state)
  local desc="$1" re="$2"; shift 2
  fresh "$BASE_RET"; "${@//@S/$S}"; SNAP0="$(snap)"
  run "" "$TTOOL" check "${RET[@]}" --slot1 valid --disable-otp-boot 0 --key-invalid 0
  expect 2 "retail: $desc" 2 "FAIL +$re" same
}
retrow "FLASH_DEVINFO differs from the entry" FLASH_DEVINFO st set-ecc @S 054 0b00
retrow "USB_BOOT_FLAGS differs from the entry" USB_BOOT_FLAGS st set-copies @S 059 3 400233
retrow "a white-label string row differs" WHITE_LABEL st set-ecc @S 110 4854
retrow "CRIT1 differs (DEBUG_DISABLE)" CRIT1 st set-copies @S 040 8 000005
retrow "BOOT_FLAGS1 DOUBLE_TAP differs" BOOT_FLAGS1 st set-copies @S 04b 3 000003
retrow "slot 1 is not the fork key" SLOT1 st set-slot @S 1 "$THIRD"
fresh "$BASE_RET"; st set-slot "$S" 0 "$THIRD"; SNAP0="$(snap)"
run "" "$TTOOL" check "${RET[@]}" --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 2 "retail: slot 0 is not SeedHammer's key (identity gate)" 2 "identity" same

########################################################################
hdr "3. unequal copies are refused by check"
odd() { # odd <case> <group> <row0> <ncopies> <copy> <extra-bit> [env...]
  local c="$1" g="$2" r0="$3" n="$4" k="$5" bit="$6"; shift 6
  fresh "$BASE_REH"
  local r=$(( 16#$r0 + k )) v
  v="$(row "$(printf '%03x' "$r")")"
  st set-row "$S" "$(printf '%03x' "$r")" "$(printf '%06x' $(( v | 16#$bit )))"
  SNAP0="$(snap)"
  env "$@" "$TOOL" check "${REH1[@]}" --slot1 valid --disable-otp-boot 0 --key-invalid 0 > "$TMP/out" 2>&1
  RC=$?; OUT="$(cat "$TMP/out")"
  expect "$c" "$g copy $k odd${1:+ ($*)}" 2 "FAIL +$g" same
}
for k in 0 1 2; do
  odd 3 BOOT_FLAGS0 048 3 "$k" 002000
  odd 3 BOOT_FLAGS1 04b 3 "$k" 000100
  odd 3 USB_BOOT_FLAGS 059 3 "$k" 000001
  odd 3 BOOT_FLAGS0 048 3 "$k" 002000 SUPPRESS_WARNING=1
  odd 3 BOOT_FLAGS1 04b 3 "$k" 000100 SUPPRESS_WARNING=1
  odd 3 USB_BOOT_FLAGS 059 3 "$k" 000001 SUPPRESS_WARNING=1
done
for k in 0 7; do
  odd 3 CRIT1 040 8 "$k" 000004
  odd 3 CRIT1 040 8 "$k" 000004 SUPPRESS_WARNING=1
done
odd 3 CRIT0 038 8 0 000001
odd 3 CRIT0 038 8 0 000001 SUPPRESS_WARNING=1
odd 3c BOOT_FLAGS1 04b 3 0 000100 COPIES_IGNORED=1 SUPPRESS_RAW_VALUE=1

########################################################################
hdr "4. writes"
fresh "$BASE_REH"; SNAP0="$(snap)"
run "" "$TOOL" disable-otp-boot "${REH1[@]}"
expect 4 "dry-run leaves the state identical" 0 "dry-run" same
no_argv 4 "dry-run issues no otp set" $'^otp\tset'
run "BURN disable-otp-boot $RSER"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
expect 4 "--execute with the right confirmation writes and passes" 0 "post-write check: PASS"
eqv 4 "all three BOOT_FLAGS0 copies hold DISABLE_OTP_BOOT" "$(row 048) $(row 049) $(row 04a)" "0x002000 0x002000 0x002000"
has_argv 4 "the write is bound with --ser" $'^otp\tset\t-s\tBOOT_FLAGS0.DISABLE_OTP_BOOT\t0x1\t--ser\t'"$RSER"'$'
fresh "$BASE_REH"; SNAP0="$(snap)"
run "BURN disable-otp-boot WRONG"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
expect 4 "wrong confirmation refused" 1 "confirmation" same
run "" "$TOOL" disable-otp-boot --profile rehearsal --ser 0123456789ABCDEF --rehearsal-key "$K0" \
  --rehearsal-slot1-key "$K1" --execute
expect 4 "--ser that matches no board: exit 1 before any write" 1 "no board" same
no_argv 4 "--ser mismatch issues no otp set" $'^otp\tset'
DEVICES=2 run "BURN disable-otp-boot $RSER"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
expect 4 "two boards in BOOTSEL refused" 1 "more than one" same
fresh "$BASE_RET"; SNAP0="$(snap)"
run "BURN disable-otp-boot $TSER"$'\n' "$TOOL" disable-otp-boot "${RET[@]}" --execute
expect 4 "retail with no recorded entries refused" 2 "no recorded retail values" same
REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY="$TREE/design/hardware/retail-otp.json" \
  run "BURN disable-otp-boot $TSER"$'\n' "$TOOL" disable-otp-boot "${RET[@]}" --execute
expect 4 "_TEST_ONLY override refuses a write command" 1 "TEST_ONLY" same
run "BURN disable-otp-boot $TSER"$'\n' "$TTOOL" disable-otp-boot "${RET[@]}" --execute
expect 4 "retail entry: disable-otp-boot writes and passes" 0 "post-write check: PASS"
eqv 4 "retail BOOT_FLAGS0 keeps its recorded bits" "$(row 048) $(row 049) $(row 04a)" "0x002020 0x002020 0x002020"
# Both orders of the two writes.
fresh "$BASE_REH"
run "BURN disable-otp-boot $RSER"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
expect 4 "order A: disable-otp-boot" 0 "post-write check: PASS"
run "BURN invalidate-spare-keys $RSER"$'\n' "$TOOL" invalidate-spare-keys "${REH1[@]}" --execute
expect 4 "order A: then invalidate-spare-keys" 0 "Slots 2 and 3 can never hold a key after this"
chk_reh valid 1 c
expect 4 "order A: final check" 0 "RESULT: REHEARSAL PROFILE PASS"
fresh "$BASE_REH"
run "BURN invalidate-spare-keys $RSER"$'\n' "$TOOL" invalidate-spare-keys "${REH1[@]}" --execute
expect 4 "order B: invalidate-spare-keys" 0 "post-write check: PASS"
eqv 4 "order B: all BOOT_FLAGS1 copies 0x000c03" "$(row 04b) $(row 04c) $(row 04d)" "0x000c03 0x000c03 0x000c03"
run "BURN disable-otp-boot $RSER"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
expect 4 "order B: then disable-otp-boot" 0 "post-write check: PASS"
chk_reh valid 1 c
expect 4 "order B: final check" 0 "RESULT: REHEARSAL PROFILE PASS"
run "BURN disable-otp-boot $RSER"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
expect 4 "re-running a finished write: no write, post-check PASS" 0 "already"
no_argv 4 "re-running a finished write issues no otp set" $'^otp\tset'

########################################################################
hdr "5. heal rule"
heal_ok() { # heal_ok <desc> <cmd> <row0> <v0> <v1> <v2> <want>
  local desc="$1" cmd="$2" r0="$3" want="$7"
  fresh "$BASE_REH"
  st set-row "$S" "$r0" "$4"; st set-row "$S" "$(printf '%03x' $((16#$r0+1)))" "$5"
  st set-row "$S" "$(printf '%03x' $((16#$r0+2)))" "$6"
  run "BURN $cmd $RSER"$'\n' "$TOOL" "$cmd" "${REH1[@]}" --execute
  expect 5 "$desc: healed" 0 "healing an unequal copy"
  grep -q "post-write check: PASS" <<<"$OUT" && ok 5 "$desc: post PASS" || bad 5 "$desc: post PASS" "no PASS"
  eqv 5 "$desc: every copy $want" "$(row "$r0") $(row "$(printf '%03x' $((16#$r0+1)))") $(row "$(printf '%03x' $((16#$r0+2)))")" "$want $want $want"
}
heal_no() { # heal_no <case> <desc> <cmd> <row0> <v0> <v1> <v2> [env...]
  local c="$1" desc="$2" cmd="$3" r0="$4"
  fresh "$BASE_REH"
  st set-row "$S" "$r0" "$5"; st set-row "$S" "$(printf '%03x' $((16#$r0+1)))" "$6"
  st set-row "$S" "$(printf '%03x' $((16#$r0+2)))" "$7"
  shift 7
  SNAP0="$(snap)"
  printf 'BURN %s %s\n' "$cmd" "$RSER" | env "$@" "$TOOL" "$cmd" "${REH1[@]}" --execute > "$TMP/out" 2>&1
  RC=$?; OUT="$(cat "$TMP/out")"
  expect "$c" "$desc" 2 "" same
}
for k in 0 1 2; do
  v=(002000 002000 002000); v[k]=000000
  heal_ok "BOOT_FLAGS0 copy $k missing the bit" disable-otp-boot 048 "${v[@]}" 0x002000
  v=(000000 000000 000000); v[k]=002000
  heal_ok "BOOT_FLAGS0 only copy $k holds the bit" disable-otp-boot 048 "${v[@]}" 0x002000
  v=(000c03 000c03 000c03); v[k]=000003
  heal_ok "BOOT_FLAGS1 copy $k missing KEY_INVALID" invalidate-spare-keys 04b "${v[@]}" 0x000c03
  v=(000003 000003 000003); v[k]=000403
  heal_ok "BOOT_FLAGS1 copy $k holds half of KEY_INVALID" invalidate-spare-keys 04b "${v[@]}" 0x000c03
done
heal_no 5 "a copy holding a bit outside the target refused" disable-otp-boot 048 002000 002002 000000
heal_no 5 "stray KEY_INVALID bit 8 in copy 0 only refused" invalidate-spare-keys 04b 000103 000003 000003
heal_no 5 "stray KEY_INVALID bit 9 in copy 0 only refused" invalidate-spare-keys 04b 000203 000003 000003
heal_no 5 "0x103/0x003/0x003 under COPIES_IGNORED refused" invalidate-spare-keys 04b 000103 000003 000003 COPIES_IGNORED=1
heal_no 5 "0x803/0x003/0x003 under COPIES_IGNORED refused" invalidate-spare-keys 04b 000803 000003 000003 COPIES_IGNORED=1
heal_no 5d "0x903/0x103/0x103 refused" invalidate-spare-keys 04b 000903 000103 000103
heal_no 5e "KEY_INVALID 0x1 in all copies: disable-otp-boot refused" disable-otp-boot 04b 000103 000103 000103
heal_no 5e "DISABLE_OTP_BOOT copies disagreeing: invalidate-spare-keys refused" invalidate-spare-keys 048 002000 000000 000000
heal_no 5e "0x803/0x003/0x003 under COPIES_IGNORED refused" invalidate-spare-keys 04b 000803 000003 000003 COPIES_IGNORED=1
# 5f: ROLLBACK_REQUIRED (bit 11), which the boot ROM sets itself.
fresh "$BASE_REH"; st set-copies "$S" 048 3 000800
run "BURN disable-otp-boot $RSER"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
expect 5f "bit 11 in all copies: disable-otp-boot writes" 0 "post-write check: PASS"
eqv 5f "bit 11 still set in every copy" "$(row 048) $(row 049) $(row 04a)" "0x002800 0x002800 0x002800"
fresh "$BASE_REH"; st set-copies "$S" 048 3 000800
run "BURN inject-copy $RSER"$'\n' "$TOOL" inject-copy --profile rehearsal --ser "$RSER" --rehearsal-key "$K0" \
  --case bf0-copy3 --execute
expect 5f "bit 11 in all copies, then bf0-copy3 injected" 0 "injected"
run "BURN disable-otp-boot $RSER"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
expect 5f "after bf0-copy3: heal" 0 "healing an unequal copy"
eqv 5f "after bf0-copy3: every copy 0x002800" "$(row 048) $(row 049) $(row 04a)" "0x002800 0x002800 0x002800"
heal_no 5f "bit 11 in one copy only: disable-otp-boot refused" disable-otp-boot 048 000800 000000 000000
fresh "$BASE_REH"; st set-copies "$S" 048 3 000800
run "BURN invalidate-spare-keys $RSER"$'\n' "$TOOL" invalidate-spare-keys "${REH1[@]}" --execute
expect 5f "bit 11 in all copies: invalidate-spare-keys writes" 0 "post-write check: PASS"
# A no-write branch whose post-check fails is exit 2, never 3. The fake fails
# every `otp get` after the first P, and P is measured as half of a clean run's
# reads (pre-state and post-check read the same rows).
fresh "$BASE_REH"; st set-copies "$S" 048 3 002000
rm -f "$S.getcount"
run "" "$TOOL" disable-otp-boot "${REH1[@]}" --execute
NGET="$(grep -c $'^otp\tget' <<<"$RUNLOG")"
SNAP0="$(snap)"; rm -f "$S.getcount"
FAIL_GET_AFTER=$((NGET/2)) run "" "$TOOL" disable-otp-boot "${REH1[@]}" --execute
expect 5 "no write issued, post-check fails: exit 2 not 3" 2 "" same
no_argv 5 "no write issued in that branch" $'^otp\tset'

########################################################################
hdr "6. interrupted write"
for n in 1 2; do
  fresh "$BASE_REH"
  FAIL_SET_AFTER=$n run "BURN disable-otp-boot $RSER"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
  expect 6 "FAIL_SET_AFTER=$n: exit 3 with the re-run text" 3 "Re-run this same command once"
  run "BURN disable-otp-boot $RSER"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
  expect 6 "FAIL_SET_AFTER=$n: the re-run heals" 0 "healing an unequal copy"
  grep -q "post-write check: PASS" <<<"$OUT" && ok 6 "FAIL_SET_AFTER=$n: re-run post PASS" \
    || bad 6 "FAIL_SET_AFTER=$n: re-run post PASS" "no PASS"
  fresh "$BASE_REH"
  FAIL_SET_AFTER=$n run "BURN invalidate-spare-keys $RSER"$'\n' "$TOOL" invalidate-spare-keys "${REH1[@]}" --execute
  expect 6 "invalidate FAIL_SET_AFTER=$n: exit 3" 3 "Re-run this same command once"
  run "BURN invalidate-spare-keys $RSER"$'\n' "$TOOL" invalidate-spare-keys "${REH1[@]}" --execute
  expect 6 "invalidate FAIL_SET_AFTER=$n: the re-run heals" 0 "post-write check: PASS"
done
fresh "$BASE_REH"
FAIL_READ_AFTER_WRITE=1 run "BURN disable-otp-boot $RSER"$'\n' "$TOOL" disable-otp-boot "${REH1[@]}" --execute
expect 6 "FAIL_READ_AFTER_WRITE: exit 3" 3 "Re-run this same command once"

########################################################################
hdr "7. profile identity gate"
fresh "$BASE_RET"; SNAP0="$(snap)"
run "BURN erase-range $TSER"$'\n' "$TOOL" erase-range --profile rehearsal --ser "$TSER" --rehearsal-key "$K0" --execute
expect 7 "erase-range --profile rehearsal on a retail-shaped board" 2 "identity" same
no_argv 7 "no erase/save/load argv recorded" $'^(erase|save|load)\t'
run "BURN inject-copy $TSER"$'\n' "$TOOL" inject-copy --profile rehearsal --ser "$TSER" --rehearsal-key "$K0" \
  --case bf0-copy3 --execute
expect 7 "inject-copy on a retail-shaped board" 2 "identity" same
no_argv 7 "no otp set argv recorded" $'^otp\tset'
SHB="$TMP/base-rehearsal-sh"; mk_rehearsal "$SHB" 77c483b745abf55c 77C483B745ABF55C
SHREH=(--profile rehearsal --ser 77C483B745ABF55C --rehearsal-key "$K0" --rehearsal-slot1-key "$K1")
fresh "$SHB"; SNAP0="$(snap)"
run "" "$TOOL" check "${SHREH[@]}" --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 7b "rehearsal on a SeedHammer CHIPID: check" 2 "SeedHammer" same
run "BURN disable-otp-boot 77C483B745ABF55C"$'\n' "$TOOL" disable-otp-boot "${SHREH[@]}" --execute
expect 7b "rehearsal on a SeedHammer CHIPID: disable-otp-boot" 2 "SeedHammer" same
run "BURN invalidate-spare-keys 77C483B745ABF55C"$'\n' "$TOOL" invalidate-spare-keys "${SHREH[@]}" --execute
expect 7b "rehearsal on a SeedHammer CHIPID: invalidate-spare-keys" 2 "SeedHammer" same
no_argv 7b "no otp set argv recorded" $'^otp\tset'

########################################################################
hdr "8. flash range"
fresh "$BASE_REH"
run "BURN erase-range $RSER"$'\n' "$TOOL" erase-range "${REH[@]}" --execute
expect 8 "rehearsal erase-range" 0 "erased and verified"
has_argv 8 "erase argv carries -r and --ser" $'^erase\t-r\t0x10000000\t0x10400000\t--ser\t'"$RSER"'$'
has_argv 8 "verify save argv carries -r and --ser" $'^save\t-r\t0x10000000\t0x10400000\t[^\t]+\\.bin\t--ser\t'"$RSER"'$'
eqv 8 "flash byte 0 erased" "$(st flash-byte "$S" 0)" "ff"
eqv 8 "flash top byte erased" "$(st flash-byte "$S" $((4*1024*1024-1)))" "ff"
fresh "$BASE_REH"; SNAP0="$(snap)"
ERASE_SKIP_BYTE=1000 run "BURN erase-range $RSER"$'\n' "$TOOL" erase-range "${REH[@]}" --execute
expect 8 "a byte left non-0xFF refused" 2 "not erased"
fresh "$BASE_REH"; SNAP0="$(snap)"
run "" "$TOOL" erase-range "${REH[@]}"
expect 8 "rehearsal dry run" 0 "dry-run" same
no_argv 8 "dry run records no erase or load argv" $'^(erase|load)\t'
fresh "$BASE_REH"
run "BURN erase-range $RSER"$'\n' "$TOOL" erase-range "${REH[@]}" --probe-only --execute
expect 8 "--probe-only on a 4 MB board: the probe finds the alias and refuses" 2 "smaller than 16 MB"
fresh "$BASE_RET"
run "BURN erase-range $TSER"$'\n' "$TTOOL" erase-range "${RET[@]}" --execute
expect 8 "retail 16 MB board: probe, erase and verify" 0 "erased and verified"
has_argv 8 "retail probe load argv" $'^load\t-v\t[^\t]+\\.bin\t-o\t0x10000000\t--ser\t'"$TSER"'$'
has_argv 8 "retail erase covers 16 MB" $'^erase\t-r\t0x10000000\t0x11000000\t--ser\t'"$TSER"'$'
fresh "$BASE_RET"; SNAP0="$(snap)"
run "" "$TTOOL" erase-range "${RET[@]}"
expect 8 "retail dry run" 0 "dry-run" same
no_argv 8 "a dry run records no load argv" $'^load\t'
R4="$TMP/base-retail-4mb"; mk_retail "$R4" $((4*1024*1024))
fresh "$R4"
run "BURN erase-range $TSER"$'\n' "$TTOOL" erase-range "${RET[@]}" --execute
expect 8 "retail on 4 MB physical flash: alias probe refuses" 2 "smaller than 16 MB"
no_argv 8 "alias refusal issues no erase" $'^erase\t'
fresh "$BASE_RET"
FAIL_LOAD_VERIFY=1 run "BURN erase-range $TSER"$'\n' "$TTOOL" erase-range "${RET[@]}" --execute
expect 8 "the probe's load failing is condemned" 2 "condemned"
# FLASH_DEVINFO with CS0 = 8 MiB: the boot ROM refuses the probe's load (G3).
R8="$TMP/base-retail-cs8"; mk_retail "$R8"; st set-ecc "$R8/state.json" 054 0b00
mk_tree "$TMP/tree8" "$TMP/cap-retail.json"
fresh "$R8"
run "BURN erase-range $TSER"$'\n' "$TTOOL" erase-range "${RET[@]}" --execute
expect 8 "FLASH_DEVINFO CS0 8 MB under retail: condemned" 2 "condemned"
fresh "$BASE_REH"; SNAP0="$(snap)"
run "" "$TOOL" save-range "${REH[@]}" --out "$TMP/save8.bin"
expect 8 "save-range" 0 "sha256" same
eqv 8 "save-range wrote the whole range" "$(stat -c%s "$TMP/save8.bin")" "$((4*1024*1024))"
run "" "$TOOL" save-range "${REH[@]}" --out "$TMP/save8.txt"
expect 8 "save-range refuses a non-.bin name" 1 "\\.bin"

########################################################################
hdr "9. picotool version and build fingerprint"
fresh "$BASE_REH"; st set "$S" picotool_version 2.2.0-a4; SNAP0="$(snap)"
chk_reh valid 0 0
expect 9 "picotool 2.2.0-a4 refused" 1 "2.3.1" same
no_argv 9 "refused before any device access" $'^(otp\tget|otp\tset|info|erase|save|load)'
fresh "$BASE_REH"; st set "$S" sdk 2.2.0; SNAP0="$(snap)"
chk_reh valid 0 0
expect 9 "2.3.1 with the wrong otp list fingerprint refused" 1 "fingerprint" same
no_argv 9 "refused before any device access" $'^(otp\tget|otp\tset|info|erase|save|load)'

########################################################################
hdr "10. inject-copy"
fresh "$BASE_REH"
run "BURN inject-copy $RSER"$'\n' "$TOOL" inject-copy --profile rehearsal --ser "$RSER" --rehearsal-key "$K0" \
  --case bf0-copy3 --execute
expect 10 "bf0-copy3" 0 "injected"
eqv 10 "bf0-copy3 wrote row 0x04a only" "$(row 048) $(row 049) $(row 04a)" "0x000000 0x000000 0x002000"
chk_reh valid 0 0
expect 10 "check refuses BOOT_FLAGS0 after bf0-copy3 (bench R5)" 2 "FAIL +BOOT_FLAGS0"
fresh "$BASE_REH"
run "BURN inject-copy $RSER"$'\n' "$TOOL" inject-copy --profile rehearsal --ser "$RSER" --rehearsal-key "$K0" \
  --case bf1-copy0 --execute
expect 10 "bf1-copy0" 0 "injected"
eqv 10 "bf1-copy0 wrote row 0x04b only" "$(row 04b) $(row 04c) $(row 04d)" "0x000803 0x000003 0x000003"
chk_reh valid 0 0
expect 10 "check refuses BOOT_FLAGS1 after bf1-copy0 (bench R7)" 2 "FAIL +BOOT_FLAGS1"
fresh "$BASE_REH"; SNAP0="$(snap)"
run "" "$TOOL" inject-copy --profile rehearsal --ser "$RSER" --rehearsal-key "$K0" --case bf1-copy0
expect 10 "inject-copy dry run" 0 "dry-run" same
run "" "$TOOL" inject-copy --profile rehearsal --ser "$RSER" --rehearsal-key "$K0" --case anything --execute
expect 10 "inject-copy takes only the two named cases" 1 "case" same
# The builder mutated to drop `-c` (so the write lands in every copy).
MUT="$TMP/mut-tree"; rm -rf "$MUT"; mkdir -p "$MUT/scripts/lib" "$MUT/design/hardware"
cp "$TOOL" "$MUT/scripts/"; cp "$REPO/design/hardware/retail-otp.json" "$MUT/design/hardware/" 2>/dev/null || true
sed -E 's/^([[:space:]]*)\[ -n "\$c" \] && PT_ARGV\+=\(-c "\$c"\)/\1: # mutated: -c dropped/' "$LIB" > "$MUT/scripts/lib/otp-read.sh"
if cmp -s "$LIB" "$MUT/scripts/lib/otp-read.sh"; then
  OUT="sed did not apply"; bad 10 "builder mutation applied" "the -c line was not found in $LIB"
else
  fresh "$BASE_REH"
  run "BURN inject-copy $RSER"$'\n' "$MUT/scripts/refugium-otp.sh" inject-copy --profile rehearsal --ser "$RSER" \
    --rehearsal-key "$K0" --case bf1-copy0 --execute
  expect 10 "builder without -c: post-injection check exits 3" 3 "stop the rehearsal; do not re-run"
fi

########################################################################
hdr "11. CHIPID test vector and --ser spelling"
fresh "$BASE_RET"; SNAP0="$(snap)"
eqv 11 "CHIPID rows of 0x$TCHIP" "$(row 000) $(row 001) $(row 002) $(row 003)" \
  "$(python3 -c "
import sys; sys.path.insert(0,'$HERE'); from fake_picotool import otp_calculate_ecc as e
c=0x$TCHIP; print(' '.join('0x%06x'%e((c>>(16*i))&0xffff) for i in range(4)))")"
run "" "$TTOOL" check --profile retail --ser "$TSER" --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 11 "--ser $TSER accepted" 0 "RESULT: PASS" same
run "" "$TTOOL" check --profile retail --ser "$TCHIP" --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 11 "lowercase --ser accepted" 0 "RESULT: PASS" same
has_argv 11 "lowercase --ser was uppercased on the wire" $'\t--ser\t'"$TSER"$'(\t|$)'
no_argv 11 "the lowercase spelling never reached picotool" $'\t--ser\t'"$TCHIP"
run "" "$TTOOL" check --profile retail --ser 09F50BF63E8D6F4 --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 11 "a 15-character --ser is a usage error" 1 "16 hex" same

########################################################################
hdr "12. replay of real 2.3.1 transcripts"
TR="$REPO/scripts/test/fixtures/transcripts"
replay() { # replay <dir> -> runs the lib's replay check
  bash -c 'source "$1"; og_replay_dir "$2"' _ "$LIB" "$1"
}
if [ -e "$TR/BENCH_RUN" ]; then
  OUT="$(replay "$TR" 2>&1)"; RC=$?
  expect 12 "bench transcripts parse to the recorded values" 0 "REPLAY PASS"
  for f in r1-capture.json r5-check-refusal.log r7-check-refusal.log; do
    [ -s "$TR/$f" ] && ok 12 "bench fixture $f present" || { OUT=""; bad 12 "bench fixture $f present" "missing or empty"; }
  done
  grep -q "FAIL +BOOT_FLAGS0" "$TR/r5-check-refusal.log" 2>/dev/null && ok 12 "R5 refusal names BOOT_FLAGS0" \
    || { OUT=""; bad 12 "R5 refusal names BOOT_FLAGS0" "not found"; }
  grep -q "FAIL +BOOT_FLAGS1" "$TR/r7-check-refusal.log" 2>/dev/null && ok 12 "R7 refusal names BOOT_FLAGS1" \
    || { OUT=""; bad 12 "R7 refusal names BOOT_FLAGS1" "not found"; }
else
  ok 12 "no BENCH_RUN marker yet: vacuous until the bench-result PR"
fi
# The replay machinery itself, on transcripts the fake produced through capture.
fresh "$BASE_REH"; st set-row "$S" 04c 000103   # an unequal copy, so RAW_VALUE appears
run "" "$TOOL" capture --ser "$RSER" --out "$TMP/cap-reh.json"
expect 12 "capture writes transcripts" 0 "transcripts"
OUT="$(replay "$TMP/cap-reh.json.transcripts" 2>&1)"; RC=$?
expect 12 "replay of capture transcripts parses to the captured values" 0 "REPLAY PASS"
sed -i 's/VALUE 0x000003/VALUE 0x000007/' "$TMP/cap-reh.json.transcripts/0x04b_named.txt"
OUT="$(replay "$TMP/cap-reh.json.transcripts" 2>&1)"; RC=$?
expect 12 "replay catches a transcript that does not match its capture" 1 "MISMATCH"

########################################################################
hdr "13. argv log: --ser on every device call, every shape probed"
# Shapes: what the probe job runs against the real binary.
SHAPES="$(bash -c 'source "$1"; pt_print_argvs ZZPROBE00000000 /nonexistent | while IFS= read -r l; do
  read -r -a a <<<"$l"; pt_shape "${a[@]}"; done' _ "$LIB" | sort -u)"
[ -n "$SHAPES" ] && ok 13 "the builder's print mode produced $(wc -l <<<"$SHAPES") shapes" \
  || { OUT=""; bad 13 "the builder's print mode produced shapes" "none"; }
NOSER=0; UNPROBED=0; TOTAL=0
while IFS= read -r line; do
  [ -n "$line" ] || continue
  IFS=$'\t' read -r -a a <<<"$line"
  TOTAL=$((TOTAL+1))
  case "${a[0]} ${a[1]:-}" in
    "version "*|"otp list") ;;
    "info ")  ;;   # the bare board-count probe (plan 3.1), exempt by design
    *) [[ "$line" == *$'\t--ser\t'* ]] || { NOSER=$((NOSER+1)); printf '    no --ser: %s\n' "$line"; } ;;
  esac
  shape="$(bash -c 'source "$1"; shift; pt_shape "$@"' _ "$LIB" "${a[@]}")"
  grep -qxF -- "$shape" <<<"$SHAPES" || { UNPROBED=$((UNPROBED+1)); printf '    unprobed shape: %s\n' "$shape"; }
done < <(sort -u "$FAKE_PT_ARGV_LOG")
OUT=""
[ "$TOTAL" -gt 0 ] && ok 13 "argv log holds $TOTAL distinct calls" || bad 13 "argv log is not empty" "empty"
[ "$NOSER" -eq 0 ] && ok 13 "every device-touching argv carries --ser" || bad 13 "every device-touching argv carries --ser" "$NOSER without"
[ "$UNPROBED" -eq 0 ] && ok 13 "every argv shape is in the probed set" || bad 13 "every argv shape is in the probed set" "$UNPROBED not probed"

########################################################################
hdr "14. constants, schema, usage"
LIBFP="$(sed -nE 's/^SH2_BOOTKEY_FP="([0-9a-f]{64})".*/\1/p' "$LIB")"
FLFP="$(sed -nE 's/.*SH2_BOOTKEY_FP:=([0-9a-f]{64}).*/\1/p' "$REPO/scripts/sh2-flash")"
eqv 14 "the fork key fingerprint in otp-read.sh equals sh2-flash's" "$LIBFP" "$FLFP"
fresh "$BASE_RET"; SNAP0="$(snap)"
BADJ="$TMP/bad-retail.json"
jq '.entries[0].crit1 = "0x1"' "$TREE/design/hardware/retail-otp.json" > "$BADJ"
REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY="$BADJ" \
  run "" "$TOOL" check "${RET[@]}" --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 14 "a malformed retail entry is exit 1" 1 "schema" same
jq '.entries[0].capture_sha256 = "00"' "$TREE/design/hardware/retail-otp.json" > "$BADJ"
REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY="$BADJ" \
  run "" "$TOOL" check "${RET[@]}" --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 14 "a retail entry whose capture sha256 does not match is exit 1" 1 "sha256" same
run "" "$TOOL" check "${RET[@]}" --rehearsal-key "$K0" --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 14 "--rehearsal-key refused under retail" 1 "rehearsal" same
run "" "$TOOL" check --profile rehearsal --ser "$RSER" --rehearsal-key "$K0" --slot1 valid --disable-otp-boot 0 --key-invalid 0
expect 14 "rehearsal check without --rehearsal-slot1-key is a usage error" 1 "rehearsal-slot1-key" same
run "" "$TOOL" check "${REH1[@]}" --slot1 valid --disable-otp-boot 0
expect 14 "check without --key-invalid is a usage error" 1 "key-invalid" same
fresh "$BASE_REH"; SNAP0="$(snap)"
run "" "$TOOL" check "${REH1[@]}" --slot1 valid --disable-otp-boot 0 --key-invalid 0 --log "$TMP/check.log"
expect 14 "--log keeps the exit status" 0 "RESULT: REHEARSAL PROFILE PASS" same
grep -q "RESULT: REHEARSAL PROFILE PASS" "$TMP/check.log" && ok 14 "--log wrote the transcript" \
  || bad 14 "--log wrote the transcript" "no RESULT line in the log"

########################################################################
hdr "RESULT"
printf '  passed: %d\n  failed: %d\n' "$PASS" "$FAIL"
if [ "$FAIL" -eq 0 ]; then printf '\033[32mALL CHECKS PASSED\033[0m\n'; exit 0; fi
printf '\033[31m%d CHECK(S) FAILED\033[0m (cases: %s)\n' "$FAIL" "$(printf '%s\n' "${FAILED_CASES[@]}" | sort -u | tr '\n' ' ')"
exit 1
