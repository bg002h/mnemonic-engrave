You are the INDEPENDENT tests/mutation reviewer (sonnet tier) for round 0 of the R0 gate on `design/IMPLEMENTATION_PLAN_hashlock_H0_reader_guards.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`, plan at `b0af794`). The fork is `/scratch/code/shibboleth/seedhammer` at main `839fa5aa`. Spec: `/scratch/code/shibboleth/mnemonic-secret/design/SPEC_ms_hashlock.md` §1 and §9 only.

ONE QUESTION: can every test the plan adds actually FAIL on the defect it names, and does the plan's mutation table hold when YOU run it — plus which mutations of the guard survive every test?

Read-only on both repos; commit nothing; no sub-agents; read no `.jsonl`. Work in your OWN scratch copies (the controller's copies are not yours):
- fork: `rm -rf /scratch/code/shibboleth/.tmp/h0-fork-tests && mkdir -p /scratch/code/shibboleth/.tmp/h0-fork-tests && (cd /scratch/code/shibboleth/seedhammer && git ls-files -z | tar --null -T - -cf - | tar -xf - -C /scratch/code/shibboleth/.tmp/h0-fork-tests)`; Go is `/scratch/code/shibboleth/.toolchain/go/bin/go` (put it first on PATH).
- engrave: `git -C /scratch/code/shibboleth/mnemonic-engrave worktree add --detach /scratch/code/shibboleth/me-worktrees/h0-tests e06e29d`; use `PATH=$HOME/.cargo/bin:$PATH CARGO_TARGET_DIR=/scratch/code/shibboleth/mnemonic-engrave/target TMPDIR=/scratch/code/shibboleth/.tmp`; the package is `-p mnemonic-engrave`. Remove the worktree when done (`git worktree remove --force ...`).
Apply the plan's blocks yourself, exactly as written (Task 1 Steps 1, 3, 5; Task 2 Steps 1-7). The corpus row's exact text is in Task 1 Step 1; after applying it `sha256sum` must print `4ac542ea8e0e36d92127b744bce0a83072f787870756bf7b86b9c947bb1370a5` — if it does not, report that first (the plan's hash claim would be false).

## Already settled — do not re-derive
- The gate proved the blocks build and the named tests pass (see `git show b0af794`). Compile findings are out of scope.
- Secret-handling defects never gate.

## Verify
1. **RED steps.** Task 1 Step 2 and Task 2 Step 1 claim specific failure text before the fix; reproduce both and quote the actual lines.
2. **The plan's declared mutations** (Task 1 Step 6; Task 2 Steps 4, 6, 7): run each, quote the failing assertion, revert, re-run green.
3. **Your own mutations** — at least these, plus any you think of: `IsPreimage` returns `d[0] >= 0x03`; `IsPreimage` returns `d[0] != msPrefixEntr`; the seam row's `device_admits` flipped to `true` (which test catches it, on which side?); `permitted()` in `seal/record.go` made to admit `ClassUnknown` in the encrypted section; the gui refusal string changed to omit "hashlock preimage"; `seal.Classify` guard kept but `isStrictMs1` guard dropped (does anything OTHER than the seam row catch it?); the Rust pin test with `Err(_) => {}` changed to accept `MsTooLong`. For each: caught by which test, or SURVIVED.
4. **False-PASS hunting.** (a) `h.mustReach("hashlock preimage")` — read `sessionHarness.mustReach` in `gui/`; does it match the text anywhere in any frame, so that a DIFFERENT screen containing those words would pass? (b) `TestAdmitSectionRefusesAPreimagePlateAsUnknown` checks `strings.Contains(err.Error(), "unknown")` — could an unrelated refusal satisfy it? (c) the seam test's three-shape rule: with the new row the counts are claimed 2 both / 4 device-only / 3 neither — recompute from the file. (d) the Rust pin test's `assert_eq!(PREIMAGE_PLATE.len(), 75)` — bytes or chars, and does it matter?
5. **Corpus for the Go port.** Is one 0x03 row enough for the guard, or does a mutation you ran survive because there is only one? (e.g. a guard keyed on the literal id `hash` rather than the prefix byte — does any row distinguish "prefix 0x03" from "id hash"? If not, say what row would.)

## Severity
Critical: a test that cannot fail on the defect it names (false PASS). Important: a declared mutation that does not behave as the plan says; a guard mutation that survives every test; a wrong count/hash claim. Minor/Nit: wording.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H0-plan-R0-r0-tests.md` (create; must not exist): the RED-step quotes; a mutation table (mutation, side, caught-by / SURVIVED, quoted assertion); the false-PASS answers; closing counts. Return a two-line summary plus the path.
