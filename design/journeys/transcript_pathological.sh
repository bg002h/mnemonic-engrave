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

mkdir -p "$W/out"

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
runcap "$W/out/md1.txt" '^md1' \
  "$MD" encode --group-size 0 --force-chunked --path bip84 "$T"

# Now — and only now — the chunk set exists on disk for the steps below.
MD1S=$(tr '\n' ' ' < "$W/out/md1.txt")

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
runcap "$W/out/mk-encode-raw.txt" '^mk1' \
  "$MK" encode --xpub "$XPUB" --origin-fingerprint 73c5da0a \
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
