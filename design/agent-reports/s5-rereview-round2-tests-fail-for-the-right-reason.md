# S5 fold-of-fold — RE-REVIEW (round 2), LENS: **can the new tests fail, and for the right reason?**

**Artifact:** `git diff 830aaf7..s5-multislot` — `da4fa98`, `750296f`, `6088487`
(worktree `/scratch/code/shibboleth/wt-s5`, HEAD `6088487`, tree clean; verified
READ-ONLY at the end of the pass: `git status --porcelain` empty, `git log --oneline -1`
= `6088487`). Every mutation below was run in `cp -a` copies under
`/tmp/claude-1000/.../scratchpad/mut{1,2,3}`.

**Question asked of this round, and the only one:** the fold's headline claim is that its
new tests go RED under the reviewer's own mutations. Do they? And do they go red for the
RIGHT reason, or for a timeout / frame-budget / panic that would get the test relaxed?

---

# VERDICT: **GREEN — 0 Critical, 0 Important**

| | count |
| --- | --- |
| Critical | **0** |
| Important | **0** |
| Minor | 1 |
| Nit | 3 |

Every one of the fold's eight claimed mutations that falls inside this lens was re-run
independently and **all went RED, at the sites claimed, with a failure message that names
the actual defect**. Three further mutations I chose myself — not on the fold's list —
also went RED, including the one fail-safe direction the fold explicitly declared
*unpinned*. One probe designed to catch a frame-budget false alarm came back clean with
**16x headroom measured**.

No new test asserts on SOURCE TEXT. The three tests that do (`TestBothEngraveFlowsReOfferTheVerify`,
`TestBuildPassesTheTailsSlotsToTheVerify`, `TestSupplyPassesTheEngravedPolicyToTheVerify`)
are pre-existing, were retained deliberately, and now pin only the argument list — which
the behavioural walk provably cannot see, because its stub eats the arguments.

---

# 1. The reviewer's own mutations, re-run. All RED.

Baseline first, in the unmutated copy, so a RED means the mutation and not the harness:

```
ok  seedhammer.com/gui  121.315s     (exit 0)
  PASS TestBothEngraveFlowsDriveTheRetryLoop            (120.70s: supply 77.04s / build 43.66s)
  PASS TestBuildAbortIsTheLastScreenOfTheProgram        (0.04s)
  PASS TestVerifyRetriesAfterACorrectableFirstSeed      (3 rows)
  PASS TestRestoreDocNamesEveryPassphrasedSeed          (2 rows)
  PASS TestRestoreDocMergesOneSeedHeldAtTwoSlots
  PASS TestVerifyIncompleteInstructionCanBeObeyed       (2 rows)
  PASS TestSupplyAbortIsTheLastScreenOfTheProgram
```

## B4 (i) — `for {` → one iteration, at BOTH call sites

`gui/multisig.go:330`, `gui/multisig_build.go:446`, rewritten to
`for mutOnce := 0; mutOnce < 1; mutOnce++ {`. Command run:
`nix develop --command go test ./gui/ -count=1 -run TestBothEngraveFlowsDriveTheRetryLoop`.

```
--- FAIL: TestBothEngraveFlowsDriveTheRetryLoop (61.19s)
    --- FAIL: .../supply (39.38s)  multisig_engrave_tail_walk_test.go:391:
        after an INCOMPLETE verify the offer was not made again. The screen reads
        "RestoreDocType:P2WSH3-of-4multisig(sorted)Descriptor:wsh(sortedmulti(3,xpub..."
    --- FAIL: .../build (21.80s)   multisig_engrave_tail_walk_test.go:420:  ... same
FAIL  seedhammer.com/gui  61.189s
```

**Right reason: yes.** The screen quoted in the failure IS the restore document — the exact
surface I-4 exists to keep an incomplete verify away from. The message names the mechanism
(one-shot offer) and the consequence (re-cut every plate).

## B4 (ii) — `{"VERIFY AGAIN", "CONTINUE"}` → `{"CONTINUE", "VERIFY AGAIN"}`, both sites

```
--- FAIL: TestBothEngraveFlowsDriveTheRetryLoop (62.01s)
    --- FAIL: .../supply  multisig_engrave_tail_walk_test.go:391:
        pressing the row LABELLED "VERIFY AGAIN" did not run the verify a second time
        (entered 1 time(s)); the screen reads "RestoreDocType:P2WSH3-of-4multisig..."
    --- FAIL: .../build   ... same
```

**Right reason: yes, and this is the one that mattered.** The claim under test was that the
loop keys on the returned INDEX with no label lookup anywhere. I confirmed the mechanism
independently: `grep -rn "multisigVerifyFn\|multisigVerifyFlow" --include="*.go" .` shows
the two call sites read `sel != 0` and nothing in `gui/` maps a row label to a verdict. The
test locates rows by DRAW POSITION (`s5RowIndexOf`, `gui/multisig_engrave_tail_walk_test.go:43-71`)
rather than by row number, which is why the swap is visible to it at all.

## B5 — the BUILD-path abort body made a no-op

`gui/multisig_build.go:402-404`, keeping the `if bundleEngrave(...) != bundleEngraveDone {`
line the grep test needs and replacing `return` with `_ = 0`:

```
--- FAIL: TestBuildAbortIsTheLastScreenOfTheProgram (0.10s)
    multisig_engrave_tail_walk_test.go:477: the program did not end after the abort; it drew:
    "Verifytheengravedplates?VerifynowSkipVerifyBundle || ..." (x200)
--- PASS: TestBothEngraveFlowsGateOnACompletedSet   (0.00s)
--- PASS: TestSupplyAbortIsTheLastScreenOfTheProgram (0.04s)
FAIL  seedhammer.com/gui  0.154s
```

**Right reason: yes, and the asymmetry B5 reported is now gone.** The grep test and the
supply twin still pass under the mutation, which is exactly what made the build path
unwatched; the new test is the only thing that sees it. The frame dump is itself the
finding — an operator who just read "Bundle Incomplete" being re-offered the verify.

## B3 — the non-vacuity row, plus both of the fold's other two

| mutation | site | result |
| --- | --- | --- |
| `correctable := false` → `true` | `gui/multisig_verify.go:767` | `--- FAIL .../Back_at_the_first_seed's_ms1_entry_still_abandons`: `a first-seed exit returned 1, want 4` |
| `correctable = correctable \|\| rejected` → `_ = rejected` | `gui/multisig_verify.go:886` | `--- FAIL .../the_first_seed's_hand-typed_ms1_is_rejected`: `returned 4, want 1`; last frame `"Thatisn'tanms1secretshare.VerifyBundle"` |
| delete `correctable = true` at the no-slot break | `gui/multisig_verify.go:859` | `--- FAIL .../the_first_seed_fills_no_slot`: `returned 4, want 1`; last frame `"Noslotmatchesthatseed. ... additandtryagain..."` |

**Right reason: yes, all three.** Each row fails on the VERDICT with the offending screen
text printed alongside it, and each row fails alone — the other two stay green, so the rows
are not interchangeable. The non-vacuity row is real: `correctable := true` at the
declaration does not satisfy it.

I also checked the fold's claim that the no-slot switch's arms all prescribe a remedy,
rather than inheriting it. Read, not described: `multisigVerifyNoSlotBody`
(`gui/multisig_verify.go`, three arms — "Try again and skip the passphrase", "Check the
passphrase before you doubt the plates", "add it and try again") and
`multisigVerifyCoveredSeedBody` (two arms — "Try again and skip the passphrase", and a
lead + "built from different words, or from these words with a different BIP-39
passphrase"). All five name an input the operator can change. The blanket
`correctable = true` after the switch is sound.

## B1 — both of the fold's mutations

| mutation | result |
| --- | --- |
| `if false && g.passphrase == ...` (grouping off) | `FAIL TestRestoreDocNamesEveryPassphrasedSeed/the_SAME_passphrase_at_both_held_slots_(B1)` — *reports 2 passphrase fact(s), want 1*, with the two facts printed; **and** `FAIL TestRestoreDocMergesOneSeedHeldAtTwoSlots` |
| `Label: joinAnd(g.labels)` → `g.labels[0]` | `FAIL TestRestoreDocMergesOneSeedHeldAtTwoSlots:257` — *the merged passphrase statement does not name "your seed for @1"; the merge dropped a held slot*, with the whole document printed |

**Right reason: yes.** Both failures print the actual restore-document text, so the reader
of a future CI log sees the artifact rather than a boolean.

---

# 2. Three mutations I chose MYSELF, which the fold did not run

These are the round's added value: the fold proved its tests fail under the reviewer's
list, and a list is not a lens.

## (a) The fail-safe direction the fold declared UNPINNED — it is pinned

The fold states plainly (`s5-fold-rereview-fold-round1.md:71-75`) that the CHOICE of key
`(Mnemonic, Passphrase)` over `MasterFP` is *"a fail-safe argument, not a behaviour any
test here can distinguish"*. That is true of the `MasterFP` variant (it needs a 2^32
collision to differ). It is **not** true of the other unsafe key, which costs nothing to
exhibit: dropping the passphrase from the key entirely.

Mutation: `if g.passphrase == s.Passphrase && slices.Equal(...)` → `if slices.Equal(g.mnemonic, s.Mnemonic)`.

```
--- FAIL: TestRestoreDocNamesEveryPassphrasedSeed/two_DIFFERENT_passphrases_at_the_two_held_slots
    the registry reports 1 passphrase fact(s), want 2.
    The fact list is one entry per SECRET, not one per held slot:
    [{Label:your seed for @0 and your seed for @1 MasterFP:2326417227 Uses:true}]
```

**This is the direction that loses funds** — two required passphrases silently rendered as
one — and row 1 of the repaired table catches it. Worth recording: the fold under-claimed
its own coverage, and the round-1 finding that the OLD test `t.Fatal`ed when the two
fingerprints came out equal is what had been suppressing this row.

## (b) CONTINUE must LEAVE — the third assertion, mutated

Mutation: `if !ok || sel != 0 { break }` → `_ = sel; if !ok { break }` at both call sites,
i.e. CONTINUE also retries.

```
--- FAIL: TestBothEngraveFlowsDriveTheRetryLoop/supply  multisig_engrave_tail_walk_test.go:391:
    pressing the row LABELLED "CONTINUE" did not leave the verify offer for the restore
    document; the screen reads "Noteveryplateisverified.Tryagain?VERIFYAGAINCONTINUEVerifyBundle"
--- FAIL: .../build   ... same
```

**Right reason: yes.** This is the assertion that stops the `calls == 2` count from being
satisfiable by a loop that never terminates. Both directions of the index/label
disagreement are therefore covered, not just one.

## (c) Is the retry test's frame budget fragile? MEASURED: no, 16x headroom

Round 1's own F-200 records a harness whose failure message blames the wrong layer, so I
tested whether `TestBothEngraveFlowsDriveTheRetryLoop` is one frame away from the same
trap. Probe: cut every post-engrave budget in `s5AssertRetryLoop` by 16x —
`pumpUntil(..., 128)` → `8` (both sites), `s5PumpUntilCalls(..., 256)` → `16`,
`pumpUntil(frame, "Descriptor:", 256)` → `16`.

```
ok  seedhammer.com/gui  97.351s     (exit 0)
```

**The test still passes at one-sixteenth of its budget.** So none of the retry-loop
assertions can plausibly fail for budget exhaustion; a RED there means the loop. (The
budget that IS fragile is `engraveOnePlate`'s 4096 inside `s5EngraveEveryPlate`, and that
is already filed as F-200 — not re-reported here.)

---

# 3. Vacuity and wrong-reason checks that came back clean

Recorded so a later reader does not have to redo them.

- **No new test greps source.** `TestVerifyIncompleteInstructionCanBeObeyed` asserts on the
  return value of `multisigVerifyIncompleteText`, i.e. the words on the screen, not on the
  file that produces them; `TestVerifyIncompleteReportsWhatTheComparatorMatched` gained the
  same assertion **on the driven frame**. B4's original defect is not reproduced.
- **The three retained grep tests do not overlap the behavioural one.** Read, not assumed:
  `TestBothEngraveFlowsReOfferTheVerify` (`gui/multisig_verify_report_test.go:1032-1064`)
  asserts the argument list, `res := `, the `res != verifyIncomplete && res != verifyFailed`
  gate and the presence of `multisigVerifyRetryLead`. None of those can see a one-iteration
  loop (confirmed: it stayed green under B4(i) in round 1 and its needles are byte-identical
  under my mutation). The seam test cannot see the arguments (the stub discards them). The
  split is correct.
- **No global-state leak from the seam.** `grep -rn "t.Parallel()" gui/` → **no matches**, so
  the package-level `multisigVerifyFn` cannot be observed mid-swap by a concurrent test;
  `t.Cleanup` restores it (`gui/multisig_engrave_tail_walk_test.go:114`), and
  `grep -rn multisigVerifyFn --include="*.go" .` finds exactly three production references
  (the declaration and the two call sites) plus the test.
- **The seam does not hide argument corruption across attempts.** I checked the one place a
  second attempt could be handed a corrupted obligation list: `verifyFreshSlots`
  (`gui/multisig_verify.go`) allocates `fresh` and only reads `expected`, so
  `engravedSlots` is not mutated by attempt 1. `supplyEngraveTail` (`gui/multisig.go:230`)
  and `buildEngraveTail` (`gui/multisig_build.go:384`) are the only producers.
- **B2's new instruction is behaviourally supported, not just reworded.**
  `TestVerifyFullModeTwoSeedsReportsTheFullSuccess`
  (`gui/multisig_verify_report_test.go:624-647`) drives Trace B's two masters through ONE
  pass and asserts "Verify OK" and `res == verifyComplete`. So "type ALL of this wallet's
  seeds in one pass" names an action that reaches a clean verdict — which is precisely what
  the old sentence did not.
- **`multisigVerifyMS1Entry`'s new third return has one caller** (`gui/multisig_verify.go:868`);
  `grep -rn multisigVerifyMS1Entry --include="*.go" .` finds no other production site, so
  the signature change cannot have silently dropped a `rejected` somewhere.
- **B5's test cannot pass vacuously.** It `t.Fatalf`s if "Bundle Incomplete" is never
  reached (`:462`) and `done` is only set after the flow returns normally, so a panic or a
  route that never aborts is a failure, not a pass.
- **B1's merge does not lose the passphrase warning when it collapses to one fact.** Read
  `buildPassphraseInventoryLines` (`gui/multisig_build_census.go:121-172`): the `len(seeds) < 2`
  early return still emits *"A BIP-39 passphrase WAS used. It is not on these plates..."*
  and *"Without it, these plates do not reach the money."* — it drops only the per-seed
  enumeration, which with one secret has nothing to enumerate. `passphraseFacts()` has
  exactly one production consumer (`gui/multisig_build.go:479`), so the joined `Label`
  reaches nothing else.

---

# 4. Minor / Nit — recorded, not gating

1. **(Minor) `s5AssertRetryLoop` misattributes a stale-lead regression — F-200's class, in
   the new file.** `gui/multisig_engrave_tail_walk_test.go:301-308`. The needle for "the
   offer was made again" is `multisigVerifyRetryLead`, so a regression that re-offers
   correctly but forgets to update the lead fails with *"after an INCOMPLETE verify the
   offer was not made again"* — which is false. Measured: mutating
   `lead = multisigVerifyRetryLead` to `_ = multisigVerifyRetryLead` at both call sites gives

   ```
   --- FAIL: TestBothEngraveFlowsDriveTheRetryLoop/supply  ...:391:
       after an INCOMPLETE verify the offer was not made again. The screen reads
       "Verifytheengravedplates?VERIFYAGAINCONTINUEVerifyBundle"
   ```

   The quoted frame shows the offer plainly WAS made again, with both retry rows drawn. The
   test still goes red — the safe direction — but the message points at the loop instead of
   the lead, which is how a test gets "fixed" by relaxing the wrong assertion. Cheap repair:
   pump on `"VERIFY AGAIN"` (the row that proves the offer reappeared) and assert the lead
   separately with its own message. Suggest folding into F-200 rather than a new item.
2. **(Nit) Only one of the two ms1-rejection arms has a row.**
   `TestVerifyRetriesAfterACorrectableFirstSeed` drives *"That isn't an ms1 secret share."*
   (wrong OBJECT, `gui/multisig_verify.go:1010`) and not *"That isn't a valid ms1 secret
   share."* (`DecodeMS1` error, `:1015`). Both set `rejected = true` on the same line, so
   this is a coverage nit, not an unpinned branch.
3. **(Nit) B2's sentence says "ALL of this wallet's seeds", and the obligation is narrower
   than that.** `expectedSlots` is the tail's own held-slot list on both paths
   (`gui/multisig.go:230`, `gui/multisig_build.go:384`), so it never contains a slot the
   operator's own seeds do not fill; "all of YOUR seeds" is the exact statement. Harmless in
   practice — the outstanding slots named on the same screen are always ones they can reach —
   but a 2-of-3 operator could read it as needing a cosigner's seed and abandon the verify.
4. **(Nit) `TestVerifyIncompleteInstructionCanBeObeyed`'s positive tokens are weak.**
   `"ALL"`, `"one pass"`, `"nothing"` are individually satisfiable by a reworded sentence
   that is still not obeyable. This is inherent to asserting on prose and is mitigated by
   the behavioural pin named in §3; listed for ledger completeness only.

---

# 5. Out of scope — one line each, as the brief requires

- `gui/singlesig.go` (F-197, F-198), `verifyRefused`'s dead end (F-199), the frame budget
  (F-200) and the retry lead's three verdict shapes (F-201): filed, not re-raised.
- R-1 refuted; I-8 ruled (b); the gate record needed no re-mint. Not re-derived.
- The five build gates: settled by the controller on `6088487`, not re-run here.

---

# 6. What a round-3 re-review should be asked

**Nothing, under this lens.** The question "can these tests fail, and for the right reason"
now has no more answers: eleven mutations across five mechanisms, every one RED at the
claimed site with a correctly-attributed message, plus a measured 16x budget margin
refuting the one wrong-reason hypothesis worth testing. The remaining items are a
diagnostic-quality Minor and three Nits, none of which holds a gate.

If another round is wanted before merge, it should be a **different question** — the
lens-closure rule, not a harder look at this one. The unasked ones I noticed while working
and deliberately did not open: whether the retry loop's *other* verdicts (`verifyFailed`,
`verifyRefused`) reach the operator sensibly now that `verifyIncomplete` carries a
zero-leg state, and whether the restore document reads correctly for a build holding three
or more slots from one master (`joinAnd` renders "a, b and c" and no test drives that
arity).

---

*Round 2, lens: tests-fail-for-the-right-reason. 11 mutations run, 11 RED, 1 budget probe
clean. Gate: **GREEN (0C / 0I)**.*
