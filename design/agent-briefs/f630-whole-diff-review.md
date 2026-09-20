# Brief — F-630 whole-diff adversarial execution review (mandatory, non-deferrable)

**The diff:** fork worktree `/scratch/code/shibboleth/sh-worktrees/f630`,
branch `f630-xpub-header-sync`, `git diff 95716e97..a126070` — three commits
(`da35965` T1 deliberately RED, `ea3c825` T2, `a126070` T3). 16 non-testdata
files, 973 insertions; 61 testdata files.

**The question:** *what did the implementation get wrong that the plan and TDD
could not catch?* The plan passed six review rounds; this round reviews the
CODE, not the design. Do not re-litigate the plan.

## Already machine-checked by the controller — do not re-derive

Verified in this worktree, independently of the implementer's report:

- `TestKeyedConformanceDescriptorsAgreeWithTheirTemplates`: **46 of 46 pass
  (43 correct-header, 3 pinned-legacy), 0 fail**.
- `TestKeyedConformanceAgreesWithRust` (the pre-existing cross-language gate):
  PASS — so D5b's card fix worked.
- `go vet ./...`: 10 lines, all `ArtifactDir`, 0 other. `gofmt -l .`: exactly
  the five-file baseline. `./md/ ./sysw/ ./mk/`: ok.
  Full gui: **1374/1374 across 24 shards**.
- The re-vendored tier is byte-identical to the primary's 246-file selection
  (0 differing). Provenance: `b2c5d693`, 50 vectors, 246 files.
- `md/testdata/forkbuilt/` holds exactly the three F-529 records + their three
  cards, plus the three pre-existing `dup_seat_*` cards.

So: **the suite is green and the numbers are real.** Spend your budget on what
green cannot show.

## Where to look hardest

1. **Can each new assertion actually fail?** The implementer reports 23
   mutations. Re-derive a few of the load-bearing ones yourself rather than
   trusting the count — especially D2b (the cross-language clause), the D5g
   membership assertion, and the pinned-legacy arm. **A clause that cannot
   fail is this cycle's entire subject matter.**
2. **The pinned-legacy arm is the dangerous one.** It ACCEPTS the 0.44.0 defect
   shape by design. Can a record that is defective in some OTHER way reach it
   and pass? Can a vendored record be routed into it?
3. **D1′'s implementation.** Slot matching by 65 bytes, strict comparison, the
   ambiguous-slot refusal, `<0;1>` resolution. Try to make it produce a false
   PASS or a false RED.
4. **The routed loaders.** The implementer found **ten** record reads where the
   plan said six, and routed all ten. Check the four extra ones are genuinely
   the same shape and were routed correctly — and that no read was missed.
5. **`bip380`'s export.** `validChecksum` → exported, plus its call site. Does
   anything else change behaviour? The firmware is reported byte-identical with
   `bip380/` reverted; confirm the export reaches nothing on the device.
6. **The vendor script's widened pattern.** Does it do exactly what T3 says,
   including the `compose_refusal_` exclusion, and is `isComposeVectorFile`'s
   directory scan now covering the newly-pinned tier?

## Rules of evidence

Execute in the worktree; you may run tests and create scratch files, but
**commit nothing and leave `git status` clean**. Reproduce each finding: the
state, and the wrong outcome. Prescribing a remedy is secondary to proving the
defect.

Severity per project standard. Secret-handling defects are never Critical or
Important here (operator ruling 2026-08-27) — log them as follow-ups.

## Output

**FINAL ACTION: write the report to
`design/agent-reports/f630-whole-diff-review.md`** (in
`/scratch/code/shibboleth/mnemonic-engrave`) and return ONLY a one-paragraph
summary, that path, and C/I/M/N counts.
