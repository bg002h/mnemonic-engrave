You are the INDEPENDENT tests/mutation reviewer (sonnet tier) for round 0 of the R0 gate on `design/IMPLEMENTATION_PLAN_hashlock_H1b_me_bump.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`, plan at `e672194`). Spec: `/scratch/code/shibboleth/mnemonic-secret/design/SPEC_ms_hashlock.md` §1 rule 2, §3, §9.

ONE QUESTION: can every test the plan adds or relies on actually FAIL on the defect it names, does the plan's RED/GREEN/mutation story hold when YOU run it from the plan's text, and which mutations of the new arms survive every test?

Read-only on the repo; commit nothing; no sub-agents; read no `.jsonl`. Work in your OWN worktree with its OWN target dir: `git -C /scratch/code/shibboleth/mnemonic-engrave worktree add --detach /scratch/code/shibboleth/me-worktrees/h1b-tests e672194`; every cargo command with `PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h1b-tests-target`; package `-p mnemonic-engrave`. Never touch `me-worktrees/h1b-gate` (the controller's). Apply the plan's steps yourself, from its text, in order (Task 1 Steps 1-3, Task 2 Steps 1-4, Task 3 Steps 1-2); `touch` any file you restore from a backup; remove the worktree and target dir when done.

## Already settled — do not re-derive
- The controller's gate (`git show e672194`) proved the blocks build, the RED step is five failures at the bare bump, Tasks 2-3 turn them green, and three mutations fail their named tests; whole crate 619/616 (three box-local `history_purge`). Compile findings are out of scope; DO re-run what you doubt.
- Secret-handling never gates.

## Verify
1. **RED, verbatim.** After Task 1 only: quote the five failing tests and their panic lines. Does the plan's Step 3 text match what you see?
2. **Declared mutations** (Task 2 Step 4 a and b; Task 3 Step 2): run each, quote the failing assertion, revert, re-green.
3. **Your own mutations** — at least: (i) `Ok((_, Payload::Preimage(_))) => Ok(RecordKind::Ms)` (the arm present but answering the wrong kind); (ii) `preimage_plate` returning `true` for every `ms`-HRP string; (iii) the `TagKindMismatch` arm placed AFTER the profile arm (does anything catch the order?); (iv) the `TagKindMismatch` arm matching `Err(_)` (any codec error) — which test distinguishes a mismatch from a bad checksum?; (v) `validate_record` mapping `Payload::Preimage` to `RecordError::Invalid(..)` instead of `PreimagePlate` — do the binary tests still name the kind? For each: caught by which test, or SURVIVED.
4. **False-PASS hunting.** (a) The witness `the_codec_decodes_the_plate_and_me_still_refuses_it`: its first assertion checks the codec, the second `validate_record` — could the second pass for a reason other than the new arm (e.g. an earlier length gate)? (b) The seam test at 0.8: which rows changed verdict and does the file still hold all three shapes? (c) `a_preimage_plate_is_named_not_misdiagnosed`'s control string is an entr-32; is there any 0x03 input for which `unknown_reason` still says "outside the profile"?
5. **The bump's blast radius.** `cargo tree -p mnemonic-engrave -i ms-codec` and the lockfile diff: which crates entered; does any test outside the hashlock set change outcome between 0.7 and 0.8 (run the whole crate at the bare bump before Task 2: list every failure beyond the five and `history_purge`)?

## Severity
Critical: a test that cannot fail on the defect it names (false PASS). Important: a declared mutation that behaves differently from the plan's claim; a guard mutation that survives every test; a RED/GREEN claim that does not reproduce. Minor/Nit: wording.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H1b-plan-R0-r0-tests.md` (create; must not exist): the RED quotes; a mutation table (mutation, caught-by / SURVIVED, quoted assertion); the false-PASS answers; the blast-radius list; closing counts. Return a two-line summary plus the path.
