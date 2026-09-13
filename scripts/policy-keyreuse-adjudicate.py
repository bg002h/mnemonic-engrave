#!/usr/bin/env python3
"""Ask Bitcoin Core to adjudicate the key-reuse trio.

Three vendored vectors reuse one placeholder at two use sites with the SAME path
expression -- `@0` as a taproot internal key AND inside its own script leaf, or
`@1`/`@2` in both branches of a wsh or_i:

    keyed_tr_multi_a            tr(@0/…,multi_a(2,@0/…,@1/…))
    keyed_tr_sortedmulti_a      tr(@0/…,sortedmulti_a(2,@0/…,@1/…))
    keyed_wsh_timelock_hashlock wsh(or_i(…multi(2,@0,@1,@2)…,…multi(1,@1,@2)…))

The two constellation halves disagree about them.  `md address` REFUSES all
three -- "@N appears at 2 use sites in this template with the same path
expression" -- while the device derives addresses, and the addresses it derives
match each vector's own .conformance.json exactly (F-513/F-514).

A refusal and a derivation are not comparable answers, so neither half settles
it.  Bitcoin Core shares no code with either and is the only third opinion
available: if Core derives what the device derives, md's refusal is a GUARD over
a well-defined descriptor, and the question is whether that guard should be a
warning.  If Core derives something else, or refuses, md's refusal is a
DISAGREEMENT and the device's addresses are the ones in doubt.

Each vector carries the concrete descriptor Core needs and the expected
addresses, so this reads them straight off disk rather than re-deriving anything
-- the harness must not become a fourth opinion.

    scripts/policy-keyreuse-adjudicate.py            # needs bitcoind already up
"""

import argparse
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from policy_keys import normalize_address  # noqa: E402

import importlib.util  # noqa: E402

_spec = importlib.util.spec_from_file_location(
    "policy_differential",
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "policy-differential.py"))
pd = importlib.util.module_from_spec(_spec)
sys.modules["policy_differential"] = pd
_spec.loader.exec_module(pd)

TRIO = ["keyed_tr_multi_a", "keyed_tr_sortedmulti_a", "keyed_wsh_timelock_hashlock"]

DEFAULT_VECTORS = "/scratch/code/shibboleth/.tmp/seedhammer-ppfix/md/testdata/vectors"


def rpc(cli_argv, *args):
    proc = subprocess.run([*cli_argv, *args], capture_output=True, text=True, timeout=60)
    return proc.returncode, proc.stdout, proc.stderr


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--vectors", default=DEFAULT_VECTORS)
    ap.add_argument("--md", default=pd.DEFAULT_MD)
    ap.add_argument("--bitcoin-bin", default=pd.DEFAULT_BITCOIN_BIN)
    ap.add_argument("--datadir", default=pd.DEFAULT_DATADIR)
    ap.add_argument("--rpc-port", type=int, default=pd.DEFAULT_RPC_PORT)
    ap.add_argument("--indices", type=int, default=3)
    ap.add_argument("--json-out", default=None)
    args = ap.parse_args()

    pd._assert_safe_datadir(os.path.abspath(args.datadir))
    cli = [
        os.path.join(args.bitcoin_bin, "bitcoin-cli"),
        f"-datadir={os.path.abspath(args.datadir)}",
        f"-conf={os.path.join(os.path.abspath(args.datadir), 'bitcoin.conf')}",
        f"-rpcport={args.rpc_port}",
        "-rpcuser=policydiff", "-rpcpassword=policydiff",
    ]
    rc, _, err = rpc(cli, "getblockchaininfo")
    if rc != 0:
        raise SystemExit(f"bitcoind not reachable on port {args.rpc_port}: {err.strip()}")

    results = []
    for name in TRIO:
        base = os.path.join(args.vectors, name)
        with open(base + ".conformance.json") as fh:
            conf = json.load(fh)
        with open(base + ".template") as fh:
            template = fh.read().strip()
        phrases = [ln.strip() for ln in open(base + ".phrase.txt")
                   if ln.strip().startswith("md1")]

        entry = {"vector": name, "template": template, "chains": {}}

        # What md itself says, quoted rather than paraphrased.
        proc = subprocess.run(
            [args.md, "address", *phrases, "--network", "mainnet",
             "--count", str(args.indices)],
            capture_output=True, text=True, timeout=60)
        entry["md_rc"] = proc.returncode
        entry["md_stdout"] = proc.stdout.strip()
        entry["md_stderr"] = pd._clean(proc.stderr) if proc.returncode else ""

        for chain in ("0", "1"):
            cdata = conf["chains"][chain]
            expected = cdata["addresses"][:args.indices]
            desc = cdata["descriptor"]
            row = {"expected_device_and_vector": expected, "descriptor": desc}
            try:
                tdesc = pd.to_tpub_descriptor(desc)
            except ValueError as exc:
                row["core"] = {"ok": False, "stage": "convert", "error": str(exc)}
                entry["chains"][chain] = row
                continue
            rc, out, err = rpc(cli, "getdescriptorinfo", tdesc)
            if rc != 0:
                row["core"] = {"ok": False, "stage": "getdescriptorinfo",
                               "error": pd._clean_rpc(err)}
                entry["chains"][chain] = row
                continue
            checksummed = json.loads(out)["descriptor"]
            rc, out, err = rpc(cli, "deriveaddresses", checksummed,
                               json.dumps([0, args.indices - 1]))
            if rc != 0:
                row["core"] = {"ok": False, "stage": "deriveaddresses",
                               "error": pd._clean_rpc(err)}
                entry["chains"][chain] = row
                continue
            got = json.loads(out)
            row["core"] = {"ok": True, "addresses": got}
            try:
                row["agree"] = ([normalize_address(a) for a in got]
                                == [normalize_address(a) for a in expected])
            except ValueError as exc:
                row["agree"] = None
                row["normalize_error"] = str(exc)
            entry["chains"][chain] = row
        results.append(entry)

    # ---- render -----------------------------------------------------------
    verdicts = []
    for e in results:
        print("=" * 78)
        print(e["vector"])
        print("-" * 78)
        print(f"  template : {e['template'][:120]}")
        if e["md_rc"]:
            print(f"  md       : REFUSES — {e['md_stderr']}")
        else:
            print(f"  md       : derives {e['md_stdout'].splitlines()[:1]}")
        agree_all = True
        core_ok_all = True
        for chain, row in e["chains"].items():
            label = "receive" if chain == "0" else "change "
            core = row["core"]
            if not core["ok"]:
                core_ok_all = False
                agree_all = False
                print(f"  core {label}: REFUSES at {core['stage']} — {core['error'][:150]}")
                continue
            mark = "MATCH" if row.get("agree") else "DIFFER"
            if not row.get("agree"):
                agree_all = False
            print(f"  core {label}: {mark}")
            print(f"      vector/device : {row['expected_device_and_vector'][0]}")
            print(f"      core          : {core['addresses'][0]}")
            if not row.get("agree"):
                for i, (a, b) in enumerate(zip(core["addresses"],
                                               row["expected_device_and_vector"])):
                    if a != b:
                        print(f"      first difference at index {i}: "
                              f"core={a} vector={b}")
                        break
        if core_ok_all and agree_all:
            v = ("Core AGREES with the device and the vector on every index and both "
                 "chains → md's refusal is a GUARD over a descriptor all three of "
                 "Core, the device and the vector agree on.")
        elif not core_ok_all:
            v = ("Core REFUSES this descriptor → md's refusal has independent "
                 "support; the device is the outlier.")
        else:
            v = ("Core DERIVES DIFFERENT addresses from the device → a real "
                 "three-way disagreement; the device's addresses are in doubt.")
        print(f"  verdict  : {v}")
        verdicts.append((e["vector"], v))
        e["verdict"] = v

    print("=" * 78)
    for name, v in verdicts:
        print(f"{name}: {v.split('→')[0].strip()}")

    if args.json_out:
        with open(args.json_out, "w") as fh:
            json.dump(results, fh, indent=2)
        print(f"\nwrote {args.json_out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
