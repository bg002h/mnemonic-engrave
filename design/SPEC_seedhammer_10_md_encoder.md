# SPEC — #10: md1 ENCODER + chunked reassembly/integrity + wallet-policy xpub-expansion

**Status:** for opus R0 gate (must reach 0C/0I before any code).
**Fork base:** `e4ca173` (T4 shipped). **Fork-side only; no upstream PR.**
**Feeds from:** `design/agent-reports/seedhammer-10-md-encoder-architect-blueprint.md` (the authoritative design + wire-format citations), `design/cycle-prep-recon-T5-bundle-sequencing.md` + the two T5 recon reports. **Cite Rust source SHAs:** `md-codec` v0.36.0 (`descriptor-mnemonic` @ `c85cd49`), `mk-codec` v0.4.0, `me-cli` v0.3.0 (`mnemonic-engrave` @ `2ee44ad`).

## 1. Why / context
The user directive (2026-06-19): build #10 **first**, at full fidelity, to faithfully mirror the m\* constellation / `me bundle` behavior — so that T5 (bundle sequencing) and T6 (flagship: derive ms1+mk1+md1 from one seed, engrave all) both get faithful md1 behavior with no compromise. The deciding fact (source-verified): md1 set-INTEGRITY requires the full md DECODER + RE-ENCODER (the md1 chunk_set_id is *derived from the re-encoded canonical payload* — `md-codec/chunk.rs:378-386` → `identity.rs:39-45`), so there is **no header-only shortcut**; #10 cannot reassemble a chunked md1 set without the encoder. The shipped Go side has the md1 DECODER (`md/md.go`, ~1400 LOC) but **no encoder, no bitWriter, no md checksum-symbol generator, and no md1 chunk handling** (chunked md1 is detected + refused, `md/md.go:1207` `ErrChunkedUnsupported`).

This cycle ports the md1 ENCODER as the faithful inverse of the shipped decoder (the proven T4 mk1-encoder recipe), plus the chunk write/read paths, identity, and the wallet-policy xpub-expansion → descriptor display. The mk1 encoder (`mk/encode.go`, T4) is the precedent: encoder = verified inverse of the shipped decoder, deterministic SHA-derived csid, per-chunk BCH via a codex32 checksum-symbol helper, round-trip parity gate. md1 differs only in being **bit-packed** (needs a `bitWriter`) and **regular-BCH-only**.

## 2. Scope

### IN (this cycle)
**Phase A — headless md codec (the shared core for #10 + T5 + T6):**
- A1. `md/bits.go`: MSB-first `bitWriter` (`write(value,count)`, `bitLen()`, `intoBytes()`) — byte-exact port of `bitstream.rs:11-84` — plus `reEmitBits(payload, bitLen)` (`bitstream.rs:220-230`). (Promote the throwaway `testBitWriter`, `md_test.go:86-117`.)
- A2. `md/encode.go` `encodePayload(*descriptor) ([]byte, int, error)`: faithful inverse of `decodePayload` (`md/md.go:826-863`), mirroring Rust `encode_payload` (`encode.rs:65-92`): `canonicalize` → validators → Header(5b) → PathDecl → UseSitePath → `writeNode` → `tlvSection.write`. All sub-writers invert their `read*` counterpart, with the encode-side validators ported (`ThresholdOutOfRange`/`ChildCountOutOfRange`/`KGreaterThanN`/`OverrideOrderViolation`/`EmptyTlvEntry`/`VarintOverflow`/path-depth/alt-count guards).
- A3. `md/canonicalize.go`: port `canonicalize_placeholder_indices` (`canonicalize.rs:168-`, ~150 LOC; `remap_indices`/`walk_collect_first`) — permutes tree indices + divergent paths + ALL per-@N TLV maps atomically. **Mandatory** (see §6 invariant I-1).
- A4. identity: `computeEncodingID(d) [16]byte` = `SHA-256(encodePayload(d))[0:16]` (`identity.rs:39-45`); `deriveChunkSetID(id) uint32` = top-20-MSB-first (`chunk.rs:175-179`).
- A5. chunk write: `ChunkHeader` type + `.write` (37 bits: `[version=4:4][chunked=1:1][csid:20][count-1:6][index:6]`, `chunk.rs:32-57`); `split(d) ([]chunkBytes, error)` (threshold `SINGLE_STRING_PAYLOAD_BIT_LIMIT=320`, `ceil` count ≤64, byte-aligned slicing, `chunk.rs:235-290`).
- A6. chunk read: `ParseChunkHeader(s) (ChunkHeader, error)` (public, mirrors `mk.Header` shape, `mk/mk.go:48-56`); `Reassemble(strs []string) (*descriptor, error)` mirroring `reassemble` (`chunk.rs:305-389`): per-chunk unwrap → header → consistency (version/csid/count) → completeness (count, no gaps) → concat → `decodePayload` → **integrity gate** (re-derive csid, compare).
- A7. `codex32/mdencode.go`: `MDChecksumSymbols(dataSyms) []byte` (regular-only BCH, analogue of shipped `MKChecksumSymbols`, `codex32/mkencode.go:18-55`, using `mdRegularTargetHi/Lo` + `mdmkPolymodInitLo`); `assembleMD1(dataSyms) string` (HRP `md1`, regular-only, mirrors `assembleMK1`); `encodeMD1String(d) (string, error)` (single-string path) + the chunked-string assembly over `split`.

**Phase B — GUI: chunked md1 gather + wallet-policy display:**
- B1. `md1Gatherer` + `md1GatherFlow` — near-clones of `mk1Gatherer`/`mk1GatherFlow` (`gui/mk1_inspect.go:48-83,156-256`) for md1 multi-chunk NFC gather, calling `md.Reassemble` at completion. (Recommend factoring a shared generic gatherer; polish, not fidelity.)
- B2. wallet-policy xpub-expansion: when the md1 carries a non-empty Pubkeys TLV, reconstruct per-@N xpubs (32B chainCode‖33B compressed pubkey + origin/fp) into `bip380.Key`, project the **bip380-expressible subset only** (singlesig `wpkh/pkh/tr-keyonly/sh-wpkh` + `wsh(sortedmulti)` + `sh(wsh(sortedmulti))`) onto `*bip380.Descriptor`; add the secp256k1 on-curve check the decoder currently no-ops (`md/md.go:1071-1077`; Rust `validate.rs:216`).
- B3. wire the projected descriptor into the existing `verifyAddressFlow(ctx, th, desc)` (`gui/verify_address.go:22`) + the existing descriptor display; route chunked md1 through B1 instead of the current refusal (`gui/gui.go:1971-1979`).

### OUT (explicitly deferred)
- **T5 bundle orchestration UX** (multi-distinct-card grouping/manifest/sequencing screens) — separate cycle; #10 delivers only the codec primitives T5 orchestrates.
- **T6 seed→md1 glue** (derive a descriptor from a seed) — uses #10's write path, separate cycle.
- **BCH error-CORRECTION on the md1 read path** (`decode_with_correction`, `chunk.rs:502-623`) — the device engraves verbatim and the gather path uses pure verify (shipped codex32 is verify-only by design, `codex32/mdmk.go:16-17`). Not needed.
- **md1 shapes outside the bip380 model** (unsorted `multi`, `multi_a`, `sortedmulti_a`, taptree, arbitrary miniscript) — decode + display read-only (existing `Renderable=false` path, `md/md.go:1237-1293`); **excluded from address-verify** (I-6).

### Phasing/split note for R0
Combined estimate ≈ 950 LOC + comparable tests. Phase A (headless codec) is the gate-critical foundation; Phase B (GUI) builds strictly on it. **R0 should assess whether to ship #10 as one cycle or split into #10a (Phase A) → #10b (Phase B).** The author's lean: one cycle if the implementer + exec-review can hold it; otherwise split at the A/B boundary (A's acceptance = byte-exact golden parity + Reassemble round-trip; it is independently valuable as the T5/T6 foundation).

## 3. Wire-format facts (locked; full citations in the blueprint §1–§3)
- **payload bit-packing:** MSB-first, last byte zero-padded on LOW bits (`bitstream.rs:29-69,81-83`).
- **single Header:** 5 bits, `(divergent<<4)|(version&0xF)`, version=4; chunked-flag = bit 0 of symbol 0 = 0 for single (since 4=`0b00100`).
- **ChunkHeader:** 37 bits, version in the **top 4 bits** (distinct from the single Header) — the `WireVersionMismatch{got:2}` trap.
- **single/chunked discriminator:** `syms[0]&1` (bit 0), used everywhere — NEVER `ChunkHeader::read`-then-catch (`bundle.rs:138-151`, `md/md.go:1207`).
- **kiw:** `32 - leading_zeros(n-1)`, clamp 0 at n∈{0,1}; n=1 ⟹ key-arg index fields emit zero bits.
- **csid:** `SHA-256(canonical payload)[0:16]` → top-20-MSB-first; pin `AB CD EF…` → `0xABCDE`.
- **varint:** LP4-ext, lengths in **BITS** not bytes; `[L:4][payload:L]`, L=15 escape, max 29 bits.
- **TLV:** per-entry sub-bitstreams, emitted sorted-by-tag ascending, `[tag:5][varint(bitLen)][reEmitBits(payload,bitLen)]`; idx strictly ascending; empty entry rejected; unknown TLVs re-emitted via `reEmitBits` (not padded bytes).
- **BCH:** md1 regular checksum (13 symbols), `POLYMOD_INIT=0x23181b3` (not codex32's 1).

## 4. Faithfulness / security spine
- **Byte-exact fidelity to Rust** is the contract: `encodePayload` output must equal the constellation's `.bytes.hex` goldens (§5). No approximation.
- **Public-only:** md1/descriptor/xpub are PUBLIC (NFC-ok). This cycle touches no secret material — no seed, no xprv, no ms1 emission. The xpub-expansion handles only public xpubs (`.Neuter`-equivalent already; pubkeys TLV carries public points). No CSPRNG (md1 csid is deterministic by derivation).
- **No mis-rendering of spend paths:** any md1 shape not faithfully bip380-expressible is display-only and excluded from address-verify (I-6) — never silently verify against a wrong address (the T2c faithful-or-refuse discipline).

## 5. Acceptance gate (the proofs; TDD)
Vendor the constellation's `tests/vectors/*.{bytes.hex,phrase.txt,descriptor.json}` subset into `md/testdata/` (hermetic; constellation is source of truth).
1. **(PRIMARY) byte-exact encoder parity:** for each MANIFEST vector, build the `descriptor` (from `.descriptor.json` or hand-built), `encodePayload`, assert `hex(bytes)==<name>.bytes.hex` AND `bitLen` matches. (e.g. `wpkh_basic=2002001800`, `wsh_with_fingerprints=204200182182142f09bd5b7ddfcafebabe`.)
2. **full-string parity:** `encodeMD1String(d)==<name>.phrase.txt` for single vectors (exercises `MDChecksumSymbols`); `ValidMD` is the independent BCH check.
3. **round-trip:** `Decode(Encode(d))==Decode(golden)`; md1 MAY assert byte-equality on `.bytes.hex` (goldens are SHA-derived canonical — unlike mk1's arbitrary csids).
4. **chunked round-trip:** `split(d)` → each chunk `ValidMD` → `Reassemble` succeeds + csid matches; drop a chunk → `ErrChunkSetIncomplete`; corrupt csid → `ErrChunkSetIdMismatch`. Port the `chunked_md1_vector` fixture (`me-cli/bundle.rs:547-585`, 6-key wsh-sortedmulti, 15-deep divergent paths, ≥4 chunks).
5. **identity pins:** `deriveChunkSetID` → `0xABCDE`; `computeEncodingID` determinism + path-sensitivity.
6. **xpub-expansion:** projected `*bip380.Descriptor` yields the same receive address `verifyAddressFlow` validates, for a singlesig + a `wsh(sortedmulti)` vector; a non-bip380 shape is display-only + NOT address-verified.
7. **no-regression:** `go test ./...` + `TestAllocs` green; new GUI screens don't break the alloc gate; chunked md1 now gathers instead of refusing (B3) without changing single-md1 behavior.
8. **error categories (errors.Is), no panics:** all encode/decode/reassemble error paths return typed errors; fuzz `encodePayload`/`Encode`/`Reassemble`/`ParseChunkHeader` for 0 panics.

## 6. Invariants (R0 must confirm each)
- **I-1 (CRITICAL):** `canonicalize` runs inside `encodePayload` (on a clone, before validation/emission). Without it a non-canonical input yields a different csid than Rust → cross-tool `ChunkSetIdMismatch`. Mandatory because T6's author-built AST can be non-canonical.
- **I-2 (CRITICAL):** MSB-first `bitWriter` + non-byte-aligned `reEmitBits` are byte-exact; their golden unit tests (port `bitstream.rs:236-407`) pass before any structural encoder code (TDD order).
- **I-3:** single Header (5b, low-nibble version) vs ChunkHeader (37b, top-nibble version) never confused; the `syms[0]&1` discriminator is used everywhere (no `ChunkHeader::read` probe as primary).
- **I-4:** varint lengths and TLV `bitLen` are in BITS; reEmit uses tracked `bitLen`, not padded byte length.
- **I-5:** `is_nums` suppresses the kiw field; `MultiKeys` writes raw kiw-width indices (not child nodes); n=1 ⟹ kiw=0 ⟹ zero-bit key args. (Pin via ported `tree.rs` bit-cost tests.)
- **I-6:** xpub-expansion projects ONLY the bip380-expressible subset; unsorted `multi`/`multi_a`/taptree/miniscript are display-only and excluded from address-verify.
- **I-7:** `MDChecksumSymbols` uses `POLYMOD_INIT=0x23181b3` (the md/mk BCH initial residue), not codex32's 1; `ValidMD` round-trips every emitted string.
- **I-8:** the encoder is the faithful inverse of the shipped decoder for the full renderable AST (single + chunked, fingerprints, divergent paths, all node types), proven by the byte-exact golden parity (§5.1) — the primary gate.
- **I-9 (no-regression):** existing single-md1 decode/display, mk1, ms1, and all shipped flows are byte-unchanged; chunked md1 transitions from refuse → gather without affecting single md1.

## 7. Biggest risks (lock in R0)
Per the blueprint §8: (1 CRITICAL) canonicalize-in-encoder; (2 CRITICAL) bitWriter/reEmitBits correctness; (3) Header vs ChunkHeader version-nibble; (4) varint/TLV bits-not-bytes; (5) is_nums/MultiKeys/n=1-kiw; (6) xpub-expansion subset narrower than renderable; (7) MDChecksumSymbols POLYMOD_INIT; (8) unknown-TLV re-emit. R0 focus per blueprint: confirm (a) canonicalize in-scope, (b) byte-exact `.bytes.hex` parity = primary gate, (c) xpub-expansion projectable-subset boundary, (d) bit-0 discriminator everywhere.

## 8. Gate reminder
This spec MUST pass opus R0 to 0C/0I before code; fold → persist verbatim to `design/agent-reports/` → re-dispatch after every fold until GREEN. Then implementation plan → its own R0 → GREEN → single-implementer TDD in a worktree → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h.
