You are the INDEPENDENT fold-verification reviewer (sonnet tier, narrowly scoped) for round 1 of the R0 gate on `design/SPEC_hashlock_H5_device_polish.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`). Round 0 ran three lenses (`design/agent-reports/hashlock-H5-spec-R0-r0-{fidelity,journey,tests}.md`); the round-0 fold is ONE commit, `d36ede5`, over the spec at `f6dd437`; its message and the spec's `## R0 round 0 folded here` paragraph map each Critical/Important finding to a change or a stated decline.

ONE QUESTION: does the fold fix every Critical and Important the three reports raised (or decline it with a true reason), with every NEW number, citation and mechanism it adds true at fork main `b9a9a30`, and no new contradiction inside the spec or against the H2 spec (`design/SPEC_hashlock_H2_device.md`)?

Read-only on every repo; for anything you execute use your own detached fork worktree `git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/h5-r1 b9a9a30` (Go `/scratch/code/shibboleth/.toolchain/go/bin/go`; remove it when done); commit nothing; no sub-agents; never read any `.jsonl`.

## Verify (execute; quote)
1. For each C/I in the three reports: the fold's change (`git diff f6dd437..d36ede5 -- design/SPEC_hashlock_H5_device_polish.md`) fixes it as stated, or the decline reason is true.
2. Every NEW file:line, headroom/pixel number, width, line count and API name the fold adds: re-grep or re-measure (fit numbers via `assertModalBodyFits`; band width from `assets.NavBtnPrimary`; lead line count with `ctx.Styles.lead`).
3. Grep the superseded phrasing the fold replaced (a fold fails by incomplete propagation) and the H2 spec sections the fold touches or cites.
4. Read the folded spec as a hostile implementer: any sentence two implementers would read differently is Important.

## Severity
A C/I not fixed and not truly declined, a new false number or citation, a new contradiction = Important. Wording = Minor/Nit. A clean round closes R0 for this spec.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H5-spec-R0-r1-fold-verification.md` (create; must not exist): a table (finding, fold change, verdict FIXED / DECLINED-OK / NOT FIXED), the executed checks, new-defect findings, closing counts, a plain GREEN / NOT GREEN. Return a two-line summary plus the path.
