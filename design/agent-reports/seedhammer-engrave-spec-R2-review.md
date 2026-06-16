# Spec-stage architect review — R2 — `mnemonic-engrave` / SeedHammer II engrave

- **Stage:** brainstorm/spec R2 (re-dispatch after R1 fold)
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect`
- **Spec under review:** `design/SPEC_seedhammer_engrave.md` (post-R1-fold)
- **Verdict:** **NOT-GREEN** — 0 Critical, 1 Important (a path error the R1 fold itself introduced). Folded below; R3 required.

---

## VERBATIM REVIEW OUTPUT

I now have everything needed. Let me note the key path discrepancy and verify each finding.

## R2 Convergence Review — `mnemonic-engrave` SPEC

### Per-finding verdicts

**1. [Important] GF32-vector vs packed-u128 mismatch — RESOLVED.**
§7 PR2 "Representation conversion" (spec:117) is accurate. md is packed-u128 polymod: `GEN_REGULAR`/`MD_REGULAR_CONST` at `md-codec/src/bch.rs:7-17` with `polymod_step` (`bch.rs:23-32`). SeedHammer's `engine` stores `generator`/`residue`/`target` as `[]fe` GF(32) vectors (`checksum.go:11-18`), 13-element regular (`newShortChecksum`, lines 29-47) and 15-element long (`newLongChecksum`, lines 50-68). The "13 regular / 15 long" element counts match the Go source exactly. The claim that constants "cannot be copied across verbatim" and need u128→`[]fe` re-expression is correct, and the §9 parity test is the right gate. An implementer would not be misled.

**2. [Minor] consts.rs path (md has none) — RESOLVED.**
§7 (spec:117) and §11 (spec:143) cite `md-codec/src/bch.rs` and explicitly note "md-codec has no `consts.rs`." Confirmed: `md-codec/src/consts.rs` does not exist; the md constants live in `md-codec/src/bch.rs:7,17`.

**3. [Minor] mk needs regular AND long — RESOLVED (with a path defect, below).**
§7 "Code coverage" (spec:118) correctly states md1=regular-only, mk1=regular+long. The source confirms both: regular `bch_verify_regular` and long `bch_verify_long` exist, with `BchCode::{Regular,Long}` and `bch_code_for_length` selecting between them. The substance is right.

**4. [Minor] cross_chunk_hash recovery-time-only — RESOLVED.**
§4 (spec:70) is accurate. `chunk.rs:1-7` states the hash "is verified at reassembly. This catches dropped, reordered, or substituted chunks that the per-chunk BCH layer alone cannot detect." `reassemble_from_chunks` (`chunk.rs:109-203`) performs the only cross-chunk hash check. `chunk_index`/`total_chunks`/`chunk_set_id` are inside the BCH-covered data part: the `Chunked` header symbols (`header.rs:67-101`) are prepended to the fragment before checksumming (`header.rs:64-66` doc, `bch.rs:497-501`). Fold is correct.

### NEW finding introduced by the folds

**[Important] Wrong mk-codec BCH file path — introduced by the Finding-1/3 folds.** §7 (spec:117, spec:118) and §11 (spec:143) cite **`mk-codec/src/bch.rs`** (incl. precise `mk-codec/src/bch.rs:323-347`). **That file does not exist.** The mk BCH code lives in **`mk-codec/src/string_layer/bch.rs`** (`GEN_REGULAR` 173-179, `GEN_LONG` 203-209, `MK_REGULAR_CONST`/`MK_LONG_CONST` consumed line 20, long checksum 318-347). The R1 fold for Finding 1 cited `mk-codec/src/consts.rs:18,21` (correct: `MK_REGULAR_CONST` line 18, `MK_LONG_CONST` line 21) **plus** "`+ bch.rs`" — and that appended path is wrong. The cited line range 323-347 *does* land on `bch_create_checksum_long`/`bch_verify_long`, but only in the `string_layer/` file, not the cited `mk-codec/src/bch.rs`. This is the same class of defect as R1 Finding 2 (a source-of-truth path an implementer/parity-test author would follow and not find), now reintroduced for mk by the very folds that fixed it for md.

### Verdict

**NOT-GREEN.** Open list (1 Important, 0 Critical):
- §7 (spec:117-118) and §11 (spec:143): replace `mk-codec/src/bch.rs` → `mk-codec/src/string_layer/bch.rs` (md-codec's `bch.rs` is top-level and correct; mk-codec's is under `string_layer/`). Line range 323-347 is correct once the directory is fixed.

All four R1 findings are substantively resolved; the lone blocker is a mechanical but load-bearing path error the fold introduced.

---

## FOLD DISPOSITION (added by main session; not part of verbatim review)

| Finding | Sev | Disposition |
|---|---|---|
| Wrong mk-codec BCH path `mk-codec/src/bch.rs` → should be `mk-codec/src/string_layer/bch.rs` (§7 ×2, §11) | **Important** | FOLDED inline after independent `ls` verification. md-codec `bch.rs` is top-level (correct); only the mk path was wrong. |

R3 re-dispatch follows to confirm GREEN.
