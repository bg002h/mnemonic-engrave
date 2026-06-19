# SPEC — T2c: on-device md1 decode→display (human-readable BIP-388 template)

> Cycle T2c of the "SeedHammer as air-gapped constellation terminal" roadmap.
> Recon: `design/cycle-prep-recon-T2c-md1-decode.md` (both protocol + fork facts independently verified vs `descriptor-mnemonic/crates/md-codec` @ 0.36.0 and fork `2fed9b6`).
> Base: fork `2fed9b6` (T2b merged). Fork-side only; no upstream PR.

## 1. Goal & scope

Let an operator **inspect** what an `md1` descriptor card decodes to — a **human-readable BIP-388 template summary** (script type, policy/threshold, key count, and per-placeholder `@N` origin fingerprint + derivation path + multipath) — on the air-gapped touchscreen, BEFORE/while engraving. md1 is PUBLIC (descriptor template, no secret), so inspection is offered unconditionally for any BCH-valid md1.

**SCOPE = SINGLE-STRING md1 only.** Template-only md1 is small (the 10-vector corpus is 9 single-string + 1 *forced*-chunked); a real template card fits one `md1…` string. **Chunked md1 is deferred** (see Out of scope) because its cross-chunk integrity check (`reassemble` → `compute_md1_encoding_id` → `derive_chunk_set_id`, `chunk.rs:294-389`, `identity.rs:39`) requires a full canonical **re-encoder** (`encode_payload`) — the encode side is out of a decode-only cycle, and porting it for a 20-bit integrity check on the rare chunked case would roughly double the package.

### In scope (T2c)
- A new `md` Go package: a MSB-first **bit reader**, the 5-bit single-payload header, in-band **chunked-flag detection** (bit 0 of symbol 0) that cleanly REFUSES a chunked md1 (`errChunkedUnsupported`, no mis-decode), and a recursive bit-packed **AST decode** of a single-string payload to an in-memory `Template` (n, origin paths, use-site multipath/wildcard, the operator tree, and the TLV section — fingerprints + parsed-but-unexpanded pubkeys), gated by the five post-decode validators (§2.12).
- A `codex32.MDDataSymbols` primitive (analogous to `MKDataSymbols`).
- A faithful **template renderer**: a human-readable summary for the **renderable subset** (§4.2); for any validated shape outside it, an explicit "complex/unsupported policy — cannot display safely" with key count + per-key origins (NEVER an approximation).
- GUI: a measure-and-advance **display** wired as an md1-only **"Inspect descriptor"** affordance in `mdmkFlow` (decodes the single scanned md1 string directly — NO multi-chunk gather).

### Out of scope (explicit)
- **Chunked (multi-part) md1** (deferred, new ledger item alongside T2c-b/#10): multi-chunk reassembly + the `encode_payload`-based cross-chunk csid integrity + the multi-string NFC gather. A scanned chunked md1 is detected and cleanly refused in T2c, not decoded. (Chunked md1 co-occurs with the large wallet-policy/xpub case, which is also deferred — they share the encoder need.)
- **Wallet-policy xpub-expansion** (T2c-b, #10): when a `Pubkeys` TLV is present, reconstructing per-`@N` xpubs, projecting onto `*bip380.Descriptor`, routing through `DescriptorScreen`, and receive-address verification. T2c **parses** the Pubkeys TLV (for bit-cursor correctness) but does NOT expand/display xpubs.
- **Full miniscript descriptor-string serialization** and **rust-miniscript-equivalent rendering**. T2c renders a faithful *summary*, not a re-serialized canonical descriptor string.
- **Engraving the decoded form.** The engrave model stays per-string-verbatim. Decode-display is read-only inspection.
- **BCH error correction.** The fork's `ValidMD` is a pure verifier; T2c decodes clean codewords only (a corrupt string fails the scan gate before reaching decode).

## 2. Invariants (R0 MUST verify each — Critical if violated)

1. **2.1 Wire-exact single-string decode vs md-codec 0.36.0.** The bit reader, the 5-bit single-payload header, the recursive AST (`read_node`, 36-tag table, Body variants, `MAX_DECODE_DEPTH=128`), origin/use-site paths (`depth(4)`+`hardened(1)`+LP4-ext varint), and the TLV section MUST match `descriptor-mnemonic/crates/md-codec` (`decode_md1_string`/`decode_payload`) byte-for-byte. Verified by corpus parity over the 9 single-string vectors (§6), NOT Go self-round-trip. (Chunked reassembly + the encoder-based csid integrity are out of scope — §1.)
1b. **2.1c Chunked md1 cleanly refused, never mis-decoded.** The in-band single-vs-chunked discriminator is **bit 0 of the first 5-bit symbol** (`chunk.rs:606`: single-payload version `4`=`0b00100` has LSB 0; a chunk header's version-then-chunked-flag makes the first symbol's LSB 1). `md.Decode` MUST check this first and return `errChunkedUnsupported` for a chunked fragment (the GUI shows "Multi-part descriptor — not yet supported"). It MUST NOT attempt to decode a chunk fragment as a single payload (which would mis-parse).
2. **2.2 `kiw` lockstep.** `key_index_width = ⌈log₂(n)⌉ = 32 - (n-1).leading_zeros()` MUST be computed exactly as md-codec and applied consistently to every `kiw`-bit key index. A drift desyncs the whole bitstream with NO post-BCH checksum to catch it — the highest-risk surface.
3. **2.3 Symbol-aligned bit count + two distinct padding budgets.** The payload bit length MUST be recovered as `5 × data_symbol_count`, NOT `len(bytes)·8` (getting this wrong breaks specific chunk counts — md-codec calls out N=3, N=8). **Two separate padding budgets, applied at their own layers, MUST NOT be cross-applied:** the codex32 symbol layer tolerates **≤4** trailing padding bits (`codex32.rs:111-112`); the TLV-decode rollback tolerates **≤7** trailing bits and rolls back a phantom partial-TLV (`tlv.rs:215-303`, threshold `:296`).
4. **2.4 Faithful-or-refuse display — distinct from decode-error.** A string is either: (a) a **decode error** — fails BCH, the chunked-flag check (§2.1c → `errChunkedUnsupported`), bit-cursor bounds, OR any of the §2.12 post-decode validators (e.g. a complex shape with elided origins → `MissingExplicitOrigin`) → the GUI shows "can't decode this card", no Template; or (b) a fully-decoded-AND-validated `Template`. For (b), the renderer renders a human-readable summary ONLY for shapes in §4.2 (`Renderable=true`); for a validated Template outside §4.2 (`Renderable=false`) it MUST show an explicit "complex policy — cannot display safely" (+ key count + per-key origins) and MUST NEVER show an approximated/partial/wrong policy. `Renderable=false` is reserved for *valid-but-complex* wires, NOT for decode failures.
4b. **2.5 Decode rejects every malformed input** — no panic, no partial Template. Reject set (md-codec `error.rs`): structural — `WireVersionMismatch` (version≠4), `TagOutOfRange` (reserved 6-bit tag / `0x3F` ext), `OperatorContextViolation` (non-canonical root tag — root ∉ {Sh,Wsh,Wpkh,Pkh,Tr}), `KGreaterThanN`, `PlaceholderIndexOutOfRange`, `TlvLengthExceedsRemaining`, `TlvOrderingViolation`, `EmptyTlvEntry`, `OverrideOrderViolation`, depth > `MAX_DECODE_DEPTH`, truncation/`BitStreamTruncated`, malformed padding (>7-bit trailing); **plus the §2.12 post-decode validator rejects** (`MissingExplicitOrigin`, `PlaceholderNotReferenced`, `PlaceholderFirstOccurrenceOutOfOrder`, `MultipathAltCountMismatch`, `ForbiddenTapTreeLeaf`, `NUMSSentinelConflict`, `InvalidXpubBytes`); **plus `errChunkedUnsupported`** (a chunked-flagged md1 — §2.1c). The bit reader MUST bounds-check every read. (Chunk-set rejects `ChunkSetIdMismatch`/`ChunkSetIncomplete` are deferred with the chunked path.)
4c. **2.12 Post-decode validators + canonical-origin dictionary (load-bearing for 2.1/2.5).** After the AST/TLV decode, md-codec runs five validators (`decode.rs:56-69`): `validate_placeholder_usage`, `validate_multipath_consistency`, `validate_tap_script_tree`, `validate_explicit_origin_required` (consulting the 5-shape canonical-origin table `canonical_origin.rs:45-79`: pkh→`m/44'/0'/0'`, wpkh→`m/84'/0'/0'`, tr-keyonly→`m/86'/0'/0'`, wsh-multi→`m/48'/0'/0'/2'`, sh-wsh-multi→`m/48'/0'/0'/1'`), `validate_xpub_bytes`. The Go port MUST replicate all five (a card md-codec rejects MUST NOT decode on-device to a displayable Template). A shape NOT in the canonical-origin table is only valid when it carries explicit per-`@N` origins (else `MissingExplicitOrigin`).
4d. **2.13 `kiw` / Tr-`is_nums` variable-width cursor (highest-fragility).** Beyond §2.2's `kiw` lockstep: `Tr` suppresses the `kiw`-bit key-index field entirely when `is_nums=1` (`tree.rs:271-276`) — the single most cursor-fragile branch after `kiw`. The decoder MUST read `is_nums(1)` then conditionally the `key_index(kiw)`. Cover with a constructed `tr(NUMS,…)` parity vector (not in the 10-vector corpus).
5. **2.6 Read-only display.** The decode-display screen contains NO engrave/NFC-write/NDEF/plate/mutation call — render + navigation returns only.
6. **2.7 No regression to the verbatim engrave path.** `validateMdmk`, the `mdmkText` engrave variants, and the mk1 "Inspect key" branch are behaviorally unchanged. The new md1 "Inspect descriptor" affordance is md1-only and additive. `TestMdmkFlowMD1NoInspect` is UPDATED (md1 now offers Inspect). The mk1 path (`TestMdmkFlowMK1ShowsInspect`) is untouched.
7. **2.8 0-alloc gate untouched.** No allocating per-frame work added to `StartScreen.Flow` or `DescriptorScreen.Confirm` (the only `TestAllocs`-gated paths). New md1 screens are not alloc-gated.
8. **2.9 No secret handling.** md1 is PUBLIC; NO `Unshared` gate, NO `wipeBytes`. Decode-display unconditional for BCH-valid md1.
9. **2.10 HRP discrimination by prefix.** Inspect-descriptor gated on the `md1` prefix; mk1 keeps "Inspect key"; non-md1/mk1 unaffected.
10. **2.11 Display paging reaches the tail, gap-free** (the T1 lesson): the summary (esp. multi-key origin lists) MUST page gap-free with no dropped tail (measure-and-advance).

## 3. Source facts (verified vs md-codec 0.36.0; full citations in the recon)

- **BCH**: `ValidMD` constants match `bch.rs` exactly (`MD_REGULAR_CONST 0x0815c07747a3392e7`, `POLYMOD_INIT 0x23181b3`); regular-only, 13-sym checksum; no correction.
- **String layer** (`header.rs`, `chunk.rs`): single-payload header = 5 bits (`bit4 divergent_paths`, `bits3..0 version=4`). Single-vs-chunked discriminator = **bit 0 of the first 5-bit symbol** (single LSB 0 / chunked LSB 1). Single-string is dominant (9/10 corpus). T2c decodes single-string and refuses chunked (§2.1c). 5↔byte MSB-first; payload bit count = `5 × data_symbol_count` (`codex32.rs:157`, `unwrap_string`); `decode_md1_string` → `decode_payload(bytes, bitcount)`. (Chunked 37-bit header + reassembly are deferred.)
- **Payload** (`bitstream.rs`, `decode.rs`, `tag.rs`, `tree.rs`, `origin_path.rs`, `use_site_path.rs`, `varint.rs`, `tlv.rs`): bit-packed recursive AST; decode order Header→PathDecl→UseSitePath→`kiw`→`read_node`→root-tag check→TLV→**5 validators**. 36-tag 6-bit table; `Body` **9-variant** (Children, Variable, MultiKeys, Tr, KeyArg, Hash256Body, Hash160Body, Timelock, Empty — `tree.rs:18-73`); origin paths explicit-only at the WIRE layer (`depth(4)`+`hardened(1)`+LP4-ext varint, max 15 comps); multipath 2..9 alts; TLV fingerprints (4B) + pubkeys (65B = chaincode‖compressed-pubkey, indexed by `@N`).
- **Post-decode validators** (`decode.rs:56-69`): `validate_placeholder_usage`, `validate_multipath_consistency`, `validate_tap_script_tree`, `validate_explicit_origin_required`, `validate_xpub_bytes`. The explicit-origin validator consults a **5-shape canonical-origin dictionary** at the VALIDATOR layer (`canonical_origin.rs:45-79`; this is distinct from mk1's wire-layer std-path table): pkh→`m/44'/0'/0'`, wpkh→`m/84'/0'/0'`, tr-keyonly→`m/86'/0'/0'`, wsh-multi→`m/48'/0'/0'/2'`, sh-wsh-multi→`m/48'/0'/0'/1'`. A shape outside this table with elided origins is rejected `MissingExplicitOrigin`.
- **Representation gap**: fork `bip380.Descriptor` models only `{singlesig, wsh(sortedmulti)}` — far narrower than md1. → T2c renders a faithful summary, not a bip380 projection (that's T2c-b for the xpub case).
- **Corpus**: Rust `MANIFEST` (10 vectors) at `md-codec/src/test_vectors.rs`; per-vector files `tests/vectors/<name>.{template,phrase.txt,bytes.hex,descriptor.json}`; all template-only, clean. No JSON/SHA pin.

## 4. Design

### 4.1 Phase A — `md` decode package (deterministic core)

`codex32` gains `MDDataSymbols(s string) ([]byte, error)` (gate on `ValidMD`, strip the 13-sym checksum, return 5-bit data symbols) — analogous to `MKDataSymbols`, generalizable.

New package `seedhammer.com/md` (deps: `codex32` + a self-contained MSB-first bit reader; NO bip380/btcec in T2c):
```go
type Template struct {
    N           int           // placeholder count (@0..@N-1), 1..=32
    Root        ScriptKind    // Wsh | Sh | Wpkh | Pkh | Tr
    Policy      PolicyKind    // SingleKey | Multi | SortedMulti | MultiA | SortedMultiA | Complex
    K, M        int           // threshold k-of-m for the multi families (else 0)
    Keys        []KeyOrigin   // per-@N: index, fingerprint (or ""), origin path, multipath/wildcard
    Renderable  bool          // false → the AST is outside §4.2; display refuses
}
type KeyOrigin struct {
    Index       int
    Fingerprint string        // 8 hex, or "" if no Fingerprints-TLV entry
    OriginPath  string        // "m/48'/0'/0'/2'" or "m"
    UseSite     string        // "<0;1>/*" etc.
}
func Decode(s string) (Template, error)   // single-string: chunked-flag check → AST decode → validate → summarize
```
Internals: `MDDataSymbols(s)` → 5-bit data symbols → **if `symbols[0]&1 == 1` return `errChunkedUnsupported`** (§2.1c) → repack symbols→bytes (MSB-first 5→8) with `bitcount = 5 × len(symbols)` → `decodePayload(bytes, bitcount)`: bit reader → Header (5 bits) → PathDecl (`divergent_paths` from header) → UseSitePath → compute `kiw` → `read_node` (root AST) → root-tag allow-list `{Sh,Wsh,Wpkh,Pkh,Tr}` → TLV section → **run all five §2.12 validators** → summarize into `Template` (set `Renderable` per §4.2). **`Decode` returns `(Template, error)`:** ANY md-codec reject — chunked-flag / BCH / bit-cursor bounds / structural / any validator (e.g. `MissingExplicitOrigin`) — returns a non-nil error and a zero `Template` (the GUI shows the appropriate message). A `Template` is returned ONLY when the wire fully decodes AND passes all five validators; `Renderable=false` is then reserved for such valid wires that fall outside §4.2 (it carries N + Keys/origins but makes no policy claim). No multi-string/gather signature (single-string scope).

### 4.2 The renderable subset (faithful-or-refuse)
`Renderable = true` ONLY for:
- `Wpkh(@k)`, `Pkh(@k)` — single-key.
- `Wsh(<multi-family>)`, `Sh(<multi-family>)`, `Sh(Wpkh(@k))`, `Sh(Wsh(<multi-family>))` — where `<multi-family> ∈ {Multi, SortedMulti, MultiA, SortedMultiA}(k, @keys...)` with KeyArg children only.
- `Tr(@k)` keyspend-only (`is_nums=false`, no script tree).
Everything else (nested miniscript: `and_*`/`or_*`/`thresh`-of-subpolicies, timelocks, hashlocks, taptree branches, NUMS-internal-key taproot, `Wsh(pk)` raw, etc.) → `Renderable = false`. The renderer distinguishes `Multi` (ordered) from `SortedMulti` (order-independent) — a policy-relevant difference it MUST surface.

**Canonical-origin interaction (R0-I3):** of the renderable shapes, only single-key (`pkh`/`wpkh`/`tr`-keyonly), `wsh(<multi>)`, and `sh(wsh(<multi>))` are in the canonical-origin table (§3) and so may arrive with elided origins (the canonical path is then implied). `Sh(<multi>)`, `Sh(SortedMulti)`, and `Sh(Wpkh(@k))` are NOT canonical → they only ever reach the renderer carrying **explicit per-`@N` origins** (else they're a `MissingExplicitOrigin` decode error, never a Template). For ALL renderable shapes the summary MUST display each key's actual decoded `OriginPath`/`Fingerprint` from the Template (never assume/claim a canonical path the wire didn't carry).

### 4.3 Phase B — GUI decode-display (NO gather)

Single-string scope ⇒ NO multi-chunk gather screen and NO `mk1Gatherer` generalization (mk1's gatherer is untouched). The md1 Inspect path decodes the single scanned string directly.

**Display** (`md1DisplayFlow`, measure-and-advance, NOT alloc-gated): for `Renderable` — a summary like `Type: P2WSH 2-of-3 multisig (sortedmulti)`, then per-key `@0  deadbeef  m/48'/0'/0'/2'  <0;1>/*`. For `!Renderable` — `Complex policy — cannot display safely` + `Keys: N` + the per-key origins. Read-only (§2.6); paging gap-free (§2.11).

**Wiring** (`gui/gui.go` `mdmkFlow`): add an `isMD := !isMK && hasMDPrefix(str)` branch — prepend "Inspect descriptor", `choice==0` → `card, err := md.Decode(str)`; on `errChunkedUnsupported` → `showError(... "Multi-part descriptor — not yet supported")`; on other error → `showError(... "Can't decode this descriptor")`; on success → `md1DisplayFlow(template)`. Generalize the existing `idx--` to "decrement if an Inspect entry was prepended" (covers both mk1 and md1). md1 no longer engrave-only; mk1 + md1-engrave paths otherwise unchanged.

## 5. File manifest (indicative; plan pins)
- **Create** `md/md.go` (bit reader, 5-bit header, chunked-flag refuse, AST decode, paths/varint/use-site, TLV, the 5 validators + canonical-origin table, Template summarize, renderable-subset classify), `md/md_test.go` (single-string corpus parity + negatives + chunked-refuse + renderable classification).
- **Create** `codex32/mddata.go` + test (`MDDataSymbols`, mirrors `MKDataSymbols`).
- **Create** `gui/md1_inspect.go` (`md1DisplayFlow`, `hasMDPrefix`; reuse the existing `chunkString`). NO gatherer changes.
- **Modify** `gui/gui.go` (`mdmkFlow` md1 "Inspect descriptor" affordance) + `gui/mk1_inspect_test.go` (update `TestMdmkFlowMD1NoInspect` → md1 now shows Inspect) + `gui/md1_inspect_test.go` (md1 display test; chunked-refuse + decode-error message paths; mk1 path regression unchanged).

## 6. TDD
- **Parity (load-bearing):** embed verbatim from md-codec 0.36.0 `tests/vectors/<name>.{phrase.txt,bytes.hex,descriptor.json}` — the **9 single-string** vectors: `wpkh_basic`, `pkh_basic`, `wsh_multi_2of2`, `wsh_multi_2of3`, `wsh_sortedmulti`, `tr_keyonly`, `sh_wsh_multi`, `wsh_divergent_paths`, `wsh_with_fingerprints`. Assert `Decode(s)` → exact `Template` (N, root, policy, k/m, per-key origins incl. fingerprints + divergent use-site, Renderable). NOTE: `wsh_with_fingerprints` and `wsh_divergent_paths` are **Renderable=true** `wsh(multi)` shapes — the fingerprints / per-key `<2;3>/*` are *displayed* (faithful), not a refusal trigger. **Provenance: md-codec-sourced only; never Go-derived; byte-strings copied from the live `tests/vectors/*` files, never the recon.**
- **Chunked-refuse:** the `wsh_multi_chunked` `phrase.txt` (its single `md1fz4aw…` chunk line) → `Decode` returns `errChunkedUnsupported` (no panic, no partial Template).
- **Negative** (assert rejection per category, NOT error-string equality — Go strings independent of Rust): structural — wire-version≠4, reserved tag, non-canonical root, K>N, placeholder-index-out-of-range, TLV-length-overflow, TLV-ordering, empty-TLV, depth>128, truncation, >7-bit trailing padding; **validator-layer (§2.12)** — `MissingExplicitOrigin` (a non-canonical shape, e.g. `sh(multi)`/`wsh(and_v(...))`, with elided origins), `PlaceholderNotReferenced`, `PlaceholderFirstOccurrenceOutOfOrder`, `MultipathAltCountMismatch`, `ForbiddenTapTreeLeaf`, `NUMSSentinelConflict`, `InvalidXpubBytes`. Each → error, no panic, no partial Template.
- **Renderable classification:** renderable vectors → friendly summary surfacing each key's decoded origin; a constructed/sourced **valid-but-complex** shape (e.g. an explicit-origin `wsh(and_v(...))` that PASSES all validators) → `Renderable=false`, refusal copy, no policy claim. Include an **explicit-origin `sh(multi)`** vector (R0-I3: renderable only with explicit origins) and a constructed **`tr(NUMS,…)`** vector (R0-M5/§2.13: `is_nums` variable-width cursor) — both sourced from md-codec round-trip, not the corpus.
- **GUI:** `md1DisplayFlow` pages to the tail gap-free; no engrave/NFC from inspect; md1 shows "Inspect descriptor", mk1 path unchanged (`TestMdmkFlowMK1ShowsInspect` still passes); chunked-md1 + decode-error → the right showError messages; `TestAllocs` passes.

## 7. Process
- **R0 gate (mandatory, this doc):** opus-architect to 0C/0I before any code. Fold → persist verbatim to `design/agent-reports/seedhammer-T2c-md1-spec-review-R*.md` → re-dispatch until GREEN.
- Then `IMPLEMENTATION_PLAN_seedhammer_T2c_md1_decode.md` → its own R0 (the plan reviewer SHOULD materialize + build/run, as for T2b; the bit-cursor is the risk surface — single-string corpus `phrase.txt`→`Template` parity is the proof). **All embedded test byte-strings/phrases MUST be copied verbatim from the live md-codec `tests/vectors/*.{bytes.hex,phrase.txt,descriptor.json}` files, NEVER from the recon** (R0-M3: the recon transposed the `wsh_multi_2of3`/`wsh_multi_chunked` payloads). Phase A (`md` decode pkg + `MDDataSymbols`, parity-GREEN over the 9 vectors + chunked-refuse + negatives) before Phase B (GUI display + wiring). The bit reader, `read_node`, paths, varint, TLV, and the 5 validators are a faithful Go port of the cited md-codec source — the implementer ports each from `descriptor-mnemonic/crates/md-codec/src/`.
- Single-implementer TDD in a worktree off `2fed9b6`. Commits signed (`-S`) + DCO (`-s`), author "Brian Goss <goss.brian@gmail.com>", trailer `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. Explicit-path staging.
- Mandatory whole-diff adversarial execution review (fuzz the bit reader/decoder for panics, per T2b) → merge no-ff → push `bg002h`.
