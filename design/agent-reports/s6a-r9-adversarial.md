# S6a R9 — independent adversarial review of `IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`

Artifact: `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md` @ `c0e615b`
Code: `/scratch/code/shibboleth/seedhammer` @ `b8a23bf` (branch `main`)
Question asked: **is this plan now safe to implement?**
Scope: §4.7a switch, §4.7c constants, §4.7d line table, §4.7e projection + its seven new
rows, P5 itself, and the single-sig half. No codebase audit; no prose review.

---

## VERDICT: RED — 5 Critical, 4 Important

---

### C-1 — `statusVerified` asserts a hand-typed ms1 check on every WATCH-ONLY run, where no ms1 exists  [JUDGEMENT]

**Where:** §4.7d line table, status 2 (`plan:984`); the seam at §4.7b (`plan:1137`);
T18 (`plan:1374`).

**The defect.** The success line is, verbatim:

> `Key and descriptor plates VERIFIED: read back and matched. The ms1 you typed matched this seed -- no ms1 plate was read.`

Both multisig paths and the single-sig path run the verify in **two modes**.
`full := modeSel == 0` (`gui/multisig.go:223`, `gui/multisig_build.go:378`), and
`full` is threaded into the verify (`gui/multisig.go:336`,
`gui/multisig_build.go:452`, `singleSigVerifyFlow(ctx, th, full, template)`).
In watch-only mode **no ms1 is typed and none is compared**:

- `gui/multisig_verify.go:876` — `ms1Readback := ""; if full { ... }`
- `gui/singlesig_verify.go:118` — same shape, `// Watch-only omits it.`
- `bundle/verify.go:75` — `if derived.MS1 == "" && readback.MS1 == "" { return nil }`
  — the ms1 leg is **skipped**.

The codebase already knows this and scopes for it: `multisigVerifyOKMessage(legs, full)`
(`gui/multisig_verify.go:1042`) branches on `full` in **all four** arms, and its doc
comment calls the watch-only case *"the commonest case this screen has"* and names the
exact bug of ignoring `full`: *"no ms1 created, none requested, none typed, none
compared … finished on 'Operator key and secret verified.' That is the doc comment's own
rule broken two lines below where it is stated."*

The plan's seam **cannot** express the distinction:

    func buildVerifyStatusLines(v verifyStatus) []string   // exactly one line   (plan:1137)

`verifyStatus` carries no mode. Arm 2 of the §4.7a switch fires on `res == verifyComplete`
irrespective of `full`. So a watch-only run that verifies cleanly prints a **positive
claim about a check that never ran**, on the durable restore document.

T18 does not catch it — it *describes* the line it tests as *"name[ing] the key and
descriptor plates only, and stat[ing] that no ms1 plate was read"*, which is not what
status 2 says. Its named mutation is *"restore 'each plate was read back and matched' —
R7's C-1, **false on every full run**"*. R7's fix moved the falsehood from full mode into
watch-only mode and the test row moved with it.

**The harm.** This is P5(a) violated on the one line an operator is most likely to
believe, in the commonest mode, on the artifact this entire cycle exists to make
truthful. It is F-198's shape — a document vouching for what nothing checked —
reintroduced by the sentence written to close it.

**Suggested remedy (UNVERIFIED):** `buildVerifyStatusLines` must take the mode (or the
status set must split the two pass statuses by mode), and the watch-only pass line must
make no ms1 claim at all. Mirror `multisigVerifyOKMessage`'s four-arm shape rather than
inventing a second scoping rule. **Not verified against the call graph** — the mode is
available at all three call sites (`full` is in scope at `gui/multisig.go:336`,
`gui/multisig_build.go:452`, and single-sig's caller), but the plan's build-order step 4
signature change would have to carry it and I have not traced §4.2's `restoreDocFlow`
callers for it.

---

### C-2 — P5(b) names a site where the distinguishing fact is NOT in scope; `bundle.Verify` is unowned by the plan  [JUDGEMENT]

**Where:** §4.7e C-3 paragraph (`plan:1114-1121`); T19 (`plan:1375`); §1.8 blast radius
(`plan:174-190`); §4.8 build order (`plan:1278-1288`); §5.1 (`plan:1450-1492`).

**The defect.** The plan says:

> `bundle.Verify` returns only **untyped errors** … **So the distinction is not
> reconstructed downstream — it is recorded at the comparator, where mk1-vs-ms1
> provenance is a value in scope.** That is P5(b).

Traced against the call graph, "the comparator" has two candidate sites and **neither has
the distinguishing fact**:

| site | has input provenance? | knows WHICH leg diverged? |
| --- | --- | --- |
| `verifyMultisigLegsPartial` (`gui/multisig_verify.go:386`) | no — it holds one opaque `error` | **no** |
| `verifyMultisig` (`gui/multisig_verify.go:184`) | yes — `ms1Readback` and `mk1, md1` are separate params | **no** — it is three lines: build a `Bundle`, call `bundle.Verify`, return its error |
| `bundle.Verify` (`bundle/verify.go:32`) | yes | **yes** — and only here |

`bundle.Verify` returns eleven distinct `errors.New` / `fmt.Errorf` values with **no
type, no sentinel, no wrapping discriminant** (stub binding ×2, mk1 decode ×2,
fingerprint, xpub, origin path, md1 string, ms1 presence, ms1 entropy, ms1 language).
The fact that separates *"a plate-derived comparison diverged"* (→ `statusDisagreed`,
|W|=1, the only status that may condemn) from *"a hand-typed ms1 diverged"*
(→ `statusUnaccounted`, |W|=2) is which of those eleven fired. **That is a value in scope
in exactly one place: inside `bundle.Verify`.**

The plan never changes package `bundle`. `bundle.Verify` appears in the plan **twice** —
in the paragraph above and in T19's mutation cell — and in neither is it an artifact the
plan owns. It is absent from §1.8's blast radius, from all nine build-order steps, from
§5.1's must-be-updated lists, and from §6's gate.

**The single-sig half has the identical hole, and the plan asserts the opposite.**
`plan:864`: *"**Single-sig is unaffected**: `singleSigVerifyFlow` has no comparison split
of this kind and no retry loop."* False for the part that matters: `verifySingleSig`
(`gui/singlesig_verify.go:48`) builds `bundle.Bundle{MS1: ms1Readback, MK1: mk1, MD1: md1}`
— the **same** hand-typed-vs-plate-derived mixture — and calls the **same**
`bundle.Verify`. Single-sig's exit 10 therefore has W = {the plates are wrong; the typed
ms1 has a typo}, |W| = 2, and needs the same provenance record. The shipped screen at
that exit already condemns steel on a keystroke: *"The read-back bundle does NOT match
the seed. **Check the engraved plates.**"*

**The harm.** T19's mutation is *"classify from `bundle.Verify`'s untyped error
downstream — **impossible by construction**, which is the point"*. The construction does
not exist, so the mutation is not impossible; it is the path of least resistance, and the
codebase already walks it — `multisigVerifyFailureText`'s doc comment
(`gui/multisig_verify.go:424`) reads *"`bundle.Verify`'s ms1 arms already say 'ms1'"*,
i.e. today's GUI reconstructs ms1-ness **from the error string**, which is precisely what
P5(b) forbids. An implementer at step 7, with no typed error to switch on, classifies
from the string or from the verdict; either way a one-character ms1 typo prints

> `WARNING: a read-back check DISAGREED with these plates. Do NOT rely on this backup: engrave a fresh set and check it before use.`

on a permanent document, over perfect steel. That is R5's I-1 regenerated, and §4.7d's
own warning (*"stamps an unclearable 'Do NOT rely on this backup' onto a document
describing PERFECTLY GOOD STEEL"*) landing at the document level where it is worst.

**Suggested remedy (UNVERIFIED):** the plan must own a change to `bundle/verify.go` —
either typed errors (`ErrMS1Diverged` / `ErrPublicLegDiverged` as `errors.Is` sentinels,
or an exported `LegKind` on a struct error), or a `verifyMultisig`/`verifySingleSig` that
compares the public legs and the ms1 leg in two separate calls so provenance is the call
site. It must appear in §1.8, in the build order **before** step 7, and in §5.1. **Not
verified**: I have not checked whether `bundle` has a Rust-primary counterpart under
the constellation rule, nor enumerated `bundle.Verify`'s other callers.

---

### C-3 — the two PASS arms classify from the VERDICT, and P5(c)'s monotonicity is FALSE for them  [JUDGEMENT]

**Where:** §4.7a switch (`plan:744-751`); P5(c) (`plan:898-901`); §4.7e row for `:987`
(`plan:1093`); §4.7a's own comment at `plan:735-736`.

**The defect.** The switch, verbatim:

    switch {
    case res == verifyComplete && sawDisagreement: status = statusVerifiedOnRetry
    case res == verifyComplete:                    status = statusVerified
    case sawDisagreement:                          status = statusDisagreed
    case sawUnaccounted:                           status = statusUnaccounted
    case obs == obsBenign:                         status = statusDidNotComplete
    default:                                       status = statusUnclassified  // P5(c)
    }

Arms 1 and 2 — the only two that emit a **positive** claim — **never read `obs` at all**.
They key on `res == verifyComplete`: a **verdict**. Four lines above, the plan's own
comment forbids exactly this:

> `// The observation is classified per RETURN PATH / ERROR / PROVENANCE (P4),`
> `// never per verdict and never per site`

and §4.7e rows the success return as an **observation** with |W| = 2:

> `| gui/multisig_verify.go:987 | verifyComplete | the success path — every leg matched a read-back plate; no ms1 plate was read | 2 | statusVerified, whose line is SCOPED |

So the plan has an observation for `:987`, gives it a world-set, and then builds the line
from the verdict instead. There is no `obsMatched`/`obsVerified` member in the enum the
switch uses (`obsDisagreed`, `obsUnaccounted`, `obsBenign` — the only three named), so
the success observation has nowhere to be recorded even if an implementer wanted to.

**P5(c) is false as a consequence.** P5(c) claims *"Incompleteness may only ever **weaken**
the printed line, never strengthen it."* Walk it: an implementer adds a return site and
forgets to classify it (P5(c)'s own named failure mode, and T17's named mutation).

- If that site returns anything **other than** `verifyComplete` → arms 3–6 apply, and the
  worst case is a weaker line. Monotone ✓.
- If that site returns `verifyComplete` → **arm 2 fires unconditionally** and mints
  `statusVerified`, the **strongest** line on the page, from an observation nobody
  classified. Monotone ✗.

The verdict-as-proxy failure the plan spent rounds 4 and 5 proving — `verifyFailed` has
five non-uniform sites, `verifyIncomplete` three — is reintroduced on the one verdict
nobody re-examined, which is R7's C-1 *by name* (*"the unexamined success row inherited
full-strength `VERIFIED`"*). Today `verifyComplete` has exactly one site, so the design is
accidentally faithful; it is not *structurally* faithful, and structure is what P5 claims
to buy over the demoted table.

**And therefore P4 is not a theorem of P5** (`plan:886-887`, question 5). P5(a)/(b)/(c)
constrain **provenance and derivability**; P4 constrains **truth across W**. A line can be
generated from a recorded observation, classified at its site, and non-default — and
still assert one member of a two-member W. The plan concedes this 25 lines later:
*"P5 splits P4's completeness into the part a tool enforces … and **the part a human still
judges (W per row)**"* (`plan:912-915`). A property whose core obligation is explicitly
left to human judgement is not entailed by the property that enforces the rest. C-1 above
is a live instance: P5(b) and P5(c) hold for status 2 and P4 fails on it.

**The harm.** The strongest line on the durable document is minted by a proxy the plan
itself proved unsound, and the default arm — P5(c)'s entire safety mechanism — is
**shadowed** for it. A future `verifyComplete` return site (a watch-only fast path, a
template short-circuit) silently vouches.

**Suggested remedy (UNVERIFIED):** give the observation enum a positive member and make
arms 1 and 2 require it (`case obs == obsMatched && sawDisagreement:` /
`case obs == obsMatched:`), so an unclassified observation on *any* verdict falls to arm 6.
Then delete the "P4 is a theorem of P5" claim or restate P4 as an independent obligation
discharged by the projection. **Not verified against the switch's other consumers.**

---

### C-4 — the observation type is never declared, and T17 cannot fail against its own named mutation  [MECHANICAL]

**Where:** §4.7a (`plan:738-743, 749`); §4.7c (`plan:1185-1205`); T17 (`plan:1373`);
§4.8 build order.

**The defect.** `obsDisagreed`, `obsUnaccounted` and `obsBenign` appear at
`plan:739`, `:741`, `:742` and `:749` — **used four times, declared zero times**. Grepped:

    grep -n 'obsBenign|obsDisagreed|obsUnaccounted|type observation' plan.md
    → 739, 741, 742, 749.  No declaration anywhere.

There is no type name, no constant block, no exhaustive member list, **no zero-value
ruling**, and no build-order step that produces one (step 2 produces `verifyStatus` +
`buildVerifyStatusLines`; step 7 is "wire the verify status into all three flows"). There
is likewise no site→observation mapping for the multisig half — §4.7e gives site→**status**,
which is the layer P5(b) says must not be the recording layer.

§4.7c spends **twelve lines and a capitalised heading** deciding this exact question for
`verifyStatus` (*"THE ZERO VALUE IS THE SAFE ONE, DELIBERATELY … Mirroring
`multisigVerifyResult`'s shape would have made `verifyComplete = iota = 0`, so the SAME
omission would print 'Plates VERIFIED'"*) and does not ask it about the variable the
classifier actually switches on.

**The harm — T17 is a false green.** T17 asserts *"No known return path reaches
`statusUnclassified`"*; its named mutation is *"add a return path and omit its
classification"*. Omitting the classification leaves `obs` at its **zero value**:

- zero = `obsBenign` (the natural ordering, and the last-named member is not usually
  first in a Go `iota` block only by accident) → arm 5 fires → `statusDidNotComplete`.
  T17 **passes**. The test does not fail against its own mutation, which §5's opening
  paragraph makes a blocking standard: *"Every test below must be shown to FAIL against
  ITS OWN MUTATION."* And `DID NOT COMPLETE` is a **stronger** line than the reserved
  one, so P5(c) is violated a second, independent way.
- zero = an undeclared `obsUnclassified` → the default arm becomes reachable and T17 is
  satisfiable — but nothing in the plan says so, and if no such member exists the default
  arm is unreachable *by construction* and T17 is a tautology that can never fail either.

Both readings are consistent with the plan as written. The plan cannot be implemented
without an implementer silently making a decision §4.7c already established is
funds-relevant.

**Suggested remedy (VERIFIED as consistent with §4.7c's own precedent, UNVERIFIED as
sufficient):** declare the type beside `verifyStatus` with the unclassified member as the
**zero value**, add the positive member C-3 needs, and add a build-order step producing
the 15-row site→observation mapping for multisig alongside step 1's single-sig mapping.

---

### C-5 — the ENUMERATED twelve-row table, which T13a/T13b are built from, was never updated for `sawUnaccounted` and contradicts §4.7e's routing  [MECHANICAL]

**Where:** §"ENUMERATED — every sequence, and what it prints" (`plan:953-979`); T13a
(`plan:1370`); T13b (`plan:1371`); T15 (`plan:1372`).

**The defect.** The table has three columns — `sequence | sawDisagreement | final res` —
and closes with:

> *"the switch depends only on `sawDisagreement` and the final `res`, so these twelve rows
> are the complete image of it — an honest statement of coverage."*

That sentence is **false against the switch printed 200 lines above it**, which reads
**four** variables: `res`, `sawDisagreement`, `sawUnaccounted`, `obs`. There is no
`sawUnaccounted` column and no row in which it is true. Concretely wrong rows:

| row (`plan`) | table says | §4.7e routing says |
| --- | --- | --- |
| `incomplete` then stop (`:962`) | `DID NOT COMPLETE` | `:738` plate count ≠ engraved count → `statusUnaccounted` |
| **`failed`** (foreign plates / undecodable / seed typo) then stop (`:970`) | **`DID NOT COMPLETE` — never condemns** | `:719` and `:724` → `statusUnaccounted`; only `:897` → `statusDidNotComplete` |
| `failed` → `complete` (`:971`) | `VERIFIED` — *"nothing was ever disagreed with"* | reaches arm 2 with `sawUnaccounted = true` (see I-1) |
| `mismatch` rows ×4 (`:965-969`) | keyed on a `res` value `mismatch` | the switch has no `verifyMismatch` arm (see I-2) |

§4.7e explicitly repaired the first of these — *"**'incomplete' is removed from §4.7d's
row-4 enumeration**, which had affirmatively swept the first two in — so they printed
`DID NOT COMPLETE` with no scope line, **violating P2**"* — and removed it from §4.7d's
row 4 only. It is still swept in here.

**The harm.** T13a is specified *"Table-driven over §4.7a's twelve rows"* and T13a/T13b
are the tests for P1 and P2. An implementer builds the fixture from this table and
asserts, byte-exact, that a `:719` foreign-or-garbled md1 prints `DID NOT COMPLETE` —
which is the **exact** defect §4.7e's C-2 fold just removed, and which T15b
(`plan:1377`) simultaneously forbids (*"the `:719` foreign-or-garbled md1 … yield[s]
`PLATES UNACCOUNTED FOR` — never `DISAGREED` and **never `DID NOT COMPLETE`**"*). Two
rows of the same test plan demand opposite outputs for the same observation. Whichever
the implementer builds, the suite goes green over a contradiction.

This is the failure mode the plan diagnosed on itself one section earlier and did not
sweep: *"the frame had been changed in prose only, and the artifacts a reader would build
from still described the old design"* (`plan:992-994`).

Two smaller stale pointers in the same rows: T13a cites *"the **six**-line table in
§4.7d"* (it has **seven** rows since the R8 fold) and *"§4.7a's twelve rows"* (§4.7a has
no table; the rows are in the unnumbered ENUMERATED section); T15 cites *"the §4.7d
observation table"* (it is §4.7e).

**Suggested remedy (UNVERIFIED):** regenerate the enumeration from the six-arm switch
with a `sawUnaccounted` column and an `obs` column, delete the false completeness
sentence, and state the row count the new space actually has. Do not hand-write it —
this table has been wrong in prose after three of the last four folds.

---

### I-1 — P2 is violated by the specified switch, and T13b as written is unsatisfiable  [MECHANICAL]

**Where:** P2 (`plan:879-883`); §4.7a arms 1–2; T13b (`plan:1371`); §4.7d
(`plan:815-819`).

**The defect.** P2 states:

> *Any sequence containing an adverse observation prints a line that carries it —
> `DISAGREED`, `PLATES UNACCOUNTED FOR`, or the repeat-check line — never bare `VERIFIED`
> and never `DID NOT COMPLETE`.*

Arm 1 reads `sawDisagreement` only. So the sequence **`unaccounted → complete`** — e.g.
`errVerifyLegHasNoPlate`, or a `:719` foreign-or-garbled md1, or a hand-typed ms1
divergence, followed by a clean retry (all four loop: `res != verifyIncomplete &&
res != verifyFailed` is false for `verifyFailed`) — falls to **arm 2** and prints **bare
`statusVerified`**. The adverse observation is dropped. P2 is false for the design.

§4.7d argues the retro-explanation for **one** of these classes only: *"an earlier
**pairing** failure is retro-explained as procedural"*. `sawUnaccounted` now carries six
observation classes, and the argument does not transfer to the **hand-typed ms1
divergence**: nothing ever reads the ms1 plate (the plan says so at `plan:1097`), so a
later clean pass — which compares a *retyped* ms1 — cannot retro-explain world (a) *"the
plate is wrong"*. The device saw an adverse signal about a plate it can never re-check
and says nothing.

T13b's assertion list is *"prints `DISAGREED` or the repeat-check line, never bare
`VERIFIED`, never `DID NOT COMPLETE`"* — it **omits** `PLATES UNACCOUNTED FOR`, which P2's
own sentence includes. Written literally, T13b fails on every unaccounted sequence; the
predictable repair is to narrow it to disagreement-only sequences, which is a false green
on precisely the status this cycle added.

**The harm.** Not a false claim (the printed line is scoped and, in full mode, true) —
but a **lost** one, on the property the plan asserts is directly assertable, with the test
that asserts it specified inconsistently with the property it names.

**Suggested remedy (UNVERIFIED):** decide explicitly, and in one place, whether a clean
pass retro-explains each of the six unaccounted classes. If not for the ms1 class, arm 1
must read `sawDisagreement || sawUnaccounted`, or a seventh status is needed. Then restate
P2 and T13b to match.

---

### I-2 — `verifyMismatch` is simultaneously superseded and load-bearing  [MECHANICAL]

**Where:** §4.7a (`plan:735`); §4.7d superseded subsection (`plan:842-857`); §5
(`plan:1409-1422`).

**The defect.** Three passages give incompatible instructions:

- `plan:735` — *"inside the existing offer loop, per attempt, **changing no control flow**"*
- `plan:842` — the subsection is headed **"SUPERSEDED AS A FIX, RETAINED AS A MEASUREMENT"**,
  and 10 lines later prescribes: *"both retry-loop conditions become
  `if res != verifyIncomplete && res != verifyFailed && res != verifyMismatch { break }`
  at `gui/multisig.go:337` and `gui/multisig_build.go:453`"*
- `plan:1409-1422` — *"THREE shipped tests pin the retry-loop condition… The
  `verifyMismatch` split changes that condition, so all three must be updated **in the
  same commit** or the atomic 5+6+7 landing is red"*, including a **source assertion**
  at `gui/multisig_verify_report_test.go:1093` that *"must be updated to the new condition
  verbatim, not loosened."*

Under the P5 switch **no arm reads any verdict but `verifyComplete`**. `verifyMismatch`
therefore has no consumer: adding it changes a normative 5-value verdict type, breaks a
shipped source assertion, and buys the status line nothing. Either it is dead and §5's
paragraph must go, or it is live and §4.7a's "changing no control flow" is false.

**The harm.** An implementer following §5 makes a normative verdict-type change the design
no longer needs, on the type whose doc comment the plan already lists as *"doubly wrong"*
(`plan:1242`), inside the atomic 5+6+7 commit.

**Suggested remedy (UNVERIFIED):** delete `verifyMismatch` from the plan entirely and
delete the three-test paragraph with it; the retry-loop condition is unchanged under P5.

---

### I-3 — `multisigVerifyFlow`'s return-arity change is unowned by §1.8, §4.8 and §5.1  [MECHANICAL]

**Where:** §4.7a `res, obs := <verify>` (`plan:737`); §1.8 (`plan:174-190`); §4.8;
§5.1 (`plan:1450-1492`).

**The defect.** `res, obs := <verify>` requires
`multisigVerifyFlow(...) multisigVerifyResult` → `(multisigVerifyResult, observation)`.
That changes the in-file test seam `var multisigVerifyFn = multisigVerifyFlow`
(`gui/multisig_verify.go:660`) and its one stub
(`gui/multisig_engrave_tail_walk_test.go:105`, restored at `:114`), plus both production
call sites.

§1.8 is titled **"Blast radius of the signature change"** and covers only `restoreDocFlow`
and `multisigRestoreDocFlow`. §4.8's step 4 is *"`restoreDocFlow` and
`multisigRestoreDocFlow` gain `status` + `extra`"* — the verify flow's arity is in no
step. §5.1 exists to be the complete list of tests that must be updated, has already been
found incomplete twice, and does not list the stub.

**The harm.** Compile-announced, so it cannot ship silently — but §5.1's stated purpose is
that the implementer not discover this list by breakage, and the atomic 5+6+7 landing is
where it bites.

**Suggested remedy (VERIFIED as the complete set of affected sites):** add
`gui/multisig_verify.go:660` + `gui/multisig_engrave_tail_walk_test.go:105` to §1.8 and
§5.1, and give the arity change its own build-order step before 7.

---

### I-4 — two of the seven new §4.7e rows describe the code wrongly; one is for a dead branch  [MECHANICAL]

**Where:** §4.7e (`plan:1065-1071`, `plan:1090`); `scripts/verify-returnsite-sweep.sh`
header.

**(a) `:701` is `verifyRefused`, not `verifyIncomplete`.** The plan states
*"`verifyIncomplete` has **three non-uniform return sites**, unexamined:"* and tables
`:701`, `:738`, `:979`. Measured:

    grep -n 'return verifyIncomplete' gui/multisig_verify.go   → 738, 938, 979
    grep -n 'return verifyRefused'    gui/multisig_verify.go   → 670, 680, 701, 794

`gui/multisig_verify.go:698-702` is the `extractReadbackMd1AndMk1s` failure and returns
`verifyRefused`. The real third `verifyIncomplete` site is `:938`, which the plan files in
the *other* table. The same false claim is baked into the committed script's header
(*"`verifyIncomplete` turned out to have three non-uniform sites"*), where it will outlive
the plan. Consequence beyond the miscount: `verifyRefused` **breaks** the caller's retry
loop (`res != verifyIncomplete && res != verifyFailed` → true), so the operator gets one
shot and no re-offer — the row's implicit "try again" framing does not hold.

**(b) `:794`'s W is false, and the branch is unreachable.** The row reads
*"`verifyFreshSlots` failed — a structural refusal **before any readback**"*. `:794` sits
**inside** the per-seed loop, after `bundleGatherFlow`, after the `slices.Equal(readbackMd1,
engravedMd1)` identity check and after the mk1 count precheck — the readback has
happened. And `verifyFreshSlots` (`gui/multisig_verify.go:318-329`) returns an error on
**one** condition, `len(expected) == 0`, which `gui/multisig_verify.go:667-670` already
returned `verifyRefused` on; `expectedSlots` is never reassigned. So `:794` is **dead
code**, and its row — a T15 fixture row — is vacuous.

**The harm.** No wrong output: the routing (`statusDidNotComplete`, |W| = 1) is correct in
both cases for the reason the rows give wrongly. But T15 is specified with the projection
**as its test fixture**, so a vacuous row is a vacuous assertion, and the script's header
now carries a false fact about the code into every future run.

**Suggested remedy (VERIFIED for (a) — the grep output is the correction; UNVERIFIED for
(b)):** correct the verdict attribution in the plan **and** in the script header; either
mark `:794` unreachable-today with its guard cited, or drop the row and let the sweep
record it as deliberately unrowed.

---

## MINOR (recorded, does not gate)

- **M-1.** T9 asserts *"each of the **six** §4.7c statuses renders its own line"*; §4.7c
  declares **seven** constants and §4.7d's line table has **seven** rows.
  `statusUnclassified`'s line is left with no renderability or uniqueness assertion, while
  §4.7 requires *exactly one* line on every rendered document.
- **M-2.** `:938`'s W reads *"the seed filled no slot this run engraved"*. `correctable`
  is also set by an **ms1 entry rejection** (`gui/multisig_verify.go:889`,
  `correctable = correctable || rejected`), so the row under-enumerates its own route.
  Routing is unaffected — both routes are plate-neutral, |W| = 1 stands (see below).
- **M-3.** `:897`'s W reads *"the operator mistyped the seed"*. By `:897` the seed has
  already matched a slot through `allUserSlots`, so a typo is not the live cause;
  `deriveMultisigLeg` can also fail in `md.FormAwareStubChunks(readbackMd1)`. Plate-neutral
  either way, because `readbackMd1` is byte-equal to `engravedMd1` by then (`:719`), so
  |W| = 1 and `statusDidNotComplete` stand.

---

## P5 CLAUSE-BY-CLAUSE

**P5(a) — positive claims are GENERATED from a recorded observation with provenance.**
**FAILS.** Two independent ways. (i) `statusVerified`'s ms1 sentence has no generating
observation at all in watch-only mode — nothing is typed, nothing is compared, and
`bundle.Verify` skips the leg (**C-1**). (ii) Both pass lines are generated from
`res == verifyComplete`, a verdict, not from the `:987` observation the projection rows
with |W| = 2 — and the observation enum has no member to record it with (**C-3**).
The scoping clause *"no ms1 plate was read"* is sound and does hold; the failure is the
**other** half of the same sentence.

**P5(b) — classification at the point of knowledge.** **FAILS AS SPECIFIED.** The
distinguishing fact for the single most consequential distinction the status map draws
(plate-derived divergence → the condemning line, vs hand-typed ms1 divergence → the
ambiguous line) is a value in scope **only inside `bundle.Verify`**, which the plan does
not change, does not list, and does not schedule (**C-2**). The named site,
"the comparator", holds an opaque `error`. The observation therefore **cannot travel** to
the status map without being reconstructed downstream — the thing P5(b) exists to forbid,
and the thing `multisigVerifyFailureText` already does today from the error string.
Single-sig has the identical hole and the plan states the opposite.

**P5(c) — monotone under omission; default arm unreachable.** **FAILS.** The default arm
is unreachable, but for the wrong reason and with the wrong consequence. Arms 1 and 2
never read `obs`, so an unclassified observation arriving on a `verifyComplete` return
produces the **strongest** line, not the fewest-claims line — monotonicity inverted
(**C-3**). And because the observation type is undeclared, a forgotten classification most
likely lands on `obsBenign` → `statusDidNotComplete`, still stronger than the reserved
line, with **T17 passing against its own named mutation** (**C-4**).

**"P4 is a theorem of P5".** **FALSE**, and the plan concedes it 25 lines later by leaving
"W per row" to human judgement. C-1 is the witness: P5(b) and P5(c) hold for status 2 and
P4 fails on it.

**"Incompleteness may only weaken a claim".** **FALSE** — see P5(c) above. It holds for
four of the six arms and fails for the two that matter most.

---

## THE SWITCH — ARM ORDER WALK

Constraints from §4.7a: `obs == obsDisagreed ⟹ sawDisagreement`; `obs == obsUnaccounted ⟹
sawUnaccounted`; both sticky; `status` assigned only inside the loop body. `res` and `obs`
are the **last attempt's**; each attempt exits at exactly one return site, so exactly one
`obs` per attempt (verified against `multisigVerifyFlow` — every path terminates at one
of the 15 `return verify*` sites).

| # | `res` | `sawDis` | `sawUnacc` | `obs` | arm | prints | true? |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | *(no attempt)* | F | F | — | none | `NOT VERIFIED` | ✓ (zero value) |
| 2 | complete | F | F | benign | 2 | `VERIFIED` | **✗ in watch-only (C-1)** |
| 3 | complete | T | F | benign | 1 | `VERIFIED on a repeat check` | ✓ |
| 4 | complete | F | **T** | benign | **2** | `VERIFIED` | scoped-true in full mode, **P2-violating (I-1)**; ✗ in watch-only |
| 5 | complete | T | T | benign | 1 | repeat-check | ✓ |
| 6 | failed/incomplete/abandoned | T | any | any | 3 | `DISAGREED` | ✓ |
| 7 | failed/incomplete/refused/abandoned | F | T | any | 4 | `PLATES UNACCOUNTED FOR` | ✓ |
| 8 | failed/incomplete/refused/abandoned | F | F | benign | 5 | `DID NOT COMPLETE` | ✓ |
| 9 | any ≠ complete | F | F | *unclassified* | 6 | `statusUnclassified` | ✓ but see C-4 |
| 10 | **complete** | F | F | *unclassified* | **2** | **`VERIFIED`** | **✗ — the monotonicity hole (C-3)** |

**Shadowing:** arm 1 correctly shadows arm 3 (P1 — a clean pass must not print
`DISAGREED`). Arm 3 correctly shadows arm 4 (`DISAGREED` outranks `UNACCOUNTED`).
Arm 2 **incorrectly** shadows arm 4 for row 4 (I-1) and shadows arm 6 for row 10 (C-3).

**The question asked — an unaccounted observation then `verifyComplete`:** row 4. It
prints **bare `statusVerified`**. In **full** mode the line is *literally* true — the
scoping clause carries it, so P4 survives — but the adverse observation is silently
dropped, breaking P2 as stated (I-1). In **watch-only** mode the same line is **false**,
because it claims a hand-typed ms1 check that never happened (C-1).

**Is the default arm genuinely unreachable?** With the three named `obs` members, yes —
trivially, since `obsDisagreed`/`obsUnaccounted` set their sticky flags and `obsBenign`
is arm 5. That makes T17 a tautology it cannot fail. With an undeclared zero value it is
reachable or not by accident (C-4). Either way T17 does **not** fail against its own
mutation, so it is a false green today.

---

## WHAT I CHECKED AND FOUND SOUND

- **`statusUnaccounted`'s line and its six-way routing.** All four `|W| = 2` rows in the
  main projection (`errVerifyLegHasNoPlate` `:394`, `:719`, `:724`, hand-typed ms1) plus
  the two added incomplete rows (`:701`, `:738`) land on a line that states **both**
  readings and **both** actions. The `|W| > 1` rule is genuinely satisfied for them.
- **The `:987` row's `|W| = 2`, and the scope-vs-ambiguity distinction.** Verified against
  the code: `multisigVerifyFlow` cuts an ms1 plate in full mode and never reads one back
  (`ms1Readback` is hand-typed at `:876-891`; `bundleGatherFlow` yields only `cardMK1` /
  `cardMD1`). The device genuinely knows *which* thing it did not look at, so scope is the
  right instrument. The row's |W| is right; the **line built on it** is not (C-1).
- **`:938` (`verifyIncomplete`, zero legs, correctable) observes nothing adverse about the
  plates.** Traced: reaching `:938` requires `:719` (md1 identity) and `:738` (mk1 count)
  to have **passed**, and `verifyMultisigLegs`/`…Partial` — the only functions that touch
  a read-back plate — are unreachable with `len(legs) == 0`. Both routes to `correctable`
  (a no-fresh-slot seed, and an ms1 entry rejection) are about hand-typed objects.
  |W| = 1 and `statusDidNotComplete` are **correct**, notwithstanding M-2.
- **`:794` observes nothing adverse about the plates.** Correct for the same reason — but
  the row's stated reason is false and the branch is dead (I-4b).
- **`statusDisagreed`'s `|W| = 1`.** I tried to break it with an NFC-read-error world, by
  analogy to `:724`'s admitted *"(b) an NFC read error"*. It holds: `verifyClaimPlate`
  (`gui/multisig_verify.go:532-549`) `mk.Decode`s every candidate and **skips** the
  undecodable, so a garbled mk1 surfaces as `errVerifyLegHasNoPlate` (|W| = 2,
  `statusUnaccounted`) and never as a comparison mismatch. A plate that pairs has a
  matching xpub. The condemning line's row is sound.
- **`statusNotVerified` as the zero value, and the type-vs-document argument** (§4.7c).
  The reasoning is right and the distinction between a forgotten *assignment* and a
  forgotten *call* is correctly drawn, with the right remedy (a rendered-document
  assertion, not a type argument).
- **§4.7f's scope line is P4-clean**, as claimed: it asserts only that the text below
  states intent, and prescribes no action.
- **§4.7b's index-0 argument.** `gui/multisig_restore.go:106` appends `extra` after
  `lines`, so a leading `status` parameter is genuinely required. Sound.
- **The sweep script's declared blind spots are honest and accurate** — in particular
  *"IT COVERS MULTISIG ONLY, TODAY … A clean 15/0 therefore covers HALF the surface this
  plan changes."* The gate does not hide its own hole.
- **`singleSigVerifyFlow` has eleven exits** (counted: seed entry, wallet-type pick,
  re-derive, templateize, gather, readback cards, ms1 entry, non-string object, ms1
  decode, comparator, success fallthrough). The count in the plan is right.

**On question 4 — is deferring single-sig's classification to build-order step 1 safe
under P5(b)?** **No, as the step is currently scoped.** Two reasons, in order of weight.
(i) Step 1's deliverable is *"the single-sig exit → **`verifyStatus`** mapping"* — a
site→status table. P5(b) requires site→**observation**, with provenance, and §4.7d/§4.7e
demoted per-site status assignment twice for exactly this reason. On the single-sig half
there is no retry loop, so per-exit status assignment is mechanically sound — but it
records no observation, which means P5(a)'s "generated from a recorded observation" has no
structure behind it on that half, only a promise. (ii) More seriously, `plan:864` tells the
step-1 author that *"single-sig … has no comparison split of this kind"*, which is false:
`verifySingleSig` mixes a hand-typed ms1 with plate-derived mk1/md1 and calls the same
untyped `bundle.Verify`. Exit 10 therefore has |W| = 2 in full mode and **must** route to
`statusUnaccounted`, and no reader of step 1's brief has been told so. The design decision
that must be made **now** is not the eleven-row table — it is `bundle.Verify`'s provenance
record (C-2), which both halves need and neither owns.

---

## HOW I WOULD SPEND THE NEXT ROUND

C-2 and C-3 are the two that cannot be folded as wording: one adds an owned change to a
package outside the plan's current surface, the other changes the switch and retires the
"P4 is a theorem of P5" claim. C-1 is a signature change plus two strings. C-4 and C-5 are
mechanical and should be **generated**, not written — the enumeration from the switch, the
verdict attributions from `grep -n 'return verify' `, both pasted. Every one of I-1..I-4
is a `grep` or a call-graph trace away, and none of them needed a review round to find;
the cheap-verification tier would have caught I-2, I-3 and I-4 before this report cost
anything.

*Reviewer note: the plan is close. Four of the five Criticals are the same root seen from
different sides — **the pass path was never subjected to the discipline the failure paths
were.** Rounds 4 through 8 audited `verifyFailed` five ways, `verifyIncomplete` three
ways, and rowed all fifteen return sites; the success return got one row, added last, and
the line built on it is the only line in the table with an unscoped positive claim in it.
The lens that has never been run on this artifact is "walk the HAPPY path".*
