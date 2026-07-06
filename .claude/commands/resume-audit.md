---
description: Resume the mnemonic-engrave funds-safety audit from persisted state
---

Resume the user-funds-safety audit of this repo. Authoritative state lives in
`design/AUDIT_FUNDS_SAFETY.md` — read it first and follow its "Resume protocol"
section exactly:

1. Check the state tracker; a step is done iff its artifact file exists on disk
   (finder reports in `design/agent-reports/funds-audit-D*-round0.md`, verification
   verdicts in `design/agent-reports/funds-audit-verify/`).
2. Re-dispatch opus finders for any missing dimension, opus refuters for any
   unverified critical/important/moderate finding (prompts reconstructable from the
   dimension descriptions in the state doc; finders are read-only w.r.t. the repo and
   MUST persist their full report file before returning).
3. When all dimensions are found+verified: synthesize (dedupe, rank), write
   `design/agent-reports/funds-audit-SYNTHESIS.md`, update the state doc summary.
4. Draft `design/SPEC_me_testing_hardening.md`, run the mandatory R0 architect gate
   to 0C/0I, then implement the test hardening with a single implementer (TDD),
   followed by the mandatory post-implementation adversarial execution review.

Update the state tracker in `design/AUDIT_FUNDS_SAFETY.md` as steps complete.
