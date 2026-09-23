#!/usr/bin/env python3
"""Coordinator-compat: measure the Core release boundary for tapscript miniscript.

Usage (from the mnemonic-engrave root):
  python3 design/evidence/coord-compat-core-boundary/probe.py <release-dir> <rpcport> > out.json

<release-dir> holds an extracted official tarball (bitcoin-<v>/bin/bitcoind).
Runs an OFFLINE MAINNET node (-connect=0 -listen=0) in a throwaway datadir: every
descriptor here carries mainnet xpubs and bc1 expectations, and a regtest node
would reject them (xpub version bytes), which would be a false refusal.

Descriptors probed:
  REP  = preset-kofn-recovery-tr from design/evidence/composer-fable-r0/fable-liana-shapes.json
         (NUMS internal key, leaves multi_a(2,...) and and_v(v:pk(..),older(26280))),
         expectations = that record's md_addr (md's own addresses).
  K1   = CONTROL-accept-kofn-recovery-flat from design/evidence/f449-stage2/liana-live-gate-expected.jsonl
         (Liana-unspendable internal xpub), expectations = Liana v15.0's receive/change 0..2.
  ALL15 = the 15 discriminating tr shapes, plus control preset-kofn-recovery-wsh (getdescriptorinfo + deriveaddresses on chain 0 only).
Each wallet import uses the per-chain /0/* and /1/* descriptors, so multipath
<0;1> support (a separate capability) cannot confound the tapscript verdict; the
multipath form is additionally probed with getdescriptorinfo only.
"""
import json, os, re, shutil, subprocess, sys, time

ROOT = os.getcwd()
rel, port = sys.argv[1], sys.argv[2]
bindir = [os.path.join(rel, d, "bin") for d in os.listdir(rel) if d.startswith("bitcoin-") and os.path.isdir(os.path.join(rel, d))][0]
dd = os.path.join(rel, "dd")
shutil.rmtree(dd, ignore_errors=True); os.makedirs(dd)
auth = [f"-datadir={dd}", f"-rpcport={port}", "-rpcuser=u", "-rpcpassword=p"]

def cli(*a):
    p = subprocess.run([f"{bindir}/bitcoin-cli", *auth, *a], capture_output=True, text=True)
    return p.returncode, (p.stdout if p.returncode == 0 else p.stderr).strip()

def ok_json(*a):
    rc, out = cli(*a)
    return (json.loads(out) if rc == 0 and out[:1] in "[{\"" else out) if rc == 0 else None, (None if rc == 0 else out)

shapes = {x["name"]: x for x in json.load(open(f"{ROOT}/design/evidence/composer-fable-r0/fable-liana-shapes.json"))}
k1 = json.loads(open(f"{ROOT}/design/evidence/f449-stage2/liana-live-gate-expected.jsonl").read().splitlines()[1])
assert k1["name"] == "CONTROL-accept-kofn-recovery-flat"
ALL15 = """preset-simple-timelocked-inheritance-tr preset-kofn-recovery-tr preset-tiered-recovery-tr
preset-hashlock-gated-tr preset-decaying-multisig-tr hashlock-gated-tr-hash160 mixed-lock-bases-tr
same-seed-two-paths-tr X09-tr-1of1-2of3older X10-tr-1of1-tworec X18-tr-inherit-older5
X19-tr-kofn-nums-older5 X20-tr-hashlock-known X23-tr-1of1-1of1older-h-spelling
X26-tr-2of3-1of1-unlocked-plus-rec""".split()

def strip(d): return re.sub(r"#[a-z0-9]{8}$", "", d)
def chain(multipath, c): return strip(multipath).replace("/<0;1>/*", f"/{c}/*")

rep = shapes["preset-kofn-recovery-tr"]
cases = {
    "REP": dict(multipath=rep["desc_md"], c0=rep["desc_chain0"], c1=rep["desc_chain1"],
                recv=rep["md_addr"]["0"], chg=rep["md_addr"]["1"]),
    "K1": dict(multipath=k1["liana_desc"], c0=k1["receive_desc"], c1=chain(k1["liana_desc"], 1),
               recv=k1["receive"], chg=k1["change"]),
}

subprocess.run([f"{bindir}/bitcoind", *auth, "-connect=0", "-listen=0", "-dnsseed=0", "-server", "-daemon"], check=True,
               capture_output=True)
res = {"release_dir": rel}
try:
    rc, out = cli("-rpcwait", "getnetworkinfo")
    res["version"] = json.loads(out)["subversion"]
    res["chain"] = json.loads(cli("getblockchaininfo")[1])["chain"]
    def checksummed(d):
        r, e = ok_json("getdescriptorinfo", strip(d))
        return (r["descriptor"], None) if r else (None, e)
    for name, c in cases.items():
        o = {}
        o["getdescriptorinfo_multipath"] = "ok" if checksummed(c["multipath"])[0] else checksummed(c["multipath"])[1]
        d0, e0 = checksummed(c["c0"]); d1, e1 = checksummed(c["c1"])
        o["getdescriptorinfo_c0"] = "ok" if d0 else e0
        o["getdescriptorinfo_c1"] = "ok" if d1 else e1
        if d0 and d1:
            r, e = ok_json("deriveaddresses", d0, "[0,2]"); o["derive_recv"] = r if r else e
            r, e = ok_json("deriveaddresses", d1, "[0,2]"); o["derive_chg"] = r if r else e
            w = f"w{name}"
            r, e = ok_json("-named", "createwallet", f"wallet_name={w}", "disable_private_keys=true", "blank=true", "descriptors=true")
            o["createwallet"] = "ok" if r else e
            req = json.dumps([{"desc": d0, "timestamp": "now", "active": True, "range": [0, 5], "internal": False},
                              {"desc": d1, "timestamp": "now", "active": True, "range": [0, 5], "internal": True}])
            r, e = ok_json(f"-rpcwallet={w}", "importdescriptors", req); o["importdescriptors"] = r if r else e
            o["getnewaddress"] = [cli(f"-rpcwallet={w}", "getnewaddress", "", "bech32m")[1] for _ in range(3)]
            o["getrawchangeaddress"] = [cli(f"-rpcwallet={w}", "getrawchangeaddress", "bech32m")[1] for _ in range(3)]
            imp_ok = isinstance(o["importdescriptors"], list) and all(x.get("success") for x in o["importdescriptors"])
            o["verdict_import"] = "ACCEPT" if imp_ok else "REFUSE"
            o["addr_match"] = (o["derive_recv"] == c["recv"] and o["derive_chg"] == c["chg"]
                               and o["getnewaddress"] == c["recv"] and o["getrawchangeaddress"] == c["chg"])
        else:
            o["verdict_import"] = "REFUSE (getdescriptorinfo)"; o["addr_match"] = None
        res[name] = o
    res["ALL15"] = {}
    # CONTROL (network-trap guard): same keys, wsh wrapper, which every release here should accept
    # with md's addresses -- proves a tr refusal is the tapscript capability, not xpub/network bytes.
    for n in ["preset-kofn-recovery-wsh"] + ALL15:
        s = shapes[n]; d, e = checksummed(s["desc_chain0"])
        if d:
            r, e2 = ok_json("deriveaddresses", d, "[0,2]")
            res["ALL15"][n] = "ACCEPT addr==md" if r == s["md_addr"]["0"] else f"ACCEPT addr MISMATCH {r or e2}"
        else:
            res["ALL15"][n] = "REFUSE: " + e.replace("\n", " ")[:160]
finally:
    cli("stop"); time.sleep(0.5)
print(json.dumps(res, indent=1))
