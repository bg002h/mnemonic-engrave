#!/usr/bin/env python3
"""Self-test for policy-differential.py's comparison and safety logic.

A differential harness whose comparator cannot report a disagreement is a
false-PASS machine: it prints `agreed=100` forever and nobody can tell it from
a harness that works.  So the comparator is mutation-tested here directly --
each case feeds it legs that are deliberately WRONG and asserts the finding
fires, then feeds it agreeing legs and asserts it stays quiet.

Also covers the two pieces where a silent bug would be expensive rather than
merely wrong: the Core datadir guard (which is what keeps this away from the
operator's live node) and the xpub->tpub conversion (a corruption there would
read as a Core disagreement on every case).

    python3 scripts/policy-differential-selftest.py
"""

import importlib.util
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location(
    "policy_differential", os.path.join(HERE, "policy-differential.py"))
pd = importlib.util.module_from_spec(_spec)
# Must be in sys.modules BEFORE exec: @dataclass resolves annotations through
# sys.modules[cls.__module__], and a hyphenated filename is never imported
# normally, so nothing else puts it there.
sys.modules["policy_differential"] = pd
_spec.loader.exec_module(pd)

sys.path.insert(0, HERE)
from policy_keys import b58decode_check, derive_account  # noqa: E402

FAILURES = []


def check(name, cond, detail=""):
    if cond:
        print(f"  ok   {name}")
    else:
        print(f"  FAIL {name} {detail}")
        FAILURES.append(name)


# ---------------------------------------------------------------------------
# Fixtures, built rather than quoted.
#
# The first draft of this file hand-wrote a `bcrt1…` address and got its
# checksum wrong; three comparator checks then "failed" for a reason that had
# nothing to do with the comparator.  Encoding the fixtures from a witness
# program removes that whole class -- and the encoder is cross-checked against
# the BIP-173 vectors below before anything else runs, so a bug in IT cannot
# quietly produce agreeing-but-wrong fixtures either.
# ---------------------------------------------------------------------------

_CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"


def _polymod(values):
    gen = [0x3B6A57B2, 0x26508E6D, 0x1EA119FA, 0x3D4233DD, 0x2A1462B3]
    chk = 1
    for v in values:
        top = chk >> 25
        chk = ((chk & 0x1FFFFFF) << 5) ^ v
        for i in range(5):
            chk ^= gen[i] if ((top >> i) & 1) else 0
    return chk


def _hrp_expand(hrp):
    return [ord(c) >> 5 for c in hrp] + [0] + [ord(c) & 31 for c in hrp]


def _to5(data):
    acc = bits = 0
    out = []
    for b in data:
        acc = (acc << 8) | b
        bits += 8
        while bits >= 5:
            bits -= 5
            out.append((acc >> bits) & 31)
    if bits:
        out.append((acc << (5 - bits)) & 31)
    return out


def bech32_address(hrp: str, witver: int, prog: bytes) -> str:
    data = [witver] + _to5(prog)
    const = 1 if witver == 0 else 0x2BC830A3
    chk = _polymod(_hrp_expand(hrp) + data + [0] * 6) ^ const
    checksum = [(chk >> 5 * (5 - i)) & 31 for i in range(6)]
    return hrp + "1" + "".join(_CHARSET[d] for d in data + checksum)


_P2WPKH = bytes.fromhex("751e76e8199196d454941c45d1b3a323f1433bd6")
_P2WSH = bytes.fromhex("1863143c14c5166804bd19203356da136c985678cd4d27a1b8c6329604903262")

# Cross-check the encoder against the published BIP-173 vectors before use.
assert bech32_address("bc", 0, _P2WPKH) == "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", \
    "bech32 encoder disagrees with BIP-173 vector 1"
assert bech32_address("bc", 0, _P2WSH) == \
    "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3", \
    "bech32 encoder disagrees with BIP-173 vector 2"

# One program in two networks (must compare EQUAL), and a different program in
# two networks (must compare UNEQUAL).
MAIN = [bech32_address("bc", 0, _P2WPKH)]
REGT = [bech32_address("bcrt", 0, _P2WPKH)]
OTHER_MAIN = [bech32_address("bc", 0, _P2WSH)]
OTHER_REGT = [bech32_address("bcrt", 0, _P2WSH)]

for _a in MAIN + REGT + OTHER_MAIN + OTHER_REGT:
    from policy_keys import normalize_address as _na
    _na(_a)  # raises if a fixture is not a decodable address

COMPOSED = {"wrapper": "wsh", "slots": [{"index": 0, "ordinal": 0, "path": 0}],
            "template_with_origins": "wsh(pk(@0/<0;1>/*))"}
POLICY = pd.Policy("wsh", (pd.SpendPath(k=1, n=1),))


def rust(main=True):
    src = MAIN if main else REGT
    return {"receive": list(src), "change": list(src)}


def device_ok(addrs=None):
    a = addrs if addrs is not None else MAIN
    return {"id": "t", "ok": True, "keys": 1, "receive": list(a), "change": list(a)}


def run_compare(**kw):
    args = dict(
        case_id="t", policy=POLICY, composed=COMPOSED, chunks=["md1x"],
        rust_main=rust(True), rust_regtest=rust(False),
        core_receive=list(REGT), core_change=list(REGT),
        device=device_ok(), device_available=True, experimental=False,
    )
    args.update(kw)
    return pd.compare(**args)


def kinds(findings):
    return {f.kind for f in findings}


print("comparator -- the control: every leg agreeing must yield NO finding")
check("all legs agree -> no findings", run_compare() == [],
      f"got {[f.kind for f in run_compare()]}")

print("\ncomparator -- mutation: each leg made wrong in turn MUST be caught")
f = run_compare(core_receive=list(OTHER_REGT))
check("Core disagrees on receive -> ADDRESS_MISMATCH", "ADDRESS_MISMATCH" in kinds(f))
f = run_compare(core_change=list(OTHER_REGT))
check("Core disagrees on change -> ADDRESS_MISMATCH", "ADDRESS_MISMATCH" in kinds(f))
f = run_compare(device=device_ok(OTHER_MAIN))
check("device disagrees -> ADDRESS_MISMATCH", "ADDRESS_MISMATCH" in kinds(f))
f = run_compare(rust_regtest={"receive": list(OTHER_REGT), "change": list(REGT)})
check("md's own networks disagree -> RUST_NETWORK_VARIANCE",
      "RUST_NETWORK_VARIANCE" in kinds(f))

print("\ncomparator -- the CHANGE chain is really compared, not just receive")
# A harness that only ever looked at receive would pass the control above and
# miss every change-chain defect; this is the case that separates them.
f = run_compare(device={"id": "t", "ok": True, "keys": 1,
                        "receive": list(MAIN), "change": list(OTHER_MAIN)})
check("device correct on receive, wrong on change -> caught",
      "ADDRESS_MISMATCH" in kinds(f) and any("change" in x.detail for x in f))

print("\ncomparator -- acceptance disagreements")
f = run_compare(core_receive=pd.Refusal("core/getdescriptorinfo", "nope"),
                core_change=pd.Refusal("core/getdescriptorinfo", "nope"))
check("md accepts, Core refuses -> ACCEPTANCE_MISMATCH", "ACCEPTANCE_MISMATCH" in kinds(f))
check("...and it is NOT classed as documented", not any(x.expected for x in f))
f = run_compare(device={"id": "t", "ok": False, "stage": "source", "error": "no"})
check("md accepts, device refuses -> ACCEPTANCE_MISMATCH", "ACCEPTANCE_MISMATCH" in kinds(f))
f = run_compare(device={"id": "t", "ok": False, "stage": "banana", "error": "no"})
check("off-contract refusal stage -> DEVICE_CONTRACT", "DEVICE_CONTRACT" in kinds(f))
f = run_compare(device={"id": "t", "ok": True, "keys": 9,
                        "receive": list(MAIN), "change": list(MAIN)})
check("device key count != compose slot count -> SLOT_COUNT_MISMATCH",
      "SLOT_COUNT_MISMATCH" in kinds(f))
f = run_compare(device=None)
check("device silent for a case id -> DEVICE_SILENT", "DEVICE_SILENT" in kinds(f))

print("\ncomparator -- an undecodable address is REPORTED, never compared away")
# The first draft of this file supplied a mis-checksummed address by accident.
# The comparator must say so rather than treat it as 'different' or 'equal'.
f = run_compare(core_receive=["bcrt1qthisisnotanaddress"])
check("undecodable Core address -> UNPARSEABLE_ADDRESS", "UNPARSEABLE_ADDRESS" in kinds(f))
f = run_compare(device={"id": "t", "ok": True, "keys": 1,
                        "receive": ["nonsense"], "change": list(MAIN)})
check("undecodable device address -> UNPARSEABLE_ADDRESS", "UNPARSEABLE_ADDRESS" in kinds(f))

print("\ncomparator -- the documented-divergence suppression is NARROW")
sane = pd.Refusal("core/getdescriptorinfo",
                  "error code: -5: wsh(..) is not sane: witnesses without signature exist")
f = run_compare(core_receive=sane, core_change=sane, experimental=True)
check("Core sanity refusal + --experimental -> classed documented",
      f and all(x.expected for x in f))
f = run_compare(core_receive=sane, core_change=sane, experimental=False)
check("same refusal WITHOUT --experimental -> stays a finding",
      f and not any(x.expected for x in f))
other = pd.Refusal("core/getdescriptorinfo",
                   "error code: -5: wsh(..) is not sane: malleable witnesses exist")
f = run_compare(core_receive=other, core_change=other, experimental=True)
check("a DIFFERENT Core sanity refusal -> stays a finding",
      f and not any(x.expected for x in f))

print("\nsignatures -- the same divergence collapses, different ones do not")
a = pd.Refusal("core/x", "wsh(multi(2,<key>,<key>)) is not sane: witnesses without signature exist")
b = pd.Refusal("core/x", "wsh(multi(3,<key>,<key>,<key>)) is not sane: witnesses without signature exist")
c = pd.Refusal("core/x", "wsh(multi(2,<key>,<key>)) is not sane: malleable witnesses exist")
check("two shapes, one reason -> one signature",
      pd._normalize_signature_text(a.text) == pd._normalize_signature_text(b.text),
      f"\n    {pd._normalize_signature_text(a.text)}\n    {pd._normalize_signature_text(b.text)}")
check("same shape, two reasons -> two signatures",
      pd._normalize_signature_text(a.text) != pd._normalize_signature_text(c.text))

print("\nCore datadir guard -- must refuse anything near the operator's node")
for bad in ("~/.bitcoin", "/home/bcg/.bitcoin", "/tmp/x", "/scratch/code/shibboleth/mnemonic-engrave"):
    try:
        pd._assert_safe_datadir(os.path.expanduser(bad))
        check(f"refuses {bad}", False, "-- it was ACCEPTED")
    except SystemExit:
        check(f"refuses {bad}", True)
try:
    pd._assert_safe_datadir("/scratch/code/shibboleth/.tmp/policy-diff-regtest")
    check("accepts the scratch datadir", True)
except SystemExit as exc:
    check("accepts the scratch datadir", False, str(exc))

print("\nxpub -> tpub conversion preserves key material exactly")
acct = derive_account(0, 2)
desc = f"wsh(pk([{acct['fingerprint']}/48'/0'/0'/2']{acct['xpub']}/<0;1>/*))#aaaaaaaa"
tdesc = pd.to_tpub_descriptor(desc)
import re as _re
tkey = _re.search(r"tpub[1-9A-HJ-NP-Za-km-z]+", tdesc).group(0)
check("depth/parent/child/chaincode/pubkey identical",
      b58decode_check(tkey)[4:] == b58decode_check(acct["xpub"])[4:])
check("version bytes swapped", b58decode_check(tkey)[:4] == (0x043587CF).to_bytes(4, "big"))
check("checksum stripped (Core recomputes it)", "#" not in tdesc)

# The conversion's guard must be able to FAIL, or it is decoration. Feed it a
# well-formed base58check string that is not a 78-byte mainnet xpub record.
from policy_keys import b58encode_check as _enc  # noqa: E402
_fake = "xpub" + _enc(b"\x04\x88\xb2\x1e" + b"\x00" * 20)[4:]
try:
    pd.to_tpub_descriptor(f"wsh(pk({_fake}/0/*))")
    check("conversion refuses a non-78-byte record", False, "-- it was ACCEPTED")
except ValueError:
    check("conversion refuses a non-78-byte record", True)
try:
    pd.to_tpub_descriptor("wsh(pk(xpubNotBase58!!!/0/*))")
    check("conversion refuses undecodable base58", False, "-- it was ACCEPTED")
except ValueError:
    check("conversion refuses undecodable base58", True)

print("\ngenerator stays inside the composer's limits")
limits = pd.read_limits(pd.DEFAULT_COMPOSE_GO)
import random as _random
rng = _random.Random(99)
bad = []
for _ in range(3000):
    p = pd.generate_policy(rng, limits)
    if len(p.paths) > limits.max_paths:
        bad.append(("paths", p))
    if p.slots > limits.max_slots:
        bad.append(("slots", p))
    if any(q.n > limits.max_keys_per_path for q in p.paths):
        bad.append(("keys", p))
    if any(q.n and not (1 <= q.k <= q.n) for q in p.paths):
        bad.append(("threshold", p))
    if not any(not q.keyless for q in p.paths):
        bad.append(("no keyed path", p))
    if p.wrapper == "tr" and any(q.keyless for q in p.paths):
        bad.append(("keyless under tr", p))
    if p.wrapper in pd.LEGACY_WRAPPERS and (
            len(p.paths) != 1 or p.paths[0].lock or p.paths[0].sha256
            or p.paths[0].unsorted or p.paths[0].n < 2):
        bad.append(("legacy shape", p))
    if any(q.keyless and not q.sha256 for q in p.paths):
        bad.append(("keyless without hashlock", p))
check("3000 generated policies all inside bounds", not bad,
      f"-- {len(bad)} violations, first: {bad[0] if bad else ''}")

print("\ngenerator is deterministic under a seed")
g1 = [pd.generate_policy(_random.Random(7), limits) for _ in range(1)]
g2 = [pd.generate_policy(_random.Random(7), limits) for _ in range(1)]
seq1 = [pd.generate_policy(r, limits) for r in [_random.Random(7)] * 1]
r = _random.Random(11)
a_run = [pd.generate_policy(r, limits) for _ in range(50)]
r = _random.Random(11)
b_run = [pd.generate_policy(r, limits) for _ in range(50)]
check("same seed -> identical 50-policy stream", a_run == b_run)
check("different seed -> different stream",
      a_run != [pd.generate_policy(_random.Random(12), limits) for _ in range(50)])

print("\ngenerator actually varies every axis it claims to")
r = _random.Random(5)
pool = [pd.generate_policy(r, limits) for _ in range(600)]
seen_wrappers = {p.wrapper for p in pool}
seen_locks = {q.lock[0] for p in pool for q in p.paths if q.lock}
check("all four wrappers appear", seen_wrappers == {"wsh", "tr", "sh", "sh-wsh"}, seen_wrappers)
check("all four lock kinds appear", seen_locks == set(pd.LOCK_KINDS), seen_locks)
check("hashlocks appear", any(q.sha256 for p in pool for q in p.paths))
check("keyless paths appear", any(q.keyless for p in pool for q in p.paths))
check("unsorted appears", any(q.unsorted for p in pool for q in p.paths))
check("path counts reach the max", max(len(p.paths) for p in pool) == limits.max_paths)
check("key counts reach the max",
      max((q.n for p in pool for q in p.paths), default=0) == limits.max_keys_per_path)

print()
if FAILURES:
    print(f"SELFTEST FAILED: {len(FAILURES)} check(s): {', '.join(FAILURES)}")
    sys.exit(1)
print("SELFTEST OK")
