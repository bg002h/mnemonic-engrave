# S5 implementation — the verify's obligation list is the ENGRAVED SLOT SET

Implementer: single agent (sonnet-tier controller-dispatched), 2026-08-15.
Worktree `/scratch/code/shibboleth/wt-s5`, branch `s5-multislot`.
Base `f0006b7` → commit **`070686a`** (working tree clean at finish).
Design implemented: `design/agent-reports/s5-verify-platecount-design-review.md`
(fable, 1C/2I), amendments 1–4 adopted as written. **No design decision was
re-opened.** Nothing was built that the review rejected: no count, no per-plate
inversion, no relaxation of `verifyMultisigLegs`.

---

## 1. What changed

### Mechanism (production)

| site | change |
| --- | --- |
| `gui/multisig_verify.go:293` | `multisigVerifyFlow(ctx, th, full, expectedSlots []int)` — the signature now carries the engraved slot set (was `(ctx, th, full)`). |
| `gui/multisig_verify.go:169` | **NEW** `verifyFreshSlots(expected, filled []int, covered map[int]bool) ([]int, error)` — the obligation rule: `expected ∩ filled`, minus covered. This is the fix. |
| `gui/multisig_verify.go:142` | **NEW** `errVerifyNoExpectedSlots` — an empty expectation is refused, not vacuously satisfied (design amendment 4 / M3). |
| `gui/multisig_verify.go:28` | **NEW** `multisigVerifyNoExpectationBody` — one string, two refusal sites, so they cannot drift. |
| `gui/multisig_verify.go:299` | Entry guard: an empty `expectedSlots` refuses **before** the gather, so the operator is not sent to the reader for a comparison that cannot run. |
| `gui/multisig_verify.go:332` | Post-gather **length precheck**, named in both directions (amendment 3). Explicitly documented as a courtesy, not the mechanism. |
| `gui/multisig_verify.go:356, 439` | Loop bound and break are `len(expectedSlots)`, not `len(readbackMk1s)`. See §5 — this closed a second, pre-existing false GREEN. |
| `gui/multisig_verify.go:374` | The derive loop calls `verifyFreshSlots`; the hand-rolled `for … if !covered[s]` filter is gone. |
| `gui/multisig_verify.go:392` | Third "nothing fresh" message (review question 5): a seed that IS a cosigner but whose slots this run did not engrave. Drawn with the **same** rule against an empty `covered`, not a second intersection. |
| `gui/multisig_verify.go:201` | `verifyMultisigLegs` — **byte-untouched**, verified by `git diff`. |
| `gui/multisig_build_tail.go:53,128,134` | `buildEngraveTail` returns `([]bundle.Bundle, []int, []bundleCard, error)`; the held-slot index is appended alongside each leg, so the obligation list has the engrave's own provenance rather than being recomputed at the call site. |
| `gui/multisig_build.go:325,364` | Build path threads `engravedSlots` from the tail into the verify. |
| `gui/multisig.go:189` | Supply path passes `[]int{idx}` — the one slot it engraved. |

### Reverted from `f0006b7` (all of it, as instructed)

- `verifyLegWithSameKey` — deleted, with its call in the derive loop.
- Its comment block asserting *"same key implies same origin"* — deleted. That
  claim is the inverse of the measured fact; the real shape is distinct keys at
  distinct origins, which is why the dedupe was inert.
- `TestReusedKeyVerifiesAgainstItsONEPlate` — deleted, replaced (§3).
- The `slices` import is **kept**: `slices.Equal` is gone, `slices.Contains` is
  now used by `verifyFreshSlots`.
- The I-2 `break` at the ms1 entry is **kept** (its parenthetical was updated
  from `readbackMk1s` to `expectedSlots`, which is the variable the claim is
  actually about now).

**Inertness of the reverted dedupe, measured not asserted:** with the whole
mechanism and its test removed, `nix develop --command go test ./gui/ -count=1`
→ `EXIT=0`, `ok seedhammer.com/gui 110.501s`. Nothing else in the suite noticed.

### Comments that would have gone stale

`multisigVerifyFlow`'s header claimed *"the readback is what says how many legs
there are to prove"*. That sentence **is** the defect. It is rewritten to name
the two wrong candidates (the seed, the readback) and why the caller is the only
source, and the gather-first ordering keeps its other, still-true justification
(residency). `errVerifyLegHasNoPlate`'s doc, which pointed at
`verifyLegWithSameKey`, now points at `verifyFreshSlots`.

---

## 2. TDD — each test RED before its implementation, verbatim

### RED 1 — the regression itself, at FLOW level

State: signature + wiring landed, dedupe reverted, **restriction not yet
written** (i.e. the flow ignores `expectedSlots`).

```
=== RUN   TestVerifySupplyShapeChecksTheONEPlateItEngraved
    multisig_verify_flow_test.go:155: the SUPPLY path's own one-plate engrave did not verify. Final screen: "Theread-backbundledoesNOTmatchtheseed.Checktheengravedplates.VerifyFailed"
        The flow engraved one plate for @0 and announced it; a verify that then demands a plate for every slot the seed fills is calling this machine's own correct output bad
--- FAIL: TestVerifySupplyShapeChecksTheONEPlateItEngraved (0.17s)
=== RUN   TestVerifyStillFailsWhenTheENGRAVEDPlateIsWrong
--- PASS: TestVerifyStillFailsWhenTheENGRAVEDPlateIsWrong (0.04s)
=== RUN   TestVerifyRefusesAnEmptyExpectation
    multisig_verify_flow_test.go:206: a verify handed NO engraved slot did not refuse; got "EngraveBundlemd1descriptors:1mk1keys:1Donewhenyouhavereviewedthese."
--- FAIL: TestVerifyRefusesAnEmptyExpectation (0.02s)
=== RUN   TestVerifyBuildShapeChecksEveryEngravedPlate
--- PASS: TestVerifyBuildShapeChecksEveryEngravedPlate (0.08s)
=== RUN   TestBuildEngraveTailReturnsTheSlotsItCut
--- PASS: TestBuildEngraveTailReturnsTheSlotsItCut (0.03s)
=== RUN   TestBuildPassesTheTailsSlotsToTheVerify
--- PASS: TestBuildPassesTheTailsSlotsToTheVerify (0.00s)
FAIL
FAIL	seedhammer.com/gui	0.394s
```

The RED message is the operator's actual screen, extracted from the rendered
frame — the measured defect reproduced end to end, not a helper's return value.

### RED 2 — the obligation rule

```
# seedhammer.com/gui [seedhammer.com/gui.test]
gui/multisig_verify_legs_test.go:391:17: undefined: verifyFreshSlots
gui/multisig_verify_legs_test.go:402:17: undefined: verifyFreshSlots
gui/multisig_verify_legs_test.go:415:17: undefined: verifyFreshSlots
gui/multisig_verify_legs_test.go:425:17: undefined: verifyFreshSlots
gui/multisig_verify_legs_test.go:430:22: undefined: errVerifyNoExpectedSlots
FAIL	seedhammer.com/gui [build failed]
```

### GREEN after the fix

All 6 new + all 5 pre-existing verify tests pass, including every guarantee the
brief listed: wrong plate per slot **naming @N** (three arms, one per slot),
right key at a wrong origin, unclaimed extra plate, missing plate, zero legs,
zero plates, and the two-master ms1 swap.

---

## 3. Tests written

`gui/multisig_verify_flow_test.go` (**new file — drives the real flow**):

| test | what it pins |
| --- | --- |
| `TestVerifySupplyShapeChecksTheONEPlateItEngraved` :151 | THE REGRESSION. One seed, two slots at distinct origins with distinct keys, ONE plate → **Verify OK**. Premise measured in the fixture, not assumed (§4). |
| `TestVerifyStillFailsWhenTheENGRAVEDPlateIsWrong` :168 | Same shape, foreign plate → **Verify Failed**. Stops the above being satisfiable by "report OK". |
| `TestVerifyRefusesAnEmptyExpectation` :196 | Empty slot set refuses, **and refuses before the gather ran**. |
| `TestVerifyBuildShapeChecksEveryEngravedPlate` :221 | Three plates, expected {0,1,2}: master A alone must NOT pass; the flow must ask for the second seed. |
| `TestBuildEngraveTailReturnsTheSlotsItCut` :249 | The obligation list's provenance: indices parallel to the legs, one per mk1 card, and each reported slot's plate declares **that slot's origin** (a right-length wrong-content list is the same defect). |
| `TestBuildPassesTheTailsSlotsToTheVerify` :307 | Source-level pin on the build call site, which sits after `bundleEngrave` and so is unreachable behaviourally. The type system demands *a* `[]int`; this demands the tail's. |

`gui/multisig_verify_legs_test.go:374` `TestVerifyFreshSlotsIsTheEngraversList`
replaces the deleted `TestReusedKeyVerifiesAgainstItsONEPlate` with 7 subtests:
the intersection (supply shape), the covered-slot skip, a slot the seed does not
fill, the empty-expectation refusal (`errors.Is`), and the three kept/added
comparator guarantees — one leg vs its one plate, a MISSING plate still FAILS,
and **ZERO plates FAILS naming @0**.

The new file's header says out loud why it exists: the legs-test file exercises
the comparator against legs a helper built and is structurally blind to the
derive loop, which is where the previous unpinned mechanism hid.

---

## 4. Mutation checks

### The mechanism (the one the brief demanded)

Deleted the `!slices.Contains(expected, s)` clause from `verifyFreshSlots` and
printed a marker on exactly the append the restriction would have skipped:

```
MUT-SLOTSET-RAN appended slot @1, which is NOT in expected [0]
--- FAIL: TestVerifySupplyShapeChecksTheONEPlateItEngraved (0.14s)
    multisig_verify_flow_test.go:155: the SUPPLY path's own one-plate engrave did not verify. Final screen: "Theread-backbundledoesNOTmatchtheseed.Checktheengravedplates.VerifyFailed"
MUT-SLOTSET-RAN appended slot @1, which is NOT in expected [0]
MUT-SLOTSET-RAN appended slot @1, which is NOT in expected [0]
--- FAIL: TestVerifyFreshSlotsIsTheEngraversList (0.07s)
    --- FAIL: TestVerifyFreshSlotsIsTheEngraversList/a_seed_filling_two_slots_proves_only_the_ONE_this_run_engraved (0.00s)
        multisig_verify_legs_test.go:396: a supply engrave of @0 owes [0 1] legs, want [0]. Every extra leg is one this run cut no plate for, and its verify FAILS over correct steel
FAIL	seedhammer.com/gui	112.678s
MUTATED_EXIT=1
```

The marker proves the deleted line's absence changed **execution**, not just
text: slot @1 was appended, which the live clause skips. The **flow** test fails,
so the mechanism is pinned end to end and not merely at the helper — which is the
exact failure of the previous attempt.

Reverted; `grep MUT-SLOTSET-RAN gui/multisig_verify.go` → empty, and the
intersection clause is back at `gui/multisig_verify.go:175`.

### The wiring, separately

The wiring mutation (flow accepts `expectedSlots` and ignores it) is **RED 1**
above — a distinct mutant of a distinct line, caught by the same flow test.

### Premise assertions (guards against a test that stops testing its subject)

`s5SupplyReadback` fatals if master A stops filling ≥2 slots, if the two slots
ever share an origin, or if they ever declare the same key — the last being
exactly the condition under which the reverted same-key dedupe would have been
live. The fixture cannot silently degrade into asserting nothing.

---

## 5. One finding beyond the brief, and what I did about it

Making `expectedSlots` the obligation forced a choice for the loop bound, the
"Verify Incomplete" denominator and the "N not checked yet" count. The design
review's own table answers it (*"arithmetic survives (slots-covered vs
slots-expected)"*, amendment 1), and I followed that — but it is **not** cosmetic:

> With the old `len(readbackMk1s)` bound: engrave 3 plates, read back only @0 and
> @1, type master A. Two legs cover both plates, `2 < 2` is false, the loop
> breaks, the bijection is satisfied — **"Verify OK", and master B's seed was
> never asked for.**

That is a pre-existing false GREEN at `f0006b7`, independent of the supply
defect, and it dies with the expected-slot denominator (2 legs < 3 expected → the
flow asks for the next seed, or reports Verify Incomplete). The post-gather
length precheck also refuses that readback earlier. I did **not** write a
dedicated regression test for it — `TestVerifyBuildShapeChecksEveryEngravedPlate`
covers the adjacent shape (3 plates present, one seed) but not the
2-plates-of-3 readback. **Recommend filing it as a follow-up owned by this
phase's reviewer**, or ask and I will add the arm.

## 6. Prose I touched, and why (the brief scopes screens/prose OUT)

Three strings changed, all forced by the mechanism rather than chosen:

- "Checked %d of the %d key plates **read back**" → "**this run engraved**". The
  number is now `len(expectedSlots)`; leaving the old noun would have made the
  sentence describe a different quantity than the one printed.
- The "next seed?" lead counts expected-minus-covered rather than
  plates-minus-legs (same reason).
- Two **new** strings: the empty-expectation refusal and the length-mismatch
  refusal, both required by the design (amendments 3 and 4).

The M2 supply announcement (*"This key is reused at slots @0 and @1"* — factually
false: the keys are not reused) is **untouched**, per the review's instruction to
file it rather than fold it. The supply path's engrave behaviour (one plate per
matched seed) is **untouched**.

## 7. Gate (true exit codes, cold GOCACHE for vet)

```
nix develop --command go test ./... -count=1     TEST_EXIT=0   ok=51   FAIL=0
nix develop --command gofmt -l ./                GOFMT_EXIT=0  stdout=0 bytes
nix develop --command go clean -cache
nix develop --command go vet ./...               VET_EXIT=1    findings=40
                                                 non-_test.go findings=0
```

`VET_EXIT=1 / 40 findings / 0 outside _test.go` is the stated clean baseline
(34 unkeyed-field in `bspline/bspline_test.go`, 2 `engrave/`, and 4 singles
incl. `gui/op/draw_test.go` go1.26). No pipes were used to judge any command:
every run redirected to a file, echoed `$?`, then grepped the file.

Committed as **`070686a`** on `s5-multislot`, gate output in the message.
`git status --porcelain` empty.

## 8. What I could not do / assumptions

- **Not pushed, not merged** — out of scope; the branch is left at `070686a`.
- The build call site's pass-through is pinned by a **source-text** test, not a
  behavioural one. The verify offer sits after `bundleEngrave`, and no unit test
  in this package drives a real engrave. This is the weakest link in the wiring
  proof and I am flagging it rather than dressing it up; the precedent
  (`funcBody`, `gui/multisig_build_title_test.go:87`) is the repo's own.
- The emulator walk, gate-record mint, `oracle/**`, screens/prose work and the
  engrave tail (`7910e00`) were left alone as instructed.
- Assumed Trace B's `@0` plate is byte-equivalent to what the SUPPLY path would
  cut for master A (same policy, same origin `keys[0].OriginPath`). Not merely
  assumed in the test: `s5SupplyReadback` re-derives the leg at
  `findUserSlot`'s returned origin and locates its plate **by decoding both**,
  so an inequivalence would fatal rather than pass.
