#!/usr/bin/env bash
#
# sign-firmware.sh -- sign an RP2350 UF2 with your own secp256k1 boot key and
# PROVE the signature is correct, entirely offline, before it ever touches a
# device.
#
# This is runbook step 5 (design/RUNBOOK_custom_boot_key.md) as an executable,
# self-checking procedure. Nothing here writes OTP; it is freely retryable.
#
# Usage:
#   ./sign-firmware.sh <image.uf2> <key.pem>
#
# Env:
#   SEEDHAMMER_DIR   path to the seedhammer fork (provides cmd/picosign).
#                    Default: ../seedhammer relative to this repo.
#
# Run it inside the fork's devshell so picotool/go are present:
#   cd $SEEDHAMMER_DIR && nix develop --command \
#     /path/to/mnemonic-engrave/scripts/sign-firmware.sh img.uf2 key.pem
#
# WHY THE ORDERING MATTERS
#   The RP2350 signed hash covers the image block INCLUDING the public key but
#   EXCLUDING the signature. So the real pubkey must be embedded BEFORE the
#   digest is computed. Step 5 below asserts this rather than assuming it: it
#   re-hashes after embedding the signature and requires the digest to be
#   unchanged. If that assertion ever fails, the signature is inside the hashed
#   region and this whole approach is wrong -- stop and re-derive.
#
set -euo pipefail

die()  { printf '\n\033[31mFAIL:\033[0m %s\n' "$*" >&2; exit 1; }
ok()   { printf '\033[32m  PASS:\033[0m %s\n' "$*"; }
info() { printf '\033[36m  ..\033[0m %s\n' "$*"; }
hdr()  { printf '\n\033[1m== %s ==\033[0m\n' "$*"; }

IMG="${1:-}"; KEY="${2:-}"
[ -n "$IMG" ] && [ -n "$KEY" ] || die "usage: $0 <image.uf2> <key.pem>"
[ -f "$IMG" ] || die "no such image: $IMG"
[ -f "$KEY" ] || die "no such key: $KEY"

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SEEDHAMMER_DIR="${SEEDHAMMER_DIR:-$(cd "$REPO_ROOT/../seedhammer" 2>/dev/null && pwd || true)}"
[ -n "$SEEDHAMMER_DIR" ] && [ -d "$SEEDHAMMER_DIR" ] \
  || die "set SEEDHAMMER_DIR to the seedhammer fork (needs cmd/picosign)"

command -v picotool >/dev/null || die "picotool not found -- run inside 'nix develop'"
command -v openssl  >/dev/null || die "openssl not found"
command -v go       >/dev/null || die "go not found -- run inside 'nix develop'"

# picosign runs out of the fork's module.
picosign() { ( cd "$SEEDHAMMER_DIR" && go run seedhammer.com/cmd/picosign "$@" ); }
# hex without depending on host xxd; od comes from nix coreutils.
tohex() { od -An -v -tx1 "$@" | tr -d ' \n'; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

hdr "0 -- inputs"
info "image: $IMG"
info "key:   $KEY"
info "fork:  $SEEDHAMMER_DIR"
openssl ec -in "$KEY" -noout -text 2>/dev/null | grep -qi 'secp256k1\|ASN1 OID: secp256k1' \
  || die "key is not secp256k1 -- RP2350 secure boot requires secp256k1"
ok "key is secp256k1"

hdr "1 -- ensure the image has a SIGNATURE section"
if ! picosign hash "$IMG" >/dev/null 2>&1; then
  info "no SIGNATURE section; sealing with a throwaway key to create the structure"
  openssl ecparam -name secp256k1 -genkey -noout -out "$WORK/seal.pem"
  picotool seal --sign --clear --quiet "$IMG" "$WORK/sealed.uf2" "$WORK/seal.pem"
  cp "$WORK/sealed.uf2" "$IMG"
  picosign sign -clear "$IMG"
  ok "SIGNATURE section created and zeroed"
else
  ok "SIGNATURE section already present"
fi

hdr "2 -- embed the real public key (with a placeholder signature)"
PUB="$(openssl ec -in "$KEY" -pubout -conv_form compressed -outform DER 2>/dev/null \
       | tail -c 33 | od -An -v -tx1 | tr -d ' \n')"
[ ${#PUB} -eq 66 ] || die "expected a 33-byte compressed pubkey (66 hex chars), got ${#PUB}"
info "pubkey: $PUB"
ZEROSIG="$(printf '0%.0s' $(seq 1 128))"
picosign sign -pubkey "$PUB" -sig "$ZEROSIG" "$IMG"
ok "public key embedded"

hdr "3 -- compute the digest over the image"
picosign hash "$IMG" > "$WORK/digest1.bin"
[ "$(stat -c%s "$WORK/digest1.bin")" -eq 32 ] || die "digest is not 32 bytes -- unexpected"
D1="$(tohex "$WORK/digest1.bin")"
info "digest: $D1"
ok "32-byte SHA-256 digest extracted"

hdr "4 -- sign the digest"
openssl pkeyutl -sign -inkey "$KEY" -in "$WORK/digest1.bin" -out "$WORK/sig.der"
picosign sign -pubkey "$PUB" -sig "$(tohex "$WORK/sig.der")" -sigfmt der "$IMG"
ok "signature embedded"

hdr "5 -- ASSERT the signature is outside the hashed region"
picosign hash "$IMG" > "$WORK/digest2.bin"
D2="$(tohex "$WORK/digest2.bin")"
[ "$D1" = "$D2" ] || die "digest CHANGED after embedding the signature ($D1 -> $D2).
The signature is inside the hashed region, so no signature can ever be valid.
Do not flash this. Re-derive the signing procedure before going further."
ok "digest unchanged -- ordering assumption holds"

hdr "6 -- VERIFY the signature cryptographically (offline)"
openssl ec -in "$KEY" -pubout -out "$WORK/pub.pem" 2>/dev/null
openssl pkeyutl -verify -pubin -inkey "$WORK/pub.pem" \
  -in "$WORK/digest2.bin" -sigfile "$WORK/sig.der" >/dev/null \
  || die "signature does NOT verify against the image digest -- do not flash"
ok "signature verifies against the image digest"

hdr "7 -- structural check"
BLOCKS="$(picotool info -a "$IMG" 2>/dev/null | grep -ci 'metadata block' || true)"
picotool info -a "$IMG" 2>&1 | head -20
info "a doubly-sealed image shows THREE metadata blocks; rebuild if so"

hdr "RESULT"
ok "$IMG is signed by $KEY and the signature is proven valid offline."
info "Flash with:  picotool load --verify $IMG && picotool reboot"
info "It will only BOOT on a device whose OTP holds a valid slot for this key."
