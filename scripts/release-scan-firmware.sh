#!/usr/bin/env bash
#
# release-scan-firmware.sh -- build the REAL firmware and look for the
# emulator's test payload in it. Release gate; not a unit test.
#
# WHY THIS EXISTS
#   cmd/emu carries a real sealed payload and the passphrase that opens it, and
#   it is on b2b heading for main. Two committed tests already argue it cannot
#   reach the device: every blob-bearing file is //go:build js, and cmd/emu is
#   `package main` which Go forbids importing. Both are mutation-tested.
#
#   But both are arguments about what the compiler SHOULD do. This is the only
#   check that looks at what it DID. It is what catches a build-tag mistake in a
#   dependency, a stray go:embed, a linker surprise -- or those two tests being
#   wrong.
#
#   That distinction has cost this project repeatedly: `go list -deps` returning
#   empty because build constraints excluded every file, and reading as absence;
#   a sed that never applied, reading as a surviving mutant. An argument that
#   something is not there is not a look.
#
# WHAT IT DOES NOT COVER
#   Only the needles below, in cmd/controller, for one target. It says nothing
#   about whether the firmware is otherwise correct, and nothing about the host.
#
# Usage: scripts/release-scan-firmware.sh [fork-path]
#   default fork-path: /scratch/code/shibboleth/seedhammer
# Exit: 0 clean, 1 a needle was found, 2 the scan could not run (treat as FAIL).

set -uo pipefail

FORK="${1:-/scratch/code/shibboleth/seedhammer}"
NIX=/nix/var/nix/profiles/default/bin/nix
OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

say() { printf '%s\n' "$*"; }
die() { printf '\n!! %s\n' "$*" >&2; exit 2; }

[ -d "$FORK" ] || die "no fork at $FORK"
[ -f "$FORK/cmd/emu/sealed_test_payload.bin" ] || \
  die "no cmd/emu/sealed_test_payload.bin in $FORK -- if the test payload was removed, delete this script too"

say "== 1 -- build cmd/controller for the real target =="
say "   $FORK"
# The exact flags .github/workflows/test.yml:29 uses, minus -size/-print-stacks
# and with a real output file rather than /dev/null.
build() { # $1 = output path
  ( cd "$FORK" && "$NIX" develop --command tinygo build -o "$1" \
      -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks \
      ./cmd/controller ) 2>&1 | tail -3
}
build "$OUT/fw.elf" || die "tinygo build (elf) failed"
build "$OUT/fw.uf2" || die "tinygo build (uf2) failed"
for f in "$OUT/fw.elf" "$OUT/fw.uf2"; do
  [ -s "$f" ] || die "$(basename "$f") is empty or missing -- a zero-byte artefact would scan CLEAN"
done
say "   elf $(stat -c%s "$OUT/fw.elf") bytes, uf2 $(stat -c%s "$OUT/fw.uf2") bytes"

say
say "== 2 -- scan =="
python3 - "$FORK" "$OUT/fw.elf" "$OUT/fw.uf2" <<'PY'
import pathlib, re, sys

fork, elf_p, uf2_p = (pathlib.Path(a) for a in sys.argv[1:4])
elf = elf_p.read_bytes()
uf2 = uf2_p.read_bytes()

# A UF2 is 512-byte blocks: 32-byte header, 476 bytes of payload, 4-byte trailing
# magic. Scanning the container raw would let a needle STRADDLE a block boundary
# and go unseen -- a blind spot that reads exactly like a clean result. So
# reassemble the flashed image before searching it.
def uf2_payload(b):
    if len(b) % 512:
        raise SystemExit("!! uf2 is not a whole number of 512-byte blocks; refusing to guess")
    out = bytearray()
    for i in range(0, len(b), 512):
        blk = b[i:i+512]
        if blk[:4] != b"UF2\n":
            raise SystemExit("!! uf2 block %d has no magic; refusing to scan a container I cannot parse" % (i//512))
        out += blk[32:32+int.from_bytes(blk[16:20], "little")]
    return bytes(out)

img = uf2_payload(uf2)
print(f"   flashed image reassembled from uf2: {len(img)} bytes ({len(uf2)//512} blocks)")

src = (fork/"cmd/emu"/"sealed_test_payload.go").read_text()
blob = (fork/"cmd/emu"/"sealed_test_payload.bin").read_bytes()
m = re.search(r'sealedTestPassphrase\s*=\s*"([^"]+)"', src)
if not m:
    raise SystemExit("!! could not read sealedTestPassphrase from source -- needles would be incomplete")
passphrase = m.group(1)

needles = [
    ("MNEMBLOB magic",          b"MNEMBLOB"),
    ("passphrase (full)",       passphrase.encode()),
    ("passphrase (first words)", b" ".join(passphrase.encode().split()[:3])),
    ("symbol sealedTestPassphrase", b"sealedTestPassphrase"),
    ("symbol sealedTestPayload",    b"sealedTestPayload"),
    ("symbol embeddedPayloadReader", b"embeddedPayloadReader"),
    ("package path cmd/emu",    b"seedhammer.com/cmd/emu"),
    ("blob (whole, 1536B)",     blob),
]
# High-entropy windows: ciphertext bytes are effectively unique, so a hit is the
# blob itself rather than a coincidental run.
#
# The entropy has to be MEASURED, not assumed from the offset. The first version
# sampled by offset and called the result high-entropy in a comment; it picked a
# window inside the blob's 115-byte trailing zero padding, and 32 zero bytes are
# in every binary ever linked. It reported a leak that was not there, on the
# first real run.
def usable(w):
    return len(set(w)) >= 20   # 32 ciphertext bytes have ~30 distinct; padding has 1

windows = []
step = max(1, (len(blob) - 32) // 24)
for off in range(0, len(blob) - 32, step):
    w = blob[off:off+32]
    if usable(w):
        windows.append((f"blob window @{off}", w))
    if len(windows) >= 12:
        break
if len(windows) < 6:
    raise SystemExit(f"!! only {len(windows)} high-entropy windows found in the blob; "
                     f"too few to scan meaningfully -- refusing to report clean")
print(f"   {len(windows)} high-entropy windows selected "
      f"(>=20 distinct bytes of 32; padding excluded by measurement)")
needles += windows

# POSITIVE CONTROL. A search that finds nothing because it is looking at the
# wrong bytes is indistinguishable from a clean result. Prove the haystack is
# really the firmware before believing anything about what is missing from it.
controls = [("control: seedhammer.com in elf", b"seedhammer.com", elf)]
ok = True
for name, needle, hay in controls:
    if needle in hay:
        print(f"   ok   {name} -- the scan is looking at real firmware")
    else:
        ok = False
        print(f"   !!   {name} NOT FOUND -- the scan is not reading the firmware; "
              f"its clean result would be meaningless")
if not ok:
    sys.exit(2)

hits = 0
for name, needle in needles:
    where = [n for n, hay in (("elf", elf), ("flashed image", img)) if needle in hay]
    if where:
        hits += 1
        print(f"   FOUND  {name}  in {', '.join(where)}")
if hits:
    print(f"\n!! {hits} needle(s) present in the firmware -- DO NOT RELEASE")
    sys.exit(1)
print(f"   none of {len(needles)} needles present in the elf or the flashed image")
PY
rc=$?

say
say "== RESULT =="
case $rc in
  0) say "   CLEAN -- no test-payload needle in the firmware"
     say "   NOT covered: only these needles, only cmd/controller, only pico-plus2." ;;
  1) say "   FAILED -- see above. Do not sign or tag." ;;
  *) say "   COULD NOT SCAN -- treat as a failure, not as a pass." ;;
esac
exit $rc
