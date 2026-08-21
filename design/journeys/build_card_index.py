#!/usr/bin/env python3
"""Build card-index.txt from `mk encode --keys --json`, CHECKING attribution.

card-index.txt captions each engraved plate with the cosigner whose key is on
it. Getting it wrong is a silent wrong answer on a metal plate, and this journey
already has two incidents:

  1. The first version recorded per-key chunk COUNTS and walked them in key
     order. `me bundle` emits plates in chunk_set_id order, not key order, so
     all 30 captions named the wrong key.
  2. The version this replaces paired card block N with key record N and
     guarded it with a COUNT. A count is permutation-blind: an adversarial
     review swapped two key records and got a self-consistent 30-line index at
     exit 0 with five silently wrong captions, because the per-card chunk total
     is recomputed from the block and so a permutation self-heals into a
     plausible-but-false number. The PDF build did not catch it either -- its
     guard only raises on ABSENT strings.

`mk encode --json` now emits each card's own origin, so every row here is
ASSERTED rather than assumed: the card must name the same origin as the key
record it is being captioned with, or the build dies.

Usage:  build_card_index.py <mk-encode.json> <keys-meta.txt> <card-index.txt>
  keys-meta.txt is `<key-number> <fingerprint> <path>` per line, in the same
  order as the --keys file that produced the JSON.
"""

import json
import sys


def main(argv):
    if len(argv) != 4:
        sys.exit(f"usage: {argv[0]} <mk-encode.json> <keys-meta.txt> <out>")
    _, json_path, meta_path, out_path = argv

    cards = json.load(open(json_path))["cards"]
    meta = [line.split() for line in open(meta_path) if line.strip()]

    if len(cards) != len(meta):
        sys.exit(f"FATAL: {len(cards)} cards for {len(meta)} key records")

    rows = []
    for card, record in zip(cards, meta):
        ki, fp, path = record
        got_fp = card["origin_fingerprint"]
        got_path = card["origin_path"]
        if got_fp != fp or got_path != path:
            sys.exit(
                f"FATAL: card captioned as key {ki} claims origin "
                f"[{got_fp}/{got_path}] but the key record says "
                f"[{fp}/{path}] -- attribution is WRONG"
            )
        strings = card["mk1_strings"]
        total = len(strings)
        for nth, s in enumerate(strings, 1):
            rows.append(f"{s} {ki} {nth} {total} {fp} {path}")

    with open(out_path, "w") as out:
        out.write("\n".join(rows) + "\n")
    print(f"card-index: {len(rows)} rows, {len(cards)} cards, origins verified")


if __name__ == "__main__":
    main(sys.argv)
