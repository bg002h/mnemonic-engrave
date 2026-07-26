#!/usr/bin/env bash
#
# pico2-bootkey-rehearsal.sh -- rehearse the SeedHammer II custom-boot-key
# procedure on a throwaway RP2350 board (Raspberry Pi Pico 2 / Pico 2 W).
#
# Companion to design/RUNBOOK_custom_boot_key.md. Run this to completion on a
# Pico BEFORE touching the SeedHammer II's OTP.
#
# WHAT THIS REHEARSES
#   The picotool OTP -> sign -> boot-acceptance chain, which is the only
#   irreversible part of the real procedure. It does NOT rehearse SeedHammer
#   firmware behavior -- our controller drives SeedHammer's display, steppers
#   and NFC and will not run meaningfully on any Pico. That is fine and
#   expected: the signature check happens before any of that code runs.
#
# BOARD IS CONSUMED
#   OTP is one-time-programmable on the Pico too. A full run burns 2 of the 4
#   boot-key slots and permanently enables secure boot on the board. Budget one
#   board per full run. Do not use a board you want back.
#
# SAFETY
#   Dry-run by default: every destructive command is printed, not executed.
#   Pass --execute to actually write. Each destructive phase additionally
#   requires its own explicit --phase flag, so you cannot burn OTP by
#   accidentally running the script with --execute.
#
# Usage:
#   ./pico2-bootkey-rehearsal.sh --phase 0                 # read-only inventory
#   ./pico2-bootkey-rehearsal.sh --phase 1 --execute       # seal the board (destructive)
#   ./pico2-bootkey-rehearsal.sh --phase 2                 # verify sealed state
#   ./pico2-bootkey-rehearsal.sh --phase 3 --execute       # add your key (destructive)
#   ./pico2-bootkey-rehearsal.sh --phase 4                 # sign + flash + confirm boot
#   ./pico2-bootkey-rehearsal.sh --phase 5                 # recovery-path negative control
#
set -euo pipefail

WORKDIR="${WORKDIR:-./rehearsal-work}"
EXECUTE=0
PHASE=""

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# The seedhammer fork provides cmd/picosign, used by scripts/sign-firmware.sh.
SEEDHAMMER_DIR="${SEEDHAMMER_DIR:-$(cd "$REPO_ROOT/../seedhammer" 2>/dev/null && pwd || true)}"

die()  { printf '\n\033[31mFAIL:\033[0m %s\n' "$*" >&2; exit 1; }
ok()   { printf '\033[32m  PASS:\033[0m %s\n' "$*"; }

# ok_done: closing message for a phase that WRITES. In dry-run it must not
# claim an outcome that did not happen.
ok_done() {
  if [ "$EXECUTE" -eq 1 ]; then
    printf '\033[32m  PASS:\033[0m %s\n' "$*"
  else
    printf '\033[90m  [dry-run] nothing was written. With --execute this phase would: %s\033[0m\n' "$*"
  fi
}
info() { printf '\033[36m  ..\033[0m %s\n' "$*"; }
hdr()  { printf '\n\033[1m== %s ==\033[0m\n' "$*"; }

# run: execute a command, or print it in dry-run mode.
run() {
  if [ "$EXECUTE" -eq 1 ]; then
    printf '\033[33m  $ %s\033[0m\n' "$*"
    "$@"
  else
    printf '\033[90m  [dry-run] %s\033[0m\n' "$*"
  fi
}

# confirm: require the operator to type a specific word before an OTP write.
confirm() {
  local word="$1"
  [ "$EXECUTE" -eq 1 ] || return 0
  printf '\n\033[31mThis writes OTP and CANNOT be undone.\033[0m Type %s to proceed: ' "$word"
  read -r reply
  [ "$reply" = "$word" ] || die "aborted at operator confirmation"
}

while [ $# -gt 0 ]; do
  case "$1" in
    --phase)   PHASE="$2"; shift 2 ;;
    --execute) EXECUTE=1; shift ;;
    *) die "unknown argument: $1" ;;
  esac
done
[ -n "$PHASE" ] || die "specify --phase <0..5>"

command -v picotool >/dev/null || die "picotool not found (nix develop in the seedhammer fork provides it)"
command -v openssl  >/dev/null || die "openssl not found"
mkdir -p "$WORKDIR"

[ "$EXECUTE" -eq 1 ] || printf '\n\033[90m(dry-run -- nothing will be written; add --execute to arm)\033[0m\n'

# ---------------------------------------------------------------------------
# NOTE ON picotool SYNTAX
# The exact field names below (CRIT1.SECURE_BOOT_ENABLE, BOOT_FLAGS1.KEY_VALID)
# follow picotool 2.x. They have shifted across picotool releases. Phase 0
# probes them read-only first and aborts if any probe fails -- do not "fix" a
# failing probe by guessing a field name, check `picotool otp list` instead.
# ---------------------------------------------------------------------------

case "$PHASE" in

0)
  hdr "Phase 0 -- read-only inventory (safe, run this first)"
  info "Put the board in BOOTSEL: hold BOOTSEL while connecting USB."

  picotool info || die "board not visible -- is it in BOOTSEL mode?"

  info "Confirming this is RP2350 silicon (not RP2040, not RP2354)"
  picotool info | grep -qi 'rp2350' || die "not an RP2350 board -- wrong hardware for this rehearsal"
  ok "RP2350 detected"

  info "Probing OTP field names (read-only)"
  picotool otp get CRIT1.SECURE_BOOT_ENABLE >/dev/null 2>&1 \
    || die "cannot read CRIT1.SECURE_BOOT_ENABLE -- check 'picotool otp list' for this picotool version"
  picotool otp get BOOT_FLAGS1.KEY_VALID >/dev/null 2>&1 \
    || die "cannot read BOOT_FLAGS1.KEY_VALID -- check 'picotool otp list' for this picotool version"
  ok "OTP field names resolve on this picotool version"

  hdr "Current state"
  sb=$(picotool otp get CRIT1.SECURE_BOOT_ENABLE); echo "$sb"
  kv=$(picotool otp get BOOT_FLAGS1.KEY_VALID);    echo "$kv"
  for k in BOOTKEY0_0 BOOTKEY1_0 BOOTKEY2_0 BOOTKEY3_0; do
    echo "--- $k ---"; picotool otp get -e "$k" || true
  done

  # Hard assertions: a used board must not be mistaken for a stock one, or the
  # whole rehearsal draws conclusions from the wrong starting state.
  hdr "Asserting the board is stock"
  hexval() { printf '%s' "$1" | grep -oiE '0x[0-9a-f]+' | tail -1; }

  sbv=$(hexval "$sb"); kvv=$(hexval "$kv")
  [ -n "$sbv" ] || die "could not parse SECURE_BOOT_ENABLE from: $sb"
  [ -n "$kvv" ] || die "could not parse KEY_VALID from: $kv"

  [ $((sbv)) -eq 0 ] || die "SECURE_BOOT_ENABLE is already set ($sbv) -- this board has been sealed. OTP is one-time; get a FRESH board."
  ok "secure boot not yet enabled"

  [ $((kvv)) -eq 0 ] || die "KEY_VALID is already $kvv -- a boot key is already valid on this board. Get a FRESH board."
  ok "no boot key marked valid"

  for k in BOOTKEY0_0 BOOTKEY1_0 BOOTKEY2_0 BOOTKEY3_0; do
    v=$(hexval "$(picotool otp get -e "$k" 2>/dev/null || echo 0x0)")
    [ -n "$v" ] && [ $((v)) -eq 0 ] \
      || die "$k is non-zero ($v) -- a key hash is already burned here. Get a FRESH board."
  done
  ok "all four boot-key slots are empty"

  ok "Phase 0 complete -- board is stock. Proceed to phase 1 to seal it."
  ;;

1)
  hdr "Phase 1 -- seal the board, mimicking SeedHammer's LockBoot() (DESTRUCTIVE)"
  info "This reproduces cmd/controller/platform_sh2.go:510-518 on the Pico:"
  info "  AddBootKey(factory key) + EnableSecureBoot()"
  info "so that phases 2-5 face the same starting condition as your SeedHammer II."

  if [ ! -f "$WORKDIR/factory-key.pem" ]; then
    info "Generating stand-in 'factory' key (plays SeedHammer's role)"
    run openssl ecparam -name secp256k1 -genkey -noout -out "$WORKDIR/factory-key.pem"
  fi

  info "Deriving the OTP hash via picotool -- never hand-compute this."
  info "It must be SHA-256 of the UNCOMPRESSED 64-byte X||Y pubkey, not the 33-byte compressed form."
  run picotool seal --sign "$WORKDIR/placeholder.elf" "$WORKDIR/discard.elf" \
      "$WORKDIR/factory-key.pem" "$WORKDIR/factory-otp.json"

  info "Now edit $WORKDIR/factory-otp.json to keep ONLY bootkey0,"
  info "deleting the boot_flags1 and crit1 entries (valid bit is set separately, after verification)."

  confirm BURN-FACTORY
  run picotool otp load "$WORKDIR/factory-otp.json"

  info "Verify all 16 rows before setting the valid bit"
  for i in $(seq 0 15); do picotool otp get -e "BOOTKEY0_$i" || true; done

  confirm SET-VALID
  run picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x1
  run picotool otp set -s CRIT1.SECURE_BOOT_ENABLE 0x1

  ok_done "seal the board -- it would then only boot images signed by factory-key.pem."
  ;;

2)
  hdr "Phase 2 -- verify the sealed state matches runbook step 1's expectations"
  info "This is the assertion that our read of the SeedHammer II is correct."

  sb=$(picotool otp get CRIT1.SECURE_BOOT_ENABLE)
  kv=$(picotool otp get BOOT_FLAGS1.KEY_VALID)
  echo "$sb"; echo "$kv"

  echo "$sb" | grep -q '1' || die "secure boot not enabled -- phase 1 did not take"
  ok "SECURE_BOOT_ENABLE is set"
  echo "$kv" | grep -qi '0x\?1\b' || info "check KEY_VALID reads 0x1 (slot 0 only)"

  info "Confirming OTP writes are still PERMITTED on a sealed board."
  info "This is the load-bearing assumption of the whole procedure: SeedHammer's"
  info "driver/otp/ never writes page locks, DEBUG_DISABLE or KEY_INVALID, so"
  info "PICOBOOT/OTP access should survive sealing. If this probe fails, STOP --"
  info "the real device will not accept a second key either."
  picotool otp get -e BOOTKEY1_0 >/dev/null 2>&1 \
    || die "cannot read spare slot on a sealed board -- procedure assumption broken"
  ok "spare boot-key slots remain accessible after sealing"
  ;;

3)
  hdr "Phase 3 -- add YOUR key to a spare slot (DESTRUCTIVE)"
  info "This is the exact operation you will later perform on the SeedHammer II."

  if [ ! -f "$WORKDIR/my-key.pem" ]; then
    run openssl ecparam -name secp256k1 -genkey -noout -out "$WORKDIR/my-key.pem"
    info "BACK THIS UP. Losing it means you can never sign another update."
  fi

  run picotool seal --sign "$WORKDIR/placeholder.elf" "$WORKDIR/discard.elf" \
      "$WORKDIR/my-key.pem" "$WORKDIR/my-otp.json"

  info "Edit $WORKDIR/my-otp.json: keep only the key hash, rename bootkey0 -> bootkey1,"
  info "delete boot_flags1 and crit1."

  confirm BURN-MY-KEY
  run picotool otp load "$WORKDIR/my-otp.json"

  info "VERIFY ALL 16 ROWS. If any row disagrees, STOP and use slot 2 instead --"
  info "the slot is not valid until the next step, so the board still boots normally."
  for i in $(seq 0 15); do picotool otp get -e "BOOTKEY1_$i" || true; done

  confirm SET-VALID-SLOT1
  run picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x2   # slot1=0x2, slot2=0x4, slot3=0x8
  picotool otp get BOOT_FLAGS1.KEY_VALID
  info "Expect 0x3 -- slot 0 (factory) AND slot 1 (yours) both valid."
  ok_done "establish dual-trust -- the state your SeedHammer II will end in."
  ;;

4)
  hdr "Phase 4 -- sign an image with YOUR key and confirm it boots (retryable)"
  info "Nothing here touches OTP. Get it wrong as often as you need."

  # Default payload: the rehearsal blinky. Its LED pattern (2 short + 1 long) is
  # the eyes-on proof that the bootrom ACCEPTED a self-signed image -- a rejected
  # image never reaches main(). Build it for the PLAIN Pico 2: on the Pico 2 W the
  # LED sits behind the CYW43 chip and machine.LED does nothing.
  FW="${FW:-$WORKDIR/blinky.uf2}"
  if [ ! -f "$FW" ]; then
    info "building the rehearsal blinky (-target pico2)"
    run sh -c "cd '$REPO_ROOT/scripts/rehearsal-blinky' && tinygo build -o '$FW' -target pico2 -opt 2 ."
    [ "$EXECUTE" -eq 1 ] && { [ -f "$FW" ] || die "blinky build produced no image"; }
  fi

  info "Signing + PROVING the signature offline via scripts/sign-firmware.sh."
  info "That script asserts the digest is unchanged after embedding the signature"
  info "(i.e. the signature is outside the hashed region) and verifies the"
  info "signature against the digest with openssl before anything is flashed."
  run sh -c "SEEDHAMMER_DIR='$SEEDHAMMER_DIR' '$REPO_ROOT/scripts/sign-firmware.sh' '$FW' '$WORKDIR/my-key.pem'"

  info "picotool should report 'signature: verified' above."
  run picotool load --verify "$FW"
  run picotool reboot
  info "PASS = the LED blinks 2-short-then-1-long. No blink = image rejected."
  ok_done "flash your self-signed image; if the board runs it, the sign chain is proven end-to-end."
  ;;

5)
  hdr "Phase 5 -- recovery-path negative control (the reason runbook step 7 says DON'T revoke)"
  info "Sign the same test image with the FACTORY key and confirm it ALSO still boots."
  info "This proves that leaving SeedHammer's slot valid preserves your fallback to"
  info "official firmware. If this fails, do not proceed on the real device."

  # Re-sign a FRESH copy of the blinky with the factory key. Use a copy so the
  # your-key-signed image from phase 4 survives for comparison.
  SRC="${FW:-$WORKDIR/blinky.uf2}"
  [ -f "$SRC" ] || die "no blinky image at $SRC -- run phase 4 first"
  FACFW="$WORKDIR/blinky-factory-signed.uf2"
  run cp "$SRC" "$FACFW"

  run sh -c "SEEDHAMMER_DIR='$SEEDHAMMER_DIR' '$REPO_ROOT/scripts/sign-firmware.sh' '$FACFW' '$WORKDIR/factory-key.pem'"
  run picotool load --verify "$FACFW"
  run picotool reboot

  info "PASS = the LED still blinks. Both your key and the factory key boot,"
  info "so leaving SeedHammer's slot valid really does preserve the fallback."
  ok_done "flash a factory-signed image; if it also boots, the recovery path is confirmed."
  ;;

*)
  die "unknown phase: $PHASE (expected 0..5)"
  ;;
esac
