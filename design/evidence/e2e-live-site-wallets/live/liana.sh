#!/usr/bin/env bash
# Liana v15.0 (LianaDescriptor::from_str via the committed harness, copied here, built against
# /scratch/code/shibboleth/.tmp/fable-liana-src-v15/liana @ v15.0) over every live descriptor + the gate's 2 controls.
set -euo pipefail
cd "$(dirname "$0")"
H=liana-harness/target/debug/liana-harness
ALL="kofn-nums kofn-liana tiered-nums tiered-liana kofn-nums-distinct kofn-liana-distinct tiered-nums-distinct tiered-liana-distinct"
python3 - $ALL <<'PY'
import json,sys
recs=[json.loads(l) for l in open("/scratch/code/shibboleth/mnemonic-engrave/design/evidence/f449-stage2/liana-live-gate-in.jsonl") if l.strip()]
out=open("liana-in.jsonl","w")
for r in recs:
    if r["role"].startswith("control"): out.write(json.dumps({"name":r["name"],"variant":r["role"],"desc":r["desc"]})+"\n")
for n in sys.argv[1:]:
    d=[l for l in open(f"{n}.descriptor.txt") if l.startswith("tr(")][0].strip()
    out.write(json.dumps({"name":"live-"+n,"variant":"device","desc":d})+"\n")
PY
# probe rule: every Liana-key descriptor must carry Liana's recipe over its own leaves
for n in $ALL; do case $n in *liana*)
  d=$(grep '^tr(' $n.descriptor.txt | sed 's/#.*//')
  want=$(echo "$d" | $H unspendable | sed -E 's/^tr\(([^/,]+).*/\1/')
  have=$(echo "$d" | sed -E 's/^tr\(([^/,]+).*/\1/')
  [[ "$have" == "$want" ]] && echo "probe-rule OK $n" || { echo "probe-rule FAIL $n have=$have want=$want"; exit 1; } ;;
esac; done
$H parse < liana-in.jsonl > liana-out.jsonl
python3 - <<'PY'
import json
for l in open("liana-out.jsonl"):
    r=json.loads(l)
    if r["ok"]: print(f'{r["name"]}: ACCEPT receive={r["receive"][:2]} change={r["change"][:2]} primary={r["primary"]["kind"]} recovery={[x["older"] for x in r["recovery"]]}')
    else: print(f'{r["name"]}: REFUSE {r["error"]}')
PY
