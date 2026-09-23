#!/usr/bin/env bash
# F-449 stage 4 Task 9 Step 1 (plan F6): Liana v15.0 over the four F-644 shapes, SPEC §7's constructed
# shape (one timelocked leaf) and two accepted controls, each with its internal key recomputed over its OWN
# leaves by the harness's `unspendable`. Run from the mnemonic-engrave root after scripts/liana-live-gate.sh PASSes.
set -euo pipefail
H=harnesses/liana/target/gate-build/debug/liana-harness
A="[73c5da0a/48'/0'/0'/3']xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zTpf7F2BuXCJ52i/<0;1>/*"
B="[3f635a63/48'/0'/0'/3']xpub6DXuQW1Q2Jpa1hNtFUcghdx7Q8kTDsqo7b54YAqZBNCH8EuSvmNSAKbAvkZ4HspgftJ1aqSMeFiZ4sr2QNEGm9geaEre3zDwiJD7C5gx5VH/<0;1>/*"
C="[66d455ea/48'/0'/0'/3']xpub6DXuQW1Q2JpZyteDRGW1pD34uhumfnZJfTsmjDkgd4xcq3L5XX2KUE1n4rmvcDT3RmdchfhbD9DkvSyVhUMBjUYi691iFszgKtf4Bfqe2nL/<0;1>/*"
D="[73c5da0a/48'/0'/1'/3']xpub6DXuQW1Q2JpZzLV9igdwdnmCSoaPVd4ZNZnvfgUsGvQ8AbNAhEmfBEMCMHctwZBuxWK8HkjqUW5F72MCSJCFfisVwRY62Kb1FuDZ66nNQe1/<0;1>/*"
E="[73c5da0a/48'/0'/2'/3']xpub6DXuQW1Q2JpZzZHXLadWbvXrMTD8ysfE7L4YZHsEvpWQ3KQ7CVporF7mSQKcphivSAdwGdLuLLHvrgaQXUeNMwpz5c1HAJHXgxvesUgj4Mb/<0;1>/*"
K="xpub661MyMwAqRbcFn1aHFGgZ359mzBqgj6rjSy73VLCdi92kFK4uHH71MRMGuWX5LqsouhUowavHR6PpPyxjctLm96D6JSZyPAr9RufepmuyEz/<0;1>/*"
ev=design/evidence/f449-stage4; mkdir -p $ev
cat > $ev/liana-probes-in.txt <<EOF
tr($K,{multi_a(2,$A,$B,$C),multi_a(2,$D,$E)})
tr($K,{multi_a(2,$A,$B),and_v(v:pk($C),after(800000))})
tr($K,{multi_a(2,$A,$B),{and_v(v:pk($C),older(100)),and_v(v:pk($D),older(100))}})
tr($K,multi_a(2,$A,$B,$C))
tr($K,and_v(v:pk($A),older(26280)))
tr($K,{multi_a(2,$A,$B),{and_v(v:pk($C),older(26280)),and_v(v:pk($D),older(52560))}})
tr($K,{pk($A),and_v(v:pk($B),older(26280))})
EOF
"$H" unspendable < $ev/liana-probes-in.txt > $ev/liana-probes-recomputed.txt
python3 - $ev <<'PY'
import json, sys
ev = sys.argv[1]
names = ["f644-two-unlocked-primaries", "f644-after-recovery", "f644-duplicate-older", "f644-no-recovery",
         "s7-single-timelocked-leaf", "nested-two-recoveries", "pk-primary-leaf-plus-recovery"]
with open(f"{ev}/liana-probes-parse-in.jsonl", "w") as f:
    for n, d in zip(names, [l.strip() for l in open(f"{ev}/liana-probes-recomputed.txt") if l.strip()]):
        f.write(json.dumps({"name": n, "variant": "liana-unspendable-xpub", "desc": d}) + "\n")
PY
"$H" parse < $ev/liana-probes-parse-in.jsonl > $ev/liana-probes-out.jsonl
python3 -c "import json;[print(r['name'],r['ok'],r.get('error','')) for r in map(json.loads,open('$ev/liana-probes-out.jsonl'))]"
python3 - $ev "$K" <<'PY'
import sys
ev,K=sys.argv[1],sys.argv[2].split('/')[0]
for i,l in enumerate(open(f"{ev}/liana-probes-recomputed.txt")):
    ik=l.strip()[3:].split('/')[0]
    print("probe",i+1,"internal key == K" if ik==K else "internal key recomputed (differs from K)")
PY
