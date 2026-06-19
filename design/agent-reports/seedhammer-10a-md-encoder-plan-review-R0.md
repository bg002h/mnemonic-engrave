# R0 GATE REVIEW — IMPLEMENTATION PLAN #10a (md1 ENCODER + identity + chunk write/read)

**Reviewer:** opus architect (MANDATORY pre-implementation plan R0 gate)
**Date:** 2026-06-19
**Artifact under review:** `design/IMPLEMENTATION_PLAN_seedhammer_10a_md_encoder.md`
**Spec (GREEN @ R0):** `design/SPEC_seedhammer_10_md_encoder.md`
**Blueprint:** `design/agent-reports/seedhammer-10-md-encoder-architect-blueprint.md`
**Spec R0 review (5 Minors folded):** `design/agent-reports/seedhammer-10-md-encoder-spec-review-R0.md`

**Authoritative sources verified against (read directly, not via prose):**
- Rust `md-codec` v0.36.0 @ `c85cd49` (`/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec`)
- Go fork @ `e4ca173` (`/scratch/code/shibboleth/seedhammer`)
- `me-cli` `bundle.rs` (`/scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/src/bundle.rs`)

---

## VERDICT: NOT GREEN

**1 Important, plus Minors.** One load-bearing fidelity gap (the per-node TAG write is dropped from the otherwise-complete 9-arm enumeration and from the Helpers list) must be folded before code. Everything else verified clean: the two CRITICAL traps are correctly TDD'd before integration, the 9 body arms + validators are present, the golden literals + vendored files are correct, and M-5 / I-3 are captured faithfully. The Important is a one-line enumeration fix; once folded and re-dispatched, this plan is GREEN-ready.

Explicit confirmations requested by the gate brief:
- **(a) Two CRITICAL traps TDD'd before integration goldens — CONFIRMED.** bitWriter (T1) and canonicalize (T2) each land with their own unit tests (T1.1 ports `bitstream.rs:236-407`; T2.1 idempotence + normalization) and committed BEFORE the byte-exact integration goldens in T3.5. Dependency order is sound (see Finding M-1 / §1).
- **(b) 9 node arms + validators complete — CONFIRMED for bodies; the per-node TAG write is MISSING (Finding I-1).** All 9 `Body` arms match `tree.rs:79-176`; all 8 validators present. The leading `node.tag.write(w)` (`tree.rs:80`) is not in the enumeration.
- **(c) Golden literals + vendored files correct — CONFIRMED with one vendoring caveat (Finding I-1 is unrelated; see M-2).** `wpkh_basic=2002001800` ✓, `wsh_with_fingerprints=204200182182142f09bd5b7ddfcafebabe` ✓; all 10 MANIFEST files exist. The `chunked_md1_vector` is a Rust *builder function*, not a copyable golden file (M-2).
- **(d) M-5 and I-3 correctly captured — CONFIRMED.** M-5 (symbol-aligned bit count, not `len(bytes)*8`) is in T7; I-3 (bit-0 discriminator first, never ParseChunkHeader-then-catch) is in T7. Both match the Rust source verbatim, including the explicit source-comment warning.

---

## CRITICAL findings

None.

---

## IMPORTANT findings (BLOCKING — must fold to reach GREEN)

### I-1. Task 3 drops the per-node TAG write from the `writeNode` enumeration and the Helpers list. (Plan §Task 3, lines 62-72.)

**Issue.** Task 3's `writeNode` "must handle all node bodies (M-3 — enumerate explicitly)" lists exactly the 9 *body* arms (KeyArg … Empty). But in the authoritative source, `write_node` emits the node's **tag FIRST**, before the body match:

```
// tree.rs:79-82
pub fn write_node(w: &mut BitWriter, node: &Node, key_index_width: u8) -> Result<(), Error> {
    node.tag.write(w);          // <-- 6-bit tag, emitted before any body
    match &node.body { ... }
```

`Tag::write` is `tag.rs:140-146`: `w.write_bits(primary, 6)` (+ optional 4-bit ext, unused in v0.30 for every renderable tag, all ≤ `Tag::True`=0x23 < the 0x3F extension prefix). The Go decoder inverse is `readTag` (`md/md.go:81-101`, called first in `readNodeDepth` at `md/md.go:333`). The plan's Helpers list (line 72) enumerates `header.write`, `pathDecl.write`, `useSitePath.write`, `tlvSection.write`, `writeVarint`, `kiw` — but **no `writeTag`** (the inverse of `readTag`).

Why this is load-bearing and not merely cosmetic: the encoder cannot be byte-exact without writing the 6-bit tag per node. An implementer following the plan's "enumerate explicitly" arm-by-arm checklist literally — and the plan elevates this enumeration to an acceptance bullet ("All 9 SPEC invariants … the 2 CRITICAL traps … before integration goldens", line 155) — has no instruction to emit the tag. The bit-cost pin `sortedmulti_2of3 = 22b` (T3.1) WOULD catch it (the 22 bits = `Tag(6) + k-1(5) + n-1(5) + 3×kiw(2)`, per `tree.rs:411` doc-comment), and the integration golden definitely would. But "a downstream test catches the omission" is exactly the fidelity-trap class the plan's own M-3 enumeration exists to prevent up front, and the gate brief requires the enumeration be complete ("Does Task 3 enumerate ALL node bodies the decoder can produce").

**Authoritative source:** `tree.rs:80` (`node.tag.write(w)`); `tag.rs:140-146` (`write` = 6-bit primary); Go inverse `md/md.go:81-101` `readTag`, called at `md/md.go:333`.

**Exact fix.** In Task 3, (a) add to the Helpers list a `writeTag(w, t)` = the inverse of `readTag` (`w.write(uint64(t), 6)`; v0.30 renderable tags never use the 0x3F extension, so the 4-bit ext branch is unreachable and may be omitted or asserted-unreachable); and (b) prepend to the 9-arm `writeNode` description an explicit step "**write the node tag (6 bits) via `writeTag` FIRST (inverting `readTag` `md/md.go:81`, mirroring `node.tag.write` `tree.rs:80`), then the body per the arm below**." Add a per-tag bit-cost assertion to T3.1 (e.g. a bare `wpkh` KeyArg node at n=1 costs 6 bits = tag only, since kiw=0 — pins the tag write in isolation; cf. `tree.rs:660` `False` primary = 6 bits no body).

---

## MINOR findings (non-blocking; fold opportunistically)

### M-1. Dependency order is sound — confirmed, no change needed (recorded for completeness).
The brief asks to confirm `computeEncodingID` (T4) is ordered after `encodePayload` (T3) — **it is** (T4 depends on T3's `encodePayload`, T3 commits first). Full chain: T1 bitWriter → T2 canonicalize → T3 encodePayload+writers+validators → T4 identity → T5 codex32 md-checksum/`encodeMD1String` → T6 chunk write (`split` depends on T3/T4) → T7 chunk read (`Reassemble` depends on T3 decode + T4 identity) → T8 no-regression/fuzz. No back-edges; every consumer follows its producer. The two CRITICAL traps (T1, T2) precede all structural/integration work, satisfying I-2's TDD-ordering mandate. Verified against the actual `encode.rs:65-92` call graph (`encode_payload` → `canonicalize_placeholder_indices` → validators → `BitWriter`) and `chunk.rs:235-290`/`:305-389` (`split`/`reassemble` both call `compute_md1_encoding_id`).

### M-2. Task 0 Step 3 mis-frames `chunked_md1_vector` as a vendorable golden FILE; it is a Rust *builder function*. (Plan line 23.)
`me-cli/src/bundle.rs:547-585` `fn chunked_md1_vector() -> Vec<String>` does **not** read any `.bytes.hex`/`.phrase.txt`/`.descriptor.json`; it constructs a `Descriptor` programmatically (n=6; `PathDeclPaths::Divergent` of 6 paths each 15 hardened components `value = c*100 + i + 1`; `Wsh→SortedMulti{k:2, indices:0..6}`; empty TLV) and calls `md_codec::chunk::split(&d)` (verified: lines 547-585, `split` at line 584). So Task 0's instruction to "copy … the `chunked_md1_vector` fixture from `me-cli/src/bundle.rs:547-585`" alongside the `.{bytes.hex,phrase.txt,descriptor.json}` files is not literally executable — there are no such files for it. The plan recovers the correct intent later (T6/T7 say "build d → `split(d)`"), so this is non-blocking, but Task 0 Step 3 should be reworded: "for `chunked_md1_vector`, **hand-build the equivalent Go `descriptor`** (n=6; 6 divergent origin paths, each 15 hardened comps `c*100+i+1`; wsh→sortedmulti k=2 indices 0..5; empty TLV) and drive it through the Go `split`/`Reassemble` round-trip — there is no golden file to copy." (The 6 vendorable file-triples are the 10 MANIFEST single/`wsh_multi_chunked` vectors only.)

### M-3. `wsh_multi_chunked` must be EXCLUDED from the `encodeMD1String` single-string parity test (T5), and is a count==1 force-chunked artifact. (Plan T5 line 102, T6 line 116, M-4 line 23.)
Verified: `wsh_multi_chunked` is `force_chunked: true` with template `wsh(multi(3,…))`; its payload is 8 bytes (`.bytes.hex = 2082001821842180`) → `ceil(64/320) = 1` chunk. Its `.phrase.txt` is a **chunk-format** string with a `chunk-set-id: 0x157ae` line + `md1fz4awqqpqsgqpsgvyyxqql8saf74dwdyqv` — NOT the single-string form `encodeMD1String` emits (the 9 true singles look like `md1yqpqqxqq8xtwhw4xwn4qh`). Consequences the plan must lock:
  - **T3.5 (byte parity):** `wsh_multi_chunked.bytes.hex` IS valid for `encodePayload` byte-equality (payload bytes are chunk-independent). Keep it. ✓
  - **T5 (`encodeMD1String == .phrase.txt`):** MUST exclude `wsh_multi_chunked` — its `.phrase.txt` is a chunk string and would never equal a single-string encode. The plan says "for each SINGLE golden," which is correct only if "single" is read as "not force_chunked." Make that explicit so the implementer's test-table filter excludes it (e.g. skip `force_chunked` rows in the `encodeMD1String` table).
  - **T6 (count==1 path):** `wsh_multi_chunked` exercises `split`→1 chunk; fine to use it for the count==1 assertion, but note its chunk string carries a real ChunkHeader (chunked flag set, csid 0x157ae), distinct from a `wpkh_basic` single. (M-4 in the spec R0 already flagged the two-artifact distinction; this is the operational consequence for the test tables.)

### M-4. `.descriptor.json` → Go `descriptor` loader is a real (non-trivial) shim, not a one-line `json.Unmarshal`; the "or hand-build" escape keeps it non-blocking. (Plan T3 line 78.)
The JSON shape is the Rust-tagged AST: `tree: {tag:"Wsh", body:{kind:"Children"|"MultiKeys"|"Tr"|"KeyArg"|…, data:…}}`, `path_decl:{tag:"Shared"|"Divergent", data:"m"|…}`, TLV maps as `[[idx,"hexfp"],…]` or `null`. The Go AST is `node{tag tag; body body}` with `body` an **interface** (childrenBody/variableBody/multiKeysBody/trBody/keyArgBody/hash256Body/hash160Body/timelockBody/emptyBody) and `tag` a `uint8` with named consts; `pathDecl{n, shared *originPath, divergent []originPath}`; TLV as typed slices + `*Present bool`. A plain `json.Unmarshal` into the Go types cannot work (interface body, string→`tag` mapping like `"SortedMulti"→tagSortedMulti`, `"Shared"/data:"m"`→empty `shared` originPath, `[[0,"deadbeef"]]`→`idxFP{0,[4]byte}`+`fpPresent`). The plan's "(write a small testdata loader … `encoding/json` is fine in `_test.go`) **or hand-build**" is feasible (a `_test.go` custom unmarshaler with a kind-dispatch + tag-name map), and `encoding/json` in `_test.go` is TinyGo-safe (test-only, never in production paths — confirmed: production-path imports stay clear of `encoding/json`/`reflect`/`math/big`). Recommend the plan explicitly note the loader is a custom shim (kind-dispatch + tag-name table + TLV-array decode), so the implementer budgets for it rather than expecting `json.Unmarshal` to "just work."

### M-5. Plan should mark the three pre-emission validators and the unknown-TLV pack as REUSE, not re-port. (Plan T3 line 60, line 72.)
`validatePlaceholderUsage` (`md/md.go:904`), `validateMultipathConsistency` (`md/md.go:977`), `validateTapScriptTree` (`md/md.go:1004`) **already exist** in the shipped Go decoder, and `readUnknownPayload` (`md/md.go:623-655`) already packs MSB-first (the correct inverse for `reEmitBits(payload, bitLen)`). Task 3 says "validatePlaceholderUsage + … validateMultipathConsistency + … validateTapScriptTree" without noting these are existing functions to call (the blueprint §0 "Reuse as-is: …validators" covers it, but the plan should too, so the implementer doesn't needlessly re-port ~120 LOC). The encode-side body validators that ARE net-new are the `tree.rs:90-139` inline guards (Threshold/ChildCount/KGreaterThanN) the plan does list as typed errors in T3.3 — correct.

### M-6. Task 5 Step 3 "`bitsToSymbols` … (if not already exposed by `MDDataSymbols`)" — the hedge is misleading; `bitsToSymbols` is definitively net-new. (Plan line 104.)
`MDDataSymbols` (`codex32/mddata.go:15`) is the **decode** direction (string→symbols, with BCH verify). The encode direction `bits_to_symbols` (bytes+bitcount → 5-bit symbols, final symbol left-justified `(val<<(5-take))&0x1F`, `codex32.rs:23-42`) does **not** exist in the Go tree (only the decode-side `symbolsToBytes` at `md/md.go:867`). Reword to "implement `bitsToSymbols` (net-new; encode-direction inverse of `symbolsToBytes`, mirroring `codex32.rs:23-42` left-justify)". Non-blocking (the Rust cite is correct), just drop the false "if not already exposed."

### M-7. Task 4 `deriveChunkSetID` adds a `& 0xFFFFF` mask not present in Rust. (Plan line 90.)
`chunk.rs:175-179` is `((b0 as u32)<<12)|((b1 as u32)<<4)|((b2 as u32)>>4)` with NO trailing mask (the value is intrinsically ≤ `0xFFFFF` since `b0<<12` tops at `0xFF000`). The plan's `… & 0xFFFFF` is a harmless no-op but diverges from "mirror exactly." Drop it for byte-faithful parity, or keep it as a defensive comment. Cosmetic.

### M-8. Task 2 omits canonicalize's leading bounds/reference guards + identity fast-path. (Plan T2 lines 46-50.)
`canonicalize_placeholder_indices` (`canonicalize.rs:168-`) opens with `check_placeholder_bounds` (`:174`) and returns `PlaceholderNotReferenced` (`:185-189`) for any `@i` (i<n) not referenced in the tree, plus an identity fast-path (`:199-201`) before applying the permutation; it remaps tree + divergent paths + all 4 TLV maps via `remap_indices` (`:102`) and `remap_sparse_tlv` (re-key + `sort_by_key`, `:141-148`). The plan's "Mirror `canonicalize.rs` exactly" technically covers these, but since T2's normalization test only exercises the permute path, add a T2.1 case for (i) `PlaceholderNotReferenced` (a tree referencing @0,@2 with n=3 → typed error) and (ii) the identity fast-path (an already-canonical AST returns deep-equal — already partly covered by the idempotence test). Non-blocking; strengthens the CRITICAL-trap coverage.

---

## Verified-correct (no action) — for the record

- **Pinned golden literals.** `wpkh_basic.bytes.hex == 2002001800` ✓; `wsh_with_fingerprints.bytes.hex == 204200182182142f09bd5b7ddfcafebabe` ✓ (read directly). The 10-entry MANIFEST (`test_vectors.rs`) = wpkh_basic, pkh_basic, wsh_multi_2of2, wsh_multi_2of3, wsh_sortedmulti, tr_keyonly, sh_wsh_multi, wsh_divergent_paths, wsh_with_fingerprints, wsh_multi_chunked — all `.{bytes.hex,phrase.txt,descriptor.json}` present.
- **Baseline (Task 0 Step 2) is achievable.** `go test ./md/... ./codex32/... ./mk/... ./bip380/...` → all `ok` (throwaway run).
- **bitWriter / re_emit_bits citations.** `BitWriter` struct+impl `bitstream.rs:11-84` ✓ (MSB-first; `into_bytes` returns buffer with final partial byte low-padded since unwritten low bits stay 0); `re_emit_bits` `bitstream.rs:220-230` ✓; writer unit tests `bitstream.rs:236-407` ✓ (incl. `write_5_bits_msb_first → 0b1011_0000`, `re_emit_bits_non_byte_aligned_source`). `md/bits.go` confirmed reader-only (no `bitWriter`/`reEmitBits`); throwaway `testBitWriter` at `md_test.go:86-117` ✓.
- **kiw.** `encode.rs:37-41` `(32 - n.saturating_sub(1).leading_zeros())`; Go `md/md.go:842` `32 - bits.LeadingZeros32(uint32(pd.n)-1)`. Plan's `kiw(n)=32-LeadingZeros32(n-1)` clamp-0 at n∈{0,1} ✓.
- **9 `writeNode` body arms.** `tree.rs:79-176` exactly: KeyArg(`index@kiw`), Children(recurse, no count), Variable(`(k-1)5b,(len-1)5b,children`, guards k,n∈1..32 & k≤n), MultiKeys(`(k-1)5b,(len-1)5b,n×index@kiw` RAW), Tr(`is_nums 1b; if !nums key_index@kiw; has_tree 1b; opt subtree`; NUMS suppresses kiw), Timelock(32b), Hash256Body(32×8b), Hash160Body(20×8b), Empty(nothing). Plan T3 arms 1-9 match. (TAG write is the I-1 gap.)
- **Encode-side validators.** ThresholdOutOfRange/ChildCountOutOfRange/KGreaterThanN inline `tree.rs:92-138`; OverrideOrderViolation/EmptyTlvEntry `tlv.rs` (`md/md.go:567,578`); VarintOverflow `varint.rs:30-31`; path-depth/alt-count guards present. All 8 in plan T3.3.
- **TLV write.** `tlv.rs:200-206`: per-entry sub-bitstreams, `sort_by_key(tag)` ascending, `[tag:5][write_varint(bit_len)][re_emit_bits(payload,bit_len)]`; idx strictly ascending; empty rejected; unknown re-emitted via `re_emit_bits`. Tag width 5 bits (distinct from the 6-bit tree tag). Plan T3 Helpers `tlvSection.write` ✓.
- **writeVarint.** `varint.rs:15-42` LP4-ext, lengths in BITS, `[L:4][payload:L]`, L=15 escape. Bit-cost pins `varint(0)=4b`/`(1)=5b`/`(84)=11b` (`varint.rs:111-128`) ✓. Origin BIP84=26b (`origin_path.rs:184-189`), +n-prefix=31b (`:243`); use-site `<0;1>/*`=16b (`use_site_path.rs:134-139`); sortedmulti 2of3=22b incl. tag (`tree.rs:411-425`). All plan T3.1 pins verified.
- **Identity.** `compute_md1_encoding_id` = SHA-256(payload)[0:16] (`identity.rs:39-45`); `derive_chunk_set_id` `(b0<<12)|(b1<<4)|(b2>>4)` (`chunk.rs:175-179`); `AB CD EF→0xABCDE` test `chunk.rs:199-207` ✓.
- **ChunkHeader.write.** `chunk.rs:32-57`: 37 bits `[version:4 TOP][chunked=1:1][csid:20][count-1:6][index:6]`; guards count∈1..64, index<count, csid<2^20. Plan T6 ✓.
- **split.** `chunk.rs:235-290`: threshold `SINGLE_STRING_PAYLOAD_BIT_LIMIT=320`, `div_ceil`, `>64→ChunkCountExceedsMax`, `bytes_per_chunk=div_ceil`, per-chunk `37+8N` bits, `wrap_payload`. Plan T6 ✓. (Sizing uses `payload.len()*8` on the WRITE side — correct; M-5 is the READ-side counterpart.)
- **M-5 (READ-side).** `chunk.rs:315-328`: `payload_byte_count = (symbol_aligned_bit_count − 37) / 8`, from `unwrap_string`'s symbol-aligned count, NOT `bytes.len()*8` — the Rust even carries an explicit source comment warning of the N=3/N=8 break. Plan T7 captures this verbatim ✓.
- **I-3 (discriminator).** Bit-0 first: Go `md/md.go:1207` `syms[0]&1 == 1`; me-cli `bundle.rs` reads bit 0 before any `ChunkHeader::read`. Plan T7 mandates `syms[0]&1` first, never parse-then-catch ✓.
- **Reassemble integrity + negative tests.** `chunk.rs:305-389`: consistency→`ChunkSetInconsistent` (:343-350); completeness→`ChunkSetIncomplete` (:355-360); gaps→`ChunkIndexGap` (:363-367); concat → `decode_payload(full, full.len()*8)` (TLV rollback ≤7 trailing zero bits, `md/md.go:549-554`); integrity `compute→derive→compare→ChunkSetIdMismatch` (:378-386). Plan T7 negative tests (drop/reorder/dup/corrupt-csid/cross-set) map to the right typed errors ✓.
- **codex32 md BCH.** `mdmkPolymodInitLo=0x23181b3` (NOT codex32's 1), `mdRegularTargetHi=0x0`/`Lo=0x0815c07747a3392e7`, `mdmkShortSyms=13`, `ValidMD` `codex32/mdmk.go:124-127`, `MKChecksumSymbols` precedent `codex32/mkencode.go:18-55`. md is regular-only (`REGULAR_CHECKSUM_SYMBOLS=13`, no long branch) — so `assembleMD1` correctly carries no `long` logic. Plan T5 / I-7 ✓.
- **bits_to_symbols / wrap_payload.** `codex32.rs:23-42` (final symbol left-justify) + `:67-83` (regular BCH over HRP "md"). Plan T5 `encodeMD1String = encodePayload→bits_to_symbols→assembleMD1` ✓.
- **mk.Header shape.** `mk/mk.go:48-56` `{Chunked, ChunkSetID, TotalChunks, ChunkIndex}`; plan's `ChunkHeader` adds `Version` (md-specific) — consistent.
- **Phase-B correctly deferred.** Plan makes NO reference to `validate.rs:221`/secp256k1/`md.go:1071`/GUI refusal change — all #10b. The spec-R0 M-2 (secp256k1 at `validate.rs:221` not `:216`) is phase-B-only and irrelevant to this plan. ✓
- **TinyGo safety.** Production paths use only `uint64`/`sha256`/stdlib bit ops; `encoding/json` confined to `_test.go` (M-4). T8 no-regression + alloc-gate + fuzz adequate. ✓

---

## Required action to reach GREEN
Fold **I-1** (add `writeTag` to Helpers + prepend the per-node tag-write step to the `writeNode` enumeration + a tag-only bit-cost pin in T3.1). Folding the Minors (esp. M-2, M-3, M-6) is recommended in the same pass. Persist this review verbatim, re-dispatch the R0 gate after the fold (folds can introduce drift), and only proceed to single-implementer TDD once the re-dispatch returns 0C/0I.

## Verification artifacts (throwaway; no source modified)
- Direct reads of `md-codec/src/{bitstream,encode,canonicalize,chunk,tree,tag,varint,origin_path,use_site_path,codex32,identity}.rs` @ `c85cd49`.
- Direct reads of `seedhammer/md/{md.go,bits.go,md_test.go}`, `mk/{mk.go,encode.go}`, `codex32/{mdmk.go,mkencode.go,mddata.go}` @ `e4ca173`.
- `me-cli/src/bundle.rs:533-585` (`chunked_md1_vector` builder).
- `cat` of all 10 `.bytes.hex` + `wpkh_basic`/`wsh_with_fingerprints`/`tr_keyonly`/`wsh_multi_chunked` `.descriptor.json` + the 9 single `.phrase.txt`.
- `go test ./md/... ./codex32/... ./mk/... ./bip380/...` → all `ok` (baseline GREEN).
