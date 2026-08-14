# Plan review — lens: does the plan implement everything the SPEC requires?

Reviewer: spec-coverage lens, 2026-08-13. Artifacts:
`design/IMPLEMENTATION_PLAN_multisig_build_repair.md` at working-tree HEAD
(`b474180`), against `design/SPEC_multisig_build_repair.md` (GREEN 0C/0I, not
re-reviewed). Fork source read at `a10d007`. Method: enumerate spec-side first
(every REQUIRED / MUST / NORMATIVE / RULED item in §1, §3, §4.1–§4.6, §5.1–§5.4,
§6 P0–P5, §7 R-1..R-4), then locate the discharging stage — so absence is
visible rather than inferred.

## Verdict

**NOT clean — 1 Critical, 5 Important, 8 Minor.** Coverage of the *behavioural*
spec is genuinely good: every requirement in §4.1, §4.1a, §4.3's outcome table,
§4.4 and §6 P0–P3/P5 lands on a named test. The hole is one whole normative
subsection (§4.6's SAFE block, three of its five bullets ABSENT), one whole
REQUIRED section (§5.1's source seam, ABSENT and actively regressed by S1), and
— the Critical — **§4.5's emulator walk has nothing to walk with**: no stage
creates the emulator-side payload that every gate from S1 onward depends on.

---

## Coverage matrix

Legend: **C** covered · **P** partial (named, not discharged) · **A** absent.

### §1 / §3 — phase boundary and scope

| # | spec requirement | norm? | stage | how | v |
| --- | --- | --- | --- | --- | --- |
| 1-1 | Phase 1 = k-of-n sortedmulti, ≥1 held key, cosigners by payload, NFC out | NORMATIVE | plan §0, §1 table, §4 | scope statement + stage table | **C** |
| 1-2 | No phase-1 work justified by phase 2 | NORMATIVE | plan §4 | exclusion list | **C** |
| 3-1 | Out: taproot, miniscript operators, recipe seam, type checker | scope | plan §4 | exclusion list | **C** |
| 3-2 | Out: `ClassCodex32Secret` as payload seed | scope | plan §4 | exclusion list, cites §5.4 | **C** |
| 3-3 | Out: Sealed Payload changes, md1 wire-format changes | scope | — | nothing in the plan touches either (grep) | **C** |

### §4.1 — multi-slot self keys (REQUIRED)

| # | spec requirement | stage | how | v |
| --- | --- | --- | --- | --- |
| 4.1-a | Operator MUST declare ≥1 held slots; source = new seed OR another account index of a seed already supplied | S5 t1 (Trace B), S4 `derived(seedID,account)` | `TestMultiSlotSelfAssembles` | **C** |
| 4.1-b | `OriginDivergent` MUST be used when origins differ; `OriginShared` when equal | S5 impl + t1 | impl bullet + assembly test | **C** |
| 4.1-c | `cosignerFromCard` MUST stop discarding the card's origin | S5 impl + t2 | `TestCosignerCardOriginIsHonoured` | **C** |
| 4.1-d | Depth-0 cosigner card MUST be refused by a **named screen** | S5 t6 | `TestDepthZeroCosignerCardIsNamedRefusal` | **C** |
| 4.1-e | Duplicate-key refusal over the **final slot set**, 65-byte chaincode‖pubkey | S4 t4 (+ S2 t4 interim window) | `TestGateRefusesDuplicateKeyAcrossFinalSlots` | **C** |
| 4.1-f | Same seed at ≥2 slots under distinct origins is legitimate, MUST proceed + notice | S4 t5 | `TestGateAcceptsSameSeedAtDistinctOrigins` | **C** |
| **4.1-g** | **Passphrase prompt is PER SEED, asked at that seed's entry; `(seed, passphrase)` is the derivation unit everywhere** | S4 impl bullet only | *no test, no gate* | **P** → **I3** |

### §4.1a — the engrave tail (REQUIRED)

| # | spec requirement | stage | how | v |
| --- | --- | --- | --- | --- |
| 4.1a-1 | Leg derivation MUST use each held slot's DECLARED origin | S5 t3 | `TestLegDerivedAtHeldSlotOrigin`, asserts the mk1 key is one the descriptor contains | **C** |
| 4.1a-2 | One mk1 per held slot, at that slot's origin (RULED) | S5 t4 | `TestOneMk1PerHeldSlot` | **C** |
| 4.1a-3 | Full mode: every distinct master's ms1 engraved, or named refusal | S5 t5 | `TestFullModeEngravesMs1ForEveryMaster`, engrave-both arm chosen, mutation-checked | **C** |
| 4.1a-4 | §4.5 byte comparison MUST extend to mk1(s) and ms1 presence | S5 gate | strengthened to byte-for-byte on every ms1 | **C** |

### §4.2 — seed material lifetime (REQUIRED)

| # | spec requirement | stage | how | v |
| --- | --- | --- | --- | --- |
| 4.2-1 | Scrubbed on **every** exit path incl. error and Back | S4 t8 | `TestBuildFlowScrubsEverySeedOnEveryExit`, enumerates exit classes | **C** |
| 4.2-2 | MAY be retained for the constructor's duration only | S4 impl | `seedID`-keyed registry | **C** |
| 4.2-3 | Working copies MUST NOT outlive the flow | S4 t8 | zeroed on each exit class | **C** |
| 4.2-4 | …and MUST NOT be **written anywhere** | — | no assertion; t8 tests zeroing, not non-persistence | **P** → M1 |
| 4.2-5 | A test MUST prove the scrub, mutation-checked | S4 t8 | delete one scrub site → red | **C** |

### §4.3 — the seed↔key consistency gate (REQUIRED)

| # | spec requirement | stage | how | v |
| --- | --- | --- | --- | --- |
| 4.3-1 | Mechanism is derivation, reusing `findUserSlot`'s compare, at construction time | S4 impl | explicit "rather than a second implementation" | **C** |
| 4.3-2 | Fingerprints MUST NOT be the mechanism | S4 impl + t6 | fp only as a contradiction check | **C** |
| 4.3-3 | Slot-assignment model: exactly one of `payloadKey` / `derived` / `both` per slot | S4 impl | named verbatim | **C** |
| 4.3-4 | …**shown on a review screen before assembly** | S4 impl bullet only | no test | **P** → M2 |
| 4.3-5 | `both` slot: card's origin **and key** authoritative; `account` is bookkeeping, input to nothing | S4 t9 | `TestGateDerivesAtTheCardsOwnOrigin`, mutation = derive at `multisigSharedOrigin()` | **C** |
| 4.3-6 | `derived`'s `account` occupies the BIP-48 account component | S4 impl + S5 t3 | binding stated; exercised at `m/48'/0'/1'/2'` | **C** |
| 4.3-7 | Fires on `both` only; NOT inferred across unrelated material (R-2) | S4 t3 | `TestGateIgnoresUnassignedCosigners` | **C** |
| 4.3-8 | Outcome: `both` matches → proceed | S4 t2 | `TestGateAcceptsBothSlotMatch` | **C** |
| 4.3-9 | Outcome: `both` mismatch → FAIL LOUDLY, name the slot, nothing engraved | S4 t1 | `TestGateFiresOnBothSlotMismatch` | **C** |
| 4.3-10 | Outcome: fingerprint present and contradicting → FAIL LOUDLY | S4 t6 | `TestGateRefusesContradictingFingerprint` | **C** |
| 4.3-11 | Outcome: two final slots identical → FAIL LOUDLY | S4 t4 | see 4.1-e | **C** |
| 4.3-12 | Outcome: one seed at ≥2 distinct origins → proceed + notice | S4 t5 | see 4.1-f | **C** |
| 4.3-13 | Outcome: `payloadKey` slots deriving from no supplied seed → no check | S4 t3 | see 4.3-7 | **C** |
| 4.3-14 | Each failing row proven to fail, mutation-checked | S4 test-list preamble | stated as the stage's condition | **C** |

### §4.4 — nested segwit nameable (REQUIRED)

| # | spec requirement | stage | how | v |
| --- | --- | --- | --- | --- |
| 4.4-1 | `scriptName` takes the whole `md.Template`; three distinct names | S3 impl | signature change spelled out | **C** |
| 4.4-2 | All three call sites updated **together** | S3 impl | `md1_inspect.go:58`, `multisig_restore.go:51`, `bundle.go:315` — **verified: exactly 3 call sites at `a10d007`** | **C** |
| 4.4-3 | A test MUST assert the three strings pairwise distinct | S3 t1 | `TestScriptNameDistinguishesNestedFromLegacy` | **C** |
| 4.4-4 | The restore document is the surface that matters most | S3 t2 | `TestRestoreDocNamesNestedSegwit` | **C** |

### §4.5 — the emulator journey gate (REQUIRED)

| # | spec requirement | stage | how | v |
| --- | --- | --- | --- | --- |
| **4.5-1** | **Every §6 stage closes only when walked in `cmd/emu`, driving real `gui`, producing an md1 the host accepts** | S1/S2/S3/S4/S5 gates all name a walk | **the walk is unexecutable: `cmd/emu`'s only payload carries no `ClassMDMK` record and no stage creates one** | **P** → **C1 (Critical)** |
| 4.5-2 | The walk MUST be automated (a script) | S0 d1 | pinned-oracle harness / walk script | **C** |
| 4.5-3 | Produced artifacts byte-compared to host output for the same inputs | plan §1a + every stage gate | per-artifact comparison plane ruled | **C** |
| 4.5-4 | From P3: md1 + every mk1 + ms1 presence | S5 gate | strengthened to byte equality | **C** |
| 4.5-5 | Named blind spot: verify readback only at P5 | plan §4 | reproduced verbatim | **C** |

### §4.6 — FAST (REQUIRED)

| # | spec requirement | stage | how | v |
| --- | --- | --- | --- | --- |
| 4.6-F1 | New tests MUST be tier 1 (`./gui/` under 10 s, synthetic time) | plan §3 | "Tests are tier 1 unless named otherwise — synctest, no real sleeps" | **C** |
| 4.6-F2 | …unless genuinely CPU-bound, in which case **skippable under `-short`** | S0 gate, S6 | marked "not tier 1"; `-short` / `testing.Short()` never named, and S0's shell-out harness is I/O-bound, not the CPU-bound case tier 2 is defined as | **P** → M5 |
| 4.6-F3 | A test waiting on real time where synctest would do is a defect | plan §3 | stated | **C** |
| 4.6-F4 | Tier-1 sweep is opportunistic, not a stage; **record the residue as a follow-up owned by `polish / v0.0.1`** | plan §4 claims "filed, not owned" | **no such follow-up exists — `FOLLOWUPS.md` ends at F-159** | **P** → M4 |

### §4.6 — SAFE (REQUIRED)

| # | spec requirement | stage | how | v |
| --- | --- | --- | --- | --- |
| **4.6-S1** | **Every seed and key in a test is public by construction; never put funds behind them** | — | not stated anywhere in the plan, which mints new fixtures (Trace B masters A and B, a `both`-slot card) | **A** → M3 |
| **4.6-S2** | **Test payloads stay confined to the `GOOS=js` build; the existing confinement test is the pattern, incl. its `checked < 50` floor** | — | zero mentions of confinement / `GOOS=js` / `//go:build js` in the plan | **A** → **I1** |
| 4.6-S3 | No hardware in the routine loop; flashing via `~/bin/sh/sh2-flash`, never `picotool` | S6 | stated verbatim | **C** |
| **4.6-S4** | **The walk's frame receiver is pinned to one origin and accepts flat filenames only (`shot_server.py` precedent)** | — | S0 d1 builds a new capture harness and says nothing about either restriction | **A** → **I4** |
| 4.6-S5 | Scrub / failure assertions MUST be mutation-checked | S4 t1–t9, S5 t5 | mutations named per test | **C** |

### §5 — sources

| # | spec requirement | stage | how | v |
| --- | --- | --- | --- | --- |
| **5.1-1** | **Source selection MUST be built ONCE, as one seam used by every seed AND KEY input in the constructor** | — | absent; S1 *removes* the existing key-source picker (`syswOffer(…ClassMDMK…)`) | **A** → **I2** |
| **5.1-2** | **Shaped so NFC is a third source added later without re-opening its call sites** | — | absent | **A** → **I2** |
| 5.1-3 | MUST NOT ship a disabled NFC branch, a stubbed source, or an unchoosable entry | plan §4 | "neither adds nor removes the existing SCAN row" | **C** |
| 5.1-4 | RULED: the existing SCAN row is left alone | plan §4 | cites §5.1 | **C** |
| 5.2-1 | The stale `TYPED-ONLY` comments MUST be corrected or deleted | S3 impl + gate | **verified: 9 occurrences / 4 files at `a10d007`, matching the plan exactly**; gate is `grep … returns 0` | **C** |
| 5.2-2 | The missing gate that should have accompanied the retirement is §4.3 | S4 | S4's opening line | **C** |
| 5.2-3 | The other three flows' gate question is not ruled here | — | plan rules nothing about them | **C** |
| 5.3 | Sealed RECOMMENDED for seed classes; unsealed permitted with its warning; neither weakened | — | plan weakens neither (no-op requirement) | **C** |
| 5.4 | Phase 1 delivers BIP-39 from the payload; ms1/codex32 out | plan §4 | exclusion, cites the carrier-type change | **C** |

### §6 — stage intentions

| # | spec requirement | stage | how | v |
| --- | --- | --- | --- | --- |
| P0-1 | `takeAll` accessor inheriting `!loaded \|\| !compared`, mutation-checked | S1 t1, t2 | both named | **C** |
| P0-2 | Feed every `ClassMDMK` through `bundleGatherFlow`'s `offer()`; no second insertion path | S1 impl | cites `bundle_flow.go:100-103` | **C** |
| P0-3 | Filter to mk1; md1 ignored, **not fatal** | S1 t4 + impl | `TestBuildIgnoresMd1RecordsInThePayload` | **C** |
| P0-4 | Over-supply gets a selection step or a named refusal | S1 t6 | `TestBuildRefusesMoreCardsThanOpenSlots` | **C** |
| P0-5 | Slot order = payload record order, shown as `@N` | S1 t5 | asserts both | **C** |
| P0-6 | Gather screen becomes a review of what the payload supplied | S1 impl | stated | **C** |
| P0-G | Gate: drivable — completed engrave **or** captured D-1 reproduction | S1 gate | disjunction preserved | **C** |
| P1-1 | Reproduce D-1, fix it, assert the fix **rasters** | S2 t1, t5 | regression test + raster floor, calibrated both ways | **C** |
| P1-2 | Fix D-4 (the "Engrave Bundle" title) in the same stage | S2 t2 | `TestBuildGatherIsNotTitledEngraveBundle` | **C** |
| P1-3 | Refuse **or** warn on a foreign-origin card while the D-2 window is open | S2 t3 | plan picks REFUSE (a permitted arm) | **C** |
| P1-G | Repro fails on unfixed code; flow reaches a completed engrave | S2 gate | plus md1 production-equality | **C** |
| P2 | §4.4 + the stale comments | S3 | see above | **C** |
| P3 | §4.1 + §4.1a + §4.2 | S5 (+ §4.2's test moved to S4 — a permitted reorder) | see above | **C** |
| P3-G | Byte comparison extended to mk1(s) and ms1 | S5 gate | strengthened | **C** |
| **P4** | **"§4.3 **and §5**. The source seam and the slot-assignment model; the gate; **R-1's ratification**"** | S4 | slot model + gate covered; **source seam dropped; R-1 never cited** | **P** → **I2**, M6 |
| P5-1 | Engrave+restore a `wsh` and an `sh(wsh)`, verified at an external coordinator | S6 1, 2 | stated | **C** |
| P5-2 | At least one build divergent-origin, multi-slot, multi-master | S6 3 | stated verbatim | **C** |

### §7 — rulings

| # | ruling | stage | v |
| --- | --- | --- | --- |
| R-1 | `I-SCRUB` ratified as already retired, and scoped | substance in S3 (comments) + S4 (bounds); **never cited, and P4 names its ratification as a deliverable** | **P** → M6 |
| R-2 | Gate fires on assignment, not inference | S4 t3 | **C** |
| R-3 | Card's origin wins for a divergent policy | S5 t2 (cited by name) | **C** |
| R-4 | A green unit suite does not close a stage | plan §3, every stage gate | **C** (but see M7: S0's gate has no walk, against plan §3's own universal) |

---

## C1 — Critical: §4.5's emulator walk has nothing to walk with

**The requirement.** §4.5 is REQUIRED and binds *every* stage: "Every stage in §6
closes only when its capability has been **walked in `cmd/emu`**, driving the
real `gui` package, and the walk has produced an md1 the host toolchain
accepts." The plan honours this in text — S1, S2, S3, S4 and S5 each name a walk
in their gate line.

**What is absent.** The walks need cosigner mk1 cards on a payload. Trace A
needs two; Trace B needs cosigner D's; S4's `both`-slot walk needs the
operator's *own* mk1 alongside a seed the payload also carries. **`cmd/emu` can
supply none of them, and no stage in the plan creates the means to.** Measured
at `a10d007`:

- `cmd/emu/platform.go:257-259` — `SyswReader()` returns
  `embeddedSyswReader{data: []byte(syswTestPayload)}`. One hardwired blob, no
  parameter.
- `cmd/emu/sysw_test_payload.go` documents that blob's contents exactly: three
  records — `ClassMnemonic`, `ClassPassphrase`, `ClassFreeText`. **No
  `ClassMDMK`.** `cmd/emu/sysw_test_payload_host_test.go:71-101`
  (`TestSyswTestPayloadCarriesThreeClasses`) pins those three classes so the set
  cannot silently shrink — but nothing adds a fourth.
- There is no injection API for the payload. `cmd/emu/nfc_js.go:27` exposes
  `window.shNFC` for tags; `grep -rn "shSysw" cmd/emu/` returns **nothing**. The
  asymmetry §3.1 leans on (`testPlatform.SyswReader()` is a settable field) is a
  *host-test* asymmetry only; the browser side is a constant.
- `grep -n -i 'cmd/emu' design/IMPLEMENTATION_PLAN_multisig_build_repair.md`
  returns **zero**. No stage names the file, the blob, or a new reader.

So S1's gate — the first gate after S0 — is unreachable as written, and so is
every gate after it.

**Why Critical rather than Important.** The implementer arriving at S1's gate
has exactly two exits, and both are the failure this spec exists to prevent.
Either they waive the walk (recreating §2.3 verbatim: a green unit suite around
a flow nobody can execute, which is *how D-1 reached hardware and steel*), or
they improvise a payload blob under time pressure — into a tree whose
confinement guard is name-keyed and will not see it (I1), in a plan whose last
stage flashes real firmware. §4.5 was made a gate because the alternative put a
defect on a plate; a gate that cannot be run will be waived, and the waiver will
not be recorded.

**Bounded fix.** Give S1 a named deliverable — it is the stage that first needs
it — and reference it from the later gates:

> **A multisig emulator payload.** `cmd/emu/sysw_multisig_payload.{go,bin}`,
> `//go:build js`, packed by `me sysw pack` with recorded provenance in the file
> header (the `sysw_test_payload.go` header is the pattern): the BIP-39
> all-zeros mnemonic, an mk1 for master A at `m/48'/0'/0'/2'`, and cosigner mk1s
> for B/C/D — enough for Trace A (2 cards), Trace B (`@3`) and S4's `both` slot.
> Public by construction (§4.6 SAFE). Digest pinned by a host test, as
> `TestSyswTestPayloadMatchesItsDigest` does. Confined per I1.

If the intent was instead a `window.shSysw` injection API, say so — that is a
different deliverable with a different confinement argument, and the plan must
pick one.

---

## I1 — Important: §4.6 SAFE's confinement requirement is ABSENT, and the existing guard will not cover the new material

**The requirement**, normative in §4.6 SAFE: "Test payloads stay confined to the
`GOOS=js` emulator build. A shipped SeedHammer II must never boot carrying a
pre-known seed or someone else's payload. The existing confinement test is the
pattern, including its `checked < 50` floor so a misrooted walk cannot pass
vacuously."

**Absent.** `grep -in 'confine\|GOOS=js\|go:build js'` over the plan returns
zero.

**Why the existing test does not silently cover it.** The guard is
**name-keyed**, not structural —
`cmd/emu/sysw_test_payload_host_test.go:113-115`:

```go
names := []string{"syswTestPayload", "syswTestDigest", "sysw_test_payload.bin"}
```

and `cmd/emu/confinement_test.go:26-33`'s `guarded` list likewise. A new blob
introduced for C1's walks carries new identifiers, and **no existing test looks
for them**. The confinement property would read as held while being untested for
the newest secret-bearing file in the tree — precisely the
`comments-outlive-their-conditions` shape, and the file's own header says the
confinement "gets tests, or it is not a property, it is a habit."

**Bounded fix.** One line in S1's deliverables and one in its gate: the new
payload's identifiers are added to the confinement guard (a fourth entry in the
`names` slice plus an allowlist entry, or a sibling test on the
`TestSyswTestPayloadIsConfinedToJSOnlyFiles` pattern including the `checked <`
floor), and the gate line is "the confinement test names the new blob —
demonstrated failing when the `//go:build js` tag is removed."

---

## I2 — Important: §5.1's source seam is ABSENT, and S1 regresses the key-input source picker

**The requirement.** §5.1 is REQUIRED: "Source selection MUST therefore be built
**once**, as one seam used by every seed **and key** input in the constructor,
rather than as a per-flow addition… It MUST be shaped so NFC is a third source
added later **without re-opening its call sites** — that is the whole reason for
building it once." §6 P4 lists it as a deliverable in the same breath as the
model the plan did build: "**The source seam** and the slot-assignment model;
the seed↔key consistency gate; R-1's ratification."

**Absent.** `§5.1` is cited exactly once in the plan (line 567), and only for the
unrelated SCAN-row ruling. S4 — which is P4 — lists four implementation bullets
(slot-source model, the gate, per-seed passphrase, the two bindings) and no
seam. `grep -n 'seam'` over the plan finds only `buildMultisigSeedHook seam` and
phase 2's recipe seam.

**And it is worse than an omission — S1 moves the other way.** Today
`gui/multisig_build.go:54` asks the operator *where the key comes from*:

```go
if body, ok := syswOffer(ctx, th, sysw.ClassMDMK, "First card from where?"); ok {
```

S1's implementation bullet replaces that with "every `ClassMDMK` record fed
through `bundleGatherFlow`'s `offer()`" — i.e. the only per-key **source
selection point in the constructor is deleted** and the payload is hardwired.
That is a per-flow decision, not a seam, and it guarantees the later NFC plan
re-opens this exact call site — the one outcome §5.1 exists to forbid. (Removing
the picker may well be right for phase 1 — a one-row picker is not a choice
per `gui/derive_xpub.go`'s D9 rule — but the spec requires the *shape* to admit
a third source, and no stage rules on that.)

**Bounded fix.** Two lines in S4's implementation, and one in S1: state whether
§5.1 is discharged by the existing `syswSeedPicker`/`syswOffer` pair (in which
case say so, say that per-slot seed entry routes through it unchanged, and give
S4 a test that the picker is reached at *each* slot's entry, not once), or that
S1's hardwiring is a deliberate phase-1 narrowing with the extension point named
— and file the re-opening cost against F-158 so the NFC plan inherits it.

---

## I3 — Important: §4.1's per-seed passphrase binding has no test

**The requirement.** §4.1, final bullet: "**The passphrase prompt is PER SEED**,
asked at that seed's entry, and the `(seed, passphrase)` pair is the derivation
unit everywhere this spec says 'seed'." The spec then states, in its own words,
that *nothing else can catch a violation*: "for a new-seed slot there is no card
to cross-check, so **no row of §4.3 could catch it**. The ms1 backup carries
entropy only, never the passphrase, so a wrong binding is **invisible in every
engraved artifact**."

**Discharge quality: PARTIAL.** The plan has one implementation bullet — S4,
"Per-seed passphrase (§4.1), asked at that seed's entry" — and **no test**.
S4 t7 (`TestGateNeverPrintsSeedOrPassphrase`) is a display assertion. S4 t8's
scrub test observes seeds through `buildMultisigSeedHook`, not pairings. This is
exactly the §4.2 shape: named in a stage's prose while nothing implements it.

**Why it matters more than a missing test usually does.** A hoisted
flow-global passphrase applied to masters A and B produces keys that are
internally consistent, engrave cleanly, pass every §4.3 row, pass the byte
comparison against the host (the host is given the same wrong pairing in the
recorded input tuple), and restore to a wallet the operator cannot reproduce.
The failure surfaces at funding time, from steel.

**Bounded fix.** One test in S4:
`TestPassphraseIsBoundPerSeed` — Trace B's two masters entered with **different**
passphrases; assert each slot's derived key equals the key derived from *its own*
`(seed, passphrase)` pair. **Mutation: hoist the first seed's passphrase to all
seeds — master B's slots must go red.** Without the mutation the test passes on
a flow-global implementation whenever the two passphrases happen to be empty.

---

## I4 — Important: §4.6 SAFE's frame-receiver restrictions are ABSENT from the harness S0 builds

**The requirement**, normative in §4.6 SAFE: "The §4.5 walk's frame receiver is
**pinned to one origin and accepts flat filenames only** —
`design/journeys/shot_server.py` is the precedent and states why both
restrictions are load-bearing."

**Absent.** `grep -in 'shot_server\|origin.*pinned\|flat filename\|CORS'` over
the plan returns zero. S0 deliverable 1 builds a **new** harness that captures
artifacts and writes gate records — the precise component this bullet governs —
and says nothing about either restriction.

The precedent's own header states the stakes (`design/journeys/shot_server.py:9-21`):
a localhost server taking "a filename and a payload from a web page… is
precisely the shape that turns a scratch tool into an **arbitrary-file-write
primitive**", and the original version's `Access-Control-Allow-Origin: *` meant
"any site the operator happened to be browsing could drive this server while it
ran." A second harness rebuilt without those two properties re-opens a hole this
project already closed once.

**Bounded fix.** One line in S0 deliverable 1: "The frame/artifact receiver
reuses `design/journeys/shot_server.py`, or reproduces both of its
restrictions — CORS pinned to one origin, and a `NAME_RE` whitelist re-checked
after path resolution." And one line in S0's gate: a negative test that a
`../` name and a foreign `Origin:` are both rejected.

---

## I5 — Important (over-reach): S5's plate-order ruling changes a flow outside §3's scope, and the spec's R0 never examined it

**The plan text**, S5: "**Public plates first, secret plates last**… The ms1-first
order is inherited convention, not a ruling, and S5 already owns
`multisigEngraveCards`' multi-ms1 generalisation." Plus: for any set containing
an ms1, the abort warning says DESTROY rather than discard.

**The spec requires neither.** Plate order appears nowhere in
`SPEC_multisig_build_repair.md`; §3's in-scope list is D-1..D-4, multi-slot
divergent origins, the payload source, the consistency gate, drivability and the
hardware gate. This is the plan making a normative device-behaviour ruling that
the spec's R0 gate never saw.

**And the blast radius is larger than the plan states.** Measured at `a10d007`:

```
gui/multisig.go:163:      cardsOut := multisigEngraveCards(b.MS1, b.MK1, b.MD1, full)
gui/multisig_build.go:167: cardsOut := multisigEngraveCards(b.MS1, b.MK1, b.MD1, full)
```

`multisigEngraveCards` is **shared with the Engrave-Multisig-from-supplied-cards
flow** (`gui/multisig.go:163`), which is not in this spec's scope. Reordering
inside it changes the plate order an operator sees in a flow this cycle is not
otherwise touching, and `gui/multisig_engrave.go:5` records that it deliberately
"mirrors `singleSigEngraveCards`" — so the change also silently diverges the two
conventions. The plan's claim that the abort text "touches no other flow's call
site" is true of the *cards-derived gate*; it is not true of the *ordering*.

The change itself is a good idea (and arrived from the failure-states lens, F2).
The finding is that it is being made in a plan rather than in the spec, and its
scope claim is wrong.

**Bounded fix.** Either (a) scope the reorder to the build flow — order the
cards at the `multisig_build.go:167` call site, leaving `multisigEngraveCards`
and `gui/multisig.go` alone, and say so; or (b) keep the shared change and state
in S5 that `gui/multisig.go:163` and the `singleSigEngraveCards` mirror are
knowingly included, with a test pinning the new order for **both** callers. In
either case add one sentence recording that the ruling originates in the plan,
not the spec.

---

## Minor — recorded only

- **M1 — §4.2's "MUST NOT be written anywhere" is unasserted.** S4 t8 proves
  zeroing on exit, not non-persistence. One line in t8's description ("and no
  seed byte reaches `sysw`, the toolpath, or any log sink") would close it.
- **M2 — §4.3's review screen has no test.** "Chosen by the operator and shown
  on a review screen before assembly" is NORMATIVE; S4 names the screen in an
  implementation bullet only. S1 t5 tests the `@N` ordering on the *gather*
  review; nothing tests that each slot's **source** is displayed.
- **M3 — §4.6 SAFE's "public by construction" is never restated.** The plan
  mints new fixtures (Trace B's masters A and B, a `both`-slot card, C1's
  payload) without the rule. One line in §3's preamble.
- **M4 — two "file it" obligations are unfiled.** §4.6 requires the tier-1 sweep
  residue be "a follow-up owned by `polish / v0.0.1`" and the plan §4 asserts it
  is "filed"; §1a says a `mk encode --chunk-set-id` flag must be filed not
  built. **Measured: `FOLLOWUPS.md` ends at F-159 and contains neither.** The
  plan states compliance it does not have.
- **M5 — S0's harness is mis-tiered.** §4.6's tier 2 is "genuinely CPU-bound
  (real KDF work), behind `testing.Short()`", and its normative bullet admits
  only tier 1 or CPU-bound-behind-`-short` for new tests. A harness that shells
  out to the primary binaries is I/O-bound and is part of the §4.5 walk — tier
  3, "per stage gate" — not tier 2, "pre-commit". Neither `-short` nor
  `testing.Short()` appears in the plan.
- **M6 — R-1 is never cited.** §6 P4 names "R-1's ratification" as a P4
  deliverable; S4 (= P4) does not mention it. The substance is discharged (S3
  corrects the comments, S4 adds the bounding gate), so this is bookkeeping —
  but P4's deliverable list is not reproduced in full anywhere, which is how the
  source seam (I2) went missing from the same sentence.
- **M7 — plan §3's universal is false for S0.** "Every stage's gate includes the
  §4.5 emulator walk"; S0's gate is unit tests plus a refusal demonstration. S0
  is plan-added and outside §6, so §4.5/R-4 do not bind it — but the plan's own
  sentence says otherwise. Add "(S0 excepted — it has no operator-visible
  capability to walk)".
- **M8 — a set of smaller over-reaches, each outside the spec.** Recorded
  together because none is wrong, and none was R0-reviewed against the spec:
  Oracle 2 in full (S0 d2/d3 — published-BIP vectors and `address_test.go`
  provenance derive from an operator criterion, not from any spec section);
  S0 d4 (the `md/testdata/` 0.36→current re-pin, which the plan itself measured
  at **zero byte drift**, i.e. coverage work in a package this spec does not
  touch — the same shape as the 54 s→10 s sweep the plan correctly refuses);
  S4's "Bound the walk-away" idle limit; S5's EXPERIMENTAL-warning rewrite and
  per-slot key display; S5 t7's determinism property plus a restore-doc recovery
  procedure; S6's ms1 read-back. Suggest one sentence per item marking it
  plan-originated, so a later reader does not go looking for the spec clause.

---

## What I checked and did not find

Stated so the next reviewer does not re-derive it:

- **No contradiction with a spec normative point.** Every place the spec offers
  two arms, the plan picks one and records why (P1's refuse-vs-warn → REFUSE;
  §4.1a-3's engrave-both-vs-refuse → engrave both; the mk1/ms1 comparison plane
  strengthened from "presence" to byte equality, explicitly a floor being
  exceeded). All permitted.
- **No smuggled scope from §3's exclusion list.** Grep for taproot, `tr(`,
  miniscript operators, Sealed Payload, wire-format and `ClassCodex32Secret`
  finds only the exclusion statements in plan §4.
- **The plan's two count-bearing claims are TRUE at `a10d007`**, re-run rather
  than inherited: `grep -rn TYPED-ONLY --include='*.go'` → **9** occurrences in
  `gui/multisig.go` ×4, `gui/bip85.go` ×2, `gui/singlesig.go` ×2,
  `gui/multisig_build.go` ×1, none in the verify flows; `scriptName` has exactly
  **3** call sites (`gui/md1_inspect.go:58`, `gui/multisig_restore.go:51`,
  `gui/bundle.go:315`) and no consumer outside `gui`.
- **§4.3's outcome table is fully discharged** — all six rows map to a named,
  mutation-checked S4 test. This is the strongest part of the plan.
- **The deferred §9 is not silently depended on.** No stage requires a
  miniscript type checker, a recipe catalogue, or a `tr` root.

Both repos left clean; this report is the only file written.
