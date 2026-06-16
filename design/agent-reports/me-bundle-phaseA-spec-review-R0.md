# `me bundle` Phase A — SPEC architect review (R0 gate)

- **Stage:** mandatory spec R0 gate before any code (project standard). Spec promoted from the GREEN design.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (adversarial; verified all 7 load-bearing API claims in crate source).
- **Scope:** `design/SPEC_me_bundle_phaseA.md` vs `mk-codec 0.4.0`, `md-codec 0.36.0`, the GREEN design, and the existing converter.
- **Verdict:** **NOT-GREEN — 0 Critical / 2 Important / 5 Minor.** Both CRUX API claims (mk1 `chunk_set_id` extraction; md1 `BitReader`/`ChunkHeader::read`) CONFIRMED implementable.

---

## VERBATIM REVIEW OUTPUT

`SINGLE_STRING_LONG_BYTES` = 56. Now from the pipeline test comment: "smallest valid bytecode = 80 bytes > 56-byte single-string capacity" — so no real-world mk1 card will ever use `SingleString`. The spec's claim holds.

Now I have all the evidence. Let me compile the full review:

---

## API verification

**API Claim 1 — `mk_codec::decode(strings: &[&str]) -> Result<KeyCard>`**

CONFIRMED. `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/key_card.rs:115`: `pub fn decode(strings: &[&str]) -> Result<KeyCard>`, re-exported at `lib.rs:51` as `pub use key_card::{KeyCard, decode, encode, encode_with_chunk_set_id}`. Internally calls `crate::string_layer::decode(strings)`. The chunked reassembly in `string_layer/chunk.rs:reassemble_from_chunks` validates chunk-set-id consistency, index completeness, and the cross-chunk SHA-256 hash.

**API Claim 2 — `mk_codec::string_layer::decode_string` with `pub corrections_applied: usize` and `pub fn data(&self) -> &[u8]`**

CONFIRMED. `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/bch.rs:645`: `pub fn decode_string(s: &str) -> Result<DecodedString, crate::Error>`. `DecodedString` at line 567: `pub struct DecodedString` with `pub corrections_applied: usize` (line 571) and `pub fn data(&self) -> &[u8]` (line 591). `data()` returns the 5-bit symbol data part (checksum stripped). Re-exported at `string_layer/mod.rs:34`: `pub use bch::{..., DecodedString, ..., decode_string, ...}`.

**API Claim 3 — CRUX: `mk_codec::string_layer::header::StringLayerHeader::from_5bit_symbols(symbols: &[u8]) -> Result<(Self, usize)>` is public, and `Chunked` variant exposes `chunk_set_id`**

CONFIRMED. `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/header.rs:120`: `pub fn from_5bit_symbols(symbols: &[u8]) -> Result<(Self, usize)>`. The `Chunked` variant (line 46) has `pub chunk_set_id: u32`. `is_chunked()` at line 179 is also public. The module is exported `pub mod header` at `string_layer/mod.rs:27`.

The spec's assertion that "real mk1 strings are always chunked" is confirmed: `SINGLE_STRING_LONG_BYTES` = 56 bytes, but the smallest valid mk1 bytecode is ~80 bytes (1 stub + path indicator + fingerprint + 73-byte compact xpub), which forces all real-world mk1 cards into the `Chunked` path. The pipeline test at `pipeline.rs:168-175` documents this explicitly: "no realistic mk1 card fits in a single string." However, the spec should explicitly state that `SingleString` headers are rejected as an invalid input to `bundle` (since grouping by `chunk_set_id` is impossible for `SingleString` inputs), or handle them gracefully. Currently the spec says nothing about this edge case.

**API Claim 4 — mk1 error variants: `ChunkSetIdMismatch`, `ChunkedHeaderMalformed(..)`, `MixedHeaderTypes`, `CrossChunkHashMismatch`**

CONFIRMED. All four exist in `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/error.rs`:
- `ChunkSetIdMismatch` (line 75) — unit variant
- `ChunkedHeaderMalformed(String)` (line 81) — covers total_chunks=0, chunk_index>=total_chunks, gaps, duplicates
- `MixedHeaderTypes` (line 93) — raised by both `pipeline::decode` and `chunk::reassemble_from_chunks`
- `CrossChunkHashMismatch` (line 98) — unit variant

**API Claim 5 — `md_codec::codex32::unwrap_string(s) -> Result<(Vec<u8>, usize)>`**

CONFIRMED. `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/codex32.rs:113`: `pub fn unwrap_string(s: &str) -> Result<(Vec<u8>, usize), Error>`. Returns `(byte-padded payload bytes, symbol_aligned_bit_count)`. It is a pure BCH verify — `bch_verify_regular` called at line 144; any corruption returns `Err`. Module exported `pub mod codex32` in `lib.rs:22`.

**API Claim 6 — `md_codec::chunk::reassemble(strings: &[&str]) -> Result<Descriptor>` errors on a single unchunked md1**

CONFIRMED. `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/chunk.rs:305`: `pub fn reassemble(strings: &[&str]) -> Result<Descriptor, Error>`. Internally calls `ChunkHeader::read` on each unwrapped payload. For an unchunked md1 string, `ChunkHeader::read` reads the version (4 bits) and then the chunked flag (1 bit). If the chunked flag is 0, it returns `Err(Error::ChunkHeaderChunkedFlagMissing)`. The spec is correct: calling `reassemble` on a single unchunked md1 errors with `ChunkHeaderChunkedFlagMissing`. Also note: `reassemble` returns `Result<Descriptor, Error>` not `Result<Descriptor>` — the spec's shorthand omits the `Error` type but is otherwise correct.

**API Claim 7 — CRUX: `md_codec::chunk::ChunkHeader::read(&mut BitReader) -> Result<Self>` is public; `pub chunk_set_id: u32`; `BitReader::new`/`with_bit_limit` publicly constructible from `&[u8]`**

CONFIRMED. `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/chunk.rs:67`: `pub fn read(r: &mut BitReader) -> Result<Self, Error>`. `ChunkHeader` struct at line 21 has `pub chunk_set_id: u32` (line 25). `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/bitstream.rs:101`: `pub fn new(bytes: &'a [u8]) -> Self` and line 113: `pub fn with_bit_limit(bytes: &'a [u8], bit_limit: usize) -> Self`. `pub mod bitstream` is exported in `lib.rs:18`. The implementation path is:
1. `let (bytes, bit_count) = md_codec::codex32::unwrap_string(s)?;`
2. `let mut r = md_codec::bitstream::BitReader::with_bit_limit(&bytes, bit_count);`
3. `let header = md_codec::chunk::ChunkHeader::read(&mut r)?;`

The spec's md1 grouping design is IMPLEMENTABLE AS WRITTEN.

---

## Issues

### Critical

None.

### Important

**I-1: The spec silently assumes all mk1 inputs have `Chunked` headers; `SingleString`-headed mk1 inputs are an unhandled edge case**

Severity: Important. The spec §4 says "call `mk_codec::string_layer::header::StringLayerHeader::from_5bit_symbols(decoded.data())`; the chunked variant carries `chunk_set_id` (real mk1 strings are always chunked, even `total_chunks == 1`). `is_chunked()` confirms."

The spec does not specify what `bundle` does if `is_chunked()` returns false (i.e., a `SingleString`-headed mk1 input). In principle this is only reachable for artificially small hand-crafted mk1 strings (bytecode ≤56 bytes), not for any valid real-world mk1 card. However, the spec's `run_bundle(input: &str) -> Result<Manifest, BundleError>` is supposed to be safe for all inputs that survive the per-string `validate()` call. `validate()` calls `decode_string` which succeeds on a `SingleString`-headed mk1 (it's a valid BCH-verified string). A `SingleString` mk1 would then reach the `from_5bit_symbols` call, return `(StringLayerHeader::SingleString { version }, 2)`, fail the `is_chunked()` check, and the spec provides no branch for this.

The implementer will either panic, silently drop the string, or return an error. The spec must name the behavior: either reject it at step 4 with exit 4 (reasonable, since such a string has no `chunk_set_id` for grouping), or treat it as a degenerate set with integrity `bch-only` (analogous to unchunked md1). The omission means two implementers could make different choices.

Concrete fix: Add to §4 bullet "mk1": "A `SingleString`-headed mk1 (possible only for synthetically small bytecodes with ≤56 bytes, never for real-world cards) → exit 4 with message 'mk1 SingleString header: unsupported for bundle (no chunk_set_id)'". This closes the unspecified branch without complicating the manifest schema.

**I-2: The spec's md1 "chunked-flag detection" uses `ChunkHeader::read` over the unwrapped payload but the unchunked path also fails with `WireVersionMismatch` — the spec maps only `ChunkHeaderChunkedFlagMissing` as the unchunked indicator**

Severity: Important. The spec §4 says: "A `ChunkHeaderChunkedFlagMissing` / `WireVersionMismatch` result means an unchunked single-payload md1 (integrity = `bch-only`)."

Looking at `ChunkHeader::read` (chunk.rs:67-85):
1. It reads 4 bits as `version`. If `version != Header::WF_REDESIGN_VERSION` (=4), it returns `Err(Error::WireVersionMismatch { got: version })`.
2. Only if version=4 does it check the chunked flag.

For an unchunked md1 string in the v0.30 wire format: the first 5-bit symbol of the payload is `[v3][v2][v1][v0][chunked]` = `[0][1][0][0][0]` = version=4, chunked=0. So `WireVersionMismatch` would NOT be returned for a real unchunked v0.30 md1 — instead `ChunkHeaderChunkedFlagMissing` is returned (version=4 passes, chunked=0 triggers the flag check). The spec's enumeration of `WireVersionMismatch` as an unchunked-detection case is technically correct only for v0.x-era strings (version≠4), which the spec correctly handles as "unchunked" too. However, a `WireVersionMismatch` could also indicate a future wire-format version (version=5, say) with a valid chunked payload — treating that as `bch-only` would be incorrect. The spec conflates two distinct failure reasons.

Concrete fix: Replace the spec's §4 dispatch clause with: "`ChunkHeaderChunkedFlagMissing` → unchunked single-payload md1 (integrity = `bch-only`); `WireVersionMismatch` → unsupported md1 wire version → exit 4 with message 'unsupported md1 wire version'; any other error from `ChunkHeader::read` → exit 4." This correctly distinguishes "not chunked" (known-safe) from "unknown version" (should be rejected, not silently treated as unchunked).

### Minor

**m-1: The spec §4 says `mk_codec::decode(&group)` but the public function signature takes `&[&str]`, not a slice-of-Strings**

The spec's notation `mk_codec::decode(&group)` implies `group` is a `Vec<&str>` or `&[&str]`. If the implementer accumulates `Vec<String>` during grouping (likely, since strings are read from stdin), they must build a `Vec<&str>` referencing those strings before calling `decode`. The spec should note: "collect chunk strings as `Vec<String>`, then build `Vec<&str>` with `group.iter().map(|s| s.as_str()).collect::<Vec<&str>>()` before calling `mk_codec::decode`."

**m-2: Manifest schema §6 is ambiguous about which fields are present for a single unchunked md1 plate**

The spec says "`chunk_set_id`/`chunk_index` omitted for unchunked md1 and ms1" but the `sets[]` entry example shows `"chunk_set_id": "0x1a2b3"` for the md1 set. For an unchunked md1, there is no `chunk_set_id`. The `sets[]` schema needs to specify that for `integrity: "bch-only"` the `chunk_set_id` field is absent (or null). Likewise the `plates[]` entry for an unchunked md1 should show no `chunk_set_id` or `chunk_index`. The example JSON in §6 shows them for what appears to be a chunked md1, but the text is ambiguous.

**m-3: §10 test #7 (cross-chunk hash mismatch for mk1) — the spec's contingency is now unnecessary; there is a direct public construction path**

The pipeline tests in `string_layer/pipeline.rs` (the real codebase, not the spec) show exactly how to construct a valid test vector: encode with `encode_with_chunk_set_id`, then flip bytes in a `ChunkFragment.fragment` (which is public at `chunk.rs:30`) before passing through the BCH layer, OR perturb 5 characters in the encoded string (as the pipeline test does). Since `encode_with_chunk_set_id` is public and `ChunkFragment.fragment` is a public field that `reassemble_from_chunks` accepts, there is a direct path to construct a `CrossChunkHashMismatch` test vector without codec internals. The contingency documented in §10 test #7 is therefore overly cautious — the implementer should be directed to this construction path. Low severity since the contingency is still acceptable.

**m-4: §6 manifest `version` field normalization for the golden test (#12) — the spec correctly notes this but doesn't specify the normalization strategy**

The spec says "the §10 manifest golden test (#12) must either pin/normalize the `version` field or assert on a version-independent projection." It doesn't say which. A concrete recommendation would help: "normalize by replacing `version` with a fixed string `"x.x.x"` before JSON comparison, or use `serde_json::Value` projection that drops the `version` key." Minor since the spec acknowledges the problem.

**m-5: The spec §5 says `ChunkedHeaderMalformed(..)` covers "dropped chunk" — this is slightly imprecise**

A dropped chunk in mk1 is detected by `reassemble_from_chunks` at chunk.rs:131-136: `if chunks.len() != total_usize`, which returns `ChunkedHeaderMalformed("received N chunks, header declares total_chunks = M")`. This is correct. But a dropped chunk could also appear as a `missing chunk_index N` error at line 185. The spec says `ChunkedHeaderMalformed` covers "dropped chunk / `total_chunks` mismatch / duplicate `chunk_index`" — this is accurate since all these surface as `ChunkedHeaderMalformed` variants. Not a bug, just slightly imprecise grouping.

---

## Assessment

**Clap shape (§3):** The design — `Option<Command>` subcommand with existing flags retained at the top level — is valid clap 4 derive pattern. The existing `--in`, `--out`, `--stdout`, `--hex`, `--base64`, `--echo` flags remain on the top-level `Cli` struct; `#[command(subcommand)] cmd: Option<Commands>` is added alongside them. When the subcommand is `None`, the existing converter behavior fires; when `Some(Commands::Bundle {...})`, the new path fires. No conflict. `--manifest` and the bundle-specific `--stdout` override live inside the `Bundle` variant or as top-level args gated on the subcommand path. One implementer caution: the existing `--stdout` flag on the converter conflicts in name with the bundle path's stdout-output flag — the spec should clarify whether `--stdout` is reused on the bundle subcommand or bundle uses `--manifest` only, to avoid `conflicts_with_all` attribute confusion.

**Pristine ordering (§4):** The ordering is sound. `validate()` is called per-string BEFORE any reassembly call. For md1, `unwrap_string` is a pure BCH verify (no correction possible); for mk1, `decode_string` corrects up to 4 symbols, and `validate()` rejects `corrections_applied != 0`. Because `mk_codec::decode` internally calls `decode_string` per chunk again (confirmed at `pipeline.rs:128`), there is a harmless double-BCH-verify on mk1 chunks. This is acceptable (no correctness issue, minor performance cost). The pristine guarantee is solid.

**Manifest schema (§6):** Mostly complete. The `sets[]` / `plates[]` split models multi-set correctly (confirmed sound in design R1 Q5). The `integrity` tristate is unambiguous. The `wallet_plates` counting rule (public plates + 1 ms1 reminder) is clear. The `version` field normalization for testing is flagged. One ambiguity: for unchunked md1, the `sets[]` entry should explicitly show no `chunk_set_id` (see m-2 above). The phase-B forward-compatibility claim (plates array enables Phase B to select which plate to preview) is sound given the schema.

**Exit codes (§3/§8):** Consistent with the converter. `0/2/3/4` cleanly covers all enumerated edge cases. The "manifest emitted only if every set is complete" rule (→ exit 4 on any set failure) is stated in both §4 and §8, no contradiction. Early ms1 exit 3 before validation of other lines is correct.

**Test list (§10):** Covers all required failure modes. Tests #3 (dropped), #4 (reordered — correctly asserts it does NOT falsely fail), #5 (duplicate), #6 (foreign set-id) are all present. Test #7 (cross-chunk hash) contingency is now unnecessary given the direct construction path (see m-3). Test #8 (md1 chunked set) requires access to a `wsh_multi_chunked`-style vector — the spec should point to `md_codec::chunk::split` + `md_codec::chunk::reassemble` as the vector source. Test #9 (pristine policy) is covered. Test #12 (manifest golden) normalization is flagged. Coverage is adequate.

**Scope:** No Phase-B leakage. The spec stays cleanly Phase A. All Phase-A requirements from the GREEN design's §A are present. No over-build detected.

---

## Verdict: NOT-GREEN (0 Critical / 2 Important)

The spec is mostly implementable as written. Both CRUX API claims verified: the mk1 `chunk_set_id` extraction path via `StringLayerHeader::from_5bit_symbols` is public and correct; the md1 grouping path via `ChunkHeader::read` over a publicly-constructible `BitReader` from `unwrap_string` bytes is public and correct. The seven API claims are substantially confirmed.

Two Important issues prevent GREEN:

**I-1:** The spec does not specify behavior for a `SingleString`-headed mk1 input (is_chunked() = false). While unreachable for real-world cards, it is reachable through `validate()` and leaves the `bundle.rs` implementer with an unspecified branch. Fix: add one sentence to §4 mk1 bullet: reject with exit 4 naming "mk1 SingleString header: unsupported for bundle."

**I-2:** The spec maps `WireVersionMismatch` as equivalent to `ChunkHeaderChunkedFlagMissing` (both signal "unchunked single-payload md1, integrity = bch-only"). This is incorrect: `WireVersionMismatch` signals an unsupported wire version that the tool cannot safely classify, not a known-unchunked payload. It should be rejected with exit 4 as "unsupported md1 wire version." Fix: split the dispatch in §4 into three cases: `ChunkHeaderChunkedFlagMissing` → bch-only; `WireVersionMismatch` → exit 4; any other error → exit 4.

---

## Fold plan (main session) — ALL FOLDED into the spec
- **I-1** → §4 mk1 bullet: `SingleString` (is_chunked()==false) → exit 4.
- **I-2** → §4 md1 bullet: 4-way dispatch (Ok→chunked; ChunkHeaderChunkedFlagMissing→bch-only; WireVersionMismatch→exit 4; other→exit 4).
- **m-1** → §4 branching: build `Vec<&str>` from `Vec<String>` before `decode`/`reassemble`.
- **m-2** → §6: unchunked md1 omitted from `sets[]`, appears in `plates[]` as `bch-only` w/o chunk_set_id/index; example md1 clarified as chunked-of-1.
- **m-3** → §10 #7: direct construction path (swap a same-index chunk from a different set) instead of the contingency.
- **m-4** → §10 #12: normalize `version` (fixed token or drop key) before golden compare.
- **m-5** → §5 left as-is (architect: "not a bug, slightly imprecise"); §5 already enumerates the variants correctly.
- **clap `--stdout` collision** (assessment) → §3/§9: bundle has NO `--stdout`; manifest is stdout-by-default + `--manifest <file>`.

Re-dispatch R1 to converge.
