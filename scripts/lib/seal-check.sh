# shellcheck shell=bash
#
# seal-check.sh -- the load-map check shared by scripts/sign-firmware.sh and
# scripts/test/seal-clear-test.sh. Sourced, never executed.
#
# `picotool seal --clear` adds signed load-map entry 0,
# `Clear 0x20000000->0x20082000`: the RP2350 bootrom zeroes all of main SRAM
# before the image runs (design/agent-reports/e3a-seal-clear-investigation.md
# section 3). The entry is inside the signed hash, so it cannot be stripped
# without breaking the signature -- but an image can be signed perfectly well
# WITHOUT it: sealed elsewhere without --clear, or resealed from an input that
# already carried a load map (picotool then ignores --clear silently). Such an
# image prints `signature: verified` and wipes nothing. So a signer must check
# for the entry, not only for the signature.
#
# The line is printed by picotool 2.3.1 main.cpp:3824 (`info_pair("load map
# entry " + i, ...)`, with the "Clear 0x..->0x.." text built at :3809-3810);
# 2.2.0-a4 prints the same text (measured 2026-10-05). The match is
# whitespace-tolerant and anchored to the whole line, and requires the entry to
# be entry 0 with exactly the RP2350 main-SRAM range.

SEAL_CLEAR_ENTRY_RE='^[[:space:]]*load map entry 0:[[:space:]]+Clear 0x20000000->0x20082000$'

# seal_has_clear_entry <captured `picotool info -a` output>
# Returns 0 when the output carries the Clear entry, 1 otherwise. A here-string,
# never `printf | grep -q`, for captured output (F-695: pipefail + SIGPIPE).
seal_has_clear_entry() {
  grep -qE "$SEAL_CLEAR_ENTRY_RE" <<<"$1"
}
