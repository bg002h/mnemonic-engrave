#!/usr/bin/env python3
"""Split a concrete descriptor across TEXT plates, and say what that costs.

The device caps a QR at dim 37 (QR v5, ~106 bytes at ECC-L), and this wallet's
concrete descriptor is 1298 bytes — 12x that. It does not fit one QR and it does
not fit one 300-char text plate either, so it goes on several TEXT plates.

WHAT THIS LOSES, stated plainly because the plates cannot say it themselves:

  * md1 chunks carry a BCH checksum and a shared chunk-set-id, so a
    mis-transcribed character is DETECTED and a mixed-up set is refused.
  * These plates carry neither. They are raw text split at a byte offset. A
    single wrong character produces a descriptor that is still syntactically
    plausible and describes a DIFFERENT wallet.

So this is the fallback, not the backup. The md1 card set is the backup; this is
a human-readable convenience for a coordinator that wants a descriptor pasted in.

Usage:  split_descriptor_plates.py <descriptor.txt> <out-dir> [budget]
"""
import hashlib
import os
import sys

BUDGET = 300  # me's conservative single-plate text budget


def main(argv):
    if len(argv) not in (3, 4):
        sys.exit("usage: " + argv[0] + " <descriptor.txt> <out-dir> [budget]")
    desc = open(argv[1]).read().strip()
    out = argv[2]
    budget = int(argv[3]) if len(argv) == 4 else BUDGET
    os.makedirs(out, exist_ok=True)

    # Reserve room for the "part i/N" header the plate carries.
    header_room = 16
    body = budget - header_room
    parts = [desc[i:i + body] for i in range(0, len(desc), body)]
    n = len(parts)
    digest = hashlib.sha256(desc.encode()).hexdigest()

    print("descriptor: %d bytes -> %d TEXT plates (%d-char budget, %d for the header)"
          % (len(desc), n, budget, header_room))
    print("sha256:     %s" % digest)
    print("  NOTE: these plates carry NO checksum. md1 does; this does not.")
    for i, p in enumerate(parts, 1):
        path = os.path.join(out, "descriptor-part-%d-of-%d.txt" % (i, n))
        with open(path, "w") as f:
            f.write("part %d/%d\n%s\n" % (i, n, p))
        print("  part %d/%d  %3d chars" % (i, n, len(p)))

    # A rejoin check, so the split is proven reversible here rather than assumed
    # at transcription time.
    rejoined = "".join(parts)
    if rejoined != desc:
        sys.exit("FATAL: the split does not rejoin to the original descriptor")
    if hashlib.sha256(rejoined.encode()).hexdigest() != digest:
        sys.exit("FATAL: rejoined digest mismatch")
    print("  rejoin check: %d parts -> %d bytes, sha256 matches" % (n, len(rejoined)))


if __name__ == "__main__":
    main(sys.argv)
