# S6a R10 — GOAL CONFORMANCE REVIEW

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md` (1347 lines)
**Code:** `/scratch/code/shibboleth/seedhammer`, `main` = `b8a23bf`
**Lens:** does the plan deliver **G1** and **G2** as stated in §0.1, and does it
smuggle **NG1** back in? Nothing else.
**Date:** 2026-08-16

---

## VERDICT: RED — 1 Critical, 1 Important (+ 3 filed out-of-scope)

Plus 4 Minor, 1 Nit, and 6 named NG1-residue items.

**The shape of the result matters more than the count.** G1 is delivered in every
mode I could reach — the label, the inventory, the passphrase statement, the seed
statement and the abort gate all land, and I found no surviving misdescription of
*what was engraved* on any of the six mode/path combinations. **G2's prohibition
also holds structurally**: the zero cell is genuinely the default, the adverse bit
is genuinely sticky, and I could not construct a reachable path where the document
asserts a *verification outcome* stronger than the two recorded bits.

Both blocking findings are in the **same seam**: the one place where a recorded
fact has to survive the trip from the verify flow to the page. §4.7b-seam declares
that seam, and the type it declares throws away the thing §4.7c says it carries.

---

## C-1 — THE PASS RECORD IS A `bool`, SO THE "GENERATED" PASS LINE CANNOT BE GENERATED  [MECHANICAL]

**Where:** §4.7b-seam (plan `:820-824`), §4.2 (plan `:491-493`), §4.7c (plan
`:851-859`). Against `gui/multisig_verify.go:986`, `gui/multisig_verify.go:1042-1063`.

**The defect.** Three sections of the plan specify this seam and they do not agree,
and the two that carry *code* both discard the mode.

1. §4.7b-seam declares the carrier:

       type verifyRecord struct {
           fullPass bool   // written AT the success return, with `full` in scope,
                           // carrying WHICH comparisons ran and matched in this mode
           adverse  bool   // sticky; written at any adverse site per 4.7b
       }

   The doc comment asserts the field carries *which comparisons ran in this mode*.
   **A `bool` carries one bit.** Measured, the success return has strictly more
   than one bit in scope — `gui/multisig_verify.go:986` is
   `showNotice(ctx, th, multisigVerifyOKTitle, multisigVerifyOKMessage(len(legs), full))`,
   i.e. `(legs int, full bool)`. `fullPass bool` discards `legs` outright and
   conflates `full` with "a pass happened".

2. §4.2's call site discards even that bit:

       restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path,
           buildVerifyStatusLines(status),
           buildPlateInventoryLines(...))

   `status` is the identifier §4.7a assigns the **`verifyStatus` enum** to
   (plan `:765-768`). So on the plan's own reading, `buildVerifyStatusLines` takes
   a four-valued enum. The plan defines `status` nowhere else; grep confirms
   `buildVerifyStatusLines` appears at plan `:492`, `:859`, `:955`, `:1019` and is
   given an argument exactly once — `status`.

3. §4.7c then says the opposite: *"`buildVerifyStatusLines` takes the **pass
   record**, which carries the mode."*

**Two of the three statements are code and both lose the mode; one is a comment.**
An implementer implements the code.

**Reproduced, not remedied.** With `buildVerifyStatusLines(status verifyStatus)`,
the `statusVerified` cell has no input but the cell itself, so its line must be a
**literal**. §4.7c supplies no literal for that row — the cell reads
*"generated from the pass record"* — so the implementer invents one. The shipped
sibling text for the identical fact is mode-branched in **all four** of its arms
(`gui/multisig_verify.go:1042-1063`), and its own comment records that the
single-leg arm *"ignored `full` and returned the secret-claiming body
unconditionally"* — I-13, already found and fixed once in this codebase. A
mode-blind document literal is that bug, re-committed onto the durable artifact.

**Which goal it breaks:** **G2**, second clause — *"never claims a check it did
not perform."* This is R9's C-1 restored by the type declaration in the section
written to close it.

**The harm.** On a **watch-only** run no ms1 is created, requested, typed or
compared (`gui/singlesig_verify.go:120-142`: the hand-type block is inside
`if full`; `verifySingleSig` drops the derived MS1 so `bundle.Verify` skips the
leg). A mode-blind pass line asserts a seed comparison on that run. The same page,
four lines down, carries §4.4's absence arm — *"this set contains NO seed"*. The
document contradicts itself about the single fact it exists to settle, and the
reader who believes the pass line concludes the steel is the whole backup and
never hunts for the separately-held words. That is the funds path.

**Note the trap in the obvious repair.** `full` *is* in scope at all three
document call sites (`gui/singlesig.go:86`, `gui/multisig.go` step (5),
`gui/multisig_build.go:377`), so passing it alongside the enum compiles and looks
right. It is exactly what §4.7g's P6 forbids — *"a mode guard says this record
entitles this clause, it never infers a fact"* — and it does not reach `len(legs)`
at all, which `multisigVerifyOKMessage`'s multi-leg arms need. **UNVERIFIED as a
remedy; recorded only so the fold does not take it.**

---

## I-1 — THE STATUS REACHES THE PAGE AS `[]string`, WHOSE ZERO VALUE IS SILENCE — AND §5 PINS ONLY TWO OF THE THREE DOCUMENT FLOWS  [MECHANICAL]

**Where:** §4.2 (plan `:483-487`), §4.7c (plan `:843-849`), §5 (plan `:1086-1092`),
§4.8 step 7 (plan `:960`).

**The defect, in two halves that compound.**

*Half one — the safe zero value is discarded at the seam.* §4.7c makes a point of
it: `statusNotFullyChecked verifyStatus = iota  // THE ZERO VALUE. Safe, and true.`
That property is the whole of §4.7a's structural-monotonicity argument. But the
parameter that actually reaches `restoreDocScreen` is `status []string`, and a
slice's zero value is `nil` — **no status line at all**, not the safe line. The
one type in the design whose zero value is safe is converted, at the seam, into a
type whose zero value is silence.

*Half two — nothing tests the third flow.* Measured, there are exactly **three**
production document flows:

    gui/singlesig.go:136        restoreDocFlow          (new, §4.2)
    gui/multisig.go:361         multisigRestoreDocFlow  (supply)
    gui/multisig_build.go:478   multisigRestoreDocFlow  (build, gated on !template)

§5 opens by naming this exact hazard — *"round 1 answered it with tests that would
all still pass if the status line were wired into two of the three document flows
and forgotten on the third"* — and then specifies: *"At least T11 and one of
T10/T12 drive a real flow to a real restore document."* **That is two of three.**
The paragraph states the defect and prescribes it.

Nothing else closes it. T20's assertion *"every rendered document carries exactly
one"* has the mutations *"return the same string for two cells; return an empty
slice"* — both mutations of the pure function, so T20 cannot see a call site that
passes `nil`. T7c drives all three flows but asserts the **seed-handling subject
clause**, not the status line. §4.8 step 4 makes the parameter mandatory, so the
compiler forces each site to pass *something* — and `nil` is something.

**Which goal it breaks:** **G2**. The plan itself rates a status-less document as
the Critical's harm, verbatim, at §4.8: *"Steps 5+6 without 7 are green AND
landable AND are exactly C-1's harm: a restore document carrying a full inventory
and completeness claim, with no verification status line on it."* §4.8 makes that
state unreachable **at landing time** by forcing steps 5+6+7 into one commit. It
does not make it unreachable by a later edit, and the settled invariant is *"the
document always renders and carries exactly one status line; never gated."*

**The harm.** One of three paths ships the inventory and the completeness sentence
— *"This backup is N plates … If any of them is missing, this backup is
incomplete."* — with no verification claim beside it, and no test goes red. That
is the pre-cycle state on that path, which is what C-1 is.

*(Rated Important, not Critical: step 7 does say "wire the verify status into all
three flows", so the plan directs the correct behaviour. What is missing is
anything that would notice if it did not happen — and §5's own diagnosis is that
"a call-site assertion alone is what let the multisig instance ship.")*

---

## G1 — WALKED, MODE BY MODE

I drove each mode against the real functions. G1 has three surfaces — the **label**
(before pressing), the **census** (before cutting), the **document** (years later)
— plus the **abort gate**. Nothing about *what was engraved* is misdescribed in
any of them.

| mode | label | document: inventory / seed / passphrase | verdict |
| --- | --- | --- | --- |
| single-sig full, no passphrase | `buildFullModeLabel(false)` → `"Full (seed + keys)"` | 3 cards; presence arm (1 ms1 card); `"No BIP-39 passphrase was used…"` | **true** |
| single-sig full, **passphrase** | `"Full (seed + keys, NOT passphrase)"` | presence arm; `"A BIP-39 passphrase WAS used. It is not on these plates and cannot be recovered from them"` + `"Without it, these plates do not reach the money."` | **true — the missing factor is named, on the label AND on the page** |
| single-sig watch-only, no passphrase | `"Watch-only (keys)"` | 2 cards; absence arm `"this set contains NO seed … no plate in this set holds them"`; no-passphrase arm | **true** |
| single-sig watch-only, **passphrase** | `"Watch-only (keys)"` | absence arm **and** the WAS-used arm together | **true** (the label omits the passphrase, but watch-only omits the seed too, so the label over-claims nothing) |
| multisig SUPPLY, either mode | already correct (`gui/multisig.go:217`) | inventory already correct; gains the seed statement and the **corrected** one-seed capacity clause | **true, and one shipped falsehood is repaired** |
| multisig BUILD, either mode | already correct (`gui/multisig_build.go:373`) | byte-identical on full (verified below); watch-only loses the false `"the plates are the secret"` pair | **true** |

**The passphrase question, answered directly: yes.** A passphrase single-sig build
now gets a document that names the missing factor, on the success path. The chain
is `gui/singlesig.go:64` (`passphrase` in scope) → §4.2's
`oneSeedPassphraseFact(passphrase != "")` → `buildPlateInventoryLines` →
`buildPassphraseInventoryLines` (`gui/multisig_build_census.go:121-142`) → the
two WAS-used lines. Both ends resolve.

**Byte-identity of the reassembled seed-handling ruling: verified by machine, not
by eye.** `seedCapacityMany` + the seed-on-plates arm reproduces
`gui/multisig_build_census.go:86-90` exactly:

    byte-identical: True

So §4.3's central claim — the BUILD path's full-mode document does not churn —
holds, and the two documents that *do* change are the two the plan says are wrong
today.

**Two G1 gaps I could not close, both already owned by the plan.**

- **`restoreDocFlow`'s two error returns (§8.6).** `gui/singlesig_restore.go:122`
  and `:127` `showError` and `return` **before** `restoreDocScreen`. On either
  error a fully-cut set produces **no document at all** — hence no plate count, no
  seed statement, and no passphrase statement, which is the half F-198 is Critical
  for. Reachability confirmed low: all four `md.ScriptKind` values map
  (`gui/singlesig_restore.go:42-55`), the xpub is device-derived, and package
  `address` carries a schnorr path so P2TR renders. `multisigRestoreDocFlow:103`
  has the identical shape today. The plan names it, bounds it, and gives a
  disposition. **Not re-rated.** One propagation note: §8.6 was written when only
  `extra` existed and now the **status** is dropped on the same path too — no
  document renders, so no over-claim, but the section's text is stale.
- **The template single-sig branch (§8.7).** See M-3.

**The abort gate.** `bundleEngraveDone` means *every plate in the plan was
engraved* (`gui/bundle_flow.go:445-446`). §4.5's guard at `gui/singlesig.go:127`
is the same guard the two multisig callers already carry
(`gui/multisig.go:291`, `gui/multisig_build.go:402`), and
`gui/bundle_flow.go:39` `return`s on the next line with nothing after it to vouch.
G1's *"an aborted set produces no document at all"* is delivered on all four
callers.

---

## G2 — WALKED, CAN THE DOCUMENT STILL OVER-CLAIM?

**Setting C-1 aside, the prohibition holds.** I walked every route I could reach
from a rendered document back to what the device recorded, and found no path where
the *verification outcome* asserted exceeds the two bits. The four-state design
does what §0.1 claims for it: it fails safe by construction rather than by
enumeration.

**The zero cell really is the default, and it catches a live case the plan did not
name.** `gui/multisig_build.go:464` gates the verify offer on
`if !template && len(legs) > 0`, while the document is gated on `if !template`
alone. So a full-policy build with **zero legs** renders a document having *never
offered a verify at all*. Neither bit is written → `default:` →
`statusNotFullyChecked` → *"These plates were not fully checked. Confirm they
restore this wallet … before relying on this backup."* True, conservative, and it
required no one to have thought of that path. Under the retired six-state
apparatus this is precisely the shape that failed open.

**Paths walked, all safe:**

| path | recorded | cell | over-claim? |
| --- | --- | --- | --- |
| verify Skipped (`sel != 0`) | neither bit | `statusNotFullyChecked` | no |
| verify offer Back (`ok == false`) | neither bit | `statusNotFullyChecked` | no |
| build path, `len(legs) == 0` — verify never offered | neither bit | `statusNotFullyChecked` | no |
| single-sig mismatch `gui/singlesig_verify.go:146` | adverse | `statusCheckDidNotPass` | no |
| partial verify, everything compared matched (`:979`) | neither (correctly **not** a full pass) | `statusNotFullyChecked` | no |
| plate-count mismatch (`:738`), then CONTINUE | adverse sticky | `statusCheckDidNotPass` | no |
| plate-count mismatch, then a clean full pass | adverse + fullPass | `statusVerifiedOnRetry` | no |
| clean pass first attempt | fullPass | `statusVerified` | **only via C-1** |
| template build | no document renders (`if !template`) | — | no |

**The classification I re-read rather than inherited** (the cite gate proves
existence, not interpretation). All four spot-checks are correct readings:

- `:738` `return verifyIncomplete` — *"Read back N key plates, but this run
  engraved M"* — a **verdict of Incomplete** that the plan classifies **adverse**.
  Correct: this is evidence about the plates, and reading the verdict instead
  would have got it wrong. This row is the clearest demonstration that §4.7's
  "no verdict is read" is load-bearing rather than stylistic.
- `:897` `return verifyFailed` — *"Couldn't re-derive the bundle from the seed"* —
  a **verdict of Failed** classified **benign**. Correct, and the mirror image of
  the row above: nothing was observed about the plates.
- `:794` `return verifyRefused` — `verifyFreshSlots` error — benign. Correct.
- `:394` `errVerifyLegHasNoPlate` — a leg whose mk1 matched no presented plate —
  adverse. Correct. (It is not itself a `return verify*` site; it reaches `:963`
  or `:984`, both already adverse, so the row is redundant but harmless.)

**Coverage arithmetic, recounted:** 15 `return verify*` sites in
`gui/multisig_verify.go` (measured). Adverse column contributes 6 (`:701 :719
:724 :738 :963 :984`), benign column 8 (`:670 :680 :696 :794 :897 :938 :940
:979`), plus `:987` named separately as the success return = **15**. Complete.

**Where an over-claim would still have to come from.** Only two mechanisms can
strengthen a claim: writing `fullPass` at a site that is not a clean pass, and
generating a pass line that names a comparison the record does not hold. The first
is guarded — one site, `:987`, with `full` in scope. The second is **C-1**, and it
is open.

**One structural gap the plan does not close, and it is the path the cycle is named
after.** §4.7b's table covers multisig only, and the single-sig eleven-exit mapping
is deferred to build-order step 1 — *"a gate, not a task"*, reviewed before step 2.
Ten of those eleven exits fail safe by the zero cell. The eleventh
(`gui/singlesig_verify.go:148`, the fall-through success return) is the one that
can over-claim, and it is also where `full` lives. See M-4.

---

## NG1 RESIDUE

§0.1 lists NG1's symptoms by name: *"Six knowledge states, per-observation
world-sets, a monotonicity property, an enforcement artifact, a coverage script."*
Four of the five have survivors. None of them is a G1/G2 defect; all of them are
dead weight, and one of them is holding a state in.

**R-1 — five property citations, zero definitions. This is the real residue.**
Machine-checked across the whole plan: `P1`, `P2`, `P5(a)`, `P5(b)`, `P5(c)` are
**referenced and never defined**. `P4` survives only as a corpse in §0.1. The
properties section that defined them was deleted in the round-9 rewrite; the
citations were not. Three of them are load-bearing:

- §4.7d holds the fourth state in on **"P2 forbids the merge"** — plan `:881`,
  in a row whose own consumption column reads **"action identical"**. So the one
  state whose stranger-facing action §4.7d concedes is identical to another's is
  retained by an undefined property.
- §4.7d `:887`: *"a sticky adverse bit violates **P1** … a non-sticky one violates
  **P2**"* — the whole two-state-collapse argument rests on two undefined terms.
- §4.7b `:797` and §4.7b-seam `:835` both invoke **P5(a)** as a requirement.
- **T24's title is "P2 on the retry path"** (plan `:1043`), so an undefined
  property has propagated into the test table.

By §0.1's own membership test — *does a stranger need it to answer "is this
everything?" or "can I trust it?"* — the on-retry line (*"An earlier check did not
pass; a later full check passed."*) reports the **device's epistemic history**, and
§4.7d states the action does not change. That is NG1's definition exactly. **The
count is settled and I propose nothing**; what I am naming is that the surviving
justification for it is a reference to a property the plan no longer contains.

**R-2 — T26 / P6 is a review obligation wearing a test's clothes.** Plan `:1045`:
*"For each clause of each pass line, in each mode, a recorded observation is
named; a clause with none is deleted"*, mutation *"add an unbacked clause to a
pass line"*. No Go assertion can detect that a clause is *unbacked* — "backed" is
a property of the reviewer's reasoning, not of the output string. Meanwhile **T22
already tests the one concrete instance** (ms1 clause absent on watch-only, present
on full) with a mutation a compiler-adjacent test really can catch. P6 is §0.1's
*"an enforcement artifact"* surviving under a new number, and T26 is the row most
likely to be satisfied vacuously.

**R-3 — the vocabulary §4.7 says it deleted is the vocabulary it uses.** Plan
`:752` lists what the rewrite removed: *"No … **world-set table** to keep
complete."* Plan `:760-761` then defines the surviving bit as *"Written at any
return site whose **world-set** contains a bad-plate world"*, and plan `:783`
heads the adverse column *"adverse (**world-set** contains a bad-plate world)"*.
The one classification that survived is specified in the retired ontology's terms.

**R-4 — "monotonicity" survives as a word.** Plan `:774` and T21 (`:1040`). §0.1
lists *"a monotonicity property"* as an NG1 symptom. Here it is genuinely
structural (`default:` is the safe cell), so the mechanism is fine — the name is
vestigial and invites a future reader to look for the property that defined it.

**R-5 — the coverage script survives, and this one is the price of the settled
count, not removable residue.** §0.1 names *"a coverage script"* as an NG1 symptom;
`./scripts/verify-returnsite-sweep.sh` exists and §4.7b leans on it. It is what the
`adverseRecorded` bit costs: a *prohibition* needs one conservative default and
**no enumeration** (§0.1), but the adverse bit needs a complete partition of every
return site. Stated plainly so the trade is visible: **states 2 and 3 are what buy
back the enumeration §0.1 says a prohibition does not need.** The operator ruled
four states; the enumeration is the bill. It fails safe (a missed site
under-claims), so it is not a defect — it is the residue that cannot be removed
without re-opening a settled decision.

**R-6 — §4.7d's two-prong membership test.** *"why four and not two, six, or
seven"*, ENFORCEABILITY × CONSUMPTION, five rows. This is obligation-over-a-
partition machinery: it argues the epistemic partition is complete and minimal.
Under G2-as-prohibition no such argument is owed. It is harmless as a record of
the operator's ruling, and it is the section a future reviewer will mistake for a
standing obligation.

**What is NOT residue, checked and cleared:** the status line itself (G2 requires
one), the adverse/benign split (§4.7d's consumption prong genuinely holds —
*"confirm before relying"* vs *"do not rely until a check passes"* are different
actions, and G2's first clause literally names evidence-against), §4.7f's scoping
line (G2: stop the page below from being read as vouched), T25 (enforces "read no
verdict", which is G2's mechanism), and the generated pass line's *"states what was
not read"* clause (the shipped screen already scopes itself the same way —
`"Other cosigners' keys are taken as supplied."`).

---

## MINOR / NIT

**M-1 — `rec`'s declaration site is unspecified, and the retry loop is where it
matters.  [MECHANICAL]** §4.7b-seam gives the flow `rec *verifyRecord` but never
says where the caller declares it. Both multisig retry loops
(`gui/multisig.go`, `gui/multisig_build.go`) call `multisigVerifyFn` **inside**
`for { … }`. A `rec` declared inside the loop resets `adverse` on every attempt, so
*adverse → abandon* degrades to `statusNotFullyChecked` and *adverse → clean pass*
degrades to bare `statusVerified`. Fails **safe** in the first case (a weaker line,
not a stronger one) and is caught by T23/T24 in both — hence Minor, not Important.
Worth one sentence in the plan because the reader who writes step 7 will not have
§4.7d's stickiness argument in front of them.

**M-2 — §4.3 says "all 8" and lists 9.  [MECHANICAL]** Measured:

    grep -rn "buildPlateInventoryLines" --include="*.go" gui/   # 8 call sites + 1 definition

8 existing (2 production, 6 test) + 1 new = **9**, which is what §4.3's own list
enumerates and what §8.4 says (*"Nine call sites"*). The header count "8" is stale.
The compiler catches a missed site, so this costs nothing but a reader's
confidence.

**M-3 — the template single-sig inventory names a keyless plate for its keys.
[JUDGEMENT]** `singleSigEngraveCards` hard-codes `summary: "wallet policy
descriptor"` (`gui/singlesig_engrave.go:41`), so a **template** engrave prints
`md1 descriptor: N plates (wallet policy descriptor)` over a plate that carries no
keys — and single-sig is the **first** path to put an inventory over a template set
(the build path skips the document entirely, `gui/multisig_build.go:464`). I agree
with §8.7 that it is not funds-losing, and the reasoning is stronger than §8.7
states: the concrete descriptor is printed **on the same page**, the mk1 carries
the key, and the inventory's own *"if any of them is missing, this backup is
incomplete"* keeps all three plates together. The summary is imprecise, not false.

**M-4 — the single-sig coverage gap closes at a step that, by the plan's own
design, cannot close it.  [MECHANICAL]** §4.7b: *"single-sig contributes zero sites
until `singleSigVerifyFlow` **gains a verdict** at build-order step 1, so this
covers multisig only."* But §4.7b-seam says the opposite three paragraphs later:
*"`singleSigVerifyFlow`, which is `void` today and **gains the parameter rather
than a return type**."* A sweep that finds `return verify*` sites will therefore
**never** see a single-sig site — not at step 1, not ever. So the path this cycle
is named after has no return-site coverage gate at all, while the multisig path
does. Minor rather than Important because what the gate would catch fails safe:
an unclassified exit sets no bit and lands in the zero cell. Measured, single-sig
has exactly **11** exits (`gui/singlesig_verify.go:68, 78, 90, 98, 112, 117, 125,
130, 138, 146, 148`), matching §4.8 step 1's count; the eleventh (`:148`) is the
success return and the only one that can over-claim.

**N-1 — §8.6 is stale on the leading parameter.** It enumerates what the error path
drops (*"no plate count, no seed statement … no passphrase statement"*) from the
era when only `extra` existed. §4.2 now also rides the **status** in, and it is
dropped there too. No over-claim results — the document does not render — so this
is bookkeeping.

---

## FILED — TRUE BUT OUT OF SCOPE

**FILE-1 — the verify-OK notice is mode-blind on the single-sig path.  [MECHANICAL]
— out of scope under §0.1.** `gui/singlesig_verify.go:148` is
`showNotice(ctx, th, "Verify OK", "The engraved bundle matches the seed.")` — two
strings, both pure ASCII, quoted from source. It is true in both modes (the
engraved bundle *is* mk1+md1, and it *does* match) and incomplete in watch-only,
where it does not say that no seed was engraved. This is §0 rule 2's *"every
screen"*, on a **screen**, at verify time — NG2's reader, not the document's. §8.8
already records it. **Correct, and not a G1 or G2 defect.** Filed, not folded.

**FILE-2 — `gui/multisig_verify.go:701` is classified adverse where a benign
reading is available.  [JUDGEMENT] — out of scope under §0.1.**
`extractReadbackMd1AndMk1s` failing means *"Read back one wallet-policy md1 AND the
operator key card(s)"*, which an operator can reach by scanning the wrong card
rather than by holding bad steel. Classifying it adverse gives `statusCheckDidNotPass`
(*"Do NOT rely on this backup"*) over what may be an operator slip. This is a
**cry-wolf** cost, and correcting it would be refining what the document reports
about the device's epistemic state — §0.1's guard, exactly. It over-warns rather
than over-claims, so G2 is untouched. Filed.

**FILE-3 — step 1 is a gate this plan has never run.  [JUDGEMENT] — out of scope
under §0.1.** §4.8 says *"Step 1 is a gate, not a task"*, and the eleven-row
single-sig table it produces does not exist. The project's own closure rule says a
plan may not close while one of its own gates has never been run. I record it
because it is true, and I do **not** rate it: the zero cell makes 10 of the 11
exits safe without the table, the 11th is named in M-4, and step 1 is explicitly
reviewed before step 2 begins. Whether that is enough is an operator call, not a
G1/G2 defect. Filed.

---

## WHAT I DID NOT DO

- Did not re-derive the four already-clean gates (15/15 return sites, 95/95
  citations, 56 glyph strings, superseded-term sweep).
- Did not re-litigate the four-state count, the goals, or the one-cycle scoping.
- Did not audit the codebase for pre-existing defects; every code reading above
  exists to test a claim the plan makes.
- Did not review prose, headings or markdown.
- Modified no file.

## ESCALATION SIGNAL

**Both blocking findings are MECHANICAL** — a struct field type against a doc
comment, and a count of tested flows against a count of flows. Neither needed
design judgement to find, and both are resolvable by reading the plan against
`grep`. The JUDGEMENT items (M-3, FILE-2, FILE-3) are all non-blocking. A fold plus
a **sonnet** claim-check over the seam — *does the record type carry every value
the pass line names, and does a test drive each of the three document flows* —
should be sufficient to close this; a further opus round is not indicated by what
this lens found.
