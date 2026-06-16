# Design — `me bundle` (bundle orchestration), Phase A + Phase B

- **Status:** Draft design (brainstorming output), pre-spec. R0 design review done (`agent-reports/me-bundle-preview-design-review-R0.md`, NOT-GREEN 4I/5m); findings FOLDED here; awaiting R1 convergence.
- **Date:** 2026-06-16
- **Provenance:** FOLLOWUP `me-bundle-preview-layer`; recon `cycle-prep-recon-me-bundle-preview-layer.md` (me-repo `5e69e70`, seedhammer `6ab12c0`). Verified vs `mk-codec 0.4.0`, `md-codec 0.36.0`, seedhammer fork + upstream v1.4.2.
- **User decisions (brainstorm Q&A):** FULL scope (manifest + workflow + integrity + faithful preview), preview as a **prebuilt sidecar** in a **bundled signed per-platform release archive** (NO runtime network), delivered as **two sequenced cycles: Phase A then Phase B**.
- **SemVer:** Phase A (`me bundle` subcommand) ⇒ MINOR, v0.1.x → **v0.2.0**. Phase B (preview) ⇒ MINOR, → **v0.3.0**.

---

# PHASE A — pure-Rust `me bundle` (THIS cycle)

The safety-bearing core: enumerate a wallet backup's plates, prove each chunk **set** is complete & consistent, emit a manifest + guided workflow. No Go interop. This is what gets a spec→R0→plan→R0→implement pass now.

## A1. CLI surface
- New subcommand: **`me bundle`**. Existing `me` (single-string→NDEF converter) is unchanged.
- Input: **newline-separated PUBLIC strings** from stdin (default) or `--in <file>`. md1/mk1 are public ⇒ acceptable; argv is still never used for string content.
- Output: manifest JSON to `--manifest <file>` or stdout (`--stdout`); the guided checklist + all human guidance to **stderr** (same stdout/stderr discipline as the converter).
- Exit codes (consistent with the converter): `0` ok, `2` usage, `3` ms1-refused, `4` invalid/integrity-failure.

## A2. Pipeline (per-string pristine FIRST, then set-integrity)
1. **Read & split** input into non-empty trimmed lines. Empty input (no strings) → usage error, exit 2.
2. **Classify each line** by HRP (`classify.rs`, reused). On the **first** `ms1` line → refuse immediately, exit 3, with the converter's RF-risk message — **before validating any line** (early-exit; resolves m-1). The tool never ingests the secret.
3. **Per-string pristine validation** (reuse the converter's `validate.rs`): md1 via `md_codec::codex32::unwrap_string` (pure verify, no correction); mk1 via `mk_codec::…decode_string` rejecting `corrections_applied != 0`. Any corrected/invalid string → exit 4 naming it. This makes pristine-ness hold **before** any reassembly (resolves I-2 — no "verified" result on a silently BCH-corrected string).
4. **Group & set-integrity:**
   - **mk1:** group by `chunk_set_id`; run `mk_codec::decode` over each group → proves complete & consistent (`ChunkSetIdMismatch`, malformed-header/dropped, duplicate `chunk_index`, reorder via slot-indexing, `CrossChunkHashMismatch`). Each group = one key card = an ordered plate run.
   - **md1:** after the §A2.3 pristine `unwrap_string` verify, read **bit 0 of the first 5-bit symbol** of the unwrapped payload = the chunked-flag, then branch (resolves **I-1** + **m-6**; md1 *can* chunk: `md-codec` `split()`/`reassemble()`, `wsh_multi_chunked` vector):
     - single string, flag = 0 → **unchunked single-payload**: integrity = `"bch-only"`. **Do NOT call `reassemble`** (it errors `ChunkHeaderChunkedFlagMissing` on an unchunked string).
     - single string, flag = 1 → **chunked-of-1 set** → `md_codec::chunk::reassemble(&[s])`.
     - multiple md1 strings → group by the (deterministic, descriptor-derived) `chunk_set_id` and `md_codec::chunk::reassemble` each group for set-completeness.
     - NB: the bit-0 "auto-dispatch" shorthand lives only in `md_codec::chunk::decode_with_correction` (the BCH-*correcting* path). The pristine path composes `unwrap_string` (verify) + manual flag inspection + `reassemble` (set-integrity) explicitly.
   - Two strings of the same HRP with **mismatched** `chunk_set_id` → reported as **separate sets** (and each flagged incomplete if partial) rather than merged (resolves m-2).
5. **Build manifest + checklist**, exit 0.

## A3. Manifest schema (JSON)
Ordered plate list; each plate:
```json
{ "plate": 1, "of": 3, "kind": "md1" | "mk1-chunk" | "ms1",
  "string": "md1...",            // omitted for ms1 (never known to the tool)
  "chunk_index": 0, "chunk_set_id": "0x…",   // omitted for unchunked/ms1
  "integrity": "set-verified" | "bch-only" | "n/a" }
```
- `set-verified` = part of a chunked set proven complete/consistent. `bch-only` = single unchunked string, per-string BCH passed, no cross-chunk hash exists (resolves m-3). `n/a` = the `ms1` plate (type-on-device placeholder, no content).
- Top-level: `{ "wallet_plates": N, "sets": [...], "ms1_required": true, "plates": [...] }`.

## A4. Guided workflow (stderr)
`plate 1/N md1 policy → push via NFC & engrave; plate 2/N mk1 1/2 → NFC & engrave; … ; ms1 secret → TYPE ON DEVICE (never via this tool).`

## A5. Error handling / edge cases (resolves m-4)
- Empty input → exit 2. All-ms1 → exit 3 at the first ms1 line. Exact-duplicate chunk (same `chunk_index` twice) → caught by `mk_codec`/`md_codec` reassembly dup-detection → exit 4. Incomplete set (missing chunk) → exit 4 with the specific codec reason. Corrupted (BCH-correctable) string → exit 4 (pristine policy, step 3). Mixed valid md1 + mk1 + partial set → manifest emitted only if every set is complete; otherwise exit 4 listing the incomplete set(s).

## A6. Files / tests
- New `crates/me-cli/src/bundle.rs` (+ `manifest.rs`); reuse `classify`/`validate`; add `me bundle` to the clap `Cli`.
- Tests: complete md1-set + mk1-set passes; each of {dropped, reordered, duplicate, foreign-set-id, corrupted} fails with the right error; ms1-refusal early-exit; empty input; manifest golden; exit codes. Pure Rust (no `go`).

---

# PHASE B — `me-preview` faithful plate preview (DEFERRED, own cycle → v0.3.0)

Captured now so the R0 findings aren't lost; gets its own brainstorm→spec→plan→R0 when Phase A ships. UX enhancement, not a safety feature.

## B1. Sidecar & trust split
- `me-preview` (Go) renders ONLY: validated public string + plate mode → `engrave.Engraving` → image. `me` (Rust) does all validation. Sidecar has no secrets, no network.
- **go.mod pins UPSTREAM seedhammer v1.4.2** — `backup.EngraveText`/`backup.Text`/`backup.Paragraph`/`font/sh`/`engrave.Params` are all pre-existing upstream; the sidecar imports `backup`+`engrave` directly (NOT `gui`), so it is **NOT blocked on PR #35** (corrects the draft's "fork meanwhile" claim; resolves Rec 4). Mirrors the `firmware/ndef-roundtrip/` replace pattern.

## B2. Faithfulness contract (resolves I-3, m-5)
- The sidecar must replicate `validateMdmk`'s exact layout: `backup.EngraveText`, QR via **`qr.Encode(s, qr.L)`** (error-correction level **L**, not M), **`qrScale = 3`**, modes TEXT+QR / TEXT / QR-only. Any deviation makes the preview QR differ from the engraved QR.
- SVG primary (walk the `Command` stream via exported `AsKnot()`/`AsDelay()`; `k.Engrave` distinguishes pen-down strokes from travel; `k.Knot.X/Y` are accessible). **B-spline `ControlPoint` knots (multiplicity ≠ 3) must be interpolated, not drawn as line segments**, or fonts mis-render. The spec must declare the fidelity target (exact B-spline vs. documented-approximate). Optional `--png`.

## B3. Delivery & version binding (resolves I-4)
- Bundled per-platform release archive (`me` + `me-preview` + `SHA256SUMS` + signature); cross-platform CI matrix. No runtime network.
- `me` locates `me-preview` beside itself / on `$PATH`; **before invoking it, `me` checks `me-preview --version`** against the expected pin. Mismatch → clear warning/refusal (never a silent stale-layout render). Absent with `--preview` → graceful degrade (manifest+checklist still emitted).

---

## Fold status vs R0
- **I-1** md1 chunking → **A2.4 / A3** (md1 grouping + `md_codec::chunk::reassemble`, `bch-only` for unchunked). FOLDED.
- **I-2** pristine policy → **A2.3** (per-string pristine validation before reassembly, reusing converter). FOLDED.
- **I-3** QR params → **B2** (`qr.L`, `qrScale=3`). FOLDED (Phase B).
- **I-4** sidecar version binding → **B3** (`me-preview --version` check). FOLDED (Phase B).
- **m-1** ms1 early-exit → A2.2. **m-2** mismatched set-ids → A2.4. **m-3** integrity field → A3. **m-4** edge cases → A5. **m-5** B-spline fidelity → B2. ALL FOLDED.
- **Rec 1** decompose → DONE (Phase A/B split, user-approved). **Rec 4** upstream pin → B1.
- **m-6** (R1-new) md1 single/chunked branching → **A2.4** (explicit: unchunked single ≠ `reassemble`). FOLDED.

**Review status:** design R0 NOT-GREEN (4I/5m) → R1 **GREEN** (0C/0I; 1 new minor m-6, folded). Loop converged (`agent-reports/me-bundle-preview-design-review-R{0,1}.md`).
