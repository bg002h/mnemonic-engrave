# CONTINUITY — coordinator-compat cycle, plan 1b (started 2026-09-23)

**Operator rulings for this cycle (2026-09-23):**
- UC (ultracode) is OFF. It was declined for this cycle, so a bare "proceed"
  does not turn it on; only `UC on` does.
- Plan 1b ships as a **pull request** in descriptor-mnemonic. Its whole-branch
  review is **`/code-review ultra <PR#>`**, run by the operator (user-triggered
  and billed; the agent cannot launch it). It REPLACES the opus whole-diff
  review. The plan-level review loop is unchanged.

**State:** recon in flight. Reports land at
`design/agent-reports/coord-compat-1b-recon.md` (design staleness after F-449)
and `design/agent-reports/coord-compat-core-boundary.md` (Core boundary
release, plus the kind-1 Core claim re-measured on a release).

**Next:** fold the recon into the design, author plan 1b, run the R0 loop to
GREEN, implement on a branch, open the PR, and hand the operator the PR number
for ultrareview.
