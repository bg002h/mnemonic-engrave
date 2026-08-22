#!/usr/bin/env bash
# derive-hashvault-keys.sh — build every input the hashvault journey consumes.
#
# WHY THIS EXISTS. The pathological journey's key files once had NO PRODUCER:
# artifacts somebody generated, committed, and could not regenerate (F-210, and
# I-1 one layer deeper). An artifact nobody can regenerate is an artifact nobody
# can check. So this journey's inputs are derived from fixed, obviously-test
# entropy, and this script is the only thing that writes them.
#
# EVERYTHING HERE IS TEST MATERIAL. The six seeds come from entropy
# 0x000…001 through 0x000…006 and the three preimages are printable strings in
# this file. Never put funds behind any of it.
set -euo pipefail

W="$(cd "$(dirname "$0")" && pwd)"
C=/scratch/code/shibboleth
MS=$C/mnemonic-secret/target/release/ms
IN="$W/inputs-hashvault"

mkdir -p "$IN/seeds" "$IN/keys" "$IN/preimages"

# The six keys and the tier each one serves. One seed per key, as specified.
ROLE=(
  "tier 1 — 3-of-3 with hash A, spendable at any time"
  "tier 1 — 3-of-3 with hash A, spendable at any time"
  "tier 1 — 3-of-3 with hash A, spendable at any time"
  "tier 2 — 2-of-2 with hash B, after older(32768) relative"
  "tier 2 — 2-of-2 with hash B, after older(32768) relative"
  "tier 3 — sole key, after(1173520) absolute"
)

echo "== seeds and keys =="
for i in 0 1 2 3 4 5; do
  ent=$(printf '%063d%x' 0 $((i + 1)))
  phrase=$("$MS" decode "$("$MS" encode --hex "$ent" --no-engraving-card 2>/dev/null | tr -d ' ')" 2>/dev/null \
             | sed -n 's/^phrase: //p')
  [ -n "$phrase" ] || { echo "FATAL: no phrase for key $i" >&2; exit 1; }
  printf '%s\n' "$phrase" > "$IN/seeds/key-$i.seed"

  # The constellation's own taproot path, ruled 2026-08-18:
  # m/270028'/coin'/account'/script' with 0' = tr.
  out=$("$MS" derive --phrase "$phrase" --template bg002h-tr --account 0 2>/dev/null)
  fp=$(printf '%s\n' "$out" | sed -n 's/^master_fingerprint: *//p')
  xp=$(printf '%s\n' "$out" | sed -n 's/^account_xpub: *//p')
  path=$(printf '%s\n' "$out" | sed -n 's/^account_path: *//p')
  [ -n "$fp" ] && [ -n "$xp" ] || { echo "FATAL: derive failed for key $i" >&2; exit 1; }

  {
    printf '# @%d — %s\n' "$i" "${ROLE[$i]}"
    printf '# seed entropy 0x%s (TEST ONLY), origin [%s/%s]\n' "$ent" "$fp" "${path#m/}"
    printf '%s\n' "$xp"
  } > "$IN/keys/key-$i.xpub"
  printf '  @%d %s at %s\n' "$i" "$fp" "$path"
done

echo "== preimages =="
# Human passphrases, per the operator's ruling: engraved as TEXT+QR plates, not
# as ms1. The POLICY commits to sha256 of the UTF-8 bytes with no trailing
# newline -- `printf %s`, never `echo` -- because a stray \n changes the hash
# and silently locks the tier forever.
PRE=(
  "correct horse battery staple vault alpha"
  "seven bridges over a quiet river bravo"
  "the last plate rings twice charlie"
)
for i in 0 1 2; do
  printf '%s' "${PRE[$i]}" > "$IN/preimages/preimage-$i.txt"
  h=$(printf '%s' "${PRE[$i]}" | sha256sum | cut -d' ' -f1)
  printf '  preimage-%d sha256 %s\n' "$i" "$h"
done

echo "== policy =="
HA=$(printf '%s' "${PRE[0]}" | sha256sum | cut -d' ' -f1)
HB=$(printf '%s' "${PRE[1]}" | sha256sum | cut -d' ' -f1)
HC=$(printf '%s' "${PRE[2]}" | sha256sum | cut -d' ' -f1)
# BIP-341 NUMS H point: no key-path spend, so every spend reveals a leaf.
NUMS=50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0
O="270028'/0'/0'/0'"
T1="and_v(v:sha256($HA),multi_a(3,@0/$O/<0;1>/*,@1/$O/<0;1>/*,@2/$O/<0;1>/*))"
T2="and_v(v:older(32768),and_v(v:sha256($HB),multi_a(2,@3/$O/<0;1>/*,@4/$O/<0;1>/*)))"
T3="and_v(v:after(1173520),pk(@5/$O/<0;1>/*))"
T4="and_v(v:after(1383520),sha256($HC))"
printf '%s\n' "tr($NUMS,{$T1,{$T2,{$T3,$T4}}})" > "$IN/wallet-policy-hashvault.txt"
echo "  wrote $IN/wallet-policy-hashvault.txt"
