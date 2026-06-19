# cycle-prep recon — 2026-06-18 — T2c md1 decode→display

**Fork HEAD at recon time:** `2fed9b6` (T2b merged).
**Design repo:** `mnemonic-engrave`, branch `master`, clean.
**md-codec source pinned for facts:** `descriptor-mnemonic/crates/md-codec` **@ 0.36.0** (authoritative Rust).
**Recon agents (parallel, both verified vs source):** `af714230a202cee4d` (md-codec wire format), `ad3b3d3588a7617db` (fork decode/display surface + bip380 leverage).

Slug: `T2c-md1-decode`. This recon independently re-verified the wire format against md-codec Rust source and the leverage surface against fork `2fed9b6`. **Headline: md1 is a recursive bit-packed AST — ~10× structurally heavier than mk1's flat byte-cursor — and the fork has no descriptor model rich enough to represent it, so the cycle is decode-to-template-display (not key-expanded descriptor projection), sub-scoped.**

---

## HEADLINE — scope decision (sub-scope T2c)

**This cycle (T2c, ledger #9): md1 decode-core + human-readable BIP-388 *template* display, for the renderable subset only; reject (never approximate) the rest.** This matches the roadmap's stated T2/md1 goal verbatim ("md1 → human-readable BIP-388 template") and covers all 10 md-codec corpus vectors (which are template-only — empty keys, `@N` placeholders).

**Deferred to a NEW follow-up item (T2c-b, ledger #10, blocked by #9): wallet-policy md1** — when the `Pubkeys` TLV is present, reconstruct the per-`@N` xpubs (65-byte chaincode‖pubkey → base58), project the renderable subset onto `*bip380.Descriptor`, and route through the existing `descriptorFlow`/`DescriptorScreen` for type/threshold/script + **free receive-address verification** via the in-tree `address` pkg (the T1 win). This is a clean wiring win **only for the bip380-expressible subset** and is genuinely separable.

Both recon agents independently and strongly recommended sub-scoping; this mirrors how T2 was itself sub-scoped (T2a/b/c).

---

## Verified facts

### F1 — BCH / version / family — ACCURATE
- Fork `codex32/mdmk.go` `ValidMD` BCH constants match md-codec `bch.rs` exactly: `MD_REGULAR_CONST = 0x0815c07747a3392e7` (`bch.rs:17` ⟷ `mdmk.go mdRegularTargetLo`, `mdRegularTargetHi=0x0`); `POLYMOD_INIT = 0x23181b3` (`bch.rs:32` ⟷ `mdmkPolymodInitLo`). Regular code only (md dropped the long code); 13-symbol checksum, BCH(93,80,8). `ValidMD` applies **no** data-part length bracket (only `len(data) >= 13`), unlike `ValidMK`. Pure verify, **no error correction** → the Go port targets **clean-decode only**.
- md-codec @ **0.36.0** (newest packaged). Wire-format version token `Header::WF_REDESIGN_VERSION = 4` (`header.rs:27`), 4-bit field; usable `{4,8,12}`. **No `family_token`/JSON envelope** — md-codec's corpus is a Rust `MANIFEST` (see F7).

### F2 — String layer: bit-packed, DIFFERENT from mk1 — ACCURATE
- **Single-payload header = 5 bits** (`header.rs:15-49`): `bit4 = divergent_paths flag`, `bits3..0 = version (=4)`.
- **Chunk header = 37 bits** (`chunk.rs:19-86`): first symbol `[v3][v2][v1][v0][chunked-flag]` = 4-bit version + 1-bit chunked flag, then 20-bit `chunk_set_id` + 6-bit `count-1` + 6-bit `index`.
- **In-band single-vs-chunked dispatch = bit 0 of the first 5-bit symbol** (the chunked flag) (`lib.rs:8-12`, `chunk.rs:599-617`). (Contrast mk1's separate `type` symbol.)
- `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 320` bits (64 data symbols) (`chunk.rs:219`); **max 64 chunks** (`chunk.rs:250`); per-chunk wire `= 37 + 8·|payload_bytes|` bits, byte-boundary split (`chunk.rs:284`).
- **Single-string IS reachable** for md1 (9/10 corpus vectors are single `md1…` strings ~26-34 chars) — unlike mk1 (always chunked). So the single-string path is a real, common case, not defensive-only.
- 5-bit↔byte repack MSB-first (`bitstream.rs:1-5`); `unwrap_string` returns a **symbol-aligned bit count = 5×data_symbols** (`codex32.rs:157`), tolerating ≤4 trailing padding bits. **CRITICAL:** the payload byte count must be recovered from the symbol-aligned bit count, **NOT `len(bytes)*8`** (`chunk.rs:316-328`; Rust calls out N=3, N=8 as the breaking cases). This is a net-new bit-reader concern mk1 never had.
- Cross-chunk integrity: `reassemble` **recomputes** the 20-bit `chunk_set_id` from the decoded descriptor's `Md1EncodingId` and checks it equals every chunk header's csid (`chunk.rs:305-389`, `ChunkSetIdMismatch`). (Contrast mk1's appended SHA-256 prefix.)

### F3 — The bit-packed recursive AST (the heavy part) — ACCURATE
- md1 payload is a fully bit-packed, sub-byte, **recursive** bitstream (`bitstream.rs` MSB-first `BitReader`/`BitWriter`). Decode order (`decode.rs:15-72`): Header(5b) → `PathDecl` (origin paths) → `UseSitePath` (multipath + wildcard) → compute `key_index_width (kiw) = ⌈log₂(n)⌉` → recursive `read_node` AST → root-tag allow-list check → `TlvSection` → post-decode validators.
- In-memory `Descriptor` (`encode.rs:16-28`): `n u8` (placeholder count 1..=32), `path_decl PathDecl`, `use_site_path UseSitePath`, `tree Node` (recursive miniscript AST), `tlv TlvSection`.
- **36-operator 6-bit Tag enum** (`tag.rs:13-137`): Wpkh`0x00` Tr`0x01` Wsh`0x02` Sh`0x03` Pkh`0x04` TapTree`0x05` Multi`0x06` SortedMulti`0x07` MultiA`0x08` SortedMultiA`0x09` … AndV`0x13` … Thresh`0x1A` After`0x1B` Older`0x1C` Sha256`0x1D` … RawPkH`0x21` False`0x22` True`0x23`; `0x3F` = extension prefix (reserved). `Body` is an 8-variant enum (`Children`, `Variable{k,children}` for Thresh, `MultiKeys{k,indices}`, `Tr{is_nums,key_index,tree}`, `KeyArg{index}`, hash/timelock bodies, `Empty`). `read_node` recurses with `MAX_DECODE_DEPTH = 128` (`tree.rs:185`).
- **`kiw` lockstep hazard:** kiw is computed independently at decode and MUST exactly match encode; a 1-bit drift anywhere (varint/tag width/kiw) silently desyncs the whole stream with **no post-BCH checksum to catch it** (`decode.rs:21-26`). This is the #1 porting risk.
- **TLV rollback:** the decoder tolerates ≤7 trailing padding bits and rolls back a phantom partial-TLV (`tlv.rs:215-304`); bit-limit scoping (`tlv.rs:359-376`) is load-bearing against malformed inflation.
- **Tr/NUMS variable width:** `is_nums` suppresses the `kiw` key-index field entirely (`tree.rs:268-292`); multi-family packs `tag(6)+k-1(5)+n-1(5)+n×index(kiw)` — off-by-one-prone.

### F4 — Paths & keys: explicit-only, NOT mk1's scheme — ACCURATE
- **Origin paths are explicit-only, no std-path dictionary** (contrast mk1's 14-entry `standardPaths` + `0xFE` + LEB128). `OriginPath = depth(4) + components`, each `hardened(1) + LP4-ext-varint(value)` (`origin_path.rs:28-66`); max 15 components.
- **LP4-ext varint** (`varint.rs:1-56`): `[L:4][payload:L]`; `L=15` ⇒ continuation, cap 2²⁹-1. (Replaces mk1's LEB128.)
- **Multipath `<0;1>`:** `UseSitePath.multipath = Option<Vec<Alternative>>`, 2..9 alts (`use_site_path.rs:43-45`); standard `<0;1>/*` is `standard_multipath()`.
- **Xpubs (wallet-policy mode only) NOT compact-73:** `Pubkeys` TLV carries **65 raw bytes each = 32-byte chaincode ‖ 33-byte compressed pubkey** (`tlv.rs:30-32`), indexed by `@N` at `kiw` bits. Fingerprints: 4 bytes each in the `Fingerprints` TLV (`tlv.rs:28`). **Template-only vectors have NO `Pubkeys` TLV** — `@N` placeholders with optional origin fingerprints/paths only.

### F5 — Representation gap (THE key risk) — ACCURATE
- md1 is a full miniscript descriptor format; the fork's only descriptor model `bip380.Descriptor` (`bip380.go:20-26`) has `Type ∈ {Singlesig, SortedMulti}` and `Script ∈ {7 BIP-388 wrapper types}`. So **only the `wsh(sortedmulti(k,…))` + singlesig (`wpkh/pkh/sh-wpkh/tr`) subset can be represented**; taptrees, nested miniscript, arbitrary thresholds, `multi` (non-sorted), NUMS, multipath-beyond-`<0;1>` have **no `bip380.Descriptor` form**.
- **Safety mandate (both recons):** the on-device display MUST be faithful or refuse. A naive projection could render a descriptor the operator's wallet would interpret differently. → decode fully; render the friendly template only for shapes we can render exactly; for the rest, an explicit "complex/unsupported policy — cannot display safely" (optionally key count + raw origins), NEVER an approximation.
- All 10 corpus vectors are **template-only** (empty `keys`) → they encode a BIP-388 template (`wsh(multi(2,@0/<0;1>/*,…))`), not a key-expanded descriptor. This is the realistic "show what's on the card" target and matches the roadmap goal.

### F6 — Fork leverage & reuse — ACCURATE
- **Current handling:** md1 + mk1 both scan to an undifferentiated `mdmkText` (`scan.go:70-71`); the prefix split happens in `mdmkFlow` via `hasMKPrefix`. T2c adds a third branch: `isMD := !isMK && md1-prefix` → prepend "Inspect descriptor", `choice==0` → `md1GatherFlow`→display. The `idx--` decrement generalizes to "decrement if an Inspect entry was prepended." **`TestMdmkFlowMD1NoInspect` (`gui/mk1_inspect_test.go`) currently asserts md1 has NO Inspect → T2c must update it.**
- **Gatherer reuse:** the `mk1Gatherer` *shape* (map[index]→string, prime-on-first, foreign/dup detection) + the `mk1GatherFlow` NFC-goroutine shell + `mk1DisplayFlow` measure-and-advance pager + `chunkString` are reusable patterns. Generalize the gatherer over a `parseHeader(string)→(setID,total,index,ok)` func; the per-format header parse + the data-symbol primitive differ (md1 needs `MDDataSymbols` + a **bit reader**, not mk1's symbol→byte repack).
- **`codex32.MDDataSymbols`** (net-new, analogous to `MKDataSymbols`): gate on `ValidMD`, strip the 13-symbol checksum, return the 5-bit data symbols. Trivially generalizable from `MKDataSymbols`. The consumer differs: md1 feeds a **bit reader** + tracks the symbol-aligned bit count (F2).
- **Alloc gate unchanged:** `TestAllocs` scope is still only `StartScreen.Flow` + `DescriptorScreen.Confirm` — new md1 screens are unconstrained. (Caveat: if T2c-b later routes md1 through `DescriptorScreen`, any new per-frame work there must stay 0-alloc.)
- **Test harness unchanged:** `runUI`/`ExtractText`/`uiContains`, `testPlatform.NFCReader()==nil` (→ pure gatherer must be unit-tested directly, as T2b did), `click`/`press`.
- **Package placement:** new `md` package, sibling to `mk`. Deps: `codex32` (ValidMD + MDDataSymbols) + a net-new bit reader. (T2c-b adds `bip380`/`bip32`/`hdkeychain`/`btcec` for xpub reconstruction.) **No import-cycle risk** (bip380/codex32/mk do not import gui).

### F7 — Parity corpus — ACCURATE (different shape from mk1/ms1)
- **Corpus = a Rust `MANIFEST: &[Vector]` (10 entries) at `md-codec/src/test_vectors.rs:42`, NOT a JSON file.** No pinned SHA-256, no `family_token`, no `schema`, no `decoder_correction` field. Per-vector artifacts live at `tests/vectors/<name>.{template,phrase.txt,bytes.hex,descriptor.json}` (`phrase.txt` = canonical md1 string; `bytes.hex` = payload; `descriptor.json` = decoded structure).
- **Go parity tests must be sourced from these per-vector files** (verbatim `phrase.txt` strings + `bytes.hex` + `descriptor.json` expectations), provenance = md-codec 0.36.0 `tests/vectors/`. The fork does no BCH correction → use the clean canonical codewords only.
- Representative CLEAN vectors (verbatim):
  - `wpkh_basic`: template `wpkh(@0/<0;1>/*)`, phrase `md1yqpqqxqq8xtwhw4xwn4qh`, bytes `2002001800`, decoded n=1, tree `Wpkh{KeyArg{0}}`, empty path/tlv.
  - `wsh_multi_2of3`: template `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))`, phrase `md1yzpqqxppsgsc8dua4tu0kekyl`, n=3, tree `Wsh{Multi{MultiKeys{k:2,indices:[0,1,2]}}}`.
  - `wsh_with_fingerprints`: template `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))`, phrase `md1yppqqxppsg2z7zdatd7aljh7h2lqp277wajaesknu`, n=2, tlv `fingerprints:[(0,deadbeef),(1,cafebabe)]`.
  - `wsh_multi_chunked` (`force_chunked`): template `wsh(multi(3,…))`, phrase 2 lines (`chunk-set-id: 0x157ae` then `md1fz4awqqpqsgqpsgvyyxqql8saf74dwdyqv`), bytes `2082001821842180`.
- Negative shapes (`error.rs` + decode checks): `WireVersionMismatch` (version≠4), `TagOutOfRange` (reserved 6-bit tag), `OperatorContextViolation{TopLevel}` (non-canonical root tag), `KGreaterThanN`, `PlaceholderIndexOutOfRange`, `TlvLengthExceedsRemaining`, `OverrideOrderViolation`, `Codex32DecodeError` (BCH/mixed-case), `ChunkSetIdMismatch`/`ChunkSetIncomplete`.

---

## Effort & phasing
- mk1 decode pkg (baseline) = `mk/mk.go` 404 LOC (+215 test). md1 core decode-relevant Rust ≈ 3,500-4,000 LOC. Go estimate: **decode-core ~900-1,300 LOC**; **template-render ~300-500 LOC** (hand-rolled — md-codec gets descriptor-string rendering free from rust-miniscript, which has no in-tree Go equivalent). ≈ 2.5-3× the mk1 cycle.
- **Phasing within T2c (one cycle, two implementation phases under one spec — the T2b pattern):**
  - **Phase A — `md` decode package:** `codex32.MDDataSymbols` + a MSB-first bit reader + single(5b)/chunked(37b) header parse + in-band dispatch + reassembly (recompute-csid integrity, symbol-aligned bit count) + recursive `read_node` AST + paths/varint/UseSitePath + TLV (fingerprints; pubkeys parsed-but-not-expanded). Decodes to an in-memory md1 `Template` struct. Parity-tested against the corpus `bytes.hex`/`phrase.txt`/`descriptor.json`.
  - **Phase B — template render + GUI:** AST → human-readable BIP-388 template string for the renderable subset (reject/flag the rest, faithfully); generalized gatherer + `md1GatherFlow` NFC shell + measure-and-advance display; "Inspect descriptor" affordance in `mdmkFlow`; update `TestMdmkFlowMD1NoInspect`.
- If Phase A alone proves too large for one implementer pass, the plan may split it further (bit-reader+chunk first, AST+TLV second). The R0 plan gate decides.

## Biggest risks (for the spec to lock)
1. **`kiw` lockstep + bit-cursor discipline** — no post-BCH checksum; a 1-bit drift corrupts everything silently. TDD against the corpus `bytes.hex` is the only safety net. Verify against Rust source, not the draft.
2. **Symbol-aligned bit count vs `len*8`** (F2) — breaks specific chunk counts (N=3, N=8).
3. **Representation gap / faithful-or-refuse display** (F5) — the spec MUST lock the exact renderable subset and mandate explicit refusal (never approximation) for everything else.
4. **TLV rollback + ≤7-bit padding tolerance** (vs mk1's strict zero-pad) — subtle to port.

## Recommended scope ordering / SemVer / lockstep
- T2c (this cycle, #9): decode-core + template display. T2c-b (new #10, blocked by #9): wallet-policy xpub-expansion → `bip380.Descriptor` → DescriptorScreen + address verification.
- Fork firmware feature; no constellation CLI surface, no GUI `schema_mirror`, no docs-manual mirror. No upstream PR.
- **Gate reminder:** `SPEC_seedhammer_T2c_md1_decode.md` MUST pass opus R0 to 0C/0I before any code; fold → persist verbatim → re-dispatch until GREEN.
