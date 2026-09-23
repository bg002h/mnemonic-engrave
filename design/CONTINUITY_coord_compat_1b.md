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

**Controller rulings before plan 1b (2026-09-23), from the recon:**
- **R-1, the Nunchuk leaf order.** The SkeletonKey stays coordinator-independent.
  Nunchuk's kind-1 cell is a source-derived, KEY-DEPENDENT rule: accept iff
  the card's leaf pubkeys are already in sorted, unique order (libnunchuk
  `descriptor.cpp:640-648`, the PR-1746 re-render round-trip, measured
  at a7cfb49). A template has no keys, so it is `Unproven { KeysAbsent }`.
  Cost if wrong: evidence keyed by skeleton alone would need a key-order field
  added later.
- **R-2, evidence transport.** Evidence recorded in mnemonic-engrave is
  VENDORED into descriptor-mnemonic by a script with a freshness check, the
  same pattern as `vendor-liana-evidence.sh`. The md-codec xtask builds the
  table from the vendored copy. Cost if wrong: a second vendoring path to
  maintain.

**PR OPEN (2026-09-23): bg002h/descriptor-mnemonic#31** (branch `cc-1b-verdicts`
at `9196b13c`, md-codec 0.48.0 / md-cli 0.20.0). The engrave records are
merged (`bc846b37`, which contains the vendored `engrave_commit` 7ca9d2f8).
Waiting on the operator's `/code-review ultra 31`. After it: fold the
findings on the branch, then merge the PR with a merge commit (not a squash),
tag `descriptor-mnemonic-md-cli-v0.20.0`, bump demo/sh2's install block, and
do F-669 before any local `md` 0.20.0 install.
