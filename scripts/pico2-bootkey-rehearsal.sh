#!/usr/bin/env bash
#
# pico2-bootkey-rehearsal.sh -- rehearse the SeedHammer II custom-boot-key
# procedure on a throwaway RP2350 board (Raspberry Pi Pico 2).
#
# Companion to design/RUNBOOK_custom_boot_key.md. Run to completion on a Pico
# BEFORE touching the SeedHammer II's OTP.
#
# WHAT THIS REHEARSES
#   The picotool OTP -> sign -> boot-acceptance chain: the only irreversible
#   part of the real procedure. It does NOT rehearse SeedHammer firmware
#   behavior -- our controller drives SeedHammer's display, steppers and NFC and
#   will not run meaningfully on any Pico. That is fine: the signature check
#   happens before any of that code runs.
#
# THE PROOF STRUCTURE (this is the point -- read it)
#   A blinking LED only proves "signature accepted" if rejection was first shown
#   to be possible. So the phases form an A/B across a single OTP write:
#     phase 3  your-key-signed image is REJECTED (your key is not burned yet)
#     phase 4  burn your key                       <-- the only change
#     phase 5  the SAME image is ACCEPTED
#   Without phase 3, a board that was never actually sealed would blink in
#   phase 5 and be recorded as a green rehearsal. Do not skip phase 3.
#
# BOARD IS CONSUMED
#   OTP is one-time on the Pico too. A full run burns 2 of 4 boot-key slots and
#   permanently enables secure boot. Budget one board per run. Use the PLAIN
#   Pico 2: on the Pico 2 W the LED sits behind the CYW43 chip, so machine.LED
#   does nothing and "no blink" becomes unreadable.
#
# SAFETY
#   Dry-run by default; --execute arms writes. Destructive phases additionally
#   require a typed confirmation. Every phase after 0 pins the physical board by
#   CHIPID and refuses to touch a different one -- including the SeedHammer II.
#
# Usage:
#   ./pico2-bootkey-rehearsal.sh --phase 0                # inventory + prep (safe)
#   ./pico2-bootkey-rehearsal.sh --phase 1 --execute      # seal board (DESTRUCTIVE)
#   ./pico2-bootkey-rehearsal.sh --phase 2                # verify sealed + locks
#   ./pico2-bootkey-rehearsal.sh --phase 3 --execute      # NEGATIVE control (no OTP)
#   ./pico2-bootkey-rehearsal.sh --phase 4 --execute      # burn your key (DESTRUCTIVE)
#   ./pico2-bootkey-rehearsal.sh --phase 5 --execute      # POSITIVE control (no OTP)
#   ./pico2-bootkey-rehearsal.sh --phase 6 --execute      # fallback control (no OTP)
#
# Phases 3/5/6 flash the board, so they need --execute to do anything real,
# even though they never write OTP.
#
# SH2 MODES (the real SeedHammer II -- all READ ONLY, no --execute needed):
#   ./pico2-bootkey-rehearsal.sh --sh2-precheck
#   ./pico2-bootkey-rehearsal.sh --sh2-verify-slot 1 --key /abs/sh2-boot-key.pem
#   ./pico2-bootkey-rehearsal.sh --sh2-verify-valid 1 --key /abs/sh2-boot-key.pem
#   ./pico2-bootkey-rehearsal.sh --make-otp-json --key K --slot 1 --out F   (no device)
#
# These exist because the phases above deliberately REFUSE the SeedHammer II
# (CHIPID pin + slot-0 tripwire), which had left the runbook instructing the
# operator to use verification code the script would not run against the real
# device -- so the actual procedure fell back to eyeballing byte-swapped hex.
# The SH2 modes invert the tripwire (they REQUIRE SeedHammer's key in slot 0)
# and contain no write path at all; the irreversible writes stay in the runbook,
# in the operator's hands.
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# Anchored to the repo, NOT the cwd: cross-phase state (keys, CHIPID, images)
# must not silently vanish when the operator runs from a different directory.
WORKDIR="${WORKDIR:-$REPO_ROOT/rehearsal-work}"
# SH2 state lives OUTSIDE rehearsal-work/. Every document teaches that the
# rehearsal directory is disposable ("the board is CONSUMED"), so a `git clean
# -xdf` or a tidy-up would silently delete the SeedHammer's CHIPID pin and the
# rehearsal keys that reject_rehearsal_key compares against -- turning two
# safety checks into no-ops that still print PASS.
SH2_DIR="${SH2_DIR:-$REPO_ROOT/sh2-state}"
SEEDHAMMER_DIR="${SEEDHAMMER_DIR:-$(cd "$REPO_ROOT/../seedhammer" 2>/dev/null && pwd || true)}"
EXECUTE=0
PHASE=""

# SeedHammer's production signing-key hash (cmd/controller/platform_sh2.go:70).
# Used as a tripwire: if a board presents this in slot 0, it is a real
# SeedHammer II and this script must never write to it.
SH_SIGNKEY_HASH="c8314536d6af61ac2e62e5991e3e4711629c54696ba8c4af08965a1d319a473b"

die()  { printf '\n\033[31mFAIL:\033[0m %s\n' "$*" >&2; exit 1; }
ok()   { printf '\033[32m  PASS:\033[0m %s\n' "$*"; }
info() { printf '\033[36m  ..\033[0m %s\n' "$*"; }
hdr()  { printf '\n\033[1m== %s ==\033[0m\n' "$*"; }
warn() { printf '\033[33m  !!\033[0m %s\n' "$*"; }

ok_done() {
  if [ "$EXECUTE" -eq 1 ]; then printf '\033[32m  PASS:\033[0m %s\n' "$*"
  else printf '\033[90m  [dry-run] nothing was done. With --execute this phase would: %s\033[0m\n' "$*"; fi
}

run() {
  if [ "$EXECUTE" -eq 1 ]; then printf '\033[33m  $ %s\033[0m\n' "$*"; "$@"
  else printf '\033[90m  [dry-run] %s\033[0m\n' "$*"; fi
}

confirm() {
  local word="$1"
  [ "$EXECUTE" -eq 1 ] || return 0
  printf '\n\033[31mThis writes OTP and CANNOT be undone.\033[0m Type %s to proceed: ' "$word"
  read -r reply
  [ "$reply" = "$word" ] || die "aborted at operator confirmation"
}

MODE=""; SH2_SLOT=""; SH2_KEY=""; JSON_OUT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --phase)          PHASE="${2:-}"; shift 2 ;;
    --execute)        EXECUTE=1; shift ;;
    --sh2-precheck)   MODE="sh2-precheck"; shift ;;
    --sh2-verify-slot) MODE="sh2-verify-slot"; SH2_SLOT="${2:-}"; shift 2 ;;
    --sh2-verify-valid) MODE="sh2-verify-valid"; SH2_SLOT="${2:-}"; shift 2 ;;
    --make-otp-json)  MODE="make-otp-json"; shift ;;
    --key)            SH2_KEY="${2:-}"; shift 2 ;;
    --slot)           SH2_SLOT="${2:-}"; shift 2 ;;
    --out)            JSON_OUT="${2:-}"; shift 2 ;;
    *) die "unknown argument: $1" ;;
  esac
done
[ -n "$PHASE" ] || [ -n "$MODE" ] || die "specify --phase <0..6>, or an SH2 mode:
  --sh2-precheck                       read-only survey of the SeedHammer II (runbook step 1)
  --sh2-verify-slot N --key <key.pem>  read-only slot readback (gates runbook step 3)
  --sh2-verify-valid N --key <key.pem> read-only post-write check (closes step 3)
  --make-otp-json --key <key.pem> --slot N --out <file>   no device access at all"

for t in picotool openssl sha256sum; do
  command -v "$t" >/dev/null || die "$t not found -- run inside 'nix develop' in the seedhammer fork"
done
# tinygo/go are only needed where images are built or signed.
case "$PHASE" in
  0|3|4|5|6)
    command -v tinygo >/dev/null || die "tinygo not found -- run inside 'nix develop'"
    command -v go     >/dev/null || die "go not found -- run inside 'nix develop'" ;;
esac
# make_otp_json seals a real RP2350 image, so --make-otp-json needs a builder
# too. Discovering that mid-SeedHammer-procedure as "blinky build failed" is
# exactly the improvisation these modes exist to remove.
case "$MODE" in
  make-otp-json)
    command -v tinygo >/dev/null || die "tinygo not found -- run inside 'nix develop'"
    command -v jq     >/dev/null || die "jq not found -- run inside 'nix develop'" ;;
esac
[ -n "$SEEDHAMMER_DIR" ] && [ -d "$SEEDHAMMER_DIR" ] \
  || die "set SEEDHAMMER_DIR to the seedhammer fork (provides cmd/picosign)"

mkdir -p "$WORKDIR"
# SH2 modes are read-only by construction; there is no armed variant, so the
# "add --execute to arm" banner would be actively misleading there.
if [ -n "$MODE" ]; then
  printf '\n\033[90m(read-only mode -- this cannot write to the device)\033[0m\n'
elif [ "$EXECUTE" -ne 1 ]; then
  printf '\n\033[90m(dry-run -- nothing will be written; add --execute to arm)\033[0m\n'
fi

# --------------------------------------------------------------------------
# OTP helpers.
#
# picotool's `otp get` output format is not contractually stable, so every
# parse here fails CLOSED: an unparseable or failed read is an error, never a
# zero. Phase 0 exercises read_slot() against the known-all-zero slots of a
# stock board, so the parser is validated before phase 4 depends on it.
# --------------------------------------------------------------------------

# Output format is taken from picotool 2.2.0 main.cpp otp_get_command::execute:
#   "ROW 0x%04x" [": " reg->name] [" (ECC)"/" (CRIT)"/" (RBIT-n)"] ["(Part i/n)"]
#   "\nVALUE 0x%06x\n"                     <- the WHOLE 24-bit row
#   "field <NAME> (bit n|bits n-m) = %x"   <- the field, BARE hex, no 0x prefix
# So a field must be read from the `field ... = ` line, never from VALUE (which
# includes unrelated bits of the same row, e.g. KEY_INVALID alongside KEY_VALID).

# otp_field <selector> -> field value as bare lowercase hex
otp_field() {
  local sel="$1" out v
  out="$(picotool otp get -n "$sel" 2>&1)" || die "OTP read failed for $sel:
$out"
  # CRIT1 is RBIT-8 and BOOT_FLAGS1 is RBIT-3; an inconsistent redundant read
  # must not be parsed as a clean value.
  printf '%s' "$out" | grep -qi 'WARNING' \
    && die "picotool reported a warning reading $sel (redundant rows disagree or ECC invalid):
$out"
  v="$(printf '%s\n' "$out" | grep -iE '^[[:space:]]*field ' | tail -1 \
       | sed -E 's/.*=[[:space:]]*//' | tr -d '[:space:]' | tr 'A-F' 'a-f')"
  [ -n "$v" ] || die "could not parse a field value for $sel from picotool output:
$out"
  printf '%s' "$v" | grep -qE '^[0-9a-f]+$' || die "unexpected field value '$v' for $sel"
  printf '%s' "$v"
}


# read_rows <selector...> -> sets ROWVALS to space-separated 4-hex-digit values,
# one per requested row, in order.
#
# ONE picotool invocation for all rows, deliberately. Querying a single row that
# is part of a sequence ("Part 2/4") can print NOTHING with exit 0 -- observed on
# real hardware for CHIPID1, where the row is readable but only when requested
# alongside its siblings. A per-row loop silently lost it.
#
# Sets a global instead of printing: `die` inside $(...) exits only the
# subshell, so a fail-closed guard called via command substitution was not
# actually closed.
ROWVALS=""
read_rows() {
  local out want=$# got
  out="$(picotool otp get -n -e "$@" 2>&1)" || die "OTP read failed for: $*
$out"
  printf '%s' "$out" | grep -qi 'WARNING' \
    && die "picotool reported a warning reading: $*
$out"
  ROWVALS="$(printf '%s\n' "$out" | grep -oiE '^[[:space:]]*VALUE 0x[0-9a-f]+' \
             | grep -oiE '0x[0-9a-f]+' | sed 's/^0[xX]//' | tr 'A-F' 'a-f' \
             | while read -r v; do printf '%04x ' $(( 16#$v & 0xffff )); done)"
  got="$(printf '%s' "$ROWVALS" | wc -w)"
  [ "$got" -eq "$want" ] || die "expected $want row values for '$*', parsed $got.
picotool output was:
$out"
  # picotool returns rows ASCENDING-SORTED with silent dedup (its filter_otp
  # builds a std::map keyed by row) -- NOT in request order. Positional mapping
  # is therefore only valid for distinct ascending selectors. Verify the row
  # names it echoed match what we asked for, in order, rather than assume it.
  local names
  names="$(printf '%s\n' "$out" | grep -oiE '^ROW 0x[0-9a-f]+: OTP_DATA_[A-Z0-9_]+' \
           | sed -E 's/.*OTP_DATA_//')"
  [ "$(printf '%s\n' "$names" | wc -l)" -eq "$want" ] \
    || die "parsed $(printf '%s\n' "$names" | wc -l) ROW names for '$*', expected $want"
  local k=1 sel
  for sel in "$@"; do
    [ "$(printf '%s\n' "$names" | sed -n "${k}p")" = "$sel" ] \
      || die "picotool returned rows in an unexpected order.
  requested position $k: $sel
  received:            $(printf '%s\n' "$names" | sed -n "${k}p")
Positional mapping is unsafe here; refusing to guess which value is which."
    k=$((k+1))
  done
}

# read_slot <n> -> sets SLOT_HEX to 64 hex chars (the 32-byte key hash).
# Each row holds two bytes, LOW BYTE FIRST, so each row is byte-swapped.
SLOT_HEX=""
read_slot() {
  local n="$1" i sels="" v
  for i in $(seq 0 15); do sels="$sels BOOTKEY${n}_${i}"; done
  # shellcheck disable=SC2086
  read_rows $sels
  SLOT_HEX=""
  for v in $ROWVALS; do SLOT_HEX="${SLOT_HEX}${v:2:2}${v:0:2}"; done
  [ ${#SLOT_HEX} -eq 64 ] || die "slot $n reassembled to ${#SLOT_HEX} hex chars, expected 64"
}

# key_hash <key.pem> -> sha256 of the UNCOMPRESSED 64-byte X||Y pubkey.
# This is what the RP2350 stores in a boot-key slot. Computed independently of
# picotool so the two can be cross-checked.
key_hash() {
  local der h
  der="$(openssl ec -in "$1" -pubout -conv_form uncompressed -outform DER 2>/dev/null \
         | tail -c 64 | od -An -v -tx1 | tr -d ' \n')" \
    || die "key_hash: openssl failed on $1"
  # 64 bytes = 128 hex chars. Without this, an unreadable PEM yields sha256 of
  # the empty string (e3b0c442...) -- a deterministic WRONG value in a path that
  # decides what gets burned into OTP.
  [ ${#der} -eq 128 ] || die "key_hash: could not extract a 64-byte public key from $1
(got ${#der} hex chars). Is it a valid secp256k1 private key?"
  # No xxd: `nix develop` prepends rather than purges PATH, so xxd would
  # silently resolve to a host binary the devshell does not provide.
  h="$(openssl ec -in "$1" -pubout -conv_form uncompressed -outform DER 2>/dev/null \
       | tail -c 64 | sha256sum | cut -d' ' -f1)"
  printf '%s' "$h"
}

CHIPID_HEX=""
chipid() {
  local out=""
  read_rows CHIPID0 CHIPID1 CHIPID2 CHIPID3
  for v in $ROWVALS; do out="${out}${v}"; done
  # CHIPID is factory-programmed and unique; all-zeros means an unprogrammed
  # part or a simulator. Never pin or compare against it -- a stale all-zero
  # pin left by a test would make the real device look like the wrong device.
  case "$out" in
    0000000000000000) die "CHIPID reads all zeros.
That is not a real RP2350 identity -- it means an unprogrammed part, or that
picotool is talking to a simulator rather than hardware. Refusing to pin or
trust this identity." ;;
  esac
  [ ${#out} -eq 16 ] || die "CHIPID reassembled to ${#out} hex chars, expected 16"
  CHIPID_HEX="$out"
}

# require_board: refuse to act on any board other than the one phase 0 saw.
require_board() {
  local pinned cur
  [ -f "$WORKDIR/board-chipid.txt" ] || die "no pinned board -- run phase 0 first"
    pinned="$(cat "$WORKDIR/board-chipid.txt")"
  chipid; cur="$CHIPID_HEX"
  [ "$cur" = "$pinned" ] || die "WRONG BOARD.
  pinned (phase 0): $pinned
  connected now:    $cur
Refusing to touch a board this rehearsal did not inventory."
  ok "board identity matches phase 0 ($cur)"

  # Independent tripwire: never write to a real SeedHammer II.
  local slot0; read_slot 0; slot0="$SLOT_HEX"
  [ "$slot0" != "$SH_SIGNKEY_HASH" ] \
    || die "SLOT 0 HOLDS SEEDHAMMER'S PRODUCTION KEY -- this is a real SeedHammer II.
This script is for the throwaway rehearsal board ONLY. Disconnect it now."
}

assert_stock_or_die() {
  local sb kv i slot
  sb="$(otp_field CRIT1.SECURE_BOOT_ENABLE)"
  kv="$(otp_field BOOT_FLAGS1.KEY_VALID)"
  [ $((16#$sb)) -eq 0 ] || die "SECURE_BOOT_ENABLE is already set (0x$sb) -- board already sealed. Get a FRESH board."
  ok "secure boot not yet enabled"
  [ $((16#$kv)) -eq 0 ] || die "KEY_VALID is already 0x$kv -- a boot key is already valid. Get a FRESH board."
  ok "no boot key marked valid"
  for i in 0 1 2 3; do
    read_slot "$i"; slot="$SLOT_HEX"
    [ "$slot" = "$(printf '0%.0s' $(seq 1 64))" ] \
      || die "boot-key slot $i is not empty (0x$slot) -- board already used. Get a FRESH board."
  done
  ok "all four boot-key slots empty (all 16 rows each, parser validated)"
}

# Page-lock rows are ecc=false in picotool's table, so they must be read WITHOUT
# -e and kept at full 24 bits: a genuinely locked page would decode as
# ECC-invalid and be misreported as a read warning rather than as "LOCKED",
# and masking to 16 bits would discard the third lock copy in bits 23:16.
read_row_raw24() {
  local sel="$1" out v
  out="$(picotool otp get -n "$sel" 2>&1)" || die "OTP read failed for $sel:
$out"
  v="$(printf '%s\n' "$out" | grep -oiE '^[[:space:]]*VALUE 0x[0-9a-f]+' | tail -1 \
       | grep -oiE '0x[0-9a-f]+' | sed 's/^0[xX]//' | tr 'A-F' 'a-f')"
  [ -n "$v" ] || die "could not parse a VALUE line for row $sel:
$out"
  printf '%06x' $(( 16#$v & 0xffffff ))
}

check_page_locks() {
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
  local l v b0 b1 b2 maj i bit lock_s lock_ns lock_bl
  for l in PAGE1_LOCK0 PAGE1_LOCK1 PAGE2_LOCK0 PAGE2_LOCK1; do
    v="$(read_row_raw24 "$l")"
    b0=$(( 16#$v & 0xff )); b1=$(( (16#$v >> 8) & 0xff )); b2=$(( (16#$v >> 16) & 0xff ))
    # Majority-vote the three copies bit by bit.
    maj=0
    for i in 0 1 2 3 4 5 6 7; do
      bit=$(( ((b0>>i)&1) + ((b1>>i)&1) + ((b2>>i)&1) ))
      [ "$bit" -ge 2 ] && maj=$(( maj | (1<<i) ))
    done
    [ "$b0" = "$b1" ] && [ "$b1" = "$b2" ] \
      || warn "$l copies disagree (0x$(printf %02x $b0)/0x$(printf %02x $b1)/0x$(printf %02x $b2)); using majority 0x$(printf %02x $maj)"

    case "$l" in
      PAGE1_LOCK1|PAGE2_LOCK1)
        lock_s=$((  maj & 0x3 ));  lock_ns=$(( (maj >> 2) & 0x3 )); lock_bl=$(( (maj >> 4) & 0x3 ))
        [ "$lock_s" -eq 0 ] || die "$l: LOCK_S=$lock_s -- Secure software may NOT write this page.
picotool writes as Secure software, so no further boot key can be added to this
device. This procedure is impossible on it. STOP."
        [ "$lock_bl" -eq 0 ] || die "$l: LOCK_BL=$lock_bl -- the bootloader does not permit user writes
to this page. No further boot key can be added. STOP."
        info "$l = 0x$v (LOCK_S=$lock_s writable, LOCK_BL=$lock_bl writable, LOCK_NS=$lock_ns)"
        ;;
      *)
        # PAGE*_LOCK0: KEY_W (bits 0-2), KEY_R (bits 3-5), NO_KEY_STATE (bit 6).
        # Only KEY_R/KEY_W gate us -- they demand a hardware OTP key we cannot
        # supply. NO_KEY_STATE only matters once a key is registered, so it is
        # informational; a bare all-zero test here would STOP on a harmless
        # device, the same shape as the page-lock bug that nearly killed this.
        local key_w=$(( maj & 0x7 )) key_r=$(( (maj >> 3) & 0x7 )) nokey=$(( (maj >> 6) & 0x1 ))
        [ "$key_r" -eq 0 ] && [ "$key_w" -eq 0 ] \
          || die "$l = 0x$v -- this page requires a hardware OTP key (KEY_R=$key_r KEY_W=$key_w).
We cannot supply one. This procedure cannot proceed on this device. STOP."
        [ "$nokey" -eq 0 ] || warn "$l: NO_KEY_STATE=1 (informational; no key is registered)"
        info "$l = 0x$v (KEY_R=0 KEY_W=0 -- no hardware key required)"
        ;;
    esac
  done
  ok "OTP page permissions allow Secure writes -- boot-key rows are writable"
}


# build_blinky: the phase-4 payload AND the seal input for phases 1/4.
# picotool seal requires a real RP2350 image, so there is no 'placeholder.elf'.
build_blinky() {
  [ -f "$WORKDIR/blinky.uf2" ] && return 0
  info "building rehearsal blinky (-target pico2)"
  ( cd "$REPO_ROOT/scripts/rehearsal-blinky" \
    && tinygo build -o "$WORKDIR/blinky.uf2" -target pico2 -opt 2 . ) \
    || die "blinky build failed"
  [ -f "$WORKDIR/blinky.uf2" ] || die "blinky build produced no image"
}

# make_otp_json <key.pem> <slot> <out.json>
# Scripts the edit the runbook used to ask the operator to do by hand, then
# asserts the result: exactly one top-level key, correct slot, 32 bytes, and
# byte-equal to the independently computed hash.
make_otp_json() {
  local key="$1" slot="$2" out="$3" raw="$WORKDIR/.seal-raw.json" expect
  expect="$(key_hash "$key")"
  build_blinky
  picotool seal --sign --quiet "$WORKDIR/blinky.uf2" "$WORKDIR/.seal-discard.uf2" "$key" "$raw" \
    || die "picotool seal failed"
  command -v jq >/dev/null || die "jq not found -- needed to transform the OTP json safely"
  jq --arg s "bootkey$slot" '{($s): .bootkey0}' "$raw" > "$out" \
    || die "failed to transform otp json"
  # Assertions before this file is ever fed to `otp load`.
  [ "$(jq -r 'keys | length' "$out")" = "1" ] || die "otp json must have exactly one key"
  [ "$(jq -r 'keys[0]' "$out")" = "bootkey$slot" ] || die "otp json targets the wrong slot"
  [ "$(jq -r ".\"bootkey$slot\" | length" "$out")" = "32" ] || die "otp json key is not 32 bytes"
  jq -e 'has("crit1") or has("boot_flags1")' "$out" >/dev/null \
    && die "otp json still contains crit1/boot_flags1 -- would seal before verification"
  local got; got="$(jq -r ".\"bootkey$slot\"[]" "$out" | awk '{printf "%02x",$1}')"
  [ "$got" = "$expect" ] \
    || die "otp json hash does not match the independently computed key hash:
  json:     $got
  openssl:  $expect"
  ok "otp json validated: single key, slot $slot, 32 bytes, matches openssl-derived hash"
  rm -f "$WORKDIR/.seal-discard.uf2" "$raw"
}

# verify_slot_or_die <slot> <key.pem> -- the automated 16-row readback.
verify_slot_or_die() {
  local slot="$1" key="$2" expect got
  expect="$(key_hash "$key")"
  read_slot "$slot"; got="$SLOT_HEX"
  [ "$got" = "$expect" ] || die "SLOT $slot READBACK MISMATCH -- DO NOT SET THE VALID BIT.
  expected: $expect
  read:     $got
The slot is not yet valid, so the board still boots normally. Use the next
free slot instead; this one is permanently unusable."
  ok "slot $slot readback matches the expected hash across all 16 rows"
}

# Flashing and judging are SEPARATE on purpose. Folding them together (as an
# `if flash_and_ask ...` condition) disabled `set -e` for the whole body, so a
# failed `picotool load` fell through to the prompt: the operator saw no blink
# BECAUSE NOTHING WAS FLASHED, answered "n", and a rejection test reported PASS.

# flash_image <image> -- dies on any flash/reboot failure.
flash_image() {
  local img="$1"
  if [ "$EXECUTE" -ne 1 ]; then
    printf '\033[90m  [dry-run] picotool load --verify %s && picotool reboot\033[0m\n' "$img"
    return 0
  fi
  [ -f "$img" ] || die "image not found: $img"
  printf '\033[33m  $ picotool load --verify %s\033[0m\n' "$img"
  picotool load --verify "$img" || die "FLASH FAILED for $img.
Nothing was written to the board, so no boot verdict can be drawn. Fix the
flash (board connected? in BOOTSEL? image valid?) and re-run this phase."
  picotool reboot || die "REBOOT FAILED after flashing $img -- no verdict can be drawn."
}

# ask_blink <question> -> prints "yes" | "no" | "skip"  ("skip" only in dry-run)
#
# REPROMPTS until the answer is literally y or n. Defaulting a stray Enter to
# "no" was dangerous: "no" is the PASSING answer for both negative controls
# (3a/3b) and for 5b, so a mistyped or absent-minded response silently produced
# the exact false proof this rehearsal exists to prevent.
# Sets BLINK_ANSWER to yes|no|skip. Deliberately NOT a $(...)-called function:
# its die would exit only the subshell, and a case with no default then skipped
# the verdict silently and let the phase print its success banner.
BLINK_ANSWER=""
ask_blink() {
  BLINK_ANSWER=""
  if [ "$EXECUTE" -ne 1 ]; then BLINK_ANSWER="skip"; return 0; fi
  local a
  while true; do
    printf '\n\033[1m%s\033[0m [y/n]: ' "$1"
    read -r a || die "no answer given (stdin closed) -- cannot draw a verdict.
This verdict is load-bearing; refusing to guess."
    case "$a" in
      y|Y|yes|YES) BLINK_ANSWER="yes"; return 0 ;;
      n|N|no|NO)   BLINK_ANSWER="no";  return 0 ;;
      *) printf '  answer y or n exactly -- this verdict is load-bearing.\n' ;;
    esac
  done
}

# bootsel_present -> "yes" | "no" | "skip"
# Machine answer for 5b: after a reboot, a device still reachable by picotool is
# sitting in BOOTSEL, i.e. the bootrom REJECTED the image. A device that ran the
# image is not enumerable this way. Removes the one prompt where y meant failure.
BOOTSEL_ANSWER=""
bootsel_present() {
  BOOTSEL_ANSWER=""
  if [ "$EXECUTE" -ne 1 ]; then BOOTSEL_ANSWER="skip"; return 0; fi
  # Poll rather than sampling once: a REJECTED board that re-enumerates slowly
  # (hubs, some xHCI stacks) would miss a single 3s check and be scored as
  # "accepted" -- failing open on the only automated verdict in the rehearsal.
  # Require sustained absence before concluding the image ran.
  local i
  for i in $(seq 1 15); do
    if picotool info >/dev/null 2>&1; then
      info "picotool sees the board after ${i}s -- it is in BOOTSEL (image rejected)"
      BOOTSEL_ANSWER="yes"; return 0
    fi
    sleep 1
  done
  info "picotool could not see the board for 15s -- it is running the image"
  BOOTSEL_ANSWER="no"
}

# sign_image <image> <key> -> produces <image-without-.uf2>.signed.uf2
# sign-firmware.sh deliberately never mutates its input, so callers must flash
# the .signed.uf2 it produces, NOT the original.
sign_image() {
  SEEDHAMMER_DIR="$SEEDHAMMER_DIR" "$REPO_ROOT/scripts/sign-firmware.sh" "$1" "$2"
}

# --------------------------------------------------------------------------
# SH2 MODES (read-only)
#
# The rehearsal phases deliberately REFUSE the SeedHammer II: require_board pins
# the rehearsal Pico's CHIPID and the slot-0 tripwire rejects any board holding
# SeedHammer's production key. That is correct for the phases -- but it left the
# runbook telling the operator to "use read_slot/verify_slot_or_die here" against
# a device the script would not touch, so the real procedure fell back to
# eyeballing byte-swapped hex: exactly the Critical the rehearsal was built to
# eliminate. These modes close that gap.
#
# They invert the tripwire (they REQUIRE SeedHammer's key in slot 0) and contain
# no write path whatsoever -- no otp load, no otp set, no picotool load. The
# irreversible writes stay in the operator's hands, in the runbook.
# --------------------------------------------------------------------------

require_spare_slot() {
  case "${1:-}" in
    1|2|3) ;;
    0) die "slot 0 is SeedHammer's PRODUCTION key and the entire recovery path.
It must never be written. Use slot 1, 2 or 3." ;;
    *) die "slot must be 1, 2 or 3 (got '${1:-}')" ;;
  esac
}

sh2_require_seedhammer() {
  local slot0 cid
  picotool info >/dev/null 2>&1 || die "no board visible -- is the SeedHammer II in BOOTSEL?"
  picotool info | grep -qi 'rp2350' || die "not an RP2350 device"
  read_slot 0; slot0="$SLOT_HEX"
  [ "$slot0" = "$SH_SIGNKEY_HASH" ] || die "This is NOT a SeedHammer II.
Slot 0 holds: $slot0
Expected:     $SH_SIGNKEY_HASH  (signKeyHash, platform_sh2.go:70)
These modes are for the real device only. If you meant the rehearsal Pico, use
--phase instead. UNPLUG EVERY OTHER RP2350 BOARD before continuing."
  ok "slot 0 holds SeedHammer's production key -- this is a SeedHammer II"
  mkdir -p "$SH2_DIR"
  chipid; cid="$CHIPID_HEX"
  if [ -f "$SH2_DIR/sh2-chipid.txt" ]; then
    [ "$cid" = "$(cat "$SH2_DIR/sh2-chipid.txt")" ] || die "WRONG DEVICE.
  pinned at --sh2-precheck: $(cat "$SH2_DIR/sh2-chipid.txt")
  connected now:            $cid
A consumed rehearsal Pico can answer with plausible-looking values. Unplug it."
    ok "device identity matches the one pinned at --sh2-precheck ($cid)"
  elif [ "$MODE" = "sh2-precheck" ]; then
    printf '%s' "$cid" > "$SH2_DIR/sh2-chipid.txt"
    ok "pinned SeedHammer II CHIPID $cid for later steps"
  else
    # Only --sh2-precheck may create the pin. Letting a later mode create it
    # would silently downgrade the wrong-device check to a first-time pin and
    # still print PASS.
    die "no SeedHammer II pinned ($SH2_DIR/sh2-chipid.txt is missing).
Run --sh2-precheck first; it is what establishes the device identity that this
mode re-checks. If you deleted $SH2_DIR mid-procedure, re-run --sh2-precheck --
it is read-only -- but be aware it will refuse a device that is already part-way
through (see its message for what to do)."
  fi
}

# Refuse any key that came out of the rehearsal (I7): rehearsal-work/ is
# documented as disposable, so burning a throwaway key's hash into the SH2 would
# strand the slot the moment that directory is deleted.
reject_rehearsal_key() {
  local key="$1" h p seen=0
  if [ "${ALLOW_UNCHECKED_KEY:-0}" = "1" ]; then
    warn "ALLOW_UNCHECKED_KEY=1 -- rehearsal-key check SKIPPED by request"
    return 0
  fi
  h="$(key_hash "$key")"
  for p in "$WORKDIR"/factory-key.pem "$WORKDIR"/my-key.pem "$WORKDIR"/third-party-key.pem; do
    [ -f "$p" ] || continue
    seen=$((seen+1))
    [ "$h" != "$(key_hash "$p")" ] || die "REFUSING: $key is a REHEARSAL key ($(basename "$p")).
Rehearsal keys live in a directory documented as disposable. Burning one into the
SeedHammer II would permanently strand that slot the moment it is deleted.
Generate a distinct, backed-up key (e.g. sh2-boot-key.pem) for the real device."
  done
  # Printing PASS after comparing against nothing is exactly the false-green
  # this guard exists to prevent.
  [ "$seen" -gt 0 ] || die "CANNOT CHECK: no rehearsal keys found in $WORKDIR.
This guard compares your key against the rehearsal keys, and there are none to
compare against -- so it can neither confirm nor deny that $key is a throwaway.
Keep rehearsal-work/ until the SeedHammer procedure is complete. If the
rehearsal is genuinely finished and the directory is gone, satisfy yourself by
hand that this key is the one you generated for the real device and backed up,
then re-run with ALLOW_UNCHECKED_KEY=1."
  ok "key is not one of the $seen rehearsal key(s) on disk"
}

case "$MODE" in
  make-otp-json)
    hdr "make-otp-json (no device access)"
    [ -n "$SH2_KEY" ] && [ -n "$SH2_SLOT" ] && [ -n "$JSON_OUT" ] \
      || die "usage: --make-otp-json --key <key.pem> --slot <N> --out <file.json>"
    [ -f "$SH2_KEY" ] || die "no such key: $SH2_KEY"
    require_spare_slot "$SH2_SLOT"
    reject_rehearsal_key "$SH2_KEY"
    make_otp_json "$SH2_KEY" "$SH2_SLOT" "$JSON_OUT"
    ok "wrote $JSON_OUT for slot $SH2_SLOT"
    info "Load it with:  picotool otp load $JSON_OUT"
    info "Then GATE the valid bit on:  $0 --sh2-verify-slot $SH2_SLOT --key $SH2_KEY"
    exit 0 ;;

  sh2-precheck)
    hdr "SeedHammer II precheck (READ ONLY -- runbook step 1)"
    sh2_require_seedhammer

    SB="$(otp_field CRIT1.SECURE_BOOT_ENABLE)"
    [ $((16#$SB & 1)) -eq 1 ] || die "SECURE_BOOT_ENABLE is 0x$SB -- expected set on a retail unit.
Do not proceed; the device is not in the state this procedure assumes."
    ok "secure boot enabled"

    KV="$(otp_field BOOT_FLAGS1.KEY_VALID)"
    [ $((16#$KV)) -eq 1 ] || die "KEY_VALID is 0x$KV, expected exactly 0x1 (slot 0 only).

If you are PART-WAY THROUGH this procedure, that is expected and not a fault --
--sh2-precheck describes a pristine retail unit only. Use
    --sh2-verify-valid <slot> --key <your key>
to check a device whose slot is already burned.

Otherwise: more than one key is already valid on a device you believed pristine.
STOP and find out why."
    ok "exactly one valid boot key (slot 0)"

    KI="$(otp_field BOOT_FLAGS1.KEY_INVALID)"
    [ $((16#$KI)) -eq 0 ] || die "KEY_INVALID is 0x$KI -- a boot key has been REVOKED on this device. STOP."
    ok "no boot key revoked"

    for s in 1 2 3; do
      read_slot "$s"; v="$SLOT_HEX"
      [ "$v" = "$(printf '0%.0s' $(seq 1 64))" ] \
        || die "spare slot $s is NOT empty (0x$v).

If YOU burned it earlier in this procedure, that is expected -- use
    --sh2-verify-slot $s --key <your key>     (before the valid bit)
    --sh2-verify-valid $s --key <your key>    (after it)
rather than --sh2-precheck, which describes a pristine unit only.

Otherwise this device has been modified by someone else. STOP."
    done
    ok "spare slots 1-3 all empty (all 16 rows each)"

    check_page_locks

    # F11(c) / follow-up bootkey-rehearsal-fidelity-residue: CRIT1 is 8-way and
    # BOOT_FLAGS1 3-way redundant. Read the raw copies individually so a partial
    # write is visible; read_row_raw24 dies on any unreadable row.
    hdr "Redundant-row raw readback (CRIT1 x8, BOOT_FLAGS1 x3)"
    # Compared here, not eyeballed. Raw row-number reads resolve without a
    # register match, so picotool takes the no-redundancy path and can NEVER
    # print a disagreement warning -- an earlier version told the operator it
    # would, which was assurance the tool cannot give. Plain assignments (not
    # printf arguments) so a failed read's `die` actually halts.
    FIRST=""; MISMATCH=0
    for r in 0x040 0x041 0x042 0x043 0x044 0x045 0x046 0x047; do
      V="$(read_row_raw24 "$r")"
      printf '  CRIT1 copy %s = 0x%s\n' "$r" "$V"
      [ -n "$FIRST" ] || FIRST="$V"
      [ "$V" = "$FIRST" ] || MISMATCH=1
    done
    [ "$MISMATCH" -eq 0 ] || die "CRIT1's 8 redundant copies DISAGREE (see above).
The hardware majority-votes these, so the device may still behave correctly, but
a critical boot flag with degraded redundancy on a machine you are about to
modify is not something to proceed past. STOP."
    ok "all 8 CRIT1 copies agree (0x$FIRST)"

    FIRST=""; MISMATCH=0
    for r in 0x04b 0x04c 0x04d; do
      V="$(read_row_raw24 "$r")"
      printf '  BOOT_FLAGS1 copy %s = 0x%s\n' "$r" "$V"
      [ -n "$FIRST" ] || FIRST="$V"
      [ "$V" = "$FIRST" ] || MISMATCH=1
    done
    [ "$MISMATCH" -eq 0 ] || die "BOOT_FLAGS1's 3 redundant copies DISAGREE (see above).
This is the row holding KEY_VALID/KEY_INVALID. STOP."
    ok "all 3 BOOT_FLAGS1 copies agree (0x$FIRST)"

    hdr "RESULT"
    ok "SeedHammer II is in the expected retail state and its OTP pages are writable."
    info "Proceed to runbook step 2. Nothing was written."
    exit 0 ;;

  sh2-verify-valid)
    hdr "SeedHammer II post-write verification (READ ONLY -- closes runbook step 3)"
    [ -n "$SH2_SLOT" ] && [ -n "$SH2_KEY" ] \
      || die "usage: --sh2-verify-valid <N> --key <key.pem>"
    require_spare_slot "$SH2_SLOT"
    [ -f "$SH2_KEY" ] || die "no such key: $SH2_KEY"
    sh2_require_seedhammer
    reject_rehearsal_key "$SH2_KEY"

    # The hash rows (this also re-proves step 2 held).
    verify_slot_or_die "$SH2_SLOT" "$SH2_KEY"

    # The valid bit itself. otp_field dies on any picotool WARNING, which the
    # bare `picotool otp get` in earlier drafts did not -- a 2-of-3 redundant
    # write prints the EXPECTED value alongside a warning, so the naive check
    # passed while leaving BOOT_FLAGS1 permanently degraded.
    WANT=$(( 1 | (1 << SH2_SLOT) ))
    KV="$(otp_field BOOT_FLAGS1.KEY_VALID)"
    if [ $((16#$KV)) -ne "$WANT" ]; then
      die "KEY_VALID is 0x$KV, expected 0x$(printf '%x' "$WANT") (slot 0 + slot $SH2_SLOT).
If it reads LOWER than expected, the redundant write was INTERRUPTED partway.
That is recoverable and costs nothing: re-run the identical
    picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x$(printf '%x' $((1 << SH2_SLOT)))
It only ever SETS bits, so repeating it is safe. Do NOT burn another slot, and
do NOT start re-signing -- your key hash is already proven correct above."
    fi
    ok "KEY_VALID is 0x$KV (slot 0 + slot $SH2_SLOT)"

    KI="$(otp_field BOOT_FLAGS1.KEY_INVALID)"
    [ $((16#$KI)) -eq 0 ] || die "KEY_INVALID is 0x$KI -- a key has been revoked. STOP."
    ok "no key revoked"

    # All three BOOT_FLAGS1 copies must agree. A degraded-but-voting-correctly
    # row is exactly what the old bare check could not see.
    A="$(read_row_raw24 0x04b)"; B="$(read_row_raw24 0x04c)"; C="$(read_row_raw24 0x04d)"
    if [ "$A" != "$B" ] || [ "$B" != "$C" ]; then
      die "BOOT_FLAGS1 redundant copies DISAGREE: 0x$A / 0x$B / 0x$C.
picotool's majority vote may still report the right value, but the redundancy
protecting the bit that says 'trust your key' is degraded. Re-run the identical
`otp set -s` (set-bits only, safe to repeat) and re-check."
    fi
    ok "all three BOOT_FLAGS1 copies agree (0x$A)"

    hdr "RESULT"
    ok "Slot $SH2_SLOT is burned, verified, valid, and its flag rows are consistent."
    info "The OTP work is COMPLETE. Everything from here (build, sign, flash) is"
    info "freely retryable and costs no slots."
    exit 0 ;;

  sh2-verify-slot)
    hdr "SeedHammer II slot readback (READ ONLY -- gates runbook step 3)"
    [ -n "$SH2_SLOT" ] && [ -n "$SH2_KEY" ] \
      || die "usage: --sh2-verify-slot <N> --key <key.pem>"
    require_spare_slot "$SH2_SLOT"
    [ -f "$SH2_KEY" ] || die "no such key: $SH2_KEY"
    sh2_require_seedhammer
    reject_rehearsal_key "$SH2_KEY"
    verify_slot_or_die "$SH2_SLOT" "$SH2_KEY"
    hdr "RESULT"
    ok "Slot $SH2_SLOT matches $SH2_KEY across all 16 rows."
    info "ONLY NOW is it safe to set the valid bit (runbook step 3)."
    info "That write is irreversible; re-read this line before typing it."
    exit 0 ;;

  "") : ;;
  *) die "unknown mode: $MODE" ;;
esac

case "$PHASE" in

0)
  hdr "Phase 0 -- inventory, board pinning, and prep (READ ONLY + local builds)"
  info "Put the board in BOOTSEL: hold BOOTSEL while connecting USB."

  picotool info >/dev/null 2>&1 || die "board not visible -- is it in BOOTSEL mode?"
  picotool info | grep -qi 'rp2350' || die "not an RP2350 board -- wrong hardware for this rehearsal"
  ok "RP2350 detected"

  hdr "Current state"
  picotool otp get CRIT1.SECURE_BOOT_ENABLE || die "cannot read CRIT1.SECURE_BOOT_ENABLE (check 'picotool otp list')"
  picotool otp get BOOT_FLAGS1.KEY_VALID    || die "cannot read BOOT_FLAGS1.KEY_VALID (check 'picotool otp list')"

  hdr "Asserting the board is stock"
  assert_stock_or_die
  check_page_locks

  hdr "Pinning this physical board"
  chipid; CID="$CHIPID_HEX"
  printf '%s' "$CID" > "$WORKDIR/board-chipid.txt"
  ok "CHIPID $CID recorded -- later phases refuse any other board"

  hdr "Generating keys (outside any destructive phase, by design)"
  for k in factory-key my-key third-party-key; do
    if [ -f "$WORKDIR/$k.pem" ]; then
      info "$k.pem exists, keeping it"
    else
      openssl ecparam -name secp256k1 -genkey -noout -out "$WORKDIR/$k.pem"
      info "generated $k.pem"
    fi
    printf '  %-18s sha256(pubkey)=%s\n' "$k" "$(key_hash "$WORKDIR/$k.pem")"
  done
  info "factory-key plays SeedHammer's role; my-key is 'yours';"
  info "third-party-key is never burned -- it is the negative control in phase 3."

  hdr "Building the payload"
  build_blinky
  ok "blinky.uf2 ready ($(stat -c%s "$WORKDIR/blinky.uf2") bytes)"

  ok "Phase 0 complete -- board is stock and pinned. Proceed to phase 1."
  ;;

1)
  hdr "Phase 1 -- seal the board, mimicking SeedHammer's LockBoot() (DESTRUCTIVE)"
  info "Reproduces cmd/controller/platform_sh2.go:510-518 on the Pico:"
  info "  AddBootKey(factory key) + EnableSecureBoot()"
  require_board
  [ -f "$WORKDIR/factory-key.pem" ] || die "factory-key.pem missing -- run phase 0"

  # Resume support: aborting at the SET-VALID confirm leaves slot 0 burned but
  # not valid. That board is fine to continue on, but a plain stock-check would
  # tell the operator to throw it away.
  read_slot 0; SLOT0="$SLOT_HEX"
  EXPECT0="$(key_hash "$WORKDIR/factory-key.pem")"
  KV0="$(otp_field BOOT_FLAGS1.KEY_VALID)"
  RESUME=0
  if [ "$SLOT0" = "$EXPECT0" ] && [ $((16#$KV0)) -eq 0 ]; then
    RESUME=1
    warn "Slot 0 already holds this factory key but is NOT yet valid."
    warn "Resuming a previously aborted phase 1 -- skipping the burn, going to verify."
  else
    assert_stock_or_die
  fi

  if [ "$RESUME" -eq 0 ]; then
    hdr "1a -- build and validate the OTP json (no writes yet)"
    make_otp_json "$WORKDIR/factory-key.pem" 0 "$WORKDIR/factory-otp.json"

    hdr "1b -- burn the key hash into slot 0"
    confirm BURN-FACTORY
    run picotool otp load "$WORKDIR/factory-otp.json"
  fi

  hdr "1c -- verify all 16 rows BEFORE setting the valid bit"
  if [ "$EXECUTE" -eq 1 ]; then verify_slot_or_die 0 "$WORKDIR/factory-key.pem"
  else info "[dry-run] would read back slot 0 and compare to the expected hash"; fi

  hdr "1d -- mark valid and enable secure boot"
  confirm SET-VALID
  run picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x1
  run picotool otp set -s CRIT1.SECURE_BOOT_ENABLE 0x1

  ok_done "seal the board -- it would then boot only factory-key-signed images."
  ;;

2)
  hdr "Phase 2 -- verify the sealed state (read only)"
  require_board

  SB="$(otp_field CRIT1.SECURE_BOOT_ENABLE)"
  KV="$(otp_field BOOT_FLAGS1.KEY_VALID)"
  info "SECURE_BOOT_ENABLE=0x$SB  KEY_VALID=0x$KV"
  [ $((16#$SB & 1)) -eq 1 ] || die "secure boot is NOT enabled (0x$SB) -- phase 1 did not take. Everything after this would be meaningless."
  ok "secure boot enabled"
  [ $((16#$KV)) -eq 1 ] || die "KEY_VALID is 0x$KV, expected exactly 0x1 (slot 0 only)"
  ok "exactly one valid boot key (slot 0)"

  verify_slot_or_die 0 "$WORKDIR/factory-key.pem"

  hdr "Are spare slots still writable on a SEALED board?"
  info "This is the load-bearing assumption of the whole procedure. Page locks are"
  info "the only read-only way to answer it; the definitive test is phase 4's write."
  check_page_locks
  warn "Page locks clear is NECESSARY, not SUFFICIENT. Phase 4 is the real test."
  ok "sealed state matches what the runbook expects of a retail SeedHammer II"
  ;;

3)
  hdr "Phase 3 -- NEGATIVE CONTROL: prove the board rejects untrusted images"
  info "No OTP writes. Freely retryable."
  info "Right now only the FACTORY key is valid, so neither of the images below"
  info "should run. If either one blinks, secure boot is NOT being enforced and"
  info "every later 'it blinks!' result would be worthless."
  require_board

  build_blinky
  # These MUST be guarded by run(): unguarded, a dry run of phase 3 (which the
  # quickstart invites as a read-only way to preview a phase) overwrote the
  # signed image phase 5 depends on with an unsigned copy, turning the positive
  # control into a false FAIL on a board whose OTP was already spent.
  run cp "$WORKDIR/blinky.uf2" "$WORKDIR/blinky-mykey.uf2"
  run cp "$WORKDIR/blinky.uf2" "$WORKDIR/blinky-unsigned.uf2"

  hdr "3a -- an image signed with YOUR key (not yet burned)"
  run sign_image "$WORKDIR/blinky-mykey.uf2" "$WORKDIR/my-key.pem"
  flash_image "$WORKDIR/blinky-mykey.signed.uf2"
  ask_blink 'Did the LED blink (2 short + 1 long)?'
  case "$BLINK_ANSWER" in
    skip) info "[dry-run] would REQUIRE no blink here" ;;
    yes)  die "REJECTION FAILED: a your-key-signed image RAN on a board where your key
is not a valid boot key. Secure boot is not being enforced. Do not continue,
and do not trust phase 1's seal." ;;
    no)   ok "your-key-signed image was rejected, as it must be" ;;
    *) die "unreachable verdict: BLINK_ANSWER='$BLINK_ANSWER' matched no case -- refusing to continue" ;;
  esac

  hdr "3b -- an unsigned image"
  # The freshly built blinky has no SIGNATURE section at all (sign-firmware.sh
  # is what adds one), so it is ALREADY unsigned -- running `picosign sign
  # -clear` on it fails with "missing SIGNATURE section" and would abort the
  # phase. Flash it as-is.
  flash_image "$WORKDIR/blinky-unsigned.uf2"
  ask_blink 'Did the LED blink?'
  case "$BLINK_ANSWER" in
    skip) info "[dry-run] would REQUIRE no blink here" ;;
    yes)  die "REJECTION FAILED: an UNSIGNED image ran. Secure boot is not enforced." ;;
    no)   ok "unsigned image was rejected" ;;
    *) die "unreachable verdict: BLINK_ANSWER='$BLINK_ANSWER' matched no case -- refusing to continue" ;;
  esac

  ok_done "prove the sealed board rejects untrusted images -- the precondition for phases 5/6 meaning anything."
  ;;

4)
  hdr "Phase 4 -- add YOUR key to a spare slot (DESTRUCTIVE)"
  info "This is the exact operation you will later perform on the SeedHammer II,"
  info "and the only thing that changes between phase 3 and phase 5."
  require_board

  KV="$(otp_field BOOT_FLAGS1.KEY_VALID)"
  [ $((16#$KV)) -eq 1 ] || die "expected KEY_VALID 0x1 before this phase, found 0x$KV"
  [ -f "$WORKDIR/my-key.pem" ] || die "my-key.pem missing -- run phase 0 (keys are never generated inside a destructive phase)"
  check_page_locks

  hdr "4a -- build and validate the OTP json (no writes yet)"
  make_otp_json "$WORKDIR/my-key.pem" 1 "$WORKDIR/my-otp.json"

  hdr "4b -- burn the key hash into slot 1"
  confirm BURN-MY-KEY
  run picotool otp load "$WORKDIR/my-otp.json"

  hdr "4c -- verify all 16 rows BEFORE setting the valid bit"
  if [ "$EXECUTE" -eq 1 ]; then verify_slot_or_die 1 "$WORKDIR/my-key.pem"
  else info "[dry-run] would read back slot 1 and compare to the expected hash"; fi

  hdr "4d -- mark slot 1 valid"
  confirm SET-VALID-SLOT1
  run picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x2   # slot1=0x2, slot2=0x4, slot3=0x8
  if [ "$EXECUTE" -eq 1 ]; then
    KV2="$(otp_field BOOT_FLAGS1.KEY_VALID)"
    [ $((16#$KV2)) -eq 3 ] || die "KEY_VALID is 0x$KV2, expected 0x3 (slot 0 + slot 1)"
    ok "KEY_VALID is 0x3 -- dual trust established"
  fi

  ok_done "burn your key into slot 1 -- the state your SeedHammer II will end in."
  ;;

5)
  hdr "Phase 5 -- POSITIVE CONTROL: the image rejected in phase 3 must now boot"
  info "No OTP writes. The ONLY difference from phase 3a is phase 4's key burn."
  require_board

  KV="$(otp_field BOOT_FLAGS1.KEY_VALID)"
  [ $((16#$KV)) -eq 3 ] || die "KEY_VALID is 0x$KV, expected 0x3 -- run phase 4 first"
  [ -f "$WORKDIR/blinky-mykey.signed.uf2" ] || die "no phase-3 SIGNED image found -- run phase 3 --execute first so this is a true A/B"
  # "Present" is not "signed by your key". Assert the artifact is the same signed
  # image phase 3 proved was rejected, or the A/B claim is unfounded.
  MYPUB="$(openssl ec -in "$WORKDIR/my-key.pem" -pubout -conv_form uncompressed \
           -outform DER 2>/dev/null | tail -c 64 | od -An -v -tx1 | tr -d ' \n' | tr 'A-F' 'a-f')"
  IMGPUB="$(picotool info -a "$WORKDIR/blinky-mykey.signed.uf2" 2>/dev/null \
            | grep -iE '^ *public key:' | awk '{print $NF}' | tr 'A-F' 'a-f' || true)"
  [ -n "$IMGPUB" ] && [ "$IMGPUB" = "$MYPUB" ] || die "the phase-3 image is NOT signed by my-key.pem.
A dry run of phase 3, or a rebuild, has replaced it. Re-run phase 3 --execute so
phase 5 tests the SAME artifact phase 3 proved was rejected."
  picotool info -a "$WORKDIR/blinky-mykey.signed.uf2" 2>/dev/null | grep -qi 'signature: *verified' \
    || die "the phase-3 image does not carry a verified signature -- re-run phase 3 --execute"
  ok "phase-3 image is intact and signed by your key (true A/B)"

  info "Re-flashing the SAME your-key-signed image phase 3 proved was rejected."
  flash_image "$WORKDIR/blinky-mykey.signed.uf2"
  ask_blink 'Did the LED blink (2 short + 1 long)?'
  case "$BLINK_ANSWER" in
    skip) info "[dry-run] would REQUIRE a blink here" ;;
    yes)  ok "ACCEPTED after the key burn -- reject->accept across one OTP write."
          ok "The signing chain and the OTP procedure are both proven on real silicon." ;;
    no)   die "Your key is burned and valid, but your signed image still will not boot.
Something in the signing chain is wrong -- investigate before touching the
SeedHammer II. See sign-firmware.sh and note M3 (possible high-s signature):
re-sign once to get a fresh nonce before concluding the chain is broken." ;;
    *) die "unreachable verdict: BLINK_ANSWER='$BLINK_ANSWER' matched no case -- refusing to continue" ;;
  esac

  # F11(d): boot acceptance so far is only ever shown with a tiny TinyGo image.
  # The real firmware has a different block structure, so prove IT is accepted
  # too. It cannot run on a Pico (no SeedHammer peripherals), so the acceptance
  # signal is negative: a REJECTED secure-boot image returns to BOOTSEL, an
  # accepted one does not.
  FW_REAL="$(ls -1 "$SEEDHAMMER_DIR"/seedhammerii-*.uf2 2>/dev/null | grep -v '\.signed\.uf2$' | head -1 || true)"
  if [ -z "$FW_REAL" ] && [ "${ACCEPT_BLINKY_ONLY:-0}" = "1" ]; then
    warn "ACCEPT_BLINKY_ONLY=1 -- skipping 5b by request."
    warn "Acceptance is proven for the 33KB blinky ONLY, not the real firmware."
    FW_REAL="__skip__"
  fi
  if [ "$FW_REAL" = "__skip__" ]; then :
  elif [ -n "$FW_REAL" ]; then
    hdr "5b -- the REAL fork firmware, signed with your key"
    run cp "$FW_REAL" "$WORKDIR/fw-mykey.uf2"
    run sign_image "$WORKDIR/fw-mykey.uf2" "$WORKDIR/my-key.pem"
    info "The board is RUNNING the phase-5 image now, so it has LEFT BOOTSEL."
    info "Re-enter BOOTSEL: hold BOOTSEL while replugging USB. (The blinky has no"
    info "USB stack, so picotool cannot reset it for you.)"
    if [ "$EXECUTE" -eq 1 ]; then
      printf '\n\033[1mPress Enter once the board is back in BOOTSEL...\033[0m '
      read -r _ || true
    fi
    flash_image "$WORKDIR/fw-mykey.signed.uf2"
    info "Expect NO blink (no LED code) and NO SeedHammer UI (no hardware)."
    info "Verdict is taken from picotool, not from you -- 'accepted but hung' and"
    info "'operator did not look' are indistinguishable by eye on a headless Pico."
    bootsel_present
    case "$BOOTSEL_ANSWER" in
      skip) info "[dry-run] would REQUIRE the board NOT to be in BOOTSEL after reboot" ;;
      yes)  die "The real firmware image was REJECTED -- the board is back in BOOTSEL
(picotool can still see it) even though the blinky was accepted. The signing
chain does not generalise to the 2.4MB image. Investigate before touching the
SeedHammer II." ;;
      no)   ok "real firmware accepted by the bootrom (picotool can no longer see the board)" ;;
      *) die "unreachable verdict: BOOTSEL_ANSWER='$BOOTSEL_ANSWER' matched no case -- refusing to continue" ;;
    esac
  else
    die "NO REAL FIRMWARE FOUND in $SEEDHAMMER_DIR (seedhammerii-*.uf2).

Phase 5b is the ONLY proof that the real 2.4MB image -- not just the 33KB
blinky -- is accepted by the bootrom. Skipping it silently would hand you a
green rehearsal whose acceptance evidence covers the wrong artifact.

Build it first:
    cd $SEEDHAMMER_DIR && env VERSION=\$(git rev-parse HEAD) nix run .#build-firmware
then re-run this phase. (Set ACCEPT_BLINKY_ONLY=1 to proceed deliberately
without that proof -- but then say so in your notes.)"
  fi
  ok_done "confirm the previously-rejected image now boots."
  ;;

6)
  hdr "Phase 6 -- FALLBACK CONTROL: factory-signed images must STILL boot"
  info "This is what runbook step 7 ('do not revoke slot 0') depends on."
  require_board

  # Preconditions, mirroring phases 4/5. Without them this phase fails OPEN: on a
  # board where phase 1 never took, the blinky boots because secure boot is off,
  # and this phase would record "both keys boot" -- a false proof of the very
  # recovery path the runbook's step 7 and Recovery section rest on.
  SB="$(otp_field CRIT1.SECURE_BOOT_ENABLE)"
  [ $((16#$SB & 1)) -eq 1 ] || die "secure boot is NOT enabled (0x$SB).
A blink here would prove nothing -- any image boots on an unsealed board.
Run phases 1 and 2 first."
  KV="$(otp_field BOOT_FLAGS1.KEY_VALID)"
  [ $((16#$KV)) -eq 3 ] || die "KEY_VALID is 0x$KV, expected 0x3 (slot 0 + slot 1) -- run phase 4 first"
  ok "preconditions hold: sealed board, dual trust"

  [ -f "$WORKDIR/factory-key.pem" ] || die "factory-key.pem missing"
  run cp "$WORKDIR/blinky.uf2" "$WORKDIR/blinky-factory.uf2"
  run sign_image "$WORKDIR/blinky-factory.uf2" "$WORKDIR/factory-key.pem"
  info "If phase 5 left the board running an image, re-enter BOOTSEL first."
  flash_image "$WORKDIR/blinky-factory.signed.uf2"
  ask_blink 'Did the LED blink?'
  case "$BLINK_ANSWER" in
    skip) info "[dry-run] would REQUIRE a blink here" ;;
    yes)  ok "Both keys boot. Leaving SeedHammer's slot valid really does preserve the fallback." ;;
    no)   die "A factory-key-signed image no longer boots on a dual-trust board.
The recovery path assumed by the runbook does not hold. STOP -- do not perform
this procedure on the SeedHammer II." ;;
    *) die "unreachable verdict: BLINK_ANSWER='$BLINK_ANSWER' matched no case -- refusing to continue" ;;
  esac
  ok_done "confirm the factory key still boots alongside yours."
  ;;

*)
  die "unknown phase: $PHASE (expected 0..6)"
  ;;
esac
