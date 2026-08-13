# SPEC — on-device wallet-policy authoring: repair, then completion

Status: **DRAFT, pre-R0.** Written 2026-08-13 from a brainstorm with the
operator. No code may be written until this passes the R0 gate at 0 Critical /
0 Important (project rule: risk-set work — keys, derivation, seed material,
many files).

**Reference convention:** `SYSW§` means a section of
`SPEC_systemwide_payloads.md`; a bare `§` means a section of THIS document.
`F-nnn` is an entry in `design/FOLLOWUPS.md`.

## 0. What this is, in one paragraph

The SeedHammer II can already *consume* a wallet policy someone else authored
and engrave the operator's leg of it. It can also, in principle, **author** one
— `buildMultisigPolicyFlow` exists, and is reached from Engrave Multisig →
"Build policy". That path is the subject of this spec. It has one severe defect
(it dead-ends), one structural limit that makes it unable to express the wallets
this project already ships journeys for (it assumes the operator holds exactly
one key), one labelling defect that makes a nested-segwit wallet indistinguish-
able from a legacy one in the **restore document**, and — most importantly — it
has never been executed end-to-end by any test or in the emulator, which is why
those defects reached hardware.

## 1. The goal, in two phases — NORMATIVE

Recorded because the phase boundary decides what is built now and what is
deliberately not.

1. **Phase 1 (this spec): on-device MULTISIG WALLET DESCRIPTOR generation.**
   The operator builds a k-of-n sorted-multisig wallet policy on the device,
   holding **one or more** of its keys, with the other cosigner keys arriving
   **by payload** (§3 — NFC is out of phase 1). The result is an md1 the
   operator can engrave and later restore. Nothing exotic: no timelocks, no
   hashlocks, no composition.
2. **Phase 2 (a subsequent cycle): arbitrary `wsh` or `tr` miniscript policies.**
   The same constructor, generalised so the operator can author policies with
   timelocks, hashlocks and composition operators under either a `wsh` or a `tr`
   root.

Phase 2 is **not** in scope here and no phase-1 work may be justified by it. See
§9 for what phase 2 needs and why it is deferred, including the constraint that
governs it.

## 2. Current state — MEASURED, not described

Every claim in this section was verified against the source or by running it on
2026-08-13. File:line citations are to
`/scratch/code/shibboleth/seedhammer` at `a10d007`.

### 2.1 What already works, and is better than expected

- **The wire format already speaks taproot and miniscript.** `md/md.go:41-49`
  defines `tagTr = 0x01`, `tagMultiA = 0x08`, `tagSortedMultiA = 0x09`, and the
  decoder handles `tagTr` at nine sites.
- **The encoder already serializes every construct.** `md/encode.go:159`
  `writeNode` is recursive and has arms for `trBody` (taproot, with the
  `is_nums` flag and an optional subtree), `timelockBody` (`after`/`older`),
  `hash256Body`, `hash160Body`, `variableBody` (`thresh`), `multiKeysBody`
  (multi / sortedmulti / multi_a / sortedmulti_a) and `childrenBody`.
- **The emitter is generic over the tree.** `md/chunk.go:121` `split(d)` and
  `FormAwareStub(d)` take a whole `descriptor` and do not branch on shape.
- **`EncodeMultisig` is thin.** `md/encode_multisig.go:89-164` is: build a
  `pathDecl` per origin mode, call `multiSigTree` (the ONLY shape-specific
  line), assemble a generic `descriptor`, `split`, `FormAwareStub`.
- **Divergent origins are already implemented.** `md/encode_multisig.go:32-63`
  defines `OriginShared` and `OriginDivergent`, with a per-cosigner
  `Origin []PathComponent`.
- **Seed-to-key matching already exists.** `gui/multisig_match.go:34`
  `findUserSlot` derives the account xpub from a mnemonic at each key's
  `OriginPath` and compares chain code ‖ pubkey. It uses no fingerprints (§4.3
  explains why that is the correct choice, not an oversight).
  **`reused []int` does NOT mean "key reuse defect".** Its own doc comment
  (`gui/multisig_match.go:24-29`) rules the opposite: "`>=2` matches → the SAME
  seed **legitimately** appears at >=2 cosigner slots under DISTINCT origins…
  every matched index in `reused` so the caller can **show a notice**." It is a
  notice for the multi-account shape §4.1 exists to build. Round 0 of this
  spec inverted that into a refusal; §4.1 and §4.3 now key on the correct
  discriminator instead.
- **Payload-sourced seeds already reach this flow.** `gui/multisig_build.go:68`
  calls `seedEntryFlow`, which routes through `syswSeedPicker` and offers FROM
  PAYLOAD whenever the session holds `ClassMnemonic`. See §2.2 D-5: the
  `TYPED-ONLY` comment above that line is stale, and the absence of §4.3's gate
  is therefore a **live exposure**, not a future one.
- **The admission table already permits what §5 needs.** SYSW§3.3.2 admits
  `ClassMnemonic`, `ClassCodex32Secret`, `ClassPassphrase`, `ClassDescriptor`
  and `ClassMDMK` to Engrave Multisig. SYSW§3.3.2 states that "an admitted cell
  is a PERMISSION, not a promise that every screen offers it" — so §5 adds a
  **carrier**, not a permission, and needs no admission change and no
  Rust-primary cycle.

### 2.2 The defects

**D-1 — the flow dead-ends (F-150 item 1).** Field-observed on hardware: a blank
screen after configuration. **NOT YET REPRODUCED** — see §2.3 for why nobody
can reproduce it today, which is itself the finding. Not to be designed against
until reproduced (§6, P1).

**D-2 — one self key is baked into the type (F-150 item 2).**
`gui/multisig_build.go:338-344` `buildPolicyParams.SelfSlot` is a single `int`,
and `gui/multisig_build.go:61` requires exactly `p.N-1` gathered cosigner cards.
The deeper cause is the origin model: `gui/multisig_build.go:421-424`
`multisigSharedOrigin()` is a locked `m/48'/0'/0'/2'`, and
`gui/multisig_build.go:437` `cosignerFromCard` **discards each card's own
origin** ("The card's Origin is IGNORED (OriginShared mode declares the single
shared origin)"). So two keys held by one operator would have to sit at the same
path — which is the same key twice, not a multisig. `OriginDivergent` exists and
is never used.

**D-3 — nested segwit is indistinguishable from legacy P2SH in the restore
document.** `gui/md1_inspect.go:20` `scriptName(k md.ScriptKind)` takes only the
root kind, so it cannot see `Template.InnerWsh`/`InnerWpkh`. **Measured** by
running the three summary surfaces against hand-built templates on 2026-08-13:

```
wsh(sortedmulti)       -> "P2WSH 2-of-3 multisig (sorted)"
sh(wsh(sortedmulti))   -> "P2SH 2-of-3 multisig (sorted)"
sh(sortedmulti) legacy -> "P2SH 2-of-3 multisig (sorted)"
sh(wpkh) BIP-49        -> "P2SH single-key"
```

The middle two are **byte-identical strings for two wallets that hash to
different addresses**. `md/md.go:1212-1219` says exactly this and instructs
consumers otherwise: "they hash to DIFFERENT addresses, so a consumer building a
`*bip380.Descriptor` MUST use this to pick P2SH_P2WSH vs P2SH and never verify
one against the other." Three callers: `gui/md1_inspect.go:58`,
**`gui/multisig_restore.go:51` (the restore document)**, `gui/bundle.go:315`.

`gui/md1_expand.go:112` *does* honour `InnerWsh` and maps to
`bip380.P2SH_P2WSH`, and `gui/multisig_derive.go:32` `deriveMultisigLeg` passes
the md1 through verbatim and derives no addresses. **So the engraved steel is
correct and only the description of it is wrong** — F-131's shape: a document
that tells the operator something false about a correct backup.

**D-4 — wrong title mid-flow.** The Build-policy cosigner gather is titled
"Engrave Bundle". Observed in the emulator 2026-08-13.

**D-5 — four `TYPED-ONLY` comments describe a mechanism that was retired, and
one of them guards a live exposure.** Found by the R0 round-0 reviewer;
**verified independently before folding**. All four cited sites call the
MULTI-SOURCE `seedEntryFlow`, not `seedEntryFlowTypedOnly`:

```
gui/bip85.go:271          seedEntryFlow   (comment at :264 says "TYPED-ONLY master (I-3)")
gui/singlesig.go:33       seedEntryFlow   (comment at :18  says "TYPED-ONLY seed (D12)")
gui/multisig.go:103       seedEntryFlow   (comment at :24  says "TYPED-ONLY seed (I-7)")
gui/multisig_build.go:68  seedEntryFlow   (comment at :67  says "TYPED-ONLY self seed (I-SCRUB)")
```

Only the two **verify** flows use `seedEntryFlowTypedOnly`, and that is
deliberate and correct (`gui/derive_xpub.go:112-123`: a verify accepting the
payload source would compare the engrave source against itself and pass
unconditionally).

This is the `comments-outlive-their-conditions` class, and round 0 of this spec
walked straight into it — §4.2, §5.2, R-1 and P4 were written from the comments
rather than from the mechanism, which is precisely what this project's rules
forbid. Corrected throughout.

**The consequence is a live exposure, not a documentation defect.** A payload
may carry a `ClassMnemonic` record AND `ClassMDMK` cards; the build flow will
today take the seed from the payload and cards from the payload, and **nothing
cross-checks them**. §4.3's gate closes a hole that exists in shipped firmware.

### 2.3 Why the defects reached hardware — the finding that reorders the plan

Three facts, each verified:

1. **`cmd/emu` cannot deliver a card to any gather.** Walked in the emulator
   2026-08-13: with a valid mk1 presented via `window.shNFC` both *before* and
   *after* entering the gather, the tally stayed `md1 descriptors: 0 / mk1 keys:
   0` and **no `nfc scan:` log line ever appeared**, meaning
   `gui/nfc_scan.go:45` `startScanner` received `nil` and never polled. Not
   specific to Build policy: plain Engrave Bundle behaves identically.
2. **The host test platform has no reader either** — `gui/gui_test.go:445`
   `testPlatform.NFCReader()` returns nil. `gui/bundle_flow.go:96` and
   `gui/mk1_inspect_test.go:104` both record the consequence.
3. **The only test that drives `buildMultisigPolicyFlow` end-to-end stops at the
   gather.** `gui/multisig_build_flow_test.go:199`
   `TestBuildFlow_GatherBeforeSeed`, whose own comment reads: "with no NFC reader
   the gather yields zero cards, so a Build flow at n=2 returns on gather Back
   WITHOUT typing a seed (the seed-hook never fires)."

A prior session already recorded the mechanism at `gui/nfc_scan.go:25-27`: "an
attempt in a real browser did not reach this loop at all, because a screen
fetches `Platform.NFCReader()` once at entry and `cmd/emu`'s source returns nil
until a record is pending."

**Therefore everything after the gather — cosigner decode, seed entry, key
derivation, `assembleBuildPolicy`, the review screen, template consent, the
experimental warning, engrave, verify offer, restore doc — has never executed
anywhere except on the operator's machine, by hand.** That is the direct cause
of D-1 reaching hardware, and it makes drivability the first stage rather than a
convenience.

**This section diagnoses the problem; it no longer prescribes the fix.** Phase 1
drops NFC (§3), so drivability is reached through the payload instead, and the
NFC harness work moves to its own later plan with F-158. §3.1 is the substitute
route and the evidence that it already exists on both sides.

## 3. Scope

**In scope (phase 1):** D-1 through D-4; multi-slot self keys with divergent
origins; **the systemwide payload** as the source for cosigner keys and for
seeds; the seed↔key consistency gate; emulator drivability; a hardware
validation gate.

**NFC is OUT of phase 1 entirely — operator decision, 2026-08-13.** *"We can
make NFC a secondary priority and omit it entirely from the first attempt at
getting user the wallet descriptor abilities. Payload will suffice for now and
nfc work can come in a separate plan at a later date."*

This is a larger simplification than it looks, and §3.1 records why.

**Also out of scope, and may not be smuggled in:** taproot authoring; any
miniscript operator; the recipe/tree-constructor generalisation (§9); a
miniscript type checker; `ClassCodex32Secret` as a payload-delivered seed
(§5.4); changes to Sealed Payload; any change to the md1 wire format.

### 3.1 Dropping NFC removes the hardest stage, not a feature

§2.3's finding was going to make "build an NFC harness" the first stage,
because the cosigner gather had no other source. With the payload as the source
instead, that stage disappears — the seam it needed **already exists on both
sides**. Verified 2026-08-13:

- **A payload can carry cosigner cards.** `me sysw pack` accepted two mk1 chunks
  and `me sysw show` reports them as `public record 0: md1/mk1 — confirmed`,
  `public record 1: md1/mk1 — confirmed`. Chunked cards are ordinary records.
- **Host tests can supply one.** `gui/gui_test.go:453`
  `func (p *testPlatform) SyswReader() sysw.Reader { return p.sysw }` — a real,
  settable field. This is the asymmetry with NFC, whose `NFCReader()` returns
  nil (`gui/gui_test.go:445`) and is why §2.3 happened.
- **The emulator can supply one.** `cmd/emu`'s `embeddedSyswReader` is working
  and was walked end to end on 2026-08-12 for
  `design/journeys/SeedHammer-II-load-payload-journey.pdf`.

So the flow becomes drivable by a test and in a browser **without touching the
NFC harness at all**. F-158 stays filed and moves to the later NFC plan; it is
no longer on this spec's critical path.

**The operator-workflow consequence, stated rather than discovered later:** with
NFC gone, cosigner keys reach the machine only by payload, which is written from
a host over USB in BOOTSEL. Building a multisig on the device therefore requires
a host step in phase 1. That is a real narrowing of the air-gapped story and it
is reversible — the later NFC plan restores tag-in-hand gathering.

## 4. Normative requirements

### 4.1 Multi-slot self keys — REQUIRED

The operator MUST be able to declare that they hold **one or more** of the n
slots. For each slot they hold, the key source is either:

- **a new seed**, entered by any admitted source (§5), or
- **another account index of a seed already supplied in this flow.**

This is the shape of the constellation's own pathological example (11 keys, 3
masters, four account indices), which `design/journeys/` already ships end to
end on the host.

Consequences that are themselves normative:

- **`OriginDivergent` MUST be used whenever the declared origins are not all
  equal.** `OriginShared` remains correct, and MUST be used, when they are.
- **`cosignerFromCard` MUST stop discarding the card's origin** when the policy
  is divergent. Which origin wins — the card's or the flow's — is **RULED in
  §7 R-3**, because the host side has the same question open as F-129.
- **A depth-0 cosigner card MUST be refused by a named screen.** mk1 permits
  `Path == "m"`, which yields zero components and trips
  `errMultisigEmptyDivergent` (`md/encode_multisig.go:104-106`) in divergent
  mode. Refusing is correct; falling through to "Couldn't assemble the wallet
  policy" is not, since §4.3 would rightly call that a silent-ish failure.
- **The duplicate-key check is over the FINAL SLOT SET, not over seed matches.**
  **REFUSE iff any two final slots carry an identical 65-byte
  chain code ‖ pubkey.** That is exact, source-independent, and subsumes every
  arrival shape — seed+card, card+card, seed+seed. A duplicated key makes a
  displayed k-of-n a (k−1)-of-(n−1) against its holder: `sortedmulti(2,K,K,X)`
  is satisfiable by K alone, because two signatures under K validate against the
  two K entries. On steel, that is quorum degradation the operator cannot see.
- **The same seed at ≥2 slots under DISTINCT origins is LEGITIMATE and MUST
  proceed**, with the notice `findUserSlot` already specifies. It is the
  multi-account wallet this section exists to make buildable. Round 0 of this
  spec refused it; see §2.1.
- **The passphrase prompt is PER SEED**, asked at that seed's entry, and the
  `(seed, passphrase)` pair is the derivation unit everywhere this spec says
  "seed". One flow-global passphrase applied to N seeds would mint keys the
  operator can only re-derive with a pairing they never chose — and for a
  new-seed slot there is no card to cross-check, so no row of §4.3 could catch
  it. The ms1 backup carries entropy only, never the passphrase, so a wrong
  binding is invisible in every engraved artifact.

### 4.1a The engrave tail — REQUIRED

§4.1 changes **assembly**. Steps 4–9 of `buildMultisigPolicyFlow`
(`gui/multisig_build.go:95-168`) are hard-wired to one seed and the locked
shared origin, and MUST be respecified with it. Round 0 omitted this; the two
scenarios below are why it is Critical rather than tidy.

1. **Leg derivation MUST use each held slot's DECLARED origin**, never
   `multisigSharedOrigin()`, once origins can diverge. Otherwise a build holding
   a slot at `m/48'/0'/1'/2'` engraves an mk1 derived at `m/48'/0'/0'/2'` — a key
   card asserting membership in a wallet **that does not contain its key**,
   stub-bound to the policy, on steel.
2. **One mk1 per held slot**, each at that slot's origin — or an explicit ruling
   that one leg suffices, with its reason. Silence is not an option.
3. **In full mode, every distinct master supplied MUST have its ms1 engraved**,
   or multi-master full mode MUST be refused with a named reason. Today
   `deriveMultisigLeg` emits one ms1 from the single `mnemonic`. A 3-of-4 where
   the operator holds three slots across masters A and B would engrave A only;
   losing B leaves two accessible legs against k=3 — **funds unspendable, from a
   backup the device labelled "Full (seed + keys)"**. That label is a claim
   about the steel.
4. **§4.5's byte comparison MUST extend to the mk1(s) and to ms1 presence.**
   Comparing only the md1 cannot see either scenario above.

### 4.2 Seed material lifetime — REQUIRED

`gui/multisig_build.go:67` currently types the self seed once and scrubs it with
a `defer`. Holding a seed across several slots invalidates that. Therefore:

**These MUSTs govern the flow's WORKING COPIES** — the `bip39.Mnemonic` buffers
and derivation intermediates — and not the systemwide session's stored record.
`gui/sysw_session.go:12-18` rules that lifetime explicitly: "LIFETIME IS THE
PROCESS… No flow clears it on exit, because a flow that did would silently
reintroduce the per-program KDF that 'once per session' exists to avoid…
Nothing here claims the records are scrubbed." A flow that scrubbed the session
record would violate SYSW§3.2.1. §5.3 is the operator-facing statement of that
residue. Round 0 wrote MUSTs that no compliant implementation could satisfy.

- A seed supplied in this flow MUST be scrubbed on **every** exit path,
  including error and Back, exactly as today.
- A seed MAY be retained **for the duration of the constructor only**, solely to
  derive further account indices for slots the operator has declared they hold.
- The working copies MUST NOT outlive the flow and MUST NOT be written
  anywhere. (Reachability from another program is a property of the session
  record, governed by SYSW§3.2.1 above, not of this flow.)
- A test MUST prove the scrub, and that test MUST be mutation-checked — a scrub
  assertion that cannot fail is the exact false-PASS this project has been burnt
  by (see `design/FOLLOWUPS.md` F-151 and the raster-ink precedent).

### 4.3 The seed↔key consistency gate — REQUIRED

**Operator requirement, 2026-08-13:** *"A payload should be able to deliver seeds
or keys or both as input into the multisig/miniscript wallet policy constructor.
If both seed and key are present, sh2 must verify key can be derived from seed
and if not, fail loudly."*

**Mechanism: derivation, never fingerprints.** Derive from the seed (plus
passphrase, if any) at the key's declared origin path and compare
chain code ‖ pubkey — i.e. `findUserSlot`'s existing comparison, run at
construction time.

Fingerprints MUST NOT be the mechanism, for two independent reasons, both
verified:

1. **A serialized xpub carries a PARENT fingerprint, not a master fingerprint.**
   BIP-32's 78 bytes are `version(4) ‖ depth(1) ‖ parent-fp(4) ‖ child(4) ‖
   chaincode(32) ‖ pubkey(33)`. These coincide only at depth 1; at the BIP-48
   account path `m/48'/0'/0'/2'` (depth 4) the parent fingerprint belongs to
   `m/48'/0'/0'` and identifies no master.
2. **The constellation does not retain even that.** `md/expand.go:62`
   `ExpandedKey.Xpub` is `[65]byte // 32B chain code ‖ 33B compressed pubkey`;
   version, depth, parent fingerprint and child number are dropped at encode.
   This is F-130, already filed.

The master fingerprint survives only as a **separate, optional** field
(`md/expand.go:60-61`, `md/encode_multisig.go:51-52`, and mk1's `Fingerprint`),
and the build flow **omits it by default** (`gui/multisig_build.go:334`,
`multisigFpChoices` → "No (omit)" at index 0). A gate keyed on fingerprints
would therefore be silent on most real payloads.

**The slot-assignment model — NORMATIVE, and the gate's trigger depends on it.**
Round 0 said the gate "fires on assignment" while no stage built a step in which
a slot could hold both a seed and a key, so a literal implementation would have
shipped a gate that never fires — the operator's requirement silently unmet,
with tests still green against synthetic assignments. So the model is ruled here
rather than left to §10:

Every slot `@0..@{n-1}` carries exactly one **source**, chosen by the operator
and shown on a review screen before assembly:

| source | meaning |
| --- | --- |
| `payloadKey(record)` | a cosigner mk1 from the payload |
| `derived(seedID, account)` | derived from a seed supplied in this flow |
| `both(seedID, account, record)` | the operator asserts the payload key at this slot **is** theirs, derived from that seed |

`both` is what makes a slot hold a seed *and* a key, and it is the only shape
that triggers the gate. It exists because it is the case the operator asked
about: a payload carrying their seed **and** their own key card.

**When the gate fires.** On a `both` slot. It is NOT inferred across unrelated
material: a payload carrying the operator's seed alongside other people's
cosigner cards is normal and MUST NOT fail.

**Outcomes — NORMATIVE:**

| condition | result |
| --- | --- |
| `both` slot, derivation matches | proceed |
| `both` slot, derivation does **not** match | **FAIL LOUDLY**, name the slot |
| a fingerprint IS present and contradicts the derivation | **FAIL LOUDLY** |
| any two FINAL slots carry an identical chain code ‖ pubkey | **FAIL LOUDLY** — §4.1's duplicate-key rule |
| one seed matches ≥2 slots at **distinct** origins | proceed, show the notice — legitimate multi-account |
| `payloadKey` slots that derive from no supplied seed | normal, no check |

The fourth row replaces round 0's "≥2 payload keys match the seed", which was
wrong twice over: it refused the legitimate multi-account wallet (§4.1), and it
required deriving against **unassigned** material, which R-2 forbids — the two
rules could not both be implemented.

"Fail loudly" means: a named error screen that states which slot, that the key
and seed disagree, and that nothing was engraved. It MUST NOT be a silent skip,
a warning the operator can page past, or a blank screen. A test MUST prove each
failing row actually fails, mutation-checked.

### 4.4 Nested segwit must be nameable — REQUIRED

`scriptName` MUST take the whole `md.Template`, not `Template.Root`, and MUST
distinguish:

| template | name |
| --- | --- |
| `Root == ScriptSh && InnerWsh` | `P2SH-P2WSH` |
| `Root == ScriptSh && InnerWpkh` | `P2SH-P2WPKH` |
| `Root == ScriptSh`, neither | `P2SH` |

All three call sites (§2.2 D-3) MUST be updated together; the restore document
is the one that matters most. A test MUST assert the three strings are pairwise
distinct — the current defect is precisely that two of them are equal.

### 4.5 The emulator journey gate — REQUIRED

**Operator requirement, 2026-08-13:** *"The emulator must be used to walk
journeys to make sure the code actually allows the user to do the thing we want
to do: make wallet policy descriptors."*

Every stage in §6 closes only when its capability has been **walked in
`cmd/emu`**, driving the real `gui` package, and the walk has produced an md1
the host toolchain accepts. A green unit suite is explicitly NOT sufficient:
§2.3 is the record of a green suite around a flow nobody could execute.

The walk MUST be automated (a script, not a remembered click sequence) so it
runs again on the next change, and the produced artifacts MUST be compared
against what the host produces for the same inputs — a byte comparison, not a
judgement. **From P3 the comparison covers the md1, every mk1, and ms1
presence** (§4.1a item 4); comparing the md1 alone cannot see a leg derived at
the wrong origin or a missing master's ms1.

**Named blind spot of the walk.** The on-device verify offer (step 10) reads
back through `multisigVerifyFlow` → `bundleGatherFlow`, which is NFC-only and
deliberately payload-refusing (`gui/derive_xpub.go:112-123`: a verify accepting
the payload source would compare the engrave source against itself and pass
unconditionally). In phase 1 it is therefore exercisable **only at P5, on
hardware**. Recorded rather than papered over, and owned by F-158's NFC plan —
a gate that hides its own blind spot is worse than no gate.

This is the `can-a-user-do-the-thing` rule, made a gate.

### 4.6 Tests must run safely and fast — REQUIRED

**Operator requirement, 2026-08-13:** *"tests must be run in a safe but fast
manner."*

**MEASURED baseline, 2026-08-13** — `go test ./gui/` takes **54.3 s**, and the
cost is concentrated in tests that wait on real wall-clock time:

```
TestPlateListMarksCutAfterACompletedEngraveAndNotAfterACancelledOne   7.75s
TestWipeGuardLifecycleAndArmed                                        5.06s
TestSpuriousTouchDoesNotHoldOffTheWipe                                3.78s
TestBip85DeriveFlow_ScrubsBothMnemonics                               3.55s
TestUnlockDerivesAtTheMaximumIterationCount                           3.38s
TestEngraveScreen                                                     3.19s
```

`testing/synctest` — synthetic time, already used in this very package at
`gui/multisig_build_flow_test.go:200` — makes time-waiting tests instant. So
most of that 54 s is recoverable without weakening a single assertion.

**FAST — normative tiers.** Each tier's cost must match how often it runs:

| tier | what | budget | runs |
| --- | --- | --- | --- |
| 1 | unit + flow tests, synthetic time | **`./gui/` under 10 s** | every edit |
| 2 | genuinely CPU-bound (real KDF work) | unbounded, behind `testing.Short()` | pre-commit |
| 3 | §4.5 emulator walk (wasm build + browser) | minutes | per stage gate |
| 4 | hardware (P5) | a flash cycle | once, at P5 |

- **New tests in this spec's stages MUST be tier 1** unless they are genuinely
  CPU-bound, in which case they MUST be skippable under `-short`.
- A test that waits on real time where `synctest` would do is a defect.
- Reducing the tier-1 budget is **opportunistic**, not a stage: convert a slow
  test when a stage already touches it. This spec does not own a 54 s → 10 s
  sweep, and P0 must not become one. Record the residue as a follow-up owned by
  `polish / v0.0.1`, per the phase-policy rule.

**SAFE — normative.** These are existing project rules, restated because this
spec's stages generate new test material:

- **Every seed and key in a test is public by construction** — a published
  BIP-39 vector, or derived deterministically from a published string. **Never
  put funds behind them.**
- **Test payloads stay confined to the `GOOS=js` emulator build.** A shipped
  SeedHammer II must never boot carrying a pre-known seed or someone else's
  payload. The existing confinement test is the pattern, including its
  `checked < 50` floor so a misrooted walk cannot pass vacuously.
- **No hardware in the routine loop.** Tiers 1–3 run with no machine attached;
  only P5 touches one, and flashing goes through `~/bin/sh/sh2-flash` rather
  than `picotool` by hand.
- **The §4.5 walk's frame receiver is pinned to one origin and accepts flat
  filenames only** — `design/journeys/shot_server.py` is the precedent and
  states why both restrictions are load-bearing.
- **A scrub or failure assertion MUST be mutation-checked.** Breaking the code
  under it must make it fail; a guard that cannot fail is worse than none,
  because it reads as coverage of exactly the thing it cannot see.

## 5. Sources for seeds and keys

### 5.1 The source seam — REQUIRED

**Operator directive, 2026-08-13:** *"We eventually want to allow payload and
NFC everywhere, including typed only."*

Source selection MUST therefore be built **once**, as one seam used by every
seed and key input in the constructor, rather than as a per-flow addition. The
seam offers the sources admitted for the program and the class (SYSW§3.3.2),
with typed entry always available.

**Phase 1 BUILDS on two sources: PAYLOAD and TYPED** (§3). It MUST be shaped so
NFC is a third source added later without re-opening its call sites — that is
the whole reason for building it once — but it MUST NOT ship a disabled NFC
branch, a stubbed source, or a menu entry that cannot be chosen. An inert
control teaches the operator that controls here may be inert, which is expensive
on a device whose other buttons cut steel.

**RULED — the existing SCAN row is left alone.** `syswSeedPicker`
(`gui/derive_xpub.go:140-142`) already offers Typed, the payload, **and Scanned
whenever the hardware reports a reader**. That row works today on real hardware
and is outside this spec's scope: masking it for the constructor alone would
re-open the per-flow divergence the shared seam exists to prevent. So phase 1
neither adds nor removes an NFC source; it **builds and tests against payload
and typed only**, and §4.5's walks exercise those two. "No inert controls"
stays true, because the SCAN row is not inert where it appears — it is simply
not what phase 1 relies on, and F-158 means it cannot be exercised by a test or
in the emulator regardless.

### 5.2 TYPED-ONLY is already retired — the text, not the mechanism, is the work

**Measured, §2.2 D-5.** All four sites call the multi-source `seedEntryFlow`.
There is no invariant left to retire and no per-site ruling framework to apply:
round 0 described four comments and mistook them for behaviour.

What remains is therefore smaller and different:

- **The four comments MUST be corrected or deleted** (P2), because a stale
  safety claim is worse than none — a future reader greps `TYPED-ONLY`, finds
  four hits, and concludes the payload cannot reach a seed entry.
- **The gate that SHOULD have accompanied that retirement is what is missing**,
  and §4.3 is it. State plainly: payload seeds already flow into the build path
  **without** any seed↔key cross-check. That is the exposure this spec closes.
- Whether the other three flows want §4.3's gate too is **not ruled here** —
  they have no key material to cross-check against, so the question is theirs.

### 5.3 The threat model that ruling must state

A machine that boots holding a spendable seed is a different object from one
that must be typed into: for an **unsealed** payload, physical possession of the
machine becomes possession of the seed. That is what the "A SECRET is stored
unencrypted in flash" screen exists to say. Therefore:

- **Sealed payloads are the RECOMMENDED carrier for seed classes.**
- The unsealed path remains permitted and keeps its existing loud warning.
- This spec does not weaken either.

### 5.4 `ClassCodex32Secret` is deferred, with cause

SYSW§3.3.2 admits `ClassCodex32Secret` to Engrave Multisig, but SYSW§3.3.2's own
reachability note records that §3.1's NORMATIVE seam signature returns
`bip39.Mnemonic`, "which cannot carry the `ClassCodex32Secret` this table admits
to all four seam programs… unservable until the seam gains a carrier type. That
is a design change with its own trade-offs, deliberately not smuggled into a
fold."

This spec honours that: **phase 1 delivers BIP-39 mnemonics from the payload.
ms1/codex32 seeds are out of scope** and need the carrier-type change as their
own stage.

## 6. Stages

Each stage closes at **0 Critical / 0 Important**, with a green full validation
suite AND its §4.5 emulator walk.

### P0 — the payload supplies the whole cosigner set

Nothing else can be verified until this lands, and the absence of it is why D-1
shipped. With NFC out of scope (§3.1) this is small, and it is a capability
worth having on its own rather than scaffolding.

Today `gui/multisig_build.go:54` takes **one** card from the payload and expects
the operator to scan the rest:

> `syswOffer(ctx, th, sysw.ClassMDMK, "First card from where?")` … "the operator
> keeps scanning the rest — the set is n-1 cards and a source that
> short-circuited the gather would cap it at whatever the payload held."

With no scanning, the payload must supply all of them.

1. `gui/sysw_session.go:114` `take` returns the **first** matching record and
   does not consume it, so it cannot yield a second card. Add a `takeAll`-style
   accessor for a class.
2. Feed **every** `ClassMDMK` record into `bundleGatherFlow`'s existing
   `offer()`, so dedup, chunk assembly and validation stay on exactly one path —
   which `gui/bundle_flow.go:100-103` already argues for: "A separate insertion
   path would be a second way for a card to become part of a bundle, and only
   one of them would have the checks."
3. **Filter to mk1 cards.** `ClassMDMK` covers md1 too, and md1 is admitted to
   this program for the *supply* path. `buildCosignerCards`
   (`gui/multisig_build.go:254-272`) refuses unless the yield is exactly n−1
   mk1s and zero md1 — so feeding every MDMK record would hard-fail any payload
   provisioned for Engrave Multisig at large (an md1 plus cards, or the 11-card
   constellation set of which this wallet needs a subset), with no on-device
   remedy. md1 records MUST be ignored here, not fatal.
4. **Rule the over-supply case:** more matching cards than open slots gets a
   selection step or a named refusal — not a fall-through.
5. **Slot order is payload record order, and MUST be shown as `@N` on the review
   screen.** Order is identity-bearing: `md/encode_multisig.go:13-21` is
   explicit that "Two callers supplying the same N keys in DIFFERENT orders mint
   DIFFERENT, both valid, md1 cards with DIFFERENT WalletPolicyId". For
   sortedmulti the *addresses* are order-independent, so this is identity and
   coordinator-matching rather than funds — but with scanning gone the order is
   fixed on the host, and the review screen offers no reorder.
6. The gather screen becomes a **review of what the payload supplied**, not a
   "Scan a card" prompt. Ruled here, not deferred (round 0 left it in §10 while
   a stage depended on it).

**Gate — the flow is DRIVABLE.** An automated test and an emulator walk carry
the flow from the template picker to **either** a completed engrave **or** a
captured reproduction of D-1. Round 0 required a completed engrave here, which
cannot hold: if D-1 lives on this path P0 can never close, and if it does not,
P1's "the reproduction test fails on the unfixed code" is unsatisfiable. The
completed-engrave gate belongs to P1.

### P1 — the dead end

Reproduce D-1 (P0 made it reachable), fix it, and assert the fix **rasters** —
F-151's lesson is that a text assertion cannot see a body that fails to draw.
Fix D-4 (the "Engrave Bundle" title) in the same stage.

**Gate:** the reproduction test fails on the unfixed code — demonstrated, not
assumed — **and** the flow now reaches a completed engrave, by test and by
emulator walk. If P0 found no D-1 on the payload path, this stage records that
as its result and keeps the completed-engrave gate; D-1 then belongs to a
source or shape P0 did not exercise, and that MUST be named rather than closed.

### P2 — nested segwit is nameable, and the stale comments go

§4.4. Plus the four `TYPED-ONLY` comments (§2.2 D-5) corrected or deleted —
grouped here because both are "the code is right and the text lies", and a
stale safety claim is the defect class this project has been burnt by three
times in one day. Smallest stage; independent of the others.

### P3 — multi-slot self, divergent origins, and the engrave tail

§4.1, §4.1a and §4.2. `SelfSlot int` becomes a set; `cosignerFromCard` stops
discarding origins; `OriginDivergent` is used when origins differ; the
duplicate-key check runs over the final slot set; per-seed passphrases.

**And the tail** (§4.1a): leg derivation at each held slot's declared origin,
the mk1 cardinality ruling, and full-mode ms1 coverage for every distinct
master. Assembly without the tail engraves a key card for a wallet that does not
contain its key.

**Gate:** §4.5's byte comparison extended to the mk1(s) and ms1 presence — the
md1 alone cannot see either failure.

### P4 — the consistency gate and the assignment model

§4.3 and §5. The source seam and the slot-assignment model; the seed↔key
consistency gate; R-1's ratification.

**Note the ordering constraint:** payload seeds already reach this flow (§2.2
D-5), so the exposure §4.3 closes is live from today until P4 lands. If that is
judged unacceptable, the gate moves ahead of P3 — it depends only on the
assignment model, not on multi-slot support. Flagged for the operator (§10 Q7).

### P5 — hardware gate

Engrave and restore **both** a `wsh` and an `sh(wsh)` multisig on the machine,
and verify each against an external coordinator. **At least one build MUST be
divergent-origin, multi-slot and multi-master** — a shared-origin single-seed
P5 would pass green around every §4.1a failure. This confirms software that is
already proven; it is not the first place the flow is executed.

## 7. Rulings this spec makes

**R-1 — `I-SCRUB` is RATIFIED as already retired, and scoped.** It does not
retire anything: `gui/multisig_build.go:68` already calls the multi-source
`seedEntryFlow` (§2.2 D-5), so a payload seed reaches the constructor today. The
ruling is that this behaviour is **correct and intended**, and is now bounded by
§4.2's lifetime rules, §4.3's consistency gate — which is what was missing — and
§5.3's threat model. `I-3`, `I-7` and `D12` are equally stale as comments;
correcting their text is P2's, and ruling their behaviour belongs to whoever
owns those flows, not to this spec.

**R-2 — the consistency gate fires on assignment, not inference.** §4.3. A
payload carrying a seed and unrelated cosigner cards is a normal payload.

**R-3 — for a divergent policy, the CARD's origin wins over the flow's shared
origin.** A cosigner card states where its key actually lives; the flow's shared
origin is a default for keys that do not say. This is the same question F-129
records as unpinned on the host side, and it is ruled here **for the device
only** — pinning the host precedence remains F-129's, and a restore test should
settle both together.

**R-4 — a green unit suite does not close a stage.** §4.5.

## 8. What is already machine-verified (for the R0 reviewer)

State this in the review brief so reviewer budget goes where tools cannot reach.
Verified on 2026-08-13 by running or by reading the cited line:

- Every file:line citation in §2 resolves at `a10d007`.
- D-3's four output strings are **run output**, not description.
- `md.EncodeMultisig`'s `OriginDivergent` arm exists and is unused by `gui`.
- `writeNode`'s seven body arms exist as listed.
- The taproot wire tags exist and the decoder handles `tagTr`.
- `ExpandedKey.Xpub` is 65 bytes, chain code ‖ pubkey only.
- `testPlatform.NFCReader()` returns nil; `TestBuildFlow_GatherBeforeSeed` exits
  at the gather.
- The emulator walk reaching the gather, and the gather never ingesting a card,
  were performed in a browser.
- §4.6's 54.3 s and its six slowest tests are `go test` output, not estimates.

- A payload carries mk1 cosigner cards: `me sysw pack` accepted a two-chunk card
  and `me sysw show` reported both records as `md1/mk1 — confirmed`. Run output.
- `testPlatform.SyswReader()` returns a settable field, unlike `NFCReader()`.

**Not verified, and flagged as such:** D-1 itself (unreproduced — reproducing it
is P1's first task, now that P0 makes the flow reachable); whether any code path
outside `gui` consumes `scriptName`. The precise cause of the emulator's nil NFC
reader is **deliberately not investigated further here** — it belongs to the
later NFC plan (F-158), and phase 1 does not depend on it.

**No Rust build gate applies.** This spec's executable content is Go, and
`scripts/plan-build-gate.sh` extracts ```rust blocks only. The Go claims here
were checked by `go test` and by reading the source; a reviewer should still run
an execution pass over §2's citations.

## 9. Phase 2 — arbitrary `wsh` / `tr` miniscript, and the constraint that governs it

Deferred, and recorded now so phase 1 does not accidentally foreclose it.

**The constraint.** There is **no miniscript type checker in the Go port.**
`decodePayloadValidated` (`md/md.go:1133`) applies five validators —
`validatePlaceholderUsage`, `validateMultipathConsistency`,
`validateTapScriptTree`, `validateExplicitOriginRequired`, `validateXpubBytes` —
all structural. `errOperatorContext` is a root-tag allow-list
(`md/md.go:848-852`). `rust-miniscript` is deliberately omitted for TinyGo.

So the device **can encode, chunk, checksum and engrave a tree that is not
spendable, and nothing on it can tell.** `and_v(v:older(65535),multi(2,…))` is
valid; drop the `v:` and it is not; the codec emits both identically. That is
F-137's shape made worse — not encodable-but-undecodable, but encodable,
decodable, engraved on steel, and unspendable.

**The operator's chosen direction, 2026-08-13:** a catalogue of **parameterised
recipes** — a shape plus a **proven parameter domain**, type-checked once on the
host by `rust-miniscript`, landed in Rust first with test vectors, then ported.
Validity is usually a property of the shape (`and_v(v:older(N),multi(k,…))`
holds for all `N`, `k`), but not always — a 15-of-15 `wsh` exceeds the 520-byte
redeemScript limit — so bounds are part of the recipe. The device's bounded
pickers are the enforcement, which is already this codebase's idiom
(`n ∈ 2..5`, `k ∈ 1..n`, marked LOCKED).

Outside the catalogue: the operator authors, and a **verify-before-funding**
surface — on screen and in the restore document — states that the device could
not check. The operator accepted this trade-off knowing F-131/F-132 are two
cases where a checklist told an operator something false; the surface must
therefore be hard to ignore rather than a line in a list.

**Eventually:** port the type system, at which point free-form authoring becomes
safe and the verify-before-funding surface can be retired. Not now.

**Note for whoever opens phase 2:** the seam is cheaper than it looks.
`multiSigTree` is already recipe #1, `split()` and `writeNode()` already do the
hard part, and `node`/`descriptor` can stay unexported if recipes live inside
`md` — which also keeps `I-VERBATIM` (one md1-bytes producer) intact.

## 10. Open questions

Round 0's Q1 is void (its stage was NFC-harness work, now out of scope). Q3 and
Q5 were **ruled** rather than left open, because §4.1/§4.3/P0 all depended on
them — an open question a stage depends on is a specification hole, not a
question. Q3 is now §4.3's assignment model; Q5 is P0 item 6.

1. Does R-3 (card origin wins) conflict with any restore path that assumes the
   shared origin, and is a restore test needed inside phase 1 rather than after?
   **The R0 reviewer verified R-3 sound and endorsed the restore test as the
   natural gate for §4.1a's fixes** — so this is now a scheduling question, not
   a correctness one.
2. Does the §4.5 walk belong in CI, given it needs a wasm build and a browser?
3. §3.1 notes phase 1 requires a host to write the payload, so an operator
   cannot build a multisig from the device alone. Is that acceptable for the
   phase-1 milestone, or does it make the later NFC plan a release blocker
   rather than a follow-on?
4. **New, and the sharpest one.** §2.2 D-5 establishes that payload seeds reach
   the constructor **today**, with no seed↔key cross-check. §4.3 closes that,
   but only at P4. Should the gate move ahead of P3 — it depends only on the
   assignment model — or is the exposure acceptable for the duration, given the
   feature is behind an EXPERIMENTAL warning and unreleased?
