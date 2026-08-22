#!/usr/bin/env bash
# derive-rcw-keys.sh — every input the REASONABLY COMPLEX WALLET journey consumes.
#
# WHY THIS EXISTS. The pathological journey's key files once had NO PRODUCER:
# artifacts somebody generated, committed, and could not regenerate (F-210). An
# artifact nobody can regenerate is an artifact nobody can check. So these
# inputs come from fixed, obviously-test entropy and this script is the only
# thing that writes them.
#
# EVERYTHING HERE IS TEST MATERIAL. Six seeds from entropy 0x000…001 through
# 0x000…006; three preimages are printable strings in this file. Never put funds
# behind any of it.
#
# TWO KEY SETS FROM THE SAME SIX SEEDS. The operator ruled (2026-08-22) that the
# tr form derives at ACCOUNT 8 and the wsh form at ACCOUNT 9:
#
#   tr   ms derive --template bg002h-tr  --account 8  ->  m/270028'/0'/8'/0'
#   wsh  ms derive --template bg002h-wsh --account 9  ->  m/270028'/0'/9'/1'
#
# The level-4 script value stays as ruled — 0' = tr, 1' = wsh
# (ms-cli/src/cmd/derive.rs:149). Account is what 8 and 9 select, which is the
# only reading that needs no code change and keeps the two wallets' keys
# disjoint. Same masters, so the six FINGERPRINTS are shared between the two
# wallets; only the account xpubs differ.
set -euo pipefail

W="$(cd "$(dirname "$0")" && pwd)"
C=/scratch/code/shibboleth
MS=$C/mnemonic-secret/target/release/ms
IN="$W/inputs-rcw"

[ -x "$MS" ] || { echo "FATAL: no ms at $MS — build mnemonic-secret first" >&2; exit 1; }

mkdir -p "$IN/seeds" "$IN/keys-tr" "$IN/keys-wsh" "$IN/preimages"

# The six keys and the tier each one serves.
ROLE=(
  "tier 1 — 3-of-3 with hash A, spendable at any time"
  "tier 1 — 3-of-3 with hash A, spendable at any time"
  "tier 1 — 3-of-3 with hash A, spendable at any time"
  "tier 2 — 2-of-2 with hash B, after older(32768) relative"
  "tier 2 — 2-of-2 with hash B, after older(32768) relative"
  "tier 3 — sole key, after(1173520) absolute"
)

# wrapper -> (ms template, account). Kept as parallel arrays rather than an
# associative array so the ORDER is fixed and the loop below is auditable.
WRAP=(tr wsh)
TMPL=(bg002h-tr bg002h-wsh)
ACCT=(8 9)

echo "== seeds =="
for i in 0 1 2 3 4 5; do
  ent=$(printf '%063d%x' 0 $((i + 1)))
  phrase=$("$MS" decode "$("$MS" encode --hex "$ent" --no-engraving-card 2>/dev/null | tr -d ' ')" 2>/dev/null \
             | sed -n 's/^phrase: //p')
  [ -n "$phrase" ] || { echo "FATAL: no phrase for key $i" >&2; exit 1; }
  printf '%s\n' "$phrase" > "$IN/seeds/key-$i.seed"
  printf '  key-%d  entropy 0x%s\n' "$i" "$ent"
done

# THE SEED FINGERPRINT, computed ONCE PER SEED and asserted identical across the
# two wrappers. It is a property of the MASTER, not of a derivation path, so if
# these ever diverged between the tr and wsh runs something is deeply wrong —
# and the assertion is cheaper than the confusion.
echo
echo "== keys =="
for w in 0 1; do
  wrap=${WRAP[$w]}
  echo "  -- $wrap: ${TMPL[$w]}, account ${ACCT[$w]}"
  for i in 0 1 2 3 4 5; do
    phrase=$(cat "$IN/seeds/key-$i.seed")
    out=$("$MS" derive --phrase - --template "${TMPL[$w]}" --account "${ACCT[$w]}" <<<"$phrase" 2>/dev/null)
    fp=$(printf '%s\n' "$out" | sed -n 's/^master_fingerprint: *//p')
    xp=$(printf '%s\n' "$out" | sed -n 's/^account_xpub: *//p')
    path=$(printf '%s\n' "$out" | sed -n 's/^account_path: *//p')
    [ -n "$fp" ] && [ -n "$xp" ] && [ -n "$path" ] || {
      echo "FATAL: derive failed for $wrap key $i" >&2; exit 1; }

    # Cross-wrapper fingerprint agreement, asserted rather than assumed.
    fpfile="$IN/seeds/key-$i.fingerprint"
    if [ -f "$fpfile" ]; then
      prev=$(cat "$fpfile")
      [ "$prev" = "$fp" ] || {
        echo "FATAL: key $i fingerprint differs between wrappers: $prev vs $fp" >&2
        echo "       a master fingerprint cannot depend on the derivation path." >&2
        exit 1; }
    else
      printf '%s\n' "$fp" > "$fpfile"
    fi

    {
      printf '# @%d — %s\n' "$i" "${ROLE[$i]}"
      printf '# %s wrapper; seed entropy 0x%063d%x (TEST ONLY)\n' "$wrap" 0 $((i + 1))
      printf '# origin [%s/%s]\n' "$fp" "${path#m/}"
      printf '%s\n' "$xp"
    } > "$IN/keys-$wrap/key-$i.xpub"
    printf '    @%d %s at %s\n' "$i" "$fp" "$path"
  done
done

echo
echo "== preimages =="
# Human passphrases. The POLICY commits to sha256 of the UTF-8 bytes with NO
# trailing newline -- `printf %s`, never `echo` -- because a stray \n changes
# the hash and silently locks that tier forever.
#
# These are the SAME three the hashlock-vault journey uses, deliberately: the
# reasonably-complex wallet's policy carries those exact digests as literals, so
# any other preimage would produce a different wallet.
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

echo
echo "== policies =="
# THE POLICIES HAVE A PRODUCER TOO. Both wrappers are derived from the named
# fixture by substituting the derivation path, rather than retyped -- a
# hand-copied 500-character policy is a transcription defect waiting to happen,
# and the two forms must stay in lockstep on everything except the path and the
# multi/multi_a difference.
FIX="$W/../fixtures/reasonably-complex-wallet"
[ -f "$FIX/tr.policy" ] && [ -f "$FIX/wsh.policy" ] || {
  echo "FATAL: no fixture policies at $FIX" >&2; exit 1; }

sed "s#270028'/0'/0'/0'#270028'/0'/8'/0'#g" "$FIX/tr.policy"  > "$IN/policy-tr.txt"
sed "s#270028'/0'/1'/1'#270028'/0'/9'/1'#g; s#270028'/0'/0'/1'#270028'/0'/9'/1'#g" \
    "$FIX/wsh.policy" > "$IN/policy-wsh.txt"

for w in tr wsh; do
  want=$([ "$w" = tr ] && echo "270028'/0'/8'/0'" || echo "270028'/0'/9'/1'")
  n=$(grep -o "$want" "$IN/policy-$w.txt" | wc -l)
  [ "$n" -eq 6 ] || {
    echo "FATAL: policy-$w.txt has $n occurrences of $want, expected 6" >&2
    echo "       the path substitution did not take -- check the fixture's paths." >&2
    exit 1; }
  # And NO stale account-0 path may survive.
  if grep -qE "270028'/0'/0'/" "$IN/policy-$w.txt"; then
    echo "FATAL: policy-$w.txt still contains an account-0 path" >&2; exit 1
  fi
  printf '  policy-%s.txt  %d chars, 6 slots at %s\n' "$w" "$(wc -c < "$IN/policy-$w.txt")" "$want"
done

echo
echo "Wrote $IN — 6 seeds, 6 fingerprints, 12 xpubs (6 tr + 6 wsh), 3 preimages, 2 policies."
