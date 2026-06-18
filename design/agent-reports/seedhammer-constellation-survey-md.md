<!--
Persisted verbatim. Constellation capability survey (2026-06-18), agent 2 of 5: md-codec /
descriptor-mnemonic. Synthesis: design/RECON_seedhammer_constellation_terminal.md. Agent's report
below as returned.
-->

# md-codec / descriptor-mnemonic → SeedHammer capability survey

All features below are **deterministic** transforms over **public** md1 data — no CSPRNG anywhere — so none hit SH's generative block. md1 is the one constellation format SH can safely take over NFC, so the natural axis is **NFC-scan → decode → display-for-verify → engrave**. Key files: `crates/md-codec/src/{decode,encode,chunk,validate,to_miniscript,derive,identity,tag,phrase}.rs`; `crates/md-cli/src/{cmd/*,format/text.rs,parse/*}`. SH already does md1 typed entry, engrave, BCH correction, NFC-scan→text/QR plate, and descriptor engrave.

| Feature | What SH could do on-device | Verdict | Rationale | Effort | Notes |
|---|---|---|---|---|---|
| **md1 decode (string → AST)** `decode_md1_string`/`decode_payload` | Parse a typed/NFC-scanned md1 into the descriptor AST | **GAP** | Deterministic gateway for everything below. SH currently treats scanned md1 as opaque bytes; it does NOT decode the bitstream into structure. | **M** | Public, NFC-safe. Port the bit reader + tag/tree/TLV readers. Must mirror `key_index_width = ⌈log₂(n)⌉` exactly or the stream desyncs. |
| **Human-readable descriptor render** `descriptor_to_template` | After decode, show the BIP-388 template (`wsh(multi(2,@0/<0;1>/*,...))`, `tr(...)`) for human verification before engraving | **GAP** *(highest value)* | Confirm policy shape/key-count/paths/multipath on-device before committing to steel. Pure string-building over the AST. | **M** | Renders all 36 tags. Display real-estate is the only constraint (scroll/paging for long policies). Template uses `@N` placeholders — no secret. |
| **Multi-chunk reassembly + cross-chunk integrity** `reassemble` | Accept N md1 chunks → verify version/chunk-set-id/count, indices `0..count-1` no gaps/dupes, concat, decode, re-derive chunk-set-id, confirm match | **GAP** | A chunked policy spans up to 64 strings; confirm a complete self-consistent set before engraving. The cross-chunk SHA-256[0..20] check is the integrity spine; SHA already on-device. | **M** | Builds on md1-decode. Independent of any secret. |
| **Structural validation** `validate_*` + root-tag allow-list | Reject malformed md1 (root tag ∉ {Sh,Wsh,Wpkh,Pkh,Tr}; placeholder coverage; multipath consistency; tap-leaf legality; xpub on-curve) | **GAP** (largely free with decode) | Runs inside `decode_payload`, so porting decode brings most along. xpub on-curve needs secp256k1 point-parse (SH has secp). | **S–M** | Bundled with decode. |
| **md1 BCH error-correction** (BCH(93,80,8), t=4) | Correct ≤4 subs per chunk | **SHIPPED** | SH already does it. | — | — |
| **Re-encode (AST → md1)** `encode_payload` | Re-serialize a decoded descriptor | **LOW-VALUE** | SH receives md1 already-formed and engraves it; no need to author md1 on-device. Verify-by-re-encode is covered by BCH + reassembly integrity. | M if ever | Encoder also has a known encode-accepts/decode-rejects k>n gap. |
| **Descriptor construction from template text** `parse_template` | Parse a hand-typed BIP-388 template | **OUT-OF-SCOPE** | Authoring, not backup; full miniscript-grammar parser; no keypad use case. | L | Belongs in host `md` CLI. |
| **Address derivation** `derive_address`/`to_miniscript` | Decode wallet-policy-mode md1 (xpubs in TLV) → `miniscript::Descriptor` → derive addresses | **OUT-OF-SCOPE** (verging GAP) | Pulls in the entire rust-miniscript + bitcoin descriptor stack — very heavy for TinyGo; address-checking is wallet/signer territory; md1 is usually engraved in *template* mode (no xpubs). | L | [App-layer survey corrects this: the fork ALREADY has a pubkey→address pipeline (`address.go`), so address display is a wiring task, not a miniscript port.] |
| **Descriptor-type coverage** (36 ops: wpkh/pkh/sh/wsh/tr; multi/sortedmulti/multi_a/sortedmulti_a; full miniscript fragments; tap-trees) | Decode/display/validate any shape | **GAP** (rolls into decode/display) | The generic AST walker handles the full tag space. Bound tap-tree recursion (BIP-341 max 128 nodes). | (part of decode) | All deterministic. |
| **Identity hashes & fingerprints** `Md1EncodingId`/`WalletDescriptorTemplateId`/`WalletPolicyId` + `fmt_policy_id_fingerprint` | Compute + display a 4-byte policy/template fingerprint as a verification anchor | **GAP** (nice-to-have) | Confirm two cards encode the same policy / match an expected value without reading the whole template. SHA on-device. | **S–M** | WalletPolicyId needs canonicalization (heavier); encoding-id/template-id fingerprints are lighter. |
| **PolicyId → 12-word BIP-39 phrase** `to_phrase` | Render the policy fingerprint as a 12-word anchor phrase | **LOW-VALUE** | Trivial but the design itself doubts the value (anchors the template, not the wallet; confusing for re-used templates). The 4-byte hex covers the need more honestly. | S | — |
| **Policy compiler** `md compile`/`--from-policy` | Compile a spending-policy → optimal miniscript | **OUT-OF-SCOPE** | Authoring/optimization, host-only; needs the full miniscript compiler. | L | — |
| **Display grouping / separator handling** | Strip whitespace/`-`/`,` on intake; re-group for display | **GAP** (trivial) | Both grouped & unbroken cards should re-ingest. | **S** | Separators aren't in the codex32 alphabet → unambiguous. |
| **Mixed-case rejection** (BIP-173) | Reject internally mixed-case md1 on intake | **GAP** (trivial) | BIP-173 conformance. Cross-chunk case heterogeneity is legal (QR workflow). | **S** | Bundled with decode intake. |

## Top SH candidates from md-codec (ranked)
1. **md1 decode → human-readable descriptor display** — M. *The* high-value gap; structural validation comes free inside `decode_payload`.
2. **Multi-chunk reassembly + cross-chunk integrity** — M. Builds on #1; SHA already present.
3. **Policy/template fingerprint display** — S–M.
4. **Intake hygiene (separator-strip + mixed-case)** — S.

**Not recommended:** address derivation (heavy — but see app-layer correction), policy compiler, template parsing, re-encode. BCH correction **shipped**.
