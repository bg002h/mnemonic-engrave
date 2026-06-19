# R0 GATE RE-REVIEW (Round R1, post-fold) — IMPLEMENTATION PLAN #10a (md1 ENCODER + identity + chunk write/read)

**Reviewer:** opus architect (MANDATORY pre-implementation plan R0 gate — re-review after fold)
**Date:** 2026-06-19
**Artifact under review:** `design/IMPLEMENTATION_PLAN_seedhammer_10a_md_encoder.md` (folded)
**Prior review re-checked:** `design/agent-reports/seedhammer-10a-md-encoder-plan-review-R0.md` (NOT GREEN — 1 Important + Minors)
**Spec (GREEN @ R0):** `design/SPEC_seedhammer_10_md_encoder.md`
**Blueprint:** `design/agent-reports/seedhammer-10-md-encoder-architect-blueprint.md`

**Authoritative sources verified against this round (read directly, file:line cited; commits confirmed via `git rev-parse`):**
- Rust `md-codec` v0.36.0 @ `c85cd49` (`/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec`) — HEAD confirmed `c85cd498c690…`, `Cargo.toml version = "0.36.0"`.
- Go fork @ `e4ca173` (`/scratch/code/shibboleth/seedhammer`) — HEAD confirmed `e4ca173621f0…`.
- `me-cli` `bundle.rs` (`/scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/src/bundle.rs`).

---

## VERDICT: GREEN

**0 Critical / 0 Important.** The lone blocking Important (I-1, per-node TAG write) is CLOSED across all three required sites, verified against authoritative Rust + Go source. Every folded Minor (M-2, M-3, M-4, M-5, M-6) is now accurate against source. The fold introduced NO drift: the two CRITICAL traps remain TDD'd-before-integration, the dependency order is intact, the 9 body arms + 8 validators + golden literals are unchanged and still correct, and M-5 / I-3 / ChunkHeader / split-threshold / POLYMOD_INIT all still match source verbatim. **The plan is cleared for single-implementer TDD in the worktree.**

Two residual cosmetics (R0 M-7 `& 0xFFFFF` no-op mask; R0 M-8 canonicalize edge-case test cases) remain un-folded; both were explicitly Minor/non-blocking in R0, are not gate-blocking, and are carried forward below for the implementer's opportunistic attention.

---

## I-1 (IMPORTANT) — per-node TAG write — **CLOSED**

The fold restores the dropped per-node tag write across all three required sites; each is verified inverse-correct against source.

**(a) `writeNode` mandates writing the 6-bit tag FIRST on every arm.** — CLOSED.
Plan §Task 3 line 62: "`writeNode` must, for EVERY node, **write the node's 6-bit TAG FIRST** (`node.tag.write(w)`, `tree.rs:80` → `tag.rs:140-146`; the inverse of Go's `readTag` `md/md.go:81-101`) — THEN the body. **R0-I1: the tag write is mandatory and applies to all arms; without it the wire desyncs …**", and each of the 9 enumerated bodies is qualified "each written AFTER its tag."
- Source: `tree.rs:79-80` — `pub fn write_node(w, node, key_index_width)` then **`node.tag.write(w);`** as the FIRST statement, *before* the `match &node.body` (line 81). Verified by direct read.
- Tag width: `tag.rs:140-146` `Tag::write` = `w.write_bits(primary, 6)` + optional 4-bit ext; the ext branch is unreachable for every renderable v0.30 tag (all codes ≤ `Tag::True`=0x23 < the 0x3F extension prefix; `tag.rs:99-137`). Verified.
- Go inverse: `readTag` reads the 6-bit primary (`md/md.go:81-97`) and is the FIRST call in `readNodeDepth` (`md/md.go:333`, immediately after the depth guard, before the body `switch`). Verified `readNode→readNodeDepth` at `md/md.go:325-333`.

**(b) `writeTag` added to the Helpers list.** — CLOSED.
Plan §Task 3 line 72 Helpers now leads with: "`writeTag` (6-bit primary tag, inverse of `readTag` `md/md.go:81-101`, codes table `tag.rs:99-137` ↔ Go `md.go:38-75`; ext `0x3F` unused in v0.30)."
- Codes table cross-check: `tag.rs:99-137` (`Wpkh=0x00 … True=0x23`) is byte-for-byte identical to the Go `tag` consts `md/md.go:38-75` (`tagWpkh=0x00 … tagTrue=0x23`). Verified both directly. `extensionPrefix6Bit = 0x3F` exists in Go (`md/md.go:77`) and is rejected on read; the encode side correctly never emits it (all renderable tags ≤ 0x23). Citation `md/md.go:81-101` is accurate (the function body is 81-97; the doc-comment header begins at 79 — within tolerance).

**(c) tag-only bit-cost pin added to Task 3 Step 1.** — CLOSED.
Plan §Task 3 Step 1 line 74: "`writeTag`=6b (single tag write asserts `bitLen()==6`, R0-I1)." Also the "bare `wpkh` KeyArg node at n=1 costs 6 bits = tag only, since kiw=0" framing in the R0 fix note is consistent.
- Source anchor: `tree.rs:653-664` `false_round_trip` writes `Tag::False`+`Body::Empty` and asserts `w.bit_len() == 6` with the in-source comment "Tag(6), no body". So a tag-only emission is 6 bits in the authoritative tests. The plan's two equivalent framings (bare `False`/`Empty`, or `wpkh` KeyArg at n=1 where kiw=0) both yield exactly 6 bits. Verified.
- Cross-pin: the existing `sortedmulti(2-of-3)=22b` pin (plan line 74) is grounded in `tree.rs:411` doc-comment `Tag(6-bit) | k-1(5) | n-1(5) | 3×kiw(2 at n=3) = 22 bits` and the `sortedmulti_2of3_bit_cost` test (`tree.rs:414-425`, `assert_eq!(w.bit_len(), 22)`). This pin **includes** the 6-bit tag, so it independently guards the tag write at the MultiKeys arm — exactly the late-catch the R0 warned about, now backstopped by the up-front isolated 6-bit pin.

**Conclusion for I-1: CLOSED.** All three sub-parts present and each inverse-correct against `tree.rs:80`, `tag.rs:140-146`, and Go `readTag`/`readNodeDepth`.

---

## Folded Minors — status (each verified against source)

### M-2 — `chunked_md1_vector` is a Rust BUILDER, not a copyable golden file — **CLOSED**
Plan line 23 now reads: "**`chunked_md1_vector` is NOT a copyable golden file** — it is a Rust *builder function* in `me-cli/src/bundle.rs:547-585` … **Hand-build the equivalent Go `descriptor`** … it has no `.bytes.hex`/`.phrase.txt` to copy; use it only for the chunked `split`→`Reassemble` round-trip (T6/T7), not byte/string parity."
- Source: `bundle.rs:547-585` `fn chunked_md1_vector() -> Vec<String>` constructs a `Descriptor` programmatically (n=6; `PathDeclPaths::Divergent` of 6 `OriginPath`s, each 15 hardened `PathComponent { value: c*100+i+1 }`, `bundle.rs:555-564`; `Wsh→Children([SortedMulti{ k:2, indices:0..6 }])`, `bundle.rs:572-581`; `TlvSection::new_empty()`, `:582`) and returns **`md_codec::chunk::split(&d)`** at `bundle.rs:584`. Verified by direct read: NO `.bytes.hex` / `.phrase.txt` / `.descriptor.json` read; it is a builder calling `split()`. Plan's recipe (n=6, 6 divergent paths × 15 hardened comps `c*100+i+1`, wsh→sortedmulti k=2 indices 0..6, empty TLV) matches the source verbatim, including the ≥4-chunk expectation (T6/T7).

### M-3 — `wsh_multi_chunked` EXCLUDED from the single-string parity table — **CLOSED**
Plan: line 23 ("FORCE-CHUNKED — its `.phrase.txt` is a multi-chunk-format string … must be EXCLUDED from the T5 single-string `encodeMD1String == .phrase.txt` parity table"); line 78 keeps it in T3 byte-parity; line 102 ("**R0-M3: EXCLUDE force-chunked vectors (e.g. `wsh_multi_chunked`) from this single-string table** — their `.phrase.txt` is a chunk-format string, covered instead by T6/T7's chunked round-trip").
- Source: `wsh_multi_chunked.phrase.txt` =
  ```
  chunk-set-id: 0x157ae
  md1fz4awqqpqsgqpsgvyyxqql8saf74dwdyqv
  ```
  i.e. a **chunk-format string** with a `chunk-set-id:` header line — NOT a single-string form (contrast a true single `wpkh_basic.phrase.txt = md1yqpqqxqq8xtwhw4xwn4qh`). Its `.bytes.hex = 2082001821842180` (8-byte pre-chunk payload) is correctly retained for T3 byte-parity. Verified by direct `cat`. The plan's three consequences (keep in T3 byte-parity; exclude from T5 single-string; valid for T6/T7 round-trip) are all stated.

### M-4 — `.descriptor.json` loader is a custom test shim, not `json.Unmarshal` — **CLOSED**
Plan line 78: "**R0-M4: the `.descriptor.json`→Go-`descriptor` loader is a non-trivial custom test shim** (interface-bodied node kind-dispatch + a string→tag map), NOT a one-line `json.Unmarshal` (the Go AST uses interface node bodies). `encoding/json` is TinyGo-safe in `_test.go`. If a given vector's JSON is awkward to map, hand-build that descriptor instead."
- Source: the Go `node` AST is `node{ tag tag; body body }` where **`body` is an interface** (`type body interface{ isBody() }`, `md/md.go:102`) with one struct per variant (`childrenBody`/`variableBody`/`multiKeysBody`/`trBody`/`keyArgBody`/`hash256Body`/`hash160Body`/`timelockBody`/`emptyBody`, `md/md.go:104-137`). A plain `json.Unmarshal` cannot populate an interface field nor map JSON tag-strings to the `tag uint8` consts, so a custom kind-dispatch + string→tag shim is genuinely required. Confirmed by direct read.

### M-5 — validators + `readUnknownPayload` REUSE (don't re-port) — **CLOSED**
Plan line 76: "**R0-M5/M6 (reuse, don't re-port):** the three pre-emission validators (`validatePlaceholderUsage`, `validateMultipathConsistency`, `validateTapScriptTree`) and `readUnknownPayload` ALREADY exist in the Go decoder (`md/md.go`) — call them, do not re-implement. Only the encode-side range guards are net-new …"
- Source: all four exist in the shipped Go decoder — `readUnknownPayload` (`md/md.go:623`), `validatePlaceholderUsage` (`md/md.go:904`), `validateMultipathConsistency` (`md/md.go:977`), `validateTapScriptTree` (`md/md.go:1004`). Confirmed via grep. The net-new encode-side range guards (`errThresholdRange`/`errChildCount`/`errKGreaterThanN`/…) the plan does list (line 76) match the inline `tree.rs:92-138` guards. Correct.

### M-6 — `bitsToSymbols` is net-new (encode direction); `MDDataSymbols` decode-only — **CLOSED**
Plan line 104: "implement … `md.bitsToSymbols` (the ENCODE-direction bits→5-bit-symbols packer; **R0-M6: this is net-new** — `MDDataSymbols` is the decode direction (symbols→bits) and does NOT expose it)." The misleading "(if not already exposed by `MDDataSymbols`)" hedge from R0 is removed.
- Source: `MDDataSymbols(s string) ([]byte, error)` (`codex32/mddata.go:15`) takes a **string** and returns 5-bit data symbols with the 13-symbol checksum stripped — its doc-comment explicitly states it is the decode/intake direction. Only the decode-side `symbolsToBytes` (`md/md.go:867`) exists; there is NO encode-direction `bitsToSymbols`/`BitsToSymbols` anywhere in `codex32/` or `md/` (grep clean). The encode-direction left-justify is `bits_to_symbols` (`codex32.rs:23-42`, final short symbol `(val << (5-take)) & 0x1F`). Plan's "net-new" is correct.

---

## No-drift re-confirmation (R0-clean items that must STILL hold)

- **Two CRITICAL traps TDD'd before integration goldens — STILL HOLDS.** bitWriter (T1: unit tests `md/bits_test.go` from `bitstream.rs:236-407`, committed before T3.5) and canonicalize (T2: idempotence + normalization, committed before T3.5). Source `encode.rs:65-92` confirms `encode_payload` opens with `canonicalize_placeholder_indices` then the validators then `BitWriter::new()` — the plan's Task-3 call graph (line 60) mirrors this exactly. No reordering introduced by the fold.
- **Dependency order T1→T2→T3→T4→T5→T6→T7→T8 — STILL HOLDS.** No back-edges. `split` (`chunk.rs:235-244`) and `reassemble` (`chunk.rs:305-389`) both call `compute_md1_encoding_id` (T4) which calls `encode_payload` (T3); identity (T4) follows encodePayload (T3). Intact.
- **9 node BODY arms + 8 encode validators — STILL HOLDS.** `tree.rs:79-176`: KeyArg(index@kiw), Children(recurse no count), Variable(k-1·5/len-1·5/children + k,n∈1..32 & k≤n guards), MultiKeys(k-1·5/len-1·5/raw n×index@kiw), Tr(is_nums·1/[key_index@kiw if !nums]/has_tree·1/opt subtree), Timelock(32), Hash256(32×8), Hash160(20×8), Empty(∅). Plan arms 1-9 (lines 63-71) match; all now correctly written AFTER the tag.
- **Golden literals — STILL HOLD.** `wpkh_basic.bytes.hex == 2002001800` ✓ (and the 5-bit header `00100` MSB-first = first byte `0x20`, matching the plan's "Header common byte=`0x20`" pin); `wsh_with_fingerprints.bytes.hex == 204200182182142f09bd5b7ddfcafebabe` ✓. All 10 MANIFEST `.bytes.hex` files present (pkh_basic, sh_wsh_multi, tr_keyonly, wpkh_basic, wsh_divergent_paths, wsh_multi_2of2, wsh_multi_2of3, wsh_multi_chunked, wsh_sortedmulti, wsh_with_fingerprints). Verified by `cat`/`ls`.
- **M-5 (read-side symbol-aligned bit count) — STILL HOLDS.** `chunk.rs:319-328`: `payload_byte_count = (symbol_aligned_bit_count − 37) / 8` from `unwrap_string`, with the explicit in-source comment warning against `bytes.len()*8` (N=3/N=8 byte-boundary break). Plan T7 line 128 captures verbatim.
- **I-3 (bit-0 discriminator first) — STILL HOLDS.** Go `md/md.go:1207` `syms[0]&1 == 1` (chunked flag = bit 0 of symbol 0); me-cli `bundle.rs:141` documents `symbols.first() & 0x01`. Plan T7 line 128/130 mandates `syms[0]&1` first, never parse-then-catch.
- **split threshold 320 / count≤64 — STILL HOLDS.** `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64*5 = 320` (`chunk.rs:219`); `div_ceil`, `>64→ChunkCountExceedsMax` (`chunk.rs:249-254`), count=max(1,·), `bytes_per_chunk=div_ceil(len,count)` (`chunk.rs:262`), per-chunk `37+8N` bits (`chunk.rs:284`). Plan T6 line 114 matches.
- **ChunkHeader.write — STILL HOLDS.** `chunk.rs:36-57`: 37 bits `[version:4 TOP][chunked=1:1][csid:20][count-1:6][index:6]`, guards count∈1..64, index<count, csid<2^20. Version=`WF_REDESIGN_VERSION=4` (`header.rs:27`). Plan T6 line 114 matches.
- **Integrity gate (decode→re-encode→derive-csid) — STILL HOLDS.** `chunk.rs:376-388`: `decode_payload(full, full.len()*8)` → `compute_md1_encoding_id` → `derive_chunk_set_id` → compare to `expected_csid` → `ChunkSetIdMismatch`. Plan T7 line 128 matches.
- **POLYMOD_INIT 0x23181b3 — STILL HOLDS.** `codex32/mdmk.go:39` `mdmkPolymodInitLo = 0x23181b3` (with the in-source "NOT codex32's 1" warning at `:9`/`:105`); `mdRegularTargetHi=0x0`/`Lo=0x0815c07747a3392e7` (`:55-56`); `mdmkShortSyms=13`; `ValidMD` (`codex32/mdmk.go:124-127`). Plan T5/I-7 line 100 matches.
- **Header version=4 + layout — STILL HOLDS.** `header.rs:27` `WF_REDESIGN_VERSION=4`; `Header::write` = `(divergent<<4)|(version&0xF)` in 5 bits. Plan T3 line 60 matches.
- **canonicalize source shape — STILL HOLDS.** `canonicalize.rs:168+`: `check_placeholder_bounds`, `PlaceholderNotReferenced` guard (`:185-189`), identity fast-path (`:199-201`), atomic remap of tree + divergent paths + 4 TLV maps. Plan T2 "Mirror `canonicalize.rs` exactly" covers it.
- **bits_to_symbols left-justify — STILL HOLDS.** `codex32.rs:23-42` final short symbol `(val << (5-take)) & 0x1F`. Plan T5 line 100 matches.
- **derive_chunk_set_id — STILL HOLDS.** `chunk.rs:175-179` `(b0<<12)|(b1<<4)|(b2>>4)`; test `0xAB,0xCD,0xEF→0xABCDE` (`chunk.rs:199-207`). Plan T4 line 90 matches (see M-7 note below).

---

## Residual non-blocking Minors (carried from R0; NOT gate-blocking)

- **M-7 (cosmetic) — `& 0xFFFFF` no-op in `deriveChunkSetID`.** Plan line 90 still appends `& 0xFFFFF` to the derive expression. Source `chunk.rs:175-179` has NO trailing mask (value is intrinsically ≤0xFFFFF since `b0<<12` tops at 0xFF000). Harmless no-op; diverges cosmetically from "mirror exactly." Drop or comment opportunistically. NON-BLOCKING.
- **M-8 (test-strengthening) — canonicalize edge-case cases.** Plan T2.1 exercises permute + idempotence; the R0 suggested also adding explicit `PlaceholderNotReferenced` and identity-fast-path test cases (`canonicalize.rs:185-189`, `:199-201`). The plan's "Mirror `canonicalize.rs` exactly" technically covers the implementation; adding the two test cases strengthens the CRITICAL-trap coverage. NON-BLOCKING; recommended.

Neither affects byte-faithful parity nor the gate.

---

## Cleared for implementation

The plan passes the R0 gate at **0 Critical / 0 Important**. I-1 is CLOSED across all three sites; every folded Minor (M-2…M-6) is accurate against authoritative source; the fold introduced no drift to the R0-clean items. **The plan is cleared for single-implementer TDD in the worktree** (Task 0→8), to be followed by the mandatory whole-diff adversarial execution review before merge.

## Verification artifacts (throwaway; no source modified)
- `git rev-parse` on both repos: Rust `c85cd498c690…` (Cargo `0.36.0`), Go `e4ca173621f0…`.
- Direct reads: `md-codec/src/{tree.rs:75-254,411-425,653-680}`, `tag.rs:95-146`, `encode.rs:30-92`, `chunk.rs:30-86,170-210,235-334,370-390}`, `header.rs:27 + write`, `varint.rs:108-130`, `codex32.rs:23-42`, `canonicalize.rs:168-205`.
- Direct reads: `seedhammer/md/md.go:{35-154,325-354,623,867,904,977,1004,1207}`, `codex32/mddata.go:1-25`, `codex32/mdmk.go:{9,22,37-56,105,124-127}`; grep confirms reader-only `md/bits.go`, no `bitsToSymbols`.
- `me-cli/src/bundle.rs:540-587` (`chunked_md1_vector` builder), `:141` (bit-0 discriminator doc).
- `cat` of `wsh_multi_chunked`/`wpkh_basic`/`wsh_with_fingerprints` golden literals; `ls` of all 10 `.bytes.hex`.
