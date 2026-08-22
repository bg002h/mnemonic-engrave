#!/usr/bin/env python3
"""THE RESTORE TEST — taproot. Do the plates plus the seeds give the wallet back?

The wsh restore test asks whether the right KEYS come back. That question is
script-independent: it compares derived xpubs, and both journeys share the same
eleven keys, origins and seeds, so running it again here would measure nothing
new.

Taproot adds a question wsh does not have. A tr() wallet's addresses depend on
the TAPTREE SHAPE as well as the keys — the Merkle root is built from the leaf
scripts and their depths, so two trees over identical keys in a different shape
spend to different addresses. A restore that recovers every key and rebuilds
the tree wrongly produces a wallet that looks correct and is not.

So this measures the whole chain, from the ENGRAVED KEYLESS TEMPLATE:

  1. read each slot's origin from the card (not from the key files)
  2. derive that key from its seed at that origin
  3. re-encode the template with the recovered keys
  4. derive addresses from the RESULT
  5. require them to equal the addresses the journey's keyed card produces

Step 5 is the assertion that covers the tree: get the shape wrong and the
addresses differ even when every key is right.

Run it:  python3 restore_test_tr_pathological.py
Exit 0 means a card-only-plus-seeds restore reproduces the wallet exactly.
"""
import glob
import json
import os
import re
import subprocess
import sys

W = os.path.dirname(os.path.abspath(__file__))
C = "/scratch/code/shibboleth"
MD = C + "/descriptor-mnemonic/target/release/md"
MS = C + "/mnemonic-secret/target/release/ms"
IN = os.path.join(W, "inputs-pathological")
OUT = os.path.join(W, "out", "tr-pathological")


def run(cmd, **kw):
    r = subprocess.run(cmd, capture_output=True, text=True, **kw)
    if r.returncode != 0:
        sys.exit("FAILED (%d): %s\n%s" % (r.returncode, " ".join(cmd[:3]), r.stderr))
    return r.stdout


def derive(phrase, account):
    """The account xpub a restorer computes from a seed, via the shipped CLI."""
    out = run([MS, "derive", "--template", "bip48-p2wsh", "--account",
               str(account), "--phrase", "-"], input=phrase)
    d = dict((k.strip(), v.strip())
             for k, v in (l.split(":", 1) for l in out.splitlines() if ":" in l))
    return d["master_fingerprint"], d["account_xpub"]


def main():
    cards = [l for l in open(os.path.join(OUT, "md1-template.txt")) if l.startswith("md1")]
    cards = [c.strip() for c in cards]
    if not cards:
        sys.exit("no keyless template card set — run transcript_tr_pathological.sh first")

    # 1. the origins the CARD states, which is what a restorer actually holds.
    inspect = run([MD, "inspect"] + cards)
    origins = dict(
        (int(m.group(1)), m.group(2))
        for m in re.finditer(r"^\s*@(\d+): (?:\[[0-9a-f]{8}/)?([^\]\n]+)\]?$",
                             inspect, re.M))
    template = next(l.split(": ", 1)[1].strip()
                    for l in inspect.splitlines() if l.startswith("template: "))
    if not origins:
        sys.exit("the card set states no per-key origins — nothing to restore from")

    # Map fingerprint -> seed by deriving each master.
    seeds = {}
    for f in sorted(glob.glob(IN + "/seeds/master-*.seed")):
        phrase = open(f).read().strip()
        fp, _ = derive(phrase, 0)
        seeds[fp] = phrase

    # The fingerprint each slot belongs to comes from the key files' headers;
    # the card carries paths, and BIP-380 origins pair a fingerprint with them.
    fps = {}
    for f in sorted(glob.glob(IN + "/keys/key-*.xpub")):
        txt = open(f).read()
        idx = int(re.search(r"@(\d+)", txt).group(1))
        fps[idx] = re.search(r"\[([0-9a-f]{8})/", txt).group(1)

    print("Restoring %d slots from the engraved template + the seeds:" % len(origins))
    recovered = {}
    for idx in sorted(origins):
        path = origins[idx]
        acct = int(re.findall(r"(\d+)'", path)[2])   # m/48'/0'/ACCOUNT'/2'
        fp = fps[idx]
        if fp not in seeds:
            sys.exit("@%d: no seed for master %s" % (idx, fp))
        got_fp, xpub = derive(seeds[fp], acct)
        recovered[idx] = (got_fp, xpub)
        print("  @%-2d %s at %-16s -> %s…" % (idx, fp, path, xpub[:24]))

    # 3. Put the origins BACK into the template, then re-encode.
    #
    # `md inspect`'s `template:` line is the bare shape -- `@0/<0;1>/*` -- while
    # the engraved card also carries each slot's origin in its TLV, listed
    # separately under `origins:`. A restorer holds both, so rebuilding the
    # annotated form (`@0/48'/0'/0'/2'/<0;1>/*`) uses only what the card states.
    # Encoding the bare shape instead loses the origins and `md address` then
    # refuses with "non-canonical wrapper requires explicit origin for @0" --
    # which is how this step announced that it had dropped them.
    annotated = template
    for idx in sorted(origins):
        annotated = annotated.replace(
            "@%d/<0;1>/*" % idx,
            "@%d/%s/<0;1>/*" % (idx, origins[idx].lstrip("m/")))
    if "@0/<0;1>/*" in annotated:
        sys.exit("could not re-annotate the template with the card's origins")

    args = [MD, "encode", annotated, "--group-size", "0", "--force-chunked"]
    for idx in sorted(recovered):
        fp, xpub = recovered[idx]
        args += ["--key", "@%d=%s" % (idx, xpub), "--fingerprint", "@%d=%s" % (idx, fp)]
    rebuilt = [l for l in run(args).splitlines() if l.startswith("md1")]
    if not rebuilt:
        sys.exit("the recovered keys did not re-encode to a card set")

    # 4/5. addresses from the REBUILT card vs the journey's keyed card.
    def addrs(cardset, chain):
        return run([MD, "address"] + cardset + ["--chain", str(chain), "--count", "2"]).split()

    keyed = [l.strip() for l in open(os.path.join(OUT, "md1-keyed.txt")) if l.startswith("md1")]
    ok = True
    for chain, label in ((0, "receive"), (1, "change")):
        want = [a for a in addrs(keyed, chain) if a.startswith("bc1")]
        got = [a for a in addrs(rebuilt, chain) if a.startswith("bc1")]
        same = want == got
        ok = ok and same
        print("\n  %s addresses" % label)
        for w, g in zip(want, got):
            print("    %s  %s" % ("MATCH    " if w == g else "DIFFERENT", g))
        if not same:
            print("    engraved card: %s" % want)
            print("    restored card: %s" % got)

    print()
    if not ok:
        sys.exit("RESTORE FAILED — the rebuilt wallet spends to different addresses.\n"
                 "  Every key can be right and this still fail: the taptree shape\n"
                 "  is part of the address, and that is what this test exists for.")
    print("RESTORED: %d of %d slots, and the rebuilt wallet spends to the SAME\n"
          "addresses as the engraved card — so the taptree shape survived too."
          % (len(recovered), len(origins)))


if __name__ == "__main__":
    main()
