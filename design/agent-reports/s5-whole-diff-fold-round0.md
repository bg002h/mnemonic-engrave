# S5 whole-diff review — FOLD (round 0)

**Responding to:** `design/agent-reports/s5-whole-diff-review-round0.md` (3 Critical / 14
Important), plus follow-up **F-189** and the operator's **I-8** decision
(`design/agent-reports/s5-i8-seed-residency-decision.md`).
**Branch:** `s5-multislot` in `/scratch/code/shibboleth/wt-s5`.
**Base:** `7da66bd` — untouched, as are the ten commits beneath it.
**Date:** 2026-08-16.

| | count |
| --- | --- |
| Critical FIXED | **3 / 3** |
| Important FIXED | **14 / 14** |
| F-189 | **DONE** (both symbols deleted) |
| NOT FIXED | 0 |
| DISPUTED | 0 findings; **2 prescribed fixes departed from** (I-6, I-8) — see below |

**Seven commits**, none of them amending or rebasing anything at or below `7da66bd`:

```
830aaf7 S5 fold: drop a vestigial `|| true` from the full-mode verify driver
7dc49be S5 fold (I-7, I-9): the origin announcement, and the RE-MINTED Trace B record
7a23bb5 S5 fold (I-5, I-1): per-seed passphrases, and the `both` arm nothing watched
a9fface S5 fold (I-6, I-8, I-10, F-189): reachability, a decision, and a retired API
9f93362 S5 fold (C-1, I-2, I-3, I-4, I-11, I-12, I-13, I-14): the verify's REPORT
5f54737 S5 fold (C-3): the SUPPLY path says what a passphrase build leaves out
4eb34ec S5 fold (C-2): the gate groups on the MASTER, not on a per-slot surrogate id
```

---

## Two prescribed fixes did NOT survive contact with the code

Both are recorded here because the review's *findings* reproduced and its *fixes* did not,
and a re-reviewer should not re-derive either.

### I-6's minimal fix would have MOVED the dead end, not closed it — measured

The review says to add `if open == 0 { return cosignerAutoFill }` to
`classifyCosignerSupply`. That alone lets the flow through to `bundleGatherFlow`, which
needs one complete card to proceed and answers **Done on zero cards** with *"No complete
cards. Pack them on the host with `me sysw pack` and load the payload again."* — the same
instruction, the same abandon, one screen later.

So the guard is added **and** `buildMultisigPolicyFlow` skips the payload machinery entirely
when it demands nothing. **Both halves are mutation-proven separately**, and the second
mutation (`if open > 0` → `if true`, i.e. the review's fix standing alone) is what shows the
flow-level skip is load-bearing rather than tidying.

### I-8's minimal fix (a) would have introduced a Critical — confirmed against the tree

The review's option (a) says to scrub each seed "as soon as its key is derived in
`buildSelfKeys`". `buildSelfKeys` runs at `gui/multisig_build.go` step (4b); `buildEngraveTail`
runs at step (9) and **re-reads every registered seed** (`seed, ok := reg.at(s.SeedID)` then
`deriveMultisigLeg(seed.Mnemonic, …)`, `gui/multisig_build_tail.go:103-107`). Scrubbing there
breaks every engrave — and worse, a zeroed `bip39.Mnemonic` reads back as
`"abandon abandon …"` and BIP-39 derivation is checksum-free PBKDF2, so a read-after-scrub
would **silently derive real keys from the all-abandon wallet on a funds path**.

The operator ruled **(b) ACCEPT AND DOCUMENT** and supplied the replacement text; both
strings were applied verbatim.

---

# Per finding

## C-1 — "Checked N of the M key plates" asserted a comparison that never ran

**FIXED** — `9f93362`.

`verifyMultisigLegs` is split. `verifyMultisigLegsPartial` is the forward half of the
bijection (every leg finds its plate; that plate passes the full `bundle.Verify`) and returns
which plates it claimed; `verifyMultisigLegs` is that plus the unclaimed sweep. The incomplete
branch now runs the partial comparator **before** it reports, and only the reverse sweep is
skipped — a run that stopped early legitimately has plates no leg was ever built for, and
refusing over those would report a failure for an operator who did nothing wrong. A leg whose
plate fails is a **Verify Failed**, not an Incomplete. `multisigVerifyIncompleteText` takes
the matched slots and the outstanding ones and names both.

**Pinned by** `TestVerifyIncompleteDoesNotCallAForeignPlateChecked` (flow level: Trace B, the
@0 plate replaced by a foreign single-sig mk1 at `m/44'`, master A typed, **STOP HERE**), with
`TestVerifyIncompleteReportsWhatTheComparatorMatched` as the non-vacuity arm so "fail on every
incomplete verify" cannot satisfy it.

**Evidence it can fail** — mutation: incomplete branch put back in front of the comparator.

```
MUTANT[C1] EXIT=1
  final screen: "Checked2keyplates:@0and@1.Comparedagainsttheplatesyoupresented,
                 andtheymatch.1slotisNOTverified:@2. …"
  a readback whose @0 plate belongs to ANOTHER WALLET was reported as an INCOMPLETE
  verify, i.e. as two plates checked and one outstanding.
```

Post-fix the same input reads
`"Noread-backkeyplatecarriesslot@0'skey. … VerifyFailed"`.

**Beyond the minimal fix:** the incomplete text no longer prescribes "Run verify again with
the remaining seeds" as a bare instruction — it names the **VERIFY AGAIN** row I-4 created, so
the sentence refers to a button that exists.

---

## C-2 — the gate grouped on `SeedID` while the flow mints one per held slot

**FIXED** — `4eb34ec`.

`buildSlotGate`'s binding map is re-keyed on `registeredSeed.MasterFP`. The `slotFromSeed` arm
now resolves the seed (it previously only existence-checked it), and the notice's label travels
with the binding since a group no longer has a single seedID to look up. MasterFP is the
fingerprint of the *(seed, passphrase)* **pair**, so the same words under two passphrases stay
two masters (SPEC 4.1), and it fails **safe**: a 4-byte collision emits a spurious notice about
unrelated seeds, never suppresses a true one.

**Fixtures rebuilt BEFORE the production line**, per the review's point 3.
`TestGateAcceptsSameSeedAtDistinctOrigins` hand-built two `slotSource`s sharing `SeedID: 0` —
a registry the flow cannot construct — and was the only thing pinning the mechanism. It now
builds its registry with **two `reg.add()` calls of one mnemonic** and takes its sources from
`buildSlotSources`. Its same-origin negative arm stays hand-built and **says so**, since
`buildSlotSources` cannot produce it.

**Pinned at FLOW level too**, because a unit test of the gate is structurally blind to this
defect: `TestBuildFlowAnnouncesTwoSlotsFromOneSeed` drives pickers → payload → two seed entries
of the same words → Key sources, and pages the review for the notice.

**Evidence it can fail** — mutation: grouping key back to `bound[uint32(s.SeedID)]`.

```
MUTANT[C2 unit] EXIT=1   "got 0 notice(s) [], want exactly 1"
MUTANT[C2 flow] EXIT=1   "the Key-sources review NEVER says the two held slots come
                          from ONE seed."
  Pages drawn (mutant): "KeysourcesWhereeachkeycomesfrom:@0yours:derivedfromyourseedfor@0
   @1yours:derivedfromyourseedfor@1,account1@2acosigner:payloadcard1,takenassupplied
   Noslotclaimstobebothaseedandacardhere…"   <- verbatim the walk record's screen
```

And the notice is now in the **committed gate record** (see I-7/I-9):
`"Slots@0and@1allcomefromyourseedfor@0,atdifferentkeyorigins.Thatisamulti-accountwalletandisallowed."`

---

## C-3 — the SUPPLY path labelled a passphrase build "Full (seed + keys)" and said nothing

**FIXED** — `5f54737`.

Two lines in `gui/multisig.go`: the engrave-mode row 0 is
`buildFullModeLabel(passphrase != "")`, and the restore doc gets
`buildPlateInventoryLines(cardsOut, …)`. The second also closes review **M-13** (the supply
restore doc carried no plate inventory at all), which F-188 made bite. The false premise in
`multisigRestoreDocFlow`'s doc comment ("the supply path has no set of its own and passes nil")
is replaced rather than left to mislead.

**Pinned by** a new flow-level walk that **cuts every plate**, because both halves are
post-decision surfaces — the label is read before the press, the restore doc printed after the
last plate. A helper-level check of either proves only that the string exists somewhere, which
it already did: `buildFullModeLabel(true)` returned the correct sentence and was unreachable
from this flow. `TestSupplyPassphraseRunTellsTheOperatorWhatIsMissing` takes the passphrase
from a payload `ClassPassphrase` record, asserts the fixture policy matches **only with** it
and matches nothing without it, cuts 7 plates and pages the restore document.

**Evidence it can fail** — the mutation here is the frozen tree itself (TDD; the test was
written and run RED before the two lines landed):

```
EXIT=1, three assertions, one per production edit:
  "the engrave-mode picker calls a PASSPHRASE build \"Full (seed + keys)\""
     screen: "Whattoengrave?Full(seed+keys)Watch-only(keys)EngraveMode"
  "the supply path's restore document never mentions the passphrase"
  "the supply path's restore document carries no plate inventory"
Post-fix: "Whattoengrave?Full(seed+keys,NOTpassphrase)Watch-only(keys)EngraveMode"
```

**Beyond the minimal fix:** `TestSupplyRestoreDocSaysSoWhenNoPassphraseWasUsed` is added as the
non-vacuity arm — "always print the passphrase warning" would otherwise satisfy the test above,
and a picker that cries DEFAULT when the operator chose is one whose warnings get ignored.

**Adjacent, NOT folded (out of scope, filed below):** `gui/singlesig.go:80` carries the same
hard-coded literal.

---

## I-1 — a `both` slot's engraved origin was unpinned

**FIXED (test only)** — `7a23bb5`. **No production change** — the shipped line is correct.

`gui/multisig_build_tail_both_test.go` drives the gate-test file's already-proven A@1 fixture
through `buildSlotSources` → `buildSelfKeys` → `buildSlotGate` (asserting it **PROCEEDS**, so
the tail arm is reachable rather than hypothetical) → `assembleBuildPolicy` →
`buildEngraveTail`, then decodes the @0 leg's mk1 and asserts **both** that its origin is the
card's declared path **and** that its key bytes are the ones the assembled policy holds at @0.
The second assertion is what makes the mutation lose funds rather than merely mislabel a plate.

**Evidence it can fail** — mutation M14, `origin = o` → `origin = derivedSlotOrigin(script, s.Account)`:

```
MUTANT[I1] EXIT=1
  "the @0 leg's mk1 declares origin m/48h/0h/0h/2h, but the card the operator
   asserted is theirs declares m/48h/0h/1h/2h"
  "the @0 leg's mk1 carries a key the policy does NOT hold at @0"
```

and, under the **same** mutation, the pre-existing tail/verify/gate tests still pass (**0
failures**) — reproducing the review's `M14 SURVIVED(green)` and showing the new test is the
only thing holding the line.

---

## I-2 — the whole `full` half of `multisigVerifyFlow` was executed by no test

**FIXED** — `9f93362`. No production change was required for arms (1) and (2).

Three new flow-level drivers, all with `full=true` over Trace B's two-master shape:

1. `TestVerifyFullModeTwoSeedsReportsTheFullSuccess` — types both seeds and both ms1 shares;
   asserts the **full-mode multi-leg success string** (`"and the ms1 you typed"`,
   `"All 3 operator key plates"`), which had zero executions in the coverage profile.
2. `TestVerifyFullModeBackAtTheSecondMs1ReportsIncomplete` — Back at the second seed's
   "Type ms1"; asserts **Verify Incomplete** naming @2.
3. `TestVerifyFullModeBindsEachMs1ToItsOwnSeed` — master A's legs against **master B's** ms1;
   asserts **Verify Failed** and that the screen carries the word `ms1`.

The fixture measures its own premise: Trace B must cut **two distinct** ms1 plates, or a
per-seed binding could pass by accident.

**Evidence it can fail** — both mutations the review named:

```
MUTANT[I2-M25] ms1 readback blanked at the assignment -> EXIT=1
  "a FULL-mode verify … did not pass. Final screen: … slot@0:verify:ms1presencemismatch"
  (i.e. every full-mode verify would fail, which is what nothing noticed)
MUTANT[I2-M19] ms1-entry `break` reverted to `return` -> EXIT=1
  "Back at the SECOND seed's ms1 entry walked out of a partial verify with no report.
   Final screen: \"\""
```

**Per the review's instruction, no time was spent on the refuted hoist scenario.**

---

## I-3 — the failure screen discarded every diagnosis

**FIXED** — `9f93362`.

`multisigVerifyFailureText(err)` type-switches on `errVerifyLegHasNoPlate` (names the slot —
its entire reason to carry one) and `errVerifyPlateUnclaimed` (names the plate), and otherwise
appends the comparator's own wrapped message. Both `Verify Failed` sites use it.

**Pinned by** `TestVerifyFailureTextNamesWhatTheComparatorFound` (three causes, pairwise
distinct, each fitting the modal), **and at screen level** by C-1's and I-2's flow tests: the
slot number and the word `ms1` are asserted on the drawn frame, not on the error object.

**Evidence it can fail** — mutation: the generic string restored.

```
MUTANT[I3] EXIT=1
  final screen: "Theread-backbundledoesNOTmatchtheseed.Checktheengravedplates.VerifyFailed"
  "the failure screen does not name the slot whose plate is missing"
```

---

## I-4 — "run verify again" prescribed a remedy that did not exist

**FIXED, option (a)** — `9f93362`, landed together with I-12 as the review required.

`multisigVerifyFlow` returns `multisigVerifyResult` (complete / incomplete / failed / refused /
abandoned). Both engraving callers loop the offer: anything short of `verifyComplete` that the
operator can act on (**incomplete** or **failed**) re-offers under
`multisigVerifyRetryLead`; a structural **refusal** or an **abandon** does not loop, because
neither changes by trying again with the same inputs.

The loops are **inline at both call sites** rather than extracted, deliberately: that keeps
`multisigVerifyFlow(ctx, th, full, engravedSlots, engraveMd1)` textually present in each flow's
body, so the two shipped wiring guards (`TestBuildPassesTheTailsSlotsToTheVerify`,
`TestSupplyPassesTheEngravedPolicyToTheVerify`) keep pinning the obligation's provenance.

**Pinned by** `TestBothEngraveFlowsReOfferTheVerify` (asserts the verdict is *read*, that the
loop condition exists, and that the retry lead states why it is being offered).

**Evidence it can fail** — mutation: one-shot offer restored on the supply path.

```
MUTANT[I4] EXIT=1
  "multisig.go discards the verify's verdict, so it cannot tell a clean pass from an
   incomplete or a failed one"
  "multisig.go does not re-offer the verify after an incomplete or a failed attempt"
```

---

## I-5 — N per-seed passphrases collapsed into one boolean

**FIXED** — `7a23bb5`.

`seedRegistry.passphraseFacts()` returns `[]seedPassphraseFact{Label, MasterFP, Uses}` — **no
mnemonic and no passphrase text cross into the display path**. `buildPassphraseInventoryLines`
takes that slice and, with more than one seed registered, emits one line per **passphrased**
seed naming its label and fingerprint, **and one per bare seed**: silence about the bare ones
reads as "all of them need it", which sends a reader hunting a passphrase that never existed
and lets them conclude the one they hold is the only factor missing.

`usesPassphrase()` **survives** for the engrave-mode label — a different question with a
genuinely boolean answer, on a row that does not wrap.

**Pinned by** `TestRestoreDocNamesEveryPassphrasedSeed` (asserts **two distinct** passphrase
statements, both labels, both fingerprints, and the "may be DIFFERENT" warning),
`TestRestoreDocSaysWhichSeedsNeedNoPassphrase` (the mixed two-master case), and
`TestSingleSeedInventoryIsUnchanged` (the regression floor — the common build must read exactly
as it always did).

**Evidence it can fail** — mutation: the enumeration short-circuited back to the two shipped
lines.

```
MUTANT[I5] EXIT=1
  "the restore document carries 0 per-seed passphrase statement(s), want 2"
  "the document does not name exactly the ONE seed that needs a passphrase"
```

**Beyond the minimal fix:** a zero fingerprint is never rendered as `00000000`
(`seedFingerprintSuffix`), and the supply path's single fact is built by a named
`oneSeedPassphraseFact` rather than a struct literal carrying a meaningless zero.

---

## I-6 — holding EVERY slot dead-ended on a self-contradictory refusal

**FIXED, with a departure from the prescribed fix** — `a9fface`. See the departures section.

`classifyCosignerSupply` returns `cosignerAutoFill` when `open == 0`, **and**
`buildMultisigPolicyFlow` skips the payload machinery entirely for a zero demand.

**Pinned by** `TestZeroDemandBuildIsNotRefusedForAPayloadItDoesNotNeed` (the `open == 0` row
across all three payload states, plus a non-vacuity block keeping under-supply a refusal) and
`TestBuildHoldingEverySlotReachesTheSeed` (flow level: a 2-of-2 the operator holds entirely,
no payload at all, asserting the flow reaches seed entry for **both** @0 and @1).

**Evidence it can fail** — two mutations:

```
MUTANT[I6-classify] `open == 0` arm removed -> EXIT=1
  "a build needing NO cosigner cards was refused for want of a cosigner-card payload
   (state=1 have=0 open=0)"
MUTANT[I6-skip-gather] `if open > 0` -> `if true` (the review's fix, ALONE) -> EXIT=1
  "a build holding EVERY slot never reached seed entry"
```

The pre-fix RED reproduced the review's screen verbatim:
`"Nopayloadisloaded,andthispolicyneedsnocosignerkeycards.Thisdevicehasnocardreader:packthecardsonthehostwith`me sysw pack`,loadthepayload,thenbuild."`

**Beyond the minimal fix:** the stale `@S`-picker comment at `gui/multisig_build.go:46` is
corrected (the review's own I-6 fix asks for this), **and so is its twin** at
`gui/multisig_build_slots.go:40-44`. That second one is M-4/M-15, a Minor — folded because it
sits directly above the gate C-2 re-keyed and *explicitly licensed the hand-built-fixture
practice that made C-2 invisible*, so leaving it would have had the file contradict a Critical
fix landed two commits earlier.

---

## I-7 — the §0.1a origin announcement stated ONE origin for a multi-account build

**FIXED** — `7dc49be`.

`buildOriginAnnouncement(script, held []heldSlotOrigin)` is a function of the origins actually
used. They come **off the assembled md1** — `buildSlotKeyStrings` already expands it and now
returns the per-slot origins from that same expansion — rather than from a second walk of the
slot sources: re-deriving them would be a second copy of the tail's origin rule, and two copies
of a rule agree until one is edited. Held slots sharing one origin keep the shipped scalar
sentence; divergent ones are enumerated.

**Pinned at FLOW level** inside `TestBuildFlowAnnouncesTwoSlotsFromOneSeed`, which now
continues past Key sources to the **Policy Review** and pages it for `m/48h/0h/1h/2h`.

**Evidence it can fail** — mutation: back to `derivedSlotOrigin(script, 0)`.

```
MUTANT[I7] EXIT=1
  "the Policy Review NEVER states @1's origin (m/48h/0h/1h/2h)."
```

**And in production**, from the re-minted record's `reviewScreen`:

```
Yourkeyorigins:@0atm/48h/0h/0h/2h,@1atm/48h/0h/1h/2hand@2atm/48h/0h/0h/2h,
theBIP-48pathfornativesegwit.
```

---

## I-8 — the seed-registry justification was false and its re-decision was never made

**FIXED — the decision is MADE IN WRITING in this diff** — `a9fface`.

Operator ruling (`design/agent-reports/s5-i8-seed-residency-decision.md`): **(b) ACCEPT AND
DOCUMENT**. No bound, no scrub change. Both replacement strings applied **verbatim**: the
"WHY NOT AN IDLE LIMIT" justification at `gui/multisig_build_census.go`, and the operator-facing
"Seed handling" ruling in `buildPlateInventoryLines`.

The two things the coordinator required not to survive **do not survive**, and the check is a
grep: `grep -rn "holds exactly one seed\|A seed you entered" gui/` returns nothing.

**Pinned by** `TestSeedResidencyRulingDescribesTheMultiSeedReality`, which asserts the plural
ruling, the two facts the decision turns on (`on the plates`, `unattended`), the absence of the
stale singular, **and** that the source no longer carries the falsified premise while it does
carry the recorded re-decision.

**Evidence it can fail** — mutation: the singular ruling restored.

```
MUTANT[I8] EXIT=1, four assertions:
  "the ruling still describes a registry holding ONE seed"
  "the ruling does not say the machine holds EVERY seed entered"
  "the ruling does not mention \"on the plates\"" / "\"unattended\""
```

**Departure:** the review's own option (a) is not implemented, and would have been a Critical.
See the departures section above.

---

## I-9 — S5 had no analogue of `TestS0GateHasARecord`

**FIXED** — `7dc49be`, landed **in the same commit as the re-mint**, as the review required.

`TestS5GateHasARecord` sits beside S0's, and both are now expressions of a table-driven
`TestEveryRequiredStageHasAGateRecord` over a `requiredStages` list, so the next stage cannot
forget. Neither skips — not under `-short`, not without the oracle binaries, not in CI.

**Evidence it can fail** — mutation: all four `S5-trace-b.*` files deleted (the review's own
reproduction).

```
MUTANT[I9] EXIT=1
  --- FAIL: TestEveryRequiredStageHasAGateRecord
      S5 has no gate record in gaterecords (stages present: [S0]).
  --- FAIL: TestS5GateHasARecord
Files restored; `git status --porcelain oracle/gaterecords/` empty.
```

---

## I-10 — the "filed rather than smuggled in" claim did not check out

**FIXED (part 2; part 1 was already landed by the coordinator)** — `a9fface`.

The comment at `gui/multisig_build_slots.go` now cites **F-196** by ID, with its owning phase,
so the claim is a grep rather than a promise.

**The ID was VERIFIED, not trusted:**

```
$ grep -n "F-196" /scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md
6979:### F-196 — a MIXED held set is not expressible through the screens
      (owning phase: **the spec — it is a model change, and earns its own R0**) `#seedhammer`
```

---

## I-11 — the abort screen promised a resume the device cannot deliver

**FIXED** — `9f93362`.

`bundleAbortWarningText` keeps the TRUE half (a re-run mints byte-identical plates, which is
what makes starting over safe) and replaces the false half — *"so you only cut the ones you are
missing"* — with what the device does: a re-run starts at plate 1, finish the set in one
sitting or start over. A seed-bearing set is additionally warned that **a re-run RE-CUTS the
seed plate**, which is the one consequence owed to an operator who has just been told to
DESTROY rather than bin a plate.

**A shipped test pinned the false promise and was rewritten, not deleted.**
`TestAbortWarningTellsTheOperatorHowToFinishTheSet` *required* the string
`"missing"`; it is now `TestAbortWarningPromisesOnlyWhatTheDeviceCanDo`, asserting the opposite
direction. `TestBundleEngraveHasNoResumeMechanism` is added as the **departure run-check**: it
reads the shipped source and fails if `bundleEngrave` ever grows a start index, a resume
parameter, or an "already cut" route — so a future resume forces the text to be rewritten with
it, which is the coupling that went missing the first time.

**Evidence it can fail** — mutation: the false promise restored.

```
MUTANT[I11] EXIT=1
  "the abort does not say a re-run RESTARTS the set"
  "the abort still promises a partial re-cut (\"only cut the ones\")"
  "the abort still promises a partial re-cut (\"you are missing\")"
```

*(A first attempt at this mutation did not apply — the `perl` pattern missed — and the run
returned exit 0. That proved nothing and is recorded rather than hidden; the mutation was
re-applied with an asserted substitution count and then killed the test.)*

---

## I-12 — an abort did not propagate

**FIXED** — `9f93362`, landed with I-4.

`bundleEngrave` returns `bundleEngraveResult`; both abort paths return `bundleEngraveAborted`.
Both multisig callers gate on it, so an aborted set never reaches the verify offer (which
cannot succeed — the md1 is emitted last) or a restore document headed *"This backup is N
plates … If any of them is missing, this backup is incomplete."* The false premise at
`gui/bundle_flow.go:481-483` is deleted; the sentence it made is now **true**, and the abort
text says so.

**Pinned by** `TestBothEngraveFlowsGateOnACompletedSet` (call-site level, stated as a
limitation) **and** `TestSupplyAbortIsTheLastScreenOfTheProgram`, which drives the real screens
to the first engrave picker, presses Back, and asserts the program **ends** with none of
`"Verify the engraved plates?"`, `"This backup is"`, `"Descriptor:"` drawn afterwards.

**Evidence it can fail** — mutation: the gate removed at both call sites.

```
MUTANT[I12] EXIT=1
  --- FAIL: TestBothEngraveFlowsGateOnACompletedSet
  --- FAIL: TestSupplyAbortIsTheLastScreenOfTheProgram      <- the FLOW-level one
```

---

## I-13 — a watch-only verify claimed "secret verified"

**FIXED** — `9f93362`. `multisigVerifyOKMessage`'s single-leg arm is `full`-aware.

**Pinned by** `TestVerifyOKMessageClaimsASecretOnlyInFullMode` over
`(legs ∈ {1,3}) × (full ∈ {false,true})`.

**One correction to the review's prescribed assertion.** It says to assert the word `"secret"`
appears **iff** `full`. That is wrong for the shipped **multi-leg full** message, which claims
the secret in different words (`"and the ms1 you typed for each seed"`) and contains no
`"secret"` — so a bare `Contains("secret")` would have failed a correct string. The predicate
is `"secret" OR "ms1"`, stated in the test as *claims a secret was checked*. The single-leg
full arm is additionally asserted to be the pinned constant, byte-unchanged.

**Evidence it can fail** — mutation: the `!full` arm removed.

```
MUTANT[I13] EXIT=1
  multisigVerifyOKMessage(1, false) = "Operator key and secret verified. …"
  claims a secret: true, want false
```

---

## I-14 — "the plates still outstanding belong to a different seed"

**FIXED** — `9f93362`.

`multisigVerifyCoveredSeedBody(engravedNone, bareWordsMatch)` replaces both inline arms.
Neither asserts a foreign seed: they say *"different words, or these words with a different
BIP-39 passphrase"*. `multisigVerifySeedIsInnocent` is wired into them for the one case it can
settle outright, so the operator gets a concrete action instead of a guess.

**Pinned by** `TestVerifyCoveredSeedBodyDoesNotAssertAForeignSeed` (all four cells; each fits
the modal; the arm that can name the passphrase does; the two engravedNone arms stay
distinguishable).

**Evidence it can fail** — mutation: the old sentence restored.

```
MUTANT[I14] EXIT=1
  "already covered: still asserts a foreign seed, which the device cannot know and
   which a one-character passphrase divergence produces"
```

**Beyond the minimal fix (necessarily):** `s5DriveVerifyTwoSeeds` dismissed its modal on the
removed phrase, so its needles moved with it; and
`TestVerifyReportsIncompleteAfterAMidLoopRefusal`'s `"Checked 2 of the 3"` assertion became
`"Checked 2 key plates"` plus per-slot naming, which is C-1's stronger claim.

---

## F-189 — the retired API

**DONE — both deleted** — `a9fface`.

*Measured before deleting, not taken from the follow-up:*

* `multisigEngraveCards` — its only non-test reference was its own definition. **Deleted.**
  `TestMultisigEngraveCards` now asserts the same one-of-each **shape** against
  `multisigEngraveCardsMulti`, the emitter an operator's plates actually come out of, so the
  property is re-pointed rather than lost.
* `findUserSlot`'s `reused` return — its one production caller
  (`gui/multisig_build_slots.go`) discarded it as `_`. **Deleted**; the signature is now
  `(int, bip32.Path, bool)`. It carried F-188's retired "this key is reused" claim, which was
  **false** — those slots hold different keys at different origins. The tests that asserted on
  it now assert on `allUserSlots`, which production actually calls.

Both deletions are **compile-enforced**: the symbol and the return value no longer exist, so a
caller cannot reappear silently.

---

# The build gate

Run on the final tree, each judged on its **true exit code**, redirected to a file with the
status echoed before any `grep`:

```
$ nix develop --command go test ./... -count=1
EXIT=0        51 ok, 0 FAIL

$ nix develop --command gofmt -l ./
EXIT=0        (empty output)

$ env GOCACHE=$(mktemp -d) nix develop --command go vet ./...
EXIT=1        40 findings, 0 of them outside _test.go
              (this IS the clean baseline here; it requires a COLD GOCACHE)

$ nix develop --command ./scripts/oracle-live.sh
EXIT=0        live checks: PASS; "discovered 7 tagged test(s) from source", 7 --- PASS

$ nix develop --command ./cmd/emu/build.sh
EXIT=0        built emu.wasm (9972075 bytes)
```

Matches the established baseline in all five.

---

# The re-mint

`oracle/gaterecords/S5-trace-b.{record,expect,walk}.json` were **re-minted**, not edited.

* `cmd/emu` rebuilt; served on a **fresh port** (8791); the walk driven in a real browser
  fire-and-forget. **462 s, 17 plates, 17 per-plate digests, `ok: true`.**
* `go run ./cmd/gaterecord -stage S5 -walk … -inputs … -base S5-trace-b -force` → **exit 0**;
  all three pinned oracles resolved by binary-sha256 at their pinned commits
  (`md 0.13.0`, `mk 0.13.0`, `ms 0.16.0`, `version_matches_pin: true`).
* `go test ./oracle/... ./cmd/emu/ -count=1` → exit 0.

**The walk now ASSERTS instead of tapping past.** Two additions to
`cmd/emu/walk_trace_b.js`, both of which **throw** rather than recording a flag — a flag lets a
walk mint a record that vouches for a sentence the device should not have drawn:

1. `ORIGINS_EXPECTED` — the Policy Review must state every origin this build derives at.
   Line 357 was `await tap(CONFIRM, 400); // past the Policy Review`, asserting nothing, which
   is how the wrong sentence was minted into a green record.
2. `claims.multiAccountNotice` — `claims.multiAccount` was the review's identified **false
   friend**: it greps the review text for `"account 1"`, proving only that the account counter
   diverged, never that C-2's notice fired.

**Proof the served wasm was fresh** is the walk's own output rather than a byte count: both new
assertions passed, and only the folded code produces those sentences. From the committed
record:

```
reviewScreen:      Yourkeyorigins:@0atm/48h/0h/0h/2h,@1atm/48h/0h/1h/2hand
                   @2atm/48h/0h/0h/2h,theBIP-48pathfornativesegwit.
keySourcesScreen:  Slots@0and@1allcomefromyourseedfor@0,atdifferentkeyorigins.
                   Thatisamulti-accountwalletandisallowed.
claims:            {multiAccount: true, cosignerSlot: true, nothingCrossChecked: true,
                    multiAccountNotice: true}
```

---

# NEW defects found and NOT folded

Both are in `gui/singlesig.go`, out of the review's scope, and are **the same two defects this
fold just closed on the multisig paths**. Reported rather than fixed, per the brief.

### N-1 (Important) — the single-sig flow has I-12's defect verbatim

`gui/singlesig.go:127` calls `bundleEngrave(ctx, th, "Engrave Single-Sig", cards)` and ignores
the result. An operator who aborts mid-set then reads *"Bundle Incomplete … This set is not a
usable backup yet"*, is offered **"Verify the engraved plates?"**, and is shown
`restoreDocFlow(...)`. The fix is now one line, because `bundleEngrave` already returns
`bundleEngraveResult`.

### N-2 (Minor) — the single-sig flow has C-3's hard-coded literal

`gui/singlesig.go:80` carries `Choices: []string{"Full (seed + keys)", "Watch-only (keys)"}`.
The review named this one itself ("Adjacent, out of scope, file it"). Whether it is a real
harm depends on whether that flow takes a BIP-39 passphrase into its derivation — **not
verified here**, and the follow-up below says so rather than asserting it.

---

# For `design/FOLLOWUPS.md` — I cannot write that file

Exact text requested, next free IDs after F-196:

```markdown
### F-197 — the SINGLE-SIG engrave does not stop on an aborted set (owning phase: **the next cycle's implementation phase**) `#seedhammer`

`gui/singlesig.go:127` calls `bundleEngrave(ctx, th, "Engrave Single-Sig", cards)` and
discards the result. It is I-12's defect verbatim, on the flow the S5 whole-diff review did
not scope: an operator who aborts mid-set reads "Bundle Incomplete ... This set is not a
usable backup yet", is then offered "Verify the engraved plates?" over a set whose last card
was never cut, and is finally shown `restoreDocFlow(...)` -- the artifact that is read years
later, alone, presented as the last word of a run the device just said produced no usable
backup.

Found during the S5 fold (2026-08-16) while landing I-12. NOT folded: out of the review's
scope, and scope creep in a fold is how a review round gets spent on unreviewed text.

The fix is now one line and the machinery already exists: `bundleEngrave` returns
`bundleEngraveResult` as of commit 9f93362, so this is

    if bundleEngrave(ctx, th, "Engrave Single-Sig", cards) != bundleEngraveDone {
        return
    }

It needs the flow-level test that goes with it -- see
`TestSupplyAbortIsTheLastScreenOfTheProgram` (gui/multisig_verify_report_test.go) for the
shape: drive to the first engrave picker, press Back, assert the program ENDS with neither
the verify offer nor the restore document drawn afterwards. A call-site assertion alone is
not enough; that is what let the multisig instance ship.

### F-198 — `gui/singlesig.go:80` hard-codes "Full (seed + keys)" (owning phase: **the next cycle's implementation phase**) `#seedhammer`

Named by the S5 whole-diff review itself as "adjacent, out of scope, file it" (C-3). S5 built
`buildFullModeLabel(passphrase bool)` because "Full (seed + keys)" is a LIE for a build with a
BIP-39 passphrase -- ms1 encodes the WORDS, the passphrase is a required spending factor and
is never engraved. The multisig BUILD path used it from the start and the multisig SUPPLY path
was wired to it in commit 5f54737; `gui/singlesig.go:80` still carries the raw literal.

NOT VERIFIED, and this entry deliberately does not claim it: whether this is a live harm
depends on whether the single-sig flow takes a passphrase into its own derivation. Check that
FIRST -- if it does, this is C-3's defect on a third path and is Critical, not a label tidy-up;
if it does not, the literal is correct and this closes as a no-op. Either way the answer
should be recorded, because "somebody assumed" is what made C-3 survive a whole stage.
```

---

# What a re-reviewer should NOT re-derive

* **R-1 stays refuted.** The worktree was clean at every checkpoint and is clean now.
* The machine baseline is unchanged and is quoted verbatim in the build-gate block above.
* Every mutation in this report was **run**, and each was restored afterwards with the tree
  re-verified. Where a mutation failed to apply, that is said (I-11).
* Two of the review's own prescribed fixes are departed from with reasons and measurements
  (I-6, I-8), and one prescribed assertion was corrected (I-13). The **findings** all
  reproduced; only those three prescriptions did not survive the code.
