# SPEC — T2b: on-device mk1 decode→display (xpub + fingerprint + path + policy-id stubs)

> Cycle T2b of the "SeedHammer as air-gapped constellation terminal" roadmap (`design/RECON_seedhammer_constellation_terminal.md`).
> Recon: `design/cycle-prep-recon-T2b-mk1-decode.md` (every protocol fact independently re-verified against `mnemonic-key/crates/mk-codec`, `family_token "mk-codec 0.2"`).
> Base: fork `4d02021` (T2a merged). Fork-side only; no upstream PR.

## 1. Goal & scope

Let an operator **inspect** what a set of `mk1` chunk strings decodes to — account **xpub**, derivation **path**, origin **fingerprint**, **network**, and **policy-id stub** count — on the air-gapped SeedHammer touchscreen, BEFORE/while engraving. mk1 is PUBLIC (account-level xpub, no secret), so inspection is offered unconditionally for any BCH-valid mk1.

### In scope (T2b)
- A new `mk` Go package: string-layer header parse, multi-chunk reassembly (cross-chunk-hash verified), bytecode decode, and full-xpub reconstruction — parity-verified against Rust-sourced vectors.
- A GUI **multi-chunk gather** sub-screen (NFC) that accumulates mk1 chunks until a complete set is present, with progress + mismatch feedback.
- A GUI **decode-display** screen (measure-and-advance paging) showing network / path / fingerprint / stub count / xpub.
- Wiring: an mk1-only **"Inspect key"** affordance in `mdmkFlow`; the existing verbatim per-string engrave path is unchanged.

### Out of scope (explicit)
- **md1 decode** (T2c, #9 — md-codec has its own chunk format; sibling, not shared code).
- **Engraving the decoded form.** The engrave model stays per-string-verbatim (`validateMdmk`/`mdmkFlow`). Decode-display is read-only inspection; it never engraves, NFC-writes, or mutates.
- **Typed entry of mk1 strings.** mk1 chunks are ~80–110 bech32 chars × N; hand-typing is impractical and mk1 is public (NFC is permitted by the security constraints). NFC-gather only. (A typed fallback may be revisited in T7.)
- **Guided bundle sequencing across codecs / `me bundle` parity** (T5, #5 — builds on this cycle's gather primitives).
- **Single-string mk1 fast path as a real case** — structurally unreachable (§3.1); the decoder still handles `total_chunks == 1` defensively but no UX assumes it.

## 2. Invariants (R0 MUST verify each — Critical if violated)

1. **2.1 Single-string is unreachable; gather is mandatory.** `XPUB_COMPACT_BYTES = 73 > SINGLE_STRING_LONG_BYTES = 56`, and a bytecode always also carries header+stub_count+≥1 stub(+fp)+path, so min bytecode ≥ 80 B → always chunked. The gather loop and reassembler are load-bearing, not optional. (recon F-headline; `consts.rs:33,53`, `pipeline.rs:73,161-164`.)
2. **2.2 Decoder matches mk-codec wire exactly.** Bytecode layout, compact-73, path table (incl. `0xFE` LEB128), string-layer header, and cross-chunk-hash reassembly MUST match `mnemonic-key/crates/mk-codec` byte-for-byte. **CRITICAL header detail (R0-C1):** `total_chunks` is stored **value−1** on the 5-bit wire (decode `+1`); `chunk_index` is stored **verbatim, 0-based (NO `+1`)** (`header.rs:88` vs `:97`, decode `:146` vs `:147`). Adding `+1` to `chunk_index` injects an off-by-one (slots `1..total`, last chunk trips `idx >= total_chunks`). Verified by Rust-sourced parity vectors (§6), NOT Go self-round-trip (there is no Go encoder).
3. **2.3 xpub reconstruction is faithful.** `depth := component_count(origin_path)`, `child_number := last_component(origin_path)` or `Normal{0}` for empty path; `network := version_to_network(version)` rejecting unknown versions; reconstructed via `hdkeychain.NewExtendedKey(version, pubkey, chaincode, parentFP, depth, childNum, false).String()` after `btcec.ParsePubKey` validates the 33-byte point. **`childNum` MUST be the raw BIP-32 hardened-bit-encoded u32** (e.g. `0x80000002` for `2'`), matching `bip380.go:102`'s use of the raw `bip32.Path` u32 — not the unhardened index (R0-M1). (`xpub_compact.rs:86-108`.)
4. **2.4 Display is read-only.** The decode-display and gather screens contain NO engrave / NFC-write / NDEF / plate / mutation call — only render + navigation returns. (Mirrors the T2a §2 safety invariant.)
5. **2.5 No regression to the verbatim engrave path.** `validateMdmk`, the `mdmkText` engrave variants, and md1 handling are behaviorally unchanged. The new affordance is mk1-only and additive. Existing GUI tests (incl. the alloc gate) pass.
6. **2.6 0-alloc gate untouched.** No allocating per-frame work is added to `StartScreen.Flow` or `DescriptorScreen.Confirm` (the only `TestAllocs`-gated paths). The new screens are not alloc-gated. (`gui_test.go:50-98`.)
7. **2.7 No secret handling claims.** mk1 is PUBLIC; there is NO `Unshared` gate, NO `wipeBytes` scrub, NO secrecy logic. Decode-display is unconditional for BCH-valid mk1. (Contrast T2a/ms1.)
8. **2.8 Decode rejects every malformed input — no panic, no partial Card.** The Go decoder MUST replicate mk-codec's full reject set across BOTH layers (R0-I1):
   - **String-layer / header** (`header.rs`, `chunk.rs:109-203`): `UnsupportedVersion` (header version field ≠ 0), `UnsupportedCardType` (reserved string-header type `0x02..=0x1F`), `ChunkedHeaderMalformed` (single-string header at head of multi-set, `total_chunks == 0`, `> MAX_CHUNKS=32`, or `chunk_index >= total_chunks`), `MixedHeaderTypes` (single-string header mid-set), chunk_set_id mismatch, total_chunks disagreement, duplicate index, missing index, count ≠ total_chunks, `CrossChunkHashMismatch`.
   - **Fragment 5→8 repack** (`bch.rs:78-100`): **strict** `five_bit_to_bytes` — reject any symbol ≥ 32, `bits >= 5` leftover, OR non-zero trailing padding bits (`MalformedPayloadPadding`). MUST NOT reuse `codex32.parts.data()` (it silently zero-pads / panics on `rem>4`).
   - **Bytecode layer** (`decode.rs`, `header.rs`, `path.rs`): `ReservedBitsSet` (bytecode-header bits 0/1/3), `UnsupportedVersion`, `InvalidPolicyIdStubCount` (stub_count == 0), `InvalidPathIndicator`, `PathTooDeep` (explicit count > 10), `InvalidPathComponent` (LEB128 overflow), `UnexpectedEnd` (truncation), `TrailingBytes`, `InvalidXpubVersion`, `InvalidXpubPublicKey` (bad secp256k1 point).
9. **2.9 mk1-vs-md1 discrimination is by HRP.** The Inspect affordance and decode are gated on the `mk1` prefix; an md1 `mdmkText` keeps the current engrave-only flow until T2c.
10. **2.10 Decode-display paging reaches the xpub tail, gap-free.** The base58 xpub (~111 chars) is the field that forces paging on 240×240; the measure-and-advance display MUST show every line with no gap and no dropped tail (the T1 paging-overflow lesson). First-class, execution-review-checkable.

## 3. Source facts (verified against `mnemonic-key/crates/mk-codec`; see recon for full citations)

### 3.1 Sizing → always chunked
`SINGLE_STRING_LONG_BYTES = 56` (`consts.rs:33`); `XPUB_COMPACT_BYTES = 73` (`consts.rs:53`); emit decision `bytecode.len() <= 56 ? single : chunked` (`pipeline.rs:73`). Min real bytecode ≥ 80 B (`pipeline.rs:354` "smallest valid bytecode = 80 bytes > 56-byte single-string capacity") → unreachable single-string.

### 3.2 Bytecode layout (`bytecode/encode.rs:7-11`, `decode.rs:19-54`)
`header(1) | stub_count(1,≥1) | stubs(4×N) | [origin_fp(4) iff header bit2] | origin_path(var) | xpub_compact(73)`.

### 3.3 compact-73 + reconstruction (`xpub_compact.rs`)
`version(4) | parent_fp(4) | chain_code(32) | public_key(33)`. MAINNET ver `0x0488B21E`, TESTNET `0x043587CF`. depth/child reconstructed from `origin_path` (§2.3).

### 3.4 Path codec (`bytecode/path.rs:28-55`)
14 standard indicators (mainnet `0x01..0x07`, testnet `0x11..0x17`; `0x16` added in 0.2.0 — **accept it**); `0xFE` = explicit: count `0..=10` (0 = no-path/depth-0), then LEB128 u32 components with BIP-32 hardened bit in high bit; all other indicators → invalid.

### 3.5 String-layer header (`string_layer/header.rs`)
Single = 2 symbols `version + type=0x00`. Chunked = 8 symbols `version + type=0x01 + chunk_set_id(20b=4 sym, big-endian) + total_chunks + chunk_index`. **`total_chunks` is stored value−1** (`:88` encode `(total_chunks - 1)`, `:146` decode `+1`); **`chunk_index` is stored verbatim, 0-based** (`:97` emit `chunk_index & 0x1F`, `:147` decode with NO `+1`). `VERSION_V0_1 = 0x00`; version ≠ 0 → `UnsupportedVersion`; type ∉ {0x00,0x01} → `UnsupportedCardType`.

### 3.6 Reassembly (`string_layer/chunk.rs:109-203`)
All chunks share version/chunk_set_id/total_chunks; differ only in chunk_index. Concatenate fragment BYTES in chunk_index order → stream; trailing 4 B = `cross_chunk_hash`; verify `== SHA-256(stream[..len-4])[0..4]` (`:189-202`); strip → bytecode. Per-chunk fragment is `bytes_to_5bit`-encoded on the wire (decode: strip header symbols, then strict `five_bit_to_bytes` per chunk — §2.8).

### 3.7 Fork string-layer alignment (`seedhammer/codex32/mdmk.go`)
`mdmk.go` pins the exact mk-codec BCH constants (MK regular `0x1062435f91072fa5c`, long `0x41890d7e441cbe97273`, POLYMOD_INIT `0x23181b3`). `ValidMK(s)` validates ONE string's BCH (regular 13-sym / long 15-sym by data length). No header parse, no reassembly, no gathering exists — all net-new (recon F6, F7).

## 4. Design

### 4.1 Phase A — `mk` decode package (deterministic core)

**`codex32` gains one pure-stdlib primitive** (it already owns the bech32/5-bit engine via `splitHRP` + `inputData`; `mk` must not duplicate bech32 decode):
```go
// MKDataSymbols returns the 5-bit data symbols (string-layer header symbols
// followed by the bytes_to_5bit-encoded fragment) of a BCH-valid mk1 string,
// with the BCH checksum (13 regular / 15 long) stripped. Errors if s is not a
// BCH-valid mk1 string. Pure-stdlib; no key-derivation deps.
func MKDataSymbols(s string) ([]byte, error)   // each elem 0..31
```
It gates on `ValidMK`, `splitHRP`s, maps each data char → 5-bit value via the existing charset, and trims the checksum length implied by `ValidMK`'s regular/long bracket.

**New package `seedhammer.com/mk`** (depends on `codex32`, `hdkeychain`, `btcec`, `chaincfg`):
```go
type Card struct {
    Network     string      // "mainnet" | "testnet"
    Path        string      // e.g. "m/48'/0'/0'/2'" (or "m" for depth-0)
    Fingerprint string      // 8 lowercase hex, or "" if absent
    Stubs       [][4]byte   // policy-id stubs (len ≥ 1)
    Xpub        string      // base58 "xpub…"/"tpub…"
}

type Header struct {
    Chunked     bool
    ChunkSetID  uint32
    TotalChunks int          // 1 for single; ≥2 in practice
    ChunkIndex  int          // 0-based
}

// ParseHeader extracts the string-layer header from one BCH-valid mk1 string.
func ParseHeader(s string) (Header, error)

// Decode reassembles a complete set of BCH-valid mk1 chunk strings (any order)
// and decodes to a Card. Enforces every §2.8 reject condition.
func Decode(strings []string) (Card, error)
```
Internals: `ParseHeader` → `codex32.MKDataSymbols` → read header (2 or 8 symbols; `total_chunks` decode `+1`, `chunk_index` verbatim — §2.2). `Decode` → per string: symbols → header + fragment (strip header symbols, then a **strict** `fiveBitToBytes` that rejects non-zero pad bits / leftover ≥5 bits / symbols ≥32 — NOT `codex32.parts.data()`; §2.8) → assemble `[]chunkFragment` → reassemble (sort by index, verify cross-chunk hash) → `decodeBytecode` (cursor over header/stub_count/stubs/[fp]/path/compact-73) → `reconstructXpub` (`hdkeychain.NewExtendedKey(version, pubkey, chaincode, parentFP, depth, childNum, false).String()`, `childNum` = raw hardened-bit u32, `btcec.ParsePubKey` validating the point) → `Card`. Path codec = a 14-entry table mirroring `STANDARD_PATHS` + `0xFE` LEB128 decode (accept `0x16`; reject count > 10, LEB128 overflow, unknown indicators).

### 4.2 Phase B — GUI gather + decode-display

**Gather sub-screen** (new; NOT alloc-gated). Entered from the first scanned mk1 chunk. Owns a fresh scanner goroutine + `ctx.Platform.NFCReader()` (safe: `StartScreen.Flow` has already returned and closed its reader before `engraveObjectFlow` runs — recon F7). State: `set map[int]string` keyed by chunk_index, plus `total`, `chunkSetID` from the first chunk's `ParseHeader`. Per additional scan:
- Non-mk1 / non-`ValidMK` / different `chunkSetID` → show "Different key — rescan" (do not add).
- Duplicate index → "Already captured chunk i".
- New index → add; render progress "Captured k of total — scan the remaining chunk(s)".
- When `len(set) == total` → `mk.Decode(values)` → on success advance to decode-display; on error show the decode error and allow rescan/back.
- Back at any time returns to the engrave chooser without side effects.

**Decode-display screen** (new; NOT alloc-gated). Renders, with the **measure-and-advance paging** pattern (T1/T2a lesson — measure each line with `ctx.Styles.body.Measure(width, "%s", line)`, page by the count shown, gap-free): `Network: mainnet` / `Path: m/48'/0'/0'/2'` / `Fingerprint: aabbccdd` (or `none`) / `Policy stubs: N` / `Account xpub:` then the full base58 xpub (the long field that forces paging on 240×240). Single Back affordance returns to the chooser. Read-only (§2.4).

### 4.3 Wiring / no-regression (`gui/gui.go` `mdmkFlow`)
`mdmkFlow` gains, for `mk1`-prefix strings only, an **"Inspect key"** choice alongside the existing engrave variants (e.g. prepend to the `ChoiceScreen`, or a Button affordance). Selecting it runs gather→decode-display, then returns to the chooser. For md1 strings the flow is byte-identical to today (§2.5, §2.9). `validateMdmk` and the engrave path are untouched.

## 5. File manifest (indicative; the plan pins exact paths/lines)
- **Create** `mk/mk.go` (Card, Header, Decode, ParseHeader, path table, bytecode decode, xpub reconstruction).
- **Create** `mk/mk_test.go` (parity vectors V1–V3 + V6 (3-stub w/fp) + V4 (1-stub no-fp) + V5/V7 (explicit-path, 3-chunk) + negative reject cases; header-parse unit tests).
- **Modify** `codex32/mdmk.go` (+`MKDataSymbols`) and `codex32/mdmk_test.go`.
- **Create** `gui/mk1_inspect.go` (gather sub-screen + decode-display flow).
- **Create** `gui/mk1_inspect_test.go` (gather state machine; mismatch/dup/incomplete; paging-reaches-xpub-tail; display-only).
- **Modify** `gui/gui.go` (`mdmkFlow` mk1 Inspect affordance) and `gui/gui_test.go` (mk1 Inspect path + md1-unchanged regression).

## 6. TDD
- **Parity (load-bearing):** embed verbatim from `mk-codec/src/test_vectors/v0.1.json` (corpus: `family_token "mk-codec 0.2"`, schema 2, 18 clean + 22 negative; cite the pinned SHA-256 `ebd8f34d8d52896e07e1faef995f18ffa61d42e2a048fb2a8c11e67f120d78ff` from `tests/vectors.rs:41`). Use **only `decoder_correction: "clean"`** vectors (the fork's `ValidMK` does NO BCH correction — a vector needing correction would fail the gate before `mk.Decode`): **V1** (`m/48'/0'/0'/2'`, 0x05, fp, 2-chunk), **V2** (`m/84'/0'/0'`, 0x03, fp, 2-chunk), **V3** (`m/48'/1'/0'/2'`, 0x15, testnet, 2-chunk), **V6** (3-stub `m/48'/0'/0'/2'`, with fp, 2-chunk — multi-stub path), **V4** (`m/84'/0'/0'`, 1-stub **no fp**, 2-chunk), **V5** (explicit `m/9999'/1234'/56'/7'` 4-comp `0xFE`, with fp, 3-chunk) and/or **V7** (explicit max 10-comp, no fp, 3-chunk). Assert `Decode(strings)` → exact `{Network, Path, Fingerprint, Stubs, Xpub}`. **Provenance: Rust-sourced only; never Go-derived.**
- **Negative (two layers — assert *rejection per category*, NOT `expected_error` string equality, since Go error strings are independent of mk-codec's Rust rendering):**
  - **Gather / BCH layer** (corpus N1–N5: invalid HRP, mixed case, bad length, invalid char, BCH-uncorrectable) — rejected by `ValidMK` before reaching `mk.Decode`; the gather screen must reject these as "not a valid mk1 chunk".
  - **`mk.Decode` structural layer** (corpus N6–N23 + constructed): `UnsupportedCardType` (reserved type), `ReservedBitsSet`, `UnsupportedVersion`, `MalformedPayloadPadding` (non-zero pad bits — N7), chunk_set_id mismatch, missing/duplicate index, count≠total, single-header-mid-set, cross-chunk-hash mismatch, `InvalidPathIndicator`, `PathTooDeep`, `InvalidPathComponent`, `TrailingBytes`, `UnexpectedEnd`, `InvalidXpubVersion`, `InvalidXpubPublicKey`. Each → error, no panic, no partial Card.
- **Header parse:** single (2-sym) and chunked (8-sym); assert `total_chunks` decodes with `+1` (wire 1 ⇒ 2) and **`chunk_index` round-trips with NO offset** (0-based) — the R0-C1 guard.
- **GUI:** gather reaches complete set from out-of-order scans; rejects foreign chunk_set_id / duplicates; decode-display pages to the xpub tail (measure-and-advance, gap-free); no engrave/NFC from inspect; md1 path unchanged; `TestAllocs` passes.

## 7. Process
- **R0 gate (mandatory, this doc):** opus-architect review to 0C/0I before any code. Fold → persist verbatim to `design/agent-reports/seedhammer-T2b-mk1-spec-review-R*.md` → re-dispatch after every fold until GREEN.
- Then `IMPLEMENTATION_PLAN_seedhammer_T2b_mk1_decode.md` → its own R0 to GREEN.
- Then single-implementer TDD in an isolated worktree off `4d02021`, **Phase A (package, parity-GREEN) before Phase B (GUI)**. Commits signed (`-S`) + DCO (`-s`), author "Brian Goss <goss.brian@gmail.com>", trailer `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. Explicit-path staging.
- Then the mandatory whole-diff adversarial execution review (persist verbatim) → merge no-ff → push `bg002h`.
