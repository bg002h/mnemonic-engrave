# #10a (Phase A) Implementation Plan — md1 ENCODER + identity + chunk write/read (headless codec)

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development or executing-plans. Steps use `- [ ]` checkboxes. TDD throughout: failing test → run-fail → minimal impl → run-pass → commit.

**Goal:** Port a byte-faithful md1 ENCODER (the inverse of the shipped `md` decoder) + identity + chunk write/read into the SeedHammer fork, proven by byte-exact parity against the constellation's Rust goldens — the shared headless foundation for T5 (bundle), T6 (md1-from-seed), and #10b (GUI display).

**Architecture:** New MSB-first `bitWriter` (`md/bits.go`), `encodePayload` + sub-writers as the structural inverse of `decodePayload` (`md/md.go`) mirroring Rust `md-codec` v0.36.0 `encode.rs`, a mandatory `canonicalize` port, identity (`computeEncodingID`/`deriveChunkSetID`), chunk `split`/`Reassemble`, and `codex32.MDChecksumSymbols`/`assembleMD1`. Follows the proven T4 mk1-encoder recipe (`mk/encode.go`); md1 differs only by bit-packing (needs `bitWriter`) and regular-BCH-only.

**Tech stack:** Go (host tests via `/home/bcg/.local/go/bin/go`; TinyGo/RP2350 target — keep imports TinyGo-safe: no `math/big`, `reflect`, `encoding/json` in production paths). Module `seedhammer.com`.

**Spec:** `design/SPEC_seedhammer_10_md_encoder.md` (GREEN @ R0, `a8d697f`). **Blueprint (port reference + citations):** `design/agent-reports/seedhammer-10-md-encoder-architect-blueprint.md`. **R0 review (Minors folded below):** `design/agent-reports/seedhammer-10-md-encoder-spec-review-R0.md`.

**Rust source to port from** (`descriptor-mnemonic/crates/md-codec` @ `c85cd49`): `src/{bitstream,encode,canonicalize,identity,chunk,tlv,tree,varint,origin_path,use_site_path,header,tag}.rs`. **Go decoder to invert:** `md/md.go`. **mk1 precedent:** `mk/encode.go`, `codex32/mkencode.go`.

---

## Task 0: Worktree + vendor goldens + baseline

**Files:** Create worktree; vendor `md/testdata/`.

- [ ] **Step 1:** From `/scratch/code/shibboleth/seedhammer`, create an isolated worktree off the current fork HEAD: `git worktree add ../seedhammer-wt-10a -b feat/10a-md-encoder e4ca173` (sibling-dir convention, matching `../seedhammer-wt-bip39`; if `.worktrees/` is gitignored prefer that). Work entirely in the worktree.
- [ ] **Step 2:** Verify clean baseline: `/home/bcg/.local/go/bin/go test ./md/... ./codex32/... ./mk/... ./bip380/...` → all pass. If not, STOP and report BLOCKED.
- [ ] **Step 3:** Vendor the constellation golden vectors into `md/testdata/vectors/` — copy the subset of `descriptor-mnemonic/crates/md-codec/tests/vectors/*.{bytes.hex,phrase.txt,descriptor.json}` named in the 10-entry MANIFEST, plus `wsh_multi_chunked`. **R0-M3: `wsh_multi_chunked` is FORCE-CHUNKED** — its `.phrase.txt` is a multi-chunk-format string, so it is valid for the T3 `.bytes.hex` byte-parity (the pre-chunk payload) and the T6/T7 chunked round-trip, but must be EXCLUDED from the T5 single-string `encodeMD1String == .phrase.txt` parity table. **R0-M2: `chunked_md1_vector` is NOT a copyable golden file** — it is a Rust *builder function* in `me-cli/src/bundle.rs:547-585` that constructs a 6-key wsh-sortedmulti (15-deep divergent paths, ≥4 chunks) and calls `split()`. **Hand-build the equivalent Go `descriptor`** in the test (it has no `.bytes.hex`/`.phrase.txt` to copy); use it only for the chunked `split`→`Reassemble` round-trip (T6/T7), not byte/string parity. Add a short `md/testdata/README.md` noting the source crate + commit `c85cd49`.
- [ ] **Step 4:** Commit (signed + DCO, author Brian Goss; trailer `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`; explicit paths). Message: `md: vendor md-codec golden vectors for the encoder (#10a)`.

---

## Task 1: MSB-first `bitWriter` + `reEmitBits` (I-2, CRITICAL)

**Files:** Modify `md/bits.go` (currently reader-only); Test `md/bits_test.go`.

Port `bitstream.rs:11-84` (`BitWriter`) + `bitstream.rs:220-230` (`re_emit_bits`). MSB-first; `intoBytes()` returns the in-progress final byte with LOW bits zero-padded.

- [ ] **Step 1: Failing unit tests FIRST (M-1).** Port the `bitstream.rs:236-407` writer unit tests into `md/bits_test.go`: e.g. writing `0x2` in 5 bits then `0x00` in 3 bits yields `[0x20]` and `bitLen()==8`; writing across a byte boundary; the golden `wpkh_basic` payload prefix `0x20 0x02` from sequential `write` calls; `reEmitBits` of a non-byte-aligned (`bitLen` not multiple of 8) payload reproduces the exact bits (round-trip a known 13-bit value through `intoBytes`→`reEmitBits`→read). Also promote the existing throwaway `testBitWriter` (`md/md_test.go:86-117`) algorithm as the reference.
- [ ] **Step 2: Run → FAIL** (`bitWriter` undefined): `/home/bcg/.local/go/bin/go test ./md/ -run TestBitWriter -v`.
- [ ] **Step 3: Implement** `bitWriter` in `md/bits.go`: fields `bytes []byte`, `nbits int`; `write(value uint64, count int)` packs MSB-first; `bitLen() int`; `intoBytes() []byte` (final partial byte already low-padded). `reEmitBits(w *bitWriter, payload []byte, bitLen int)` writes the first `bitLen` bits of `payload` MSB-first (mirror `bitstream.rs:220-230`). Mirror the exact algorithm of the proven `testBitWriter`.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `md: MSB-first bitWriter + reEmitBits (#10a, I-2)`.

---

## Task 2: `canonicalize` (I-1, CRITICAL)

**Files:** Create `md/canonicalize.go`; Test `md/canonicalize_test.go`.

Port `canonicalize_placeholder_indices` (`canonicalize.rs:168-…`, incl. `remap_indices`/`walk_collect_first`): walk the tree in canonical order assigning placeholder indices `@0,@1,…` by first-use, then remap the tree key indices, the divergent path-decl entries, AND all four per-@N TLV maps (use-site-path-overrides, fingerprints, pubkeys, origin-path-overrides) atomically. Operates on a clone of the descriptor.

- [ ] **Step 1: Failing tests.** In `md/canonicalize_test.go`: (a) **idempotence** — decode each renderable golden `.phrase.txt` (the decoder yields a canonical AST), `canonicalize` it, assert the AST is unchanged (deep-equal). (b) **normalization** — hand-build a descriptor whose placeholder indices are permuted (e.g. tree uses `@1` before `@0`), `canonicalize`, assert indices are re-assigned by first-use AND every per-@N TLV entry moved with its key. (c) the resulting `encodePayload` (after Task 3) of the permuted-then-canonicalized form equals that of the canonical form — add this assertion once Task 3 lands.
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `md/canonicalize.go`: `canonicalize(d *descriptor) *descriptor` (returns a canonicalized clone). Mirror `canonicalize.rs` exactly: collect first-use order, build the old→new index map, apply to tree `KeyArg`/`MultiKeys`/`Tr` indices, the `pathDecl` divergent entries, and each TLV map (re-key + re-sort ascending by new index).
- [ ] **Step 4: Run → PASS** (idempotence + normalization; the encodePayload-equality assertion is wired in Task 3).
- [ ] **Step 5: Commit** — `md: canonicalize_placeholder_indices port (#10a, I-1)`.

---

## Task 3: `encodePayload` + sub-writers + encode validators (the core)

**Files:** Create `md/encode.go`; Test `md/encode_test.go`.

`encodePayload(d *descriptor) ([]byte, int, error)` mirroring `encode.rs:65-92`: `d = canonicalize(d)` → `validatePlaceholderUsage` + (if overrides) `validateMultipathConsistency` + (if Tr) `validateTapScriptTree` → `bitWriter`; write Header(5b, `(divergent<<4)|(version&0xF)`, version=4) → `pathDecl.write` → `useSitePath.write` → `writeNode(tree, kiw)` → `tlvSection.write(kiw)` → return `(bw.intoBytes(), bw.bitLen(), nil)`. Each sub-writer is the inverse of its `read*` in `md/md.go`.

`writeNode` must, for EVERY node, **write the node's 6-bit TAG FIRST** (`node.tag.write(w)`, `tree.rs:80` → `tag.rs:140-146`; the inverse of Go's `readTag` `md/md.go:81-101`) — THEN the body. **R0-I1: the tag write is mandatory and applies to all arms; without it the wire desyncs (caught only late by the `sortedmulti=22b` pin + integration goldens, defeating the M-3 enumeration's purpose).** The 9 node BODIES (M-3 — enumerate explicitly, inverting `readNodeDepth` `md/md.go:329-489`, mirroring `tree.rs:79-176`), each written AFTER its tag:
1. **KeyArg** — `index` at `kiw` bits (`tree.rs:82-84`).
2. **Children** — recurse each child, no count prefix (`tree.rs:85-89`).
3. **Variable (Thresh)** — `(k-1)` 5b, `(len-1)` 5b, children; enforce `k,n∈1..=32` & `k≤n` (errs `ThresholdOutOfRange`/`ChildCountOutOfRange`/`KGreaterThanN`, `tree.rs:90-114`).
4. **MultiKeys** — `(k-1)` 5b, `(len-1)` 5b, then **raw `kiw`-width indices** (NOT child nodes) (`tree.rs:115-139`).
5. **Tr** — `is_nums` 1b; if `!is_nums` write `key_index` at `kiw` (normalize to 0 under NUMS); `has_tree` 1b; optional subtree (`tree.rs:140-159`).
6. **Timelock** — 32b (`tree.rs:160-162`).
7. **Hash256Body** — 32 bytes ×8b (`tree.rs:163-167`).
8. **Hash160Body** — 20 bytes ×8b (`tree.rs:168-172`).
9. **Empty** — nothing (`tree.rs:173`).
Helpers: `writeTag` (6-bit primary tag, inverse of `readTag` `md/md.go:81-101`, codes table `tag.rs:99-137` ↔ Go `md.go:38-75`; ext `0x3F` unused in v0.30), `header.write`, `pathDecl.write` (`(n-1)`5b + Shared/Divergent, depth 4b max 15, per-comp hardened1b+LP4varint, `origin_path.rs`), `useSitePath.write` (has_multipath 1b; if set `(alt_count-2)`3b range 2..=9 + alts; wildcard_hardened 1b — read the `hasMultipath` flag not `len>0`, `use_site_path.rs:80-96`), `tlvSection.write` (per-entry sub-bitstreams → sorted-by-tag ascending → `[tag:5][writeVarint(bitLen)][reEmitBits(payload,bitLen)]`; idx strictly ascending `OverrideOrderViolation`; empty `EmptyTlvEntry`; unknown TLVs re-emitted via `reEmitBits`, `tlv.rs:86-208`), `writeVarint` (LP4-ext, lengths in BITS, `varint.rs:15-42`), `kiw(n) = 32 - bits.LeadingZeros32(n-1)` clamp-0 at n∈{0,1} (`encode.rs:37-41`, mirror `md/md.go:842`).

- [ ] **Step 1: Failing per-writer bit-cost unit tests FIRST (M-1, lock before integration goldens).** Port the bit-cost pins from Rust: `writeTag`=6b (single tag write asserts `bitLen()==6`, R0-I1); `writeVarint(0)`=4b, `(1)`=5b, `(84)`=11b (`varint.rs:110-128`); BIP84 origin path=26b, with n-prefix=31b (`origin_path.rs:184-189,243-267`); `<0;1>/*` use-site=16b (`use_site_path.rs:134-139`); `sortedmulti(2-of-3)` MultiKeys at n=3=22b (`tree.rs:411-425`); Header common byte=`0x20`. Each asserts `bw.bitLen()` after the single writer call.
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `md/encode.go` per above (invert each `md/md.go` reader; port each `tree.rs`/`*.rs` writer). **R0-M5/M6 (reuse, don't re-port):** the three pre-emission validators (`validatePlaceholderUsage`, `validateMultipathConsistency`, `validateTapScriptTree`) and `readUnknownPayload` ALREADY exist in the Go decoder (`md/md.go`) — call them, do not re-implement. Only the encode-side range guards are net-new; define typed encode errors (`errThresholdRange`, `errChildCount`, `errKGreaterThanN`, `errOverrideOrder`, `errEmptyTLV`, `errVarintOverflow`, `errPathDepth`, `errAltCount`).
- [ ] **Step 4: Run per-writer tests → PASS.**
- [ ] **Step 5: PRIMARY GATE — byte-exact golden parity (§5.1).** Add `TestEncodePayloadGoldens`: for each vendored vector, build the `descriptor` from `.descriptor.json`, call `encodePayload`, assert `hex(bytes) == <name>.bytes.hex` AND `bitLen` matches. **R0-M4: the `.descriptor.json`→Go-`descriptor` loader is a non-trivial custom test shim** (interface-bodied node kind-dispatch + a string→tag map), NOT a one-line `json.Unmarshal` (the Go AST uses interface node bodies). `encoding/json` is TinyGo-safe in `_test.go`. If a given vector's JSON is awkward to map, hand-build that descriptor instead. Pin literals: `wpkh_basic`→`2002001800`, `wsh_with_fingerprints`→`204200182182142f09bd5b7ddfcafebabe`. Wire the Task-2 `encodePayload(permuted)==encodePayload(canonical)` assertion here too.
- [ ] **Step 6: Run → PASS** (all manifest vectors byte-exact).
- [ ] **Step 7: Commit** — `md: encodePayload + sub-writers + validators, byte-exact vs Rust goldens (#10a, I-8)`.

---

## Task 4: identity (`computeEncodingID`, `deriveChunkSetID`)

**Files:** Modify `md/encode.go` (or `md/identity.go`); Test `md/identity_test.go`.

- [ ] **Step 1: Failing tests.** `deriveChunkSetID([]byte{0xAB,0xCD,0xEF,…})==0xABCDE` (`chunk.rs:199-207`); `computeEncodingID` is deterministic for a fixed descriptor and **path-sensitive** (two descriptors differing only in a derivation path yield different ids) (`identity.rs:301-322`).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `computeEncodingID(d *descriptor) ([16]byte, error)` = `sha256(encodePayload(d).bytes)[0:16]` (`identity.rs:39-45`); `deriveChunkSetID(id [16]byte) uint32 = ((uint32(id[0])<<12)|(uint32(id[1])<<4)|(uint32(id[2])>>4)) & 0xFFFFF` (`chunk.rs:175-179`).
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `md: identity computeEncodingID + deriveChunkSetID (#10a)`.

---

## Task 5: `codex32.MDChecksumSymbols` + `assembleMD1` + `encodeMD1String` (single-string)

**Files:** Create `codex32/mdencode.go`; Modify `md/encode.go` (add `encodeMD1String`); Test `codex32/mdencode_test.go`, `md/encode_test.go`.

`MDChecksumSymbols` = the regular-only analogue of `MKChecksumSymbols` (`codex32/mkencode.go:18-55`) using `mdRegularTargetHi/Lo` + `mdmkPolymodInitLo` (`codex32/mdmk.go:39,55-56`) — **POLYMOD_INIT=0x23181b3, NOT codex32's 1 (I-7)**. `assembleMD1(dataSyms []byte) string` mirrors `assembleMK1` (`mk/encode.go:297-312`) with HRP `"md1"`, regular checksum only. `encodeMD1String(d) (string,error)` = `encodePayload` → `bits_to_symbols` (5b, final symbol left-justified/low-zero-padded, `codex32.rs:23-42`) → `assembleMD1`.

- [ ] **Step 1: Failing tests.** `codex32/mdencode_test.go`: for a known data-symbol vector, `MDChecksumSymbols` produces 13 symbols and `ValidMD("md1"+data+checksum)==true` (`codex32/mdmk.go:124-127`). `md/encode_test.go` `TestEncodeMD1StringGoldens`: for each SINGLE-STRING golden, `encodeMD1String(decode(phrase)) == <name>.phrase.txt` AND `ValidMD(result)`. **R0-M3: EXCLUDE force-chunked vectors (e.g. `wsh_multi_chunked`) from this single-string table** — their `.phrase.txt` is a chunk-format string, covered instead by T6/T7's chunked round-trip.
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `codex32/mdencode.go` + `md.encodeMD1String` + `md.bitsToSymbols` (the ENCODE-direction bits→5-bit-symbols packer; **R0-M6: this is net-new** — `MDDataSymbols` is the decode direction (symbols→bits) and does NOT expose it).
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `codex32: MDChecksumSymbols + assembleMD1; md: encodeMD1String single-string parity (#10a, I-7)`.

---

## Task 6: `ChunkHeader.write` + `split` (chunked write)

**Files:** Modify `md/encode.go` (or `md/chunk.go`); Test `md/chunk_test.go`.

`ChunkHeader{Version, Chunked, ChunkSetID uint32, TotalChunks, ChunkIndex int}` + `.write(bw)` = 37 bits `[version:4][chunked=1:1][csid:20][count-1:6][index:6]` (version in TOP 4 bits, **distinct from the single Header**, `chunk.rs:32-57`); guards count∈1..=64, index<count, csid<2^20. `split(d) ([]string, error)` (`chunk.rs:235-290`): `encodePayload` → `computeEncodingID`→`deriveChunkSetID`; threshold `SINGLE_STRING_PAYLOAD_BIT_LIMIT=320`; `count=max(1,ceil(payloadBytes*8/320))`, `>64`→err; `bytesPerChunk=ceil(len/count)`; per chunk i: `bitWriter` → header.write → payload bytes ×8b → `bits_to_symbols(chunkBits=37+8*len)` → `assembleMD1`.

- [ ] **Step 1: Failing tests.** For `wsh_multi_chunked` and `chunked_md1_vector`: `split(d)` returns `count` chunks (count≥2, and ≥4 for `chunked_md1_vector`), each `ValidMD`, each `ParseChunkHeader` (Task 7) reports the same csid & total and indices `0..count-1`. A small descriptor → `split` produces 1 chunk (count==1 path). csid equals `deriveChunkSetID(computeEncodingID(d))`.
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `ChunkHeader.write` + `split`.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `md: ChunkHeader.write + split (chunked write) (#10a)`.

---

## Task 7: `ParseChunkHeader` + `Reassemble` (chunked read + integrity)

**Files:** Modify `md/encode.go`/`md/chunk.go` + `md/md.go` (route chunked); Test `md/chunk_test.go`.

`ParseChunkHeader(s string) (ChunkHeader, error)` (public, mirrors `mk.Header` shape `mk/mk.go:48-56`): unwrap → read 37-bit header; **the single/chunked discriminator is `syms[0]&1` (bit 0), used first — never via this parse-then-catch (I-3)**. `Reassemble(strs []string) (*descriptor, error)` mirroring `reassemble` (`chunk.rs:305-389`): per chunk `unwrapString`→`(bytes, symbolAlignedBitCount = 5*dataSymCount)`; read header; **`payloadByteCount = (symbolAlignedBitCount - 37) / 8` (M-5 — use the symbol-aligned bit count from unwrap, NOT `len(bytes)*8`)**; consistency (version/csid/count all-equal → `errChunkSetInconsistent`); completeness (len==count, sorted indices 0..count-1 no gaps → `errChunkSetIncomplete`/`errChunkIndexGap`); concat payload bytes; `decodePayload(full, len(full)*8)` (TLV-rollback tolerates ≤7 trailing zero bits, `md/md.go:549-554`); **integrity: `deriveChunkSetID(computeEncodingID(decoded)) == headerCsid` else `errChunkSetIdMismatch` (`chunk.rs:378-386`)**.

- [ ] **Step 1: Failing tests.** Round-trip: `split(d)`→`Reassemble`→`Decode`-equal to `d` for `wsh_multi_chunked` + `chunked_md1_vector`; drop a chunk → `errChunkSetIncomplete`; reorder → still OK (sorted); duplicate index → incomplete; flip a header csid bit → `errChunkSetIdMismatch`; mix a chunk from a different set → `errChunkSetInconsistent`. `ParseChunkHeader` on a single md1 → reports `Chunked==false` (via bit-0), and `Reassemble` is only called when bit-0 set.
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `ParseChunkHeader` + `Reassemble` + the `unwrapString` helper exposing the symbol-aligned bit count. Do NOT yet change the GUI refusal (that's #10b); only expose the codec API.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `md: ParseChunkHeader + Reassemble (chunked read + integrity gate) (#10a, I-3,M-5)`.

---

## Task 8: No-regression + fuzz (I-9, §5.7-8)

**Files:** Test only.

- [ ] **Step 1:** `/home/bcg/.local/go/bin/go test -count=1 ./...` + `TestAllocs` → all green; existing single-md1 decode/display, mk1, ms1 unchanged. `go vet ./md/... ./codex32/...` clean (vs baseline); `gofmt -l` on touched files empty.
- [ ] **Step 2: Add fuzz harnesses** `FuzzEncodePayload` (random decoded descriptors → encode → decode round-trip, no panic), `FuzzReassemble`, `FuzzParseChunkHeader` (arbitrary strings, no panic, typed errors only). Run each ≥1M execs.
- [ ] **Step 3: Run → 0 panics.**
- [ ] **Step 4: Commit** — `md: no-regression + fuzz harnesses for the encoder (#10a, I-9)`.

---

## Acceptance (the GREEN bar for the exec review)
- **PRIMARY:** `encodePayload` byte-exact vs all `.bytes.hex` goldens (Task 3.5); `encodeMD1String` == `.phrase.txt` for singles (Task 5).
- Chunked `split`→`Reassemble`→`Decode` round-trip; drop/reorder/dup/corrupt-csid/cross-set all caught with the correct typed error (Task 7).
- `deriveChunkSetID`→`0xABCDE`; `computeEncodingID` deterministic + path-sensitive (Task 4).
- canonicalize idempotent + normalizing; `encodePayload(permuted)==encodePayload(canonical)` (Task 2/3).
- Full suite + `TestAllocs` green; fuzz 0 panics; vet/gofmt clean; no shipped behavior changed (Task 8).
- All 9 SPEC invariants demonstrably held; the 2 CRITICAL traps (canonicalize, bitWriter) proven by their own tests before integration goldens.

## Self-review (author, pre-R0)
- Spec coverage: A1→T1, A2→T3, A3→T2, A4→T4, A5→T6, A6→T7, A7→T5; B1-B3 are #10b (out of scope here). ✓
- Minors: M-1 (per-writer bit-cost tests first) → T1.1, T3.1; M-3 (9 Body arms) → T3 enumerated; M-4 (two distinct chunked fixtures) → T0.3; M-5 (symbol-aligned bit count) → T7.3. M-2 is Phase B (#10b). ✓
- No placeholders; types consistent (`descriptor`/`node`/`bitWriter`/`ChunkHeader` used uniformly); each code step names the Rust source to port + the Go reader to invert. ✓

## Gate
This plan MUST pass opus R0 to 0C/0I before code; fold → persist verbatim to `design/agent-reports/` → re-dispatch after every fold until GREEN. Then single-implementer TDD in the worktree → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h.
