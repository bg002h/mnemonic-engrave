# S6a R12 — CLOSING REVIEW

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Code:** `/scratch/code/shibboleth/seedhammer`, `main` = `b8a23bf` (working tree clean)
**Question asked:** is there any remaining reason NOT to begin implementation?
**Lenses run:** §4.1–§4.6 fresh audit (G1's fix, never audited); §4.8 + §5 as an
executable sequence; the single-sig half of P5(a); first-commit-red hunting.

## VERDICT: RED — 1 Critical, 2 Important (+ 2 filed)

Baseline re-measured before anything below: `nix develop --command go build ./...`
→ EXIT=0. `TestSupplyPassphraseRunTellsTheOperatorWhatIsMissing` → PASS (42.7s).

---

### C-1 — `passRecord{full, legs}` cannot generate a truthful pass line for BOTH single-sig and multisig; the collision is at the commonest cell [MECHANICAL]

**Where:** §4.7b-seam (the `passRecord` / `verifyRecord` types and
`func buildVerifyStatusLine(rec verifyRecord) string`), consumed by §4.2's
single-sig call site and by §4.8 step 7's "all three flows".

**The defect.** The record has exactly two fields, and **neither of them
distinguishes a single-sig wallet from a multisig one**. §4.7b-seam is explicit
that this is not an oversight to be patched at the call site: *"ONE BUILDER, ONE
INPUT, AND IT IS THE RECORD"*, and §4.2's call site passes `rec`, not a status.
So one function, one record, three flows.

The truthful pass line differs by path, and this codebase has already decided
that it does — in both directions:

```
gui/multisig_verify.go:32
  multisigVerifyOKBody = "Operator key and secret verified. Other cosigners' keys are taken as supplied."

gui/multisig_verify.go:1042-1063   multisigVerifyOKMessage(legs int, full bool)
  ALL FOUR arms end in "Other cosigners' keys are taken as supplied."

gui/singlesig_verify.go:148
  showNotice(ctx, th, "Verify OK", "The engraved bundle matches the seed.")
```

and `gui/multisig_verify_test.go:171` `TestMultisigVerifyNoticeIsHonest` pins the
two halves against each other by name — the multisig notice **must** contain
`"taken as supplied"` and **must not** contain `"matches the seed"`, which is
verbatim the single-sig notice. The tree's own gate says the multisig pass claim
is an over-claim without the cosigner scoping, and that the single-sig wording is
the thing being scoped away from.

**The collision is reachable and is the common case.** Both of the following
produce the identical record `{full: true, legs: 1}`:

| run | why `legs == 1` |
| --- | --- |
| single-sig, Full | `singleSigReadbackCards` (`gui/singlesig_verify.go:23-42`) accepts **exactly one** mk1 and one md1; more than one of either is `ok == false` |
| multisig (build or supply), Full, operator holds one slot | the `legs <= 1` arm of `multisigVerifyOKMessage`; §4.4 of the plan itself names this "the common case: the operator holds one key in a 2-of-3" |

Watch-only collides the same way at `{full: false, legs: 1}`.

**So `buildVerifyStatusLine` is forced to choose, and both arms break a goal:**

- **Include the scoping clause** → the single-sig restore document tells a
  stranger, years later, that this wallet has *other cosigners* whose keys are not
  in the set. **G1 — the device misdescribes what it engraved.** The harm is the
  one §4.4 spends a paragraph naming: a reader who concludes a complete backup is
  incomplete "concludes a plate is missing from a complete set, and stops". The
  same page carries "If any of them is missing, this backup is incomplete", so the
  two lines compound rather than cancel.
- **Omit it** → the multisig restore document asserts a verification with none of
  the scoping the shipped screen was fixed to carry. **G2 — the device claims a
  check it did not perform.** The cosigner keys came from the supplied/assembled
  md1 and were compared against nothing. `TestMultisigVerifyNoticeIsHonest` pins
  the *notice* only, so this ships green.

And the plan's own property forces the worse arm: **P5(a)** says "a claim with no
record is not constructible", so the clause whose generating fact has no record
field gets deleted — which is the G2 arm.

**Why this is in scope, tested against §0.1's guard.** This is not an NG1
increment. §4.7c has *already* committed the pass line to "names exactly the
comparisons this mode ran, **and states what was not read**" — the obligation is
settled design, not a reviewer's addition. The finding is that the chosen input
cannot produce a truthful instance of the line the plan already promises, on one
of the three flows it already promises it on. It is R9's C-1 exactly — a record
too narrow to carry a fact the pass line depends on — applied to the `full` axis
in round 9 and not to the path axis. P6's own wording anticipated it: *"in each
mode the flow supports"*; single-sig's mode space is not multisig's.

**The harm:** funds. Either a recoverable single-sig backup is abandoned by a
reader who believes cosigner plates are missing, or a multisig backup is relied on
whose cosigner keys the device never read and no longer says so.

**Remedy — UNVERIFIED.** The obvious shape is a third discriminant on
`passRecord` (a wallet-shape / "keys taken as supplied" bit written at each
success return, where the fact is in scope). Not resolved against the call graph
here; reproduce the defect, not the remedy.

---

### I-1 — step 7's out-parameter breaks FOUR shipped source assertions the plan says "keep passing untouched" [MECHANICAL]

**Where:** §5's paragraph *"THE RETRY-LOOP CONDITION IS NOT CHANGED, AND THE THREE
SHIPPED TESTS THAT PIN IT NEED NO UPDATE"*, against §4.7b-seam's
`multisigVerifyFlow(..., rec *verifyRecord)` and *"The test seam
`multisigVerifyFn` gains the parameter too"*.

**The defect.** The claim is true of the *condition* and false of the *call*.
Measured:

```
$ grep -rn 'multisigVerifyFn(ctx, th, full, engravedSlots, [a-zA-Z]*Md1)"' --include="*_test.go" gui/
gui/multisig_verify_flow_test.go:373:   "multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1)"
gui/multisig_verify_flow_test.go:394:   "multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1)"
gui/multisig_verify_report_test.go:1079:"multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1)"
gui/multisig_verify_report_test.go:1081:"multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1)"
```

Every one of those needles **includes the closing paren**. Step 7 rewrites the
call to `multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1, &rec)`, so
all four stop matching. `TestBothEngraveFlowsReOfferTheVerify`
(`gui/multisig_verify_report_test.go:1076`) uses each needle **twice** — once
bare (`Contains(body, tc.call)`) and once as `"res := "+tc.call` — so it fails
four times, and `gui/multisig_verify_flow_test.go` fails twice more. There is
also a compile-level site the plan does not name:
`gui/multisig_engrave_tail_walk_test.go:105`, where the seam is assigned a stub
`func(ctx *Context, th *Colors, full bool, expectedSlots []int, ...)`.

**Which goal it breaks:** none directly — it makes §4.8 step 7 unexecutable as
written, and it misdirects the implementer at the moment they hit the red.

**The harm.** §5.1's rule is "UPDATED, NOT WEAKENED", and §5.1's list has two
entries (the six inventory call sites, the three census walks) — neither covers
these. An implementer told in writing that these tests need no update, who then
finds them red, has been set up to *weaken* them: the cheap green is to relax the
needle, and the `res := ` half of `TestBothEngraveFlowsReOfferTheVerify` is the
only executing assertion that the two multisig flows **read the verdict at all**.
Losing it re-opens the S5 defect that test was written for. It also spends a step-7
implementer's budget on a surprise the plan had the facts to predict — §5 already
cites this very test file for the condition.

The correct claim is narrower and is worth writing down: *the retry-loop
CONDITION is byte-unchanged and its three condition assertions hold; the CALL
gains an argument and its four call-text assertions must be re-pinned with the
new argument, not loosened.*

---

### I-2 — §4.8's step/test assignment is not executable: T20 is scheduled where it cannot run, step 7 names three tests that do not exist, and T7c is scheduled nowhere [MECHANICAL]

**Where:** §4.8 rows 2 and 7, against §5.

Three separate, independently confirmed defects in the one section whose job is to
be followed literally:

**(a) T20 is scheduled to step 2, where it cannot land.** Step 2's cell reads
*"`verifyRecord` + `passRecord` + `buildVerifyStatusLine` + **T20, T21, T22,
T26** | pure functions over a record, no callers yet."* But §5 (line 1135) says
**"T20 asserts a status line on the rendered document of all three flows"** —
`gui/singlesig.go:136`, `gui/multisig_build.go:478`, `gui/multisig.go:361` — and
five lines later, **"Pure-function assertions do not satisfy this: *a call-site
assertion alone is what let the multisig instance ship.*"** Rendered documents on
all three flows do not exist until steps 4, 5 and 7. So step 2 cannot be completed
as specified, and the only version of T20 that *can* land at step 2 is precisely
the version §5 says does not satisfy the requirement. T20 is the row that exists
because R1 found the cycle's Critical had no test; §5's own paragraph is titled
*"That is the defect written into its own remedy."*

**(b) Step 7 justifies itself with three test IDs that appear nowhere else in the
plan.** Row 7's cell: *"**T9/T13a/T13b need a rendered document and a multisig
retry loop**, neither of which exists at step 2"*. Measured — `T9`, `T13a` and
`T13b` occur exactly once in the whole document, on that line. They are residue of
a pre-R10 numbering; every other reference in §4.8 and §5 uses T20–T26. Step 7's
row therefore states no true reason for its own contents, and an implementer
reconciling "which rows does step 7 own" has a list (T11, T23, T24, T25) whose
justification names a different, empty set — while T20's document half, which
genuinely belongs there, is filed under step 2.

**(c) T7c is scheduled to no step at all.** §4.8 mentions T8 (step 8), T20/T21/
T22/T26 (step 2) and T11/T23/T24/T25 (step 7). T1–T7 are inferable from step 5.
**T7c is in none of the nine rows**, despite §8 blind spot 4 stating *"So **T7c is
required**: drive each of the three flows to its restore document and assert the
ruling's subject clause matches that path's capacity. Without it, a mis-wired
capacity is invisible."* Blind spot 4 also measures that the build path is guarded
only *by accident* and that **the supply path and single-sig have no guard** — and
§3.1.1 is the section saying the supply path's capacity argument is where this
cycle changes S5-reviewed output. A required row with no owning step is a row that
can be dropped without violating any step of the build order.

**Which goal it breaks:** executability of the build order; (c) has a G1 edge —
a mis-wired `seedCapacity` prints "Every seed you entered -- this build can hold
several --" on a path that holds exactly one, which §3.1.1 calls a falsehood this
cycle exists to correct, and which no compiler and no other test detects.

---

## §4.1–§4.6 — G1's FIX, AUDITED FRESH

Audited as if unreviewed, against the tree rather than against the plan. **No
Critical or Important found in §4.1–§4.6.** Each item below was resolved against
source, not against the plan's own citation.

**§4.1 — the label.** `passphrase` is declared `gui/singlesig.go:64` and assigned
`:72`; the mode picker is `:77-81`. In scope, correct. `buildFullModeLabel`
(`gui/multisig_build_census.go:248-253`) returns the two strings the plan quotes.
The longer row already ships on `gui/multisig.go:217` through the same
`ChoiceScreen`, so `assertChoiceLabelFits` is satisfied by the same evidence.
`TestEngraveSingleSigFlowFull` selects by index (`Button3` = choice 0), so no
existing test breaks and none currently pins the literal — §1.9 measured. ✓

**§4.2 — the inventory.** `cards` is live from `gui/singlesig.go:126`.
`restoreDocFlow` has one production call site and zero test call sites (measured:
`grep -rn "restoreDocFlow(" --include="*.go" gui/` → the definition at
`gui/singlesig_restore.go:119` and `gui/singlesig.go:136` only). The leading-
parameter argument is sound: `restoreDocScreen` (`gui/singlesig_restore.go:137`)
opens `start := 0` and renders from `lines[start]`, so index 0 is page 1 and a
trailing `extra` provably cannot reach it.

**Blind spot 2 ("overflow is not a new risk") is CORRECT, and I checked it rather
than accepting it.** I drove the real supply-path document and dumped its pages:
the shipped seed-handling ruling (≈310 chars, the longest string that will sit
beside the new status line) renders **complete on its own page**, through
`"Power the device off when you are done."` The longest new line
(`statusCheckDidNotPass`, ≈248 chars) is shorter. The status line will take page 1
alone and push `Type:`/`Descriptor:` to page 2, which is what §4.7 wants. No
truncation, no unreachable tail.

**§4.3 — the capacity-keyed ruling.** The byte-identity claim holds. Assembling
`base + subject(seedCapacityMany) + seed-on-plates tail` reproduces
`gui/multisig_build_census.go:86-90` character for character. All six existing
test call sites pass a card set containing `cardMS1`
(`gui/multisig_build_prose_test.go:365-369, 421-425`;
`gui/multisig_build_perseed_passphrase_test.go:~130, 243, 300`), so every one takes
the seed-on-plates arm and every existing assertion — `"on the plates"`,
`"unattended"`, `"Every seed"` — survives step 3 unchanged. The source assertion in
`TestSeedResidencyRulingDescribesTheMultiSeedReality` looks for
`"holds exactly one seed"`; the new subject string is `"holds exactly one --"`,
which does not match. **Step 3 lands green.** ✓

`buildPlateInventoryLines` call sites measured: 2 production + 6 test = 8 existing,
9 after §4.2. §4.3's header "all 8, measured" with a 9th flagged `(new)` is
consistent with §8 blind spot 4's "nine call sites"; not a finding.

**§4.4 — the seed statement.** The ms1-card-count discriminant is right and the
label claim checks out: single-sig labels the card exactly `"ms1 secret share"`
(`gui/singlesig_engrave.go:25`), multisig at n=1 yields the same via
`numberedLabel` (`gui/multisig_engrave.go:63-68`), at n>1 the numbered prefix. Card
count equals plate count for ms1 on both paths (`strings: []string{...}`, one
element, both sites), so "the plate marked" / "the plates marked" is never off by
one. Every new string is ASCII: the presence arms use `'` (U+0027), which is not in
`gui/multisig_build_prose_test.go:395`'s reject set. The `bundleSetCarriesASecret`
reuse (`gui/bundle_flow.go:482`) genuinely is one definition shared with the abort
warning, so the two cannot disagree. ✓

**§4.5 — the abort gate.** `bundleEngrave` returns `bundleEngraveDone` only after
the full plan is cut (`gui/bundle_flow.go:433`); the mnemonic scrub is a `defer`
registered at `gui/singlesig.go:50`, so the early `return` still scrubs. The
comment correction is real: `bundleEngrave` has four production call sites and
three carry a post-engrave tail, of which two gate — §1.5's table is accurate. ✓

**§4.6 — the pre-engrave census.** The one thing I expected to find here is not
there. Reusing the literal `"Plates To Cut"` at a second production site does
**not** break the needle gate: `buildFlowNeedles` pins `"Plate Count"`
(`cmd/emu/needle_test.go`), not `"Plates To Cut"`, and `"Plates To Cut"` is in no
`decoyNeedles` entry, no `contentNeedles` entry, and no `NEEDLE_*` declaration in
any `cmd/emu/walk_*.js`. Its only other uses are `pumpUntil` needles inside four
**multisig** Go walks, which a single-sig production site cannot reach. No
emulator walk drives `engraveSingleSigFlow` (checked all five `walk_*.js`; the
"single" hits are `single-site`, `single-slot`, and a comment). §3.1.5's choice is
safe. ✓

**§5.1(b) is complete and its citations are exact.** `gui/singlesig_flow_test.go:82→83`
(`Card 1 of 3`), `:121→122` (`Card 1 of 2`), `gui/template_engrave_test.go:128→129`
(`Card 1 of 3`). Exactly four tests call `engraveSingleSigFlow` and the fourth,
`TestEngraveSingleSigFlowSeedScrubbed` (`:141`), backs out at the wallet-type
picker (`click(Button1)` after `pumpUntil("Wallet Type")`) and never reaches the
engrave — so three is the count, as claimed. ✓

---

## BUILD ORDER — EXECUTABLE?

**Not as written.** Three defects, all in I-1/I-2 above, plus three Minors.

The *ordering logic* is sound and I could not break it. Steps 1–4 leave the tree
green; the 5+6+7 bundle argument is correct and the reason given for it — that
5+6 without 7 is "green AND landable AND exactly C-1's harm" — is the right reason.
Step 4 leaving the two multisig documents with an empty status line is not a
regression (they have none today), and T20 at step 7 catches a leftover `""`.

What is not executable is the **test-to-step assignment** (I-2) and the **claim
that step 7 needs no existing-test updates** (I-1).

**Is step 1 reviewable when produced?** Marginally, and one wording change would
fix it — see M-1. The eleven exits are real: `singleSigVerifyFlow`
(`gui/singlesig_verify.go:65-149`) has ten explicit `return`s (`:69 :78 :90 :98
:112 :117 :125 :130 :138 :146`) plus the fall-through past
`showNotice(...)` at `:148`. §4.7b's fifteen-row multisig table gives the reviewer
a worked criterion to apply. What step 1 lacks is a stated acceptance criterion of
its own, and its one-line brief points at the wrong vocabulary.

---

## THE SINGLE-SIG HALF

This is where C-1 lives, and the answer to the question as posed —
*"does P5(a) actually hold there, or does it only hold on multisig?"* — is:

**P5(a) holds *mechanically* on single-sig and fails *semantically*.** The record
is written at the eleven sites where the facts are in scope, the pass line is
built from the record, and a claim with no record is not constructible — all four
of those are satisfied. What fails is that the record was designed against
multisig's success return, and single-sig's differs in a way the record cannot
express (C-1). P5(a) is a construction rule; it guarantees claims come *from* the
record, not that the record is *adequate*. On single-sig it is not.

Two further things I checked and cleared:

- **`template` is not a missing record field.** Single-sig offers verify even for
  a template engrave (`gui/singlesig.go:131-133`), where multisig build skips it
  (`gui/multisig_build.go:464`). But `verifySingleSig` compares the same three
  legs either way, and the pass line names *which plates were compared*, not what
  they contain — so no clause changes. §8 blind spot 7 already owns the related
  label defect. Not a finding.
- **A skipped verify is structurally safe.** `rec` stays zero, the `default:` arm
  is the zero cell, `statusNotFullyChecked`. The plan does not spell out where
  `rec` is declared in `engraveSingleSigFlow`, but the `switch` makes the omission
  unfalsifiable in the safe direction. Not a finding.

The coverage script's declared blind spot ("single-sig contributes zero sites
until `singleSigVerifyFlow` gains a verdict at build-order step 1") is honestly
stated and remains true; nothing in §4.7b claims otherwise.

---

## MINOR / NIT

**M-1 [MECHANICAL] — step 1 asks for the wrong artifact.** §4.8 row 1: *"Write the
single-sig exit → `verifyStatus` mapping (§4.7c)"*. Under the four-state design an
exit does not map to a `verifyStatus`; it maps to a **record write** — set
`adverse`, write `pass`, or neither — and `verifyStatus` is derived *inside*
`buildVerifyStatusLine`. §4.7c is the four-lines section and classifies no return
site; §4.7b is. Mapping an exit straight to a status is the shape round 9
replaced, and it is where monotonicity stops being structural. One-word fix, but
it is the brief for the plan's only delegated decision, so it should be right
before it is handed over.

**M-2 [MECHANICAL] — "the three false comments (§4.7c)" enumerates none, and only
two exist in the plan.** §4.8 row 8. §4.7c contains no comment corrections. §1.5
identifies `gui/bundle_flow.go:535` ("both engraving callers"); §3.2 identifies the
second in passing — confirmed at `gui/multisig_verify.go:78`, *"FOUR OUTCOMES, NOT
A BOOL"* over five constants (`gui/multisig_verify.go:88-100`). The third is never
named anywhere. T8 covers only the first.

**M-3 [MECHANICAL] — §4.5 and §4.8 disagree on which step lands the
`bundle_flow.go:535` correction.** §4.5: *"is corrected in the same change"* as the
abort gate (step 5). §4.8 row 8: *"deliberately last so it cannot mask a
behavioural regression"* (step 8). Both are defensible; the plan must pick one.

**M-4 [MECHANICAL] — §4.7f's scope line has no carrier.** §4.7b-seam fixes
`buildVerifyStatusLine` at `string` — *"exactly one line"* — and §4.2 at a single
`status string` parameter, but §4.7f requires a second line that *"renders
immediately after the status line"* under one cell. Either it is concatenated into
the one string (consistent with §4.7c's `statusVerifiedOnRetry` row, which does
exactly that) or the signature is wrong. T11 passes either way; the plan should
say which. Rendering is fine either way — see the pager measurement above.

**N-1 — §5 cites "the §4.7d table" three times for the status lines, which are in
§4.7c.** §4.7d has a table of its own (the membership test), so an implementer told
to "compare the entire string against the §4.7d table" is pointed at the wrong one.

---

## FILED — TRUE BUT OUT OF SCOPE

**F-a — `gui/singlesig_verify.go:148` is mode-blind on the success screen.**
`showNotice(ctx, th, "Verify OK", "The engraved bundle matches the seed.")` fires
on watch-only runs too, where `verifySingleSig` drops the derived ms1
(`gui/singlesig_verify.go:51-55`) and no seed comparison occurs. This is the exact
string `TestMultisigVerifyNoticeIsHonest` (`gui/multisig_verify_test.go:184`) calls
"the over-claim", and it is R9's C-1 class one surface over.
**FILE — already declared verbatim as §8 blind spot 8**, and NG2 puts screen
diagnosis outside this cycle. Recorded so it is not re-discovered as new; it is
also the corroborating evidence for C-1 and should be re-read whenever C-1 is
folded.

**F-b — the watch-only document says "This backup is N plates" about a set the
operator was told to add a plate to.** `bundleEngrave` shows the ms1 hand-engrave
reminder on every seedless set (`gui/bundle_flow.go:430-432`), so a watch-only
operator ends up holding N+1 plates while the document names N and §4.4's absence
arm says "no plate in this set holds them". Truthful about what the *device*
engraved; potentially confusing about what the *reader* is holding.
**FILE — out of scope under §0.1.** It is pre-existing and unchanged on both
multisig paths; §4.2 only extends it to the third path; and closing it means the
document reporting on an action the device did not perform.

---

## WHAT I DELIBERATELY DID NOT DO

No codebase audit for pre-existing defects. No prose, heading or markdown review.
No remedies resolved against the call graph — C-1's suggested shape is marked
UNVERIFIED on purpose. I did not re-derive the five clean gates, the 15/15
return-site sweep, the 96/96 citations, the 56/56 glyphs, or the adverse/benign
spot-checks; every citation I *did* touch, I resolved independently, and all of
them held.

**One honest note on the closing question.** Ten of the eleven prior rounds found
the design sound, and I looked hard for a reason to say GREEN. C-1 is not a
refinement of the epistemic apparatus — it is the same defect R9 closed on the
`full` axis, still open on the path axis, and the shipped tree contains the
evidence for it in two places. I-1 and I-2 are one grep and one grep apart. I do
not believe any of the three is manufactured, and I would not have filed the
Minors on their own.
