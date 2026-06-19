<!--
Persisted verbatim. opus-architect R1 GATE re-review of SPEC_seedhammer_T2b_mk1_decode.md
(folded, commit 472eca8). Reviewer agentId adaacd43465114b99. Verdict: GREEN 0C/0I. All three R0
blocking findings (C-1 chunk_index off-by-one, I-1 reject set / strict five_bit_to_bytes / negative-
test layering, I-2 phantom multi-stub-no-fp vector) verified CLOSED against mnemonic-key/crates/mk-codec
source with line-exact citations; all four MINORs folded (M-4 promoted to invariant 2.10); no fold-
introduced drift. Sole residual: a non-blocking 2-line citation imprecision (pipeline.rs:354→:352,
inherited from R0) — fixed in the GREEN commit. Disposition: GREEN — cleared to the implementation
plan (which carries its own R0 gate). The text below is the agent's report verbatim (HTML entities
un-escaped from the notification transport: &lt;→<, &gt;→>, &amp;→&).
-->

# R1 Gate Re-Review — SPEC_seedhammer_T2b_mk1_decode.md (folded, `472eca8`)

**Reviewer:** opus-architect | **Prior:** R0 NOT GREEN 1C/2I (`seedhammer-T2b-mk1-spec-review-R0.md`) | **Authoritative source:** `mnemonic-key/crates/mk-codec` (`family_token "mk-codec 0.2"`, schema 2). Per the R1 brief I re-verified ONLY the folds and scanned for fold-introduced drift; R0-cleared items (sizing/unreachability, bytecode layout, compact-73, path table, reassembly contract, reader lifecycle, package split, alloc gate) were not re-litigated.

## Fold verification

**C-1 — CLOSED.** The `chunk_index = value−1` falsehood is fully corrected in all three locations the R0 flagged, with exact-matching source citations:
- §2.2 (line 27): "`total_chunks` is stored **value−1** on the 5-bit wire (decode `+1`); `chunk_index` is stored **verbatim, 0-based (NO `+1`)**" — matches `header.rs:88` (`(total_chunks - 1) & 0x1F`), `:97` (`chunk_index & 0x1F`, verbatim), `:146` (decode `+1`), `:147` (decode, no `+1`). The added off-by-one hazard note ("slots `1..total`, last chunk trips `idx >= total_chunks`") is accurate.
- §3.5 (line 55): cites `:88`/`:146` for `total_chunks` and `:97`/`:147` for `chunk_index` verbatim — line-exact against source.
- §6 (line 130): "`total_chunks` decodes with `+1` (wire 1 ⇒ 2) and `chunk_index` round-trips with NO offset (0-based) — the R0-C1 guard." Consistent.
- §4.1 internals (line 101) also restated correctly ("`total_chunks` decode `+1`, `chunk_index` verbatim — §2.2"). No residual `value−1`-for-chunk_index claim survives anywhere.

**I-1 — CLOSED.** The reject set and negative-test layering are now complete and correctly stratified:
- Strict `five_bit_to_bytes` contract (§2.8 line 35, §4.1 line 101): "reject any symbol ≥ 32, `bits >= 5` leftover, OR non-zero trailing padding bits" — exactly matches `bch.rs:78-100` (`v >= 32` → None at :83; `bits >= 5` → None at :94; `(acc & ((1<<bits)-1)) != 0` → None at :97). The "MUST NOT reuse `codex32.parts.data()`" instruction stands.
- Bytecode reject variants (§2.8 line 36) all map to real `Error` variants confirmed in source: `ReservedBitsSet` (bits 0/1/3 — `header.rs:26` `RESERVED_MASK = 0b0000_1011`, exactly bits 0,1,3), `UnsupportedVersion`, `InvalidPolicyIdStubCount` (stub_count==0, `decode.rs:26`), `InvalidPathIndicator`, `PathTooDeep` (`decode.rs` test :240, cap 10), `InvalidPathComponent` (LEB128 overflow), `UnexpectedEnd`, `TrailingBytes` (`decode.rs:46`), `InvalidXpubVersion`, `InvalidXpubPublicKey`.
- String/header reject set (§2.8 line 34): `UnsupportedVersion`, `UnsupportedCardType` (reserved `0x02..=0x1F`), `ChunkedHeaderMalformed`, `MixedHeaderTypes`, chunk_set_id mismatch, total disagreement, dup/missing index, count≠total, `CrossChunkHashMismatch` — all confirmed in `header.rs:120-176` and `chunk.rs:54-200`. `MalformedPayloadPadding` confirmed a real variant (`error.rs:70`).
- Negative-test layering (§6 lines 127-129): correctly carves **N1-N5** as `ValidMK`/gather-layer rejects (before `mk.Decode`) and **N6-N23** as `mk.Decode` structural rejects, and explicitly instructs "assert *rejection per category*, NOT `expected_error` string equality, since Go error strings are independent of mk-codec's Rust rendering." This is sound and resolves the R0 conflation.

**I-2 — CLOSED.** The non-existent "multi-stub-no-fp" combined vector is gone from both §5 (line 119) and §6 (line 126). Replaced with real corpus vectors, each verified against `src/test_vectors/v0.1.json` (40 vectors, 18 clean + 22 negative):
- V1 = `m/48'/0'/0'/2'`, fp `aabbccdd`, 1-stub, mainnet, 2-chunk, `clean` ✓
- V2 = `m/84'/0'/0'`, fp, mainnet, 2-chunk, `clean` ✓
- V3 = `m/48'/1'/0'/2'`, testnet, 2-chunk, `clean` ✓
- V4 = `m/84'/0'/0'`, **1-stub, no-fp** (`origin_fingerprint: null`), 2-chunk, `clean` ✓
- V5 = explicit `m/9999'/1234'/56'/7'` (4-comp `0xFE`), fp, **3-chunk**, `clean` ✓
- V6 = **3-stub** (`dead0001/dead0002/dead0003`) **WITH fp** `f00dcafe`, `m/48'/0'/0'/2'`, 2-chunk, `clean` ✓
- V7 = explicit `m/0'/1'/2'/3'/4'/5'/6'/7'/8'/9'` (max 10-comp), **no-fp**, **3-chunk**, `clean` ✓

All 7 are `decoder_correction: "clean"`; the "use only clean vectors" instruction (§6 line 126) is correct given the fork's `ValidMK` does no BCH correction (R0-cleared finding #7). SHA pin `ebd8f34d8d52896e07e1faef995f18ffa61d42e2a048fb2a8c11e67f120d78ff` confirmed present at `tests/vectors.rs:41`. The "Rust-sourced only; never Go-derived" provenance rule is preserved.

**MINOR folds:**
- **M-1 — CLOSED.** §2.3 (line 28) now states "`childNum` MUST be the raw BIP-32 hardened-bit-encoded u32 (e.g. `0x80000002` for `2'`), matching `bip380.go:102`'s use of the raw `bip32.Path` u32 — not the unhardened index." Confirmed against `bip380/bip380.go:102` (`childNum = k.DerivationPath[len-1]`, raw u32 passed straight to `hdkeychain.NewExtendedKey`) and mk-codec `xpub_compact.rs:94-97` (`child_number = components.last().unwrap_or(Normal{0})`, the terminal `ChildNumber` carrying the hardened bit). Restated in §4.1 (line 101). Correct.
- **M-2 — CLOSED.** Alloc-gate citation updated to `gui_test.go:50-98` (§2.6 line 31, §4.2 line 105 cites `gui_test.go:50-98`). pipeline.rs citations `:73` (emit decision, confirmed `bytecode.len() <= SINGLE_STRING_LONG_BYTES` at `string_layer/pipeline.rs:73`) and `:161-164` (the 84-byte comment, confirmed at `:163`) are correct.
- **M-3 — CLOSED.** §3.6 (line 58) now cites `chunk.rs:109-203` (was `:109-196`) and the cross-chunk-hash check `:189-202`; matches source (`CrossChunkHashMismatch` at `chunk.rs:200`).
- **M-4 — CLOSED (promoted).** Display-paging is now a first-class invariant **2.10** (line 38): "Decode-display paging reaches the xpub tail, gap-free … First-class, execution-review-checkable." The invariant count grew from 9 to 10, consistent with §2.4/§2.5/§2.6 renumbering being unaffected (they kept their semantic content).

## Drift check

No fold introduced any new contradiction:
- §2.2 / §3.5 / §4.1 / §6 are mutually consistent on the `total_chunks` `+1` vs `chunk_index` verbatim distinction.
- §2.8 (full reject set) ↔ §4.1 (decoder internals) ↔ §6 (negative-test plan) are consistent: the strict `fiveBitToBytes` and the two-layer (`ValidMK` vs `mk.Decode`) split appear identically in all three.
- §5 manifest ↔ §6 TDD agree on the exact vector set (V1-V3, V4, V5/V7, V6) — the stale "multi-stub-no-fp" string is purged from both.
- `MAX_CHUNKS=32` (§2.8/§3.5) and `MAX_PATH_COMPONENTS=10` (§3.4) confirmed against `consts.rs:42,27`.
- `0x16` accept (§3.4) confirmed against `path.rs:53` (`m/48'/1'/0'/1'`, v0.2.0+).

## Findings

### CRITICAL
None.

### IMPORTANT
None.

### MINOR
- **m-1 (sub-trivial, non-blocking).** §2.1 and §3.1 cite the "smallest valid bytecode = 80 bytes" comment as `pipeline.rs:354`; the actual line is `string_layer/pipeline.rs:352` (2-line drift). The R0 itself cited `:354`, so this predates the fold and is within the noise band; not worth a re-dispatch. Suggest the implementation plan refresh it to `:352` when it pins exact lines. (The semantic claim — 80 B > 56 B single-string cap → always chunked — is correct.)

## Verdict

**GREEN — 0 Critical / 0 Important**

All three blocking findings (C-1 chunk_index off-by-one, I-1 incomplete/conflated reject set + reused-padding hazard, I-2 phantom multi-stub-no-fp vector) are closed against authoritative `mk-codec` source with line-exact citations, and all four MINORs are folded (M-4 promoted to invariant 2.10). Folding introduced no contradiction or citation regression; §2.2/§2.8/§3.5/§4.1/§5/§6 are internally consistent. The sole residual is a 2-line citation imprecision (`pipeline.rs:354`→`:352`) that is non-blocking and inherited from R0. The spec is cleared to proceed to `IMPLEMENTATION_PLAN_seedhammer_T2b_mk1_decode.md` (which carries its own R0 gate).
