#!/usr/bin/env python3
"""Independent generator for the MNEMBLOB v2 vectors of
design/SPEC_encrypted_payload_delivery.md §14 (spec S1b, 2026-11-22).

Written in Python on purpose: it shares no code with the Rust `seal` module it
pins, so a Rust bug cannot also be a vector bug. Inputs are the literal ms1
strings below (made by `ms encode --hex -`, ms 0.20.4, from test entropies;
test material only, never a real seed).

Usage:
  gen-mnemblob-v2-vectors.py            print every vector as JSON
  gen-mnemblob-v2-vectors.py --check F  exit 1 unless every value the spec
                                        prints (header hex, key, tag, blob
                                        sha256, pubhash, lengths) occurs in F

NOT covered: this script does not run the Rust or Go code, does not check
the spec's prose, refusal texts or test-row names, and does not build UF2s.
It checks that the spec's printed vector values equal an independent
computation, and nothing else.
"""
import hashlib
import json
import sys

from cryptography.hazmat.primitives.ciphers.aead import AESGCM

CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"

MS1 = {
    "A16": "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
    "A": "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w",
    "B": "ms10entrsqqg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3ztl5m2c7mg3ls6",
    "C": "ms10entrsqq3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zym4y84zxyuwtkl",
    "D": "ms10entrsqqenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxgyv57h75aydvt",
    "E": "ms10entrsqpzyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygxfk0vzg2zk356",
    "F": "ms10entrsqp2424242424242424242424242424242424242424242424242424c7u8hs6ruhww",
    "G": "ms10entrsqpnxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxv9jwqcdg9hrrgt",
    "P": "ms10entrsqrllllllllllllllllllllllllllllllllllllllllllllllllll7ydtcvhdp9ycqe",
}
# Vector D's public cards and ms1 (crates/me-cli/testdata/seal_vectors.json).
D_PUBLIC = [
    "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g",
    "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8",
    "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3",
    "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374",
    "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz",
]
D_MS1 = "ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw"

# n-word iteration floor (S1b STOP-4, Brian 2026-11-22). --check requires the spec text to carry it.
NWORD_FLOOR = 600_000
FLOOR_TEXTS = [
    "`iterations < {f_us}` is refused",
    "MIN_ITERATIONS_NWORD: u32 = {f_us}",
    "[{f_nc}, 2000000] for a 6 to 12 word password",
    "`seal.MinIterationsNWord = {f_us}`",
    "a floor of {f_c}, NOT the recommendation",
    "`{f_nc}` unchanged -> accepted",
]

BEEF12 = " ".join(["beef"] * 12)
BEEF6 = " ".join(["beef"] * 6)
ZOO12 = " ".join(["zoo"] * 12)


def ms1_payload(s):
    """The codex32 payload bytes of an ms1 string (header 6 chars, checksum 13)."""
    data = s[3:]
    chars = data[6:-13]
    bits = "".join(format(CHARSET.index(c), "05b") for c in chars)
    return bytes(int(bits[i:i + 8], 2) for i in range(0, len(bits) - len(bits) % 8, 8))


def ms1_letter(s):
    p = ms1_payload(s)
    assert p[0] == 0x00, "ms1 reserved-prefix byte"
    return chr(ord("A") + (p[1] >> 4))


def normalise(words):
    """mr-codec plates_key / EPD §8.1: split on whitespace, lower-case, join by one space."""
    return " ".join(w.lower() for w in words.split())


def titles_record(lines_after_magic, magic="refugium-plates 2", final_lf=True):
    if final_lf:
        block = magic + "\n" + "".join(line + "\n" for line in lines_after_magic)
    else:
        block = magic + "\n" + "\n".join(lines_after_magic)
    return "text:" + block.encode("utf-8").hex()


def header(version, pw_form, iterations, salt, iv, pub_len, ct_len):
    sealed = ct_len > 0
    return (b"MNEMBLOB" + bytes([version, 1 if sealed else 0, 1 if sealed else 0, pw_form])
            + iterations.to_bytes(4, "big") + salt + iv
            + pub_len.to_bytes(4, "big") + ct_len.to_bytes(4, "big"))


def pubhash(records, sealed):
    h = hashlib.sha256()
    h.update(b"MNEMBLOB/pub/v1")
    h.update(b"\x00")
    h.update(b"\x01" if sealed else b"\x00")
    h.update(bytes([len(records)]))
    h.update("\n".join(records).encode())
    return h.digest()[:16].hex()


def rep(pair, n):
    return bytes.fromhex(pair * n)


def build(name, pw_form, words, iterations, salt, iv, public, secret, version=2):
    pub = "\n".join(public).encode()
    sec = "\n".join(secret).encode()
    hdr = header(version, pw_form, iterations, salt, iv, len(pub), len(sec))
    key = hashlib.pbkdf2_hmac("sha256", normalise(words).encode(), salt, iterations, 32)
    ct = AESGCM(key).encrypt(iv, sec, hdr + pub)
    blob = hdr + pub + ct
    return {
        "name": name,
        "version": version,
        "pw_form": pw_form,
        "password": words,
        "iterations": iterations,
        "salt_hex": salt.hex(),
        "iv_hex": iv.hex(),
        "public": public,
        "secret": secret,
        "pub_len": len(pub),
        "ct_len": len(sec),
        "record_count": len(public) + len(secret),
        "blob_len": len(blob),
        "header_hex": hdr.hex(),
        "derived_key_hex": key.hex(),
        "tag_hex": ct[-16:].hex(),
        "blob_sha256": hashlib.sha256(blob).hexdigest(),
        "pubhash_sealed": pubhash(public, True) if public and not any(r.startswith("text:") for r in public) else None,
        "letters": [ms1_letter(s) if s.startswith("ms1") else None for s in secret],
    }


def vectors():
    seven = [MS1[c] for c in "ABCDEFG"]
    t7 = [f"title SEED {c}" for c in "ABCDEF"] + ["title HOT SEED G"]
    neg_salt = rep("0bad", 8)
    v = [
        build("V2-A", 6, BEEF6, 600_000, rep("feed", 8), rep("face", 6), [],
              [MS1["A16"], MS1["P"], titles_record(["title SEED A", "title KEEP; DO NOT LOAD"])]),
        build("V2-B", 12, ZOO12, 600_000, rep("deaf", 8), rep("fade", 6), [],
              seven + [titles_record(t7)]),
        build("V2-C", 0, BEEF12, 100_000, rep("dead", 8), rep("beef", 6), D_PUBLIC, [D_MS1]),
        build("N-count", 6, BEEF6, 600_000, neg_salt, rep("0c01", 6), [],
              [MS1["A16"], MS1["P"], titles_record(["title SEED A"])]),
        build("N-letter", 6, BEEF6, 600_000, neg_salt, rep("0c02", 6), [],
              [MS1["B"], titles_record(["title SEED C"])]),
        build("N-hot-letter", 6, BEEF6, 600_000, neg_salt, rep("0c03", 6), [],
              [MS1["G"], titles_record(["title HOT SEED F"])]),
        build("N-magic", 6, BEEF6, 600_000, neg_salt, rep("0c04", 6), [],
              [MS1["A16"], titles_record(["title SEED A"], magic="refugium-plates 1")]),
        build("N-two", 6, BEEF6, 600_000, neg_salt, rep("0c05", 6), [],
              [MS1["A16"], titles_record(["title SEED A"]), titles_record(["title SEED A"])]),
        build("N-not-last", 6, BEEF6, 600_000, neg_salt, rep("0c06", 6), [],
              [titles_record(["title SEED A"]), MS1["A16"]]),
        build("N-title-set", 6, BEEF6, 600_000, neg_salt, rep("0c07", 6), [],
              [MS1["A16"], titles_record(["title SEED Q"])]),
        build("N-no-final-lf", 6, BEEF6, 600_000, neg_salt, rep("0c08", 6), [],
              [MS1["A16"], titles_record(["title SEED A"], final_lf=False)]),
        build("N-empty", 6, BEEF6, 600_000, neg_salt, rep("0c09", 6), [],
              [MS1["A16"], titles_record([])]),
        build("N-public-titles", 6, BEEF6, 600_000, neg_salt, rep("0c0a", 6),
              [titles_record(["title SEED A"])], [MS1["A16"]]),
        # n-word floor (S1b STOP-4, Brian 2026-11-22): byte 11 = 6 below 600,000 is refused at the header.
        # The accepted side at exactly 600,000 with byte 11 = 6 is V2-A; byte 11 = 0 at 100,000 is V2-C.
        build("N-floor-599999", 6, BEEF6, 599_999, rep("0bee", 8), rep("0c0c", 6), [],
              [MS1["A16"], MS1["P"], titles_record(["title SEED A", "title KEEP; DO NOT LOAD"])]),
        build("N-v1-text", 0, BEEF12, 100_000, neg_salt, rep("0c0b", 6), [],
              [MS1["A16"], titles_record(["title SEED A"])], version=1),
    ]
    return v


def main():
    vs = vectors()
    if len(sys.argv) == 3 and sys.argv[1] == "--check":
        text = open(sys.argv[2], encoding="utf-8").read()
        flat = text.replace("\n", "").replace(" ", "")
        missing = []
        for v in vs:
            for k in ("header_hex", "derived_key_hex", "tag_hex", "blob_sha256", "pubhash_sealed"):
                val = v[k]
                if val and val not in flat:
                    missing.append(f"{v['name']} {k} {val}")
            row = "|{}|{}|{}|{}|{}|{}|{}|".format(
                v["name"], v["pw_form"], v["iterations"], v["pub_len"], v["ct_len"],
                v["record_count"], v["blob_len"])
            if row not in flat:
                missing.append(f"{v['name']} shape row {row}")
            drow = "|{}|`{}`|`{}`|`{}`|".format(
                v["name"], v["derived_key_hex"], v["tag_hex"], v["blob_sha256"])
            if drow not in flat:
                missing.append(f"{v['name']} digest row {drow}")
            for s in v["secret"] + v["public"]:
                if s.startswith("text:") and s not in flat:
                    missing.append(f"{v['name']} titles record {s[:24]}...")
        floor_args = dict(f_us=f"{NWORD_FLOOR:_}", f_nc=str(NWORD_FLOOR), f_c=f"{NWORD_FLOOR:,}")
        flat1 = " ".join(text.split())
        for t in FLOOR_TEXTS:
            want = t.format(**floor_args)
            if want not in flat1:
                missing.append(f"floor text {want}")
        by = {v["name"]: v for v in vs}
        assert by["V2-A"]["iterations"] == NWORD_FLOOR and by["N-floor-599999"]["iterations"] == NWORD_FLOOR - 1
        for m in missing:
            print("MISSING", m)
        print(f"checked {len(vs)} vectors, {len(missing)} missing")
        sys.exit(1 if missing else 0)
    if len(sys.argv) == 2 and sys.argv[1] == "--tables":
        print("| vector | pw_form | iterations | pub_len | ct_len | records | blob length |")
        print("| --- | --- | --- | --- | --- | --- | --- |")
        for v in vs:
            print("| {} | {} | {} | {} | {} | {} | {} |".format(
                v["name"], v["pw_form"], v["iterations"], v["pub_len"], v["ct_len"],
                v["record_count"], v["blob_len"]))
        print()
        print("| vector | derived key | tag | blob sha256 |")
        print("| --- | --- | --- | --- |")
        for v in vs:
            print("| {} | `{}` | `{}` | `{}` |".format(
                v["name"], v["derived_key_hex"], v["tag_hex"], v["blob_sha256"]))
        print()
        print("```")
        for v in vs:
            h = v["header_hex"]
            print("{:<16}{}".format(v["name"], h[:64]))
            print("{:<16}{}".format("", h[64:]))
        print("```")
        return
    json.dump(vs, sys.stdout, indent=1)
    print()


if __name__ == "__main__":
    main()
