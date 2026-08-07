# Plan A R0 round 4 — by execution. **BLOCKED on 1 Important.**

Reviewer: opus, execute brief. Dispatched 2026-08-07 against the plan at `2ed2695`.

Brief stated what `scripts/plan-build-gate.sh` already proved (seal module compiles,
55 tests, clippy clean) so reviewer budget went to the surface the gate cannot
reach: `src/main.rs` and `src/validate.rs`, which the plan supplies as fragments
rather than whole files. Six named mutations were required to each die by their
named test. Mutations (c) and (e) were flagged in advance as the ones most likely
to be cosmetic.

Verdict: **0 Critical / 1 Important / 2 Minor / 4 Nit — GATE: BLOCKED.**

The Important is the seventh row of the plan's own mutation table — `CR trimmed at
the CLI` — whose killer is a **manual invocation**, not a test function. That is
precisely the shape round 3 graded C-2 as Critical; the round-3 fold re-pointed the
uppercase row and left this one. Measured: mutating `run_seal_cli` to trim leaves
the full 168-test suite green while §6.4's "CRLF is rejected, not tolerated"
silently degrades to normalisation.

Both round-3 Criticals and all three Importants are genuinely, executably closed,
and all six named mutations died by exactly their named test — including (c), the
round-3 survivor.

---

Tree is clean; the plan file is unchanged between `2ed2695` and current HEAD. Report below.

---

## What I executed

Built a scratch crate at `/tmp/.../scratchpad/exec` = the gate's seal-module output **plus** the three uncovered surfaces: `validate.rs`'s 74‑77 replacement, `main.rs`'s three fragments (extracted **verbatim** from the plan's Task 9 fences, not retyped), `tests/seal_cli.rs`, `tempfile`, and the md‑codec 0.42 bump.

| Command | Result |
|---|---|
| `cargo build -p mnemonic-engrave --all-targets` | clean, **zero warnings** |
| `cargo test -p mnemonic-engrave` | **168 passed / 0 failed** (lib 116, main 1, cli 30, cross_lang 1, golden 3, preview_cross_lang 1, prop 6, **seal_cli 10**) |
| `cargo clippy -p mnemonic-engrave -- -D warnings` (the plan's stated gate) | clean |
| `cargo +1.85.0 test -p mnemonic-engrave --locked` (MSRV) | all 168 pass |

`main.rs` assembles and compiles as written: `run_seal_cli(out: &std::path::Path)` deref-coerces from the `&PathBuf` binding, both early-return blocks bind against `&cli.command`, `EXIT_*`/`write_private` resolve, `run_hash_cli`'s `is_secret()` gate compiles and fires (`me hash --unsealed <ms1>` → exit 4). `validate.rs` is a pure extraction — all 103 pre-existing tests still pass (61 lib + 1 + 30 + 1 + 3 + 1 + 6 = 103, matching the plan's Task 1 claim exactly).

## Round-3 fixes

| # | Fixed? | Note |
|---|---|---|
| C-1 | **YES** | Measured end-to-end. Clean vs. one-space-padded invocation: **byte-identical UF2** (`e4418e04…`, 512 B) and now the **same** hash `fcc7 217e d26e 4ad7 bff2 e52c 4467 187e`, equal to `me hash --unsealed`. Reverting to `.as_str()` reproduces round 3's `2aee 5d32 …` exactly. |
| C-2 | **YES** | Mutation (c) now dies: 1 failure of 116, and it is exactly `refuses_an_uppercase_bip39_mnemonic`. |
| I-1 | **YES** | `decode_public_set` compiles; per-arm stringify is sound. |
| I-2 | **YES** (retraction) | `encode_section(&[String])` compiles. No dangling `Zeroizing` — `main.rs:8`'s import is pre-existing and still used by `run()`. Zero unused-import warnings. But see **M-1**. |
| I-3 | **YES** | `to_vec()` compiles. |
| M/N | **YES** | Measured **53 / 55 / 10**. Per-module: wire 8, crypto 5, passphrase 5, record 7, pubhash 5, container 4, `mod` 19, uf2 2 — the plan's breakdown is right in every term. |

## Mutations — all six killed by exactly the named test

| | Mutation | Failing tests | Killer hit |
|---|---|---|---|
| a | group by HRP alone | 1 | `vector_g_multisig_public_section_spans_six_cards` |
| b | combined 1..24 cap deleted | 1 | `refuses_more_than_24_records_across_both_sections` |
| c | lowercase guard deleted from `record_or_mnemonic` | 1 | `refuses_an_uppercase_bip39_mnemonic` ← **round-3 survivor, now dead** |
| d | iterations guard below the public-only return | 1 | `refuses_out_of_range_iterations_on_the_public_only_path` |
| e | hash refs reverted to `.as_str()` | 1 | `printed_hash_matches_me_hash_regardless_of_surrounding_whitespace` ← **C-1's new killer, verified live** |
| f | CR scan deleted from `encode_section` | 1 | `refuses_embedded_separators_and_bad_lengths` |

## Vectors

A–G unchanged: `git diff 6e5c0bd 2ed2695` touches **no** vector literal (the only added vector-shaped line is the new uppercase test's salt/iv args). All seven vector tests pass against their pinned lengths and sha256s.

Vector G grouping, instrumented rather than inferred:
```
GROUPS: [(('d', Some(841149), 0), 6), (('k', Some(153720), 0), 2),
         (('k', Some(153721), 0), 2), (('k', Some(153723), 0), 2)]
```
**Four cards** — 841149 ×6, and three mk1 cards of 2 chunks each. Exactly the doc comment's claim.

## New defects

### IMPORTANT 1 — the "CR trimmed at the CLI" mutant has no killer and survives the whole suite

**Location:** plan line 2386 (mutation table) / line 2242 (`run_seal_cli`), `design/IMPLEMENTATION_PLAN_encrypted_payload_hostA.md`.

**Defect:** the row reads `| CR trimmed at the CLI before the container sees it | §11.2 CLI case: me seal --plaintext "md1…\r" must be REFUSED |` — a **manual invocation**, not a test function. This is the identical shape round 3 graded C-2 ("a manual invocation is not a killer"); the fold re-pointed the uppercase row and left this one.

**Measured.** Unmutated: `me seal --plaintext "<md1>\r" …` → `me: record 0 contains '\r'`, exit 4, no file. Mutating `run_seal_cli` to `plaintext.iter().map(|s| s.trim().to_string())` — the exact thing the four-line comment above that binding exists to forbid — the CLI **accepts** it, exit 0, writes the UF2, **and the full 168-test suite stays green** (116/1/30/1/3/1/6/10, zero failures). §6.4's "CRLF is rejected, not tolerated" silently becomes normalisation with nothing to notice.

Graded Important rather than Critical because, unlike C-1, the mutant's blob and hash are *correct* (`fcc7 217e …`, 512 B — same as clean); the harm is a normative refusal loosened with no regression guard, not a wrong result. The controller may reasonably grade it Critical by precedent with C-2.

**Fix:** add to `tests/seal_cli.rs` and point the row at it —
```rust
#[test]
fn refuses_a_record_carrying_a_cr() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    me().args(["seal", "--plaintext", &format!("{MD1}\r"), "--out", out.to_str().unwrap()])
        .assert().failure().stderr(predicate::str::contains("record separator"));
    assert!(!out.exists());
}
```

### MINOR 1 — the I-2 retraction left the false claim it was folded to remove

**Location:** plan lines 2235‑2237, inside `run_seal_cli`.

The fold deleted the `Zeroizing` mechanism and its description but kept the sentence above it: `// Global Constraint: secrets wiped on every path. … so this is defence in depth on the heap copy we control.` — sitting directly over `let secret: Vec<String> = payload.to_vec();`, which zeroizes nothing. It also quotes a Global Constraint sentence that no longer exists: the constraint now says **"The record `Vec<String>` is NOT zeroized"**. This is precisely the "claim a guarantee that is not delivered" that I-2 was folded to stop. Delete the three lines, or restate them as "not zeroized — argv exposes these anyway (§9)".

### MINOR 2 — the new test inherited another test's doc comment

**Location:** plan lines 1585‑1599.

The insertion put `refuses_an_uppercase_bip39_mnemonic` between the existing doc comment and the function it documented. The uppercase test now carries "The guard sits above `seal()`'s public-only early return … both other iteration tests take the secret path" — the rationale for mutation **(d)** — and `refuses_out_of_range_iterations_on_the_public_only_path` is left with no doc comment at all. Compiles and passes; but it files (d)'s recorded rationale under (c)'s test, so deleting one loses the other's justification. Move the first four lines back down.

### NIT 1 — vector G's test name still says "six cards"

`fn vector_g_multisig_public_section_spans_six_cards` — the doc comment immediately above it says FOUR, records that "an earlier draft said six cards", and I measured four. Commit `6e5c0bd` corrected the prose and not the identifier. The mutation table quotes the six-card name too.

### NIT 2 — `cargo clippy --all-targets -- -D warnings` fails (5 lints in the plan's test code)

4 × `manual_repeat_n` (`container.rs:89,91`, `mod.rs:24,224`) and 1 × `needless_borrow` (`passphrase.rs:69`). The plan's **stated** gate (`cargo clippy -p mnemonic-engrave -- -D warnings`) passes, and CI runs no clippy at all (`.github/workflows/release.yml` is `cargo test --locked` at 1.85.0) — so this blocks nothing. Informational only; `repeat_n` is 1.82-stable and safe under MSRV if anyone does tidy them.

### NIT 3 — Global Constraint says "all 18 Task 7 tests"; measured 19

Off by one against the test the same commit added.

### NIT 4 — the new test locates the hash line by "8 whitespace tokens"

`err.lines().find(|l| l.split_whitespace().count() == 8)`. The `load:  picotool load --verify <path>   (machine in BOOTSEL)` line **also** has exactly 8 tokens. `.find()` returns the first and the hash line precedes it, so it is correct today — but this is the same token-count heuristic that `passphrase_line`'s own doc comment (plan lines 2005‑2012) warns made an assertion vacuous. Anchoring on the preceding `public data hash (` line would be robust.

## VERDICT

```
Critical: 0   Important: 1   Minor: 2   Nit: 4
GATE: BLOCKED
```

The six named mutations and both round-3 Criticals are genuinely, executably fixed — this is a real close on C-1/C-2/I-1/I-2/I-3 and the counts. The single blocker is a seventh row in the plan's own mutation table that was never given a killer, of exactly the class round 3 graded Critical; it is one ~6-line test away from GREEN and needs no re-derivation of anything above.

**CONFIDENCE: execution.** Every claim above is a measurement: full assembly compiled and run (168 tests, twice — nightly and MSRV 1.85.0 `--locked`), seven mutations applied-and-restored under an assert-the-anchor-matched harness with the failing-test set captured each time, the C-1 hash/blob equality checked against real `me seal`/`me hash` invocations, and vector G's grouping printed from an instrumented `decode_public_set` rather than inferred. One process note: my first C-1 check ran a stale binary left over from mutation (e) and appeared to show C-1 unfixed — I caught it, rebuilt, and the numbers above are from the rebuilt binary. Nothing here is read-only inference except the two NITs about naming and counts.

**User tree: clean** (`git status --porcelain` empty, no stashes). All work was done under the scratchpad.
