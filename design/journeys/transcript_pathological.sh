#!/usr/bin/env bash
# The host half of the pathological-wallet journey, captured verbatim --
# including the three places the toolchain refuses. Failures here are results,
# not script bugs; each is reported with its real exit code.
set -u

W="$(cd "$(dirname "$0")" && pwd)"
C=/scratch/code/shibboleth
MD=$C/descriptor-mnemonic/target/release/md
MK=$C/mnemonic-key/target/release/mk
MS=$C/mnemonic-secret/target/release/ms
ME=$C/mnemonic-engrave/target/release/me
export PATH="$C/mnemonic-engrave/target/release:$PATH"

run() { echo "\$ $*"; "$@" 2>&1; echo "[exit $?]"; echo; }

T=$(cat "$W/inputs-pathological/wallet-policy.txt")
MD1S=$(tr '\n' ' ' < "$W/out/md1.txt")

echo "########## versions"
run "$MD" --version
run "$MK" --version
run "$MS" --version
run "$ME" --version
run "$C/mnemonic-engrave/target/release/me-preview" --version

echo "########## 1. the policy — 11 keys, four timelock kinds, a hashlock"
run cat "$W/inputs-pathological/wallet-policy.txt"

echo "########## 2. it does not fit one string"
run "$MD" encode --group-size 0 "$T"

echo "########## 3. so it is chunked -- WITH the origin the warning asked for"
run "$MD" encode --group-size 0 --force-chunked --path bip84 "$T"

echo "########## 4. the chunk set decodes back to the same 11-key policy"
run "$MD" inspect $MD1S

echo "########## 5. OBSTACLE 1 — mk cannot derive the stub from a CHUNKED md1"
XPUB=$(grep '^xpub' "$W/inputs-pathological/keys/key-00.xpub")
FIRST=$(head -1 "$W/out/md1.txt")
run "$MK" encode --xpub "$XPUB" --origin-fingerprint 73c5da0a \
  --origin-path "m/84'/0'/0'" --from-md1 "$FIRST" --group-size 0

echo "########## 6. the stub, derived by hand from the template id"
echo "MEASURED, not cited: on a single-string wallet where --from-md1 DOES work,"
echo "mk embedded stub 726a6663 while that wallet's ids were"
echo "  wallet-descriptor-template-id: 726a666305756435...  <-- the stub matches THIS"
echo "  wallet-policy-id:              f05e8a1c282f7740...  <-- and NOT this,"
echo "though SPEC_mk_v0_1.md 3.3 names the WalletPolicyId. So mk follows the"
echo "template-id; this wallet's is 5b48af35d4321a3a..., giving stub 5b48af35."
echo
run bash -c "$MD inspect $MD1S 2>/dev/null | sed -n 's/^wallet-descriptor-template-id: //p'"

echo "########## 7. the eleven key cards"
run "$MK" encode --xpub "$XPUB" --origin-fingerprint 73c5da0a \
  --origin-path "m/84'/0'/0'" --policy-id-stub 5b48af35 --group-size 0
run "$MK" decode $(sed -n '1,2p' "$W/out/mk-encode-raw.txt")

echo "########## 8. me bundle: validates, and prints the plate checklist"
rm -rf "$W/out/plates" && mkdir -p "$W/out/plates"
run "$ME" bundle --in "$W/inputs-pathological/backup-strings.txt" --preview "$W/out/plates" --png --manifest "$W/out/manifest.json"

echo "########## 9. the ids, and which one mk actually uses for the stub"
run bash -c "$MD inspect $MD1S 2>/dev/null | grep -E 'policy-id|template-id'"

echo "########## 10. the seed, and the refusal that still holds"
run cat "$W/inputs-pathological/seeds/master-A.seed"
run "$MS" encode --phrase "$(cat "$W/inputs-pathological/seeds/master-A.seed")"
MS1=$("$MS" encode --phrase "$(cat "$W/inputs-pathological/seeds/master-A.seed")" --no-engraving-card 2>/dev/null | tr -d ' ')
run "$ME" --in <(printf '%s\n' "$MS1") --hex
