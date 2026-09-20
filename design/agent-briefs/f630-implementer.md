# Brief — F-630 implementer (ONE agent, TDD, executing a GREEN plan)

**The plan is authoritative and GREEN (0C/0I after six review rounds). Execute
it; do not redesign it.** If you believe a step is wrong, STOP and report —
do not improvise a different design. A plan this heavily reviewed has reasons
behind clauses that look arbitrary, and several are recorded in the plan itself.

**Plan:** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`
Read it in full before writing anything. The Decisions section explains WHY
each clause exists; the Tasks section is what you execute.

**Worktree (work ONLY here):** `/scratch/code/shibboleth/sh-worktrees/f630`,
branch `f630-xpub-header-sync`, based on fork main `95716e97`.
Primary (read-only, for the re-vendor): `/scratch/code/shibboleth/descriptor-mnemonic` at `b2c5d693`.
Go: `export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH`
Scope Go runs with `-run`; the whole gui suite is `scripts/gui-shard-test.sh ./gui/ 24` (~112s).
**Never run the same suite twice to collect counts and failures separately** —
capture once to a file and grep it.

## Execute T1 → T2 → T3, then T4's LOCAL gates only

**Do NOT push.** T4's push step is the controller's; stop after the local gates
and report. Commit at each task boundary so the history shows the sequence —
T1's commit is deliberately RED (that is its purpose) and its message must
carry the measured acceptance output.

Acceptance numbers, from the plan — these are measured facts, not targets to
tune toward. If you see a different number, that is a finding to REPORT, not a
gate to adjust:

- T1 against the stale corpus: **44 of 46 fail, 2 pass** (`keyed_tr_keyonly`,
  `keyed_wpkh`).
- After T2, the same run against the stale corpus: **5 pass / 41 fail**.
- After T3: **43 of 46 correct-header, plus 3 pinned-legacy.** Not 46 of 46 —
  that is unreachable by construction and the plan explains why (D5g).
- T3's re-vendor moves exactly 41 records descriptor-only (82 chain entries),
  3 wholesale, 0 address lines and 0 id lines outside those 3, 0 files added
  or removed. The script should print `246 files, 50 vectors`.

## The traps this plan already paid for — do not re-discover them

- **T3 and the re-vendor are ONE action** (the script copies then hashes its
  destination). Copy the vectors directory aside BEFORE running it, or the diff
  expectation cannot be checked.
- **T3 has THREE edits**, the third being `isComposeVectorFile`.
- Count `composeVectorNames` with `ast`, never `grep -o '"[^"]*"'` — a comment
  inside the literal quotes a phrase, so grep says 37 when it holds 36.
- The membership assertion is `t.Errorf`, never `t.Fatalf`.
- D1′ compares **strictly** — no hardening normalisation.
- An ambiguous slot lookup must fail loudly, not pick one.
- `go vet ./...` **exits 1 at baseline** with ten `ArtifactDir` diagnostics;
  compare the diagnostic SET, never the exit code.
- `gofmt -l .` baseline is **five files**, not empty.

## TDD and mutation discipline

Tests before implementation. For every new assertion, **prove it can fail**:
break the thing it checks, watch it go red, restore. A gate that has never
failed is a hypothesis. Record those mutation results in the commit message —
this cycle exists because a gate reported green while checking nothing.

## Report

Write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f630-implementation.md`
**as your final action**: what you did per task, the measured numbers against
the acceptance list above, the mutation results, every gate's output, and
anything you found that the plan got wrong. Return only a short summary plus
that path. Leave the worktree with all work COMMITTED and `git status` clean.
