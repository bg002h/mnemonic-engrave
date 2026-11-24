0C/0I/0M/0N

# Whole-diff review: me bug-hunt build (production head 8f91ff54 vs base 286c3bf9)

Reviewer: Sonnet 5.5. Read-only on the fix worktree. Execution was in a scratch worktree (review-wd at 8f91ff54, CARGO_TARGET_DIR /scratch/cache/bughunt/me/review), since removed.

Not checked: the gate figures the brief marks settled (705/705, fmt, clippy, mutation counts) were not re-run. The FIX-vs-diff check below was a read of `git diff 286c3bf9 8f91ff54` against FIX sections 1 and 2. Quotes are paraphrased from the FIX where marked "paraphrase", not verbatim; only the code lines were read verbatim.

## 1. FIX vs diff, item by item

- L5, `seal/uf2.rs`. FIX: add `use super::wire::REGION_LEN;` after the module doc, and as first statement of `to_uf2` an `assert!(blob.len() as u64 <= REGION_LEN, "seal blob {} bytes exceeds the {REGION_LEN}-byte seal region", blob.len())` before the `debug_assert!`. Code: the same `use` line after the module doc (uf2.rs:7), and the identical assert as the first statement, before `debug_assert!`. Matches.
- #3, `sysw/mod.rs`. FIX: before the final `seal_with(...)`, after the passphrase checks, add `if payload.secret.is_empty() { let mut blob = header.encode().to_vec(); blob.extend_from_slice(&pub_bytes); return bound(blob); }`. Code: identical block, placed after the `NotEnterableOnDevice` loop and before `seal_with`. Matches.
- #2, `preview.rs`. FIX: after `let out_path = ...` add a `match std::fs::remove_file(&out_path)` with `Ok(())` and `NotFound` arms passing and any other error returning `PreviewError::Spawn(e)`. Code: identical, placed before `Command::new(sidecar)`. Matches.
- #1, `mnemonic-io-lib/src/write.rs`. FIX steps 1-4: (1) drop `truncate(true)`; (2) after the cfg(unix) `set_permissions` block add `if f.metadata()?.is_file() { f.set_len(0)?; }` outside cfg; (3) replace the doc paragraph; (4) rename the test text. Code: all four present. The doc paragraph and both test strings match the FIX text.
- #4, `main.rs`. FIX: (1) add the `as_present: bool` parameter; (2) `in_path.filter(|_| argv.is_empty())`; (3) insert the argv-and-`--in` block after the guard loop and before the `-` stdin comment; (4) the caller passes `r#as.is_some()` in the fmt-wrapped form; (5) append `, false` to the two unit-test calls. Code: all five present. The refusal text `pass records on argv OR via --in, not both` with `EXIT_USAGE` and the `--as` pass-through return `Records { records: argv.to_vec(), document: argv.join("\n") }` match.
- Spec delta (paraphrase of the FIX table): #4 row at SPEC_systemwide_payloads.md:908 has the appended sentence. The #2 A1 paragraph, the L5 §5 paragraph, and the three #1 edits in SPEC_me_preview_hardening.md (lines 106, 108, 118-120) are all present with the FIX wording. No divergence found. The `_` in the preview-hardening L108 edit keeps the "`fs::write` truncates; without it ..." remainder as the FIX specifies.
- The diff touches nothing under `firmware/`, `third_party/` or the submodule. Not exhaustively checked beyond `git diff --stat`, which lists only the nine files in scope.

## 2. Tests can fail

Method: in the scratch worktree, `git apply -R` of each fix commit's production hunk (`crates/me-cli/src` and `crates/mnemonic-io-lib/src` only, tests kept), then `cargo +1.85.0 nextest run --locked --workspace -E 'test(/^bughunt_me_/)'`, then re-apply. Each was applied and restored cleanly.

| reverted commit | failing rows | summary line |
|---|---|---|
| 05c57012 (L5) | 05, 05b | 14 run: 12 passed, 2 failed |
| 12357ca5 (#3) | 03, 03b | 12 passed, 2 failed |
| 2c6587d4 (#2) | 02, 02b | 12 passed, 2 failed |
| e387a82d (#1) | 01 | 13 passed, 1 failed |
| 8f91ff54 (#4) | 04, 04b, 04c, 04d | 10 passed, 4 failed |

Every fix has at least one row that goes red when it is reverted. Row 01b (FIFO guard) stays green under the #1 revert by design: it guards against a regression in the fix, not against the bug. Its ability to fail comes from the mutation gate, not from my revert run.

## 3. Timeout mutant (`main.rs:1482 run_sysw -> 0`)

Reasoned from the test source, not run by me. Row 01b spawns a reader thread doing `std::fs::read(ff)` on a FIFO, then runs `me sysw pack --out <ff>`. If `run_sysw` returns 0 without writing, nothing opens the FIFO for writing, so the reader's `open` blocks forever and `reader.join()` never returns. The test hangs and the mutation run times out. A timeout counts as detection, because the mutant cannot pass.

## Findings

None.

0C/0I/0M/0N
