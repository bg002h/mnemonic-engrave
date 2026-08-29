# CONTINUITY — the overnight run, 2026-08-28

**Operator mandate (AskUserQuestion, answered "Plan + S1 + S3"):** work
autonomously overnight: draft `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md`,
take it through its R0 loop to GREEN, then implement **S1** (the input
cascade + admission predicate + the 68-row `descriptor_seam_vectors.json`)
and **S3** (`--as md1` end to end) with TDD, single implementer, worktree,
and the mandatory post-implementation adversarial review. Target: a
demonstrable `me sysw pack --as md1` at the desk by morning, or a parked
state with a written note at whichever gate stopped it.

**Defaults in force (stated to the operator, unobjected):**
- F-413: build spec-as-written (`ypub` refused, executable remedy); fable
  consult substitutes for the operator only if it gates.
- Gates: plan R0 loop, opus default, fable on the 2026-08-28 triggers;
  post-implementation adversarial review is mandatory and non-deferrable.
- Stop-rules (operator, mid-turn 2026-08-28, superseding the ~5 draft):
  **5 rounds is expected and considered good; at 15 opus reviews the
  reviewer tier SWITCHES TO FABLE; 25 rounds is a HARD STOP** — park with
  a written note. Funds-risk decisions a consult cannot legitimately
  settle → park.
- Hard boundaries: NO tags, releases, crate publishes, or on-device
  actions. Pushes via `scripts/push-via-staging.sh` / agents are in scope.

**Spec baseline:** `SPEC_descriptor_input.md` FINAL GREEN at `b949d18`
(r20 closure verdict in `R0-descriptor-input-spec-r20-closure.md`, whose
leaves-open list is the plan's input inventory).


**Added mid-run (operator directive): before sending any fold to review,
run the propagation check** — old forms grepped to zero, new rules checked
for unpropagated siblings, arithmetic recomputed at every site. First run
found six plan-side misses of the r2 fold (held for the r3 fold,
`scratchpad/f412/held-fixes.md`, declared self-found).

## Progress log (written as phases close)

- **Plan GREEN at round 5** (`c3fefe4`), within the "5 is expected and good"
  budget. Opus rounds used so far this overnight cycle: 6 of the 15 that
  would trigger the fable switch.
- **P0 closed** — 71-row vector file (sha256 `0393592f…`), both harnesses,
  zero findings; branch `impl/descriptor-s1s3`.
- **F-413 RESOLVED** (fable consult, `RULING_f413_slip132.md`): REFUSE
  STANDS; bare-ypub normalisation is a promotion-table reopen, not a byte
  swap. F-426 filed (device-side ypub case, with S2). Pushed at `1f634d5`.
- **P1 closed GREEN at review round 1** (0C/0I/6M/2N, `IMPL-P1-review.md`,
  persisted on the branch at `44e121a`). Cascade verified statement-by-
  statement against the fork; 37/37 gate rows hand-reproduced. The six
  Minors all carry into P2/P3 briefs (M1 = §6's five false `multi`
  sentences → authorized spec amendment in P2.4; F-1's direct-construction
  test → P2.2; annotation fix + M6 note → P3).
- **P2 implementer dispatched** (opus, same worktree) with all nine carry
  items enumerated. Next after its report: the MANDATORY post-implementation
  adversarial execution review over the whole S1+S3 diff, then P3.
