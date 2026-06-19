# SeedHammer #10a (Phase A) — md1 Encoder: Whole-Diff Adversarial Execution Review

**Reviewer:** opus architect (independent, non-deferrable post-implementation review)
**Date:** 2026-06-19
**Scope:** cumulative diff `e4ca173..feat/10a-md-encoder` (9 commits) in worktree
`/scratch/code/shibboleth/seedhammer-wt-10a`, HEAD `a98860e`.
**Authoritative Rust source:** `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec`
@ `c85cd49` (verified == the blueprint's pinned commit).
**Go toolchain:** `/home/bcg/.local/go/bin/go` = go1.26.4.

---

## VERDICT: GREEN

0 Critical / 0 Important. All byte-fidelity, golden-authenticity, canonicalize,
tag-write, chunk-integrity, BCH-checksum, scope, and no-regression checks pass.
Three non-blocking Minors are listed at the end for the implementer's awareness.

This gate is **meaningful and non-circular** (see §A) — the byte-exact parity
test compares an independently-built AST (from the vendored Rust `descriptor.json`)
against the vendored Rust `.bytes.hex`/`.phrase.txt` goldens, both of which I
diffed byte-for-byte against the authoritative source and confirmed identical.

---

## Mandatory explicit statements

- **(a) Goldens are authentic.** I diffed all 30 vendored golden files
  (`*.bytes.hex`, `*.phrase.txt`, `*.descriptor.json` × 10 vectors) against the
  authoritative Rust source at `md-codec/tests/vectors/` — **every one matches
  byte-for-byte** (`diff -q` MATCH on all). The pinned literals appear verbatim
  in the Rust source: `wpkh_basic.bytes.hex = 2002001800` and
  `wsh_with_fingerprints.bytes.hex = 204200182182142f09bd5b7ddfcafebabe`
  (`grep` hit in `md-codec/tests/vectors/`). The `sortedmulti` golden is
  `2082001821c22180` (carries the `0x22` sortedmulti tag bits). NOT fabricated to
  match a buggy encoder.
- **(b) canonicalize is correct.** `md/canonicalize.go` faithfully ports
  `canonicalize.rs:45-248`: first-use index assignment order, identity fast path,
  ATOMIC remap of tree indices + the inverse-perm divergent path reorder + ALL
  FOUR per-`@N` TLV maps (each remapped then re-sorted ascending). The
  nil-vs-empty distinction is preserved via `cloneSlice[T]` (returns nil for nil).
  A deliberately-permuted descriptor canonicalizes to the same bytes as its
  canonical form (`TestEncodePayloadCanonicalEquality`) and produces the same
  Rust-derived csid (see §A.3).
- **(c) The 6-bit TAG write is on all 9 arms.** `writeNode` (md/encode.go:158)
  calls `writeTag(w, n.tag)` at line 159 **before** the body `switch` (line 160),
  so the tag is written unconditionally for every body kind (keyArg, children,
  variable, multiKeys, tr, timelock, hash256, hash160, empty — all 9). This
  mirrors `tree.rs:80` (`node.tag.write(w)` as the first statement of
  `write_node`). No arm can skip it.
- **(d) Byte-exact parity is a meaningful (non-circular) gate.** The test input
  (AST) is built by an **independent JSON parser** (`loadDescriptor` /
  `buildNode` / `parsePathString`, testdata_test.go:143-) from the authentic Rust
  `descriptor.json`; the expected output is the authentic Rust `.bytes.hex` /
  `.phrase.txt`. A wrong Go encoder would diverge. Independently corroborated by
  re-deriving csid `0x157ae` from the golden bytes via SHA-256 (§A.3) and by the
  `codex32.ValidMD` independent BCH check on every emitted string.

---

## Test / vet / fuzz output observed (re-run myself, not trusted from report)

```
go test -count=1 ./...        → ALL PASS (md, codex32, mk, gui, + entire repo)
go vet ./md/... ./codex32/... → exit 0 (clean)
go vet ./...                  → exit 0 (only PRE-EXISTING bspline_test.go
                                unkeyed-literal infos, unrelated to this diff)
gofmt -l <every touched .go>  → empty (all formatted)
go build ./...                → exit 0
```

Targeted parity (verbose):
```
TestEncodePayloadGoldens         PASS (10 subtests incl. wsh_multi_chunked)
TestEncodeMD1StringGoldens       PASS (9 single-string subtests; each + ValidMD)
TestEncodePayloadCanonicalEquality PASS
TestSplit{Chunked,ForceChunked,SmallSingleChunk} PASS
TestReassemble{RoundTrip,ReorderOK,DropChunk,Duplicate,CorruptCSID,CrossSet} PASS
TestParseChunkHeaderSingleIsNotChunked PASS
TestDeriveChunkSetID / TestComputeEncodingID{Deterministic…,IsSHA256Prefix} PASS
TestCanonicalize{IdempotentOnGoldens,NormalizesPermuted,NotReferenced,…,OutOfRange} PASS
TestMDChecksumSymbolsRoundTrip / TestAssembleMD1Valid PASS
```

Fuzz (re-run myself, meaningful bursts, 0 panics/crashes):
```
FuzzEncodePayload     45s → 22,802,547 execs, 0 crashes
FuzzReassemble        45s → 26,300,782 execs, 0 crashes
FuzzParseChunkHeader  45s → 27,596,370 execs, 0 crashes
```
`FuzzEncodePayload` is not just no-panic: on the decode-success path it asserts a
decoded descriptor always re-encodes AND that encode is idempotent (canonical
stability) — the exact property a canonicalize/encoder bug would break. 22.8M
execs over the decoder-accepted input space, 0 failures.

`TestAllocs` (mentioned in the prompt) does not exist in this diff — it is a
pre-existing GUI test (`gui/*_test.go`), unrelated. No alloc-budget test was in
scope for #10a.

---

## §A. Golden authenticity & non-circularity (the most important check)

1. **Byte-for-byte diff vs source.** All 30 files MATCH (script over all 10
   vectors × {bytes.hex, phrase.txt, descriptor.json}). No file altered.
2. **Independent AST construction.** `loadDescriptor` parses `descriptor.json`
   into the Go AST via its own JSON schema decoder — no encoder involvement. So
   `TestEncodePayloadGoldens` (encode AST → compare to `.bytes.hex`) is a true
   inverse-of-Rust parity gate.
3. **Cross-tool csid re-derivation (independent of both encoders).** I computed
   `SHA-256(wsh_multi_chunked golden payload bytes 2082001821842180)[0:3]`, took
   the top-20 bits MSB-first → `0x157ae`. This matches BOTH the test's hardcoded
   constant (chunk_test.go:104) AND the Rust source's own annotation in
   `wsh_multi_chunked.phrase.txt` (`chunk-set-id: 0x157ae`). Proves the
   csid/identity path is byte-correct and the constant is Rust-sourced, not
   fabricated.

## §B. bitWriter byte-fidelity (I-2 trap) — CORRECT

`md/bits.go` bitWriter is a line-faithful port of `bitstream.rs:18-84`: identical
MSB-first packing (`freeInByte`/`chunk`/`shift`/`byteShift`), masking, `bitLen`,
and `into_bytes` semantics (low-pad final byte). `reEmitBits` matches
`re_emit_bits` (1-byte chunks, bit-limited source reader). The change to
`md/bits.go` is **purely additive** — the existing `bitReader` (the shipped
decoder's reader) is byte-untouched (verified via `git diff`).

The implementer's plan-prose correction is **valid**: header value 4 = `0b00100`
written as 5 bits, then 3 zero bits → byte `0b00100_000` = `0x20`. This is the
common-case header byte and matches `header.rs:114-125`. The unit tests pin
**independently hand-computed** values matching the Rust test goldens
(`bitstream.rs:236-407`), not values the impl merely happens to produce
(e.g. `TestBitWriterTwo5BitValues` pins `[0xf8 0x40]`,
`TestBitWriterPlanPin5then3` pins `[0x20]`).

## §C. canonicalize (I-1 trap) — CORRECT

Verified arm-by-arm against `canonicalize.rs`:
- `walkCollectFirst` (rs:45-98): is_nums skip, MultiKeys raw-index registration,
  bounds-guarded `seen` access — matches.
- `remapIndices` (rs:102-139): all index-bearing arms remapped, is_nums skip,
  recursive — matches.
- Divergent reorder (rs:206-219): `newPaths[newIdx] = oldPaths[inverse[newIdx]]`
  matches the Rust inverse-perm push exactly (go: chunk canonicalize.go:68-80).
- All four TLV maps remapped + re-sorted ascending
  (`remap{UseSite,FP,Pub,Origin}Vec` ↔ `remap_tlv_vec`).
- Identity fast path + `errPlaceholderNotReferenced` + `errPlaceholderRange`
  (out-of-range pre-check) — matches.
- nil-vs-empty preserved (`cloneSlice` returns nil for nil; `TestCanonicalize…`
  uses `reflect.DeepEqual`, which would catch a collapse).

`TestCanonicalizeNormalizesPermuted` exercises the full atomic remap (tree +
divergent + FP) AND asserts the input is untouched (clone semantics).
`TestEncodePayloadCanonicalEquality` proves permuted→canonical byte equality.

## §D. Per-node TAG write — CORRECT (see statement (c)).

## §E. Encoder structural fidelity — CORRECT

Spot-checked every writer against its Rust counterpart and the decoder it inverts:
- `kiw` = `32 - clz(n-1)`, clamped 0 at n≤1 (encode.go:33 ↔ encode.rs:37-41;
  matches decoder md.go:842).
- is_nums kiw-suppression in `writeNode` trBody (encode.go:205-208 ↔
  tree.rs:151-154).
- MultiKeys raw indices at kiw width; (k-1)/(n-1) in 5 bits; k/n range + k≤n
  guards (encode.go:187-202 ↔ tree.rs:115-139).
- varint: `bits_needed` form, ≤14 short path, L=15 extension
  (L_high 4b + low 14b + high L_high b), overflow at L_high>15
  (encode.go:51-73 ↔ varint.rs:15-42).
- header low-nibble version `(divergent<<4)|(version&0xf)` in 5 bits
  (encode.go:77-80 ↔ header.rs:30-33).
- TLV: sorted-by-tag (stable; insertion sort ↔ Rust stable sort_by_key),
  per-entry build into sub-bitWriter, empty-reject, strict-ascending-idx,
  `[tag:5][varint(bitLen)][reEmitBits]`, unknown-TLV re-emit verbatim
  (encode.go:246-352 ↔ tlv.rs:86-208). FP=4 bytes, xpub=65 bytes — matches.
- ChunkHeader version is the TOP nibble; single Header version is the LOW nibble
  — distinct, both correct.

`FuzzEncodePayload` round-trip (decode→encode→decode→encode idempotent) holds
across 22.8M execs — covers renderable shapes beyond the 10 goldens.

## §F. chunk write/read + integrity — CORRECT

- split: threshold 320 bits, count = max(1, ceil(bytes*8/320)), >64 → error,
  byte-boundary slices, per-chunk `37 + 8N` bits (chunk.go:110-167 ↔
  chunk.rs:235-290).
- **M-5 confirmed**: `payloadByteCount = (symBits - 37) / 8` (chunk.go:221), NOT
  `len(b)*8` — matches chunk.rs:328 exactly.
- **I-3 confirmed**: `ParseChunkHeader` consults `syms[0]&1` FIRST (chunk.go:182),
  returning `Chunked:false` for single strings without a 37-bit parse.
- Reassemble: consistency (version/csid/count all-equal), completeness
  (len==count), sort-by-index + gap check, concat, decode, integrity csid gate.
  Negative cases map to typed errors and there is **no silent-accept path**:
  drop/dup → `errChunkSetIncomplete`; corrupt-but-consistent csid →
  `errChunkSetIDMismatch` (integrity gate fires); cross-set → `errChunkSetInconsist`;
  index gap → `errChunkIndexGap`. Note: Rust `reassemble` calls `decode_payload`
  (which itself runs all validators inline, decode.rs:56-69); Go calls
  `decodePayloadValidated` — correct parity (Go's two-function split).

## §G. MDChecksumSymbols (I-7) — CORRECT

`MDChecksumSymbols` builds the BCH engine **identically to the verify path**
`verifyMDMK`/`ValidMD` (mdmk.go:92-127): `newShortChecksum().generator`,
`residue = unpackSyms(0, mdmkPolymodInitLo, 13)` where
`mdmkPolymodInitLo = 0x23181b3` (= the stated POLYMOD_INIT, regular-only),
`target = md regular target`, then `inputHRP("md") + inputData + inputTarget` so
the residue IS the checksum. This is the analogue of `MKChecksumSymbols`. Every
emitted string passes `codex32.ValidMD` (`TestEncodeMD1StringGoldens`,
`TestAssembleMD1Valid`, all split tests) and equals the Rust `.phrase.txt`.

## §H. No-regression + scope + TinyGo safety (I-9) — CORRECT

- `md/md.go` is **NOT in the diff** — the shipped single-md1 decode/display path
  is byte-untouched. `md/bits.go` is additive-only.
- mk1, ms1 (codex32), and gui all pass unchanged. **gui/ is untouched** — the
  chunked-md1 GUI refusal (#10b) is NOT modified.
- Public surface is minimal/justified: `ChunkHeader`, `ParseChunkHeader`,
  `Reassemble` (md); `MDChecksumSymbols`, `AssembleMD1` (codex32). `split`,
  `encodePayload`, `encodeMD1String`, `computeEncodingID` stay unexported.
- Production import hygiene: no `math/big`/`reflect`/`encoding/json` in any
  non-test md/codex32 file (those appear only in `_test.go`). `math/bits` (stdlib
  bit-twiddling, not `math/big`) and `crypto/sha256` are fine.
- `cloneSlice[T any]` uses only make/copy/len — monomorphized, no runtime
  reflection; TinyGo-safe (TinyGo supports generics since 0.27; go.mod is 1.25).

## §I. Security/faithfulness spine — CORRECT

No secret material touched (public md1/descriptor only; SHA-256 over the public
payload). Faithful-or-refuse holds: every encode arm with a range/structural
violation returns a typed error (k/n range, k≤n, alt-count, path depth, divergent
count, varint overflow, empty-TLV, override-order) — there is no path that
silently mis-encodes a card a decoder can't read back.

---

## Minors (non-blocking)

- **M-1 (informational, design choice).** Rust derives kiw from `Descriptor.n`
  (`key_index_width()` → `self.n`) while writing the `n-1` wire field from
  `PathDecl.n`; the Go encoder keys BOTH off `pathDecl.n` (encode.go:406,
  writePathDecl). For every well-formed descriptor (and everything the decoder
  produces, where `descriptor.n == pathDecl.n` is guaranteed — md.go:857,
  decode.rs:49) this is byte-identical to Rust. The only divergence is a
  hand-built AST with `descriptor.n != pathDecl.n`, which is malformed input that
  Rust itself handles inconsistently. Go's single-source-of-truth is arguably more
  robust; not a regression, not exercised. Optionally add a guard rejecting
  `descriptor.n != pathDecl.n` for defense-in-depth.
- **M-2 (informational).** `cloneSlice[T any]` is the first generic in
  firmware-production code (no other firmware-production generic precedent found).
  TinyGo ≥0.27 supports generics and `sort` is already firmware precedent (below),
  so this is fine; flagged only because a hardware-target (TinyGo) build could not
  be run in this review environment — recommend a one-time `tinygo build` of the
  controller target to confirm before tagging.
- **M-3 (informational).** `md/canonicalize.go` imports `sort` and uses
  `sort.SliceStable` (reflect-backed) on the encoder production path. This is NOT
  a new TinyGo risk: `sort` is already imported by firmware-production packages in
  the controller build (`bip39/bip39.go`, `gui/slip39_polish.go`,
  `engrave/engrave.go`, `slip39/`, `bc/fountain/`, `font/bitmap/`). The entry
  counts are tiny (≤4 sparse + a few unknowns), so the hand-rolled insertion sorts
  used elsewhere in the diff (`sortTLVEntriesByTag`, Reassemble's index sort) could
  optionally replace the four `sort.SliceStable` calls for consistency, but it is
  not required for correctness or TinyGo-compatibility.

---

## Bottom line

The diff is a faithful, byte-exact Go port of the Rust md1 encoder + identity +
chunk codec. The byte-parity gate is non-circular (independently-built input vs
authentic source goldens, cross-confirmed by an independent csid SHA-256
derivation and the independent `ValidMD` BCH check). canonicalize, the per-node
tag write, the M-5 payload-byte-count, the I-3 discriminator, and the integrity
gate are all correct. No Critical or Important findings. Three informational
Minors. **VERDICT: GREEN.**
