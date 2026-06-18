<!--
Persisted verbatim. Constellation capability survey (2026-06-18), agent 3 of 5: mk-codec /
mnemonic-key. Synthesis: design/RECON_seedhammer_constellation_terminal.md. Agent's report below.
-->

## mk-codec / mnemonic-key feature inventory → SeedHammer on-device verdicts

**mk1 is xpub-only and PUBLIC.** A `KeyCard` carries: `policy_id_stubs: Vec<[u8;4]>`, `origin_fingerprint: Option<Fingerprint>`, `origin_path: DerivationPath`, `xpub: bitcoin::bip32::Xpub`. Wire form = 73-byte compact xpub. **No xprv, no seed, no BIP-39, no RNG over key material** (verified). Only RNG = a 20-bit `chunk_set_id` reassembly tag. Matches SH's "md1/mk1 PUBLIC → NFC OK; never exports private keys" spine.

| Feature | What SH could do on-device | Verdict | Rationale | Effort | Notes |
|---|---|---|---|---|---|
| **mk1 ENCODE** (KeyCard→Vec<String>, auto regular/long + chunk) | Build an mk1 from fields, engrave/NFC-write | **SHIPPED** (typed entry) / **partial GAP** (field-assembly) | Typed entry + engrave + NFC-write shipped. Assembling from primitive fields the device computed is not — only valuable paired with xpub-derivation. | M | `encode_with_chunk_set_id` makes it deterministic; single-string mk1 has no chunk_set_id → NOT HW-blocked. |
| **mk1 DECODE** (Vec<String>→KeyCard; reassembly+BCH+cross-chunk hash) | Decode typed/NFC mk1 → **display xpub + origin fingerprint + derivation path + policy-id stubs** for pre-engrave verification | **GAP** (top) | Highest-value deterministic gap. Verify *what's inside* before committing to steel, mirroring SH's BIP-39 fingerprint-confirm UX. Reuses shipped BCH. | M | Deterministic, public. Needs bip32 xpub serialization (SH has bip32). UX: a decoded-card review screen. |
| **xpub derivation FROM an entered BIP-39 seed** (NOT in mk-codec — lives upstream) | From a hand-typed seed, derive the account xpub at a user-selected standard path (m/84'/0'/0' etc.) to engrave as mk1 | **GAP** (high value, security-sensitive) | mk-codec is xpub-in only, so not "shipped elsewhere." SH already derives a master fingerprint via bip32 from a seed → account-xpub derivation is within reach, fully deterministic (NOT TRNG-blocked). The natural producer for the encode path. | L | Touches seed/xprv internally but emits ONLY the public xpub (no xprv to NFC/engrave). UX: path-selection picker (44/49/84/86/48/87 + account). |
| **BCH error-correction** (`bch_correct_regular/_long`, t=4) | Auto-repair ≤4 mistyped chars/chunk | **SHIPPED** | SH does it. | — | Substitution-only (indel rejected). |
| **BCH checksum VERIFY** | Validate a typed/scanned mk1 | **SHIPPED** | t=0 case of the shipped code. | — | — |
| **Cross-chunk integrity hash** (SHA-256(bytecode)[0..4]) | Detect missing/reordered/wrong chunks | **GAP** (rides with multi-chunk decode) | Free with decode (SHA on-device); required for correctness; no standalone value. | S | — |
| **Chunking / multi-string split** | Emit a large mk1 as multiple strings | **LOW-VALUE** | Most single-sig account xpubs fit one string; multi-chunk is the deep-multisig long case. | M | chunk_set_id is a reassembly nonce, not key material; deterministic override exists. |
| **Master fingerprint computation** | Compute/display master fp + the xpub's fingerprint | **SHIPPED** | SH computes master fp via bip32; xpub-fingerprint folds into the decode review. | — | — |
| **Origin/derivation-path encoding** | Parse/display/select a BIP-380 origin path | **GAP** (display, rides decode) / N/A (wire internal) | Display is part of decode review; path selection part of seed→xpub derivation. | S | — |
| **SLIP-0132 prefix normalization** (ypub/zpub→xpub) | Normalize a typed ypub/zpub before engraving | **LOW-VALUE** | Niche input on a keypad device. | M | Deterministic base58check version-swap. |
| **policy_id_stub from md1** (`derive_stub_from_md1`) | Pair an md1 policy card with the mk1 xpub via the stub | **OUT-OF-SCOPE / deferred** | Requires the full md-codec WalletPolicyId machinery on-device — large dependency. | L | Defer unless SH gains md1 policy support. |
| **Cross-binding stub VERIFY** (`verify --from-md1`) | Confirm an mk1 stub matches the md1 policy card | **OUT-OF-SCOPE / deferred** | Same dependency. | M | — |
| **Public child-xpub derivation** (`mk derive`) | Derive a child xpub at an unhardened path | **OUT-OF-SCOPE / LOW-VALUE** | Wallet/explorer territory, little engraving value. | M | Off-mission. |
| **Address rendering** (`mk address`) | Show receive addresses from a card's xpub | **OUT-OF-SCOPE** | Wallet/verifier scope. [App-layer corrects: feasible via the in-tree `address` pkg — a wiring task — and IS high-value for verification.] | L | — |
| **Address-type inference from path** | Heuristic script-type from origin path | **OUT-OF-SCOPE** | Input to address rendering. | — | — |
| **`inspect`** | Richer decode view (path breakdown, xpub fp, per-chunk BCH variant) | **GAP** (folds into decode review) | The valuable parts ARE the decode-review screen. | S | — |
| **`vectors` / `gui-schema`** | — | **OUT-OF-SCOPE** | Maintainer/GUI tooling. | — | — |
| **Privacy-preserving mode** (omit origin_fingerprint) | Honor/display the fingerprint-omitted flag | **GAP** (trivial, rides decode) | Display flag during decode. | S | — |
| **Version/reserved-bit validation** | Reject unknown-version/reserved-bit mk1 | **GAP** (rides decode) | Necessary decoder correctness. | S | — |
| **Display grouping / separators** | Render mk1 in readable groups | **SHIPPED-ish / LOW-VALUE** | SH already lays out strings for engraving. | S | — |

### Top SH candidates from mk-codec (ranked)
1. **Decode an mk1 → DISPLAY xpub + origin fingerprint + path + policy-id stubs for pre-engrave verification** — GAP, M. Deterministic, public, reuses shipped BCH + bip32. Absorbs inspect / privacy-flag / cross-chunk integrity / version validation.
2. **Derive an account xpub from an entered BIP-39 seed (+ path selection) and engrave as mk1** — GAP, L. Not in mk-codec; SH already derives master fp, so within reach, deterministic (NOT TRNG-blocked); only the public xpub is emitted.
3. **mk1 field-assembly + single-string encode** — partial GAP, M. Valuable once #2 exists.

**No mk-codec feature is HW-BLOCKED** (entirely public-key/deterministic; the one CSPRNG touchpoint has a deterministic override and is absent from single-string cards). OUT-OF-SCOPE: address rendering [but see app-layer], child-xpub, md1-stub cross-binding, vectors/gui-schema. LOW-VALUE: SLIP-132, chunking, grouping.
