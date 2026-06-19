<!--
Persisted verbatim. opus-architect focused R0 GATE review of the SPEC AMENDMENT that rescoped T2c to
SINGLE-STRING md1 (commit 3dd9c16), after a plan-time defect: the chunked path's csid integrity
(compute_md1_encoding_id → encode_payload, identity.rs:40 → chunk.rs:379) needs a full canonical
re-encoder (out of a decode-only cycle). Reviewer agentId a8253aeed1eec3668. Verdict: GREEN 0C/0I. The
reviewer empirically verified (built a throwaway md-codec harness, since removed): the encoder
dependency is real; single-string decode_md1_string/decode_payload is encoder-free; the chunked-flag
discriminator symbols[0]&1 routes a real chunk line to errChunkedUnsupported with NO mis-decode; the
single-string decode core (order/kiw/validators/canonical-origin/9-variant Body) is unchanged + matches
source; the 9 single-string parity vectors + the wsh_multi_chunked refuse-vector exist as described;
wsh_with_fingerprints/wsh_divergent_paths are Renderable=true wsh(multi) (origins displayed, not
refused); no dangling reassembly/gather/ParseHeader/multi-string refs; §2.5 coherent (dropped
KeyCountOutOfRange is decode-unreachable). 2 non-blocking MINORs: M1 (residual "reassembly integrity"
phrase in §2.4 — FIXED in the GREEN commit) and M2 (cosmetic non-monotonic §2.x label order, pre-existing
R1-M8, left as-is). Disposition: GREEN — cleared to implementation-plan authoring. Text below verbatim
(HTML entities un-escaped: &lt;→<, &gt;→>, &amp;→&).
-->

# R0 GATE REVIEW — SPEC_seedhammer_T2c_md1_decode.md (rescoped, commit `3dd9c16`)

Reviewed against authoritative source `descriptor-mnemonic/crates/md-codec` **@ 0.36.0** (`Cargo.toml` version confirmed `0.36.0`, tree at `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec`) and fork `seedhammer` HEAD **`2fed9b6`** (matches spec base). I read the full rescoped spec, both prior reviews (R0 0C/3I, R1 GREEN), and the rescope diff. I empirically verified the chunked-flag routing and single-string decode by building a throwaway harness against md-codec (decoding all 9 single-string vectors + the chunk line), since removed.

## Verification Results

**1. Encoder dependency is real (rescope rationale) — CONFIRMED.**
- `compute_md1_encoding_id` calls `encode_payload(d)?` at `identity.rs:40` (inside the `:39-45` fn), then SHA-256.
- `reassemble` (`chunk.rs:305-389`) imports `compute_md1_encoding_id` (`:308`), decodes via `decode_payload(&full_bytes, …)` (`:376`), then runs the cross-chunk integrity check `compute_md1_encoding_id(&descriptor)?` → `derive_chunk_set_id` (`:379-380`), comparing to `expected_csid`.
- Single-string `decode_md1_string` (`decode.rs:79-82`) → `decode_payload` (`:15-72`) uses ONLY `BitReader`, `Header::read`, `PathDecl::read`, `UseSitePath::read`, `read_node`, `TlvSection::read`, and the five validators — **no `encode`/`identity` import**. Single-string decode is achievable without the encoder; chunked is not. **Rescope rationale is sound.**

**2. Chunked-flag discriminator (§2.1c / §3 / §4.1) — CONFIRMED (empirically).**
- Single header `write` (`header.rs:30-33`): `(divergent_paths<<4)|(version&0xF)` as 5 bits MSB-first; version 4 = `0b0100` → 5-bit symbol `0bD0100`, LSB **0**.
- Chunk header `write` (`chunk.rs:51-55`): version(4)=`0b0100` then `chunked=1`(1) → first 5 bits `0b01001`, LSB **1**. Layout documented at `chunk.rs:3-6`.
- Dispatch in source: `chunk.rs:606` `symbols.first().map(|s| s & 0x01)`; `==0` → `decode_md1_string` (`:613`), `==1` → `reassemble` (`:616-621`). Matches `symbols[0]&1`.
- **Routing chain verified end-to-end:** a chunk fragment is independently `wrap_payload`'d with its own per-chunk BCH (`chunk.rs:286`), so it passes `ValidMD` → `MDDataSymbols` returns the chunk's data symbols, `symbols[0]` being the chunk-header's first 5-bit symbol. Empirically: the `md1fz4aw…` chunk line → `unwrap_string` ok (BCH valid), `sym0 = 0b01001`, `sym0&1 = 1`. So the spec's `symbols[0]&1 == 1 → errChunkedUnsupported` **fires correctly**. (As a secondary safety net, `decode_md1_string` on that line returns `WireVersionMismatch{got:9}`, not a silent mis-decode — but the spec's explicit pre-check is the cleaner, intended route and gives the right UX message.) Discriminator is CORRECT; no mis-routing.

**3. Single-string decode content unchanged — CONFIRMED.**
- Decode order `decode.rs:18-69` = Header→PathDecl→UseSitePath→kiw→read_node→root-tag allow-list `{Sh,Wsh,Wpkh,Pkh,Tr}` (`:36-44`)→TLV→5 validators (`:56-69`) — matches §4.1.
- kiw `decode.rs:26` `(32 - (n).saturating_sub(1).leading_zeros())` — matches §2.2.
- Canonical-origin table `canonical_origin.rs:45-79`: pkh→`44'/0'/0'` (:48), wpkh→`84'/0'/0'` (:50), tr-keyonly→`86'/0'/0'` (:52-54), wsh-multi→`48'/0'/0'/2'` (:58-62), sh-wsh-multi→`48'/0'/0'/1'` (:65-73), else `None` — matches §2.12/§3 byte-for-byte.
- `Body` 9-variant (`tree.rs:18-73`) — matches §3. Empirically decoded all 9 single-string vectors OK with correct root/body. The rescope edits (diff at `3dd9c16`) are surgical scope-narrowing prose; the decode core spec text was not damaged.

**4. Parity set (§6) — CONFIRMED.**
- All 9 named vectors exist in `tests/vectors/` as single `md1…` lines (1 line each). `wsh_multi_chunked.phrase.txt` is the chunked one: a 2-line file (line 1 = comment `chunk-set-id: 0x157ae`; line 2 = the chunk string `md1fz4awqqpqsgqpsgvyyxqql8saf74dwdyqv`). The spec §6 references "its single `md1fz4aw…` chunk line" — correctly aware of the comment line. `MANIFEST` (`test_vectors.rs:42-55`) confirms `wsh_multi_chunked` is the only `force_chunked: true` (k=3), distinct from `wsh_multi_2of3` (k=2).
- **`wsh_divergent_paths`**: template `wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))`, decodes to `root=Wsh body=Multi` → genuinely `wsh(multi(...))`, Renderable=true. The divergent `<2;3>/*` is a per-key use-site, not a shape change.
- **`wsh_with_fingerprints`**: template `wsh(multi(2,…))` + fingerprints TLV bytes, decodes to `root=Wsh body=Multi` → genuinely `wsh(multi(...))`, Renderable=true. Fingerprints-TLV does not change the renderable tree.
- Both are faithfully **displayed** (origins/fingerprints surfaced), not refusal triggers. The spec's prior recon misread is corrected.

## Internal-consistency / completeness

- **No dangling chunked references.** Full-spec grep: every `reassembl`/`gather`/`csid`/`chunk-set`/`37-bit` mention is confined to the scope/deferral statements (§1), the rationale citation, or the §2.1c/§2.5 refuse-path explanation. `ParseHeader` — **gone**. `Decode([]string)` — **gone**; §4.1 is `func Decode(s string) (Template, error)`. §4.3 has NO gather and explicitly "NO `mk1Gatherer` generalization (mk1's gatherer is untouched)". §5 manifest: "NO gatherer changes". Clean.
- **§2.5 reject set coherent after rescope.** Chunk-set rejects `ChunkSetIdMismatch`/`ChunkSetIncomplete` correctly removed (noted deferred); `errChunkedUnsupported` added. The R1-flagged M6 (`InvalidPresenceByte`) and M7 (`UnexpectedEnd`→`BitStreamTruncated`) are both fixed in this commit. All 18 named error variants verified present in `error.rs`. The rescope also dropped `KeyCountOutOfRange` (and "n=0/>32" from §6) — this is **correct, not a regression**: on the decode path `PathDecl::read` (`origin_path.rs:135`) computes `n = read_bits(5) + 1` ∈ 1..=32 by construction, so `KeyCountOutOfRange` is structurally unreachable when decoding (it is an encode-side guard, `origin_path.rs:111`). Removing it tightens the spec.
- **Single-string T2c is a coherent, complete, standalone, testable cycle.** It delivers the roadmap "human-readable BIP-388 template" goal for the 9/10 corpus that is single-string (the realistic template-card case), with full validator fidelity. The chunked-refuse gives clean UX (`errChunkedUnsupported` → "Multi-part descriptor — not yet supported"), no crash, no mis-decode.
- **Invariants hang together.** Numbering is 2.1, 2.1c, 2.2…2.13 (non-monotonic label order 2.5→2.12→2.13→2.6, pre-existing R1-M8, all cross-refs resolve). No invariant is vacuous or contradictory after the edits.
- **Deferring chunked to #10 is the right call.** A "cheaper" path that keeps chunked by skipping the csid integrity check is unsound: it would still require porting the 37-bit chunk header, multi-string gather, and sort/gap reassembly, AND skipping `compute_md1_encoding_id` drops md-codec's only cross-chunk binding — for a genuine multi-chunk set that risks decoding a mismatched/corrupted set to a wrong template, violating the faithfulness invariant §2.4. The single forced-chunked corpus vector could be decoded without the check, but generalizing that to real multi-chunk cards cannot. Defer-with-the-encoder is correct.

## Findings

### CRITICAL
*(none)*

### IMPORTANT
*(none)*

### MINOR
- **M1 — Residual "reassembly integrity" in §2.4.** Line 32 still lists a decode error as one that "fails BCH, bit-cursor bounds, **reassembly integrity**, OR any of the §2.12 validators." In single-string scope there is no reassembly, so this category is dead. Not contradictory (it's a superset enumeration), but a leftover that could momentarily suggest a reassembly-integrity check is in scope. Fix: drop "reassembly integrity" from the §2.4 (a) list (or qualify "—out of scope, §1"). Non-blocking.
- **M2 (cosmetic, pre-existing R1-M8) — non-monotonic §2.x labels** (2.1, 2.1c, 2.2…2.5, 2.12, 2.13, 2.6…2.11). All labels unique, all cross-refs resolve. Harmless; left as-is by R1. Could renumber opportunistically.

## Verdict

**GREEN — 0 Critical / 0 Important**

The rescope is correct, complete, internally consistent, and protocol-faithful. The encoder dependency justifying the deferral is real (`identity.rs:40` → `chunk.rs:379`), and single-string decode is genuinely encoder-free (`decode.rs:79-82`→`:15-72`). The chunked-flag discriminator (`symbols[0]&1`) is verified correct and empirically routes a real chunk line to refuse — no mis-decode. The single-string decode core (decode order, kiw, validators, canonical-origin table, 9-variant Body) is unchanged and matches source. The 9 single-string parity vectors and the `wsh_multi_chunked` refuse-vector exist as described; `wsh_with_fingerprints`/`wsh_divergent_paths` are confirmed Renderable=true `wsh(multi)`. No dangling reassembly/gather/`ParseHeader`/multi-string references survive; §2.5 is coherent (and tightened — the dropped `KeyCountOutOfRange` is decode-unreachable). Two MINORs (residual "reassembly integrity" phrase in §2.4; cosmetic label order) are non-blocking and may be fixed opportunistically in the implementation plan. Cleared for implementation-plan authoring.
