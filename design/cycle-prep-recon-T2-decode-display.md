# cycle-prep recon — 2026-06-18 — T2 (decode→display→verify for md1/mk1/ms1)

**Base:** fork `main` `68e6ead`. **Recon agent:** aafb79c2a3b013cf5 (5 parallel sub-agents + ground-truth verification). Synthesis of the decode-porting surface + sub-scoping for roadmap tier T2 (`design/RECON_seedhammer_constellation_terminal.md`).

## Headline finding (corrects the survey's "S–M each")

The three formats differ wildly in port cost, and **all three payload decoders are from-scratch Go ports**: the fork's `ValidMD`/`ValidMK` (`codex32/mdmk.go:124,136`) are **pure BCH checksum verifiers — they decode no payload**, and the fork engraves md1/mk1 **verbatim as opaque strings** (`mdmkFlow`, `gui/gui.go:1930`). What the fork gives for free is the **codex32 BCH/string layer** (parity-tested) + the **English bip39 wordlist + words API** + the crypto libs — NOT the payload decode. (Guard against the false-consensus that "we need a Go codex32/BCH port" — that already exists in-tree.)

## Per-format

### ms1 → entropy / BIP-39 words + `mnem` byte + inspect — **XS–S** (do FIRST = T2a)
- **Rust:** `ms-codec/src/{decode,payload,inspect,consts}.rs`. Decode is a **byte-slice** (not a bitstream): codex32 BCH parse → `data()` → strip prefix byte (`0x00`=entr / `0x02`=mnem) → for mnem the next byte is the language index (0–9, `consts.rs:47-58`) → rest is entropy (16/20/24/28/32 B).
- **Fork already has (large reuse):** codex32 BCH + `ParsePrefix`/`Fields`; `codex32.String.Seed()`/`.Split()` (payload bytes + id/threshold/idx); `confirmCodex32Flow` (`gui/codex32_polish.go:83`) **already shows the inspect line** (Unshared-secret vs Share-X, id, char count, even Recover) — so "share k-of-N" inspect is substantially built; `bip39` (English) entropy↔words API; `SeedScreen.Draw` (`gui/gui.go:2221`) already renders BIP-39 words on-screen.
- **PORT delta (~150–300 LOC):** strip the prefix byte + read entropy from `Seed()`; read the **`mnem` language byte** + the 10-name table; wire entropy→`bip39.New()`→words into a display screen on the ms1 branch (today ms1 confirms then engraves verbatim).
- **KEY DESIGN DECISION (the non-English footprint):** the fork's bip39 is **English-only** (2048-word `wordlist.txt`; no other language files). For `mnem` language ≥1 the device **cannot render words**. **v1 decision (footprint-conscious, still safe): show the entropy hex + the language NAME + a "non-English — words not shown on this device" note; do NOT ship extra wordlists yet.** The `mnem` byte is the format's whole point (kills silent-wrong-wallet), so it MUST be read + surfaced even when words can't be. Shipping N more wordlists (~20–40 KB each, RP2350 flash-bounded) is a deferred enhancement.
- **Secrecy:** SECRET — display-only, never NFC; reuse `SeedScreen`'s sensitive-word treatment; explicitly scrub entropy buffers (Go GC won't).
- **GUI hook:** the `codex32.String` branch of `inputCodex32Flow`→`engraveCodex32` (`gui/gui.go:1874`); insert a words/inspect display between `confirmCodex32Flow` and engrave.

### mk1 → xpub + origin-fp + path + policy-id stubs — **S–M** (T2b)
- **Rust:** `mk-codec/src/{string_layer/*,bytecode/*,key_card}.rs`; decode-core ≈ 450 LOC, a **sequential byte cursor** (no bitstream/tree). Pipeline: BCH → 5-bit header (single vs chunked) → optional chunk reassembly (SHA-256[0:4] cross-chunk check) → `decode_bytecode`: 1-byte header, stub count, N×4-byte policy_id_stubs, optional 4-byte fingerprint, origin_path (1-byte standard-path table OR `0xFE` LEB128), 73-byte compact xpub.
- **The one subtlety — compact-xpub reconstruction** (`bytecode/xpub_compact.rs:71`): 73 bytes = `[version 4][parent_fp 4][chaincode 32][pubkey 33]`; **depth + child_number are NOT on the wire** — reconstruct from `origin_path` (`depth = component count`; `child = last component, or Normal{0}` if empty). Needs one secp256k1 point-parse + base58check assembly.
- **Fork already has (all crypto in-tree):** `btcutil/v2/hdkeychain` (xpub `.String()` = base58check), `btcec/v2` + `decred secp256k1/v4` (point parse), `chaincfg/v2` (xpub/tpub versions), `bip32.Fingerprint`/`Path.String()`. Reconstruction→display = byte-assembly + one `hdkeychain` `.String()`.
- **PORT delta (~450–700 LOC):** the byte-cursor bytecode decoder + the standard-path table + LEB128 path + compact-xpub layout + depth/child reconstruction + multi-chunk reassembly.
- **Landmines:** don't trust on-wire depth/child (zeroed) — reconstruct from path; empty-path→`Normal{0}`; port the 1-byte standard-path table (44/49/84/86/48/87 + testnet) exactly; reject reserved header bits; SHA-256 truncated to 4 bytes.
- **Secrecy:** PUBLIC — NFC-OK. **Zero HW-blocked items.**
- **GUI hook:** the `mdmkText` branch (`mdmkFlow`); insert a decoded-field display before the engrave ChoiceScreen (mirror `DescriptorScreen.Confirm`).

### md1 → BIP-388 descriptor template — **L (the heavy one)** (T2c, its own full cycle)
- **Rust:** `md-codec/src/{decode,bitstream,tree,tlv,tag,...}.rs` (decode ~4.1K LOC incl. tests) + the renderer `md-cli/src/format/text.rs` (`descriptor_to_template`, ~270 effective LOC). Pipeline: `BitReader` → key-index-width header → recursive `read_node` (6-bit tag tree over **36 operators**, `tag.rs`) → `TlvSection::read` → a `Descriptor` AST → template render (`wsh(multi(2,@0/<0;1>/*,...))`).
- **CRUCIAL: the decode + template render is entirely `miniscript`-crate-FREE** — only `to_miniscript.rs`/`derive.rs` import miniscript, and those are for address derivation (T1/T3, out of T2 scope). The renderer walks the AST directly → policy shape + `@i` placeholders + key count + derivation paths. **No miniscript engine port needed for display.**
- **Fork already has:** the md1 codex32 BCH layer (`ValidMD`); `bip380.Descriptor.String()` + `nonstandard.OutputDescriptor` — but those consume/produce **text** descriptors, useless for the md1 **binary bitstream**. The bitstream→AST gap is exactly what's missing.
- **PORT delta (~2,000–3,000 LOC new Go):** the entire `BitReader`, the recursive 36-tag `read_node`, the TLV reader, path/use-site decoders, and the `descriptor_to_template` walker. Comparable in weight to the original m\*1 BCH decoder cycle.
- **Landmines:** ALL 36 tags must be handled (no partial coverage); the bit-reader/tree/TLV ordering is spec-tight (regenerate Go golden vectors from the Rust `text.rs:533-710` tests); multipath `@i/<0;1>/*` + use-site-path overrides; tap-tree recursion + `Tr` NUMS internal-key case; **defer md1 multi-chunk reassembly to T5**.
- **Secrecy:** PUBLIC — NFC-OK. **GUI hook:** the `mdmkText` branch; scrollable template display (reuse the T1 `descriptorAddressFlow` list pattern, `gui/address_polish.go`).

## Sub-scoping decision (LOCKED)

**Three SEPARATE sub-cycles, ms1 → mk1 → md1** (NOT combined — they share only the in-tree codex32 BCH layer; payload decoders share no code [byte-slice vs byte-cursor vs bitstream] and have different secrecy models):
- **T2a = ms1 decode-display** (XS–S) — first, cheap high-value win; the only real decision is the non-English `mnem` handling (decided above: name+warning, no extra wordlists yet).
- **T2b = mk1 decode-display** (S–M) — self-contained byte-cursor port; spec the compact-xpub reconstruction carefully.
- **T2c = md1 decode-display** (L) — its own full heavy cycle with its own R0/exec-review; defer md1 multi-chunk to T5.

## Cross-cutting GUI/alloc

- Hook all three between input/confirm and engrave in the `inputCodex32Flow`→`engraveObjectFlow` dispatch (ms1 via `codex32.String`; md1/mk1 via `mdmkText`).
- Reuse the just-merged **T1 `descriptorAddressFlow`** scrollable display-list pattern (`gui/address_polish.go`).
- **0-alloc gate** (`TestAllocs`) covers only `StartScreen.Flow` + `DescriptorScreen.Confirm`; a new decode-display screen isn't auto-gated, but follow the fixed-slice/no-append-chain nav discipline as a courtesy.
- **Secret display:** ms1 words use the seed-word treatment + never NFC/QR; md1/mk1 are public.
