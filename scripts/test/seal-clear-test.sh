#!/usr/bin/env bash
#
# seal-clear-test.sh -- prove that a picotool build can `seal --sign --clear` a
# UF2 and that the result verifies (F-701; plan
# design/IMPLEMENTATION_PLAN_e3a_picotool_seal_patch.md section 2).
#
#   scripts/test/seal-clear-test.sh <picotool> [--expect-fail]
#
# Stock picotool 2.3.1 cannot do this: two upstream bugs from commit 3c743bd
# (design/agent-reports/e3a-seal-clear-investigation.md section 1). Bug A makes
# the UF2/BIN seal path use an unknown model, so `--clear` exits 248 "unknown
# sram end". Bug B, hidden behind A, hashes the image before the Clear size word,
# so picotool's own `--clear` signature is wrong. This repo's flake carries
# nix/patches/picotool-2.3.1-seal-clear-fix.patch, which fixes both.
#
# Default mode (the patched build, CI's `.#picotool`):
#   1. `seal --sign --clear` exits 0;
#   2. `info -a` prints `signature: verified` and never any other verdict, and
#      prints `load map entry 0: Clear 0x20000000->0x20082000`;
#   3. tamper control: one flipped payload byte turns the verdict to
#      `incorrect`, so the check can tell a right signature from a wrong one;
#   4. P4 regression guard: `seal --sign` WITHOUT --clear exits 0, verifies,
#      and carries no Clear entry;
#   5. the shared Clear-entry check (scripts/lib/seal-check.sh) that
#      scripts/sign-firmware.sh refuses on accepts (2) and REFUSES (4), and
#      sign-firmware.sh is wired to it.
# --expect-fail (the unpatched build, CI's `.#picotool-unpatched`): step 1 must
#   exit 248 with "unknown sram end" -- the negative control that shows the test
#   can tell patched from unpatched. Steps 4 and 5 still run; step 3 is skipped
#   (there is no sealed --clear output to tamper with).
#
# What each half of the patch is held to: a build without hunk A fails step 1
# (248); a build with hunk A but not hunk B seals, but its own signature fails
# step 2 (`incorrect`). The tamper control is what makes step 2's `verified`
# mean something. CI has a negative control only for hunk A (the unpatched
# build); hunk B's red run is the one-time mutation run recorded in
# design/agent-reports/e3a-seal-patch-impl-report.md.
#
# A third picotool bug, unrelated to the patch and present in 2.2.0-a4 too
# (der_to_raw, below), makes a fraction of a percent of picotool's own
# signatures wrong at random. A seal is retried with a new key, at most twice,
# only when its signature carries that bug's exact fingerprint; anything else is
# judged on the first attempt.
#
# No device is touched: `info -a` always gets a file argument. Each seal uses a
# throwaway key generated for it.
#
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
# shellcheck source-path=SCRIPTDIR
# shellcheck source=../lib/seal-check.sh
. "$REPO/scripts/lib/seal-check.sh"

PT="${1:-}"
MODE="${2:-}"
[ -n "$PT" ] && [ -x "$PT" ] || { echo "seal-clear-test: usage: $0 <picotool> [--expect-fail]"; exit 2; }
case "$MODE" in ""|--expect-fail) ;; *) echo "seal-clear-test: unknown option '$MODE'"; exit 2 ;; esac

FIXTURE="$HERE/fixtures/seal/blinky.uf2"
FIXTURE_SHA256="e48659d727367cd2c23bffa6ad4b56d7388f83c9f9e3a6b4998c28431f8454a9"
CLEAR_LINE="load map entry 0:    Clear 0x20000000->0x20082000"

PASS=0; FAIL=0
ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf ' FAIL  %s\n' "$*"; FAIL=$((FAIL + 1)); }

echo "picotool: $PT${MODE:+ ($MODE)}"

# The fixture first: every verdict below is about THIS image.
GOT_SHA="$(sha256sum "$FIXTURE" 2>/dev/null | cut -d' ' -f1)"
if [ "$GOT_SHA" != "$FIXTURE_SHA256" ]; then
  echo "seal-clear-test: fixture $FIXTURE has sha256 '$GOT_SHA', want $FIXTURE_SHA256"
  exit 2
fi
ok "fixture sha256 $FIXTURE_SHA256"

DIR="$(mktemp -d "${TMPDIR:-/tmp}/seal-clear.XXXXXX")"
trap 'rm -rf "$DIR"' EXIT

# sig_verdict <info -a output> -> prints "verified", "incorrect", "none" or
# "mixed". The `signature:` line prints twice in `info -a` (Program Information
# and the signed Metadata Block); 2.3.1 main.cpp:3868 prints it, as
# `verified` or `incorrect`. Lines are matched whole, so `signature value:` is
# never counted.
sig_verdict() {
  local nv ni nall
  nall="$(grep -cE '^[[:space:]]*signature:' <<<"$1")"
  nv="$(grep -cE '^[[:space:]]*signature:[[:space:]]+verified$' <<<"$1")"
  ni="$(grep -cE '^[[:space:]]*signature:[[:space:]]+incorrect$' <<<"$1")"
  if [ "$nall" -eq 0 ]; then echo none
  elif [ "$nv" -eq "$nall" ]; then echo verified
  elif [ "$ni" -eq "$nall" ]; then echo incorrect
  else echo mixed; fi
}

# der_to_raw_defect <info -a output>: true when the printed signature value
# carries the fingerprint of a THIRD picotool bug, independent of the patch and
# present in 2.2.0-a4 and 2.3.1 alike: bintool/mbedtls_wrapper.c der_to_raw
# (2.3.1 :169 for r, :179 for s) copies a DER integer shorter than 32 bytes
# with `memcpy(r + (32 - b2), der + 4, (32 - b2))` -- length 32-b2 instead of
# b2 -- so a half whose value is below 2^247 (a 31-byte DER integer) is stored as k zero bytes, the
# first k bytes of the integer, then zeros (k = 32 - b2). picotool's own
# signature is then wrong (measured 2026-10-05: 7 failing runs in 500 on the
# patched build, 3 in 200 on 2.2.0-a4, 3 bad signatures in 1500 seals). The
# pattern is matched exactly for k = 1..4. A retry can only go green if a
# fresh seal then verifies through the verifier's own entry-order hash, so a
# deterministic wrong digest (a hunk-B defect) can never be retried into a
# pass; its random-looking signature also matches the pattern with
# probability below 2^-223.
der_to_raw_defect() {
  local line hex half k
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*signature\ value:[[:space:]]+([0-9A-Fa-f]{128})$ ]] || continue
    hex="${BASH_REMATCH[1]^^}"
    for half in "${hex:0:64}" "${hex:64:64}"; do
      for k in 1 2 3 4; do
        if [[ "$half" =~ ^(00){$k}[0-9A-F]{$((2 * k))}(00){$((32 - 2 * k))}$ ]]; then return 0; fi
      done
    done
  done <<<"$1"
  return 1
}

# seal_attempts <out.uf2> [seal options...]: seal the fixture with a fresh
# throwaway key and run `info -a` on the result; sets RC, OUT, INFO and V.
# Retries with a new key, at most twice, ONLY when the verdict is `incorrect`
# AND der_to_raw_defect matches -- the signer bug above, which no fix here
# touches. Every other result, including a deterministic `incorrect` and any
# non-zero exit, is final on the first attempt. Each retry is printed.
seal_attempts() {
  local out="$1" attempt; shift
  for attempt in 1 2 3; do
    openssl ecparam -name secp256k1 -genkey -noout -out "$DIR/key.pem" 2>/dev/null \
      || { echo "seal-clear-test: openssl could not generate a secp256k1 key"; exit 2; }
    rm -f "$out"
    RC=0; INFO=""; V=""
    OUT="$("$PT" seal --sign "$@" --quiet "$FIXTURE" "$out" "$DIR/key.pem" 2>&1)" || RC=$?
    [ "$RC" -eq 0 ] && [ -s "$out" ] || return 0
    INFO="$("$PT" info -a "$out" 2>&1)"
    V="$(sig_verdict "$INFO")"
    if [ "$V" = "incorrect" ] && der_to_raw_defect "$INFO" && [ "$attempt" -lt 3 ]; then
      printf '  note seal %s attempt %d: picotool der_to_raw short-integer signature; resealing with a new key\n' "${*:-(no options)}" "$attempt"
      continue
    fi
    return 0
  done
}

# ---------------------------------------------------------------------------
# 1-3. seal --sign --clear
# ---------------------------------------------------------------------------
seal_attempts "$DIR/clear.uf2" --clear

if [ "$MODE" = "--expect-fail" ]; then
  if [ "$RC" -eq 248 ] && [[ "$OUT" == *"unknown sram end"* ]]; then
    ok "unpatched: seal --sign --clear exits 248 \"unknown sram end\" (bug A)"
  else
    bad "unpatched: seal --sign --clear gave exit $RC, want 248 \"unknown sram end\": $OUT"
  fi
else
  if [ "$RC" -eq 0 ] && [ -s "$DIR/clear.uf2" ]; then
    ok "seal --sign --clear exits 0"
    INFO_C="$INFO"
    if [ "$V" = "verified" ]; then ok "--clear output: every signature line is 'verified'"
    else bad "--clear output: signature verdict '$V', want verified"; fi
    # 2.3.1 main.cpp:3824 prints the entry (text built at :3809-3810).
    if grep -qxF " $CLEAR_LINE" <<<"$INFO_C"; then ok "--clear output: '$CLEAR_LINE'"
    else bad "--clear output: no line '$CLEAR_LINE'"; fi
    if seal_has_clear_entry "$INFO_C"; then ok "shared Clear check (sign-firmware.sh) accepts the --clear output"
    else bad "shared Clear check rejects the --clear output"; fi

    # 3. Tamper control: XOR 0x01 into byte 32 of the sealed UF2 -- block 0's
    # first payload byte. Assert block 0 targets 0x10000000 (UF2 header word at
    # offset 12, little-endian) so the flipped byte is in the hashed range.
    cp "$DIR/clear.uf2" "$DIR/tamper.uf2"
    TGT="$(od -An -v -tx1 -j12 -N4 "$DIR/tamper.uf2" | tr -d ' \n')"
    if [ "$TGT" != "00000010" ]; then
      bad "tamper: block 0 targets $TGT (LE), want 00000010 = 0x10000000"
    else
      B="$(od -An -v -tu1 -j32 -N1 "$DIR/tamper.uf2" | tr -d ' \n')"
      # shellcheck disable=SC2059  # the format is a computed octal escape
      printf "\\$(printf '%03o' $((B ^ 1)))" \
        | dd of="$DIR/tamper.uf2" bs=1 seek=32 count=1 conv=notrunc 2>/dev/null
      B2="$(od -An -v -tu1 -j32 -N1 "$DIR/tamper.uf2" | tr -d ' \n')"
      if [ "$B2" -ne $((B ^ 1)) ]; then
        bad "tamper: byte 32 is $B2 after the flip, want $((B ^ 1))"
      else
        V="$(sig_verdict "$("$PT" info -a "$DIR/tamper.uf2" 2>&1)")"
        if [ "$V" = "incorrect" ]; then ok "tamper control: one flipped payload byte -> 'incorrect'"
        else bad "tamper control: verdict '$V' after a payload flip, want incorrect"; fi
      fi
    fi
  else
    bad "seal --sign --clear gave exit $RC (want 0): $OUT"
  fi
fi

# ---------------------------------------------------------------------------
# 4-5. seal --sign without --clear (both modes), and the refusal
# ---------------------------------------------------------------------------
seal_attempts "$DIR/noclear.uf2"
if [ "$RC" -eq 0 ] && [ -s "$DIR/noclear.uf2" ]; then
  ok "seal --sign (no --clear) exits 0"
  INFO_N="$INFO"
  if [ "$V" = "verified" ]; then ok "no --clear output: every signature line is 'verified'"
  else bad "no --clear output: signature verdict '$V', want verified"; fi
  if grep -qE '^[[:space:]]*load map entry [0-9]+:[[:space:]]+Clear ' <<<"$INFO_N"; then
    bad "no --clear output carries a Clear entry"
  else
    ok "no --clear output carries no Clear entry"
  fi
  if seal_has_clear_entry "$INFO_N"; then
    bad "shared Clear check ACCEPTS a verified image with no Clear entry"
  else
    ok "shared Clear check refuses a verified image with no Clear entry"
  fi
else
  bad "seal --sign (no --clear) gave exit $RC (want 0): $OUT"
fi

# sign-firmware.sh must actually call the shared check on its final `info -a`
# capture, and die when it fails; otherwise the refusal above guards nothing.
SF="$REPO/scripts/sign-firmware.sh"
if grep -qE '^[^#]*\. "\$REPO_ROOT/scripts/lib/seal-check\.sh"' "$SF" \
   && grep -qE '^[^#]*seal_has_clear_entry "\$INFO"[[:space:]]*\\?$' "$SF" \
   && SFW="$(grep -A1 -E '^[^#]*seal_has_clear_entry "\$INFO"' "$SF")" \
   && grep -qE '\|\|[[:space:]]*die ' <<<"$SFW"; then
  ok "sign-firmware.sh sources seal-check.sh and dies when seal_has_clear_entry fails"
else
  bad "sign-firmware.sh is not wired to the shared Clear check (source + seal_has_clear_entry \"\$INFO\" || die)"
fi

echo
echo "seal-clear-test: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
