# Plan-stage architect review — plan-R1 — `me` converter

- **Stage:** implementation-plan R1 (re-dispatch after plan-R0 fold) — convergence check
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect`
- **Plan under review:** `design/IMPLEMENTATION_PLAN_me_converter.md` (post plan-R0 fold)
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Plan cleared its mandatory pre-implementation R0 gate.

---

## VERBATIM REVIEW OUTPUT

I have all the evidence I need. Let me verify the critical correctability question (C1b) carefully, since that is the subtlest point.

The key claim in `rejects_corrupted_mk1` is that flipping the LAST character (a checksum symbol) of a valid mk1 string yields `Ok{corrections_applied:1}`, not `Err(BchUncorrectable)`. Let me trace the source evidence.

**Evidence chain for C1(b):**

The test flips the last char of `MK1_VALID` (line 475-477). The last char is a checksum symbol. In `decode_string` (bch.rs:645), the full data part (including checksum) is parsed to 5-bit `values`, then passed to `bch_correct_regular`/`bch_correct_long` (line 678-682), which corrects up to t=4 substitutions anywhere in `data_with_checksum` — including checksum positions. The proof that a single checksum-symbol corruption is correctable is the in-tree test `bch_correct_returns_correction_result_with_position` (bch.rs:1160-1173): it damages checksum position 9 and asserts `corrected_positions == vec![9]` succeeds. And `decode_recovers_one_error` (bch.rs:1286-1301) does the exact `'q'↔'p'` flip the plan uses and asserts `corrections_applied == 1`. So a single-symbol flip (data OR checksum) is BCH-correctable → `Ok{corrections_applied:1}` → `MkCorrected(1)`. Confirmed: a 1-symbol flip is never uncorrectable for t=4 BCH.

Now let me give the verdict.

---

## Verdict: GREEN (0 Critical / 0 Important)

plan-R1 converges. All five plan-R0 findings RESOLVED; the folds introduce no new Critical/Important. Evidence below.

### Per-finding

**C1 [Critical] — mk1 pristine requirement. RESOLVED.**
- (a) `DecodedString.corrections_applied: usize` is a real public field — `bch.rs:567` (struct, `#[non_exhaustive]` but field is `pub`), declared at `bch.rs:571`. Field access `decoded.corrections_applied` (plan line 429) type-checks; `#[non_exhaustive]` only blocks struct-literal construction, not field reads, so the plan compiles.
- (b) **The crux — a 1-symbol flip is BCH-CORRECTABLE, not uncorrectable.** `decode_string` runs `bch_correct_regular`/`_long` over the full data+checksum with t=4 capacity. In-tree proof: `decode_recovers_one_error` (`bch.rs:1286-1301`) performs the identical `'q'↔'p'` single-char flip and asserts `corrections_applied == 1`; `bch_correct_returns_correction_result_with_position` (`bch.rs:1160-1173`) proves a flip on a checksum symbol is also correctable. So `Ok{corrections_applied:1}` → `Err(MkCorrected(1))`, and `matches!(.., Err(ValidateError::MkCorrected(_)))` holds. No flag.
- (c) Pristine `MK1_VALID` → `corrections_applied == 0` → `Ok` (`bch.rs:393-399`, `bch_correct_regular_clean_input` `bch.rs:1085`). `accepts_valid_mk1` consistent.

**C2 [Critical] — md1 vector. RESOLVED.** `md1yqpqqxqq8xtwhw4xwn4qh` is the current md-codec vector: `md-cli/tests/smoke.rs:21` (`stdout("md1yqpqqxqq8xtwhw4xwn4qh\n")`, the v0.30 re-pin). 24 chars → Task 6 golden "32 bytes" holds. Passes `unwrap_string` (HRP `md` + 11 data + 13 checksum = 24, `codex32.rs:113-161`). Zero occurrences of the stale string remain in the plan.

**I1 [Important] — citation accuracy. RESOLVED.** Task 3 comment + Step 2 now state md1 must be regenerated (md CLI / `encode_md1_string`; `test_vectors.rs` holds TEMPLATES), mk1 copied from `v0.1.json`. Accurate.

**M1 [Minor] — TDD. RESOLVED.** TDD note added; Task 1/2 Step 1 relabeled "Write the module and its tests." (Task 3 Step 1 still reads "...+ failing tests" — harmless, consistent with the documented deviation.)

### New Critical/Important from the folds: NONE.
- `ValidateError::MkCorrected(usize)` variant, its `Display` arm, and the `matches!` assertion are internally consistent and type-check.
- `validate()` compiles: `decode_string` → `Result<DecodedString, mk_codec::Error>` mapped via `ValidateError::Mk`, then `decoded.corrections_applied` read. Sound.
- Pre-existing cosmetic nit (not introduced here, not C/I): `bch.rs:573,580` doc-comments say "chars after `md1`" — md-codec copy-paste in mk-codec docs.

**GREEN. Ship plan-R1.**

---

## Loop summary (added by main session)

| Round | Verdict | Findings folded |
|---|---|---|
| plan-R0 | NOT-GREEN | 2C (mk1 auto-correct ⇒ require pristine; stale md1 vector) + 1I (vector source citation) + 2 minor (TDD label, fixtures) |
| **plan-R1** | **GREEN** | none — converged |

Plan cleared the mandatory pre-implementation R0 gate at plan-R1. Eligible for execution.
