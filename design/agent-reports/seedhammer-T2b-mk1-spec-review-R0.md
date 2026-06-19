<!--
Persisted verbatim. opus-architect R0 GATE review of SPEC_seedhammer_T2b_mk1_decode.md
(commit 3dec65f) BEFORE any code. Reviewer agentId a3f20113743592b14. Verdict: NOT GREEN — 1C/2I.
The gate caught a false load-bearing protocol fact (C-1: chunk_index is NOT stored value-1 — only
total_chunks is; verified header.rs:88 vs :97, decode :146 vs :147). I independently re-confirmed
C-1, I-1 (strict five_bit_to_bytes padding rejection in bch.rs; fork ValidMK does NO BCH correction),
and I-2 (no multi-stub-no-fp vector exists — V6=3-stub WITH fp, V4/V12=1-stub no-fp, V5/V7=explicit
3-chunk; SHA pin ebd8f34d...78ff) against mnemonic-key/crates/mk-codec source before folding.
Disposition: folding C-1/I-1/I-2 + MINORs → re-dispatch R1. The text below is the agent's report
verbatim (HTML entities un-escaped from the notification transport: &lt;→<, &gt;→>, &amp;→&).
-->

# R0 Gate Review — SPEC_seedhammer_T2b_mk1_decode.md

**Reviewer:** opus-architect | **Base:** fork `4d02021` (confirmed HEAD) | **Authoritative source:** `mnemonic-key/crates/mk-codec` (`family_token "mk-codec 0.2"`, schema 2)

## Verification Results

### Protocol facts (verified against mk-codec source, not the spec/recon)

**1. Sizing / single-string unreachability — CONFIRMED.**
`consts.rs:33` `SINGLE_STRING_LONG_BYTES = 56`; `consts.rs:53` `XPUB_COMPACT_BYTES = 73`. Emit decision `pipeline.rs:73` is `if bytecode.len() <= SINGLE_STRING_LONG_BYTES` (the `<=`, spec writes `<= 56` — correct). Comment `pipeline.rs:163-164` states a typical card "= 84 bytes; this exceeds SINGLE_STRING_LONG_BYTES (= 56) and therefore lands in the chunked path"; `pipeline.rs:354` "smallest valid bytecode = 80 bytes > 56-byte single-string capacity." Single-string is structurally unreachable for any real card. Gather + reassembly are mandatory. The spec's headline scoping is correct.

**2. Bytecode layout & decode order — CONFIRMED (with extra reject paths the spec under-enumerates).**
`encode.rs:5-12` / `decode.rs:19-56`: `header(1) | stub_count(1,≥1) | stubs(4×N) | [fp(4) iff bit2] | path(var) | xpub_compact(73)`. Decode cursor order matches §3.2. `stub_count==0` → `InvalidPolicyIdStubCount` (`decode.rs:26`). Fingerprint flag = bit 2 (`header.rs:23` `FINGERPRINT_FLAG_MASK = 0b0000_0100`). **Additional rejects not in §2.8:** `ReservedBitsSet` (bits 0/1/3, `header.rs:26,45`), `UnsupportedVersion` (`header.rs:42`), `TrailingBytes` (`decode.rs:46`), `UnexpectedEnd` (truncation, `decode.rs:59`).

**3. compact-73 + xpub reconstruction — CONFIRMED.**
`xpub_compact.rs:8-15` layout `version(4)|parent_fp(4)|chain_code(32)|public_key(33)`. MAINNET `0x0488B21E` (`:25`), TESTNET `0x043587CF` (`:28`). `reconstruct_xpub` (`:86-108`): `depth := components.len()`, `child_number := components.last().unwrap_or(Normal{0})`, `version_to_network` rejects unknown version (`:67` `InvalidXpubVersion`), public key validated via `PublicKey::from_slice` (`:98` → `InvalidXpubPublicKey`). Version is also validated eagerly at decode time (`:126`). Empty-path → `Normal{0}` case confirmed (`:94-97`, test `:174`). The fork already implements the identical Go pattern at `bip380/bip380.go:97-113` (`hdkeychain.NewExtendedKey(version, keyData, chainCode, parentFP, depth, childNum, false).String()`).

**4. Path codec — CONFIRMED.**
`path.rs:38-55` — 14 entries exactly as the recon table (mainnet `0x01..0x07`, testnet `0x11..0x17`). `0x16 = m/48'/1'/0'/1'` added in 0.2.0 (`:53`; corpus V18 exercises it). `0xFE` explicit (`:28,103`): count `0..=10` (`MAX_PATH_COMPONENTS=10`, `:114` `count > MAX → PathTooDeep`), `0`=empty path (`:117`), LEB128 u32 with hardened bit in high bit (`:122` `raw & 0x8000_0000`). Invalid indicators → `InvalidPathIndicator` (`:109`). **Extra path rejects:** `PathTooDeep`, `InvalidPathComponent` (LEB128 overflow `:159`), `UnexpectedEnd`.

**5. String-layer header — DISCREPANCY (load-bearing).**
`header.rs`: Single = 2 symbols `version+type=0x00` (`SINGLE_HEADER_SYMBOLS=2`, `:20`). Chunked = 8 symbols `version+type=0x01+chunk_set_id(4 sym big-endian)+total_chunks+chunk_index` (`:24,89-98`). **`total_chunks` is stored value−1** (`:88` encode `(total_chunks - 1)`, `:146` decode `+1`) — confirmed. **But `chunk_index` is stored VERBATIM (0-based), NOT value−1**: `:97` emits `chunk_index & 0x1F`, `:147` decodes `symbols[7] & 0x1F` with **no `+1`**. The spec asserts (twice — §2.2, §3.5, and again §6 "header parse") that *both* `total_chunks`/`chunk_index` are value−1. That is wrong for `chunk_index`. See CRITICAL-1.

**6. Reassembly contract — CONFIRMED.**
`chunk.rs:109-203`. All chunks share version/chunk_set_id/total_chunks, only chunk_index varies (`:149-156`). `cross_chunk_hash = SHA-256(canonical_bytecode)[0..4]` appended to the stream before split (`:67-70`), verified at reassembly over `stream[..len-4]` (`:195-199` → `CrossChunkHashMismatch`). Reject conditions all present: empty list, single-string-at-head (`ChunkedHeaderMalformed` `:123`), single-string mid-set (`MixedHeaderTypes` `:176`), chunk_set_id mismatch (`:149`), total_chunks disagreement (`:152`), idx≥total (`:158`), duplicate (`:163`), missing (`:184`), count≠total (`:131`).

**7. Fork string-layer alignment — CONFIRMED.**
`codex32/mdmk.go`: `mkRegularTargetHi=0x1/Lo=0x62435f91072fa5c` → `0x1062435f91072fa5c` = `consts.rs:18`; `mkLongTargetHi=0x418/Lo=0x90d7e441cbe97273` → `0x41890d7e441cbe97273` = `consts.rs:21`; `mdmkPolymodInitLo=0x23181b3`. `ValidMK` (`:136`) does per-string-only BCH validation (regular 14..=93 / long 96..=108, rejecting 94..=95 and out-of-range), **no error correction, no header parse, no reassembly**. All net-new. Note `ValidMK` is a *pure verifier* — it does NOT correct (unlike mk-codec's `decode_string`, which auto-corrects t≤4).

**8. Parity vectors — CONFIRMED, richer than spec claims.**
`tests/vectors.rs:41` pins SHA-256 `ebd8f34d8d52896e07e1faef995f18ffa61d42e2a048fb2a8c11e67f120d78ff`; `family_token "mk-codec 0.2"` (`:132`), schema 2, 18 clean + 22 negative (40 total). V1 (`m/48'/0'/0'/2'`, 0x05, fp `aabbccdd`, stub `11223344`, mainnet, 2-chunk, xpub `xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc`), V2 (`m/84'/0'/0'`, 0x03, fp `deadbeef`, mainnet, 2-chunk), V3 (`m/48'/1'/0'/2'`, 0x15, fp `10203040`, testnet, 2-chunk) all present and match. Explicit-path (`0xFE`): V5 (4 comps, 3-chunk) and V7 (10 comps, 3-chunk). 3-chunk: V5, V7. **"multi-stub-no-fp" as a single vector does NOT exist** — V6 is multi-stub *with* fp; V4/V12/V16/V17 are 1-stub no-fp. See IMPORTANT-2. Negative corpus N1-N23 covers every reject path.

### GUI / feasibility checks

**Reader lifecycle (§4.2) — CONFIRMED.** `StartScreen.Flow` (`gui.go:1519`) opens its NFCReader at `:1521` with a `defer` (`:1524-1528`) that closes the reader and joins the scanner goroutine (`<-closed`). `uiFlow` (`:1480-1502`) calls `engraveObjectFlow` (`:1499`) only *after* `Flow` returns (`:1481`), so the reader is closed before `mdmkFlow` runs. A gather sub-screen invoked from `mdmkFlow` can safely own a fresh scanner+NFCReader — **no two-reader contention**. The scan loop returns exactly one object per `Flow` return (`:1605`), confirming no existing multi-string accumulation.

**`MKDataSymbols` primitives (§4.1) — CONFIRMED feasible.** `codex32` exposes `splitHRP` (`codex32.go:453`), `feFromRune`/`feFromInt`, and `Alphabet` (`:21`). `MKDataSymbols` can map data chars → 5-bit and trim the 13/15-symbol checksum by re-applying `ValidMK`'s length bracket. **Caveat:** `codex32.parts.data()` (`:417`) does 5-bit→8-bit unpacking but is tied to the codex32 share layout (threshold/id/shareIdx) and silently zero-pads (panics on `rem>4`) — it is NOT reusable for mk1; `mk` must implement its own strict `five_bit_to_bytes` (rejecting non-zero pad bits per `bch.rs:78-100`). See IMPORTANT-1.

**Package split — CONFIRMED sound.** `codex32` stays pure-stdlib; `mk` depends on `hdkeychain`, `btcec/v2`, `chaincfg/v2` — all direct deps in `go.mod:7-9`, module-mode (no vendor dir), already used in-fork (`bip380.go:104`, `biptool/main.go:166`). No existing `mk/` dir or `MKDataSymbols`.

**Alloc gate (§2.6) — CONFIRMED.** `TestAllocs` (`gui_test.go:93`) → `BenchmarkAllocs` (`:50`) exercises *only* `StartScreen.Flow` (`:66`) and `DescriptorScreen.Confirm` (`:69`). New screens are not alloc-gated. (Spec cites `gui_test.go:39-96`; actual range `:50-98` — citation slightly off, MINOR.)

---

## Findings

### CRITICAL

**C-1. `chunk_index` is NOT stored value−1 — the spec asserts it is (3×). A literal implementation produces an off-by-one that breaks reassembly.**
Where: §2.2 invariant, §3.5 ("`total_chunks`/`chunk_index` stored as value−1 (decode `+1`)"), §6 ("the `+1` boundary"). Source: `header.rs:88` (`total_chunks - 1`) vs `:97` (`chunk_index & 0x1F`, verbatim) and `:146` (decode `+1`) vs `:147` (decode, **no `+1`**). Only `total_chunks` carries the off-by-one; `chunk_index` is a 0-based index emitted/read as-is. Why it matters: a Go reader that adds 1 to `chunk_index` per the spec yields indices `1..total` instead of `0..total-1`, tripping `chunk_index >= total_chunks` rejection on the last chunk and/or assembling fragments in the wrong slots → wrong xpub or spurious decode error. This is precisely the "plausible-but-wrong load-bearing fact" class R0 exists to catch. Fix: rewrite the invariant/§3.5/§6 to read "**`total_chunks` stored value−1 (decode `+1`); `chunk_index` stored verbatim, 0-based (no `+1`).**" Add a header-parse test asserting `chunk_index` round-trips with no offset.

### IMPORTANT

**I-1. The §2.8 reject set and §6 negative-test plan are incomplete and conflate two rejection layers.** Where: §2.8, §3.2, §3.4, §6. The spec's reject enumeration omits real mk-codec reject paths the Go decoder must replicate: `ReservedBitsSet` (header bits 0/1/3), `UnsupportedVersion` (bytecode + string-layer header), `TrailingBytes`, `UnexpectedEnd`/truncation, `MalformedPayloadPadding` (per-chunk non-zero pad bits — `bch.rs:78`, corpus N7), `PathTooDeep`/`InvalidPathComponent` (distinct from `InvalidPathIndicator`), `UnsupportedCardType` (reserved string-header type `0x02..=0x1F`, N6), and the long/regular `five_bit_to_bytes` strict-padding contract. Additionally, negative vectors split across two layers: **N1-N5** (`invalid HRP`, `mixed case`, `length`, `invalid char`, `BCH uncorrectable`) are rejected by `ValidMK`/the BCH layer (the fork does *no* error correction, unlike mk-codec's t≤4 correction — so the fork rejects N5 at the gate with a *different* path/message than the corpus's `expected_error`); **N6-N23** are post-BCH structural rejects caught by `mk.Decode`. Why it matters: §6 says "from the corpus's schema-2 reject entries" implying byte-equal `expected_error` matching, but the fork's Go error strings are independent of mk-codec's Rust error rendering, and N1-N5's inputs never reach `mk.Decode`. Fix: (a) enumerate the full reject set in §2.8 including the padding/reserved/trailing/version/card-type/path-depth cases; (b) state explicitly that negative tests assert *rejection (no panic, no partial Card)* per category, NOT `expected_error` string equality; (c) carve N1-N5 as `ValidMK`/gather-layer rejects, N6-N23 as `mk.Decode` rejects; (d) require a strict per-chunk `five_bit_to_bytes` (reject non-zero pad bits), NOT reuse of `codex32.parts.data()`.

**I-2. The "multi-stub-no-fp" parity vector named in §6 does not exist in the corpus.** Where: §6 ("plus multi-stub-no-fp"), §5 manifest ("multi-stub-no-fp"). The corpus has V6 (3-stub *with* fp) and several 1-stub *no-fp* vectors (V4/V7/V12/V16/V17), but no single vector that is both multi-stub and no-fp. Why it matters: the spec mandates embedding a vector that isn't there; an implementer either invents a non-Rust-sourced vector (violating the "Rust-sourced only, never Go-derived" provenance rule in §2.2/§6) or silently drops a coverage case. Fix: cite the actual vector names to embed — e.g. V6 (3-stub with fp) for the multi-stub path and V4 or V7 (1-stub no-fp) for the no-fp path — and drop the non-existent "multi-stub-no-fp" combined vector (or note it as an optional constructed case, clearly flagged non-parity).

### MINOR

**M-1. `child_number` must be passed to `hdkeychain.NewExtendedKey` as the raw hardened-bit-encoded u32.** §2.3 says `child_number := last_component(origin_path)` (correct vs mk-codec), but the plan must ensure the Go `childNum uint32` is the BIP-32 raw value (e.g. `0x80000002` for `2'`), matching `bip380.go:102`'s use of the raw `bip32.Path` u32 — not the unhardened index. Spec-level concept is right; flag for the implementation plan.

**M-2. Citation drift.** §2.6/§4.2 cite `gui_test.go:39-96`; the actual alloc gate is `:50-98`. §2.1 cites `pipeline.rs:73,161-164` (correct) but the "smallest = 80 bytes" line is `:354`. Refresh line numbers when folding.

**M-3. §3.7 cites `chunk.rs:109-196` for reassembly; the function spans `:109-203`** (the cross-chunk-hash check is `:189-202`). Minor range trim.

**M-4. Display-paging hazard (T1 lesson) is asserted (§4.2 measure-and-advance) but not made a verifiable invariant.** §2 has 9 invariants; the "pages reach the xpub tail, gap-free" property is only in §6 GUI tests, not an invariant. Consider promoting it (the base58 xpub is ~111 chars and is the field that forces paging on 240×240) so the R0/execution review can check it as a first-class requirement. Not blocking.

---

## Verdict

**NOT GREEN — 1C / 2I**

The spec is structurally sound, the package/phase split is correct, the reader-lifecycle and package-dependency feasibility all check out, and the headline scoping (single-string unreachable → gather mandatory) is verified true. But it must not proceed to implementation while C-1 stands: the `chunk_index` value−1 claim is a false load-bearing protocol fact that would inject an off-by-one into the reassembler. I-1 (incomplete reject set + conflated BCH-vs-structural negative-test layers + reused-padding hazard) and I-2 (a mandated parity vector that doesn't exist, colliding with the Rust-sourced-only rule) are both correctness-affecting. Fold all three, refresh the MINOR citations, persist this review verbatim to `design/agent-reports/`, and re-dispatch.
