# S6a R15 — VERIFY THE TWO FOLDS + BOUNDED ATTACK

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Folds under review:** `f71ef44`, `4f40f1f`
**Prior report:** `design/agent-reports/s6a-r14-verify-attack.md` (RED, 0C/3I)
**Code:** `/scratch/code/shibboleth/seedhammer`, main = `b8a23bf3dcf45f0b996bedf8b17f7141f092d282`

## VERDICT: RED — 0 Critical, 1 Important (+ 0 filed)

R14's three Importants are genuinely fixed, all three propagated to every place
the old statement lived (verified below in full, not spot-checked). Fold #2
(controller-found, unreviewed) fixes three real defects of its own and its
factual/numeric claims all check out against the fork. But fold #2 also
introduces one new internal contradiction: its acceptance-criteria table
reintroduces, in the very row next to it, the exact term ("`verifyStatus`")
that fold #1 had deliberately replaced with the architecturally correct one
("`verifyRecord`") one round earlier — and the claim it makes with that term is
demonstrably imprecise for single-sig specifically, per the document's own
words elsewhere in the same file.

---

### I-1 — Step 1's new acceptance criteria say the eleven exits map to
`verifyStatus`, contradicting the row's own artifact name and the document's
own "unreachable by construction" text [MECHANICAL]

**Where:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md:1062` (build-order
step 1's cell, fold #1's wording) vs. `:1082` (step 1's acceptance-criteria
table, fold #2's wording, unreviewed) vs. `:1167-1169` (pre-existing §5.2 text,
untouched by either fold).

**The defect.** Line 1062 (fold #1, in direct response to R14 I-2) deliberately
renamed the step-1 artifact from *"the single-sig exit → `verifyStatus`
mapping"* to:

    | 1 | ... (a) the single-sig eleven-exit → `verifyRecord` mapping, ...

`verifyRecord` is the correct target: `singleSigVerifyFlow` gains an
out-parameter `rec *verifyRecord` (§4.7b-seam, `:849`, by analogy with
`multisigVerifyFlow`), and each exit's job is to set one or both of the two
booleans inside it (`fullPassRecorded`, `adverseRecorded`) — `verifyStatus` is
then *derived*, once, by a single fixed switch (§4.7a, `:764-768`) that is not
part of the eleven-exit mapping at all and is scheduled separately, at build
step 2.

Fold #2's new acceptance-criteria row, one column over in the exact same table
row, reintroduces the replaced term:

    | (a) the eleven-exit mapping | ... each maps to one of the four
    `verifyStatus` values; ...

This is not merely imprecise wording — it is factually wrong for two of the
four values on the single-sig path specifically. The document's own
pre-existing §5.2 text (untouched by both folds) says so directly:

    :1167-1169  "T23 AND T24 CANNOT RUN ON THE SINGLE-SIG PATH... Single-sig
    has no retry loop -- its verify is a one-shot `if sel == 0 { ... }` -- so
    `failed -> abandoned` and `incomplete -> complete` are unreachable by
    construction."

`failed → abandoned` and `incomplete → complete` are the two transitions that
land on `statusVerifiedOnRetry` (needs a prior adverse write *and* a later pass,
impossible within one non-looping call) and, for the "abandoned" default case,
require reaching `statusNotFullyChecked` from *inside* the eleven exits, which
also cannot happen (the zero-value default state is what a *skipped* verify —
never calling `singleSigVerifyFlow` at all — leaves behind, not something any
of its eleven internal exits writes). So of the eleven exits, at most two of
the four `verifyStatus` values are ever reachable (`statusCheckDidNotPass` via
the ten adverse returns, `statusVerified` via the one fall-through) — not "one
of the four" in the sense the acceptance criterion implies.

**Evidence.**
```
$ grep -n "maps to\|mapping table\|the mapping\b" design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
874:mapping table must carry the fall-through as a row like any other.
1082:| (a) the eleven-exit mapping | ... each maps to one of the four `verifyStatus` values; ...

$ grep -n "verifyRecord\` mapping\|verifyStatus\` mapping\|verifyStatus\` values" design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
1062:| 1 | ... (a) the single-sig eleven-exit → `verifyRecord` mapping, ...
1082:| (a) the eleven-exit mapping | ... each maps to one of the four `verifyStatus` values; ...

$ sed -n '1167,1169p' design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
**T23 AND T24 CANNOT RUN ON THE SINGLE-SIG PATH, and §5 put every test there
(R3 I-3).** Single-sig has **no retry loop** -- its verify is a one-shot
`if sel == 0 { ... }` -- so `failed → abandoned` and `incomplete → complete` are
unreachable by construction.

$ cd /scratch/code/shibboleth/seedhammer && sed -n '90,140p' gui/singlesig.go | grep -n "singleSigVerifyFlow"
43:		singleSigVerifyFlow(ctx, th, full, template)
```
Confirmed independently in the fork: the verify choice
(`gui/singlesig.go:126-132`) is a single `if sel, ok := verifyChoice.Choose(...); ok && sel == 0` —
no loop, no re-offer — so `singleSigVerifyFlow` executes at most once per
engrave session, matching the document's own "no retry loop" claim.

**The harm.** Bounded by strong mitigating context, so I am not calling this
Critical: the acceptance table sits directly under, and cross-references
(§4.7b-seam), the exact boolean-based design; T25's own row two hundred lines
away ("no verdict is read... only the two recorded booleans") reinforces it;
and the row's own second, operative clause — "no exit maps to a pass state on a
path the device did not observe passing (G2)" — is stated correctly and is the
part that actually gates anything. But if a reviewer or implementer applies
*only* this acceptance row literally — treating "each maps to one of the four
`verifyStatus` values" as a per-exit **write target** rather than a downstream
derived consequence — the natural implementation is to have exits assign a
`verifyStatus` value directly, bypassing `passRecord`. That reproduces exactly
R9's C-1, the historical bug this whole four-state design exists to close
(`:918`: "a status enum has already lost the mode, and the mode is exactly what
the pass line must not lose") — `verifyStatus` alone cannot carry `full` (T22)
or `suppliedCosigners` (T27), only `passRecord` can. This is the same
propagation shape the document's own §4.2 retrospective names — a term
corrected in one place, reintroduced uncorrected one column over — occurring a
fifth time, inside the fold that fixed the fourth instance.

**Remedy — UNVERIFIED.** Presumably: reword `:1082`'s clause to reference
`verifyRecord` (or `passRecord`'s two fields) as the write target, and state
the `verifyStatus` correspondence as a downstream/derived fact rather than a
per-exit mapping target — e.g. "each exit sets `fullPassRecorded` and/or
`adverseRecorded` such that, once run through §4.7a's switch, the result is
never a pass state on a path not observed passing." Not resolved against the
call graph.

---

## PART 1 — R14's THREE FINDINGS: FIXED / PARTIAL / NOT FIXED

**I-1 (build-order step 7 said "four source assertions," contradicting §5.1's
corrected count) — FIXED, fully propagated.**

```
$ grep -n "four source assertions\|twelve sites" design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
1068:| 7 | ... update **all TWELVE call sites plus the stub** (§5.1) ... four source
     assertions to the new call **verbatim**, eight direct `multisigVerifyFlow(...)`
     sites and the stub closure to the new arity | ...
1227:**All twelve sites and the stub are updated in step 7's single commit** ...
1228:four source assertions to the new call **verbatim, never loosened to a ...
```
The build-order cell (`:1068`) and §5.1's own prose (`:1227-1228`) now agree
exactly on the count and the breakdown (4 verbatim assertions + 8 arity
updates + 1 stub = 13 touch points, "twelve sites and the stub"). Only one
place in the whole document ever stated a count for this ("four source
assertions" as a total, or "twelve sites and the stub") and it is now
consistent everywhere it appears. Independently re-measured against the fork
(unchanged since R14, since neither fold touched code):
```
$ grep -rn "multisigVerifyFlow(" --include="*.go" gui/ | grep -v "func multisigVerifyFlow\|var multisigVerifyFn" | wc -l
8
$ grep -rn "multisigVerifyFn(" --include="*_test.go" gui/ | wc -l
4
$ grep -n "multisigVerifyFn = func" gui/multisig_engrave_tail_walk_test.go
105:	multisigVerifyFn = func(ctx *Context, th *Colors, full bool, expectedSlots []int,
```
8 + 4 + 1 stub = 13, matches. **FIXED.**

**I-2 (T27 and the `suppliedCosigners` formula's review step were absent from
the build order) — FIXED, fully propagated, both halves.**

Half 1 (the formula's review checkpoint): step 1's cell now carries it
explicitly as artifact (b), with an explicit backward reference —
`design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md:1062`: *"(b) the
`suppliedCosigners` expression — ... **(b) was asserted in prose and scheduled
nowhere — R14 I-2.**"* — and fold #2 additionally gave it acceptance criteria
(`:1083`) it did not have even after fold #1. Half 2 (T27's build-order
placement): `:1068`, step 7's list now reads *"T11, T20, T23, T24, T25,
**T27**"* — confirmed via the same grep the R14 report ran:
```
$ grep -n "T27\b" design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
1068:| 7 | ... plus **T11, T20, T23, T24, T25, T27** — ...
1161:| **T27** | **the PATH axis (R12 C-1, R13 C-1).** ...
```
T27 now appears in exactly two places: its test-table row and the build order,
matching every other T2x test's pattern. **FIXED.** (See Part 3 below for
whether step 7 is the *correct* place, not just *a* place — it is.)

**I-3 (§4.7c, the document's own stated sole authority for the printed lines,
never learned the cosigner clause) — FIXED, fully propagated.**

Fold #1 edited §4.7c's own table directly (the exact location R14 named as the
authority R13 assigned it), not just the separate obligation table:
```
$ sed -n '938,939p' design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
| pass, no adverse | `statusVerified` | ... **and appends `Other cosigners' keys
  are taken as supplied.` iff `rec.pass.suppliedCosigners > 0`** |
| pass, adverse | `statusVerifiedOnRetry` | the generated pass line **including
  the cosigner clause on the same condition**, plus `An earlier check did not
  pass; a later full check passed.` |
```
Both pass rows carry the clause and its condition now, word-for-word
consistent with the obligation table's READ row (`:899`: *"renders **iff
`rec.pass.suppliedCosigners > 0`**"*). Fold #1 additionally added an explicit
forward-looking rule at `:943-946`: *"§4.7c IS THE SOLE AUTHORITY FOR WHAT THE
BUILDER PRINTS, and it must therefore carry every clause... Any future clause
lands **here first**."* — closing the exact propagation gap R14 found and
naming the mechanism so it does not recur a third time. `grep -n -i "cosigner"`
(full output read) shows only the expected 8 hit-groups (struct comment,
obligation-table heading/prose, obligation-table rows, §4.7c's two pass rows,
the new sole-authority paragraph, T27's row) — no orphaned or contradicting
description remains. **FIXED.**

## PART 2 — FOLD #2 (`4f40f1f`), UNREVIEWED TEXT

Three pieces of new text: (1) the measured-exits block under §4.7b-seam
(`:860-877`), (2) step 1's "two artifacts" prose fix (`:1071-1074`), (3) the
new step-1 acceptance-criteria table (`:1079-1084`).

**(1) Measured-exits block — independently re-verified, exact.**
```
$ cd /scratch/code/shibboleth/seedhammer && awk 'NR>=65 && NR<=149 && /return/{print NR}' gui/singlesig_verify.go
69 78 90 98 112 117 125 130 138 146
$ sed -n '145,149p' gui/singlesig_verify.go
		showError(ctx, th, "Verify Failed", "The read-back bundle does NOT match the seed. Check the engraved plates.")
		return
	}
	showNotice(ctx, th, "Verify OK", "The engraved bundle matches the seed.")
}
```
Ten explicit `return`s, closing brace at line 149 after the `showNotice` at
148 — matches the plan's claim exactly, including which specific line numbers
are which kind of exit.
```
$ grep -rn 'singleSigVerifyFlow(' --include='*.go' .
gui/singlesig_verify.go:65:func singleSigVerifyFlow(ctx *Context, th *Colors, full, template bool) {
gui/singlesig.go:132:		singleSigVerifyFlow(ctx, th, full, template)
```
Exactly one call site, no stub, no test callers — matches the plan's claim.
No contradiction found in this block; the one defect it interacts with is I-1
above (the acceptance-criteria table appended right after it).

**(2) "Two artifacts" prose fix — FIXED, no new duplicate found.**
```
$ grep -n "table of eleven rows\|table of.*rows\|BOTH.*artifacts\|two artifacts" design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
1062:| 1 | **Write BOTH step-1 artifacts** ...
1073:eleven rows *and* the `suppliedCosigners` expression — **both** reviewed before
```
Only one live description of what step 1 produces remains, and it agrees with
the build-order cell above it. The stale "It produces a table of eleven rows"
singular-artifact sentence fold #2's own commit message describes replacing is
gone — confirmed by absence, not by trusting the commit message.

**(3) Acceptance-criteria table — see I-1 above.** Row (b) (`suppliedCosigners`
acceptance) was independently checked against the fork and matches: `keys` /
`covered` are both genuinely in scope at `gui/multisig_verify.go:987` (R14
already walked this function end to end and this fold does not touch it); "0
on every single-sig path" and "counts keys not covered" are consistent with
the obligation table's WRITTEN row (`:898`). No defect found in row (b).

## PART 3 — T27 SCHEDULABILITY

**T27 is genuinely schedulable at step 7 — not merely asserted there —
by the identical reasoning the document already applies to T20.**

T20's placement rule (`:1063`, `:1068`): its assertion needs "call sites [through
production flows] that do not exist until step 7," because step 7 is *"Wire the
verify status into all three flows"* (`:1068`) — the first point at which *any*
flow has a verify-status line wired into its rendered document at all. Before
step 7, no call site anywhere produces a rendered document carrying a verify
status line, for any flow — so no pure-function test at step 2 or elsewhere can
satisfy "asserted through a production flow, on the rendered document" (§5.1's
own standard, already settled and not re-litigated here).

T27's own row (`:1161`) needs the same thing for a narrower claim: a clean pass
on the single-sig flow (clause absent) and a clean pass on *a* multisig flow
(clause present) — "both flows" (2), not T20's "all three rendered documents."
Since step 7 is the *first* point either family's flow has the status line
wired at all, step 7 is also the *earliest possible* point T27's prerequisites
exist — identical to T20's situation, just needing one flow of each of the two
families rather than call sites through all three. No earlier build-order step
(1-6) wires verify status into any rendered document, so T27 cannot be pulled
earlier, and nothing later than step 7 is required either (single-sig is wired
at step 5-6, multisig gets its `&rec` argument and status-line wiring at step 7
alongside the twelve-site arity change) — so step 7 is not just *a* valid slot,
it is the *only* valid slot.

One caveat, not a scheduling defect and not gated here: T27's row states its
mutation-catching condition requires observing the clause present on a
multisig pass (`suppliedCosigners > 0`), which per the obligation table's
WRITTEN row means a policy with at least one key *not* covered by a verified
leg — i.e., not the self-multisig "operator holds every slot" fixture R14
already checked (`open == 0`, `gui/multisig_build.go:96`, correctly yields
`suppliedCosigners == 0` there). The plan does not name which existing walk
fixture (if any) already has partial coverage suitable for T27, or whether one
must be constructed at step 7. This is an implementation-time detail of the
same kind the plan already leaves open for other tests at their landing step
(e.g. T7c's exact walk parameters), not a build-order placement defect, so
filed rather than gated: **T27's non-vacuity depends on a multisig test
fixture with `suppliedCosigners > 0` existing or being built at step 7; not
specified by name anywhere in this plan.**

## PART 4 — ATTACK

Bounded, focused pass beyond the two folds' own diffs: re-walked the record
end-to-end for any new gap fold #1/#2 might have opened, re-confirmed the
twelve-sites-and-stub arithmetic against the fork fresh (not cited from R14 —
see Part 1, I-1), checked every new sentence fold #2 added against the rest of
the document for contradiction, and checked T27's placement against the
document's own reasoning pattern (Part 3).

The one defect found is I-1 above, which is really a **Part 2** finding (an
unreviewed fold introducing a documentation-only contradiction) rather than an
attack on the design — it does not, on its own, produce a wrong result under
the design as actually specified elsewhere in the same document; it produces a
wrong result only if an implementer reads the acceptance-criteria row in
isolation and ignores the row one column to its left plus roughly 200 lines of
directly-relevant, already-correct surrounding text. **No separate, additional
attack finding beyond I-1 was found.** Specifically checked and cleared:

- No new stale duplicate of "table of eleven rows" or "four source assertions"
  anywhere else in the document (`grep -n` for both, full output read, no
  hits outside the corrected locations already quoted above).
- Fold #2's `singlesig_verify.go` line citations (`:65`, `:69`...`:146`,
  `:148`, `:149`, `:65-149`) all resolve to what they claim — checked by
  reading the file directly, not the plan's description of it.
- The single-sig one-call-site claim does not contradict the "twelve sites"
  multisig claim; they describe different functions and both are internally
  consistent with the fork.
- No G1/G2 violation traced to any literal instruction in the plan as a
  whole — the one contradiction found (I-1) is confined to a review
  checklist's imprecise wording, not a build step's literal instruction.

"Nothing else found" is a genuine result of this pass, not an unsearched gap.
