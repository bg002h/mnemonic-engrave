#!/usr/bin/env python3
"""Rewrite md's regtest descriptor: each leaf [origin]xpub... -> [origin]tpub (or tprv for slots in PRIV).
md emits leaf keys with MAINNET version bytes even under --network regtest; the key+chaincode are the same,
checked here by decoding both before substituting. usage: mkdesc.py <md-descriptor.txt> <priv-origins-comma|-> """
import sys, re, hashlib
B58="123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
def dec(s):
    n=0
    for ch in s: n=n*58+B58.index(ch)
    b=n.to_bytes(82,"big"); assert hashlib.sha256(hashlib.sha256(b[:-4]).digest()).digest()[:4]==b[-4:]; return b[:-4]
keys={}
for line in open("keys.txt"):
    pr,pu=line.split(); o=pr.split("]")[0]+"]"; keys[o]=(pr.split("]")[1],pu.split("]")[1])
desc=[l for l in open(sys.argv[1]) if l.startswith("tr(")][0].strip().split("#")[0]
priv=set(sys.argv[2].split(",")) if sys.argv[2]!="-" else set()
def sub(m):
    o, x = m.group(1), m.group(2)
    tprv, tpub = keys[o]
    a, b = dec(x), dec(tpub)
    assert a[4]==b[4] and a[9:]==b[9:], f"key/chaincode mismatch for {o}"
    return o + (tprv if o in priv else tpub)
out=re.sub(r"(\[[0-9a-f]{8}/[0-9'/]+\])(xpub[1-9A-HJ-NP-Za-km-z]+)", sub, desc)
print(out)
