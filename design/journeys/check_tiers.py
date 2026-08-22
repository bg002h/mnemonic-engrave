#!/usr/bin/env python3
"""Gate the journey's PROSE tier table against the POLICY it claims to describe.

WHY THIS EXISTS. The four-tier table in the transcript's section 1 reads like the
source of the wallet. It is not. The arrow runs the other way:

    operator's English spec
      -> a policy string HAND-WRITTEN once (hashlock-vault cycle, f39958b)
      -> copied byte-for-byte into design/fixtures/…/tr.policy (5b15b18)
      -> `sed` path substitution -> inputs-rcw/policy-{tr,wsh}.txt
      -> and, separately, a PROSE DESCRIPTION of it typed into the transcript

So the table is downstream of the policy and nothing tied them together. It was
checked once, by hand, and found accurate — which is exactly the state every
other unverified claim in this repo was in before it turned out to be wrong.

A description that cannot drift is worth more than one that happens to be right,
so this asserts each tier against the real string, per wrapper.

    python3 check_tiers.py inputs-rcw/policy-tr.txt tr
    python3 check_tiers.py inputs-rcw/policy-wsh.txt wsh

Exit 0 iff every claim holds. Non-zero names the first that does not.
"""
import re
import sys

H1 = "ede0b28a805feca09a77c48f82babc1249c79aa840de94fa4a85bf55a453c813"
H2 = "9ce09a8df1d1f8bc8892441a9eb30d0a18bc7db81bb0fb3f54c6d9064e7b76ad"
H3 = "4743d7c47df21d29e3ed3dfec5d0c0a884ccc2708637dddf771c36d214056954"


def leaves(pol):
    """Return the four OUTERMOST `and_v(...)` groups — the spend branches.

    Works for BOTH shapes: `tr()`'s taptree `{a,{b,{c,d}}}` and `wsh()`'s
    `or_i(a,or_i(b,or_i(c,d)))`.

    Outermost matters and a regex cannot do it. Tier 2 is a NESTED and_v —
    `and_v(v:older(32768),and_v(v:sha256(H2),multi_a(2,...)))` — so a
    single-level pattern grabs the INNER group and reports the branch as
    missing its timelock. That is exactly the false FAIL this function was
    first written with, which is a fair warning about describing structure
    with patterns instead of balancing brackets.
    """
    spans = []
    for m in re.finditer(r"and_v\(", pol):
        i = m.end() - 1
        depth = 0
        for j in range(i, len(pol)):
            if pol[j] == "(":
                depth += 1
            elif pol[j] == ")":
                depth -= 1
                if depth == 0:
                    spans.append((m.start(), j + 1))
                    break
    # Keep only spans not contained in another — the outermost branches.
    outer = [(a, b) for (a, b) in spans
             if not any(x < a and b <= y for (x, y) in spans if (x, y) != (a, b))]
    outer.sort()
    return [pol[a:b] for a, b in outer]


def main():
    if len(sys.argv) != 3 or sys.argv[2] not in ("tr", "wsh"):
        sys.exit(f"usage: {sys.argv[0]} <policy-file> <tr|wsh>")
    pol = open(sys.argv[1]).read().strip()
    wrap = sys.argv[2]
    # multi_a is tapscript-only; the wsh form carries plain multi. Asserting the
    # RIGHT one per wrapper is itself a check — a wsh policy containing multi_a
    # would not be a valid wallet, and a tr one containing multi would be a
    # different script.
    m = "multi_a" if wrap == "tr" else "multi"

    claims = [
        ("tier 1", "@0+@1+@2 and sha256(H1), NO timelock",
         lambda l: f"{m}(3," in l and H1 in l and "older(" not in l and "after(" not in l
         and all(f"@{i}/" in l for i in (0, 1, 2))),
        ("tier 2", "@3+@4 and sha256(H2) after older(32768) [relative]",
         lambda l: f"{m}(2," in l and H2 in l and "older(32768)" in l
         and all(f"@{i}/" in l for i in (3, 4))),
        ("tier 3", "@5 alone after after(1173520) [absolute]",
         lambda l: "after(1173520)" in l and "pk(@5/" in l
         and "sha256" not in l and "multi" not in l),
        ("tier 4", "sha256(H3) ALONE after after(1383520) — NO KEY",
         lambda l: "after(1383520)" in l and H3 in l and "@" not in l and "pk(" not in l),
    ]

    ls = leaves(pol)
    if len(ls) != 4:
        sys.exit(f"FAIL[{wrap}]: parsed {len(ls)} spend branches, expected 4")

    bad = []
    for (name, desc, pred), leaf in zip(claims, ls):
        if pred(leaf):
            print(f"  {wrap} {name}: OK   {desc}")
        else:
            print(f"  {wrap} {name}: FALSE {desc}")
            print(f"      branch as written: {leaf[:110]}")
            bad.append(name)

    # The keyless tier is the wallet's defining property and the reason every
    # encode carries --experimental. Assert it exists EXACTLY once, so neither
    # losing it nor gaining a second one passes quietly.
    keyless = [l for l in ls if "@" not in l and "pk(" not in l]
    if len(keyless) != 1:
        print(f"  {wrap}: {len(keyless)} keyless branches, expected exactly 1")
        bad.append("keyless-count")

    if bad:
        sys.exit(f"FAIL[{wrap}]: the transcript's tier table does not match the "
                 f"policy: {', '.join(bad)}")
    print(f"  {wrap}: tier table matches the policy, and exactly one tier is keyless")


if __name__ == "__main__":
    main()
