# `me bundle` Phase A — PLAN architect review (R0 gate)

- **Stage:** mandatory plan R0 gate before any code (project standard). Plan written via writing-plans, self-reviewed.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-architect` (adversarial; verified every code block's APIs in pinned-crate source).
- **Scope:** `design/IMPLEMENTATION_PLAN_me_bundle_phaseA.md` vs `mk-codec 0.4.0`, `md-codec 0.36.0`, the GREEN spec, the existing converter.
- **Verdict:** **NOT-GREEN — 0 Critical / 2 Important / 3 Minor.** All 8 API claims verified in source; 2 blockers (one compile, one TDD-vector) folded.

---

## VERBATIM REVIEW OUTPUT

Now I have all the information I need for a thorough review. Let me compile the complete analysis.

## API verification

### Item 1: mk1 header extraction (Task 5)
CONFIRMED (`mk-codec-0.4.0/src/string_layer/header.rs:120`, `src/string_layer/mod.rs:35`). `decode_string(s).data() -> &[u8]`; `StringLayerHeader::from_5bit_symbols(&[u8]) -> Result<(StringLayerHeader, usize)>` public; `Chunked { version, chunk_set_id, total_chunks, chunk_index }` (`header.rs:45-57`) and `SingleString { version }` (`header.rs:38-41`) confirmed. **`StringLayerHeader` is `#[non_exhaustive]` (`header.rs:33`)** → external exhaustive match without `_` fails to compile (E0004). NEEDS-CHANGE (Important #1).

### Item 2: md1 header extraction (Task 5)
CONFIRMED. `md_codec::codex32::unwrap_string(s) -> Result<(Vec<u8>, usize), Error>` (`codex32.rs:113`); `md_codec::bitstream::BitReader::with_bit_limit(&[u8], usize)` (`bitstream.rs:113`); `md_codec::chunk::ChunkHeader::read(&mut BitReader) -> Result<Self, Error>` (`chunk.rs:67`) with public `chunk_set_id/count/index` (`chunk.rs:21-29`). CRUX confirmed: `Error::ChunkHeaderChunkedFlagMissing` (`error.rs:236`), `Error::WireVersionMismatch { got: u8 }` (`error.rs:34-37`). `md_codec::Error` is NOT `#[non_exhaustive]`, so the 3-arm + `Err(e)` catch-all match is valid.

### Item 3: Set-integrity (Task 6)
CONFIRMED. `mk_codec::decode(&[&str]) -> Result<KeyCard>` (`key_card.rs:116`, re-export `lib.rs:51`); `md_codec::chunk::reassemble(&[&str]) -> Result<Descriptor, Error>` (`chunk.rs:305`, re-export `lib.rs:43`). Both take `&[&str]` and return their crate Error; `.map_err(BundleError::SetIncomplete*)` type-checks.

### Item 4: Error wrapping (Task 5)
CONFIRMED. `ValidateError::Mk(mk_codec::Error)` (`validate.rs:12`), `ValidateError::Md(md_codec::Error)` (`validate.rs:10`); `from_5bit_symbols` error is `mk_codec::Error`; `unwrap_string` error is `md_codec::Error`.

### Item 5: Serde (Task 2)
CONFIRMED. `rename_all="lowercase"` → md1/mk1; per-variant renames → spec strings; `skip_serializing_if="Option::is_none"` omits absent chunk fields. Matches the Task-2 tests.

### Item 6: clap optional subcommand (Task 8)
CONFIRMED — no compile issue. `--in`/`--manifest` are scoped inside the `Bundle` variant; top-level flags remain optional. `me --hex` (None subcommand) → existing `run()`; `me bundle` → `run_bundle_cli`. Dead top-level flags in the bundle path are accepted by clap and ignored by the driver (acceptable).

### Item 7: `foreign_index1_same_setid` (Task 7)
CONFIRMED. The two V2_bip84 strings match `v0.1.json:35-36` and decode to a 2-chunk card. Pairing V1's index-0 (set 0x12345) with V2-re-encoded-at-0x12345's index-1: chunk_set_id + index-coverage checks pass, then the cross-chunk hash over the spliced bytecode mismatches → `CrossChunkHashMismatch`. `mk_codec::decode` REJECTS the combination; the test asserts `SetIncompleteMk(..)` (any decode error) → MATCHES.

### Item 8: `chunked_md1_vector` (Task 7)
CONFIRMED public path exists, but the plan left it ambiguous. `md_codec::chunk::split(&Descriptor) -> Result<Vec<String>, Error>` is public (`chunk.rs:235`); a `Descriptor` is buildable from public types (all `Descriptor` fields pub, and `md_codec` re-exports `OriginPath/PathComponent/PathDecl/PathDeclPaths/Tag/TlvSection` + `pub mod tree/use_site_path`). md-codec's OWN `tests/bch_adversarial.rs::multi_chunk_descriptor()` builds a ≥4-chunk descriptor this way. But the plan didn't pin this — NEEDS-CHANGE (Important #2).

## Issues

### Critical
None.

### Important

**Important #1 — `StringLayerHeader` `#[non_exhaustive]` makes the Task-5 match fail to compile.** External crate exhaustive match needs a `_` arm. Fix: replace `H::SingleString {..} => ..` with `_ => Err(BundleError::Mk1SingleString(s.to_string()))`.

**Important #2 — `chunked_md1_vector` has no concrete public construction path.** `md_codec::chunk::split` is public and a `Descriptor` is buildable from public types, but the plan deferred this to the implementer with no pinned recipe. Fix: pin md-codec's own `multi_chunk_descriptor()` builder (verbatim, public types) + `split`, or commit pre-computed chunk literals. Without it the TDD red test for `md1_chunked_set_verifies_and_drop_fails` can't be written deterministically.

### Minor

**Minor #1 (m-1) — ms1 early-exit ordering vs spec §4.2.** The `lines().map(parse_line).collect::<Result>()` short-circuits at the first ms1, but lines BEFORE it (in document order) are classified AND validated first. Spec §4.2 says "exit 3 before validating ANY line." Security property (ms1 content never validated) holds; tests pass on error-type. Strict conformance wants a classify-only pre-scan, then validate. Minor.

**Minor #2** — the `_` arm fix for Important #1 also gives forward-compat for future mk-codec variants (subsumed).

**Minor #3** — `foreign_index1_same_setid` is fully pinned (V2 literals correct); the md1 side (`chunked_md1_vector`) is the gap covered by Important #2.

## Assessment

**TDD/Decomposition:** appropriately bite-sized; sound red→green→commit ordering; types→errors→parse→group→coverage→CLI is correct. Only gap: Task-7 helpers (Important #2).

**Spec coverage:** the self-review spec→task map is accurate; §3–§11 all covered; no over-build. Chunked-of-1 md1 correctly flows through `Md1Chunk`→group→`reassemble`→`SetEntry`; unchunked md1 correctly omitted from `sets[]`.

**Correctness traps traced:** `total = chunks.len()` is correct only after `decode`/`reassemble` succeeds (which enforces count==total) — OK. Plate renumbering OK. Checklist `total` lookup (`Some(&s.chunk_set_id) == p.chunk_set_id.as_ref()`) type-checks (`Option<&String>` both sides). `wallet_plates` counts the ms1 reminder — matches spec. ms1 short-circuit: first ms1 in document order wins (Minor #1 ordering nuance).

## Verdict: NOT-GREEN (0 Critical / 2 Important / 3 Minor)

I-1 (`StringLayerHeader` non_exhaustive → add `_` arm) and I-2 (pin the `chunked_md1_vector` construction via md-codec's public `multi_chunk_descriptor` + `split`) must be fixed before execution. Minors are advisory; m-1 (ms1 ordering) is worth a two-pass fix for strict spec conformance.

---

## Fold plan (main session) — ALL FOLDED into the plan
- **I-1** → Task 5 mk1 match now uses `_ => Err(Mk1SingleString)` with a `#[non_exhaustive]` comment.
- **I-2** → Task 7 `chunked_md1_vector` now contains the verbatim public `multi_chunk_descriptor` builder (md-codec `tests/bch_adversarial.rs`, all public types) + `md_codec::chunk::split`; the `_fixture` indirection removed; Task-7 Step 3 + self-review placeholder-scan updated.
- **m-1** → Task 6 `run_bundle` now does an empty-check + classify-only ms1 pre-scan BEFORE the validating `parse_line` pass.
- **m-2/m-3** → subsumed by I-1/I-2.

Re-dispatch plan-R1 to converge (verify the new `multi_chunk_descriptor` builder's public types compile externally, and that no fold introduced a new issue).
