# RECON (T5, agent aa63020b3ad9a1c13, 2026-06-19) — bundle-set protocol facts (source-verified)

**Question:** What does "a complete, consistent chunked md1/mk1 SET" mean, and how are completeness/consistency/integrity proven — does a device-side mirror of host `me bundle` need the (deferred #10) md ENCODER?

**Crate/commit pins:** `me-cli` v0.3.0 @ `2ee44ad` (pins `md-codec="0.36"`, `mk-codec="0.4"`); `mk-codec` v0.4.0 @ `913febc`; `md-codec` v0.36.0 @ `c85cd49`.

---

## 1. Chunk header / set model

### mk1 — `StringLayerHeader::Chunked`, 8 five-bit symbols (`mk-codec/src/string_layer/header.rs`)
| Field | Width | Encoding | Source |
|---|---|---|---|
| version | 5b (1 sym) | verbatim; v0.1=0x00; nonzero→UnsupportedVersion | header.rs:30,124-127 |
| type | 5b (1 sym) | 0x00=SingleString, 0x01=Chunked; 0x02..=UnsupportedCardType | header.rs:16-17,128-129,174 |
| chunk_set_id | **20b (4 sym)** big-endian | verbatim 0..=0xFFFFF | header.rs:27,92-95(w),138-141(r) |
| total_chunks | 5b | **stored value−1**; semantic 1..=32 (MAX_CHUNKS=32) | header.rs:88(w),146(r) |
| chunk_index | 5b | **verbatim, 0-based** | header.rs:97(w),147(r) |
Header validity (header.rs:154-163): total 1..=32; index<total else ChunkedHeaderMalformed. SingleString header = 2 syms, NO chunk_set_id (header.rs:20,38-41). "total−1 but index verbatim" roadmap claim CONFIRMED.

### md1 — `ChunkHeader`, 37 bits (`md-codec/src/chunk.rs`)
| Field | Width | Encoding | Source |
|---|---|---|---|
| version | 4b | WF_REDESIGN_VERSION (v0.30=4); mismatch→WireVersionMismatch | chunk.rs:51,68-71 |
| chunked flag | 1b | first sym MSB-first `[v3..v0][chunked]`; must=1 | chunk.rs:4-6,52,72-75 |
| chunk_set_id | **20b** | verbatim 0..=0xFFFFF | chunk.rs:46-49,53(w),76(r) |
| count(total) | 6b | **stored count−1**; valid 1..=64 | chunk.rs:54(w),77(r),37-39 |
| index | 6b | **verbatim 0-based**; index<count | chunk.rs:55(w),78(r),40-45 |
Total 37b. md1 count is 6b/max-64 (wider than mk1 5b/max-32). md1 chunk_set_id is **DERIVED** (top 20 bits of SHA-256-based Md1EncodingId, chunk.rs:175-179), NOT random; mk1's is **random** 20-bit CSPRNG (pipeline.rs:45-49) or caller-pinned.

**md1 single-vs-chunked discriminator:** bit 0 of first 5-bit symbol (`symbols.first() & 0x01`). `me bundle` probes this BEFORE `ChunkHeader::read` to dodge an md-codec 0.36 quirk where a pristine single md1 makes `ChunkHeader::read` return spurious `WireVersionMismatch{got:2}` (bundle.rs:135-151, documented). Device mirror must replicate.

## 2. Completeness (all indices 0..total−1 once)
- **mk1** `reassemble_from_chunks` (mk-codec string_layer/chunk.rs:109-203): count==total (131-136); slot array, idx>=total rejected (158-162), dup slot rejected (163-167), missing slot→error (183-186).
- **md1** `reassemble` (md-codec chunk.rs:305-389): len==expected_count else ChunkSetIncomplete (351-356); sort by index, verify index==i → ChunkIndexGap (359-367; dup also trips this).
- **`me bundle`** (me-cli/bundle.rs): groups by chunk_set_id in BTreeMap, hands whole group to codec — mk1 `mk_codec::decode(&refs)` (273), md1 `md_codec::chunk::reassemble(&refs)` (246). Delegates completeness to codec; drop→SetIncompleteMk/Md→exit 4 (tests :456,:502).

## 3. Consistency (same version/set_id/total across set)
- mk1: chunk_set_id vs first (ChunkSetIdMismatch, chunk.rs:149-151), total (ChunkedHeaderMalformed, 152-156), SingleString in chunked set→MixedHeaderTypes (170-177).
- md1: count, chunk_set_id, version vs chunk0 → ChunkSetInconsistent (md-codec chunk.rs:343-350).
- `me bundle` adversarial cases: dup index→exit4 (test :480); index≥total rejected at header parse (mk1 159-162/md1 40-45); mismatched totals/set-id→exit4; **foreign chunk w/ DIFFERENT csid** → lands in own singleton group → incomplete (test :424-432); **foreign chunk re-stamped SAME csid** → caught only by cross-chunk integrity hash §4 (test cross_chunk_hash_mismatch_fails :489-500).

## 4. Integrity — THE PIVOTAL DIFFERENCE
### mk1 — `SHA-256(bytecode)[0..4]` at stream END — stream-level, **NO encoder**
- Encoder: stream = canonical_bytecode || SHA-256(bytecode)[0..4], split into ≤53B fragments (mk-codec chunk.rs:66-70; CROSS_CHUNK_HASH_BYTES=4 consts.rs:45).
- Reassembly: concat in index order, split trailing 4B, recompute SHA-256 over recovered bytecode, compare→CrossChunkHashMismatch (chunk.rs:195-201). Computed directly over reassembled bytes — NO structural decoder, NO re-encoder. Roadmap claim CONFIRMED.
- Caveat: `me bundle` calls higher-level `mk_codec::decode` (bundle.rs:273) which runs reassemble_from_chunks (hash check, encoder-free) THEN decode_bytecode→KeyCard (pipeline.rs:150-151). The set-integrity subset (complete+consistent+cross-chunk-hash) is fully in `reassemble_from_chunks`, needs no encoder/decoder. A device mirror wanting only "is this set complete/consistent/integral" can stop at the reassemble equivalent.

### md1 — encoding-id integrity — **REQUIRES full DECODER + full RE-ENCODER**
- md1 has NO trailing hash in stream. After concat, `reassemble`:
  1. `decode_payload(&full_bytes, len*8)` → structural Descriptor (chunk.rs:376) — full top-level decoder (header/path/tree/TLV/placeholder/multipath/taptree/origin/xpub, decode.rs:15-72).
  2. `compute_md1_encoding_id(&descriptor)` (chunk.rs:379) — **re-runs `encode_payload(d)`** then SHA-256 (identity.rs:39-44).
  3. `derive_chunk_set_id(md1_id)` (top 20b) vs header chunk_set_id → ChunkSetIdMismatch (chunk.rs:380-386).
- This is the ONLY md1 cross-chunk integrity gate and is intrinsically payload-level: the comparison value is a function of the RE-ENCODED canonical payload, not wire bytes. `compute_md1_encoding_id`→`encode_payload` IS the md ENCODER (the deferred #10 dependency). No header-only shortcut exists; `reassemble` unconditionally decodes+re-encodes before returning.

## 5. ms1 refusal
- Classify by HRP before `1`, case-insensitive (classify.rs:40-52); `ms`→Format::Ms.
- `run_bundle` classify-only pre-scan over ALL lines BEFORE validating any (bundle.rs:188-192): any ms1→`RefusedSecret` immediately (no BCH-validation of any line). `parse_line` also refuses (97-98). **Exit 3** (distinct from 2=empty/usage, 4=invalid/integrity). Tests :370,:470. Error tells operator to hand-enter ms1 on device (New>Input Seed>CODEX32), never NFC/tool. Maps to device secret-spine.

## 6. Minimal device-side T5 checks — header-only vs payload-level
Per-string before grouping (both): classify by HRP + **refuse ms1 up front, pre-scan all before validating any** (cheap, no codec); per-string BCH/codex32 pristine validation (per-chunk, no set encoder); parse chunk header→(csid,total,index) (header-only, cheap; mind md1 single-vs-chunked flag probe).

| Per-set check | mk1 | md1 |
|---|---|---|
| Completeness | header-only, no encoder | header-only, no encoder |
| Consistency | header-only | header-only |
| Cross-chunk integrity | **stream-level, NO encoder** (recompute SHA-256[0..4]) | **PAYLOAD-LEVEL, needs DECODER+ENCODER** (decode→re-encode→derive csid→compare) |

## VERDICT
**md1 set-verification REQUIRES the full md encoder (and decoder)** — settled by md-codec/src/chunk.rs:376 (decode_payload), :379 (compute_md1_encoding_id→encode_payload), :380-382 (the only md1 cross-chunk integrity gate) + identity.rs:39-44. No trailing-hash shortcut (contrast mk1 chunk.rs:198-199); md1 csid is derived from re-encoded payload (chunk.rs:175-179) so the integrity value can't be reconstructed without re-encoding.

**Consequence for T5 scope:**
- **T5 can ship mk1-only set-sequencing/verification today, NO md encoder** — completeness+consistency+cross-chunk-hash all header/stream-level (reassemble_from_chunks, no encode_* call).
- **Full md1 set-integrity needs #10 (the md encoder).** A partial md1 path (header-only completeness+consistency, skipping integrity) is strictly weaker than `me bundle` and would silently accept a foreign chunk re-stamped with the same csid (md1 analogue of cross_chunk_hash_mismatch_fails). Faithful md1 mirror ⇒ encoder required.

## Reconciliation
No roadmap/FOLLOWUPS claim contradicted by source. FOLLOWUPS:72 "via mk_codec::decode / md_codec::chunk::reassemble" accurate (bundle.rs:273,246). The asymmetry above (mk1 encoder-free, md1 needs decode+re-encode) is the deciding factor for T5 vs #10 — under-stated in prose, now explicit.

Key planner files: me-cli/src/{bundle,classify}.rs; mk-codec/src/string_layer/{header,chunk,pipeline}.rs; md-codec/src/{chunk,identity,decode}.rs.
