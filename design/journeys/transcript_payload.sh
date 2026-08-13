#!/usr/bin/env bash
# The host half of the Load Payload journey, captured verbatim -- including the
# place the toolchain refuses. Failures here are RESULTS, not script bugs; each
# is reported with its real exit code.
#
# Never `cmd && echo OK`: on failure that prints nothing at all, which reads as
# absence rather than as a failure (F-147). run() prints the exit code
# unconditionally, and takes it from the command rather than from a pipe -- a
# `| head` in between reports HEAD's status, which has hidden a real exit 4 from
# this very toolchain more than once.
set -u

W="$(cd "$(dirname "$0")" && pwd)"
C=/scratch/code/shibboleth
ME=$C/mnemonic-engrave/target/release/me
FORK=$C/seedhammer
OUT="$W/out"
mkdir -p "$OUT"

run() { echo "\$ $*"; "$@" 2>&1; echo "[exit $?]"; echo; }

echo "########## versions"
run "$ME" --version

echo "########## 1. the three records an operator writes"
run cat "$W/inputs-payload/records.txt"

echo "########## 2. OBSTACLE -- the obvious way to write them is refused"
echo "This is what anyone writes the first time: the passphrase and the free"
echo "text as themselves. Spec 5.3.1 makes text:/pass: bodies LOWERCASE HEX,"
echo "and a non-hex body is ClassUnknown rather than free text -- so the"
echo "container is refused rather than silently carrying a record no program"
echo "will claim."
echo
run cat "$W/inputs-payload/records-as-first-written.txt"
run "$ME" sysw pack --no-passphrase --in "$W/inputs-payload/records-as-first-written.txt" --out "$OUT/rejected.bin"

echo "########## 3. so the bodies are hex-encoded, which is a one-liner"
run bash -c "printf 'correct horse battery staple' | xxd -p -c 256"
run bash -c "printf 'SEEDHAMMER II DEMO PAYLOAD' | xxd -p -c 256"

echo "########## 4. pack -- the container"
rm -f "$OUT/payload.bin"
run "$ME" sysw pack --no-passphrase --in "$W/inputs-payload/records.txt" --out "$OUT/payload.bin"
run stat -c '%n  %s bytes' "$OUT/payload.bin"
run bash -c "xxd -l 16 '$OUT/payload.bin'"

echo "########## 5. show -- what it holds, and the digest the operator compares"
run "$ME" sysw show "$OUT/payload.bin"

echo "########## 6. the same bytes are what the emulator embeds"
echo "cmd/emu/sysw_test_payload.bin in the fork is this container. If these"
echo "differ, the journey's device screenshots are of a different payload than"
echo "the one this transcript packed, and the document is wrong in the one way"
echo "a reader cannot detect."
echo
run cmp "$OUT/payload.bin" "$FORK/cmd/emu/sysw_test_payload.bin"
run bash -c "sha256sum '$OUT/payload.bin' '$FORK/cmd/emu/sysw_test_payload.bin'"
echo "\$ grep syswTestDigest $FORK/cmd/emu/sysw_test_payload.go"
grep -n 'const syswTestDigest' "$FORK/cmd/emu/sysw_test_payload.go" 2>&1
echo "[exit $?]"
echo

echo "########## 7. --region: the full 65536-byte image that is written to flash"
echo "The tail is 0xFF, the ERASED state of NOR flash, so this is byte-for-byte"
echo "what the sector looks like with only the container written. Zero-padding"
echo "would be a WRITE of 64 KiB to say nothing."
echo
rm -f "$OUT/payload-region.bin"
run "$ME" sysw pack --no-passphrase --in "$W/inputs-payload/records.txt" --region --out "$OUT/payload-region.bin"
run stat -c '%n  %s bytes' "$OUT/payload-region.bin"
run bash -c "xxd -s 260 -l 32 '$OUT/payload-region.bin'"

echo "########## 8. padding does not change the digest"
echo "The digest is over the container's public records, not over the region"
echo "image -- so the value the operator compares on the device is the same"
echo "whichever form was written."
echo
run "$ME" sysw show "$OUT/payload-region.bin"

echo "########## 9. wipe -- the overwrite image (spec 5.5), a HOST operation"
echo "The firmware never writes flash (spec 13 D10). Removing a payload is done"
echo "from the host, over USB, with the machine in BOOTSEL -- the same way it"
echo "was put there. --fill ones is the erased state, leaving a region"
echo "indistinguishable from one never written."
echo
rm -f "$OUT/wipe-ones.bin" "$OUT/wipe-random.bin"
run "$ME" sysw wipe --fill ones --out "$OUT/wipe-ones.bin"
run "$ME" sysw wipe --out "$OUT/wipe-random.bin"
run stat -c '%n  %s bytes' "$OUT/wipe-ones.bin" "$OUT/wipe-random.bin"
run bash -c "xxd -l 16 '$OUT/wipe-ones.bin'"
run bash -c "xxd -l 16 '$OUT/wipe-random.bin'"

echo "########## 10. a wiped region carries no container"
run "$ME" sysw show "$OUT/wipe-ones.bin"

echo "########## 11. the delivery step, on a machine in BOOTSEL"
echo "picotool lives in the fork's nix devshell, not on PATH -- and NOT piped"
echo "through head, which is how the first version of this section reported"
echo "[exit 0] for a 'nix: command not found'. That is F-147 exactly, in the"
echo "script whose header says it avoids F-147."
echo
NIX=/nix/var/nix/profiles/default/bin/nix
run bash -c "cd '$FORK' && $NIX develop --command picotool version"
echo "\$ picotool load --verify -t bin -o 0x10D00000 out/payload-region.bin"
echo "(not run here: it requires the machine attached in BOOTSEL. It HAS been"
echo "run -- the payload is on the engraver, and the device shows the digest"
echo "section 5 printed above.)"
echo
