You are the INDEPENDENT spec-coverage + comprehension reviewer (sonnet tier) for round 0 of the R0 gate on `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`, plan at `02abee6`). Spec: `design/SPEC_hashlock_H2_device.md` (GREEN `55ee7a4`). Fork: `/scratch/code/shibboleth/seedhammer` main `c4a64fc` (read-only).

ONE QUESTION: does every normative sentence of the spec map to plan code that a fresh implementer can execute from the plan ALONE, and is every fact the plan states about the fork true at `c4a64fc`?

Checks (mechanical; quote evidence):
1. **Coverage matrix.** For each spec section and each MUST/NEVER/refuse/copy sentence in §2-§7 and §10: the plan task+step that implements it, or GAP. Do not trust the plan's Self-review paragraph -- build the matrix yourself from the spec text.
2. **Placeholder scan.** grep the plan for `TBD|TODO|later|similar to Task|appropriate|handle edge|as needed|…` inside code blocks and steps; any step that says what to do without showing the code.
3. **Name and type consistency** across tasks: every function/const/type used in Task N is defined in Task <= N or exists in the fork (grep it); signatures agree (`DeriveHardened`, `ValidatePhrase`, `IsMS1Shaped`, `hashlockPhraseRoute`, `composerCopy*`, `hashByPhrase`, the label constants); test names cited in Steps vs the tests defined.
4. **Citations.** Every `file:line` and every quoted identifier in §10 and in the plan's prose against the fork at `c4a64fc` (`git -C /scratch/code/shibboleth/seedhammer show c4a64fc:<file> | sed -n`); every claim of the form "the fork does X" grep-verified. The gate's report `design/agent-reports/hashlock-H2-plan-build-gate.md` lists fixes it needed -- confirm the plan's prose does not still describe the pre-fix shape.
5. **Interfaces blocks**: does each task's Consumes/Produces match what the neighbouring tasks actually use.
6. **Commit steps**: `git add` lists complete for the files each task edits (H1b's plan missed two files -- check every task).

Read-only; commit nothing; no sub-agents; read no `.jsonl`; no scratch copies (you may READ `/scratch/code/shibboleth/.tmp/h2-gate`; never modify it).

## Severity
Important: a spec MUST with no implementing step; a false fork fact; a name used before definition; an incomplete `git add`. Minor/Nit: wording, stale prose.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H2-plan-R0-r0-coverage.md` (create; must not exist): the coverage matrix; the citation table (cite / true-false / evidence); findings `### I-n / M-n / N-n -- title`; closing counts. Return a two-line summary plus the path.
