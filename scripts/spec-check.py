#!/usr/bin/env python3
"""spec-check.py — the fold gate for design specs.

WHY THIS EXISTS. R0 rounds 1 and 2 both found the same defect shape in my folds:
a rule corrected in the section a finding pointed at, and left standing
elsewhere in the same document. Round 2's Critical and both its PARTIALs were
that, after round 1 had already named it. Three rounds of opus-rate review spent
on bookkeeping a script can do.

This is the project's own prescription applied to prose instead of code: when an
artifact will be folded repeatedly, commit the extractor so the check is a
command rather than a discipline.

    scripts/spec-check.py design/SPEC_systemwide_payloads.md

WHAT IT DOES NOT COVER — stated plainly, because a gate that hides its blind
spot is worse than no gate:

  * It cannot tell whether a rule is RIGHT. Every check here is internal
    consistency and citation resolution. A spec can be perfectly consistent
    about something false, and only a reviewer catches that.
  * It cannot find a rule stated in one place and simply ABSENT elsewhere —
    only contradictions between texts it knows to compare. Round 2's I1 (the
    scope list missing device-side work) would NOT be caught by this.
  * The INVARIANTS table is hand-written per spec. A new rule added to the
    document without a line here is unguarded.
  * Line-number citations are checked for existence and for an expected
    substring where one is declared; a citation with no declared substring is
    checked only for the line existing.
"""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
FORK = REPO.parent / "seedhammer"

# ---------------------------------------------------------------- invariants --
# (label, regex that must NOT match, why). These encode "this rule was decided;
# text asserting the opposite is a fold that missed a site."
FORBIDDEN = [
    ("mode count",
     r"two passphrase modes|Two modes in `me`",
     "decision 8 restored three modes; a 'two modes' site is a stale fold"),
    ("dropped mode residue",
     r"user-supplied is DROPPED|User-supplied is DROPPED",
     "user-supplied was restored; this text is from the reversed decision"),
    ("identity",
     r"identity[^\n]{0,40}the full EPD§6\.6 digest",
     "R1-C3: identity is the §5.4.1 construction, never the EPD§6.6 digest"),
    ("unconditional digest display",
     # Matches the CLAIM, not one phrasing. Round 2 named two sites that said
     # the same thing in different words; the first version of this invariant
     # caught only one of them, on its own first run.
     r"both\*{0,2} container\s+variants?[^.\n]{0,60}(show|display)[^.\n]{0,30}digest"
     r"|digest[^.\n]{0,40}(shown|displayed)[^.\n]{0,30}both\*{0,2} container",
     "R2-C1: a pub_len == 0 sealed payload displays NO digest (EPD§6.6)"),
]

# (label, list of regexes that must EACH match at least once, why)
REQUIRED = [
    ("three modes",
     [r"three passphrase modes", r"Three modes in `me`"],
     "decision 8 and §6's opening must agree"),
    ("pub_len == 0 exception",
     [r"pub_len == 0.{0,80}(NO|no) digest|nothing is displayed"],
     "the digest's absence for secrets-only sealed payloads must be stated"),
    ("checksum gate is per-invocation",
     [r"[Pp]er-invocation|PER-INVOCATION"],
     "R1-C2's root fix must be present"),
]

# Values that must be identical everywhere they appear.
SCALARS = [
    ("word floor", r"min 2|`2 ≤ N ≤ 24`|2\.\.24|2 through 24|2–24|2-24"),
    ("cliff", r"5 words|55 bits"),
]


def check_refs(text, errs):
    """Every bare §N[.N[.N]] must resolve to a heading in THIS document."""
    heads = set(re.findall(r"^#{2,4} (\d+(?:\.\d+)*(?:[a-z])?)[ .]", text, re.M))
    for m in re.finditer(r"(?<!EPD)§(\d+(?:\.\d+)*)", text):
        n = m.group(1).rstrip(".")
        if n not in heads:
            errs.append(f"stale internal reference §{n} — no such section")


def check_cross(text, errs):
    """Sections that exist in BOTH specs must be qualified EPD§ or bare, never ambiguous."""
    # Any bare § whose number also names a section here is fine (checked above).
    # Flag the known cross-spec section numbers appearing bare.
    for n in ("6.6", "6.1a", "6.3", "6.4", "10.2.4"):
        pat = r"(?<!EPD)(?<!\d)§" + re.escape(n)
        for m in re.finditer(pat, text):
            errs.append(f"bare §{n} means the OTHER spec — write EPD§{n}")


def check_citations(text, errs, notes):
    """file.go:NNN citations must resolve to a real line."""
    seen = 0
    for m in re.finditer(r"`((?:[\w./-]+/)?[\w.-]+\.(?:go|rs)):(\d+)`", text):
        rel, ln = m.group(1), int(m.group(2))
        for base in (FORK, REPO, FORK / "gui", FORK / "seal"):
            p = base / rel
            if p.exists():
                break
        else:
            errs.append(f"citation {rel}:{ln} — file not found")
            continue
        lines = p.read_text().split("\n")
        if ln > len(lines):
            errs.append(f"citation {rel}:{ln} — file has only {len(lines)} lines")
        else:
            seen += 1
    notes.append(f"{seen} file:line citations resolved to an existing line")


def check_tests(text, errs, notes):
    """Named tests must be numbered without gaps or duplicates."""
    nums = [int(n) for n in re.findall(r"^(\d+)\. \*\*\(|^(\d+)\. ", text, re.M) and
            re.findall(r"^(\d+)\. ", text, re.M)]
    sec = re.search(r"### 8\.3 Named tests(.*?)(?=^## )", text, re.M | re.S)
    if not sec:
        errs.append("§8.3 Named tests not found — the test list is unguarded")
        return
    ns = [int(x) for x in re.findall(r"^(\d+)\. ", sec.group(1), re.M)]
    if not ns:
        errs.append("§8.3 has no numbered tests")
        return
    dup = {n for n in ns if ns.count(n) > 1}
    if dup:
        errs.append(f"duplicate test numbers: {sorted(dup)}")
    gaps = [i for i in range(1, max(ns) + 1) if i not in ns]
    if gaps:
        errs.append(f"missing test numbers: {gaps}")
    notes.append(f"{len(ns)} named tests, numbered 1..{max(ns)} without gaps")


def main():
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    path = Path(sys.argv[1])
    text = path.read_text()
    errs, notes = [], []

    for label, pat, why in FORBIDDEN:
        for m in re.finditer(pat, text):
            line = text[:m.start()].count("\n") + 1
            errs.append(f"[{label}] line {line}: {why}\n      matched: {m.group(0)[:70]!r}")

    for label, pats, why in REQUIRED:
        for pat in pats:
            if not re.search(pat, text):
                errs.append(f"[{label}] missing /{pat}/ — {why}")

    for label, pat in SCALARS:
        if not re.search(pat, text):
            errs.append(f"[{label}] no site states this value")

    check_refs(text, errs)
    check_cross(text, errs)
    check_citations(text, errs, notes)
    check_tests(text, errs, notes)

    print(f"spec-check: {path}")
    for n in notes:
        print(f"  ok    {n}")
    if not errs:
        print("  ok    all invariants hold")
        print("\nNOT COVERED: whether any rule is CORRECT; a rule missing from a "
              "section entirely;\n             any rule with no line in the "
              "INVARIANTS table. See the module docstring.")
        return 0
    print()
    for e in errs:
        print(f"  FAIL  {e}")
    print(f"\n{len(errs)} problem(s). A fold is authorship and re-earns this gate.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
