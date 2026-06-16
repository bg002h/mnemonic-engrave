# `me bundle` (bundle orchestration) — DESIGN architect review — R1 (convergence)

- **Stage:** R1 convergence of the pre-spec DESIGN review. Re-dispatch after the R0 fold + Phase A/B restructure.
- **Date:** 2026-06-16
- **Reviewer:** `feature-dev:code-architect` (adversarial; verified all load-bearing claims against `md-codec 0.36.0`, `mk-codec 0.4.0`, and the converter's `validate.rs`).
- **Scope:** folded `design/DESIGN_me_bundle_preview.md` (Phase A full detail + Phase B deferred stub) vs the R0 findings.
- **Verdict:** **GREEN — 0 Critical / 0 Important** (1 new Minor m-6, since folded). Loop converged.

---

## VERBATIM REVIEW OUTPUT

I now have all the evidence I need. Let me verify one more critical aspect — whether `unwrap_string` for a single md1 CHUNK string (which has a chunk header as its first bits) would work as a per-chunk pristine pre-check:

Based on my reading of `chunk.rs` lines 305-389, `reassemble()` calls `unwrap_string(s)` per chunk at line 321 — this is the BCH verify step. Let me also confirm how `decode_with_correction` (the auto-correcting entry point) handles single-chunk dispatch, which is the key auto-dispatch question for I-1:

The evidence from `chunk.rs` lines 597-617 is definitive: `decode_with_correction` has an auto-dispatch block at line 599 — if `strings.len() == 1`, it inspects `chunked_flag = symbols.first().map(|s| s & 0x01)`, and if `chunked_flag == 0`, routes to `decode_md1_string`; if 1, falls through to `reassemble`. This auto-dispatch is inside `decode_with_correction` (the BCH-correcting path), not in `reassemble` itself.

Now I have all the evidence to write the full R1 review.

---

# `me bundle` (bundle orchestration) — DESIGN architect review — R1

- **Stage:** R1 (convergence), verifying fold of R0's NOT-GREEN findings.
- **Date:** 2026-06-16
- **Reviewer:** sonnet `feature-dev:code-architect` (adversarial; verified all load-bearing claims against real source).
- **Scope:** `design/DESIGN_me_bundle_preview.md` (folded design) vs `md-codec-0.36.0` source, `mk-codec-0.4.0` source, and the existing converter's `validate.rs`.

---

## Per-Finding Status

**I-1 (md1 chunking) — RESOLVED with one Minor caveat (see §Issues below)**

The fold folds md1 chunking into §A2.4 and directs the spec to use `md_codec::chunk::reassemble` for set-integrity. This is architecturally correct and the API exists. However, the design's specific claim in §A2.4 — "detect single vs chunked md1 (chunked-flag = bit 0 of the first 5-bit symbol, per md-codec decoder auto-dispatch)" — is technically correct but points to the wrong entry point for the pristine-first use case. The auto-dispatch (bit-0 inspection + routing to `decode_md1_string` vs `reassemble`) lives inside `md_codec::chunk::decode_with_correction` (`chunk.rs` lines 599-617), which is the BCH-CORRECTING entry point. The pristine pure-verify entry point, `md_codec::codex32::unwrap_string`, does NOT do this auto-dispatch: it unwraps the raw byte payload and returns it; the caller then has to parse the first symbol's bit 0 manually to distinguish single from chunked.

In practice §A2.3 calls `unwrap_string` for pristine pre-check, then §A2.4 calls `reassemble` for set-integrity — the two-step ordering is sound. But the design implies the auto-dispatch is in the pure-verify path, when it is only in the correcting path. This is a Minor precision error in the design prose, not an architectural hole: the actual two-step sequence (unwrap_string per chunk → manual bit-0 inspection → group + reassemble) works correctly with the public API and is exactly what `reassemble`'s internal code does (it calls `unwrap_string` per chunk at line 321, then parses the `ChunkHeader` from the resulting bitstream). The spec writer needs to be precise: pristine check = `unwrap_string`; chunked-flag detection = read bit 0 of first symbol from the unwrapped bytes; set-integrity = `reassemble`. The shorthand "auto-dispatch" is technically only true of `decode_with_correction`.

The public API surface (`lib.rs` lines 42-44) exports: `reassemble`, `split`, `ChunkHeader`, `CorrectionDetail`, `decode_with_correction`, `derive_chunk_set_id`. There is no single unified entry point that does "pristine-verify + chunked-vs-single detection + reassemble" — the tool must compose them. The fold's two-step approach is the correct composition.

**I-2 (decode auto-corrects) — RESOLVED**

Code evidence confirms the fold is sound:

`validate.rs` lines 41-55 in the existing converter show the exact reuse pattern: `Format::Md` → `md_codec::codex32::unwrap_string(s).map(|_| ())` (pure BCH verify, returns `Err` on any corruption — no correction); `Format::Mk` → `mk_codec::string_layer::decode_string(s)` then reject if `decoded.corrections_applied != 0`.

`bch.rs` lines 645-690 (`decode_string`) confirm it calls `bch_correct_regular` or `bch_correct_long`, which apply BCH correction and populate `CorrectionResult::corrections_applied`. A clean string gets `corrections_applied: 0` (`bch_correct_*` lines 393-398, 452-457).

`pipeline.rs` lines 118-151 (`decode`) calls `decode_string(s)?` per chunk (line 128) before assembling. This means `mk_codec::decode` silently BCH-corrects per chunk before the cross-chunk hash check.

The fold's §A2.3 pre-check sequence — call `validate()` (which internally uses `decode_string` + `corrections_applied != 0` rejection) on every mk1 chunk string BEFORE passing any to `mk_codec::decode` — closes the gap. No ordering hole: pristine rejection at step 3 means no chunk with a non-zero correction count ever reaches the `mk_codec::decode` call at step 4.

For md1 chunks: `unwrap_string` is a pure verify with no correction path — any BCH failure returns `Err`. An individual md1 chunk string IS individually parseable by `unwrap_string` (as confirmed by `reassemble`'s own implementation calling it per-chunk at line 321). So the pristine-first claim for md1 chunks holds.

**I-3 (QR params) — RESOLVED (Phase B)**

Correctly deferred to §B2, with the parameters pinned: "`qr.Encode(s, qr.L)` (error-correction level L, not M), `qrScale = 3`". Not lost.

**I-4 (sidecar version binding) — RESOLVED (Phase B)**

Correctly deferred to §B3: "`me` checks `me-preview --version` against the expected pin. Mismatch → clear warning/refusal." Not lost.

**m-1 (ms1 early-exit) — RESOLVED**

§A2.2: "On the first ms1 line → refuse immediately, exit 3, with the converter's RF-risk message — before validating any line (early-exit)." Explicit early-exit semantics pinned.

**m-2 (mismatched set-ids) — RESOLVED**

§A2.4: "Two strings of the same HRP with mismatched chunk_set_id → reported as separate sets (and each flagged incomplete if partial) rather than merged." Explicit semantics stated.

**m-3 (integrity field semantics) — RESOLVED**

§A3 manifest schema: `"bch-only"` for unchunked single md1, `"set-verified"` for chunk-set-proven, `"n/a"` for ms1 placeholder. Three-value semantics are unambiguous.

**m-4 (edge cases) — RESOLVED**

§A5 enumerates: empty input → exit 2; all-ms1 → exit 3 at first line; duplicate chunk_index → exit 4; incomplete set → exit 4; corrupted string → exit 4; manifest emitted only if every set is complete else exit 4.

**m-5 (B-spline fidelity) — RESOLVED (Phase B)**

§B2: "B-spline ControlPoint knots (multiplicity ≠ 3) must be interpolated, not drawn as line segments, or fonts mis-render. The spec must declare the fidelity target." Captured.

**Rec-4 (upstream pin) — RESOLVED**

§B1: "go.mod pins UPSTREAM seedhammer v1.4.2" — corrects the draft's wrong claim. Confirmed against the earlier R0 finding that `backup.EngraveText` is present in upstream v1.4.2.

---

## Issues

### Critical

None.

### Important

None.

### Minor

**m-6 (new): §A2.4 "auto-dispatch" prose is imprecise for the pristine path.**

The design's §A2.4 states "detect single vs chunked md1 (chunked-flag = bit 0 of the first 5-bit symbol, per md-codec decoder auto-dispatch)." The phrase "decoder auto-dispatch" technically refers to the behaviour inside `md_codec::chunk::decode_with_correction` (the BCH-correcting entry point, `chunk.rs` lines 599-617), NOT inside the pristine-verify entry point `md_codec::codex32::unwrap_string`. The pristine path requires the spec writer to: (1) call `unwrap_string(chunk_str)` for BCH-pure-verify; (2) manually inspect bit 0 of the first 5-bit symbol in the returned byte payload to detect chunked vs single; (3) for multiple md1 strings, always call `md_codec::chunk::reassemble(&[...])`. `reassemble` itself has no single-string auto-dispatch — passing a single unchunked md1 string to `reassemble` would fail with `ChunkHeaderChunkedFlagMissing` (because `ChunkHeader::read` at `chunk.rs` line 73 errors if `chunked == false`). Therefore the spec must explicitly describe the branching logic for a single md1 input (single unchunked: route to `decode_md1_string` after unwrap; single chunked count=1: route to `reassemble`; multiple: always `reassemble`). The current design prose leaves the boundary ambiguous for the case of a single input md1 string whose chunked-flag is 0.

Concretely: if `me bundle` receives exactly one md1 string and it is unchunked (single-payload), calling `reassemble(&[s])` would fail. The spec must state this branch. The fix is one sentence in §A2.4: "A single md1 string with chunked-flag = 0 is an unchunked single-payload; BCH-verify via `unwrap_string`, integrity = 'bch-only'. A single md1 string with chunked-flag = 1 is a chunked-of-1 set; call `reassemble`. Multiple md1 strings → always `reassemble`."

---

## Assessment

**I-1 fold correctness:** The two-step sequence — per-chunk `unwrap_string` for pristine BCH-verify, followed by `reassemble` for set-integrity — correctly composes the public API. The `reassemble` function internally calls `unwrap_string` per chunk again (chunk.rs line 321), so there is a double-verify on the BCH layer, which is harmless. The cross-chunk integrity check (chunk.rs lines 378-386) re-derives the `chunk_set_id` from the decoded descriptor and compares it to the header's advertised id — this is the correct "set-verified" proof. The single remaining precision gap is the §A2.4 prose about auto-dispatch, which is Minor m-6 above: the actual `decode_with_correction` auto-dispatch cannot be used in the pristine path, and passing a single unchunked md1 string to `reassemble` would fail. The spec must make the branching explicit.

**I-2 fold correctness:** The `validate.rs` reuse is clean. `md_codec::codex32::unwrap_string` is a pure BCH verify (no correction). `mk_codec::string_layer::decode_string` corrects and exposes `corrections_applied`; the existing `validate()` function already rejects `> 0`. Because `validate()` is called per-string BEFORE any reassembly, the ordering hole R0 identified is closed. The only edge subtlety is that for md1 chunks, the pristine check via `unwrap_string` operates on the full chunk string including its 37-bit chunk header prefix — which is correct since BCH covers the entire data-part payload, header bits included.

**Phase A self-containment:** Phase A has no Phase B dependency. All calls are to `md-codec` and `mk-codec` crates already in the dependency tree. The manifest schema (§A3) is forward-compatible for Phase B: the `sets[]` array with per-set `chunk_set_id`, `integrity` field, and `plates[]` is sufficient for a Phase-B consumer to select which plate to preview. No implicit coupling.

**Multi-set manifest (Q5 from the review charter):** The top-level `sets: [...]` in §A3 models an array of independent sets. A wallet backup with one md1 descriptor set and two mk1 key cards (each a separate chunk set) would produce `sets: [ {kind:"md1", chunk_set_id: "0xABCDE", plates:[...]}, {kind:"mk1-chunk", chunk_set_id:"0x12345", plates:[...]}, {kind:"mk1-chunk", chunk_set_id:"0x67890", plates:[...]} ]`. The grouping logic in §A2.4 handles each `chunk_set_id` as an independent group, so >1 independent sets are handled correctly. The manifest's `sets[]` array confirms this without ambiguity. No issue here.

**Exit-code consistency:** §A1 declares `0/2/3/4`. §A5 maps: empty input → 2 (usage), ms1 → 3, invalid/integrity failure → 4, success → 0. This is consistent with the converter. The "manifest emitted only if every set is complete, else exit 4" rule in §A5 is coherent: if any set has `ChunkSetIncomplete` or mismatched ids, the tool exits 4 without emitting the manifest. Incomplete sets can still be diagnosed from the stderr guidance.

**No new Important issues surfaced by the restructure.** The A/B split is clean. The Phase B stub is a non-binding record, not a dependency. The fold table at the bottom of the design correctly maps all 9 findings to their resolution locations.

---

## Verdict: GREEN (0 Critical / 0 Important)

The fold genuinely resolves all four R0 Important findings (I-1, I-2) with correct API composition, and correctly defers I-3/I-4 to Phase B without losing them. All five Minor findings from R0 are addressed. One new Minor finding (m-6) surfaces: the §A2.4 prose about "auto-dispatch" is imprecise — the spec must make explicit that a single unchunked md1 string must NOT be passed to `reassemble` (it would fail with `ChunkHeaderChunkedFlagMissing`), and instead should be BCH-verified via `unwrap_string` and treated as "bch-only" integrity. This is a Minor spec-clarity gap, not an architectural hole. Phase A is a sound, self-contained cycle. **GREEN — 0 Critical / 0 Important / 1 Minor (m-6, new).**

---

## Loop summary (added by main session)
| Round | Verdict | Folded |
|---|---|---|
| R0 | NOT-GREEN | 4 Important (I-1 md1 chunking, I-2 decode auto-corrects/pristine policy, I-3 QR params, I-4 sidecar version) + 5 Minor; + Rec-1 decompose (Phase A/B split, user-approved) + Rec-4 upstream pin |
| **R1** | **GREEN** | 1 new Minor (m-6 md1 single/chunked branching) → folded into §A2.4; converged |

Design converged GREEN. Next: present the GREEN Phase A design to the user for approval → write the Phase A spec → spec self-review → user review → spec R0 gate → plan.
