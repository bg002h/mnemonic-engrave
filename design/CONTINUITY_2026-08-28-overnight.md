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
- **P2 built and gated** (branch `5b0007a`: 544/544, zero ignores, 36==36,
  §6 amendment isolated at `de35e30`), then the MANDATORY adversarial
  execution review came back **RED 1C/1I/3M/3N** (`32b94c4`,
  `IMPL-S1S3-adversarial-review.md`). C1: `--as descriptor` on a `multi`
  short-circuits conjuncts 2–8 → false "`--as md1` encodes multi" referral
  on an anyone-can-spend `multi(0,…)`. I1: the different-depths address
  line is false twice over. Round-trip, record surface (53-case baseline),
  both walk journeys, §6 amendment: clean. Fold dispatched back to the one
  P2 implementer with controller rulings: M3 = warning prints exactly on
  paths that pack; N1 = deliberate divergence, stands. Re-review (opus,
  round 8 of 15) follows the fold.
- **Fold-1 landed** (`83703b4`: C1 conjunct-1 split so the flag-dependent
  arm runs last; I1 fixed by NEW `derive.rs` — key-by-key derivation, the
  device's own address printed; M1/M2/M3/N2 folded; N1 declined per
  controller ruling; W14 erratum 2). 560/560, controller re-ran. Re-review
  dispatched (opus, round 8) — brief centres on derive.rs as new
  funds-path code, incl. constructing the unexercised 17–20-key push_int
  branch against an independent oracle.
- **Re-review RED 0C/1I/1M/3N** (`171ec42`): code clean — derive.rs held
  against an independent oracle + device on 91 wallets — but C1's reorder
  left three RECORD sites stating the old conjunct order (spec §7 clause
  8, §5.4 parenthetical, one vector `source` annotation, fork-pinned).
  Fold-1's sweep was scoped to crates/, which is why. Fold-2 dispatched to
  the same implementer: records amendment, vector-file byte change batched
  with the parked F-2 annotation (one sha bump, both repos in lockstep),
  M-A quote escaping, whole-repo sweep this time. N-c (N1's FOLLOWUPS
  entry) is controller-owned at P3. Opus rounds used: 8.
- **Fold-2 landed** (engrave `e56ae1b`, fork `a5e29b4`; vector sha bumped
  ONCE to `542cd492…`, byte-identical both repos, F-2 annotation batched
  in). The whole-repo sweep found a 4th I-A site unnamed by the review:
  the vector GENERATOR (rows.py) still carried both superseded strings —
  a re-run would have silently reverted the fix; repaired + 71/71 verified.
  P3 gains two items: (a) `cross_lang::rust_ndef_parses_in_seedhammer_go_reader`
  passes-by-skip with `go` off PATH (every suite count this cycle measured
  in that state; must RUN for real on master post-merge, submodule
  populated); (b) N1's declination gets its FOLLOWUPS entry. Fold-2
  verification dispatched to SONNET (mechanical fold-vs-findings). Opus
  rounds still 8.
- **P2 CLOSED GREEN** at IMPL-S1S3-fold2-verify (sonnet, 0C/0I/1M — the
  Minor is F-428). Rounds this cycle: 8 opus + 1 sonnet.
- **P3 executed:** IMPL-P0-review's late persist owned (`ce8f8c1`); merge
  `c9c3625`; the pass-by-skip cross_lang test RUN FOR REAL on master
  (ME_REQUIRE_GO=1, go 1.26.7 via nix, PASS 0.29s) with the full suite
  562/562; F-416 amendment (`9c88152`); FOLLOWUPS: F-416/F-419/F-421
  RESOLVED, F-420 marked eligible-open, F-425 push noted, F-427/F-428/
  F-429 filed; CHANGELOG Unreleased entry names §11 items 2–5 discharged,
  1/6 S2-parked (F-418).
