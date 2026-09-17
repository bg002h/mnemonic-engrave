#!/usr/bin/env bash
# Run every command in CLI_SPINE.md and check the outputs that matter.
#
# The point is not that the commands exit 0 -- it is that the NUMBERS and the
# REFUSALS are still what the talk track says. A spine whose Template-ID has
# drifted is a demo that contradicts the device in front of an audience.
#
# Run it before you travel, and after any release.
set -uo pipefail
cd "$(dirname "$0")"

pass=0; fail=0
ok()   { printf '  \033[32mok\033[0m   %s\n' "$1"; pass=$((pass+1)); }
bad()  { printf '  \033[31mFAIL\033[0m %s\n     %s\n' "$1" "$2"; fail=$((fail+1)); }
need() { command -v "$1" >/dev/null || { echo "missing: $1 -- see CLI_SPINE.md"; exit 2; }; }

need md; need ms

# CAPABILITY CHECK, not just presence. `md compose` and `ms split --in` are both
# NEWER than the crates.io releases: published md-cli 0.13.0 has no `compose`
# subcommand at all, and published ms-cli 0.14.0 has no `--in` on `split`. A
# rehearsal run against those fails eight assertions with empty output, which
# reads like a broken gate rather than the wrong binary. Say it plainly instead.
if ! md compose --help >/dev/null 2>&1; then
  echo "This md has no 'compose' subcommand -- it is the crates.io build." >&2
  echo "  cargo install --git https://github.com/bg002h/descriptor-mnemonic md-cli" >&2
  exit 2
fi
if ! ms split --help 2>&1 | grep -q -- '--in'; then
  echo "This ms has no '--in' on split -- it is the crates.io build, and it" >&2
  echo "would make you put a seed on the command line." >&2
  echo "  cargo install --git https://github.com/bg002h/mnemonic-secret ms-cli" >&2
  exit 2
fi

tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

echo "== versions"
printf '   md %s\n   ms %s\n' "$(md --version | awk '{print $2}')" "$(ms --version | awk '{print $2}')"

echo "== 1 compose"
H=2d711642b726b04401627ca9fbac32f5c8530fb1903cc4db02258717921a4881
t=$(md compose --wrapper wsh --preset 'plain-multisig,2of3' 2>/dev/null)
[[ $t == wsh\(sortedmulti\(2,@0/48\'* ]] && ok "plain-multisig lowers to sortedmulti" || bad "plain-multisig" "$t"
t=$(md compose --wrapper tr --preset 'decaying-multisig,2of3,2of5,older1=1000,older2=26280,after=900000' 2>/dev/null)
[[ $t == *older\(1000\)* && $t == *older\(26280\)* && $t == *after\(900000\)* ]] \
  && ok "decaying-multisig carries all three locks" || bad "decaying-multisig" "$t"
t=$(md compose --wrapper wsh --preset "hashlock-gated,sha256=$H,older=4320" 2>/dev/null)
[[ $t == *sha256\($H\)* && $t == *older\(4320\)* ]] && ok "hashlock-gated commits to the digest" || bad "hashlock-gated" "$t"
ZEN=$(md compose --wrapper tr --path '1of1,older=32768' 2>/dev/null)
[[ $ZEN == *older\(32768\)* ]] && ok "zen-hodl locks 32768 blocks" || bad "zen-hodl" "$ZEN"

echo "== 2 damage and repair"
BAD=md1yqfdsqsjuqqpr5e55uzqqgqqqrqqvf4d7h59r2
GOOD=md1yqfdsssjuqqcr5e55uqqqgqqq6qqvf4d7h59r2
if md decode "$BAD" >/dev/null 2>&1; then bad "the damaged string should NOT decode" "it decoded"; else ok "damaged string is rejected"; fi
out=$(md repair "$BAD" 2>/dev/null); rc=$?
[[ $rc -eq 5 ]] && ok "md repair exits 5 (REPAIR_APPLIED)" || bad "md repair exit code" "got $rc, want 5"
grep -q '4 corrections' <<<"$out" && ok "repair reports 4 corrections" || bad "correction count" "$(head -2 <<<"$out")"
[[ $(grep -o '^md1[a-z0-9]*' <<<"$out" | head -1) == "$GOOD" ]] \
  && ok "repaired string is byte-identical to the original" || bad "repair output" "$(grep -o '^md1[a-z0-9]*' <<<"$out" | head -1)"

echo "== 3 the cross-implementation identity"
ID=$(md inspect "$GOOD" 2>/dev/null | awk '/wallet-descriptor-template-id:/{print $2}')
[[ $ID == 73c33a5dea17b45376a6246995df0cbb ]] \
  && ok "Template-ID matches what the device shows ($ID)" \
  || bad "Template-ID DRIFTED -- the talk track now contradicts the device" "got $ID"

echo "== 4 shamir"
printf 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about\n' > "$tmp/seed.txt"
ms split --in "$tmp/seed.txt" -k 3 -n 5 --group-size 0 > "$tmp/shares.txt" 2>/dev/null
n=$(grep -c '^ms1' "$tmp/shares.txt")
[[ $n -eq 5 ]] && ok "split produced 5 shares" || bad "share count" "got $n"
grep -o '^ms1[a-z0-9]*' "$tmp/shares.txt" | sed -n '1p;3p;5p' > "$tmp/three.txt"
grep -q 'abandon abandon' < <(ms combine --in "$tmp/three.txt" 2>&1) \
  && ok "any 3 shares rebuild the phrase" || bad "combine of 3 failed" ""
err=$(ms combine --in <(grep -o '^ms1[a-z0-9]*' "$tmp/shares.txt" | sed -n '1p;3p') 2>&1 >/dev/null)
grep -q 'have 2, need 3' <<<"$err" && ok "2 shares are refused, by that exact wording" || bad "threshold wording" "$err"

echo "== 5 the argv refusal"
err=$(ms split --phrase "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" -k 3 -n 5 2>&1 >/dev/null)
grep -q 'ARGV' <<<"$err" && ok "a seed on argv is refused" || bad "argv guard did not fire" "$(head -1 <<<"$err")"

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[[ $fail -eq 0 ]] || exit 1
