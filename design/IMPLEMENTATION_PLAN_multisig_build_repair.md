# IMPLEMENTATION PLAN — on-device wallet-policy authoring, phase 1

Status: **DRAFT, pre-R0.** Descends from `SPEC_multisig_build_repair.md`, which
passed R0 at **GREEN 0C/0I** (rounds 0 and 1 persisted in
`design/agent-reports/multisig-build-repair-spec-R0-round{0,1}.md`). This plan
takes its own R0 pass before any code.

**Reference convention:** `§` is a section of the SPEC. `SYSW§` is
`SPEC_systemwide_payloads.md`. Source paths are in
`/scratch/code/shibboleth/seedhammer` unless stated.

## 0. What this delivers

An operator builds a k-of-n sorted-multisig **wallet policy descriptor on the
device**, holding one or more of its keys, with the remaining cosigner keys
arriving on a systemwide payload — and engraves it. Phase 2 (arbitrary wsh/tr
miniscript) is out of scope by §1.

## 1. Stage order, and the one constraint that forces it

| stage | delivers | spec |
| --- | --- | --- |
| **S1** | the payload supplies the whole cosigner set | P0 |
| **S2** | the dead end, the title, the interim origin refusal | P1 |
| **S3** | nested segwit is nameable; the four stale comments die | P2 |
| **S4** | the slot-assignment model + the seed↔key gate | P4 (moved) |
| **S5** | multi-slot self, divergent origins, **and the engrave tail** | P3 + §4.1a |
| **S6** | hardware validation | P5 |

**The constraint: S5's assembly and tail are ONE stage and must not be split.**
That is C2 restated as scheduling. Assembly alone produces a policy whose legs
are still derived at the locked shared origin — a key card asserting membership
in a wallet that does not contain its key, on steel. A stage that could close
green in that state is a stage that ships C2.

**S4 before S5 — the deviation from the spec's numbering, and why.** §10 Q4 asks
whether §4.3's gate moves ahead of multi-slot work. It is planned here as
**yes**, on two grounds: the exposure is live today (§2.2 D-5 — payload seeds
already reach the constructor with no cross-check), and the gate depends only on
the assignment model, not on multi-slot support. **If the operator rules
otherwise, swap S4 and S5; nothing else changes.**

## 2. The journeys — the map that keeps a missing stage visible

Both traces are the R0 round-1 reviewer's, and they are this plan's acceptance
criteria rather than illustrations. A stage that closes green while its trace
still breaks has not closed.

**Trace A — ordinary.** 2-of-3. The operator holds one key. Two cosigner mk1
cards are on a payload. Expected: correct descriptor engraved, **from the end of
S2**.

```
boot → SKIP/LOAD payload → digest compare → Engrave Multisig → Build policy
  → template(wsh) → n=2..5 → k → self-slot → fp
  → cosigner review (S1: from the payload, not "Scan a card")
  → seed entry (typed OR payload)
  → policy review → form → EXPERIMENTAL → mode → engrave → restore doc
```

**Trace B — flagship.** n=4, k=3. The operator holds `@0 = A·acct0`,
`@1 = A·acct1`, `@2 = B·acct0`; `@3` is cosigner D's card on the payload.
Expected: correct descriptor engraved, **from the end of S5**, with divergent
origins, one mk1 per held slot, and ms1 for **both** masters in full mode.

Trace B is the wallet round 0's C1 would have refused. It is the reason S5
exists and the reason S6 may not close without rehearsing it on hardware.

## 3. Per-stage detail

Every stage: **tests first**, then implementation, then the gate. Every stage's
gate includes the §4.5 emulator walk. Tests are tier 1 (§4.6) unless named
otherwise — synthetic time via `testing/synctest`, no real sleeps.

---

### S1 — the payload supplies the whole cosigner set

**Tests first**

1. `TestSyswTakeAllYieldsEveryMDMKRecord` — a session holding three `ClassMDMK`
   records yields all three; `take` still yields the first only.
2. `TestSyswTakeAllRefusesBeforeCompared` — `!loaded || !compared` refuses.
   **Mutation-checked** (§4.6, spec M-D): delete the guard and this must fail.
   Without it an unauthenticated payload's cards reach the constructor, and with
   fingerprints omitted by default the review screen cannot surface a swap.
3. `TestBuildGathersEveryCosignerFromPayload` — n=3, two mk1 cards (each 2
   chunks) on the payload, zero scans; the gather yields two complete cards.
4. `TestBuildIgnoresMd1RecordsInThePayload` — an md1 alongside the cards does
   **not** fail the build (spec P0 item 3).
5. `TestBuildSlotOrderIsPayloadRecordOrder` — asserts `@N` assignment follows
   payload record order, and that the review screen shows it. Order is
   identity-bearing (`md/encode_multisig.go:13-21`).
6. `TestBuildRefusesMoreCardsThanOpenSlots` — named refusal, not a fall-through.

**Implementation**

- `gui/sysw_session.go` — add the `takeAll`-style accessor, inheriting `take`'s
  loaded/compared refusal.
- `gui/multisig_build.go:54` — replace the single `syswOffer` seeding of
  `ctx.syswBundleSeed` with every `ClassMDMK` record fed through
  `bundleGatherFlow`'s `offer()`. Do not add a second insertion path:
  `gui/bundle_flow.go:100-103` states why.
- Filter md1 records out before `buildCosignerCards`, which refuses on them.
- The gather screen becomes a **review of what the payload supplied** (spec P0
  item 6). Title fixed in S2 with the rest of D-4.

**Gate.** Trace A reaches the gather with both cards, by test and by emulator
walk. Then: **either the flow completes an engrave, or D-1 reproduces and is
captured as a failing test** (spec P0 gate — round 0's I2).

---

### S2 — the dead end, the title, the interim origin refusal

**Tests first**

1. The D-1 reproduction from S1, promoted to a regression test. It **MUST fail
   on the unfixed code** — demonstrated, not assumed. If S1 found no D-1 on the
   payload path, this stage records that as its result and names the source or
   shape that was not exercised, rather than closing silently.
2. `TestBuildGatherIsNotTitledEngraveBundle` — D-4.
3. `TestBuildRefusesForeignOriginCardBeforeS5` — spec M-E: until S5,
   `cosignerFromCard` still discards origins, so a card whose declared origin
   differs from the shared origin must be refused or warned, not silently
   stamped `m/48'/0'/0'/2'`.
4. **A raster assertion on whatever D-1 turns out to be.** If the defect is a
   screen whose body does not draw, a text assertion cannot see it — F-151.
   Calibrate the floor against the real defect, measured both ways; F-151's
   first guess of 2000 px passed the defect it was written for.

**Gate.** Trace A completes end to end: engrave, by test and by emulator walk,
producing an md1 the host accepts byte for byte.

---

### S3 — nested segwit is nameable; the stale comments die

**Tests first**

1. `TestScriptNameDistinguishesNestedFromLegacy` — the three names are
   **pairwise distinct**. The defect is that two are equal, so a test that only
   checks P2SH-P2WSH would pass today.
2. `TestRestoreDocNamesNestedSegwit` — at `gui/multisig_restore.go:51`
   specifically. It is the surface that matters: the operator reads it years
   later, alone.

**Implementation**

- `scriptName(tpl md.Template)` instead of `scriptName(k md.ScriptKind)`;
  `ScriptSh + InnerWsh → P2SH-P2WSH`, `ScriptSh + InnerWpkh → P2SH-P2WPKH`,
  bare `ScriptSh → P2SH` (§4.4).
- All three callers together: `gui/md1_inspect.go:58`,
  `gui/multisig_restore.go:51`, `gui/bundle.go:315`. Round 1 confirmed
  `scriptName` has no consumers outside `gui`, so that is the complete set.
- Delete or correct the four `TYPED-ONLY` comments (§2.2 D-5) at
  `gui/bip85.go:264`, `gui/singlesig.go:18`, `gui/multisig.go:24`,
  `gui/multisig_build.go:67`. They describe a retired mechanism, and a future
  reader greps `TYPED-ONLY`, finds four hits, and concludes the payload cannot
  reach a seed entry.

**Gate.** Emulator walk shows `P2SH-P2WSH` on the restore doc for an `sh(wsh)`
build. `grep -rn TYPED-ONLY` returns only the two verify sites, which are true.

---

### S4 — the slot-assignment model and the seed↔key gate

Closes the live exposure of §2.2 D-5.

**Tests first** — the gate's failing rows must be proven to fail, each
mutation-checked, or this stage ships a check that cannot fire:

1. `TestGateFiresOnBothSlotMismatch` — a `both` slot whose payload key does not
   derive from the payload seed → **FAIL LOUDLY**, naming the slot. Nothing
   engraved.
2. `TestGateAcceptsBothSlotMatch` — the honest case proceeds.
3. `TestGateIgnoresUnassignedCosigners` — a payload with the operator's seed and
   two unrelated cosigner cards is **normal** and must not fail. This is the
   false-positive that would make the feature unusable.
4. `TestGateRefusesDuplicateKeyAcrossFinalSlots` — §4.1's discriminator: two
   final slots with identical 65-byte chaincode‖pubkey → refuse.
   `sortedmulti(2,K,K,X)` is spendable by K alone.
5. `TestGateAcceptsSameSeedAtDistinctOrigins` — the legitimate multi-account
   shape proceeds with a notice. **This is the test round 0's C1 would have
   failed**, and it is why it exists.
6. `TestGateRefusesContradictingFingerprint` — present-and-wrong fp → refuse.
7. `TestGateNeverPrintsSeedOrPassphrase` — no failure message contains seed
   words or passphrase text. Mutation-checked by splicing them in; stderr and
   screen text both.

**Implementation**

- The slot-source model of §4.3: `payloadKey(record)`,
  `derived(seedID, account)`, `both(seedID, account, record)`, with a review
  screen the operator confirms before assembly.
- The gate at construction time, reusing `findUserSlot`'s derive-and-compare
  (`gui/multisig_match.go:34`) rather than a second implementation.
- Per-seed passphrase (§4.1), asked at that seed's entry.
- Bindings per spec M-B: in a `both` slot the card's origin and key are
  authoritative; `account` is bookkeeping; `derived`'s `account` is the BIP-48
  account component.

**Gate.** Every failing row demonstrated failing. Emulator walk of the `both`
happy path and of one loud failure.

---

### S5 — multi-slot self, divergent origins, and the engrave tail

**One stage. Do not split** (§1).

**Tests first**

1. `TestMultiSlotSelfAssembles` — Trace B's shape assembles with
   `OriginDivergent` and the correct per-slot origins.
2. `TestCosignerCardOriginIsHonoured` — R-3: the card's declared origin reaches
   the descriptor, not the flow's shared origin.
3. `TestLegDerivedAtHeldSlotOrigin` — **C2's first scenario.** A slot held at
   `m/48'/0'/1'/2'` produces an mk1 derived there, not at
   `multisigSharedOrigin()`. Assert the mk1's key is one the descriptor
   contains.
4. `TestOneMk1PerHeldSlot` — cardinality, ruled in §4.1a item 2.
5. `TestFullModeEngravesMs1ForEveryMaster` — **C2's second scenario.** A 3-of-4
   across masters A and B in full mode engraves both ms1s, or refuses with a
   named reason. Losing B otherwise leaves two legs against k=3: unspendable,
   from a backup labelled "Full (seed + keys)".
6. `TestDepthZeroCosignerCardIsNamedRefusal` — spec M-1: `Path == "m"` trips
   `errMultisigEmptyDivergent` (`md/encode_multisig.go:104-106`); refuse by a
   named screen, not a fall-through "Couldn't assemble".

**Implementation**

- `buildPolicyParams.SelfSlot int` → a set of held slots.
- `cosignerFromCard` stops discarding `card.Origin`; `OriginDivergent` when
  origins differ, `OriginShared` when they do not.
- The tail: `deriveMultisigLeg` per held slot at that slot's origin; ms1 per
  distinct master in full mode.
- Remove S2's interim foreign-origin refusal, which this stage supersedes.

**Gate.** Trace B completes: correct descriptor, by test and by emulator walk.
**The §4.5 byte comparison extends to every mk1 and to ms1 presence** — the md1
alone cannot see either C2 scenario.

---

### S6 — hardware validation

**Not tier 1.** One flash cycle, via `~/bin/sh/sh2-flash` (never `picotool` by
hand — the build output is unsigned).

1. Engrave and restore a `wsh` multisig; verify against an external coordinator.
2. Engrave and restore an `sh(wsh)` multisig; same. Confirms S3 on the plate,
   not just the screen.
3. **At least one build MUST be divergent-origin, multi-slot and multi-master**
   (§6 P5). A shared-origin single-seed run would pass green around every
   §4.1a failure.

**Gate.** All three restore correctly at an external coordinator. This confirms
software already proven; it is not the first place the flow is executed.

## 4. What is NOT in this plan, deliberately

- **NFC.** Its own later plan, with F-158. This plan neither adds nor removes
  the existing SCAN row (§5.1) — it builds and tests against payload and typed.
- **Taproot, miniscript operators, the recipe seam, the type-checker port** —
  phase 2 (§9).
- **`ClassCodex32Secret` as a payload seed** — needs SYSW§3.1's carrier-type
  change (§5.4).
- **The 54.3 s → 10 s test sweep** (§4.6). New tests here are tier 1; converting
  the existing slow ones is opportunistic and filed, not owned. Letting it into
  S1 is how S1 becomes a test-infrastructure project.
- **The on-device verify readback**, which is NFC-only and therefore exercisable
  only at S6 (§4.5's named blind spot).

## 5. Known blind spots of this plan's own gates

Stated because a gate that hides its blind spot is worse than none:

- The emulator walk drives the real `gui` package but not real hardware: no
  stepper motion, no plate, no NFC. S6 is the only stage that touches those.
- `plan-cite-gate.sh` proves a cited line exists, not that it says what this
  plan claims. That gap is exactly where R0 round 0 found I4.
- Trace A and Trace B are two shapes. They are not proof that every k-of-n and
  wrapper combination works; the per-stage tests carry that.
