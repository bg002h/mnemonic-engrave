# S5 fold — C1 (policy identity), I1 (byte-identical plate collapse), M2

Implementer: single implementer, TDD, worktree `/scratch/code/shibboleth/wt-s5`,
branch `s5-multislot`. Parent `853534a`, fold commit **`e7da39a`**. Answering
`design/agent-reports/s5-verify-seam-fable-review.md`.

Result: **C1 fixed, I1 fixed per the ruling, M2 fixed, M1 filed not fixed.**
Gate green (below). Working tree clean at finish; every mutation probe reverted
and grep-verified gone before the commit.

---

## 1. What changed, with `file:line`

All paths are in `/scratch/code/shibboleth/wt-s5`, at `e7da39a`.

### C1 — the obligation carries the policy

| site | change |
| --- | --- |
| `gui/multisig_verify.go:360` | `multisigVerifyFlow` signature gains `engravedMd1 []string` |
| `gui/multisig_verify.go:412` | `if !slices.Equal(readbackMd1, engravedMd1)` → named refusal. Runs **before** the decode, **before** the length precheck, **before** any secret is asked for |
| `gui/multisig_verify.go:376` | empty-`engravedMd1` guard, before the gather (an absent policy would make the equality vacuous) |
| `gui/multisig_verify.go:41,51` | the two new operator strings, as named constants |
| `gui/multisig_verify.go:16-30` | the file header's "compared to a clone of itself" scoping note reconciled: still true of the comparator, no longer the whole story of the flow |
| `gui/multisig.go:297` | supply call site passes `suppliedMd1` |
| `gui/multisig_build.go:373` | build call site passes `engraveMd1` |

`verifyMultisigLegs` and `verifyClaimPlate` are **byte-untouched** (`git diff
853534a..e7da39a -- gui/multisig_verify.go` has no hunk in that range).

The engraved md1 reaches steel verbatim on both paths, so an honest readback
reproduces it exactly: `supplyEngraveTail` passes `suppliedMd1` straight to
`multisigEngraveCardsMulti` (`gui/multisig_supply_tail.go:172`), and
`buildEngraveTail` passes `engraveMd1` (`gui/multisig_build_tail.go:134`). No
false-RED surface was introduced — verified by the four pre-existing honest-path
flow tests staying green (§4).

### I1 — one plate for a byte-identical pair, announced

| site | change |
| --- | --- |
| `gui/multisig_supply_tail.go:128,152-157` | mk1-identity dedupe keyed on the **minted string** (`strings.Join(b.MK1, "|")`), exactly as the ms1 dedupe above it. `slots` gains an entry only where a plate was appended, so the obligation list collapses with the plates and can never name steel that does not exist |
| `gui/multisig_supply_tail.go:180` | new `multisigSlotsShareAKey(keys, matched)` — predicate only, deliberately carries **no count** |
| `gui/multisig.go:185-198` | the multi-slot notice is now two-armed; the collapsed arm drops the false "each of those slots holds a DIFFERENT key" |
| `gui/multisig.go:258-266` | the census gains a collapse NOTE, **prepended** (this screen is confirmable from any page, so a note on page three is a note the operator can commit past unread) |
| `gui/multisig_supply_tail.go:27-42` | the file header's "deduping legs by identical mk1 is inert" paragraph scoped to the reused-seed shape, so it does not read as a blanket verdict against the mechanism a hundred lines below it |

Why the notice states no count: it is drawn **before** the engrave-mode choice,
so no leg exists yet, and any number there would be a second predictor of the
tail. The census number comes from the tail's own return (`len(engravedSlots) <
len(slots)`), so it cannot claim a collapse that did not happen or miss one that
did.

### M2 — the three mid-loop refusals

`gui/multisig_verify.go:454` — `return` → `break`, placed **after** the switch,
not inside it (a `break` inside a Go `switch` breaks the switch; three arms that
each looked like they stopped the verify and did not would be worse than the
defect being fixed).

---

## 2. Operator-facing strings added or changed, verbatim

**Added** (`gui/multisig_verify.go:51`, title `Verify Bundle`):

> The read-back wallet policy is NOT the wallet policy this run engraved. These plates belong to a different wallet. Present the md1 and the key plates this run cut.

**Added** (`gui/multisig_verify.go:41`, title `Verify Bundle`):

> This run's wallet policy did not reach the verify, so there is nothing to check the plates against.

**Added** (`gui/multisig.go:185-191`, title `Engrave Multisig`, the collapsed arm
of the multi-slot notice; `%s` is `formatSlotList(slots)`):

> Your seed is at slots %s of this policy, and more than one of them holds the SAME key at the SAME key path. Identical plates would carry identical information, so this run cuts one plate per DISTINCT key, not one per slot. The next screen states the exact count.

**Added** (`gui/multisig.go:260-264`, first line of the `Plates To Cut` census):

> NOTE: slots %s hold only %s between them, so this run cuts %s, not one per slot.

rendered for the delivered shape as:

> NOTE: slots @0 and @1 hold only 1 distinct key between them, so this run cuts 1 key plate, not one per slot.

**Unchanged** (now the *else* arm at `gui/multisig.go:193-197`) — the existing
reused-seed notice, byte-for-byte:

> Your seed is at slots %s of this policy, at a different key path in each, so each of those slots holds a DIFFERENT key. This run engraves %s, one per slot.

No em- or en-dashes in any added string (zero-pixel glyphs, F-78/F-151). No added
string collides with a walk needle in `cmd/emu/needle_test.go`, and no added
*comment* quotes one — `TestBuildFlowNeedlesHaveExactlyOneProductionSite` and
`TestDecoyNeedlesAreStillAmbiguous` are green.

The `Verify OK` copy, `multisigVerifyNoExpectationBody`, and the three mid-loop
refusal messages are unchanged.

---

## 3. Tests, with verbatim RED

New file `gui/multisig_verify_policy_test.go`, new file
`gui/multisig_supply_dupslot_test.go`. Every assertion drives the **real flow**
(`multisigVerifyFlow` / `supplyMultisigPolicyFlow`) through the real gatherer,
seed entry and screens — none is a helper-level test.

### C1 — `TestVerifyRefusesPlatesFromADifferentPolicy`

Fixture `s5PolicyPair`: wallet **P** = Trace B 3-of-4; wallet **P′** = the same
four cosigners at 2-of-4, assembled and engraved through the production
`assembleBuildPolicy` + `buildEngraveTail`. Premises measured, not assumed: the
two md1s differ; master B fills exactly one slot in each; it is the **same
index** in both, so the substitution is index-compatible and cannot pass on the
slot set.

RED at `853534a` (unpiped, `-v`):

```
=== RUN   TestVerifyRefusesPlatesFromADifferentPolicy
    multisig_verify_policy_test.go:119: final screen: "Operatorkeyandsecretverified.Othercosigners'keysaretakenassupplied.VerifyOK"
    multisig_verify_policy_test.go:121: plates from a DIFFERENT wallet verified clean. Final screen: "Operatorkeyandsecretverified.Othercosigners'keysaretakenassupplied.VerifyOK"
        The plate this run cut and the md1 this run cut were never read: the slot set carries index @2 but not the policy that index means, so the readback supplied the policy AND the evidence and compared them against each other
--- FAIL: TestVerifyRefusesPlatesFromADifferentPolicy (0.17s)
```

That is the reviewer's executed false GREEN, reproduced independently.

GREEN after the fix, final screen verbatim:

```
"Theread-backwalletpolicyisNOTthewalletpolicythisrunengraved.Theseplatesbelongtoadifferentwallet.Presentthemd1andthekeyplatesthisruncut.VerifyBundle"
```

(the UI harness strips spaces). The refusal lands **before seed entry** — it is
driven with `s5DriveVerifyTolerant` because `s5DriveVerify` fatals when the
gather does not hand off to a seed, which is itself the proof the refusal is
pre-secret.

Companion arms:

- `TestVerifyStillPassesItsOwnPolicy` — the same fixture's OWN md1 and OWN plate
  still reach `Verify OK`. Without it, "refuse everything" satisfies the arm
  above. Green before **and** after (so it is a genuine non-vacuity guard, not a
  second RED).
- `TestVerifyRefusesAMissingEngravedPolicy` — an empty `engravedMd1` is refused
  before the gather, with `nothing to check the plates against`.
- `TestSupplyPassesTheEngravedPolicyToTheVerify` — source-level wiring assertion
  on the **second** call site. C1 was a two-caller defect; only the build path
  had such a test.
- `TestBuildPassesTheTailsSlotsToTheVerify` — updated to require
  `multisigVerifyFlow(ctx, th, full, engravedSlots, engraveMd1)`.

### I1 — three arms

RED at `853534a` (unpiped, `-v`, abridged to the assertions):

```
=== RUN   TestSupplyTailCollapsesByteIdenticalPlates
    multisig_supply_dupslot_test.go:143: the tail's obligation list is [0 1], want [0]. ...
--- FAIL: TestSupplyTailCollapsesByteIdenticalPlates (0.04s)
=== RUN   TestSupplyDuplicateSlotVerifiesItsOwnOutput
    multisig_supply_dupslot_test.go:188: final screen: "Readback1keyplate,butthisrunengraved2keyplates.Presentexactlytheplatesthisruncut.VerifyBundle"
--- FAIL: TestSupplyDuplicateSlotVerifiesItsOwnOutput (0.06s)
=== RUN   TestSupplyFlowAnnouncesTheCollapseBeforeTheFirstCut
    multisig_supply_dupslot_test.go:210: multi-slot notice: "Yourseedisatslots@0and@1ofthispolicy,atadifferentkeypathineach,soeachofthoseslotsholdsaDIFFERENTkey.Thisrunengraves2keyplates,oneperslot.EngraveMultisig"
    multisig_supply_dupslot_test.go:211: census screen: "PlatesToCutThisengraves9plates.ms1secretshare:1plate(secretseedbackup)mk1key1of2:2plates(accountkeycard)mk1key2of2:2plates(accountkeycard)md1descriptor:4plates(walletpolicydescriptor)Eachplatetakesminutestocut...."
    multisig_supply_dupslot_test.go:212: first engrave screen: "ChooseengravingTEXT+QRTEXTONLYQRONLYCard1of4|Plate1of1"
    multisig_supply_dupslot_test.go:215: the census does not announce the collapse. ...
    multisig_supply_dupslot_test.go:224: the census still numbers TWO key plates for one distinct key: ...
    multisig_supply_dupslot_test.go:230: the multi-slot notice still asserts that each slot holds a DIFFERENT key: ...
    multisig_supply_dupslot_test.go:241: the engrave set is not ms1 + 1 mk1 + md1. The first plate announces "...Card1of4|Plate1of1", want "Card 1 of 3"
--- FAIL: TestSupplyFlowAnnouncesTheCollapseBeforeTheFirstCut (0.08s)
```

The second one is the reviewer's exact permanent false RED, verbatim.

GREEN after the fix, the three screens verbatim:

```
multi-slot notice: "Yourseedisatslots@0and@1ofthispolicy,andmorethanoneofthemholdstheSAMEkeyattheSAMEkeypath.Identicalplateswouldcarryidenticalinformation,sothisruncutsoneplateperDISTINCTkey,notoneperslot.Thenextscreenstatestheexactcount.EngraveMultisig"
census screen:     "PlatesToCutNOTE:slots@0and@1holdonly1distinctkeybetweenthem,sothisruncuts1keyplate,notoneperslot.Thisengraves7plates.ms1secretshare:1plate(secretseedbackup)mk1key:2plates(accountkeycard)md1descriptor:4plates(walletpolicydescriptor)Eachplatetakesminutestocut..."
first engrave:     "ChooseengravingTEXT+QRTEXTONLYQRONLYCard1of3|Plate1of1"
verify final:      "Operatorkeyandsecretverified.Othercosigners'keysaretakenassupplied.VerifyOK"
```

Fixture premises, all measured in `s5DupSlotPremise` /
`TestSupplyTailCollapsesByteIdenticalPlates` rather than assumed: the policy is
FULL (`allSlotsHaveXpub`), master B fills both slots, the two slots declare the
**same key at the same origin**, and the two derived mk1s are **byte-identical**
(compared at `deriveMultisigLeg`, not inferred from the keys agreeing). The md1
is minted through `md.EncodeMultisig` directly, because `assembleBuildPolicy`
refuses duplicates — that asymmetry is exactly the admission the ruling rests on.

### M2 — `TestVerifyReportsIncompleteAfterAMidLoopRefusal`

Needed a new two-seed driver (`s5DriveVerifyTwoSeeds`): the mid-loop refusals
only differ from a first-seed refusal once `legs` is non-empty. Trace B, expected
`{0,1,2}`; master A covers @0/@1; the operator then types master C, which **is**
a cosigner (the payload card at @3) whose slot this run did not engrave —
premise measured in the test, so it cannot silently drift onto the "not a
cosigner" arm.

GREEN, final screen verbatim:

```
"Checked2ofthe3keyplatesthisrunengraved.TherestwereNOTverified.Runverifyagainwiththeremainingseedsbeforefundingthiswallet.VerifyIncomplete"
```

### Existing guarantees, re-run and still green

`TestVerifyCoversEveryLeg` (wrong plate per slot FAILS naming the slot; right
key at wrong origin FAILS; unclaimed extra plate FAILS; missing plate FAILS),
`TestVerifyStillFailsWhenTheENGRAVEDPlateIsWrong`,
`TestVerifyRefusesAnEmptyExpectation`, `TestVerifyRefusesAPartialReadbackOfA
ThreePlateBuild`, `TestVerifyBuildShapeChecksEveryEngravedPlate`,
`TestVerifyOneSlotRunChecksTheONEPlateItEngraved`,
`TestVerifyFreshSlotsIsTheEngraversList`, `TestVerifyPairsByKeyNotByOrigin`,
`TestVerifyCoversEveryMastersSecret`, `TestSupplyEngraveVerifiesItsOwnOutput`,
`TestSupplyEngraveTailCutsAPlatePerMatchedSlot`,
`TestSupplyFlowEngravesAPlatePerMatchedSlot`,
`TestSupplyFlowAnnouncesWhatWillBeCut`. Zero-legs / zero-plates refusals
(`errVerifyNoLegs`, `errVerifyNoExpectedSlots`) unchanged and pinned.

`TestVerifyRefusesAnEmptyExpectation` was tightened while updating it: it now
passes a **real** md1 alongside the empty slot set, so it cannot pass on the
neighbouring empty-policy guard instead of the one it names.

---

## 4. Flow-level mutation checks

Each mutation left the condition in place and replaced only the consequence with
a `fmt.Println` marker, so the printed line **proves the site executed with the
triggering input** rather than merely proving the edit landed. All three
reverted; `grep -rn "MUTATION-PROBE" gui/` → `NO PROBES LEFT` before the commit.

**C1 — md1-binding removed** (`gui/multisig_verify.go:412`):

```
=== RUN   TestVerifyRefusesAMissingEngravedPolicy
--- PASS: TestVerifyRefusesAMissingEngravedPolicy (0.12s)
=== RUN   TestVerifyRefusesPlatesFromADifferentPolicy
MUTATION-PROBE-C1: md1-binding site REACHED; readback != engraved; refusal REMOVED
    multisig_verify_policy_test.go:125: final screen: "Operatorkeyandsecretverified.Othercosigners'keysaretakenassupplied.VerifyOK"
--- FAIL: TestVerifyRefusesPlatesFromADifferentPolicy (0.14s)
=== RUN   TestVerifyStillPassesItsOwnPolicy
--- PASS: TestVerifyStillPassesItsOwnPolicy (0.16s)
```

The marker printed → the `slices.Equal` line ran and evaluated **false** on a
differing md1; the suite went RED at flow level with the exact false GREEN. The
mechanism is what holds the test up.

**I1 — mk1 dedupe removed** (`gui/multisig_supply_tail.go:152`):

```
=== RUN   TestSupplyTailCollapsesByteIdenticalPlates
MUTATION-PROBE-I1: mk1 dedupe site REACHED; byte-identical plate; collapse REMOVED
    ... the tail's obligation list is [0 1], want [0]. ...
--- FAIL: TestSupplyTailCollapsesByteIdenticalPlates (0.05s)
=== RUN   TestSupplyDuplicateSlotVerifiesItsOwnOutput
MUTATION-PROBE-I1: ... collapse REMOVED
    ... final screen: "Readback1keyplate,butthisrunengraved2keyplates.Presentexactlytheplatesthisruncut.VerifyBundle"
--- FAIL: TestSupplyDuplicateSlotVerifiesItsOwnOutput (0.06s)
=== RUN   TestSupplyFlowAnnouncesTheCollapseBeforeTheFirstCut
MUTATION-PROBE-I1: ... collapse REMOVED
--- FAIL: TestSupplyFlowAnnouncesTheCollapseBeforeTheFirstCut (0.11s)
=== RUN   TestSupplyEngraveTailCutsAPlatePerMatchedSlot
--- PASS: TestSupplyEngraveTailCutsAPlatePerMatchedSlot (0.09s)
=== RUN   TestSupplyEngraveVerifiesItsOwnOutput
--- PASS: TestSupplyEngraveVerifiesItsOwnOutput (0.13s)
```

The two **reused-seed** tests stayed green under the mutation. That is the
measured proof of the ruling's scoping claim: the dedupe is inert for the shape
a previous block deleted it from, and load-bearing only for the identical-key
shape. It also proves the dedupe cannot be dropping a plate the reused-seed
operator needs.

**M2 — `break` reverted to `return`** (`gui/multisig_verify.go:454`):

```
=== RUN   TestVerifyReportsIncompleteAfterAMidLoopRefusal
MUTATION-PROBE-M2: mid-loop refusal site REACHED; break reverted to return
    multisig_verify_policy_test.go:269: final screen: "Thatseedisacosigner,butnoneofitsslotswereengravedinthisrun.Theplatesstilloutstandingbelongtoadifferentseed.VerifyBundle"
    multisig_verify_policy_test.go:274: the flow walked out of a PARTIAL verify with no report. ...
--- FAIL: TestVerifyReportsIncompleteAfterAMidLoopRefusal (0.05s)
```

The final screen under the mutation is the refusal itself, with two plates
verified and no verdict — the defect exactly as the reviewer traced it.

---

## 5. Gate — verbatim, unpiped, true exit codes

Run in `/scratch/code/shibboleth/wt-s5` under `nix develop`, each redirected to
a file with `$status` echoed separately (never judged through a pipe).

```
nix develop --command go test ./... -count=1
  TEST_EXIT=0
  ok=51  fail=0

nix develop --command gofmt -l ./
  GOFMT_EXIT=0  lines=0

env GOCACHE=<fresh empty dir> nix develop --command go vet ./...
  VET_EXIT=1
  vet_findings=40
  findings outside _test.go: 0
```

`go vet` exit 1 / 40 findings / 0 outside `_test.go` **is** the pinned clean
baseline. GOCACHE was a freshly created empty directory for that run, so the
count is not a cached result. First line of the vet output for identification:

```
gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file is go1.25)
```

Baseline at `853534a` was independently re-measured by me before touching
anything: `go test ./... -count=1` exit 0, 51 ok / 0 FAIL, `gofmt -l ./` empty.
Identical after the fold — the fold adds no package and no skip.

Commit **`e7da39a`** on `s5-multislot`, 8 files, +842/−24, gate output in the
message. `git status --porcelain` empty at finish.

---

## 6. M1 — filed, NOT fixed (as directed)

Not fixed, per the brief. It is pre-existing misattribution, unchanged by this
seam and not introduced by it: `gui/multisig_verify.go:479-481` reports a
passphrase divergence between engrave and verify as *"That seed is not a
cosigner of the read-back policy"*, blaming the seed. The engrave accepts a
payload-borne passphrase (`syswPassphraseFlow`, `gui/multisig.go:147`); the
verify requires it re-typed (`passphraseFlow`, deliberate per §7.4); a correct
seed with a mistyped passphrase makes `allUserSlots` empty, and the operator is
told their wallet is wrong.

**I could not file it into `design/FOLLOWUPS.md` myself** — the brief confines my
edits to the worktree, and `FOLLOWUPS.md` lives in the `mnemonic-engrave` repo
the controller owns. Ready to paste, following F-189/F-190's form:

> ### F-191 — a passphrase divergence between engrave and verify is reported as "That seed is not a cosigner" (owning phase: **`SPEC_multisig_build_repair.md` S5.D** — with the screens/prose block) `#seedhammer`
>
> Filed 2026-08-16 by the S5 policy-identity fold implementer, which could not
> write this file from its worktree. Found by the fable seam review (M1).
>
> `gui/multisig_verify.go:479-481`. The engrave accepts a payload-borne
> passphrase (`syswPassphraseFlow`, `gui/multisig.go:147`); the verify requires
> it re-typed (`passphraseFlow`, deliberate per §7.4). A CORRECT seed with a
> forgotten or mistyped passphrase makes `allUserSlots` return empty, so the
> flow names the SEED and never mentions the passphrase — on plates that are
> perfectly good.
>
> **It is a false RED that teaches the operator the wrong lesson.** "My seed
> isn't in my wallet" is the most alarming sentence this device can say, and
> here it is caused by a keystroke. The device knows a passphrase was offered
> and can say so; distinguishing "no slot matches with this passphrase" from
> "no slot matches at all" costs one re-derivation with the empty passphrase.
>
> Pre-existing in kind (the pre-S5 verify had the same shape through
> `findUserSlot`) and unchanged by the S5 seam, so it did not gate the fold.

---

## 7. What I could not do, and what I assumed

- **`FOLLOWUPS.md` not written** — see §6. The entry is above verbatim and needs
  the controller to land it.
- **No emulator walk was run**, per the out-of-scope list. `cmd/emu`'s Go tests
  (needle uniqueness, walk-needle binding) are inside the 51-package suite and
  are green; the JS walks themselves were not driven.
- **Screens/prose block, gate-record mint, `oracle/**`, `buildEngraveTail`, the
  BUILD path's engrave behaviour** untouched, per the out-of-scope list.
- **One assumption, checked rather than believed:** that an honest readback of
  this run's own md1 reproduces the engraved chunk strings **exactly**, which is
  what makes `slices.Equal` the right comparison rather than a decoded-field
  one. Checked by execution, not by reading the encoder — four honest-path flow
  tests that gather the md1 through the real payload seam and back
  (`TestSupplyEngraveVerifiesItsOwnOutput`,
  `TestVerifyOneSlotRunChecksTheONEPlateItEngraved`,
  `TestVerifyStillPassesItsOwnPolicy`, `TestSupplyDuplicateSlotVerifiesItsOwn
  Output`) all reach `Verify OK` with the equality live. Had the gatherer
  re-chunked or reordered, every one of them would be RED.
- **One judgement call the brief left open:** the collapse NOTE is *prepended* to
  the census rather than appended, because `confirmReviewScreen`
  (`gui/multisig_build.go:1405`) lets the operator confirm from **any** page —
  an appended note is one the operator can commit past without ever drawing it.
  The notice at step (4a) carries the fuller explanation; the census carries the
  exact, tail-derived number.
- **A fourth `return` in the same loop was left alone**, deliberately and stated
  here rather than silently: `gui/multisig_verify.go:459` (the `ferr != nil` arm)
  and the derive-error arm below it still `return`. The reviewer named three, and
  the `ferr` arm is unreachable given the entry guard (`expectedSlots` is
  non-empty by then). If a later reader wants total consistency it is a one-line
  change, but it would be a change with no reachable behaviour behind it, which
  is worse than the inconsistency.
