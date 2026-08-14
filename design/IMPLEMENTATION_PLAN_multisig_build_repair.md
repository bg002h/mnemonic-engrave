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

**PIN BY SOURCE COMMIT, NOT BY `--version`.** A version string is
self-reported by the binary, so a substituted tool spoofs the pin and
launders a device defect through every byte-identity gate in this plan — the
whole gate spine, defeated by one file on `PATH`. The walk script MUST
resolve each oracle to a source commit (build from a pinned checkout, or
record and check a binary hash) and print that commit, not the version, into
the gate record. S0 exists so gates are not anchored to something
untrustworthy, and an unauthenticated self-report is exactly that.
**The walk script MUST print the resolved oracle SOURCE COMMITS into every gate
record** — not version strings, which are self-reported and therefore spoofable
(see the pinning rule below) — so a stale or substituted oracle is visible
rather than silent, and MUST record the full input tuple
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
gate includes the §4.5 emulator walk.

**A walk's expected artifact census MUST derive from the recorded input
tuple, never from what the walk produced.** And where a stage's walk produces
**no** artifact — S1 and S3 end at a screen, not an engrave — the census is
inert, so those walks assert on `shScreen()` text at a named screen instead. A
gate whose only check cannot fire is not a gate; say which check each stage's
walk actually runs. "Every mk1 and every ms1
matched" is vacuously true of a walk that fell over after plate one. The
script computes how many md1 chunks, mk1s and ms1s the inputs REQUIRE and
fails when fewer arrive — a partial walk may never satisfy a total gate.

Tests are tier 1 (§4.6) unless named
otherwise — synthetic time via `testing/synctest`, no real sleeps.

---

### S0 — the oracles, and the harness that uses them

Every later gate leans on S0, and a gate anchored to a stale, unattributed or
absent oracle reads as proof while proving nothing. **Round-0-through-3 of this
plan named S0's deliverables without opening any of them**; the journeys and
spec-coverage lenses then found, independently, that §4.5's walk had nothing to
walk with. S0 is written out here in full for that reason.

**Deliverables, in order. 1 precedes 2 and the order is load-bearing.**

1. **Make the test-payload confinement guard STRUCTURAL, before adding a second
   payload.** Today's guard is keyed to a literal:
   `names := []string{"syswTestPayload", "syswTestDigest", "sysw_test_payload.bin"}`
   — a hand-maintained list, and hand-maintained lists go stale (this cycle
   found "the four `TYPED-ONLY` comments" was nine). Adding a fourth name lets
   the FIFTH blob escape.
   Derive the protected set from the tree instead: **every `//go:embed` under
   `cmd/emu` must live in a `//go:build js` file, and its identifiers must not
   appear outside a small allowed js-only set.** Keep a `checked < 50`-style
   floor so a misrooted walk cannot pass vacuously.
   **Mutation proof comes free from deliverable 2:** write the guard, point it
   at the new unconfined blob, watch it go red, then confine the blob and watch
   it go green. A shipped SeedHammer II must never boot carrying a payload
   somebody else packed, so this guard earns a mutation check more than most.
2. **A SECOND js-only test payload carrying cosigner cards.** `cmd/emu`'s
   existing blob holds exactly `ClassMnemonic`, `ClassPassphrase`,
   `ClassFreeText` and **zero `ClassMDMK`** (265 bytes, verified), so no walk can
   reach a cosigner gather — Trace A halts one screen before D-1.
   **It must be a second blob, not an edit to the first:** the first's digest is
   pinned in `cmd/emu/sysw_test_payload.go` and photographed in the published
   Load Payload journey PDF, and mutating it makes that document wrong.
   **It must satisfy EVERY stage's walk, not just S1's.** Enumerated, because
   getting this wrong reproduces the defect one stage later: S1/S2 need Trace
   A's two cosigner cards; **S3 needs an `sh(wsh)` build** (its gate asserts
   `P2SH-P2WSH` reaches the restore doc); **S4 needs a `both`-slot case** — a
   card whose key genuinely derives from a payload seed, plus a sibling that
   does not, so the gate can be walked in both directions; S5 needs Trace B's
   multi-account, multi-master set. Build the inventory from that list.
   **State the record inventory in its provenance comment** — how many mk1
   cards, at which origins, for which traces — so a future reader learns the
   contents without opening the blob. Not stating it is exactly how this
   Critical happened. Expect it to be materially larger than 265 bytes: every
   mk1 carrying an xpub is ≥2 chunks, and Trace B needs several cards.
3. **A walk harness that can actually drive the emulator — with these shapes,
   not "an API".** §4.5 requires the walk be AUTOMATED. Today `cmd/emu` exposes
   only `window.shNFC` and `window.shToolpath`, input is raw canvas pointer
   events, and **nothing returns an engraved string**, so §4.5's byte comparison
   has no mechanism. **None of the following exist yet; all are S0's to write:**

   | shape | purpose |
   | --- | --- |
   | `window.shTap(x, y)` | drive input at device coordinates, so a walk is a script rather than synthetic pointer events |
   | `window.shScreen()` | read the current frame's text, so a walk can assert where it is and fail informatively when it is somewhere else |
   | `shToolpath.strings()` | **the engraved md1/mk1/ms1 strings out of a completed walk** — the mechanism §4.5's byte comparison currently lacks. Reaches the recorder through `gui.PlateAware`, which exists in the emulator build and not the firmware's |
   | `window.shSysw(blob)` | choose which payload a walk runs against, instead of depending on whichever blob was compiled in |

   A named API is not a buildable deliverable. These four are the deliverable.
4. **The frame receiver keeps its existing security properties** (§4.6 SAFE):
   pinned to one origin, flat filenames only, resolved-path re-check.
   `design/journeys/shot_server.py` is the precedent and its docstring states
   why both restrictions are load-bearing. A new harness may not quietly drop
   them.
5. **Oracle resolution BY SOURCE COMMIT, not by `--version`.** A version string
   is self-reported, so a substituted binary spoofs the pin and launders a
   device defect through every byte-identity gate in this plan. The harness
   resolves each oracle to a source commit — a pinned checkout, or a recorded
   and checked binary hash — refuses to run against vendored fork testdata, and
   **prints the resolved commits plus the full input tuple into every gate
   record**.
6. **Published-BIP vectors, vendored with provenance**, in the shape of
   `md/testdata/README.md` — source repo, commit, path, per-file meaning —
   modelled on the `bip-test-vector-audit-matrix` reports in `mnemonic-key` /
   `mnemonic-secret`. **Open the sources and inventory them before writing the
   test list** (§1a): the previous list followed an author's memory and two of
   its three tests were unwritable.
7. **A provenance header for `address/address_test.go`'s existing fixtures**:
   either cite where they came from, or replace them with **BIP-383**
   scriptPubKey vectors. Unattributed expected-addresses are self-agreement
   wearing the costume of a test.
8. **The md vendored-vector re-pin: 0.36.0 → current** (`md/testdata/`). Its
   gate line: `go test ./md/` passes against the re-pinned vectors AND
   `md/testdata/README.md`'s provenance block names the new commit and version.
   Measured 2026-08-13: 0.36 → 0.42 shows **zero byte drift** across all 30
   vectors, so this is a **coverage catch-up, not a correctness repair** — the
   vendored vectors are an older, smaller sample, and a gate accepting them
   would prove agreement with a subset of ourselves.

   **NOT included:** rounds 0–1 carried a claim that the fork's `mk` decoder
   needed a `0.2 → 0.4.x/V19` re-pin before a depth-0 card could be read. It is
   **false**, machine-checked twice — a `Path == "m"` card round-trips through
   the real encoder at `a10d007` with no code changes. The claim came from a
   stale comment (`mk/mk.go:5`) and survived three rounds. Grep the mechanism,
   not the claim.

**Tests first**

- `TestEmbeddedPayloadsAreStructurallyConfined` — deliverable 1. Discovers every
  `//go:embed` under `cmd/emu`, requires each in a `//go:build js` file with
  identifiers unreferenced outside the allowed set, floor against a misrooted
  walk. **Mutation: the unconfined second blob must turn it red.**
- `TestCosignerPayloadCarriesTheTracesCards` — the second blob decodes, and
  carries the mk1 count and origins Trace A and Trace B each need. Pins the
  inventory so a later shrink cannot silently strand a walk.
- `TestWalkHarnessDrivesAndExtracts` — the harness can drive input and return an
  engraved string from a completed walk. Without this, every "byte comparison"
  in this plan is unimplementable.
- `TestBip383WshMultiScriptPubKeyMatchesPublishedVectors` — BIP-**383**'s
  `wsh(multi(…))` / `wsh(sortedmulti(…))` vectors through `bip380`, compared at
  **scriptPubKey**. Not addresses: 383 does not publish them.
- `TestBip67SortedMultiKeyOrder` — BIP-67's ordering vectors. A wrong sort is a
  wrong address, silently, and "sorted" is in the name of the thing we build.
- `TestBip141NestedSegwitScriptDiffersFromLegacy` — BIP-141's P2SH-P2WSH Example
  (scriptPubKey + redeemScript). The address is **derived locally from** that
  vector, not quoted from it; S0's README records it at that weaker, honest
  level. Anchors S3's D-3 fix below the label.
- `TestOracleHarnessRefusesVendoredTestdata` — mutation-checked: point it at
  `md/testdata` and it must fail.
- `TestOracleHarnessPinsBySourceCommit` — a binary whose self-reported version
  matches but whose source commit does not must be refused.

**Gate.** All eight tests pass; the confinement mutation is demonstrated red
then green; the harness prints resolved oracle **commits** and the input tuple
into the gate record; and **one end-to-end smoke walk drives the emulator to a
completed engrave and returns the md1 string**. That last clause is the whole
point of S0 — until a walk can produce an artifact, no later stage's walk gate
means anything.

**Not tier 1** (§4.6): the harness shells out to primary binaries and builds
wasm. Mark it tier 2 and keep it out of the inner loop.


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
7. `TestUnderSupplyRefusalNamesTheHostRoute` — **refusals must speak phase-1
   language.** Today's tell the operator to *scan* a card, an instruction phase 1
   removed with NFC. A payload holding 3 of 4 needed cards, or a card whose chunk
   set is incomplete, must name the only route that exists: rewrite the payload
   on the host and deliver it again. A refusal prescribing an impossible action
   is worse than a bare failure — the operator goes looking for a reader that is
   not there.

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
- **Preserve the per-key source seam while replacing its picker.** S1 removes
  the `syswOffer(…ClassMDMK…)` call, today the only per-key source choice point.
  Deleting it outright guarantees the later NFC plan must re-open this call site
  — the one thing §5.1's build-it-once seam exists to prevent. Replace the
  picker, keep the seam: one place answering "where does this key come from",
  with payload as its only phase-1 answer.

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
4. **`TestS2RefusesDuplicateKeysBeforeS4`** — the duplicate-key window. **No
   duplicate check exists anywhere in the code today**, and §4.1's final-slot-set
   check does not land until S4 — so from S2, which makes engraves complete,
   until S4 the flow would silently engrave a policy containing the same key
   twice. That is the quorum degradation §4.1 exists to refuse:
   `sortedmulti(2,K,K,X)` is spendable by K alone. The interim check is a byte
   comparison over the assembled slots and depends on nothing S4 introduces, so
   it costs S2 almost nothing and closes a window that would otherwise ship.
5. **A raster assertion on whatever D-1 turns out to be.** If the defect is a
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

**Gate.** Emulator walk of an **`sh(wsh)` build** — a shape neither Trace A nor
Trace B carries, so S0's payload must supply it (S0 deliverable 2) — showing
`P2SH-P2WSH` on the restore doc, and **`grep -rn TYPED-ONLY --include='*.go'` returns 0**.
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
8. **`TestBuildFlowScrubsEverySeedOnEveryExit`** — **spec §4.2's mandated test,
   which no stage of this plan had claimed.** `grep -i scrub` over the plan
   returned zero before this line, while §4.2 is normative REQUIRED and ends "A
   test MUST prove the scrub, and that test MUST be mutation-checked." Today's
   single-seed `defer` (`gui/multisig_build.go:75-79`) is sound; S4 and S5
   replace it with a `seedID`-keyed registry holding several masters and
   multiply the exits — per-slot seed entry Back, slot review Back, gate FAIL
   screens, tail abort, `ctx.Done` unwind. A registry missing one exit leaves N
   masters' seeds in RAM and nothing else would notice.
   Observe every entered seed through the existing `buildMultisigSeedHook` seam,
   assert zeroed on each exit class, **mutation-checked by deleting one scrub
   site**. Precedent: `TestBip85DeriveFlow_ScrubsBothMnemonics`.
9. **`TestSeedEntryScreensNameTheirSlot`** — with several seeds in one flow,
   every seed-entry and passphrase screen must say WHICH SLOT it is for.
   Unlabelled, the operator cannot tell the second seed prompt from a repeat of
   the first, and a passphrase entered against the wrong slot mints a key no
   §4.3 row can catch (there is no card to cross-check a new-seed slot against).
10. **`TestPerSeedPassphraseBindsToItsOwnSeed`** — §4.1's per-seed passphrase,
   which had an implementation bullet and no test. The spec says no §4.3 row can
   catch a violation, so this is the only thing that can: two seeds with
   different passphrases derive two different keys, and a flow-global passphrase
   applied to both must turn it red.
11. **`TestGateDerivesAtTheCardsOwnOrigin`** — the origin binding, as a
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

**The slot-source screen must not speak spec language.** `payloadKey`,
`derived(seedID, account)` and `both` are names for a data model, not for what
an operator is choosing. On screen they must read as the decision being made —
whose key this slot is, and where it came from — or the operator either
approves a model they have not understood or is alarmed by a distinction that
does not concern them.

**Every comparison the device asks for must be one the operator can perform.**
S5's "unambiguous digest" arm and the inherited "match your coordinator" line
both name checks with no artifact on the other side: the operator has nothing
to compare a digest against. Either the device shows something their coordinator
also shows — the descriptor, or the per-slot keys — or the instruction is
removed. This is the fingerprint defect (A1) one level up, and it is the third
instance of it in this plan; treat "verify X" as a claim that X is obtainable.

**The gate's FAIL screen must not make silencing it the obvious next move.**
After a seed↔key mismatch the only route the operator can currently see is
reassigning that slot to `payloadKey` — which stops the check running rather
than resolving the disagreement. The screen MUST name the likely causes (a
mistyped or wrong-seed passphrase; a card from a different wallet), say plainly
that reassigning the slot **suppresses the check rather than fixing it**, and
name the host route. A safety gate whose obvious next action disables it is not
a gate.

**Tell the operator how many plates before the tail starts, and what the set
is afterwards.** Trace B cuts 6–9 plates over hours. The operator gets no count
before committing and no inventory after, so neither they nor the person who
finds the plates in five years can tell whether the set is complete. A census
before the engrave and a set inventory on the restore doc — F-131 and F-132 are
both cases where a restore document's silence cost more than its errors.

**Bound the walk-away.** `wipeGuard` brackets only the unlock session, while
this flow holds **several masters' seeds** in its registry with no time bound —
an operator who walks away mid-build leaves them live indefinitely. S4 owns the
registry, so S4 rules the bound: an idle limit that scrubs and exits, or an
explicit recorded decision that the build flow is non-wiping like the rest of
the systemwide surface (SYSW§3.2.1), stated in the restore doc. Silence is the
one option unavailable — it is a choice nobody made.

**Gate.** Every failing row demonstrated failing. Emulator walk of the `both`
happy path and of one loud failure — **a `both` slot is in neither trace, so
S0's payload must supply the case** (S0 deliverable 2): a card whose key
genuinely derives from a payload seed, and a sibling that does not.

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

7. **`TestReRunMintsByteIdenticalPlates`** — the designed answer to
   interruption. Trace B's tail is **6–9 plates over hours**; nothing records
   which were cut, and a power loss loses that state. Recovery is nonetheless
   possible because **the encoders are deterministic** — no `rand` in `md/`,
   `mk/` or `codex32/`, and mk's `chunk_set_id` derives from the bytecode rather
   than randomness — so re-running the same inputs mints byte-identical plates
   and the operator re-cuts only what is missing. **That property is
   load-bearing and currently unpinned:** assert it, and put the recovery
   procedure where an interrupted operator will actually see it. **Not only the
   restore doc** — that is printed at the end of a successful run, and an
   operator whose engrave died never reaches it. The abort screen currently says
   "discard… start over"; it must instead say that re-running the same inputs
   reproduces the same plates, so only the missing ones need cutting.
   The emulator's existing `shToolpath` digest-equality check is the tool.
8. **`TestGateStillFiresAfterOriginsDiverge`** — S4's `TestGateDerivesAtTheCardsOwnOrigin` fixture, re-run
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

**The backup must say what is NOT in it.** A BIP-39 passphrase is a required
spending factor, it is **never engraved**, and neither "Full (seed + keys)" nor
the restore document mentions it. F-132's device sibling exactly — that finding
was a hashlock preimage required to spend, absent from the backup and unmentioned
by it. Where a passphrase was used, the mode label and the restore doc MUST both
say the backup is incomplete without it. The restore doc is read years later,
alone, often by someone who was not the operator.

**The EXPERIMENTAL warning must stop teaching a check that cannot check.**
It currently tells the operator to verify per-slot fingerprints before funding.
Fingerprints are **omitted by default**, and when included they are
**card-self-declared and unbound to the key** (`mk/mk.go:136,286`) — an attacker
forges a matching one for free. Meanwhile the pre-engrave review shows **no keys
at all**, so the operator confirms a policy whose contents they have never seen.
S5 must: (1) show the per-slot keys, or an unambiguous digest of them, on the
review screen; (2) rewrite the warning to demand a **key or descriptor
comparison against an independent source**, and say plainly that a matching
fingerprint is not verification. The external-coordinator restore at S6 is the
real backstop, and the warning should name it as such.

**Plate order and abort text — RULED, because today's behaviour instructs seed
leakage.** Full mode cuts the **ms1 seed plate FIRST**, and the abort warning
says "discard the engraved plate(s)". Correct for a public plate; for one
carrying a seed it tells the operator to bin their secret. Nothing distinguishes
them.

**SHIPS HERE — the wording.** For any set containing an ms1, the abort warning
says **DESTROY, not discard**, for any secret plate already cut. Not a new
requirement: "discard the engraved plate(s)" is **wrong today** for a plate
carrying a seed, so this corrects shipped text — and it is the half that
prevents the harm, because a warning saying destroy protects the operator
whatever order the plates came out in. Cards-derived per the existing R0-I2
pattern, so no other flow's call site changes.

**DEFERRED — the reordering.** "Public plates first, secret last" is a design
change, not a text fix: it alters what exists in the world at each moment of an
abort, and it edits `multisigEngraveCards`, shared with `gui/multisig.go:163` —
a flow this plan does not own. It also rests on "ms1-first is inherited
convention, not a ruling", which nobody checked, and there is a plausible reason
for the current order (the seed plate is the longest cut; failing early on it
beats failing after four public plates). **Filed for the spec, with the
shared-code impact noted, to earn its own R0.**

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
3. **Confirm the interruption story on hardware.** S5 asserts that re-running
   the same inputs mints byte-identical plates — the only recovery route an
   interrupted operator has. Interrupt one real engrave mid-set, re-run, and
   confirm the re-cut plates match. In software it is deterministic encoders; on
   the machine it is what a recovery depends on.
4. **At least one build MUST be divergent-origin, multi-slot and multi-master**
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
