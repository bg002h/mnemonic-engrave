# `me bundle` Phase A — execution review (two-stage, R0→R1 GREEN)

- **Stage:** post-implementation two-stage review (spec-compliance + code-quality/architect), the per-phase gate. Subagent-driven implementation on `feat/me-bundle-phaseA` (commits `93bbaca..7eeb7d8`).
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-reviewer` (adversarial).
- **Outcome:** R0 NOT-GREEN (0C / 2I / 2m) → fold `7eeb7d8` (add spec §10 tests #2/#6/#9) → R1 **GREEN (0C/0I)**.
- **Final build:** 46 tests pass (33 lib / 11 cli / 1 cross-lang / 1 golden), clippy `-D warnings` clean, fmt clean.

## Notable: implementation-time deviation (verified correct)
The plan's md1 chunked-vs-unchunked discrimination relied on `md_codec::chunk::ChunkHeader::read` returning `ChunkHeaderChunkedFlagMissing` for an unchunked md1. Against the real md-codec 0.36, an unchunked md1 (`md1yqpqqxqq8xtwhw4xwn4qh`, first symbol `0b00100`) instead yields `WireVersionMismatch{got:2}` (read checks the top-4-bit version before the chunked flag). TDD caught it (`parses_unchunked_md1_as_bch_only` failed on the literal plan code). The implementer fixed it using md-codec's OWN canonical discriminator — bit 0 of the first 5-bit symbol (`symbols.first() & 0x01`, exactly `decode_with_correction`'s auto-dispatch) — branching first, preserving the plan's chunked-path arms. The R0 review verified this in source (extraction via `read_bits(5)` then `& 0x01` is correct; the wrong `bytes[0] & 0x01` was explicitly ruled out). This is the design's intended behavior.

---

## VERBATIM — R0 (two-stage)

[Stage 1 — spec compliance: PASS across §3–§11, except §10 tests #2 and #6 absent (#9 partial). Stage 2 — code quality: deviation analysis (a/b/c/d) all CONFIRMED correct; security spine, no-panic, run_bundle correctness, manifest serialization, clap, all PASS.]

### Issues (R0)
**Critical:** None.
**Important:**
- **I-1:** Spec §10 test #2 missing — one md1 + two distinct mk1 sets (different `chunk_set_id`) → `sets[]` has 2 entries; the multi-chunk_set_id BTreeMap grouping path untested.
- **I-2:** Spec §10 test #6 missing — two mk1 with mismatched `chunk_set_id` presented as one intended set → two incomplete groups → exit 4. The existing `cross_chunk_hash_mismatch_fails` uses the SAME id (test #7), so #6 is uncovered.
**Minor:**
- **M-1:** test #9 (pristine policy) only exercised via `parse_line`, not end-to-end through `run_bundle`.
- **M-2:** `parse_line` re-decodes (`validate` + `decode_string`/`unwrap_string` twice) — redundant work, not a bug; accepted.

**R0 Verdict: NOT-GREEN (0 Critical / 2 Important)** — spec §10 #2 and #6 absent; all other requirements correctly implemented; deviation analysis fully sound.

### Deviation analysis (R0, condensed verbatim)
(a) bit 0 of the first 5-bit symbol IS md-codec's canonical chunked-flag — `chunk.rs` docstring "[v3][v2][v1][v0][chunked]"; `decode_with_correction`: `symbols.first().map(|s| s & 0x01)`. (b) chunked md1 first symbol `0b0100_1`=9, `9&1`=1 → `ChunkHeader::read` → chunk_set_id; correct. (c) unsupported wire version: CHUNKED bad version → `WireVersionMismatch` → exit 4; unchunked bch-only already BCH-pristine-validated (not re-checking its internal version is per-spec). (d) extraction: `unwrap_string` packs symbol0 into the top 5 bits of byte 0; `BitReader::read_bits(5)` returns symbol0; `&0x01` = chunked flag — matches md-codec exactly; the wrong `bytes[0] & 0x01` would read a data bit of symbol1 — correctly avoided.

---

## VERBATIM — R1 (convergence)

**I-1 (spec §10 #2) — RESOLVED.** `multi_set_two_distinct_mk1_cards` feeds `[MD1_UNCHUNKED, MK1_A, MK1_B, MK1_C, MK1_D]`; asserts `sets.len()==2` (both `Kind::Mk1`, `SetVerified`), one id `0x12345`, distinct ids, `wallet_plates==6`, ms1 last. Genuinely exercises two independent BTreeMap groups, both passing `mk_codec::decode`. MK1_C/MK1_D proven real (a clean 2-chunk set @ `0x23456`).

**I-2 (spec §10 #6) — RESOLVED.** `foreign_mismatched_set_ids_fail`: `[MK1_A, MK1_C]` — one chunk from each of two distinct sets (0x12345 / 0x23456), each an incomplete 1-of-2 group → `Err(SetIncompleteMk(..))`. Structurally distinct from the same-id cross-hash test #7.

**M-1 (spec §10 #9 via run_bundle) — RESOLVED.** `run_bundle_rejects_corrupted_mk1`: a 1-symbol-flipped mk1 through the full `run_bundle` pipeline → `Err(Validate(_, ValidateError::MkCorrected(_)))`.

**Scope:** `7eeb7d8` touched ONLY the `#[cfg(test)]` module of `bundle.rs` (+2 consts, +3 tests). Production code (`run_bundle`/`parse_line`/manifest/`main.rs`) UNCHANGED — all R0 PASS findings (spec §3–§11, deviation analysis, security spine, manifest schema, golden) still hold. No new test vacuous; no const collision.

### Issues (R1)
**Critical:** None. **Important:** None. **Minor:** None.

**R1 Verdict: GREEN (0 Critical / 0 Important).** All three R0 open findings resolved by `7eeb7d8`; production code untouched; new tests well-formed and non-vacuous; no regressions.

---

## Loop summary
| Round | Verdict | Folded |
|---|---|---|
| R0 (two-stage) | NOT-GREEN | 2 Important (spec §10 tests #2 multi-set + #6 foreign-set-id absent) + 2 Minor (M-1 #9 via parse_line only; M-2 redundant decode — accepted) |
| **R1** | **GREEN** | 3 tests added (`7eeb7d8`): multi-set, foreign-mismatched-set-id, run_bundle pristine — converged |

Implementation complete & GREEN. md1-discrimination deviation verified correct against md-codec source. Next: finish the branch (merge/PR) + ship `me` v0.2.0.
