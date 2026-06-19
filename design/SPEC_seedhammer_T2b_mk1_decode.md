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
2. **2.2 Decoder matches mk-codec wire exactly.** Bytecode layout, compact-73, path table (incl. `0xFE` LEB128), string-layer header (`total_chunks`/`chunk_index` stored **value−1**, decode `+1`), and cross-chunk-hash reassembly MUST match `mnemonic-key/crates/mk-codec` byte-for-byte. Verified by Rust-sourced parity vectors (§6), NOT Go self-round-trip (there is no Go encoder).
3. **2.3 xpub reconstruction is faithful.** `depth := component_count(origin_path)`, `child_number := last_component(origin_path)` or `Normal{0}` for empty path; `network := version_to_network(version)` rejecting unknown versions; reconstructed via `hdkeychain.NewExtendedKey(...).String()` after `btcec.ParsePubKey` validates the 33-byte point. (`xpub_compact.rs:86-101`.)
4. **2.4 Display is read-only.** The decode-display and gather screens contain NO engrave / NFC-write / NDEF / plate / mutation call — only render + navigation returns. (Mirrors the T2a §2 safety invariant.)
5. **2.5 No regression to the verbatim engrave path.** `validateMdmk`, the `mdmkText` engrave variants, and md1 handling are behaviorally unchanged. The new affordance is mk1-only and additive. Existing GUI tests (incl. the alloc gate) pass.
6. **2.6 0-alloc gate untouched.** No allocating per-frame work is added to `StartScreen.Flow` or `DescriptorScreen.Confirm` (the only `TestAllocs`-gated paths). The new screens are not alloc-gated. (`gui_test.go:39-96`.)
7. **2.7 No secret handling claims.** mk1 is PUBLIC; there is NO `Unshared` gate, NO `wipeBytes` scrub, NO secrecy logic. Decode-display is unconditional for BCH-valid mk1. (Contrast T2a/ms1.)
8. **2.8 Reassembly rejects malformed sets.** chunk_set_id mismatch, single-string header mid-set, `chunk_index >= total_chunks`, duplicate/missing indices, count ≠ total_chunks, and cross-chunk-hash mismatch all surface as decode errors (not panics, not silent wrong-xpub). (`chunk.rs:109-196`.)
9. **2.9 mk1-vs-md1 discrimination is by HRP.** The Inspect affordance and decode are gated on the `mk1` prefix; an md1 `mdmkText` keeps the current engrave-only flow until T2c.

## 3. Source facts (verified against `mnemonic-key/crates/mk-codec`; see recon for full citations)

### 3.1 Sizing → always chunked
`SINGLE_STRING_LONG_BYTES = 56` (`consts.rs:33`); `XPUB_COMPACT_BYTES = 73` (`consts.rs:53`); emit decision `bytecode.len() <= 56 ? single : chunked` (`pipeline.rs:73`). Min real bytecode ≥ 80 B → unreachable single-string.

### 3.2 Bytecode layout (`bytecode/encode.rs:7-11`, `decode.rs:19-54`)
`header(1) | stub_count(1,≥1) | stubs(4×N) | [origin_fp(4) iff header bit2] | origin_path(var) | xpub_compact(73)`.

### 3.3 compact-73 + reconstruction (`xpub_compact.rs`)
`version(4) | parent_fp(4) | chain_code(32) | public_key(33)`. MAINNET ver `0x0488B21E`, TESTNET `0x043587CF`. depth/child reconstructed from `origin_path` (§2.3).

### 3.4 Path codec (`bytecode/path.rs:28-55`)
14 standard indicators (mainnet `0x01..0x07`, testnet `0x11..0x17`; `0x16` added in 0.2.0 — **accept it**); `0xFE` = explicit: count `0..=10` (0 = no-path/depth-0), then LEB128 u32 components with BIP-32 hardened bit in high bit; all other indicators → invalid.

### 3.5 String-layer header (`string_layer/header.rs`)
Single = 2 symbols `version + type=0x00`. Chunked = 8 symbols `version + type=0x01 + chunk_set_id(20b=4 sym, big-endian) + total_chunks + chunk_index`. `total_chunks`/`chunk_index` stored as **value−1** (decode `+1`). `VERSION_V0_1 = 0x00`.

### 3.6 Reassembly (`string_layer/chunk.rs:109-196`)
All chunks share version/chunk_set_id/total_chunks; differ only in chunk_index. Concatenate fragment BYTES in chunk_index order → stream; trailing 4 B = `cross_chunk_hash`; verify `== SHA-256(stream[..len-4])[0..4]`; strip → bytecode. Per-chunk fragment is `bytes_to_5bit`-encoded on the wire (decode: strip header symbols, repack 5-bit→8-bit per chunk).

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
Internals: `ParseHeader` → `codex32.MKDataSymbols` → read header (2 or 8 symbols, `+1` decode). `Decode` → per string: symbols → header + fragment (strip header symbols, repack 5-bit→8-bit) → assemble `[]chunkFragment` → reassemble (sort by index, verify cross-chunk hash) → `decodeBytecode` (cursor over header/stub_count/stubs/[fp]/path/compact-73) → `reconstructXpub` (`hdkeychain.NewExtendedKey(version, pubkey, chaincode, parentFP, depth, childNum, false).String()`, with `btcec.ParsePubKey` validating the point) → `Card`. Path codec = a 14-entry table mirroring `STANDARD_PATHS` + `0xFE` LEB128 decode (accept `0x16`).

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
- **Create** `mk/mk_test.go` (parity vectors V1–V3 + multi-stub-no-fp + 3-chunk + explicit-path + negative reject cases; header-parse unit tests).
- **Modify** `codex32/mdmk.go` (+`MKDataSymbols`) and `codex32/mdmk_test.go`.
- **Create** `gui/mk1_inspect.go` (gather sub-screen + decode-display flow).
- **Create** `gui/mk1_inspect_test.go` (gather state machine; mismatch/dup/incomplete; paging-reaches-xpub-tail; display-only).
- **Modify** `gui/gui.go` (`mdmkFlow` mk1 Inspect affordance) and `gui/gui_test.go` (mk1 Inspect path + md1-unchanged regression).

## 6. TDD
- **Parity (load-bearing):** embed verbatim from `mk-codec/src/test_vectors/v0.1.json` (cite its pinned SHA-256 in `tests/vectors.rs`): V1 (`m/48'/0'/0'/2'`, 0x05, fp, 2-chunk), V2 (`m/84'/0'/0'`, 0x03, fp, 2-chunk), V3 (`m/48'/1'/0'/2'`, 0x15, testnet, 2-chunk), plus multi-stub-no-fp, a 3-chunk, and an explicit-path (`0xFE`) vector. Assert `Decode(strings)` → exact `{Network, Path, Fingerprint, Stubs, Xpub}`. **Provenance: Rust-sourced only; never Go-derived.**
- **Negative:** from the corpus's schema-2 reject entries + constructed cases — chunk_set_id mismatch, missing/duplicate index, count≠total, single-header-mid-set, cross-chunk-hash mismatch, unknown path indicator, bad point, unknown version. Each → error, no panic, no partial Card.
- **Header parse:** single (2-sym) and chunked (8-sym) incl. the `+1` boundary (total_chunks=2 ⇒ wire 1).
- **GUI:** gather reaches complete set from out-of-order scans; rejects foreign chunk_set_id / duplicates; decode-display pages to the xpub tail (measure-and-advance, gap-free); no engrave/NFC from inspect; md1 path unchanged; `TestAllocs` passes.

## 7. Process
- **R0 gate (mandatory, this doc):** opus-architect review to 0C/0I before any code. Fold → persist verbatim to `design/agent-reports/seedhammer-T2b-mk1-spec-review-R*.md` → re-dispatch after every fold until GREEN.
- Then `IMPLEMENTATION_PLAN_seedhammer_T2b_mk1_decode.md` → its own R0 to GREEN.
- Then single-implementer TDD in an isolated worktree off `4d02021`, **Phase A (package, parity-GREEN) before Phase B (GUI)**. Commits signed (`-S`) + DCO (`-s`), author "Brian Goss <goss.brian@gmail.com>", trailer `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. Explicit-path staging.
- Then the mandatory whole-diff adversarial execution review (persist verbatim) → merge no-ff → push `bg002h`.
