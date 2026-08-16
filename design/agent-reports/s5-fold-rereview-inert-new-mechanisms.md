# S5 fold re-review — LENS: are the fold's NEW mechanisms INERT?

**Scope:** `git diff 7da66bd..830aaf7` in `/scratch/code/shibboleth/wt-s5` (7 commits). The
10 commits at and below `7da66bd` were NOT re-reviewed. `gui/singlesig.go` was not
investigated (F-197/F-198 already filed — verified present at
`design/FOLLOWUPS.md:7020` and `:7047`).

**Method:** the frozen worktree was never edited — `git status --porcelain` in
`/scratch/code/shibboleth/wt-s5` is empty at the end of this review. Every mutation was
applied to a `cp -a` copy under my scratch dir and run as
`nix develop --command go test ./gui/ -count=1` (whole package, not `-run`-scoped), plus
`./oracle/ ./cmd/emu/` for the gate-record mechanism. Baseline on the unmutated copy:
`ok seedhammer.com/gui 126.826s`, exit 0.

**Verdict: 2 Important, 2 Minor, 2 Nit. No Critical.** 22 mutations run; **17 went RED, 5
stayed GREEN**. Every Critical the fold claims to close (C-1, C-2, C-3) is pinned at FLOW
level and reproduced RED here. The two Importants below are both *unpinned fixes*: the
shipped code is correct, and the only thing standing between it and a silent regression is
a `strings.Contains` over the source text.

---

## The mutation matrix (all run; RED = at least one test failed)

| fold mechanism | mutation applied | result | which test caught it |
| --- | --- | --- | --- |
| C-1 partial comparator | the `verifyMultisigLegsPartial` call deleted from the incomplete branch | RED | `TestVerifyIncompleteDoesNotCallAForeignPlateChecked` (**flow**), `TestVerifyFullModeBindsEachMs1ToItsOwnSeed` |
| C-2 MasterFP grouping | grouping key + `order` reverted to `s.SeedID` | RED | `TestGateAcceptsSameSeedAtDistinctOrigins`, `TestBuildFlowAnnouncesTwoSlotsFromOneSeed` (**flow**) |
| C-3a supply mode label | `buildFullModeLabel(passphrase != "")` → the literal `"Full (seed + keys)"` | RED | `TestSupplyPassphraseRunTellsTheOperatorWhatIsMissing` (**flow**) |
| C-3b supply inventory | `buildPlateInventoryLines(cardsOut, oneSeedPassphraseFact(...))` → `nil` | RED | same (**flow**) |
| I-3 failure diagnosis | `multisigVerifyFailureText` short-circuited to the old generic string | RED | `TestVerifyFailureTextNamesWhatTheComparatorFound` + 2 **flow** tests |
| **I-4 retry loop** | `for {` → `for once := 0; once < 1; once++ {`, both call sites | **GREEN** | **nothing** — see Finding 1 |
| I-5a per-seed enumeration | `if len(seeds) < 2` → `if true` | RED | `TestRestoreDocNamesEveryPassphrasedSeed`, `TestRestoreDocSaysWhichSeedsNeedNoPassphrase` |
| I-5b bare-seed lines | the `for _, s := range bare` loop deleted | RED | `TestRestoreDocSaysWhichSeedsNeedNoPassphrase` |
| I-6a `open == 0` arm | arm deleted from `classifyCosignerSupply` | RED | `TestZeroDemandBuildIsNotRefusedForAPayloadItDoesNotNeed` |
| I-6b flow-level skip | `if open > 0` → `if true` (i.e. the review's own prescribed fix, standing alone) | RED | `TestBuildHoldingEverySlotReachesTheSeed` (**flow**) |
| I-7 origin enumeration | `heldOriginSummary(...)` → `derivedSlotOrigin(script, 0).String()` | RED | `TestBuildFlowAnnouncesTwoSlotsFromOneSeed` (**flow**) |
| I-8 residency ruling | singular "A seed you entered ..." restored | RED | `TestSeedResidencyRulingDescribesTheMultiSeedReality` |
| I-9 `requiredStages` | `oracle/gaterecords/S5-trace-b.*` deleted | RED | `TestEveryRequiredStageHasAGateRecord` **and** `TestS5GateHasARecord` |
| I-11 abort prose | the "only cut the ones you are missing" promise restored | RED | `TestAbortWarningPromisesOnlyWhatTheDeviceCanDo` |
| I-12 SUPPLY gate | `return` → no-op, the greped `if` line left intact | RED | `TestSupplyAbortIsTheLastScreenOfTheProgram` (**flow**) |
| **I-12 BUILD gate** | `return` → no-op, the greped `if` line left intact | **GREEN** | **nothing** — see Finding 2 |
| I-13 `!full` arm | the arm deleted from `multisigVerifyOKMessage` | RED | `TestVerifyOKMessageClaimsASecretOnlyInFullMode` (helper only) |
| **I-13 wiring** | `multisigVerifyOKMessage(len(legs), full)` → `(len(legs), true)` | **GREEN** | **nothing** — see Finding 3 |
| I-14 covered-seed body | the "belong to a different seed" sentence restored, `bareWordsMatch` disabled | RED | `TestVerifyCoveredSeedBodyDoesNotAssertAForeignSeed` (helper only) |
| C-1 refactor safety | the unclaimed sweep deleted from `verifyMultisigLegs` | RED | `TestVerifyCoversEveryLeg` |
| nit: zero-fingerprint guard | `if fp == 0` → `if false` | **GREEN** | Finding 5 |
| nit: `heldSlotOrigins` bounds guard | `if s < 0 || ...` → `if false` | **GREEN** | Finding 6 |

Corroborating machine evidence, from a full `go test ./gui/ -count=1 -coverprofile` run on
the unmutated copy (`coverage: 83.8% of statements`), parsed for zero-count blocks:

```
gui/multisig.go        332-333, 333-334, 336-337   count 0   <- the supply retry loop body
gui/multisig_build.go  402-404                     count 0   <- the BUILD abort gate's return
gui/multisig_build.go  448-449, 449-450, 452-453   count 0   <- the build retry loop body
gui/multisig_verify.go 648-653, 655-658, 847-849   count 0   <- verifyAbandoned / verifyRefused
```

---

# Findings

## 1. (Important) I-4's re-offer loop is pinned by a source-text grep only; no test ever runs a second verify attempt

**File:** `gui/multisig.go:325-338`, `gui/multisig_build.go:441-454`
**Pin:** `gui/multisig_verify_report_test.go:733` `TestBothEngraveFlowsReOfferTheVerify`

That test is four `strings.Contains` calls over `funcBody(t, file, fn)` — it reads the
source text of the two flows and asserts the substrings `res := multisigVerifyFlow(...)`,
`res != verifyIncomplete && res != verifyFailed` and `multisigVerifyRetryLead` are present.
It never executes the loop.

**Concrete failing input / mutation.** Change only the loop header at both sites, leaving
all four greped substrings byte-identical:

```go
lead, choices := "Verify the engraved plates?", []string{"Verify now", "Skip"}
for mutOnce := 0; mutOnce < 1; mutOnce++ {          // was: for {
```

This restores the exact pre-fold behaviour — a one-shot offer. Command I ran, on a `cp -a`
copy:

```
$ nix develop --command go test ./gui/ -count=1
ok  	seedhammer.com/gui	88.817s
EXIT=0
```

Independently confirmed by coverage: `gui/multisig.go:332-337` and
`gui/multisig_build.go:448-453` have execution count **0**, and
`grep -rn "VERIFY AGAIN" gui/*_test.go` returns nothing — no test in the package ever sees
the retry screen. `verifyAbandoned` and `verifyRefused` (`multisig_verify.go:648-653`,
`655-658`, `847-849`) are also never executed, so the very distinction that decides whether
to loop is unexercised at flow level.

**Why it matters here.** I-4's finding was *"'run verify again' prescribed a remedy that did
not exist"*. The fold's own new text now makes that prescription concrete —
`multisigVerifyIncompleteText` says *"Choose VERIFY AGAIN on the next screen and type the
remaining seed, or do not fund this wallet until you have."* If the loop regresses, that
sentence names a button the operator will never be shown, and a partially-verified 3-of-4
gets funded. The finding is not re-opened, but it is **moved**: from "no mechanism" to "a
mechanism nothing watches".

**Suggested fix, resolved against the real call graph before proposing it.** The shape
already exists and is proven: `TestSupplyAbortIsTheLastScreenOfTheProgram`
(`gui/multisig_verify_report_test.go:639`) drives `supplyMultisigPolicyFlow` through
`runUI`/`pumpUntil` to the engrave picker. Extending that driver past a completed engrave
(or reusing `s5DriveVerifyStopAfterOneSeed`'s STOP-HERE input at the flow level rather than
by calling `multisigVerifyFlow` directly) reaches the offer, and asserting that the string
`multisigVerifyRetryLead` is drawn after an incomplete verdict would kill the mutation
above. The supply path is the cheaper of the two to drive — it is the one already wired for
`ctx.syswBundleSeeds` in that file.

---

## 2. (Important) I-12's BUILD-path abort gate is inert; only the SUPPLY path is pinned behaviourally

**File:** `gui/multisig_build.go:402-404`
**Pin:** `gui/multisig_verify_report_test.go:607` `TestBothEngraveFlowsGateOnACompletedSet`

That test greps each flow's body for the literal
``if bundleEngrave(ctx, th, "Build Policy", cardsOut) != bundleEngraveDone {`` — the `if`
line, not the `return` inside it.

**Concrete failing input / mutation.** Keep the greped line, empty the body:

```go
if bundleEngrave(ctx, th, "Build Policy", cardsOut) != bundleEngraveDone {
	_ = cardsOut                                    // was: return
}
```

```
$ nix develop --command go test ./gui/ -count=1
ok  	seedhammer.com/gui	88.242s
EXIT=0
```

The same mutation applied to the SUPPLY site (`gui/multisig.go:291-293`) goes **RED** —
`--- FAIL: TestSupplyAbortIsTheLastScreenOfTheProgram` — so the asymmetry is real and it is
the build path that is exposed. Coverage agrees: `gui/multisig_build.go:402-404` count 0.

**Why it matters here.** With the body gone, an operator who runs out of blanks at plate 12
of 17 on the built-policy path (Trace B's shape) reads *"Bundle Incomplete ... This set is
not a usable backup yet"*, is then offered *"Verify the engraved plates?"* over a set whose
md1 was never cut, and finally gets `multisigRestoreDocFlow` headed *"This backup is 17
plates ... If any of them is missing, this backup is incomplete."* That is I-12 verbatim,
on the flow that cuts the most steel. The fold's report states the call-site-only pin as a
limitation for *both* paths; measurement shows it is a limitation for exactly one, and the
report does not distinguish them.

**Suggested fix.** `TestSupplyAbortIsTheLastScreenOfTheProgram` is the template and it
already exists in the same file; the build path needs the analogous driver. The build flow
is longer to drive, but `TestBuildHoldingEverySlotReachesTheSeed`
(`gui/multisig_build_allslots_test.go:102`) and `s5DriveToGate`
(`gui/multisig_build_s5_flow_test.go:74`) already reach deep into it, so the driver exists
to be extended rather than written.

---

## 3. (Minor) I-13's fix is pinned only in the pure function; the single `full` argument that decides it is unwatched

**File:** `gui/multisig_verify.go:894` (`showNotice(ctx, th, multisigVerifyOKTitle, multisigVerifyOKMessage(len(legs), full))`)

**Mutation:** `multisigVerifyOKMessage(len(legs), true)`.

```
$ nix develop --command go test ./gui/ -count=1
ok  	seedhammer.com/gui	111.051s
EXIT=0
```

Deleting the `!full` arm itself DOES go red (`TestVerifyOKMessageClaimsASecretOnlyInFullMode`),
so the fold's new text is pinned — but it is pinned as a string function. Nothing at flow
level asserts that a **watch-only** run reaches *"Operator key verified."* rather than
*"Operator key and secret verified."*, which is precisely I-13's finding: a run that created
no ms1, requested none and compared none, claiming a secret was checked.

Recorded as Minor rather than Important because the call site is **unchanged by this fold**
(`full` was already threaded there before `7da66bd`), so this is a pre-existing coverage gap
that the fold's fix now depends on, not a defect the fold introduced. It belongs with
Finding 1's test work: the same flow driver would cover it.

---

## 4. (Minor) the new retry lead reports a FAILED verify as merely incomplete

**File:** `gui/multisig_verify.go:61` (`multisigVerifyRetryLead`), consumed at
`gui/multisig.go:336` and `gui/multisig_build.go:452`

`multisigVerifyRetryLead = "Not every plate is verified. Try again?"` is one string for two
verdicts. The callers re-offer on `verifyIncomplete` **and** on `verifyFailed`, and
`verifyFailed` includes the hard states:

* readback md1 ≠ engraved md1 → `multisigVerifyForeignPolicyBody` ("These plates belong to a
  different wallet") — `gui/multisig_verify.go:674`
* `errVerifyLegHasNoPlate` / `errVerifyPlateUnclaimed` from
  `verifyMultisigLegsPartial` — `gui/multisig_verify.go:868`, `:888`

**Concrete trigger:** present a byte-valid md1 from another wallet at the post-engrave verify
offer. The operator sees `Verify Failed` + "These plates belong to a different wallet", and
the *next* screen says "Not every plate is verified. Try again?" — a sentence whose only
reading is "the run was partial". The correct diagnosis is shown first, so this is a
downgrade of a failure by the following screen rather than a false GREEN; it is the same
class C-1 was about (a failure narrated as an incomplete) at one screen's remove.

Cheapest fix that does not need new machinery: pass the verdict into the lead, e.g. a
`multisigVerifyRetryLeadFor(res)` with a second string for `verifyFailed` ("The verify did
not pass. Try again?"). Note this would also give Finding 1's test something verdict-specific
to assert.

---

## 5. (Nit) `seedFingerprintSuffix`'s zero-fingerprint guard is both unreachable and unpinned

**File:** `gui/multisig_build_census.go:183`

Mutating `if fp == 0` → `if false` leaves the suite GREEN
(`ok seedhammer.com/gui 79.069s`, exit 0). It is unreachable in production as well as
untested: the only caller that supplies a zero fingerprint is `oneSeedPassphraseFact`
(`gui/multisig.go:359`), whose single-element slice takes the `len(seeds) < 2` early return
in `buildPassphraseInventoryLines` and never reaches the suffix. The guard's doc comment
argues for its existence on a case the code cannot produce. Harmless; recorded so it is not
mistaken for a live check later.

## 6. (Nit) `heldSlotOrigins`' bounds/empty guard is unpinned

**File:** `gui/multisig_build.go:1670`

`if s < 0 || s >= len(slotOrigins) || slotOrigins[s] == ""` → `if false` leaves the suite
GREEN (`ok seedhammer.com/gui 78.946s`, exit 0) and does not panic — no test drives a held
slot outside the assembled set. This is a defensive panic-guard whose own comment says the
production flow cannot produce the input, so a green mutation is the expected result; it is
listed only for completeness of the "which new mechanisms are unwatched" ledger.

---

# Also observed, NOT findings

* **Out of scope, one line each:** `gui/singlesig.go` — already filed as F-197/F-198 and
  verified present in `design/FOLLOWUPS.md`; not investigated.
* `buildOriginAnnouncement`'s default arm (`gui/multisig_build.go:1646`) now renders real
  per-slot origins followed by the fixed appositive *", the BIP-48 path for native segwit"*.
  For a `both` slot the origin comes from the card's declared path, which S5 no longer
  constrains to BIP-48, so an exotic card could produce "@0 at m/45h, the BIP-48 path for
  native segwit". Strictly better than the pre-fold sentence (which stated a path the build
  did not use), and I could not reach it from the shipped screens in the time available, so
  I am not raising it as a finding — recorded so it is not lost.
* `cmd/emu/walk_trace_b.js`'s two new `throw`s (`ORIGINS_EXPECTED`, `claims.multiAccountNotice`)
  are real assertions and the committed record shows them satisfied, but they are JS and are
  reachable only from a browser walk — `go test ./...` cannot run them. Their evidence is the
  re-minted `S5-trace-b.record.json`, which I read: `reviewScreen` carries
  `@0atm/48h/0h/0h/2h,@1atm/48h/0h/1h/2h`, `keySourcesScreen` carries
  `Slots@0and@1allcomefromyourseedfor@0`, and `claims.multiAccountNotice: true`.

# Machine checks I re-ran myself (not taken from the fold's report)

```
$ git -C /scratch/code/shibboleth/wt-s5 status --porcelain      -> empty (frozen tree untouched)
$ grep -rn "holds exactly one seed\|A seed you entered" gui/     -> hits in _test.go only (the
                                                                    assertions); production clean
$ grep -n "^### F-19[678]" design/FOLLOWUPS.md                   -> 6979, 7020, 7047 (all filed)
$ nix develop --command go test ./gui/ -count=1                  -> ok, 126.826s (unmutated copy)
$ nix develop --command go test ./oracle/ ./cmd/emu/ -count=1    -> ok (unmutated copy)
```

# Answer to the two halves of the question

1. **Did the fold CLOSE each finding, or MOVE it?** All 17 are closed in the shipped code.
   Two are closed in a way that is one refactor from re-opening and that no executing test
   would notice — **I-4** (Finding 1) and **I-12 on the build path** (Finding 2). Those two
   are moved, not open. C-1, C-2, C-3, I-6 and I-7 are closed at FLOW level and I reproduced
   each RED.
2. **Did the fold INTRODUCE a new defect?** One prose defect of its own authorship
   (Finding 4 — the single retry lead narrating a hard failure as an incomplete) and two
   inert guards it added (Findings 5 and 6). No Critical, and no correctness defect: the
   `open == 0` skip leaves `chosen`/`cosigners`/`origins` nil on paths that all handle nil
   (`buildSlotSources` gates on `gi < len(chosen)`, `buildProvenanceLines` returns nil on an
   empty slice), the registry's `defer reg.scrub()` (`gui/multisig_build.go:227`) is installed
   before the new early `return`s so `reg.passphraseFacts()` at `:475` still reads live
   passphrases, `MasterFP` is derived unconditionally in `seedRegistry.add`, and the new
   `for {}` offers cannot spin because `ChoiceScreen.Choose` returns `ok=false` on `ctx.Done`.
