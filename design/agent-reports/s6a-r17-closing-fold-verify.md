# S6a R17 — CLOSING FOLD VERIFICATION

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Fold under review:** `6a2198f` (diff `c3e9705..6a2198f`)
**Prior report:** `design/agent-reports/s6a-r16-fold-verify.md` (RED, 0C/1I)
**Code:** `/scratch/code/shibboleth/seedhammer`, main = `b8a23bf3dcf45f0b996bedf8b17f7141f092d282` (confirmed via `git rev-parse HEAD`)

## VERDICT: GREEN — 0 Critical, 0 Important (+ 0 filed)

No findings. R16's I-1 is FIXED and fully propagated. The fold's two new
substantive claims — "exactly one state (`statusVerifiedOnRetry`) is
unreachable" and "`gui/singlesig_verify.go:89` is the benign, byte-identical
twin of `gui/multisig_verify.go:896/897`" — both check out against the fork.
The acceptance-row rewrite (NEITHER option + mandatory adverse/benign bit)
introduces no contradiction elsewhere in the document. This is the closing
round; the R0 gate for this plan is satisfied.

---

## PART 1 — R16 I-1: FIXED

**The specific claim R16 flagged is gone.** `grep -n "ten adverse\|only \*\*two\*\* of the four" design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
returns exactly one hit for "ten adverse", and it is in the *corrected*
sentence describing the old defect ("It called all ten returns 'adverse'...",
`:1104`), not a live re-assertion. Zero hits for "only **two** of the four".

**The corrected claim is true.** The new paragraph (`:1093-1099`) states
exactly one of the four states — `statusVerifiedOnRetry` — is unreachable
from inside the eleven exits, and the other three (including the zero cell)
are reachable. Verified directly against `singleSigVerifyFlow`
(`gui/singlesig_verify.go:65-149`, no loop, no goto — straight-line code):
- `statusVerifiedOnRetry` requires `fullPassRecorded && adverseRecorded` in
  one call. `fullPassRecorded` is written only at the line-149 fall-through,
  which is reached only by never hitting any of the ten `return`s first — so
  no single call can set both. Confirmed unreachable.
- `statusVerified` (fall-through only, `:149`) — reachable.
- `statusCheckDidNotPass` (needs ≥1 adverse-classified return) — reachable:
  `gui/singlesig_verify.go:146` (`err := verifySingleSig(...)`) is a genuine
  comparison mismatch, textually the exact case named in §4.7c's
  `statusCheckDidNotPass` line ("a comparison did not match"). Adverse beyond
  dispute.
- `statusNotFullyChecked` (zero cell, needs ≥1 benign-classified return) —
  reachable: `gui/singlesig_verify.go:89-90` is the byte-identical twin of
  the table-classified-benign `gui/multisig_verify.go:896/897` (below).

**Propagation check.** Grepped the whole document for `adverse` (14 hits) and
inspected each. One passage superficially resembles the fixed defect —
`:869-871`, untouched by this or the R16 fold: *"An implementer told to
'write the record at each return site' writes ten adverse records and never
writes the pass record at all... the document would say not fully checked on
a run that fully checked."* Read in context (`:858-872`) this is a *warning
against* a naive literal reading that conflates "return statement" with
"adverse", illustrating the *fall-through-gets-missed* trap, not an assertion
that all ten returns are in fact adverse — its own next clause calls the
outcome wrong. It predates the R15/R16/R17 cycle and is orthogonal to R16
I-1's classification defect (misclassifying benign returns as adverse). Not a
propagation instance. No other residual claim implying all ten single-sig
returns are adverse, or that the zero cell is reachable only by skipping the
flow, was found.

**FIXED, fully propagated.**

## PART 2 — NEW DEFECTS IN THE FOLD

**(a) "Exactly one unreachable" claim.** True — see Part 1 derivation above.
`gui/singlesig.go:131` (`if sel, ok := verifyChoice.Choose(ctx, th); ok && sel
== 0 {`) and `:132` (`singleSigVerifyFlow(ctx, th, full, template)`) confirmed
by `grep -n`, both matching the plan's citations exactly; `engraveSingleSigFlow`
(`gui/singlesig.go:38`) is itself straight-line with no loop around that call.
No second state is unreachable, and no additional state was wrongly declared
reachable. No defect.

**(b) The byte-identical/benign claim.** Confirmed by direct read:
```
gui/singlesig_verify.go:89   showError(ctx, th, "Verify Bundle", "Couldn't re-derive the bundle from the seed.")
gui/multisig_verify.go:896   showError(ctx, th, "Verify Bundle", "Couldn't re-derive the bundle from the seed.")
gui/multisig_verify.go:897   return verifyFailed
```
Byte-identical message strings, confirmed. §4.7b's table (`:783`, untouched,
R12-audited CLEAN) cites this exact site as `gui/multisig_verify.go:897`
("re-typed seed will not derive") in the **benign** column, and the fold's new
paragraph cites the same `:897` for the precedent — matching the table's own
convention (which cites `return` lines, not `showError` lines). The fold then
separately cites `:89` (the `showError` line, not `:90`, the `return`) for the
single-sig side of the "byte-identical...same message string" comparison.
**[MECHANICAL, Nit, not gating]** — this is a one-line-off citation-style
asymmetry (return-line convention on the multisig side, showError-line on the
single-sig side) inside a single sentence making two independently-true
sub-claims; both cited lines are individually correct for what they support,
and the referenced exit is unambiguous in context (the `:88-90` block). Not a
factual error, no G1/G2 impact. No defect gates on this.

**(c) Acceptance row (a): NEITHER option + mandatory adverse/benign bit.**
`grep -n "and/or"` over the folded plan: 0 hits — R16's specific objection
("'and/or' excluded the benign case") is fully removed, not just in row (a)
but everywhere. Checked the new row against surrounding material for
contradiction: T21 (`:1193`, "the ZERO CELL is the default... including a
return path added with no classification at all") *reinforces* the NEITHER
option rather than conflicting with it. T20, T23, T24, T25, T27 (`:1194-1200`)
make no claim about which/how many single-sig exits are adverse and are
unaffected. §4.7a's switch (`:764-768`, untouched) and §4.7b's table
(`:774-786`, untouched) are the authorities the new row text defers to, and
both already support "NEITHER is a valid answer for a benign exit." No
contradiction found. No defect.

## PART 3 — BOUNDED ATTACK

One additional pass confined to what this fold touched (the rewritten
reachability paragraph and row (a)), beyond (a)/(b)/(c) above:

- Checked whether the fold's phrase "which is precisely how a prohibition
  fails safe (§0.1)" accurately cites §0.1: §0.1 (`design/...md`, the "A
  prohibition fails safe by construction. An obligation fails open" line)
  matches verbatim. Accurate citation, not a paraphrase drift.
- Checked whether the new material crosses into NG1 (reporting the
  verification's epistemic status, out of scope by default per §0.1's guard):
  the fold's content is entirely about which internal boolean an *exit*
  writes (implementer/reviewer-facing design bookkeeping), not about new
  device-facing text describing what the device knows. No NG1 exposure.
- Re-checked the ten-return / eleven-exit count directly rather than trusting
  the plan's own quoted `awk` output: `awk 'NR>=65 && NR<=149 &&
  /return/{print NR}' gui/singlesig_verify.go` → `69 78 90 98 112 117 125 130
  138 146` (10 lines) plus the `:149` fall-through = 11, matching the plan
  exactly.

No second, independent defect found. **Nothing found beyond (a)/(b)/(c)'s
one Nit — no manufactured finding.**

---

**Closing assessment.** Six rounds (R12–R17) have now found zero design
defects; every finding since the four-state rewrite (R15's `verifyStatus`
leak, R16's "ten adverse returns") has been transcription, propagation, or
controller over-claim in explanatory text adjacent to the gate — never the
gate itself, and never re-discovered on re-check. This round found the same:
0 Critical, 0 Important. The R0 gate closes GREEN.
