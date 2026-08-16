# S5 fold — RE-REVIEW (round 1) — GATE VERDICT

**Artifact:** the S5 fold, commits `5f54737..830aaf7` on `s5-multislot`
(worktree `/scratch/code/shibboleth/wt-s5`, HEAD `830aaf7`, tree clean, READ-ONLY).
**Question asked of this round, and the only one:** did the fold CLOSE round 0's
3 Critical / 14 Important (+ F-189), and did the fold itself introduce anything new?
This is not a fresh audit; round 0's settled facts are restated below rather than re-derived.

---

# VERDICT: **RED — 5 blocking (0 Critical, 5 Important)**

| | count |
| --- | --- |
| Critical | **0** |
| Important | **5** |
| Minor | 5 |
| Nit | 4 |

Round 0's three Criticals are gone: no finding in this round reaches Critical, and no
surviving finding produces a false "Verify OK", a false Verify-Complete verdict, or a
plate/secret mis-encoded on steel. Every blocking item below is either a *document that
mis-describes a correct build*, a *screen whose instruction cannot be obeyed*, or a
*new safety mechanism that no executing test would notice the loss of*.

Seven blocking findings arrived from the lens passes; each was put through a
two-skeptic refutation pass and **all seven survived, none were refuted**. They dedupe
to **five** distinct defects (B2 absorbs two lens reports of the same screen; B4 absorbs
two mutations of the same unwatched mechanism).

---

# The five blocking findings, most severe first

Every "minimal fix" below was resolved against the real call graph — callers grepped,
signatures read, and (for B1) **written, compiled and run** in a `cp -a` copy before
being prescribed. Round 0 taught that a prescribed fix is not authoritative: two of its
own would have failed on contact with the code, and one (I-8(a)) would have introduced
a Critical. The same discipline is applied here, and B2 is the case where it bit — see
its "fix NOT prescribed" note.

---

## B1 (Important) — the restore document lists ONE seed as TWO seeds needing "DIFFERENT passphrases"

**Defect.** `seedRegistry.passphraseFacts()` emits one fact per registry ENTRY, but the
build flow registers one entry per HELD SLOT, so a single master held at two slots is
printed twice on the artifact that outlives the operator — with the *same* fingerprint on
both lines and a sentence telling the reader to "record each one against its fingerprint".

**Site.** `gui/multisig_build_slots.go:259-267` (`passphraseFacts`, new in `7a23bb5`),
consumed at `gui/multisig_build_census.go:146-168`, called from `gui/multisig_build.go:475`.

**Trigger (measured, not argued).** Trace B's flagship shape: hold `@0` and `@1` from
master A and `@2` from master B; type the same passphrase at both of master A's per-slot
prompts (`buildSeedForSlot`, `gui/multisig_build.go:602-637`, calls `reg.add` once per
held slot, `:614`). I ran the production chain in a `cp -a` copy (frozen tree untouched):

```
facts: [{Label:your seed for @0 MasterFP:3391906514 Uses:true}
        {Label:your seed for @1 MasterFP:3391906514 Uses:true}]

Needs a passphrase: your seed for @0 (master fingerprint ca2c62d2). If more than one is
  listed here they may be DIFFERENT passphrases; record each one against its fingerprint.
Needs a passphrase: your seed for @1 (master fingerprint ca2c62d2). If more than one is
  listed here they may be DIFFERENT passphrases; record each one against its fingerprint.
Needs NO passphrase: your seed for @2 (master fingerprint b8688df1).
```

Two surfaces authored in the **same fold** now state opposite cardinalities for the same
secret: the Key-sources gate (`gui/multisig_build_slots.go:402-421,537`) was re-keyed on
`MasterFP` by C-2's fix and correctly says "Slots @0 and @1 all come from your seed"; the
restore document, three steps later, says two seeds. A reader who cannot find the second
passphrase must decide whether a fully recoverable backup is unrecoverable.

**Why nothing caught it.** `TestRestoreDocNamesEveryPassphrasedSeed`
(`gui/multisig_build_perseed_passphrase_test.go:52-65`) uses passphrases `alpha`/`beta`
and *Fatals if the two fingerprints are equal* — it structurally cannot see this cell.
`TestRestoreDocSaysWhichSeedsNeedNoPassphrase` uses two masters;
`TestSingleSeedInventoryIsUnchanged` registers one seed;
`TestGateAcceptsSameSeedAtDistinctOrigins` builds the right shape but calls only
`buildSlotGate`, with an empty passphrase. The committed Trace B walk takes the
"No BIP-39 passphrase was used" arm (`cmd/emu/walk_trace_b.js:482`, `picks` all `skip`).

**Minimal fix — WRITTEN, BUILT AND RUN, not just prescribed.** Group in
`passphraseFacts()` on the exact derivation unit and join the group's labels:

- key on `(Mnemonic, Passphrase)` via `slices.Equal` — the registry holds both, and this
  is exact. **Not** on `MasterFP`: a 4-byte fingerprint collision would MERGE two
  unrelated seeds and silently drop a required passphrase, which fails in the wrong
  direction. (`buildSlotGate` can accept collisions because there they only produce a
  spurious notice; here they would suppress a true one.)
- render the merged `Label` with the package's existing `joinAnd`
  (`gui/multisig_build_payload.go:431`).

Result of running it in the copy: Trace B reads
`Needs a passphrase: your seed for @0 and your seed for @1 (master fingerprint ca2c62d2).`
+ `Needs NO passphrase: your seed for @2 (…b8688df1).`, and the one-master/two-slot build
collapses to a single fact, which then correctly takes `buildPassphraseInventoryLines`'
documented `len(seeds) < 2` single-seed arm (`gui/multisig_build_census.go:144-146`) — the
two singular lines, no fingerprints, which is exactly right for one seed. **I checked the
composition with that early return specifically; it is the part a paper fix would have
missed.** All existing restore-doc/gate tests still pass with the fix applied
(`TestRestoreDoc*`, `TestSingleSeedInventoryIsUnchanged`,
`TestGateAcceptsSameSeedAtDistinctOrigins` — `ok seedhammer.com/gui 0.152s`).
The fix must land with a test for the same-master/**same**-passphrase cell; today no test
in the tree can fail on it.

---

## B2 (Important) — the new incomplete screen instructs "VERIFY AGAIN … type the remaining seed", and obeying it reports the already-checked plates as NOT verified

*(two lens reports of one defect, merged: `new-defects` and `verify-flow`.)*

**Defect.** `multisigVerifyIncompleteText`'s new closing sentence names a retry that
carries none of the first pass's coverage, so the instruction can never reach Verify OK
and re-issues itself forever.

**Site.** `gui/multisig_verify.go:459-472`, sentence at **:466** (new in `9f93362`),
against `legs`/`covered`/`typed` as function locals at `:697-707` and the two retry loops
at `gui/multisig.go:325-337` and `gui/multisig_build.go:441-453`.

**Trigger (driven through the real screens).** Trace B, three engraved slots across two
masters, honest readback. Pass 1: type master A, STOP HERE →
*"Checked 2 key plates: @0 and @1 … 1 slot is NOT verified: @2 … Choose VERIFY AGAIN on
the next screen and type the remaining seed."* Press VERIFY AGAIN and type **only** the
remaining seed, exactly as instructed → *"Checked 1 key plate: @2 … 2 slots are NOT
verified: @0 and @1. Nothing has been proved about those plates."* The verdict is
`verifyIncomplete`, which re-arms the loop, which re-offers the same instruction. The
only route to Verify OK is typing **every** seed inside one attempt, which no screen says.
Both skeptics reproduced this independently against the fold's own drivers
(`s5DriveVerifyStopAfterOneSeed` over `s5TraceBFullReadback`), with identical frame text.

**Minimal fix — the WORDING, and deliberately NOT the state.** Reword `:466` to say the
retry starts over and every seed must be typed in one pass (e.g. *"Choose VERIFY AGAIN on
the next screen and type ALL of this wallet's seeds in one pass; a new attempt starts from
nothing. Until then, do not fund this wallet."*).

**The state-carrying fix is NOT prescribed, and this is the round-0 lesson applied.** I
resolved it against the call graph and it would introduce a funds-safety defect:
`multisigVerifyFlow` re-runs `bundleGatherFlow` on every invocation
(`gui/multisig_verify.go:646`), so attempt 2 may be presented a **different plate set**
than attempt 1. Carrying `covered`/`legs` across calls would let a Verify OK be assembled
from two readbacks that were never both true at once — a clean verdict no single readback
ever proved. Rewording is the whole fix; if coverage carry-over is ever wanted it needs
its own design (re-verifying the retained legs against the new readback), not a hoisted
variable. Blast radius of the reword: **zero** — `grep` over `gui/` and `cmd/` finds no
test and no emulator walk that matches the sentence.

---

## B3 (Important) — the retry reaches only 2 of the 5 verdicts, so every screen that says "try again" on the FIRST seed dead-ends into the restore document

**Defect.** Both callers re-offer on `verifyIncomplete`/`verifyFailed` only. A first-seed
failure breaks out with `legs` still empty and returns `verifyAbandoned`, so the screens
that explicitly tell the operator to change an input and retry are followed by the restore
document — the exact shape I-4 existed to remove.

**Site.** `gui/multisig_verify.go:790` and `:809` (breaks), `:847-849`
(`len(legs)==0 → verifyAbandoned`), against `gui/multisig.go:333` and
`gui/multisig_build.go:449`.

**Trigger.** Single-held-slot FULL build, "Verify now", present plates, re-type the seed,
then either (a) the seed fills no slot — the F-191 screen prints *"…add it and try again:
without it the same words derive a different wallet."*, or `multisigVerifyNoSlotBody(true,true)`
prints *"Your plates are fine. Try again and skip the passphrase."* — or (b) the hand-typed
ms1 is rejected by `multisigVerifyMS1Entry` (`:904-921`: wrong HRP, or a checksum-valid
string `DecodeMS1` refuses, e.g. a k>0 SSS share). Both land at `verifyAbandoned`; there is
no retry, and the next screen is *"This backup is N plates…"*. Probes: verdict `4`
(`verifyAbandoned`) on both routes, reproduced independently by both skeptics. One skeptic
correctly narrowed the narrative: a plain one-character ms1 typo is intercepted by
`inputCodex32Flow`'s BCH "Fix?" offer (`gui/gui.go:1038-1049`) rather than reaching the
"not a valid ms1" branch — the dead end is unchanged, only the route to it is.

**Minimal fix.** Distinguish *"the operator was shown a correctable screen"* from *"the
operator walked out"*, inside the flow, rather than widening the callers' gate:

- add a local `correctable bool`, set immediately before the `break` at `:790` (all three
  arms of that switch print a remedy) and before the `break` at `:809` **only** when
  `multisigVerifyMS1Entry` reports a rejected object rather than a Back — which needs a
  second return value from that helper (it already distinguishes the two internally,
  `:904-921`);
- at `:847`, `if len(legs) == 0 { if correctable { return verifyIncomplete }; return verifyAbandoned }`.

Checked against the call graph: the verdict has exactly two production consumers
(`gui/multisig.go:332`, `gui/multisig_build.go:448` — grepped, all other hits are tests),
both of which only branch the loop on it, so returning `verifyIncomplete` here re-offers
under `multisigVerifyRetryLead` ("Not every plate is verified. Try again?") — truthful for
this state — and shows no screen of its own, which is correct because the remedy screen
was just shown. `verifyAbandoned` is referenced by **no test** (grep: only its declaration
and its two return sites), so this change breaks nothing — and equally, nothing would have
caught it, which is B4.

---

## B4 (Important) — I-4's whole retry mechanism is pinned by `strings.Contains` over the caller's source; two different mutations leave the suite GREEN

*(two mutations of one unwatched mechanism, merged.)*

**Defect.** `TestBothEngraveFlowsReOfferTheVerify` is four `strings.Contains` calls over
`funcBody()`. No test ever executes a second verify attempt, and no test ever presses a row
on the retry screen, so the loop's behaviour and its row-to-index mapping are unpinned.

**Site.** `gui/multisig_verify_report_test.go:733-765`, over `gui/multisig.go:325-338` and
`gui/multisig_build.go:441-454`.

**Trigger — two independent mutations, both green:**

1. `for {` → `for mutOnce := 0; mutOnce < 1; mutOnce++ {` at both call sites (every grepped
   substring byte-identical): restores the pre-fold one-shot offer, so the fold's own new
   sentence names a button the operator is never shown. Full suite:
   `ok seedhammer.com/gui 135.902s`, exit 0.
2. `[]string{"VERIFY AGAIN", "CONTINUE"}` → `{"CONTINUE", "VERIFY AGAIN"}` at both sites:
   the loop keys on `sel != 0` positionally (`ChoiceScreen.Choose` returns an index,
   `gui/gui.go:1669-1702`; no label lookup anywhere), so **VERIFY AGAIN now exits the
   verify and CONTINUE re-runs it**. Whole tree: `go test ./... -count=1` exit 0, 0 FAIL.

Corroborating measurements: coverage on the unmutated tree shows `gui/multisig.go:332-337`
and `gui/multisig_build.go:448-453` at execution count **0**; `grep -rn "VERIFY AGAIN"
gui/*_test.go` returns nothing; the only two tests that reach the "Verify the engraved
plates?" screen (`gui/multisig_build_walk_test.go:393`,
`gui/multisig_supply_passphrase_test.go:235`) immediately press **Skip**.

**Minimal fix.** Give the loop a seam and drive it. Resolved against the code:
`multisigVerifyFlow` has exactly two production call sites (grepped above), and the package
already carries in-file test-only seams of exactly this shape (`bip85SeedHook`,
`bip85PkeyHook`, `freetextPlateHook`, `freetextEngraveHook`, `buildMultisigSeedHook`). So:
declare `var multisigVerifyFn = multisigVerifyFlow` beside the flow, have both callers call
`multisigVerifyFn`, and add one test that substitutes a stub returning
`verifyIncomplete` then `verifyComplete`, drives the real `ChoiceScreen` with `click`, and
asserts (a) the offer is drawn a second time carrying `multisigVerifyRetryLead`, (b) the
row the operator presses to retry is the one **labelled** VERIFY AGAIN, and (c) CONTINUE
leaves. That single test kills both mutations. Note the grep test must stay: the seam pins
behaviour, the grep pins the obligation wiring (`engravedSlots` + the ENGRAVED md1), and
neither covers the other.

---

## B5 (Important) — I-12's BUILD-path abort gate is behaviourally inert; only the SUPPLY path is pinned by an executing test

**Defect.** The build path's abort `return` is real and correct in production, but nothing
executes it: replacing its body with a no-op leaves the whole suite green. The identical
mutation on the supply path goes red. The fold's report states the call-site-only pin as a
limitation for *both* paths; measurement says it is a limitation for exactly one — and the
unwatched one is the flow that cuts the most steel.

**Site.** `gui/multisig_build.go:402-404`, against the supply twin at
`gui/multisig.go:291-293` and its behavioural test
`TestSupplyAbortIsTheLastScreenOfTheProgram` (`gui/multisig_verify_report_test.go:639`).

**Trigger.** Keep the `if bundleEngrave(ctx, th, "Build Policy", cardsOut) != bundleEngraveDone {`
line that `TestBothEngraveFlowsGateOnACompletedSet` greps for; make the body a no-op. An
operator who runs out of blanks at plate 12 of 17 on the built-policy path reads *"Bundle
Incomplete … This set is not a usable backup yet"*, is then offered a verify over a set
whose md1 was never cut, and is shown a restore document headed *"This backup is 17 plates
… If any of them is missing, this backup is incomplete."* Full suite with the mutation:
`ok seedhammer.com/gui 107-109s`, exit 0. Supply-side mutation: `--- FAIL:
TestSupplyAbortIsTheLastScreenOfTheProgram`, exit 1. `grep -l "Bundle Incomplete"
gui/multisig_build*_test.go` → no hits.

**Minimal fix.** Mirror the supply-side test on the build path. Checked that the harness
reaches: `gui/multisig_build_walk_test.go` already drives `buildMultisigPolicyFlow` under
`runUI`/`pumpUntil` through "Plates To Cut" → "Choose engraving" → "Verify the engraved
plates?" (`:355`, `:373`, `:393`), which is the same route
`TestSupplyAbortIsTheLastScreenOfTheProgram` walks; press **Back** at the first plate's
style picker to take `bundleEngrave`'s set-level abort, dismiss the modal, then assert the
flow ENDS and that none of `"Verify the engraved plates?"`, `"This backup is"`,
`"Descriptor:"` is drawn afterwards. No new harness is needed; ten of these already exist.

---

# What was REFUTED

**Nothing.** Seven blocking findings entered the two-skeptic refutation pass and seven
survived; the refuted list for this round is **empty**. Recorded so a later reader does not
mistake the empty list for a missing pass.

Carried in from earlier in this cycle, and **not to be reinstated**:

- **R-1 — refuted** (settled before this round; do not re-open).
- **I-8** — ruled **(b)**; the re-decision is made and recorded. Not a finding.
- **`gui/singlesig.go`** — out of scope for this branch. Round 0's N-1 (single-sig has
  I-12's defect verbatim) and N-2 (single-sig hard-codes "Full (seed + keys)") are filed as
  **F-197** and **F-198** with an owning phase of the next cycle's implementation phase.
  They are not blocking findings of this gate and must not be re-raised as such.

---

# Round-0 ledger: are all 18 CLOSED?

**No — 16 of 18 are closed as authored; 2 were MOVED rather than closed.**
The 18 items are C-1..C-3, I-1..I-14, and F-189.

| item | status |
| --- | --- |
| C-1 comparator count asserted before it ran | **CLOSED** (the incomplete branch now runs `verifyMultisigLegsPartial` before it counts) |
| C-2 gate grouped on `SeedID` | **CLOSED in the gate** — but the identity rule was not swept into the sibling the same fold wrote; see B1 |
| C-3 SUPPLY labelled a passphrase build "Full (seed + keys)" | **CLOSED** |
| I-1 `both` slot's engraved origin unpinned | **CLOSED** |
| I-2 the `full` half executed by no test | **CLOSED** |
| I-3 failure screen discarded the diagnosis | **CLOSED** |
| **I-4** "run verify again" prescribed a remedy that did not exist | **MOVED — not closed** |
| **I-5** N per-seed passphrases collapsed into one boolean | **MOVED — not closed** |
| I-6 holding EVERY slot dead-ended | **CLOSED** (one Minor rides along: the Key-sources prose it now reaches still says "The cosigner keys are taken as supplied" on a policy with none) |
| I-7 §0.1a origin announcement stated ONE origin | **CLOSED** (one Nit: singular "the BIP-48 path" over an enumeration) |
| I-8 registry justification false, re-decision never made | **CLOSED** — ruled (b) |
| I-9 no S5 analogue of `TestS0GateHasARecord` | **CLOSED** |
| I-10 "filed rather than smuggled in" did not check out | **CLOSED** |
| I-11 abort screen promised a resume | **CLOSED** |
| I-12 an abort did not propagate | **CLOSED in code on both paths**; the build path's guard is executed by no test — B5 |
| I-13 watch-only verify claimed "secret verified" | **CLOSED** in the string function; the deciding argument at the call site is unpinned (Minor) |
| I-14 "outstanding plates belong to a different seed" | **CLOSED** |
| F-189 retired API | **CLOSED** |

**MOVED, stated plainly:**

- **I-4 is MOVED.** The one-shot offer did become a loop, and that half is real. But I-4's
  own success condition — as written in the fold's own comment, *"so the incomplete
  screen's instruction can be TRUE"* — is not met: obeying the instruction literally can
  never reach Verify OK (B2), the loop is unreachable from three of the five verdicts
  including every first-seed "try again" screen (B3), and no executing test can tell the
  loop from the one-shot it replaced (B4). The remedy exists; the prescription still does
  not describe it.
- **I-5 is MOVED.** The single-boolean collapse is gone, and in its place is the inverse
  error of the same class: one seed reported as two required passphrases, with one
  fingerprint on both lines (B1). C-2's fix re-keyed the gate on `MasterFP` in this same
  fold; the new `passphraseFacts` was left keyed on registry position, so the fold both
  diagnosed the identity mistake and re-committed it three files away.

---

# Minor / Nit — recorded, not gating

None of these holds the gate. Fix inline where cheap; file the rest with an owning phase.

1. **(Minor)** `gui/multisig_build_slots.go:699-701` — I-6 made the zero-cosigner build
   reachable, and the Key-sources review it now reaches still ends "The cosigner keys are
   taken as supplied." on a policy with no cosigner keys. The prose predates the fold; the
   fold made it reachable.
2. **(Minor)** `gui/multisig_verify.go:459` — `multisigVerifyIncompleteText` is the only new
   modal body from this fold with no `assertModalBodyFits` guard; its two siblings from the
   same commit got one. Measured headroom today: 244 / 244 / 201 chars against an 80-char
   margin, so it fits — what is missing is the guard.
3. **(Minor)** `gui/multisig_verify.go:894` — I-13's fix is pinned in the pure string
   function only; changing the sole call site's `full` argument to `true` restores I-13
   verbatim and nothing goes red. Pre-existing call-site gap the new fix now leans on.
4. **(Minor)** `gui/multisig_verify.go:61` — `multisigVerifyRetryLead` ("Not every plate is
   verified. Try again?") is also shown after a **failed** verify (foreign policy,
   comparator failures at `:674`, `:681`, `:868`, `:888`), narrating a failure as an
   incomplete. Suggested: `multisigVerifyRetryLeadFor(res)` with a second string — which
   also gives B4's new flow test something verdict-specific to assert.
5. **(Minor)** `gui/multisig_verify_policy_test.go:224` — dead dismissal needle: the driver
   still dismisses on "cannot prove any of these plates", which after F-191 exists nowhere
   in production (grep: one hit, the needle itself).
6. **(Nit)** `gui/multisig_build.go:1629-1648` — the enumerated origin announcement keeps
   the singular "the BIP-48 path", and the `shWsh` arm reads "derives your keys at @0 at
   m/… and @1 at m/…".
7. **(Nit)** `gui/multisig_build_census.go:183` — `seedFingerprintSuffix`'s `fp == 0` guard
   is unreachable in production (its only zero-fp caller takes the `len(seeds) < 2` early
   return) and unpinned (`if fp == 0` → `if false` stays green).
8. **(Nit)** `gui/multisig_build.go:1670` — `heldSlotOrigins`' bounds/empty guard is unpinned;
   expected for a defensive panic-guard, listed for ledger completeness.
9. **(Nit)** `gui/multisig_verify.go:464` — "Compared against the plates you presented, and
   they match" is a claim about all presented plates while only `len(legs)` were compared;
   the next sentence repairs it, which is why it is a Nit.

---

# Facts this round treated as SETTLED (do not re-derive)

- The five gates pass.
- `go vet` exit 1 / 40 findings is **test-only** and clean.
- I-8 is ruled **(b)**.
- `gui/singlesig.go` is out of scope; F-197 and F-198 are filed.
- **R-1 is refuted.**
- Round 0's own two failed prescriptions (I-6's minimal fix would have moved the dead end;
  I-8's fix (a) would have introduced a Critical) are recorded in
  `design/agent-reports/s5-whole-diff-fold-round0.md:32-64` and were the reason every fix
  above was resolved against the code before being written down.

# What a round-2 re-review should be asked

Scope it to *"did the fold close B1..B5, and did that fold introduce a new defect"* —
not a fresh audit. B1's fix and B3's fix both change code that no current test can fail on,
so the round-2 brief should state that B1's new test (same master, same passphrase, two
held slots) and B4's driven retry test are part of the deliverable, and that their absence
is itself a blocking finding. The build gate covers none of this: it is Go behaviour behind
a UI, so the executing tests named in B4 and B5 are the only mechanism that can.

---

*Round 1. 7 lens findings → 0 refuted → 5 after dedupe. Gate: **RED**.*
