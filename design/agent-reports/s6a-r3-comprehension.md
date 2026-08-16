# S6a R3 — COMPREHENSION lens

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Code:** `/scratch/code/shibboleth/seedhammer` @ `b8a23bf` (clean tree)
**Question asked, and the only one answered:** could a competent Go engineer who
has never seen this project execute this plan end to end, from the plan alone,
and produce the intended result?

**Answer: no.** Not because the design is wrong, but because the plan's own
authoritative statement of the C-1 algorithm contradicts the C-1 table six lines
below it; because the piece of new logic that algorithm describes has no named
home, no signature and no unit under test; because the single-sig half of the
Critical is delegated to "a table the implementer produces" with no shape, no
approver and no gate; and because two in-scope deliverables (the supply-path
document correction, the T6a spec update) have no test and no design section
respectively.

## VERDICT: RED — 0 Critical, 8 Important

Nothing here is funds-bearing *in the vouching direction* under a faithful
reading, which is why the count is 0 Critical. I-2 gets within one plausible
misreading of it, and I say so under that finding.

---

## THE SCOPE MATRIX

Built from §3, §3.1, §3.2 and §4/§5 directly, not from the plan's own claims.

| scope item | design section? | test? | gaps |
| --- | --- | --- | --- |
| **F-198a** — mode label | §4.1 | T1 (positive), T3 (non-vacuity) | none |
| **F-198b** — restore doc gains an inventory | §4.2 | T2 | call site references `worstStatus`, defined nowhere (I-4) |
| **C-1** — status line, all three paths | §4.7, 4.7a, 4.7b, 4.7c | T9–T14 | algorithm contradicts its own table (I-1); severity order undefined and inverted vs. the type (I-2); ranking logic unnamed, no unit under test (I-3); single-sig mapping delegated (I-4); third call site unlisted (I-5) |
| **F-197** — abort gate | §4.5 | T5 | none |
| **F-197** — false `bundle_flow.go:535` comment | §4.5 | T8 (+ §5.2's positive half) | none |
| **F-195** — seed statement | §4.4 | T4 (unit), T2 (doc seam) | declared in §5.2; acceptable |
| **F-202** — pre-engrave census | §4.6 | T6 (+ §5.1b walk repairs) | none |
| **§1.3 landmine** — `seedCapacity` ruling | §4.3 | T7 (arms only) | **the wiring of 2 of 3 paths is tested by nothing** (I-7) |
| **§3.1.1** — supply-path document is *corrected* (shipped, S5-reviewed text changes) | §4.3 | **none** | I-7 |
| **§3.1.7** — `SPEC_seedhammer_T6a_singlesig_flagship.md` is updated this cycle | **none** | **none** | I-8 |

Every filed item in §3's table has a design section and a test. The gaps are all
in the two things §3's table does *not* list: the capacity refactor's wiring and
the spec update.

---

### I-1 — §4.7a's algorithm prints the retry warning on a clean first-try verify, contradicting §4.7a's own table

**Where:** §4.7a (the algorithm block and the severity line) vs. §4.7a's
enumerated table row 2, vs. §4.7c's constant block.

**The defect.** The algorithm is stated as, verbatim:

    worst := statusNotVerified          // the zero value; see §4.7c
    on each verify attempt:
        worst = max(worst, severity(verdict))
    if the FINAL attempt was a clean pass:
        status = verifiedFirstTry   if worst is no worse than complete
               = verifiedOnRetry    otherwise

and the severity order immediately below is *"disagreed > did-not-complete >
not-verified > verified"*. So `not-verified` is **more severe than** `verified`.

Trace the commonest success, a single `verifyComplete`:

- `worst` starts at `not-verified`.
- one attempt, verdict `complete`, severity `verified`.
  `max(not-verified, verified)` = **not-verified** — the seed wins, because the
  seed is not the minimum of the stated order.
- final attempt was a clean pass. Is `worst` (not-verified) "no worse than
  complete" (verified)? **No.** → `verifiedOnRetry`.

The device prints `Plates VERIFIED on a repeat check, after an earlier read-back
DISAGREED. Confirm they restore before relying on this backup.` on a durable
document, on a run where nothing ever disagreed.

§4.7a's own table, six lines further down, says row 2 is `complete` → `VERIFIED`.
The two statements are incompatible, and the plan explicitly nominates the
algorithm as the authority: *"Stated as an algorithm rather than as prose,
because the R1 fold reasoned about this in prose and got it wrong twice."*

**The root of it is a genuine collision between two sections, not a typo.** §4.7c
justifies `statusNotVerified` as the zero value so that *a forgotten assignment*
is safe. §4.7a needs the accumulator seeded at the severity **minimum**, which by
its own stated order is `verified`. One variable cannot be both, and no section
notices.

**What they would plausibly do instead:** implement it exactly as written. No row
of §5 catches this — T9 tests the renderer, T10 tests `failed → abandoned`, T12
tests `incomplete → complete`, T13 asserts only that a pass never prints worse
than `NOT VERIFIED` (and `verifiedOnRetry` is a pass line, so T13 is satisfied),
T14 tests the zero value. **Nothing asserts that a plain `complete` prints the
first-try line.** I checked all fourteen rows.

Not Critical because the error is in the over-warning direction. It is still a
false sentence on the artifact read years later, which is the defect class this
cycle exists to kill.

---

### I-2 — `severity()` is undefined, and §4.7c's constant order is the inverse of it above `statusDisagreed`

**Where:** §4.7a (algorithm + severity line) vs. §4.7c (the `iota` block).

**The defect.** §4.7a calls `severity(verdict)` and `max(...)`. Neither is
defined anywhere in the plan, and the type the values live in is declared in
§4.7c in an order that is **not** the severity order:

| §4.7c constant | value | §4.7a severity rank (0 = least severe) |
| --- | --- | --- |
| `statusNotVerified` | 0 | 1 |
| `statusDidNotComplete` | 1 | 2 |
| `statusDisagreed` | 2 | 3 |
| `statusVerifiedOnRetry` | 3 | (pass) |
| `statusVerified` | 4 | **0** |

§4.7c's doc comment justifies the ordering solely on the zero value and never
warns that the numeric order must not be used as a rank. Given an `iota` type and
an instruction to "keep the worst" with `max`, the idiomatic Go an implementer
writes is `if v > worst { worst = v }`.

**What they would plausibly do instead.** With that one line: `failed → complete`
accumulates `max(0, 2, 4) = 4 = statusVerified`, the final attempt is a clean
pass, `worst` *is* "complete", so the document prints `Plates VERIFIED: each
plate was read back and matched.` over a set whose read-back **disagreed**. That
is C-1 reintroduced, in the funds-bearing direction, by following an
under-specified sentence.

It is Important rather than Critical for two reasons, both of which I verified
rather than assumed: the severity order *is* stated in words immediately below
the algorithm, so `>` on the raw constants is a misreading rather than
faithfulness; and T12 (`incomplete → complete` must not print bare `VERIFIED`)
does fail against that implementation — **provided T12 has something to test,
which is I-3.**

The plan owes a named comparator with a written body, not a `severity()` the
reader must reconstruct from a prose ordering that disagrees with the type.

---

### I-3 — §4.7b claims "ONE SEAM" but specifies only the renderer; the ranking has no name, no signature and no unit under test

**Where:** §4.7b ("ONE SEAM, AND IT IS THE ONE THAT ALREADY EXISTS") vs. §4.7a
(the accumulator) vs. §5's header and T10/T12/T13.

**The defect.** §4.7 introduces **two** pieces of new logic:

1. `buildVerifyStatusLines(v verifyStatus) []string` — named, signature given.
2. the worst-seen accumulator of §4.7a — **not named anywhere in the plan.**

§4.7b's heading asserts there is one seam. There are two, and it is the unnamed
one that carries the Critical's correctness.

Three concrete consequences an implementer cannot resolve from the plan:

- **T10, T12 and T13 have no unit under test.** They are all assertions about the
  ranking, and the only exported-ish surface the plan names takes a
  *already-decided* `verifyStatus`.
- **The sequences those tests need are unreachable on the single-sig path.**
  Measured: `gui/singlesig.go:130-133` is a one-shot `if sel == 0 { ... }`, with
  no retry loop. `failed → abandoned` (T10) and `incomplete → complete` (T12)
  exist only on `gui/multisig.go:330-342` and `gui/multisig_build.go:445-458`.
  §5's header says *"New file: `gui/singlesig_truth_test.go`"* and names no other.
- **If the accumulator is inlined into all three flows** — the shape the plan's
  own §4.7 text suggests, since it describes the logic per-path — then a T10/T12
  driven through one flow proves nothing about the other two. That is verbatim
  the defect §5 says R2 I-3 closed: *"tests that would all still pass if the
  status line were wired into two of the three document flows and forgotten on
  the third."*

**What they would plausibly do instead:** put T10/T12/T13 in
`gui/singlesig_truth_test.go` against a helper they invent, satisfying the letter
of §5 while the two multisig flows carry a hand-copied ranking neither test
touches.

---

### I-4 — the single-sig half of the Critical is a table the implementer produces, with no shape, no approver, no location and no gate; and `worstStatus` is defined nowhere

**Where:** §4.7c ("Single-sig's mapping must be written, not left to the
implementer"), §4.2's call-site snippet, §4.7b's last line.

**The defect.** Four separate holes, all on the *more travelled* path:

1. **`singleSigVerifyFlow`'s new signature is never stated.** §4.7c says it "has
   eleven exit points and today returns nothing" (I confirmed eleven:
   `gui/singlesig_verify.go` returns at `:69, :78, :90, :98, :112, :117, :125,
   :130, :138, :146`, plus fall-off after `:148`). It never writes
   `func singleSigVerifyFlow(ctx *Context, th *Colors, full, template bool) verifyStatus`.
   Every other signature in §4 is written out; this one, the Critical's own, is not.
2. **`worstStatus` appears exactly once in the whole plan** — inside §4.2's call
   site, `buildVerifyStatusLines(worstStatus)` — and is never declared, typed, or
   cross-referenced. §4.7a calls the same thing `worst`. §4.2 does not point
   forward to §4.7 for it.
3. **The mapping table has no specification.** Not its columns, not where it is
   written (the plan? a report in `design/agent-reports/`? a comment in the
   source?), not who reviews it, not what happens if the review disagrees, and
   §6's validation gate does not mention it. §4.7c says only *"The implementer
   produces the mapping as the **first** step of §4.7, and it is reviewed before
   the rest of the section is built."* The plan's own status line says no code
   may be written before it closes GREEN; this introduces a second, undefined
   review gate *inside* implementation.
4. **§4.7b says "Single-sig gets its own status type (§4.7c)"** while §4.7c
   defines a single `verifyStatus` shared by all three paths. Read alone, that
   sentence licenses a second `singleSigVerifyStatus` type — after which T9's
   "each of the five §4.7c statuses" and T14's zero-value assertion cover only
   half the tree.

**What they would plausibly do instead:** write the mapping as an inline comment,
have nobody review it, and map every early `return` (seed-entry cancel, wallet
pick cancel, gather cancel, ms1-input cancel) to `statusNotVerified` on the
reasoning that the operator "did not really verify" — producing `Plates NOT
VERIFIED` where §4.7a's table says `refused / abandoned` → `DID NOT COMPLETE`.
Both are defensible; the plan picks neither, and it is the one screen that tells
a stranger in five years whether the steel was ever checked.

---

### I-5 — `multisigRestoreDocFlow` has three call sites, not two, and the third renders a real document that would carry no status line

**Where:** §3.2 ("Both multisig call sites change"), §4.7b ("at both call
sites"), §5.1's two lists, §4.7 ("A rendered document with no status line is a
defect, not a default"), T9.

**The defect.** Measured:

    grep -rn "multisigRestoreDocFlow(" --include="*.go" gui/
    gui/multisig.go:361
    gui/multisig_build.go:478
    gui/multisig_nested_name_test.go:230      <-- unlisted
    gui/multisig_restore.go:100 (definition)

`gui/multisig_nested_name_test.go:230` is
`multisigRestoreDocFlow(ctx, &descriptorTheme, tpl, keys, nil)` inside
`TestRestoreDocNestedNameIsActuallyDrawn`, which **rasterises the document and
asserts an ink floor**. It is in neither §5.1(a) (the six
`buildPlateInventoryLines` sites) nor §5.1(b) (the three single-sig walks), and
§1.8 measured the blast radius of `restoreDocFlow` but never of
`multisigRestoreDocFlow`.

**What they would plausibly do instead:** hit the compile error, pass
`nil, nil`, and move on. The tree then contains an executing test that renders a
restore document with **no status line at all** — which §4.7 declares "a defect,
not a default" and which makes T9's *"every rendered document carries exactly
one"* false in-tree. There is also a second-order effect: the test's
`buildWalkRasterFloor` ink assertion is calibrated against the current line set,
and prepending a status line changes what page 1 draws.

The plan must either list this site with the status it should pass, or state
that a `nil` status in a test is permitted and reconcile that with T9.

---

### I-6 — §5's evidence rule is unsatisfiable for most of §5, and is contradicted outright by T3

**Where:** §5 opening paragraph vs. the T-row mutation column vs. §5.2's T3 note.

**The defect.** §5 opens: *"**Every test below must be shown to FAIL against the
unfixed tree.**"* Against `b8a23bf`:

- **T3 must PASS.** It is the non-vacuity arm: *a bare run's label does NOT
  contain `NOT passphrase`*. That is true today (`gui/singlesig.go:80` is the
  bare literal). §5.2 then instructs T3's document half to assert *"directly on
  `buildPassphraseInventoryLines`"* — which also passes today. A test the plan
  requires to fail, and which the plan elsewhere requires to pass.
- **At least eight rows cannot fail, because they cannot compile.** T2, T4, T7,
  T9, T10, T11, T12, T13 and T14 name `seedCapacityOne`/`Many`,
  `buildSeedInventoryLines`, `verifyStatus`, `buildVerifyStatusLines`, or the new
  two-parameter `restoreDocFlow` — none of which exist in the unfixed tree. The
  package does not build, so *every* test in `gui/` "fails", including tests that
  have nothing to do with this cycle.

The mutation column is clearly the operative protocol (implement → mutate →
observe). The opening sentence is a different, older protocol left standing.

**What they would plausibly do instead:** report the package-level compile
failure as the required FAIL evidence for eight rows. That satisfies §5's
sentence and proves nothing — and it is precisely the false-evidence shape §5's
next sentence forbids (*"proves the mutated line RAN, not merely that the edit
landed"*). Only T1, T5, T6 and T8 can genuinely be shown red-before-green.

---

### I-7 — the capacity **wiring** on two of three paths is tested by nothing, and §8.4 claims coverage that does not exist

**Where:** §4.3 (call-site list), §3.1.1 (the supply-path correction), §5's T7,
§8.4.

**The defect.** §4.3 routes three production call sites: `multisig_build.go:479`
→ `seedCapacityMany`, `multisig.go:362` → `seedCapacityOne`,
`singlesig.go:136` → `seedCapacityOne`. §4.3 then names the guard for exactly one
of them (`TestSeedResidencyRulingDescribesTheMultiSeedReality`, which asserts
`"Every seed"` and therefore pins the **build** path).

Measured, no other test in the tree asserts the seed-handling ruling:

    grep -rn "can hold several\|Every seed\|seed you entered" --include="*.go" gui/
    # gui/multisig_build_census.go:86-87 (the string)
    # gui/multisig_build_prose_test.go:375,377,382 (the ONE unit assertion)

T7 asserts the two arms of the *function*. It never asserts that
`gui/multisig.go:362` passes `seedCapacityOne` or that `gui/singlesig.go:136`
does. §8.4 states *"the supply path and single-sig are covered only by T7 and by
review"* — that overstates it: T7 covers neither, so the coverage is **review
only**.

This matters more than a generic coverage gap because §3.1.1 declares the supply
path's document *changes*: shipped, S5-reviewed prose on a funds-bearing artifact
moves from "this build can hold several" to "this build holds exactly one". A
deliberate change to a durable document, with nothing that fails if it silently
does not happen — or happens on the wrong path.

**What they would plausibly do instead:** wire `gui/multisig.go:362` to
`seedCapacityMany` "to keep the multisig documents byte-identical" (a reading
§3.1's assumption 8 explicitly warns is wrong but which the compiler and the
suite both accept), and ship the supply path still saying a false sentence — the
§1.3 landmine, on the path §1.3 did not check.

---

### I-8 — §3.1.7's spec update is an in-cycle deliverable with no design section, no content and no test

**Where:** §3.1 assumption 7; absent from §4, §5, §6 and §7.

**The defect.** §3.1.7 rules: *"So the spec is updated in this cycle, in its own
commit, separate from the plan and from the code."* The spec is real and the
citation is accurate —
`design/SPEC_seedhammer_T6a_singlesig_flagship.md:36` describes the restore doc
exhaustively as four fields.

Nothing else in the plan mentions it. §4 has no section for it. §5 has no row.
§6's validation gate does not check it. §7 ("What is NOT in this plan") does not
exclude it. So it is in scope by exactly one sentence, and that sentence does not
say what the updated text should assert, whether the spec's exhaustiveness clause
becomes non-exhaustive or is re-enumerated, or whether the updated spec re-enters
R0.

**What they would plausibly do instead:** finish the Go work, run §6's four
commands green, and ship — leaving the spec describing a four-field document that
now has three more kinds of line on it. The plan names that exact outcome as
*"the trap this project has been caught by before"* and then does not schedule
the escape.

---

## BUILD ORDER

**The plan gives one ordering sentence in total**, and it is wrong: §4.7c says
the single-sig mapping table is *"the **first** step of §4.7"*, but the mapping's
target type (`verifyStatus`) is itself defined in §4.7c and the ranking it feeds
is §4.7a — so the table cannot be written, let alone reviewed, before the
sections that define what it maps onto exist.

More consequentially, **§4 is not presented in a buildable order**. §4.2 is the
second section and is the *last* thing that can be built: its call-site snippet
consumes `worstStatus` (§4.7, unnamed) and `seedCapacityOne` (§4.3). An
implementer working top-to-bottom stalls on page one of the design.

Here is an order that works, with the atomicity boundaries the plan never states.
Every stage below leaves the tree compiling and `go test ./...` green.

1. **§4.1 label** + T1 + T3's label half. Standalone; one line.
2. **§4.5 abort gate** + the `gui/bundle_flow.go:535` comment correction + T5 +
   T8. Standalone. Land it *before* §4.6 so T5 does not need the census press.
3. **§4.6 census** + **§5.1(b)'s three walk repairs, in the same commit** + T6 +
   T5's added press. Atomic: `confirmReviewScreen` only advances on a press and
   `pumpUntil` never presses, so `TestEngraveSingleSigFlowFull/WatchOnly/Template`
   go red the instant the census lands.
4. **§4.3 `seedCapacity` + §4.4 `buildSeedInventoryLines` + all 8 existing
   `buildPlateInventoryLines` call sites + T7 + T4.** Atomic — a signature change
   with 8 dependents (measured: `multisig_build_prose_test.go:369,424,425`;
   `multisig.go:362`; `multisig_build.go:479`;
   `multisig_build_perseed_passphrase_test.go:134,246,304`). Note that §4.3 says
   "all 8" and then lists 9 entries including the not-yet-existing
   `singlesig.go:136`, and §8.4 says "Eight call sites now carry an argument" when
   the post-change count is 9. Measure, do not transcribe.
5. **§4.7c `verifyStatus` + `buildVerifyStatusLines` + the (unnamed) ranking
   comparator** + T9 + T14. Pure additions; nothing breaks.
6. **The single-sig verdict→status mapping** (I-4) — `singleSigVerifyFlow` gains a
   return. This is where the plan's "first step" actually belongs: fifth.
7. **§4.7b `multisigRestoreDocFlow` signature** + `multisig.go:361`,
   `multisig_build.go:478` **and `multisig_nested_name_test.go:230`** (I-5).
   Atomic.
8. **§4.2 `restoreDocFlow` signature** + `singlesig.go:136`. Depends on 4, 5 and 6.
9. T2, T10, T11, T12, T13.

Coherent intermediate states exist at every one of those nine boundaries, so this
is emphatically not all-or-nothing — but the implementer has to derive that
themselves, and the two atomic sets (4 and 7) are the ones where a partial commit
leaves a red tree.

---

## WHAT I FOUND CLEAR AND EXECUTABLE

Named because a report that only lists defects misrepresents the artifact.

- **§1's measured-facts section does its job.** Every claim I spot-checked
  against the fork held: `restoreDocFlow`'s one production / zero test call
  sites, the four `bundleEngrave(ctx` sites and which two gate, the eleven exit
  points of `singleSigVerifyFlow`, the five `multisigVerifyResult` constants, the
  three false comment sites, `buildFullModeLabel`'s two arms. §1.8's
  "this is the current shape and explicitly NOT the shape to mirror" flag is
  exactly the right way to keep a superseded measurement in a document.
- **§4.1, §4.5 and §4.6 are executable as written** — a stranger could land all
  three from the snippets alone, and §5.1(b)'s diagnosis of why the three walks
  break (`pumpUntil` pumps but never presses; `confirmReviewScreen` loops until a
  press) is correct and gives the exact repair.
- **§4.4's wording rationale is the strongest part of the plan.** The
  `numberedLabel` n=1 argument — *each* over one unnumbered plate makes a reader
  conclude a plate is missing — is a real defect caught before it was written, and
  the three "on purpose" clauses (YOUR not THE, no sufficiency claim, no address
  claim) each cite the code that makes the alternative false.
- **§4.7's incentive argument is airtight and correctly resolved.** Any gate keyed
  on FAILED teaches the operator to skip the verify; the document's facts are
  seed-derived, not plate-derived. Both are load-bearing and both are measured.
- **§4.7a's enumerated table is right.** All ten rows are correct under the
  intended rule, including the two that were previously wrong (`failed →
  abandoned`, `incomplete → complete`). It is the *algorithm* that disagrees with
  it, not the table.
- **§5.2's per-test refinements are unusually good** — the pager/`s5PageForNeedle`
  point, the "T5 needs no engraver" point, the T8-needs-a-positive-half point.
  These are exactly the things an implementer rediscovers at cost.
- **§8's blind-spot list is honest and specific**, including naming the template
  branch that §3 omitted.

---

## OUTSIDE MY LENS

One line each, as instructed — not findings, not re-litigated.

- Prior lenses (adversarial funds, §4-snippet executability, test falsifiability,
  fold-vs-findings, spec coverage) are closed; I did not re-run them.
- `append(append(status, lines...), extra...)` aliases `status`'s backing array if
  `buildVerifyStatusLines` ever returns a slice with spare capacity — benign as
  specified (one line, len == cap), worth a comment, not a finding here.
- §3's header says *"S6a contains four items"* over a five-row table (C-1 was
  added in R0 and the sentence was not propagated), and *"they land in the same
  fifteen lines of the same function"* is not true of §4.1 or of §4.4's
  `multisig_build_census.go` edits — clarity, not correctness.
- §6.1 records "~40 file:line locations" and a gate output of "76 / 76"; §6.2
  records "41" operator strings. The current runs report 90 and 44. The numbers
  are labelled "R0 fold:", so they are historical by construction — but a reader
  cannot tell from the plan whether the gate was re-run after the last fold.
- §4.2's "a **leading** `status` and a **trailing** `extra`" describes *line*
  placement while both parameters sit at the end of the parameter list; the
  snippet disambiguates it.
