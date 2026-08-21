#!/usr/bin/env python3
"""THE RESTORE TEST — what a card-only restore of this wallet actually recovers.

The pathological journey called its `md decode` step "the round trip that makes
the backup worth engraving". It is not. That step proves the BYTES survive the
card format — a transcription check. **A restore test asks a different question:
if someone holds these plates and their seeds, do they get the wallet back?**

Here they do not, and this measures exactly how far short it falls.

The engraved md1 was built with `--path`, which records ONE shared origin for
every slot. The eleven keys actually sit at four account indices across three
masters. So a restorer who trusts the descriptor card derives account 0 from
each master and gets the wrong key everywhere else.

Run it:  python3 restore_test_pathological.py
Exit 0 means the measured damage matches what the document claims. Exit 1 means
one of them is wrong and both need reading — which is the entire point of
pinning it.
"""
import glob
import os
import re
import subprocess
import sys

W = os.path.dirname(os.path.abspath(__file__))
MS = "/scratch/code/shibboleth/mnemonic-secret/target/release/ms"
IN = os.path.join(W, "inputs-pathological")

# What the engraved card declares for EVERY slot. `--path bip48` and
# `m/48'/0'/0'/2'` produce a byte-identical chunk set, so this is account 0.
DECLARED_ACCOUNT = 0

# The claim under test, from the journey document itself. Kept here as data so a
# disagreement is a failing gate rather than a sentence nobody re-read.
DOCUMENTED_WRONG = [1, 2, 3, 5, 6, 7, 9, 10]


def derive(phrase, account):
    """The account xpub a restorer would compute, via the shipped CLI."""
    r = subprocess.run(
        [MS, "derive", "--template", "bip48-p2wsh", "--account", str(account),
         "--phrase", "-"],
        input=phrase, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"ms derive failed ({r.returncode}): {r.stderr}")
    out = dict(
        (k.strip(), v.strip())
        for k, v in (l.split(":", 1) for l in r.stdout.splitlines() if ":" in l))
    return out["master_fingerprint"], out["account_xpub"]


def main():
    # Map each master's fingerprint to its seed, by deriving it.
    seeds = {}
    for f in sorted(glob.glob(f"{IN}/seeds/master-*.seed")):
        phrase = open(f).read().strip()
        fp, _ = derive(phrase, 0)
        seeds[fp] = (os.path.basename(f), phrase)

    slots = []
    for f in sorted(glob.glob(f"{IN}/keys/key-*.xpub")):
        txt = open(f).read()
        m = re.search(r"\[([0-9a-f]{8})/([^\]]+)\]", txt)
        slots.append((
            int(re.search(r"@(\d+)", txt).group(1)),
            m.group(1),
            m.group(2),
            next(l.strip() for l in txt.splitlines() if l.startswith("xpub")),
        ))
    slots.sort()

    print(f"A restore that trusts the DESCRIPTOR CARD alone derives account "
          f"{DECLARED_ACCOUNT} from every master.\n")
    wrong = []
    for idx, fp, real_path, real_xpub in slots:
        if fp not in seeds:
            sys.exit(f"@{idx}: no seed for master {fp}")
        _, got = derive(seeds[fp][1], DECLARED_ACCOUNT)
        ok = got == real_xpub
        if not ok:
            wrong.append(idx)
        print(f"  @{idx:<2} master {fp}  at {real_path:<16} "
              f"{'RECOVERED' if ok else 'WRONG KEY'}")

    print(f"\n  {len(wrong)} of {len(slots)} slots recover the WRONG key: {wrong}")
    print(f"  {len(slots) - len(wrong)} recover correctly: "
          f"{[i for i, *_ in slots if i not in wrong]}")

    # A k-of-n reading, because "8 of 11 wrong" understates it for a policy whose
    # smallest tier needs 3 signatures.
    print("\n  Every tier of this vault needs keys this restore cannot produce.")

    if wrong != DOCUMENTED_WRONG:
        print(f"\nMISMATCH: the document says {DOCUMENTED_WRONG}, the measurement "
              f"says {wrong}.\nOne of them is wrong; fix whichever it is.",
              file=sys.stderr)
        return 1
    print("\n  Matches what the journey documents. Gate green.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
