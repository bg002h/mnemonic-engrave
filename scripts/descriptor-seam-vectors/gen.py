#!/usr/bin/env python3
"""Generate crates/me-cli/testdata/descriptor_seam_vectors.json.

Authored host columns come from rows.py; EVERY device-side and value column is
MEASURED here -- the Go probe (nonstandard/bip380/address/md at the fork rev
goprobe/go.mod's `replace` points at) and the debug `md` binary.  Nothing is
transcribed from a report.
"""
import hashlib, json, os, subprocess, sys

SP = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, SP)
import rows as R

# Toolchain, overridable; the defaults are what P0's measurements were taken
# with. MD must be the DEBUG binary built from the descriptor-mnemonic tree --
# the installed ~/.cargo/bin/md is stale and lacks `descriptor` (SPEC S2).
# goprobe/go.mod carries `replace seedhammer.com => SEAM_FORK`; point it at the
# fork worktree this corpus is pinned to (S2: the s2/descriptor-arm worktree at
# 0abbf81, carrying P3.1's parse fix and P3.4's ypubVer case).
GO = os.environ.get("SEAM_GO", "/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go")
MD = os.environ.get("SEAM_MD", "/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md")
RSPROBE = os.path.join(SP, "rsprobe", "target", "debug", "rsprobe")
PROBE = os.path.join(SP, "goprobe")

B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def b58check(payload: bytes) -> str:
    chk = hashlib.sha256(hashlib.sha256(payload).digest()).digest()[:4]
    n = int.from_bytes(payload + chk, "big")
    out = ""
    while n:
        n, r = divmod(n, 58)
        out = B58[r] + out
    for b in payload + chk:
        if b == 0:
            out = "1" + out
        else:
            break
    return out


def probe(items):
    p = subprocess.run([GO, "run", "."], cwd=PROBE, input=json.dumps(items),
                       capture_output=True, text=True)
    if p.returncode != 0:
        sys.exit("go probe failed:\n" + p.stderr)
    return json.loads(p.stdout)


def md_encode(template, keys, fps, path):
    cmd = [MD, "encode", template, "--path", path]
    for i, k in enumerate(keys):
        cmd += ["--key", "@%d=%s" % (i, k)]
    if fps:
        for i, f in enumerate(fps):
            cmd += ["--fingerprint", "@%d=%s" % (i, f)]
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        return None, p.stderr.strip()
    return [l for l in p.stdout.split("\n") if l.strip()], None


def md_run(sub, strs, extra=()):
    p = subprocess.run([MD, sub] + strs + list(extra), capture_output=True, text=True)
    if p.returncode != 0:
        return None, p.stderr.strip()
    return p.stdout, None


# ---- 1. materialise the constructed 77-byte base58check token --------------
B58_77 = b58check(bytes(range(77)))
for r in R.ROWS:
    if r["input"] == "__B58_77__":
        r["input"] = B58_77

# ---- 2. device pass A: device_admits on the raw input ----------------------
passA = probe([{"name": r["name"], "input": r["input"],
                "probe": r.get("device_probe") == "panic:parse"} for r in R.ROWS])
A = {o["name"]: o for o in passA}

# ---- 3. device pass B: canonical + addresses from the parse source ---------
needB = [r for r in R.ROWS if r.get("host_admits") or r.get("want_addr") or r.get("want_wid")]
passB = probe([{"name": r["name"],
                "input": r.get("canonical_from") or r.get("addr_from") or r["input"],
                "wallet_id": bool(r.get("want_wid")),
                "probe": r.get("device_probe") == "panic:parse"} for r in needB])
B = {o["name"]: o for o in passB}

problems, notes = [], []
out_rows = []
for r in R.ROWS:
    a = A[r["name"]]
    row = {
        "name": r["name"],
        "input": r["input"],
        "sha256": hashlib.sha256(r["input"].encode()).hexdigest(),
        "host_admits": r["host_admits"],
    }
    if r.get("device_probe") == "panic:parse":
        # device_admits is OMITTED: the predicate cannot be evaluated without
        # panicking the parser, so either boolean would be a false claim.
        pass
    else:
        row["device_admits"] = a["device_admits"]
        if a.get("parse_panic"):
            problems.append("%s: UNEXPECTED parse panic %r" % (r["name"], a["parse_panic"]))
    row["md1_admits"] = r["md1_admits"]
    row["format"] = r["format"]

    b = B.get(r["name"])
    if r["host_admits"]:
        if not b or not b.get("canonical"):
            problems.append("%s: host_admits with no derivable canonical (%r)"
                            % (r["name"], b and (b.get("parse_err") or b.get("encode_panic"))))
        else:
            row["canonical"] = b["canonical"]
            if b.get("fixed_point") is not True:
                problems.append("%s: canonical is NOT a device fixed point (%r)"
                                % (r["name"], b.get("reparse_err")))

    # ---- md1 route ----
    md1_addr = {}
    md1_desc = None
    if r.get("md1_route"):
        t, keys, fps, path = r["md1_route"]
        strs, err = md_encode(t, keys, fps, path)
        if err:
            problems.append("%s: md encode failed: %s" % (r["name"], err))
        else:
            js, err = md_run("address", strs, ("--index", "0", "--count", "2", "--json"))
            if err:
                problems.append("%s: md address failed: %s" % (r["name"], err))
            else:
                for e in json.loads(js)["addresses"]:
                    md1_addr[e["index"]] = e["address"]
            js, err = md_run("inspect", strs, ("--json",))
            if err:
                problems.append("%s: md inspect failed: %s" % (r["name"], err))
            else:
                md1_wid = json.loads(js)["wallet_policy_id"]["hex"]
                if r.get("want_wid"):
                    # Third, independent computation: the PUBLISHED md-codec
                    # 0.42 -- the exact crate `me` links -- over the same set.
                    rp = subprocess.run([RSPROBE], input="\n".join(strs),
                                        capture_output=True, text=True)
                    if rp.returncode != 0:
                        problems.append("%s: rsprobe failed: %s" % (r["name"], rp.stderr.strip()))
                    elif rp.stdout.strip() != md1_wid:
                        problems.append("%s: published md-codec 0.42 %s != md binary %s"
                                        % (r["name"], rp.stdout.strip(), md1_wid))
                    gw = b.get("wallet_id")
                    if not gw:
                        problems.append("%s: Go wallet_id missing: %s"
                                        % (r["name"], b.get("wallet_id_err")))
                    elif gw != md1_wid:
                        problems.append("F-212 CLASS DIVERGENCE %s: Rust %s != Go %s"
                                        % (r["name"], md1_wid, gw))
                    else:
                        row["wallet_id"] = md1_wid
                        notes.append("%s wallet_id %s: md-cli == published md-codec 0.42 == fork Go md"
                                     % (r["name"], md1_wid))
            if r.get("md_descriptor_contains"):
                d, err = md_run("descriptor", strs)
                if err:
                    problems.append("%s: md descriptor failed: %s" % (r["name"], err))
                elif r["md_descriptor_contains"] not in d:
                    problems.append("%s: md descriptor read-back lacks %r"
                                    % (r["name"], r["md_descriptor_contains"]))
                else:
                    row["md_descriptor_contains"] = r["md_descriptor_contains"]
    elif r.get("want_wid"):
        problems.append("%s: want_wid without an md1_route" % r["name"])

    # ---- address_0 / address_1 ----
    if r.get("want_addr"):
        dev = {0: (b or {}).get("address_0"), 1: (b or {}).get("address_1")}
        for i in [int(c) for c in r["want_addr"]]:
            d, m = dev.get(i), md1_addr.get(i)
            if d and m and d != m:
                problems.append("%s: address_%d route split: device %s != md1 %s"
                                % (r["name"], i, d, m))
            v = d or m
            if not v:
                problems.append("%s: address_%d underivable (device err %r)"
                                % (r["name"], i, (b or {}).get("address_%d_err" % i)))
            else:
                row["address_%d" % i] = v
                if d and m:
                    notes.append("%s address_%d: device == md1 route (%s)" % (r["name"], i, v))

    if r.get("device_probe"):
        row["device_probe"] = r["device_probe"]
    if r.get("gate"):
        row.update(r["gate"])
    row["covers"] = r["covers"]
    row["source"] = r["source"]
    out_rows.append(row)

if problems:
    print("PROBLEMS:", file=sys.stderr)
    for p in problems:
        print("  " + p, file=sys.stderr)
    sys.exit(1)

REFUSAL_ROWS = json.load(open(os.path.join(SP, "refusal_rows.json")))
doc = {
    "_comment": json.load(open(os.path.join(SP, "comment.json"))),
    "invariant": "host_admits(input) => device_admits(canonical(input))",
    "refusal_rows": REFUSAL_ROWS,
    "vectors": out_rows,
}
dest = sys.argv[1]
with open(dest, "w") as f:
    json.dump(doc, f, indent=2, ensure_ascii=True)
    f.write("\n")
raw = open(dest, "rb").read()
print("wrote %s: %d bytes, %d rows, sha256 %s"
      % (dest, len(raw), len(out_rows), hashlib.sha256(raw).hexdigest()))
for n in notes:
    print("  note: " + n)
