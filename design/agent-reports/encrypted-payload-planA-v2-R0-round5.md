# Plan A R0 round 5 — fold check on `b946399`. **BLOCKED: 1 Critical.**

Reviewer: sonnet, verification tier, tight scope. Dispatched 2026-08-07.

Brief asked one question — did `b946399`'s fold fix each of round 4's seven
findings, and did it introduce a new defect — and named what was already settled
so none of it would be re-derived. It required the CR mutation to be *run*, not
read: "Do not accept 'the test looks right'; run it."

Verdict: **1 Critical / 0 Important / 0 Minor / 0 Nit — GATE: BLOCKED.**

**The Critical is the controller's own fold defect.** The test I added to close
round 4's Important, `refuses_a_record_carrying_a_cr`, passed `MD1` alone — a
single chunk of a three-chunk md1 card. `check_public` → `decode_public_set`
rejects an incomplete card set *before* `encode_section`'s CR scan runs, so the
test failed with `d-card: chunk set incomplete: got 1 chunks, expected 3`
identically whether the CLI trimmed or not. It could not fail for the right
reason and could not distinguish the mutant at all. The suite was red at
`b946399`, and the plan's own "expect 11 passed" was false.

**This is exactly the blind spot I had documented one commit earlier and left
open.** `plan-build-gate.sh` was extended in that same commit to compile
`tests/seal_cli.rs`, with a header stating its assertions are "unexercised — that
is a reviewer's job." The very next thing added to that file was a test that
compiled and could never pass. Naming a blind spot is not closing it.

Fixed in the follow-up commit by passing the complete card set (`MD1`, `MD1B`,
`{MD1C}\r`) and pinning the record index. Verified by execution afterwards:
11/11 pass unmutated; under the CR-trim mutation exactly one test fails, and it
is `refuses_a_record_carrying_a_cr`. Filed as **F-68** so the gate stops relying
on a reviewer for this class.

The other six findings (M-1, M-2, N-1, N-2, N-3, N-4) are all confirmed fixed by
measurement, and mutation (e) still dies by its named test.

---

## Verification report — commit b946399 fold of round 4

### 1. What I executed

| Command | Result |
|---|---|
| `./scripts/plan-build-gate.sh` | PASS — seal module compiles, 55 lib tests pass, `tests/seal_cli.rs` compile-checks (`--no-run`), `clippy --lib -D warnings` clean, `clippy --all-targets` clean |
| Assembled full crate at `/tmp/.../scratchpad/exec`: gate's seal-module output + Task 9's three `main.rs` fragments (enum variants, early returns, `run_seal_cli`/`run_hash_cli`) inserted **verbatim** from the plan's fences, plus the already-extracted `tests/seal_cli.rs` and `validate.rs` | assembled |
| `cargo build -p mnemonic-engrave --all-targets` | clean, zero warnings |
| `cargo test -p mnemonic-engrave` (unmutated, post-fold) | **168 passed / 1 FAILED** (lib 116, main 1, cli 30, cross_lang 1, golden 3, preview_cross_lang 1, prop 6, seal_cli **10 passed / 1 failed**) |
| `cargo test -p mnemonic-engrave --test seal_cli` | `refuses_a_record_carrying_a_cr` **FAILS on unmutated code** |
| Root-cause check: ran `me seal --plaintext "<MK1[0]>\r"` (a different chunk fragment) | same failure shape — confirms the bug is general, not specific to the MD1 constant |
| Read Task 4's own test `refuses_an_incomplete_card_set` (plan line ~798) | documents, pre-existing: "one md1 chunk of three" / "one mk1 chunk of two" → error, **before** any CR scan runs |
| Ran `me seal --plaintext MD1 --plaintext MD1B --plaintext "MD1C\r"` (all 3 chunks, CR on the last) on unmutated binary | **correctly refused**: `record 2 contains '\r', which is the record separator`, exit 4, no file — proves the underlying CR-refusal logic is sound |
| Applied CLI-trim mutation, reran the same 3-chunk command | **silently accepted**: exit 0, 512-byte UF2 written, hash `fcc7 217e d26e 4ad7 bff2 e52c 4467 187e` (matches round 4's "clean" reference hash) — proves the mutation is real and dangerous, and a *correctly constructed* test would have caught it |
| Applied mutation (e) (`hash refs reverted to .as_str()`), ran `--test seal_cli` | `printed_hash_matches_me_hash_regardless_of_surrounding_whitespace` **fails**: `left: "2aee 5d32 4877 f35b..."` vs `right: "fcc7 217e d26e 4ad7..."` — matches round 3/4's pinned values exactly |
| Restored `main.rs` from backup after each mutation, rebuilt, confirmed diff-clean | confirmed unmutated each time |

### 2. Findings table

| Finding | Fixed? | Measurement |
|---|---|---|
| **I-1** (CR trimmed at CLI, mutation-table row named a manual invocation) | **NO — the fix is broken** | The row now points at `refuses_a_record_carrying_a_cr` (syntactically satisfies "must name a test"), but that test **fails on unmutated code**: `me seal --plaintext "MD1\r"` (MD1 alone) fails with `d-card: chunk set incomplete: got 1 chunks, expected 3` — an unrelated pre-existing error from Task 4's `decode_public_set`, not the CR scan. The assertion `stderr.contains("contains '\\r', which is the record separator")` never matches, mutated or not. |
| M-1 (wipe comment) | YES | `git diff` confirms comment now reads "These records are NOT zeroized... claiming a wipe here would be a guarantee this code does not deliver" — no false claim remains |
| M-2 (doc comment misplacement) | YES | `refuses_an_uppercase_bip39_mnemonic` carries its own comment ("Kills the mutation that deletes `record_or_mnemonic`'s lowercase check..."); `refuses_out_of_range_iterations_on_the_public_only_path` carries its own ("The guard sits above `seal()`'s public-only early return...") — confirmed by reading plan lines 1583–1605 |
| N-1 (`spans_six_cards` identifier) | YES | `grep -rn spans_six_cards` across the repo returns **zero hits in the plan or any `.rs`** — the only 2 hits are inside `design/agent-reports/encrypted-payload-planA-v2-R0-round4.md`, which is a historical record persisted verbatim *before* the fold, correctly quoting the old name as part of documenting the defect. Not a live artifact. |
| N-2 (`--all-targets` clippy, 5 lints) | YES | `cargo clippy -p mnemonic-engrave --all-targets -- -D warnings` on the assembled crate: clean, zero output |
| N-3 (Global Constraint "18 Task 7 tests") | YES | Counted `#[test]` between `### Task 7` and `### Task 8`: **19**, matching the corrected text |
| N-4 (8-whitespace-token hash-line heuristic) | YES | Replaced with banner-anchor (`lines[i+1]` after `starts_with("public data hash (")`); test still passes unmutated and still dies under mutation (e), see above |

### 3. Mutation results

- **CR-at-CLI mutation** (`plaintext.iter().map(|s| s.trim().to_string()).collect()`): `refuses_a_record_carrying_a_cr` fails — but it *also* fails identically on unmutated code (same message, same exit code). The test cannot distinguish mutated from correct behavior at all; it is dead on arrival, not a "surviving mutant" in the usual silent-green sense — it's simply broken.
- **Mutation (e)** (hash refs reverted to `.as_str()`): exactly one test fails, `printed_hash_matches_me_hash_regardless_of_surrounding_whitespace`, with `left`/`right` hash values matching round 3/4's pinned reference values. This one is genuinely fixed and genuinely still kills its mutant.

### 4. Measured test counts

lib **116** | main (unittests) **1** | cli **30** | cross_lang **1** | golden **3** | preview_cross_lang **1** | prop **6** | seal_cli **11 total → 10 passed, 1 failed**

**Total: 169 tests run, 168 passed, 1 failed.** Plan's Task 9 Step 4 claim "expect 11 passed" for `seal_cli` is **false as measured** — it is 10 passed / 1 failed.

### 5. New defects

**CRITICAL — `refuses_a_record_carrying_a_cr` (tests/seal_cli.rs, plan lines ~2143–2153) does not test what it claims and fails on correct code.**

- **Location:** `design/IMPLEMENTATION_PLAN_encrypted_payload_hostA.md` lines 2143–2153 (the test) and line 2415 (the mutation-table row that now cites it).
- **Root cause:** the test supplies only `MD1` (a single chunk of a pre-existing 3-chunk multisig fragment, per Task 4's own `refuses_an_incomplete_card_set` test) with `\r` appended. `check_public` → `decode_public_set` rejects it for "chunk set incomplete" *before* `encode_section`'s untrimmed CR scan is ever reached — this is documented, pre-existing Task 4 behavior, not new.
- **Measured:** the assertion fails identically whether `run_seal_cli` is mutated or not (`d-card: chunk set incomplete: got 1 chunks, expected 3` in both cases). Supplying the complete 3-chunk set (`MD1`, `MD1B`, `MD1C\r`) instead correctly refuses on unmutated code (`record 2 contains '\r'...`) and correctly leaks through under the mutation (exit 0, valid UF2, correct hash) — proving the underlying feature works and only the test's record choice is broken.
- **Effect:** the exact defect this round exists to prevent — I-1 is not actually closed; no test currently guards the CR-trim regression; and the committed plan document's own stated CLI-test count (Task 9 Step 4: "expect 11 passed") is false, i.e. the suite is red at HEAD once actually run, which the build gate's documented blind spot let through unnoticed.
- **Fix:** replace the test body to supply the complete card set with the CR on the affected chunk, e.g. `me().args(["seal", "--plaintext", MD1, "--plaintext", MD1B, "--plaintext", &format!("{MD1C}\r"), "--out", ...])`, asserting `.failure()` and the same stderr substring. Also correct Task 9 Step 4's "expect 11 passed" only after that fix actually makes it 11/11.

No other new defects found. No regressions in the previously-closed items (C-1, C-2, I-1 vs I-2/I-3, vectors A–G, MSRV/clippy surface) — all still hold as measured.

### 6. VERDICT

```
Critical: 1   Important: 0   Minor: 0   Nit: 0
GATE: BLOCKED
```

### 7. CONFIDENCE

**Execution**, for every claim above except N-1's characterization of the two agent-report hits as "historical, not live" (inference from reading the surrounding text, though the grep itself is execution). Everything else — build-gate run, full crate assembly and build, the 169-test run, both mutations applied/restored/rebuilt with diffs confirmed clean between each, the MK1 cross-check, the 3-chunk complete-set cross-check (both unmutated-refuses and mutated-leaks), the Task 7 test count, and the clippy `--all-targets` run — is a direct measurement, output pasted above.

### 8. Repo state

`git status --porcelain` in `/scratch/code/shibboleth/mnemonic-engrave` is empty; HEAD is unchanged at `b946399`. All work was done under `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/b4bd97f9-770f-407e-9105-3c9dcd1dc62b/scratchpad/`. The user's tree is clean.
