# Spec-stage architect review — R3 — `mnemonic-engrave` / SeedHammer II engrave

- **Stage:** brainstorm/spec R3 (re-dispatch after R2 fold) — convergence check
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect`
- **Spec under review:** `design/SPEC_seedhammer_engrave.md` (post-R2-fold)
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Reviewer-loop converged; spec passes the pre-implementation R0 gate.

---

## VERBATIM REVIEW OUTPUT

All citations verified against source. The fold is correct and complete.

## Verification results

**1. Every md/mk source-path citation in §7 and §11 now correct? Yes.**

§7 PR2 "Representation conversion" (line 117):
- `md-codec/src/bch.rs:7-21` `MD_REGULAR_CONST`, `GEN_REGULAR` — confirmed: `GEN_REGULAR` at bch.rs:7, `MD_REGULAR_CONST` at bch.rs:17 (within 7-21). Top-level path correct.
- `mk-codec/src/consts.rs:18,21` `MK_REGULAR_CONST`, `MK_LONG_CONST` — confirmed: consts.rs:18 `MK_REGULAR_CONST`, consts.rs:21 `MK_LONG_CONST`.
- generators in `mk-codec/src/string_layer/bch.rs:173` `GEN_REGULAR`, `GEN_LONG` — confirmed: `GEN_REGULAR` at string_layer/bch.rs:173; `GEN_LONG` at :203 (same file, citing the block anchor 173).

§7 PR2 "Code coverage" (line 118):
- md1 regular-only `md-codec/src/bch.rs:1` — confirmed: bch.rs:1 doc comment "regular code only".
- mk1 regular+long `mk-codec/src/string_layer/bch.rs:318-347` — confirmed: `bch_create_checksum_long` at :323, `bch_verify_long` at :340-347, long-code verify squarely in range.

§11 (line 143): source-of-truth = `md-codec/src/bch.rs` and `mk-codec/src/{consts.rs,string_layer/bch.rs}`; "md-codec has no consts.rs, mk-codec's BCH lives under string_layer/" — all consistent with filesystem (md `bch.rs` is top-level; mk targets in `consts.rs`, generators under `string_layer/bch.rs`).

**2. New Critical/Important from the fold? None.** No dangling references to the old `mk-codec/src/bch.rs`; all line numbers land on the cited symbols.

**3. Verdict: GREEN (0 Critical / 0 Important).**

---

## Loop summary (added by main session)

| Round | Verdict | Open C / I at entry | Action |
|---|---|---|---|
| R0 | SOUND-WITH-CHANGES | 1 Critical, several Important | folded NDEF framing, mk multi-string reframe, BCH validation, argv→stdin, PR split |
| R1 | NOT-GREEN | 0C / 1I (+3 minor) | folded GF32↔u128 representation note, code-coverage, consts path, cross-chunk-hash callout |
| R2 | NOT-GREEN | 0C / 1I | folded mk BCH path `bch.rs` → `string_layer/bch.rs` (fold-induced drift, caught by re-dispatch) |
| **R3** | **GREEN** | **0C / 0I** | none — converged |

Spec cleared the mandatory pre-implementation R0 gate at R3.
