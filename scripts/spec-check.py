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
  * The INVARIANTS and SINGLE_DEF tables are hand-written and PHRASING-SHAPED.
    R0 round 4 measured SINGLE_DEF at 1 kill in 5 mutants. They are a safety
    net, not the mechanism. BARE is the mechanism: it forbids the bare term
    rather than trying to recognise a definition, so no wording evades it.
  * Line-number citations are checked for FILE EXISTENCE AND LINE RANGE ONLY.
    A citation redirected to an unrelated line in the same file passes. R0
    round 3 mutation-tested this and was right to: the previous version of this
    very docstring claimed "checked for an expected substring where one is
    declared", a mechanism check_citations has never implemented. That is a
    control documented by intent rather than behaviour -- the F-123 mistake --
    inside the tool written to catch it. EXPECT declares the substrings that
    ARE checked; anything not listed there gets existence only.
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


# Citations whose LINE CONTENT is pinned, not merely its existence. A citation
# not listed here is checked for file+range only -- see the docstring.
EXPECT = {
    "gui/derive_xpub.go:82": "func seedEntryFlow",
    "gui/gui.go:758": "refreshCands",
    "gui/unlock_kdf.go:168": "Valid",
    "gui/unlock_kdf.go:359": "Valid",
    "seal/open.go:76": "func NormalisePassphrase",
    "seal/wire.go:95": "func (h Header) Sealed",
    "seal/session.go:17": "ClassCodex32Secret",
    "gui/passphrase_keyboard.go:76": "func NewPassphraseKeyboard",
}


def check_citations(text, errs, notes):
    """file.go:NNN citations must resolve to a real line, and a pinned one must
    still contain what the spec says it does."""
    seen = pinned = 0
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
            continue
        seen += 1
        want = EXPECT.get(f"{rel}:{ln}")
        if want is not None:
            pinned += 1
            if want not in lines[ln - 1]:
                errs.append(f"citation {rel}:{ln} no longer contains {want!r} — "
                            f"line reads: {lines[ln-1].strip()[:60]!r}")
    notes.append(f"{seen} file:line citations resolved; {pinned} pinned to their content")


# ------------------------------------------------------- single definition --
# (term, regex matching a DEFINITIONAL phrasing). Every match must fall inside
# §12, which is the single source. A match outside it is a second definition --
# the defect that cost R0 rounds 1, 2 and 3, five partials each time.
SINGLE_DEF = [
    ("cliff",
     r"the cliff (is|means|=)\b"
     r"|cliff[^.\n]{0,30}\bif and only if\b"
     r"|(below|above) the cliff[^.\n]{0,25}(iff|means|is defined)"
     r"|cliff is (a |an )?\w+ (or more|threshold)"),
    ("compared",
     r"`?compared`? (is|flag is) (set|satisfied) by (EITHER|either|a|the)"),
    ("identity",
     r"[Tt]he identity is `?SHA-256|identity IS the"),
]


# ------------------------------------------------------------- bare terms --
# THE STRUCTURAL CHECK. R0 round 4 measured the phrasing-based check at 1 kill
# in 5 mutants: a regex over "definitional phrasings" can only catch the
# phrasings its author imagined, so it can never be structural.
#
# This does not try to recognise a definition. It makes restating one
# unwritable: outside §12, the term may appear ONLY in its reference form.
# No wording evades it, because it never inspects the wording.
BARE = [
    ("cliff", r"\bcliffs?\b", "[cliff]"),
    ("compared", r"`compared`", "[compared]"),
    ("identity", r"`identity`", "[identity]"),
]


def check_bare_terms(text, errs, notes):
    """Outside §12, a governed term may appear only as its [reference]."""
    m = re.search(r"^## 12\. ", text, re.M)
    if not m:
        errs.append("§12 missing — nothing owns the rules")
        return
    body = text[:m.start()]
    for term, pat, ref in BARE:
        masked = body.replace(ref, "\x00" * len(ref))
        for mm in re.finditer(pat, masked, re.I):
            line = masked[:mm.start()].count("\n") + 1
            errs.append(f"[{term}] line {line}: bare {mm.group(0)!r} outside §12 — "
                        f"write {ref}. Any prose about it is a second definition "
                        f"waiting to happen")
    notes.append(f"{len(BARE)} governed terms appear only as references outside §12")


def check_single_def(text, errs, notes):
    """A rule may be DEFINED in exactly one place: §12."""
    m = re.search(r"^## 12\. ", text, re.M)
    if not m:
        errs.append("§12 (normative definitions) missing — nothing owns the rules")
        return
    lo = m.start()
    for term, pat in SINGLE_DEF:
        outside = [mm for mm in re.finditer(pat, text, re.I) if mm.start() < lo]
        for mm in outside:
            line = text[:mm.start()].count("\n") + 1
            errs.append(f"[{term}] line {line}: a SECOND definition outside §12 — "
                        f"reference `[{term}]` instead\n      matched: {mm.group(0)[:60]!r}")
    notes.append(f"{len(SINGLE_DEF)} rules defined once, in §12")


def check_tests(text, errs, notes):
    """Named tests must be numbered without gaps or duplicates."""
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
    check_single_def(text, errs, notes)
    check_bare_terms(text, errs, notes)
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
