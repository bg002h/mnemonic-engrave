#!/usr/bin/env bash
# For each captured live wallet: extract md1 + device addresses, decode with md, compare.
set -uo pipefail
cd "$(dirname "$0")"
ALL="kofn-nums kofn-liana tiered-nums tiered-liana kofn-nums-distinct kofn-liana-distinct tiered-nums-distinct tiered-liana-distinct"
python3 - $ALL <<'PY'
import json,re,sys
for n in sys.argv[1:]:
    d=json.load(open(f"live-{n}.json")); assert d["ok"], n; r=d["r"]
    flat=[s for e in r["strings"]["strings"] for s in e.split("\n") if s]
    open(f"{n}.md1.txt","w").write("\n".join(flat)+"\n")
    j="".join(r["consent"])
    addrs=re.findall(r"(Receive|Change)(\d):(?:Review)?(bc1p[0-9a-z]{58})",j)
    assert len(addrs)==4,(n,addrs)
    open(f"{n}.device-addrs.txt","w").write("\n".join(f"{a} {b} {c}" for a,b,c in addrs)+"\n")
    pid=re.search(r"Policy-ID:([0-9a-f]{32})",j).group(1)
    open(f"{n}.device-policy-id.txt","w").write(pid+"\n")
PY
for n in $ALL; do
  md decode --in $n.md1.txt > $n.decode.txt 2>&1 || echo "$n decode FAILED"
  md decode --json --in $n.md1.txt > $n.decode.json 2>&1
  md descriptor $(cat $n.md1.txt) > $n.descriptor.txt 2>&1 || echo "$n descriptor FAILED"
  { md address $(cat $n.md1.txt) --count 2; md address $(cat $n.md1.txt) --change --count 2; } > $n.md-addrs.txt 2>&1
  if diff <(awk '{print $3}' $n.device-addrs.txt) <(grep -oE 'bc1p[0-9a-z]{58}' $n.md-addrs.txt) >/dev/null; then echo "$n DEVICE==MD"; else echo "$n DEVICE!=MD"; fi
done
