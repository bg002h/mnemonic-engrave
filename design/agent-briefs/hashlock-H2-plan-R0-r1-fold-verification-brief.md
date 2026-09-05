You are the INDEPENDENT fold-verification reviewer (sonnet tier, narrowly scoped) for round 1 of the R0 gate on `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`). Round 0 ran five lenses (`design/agent-reports/hashlock-H2-plan-R0-r0-{fidelity,tests,journey,adversarial,coverage}.md`) and a refute pass (`hashlock-H2-plan-R0-r0-refute.md`, the deduplicated CONFIRMED/PARTIAL list is the fold's contract). The round-0 fold is ONE commit, `<FOLD_SHA>`, over the plan at `02abee6`; its message and the plan's `## R0 round 0 folded here` paragraph map each confirmed finding to its change.

ONE QUESTION: does the fold fix every CONFIRMED and PARTIAL finding in the refute list, decline the rest with a stated reason, and introduce no new defect -- with every NEW claim, citation and code block true?

Read-only on every repo (`/scratch/code/shibboleth/mnemonic-engrave`; `/scratch/code/shibboleth/seedhammer` at main `c4a64fc`; the gated scratch tree `/scratch/code/shibboleth/.tmp/h2-gate`, which now carries the fold's code changes re-wired by the controller -- read, never modify); commit nothing; no sub-agents; no scratch copies; read no `.jsonl`.

## Already settled -- do not re-derive
- Round 0's REFUTED findings are closed; do not re-review them. Rulings L5, L7, L12, L15, L16, L22, L24 stand.
- `scripts/h2-plan-blocks-vs-tree.sh` at `<FOLD_SHA>` reports every block PASS (the controller ran it; output in the fold commit's message) -- the plan's code equals the gated tree; do not report block/tree mismatches unless you find one the script cannot see (a block without a path header).

## Verify (execute; quote)
1. For each CONFIRMED/PARTIAL finding in the refute list: the plan's change (`git diff 02abee6..<FOLD_SHA> -- design/IMPLEMENTATION_PLAN_hashlock_H2_device.md`), does it fix the defect as stated (not merely reword it), and is the mapping paragraph honest.
2. Every NEW file:line citation, constant, count, test name and "MUTATION:" claim the fold adds: grep it against the fork at `c4a64fc` and the gated tree (a fold adds citations that were never gated).
3. Declined findings: is the reason stated and true.
4. Contradictions the fold introduced within the plan (grep the superseded phrasing: a fold fails by incomplete propagation) and against the spec `design/SPEC_hashlock_H2_device.md` (GREEN `55ee7a4`).
5. The test/whole-package numbers the fold quotes vs the gate report and any controller re-run quoted in the fold commit.

## Severity
A confirmed finding not fixed, a new false claim, or a new contradiction = Important. Wording = Minor/Nit. A clean round closes R0 for this plan.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H2-plan-R0-r1-fold-verification.md` (create; must not exist): a table (finding, fold change, verdict FIXED / PARTIAL / NOT FIXED / DECLINED-OK), the executed checks with output, new-defect findings, closing counts and a plain GREEN / NOT GREEN. Return a two-line summary plus the path.
