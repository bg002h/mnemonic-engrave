# Lens-fold check — `4bbaa16` (both lens reports folded together)

Reviewer: mechanical fold-verification lens, sonnet, 2026-08-13. Scope: does
`4bbaa16` correctly and completely absorb `multisig-build-repair-plan-lens-adversarial.md`
(A1/A2 + A3–A5 minor) and `multisig-build-repair-plan-lens-failure-states.md`
(F1–F7 + M1/M2 minor), without introducing a new defect. Isolated diff:
`git diff c82574f..4bbaa16 -- design/IMPLEMENTATION_PLAN_multisig_build_repair.md`
(96 insertions / 4 deletions, single file). Both repos left clean.

## Verdict

**NOT clean — 2 Critical, 1 Important, 5 Minor (silently dropped).** The fold
correctly *addressed* all 9 major findings (A1, A2, F1–F7) in the sense of
adding on-point plan text, but repeats this author's documented failure mode
twice: A2's fix left the two locations the adversarial report actually cited
saying the opposite of the new paragraph, and the fold's own S4/S5
renumbering broke the plan's one numeric cross-reference. F1's fix is also
only two-thirds landed. All five Minors across both reports were dropped with
no acknowledgment.

## Per-finding table

| Finding | Fixed? | New defect? |
| --- | --- | --- |
| A1 (cosigner substitution / forgeable fp) | Yes — S5 gets a new block (lines ~510–520): show per-slot keys/digest on review, rewrite EXPERIMENTAL warning to demand key comparison and say a matching fp is not verification. Items 1–2 of the report's 3-item fix are in. | Item 3 (name SEALED as the recommended carrier for untrusted delivery, cross-ref SYSW§5.3) is **not** present — no "sealed" or "out-of-band" text anywhere in the file. Not a new defect; an incomplete absorption (see Minors/gaps below). |
| A2 (oracle pinned "by version", spoofable) | Partially — new paragraph added in §1a: "PIN BY SOURCE COMMIT, NOT BY `--version`." | **Critical.** See propagation sweep — the two locations A2 actually quoted (S0 deliverable 1, S0 gate line) are unchanged and still say "by version" / "prints oracle versions," and a third sentence two lines below the new paragraph says the literal opposite. |
| F1 (interrupted multi-plate tail) | Partially — S5 test 7 `TestReRunMintsByteIdenticalPlates` added, pinning the determinism property (fix part a). | Not a new defect, but incomplete: fix part (b) — "one sentence in the S6 gate: at least one hardware run interrupts a set and completes it by re-run" — is **absent from S6** (S6 still has only its original 3 items, unchanged by the diff). Fix part (c) — add an interrupted/resumed plate to a §4.5 walk — is mentioned as available tooling ("the `shToolpath` digest-equality check is the tool") but never made a walk requirement in S5's Gate line. **Important**: F1 is recorded as folded but two of its three concrete asks are missing. |
| F2 (ms1-first order + "discard") | Yes — new "Plate order and abort text — RULED" block in S5: public-first/secret-last ordering, DESTROY not discard for cut secret plates. | None found. |
| F3 (§4.2 scrub has no owner) | Yes — S4 test 8 `TestBuildFlowScrubsEverySeedOnEveryExit` added, matching the report's proposed name/mechanism. | None found in isolation, but see M2 interaction below (Minors). |
| F4 (partial walk vacuously passes total gate) | Yes — new §3 paragraph (applies to every stage, not just S5): expected artifact census derives from the recorded input tuple, "a partial walk may never satisfy a total gate." Placed globally rather than S5-only, which also covers S1's disjunctive-gate gap the report separately named. | None found. |
| F5 (duplicate-key window S2→S4) | Yes — S2 test 4 `TestS2RefusesDuplicateKeysBeforeS4` added exactly as specified. | None found. |
| F6 (under-supply refusal names "scan") | Yes — S1 test 7 `TestUnderSupplyRefusalNamesTheHostRoute` added, covering both the count refusal and the incomplete-chunk-set drop message in one test. | None found; no separate Implementation-bullet line drops "scan" from the message text, but the test's own description already states the requirement, so this is not scored as incomplete. |
| F7 (walk-away, unbounded seed retention) | Yes — new "Bound the walk-away" paragraph in S4: rules that S4 must either bracket with `wipeGuard` or record an explicit non-wiping decision in the restore doc. | None found. |

## Minors — addressed or dropped

None of the five Minors from either report appear anywhere in the fold, and
none is acknowledged as consciously deferred or declined (`grep -n "A3\|A4\|A5\b\|M1\b\|M2\b"` over the plan returns nothing):

- **A3** (rule out the over-supply "selection step" arm) — silently dropped.
- **A4** (`PublicDataHash` truncates record count to 1 byte) — silently dropped.
- **A5** (plaintext digest is only as strong as the delivery channel; fold one sentence) — silently dropped. Same theme as A1's unfolded item 3.
- **M1** (Back semantics of new S4/S5 screens unruled) — silently dropped.
- **M2** (ms1 rides as a Go `string`, unscrubbable; the scrub test must scope itself to scrubbable buffers and name the string residue as a blind spot) — silently dropped. This one interacts directly with the newly folded F3: the new S4 test 8 text ("assert zeroed on each exit class") does not carry M2's caveat, though the test's own stated scope ("every entered seed" via `buildMultisigSeedHook`) is about mnemonics/passphrases, not the derived ms1 legs, so it does not appear to overclaim as written — but the caveat the reviewer asked to be *named* is still absent.

All five are **Minor: recorded only** — they do not block, but "silently ignored" is itself the finding per the review brief.

## Propagation sweep

Ran `scripts/fold-propagation-check.sh` against the oracle-pin claim (the
script's own header cites this exact plan's history of this exact failure
mode — three of six prior folds were incomplete propagation, one being S3's
`TYPED-ONLY` count):

```
$ bash scripts/fold-propagation-check.sh design/IMPLEMENTATION_PLAN_multisig_build_repair.md \
    'resolves the primary toolchain[^.]*by version' \
    'print(s)? the (resolved )?oracle versions' \
    'the walk script must print the oracle versions'

== propagation check: IMPLEMENTATION_PLAN_multisig_build_repair.md ==
  gone   resolves the primary toolchain[^.]*by version
  LEFT   print(s)? the (resolved )?oracle versions
           63:**The walk script MUST print the oracle versions into every gate record**, so a
  LEFT   the walk script must print the oracle versions
           63:**The walk script MUST print the oracle versions into every gate record**, so a

   SUPERSEDED PHRASING SURVIVES -- the fold is not finished.
```

The first pattern reported "gone" only because it is a single-line ERE and the
source sentence wraps across two lines (189→190); grep matches per line, not
per logical sentence. Hand-sweep confirms it did NOT actually go:

```
$ grep -n -i "by version\|prints the resolved oracle versions\|print the oracle versions\|prints oracle versions" \
    design/IMPLEMENTATION_PLAN_multisig_build_repair.md
63:**The walk script MUST print the oracle versions into every gate record**, so a
190:   by version, refuses to run against vendored fork testdata, and **prints the
243:**Gate.** The three BIP vector tests pass; the harness prints oracle versions;
```

**Critical (A2 propagation).** Three surviving sites, none touched by the fold:

1. **Line 63** — two sentences after the new "PIN BY SOURCE COMMIT, NOT BY
   `--version`" paragraph, in the same section, the pre-existing sentence
   still reads: *"The walk script MUST print the oracle versions into every
   gate record."* Direct, adjacent self-contradiction.
2. **Lines 189–191 (S0 deliverable 1)** — this is the exact text A2 quoted as
   the defect ("*S0 deliverable 1 says the walk script 'resolves the primary
   toolchain **by version**' and 'prints the resolved oracle versions.'*").
   Untouched by the diff: *"The walk script resolves the primary toolchain by
   version, refuses to run against vendored fork testdata, and prints the
   resolved oracle versions plus the full input tuple into every gate
   record."*
3. **Line 243 (S0 Gate line)** — *"the harness prints oracle versions"* —
   also untouched.

The fold added a correct new paragraph but never edited the normative
deliverable text or the gate criterion an implementer actually builds
against — both still describe, and the gate line still checks for, the
insecure "by version" behavior the new paragraph forbids two lines above it.
A2 is recorded as folded; at the two sites the report actually cited, it is
not.

Hand-swept the remaining candidate phrasings named in the brief — plate
ordering, "discard", fingerprint-verification instruction, scrub
requirement, "scan" refusal language — no other superseded claim survives:

```
$ grep -n -i "discard"     design/IMPLEMENTATION_PLAN_multisig_build_repair.md   # 4 hits, all current/intentional (2 re: cosignerFromCard, 2 re: the new DESTROY-not-discard ruling itself)
$ grep -n -i "destroy"     design/IMPLEMENTATION_PLAN_multisig_build_repair.md   # 1 hit, the new ruling
$ grep -n -i "ms1.*first|first.*ms1|plate order" design/IMPLEMENTATION_PLAN_multisig_build_repair.md
  # "ms1-first order is inherited convention, not a ruling" -- correctly framed as historical
$ grep -n -i "fingerprint" design/IMPLEMENTATION_PLAN_multisig_build_repair.md   # all current; no site still tells the operator fp alone verifies
$ grep -n -i "scrub"       design/IMPLEMENTATION_PLAN_multisig_build_repair.md   # all in the new S4 test 8 block + the walk-away paragraph; no stale "no scrub owned" claim remains
$ grep -n -i "scan"        design/IMPLEMENTATION_PLAN_multisig_build_repair.md   # 4 hits: 2 pre-existing correct usages, 1 in the new S1 test 7, 1 for out-of-scope NFC (§4, deliberate)
$ grep -n "SEALED|sealed|out-of-band" design/IMPLEMENTATION_PLAN_multisig_build_repair.md
  # zero hits -- A1 item 3 / A5 never landed (recorded above as a Minor/incomplete-absorption gap, not a contradiction)
```

## Test-numbering consistency check

Sequential-numbering audit of every list the fold touched:

- **S1** (tests 1–7): sequential, no gaps/dupes. New test 7 appended at the end — no renumbering of prior items, so nothing downstream could break.
- **S2** (tests 1–5): new test 4 (`TestS2RefusesDuplicateKeysBeforeS4`) inserted before the old test 4 (raster assertion), which becomes test 5. Sequential, no gaps/dupes.
- **S4** (tests 1–9): new test 8 (`TestBuildFlowScrubsEverySeedOnEveryExit`) inserted before the old test 8 (`TestGateDerivesAtTheCardsOwnOrigin`), which becomes test 9. Sequential, no gaps/dupes — **but this is a renumbering, and the plan has exactly one place that cross-references an S4 test by number.**
- **S5** (tests 1–6, then 7–8 after the Implementation subsection — this split predates the fold): new test 7 (`TestReRunMintsByteIdenticalPlates`) inserted before the old test 7 (`TestGateStillFiresAfterOriginsDiverge`), which becomes test 8. Sequential, no gaps/dupes in isolation.

Cross-reference sweep (`grep -n "S1 test\|S2 test\|S3 test\|S4 test\|S5 test\|S6 test"`) finds exactly **one** hit in the whole plan: line 496, inside S5's (now) test 8:

> `TestGateStillFiresAfterOriginsDiverge` — **S4 test 8's fixture**, re-run through the REAL post-rewire flow...

**Critical (renumbering propagation).** Confirmed against the pre-fold tree
(`git show c82574f:design/...` — pre-fold, S4 test 8 = `TestGateDerivesAtTheCardsOwnOrigin`,
and the then-S5-test-7 correctly said "S4 test 8's fixture" pointing to it).
Post-fold, S4's insertion pushed `TestGateDerivesAtTheCardsOwnOrigin` to test
**9**, but this S5 cross-reference — itself renumbered from 7 to 8 by the
same commit — was carried over **verbatim** and still says "S4 test 8,"
which now names `TestBuildFlowScrubsEverySeedOnEveryExit` (the scrub test),
not the origin-binding test. This is exactly the class of defect the brief
flagged as highest-yield: the fold touched both lists in the same commit and
missed the one line that ties them together. Mitigating factor: the fixture
itself is redescribed inline in the same sentence ("the same `both` slot
declaring `m/48'/0'/1'/2'`"), so an implementer reading the whole paragraph
is not actually misled about what to build — but a reader using the number to
jump to S4 lands on the wrong test, and the citation is simply false.

## Findings

1. **[Critical] A2's fix does not reach the deliverable or the gate.** Lines
   189–191 (S0 deliverable 1) and line 243 (S0 gate line) — the two
   locations the adversarial report's A2 finding actually quoted — are
   unchanged and still specify/check "by version" / "prints oracle
   versions." Line 63, two sentences after the new corrective paragraph in
   the same section, restates the superseded MUST verbatim. An implementer
   building to the stated deliverable and gate would satisfy the insecure,
   already-refuted requirement, not the new one. **Fix:** edit lines
   189–191, 243, and 63 to match the new paragraph (source commit / hash,
   not version string) — the four locations must say the same thing.

2. **[Critical] Stale cross-reference: S5's "S4 test 8" now names the wrong test.** Line 496. S4's insertion (fixing F3) pushed the origin-binding
   test from S4-test-8 to S4-test-9; S5's insertion (fixing F1) pushed the
   citing test from S5-test-7 to S5-test-8, but did not update the number it
   cites. **Fix:** "S4 test 8's fixture" → "S4 test 9's fixture."

3. **[Important] F1 is only one-third landed.** Only fix part (a) — the
   determinism test in S5 — was folded. Fix part (b), an S6 hardware
   confirmation that a real interrupted set completes by re-run, is absent
   from S6 (unchanged 3-item list). Fix part (c), adding an
   interrupted/resumed plate to a §4.5 walk, is named as available tooling
   but never made a requirement of S5's Gate. **Fix:** add the S6 sentence
   and make the walk requirement explicit in S5's Gate line, or record that
   these two were deliberately deferred and why.

4. **[Minor, recorded only] Five Minors silently dropped with no
   acknowledgment:** A3 (rule out the selection-step arm), A4
   (`PublicDataHash` truncation), A5 (plaintext-digest channel caveat — same
   theme as A1's unfolded item 3 on SEALED), M1 (Back semantics of new S4/S5
   screens), M2 (ms1-as-Go-string scrub blind spot, which bears directly on
   the newly folded S4 test 8). None blocks; all should either be folded in
   a follow-up pass or explicitly recorded as declined so the next reader
   does not have to re-derive that they were seen and set aside.
