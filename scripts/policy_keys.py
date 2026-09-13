#!/usr/bin/env python3
"""Deterministic BIP-32 key material and network-independent address normalisation
for `policy-differential.py`.

Self-contained on purpose: the harness must not depend on any of the three
implementations it is comparing, or a shared library becomes a shared bug.  The
only third-party import is `ecdsa`, used for secp256k1 point arithmetic alone.

Two jobs:

1.  `derive_account(seed_index, script_type)` -> the xpub/tpub pair and master
    fingerprint for one cosigner at the composer's §4f default origin,
    `m/48'/0'/<seed_index>'/<script_type>'`.  Real derivation, not a placeholder:
    the fingerprints and depths that go on the command line are then genuine, so
    a refusal from `md` or from Core is a real finding rather than an artefact of
    fabricated key material.

2.  `normalize_address(addr)` -> a network-INDEPENDENT identity for an address.
    This is what lets the three legs be compared at all.  The device is
    mainnet-only, Core is on a throwaway regtest datadir, and an address string
    carries the network in its human-readable part (bc1/bcrt1) or its base58
    version byte.  The *script* does not: `bc1q<prog>` and `bcrt1q<prog>` encode
    the same witness program, and `3<h160>` and `2<h160>` the same script hash.
    Comparing the decoded payload compares the scriptPubKey, which is the thing
    that actually has to agree.
"""

import hashlib
import hmac
from typing import Dict, Tuple

from ecdsa.curves import SECP256k1
from ecdsa.ellipticcurve import Point

CURVE_ORDER = SECP256k1.order
CURVE_GEN = SECP256k1.generator

# BIP-32 serialisation version bytes.
VERSION_XPUB = 0x0488B21E  # mainnet public
VERSION_TPUB = 0x043587CF  # testnet/signet/regtest public

HARDENED = 0x80000000

# The composer's §4f default origin: 48'/0'/<account>'/<script_type>'.
# Script type by wrapper, read off `md compose --json`'s template_with_origins:
SCRIPT_TYPE_BY_WRAPPER = {
    "sh-wsh": 1,
    "wsh": 2,
    "sh": 2,
    "tr": 3,
}


# --------------------------------------------------------------------------
# base58check
# --------------------------------------------------------------------------

_B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def b58encode_check(payload: bytes) -> str:
    chk = hashlib.sha256(hashlib.sha256(payload).digest()).digest()[:4]
    raw = payload + chk
    n = int.from_bytes(raw, "big")
    out = ""
    while n > 0:
        n, r = divmod(n, 58)
        out = _B58[r] + out
    for byte in raw:
        if byte != 0:
            break
        out = "1" + out
    return out


def b58decode_check(s: str) -> bytes:
    n = 0
    for ch in s:
        idx = _B58.find(ch)
        if idx < 0:
            raise ValueError(f"bad base58 character {ch!r}")
        n = n * 58 + idx
    raw = n.to_bytes((n.bit_length() + 7) // 8, "big")
    pad = 0
    for ch in s:
        if ch != "1":
            break
        pad += 1
    raw = b"\x00" * pad + raw
    if len(raw) < 5:
        raise ValueError("base58 payload too short")
    payload, chk = raw[:-4], raw[-4:]
    if hashlib.sha256(hashlib.sha256(payload).digest()).digest()[:4] != chk:
        raise ValueError("bad base58 checksum")
    return payload


# --------------------------------------------------------------------------
# bech32 / bech32m (BIP-173, BIP-350) -- decode only
# --------------------------------------------------------------------------

_BECH32 = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
_BECH32M_CONST = 0x2BC830A3


def _bech32_polymod(values):
    gen = [0x3B6A57B2, 0x26508E6D, 0x1EA119FA, 0x3D4233DD, 0x2A1462B3]
    chk = 1
    for v in values:
        top = chk >> 25
        chk = ((chk & 0x1FFFFFF) << 5) ^ v
        for i in range(5):
            chk ^= gen[i] if ((top >> i) & 1) else 0
    return chk


def _bech32_hrp_expand(hrp):
    return [ord(c) >> 5 for c in hrp] + [0] + [ord(c) & 31 for c in hrp]


def _convertbits(data, frombits, tobits, pad=True):
    acc = 0
    bits = 0
    ret = []
    maxv = (1 << tobits) - 1
    max_acc = (1 << (frombits + tobits - 1)) - 1
    for value in data:
        if value < 0 or (value >> frombits):
            return None
        acc = ((acc << frombits) | value) & max_acc
        bits += frombits
        while bits >= tobits:
            bits -= tobits
            ret.append((acc >> bits) & maxv)
    if pad:
        if bits:
            ret.append((acc << (tobits - bits)) & maxv)
    elif bits >= frombits or ((acc << (tobits - bits)) & maxv):
        return None
    return ret


def bech32_decode(addr: str):
    """-> (hrp, witness_version, program_bytes) or None."""
    if any(ord(c) < 33 or ord(c) > 126 for c in addr):
        return None
    if addr.lower() != addr and addr.upper() != addr:
        return None
    addr = addr.lower()
    pos = addr.rfind("1")
    if pos < 1 or pos + 7 > len(addr) or len(addr) > 90:
        return None
    hrp = addr[:pos]
    if any(c not in _BECH32 for c in addr[pos + 1:]):
        return None
    data = [_BECH32.find(c) for c in addr[pos + 1:]]
    const = _bech32_polymod(_bech32_hrp_expand(hrp) + data)
    if const == 1:
        spec = "bech32"
    elif const == _BECH32M_CONST:
        spec = "bech32m"
    else:
        return None
    data = data[:-6]
    if not data:
        return None
    witver = data[0]
    prog = _convertbits(data[1:], 5, 8, False)
    if prog is None or len(prog) < 2 or len(prog) > 40:
        return None
    if witver > 16:
        return None
    if witver == 0 and len(prog) not in (20, 32):
        return None
    # BIP-350: v0 uses bech32, v1+ uses bech32m.
    if (witver == 0) != (spec == "bech32"):
        return None
    return hrp, witver, bytes(prog)


# Base58 address version bytes by network, for P2PKH and P2SH.
_P2PKH_VERSIONS = {0x00: "mainnet", 0x6F: "testlike"}
_P2SH_VERSIONS = {0x05: "mainnet", 0xC4: "testlike"}


def normalize_address(addr: str) -> str:
    """A network-INDEPENDENT identity for a bitcoin address.

    Returns e.g. `wit:0:<40 hex>` for P2WPKH/P2WSH, `wit:1:<64 hex>` for P2TR,
    `p2sh:<40 hex>`, `p2pkh:<40 hex>`.  Two addresses on different networks that
    lock the same scriptPubKey normalise to the same string; two that do not,
    do not.  Raises ValueError on anything unparseable, because a silently
    unnormalised address would compare equal to nothing and read as a finding.
    """
    addr = addr.strip()
    if not addr:
        raise ValueError("empty address")
    dec = bech32_decode(addr)
    if dec is not None:
        _hrp, witver, prog = dec
        return f"wit:{witver}:{prog.hex()}"
    payload = b58decode_check(addr)  # raises on a bad checksum
    if len(payload) != 21:
        raise ValueError(f"unexpected base58 payload length {len(payload)}")
    version, h160 = payload[0], payload[1:]
    if version in _P2PKH_VERSIONS:
        return f"p2pkh:{h160.hex()}"
    if version in _P2SH_VERSIONS:
        return f"p2sh:{h160.hex()}"
    raise ValueError(f"unknown base58 address version 0x{version:02x}")


# --------------------------------------------------------------------------
# BIP-32
# --------------------------------------------------------------------------

def _ser32(i: int) -> bytes:
    return i.to_bytes(4, "big")


def _point_to_pubkey(point) -> bytes:
    x = point.x()
    y = point.y()
    return (b"\x03" if y & 1 else b"\x02") + x.to_bytes(32, "big")


def _pubkey_of(k: int) -> bytes:
    return _point_to_pubkey(CURVE_GEN * k)


def _hash160(b: bytes) -> bytes:
    return hashlib.new("ripemd160", hashlib.sha256(b).digest()).digest()


def _fingerprint(pubkey: bytes) -> bytes:
    return _hash160(pubkey)[:4]


class ExtKey:
    """A BIP-32 extended PRIVATE key.  Private throughout: the harness only ever
    serialises the public half."""

    __slots__ = ("k", "chain", "depth", "parent_fp", "child_number")

    def __init__(self, k, chain, depth=0, parent_fp=b"\x00\x00\x00\x00", child_number=0):
        self.k = k
        self.chain = chain
        self.depth = depth
        self.parent_fp = parent_fp
        self.child_number = child_number

    @classmethod
    def from_seed(cls, seed: bytes) -> "ExtKey":
        I = hmac.new(b"Bitcoin seed", seed, hashlib.sha512).digest()
        k = int.from_bytes(I[:32], "big")
        if k == 0 or k >= CURVE_ORDER:
            raise ValueError("invalid master key from seed")
        return cls(k, I[32:])

    def pubkey(self) -> bytes:
        return _pubkey_of(self.k)

    def fingerprint(self) -> str:
        return _fingerprint(self.pubkey()).hex()

    def derive(self, index: int) -> "ExtKey":
        if index >= HARDENED:
            data = b"\x00" + self.k.to_bytes(32, "big") + _ser32(index)
        else:
            data = self.pubkey() + _ser32(index)
        I = hmac.new(self.chain, data, hashlib.sha512).digest()
        tweak = int.from_bytes(I[:32], "big")
        if tweak >= CURVE_ORDER:
            raise ValueError("derived tweak out of range")
        child = (tweak + self.k) % CURVE_ORDER
        if child == 0:
            raise ValueError("derived key is zero")
        return ExtKey(child, I[32:], self.depth + 1, _fingerprint(self.pubkey()), index)

    def derive_path(self, path) -> "ExtKey":
        node = self
        for index in path:
            node = node.derive(index)
        return node

    def serialize_public(self, version: int) -> str:
        raw = (
            version.to_bytes(4, "big")
            + bytes([self.depth])
            + self.parent_fp
            + _ser32(self.child_number)
            + self.chain
            + self.pubkey()
        )
        return b58encode_check(raw)


_ACCOUNT_CACHE: Dict[Tuple[int, int], dict] = {}


def _master_for(seed_index: int) -> ExtKey:
    # A fixed, documented domain-separated seed so the key pool is reproducible
    # from this file alone -- no committed key blob to drift out of sync.
    seed = hashlib.sha512(b"mnemonic-engrave/policy-differential/v1/" + _ser32(seed_index)).digest()
    return ExtKey.from_seed(seed)


def derive_account(seed_index: int, script_type: int) -> dict:
    """The cosigner at `m/48'/0'/<seed_index>'/<script_type>'`.

    -> {"xpub", "tpub", "fingerprint", "origin"} where `origin` is the
    unhardened-marker path string md and Core both want, e.g. 48'/0'/3'/2'.
    """
    key = (seed_index, script_type)
    cached = _ACCOUNT_CACHE.get(key)
    if cached is not None:
        return cached
    master = _master_for(seed_index)
    path = [48 + HARDENED, 0 + HARDENED, seed_index + HARDENED, script_type + HARDENED]
    node = master.derive_path(path)
    out = {
        "xpub": node.serialize_public(VERSION_XPUB),
        "tpub": node.serialize_public(VERSION_TPUB),
        "fingerprint": master.fingerprint(),
        "origin": f"48'/0'/{seed_index}'/{script_type}'",
    }
    _ACCOUNT_CACHE[key] = out
    return out


# --------------------------------------------------------------------------
# self-test: BIP-32 vector 1, plus normalisation round-trips
# --------------------------------------------------------------------------

def _selftest() -> None:
    # BIP-32 test vector 1, seed 000102030405060708090a0b0c0d0e0f.
    m = ExtKey.from_seed(bytes.fromhex("000102030405060708090a0b0c0d0e0f"))
    assert m.serialize_public(VERSION_XPUB) == (
        "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8"
    ), "BIP-32 vector 1 m/ failed"
    c = m.derive(0 + HARDENED)
    assert c.serialize_public(VERSION_XPUB) == (
        "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw"
    ), "BIP-32 vector 1 m/0' failed"
    c = c.derive(1)
    assert c.serialize_public(VERSION_XPUB) == (
        "xpub6ASuArnXKPbfEwhqN6e3mwBcDTgzisQN1wXN9BJcM47sSikHjJf3UFHKkNAWbWMiGj7Wf5uMash7SyYq527Hqck2AxYysAA7xmALppuCkwQ"
    ), "BIP-32 vector 1 m/0'/1 failed"

    # base58 round trip
    assert b58decode_check(b58encode_check(b"\x05" + b"\x11" * 20)) == b"\x05" + b"\x11" * 20

    # Network independence is the whole point: the same program on two networks
    # must normalise identically, and a different program must not.
    assert normalize_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4") == \
        normalize_address("bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080")
    assert normalize_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4") == \
        "wit:0:751e76e8199196d454941c45d1b3a323f1433bd6"
    # BIP-350 vector: a v1 (bech32m) program.
    assert normalize_address("bc1pw508d6qejxtdg4y5r3zarvary0c5xw7kw508d6qejxtdg4y5r3zarvary0c5xw7kt5nd6y") == \
        "wit:1:751e76e8199196d454941c45d1b3a323f1433bd6751e76e8199196d454941c45d1b3a323f1433bd6"
    # A v0 program in bech32m must be refused, not silently accepted.
    try:
        normalize_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kemeawh")  # wrong checksum spec
        raise AssertionError("bech32m-encoded v0 accepted")
    except ValueError:
        pass
    # P2SH across networks.
    assert normalize_address(b58encode_check(b"\x05" + b"\x22" * 20)) == \
        normalize_address(b58encode_check(b"\xc4" + b"\x22" * 20))
    print("policy_keys selftest OK")


if __name__ == "__main__":
    _selftest()
    for i in range(3):
        acct = derive_account(i, 2)
        print(i, acct["fingerprint"], acct["origin"], acct["xpub"][:24], acct["tpub"][:24])
