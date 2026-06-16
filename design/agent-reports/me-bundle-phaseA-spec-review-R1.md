# `me bundle` Phase A — SPEC architect review — R1 (convergence)

- **Stage:** R1 convergence of the mandatory spec R0 gate. Re-dispatch after the R0 fold.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (adversarial).
- **Scope:** folded `design/SPEC_me_bundle_phaseA.md` vs the R0 findings (`me-bundle-phaseA-spec-review-R0.md`).
- **Verdict:** **GREEN — 0 Critical / 0 Important** (1 new Minor new-minor-1, since folded). Spec R0 gate converged; spec is implementation-ready.

---

## VERBATIM REVIEW OUTPUT

I now have both documents. Let me verify each fold carefully against the spec text.

---

## Per-finding status

**I-1 (mk1 SingleString unhandled): RESOLVED**

§4 mk1 bullet now states explicitly: "If `is_chunked()` is **false** (a `SingleString` header — possible only for a synthetic mk1 with ≤56-byte bytecode, never a real-world card) → **exit 4**, message `"mk1 SingleString header: unsupported for bundle (no chunk_set_id)"`". This directly implements the R0 concrete fix: the unspecified branch is now named, the exit code is 4 (correct for invalid input — §3/§8 define exit 4 as "invalid string or set-integrity failure," which a non-groupable mk1 header satisfies), and the message is accurate (the string genuinely has no `chunk_set_id` to group by). The fold is sound.

**I-2 (WireVersionMismatch conflated with unchunked): RESOLVED**

§4 md1 bullet now presents a clean 4-way dispatch:
- `Ok(header)` → chunked; use `header.chunk_set_id`
- `Err(ChunkHeaderChunkedFlagMissing)` → unchunked bch-only
- `Err(WireVersionMismatch{..})` → exit 4, "unsupported md1 wire version"
- any other `Err` → exit 4

This directly implements the R0 concrete fix. The critical distinction is preserved: a version=4, chunked=0 md1 (real unchunked v0.30) returns `ChunkHeaderChunkedFlagMissing` and is correctly classified as bch-only; a future version≠4 md1 returns `WireVersionMismatch` and is correctly rejected rather than silently misclassified as unchunked. No conflation remains.

**m-1 (Vec<&str> from Vec<String>): RESOLVED**

§4 "Branching" sub-block now states: "strings are owned as `Vec<String>`, so per group build `let refs: Vec<&str> = group.iter().map(String::as_str).collect();` before the call." This is the exact idiomatic pattern needed for both `mk_codec::decode(&refs)` and `md_codec::chunk::reassemble(&refs)`.

**m-2 (unchunked md1 sets[]/plates[] split ambiguity): RESOLVED**

§6 now states explicitly in the prose block: "An **unchunked single md1** is NOT a chunk set: it appears **only in `plates[]`** with `integrity: "bch-only"` and **no `chunk_set_id`/`chunk_index`**, and is **omitted from `sets[]`**." The example JSON's md1 entry is clarified as "a *chunked-of-1*" (`total: 1`, with a `chunk_set_id`, `set-verified`). The `integrity` tristate definition at the start of §6 now reads: "`"bch-only"` (single unchunked md1 — per-string BCH passed, no cross-chunk hash exists; resolves m-3)". The schema is now unambiguous for the unchunked case.

**m-3 (CrossChunkHashMismatch construction path): RESOLVED**

§10 test #7 now says: "construct the vector via the public path (resolves m-3) — `mk_codec::encode_with_chunk_set_id` to get a valid set, then swap one chunk for a *different* set's same-index chunk (or re-encode a different KeyCard at the same `chunk_set_id`) so each chunk is individually pristine but the cross-chunk hash disagrees → `CrossChunkHashMismatch` → exit 4. (No codec-internals access needed; mirrors the `pipeline.rs` perturbation tests.)" This eliminates the overly cautious contingency and gives the implementer a direct, public-API construction path.

**m-4 (version normalization for golden test): RESOLVED**

§10 #12 now states: "**Normalize the `version` field before comparison** (resolves m-4) — e.g. overwrite it with a fixed `"x.y.z"` token, or compare a `serde_json::Value` with the `version` key removed — so a routine crate version bump does not break the golden." Also §6 prose confirms: "`"0.2.0"` literal above is illustrative, NOT hardcoded" and the `env!("CARGO_PKG_VERSION")` note is present. Both the schema definition and the test instruction are now concrete.

**m-5 (ChunkedHeaderMalformed imprecision): RESOLVED (accepted as-is)**

R0 assessed this as not a bug — "slightly imprecise grouping" — and the fold plan recorded: "§5 left as-is (architect: 'not a bug, slightly imprecise'); §5 already enumerates the variants correctly." §5 in the folded spec is unchanged from R0 baseline and correctly lists the mk1 error variants. No action was needed and none was taken. Accepted.

**clap `--stdout` collision: RESOLVED**

§3 now states: "Bundle is stdout-by-default + `--manifest` for a file" and explicitly: "(No bundle-specific `--stdout` flag — that name belongs to the converter path; reusing it here would collide.)" §9 reiterates: "`--manifest <file>` for a file — no bundle `--stdout`". The collision is explicitly foreclosed in both the CLI surface section and the file-structure section.

---

## Issues

### Critical

None.

### Important

None.

### Minor

**New-minor-1: Structural overlap between the §4 md1 "chunk_set_id extraction" sub-block and the §4 "Branching" sub-block creates redundant md1 description (clarity, not correctness)**

The §4 md1 bullet under "chunk_set_id extraction" already specifies the full 4-way dispatch (`Ok` → chunked; `ChunkHeaderChunkedFlagMissing` → bch-only; `WireVersionMismatch` → exit 4; other → exit 4). The immediately following "Branching (resolves I-1 + m-6)" sub-block then re-describes the same md1 paths in abbreviated form. These two descriptions are internally consistent — no contradiction — but the duplication means the branching sub-block re-covers ground already fully specified in the extraction sub-block. No implementer will be misled (the two are consistent), but the spec would be cleaner if the Branching sub-block's md1 row cross-referenced rather than re-described. Severity: Minor. New — introduced by the I-2 fold adding the 4-way dispatch alongside the pre-existing Branching sub-block.

---

## Assessment

**I-1 fold tracing:** The exit-4 branch for `SingleString` mk1 is well-placed in §4, occurs after `is_chunked()` returns false, names the exit code correctly (exit 4 = invalid/integrity; §3 confirms this), and the message is accurate. Sound.

**I-2 fold tracing:** The 4-way dispatch correctly mirrors the actual `ChunkHeader::read` execution path. For a real unchunked v0.30 md1: version=4 passes, chunked-flag=0 triggers `ChunkHeaderChunkedFlagMissing` → bch-only branch, correct. For a future-version md1 with version≠4: `WireVersionMismatch` before the chunked-flag check → exit 4, correct. The two failure modes are properly separated. Sound.

**m-2 fold tracing:** §6 clearly states unchunked md1 is absent from `sets[]` and present in `plates[]` as bch-only without `chunk_set_id`/`chunk_index`. The example JSON shows a chunked-of-1 md1 (consistent: a chunked-of-1 has a `chunk_set_id`, goes through `reassemble`, is `set-verified`). §10 test #8's added assertion matches §6. Plate ordering and `wallet_plates` still hold with an unchunked md1 present. No conflict.

**Regression sweep:**
- All 7 API claims remain valid; the fold edits touched §3/§4/§6/§9/§10 prose only — no signature references broken. The CRUX paths (mk1 `from_5bit_symbols`; md1 `ChunkHeader::read` via `BitReader::with_bit_limit`) are still correctly specified.
- §5 error mapping intact.
- §3 input/output/exit codes coherent (stdin/`--in` in; manifest to stdout default or `--manifest <file>`; checklist to stderr; exits 0/2/3/4 consistently applied).
- Phase-A scope clean; no Phase-B leakage (§2/§11 defer preview/release/Go interop).
- All GREEN design §A requirements present (prove set integrity; manifest; checklist; no ms1 ingestion/emission; converter backward-compat).

The only new issue (new-minor-1) is clarity redundancy, not a logical contradiction; no implementer ambiguity. Does not rise to Important.

---

## Verdict: GREEN (0 Critical / 0 Important)

All R0 findings — I-1, I-2, m-1 through m-5, and the clap `--stdout` collision — are genuinely resolved. The one new issue (extraction/Branching overlap for md1) is Minor with no implementer ambiguity. The 7 API claims are intact, the exit-code model is coherent, the manifest schema is unambiguous for all input categories, and the test list is complete. The spec is ready for implementation.

---

## Loop summary (added by main session)
| Round | Verdict | Folded |
|---|---|---|
| R0 | NOT-GREEN | 2 Important (I-1 mk1 SingleString unhandled; I-2 WireVersionMismatch conflated with unchunked) + 5 Minor + clap `--stdout` collision |
| **R1** | **GREEN** | 1 new Minor (new-minor-1 §4 extraction/Branching md1 overlap) → collapsed the Branching sub-block to a per-group-integrity statement; converged |

Spec R0 gate GREEN. Both CRUX API claims (mk1 chunk_set_id extraction; md1 BitReader/ChunkHeader::read) verified in source. Next: user reviews the spec → writing-plans (Phase A plan) → plan R0 gate → subagent-driven implementation.
