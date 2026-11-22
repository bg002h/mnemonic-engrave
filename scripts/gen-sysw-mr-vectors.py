#!/usr/bin/env python3
"""Generate the Class::Mr / kit-door vectors of SPEC_systemwide_payloads §14.

WHY A PYTHON GENERATOR. The vectors are the contract three implementations
inherit (me's Rust, the fork's Go port, mr-codec's sitting writer). Vectors
produced by the first implementation would only prove it agrees with itself.
So the mr1 string layer (BIP 93's ms32 polymod, mr's own residue) and the
37-bit chunk framing are re-implemented here from mnemonic-refugium
design/SPEC_refugium_mr1.md §3 ("String layer and checksum", "Chunked form"),
deliberately small and slow. `--self-test` checks the implementation against
every real mr1 card this script knows before anything is emitted.

The ORACLE for the mr1 format itself is `mr` (mnemonic-refugium), not this
script: scripts/check-sysw-mr-vectors.sh runs `mr decode` on every vector and
compares its verdict with the `mr_decode` field written here.

WHAT THIS DOES NOT COVER (stated so a reviewer knows): md1/mk1 confirmation
([mdmk-decode]) is not re-implemented; the kit vectors take md1/mk1 strings
from `mk encode` / `mr create` output and the check script confirms them
with `me sysw show`. Plate counts (M in "N cards, M plates") are geometry and
are not in these vectors (the fork's packer owns them, contract S5).

Every string here is from a SIGNET card made from BIP-39 test-vector seeds,
or is constructed; no mainnet key and no secret appears.

Usage:
    gen-sysw-mr-vectors.py --self-test
    gen-sysw-mr-vectors.py --out design/vectors/sysw_mr_v1_vectors.json
"""
import hashlib
import json
import sys

ALPHA = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
GEN = [0x19DC500CE73FDE210, 0x1BFAE00DEF77FE529, 0x1FBD920FFFE7BEE52,
       0x1739640BDEEE3FDAD, 0x07729A039CFC75F5A]
INIT = 0x23181B3
MR_CONST = 0x085D21404AEE15059
CHECKSUM = 13
MIN_L, MAX_L = 14, 93          # symbols after "mr1", checksum included
HEADER_BITS = 37
CHUNK_BYTES_MAX = 45

# ---- the signet sources (mr 0.1.0-alpha.1 2c67cebf, BIP-39 test-vector seeds)
A1 = "mr1q2qyyter2scsqzpl40veqyppqqxpxpgnxjakqqn88sqhs32rd4eysmyjsmg0770kkufgjegg"
A2 = ["mr1lg2c0qsqzsppz7g65xyqps0rl40veqpqcgqpsfs6pz9qu2fc4ycmcsxm3ajxhutn",
      "mr1lg2c0qstf0dsqyeeuqxygz5xmtjgsmyjsm2v8fxf6wfzvyeaqpj52ckkns06uf"]
B2 = ["mr1lj3x5qsqzszpz7g65x9q3s0rl40veqpqcgqpsfs6pz9qu2fc4ychnw0zrxgja86j",
      "mr1lj3x5qstf0dsqyeeuqxygz5xmtjgsmyjsm2v8fxf6wfzvyeaqknhs48cfxu72m"]
MK = [  # four mk1 plates, two strings each, from the `mk encode` lines of A2's create
    ["mk1qppc9xpqqsqm3vrdep2rd4eylcpue0vssqygrqyqsqygpqyqsqyqgdv8eag9v5c296qfpc56e6xc4ed689t8w76l8hpsgymuz8nr44036mz6",
     "mk1qppc9xpplucp0dtenpf4xrp3mw9xprqrg9euqv046sd377nklhzwuf0rsq04pxqr0ncmucqczcdwnac6uekquzn8cc0xlmt426fyx0uvz"],
    ["mk1qpk4mtpqqsqm3vrdeqxey5x6lcpue0vssqygrqyqsqygrqyqsqyqgdv8eaq3fjne26pjwjyhtpdpu2pnuy2qjvpakcqrgxn4qsa8a092r3u5",
     "mk1qpk4mtppmtf325qnac9cxy99x2zqgdszpve25uz55thheer40jw8rn9ask8xpnkvscsvj3hf7g5ws29nmyqt2a43l5uzaxkxagmr8mjvg"],
    ["mk1qpumhjpqqsqm3vrdepsaye8flcpue0vssqygrqyqsqyg9qyqsqyqgdv8eaqv3sh2w8clkrlywgwep524ryw7ry0788h3c62lht8yc97zuj0f",
     "mk1qpumhjppc04r2g49rl4x2hdcldcwqnqrrvluur9spfxscytkjlks79u7kthvjxwkyqys68vx5w63mssz6hp7dmedq5uythh2tplszm8et"],
    ["mk1qprduapqqsqm3vrdeqjycfn6lcpue0vssqygrqyqsqyg8qyqsqyqgdv8elctket6rgys0yye8c7dv4af8qkj7qs3z23g6gx4fsrynsykuq9w",
     "mk1qprduappkn4ru8adws2dxu4resk0njszjqtw5dgm3pwcutcdh4rn3rxq3tfher5dannc78wk3l29gzkzu8qpk7wk0qea037ljtsn276h8"],
]
MD = ["md1fgl68qs9qel40veprs874anyywx0atmxggu50atmxgguhqqxpxrgyg5qmkljvvahhzr8z",
      "md1fgl68qst3f8z5nxj7mqqfnncqvgs9gdkhy3pkf9pk5cwjvn5ujycfn6qhzc5yfl9s9ajk"]
TEXT = "text:68656c6c6f"   # "hello", ClassFreeText

# mainnet cards from mnemonic-refugium crates/mr-cli/tests/fj1_base/cards.json,
# used ONLY by --self-test (never emitted as vectors)
SELFTEST_CARDS = [
    ["mr1lrnx4psqz3pyzzqycnppqq5qygs56dnsssqyg2xvyjr2ah40veqpqcgqpsfsxpzqmkedyluxu2sst",
     "mr1lrnx4pstye5hhqqqrx4qrzyzkl4x7dvelvq500gxft9m26w4qsppggqpsfsxzyqs3kq5v5mqjkh9w",
     "mr1lrnx4ps5ufnfwhqqqrx4qrrfpt06n0xnx0mq9r4aqe9vh8jz3hakfgdcfxeth42sfr4u9l0p97s5y",
     "mr1lrnx4ps7uhzpqsqrqnqvzz9xd9ecqqqe4gqhe3jz3hakxsmsjdjjkn4gyqqpltsyuan9vmnc"],
    ["mr1l8x6qqsqrs3zzzqytczrqqsqgssnszpcgc027ejqpqggqpsfs2ye5hwqqqxd2q9uykwxsmaw906sm5zdcj",
     "mr1l8x6qqsd3n0lh3v3wjpqsqrqnq5x2v6ttsqqpn2s9g97vt8rd3ndlmckghv8xz4mgsqq2tap3a9mccvz"],
    ["mr1q2qyqggqpz06hkvsyppqqxpxpgnxjacqqqe4gqhsjecmvvmlautyt5he0cl8ja2ql29"],
    A2, B2, [A1],
]


def polymod(vals):
    r = INIT
    for v in vals:
        b = r >> 60
        r = ((r & 0x0FFFFFFFFFFFFFFF) << 5) ^ v
        for i in range(5):
            if (b >> i) & 1:
                r ^= GEN[i]
    return r


def hrp_expand(h):
    return [ord(c) >> 5 for c in h] + [0] + [ord(c) & 31 for c in h]


def checksum(data):
    r = polymod(hrp_expand("mr") + data + [0] * CHECKSUM) ^ MR_CONST
    return [(r >> (5 * (CHECKSUM - 1 - i))) & 31 for i in range(CHECKSUM)]


def render(data):
    return "mr1" + "".join(ALPHA[v] for v in data + checksum(data))


def syms(s):
    return [ALPHA.index(c) for c in s[3:]]


def bits_of(data_syms):
    return "".join(format(v, "05b") for v in data_syms)


def syms_of(bits):
    assert len(bits) % 5 == 0
    return [int(bits[i:i + 5], 2) for i in range(0, len(bits), 5)]


# ---- the string rule (§14b), in its normative order --------------------------
def string_reason(rec):
    """None if the record is Class::Mr, else the reason code. rec is trimmed."""
    if any("A" <= c <= "Z" for c in rec):
        return "MR_CASE"
    body = rec[3:]
    if any(c not in ALPHA for c in body):
        return "MR_CHARACTER"
    if not (MIN_L <= len(body) <= MAX_L):
        return "MR_LENGTH"
    if polymod(hrp_expand("mr") + syms(rec)) != MR_CONST:
        return "MR_CHECKSUM"
    first = body[0]
    if first in "ua7":
        return "MR_RESERVED_SYMBOL"
    if first == "l":
        d = len(body) - CHECKSUM
        if d < 8:
            return "MR_CHUNK_SHORT"
        b = bits_of(syms(rec)[:d])[HEADER_BITS:]
        pad = len(b) % 8
        if pad > 4 or "1" in b[len(b) - pad:]:
            return "MR_CHUNK_PADDING"
    return None


def is_candidate(rec):
    return rec[:3].lower() == "mr1"


def chunk(rec):
    """(csid, count, index, slice bytes) of a Class::Mr chunk string."""
    s = syms(rec)[:-CHECKSUM]
    b = bits_of(s)
    csid = int(b[5:25], 2)
    count = int(b[25:31], 2) + 1
    index = int(b[31:37], 2)
    rest = b[HEADER_BITS:]
    n = len(rest) // 8
    sl = bytes(int(rest[8 * i:8 * i + 8], 2) for i in range(n))
    return csid, count, index, sl


def make_chunk(csid, count, index, sl):
    b = "11111" + format(csid, "020b") + format(count - 1, "06b") + format(index, "06b")
    b += "".join(format(x, "08b") for x in sl)
    b += "0" * ((-len(b)) % 5)
    return render(syms_of(b))


def csid_of(content):
    h = hashlib.sha256(b"mr1/chunks" + content).digest()
    return int.from_bytes(h[:3], "big") >> 4


def slice_lengths(total):
    count = max(2, -(-total // CHUNK_BYTES_MAX))
    each = -(-total // count)
    return count, [each] * (count - 1) + [total - each * (count - 1)]


# ---- the set rule (§14c) and the kit rule (§14d) ----------------------------
def is_chunk(rec):
    return rec[3] == "l"


def set_problem(records):
    """First (code, detail) of the mr1 chunk-set rule over a payload, or None.

    Assumes every mr1 candidate already passed the string rule."""
    groups = {}
    order = []
    for i, r in enumerate(records):
        if is_candidate(r) and is_chunk(r):
            csid = chunk(r)[0]
            if csid not in groups:
                groups[csid] = []
                order.append(csid)
            groups[csid].append(i)
    for csid in order:
        idx = groups[csid]
        first = records[idx[0]][:8]
        heads = [chunk(records[i]) for i in idx]
        c1 = heads[0][1]
        for i, h in zip(idx, heads):
            if h[1] != c1:
                return ("SET_COUNT", {"begins": first, "record": i, "count": h[1], "first_count": c1})
        for i, h in zip(idx, heads):
            if h[2] >= c1:
                return ("SET_INDEX", {"begins": first, "record": i, "index": h[2], "count": c1})
        seen = {}
        for i, h in zip(idx, heads):
            if h[2] in seen:
                return ("SET_DUPLICATE", {"begins": first, "index": h[2], "records": [seen[h[2]], i]})
            seen[h[2]] = i
        for k in range(c1):
            if k not in seen:
                return ("SET_MISSING", {"begins": first, "index": k, "count": c1})
        by_index = sorted(heads, key=lambda h: h[2])
        content = b"".join(h[3] for h in by_index)
        want_count, want = slice_lengths(len(content))
        if c1 != want_count or [len(h[3]) for h in by_index] != want or len(content) < 2:
            return ("SET_SLICES", {"begins": first})
        if csid_of(content) != csid:
            return ("SET_ID", {"begins": first})
        lo, hi = idx[0], idx[-1]
        for j in range(lo, hi + 1):
            if j not in idx:
                return ("CARD_SPLIT", {"kind": "mr1", "begins": first, "record": j})
    return None


# ---- self-test ------------------------------------------------------------------
def self_test():
    n = 0
    for card in SELFTEST_CARDS:
        for s in card:
            assert string_reason(s) is None, (s, string_reason(s))
            n += 1
        if len(card) > 1:
            assert set_problem(card) is None, (card, set_problem(card))
            heads = [chunk(s) for s in card]
            content = b"".join(h[3] for h in sorted(heads, key=lambda h: h[2]))
            assert csid_of(content) == heads[0][0]
            assert slice_lengths(len(content))[1] == [len(h[3]) for h in sorted(heads, key=lambda h: h[2])]
            # re-encoding the parsed chunks reproduces the strings byte for byte
            for s, h in zip(card, heads):
                assert make_chunk(*h) == s, s
    # the residue constant is mr's own: top 65 bits of SHA-256("shibbolethnumsrefugium")
    d = hashlib.sha256(b"shibbolethnumsrefugium").digest()
    assert int.from_bytes(d[:16], "big") >> (128 - 65) == MR_CONST
    print(f"self-test OK: {n} strings, {sum(len(c) > 1 for c in SELFTEST_CARDS)} chunk sets")


# ---- constructions ------------------------------------------------------------
def flip_case_at(s, k):
    return s[:k] + s[k].upper() + s[k + 1:]


def substitute(s, k):
    c = s[k]
    alt = ALPHA[(ALPHA.index(c) + 1) % 32]
    return s[:k] + alt + s[k + 1:]


# The DECIDED outcome of every string and set vector, written by hand from
# §14 and asserted against the rule implemented above, so a mutant of the rule
# (a length bound off by one, a dropped check) fails the generator instead of
# rewriting the vectors to agree with itself.
DECIDED = {
    "single-card": None, "chunk-0": None, "chunk-1": None,
    "upper": "MR_CASE", "mixed": "MR_CASE", "character-b": "MR_CHARACTER",
    "checksum": "MR_CHECKSUM", "too-long": "MR_LENGTH", "too-short": "MR_LENGTH",
    "reserved-u": "MR_RESERVED_SYMBOL", "chunk-short": "MR_CHUNK_SHORT",
    "chunk-pad-long": "MR_CHUNK_PADDING", "chunk-pad-nonzero": "MR_CHUNK_PADDING",
    "complete": None, "complete-reversed": None, "missing": "SET_MISSING",
    "duplicate": "SET_DUPLICATE", "count": "SET_COUNT", "index": "SET_INDEX",
    "id": "SET_ID", "slices": "SET_SLICES", "count-formula": "SET_SLICES",
    "split": "CARD_SPLIT", "mixed-cards": "SET_MISSING", "framing-only": None,
}


def vectors():
    c0, c1 = A2
    h0, h1 = chunk(c0), chunk(c1)
    out = {"version": 1,
           "spec": "SPEC_systemwide_payloads.md §14 (S1, 2026-11-22)",
           "generator": "scripts/gen-sysw-mr-vectors.py",
           "network_of_cards": "signet",
           "string_vectors": [], "set_vectors": [], "kit_vectors": []}
    sv = out["string_vectors"]

    def s(name, rec, why, mr_verdict, mr_says, mr_with_partner=False):
        r = string_reason(rec)
        assert r == DECIDED[name], (name, r, DECIDED[name])
        args = [rec, c0] if mr_with_partner else [rec]
        sv.append({"name": name, "record": rec, "class": "Mr" if r is None else "Unknown",
                   "reason": r, "why": why,
                   "mr": {"args": args, "verdict": mr_verdict, "says": mr_says}})
    s("single-card", A1, "a signet single-string card (owner-heir), 75 characters", "accept", "shape 0")
    s("chunk-0", c0, "chunk 0 of a signet 2-chunk card (2of3-heir)", "refuse", "chunk 1 is missing")
    s("chunk-1", c1, "chunk 1 of the same card", "refuse", "chunk 0 is missing")
    s("upper", A1.upper(), "all upper case: mr reads it, the container refuses it", "accept", "shape 0")
    s("mixed", flip_case_at(A1, 10), "one capital letter", "refuse", "mixes upper and lower case")
    s("character-b", A1[:20] + "b" + A1[21:], "'b' is not in the mr1 alphabet", "refuse", "not in the codex32 alphabet")
    s("checksum", substitute(A1, 30), "one substitution", "refuse", "checksum does not verify")
    long_data = syms(A1)[:-CHECKSUM]
    long_data = (long_data * 2)[:81]
    s("too-long", render(long_data), "94 symbols after mr1, checksum valid", "refuse", "94 symbols after mr1")
    s("too-short", render([]), "13 symbols after mr1 (no data), checksum valid", "refuse", "card header ends inside")
    s("reserved-u", render([ALPHA.index("u")] + syms(A1)[1:-CHECKSUM]), "first data symbol u (card version 7)", "refuse", "card version 7 is reserved")
    s("chunk-short", render([31, 0, 0, 0, 0, 0, 0]), "a chunk with 7 data symbols, checksum valid", "refuse", "too few for its 37-bit chunk header", True)
    d1 = syms(c1)[:-CHECKSUM] + [0]
    s("chunk-pad-long", render(d1), "chunk 1 plus one zero symbol: 5 padding bits", "refuse", "ends in 5 bits", True)
    d0 = syms(c0)[:-CHECKSUM]
    b0 = bits_of(d0)
    pad0 = (len(b0) - HEADER_BITS) % 8
    assert pad0 >= 1, pad0
    b0n = b0[:-1] + "1"
    s("chunk-pad-nonzero", render(syms_of(b0n)), f"chunk 0 with its last of {pad0} padding bits set", "refuse", "ends in 2 bits", True)

    st = out["set_vectors"]

    def setv(name, recs, why, mr_verdict, mr_says):
        bad = [string_reason(r) for r in recs if is_candidate(r)]
        assert all(b is None for b in bad), (name, bad)
        p = set_problem(recs)
        assert (None if p is None else p[0]) == DECIDED[name], (name, p, DECIDED[name])
        st.append({"name": name, "records": recs,
                   "pack": "ok" if p is None else "refused",
                   "reason": None if p is None else p[0],
                   "detail": None if p is None else p[1], "why": why,
                   "mr": {"args": recs, "verdict": mr_verdict, "says": mr_says}})
    setv("complete", [c0, c1], "the 2-chunk card, in order", "accept", "shape 0")
    setv("complete-reversed", [c1, c0], "the same card, chunk 1 first: order inside a card is free", "accept", "shape 0")
    setv("missing", [c0], "chunk 1 of 2 missing", "refuse", "chunk 1 is missing")
    setv("duplicate", [c0, c0, c1], "chunk 0 twice, even byte-identical", "refuse", "chunk 0 is given twice")
    setv("count", [c0, make_chunk(h1[0], 3, h1[2], h1[3])], "chunk 1 re-framed to say 3 chunks", "refuse", "says the card has 3 chunks")
    setv("index", [c0, make_chunk(h1[0], 2, 2, h1[3])], "chunk 1 re-framed as index 2 of 2", "refuse", "gives index 2 in a set of 2")
    bad = bytes([h1[3][0] ^ 1]) + h1[3][1:]
    setv("id", [c0, make_chunk(h1[0], 2, 1, bad)], "chunk 1 with one content bit flipped (checksum valid)", "refuse", "these chunks are not one card")
    content = bytes(range(50))
    cs = csid_of(content)
    setv("slices", [make_chunk(cs, 2, 0, content[:30]), make_chunk(cs, 2, 1, content[30:])],
         "50 bytes sliced 30+20 instead of 25+25; id correct", "refuse", "[30, 20] bytes")
    setv("count-formula", [make_chunk(cs, 3, 0, content[:17]), make_chunk(cs, 3, 1, content[17:34]),
                           make_chunk(cs, 3, 2, content[34:])],
         "50 bytes in 3 chunks where the rule gives 2; id correct", "refuse", "[17, 17, 16] bytes")
    setv("split", [c0, MD[0], MD[1], c1], "an md1 card between the two chunks", "refuse", "is an md1 string")
    setv("mixed-cards", [A2[0], B2[1]], "chunk 0 of one card and chunk 1 of another", "refuse", "is of the chunk set that begins")
    content3 = bytes(range(100))
    cs3 = csid_of(content3)
    _, ls3 = slice_lengths(100)
    synth, at = [], 0
    for i, n in enumerate(ls3):
        synth.append(make_chunk(cs3, len(ls3), i, content3[at:at + n]))
        at += n
    setv("framing-only", synth, "3 chunks of 100 bytes that are not a card: the container "
         "does not decode cards (STOP-1); mr refuses it", "refuse", "U field")

    kv = out["kit_vectors"]
    mk_all = [x for p in MK for x in p]

    def kit(name, recs, verdict, reason, detail, cards, why):
        kv.append({"name": name, "records": recs, "kit": verdict, "reason": reason,
                   "detail": detail, "cards": cards, "why": why})

    def card(kind, strs):
        return {"kind": kind, "strings": len(strs), "begins": strs[0][:8], "ends": strs[-1][-4:]}
    kit("kit-2of3", [c0, c1] + mk_all + MD, "eligible", None, None,
        [card("mr1", A2)] + [card("mk1", p) for p in MK] + [card("md1", MD)],
        "the 2of3-heir card, its four mk1 plates and the (0,0) md1 card: 6 cards")
    kit("kit-single", [A1], "eligible", None, None, [card("mr1", [A1])], "one single-string card")
    kit("kit-text", [c0, c1, TEXT], "refused", "KIT_OTHER_RECORD", {"record": 2, "class": "free text"},
        None, "free text beside the card")
    kit("kit-mk1-alone", [c0, c1, MK[0][0]], "refused", "KIT_UNCONFIRMED_MDMK", {"record": 2},
        None, "one string of a 2-string mk1 plate")
    kit("kit-mk1-split", [c0, c1, MK[0][0], MK[1][0], MK[0][1], MK[1][1]], "refused", "CARD_SPLIT",
        {"kind": "mk1", "begins": MK[0][0][:8], "record": 3}, None,
        "two mk1 plates interleaved")
    kit("kit-upper", [c0.upper(), c1.upper()], "refused", "MR_CASE", {"record": 0}, None,
        "the card in upper case: the string rule runs first")
    kit("kit-missing", [c0] + mk_all, "refused", "SET_MISSING",
        {"begins": c0[:8], "index": 1, "count": 2}, None, "the card's chunk 1 left out")
    kit("no-mr1", mk_all, "none", None, None, None, "no mr1 record: no kit door, Engrave Bundle as today")
    return out


def main(argv):
    if "--self-test" in argv:
        self_test()
        return 0
    if "--out" in argv:
        self_test()
        path = argv[argv.index("--out") + 1]
        with open(path, "w") as f:
            json.dump(vectors(), f, indent=1, ensure_ascii=True)
            f.write("\n")
        print(f"wrote {path}")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
