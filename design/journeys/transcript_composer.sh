#!/usr/bin/env bash
# The host half of the WALLET POLICY COMPOSER journey (composer S4, spec §12
# items 2 and 3).
#
# The premise: nobody hands this operator a policy. They build one ON the
# device, from a systemwide payload they packed themselves, and the device shows
# them ids and addresses for a wallet that has never existed anywhere else. This
# script is the OTHER implementation of the same wallet -- Rust, on a host, from
# the same two BIP-39 vectors -- so the device half (capture_composer.py) has
# something to disagree with.
#
# TWO ARMS, because the composer has two:
#
#   KEYED    wsh, 2-of-2 (master A at accounts 0' and 1') OR one key of master B
#            behind a 12960-block wait and a sha256 hashlock. Ids, four
#            addresses, the fingerprinted template, three key cards.
#   KEYLESS  tr, one 2-of-3 path, no keys at all. ONE 56-character md1 string,
#            which is also the plate the operator cuts on the real machine.
#
# THE KEYLESS STRING IS THE CHUNKED FORM, AND THE DISTINCTION IS THE WHOLE
# POINT. The device is chunk-form-always (md.Composed.Chunks is split(...)),
# and a template this short encodes UNCHUNKED on the host by default. `md
# verify` and `md inspect` accept BOTH forms identically -- same template, same
# wallet-descriptor-template-id, same md1-encoding-id -- so NO verify step can
# tell them apart and only a byte comparison can. Every md1 oracle below is
# therefore minted with --force-chunked --group-size 0.
#
# Every artifact the device half consumes is WRITTEN here, by runcap, from a
# real command's real stdout. Nothing downstream reads a file nothing upstream
# produced -- the failure mode F-210 was filed for.
#
# GATES, not decoration. This script EXITS NON-ZERO if any check below fails:
# the fork's Go generator disagreeing with the host's records file, the packed
# blob differing from the one embedded in cmd/emu, a chunk count moving, or the
# 56-character keyless string coming back as anything else. A transcript that
# records a wrong value and exits 0 is worse than no transcript.
set -u

W="$(cd "$(dirname "$0")" && pwd)"
C=/scratch/code/shibboleth
MD="${MD:-$C/descriptor-mnemonic/target/release/md}"
MK="${MK:-$HOME/.cargo/bin/mk}"
MS="${MS:-$HOME/.cargo/bin/ms}"
ME="${ME:-$C/mnemonic-engrave/target/debug/me}"
# The fork checkout holding cmd/buildpayloadcomposer and cmd/emu's embedded
# blob. Overridable because this script is run from a WORKTREE at least as often
# as from the main checkout, and the default resolves to the main one -- which
# is the wrong tree while a fork change is still on a branch.
FORK="${FORK:-$C/seedhammer}"
GO="${GO:-$C/.toolchain/go/bin/go}"
OUT="$W/out/composer"
mkdir -p "$OUT" "$OUT/cards"

FAILURES=0

run() { echo "\$ $*"; "$@" 2>&1; echo "[exit $?]"; echo; }

# Capture the STDOUT lines matching $keep into $out. A capture that matches
# nothing DELETES the file rather than leaving a stale one, so a later step
# reading it fails loudly instead of using yesterday's value.
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
    FAILURES=$((FAILURES + 1))
  fi
  rm -f "$sout" "$serr"
  echo "[exit $rc]"
  echo
}

# The same, over stdout AND stderr. MEASURED, not defensive: `me` 0.8.0 prints
# `sysw show`'s sealed/pub_len/identity/record lines on stdout and the `digest:`
# line -- the sixteen hex digits an operator compares across the air gap -- on
# STDERR. A stdout-only capture of it silently writes nothing.
runcapboth() {
  local out="$1" keep="$2"; shift 2
  echo "\$ $*"
  local both rc
  both="$(mktemp)"
  "$@" >"$both" 2>&1; rc=$?
  cat "$both"
  if ! grep -E "$keep" "$both" > "$out"; then
    rm -f "$out"
    printf 'runcapboth: CAPTURE FAILED -- nothing matched /%s/ on either stream\n' "$keep"
    [ "$rc" -eq 0 ] && rc=1
    FAILURES=$((FAILURES + 1))
  fi
  rm -f "$both"
  echo "[exit $rc]"
  echo
}

# A named check. Prints PASS or FAIL with both sides, and FAIL is fatal to the
# script's exit code -- never to its progress, because a run that stops at the
# first disagreement records less than one that reports all of them.
gate() {
  local what="$1" got="$2" want="$3"
  if [ "$got" = "$want" ]; then
    printf 'GATE PASS  %s\n           = %s\n' "$what" "$want"
  else
    printf 'GATE FAIL  %s\n           got  %s\n           want %s\n' "$what" "$got" "$want"
    FAILURES=$((FAILURES + 1))
  fi
}

echo "########## versions -- BY PATH, because a bare name has been an alias here"
run "$MD" --version
run "$MK" --version
run "$MS" --version
run "$ME" --version
run "$GO" version
echo "fork checkout: $FORK"
run git -C "$FORK" rev-parse --short HEAD
echo

echo "########## 1. the two masters, and the account xpubs the device derives"
echo
echo "Master A = BIP-39's \"abandon ... about\" vector; master B = its \"legal"
echo "winner ... yellow\". PUBLISHED TEST VECTORS, public by construction."
echo "Never put funds behind them."
echo
MASTER_A="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
MASTER_B="legal winner thank year wave sausage worth useful legal winner thank yellow"

# --phrase - rather than --phrase "$M": argv is world-readable through
# /proc/$PID/cmdline. These are published vectors, so nothing is at stake here;
# the spelling is the habit, kept where it costs nothing. The phrase is still
# PRINTED, because a transcript nobody can retype is a picture of a command
# rather than a record of one -- and this one is in BIP-39's own appendix.
msderive() {
  local phrase="$1"; shift
  echo "\$ printf '%s' '$phrase' | $MS derive --phrase -" "$@"
  printf '%s' "$phrase" | "$MS" derive --phrase - "$@" 2>&1
  echo "[exit $?]"
  echo
}
msderive "$MASTER_A"
msderive "$MASTER_A" --template bip48-p2wsh --account 0
msderive "$MASTER_A" --template bip48-p2wsh --account 1
msderive "$MASTER_B"
msderive "$MASTER_B" --template bip48-p2wsh --account 0

FP_A=73c5da0a
FP_B=b8688df1
K0=$(printf '%s' "$MASTER_A" | "$MS" derive --phrase - --template bip48-p2wsh --account 0 2>/dev/null | awk '/^account_xpub:/{print $2}')
K1=$(printf '%s' "$MASTER_A" | "$MS" derive --phrase - --template bip48-p2wsh --account 1 2>/dev/null | awk '/^account_xpub:/{print $2}')
K2=$(printf '%s' "$MASTER_B" | "$MS" derive --phrase - --template bip48-p2wsh --account 0 2>/dev/null | awk '/^account_xpub:/{print $2}')
gate "A@0 xpub" "$K0" "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"
gate "A@1 xpub" "$K1" "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"
gate "B@0 xpub" "$K2" "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"
echo

echo "########## 2. the composer payload's records, minted on the HOST"
echo
echo "Five records. \`key:\` bodies are the hex of the BRACKETED text; \`hash:\`"
echo "is the digest itself; \`now:\` is the hex of \"<seconds>,<height>\". The"
echo "\`now:\` is supplied EXPLICITLY -- an operator-supplied one wins, so"
echo "nothing is auto-appended and the blob's digest does not move with the"
echo "wall clock. The seed goes in as its words, sniffed as ClassMnemonic."
echo
{
  printf 'key:%s\n' "$(printf '%s' "[$FP_A/48'/0'/0'/2']$K0" | xxd -p -c 512)"
  printf 'key:%s\n' "$(printf '%s' "[$FP_A/48'/0'/1'/2']$K1" | xxd -p -c 512)"
  printf 'hash:abababababababababababababababababababababababababababababababab\n'
  printf 'now:%s\n' "$(printf '%s' '1788220800,905000' | xxd -p -c 512)"
  printf '%s\n' "$MASTER_B"
} > "$OUT/records.txt"
run cat "$OUT/records.txt"

echo "########## 2a. GATE -- the fork's Go generator emits the same five bytes"
echo
echo "cmd/buildpayloadcomposer derives through the DEVICE's own path"
echo "(bip39.MnemonicSeed -> hdkeychain.NewMaster -> bip32.Derive -> Neuter)."
echo "This diff is two implementations, in two languages, meeting on a byte"
echo "sequence. It is the check the emulator's whole payload leg rests on: if"
echo "the generator drifted, the device would show a digest this transcript"
echo "never computed."
echo
echo "\$ diff <($GO run ./cmd/buildpayloadcomposer) $OUT/records.txt"
( cd "$FORK" && CGO_ENABLED=0 GOFLAGS=-mod=readonly "$GO" run ./cmd/buildpayloadcomposer 2>/dev/null ) > "$OUT/records-from-fork.txt"
if diff "$OUT/records-from-fork.txt" "$OUT/records.txt"; then
  echo "[exit 0]"
  gate "records.txt == cmd/buildpayloadcomposer stdout" "identical" "identical"
else
  echo "[exit $?]"
  gate "records.txt == cmd/buildpayloadcomposer stdout" "DIFFERENT" "identical"
fi
echo

echo "########## 3. the payload the device loads"
echo
runcapboth "$OUT/payload.digest.txt" '^digest:' \
  "$ME" sysw pack --no-passphrase --in "$OUT/records.txt" --out "$OUT/payload.bin"
run "$ME" sysw show "$OUT/payload.bin"

echo "########## 3a. GATE -- byte-identical to the blob embedded in cmd/emu"
echo
echo "cmd/emu/sysw_composer_payload.bin was packed from the same generator, so"
echo "these must be the same 782 bytes. \`me sysw pack\` is DETERMINISTIC for the"
echo "unsealed variant -- salt and IV are only consumed on the sealed path --"
echo "which is what makes this a byte comparison rather than a structural one."
echo
echo "\$ cmp $OUT/payload.bin $FORK/cmd/emu/sysw_composer_payload.bin"
if cmp "$OUT/payload.bin" "$FORK/cmd/emu/sysw_composer_payload.bin"; then
  echo "[exit 0]"
  gate "payload.bin == cmd/emu/sysw_composer_payload.bin" "identical" "identical"
else
  echo "[exit $?]"
  gate "payload.bin == cmd/emu/sysw_composer_payload.bin" "DIFFERENT" "identical"
fi
DIGEST="$(awk '{ $1=""; sub(/^ +/,""); print }' "$OUT/payload.digest.txt")"
gate "payload digest" "$DIGEST" "dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b"
echo

echo "########## 4. THE KEYED ARM -- what the composer builds on screen"
echo
echo "wsh; Path 1 = 2-of-2 (A@0, A@1); Path 2 = one key of master B behind a"
echo "12960-block relative wait AND the payload's sha256 hashlock."
echo
runcap "$OUT/compose.json" '.' \
  "$MD" compose --wrapper wsh --path 2of2 \
    --path '1of1,older=12960,sha256=abababababababababababababababababababababababababababababababab' \
    --json

echo "md's own lowest-free rule puts the UNSEATED @2 at 48'/0'/2'/2' -- and that"
echo "is exactly what the device's FIRST stub screen shows, before seating. But"
echo "the DEVICE seats @2 from seed B at B's OWN account 0' (§4f: each master at"
echo "its own hardened account by ordinal among the slots that master fills;"
echo "master B fills one slot, so ordinal 0). So the host oracle is minted with"
echo "THAT origin, not with md's placeholder:"
echo
KEYED_TEMPLATE="wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),and_v(v:pkh(@2/48'/0'/0'/2'/<0;1>/*),and_v(v:sha256(abababababababababababababababababababababababababababababababab),older(12960)))))"
printf '%s\n' "$KEYED_TEMPLATE" > "$OUT/keyed.template"
run cat "$OUT/keyed.template"

KEYS=(--key "@0=$K0" --key "@1=$K1" --key "@2=$K2"
      --fingerprint "@0=$FP_A" --fingerprint "@1=$FP_A" --fingerprint "@2=$FP_B")

echo "########## 4a. the full policy, as the device engraves it (form A)"
echo
# --force-chunked --group-size 0: the device is chunk-form-always, and the
# grouped five-character display form lives on stderr where a capture cannot
# reach it. --group-size 0 also keeps the stderr echo unbroken, so the two
# streams show the same strings.
runcap "$OUT/keyed.md1.txt" '^md1' \
  "$MD" encode "$KEYED_TEMPLATE" "${KEYS[@]}" --force-chunked --group-size 0
gate "keyed md1 chunk count" "$(wc -l < "$OUT/keyed.md1.txt")" "7"
gate "keyed md1 chunk 1" "$(head -1 "$OUT/keyed.md1.txt")" \
  "md1flv5xrq9qjtvyyy5jmpprjjtvyyy4qqxpxqcyy2v6tjv6a4w46h2at4w46h2at4w46h2at4gj8fq03ncnsuxv"
echo

# EACH CHUNK IS ONE ARGUMENT. mapfile splits on newlines only; `$(cat file)`
# word-splits, and with --group-size 5 that hands the tools five-character
# fragments and a message that reads like a corrupt card rather than a quoting
# bug.
mapfile -t CHUNKS < "$OUT/keyed.md1.txt"

echo "########## 4b. what the device must prove to"
echo
echo "The ids come from \`md inspect\` OF THE CHUNKS, not of the arguments that"
echo "produced them: that proves the identity the engraved strings carry."
echo
runcap "$OUT/keyed.id.txt" '^(wallet-policy-id|wallet-descriptor-template-id|md1-encoding-id|wallet-policy-id-fingerprint):' \
  "$MD" inspect "${CHUNKS[@]}"
gate "Template-ID" "$(awk -F': ' '/^wallet-descriptor-template-id:/{print $2}' "$OUT/keyed.id.txt")" \
  "531ab9e1777f018ae53694387dd0d128"
gate "Policy-ID" "$(awk -F': ' '/^wallet-policy-id:/{print $2}' "$OUT/keyed.id.txt")" \
  "4dd749a8372af515a61d7104faf944ef"
gate "md1-encoding-id" "$(awk -F': ' '/^md1-encoding-id:/{print $2}' "$OUT/keyed.id.txt")" \
  "fb28698ee8bdbc18c6ee36598f2124fe"
echo
echo "The device's mk1 stubs are the FIRST FOUR BYTES of those two ids:"
echo "  mk1 stub (template): 531ab9e1"
echo "  mk1 stub (policy):   4dd749a8"
echo

# Receive AND change. Change is where a policy mismatch silently loses funds,
# which is why the consent screen shows both and why both are pinned here.
runcap "$OUT/keyed.receive.txt" '^bc1' \
  "$MD" address --template "$KEYED_TEMPLATE" "${KEYS[@]}" --count 2
runcap "$OUT/keyed.change.txt" '^bc1' \
  "$MD" address --template "$KEYED_TEMPLATE" "${KEYS[@]}" --change --count 2
gate "Receive 0" "$(sed -n 1p "$OUT/keyed.receive.txt")" "bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4l"
gate "Receive 1" "$(sed -n 2p "$OUT/keyed.receive.txt")" "bc1qkd729k2r3kvrewzgdtpj0quhrrv9u4jgndt2zsmy6ypnr7rslzwsfhmu9a"
gate "Change 0"  "$(sed -n 1p "$OUT/keyed.change.txt")"  "bc1q9ms8tdk54dzaelef0rrg82fpm3s9nfgyr30aed96rnyuj02hhgrqy3dyru"
gate "Change 1"  "$(sed -n 2p "$OUT/keyed.change.txt")"  "bc1q3cs923r9rdcv5s8zmwkd5strrh7svzzpg2yrl4hcue3f3fv4lyfsdp3tz9"
echo

echo "########## 4c. read the card set back, as the device will"
echo
run "$MD" decode "${CHUNKS[@]}"

echo "########## 5. FORM B -- the template plate plus three key cards"
echo
echo "The template WITH fingerprints is what the SECOND stub screen and form B's"
echo "first plate carry. It shares the Template-ID with the first stub screen's"
echo "unseated chunk set and is NOT the same string: that one puts @2 at 2'."
echo
runcap "$OUT/keyed-template.md1.txt" '^md1' \
  "$MD" encode "$KEYED_TEMPLATE" --fingerprint "@0=$FP_A" --fingerprint "@1=$FP_A" \
    --fingerprint "@2=$FP_B" --force-chunked --group-size 0
gate "form-B template chunk count" "$(wc -l < "$OUT/keyed-template.md1.txt")" "2"
gate "form-B template chunk 1" "$(sed -n 1p "$OUT/keyed-template.md1.txt")" \
  "md1fxnz3qs9qjtvyyy5jmpprjjtvyyy4qqxpxqcyy2v6tjv6a4w46h2at4w46h2at4w46h2shlte30qvuhvrq"
gate "form-B template chunk 2" "$(sed -n 2p "$OUT/keyed-template.md1.txt")" \
  "md1fxnz3qsw46h2at4w46h2at4w46h2at4w46h2msqqqv4qp0npeutks2tnchdq4ts6yd7yq5swf47peq533w"
echo

echo "The three key cards, EACH STAMPED WITH BOTH STUBS. The device's census"
echo "counts one plate per card, at these chunk counts."
echo
runcap "$OUT/cards/slot0.mk1.txt" '^mk1' \
  "$MK" encode --xpub "$K0" --origin-fingerprint "$FP_A" --origin-path "m/48'/0'/0'/2'" \
    --policy-id-stub 531ab9e1 --policy-id-stub 4dd749a8 --group-size 0
runcap "$OUT/cards/slot1.mk1.txt" '^mk1' \
  "$MK" encode --xpub "$K1" --origin-fingerprint "$FP_A" --origin-path "m/48'/0'/1'/2'" \
    --policy-id-stub 531ab9e1 --policy-id-stub 4dd749a8 --group-size 0
runcap "$OUT/cards/slot2.mk1.txt" '^mk1' \
  "$MK" encode --xpub "$K2" --origin-fingerprint "$FP_B" --origin-path "m/48'/0'/0'/2'" \
    --policy-id-stub 531ab9e1 --policy-id-stub 4dd749a8 --group-size 0
gate "card @0 chunk count" "$(wc -l < "$OUT/cards/slot0.mk1.txt")" "2"
gate "card @1 chunk count" "$(wc -l < "$OUT/cards/slot1.mk1.txt")" "3"
gate "card @2 chunk count" "$(wc -l < "$OUT/cards/slot2.mk1.txt")" "2"
echo
echo "Read each card back -- the round trip that makes the encode evidence:"
echo
for s in 0 1 2; do
  mapfile -t CARD < "$OUT/cards/slot$s.mk1.txt"
  run "$MK" decode "${CARD[@]}"
done

echo "########## 6. THE KEYLESS ARM -- and the plate the operator cuts"
echo
echo "tr; ONE path, 2-of-3; no lock, no hash; NO keys at all."
echo
run "$MD" compose --wrapper tr --path 2of3
KEYLESS_TEMPLATE="tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*))"
printf '%s\n' "$KEYLESS_TEMPLATE" > "$OUT/keyless-tr.template"
run cat "$OUT/keyless-tr.template"

echo "--force-chunked, and this is the one place it is load-bearing rather than"
echo "habitual. Without it this template encodes UNCHUNKED (47 characters) while"
echo "the DEVICE is chunk-form-always. Both forms verify, inspect and decode"
echo "identically -- same template, same ids -- so the substitution is invisible"
echo "to every md verb, and only the byte comparison below catches it."
echo
runcap "$OUT/keyless-tr.md1.txt" '^md1' \
  "$MD" encode "$KEYLESS_TEMPLATE" --force-chunked --group-size 0
KEYLESS_MD1="$(cat "$OUT/keyless-tr.md1.txt")"
gate "keyless md1 chunk count" "$(wc -l < "$OUT/keyless-tr.md1.txt")" "1"
gate "keyless md1 string" "$KEYLESS_MD1" "md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3"
gate "keyless md1 length" "${#KEYLESS_MD1}" "56"
echo

echo "For contrast, THE FORM THIS ORACLE IS NOT -- the host's unchunked default:"
echo
run "$MD" encode "$KEYLESS_TEMPLATE" --group-size 0
echo "Both of the next two exit 0, against DIFFERENT strings. That is the point:"
echo "no verify step can tell the two forms apart."
echo
UNCHUNKED="$("$MD" encode "$KEYLESS_TEMPLATE" --group-size 0 2>/dev/null)"
gate "the unchunked form is a DIFFERENT string" \
  "$([ "$UNCHUNKED" != "$KEYLESS_MD1" ] && echo different || echo same)" "different"
gate "the unchunked form's length" "${#UNCHUNKED}" "47"
run "$MD" verify --template "$KEYLESS_TEMPLATE" "$KEYLESS_MD1"
run "$MD" verify --template "$KEYLESS_TEMPLATE" "$UNCHUNKED"

runcap "$OUT/keyless-tr.id.txt" '^(wallet-descriptor-template-id|md1-encoding-id):|^  @' \
  "$MD" inspect "$KEYLESS_MD1"
gate "keyless Template-ID" \
  "$(awk -F': ' '/^wallet-descriptor-template-id:/{print $2}' "$OUT/keyless-tr.id.txt")" \
  "e0863d3ccac31a64d3b5e14b85ccd6c0"
echo
run "$MD" decode "$KEYLESS_MD1"

echo "########## Artifacts for the device half"
run ls -la "$OUT" "$OUT/cards"

echo "########## RESULT"
if [ "$FAILURES" -eq 0 ]; then
  echo "all gates passed"
  exit 0
fi
echo "$FAILURES GATE FAILURE(S) -- the host oracle does not say what this"
echo "transcript claims. Do NOT re-pin: find out which side moved."
exit 1
