# S6a round 8 — CHEAP PRE-REVIEW VERIFICATION PASS on the R7 fold

**Fold checked:** `git diff 74e34e4..HEAD -- design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
(commit `deedf4a`, "P5 replaces P4 as the enforceable property; the table fails OPEN")
**Answers to:** `design/agent-reports/s6a-r7-adversarial.md` (3C/3I) and
`design/agent-reports/s6a-blindspot-pass-2.md`
**Code:** `/scratch/code/shibboleth/seedhammer` @ `b8a23bf` (fork `main`)
**Scope:** mechanical/factual verification only — no design opinion on whether
P5 is the right property.

---

## VERDICT: DIRTY — 4 false, 3 stale, 3 structural

The fold's headline fixes (the C-1 scoped pass line, the C-2 missing rows, the
P5 property statement) are each individually correct and verified against the
running code. But the fold **never updated the actual switch statement** that
is supposed to realize the fix, **never declared the constant** its own new
test (T17) requires, and **contradicts its own new sentences within the same
new block** in two separate places. This is the same "changed in prose, not in
the mechanical artifact" failure mode the fold's own commit message warns
about, reproduced inside the fold itself.

---

## THE C-1 SCOPED-LINE CLAIM

**Verified true, all four sub-claims.**

- A full single-sig run cuts three cards: `singleSigEngraveCards`
  (`gui/singlesig_engrave.go:20-45`) appends `cardMS1` only under `full`, then
  unconditionally appends `cardMK1` and `cardMD1`. Confirmed by direct read.
- `singleSigReadbackCards` (`gui/singlesig_verify.go:23-42`) switches only on
  `case cardMK1` / `case cardMD1` — no `cardMS1` arm exists. Confirmed.
- `multisigVerifyOKMessage`'s doc comment (`gui/multisig_verify.go:1023-1038`,
  read directly) says, verbatim: *"the success notice, SCOPED TO WHAT WAS
  CHECKED"* and *"IT DOES NOT CLAIM THE SEED PLATES WERE COUNTED... the device
  cannot know how many ms1 plates exist and must not imply it checked them
  all."* Confirmed verbatim.
- The two new scoped lines ("`Key and descriptor plates VERIFIED: read back
  and matched. The ms1 you typed matched this seed -- no ms1 plate was read.`"
  and the repeat-check analogue) were checked against `bundle.Verify`
  (`bundle/verify.go:32-104`, read in full): `readback.MS1` is the
  hand-typed string, `derived.MS1` comes from the seed re-typed at verify
  time, and the leg compares recovered *entropy*. "The ms1 you typed matched
  this seed" is therefore literally what was checked — it claims nothing about
  the ms1 **plate**, exactly the C-1 remedy. No over-claim found.

## THE SEVEN NEW ROWS — |W| AUDIT

All seven sites resolved directly against `gui/multisig_verify.go` by reading
`multisigVerifyFlow` end to end (lines 662-988) and cross-checked with
`grep -n 'return verify'`.

| site | plan's claim | verified |
| --- | --- | --- |
| `:670` | `verifyRefused`, no expected slots, `statusDidNotComplete` | correct |
| `:680` | `verifyRefused`, no engraved md1, `statusDidNotComplete` | correct |
| `:696` | `verifyAbandoned`, operator declined to present plates | correct |
| `:794` | `verifyRefused`, "`verifyFreshSlots` failed — a structural refusal **before any readback**" | **wrong on two counts.** (1) `:794`'s `ferr != nil` branch is reachable only if `verifyFreshSlots` receives an empty `expected` slice (`bundle/multisig_verify.go:318-320` — read directly), but `expectedSlots` is already checked non-empty at `:668-670` and never mutated — **the path is unreachable**, exactly as R7's own return-site table already noted ("unreachable (guarded at `:668`); ok") and the fold's new row does not say so. (2) "before any readback" is false: by the time `:794` executes, the mk1/md1 readback over NFC has already completed (`:683-738`) and the operator has already typed a seed (`:774`) — readback happened first, not never. |
| `:938` | `verifyIncomplete`, "zero legs, correctable — the seed filled no slot this run engraved" | **partly wrong.** `correctable` is set true for **two** distinct reasons (`gui/multisig_verify.go:859`, `len(fresh)==0`; and `:886`, an ms1 typed-but-rejected). Only the first matches "the seed filled no slot"; in the second, the seed **did** fill a slot and the ms1 was rejected instead. `statusDidNotComplete`/`|W|=1` still holds either way (nothing is observed about the *plates* in either branch), so the classification is right, but the stated reason is only half true — the same shape as R7's already-filed M-1. |
| `:940` | `verifyAbandoned`, zero legs, not correctable | correct |
| `:987` | `verifyComplete`, success, `|W|=2`, scoped `statusVerified` | correct, and this is the row that actually fixes C-1 |

No `|W|=1` row was found asserting something ambiguous; the `:794`/`:938`
inaccuracies are about the *world description*, not about whether the routed
status is safe — both still land somewhere that claims nothing false.

## GATES — ACTUAL OUTPUT

    ./scripts/verify-returnsite-sweep.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
    -> verdict return sites: 15 ; unrowed in the plan: 0 ; exit=0

    ./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
    -> citations resolved: 116 / 116 ; dangling: 0 ; exit=0

    ./scripts/plan-glyph-check.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
    -> operator strings scanned: 56 ; undrawable: 0 ; exit=0

**All three confirmed exactly as the fold commit claims.** stderr was captured
separately for all three; empty in every case, so nothing is hiding behind a
wrapper.

**But the sweep's "15, 0 unrowed" says nothing about single-sig.** The script
scans both `gui/multisig_verify.go` and `gui/singlesig_verify.go`
(confirmed by reading `scripts/verify-returnsite-sweep.sh:48`), and its regex
is `return[[:space:]]+verify[A-Z]`. `singleSigVerifyFlow` is still declared
`func singleSigVerifyFlow(...)` — **void, no verdict type at all** — so
`grep -nE 'return[[:space:]]+verify[A-Z]' gui/singlesig_verify.go` returns
**zero matches** (confirmed directly). All 15 rowed sites are multisig-only.
Single-sig's eleven exits (10 explicit `return` + 1 fall-through, hand-counted
against `singleSigVerifyFlow`, `gui/singlesig_verify.go:65-149` — matches the
plan's own "eleven exits" claim) are **structurally invisible** to this gate
until step 1 of §4.8 gives them a typed verdict. "15/0" is real but is not
evidence the single-sig side — this cycle's actual subject — has any row
coverage yet.

I also ran `./scripts/plan-fold-sweep.sh` (inference mode, vs `HEAD~1`) for
completeness: reports clean, 1 term removed, no survivor. Not load-bearing —
the script's own header warns inference mode "misses any fold whose subject
IS the renamed term," and by construction it only catches an *old* term
surviving; every defect below is a *new* sentence contradicting another *new*
sentence, which this script does not and cannot check.

## STALE REFERENCES TO P4 / THE FIVE-STATUS DESIGN — AND THREE STRUCTURAL DEFECTS

### Structural (Critical) — the code the fold shows cannot do what the fold says it does

**S-1 — the §4.7a switch never reads `sawUnaccounted`; it cannot produce
`statusUnaccounted`.** Read the full code block at plan lines 730-749
character by character. `sawUnaccounted` is declared (`:732`) and set sticky
inside the classify switch (`:741-742`), then **never referenced again**. The
mapping switch is:

    switch {
    case res == verifyComplete && sawDisagreement: status = statusVerifiedOnRetry
    case res == verifyComplete:                    status = statusVerified
    case sawDisagreement:                          status = statusDisagreed
    default:                                       status = statusDidNotComplete
    }

There is no `case sawUnaccounted:` arm. Every observation the fold just spent
its whole C-2 section routing to `obsUnaccounted` — `errVerifyLegHasNoPlate`,
`:719`, `:724`, the hand-typed ms1 divergence, `:701`, `:738` — sets
`sawUnaccounted = true` and then, because nothing downstream reads it, falls
through to `default: status = statusDidNotComplete`. Confirmed by `grep -n`
across the whole document: `sawUnaccounted` appears at lines 732, 741-742 and
nowhere else; `case obs` appears only at 739 and 741; there is exactly one
`default:` in the file (line 748) and it assigns `statusDidNotComplete`, never
`statusUnaccounted`. This directly contradicts prose at plan line 808-811
("one more sticky fact — `sawDisagreement` and `sawUnaccounted`, with
**explicit switch-arm order**, DISAGREED outranking UNACCOUNTED") — that
switch-arm order is described but never written.

**S-2 — `statusUnclassified` (row 1 of the fold's own 7-row status table, and
the subject of the fold's own new test T17) is never declared as a Go
constant anywhere in the plan, and never assigned anywhere.** `grep -ni
unclassified` over the whole file returns exactly two hits: the status-table
row (line 981) and T17's row (line 1355). §4.7c's const block (lines
1179-1192, unmodified by this fold except for the `statusUnaccounted`
insertion — confirmed against the diff hunk `@@ -1008,6 +1185,7@@`, which adds
only that one line) has six constants: `statusNotVerified`,
`statusDidNotComplete`, `statusUnaccounted`, `statusDisagreed`,
`statusVerifiedOnRetry`, `statusVerified`. No seventh. T17 says "no known
return path reaches `statusUnclassified`" — true only in the vacuous sense
that the identifier does not exist in the plan's own code at all, not because
an exhaustive-default mechanism was built and proven unreachable. **P5(c)'s
described mechanism — "an unrecognized observation maps to the line making the
fewest claims" — has no implementation in the plan.** The switch's actual
`default:` arm produces `statusDidNotComplete`, a status whose own §4.7d
definition claims "an attempt ran and ended with no adverse observation about
the plates" — i.e. exactly the confident, fails-open behavior P5(c) exists to
eliminate. Both S-1 and S-2 mean: as written, this plan is not implementable
without an implementer silently inventing both the constant and the switch
logic the prose promises but the code omits.

**S-3 — §4.7e's own heading and lead sentence contradict its own body, and
both halves were added in the same fold hunk.** Confirmed against the diff:
the entire section from `#### 4.7e THE OBSERVATION TABLE — P4's enforcement
artifact` (plan line 1034) through `provenance alone...` (line 1119) is new
text from hunk `@@ -933,13 +976,147 @@` — a single 147-line insertion, not a
carry-over from an earlier round. Within that one insertion:

- Line 1034 (heading): *"THE OBSERVATION TABLE — P4's enforcement artifact"*
- Line 1036 (first sentence of the body): *"P4 is enforced by this table, and
  T15 tests it."*
- Line 1053, ~17 lines later, same section: *"THIS TABLE IS A REVIEW
  PROJECTION, NOT THE ENFORCEMENT MECHANISM (R7 / P5)."*

A heading and its own opening sentence assert the table **is** the
enforcement mechanism; the same block's own follow-up paragraph asserts the
opposite. Both are new. This is not staleness in the usual sense (old text
outlived by a new decision) — it is the fold writing the contradiction and the
correction in the same breath and never reconciling the heading with the
correction.

### False claims left standing after the fold (Important)

**F-1 — the observation-table row for the clean pass still miscites `:984`
as the success site.** Line 1044: `| every leg matched its plate | ':984'
success | ... | statusVerified |`. Directly executed: `grep -n 'return
verify' gui/multisig_verify.go` shows `:984` is `return verifyFailed` (the
**full**-comparator mismatch return) and `:987` is `return verifyComplete`
(the actual success return). This is R7's own M-2, filed against this exact
row, and it is **unfixed** — worse, it now directly contradicts the fold's
own new row 47 lines later (line 1091), which correctly names `:987` as "the
success path."

**F-2 — "Six rows..." undercounts the table directly above it.** Line 989:
*"Six rows, matching §4.7c's six constants and §4.7d's six knowledge
states."* The table it is describing (lines 979-987) has **seven** numbered
rows (1 `statusUnclassified` through 7 `statusDisagreed`). "Six constants" is
true of §4.7c's const block only because of S-2 (that block is missing
`statusUnclassified`) — so this caption is simultaneously an undercount of the
table and indirect confirmation of S-2.

**F-3 — "Every `|W| = 2` row above lands in `statusUnaccounted`... it is the
only line that can be true of a two-world observation" (lines 1102-1104) is
false, and is directly refuted by the fold's own new content ten lines
above it.** This is R7's own M-5, unfixed, and now made worse: the `:987` row
(line 1091, added in this same fold) is `|W| = 2` and lands in
`statusVerified` with a **scoped** line — not `statusUnaccounted`. So the
claim "every `|W|=2` row... lands in `statusUnaccounted`" is falsified by the
row immediately above it in the same section, and "the only line that can be
true of a two-world observation" is falsified by the very mechanism (scoping)
the fold used to fix C-1.

### Stale, non-gating (Minor — recorded, matches R7's own severity)

- T15 (line 1354) still says *"The §4.7d **observation table** is the test
  fixture"* — the table is in §4.7e, not §4.7d. R7's M-3, unchanged
  byte-for-byte from the pre-fold text (verified: `git show
  74e34e4:design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md | grep 'T15.*P4'`
  is identical to HEAD).
- Row 8 (`:897`, "re-typed seed will not derive... the operator mistyped the
  seed") is unchanged and still describes R7's M-1 defect (the real world-fact
  is a device-side encode failure, not an operator typo) — filed Minor by R7,
  left as-is, consistent with R7's own ruling that the conclusion
  (`statusDidNotComplete`, `|W|=1`) survives regardless.
- §4.8's build order (lines 1260-1271) still has no step for T15, T15b or T16
  (R7's M-3), and now **also** omits the three tests this fold itself added —
  T17, T18, T19. Six of the plan's property-enforcing tests currently have no
  owning build step, up from three before this fold.

---

## BOTTOM LINE

C-1, C-2's row content, and C-3's design rationale are sound and verified
against the running code. What is not sound is the fold's own **mechanism**:
the switch shown cannot produce the status the fold spent the whole round
adding, the constant that status needs was never declared, and the section
naming itself the enforcement artifact contradicts its own correction in the
same paragraph. None of this is a reach-for-a-lens finding — every claim above
was checked by direct file read or `grep -n` against the plan and the fork.
**Not safe to send to the next adversarial round as-is**: S-1 and S-2 alone
would make an implementer either invent code the plan never specified, or
transcribe a switch that silently defeats the fold's entire C-2 fix.
