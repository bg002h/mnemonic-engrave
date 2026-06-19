# cycle-prep recon — 2026-06-18 — T2b mk1 decode→display

**Fork HEAD at recon time:** `4d02021` (Merge feat/ms1-decode-display: T2a)
**Design repo:** `mnemonic-engrave`, branch `master`, clean.
**mk-codec source pinned for facts:** `mnemonic-key/crates/mk-codec` (`family_token = "mk-codec 0.2"`, wire spec `SPEC_mk_v0_1.md`).

Slug: `T2b-mk1-decode`. This recon **independently re-verified every load-bearing protocol fact against authoritative Rust source** (not the T2 cycle-prep draft), per the project external-fact-verification policy. The headline finding overturns the original S–M sizing.

---

## HEADLINE — verified scoping change

**Single-string mk1 is structurally UNREACHABLE for any real key card. Every real mk1 card is ≥ 2 chunks. Multi-chunk reassembly + multi-string input gathering are therefore MANDATORY in T2b and CANNOT be deferred to T5.**

Source-of-truth chain (all re-read this session):
- `mk-codec/src/consts.rs:33` — `SINGLE_STRING_LONG_BYTES = 56`.
- `mk-codec/src/consts.rs:53` — `XPUB_COMPACT_BYTES = 73` (asserted `= 4 + 4 + 32 + 33`).
- `mk-codec/src/string_layer/pipeline.rs:73` — emit decision is `if bytecode.len() <= SINGLE_STRING_LONG_BYTES { single } else { chunked }`.
- The compact xpub **alone** is 73 B > 56 B. A bytecode always contains `header(1) + stub_count(1) + ≥1 stub(4) + [fp(4)] + path(≥1) + xpub_compact(73)` ≥ **80 B (no-fp) / 84 B (with-fp)**. The codec's own fixture comment (`pipeline.rs:161-164`) states a typical card "= 84 bytes; this exceeds SINGLE_STRING_LONG_BYTES (= 56) and therefore lands in the chunked path."
- Every vector in `mk-codec/src/test_vectors/v0.1.json` with `decoder_correction: "clean"` carries `total_chunks ≥ 2`.

**Consequence:** the T2 cycle-prep's "mk1 (S–M)" estimate is revised to **S–M→M (~950–1350 LOC)**. The chunk-reassembly layer is not optional polish; it is the only way to decode any real mk1 at all.

---

## Per-fact verification

### F1 — Bytecode layout & decode order — ACCURATE (re-read `bytecode/encode.rs:7-11`, `decode.rs:19-54`)
```
[bytecode_header   : 1 B]   version + reserved + fingerprint flag (bit 2)
[stub_count        : 1 B]   MUST be ≥ 1 (stub_count == 0 → error)
[policy_id_stubs   : 4 × N B]
[origin_fingerprint: 4 B]   present iff header bit 2 set
[origin_path       : variable]   std-table indicator (1 B) OR 0xFE+count+LEB128
[xpub_compact      : 73 B]
```
Decode cursor order (`decode.rs`): header → stub_count(≥1) → N×stub → [fp if flag] → `decode_path` → `decode_xpub_compact`(73) → `reconstruct_xpub(compact, origin_path)`.

### F2 — Header bit-2 fingerprint flag — ACCURATE (`bytecode/header.rs:8,33,50`)
`fingerprint_flag = byte & FINGERPRINT_FLAG_MASK (bit 2) != 0`. Shape mirrors md1's header; bit-2 semantics align.

### F3 — compact-73 layout & depth/child reconstruction — ACCURATE (`bytecode/xpub_compact.rs`)
```
[version           : 4 B]   MAINNET 0x0488B21E (xpub) | TESTNET 0x043587CF (tpub)
[parent_fingerprint: 4 B]
[chain_code        : 32 B]
[public_key        : 33 B]  compressed secp256k1 point
= 73 B
```
`depth` and `child_number` are **DROPPED off-wire** and reconstructed (`reconstruct_xpub`, `:86-101`):
- `network := version_to_network(version)` — reject unknown version (`Error`).
- `depth := component_count(origin_path)`.
- `child_number := last_component(origin_path)`, **or `Normal{0}` when origin_path is empty** (depth-0 / no-path key, e.g. a WIF — v0.4.0+ convention).
- Full `Xpub{version, network, depth, parent_fingerprint, child_number, chain_code, public_key}` is assembled, then serialized to the standard base58 xpub string.

**Go equivalent:** `hdkeychain.NewExtendedKey(version[4], pubkey[33], chaincode[32], parentFP[4], depth, childNum, false /*isPrivate*/).String()`. Validate the 33-byte point with `btcec.ParsePubKey` before assembling. (`hdkeychain`, `btcec` are in-tree in the fork.)

### F4 — Standard-path table (14 entries) + 0xFE explicit path — ACCURATE (`bytecode/path.rs:30-55`)
| ind | path | | ind | path |
|----|------|--|----|------|
|0x01|m/44'/0'/0'|  |0x11|m/44'/1'/0'|
|0x02|m/49'/0'/0'|  |0x12|m/49'/1'/0'|
|0x03|m/84'/0'/0'|  |0x13|m/84'/1'/0'|
|0x04|m/86'/0'/0'|  |0x14|m/86'/1'/0'|
|0x05|m/48'/0'/0'/2'| |0x15|m/48'/1'/0'/2'|
|0x06|m/48'/0'/0'/1'| |0x16|m/48'/1'/0'/1'|
|0x07|m/87'/0'/0'|  |0x17|m/87'/1'/0'|

- `0xFE` = explicit path: 1-byte component count `0..=10` (0 = no-path/depth-0, e.g. WIF), then each component as **LEB128 u32 with the BIP-32 hardened bit in the high bit**.
- Reserved/invalid: `0x00`, `0x08..=0x10`, `0x18..=0xFD`, `0xFF` → `InvalidPathIndicator`.
- **Version flag:** `0x16` (BIP-48 testnet nested-segwit multisig) was added in mk-codec **0.2.0**; wire-additive (v0.1.x decoders reject it). **The Go decoder SHOULD accept all 14 indicators incl. 0x16** to match the current canonical codec the fork's BCH layer already pins (see F6).

### F5 — String-layer header + chunk reassembly contract — ACCURATE (`string_layer/header.rs`, `chunk.rs`)
Header lives at the **5-bit symbol layer** (after `mk1` HRP, before fragment payload):
- **Single-string header** = 2 symbols: `version(5b) + type=0x00(5b)`.
- **Chunked header** = 8 symbols: `version + type=0x01 + chunk_set_id(20b = 4 symbols, big-endian) + total_chunks(5b) + chunk_index(5b)`.
- `VERSION_V0_1 = 0x00`; `MAX_CHUNK_SET_ID = (1<<20)-1`.
- **CRITICAL wire detail:** `total_chunks` and `chunk_index` are stored as **`value − 1`** on the 5-bit wire (range 1..=32 → 0..=31); decode is `wire + 1` (`header.rs:83`). The Go reader MUST apply the `+1`.

Reassembly (`chunk.rs:109` `reassemble_from_chunks`):
1. All chunks share `version`, `chunk_set_id`, `total_chunks`; only `chunk_index` varies.
2. Reject a `SingleString` header at any non-leading position in a multi-chunk set (header-types-disagree).
3. Reject `chunk_set_id` mismatch; reject `chunk_index >= total_chunks`; reject received-count ≠ `total_chunks`; reject duplicate/missing index slots.
4. Concatenate fragments **in `chunk_index` order**.
5. Trailing 4 bytes = `cross_chunk_hash`; verify `== SHA-256(reassembled_bytecode_without_hash)[0..4]` (`CROSS_CHUNK_HASH_BYTES = 4`, `consts.rs:45`); strip → canonical bytecode. Mismatch → `CrossChunkHashMismatch`.

Split (encode side, for reference): bytecode ‖ cross_chunk_hash, split into `CHUNKED_FRAGMENT_LONG_BYTES = 53`-byte fragments; chunk 0 lands in long-code BCH territory, trailing short chunk falls back to regular code (`pipeline.rs:15-21`).

### F6 — Fork string-layer already pins mnemonic-key mk-codec — ACCURATE (`seedhammer/codex32/mdmk.go`)
`mdmk.go` BCH constants are copied verbatim from `mnemonic-key/crates/mk-codec/src/consts.rs`:
- `mkRegularTargetHi/Lo` = `0x1`/`0x62435f91072fa5c` → `0x1062435f91072fa5c` = `consts.rs:18` (`MK_REGULAR_CONST`).
- `mkLongTargetHi/Lo` = `0x418`/`0x90d7e441cbe97273` → `0x41890d7e441cbe97273` = `consts.rs:21` (`MK_LONG_CONST`).
- `mdmkPolymodInitLo = 0x23181b3`.
`ValidMK(s)` (`mdmk.go:136-143`) branches on data-part length → short (regular) vs long checksum; it validates **one** string's BCH only. There is **no** chunk-header parse, no reassembly, no multi-string gathering anywhere in the fork. → reassembly + gathering UX is **net-new**.

### F7 — GUI hook & current single-string-only scan path — ACCURATE (`gui/scan.go:70-78`, `gui/gui.go:1878-1933`)
`scan.go:70` — when `ValidMD || ValidMK`, returns `mdmkText(buf)` (one BCH-valid string). `gui.go:1878` `case mdmkText:` → `mdmkFlow` → engrave variant chooser (TEXT+QR / TEXT / QR-ONLY). **The scan path yields exactly ONE string per scan; there is no multi-string accumulation.** T2b must add a gather loop that collects N strings until `total_chunks` distinct indices are present, then decodes.

### F8 — Rust-sourced parity corpus available — ACCURATE (`mk-codec/src/test_vectors/v0.1.json`, `family_token "mk-codec 0.2"`, schema 2)
Corpus is SHA-256-pinned in `mk-codec/tests/vectors.rs`; vectors are generator-stable (`gen_mk_vectors`). Representative clean vectors confirmed present:
- **V1_bip48_mainnet_1_stub_with_fp** — `m/48'/0'/0'/2'` (ind 0x05), fp `aabbccdd`, stub `11223344`, mainnet, 2 chunks. Strings: `mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf` + `mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x`. canonical_bytecode_hex `040111223344aabbccdd050488b21e…078f`. xpub `xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc`.
- **V2_bip84_mainnet_1_stub_with_fp** — `m/84'/0'/0'` (ind 0x03), fp `deadbeef`, stub `c0ffee00`, mainnet, 2 chunks.
- **V3_bip48_testnet_1_stub_with_fp** — `m/48'/1'/0'/2'` (ind 0x15), fp present, testnet (tpub), 2 chunks.
- Corpus also carries (per the T2 recon) multi-stub no-fp, 3-chunk, and explicit-path (0xFE) vectors, plus negative/`expected_error` vectors (schema-2 reject cases).

**Provenance flag:** the Go port's embedded vectors MUST be lifted verbatim from `v0.1.json` (and the negative cases from its schema-2 reject entries), NOT round-tripped through the Go encoder (there is none) and NOT hand-derived. Cite the pinned SHA-256 from `tests/vectors.rs`.

### F9 — mk1 is PUBLIC — ACCURATE
mk1 carries an account-level **xpub** + fingerprint + path + opaque policy-id stubs. No secret material. → **no secrecy gate, no `wipeBytes` scrub, no `Unshared` probe** (contrast T2a/ms1, which is SECRET). Decode-display is unconditionally offered for any BCH-valid mk1. (Consistent with the SECURITY CONSTRAINTS: md1/mk1/xpub/descriptor are PUBLIC, NFC-ok.)

---

## Package placement
**NEW Go package** — `codex32` is deliberately pure-stdlib (BCH + bech32 only); mk1 decode needs `hdkeychain` / `btcec` / `chaincfg`. Put the decoder in a sibling package, e.g. `seedhammer.com/mk` (or `codex32/mkdecode`). It depends on `codex32` for `ValidMK` + symbol extraction primitives, plus the BIP-32 libs. Keeps `codex32` dependency-clean and matches the T2c/md1 split to come.

---

## Recommended brainstorm-session scope

**One T2b cycle, two implementation phases under a single spec** (mirrors T2a's package-then-GUI shape; the decoder is independently parity-verifiable, the GUI is net-new UX):

- **Phase A — `mk` decode package (deterministic core).** String-layer header parse (single/chunked, `+1` decode) → chunk reassembly (cross-chunk-hash verified) → `decode_bytecode` (header/stub_count/stubs/fp/path/compact-73) → `reconstruct_xpub` via `hdkeychain`/`btcec`. Public API ≈ `DecodeMK(strings []string) (MK1Card, error)` + a single-string-header probe for the gather loop (`needs N chunks`). Fully TDD'd against the Rust-sourced `v0.1.json` vectors (V1–V3 + multi-stub no-fp + 3-chunk + explicit-path + negative reject cases). ~550–750 LOC.
- **Phase B — GUI multi-string gather + decode-display flow.** New gather UX: accumulate scanned/typed mk1 strings, parse each chunked header to learn `total_chunks` + which indices are still missing, show progress ("chunk 2 of 3 — scan the remaining card"), reassemble + decode when complete, then a measure-and-advance display of {network, path, fingerprint, stub count, xpub} before the existing `mdmkFlow` engrave chooser. Display-only; no NFC/engrave mutation from the decode screen. ~400–600 LOC.

**SemVer / lockstep:** fork firmware feature; no constellation CLI surface, no GUI `schema_mirror`, no docs-manual mirror. No upstream PR (fork is the maintained line).

**Ordering / dependencies:** T2b (#8) blocks T2c (#9, md1 — shares the chunk-reassembly *pattern* but md-codec has its own chunk format, so the code is sibling-not-shared). T5 (#5) "guided bundle sequencing" now builds **on top of** T2b's gather UX rather than introducing reassembly from scratch — note this in T5's eventual spec.

**Gate reminder:** this recon feeds the spec. `SPEC_seedhammer_T2b_mk1_decode.md` MUST pass an opus-architect R0 to 0C/0I before any code; fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN.
