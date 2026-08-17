# S6a R14 — VERIFY THE FOLD'S CLAIMS + ADVERSARIAL ATTACK

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Fold under review:** `git diff 2453561..HEAD -- design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
(commit `83d935d`, "reports+tooling+plan: fold R13 -- DECLARING IS NOT WIRING, for the second time")
**Prior report checked against:** `design/agent-reports/s6a-r13-verify-attack.md` (RED, 1C/1I)
**Code:** `/scratch/code/shibboleth/seedhammer`, `main` = `b8a23bf3dcf45f0b996bedf8b17f7141f092d282`
**New gate under review:** `scripts/plan-wiring-check.sh`

## VERDICT: RED — 0 Critical, 3 Important (+ 0 filed)

R13's C-1 (`suppliedCosigners` declared, never wired) is genuinely fixed at the
one place that matters most — the write-site value is now a real, in-scope,
computable definition, not a guess. But the fold only edited three spots
(§4.7b-seam's obligation table, the T27 test row, §5.1's blast-radius list) and
did not propagate into the two tables the document itself calls authoritative —
§4.7c ("the sole stated authority for what `buildVerifyStatusLine` prints",
R13's own words) and §4.8 (the build order). All three findings below are the
same failure mode this document's own §4.2 retrospective already names: **"Folds
fail by incomplete propagation — the facts get corrected and the duplicates are
left standing."** It recurred a third time, inside the fold whose whole purpose
was closing the second instance of exactly this class.

---

### I-1 — Build-order step 7 still says "four source assertions," contradicting
§5.1's own corrected count in the same document [MECHANICAL]

**Where:** §4.8's build order, step 7's cell (current line 1036) against §5.1
(current lines ~1171-1187), both un-diffed and diffed respectively by this fold.

**The defect.** This fold edited §5.1 to replace "All four must be updated in
step 7's commit... plus the `multisigVerifyFn` stub assignment" with "**All
twelve sites and the stub** are updated in step 7's single commit — the four
source assertions... and the eight call sites plus the stub to the new arity."
That correction is accurate (verified below, Part 1 §2). But §4.8's own
step-7 row — the document's "what to do first" summary, i.e. the artifact an
implementer follows procedurally — was **not touched by this diff** and still
reads: *"Wire the verify status into all three flows, plus T11, T20, T23, T24,
T25 — and update the **four source assertions** pinning the `multisigVerifyFn`
call (§5.1) in this same commit."*

**Evidence.**
```
$ grep -n "four source assertions\|twelve sites" design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
1036:| 7 | ... and update the **four source assertions** pinning the `multisigVerifyFn` call (§5.1) ...
1187:**All twelve sites and the stub are updated in step 7's single commit** — ...
```
The build-order cell cites §5.1 by name for its own scope, and §5.1 (as of
this fold) no longer agrees with the number the cell quotes.

**The harm.** An implementer following the build-order table literally — which
is what it exists for — updates 4 sites and stops, leaving 8 more
`multisigVerifyFlow(` call sites and the `:105` stub at the old 5-argument
arity. Per R13's own analysis (unchanged, re-confirmed below), that is a
package-wide Go compile error across at least 6 files and upward of a dozen
test functions, not a soft test failure. This is the identical shape R12 rated
Important (I-2, for T20's placement) and R13 rated Important (I-1, for the
8-site undercounting) — recurring here as a documentation-propagation gap
inside the very fold meant to close the second one.

**Remedy — UNVERIFIED**, per instruction. Presumably step 7's cell needs its
count corrected to match §5.1; not resolved against the call graph here.

---

### I-2 — Neither T27 nor the `suppliedCosigners` write-formula's claimed
review step appears anywhere in the build order [MECHANICAL]

**Where:** §4.7b-seam's new obligation table (current line 875, *"The exact
expression is the implementer's step-1 output... reviewed before step 2 for the
same reason [as the eleven-exit mapping]"*) and §5's test table's new **T27**
row, against §4.8's build order (current lines 1030-1037), untouched by this
fold.

**The defect, two parts:**

1. **The "step-1 output" claim doesn't match step 1's own scope.** §4.8's
   step-1 cell reads: *"Write the single-sig exit → `verifyStatus` mapping
   (§4.7c) and get it reviewed | eleven exits, and every later step depends on
   it."* §4.8 separately states *"Step 1 is a gate, not a task. It produces a
   table of **eleven rows**..."* — i.e. step 1 is explicitly and only about
   `singleSigVerifyFlow`'s eleven exits mapping to the four `verifyStatus`
   values. `suppliedCosigners`'s write-site is `multisigVerifyFlow`'s single
   success return (`gui/multisig_verify.go:987`) — a different function, a
   different table, not one of the eleven rows. Nothing in §4.8 schedules a
   review of the `suppliedCosigners` formula at all.
2. **T27 is absent from every build-order cell.** T27 (per its own row, "Assert
   on the rendered line of both flows") needs the same prerequisite the
   document already worked out for T20 — production call sites through both
   flows, which "do not exist until step 7" (§4.8, T20's own placement note,
   R12 I-2). Every other test in the T20-T26 family is explicitly named at its
   landing step (T21/T22/T26 at step 2; T20/T23/T24/T25 at step 7) — T27 is
   named at neither.

**Evidence.**
```
$ grep -n "T27\|T21, T22, T26\|T11, T20, T23, T24, T25" design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
1031:| 2 | ... + **T21, T22, T26** | ...
1036:| 7 | ... plus **T11, T20, T23, T24, T25** ...
1121:| **T27** | **the PATH axis (R12 C-1, R13 C-1).** ...
```
T27 appears in the test table (line 1121) and nowhere in the build order.

**The harm.** Not a compile-break like I-1 — a scheduling gap. The document
elsewhere explicitly warns about the consequence of writing a test before its
prerequisites exist: T5/T23/T24's discussion (§5.2) shows a misplaced test can
"pass vacuously, never reaching the sequence they name." T27 is the one test
this cycle added specifically to catch a regression of R13's Critical
(`suppliedCosigners` reverting to unwired). A test written at the wrong point
because the build order gives no instruction is exactly the failure mode that
would let that regression back in silently, with a green suite. This is
JUDGEMENT on how likely an implementer is to mis-place it (a careful one would
infer T27 belongs with T20 by the same reasoning already in the document), but
the *absence of the instruction itself* is MECHANICAL and demonstrated above.

**Remedy — UNVERIFIED.** Presumably: cite `gui/multisig_verify.go:987` at step
1 or add a companion mapping table for it, and add T27 to step 7's list
alongside T20. Not resolved against the call graph.

---

### I-3 — §4.7c, the document's own stated sole authority for what
`buildVerifyStatusLine` prints, was not updated to mention the cosigner clause
[MECHANICAL]

**Where:** §4.7c "THE FOUR LINES" (current lines 899-913) against §4.7b-seam's
new obligation table (current lines 860-876), both in §4.7 but not
cross-referenced.

**The defect.** R13's C-1 named three missing pieces: a write-site, **"a
generation rule"** (explicitly located at §4.7c: *"§4.7c's table is the one
place the plan says what each of the four rendered lines actually contains"*),
and a test. This fold's new obligation table supplies the write-site and the
test, and states the read rule in prose (*"in `buildVerifyStatusLine`'s pass
arm: the clause ... renders iff `rec.pass.suppliedCosigners > 0`"*) — but
§4.7c's own "pass, no adverse" cell, the location R13 named by section number,
is untouched and still reads only: *"generated from the pass record — names
exactly the comparisons this mode ran, and states what was not read"* — no
mention of a cosigner clause or `suppliedCosigners`.

**Evidence.**
```
$ grep -n -i "cosigner" design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
830, 839, 841   — inside passRecord's own struct-comment (declaration)
860, 864        — the new "MUST BE WRITTEN, READ AND TESTED" heading + prose
871, 872        — the new obligation table (WRITTEN / READ rows)
1121            — T27's row
```
Zero hits inside §4.7c's own table (lines 899-913) or its surrounding
paragraph. The document now carries two descriptions of what generates the
"pass, no adverse" line, one silent on the cosigner clause and one explicit
about it, uncross-referenced.

**The harm.** Lower than I-1/I-2: the specification is genuinely present,
unambiguous, and sits two subsections earlier in the same top-level section
(§4.7), directly upstream of what it describes — an implementer reading §4.7
straight through would very likely see both. This is why it is Important and
not Critical: unlike R13's original C-1, an implementation following the plan
literally does not necessarily drop the clause — it depends on whether §4.7c
alone is treated as authoritative, which is exactly the role the prior report
assigned it. Recorded as a real internal-consistency defect per item 5 of the
brief, not manufactured to fill a quota.

**Remedy — UNVERIFIED.** Presumably add one clause to §4.7c's "pass, no
adverse" cell text or a forward-reference to §4.7b-seam's table. Not resolved
against the call graph.

---

## PART 1 — CLAIMS VERIFIED

**1. Is `suppliedCosigners` now actually wired — all three ways?**

- **WRITE site is genuinely computable, not asserted.** Read
  `multisigVerifyFlow` end to end (`gui/multisig_verify.go:662-988`). At the
  success return (`gui/multisig_verify.go:986-987`, `showNotice(...)` then
  `return verifyComplete`), both `keys` (`_, keys, err :=
  md.ExpandWalletPolicyChunks(readbackMd1)`, line ~721, the full wallet
  policy) and `legs` / `covered` (the per-slot map populated across the
  per-seed loop, only ever set for slots in `expectedSlots`) are function-local
  variables genuinely in scope at that line — confirmed by reading the
  function, not by trusting the citation. "The number of policy keys NOT
  covered by a verified leg" is a real, computable value there (e.g. count of
  `keys` indices absent from `covered`), matching the fold's claim.
  **Additionally checked the R13 "landmine"** (self-multisig, operator holds
  every slot, `open == 0` per `gui/multisig_build.go:96`): under this precise
  definition the value correctly evaluates to 0 in that scenario — there
  genuinely are no unverified cosigners left, so 0 is the *correct* answer,
  not a collision with single-sig. The fold's more precise wording resolves
  the landmine R13 flagged as merely unresolved; it does not just restate it.
- **Single-sig has exactly one place to write 0.** `singleSigVerifyFlow`
  (`gui/singlesig_verify.go:65-149`) is confirmed `void` today, with exactly
  **eleven** exit points (10 explicit `return` statements plus the fall-through
  `showNotice(ctx, th, "Verify OK", ...)` at line 148 — counted directly, not
  cited) — matches the plan's "eleven exits" claim independently. The one true
  success exit is unambiguous; writing a hard-coded 0 there is trivial and
  correctly reasoned ("no policy cosigners" is definitionally true for
  single-sig).
- **T27 asserts on the rendered line of both flows, not a helper**, per its
  own text: *"Assert on the rendered line of both flows"* — matching the
  established pattern (T11, T7c) this document already uses to distinguish
  production-flow assertions from helper-level ones. This is a plan-level
  claim (T27 does not exist yet); accepted as adequately specified. **But see
  I-2** — T27 is not scheduled to any build-order step, so *when* it gets
  written, relative to its prerequisites, is unspecified.
- **Where the claims fall short: I-2 and I-3 above** — the generation rule is
  specified, but not at the location the prior report called authoritative,
  and the write-formula's claimed review checkpoint doesn't exist in §4.8.

**2. The twelve call sites + stub (R13 I-1) — re-verified exhaustively.**

```
$ grep -rn "multisigVerifyFlow(" --include="*.go" gui/ | grep -v "func multisigVerifyFlow\|var multisigVerifyFn"
gui/multisig_verify_report_test.go:38, :348, :576
gui/multisig_verify_flow_test.go:118, :224, :250
gui/multisig_supply_multislot_test.go:271
gui/multisig_verify_policy_test.go:177
```
Exactly 8, exactly the 8 the fold lists — no more, no fewer.
```
$ grep -rn "multisigVerifyFn(" --include="*_test.go" gui/
gui/multisig_verify_report_test.go:1079, :1081
gui/multisig_verify_flow_test.go:373, :394
```
Exactly the 4 source-assertion needles, unchanged from R13. Stub confirmed at
`gui/multisig_engrave_tail_walk_test.go:105` (`multisigVerifyFn = func(ctx
*Context, th *Colors, full bool, expectedSlots []int, engravedMd1 []string)
multisigVerifyResult {...}`). **4 + 8 = 12, "and the stub" = 13 touch points —
the fold's "twelve sites and the stub" phrasing is exact.**

**Checked for sites the plan still misses, per the brief's instruction to grep
exhaustively:**
- `singleSigVerifyFlow(` has exactly **one** call site,
  `gui/singlesig.go:132` — confirmed by grep, zero test call sites. This
  matches R13's own "checked and cleared" list (it was already accounted for
  there); this fold does not change it and no new gap exists.
- The **two production call sites through the `multisigVerifyFn` seam**
  (`gui/multisig.go:336`, `gui/multisig_build.go:452`) will also need a
  `&rec` argument once the var's function type changes. These are **not**
  separately itemized in the "twelve sites" list, but they don't need to be:
  they are the actual subject of step 7's core instruction ("wire the verify
  status into all three flows"), not collateral test-blast-radius the way the
  12 are — this is a scope distinction, not an omission. Not filed.
- No other stub/seam assignment exists: `grep -rn "multisigVerifyFn\b"` over
  `gui/` returns only the two production call sites, the var declaration, the
  `:105` stub, and the 4 test needles already accounted for — nothing new.

**3. `restoreDocFlow`/`multisigRestoreDocFlow` blast radius — re-confirmed
unaffected and still accurate**, machine-checked fresh (not merely cited from
R13): `restoreDocFlow(` → 1 site (`gui/singlesig.go:136`);
`multisigRestoreDocFlow(` → 3 sites (`gui/multisig.go:361`,
`gui/multisig_build.go:478`, `gui/multisig_nested_name_test.go:230`). Matches
§4.2/§4.8 exactly; this fold did not touch this area and nothing regressed.

## PART 2 — ATTACK FINDINGS

**4. The record end to end, all three flows.** Unchanged by this fold except
for the `suppliedCosigners` sub-path, so R13's own trace still holds and was
re-walked, not merely cited: `rec` is a caller-local `verifyRecord` value
(never a pointer at the call boundary) passed by address in both single-sig
(`gui/singlesig.go`, declared before the verify-offer choice,
conditionally written by `singleSigVerifyFlow`, always read by
`restoreDocFlow(..., buildVerifyStatusLine(rec), ...)`) and multisig (via
`multisigVerifyFn`/`multisigRestoreDocFlow`) — the out-parameter cannot be
`nil` in production; a skipped verify leaves it at zero value, handled safely
by the `default:` arm. `suppliedCosigners` is now written (multisig success
return) and read (`buildVerifyStatusLine`'s pass arm, per the obligation
table) — no longer the "written nowhere, read nowhere" case R13 found. The
gap that remains is not "unwired" but "wired in a location the document's own
authority table doesn't reflect" (I-3) and "wired with no scheduled review or
test placement" (I-1, I-2).

**5. Whole-file consistency.** `grep -n "passRecord{"` → 1 hit, the
declaration only, no stale two-field literal. `grep -n "T9\b\|T13a\|T13b"` → 1
hit, the same historical corrective sentence as R13 found, not a live
citation. `grep -n "sawUnaccounted"` → 1 hit, inside the new obligation
table's own retrospective sentence, correctly historical. **New in this
round:** the "four source assertions" vs. "twelve sites and the stub"
contradiction (I-1) and T27's absence from the build order (I-2) — both are
exactly the class of stale-cross-reference defect this check exists to catch,
and both are new, introduced by this fold's partial edit.

**6. Build-order step 1 / first-commit red.** Step 1 remains a review artifact
(a table), unchanged by this fold — cannot be "red" in a build-gate sense; no
new blocker. **Step 7 is a different story**: if followed literally per its
own cell text (I-1), the first commit containing step 7 would be a package-wide
Go compile error (not enough arguments in call to `multisigVerifyFlow`) at 8
test call sites plus a stub type mismatch — this reproduces R13's I-1 exactly,
inside the artifact meant to have fixed it.

## THE NEW GATE — DOES IT WORK

`scripts/plan-wiring-check.sh` parses `type X struct {...}` and `const (...)`
blocks, extracts field/member names, and flags any name with zero mentions
outside its own declaration block. Run fresh:

```
$ ./scripts/plan-wiring-check.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
wired  suppliedCosigners        3 mention(s) outside its declaration
... (11 declared names checked, 0 never used elsewhere)
exit=0
```

It does what its header claims for the narrow thing it claims to do: it would
have caught R13's finding (all 3 of `suppliedCosigners`'s original mentions
were inside its own declaration block; now 3 are outside).

**False negative, demonstrated against this exact round's own findings, not
hypothetical.** The gate reports `suppliedCosigners` "wired" with 3 outside
mentions — but two of those three are lines 871-872 of the *same* new
obligation-table paragraph (a single contiguous prose block), and the gate
cannot distinguish "mentioned in three independent, load-bearing places" from
"mentioned three times in one paragraph that itself never touched the
document's declared sole-authority table (§4.7c)." I-3 above is a real,
demonstrated gap the gate reports GREEN on. More fundamentally: **I-1 and I-2
are entirely outside what this gate can ever see.** "T27," "four source
assertions," and "step 1" are not struct fields or const members — they are
markdown table prose — so the gate's regex never inspects them at all,
by its own documented scope ("It only sees `type X struct` or `const()`
blocks"). Two of this round's three Important findings are structurally
invisible to the new gate, not merely missed by chance. The gate's own header
admits "mention ≠ correctly wired"; this round shows that gap is not
theoretical — it is exactly where this round's findings live.

## GATES — ACTUAL OUTPUT

```
$ export PATH="/nix/var/nix/profiles/default/bin:$PATH"
$ cd /scratch/code/shibboleth/mnemonic-engrave
$ for g in verify-returnsite-sweep plan-cite-check plan-glyph-check plan-table-check plan-wiring-check; do
    ./scripts/$g.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md; echo "$g exit=$?"
  done
```
(stdout and stderr captured separately for each; no cross-stream contamination
observed)

- `verify-returnsite-sweep.sh` → **15 verdict return sites; 0 unrowed. exit=0**
  (same documented scope limitation as R13: single-sig contributes zero sites
  until `singleSigVerifyFlow` gains a verdict; multisig-only coverage)
- `plan-cite-check.sh` → **107 / 107 citations resolved, 0 dangling. exit=0**
  (up from R13's 102/102 — the new write-site, call-site, and stub citations
  this fold added all resolve; this machine-corroborates Part 1 §2's manual
  grep)
- `plan-glyph-check.sh` → **60 operator strings scanned, 0 undrawable. exit=0**
- `plan-table-check.sh` → **80 table rows checked, 0 malformed. exit=0**
  (up from 76 — the new obligation table + T27 row, both well-formed)
- `plan-wiring-check.sh` → **11 declared names checked, 0 never used
  elsewhere. exit=0** — see "THE NEW GATE" above for what this GREEN does and
  does not mean.

The brief's header says "run all six"; the loop it specifies lists five
script names. Ran exactly the five named; no sixth was specified. All five
GREEN. None of them can see I-1, I-2, or I-3 — all three were found by reading
the cited sections directly and cross-checking their claims against §4.8 and
§4.7c, not by any gate.

## WHAT WAS NOT DONE

No codebase audit for pre-existing defects. No prose/markdown review beyond
the specific consistency checks the brief requested. No re-litigation of the
four-state design, the goals, §0.1, or NG1/NG2. All three findings above are
IN SCOPE under §0.1 — they concern G2's mechanism (whether the fix for R13's
Critical is actually reachable as specified) and plan-internal consistency,
not the device's epistemic-state reporting; nothing here expands what the
document claims about what the device knows. Remedies are marked UNVERIFIED
and not resolved against the call graph, per instruction.
