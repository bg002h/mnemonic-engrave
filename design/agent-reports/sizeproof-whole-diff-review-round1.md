# Whole-diff review — SIZEPROOF — round 1 (fold check) — **GREEN**

Single opus lane, dispatched 2026-08-05 against `c9cc4db`, scoped to two
questions: did the fold close round 0's two Importants, and did it introduce a
new defect. The two operator-deferred items were declared accepted in the brief
so they would not be re-filed. Persisted VERBATIM.

VERDICT: **GREEN — 0 Critical / 0 Important / 0 Minor / 0 Nit.**
Safe to merge to `main` and cut a release from.

---
## Findings

*(none)*

## Notes

GREEN — 0C/0I/0m/0n. `c9cc4db` closes both round-0 Importants and introduces no new defect. Safe to merge to `main` and cut a release from.

## Q1 — both Importants closed

**I1 (admission vs. the router).** `gui/freetext_flow.go:338` now reads `admitQR := useQR && !ftSizedBlocks(blocks)` and passes `admitQR` to `AdmissibleBlocks`, while `ftFitAt` still receives the raw `useQR` (correct — `ftFitAt` IS the router and ignores it for sized blocks). `gui/freetext_flow.go:516` is now `if !useQR || ftSizedBlocks(blocks) {`, so the QR remedy is not offered for a composition `ftSizedBlocks` accepts. Exactly the fix the review specified, at the caller; `backup.AdmissibleBlocks` is untouched (the fold is 2 files, and `backup/runningy_test.go` is not one of them).

**I2 (spec 7.17(a2)).** `TestSizeProofConfirmReportsTheQRFromTheFit` walks the item literally: `ftPastQR(h, true)` → load ladder → `ftBack` → `ftChoose(h, "qr", 1)` ("Add QR") → Title/Footer/Confirm, then asserts `QR: no` present, `QR: yes` absent, `ftWarnQR` absent, for BOTH triggers — and then taps through to the built plate and asserts `r.got.QR == nil`, so "QR: no" is checked against the steel and not against a second reading of the same field. A sibling subtest asserts `QR: yes` + `ftWarnQR` DO appear on an ordinary QR plate, so the absence assertions are non-vacuous.

## Q2 — no new defect

**Non-ladder verdicts cannot move.** `ftSizedBlocks` requires `len(blocks) != 0` and EVERY block's `SizeMM != 0` (`gui/freetext_flow.go:350-360`), and the only runs in the tree that set a non-zero `SizeMM` are the ten ladder entries at `gui/freetext_proof.go:477-488`, every run of both plans non-zero. For every other plan `ftSizedBlocks` is false, so `admitQR == useQR` identically. This is structural, not sampled. `TestAdmissibleBlocksVerdictDoesNotMove` (`backup/runningy_test.go:266`, five cases) is untouched by the fold and passes — note it calls `AdmissibleBlocks` directly, so it guards that function's contract rather than the caller; the caller-side guarantee is the structural argument above plus the unmoved goldens.

**The `plan.Blocks(text)` hoist in `ftRefuse` is inert.** For non-sized blocks the guard reduces to `!useQR`, character-identical to before. `ftPlan.Blocks` is pure, total, and always returns ≥1 block; `plan` is never nil (`plan := &ftPlanSH` at `gui/freetext_flow.go:947`, and `ftProofLoader` only ever writes a real plan), so the unconditional call cannot panic. The QR remedy still appears where it should — proven by the mutation below, which produced the full "Removing the QR frees about 374 characters… Keep the QR / Remove the QR" frame.

**The four new tests are real — I mutated the tree and watched each one die.** Done in my own detached worktree at `c9cc4db` (created, mutated, removed; the review worktree was never modified):
- Revert `admitQR := useQR && !ftSizedBlocks(blocks)` → `admitQR := useQR`: `TestSizeProofAdmissionIgnoresAStaleQRChoice` FAILS on BOTH triggers (front 20/24-vs-12/24 on the equality assertion — this is why the test asserts equality rather than the back's measured numbers; back 30/24 inadmissible), `TestSizeProofAdvancesWithTheQRReEnabled` FAILS on BACK, `TestSizeProofConfirmReportsTheQRFromTheFit` FAILS on BACK.
- Revert `if !useQR || ftSizedBlocks(blocks)` → `if !useQR`: `TestSizeProofRefusalDoesNotOfferAQRTheLadderCannotCarry` FAILS on both of its assertions.
- Make `ftConfirmSummary` read the flow's flag instead of `f.plate.QR` (threaded a mutant flag from `engraveTextFlow`): `TestSizeProofConfirmReportsTheQRFromTheFit` FAILS on BOTH triggers, rendering "QR: yes" and the privacy warning — while the pre-existing `TestSizeProofDropsTheQRTheOperatorChose` stays GREEN, confirming the commit's own account of why the old test never caught this.

**No surviving flag-for-plate read.** Enumerated every `useQR` site: `ftFitAt` (the router — correct to take the raw flag), the evaluate cache key (over-invalidation only), `ftProofOffer` (`useQR && !p.NeedsWholePlate()`, a load-time choice before any plate exists; both ladders have `TextQR: ""` so the drop applies), `ftRefuse` (now guarded), `*useQR = false` (unreachable for a sized composition). Everything that DESCRIBES a plate reads the plate: `ftConfirmSummary` `f.plate.QR != nil` (:751), `preview.go` `HasQR: fitted.QR != nil`, `fittedPreviewAt` through `ftFitAt`. Residual: the stale flag is now never cleared for a ladder, which is inert — it changes no screen and no plate, and is the already-accepted `sizeproof-qr-step-must-not-offer-what-it-drops` follow-up.

## Verification evidence

- `nix develop --command go test -count=1 ./...` — GREEN, every package, zero failures (ran twice; second run filtered for non-`ok` lines returned nothing).
- `git diff --stat 3c3a2ad..c9cc4db -- '*testdata*'` — EMPTY. Zero goldens moved across the whole branch.
- `gofmt -l gui/freetext_flow.go gui/freetext_sizeproof_test.go` — clean.
- `git status --short` empty on entry and on exit; HEAD still `c9cc4db`. One temporary worktree at `…/scratchpad/wt-mergegate`, removed with `git worktree remove --force` + `prune`; no probe file written into the repo, no tracked file modified.

Round 0's accepted items (`sizeproof-admission-count-at-its-own-rungs`, `sizeproof-qr-step-must-not-offer-what-it-drops`) and its CLEARED set were not re-litigated. Nothing new was found worth recording even as a Nit.
