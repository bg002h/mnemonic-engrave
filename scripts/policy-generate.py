#!/usr/bin/env python3
"""Generate wallet policies and compare what each implementation derives from them.

WHY THIS EXISTS
---------------
The md1 decoder is fuzzed four ways and the codec has property tests on its
primitives. The POLICY SPACE is untested: until this script, nothing built a
wallet policy that was not already in the vendored corpus, pushed it through the
whole seam -- compose, encode to md1, decode back, seat keys, derive an address
-- and asked two independent implementations whether they agreed.

The corpus is 67 vectors written by hand over months. This generates thousands,
and the interesting ones are the shapes nobody thought to write down.

WHAT IT COMPARES
----------------
Two implementations, both reached through the md1 round trip, so the codec is
inside the comparison rather than beside it:

  Rust   `md address <chunks>`  -- the primary
  Go     `cmd/policyprobe`      -- the device, through gui.policyAddressAt,
                                   the same router the inspect screen calls

A DISAGREEMENT -- both derive, different addresses -- is the finding this exists
to produce, and it is a funds-safety finding: the host and the device would show
an operator two different addresses for one engraved card.

An ASYMMETRIC REFUSAL -- one derives, the other declines -- is a smaller finding
and is counted separately. It is usually a deliberate difference in what each
side supports, and F-513 is one that turned out to be real.

WHY IT IS SEEDED
----------------
Every run is reproducible from `--seed`, and the manifest records the exact
`md compose` arguments for each case. A finding that cannot be reproduced by
hand is a rumour; this prints the command that makes it happen again.

WHAT IT DOES NOT COVER, stated plainly
--------------------------------------
  * Bitcoin Core is not consulted here. Two implementations agreeing is weaker
    than three, and both of these descend from the same specification. The Core
    leg lives in the differential driver.
  * Only the ADDRESS is compared. Two policies can derive the same address and
    differ in the script they would actually spend from, though for the
    wrappers here the address commits to the script.
  * Generation is over the COMPOSER's language, so it reaches the policies an
    operator can build on the device and not the wider set `md encode` accepts.
    A shape the composer cannot express cannot be generated here.
"""

import argparse
import json
import os
import random
import subprocess
import sys

WRAPPERS = ["wsh", "tr", "sh-wsh", "sh"]

# A digest with no known preimage, so a generated policy is never accidentally
# spendable by anyone reading this file.
SHA256_DIGESTS = [
    "b867db8781e4b0ea1dcdc1f1a4b1e6b09c4f6e1b8e6c4e0e0d4a1c6f2b3e5a7d",
    "a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad",
    "5f2c8e1d9a3b7c4e6f0d2a8b1c5e9f3a7d4b6c8e0f2a4b6c8d0e2f4a6b8c0d2e",
]

LOCKS = [
    None,
    "older=144",
    "older=65535",
    "after=800000",
    "after=1000000",
]

# The values a boundary lives on, all of which the composer ACCEPTS -- verified
# by running each one. The ones it rejects (older=0, older=65536, after=0,
# after=500000000 without the `t`, after=499999999t) are not here: a generator
# that spends its run watching the composer refuse measures the composer's
# argument parser, which has its own tests, and not the seam.
#
# The `u` and `t` suffixes are the interesting half. `older=Nu` sets BIP-68's
# type flag, so older=1u lowers to older(4194305), not older(1) -- a 22-bit
# shift that an independent reimplementation has every chance of getting wrong,
# and that no vendored vector exercises. `after=Tt` crosses BIP-65's
# height/time boundary at 500000000, where the same integer means two different
# things depending on which side of it you are on.
LOCKS_EDGE = [
    None,
    "older=1",            # the smallest relative height
    "older=65535",        # the largest
    "older=1u",           # -> older(4194305): type flag set, one unit
    "older=65535u",       # -> older(4259839): type flag set, the largest
    "after=1",            # the smallest absolute height
    "after=499999999",    # the largest height before it reads as a time
    "after=500000000t",   # the smallest time, the same integer the line above rejects
    "after=2147483647t",  # the largest time
]


def run(cmd, stdin=None):
    """Run a command and return (rc, stdout, first line of stderr)."""
    p = subprocess.run(cmd, input=stdin, capture_output=True, text=True)
    err = (p.stderr.strip().splitlines() or [""])[0]
    return p.returncode, p.stdout, err


def load_xpubs(vectors_dir):
    """Collect the distinct test xpubs the corpus already uses.

    Reusing the corpus keys rather than minting new ones keeps a generated
    finding directly comparable to a vendored vector, and keeps this script from
    needing key derivation of its own -- one less implementation in a harness
    whose whole point is that implementations disagree.
    """
    seen = {}
    for name in sorted(os.listdir(vectors_dir)):
        if not name.endswith(".conformance.json"):
            continue
        try:
            with open(os.path.join(vectors_dir, name)) as fh:
                doc = json.load(fh)
        except (OSError, ValueError):
            continue
        for key in doc.get("keys") or []:
            xpub = key.get("xpub")
            if xpub:
                seen[xpub] = True
    return sorted(seen)


def make_policy(rng, max_slots, locks=LOCKS, max_paths=8, max_n=6):
    """Draw one policy: a wrapper and an ordered list of spend paths.

    Slot budget is tracked across paths because the composer's limit is on the
    TOTAL, and a generator that ignores it would spend most of its run watching
    `md compose` refuse.
    """
    wrapper = rng.choice(WRAPPERS)
    if wrapper in ("sh", "sh-wsh"):
        # The legacy wrappers hold one plain sorted multisig and nothing else,
        # so drawing locks and hashes for them only generates refusals. Biasing
        # here rather than filtering afterwards is what keeps a run's yield
        # honest: a `compose` refusal the generator could have predicted is
        # noise in the counts, not a measurement of anything.
        # n >= 2: a legacy wrapper holds a sorted MULTISIG, and n = 1 is not one.
        n = rng.randint(2, 4)
        return wrapper, ["%dof%d" % (rng.randint(1, n), n)], n
    paths = []
    slots = 0
    for _ in range(rng.randint(1, max_paths)):
        n = rng.randint(1, max_n)
        if slots + n > max_slots:
            break
        k = rng.randint(1, n)
        slots += n
        spec = "%dof%d" % (k, n)
        lock = rng.choice(locks)
        if lock:
            spec += "," + lock
        if rng.random() < 0.35:
            spec += ",sha256=" + rng.choice(SHA256_DIGESTS)
        paths.append(spec)
    if not paths:
        paths = ["1of1"]
        slots = 1
    return wrapper, paths, slots


def compose(md, wrapper, paths):
    """Lower the paths and return both template forms.

    --json is used rather than parsing stdout because the origin-less form is
    what `md decode` gives back, and comparing those two is the round-trip
    assertion. Reading it off stdout would mean re-deriving it by stripping
    origins here -- a third implementation of the thing under test, in the
    harness, which is the mistake this whole leg exists to avoid.
    """
    cmd = [md, "compose", "--wrapper", wrapper]
    for p in paths:
        cmd += ["--path", p]
    cmd += ["--json"]
    rc, out, err = run(cmd)
    if rc != 0:
        return None, err
    try:
        doc = json.loads(out)
    except ValueError as exc:
        return None, "unparseable compose json: %s" % exc
    if not doc.get("template_with_origins") or not doc.get("template"):
        return None, "compose json carried no template"
    return doc, None


def decode_template(md, chunks):
    """Ask the primary to read the card back, returning the origin-less template."""
    rc, out, err = run([md, "decode"] + chunks)
    if rc != 0:
        return None, err
    for line in out.splitlines():
        line = line.strip()
        if line and not line.startswith(("note:", "warning:", "md:")):
            return line, None
    return None, "decode produced no template"


def encode(md, template, xpubs, slots):
    cmd = [md, "encode", template]
    for i in range(slots):
        cmd += ["--key", "@%d=%s" % (i, xpubs[i % len(xpubs)])]
    cmd += ["--force-chunked"]
    rc, out, err = run(cmd)
    if rc != 0:
        return None, err
    chunks = [l.strip().replace(" ", "") for l in out.splitlines()
              if l.strip().replace(" ", "").startswith("md1")]
    if not chunks:
        return None, "encode produced no md1 strings"
    return chunks, None


def bundle(me, chunks, tmpdir):
    """Ask `me bundle` whether this card set can be ENGRAVED.

    This is the leg between "the codec accepts it" and "a plate exists". A
    policy can encode cleanly and still be unmintable -- the 64-chunk wire cap
    (F-515) is exactly that shape -- and the operator meets that wall after
    choosing the keys and the shape, which is the worst moment to meet it.

    Exit codes are read from the process, never through a pipe: `me bundle`
    returns 4 on an incomplete set and 0 on a good one, and a `| tail` between
    them reports tail's 0 for both.
    """
    path = os.path.join(tmpdir, "bundle-in.txt")
    with open(path, "w") as fh:
        fh.write("\n".join(chunks) + "\n")
    rc, _, err = run([me, "bundle", "--in", path])
    return (rc == 0), err


def rust_addresses(md, chunks, count):
    cmd = [md, "address"] + chunks + [
        "--network", "mainnet", "--index", "0", "--count", str(count), "--json",
    ]
    rc, out, err = run(cmd)
    if rc != 0:
        return None, err
    try:
        doc = json.loads(out)
    except ValueError as exc:
        return None, "unparseable json: %s" % exc
    addrs = doc.get("addresses") or []
    return [a["address"] if isinstance(a, dict) else a for a in addrs], None


def probe_batch(fork, cases):
    """Run the whole batch through one policyprobe process.

    One process, not one per case: the tool is a co-process by design and flushes
    each result as it is produced.
    """
    env = dict(os.environ)
    env["PATH"] = "/scratch/code/shibboleth/.toolchain/go/bin:" + env.get("PATH", "")
    env.setdefault("TMPDIR", "/scratch/code/shibboleth/.tmp")
    proc = subprocess.run(["go", "run", "./cmd/policyprobe"],
                          input="\n".join(json.dumps(c) for c in cases),
                          capture_output=True, text=True, cwd=fork, env=env)
    if proc.returncode != 0:
        sys.exit("policyprobe failed: %s" % proc.stderr.strip()[:300])
    out = {}
    for line in proc.stdout.splitlines():
        doc = json.loads(line)
        out[doc["id"]] = doc
    return out


def corpus_cases(vectors_dir, indices):
    """Every vendored vector, as probe cases, in a stable order."""
    cases = []
    for name in sorted(os.listdir(vectors_dir)):
        if not name.endswith(".phrase.txt"):
            continue
        vid = name[: -len(".phrase.txt")]
        chunks = []
        with open(os.path.join(vectors_dir, name)) as fh:
            for line in fh:
                line = line.strip().replace(" ", "")
                if line.startswith("md1"):
                    chunks.append(line)
        if chunks:
            cases.append({"id": vid, "chunks": chunks,
                          "indices": list(range(indices))})
    return cases


def run_corpus(args, probe_results, cases):
    """Compare each vendored vector's verdict against the recorded baseline.

    WHY A BASELINE AND NOT A COUNT. A run that only prints "45 derivable" passes
    while one vector starts deriving and another stops, which is the drift most
    worth catching -- the counts are equal and the device has changed its mind
    about two policies. So the baseline is PER VECTOR, and the diff names which
    ones moved and in which direction.

    It is a file that must be edited deliberately, like a golden test, because
    the alternative is a gate that rewrites its own expectations and therefore
    cannot fail.
    """
    observed = {}
    for case in cases:
        cid = case["id"]
        dev = probe_results.get(cid)
        if dev is None:
            observed[cid] = {"device": "MISSING"}
            continue
        if dev.get("panic"):
            observed[cid] = {"device": "panic"}
            continue
        if not dev.get("ok"):
            observed[cid] = {"device": dev.get("stage", "?")}
            continue
        rust, _ = rust_addresses(args.md, case["chunks"], args.indices)
        if rust is None:
            observed[cid] = {"device": "ok", "rust": "refused"}
        elif rust == dev["receive"]:
            observed[cid] = {"device": "ok", "rust": "agrees"}
        else:
            observed[cid] = {"device": "ok", "rust": "DISAGREES"}
    return observed


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--count", type=int, default=200, help="policies to generate")
    ap.add_argument("--seed", type=int, default=1, help="RNG seed; a run is reproducible from it")
    ap.add_argument("--md", default="/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md")
    ap.add_argument("--me", default="/scratch/code/shibboleth/mnemonic-engrave/target/debug/me",
                    help="path to the `me` binary for the engrave leg")
    ap.add_argument("--no-bundle", action="store_true",
                    help="skip the engrave leg (`me bundle`)")
    ap.add_argument("--fork", default="/scratch/code/shibboleth/seedhammer",
                    help="fork checkout holding cmd/policyprobe")
    ap.add_argument("--indices", type=int, default=2, help="addresses per chain to compare")
    ap.add_argument("--manifest", default=None, help="write every case's compose args here")
    ap.add_argument("--corpus", action="store_true",
                    help="run the VENDORED vectors instead of generating, and diff "
                         "each one's verdict against the recorded baseline")
    ap.add_argument("--baseline",
                    default="/scratch/code/shibboleth/mnemonic-engrave/design/policy-corpus-baseline.json",
                    help="per-vector baseline for --corpus")
    ap.add_argument("--write-baseline", action="store_true",
                    help="with --corpus, OVERWRITE the baseline with what was observed. "
                         "Deliberate, never automatic: a gate that rewrites its own "
                         "expectations cannot fail.")
    ap.add_argument("--edges", action="store_true",
                    help="draw locks from the BOUNDARY set (BIP-68 type flag, BIP-65 "
                         "height/time split) instead of ordinary values")
    args = ap.parse_args()

    vectors = os.path.join(args.fork, "md", "testdata", "vectors")
    xpubs = load_xpubs(vectors)
    if not xpubs:
        sys.exit("no xpubs found in %s" % vectors)
    max_slots = len(xpubs)

    if args.corpus:
        return corpus_main(args, vectors)

    rng = random.Random(args.seed)
    cases, manifest = [], {}
    gen_refused = {}
    for i in range(args.count):
        cid = "gen-%05d" % i
        wrapper, paths, slots = make_policy(
            rng, max_slots, locks=LOCKS_EDGE if args.edges else LOCKS)
        composed, err = compose(args.md, wrapper, paths)
        template = composed["template_with_origins"] if composed else None
        if composed is None:
            gen_refused.setdefault("compose: " + err[:60], 0)
            gen_refused["compose: " + err[:60]] += 1
            continue
        chunks, err = encode(args.md, template, xpubs, slots)
        if chunks is None:
            gen_refused.setdefault("encode: " + err[:60], 0)
            gen_refused["encode: " + err[:60]] += 1
            continue
        cases.append({"id": cid, "chunks": chunks,
                      "indices": list(range(args.indices))})
        manifest[cid] = {"wrapper": wrapper, "paths": paths, "slots": slots,
                         "template": template,
                         "template_origin_less": composed["template"]}

    if not cases:
        sys.exit("every generated policy was refused before it reached a comparison")

    device = probe_batch(args.fork, cases)

    agree = disagree = roundtrip_broken = bundle_refused = 0
    tmpdir = os.environ.get("TMPDIR", "/tmp")
    device_only = rust_only = both_refused = 0
    findings = []
    for case in cases:
        cid = case["id"]
        dev = device.get(cid)
        if dev is None:
            findings.append((cid, "DROPPED", "policyprobe returned no line for this case"))
            continue
        if dev.get("panic"):
            findings.append((cid, "PANIC", dev.get("error", "")))
            continue
        # The ROUND TRIP, checked before the addresses: does the card decode
        # back to the template it was encoded from? Two policies deriving the
        # same address is much weaker evidence than two templates being equal,
        # and an encoder that dropped a lock would still produce an address
        # both sides agree on -- they would simply agree about the wrong wallet.
        if not args.no_bundle:
            ok, berr = bundle(args.me, case["chunks"], tmpdir)
            if not ok:
                bundle_refused += 1
                findings.append((cid, "BUNDLE-REFUSED", berr))

        back, derr = decode_template(args.md, case["chunks"])
        want = manifest[cid]["template_origin_less"]
        if back is None:
            findings.append((cid, "ROUNDTRIP-UNREADABLE", derr))
        elif back != want:
            roundtrip_broken += 1
            findings.append((cid, "ROUNDTRIP",
                             "encoded %s | decoded %s" % (want, back)))

        rust, rerr = rust_addresses(args.md, case["chunks"], args.indices)
        dev_ok = bool(dev.get("ok"))
        rust_ok = rust is not None
        if dev_ok and rust_ok:
            if dev["receive"] == rust:
                agree += 1
            else:
                disagree += 1
                findings.append((cid, "DISAGREE",
                                 "device %s | rust %s" % (dev["receive"], rust)))
        elif dev_ok and not rust_ok:
            device_only += 1
            findings.append((cid, "RUST-REFUSED", rerr))
        elif rust_ok and not dev_ok:
            rust_only += 1
            findings.append((cid, "DEVICE-REFUSED", dev.get("error", "")))
        else:
            both_refused += 1

    print("generated %d, compared %d (seed %d%s)" % (
        args.count, len(cases), args.seed, ", EDGES" if args.edges else ""))
    if gen_refused:
        print("refused before comparison:")
        for reason, n in sorted(gen_refused.items(), key=lambda kv: -kv[1]):
            print("  %4d  %s" % (n, reason))
    print("agree            %d" % agree)
    print("DISAGREE         %d" % disagree)
    print("device only      %d  (rust refused)" % device_only)
    print("rust only        %d  (device refused)" % rust_only)
    print("both refused     %d" % both_refused)
    print("ROUNDTRIP broken %d  (card does not decode to what it encoded)" % roundtrip_broken)
    if not args.no_bundle:
        print("BUNDLE refused   %d  (encodes, but `me bundle` will not mint it)" % bundle_refused)

    panics = [f for f in findings if f[1] == "PANIC"]
    if panics:
        print("PANICS: %d" % len(panics))

    shown = {}
    for cid, kind, detail in findings:
        if kind in ("DISAGREE", "PANIC", "DROPPED", "ROUNDTRIP", "ROUNDTRIP-UNREADABLE"):
            print("  %s %s: %s" % (kind, cid, detail))
            print("    md compose --wrapper %s %s" % (
                manifest[cid]["wrapper"],
                " ".join("--path %s" % p for p in manifest[cid]["paths"])))
        else:
            shown.setdefault(kind + ": " + detail[:70], []).append(cid)
    for key, ids in sorted(shown.items(), key=lambda kv: -len(kv[1])):
        print("  %4d  %s  (e.g. %s)" % (len(ids), key, ids[0]))

    if args.manifest:
        with open(args.manifest, "w") as fh:
            json.dump(manifest, fh, indent=1, sort_keys=True)
        print("manifest: %s" % args.manifest)

    # A disagreement or a panic is a failure of the SYSTEM, not of this script,
    # and the exit code says so, so a CI job can gate on it.
    return 1 if (disagree or panics or roundtrip_broken) else 0


def corpus_main(args, vectors):
    cases = corpus_cases(vectors, args.indices)
    if not cases:
        sys.exit("no vendored vectors under %s" % vectors)
    results = probe_batch(args.fork, cases)
    observed = run_corpus(args, results, cases)

    if args.write_baseline:
        with open(args.baseline, "w") as fh:
            json.dump(observed, fh, indent=1, sort_keys=True)
            fh.write("\n")
        print("baseline written: %s (%d vectors)" % (args.baseline, len(observed)))
        return 0

    try:
        with open(args.baseline) as fh:
            expected = json.load(fh)
    except OSError as exc:
        sys.exit("no baseline at %s (%s). Create it with --write-baseline once you "
                 "have checked the verdicts by hand." % (args.baseline, exc))

    moved, gone, appeared = [], [], []
    for cid, want in sorted(expected.items()):
        got = observed.get(cid)
        if got is None:
            gone.append(cid)
        elif got != want:
            moved.append((cid, want, got))
    for cid in sorted(observed):
        if cid not in expected:
            appeared.append(cid)

    counts = {}
    for v in observed.values():
        key = v["device"] if v["device"] != "ok" else "ok/" + v.get("rust", "?")
        counts[key] = counts.get(key, 0) + 1
    print("corpus: %d vectors" % len(observed))
    for key in sorted(counts):
        print("  %-16s %d" % (key, counts[key]))

    if not (moved or gone or appeared):
        print("every vector matches the baseline")
        return 0
    for cid, want, got in moved:
        print("  MOVED    %s: baseline %s, now %s" % (cid, want, got))
    for cid in gone:
        print("  GONE     %s: in the baseline, not in the corpus" % cid)
    for cid in appeared:
        print("  NEW      %s: in the corpus, not in the baseline" % cid)
    print("A MOVED line is the device or the primary changing its mind about a "
          "policy. Read it before rewriting the baseline.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
