<!--
Persisted verbatim. Constellation capability survey (2026-06-18), agent 5 of 5: application layer
(shibboleth-wallet / mnemonic-gui / mnemonic-engrave). Synthesis:
design/RECON_seedhammer_constellation_terminal.md. KEY FINDING: the fork firmware ALREADY has the
full pubkey→address pipeline (address/address.go: Receive/Change/Supported/addressAt/derivePubKey),
tested but NOT imported by gui/cmd — so on-device address derivation is a UI-WIRING task, not a
crypto port. (Independently confirmed by the controller post-survey: gui/ + cmd/ do not import
seedhammer.com/address.) Agent's report below as returned.
-->

## Application-layer features vs. SeedHammer on-device mirror

| App-layer feature (source) | What SH could do on-device | Verdict | Rationale | Effort | Feasibility / security notes |
|---|---|---|---|---|---|
| **Receive/change address derivation** (`mnemonic addresses`, `md/mk address`; shibboleth-wallet) | Derive + display receive/change addresses from an engraved/scanned mk1 (xpub) or md1 (descriptor) — verify funds destination without a separate computer | **GAP** | Real air-gap pain point. The single highest-value mirror. | **S–M** | Crypto fully present in `third_party/seedhammer/address/address.go` (`Receive`/`Change`/`addressAt`, secp256k1 CKDpub + base58check/bech32/bech32m/P2TR + sortedmulti) — **tested but NOT imported by `cmd/controller`/`gui`**: a **UI-wiring task, not crypto**. Deterministic, no RNG. md1/mk1 PUBLIC → safe to display. |
| **Receive-address verification** ("is this address mine?" — shibboleth-wallet's verification pillar) | Type/scan an address, gap-limit scan a derived range, confirm match + show index/chain | **GAP** | Defends against malware-swapped receive addresses; canonical air-gap use. Builds on the address primitive. | **M** | Same `address` pkg + a bounded index loop. No camera (typed/NFC address entry, acceptable for an address string). |
| **Descriptor / xpub VERIFICATION display** (`md/mk decode`/`inspect`) | Decode a scanned/engraved md1→BIP-388 template and mk1→xpub+origin+fingerprint, shown for human confirmation | **GAP** (partial SHIPPED) | SH engraves descriptors + scans public md1/mk1; the gap is a structured decode-and-display screen ("what does this script do?"). | **S–M** | `bip380` parse + `nonstandard` already in the firmware import graph. Pure parse/format, deterministic, public. |
| **On-device bundle / multi-plate planning** (`me bundle` host tool) | Guided multi-plate sequence: walk the user plate-by-plate over a chunked md1/mk1 set, tracking which chunks done + the ms1-type-on-device reminder | **GAP** | `me bundle` is host-only; on-device a user scanning a multi-chunk set has no guided sequencing/completeness check. | **M** | Chunk metadata lives INSIDE the BCH-covered string. Reassembly/integrity is deterministic. Confirm the md/mk Go codecs expose reassembly. |
| **Plate preview before engrave** (`me bundle --preview` SVG/PNG) | Show a fit/layout preview on the touchscreen before cutting | **GAP** (low-ish) | The fit/mode logic (`validateMdmk`/`engraveBest`) IS the device's own curve math; a preview screen surfaces it pre-cut. | **S–M** | Same `backup.EngraveText`/`bspline` math on-device. Device already validates fit, so value is modest. |
| **Multisig coordination / cosigner assembly** (`restore --cosigner`, shibboleth-wallet) | Assemble a multisig descriptor from multiple scanned mk1 cosigner cards | **OUT-OF-SCOPE** (partial GAP) | Beyond backup/engrave; coordinator role. (Displaying an assembled sortedmulti's address IS in-scope.) | L | `address` pkg supports sortedmulti, but full coordination = wallet territory. |
| **Format conversion** (`mnemonic convert`, `*/encode/decode`) | On-device conversion between seed formats | **SHIPPED / partial** | SH already does BIP-39/codex32/SLIP-39/Seed-XOR/md1/mk1 entry+recovery+engrave. Net-new conversions LOW-VALUE for a backup device. | — | Codecs present; deterministic. |
| **Final-word / checksum completion** (`mnemonic final-word`) | Compute valid Nth BIP-39 word(s) | **SHIPPED** | SH does BIP-39 entry with correction. | — | — |
| **BCH error-correction / repair** | Correct mistyped ms1/mk1/md1 | **SHIPPED** | SH does it. | — | — |
| **Seed-XOR / SLIP-39 / codex32 split & combine** | Split / recombine | **SHIPPED (codex32, Seed-XOR combine) / HW-or-GAP (SLIP-39 split)** | codex32 recombination present. **SLIP-39 is a word-list stub — no Shamir/GF(256) math.** Generative split is HW-BLOCKED. | — | Deterministic recombine fine; split blocked. |
| **PSBT construction / signing** (planned shibboleth-wallet) | Sign/build txs | **OUT-OF-SCOPE** | Backup tool; no private-key export, not a signer. | — | **No PSBT anywhere in firmware** (0 hits). No miniscript engine. |
| **BIP-85 child derivation** (`bip85.go` present) | Derive child seeds | **OUT-OF-SCOPE / LOW-VALUE** | Exists in firmware but generative/derivation outside backup-engrave mission. | — | Deterministic (HMAC-SHA512) but off-mission. |
| **Nostr / Silent-Payments / message-verify** | nsec/npub, sp1, verify sigs | **OUT-OF-SCOPE / LOW-VALUE** | Not backup/engrave. | — | Off-mission. |
| **Wallet export formats** (`export-wallet` 11 formats; `import-wallet`) | Emit/parse vendor wallet files | **OUT-OF-SCOPE** | Host interop / file-transport; SH's transport is engrave + NFC of public cards. | — | — |
| **Passphrase brute-search / xpub-path search** (`xpub-search-*`) | Search a path/account/passphrase for a target xpub | **LOW-VALUE / OUT-OF-SCOPE** | Compute-heavy brute search ill-suited to the MCU; off-mission. | — | — |

## Top SH candidates from the application layer (ranked)
1. **On-device receive/change address derivation + display** (GAP, S–M). Clearest + cheapest air-gap win: the full secp256k1→address pipeline already exists + tested in `address/address.go`, simply NOT wired into `cmd/controller`/`gui`. Derive from an engraved/scanned mk1/md1 and show the address. Deterministic, public, no RNG.
2. **On-device receive-address verification** (GAP, M). "Is this address mine?" — gap-limit scan + confirm a typed/scanned address. Reuses #1; the shibboleth-wallet verification pillar, delivered air-gapped.
3. **On-device descriptor/xpub verification display** (GAP, S–M). Decode md1→template, mk1→xpub+origin+fp into a "what this is" screen. Parsers already in the firmware import graph.
4. **On-device bundle / multi-plate guided sequencing** (GAP, M). Device-side `me bundle`: prove completeness via in-band chunk metadata + cross-chunk hash, walk the plate sequence.
5. **On-device plate preview before engrave** (GAP, S–M; modest). Surface the device's own fit math as a pre-cut preview.

Boundary findings: **PSBT signing + miniscript evaluation firmly OUT-OF-SCOPE** (neither in firmware — 0 PSBT hits, only fixed script templates). **SLIP-39 secret-sharing is a stub** (word-list only) so a split/combine mirror needs porting; *generative* split is HW-BLOCKED. Caveat: `shibboleth-wallet` is a **pre-implementation planning stub** (two design docs, no code) — its "features" are stated intent (miniscript verification UX, BIP-388, PSBT-via-Coldcard), treated as requirement signals, not running consumers.
