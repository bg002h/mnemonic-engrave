# S6a R11 — CHEAP PRE-REVIEW VERIFICATION PASS

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md` (1395 lines)
**Fold checked:** `git diff dc42e27..HEAD -- design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
(spans commits `d5473f4` "close R9's residue" and `b83666c` "fold R10")
**Code:** `/scratch/code/shibboleth/seedhammer`, `main` = `b8a23bf` (matches plan's stated baseline)
**Answers:** `design/agent-reports/s6a-r10-goal-conformance.md` (1C/1I, both about the seam)
**Date:** 2026-08-16

---

## VERDICT: DIRTY — 0 false, 6 stale, 1 structural

The fold **correctly closes R10's C-1 and I-1**: the record now carries the mode
via a nested `passRecord{full, legs}` (not a bare `bool`), the builder returns a
`string` (not a `[]string` whose zero value is silence), and every seam code
citation checked against the fork resolves exactly as claimed. All three
machine gates the brief names come back clean at their claimed counts, run live.

What's dirty is **propagation debt left over from the T9–T19 → T20–T26 test
renumbering** (this fold renamed the tests at one location and missed several
siblings — this is the *incomplete propagation* class this pass exists to
catch), plus **two stale cached gate-output numbers** in §6.1/§6.2 that no
longer match a live run, plus **one markdown table row this fold itself broke**
(a dropped cell, not a content defect). Nothing here is a false engineering
claim and nothing reopens R10's C-1/I-1.

---

## SEAM CODE CLAIMS

All confirmed true against `/scratch/code/shibboleth/seedhammer` at `b8a23bf`.

| claim | check | result |
| --- | --- | --- |
| `gui/multisig_verify.go:986` has both `len(legs)` and `full` in scope at the success return | `sed -n '975,995p' gui/multisig_verify.go` | **true** — `showNotice(ctx, th, multisigVerifyOKTitle, multisigVerifyOKMessage(len(legs), full))` |
| `gui/singlesig.go:136` — single-sig call site | read | **true** — `restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path)` (pre-cycle signature, as expected — this is a plan, not yet implemented) |
| `gui/multisig_build.go:478` — build call site | read | **true** — `multisigRestoreDocFlow(ctx, th, tpl, keys, buildPlateInventoryLines(cardsOut, reg.passphraseFacts()))` |
| `gui/multisig.go:361` — supply call site | read | **true** — `multisigRestoreDocFlow(ctx, th, tpl, keys, buildPlateInventoryLines(cardsOut, oneSeedPassphraseFact(passphrase != "")))` |
| `gui/multisig_nested_name_test.go:230` passes `nil` | read | **true** — `multisigRestoreDocFlow(ctx, &descriptorTheme, tpl, keys, nil)` |

**Adverse/benign spot-check, 7 adverse + 8 benign (exceeds the 3+3 asked):**
`:719` foreign-or-garbled md1 → `verifyFailed`; `:724` decode failure → `verifyFailed`;
`:394` `errVerifyLegHasNoPlate`; `:738` plate-count mismatch → `verifyIncomplete`;
`:701` readback filter → `verifyRefused`; `:963` `verifyMultisigLegsPartial`
mismatch → `verifyFailed`; `:984` `verifyMultisigLegs` mismatch → `verifyFailed` —
all adverse, all confirmed. `:897` re-derive failure → `verifyFailed`; `:938`/`:940`
zero legs → `verifyIncomplete`/`verifyAbandoned`; `:696` → `verifyAbandoned`;
`:670`/`:680` → `verifyRefused`; `:794` fresh-slots refusal → `verifyRefused`;
`:979` partial verify → `verifyIncomplete` — all benign, all confirmed. §4.7b's
table (unchanged by this fold) is accurate.

---

## SIGNATURE CONSISTENCY

**Clean.** Grepped the whole file for every variant:

- `status []string` → **zero** remaining hits; every occurrence now reads
  `status string` (lines 272, 480, 484).
- `append([]string{status}, ...)` — the one composition site (`:486`) uses it
  correctly; the two other `append(lines, extra...)` / `append(status, lines...)`
  fragments in the file (`:267`, `:505`) are explicitly-flagged **historical**
  text describing the *pre-cycle* shape ("NOT the shape to mirror" / "R2 C-1's
  own defect class"), not the target design — correctly left alone.
- `buildVerifyStatusLine(status …)` / `buildVerifyStatusLines(status …)` — **zero**
  hits. Nothing still hands the builder a `verifyStatus` enum instead of the
  record.
- `buildVerifyStatusLines` (plural) — survives only inside one explanatory
  clause narrating what the *old* signature was (`:847`, "then plural,
  `buildVerifyStatusLines`"); correctly historical, not a live claim.

**C-1's actual fix, verified structurally sound:** `passRecord{full bool, legs int}`
nested inside `verifyRecord{pass *passRecord, adverse bool}` genuinely carries
both bits R10 found missing from the old `fullPass bool`. `buildVerifyStatusLine(rec verifyRecord) string`
is cited consistently everywhere (`:492`, `:835`†, `:851`, `:884`, `:998`, `:1062`).

†Note (not a defect, recorded for completeness): `multisigVerifyFlow(..., rec *verifyRecord)`
takes a *pointer* while `buildVerifyStatusLine(rec verifyRecord)` takes a *value*
— internally consistent Go (`var rec verifyRecord; flow(..., &rec); buildVerifyStatusLine(rec)`)
but the plan never shows the declaration site that makes this work. That gap is
R10's own **M-1**, already filed Minor and explicitly not re-rated by R10; this
pass did not find anything beyond what R10 already recorded.

---

## TEST-TABLE COHERENCE

**Not clean.** The fold renamed tests at exactly one location (the §4.8 step-2
row and the "does not COMPILE" paragraph, both correctly updated: `T14`→`T26`,
`T9/T13a/T13b`→`T20/T21/T22`) but left **five more locations** still naming the
deleted IDs. Grep, current file:

| line | text | status |
| --- | --- | --- |
| 1003 | `§4.8` step 7: *"plus T9, T10, T11, T12, T13a, T13b"* / *"T9/T13a/T13b need a rendered document..."* | **stale** — only T11 still exists |
| 1113 | *"which let T13a's own named mutation pass"* | **stale** |
| 1147 | *"T9–T12 exist because the R1 round found the cycle's Critical had NO TEST."* | **stale** |
| 1153–1158 | *"T10 is the sharpest test in this plan..."* / *"If T10 does not fail against that implementation..."* | **stale**, whole paragraph |

None of `T14, T15, T15b, T16, T17, T18, T19` appear anywhere (clean on those).
Cross-checked against the pre-fold text at `dc42e27`: these five spots were
**already** stale before this fold started (the renumbering itself predates
`dc42e27` in the test table, which already read T20–T26 at that commit while
§4.8 step 2 still said "T14 only" — an earlier fold's own incomplete
propagation). This fold fixed the one instance R10's citation touched and left
the rest — the propagation is still incomplete, just less incomplete than
before. All five are **Important** (stale claims that will actively mislead an
implementer building from §4.8's build order, which is the section most likely
to be read literally and mechanically).

---

## PROPERTY CITATIONS

**Clean — R10's finding is closed.** `P1`, `P2`, `P5(a)`, `P5(b)`, `P5(c)` each
now have an explicit status in the new §4.7f-props table (lines 946–950):
theorem / theorem / retained-as-design / retired / theorem, respectively. `P6`
is defined at its own blockquote (`:962`). `P4` remains only as a labelled
corpse in §0.1 (`:79`) — R10 explicitly did not count this against the "five
citations, zero definitions" finding, and it isn't a live citation requiring a
table entry.

`T24`'s title *"P2 on the retry path"* (`:1086`) and §4.7d's prose (*"a sticky
adverse bit violates P1... a non-sticky one violates P2"*, `:912`) both use P1/P2
**before** their formal definition appears later in the document (§4.7f-props is
below both usages) — forward references, not undefined terms; not a defect.

---

## GATES — ACTUAL OUTPUT

All three commands run live from `/scratch/code/shibboleth/mnemonic-engrave`,
stdout/stderr captured separately.

```
./scripts/verify-returnsite-sweep.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
─── verdict return sites: 15 ; unrowed in the plan: 0
exit=0
```
Matches the plan's prose claim ("All fifteen `return verify*` sites are now
classified") exactly. No cached number is embedded in the plan text for this
gate, so nothing here can go stale.

```
./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
─── citations resolved: 96 / 96 ; dangling: 0
exit=0
```
**Does NOT match the plan's own cached comment.** §6.1 (`:1279`) reads:
`# R0 fold: citations resolved: 76 / 76 ; dangling: 0`. That number is **20
citations stale** — measured against `dc42e27` (pre-fold) it was already 97/97,
so this predates the current fold and nobody has updated it across at least
two folds. **Stale, Important.**

```
./scripts/plan-glyph-check.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md
─── operator strings scanned: 56 ; undrawable: 0
exit=0
```
**Also does not match.** §6.2 (`:1289`) reads:
`# R0 fold: operator strings scanned: 41 ; undrawable: 0`. Actual is 56 (was 54
at `dc42e27`, pre-fold). **Stale, Important**, same root cause as above: a
one-time snapshot labelled "R0 fold" was never refreshed as the document grew
over nine subsequent rounds.

**Both gates otherwise pass with 0 dangling / 0 undrawable** — the *content* the
gates check is fine; only the plan's own cached summary of a prior run is wrong.

---

## STRUCTURAL — a table row this fold broke

§4.7d's membership-test table, row "VERIFIED vs on-repeat" (`:906`), lost its
4th column. Every sibling row in the same table has 5 `|` characters (4 cells);
this row has 4 `|` (3 cells) — confirmed by an `awk` pipe-count sweep across
every table in the document, which found exactly this one anomaly and no
others. Before the fold (`dc42e27`) the row was well-formed with 5 pipes:

```
| VERIFIED vs on-repeat | same two bits | action identical — but **P2 forbids the merge** | **kept, and free** |
```

After the fold, the verdict cell's closing `|` was dropped when "P2 forbids the
merge" was replaced with "kept because it is FREE...":

```
| VERIFIED vs on-repeat | same two bits | action identical — **kept because it is FREE**, being a cell of the same 2x2, not because a property demands it |
```

The content itself is not wrong (it correctly answers R10's finding that this
row's retention rested on an undefined property), but the row will render
mis-aligned against its header (`distinction | prong 1 | prong 2 | verdict`) in
any Markdown renderer. **Important**, mechanical, one-line fix (add the missing
`|` to split "kept because it is FREE..." into its own verdict cell).

---

## WHAT I DID NOT DO

- Did not re-litigate R10's C-1/I-1 findings or re-derive whether the seam
  design is sound — confirmed the fold's own claims about it resolve, per the
  brief.
- Did not audit for new defects outside what the fold's claims touch.
- Did not evaluate whether T20's compact table-row description versus its
  surrounding prose (which asserts it drives all three document flows) is a
  coverage-adequacy question — that is a design-judgment call about test
  design, not a factual/stale contradiction, and R10 already covers this
  ground; out of scope for this pass.
- Did not run `plan-build-gate.sh` — the brief named exactly three gate
  commands to run; this document (a design plan, not code) is not in that
  script's remit and the brief did not ask for it.
- Modified no file.
