#!/usr/bin/env bash
# F-449 stage 4 Task 9 Step 1 (plan F7): Bitcoin Core imports the kind-1 kofn-recovery descriptor and its
# getnewaddress 0..2 equal Liana's recorded receive addresses. Offline mainnet node, throwaway datadir.
# Run from the mnemonic-engrave root. B = a Bitcoin Core build dir.
set -euo pipefail
B=${B:-/scratch/code/bitcoin/build/bin}; DD=${DD:-/scratch/code/shibboleth/.tmp/f449s4-core-dd}
ev=design/evidence/f449-stage4
rm -rf "$DD"; mkdir -p "$DD"
cli() { "$B/bitcoin-cli" -datadir="$DD" -rpcport=18977 -rpcuser=u -rpcpassword=p "$@"; }
"$B/bitcoind" -datadir="$DD" -connect=0 -listen=0 -rpcport=18977 -rpcuser=u -rpcpassword=p -daemon -server
trap 'cli stop >/dev/null 2>&1 || true' EXIT
cli -rpcwait getblockcount >/dev/null
cli -named createwallet wallet_name=k1 disable_private_keys=true blank=true >/dev/null
D=$(sed -n 2p design/evidence/f449-stage2/liana-live-gate-expected.jsonl | python3 -c 'import json,sys; print(json.load(sys.stdin)["liana_desc"])')
DC=$(cli getdescriptorinfo "$D" | python3 -c 'import json,sys; print(json.load(sys.stdin)["descriptor"])')
{ "$B/bitcoind" -version | head -1; cli -rpcwallet=k1 importdescriptors '[{"desc":"'"$DC"'","timestamp":"now","active":true,"range":[0,5]}]'
  for i in 0 1 2; do cli -rpcwallet=k1 getnewaddress "" bech32m; done; } | tee $ev/core-kofn-import.txt
sed -n 2p design/evidence/f449-stage2/liana-live-gate-expected.jsonl | python3 -c 'import json,sys; print(*json.load(sys.stdin)["receive"], sep=chr(10))' > $ev/core-kofn-liana-receive.txt
if diff <(tail -3 $ev/core-kofn-import.txt) $ev/core-kofn-liana-receive.txt; then echo "CORE == LIANA (receive 0..2)"; else echo "CORE != LIANA"; exit 1; fi
