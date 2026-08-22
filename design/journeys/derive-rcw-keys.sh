#!/usr/bin/env bash
# derive-rcw-keys.sh — every input the REASONABLY COMPLEX WALLET journey consumes.
#
# WHY THIS EXISTS. The pathological journey's key files once had NO PRODUCER:
# artifacts somebody generated, committed, and could not regenerate (F-210). An
# artifact nobody can regenerate is an artifact nobody can check. So these
# inputs come from fixed, obviously-test entropy and this script is the only
# thing that writes them.
#
# EVERYTHING HERE IS TEST MATERIAL. Seven seeds from entropy 0x000…001 through
# 0x000…007; three passphrases are printable strings in this file. Never put
# funds behind any of it.
#
# TIER 4 IS KEYED (operator ruling 2026-08-22: "keyless path is not reasonable").
# It was `after(1383520) AND sha256(C)` with no key, which made that tier bearer
# access to anyone holding the preimage — and made stock rust-miniscript refuse
# the tr form outright ("All spend paths must require a signature"), the same
# NeedsSignature() check that closes Bitcoin Core and Nunchuk. Tier 4 is now
# `after(1383520) AND sha256(C) AND pk(@6)`, so @6 is the seventh seed below.
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

FIXCHK="$W/../fixtures/reasonably-complex-wallet"
[ -f "$FIXCHK/tr.policy" ] || { echo "FATAL: no fixture policy at $FIXCHK" >&2; exit 1; }

# The six keys and the tier each one serves.
ROLE=(
  "tier 1 — 3-of-3 with hash A, spendable at any time"
  "tier 1 — 3-of-3 with hash A, spendable at any time"
  "tier 1 — 3-of-3 with hash A, spendable at any time"
  "tier 2 — 2-of-2 with hash B, after older(32768) relative"
  "tier 2 — 2-of-2 with hash B, after older(32768) relative"
  "tier 3 — sole key, after(1173520) absolute"
  "tier 4 — sole key, after(1383520) absolute + hash C"
)

# wrapper -> (ms template, account). Kept as parallel arrays rather than an
# associative array so the ORDER is fixed and the loop below is auditable.
WRAP=(tr wsh)
TMPL=(bg002h-tr bg002h-wsh)
ACCT=(8 9)

echo "== seeds =="
for i in 0 1 2 3 4 5 6; do
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
  for i in 0 1 2 3 4 5 6; do
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
# Human passphrases, hashed TWICE. This is not decoration -- it is what makes
# the hashlock tiers spendable at all.
#
# Miniscript's sha256(H) fragment compiles to
#   OP_SIZE <32> OP_EQUALVERIFY OP_SHA256 <H> OP_EQUAL
# so the WITNESS PREIMAGE MUST BE EXACTLY 32 BYTES. That is in the script, so it
# is consensus-enforced, not a library rule. These passphrases are 34-40 bytes,
# so committing to sha256(phrase) directly -- as this file did until
# 2026-08-22 -- produced three tiers that could never be spent by anyone.
#
# The fix keeps the passphrase human and the preimage 32 bytes:
#   witness preimage = sha256(phrase)          <- 32 bytes, the .hex file
#   policy literal   = sha256(sha256(phrase))  <- what the descriptor commits to
# A recoverer remembers the phrase and hashes it once to get the preimage.
#
# The phrase bytes carry NO trailing newline -- `printf %s`, never `echo` --
# because a stray \n changes both digests and silently locks that tier forever.
PRE=(
  "correct horse battery staple vault alpha"
  "seven bridges over a quiet river bravo"
  "the last plate rings twice charlie"
)
for i in 0 1 2; do
  printf '%s' "${PRE[$i]}" > "$IN/preimages/preimage-$i.txt"
  inner=$(printf '%s' "${PRE[$i]}" | sha256sum | cut -d' ' -f1)
  outer=$(printf '%s' "$inner" | xxd -r -p | sha256sum | cut -d' ' -f1)

  # The preimage that goes in the witness, as raw hex. Assert 32 bytes: if this
  # ever stops holding, the tier is unspendable and the run must stop.
  [ ${#inner} -eq 64 ] || { echo "FATAL: preimage-$i is ${#inner} hex chars, not 64 (32 bytes)" >&2; exit 1; }
  printf '%s' "$inner" > "$IN/preimages/preimage-$i.hex"

  # And the literal must actually be the one the fixture policy commits to.
  grep -q "sha256($outer)" "$FIXCHK/tr.policy" || {
    echo "FATAL: preimage-$i double-hash $outer is not a literal in tr.policy" >&2
    echo "       the passphrase and the policy have drifted apart." >&2; exit 1; }

  printf '  preimage-%d  phrase %d B -> preimage %s -> literal %s\n' \
         "$i" "${#PRE[$i]}" "${inner:0:16}..." "${outer:0:16}..."
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
  [ "$n" -eq 7 ] || {
    echo "FATAL: policy-$w.txt has $n occurrences of $want, expected 7" >&2
    echo "       the path substitution did not take -- check the fixture's paths." >&2
    exit 1; }
  # And NO stale account-0 path may survive.
  if grep -qE "270028'/0'/0'/" "$IN/policy-$w.txt"; then
    echo "FATAL: policy-$w.txt still contains an account-0 path" >&2; exit 1
  fi
  printf '  policy-%s.txt  %d chars, %d slots at %s\n' "$w" "$(wc -c < "$IN/policy-$w.txt")" "$n" "$want"
done

echo
echo "Wrote $IN — 7 seeds, 7 fingerprints, 14 xpubs (7 tr + 7 wsh), 3 preimages, 2 policies."
