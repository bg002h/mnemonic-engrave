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
runcap "$W/out/pathological/md1.txt" '^md1' \
  "$MD" encode --group-size 0 --force-chunked --path bip48 "$T"

# Now — and only now — the chunk set exists on disk for the steps below.
MD1S=$(tr '\n' ' ' < "$W/out/pathological/md1.txt")

echo "########## 4. the chunk set decodes back to the same 11-key policy"
run "$MD" inspect $MD1S

echo "########## 5. OBSTACLE 1 — mk cannot derive the stub from a CHUNKED md1"
XPUB=$(grep '^xpub' "$W/inputs-pathological/keys/key-00.xpub")
# Origin read from the key file rather than typed in, so it cannot drift from
# the key it describes — it was hardcoded to key-00's old bip84 origin, which
# item 5 has since moved to m/48'/0'/0'/2'.
K0FP=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$W/inputs-pathological/keys/key-00.xpub" | head -1)
K0PATH=$(sed -n 's/.*origin \[[0-9a-f]*\/\([^]]*\)\].*/\1/p' "$W/inputs-pathological/keys/key-00.xpub" | head -1)
FIRST=$(head -1 "$W/out/pathological/md1.txt")
run "$MK" encode --xpub "$XPUB" --origin-fingerprint "$K0FP" \
  --origin-path "m/$K0PATH" --from-md1 "$FIRST" --group-size 0

echo "########## 6. the stub, derived by hand from the template id"
echo "MEASURED, not cited: on a single-string wallet where --from-md1 DOES work,"
echo "mk embedded stub 726a6663 while that wallet's ids were"
echo "  wallet-descriptor-template-id: 726a666305756435...  <-- the stub matches THIS"
echo "  wallet-policy-id:              f05e8a1c282f7740...  <-- and NOT this,"
echo "though SPEC_mk_v0_1.md 3.3 names the WalletPolicyId. So mk follows the"
echo "template-id, and the stub below is DERIVED from it rather than typed in."
echo
run bash -c "$MD inspect $MD1S 2>/dev/null | sed -n 's/^wallet-descriptor-template-id: //p'"

# The stub used to be the literal 5b48af35, hardcoded two lines below with
# nothing checking it still matched. It is now derived from the same command
# the step above prints, so the narrative and the cards cannot drift apart.
# (Verified equal to the old hardcode at the time of the change.)
STUB=$("$MD" inspect $MD1S 2>/dev/null \
        | sed -n 's/^wallet-descriptor-template-id: //p' | cut -c1-8)
if [ -z "$STUB" ]; then
  echo "FATAL: could not derive the policy-id stub from the template id" >&2
  exit 1
fi

echo "########## 7. the eleven key cards — ALL of them, each with its own origin"
# Was: one card for key-00 only, under a heading that said eleven.
: > "$W/out/pathological/mk-encode-raw.txt"
for f in "$W"/inputs-pathological/keys/key-*.xpub; do
  KX=$(grep '^xpub' "$f")
  KFP=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$f" | head -1)
  KPATH=$(sed -n 's/.*origin \[[0-9a-f]*\/\([^]]*\)\].*/\1/p' "$f" | head -1)
  if [ -z "$KX" ] || [ -z "$KFP" ] || [ -z "$KPATH" ]; then
    echo "FATAL: $f is missing an xpub or an origin header" >&2
    exit 1
  fi
  # I-2: the encode's exit status USED TO BE DISCARDED by the pipe, so a typo
  # in a key header silently dropped that cosigner's card from the ENGRAVED
  # bundle at exit 0 — 23 plates instead of 25, and every later plate caption
  # naming the wrong master because caption index and card index desynced.
  # A short bundle is a worse failure than the stale one this commit removed.
  if ! CARD=$("$MK" encode --xpub "$KX" --origin-fingerprint "$KFP" \
        --origin-path "m/$KPATH" --policy-id-stub "$STUB" --group-size 0 2>&1); then
    echo "FATAL: mk encode failed for $f:" >&2
    printf '%s\n' "$CARD" >&2
    exit 1
  fi
  # Chunk count is NOT fixed at 2. Item 5 moved these keys to BIP-48's
  # four-level origin `m/48'/0'/N'/2'`, and the longer path pushes most cards
  # to THREE chunks (account 0 still fits in two). This guard caught that
  # assumption the moment it became false, which is the whole point of it —
  # so it now demands "at least one", not "exactly two".
  NL=$(printf '%s\n' "$CARD" | grep -c '^mk1')
  if [ "$NL" -lt 1 ]; then
    echo "FATAL: $f produced no mk1 lines" >&2
    exit 1
  fi
  printf '%s\n' "$CARD" | grep '^mk1' >> "$W/out/pathological/mk-encode-raw.txt"
done

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
for f in "$W"/inputs-pathological/keys/key-*.xpub; do
  ki=$(basename "$f" .xpub | sed 's/key-0*//'); [ -z "$ki" ] && ki=0
  KX=$(grep '^xpub' "$f")
  KFP=$(sed -n 's/.*origin \[\([0-9a-f]*\)\/.*/\1/p' "$f" | head -1)
  KPATH=$(sed -n 's/.*origin \[[0-9a-f]*\/\([^]]*\)\].*/\1/p' "$f" | head -1)
  nth=0
  tot=$("$MK" encode --xpub "$KX" --origin-fingerprint "$KFP" \
        --origin-path "m/$KPATH" --policy-id-stub "$STUB" --group-size 0 \
        2>/dev/null | grep -c '^mk1')
  while read -r card; do
    nth=$((nth + 1))
    printf '%s %s %s %s %s %s\n' "$card" "$ki" "$nth" "$tot" "$KFP" "$KPATH" \
      >> "$W/out/pathological/card-index.txt"
  done < <("$MK" encode --xpub "$KX" --origin-fingerprint "$KFP" \
             --origin-path "m/$KPATH" --policy-id-stub "$STUB" --group-size 0 \
             2>/dev/null | grep '^mk1')
done
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
