#!/usr/bin/env bash
# derive-pathological-keys.sh — regenerate inputs-pathological/keys/*.xpub from
# the three committed master seeds.
#
# WHY THIS EXISTS (two reasons, the second is the general one).
#
# 1. DoNextList item 5. The eleven keys were originally derived at BIP-84's
#    THREE-level single-sig account path `m/84'/0'/N'`. But the wallet is
#    `wsh(<miniscript>)`, which md classifies as a MultiSig script context, and
#    that context requires a depth-FOUR xpub:
#
#        md: --key @0: expected depth 4 for this script context, got 3
#
#    The check is an exact match on the xpub's own depth byte
#    (`crates/md-cli/src/parse/keys.rs:67-77`), and it is a THRESHOLD, not a
#    scale: one tap leaf or eleven, `wsh(multi(2,…))` or a deeply nested tree —
#    all want exactly 4. Only a bare single-key wallet (`wpkh`, `pkh`,
#    `sh(wpkh(`, key-path-only `tr`) wants 3. So NO ADDRESS could ever be
#    derived for this wallet, which is why no journey has ever shown one: it
#    could not. Re-derived here at BIP-48's P2WSH path `m/48'/0'/N'/2'`.
#
# 2. The key files themselves had NO PRODUCER — the same defect class as
#    F-210 and I-1, one layer deeper. They were artifacts somebody once
#    generated, committed, and could not regenerate. (The OPERATOR journey's
#    twelve keys still have none; they were reproducible only by luck, and
#    that is filed separately.) An artifact nobody can regenerate is an
#    artifact nobody can check.
#
# THE MASTERS DO NOT CHANGE. Only the account path does, so the master
# fingerprints are identical before and after — verified 73c5da0a / b8688df1 /
# 28645006 in both. The three seeds are BIP-39's own published test vectors and
# are public by construction.
#
# Usage:  bash derive-pathological-keys.sh          # rewrite the key files
#         bash derive-pathological-keys.sh --check  # verify, change nothing
set -uo pipefail

W="$(cd "$(dirname "$0")" && pwd)"
MS="${MS:-/scratch/code/shibboleth/mnemonic-secret/target/release/ms}"
KEYS="$W/inputs-pathological/keys"
SEEDS="$W/inputs-pathological/seeds"
CHECK=0
[ "${1:-}" = "--check" ] && CHECK=1

[ -x "$MS" ] || { echo "FATAL: ms not found at $MS (set MS=)" >&2; exit 1; }

# @i → master letter, account index, and the tier prose the file carries.
# The tier text is what makes each key legible in the journey document, so it
# is data here rather than something a reader has to reconstruct.
ROWS=(
  "0|A|0|tier 1 (3-of-3, after height 1000000 + secret word)"
  "1|A|1|tier 1"
  "2|A|2|tier 1"
  "3|A|3|tier 2 (2-of-3, after time 1893456000 + secret word)"
  "4|B|0|tier 2"
  "5|B|1|tier 2"
  "6|B|2|tier 3 (both, older 65535 blocks)"
  "7|B|3|tier 3"
  "8|C|0|tier 4 (any 1 of 3, older 4255898 = ~365d)"
  "9|C|1|tier 4"
  "10|C|2|tier 4"
)

rc=0
for row in "${ROWS[@]}"; do
  IFS='|' read -r i m acct tier <<< "$row"
  seed="$SEEDS/master-$m.seed"
  [ -f "$seed" ] || { echo "FATAL: missing $seed" >&2; exit 1; }

  out=$(printf '%s' "$(cat "$seed")" \
        | "$MS" derive --phrase - --template bip48-p2wsh --account "$acct" 2>/dev/null)
  fp=$(printf '%s\n' "$out"   | sed -n 's/^master_fingerprint: *//p')
  path=$(printf '%s\n' "$out" | sed -n 's/^account_path: *//p')
  xpub=$(printf '%s\n' "$out" | sed -n 's/^account_xpub: *//p')

  if [ -z "$fp" ] || [ -z "$path" ] || [ -z "$xpub" ]; then
    echo "FATAL: ms derive produced no key for master-$m account $acct" >&2
    exit 1
  fi

  # `origin [fp/path]` is what transcript_pathological.sh parses back out, so
  # the header is an interface, not a comment. Strip the leading `m/`.
  # NOTE the trailing newline is re-added below: `$(...)` strips trailing
  # newlines, so building the body this way and writing it with `printf '%s'`
  # produced files with NO final newline. Caught by the control run (all 11
  # "mismatched" while a direct write diffed clean), not by reading.
  body=$(printf '# @%s — %s\n# master %s, origin [%s/%s]\n%s' \
          "$i" "$tier" "$m" "$fp" "${path#m/}" "$xpub")

  f=$(printf '%s/key-%02d.xpub' "$KEYS" "$i")
  if [ "$CHECK" = "1" ]; then
    if ! printf '%s\n' "$body" | diff -q - "$f" >/dev/null 2>&1; then
      echo "DRIFT: $f does not match a fresh derivation"
      rc=1
    fi
  else
    printf '%s\n' "$body" > "$f"
  fi
done

if [ "$CHECK" = "1" ]; then
  [ "$rc" = "0" ] && echo "all 11 key files match a fresh derivation"
  exit "$rc"
fi
echo "wrote 11 key files to $KEYS"
