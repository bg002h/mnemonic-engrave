# S5 fold — the FOLD of round 1's B1..B5

**Worktree:** `/scratch/code/shibboleth/wt-s5`, branch `s5-multislot`.
**Base:** `830aaf7` (unchanged, still addressable; nothing rebased, amended or reordered).
**Landed:** three commits, tree clean.

| commit | covers |
| --- | --- |
| `da4fa98` | B1 — the restore document counts SECRETS, not held slots |
| `750296f` | B2, B3 — the verify's instruction can be obeyed, and its remedy screens reach the retry |
| `6088487` | B4, B5 — the retry loop and the BUILD abort, driven by executing tests |

**5 of 5 FIXED. 0 disputed. 0 out-of-scope fixes.** One NEW defect found and
NOT folded (reported below). Two interactions with existing Minors recorded.

---

## B1 — FIXED (`da4fa98`)

**Change.** `seedRegistry.passphraseFacts()` now emits one fact per SECRET, not
per registry entry, grouping on the exact derivation unit `(Mnemonic,
Passphrase)` via `slices.Equal` and joining the group's labels with the
package's `joinAnd`. `gui/multisig_build_slots.go:286-320` (rationale `:247-285`).

**The key is (Mnemonic, Passphrase) and deliberately NOT MasterFP,** for the
reviewer's reason, which I re-derived rather than inherited: this dedupe
SUPPRESSES a sentence, so a 4-byte fingerprint collision between two unrelated
seeds would merge them and drop a required passphrase off the artifact that
outlives the operator. `buildSlotGate` can key on `MasterFP` because there a
collision only ADDS a spurious notice. Same identity rule, opposite failure
direction. The reasoning is recorded in the function's own comment so the two
sites cannot be "unified" later by someone who notices they disagree.

**Tests pinning it** (`gui/multisig_build_perseed_passphrase_test.go`):

* `TestRestoreDocNamesEveryPassphrasedSeed` — **repaired**, as required. It drove
  `alpha`/`beta` only and `t.Fatal`ed when the two fingerprints came out EQUAL,
  which is the premise check for that fixture and simultaneously a refusal to run
  the one cell B1 lives in. It is now a table over both cells, with the premise
  asserted in BOTH directions (row 1 requires distinct fingerprints, row 2
  requires equal ones).
* `TestRestoreDocMergesOneSeedHeldAtTwoSlots` — **new**, round 1's measured Trace
  B shape (A@0 + A@1 under one passphrase, B@2 bare). It is the MIXED cell on
  purpose: with a bare seed present the fact list still has two entries, so the
  enumeration arm runs and the merged LABEL is actually drawn. The pure
  two-slot case collapses to the documented `len(seeds) < 2` single-seed arm and
  renders no label at all — both cells are asserted, in different tests.

**Mutations run, both RED:**

1. disable the grouping (`if false && g.passphrase == ... `), restoring one fact
   per entry. Whole `gui` suite:

   ```
   --- FAIL: TestRestoreDocNamesEveryPassphrasedSeed (0.02s)
   --- FAIL: TestRestoreDocMergesOneSeedHeldAtTwoSlots (0.01s)
   FAIL	seedhammer.com/gui	157.032s        (exit 1)
   ```

2. keep the merge, drop `joinAnd` (`Label: g.labels[0]`):

   ```
   --- FAIL: TestRestoreDocNamesEveryPassphrasedSeed/the_SAME_passphrase_at_both_held_slots_(B1)
     the merged fact's label "your seed for @0" does not name "your seed for @1",
     so the merge lost a held slot rather than a duplicate sentence
   --- FAIL: TestRestoreDocMergesOneSeedHeldAtTwoSlots
     the merged passphrase statement does not name "your seed for @1"; the merge
     dropped a held slot
   ```

**NOT PINNED, stated rather than implied.** The CHOICE of key is a fail-safe
argument, not a behaviour any test here can distinguish. `(Mnemonic, Passphrase)`
and `MasterFP` agree on every input except a fingerprint collision, and
exhibiting one costs 2^32 work. A reviewer should read the comment, not look for
a red test.

---

## B2 — FIXED (`750296f`)

**Change.** `multisigVerifyIncompleteText`'s closing sentence, and only the
wording. `gui/multisig_verify.go:459-495`. It now reads:

> Choose VERIFY AGAIN on the next screen and type ALL of this wallet's seeds in
> one pass; a new attempt keeps nothing from this one. Until then, do not fund
> this wallet.

**The state-carrying fix is NOT taken, and I resolved the reviewer's reasoning
against the call graph rather than accepting it.** `multisigVerifyFlow` calls
`bundleGatherFlow` at `gui/multisig_verify.go:691` on every
invocation, so attempt 2 is presented whatever plate set is on the bench then.
Carrying `covered`/`legs` across attempts would let a Verify OK be assembled
from two readbacks that were never both true at once. The ruling is recorded in
the function's comment, with the note that coverage carry-over is a design
(re-verify the retained legs against the new readback), not a hoisted variable.

**Tests pinning it:**

* `TestVerifyIncompleteInstructionCanBeObeyed` (new, two rows) — asserts the
  banned phrasing is gone, that each clause of the replacement is present
  separately (`VERIFY AGAIN`, `ALL`, `one pass`, `nothing`), that the funding
  warning survives, and that the longer body still DRAWS.
* `TestVerifyIncompleteReportsWhatTheComparatorMatched` (existing) — gained the
  same "must not say `type the remaining seed`" assertion **on the driven
  frame**, because a string test cannot see a body that never reaches a screen.

**Mutation run, RED** — restore the old sentence verbatim:

```
--- FAIL: TestVerifyIncompleteReportsWhatTheComparatorMatched (0.14s)
--- FAIL: TestVerifyIncompleteInstructionCanBeObeyed (0.26s)
    --- FAIL: .../one_outstanding
    --- FAIL: .../two_outstanding
```

**Fit MEASURED, not argued.** `assertModalBodyFits` reports the new body at
**269** and **272** characters drawn in full, **headroom 195** against the
80-character margin (the pre-fold body measured 244 headroom). Note this
incidentally closes round 1's **Minor 2** for this function — the guard it said
was missing now exists. I judged that in scope because it proves MY edit, not
because the Minor was on the list; flagging it so it is not mistaken for scope
creep.

---

## B3 — FIXED (`750296f`)

**Change.** A `correctable bool` local in `multisigVerifyFlow`, set at the
no-slot/covered-seed break and at the ms1-entry break **only when the helper
reports a rejected object**, consumed at the zero-legs return.
`multisigVerifyMS1Entry` gained a third return value (`rejected`) so a Back and
a rejection stop being the same bool. Sites, resolved against the tree at
`6088487`: declaration `gui/multisig_verify.go:767`, set at `:859` (the no-slot /
covered-seed break) and `:886` (`correctable = correctable || rejected`),
consumed at `:937`, helper signature at `:1004`.

**Reproduced before fixing**, exactly as round 1 measured — verdict `4`
(`verifyAbandoned`) on both routes:

```
--- FAIL: TestVerifyRetriesAfterACorrectableFirstSeed/the_first_seed_fills_no_slot
    last frame: "Noslotmatchesthatseed.IfthiswalletwasbuiltwithaBIP-39passphrase,
                 additandtryagain:withoutitthesamewordsderiveadifferentwallet."
    verdict: 4       a first-seed exit returned 4, want 1
--- FAIL: .../the_first_seed's_hand-typed_ms1_is_rejected
    last frame: "Thatisn'tanms1secretshare.VerifyBundle"
    verdict: 4       a first-seed exit returned 4, want 1
```

The reviewer's prescription **survived contact with the code and was implemented
as written**, including the second return value. One implementation detail the
prescription left open: I set `correctable` unconditionally after the no-slot
switch rather than per arm, because all of its arms print a remedy — recorded in
the comment so it is a decision, not an accident.

**Test pinning it:** `TestVerifyRetriesAfterACorrectableFirstSeed`, three rows,
all driven through the real screens. Route (b) is reached by typing a real
engraved **mk1** at the "Type ms1" prompt, which is checksum-valid and is not a
`codex32.String`, so it exercises the object-rejection arm rather than a
keyboard error.

**Mutations run, all RED:**

| mutation | red test |
| --- | --- |
| `correctable = true` → `false` at the no-slot break | `.../the_first_seed_fills_no_slot` |
| `_ = rejected` instead of `correctable = correctable \|\| rejected` | `.../the_first_seed's_hand-typed_ms1_is_rejected` |
| **NON-VACUITY:** `correctable := true` at the declaration | `.../Back_at_the_first_seed's_ms1_entry_still_abandons` |

The third row is what stops "re-offer on everything" from passing the other two:
a Back is the operator leaving, and it must still abandon.

---

## B4 — FIXED (`6088487`). Yes, it fails under BOTH of the reviewer's exact mutations.

**Change.** `var multisigVerifyFn = multisigVerifyFlow`
(`gui/multisig_verify.go:660`, with its rationale at `:639-659`), dispatched by
both callers (`gui/multisig.go:336`, `gui/multisig_build.go:452`). The seam substitutes the
VERDICT SOURCE only: the offer screens, the `ChoiceScreen` row mapping, the loop
condition and the retry lead are all production code under the test, which is
where both mutations live.

**Test pinning it:** `TestBothEngraveFlowsDriveTheRetryLoop`
(`gui/multisig_engrave_tail_walk_test.go`), on **both** call sites, each
completing a REAL engrave first (supply: 14 plates; build: 9). With the stub
returning `verifyIncomplete` every time it drives three offers — "Verify now",
then the row that SAYS "VERIFY AGAIN", then the row that SAYS "CONTINUE" —
locating each row by **where its label is DRAWN**, never by row number, and
asserts the verify was entered **exactly twice**.

**The reviewer's two mutations, run verbatim, both RED at both call sites:**

1. `for {` → `for mutOnce := 0; mutOnce < 1; mutOnce++ {` at both sites:

   ```
   --- FAIL: TestBothEngraveFlowsDriveTheRetryLoop/supply (77.93s)
       after an INCOMPLETE verify the offer was not made again. The screen reads
       "RestoreDocType:P2WSH3-of-4multisig(sorted)Descriptor:wsh(sortedmulti(3,..."
   --- FAIL: TestBothEngraveFlowsDriveTheRetryLoop/build (47.22s)
       ... "RestoreDocType:P2WSH2-of-3multisig(sorted)..."
   FAIL	seedhammer.com/gui	125.181s        (exit 1)
   ```

2. `[]string{"VERIFY AGAIN", "CONTINUE"}` → `{"CONTINUE", "VERIFY AGAIN"}` at
   both sites:

   ```
   --- FAIL: TestBothEngraveFlowsDriveTheRetryLoop/supply (84.19s)
       pressing the row LABELLED "VERIFY AGAIN" did not run the verify a second
       time (entered 1 time(s)); the screen reads "RestoreDocType:P2WSH3-of-4..."
   --- FAIL: TestBothEngraveFlowsDriveTheRetryLoop/build (52.01s)
       ... same
   FAIL	seedhammer.com/gui	136.209s        (exit 1)
   ```

**This is a FLOW-level pin, not a helper-level one:** the failing assertions are
inside `supplyMultisigPolicyFlow` and `buildMultisigPolicyFlow` driven end to
end through a completed engrave. What it does NOT pin is which obligation
crosses the call, because the stub eats the arguments — which is why the three
source-grep tests stay.

**Three grep tests updated, not deleted.** `TestBothEngraveFlowsReOfferTheVerify`
was the one the reviewer named; **`TestBuildPassesTheTailsSlotsToTheVerify` and
`TestSupplyPassesTheEngravedPolicyToTheVerify`
(`gui/multisig_verify_flow_test.go:373, 394`) also grep the same call string and
the re-review did not list them.** I found them by grepping the superseded
phrasing after the seam landed; a full-suite run confirmed they were the ONLY
two failures at that point. Their needles now name `multisigVerifyFn`; the
ARGUMENTS they exist for are unchanged, and their comments — which claimed no
behavioural test in the package could reach the offer — are corrected rather
than left to go stale.

---

## B5 — FIXED (`6088487`). Yes, it fails under the reviewer's exact mutation.

**Test pinning it:** `TestBuildAbortIsTheLastScreenOfTheProgram`, the build-path
twin of `TestSupplyAbortIsTheLastScreenOfTheProgram`. It drives
`buildMultisigPolicyFlow` to the first plate's engrave-style picker, presses
Back to take `bundleEngrave`'s set-level abort, dismisses the modal, and asserts
the flow ENDS with none of `"Verify the engraved plates?"`, `"This backup is"`,
`"Descriptor:"` drawn afterwards. **The reviewer was right that no new harness
was needed** — the route is the one `gui/multisig_build_walk_test.go` already
walks. No production change was required; the guard was already correct.

**Mutation run, RED** — keep the `if bundleEngrave(ctx, th, "Build Policy",
cardsOut) != bundleEngraveDone {` line that `TestBothEngraveFlowsGateOnACompletedSet`
greps for, make the BODY a no-op:

```
--- FAIL: TestBuildAbortIsTheLastScreenOfTheProgram (0.07s)
    the program did not end after the abort; it drew: ...
FAIL	seedhammer.com/gui	0.105s        (exit 1)
```

`TestSupplyAbortIsTheLastScreenOfTheProgram` and
`TestBothEngraveFlowsGateOnACompletedSet` both still PASSED under that mutation,
which is exactly the asymmetry B5 reported, now removed.

---

## Build gate — verbatim, on `6088487`

```
go test ./... -count=1                 EXIT=0  ok=51  FAIL=0
gofmt -l ./                            EXIT=0  lines=0
./scripts/oracle-live.sh               EXIT=0  discovered=7  ran=7  verdict=live checks: PASS (exit 0)
./cmd/emu/build.sh                     EXIT=0  built emu.wasm (9976131 bytes); serve this directory and open index.html
go vet ./... (COLD GOCACHE)            EXIT=1  findings=40  outside_test=0
```

All five match the pre-fold baseline exactly. `go vet` exit 1 / 40 findings / 0
outside `_test.go` IS the clean result here and required `GOCACHE=$(mktemp -d)`
— warm, vet prints nothing, exits 0 and proves nothing. Every exit code above
was taken **unpiped**, from `$status` immediately after the command, with output
redirected to a file first.

Baseline re-measured on `830aaf7` before any edit: `go test ./... -count=1` exit
0, 51 `ok`, 0 `FAIL`.

## Gate record — NOT re-minted, and not needed

Checked rather than assumed, in two directions:

* The changed screen strings (`the remaining seed`, `VERIFY AGAIN`, `Verify
  Incomplete`, `No slot matches`, `isn't an ms1`) appear in **no** file under
  `oracle/gaterecords/` and **no** `cmd/emu/*.js` walk. The grep was
  positive-controlled against `Verify the engraved plates`, a string this fold
  did NOT change, which hit three walk files — so the empty result is absence,
  not a grep that never ran.
* `S5-trace-b.walk.json` records `seedSources: ["@0:payload", "@1:payload",
  "@2:typed"]` — **one** typed seed, so the registry holds one entry and B1's
  merge cannot move it. Its `restoreDoc` takes the "No BIP-39 passphrase was
  used" arm either way, since `buildPassphraseInventoryLines` returns that single
  line whenever no fact carries a passphrase, merged or not.
* Empirically: `go test ./...` compares every record against its committed
  expectation and is green, and `oracle-live.sh` PASSes 7/7.

No gate record was hand-edited.

---

## NEW defect — REPORTED, NOT FOLDED

**N-3 (Important, same class as B3, at a site B3 did not name):
`verifyRefused` dead-ends on a correctable input error.**

`gui/multisig_verify.go:698-702`:

```go
readbackMd1, readbackMk1s, ok := extractReadbackMd1AndMk1s(cards)
if !ok {
    showError(ctx, th, "Verify Bundle", "Read back one wallet-policy md1 AND the operator key card(s) (mk1).")
    return verifyRefused
}
```

That screen tells the operator exactly what to present — a correctable input
mistake, in the same sense as B3's two routes — and `verifyRefused` is a verdict
neither caller re-offers on, so the next screen is the restore document. B3's
prescription scoped `correctable` to the two `break`s at the seed and ms1
entries; this is a third site of the same shape, reachable before any seed is
typed (present the mk1s and forget the md1, or bring one plate short). I did not
fold it: it is outside B1..B5 and folding an unreviewed control-flow change into
a fold is the thing the brief forbids. It wants a decision, not a reflex — the
same `correctable` local would cover it, but `verifyRefused` also carries two
programmer-error refusals that must NOT loop.

## Interactions with existing Minors — recorded, not fixed

* **Minor 4 gets slightly worse and slightly more urgent.** B3 routes more states
  to `multisigVerifyRetryLead` ("Not every plate is verified. Try again?"),
  including "the seed you typed fills no slot". The lead is still literally TRUE
  there (zero plates are verified), so this is not a new defect — but
  `multisigVerifyRetryLeadFor(res)` now has three verdict shapes to distinguish
  rather than two, and B4's new flow test gives it something verdict-specific to
  assert against.
* **Minor 2 is closed for `multisigVerifyIncompleteText` only**, as a side effect
  of proving B2's longer body draws. Its two siblings are unaffected.

---

## Text for `design/FOLLOWUPS.md` (I cannot write that file)

```
- **F-199 (Important) — `verifyRefused` dead-ends on a correctable readback.**
  Owning phase: next cycle's implementation phase.
  `gui/multisig_verify.go:698-702` shows "Read back one wallet-policy md1 AND
  the operator key card(s) (mk1)." and returns `verifyRefused`, which neither
  engrave caller re-offers on, so the restore document is the next screen. Same
  class as round-1 B3 at a third site: a screen naming a remedy followed by a
  document headed "If any of them is missing, this backup is incomplete". Found
  while folding B3; deliberately NOT folded, because B3's prescription scoped
  `correctable` to the seed-entry and ms1-entry breaks. Needs a decision rather
  than a reflex: `verifyRefused` also carries two programmer-error refusals
  (empty `expectedSlots`, missing engraved md1) that must NOT loop, so the fix
  is per-site, not per-verdict. Reachable before any seed is typed: present the
  mk1s and forget the md1, or bring one plate short.

- **F-200 (Minor) — `engraveOnePlate`'s frame budget is harness-dependent, and
  the failure looks like a broken flow.** Owning phase: next cycle's
  implementation phase.
  `gui/multisig_build_walk_test.go:443` gives one plate 4096 frames. MEASURED on
  the same plate: the engraver closed at frame **881** under `runUITouchRaster`
  and at frame **10585** under plain `runUI`, because virtual time in the
  synctest bubble advances per idle point rather than per frame.
  `s5EngraveOnePlate` (`gui/multisig_supply_passphrase_test.go:110`) carries
  32768 for the same reason and says nothing about it. A future test that pairs
  `runUI` with `engraveOnePlate` fails on "the engrave never closed the
  engraver, so no plate was cut" with the engrave running perfectly — it cost an
  hour in this fold. Either make the budget a function of the harness or have
  the helper state its precondition. Recorded in
  `gui/multisig_engrave_tail_walk_test.go` at `s5EngraveEveryPlate` meanwhile.

- **F-201 (Minor) — `multisigVerifyRetryLeadFor(res)`, upgraded from round 1's
  Minor 4.** Owning phase: next cycle's implementation phase.
  Round 1 filed the retry lead ("Not every plate is verified. Try again?") as
  narrating a FAILED verify as an incomplete. B3 adds a third shape: a first-seed
  refusal with zero legs. The lead stays true in all three, but it is now the
  single sentence covering "some plates checked", "a comparison disagreed" and
  "the seed you typed fills no slot". `TestBothEngraveFlowsDriveTheRetryLoop`
  already drives the offer through a seam that can return any verdict, so a
  verdict-specific lead is now assertable at flow level.
```

---

## What a round-2 re-review should be asked

Scope it to *"did this fold close B1..B5, and did it introduce a new defect"*.
Facts already settled and not to be re-derived: the five gates pass and match
the baseline (numbers above, unpiped); `go vet` exit 1 / 40 / 0-outside-`_test.go`
is clean and needs a cold cache; the gate record needed no re-mint and why; the
eight mutations above are RUN, with the RED output quoted. The build gate covers
none of the UI behaviour — the executing tests named under B3, B4 and B5 are the
only mechanism that can, so the questions worth budget are whether those tests
can fail for the RIGHT reason and whether B1's dedupe can fail unsafe.

*Fold of round 1. 5 of 5 fixed, 8 mutations run and RED, 1 new defect reported.*
