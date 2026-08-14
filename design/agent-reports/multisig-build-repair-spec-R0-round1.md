# R0 round 1 — SPEC_multisig_build_repair.md (fold verification + the headline traces)

Reviewer: independent architect, round 1, 2026-08-13. Scope per brief: (1) the
headline question — will this plan actually let a user create wallet policy
descriptors on the device, traced concretely twice; (2) did the fold
(`82bae73..581d02a`) fix round 0's 2C/7I, and did it introduce new defects.
Source examined at `/scratch/code/shibboleth/seedhammer` @ `a10d007`
(confirmed by `git log`). Settled facts from the brief (R-3 soundness, the
TYPED-ONLY site re-verification, `scriptName` consumer scope, SYSW§3.3.2's
admission row, cite-gate pass) were treated as inputs and not re-derived.

## Verdict

**GREEN 0C/0I** — the gate closes. Both traces reach a correct descriptor.
Five Minors/Nits recorded below; none gates.

---

## THE HEADLINE QUESTION

**Yes — this plan lets the operator create wallet policy descriptors on the
device.** Trace A completes at P1 close; Trace B completes at P3 close and is
then *mandatorily rehearsed* by P5 ("at least one build MUST be
divergent-origin, multi-slot and multi-master"). Neither trace hits a step
that exists in no section. Three early steps are assumed-as-shipped rather
than specified, each with journey or measured evidence — flagged inline. The
structural defense against the `can-a-user-do-the-thing` failure is that §4.5
makes the walk itself a per-stage closing gate with a byte comparison against
host output, so a stage cannot close green around an inert seam.

### Trace A — the ordinary case (2-of-3, operator holds one key, two cosigner mk1 cards on a payload)

| # | screen | section | specified or assumed? |
|---|--------|---------|----------------------|
| 0 | (host, no screen) `me sysw pack` writes the two mk1 card chunk-sets; device in BOOTSEL | §3.1 | **Specified** — pack/show verified as run output (§8); the host-step consequence is stated, and §10 Q3 asks its acceptability rather than hiding it |
| 1 | Power on → boot payload offer (`syswLoadFlow` at boot, `gui/gui.go:1761-1765`) | §3.1 bullet 3 | **Assumed-as-shipped**, with evidence: the Load Payload journey was walked end-to-end 2026-08-12 (`design/journeys/SeedHammer-II-load-payload-journey.pdf`). Unchanged surface; acceptable |
| 2 | Payload digest confirm (`[compared]`, SYSW§12.2) | none in THIS spec | **Assumed via SYSW§** — normative there ("a record is admitted for consumption only when `[compared]` is true", SYSW§5.4.1). See new finding M-D: P0's `takeAll` should restate that it inherits `take`'s refusal |
| 3 | Main menu → Engrave Multisig | §0 | Assumed-as-shipped (program exists; §0 names the front door) |
| 4 | "Supply or build a policy?" → Build policy (`engraveMultisigFlow`, `gui/multisig.go:41-56`) | §0 | **Specified** as measured current state |
| 5 | Bounded pickers: Template / n=3 / k=2 / your slot / fingerprints (`buildParamPickFlow`) | §2.2 D-2 (current), §4.3 model (end state, P3/P4) | **Specified**. At P0/P1 the existing single-`@S` picker suffices for this trace; post-P3 the slot-source assignment model governs and a one-held-slot set is its degenerate case (§4.1 "one or more") |
| 6 | Cosigner supply from the payload: `takeAll` → `bundleGatherFlow.offer()` → filter to mk1 (item 3) → over-supply rule (item 4) → order = payload record order shown as `@N` (item 5) → the gather screen is a **review of what the payload supplied** (item 6); title fixed at P1 (D-4) | §6 P0 items 1–6 | **Specified** — the fold closed the round-0 holes (I5, old Q5): md1 records ignored not fatal, over-supply ruled, order ruled and displayed |
| 7 | "Input Seed / Where from?" → TYPE IT → word entry (payload holds no `ClassMnemonic`, so no FROM PAYLOAD row; SCAN row on hardware ruled left alone) | §5.1, §5.2, §2.2 D-5 | **Specified** — the source seam is measured behavior, not aspiration |
| 8 | Passphrase prompt (per seed; one seed here) | §4.1 final bullet | **Specified** |
| 9 | Derivation + assembly: self key at the declared origin (P0/P1: locked shared origin; post-P3: `derived(seed, account)`, `OriginShared` since all declared origins equal), cosigners from cards, `md.EncodeMultisig` | §4.1, §2.1 | **Specified** |
| 10 | Duplicate-key check over the final slot set (no dupes here); consistency gate does not fire (no `both` slot — §4.3 row 6) | §4.1, §4.3 (P4) | **Specified** |
| 11 | Policy Review: stub + `@N` slots + fp note | existing screen + P0 item 5 | **Specified** |
| 12 | Full-vs-template md1; unskippable EXPERIMENTAL warning; Full/Watch-only mode | §0/§2 (measured, unchanged) | Assumed-as-measured; acceptable — no stage touches these screens |
| 13 | Engrave: plates for [ms1?, mk1, md1]. D-1 (the blank-screen dead end) lives somewhere on this path today | §6 P1 gate + §4.5 | **Specified** — P1 closes only on a completed engrave by test AND emulator walk, with the byte comparison vs host output. The fold's P0/P1 gate split (I2 fix) makes this reachable in either D-1 branch |
| 14 | Verify offer — NFC-readback only; operator skips in phase 1 | §4.5 named blind spot | **Specified** (recorded, owned by F-158) |
| 15 | Restore doc — correct naming after P2 (§4.4), including the P2SH-P2WSH distinction | §4.4, §6 P2 | **Specified** |

**Verdict A: the operator obtains a correct descriptor** (deliverable at P1
close). One interim caveat recorded as new finding M-E: between P1 and P3, a
cosigner card minted at a *non*-shared origin is still silently stamped with
the shared origin (today's D-2, owned by P3) — this does not affect the
ordinary case traced here, and the EXPERIMENTAL warning mandates coordinator
verification before funding.

### Trace B — the flagship case (n=4, k=3; operator holds @0=A·acct0, @1=A·acct1, @2=B·acct0; @3 is cosigner D's card on the payload)

| # | screen | section | specified or assumed? |
|---|--------|---------|----------------------|
| 0–4 | as Trace A (payload carries D's mk1 card only) | as above | as above |
| 5 | Pickers n=4, k=3; **held slots declared as a set with per-slot sources**: @0=`derived(A,0)`, @1=`derived(A,1)`, @2=`derived(B,0)`, @3=`payloadKey(D)`; shown on a review screen before assembly | §4.3 assignment model (NORMATIVE), §4.1, §6 P3/P4 | **Specified at the model level** — the fold's I1 fix. The concrete picker layout is implementation, but the model rules every datum the screen must capture, and §4.5's walk is the gate that the screens actually exist and compose. This is the correct altitude for a spec |
| 6 | Cosigner supply: one card for one open slot. The pre-P3 "exactly n−1" count is superseded by the model (every slot carries exactly one source, so open slots = n − held) | §4.3 model; P0 item 4's "open slots" wording | **Specified by entailment** — no fixed n−1 survives in the end-state model |
| 7 | Seed entry ×2: seed A typed + its passphrase (per-seed prompt), retained for the constructor to derive account 1 (§4.2 working-copy rules); seed B typed + its passphrase | §4.1, §4.2 | **Specified** — I6's and I7's fixes are exactly what this step needs |
| 8 | Origins: @0 `m/48'/0'/0'/2'`, @1 `m/48'/0'/1'/2'`, @2 `m/48'/0'/0'/2'`, @3 = card D's declared origin (R-3). Not all equal → `OriginDivergent` MUST be used; a depth-0 card is refused by a named screen | §4.1, §7 R-3 | **Specified** (account→BIP-48-component mapping is implied by §4.1a's example rather than stated — Nit, folded into M-B) |
| 9 | Checks: duplicate-key over the final set — A·acct0, A·acct1, B·acct0, D are four distinct keys (hardened account derivation) → proceed. Same seed at ≥2 slots under distinct origins → proceed **with the notice**. This is the exact input round 0's C1 rule refused; the folded rule admits it | §4.1, §4.3 rows 4–5 | **Specified** — C1's fix, verified adversarially below |
| 10 | Consistency gate: no `both` slot → no check (row 6) | §4.3 (P4) | **Specified** |
| 11 | Review: stub + `@N` slot sources | §4.3 + P0 item 5 | **Specified** |
| 12 | EXPERIMENTAL warning; mode choice. **Full mode → §4.1a item 3: an ms1 for BOTH masters A and B, or a named refusal of multi-master full.** Either arm closes C2's unspendable-backup scenario; neither is silent | §4.1a item 3 | **Specified** |
| 13 | Engrave tail: leg derivation per held slot at ITS declared origin (item 1) — never `multisigSharedOrigin()`; one mk1 per held slot (item 2 default arm; see M-C); §4.5's comparison covers the md1, **every mk1**, and ms1 presence from P3 | §4.1a, §4.5, §6 P3 gate | **Specified** — C2's fix. The wrong-origin-mk1 and missing-ms1 failures are now both visible to a gate |
| 14 | Verify offer — blind spot, skip; P5 exercises it on hardware | §4.5 | **Specified** |
| 15 | Restore doc: per-slot origins shown; then **P5 REQUIRES at least one divergent-origin, multi-slot, multi-master hardware build** — i.e., this exact trace is rehearsed on the machine before ship | §6 P5 | **Specified** |

**Verdict B: the operator obtains a correct descriptor** (at P3 close; P4 adds
the both-slot gate, which this trace does not need; P5 proves this shape on
steel). Round 0's C1 would have refused this wallet at step 9; the folded
discriminator admits it and refuses only genuine key duplication.

---

## Fold verification

| finding | fixed? | new defect from the fix? |
|---|---|---|
| **C1** (wrong reuse discriminator) | **YES.** Refusal now keys on *identical 65-byte chain code ‖ pubkey at any two final slots* (§4.1), source-independent; the multi-account shape proceeds with the notice (§4.1, §4.3 row 5); §2.1's `reused` description corrected with the verbatim doc comment (checked against `gui/multisig_match.go:24-29` — accurate); round 0's row 3 deleted, restoring R-2 consistency. Adversarial walk: the false-negative (`sortedmulti(2,K,K,X)` via the operator's own old card) is now refused by the final-set check regardless of arrival shape; the false-positive (Trace B step 9) now proceeds. | None found. The wording tension between "the gate fires on a `both` slot" and outcomes row 4 (the duplicate check is unconditional) is resolved by §4.1 stating the check as its own always-on rule; row 4 merely cross-references it. |
| **C2** (engrave tail unrespecified) | **YES.** §4.1a items 1–4 are round 0's four fix items verbatim-in-substance; P3 owns the tail explicitly; §4.5 extended to every mk1 + ms1 presence from P3; P5 mandates the divergent/multi-slot/multi-master build. Verified against `gui/multisig_derive.go:32-74`: `deriveMultisigLeg` does emit one mk1 at the passed origin and one ms1 from the single mnemonic — §4.1a's premise is measured, not described. | **M-C** (Minor): item 2 keeps the cardinality as a disjunction and P3's stage text schedules "the mk1 cardinality ruling" as work — the default arm should be named now (below). |
| **I1** (no assignment mechanism) | **YES.** The three-source slot model is normative in §4.3, the `both` trigger is defined against it, old Q3/Q5 are ruled not asked. | **M-B** (Minor): the `account` element of `both(seedID, account, record)` is consumed by neither the gate (which derives at the *key's declared origin*) nor the encoding (card origin wins, R-3) — pin its role (below). |
| **I2** (P0/P1 gates mutually exclusive) | **YES.** P0 = drivable (completed engrave OR captured D-1 repro); P1 = repro-fails-on-unfixed AND completed engrave, with an explicit named-not-closed branch if D-1 never manifests on the payload path. Both branches implementable. | None. |
| **I3** (NFC contradictions) | **YES.** NFC struck from §1, §5.4, R-1; the SCAN row ruled exactly as round 0 recommended (§5.1). Grep of the folded spec: every residual "NFC" mention is diagnostic, scoping, or the named blind spot — no normative sentence licenses NFC in phase 1. | None. |
| **I4** (TYPED-ONLY mischaracterization) | **YES.** §2.2 D-5 added (verified independently by the author pre-fold, per brief); §5.2 rewritten as text-not-mechanism work; R-1 retitled as ratifying+scoping; the live exposure stated plainly and surfaced as §10 Q4 with the P4-ordering option. | **M-A** (Minor): P4's note cites "§10 Q7" — a dangling pointer; the question is Q4 (below). |
| **I5** (ingest-everything dead-ends payloads) | **YES.** P0 items 3 (filter to mk1, md1 ignored not fatal — `buildCosignerCards` citation checked against `gui/multisig_build.go:254-272`, accurate), 4 (over-supply ruled), 5 (slot order = payload record order, shown `@N`; the `md/encode_multisig.go:13-21` ordering-contract quote is verbatim), 6 (gather screen ruled). | None. |
| **I6** (unsatisfiable lifetime MUSTs) | **YES.** §4.2 scoped to the flow's working copies; session-record lifetime ceded to SYSW§3.2.1 with the `gui/sysw_session.go:12-18` ruling quoted (quote settled per brief). | None. |
| **I7** (passphrase binding) | **YES.** Per-seed prompt at entry; `(seed, passphrase)` is the derivation unit everywhere; rationale includes the no-cross-check-for-new-seeds point. | None. |
| **M1** (depth-0 card) | **YES** — §4.1 requires refusal by a named screen; `errMultisigEmptyDivergent` citation accurate (`md/encode_multisig.go:104-106`). The screen itself is named at implementation, which meets the "designed refusal, not fall-through" intent. | None. |
| **M2** (verify blind spot) | **YES** — §4.5 records it, owned by F-158. Verified: `multisigVerifyFlow` uses `seedEntryFlowTypedOnly` + `bundleGatherFlow` NFC readback with an explicit no-payload-offer comment (`gui/multisig_verify.go:50,68-76`) — the spec's claim is exact. | None. |
| **M3** (`reused` misstated) | **YES** — §2.1 rewritten with the verbatim quote. | None. |

Claim-vs-line pass over fold-ADDED citations (my residual duty per brief): all
checked true — §4.1a's `gui/multisig_build.go:95-168` step range; §4.3's
`multisigFpChoices` "No (omit)" at index 0 (`gui/multisig_build.go:334`);
§5.1's `syswSeedPicker` three-row description (`gui/derive_xpub.go:140-161`);
§4.5's `gui/derive_xpub.go:112-123` self-comparison rationale; P0 item 1's
`take` first-match/non-consuming (`gui/sysw_session.go:114-124`); D-5's
consequence claim (nothing in the build path cross-checks a payload seed
against payload cards — confirmed by reading the full flow: `findUserSlot` is
supply/verify-path only).

---

## New findings (all Minor/Nit — recorded, none gates)

### M-A (Minor) — dangling cross-reference: P4 cites "§10 Q7"

**Where.** §6 P4: "Flagged for the operator (§10 Q7)." §10 contains questions
1–4; the exposure question is **Q4**. No numbering scheme, old or new, contains
a Q7. **Failure scenario:** the operator, asked to rule on the P4-ordering
question, greps for Q7, finds nothing, and the live-exposure decision the fold
itself calls "the sharpest one" goes unanswered into implementation.
**Fix:** one character — "§10 Q4".

### M-B (Minor) — the `account` element of `both(seedID, account, record)` has no consumer

**Where.** §4.3's source table vs. its mechanism paragraph. The gate derives
"at the key's declared origin path" (the card's origin, per R-3), and the
encoded slot takes the card's key and origin — so the tuple's `account` is an
input to nothing. **Failure scenario:** an implementer, needing `account` to
matter, derives the check at `m/48'/0'/account'/2'` instead of the card's
declared origin; an operator who mis-remembers the account then gets FAIL
LOUDLY on a genuinely-theirs card (annoying, fail-closed) — or the reviewer of
P4 has to re-derive which origin was meant. **Fix:** one sentence in §4.3: in
a `both` slot the card's declared origin and key are authoritative (R-3); the
`account` field is display/bookkeeping only (or drop it from the tuple). While
there: state that the account index of `derived(seedID, account)` occupies the
BIP-48 account component (`m/48'/0'/account'/2'`), which §4.1a item 1's
example implies but nothing rules.

### M-C (Minor) — §4.1a item 2 defers a normative ruling into P3

**Where.** "One mk1 per held slot … — or an explicit ruling that one leg
suffices, with its reason"; P3's stage text then lists "the mk1 cardinality
ruling" as stage work. The spec is the document that rules; a stage is where
rulings are executed. Both arms are safe (extra legs, or fewer legs with the
md1 still carrying every xpub), so this cannot mint wrong steel — but the §4.5
byte comparison from P3 covers "every mk1", which presumes the one-per-slot
arm. **Fix:** one sentence: one mk1 per held slot IS the phase-1 rule; a
contrary ruling requires its own recorded reason. (This also makes §4.5's
"every mk1" well-defined.)

### M-D (Minor) — P0's `takeAll` accessor is specified without its `[compared]` precondition

**Where.** P0 item 1 (pre-fold text; surfaced by the trace pass, so recorded
here). `take` refuses while `!loaded || !compared` — the consumption-side
enforcement of SYSW§5.4.1's normative rule ("a record is admitted for
consumption only when `[compared]` is true for the identity it came from").
The new accessor is a second consumption path, and P0 item 2's own quoted
rationale names the hazard: "only one of them would have the checks."
**Failure scenario:** `takeAll` written without the guard hands an
*unauthenticated* payload's cosigner cards to the constructor; with
fingerprints omitted by default the review screen cannot surface a swapped
key. Severity is Minor, not Important, because SYSW§5.4.1 already binds the
implementer normatively and the guard sits two lines above the loop any
implementer will copy — the corpus is not silent, this spec merely is.
**Fix:** one sentence in P0 item 1: the accessor inherits `take`'s
loaded/compared refusal, and its test proves the refusal (mutation-checked,
per §4.6).

### M-E (Minor) — the P1→P3 window keeps D-2's silent origin-stamping for foreign-origin cards

**Where.** Staging. P1 makes engraves complete while `cosignerFromCard` still
discards card origins until P3 — so in that window a cosigner card minted at a
non-shared origin is stamped `m/48'/0'/0'/2'` in the descriptor, silently.
Funds-correct (addresses derive from the keys), restore fail-closed, and it is
exactly today's shipped D-2 behind the mandatory EXPERIMENTAL
verify-against-coordinator warning — so Minor, and the ordinary Trace A is
unaffected. **Fix (optional, one sentence in P0 or P1):** until P3, refuse or
warn when a gathered card's declared origin differs from the shared origin —
the comparison is one line and turns the interim silence into a designed
refusal. Alternatively, record the window's acceptability alongside §10 Q4,
which flags the analogous §4.3 exposure.

---

## Gate disposition

**GREEN 0C/0I.** Round 0's two Criticals and seven Importants are each fixed,
and no fix introduced a defect above Minor. The headline question is answered
affirmatively by both traces: the ordinary operator gets their descriptor at
P1; the flagship multi-account operator — the wallet round 0's C1 would have
refused — gets theirs at P3, and P5 refuses to close without rehearsing that
exact shape on hardware. The five Minors above are one-sentence edits; per
the proportional re-review rule, folding them does not re-trigger a round.
