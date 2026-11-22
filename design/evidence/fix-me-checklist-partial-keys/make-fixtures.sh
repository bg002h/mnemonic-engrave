#!/usr/bin/env bash
# make-fixtures.sh — the producer of this directory's md1/mk1 fixtures for
# design/FIX_me_recovery_checklist_partial_keys.md (bh-me-C1).
#
# A signet 2-of-3 wsh(sortedmulti) at m/48'/1'/0'/2' whose three keys come
# from BIP-39's own published test vectors (public by construction; signet
# only; never fund). Each fixture embeds a different subset of the three xpubs.
# The tpubs were derived with `ms derive --phrase - --template bip48-p2wsh
# --network testnet` (testnet serialization = signet).
#
# Usage:  bash make-fixtures.sh           # rewrite the fixture files
#         bash make-fixtures.sh --check   # verify the committed files, change nothing
# Needs `md` and `mk` on PATH (measured with md 0.20.7).
# NOT covered: the *.notes / *.missing.json expectations (those are the spec's
# text, checked by scripts/fix-me-checklist-partial-keys-gate.sh).
set -uo pipefail
D="$(cd "$(dirname "$0")" && pwd)"
CHECK=0
[ "${1:-}" = "--check" ] && CHECK=1
K0=tpubDFH9dgzveyD8zTbPUFuLrGmCydNvxehyNdUXKJAQN8x4aZ4j6UZqGfnqFrD4NqyaTVGKbvEW54tsvPTK2UoSbCC1PJY8iCNiwTL3RWZEheQ # 73c5da0a
K1=tpubDEfobrrtptRTbKf4gysDhoabneABDTAcdj3Vbn4XwPsLE2pmqpizSPRG6zHsbAMuiSgWmWPsYCLHTKTPpyrGJ5rAoTpKoQNZcxodiPf2tSJ # b8688df1
K2=tpubDEwqCvJxKwKWX9xvRe48uofWJn1Y89Jn8UeH1Efrjb1UEVjUDy3URYTiqWaVCW7WdvHrL8XrSihHEhTwv5H3VDJoakjuCHiAnr6xcF2Xm4s # 28645006
T='wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))'
P=(--path "m/48'/1'/0'/2'" --network signet)
F3=(--fingerprint @0=73c5da0a --fingerprint @1=b8688df1 --fingerprint @2=28645006)
F02=(--fingerprint @0=73c5da0a --fingerprint @2=28645006)
TMP="$(mktemp -d "${TMPDIR:-/tmp}/mkfx.XXXXXX")"
trap 'rm -f "$TMP"/*; rmdir "$TMP"' EXIT
fail=0
emit() { # name, command...
    local n=$1; shift
    if ! "$@" >"$TMP/$n.txt" 2>"$TMP/$n.err"; then
        echo "FAIL $n: producer exited non-zero"; sed -n 1,3p "$TMP/$n.err"; fail=1; return
    fi
    if [ $CHECK = 1 ]; then
        if cmp -s "$TMP/$n.txt" "$D/$n.txt"; then echo "ok   $n"; else echo "FAIL $n differs from committed"; fail=1; fi
    else
        cp "$TMP/$n.txt" "$D/$n.txt"; echo "wrote $n.txt"
    fi
}
emit k0     md encode "$T" "${P[@]}" "${F3[@]}"
emit k1     md encode "$T" "${P[@]}" "${F3[@]}" --key "@0=$K0"
emit k1only md encode "$T" "${P[@]}" "${F3[@]}" --key "@1=$K1"
emit k02    md encode "$T" "${P[@]}" "${F3[@]}" --key "@0=$K0" --key "@2=$K2"
emit k3     md encode "$T" "${P[@]}" "${F3[@]}" --key "@0=$K0" --key "@1=$K1" --key "@2=$K2"
emit k1nofp md encode "$T" "${P[@]}" "${F02[@]}" --key "@0=$K0"
emit mk1-slot1 mk encode --xpub "$K1" --origin-fingerprint b8688df1 --origin-path "m/48'/1'/0'/2'" --from-md1-set "$D/k1.txt"
exit $fail
