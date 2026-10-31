# Host-to-Device Interfaces & Constellation Contracts

This document is the interface reference and handoff contract between the **SeedHammer II (`SH2`) firmware** (`bg002h/seedhammer`) and the **Shibboleth host constellation** (`mnemonic-engrave`, `descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret`, `mnemonic-transaction`, `mnemonic-toolkit`, `mnemonic-gui`, and `mnemonic-refugium`).

---

## 1. Summary Matrix of Host-to-Device Interfaces

| Interface / Format | Primary Producer(s) | Firmware Consumer(s) | Transport / Medium | Current Version / Pin |
|---|---|---|---|---|
| **NFC NDEF Tag Payloads** | `me` CLI (`crates/me-cli`), mobile/desktop NFC writers | `nfc/poller`, `gui/scan.go`, `gui/bundle.go`, `gui/transaction.go` | ISO14443A Type 2/4 & ISO15693 Type 5 NFC (disabled under `-tags refugium`) | NFC Forum NDEF Well-Known `T` (UTF-8) and `U` records (`≤ 8 KiB`) |
| **Systemwide Flash Payload (`sysw`)** | `me sysw pack` (`crates/me-cli/src/sysw`), `mr-gui` (`mnemonic-refugium`) | `sysw/read_tinygo.go`, `sysw/header.go`, `sysw/open.go`, `gui/sysw_load.go`, `gui/sysw_session.go`, `gui/sysw_source.go` | USB BOOTSEL `picotool load -t bin -o 0x10D00000` (64 KiB XIP flash) | `MNEMSYSW` v1 (`version = 0x01`); vectors pinned at `mnemonic-engrave` `c50979f1` & `1cbecbfd` |
| **Encrypted Sealed Flash Payload (`seal`)** | `me seal` (`crates/me-cli/src/seal`) | `seal/read_tinygo.go`, `seal/open.go`, `gui/unlock_flow.go`, `gui/unlock_kdf.go`, `gui/unlock_session.go` | USB BOOTSEL `picotool load -t bin -o 0x10E00000` (64 KiB XIP flash) | `MNEMBLOB` v1 (`version = 0x01`); vectors pinned at `mnemonic-engrave` `4df43d6` |
| **`md1` Descriptor & Wallet Policy** | `md-codec` (`descriptor-mnemonic`), `me`, `mtk`, `mr-gui`, or on-device `composerFlow` | `codex32/mdmk.go`, `md/`, `gui/md1_gather.go`, `gui/wallet_policy.go` | NFC NDEF, `sysw`, `seal`, or on-device composer | Wire v4 & v8; `me-cli` pins `md-codec` `0.48.4` (`6cf6599a`); Go pins `5a0a4f41` + `430ea478` + `b2c5d693` |
| **`mk1` Extended Public Key Card** | `mk-codec` (`mnemonic-key`), `me`, `mtk`, `mr-gui`, or on-device `deriveXpubFlow` / `singleSigEngraveFlow` | `codex32/mdmk.go`, `mk/`, `gui/mk1_inspect.go`, `gui/key_card_seating.go` | NFC NDEF, `sysw`, `seal`, or on-device derivation | `mk-codec` `0.5` (`mk/testdata/csid_ext_v0.1.json`) |
| **`ms1` Codex32 Share & `m` Secret** | `ms-codec` (`mnemonic-secret`), `me`, `mtk`, `mr-gui`, or on-device entry/minting | `codex32/codex32.go`, `codex32/mspayload.go`, `codex32/msencode.go`, `hashlock/`, `gui/codex32_polish.go`, `gui/composer_hashlock_plates.go` | On-device touch keyboard, `sysw`, `seal`, or NFC (`Backup Wallet` only; refused in `bundleGatherer`) | BIP-93 + `m`-payload tags `0x00`/`0x02`/`0x03`; `ms-codec` `0.10.0` (`e5dff6f9` / `61904c55`) |
| **`mt1` & `tx:` Transaction Backups** | `mt-codec` (`mnemonic-transaction`), `me`, `mtk` | `mt/`, `txqr/`, `gui/transaction.go` | NFC NDEF or `sysw` (`ClassMt`, `ClassTx`) | `mt-codec` `0.1.0` (`72b79b87`) |
| **`me-preview` Sidecar Protocol** | `crates/me-cli/src/preview.rs` (`mnemonic-engrave`) | `preview/main.go` (links `third_party/seedhammer`) | Host CLI subprocess (`--preview <DIR> [--png]` / `ME_PREVIEW_BIN`) | Fork submodule `7b6f2fbb` (24 commits behind `origin/main`) |
| **RP2350 Signed UF2 & OTP Boot Key** | `flake.nix` (`picotool` 2.3.1), `cmd/picosign`, `scripts/sign-firmware.sh` | RP2350 mask ROM bootrom, `driver/otp`, `cmd/controller/platform_sh2.go` | USB BOOTSEL (`picotool load` / `picotool otp`) or on-device `LockBoot` | `picotool` `2.3.1` (`aa92bd55` + `sram-range-2.3.1.patch`); `secp256k1` (`CRIT1=0x43`) |

---

## 2. Detailed Interface Specifications

### 2.1 NFC Tag Framing & Payload Classification (`NDEF`)

- **Producer:** `me` CLI (`crates/me-cli` NDEF writer), standard NFC tools writing NFC Forum Type 2 (`NTAG21x`), Type 4 (ISO-DEP APDU), or Type 5 (ISO15693 `ST25TV`/`ICODE`) tags.
- **Consumer:** `driver/st25r3916` → `nfc/poller` (`nfc/type2`, `nfc/type4`, `nfc/type5`) → `gui/scan.go:scanner.Scan` (`parseNDEF`).
  - **Refugium Profile Note:** Under `-tags refugium`, `cmd/controller/nfc_refugium.go:initNFC` powers down `st25r3916` at boot (`d.Close()`, writing `regOpCtrl 0x02 = 0x00`) and returns `nil`; all NFC scan loops are unreachable.
- **Format / Protocol:**
  - Maximum NDEF message size: `8192` bytes (`scanner.Scan` buffer `make([]byte, 8192)` in `gui/scan.go`).
  - `parseNDEF` strips the NDEF record header (`TNF = 0x01` Well-Known):
    - Type `'T'` (Text): strips the status byte and IANA language tag (`status & 0x3f` bytes; rejects UTF-16 `status & 0x80 != 0`).
    - Type `'U'` (URI) or raw payload: passes payload bytes through.
  - Multi-line text payloads (`\n` or `\r\n`) are split and offered line-by-line in `bundleGatherFlowResume` (`gui/bundle_flow.go`).
- **Validation Owner:**
  - `gui/scan.go` owns NDEF unwrapping and dispatches to caller-specific parsers:
    - `Backup Wallet` (`gui/gui.go`): tries `seedqr.Parse`, `codex32.New`, `bip39.ParseMnemonic`, `slip39.ParseShare`.
    - `Engrave Bundle` / `Wallet Policy` (`gui/bundle.go:bundleGatherer.offer`): classifies lines via `classifyScanLine`:
      - Accepts `md1` (`codex32.ValidMD`) and chunked `mk1` (`codex32.ValidMK`).
      - **Explicitly refuses** `ms1` secret shares (`clsMs1Refuse` → `"Secret share refused"`) and unchunked single `mk1` cards (`clsSingleMK1Refuse`) on the bundle NFC door.
      - Accepts UR `crypto-output` (`ur.Decoder`) and plaintext BIP-380 descriptors (`bip380.Parse`).
    - `Engrave Transaction` (`gui/transaction.go:txGatherer.offer`): accepts `mt1` chunks (`mt.ParseChunk`) or `tx:` records.
- **Authoritative Specification:** `design/SPEC_engrave_transaction.md`, `design/SPEC_wallet_policy_composer.md`, and `gui/scan.go` / `gui/bundle.go`.
- **Compatibility Evidence:** `gui/scan_test.go`, `gui/bundle_test.go`, `gui/transaction_test.go`.
- **Lockstep Update Rules:**
  - Adding a new NDEF record wrapper or multi-record NDEF message format requires updating `parseNDEF` in `gui/scan.go` and the line splitter in `gui/bundle_flow.go`.
  - Increasing payload length above `8192` bytes requires enlarging the `buf` allocation in `gui/scan.go:scanner.Scan`.

---

### 2.2 Systemwide Flash Payload (`sysw` / `MNEMSYSW` at `0x10D00000`)

- **Producer:** `me sysw pack` (`crates/me-cli/src/sysw/mod.rs`), orchestrated by `mr-gui` (`mnemonic-refugium`) or host scripts, flashed via `picotool load -t bin payload.sysw -o 0x10D00000`.
- **Consumer:** `sysw.XIPReader` (`sysw/read_tinygo.go`) at `RegionAddr = 0x10D00000` → `sysw.ParseHeader` (`sysw/header.go`) & `sysw.Open` (`sysw/open.go`) → `gui/sysw_load.go:syswLoadFlow` → `gui/sysw_session.go:syswSession` → `gui/sysw_source.go`.
- **Format / Protocol (`MNEMSYSW` v1):**
  - Flash region: `0x10D00000..0x10D10000` (`65,536` bytes = `RegionLen`, 16 × 4 KiB sectors; `MaxSectionLen = 32,734`).
  - Fixed 52-byte **Big-Endian** header (`HeaderLen = 52`, `sysw/header.go:25–84`):
    - `[0..8]`: Magic `"MNEMSYSW"`
    - `[8]`: Version `0x01`
    - `[9]`: `kdf_id` (`0x01` = `KDFPBKDF2SHA256` when `ct_len > 0`, `0x00` when `ct_len == 0`)
    - `[10]`: `aead_id` (`0x01` = `AEADAES256GCM` when `ct_len > 0`, `0x00` when `ct_len == 0`)
    - `[11]`: Reserved (`0x00`)
    - `[12..16]`: `iterations` (`uint32` BE; if sealed: `100,000 <= iterations <= 2,000,000`; if unsealed: `0`)
    - `[16..32]`: `salt` (16 bytes; all-zero if unsealed)
    - `[32..44]`: `iv` (12 bytes; all-zero if unsealed)
    - `[44..48]`: `pub_len` (`uint32` BE, `<= 32,734`)
    - `[48..52]`: `ct_len` (`uint32` BE; `h.Sealed()` is `h.CtLen > 0`)
  - Body layout (`sysw/open.go:44–75`):
    - Public section: `blob[52 : 52+pub_len]` containing LF-separated (`\n`, `0x0A`) UTF-8 record lines.
    - Secret section: when unsealed (`ct_len == 0`), all records reside in `pub_len` and there is no secret section or tag. When sealed (`ct_len > 0`), `blob[52+pub_len : 52+pub_len+ct_len]` contains AES-256-GCM ciphertext + 16-byte GCM tag over `AAD = header[0..52] || public_section` with key `PBKDF2-HMAC-SHA256(utf8(words_joined_by_space), salt, iterations, 32)`. Decrypts to LF-separated (`\n`) UTF-8 record lines.
  - Public Data Hash (`sysw/pubhash.go:13–51`):
    - 128-bit (`[16]byte`) digest: `SHA256("MNEMSYSW/pub/v1" || 0x00 || byte(sealed) || byte(len(records)) || strings.Join(records, "\n"))[:16]`, formatted by `sysw.FormatHash` as 8 space-separated 4-hex groups (`xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx`).
  - Validating record classification (`sysw.Classify` in `sysw/record.go:24–154`, `sysw/classify.go`, `sysw/composer_records.go` — 15 `sysw.Class` enum values: `ClassUnknown` + 14 classes):
    - `ClassMnemonic` (checksum-valid BIP-39 words), `ClassCodex32Secret` (valid `ms1` share/secret), `ClassPassphrase` (`pass:<lowercase-hex>` decoded via `sysw.DecodeBody`; refused under `-tags refugium`), `ClassFreeText` (`text:<lowercase-hex>`), `ClassDescriptor` (valid descriptor string), `ClassMDMK` (valid `md1`/`mk1`), `ClassAddress` (enum value 7, reserved/unused), `ClassMt` (valid `mt1`), `ClassTx` (`tx:<lowercase-hex>` parsed via `mt.ParseTx`), `ClassKey` (`key:<lowercase-hex>` composer key record), `ClassHash` (`hash:` composer digest record), `ClassNow` (`now:` composer locktime height/time record), `ClassPreimage` (`preimage:` record), `ClassPhrase` (`phrase:<method>:<lowercase-hex>` record).
- **Version / Pin:**
  - Format version `0x01`.
  - `sysw/testdata/sysw_vectors.json` pinned to `mnemonic-engrave` commit `c50979f1a389858825ec97be36dffb25c01c193a` (SHA-256 `7e58779d7f0c80ab4713d17ae50c5200197cc422f77a7ef280a22acbc291a0ac`).
  - `sysw/testdata/record_class_vectors.json` pinned to `mnemonic-engrave` commit `1cbecbfd5ae387f19c864712e7be33ff5b6cd9c5` (82 vectors, SHA-256 `c5f721455b75513b4217c200cf49b4b0eed471fbfcb704121c3cebbd09066770`).
- **Validation Owner:**
  - Container integrity & record validation (`sysw.Classify`): `sysw` package (`sysw/header.go`, `sysw/open.go`, `sysw/classify.go`, `sysw/record.go`).
  - Cross-record provenance flags (`[compared]`, `[mdmk-decode]`, `[mt-decode]`): `gui/sysw_session.go` and destination flows (`gui/sysw_source.go`).
- **Authoritative Specification:** `design/SPEC_systemwide_payloads.md` and `design/SPEC_sh2_sysw_consumption.md`.
- **Compatibility Evidence:** `sysw/conformance_test.go`, `sysw/classify_conformance_test.go`, `sysw/vendored_vectors_test.go`, `sysw/vendored_vectors_live_test.go` (`-tags oraclelive`), `gui/sysw_load_test.go`.
- **Lockstep Update Rules:**
  - Any change to the 52-byte Big-Endian header, section limits (`MaxSectionLen = 32734`), `"MNEMSYSW/pub/v1"` hash preimage, or `sysw.Classify` rules in `crates/me-cli/src/sysw/` **must** regenerate `sysw/testdata/sysw_vectors.json` and `sysw/testdata/record_class_vectors.json` and update `seedhammer/sysw/` + `*.provenance.json` in lockstep.

---

### 2.3 Encrypted Sealed Flash Payload (`seal` / `MNEMBLOB` at `0x10E00000`)

- **Producer:** `me seal` (`crates/me-cli/src/seal/mod.rs`), flashed via `picotool load -t bin payload.bin -o 0x10E00000`.
- **Consumer:** `seal.XIPReader` (`seal/read_tinygo.go`) at `PayloadAddr = 0x10E00000` → `seal.Opener` (`seal/open.go`) → `gui/unlock_flow.go:unlockPayloadFlow`.
- **Format / Protocol (`MNEMBLOB` v1):**
  - Flash region: `0x10E00000..0x10E10000` (`65,536` bytes; max blob size `16,450` bytes).
  - Fixed 52-byte **Big-Endian** header (`HeaderLen = 52`, `seal/wire.go:81–209`): identical field layout to `MNEMSYSW` (`[0..8]` Magic `"MNEMBLOB"`, `[8]` Version `0x01`, `[9]` `kdf_id`, `[10]` `aead_id`, `[11]` `0x00`, `[12..16]` `iterations` `uint32` BE, `[16..32]` `salt`, `[32..44]` `iv`, `[44..48]` `pub_len` `uint32` BE, `[48..52]` `ct_len` `uint32` BE), with `MaxSectionLen = 8,191`, `MaxRecords = 24`, `MaxRecordLen = 512`.
  - Stricter LF-separated (`\n`) record admission rules (`seal/open.go`, `seal/container.go`, `seal/record.go`):
    - **Public section:** admits `ClassMDMK` only (must be lowercase, `codex32.ValidMD`/`ValidMK`, and fully reassemblable/decodable as a coherent set via `decodePublicSet`).
    - **Secret section:** encrypted with AES-256-GCM over `AAD = header[0..52] || public_section`; key derived from a **12-word checksum-valid BIP-39 passphrase** via PBKDF2-HMAC-SHA256 (`100,000..2,000,000` iterations). Admits `ClassMDMK`, `ClassCodex32Secret` (`<= 90` chars, valid `codex32.New`), and `ClassMnemonic` (12 or 24 checksum-valid BIP-39 words).
    - **Public Data Hash (`seal/pubhash.go:28–80`):** 128-bit (`[16]byte`) digest `SHA256("MNEMBLOB/pub/v1" || 0x00 || byte(sealed) || byte(len(records)) || strings.Join(records, "\n"))[:16]`, formatted by `seal.FormatHash` as 8 space-separated 4-hex groups (`xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx`).
- **Version / Pin:**
  - Format version `0x01`.
  - `seal/testdata/vectors.json` pinned to `mnemonic-engrave` commit `4df43d6` (SHA-256 `333ac47e7f61d031c995b85510565bfffd86cd1992f09b0230c1484fffd4d4bc`).
- **Validation Owner:** `seal` package (`seal/open.go`, `seal/record.go`) owns both container and strict record admission (`seal.Classification`) before returning `AdmittedPayload` to `gui/unlock_flow.go`.
- **Authoritative Specification:** `design/SPEC_encrypted_payload_delivery.md`.
- **Compatibility Evidence:** `seal/vectors_test.go`, `seal/open_test.go`, `gui/unlock_flow_test.go`.
- **Lockstep Update Rules:**
  - Changing `MNEMBLOB` header layout, `MaxSectionLen`, `"MNEMBLOB/pub/v1"` hash preimage, or `seal::record` admission rules in `crates/me-cli/src/seal` requires regenerating `seal/testdata/vectors.json` (`ME_EMIT_VECTORS=1 cargo test -p mnemonic-engrave --lib seal::tests::emit_vectors`) and updating `seedhammer/seal/` in lockstep.

---

### 2.4 Constellation String Codecs (`md1`, `mk1`, `ms1`, `mt1`, `hashlock`)

#### 2.4.1 `md1` — Descriptor & Wallet Policy Mnemonic
- **Producer:** `descriptor-mnemonic` (`crates/md-codec`), `me`, `mtk`, `mr-gui`, or on-device `gui/composer_flow.go` / `gui/singlesig_engrave.go` / `gui/multisig_engrave.go`.
- **Consumer:** `codex32/mdmk.go` (`ValidMD`), `md/` (`Decode`, `Reassemble`, `ExpandWalletPolicyChunks`, `FormAwareIdChunks`, `DeriveAddress`), `gui/md1_gather.go`, `gui/wallet_policy.go`.
- **Format / Protocol:**
  - HRP `md`, separator `1`, 5-bit Bech32 payload (unchunked or prefixed by a **37-bit chunk header** `[version:4][chunked=1:1][csid:20][count-1:6][index:6]` in `md/chunk.go:44–75`), followed by a **13-char regular BCH(93,80,8) checksum only** (`mdRegularMaxLen = 93` data symbols = max 96 chars total; `POLYMOD_INIT = 0x23181b3`, NUMS residue target).
  - Supports wire version 4 (`fe` = `0`) and version 8 (`p` = `1`, Taproot script trees, `multi_a`, `sortedmulti_a`, timelocks, hashlocks).
  - `WalletPolicyId` (`md/walletpolicyid.go:11–30`) and `WalletDescriptorTemplateId` (`md/template_id.go:21–39`) return **16-byte (`[16]byte`, 128-bit)** SHA-256 prefixes over canonical binary bitstream preimages (`writeNode(tree)` + per-`@N` binary key records for `WalletPolicyId`; `useSitePath || writeNode(tree) || [UseSitePathOverrides-TLV]` for `WalletDescriptorTemplateId`); their 4-byte prefixes are `WalletPolicyIDStub` and `WalletDescriptorTemplateIdStub`.
- **Version / Pin:** `me-cli` pins `md-codec` `0.48.4` (`6cf6599a`); Go `seedhammer/md` vectors pinned at `5a0a4f41`, `430ea478`, and `b2c5d693` (see [SOURCE_MAP.md](SOURCE_MAP.md#5-constellation-go-codec-provenance--update-procedure)).
- **Validation Owner:** `codex32.ValidMD` (BCH checksum) + `md.Reassemble` / `md.Decode` (wire AST, key-table consistency, Taproot leaf versions, duplicate keys).
- **Authoritative Specification:** `descriptor-mnemonic` crate docs and `design/SPEC_wallet_policy_composer.md`.
- **Compatibility Evidence:** `md/md_test.go`, `md/conformance_keyed_test.go`, `md/compose_vectors_pin_test.go`, `md/liana_test.go`, `oracle/live_test.go`.
- **Lockstep Update Rules & Known Divergences:**
  - Any wire opcode, canonical formatting, or `WalletPolicyId` bitstream change in `md-codec` requires re-vendoring `md/testdata/*.json` via `scripts/vendor-*.sh`.
  - **Known open divergences:** Go `md` does not yet implement pathless shared-origin decoding (`sh_wpkh`, `F-166`) or `md-codec 0.44.0` rendered-xpub header/origin agreement (see [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md); `F-212` elided-origin `WalletPolicyId` parity was closed on 2026-08-20).

#### 2.4.2 `mk1` — Extended Public Key Card Mnemonic
- **Producer:** `mnemonic-key` (`crates/mk-codec`), `me`, `mtk`, `mr-gui`, or on-device `deriveXpubFlow` / `singleSigEngraveFlow`.
- **Consumer:** `codex32/mdmk.go` (`ValidMK`), `mk/` (`Decode`, `DecodeSingle`, `Assembler`, `DerivedChunkSetID`), `gui/mk1_inspect.go`, `gui/key_card_seating.go`.
- **Format / Protocol:**
  - HRP `mk`, separator `1`, string-layer header of **2 symbols** (single) or **8 symbols** (chunked: `[version=0][type=1][csid:4 syms (20b)][total-1:1 sym][index:1 sym]` in `mk/mk.go:38–92`), 5-bit payload encoding version bytes (`xpub`/`zpub`/`tpub`/etc.), master fingerprint (4 bytes), derivation path steps, 33-byte compressed pubkey, 32-byte chaincode, optional policy/template ID stub, and regular (13-char) or long (15-char) BCH checksum.
  - Derived chunk set ID: `mk.DerivedChunkSetID(card)` (`mk/encode.go:64, 356`) computes the **top 20 bits of `SHA-256(encodeBytecode(card))`**.
- **Version / Pin:** `mk-codec` `0.5` (`mk/testdata/csid_ext_v0.1.json`).
- **Validation Owner:** `codex32.ValidMK` + `mk.Decode` / `mk.Assembler` + `gui/bundle.go:showBundleCSIDMismatchNotices`.
- **Authoritative Specification:** `mnemonic-key` (`crates/mk-codec`).
- **Compatibility Evidence:** `mk/mk_test.go`, `mk/chunk_set_id_parity_test.go`, `oracle/live_test.go`.
- **Lockstep Update Rules:** Wire bytecode or `DerivedChunkSetID` changes in `mk-codec` must be ported to `seedhammer/mk/` with updated `mk/testdata/` vectors.

#### 2.4.3 `ms1` — BIP-93 `codex32` Shares & `m`-Format Binary Secrets (`entr`, `mnem`, `hash`)
- **Producer:** `mnemonic-secret` (`crates/ms-codec`), `me`, `mtk`, `mr-gui`, or on-device `codex32` minting / `composerHashlockPlatesFlow`.
- **Consumer:** `codex32/` (`New`, `Interpolate`, `DecodeMS1`, `EncodeMS1`, `EncodeMS1Preimage`, `DecodeMS1Preimage`, `Correct`), `hashlock/`, `gui/codex32_polish.go`, `gui/ms1_decode.go`, `gui/composer_hashlock_plates.go`.
- **Format / Protocol:**
  - Standard BIP-93 `ms1` strings (48 chars for 128-bit secret, 74 chars for 256-bit secret, up to 90 chars with 15-char long checksum).
  - For unshared threshold `k = 0` (`0` after `ms1`) and share index `s`: raw BIP-93 entropy (`entr` / `s000`).
  - For `m`-format binary secret payloads (share index `m` or `M`): 5-bit groups unpack into `[tag: 1 byte][body...]`:
    - `tag = 0x00` (`KindRawEntropy`, ID `entr`): 16 or 32 bytes of raw BIP-32 seed entropy.
    - `tag = 0x02` (`KindMnemonic`, ID `mnem`): `[lang_selector: 1 byte][16 or 32 bytes BIP-39 entropy]` (`0x00` = English; non-English selectors decode entropy on `SH2` only when routed as raw entropy or refused for word display).
    - `tag = 0x03` (`KindHashlock`, ID `hash`): `[hash_func_id: 1 byte][32-byte preimage]` where `0x00=hash160`, `0x01=sha256`, `0x02=ripemd160`, `0x03=hash256` (`codex32/msencode.go`, `hashlock/hashlock.go`).
- **Version / Pin:** `ms-codec` `0.10.0` (`e5dff6f9c2b7ae7dc02a4a137e7322abab32211e` in `backup/testdata/ms1_seedqr_vectors.provenance.json`; `61904c5576d8ab280fb5587ac23db72531be950e` in `hashlock/testdata/hashlock-v0.8.provenance.json`).
- **Validation Owner:** `codex32.New` (BIP-93 regular/long polymod), `codex32.DecodeMS1`, `codex32.DecodeMS1Preimage`.
- **Authoritative Specification:** `mnemonic-secret` (`crates/ms-codec`) and `design/SPEC_wallet_policy_composer.md`.
- **Compatibility Evidence:** `codex32/codex32_test.go`, `codex32/mspayload_test.go`, `codex32/msencode_test.go`, `hashlock/hashlock_test.go`, `backup/ms1_seedqr_test.go`, `oracle/live_test.go`.
- **Lockstep Update Rules:** Any new `m`-payload tag byte or language selector in `ms-codec` must be added to `codex32/mspayload.go`, `codex32/msencode.go`, and `hashlock/hashlock.go` with updated vector files.

#### 2.4.4 `mt1` & `tx:` — Transaction Backups
- **Producer:** `mnemonic-transaction` (`crates/mt-codec`), `me`, `mtk`.
- **Consumer:** `mt/` (`ParseChunk`, `Reassemble`, `Decode`, `ParseTx`), `txqr/` (`EncodeSet`), `gui/transaction.go`.
- **Format / Protocol:**
  - `mt1` (`mt/mt.go:39–178`, `codex32/mtdata.go:19–42`): HRP `mt`, separator `1`, **11-symbol (55-bit) chunk header** (`version:5 (=1, 'p') || chunk_set_id:20 || total_chunks-1:15 || chunk_index:15`), 5-bit payload carrying raw transaction chunk bytes (no flags byte or `u24` length prefix), and a **13-symbol regular BCH(93,80,8) checksum only** (no long code; max 96 chars total; `mdmkPolymodInitLo = 0x23181b3`, NUMS target `0x0001a2fc877f9528d7c1` = top 65 bits of `SHA-256("shibbolethnumstransaction")`).
  - `Tx.ChunkSetID()` (`mt/mt.go:162`): top 20 bits (first 5 hex characters) of `TxidDisplay`.
  - `txqr` (`txqr/txqr.go`): `txqr.EncodeSet(data, k, level)` encodes raw transaction bytes in **QR byte mode** (`k = 1` single symbol, or `2 <= k <= 16` ISO/IEC 18004 **Structured Append** 20-bit headers + byte-mode payloads).
- **Version / Pin:** `mt-codec` `0.1.0` (`72b79b8762ec208b6143afade8c4b66a551bd29d`).
- **Validation Owner:** `mt.Decode` + `mt.ParseTx` (`gui/transaction.go:41–109` surfaces unconfirmed `mt1` sets (`Header.Version == 0`) and unsigned `tx:` records (`IsEffectivelyUnsigned()`) on the review screen and keeps them engraveable under a mandatory substituted warning legend `subst` per rulings `2026-08-25a`/`2026-08-25b`).
- **Authoritative Specification:** `design/SPEC_engrave_transaction.md` and `mnemonic-transaction/crates/mt-codec`.
- **Compatibility Evidence:** `mt/mt_test.go`, `txqr/txqr_test.go`, `gui/transaction_test.go`.
- **Lockstep Update Rules:** Any change to the 55-bit `mt1` header layout or BCH constants in `mt-codec` requires updating `seedhammer/mt/` and `seedhammer/codex32/mtdata.go`.

---

### 2.5 `me-preview` Go Sidecar Protocol & Submodule Boundary

- **Producer:** `crates/me-cli/src/preview.rs` (`me --preview <DIR> [--png]`).
- **Consumer:** `preview/main.go` (`me-preview` binary, resolved via `ME_PREVIEW_BIN` or adjacent to `me`).
- **Format / Protocol (`preview/main.go:25–121`, `crates/me-cli/src/preview.rs:111–195`):**
  - Version handshake: `me-preview --version` → stdout `me-preview <semver>` (must match `me`'s `CARGO_PKG_VERSION`).
  - Render invocation: `me-preview render --format <svg|png> --out <FILE|-> [--mode <text+qr|text|qr>]` with the **raw plate text piped on `stdin`**. `me-preview` writes the SVG/PNG (`0o600`, or stdout if `--out -`) and prints `mode <chosenMode>` on stdout (or stderr if `--out -`).
- **Version / Pin & Submodule Boundary:**
  - `preview/go.mod` uses `replace seedhammer.com => ../third_party/seedhammer`, which is pinned to **`bg002h/seedhammer` fork commit `7b6f2fbbdb909ef812ccc8c90768280c5fa7a4d4`** (24 commits behind `seedhammer` `origin/main` `abdcb182`).
  - `preview/params.go:18` still hardcodes `EngravingSpeed: 8 * mm` (copied before `cmd/controller/platform_sh2.go:223` lowered it to `4 * mm`). For latest fork-HEAD rendering, use `cmd/plateview` inside `seedhammer`.
- **Validation Owner:** `crates/me-cli/src/preview.rs` and `preview/render_test.go`.
- **Authoritative Specification:** `preview/main.go` and `crates/me-cli/src/preview.rs`.
- **Lockstep Update Rules:** Advancing `third_party/seedhammer` to `seedhammer` `origin/main` requires updating `preview/` golden SVG/PNG expectations if layout or font metrics changed (`design/FOLLOWUPS.md` `F-213`).

---

### 2.6 Firmware Build, RP2350 Image Signing & OTP Provisioning

- **Producer:**
  - Nix devshell in `flake.nix` providing TinyGo + **patched `picotool` 2.3.1** (upstream `aa92bd55` + `nix/patches/picotool-seal-sram-range-2.3.1.patch` fixing SRAM clear end address `0x20082000` → `0x20080000` so Scratch X/Y stacks survive bootrom load-map execution).
  - `cmd/picosign` + `scripts/sign-firmware.sh` (in `mnemonic-engrave/scripts/`).
  - OTP provisioning scripts in `mnemonic-engrave/scripts/`: `scripts/pico2-bootkey-rehearsal.sh` and `scripts/refugium-otp.sh`.
- **Consumer:** RP2350 mask ROM bootrom, `driver/otp/otp_rp2350.go`, and `cmd/controller/platform_sh2.go` (`LockBoot`).
- **Format / Protocol:**
  - UF2 family ID: `0xe48bff59` (`rp2350-arm-s`).
  - `picotool seal --quiet --hash --sign --clear build/controller.elf build/controller.uf2 <dummy.pem> --no-overwrite` emits a `PICOBIN_BLOCK` with load map (hash + SRAM clear `[0x20000000, 0x20080000)`), SHA-256 hash, 64-byte public key (`X || Y`), and 64-byte signature (`r || s`).
  - `cmd/picosign -uf2 ... -key ... -sigfmt raw` replaces the dummy key/signature with a `secp256k1` signature over the SHA-256 of the load-map ranges (`picobin/picobin.go`).
  - OTP layout (`driver/otp`, `docs/custom-firmware.md`):
    - `BOOTKEY0..3` (`0x0080..0x00bf`, ECC): SHA-256 of the 64-byte little-endian `secp256k1` public key.
    - `CRIT1` (`0x0005`, raw 3× redundancy): `0x000043` (`SECURE_BOOT_ENABLE | DEBUG_DISABLE | GLITCH_DETECTOR_ENABLE | GLITCH_DETECTOR_SENS=2`).
    - `BOOT_FLAGS0` (`0x0048`, raw 3× redundancy): bits `(1 << (slot + 8))` (`KEY_VALID`) and optionally `KEY_INVALID` (`bits 12..15`).
    - Page locks: `PAGE1_LOCK1` (`0x0f85`) and `PAGE2_LOCK1` (`0x0f89`) set to `0x15` (`LOCK_BL`).
- **Authoritative Specification:** `docs/custom-firmware.md`, `design/PICOTOOL_PIN.md`, and `FIRMWARE-QUICKSTART.txt`.
- **Compatibility Evidence:** `picobin/picobin_test.go`, `uf2/uf2_test.go`, `cmd/picosign/main.go`, `scripts/refugium-elf-check.sh`, and physical Pico 2 rehearsals recorded in `design/PICOTOOL_PIN.md`.
- **Lockstep Update Rules:** Never replace the devshell's patched `picotool` with unpatched upstream `picotool 2.3.1` when using `picotool seal --clear`; unpatched `2.3.1` emits SRAM clear end `0x20082000`, which wipes the bootrom's own stack in Scratch Y (`0x20081000..0x20082000`) and bricks boot into `USB_BOOT`.
