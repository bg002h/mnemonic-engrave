You are the INDEPENDENT tests/mutation reviewer (sonnet tier) for round 0 of the R0 gate on `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`, plan at `02abee6`). Spec: `design/SPEC_hashlock_H2_device.md` (GREEN `55ee7a4`) §2, §3, §4.6, §7. Fork: `/scratch/code/shibboleth/seedhammer` main `c4a64fc` (read-only). Corpus: ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json` at `cd0a60f`.

ONE QUESTION: can every test the plan adds actually FAIL on the defect it names, does the plan's RED/GREEN/mutation story hold when YOU wire it from the plan's text, and which mutations of the guards survive every test?

Read-only on the fork and the repo; commit nothing; no sub-agents; read no `.jsonl`. Work in your OWN scratch copy of the fork: `rm -rf /scratch/code/shibboleth/.tmp/h2-tests && mkdir -p /scratch/code/shibboleth/.tmp/h2-tests && (cd /scratch/code/shibboleth/seedhammer && git ls-files -z | tar --null -T - -cf - | tar -xf - -C /scratch/code/shibboleth/.tmp/h2-tests)`; Go is `/scratch/code/shibboleth/.toolchain/go/bin/go` first on PATH. Never touch `/scratch/code/shibboleth/.tmp/h2-gate` (the controller's gate scratch; you MAY read its files to copy the gate's fixes rather than re-deriving them — the gate report `design/agent-reports/hashlock-H2-plan-build-gate.md` lists them). Apply Tasks 1-4 from the plan's text (the gate's fixes are already folded at 02abee6; `scripts/h2-plan-blocks-vs-tree.sh` proves the blocks equal the gated tree); revert every mutation; remove nothing you did not create.

## Already settled — do not re-derive
- The gate proved the wired tree compiles and the plan's tests pass (see its report). Compile findings are out of scope.
- Secret-handling never gates.

## Verify
1. **RED steps, verbatim**: Task 1 Step 3, Task 2 Step 2, Task 3 Step 1, Task 4 Step 2 — quote what actually happens at each.
2. **Declared mutations**: Task 1 Step 5 (six), Task 2 Step 4 (one), Task 4 Step 4 (five) — run each, quote the failing assertion(s), revert, re-green. The plan credits each to specific tests/rows; a credit that is wrong is Important.
3. **Your own mutations** — at least: (i) `IsMS1Shaped` without `TrimSpace`; (ii) `IsMS1Shaped` with `minMS1Len = 47`; (iii) `ValidatePhrase` checking the cap BEFORE the shape test; (iv) `DeriveHardened` ignoring `progress`'s false (Back during derivation no longer abandons); (v) the relation line computed against `st.list` digests instead of the payload's; (vi) the confirm modal's HOLD assigning BEFORE the screen returns (i.e. assign then confirm); (vii) `hashByPhrase` never set; (viii) `composerHashEdit`'s `default:` arm clearing the lock again. For each: caught by which test, or SURVIVED.
4. **False-PASS hunting.** (a) `TestHashlockPhraseRouteDoesNotNormalise`: could the typed bytes reach `ValidatePhrase` already normalised by the KEYBOARD (does `PassphraseKeyboard` ever change case or collapse spaces? read `passphrase_keyboard.go`) so the test passes while the flow normalises? (b) `TestHashlockBackContractKeepsThePath`: does `mustReach("28/100")` prove the phrase survived, or would an empty phrase screen also show a counter? (c) `TestModalsThisBlockTouchesAreDrawnInFull`'s new rows: does the table's renderer use the same modal as `composerConfirmScreen`? (d) the corpus tests compare against constants — confirm no test recomputes the expected value with the code under test.
5. **Corpus sufficiency.** Which spec §2 clauses have no corpus row (e.g. a phrase with a TAB inside, a 0x7F, a 100-character phrase that is also 64-hex-shaped? impossible; say so) and whether the plan's own tests add them.

## Severity
Critical: a test that cannot fail on the defect it names (false PASS). Important: a declared mutation whose effect differs from the plan's claim; a guard mutation that survives every test; a RED that does not reproduce. Minor/Nit: wording.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H2-plan-R0-r0-tests.md` (create; must not exist): the RED quotes; a mutation table (mutation, caught-by / SURVIVED, quoted assertion); the false-PASS answers; the corpus-sufficiency list; closing counts. Return a two-line summary plus the path.
