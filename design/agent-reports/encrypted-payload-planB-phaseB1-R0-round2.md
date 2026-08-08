# Plan B Phase B1 — R0 round 2 (sonnet), scoped to Task 4a

**Artifact:** Task 4a only. **Primary object of review: `git diff 6b63826..633e2e3`.**

**Brief:** one question — did the round-1 fold to Task 4a fix the finding, and did it
introduce a new defect. Every other task declared out of scope; round 0's four findings,
the label rule (verified against vectors D and G in round 1), and all 43+5 citations
declared settled. The brief said plainly that a clean result was the expected outcome and
that findings must not be manufactured to look thorough.

**Verdict: 0C / 1I / 0M / 0N.** The ordering fix is SOUND. The Important is against a
paragraph the round-1 fold added *beyond* the finding it was responding to.

---

## Report, verbatim

FOLD: PARTIAL — the round-1 ordering fix (pass 1/2 unchanged, grouping computed only afterward over already-admitted records, `groupCards` called once) is correct and matches `AdmitSection`/`decodePublicSet`/`cardKey`'s real control flow, but the fold's own newly-added "SectionPublic only" paragraph introduces a new, evidenced-false claim about what encrypted-section records can be.

## Verification of the ordering fix (sound — no finding)

- `AdmitSection` (`seal/record.go:158-191`, seedhammer): pass 1 (`firstUpperASCII`) and pass 2 (`Classify`/`permitted`) run unchanged in the per-record loop building `out`; the `if section == SectionPublic { ... decodePublicSet(strs) }` block runs only after that loop completes. The fold's ordering claim matches this exactly.
- Fail-closed unreachability: `permitted(section, c)` (`seal/record.go:147-153`) returns `true` for `SectionPublic` **only** when `c == ClassMDMK` — confirmed by reading the function body directly. So every record that survives pass 2 in `SectionPublic` is `ClassMDMK` by construction, and `cardKey`'s `default` branch is genuinely unreachable at the point grouping runs. This is the round-1 finding's core safety claim, and it holds.
- Implementability of "widen `decodePublicSet`'s return type, or have `AdmitSection` call `groupCards` itself": both are mechanically straightforward — `decodePublicSet` already computes `keys, groups, err := groupCards(records)` locally (`seal/record.go:310`) and only needs to also return them; `AdmitSection` already builds the `strs []string` needed by `groupCards` at the same call site. No structural obstacle from the `[][]byte` vs `[]string` mismatch — `strs` already exists at exactly the right point.
- Per-HRP card ordering is deterministic: `groupCards`'s `keys []groupKey` is built by iterating the input slice in order and appending on first-seen (`seal/record.go:234-240`) — not from map iteration. Two devices given the same input order produce the same `keys` order. Q3's nondeterminism worry does not apply.

## New finding

### [I] The fold's new "SectionPublic only" paragraph misstates what encrypted-section records can be, contradicting NORMATIVE spec text and the plan's own test vectors
**Where:** Task 4a, the paragraph beginning "These fields are populated for `SectionPublic` only" (new in this fold, `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB1.md`).
**Claim:** "...an encrypted-section `AdmittedRecord` carries `HRP == 0` and zero indices. That is correct for B1, which never sees a secret record — but B2 labels secret plates too (§10.2.2), and its records can be `ms1` or a bare mnemonic, **neither of which is a card at all**."
**Reality:**
- SPEC §6.3 (NORMATIVE), `design/SPEC_encrypted_payload_delivery.md:520-521`: "The encrypted section may carry anything — `ms1`, `mk1`, `md1`, a BIP-39 mnemonic — since it is confidential by construction." `md1`/`mk1` are exactly the card formats.
- `permitted()` (`seal/record.go:147-153`) codes this: `ClassMDMK` returns `true` unconditionally, regardless of section — it is not gated on `SectionPublic`. A `mk1`/`md1` (card) record is just as admissible in `SectionEncrypted` as in `SectionPublic`.
- Live fixtures already in the plan's own test corpus confirm this is not a theoretical edge case: `seal/testdata/vectors.json` vector C's `secret` array is `[ms1, mk1, mk1, md1, md1, md1]` and vector F's is `[ms1, ms1, ms1, mk1×6, md1×6]` — i.e. 12 of vector F's 15 "secret" records are `ClassMDMK` cards, and Task 4a's own "Tests" paragraph names vector F for label-row testing.
**Consequence:** The paragraph is attempting to name a trap for B2 ("B2 must not assume these fields are meaningful for secrets") but names the wrong one. It tells B2 that no secret-section record is ever groupable ("neither of which is a card at all"), when `mk1`/`md1` cards routinely land in the encrypted section per spec and per existing fixtures. A B2 implementer trusting this text will build secret-plate labeling with no card-grouping path at all, then hit vector F/C and discover encrypted `mk1`/`md1` records need exactly the same `mk1 1/2` / `mk1 2/3 · 1/2` labeling Task 4a built for public — except `AdmitSection`'s Pass 3 (`decodePublicSet`/`groupCards`) never runs for `SectionEncrypted`, so B2 has no grouping to reuse and is pushed toward the "second classifier in the UI" divergence Task 4a itself warns against two paragraphs earlier. It also leaves the (unchanged) `HRP` field doc comment — "HRP is 'd' (md1) or 'k' (mk1) for ClassMDMK records, 0 otherwise" — silently false for encrypted-section `ClassMDMK` records, which get `HRP == 0` despite being `ClassMDMK`.
**Fix:** Correct the paragraph to state that the SectionPublic-only scoping means `HRP`/`CardIndex`/etc. are zero for **every** encrypted-section record regardless of its classification — including `mk1`/`md1` — not because those records aren't cards, but because Pass 3 (the only place grouping is computed) never runs for `SectionEncrypted`. Flag for B2, as an owned follow-up: it will need its own §6.3-style grouping over the encrypted section's `ClassMDMK` subset, reusing `groupCards`/`cardKey` (same-package, already proven correct by this fold's ordering fix) rather than re-deriving classification in `gui`.

VERDICT: 0C / 1I / 0M / 0N
