#!/usr/bin/env python3
"""Regtest SPEND test on Bitcoin Core 31.1 (release binary) for the 4 policy shapes.
Keys: BIP-39 vectors A/B/C at m/48'/1'/k'/3' (bip32.py), seated as the live device seated them.
For each wallet: (a) primary path spend; (b) recovery path rejected one block before the older()
lock matures, accepted at the first block it may; (c) key path: every spend is a SCRIPT-path spend."""
import json, subprocess, sys
def cli(*a, wallet=None):
    cmd = ["./cli"] + ([f"-rpcwallet={wallet}"] if wallet else []) + [str(x) for x in a]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode: raise RuntimeError(f"{' '.join(cmd[:3])}...: {r.stderr.strip()}")
    out = r.stdout.strip()
    try: return json.loads(out)
    except Exception: return out
ORI = {l.split("]")[0]+"]": l.split()[0] for l in open("keys.txt")}
A0, A1, B0, B1, C0 = list(ORI)[0], list(ORI)[1], list(ORI)[2], list(ORI)[3], list(ORI)[4]
PLAN = {  # slot order: kofn @0=A0 @1=B0 @2=C0 @3=A1 ; tiered @0=A0 @1=B0 @2=A1 @3=B1
  "kofn-nums":    {"prim": [A0, B0], "rec": [A1]},
  "kofn-liana":   {"prim": [A0, B0], "rec": [A1]},
  "tiered-nums":  {"prim": [A0, B0], "rec": [A1]},
  "tiered-liana": {"prim": [A0, B0], "rec": [A1]},
}
OLDER = 26280
log = {"core": cli("getnetworkinfo")["subversion"], "wallets": {}}
def desc_with(n, privs):
    d = subprocess.run(["python3", "mkdesc.py", f"{n}.descriptor.txt", ",".join(privs) if privs else "-"],
                       capture_output=True, text=True, check=True).stdout.strip()
    return cli("getdescriptorinfo", d)["checksum"] and d + "#" + cli("getdescriptorinfo", d)["checksum"]
def mkwallet(name, desc, priv):
    cli("-named", "createwallet", f"wallet_name={name}", f"disable_private_keys={'false' if priv else 'true'}", "blank=true")
    r = cli("importdescriptors", json.dumps([{"desc": desc, "timestamp": 0, "active": True}]), wallet=name)
    assert all(x["success"] for x in r), r
    return [w for x in r for w in x.get("warnings", [])]
cli("-named", "createwallet", "wallet_name=miner")
mine_addr = cli("getnewaddress", wallet="miner")
cli("generatetoaddress", 110, mine_addr)
def witness_kind(txhex):
    tx = cli("decoderawtransaction", txhex)
    w = tx["vin"][0]["txinwitness"]
    if len(w) == 1 and len(w[0]) in (128, 130): return "KEY-PATH", None, tx
    ctrl, script = w[-1], w[-2]
    return "script-path", {"control_block_prefix": ctrl[:2], "leaf_script": script, "stack_items": len(w),
                          "has_CSV": "b2" in script and script.endswith("b2") or " OP_CSV" in cli("decodescript", script)["asm"],
                          "leaf_asm": cli("decodescript", script)["asm"]}, tx
for n, p in PLAN.items():
    pub = desc_with(n, [])
    W = {"descriptor_public": pub}
    W["watch_warnings"] = mkwallet(f"w-{n}", pub, False)
    W["prim_warnings"] = mkwallet(f"p-{n}", desc_with(n, p["prim"]), True)
    W["rec_warnings"] = mkwallet(f"r-{n}", desc_with(n, p["rec"]), True)
    addrs = cli("deriveaddresses", pub, "[0,1]")[0]
    W["fund_addrs"] = addrs
    W["fund_txids"] = [cli("sendtoaddress", a, 1.0, wallet="miner") for a in addrs]
    log["wallets"][n] = W
cli("generatetoaddress", 1, mine_addr)
fund_height = cli("getblockcount")
log["fund_height"] = fund_height
def utxo(wallet, addr):
    u = [x for x in cli("listunspent", 0, 9999999, json.dumps([addr]), wallet=wallet)]
    assert len(u) == 1, (wallet, addr, u); return u[0]
def build(wallet, u, seq=None):
    inp = {"txid": u["txid"], "vout": u["vout"]}
    if seq is not None: inp["sequence"] = seq
    ps = cli("-named", "walletcreatefundedpsbt", f"inputs={json.dumps([inp])}", f"outputs={json.dumps([{mine_addr: u['amount']}])}",
             f"options={json.dumps({'add_inputs': False, 'subtractFeeFromOutputs': [0], 'fee_rate': 5})}", wallet=wallet)["psbt"]
    sp = cli("walletprocesspsbt", ps, "true", "DEFAULT", "true", "true", wallet=wallet)
    fin = cli("finalizepsbt", sp["psbt"])
    return sp, fin
# (a) primary path, now
for n, p in PLAN.items():
    W = log["wallets"][n]; u = utxo(f"p-{n}", W["fund_addrs"][0])
    sp, fin = build(f"p-{n}", u)
    W["primary"] = {"complete": fin["complete"]}
    kind, leaf, tx = witness_kind(fin["hex"])
    W["primary"].update({"witness": kind, "leaf": leaf, "txid": cli("sendrawtransaction", fin["hex"])})
# (b) recovery path: build with nSequence=OLDER, try at each boundary
rec = {}
for n, p in PLAN.items():
    W = log["wallets"][n]; u = utxo(f"r-{n}", W["fund_addrs"][1])
    sp, fin = build(f"r-{n}", u, OLDER)
    kind, leaf, tx = witness_kind(fin["hex"])
    W["recovery"] = {"complete": fin["complete"], "witness": kind, "leaf": leaf, "nSequence": tx["vin"][0]["sequence"], "tx_version": tx["version"]}
    rec[n] = fin["hex"]
    # also: can the PRIMARY wallet (no recovery key) produce a recovery spend?  and the recovery wallet a primary one?
cli("generatetoaddress", 1, mine_addr)  # confirm the primary spends
for n in PLAN:
    W = log["wallets"][n]; W["primary"]["confirmations"] = cli("gettransaction", W["primary"]["txid"], wallet=f"p-{n}")["confirmations"]
def try_send(h):
    try: return {"accepted": True, "txid": cli("sendrawtransaction", h)}
    except RuntimeError as e: return {"accepted": False, "error": str(e).split(": ", 1)[-1]}
# immediately (lock far from mature)
for n in PLAN: log["wallets"][n]["recovery"]["attempt_immediate"] = dict(try_send(rec[n]), tip=cli("getblockcount"))
# mine to tip = fund_height + OLDER - 2  (one block short of the first block that may include it)
target = fund_height + OLDER - 2
while cli("getblockcount") < target:
    cli("generatetoaddress", min(2000, target - cli("getblockcount")), mine_addr)
for n in PLAN: log["wallets"][n]["recovery"]["attempt_one_short"] = dict(try_send(rec[n]), tip=cli("getblockcount"))
cli("generatetoaddress", 1, mine_addr)
for n in PLAN: log["wallets"][n]["recovery"]["attempt_mature"] = dict(try_send(rec[n]), tip=cli("getblockcount"))
cli("generatetoaddress", 1, mine_addr)
for n in PLAN:
    R = log["wallets"][n]["recovery"]
    if R["attempt_mature"]["accepted"]:
        R["confirmations"] = cli("gettransaction", R["attempt_mature"]["txid"], wallet=f"r-{n}")["confirmations"]
# (c) key path: what private material does each wallet hold for the internal key?
for n in PLAN:
    W = log["wallets"][n]
    ld = cli("listdescriptors", "true", wallet=f"p-{n}")["descriptors"]
    W["private_descriptor_internal_key"] = [d["desc"].split(",")[0] for d in ld]
json.dump(log, open("spend-result.json", "w"), indent=1)
print(json.dumps(log, indent=1))
