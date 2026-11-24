# Whole-diff bughunt sh2, slice 2 (Sonnet, tests-can-fail lens)

Fork `bughunt/sh2-fix` 319ed85..91fb004; me `bughunt/sh2` 393d6564..6c3331db.
Scratch: `/scratch/cache/bughunt/sh2/whole-diff-s2/` (plant*.py, plant*.log, unskip.log).
Method: revert one fix hunk by text substitution in a scratch clone at 91fb004, run the
row(s) by name with `go test -count=1 -run`, restore with `git checkout`.

## Result
No Critical, no Important.

### Plants (fix defect put back, RED row must fail)
Every plant below made its §6.1 row fail for the stated reason: 01, 01bw, 02, 03, 04,
06a/b/c, 07, 11, 13, 14, 21, 21b, 22, 30c, 30d (l, h, v, c and s variants), 31a, 31b,
32, 39 (all three seam func-literals), 49, 50b, 50c. Several first plants did not
compile (declared and not used); redone as `&& false` / `i > 1000` forms and then
failed on the right assertion, so those are real failures, not compile errors.

### Skips (§6.2)
Stripped all 38 `t.Skip("deferred-minor...` lines in the scratch clone and ran
`go test -count=1 -run TestBughuntSh2 ./...`: exactly 38 rows fail, and the set equals
the 38 skipped test names (diffed: SAME). Sh240 (FP row) is not among them and passes.
No skip hides a row that already passes, and no skipped row is outside the table.
Sh210c fails with a child-process OOM (17 GiB alloc), which is its stated RED reason.

### §7 expectation changes
Read each; none loosens an assertion that guards a fixed behaviour. The seam vector
pins (29 device-true, 45 device-false, 0 panic) are exact counts, so a regression moves them.

## Minor
- M-1 fix 50a zero-keys check and the Policy-header-missing check are each redundant
  with the threshold range check: planting either out leaves the suite green. Dead-code
  guard, fails closed either way.

## Nit
- N-1 21b is a guard row that passes on base (build report already says so); it cannot
  fail on the base defect, only on a later regression.

C / I / M / N: 0 / 0 / 1 / 1
