#!/usr/bin/env python3
"""Minimal BIP-39 -> BIP-32 derivation (hashlib + ecdsa). Prints [fp/path]tprv and tpub.
usage: bip32.py <net main|test> <path m/48h/1h/0h/3h> <mnemonic words...>"""
import hashlib, hmac, sys, unicodedata
from ecdsa import SECP256k1
from ecdsa.ellipticcurve import PointJacobi
G = SECP256k1.generator; N = SECP256k1.order
B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
def b58c(b):
    b = b + hashlib.sha256(hashlib.sha256(b).digest()).digest()[:4]
    n = int.from_bytes(b, "big"); s = ""
    while n: n, r = divmod(n, 58); s = B58[r] + s
    return "1" * (len(b) - len(b.lstrip(b"\0"))) + s
def ser_p(P):
    return bytes([2 + (P.y() & 1)]) + P.x().to_bytes(32, "big")
def h160(b): return hashlib.new("ripemd160", hashlib.sha256(b).digest()).digest()
def derive(seed, path):
    I = hmac.new(b"Bitcoin seed", seed, hashlib.sha512).digest()
    k, c = int.from_bytes(I[:32], "big"), I[32:]
    mfp = h160(ser_p(G * k))[:4]; pfp = b"\0\0\0\0"; depth = 0; child = 0
    for part in path.split("/")[1:]:
        hard = part[-1] in "h'"; i = int(part.rstrip("h'")) + (0x80000000 if hard else 0)
        pub = ser_p(G * k)
        data = (b"\0" + k.to_bytes(32, "big") if hard else pub) + i.to_bytes(4, "big")
        I = hmac.new(c, data, hashlib.sha512).digest()
        pfp = h160(pub)[:4]; k = (int.from_bytes(I[:32], "big") + k) % N; c = I[32:]; depth += 1; child = i
    return mfp, depth, pfp, child, c, k
def main():
    net, path, words = sys.argv[1], sys.argv[2], " ".join(sys.argv[3:])
    seed = hashlib.pbkdf2_hmac("sha512", unicodedata.normalize("NFKD", words).encode(), b"mnemonic", 2048)
    mfp, depth, pfp, child, c, k = derive(seed, path)
    vpriv, vpub = (bytes.fromhex("0488ade4"), bytes.fromhex("0488b21e")) if net == "main" else (bytes.fromhex("04358394"), bytes.fromhex("043587cf"))
    body = bytes([depth]) + pfp + child.to_bytes(4, "big") + c
    origin = mfp.hex() + path[1:].replace("h", "'")
    print(f"[{origin}]" + b58c(vpriv + body + b"\0" + k.to_bytes(32, "big")))
    print(f"[{origin}]" + b58c(vpub + body + ser_p(G * k)))
main()
