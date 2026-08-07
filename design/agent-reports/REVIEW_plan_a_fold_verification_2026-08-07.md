# Fold verification — Plan A whole-diff review response

- **Date:** 2026-08-07
- **Reviewer:** independent sonnet agent (mechanical/verification tier), read-only
- **Under review:** `40d2749` only — the fold responding to
  `REVIEW_plan_a_whole_diff_2026-08-07.md` (persisted at `c67733f`)
- **Scope given:** two questions only — did the fold fix each finding, and did it
  introduce a new defect. Explicitly forbidden from a fresh audit; the production
  code's clean opus review at `c67733f` was stated as settled.

This closes the R0-style loop for the implementation gate. Per the standing
proportional-re-review rule, a re-review returning 0C/0I **closes** the loop —
no further round.

---

## VERBATIM REPORT

Only the two files listed, matching the commit message exactly. All claims check out. Final verdict.

## Findings

**1. `[Important]` §6.6 sealed-byte unexercised in mixed shape — FIXED.**
New test `mixed_payload_prints_the_sealed_hash_not_the_unsealed_one` genuinely exercises a mixed payload (`--seal-secret MS1` + 5 `--plaintext` records, exit 0, `success()` asserted). Confirmed by mutation on a copy-then-restore cycle: reverting `sealed.passphrase.is_some()` → `false` in `main.rs` makes exactly this test fail (`left: "70f3 e35a…" right: "a26e d22b…"`), the other 11 CLI tests stay green, and the tree recompiled clean after restore. The banner literal `"public data hash (5 records, SEALED):"` matches `main.rs:376-383` byte-for-byte (5 plaintext records, `sealed.passphrase.is_some()` true). Both pinned hex literals verified against `design/SPEC_encrypted_payload_delivery.md` §11.4: vector D sealed = `a26e d22b b747 dfd0 2367 06ad 14c1 9679`, vector E unsealed = `70f3 e35a acf7 47db c40f 8376 91aa 61e0` — both match exactly. The added cross-check against `me hash --sealed` invokes an independently-coded path (`run_hash_cli`, not `run_seal_cli`), so it is not self-referential; it supplements the literal pin rather than replacing it.

**2. `[Minor]` stdout not asserted empty — FIXED.**
`.stdout(predicate::str::is_empty())` added to `seals_and_prints_the_passphrase_to_stderr_only`. Confirmed by mutation: adding a stray `println!` alongside the passphrase `eprintln!` in `main.rs` makes this test fail with the exact leaked passphrase text; restored clean, recompiled, green.

**3. `[Minor]` vector G's §6.6 hash unpinned — FIXED.**
Literal `be11 7b56 9cc4 cd6e b47d 32b6 fd32 ccb8` added to `vector_g_multisig_public_section_spans_four_cards` in `seal/mod.rs`, matches SPEC §11.4 vector G exactly, called with `sealed: true` (correct — G is a mixed payload) against 12 public records (matches the `(public.len(), secret.len()) == (12, 3)` assertion just above it). Confirmed by mutation: flipping the `sealed` arg to `false` fails with `left: "fc10 4898…" right: "be11 7b56…"`, matching the commit message's claimed transcript exactly; restored clean, recompiled, green.

**New defects introduced by the fold: none found.**
- No self-referential assertion shape reintroduced (all three new assertions compare production output against independently-sourced literals or an independently-coded path).
- `git show 40d2749 --name-status` touches only `crates/me-cli/src/seal/mod.rs` and `crates/me-cli/tests/seal_cli.rs`; both changes are inside `#[cfg(test)]`/integration-test code — "test-only, no production code changed" is accurate.
- Numbers re-verified independently: `cargo test -p mnemonic-engrave` → 170 passed, 0 failed (116+1+30+1+3+1+6+12+0); `cargo clippy --all-targets -- -D warnings` → clean; `cargo fmt --check` on both touched files → clean (unrelated pre-existing formatting drift exists in `lib.rs`, `preview.rs`, other lines of `main.rs`, and `tests/cli.rs`, but none of it is in the two files this commit touches, so it's out of scope here).
- Tree left clean throughout (`git status --short` empty after every mutation-and-restore cycle).

Clean fold.

`VERDICT: 0 Critical, 0 Important, 0 Minor, 0 Nit`

---

## Gate status

**Plan A implementation is GREEN**: full validation suite passes (170/170) and
0 Critical / 0 Important across both the whole-diff review and this fold
verification. The next gate is Plan B (firmware), which binds to the vectors
this implementation emits and may never lead them (Rust-primary rule).
