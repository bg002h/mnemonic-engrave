# `me bundle` (bundle orchestration + preview) — DESIGN architect review — R0

- **Stage:** pre-spec DESIGN review (R0), per the iterative-architect-review standard applied to the design before a spec is written (user-requested: "because this is big, ask architect for review now").
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (adversarial; verified all load-bearing claims against real code).
- **Scope:** `design/DESIGN_me_bundle_preview.md` vs the actual seedhammer fork (`6ab12c0`), upstream v1.4.2 ref, `mk-codec 0.4.0`, `md-codec 0.36.0`, and the existing converter.
- **Verdict:** **NOT-GREEN — 0 Critical / 4 Important / 5 Minor.**

---

## VERBATIM REVIEW OUTPUT

Now I have all the information I need to write a complete, authoritative review. Let me compile my findings.

Key findings summary:
1. **Faithfulness**: `validateMdmk` → `backup.EngraveText` CONFIRMED. Exact call path identified.
2. **md1 chunking**: `md-codec-0.36.0/src/chunk.rs` has a full `split()`/`reassemble()` API for chunked md1 — CONFIRMED, the design has a gap.
3. **mk_codec::decode integrity**: Fully confirmed. But `decode()` BCH-auto-corrects per-string — the design doesn't mention this issue.
4. **Go module dep**: `backup.EngraveText` is in upstream v1.4.2 — NO fork dependency for the sidecar.
5. **Command visibility**: `AsKnot()` and `AsDelay()` are exported methods on `Command`. The return type `splineKnot` is unexported BUT usable via Go's implicit interface — an external caller can call `cmd.AsKnot()` and use the result structurally. This is NOT a blocker.
6. **Scope**: Full scope in one cycle is risky given preview + release machinery.
7. **ms1 refusal**: The design places ms1 refusal after classification (before validation), consistent with the converter's ordering.
8. **`me bundle` reads multiple strings** from stdin (unlike the single-string converter) — the design mentions this but the refusal logic needs to fire per-line, not once for the whole input.

---

## Verified Facts

### Question 1: Faithfulness crux — does `mdmkFlow`/`validateMdmk` use `backup.EngraveText`?

CONFIRMED. The exact call path in `/scratch/code/shibboleth/seedhammer/gui/gui.go`:

- `validateMdmk` (line 1746) calls `qr.Encode(s, qr.L)` for QR at level L, `const qrScale = 3`, and builds three `textEngraving` structs: `{Text: s, QR: qrc, QRScale: qrScale}`, `{Text: s}`, `{QR: qrc, QRScale: qrScale}`.
- Each is wrapped as `backup.Text{Paragraphs: []backup.Paragraph{e.Paragraph}, Font: sh.Font}`.
- `backup.EngraveText(params, plate)` is called at line 1769. `toPlate` checks it fits; if none fit, error.
- `mdmkFlow` (line 1786) calls `validateMdmk(ctx.Platform.EngraverParams(), string(s))` and presents the valid variants via `ChoiceScreen`.

The design's claim that the sidecar should reuse `backup.EngraveText` is CONFIRMED as the correct entry point. The sidecar must also use `qr.Encode(s, qr.L)` (level L, not M) and `qrScale = 3` — the design mentions "TEXT+QR / TEXT / QR-only" but does not yet specify the QR error-correction level. This is a needed detail.

The `sh.Font` used in `validateMdmk` is the `seedhammer.com/font/sh` package, present in both the upstream v1.4.2 and the fork. The sidecar can import it directly.

The design claim "reuses `backup.EngraveText`" is CONFIRMED as architecturally correct.

### Question 2: md1 chunking gap

CONFIRMED — md1 CAN produce multi-string chunked output. The evidence:

`/home/bcg/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/md-codec-0.36.0/src/chunk.rs` exposes a public `split(d: &Descriptor) -> Result<Vec<String>, Error>` function (line 235) that splits a descriptor into multiple `md1` codex32 strings when the payload exceeds `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64 × 5 = 320 bits`. The format has a `ChunkHeader` with `chunk_set_id` (20-bit), `count` (1..=64), and `index` (0-based). A `reassemble(strings: &[&str])` function (line 305) does cross-chunk integrity via a derived `chunk_set_id` from `compute_md1_encoding_id` — different mechanism from mk1's trailing hash but equally real.

The test vectors confirm chunked md1 exists: `tests/vectors/wsh_multi_chunked.*` is a named test vector set. The design's phrase "the design currently treats md1 as a single policy plate" acknowledges this as an open question, and the answer is: **md1 can chunk and the design has a gap**.

Design claim "md1 = single policy plate" is REFUTED for large descriptors. NEEDS-CHANGE.

### Question 3: `mk_codec::decode` integrity guarantees

CONFIRMED with full evidence from the source.

`mk_codec::decode` → `string_layer::decode` (pipeline.rs lines 118-151) → `reassemble_from_chunks` (chunk.rs lines 109-202) validates:

- **Dropped chunk**: `chunks.len() != total_usize` → `Error::ChunkedHeaderMalformed("received N chunks, header declares total_chunks = M")` (chunk.rs line 131).
- **Duplicate chunk_index**: `slots[idx].is_some()` → `Error::ChunkedHeaderMalformed("duplicate chunk_index N")` (chunk.rs line 163).
- **Reordered chunks**: handled by slot-indexing — chunks placed into `slots[chunk_index]`; the final `slots.iter().enumerate()` loop catches missing slots (chunk.rs line 184-186).
- **Foreign `chunk_set_id`**: `chunk_set_id != set_id` → `Error::ChunkSetIdMismatch` (chunk.rs line 150).
- **Cross-chunk hash**: `SHA-256(bytecode)[0..4]` verified at reassembly (chunk.rs lines 189-201) → `Error::CrossChunkHashMismatch`.
- **Mixed header types**: `Error::MixedHeaderTypes`.

Edge case for single-chunk: the `pipeline.rs` encoder emits `SingleString` only if `bytecode.len() <= SINGLE_STRING_LONG_BYTES = 56`. The comment at line 156 states "smallest valid bytecode = 80 bytes > 56-byte single-string capacity, so v0.1 encoders never emit SingleString." In practice, all real mk1 cards use `Chunked` headers (even if total_chunks=1). For a `Chunked(total=1)` input, `reassemble_from_chunks` with one chunk still appends the `cross_chunk_hash` and verifies it (chunk.rs lines 189-201). So the cross-chunk hash IS present and verified even for single-chunk encodings.

Critical nuance: `decode()` calls `decode_string()` per chunk (pipeline.rs line 128), which BCH-auto-corrects up to 4 errors. The design says `me bundle` runs `mk_codec::decode` to prove "complete & consistent," but the design does NOT specify that `me bundle` must additionally check `corrections_applied == 0` (as the per-string converter does). If the design intends pristine-only input for `me bundle`, the spec must explicitly require checking `DecodedString::corrections_applied` on each chunk. Design claim on this is NEEDS-CHANGE (important detail missing).

### Question 4: Go module dep target — fork or upstream?

`backup.EngraveText` (with `backup.Text`, `backup.Paragraph`, `QRScale` field) is present in the **upstream** v1.4.2 at `/scratch/code/shibboleth/seedhammer-ref-v1.4.2/backup/backup.go` lines 32-41 and 252. The `font/sh` package is also present in upstream v1.4.2. The sidecar uses neither `gui/gui.go` nor any PR #35-specific code.

CONFIRMED: the preview sidecar's `go.mod` can pin the **upstream** seedhammer (not the fork), avoiding any dependency on the unmerged PR #35. The existing ndef-roundtrip harness already demonstrates this pattern using `replace seedhammer.com => ../../../seedhammer-ref-v1.4.2`.

Design claim "require-pinned to a seedhammer commit (upstream once #35 merges; fork meanwhile)" is WRONG — the sidecar can already pin upstream v1.4.2 today. NEEDS-CHANGE.

### Question 5: Sidecar rendering feasibility — Command API visibility

The `Command` struct in `engrave/engrave.go` has all-lowercase (private) fields `kind` and `args`. However:

- `Command.AsKnot()` (line 87) is an **exported** method returning an unexported type `splineKnot`. In Go, an exported method on a type can return an unexported type — the caller receives the value via short variable declaration (`k, ok := cmd.AsKnot()`) and can access the exported fields `k.Engrave` (bool), `k.Knot` (bezier.Point with exported X, Y), `k.Multiplicity` (int). This is fully usable from an external package; the caller simply cannot declare a variable as `var k engrave.splineKnot` by name.
- `Command.AsDelay()` (line 78) returns `(denom, nom uint, ok bool)` — fully exported.
- The constructor functions `Move()`, `Line()`, `ControlPoint()`, `Delay()` are all exported.

Therefore: an external Go sidecar CAN iterate over an `engrave.Engraving` (which is `iter.Seq[Command]`), call `cmd.AsKnot()` on each, and branch on `k.Engrave` to distinguish pen-down engraving strokes (lineCmd = `Engrave: true`) from pen-up travel (moveCmd = `Engrave: false`). The `k.Knot.X` and `k.Knot.Y` coordinates are accessible.

**However**, `ControlPoint` commands (from font b-spline rendering, line 1380 of engrave.go) produce `AsKnot()` with `Multiplicity != 3`. A naïve SVG renderer that treats every `AsKnot()` result as a line endpoint would mis-render font curves. The sidecar must handle B-spline control points correctly (multiplicity 1 = knot, 3 = clamping knot). This is a known complexity but not a blocker — the `PlanEngraving` function can be used to convert the raw command stream to a `bspline.Curve`, though that adds stepper-physics parameters. For a visual SVG preview (not a toolpath plan), the sidecar may simplify by treating all knots as line segments, which would approximate but not pixel-perfectly replicate the curves.

CONFIRMED: no package-private blocker. The `AsKnot()` / `AsDelay()` API is sufficient for external rendering. B-spline rendering requires care but is not a blocking visibility issue.

### Question 6: Scope/decomposition

Full scope (manifest + workflow + integrity + preview sidecar + cross-platform signed release CI) in one cycle introduces significant implementation risk for a pre-spec stage. Specifically:

- The Rust-only manifest+integrity+workflow component is well-bounded (~300 LOC, depends only on existing `mk-codec` and `md-codec` deps).
- The preview sidecar is independent, adds a Go build system, cross-platform binary distribution, signing infrastructure, and a new Go module.
- Release CI with signed archives is a substantial operational deliverable.

These are functionally independent subsystems with different risk profiles. The project standard (brainstorm→spec→plan→R0→implement per phase) is designed for bounded cycles.

---

## Issues

### Critical

**None.**

### Important

**I-1: md1 chunking gap — design omits chunked md1 entirely.**

The design (§3) states "the design currently treats md1 as a single policy plate." The md-codec-0.36.0 source confirms this is wrong for large descriptors: `md-codec` has a public `split(d: &Descriptor) -> Result<Vec<String>, Error>` (chunk.rs line 235) and `reassemble(strings: &[&str]) -> Result<Descriptor, Error>` (line 305), with a named test vector `wsh_multi_chunked` confirming the feature ships. A `wsh(sortedmulti(k, ...))` policy with many cosigners can exceed the single-string limit of 64 × 5 = 320 bits.

If `me bundle` receives a chunked md1 input — multiple strings with `md1` HRP — and the design only groups `mk1` by `chunk_set_id`, the md1 strings will be treated as separate single-string plates with no set-completeness check. This produces a silent gap: the tool will emit a manifest claiming "plate 1/N md1 policy" for each chunk without verifying the set is complete or that the `chunk_set_id` values match.

The md1 cross-chunk integrity mechanism also differs from mk1: md1 derives its `chunk_set_id` deterministically from `compute_md1_encoding_id(descriptor)` (chunk.rs line 244), not a CSPRNG. Integrity is checked at reassembly by re-deriving the ID and comparing (chunk.rs lines 379-386). The spec must address: detect md1 chunking by reading the first 5-bit symbol's bit 0 (the chunked-flag per `lib.rs` "decoder auto-dispatch"), group by `chunk_set_id`, and call `md_codec::chunk::reassemble()` for set-completeness proof.

**Fix**: The spec must include a md1 chunk-grouping + integrity path parallel to the mk1 path, using `md_codec::chunk::reassemble()` (or at minimum detecting and refusing partial chunked md1 sets).

---

**I-2: `mk_codec::decode` auto-corrects — pristine-input policy not specified.**

The design (§3) says `me bundle` runs `mk_codec::decode` to prove completeness and consistency. But `mk_codec::decode` calls `decode_string()` internally (pipeline.rs line 128), which BCH-corrects up to 4 substitution errors per chunk silently. A user who supplies a corrupted mk1 string (1–4 errors) will get a "verified" integrity result from `decode`, even though the input is not pristine.

The existing per-string converter explicitly rejects corrected inputs (`corrections_applied != 0` → `ValidateError::MkCorrected`, validate.rs line 48). The design gives no equivalent policy for `me bundle`'s integrity check. Since `me bundle` does NOT engrave (it only validates and produces a manifest), the risk is different: a user who runs `me bundle` with a slightly corrupted mk1 might see "integrity: verified" even though the string they intend to engrave is corrupt.

The spec must decide: (a) require pristine inputs to `me bundle` (check `DecodedString::corrections_applied == 0` for each chunk), or (b) explicitly allow auto-correction and surface a warning per corrected symbol. Option (a) is consistent with the existing converter and the "engrave verbatim" principle.

**Fix**: Spec must state explicitly whether `me bundle` enforces pristine mk1 chunk inputs. Given the project's pattern, option (a) is the right default.

---

**I-3: Design does not specify QR encoding parameters for the sidecar.**

The design (§5) says the sidecar renders "the same mode the firmware picks (TEXT+QR / TEXT / QR-ONLY via `backup.EngraveText`)." The `validateMdmk` function uses `qr.Encode(s, qr.L)` (error-correction level L) and `qrScale = 3` (gui.go lines 1747, 1751). The design does not pin these parameters. If the sidecar uses level M (as `EngraveSeedString` does, backup.go line 78) or a different scale, the preview QR will differ from the actual engraved QR (different size, different module density). A user who scans the preview QR expecting it to match the plate would get a different QR.

**Fix**: The spec must explicitly document that the sidecar uses `qr.Encode(s, qr.L)` and `qrScale = 3`, matching `validateMdmk`'s parameters exactly.

---

**I-4: Sidecar-to-crate version binding is underspecified — stale sidecar risk.**

The design (§6, §5, §8 open question 5) proposes: `me` locates `me-preview` "next to itself or on `$PATH`" with "sidecar version pinned to crate version" enforced via checksums in the archive. The archive checksum guards against corruption but does NOT prevent a user from installing a mismatched `me-preview` separately (e.g., from a previous release on `$PATH`).

If the firmware's layout algorithm changes (e.g., margin parameters, QR scale, font) and the sidecar is rebuilt with a new seedhammer pin, an old `me-preview` on PATH would silently render using old layout — the preview would differ from the current device behavior without any error. The design gives no mechanism for `me` to verify the sidecar's own version at runtime before invoking it.

**Fix**: The spec must require `me-preview` to emit a machine-readable version identifier (e.g., via `me-preview --version` returning a semver or commit hash) which `me` checks before any preview invocation. Mismatched version → warning or error, not silent fallback to a stale sidecar.

---

### Minor

**m-1: `ms1` refusal ordering in multi-string context.**

The design (§2) says "ms1 on any line → refused, exit 3." The existing converter refuses ms1 before validation (lib.rs lines 56-58: classify → check `fmt == Format::Ms` → refuse, never reaching validate). For `me bundle`, reading multiple lines, the refusal must still fire before any validation of any string. The design does not state whether processing stops at the first ms1 line or scans all lines first. Stopping early is correct (consistent with the converter). Minor issue since the design signals correct intent; the spec should pin early-exit semantics.

**m-2: Multiple md1 strings of different chunk_set_ids.**

The design (§2) says "groups mk1 chunks by chunk_set_id (supports >1 mk1 set)." It does not specify behavior for: two md1 strings that are actually from different encodings (different descriptors), or two md1 strings with the same chunk_set_id but wrong count. The spec should state: any two md1 strings with mismatched HRP-derived chunk_set_ids are treated as separate (invalid) partial sets and reported as such.

**m-3: Manifest `integrity` field semantics for unchunked md1.**

The manifest schema (§4) shows `integrity: "verified"|"n/a"`. For a non-chunked md1 (single string with chunked-flag=0), individual BCH verification passes at the validate stage; there is no cross-chunk hash. The field should be `"bch-only"` or the design should clarify that "n/a" means "per-string BCH passed but no cross-chunk hash exists." This is a schema clarity issue.

**m-4: Input model — empty input, all-ms1 input, duplicate strings.**

The design does not explicitly handle: (a) empty stdin → what exit code? (b) all lines are ms1 → exit 3, but does the tool refuse and stop at first ms1 or scan all? (c) exact duplicate mk1 strings (same string twice) → the chunk.rs duplicate detection (line 163) would catch same `chunk_index`, but two identical strings with the same index from a user error. The spec should enumerate these edge cases explicitly.

**m-5: `Engraving` B-spline rendering in SVG — multiplicity handling.**

The sidecar, when generating SVG from the `Command` stream, must handle `ControlPoint` commands (which `AsKnot()` returns with `Multiplicity = 1`) correctly to render vector font strokes. A linear-segments-only approximation would produce subtly wrong letter shapes. The spec should acknowledge whether the SVG preview is "approximate" (linear segments from knots) or "exact" (full B-spline interpolation), and document the chosen approach. This affects user trust ("faithful" vs "approximate" preview).

---

## Recommendations

**1. Decompose into Phase A and Phase B.**

Phase A (Rust-only, one bounded cycle): manifest + guided workflow checklist + mk1 chunk-set integrity (`mk_codec::decode`) + md1 chunk grouping + integrity (`md_codec::chunk::reassemble`). This delivers the primary safety value (dropped-chunk detection) with no Go interop. Estimated ~300–500 LOC, all existing deps, testable in pure Rust.

Phase B (separate cycle): preview sidecar (`me-preview` Go binary) + bundled release archive + cross-platform CI + signed checksums. Preview is a UX enhancement, not a safety feature. Deferring it lets Phase A ship sooner and lets the PR #35 upstream story develop before the sidecar dep is frozen.

The recon document (`cycle-prep-recon-me-bundle-preview-layer.md`) already recommended this decomposition at lines 34-39: "Rough sizing: manifest + workflow + set-integrity ≈ a few hundred LOC of Rust + tests (uses `mk-codec::decode` already a dep); preview adds a Go sidecar."

**2. Spec must address md1 chunking explicitly.**

Use `md_codec::chunk::reassemble()` for multi-string md1 inputs. Detect single vs. chunked md1 via the first 5-bit symbol's bit 0 (as documented in `md-codec/src/lib.rs` "decoder auto-dispatch"). Group by chunk_set_id derived from `derive_chunk_set_id`. This parallels the mk1 path.

**3. Pristine-input policy.**

The spec should explicitly require `DecodedString::corrections_applied == 0` for all mk1 chunks passed to `me bundle`, returning exit 4 with the correction details. This is consistent with the converter's existing pattern and prevents a "verified" result on a string the user would engrave in corrupted form.

**4. Sidecar go.mod can target upstream NOW.**

The sidecar's `go.mod` should target upstream v1.4.2 today, not the fork. `backup.EngraveText`, `backup.Text`, `backup.Paragraph`, `font/sh`, `engrave.Params`, and the full rendering stack are all present in upstream. The sidecar does NOT import `gui/gui.go` at all — it calls `backup.EngraveText` directly, replicating the `validateMdmk` layout logic without pulling in the GUI package. No PR #35 dependency.

**5. Pin QR parameters in the spec.**

Spec must state: `qr.Encode(s, qr.L)` for the QR payload, `qrScale = 3`, to match `validateMdmk`'s exact parameters. This ensures the preview QR and the engraved QR are identical.

**6. Runtime sidecar version check.**

`me-preview --version` must return a parseable version or commit hash. `me` must verify it before invoking preview. Mismatched version emits a clear warning; missing `me-preview` (with `--preview`) degrades gracefully. This should be in the spec as a first-class requirement, not deferred.

---

## Verdict: NOT-GREEN

**Critical: 0 | Important: 4 | Minor: 5**

The design has 4 Important issues that must be resolved before a spec is written:

- I-1: md1 chunking completely absent from the design (real md1 cards can chunk; the design's "single policy plate" assumption is wrong for large descriptors).
- I-2: `mk_codec::decode` auto-corrects — pristine-input policy not specified, leaving a silent gap vs. the converter's behavior.
- I-3: QR encoding parameters (level L, scale 3) not pinned in the sidecar spec — faithfulness gap.
- I-4: Sidecar version binding underspecified — stale sidecar silently renders wrong preview.

Additionally, scope decomposition (Phase A: Rust-only integrity + manifest; Phase B: preview + release CI) is strongly recommended before writing the spec.

---

## Fold plan (main session)
- **I-1 / I-2 / m-1..m-4** belong to the integrity+manifest core (Phase A regardless of decomposition) → fold into the design now.
- **I-3 / I-4 / m-5** are preview-sidecar concerns (Phase B) → fold into the preview section.
- **Decomposition (Rec 1)** = a sequencing decision surfaced to the user before folding/re-dispatch (the user chose FULL scope; splitting into A→B sequences it without dropping anything).
- Correct the WRONG draft claim (Rec 4): sidecar pins UPSTREAM v1.4.2 now, not the fork.
- Re-dispatch R1 after folding to converge to GREEN.
