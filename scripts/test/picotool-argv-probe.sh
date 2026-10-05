#!/usr/bin/env bash
#
# picotool-argv-probe.sh -- run every argv shape the argv builder in
# scripts/lib/otp-read.sh can emit against the REAL picotool, with no board
# attached (plan E3a section 6.4; CI job `picotool argv probe`).
#
# UNPLUG EVERY BOARD FIRST. The script refuses to start, and aborts mid-run,
# if picotool can see any board.
#
# picotool's parser matches option groups in declaration order, and an option
# out of place silently becomes a selector (fact F14): the call then runs on
# whatever board is attached. With no board attached, a call whose --ser was
# parsed fails with exit 249 and "...found with serial number <S>." (wrapped
# before the serial when stdout is not a terminal, F18); a call whose --ser was
# NOT parsed fails with 249 and no serial. So every builder shape must give 249
# AND, after joining wrapped lines, "with serial number ZZPROBE00000000.".
#
# Also checked: `version -s` is the pin, the `otp list` fingerprint, the bare
# board-count `info` (no --ser by design), and a negative control: the
# misordered argv F-619 measured (`otp get -n -c 1 --ser S 0x04b`) must NOT
# carry the serial, which shows the check can tell the two apart.
#
#   scripts/test/picotool-argv-probe.sh [path/to/picotool]
#
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=../lib/otp-read.sh
. "$HERE/../lib/otp-read.sh"

PT="${1:-$(command -v picotool || true)}"
[ -n "$PT" ] && [ -x "$PT" ] || { echo "picotool-argv-probe: no picotool (pass its path)"; exit 2; }
SER="ZZPROBE00000000"
WANT="with serial number $SER."

DIR="$(mktemp -d "${TMPDIR:-/tmp}/argv-probe.XXXXXX")"
trap 'rm -rf "$DIR"' EXIT
case "$DIR" in *[[:space:]]*) echo "picotool-argv-probe: temp dir has a space: $DIR"; exit 2 ;; esac
# Files the shapes name. None is ever written to a device: no board is attached.
printf '{}\n' > "$DIR/probe.json"
head -c 4096 /dev/zero | tr '\0' '\377' > "$DIR/probe.bin"

PASS=0; FAIL=0
ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf ' FAIL  %s\n' "$*"; FAIL=$((FAIL + 1)); }

OUT=""; RC=0
probe() { # probe <argv...> -> OUT (wrapped lines joined), RC
  RC=0
  OUT="$("$PT" "$@" 2>&1)" || RC=$?
  OUT="$(printf '%s' "$OUT" | tr '\n' ' ' | tr -s ' ')"
}

echo "picotool: $PT"

# NO BOARD MAY BE ATTACHED. The shapes below include `otp set`, `erase` and
# `load`; only --ser keeps them off a board, and a regression in --ser parsing
# is exactly what this probe exists to catch. So before every device argv,
# require that picotool sees no RP-series board at all, and abort otherwise.
require_no_board() {
  local rc=0 out
  out="$("$PT" info 2>&1)" || rc=$?
  if [ "$rc" -ne 249 ] || [[ "$out" != *"No accessible"* ]]; then
    echo "picotool-argv-probe: ABORT -- a board may be attached (info exit $rc). Unplug every RP2040/RP2350 board and re-run."
    exit 2
  fi
}
require_no_board

probe version -s
if [ "$RC" -eq 0 ] && [ "${OUT% }" = "$PICOTOOL_PIN" ]; then ok "version -s = $PICOTOOL_PIN"
else bad "version -s printed '$OUT' (exit $RC), want $PICOTOOL_PIN"; fi

probe otp list -n MAC0
if [ "$RC" -eq 0 ] && [[ "$OUT" == *"ROW 0x0062: OTP_DATA_MAC0"* ]]; then ok "fingerprint: otp list -n MAC0 names row 0x0062"
else bad "fingerprint: otp list -n MAC0 printed '$OUT' (exit $RC)"; fi
probe otp list -n BOOT_FLAGS0.DISABLE_BOOTSEL_EXEC2
if [ "$RC" -eq 0 ] && [[ "$OUT" != *DISABLE_BOOTSEL_EXEC2* ]]; then ok "fingerprint: DISABLE_BOOTSEL_EXEC2 absent (pico-sdk 2.3.1)"
else bad "fingerprint: DISABLE_BOOTSEL_EXEC2 present or error: '$OUT' (exit $RC)"; fi

N=0
while IFS= read -r line; do
  [ -n "$line" ] || continue
  read -r -a argv <<<"$line"
  case "${argv[*]}" in
    "version -s"|"otp list -n "*) continue ;;   # checked above
    info)
      probe info
      if [ "$RC" -eq 249 ] && [[ "$OUT" == *"No accessible"* ]] && [[ "$OUT" != *"serial number"* ]]; then
        ok "info (bare board-count probe, no --ser by design): 249, no device"
      else bad "info: exit $RC '$OUT'"; fi
      continue ;;
  esac
  N=$((N + 1))
  require_no_board
  probe "${argv[@]}"
  if [ "$RC" -eq 249 ] && [[ "$OUT" == *"$WANT"* ]]; then ok "$line"
  else bad "$line -> exit $RC: $OUT"; fi
done < <(pt_print_argvs "$SER" "$DIR")
[ "$N" -gt 0 ] || bad "the builder's print mode produced no device argvs"

# Negative control: F-619's argument order. --ser is not parsed, so no serial.
require_no_board
probe otp get -n -c 1 --ser "$SER" 0x04b
if [ "$RC" -eq 249 ] && [[ "$OUT" != *"$WANT"* ]]; then
  ok "negative control: misordered 'otp get -n -c 1 --ser S 0x04b' loses --ser (the check discriminates)"
else bad "negative control: misordered argv gave exit $RC '$OUT' -- the probe cannot tell order apart"; fi

printf '\n  %d argv shape(s) probed; passed %d, failed %d\n' "$N" "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
echo "PROBE PASS"
