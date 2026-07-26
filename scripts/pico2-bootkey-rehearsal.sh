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
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# Anchored to the repo, NOT the cwd: cross-phase state (keys, CHIPID, images)
# must not silently vanish when the operator runs from a different directory.
WORKDIR="${WORKDIR:-$REPO_ROOT/rehearsal-work}"
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

while [ $# -gt 0 ]; do
  case "$1" in
    --phase)   PHASE="${2:-}"; shift 2 ;;
    --execute) EXECUTE=1; shift ;;
    *) die "unknown argument: $1" ;;
  esac
done
[ -n "$PHASE" ] || die "specify --phase <0..6>"

for t in picotool openssl sha256sum; do
  command -v "$t" >/dev/null || die "$t not found -- run inside 'nix develop' in the seedhammer fork"
done
# tinygo/go are only needed where images are built or signed.
case "$PHASE" in
  0|3|4|5|6)
    command -v tinygo >/dev/null || die "tinygo not found -- run inside 'nix develop'"
    command -v go     >/dev/null || die "go not found -- run inside 'nix develop'" ;;
esac
[ -n "$SEEDHAMMER_DIR" ] && [ -d "$SEEDHAMMER_DIR" ] \
  || die "set SEEDHAMMER_DIR to the seedhammer fork (provides cmd/picosign)"

mkdir -p "$WORKDIR"
[ "$EXECUTE" -eq 1 ] || printf '\n\033[90m(dry-run -- nothing will be written; add --execute to arm)\033[0m\n'

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
  v="$(printf '%s\n' "$out" | grep -iE '^[[:space:]]*field ' | tail -1 \
       | sed -E 's/.*=[[:space:]]*//' | tr -d '[:space:]' | tr 'A-F' 'a-f')"
  [ -n "$v" ] || die "could not parse a field value for $sel from picotool output:
$out"
  printf '%s' "$v" | grep -qE '^[0-9a-f]+$' || die "unexpected field value '$v' for $sel"
  printf '%s' "$v"
}

# read_row <selector> -> exactly 4 hex digits (low 16 bits of the row)
read_row() {
  local sel="$1" out v
  out="$(picotool otp get -n -e "$sel" 2>&1)" || die "OTP read failed for $sel:
$out"
  printf '%s' "$out" | grep -qi 'WARNING' \
    && die "picotool reported a warning reading $sel (invalid ECC or unequal redundant rows):
$out"
  v="$(printf '%s\n' "$out" | grep -oiE '^[[:space:]]*VALUE 0x[0-9a-f]+' | tail -1 \
       | grep -oiE '0x[0-9a-f]+' | sed 's/^0[xX]//' | tr 'A-F' 'a-f')"
  [ -n "$v" ] || die "could not parse a VALUE line for row $sel from picotool output:
$out"
  # VALUE is the 24-bit raw row; ECC-corrected data occupies the low 16 bits.
  printf '%04x' $(( 16#$v & 0xffff ))
}

# read_slot <n> -> 64 hex chars: the 32-byte key hash in slot n.
# Each row holds two bytes, LOW BYTE FIRST, so each row's 4 hex digits are
# byte-swapped on reassembly.
read_slot() {
  local n="$1" i row out=""
  for i in $(seq 0 15); do
    row="$(read_row "BOOTKEY${n}_${i}")"
    out="${out}${row:2:2}${row:0:2}"
  done
  printf '%s' "$out"
}

# key_hash <key.pem> -> sha256 of the UNCOMPRESSED 64-byte X||Y pubkey.
# This is what the RP2350 stores in a boot-key slot. Computed independently of
# picotool so the two can be cross-checked.
key_hash() {
  openssl ec -in "$1" -pubout -conv_form uncompressed -outform DER 2>/dev/null \
    | tail -c 64 | sha256sum | cut -d' ' -f1
}

chipid() { local i out=""; for i in 0 1 2 3; do out="${out}$(read_row "CHIPID$i")"; done; printf '%s' "$out"; }

# require_board: refuse to act on any board other than the one phase 0 saw.
require_board() {
  local pinned cur
  [ -f "$WORKDIR/board-chipid.txt" ] || die "no pinned board -- run phase 0 first"
    pinned="$(cat "$WORKDIR/board-chipid.txt")"
  cur="$(chipid)"
  [ "$cur" = "$pinned" ] || die "WRONG BOARD.
  pinned (phase 0): $pinned
  connected now:    $cur
Refusing to touch a board this rehearsal did not inventory."
  ok "board identity matches phase 0 ($cur)"

  # Independent tripwire: never write to a real SeedHammer II.
  local slot0; slot0="$(read_slot 0)"
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
    slot="$(read_slot $i)"
    [ "$slot" = "$(printf '0%.0s' $(seq 1 64))" ] \
      || die "boot-key slot $i is not empty (0x$slot) -- board already used. Get a FRESH board."
  done
  ok "all four boot-key slots empty (all 16 rows each, parser validated)"
}

check_page_locks() {
  local l
  for l in PAGE1_LOCK0 PAGE1_LOCK1 PAGE2_LOCK0 PAGE2_LOCK1; do
    local v; v="$(read_row "$l")"
    [ $((16#$v)) -eq 0 ] \
      || die "$l is non-zero (0x$v) -- OTP pages are LOCKED. No further boot key can be added to this device."
  done
  ok "OTP page locks clear (PAGE1/PAGE2) -- boot-key rows are writable"
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
  got="$(read_slot "$slot")"
  [ "$got" = "$expect" ] || die "SLOT $slot READBACK MISMATCH -- DO NOT SET THE VALID BIT.
  expected: $expect
  read:     $got
The slot is not yet valid, so the board still boots normally. Use the next
free slot instead; this one is permanently unusable."
  ok "slot $slot readback matches the expected hash across all 16 rows"
}

# flash_and_ask <image> <question>
flash_and_ask() {
  local img="$1" question="$2"
  run picotool load --verify "$img"
  run picotool reboot
  [ "$EXECUTE" -eq 1 ] || return 0
  printf '\n\033[1m%s\033[0m [y/n]: ' "$question"
  read -r a
  [ "$a" = "y" ] || return 1
  return 0
}

sign_image() {
  SEEDHAMMER_DIR="$SEEDHAMMER_DIR" "$REPO_ROOT/scripts/sign-firmware.sh" "$1" "$2"
}

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
  CID="$(chipid)"
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
  assert_stock_or_die

  [ -f "$WORKDIR/factory-key.pem" ] || die "factory-key.pem missing -- run phase 0"

  hdr "1a -- build and validate the OTP json (no writes yet)"
  make_otp_json "$WORKDIR/factory-key.pem" 0 "$WORKDIR/factory-otp.json"

  hdr "1b -- burn the key hash into slot 0"
  confirm BURN-FACTORY
  run picotool otp load "$WORKDIR/factory-otp.json"

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
  cp "$WORKDIR/blinky.uf2" "$WORKDIR/blinky-mykey.uf2"
  cp "$WORKDIR/blinky.uf2" "$WORKDIR/blinky-unsigned.uf2"

  hdr "3a -- an image signed with YOUR key (not yet burned)"
  run sign_image "$WORKDIR/blinky-mykey.uf2" "$WORKDIR/my-key.pem"
  if flash_and_ask "$WORKDIR/blinky-mykey.uf2" "Did the LED blink (2 short + 1 long)?"; then
    die "REJECTION FAILED: a your-key-signed image RAN on a board where your key
is not a valid boot key. Secure boot is not being enforced. Do not continue,
and do not trust phase 1's seal."
  fi
  ok "your-key-signed image was rejected, as it must be"

  hdr "3b -- an unsigned image"
  run sh -c "( cd '$SEEDHAMMER_DIR' && go run seedhammer.com/cmd/picosign sign -clear '$WORKDIR/blinky-unsigned.uf2' )"
  if flash_and_ask "$WORKDIR/blinky-unsigned.uf2" "Did the LED blink?"; then
    die "REJECTION FAILED: an UNSIGNED image ran. Secure boot is not enforced."
  fi
  ok "unsigned image was rejected"

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
  [ -f "$WORKDIR/blinky-mykey.uf2" ] || die "no phase-3 image found -- run phase 3 first so this is a true A/B"

  info "Re-flashing the SAME your-key-signed image phase 3 proved was rejected."
  if flash_and_ask "$WORKDIR/blinky-mykey.uf2" "Did the LED blink (2 short + 1 long)?"; then
    ok "ACCEPTED after the key burn -- reject->accept across one OTP write."
    ok "The signing chain and the OTP procedure are both proven on real silicon."
  else
    die "Your key is burned and valid, but your signed image still will not boot.
Something in the signing chain is wrong -- investigate before touching the
SeedHammer II. See sign-firmware.sh and note M3 (possible high-s signature):
re-sign once to get a fresh nonce before concluding the chain is broken."
  fi
  ok_done "confirm the previously-rejected image now boots."
  ;;

6)
  hdr "Phase 6 -- FALLBACK CONTROL: factory-signed images must STILL boot"
  info "This is what runbook step 7 ('do not revoke slot 0') depends on."
  require_board

  [ -f "$WORKDIR/factory-key.pem" ] || die "factory-key.pem missing"
  cp "$WORKDIR/blinky.uf2" "$WORKDIR/blinky-factory.uf2"
  run sign_image "$WORKDIR/blinky-factory.uf2" "$WORKDIR/factory-key.pem"

  if flash_and_ask "$WORKDIR/blinky-factory.uf2" "Did the LED blink?"; then
    ok "Both keys boot. Leaving SeedHammer's slot valid really does preserve the fallback."
  else
    die "A factory-key-signed image no longer boots on a dual-trust board.
The recovery path assumed by the runbook does not hold. STOP -- do not perform
this procedure on the SeedHammer II."
  fi
  ok_done "confirm the factory key still boots alongside yours."
  ;;

*)
  die "unknown phase: $PHASE (expected 0..6)"
  ;;
esac
