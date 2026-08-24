#!/usr/bin/env python3
"""Generate the pinned mt1 test vectors — INDEPENDENTLY of mt-codec.

WHY THIS EXISTS AND WHY IT IS DELIBERATELY DUMB
================================================
`mt-codec` cannot be checked against vectors `mt-codec` produced: that is a
self-consistency check, and it passes just as happily on a wrong NUMS constant.
The spec's own named hazard (§12.22) is a constant copied from a sibling, and a
wrong constant yields chunks that are *self-consistent* and unreadable by every
other implementation — surfacing at recovery, indistinguishable from steel damage.

So bech32, the 55-bit header layout and the BCH polymod are re-implemented here
from BIP-93 and `SPEC_mt_v0_1.md` §10.13(a2), in Python, slower and dumber than
the crate on purpose. An independent derivation is the only kind that can
disagree.

VALIDATED BEFORE USE, and this is not optional: `--self-test` reproduces all 40
of `mk-codec`'s committed vectors (19 regular-code, 21 long-code). If this
implementation were wrong, every vector it emits would be wrong and the whole
defence would rest on a broken generator. Run it first.

ORDERING (plan §S0): this runs in `mnemonic-engrave` and completes BEFORE
`mt-codec`'s first commit. Regenerating the vectors means re-running THIS, never
the crate — re-deriving from the implementation is how a wrong vector launders
itself into looking correct.

Usage:
    gen-mt1-vectors.py --self-test
    gen-mt1-vectors.py --from-regtest <datadir> --rpcport <port> --out design/vectors
"""

import argparse
import hashlib
import json
import math
import subprocess
import sys
from pathlib import Path

# ── bech32, from BIP-173's charset ────────────────────────────────────────────
CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"

# ── BCH, from BIP-93. GEN_* are the code's generator polynomials; POLYMOD_INIT
#    is shared across the constellation's formats and cancels between create and
#    verify, so it is NOT what separates them (see hrp_expand below).
GEN_REGULAR = [
    0x19DC500CE73FDE210, 0x1BFAE00DEF77FE529, 0x1FBD920FFFE7BEE52,
    0x1739640BDEEE3FDAD, 0x07729A039CFC75F5A,
]
GEN_LONG = [
    0x3D59D273535EA62D897, 0x7A9BECB6361C6C51507, 0x543F9B7E6C38D8A2A0E,
    0x0C577EAECCF1990D13C, 0x1887F74F8DC71B10651,
]
POLYMOD_INIT = 0x23181B3
REGULAR = (GEN_REGULAR, 60, 0x0FFFFFFFFFFFFFFF, 13)
LONG = (GEN_LONG, 70, 0x3FFFFFFFFFFFFFFFFF, 15)

# ── the constellation's NUMS constants: top bits of SHA-256(<domain string>).
#    mt's is DERIVED here rather than pasted, so the constant and its domain
#    string cannot drift apart (the drift test, in code).
MT_DOMAIN = b"shibbolethnumstransaction"
MD_DOMAIN = b"shibbolethnums"
MK_DOMAIN = b"shibbolethnumskey"


def nums_const(domain: bytes, bits: int) -> int:
    """Top `bits` of SHA-256(domain), as the constellation defines it."""
    return int.from_bytes(hashlib.sha256(domain).digest(), "big") >> (256 - bits)


MT_REGULAR_CONST = nums_const(MT_DOMAIN, 65)
MD_REGULAR_CONST = nums_const(MD_DOMAIN, 65)
MK_REGULAR_CONST = nums_const(MK_DOMAIN, 65)
MK_LONG_CONST = nums_const(MK_DOMAIN, 75)

# mt1's wire format, SPEC §10.13(a2). Every field a whole number of 5-bit
# symbols — which is what lets a hand engraver elide the invariant prefix.
HRP = "mt"
VERSION = 1
W_VERSION, W_SETID, W_COUNT, W_INDEX = 5, 20, 15, 15
HEADER_BITS = W_VERSION + W_SETID + W_COUNT + W_INDEX          # 55
HEADER_SYMS = HEADER_BITS // 5                                  # 11
PAYLOAD_CEILING = 40                                            # §3b


def polymod(values, params):
    gen, shift, mask, _ = params
    residue = POLYMOD_INIT
    for v in values:
        b = residue >> shift
        residue = ((residue & mask) << 5) ^ v
        for i, g in enumerate(gen):
            if (b >> i) & 1:
                residue ^= g
    return residue


def hrp_expand(hrp: str):
    """BIP-173 HRP expansion. This is what separates the formats: the HRP is
    folded into the polymod, so a chunk under one HRP cannot verify under
    another whatever the constants are (proven by --self-test)."""
    return [ord(c) >> 5 for c in hrp] + [0] + [ord(c) & 31 for c in hrp]


def create_checksum(hrp, data, const, params=REGULAR):
    nsym = params[3]
    pm = polymod(hrp_expand(hrp) + list(data) + [0] * nsym, params) ^ const
    return [(pm >> (5 * (nsym - 1 - i))) & 0x1F for i in range(nsym)]


def verify(hrp, data_with_checksum, const, params=REGULAR):
    return polymod(hrp_expand(hrp) + list(data_with_checksum), params) == const


def bytes_to_symbols(payload: bytes):
    """8-bit bytes to 5-bit symbols, MSB-first, zero-padded to a boundary."""
    out, acc, n = [], 0, 0
    for byte in payload:
        acc = (acc << 8) | byte
        n += 8
        while n >= 5:
            n -= 5
            out.append((acc >> n) & 0x1F)
    if n:
        out.append((acc << (5 - n)) & 0x1F)
    return out


def symbols_to_bytes(symbols, nbytes):
    out, acc, n = bytearray(), 0, 0
    for v in symbols:
        acc = (acc << 5) | v
        n += 5
        while n >= 8:
            n -= 8
            out.append((acc >> n) & 0xFF)
    return bytes(out[:nbytes])


def chunking(payload_len):
    """§3b. 40 is the CEILING the count derives from, never a chunk's size."""
    count = max(1, math.ceil(payload_len / PAYLOAD_CEILING))
    return count, math.ceil(payload_len / count)


def encode(tx: bytes, txid_display: str):
    set_id = int(txid_display[:5], 16)          # top 20 bits of the DISPLAY form
    count, bpc = chunking(len(tx))
    strings = []
    for index in range(count):
        bits = ((VERSION << (W_SETID + W_COUNT + W_INDEX))
                | (set_id << (W_COUNT + W_INDEX))
                | ((count - 1) << W_INDEX)
                | index)
        header = [(bits >> (HEADER_BITS - 5 * (i + 1))) & 0x1F
                  for i in range(HEADER_SYMS)]
        data = header + bytes_to_symbols(tx[index * bpc:(index + 1) * bpc])
        full = data + create_checksum(HRP, data, MT_REGULAR_CONST)
        strings.append(HRP + "1" + "".join(CHARSET[v] for v in full))
    return strings, count, bpc, set_id


def parse_header(symbols):
    """Read version / set_id / count / index back out of the 11 header symbols."""
    bits = 0
    for v in symbols[:HEADER_SYMS]:
        bits = (bits << 5) | v
    index = bits & ((1 << W_INDEX) - 1)
    count = ((bits >> W_INDEX) & ((1 << W_COUNT) - 1)) + 1
    set_id = (bits >> (W_INDEX + W_COUNT)) & ((1 << W_SETID) - 1)
    version = bits >> (W_INDEX + W_COUNT + W_SETID)
    return version, set_id, count, index


def decode(strings, total_len):
    """Reassemble, so the generator proves its own round trip.

    Ordering comes from the HEADER's index field, never from the order the
    strings arrived in and never from sorting them — an operator hands them over
    in any order (§1.1a), and sorting lexicographically happens to almost work
    while being wrong, which is the worst kind of nearly-right.

    Each chunk is padded to a symbol boundary independently, so the byte count
    per chunk is known from the chunking rule rather than inferred from the
    symbol count — inferring it over-reads the final chunk's padding.
    """
    _, _, count, _ = parse_header([CHARSET.index(c) for c in strings[0][len(HRP) + 1:]])
    _, bpc = chunking(total_len)
    ordered = [None] * count
    for s in strings:
        syms = [CHARSET.index(c) for c in s[len(HRP) + 1:]]
        _, _, _, index = parse_header(syms)
        want = bpc if index < count - 1 else total_len - (count - 1) * bpc
        ordered[index] = symbols_to_bytes(syms[HEADER_SYMS:-REGULAR[3]], want)
    assert all(c is not None for c in ordered), "a chunk index is missing"
    return b"".join(ordered)


def elide(strings):
    """§3b: first string full, the rest carrying index + payload only."""
    prefix_len = (W_VERSION + W_SETID + W_COUNT) // 5      # 8 symbols
    body = len(HRP) + 1 + prefix_len
    return [strings[0]] + [s[body:] for s in strings[1:]]


# ── self-test: prove this implementation before trusting a byte of its output ─
def self_test(mk_vectors: Path) -> bool:
    ok = True
    # (1) the drift test, for all three formats — constants reproduce from their
    #     domain strings. Pinned literals, so a refactor cannot move both sides.
    for name, got, want in (
        ("MT", MT_REGULAR_CONST, 0x1A2FC877F9528D7C1),
        ("MD", MD_REGULAR_CONST, 0x0815C07747A3392E7),
        ("MK", MK_REGULAR_CONST, 0x1062435F91072FA5C),
        ("MK_LONG", MK_LONG_CONST, 0x41890D7E441CBE97273),
    ):
        good = got == want
        ok &= good
        print(f"  {'ok  ' if good else 'FAIL'} {name}_CONST from its domain string: 0x{got:x}")

    # (2) the constants are DISTINCT — the copy-paste tripwire
    distinct = len({MT_REGULAR_CONST, MD_REGULAR_CONST, MK_REGULAR_CONST}) == 3
    ok &= distinct
    print(f"  {'ok  ' if distinct else 'FAIL'} mt/md/mk constants are pairwise distinct")

    # (3) reproduce mk's committed corpus — validates bech32, hrp_expand, the
    #     polymod loop and BOTH generator polynomials against known-good data
    if mk_vectors.exists():
        v = json.loads(mk_vectors.read_text())
        strs = [s for vec in v["vectors"]
                for s in vec.get("expected", {}).get("strings", [])]
        r = sum(verify("mk", [CHARSET.index(c) for c in s.rpartition("1")[2]],
                       MK_REGULAR_CONST, REGULAR) for s in strs)
        l = sum(verify("mk", [CHARSET.index(c) for c in s.rpartition("1")[2]],
                       MK_LONG_CONST, LONG) for s in strs)
        good = (r + l) == len(strs) and len(strs) > 0
        ok &= good
        print(f"  {'ok  ' if good else 'FAIL'} mk corpus: {r + l}/{len(strs)} "
              f"verify ({r} regular, {l} long)")
    else:
        print(f"  skip  mk corpus not found at {mk_vectors}")

    # (4) HRP separation — proven, not asserted. A genuine mk1 string must fail
    #     under any other HRP, against every constant. This is why the spec's
    #     cross-format claim was retracted: the HRP already separates them.
    if mk_vectors.exists() and strs:
        s = next((x for x in strs if verify("mk",
                 [CHARSET.index(c) for c in x.rpartition("1")[2]],
                 MK_REGULAR_CONST, REGULAR)), None)
        if s:
            syms = [CHARSET.index(c) for c in s.rpartition("1")[2]]
            leaks = [h for h in ("md", "mt")
                     for k in (MK_REGULAR_CONST, MD_REGULAR_CONST, MT_REGULAR_CONST)
                     if verify(h, syms, k, REGULAR)]
            ok &= not leaks
            print(f"  {'ok  ' if not leaks else 'FAIL'} HRP separation: an mk1 string "
                  f"verifies under no other HRP/constant pair")
    return bool(ok)


def rpc(datadir, port, *args):
    p = subprocess.run(["bitcoin-cli", f"-datadir={datadir}", f"-rpcport={port}",
                        "-stdin"], input="\n".join(args) + "\n",
                       capture_output=True, text=True)
    if p.returncode != 0:
        raise RuntimeError(f"bitcoin-cli {args[0]} failed: {p.stderr.strip()}")
    return p.stdout.strip()


def build_vector(datadir, port, raw_hex, label, note):
    d = json.loads(rpc(datadir, port, "decoderawtransaction", raw_hex))
    tx = bytes.fromhex(raw_hex)
    strings, count, bpc, set_id = encode(tx, d["txid"])
    assert all(verify(HRP, [CHARSET.index(c) for c in s[3:]], MT_REGULAR_CONST)
               for s in strings), f"{label}: a generated string does not verify"
    assert decode(strings, len(tx)) == tx, f"{label}: round trip failed"
    # ORDER-INDEPENDENCE, asserted because a `sorted()` bug in this very function
    # passed the in-order round trip: §1.1a takes strings "in any order", so
    # in-order success proves nothing about the property that is specified.
    shuffled = list(reversed(strings))
    assert decode(shuffled, len(tx)) == tx, f"{label}: round trip is order-dependent"
    if len(strings) > 2:
        rotated = strings[3:] + strings[:3]
        assert decode(rotated, len(tx)) == tx, f"{label}: round trip is order-dependent"
    prefixes = {s[3:3 + 8] for s in strings}
    assert len(prefixes) == 1, f"{label}: invariant prefix is not invariant"
    dsha = hashlib.sha256(hashlib.sha256(tx).digest()).digest()[::-1].hex()
    assert dsha == d["hash"], f"{label}: double-SHA256 of the bytes is not the wtxid"
    return {
        "label": label, "note": note,
        "raw_hex": raw_hex, "size_bytes": len(tx),
        "txid": d["txid"], "wtxid": d["hash"],
        "txid_is_wtxid": d["txid"] == d["hash"],
        "locktime": d["locktime"],
        "inputs": [{"txid": i["txid"], "vout": i["vout"],
                    "witness_items": len(i.get("txinwitness", []))} for i in d["vin"]],
        "chunk_count": count, "bytes_per_chunk": bpc,
        "last_chunk_bytes": len(tx) - (count - 1) * bpc,
        "set_id": f"0x{set_id:05x}",
        "invariant_prefix": prefixes.pop(),
        "string_lengths": [len(s) for s in strings],
        "strings": strings,
        "strings_elided": elide(strings),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--from-regtest", metavar="DATADIR")
    ap.add_argument("--rpcport", default="18999")
    ap.add_argument("--tx", action="append", default=[],
                    help="LABEL=HEXFILE:note — repeatable")
    ap.add_argument("--out", default="design/vectors")
    ap.add_argument("--mk-vectors", default="/scratch/code/shibboleth/mnemonic-key/"
                    "crates/mk-codec/src/test_vectors/v0.1.json")
    a = ap.parse_args()

    print("SELF-TEST — this implementation is validated before it is trusted")
    if not self_test(Path(a.mk_vectors)):
        print("\nSELF-TEST FAILED — emitting no vectors.", file=sys.stderr)
        return 1
    print("  SELF-TEST PASSED\n")
    if a.self_test:
        return 0

    vectors = []
    for spec in a.tx:
        label, _, rest = spec.partition("=")
        path, _, note = rest.partition(":")
        vectors.append(build_vector(a.from_regtest, a.rpcport,
                                    Path(path).read_text().strip(), label, note))
        v = vectors[-1]
        print(f"  {label}: {v['size_bytes']} B -> {v['chunk_count']} chunks x "
              f"{v['bytes_per_chunk']} B, lengths {sorted(set(v['string_lengths']))}, "
              f"prefix {v['invariant_prefix']!r}")

    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    payload = {
        "format": "mt1", "wire_version": VERSION, "hrp": HRP,
        "header_bits": HEADER_BITS, "header_symbols": HEADER_SYMS,
        "mt_regular_const": f"0x{MT_REGULAR_CONST:x}",
        "nums_domain": MT_DOMAIN.decode(),
        "payload_ceiling_bytes": PAYLOAD_CEILING,
        "generator": "scripts/gen-mt1-vectors.py (mnemonic-engrave)",
        "vectors": vectors,
    }
    (out / "mt1_v1_vectors.json").write_text(json.dumps(payload, indent=2) + "\n")

    md = [f"# `mt1` v{VERSION} test vectors\n",
          "> **Generated by `scripts/gen-mt1-vectors.py` in `mnemonic-engrave`,",
          "> INDEPENDENTLY of `mt-codec`.** Regenerate with that script, never with",
          "> the crate — re-deriving from the implementation under test is how a wrong",
          "> vector launders itself into looking correct. The machine-readable form is",
          "> `mt1_v1_vectors.json`, and it is the file `mt-codec`'s SHA-256 pin covers.\n",
          f"Header: **{HEADER_BITS} bits = {HEADER_SYMS} symbols**, per-field aligned.",
          f"`MT_REGULAR_CONST = 0x{MT_REGULAR_CONST:x}`, derived from",
          f"`SHA-256(\"{MT_DOMAIN.decode()}\")`.\n"]
    for v in vectors:
        md += [f"## {v['label']} — {v['note']}\n",
               f"- **{v['size_bytes']} bytes** → {v['chunk_count']} chunks of "
               f"{v['bytes_per_chunk']} B (last {v['last_chunk_bytes']} B)",
               f"- `txid`  `{v['txid']}`",
               f"- `wtxid` `{v['wtxid']}` — **differs from the txid**, which is what pins",
               "  §1.1's `TX` row: double-SHA-256 of the engraved bytes is the *wtxid*",
               f"- `nLockTime` {v['locktime']}; set id `{v['set_id']}`",
               f"- invariant prefix `{v['invariant_prefix']}` (8 symbols, identical on every string)",
               f"- string lengths {v['string_lengths']}\n",
               "```"] + v["strings"] + ["```\n"]
    (out / "mt1_v1_vectors.md").write_text("\n".join(md))
    print(f"\n  wrote {out}/mt1_v1_vectors.json and .md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
