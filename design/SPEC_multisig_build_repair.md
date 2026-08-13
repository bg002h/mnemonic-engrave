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
   The operator, air-gapped, builds a k-of-n sorted-multisig wallet policy on
   the device, holding **one or more** of its keys, with the other cosigner keys
   arriving by NFC or payload. The result is an md1 the operator can engrave and
   later restore. Nothing exotic: no timelocks, no hashlocks, no composition.
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
  `OriginPath` and compares chain code ‖ pubkey, reporting `reused []int` when
  one key occupies several slots. It uses no fingerprints (§4.3 explains why
  that is the correct choice, not an oversight).
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
- **A seed reused across slots at the SAME origin is a defect, not a
  configuration.** It produces one key in two slots; a 2-of-3 built that way is
  a 1-of-2. `findUserSlot` already detects this (`reused []int`) and the
  constructor MUST refuse it loudly.

### 4.2 Seed material lifetime — REQUIRED

`gui/multisig_build.go:67` currently types the self seed once and scrubs it with
a `defer`. Holding a seed across several slots invalidates that. Therefore:

- A seed supplied in this flow MUST be scrubbed on **every** exit path,
  including error and Back, exactly as today.
- A seed MAY be retained **for the duration of the constructor only**, solely to
  derive further account indices for slots the operator has declared they hold.
- It MUST NOT outlive the flow, MUST NOT be written anywhere, and MUST NOT be
  reachable from any other program.
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

**When the gate fires.** On **assignment** — wherever the operator has placed
both a seed and a key into the same slot. It is NOT inferred across unrelated
material: a payload carrying the operator's seed alongside other people's
cosigner cards is normal and MUST NOT fail.

**Outcomes — NORMATIVE:**

| condition | result |
| --- | --- |
| assigned slot, derivation matches | proceed |
| assigned slot, derivation does **not** match | **FAIL LOUDLY**, name the slot |
| assigned slot, ≥2 payload keys match the seed | **FAIL LOUDLY** — key reuse |
| a fingerprint IS present and contradicts the derivation | **FAIL LOUDLY** |
| unassigned keys that do not derive from any seed | normal, no check |

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
runs again on the next change, and the produced md1 MUST be compared against
what the host produces for the same inputs — a byte comparison, not a
judgement.

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

**In phase 1 the seam has exactly two sources: PAYLOAD and TYPED** (§3). It MUST
be shaped so NFC is a third source added later without re-opening its call
sites — that is the whole reason for building it once — but it MUST NOT ship a
disabled NFC branch, a stubbed source, or a menu entry that cannot be chosen. An
inert control teaches the operator that controls here may be inert, which is
expensive on a device whose other buttons cut steel (the same argument
`gui/multisig_build.go` already makes for the pager it made conditional).

### 5.2 Retiring TYPED-ONLY — REQUIRED, by explicit ruling

`TYPED-ONLY` is a named invariant in four places: `gui/bip85.go:264` (`I-3`),
`gui/singlesig.go:18` (`D12`), `gui/multisig.go:24` (`I-7`), and
`gui/multisig_build.go:67` (`I-SCRUB`). It MUST be retired **by ruling per
site**, never eroded flow by flow. This spec rules only `I-SCRUB` (§7 R-1); the
other three keep their invariant until their own ruling.

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

This spec honours that: **phase 1 delivers BIP-39 mnemonics from payload and
NFC. ms1/codex32 seeds are out of scope** and need the carrier-type change as
their own stage.

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
3. Decide what the gather screen becomes when nothing can be scanned — a review
   of what the payload supplied, not a "Scan a card" prompt. See §10 Q5.

**Gate:** an automated test drives `buildMultisigPolicyFlow` from the template
picker to a completed engrave using **only** a payload, and an emulator walk
does the same in a browser (§4.5).

### P1 — the dead end

Reproduce D-1 (now possible), fix it, and assert the fix **rasters** — F-151's
lesson is that a text assertion cannot see a body that fails to draw. Fix D-4
(the "Engrave Bundle" title) in the same stage. **Gate:** the reproduction test
fails on the unfixed code — demonstrated, not assumed.

### P2 — nested segwit is nameable

§4.4. Smallest stage; independent of the others.

### P3 — multi-slot self and divergent origins

§4.1 and §4.2. `SelfSlot int` becomes a set; `cosignerFromCard` stops discarding
origins; `OriginDivergent` is used when origins differ. Key reuse refused.

### P4 — the payload as a SEED source, and the consistency gate

§4.3 and §5. The source seam; seeds from the payload (P0 already did keys); the
seed↔key consistency gate; the `I-SCRUB` ruling.

### P5 — hardware gate

Engrave and restore **both** a `wsh` and an `sh(wsh)` multisig on the machine,
and verify each against an external coordinator. This confirms software that is
already proven; it is not the first place the flow is executed.

## 7. Rulings this spec makes

**R-1 — `I-SCRUB` is retired for the Build-policy constructor only.** The self
seed MAY come from a payload or NFC, subject to §4.2's lifetime rules, §4.3's
consistency gate and §5.3's threat model. `I-3`, `I-7` and `D12` are untouched.

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

## 10. Open questions for R0

1. Is P0's isolation likely to find a third cause beyond the two named in §6?
2. Does R-3 (card origin wins) conflict with any restore path that assumes the
   shared origin, and is a restore test needed inside phase 1 rather than after?
3. §4.1 allows "another account index of a seed already supplied". Should the
   account index be operator-chosen or auto-assigned ascending? Auto is simpler
   and harder to get wrong; operator-chosen matches wallets that already exist.
4. Does the §4.5 walk belong in CI, given it needs a wasm build and a browser?
5. With nothing scannable in phase 1, what does the cosigner-gather screen
   become — a review of what the payload supplied, or does it disappear into the
   parameter pickers? It currently says "Scan a card, or Done" and is titled
   "Engrave Bundle" (F-159). Whatever replaces it must not become a screen that
   NFC has to fight its way back into later.
6. §3.1 notes phase 1 requires a host to write the payload, so an operator
   cannot build a multisig from the device alone. Is that acceptable for the
   phase-1 milestone, or does it make the later NFC plan a release blocker
   rather than a follow-on?
