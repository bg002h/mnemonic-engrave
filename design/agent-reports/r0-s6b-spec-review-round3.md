# R0 architect review — `SPEC_s6b_pre_flash_cycle.md`, round 3 (closure check)

**Artifact:** `design/SPEC_s6b_pre_flash_cycle.md`, as folded in commit `f4b75d1`
(on top of `ee4aee0`, which persisted round 2's report).
**Source under review:** fork `bg002h/seedhammer`, `main` =
`b1479a1b38f6b045d27443764c858906e4e6e122` (re-verified: `git rev-parse HEAD`
matches, tree clean).
**Scope, per brief:** (1) did the fold close N1/N2/N3; (2) did the fold, scoped
to the exact text it changed, introduce a new defect. Not a fresh audit; R-A…R-M
and round-1's 11 CLOSED findings are not relitigated.

---

## 1. The two questions

| id | status | evidence |
| --- | --- | --- |
| **N1** (Critical — scrub-point claim wrong, no call site for R2's offer) | **CLOSED** | §2.3d's scrub-point claim now reads exactly as source has it: the defer is registered at `gui/singlesig.go:50-54` and fires at function **return** — verified by direct read of `gui/singlesig.go`. New §2.3e locates the offer between the verify offer (`:188-192`, `var rec` through the `if sel==0` block) and `restoreDocFlow` (`:221-223`) — both line ranges verified exact. Scope check at that insertion point, verified directly against source: `passphrase` is declared `:64`, live through `:223` (no shadowing) — in scope; `masterFP` is bound at `:107`, read at `:221` — in scope; `mnemonic` is scrubbed only by the `:50` defer at return, so it is live at the insertion point — in scope. GATE 2.3e added. |
| **N2** (Important — `bundlePlate` has no `kind`, ms1 plate can't be excepted) | **CLOSED** | Verified against source: `bundlePlate` (`gui/bundle_flow.go:346-353`) has exactly the 6 fields the fold's Round-2 report named, no `kind`. `bundleCard` (`gui/bundle.go:33-38`) does carry `kind bundleCardKind`. `bundlePlatePlan` (`gui/bundle_flow.go:358-373`) iterates `cards []bundleCard` and constructs each `bundlePlate` from `c` — `c.kind` is available at exactly the point the fold's prescribed one-field fix needs it. The new MECHANISM block (§1.2, inserted after the pre-existing I3 text, before §1.2a) states the fix precisely as verifiable: add `kind bundleCardKind` to `bundlePlate`, populate `kind: c.kind` in `bundlePlatePlan`, condition `bundleEngrave`'s title/footer pass-through on `p.kind != cardMS1`. This is spec prose prescribing an implementation fix, not code — appropriate for a spec document. |
| **N3** (Important — GATE 2.3d/3.2a missing from §6 table) | **CLOSED** | §6 table now has rows for `2.3d`, `2.3e`, and `3.2a`, all present and matching their inline definitions. Re-derived the count independently (not trusting the commit message): `grep -noE '\*\*GATE [0-9.a-z]+' design/SPEC_s6b_pre_flash_cycle.md \| sort -u` finds **19** unique inline `**GATE` definitions (`1.2a, 1.3, 2.2, 2.3, 2.3b, 2.3c, 2.3d, 2.3e, 2.4a, 2.4b, 2.5, 3.1, 3.2, 3.2a, 3.3, 4, 5.1, 5.1b, 5.3`); the §6 table has 22 rows, of which 3 (`1.1`, `1.2`, the `—` "me CLI untouched" row) pre-date the `**GATE N:**` labelling convention and were never claimed to have inline definitions. The remaining 19 table rows match the 19 inline definitions 1:1, both directions — the commit's "19 inline gates, 19 in table, 0 missing" claim is accurate. |

**All three: CLOSED.**

---

## 2. New finding — scoped strictly to text this fold added

### New-1 — Nit — §2.3e cites a function name that does not exist (`singleSigEngraveFlow`)

**The defect.** New §2.3e (line 348) states: *"The offer is inserted in
`singleSigEngraveFlow`, between the verify offer..."* — this function does not
exist anywhere in the fork. Checked: `grep -rniE "singlesigengraveflow"
--include="*.go" .` over the whole repo returns **zero** hits, any casing. The
real function, confirmed at `gui/singlesig.go:38`, is `engraveSingleSigFlow`
(word order transposed from the spec's citation).

**Reachable case.** Anyone reading §2.3e literally and grepping the codebase for
`singleSigEngraveFlow` at implementation time.

**Why it is only a Nit, not Important.** The two things that actually pin the
insertion point — the file (`gui/singlesig.go`) and the line ranges
(`:188-192`, `:221-223`) — are both correct and were independently verified
against source in §1 above. The wrong function name doesn't change where an
implementer would insert code; it's a naming slip in prose alongside an
otherwise-exact citation, not a false claim about mechanism or behaviour.
Nothing in KEEP-IT-SIMPLE or the severity rubric makes a non-load-bearing typo
a gate.

**Smallest fix.** One-word-order swap: `singleSigEngraveFlow` →
`engraveSingleSigFlow` at line 348.

---

## 3. Escalation

None. N1's underlying design question (where R2's offer is chained, whether
same-session or cross-invocation) was escalated in round 2 and appears settled
by this fold's choice (inline within `engraveSingleSigFlow`, before return) —
that choice is a specification decision, not a verification question, and is
outside this round's scope.

---

## Verdict

`GREEN 0C/0I`

(One Nit recorded above — non-gating, does not reopen the loop.)
