# S5 block C — the multi-select slot picker, and per-leg verify

**Implementer:** single implementer (this agent), TDD.
**Worktree:** `/scratch/code/shibboleth/wt-s5`, branch `s5-multislot`.
**Base:** `7910e00` (clean). **Commit:** `4b10319`. Working tree clean after commit.
**Toolchain:** `nix develop --command …` (Go 1.26; `go.mod` says 1.25.10).

Nothing outside `wt-s5` was touched. `/scratch/code/shibboleth/seedhammer` and
`/scratch/code/shibboleth/seedhammer-s5` were not read from or written to.

---

## 1. What changed

### 1a. The multi-select picker (`@S` → a SET)

| site | change |
| --- | --- |
| `gui/multisig_build.go:682` | **new** `multisigRemainingSlotChoices(n, held) (labels []string, slots []int)` — the not-yet-held slots, with the slot each row maps to (index-aligned, so a chosen ROW is never mistaken for a slot NUMBER once `@0` is taken). |
| `gui/multisig_build.go:725` | **new** `multisigSelfSlotPickFlow(ctx, th, n) ([]int, bool)` — the picker. |
| `gui/multisig_build.go:861` | `case stageSelfSlot` now calls the picker and assigns the set; the old single-select `ChoiceScreen` + `p.SelfSlots = []int{sIdx}` and its "the multi-select screen is a later block's" comment are gone. |
| `gui/multisig_build.go:3-9` | `slices` added to imports (`slices.Sort` on the held set). |

**Shape — composed, not invented.** I checked for an existing multi-select widget
first, as briefed: `grep -rn "MultiSelect\|multiSelect\|Multiselect\|Checkbox\|checkbox" --include=*.go gui/`
returns **zero** production hits (only three comments containing the words
"multi-select"). The shipped idiom for "produce a subset as `[]int`" is
`buildCosignerPickFlow` (`gui/multisig_build_payload.go:314`) — a loop of bounded
`ChoiceScreen`s. I reused that. A toggle-list would have needed a selection
MARKER, and a marker is one more glyph that can raster to nothing on the screen
that decides which keys are the operator's — the F-78/F-151 failure, twice
already in this tree.

Sequence: `"Your slot" / "Which slot is your key?"` (**unchanged, character for
character**) → `"Your slots" / "Do you hold another slot?"` (row 0 = `NO, THAT IS
ALL`) → `"Your slots" / "Which other slot is yours?"` over the remaining slots →
repeat. With exactly one slot left, saying yes takes it **without drawing a
one-row picker** (this package's own rule, quoted in the code).

Three consequences that were deliberate:

- **The all-defaults walk is unchanged in OUTCOME.** Row 0 of the first screen is
  `@0` and row 0 of the "another?" screen is "no", so accepting every default
  still produces `{@0}` — what the pre-S5 picker produced. Pinned by
  `TestSelfSlotPickerDefaultIsTheShippedOne`.
- **`"Which slot is your key?"` stays a single-site walk needle**
  (`cmd/emu/needle_test.go:49`), so the three walk drivers keep their anchor.
- **Back never confirms.** Back at any of the three surfaces returns `ok=false`,
  which `buildParamPickFlow` reads as "step back one stage" — its documented rule
  for every stage above the first. An operator who has picked `@0`, been asked
  about a second slot and pressed Back has not said "just @0".

### 1b. The `both` question — my decision, and what it costs

See §4. Code: `gui/multisig_build_slots.go:494` (`buildSelfSourceFlow` now takes
the whole held set) and `:528` (**new** `buildSelfSourceLead`), called from
`gui/multisig_build.go:83`.

### 1c. Per-leg verify

| site | change |
| --- | --- |
| `gui/multisig_match.go:64` | **new** `allUserSlots(...) []int` — EVERY slot a (seed, passphrase) pair accounts for. |
| `gui/multisig_match.go:34` | `findUserSlot` is now a thin wrapper over it. Its contract (first match, `reused` only at ≥2) is unchanged, and asserted so by `TestAllUserSlotsFindsEverySlotOneSeedFills`. The `cc‖pk` comparison now has exactly ONE site. |
| `gui/multisig_supply.go:61` | **new** `extractReadbackMd1AndMk1s(cards) (md1 []string, mk1s [][]string, ok bool)`. |
| `gui/multisig_supply.go` | **deleted** `extractSuppliedMd1AndMk1`. See §6. |
| `gui/multisig_verify.go:74` | **new** `verifyLeg{Slot, B, MS1Readback}`. |
| `gui/multisig_verify.go:90,102,113` | **new** `errVerifyLegHasNoPlate`, `errVerifyPlateUnclaimed`, `errVerifyNoLegs`. |
| `gui/multisig_verify.go:133` | **new** `verifyMultisigLegs(legs, mk1s, md1) error` — the bijection. |
| `gui/multisig_verify.go:159` | **new** `verifyClaimPlate` — pairing. |
| `gui/multisig_verify.go:206` | `multisigVerifyFlow(ctx, th, full bool)` — rewritten. The `derived bundle.Bundle` parameter is gone; it was **never read** in the old body. |
| `gui/multisig_verify.go:348,385` | **new** `multisigVerifyMS1Entry`, `multisigVerifyOKMessage`. |
| `gui/multisig_build.go:356` | call site rewired; **the `:344-351` stale-limit comment is deleted** and replaced by one describing what the verify now does. |
| `gui/multisig.go:181` | the supply path's call site rewired. One code path, not two. |

**The comparator is a bijection**, and both directions are funds-bearing: every
re-derived leg must find its plate (a leg with no plate is the shipped defect
verbatim — check what was shown, call the rest verified), and every plate must be
claimed (an unclaimed plate is steel in the operator's hands that "Verify OK"
would vouch for).

**Pairing is by the mk1's XPUB**, and the choice is load-bearing. The origin PATH
is **not** unique across masters — Trace B's `@0` (master A) and `@2` (master B)
both declare `m/48h/0h/0h/2h` — so a path-keyed pairing hands B's plate to A's leg
and reports a mismatch on an honest readback. The xpub is unique, and pairing on
it leaves `bundle.Verify` real work: fingerprint, origin path, md1 exact string,
mk1↔md1 stub binding on both sides, ms1 recovered entropy and wordlist. A plate
carrying the right key at a lying origin pairs here and fails there — asserted.

**Flow changes**, both consequences of several held slots:

1. **The gather runs FIRST**, before any seed. The readback is what says how many
   legs there are to prove. It also matches the build path's own posture
   (`TestBuildFlow_GatherBeforeSeed`): no secret is resident while a public set is
   resolved.
2. **Several seeds.** Trace B's three plates span two masters, so the flow loops:
   type a seed, cover the slots it accounts for, type that seed's ms1 (full only),
   and if plates remain, offer the next seed. One ms1 **per seed**, carried on the
   leg — not a flow-global — because a build across two masters engraves two seed
   plates and comparing either leg against whichever ms1 the flow happened to hold
   is how a "Full" backup carrying master A twice verifies clean.
3. **Stopping early is `Verify Incomplete`**, naming how many of how many plates
   were checked. Never a pass, never silence. Back at the FIRST seed entry (zero
   legs) returns silently, which is the shipped abandon behaviour preserved.
4. **One scrub site, deferred before the first seed exists** (`:230`), the same
   design as the build flow's `seedRegistry`: every exit is covered by
   construction rather than by remembering.

### 1d. Existing tests and walk drivers updated for the new screen

Test helpers gained the one tap the new screen costs. All are matched on the
**lead**, not the title, because the new title `"Your slots"` contains
`"Your slot"` as a substring and a title match would find the wrong screen:

`gui/multisig_build_flow_test.go:199` (`buildWalkParamPickers`, 11 callers),
`gui/bundle_gather_refusal_test.go:49`, `gui/multisig_build_payloadseed_test.go:60`,
`gui/multisig_build_walk_test.go:179`, `gui/multisig_build_payload_test.go:84`.

`cmd/emu/walk_build_policy.js`, `walk_s3_nested.js`, `walk_s4_gate.js` each gained
one `choose(0, 2, …)` for the new screen. **See §6 — I could not run these.**

---

## 2. Tests, and the RED I saw

TDD: both new test files were written and run **before** any production code
existed. The verbatim red:

```
$ nix develop --command go test ./gui/ -count=1 -run 'TestSelfSlot|TestVerifyCovers|TestAllUserSlots|TestSelfSource'
EXIT=1
# seedhammer.com/gui [seedhammer.com/gui.test]
gui/multisig_verify_legs_test.go:62:98: undefined: verifyLeg
gui/multisig_verify_legs_test.go:72:11: undefined: allUserSlots
gui/multisig_verify_legs_test.go:76:16: undefined: verifyLeg
gui/multisig_verify_legs_test.go:82:21: undefined: verifyLeg
gui/multisig_verify_legs_test.go:89:52: undefined: verifyLeg
gui/multisig_verify_legs_test.go:124:13: undefined: verifyMultisigLegs
gui/multisig_verify_legs_test.go:146:11: undefined: verifyMultisigLegs
gui/multisig_build_selfslots_test.go:50:14: undefined: multisigSelfSlotPickFlow
gui/multisig_build_selfslots_test.go:295:11: undefined: buildSelfSourceLead
gui/multisig_build_selfslots_test.go:306:11: undefined: buildSelfSourceLead
gui/multisig_verify_legs_test.go:146:11: too many errors
FAIL	seedhammer.com/gui [build failed]
FAIL
```

**Stated plainly: this first red is a COMPILE failure, which is a weak red** — it
proves the API did not exist, not that the behaviour was absent. The *behavioural*
red for the load-bearing claim is the mutation check in §3, which is red against
working code.

A second, genuinely behavioural red arrived the moment the picker landed and the
full suite ran. Counted, not estimated: **12 top-level tests, 20 including
subtests** (`grep -c "^--- FAIL"` = 12, `grep -c -- "--- FAIL"` = 20). Verbatim,
in full:

```
--- FAIL: TestGatherPendingRefusalIsReadableFromBuild (0.20s)
    bundle_gather_refusal_test.go:55: Include key fingerprints? not shown; got "Doyouholdanotherslot?NO,THATISALLYES,ONEMOREYourslots"
--- FAIL: TestBuildFlowRefusesDuplicateBeforeReview (0.05s)
    multisig_build_dupkey_test.go:242: Fingerprints picker not shown
--- FAIL: TestBuildFlowDuplicateNeverReachesReview (0.01s)
    multisig_build_dupkey_test.go:330: Fingerprints picker not shown
--- FAIL: TestBuildFlow_GatherBeforeSeed (0.02s)
    --- FAIL: TestBuildFlow_GatherBeforeSeed/no_payload_refuses_before_the_gather,_naming_the_host_route (0.01s)
        multisig_build_flow_test.go:239: Fingerprints picker not shown
    --- FAIL: TestBuildFlow_GatherBeforeSeed/with_a_payload_the_gather_runs,_and_Back_leaves_before_any_seed (0.01s)
        multisig_build_flow_test.go:276: Fingerprints picker not shown
--- FAIL: TestBuildFlowAcceptsDivergentOriginCard (0.03s)
    multisig_build_origin_test.go:149: Fingerprints picker not shown
--- FAIL: TestBuildGathersEveryCosignerFromPayload (0.00s)
    multisig_build_payload_test.go:191: fp picker not shown
--- FAIL: TestBuildIgnoresMd1RecordsInThePayload (0.00s)
    multisig_build_payload_test.go:370: fp picker not shown
--- FAIL: TestBuildOverSupplySelectionIsWalkable (0.01s)
    multisig_build_payload_test.go:744: fp picker not shown
--- FAIL: TestBuildTakesTheSelfSeedFromThePayload (0.11s)
    multisig_build_payloadseed_test.go:66: Include key fingerprints? not shown; got "Doyouholdanotherslot?NO,THATISALLYES,ONEMOREYourslots"
--- FAIL: TestBuildRefusesDuplicateOnAPayloadSourcedSeed (0.00s)
    multisig_build_payloadseed_test.go:160: Fingerprints picker not shown
--- FAIL: TestGateStillFiresAfterOriginsDiverge (0.04s)
    --- FAIL: TestGateStillFiresAfterOriginsDiverge/PROCEED_when_the_key_is_genuinely_derived_at_the_card's_own_origin (0.00s)
        multisig_build_s5_flow_test.go:125: Fingerprints picker not shown
    --- FAIL: TestGateStillFiresAfterOriginsDiverge/FAIL_naming_the_slot_when_the_key_was_derived_somewhere_else (0.03s)
        multisig_build_s5_flow_test.go:166: Fingerprints picker not shown
--- FAIL: TestBuildFlowScrubsEverySeedOnEveryExit (0.02s)
    --- FAIL: TestBuildFlowScrubsEverySeedOnEveryExit/Back_at_the_passphrase_prompt (0.01s)
        multisig_build_scrub_test.go:127: Fingerprints picker not shown
    --- FAIL: TestBuildFlowScrubsEverySeedOnEveryExit/the_gate_FAIL_screen (0.00s)
        multisig_build_scrub_test.go:176: Fingerprints picker not shown
    --- FAIL: TestBuildFlowScrubsEverySeedOnEveryExit/Back_at_the_EXPERIMENTAL_warning (0.00s)
        multisig_build_scrub_test.go:239: Fingerprints picker not shown
    --- FAIL: TestBuildFlowScrubsEverySeedOnEveryExit/ctx.Done_unwind (0.00s)
        multisig_build_scrub_test.go:280: Fingerprints picker not shown
```

That is the correct red for "a new screen exists in the flow", and it is what
drove §1d. Two of the failures print the screen the flow was actually stuck on —
`"Doyouholdanotherslot?NO,THATISALLYES,ONEMOREYourslots"` — which is the new
picker's second screen with both its rows, drawn and rendering. All were fixed by
adding the tap the screen costs, never by weakening an assertion.

### New tests

`gui/multisig_build_selfslots_test.go`

| test | claim |
| --- | --- |
| `TestSelfSlotPickerDefaultIsTheShippedOne` | all-defaults still yields `{@0}` — the regression floor for every existing walk. |
| `TestSelfSlotPickerSelectsASet` | picks `@2` then `@0` on n=4, gets `[0 2]`. Non-contiguous **and** not in pick order, so it cannot pass by returning a range or the taps verbatim. |
| `TestSelfSlotPickerTakesTraceB` | `{0,1,2}` of n=4 — the shape S5 exists for. |
| `TestSelfSlotPickerBackAbandons` | 3 subtests: Back at each of the three surfaces returns `ok=false`; in particular Back at "another slot?" does **not** confirm the partial set. |
| `TestSelfSlotPickerNeverAsksAOneAnswerQuestion` | with one slot left, no screen is drawn for it. |
| `TestSelfSlotSetReachesParams` | drives the WHOLE `buildParamPickFlow` and reads `p.SelfSlots == [0 2]`. Separate from the picker's own test on purpose: a right picker plus a flow that drops the set is two green units and a broken flow. |
| `TestSelfSourceQuestionNamesEverySlotItAnswersFor` | the one-slot lead still carries the pinned needle `"key on a card?"`; the plural lead names `@0`, `@1` and `@2`. |

`gui/multisig_verify_legs_test.go`

| test | claim |
| --- | --- |
| `TestVerifyCoversEveryLeg` | 8 subtests over Trace B's real engrave output: honest 3-plate readback PASSES; a foreign-but-valid plate substituted for **each** leg in turn FAILS **naming that slot** (including the last — the "not just the first" half); a plate with the right key at a lying origin FAILS; a 4th unclaimed plate FAILS; a missing plate FAILS; zero legs FAILS rather than vacuously passing. |
| `TestVerifyCoversEveryMastersSecret` | full-mode Trace B: each master's ms1 against its own legs PASSES; master B's legs against master **A's** seed plate FAILS. |
| `TestAllUserSlotsFindsEverySlotOneSeedFills` | master A accounts for `[0 1]` of Trace B; `findUserSlot`'s first-match contract is unchanged by the refactor. |

`gui/multisig_supply_test.go` — `TestExtractSuppliedMd1AndMk1` became
`TestExtractReadbackMd1AndMk1s`, with the "two mk1 → ambiguous, refuse" row
replaced by **"SEVERAL mk1 + one md1 → ok, in gather order"**. That row is the
whole defect at the filter layer: a readback filter that drops plates 2 and 3
makes the verify single-leg whatever the comparator downstream does. Every other
row (one md1, no ms1, missing card) is unchanged.

The plates and legs in the verify tests come through the **production**
`assembleBuildPolicy` + `buildEngraveTail`, and plate↔leg lookup in the test
helper decodes both rather than trusting engrave order.

---

## 3. Mutation check

Mutation applied to `gui/multisig_verify.go:134` — `for _, l := range legs` →
`for _, l := range legs[:1]`, with a marker printed **from inside the mutated
loop** so an edit that landed but never executed could not pass for evidence.

Verbatim (mutant run):

```
$ nix develop --command go test ./gui/ -count=1 -run 'TestVerifyCoversEveryLeg|TestVerifyCoversEveryMastersSecret' -v
MUTANT EXIT=1
=== RUN   TestVerifyCoversEveryLeg
=== RUN   TestVerifyCoversEveryLeg/every_leg_matched_by_its_own_plate_PASSES
MUTANT-RAN: verifying only legs[0] (@0) of 3 leg(s)
    multisig_verify_legs_test.go:125: an honest three-plate readback FAILED: verify: read-back key plate 2 belongs to no leg of this policy
=== RUN   TestVerifyCoversEveryLeg/a_WRONG_plate_for_@0_FAILS
MUTANT-RAN: verifying only legs[0] (@0) of 3 leg(s)
=== RUN   TestVerifyCoversEveryLeg/a_WRONG_plate_for_@1_FAILS
MUTANT-RAN: verifying only legs[0] (@0) of 3 leg(s)
    multisig_verify_legs_test.go:154: the failure "verify: read-back key plate 2 belongs to no leg of this policy" does not name @1, so the operator cannot tell WHICH plate to re-cut
=== RUN   TestVerifyCoversEveryLeg/a_WRONG_plate_for_@2_FAILS
MUTANT-RAN: verifying only legs[0] (@0) of 3 leg(s)
    multisig_verify_legs_test.go:154: the failure "verify: read-back key plate 2 belongs to no leg of this policy" does not name @2, so the operator cannot tell WHICH plate to re-cut
=== RUN   TestVerifyCoversEveryLeg/a_plate_carrying_the_right_key_at_the_wrong_origin_FAILS
MUTANT-RAN: verifying only legs[0] (@0) of 3 leg(s)
=== RUN   TestVerifyCoversEveryLeg/a_plate_no_leg_claims_FAILS
MUTANT-RAN: verifying only legs[0] (@0) of 3 leg(s)
=== RUN   TestVerifyCoversEveryLeg/a_leg_with_no_plate_at_all_FAILS
MUTANT-RAN: verifying only legs[0] (@0) of 3 leg(s)
=== RUN   TestVerifyCoversEveryLeg/no_legs_at_all_FAILS_rather_than_vacuously_passing
--- FAIL: TestVerifyCoversEveryLeg (0.12s)
    --- FAIL: TestVerifyCoversEveryLeg/every_leg_matched_by_its_own_plate_PASSES (0.00s)
    --- PASS: TestVerifyCoversEveryLeg/a_WRONG_plate_for_@0_FAILS (0.00s)
    --- FAIL: TestVerifyCoversEveryLeg/a_WRONG_plate_for_@1_FAILS (0.00s)
    --- FAIL: TestVerifyCoversEveryLeg/a_WRONG_plate_for_@2_FAILS (0.00s)
    --- PASS: TestVerifyCoversEveryLeg/a_plate_carrying_the_right_key_at_the_wrong_origin_FAILS (0.00s)
    --- PASS: TestVerifyCoversEveryLeg/a_plate_no_leg_claims_FAILS (0.00s)
    --- PASS: TestVerifyCoversEveryLeg/a_leg_with_no_plate_at_all_FAILS (0.00s)
    --- PASS: TestVerifyCoversEveryLeg/no_legs_at_all_FAILS_rather_than_vacuously_passing (0.00s)
=== RUN   TestVerifyCoversEveryMastersSecret
=== RUN   TestVerifyCoversEveryMastersSecret/each_master's_seed_plate_matched_to_its_own_legs_PASSES
MUTANT-RAN: verifying only legs[0] (@0) of 3 leg(s)
    multisig_verify_legs_test.go:247: an honest full readback FAILED: verify: read-back key plate 2 belongs to no leg of this policy
=== RUN   TestVerifyCoversEveryMastersSecret/master_B's_legs_against_master_A's_seed_plate_FAILS
MUTANT-RAN: verifying only legs[0] (@0) of 3 leg(s)
--- FAIL: TestVerifyCoversEveryMastersSecret (0.04s)
    --- FAIL: TestVerifyCoversEveryMastersSecret/each_master's_seed_plate_matched_to_its_own_legs_PASSES (0.01s)
    --- PASS: TestVerifyCoversEveryMastersSecret/master_B's_legs_against_master_A's_seed_plate_FAILS (0.01s)
FAIL
FAIL	seedhammer.com/gui	0.174s
FAIL
```

**Evidence the mutated line RAN:** `MUTANT-RAN: verifying only legs[0] (@0) of
3 leg(s)` printed on **every** arm, and the trailing `of 3 leg(s)` is read off
`len(legs)` at that line — so the marker also proves the slice held three legs
while the loop iterated one. Four subtests across two tests went RED, on two
independent grounds: the honest-readback PASS arms (leftover plates) and the
slot-naming assertions on legs `@1` and `@2` (which is the per-leg coverage claim
itself, not the leftover check).

Mutation reverted; the post-revert gate in §5 is the proof.

---

## 4. The decision you asked me to make and state

> `buildSelfSourceFlow` asks the derived-vs-`both` question ONCE and applies the
> answer to every held slot. Per-slot, or one answer for all?

**I kept ONE ANSWER FOR ALL HELD SLOTS, and made the question announce its own
scope.** The lead now reads `"Are your @0, @1 and @2 keys on cards?"` when several
slots are held, and remains the byte-identical `"Is your @0 key on a card?"` when
one is (that string is a pinned single-site walk needle). The reasoning is written
into `gui/multisig_build_slots.go:494` so it outlives this report.

Against §0.1's ladder:

1. **Authority — nothing to cite.** No standard rules whether a held slot's key
   also sits on a card. What replaces an authority here is the operator's own
   answer, and they do give one.
2. **Auditability — the binding clause — PASSES, and it is what licenses the
   assumption.** The resulting per-slot assignment is printed slot by slot on
   `buildSlotSourceLines`' "Key sources" review **before anything is derived or
   assembled**: `@1 yours: derived from your seed for @1` against `@1 yours:
   payload card 2, checked against …`. A wrong answer is readable, not invisible.
   It is readable in the KEPT artifacts too: a `derived` slot's key and origin
   land in that slot's engraved mk1 and on the restore doc.
3. **Reversibility — PASSES, and this is where the code changed.** Clause 3 puts
   the announcement on the decision surface itself. The old screen named ONE slot
   and silently answered for the rest; that announced the assumption nowhere, and
   an operator cannot audit a question they were not asked. Naming every slot in
   the lead is the fix.

**What it costs, plainly.** A genuinely MIXED build — `@0` on a card, `@1` derived
— is **not expressible through the screens**. Both wrong answers are loud, not
silent: answering YES when only some held slots are carded fails at the gate,
naming the slot whose card does not derive from its seed
(`errBuildSeedKeyMismatch`); answering NO derives every held slot and says so on
the review. Neither reaches steel quietly.

**Why not per-slot.** Asking per-slot means `buildPolicyParams.SelfFromCard` stops
being a bool and becomes a per-slot set — and that propagates into
`buildSlotSources`, `buildCosignerOrigins`, `buildSlotProvenance` and the
pre-gather supply arithmetic (`open` stops being `N` or `N − held` and becomes
`N − held + bothCount`). That is a change to **the model the brief told me is done
and reviewed to 0C/0I**, and the brief also told me that needing to change it is a
finding to report rather than rework. So: reported, not smuggled in. The
limitation is a PICKER limit, not a model limit — `slotSource` is already
per-slot and `assembleBuildPolicy` already reads the mixture off the held-key
set, so the later change is additive.

**Follow-up to file** (I did not write to `design/FOLLOWUPS.md`; the brief scoped
my edits to the worktree and my writes here to this report):

> **F-new — a MIXED held set is not expressible through the screens.** The
> derived-vs-`both` question is asked once and applied to every held slot
> (`gui/multisig_build_slots.go:494`). An operator holding `@0` on a card and `@1`
> from a seed alone cannot say so; both wrong answers are loud (gate refusal, or a
> derived slot announced on the "Key sources" review) but neither is what they
> meant. Expressing it means `buildPolicyParams.SelfFromCard` becoming a per-slot
> set, which touches `buildSlotSources`, `buildCosignerOrigins`,
> `buildSlotProvenance` and the pre-gather supply arithmetic. **Owning phase:
> the spec (it is a model change, and earns its own R0).**

---

## 5. The gate — verbatim, unpiped, true exit codes

Every command run as `nix develop --command …` from `/scratch/code/shibboleth/wt-s5`,
redirected to a file with `$status` echoed **before** any grep. Nothing was judged
through a pipe.

```
GATE 1/3 go test ./... -count=1  EXIT=0
  ok lines:   51
  FAIL lines: 0

GATE 2/3 gofmt -l ./  EXIT=0
  stdout bytes: 0        (empty; the 60 bytes seen on an earlier run were nix's
                          "warning: Git tree ... is dirty" on STDERR, not gofmt output)

GATE 3/3 go vet ./...  (GOCACHE=<fresh empty dir>)  EXIT=1
  file:line findings: 40
  findings outside _test.go: 0
  classes: 33 × "seedhammer.com/bezier.Point struct literal uses unkeyed fields"
            7 × "testing.ArtifactDir requires go1.26 or later (file is go1.25)"
  first lines:
    gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file is go1.25)
    bspline/bspline_test.go:126:19: seedhammer.com/bezier.Point struct literal uses unkeyed fields
    bspline/bspline_test.go:126:27: seedhammer.com/bezier.Point struct literal uses unkeyed fields
  none of the 40 is in a file this block touched.

EXTRA (not required, run because three walk drivers changed and CI vets the wasm build):
GOOS=js GOARCH=wasm go vet ./cmd/emu/  EXIT=0
```

`exit 1 / 40 findings / 0 outside _test.go` is the stated clean baseline, and the
`GOCACHE` was a freshly created empty directory, so vet actually ran rather than
reporting exit 0 with no output.

Commit `4b10319` on `s5-multislot` carries this gate output in its message.
`git status --short` after the commit is empty.

---

## 6. Could not do / had to assume — read this part

1. **I edited three emulator walk drivers and COULD NOT RUN THEM.**
   `cmd/emu/walk_build_policy.js`, `walk_s3_nested.js` and `walk_s4_gate.js` each
   tap `choose(selfSlot, n, "Include key fingerprints?", …)`, which asserts the
   screen AFTER the slot pick is the fp picker. The new "Do you hold another slot?"
   screen sits between them, so all three walks would have broken at runtime. I
   added one `choose(0, 2, …)` to each. **CI does not run the walks** (measured:
   `.github/workflows/test.yml` only does `GOOS=js GOARCH=wasm go vet ./cmd/emu/`
   plus the static `needle_test.go` checks), so the required gate cannot catch a
   mistake here. The walks are browser-driven and I did not run one. **Someone
   minting a walk record must re-run all three, not just S5's own.** The static
   checks that DO cover them pass: the new lead is deliberately *not* declared as a
   `NEEDLE_*` constant, so `TestWalkNeedleLiteralsAreAllPinned` is unaffected, and
   `"key on a card?"` / `"Which slot is your key?"` each still have exactly one
   production site (`go test ./cmd/emu/` is inside the green 51).

2. **I deleted a production function the brief did not mention.**
   `extractSuppliedMd1AndMk1` had exactly one production caller — the verify — and
   after the rewrite it had none. Leaving a function documented as "the verify's
   readback filter" alive with no caller, immediately next to its replacement, is
   the trap this repo's own notes call out. I deleted it and migrated its six
   subtests (with the "two mk1 → ambiguous" row inverted, since that refusal *is*
   the single-leg defect at the filter layer) and the three call sites in
   `gui/multisig_verify_test.go`. If you would rather it had stayed, that is one
   revert of `gui/multisig_supply.go` plus the test migration.

3. **The first RED was a compile failure, not a behavioural one**, for the newly
   introduced API. Stated in §2 rather than dressed up. The behavioural reds that
   matter are the 16 pre-existing tests that went red on the added screen, and the
   mutation check in §3.

4. **A partial readback FAILS rather than reporting "incomplete".** If the operator
   reads back 1 of 3 plates and types a seed covering 2 slots, `len(legs)` (2)
   already ≥ `len(mk1s)` (1), so the flow goes to the comparator and reports
   `Verify Failed` ("no read-back key plate carries slot @1's key") rather than
   `Verify Incomplete`. It never falsely passes, but the message blames the plates
   rather than the readback. Left as-is; noting it because the wording could be
   sharper.

5. **Holding ALL n slots still dies at the pre-gather supply refusal when no
   payload is loaded.** `classifyCosignerSupply` refuses on
   `state != cosignerSourceLoaded` regardless of `open`, so an all-own-seeds build
   with `open == 0` refuses for want of a payload it does not need. This is
   **pre-existing** (`gui/multisig_build_payload.go:204`), the picker now merely
   makes the shape reachable. I did not touch it — it is not this block's, and
   "fix it" would be a behaviour change to S1's ruling. Worth a follow-up.

6. **Not done, by instruction:** the review-screen per-slot keys, the EXPERIMENTAL
   warning rewrite, DESTROY-not-discard, the passphrase disclosure, F-182, F-185,
   the emulator walk itself, and the gate-record mint. None of them is half-done —
   no partial edits toward any of them are in the commit.

7. **The frozen plan was implemented, not redesigned.** I did not need to change
   `buildPolicyParams.SelfSlots`, `buildEngraveTail`, or the ms1 dedupe key (still
   keyed on the ms1 STRING; the comment at `gui/multisig_build_tail.go:53-75` is
   untouched). The one place the plan left a decision open is §4, and it is decided
   and stated rather than defaulted.
