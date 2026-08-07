# Plan A R0 round 6 — **GREEN. Plan A's R0 loop is CLOSED at six rounds.**

Reviewer: sonnet, verification tier. Dispatched 2026-08-07 against `a3f49cf`.

Two questions, deliberately narrow:

1. **Did round 5's Critical actually get fixed?** The CR test had passed a single
   md1 chunk where the card-set decode demands three, so it died on "chunk set
   incomplete" and could not see the mutant.
2. **Re-run the five mutations nobody had independently checked.** Round 4
   claimed `CONFIDENCE: execution` and was wrong about one measurement — it
   reported a lone chunk with a CR yielding the CR error, where round 5 proved it
   yields "chunk set incomplete". Mutations (a), (b), (c), (d) and (f) rested on
   that single report's word. If a report can be wrong while claiming execution,
   one report is not enough.

Verdict: **0 Critical / 0 Important / 0 Minor / 0 Nit — GATE: PASS.**

Both came back clean. The CR test passes 11/11 unmutated and, under the CR-trim
mutation, exactly one test fails and it is `refuses_a_record_carrying_a_cr`. All
five re-run mutations die by exactly their named test — no more, no less. The
doubt round 4's error introduced is now retired by independent measurement rather
than by assumption.

Note the reviewer confirmed the restore was real each time — `mv` then `touch`,
with a `Compiling` line observed before re-testing. That is the stale-binary trap
that fooled round 4 and the controller once each; it did not fool this round.

---

## 1. Commands run and results

| Command | Result |
|---|---|
| `TMPDIR=<scratchpad>/verify-r6 ./scripts/plan-build-gate.sh` | PASS — seal module extracted+compiles, 55 seal-lib tests pass, `tests/seal_cli.rs` compile-checks, clippy `--lib` and `--all-targets` clean (reproduces the "already settled" gate result) |
| Manually inserted Task 9's 3 `main.rs` fragments verbatim from the plan's fences (enum `Seal`/`Hash` variants; early returns in `run()`; `run_seal_cli`/`run_hash_cli`) | assembled full `me` binary |
| `cargo build -p mnemonic-engrave --all-targets` | clean, zero warnings |
| `cargo test -p mnemonic-engrave --lib` (unmutated baseline) | **116 passed / 0 failed** |
| `cargo test -p mnemonic-engrave --test seal_cli` (unmutated baseline) | **11 passed / 0 failed** |
| 6 mutation cycles (CR-trim + a,b,c,d,f), each: apply → grep-confirm applied → build/test → restore via `cp` from backup + `touch` → rebuild (confirmed "Compiling" line each time) → re-test green | see tables below |
| Final: `grep -rn "MUTATION ("` across `src/` | no hits — fully unmutated |
| Final full rebuild + `--lib` + `--test seal_cli` | 116/116 and 11/11 green |
| `git status --porcelain` in the real repo | empty |

## 2. Question 1 — the round-5 Critical

| Step | Result |
|---|---|
| **a. Unmutated** | `cargo test --test seal_cli`: **11/11 pass**, including `refuses_a_record_carrying_a_cr`. |
| **b. CR-trim mutation** (`plaintext.iter().map(|s| s.trim().to_string()).collect()`) | **10 passed / 1 failed.** Only failing test: `refuses_a_record_carrying_a_cr` — panics on `Unexpected success`, exit 0, UF2 written, hash `fcc7 217e d26e 4ad7 bff2 e52c 4467 187e` (matches round 4/5's reference exactly). No other test failed. |
| **c. Restored** | `mv`-then-`touch` on `main.rs`; rerun showed a `Compiling mnemonic-engrave...` line (proving the rebuild wasn't skipped), then **11/11 pass** again. |

The round-5 Critical is fixed: the test now dies for the right reason (only under the mutant) and is silent (passes) on correct code.

## 3. Question 2 — five mutations re-run independently

| Mutant | Applied cleanly? | Failing tests | Dies by named test? |
|---|---|---|---|
| (a) HRP-only grouping (`record.rs`: `let csid = csid.map(|_| 0);` before the grouping key) | Yes | `seal::tests::vector_g_multisig_public_section_spans_four_cards` (115/1) — panics `k-card: chunked-header malformed: received 6 chunks, header declares total_chunks = 2` | **YES**, exactly |
| (b) delete combined 1..24 cap (`mod.rs` `seal_deterministic`) | Yes | `seal::tests::refuses_more_than_24_records_across_both_sections` (115/1) | **YES**, exactly |
| (c) delete lowercase guard in `record_or_mnemonic` (`mod.rs`) | Yes | `seal::tests::refuses_an_uppercase_bip39_mnemonic` (115/1) | **YES**, exactly |
| (d) iterations guard moved below public-only early return (`mod.rs` `seal()`) | Yes | `seal::tests::refuses_out_of_range_iterations_on_the_public_only_path` (115/1) | **YES**, exactly |
| (f) delete CR scan from `encode_section` (`container.rs`) | Yes | `seal::container::tests::refuses_embedded_separators_and_bad_lengths` (115/1) — `encode_section(&[format!("{A}\r")]).is_err()` fails, i.e. trailing CR silently accepted after trim | **YES**, exactly |

Each mutant killed by exactly its named test, no more, no less; each restore verified by checksum match against the pre-mutation backup plus a green rebuild.

## 4. New defects

None. All measurements matched the plan's mutation table and round-5's claims exactly.

## 5. VERDICT

```
Critical: 0   Important: 0   Minor: 0   Nit: 0
GATE: PASS
```

## 6. CONFIDENCE

**Execution**, for every claim above. All builds, test runs, mutation applications, and restorations were performed directly against a freshly-assembled full crate (base seal module via `plan-build-gate.sh` + Task 9's three `main.rs` fragments inserted verbatim from the plan's fences), not inferred or read from prior reports. Output pasted above for each step.

## 7. Repo state

`git status --porcelain` in `/scratch/code/shibboleth/mnemonic-engrave` is empty; HEAD unchanged at `a3f49cf`. All work occurred under `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/b4bd97f9-770f-407e-9105-3c9dcd1dc62b/scratchpad/verify-r6/`. The user's tree is clean.

Both questions came back clean — **GATE: PASS**, closing Plan A's R0 loop.
