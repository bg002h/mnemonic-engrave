# S6a R16 — DID THE R15 FOLD LAND CLEAN?

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Fold under review:** `4c40973` (diff `5588e98..4c40973`)
**Prior report:** `design/agent-reports/s6a-r15-verify-attack.md` (RED, 0C/1I)
**Code:** `/scratch/code/shibboleth/seedhammer`, main = `b8a23bf3dcf45f0b996bedf8b17f7141f092d282`

## VERDICT: RED — 0 Critical, 1 Important (+ 0 filed)

R15's I-1 is genuinely FIXED and fully propagated — no other place in the
document still describes the step-1 mapping as producing statuses. Part (a)
(the acceptance-row rewrite) and part (c) (T27's non-vacuity addition) both
check out clean against the fork. But part (b) — the new "only two of the four
states are reachable" paragraph the fold added as a self-described "mechanical"
check for the future step-1 reviewer — makes a factual claim about the code
("the ten adverse returns") that contradicts the document's own established
classification rule when checked against one of those ten returns directly,
and the contradiction is load-bearing: applied literally, it steers a future
step-1 mapping toward exactly the class of G2 violation this whole cycle exists
to prevent.

---

### I-1 — The fold's new "ten adverse returns" claim misclassifies at least one
of `singleSigVerifyFlow`'s ten `return` sites against the document's own
established adverse/benign rule, and the paragraph's own mechanical check would
reject the *correct* classification as wrong [MECHANICAL]

**Severity: Important.**

**Where:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md:1097-1104` (new
paragraph, fold `4c40973`) vs. `:756-757` (§4.7a's definition of `adverseRecorded`,
untouched) vs. `:774-786` (§4.7b's adverse/benign table, untouched, R12-audited
CLEAN) vs. `gui/singlesig_verify.go:65-149`.

**The defect.** The fold's new paragraph states, as a fact the step-1 reviewer
can apply "mechanically":

    :1097-1099  "from *inside* the eleven exits, only **two** of the four
    states are reachable on single-sig — `statusVerified` (the fall-through)
    and `statusCheckDidNotPass` (the ten adverse returns)."

    :1103-1104  "**A proposed mapping that reaches either of those two
    [statusVerifiedOnRetry, statusNotFullyChecked] from within the eleven
    exits is wrong**, and that is a check the reviewer can apply mechanically."

This presupposes all ten `return` statements in `singleSigVerifyFlow` are
*adverse* under the document's own definition (`:757`: "Written at any return
site whose world-set contains a bad-plate world"). That is false for at least
one of them.

`gui/singlesig_verify.go:89-90`:
```
89:		showError(ctx, th, "Verify Bundle", "Couldn't re-derive the bundle from the seed.")
90:		return
```
This fires when `deriveSingleSigBundle` fails on the just-re-typed seed —
before any plate, card, or readback has been touched. Compare
`gui/multisig_verify.go:896-897`:
```
896:				showError(ctx, th, "Verify Bundle", "Couldn't re-derive the bundle from the seed.")
897:				return verifyFailed
```
Byte-identical message, identical failure (re-derivation from the freshly
re-typed seed fails, nothing about the plates observed yet). The plan's own
§4.7b table — untouched by this or any fold, and already independently audited
CLEAN by R12 — classifies this **exact** multisig case as **BENIGN**, not
adverse:

    :774-784 (table)
    | adverse (world-set contains a bad-plate world) | benign (nothing observed about the plates) |
    | `gui/multisig_verify.go:719` foreign-or-garbled md1 | `gui/multisig_verify.go:897` re-typed seed will not derive |

So by the document's own established rule, applied to the identical failure
mode, `gui/singlesig_verify.go:90` is a **benign** exit — it should write
*neither* `fullPassRecorded` nor `adverseRecorded`. (The same reasoning
plausibly extends to the four user-cancellation exits at `:69`, `:78`, `:112`,
`:125` — no `showError` call, matching the table's "abandons"/"structural
refusals" benign entries — and to `:98`'s `templateizeBundle` failure, the same
class as `:90`. The finding does not depend on those; one exact, tabled
counterexample is sufficient to falsify "the ten adverse returns.")

**This directly contradicts the fold's own next sentence:**

    :1101-1103  "`statusNotFullyChecked` is the zero cell, which is what
    *never calling the flow* leaves behind, not something an exit writes."

A benign return *is* an exit, reached *from inside* a call to the flow, that
writes neither boolean — landing exactly on `statusNotFullyChecked`. The
paragraph's own two claims cannot both be true.

**The harm is not merely internal inconsistency.** §4.7c's rendered line for
`statusCheckDidNotPass` (`:938`, untouched) reads:

    "A verification check ran and did not pass: a comparison did not match,
    or a plate could not be read or accounted for. Do NOT rely on this
    backup..."

If a step-1 author (or the future reviewer, following this fold's own
"mechanical" instruction) marks `:90`'s exit adverse — because the paragraph's
"the ten adverse returns" told them to, and because the paragraph's mechanical
rule would flag the *correct* (benign) classification as "wrong" for landing on
`statusNotFullyChecked` from inside the exits — then a run where the operator
simply re-types a seed that fails to derive, before any plate is ever read
back, renders "A verification check ran and did not pass." No check ran. That
is a literal instance of the two things §0.1 names as G2: the device vouching
for nothing it observed, and claiming a check it did not perform — the same
shape of defect this entire four-state design exists to make structurally
impossible, reproduced on the negative arm instead of the positive one R9's
C-1 broke.

**Mitigating context, and why this is Important rather than Critical.** The
formal acceptance criteria in row (a) (`:1082`, R15 I-1's fix) does not itself
repeat "the ten adverse returns" — its operative text is the general G2
principle ("no exit writes a pass record on a path the device did not observe
passing") plus the requirement that every exit name a `verifyRecord` write.
§4.7b's classification methodology, which actually governs the correct answer,
is untouched and remains correct. A step-1 author who does the per-site
classification work the way §4.7b prescribes (rather than trusting this new
summary paragraph) still lands correctly. The defect is confined to explanatory
text adjacent to the gate, not the gate itself — the same shape R15 used to
keep its own I-1 at Important rather than Critical. Unlike R15's I-1, though,
there is no nearby correct clause specifically guarding the *adverse* side the
way row (a)'s G2 clause guards the *pass* side, so this is a less-mitigated
Important, not a Minor.

**Remedy — UNVERIFIED.** Presumably: drop the parenthetical "(the ten adverse
returns)" and the mechanical rejection rule for `statusNotFullyChecked`, or
replace it with "at least one of the ten explicit returns and the fall-through"
without asserting how many of the ten are adverse — that classification is
step 1's actual job, not something this paragraph should pre-empt. Not
resolved against what step 1 will eventually produce.

---

## PART 1 — R15 I-1: FIXED

`grep -n '`verifyStatus`'` and `grep -n "maps to\|eleven-exit"` (both run
against the folded plan) show exactly one place in the whole document that
still names `verifyStatus` in connection with the eleven-exit mapping —
`:1085-1089`, the new "why (a) forbids naming a `verifyStatus` value"
paragraph, which *explains* why it must not be named, not an instance of the
defect. Line `:1062` (build-order step 1's cell) and `:1082` (the
acceptance-criteria row) now agree: both describe the mapping's target as
`verifyRecord`/its two booleans, never `verifyStatus`. No orphaned instance of
"each maps to one of the four `verifyStatus` values" remains anywhere in the
document. **FIXED, fully propagated.**

## PART 2 — NEW DEFECTS IN THE FOLD

**(a) Rewritten acceptance row.** Checked against the rest of the document
(Part 1 above) and against §4.7a/§4.7b's terminology — consistent, no defect.

**(b) The reachability paragraph.** See I-1 above — **defect found.**

**(c) T27's non-vacuity addition.** `gui/multisig_build.go:96` reads
`open := p.N - len(p.SelfSlots)`, confirmed by direct read — supports the
plan's claim that the self-multisig "operator holds every slot" fixture yields
`open == 0`. This text is carried over verbatim from R15's own Part 3 caveat
(already independently checked there against `gui/multisig_verify.go:987`'s
`keys`/`covered` scope, not re-derived here per the brief). No defect found.

## PART 3 — BOUNDED ATTACK

One additional pass beyond (a)/(b)/(c) above, confined to what the fold
touched: checked the fall-through's own claim ("only reachable via a pass") —
`singleSigVerifyFlow` is straight-line code with no loop or goto; the only path
to the line-148/149 fall-through is falling through every prior `return`,
including the `verifySingleSig` check at `:144-146`, so the fall-through
cannot be reached without that call returning nil. That part of the fold's
claim holds. No second, independent defect was found beyond I-1.

**No G1/G2 violation was found in the formal acceptance criteria (row (a) or
(b)) or in T27's addition** — the one defect found (I-1) is confined to the
explanatory "consequence" paragraph adjacent to the gate, not the gate itself.
