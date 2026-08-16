# S6a R3 — independent adversarial review of the R2/R3 fold text

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Scope:** `git diff 9edc641..HEAD` — the text added by commits `cac34a0` (fold
S6a R2) and `0e8a558` (fold the R3 pre-verification finding). 217 insertions.
That text has been reviewed zero times; nothing else here is in scope.
**Code baseline:** `/scratch/code/shibboleth/seedhammer`, `main`, clean.
**Reviewer:** independent context, opus tier. Read-only — no file modified.

**Taken as settled per brief and NOT re-derived:** the render-not-gate operator
decision; cycle scope; the append shape at `gui/multisig_restore.go:106`;
`restoreDocScreen`'s `start := 0` + live Done; both retry-loop break conditions;
`multisigVerifyResult`'s five constants; `singleSigVerifyFlow`'s 11 exits;
`multisigVerifyIncompleteText`; `seedEntryFlow`'s payload admission; the
seed-handling byte-identity; the 90-citation and 44-glyph gates.

---

## VERDICT: RED — 2 Critical, 5 Important

---

### C-1 — NO reading of §4.7a's algorithm reproduces §4.7a's table. The severity ordering the table requires is not the one the section states, and §4.7c declares a third, contradictory one

**Where:** §4.7a, the algorithm block (plan `:684-694`), its severity line, and
the enumerated table (`:709-720`); §4.7c's `const` block (`:795-807`).

**The defect.** The fold's own justification for writing an algorithm was that
"the R1 fold reasoned about this in prose and got it wrong twice". The algorithm
is:

    worst := statusNotVerified          // the zero value; see §4.7c
    on each verify attempt:
        worst = max(worst, severity(verdict))
    if the FINAL attempt was a clean pass:
        status = verifiedFirstTry   if worst is no worse than complete
               = verifiedOnRetry    otherwise
    else:
        status = line(worst)

    severity, most severe first: disagreed > did-not-complete > not-verified > verified

`worst` is **seeded with a non-observation that outranks a clean pass.** Trace
table row 2 (`complete`, the single most desirable outcome in the cycle):

| step | value |
| --- | --- |
| init | `worst = not-verified` |
| attempt 1, verdict `verifyComplete`, severity `verified` | `max(not-verified, verified)` — and the stated order puts **`not-verified` ABOVE `verified`** — so `worst = not-verified` |
| final attempt was a clean pass? | yes |
| "worst is no worse than complete"? | `complete`'s severity is `verified`; `worst` is `not-verified`, which the stated order ranks **worse** → **false** |
| result | `status = verifiedOnRetry` |

So a **first-try clean verify prints**
`Plates VERIFIED on a repeat check, after an earlier read-back DISAGREED. Confirm they restore before relying on this backup.`
The table's row 2 says `VERIFIED` and its "worst seen" column says `verified`.
The algorithm and the table disagree, and the algorithm's output is a **false
statement of a disagreement that never occurred**, engraved on the durable
artifact.

It is worse than one bad row. Because `worst` starts at `not-verified` and `max`
is monotone non-decreasing, `worst` can **never** reach `verified`. The guard
"worst is no worse than complete" is therefore **unsatisfiable**, `verifiedFirstTry`
is **dead**, and the operator's own blessed clean-pass line
(`Plates VERIFIED: each plate was read back and matched.`, `s6a-c1-verify-tail-decision.md`
"TEXT", verbatim) is unreachable on every path in the plan.

**The alternative reading is also wrong, differently.** `worst` is initialised
from a `verifyStatus` constant, `verifyStatus` is an `int` type, and Go's builtin
`max` compiles over it. Under numeric `max` on §4.7c's declared order —

    statusNotVerified=0  statusDidNotComplete=1  statusDisagreed=2  statusVerifiedOnRetry=3  statusVerified=4

— `statusVerified` is the **largest**, so `max` treats a clean pass as the worst
thing seen:

| sequence | numeric `max` | prints | table says |
| --- | --- | --- | --- |
| `incomplete` → `complete` | `max(0,1)=1`, `max(1,4)=4` = `statusVerified` | bare `VERIFIED` | `VERIFIED on a repeat check` |
| `failed` → `complete` | `max(0,2)=2`, `max(2,4)=4` = `statusVerified` | bare `VERIFIED` | `VERIFIED on a repeat check` |
| `incomplete` → `failed` → `complete` | → `4` | bare `VERIFIED` | `VERIFIED on a repeat check` |

All three repeat-check rows collapse to bare `VERIFIED` — erasing a real
DISAGREED from the record, which §4.7a itself names as forbidden ("Printing a
bare `VERIFIED` instead would hide a real anomaly from the stranger reading the
page").

**The divergence is unavoidable, and I derived the constraint.** For row 2 to
print bare `VERIFIED`, `max`'s order must satisfy `not-verified ≤ verified`. For
rows 8–10 to print the repeat-check line, it must satisfy
`did-not-complete > verified` and `disagreed > verified`. A consistent order
exists — `disagreed > did-not-complete > verified > not-verified` — and it is
**neither** the order §4.7a states (`not-verified` and `verified` swapped)
**nor** the order §4.7c declares. The section that exists to replace prose with
an algorithm states an algorithm that cannot produce its own table under any
reading.

**Corroborating symptom, measured:** `grep -n "verifiedFirstTry"` over the plan
returns **one** hit, line 688 — it is used in the algorithm and defined nowhere.
§4.7c defines `statusVerified` / `statusVerifiedOnRetry`; the algorithm writes
`verifiedFirstTry` / `verifiedOnRetry`. `severity()`, `line()` and `max()` are
likewise undefined, and their domains do not line up: `severity` has **four**
ranks, `verifyStatus` has **five** constants, and `statusVerifiedOnRetry` has no
severity rank at all. This is R2 I-1's finding ("`verifyStatus` is never
defined") reintroduced in the fold that closed it, and it is the mechanism by
which `max`'s ordering became ambiguous.

**This defect was INTRODUCED by the fold.** The deleted text said "The flow keeps
the **most severe** outcome *observed*" — over observations only, which yields
`verified` for row 2 correctly. Making it precise added the seed value that
breaks it.

**The harm.** Either a first-try clean verify permanently records a disagreement
that never happened (reading A) — misleading the stranger the whole design is
written for into distrusting good steel and possibly re-cutting a correct
backup — or a genuine DISAGREED is silently upgraded to a bare vouch (reading
B), which is verbatim R1 C-1's Critical. Both are false statements on the
funds-safety artifact, the exact class §4.3, §4.4 and §4.7 exist to eliminate.
No T-row detects it: T9 asserts each status renders its own line (true under both
readings), T10 and T12 assert non-pass and repeat-check sequences, and **no row
asserts what a plain first-try `complete` prints**.

**Suggested remedy (UNVERIFIED):** state one ordering, in one place, that
satisfies `disagreed > did-not-complete > verified > not-verified`, and make
§4.7c's `iota` order that ordering so a numeric `max` is correct by construction;
then re-derive all ten table rows from it rather than asserting them.

---

### C-2 — "the FINAL attempt was a clean pass" is undefined when there were ZERO attempts, and its natural encoding is `multisigVerifyResult`, whose zero value is `verifyComplete`

**Where:** §4.7a's algorithm, the `if the FINAL attempt was a clean pass:` line,
against `gui/multisig.go:330-343` and `gui/multisig_build.go:446-459`; and
§4.7c's zero-value argument.

**The defect.** In both shipped loops, the verdict is declared **inside** the loop
body:

    for {
        sel, ok := verifyChoice.Choose(ctx, th)
        if !ok || sel != 0 { break }          // <- "Skip" leaves here, zero attempts
        res := multisigVerifyFn(...)          // <- res is scoped to the iteration
        if res != verifyIncomplete && res != verifyFailed { break }
        ...
    }

To evaluate "the FINAL attempt was a clean pass" **after** the loop, `res` must
be hoisted: `var res multisigVerifyResult` above the `for`. Its zero value is
`verifyComplete` — measured, `gui/multisig_verify.go:90`,
`verifyComplete multisigVerifyResult = iota`.

On the `S` row — the operator presses **Skip** at the first offer, which §4.7c
itself calls "the single most common outcome" — the loop breaks before
`multisigVerifyFn` is ever called. `res` is still `verifyComplete`, "the FINAL
attempt was a clean pass" evaluates **true**, and the document prints a VERIFIED
line over a verify that never ran.

The plan spends a whole subsection (§4.7c) arguing that the zero value must be
the safe one, and names this exact hazard verbatim — *"Mirroring
multisigVerifyResult's shape would have made verifyComplete = iota = 0, so the
SAME omission would print 'Plates VERIFIED' over plates nothing ever checked.
That is the whole Critical, reachable by forgetting one assignment."* — and then
its own algorithm, one subsection earlier, introduces a **second** state variable
whose type is `multisigVerifyResult` and whose zero value is exactly that. The
hazard was diagnosed and re-committed in the same fold.

The algorithm nowhere says how "the final attempt" is represented, nor that a
zero-attempt run must not reach that branch. §3.2's surviving claim that "the
retry loop's control flow does not change" is technically true and actively
unhelpful here: hoisting a declaration is not control flow, so the sentence reads
as permission for exactly the encoding that breaks.

**On the single-sig path the term is well-defined but for a different reason.**
`gui/singlesig.go:130-133` is a one-shot `if sel, ok := ...; ok && sel == 0 { singleSigVerifyFlow(...) }`
— at most one attempt, so "final" is unambiguous when it exists. With zero
attempts (Skip / back out) the term is equally undefined there, and single-sig
has no verdict variable at all today, so the plan must specify one; it does not
(see I-4).

**The harm.** A restore document that prints `Plates VERIFIED: each plate was
read back and matched.` over a set nothing ever read back, on the commonest
operator action in the section, on both multisig paths. That is the original
C-1 Critical restored in full. Nothing in §5 detects it — **T14 pins the zero
value of `verifyStatus`, not of `multisigVerifyResult`**, and the `S` row appears
in no T-row's stated assertion.

**Suggested remedy (UNVERIFIED):** make "an attempt happened, and it was clean" a
property of `worst` (a value in the new type, which the fold already proved has
the safe zero) rather than of a hoisted `multisigVerifyResult`, and state in
§4.7a what the branch evaluates to when the attempt count is zero.

---

### I-1 — §4.7c's zero-value guarantee does not reach the seam §4.7b and §4.2 actually specify: the parameter is `status []string`, whose zero value is "no status line at all"

**Where:** §4.7c's `const` comment vs. §4.7b's and §4.2's signatures.

**The defect.** §4.7c: *"A path that forgets to set a status prints 'NOT
VERIFIED' -- conservative and true-ish -- and can never print a vouch."* That is
true of the **variable inside the flow**. It is not true of the **seam**, which
both §4.2 and §4.7b specify as `status []string`, not `verifyStatus`. The zero
value a caller can forget at the seam is `nil`, and
`append(append(nil, lines...), extra...)` renders a document with **zero** status
lines.

That is not a hypothetical omission: it is the state the operator decision
declares must be **unrepresentable** — `s6a-c1-verify-tail-decision.md`,
assumption 4, *"a document with none is a defect, not a default"*, and its WHY
section, *"A document with no status line must be unrepresentable, so silence can
never be mistaken for a pass."* §4.7 restates it: *"A rendered document with no
status line is a defect, not a default."* The chosen seam makes it a one-token
default, and there is already a caller that renders a restore document this way
(see I-2).

Not Critical: both **production** call sites are specified to pass a built
status. But the fold's central safety argument — that the type's ordering makes
omission safe — does not survive the translation into the parameter it chose,
and the plan asserts that it does.

**Suggested remedy (UNVERIFIED):** pass the `verifyStatus` value and let
`restoreDocFlow` / `multisigRestoreDocFlow` call `buildVerifyStatusLines`
themselves, so the zero value the fold proved safe is the one a caller can
actually forget.

---

### I-2 — "at both call sites" is wrong: `multisigRestoreDocFlow` has THREE callers, and §1.8 — the blast-radius section — was extended to cover it without extending the measurement

**Where:** §4.7b (*"so `multisigRestoreDocFlow` **does** change signature, at both
call sites"*), §3.2 (*"Both multisig call sites change"*), §1.8's new paragraph.

**The defect.** Measured, not recalled:

    $ grep -rn "multisigRestoreDocFlow(" --include="*.go" .
    gui/multisig_restore.go:100:func multisigRestoreDocFlow(... extra []string) {
    gui/multisig.go:361:            multisigRestoreDocFlow(ctx, th, tpl, keys,
    gui/multisig_nested_name_test.go:230:    multisigRestoreDocFlow(ctx, &descriptorTheme, tpl, keys, nil)
    gui/multisig_build.go:478:              multisigRestoreDocFlow(ctx, th, tpl, keys,

Three callers: two production and **`gui/multisig_nested_name_test.go:230`**,
which passes `nil` for `extra` today. That test —
`TestRestoreDocNestedNameIsActuallyDrawn` — drives a **real, rendered restore
document** through `runUITouchRaster` + `pumpUntil`, so under the new signature
the minimal repair (`..., nil, nil)`) produces an executing, green test that
renders a restore document with **no verification status line** — a live
counterexample to §4.7's normative guarantee and to I-1 above. §5.1's "updated,
not weakened" inventory covers `buildPlateInventoryLines` call sites and the
three census-blocked walks; this one appears nowhere.

The fold rewrote §1.8 — the section whose entire job is *"Blast radius of the
signature change"* — to declare that **both** functions now change signature, and
left the measurement covering only `restoreDocFlow` ("exactly one production call
site and zero test call sites", with the grep pasted). The equivalent grep for
`multisigRestoreDocFlow` is never run, though §4.3 four sections later
demonstrates the plan knows to enumerate test call sites ("Call sites (all 8,
measured)" — production and test).

This is the plan's own recurring class, restated by this very fold: round 0 said
one false-comment site, round 1 said two, this fold says three and lectures that
the table is "pasted from `sed` output rather than transcribed" — while the count
in the adjacent paragraph is off by one in the same direction.

---

### I-3 — the incentive invariant as stated is FALSE on two rows of the plan's own table, by the plan's own severity ordering

**Where:** §4.7a, *"**The incentive invariant this must satisfy, and which R1
violated twice:** running the verify can never produce a worse line than skipping
it."*

**The defect.** Rows 3 and 4 of the table:

| sequence | prints |
| --- | --- |
| `incomplete` then stop | `DID NOT COMPLETE` |
| `refused` / `abandoned` | `DID NOT COMPLETE` |

and row 1 (`S`, skip) prints `NOT VERIFIED`. §4.7a's own severity line ranks
`did-not-complete` **strictly above** `not-verified`. So running the verify and
having it end incomplete/refused/abandoned produces, by the plan's own ordering,
a strictly worse line than never running it. The reachable path is mundane and
does not require operator error: press "Verify now", the NFC gather fails or the
operator backs out → `verifyRefused`/`verifyAbandoned` → `DID NOT COMPLETE`;
press "Skip" → `NOT VERIFIED`. That is verbatim *"running the check is the way to
lose something"*, the trap the C-1 decision names as decisive.

The very next sentence silently narrows the claim — *"Every row above is at least
as good as `NOT VERIFIED` **whenever a pass was achieved**"* — to a different,
weaker, and true proposition. Two propositions in adjacent sentences, only the
weak one satisfied, and the strong one is the one set in bold and used as the
criterion by which R1's design is judged wrong twice.

This matters beyond wording because the two propositions prescribe opposite
repairs. If the bold version is the requirement, rows 3 and 4 are defects and the
operator's own blessed four-state mapping (`DID NOT COMPLETE` for
incomplete/refused/abandoned, `s6a-c1-verify-tail-decision.md` "DECISION") is a
declared departure the plan does not declare. If the narrow version is the
requirement, then the bold sentence is false and cannot be used to justify the
ranking. The plan asserts both and resolves neither.

---

### I-4 — R2 I-2 is NOT closed: the single-sig exit→status mapping is still absent, `singleSigVerifyFlow`'s required signature change is never stated, and §4.2's `worstStatus` has no specified provenance

**Where:** §4.7c's *"Single-sig's mapping must be written, not left to the
implementer (R2 I-2)"* paragraph; §4.2's call-site snippet.

**The defect.** R2 I-2's title is *"the single-sig exit→verdict mapping is still
unspecified"* — "still", because R1 had already been asked. The fold's response
restates the finding in the reviewer's own terms (*"the plan owes that table
rather than a resemblance"*) and then **defers it**: *"The implementer produces
the mapping as the **first** step of §4.7, and it is reviewed before the rest of
the section is built."* Zero of the eleven exits are mapped. The one exit R2 I-2
named as load-bearing — `gui/singlesig_verify.go:145`, the failed comparison, the
exit the whole cycle exists for — is mapped nowhere.

Two further consequences the fold did not pick up:

1. **`singleSigVerifyFlow` must gain a return type, and the plan never says so.**
   Measured: `func singleSigVerifyFlow(ctx *Context, th *Colors, full, template bool)`
   (`gui/singlesig_verify.go:65`), one caller (`gui/singlesig.go:132`). §4.7c
   states it "today returns nothing" as a fact about the defect; no section
   states the target signature, and §1.8 — the blast-radius section — does not
   cover it, though it covers `restoreDocFlow`.
2. **§4.2's call site uses a variable the plan never produces.**
   `buildVerifyStatusLines(worstStatus)` — `grep -n "worstStatus"` returns
   **one** hit in the whole plan, the call site itself. Nothing says where
   `worstStatus` is declared, what type it is, or how the §4.7a algorithm's
   `worst` becomes it on a path with no loop.

Under this project's own R0 rule, the plan is the artifact that must be GREEN
before implementation. Deferring a mapping whose worst case is `DID NOT COMPLETE`
printed over plates the device just said do not match — R2 I-2's stated harm —
moves a gated design decision past the gate. And R2 I-2's second half ("No T-row
in §5 asserts this mapping") is also untouched: T9 is over
`buildVerifyStatusLines`, T10/T12 are multisig loop sequences, T11 is index-0,
T13 is the invariant, T14 is the zero value. None asserts an exit→status mapping.

---

### I-5 — R2 I-3 is not closed by its own arithmetic: "at least T11 and one of T10/T12" pins at most TWO of the three document flows — verbatim the state I-3 called the defect

**Where:** §5's new closing paragraph, *"T9–T14 must pin a PRODUCTION CALL SITE,
not just the pure functions (R2 I-3)."*

**The defect.** The paragraph states the problem correctly — *"tests that would
all still pass if the status line were wired into two of the three document flows
and forgotten on the third"* — and then sets a requirement that permits exactly
that: **"At least T11 and one of T10/T12 drive a real flow to a real restore
document."** Two tests, and the paragraph names **no flow for either**. One flow
plus one flow covers at most two of the three call sites (`gui/singlesig.go:136`,
`gui/multisig.go:361`, `gui/multisig_build.go:478`). The third is unpinned by
construction, and the remedy sentence sits three lines under the sentence
describing that as the harm.

It is worse than a coin flip. R2 I-3 already established that T10/T12 are
sequence tests over the retry loop, drivable only on multisig via the
`multisigVerifyFn` seam (single-sig has no loop and no seam), while T11 is the
index-0 test naturally written on single-sig. So the likely satisfying pair is
single-sig + **one** multisig path, leaving the other multisig path — most
plausibly the BUILD path at `gui/multisig_build.go:478`, which sits behind
`if !template` and its own separate loop — with no assertion that its document
carries a status line at all.

**Second half of I-3, also untouched.** I-3 said: *"The fold added four rows and
**no §5.2 bullet for any of them**, so the costs a document-level status test
faces on the multisig paths (`newEngraver()`, `sh2DisplaySize`, `s5EngraveOnePlate`
per plate, the pager) are unstated for exactly the four tests that would hit
them."* Measured: §5.2 (plan `:939-970`) is **unmodified by this fold** — its last
bullet is T8. Two more rows were added (T13, T14) and the costs are still
unstated, for six rows now instead of four.

---

## ALGORITHM vs TABLE

Traced row by row. `S` = skip / never offered. "A" = the ordering §4.7a states
(`disagreed > did-not-complete > not-verified > verified`) applied to `max` with
`worst` seeded at `not-verified`. "B" = Go's builtin `max` over §4.7c's `iota`
constants (`statusNotVerified=0 … statusVerified=4`).

| # | sequence | table prints | reading A prints | reading B prints |
| --- | --- | --- | --- | --- |
| 1 | `S` | `NOT VERIFIED` | `NOT VERIFIED` ✓ | `NOT VERIFIED` ✓ *(but see C-2)* |
| 2 | `complete` | `VERIFIED` | **`VERIFIED on a repeat check` ✗** | `VERIFIED` ✓ |
| 3 | `incomplete` then stop | `DID NOT COMPLETE` | ✓ | ✓ |
| 4 | `refused` / `abandoned` | `DID NOT COMPLETE` | ✓ | ✓ |
| 5 | `failed` then stop | `DISAGREED` | ✓ | ✓ |
| 6 | `failed` → `abandoned` | `DISAGREED` | ✓ | ✓ |
| 7 | `failed` → `incomplete` | `DISAGREED` | ✓ | ✓ |
| 8 | `incomplete` → `complete` | `VERIFIED on a repeat check` | ✓ | **bare `VERIFIED` ✗** |
| 9 | `failed` → `complete` | `VERIFIED on a repeat check` | ✓ | **bare `VERIFIED` ✗** |
| 10 | `incomplete` → `failed` → `complete` | `VERIFIED on a repeat check` | ✓ | **bare `VERIFIED` ✗** |

**Every reading diverges.** Reading A loses the only unqualified pass in the
table and replaces it with a claim of a disagreement that never happened; reading
B loses all three preserved-anomaly rows and replaces them with a bare vouch. The
constraint set the table imposes on `max`'s ordering is
`disagreed > did-not-complete > verified > not-verified`, which is stated nowhere
in the plan. See C-1.

**Additionally, C-2 crosses the whole table:** row 1's correctness in both columns
above assumes an implementation that knows zero attempts occurred. Under the
natural hoist of `res` out of the shipped loops, row 1 prints a VERIFIED line in
**both** readings.

**Single-sig, on a path with no retry loop.** `gui/singlesig.go:130-133` is a
one-shot offer. With one attempt, "the FINAL attempt" is unambiguous; with zero
it is undefined (C-2). Reachable statuses there are `{NOT VERIFIED, VERIFIED,
DID NOT COMPLETE, DISAGREED}` — `verifiedOnRetry` is structurally unreachable,
which is correct but is stated nowhere, and under reading A it becomes the
**only** thing a successful single-sig verify can print.

---

## Minor / Nit (recorded, not gating)

- **M-1 — T13 is not executable as phrased, and names no mutation.** *"Over every
  row of §4.7a's table, no sequence containing a clean pass prints a line worse
  than `NOT VERIFIED`"* quantifies over rows of a markdown table, not over a
  program object; a Go test can only enumerate sequences it hard-codes, which
  makes T13 the union of T9/T10/T12's examples rather than "the property, where
  T10/T12 are instances". "Worse than" needs a total order over rendered lines
  that C-1 shows the plan states two contradictory versions of. And its mutation
  column reads "any ranking regression" — a class, not an edit — against §5's own
  standing rule that "the implementer reports, per test, the mutation applied and
  the failure message observed". Every other row names a concrete edit. Finally,
  T13's scope (`no sequence containing a clean pass`) excludes rows 3 and 4, the
  only rows that actually violate the invariant it is named for (I-3).
- **M-2 — T11 asks for an observation the tree provides no seam for.** *"the
  status line is at **slice index 0** of what `restoreDocScreen` receives —
  asserted **through a production flow**, not on a helper."* A production-flow
  test observes rendered frames; `restoreDocScreen` has no test seam (unlike
  `multisigVerifyFn`). The achievable assertion is "on the first frame", which
  §4.7b treats as equivalent to index 0 but which does not distinguish index 0
  from index 3. §5.2 has no bullet stating this cost (see I-5).
- **M-3 — §3.1 item 7 addresses one of the three spec locations the finding
  named.** The R2 spec-coverage Important cited
  `SPEC_seedhammer_T6a_singlesig_flagship.md:36/66/78`; item 7 quotes `:36` only.
  Verified: `:66` is the B5 acceptance gate ("restore doc: fp + descriptor +
  first recv/change addr match; greps clean of xprv") and `:78` is I-7. Neither
  is *falsified* by adding true non-secret lines, so this is staleness, not
  breakage. Item 7 also commits that "the spec is updated in this cycle, in its
  own commit" with no corresponding step in §6's validation gate and no entry in
  §7 — an obligation with no gate.
- **N-1 — `append(append(status, lines...), extra...)` is correct Go and carries
  no live aliasing hazard.** Checked all three callers: both builders return
  freshly-allocated slices (`buildVerifyStatusLines` returns "exactly one line",
  so `cap == len == 1` and the inner `append` must allocate), and
  `gui/multisig_nested_name_test.go:230` passes `nil`. The hazard — the inner
  `append` writing `lines` into a caller's spare capacity — is latent in the
  parameter type only, and no current or specified caller retains or reuses the
  slice it passes. Not a finding; recorded because the brief asked.
- **N-2 — the "every sequence" header overclaims** on an unbounded loop state
  space (arbitrarily many VERIFY AGAIN retries). Already recorded as a
  non-defect caveat by the R3 pre-verification pass; every omitted sequence
  reduces to a listed row's bucket. Not re-filed.

---

## WHAT I CHECKED AND FOUND SOUND

- **§4.2 and §4.7b now agree.** Both specify `status []string` leading and
  `extra []string` trailing, with the identical body
  `restoreDocScreen(ctx, th, append(append(status, lines...), extra...))`. §4.7b's
  closing sentence *"the same new leading parameter §4.2 adds to `restoreDocFlow`"*
  is now true. **The R3 pre-verification finding is closed.** ("Leading" describes
  position in the emitted slice, not in the parameter list — §4.2's call-site
  snippet passes `status` as the 8th argument, consistently.)
- **§4.3's "seed material" rewording closes R2 I-5 cleanly.** It is
  number-neutral and provenance-neutral, so it is true on the multi-seed BUILD
  path and on a payload-sourced run alike, where "the words you typed" was false
  and "your seed" wobbled. It is ASCII (no glyph-gate exposure). No shipped test
  pins the replaced clause — `grep -rn "still holding" --include="*.go" .` returns
  only unrelated hits in `gui/wipe_inventory_audit_test.go:275` and
  `engrave/engrave.go:1725`. The byte-identity claim for the `seedCapacityMany` +
  seed-on-plates arm is unaffected by this edit (that arm is untouched).
- **§4.7c's three-row false-comment table is accurate.** All three sites exist and
  say what the table quotes (`gui/multisig_build.go:439`, `gui/multisig.go:321-322`,
  `gui/multisig_verify.go:78-79`), and the third genuinely carries both errors —
  the type has five constants and `FOUR OUTCOMES, NOT A BOOL` sits two lines above
  them. R2 I-4 closed.
- **The zero-value *ordering* argument in §4.7c is correct on its own terms.**
  `statusNotVerified = iota` does make a forgotten in-flow assignment print
  `NOT VERIFIED` rather than a vouch, and the counterfactual it cites is real:
  `verifyComplete multisigVerifyResult = iota` at `gui/multisig_verify.go:90`.
  The defects are that the argument does not reach the seam (I-1) and that the
  algorithm reintroduces the hazard in a second variable (C-2) — not that the
  ordering is wrong.
- **§3.1 item 7 does not break cross-references.** The insertion renumbered the
  old item 7 → 8; the plan's only §3.1.N references are to §3.1.1, §3.1.2 and
  §3.1.6, all unaffected.
- **§4.7c's "eleven exit points" is exact.** Ten explicit `return`s plus the
  fall-through after `showNotice("Verify OK", ...)` — counted from
  `gui/singlesig_verify.go:65-149`.
- **§3.2's corrected cost paragraph is true** where it is checkable: five
  constants, no skip value, both multisig sites change signature (the count is the
  defect — I-2), and "the retry loop's control flow does not change" holds
  literally (hoisting a declaration is not control flow — though see C-2 for why
  that sentence is unhelpful here).
- **The two rows T10 and T12 target the right sequences** and their stated
  mutations are concrete and genuinely falsifying against the round-0 and round-1
  specifications respectively.
- **T14 is well-formed** and its mutation ("reorder the constants so
  `statusVerified` is 0") is concrete and would fail. It simply guards the wrong
  type to catch C-2.

---

*Reviewed the fold text only. Did not audit the codebase or the plan's unchanged
sections. No file modified.*
