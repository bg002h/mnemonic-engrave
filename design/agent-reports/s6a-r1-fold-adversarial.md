# S6a R1 — adversarial review of the FOLD's new text

**Scope:** only what `git diff b54f7ee..HEAD -- design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
added — §3.2, §4.3 (rewritten), §4.4 (rewritten), §4.7 (new), §5.1, §5.2, §6.2,
§8 items 6–8. Not a fresh audit of the plan or the fork. Old-finding coverage is
another agent's pass and is not duplicated here.

**Fork read at:** `main` = `b8a23bf`.

## VERDICT: RED — 2 Critical, 3 Important

---

### C-1 — "hold the LAST verdict" silently downgrades a DISAGREED to DID NOT COMPLETE

**Where:** §3.2 ("hold the last verdict outside the retry loop and pass it to the
document"), restated as normative design in §4.7 ("the change is to hold the last
verdict outside the retry loop and pass it to `multisigRestoreDocFlow`").

**The defect.** §4.7's status table keys the strongest line on an **event** — "a
comparison disagreed" → `WARNING: a read-back check DISAGREED with these plates.
Do NOT rely on this backup.` The mechanism the same fold prescribes reports a
**final state**. On both multisig paths the retry loop runs more than once, so
the two are not the same thing.

Traced against the real loops (`gui/multisig.go:329-343`, and the identical block
at `gui/multisig_build.go:445-458`):

```
lead, choices := "Verify the engraved plates?", []string{"Verify now", "Skip"}
for {
    sel, ok := verifyChoice.Choose(ctx, th)
    if !ok || sel != 0 { break }
    res := multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1)
    if res != verifyIncomplete && res != verifyFailed { break }   // <- loops on FAILED
    lead = multisigVerifyRetryLead
    choices = []string{"VERIFY AGAIN", "CONTINUE"}
}
```

`verifyFailed` is one of the two verdicts that **keeps the loop alive**. So the
last verdict is very often not the failed one. Two presses reproduce it:

1. "Verify now" → the readback md1 is not this run's policy
   (`!slices.Equal(readbackMd1, engravedMd1)`, `gui/multisig_verify.go:918`) →
   `showError(... multisigVerifyForeignPolicyBody)` → **`verifyFailed`**. Loop
   re-offers with `VERIFY AGAIN` / `CONTINUE`.
2. "VERIFY AGAIN" → the operator presses Back at `bundleGatherFlow` →
   **`verifyAbandoned`** (`gui/multisig_verify.go:696`) → `res` is neither
   incomplete nor failed → **break**.

Last verdict = `verifyAbandoned` → §4.7's table gives
`Plate verification DID NOT COMPLETE. Confirm they restore before relying on this backup.`

The same happens for `verifyFailed` → `verifyIncomplete` (the length precheck at
`:936` returns incomplete without comparing anything new) and for
`verifyFailed` → `verifyRefused` (`:702`, `:920`). In every one of those, a
comparison **did** disagree, nothing later refuted it, and the durable artifact
records a mild advisory instead of `Do NOT rely on this backup`.

**The harm.** This is C-1's own harm, reintroduced by C-1's own remedy, on the
two paths §3.2 pulled into scope specifically to remove it. The stranger reading
the steel in five years cannot tell "the operator stopped early" from "the
machine compared this backup and said it was wrong". `DID NOT COMPLETE` and
`NOT VERIFIED` both end "Confirm they restore before relying on this backup" —
routine housekeeping. The DISAGREED line is the only one that says do not rely on
it, and it is the one that gets erased, by two button presses, with no new
evidence. Single-sig is unaffected (one-shot `if sel == 0`,
`gui/singlesig.go:130-133`, no loop) — this is purely a defect in the multisig
half the fold added.

**Suggested remedy (UNVERIFIED — the supersession rule is an operator call):**
make DISAGREED sticky rather than last-wins: latch `sawFailed` across the loop
and render the WARNING line if it is set. Whether a subsequent `verifyComplete`
may clear the latch is a decision, not a detail — it is the only later verdict
that constitutes fresh proof (incomplete/refused/abandoned each prove nothing
about the plate that failed), and `s6a-c1-verify-tail-decision.md` does not
answer it. Note that the decision doc also says "the last verdict", so the fix is
a specification refinement inside the decided shape, not a re-litigation of it.

---

### C-2 — the cycle's Critical is the only item in the plan with NO test

**Where:** §5 (test table T1–T8), §5.1, §5.2 — all three touched by the fold, none
mentioning §4.7.

**The defect.** §5 opens with a normative rule: *"Every test below must be shown
to FAIL against the unfixed tree… 9 of round 0's 17 blocking findings in S5 were
reproduced by mutating the tree and watching a green suite stay green."* The
table then covers every item in §3: T1/T2 → F-198, T3 → non-vacuity, T4/T7 →
F-195, T5 → F-197, T6 → **F-202, a Minor**, T8 → the false comment.

**C-1 — the Critical — has no T-row, no mutation, and no acceptance criterion.**
This is not an omission the fold missed by not looking: the fold rewrote the test
section twice over. §5.1 added a walk-repair list; §5.2 went through T2, T3, T4,
T5, T7 and T8 one at a time adding per-test refinements. It walked the whole test
plan and added nothing for the section it had just written. §8's blind-spot list
— to which the fold added three entries — does not name it either.

**The harm.** §4.7 states an invariant in the strongest terms available: *"The
restore document **always renders**, and always carries **exactly one**
verification status line. A document with none is a defect, not a default —
silence must never be mistakable for a pass."* Nothing in the plan checks it. The
change spans **three** production document call sites
(`gui/singlesig.go:136`, `gui/multisig.go:361`, `gui/multisig_build.go:478`), and
an implementer who wires two of three ships a green suite and a device that is
silent on one path — which is exactly the state F-197's own follow-up warns
about ("a call-site assertion alone is not enough — that is exactly what let the
multisig instance ship"). It also leaves the highest-consequence mapping in the
cycle unpinned: `singleSigVerifyFlow`'s
`showError(ctx, th, "Verify Failed", "The read-back bundle does NOT match the
seed. Check the engraved plates.")` (`gui/singlesig_verify.go:145`) **must** map to
DISAGREED and not to DID NOT COMPLETE, and no test in the plan would notice if it
did not. C-1 above is a second thing a test would have caught and nothing here
would.

**Suggested remedy (UNVERIFIED):** a T9 with four arms (one per status line),
mutation "return the skipped/NOT-VERIFIED constant unconditionally", asserted
through `s5PageForNeedle` on all three paths — plus one arm proving the DISAGREED
line survives a `failed → abandoned` retry sequence, which is C-1's regression
test. The multisig arms can drive `multisigVerifyFn`, the in-file test seam the
retry loops already dispatch through (`gui/multisig.go:329` comment), so no
engraver or NFC harness is needed for the verdict half.

---

### I-1 — §4.7 names two different seams for one line, and every other section still describes a signature with no room for it

**Where:** §4.7 vs §4.2, §4.3's call-site enumeration, §4.4's placement list,
§5.1(a).

**The defect.** §4.7 says single-sig "threads the outcome (or 'skipped') **into
the inventory**" — i.e. into `buildPlateInventoryLines`. One paragraph later it
says multisig should "**pass it to `multisigRestoreDocFlow`**" — a different
function, one layer out, whose `extra []string` is appended *after* the
descriptor and addresses (`gui/multisig_restore.go:106`). Those are two different
seams producing two different document positions for the same normative line, in
a plan whose entire subject is one string being true in one place and false in
another.

Nothing else in §4 or §5 was updated to admit the verdict at all:

- **§4.2** shows the single-sig call site in full —
  `restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path,
  buildPlateInventoryLines(cards, oneSeedPassphraseFact(passphrase != ""), seedCapacityOne))`
  — with no verdict argument anywhere.
- **§4.3** enumerates the call sites and states what each passes; capacity only.
- **§4.4** gives the inventory's internal ordering as exactly four items
  (plate list → seed statement → passphrase statement → seed-handling ruling).
  The status line is not among them, and §4.4 is the section the fold rewrote.
- **§5.1(a)** says the six existing test call sites "gain **a** capacity
  argument" — singular. If the verdict is a `buildPlateInventoryLines`
  parameter they gain two, and each of the six must choose a verdict, which
  changes what those documents render (measured: 6 test sites, at
  `gui/multisig_build_prose_test.go:369,424,425` and
  `gui/multisig_build_perseed_passphrase_test.go:134,246,304`).

**The harm.** The implementer must invent the seam. Every other item in §4 is
specified down to the argument list; the Critical is specified down to "thread
it". The likely outcome of an invented seam is the two-seams-two-positions
outcome §4.7 literally writes down, so the single-sig and multisig documents
would place the status line differently — which is how the shared-string class
this cycle exists to kill gets recreated at the seam.

Related and unstated: §4.7 says single-sig "gets a result type mirroring
`multisigVerifyResult`'s shape" but gives no return→verdict mapping.
`singleSigVerifyFlow` has **nine** exit points (`gui/singlesig_verify.go:66, 71,
90, 96, 112, 118, 124, 130, 138, 145, 148`), one of which is the failed
comparison. §4.7's four-line table maps verdicts, not returns, and the mapping in
between is where a FAILED becomes a NOT VERIFIED.

**Suggested remedy (UNVERIFIED):** pick one seam and say so — the verdict as a
third parameter of `buildPlateInventoryLines` keeps all three paths rendering the
line in one place and one order, at the cost of a second new argument on the six
test sites; then update §4.2's snippet, §4.3's enumeration, §4.4's ordering list
and §5.1(a) to match, and give the single-sig return→verdict table explicitly.

---

### I-2 — the status line's position is unspecified, and the decided framing model depends on it

**Where:** §4.7, read against §4.4's placement list and §5.2's own pager finding.

**The defect.** The operator decision this section implements says the inventory
lines *"stay too, **framed by** the status line"*. Framing is a position claim.
§4.7 makes none, and §4.4's ordering list — rewritten in the same fold — has no
slot for it.

The default (append last) is the bad one, and the fold established the facts that
make it bad. Measured in the code: `restoreDocScreen`
(`gui/singlesig_restore.go:137`) is a pager whose **Done** button is live on page
one (`doneBtn := &Clickable{Button: Button3, AltButton: Center}`;
`if backBtn.Clicked(ctx) || doneBtn.Clicked(ctx) { return }`), and both doc flows
append `extra` after the descriptor chunks and both addresses. §5.2 says this
itself: *"the inventory lands on the last page(s) — a single-frame assertion
misses it."* Appended last, the status line sits **after** the seed-handling
ruling — a ~330-character paragraph that wraps to many lines — so the sentence
that is supposed to frame *"If any of them is missing, this backup is
incomplete."* can land a page or more behind it, on a screen the reader can exit
at any time.

The codebase has already decided this exact question in the other direction, and
recorded why. `gui/multisig.go:255-280` prepends the slot-collapse note **first**
on the census, with the reason in the comment: *"It leads the list because this
screen is confirmable from any page: a note on page three is a note the operator
can commit past without reading."* That is the same screen family, the same
failure mode, and the plan does not cite it.

**The harm.** C-1's remedy is adjacency — the fold's own words: "a line that
vouches sitting next to evidence that contradicts it". A DISAGREED warning the
reader never pages to is silence, which the decision doc explicitly rejects
("silence can never be mistaken for a pass"). The document would satisfy the
letter of §4.7 while delivering the state it was written to prevent.

**Suggested remedy (UNVERIFIED):** state the position normatively — the status
line first in the inventory block, immediately above `This backup is N plates:`,
mirroring the collapse-note precedent — and have C-2's test assert the page it
lands on, not merely that it is present.

---

### I-3 — the four status strings are asserted to serve all three paths without the per-path audit §4.3 performs, and one of them contradicts §4.4 in the same document

**Where:** §3.2 ("the same four status strings serve all three paths"), §4.7's
table, read against §4.4's presence arms.

**The defect.** §4.3 is the fold's showpiece: it takes one shared sentence,
audits it clause by clause across every mode, finds that "the plates are the
secret" is false on watch-only, and splits the arm. §4.7 writes four brand-new
shared sentences and asserts, in one clause of §3.2, that they serve all three
paths. No audit is shown. Applying §4.3's own method:

**`Plates VERIFIED: each plate was read back and matched the seed.`**

- On a **multisig build across two masters** the verify types more than one seed
  — `gui/multisig_verify.go:906` (*"TYPE THE NEXT SEED"*) — and the shipped
  on-screen success message is scrupulous about it: *"All %d operator key plates
  verified, **and the ms1 you typed for each seed**"*
  (`multisigVerifyOKMessage`, `gui/multisig_verify.go:1057-1059`). The new
  document line collapses that to **"the seed"**, singular.
- It therefore **contradicts §4.4's own several arm** four lines away, in the
  same document, on the same run: *"Seed: this set contains YOUR **seeds**, on
  the **plates** marked 'ms1 secret share'."* One new line says several seeds,
  the other says the seed. This is the §1.3 landmine the plan names, committed by
  the section that names it — the identical shape §3.1.6 was rewritten to
  confess.
- "each plate was **read back**" is also the wrong verb for the ms1 in full mode
  on every path: the ms1 is **hand-typed, never NFC** (`gui/singlesig_verify.go:120`,
  `gui/multisig_verify.go:866`), a distinction the shipped message preserves
  deliberately and this one erases.
- On all three **watch-only** modes the line sits beside §4.4's absence arm —
  *"Seed: this set contains NO seed… no plate in this set holds them."* —
  while announcing that the plates "matched the seed". True as written (the
  comparison seed is re-typed, not read off steel), but it reads as though a seed
  is part of the set, next to the sentence whose whole job is to deny that.

**The harm.** A stranger counting one ms1 plate against a document that says
"seeds" concludes a plate is missing and stops — the exact failure mode §4.4
spends a paragraph preventing, and which
`gui/multisig_build_census.go:110-114` names as the thing the document exists to
prevent. Here the two halves of the same document disagree about the count
outright.

**Suggested remedy (UNVERIFIED):** run §4.3's method over all four status
strings, mode by mode, and either make VERIFIED count-neutral (e.g. "Plates
VERIFIED: every plate in this set was checked against the seed words you
re-entered.") or arm it like the seed statement. Whichever is chosen, the choice
belongs in the plan as an audit table, not as an assertion in §3.2.

---

## Minor / Nit (recorded, not gating)

- **M-1 — §3.2 says `multisigVerifyResult` "already exists with the four outcomes
  the status line needs"; it has five constants.** Measured:
  `verifyComplete`, `verifyIncomplete`, `verifyFailed`, `verifyRefused`,
  `verifyAbandoned` (`gui/multisig_verify.go:85-100`). §4.7, in the same fold,
  correctly calls it "a 5-value `multisigVerifyResult`". The "four" appears to
  have been taken from the type's own doc comment ("FOUR OUTCOMES, NOT A BOOL",
  `:79`) rather than from the constants — precisely the rule §8 item 8 was added
  to confess. No design consequence: §4.7's table folds refused and abandoned in
  with incomplete, so five verdicts plus "skipped" map onto four lines. But §3.2's
  affordability argument is stated on a count that is wrong.
- **M-2 — "The restore document **always renders**" is false on the multisig build
  template path.** `gui/multisig_build.go:474` guards the whole doc block with
  `if !template`, and the fold's own §8 item 7 says so ("the build path skips the
  document entirely"). The operative guarantee (a rendered document always
  carries exactly one status line) survives; the absolute wording does not, and
  an implementer could read it as a mandate to add a restore document to the
  template build path — real scope growth the fold did not price.
- **M-3 — the new watch-only ruling arm is singular after a plural subject.**
  `seedCapacityMany` + no seed on plates assembles to *"…**Every seed** you
  entered -- this build can hold several -- stays in device memory until the build
  ends. Do not leave a mid-build machine unattended: **it is holding your
  seed**."* Understated on a multi-seed watch-only build. Same class as I-3,
  lower stakes (the warning's force does not depend on the count).
- **Nit — the call-site count.** §4.3's "(all 8, measured)" is right for the
  *existing* sites (2 production + 6 test, verified by grep) but the list beneath
  it enumerates nine, and §8 item 4 says "Eight call sites now carry an
  argument" — after the change there are nine. §8 item 4 is pre-fold text and
  outside this review's scope; noted because the fold rewrote the section around
  it.

---

## THE ASSEMBLED DOCUMENT, MODE BY MODE

Card sets measured: single-sig full `[ms1, mk1, md1]`, watch-only `[mk1, md1]`
(`gui/singlesig_engrave.go:20-43`); multisig `[ms1 × distinct seeds, mk1 × held
slots, md1]`, watch-only drops the ms1s (`gui/multisig_engrave.go:32-50`,
`gui/multisig_build_tail.go:86-133`). Cosigner keys are **never** engraved — only
held/matched slots get an mk1 — so `expectedSlots` covers every mk1 in the set.

| # | mode | §4.3 ruling | §4.4 seed statement | §4.7 status line |
| --- | --- | --- | --- | --- |
| A | single-sig **full** | one-seed subject + "the plates are the secret" | 1 ms1 → "contains YOUR seed, on the plate marked 'ms1 secret share'" | any of four; DISAGREED reachable |
| B | single-sig **watch-only** | one-seed subject, plates clauses dropped, "it is holding your seed" | absence arm | any of four |
| C | multisig **build, full** | **byte-identical to shipped** (verified below) | n=1 → singular arm; n≥2 → "YOUR seeds… the plates" | **contradicts §4.4 at n≥2 (I-3)**; **DISAGREED erasable (C-1)** |
| D | multisig **build, watch-only** | many subject + "it is holding your seed" (M-3) | absence arm | **DISAGREED erasable (C-1)** |
| E | multisig **supply, full** | one-seed subject (deliberate churn, §3.1.1) | 1 ms1 → singular arm | **DISAGREED erasable (C-1)** |
| F | multisig **supply, watch-only** | one-seed subject, plates clauses dropped | absence arm | **DISAGREED erasable (C-1)** |

Read as prose, the assembled tail in **mode C with two masters, after a
comparison disagreed on the second leg and the operator then backed out of the
retry**, is:

> This backup is 6 plates: … If any of them is missing, this backup is incomplete.
> Seed: this set contains YOUR **seeds**, on the **plates** marked 'ms1 secret share'. Treat each of those plates as the secret itself.
> \[passphrase lines]
> Seed handling: … Every seed you entered -- this build can hold several -- …
> Plate verification DID NOT COMPLETE. Confirm they restore before relying on this backup.

Two things are wrong there and both are new text. The device said a read-back
disagreed; the record says the check did not finish (C-1). And in mode C with a
clean pass the same document says "YOUR seeds… the plates" in one line and
"matched **the** seed" in the next (I-3).

The reading that is **right**, and worth saying plainly, is the watch-only pair
the fold was written to fix. In modes B, D and F the document now reads *"this
set contains NO seed… no plate in this set holds them"* and, four lines down,
*"Do not leave a mid-build machine unattended: **it is holding your seed**"* —
the machine, not the plates. The self-contradiction §4.3 was rewritten to remove
is genuinely gone in all three watch-only modes. Modes A and E are internally
consistent across all four verify states.

---

## WHAT I CHECKED AND FOUND SOUND

- **§4.3's byte-identity claim — mechanically verified TRUE.** Assembled
  `base + subject(seedCapacityMany) + seed-on-plates` and compared against the
  shipped literal at `gui/multisig_build_census.go:85-90` character by character:
  **equal**. The multisig BUILD path's full-mode document does not churn. Both
  the leading comma placement and the "on a full build" retention (§4.3's
  deliberately-vestigial clause) are exactly what the identity requires — that
  paragraph is not an oversight and is right to keep it.
- **§4.4's two predicates cannot disagree.** The absence arm uses
  `bundleSetCarriesASecret`, the presence arms use an ms1 card count. Traced:
  `bundleSetCarriesASecret(cards) = !bundleShowMs1Reminder(cards)`
  (`gui/bundle_flow.go:482-484`), and `bundleShowMs1Reminder` returns false iff
  any card is `cardMS1` (`:457-464`). So "carries a secret" and "ms1 count ≥ 1"
  are the same set exactly — no unhandled third state, and the abort warning
  still cannot disagree with the document. §4.4's claim survives its own
  second discriminant.
- **§4.4's singular/plural discriminant is safe on plate counts.** Every ms1 card
  is single-plate by construction (`strings: []string{s}` at
  `gui/multisig_engrave.go:36` and `gui/singlesig_engrave.go:26`), so card count
  equals plate count for ms1 and "the plate marked…" can never undercount steel.
  The n=1 fix (R0 I-1) is correct and the reasoning about `numberedLabel` leaving
  a one-leg build unnumbered checks out (`gui/multisig_engrave.go:30-37`).
- **§3.2's fall-through claim is TRUE on both multisig paths.** Supply
  (`gui/multisig.go:330-343`) and build (`gui/multisig_build.go:445-458`) both
  `break` on `!ok || sel != 0`, and the document below is guarded only by
  `if !template` on the build path and by nothing on the supply path. An operator
  pressing CONTINUE after a FAILED does reach `multisigRestoreDocFlow`, and
  `gui/multisig.go:322`'s comment is false in its own file, as §4.7 says.
- **§3.2's affordability claim holds, apart from C-1.** Threading the verdict
  touches two `multisigRestoreDocFlow` call sites, one variable hoisted above each
  loop, and no control flow. The build path's `if !template && len(legs) > 0`
  guard needs the variable declared outside it; `legs` is never empty there
  anyway (`errBuildNoHeldSlot`, `gui/multisig_build_tail.go:131`). No structural
  change is required, so the §3.2 fallback ("named gate on the hardware flash")
  should not be needed. Scope growth risk: low.
- **"Each plate was read back" does not over-claim plate COVERAGE on multisig.**
  I checked the obvious way this could be a Critical — a set containing plates the
  verify never inspects. It does not: `buildEngraveTail` mints one mk1 per **held**
  slot only, `expectedSlots` is the tail's own return, the length precheck
  (`gui/multisig_verify.go:936`) forces every engraved mk1 to be presented, the md1
  is byte-compared against `engravedMd1` (`:918`), and full-mode ms1s are typed one
  per seed. Cosigners' keys are supplied, never engraved, so
  `multisigVerifyOKMessage`'s "Other cosigners' keys are taken as supplied" is
  about policy keys, not about plates in the set. The line's problem is the seed
  count (I-3), not coverage.
- **§5.1(b)'s prescribed repair is correct — VERIFIED against the call graph.**
  `confirmReviewScreen` returns `false` on `backBtn` (Button1) and `true` on
  `contBtn` (Button3 / primary / checkmark) — `gui/multisig_build.go:1730-1734`.
  So `click(&ctx.Router, Button3)` confirms, matching the cited in-tree control at
  `gui/multisig_verify_report_test.go:1009-1013` and the press
  `TestEngraveSingleSigFlowTemplate` already makes through the template
  `confirmReviewScreen` at `gui/template_engrave_test.go:126`. The diagnosis is
  right too: `pumpUntil` only pumps frames, and all three walks would park on the
  census. The `Card 1 of 3` / `Card 1 of 2` non-weakening rule is well founded —
  those are the needles those two tests turn on.
- **§5.2's pager and harness claims check out.** `restoreDocScreen` appends
  `extra` last on both paths (`gui/singlesig_restore.go:131`,
  `gui/multisig_restore.go:106`) and pages with Button2 while Button3/Center exits;
  the single-sig walks do run on a plain `newPlatform()`
  (`gui/singlesig_flow_test.go:53`, `:95`), so T5 genuinely needs no engraver.
  The six existing `buildPlateInventoryLines` test sites are exactly the six §4.3
  lists.
- **Citation gate re-run by me:** `./scripts/plan-cite-check.sh` → `76 / 76 ;
  dangling: 0`, exit 0.

I did **not** re-audit the plan's unchanged sections, the old findings' coverage,
prose quality, or line numbers.
