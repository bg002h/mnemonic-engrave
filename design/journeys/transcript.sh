#!/usr/bin/env bash
# Replays the host half of the journey, capturing each command and its real
# output. Nothing here is illustrative: every block in the PDF is this script's
# stdout+stderr, verbatim.
set -u

W="$(cd "$(dirname "$0")" && pwd)"
C=/scratch/code/shibboleth
MD=$C/descriptor-mnemonic/target/release/md
MK=$C/mnemonic-key/target/release/mk
MS=$C/mnemonic-secret/target/release/ms
ME=$C/mnemonic-engrave/target/release/me

# Keep the sidecar discoverable: `me` looks beside its own executable only.
export PATH="$C/mnemonic-engrave/target/release:$PATH"

run() {
  echo "\$ $*"
  "$@" 2>&1
  echo "[exit $?]"
  echo
}

# runcap <outfile> <keep-regex> <cmd...>
#
# Runs <cmd...> like run() does for transcript purposes, and ALSO writes the
# lines of its STDOUT matching <keep-regex> to <outfile>.
#
# F-210: `run()` echoes a command's output into the transcript and throws it
# away, so three intermediates below (`md-encode-raw.txt`, `mk-encode-raw.txt`,
# `ms-encode.txt`) were READ by later steps that nothing ever WROTE. The
# journey only appeared to work because some earlier, uncommitted process had
# left those files in out/ — which is also why the committed transcript shows
# `mk inspect` consuming an mk1 that the `mk encode` directly above it never
# produced.
#
# The regex is required, not optional: `md encode` prints `chunk-set-id: 0x…`
# on stdout alongside the md1 lines. Capturing raw stdout would replace "file
# missing" with "file subtly wrong", which is harder to notice.
runcap() {
  local out="$1" keep="$2"; shift 2
  echo "\$ $*"
  local sout serr rc
  sout="$(mktemp)"; serr="$(mktemp)"
  "$@" >"$sout" 2>"$serr"; rc=$?
  cat "$sout"; cat "$serr"
  grep -E "$keep" "$sout" > "$out" || true
  rm -f "$sout" "$serr"
  echo "[exit $rc]"
  echo
}

mkdir -p "$W/out"

echo "########## versions"
run "$MD" --version
run "$MK" --version
run "$MS" --version
run "$ME" --version
run "$C/mnemonic-engrave/target/release/me-preview" --version

echo "########## 1. the wallet policy, from file"
run cat "$W/inputs/wallet-policy.txt"
runcap "$W/out/md-encode-raw.txt" '^md1' \
  "$MD" encode --group-size 0 "$(cat "$W/inputs/wallet-policy.txt")"
run "$MD" bytecode "$(grep '^md1' "$W/out/md-encode-raw.txt")"
run "$MD" inspect "$(grep '^md1' "$W/out/md-encode-raw.txt")"

echo "########## 2. one cosigner key, from file"
run cat "$W/inputs/keys/cosigner-00.xpub"
XPUB=$(grep '^xpub' "$W/inputs/keys/cosigner-00.xpub")
MD1=$(grep '^md1' "$W/out/md-encode-raw.txt")
runcap "$W/out/mk-encode-raw.txt" '^mk1' \
  "$MK" encode --xpub "$XPUB" --origin-fingerprint ae6647ee \
  --origin-path "m/48'/0'/0'/2'" --from-md1 "$MD1" --group-size 0
run "$MK" inspect $(sed -n '1,2p' "$W/out/mk-encode-raw.txt")

echo "########## 3. one seed, from file (SECRET)"
run cat "$W/inputs/seeds/cosigner-00.seed"
runcap "$W/out/ms-encode.txt" '^ms1' \
  "$MS" encode --phrase "$(cat "$W/inputs/seeds/cosigner-00.seed")"

echo "########## 4. the bundle: validate + plate manifest + checklist"
run head -3 "$W/inputs/backup-strings.txt"
run wc -l "$W/inputs/backup-strings.txt"
rm -rf "$W/out/plates" && mkdir -p "$W/out/plates"   # --preview requires a clean dir
run "$ME" bundle --in "$W/inputs/backup-strings.txt" \
  --preview "$W/out/plates" --png --manifest "$W/out/manifest.json"

echo "########## 5. NDEF for the descriptor card"
run "$ME" --in <(printf '%s\n' "$MD1") --hex

echo "########## 5b. and the refusal that matters: ms1 never leaves the machine"
MS1=$(grep '^ms1' "$W/out/ms-encode.txt" | tr -d ' ')
run "$ME" --in <(printf '%s\n' "$MS1") --hex

echo "########## 6. a systemwide payload carrying the public cards"
run "$ME" sysw pack --no-passphrase "$MD1" --out "$W/out/sysw-public.bin"
run "$ME" sysw show "$W/out/sysw-public.bin"
run "$ME" sysw wipe --fill zeros --out /dev/null
