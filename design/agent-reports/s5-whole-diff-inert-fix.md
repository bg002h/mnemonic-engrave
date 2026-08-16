# S5 whole-diff review — LENS: INERT-FIX HUNT, at FLOW level

Agent: opus, post-implementation gate, `main..s5-multislot` @ `7da66bd` (frozen, read-only).
Date: 2026-08-16.

## The one question

For each mechanism S5 added: **if I delete it, does a test actually go RED?**

## Method — 28 ACTUALLY-RUN mutations, not argued ones

The frozen worktree was never touched. `cp -a /scratch/code/shibboleth/wt-s5` into a
scratch lane per mutation, one exact-string edit applied by a Python harness that
**fails the run if its anchor does not match exactly once** (so a mutation cannot
silently no-op), then:

```
nix develop --command go test ./gui/... ./cmd/emu/... -count=1
```

28 mutations, 6 parallel lanes. Baseline in the copied tree measured first: **green**
(`gui 115.727s`, all packages ok). Anchor dry-run: 28/28 matched exactly once.

Harness: `<scratch>/mutations.py`, `<scratch>/run_mut.sh`; per-mutation output in
`<scratch>/out-<id>.txt`; verdicts in `<scratch>/results.txt`.

### Result: 22 KILLED, 6 SURVIVED

| mutation | mechanism disabled | verdict |
| --- | --- | --- |
| M1 | build tail ms1 dedupe (the C1 fix) | KILLED — `TestFullModeEngravesMs1ForEveryMaster`, `TestVerifyCoversEveryMastersSecret` |
| M2 | supply tail mk1 byte-identical dedupe | KILLED (3 tests) |
| M3 | supply tail ms1 dedupe | KILLED (4 tests) |
| M4 | `slices.Equal(readbackMd1, engravedMd1)` policy binding (the C3 fix) | KILLED — `TestVerifyRefusesPlatesFromADifferentPolicy` |
| M5 | `len(engravedMd1)==0` refusal | KILLED |
| M6 | `errVerifyLegHasNoPlate` → skip | KILLED — `TestVerifyCoversEveryLeg` (4 subtests), `TestVerifyFreshSlotsIsTheEngraversList` |
| M7 | unclaimed-plate sweep | KILLED |
| M8 | `verifyFreshSlots` `expected ∩ filled` | KILLED (5 tests) |
| M9 | `verifyFreshSlots` empty-expected refusal | KILLED |
| M10 | `errVerifyNoLegs` → vacuous `nil` | KILLED — `TestVerifyCoversEveryLeg/no_legs_at_all_FAILS_rather_than_vacuously_passing` |
| M11 | §4.1 `duplicateSlotPair` | KILLED (9 tests) |
| M12 | `derivedSlotOrigin` template-awareness (sh(wsh)→1') | KILLED |
| M13 | supply tail → first matched slot only (revert F-188) | KILLED (3 tests) |
| **M14** | **`both` slot derives at the CARD's origin** | **SURVIVED** |
| M15 | ms1-run → mk1-run → md1 emission order | KILLED |
| M16 | "Verify Incomplete" report | KILLED |
| **M17** | **readback-count precheck** | **SURVIVED** |
| **M18** | **`errBuildNoHeldSlot`** | **SURVIVED** |
| **M19** | **ms1-entry `break` → `return`** | **SURVIVED** |
| M20 | no-slot refusal `break` → `return` | KILLED |
| **M21** | **`errSupplyNoMatchedSlot`** | **SURVIVED** |
| M22 | verify entry empty-expectation refusal | KILLED |
| M23 | supply census collapse NOTE | KILLED |
| M24 | `multisigSlotsShareAKey` notice arm | KILLED |
| **M25** | **`MS1Readback` dropped from every leg in the flow** | **SURVIVED** |
| M26 | `OriginDivergent` → always shared | KILLED |
| M27 | verify obligation = filled instead of expected | KILLED |
| M28 | supply verify passes `slots` not `engravedSlots` | KILLED |

The four Criticals this cycle already found are all **pinned** (M1, M2/M13, M4, plus the
walk/needle work). The lens paid off on the six that are not.

---

## FINDINGS

### 1. IMPORTANT — a `both` slot's card-origin binding is unpinned, and mutating it mints a key the policy does not hold

**`gui/multisig_build_tail.go:95-99`**

```go
case slotFromBoth:
    o, err := bip32.ParsePath(cards[s.Card].Path)
    ...
    origin = o
```

The doc comment above it states the funds-bearing rule: *"`both` -> the CARD's declared
origin, because SPEC M-B makes the card authoritative in a `both` slot."*

**Replacing `origin = o` with `origin = derivedSlotOrigin(script, s.Account)` leaves the
whole suite GREEN** (M14).

That is not a semantic no-op — it changes the engraved steel. Measured: no test anywhere
calls `buildEngraveTail` with a `slotFromBoth` source. Every caller
(`multisig_build_s5_test.go:248,297,354,490`, `multisig_verify_legs_test.go:39`,
`multisig_verify_flow_test.go:300`, `multisig_verify_policy_test.go:91`) uses
`s5TraceB(t)`, whose three held slots are all `slotFromSeed`. The `slotFromBoth` fixtures
that do exist (`multisig_build_gate_test.go`, `multisig_build_scrub_test.go`) only reach
`buildSlotGate`, never the tail.

`buildSlotSources` (`gui/multisig_build.go:474-479`) never sets `Account` on a `both`
slot, so it is always 0 and `derivedSlotOrigin(wsh, 0)` is always `m/48h/0h/0h/2h`,
regardless of what the card declares.

**Concrete failing state.** A build the *production gate accepts*: `wsh` 2-of-2, operator
holds @0, answers "my key is on a card", and picks payload roster card 3 = **A@1**, which
declares `m/48h/0h/1h/2h` (the card `cmd/buildpayloadcards` writes into
`cmd/emu/sysw_cards_payload.bin`). Under the mutation the tail derives master A at
account **0**, so the engraved mk1 declares `m/48h/0h/0h/2h` and carries a key the
assembled policy does **not** hold at @0 — a key plate asserting membership of a wallet
whose @0 is a different key. On the real machine that is a plate the coordinator will not
recognise and a slot the operator cannot prove.

**Verified, both directions.** I wrote a 60-line test into my scratch copy
(`gui/zz_m14_proof_test.go`, built through `buildSlotSources` → `buildSlotGate` →
`assembleBuildPolicy` → `buildEngraveTail`):

```
frozen tree:  --- PASS: TestM14BothSlotLegMatchesThePolicyKey (0.03s)
                  card origin "m/48h/0h/1h/2h"; derivedSlotOrigin(wsh,0) "m/48h/0h/0h/2h"
with M14:     --- FAIL: the @0 leg's mk1 declares origin m/48h/0h/0h/2h, but the policy
                  slot's card declares m/48h/0h/1h/2h
              --- FAIL: the @0 leg's mk1 carries a key the policy does NOT hold at @0
```

`buildSlotGate` returned nil on this shape, so it is reachable rather than hypothetical.

**The code is correct today.** The finding is that nothing holds it there, at the one
site that decides what goes on steel, for the one slot kind the S5 diff introduced to the
tail. Fix is the ~60 lines above.

---

### 2. IMPORTANT — `multisigVerifyFlow` is never driven in FULL mode, so the whole ms1 half of the verify is flow-unpinned

**`gui/multisig_verify.go:596-612` (ms1 entry + its `break`), `:621` (`MS1Readback` per
leg), `:677-694` (`multisigVerifyMS1Entry`)**

Two independent mutations survive, and they share one root cause.

* **M25** — `legs = append(legs, verifyLeg{Slot: s, B: b, MS1Readback: ms1Readback})` →
  `MS1Readback: ""`. In full mode the derived leg carries a non-empty `MS1` and the
  readback would be empty, which `bundle/verify.go:77-79` treats as a hard **"ms1
  presence mismatch"**. Every full-mode verify would fail. **Suite stays GREEN.**
* **M19** — the ms1 entry's `break` at `:609` → `return`. The comment there says in
  its own words what that reintroduces: *"a bare return here walked out of the flow with
  NO SCREEN AT ALL -- some plates checked, some not, nothing said, and on the build path
  the next thing the operator saw was the restore document."* **Suite stays GREEN.**

Its sibling, the no-slot refusal `break` at `:590`, IS pinned — M20 was killed by
`TestVerifyReportsIncompleteAfterAMidLoopRefusal`. The two fixes are described as the
same fix for the same reason in the same function; only one of them is held.

**Measured cause.** Every test driver passes `full=false`:

```
gui/multisig_verify_policy_test.go:177   multisigVerifyFlow(ctx, &descriptorTheme, false, ...)
gui/multisig_verify_flow_test.go:114     ... false ...
gui/multisig_verify_flow_test.go:220     ... false ...
gui/multisig_verify_flow_test.go:246     ... false ...
gui/multisig_supply_multislot_test.go:271 ... false ...
```

and `multisigVerifyMS1Entry` has **zero** references outside its own definition
(`grep -rn multisigVerifyMS1Entry gui/ cmd/`). `s5DriveVerifyTwoSeeds` — the only
multi-seed driver — also passes `false`, so the multi-seed × full-mode cell, which is
Trace B's shipping shape, is untested end to end.

**What that leaves unproven.** `TestVerifyCoversEveryMastersSecret` constructs
`verifyLeg` values by hand and calls `verifyMultisigLegs` directly. That proves the
comparator. It cannot prove the thing the comparator depends on: that the flow binds
**each seed's own typed ms1 to that seed's legs**. `verifyLeg`'s own doc says why that
matters — *"a build across two masters engraves two seed plates, and comparing either leg
against whichever ms1 the flow happened to hold is how a 'Full' backup carrying master A
twice verifies clean while master B -- which k=3 needs -- is gone."* That is the exact
funds-loss the S5 verify exists to prevent, and the assignment that prevents it is
`:621`, which M25 shows nothing observes.

**Concrete failing state (M19, the operator-visible one).** Trace B, **full** mode,
3 plates / 2 masters. Operator verifies master A (covers @0 and @1, types A's ms1), is
offered the next seed, types master B, then presses Back at "Type ms1". With `return`:
the flow exits silently, no "Verify Incomplete", and on the build path the next screen is
the restore document — 2 of 3 plates checked and the operator has no way to know.

Fix: one flow-level full-mode driver (gather md1 + 3 mk1, two seeds, ms1 typed per seed)
and a Back-at-ms1 arm asserting "Verify Incomplete".

---

### 3. MINOR — the readback-count precheck is unpinned; only its comment says what it does

**`gui/multisig_verify.go:493-500`**

Deleting the whole `len(readbackMk1s) != len(expectedSlots)` refusal leaves the suite
GREEN (M17). Its message —

> "Read back %s, but this run engraved %s. Present exactly the plates this run cut."

— occurs in the tree only at that production site and in two *comments*
(`gui/multisig_supply_tail.go:91`, `gui/multisig_supply_dupslot_test.go:22`). No
assertion. `TestVerifyRefusesAPartialReadbackOfAThreePlateBuild` passes with the precheck
removed, so it is exercising the bijection, not this.

Correctly rated Minor: the code calls itself *"a courtesy, not the mechanism"* and that
is true — a short readback fails at `errVerifyLegHasNoPlate`, a long one at
`errVerifyPlateUnclaimed`, both pinned. What is lost on regression is only *when* the
operator learns, which the comment claims as the benefit ("learns it before typing a
seed") and nothing checks.

---

### 4. MINOR — two named structural refusals are unreachable and unasserted

**`gui/multisig_build_tail.go:131-133` (`errBuildNoHeldSlot`, M18) and
`gui/multisig_supply_tail.go:160-162` (`errSupplyNoMatchedSlot`, M21)** — both survive
deletion.

Both are honest defence-in-depth for a future caller, and both say so. Confirmed
unreachable today: `gui/multisig_build.go:51-55` refuses an empty `SelfSlots` before the
tail ("The @S picker always sets exactly one held slot, so this cannot fire today"), and
`gui/multisig.go:163-166` refuses zero matches before `supplyEngraveTail`. Neither
identifier is referenced by any test (`grep -rn 'errBuildNoHeldSlot\|errSupplyNoMatchedSlot' gui/`
returns only the definitions and three comments).

Recorded, not gating. A one-line direct call each would pin them; they are cheap because
they need no flow.

---

## What I checked and did NOT find a defect in

Reported here so a later round does not re-spend on it:

* The four Criticals' fixes are all genuinely pinned (M1, M2, M4, M13 all killed, each by
  a test that names the defect).
* The bijection is pinned in **both** directions independently (M6 and M7 killed
  separately), and its empty case is pinned against a vacuous pass (M10).
* `verifyFreshSlots`' intersection is pinned three ways (M8, M9, M27).
* The engrave emission-order contract survives no mutation (M15 killed) — the oracle
  artifact-shape rule is real, not documentation.
* `OriginDivergent` and the §4.1 duplicate check are heavily pinned (M26: 6+ tests;
  M11: 9 tests).
* The supply path's obligation really is the tail's return, not a recomputation (M28
  killed).
* `bundle.Verify`'s ms1 leg compares recovered **entropy**, not the string
  (`bundle/verify.go:83-95`), so an incidentally re-encoded ms1 still matches — read, not
  inferred.
* Already filed, seen and skipped: F-189 (`multisigEngraveCards` retired), F-191.
* Not reported (pre-existing, ~3e-6, and outside S5's diff): `bundleGatherer.offerChunkedMK1`
  dedupes on a **20-bit** `chunk_set_id` (`mk/encode.go:237,329`), so two distinct plates
  of one run colliding there would be silently swallowed as a duplicate.

## Verdict for this lens

Lens is **not** closed by a clean round — it closed by running out of S5 mechanisms worth
mutating. Six unpinned mechanisms found; **two are Important** and both are the cycle's
own headline class ("an unpinned fix is indistinguishable from an inert one"), sitting on
the two surfaces that decide what goes on steel and whether the steel is believed. Finding
1 is proven by a test that runs; Finding 2 is proven by five measured call sites plus two
surviving mutations of code whose own comments describe the defect they prevent.

Both are fixable with test-only changes. No production code needs to move.
