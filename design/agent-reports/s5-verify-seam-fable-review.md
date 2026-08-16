# S5 seam review — verify slot set (070686a) x supply per-slot engrave (853534a)

Reviewer: fable (independent adversarial, highest-stakes tier). Date: 2026-08-16.
Worktree `/scratch/code/shibboleth/wt-s5`, branch `s5-multislot`, HEAD `853534a`,
clean at start and **clean at finish** (`git status --porcelain` empty; every
probe mutation reverted, probe test file deleted).

Question answered: do the two changes hold TOGETHER, and can their INTERACTION
produce a false GREEN. Method: full read of the seam
(`gui/multisig.go`, `gui/multisig_supply_tail.go`, `gui/multisig_verify.go`,
`gui/multisig_build_tail.go`, `gui/multisig_build.go:1-420,920-1215`,
`gui/multisig_match.go`, `gui/multisig_supply.go`, `gui/multisig_derive.go`,
`gui/multisig_engrave.go`, `gui/multisig_build_census.go`, `bundle/verify.go`,
`codex32/msencode.go`, `gui/bundle.go:150-230`), both implementation reports,
the fable design gate (`s5-verify-platecount-design-review.md`), the frozen plan
§S5 — plus **three executed probes** in the worktree (temporary test file +
one temporary mutation, both reverted). Targeted tests only; the full-suite
green at `853534a` was settled by the controller and was not re-run.

**Verdict up front: NOT safe to ship this seam as it stands — 1 Critical,
1 Important, 2 Minor.** The two changes agree with each other everywhere the
readback policy is the policy this run engraved: the honest supply seam
verifies its own output end to end, the bijection held under every in-policy
attack I ran, and the intersection guard is live and pinned. What fails is the
boundary of that condition: the slot set binds the verify to this run's slot
*indices* but not to this run's *policy*, and the policy is re-read from the
evidence being judged.

---

## Findings

### C1 — CRITICAL — the obligation channel carries slot indices but not the policy they index, so a different wallet's plates satisfy "Verify OK" while the just-cut plates are never read. **CONFIRMED (executed end to end).**

Sites: `gui/multisig_verify.go:327-339` (the policy is decoded from the
READBACK md1), `:388` (`allUserSlots` runs against the readback policy's keys),
`:445-446` (every leg is derived at the readback policy's origins, over the
readback md1 — which is why `bundle.Verify`'s md1 leg is, by the file's own
header admission at `:18-21`, "the supplied input compared to a clone of
itself"). Both callers hold the engraved md1 at the call site and do not pass
it: `gui/multisig.go:196,250` (`suppliedMd1`), `gui/multisig_build.go:281,325,364`
(`engraveMd1`).

Executed failure scenario (temporary probe test, deleted after the run):

- This run: wallet **P** = Trace B, 3-of-4, engraved with master B → the
  engraver's declaration is `expectedSlots = [2]`, one plate, bound to P.
- On the reader at verify: wallet **P′** = the SAME four cosigners at
  threshold **2-of-4** — a different wallet, different md1 bytes, different
  policy-id stub — as cut by P′'s own earlier run (deterministic encoders):
  md1(P′) + master B's @2 plate bound to P′.
- Operator types their real seed (master B), real (empty) passphrase.
- Precheck: 1 card == 1 expected. `allUserSlots(B, keys(P′)) = [2]`;
  `fresh = [2]`; the leg derives at P′'s @2 origin over md1(P′); it pairs with
  P′'s plate; stub binding, fingerprint, xpub, path, md1-exact all compare
  P′ against P′ and agree.
- **Final screen, captured from the driven flow:**
  `"Operator key and secret verified. Other cosigners' keys are taken as
  supplied. Verify OK"` — over a plate this run did not cut, for a wallet with
  a different spending threshold, while the plate and md1 this run DID cut were
  never read.

That is the brief's false-GREEN definition verbatim, and it is the same attack
the design gate's C1 killed the count design with — "a byte-valid plate from an
earlier run passes while the just-cut plate is never read" — moved one level
up, from slot identity to policy identity. The slot set closes it within one
policy (I confirmed the substituted same-wallet-other-slot readback and the
foreign-plate readback both go RED); it does not close it across policies,
because the integers in `expectedSlots` are re-based onto whatever policy the
evidence supplies. The realistic reach is exactly the operator the verify
exists for: a re-cut after changing k or rotating a cosigner, with the
superseded generation's steel on the same bench — the plates are visually
interchangeable, and no screen in the verify displays any identity of the
read-back policy.

Two aggravators make this Critical AT THIS GATE rather than a carried legacy
scoping (the pre-S5 verify passes the same substitution, so it is not a
regression introduced by either change):

1. The seam's new contract SAYS run-binding, on screen and in the record:
   "the slots THIS RUN ENGRAVED" (070686a's title), "Present exactly the
   plates this run cut" (`gui/multisig_verify.go:348-352`), "Checked %d of the
   %d key plates this run engraved" (`:486-490`). The mechanism delivers
   run-binding of cardinality and indices only; the words claim more than the
   check performs, in the direction that loses funds.
2. Both call sites already hold the one datum that closes the class (the
   engraved md1 strings). Passing it — either as an equality precheck
   (`readbackMd1` vs engraved md1; the encoder is deterministic, plan S5
   item 7) or by deriving legs against the SESSION's md1 so `bundle.Verify`'s
   md1-exact and stub-binding legs do real work — is the same "only the
   engraver knows" provenance the slot set itself rides on, and contradicts
   nothing in the frozen plan. §7.4 is not implicated: the evidence still
   comes from the plates; only the obligation gains the policy identity it
   already claims to have. (Direction noted for the owner; not a design I am
   imposing.)

### I1 — IMPORTANT — a supplied policy seating the same (xpub, origin) at two slots engraves two byte-identical plates that the gather can never read back as two, so the run is permanently unverifiable and the refusal instructs the impossible. **CONFIRMED (executed end to end).**

Sites: `gui/multisig_supply_tail.go:80-101` (one leg per matched slot, no
identical-card handling), `gui/multisig.go:162-177` (admission + the notice),
`gui/multisig_verify.go:347-353` (the length precheck), `gui/bundle.go:177-179`
(the gatherer keys mk1 cards on the payload-derived `chunk_set_id`, so a
byte-identical second card is `bundleDuplicate` — settled in the design gate's
C1). Supply-path admission of duplicate-key policies is deliberate scoping,
stated at `gui/multisig_build.go:1190-1197` ("repeated keys and all");
`duplicateSlotPair` guards only the build path.

Executed scenario: a supplied 2-of-2 md1 declaring master B's key at BOTH
slots, same origin (`md.EncodeMultisig` accepts it; `ExpandWalletPolicyChunks`
decodes it; `allSlotsHaveXpub` passes). `allUserSlots` → `[0 1]`;
`supplyEngraveTail` → `engravedSlots [0 1]`, and the two mk1 cards are
**byte-identical** (measured). The operator cuts both plates, presents both at
verify, and the driven flow ends at:

`"Read back 1 key plate, but this run engraved 2 key plates. Present exactly
the plates this run cut. Verify Bundle"`

— with the operator doing exactly that. The gather can structurally never
yield the second identical card, so the precheck is unsatisfiable forever: an
honest, announced, hours-long engrave that can never be verified, telling the
operator on every attempt that they are presenting the wrong steel. False RED,
permanent, introduced by the interaction — at `070686a` the supply path cut
one plate and passed `[]int{idx}`, and this shape verified fine. En route, the
multi-slot notice (`gui/multisig.go:173-176`) tells this operator "each of
those slots holds a DIFFERENT key. This run engraves 2 key plates, one per
slot" — false for this policy, the same claims-a-shape-the-code-does-not-have
defect class that F-188's commit message calls "the tell". Reach requires a
degenerate or hostile supplied md1 (coordinator bug, copy-paste duplicate),
which is also a wallet whose threshold one seed can satisfy alone — worth a
named refusal or a one-plate rule at the cross-match step rather than an
engrave the device then disowns. Not funds-losing; blocks as Important.

### M1 — MINOR — a passphrase divergence between engrave and verify is reported as "That seed is not a cosigner of the read-back policy", blaming the seed for a passphrase mismatch. **CONFIRMED (traced).**

`gui/multisig_verify.go:409-411`. The engrave accepts a payload-borne
passphrase (`syswPassphraseFlow`, `gui/multisig.go:147`); the verify requires
it re-typed (`passphraseFlow`, `:383`, deliberate per §7.4). Correct seed +
forgotten/mistyped passphrase → `allUserSlots` returns empty → the seed is
named, the passphrase never mentioned, on the flow's honest plates. An
operator taught "my seed isn't in my wallet" by a passphrase slip is the
false-RED trust erosion the brief names. Pre-existing in kind (the pre-S5
verify had the same shape through `findUserSlot`); unchanged by this seam;
recorded so it is owned rather than rediscovered.

### M2 — MINOR — the three mid-loop refusals exit a PARTIAL verify without the "Verify Incomplete" report, against the flow's own posture. **CONFIRMED (traced).**

`gui/multisig_verify.go:394-421`: all three `len(fresh) == 0` messages
`return` even when `legs` is non-empty. Scenario: 3-plate build across two
masters; seed A covers @0/@1; the operator mistypes seed B's passphrase →
"That seed is not a cosigner..." → the flow returns; the build path proceeds
to the restore doc. No screen ever says the three plates were NOT verified —
the exact outcome the ms1-entry Back was converted from `return` to `break` to
prevent (`:429-440`, same file, same reasoning). Not a pass, a screen was
shown, so Minor — but the inconsistency is one refactor away from being cited
as precedent.

---

## The attack list, answered

1. **The interaction.** The slot set (A) consumes and the slot set (B)
   engraves are the same integers with potentially different referents: the
   verify re-derives `filled` against the READBACK policy (C1 — executed false
   GREEN). Re-typed passphrase diverges → M1 (honest-direction refusal, wrong
   attribution). Re-typed different seed → honest refusals, all three arms
   traced (`:408-419`), including the new third message for a cosigner whose
   slots this run did not engrave. `both`-slot card: the built md1 records the
   card's own (xpub, origin) (card authoritative, `buildSelfKeys` skips it),
   the gate has proved the seed derives that key at that path, so
   `allUserSlots` at verify matches it and the leg derives where the plate was
   cut — no divergence found (traced; the engrave tail itself is `7910e00`,
   out of scope). Nothing else between engrave and verify recomputes or
   mutates the slot set: it is minted once in the tail loop that cuts the
   cards and travels by value.
2. **Is the intersection now a no-op?** On the supply path yes in honest runs
   (expected == filled by construction) — and that is why the guard's fixture
   was correctly re-pointed at the build shape. On the BUILD path it is live:
   an admitted cosigner card carrying a different key from the operator's seed
   at another origin (`duplicateSlotPair` refuses only identical keys) makes
   `filled ⊃ expected`, and without the intersection the extra leg finds no
   plate → false RED. **Executed:** deleting the `slices.Contains` clause at
   `853534a` turns `TestVerifyOneSlotRunChecksTheONEPlateItEngraved` and
   `TestVerifyFreshSlotsIsTheEngraversList` RED at flow level
   (`TestSupplyEngraveVerifiesItsOwnOutput` stays green under the mutation,
   confirming the supply path alone would not protect it). The guard is not
   one edit from silently gone. Mutation reverted, verified byte-clean.
3. **The re-typed seed.** See attack 1 and M1/M2. No dishonest GREEN found on
   any seed/passphrase divergence; the failures are refusal-shaped and at
   worst misattributed (M1) or under-reported (M2).
4. **One ms1 per distinct seed, supply path.** Holds. `codex32.EncodeMS1` is
   deterministic (fixed `"entr"` identifier, no randomness —
   `codex32/msencode.go:17-31`), the dedupe keys on the minted string
   (`gui/multisig_supply_tail.go:96-99`), one seed → one string → one plate;
   the driven flow's census showed exactly one `ms1 secret share` for a
   two-slot engrave, and watch-only emits none (`b.MS1 == ""` never enters the
   map). At verify each leg carries the ms1 typed for ITS seed
   (`verifyLeg.MS1Readback`), compared against entropy re-derived from that
   same re-typed seed — the two-master swap arm is pinned in the legs tests.
5. **The census.** Truthful for every shape I could construct: totals derive
   from `bundlePlatePlan`, the same function `bundleEngrave` loops
   (`gui/multisig_build_census.go:36-46`), so the count cannot drift from the
   cut; per-card lines count physical plates (`len(c.strings)`), and the
   14-plate Trace-B census sums exactly. It runs after mode selection, so
   watch-only counts are right. For the duplicate-slot policy the census is
   honest (2 identical plates announced as 2); it is the earlier NOTICE that
   lies (folded into I1).
6. **Tail drift.** None found. Same iteration order (ascending slots), same
   parallel slots/mk1s construction, same ms1 dedupe key (the engraved
   string), same emitter (`multisigEngraveCardsMulti`, one site), same
   zero-mk1 refusal posture, same per-slot-origin rule. The one asymmetry —
   build clears `b.MS1` on a dedup'd leg because it returns `legs`, supply
   returns no legs — has no consumer-visible effect. The two rules are
   genuinely one rule.

## What I verified green and did not re-derive

The controller's full-suite gate at `853534a` (51 ok / 0 FAIL, gofmt clean) —
settled, not re-run. `verifyMultisigLegs` byte-untouched across the range —
settled. The honest seam end to end: the supply tail's own plates + its own
slot list through the real `multisigVerifyFlow` → clean
(`TestSupplyEngraveVerifiesItsOwnOutput`, re-observed green during my mutation
run). In-policy substitutions (wrong plate, foreign plate, missing plate,
partial readback, master-A-alone on a three-plate build) all RED with named
messages — held by existing tests I read, two of which I executed.

## Verdict

**Together, within one policy, the two changes hold** — the engrave rule and
the check rule agree at the source, every in-policy false-GREEN path I attacked
is closed, and both directions of the bijection are pinned by tests that I
proved can fail. **Across policies they do not hold: C1 is an executed false
GREEN in the exact class the design gate was convened to kill, and the seam's
own screens now claim the run-binding it lacks.** I would not ship this seam
until the verify's obligation carries the policy identity the callers already
hold (C1) and the duplicate-slot supply shape has a ruled outcome (I1). M1/M2
are recorded for ownership and do not gate.

Probes: one temporary test file (two probe tests, both outputs quoted above)
and one temporary one-line mutation; both removed; `git status --porcelain`
empty at `853534a` at finish.
