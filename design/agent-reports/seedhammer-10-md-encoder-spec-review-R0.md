# R0 GATE REVIEW — SPEC #10 (md1 encoder + chunked + xpub-expansion)

**Reviewer:** opus architect (R0 mandatory pre-implementation gate)
**Date:** 2026-06-19
**Artifact:** `design/SPEC_seedhammer_10_md_encoder.md`
**Feeds from:** `design/agent-reports/seedhammer-10-md-encoder-architect-blueprint.md`
**Authoritative sources verified against:** `descriptor-mnemonic/crates/md-codec` v0.36.0 @ `c85cd49` (Rust); `seedhammer` @ `e4ca173` (Go); `mnemonic-engrave/crates/me-cli` v0.3.0.

---

## VERDICT: GREEN

0 Critical / 0 Important. All Minors below are non-blocking. The SPEC is cleared to advance to the implementation-plan phase (which carries its own R0 gate).

I independently confirmed every locked wire-fact in SPEC §3 and blueprint §1–§3 against the actual Rust source text (not the prose). No drift found. The one previously-feared false-consensus item (csid = direct 20-bit hash projection) is correctly modeled in the SPEC as SHA-256→first-16-bytes→top-20-MSB-first.

---

## 1. Protocol-fact correctness (PRIMARY) — every locked fact verified

Each fact below was checked against the cited Rust line(s) AND, where the encoder must invert it, the shipped Go decoder.

| SPEC fact | Verified? | Authoritative source |
|---|---|---|
| MSB-first packing, last byte zero-padded on LOW bits | ✅ | `bitstream.rs:29-69` (write_bits places bits MSB-first via `byte_shift = free_in_byte - chunk`), `:81-83` (`into_bytes` returns buffer; trailing partial byte already low-padded since unwritten low bits are 0). Test `write_5_bits_msb_first` → `0b1011_0000` (`:237-243`). |
| single Header 5b, `(divergent<<4)|(version&0xF)`, version=4 | ✅ | `header.rs:30-33` (`(divergent_paths<<4) | (version&0b1111)`), `:27` (`WF_REDESIGN_VERSION = 4`). Test `header_common_case_byte_value` → byte `0x20` (`:113-125`). |
| chunked-flag = bit 0 of symbol 0 = 0 for single (4 = `0b00100`) | ✅ | `header.rs:118-119` (`0b00100`); Go `md.go:1207` `syms[0]&1 == 1` discriminator. |
| ChunkHeader 37b, **version in top 4 bits** (distinct from single Header low-nibble) | ✅ | `chunk.rs:1-6` (layout comment), `:51-55` (`write_bits(version,4)` FIRST, then chunked=1, csid:20, count-1:6, index:6). |
| `WireVersionMismatch{got:2}` trap | ✅ | `chunk.rs:67-71` (`read` rejects version≠4); `header.rs:100-110` (v0.x chunked misread → got=2); me-cli `bundle.rs:134-167` replicates the bit-0-first probe. |
| discriminator = `syms[0]&1`, NEVER `ChunkHeader::read`-then-catch | ✅ | me-cli `bundle.rs:144-151` reads `syms[0]&0x01` first; only calls `ChunkHeader::read` when set. Go `md.go:1207` same. |
| kiw = `32 − clz(n−1)`, clamp 0 at n∈{0,1}; n=1 ⟹ zero-bit key args | ✅ | `encode.rs:37-41` (`(32 - (n as u32).saturating_sub(1).leading_zeros())`); Go `md.go:842` (`32 - bits.LeadingZeros32(uint32(pd.n)-1)`). Test `key_arg_n1_zero_bits` (`tree.rs:336-346`) — at kiw=0, KeyArg emits 0 bits. |
| csid = SHA-256(payload)[0:16] → top-20-MSB-first; `AB CD EF`→`0xABCDE` | ✅ | `identity.rs:39-45` (first 16 bytes of SHA-256 over `encode_payload` bytes); `chunk.rs:175-179` (`(b0<<12)|(b1<<4)|(b2>>4)`). Test `derive_chunk_set_id_msb_first_extraction` → `0xABCDE` (`chunk.rs:199-207`). Go `mk/encode.go:316-319` `top20` is byte-identical arithmetic. **NOT a direct projection** — confirmed via the 16-byte intermediate `Md1EncodingId`. |
| varint LP4-ext, lengths in **BITS**; `[L:4][payload:L]`, L=15 escape, max 29 bits | ✅ | `varint.rs:15-42` (`bits_needed = 32 - value.leading_zeros()`; L=15 escape → `[L_high:4][low:14][high:L_high]`; overflow at l_high>15). Tests: `varint_84_costs_11_bits` (`:124`), `varint_max_u31` = `(1<<29)-1` (`:102-107`). Bits-not-bytes pinned by `identity.rs:498-513` golden (varint(26)=9 bits because L=5). |
| TLV: per-entry sub-bitstreams, emitted sorted-by-tag asc, `[tag:5][varint(bitLen)][reEmitBits]` | ✅ | `tlv.rs:200` (`entries.sort_by_key(tag)`), `:202-206` (`write_bits(tag,5)` + `write_varint(bit_len)` + `re_emit_bits(payload,bit_len)`). |
| TLV tag width = 5 bits (SEPARATE from 6-bit tree tag) | ✅ | `tlv.rs:203` `write_bits(tag, 5)`; tree tag is 6-bit primary `tag.rs:142`. The two tag spaces are distinct (`tag.rs:7-8` comment). SPEC §3 `[tag:5]` and blueprint trap (D) "6-bit primary" both correct, applied to different namespaces. |
| TLV idx strictly ascending; empty rejected; unknown re-emit via reEmitBits | ✅ | `tlv.rs:106-116` (`OverrideOrderViolation` on `idx<=prev`), `:100-104` (`EmptyTlvEntry` on empty vec), `:197-199` (unknowns re-pushed and re-emitted via the same `re_emit_bits` path). Go `md.go` retains `tlvUnknown{tag,payload,bitLen}` (`:516-520`) + `readUnknownPayload` MSB-first pack (`:621-655`). |
| is_nums kiw-suppression; n=1 ⟹ kiw=0 | ✅ | `tree.rs:151-154` (writes is_nums:1; key_index ONLY if `!is_nums`); Go `md.go:431-443` inverse. Tests `tr_nums_flag_round_trip` (`tree.rs:540`), `tr_nums_n_1_bare_round_trip` (`:693`). |
| MultiKeys raw kiw-width indices (not child nodes); `(k-1):5, (n-1):5, n×index@kiw` | ✅ | `tree.rs:115-139` (writes k-1:5, indices.len()-1:5, then raw indices @kiw); Go `md.go:385-407`. Test `sortedmulti_2of3_bit_cost` = 22 bits (`tree.rs:413-425`). |
| split threshold 320 bits (=64×5), count≤64, byte-aligned | ✅ | `chunk.rs:219` (`SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64*5`), `:249-259` (`div_ceil(320)`, `>64 → ChunkCountExceedsMax`), `:262` (`bytes_per_chunk = len.div_ceil(count)`), `:281-284` (per byte `write_bits(b,8)`). |
| ChunkHeader count stored count−1, index verbatim | ✅ | `chunk.rs:54` (`(count-1) as u64, 6`), `:55` (`index, 6`). Read `:77` (`+1`). |
| Reassemble integrity = decode→re-encode→derive-csid→compare | ✅ | `chunk.rs:376-386` (`decode_payload` → `compute_md1_encoding_id` → `derive_chunk_set_id` → compare to header csid → `ChunkSetIdMismatch`). This is exactly why the READ half needs the full encoder — SPEC §1's deciding fact confirmed. |
| MDChecksumSymbols POLYMOD_INIT = `0x23181b3`, regular-only (13 syms) | ✅ | `codex32/mdmk.go:39` (`mdmkPolymodInitLo = 0x23181b3`), `:55-56` (md regular target hi=0, lo=`0x0815c07747a3392e7`), `:41` (`mdmkShortSyms = 13`). `MKChecksumSymbols` precedent `codex32/mkencode.go:18-55` uses the same engine; `ValidMD` is regular-only (`mdmk.go:124-127`). |

**Conclusion:** No factual drift. Every value the encoder will emit is pinned to a verified source line.

---

## 2. Completeness of scope + invariants

The 9 invariants (I-1..I-9) are correct and sufficient. Sub-writers enumerated in A2 cover the full renderable AST. Cross-checks:

- **All AST node bodies covered.** `write_node` Rust arms (`tree.rs:81-174`): KeyArg, Children, Variable(Thresh), MultiKeys, Tr, Timelock, Hash256Body, Hash160Body, Empty — each has a Go decoder inverse (`md.go:339-...`) the encoder must mirror. SPEC A2 names the validators; the body-writers are implied by "all sub-writers invert their `read*` counterpart" — adequate at spec altitude (the impl plan should enumerate the 9 body arms explicitly; see Minor M-3).
- **Encode-side validators present.** SPEC A2 lists ThresholdOutOfRange / ChildCountOutOfRange / KGreaterThanN / OverrideOrderViolation / EmptyTlvEntry / VarintOverflow / path-depth / alt-count. Verified against the Rust encoder-side guards: `tree.rs:92-108` (k,n∈1..=32, k≤n), `tlv.rs:100-116` (empty + ordering), `varint.rs:30-31` (overflow), `origin_path.rs:55-60` (PathDepthExceeded), `use_site_path.rs:82-83` (AltCountOutOfRange), `origin_path.rs:111-113` (KeyCountOutOfRange). Plus `encode.rs:69-77` runs `validate_placeholder_usage`, `validate_multipath_consistency`, and `validate_tap_script_tree` BEFORE emission — SPEC A2 captures these as "the encode-side validators ported."
- **secp256k1 on-curve check** correctly placed in B2 (the decoder no-ops it, `md.go:1071-1077`; Rust `validate.rs:221` does `PublicKey::from_slice(&xpub[32..65])`). In-scope and correct.
- **canonicalize permutation of TLV maps** — covered by I-1/A3, verified atomic over tree + divergent paths + all 4 TLV maps (`canonicalize.rs:204-232`).

No load-bearing element is missing.

---

## 3. R0-focus items — explicit rulings

### 3(a) — Is `canonicalize` correctly mandatory and in-scope? **RULING: YES, correct.**
- The Go decoder NEVER ported `canonicalize`: it instead *rejects* non-canonical wires via `validate_placeholder_usage` (Rust `canonicalize.rs:26-29` documents this; Go `md.go` enforces placeholder ordering on read). So the encoder is genuinely the first Go code that needs `canonicalize_placeholder_indices`.
- `encode_payload` canonicalizes on a clone FIRST (`encode.rs:66-68`), so csid is over canonical bytes (`identity.rs:40` hashes `encode_payload` output).
- READ-path inputs are always canonical (decoder rejects otherwise), so canonicalize is a near-no-op there (identity fast-path `canonicalize.rs:199-201`). But T6's author-built ASTs (whose write path lands in #10) can be non-canonical → mandatory. I-1's CRITICAL tag is justified.

### 3(b) — Is byte-exact `.bytes.hex` golden parity the right PRIMARY gate? **RULING: YES, correct and achievable.**
- The md1 goldens ARE SHA-derived canonical (`identity.rs:39-45`), so byte-equality is achievable — unlike mk1's arbitrary csids (where `mk/encode_test.go` can only assert structural round-trip). I independently confirmed two cited values byte-for-byte: `wpkh_basic.bytes.hex = 2002001800` and `wsh_with_fingerprints.bytes.hex = 204200182182142f09bd5b7ddfcafebabe`. Both vendored goldens exist with matching `.descriptor.json`. The 10-vector manifest (pkh_basic, sh_wsh_multi, tr_keyonly, wpkh_basic, wsh_divergent_paths, wsh_multi_2of2, wsh_multi_2of3, wsh_multi_chunked, wsh_sortedmulti, wsh_with_fingerprints) covers single + chunked + fingerprints + divergent + sortedmulti + tr-keyonly + sh(wsh) — the full renderable surface.

### 3(c) — Is the xpub-expansion projectable-subset boundary correct and safe? **RULING: YES, correct.**
- bip380 truly cannot express unsorted multi / multi_a / sortedmulti_a / taptree: `bip380.go:332-337` accepts ONLY `sortedmulti` for the multi type; `MultisigType` has only `Singlesig` and `SortedMulti` (`bip380.go:90-95`); scripts are wpkh/pkh/sh/wsh/tr + P2SH-wrapped (`bip380.go:298-330`). Display-only + address-verify-excluded for non-bip380 shapes (I-6) is the right faithful-or-refuse call — verifying a non-sorted multisig against a sorted address would mis-render the spend path. `verifyAddressFlow(ctx, th, desc *bip380.Descriptor)` signature (`gui/verify_address.go:22`) matches B3.

### 3(d) — Is the bit-0 discriminator used everywhere (no `ChunkHeader::read` probe as primary)? **RULING: YES, correct.**
- I-3 and SPEC §3 line 42 mandate `syms[0]&1` everywhere; me-cli `bundle.rs:144-151` is the precedent (reads bit 0 first, only calls `ChunkHeader::read` on chunked). Go `md.go:1207` already uses it for the refusal path. The SPEC correctly forbids `ChunkHeader::read`-then-catch as primary (it would spuriously emit `WireVersionMismatch{got:2}` on single strings, `chunk.rs:67-71` × `header.rs:31`).

---

## 4. One-cycle vs split (SPEC §2 split note) — RULING

**Recommendation: SPLIT into #10a (Phase A headless codec) → #10b (Phase B GUI), at the documented A/B boundary.**

Rationale:
1. **Clean, real boundary.** Phase A is pure headless codec (bits/encode/canonicalize/identity/chunk/codex32) — no GUI, no allocator-gate surface. Phase B is GUI gather + bip380 projection + secp256k1 + verifyAddressFlow wiring. The dependency is strictly A→B; no back-edge.
2. **Phase A is independently gate-able AND independently valuable.** Its acceptance (byte-exact `.bytes.hex` parity + Reassemble round-trip + identity pins) is fully self-contained and is the literal T5+T6 foundation. T5 (bundle sequencing) needs `Reassemble`/`split`/`ParseChunkHeader`; T6 (seed→md1) needs `encodePayload`/`split` — neither needs Phase B. Shipping A first de-risks both downstream cycles earlier.
3. **Cognitive load.** ~950 LOC + comparable tests ≈ 1900 LOC of new surface, spanning bit-twiddling fidelity (A) AND GUI/secp256k1/descriptor-projection (B) — two very different review modes. A single mandatory whole-diff adversarial exec review over 1900 LOC mixing both is the weakest link; splitting gives each reviewer a coherent, smaller diff in one mode.
4. **The CRITICAL traps (I-1 canonicalize, I-2 bitWriter/reEmitBits) all live in Phase A.** Gating A to GREEN first means the byte-fidelity spine is proven before any GUI work builds on it — exactly the TDD ordering I-2 demands.

The author's lean ("one cycle if the implementer + exec-review can hold it; otherwise split") is reasonable, but the balance tips to split: the boundary is clean, Phase A is valuable alone, and the exec-review-load argument is decisive given the project's mandatory non-deferrable whole-diff review. This is a recommendation, not a gate condition — the SPEC remains GREEN either way, since it already documents the boundary and Phase A's standalone acceptance criteria.

---

## 5. Faithfulness / security spine — confirmed

- **No secret material touched.** md1/descriptor/xpub are public; xpub-expansion handles only public points (`Pubkeys` TLV = 32B chaincode ‖ 33B compressed pubkey, `tlv.rs:29-32`). No seed/xprv/ms1 emission. No CSPRNG (csid deterministic, `identity.rs:40` + `chunk.rs:175`). SPEC §4 accurate.
- **Faithful-or-refuse for non-bip380 shapes** — I-6 / §B2 / blueprint §4 correctly exclude unsorted multi / multi_a / taptree / miniscript from address-verify (display-only). The decoder's `classifyPolicy` (`md.go:1237-1293`) already returns `PolicyComplex`/non-renderable for these; B2 narrows further to bip380-expressible. Correct: address-verify only for singlesig + sortedmulti.
- **No mis-rendering of spend paths** — §4 + I-6 capture the T2c discipline. Confirmed.

---

## 6. Test strategy adequacy — confirmed sufficient

The 8-point acceptance gate (§5) plus the golden-vector vendoring is sufficient to PROVE byte-fidelity and catch the enumerated traps:
- §5.1 byte-exact parity (PRIMARY) — the strongest oracle; SHA-derived goldens make it achievable (3b).
- §5.2 full-string parity exercises `MDChecksumSymbols`; `ValidMD` is the independent BCH check (I-7).
- §5.4 chunked round-trip + drop-chunk + corrupt-csid, using the verified `chunked_md1_vector` fixture (me-cli `bundle.rs:547-585`: 6-key wsh-sortedmulti, 15-deep divergent, ≥4 chunks via `split`).
- §5.5 identity pins (`0xABCDE`, determinism, path-sensitivity).
- §5.8 fuzz for 0 panics on all encode/decode/reassemble/ParseChunkHeader paths.
- Baseline confirmed: `go build ./md/... ./mk/... ./codex32/... ./bip380/...` is currently GREEN (throwaway build; EXIT 0), so §5.7 no-regression has a clean starting point. `md/bits.go` is confirmed reader-only (no `bitWriter`/`reEmitBits`), validating the A1 net-new claim and the I-2 TDD ordering.

**One addition the TDD plan should adopt (Minor M-1):** port the Rust bit-cost UNIT tests (the `tree.rs` / `origin_path.rs` / `use_site_path.rs` / `varint.rs` `*_bit_cost` / `*_costs_N_bits` tests) as Go unit tests on the new writers, run BEFORE the structural-encoder golden tests — I-2 calls for the bitWriter golden tests (`bitstream.rs:236-407`) first, and the per-node bit-cost pins (e.g. `sortedmulti_2of3_bit_cost = 22`, `use_site_path_standard_bit_cost = 16`, `origin_path_bit_cost_bip84 = 26`, `varint_84_costs_11_bits`) are the cheapest way to localize a fidelity bug to a single writer before the SHA swallows it in the integration golden.

---

## MINOR findings (non-blocking; listed for the impl plan)

- **M-1 (test ordering):** Add the per-writer bit-cost unit tests (port from Rust `*_bit_cost`/`*_costs_N_bits`) ahead of the `.bytes.hex` integration goldens, per I-2. (See §6.)
- **M-2 (citation imprecision):** SPEC §B2 and blueprint §4 cite the secp256k1 check as `validate.rs:216`; the actual `PublicKey::from_slice(&xpub[32..65])` call is `validate.rs:221` (function/doc block begins ~211). Harmless — fix the line number in the impl plan.
- **M-3 (enumeration at impl-plan altitude):** SPEC A2 says "all sub-writers invert their `read*` counterpart" without naming the 9 `Body` arms. Fine for a spec; the impl plan should enumerate them (KeyArg / Children / Variable / MultiKeys / Tr / Timelock / Hash256Body / Hash160Body / Empty) to guarantee none is dropped — `tree.rs:81-174` is the checklist.
- **M-4 (chunked-golden nuance):** The `wsh_multi_chunked` golden is a small force-chunked vector (its `.phrase.txt` carries `chunk-set-id: 0x157ae` + a single short `md1...` string, `.bytes.hex = 2082001821842180`), NOT the ≥4-chunk case. The ≥4-chunk multi-chunk round-trip correctly comes from the `chunked_md1_vector` fixture (me-cli `bundle.rs:547-585`). The SPEC already uses the right source for §5.4; just be aware the two are different artifacts so the test harness doesn't conflate them.
- **M-5 (Reassemble bit-count detail):** When porting `Reassemble`, replicate Rust's use of the **symbol-aligned bit count** from `unwrap_string` (NOT `len(bytes)*8`) to recover `payload_byte_count = (symbol_aligned_bit_count - 37) / 8` (`chunk.rs:315-328`). Using `bytes.len()*8` over-estimates by up to 7 bits and breaks round-trip for certain N (e.g. N=3, N=8). The Go `ParseChunkHeader`/`Reassemble` must thread the symbol-aligned bit count, not the byte length. (The SPEC's A6 "byte-aligned slicing" wording is about the WRITE side; this is the READ-side counterpart the impl plan must lock.)

---

## Verification artifacts (throwaway)
- `go build ./md/... ./mk/... ./codex32/... ./bip380/...` in `/scratch/code/shibboleth/seedhammer` — EXIT 0 (no-regression baseline).
- Direct reads of all cited Rust files in `descriptor-mnemonic/crates/md-codec/src/` @ `c85cd49` and Go files in `seedhammer/` @ `e4ca173` plus `mnemonic-engrave/crates/me-cli/src/bundle.rs`.
- `cat` of the 10 vendored `.bytes.hex` + two `.descriptor.json` to confirm SPEC §5.1 byte values.

No source was modified.
