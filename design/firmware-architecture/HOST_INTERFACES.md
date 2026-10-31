# Host-to-Device Interfaces & Constellation Contracts

This document is the interface reference and handoff contract between the **SeedHammer II (`SH2`) firmware** (`bg002h/seedhammer`) and the **Shibboleth host constellation** (`mnemonic-engrave`, `descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret`, `mnemonic-transaction`, `mnemonic-toolkit`, `mnemonic-gui`, `mnemonic-refugium`, and `refugium-wallet`).

---

## 1. Summary Matrix of Host-to-Device Interfaces

| Interface / Format | Primary Producer(s) | Firmware Consumer(s) | Transport / Medium | Current Version / Pin |
|---|---|---|---|---|
| **NFC NDEF Tag Payloads** | `me` CLI (`crates/me-cli/src/ndef.rs`, `<= 249` text bytes), standard NFC writers | `nfc/poller`, `nfc/ndef`, `gui/scan.go`, `gui/bundle.go`, `gui/transaction.go` | ISO14443A Type 2/4 & ISO15693 Type 5 NFC (disabled under `-tags refugium`) | NFC Forum NDEF Well-Known `'T'` (UTF-8) and `'U'` records (`<= 8,191` decoded bytes per record in `scanner.Scan`) |
| **Systemwide Flash Payload (`sysw`)** | `me sysw pack` (`crates/me-cli/src/sysw`), `mr` / `mr-gui` (`mnemonic-refugium` / `refugium-wallet`) | `sysw/read_tinygo.go`, `sysw/header.go`, `sysw/open.go`, `gui/sysw_load.go`, `gui/sysw_session.go`, `gui/sysw_source.go` | USB BOOTSEL `picotool load -t bin -o 0x10D00000` (64 KiB XIP flash) | `MNEMSYSW` v1 (`version = 0x01`); vectors pinned at `mnemonic-engrave` `c50979f1` & `1cbecbfd` |
| **Encrypted Sealed Flash Payload (`seal`)** | `me seal` (`crates/me-cli/src/seal`) | `seal/read_tinygo.go`, `seal/open.go`, `gui/unlock_flow.go`, `gui/unlock_kdf.go`, `gui/unlock_session.go` | USB BOOTSEL `picotool load -t bin -o 0x10E00000` (64 KiB XIP flash) | `MNEMBLOB` v1 (`version = 0x01`); vectors pinned at `mnemonic-engrave` `4df43d6` |
| **`md1` Descriptor & Wallet Policy** | `md-codec` (`descriptor-mnemonic`), `me`, `mnemonic`, `mr` / `mr-gui`, or on-device `composerFlow` | `codex32/mdmk.go`, `md/`, `gui/md1_gather.go`, `gui/wallet_policy.go` | NFC NDEF, `sysw`, `seal`, or on-device composer | Wire v4 & v8; `me-cli` pins `md-codec` `0.48.4` (`6cf6599a`); Go pins `5a0a4f41` + `430ea478` + `b2c5d693` |
| **`mk1` Extended Public Key Card** | `mk-codec` (`mnemonic-key`), `me`, `mnemonic`, `mr` / `mr-gui`, or on-device `deriveXpubFlow` / `singleSigEngraveFlow` | `codex32/mdmk.go`, `mk/`, `gui/mk1_inspect.go`, `gui/key_card_seating.go` | NFC NDEF (chunked only), `sysw`, `seal`, or on-device derivation | `mk-codec` `0.5` (`mk/testdata/csid_ext_v0.1.json`) |
| **`ms1` Codex32 Share & `m` Secret** | `ms-codec` (`mnemonic-secret`), `me`, `mnemonic`, `mr` / `mr-gui`, or on-device entry/minting | `codex32/codex32.go`, `codex32/mspayload.go`, `codex32/msencode.go`, `hashlock/`, `gui/codex32_polish.go`, `gui/composer_hashlock_plates.go` | On-device touch keyboard, `sysw`, `seal`, or NFC (`Backup Wallet` only; refused in `bundleGatherer`) | BIP-93 + `m`-payload prefixes `0x00`/`0x02` (ID `entr`, index `s`) & `0x03` (ID `hash`, index `s`); `ms-codec` `0.10.0` (`e5dff6f9` / `61904c55`) |
| **`mt1` & `tx:` Transaction Backups** | `mt-codec` (`mnemonic-transaction`), `me` | `mt/`, `txqr/`, `gui/transaction.go` | `mt1` via NFC NDEF or `sysw` (`ClassMt`); `tx:` via `sysw` only (`ClassTx`) | `mt-codec` `0.1.0` (`72b79b87`) |
| **`me-preview` Sidecar Protocol** | `me bundle --preview <DIR> [--png]` (`crates/me-cli/src/preview.rs`) | `preview/main.go` (links `third_party/seedhammer`) | Host CLI subprocess (`me-preview render` / `ME_PREVIEW_BIN`) | Fork submodule `7b6f2fbb` (24 commits behind `origin/main`) |
| **RP2350 Signed UF2 & OTP Boot Key** | `flake.nix` (`picotool` 2.3.1 + `picotool-2.3.1-seal-clear-fix.patch`), `cmd/picosign`, `scripts/sign-firmware.sh` | RP2350 mask ROM bootrom, `driver/otp`, `cmd/controller/platform_sh2.go` | USB BOOTSEL (`picotool load` / `picotool otp`) or on-device `LockBoot` | `picotool` `2.3.1` (`F-701` `Clear 0x20000000->0x20082000`); `secp256k1` (`CRIT1` `0x040`, `BOOT_FLAGS1` `0x04b`) |

---

## 2. Detailed Interface Specifications

### 2.1 NFC Tag Framing, Record Decoding & Caller Admission (`NDEF`)

- **Producer:** `me` CLI (`crates/me-cli/src/ndef.rs`), standard NFC tools writing NFC Forum Type 2 (`NTAG21x`), Type 4 (ISO-DEP APDU), or Type 5 (ISO15693 `ST25TV`/`ICODE`) tags.
- **Consumer:** `driver/st25r3916` → `nfc/poller/poller.go` (`nfc/type2`, `nfc/type4`, `nfc/type5`) → `nfc/ndef/ndef.go` (`MessageReader`, `RecordReader`) → `gui/scan.go:scanner.Scan`.
  - **Refugium Profile Note:** Under `-tags refugium`, `cmd/controller/nfc_refugium.go:initNFC` powers down `st25r3916` at boot (`d.Close()`, writing `regOpCtrl 0x02 = 0x00`) and returns `nil`; all NFC scan loops are unreachable.
- **Format / Protocol & Framing Boundaries:**
  - **Host `me` NDEF writer (`crates/me-cli/src/ndef.rs:29–61`):** Emits a single Short Record (`SR = 1`) Well-Known Text (`'T'`, language `"en"`) wrapped in a 1-byte length Type 2 TLV (`0x03 <len> ... 0xFE`), limiting `me`'s single-record NDEF output to **249 text bytes** (`MAX_TEXT_LEN = 249`).
  - **Device NDEF record decoder (`nfc/poller/poller.go:84–90`, `nfc/ndef/ndef.go:95–229`):** `poller` wraps the tag stream in `ndef.NewMessageReader` and `ndef.NewRecordReader` *before* calling `scanner.Scan`. `RecordReader.Read` parses each record header, returns `io.EOF` at record boundaries, and skips chunked records (`flagCF`) or non-Well-Known TNFs (`tnf != 0x01`):
    - Well-Known `'T'` (Text, `nfc/ndef/ndef.go:183–201`): strips the status byte and IANA language code (`status & 0x3f` bytes) and skips UTF-16 records (`status & 0x80 != 0`).
    - Well-Known `'U'` (URI, `nfc/ndef/ndef.go:202–224`): consumes the URI identifier byte (`0x00` `""`, `0x01` `"http://www."`, `0x02` `"https://www."`, `0x03` `"http://"`, `0x04` `"https://"`; skips all other prefix codes) and prepends the expanded prefix string to the decoded content stream.
  - **Decoded-content buffer cap (`gui/scan.go:29–40`):** `scanner.Scan` allocates an `8,192`-byte buffer (`make([]byte, 8*1024)`) per record and triggers `errScanOverflow` when `s.n == len(s.buf)`, so the maximum decoded record content is **8,191 bytes**.
  - **No embedded line splitting:** Each NDEF record produces a single `scanner.Scan` object; embedded `\n` or `\r\n` bytes inside one Text record are **not** split into multiple chunk offers (`gui/bundle_flow.go:263–267`). Multi-chunk NFC input requires separate NDEF records or separate tag taps.
- **Scanner Recognition vs. Per-Program Caller Admission:**
  - **`scanner.Scan` (`gui/scan.go:58–107`):** Recognizes one decoded record as:
    1. `"command: <cmd>"` → `debugCommand` (ignored under `-tags refugium` via `gui/debugcmd_refugium.go`),
    2. `text:<lowercase-hex>` or `pass:<lowercase-hex>` → `freeTextScan` / `passScan` decoded via `sysw.DecodeBody` (matched before sniffers; malformed hex is refused with `errScanUnknownFormat`),
    3. `bip39.Parse` → `bip39.Mnemonic` (note: the `slip39.ParseShare` arm at `gui/scan.go:82–86` is commented out to save ~55 KB RAM, and `scanner.Scan` has no `seedqr` or `UR` decoder),
    4. `nonstandard.OutputDescriptor` → `bip380.Descriptor`,
    5. `codex32.New` with `!codex32.IsPreimage(s)` → `codex32.String`,
    6. `codex32.ValidMD` or `codex32.ValidMK` → `mdmkText`,
    7. `codex32.ValidMT` → `mtText`,
    8. `btcaddr.DecodeAddress` (mainnet/testnet3) → `addressText`.
  - **Per-program caller admission:**
    - `Backup Wallet` (`gui/gui.go`): admits `bip39.Mnemonic` and `codex32.String`.
    - `Engrave Bundle` / `Wallet Policy` NFC gatherer (`gui/bundle.go:157–192`, `classify(obj any)`): admits `mdmkText` (`md1` standalone or chunked; `mk1` chunked only), **explicitly refuses** `codex32.String` (`clsMs1Refuse` → `"Secret share refused"`) and unchunked single `mk1` (`clsSingleMK1Refuse`), and drops (`clsDrop`) all other `scanner.Scan` types (including `bip380.Descriptor`).
    - `Engrave Transaction` NFC gatherer (`gui/transaction.go:686–692, 750–765`): admits only `mtText` objects passing `mt.ParseHeader`. (`tx:<hex>` records are **not** admitted over NFC; they are consumed from `ctx.sysw.takeAll(sysw.ClassTx)` at `gui/transaction.go:451–484`.)
- **Authoritative Specification:** `design/SPEC_engrave_transaction.md`, `design/SPEC_wallet_policy_composer.md`, `nfc/ndef/ndef.go`, and `gui/scan.go` / `gui/bundle.go`.
- **Compatibility Evidence:** `nfc/ndef/ndef_test.go`, `gui/scan_test.go`, `gui/bundle_test.go`, `gui/transaction_test.go`.
- **Lockstep Update Rules:**
  - Changing NDEF record framing or URI prefix handling requires updating `nfc/ndef/ndef.go` (and `crates/me-cli/src/ndef.rs` if changing the host single-record TLV encoder, which is capped at 249 text bytes).
  - Increasing decoded record capacity above `8,191` bytes requires enlarging `s.buf` in `gui/scan.go:scanner.Scan`.

---

### 2.2 Systemwide Flash Payload (`sysw` / `MNEMSYSW` at `0x10D00000`)

- **Producer:** `me sysw pack` (`crates/me-cli/src/sysw/mod.rs`), orchestrated by `mr` / `mr-gui` (`mnemonic-refugium` / `refugium-wallet`) or host scripts, flashed via `picotool load -t bin payload.sysw -o 0x10D00000`.
- **Consumer:** `sysw.XIPReader` (`sysw/read_tinygo.go`) at `RegionAddr = 0x10D00000` → `sysw.ParseHeader` (`sysw/header.go`) & `sysw.Open` (`sysw/open.go`) → `gui/sysw_load.go:syswLoadFlow` → `gui/sysw_session.go:syswSession` → `gui/sysw_source.go`.
- **Format / Protocol (`MNEMSYSW` v1):**
  - Flash region: `0x10D00000..0x10D10000` (`65,536` bytes = `RegionLen`, 16 × 4 KiB sectors; `MaxSectionLen = 32,734`).
  - Fixed 52-byte **Big-Endian** header (`HeaderLen = 52`, `sysw/header.go:25–84`, `sysw/wire.go:92–110`):
    - `[0..8]`: Magic `"MNEMSYSW"`
    - `[8]`: Version `0x01`
    - `[9]`: `kdf_id` (`0x01` = `KDFPBKDF2SHA256` when `ct_len > 0`, `0x00` when `ct_len == 0`)
    - `[10]`: `aead_id` (`0x01` = `AEADAES256GCM` when `ct_len > 0`, `0x00` when `ct_len == 0`)
    - `[11]`: Reserved (`0x00`)
    - `[12..16]`: `iterations` (`uint32` BE; if sealed: `100,000 <= iterations <= 2,000,000`; if unsealed: `0`)
    - `[16..32]`: `salt` (16 bytes; all-zero if unsealed)
    - `[32..44]`: `iv` (12 bytes; all-zero if unsealed)
    - `[44..48]`: `pub_len` (`uint32` BE, `<= 32,734`)
    - `[48..52]`: `ct_len` (`uint32` BE, `<= 32,734`; `h.Sealed()` is `h.CtLen > 0`; **`ct_len` is the ciphertext length excluding the 16-byte GCM authentication tag**)
  - Body layout (`sysw/wire.go:102–110`, `sysw/open.go:44–75`, `crates/me-cli/src/sysw/mod.rs:634–650`):
    - Public section: `blob[52 : 52+pub_len]` containing LF-separated (`\n`, `0x0A`) UTF-8 record lines.
    - Secret section & separate 16-byte GCM tag:
      - When unsealed (`ct_len == 0`), total wire length is `52 + pub_len` and there is no secret section or tag.
      - When sealed (`ct_len > 0`), `blob[52+pub_len : 52+pub_len+ct_len]` is the AES-256-GCM ciphertext (`ct_len` bytes) and **`blob[52+pub_len+ct_len : 52+pub_len+ct_len+16]` is the separate 16-byte AES-256-GCM authentication tag** (`TagLen = 16`), so **`h.TotalLen() = 52 + pub_len + ct_len + 16`**. Encrypted over `AAD = header[0..52] || public_section` with key `PBKDF2-HMAC-SHA256(utf8(words_joined_by_space), salt, iterations, 32)`. Decrypts to LF-separated (`\n`) UTF-8 record lines.
  - Public Data Hash (`sysw/pubhash.go:13–51`):
    - 128-bit (`[16]byte`) digest: `SHA256("MNEMSYSW/pub/v1" || 0x00 || byte(sealed) || byte(len(records)) || strings.Join(records, "\n"))[:16]`, formatted by `sysw.FormatHash` as 8 space-separated 4-hex groups (`xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx`).
  - Validating record classification (`sysw.Classify` in `sysw/classify.go:35–175`, `sysw/record.go`, `sysw/composer_records.go` — 15 `sysw.Class` enum values: `ClassUnknown` + 14 classes):
    - `ClassMnemonic`: checksum-valid lowercase BIP-39 mnemonic of **12, 15, 18, 21, or 24 words** (`isStrictMnemonic`).
    - `ClassPreimage`: 75-character unshared `ms1` codex32 preimage plate (`<= 90` chars, threshold `'0'`, 4-char ID **`hash`**, share index `'s'`, 33-byte binary payload `[0x03][32-byte preimage]` via `codex32.IsPreimagePlate`; matched before `isStrictMs1`).
    - `ClassCodex32Secret`: valid `ms1` codex32 string `<= 90` chars (`MaxEngraveableMs1Len`) that is neither `codex32.IsPreimage` nor a `0x03`-kind single under ID `hash` (`!isHashIdPreimageKind`, `F-503`). **Intentional host/device admission asymmetry:** `me` (`crates/me-cli/src/seal/record.rs:177–231`) validates seed records through `ms_codec::decode` (requiring `Entr` `0x00`, `Mnem` `0x02`, or `k >= 2` share), whereas `seedhammer` admits the wider BIP-93-valid `ms1` population up to 90 chars (`host-admits` $\implies$ `device-admits`, tested by `crates/me-cli/tests/codex32_seam.rs` and `sysw/codex32_seam_test.go`).
    - `ClassPassphrase`: `pass:<lowercase-hex>` decoded via `sysw.DecodeBody` (refused under `-tags refugium`).
    - `ClassFreeText`: `text:<lowercase-hex>` decoded via `sysw.DecodeBody`.
    - `ClassDescriptor`: valid single-line descriptor record (`isDescriptorRecord`).
    - `ClassMDMK`: BCH-valid `md1` or `mk1` string (`codex32.ValidMD` / `ValidMK`).
    - `ClassAddress`: enum value 7 (reserved/unused).
    - `ClassMt`: BCH-valid `mt1` string whose header parses via `mt.ParseHeader` (wire version `1`).
    - `ClassTx`: `tx:<lowercase-hex>` raw transaction record parsed via `mt.ParseTx`.
    - `ClassKey`: `key:<lowercase-hex>` composer key record.
    - `ClassHash`: `hash:<algo>:<digest-hex>` composer digest record (`sha256`, `hash256`, `ripemd160`, `hash160`).
    - `ClassNow`: `now:<uint32>` composer locktime reference record.
    - `ClassPhrase`: **`phrase:<lowercase-hex(UTF8(method + "," + phrase))>`** where `method` is `"hardened"` or `"sha256"` and `ParsePhraseRecord` (`sysw/composer_records.go:150–200`, `crates/me-cli/src/sysw/composer_records.rs:27–40`) unhexes the body and splits at the **first comma** (`strings.Cut(string(b), ",")`).
- **Version / Pin:**
  - Format version `0x01`.
  - `sysw/testdata/sysw_vectors.json` pinned to `mnemonic-engrave` commit `c50979f1a389858825ec97be36dffb25c01c193a` (SHA-256 `7e58779d7f0c80ab4713d17ae50c5200197cc422f77a7ef280a22acbc291a0ac`).
  - `sysw/testdata/record_class_vectors.json` pinned to `mnemonic-engrave` commit `1cbecbfd5ae387f19c864712e7be33ff5b6cd9c5` (82 vectors, SHA-256 `c5f721455b75513b4217c200cf49b4b0eed471fbfcb704121c3cebbd09066770`).
- **Validation Owner:**
  - Container integrity & record validation (`sysw.Classify`): `sysw` package (`sysw/header.go`, `sysw/open.go`, `sysw/classify.go`, `sysw/record.go`).
  - Cross-record provenance flags (`[compared]`, `[mdmk-decode]`, `[mt-decode]`): `gui/sysw_session.go` and destination flows (`gui/sysw_source.go`).
- **Authoritative Specification:** `design/SPEC_systemwide_payloads.md` and `design/SPEC_sh2_sysw_consumption.md`.
- **Compatibility Evidence:** `sysw/conformance_test.go`, `sysw/classify_conformance_test.go`, `sysw/codex32_seam_test.go`, `sysw/vendored_vectors_test.go`, `sysw/vendored_vectors_live_test.go` (`-tags oraclelive`), `gui/sysw_load_test.go`.
- **Lockstep Update Rules:**
  - Any change to the 52-byte Big-Endian header, section limits (`MaxSectionLen = 32734`), `"MNEMSYSW/pub/v1"` hash preimage, or `sysw.Classify` rules in `crates/me-cli/src/sysw/` **must** regenerate `sysw/testdata/sysw_vectors.json` and `sysw/testdata/record_class_vectors.json` and update `seedhammer/sysw/` + `*.provenance.json` in lockstep.

---

### 2.3 Encrypted Sealed Flash Payload (`seal` / `MNEMBLOB` at `0x10E00000`)

- **Producer:** `me seal` (`crates/me-cli/src/seal/mod.rs`), flashed via `picotool load -t bin payload.bin -o 0x10E00000`.
- **Consumer:** `seal.XIPReader` (`seal/read_tinygo.go`) at `PayloadAddr = 0x10E00000` → `seal.Opener` (`seal/open.go`) → `gui/unlock_flow.go:unlockPayloadFlow`.
- **Format / Protocol (`MNEMBLOB` v1):**
  - Flash region: `0x10E00000..0x10E10000` (`65,536` bytes; max blob size `16,450` bytes = `52 + 2*8191 + 16`).
  - Fixed 52-byte **Big-Endian** header (`HeaderLen = 52`, `seal/wire.go:81–209`): identical field layout to `MNEMSYSW` (`[0..8]` Magic `"MNEMBLOB"`, `[8]` Version `0x01`, `[9]` `kdf_id`, `[10]` `aead_id`, `[11]` `0x00`, `[12..16]` `iterations` `uint32` BE, `[16..32]` `salt`, `[32..44]` `iv`, `[44..48]` `pub_len` `uint32` BE, `[48..52]` `ct_len` `uint32` BE), with `MaxSectionLen = 8,191`, `MaxRecords = 24`, `MaxRecordLen = 512`. As in `MNEMSYSW`, **`ct_len` excludes the separate 16-byte AES-256-GCM authentication tag**, so a sealed blob has total length **`52 + pub_len + ct_len + 16`** (`seal/wire.go:196–200`).
  - Stricter LF-separated (`\n`) record admission rules (`seal/open.go`, `seal/container.go`, `seal/record.go`):
    - **Public section:** admits `ClassMDMK` only (must be lowercase, `codex32.ValidMD`/`ValidMK`, and fully reassemblable/decodable as a coherent set via `decodePublicSet`).
    - **Secret section:** encrypted with AES-256-GCM (`ct_len` ciphertext bytes + 16-byte GCM tag) over `AAD = header[0..52] || public_section`; key derived from a **12-word checksum-valid BIP-39 unlock passphrase** via PBKDF2-HMAC-SHA256 (`100,000..2,000,000` iterations). Admits `ClassMDMK`, `ClassCodex32Secret` (`<= 90` chars, valid `codex32.New` with `!codex32.IsPreimage` — note the same `host-admits` $\implies$ `device-admits` seam with `crates/me-cli/src/seal/record.rs` as in §2.2), and `ClassMnemonic` (checksum-valid BIP-39 mnemonic of **12, 15, 18, 21, or 24 words** via `bip39.Parse` in `seal/record.go:263–269` and `crates/me-cli/src/seal/mod.rs:297–339`).
    - **Public Data Hash (`seal/pubhash.go:28–80`):** 128-bit (`[16]byte`) digest `SHA256("MNEMBLOB/pub/v1" || 0x00 || byte(sealed) || byte(len(records)) || strings.Join(records, "\n"))[:16]`, formatted by `seal.FormatHash` as 8 space-separated 4-hex groups (`xxxx xxxx xxxx xxxx xxxx xxxx xxxx xxxx`).
- **Version / Pin:**
  - Format version `0x01`.
  - `seal/testdata/vectors.json` pinned to `mnemonic-engrave` commit `4df43d6` (SHA-256 `333ac47e7f61d031c995b85510565bfffd86cd1992f09b0230c1484fffd4d4bc`).
- **Validation Owner:** `seal` package (`seal/open.go`, `seal/record.go`) owns both container and strict record admission (`seal.Classification`) before returning `AdmittedPayload` to `gui/unlock_flow.go`.
- **Authoritative Specification:** `design/SPEC_encrypted_payload_delivery.md`.
- **Compatibility Evidence:** `seal/vectors_test.go`, `seal/open_test.go`, `crates/me-cli/tests/codex32_seam.rs`, `gui/unlock_flow_test.go`.
- **Lockstep Update Rules:**
  - Changing `MNEMBLOB` header layout, `MaxSectionLen`, `"MNEMBLOB/pub/v1"` hash preimage, or `seal::record` admission rules in `crates/me-cli/src/seal` requires regenerating `seal/testdata/vectors.json` (`ME_EMIT_VECTORS=1 cargo test -p mnemonic-engrave --lib seal::tests::emit_vectors`) and updating `seedhammer/seal/` in lockstep.

---

### 2.4 Constellation String Codecs (`md1`, `mk1`, `ms1`, `mt1`, `hashlock`)

#### 2.4.1 `md1` — Descriptor & Wallet Policy Mnemonic
- **Producer:** `descriptor-mnemonic` (`crates/md-codec`, CLI `md`), `me`, `mnemonic`, `mr` / `mr-gui`, or on-device `gui/composer_flow.go` / `gui/singlesig_engrave.go` / `gui/multisig_engrave.go`.
- **Consumer:** `codex32/mdmk.go` (`ValidMD`), `md/` (`Decode`, `Reassemble`, `ExpandWalletPolicyChunks`, `FormAwareIdChunks`, `DeriveAddress`), `gui/md1_gather.go`, `gui/wallet_policy.go`.
- **Format / Protocol:**
  - HRP `md`, separator `1`, 5-bit Bech32 payload (unchunked or prefixed by a **37-bit chunk header** `[version:4][chunked=1:1][csid:20][count-1:6][index:6]` in `md/chunk.go:44–75`), followed by a **13-char regular BCH(93,80,8) checksum only** (`mdRegularMaxLen = 93` data symbols = max 96 chars total; `POLYMOD_INIT = 0x23181b3`, NUMS residue target).
  - Supports wire version 4 (`fe` = `0`) and version 8 (`p` = `1`, Taproot script trees, `multi_a`, `sortedmulti_a`, timelocks, hashlocks).
  - `WalletPolicyId` (`md/walletpolicyid.go:11–30`) and `WalletDescriptorTemplateId` (`md/template_id.go:21–39`) return **16-byte (`[16]byte`, 128-bit)** SHA-256 prefixes over canonical binary bitstream preimages (`writeNode(tree)` + per-`@N` binary key records for `WalletPolicyId`; `useSitePath || writeNode(tree) || [UseSitePathOverrides-TLV]` for `WalletDescriptorTemplateId`); their 4-byte prefixes are `WalletPolicyIDStub` and `WalletDescriptorTemplateIdStub`.
- **Version / Pin:** `me-cli` pins `md-codec` `0.48.4` (`6cf6599a`); Go `seedhammer/md` vectors pinned at `5a0a4f41`, `430ea478`, and `b2c5d693` (see [SOURCE_MAP.md](SOURCE_MAP.md#5-constellation-go-codec-provenance--update-procedure)).
- **Validation Owner:** `codex32.ValidMD` (BCH checksum) + `md.Reassemble` / `md.Decode` (wire AST, key-table consistency, Taproot leaf versions, duplicate keys).
- **Authoritative Specification:** `descriptor-mnemonic` crate docs, `design/SPEC_liana_unspendable_internal_key.md`, and `design/SPEC_wallet_policy_composer.md`.
- **Compatibility Evidence:** `md/md_test.go`, `md/conformance_keyed_test.go`, `md/compose_vectors_pin_test.go`, `md/liana_test.go`, `oracle/live_test.go`.
- **Lockstep Update Rules & Known Divergences:**
  - Any wire opcode, canonical formatting, or `WalletPolicyId` bitstream change in `md-codec` requires re-vendoring `md/testdata/*.json` via `scripts/vendor-*.sh`.
  - **Known open divergences:** Go `md` does not yet implement pathless shared-origin decoding (`sh_wpkh`, `F-166`) or `md-codec 0.44.0` rendered-xpub header/origin agreement (see [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md); `F-212` elided-origin `WalletPolicyId` parity was closed on 2026-08-20).

#### 2.4.2 `mk1` — Extended Public Key Card Mnemonic
- **Producer:** `mnemonic-key` (`crates/mk-codec`, CLI `mk`), `me`, `mnemonic`, `mr` / `mr-gui`, or on-device `deriveXpubFlow` / `singleSigEngraveFlow`.
- **Consumer:** `codex32/mdmk.go` (`ValidMK`), `mk/` (`Decode`, `DecodeSingle`, `Assembler`, `DerivedChunkSetID`), `gui/mk1_inspect.go`, `gui/key_card_seating.go`.
- **Format / Protocol:**
  - HRP `mk`, separator `1`, string-layer header of **2 symbols** (single) or **8 symbols** (chunked: `[version=0][type=1][csid:4 syms (20b)][total-1:1 sym][index:1 sym]` in `mk/mk.go:38–92`), 5-bit payload encoding version bytes (`xpub`/`zpub`/`tpub`/etc.), master fingerprint (4 bytes), derivation path steps, 33-byte compressed pubkey, 32-byte chaincode, optional policy/template ID stub, and regular (13-char) or long (15-char) BCH checksum.
  - Derived chunk set ID: `mk.DerivedChunkSetID(card)` (`mk/encode.go:64, 356`) computes the **top 20 bits of `SHA-256(encodeBytecode(card))`**.
- **Version / Pin:** `mk-codec` `0.5` (`mk/testdata/csid_ext_v0.1.json`).
- **Validation Owner:** `codex32.ValidMK` + `mk.Decode` / `mk.Assembler` + `gui/bundle.go:showBundleCSIDMismatchNotices`.
- **Authoritative Specification:** `mnemonic-key` (`crates/mk-codec`).
- **Compatibility Evidence:** `mk/mk_test.go`, `mk/chunk_set_id_parity_test.go`, `oracle/live_test.go`.
- **Lockstep Update Rules:** Wire bytecode or `DerivedChunkSetID` changes in `mk-codec` must be ported to `seedhammer/mk/` with updated `mk/testdata/` vectors.

#### 2.4.3 `ms1` — BIP-93 `codex32` Shares & `m`-Format Binary Secrets (`entr` & `hash`)
- **Producer:** `mnemonic-secret` (`crates/ms-codec`, CLI `ms`), `me`, `mnemonic`, `mr` / `mr-gui`, or on-device `codex32` minting / `composerHashlockPlatesFlow`.
- **Consumer:** `codex32/` (`New`, `Interpolate`, `DecodeMS1`, `EncodeMS1`, `EncodeMS1Preimage`, `DecodeMS1Preimage`, `IsPreimage`, `IsPreimageKind`, `IsPreimagePlate`, `Correct`), `hashlock/`, `gui/codex32_polish.go`, `gui/ms1_decode.go`, `gui/composer_hashlock_plates.go`.
- **Format / Protocol (`crates/ms-codec/src/{consts,payload,envelope}.rs`, `codex32/{codex32,mspayload,msencode}.go`):**
  - **Parser vs. plate-length boundary (`seal/record.go:94–108`):** `codex32.New` parses BIP-93 regular-checksum strings (`48..93` chars, 13-char checksum) and long-checksum strings (`125..127` chars, 15-char checksum), whereas the engravable plate limit (`MaxEngraveableMs1Len`) is **90 chars** (so all long-checksum strings exceed the 90-char plate limit and are rejected by `seal`/`sysw` admission).
  - **Unshared singles (`k = 0`):** Always carry threshold `'0'` and share index **`'s'`** (`codex32/codex32.go:169–170` rejects any threshold-`0` string whose share index is not `'s'`).
  - **`m`-format payload prefix dispatch (`Seed()[0]` on unshared singles):**
    - **`0x00` (`Entr`, 4-char ID `'entr'`, share index `'s'`):** `[0x00][entropy]` where `entropy` is **16, 20, 24, 28, or 32 bytes** of **BIP-39 entropy** (12/15/18/21/24 words, yielding regular-checksum string lengths of **50, 56, 62, 69, or 75 chars**).
    - **`0x02` (`Mnem`, 4-char ID `'entr'`, share index `'s'`):** `[0x02][language: 1 byte (0..9)][entropy]` where `language` indexes `MSLanguageNames` (`0` = English) and `entropy` is **16, 20, 24, 28, or 32 bytes** of **BIP-39 entropy** (`codex32/mspayload.go:21–60`, `codex32/msencode.go:17–38`). Both `Entr` and `Mnem` singles use the 4-char ID **`entr`** (never `mnem`).
    - **`0x03` (`Preimage`, 4-char ID `'hash'`, share index `'s'`):** `[0x03][preimage: 32 bytes]` — exactly **33 payload bytes** (**75 characters** with regular checksum; `codex32/msencode.go:61–69`, `codex32/mspayload.go:135–179`). There is **no** hash-function-ID byte inside the preimage payload; hash algorithm selection (`hash160`, `sha256`, `ripemd160`, `hash256`) belongs to `hash:<algo>:<digest-hex>` composer records and the policy AST.
  - **K-of-N shares (`2 <= k <= 9`):** Carry Shamir secret-sharing evaluation points; payload prefix bytes (`0x00`/`0x02`/`0x03`) are inspected only after polynomial interpolation (`codex32.Interpolate`).
- **Version / Pin:** `ms-codec` `0.10.0` (`e5dff6f9c2b7ae7dc02a4a137e7322abab32211e` in `backup/testdata/ms1_seedqr_vectors.provenance.json`; `61904c5576d8ab280fb5587ac23db72531be950e` in `hashlock/testdata/hashlock-v0.8.provenance.json`).
- **Validation Owner:** `codex32.New` (BIP-93 regular/long polymod), `codex32.DecodeMS1`, `codex32.DecodeMS1Preimage`, `codex32.IsPreimage`, `codex32.IsPreimagePlate`.
- **Authoritative Specification:** `mnemonic-secret` (`crates/ms-codec`, `design/SPEC_ms_v0_1.md`, `design/SPEC_ms_hashlock.md`) and `design/SPEC_wallet_policy_composer.md`.
- **Compatibility Evidence:** `codex32/codex32_test.go`, `codex32/mspayload_test.go`, `codex32/msencode_test.go`, `sysw/codex32_seam_test.go`, `hashlock/hashlock_test.go`, `backup/ms1_seedqr_test.go`, `oracle/live_test.go`.
- **Lockstep Update Rules:** Any new `m`-payload prefix byte or language selector in `ms-codec` must be added to `codex32/mspayload.go`, `codex32/msencode.go`, and `hashlock/hashlock.go` with updated vector files.

#### 2.4.4 `mt1` & `tx:` — Transaction Backups
- **Producer:** `mnemonic-transaction` (`crates/mt-codec`, CLI `mt`) and `me`. (Normative specification `design/SPEC_mt_v0_1.md` and independent vector generator `scripts/gen-mt1-vectors.py` live in `mnemonic-engrave`.)
- **Consumer:** `mt/` (`ParseHeader`, `Reassemble`, `Decode`, `ParseTx`), `txqr/` (`EncodeSet`), `gui/transaction.go`.
- **Format / Protocol:**
  - `mt1` (`mt/mt.go:39–178`, `codex32/mtdata.go:19–42`): HRP `mt`, separator `1`, **11-symbol (55-bit) chunk header** (`version:5 (=1, 'p') || chunk_set_id:20 || total_chunks-1:15 || chunk_index:15`), 5-bit payload carrying raw transaction chunk bytes (no flags byte or `u24` length prefix), and a **13-symbol regular BCH(93,80,8) checksum only** (no long code; max 96 chars total; `mdmkPolymodInitLo = 0x23181b3`, NUMS target `0x0001a2fc877f9528d7c1` = top 65 bits of `SHA-256("shibbolethnumstransaction")`).
  - `Tx.ChunkSetID()` (`mt/mt.go:162`): top 20 bits (first 5 hex characters) of `TxidDisplay`.
  - `txqr` (`txqr/txqr.go`): `txqr.EncodeSet(data, k, level)` encodes raw transaction bytes in **QR byte mode** (`k = 1` single symbol, or `2 <= k <= 16` ISO/IEC 18004 **Structured Append** 20-bit headers + byte-mode payloads).
- **Version / Pin:** `mt-codec` `0.1.0` (`72b79b8762ec208b6143afade8c4b66a551bd29d`).
- **Validation Owner & Confirmation / Signed-Input Semantics (`mt/mt.go:39–155`, `gui/transaction.go:432–484`):**
  - Header wire version is strictly **`mtVersion = 1`** (`mt/mt.go:40, 96–98`); version `0` (or any version $\neq 1$) is rejected by `mt.ParseHeader`.
  - A set of valid version-1 `mt1` chunks is marked **unconfirmed** (`confirmed: false` in `gui/transaction.go:432–448`) when full set reassembly/decoding (`mt.Decode(set)`) fails (e.g., incomplete chunk set, uneven chunk lengths, invalid transaction wire bytes, `chunk_set_id` / `txid` binding mismatch, or `ErrUnsignedInputs`).
  - For both `mt.Decode` and `tx:` records (`gui/transaction.go:471–484`), **`Tx.EveryInputSigned`** (with per-input indices in **`Tx.UnsignedInputs`**, `mt/mt.go:136–155` and `crates/me-cli/src/sysw/tx.rs:20–24`) checks a **structural wire property**: every input carries a non-empty `scriptSig` or at least one witness item. It does **not** verify cryptographic signatures, evaluate scripts, or check blockchain confirmation. Unconfirmed `mt1` sets and unsigned `tx:` records remain engraveable with a mandatory substituted warning legend (`subst`).
- **Authoritative Specification:** `design/SPEC_mt_v0_1.md`, `design/SPEC_engrave_transaction.md`, and `mnemonic-transaction/crates/mt-codec`.
- **Compatibility Evidence:** `mt/mt_test.go`, `txqr/txqr_test.go`, `gui/transaction_test.go`.
- **Lockstep Update Rules:** Any change to the 55-bit `mt1` header layout or BCH constants in `mt-codec` requires updating `seedhammer/mt/` and `seedhammer/codex32/mtdata.go`.

---

### 2.5 `me-preview` Go Sidecar Protocol & Submodule Boundary

- **Producer:** `me bundle --preview <DIR> [--png]` (`crates/me-cli/src/main.rs:47–71`, `crates/me-cli/src/preview.rs`).
- **Consumer:** `preview/main.go` (`me-preview` binary, resolved via `ME_PREVIEW_BIN` or adjacent to `me`).
- **Format / Protocol (`preview/main.go:25–121`, `crates/me-cli/src/preview.rs:111–195`):**
  - Version handshake: `me-preview --version` → stdout `me-preview <semver>` (must match `me`'s `CARGO_PKG_VERSION`).
  - Render invocation: `me-preview render --format <svg|png> --out <FILE|-> [--mode <text+qr|text|qr>]` with the **raw plate text piped on `stdin`**. `me-preview` writes the SVG/PNG (`0o600`, or stdout if `--out -`) and prints `mode <chosenMode>` on stdout (or stderr if `--out -`).
- **Version / Pin & Submodule Boundary:**
  - `preview/go.mod` uses `replace seedhammer.com => ../third_party/seedhammer`, which is pinned to **`bg002h/seedhammer` fork commit `7b6f2fbbdb909ef812ccc8c90768280c5fa7a4d4`** (24 commits behind `seedhammer` `origin/main` `abdcb182`).
  - `preview/params.go:18` still hardcodes `EngravingSpeed: 8 * mm` (copied before `cmd/controller/platform_sh2.go:223` lowered it to `4 * mm`). For latest fork-HEAD rendering, use `go run ./cmd/plateview -plate <name> -o <out.svg>` inside `seedhammer`.
- **Validation Owner:** `crates/me-cli/src/preview.rs` and `preview/render_test.go`.
- **Authoritative Specification:** `preview/main.go` and `crates/me-cli/src/preview.rs`.
- **Lockstep Update Rules:** Advancing `third_party/seedhammer` to `seedhammer` `origin/main` requires updating `preview/` golden SVG/PNG expectations if layout or font metrics changed (`design/FOLLOWUPS.md` `F-213`).

---

### 2.6 Firmware Build, RP2350 Image Signing & OTP Provisioning

- **Producer:**
  - Nix devshell in `mnemonic-engrave/flake.nix:27–40` (imported by `seedhammer/flake.nix:73–74`) providing TinyGo `0.41.1` + **patched `picotool` 2.3.1** (`nix/patches/picotool-2.3.1-seal-clear-fix.patch`, `F-701`).
  - `cmd/picosign` (`seedhammer/cmd/picosign/main.go`) + `scripts/sign-firmware.sh` (`mnemonic-engrave/scripts/`).
  - OTP provisioning & verification scripts in `mnemonic-engrave/scripts/`: `scripts/pico2-bootkey-rehearsal.sh` and `scripts/refugium-otp.sh`.
- **Consumer:** RP2350 mask ROM bootrom, `driver/otp/otp.go` + `driver/otp/otp_rp2350.go`, and `cmd/controller/platform_sh2.go` (`LockBoot`).
- **Format / Protocol:**
  - **UF2 family ID:** `0xe48bff59` (`rp2350-arm-s`, `uf2.FamilyRP2350ARMSigned`).
  - **`picotool 2.3.1` patch & SRAM clear entry (`nix/patches/picotool-2.3.1-seal-clear-fix.patch:11–55`, `scripts/lib/seal-check.sh:22`):**
    - Fixes two upstream `picotool 2.3.0`/`2.3.1` bugs when running `picotool seal --sign --clear` on UF2/BIN inputs: (1) passes `model` (from the image's `IMAGE_DEF`) instead of `in.get_model()` in `sign_guts_bin` (avoiding exit 248 `"unknown sram end"`), and (2) appends `bin` **after** the Clear/pin size words in `get_lm_hash_data` (`to_hash.insert(to_hash.end(), ...)`), matching the RP2350 bootrom hash order (`clear_size || pin_size || bin`).
    - Sealing (`picotool seal --sign --clear --quiet "$IN_UF2" "$OUT_UF2" "$DUMMY_PEM"`) emits a `PICOBIN_BLOCK` whose required load-map entry 0 is **`Clear 0x20000000->0x20082000`**, enforced by `seal_has_clear_entry` in `scripts/lib/seal-check.sh:22` and `scripts/sign-firmware.sh:195–199`.
  - **`cmd/picosign` & external signing sequence (`cmd/picosign/main.go:26–170`, `picobin/picobin.go:300–347`, `scripts/sign-firmware.sh:105–200`):**
    - `cmd/picosign` is an inspection/embedding utility (`picosign hash <img.uf2>`, `picosign sign [-clear] [-pubkey <33B-compressed-hex>] [-sig <sig-hex>] [-sigfmt raw|der] <img.uf2>`, `picosign extract <img.uf2>`); it does not hold private keys or generate signatures.
    - Because `picobin.Image.HashData` (`picobin/picobin.go:343–344`) hashes the load-map ranges **and** the defined portion of the `PICOBIN` block **including the embedded 64-byte public key** (`X || Y`), signing must: (1) embed the real public key with a zero placeholder signature (`picosign sign -pubkey "$PUB" -sig "$ZEROSIG" "$IMG"`), (2) extract the 32-byte digest (`picosign hash "$IMG" > digest.bin`), (3) sign `digest.bin` externally (`openssl pkeyutl -sign`), (4) embed the real signature (`picosign sign -pubkey "$PUB" -sig "$SIG_HEX" -sigfmt der "$IMG"`), and (5) verify that `picosign hash` is unchanged, `picosign extract` matches `r || s`, `picotool info -a` reports `signature: verified`, and `seal_has_clear_entry` confirms `load map entry 0: Clear 0x20000000->0x20082000`.
  - **RP2350 OTP registers & boot-key hash (`driver/otp/otp.go:14–30, 93–170, 279–310`, `scripts/refugium-otp.sh:395–408, 583–595`, `scripts/lib/otp-read.sh:464–473`):**
    - **`BOOTKEY0..3` (`BOOTKEY0_0 = 0x080`, slot $i \in \{0..3\}$ at `0x080 + i*16`, 16 ECC rows per slot):** Stores the 32-byte **SHA-256 digest over the 64-byte uncompressed `secp256k1` public key coordinates `X || Y` in Big-Endian byte order** (32-byte big-endian `X` followed by 32-byte big-endian `Y`, excluding the `0x04` uncompressed prefix, per `cmd/picosign/main.go:142–143` `FillBytes` and `scripts/lib/otp-read.sh:464–473`). In OTP storage, the 32-byte digest is written across 16 two-byte ECC rows (`scripts/pico2-bootkey-rehearsal.sh:227–234`).
    - **`CRIT1` (`0x040`, raw **8× redundancy** across rows `0x040..0x047`):** `driver/otp.EnableSecureBoot()` (`otp.go:93–95`) sets bit 0 (`0b1` = `SECURE_BOOT_ENABLE`) via `writeOrRow(CRIT1, 8, 0b1)`, and `IsSecureBootEnabled()` reads `readOrRow(CRIT1, 8)&0b1 != 0`. (Host audit script `scripts/refugium-otp.sh:397–398` also reads `CRIT0` at `0x038` with 8× redundancy and checks profile-specific `CRIT1` criteria.)
    - **`BOOT_FLAGS1` (`0x04b`, raw **3× redundancy** across rows `0x04b..0x04d`):** Bits `0..3` (`0b1 << slot`) are **`KEY_VALID`** (set by `WriteBootKey` via `writeOrRow(BOOT_FLAGS1, 3, 0b1<<slot)`); bits `8..11` (`(0b1 << slot) << 8`, i.e., `(bf1 >> 8) & 0xf`) are **`KEY_INVALID`** (key revocation). `IsBootKeyValid(slot)` (`otp.go:289–296`) requires `bf1&(1<<slot) != 0 && bf1&((1<<slot)<<8) == 0`. (`BOOT_FLAGS0` is a separate register at `0x048` with 3× redundancy.)
    - **Page locks (`scripts/refugium-otp.sh:405–408, 583–589`):** `PAGE1_LOCK0` (`0xf82`), `PAGE1_LOCK1` (`0xf83`), `PAGE2_LOCK0` (`0xf84`), and `PAGE2_LOCK1` (`0xf85`). In `scripts/refugium-otp.sh:583–588`, the checked profile values are `LOCK0 = 0` and `LOCK1 = 0x040404`.
- **Authoritative Specification:** `seedhammer/docs/custom-firmware.md` ([GitHub baseline](https://github.com/bg002h/seedhammer/blob/abdcb1825ed3bde5e01ff0e4da33f76f08f604c2/docs/custom-firmware.md)), [PICOTOOL_PIN.md](../PICOTOOL_PIN.md), and [FIRMWARE-QUICKSTART.txt](../../FIRMWARE-QUICKSTART.txt).
- **Compatibility Evidence:** `picobin/picobin_test.go`, `uf2/uf2_test.go`, `cmd/picosign/main.go`, `scripts/test/seal-clear-test.sh`, `scripts/refugium-elf-check.sh`, and physical Pico 2 rehearsals recorded in [PICOTOOL_PIN.md](../PICOTOOL_PIN.md).
- **Lockstep Update Rules:** Always use the devshell's patched `picotool` (`nix/patches/picotool-2.3.1-seal-clear-fix.patch`) and verify `load map entry 0: Clear 0x20000000->0x20082000` via `scripts/lib/seal-check.sh` when sealing UF2/BIN firmware images.
