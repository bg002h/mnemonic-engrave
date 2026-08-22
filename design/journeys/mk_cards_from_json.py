#!/usr/bin/env python3
"""Extract the mk1 strings from `mk encode --keys --json` into a flat file.

A tiny script rather than an inline `python3 -c` in the transcripts, because
that form has now broken twice on the same thing: an f-string containing
escaped quotes, nested inside single-quoted shell, inside a heredoc. Both times
the failure was a SyntaxError at run time, not at review time.

Usage:  mk_cards_from_json.py <mk-encode.json> <out.txt>
Prints a one-line summary; exits non-zero if the JSON has no cards.
"""
import json
import sys


def main(argv):
    if len(argv) != 3:
        sys.exit("usage: " + argv[0] + " <mk-encode.json> <out.txt>")
    doc = json.load(open(argv[1]))
    cards = doc.get("cards", [])
    if not cards:
        sys.exit("FATAL: " + argv[1] + " contains no cards")
    total = 0
    with open(argv[2], "w") as out:
        for card in cards:
            for s in card["mk1_strings"]:
                out.write(s + "\n")
                total += 1
    print(str(doc.get("card_count", len(cards))) + " key cards -> "
          + str(total) + " mk1 chunks")


if __name__ == "__main__":
    main(sys.argv)
