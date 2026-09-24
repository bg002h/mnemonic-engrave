#!/usr/bin/env bash
# Bitcoin Core RELEASE binaries, MAINNET, fully offline (no peers, no dns), watch-only import of
# each live descriptor, then wallet-derived receive/change addresses compared to the device's.
set -uo pipefail
cd "$(dirname "$0")"
ALL="kofn-nums kofn-liana tiered-nums tiered-liana kofn-nums-distinct kofn-liana-distinct tiered-nums-distinct tiered-liana-distinct"
for V in 29.4 31.1; do
  B=/scratch/code/shibboleth/.tmp/core-releases/$V/bitcoin-$V/bin
  DD=$PWD/core-main-$V; rm -rf $DD; mkdir -p $DD
  case $V in 29.4) RPC=18901;; 31.1) RPC=18911;; esac
  $B/bitcoind -datadir=$DD -chain=main -connect=0 -listen=0 -dnsseed=0 -fixedseeds=0 -rpcport=$RPC -server -daemon -disablewallet=0 -prune=0 >/dev/null
  CLI="$B/bitcoin-cli -datadir=$DD -chain=main -rpcport=$RPC"
  for i in $(seq 60); do $CLI getblockchaininfo >/dev/null 2>&1 && break; sleep 0.5; done
  echo "== Core $($CLI -version | head -1) chain=$($CLI getblockchaininfo | python3 -c 'import json,sys;d=json.load(sys.stdin);print(d["chain"],"blocks",d["blocks"])') peers=$($CLI getconnectioncount)"
  for n in $ALL; do
    D=$(grep '^tr(' $n.descriptor.txt)
    $CLI -named createwallet wallet_name=$n disable_private_keys=true blank=true >/dev/null
    R=$($CLI -rpcwallet=$n importdescriptors "[{\"desc\":\"$D\",\"timestamp\":\"now\",\"active\":true}]" 2>&1)
    OK=$(echo "$R" | python3 -c 'import json,sys; r=json.load(sys.stdin); print(all(x["success"] for x in r), [x.get("error",{}).get("message") for x in r if not x["success"]], [w for x in r for w in x.get("warnings",[])])' 2>&1)
    A=$( { $CLI -rpcwallet=$n getnewaddress "" bech32m; $CLI -rpcwallet=$n getnewaddress "" bech32m; $CLI -rpcwallet=$n getrawchangeaddress bech32m; $CLI -rpcwallet=$n getrawchangeaddress bech32m; } 2>&1 | tr '\n' ' ')
    DEV=$(awk '{printf "%s ", $3}' $n.device-addrs.txt)
    # also deriveaddresses on the single-path forms
    DA=$( { $CLI deriveaddresses "$D" "[0,1]" ; } 2>&1 | tr -d ' \n')
    [[ "$A" == "$DEV" ]] && M=MATCH || M=MISMATCH
    echo "$V $n import_ok=$OK wallet_addrs=$M"
    echo "   wallet: $A"; echo "   device: $DEV"; echo "   deriveaddresses(multipath,[0,1]): $DA"
  done
  $CLI stop >/dev/null; sleep 2
done
