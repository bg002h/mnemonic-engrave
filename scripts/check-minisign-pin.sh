#!/bin/sh
# check-minisign-pin.sh — every hard-coded copy of the release-signing public
# key must equal minisign.pub (the file shipped inside every archive).
#
# The key is written out in: .github/workflows/release.yml (VERIFY.txt text and
# the verify step), README.md (key block and verify command) and
# scripts/release-workflows/gen.py (the generated workflows' MINISIGN_PUBKEY).
# A rotation that misses one would ship a minisign.pub that disagrees with the
# verify step, and nothing else would fail. The one exception is the RETIRED
# key, allowed only on README lines that say "retired" (the <= v0.12.0 note).
#
# Usage: sh scripts/check-minisign-pin.sh [repo-root]   (default: this repo)
set -eu

root=${1:-$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)}
RETIRED="RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW"
fail=0
TMP=$(mktemp)
trap 'rm -f "$TMP"' EXIT
bad() { printf 'FAIL %s\n' "$1"; fail=1; }

PIN=$(sed -n 2p "$root/minisign.pub")
printf '%s\n' "$PIN" | grep -qxE 'RW[A-Za-z0-9+/]{54}' \
    || { echo "FAIL minisign.pub line 2 is not a minisign public key: '$PIN'"; exit 1; }

# check <file> <minimum copies of the pin expected>
check() {
    _f="$root/$1"
    [ -f "$_f" ] || { bad "$1 is missing"; return; }
    _n=0
    # every key-shaped token, with its line, one per output line
    grep -n -oE 'RW[A-Za-z0-9+/]{54}' "$_f" > "$TMP" || true
    while IFS=: read -r _line _key; do
        if [ "$_key" = "$PIN" ]; then
            _n=$((_n + 1))
        elif [ "$_key" = "$RETIRED" ] && [ "$1" = README.md ] \
             && sed -n "${_line}p;$((_line - 1))p" "$_f" | grep -qi retired; then
            :
        else
            bad "$1:$_line carries $_key, not the pinned $PIN"
        fi
    done < "$TMP"
    if [ "$_n" -lt "$2" ]; then
        bad "$1 carries the pin $_n time(s); expected at least $2"
    else
        printf 'ok   %s: %s copies of the pin\n' "$1" "$_n"
    fi
}

check .github/workflows/release.yml 2
check README.md 2
check scripts/release-workflows/gen.py 1

if [ "$fail" -eq 0 ]; then
    echo "check-minisign-pin: OK (pin $PIN)"
else
    echo "check-minisign-pin: FAILED" >&2
    exit 1
fi
