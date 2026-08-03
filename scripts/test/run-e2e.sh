#!/usr/bin/env bash
#
# run-e2e.sh -- exercise the ENTIRE boot-key plan without hardware.
#
# Drives scripts/pico2-bootkey-rehearsal.sh phases 0->6 and all three SH2 modes
# against scripts/test/fake-picotool, a stateful OTP simulator, and asserts both
# the happy path and the refusals. Real signing/sealing is used throughout
# (fake-picotool delegates those to the real picotool), so the sign chain is
# genuinely exercised.
#
# WHAT THIS DOES AND DOES NOT PROVE
#   Proves: the OTP state machine, the json transform, the 16-row readback and
#   its byte-swap reassembly, phase ordering and preconditions, the SH2 modes,
#   and every refusal path.
#   Does NOT prove: that the RP2350 bootrom accepts or rejects any image. Only
#   the Pico 2 rehearsal can show that. This is a regression harness, not a
#   substitute for it.
#
# Usage (needs picotool/tinygo/go, so run from the fork's devshell):
#   cd /scratch/code/shibboleth/seedhammer
#   nix develop --command ../mnemonic-engrave/scripts/test/run-e2e.sh
#
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
R="$REPO/scripts/pico2-bootkey-rehearsal.sh"
SH_HASH="c8314536d6af61ac2e62e5991e3e4711629c54696ba8c4af08965a1d319a473b"

PASS=0; FAIL=0
ok()   { printf '\033[32m  ok  \033[0m %s\n' "$*"; PASS=$((PASS+1)); }
bad()  { printf '\033[31m FAIL \033[0m %s\n' "$*"; FAIL=$((FAIL+1)); }
hdr()  { printf '\n\033[1m== %s ==\033[0m\n' "$*"; }

command -v picotool >/dev/null || { echo "picotool not found -- run inside 'nix develop'"; exit 2; }
command -v tinygo   >/dev/null || { echo "tinygo not found -- run inside 'nix develop'"; exit 2; }
export REAL_PICOTOOL="$(command -v picotool)"

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
export WORKDIR="$TMP/work"; mkdir -p "$WORKDIR"
# Isolate SH2 state too, or the harness pins a fake all-zero CHIPID into the
# REAL sh2-state/ and the next run against the actual SeedHammer dies "WRONG
# DEVICE" -- a tampering-shaped alarm caused entirely by a test.
export SH2_DIR="$TMP/sh2-state"; mkdir -p "$SH2_DIR"
export PATH="$HERE:$PATH"            # fake-picotool shadows the real one
ln -sf "$HERE/fake-picotool" "$HERE/picotool" 2>/dev/null || true
trap 'rm -f "$HERE/picotool"; rm -rf "$TMP"' EXIT

run_phase() { # run_phase <desc> <expect-pass|expect-fail> <input> <args...>
  local desc="$1" expect="$2" input="$3"; shift 3
  local out rc
  out="$(printf '%s' "$input" | "$R" "$@" 2>&1)"; rc=$?
  if [ "$expect" = "expect-pass" ]; then
    [ $rc -eq 0 ] && ok "$desc" || { bad "$desc (exit $rc)"; printf '%s\n' "$out" | tail -4; }
  else
    [ $rc -ne 0 ] && ok "$desc (correctly refused)" || bad "$desc — SHOULD HAVE FAILED"
  fi
}

state_is() { # state_is <key> <expected>
  local got; got="$(grep "^$1=" "$OTPSTATE" 2>/dev/null | tail -1 | cut -d= -f2-)"
  [ "$got" = "$2" ] && ok "OTP $1 == $2" || bad "OTP $1 is '${got:-unset}', expected '$2'"
}

########################################################################
hdr "A. Pico 2 rehearsal, phases 0 -> 6, in order"
export OTPSTATE="$TMP/pico.state"; printf 'SB=0\nKV=0\nKI=0\n' > "$OTPSTATE"

run_phase "phase 0 (inventory, pin, keys, payload)" expect-pass "" --phase 0
state_is SB 0
state_is KV 0

run_phase "phase 1 (seal)" expect-pass $'BURN-FACTORY\nSET-VALID\n' --phase 1 --execute
state_is SB 1
state_is KV 1
FACT="$(openssl ec -in "$WORKDIR/factory-key.pem" -pubout -conv_form uncompressed \
        -outform DER 2>/dev/null | tail -c 64 | sha256sum | cut -d' ' -f1)"
state_is SLOT0 "$FACT"

run_phase "phase 2 (verify sealed)" expect-pass "" --phase 2
run_phase "phase 3 (negative control, both rejected)" expect-pass $'n\nn\n' --phase 3 --execute
run_phase "phase 4 (burn my key)" expect-pass $'BURN-MY-KEY\nSET-VALID-SLOT1\n' --phase 4 --execute
state_is KV 3
MINE="$(openssl ec -in "$WORKDIR/my-key.pem" -pubout -conv_form uncompressed \
        -outform DER 2>/dev/null | tail -c 64 | sha256sum | cut -d' ' -f1)"
state_is SLOT1 "$MINE"

# From here the simulated bootrom "runs" flashed images.
printf 'PENDING_OK=1\n' >> "$OTPSTATE"
run_phase "phase 5 (+5b real firmware) accepted" expect-pass $'y\n' --phase 5 --execute
run_phase "phase 6 (fallback control)" expect-pass $'y\n' --phase 6 --execute

########################################################################
hdr "B. Refusals on the rehearsal path"

# A blink in the negative control means secure boot is NOT enforced -> must die.
run_phase "phase 3 must FAIL if an image blinks" expect-fail $'y\n' --phase 3 --execute

# Out-of-order / precondition failures on a fresh unsealed board.
export OTPSTATE="$TMP/fresh.state"; printf 'SB=0\nKV=0\nKI=0\n' > "$OTPSTATE"
rm -f "$WORKDIR/board-chipid.txt"
run_phase "phase 0 on a stock board" expect-pass "" --phase 0
run_phase "phase 6 must FAIL on an unsealed board" expect-fail $'y\n' --phase 6 --execute
run_phase "phase 5 must FAIL before phase 4" expect-fail $'y\n' --phase 5 --execute
run_phase "phase 2 must FAIL before phase 1" expect-fail "" --phase 2

# A used board must not be mistaken for a stock one.
printf 'SB=1\n' >> "$OTPSTATE"
run_phase "phase 0 must FAIL on an already-sealed board" expect-fail "" --phase 0

########################################################################
hdr "C. SH2 modes against a simulated retail SeedHammer II"
export OTPSTATE="$TMP/sh2.state"
printf 'SB=1\nKV=1\nKI=0\nSLOT0=%s\n' "$SH_HASH" > "$OTPSTATE"
rm -f "$SH2_DIR/sh2-chipid.txt"

run_phase "--sh2-precheck on a retail unit" expect-pass "" --sh2-precheck
openssl ecparam -name secp256k1 -genkey -noout -out "$TMP/sh2-boot-key.pem" 2>/dev/null

run_phase "--make-otp-json refuses a REHEARSAL key" expect-fail "" \
  --make-otp-json --key "$WORKDIR/my-key.pem" --slot 1 --out "$TMP/x.json"
run_phase "--make-otp-json accepts a distinct SH2 key" expect-pass "" \
  --make-otp-json --key "$TMP/sh2-boot-key.pem" --slot 1 --out "$TMP/sh2-otp.json"

picotool otp load "$TMP/sh2-otp.json" >/dev/null 2>&1
run_phase "--sh2-verify-slot passes with the right key" expect-pass "" \
  --sh2-verify-slot 1 --key "$TMP/sh2-boot-key.pem"

openssl ecparam -name secp256k1 -genkey -noout -out "$TMP/wrong.pem" 2>/dev/null
run_phase "--sh2-verify-slot FAILS with the wrong key" expect-fail "" \
  --sh2-verify-slot 1 --key "$TMP/wrong.pem"

# The rehearsal phases must refuse a real SeedHammer II.
run_phase "phase 0 refuses a SeedHammer II" expect-fail "" --phase 0
run_phase "phase 1 refuses a SeedHammer II" expect-fail $'BURN-FACTORY\n' --phase 1 --execute

# --sh2-verify-valid: the post-write gate. Previously ZERO coverage -- not on
# hardware, not here -- despite gating the last irreversible step.
hdr "D. --sh2-verify-valid (post-write gate)"

# Before the valid bit is set, it must refuse.
run_phase "--sh2-verify-valid FAILS before the valid bit is set" expect-fail "" \
  --sh2-verify-valid 1 --key "$TMP/sh2-boot-key.pem"

picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x2 >/dev/null 2>&1
run_phase "--sh2-verify-valid passes once KEY_VALID is 0x3" expect-pass "" \
  --sh2-verify-valid 1 --key "$TMP/sh2-boot-key.pem"
state_is KV 3

# Wrong key must fail even with the valid bit correctly set.
run_phase "--sh2-verify-valid FAILS with the wrong key" expect-fail "" \
  --sh2-verify-valid 1 --key "$TMP/wrong.pem"

# Wrong slot: slot 2 was never burned, so both hash and valid-bit are wrong.
run_phase "--sh2-verify-valid FAILS for a slot that was never burned" expect-fail "" \
  --sh2-verify-valid 2 --key "$TMP/sh2-boot-key.pem"

# An EXTRA valid bit is not an interrupted write and must not be reported as one.
picotool otp set -s BOOT_FLAGS1.KEY_VALID 0x4 >/dev/null 2>&1
run_phase "--sh2-verify-valid FAILS on an unexpected extra valid bit" expect-fail "" \
  --sh2-verify-valid 1 --key "$TMP/sh2-boot-key.pem"

# --sh2-verify-slot must refuse once the slot is already valid (its advice would be wrong).
run_phase "--sh2-verify-slot refuses an already-valid slot" expect-fail "" \
  --sh2-verify-slot 1 --key "$TMP/sh2-boot-key.pem"

hdr "E. key and curve guards"
openssl ecparam -name prime256v1 -genkey -noout -out "$TMP/p256.pem" 2>/dev/null
run_phase "--make-otp-json REFUSES a non-secp256k1 (P-256) key" expect-fail "" \
  --make-otp-json --key "$TMP/p256.pem" --slot 1 --out "$TMP/p256.json"
openssl ecparam -name secp384r1 -genkey -noout -out "$TMP/p384.pem" 2>/dev/null
run_phase "--make-otp-json REFUSES a non-secp256k1 (P-384) key" expect-fail "" \
  --make-otp-json --key "$TMP/p384.pem" --slot 1 --out "$TMP/p384.json"

# reject_rehearsal_key must fail CLOSED when it has nothing to compare against.
mv "$WORKDIR/my-key.pem" "$WORKDIR/my-key.pem.bak"
run_phase "--make-otp-json refuses when a rehearsal key is missing" expect-fail "" \
  --make-otp-json --key "$TMP/sh2-boot-key.pem" --slot 3 --out "$TMP/x3.json"
mv "$WORKDIR/my-key.pem.bak" "$WORKDIR/my-key.pem"

# ...and the documented escape hatch must work.
mv "$WORKDIR/my-key.pem" "$WORKDIR/my-key.pem.bak"
export ALLOW_UNCHECKED_KEY=1
run_phase "ALLOW_UNCHECKED_KEY=1 overrides it deliberately" expect-pass "" \
  --make-otp-json --key "$TMP/sh2-boot-key.pem" --slot 3 --out "$TMP/x3.json"
unset ALLOW_UNCHECKED_KEY
mv "$WORKDIR/my-key.pem.bak" "$WORKDIR/my-key.pem"

# ...and the SH2 modes must refuse anything that is not one.
export OTPSTATE="$TMP/pico.state"
run_phase "--sh2-precheck refuses a non-SeedHammer board" expect-fail "" --sh2-precheck

########################################################################
hdr "RESULT"
printf '  passed: %d\n  failed: %d\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ] && { printf '\033[32mALL CHECKS PASSED\033[0m\n'; exit 0; }
printf '\033[31m%d CHECK(S) FAILED\033[0m\n' "$FAIL"; exit 1
