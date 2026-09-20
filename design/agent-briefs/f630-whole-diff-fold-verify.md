# Brief — verify the F-630 whole-diff fold (sonnet, mechanical, pre-push)

**One question: did the fold close the whole-diff review's Important and two
structural Minors without introducing a defect?** This is the last gate before
the branch is pushed.

- Report (input): `design/agent-reports/f630-whole-diff-review.md` (0C/1I/3M/4N)
- The fold: worktree `/scratch/code/shibboleth/sh-worktrees/f630`, commit
  `2cbca02`. `git show 2cbca02` is exactly the fold; `git diff 95716e97..HEAD`
  is the whole branch.

Go: `export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH`
Full gui: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`,
run FROM the worktree. You may run tests and create scratch files; **commit
nothing and leave `git status` clean.**

## What the fold claims, and what to check

1. **The Important** — `eachKeyedVector` now unions the vendored tier with
   `forkbuilt/`, in BOTH packages, and the gate asserts every pin reached the
   legacy arm. Re-run the review's own reproduction (delete
   `md/testdata/vectors/keyed_tr_multi_a.*`, restore after) and confirm
   coverage is preserved. Then try to find a *different* way to make a pin go
   unexamined while the gate stays green.
2. **The counts are now asserted, not logged.** Check the assertions are
   actually reachable and correct — in particular whether the "tier arithmetic"
   assertion is a tautology. If it can never fail, say so; a clause that cannot
   fire is this cycle's subject matter.
3. **`TestForkbuiltPinsAreAClosedHashedSet`** — an exhaustive, hashed set over
   `md/testdata/forkbuilt/`. Verify the nine hashes are right, that it catches
   a smuggled file AND an edited pin, and that it cannot pass vacuously.
4. **Did the fold break anything?** The union changes what several gui tests
   iterate over. Confirm the full surface is still green and that no test now
   silently iterates a name it cannot handle.

## Also: adjudicate the residue

The review left 1 Minor and 4 Nits unfixed. For each, say whether it is
genuinely non-blocking for a push, in one line. Do not fix them; judge them.
Specifically the report's claim that D1's parent-fingerprint clause has no
mutation row, and the Nit about a dominated clause that can never fire alone.

## Already established — do not re-derive

Before the fold: gate 46/46 (43+3); `go vet` 10 ArtifactDir only; `gofmt -l .`
the five-file baseline; `./md/ ./sysw/ ./mk/` ok; 1374/1374 gui. The re-vendor
is byte-identical to the primary's 246-file selection at `b2c5d693`.

## Out of scope

Re-litigating the plan or earlier rounds. Style. New features.

## Output

Severity per project standard; secret-handling is never C/I here.
**FINAL ACTION: write the report to
`design/agent-reports/f630-whole-diff-fold-verify.md`** and return ONLY a
one-paragraph summary, that path, the per-finding tally, your residue
adjudication, and C/I/M/N counts for any NEW defect.
