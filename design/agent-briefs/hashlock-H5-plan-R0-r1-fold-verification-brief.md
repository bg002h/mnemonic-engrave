You are the INDEPENDENT fold-verification reviewer (sonnet tier, narrowly scoped) for round 1 of the R0 gate on `design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`). Round 0 ran three lenses (`design/agent-reports/hashlock-H5-plan-R0-r0-{fidelity,tests,journey}.md`); the round-0 fold is `<FOLD_SHA>` over the plan at `0c2b13e` (and a companion spec fold to `design/SPEC_hashlock_H5_device_polish.md`, same commit or the one before it); the plan's `## R0 round 0 folded here` paragraph maps each Critical/Important to a change or a stated decline. The plan author's gated tree is `/scratch/code/shibboleth/.tmp/h5-gate`; the controller re-wired the fold's code changes into it before dispatching you.

ONE QUESTION: does the fold fix every Critical and Important the three reports raised (or decline it with a true reason), do the plan's code blocks still equal the gated tree (`scripts/h5-plan-blocks-vs-tree.sh` must PASS -- run it), do the changed tests still FAIL under their named mutations (run the ones the fold touched), and is every NEW number, citation and copy string true when YOU measure it?

Own copy for execution: `rm -rf /scratch/code/shibboleth/.tmp/h5-r1 && mkdir -p /scratch/code/shibboleth/.tmp/h5-r1 && cp -a /scratch/code/shibboleth/.tmp/h5-gate/. /scratch/code/shibboleth/.tmp/h5-r1/` (Go `/scratch/code/shibboleth/.toolchain/go/bin/go`; remove when done); never modify the gate tree; read-only on every repo; commit nothing; no sub-agents; never read any `.jsonl`.

## Verify (execute; quote)
1. Per C/I finding: the diff `git diff 0c2b13e..<FOLD_SHA> -- design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md design/SPEC_hashlock_H5_device_polish.md`, and whether it fixes the defect as stated.
2. The checker on the folded plan against the gate tree; the whole gui shard set once at the gate tree; the fit numbers the fold quotes (write-down repair 343/107; plain form 133/397; any others) re-measured with `assertModalBodyFits`.
3. The mutations named for the changed/added tests (the write-down line; the plain form's plural; the reconcile screen; Step 12's revert discipline) -- apply, quote, revert.
4. Superseded phrasing grepped in the plan and the spec (a fold fails by incomplete propagation); the spec and plan agree on every copy string byte for byte.

## Severity
A C/I not fixed and not truly declined, a checker FAIL, a test that stays green under its mutation, a new false number or contradiction = Important. Wording = Minor/Nit. A clean round closes R0 for the plan.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H5-plan-R0-r1-fold-verification.md` (create; must not exist): the table (finding, change, verdict), executed checks with outputs, closing counts, a plain GREEN / NOT GREEN. Return a two-line summary plus the path.
