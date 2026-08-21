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

# runcap <outfile> <keep-regex> <cmd...>
#
# Runs <cmd...> like run() does for transcript purposes, and ALSO writes the
# lines of its STDOUT matching <keep-regex> to <outfile>.
#
# F-210: this is the mechanism these scripts never had. `run()` echoes a
# command's output into the transcript and then throws it away, so several
# intermediates were READ by later steps that nothing ever WROTE — the journey
# only ever "worked" because some earlier, uncommitted process had left those
# files lying in out/.
#
# The regex is a required argument rather than an optional one because the
# consumers slurp WHOLE files: `md encode` prints `chunk-set-id: 0x…` on stdout
# alongside the md1 lines, and feeding that to `md inspect` fails. Capturing
# raw stdout would have replaced "file missing" with "file subtly wrong", which
# is worse.
#
# Note on ordering: run() interleaves via `2>&1`; runcap prints all stdout then
# all stderr. For every command captured here stderr is a trailing note, so the
# transcript text is unchanged in practice.
runcap() {
  local out="$1" keep="$2"; shift 2
  echo "\$ $*"
  local sout serr rc
  sout="$(mktemp)"; serr="$(mktemp)"
  "$@" >"$sout" 2>"$serr"; rc=$?
  cat "$sout"; cat "$serr"
  # I-2 (exec review): a ZERO-MATCH capture used to be swallowed by `|| true`,
  # leaving an empty file. Seven of eight consumers then fail loudly, but
  # `transcript_pathological.sh`'s template-id step printed BLANK at [exit 0].
  # That step no longer feeds a hardcoded --policy-id-stub (F-127 let the cards
  # derive their own binding), but it still CROSS-CHECKS the stub mk chose, and
  # a blank capture would silently retire the check rather than fail it.
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

# The two journeys used to share $W/out, so running one CLOBBERED the other's
# engraved bundle, key cards, manifest and plates — and a PDF built afterwards
# would then read the wrong journey's artifacts. Same stale-artifact class as
# F-210, one directory up. Each journey owns its own subtree now.
mkdir -p "$W/out/pathological"

T=$(cat "$W/inputs-pathological/wallet-policy.txt")
# NOTE: MD1S is deliberately NOT set here. It used to be, reading out/md1.txt
# at this line — sixteen lines BEFORE step 3, the only command that could
# produce it. It is assigned immediately after step 3 instead.

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
# NO --path. The policy now states each slot's TRUE origin inline, and --path
# would flatten all eleven onto one shared value -- which is what this journey
# used to engrave, and what its own restore test measured as recovering 3 of 11
# keys. Costs one extra chunk (4 vs 3) and changes nothing else: the template-id
# is identical, so the mk1 key plates' stubs still bind.
runcap "$W/out/pathological/md1.txt" '^md1' \
  "$MD" encode --group-size 0 --force-chunked "$T"

# Now — and only now — the chunk set exists on disk for the steps below.
MD1S=$(tr '\n' ' ' < "$W/out/pathological/md1.txt")

echo "########## 4. the chunk set decodes back to the same 11-key policy"
run "$MD" inspect $MD1S

echo "########## 5. mk derives the stub from the CHUNKED md1 itself"
# This section used to be headed "OBSTACLE 1 -- mk cannot derive the stub from
# a CHUNKED md1", and everything below it existed to work around that: the
# stub was pulled out of `md inspect` by hand and passed as --policy-id-stub to
# all eleven encodes.
#
# F-127 closed the obstacle, and it was two defects, not one. mk vendored
# md-codec 0.34.0, which predates the v9 chunk wire series and refused every
# chunk with "wire-format version mismatch: got 9, expected 4"; and mk decoded
# each --from-md1 value INDEPENDENTLY, so even on a current codec a four-chunk
# card was four incomplete sets. Both are fixed: values are now grouped by the
# 20-bit chunk-set id in their wire header, and each GROUP yields one stub.
#
# Worth stating because it sets the scale of what was hidden: a keyed wallet
# policy is 246 data symbols and the codex32 regular code caps a single md1
# string at 80, so EVERY keyed card in the constellation is chunked. The
# capability was not degraded for large wallets -- it was absent for all of
# them, and only ever worked on templates small enough to fit one string.
XPUB=$(grep '^xpub' "$W/inputs-pathological/keys/key-00.xpub")
# Origin read from the key file rather than typed in, so it cannot drift from
# the key it describes.
K0FP=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$W/inputs-pathological/keys/key-00.xpub" | head -1)
K0PATH=$(sed -n 's/.*origin \[[0-9a-f]*\/\([^]]*\)\].*/\1/p' "$W/inputs-pathological/keys/key-00.xpub" | head -1)
mapfile -t MD1_CHUNKS < <(grep '^md1' "$W/out/pathological/md1.txt")
if [ "${#MD1_CHUNKS[@]}" -lt 2 ]; then
  echo "FATAL: expected a CHUNKED md1; got ${#MD1_CHUNKS[@]} chunk(s)" >&2
  exit 1
fi
FROM_MD1=(); for c in "${MD1_CHUNKS[@]}"; do FROM_MD1+=(--from-md1 "$c"); done
run "$MK" encode --xpub "$XPUB" --origin-fingerprint "$K0FP" \
  --origin-path "m/$K0PATH" "${FROM_MD1[@]}" --group-size 0

echo "########## 6. the stub mk chose, cross-checked against the template id"
# The hand-derivation that used to live here is now a CHECK rather than a
# workaround, and it is worth keeping in that form: it is the only thing in
# this journey that would notice if mk's form-aware dispatch silently switched
# to the WalletPolicyId. This card is a keyless template
# (`wallet-policy-mode: false`), so the stub must be the top 4 bytes of the
# key-stable WalletDescriptorTemplateId, NOT the key-dependent WalletPolicyId.
run bash -c "$MD inspect $MD1S 2>/dev/null | grep -E 'wallet-policy-mode|template-id|wallet-policy-id:'"

STUB=$("$MD" inspect $MD1S 2>/dev/null \
        | sed -n 's/^wallet-descriptor-template-id: //p' | cut -c1-8)
if [ -z "$STUB" ]; then
  echo "FATAL: could not derive the policy-id stub from the template id" >&2
  exit 1
fi
POLICY_STUB=$("$MD" inspect $MD1S 2>/dev/null \
        | sed -n 's/^wallet-policy-id: //p' | cut -c1-8)
# Read the stub back OUT of the card mk just built, rather than trusting that
# it used the one we expect. `mk decode` is a different code path from the
# derivation under test.
MK_STUB=$("$MK" encode --xpub "$XPUB" --origin-fingerprint "$K0FP" \
        --origin-path "m/$K0PATH" "${FROM_MD1[@]}" --group-size 0 2>/dev/null \
        | grep '^mk1' | tr '\n' ' ' \
        | xargs "$MK" decode 2>/dev/null \
        | sed -n 's/^policy_id_stubs: *//p' | head -1)
if [ "$MK_STUB" != "$STUB" ]; then
  echo "FATAL: mk embedded stub '$MK_STUB' but the template id says '$STUB'" >&2
  exit 1
fi
if [ -n "$POLICY_STUB" ] && [ "$MK_STUB" = "$POLICY_STUB" ]; then
  echo "FATAL: mk used the WalletPolicyId stub for a KEYLESS template" >&2
  exit 1
fi
echo "mk --from-md1 embedded $MK_STUB == template-id prefix $STUB (and != policy-id $POLICY_STUB)"
echo

echo "########## 7. the eleven key cards — ALL of them, in ONE command"
# Was: one `mk encode` per cosigner in a shell loop, because `mk encode --xpub`
# took a single key. F-223 moved the loop inside the tool -- `mk encode --keys`
# mints one card per record -- so eleven invocations became one. The output is
# byte-identical to the loop it replaces; that equivalence is pinned by
# `batch_matches_per_key_loop` in mnemonic-key, not asserted here.
#
# The key file is BIP-380 origin notation, one record per line. It is BUILT
# from the same key-*.xpub files the loop read, so nothing new is trusted --
# and a record carries its fingerprint, path and key together, which is the
# property that makes the desync the old loop could suffer impossible.
KEYFILE="$W/out/pathological/keys.txt"
KEYMETA="$W/out/pathological/keys-meta.txt"
: > "$KEYFILE"
: > "$KEYMETA"
for f in "$W"/inputs-pathological/keys/key-*.xpub; do
  KX=$(grep '^xpub' "$f")
  KFP=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$f" | head -1)
  KPATH=$(sed -n 's/.*origin \[[0-9a-f]*\/\([^]]*\)\].*/\1/p' "$f" | head -1)
  if [ -z "$KX" ] || [ -z "$KFP" ] || [ -z "$KPATH" ]; then
    echo "FATAL: $f is missing an xpub or an origin header" >&2
    exit 1
  fi
  printf '[%s/%s]%s\n' "$KFP" "$KPATH" "$KX" >> "$KEYFILE"
  # Parallel metadata, SAME ORDER, for the card index below: the key's own
  # number plus the origin it declares. Written here rather than re-derived
  # later so the two files cannot disagree about what record N is.
  ki=$(basename "$f" .xpub | sed 's/key-0*//'); [ -z "$ki" ] && ki=0
  printf '%s %s %s\n' "$ki" "$KFP" "$KPATH" >> "$KEYMETA"
done
run head -2 "$KEYFILE"

# I-2: the encode's exit status USED TO BE DISCARDED by the pipe, so a typo in
# a key header silently dropped that cosigner's card from the ENGRAVED bundle
# at exit 0 -- 23 plates instead of 25, and every later plate caption naming
# the wrong master because caption index and card index desynced. A short
# bundle is a worse failure than the stale one that commit removed. With
# --keys the whole run fails on a bad record, naming its line number, so a
# partial bundle is no longer reachable at all.
CARDS_OUT="$W/out/pathological/mk-encode-stdout.txt"
CARDS_ERR=$(mktemp)
echo "\$ $MK encode --keys $KEYFILE ${#FROM_MD1[@]} --from-md1 args --group-size 0"
if ! "$MK" encode --keys "$KEYFILE" "${FROM_MD1[@]}" --group-size 0 \
      > "$CARDS_OUT" 2> "$CARDS_ERR"; then
  echo "FATAL: mk encode --keys failed:" >&2
  cat "$CARDS_ERR" >&2
  rm -f "$CARDS_ERR"
  exit 1
fi
cat "$CARDS_ERR"; rm -f "$CARDS_ERR"
echo "[exit 0]"
echo
# stdout ONLY, so the blank line BETWEEN cards still delimits them. The
# watch-only advisory goes to stderr; folding it in with 2>&1 would land a
# non-mk1 line inside a card block and merge two cards into one.
grep '^mk1' "$CARDS_OUT" > "$W/out/pathological/mk-encode-raw.txt"

# Belt and braces: the engraved bundle must have exactly the expected shape.
# Belt and braces: every key must have contributed at least one chunk. The
# total is no longer 2*NKEYS (see above), so the invariant is "no key silently
# missing" rather than an exact line count.
NKEYS=$(ls "$W"/inputs-pathological/keys/key-*.xpub | wc -l)
GOT=$(grep -c '^mk1' "$W/out/pathological/mk-encode-raw.txt")
if [ "$GOT" -lt "$NKEYS" ]; then
  echo "FATAL: $NKEYS keys produced only $GOT mk1 lines — a card is missing" >&2
  exit 1
fi
echo "note: $NKEYS key cards -> $GOT mk1 chunks (BIP-48 origins push most cards to 3)"

# Record which KEY produced each mk1 STRING, so the document can caption a
# plate from the plate's own content.
#
# The first attempt recorded per-key chunk COUNTS and let the builder walk them
# in key order. That was wrong and worse than the bug it replaced: `me bundle`
# emits plates in chunk_set_id order, not key order, so all 30 card captions
# named the wrong key — a silent wrong answer where there had been a loud
# IndexError. Keying on the string removes the ordering assumption entirely.
: > "$W/out/pathological/card-index.txt"
# Derived from the SINGLE batch output above rather than from 22 more `mk
# encode` calls (two per key: one for the chunk total, one for the strings).
# That loop is what made this transcript run mk encode 33 times; it now runs
# it twice, and neither extra call can disagree with the cards actually
# engraved because there are no extra calls.
#
# `--keys` emits cards in FILE order separated by a blank line, and keys-meta
# was written in that same order, so block N belongs to meta line N. awk's
# paragraph mode (RS="") splits on the blank lines.
awk -v RS= -v meta="$KEYMETA" -v out="$W/out/pathological/card-index.txt" '
# RS="" governs getline-from-file too, so the meta file arrives as ONE
# paragraph rather than 11 lines. Split it on newlines explicitly; reading it
# line-wise here would silently give n=1 and mis-attribute every plate.
BEGIN {
  n = 0
  while ((getline l < meta) > 0) {
    c = split(l, T, "\n")
    for (i = 1; i <= c; i++) if (T[i] != "") m[n++] = T[i]
  }
}
{
  if (NR > n) { print "FATAL: more card blocks than key records" > "/dev/stderr"; exit 1 }
  split(m[NR-1], M, " ")
  tot = split($0, L, "\n")
  for (j = 1; j <= tot; j++)
    printf "%s %s %s %s %s %s\n", L[j], M[1], j, tot, M[2], M[3] >> out
}
END { if (NR != n) { print "FATAL: " NR " card blocks for " n " key records" > "/dev/stderr"; exit 1 } }
' "$CARDS_OUT" || exit 1

run wc -l "$W/out/pathological/card-index.txt"
run wc -l "$W/out/pathological/mk-encode-raw.txt"
run "$MK" decode $(sed -n '1,2p' "$W/out/pathological/mk-encode-raw.txt")

echo "########## 7b. the bundle this journey ENGRAVES is BUILT here, not shipped"
echo "F-210/I-1: inputs-pathological/backup-strings.txt used to be a tracked"
echo "fixture that nothing produced. It drifted against mk, so the journey"
echo "printed one card and engraved a different one for the same key. The"
echo "engraved file is now assembled from this run's own md1 chunks and key"
echo "cards, so print and engrave cannot disagree."
echo
cat "$W/out/pathological/md1.txt" "$W/out/pathological/mk-encode-raw.txt" > "$W/out/pathological/backup-strings.txt"
run wc -l "$W/out/pathological/backup-strings.txt"

echo "########## 8. me bundle: validates, and prints the plate checklist"
rm -rf "$W/out/pathological/plates" && mkdir -p "$W/out/pathological/plates"
run "$ME" bundle --in "$W/out/pathological/backup-strings.txt" --preview "$W/out/pathological/plates" --png --manifest "$W/out/pathological/manifest.json"

echo "########## 9. the ids, and which one mk actually uses for the stub"
run bash -c "$MD inspect $MD1S 2>/dev/null | grep -E 'policy-id|template-id'"

echo "########## 9b. THE ADDRESSES — the check this wallet could never do before"
echo "Item 5: the eleven keys used to sit at BIP-84's three-level single-sig"
echo "account path, while wsh(<miniscript>) is a MultiSig script context that"
echo "requires a depth-FOUR xpub. md refused every one of them, so NO ADDRESS"
echo "could be derived for this wallet by any tool -- which is why no journey"
echo "ever showed one. Re-derived at m/48'/0'/N'/2', they compose."
echo
echo "This is the FUNCTIONAL half of a round trip: the structural half proves"
echo "the bytes survived, and this proves they mean the wallet we intended."
echo "Compare these against your coordinator before engraving anything."
echo
KEYARGS=()
for f in "$W"/inputs-pathological/keys/key-*.xpub; do
  ki=$(basename "$f" .xpub | sed 's/key-0*//'); [ -z "$ki" ] && ki=0
  KEYARGS+=(--key "@$ki=$(grep '^xpub' "$f")")
done
# A full-policy md1 (keys embedded) is what carries the origin into the
# derivation. The engraved artifact stays the KEYLESS template above; this is
# the same wallet with its keys supplied, purely to check the addresses.
FULL=$("$MD" encode --group-size 0 --force-chunked --path bip48 "${KEYARGS[@]}" "$T" 2>/dev/null | grep '^md1')
if [ -z "$FULL" ]; then
  echo "FATAL: could not build the keyed policy for address derivation" >&2
  exit 1
fi
run "$MD" address $FULL --chain 0 --count 3
run "$MD" address $FULL --chain 1 --count 3

echo "########## 10. the seed, and the refusal that still holds"
run cat "$W/inputs-pathological/seeds/master-A.seed"
run "$MS" encode --phrase "$(cat "$W/inputs-pathological/seeds/master-A.seed")"
MS1=$("$MS" encode --phrase "$(cat "$W/inputs-pathological/seeds/master-A.seed")" --no-engraving-card 2>/dev/null | tr -d ' ')
run "$ME" --in <(printf '%s\n' "$MS1") --hex

echo "########## 9c. THE RESTORE TEST — what a card-only restore actually recovers"
# Step 3 above proves the BYTES survive. This asks the different question: if
# someone holds these plates and their seeds, do they get the wallet back?
#
# They do not. `--path` recorded ONE shared origin while the eleven keys sit at
# four account indices across three masters, so a restorer trusting the
# descriptor card derives account 0 from each master and recovers three of
# eleven. The script exits non-zero if the measured damage stops matching what
# the document claims -- which is how the document's original "@3-@10" was
# caught being wrong in both directions.
run python3 "$W/restore_test_pathological.py"
