You are the INDEPENDENT fold-verification reviewer (sonnet tier, targeted) for round 1 of the R0 gate on `design/IMPLEMENTATION_PLAN_hashlock_H1b_me_bump.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`). Round 0 = two lenses, persisted verbatim: fidelity (opus) at `9458bd3` — `design/agent-reports/hashlock-H1b-plan-R0-r0-fidelity.md`, 0C/2I/6M/2N — and tests (sonnet) at `b923e41` — `design/agent-reports/hashlock-H1b-plan-R0-r0-tests.md`, 0C/4I/2M. The fold is ONE commit, `b7ced42`, over the gate-green draft `e672194`; its message maps every finding to a change and quotes the re-run gate (controller hand-wire in `me-worktrees/h1b-gate`, own target dir — do not touch that worktree).

ONE QUESTION: did the fold address every Important from both reports — FIXED / PARTIAL / NOT FIXED / DECLINED-with-reason, one line each — with the plan's code blocks now matching what actually runs, and without a contradiction or a false claim of its own?

Read-only on the repo; commit nothing; no sub-agents; read no `.jsonl`. You MAY run the plan's code in your OWN worktree with its OWN target dir: `git -C /scratch/code/shibboleth/mnemonic-engrave worktree add --detach /scratch/code/shibboleth/me-worktrees/h1b-verify b7ced42`; `PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h1b-verify-target`; package `-p mnemonic-engrave`. Apply the plan's steps FROM ITS TEXT (Task 1 Steps 1-3, Task 2 Steps 1-4, Task 3 Steps 1-2); `touch` files you restore from backups; remove the worktree and target dir when done.

## Already settled — do not re-derive
- The two reports are the specification for this round; read them first, then `git show b7ced42`.
- The gate proved the fold's blocks build and pass and that the named mutations fail (the message quotes it). Report only what does NOT reproduce.
- ms-codec 0.8.0 is on crates.io; `me`'s pin moves to `"0.8"`; the three `history_purge` failures are box-local.

## Verify
1. **The finding table**: fidelity I-1, I-2; tests I-1, I-2, I-3, I-4 — the plan section that carries each fix, quoted, and a verdict; then the Minors/Nits (fidelity M-1..M-6, N-1, N-2; tests M-1, M-2), one line each.
2. **Applied from the text**: after Task 1 only, the RED set is what Task 1 Step 3 now says (six failures at the bare bump incl. `record_corpus`; the stated mechanism is ADMISSION — confirm by reading the two binary tests' actual failure lines). After Task 2 + Task 3: the targeted set green; `cargo fmt -p mnemonic-engrave -- --check` clean WITHOUT any edit beyond the plan's text (tests I-2); `me sysw pack` and `me seal` on the 50-character malformed plate `ms10hashsqw46h2at4w46h2at4w46h2at4w4ssrnvvaudn2k4d` and on the entr-id mismatch row both name what the plan says at exit 4.
3. **The predicate's negative space**: with the plan's `preimage_plate`, `sysw::classify` on the seam corpus's `bip93-plain-payload-0x03` (48 chars) and `bip93-plain-33-byte-payload-0x31` rows must NOT answer preimage; the `bip93-plain-33-byte-payload-0x03` row MUST; a K-of-N share whose data begins 0x03 (build one with `ms split` on any entr string? — if not constructible, say so) must NOT. Quote outputs.
4. **Mutations**: re-run fidelity I-2's guard (make the wildcard `Ok(_) => Ok(RecordKind::Ms)` again — which test, if any, catches it? The plan says none can and the wildcard is the guard; state whether that is acceptable or a gap), and the two Step 4 mutations as now described.
5. **New contradictions**: the fold rewrote Task 1 Step 3, Task 2 Steps 1-4, Task 3, Task 4, the self-review. Read as a hostile implementer: any sentence contradicting another or the code blocks?

## Severity
An Important marked FIXED but not fixed = Critical. A code block that does not do what the plan's prose says, a count that does not reproduce, a new contradiction = Important. Wording = Minor/Nit. A clean round closes R0 (lens-closure: fidelity, tests/mutation, fold-verification).

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H1b-plan-R0-r1-fold-verification.md` (create; must not exist): the finding table, the executed checks with output, closing counts and GREEN / NOT GREEN. Return a two-line summary plus the path.
