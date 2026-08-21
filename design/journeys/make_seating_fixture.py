#!/usr/bin/env python3
"""Mint the fixture for the key-card seating walk.

A KEYLESS template, one mk1 key card per slot, and the addresses the HOST
derives for the seated wallet — everything from the conformance corpus, so the
expected values have the primary Rust implementation behind them rather than
this script.

    python3 make_seating_fixture.py

Writes out/seating/seating-fixture.json, which capture_seating.py reads.
"""
import json
import os
import subprocess
import sys

W = os.path.dirname(os.path.abspath(__file__))
C = "/scratch/code/shibboleth"
MD = f"{C}/descriptor-mnemonic/target/release/md"
MK = f"{C}/mnemonic-key/target/release/mk"
V = f"{C}/seedhammer/md/testdata/vectors"
OUT = os.path.join(W, "out", "seating")
VECTOR = "keyed_tr_with_leaf"


def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"{' '.join(cmd[:3])}… failed ({r.returncode}):\n{r.stderr}")
    return r.stdout


def main():
    os.makedirs(OUT, exist_ok=True)
    rec = json.load(open(f"{V}/{VECTOR}.conformance.json"))
    xpub = {k["index"]: k["xpub"] for k in rec["keys"]}
    template_text = open(f"{V}/{VECTOR}.template").read().strip()

    # The KEYLESS template: the same policy text with no --key at all.
    tmpl = [l for l in run([MD, "encode", template_text,
                            "--group-size", "0", "--force-chunked"]).splitlines()
            if l.startswith("md1")]

    # The stub every key card must carry. Taken from `md inspect` rather than
    # from `mk encode --from-md1`, which CANNOT read a current md1 — it refuses
    # with "wire-format version mismatch: got 9, expected 4" because mk vendors
    # an older md-codec (F-127). When that is fixed this should go back to
    # --from-md1 and stop hardcoding a derived value.
    ins = run([MD, "inspect"] + tmpl)
    tid = next(l.split(": ", 1)[1].strip() for l in ins.splitlines()
               if l.startswith("wallet-descriptor-template-id:"))
    stub = tid[:8]

    # One card per slot, each declaring its own origin.
    origins = {}
    for line in ins.splitlines():
        line = line.strip()
        if line.startswith("@") and ":" in line:
            slot, path = line.split(":", 1)
            origins[int(slot[1:])] = path.strip().strip("[]")
    if not origins:
        sys.exit("md inspect reported no origins; cannot build key cards")

    cards = []
    for i in sorted(origins):
        path = origins[i]
        if not path.startswith("m/"):
            path = "m/" + path
        cards.append([l.strip() for l in run([
            MK, "encode", "--xpub", xpub[i], "--origin-path", path,
            "--origin-fingerprint", "73c5da0a", "--policy-id-stub", stub,
            "--group-size", "0"]).splitlines() if l.strip().startswith("mk1")])

    # PRESENT THEM IN REVERSE. Seating is by declaration, never by gather order,
    # and a fixture that happens to be in slot order would not notice if that
    # stopped being true.
    cards.reverse()

    # What the host derives — from the KEYED form of the same policy, which is
    # the wallet the seating must reconstruct.
    keyargs = []
    for i in sorted(xpub):
        keyargs += ["--key", f"@{i}={xpub[i]}"]
    addrs = []
    for chain in ("0", "1"):
        flag = ["--change"] if chain == "1" else []
        addrs += [l for l in run([MD, "address", "--template", template_text]
                                 + keyargs + flag + ["--count", "2"]).splitlines()
                  if l.startswith("bc1")]

    # THE ID THE SCREEN WILL SHOW IS THE TEMPLATE ID, not the policy id, and
    # that is correct rather than a limitation. The gathered card IS a keyless
    # template; its authoritative identity is the key-STABLE template id, and
    # `FormAwareIdChunks` returns exactly that. Showing the key-DEPENDENT policy
    # id after seating would claim the card carries keys it does not — the
    # engraved plate would still decode to a template, and an operator
    # comparing that id against the plate later would find it absent.
    #
    # What proves WHICH wallet the seating produced is the addresses, which is
    # why they are the substantive half of this fixture.
    pid = tid

    # A card that is PERFECTLY VALID and belongs to the wrong form of this
    # wallet: minted on the KEYED policy's stub instead of the template's.
    #
    # This is the refusal arm's card, and it is the realistic one. Corrupting
    # bytes does not produce a wrong-stub card — mk1 carries a BCH checksum, so
    # a mangled card is dropped at the SCANNER and never reaches seating at all
    # (measured: the gather tally stayed at 0 key cards). The case an operator
    # actually hits is a card made for the full-policy card, whose stub is the
    # POLICY id while a template expects the TEMPLATE id.
    keyed_ins = run([MD, "inspect"] + [
        l.strip().replace(" ", "") for l in open(f"{V}/{VECTOR}.phrase.txt")
        if l.strip().startswith("md1")])
    policy_id = next(l.split(": ", 1)[1].strip() for l in keyed_ins.splitlines()
                     if l.startswith("wallet-policy-id:"))
    first = sorted(origins)[0]
    wrong_path = origins[first]
    if not wrong_path.startswith("m/"):
        wrong_path = "m/" + wrong_path
    wrong = [l.strip() for l in run([
        MK, "encode", "--xpub", xpub[first], "--origin-path", wrong_path,
        "--origin-fingerprint", "73c5da0a", "--policy-id-stub", policy_id[:8],
        "--group-size", "0"]).splitlines() if l.strip().startswith("mk1")]

    fx = {"template": tmpl, "keyCards": cards, "policyId": pid,
          "addresses": addrs, "wrongStubCard": wrong}
    p = os.path.join(OUT, "seating-fixture.json")
    with open(p, "w") as f:
        json.dump(fx, f, indent=1)
    print(f"wrote {p}")
    print(f"  template {len(tmpl)} chunk(s), {len(cards)} key card(s) (reversed), "
          f"template id {pid}")
    for a in addrs:
        print(f"  {a}")
    print(f"  refusal card: {len(wrong)} chunk(s), stubbed on the POLICY id {policy_id[:8]}")


if __name__ == "__main__":
    main()
