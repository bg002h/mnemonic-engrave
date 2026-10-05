#!/usr/bin/env bash
#
# refugium-otp.sh -- the Refugium expected OTP state on an RP2350 engraver
# (plan E3a, design/IMPLEMENTATION_PLAN_e3a_refugium_otp.md section 3; UI spec
# design/SPEC_ui_refugium_wallet.md section 4.4 in refugium-wallet).
#
#   check   --profile P --ser S [--rehearsal-key K --rehearsal-slot1-key K1]
#           --slot1 empty|key|valid --disable-otp-boot 0|1 --key-invalid 0|c
#   capture --ser S --out <file.json>
#   disable-otp-boot      --profile P --ser S [--rehearsal-key K --rehearsal-slot1-key K1] [--execute]
#   invalidate-spare-keys --profile P --ser S [--rehearsal-key K --rehearsal-slot1-key K1] [--execute]
#   erase-range --profile P --ser S [--rehearsal-key K] [--probe-only] [--execute]
#   save-range  --profile P --ser S [--rehearsal-key K] --out <file.bin>
#   inject-copy --profile rehearsal --ser S --rehearsal-key K --case bf0-copy3|bf1-copy0 [--execute]
#
#   any command: --log <file>   (the tool tees its own transcript; the exit code is kept)
#
# Profiles: `retail` (a SeedHammer II: slot 0 holds SeedHammer's production key,
# recorded values in design/hardware/retail-otp.json) and `rehearsal` (a plain
# Pico 2 that R, scripts/pico2-bootkey-rehearsal.sh, has sealed). The profile is
# never trusted from the command line alone: the identity gate checks slot 0
# (and, for rehearsal, the CHIPID) before anything else touches the board.
#
# Every picotool call is built by scripts/lib/otp-read.sh in picotool's declared
# option order and bound to the board with --ser (plan fact F14/F15).
#
# Exit codes:
#   0  pass
#   1  usage or environment (bad flags, wrong picotool, no board, two boards,
#      wrong confirmation)
#   2  state refused (any FAIL, any unreadable row); nothing was written by OTP
#   3  an `otp set` was issued and something after it failed
#
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib/otp-read.sh
. "$HERE/lib/otp-read.sh"

RERUN_TEXT="The write may be partial. Re-run this same command once: it only adds bits and heals a missing copy. If the re-run exits non-zero for any reason, stop and do not use this board for seeds."
INJECT_TEXT="stop the rehearsal; do not re-run. Record the copies printed above."

say()  { printf '%s\n' "$*"; }
die1() { printf '\nERROR: %s\n' "$*"; exit 1; }
die2() { printf '\nREFUSED: %s\n' "$*"; exit 2; }
die3() { printf '\nWRITE ISSUED, THEN: %s\n\n%s\n' "$*" "$RERUN_TEXT"; exit 3; }
die3i() { printf '\nWRITE ISSUED, THEN: %s\n\ninject-copy: %s\n' "$*" "$INJECT_TEXT"; exit 3; }
hx() { printf '0x%06x' "$1"; }

usage() {
  sed -n '7,16p' "$0" | sed 's/^# \{0,1\}//'
  exit 1
}

# --------------------------------------------------------------------------
# Command line
# --------------------------------------------------------------------------
ORIG_ARGS=("$@")
CMD="${1:-}"; [ $# -gt 0 ] && shift
case "$CMD" in
  check|capture|disable-otp-boot|invalidate-spare-keys|erase-range|save-range|inject-copy) ;;
  ""|-h|--help) usage ;;
  *) die1 "unknown command '$CMD'" ;;
esac

PROFILE=""; SER=""; RKEY=""; RKEY1=""; SLOT1=""; DOB=""; KI=""; EXECUTE=0; OUTF=""
CASE=""; PROBE_ONLY=0; LOG=""
need() { [ $# -ge 2 ] && [ -n "$2" ] || die1 "$1 needs a value"; }
while [ $# -gt 0 ]; do
  case "$1" in
    --profile)             need "$@"; PROFILE="$2"; shift 2 ;;
    --ser)                 need "$@"; SER="$2"; shift 2 ;;
    --rehearsal-key)       need "$@"; RKEY="$2"; shift 2 ;;
    --rehearsal-slot1-key) need "$@"; RKEY1="$2"; shift 2 ;;
    --slot1)               need "$@"; SLOT1="$2"; shift 2 ;;
    --disable-otp-boot)    need "$@"; DOB="$2"; shift 2 ;;
    --key-invalid)         need "$@"; KI="$2"; shift 2 ;;
    --out)                 need "$@"; OUTF="$2"; shift 2 ;;
    --case)                need "$@"; CASE="$2"; shift 2 ;;
    --log)                 need "$@"; LOG="$2"; shift 2 ;;
    --execute)             EXECUTE=1; shift ;;
    --probe-only)          PROBE_ONLY=1; shift ;;
    *) die1 "unknown argument '$1' for $CMD" ;;
  esac
done

# --log: re-run this same command with stdout and stderr through tee, and keep
# its exit code (an operator piping into tee would lose it).
if [ -n "$LOG" ] && [ -z "${REFUGIUM_OTP_LOGGING:-}" ]; then
  REFUGIUM_OTP_LOGGING=1 "$0" "${ORIG_ARGS[@]}" 2>&1 | tee -a "$LOG"
  exit "${PIPESTATUS[0]}"
fi

# Which flags each command takes.
allowed() { # allowed <flag-var-name...>: every flag set must be in the list
  local v ok a
  for v in PROFILE RKEY RKEY1 SLOT1 DOB KI OUTF CASE EXECUTE PROBE_ONLY; do
    case "$v" in EXECUTE|PROBE_ONLY) [ "${!v}" = 1 ] || continue ;; *) [ -n "${!v}" ] || continue ;; esac
    ok=0
    for a in "$@"; do [ "$a" = "$v" ] && ok=1; done
    [ "$ok" = 1 ] || die1 "$(flagname "$v") is not taken by $CMD"
  done
}
flagname() {
  case "$1" in
    PROFILE) echo --profile ;; RKEY) echo --rehearsal-key ;; RKEY1) echo --rehearsal-slot1-key ;;
    SLOT1) echo --slot1 ;; DOB) echo --disable-otp-boot ;; KI) echo --key-invalid ;; OUTF) echo --out ;;
    CASE) echo --case ;; EXECUTE) echo --execute ;; PROBE_ONLY) echo --probe-only ;;
  esac
}
case "$CMD" in
  check)                 allowed PROFILE RKEY RKEY1 SLOT1 DOB KI ;;
  capture)               allowed OUTF ;;
  disable-otp-boot|invalidate-spare-keys) allowed PROFILE RKEY RKEY1 EXECUTE ;;
  erase-range)           allowed PROFILE RKEY PROBE_ONLY EXECUTE ;;
  save-range)            allowed PROFILE RKEY OUTF ;;
  inject-copy)           allowed PROFILE RKEY CASE EXECUTE ;;
esac

[ -n "$SER" ] || die1 "--ser is required: the board's CHIPID as 16 hex characters (CHIPID3..0)"
[[ "$SER" =~ ^[0-9A-Fa-f]{16}$ ]] || die1 "--ser must be exactly 16 hex characters (CHIPID3..0), got '$SER'"
SER="${SER^^}"

# Commands that judge slot 1 need the slot-1 key under rehearsal.
JUDGES_SLOT1=0
case "$CMD" in check|disable-otp-boot|invalidate-spare-keys) JUDGES_SLOT1=1 ;; esac

if [ "$CMD" != capture ]; then
  case "$PROFILE" in
    retail)
      [ -z "$RKEY" ] && [ -z "$RKEY1" ] \
        || die1 "--rehearsal-key and --rehearsal-slot1-key are refused under --profile retail" ;;
    rehearsal)
      [ -n "$RKEY" ] || die1 "--profile rehearsal needs --rehearsal-key (slot 0's key: R's rehearsal-work/factory-key.pem)"
      if [ "$JUDGES_SLOT1" = 1 ]; then
        [ -n "$RKEY1" ] || die1 "--profile rehearsal needs --rehearsal-slot1-key for $CMD (slot 1's key: R's rehearsal-work/my-key.pem)"
      fi ;;
    "") die1 "--profile retail|rehearsal is required for $CMD" ;;
    *) die1 "--profile must be retail or rehearsal, got '$PROFILE'" ;;
  esac
fi

case "$CMD" in
  check)
    case "$SLOT1" in empty|key|valid) ;; "") die1 "check needs --slot1 empty|key|valid" ;; *) die1 "--slot1 must be empty, key or valid" ;; esac
    case "$DOB" in 0|1) ;; "") die1 "check needs --disable-otp-boot 0|1" ;; *) die1 "--disable-otp-boot must be 0 or 1" ;; esac
    case "$KI" in 0) ;; c|C) KI=c ;; "") die1 "check needs --key-invalid 0|c" ;; *) die1 "--key-invalid must be 0 or c" ;; esac ;;
  capture)
    [ -n "$OUTF" ] || die1 "capture needs --out <file.json>"
    [ ! -e "$OUTF" ] || die1 "--out $OUTF exists; capture never overwrites"
    [ ! -e "$OUTF.transcripts" ] || die1 "$OUTF.transcripts exists; capture never overwrites" ;;
  save-range)
    [ -n "$OUTF" ] || die1 "save-range needs --out <file.bin>"
    [[ "$OUTF" == *.bin ]] || die1 "--out must end in .bin (picotool saves raw bytes only to a .bin name)"
    [ ! -e "$OUTF" ] || die1 "--out $OUTF exists; save-range never overwrites" ;;
  inject-copy)
    [ "$PROFILE" = rehearsal ] || die1 "inject-copy runs only under --profile rehearsal"
    case "$CASE" in bf0-copy3|bf1-copy0) ;; "") die1 "inject-copy needs --case bf0-copy3|bf1-copy0" ;;
      *) die1 "inject-copy takes only --case bf0-copy3 or --case bf1-copy0, got '$CASE'" ;; esac ;;
esac

# The retail values file. Only REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY overrides the
# path, and with it set nothing may write and check can never say PASS.
RETAIL_JSON="$HERE/../design/hardware/retail-otp.json"
TEST_ONLY=0
if [ -n "${REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY:-}" ]; then
  TEST_ONLY=1
  RETAIL_JSON="$REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY"
  case "$CMD" in
    disable-otp-boot|invalidate-spare-keys|erase-range|inject-copy)
      die1 "REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY is set: every write command is refused (test entries only)" ;;
  esac
  [ "$EXECUTE" = 0 ] || die1 "REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY is set: --execute is refused"
  say "NOTE: REFUGIUM_OTP_RETAIL_JSON_TEST_ONLY=$RETAIL_JSON -- test entries; RESULT can never be PASS"
fi

for t in picotool jq openssl sha256sum stat tr od; do
  command -v "$t" >/dev/null || die1 "$t not found -- run inside 'nix develop .#otp'"
done

# --------------------------------------------------------------------------
# picotool: exactly the pinned version, built with the pinned SDK
# (design/PICOTOOL_PIN.md). Checked before any device access.
# --------------------------------------------------------------------------
pt_build_version; pt_exec
PTV="$(printf '%s' "$PT_OUT" | tr -d '[:space:]')"
[ "$PT_RC" -eq 0 ] && [ "$PTV" = "$PICOTOOL_PIN" ] \
  || die1 "picotool '$PTV' is not the pinned $PICOTOOL_PIN (design/PICOTOOL_PIN.md); run inside 'nix develop .#otp'"
pt_build_otp_list MAC0; pt_exec
FP_MAC0=0; [[ "$PT_OUT" == *"ROW 0x0062: OTP_DATA_MAC0"* ]] && FP_MAC0=1
pt_build_otp_list BOOT_FLAGS0.DISABLE_BOOTSEL_EXEC2; pt_exec
FP_EXEC2=0; [[ "$PT_OUT" == *DISABLE_BOOTSEL_EXEC2* ]] && FP_EXEC2=1
[ "$FP_MAC0" = 1 ] && [ "$FP_EXEC2" = 0 ] \
  || die1 "picotool $PTV has the wrong otp list fingerprint (MAC0 present=$FP_MAC0, DISABLE_BOOTSEL_EXEC2 present=$FP_EXEC2): it was not built against pico-sdk 2.3.1"
say "picotool $PTV (pico-sdk 2.3.1 fingerprint: MAC0 present, DISABLE_BOOTSEL_EXEC2 absent)"

# --------------------------------------------------------------------------
# Retail entries (plan 3.2), schema-checked before use. Loaded for every
# retail command that judges OTP state.
# --------------------------------------------------------------------------
NENT=0
declare -a E_ID E_CRIT0 E_CRIT1 E_BF0 E_BF1 E_FD E_UBF E_WLA E_WL
HEX6='^0x[0-9a-f]{6}$'
load_retail() {
  local f="$RETAIL_JSON" i fld v dir cap sha got rows k
  local -A EF=()
  [ -f "$f" ] || die1 "schema: retail values file $f is missing"
  jq -e . "$f" >/dev/null 2>&1 || die1 "schema: $f is not valid JSON"
  [ "$(jq -r '.schema // ""' "$f")" = "refugium-retail-otp/1" ] \
    || die1 "schema: $f does not declare schema refugium-retail-otp/1"
  [ "$(jq -r '.entries | type' "$f")" = array ] || die1 "schema: $f has no entries array"
  NENT="$(jq '.entries | length' "$f")"
  dir="$(cd "$(dirname "$f")" && pwd)"
  for ((i = 0; i < NENT; i++)); do
    for fld in id capture capture_sha256 date history crit0 crit1 boot_flags0 boot_flags1 \
               flash_devinfo usb_boot_flags usb_white_label_addr; do
      v="$(jq -r --argjson i "$i" --arg k "$fld" '.entries[$i][$k] | if type == "string" then . else "<not a string>" end' "$f")"
      case "$fld" in
        capture_sha256) [[ "$v" =~ ^[0-9a-f]{64}$ ]] || die1 "schema: entry $i capture_sha256 '$v' is not 64 lowercase hex" ;;
        capture) [[ "$v" =~ ^captures/[A-Za-z0-9._-]+\.json$ ]] && [[ "$v" != *..* ]] \
                   || die1 "schema: entry $i capture '$v' must be captures/<name>.json beside $f" ;;
        id|date|history) [ -n "$v" ] && [ "$v" != "<not a string>" ] || die1 "schema: entry $i $fld missing" ;;
        *) [[ "$v" =~ $HEX6 ]] || die1 "schema: entry $i $fld '$v' is not 0x + 6 lowercase hex digits" ;;
      esac
      EF[$fld]="$v"
    done
    # Cells that are never taken from an entry must be clear in it.
    (( (16#${EF[boot_flags0]#0x} & 0x6800) == 0 )) \
      || die1 "schema: entry $i boot_flags0 carries DISABLE_OTP_BOOT, ENABLE_OTP_BOOT or bit 11 (never taken from an entry)"
    (( (16#${EF[boot_flags1]#0x} & 0x0f0f) == 0 )) \
      || die1 "schema: entry $i boot_flags1 carries KEY_VALID or KEY_INVALID (never taken from an entry)"
    rows="$(jq -r --argjson i "$i" '.entries[$i].white_label_rows | if type == "object" then (to_entries[] | "\(.key)=\(.value)") else "<bad>" end' "$f")"
    [ "$rows" != "<bad>" ] || die1 "schema: entry $i white_label_rows is not an object"
    for k in $rows; do
      [[ "${k%%=*}" =~ ^0x[0-9a-f]{3}$ ]] && [[ "${k#*=}" =~ $HEX6 ]] \
        || die1 "schema: entry $i white_label_rows item '$k' is not 0xRRR=0xVVVVVV"
    done
    cap="$dir/${EF[capture]}"
    [ -f "$cap" ] || die1 "schema: entry $i capture file $cap is missing"
    sha="${EF[capture_sha256]}"
    got="$(sha256sum "$cap" | cut -d' ' -f1)"
    [ "$got" = "$sha" ] || die1 "schema: entry $i capture ${EF[capture]} sha256 is $got, the entry records $sha"
    E_ID[i]="${EF[id]}"; E_CRIT0[i]=$((16#${EF[crit0]#0x})); E_CRIT1[i]=$((16#${EF[crit1]#0x}))
    E_BF0[i]=$((16#${EF[boot_flags0]#0x})); E_BF1[i]=$((16#${EF[boot_flags1]#0x}))
    E_FD[i]=$((16#${EF[flash_devinfo]#0x})); E_UBF[i]=$((16#${EF[usb_boot_flags]#0x}))
    E_WLA[i]=$((16#${EF[usb_white_label_addr]#0x})); E_WL[i]="$(printf '%s ' $rows)"
  done
}
if [ "$PROFILE" = retail ]; then
  case "$CMD" in
    check|disable-otp-boot|invalidate-spare-keys)
      load_retail
      [ "$NENT" -gt 0 ] || die2 "no recorded retail values; run \`capture\` on a retail unit (H0)"
      say "retail values: $NENT recorded entr$( [ "$NENT" = 1 ] && echo y || echo ies) in $RETAIL_JSON" ;;
  esac
fi

# --------------------------------------------------------------------------
# Exactly one RP2350 in BOOTSEL, and it is --ser.
# --------------------------------------------------------------------------
# shellcheck disable=SC2034  # read by the library's readers
PT_SER="$SER"
# Bare `info` is the one call made without --ser on purpose: it counts boards.
# Its exit code cannot tell one board from two (F23), the text can.
pt_build_info ""; pt_exec
if [[ "$PT_OUT" == *"Multiple RP-series devices in BOOTSEL mode found:"* ]]; then
  die1 "more than one RP-series board is in BOOTSEL; unplug every board except $SER"
fi
if [ "$PT_RC" -ne 0 ] || [[ "$PT_OUT" == *"No accessible"* ]]; then
  die1 "no board in BOOTSEL (picotool info exited $PT_RC): hold BOOTSEL while connecting USB"
fi
pt_build_info "$SER"; pt_exec
if [ "$PT_RC" -ne 0 ]; then
  die1 "no board in BOOTSEL with serial $SER (picotool exited $PT_RC). --ser is the CHIPID, CHIPID3..0 in uppercase."
fi

# --------------------------------------------------------------------------
# Identity gate (plan 3.1), before anything else touches the device.
# --------------------------------------------------------------------------
RKEY_HASH=""; RKEY1_HASH=""
if [ "$PROFILE" = rehearsal ]; then
  try_key_hash "$RKEY" || die1 "--rehearsal-key: $ERR"
  RKEY_HASH="$KEY_HASH"
  say "rehearsal key (slot 0): $RKEY sha256(X||Y)=$RKEY_HASH"
  if [ -n "$RKEY1" ]; then
    try_key_hash "$RKEY1" || die1 "--rehearsal-slot1-key: $ERR"
    RKEY1_HASH="$KEY_HASH"
    say "rehearsal slot-1 key:      $RKEY1 sha256(X||Y)=$RKEY1_HASH"
  fi
fi

# try_gate -> 0, or 1 with ERR and GATE_CODE (1: identity of the board itself,
# 2: the profile does not hold).
GATE_CODE=0; SLOT0_HEX=""
try_gate() {
  local s
  GATE_CODE=1
  try_chipid || return 1
  [ "$CHIPID_SER" = "$SER" ] || { ERR="the board answering to --ser $SER reports CHIPID $CHIPID_SER"; return 1; }
  [ "$CMD" = capture ] && return 0
  GATE_CODE=2
  try_read_slot 0 || return 1
  SLOT0_HEX="$SLOT_HEX"
  if [ "$PROFILE" = retail ]; then
    [ "$SLOT0_HEX" = "$SH_SIGNKEY_HASH" ] || { ERR="identity gate: slot 0 holds $SLOT0_HEX, not SeedHammer's production key; this is not a retail SeedHammer II"; return 1; }
  else
    for s in "${SEEDHAMMER_SERS[@]}"; do
      [ "$s" != "$CHIPID_SER" ] || { ERR="identity gate: CHIPID $CHIPID_SER is a SeedHammer II (design/HARDWARE_INVENTORY.md); the rehearsal profile refuses it"; return 1; }
    done
    [ "$SLOT0_HEX" != "$SH_SIGNKEY_HASH" ] || { ERR="identity gate: slot 0 holds SeedHammer's production key; this is a SeedHammer II, not a rehearsal board"; return 1; }
    [ "$SLOT0_HEX" = "$RKEY_HASH" ] || { ERR="identity gate: slot 0 holds $SLOT0_HEX, not the hash of --rehearsal-key ($RKEY_HASH)"; return 1; }
  fi
  return 0
}
gate_or_exit() {
  try_gate && return 0
  [ "$GATE_CODE" = 1 ] && die1 "$ERR"
  die2 "$ERR"
}

# --------------------------------------------------------------------------
# Reading the board
# --------------------------------------------------------------------------
declare -A G_OK G_ERR G_VOTE G_WARN G_RAW G_COPIES
declare -A SLOTV SLOTERR PLV PLERR RAWV RAWERR

# read_group <NAME> <row0> <ncopies>: the named read (vote, WARNING, RAW_VALUE),
# the `-c 1` read of the first row and the bare reads of the unnamed copies.
read_group() {
  local nm="$1" r0="$2" n="$3" i copies
  G_OK[$nm]=0; G_ERR[$nm]=""; G_VOTE[$nm]=""; G_WARN[$nm]=0; G_RAW[$nm]=""; G_COPIES[$nm]=""
  try_og_row "" "$r0" "OTP_DATA_$nm" || { G_ERR[$nm]="$ERR"; return 1; }
  G_VOTE[$nm]="$OG_INT"; G_WARN[$nm]="$OG_WARN"; G_RAW[$nm]="$OG_RAW"
  try_og_row "-c 1" "$r0" "OTP_DATA_$nm" || { G_ERR[$nm]="$ERR"; return 1; }
  # G1: -c 1 never prints RAW_VALUE or a WARNING; if it did, it was not a
  # one-copy read and its value is not copy 0.
  [ "$OG_WARN" = 0 ] && [ -z "$OG_RAW" ] || { G_ERR[$nm]="the -c 1 read of $(printf '0x%03x' "$r0") printed a WARNING or RAW_VALUE"; return 1; }
  copies="$OG_INT"
  for ((i = 1; i < n; i++)); do
    try_og_row "" $((r0 + i)) "" || { G_ERR[$nm]="$ERR"; return 1; }
    [ "$OG_WARN" = 0 ] && [ -z "$OG_RAW" ] || { G_ERR[$nm]="the bare read of $(printf '0x%03x' $((r0 + i))) printed a WARNING or RAW_VALUE"; return 1; }
    copies+=" $OG_INT"
  done
  G_COPIES[$nm]="$copies"; G_OK[$nm]=1
}

copies_hex() { local v o=""; for v in $1; do o+="$(hx "$v")/"; done; printf '%s' "${o%/}"; }
all_equal() { local first="" v; for v in $1; do [ -n "$first" ] || first="$v"; [ "$v" = "$first" ] || return 1; done; return 0; }
raw_matches() { # raw_matches <NAME>: RAW_VALUE's list equals the per-copy reads, copy for copy
  local -a rv cv; local i
  read -r -a rv <<<"${G_RAW[$1]}"; read -r -a cv <<<"${G_COPIES[$1]}"
  [ "${#rv[@]}" -eq "${#cv[@]}" ] || return 1
  for ((i = 0; i < ${#cv[@]}; i++)); do [ $((rv[i])) -eq "${cv[i]}" ] || return 1; done
  return 0
}

# copy_rule <NAME> -> 0, or 1 with WHY (plan 3.2: (a) per-copy reads equal,
# (b) no WARNING, (d) RAW_VALUE equals the per-copy reads; (c) is the caller's).
copy_rule() {
  local nm="$1"
  WHY=""
  [ "${G_OK[$nm]}" = 1 ] || { WHY="unreadable: ${G_ERR[$nm]}"; return 1; }
  all_equal "${G_COPIES[$nm]}" || { WHY="copies differ: $(copies_hex "${G_COPIES[$nm]}") (copy 0 from -c 1)"; return 1; }
  [ "${G_WARN[$nm]}" = 0 ] || { WHY="picotool printed a WARNING for the named read (copies $(copies_hex "${G_COPIES[$nm]}"))"; return 1; }
  if [ -n "${G_RAW[$nm]}" ]; then
    raw_matches "$nm" || { WHY="RAW_VALUE=${G_RAW[$nm]// /;} disagrees with the per-copy reads $(copies_hex "${G_COPIES[$nm]}")"; return 1; }
  fi
  return 0
}

# copies_equal <NAME>: the write branches' equality (plan 3.3): no WARNING, no
# RAW_VALUE line, and all per-copy reads agree.
copies_equal() {
  local nm="$1"
  [ "${G_OK[$nm]}" = 1 ] || return 1
  [ "${G_WARN[$nm]}" = 0 ] && [ -z "${G_RAW[$nm]}" ] || return 1
  all_equal "${G_COPIES[$nm]}"
}

read_raw() { # read_raw <row> <want-name>: RAWV[row] (24-bit raw, `-r`), or RAWERR[row]
  local r="$1" key; key="$(printf '0x%03x' "$1")"
  try_og_row "-r" "$r" "$2" || { RAWERR[$key]="$ERR"; return 1; }
  RAWV[$key]="$OG_INT"
}

read_board() {
  local n l r k i
  read_group CRIT0 0x038 8
  read_group CRIT1 0x040 8
  read_group BOOT_FLAGS1 0x04b 3
  read_group BOOT_FLAGS0 0x048 3
  read_group USB_BOOT_FLAGS 0x059 3
  for n in 1 2 3; do
    if try_read_slot "$n"; then SLOTV[$n]="$SLOT_HEX"; SLOTERR[$n]=""; else SLOTV[$n]=""; SLOTERR[$n]="$ERR"; fi
  done
  for l in PAGE1_LOCK0:0xf82 PAGE1_LOCK1:0xf83 PAGE2_LOCK0:0xf84 PAGE2_LOCK1:0xf85; do
    if try_og_row "" "${l#*:}" "OTP_DATA_${l%%:*}"; then PLV[${l%%:*}]="$OG_INT"; PLERR[${l%%:*}]=""
    else PLV[${l%%:*}]=""; PLERR[${l%%:*}]="$ERR"; fi
  done
  if [ "$PROFILE" = retail ]; then
    read_raw 0x054 OTP_DATA_FLASH_DEVINFO
    read_raw 0x05c OTP_DATA_USB_WHITE_LABEL_ADDR
    for ((i = 0; i < NENT; i++)); do
      for k in ${E_WL[i]}; do
        r="${k%%=*}"
        [ -n "${RAWV[$r]:-}${RAWERR[$r]:-}" ] || read_raw "$((16#${r#0x}))" "*"
      done
    done
  fi
}

# --------------------------------------------------------------------------
# Judging (plan 3.2)
# --------------------------------------------------------------------------
declare -a ROW_NAMES ROW_RESULTS
declare -A ROW_SEEN
NFAIL=0
TOOL_ERR=""
rowres() { # rowres <NAME> PASS|FAIL <detail>
  if [ -n "${ROW_SEEN[$1]:-}" ]; then TOOL_ERR="row $1 compared twice"; fi
  ROW_SEEN[$1]=1; ROW_NAMES+=("$1"); ROW_RESULTS+=("$2")
  [ "$2" = PASS ] || NFAIL=$((NFAIL + 1))
  printf '  %-4s  %-22s %s\n' "$2" "$1" "$3"
}

# entry_matches <i>: every recorded cell of entry i equals the board.
entry_matches() {
  local i="$1" k r
  [ "${G_OK[CRIT0]}" = 1 ] && [ "${G_VOTE[CRIT0]}" -eq "${E_CRIT0[i]}" ] || return 1
  [ "${G_OK[CRIT1]}" = 1 ] && [ "${G_VOTE[CRIT1]}" -eq "${E_CRIT1[i]}" ] || return 1
  [ "${G_OK[BOOT_FLAGS0]}" = 1 ] && [ $(( G_VOTE[BOOT_FLAGS0] & ~0x6800 )) -eq "${E_BF0[i]}" ] || return 1
  [ "${G_OK[BOOT_FLAGS1]}" = 1 ] && [ $(( G_VOTE[BOOT_FLAGS1] & ~0x0f0f )) -eq "${E_BF1[i]}" ] || return 1
  [ "${G_OK[USB_BOOT_FLAGS]}" = 1 ] && [ "${G_VOTE[USB_BOOT_FLAGS]}" -eq "${E_UBF[i]}" ] || return 1
  [ -n "${RAWV[0x054]:-}" ] && [ "${RAWV[0x054]}" -eq "${E_FD[i]}" ] || return 1
  [ -n "${RAWV[0x05c]:-}" ] && [ "${RAWV[0x05c]}" -eq "${E_WLA[i]}" ] || return 1
  for k in ${E_WL[i]}; do
    r="${k%%=*}"
    [ -n "${RAWV[$r]:-}" ] && [ "${RAWV[$r]}" -eq $((16#${k#*=0x})) ] || return 1
  done
  return 0
}

ENT=-1
choose_entry() {
  local i
  ENT=-1
  for ((i = 0; i < NENT; i++)); do
    if entry_matches "$i"; then ENT=$i; break; fi
  done
  if [ "$ENT" -lt 0 ]; then
    ENT=0
    say "  no recorded entry matches every recorded cell; rows below are compared against entry ${E_ID[0]}"
  else
    say "  retail entry: ${E_ID[ENT]} (every recorded cell matches)"
  fi
}

group_line() { # group_line <NAME> <expected-ok 0|1> <detail>: copy rule then the value test
  local nm="$1"
  if ! copy_rule "$nm"; then rowres "$nm" FAIL "$WHY"; return; fi
  if [ "$2" = 1 ]; then rowres "$nm" PASS "$3"; else rowres "$nm" FAIL "$3"; fi
}

# judge <slot1 empty|key|valid> <dob 0|1|-> <ki 0|c|-> -> prints the rows,
# sets NFAIL; RESULT_OK=1 only if NFAIL is 0 and the compared set equals the
# required set. "-" marks the target row of a write (judged by its branch).
RESULT_OK=0
judge() {
  local s1="$1" dob="$2" ki="$3" v want kv want_kv want_slot k r name base ok s
  local -a req=()
  local zero64; zero64="$(printf '0%.0s' $(seq 1 64))"
  ROW_NAMES=(); ROW_RESULTS=(); ROW_SEEN=(); NFAIL=0; TOOL_ERR=""; RESULT_OK=0
  [ "$PROFILE" = retail ] && choose_entry

  rowres CHIPID PASS "$CHIPID_SER (= --ser)"
  rowres SLOT0 PASS "identity gate: $( [ "$PROFILE" = retail ] && echo "SeedHammer's production key" || echo "--rehearsal-key's hash")"

  # CRIT0, CRIT1 x8
  if [ "$PROFILE" = retail ]; then
    v="${G_VOTE[CRIT0]:-0}"; group_line CRIT0 $(( v == E_CRIT0[ENT] )) "vote $(hx "$v"), entry $(hx "${E_CRIT0[ENT]}")"
    v="${G_VOTE[CRIT1]:-0}"; group_line CRIT1 $(( v == E_CRIT1[ENT] )) "vote $(hx "$v"), entry $(hx "${E_CRIT1[ENT]}")"
  else
    v="${G_VOTE[CRIT0]:-0}"; group_line CRIT0 $(( v == 0 )) "vote $(hx "$v"), want 0x000000 in all 8"
    v="${G_VOTE[CRIT1]:-0}"; group_line CRIT1 $(( v == 1 )) "vote $(hx "$v"), want 0x000001 in all 8"
  fi

  # Slot 1 and KEY_VALID, judged together.
  if [ "$PROFILE" = retail ]; then want_slot="$SH2_BOOTKEY_FP"; else want_slot="$RKEY1_HASH"; fi
  case "$s1" in empty) want_kv=1 ;; key) want_kv=1 ;; valid) want_kv=3 ;; esac
  # KEY_VALID comes from BOOT_FLAGS1's vote once its copy rule holds. When
  # BOOT_FLAGS1 is the target of invalidate-spare-keys (ki "-"), its copies are
  # judged by the write's branch rule instead, which bounds every copy's
  # KEY_VALID bits by E (0x3); the vote is then used as read.
  if [ -n "${SLOTERR[1]}" ]; then rowres SLOT1 FAIL "unreadable: ${SLOTERR[1]}"
  elif [ "$ki" != "-" ] && ! copy_rule BOOT_FLAGS1; then rowres SLOT1 FAIL "KEY_VALID unknown: BOOT_FLAGS1 $WHY"
  elif [ "${G_OK[BOOT_FLAGS1]}" != 1 ]; then rowres SLOT1 FAIL "KEY_VALID unknown: BOOT_FLAGS1 unreadable: ${G_ERR[BOOT_FLAGS1]}"
  else
    kv=$(( G_VOTE[BOOT_FLAGS1] & 0xf ))
    if [ "$s1" = empty ]; then
      [ "${SLOTV[1]}" = "$zero64" ] && [ "$kv" -eq "$want_kv" ] \
        && rowres SLOT1 PASS "empty, KEY_VALID 0x$kv" \
        || rowres SLOT1 FAIL "--slot1 empty wants slot 1 all zero and KEY_VALID 0x1; slot 1 ${SLOTV[1]}, KEY_VALID 0x$(printf '%x' "$kv")"
    else
      [ "${SLOTV[1]}" = "$want_slot" ] && [ "$kv" -eq "$want_kv" ] \
        && rowres SLOT1 PASS "fork key, KEY_VALID 0x$kv" \
        || rowres SLOT1 FAIL "--slot1 $s1 wants slot 1 $want_slot and KEY_VALID 0x$want_kv; slot 1 ${SLOTV[1]}, KEY_VALID 0x$(printf '%x' "$kv")"
    fi
  fi
  for s in 2 3; do
    if [ -n "${SLOTERR[$s]}" ]; then rowres "SLOT$s" FAIL "unreadable: ${SLOTERR[$s]}"
    elif [ "${SLOTV[$s]}" = "$zero64" ]; then rowres "SLOT$s" PASS "empty"
    else rowres "SLOT$s" FAIL "not empty: ${SLOTV[$s]}"; fi
  done

  # BOOT_FLAGS1 x3: KEY_INVALID as stated; every bit outside KEY_VALID and
  # KEY_INVALID equals the entry (retail) or 0 (rehearsal).
  if [ "$ki" != "-" ]; then
    if [ "$ki" = c ]; then want=12; else want=0; fi
    base=0; [ "$PROFILE" = retail ] && base="${E_BF1[ENT]}"
    v="${G_VOTE[BOOT_FLAGS1]:-0}"
    group_line BOOT_FLAGS1 $(( ((v >> 8) & 0xf) == want && (v & ~0x0f0f) == base )) \
      "vote $(hx "$v"): KEY_INVALID 0x$(printf '%x' $(( (v >> 8) & 0xf ))) (want 0x$(printf '%x' "$want")), other bits $(hx $(( v & ~0x0f0f & 0xffffff ))) (want $(hx "$base"))"
  fi

  # BOOT_FLAGS0 x3: ENABLE_OTP_BOOT 0; DISABLE_OTP_BOOT = flag; bit 11 either;
  # every other bit the entry's (retail) or 0 (rehearsal).
  if [ "$dob" != "-" ]; then
    base=0; [ "$PROFILE" = retail ] && base="${E_BF0[ENT]}"
    want=$(( base | (dob << 13) ))
    v="${G_VOTE[BOOT_FLAGS0]:-0}"
    group_line BOOT_FLAGS0 $(( (v & ~0x800) == want )) \
      "vote $(hx "$v"): want $(hx "$want") (bit 11 either value)"
  fi

  # FLASH_DEVINFO
  if [ "$PROFILE" = retail ]; then
    if [ -n "${RAWERR[0x054]:-}" ]; then rowres FLASH_DEVINFO FAIL "unreadable: ${RAWERR[0x054]}"
    elif [ "${G_OK[BOOT_FLAGS0]}" != 1 ]; then rowres FLASH_DEVINFO FAIL "FLASH_DEVINFO_ENABLE unknown: BOOT_FLAGS0 unreadable"
    else
      v="${RAWV[0x054]}"; ok=$(( v == E_FD[ENT] ))
      if (( G_VOTE[BOOT_FLAGS0] & 0x20 )) && (( ((v >> 8) & 0xf) != 0xc )); then ok=0; fi
      [ "$ok" = 1 ] && rowres FLASH_DEVINFO PASS "raw $(hx "$v") = entry; CS0_SIZE 0x$(printf '%x' $(( (v >> 8) & 0xf )))" \
        || rowres FLASH_DEVINFO FAIL "raw $(hx "$v"), entry $(hx "${E_FD[ENT]}"); with FLASH_DEVINFO_ENABLE set CS0_SIZE must be 0xc (16 MiB)"
    fi
  else
    if [ "${G_OK[BOOT_FLAGS0]}" != 1 ]; then rowres FLASH_DEVINFO FAIL "FLASH_DEVINFO_ENABLE unknown: BOOT_FLAGS0 unreadable"
    elif (( G_VOTE[BOOT_FLAGS0] & 0x20 )); then rowres FLASH_DEVINFO FAIL "FLASH_DEVINFO_ENABLE (BOOT_FLAGS0 bit 5) is set"
    else rowres FLASH_DEVINFO PASS "FLASH_DEVINFO_ENABLE 0"; fi
  fi

  # USB_BOOT_FLAGS x3
  v="${G_VOTE[USB_BOOT_FLAGS]:-0}"
  if [ "$PROFILE" = retail ]; then
    group_line USB_BOOT_FLAGS $(( v == E_UBF[ENT] )) "vote $(hx "$v"), entry $(hx "${E_UBF[ENT]}")"
  else
    group_line USB_BOOT_FLAGS 1 "copies equal (value not compared: CANNOT PROVE)"
  fi

  # White-label address, table and strings (retail only).
  if [ "$PROFILE" = retail ]; then
    if [ -n "${RAWERR[0x05c]:-}" ]; then rowres USB_WHITE_LABEL_ADDR FAIL "unreadable: ${RAWERR[0x05c]}"
    elif [ "${RAWV[0x05c]}" -eq "${E_WLA[ENT]}" ]; then rowres USB_WHITE_LABEL_ADDR PASS "raw $(hx "${RAWV[0x05c]}") = entry"
    else rowres USB_WHITE_LABEL_ADDR FAIL "raw $(hx "${RAWV[0x05c]}"), entry $(hx "${E_WLA[ENT]}")"; fi
    for k in ${E_WL[ENT]}; do
      r="${k%%=*}"; name="WHITE_LABEL_$r"; want=$((16#${k#*=0x}))
      if [ -n "${RAWERR[$r]:-}" ]; then rowres "$name" FAIL "unreadable: ${RAWERR[$r]}"
      elif [ "${RAWV[$r]}" -eq "$want" ]; then rowres "$name" PASS "raw $(hx "$want") = entry"
      else rowres "$name" FAIL "raw $(hx "${RAWV[$r]}"), entry $(hx "$want")"; fi
    done
  fi

  # Page locks: LOCK1 0x040404 and LOCK0 0, exactly.
  for name in PAGE1_LOCK0 PAGE1_LOCK1 PAGE2_LOCK0 PAGE2_LOCK1; do
    case "$name" in *LOCK1) want=$((0x040404)) ;; *) want=0 ;; esac
    if [ -n "${PLERR[$name]}" ]; then rowres "$name" FAIL "unreadable: ${PLERR[$name]}"
    elif [ "${PLV[$name]}" -eq "$want" ]; then rowres "$name" PASS "$(hx "$want")"
    else rowres "$name" FAIL "$(hx "${PLV[$name]}"), want $(hx "$want")"; fi
  done

  # The required set comes from the profile and the matched entry, never from
  # what the board reported.
  req=(CHIPID SLOT0 CRIT0 CRIT1 SLOT1 SLOT2 SLOT3 FLASH_DEVINFO USB_BOOT_FLAGS PAGE1_LOCK0 PAGE1_LOCK1 PAGE2_LOCK0 PAGE2_LOCK1)
  [ "$ki" = "-" ] || req+=(BOOT_FLAGS1)
  [ "$dob" = "-" ] || req+=(BOOT_FLAGS0)
  if [ "$PROFILE" = retail ]; then
    req+=(USB_WHITE_LABEL_ADDR)
    for k in ${E_WL[ENT]}; do req+=("WHITE_LABEL_${k%%=*}"); done
  fi
  for name in "${req[@]}"; do
    [ -n "${ROW_SEEN[$name]:-}" ] || { rowres "$name" FAIL "required row never compared (tool error)"; }
  done
  [ "${#ROW_NAMES[@]}" -eq "${#req[@]}" ] || TOOL_ERR="${TOOL_ERR:-compared ${#ROW_NAMES[@]} rows, required ${#req[@]}}"
  if [ -n "$TOOL_ERR" ]; then say "  TOOL ERROR: $TOOL_ERR"; NFAIL=$((NFAIL + 1)); fi
  [ "$NFAIL" -eq 0 ] && RESULT_OK=1
  return 0
}

cannot_prove() {
  [ "$PROFILE" = rehearsal ] || return 0
  say ""
  say "CANNOT PROVE (rehearsal profile):"
  say "  - that slot 0 holds SeedHammer's key (it holds --rehearsal-key's)"
  say "  - the retail white-label, USB_BOOT_FLAGS and FLASH_DEVINFO values"
  say "  - flash above 4 MB"
  say "  - SeedHammer's on-device sealing (bootkey-rehearsal-fidelity-residue (b))"
}

result_line() { # after a judge that passed
  if [ "$PROFILE" = rehearsal ]; then say "RESULT: REHEARSAL PROFILE PASS — not a retail check"
  elif [ "$TEST_ONLY" = 1 ]; then say "RESULT: TEST ENTRY — not a retail check"
  else say "RESULT: PASS (retail entry ${E_ID[ENT]})"; fi
}

confirm() { # confirm <step> [extra line]
  local want="BURN $1 $SER" reply=""
  printf '\nThis step is IRREVERSIBLE on CHIPID %s.\n' "$SER"
  [ -z "${2:-}" ] || printf '%s\n' "$2"
  printf 'Type exactly "%s" to proceed: ' "$want"
  IFS= read -r reply || reply=""
  printf '\n'
  [ "$reply" = "$want" ] || die1 "confirmation did not match (got '$reply'); nothing was written"
}

# --------------------------------------------------------------------------
# Commands
# --------------------------------------------------------------------------
cmd_check() {
  gate_or_exit
  say ""
  say "check --profile $PROFILE --ser $SER --slot1 $SLOT1 --disable-otp-boot $DOB --key-invalid $KI"
  read_board
  judge "$SLOT1" "$DOB" "$KI"
  cannot_prove
  if [ "$RESULT_OK" = 1 ]; then result_line; exit 0; fi
  say "RESULT: FAIL ($NFAIL row(s))"
  exit 2
}

# The write steps (plan 3.3).
DERIVED=""
cmd_write() {
  local step="$CMD" tgt T E sel val c0 bit11 k extra=""
  local -a cs
  gate_or_exit
  read_board
  if [ "$step" = disable-otp-boot ]; then
    tgt=BOOT_FLAGS0; T=$((0x2000)); sel=BOOT_FLAGS0.DISABLE_OTP_BOOT; val=0x1
    copy_rule BOOT_FLAGS1 || die2 "pre-state: BOOT_FLAGS1 $WHY"
    DERIVED=$(( (G_VOTE[BOOT_FLAGS1] >> 8) & 0xf ))
    case "$DERIVED" in 0) DERIVED=0 ;; 12) DERIVED=c ;;
      *) die2 "pre-state: KEY_INVALID is 0x$(printf '%x' "$DERIVED") in every copy; disable-otp-boot accepts only 0 or 0xc" ;; esac
    say ""; say "pre-state (KEY_INVALID derived from the board: $DERIVED):"
    judge valid - "$DERIVED"
  else
    tgt=BOOT_FLAGS1; T=$((0xc00)); sel=BOOT_FLAGS1.KEY_INVALID; val=0xc
    copy_rule BOOT_FLAGS0 || die2 "pre-state: BOOT_FLAGS0 $WHY"
    DERIVED=$(( (G_VOTE[BOOT_FLAGS0] >> 13) & 1 ))
    case "$DERIVED" in 0|1) ;; *) die2 "pre-state: DISABLE_OTP_BOOT is $DERIVED; invalidate-spare-keys accepts only 0 or 1" ;; esac
    say ""; say "pre-state (DISABLE_OTP_BOOT derived from the board: $DERIVED):"
    judge valid "$DERIVED" -
    extra="Slots 2 and 3 can never hold a key after this."
  fi
  [ "$RESULT_OK" = 1 ] || die2 "pre-state check failed ($NFAIL row(s)); nothing was written"

  # E: the expected post-write value of the target row, from the expected
  # state, never from copy 0.
  [ "${G_OK[$tgt]}" = 1 ] || die2 "$tgt unreadable: ${G_ERR[$tgt]}"
  read -r -a cs <<<"${G_COPIES[$tgt]}"
  if [ "$tgt" = BOOT_FLAGS0 ]; then
    E=0; [ "$PROFILE" = retail ] && E="${E_BF0[ENT]}"
    E=$(( E | 0x2000 ))
    bit11=$(( cs[0] & 0x800 ))
    for c0 in "${cs[@]}"; do
      [ $(( c0 & 0x800 )) -eq "$bit11" ] || die2 "BOOT_FLAGS0 copies disagree on bit 11 (ROLLBACK_REQUIRED): $(copies_hex "${G_COPIES[$tgt]}")"
    done
    E=$(( E | bit11 ))
  else
    E=0; [ "$PROFILE" = retail ] && E="${E_BF1[ENT]}"
    E=$(( E | 0x003 | 0xc00 ))
  fi
  c0="${cs[0]}"
  say ""
  say "$tgt copies $(copies_hex "${G_COPIES[$tgt]}") (copy 0 from -c 1); E $(hx "$E"), T $(hx "$T")"

  if copies_equal "$tgt" && [ "$c0" -eq "$E" ]; then
    say "already written: every copy equals E; no write."
    cmd_postcheck 0
  elif copies_equal "$tgt" && [ "$c0" -eq $(( E & ~T )) ]; then
    say "copies equal E & ~T: writing T."
  elif ! copies_equal "$tgt"; then
    # The heal rule (plan 3.3), conditions 1-3 on the per-copy reads.
    for k in "${cs[@]}"; do
      [ $(( k & ~E & 0xffffff )) -eq 0 ] || die2 "heal refused: copy $(hx "$k") holds a bit outside E $(hx "$E")"          # 1
      [ $(( k & ~T & 0xffffff )) -eq $(( E & ~T & 0xffffff )) ] || die2 "heal refused: copy $(hx "$k") differs from E outside T"  # 2
    done
    [ $(( c0 | T )) -eq "$E" ] || die2 "heal refused: copy 0 | T = $(hx $(( c0 | T ))), not E $(hx "$E") (picotool computes the write from copy 0)"  # 3
    # 4: RAW_VALUE present and equal to the per-copy reads, copy for copy.
    [ -n "${G_RAW[$tgt]}" ] || die2 "heal refused: the named read printed no RAW_VALUE for unequal copies"
    raw_matches "$tgt" || die2 "heal refused: RAW_VALUE=${G_RAW[$tgt]// /;} disagrees with the per-copy reads $(copies_hex "${G_COPIES[$tgt]}") (-c 1 is not reading copy 0 on this board)"
    say "healing an unequal copy: copies $(copies_hex "${G_COPIES[$tgt]}") -> $(hx "$E") in every copy"
  else
    die2 "$tgt copies are equal at $(hx "$c0"), which is neither E $(hx "$E") nor E & ~T $(hx $(( E & ~T )))"
  fi

  pt_build_otp_set "$SER" -s -- "$sel" "$val"
  if [ "$EXECUTE" != 1 ]; then
    say "dry-run: would run: picotool ${PT_ARGV[*]}"
    say "dry-run: nothing was written. Add --execute to write."
    exit 0
  fi
  confirm "$step" "$extra"
  say "\$ picotool ${PT_ARGV[*]}"
  pt_exec
  say "$PT_OUT"
  [ "$PT_RC" -eq 0 ] || die3 "picotool otp set exited $PT_RC"
  cmd_postcheck 1
}

# cmd_postcheck <written 0|1>: the full check with the step's flag set.
cmd_postcheck() {
  local written="$1"
  say ""; say "post-write check:"
  if ! try_gate; then [ "$written" = 1 ] && die3 "post-write read: $ERR"; die2 "post-check read: $ERR"; fi
  read_board
  if [ "$CMD" = disable-otp-boot ]; then
    judge valid 1 "$DERIVED"
  else
    judge valid "$DERIVED" c
  fi
  if [ "$RESULT_OK" = 1 ]; then
    if [ "$written" = 1 ]; then say "post-write check: PASS"; else say "check: PASS (no write was issued)"; fi
    cannot_prove
    exit 0
  fi
  [ "$written" = 1 ] && die3 "post-write check failed ($NFAIL row(s))"
  die2 "post-check failed ($NFAIL row(s)); no write was issued"
}

# Flash (plan 3.4).
FLASH_FROM=$((0x10000000))
flash_to() { if [ "$PROFILE" = retail ]; then echo $((0x11000000)); else echo $((0x10400000)); fi; }
TMPD=""
mk_tmpd() { TMPD="$(mktemp -d "${TMPDIR:-/tmp}/refugium-otp.XXXXXX")" || die1 "mktemp failed"; trap 'rm -rf "$TMPD"' EXIT; }
condemned() { die2 "$1. The boot ROM refused this range or the flash did not behave: treat the engraver as an unknown image (condemned)."; }

alias_probe() {
  local m="$TMPD/marker.bin" off got
  head -c 4096 /dev/urandom > "$m" || die1 "cannot make the probe marker"
  say "alias probe: 4 KiB random marker sha256 $(sha256sum "$m" | cut -d' ' -f1)"
  pt_build_load "$SER" "$m" "$(printf '0x%08x' "$FLASH_FROM")"; say "\$ picotool ${PT_ARGV[*]}"; pt_exec; say "$PT_OUT"
  [ "$PT_RC" -eq 0 ] || condemned "the probe's load exited $PT_RC"
  for off in 4 8 12; do
    got="$TMPD/probe-$off.bin"
    pt_build_save "$SER" "$(printf '0x%08x' $(( FLASH_FROM + off * 1048576 )))" \
      "$(printf '0x%08x' $(( FLASH_FROM + off * 1048576 + 4096 )))" "$got"
    pt_exec
    if [ "$PT_RC" -ne 0 ]; then rm -f "$got"; condemned "reading +$off MiB exited $PT_RC"; fi
    if [ "$(sha256sum < "$m")" = "$(sha256sum < "$got")" ]; then
      die2 "the marker reappears at +$off MiB: this flash is smaller than 16 MB (condemned)"
    fi
  done
  say "alias probe: no copy of the marker at +4, +8 or +12 MiB"
}

cmd_erase() {
  local to f left
  to="$(flash_to)"
  gate_or_exit
  if [ "$EXECUTE" != 1 ]; then
    if [ "$PROBE_ONLY" = 1 ]; then say "dry-run: would write a 4 KiB marker at 0x10000000 and look for it at +4/+8/+12 MiB"
    else
      [ "$PROFILE" = retail ] && say "dry-run: would run the alias probe (writes a 4 KiB marker), then"
      say "dry-run: would erase $(printf '0x%08x' "$FLASH_FROM")-$(printf '0x%08x' "$to") and read it back"
    fi
    say "dry-run: nothing was written. Add --execute to write."
    exit 0
  fi
  confirm erase-range
  mk_tmpd
  if [ "$PROFILE" = retail ] || [ "$PROBE_ONLY" = 1 ]; then alias_probe; fi
  [ "$PROBE_ONLY" = 1 ] && { say "probe-only: done"; exit 0; }
  pt_build_erase "$SER" "$(printf '0x%08x' "$FLASH_FROM")" "$(printf '0x%08x' "$to")"
  say "\$ picotool ${PT_ARGV[*]}"; pt_exec; say "$PT_OUT"
  [ "$PT_RC" -eq 0 ] || condemned "erase exited $PT_RC"
  f="$TMPD/readback.bin"
  pt_build_save "$SER" "$(printf '0x%08x' "$FLASH_FROM")" "$(printf '0x%08x' "$to")" "$f"
  pt_exec
  if [ "$PT_RC" -ne 0 ]; then rm -f "$f"; condemned "the read-back exited $PT_RC"; fi
  [ "$(stat -c%s "$f")" -eq $(( to - FLASH_FROM )) ] || condemned "the read-back is $(stat -c%s "$f") bytes, the range is $(( to - FLASH_FROM ))"
  # Count non-0xFF bytes into a file and stat it: no pipe into grep -q (F-695).
  tr -d '\377' < "$f" > "$TMPD/left.bin"
  left="$(stat -c%s "$TMPD/left.bin")"
  [ "$left" -eq 0 ] || die2 "flash not erased: $left byte(s) in the range are not 0xFF (condemned)"
  say "flash $(printf '0x%08x' "$FLASH_FROM")-$(printf '0x%08x' "$to") erased and verified (every byte 0xFF)"
  exit 0
}

cmd_save() {
  local to
  to="$(flash_to)"
  gate_or_exit
  pt_build_save "$SER" "$(printf '0x%08x' "$FLASH_FROM")" "$(printf '0x%08x' "$to")" "$OUTF"
  say "\$ picotool ${PT_ARGV[*]}"; pt_exec; say "$PT_OUT"
  if [ "$PT_RC" -ne 0 ]; then rm -f "$OUTF"; condemned "save exited $PT_RC (the partial file was deleted)"; fi
  [ "$(stat -c%s "$OUTF")" -eq $(( to - FLASH_FROM )) ] || { rm -f "$OUTF"; condemned "save wrote the wrong size"; }
  say "sha256 $(sha256sum "$OUTF" | cut -d' ' -f1)  $OUTF"
  exit 0
}

# inject-copy (plan 3.6): one copy only, then prove it.
cmd_inject() {
  local nm r0 bit target_k v i
  local -a before after
  gate_or_exit
  if [ "$CASE" = bf0-copy3 ]; then nm=BOOT_FLAGS0; r0=$((0x048)); bit=$((0x2000)); target_k=2
  else nm=BOOT_FLAGS1; r0=$((0x04b)); bit=$((0x800)); target_k=0; fi
  read_group "$nm" "$r0" 3
  copies_equal "$nm" || die2 "inject-copy: $nm copies are not equal before the injection: $(copies_hex "${G_COPIES[$nm]:-}") ${G_ERR[$nm]:-}"
  read -r -a before <<<"${G_COPIES[$nm]}"
  [ $(( before[0] & bit )) -eq 0 ] || die2 "inject-copy: the bit $(hx "$bit") is already set in $nm"
  if [ "$CASE" = bf0-copy3 ]; then
    pt_build_otp_set "$SER" -s -- 0x04a "$(hx "$bit")"
  else
    pt_build_otp_set "$SER" -c 1 -s -- 0x04b "$(hx $(( before[0] | bit )))"
  fi
  say "$nm copies before: $(copies_hex "${G_COPIES[$nm]}")"
  if [ "$EXECUTE" != 1 ]; then
    say "dry-run: would run: picotool ${PT_ARGV[*]}"
    say "dry-run: nothing was written. Add --execute to write."
    exit 0
  fi
  confirm inject-copy
  say "\$ picotool ${PT_ARGV[*]}"
  pt_exec; say "$PT_OUT"
  [ "$PT_RC" -eq 0 ] || die3i "picotool otp set exited $PT_RC"
  read_group "$nm" "$r0" 3 || die3i "read-back: ${G_ERR[$nm]}"
  read -r -a after <<<"${G_COPIES[$nm]}"
  say "$nm copies after:  $(copies_hex "${G_COPIES[$nm]}")"
  # Copy 0 comes from -c 1; cross-check it with RAW_VALUE[0] (F10).
  [ -n "${G_RAW[$nm]}" ] || die3i "the named read printed no RAW_VALUE although one copy should now differ"
  raw_matches "$nm" || die3i "RAW_VALUE=${G_RAW[$nm]// /;} disagrees with the per-copy reads (-c 1 is not reading copy 0 on this board)"
  for i in 0 1 2; do
    if [ "$i" = "$target_k" ]; then v=$(( before[i] | bit )); else v="${before[i]}"; fi
    [ "${after[i]}" -eq "$v" ] || die3i "copy $i is $(hx "${after[i]}"), expected $(hx "$v")"
  done
  say "injected: copy $target_k of $nm now holds $(hx "${after[target_k]}"), every other copy unchanged"
  exit 0
}

# capture (plan 3.5): read-only, no profile.
cmd_capture() {
  local tdir rowsj="" r key v k s idx data len utf off nrows text j c out_json
  local -a groups
  gate_or_exit
  tdir="$OUTF.transcripts"
  mkdir -p "$tdir" || die1 "cannot create $tdir"
  : > "$tdir/manifest.tsv"
  declare -A CAP
  capfail() { rm -rf "$tdir"; die2 "capture incomplete: $1"; }
  # Copied rows: copy 0 with -c 1, the others bare.
  groups=(CRIT0:0x038:8 CRIT1:0x040:8 BOOT_FLAGS0:0x048:3 BOOT_FLAGS1:0x04b:3 USB_BOOT_FLAGS:0x059:3)
  for g in "${groups[@]}"; do
    IFS=: read -r nm r c <<<"$g"
    read_group "$nm" "$r" "$c" || capfail "$nm: ${G_ERR[$nm]}"
    j=0
    for v in ${G_COPIES[$nm]}; do CAP[$(printf '0x%03x' $(( r + j )))]="$v"; j=$((j + 1)); done
  done
  for r in 0 1 2 3; do read_raw "$r" "*" || capfail "${RAWERR[$(printf '0x%03x' "$r")]}"; done
  for ((r = 0x080; r < 0x0c0; r++)); do read_raw "$r" "*" || capfail "${RAWERR[$(printf '0x%03x' "$r")]}"; done
  read_raw 0x054 OTP_DATA_FLASH_DEVINFO || capfail "${RAWERR[0x054]}"
  read_raw 0x05c OTP_DATA_USB_WHITE_LABEL_ADDR || capfail "${RAWERR[0x05c]}"
  for l in PAGE1_LOCK0:0xf82 PAGE1_LOCK1:0xf83 PAGE2_LOCK0:0xf84 PAGE2_LOCK1:0xf85; do
    try_og_row "" "${l#*:}" "OTP_DATA_${l%%:*}" || capfail "$ERR"
    CAP[${l#*:}]="$OG_INT"
  done
  for k in "${!RAWV[@]}"; do CAP[$k]="${RAWV[$k]}"; done

  # White label (F7): USB_BOOT_FLAGS bit 22 enables the 16-row table at the
  # row USB_WHITE_LABEL_ADDR names; each valid STRDEF (entries 4, 5, 6, 8-15)
  # points at string rows: length = low 7 bits (x2 rows... one char per row if
  # bit 7, UTF-16), row offset = high byte.
  local ubf="${G_VOTE[USB_BOOT_FLAGS]}" wla=$(( CAP[0x05c] & 0xffff )) wl_en=false
  local -a trows=() srows=()
  local strings="[]"
  if (( ubf & (1 << 22) )); then
    wl_en=true
    (( wla + 16 <= 0xf80 )) || capfail "white-label table at $(printf '0x%03x' "$wla") runs past the user rows"
    for ((idx = 0; idx < 16; idx++)); do
      r=$(( wla + idx )); key="$(printf '0x%03x' "$r")"
      read_raw "$r" "*" || capfail "${RAWERR[$key]}"
      CAP[$key]="${RAWV[$key]}"; trows+=("$key")
    done
    for idx in 4 5 6 8 9 10 11 12 13 14 15; do
      (( ubf & (1 << idx) )) || continue
      data=$(( CAP[$(printf '0x%03x' $(( wla + idx )))] & 0xffff ))
      len=$(( data & 0x7f )); utf=$(( (data >> 7) & 1 )); off=$(( (data >> 8) & 0xff ))
      if [ "$utf" = 1 ]; then nrows=$len; else nrows=$(( (len + 1) / 2 )); fi
      (( wla + off + nrows <= 0xf80 )) || capfail "white-label string $idx runs past the user rows"
      srows=(); text=""
      for ((j = 0; j < nrows; j++)); do
        r=$(( wla + off + j )); key="$(printf '0x%03x' "$r")"
        read_raw "$r" "*" || capfail "${RAWERR[$key]}"
        CAP[$key]="${RAWV[$key]}"; srows+=("$key")
        v=$(( RAWV[$key] & 0xffff ))
        if [ "$utf" = 1 ]; then text+="$(printf '\\u%04x' "$v")"
        else
          text+="$(printf '\\u%04x' $(( v & 0xff )))"
          (( 2 * j + 1 < len )) && text+="$(printf '\\u%04x' $(( v >> 8 )))"
        fi
      done
      strings="$(jq -c --argjson idx "$idx" --argjson rows "$(printf '%s\n' "${srows[@]}" | jq -R . | jq -sc .)" \
                 --arg t "$text" '. + [{index: $idx, utf16: false, rows: $rows, text: ($t | "\"" + . + "\"" | fromjson)}]' <<<"$strings")"
      [ "$utf" = 1 ] && strings="$(jq -c '.[-1].utf16 = true' <<<"$strings")"
    done
  fi

  for k in $(printf '%s\n' "${!CAP[@]}" | sort); do rowsj+="\"$k\":\"$(hx "${CAP[$k]}")\","; done
  rowsj="{${rowsj%,}}"

  # Transcripts for the replay check (plan 6.2 case 12): `otp get -n` of 0x040,
  # 0x048, 0x04b and 0x054, plain, with -r and with -c 1, and the values each
  # must parse to, computed from the per-copy reads.
  local fl suffix name width val warn raw cps crit nn
  for r in 0x040 0x048 0x04b 0x054; do
    case "$r" in 0x040) nn=CRIT1; crit=1; c=8 ;; 0x048) nn=BOOT_FLAGS0; crit=0; c=3 ;; 0x04b) nn=BOOT_FLAGS1; crit=0; c=3 ;; 0x054) nn=FLASH_DEVINFO; crit=0; c=1 ;; esac
    name="OTP_DATA_$nn"
    cps=(); for ((j = 0; j < c; j++)); do cps+=("${CAP[$(printf '0x%03x' $(( r + j )))]}"); done
    for fl in "" "-r" "-c 1"; do
      case "$fl" in "") suffix=named ;; "-r") suffix=r ;; *) suffix=c1 ;; esac
      local -a fa=(); [ -n "$fl" ] && read -r -a fa <<<"$fl"
      pt_build_otp_get "$SER" "${fa[@]}" -n -- "$r"; pt_exec
      [ "$PT_RC" -eq 0 ] || capfail "transcript $r $fl exited $PT_RC"
      printf '%s\n' "$PT_OUT" > "$tdir/${r}_${suffix}.txt"
      warn=0; raw=""
      if [ "$nn" = FLASH_DEVINFO ] && [ "$suffix" != r ]; then
        width=4; val=$(( cps[0] & 0xffff ))
        if [ "$(ecc16 "$val")" -ne "${cps[0]}" ]; then warn=1; raw="$(hx "${cps[0]}")"; fi
      elif [ "$suffix" = c1 ] || [ "$nn" = FLASH_DEVINFO ]; then
        width=6; val="${cps[0]}"
      else
        width=6; val="$(vote "$crit" "${cps[@]}")"
        if ! all_equal "${cps[*]}"; then warn=1; raw=""; for v in "${cps[@]}"; do raw+="$(hx "$v") "; done; raw="${raw% }"; fi
      fi
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "${r}_${suffix}.txt" "$r" "$name" "$width" "$(hx "$val")" "$warn" "$raw" >> "$tdir/manifest.tsv"
    done
  done

  local trj srj
  trj="$(printf '%s\n' "${trows[@]+"${trows[@]}"}" | jq -R 'select(length > 0)' | jq -sc .)"
  srj="$strings"
  out_json="$(jq -n --arg chipid "$SER" --arg v "$PTV" --argjson rows "$rowsj" \
      --argjson en "$wl_en" --argjson tr "$trj" --argjson st "$srj" \
      --arg tdir "$(basename "$tdir")" \
      '{format: "refugium-otp-capture/1", chipid: $chipid, picotool_version: $v,
        fingerprint: {MAC0: true, DISABLE_BOOTSEL_EXEC2: false},
        rows: $rows,
        white_label: {enabled: $en, table_rows: $tr, strings: $st},
        transcripts: $tdir}')" || capfail "jq could not assemble the capture"
  printf '%s\n' "$out_json" > "$OUTF"
  say "wrote $OUTF (sha256 $(sha256sum "$OUTF" | cut -d' ' -f1))"
  say "transcripts in $tdir ($(( $(wc -l < "$tdir/manifest.tsv") )) reads)"
  exit 0
}

case "$CMD" in
  check) cmd_check ;;
  capture) cmd_capture ;;
  disable-otp-boot|invalidate-spare-keys) cmd_write ;;
  erase-range) cmd_erase ;;
  save-range) cmd_save ;;
  inject-copy) cmd_inject ;;
esac
