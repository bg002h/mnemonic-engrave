# shellcheck shell=bash
#
# otp-read.sh -- shared picotool plumbing for scripts/pico2-bootkey-rehearsal.sh
# ("R") and scripts/refugium-otp.sh. Sourced, never executed.
#
# Three parts:
#
#   1. Constants both tools compare against.
#   2. THE ARGV BUILDER. Every picotool call either tool makes is built here, in
#      picotool's declared option order (plan E3a fact F14). picotool's parser
#      matches option groups in DECLARATION order, and an option written after
#      a later group silently becomes a selector that matches no row -- the
#      call then runs on whatever board is attached, with no error. That is
#      what F-619 measured when `otp get -n -c 1 0x04b` "did nothing": `-c 1`
#      had become a selector. So no call site writes picotool argv by hand:
#
#        otp get  [-c N] [-r] [-e] [-n] [--ser S] <selector...>
#        otp set  [-c N] [-r] [-e] [-s] <selector> <value> [--ser S]
#        otp load <file> [--ser S]
#        erase -r <from> <to> [--ser S]
#        save  -r <from> <to> <file.bin> [--ser S]
#        load  -v <file.bin> -o <offset> [--ser S]
#
#      An empty serial means no --ser (R's calls, unchanged behaviour). The
#      builder fills the global array PT_ARGV; the caller runs
#      `picotool "${PT_ARGV[@]}"`. pt_print_argvs prints every shape the
#      builder can emit, which is what the CI probe runs against the real
#      binary (plan section 6.4).
#   3. Readers. R's readers as non-dying try_<name> functions (return 0/1, set
#      their globals, put the message in ERR); R keeps thin wrappers that die.
#      Plus the strict single-row reader refugium-otp.sh uses (try_og_row) and
#      the transcript replay (og_replay_dir).
#
# Nothing here calls `exit` or `die`, and nothing here may be called inside
# $(...) by refugium-otp.sh: a reader's globals would be set in a subshell and
# lost.

# --------------------------------------------------------------------------
# 1. Constants
# --------------------------------------------------------------------------

# SeedHammer's production signing-key hash (fork cmd/controller/platform_sh2.go:70).
SH_SIGNKEY_HASH="c8314536d6af61ac2e62e5991e3e4711629c54696ba8c4af08965a1d319a473b"
# The fork's boot key, sha256 of its uncompressed X||Y (burned in slot 1 of
# SeedHammer #1 and #2). The same value is scripts/sh2-flash's SH2_BOOTKEY_FP
# default; scripts/test/run-e2e-otp.sh (case 14) fails if the two differ.
SH2_BOOTKEY_FP="846aa289f2f317e55ff03f90555132302842cff2f68ee45712834a25d64cabb4"
# The SeedHammer IIs on the bench (design/HARDWARE_INVENTORY.md), in --ser form:
# CHIPID3..0, uppercase, no 0x (fact F15). The rehearsal profile refuses them.
SEEDHAMMER_SERS=(77C483B745ABF55C 09F50BF63E8D6F46 DB2010F935ED25B8)
# The one picotool refugium-otp.sh accepts (design/PICOTOOL_PIN.md).
PICOTOOL_PIN="2.3.1"

PT_SER=""   # never inherited from the environment: R calls with no --ser
PT_ARGV=()
PT_OUT=""
PT_RC=0
ERR=""

# --------------------------------------------------------------------------
# 2. The argv builder
# --------------------------------------------------------------------------

# pt_build_otp_get <ser> [-c N] [-r] [-e] [-n] -- <selector...>
# Options may be given in any order; they are emitted in declaration order.
pt_build_otp_get() {
  local ser="$1"; shift
  local c="" r=0 e=0 n=0
  while [ $# -gt 0 ] && [ "$1" != "--" ]; do
    case "$1" in
      -c) c="$2"; shift 2 ;;
      -r) r=1; shift ;;
      -e) e=1; shift ;;
      -n) n=1; shift ;;
      *) ERR="pt_build_otp_get: unknown option $1"; return 2 ;;
    esac
  done
  [ "${1:-}" = "--" ] && shift
  [ $# -ge 1 ] || { ERR="pt_build_otp_get: no selector"; return 2; }
  PT_ARGV=(otp get)
  [ -n "$c" ] && PT_ARGV+=(-c "$c")
  [ "$r" = 1 ] && PT_ARGV+=(-r)
  [ "$e" = 1 ] && PT_ARGV+=(-e)
  [ "$n" = 1 ] && PT_ARGV+=(-n)
  [ -n "$ser" ] && PT_ARGV+=(--ser "$ser")
  PT_ARGV+=("$@")
  return 0
}

# pt_build_otp_set <ser> [-c N] [-r] [-e] [-s] -- <selector> <value>
pt_build_otp_set() {
  local ser="$1"; shift
  local c="" r=0 e=0 s=0
  while [ $# -gt 0 ] && [ "$1" != "--" ]; do
    case "$1" in
      -c) c="$2"; shift 2 ;;
      -r) r=1; shift ;;
      -e) e=1; shift ;;
      -s) s=1; shift ;;
      *) ERR="pt_build_otp_set: unknown option $1"; return 2 ;;
    esac
  done
  [ "${1:-}" = "--" ] && shift
  [ $# -eq 2 ] || { ERR="pt_build_otp_set: want <selector> <value>"; return 2; }
  PT_ARGV=(otp set)
  [ -n "$c" ] && PT_ARGV+=(-c "$c")
  [ "$r" = 1 ] && PT_ARGV+=(-r)
  [ "$e" = 1 ] && PT_ARGV+=(-e)
  [ "$s" = 1 ] && PT_ARGV+=(-s)
  PT_ARGV+=("$1" "$2")
  [ -n "$ser" ] && PT_ARGV+=(--ser "$ser")
  return 0
}

# pt_build_otp_load <ser> <file>
pt_build_otp_load() {
  PT_ARGV=(otp load "$2")
  [ -n "$1" ] && PT_ARGV+=(--ser "$1")
  return 0
}

# pt_build_erase <ser> <from> <to>
pt_build_erase() {
  PT_ARGV=(erase -r "$2" "$3")
  [ -n "$1" ] && PT_ARGV+=(--ser "$1")
  return 0
}

# pt_build_save <ser> <from> <to> <file.bin>
pt_build_save() {
  PT_ARGV=(save -r "$2" "$3" "$4")
  [ -n "$1" ] && PT_ARGV+=(--ser "$1")
  return 0
}

# pt_build_load <ser> <file.bin> <offset>   (always verifies: -v)
pt_build_load() {
  PT_ARGV=(load -v "$2" -o "$3")
  [ -n "$1" ] && PT_ARGV+=(--ser "$1")
  return 0
}

# pt_build_info <ser>   (empty serial: the bare board-count probe)
pt_build_info() {
  PT_ARGV=(info)
  [ -n "$1" ] && PT_ARGV+=(--ser "$1")
  return 0
}

pt_build_version() { PT_ARGV=(version -s); return 0; }

# pt_build_otp_list <selector>
pt_build_otp_list() { PT_ARGV=(otp list -n "$1"); return 0; }

# pt_exec -> PT_OUT (stdout+stderr), PT_RC. Never dies.
pt_exec() {
  PT_RC=0
  PT_OUT="$(picotool "${PT_ARGV[@]}" 2>&1)" || PT_RC=$?
  return 0
}

# pt_print_argvs <ser> <dir>: one concrete argv per line, covering every shape
# the builder can emit (every option combination, one and several selectors),
# with file arguments under <dir>. Print mode for the CI probe (plan 6.4): the
# probe runs each line against the real picotool with no board attached.
pt_print_argvs() {
  local ser="$1" dir="$2" c r e x sels
  local -a o
  pt_build_version; printf '%s\n' "${PT_ARGV[*]}"
  pt_build_otp_list MAC0; printf '%s\n' "${PT_ARGV[*]}"
  pt_build_otp_list BOOT_FLAGS0.DISABLE_BOOTSEL_EXEC2; printf '%s\n' "${PT_ARGV[*]}"
  pt_build_info ""; printf '%s\n' "${PT_ARGV[*]}"
  pt_build_info "$ser"; printf '%s\n' "${PT_ARGV[*]}"
  for c in 0 1; do for r in 0 1; do for e in 0 1; do for x in 0 1; do
    for sels in "0x04b" "CHIPID0 CHIPID1"; do
      o=(); [ "$c" = 1 ] && o+=(-c 1); [ "$r" = 1 ] && o+=(-r); [ "$e" = 1 ] && o+=(-e); [ "$x" = 1 ] && o+=(-n)
      # shellcheck disable=SC2086  # $sels is one or two selector words on purpose
      pt_build_otp_get "$ser" "${o[@]}" -- $sels; printf '%s\n' "${PT_ARGV[*]}"
    done
    o=(); [ "$c" = 1 ] && o+=(-c 1); [ "$r" = 1 ] && o+=(-r); [ "$e" = 1 ] && o+=(-e); [ "$x" = 1 ] && o+=(-s)
    pt_build_otp_set "$ser" "${o[@]}" -- 0x04b 0x000803; printf '%s\n' "${PT_ARGV[*]}"
  done; done; done; done
  pt_build_otp_load "$ser" "$dir/probe.json"; printf '%s\n' "${PT_ARGV[*]}"
  pt_build_erase "$ser" 0x10000000 0x10001000; printf '%s\n' "${PT_ARGV[*]}"
  pt_build_save "$ser" 0x10000000 0x10001000 "$dir/probe-save.bin"; printf '%s\n' "${PT_ARGV[*]}"
  pt_build_load "$ser" "$dir/probe.bin" 0x10000000; printf '%s\n' "${PT_ARGV[*]}"
}

# pt_shape <argv...>: the argv with the serial, every number and every file
# replaced by placeholders, and the selectors of `otp get`/`otp list` by <SEL>
# (a run of several by <SEL>+), so the probe's argvs and the fake's argv log
# normalise the same way.
pt_shape() {
  local cmd="$1" sub="" k=0 tok npos=0
  local -a out=("$1") args
  shift
  if [ "$cmd" = otp ]; then sub="${1:-}"; out+=("$sub"); shift; fi
  args=("$@")
  while [ "$k" -lt "${#args[@]}" ]; do
    tok="${args[$k]}"
    case "$tok" in
      --ser) out+=(--ser '<S>'); k=$((k+2)); continue ;;
      -c|-o) out+=("$tok" '<N>'); k=$((k+2)); continue ;;
      -r)
        if [ "$cmd" = erase ] || [ "$cmd" = save ]; then out+=(-r '<N>' '<N>'); k=$((k+3))
        else out+=(-r); k=$((k+1)); fi
        continue ;;
      -*) out+=("$tok"); k=$((k+1)); continue ;;
    esac
    case "$cmd${sub:+ $sub}" in
      "otp get"|"otp list")
        if [ "${out[-1]}" = '<SEL>' ] || [ "${out[-1]}" = '<SEL>+' ]; then out[-1]='<SEL>+'; else out+=('<SEL>'); fi ;;
      "otp set")
        if [ "$npos" -eq 0 ]; then out+=('<SEL>'); else out+=('<N>'); fi ;;
      *) out+=('<F>') ;;
    esac
    npos=$((npos+1)); k=$((k+1))
  done
  printf '%s\n' "${out[*]}"
}

# --------------------------------------------------------------------------
# 3a. R's readers, as try_ forms. Same picotool calls (through the builder),
#     same parsing, same messages as R had inline; R's wrappers die on 1.
# --------------------------------------------------------------------------

# picotool's `otp get` output (2.3.1 main.cpp otp_get_command::execute,
# PT:9162-9330; 2.2.0-a4 has the same layout except that an ECC VALUE was the
# 24-bit re-encoding there and is the 16-bit data in 2.3.1):
#   "ROW 0x%04x" [": " name] [" (ECC)"/" (CRIT)"/" (RBIT-n)"] [" (Part i/n)"]
#   "\nVALUE 0x%06x\n"  (0x%04x for an ECC row in 2.3.1)
#   "field <NAME> (bit n|bits n-m) = %x"   <- the field, BARE hex, no 0x prefix
# So a field must be read from the `field ... = ` line, never from VALUE.

# try_otp_field <selector> -> OTP_FIELD_VAL (bare lowercase hex)
OTP_FIELD_VAL=""
try_otp_field() {
  local sel="$1" out v
  OTP_FIELD_VAL=""
  pt_build_otp_get "$PT_SER" -n -- "$sel" || return 1
  out="$(picotool "${PT_ARGV[@]}" 2>&1)" || { ERR="OTP read failed for $sel:
$out"; return 1; }
  # CRIT1 is RBIT-8 and BOOT_FLAGS1 is RBIT-3; an inconsistent redundant read
  # must not be parsed as a clean value.
  # F-695: every WARNING trap is a pure-bash test, never `printf | grep -q`:
  # under pipefail a SIGPIPE'd printf makes the pipeline false and `&& die` is
  # SKIPPED (fail OPEN). Not a here-string either: at >=64 KiB bash backs one
  # with a temp file, and a full /tmp would skip grep and the trap (review M1).
  if [[ ${out,,} == *warning* ]]; then
    ERR="picotool reported a warning reading $sel (redundant rows disagree or ECC invalid):
$out"; return 1
  fi
  v="$(printf '%s\n' "$out" | grep -iE '^[[:space:]]*field ' | tail -1 \
       | sed -E 's/.*=[[:space:]]*//' | tr -d '[:space:]' | tr 'A-F' 'a-f')"
  [ -n "$v" ] || { ERR="could not parse a field value for $sel from picotool output:
$out"; return 1; }
  printf '%s' "$v" | grep -qE '^[0-9a-f]+$' || { ERR="unexpected field value '$v' for $sel"; return 1; }
  OTP_FIELD_VAL="$v"
}

# try_read_rows <selector...> -> ROWVALS: space-separated 4-hex-digit values,
# one per requested row, in order.
#
# ONE picotool invocation for all rows, deliberately. Querying a single row that
# is part of a sequence ("Part 2/4") can print NOTHING with exit 0 -- observed on
# real hardware for CHIPID1 with picotool 2.2.0-a4, where the row is readable
# but only when requested alongside its siblings (fixed in 2.3.1, PT D7; the
# batching stays).
ROWVALS=""
try_read_rows() {
  local out want=$# got
  ROWVALS=""
  pt_build_otp_get "$PT_SER" -e -n -- "$@" || return 1
  out="$(picotool "${PT_ARGV[@]}" 2>&1)" || { ERR="OTP read failed for: $*
$out"; return 1; }
  if [[ ${out,,} == *warning* ]]; then
    ERR="picotool reported a warning reading: $*
$out"; return 1
  fi
  ROWVALS="$(printf '%s\n' "$out" | grep -oiE '^[[:space:]]*VALUE 0x[0-9a-f]+' \
             | grep -oiE '0x[0-9a-f]+' | sed 's/^0[xX]//' | tr 'A-F' 'a-f' \
             | while read -r v; do printf '%04x ' $(( 16#$v & 0xffff )); done)"
  got="$(printf '%s' "$ROWVALS" | wc -w)"
  [ "$got" -eq "$want" ] || { ERR="expected $want row values for '$*', parsed $got.
picotool output was:
$out"; return 1; }
  # picotool returns rows ASCENDING-SORTED with silent dedup (its filter_otp
  # builds a std::map keyed by row) -- NOT in request order. Positional mapping
  # is therefore only valid for distinct ascending selectors. Verify the row
  # names it echoed match what we asked for, in order, rather than assume it.
  local names
  names="$(printf '%s\n' "$out" | grep -oiE '^ROW 0x[0-9a-f]+: OTP_DATA_[A-Z0-9_]+' \
           | sed -E 's/.*OTP_DATA_//')"
  [ "$(printf '%s\n' "$names" | wc -l)" -eq "$want" ] \
    || { ERR="parsed $(printf '%s\n' "$names" | wc -l) ROW names for '$*', expected $want"; return 1; }
  local k=1 sel
  for sel in "$@"; do
    [ "$(printf '%s\n' "$names" | sed -n "${k}p")" = "$sel" ] || { ERR="picotool returned rows in an unexpected order.
  requested position $k: $sel
  received:            $(printf '%s\n' "$names" | sed -n "${k}p")
Positional mapping is unsafe here; refusing to guess which value is which."; return 1; }
    k=$((k+1))
  done
}

# try_read_slot <n> -> SLOT_HEX: 64 hex chars (the 32-byte key hash).
# Each row holds two bytes, LOW BYTE FIRST, so each row is byte-swapped.
SLOT_HEX=""
try_read_slot() {
  local n="$1" i v
  local -a sels=()
  SLOT_HEX=""
  for i in $(seq 0 15); do sels+=("BOOTKEY${n}_${i}"); done
  try_read_rows "${sels[@]}" || return 1
  for v in $ROWVALS; do SLOT_HEX="${SLOT_HEX}${v:2:2}${v:0:2}"; done
  [ ${#SLOT_HEX} -eq 64 ] || { ERR="slot $n reassembled to ${#SLOT_HEX} hex chars, expected 64"; return 1; }
}

# try_chipid -> CHIPID_HEX (R's word-reversed spelling: rows 0,1,2,3 in row
# order) and CHIPID_SER (picotool's --ser spelling: CHIPID3..0, uppercase).
CHIPID_HEX=""; CHIPID_SER=""
try_chipid() {
  local out="" v ser=""
  CHIPID_HEX=""; CHIPID_SER=""
  try_read_rows CHIPID0 CHIPID1 CHIPID2 CHIPID3 || return 1
  for v in $ROWVALS; do out="${out}${v}"; ser="${v}${ser}"; done
  # CHIPID is factory-programmed and unique; all-zeros means an unprogrammed
  # part or a simulator. Never pin or compare against it -- a stale all-zero
  # pin left by a test would make the real device look like the wrong device.
  case "$out" in
    0000000000000000) ERR="CHIPID reads all zeros.
That is not a real RP2350 identity -- it means an unprogrammed part, or that
picotool is talking to a simulator rather than hardware. Refusing to pin or
trust this identity."; return 1 ;;
  esac
  [ ${#out} -eq 16 ] || { ERR="CHIPID reassembled to ${#out} hex chars, expected 16"; return 1; }
  CHIPID_HEX="$out"
  CHIPID_SER="${ser^^}"
}

# Page-lock rows are ecc=false in picotool's table, so they must be read WITHOUT
# -e and kept at full 24 bits: a genuinely locked page would decode as
# ECC-invalid and be misreported as a read warning rather than as "LOCKED",
# and masking to 16 bits would discard the third lock copy in bits 23:16.
# try_read_row_raw24 <selector> -> ROW_RAW24 (6 hex digits)
ROW_RAW24=""
try_read_row_raw24() {
  local sel="$1" out v
  ROW_RAW24=""
  pt_build_otp_get "$PT_SER" -n -- "$sel" || return 1
  out="$(picotool "${PT_ARGV[@]}" 2>&1)" || { ERR="OTP read failed for $sel:
$out"; return 1; }
  # F-619: this function once had NO warning trap, unlike otp_field and
  # read_rows. It reads the page-lock rows and the BOOT_FLAGS1/CRIT1 copies --
  # all majority-vote-encoded -- so an inconsistent redundant read was parsed
  # as a clean value in exactly the places redundancy is being checked. `0x04b`
  # resolves to the NAMED row (picotool prints `OTP_DATA_BOOT_FLAGS1 (RBIT-3)`)
  # while `0x04c`/`0x04d` print bare, so the three reads are NOT symmetric and
  # an A/B/C comparison cannot be the thing that catches a degraded row. This
  # trap is.
  if [[ ${out,,} == *warning* ]]; then
    ERR="picotool reported a warning reading row $sel (redundant rows disagree or ECC invalid):
$out"; return 1
  fi
  v="$(printf '%s\n' "$out" | grep -oiE '^[[:space:]]*VALUE 0x[0-9a-f]+' | tail -1 \
       | grep -oiE '0x[0-9a-f]+' | sed 's/^0[xX]//' | tr 'A-F' 'a-f')"
  [ -n "$v" ] || { ERR="could not parse a VALUE line for row $sel:
$out"; return 1; }
  ROW_RAW24="$(printf '%06x' $(( 16#$v & 0xffffff )))"
}

# try_check_page_locks -> PL_LINES: "info|warn<TAB>message" lines to print, in
# order. On a refusal ERR holds R's message and PL_LINES what preceded it.
#
# Page-lock rows are NOT simply "zero = fine". Each is a byte replicated
# 3-way (majority-vote encoded) with three 2-bit permission fields, per
# picotool's own OTP table (generated from the RP2350 datasheet):
#
#   LOCK_S  (bits 0-1)  0x0 = page fully accessible by SECURE software
#   LOCK_NS (bits 2-3)  0x0/0x1 = Non-secure may read (0x1 = NS read-only)
#   LOCK_BL (bits 4-5)  0x0 = bootloader permits user reads AND writes
#
# picotool/PICOBOOT acts as SECURE software, so only LOCK_S and LOCK_BL gate
# us. A retail SeedHammer II ships with 0x040404 -- i.e. LOCK_NS=1, merely
# restricting Non-secure software to reads. An earlier version of this
# function required all-zero and would have declared that device permanently
# unusable. Verified against real hardware 2026-08-03.
PL_LINES=()
try_check_page_locks() {
  local l v b0 b1 b2 maj i bit lock_s lock_ns lock_bl key_w key_r nokey
  PL_LINES=()
  for l in PAGE1_LOCK0 PAGE1_LOCK1 PAGE2_LOCK0 PAGE2_LOCK1; do
    try_read_row_raw24 "$l" || return 1
    v="$ROW_RAW24"
    b0=$(( 16#$v & 0xff )); b1=$(( (16#$v >> 8) & 0xff )); b2=$(( (16#$v >> 16) & 0xff ))
    # Majority-vote the three copies bit by bit.
    maj=0
    for i in 0 1 2 3 4 5 6 7; do
      bit=$(( ((b0>>i)&1) + ((b1>>i)&1) + ((b2>>i)&1) ))
      [ "$bit" -ge 2 ] && maj=$(( maj | (1<<i) ))
    done
    if ! { [ "$b0" = "$b1" ] && [ "$b1" = "$b2" ]; }; then
      PL_LINES+=("warn	$l copies disagree (0x$(printf %02x $b0)/0x$(printf %02x $b1)/0x$(printf %02x $b2)); using majority 0x$(printf %02x $maj)")
    fi
    case "$l" in
      PAGE1_LOCK1|PAGE2_LOCK1)
        lock_s=$((  maj & 0x3 ));  lock_ns=$(( (maj >> 2) & 0x3 )); lock_bl=$(( (maj >> 4) & 0x3 ))
        [ "$lock_s" -eq 0 ] || { ERR="$l: LOCK_S=$lock_s -- Secure software may NOT write this page.
picotool writes as Secure software, so no further boot key can be added to this
device. This procedure is impossible on it. STOP."; return 1; }
        [ "$lock_bl" -eq 0 ] || { ERR="$l: LOCK_BL=$lock_bl -- the bootloader does not permit user writes
to this page. No further boot key can be added. STOP."; return 1; }
        PL_LINES+=("info	$l = 0x$v (LOCK_S=$lock_s writable, LOCK_BL=$lock_bl writable, LOCK_NS=$lock_ns)")
        ;;
      *)
        # PAGE*_LOCK0: KEY_W (bits 0-2), KEY_R (bits 3-5), NO_KEY_STATE (bit 6).
        # Only KEY_R/KEY_W gate us -- they demand a hardware OTP key we cannot
        # supply. NO_KEY_STATE only matters once a key is registered, so it is
        # informational; a bare all-zero test here would STOP on a harmless
        # device, the same shape as the page-lock bug that nearly killed this.
        key_w=$(( maj & 0x7 )); key_r=$(( (maj >> 3) & 0x7 )); nokey=$(( (maj >> 6) & 0x1 ))
        { [ "$key_r" -eq 0 ] && [ "$key_w" -eq 0 ]; } \
          || { ERR="$l = 0x$v -- this page requires a hardware OTP key (KEY_R=$key_r KEY_W=$key_w).
We cannot supply one. This procedure cannot proceed on this device. STOP."; return 1; }
        [ "$nokey" -eq 0 ] || PL_LINES+=("warn	$l: NO_KEY_STATE=1 (informational; no key is registered)")
        PL_LINES+=("info	$l = 0x$v (KEY_R=0 KEY_W=0 -- no hardware key required)")
        ;;
    esac
  done
}

# try_key_hash <key.pem> -> KEY_HASH: sha256 of the UNCOMPRESSED 64-byte X||Y
# pubkey. This is what the RP2350 stores in a boot-key slot. Computed
# independently of picotool so the two can be cross-checked.
KEY_HASH=""
try_key_hash() {
  local f full h errtxt
  KEY_HASH=""
  [ -f "$1" ] || { ERR="key_hash: no such key file: $1"; return 1; }
  [ -r "$1" ] || { ERR="key_hash: $1 exists but is not readable by you.
(Owned by another user? A root-owned file left by running something under sudo?)"; return 1; }
  # Discriminate 'openssl cannot read this at all' from 'wrong curve'. Collapsing
  # both into 'not secp256k1' sends the operator off regenerating a perfectly
  # good key when the real fault is permissions, a passphrase, or a bad format.
  errtxt="$(openssl ec -in "$1" -noout -text 2>&1 >/dev/null)" || { ERR="key_hash: openssl cannot read $1 as an EC private key.
This is NOT a curve problem -- do not regenerate yet. openssl said:
$errtxt
(Passphrase-protected? Wrong file? A public key rather than a private one?)"; return 1; }
  printf '%s' "$errtxt" | grep -qi 'ASN1 OID: secp256k1' \
    || openssl ec -in "$1" -noout -text 2>/dev/null | grep -qi 'ASN1 OID: secp256k1' \
    || { ERR="key_hash: $1 is NOT a secp256k1 key.
RP2350 secure boot requires secp256k1. EVERY EC curve yields a 64-byte slice
that looks superficially valid, so without this check a P-256 or P-384 key would
pass --make-otp-json AND --sh2-verify-slot, get burned into OTP, and never match
any signature you could produce -- permanently spending a slot."; return 1; }

  # Capture the point ONCE and hash exactly the bytes that were validated, so
  # what is asserted is what is burned.
  f="$(mktemp)"
  openssl ec -in "$1" -pubout -conv_form uncompressed -outform DER 2>/dev/null \
    | tail -c 65 > "$f" || { rm -f "$f"; ERR="key_hash: openssl failed on $1"; return 1; }
  # secp256k1 uncompressed point: exactly 65 bytes, 0x04 || X(32) || Y(32).
  [ "$(stat -c%s "$f")" -eq 65 ] \
    || { ERR="key_hash: expected a 65-byte uncompressed point from $1, got $(stat -c%s "$f") bytes"; rm -f "$f"; return 1; }
  full="$(od -An -v -tx1 -N1 "$f" | tr -d ' \n')"
  [ "$full" = "04" ] \
    || { rm -f "$f"; ERR="key_hash: public key is not uncompressed (leading byte 0x$full, expected 0x04)"; return 1; }
  # The OTP value is sha256 over X||Y -- the 64 bytes AFTER the 0x04 prefix.
  h="$(tail -c 64 "$f" | sha256sum | cut -d' ' -f1)" || { rm -f "$f"; ERR="key_hash: hashing failed for $1"; return 1; }
  rm -f "$f"
  [ ${#h} -eq 64 ] || { ERR="key_hash: sha256 produced ${#h} chars for $1"; return 1; }
  KEY_HASH="$h"
}

# --------------------------------------------------------------------------
# 3b. The strict single-row reader (refugium-otp.sh)
#
# Layout (G-facts, design/agent-reports/e3a-g-facts.md section 2, from
# main.cpp PT:9162-9330 compiled with fake OTP contents):
#   - RAW_VALUE= prints only when copies differ (vote branch) or an ECC row
#     fails its check, never under -c 1; it may be preceded on the same line by
#     one or more "(flipping raw value to 0x%08x)" notes;
#   - stdout is word-wrapped at 80 columns when it is not a terminal (F18): a
#     continuation line starts at column 14; an 8-copy RAW_VALUE list is ONE
#     overlong line whose warning may follow with no space;
#   - a selector matching nothing prints nothing and exits 0, so the ROW
#     header is required, never the exit code alone.
# --------------------------------------------------------------------------

# og_join <text>: join continuation lines (>= 14 leading spaces) to the line
# before them, with one space.
og_join() {
  printf '%s\n' "$1" | awk '
    NR == 1 { prev = $0; next }
    /^              [^ ]/ { s = $0; sub(/^ +/, "", s); prev = prev " " s; next }
    { print prev; prev = $0 }
    END { if (NR > 0) print prev }'
}

# og_parse <text> <row> <want-name> [width]
#   want-name: OTP_DATA_<NAME>, "" (the row must be unnamed) or "*" (either)
#   width: hex digits VALUE must have, 6 (default) or 4
# -> OG_VALUE (hex digits), OG_WARN (0/1), OG_RAW (space-separated 0x%06x
#    list, empty when absent), OG_FLIP (0/1), OG_NAME
og_parse() {
  local text="$1" row=$(( $2 )) want="$3" width="${4:-6}"
  local joined line nrow=0 nval=0 nraw=0 hrow
  OG_VALUE=""; OG_WARN=0; OG_RAW=""; OG_FLIP=0; OG_NAME=""
  joined="$(og_join "$text")"
  if [[ ${joined,,} == *warning* ]]; then OG_WARN=1; fi
  while IFS= read -r line; do
    if [[ "$line" == *"(flipping raw value to 0x"* ]]; then
      OG_FLIP=1
      line="$(printf '%s' "$line" | sed -E 's/\(flipping raw value to 0x[0-9a-f]{8}\)//g')"
    fi
    if [[ "$line" =~ ^[[:space:]]*$ ]]; then continue; fi
    if [[ "$line" =~ ^ROW\ 0x([0-9a-f]{4})(:\ (OTP_DATA_[A-Z0-9_]+))?(\ \([^\)]*\))*$ ]]; then
      nrow=$((nrow+1)); hrow=$((16#${BASH_REMATCH[1]})); OG_NAME="${BASH_REMATCH[3]}"
      continue
    fi
    if [[ "$line" =~ ^\ +RAW_VALUE=(0x[0-9a-f]{6}(\;0x[0-9a-f]{6})*)\ ?\(WARNING\ -\ [A-Z\ \']+\)$ ]]; then
      nraw=$((nraw+1)); OG_RAW="${BASH_REMATCH[1]//;/ }"
      continue
    fi
    if [[ "$line" =~ ^\ {4}VALUE\ 0x([0-9a-f]+)$ ]]; then
      nval=$((nval+1)); OG_VALUE="${BASH_REMATCH[1]}"
      continue
    fi
    if [[ "$line" =~ ^\ {4}field\ [A-Z0-9_]+\ \(bits?\ [0-9-]+\)\ =\ [0-9a-f]+$ ]]; then continue; fi
    # The fake's SUPPRESS_RAW_VALUE prints the warning on a line of its own;
    # real picotool never does, but it is no less a warning.
    if [[ "$line" =~ ^\ +\(WARNING\ -\ [A-Z\ \']+\)$ ]]; then continue; fi
    ERR="unexpected line in picotool output for row $(printf '0x%03x' "$row"): '$line'"
    return 1
  done <<<"$joined"
  [ "$nrow" -eq 1 ] || { ERR="expected one ROW header for row $(printf '0x%03x' "$row"), got $nrow:
$text"; return 1; }
  [ "$hrow" -eq "$row" ] || { ERR="asked for row $(printf '0x%03x' "$row"), picotool printed row $(printf '0x%03x' "$hrow")"; return 1; }
  if [ "$want" != "*" ] && [ "$OG_NAME" != "$want" ]; then
    ERR="row $(printf '0x%03x' "$row") printed as '${OG_NAME:-unnamed}', expected '${want:-unnamed}'"; return 1
  fi
  [ "$nraw" -le 1 ] || { ERR="more than one RAW_VALUE line for row $(printf '0x%03x' "$row")"; return 1; }
  [ "$nval" -eq 1 ] || { ERR="expected one VALUE line for row $(printf '0x%03x' "$row"), got $nval"; return 1; }
  [ "${#OG_VALUE}" -eq "$width" ] || { ERR="VALUE for row $(printf '0x%03x' "$row") has ${#OG_VALUE} hex digits, expected $width (an ECC-decoded read where a raw one was asked for?)"; return 1; }
  return 0
}

# try_og_row <flags> <row> <want-name>: one `otp get [flags] -n --ser S <row>`
# parsed by og_parse. flags: "" | "-c 1" | "-r". -> OG_* as og_parse, and
# OG_INT (VALUE as an integer).
try_og_row() {
  local flags="$1" row="$2" want="$3" sel
  local -a fl=()
  OG_INT=""
  case "$flags" in
    "") ;;
    "-c 1") fl=(-c 1) ;;
    "-r") fl=(-r) ;;
    *) ERR="try_og_row: unsupported flags '$flags'"; return 1 ;;
  esac
  sel="$(printf '0x%03x' "$row")"
  pt_build_otp_get "$PT_SER" "${fl[@]}" -n -- "$sel" || return 1
  pt_exec
  [ "$PT_RC" -eq 0 ] || { ERR="picotool exited $PT_RC reading row $sel${flags:+ ($flags)}:
$PT_OUT"; return 1; }
  og_parse "$PT_OUT" "$row" "$want" 6 || return 1
  OG_INT=$((16#$OG_VALUE))
}

# --------------------------------------------------------------------------
# 3c. Arithmetic shared with capture and replay
# --------------------------------------------------------------------------

# ecc16 <data16> -> the 22-bit ECC row picotool computes (port of
# otp_calculate_ecc, PT:5469-5486).
ecc16() {
  local x=$(( $1 & 0xffff )) p0 p1 p2 p3 p4 p5
  par() { local v=$1 c=0; while [ "$v" -ne 0 ]; do c=$((c ^ (v & 1))); v=$((v >> 1)); done; printf '%d' "$c"; }
  p0=$(par $(( x & 0xad5b ))); p1=$(par $(( x & 0x366d ))); p2=$(par $(( x & 0xc78e )))
  p3=$(par $(( x & 0x07f0 ))); p4=$(par $(( x & 0xf800 )))
  p5=$(( $(par "$x") ^ p0 ^ p1 ^ p2 ^ p3 ^ p4 ))
  printf '%d' $(( x | ((p0 | (p1<<1) | (p2<<2) | (p3<<3) | (p4<<4) | (p5<<5)) << 16) ))
}

# vote <crit 0|1> <v...> -> picotool's per-bit vote (PT:9263-9276):
# set if sets >= clears, or for a CRIT register if sets >= 3.
vote() {
  local crit="$1" b v sets clears out=0; shift
  for b in $(seq 0 23); do
    sets=0; clears=0
    for v in "$@"; do if (( (v >> b) & 1 )); then sets=$((sets+1)); else clears=$((clears+1)); fi; done
    if [ "$sets" -ge "$clears" ] || { [ "$crit" = 1 ] && [ "$sets" -ge 3 ]; }; then out=$(( out | (1 << b) )); fi
  done
  printf '%d' "$out"
}

# og_replay_dir <dir>: <dir>/manifest.tsv lists transcripts (written by
# `refugium-otp.sh capture`) as
#   file <TAB> row <TAB> name <TAB> width <TAB> value <TAB> warn <TAB> raw
# with value/raw computed from the per-copy reads. Each transcript must parse
# (og_parse) to exactly those values. Prints REPLAY PASS or MISMATCH lines.
og_replay_dir() {
  local dir="$1" f row name width value warn raw n=0 bad=0
  [ -f "$dir/manifest.tsv" ] || { echo "MISMATCH: no manifest.tsv in $dir"; return 1; }
  while IFS=$'\t' read -r f row name width value warn raw; do
    [ -n "$f" ] || continue
    n=$((n+1))
    if ! og_parse "$(cat "$dir/$f")" "$((16#${row#0x}))" "$name" "$width"; then
      echo "MISMATCH: $f does not parse: $ERR"; bad=$((bad+1)); continue
    fi
    if [ "$((16#$OG_VALUE))" -ne "$((16#${value#0x}))" ] || [ "$OG_WARN" != "$warn" ] || [ "$OG_RAW" != "$raw" ]; then
      echo "MISMATCH: $f parsed VALUE 0x$OG_VALUE warn=$OG_WARN raw='$OG_RAW'; recorded $value warn=$warn raw='$raw'"
      bad=$((bad+1))
    fi
  done < "$dir/manifest.tsv"
  [ "$n" -gt 0 ] || { echo "MISMATCH: empty manifest in $dir"; return 1; }
  [ "$bad" -eq 0 ] || return 1
  echo "REPLAY PASS: $n transcript(s) in $dir"
}
