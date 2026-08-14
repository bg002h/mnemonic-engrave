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
| **S0** | the oracles: pinned primary toolchain + published-BIP address vectors | §1a |
| **S1** | the payload supplies the whole cosigner set | P0 |
| **S2** | the dead end, the title, the interim origin refusal | P1 |
| **S3** | nested segwit is nameable; all 9 stale `TYPED-ONLY` comments die | P2 |
| **S4** | the slot-assignment model + the seed↔key gate | P4 (moved) |
| **S5** | multi-slot self, divergent origins, **and the engrave tail** | P3 + §4.1a |
| **S6** | hardware validation | P5 |

**The constraint: S5's assembly and tail are ONE stage and must not be split.**
That is C2 restated as scheduling. Assembly alone produces a policy whose legs
are still derived at the locked shared origin — a key card asserting membership
in a wallet that does not contain its key, on steel. A stage that could close
green in that state is a stage that ships C2.

**S4 before S5 — RULED by the operator 2026-08-13** ("Agreed. Safety first."),
answering §10 Q4. The exposure is live today (§2.2 D-5 — payload seeds already
reach the constructor with no cross-check) and the gate depends only on the
assignment model, not on multi-slot support. Recorded as settled so a later
reader does not reopen it.

## 1a. The oracles — what "correct" is measured against

Operator criterion, 2026-08-13: *"Assess safety by comparing with mnemonic
constellation output and measure byte identical"*, and *"we can also test
address derivation from published bip test cases."*

**Three oracles, and they are not interchangeable.**

**Oracle 1 — the CURRENT PRIMARY toolchain, byte for byte.** Not the fork's
vendored testdata, and not whatever binary is on `PATH`. The pins today are
`md-codec 0.42.x` (or `me`, which pins it), `mk-codec 0.4.2`, `ms-codec 0.7.0`.
**The walk script MUST print the oracle versions into every gate record**, so a
stale oracle is visible rather than silent, and MUST record the full input tuple
(template, n, k, slot order, fp choice, per-slot origins, seeds) so "same
inputs" is reproducible rather than remembered.

Comparing against vendored fork testdata **satisfies no gate** — but the reason
is coverage, not corruption, and that distinction was itself an inherited claim
until it was run. The fork's md parity vectors are pinned to **v0.36.0** against
a primary at **0.42.0**, and `mk/mk.go:5` pins "mk-codec 0.2" against 0.4.2.
**Measured 2026-08-13: 0.36 → 0.42 shows ZERO byte drift across all 30 vectors.**
So "the drift is measured" — which this plan asserted for two rounds — was
false. The vendored vectors are not wrong; they are simply an old and smaller
sample, and a gate that accepted them would prove agreement with a subset of
ourselves. S0 deliverable 4 is a **coverage catch-up, not a correctness
repair**. F-127 remains the record of what a genuinely divergent pin cost.

**The comparison plane, per artifact — ruled here, before any code:**

| artifact | relation | why not plain string equality |
| --- | --- | --- |
| md1 | **full string equality** against the primary's output for the same inputs | deterministic on both sides |
| mk1 | **(a)** the current primary accepts the chunks, **AND (b)** field equality via `mk verify --xpub --origin-fingerprint --origin-path --policy-id-stub`, which RUNS on fork-encoded chunks (checked). `canonical_payload_bytes` is a Rust **library** API with no `mk` CLI surface — there is no `mk bytecode` — so the original wording named a relation nothing could execute | the primary mints a fresh CSPRNG 20-bit `chunk_set_id` per encode with no CLI override, while the fork derives it from the bytecode — so literal equality fails on every honest run. The id is excluded because **the primary randomizes it by design**; this is a ruled property of the format, not a test-time convenience |
| ms1 | **full string equality** against `ms encode --hex <that master's entropy>` | deterministic; this is C1 |

A `--chunk-set-id` flag on `mk encode` would restore full mk1 string identity.
**File it, do not build it** — a host-side change with its own cycle.

**Oracle 2 — published BIP test vectors, for ADDRESSES.** Oracle 1 proves two
implementations agree; it cannot prove both are not wrong the same way. The
constellation's own journeys found four host-side defects (F-127, F-128, F-130,
F-140), so "the host said so" is not ground truth. Published vectors are ground
truth from outside the project.

Measured gap this closes: `address/address_test.go` asserts real derived
addresses for `pkh`, `wpkh` and `wsh` multisig, but its fixtures carry **no
cited provenance** — no BIP reference, no source. `bip380/bip380_test.go` has
two tests, both parsing/compaction, neither citing a BIP. So device address
derivation is currently self-consistent, not standard-conformant.

| BIP | what it ACTUALLY supplies | assertion level | stage |
| --- | --- | --- | --- |
| **383** | `wsh(multi(…))` / `wsh(sortedmulti(…))` vectors | **scriptPubKey**, not address | S2 (Trace A) |
| **67** | deterministic key sorting | key order | S5 |
| **141** | P2SH-P2WSH Example: scriptPubKey + redeemScript | address **derived from** a published vector, not quoted from one — S0 must say which | S3 |
| **39** | mnemonic → seed | seed | already used (`abandon…about`) |

**Corrected 2026-08-13 by the inherited-fact audit, and the correction matters
more than the table.** The previous version cited **BIP-382** for
`wsh(multi(…))` — that is **BIP-383**; 382 is `wsh()` alone and contains no
`multi(`. It also promised **addresses** from 382 and 141/143, which publish
scriptPubKeys and no addresses, and cited **BIP-32** for `m/48'` derivation,
which its vectors never touch. **BIP-48 publishes no vectors at all** — its
Examples table is path semantics with no keys — so no published vector pins
`m/48'`; S0's provenance README must say so rather than imply one was used.

I wrote that table from memory while adding the oracle whose whole purpose was
to stop us trusting ourselves. It is the third instance this cycle of a
plausible, load-bearing, never-executed claim, and the most expensive, because
S0 is the stage every later gate trusts.

**S0 may not quietly relax to "the tests we could write passed."** That is
exactly the failure deliverable 3 names about `address_test.go`'s unattributed
fixtures.

Precedent to model on: `bip341-wallet-test-vectors.json` is already vendored in
md-codec, BIP-173/350 vectors in the vendored bech32 crate, and both
`mnemonic-key` and `mnemonic-secret` carry a `bip-test-vector-audit-matrix`
agent report. The device side has no equivalent; S0 creates one.

**Oracle 3 — an external coordinator, at S6.** Independent of both. Byte-identity
plus an external restore is the two-oracle structure the plan's own review
endorsed.

**Adjudication.** A divergence at any gate is settled **Rust-first**: if the
primary is wrong it is fixed there with a test vector, and the Go change is the
convergence port. A divergence against a published BIP vector outranks both.

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

### S0 — the oracles

Every later gate depends on these, and a gate anchored to a stale or
unattributed oracle is worse than none — it reads as proof.

**Deliverables**

1. **A pinned-oracle harness.** The walk script resolves the primary toolchain
   by version, refuses to run against vendored fork testdata, and **prints the
   resolved oracle versions plus the full input tuple into every gate record**.
2. **Published-BIP address vectors, vendored with provenance**, in the shape of
   `md/testdata/README.md` — source repo, commit, path, per-file meaning — and
   modelled on the existing `bip-test-vector-audit-matrix` reports in
   `mnemonic-key` / `mnemonic-secret`.
3. **A provenance header for `address/address_test.go`'s existing fixtures**:
   either cite where they came from, or replace them with **BIP-383**
   scriptPubKey vectors (compared at scriptPubKey, per §1a — **not** BIP-382,
   which publishes no addresses; an earlier draft said 382 here and the
   correction two sections above did not reach this line).
   Unattributed expected-addresses are self-agreement wearing the costume of a
   test.
4. **The md vendored-vector re-pin: 0.36.0 → current** (`md/testdata/`).
   **S0 owns this**, stated explicitly because round 0 asked for an owner and an
   earlier fold dropped the sentence. Its gate line: `go test ./md/` passes
   against the re-pinned vectors, and `md/testdata/README.md`'s provenance block
   names the new commit and version — a re-pin whose provenance is not updated
   is the drift it was meant to end. If it proves larger than S0 should carry it
   becomes its own stage **before S5** — it may not become an unowned assumption
   again.

   **NOT included, and the reason is worth keeping.** Rounds 0–1 of this plan
   carried a claim that the fork's `mk` decoder needed a `0.2 → 0.4.x/V19`
   re-pin before a depth-0 card could be read. **That is false, and it was
   machine-checked twice** — a round-trip of a `Path == "m"` card through the
   real encoder at `a10d007`, no code changes, encodes to 2 chunks and decodes
   back with `Path == "m"` and the xpub intact. The primary's changelog records
   V19 as "no wire or runtime-behavior change", and the fork's decode path
   handles depth-0 generically. The claim entered from a **stale comment** —
   `mk/mk.go:5`'s `// (family_token "mk-codec 0.2")` — and survived three review
   rounds because every reader, including me, took the comment for the
   mechanism. Second time this cycle after D-5. Grep the mechanism, not the
   claim.

**Tests first**

- `TestBip383WshMultiScriptPubKeyMatchesPublishedVectors` — BIP-**383**'s
  `wsh(multi(…))` / `wsh(sortedmulti(…))` vectors through `bip380`, compared at
  **scriptPubKey**. Not addresses: 383 does not publish them.
- `TestBip67SortedMultiKeyOrder` — BIP-67's ordering vectors. A wrong sort is a
  wrong address, silently, and "sorted" is in the name of the thing we build.
- `TestBip141NestedSegwitScriptDiffersFromLegacy` — BIP-141's P2SH-P2WSH Example
  (scriptPubKey + redeemScript). The address is **derived locally from** that
  vector, not quoted from it; S0's README must record it at that weaker,
  honest level. Anchors S3's D-3 fix below the label.

**Before writing any of the three: open the sources and inventory what they
contain**, and let the test list follow the inventory. The previous test list
followed an author's memory, and two of its three tests were unwritable.
- `TestOracleHarnessRefusesVendoredTestdata` — mutation-checked: point it at
  `md/testdata` and it must fail.

**Gate.** The three BIP vector tests pass; the harness prints oracle versions;
the refusal test is demonstrated failing when pointed at vendored data. **Not
tier 1** if the harness shells out to the primary binaries — mark it and keep it
out of the inner loop (§4.6 tier 2).

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
   differs from the shared origin must not be silently stamped
   `m/48'/0'/0'/2'`. The spec permits refuse OR warn; **this plan picks REFUSE**,
   so the test's name matches its body and the assertion has one arm.
4. **A raster assertion on whatever D-1 turns out to be.** If the defect is a
   screen whose body does not draw, a text assertion cannot see it — F-151.
   Calibrate the floor against the real defect, measured both ways; F-151's
   first guess of 2000 px passed the defect it was written for.

**Gate.** Trace A completes end to end: engrave, by test and by emulator walk.
The md1 is compared by **production**, not acceptance: the current primary
BUILDS an md1 from the same inputs and the strings are equal (§1a). "The host
decodes it" is the weaker relation and does not satisfy this gate.

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
- Delete or correct **all 9** `TYPED-ONLY` occurrences (§2.2 D-5). **Measured,
  not counted from the spec's four cited sites:**
  `gui/multisig.go` ×4, `gui/bip85.go` ×2, `gui/singlesig.go` ×2,
  `gui/multisig_build.go` ×1 — and **none in the verify flows**, which are
  correct by calling `seedEntryFlowTypedOnly` and never used the phrase.
  They describe a retired mechanism, and a future reader greps `TYPED-ONLY`,
  finds hits, and concludes the payload cannot reach a seed entry.
  The spec cites four because four are the *doc-comment* sites it analysed;
  the grep is the authority here, and the gate below is keyed to it. Re-run the
  grep before starting — the count is a fact about the tree, not a constant.

**Gate.** Emulator walk shows `P2SH-P2WSH` on the restore doc for an `sh(wsh)`
build, and **`grep -rn TYPED-ONLY --include='*.go'` returns 0**.
Measured: there are **9** occurrences across 4 files (`gui/multisig.go` ×4,
`gui/bip85.go` ×2, `gui/singlesig.go` ×2, `gui/multisig_build.go` ×1) — not the
4 an earlier draft assumed, and **none in the verify flows**, so the previous
gate line ("returns only the two verify sites") was unsatisfiable and its
premise was wrong. The verify flows are correct because they call
`seedEntryFlowTypedOnly`; they never used the phrase.

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
8. **`TestGateDerivesAtTheCardsOwnOrigin`** — the origin binding, as a
   PROCEED/FAIL pair on one fixture: a `both` slot whose card declares
   `m/48'/0'/1'/2'`.
   **PROCEED** when the key is genuinely derived there.
   **FAIL LOUDLY, naming the slot**, when the same card carries a key derived at
   the shared origin instead.
   **Mutation: make the gate derive at `multisigSharedOrigin()` instead of the
   card's declared origin — the PROCEED case must go red.** Without this, a gate
   wrapper that hardcodes the shared origin passes every other S4 test, because
   S2's interim refusal makes the two values indistinguishable for the whole of
   S4. `findUserSlot` is origin-correct by construction (it derives at each key's
   own `k.OriginPath`); the wrapper built on top of it is new code and is not.

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
   across masters A and B in full mode **engraves both ms1s**. The spec permits
   a refusal arm; **this plan picks the engrave-both arm**, because a test
   asserting a disjunction passes on either branch and so cannot be
   mutation-checked. Losing B otherwise leaves two legs against k=3:
   unspendable, from a backup labelled "Full (seed + keys)".
   The test decodes **each** ms1 and compares its entropy to the master it
   claims. **Mutation:** make the engrave loop capture one mnemonic variable, so
   both ms1s carry master A's entropy — the test must go red. Without that
   assertion the bug ships a "Full" backup that is missing a master and passes
   every other gate in this plan.
6. `TestDepthZeroCosignerCardIsNamedRefusal` — spec M-1: `Path == "m"` trips
   `errMultisigEmptyDivergent` (`md/encode_multisig.go:104-106`); refuse by a
   named screen, not a fall-through "Couldn't assemble". **The premise is already
   sound**, checked rather than assumed: a depth-0 card round-trips through the
   fork's encoder today (S0 deliverable 4's note), so the flow reaches
   `errMultisigEmptyDivergent` and this test has something to assert against. No
   re-pin gates it.

**Implementation**

- `buildPolicyParams.SelfSlot int` → a set of held slots.
- `cosignerFromCard` stops discarding `card.Origin`; `OriginDivergent` when
  origins differ, `OriginShared` when they do not.
- The tail: `deriveMultisigLeg` per held slot at that slot's origin; ms1 per
  distinct master in full mode.
- Remove S2's interim foreign-origin refusal, which this stage supersedes.

7. **`TestGateStillFiresAfterOriginsDiverge`** — S4 test 8's fixture, re-run
   through the REAL post-rewire flow rather than synthetically: the same `both`
   slot declaring `m/48'/0'/1'/2'`, **PROCEED** when honestly derived there and
   **FAIL naming the slot** when not, **mutation-checked the same way**.
   The specificity is the point: "the gate still fires" is satisfiable by
   `assemble(divergentInput); assertNoError()` — a smoke test that never checks
   WHICH origin the gate derived against, which is the binding the whole check
   exists to protect.
   Why it must be re-proven at all: S2's interim refusal means a divergent-origin
   input cannot reach the gate during S4, so every S4 gate test is necessarily
   synthetic. S5 removes that refusal and rewires the origins the gate derives
   against, so S5 is the first stage where the gate runs for real — and if it is
   not re-proven here, S4's proof expires silently.

**Gate.** Trace B completes: correct descriptor, by test and by emulator walk.
**The §4.5 comparison extends to every mk1 and to EVERY ms1, byte for byte**
(§1a): each engraved ms1 must equal `ms encode --hex <that master's entropy>`
from the current primary, and each mk1 must satisfy the two-part mk1 relation.
"ms1 presence" was this plan's earlier wording and it was a defect, not a
scoping — the spec's presence requirement is a floor, and byte comparison
satisfies it a fortiori. The md1 alone cannot see either C2 scenario.

---

### S6 — hardware validation

**Not tier 1.** One flash cycle, via `~/bin/sh/sh2-flash` (never `picotool` by
hand — the build output is unsigned).

1. Engrave and restore a `wsh` multisig; verify against an external coordinator.
2. Engrave and restore an `sh(wsh)` multisig; same. Confirms S3 on the plate,
   not just the screen.
3. **At least one build MUST be divergent-origin, multi-slot and multi-master**
   (§6 P5). A shared-origin single-seed run would pass green around every
   §4.1a failure. **In the same flash cycle, restore master B's mnemonic from
   its engraved ms1 plate** — the ms1 class is the least-gated artifact (C1),
   and a plate nobody has read back is a plate nobody has tested.

**Gate.** All three restore correctly at an external coordinator, **and master
B's mnemonic restores from its ms1 plate**. This confirms software already
proven; it is not the first place the flow is executed.

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
