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
  # I-2 (exec review): a ZERO-MATCH capture used to be swallowed by `|| true`,
  # leaving an empty file. Seven of eight consumers then fail loudly, but
  # `transcript_pathological.sh`'s template-id step printed BLANK at [exit 0] --
  # and that step is what justifies the hardcoded --policy-id-stub below it.
  # An empty capture is a capture FAILURE, so say so and delete the file: a
  # later read then dies with "no such file" instead of reading nothing.
  if ! grep -E "$keep" "$sout" > "$out"; then
    rm -f "$out"
    printf 'runcap: CAPTURE FAILED -- no stdout line matched /%s/\n' "$keep"
    printf 'runcap: %s not written; any later step reading it will fail loudly\n' "$out"
    [ "$rc" -eq 0 ] && rc=1
  fi
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

# ALL twelve cosigner cards, each with its own origin read from its own key
# file — was: cosigner-00 only. The stub comes from --from-md1, so it is
# derived from the policy above rather than typed in.
: > "$W/out/mk-encode-raw.txt"
for f in "$W"/inputs/keys/cosigner-*.xpub; do
  KX=$(grep '^xpub' "$f")
  KFP=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$f" | head -1)
  KPATH=$(sed -n 's/.*origin \[[0-9a-f]*\/\([^]]*\)\].*/\1/p' "$f" | head -1)
  if [ -z "$KX" ] || [ -z "$KFP" ] || [ -z "$KPATH" ]; then
    echo "FATAL: $f is missing an xpub or an origin header" >&2
    exit 1
  fi
  # I-2: the encode's exit status USED TO BE DISCARDED by the pipe, so a typo
  # in a key header silently dropped that cosigner's card from the ENGRAVED
  # bundle at exit 0. A short bundle is a worse failure than the stale one
  # this change set out to remove.
  if ! CARD=$("$MK" encode --xpub "$KX" --origin-fingerprint "$KFP" \
        --origin-path "m/$KPATH" --from-md1 "$MD1" --group-size 0 2>&1); then
    echo "FATAL: mk encode failed for $f:" >&2
    printf '%s\n' "$CARD" >&2
    exit 1
  fi
  NL=$(printf '%s\n' "$CARD" | grep -c '^mk1')
  if [ "$NL" -ne 2 ]; then
    echo "FATAL: $f produced $NL mk1 line(s), expected 2" >&2
    exit 1
  fi
  printf '%s\n' "$CARD" | grep '^mk1' >> "$W/out/mk-encode-raw.txt"
done

NKEYS=$(ls "$W"/inputs/keys/cosigner-*.xpub | wc -l)
WANT=$(( NKEYS * 2 ))
GOT=$(grep -c '^mk1' "$W/out/mk-encode-raw.txt")
if [ "$GOT" -ne "$WANT" ]; then
  echo "FATAL: expected $WANT mk1 lines for $NKEYS keys, got $GOT" >&2
  exit 1
fi
run wc -l "$W/out/mk-encode-raw.txt"
run "$MK" inspect $(sed -n '1,2p' "$W/out/mk-encode-raw.txt")

echo "########## 2b. the bundle this journey ENGRAVES is BUILT here, not shipped"
echo "F-210/I-1: inputs/backup-strings.txt used to be a tracked fixture that"
echo "nothing produced. It drifted against mk, so the journey printed one card"
echo "and engraved a different one for the same key. The engraved file is now"
echo "assembled from this run's own md1 and key cards."
echo
cat "$W/out/md-encode-raw.txt" "$W/out/mk-encode-raw.txt" > "$W/out/backup-strings.txt"
run wc -l "$W/out/backup-strings.txt"

echo "########## 3. one seed, from file (SECRET)"
run cat "$W/inputs/seeds/cosigner-00.seed"
runcap "$W/out/ms-encode.txt" '^ms1' \
  "$MS" encode --phrase "$(cat "$W/inputs/seeds/cosigner-00.seed")"

echo "########## 4. the bundle: validate + plate manifest + checklist"
run head -3 "$W/out/backup-strings.txt"
run wc -l "$W/out/backup-strings.txt"
rm -rf "$W/out/plates" && mkdir -p "$W/out/plates"   # --preview requires a clean dir
run "$ME" bundle --in "$W/out/backup-strings.txt" \
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
