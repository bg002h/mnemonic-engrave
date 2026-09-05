You are the INDEPENDENT fold-verification reviewer (sonnet tier, narrowly scoped) for round 2 of the R0 gate on `design/IMPLEMENTATION_PLAN_hashlock_H0_reader_guards.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`). Round 1's report is `design/agent-reports/hashlock-H0-plan-R0-r1-fold-verification.md` (committed `97dab8c`): 8/8 C+I fixed, ONE new Important — the round-0 fold's whole-crate gate claim ("615/616") did not reproduce because Task 1 Step 1's four seam-corpus rows are enumerated by `tests/record_corpus.rs` (S2's invariant-2 capture) and red three of its tests, and the plan never mentioned that file. The r2 fold is ONE commit, `64a6e0d`, over `fdfb040`: a new Task 1 Step 1b (extend `testdata/record_corpus_pre_s2.json` 33 → 37 with the argument invariant 2 demands), the File Structure row, Step 9's `git add` and its measurement method, and a Global Constraint on how whole-crate numbers are measured.

ONE QUESTION: does the r2 fold fix the r1 Important — so that following Task 1 exactly, in order, leaves the whole crate green except for the three box-local `history_purge` failures — without a new contradiction or a false claim?

Read-only on every repo; commit nothing; no sub-agents; read no `.jsonl`. Work in your OWN detached worktree with its OWN target dir: `git -C /scratch/code/shibboleth/mnemonic-engrave worktree add --detach /scratch/code/shibboleth/me-worktrees/h0-verify2 64a6e0d`, then every cargo command with `PATH=$HOME/.cargo/bin:$PATH CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h0-verify2-target TMPDIR=/scratch/code/shibboleth/.tmp`, package `-p mnemonic-engrave`. Remove the worktree and the target dir when done. Do not touch `me-worktrees/h0-gate2` (the controller's).

## Already settled — do not re-derive
- Round 0 and round 1 closed every C/I of the fidelity and tests lenses; the Go side is unchanged since `fdfb040` and is out of scope.
- The three `history_purge` failures (`the_harness_records_history_at_all`, `the_emitted_zsh_recipe_actually_purges_the_entry`, `editing_the_file_alone_is_the_trap_the_message_warns_about`) fail identically on untouched `master` (no `/usr/bin/zsh` on this box); you may confirm with one run on `master`'s checkout but do not investigate them.

## Verify
1. Apply Task 1 EXACTLY as written, in order (Steps 1, 1b, 3, 5, 7), to your worktree. After Step 1b: `cargo nextest run --locked -p mnemonic-engrave --test record_corpus` must be 6/6 with 37 records — quote it. Check the four entries sit directly after `codex32_seam/bip93-bad-checksum`, in the seam file's row order, and that `class`/`consult` are what `sysw::classify` and `descriptor::consult` actually answer (the tests prove it; say so).
2. Whole crate: `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` — quote the Summary line and every FAIL name. The claim under test: `616 tests run: 613 passed, 3 failed, 2 skipped`, the 3 all `history_purge`.
3. The argument in Step 1b ("added, not moved; all four Unknown on the host at 0.7; the descriptor gate refuses each as a record"): is each clause true? `host_admits` is `false` for all four rows in the seam corpus — check; does invariant 2's own text in `record_corpus.rs`'s doc comment allow this kind of change with this argument, or does it forbid growth?
4. New contradictions: the STATUS line, the new Global Constraint, the File Structure row, Step 1b, Step 9's Expected and `git add`, the self-review's r1 paragraph — read as a hostile implementer.

## Severity
The r1 Important not fixed, or a whole-crate number that does not reproduce = Important. A new contradiction = Important. Wording = Minor/Nit. A clean round closes R0 for this plan.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H0-plan-R0-r2-fold-verification.md` (create; must not exist): the executed checks with output, the four clauses of item 3 with a verdict each, closing counts and a plain GREEN / NOT GREEN. Return a two-line summary plus the path.
