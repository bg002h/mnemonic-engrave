# Brief — F-630 plan round 3: did the r2 fold land, and is the plan IMPLEMENTABLE as written?

Plan at `692d86fa`: `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`.
Prior rounds (settled, not reopened): `design/agent-reports/f630-plan-r0-opus.md`
(1C/7I/4M/2N, all fixed), `…-r0-fold-verify.md` (NEW-1 Critical, fixed),
`…-r2-boundary-verify.md` (0C/3I).

## Two questions

**A. Did the r2 fold land?** For each of r2's three Importants (Q1 evasion by
string literal, Q2 false-PASS vacuity, Q4 `md/` outside the boundary), say
FIXED / PARTIAL / NOT-FIXED and quote the plan text.

**B. Is the plan implementable exactly as written — the lens no round has used
yet.** Walk T1 → T2 → T3 → T4 as an implementer who may not invent anything.
At each step: does the plan make every decision the step needs, or is there a
choice left open where two readings produce materially different code? Name
each gap as *"at T<n>, an implementer must decide X, and the plan does not say"*.

Specifically worth testing:

1. **D5e's one exemption.** The plan exempts
   `md/compose_vectors_pin_test.go:158` via a `//go:vectortier vendored`
   marker. Build the scan and check: are there other occurrences among the 16
   in code that ALSO must not be routed, which the plan silently assumes are
   safe to route? Route them and run the affected tests. `md/compose_pkh_emit_test.go:37`
   and `md/compose_stubs_test.go:15` are the ones to look at hardest — what do
   they pair the record with, and does a pin-preferring record change their
   result once T2 adds three `forkbuilt/` records?
2. **The token scan.** Implement it with `go/scanner` as D5b/D5e describe and
   run it on the tree at `95716e97`. Does it find exactly the occurrences the
   plan predicts (18 total, 2 in comments, 16 in code)? If the count differs,
   the plan's number is wrong and T2 cannot be checked against it.
3. **D5c's self-test.** The plan requires the gate to flag a synthetic
   known-bad. Is "feed a known-bad string to the same matcher" actually
   sufficient, or can the matcher pass its self-test and still miss a real
   file (e.g. the self-test exercises the matcher but not the file-walk that
   feeds it)?
4. **T1's gate needs a card AND a record** per D2b. After T2, for the three
   pinned vectors, does `md`'s new `vectorRecordFor` exist in time for T1?
   T1 comes first and D5's loaders arrive at T2 — is T1 implementable at T1,
   or does it depend on T2?
5. **T3's literals.** The plan says update `composeVectorNames` (36) and the
   `176`. After widening to `^(keyed_|compose_)` minus `compose_refusal_`,
   what are the two correct new numbers? Compute them; the plan does not state
   them and an implementer must not guess.

## Already established — do not re-derive

- The fork already emits the 0.44.0-correct header; F-630 steps 3 and 4 are
  unnecessary/done.
- Drift: 46 keyed records → 2 identical, 41 descriptor-only (82 chain
  entries), 3 semantic (F-529's). 88 across all 44. The gate reds 44.
- The three split card/record sites and the four non-intersecting readers are
  verified, with their line numbers and vector names.
- `md/testdata/forkbuilt/` holds five `.md1.txt` and no records.
- 284/284 bracket fingerprints agree with `ExpandedKey.Fingerprint`.

## Out of scope

Re-verifying earlier rounds' findings. Style and wording. Proposing the plan do
more than it must to be executable and correct. Reviewing an implementation —
none exists.

## Rules of evidence

Reproduce, do not prescribe. Run commands in the fork; do NOT modify tracked
files; delete scratch files and verify both trees clean. A finding must name
the state and the wrong outcome, or the decision an implementer cannot make.

## Output

Severity per project standard. **FINAL ACTION: write the report to
`design/agent-reports/f630-plan-r3-implementability.md`** and return ONLY a
one-paragraph summary, that path, the A-verdicts, and C/I/M/N counts.
