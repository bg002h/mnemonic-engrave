#!/usr/bin/env python3
"""hashlock-notice-classify.py -- count the CARD lines and the NOTICE lines
above the card/notice boundary in `ms hashlock`, so F-536's enumeration is a
command rather than a number somebody wrote by hand.

WHY THIS EXISTS. F-536 is the follow-up tracking hazard notices still scoped to
the engraving card. Its enumeration has been WRONG TWICE:

  * first version (R0 round 8, I-3): enumerated by grepping `WARNING:` rather
    than by the rule, so it missed the two notices that do not carry that word
    -- including the sharpest, a DATA-LOSS line -- and then closed with "these
    three are the remaining instances of the class, already located."
  * second version (R0 round 9): claimed "seventeen of them, eight are card
    content and five are notices". 8 + 5 = 13. The stated total did not
    reconcile with itself, and one notice was still missing.

Both are the same defect: an enumeration ASSERTED instead of COUNTED, inside the
follow-up whose whole job is to say what has not been handled. So: run this,
paste its output, never retype the numbers.

THE RULE IT APPLIES (stated at the boundary in the source itself):
  CARD   -- content the operator transcribes or engraves. `--no-engraving-card`
            may suppress it.
  NOTICE -- a hazard in what was just emitted. Suppressing the card is not
            consent to lose it, so it belongs BELOW the boundary.

WHAT IT DOES NOT DO. It does not judge: the NOTICE_MARKS list below is the
human classification, and this script only makes applying it repeatable and the
arithmetic checkable. A line added above the boundary that is a hazard and is
not in that list will be counted as card content and this script will not
notice. Re-read the list when the card changes.

Usage:  scripts/hashlock-notice-classify.py [path/to/ms-cli/src/cmd/hashlock.rs]
Exit:   0 always -- this is a measurement, not a gate.
"""
import re
import sys

DEFAULT = "/scratch/code/shibboleth/ms-worktrees/hashkinds-p2/crates/ms-cli/src/cmd/hashlock.rs"

# A hazard in what was just emitted. Matched on CONTENT, never on line number:
# an earlier attempt keyed on line numbers and silently matched nothing after a
# one-line shift, reporting 0 notices.
NOTICE_MARKS = [
    "THIS RECORD CARRIES THE PHRASE",          # a file on disk holds the phrase
    "Never use this phrase as a passphrase",   # the operator's other accounts
    "brainwallet construction",                # the method is guessable
    "72 days on one GPU",                      # the phrase is too short
    "publishes these 32 bytes in the clear",   # --hex: another secret goes public
    "the only copy until you cut the plate",   # --random: DATA LOSS
]

def main() -> int:
    path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT
    src = open(path).read()
    try:
        start = src.index("    if !args.no_engraving_card {")
        end = src.index("    // ─── NOTICES THAT SURVIVE")
    except ValueError:
        print("boundary markers not found -- did the card block move?", file=sys.stderr)
        return 0
    block, base = src[start:end], src[:start].count("\n") + 1

    rows = []
    for m in re.finditer(r'writeln!\(\s*stderr,\s*\n?\s*"((?:[^"\\]|\\.)*)', block):
        line = base + block[: m.start()].count("\n")
        text = m.group(1).replace("\\\n", "").strip()
        rows.append((line, text, any(k in text for k in NOTICE_MARKS)))

    for line, text, is_notice in rows:
        print(f"{'NOTICE' if is_notice else 'card  '} {line}: {text[:64]}")

    notices = sum(1 for *_, n in rows if n)
    cards = len(rows) - notices
    print(f"\nTOTAL {len(rows)}   card {cards}   notices {notices}")
    # The check that would have caught both wrong versions of F-536.
    print(f"arithmetic reconciles: {cards + notices == len(rows)}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
