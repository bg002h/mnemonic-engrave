#!/usr/bin/env bash
# The host half of the Wallet Policy journey.
#
# The premise: someone ELSE built this wallet. Their coordinator produced a
# BIP-388 template and four cosigner xpubs; this script turns that into the card
# set they hand over, and derives — off-device, in Rust — the wallet id and the
# addresses that card set MUST prove to. The device half
# (capture_walletpolicy.py) then reads what SeedHammer II shows for the same
# cards and refuses to finish if the two disagree.
#
# Every artifact the device half consumes is WRITTEN here, by runcap, from a
# real command's real stdout. Nothing downstream reads a file nothing upstream
# produced — the failure mode F-210 was filed for.
set -u

W="$(cd "$(dirname "$0")" && pwd)"
C=/scratch/code/shibboleth
MD=$C/descriptor-mnemonic/target/release/md
IN="$W/inputs-walletpolicy"
OUT="$W/out"
mkdir -p "$OUT"

run() { echo "\$ $*"; "$@" 2>&1; echo "[exit $?]"; echo; }

runcap() {
  local out="$1" keep="$2"; shift 2
  echo "\$ $*"
  local sout serr rc
  sout="$(mktemp)"; serr="$(mktemp)"
  "$@" >"$sout" 2>"$serr"; rc=$?
  cat "$sout"; cat "$serr"
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

TEMPLATE="$(cat "$IN/policy.template")"
# Retained for the prose below; NOT passed to any command any more.
PATHARG="$(cat "$IN/origin.path")"
K0="$(cat "$IN/key0.xpub")"; K1="$(cat "$IN/key1.xpub")"
K2="$(cat "$IN/key2.xpub")"; K3="$(cat "$IN/key3.xpub")"
FP="$(cat "$IN/master.fingerprint")"

# Every command below carries the SAME four flags. Kept in one array rather than
# retyped six times: a wallet identity that changes when one invocation drops a
# --fingerprint is exactly the drift this journey is about, and it would be
# invisible in a transcript where each command spells its own arguments.
KEYS=(--key "@0=$K0" --key "@1=$K1" --key "@2=$K2" --key "@3=$K3"
      --fingerprint "@0=$FP" --fingerprint "@1=$FP"
      --fingerprint "@2=$FP" --fingerprint "@3=$FP")

# NO --path, AND THAT IS THE WHOLE POINT (F-217). The first version of this
# journey passed --path "48'/0'/0'/2'", which FLATTENS per-key origins to one
# shared path -- so the card declared all four keys at account 0 when they live
# at accounts 0..3. A (fingerprint, path) pair names exactly one key under
# BIP-32, so that card described a wallet that cannot exist, and `md encode`
# now refuses it outright.
#
# The origins belong in the TEMPLATE (`@0/48'/0'/0'/2'/<0;1>/*`), which is
# where md has always taken per-key origins -- see policy.template.

echo "=== 0. What arrived from the other operator ==="
echo
run cat "$IN/policy.template"
echo "Four cosigner xpubs, at accounts 0..3 under $PATHARG's prefix:"
run head -c 40 "$IN/key0.xpub"
echo

echo "=== 1. The card set ==="
echo
# --force-chunked is now REDUNDANT here: F-136 is fixed and `md encode` chunks
# automatically on overflow (descriptor-mnemonic 9af0975). Kept because the
# output is byte-identical either way and this transcript is a recorded walk --
# removing it would change the printed command without changing a single card. --path, because a tr() wrapper has no
# canonical default origin and without one the card only PARTIAL-decodes.
runcap "$OUT/walletpolicy.md1.txt" '^md1' \
  "$MD" encode "$TEMPLATE" "${KEYS[@]}" --force-chunked

echo "The operator carries $(wc -l < "$OUT/walletpolicy.md1.txt") md1 chunks."
echo

# EACH CHUNK IS ONE ARGUMENT, and every chunk contains SPACES ("md1fu xxeqq
# pqggq ..."). `$(cat file)` word-splits on those spaces, so the tools received
# five-character fragments and refused with "character '1' not in codex32
# alphabet" -- a message that reads like a corrupt card rather than like a
# quoting bug. mapfile splits on newlines only.
mapfile -t CHUNKS < "$OUT/walletpolicy.md1.txt"

echo "=== 2. What the device must prove to ==="
echo
# The wallet id is KEY-DEPENDENT here (a full policy, not a template), so it is
# the one that answers "is this MY wallet, with these cosigners".
# There is no `md id`; the ids come from `md inspect` of the CARDS, which is the
# stronger place to read them anyway — it proves the identity the engraved
# strings carry, not the identity of the arguments that produced them.
runcap "$OUT/walletpolicy.id.txt" '^wallet-(policy|descriptor-template)-id:' \
  "$MD" inspect "${CHUNKS[@]}"

# Receive AND change. Change is where a policy mismatch silently loses funds,
# which is why the device's consent screen shows both and why both are pinned
# here (plan D6).
runcap "$OUT/walletpolicy.receive.txt" '^bc1' \
  "$MD" address --template "$TEMPLATE" "${KEYS[@]}" --count 2

runcap "$OUT/walletpolicy.change.txt" '^bc1' \
  "$MD" address --template "$TEMPLATE" "${KEYS[@]}" --change --count 2

echo "=== 3. Read the card set back, as the device will ==="
echo
# The round trip that makes step 1 evidence rather than output: decode the cards
# and require the template that comes back.
run "$MD" decode "${CHUNKS[@]}"

echo "=== Artifacts for the device half ==="
run ls -la "$OUT/walletpolicy.md1.txt" "$OUT/walletpolicy.id.txt" \
   "$OUT/walletpolicy.receive.txt" "$OUT/walletpolicy.change.txt"
