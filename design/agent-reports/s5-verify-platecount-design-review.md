# R0 design review — S5 verify expected-plate-count fix (pre-implementation)

Reviewer: fable (dispatched adversarial gate). Date: 2026-08-15.
Subject: the PROPOSED design — `multisigVerifyFlow` gains a caller-supplied
expected plate count (build: `len(legs)`; supply: `1`), legs derived per
READ-BACK PLATE (plate → slot by key), count shortfall a named failure.
Code base examined: `/scratch/code/shibboleth/wt-s5` @ `f0006b7` (clean; left
clean — `git status --porcelain` empty at finish). No code was written or
mutated. All traces below were walked against the real source:
`gui/multisig_verify.go`, `gui/multisig.go:141-181`, `gui/multisig_build.go:325-358`,
`gui/multisig_build_tail.go`, `gui/multisig_match.go`, `gui/bundle.go:127-264`,
`gui/bundle_flow.go:143-411`, `gui/multisig_supply.go:61-79`,
`gui/multisig_verify_legs_test.go`, and the frozen plan §0 / S5
(`design/IMPLEMENTATION_PLAN_multisig_build_repair.md:1137-1298`).

Verdict up front: **NOT safe to implement as written — 1 Critical, 2 Important.**
The root diagnosis is right and the caller-supplied-expectation mechanism is the
correct family of fix. The specific mechanism — a bare COUNT plus a per-plate
pairing inversion — is a lossy projection of what the caller actually knows, and
the loss is exactly where the false GREENs live. The amendment (below) is small:
pass the engraved **slot set** instead of its cardinality, keep the existing
bijection untouched, and drop the per-plate inversion.

---

## Findings

### C1 — CRITICAL — a bare count validates cardinality, not identity: a substituted same-wallet plate at a different slot verifies OK. **CONFIRMED (traced).**

Design elements at fault: points 1–3 jointly (count + per-plate pairing).

The count answers "how many plates should exist"; it cannot answer "which
slots' plates were cut". The caller knows the second and the design throws it
away. Concrete trace, every step against real code:

- Wallet W is Trace-B-shaped: @0 = master A acct 0, @1 = master A acct 1,
  @2 = master B. A plate carrying @1's key exists from an earlier build run
  (or another device — the encoders are deterministic by design,
  plan S5 item 7, so any run of W mints byte-identical plates).
- Today the operator runs the SUPPLY flow with W's md1 and seed A.
  `findUserSlot` returns `reused=[0,1]`, ONE plate is cut — slot @0's key
  (`gui/multisig.go:141-149,172-173`). Verify is offered; the caller passes
  count = 1.
- At readback the operator presents the OLD @1 plate instead of the just-cut
  @0 plate. Gather yields 1 mk1 card. Count 1 == 1 → passes.
- Per-plate pairing: the plate's key sits at slot @1 of the read-back md1 →
  pairs to @1. Seed A fills @1 (`allUserSlots` → {0,1}), the leg derives at
  `keys[1].OriginPath`, and `bundle.Verify` PASSES — the plate is a genuine,
  honest plate of this wallet: right key, right origin, right md1, right stub.
- **Screen: "Verify OK". The plate the machine just engraved was never read.**

That is the brief's false-GREEN definition verbatim — "'Verify OK' over a
plate never checked, or a wrong plate". Both the pre-S5 verify (first-match
leg @0 finds no plate) and the current f0006b7 verify reject this readback;
the count design is the first version of this flow that would pass it.

The same weakness reappears in build if pairing is not explicitly injective:
expected 3, readback {@0-plate, @0-plate-variant, @2-plate} — 3 cards, count
satisfied, @1's plate missing. (The natural byte-identical duplicate is blocked
upstream — `bundleGatherer` dedupes by payload-derived `chunk_set_id`,
`gui/bundle.go:177-179` — but a same-key different-bytes card, e.g. a doctored
or stale-format plate, gathers as a distinct card.) Injectivity must be stated,
and even with it the supply trace above still passes.

**Fix (and it is smaller than the count design):** pass the engraved **slot
set**, not its size. Build: collect the `slot` loop variable in
`buildEngraveTail` (`gui/multisig_build_tail.go:77`, currently discarded) and
pass those indices. Supply: pass `[]int{idx}` from `findUserSlot`. In the
verify's derive loop, replace "every slot the seed fills" with "every EXPECTED
slot the seed fills" (`fresh = expectedSlots ∩ allUserSlots(seed)`), and leave
`verifyMultisigLegs` — the bijection — **byte-untouched**. Then: the false RED
dies (supply expects {0}, no leg is manufactured for @1); a lost plate still
fails, NAMING its slot (`errVerifyLegHasNoPlate`); the substituted-plate trace
above fails correctly (@0's leg finds no plate, and the @1 plate is unclaimed);
and every existing test in `multisig_verify_legs_test.go` keeps its subject.
This is the same session-truth provenance as the count — §7.4 is untouched
because the *proof* is still a leg re-derived from a re-typed seed; only the
*obligation list* comes from the engraver, which is the design's own thesis
("only the engraver knows what it cut").

### I1 — IMPORTANT — per-plate pairing loses the slot-naming guarantee the tests pin. **CONFIRMED (traced).**

Design element at fault: point 3 (derive per read-back plate).

`TestVerifyCoversEveryLeg` requires a wrong-plate failure to NAME the slot:
`multisig_verify_legs_test.go:152-156` — *"the failure %q does not name %s, so
the operator cannot tell WHICH plate to re-cut"*. Trace under the proposed
design: build, count 3, readback {@0, foreign, @2}. Count passes (3 == 3).
Plates @0 and @2 pair and verify; the foreign plate pairs to NO slot. The only
nameable object is the plate ("plate 2 belongs to no slot") — the fact that
**@1's plate is missing** is not derivable from a count, because the flow does
not know which slots had plates (build with a held subset, supply with one).
The verify still goes RED, so this is not a false GREEN — but a pinned,
operator-facing guarantee ("which plate do I re-cut" has an answer) silently
degrades, and the existing test would have to be weakened to ship it. Under the
C1 amendment the guarantee survives unmodified: leg @1 exists (expected slot),
finds no plate, `errVerifyLegHasNoPlate{1}` names it.

### I2 — IMPORTANT — only SHORTFALL is specified as a failure; unpairable, undecodable, duplicate-slot and EXCESS readbacks are unspecified, and the removed bijection is what currently catches all of them. **CONFIRMED design gap; failure trace PLAUSIBLE.**

Design element at fault: point 4 (and the silent removal of the unclaimed
sweep).

The design names one failure: fewer plates than expected. It says nothing
about: (a) a plate whose key sits at no slot; (b) a plate that does not decode;
(c) two plates pairing to one slot; (d) MORE plates than expected. Today all
four are structurally caught by the bijection (`errVerifyPlateUnclaimed`, the
claimed[] sweep, `verifyClaimPlate`'s skip-then-sweep contract at
`gui/multisig_verify.go:191-212`). The codebase's own precedent is the trap:
`verifyClaimPlate` deliberately SKIPS an undecodable plate *because the sweep
catches it* — an implementer following that precedent under a design whose only
named failure is a shortfall produces a verify where a corrupt or foreign extra
plate is skipped, the count arithmetic is satisfied, and the screen says
Verify OK. That is the "skipped gates are the default failure" class. If any
form of this design proceeds, every non-happy readback shape must have a named
outcome in the design text, not in the implementer's judgment. (Under the C1
amendment this finding dissolves: the bijection is retained verbatim, and
`errVerifyNoLegs`, the unclaimed sweep, and the undecodable-plate contract all
survive with their existing tests.)

### M1 — MINOR — a plate→slot matcher is a second copy of the funds-safety key comparison.

`allUserSlots` states its own law (`gui/multisig_match.go:47-51`): the
canonical (chainCode ‖ compressedPubkey) comparison exists at exactly ONE site,
"two copies of a funds-safety comparison is how the two come apart." Point 3's
plate→slot pairing needs that same comparison (plate xpub vs `keys[s].Xpub`),
which today exists only leg→plate (`verifyClaimPlate`, mk-to-mk). Whatever
design ships, the new matching site must route through a shared helper rather
than re-spelling the rule.

### M2 — MINOR — the supply announcement this fix leans on is factually false, and after the fix it becomes the operator's only explanation of the 1-plate expectation.

"This key is reused at slots @0 and @1; engraving the first (@0)"
(`gui/multisig.go:146-148`) — the keys are NOT reused; the SEED fills two slots
with two DIFFERENT keys at different origins (the measured root fact). Once
verify passes over one plate, this screen is the only thing telling the
operator why one plate covers two slots — and it tells them the wrong model
(@1's key is not on that plate; recovery of @1 rides on seed + md1, not on the
steel). `gui/multisig.go` is a flow the plan does not own: file the wording fix
as a follow-up with an owner, do not fold it silently into this work.

### M3 — MINOR — "plate count" is actually the mk1 CARD count, and a zero/absent expectation needs a stated refusal.

A chunked mk1 card spans several physical plates (`bundlePlatePlan`,
`gui/bundle_flow.go:348-363`); the thing counted on both sides is cards
(`extractReadbackMd1AndMk1s` returns `mk1s [][]string`). Operator-facing text
must keep the existing `plateWord` phrasing honest. And both call sites
guarantee expectation ≥ 1 today (`errBuildNoHeldSlot`; supply's constant), but
the design should state that an empty expectation refuses rather than
vacuously passing — the `errVerifyNoLegs` posture applied to the new
parameter.

---

## The attack list, answered case by case

Cases the current tests hold, walked under the design AS WRITTEN (count +
per-plate), then under the C1 amendment (slot set + untouched bijection):

| case | as written | amended |
| --- | --- | --- |
| honest full readback (Trace B, 3 plates) | PASS ✓ | PASS ✓ |
| supply reused-seed, 1 plate (the false RED) | PASS ✓ (fixed) | PASS ✓ (fixed) |
| wrong plate for a slot | RED, but cannot name the slot (I1) | RED naming the slot ✓ |
| right key, lying origin | RED ✓ (pairs by key, `bundle.Verify` fails Path) | RED ✓ |
| unclaimed extra plate | UNSPECIFIED (I2) | RED ✓ (sweep intact) |
| missing plate | RED via count ✓ | RED naming the slot ✓ |
| substituted same-wallet other-slot plate | **FALSE GREEN (C1)** | RED ✓ |
| duplicate byte-identical plate | unreachable (gather csid-dedupe) — count then shortfalls ✓ | same ✓ |
| same-key different-bytes second plate | UNSPECIFIED unless pairing injective (C1/I2) | RED ✓ (unclaimed) |
| undecodable plate | UNSPECIFIED (I2) | RED ✓ (skip-then-sweep contract intact) |
| zero legs / zero plates | gather refuses 0 plates; zero-leg guard must be re-specified (I2) | `errVerifyNoLegs` survives verbatim ✓ |
| aborted engrave (1 of 3 cut, `bundleEngrave` returns void, verify still offered) | shortfall named "1 of 3" — honest ✓ | leg-no-plate names the uncut slot — honest ✓ (optionally also the fast length precheck) |
| multi-seed loop / Verify Incomplete | arithmetic survives (plates-verified vs plates-read vs count) ✓ | arithmetic survives (slots-covered vs slots-expected) ✓ |

Count trustworthiness (question 2): the count's provenance is fine — same
session, no menu entry (both callers confirmed: `gui/multisig.go:181`,
`gui/multisig_build.go:356`), gather dedupe makes double-cut plates a shortfall
rather than an excess, and an aborted engrave produces an honest named
shortfall, which is the RIGHT screen. The count's problem is not trust; it is
information content (C1).

Pairing well-definedness (question 3): a key at several slots of one md1 is
ambiguous but benign only when the duplicate slots share an origin (identical
legs); the design must still fix a deterministic rule (lowest slot) and say so.
A doctored md1 declaring one xpub at two different origins makes the ambiguity
outcome-bearing; the amended design never pairs plate→slot, so the question
does not arise there.

Simpler/safer design (question 4): passing the expectation is right; a bare
count is the wrong shape for it — the slot set is the same provenance, the
same two call-site edits, strictly more information, and a smaller diff (the
bijection and its whole test file survive). The deeper alternative — make the
SUPPLY path engrave a plate per matched slot so the engrave and check rules
agree with no expectation channel at all — is coherent and arguably the better
long-term product (it would also make a future cold/menu-entry verify possible),
but it is a normative change to steel output in a flow this plan explicitly
does not own, it is NOT required to fix the defect, and it would need its own
gate. File it as a follow-up question together with M2; do not fold it in here.

Multi-seed loop (question 5): no termination or arithmetic break found in
either variant. One new message is needed under the amendment: a seed that
fills only NON-expected slots (e.g. cosigner B's seed after a supply engrave of
@0) is a third case distinct from "not a cosigner" and "already checked" —
"that seed's slots were not engraved in this run."

Frozen-plan compliance: no contradiction. The plan's S5 file-touch table
already lists `gui/multisig_verify.go` (signature change anticipated) and
`gui/multisig.go` (call-site-level touch for the S5 signature work);
`gui/multisig_build_tail.go` is S5-owned, so returning the held-slot indices is
in scope. The inert-dedupe revert (`verifyLegWithSameKey`, the false
"same key implies same origin" comment at `gui/multisig_verify.go:355-357`, the
tautological first assertion of `TestReusedKeyVerifiesAgainstItsONEPlate`) is
part of this work under either variant and is consistent with the plan.

---

## Verdict

**Needs specific amendments before any code — not safe as written (1C, 2I).**
The diagnosis (engrave rule vs check rule disagree; only the engraver knows
what it cut; the caller must supply the expectation) is sound and stands.
Amendments:

1. **Replace the count with the engraved slot set** (C1, and it resolves I1
   for free). Build: `buildEngraveTail` also returns the held slot indices it
   already iterates; supply: `[]int{idx}` from `findUserSlot`.
2. **Drop point 3's per-plate inversion.** Restrict the existing derive loop to
   `expectedSlots ∩ allUserSlots(seed)` and keep `verifyMultisigLegs` — the
   bijection and every named failure — byte-untouched (I1, I2).
3. Keep point 4's spirit as a **fast post-gather length precheck**
   (`len(readbackMk1s)` vs `len(expectedSlots)`, named both directions), with
   the per-slot bijection failures remaining the authoritative check (I2).
4. State the outcome of every non-happy readback shape in the design text —
   unpairable, undecodable, excess, empty expectation (I2, M3).
5. Route any new key-comparison through the single existing site (M1).
6. File follow-ups, with owners, for the false "key is reused" supply wording
   and the engrave-per-matched-slot product question (M2 / question 4) —
   `gui/multisig.go` is outside this plan's ownership.

With amendments 1–4 adopted, the redesign preserves every guarantee in the
current test suite verbatim, ends the false RED (traced: supply reused case
passes with one leg against its one plate), and keeps "a lost plate FAILS,
naming the slot". Re-review after the fold can be scoped to "does the amended
text match these amendments" — the traces above are settled and need not be
re-derived.
