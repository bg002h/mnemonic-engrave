# Encrypted Payload Delivery — Plan A (Rust host, `me seal`) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `me seal` and `me hash` to the `mnemonic-engrave` CLI: encrypt a constellation payload under a host-generated 12-word BIP-39 passphrase and emit a `data`-family UF2 the operator loads into SeedHammer II flash at `0x10E00000`.

**Architecture:** A `seal/` module implementing `SPEC_encrypted_payload_delivery.md` — a 52-byte authenticated header, an optional cleartext **public section** carried in the AEAD's Associated Data, an optional **encrypted section**, PBKDF2-HMAC-SHA256 → AES-256-GCM, and a fixed public-data hash the operator compares out-of-band. All randomness is host-side; the device only decrypts.

**Tech Stack:** Rust 2021, `aes-gcm`, `pbkdf2`, `sha2`, `bip39`, `rand`, `zeroize`, `md-codec 0.42`, `mk-codec`, `ms-codec`.

**Spec:** `design/SPEC_encrypted_payload_delivery.md` @ `0d19c27` (R0 GREEN). Section references (§6.2, §6.6, …) are to that document. **Read §2.2, §6, §7, §9 and §11 before starting.**

**This plan supersedes the version at `a00d4be`**, which described a 48-byte header, a `payload_kind` byte that no longer exists, and three vectors instead of six. Do not consult it.

## Global Constraints

- **Rust-primary rule.** This crate is the normative implementation. The Go firmware port is downstream and binds to the vectors produced here. Never change wire behaviour to match Go.
- **No caller-supplied salt or IV in any public API** (§7.2). One fresh salt per seal is what makes GCM's nonce rule structurally unbreakable. The deterministic seam is `pub(crate)` and unit-tested from inside the module — never exported, never a CLI flag.
- **No `--addr` flag** (§9). `0x10E00000` is normative; past `0x11000000` a write wraps to `0x10000000` and destroys the signed firmware.
- **The CLI MUST NOT accept a user-supplied passphrase** (§8). Total strength is passphrase entropy + ~20 KDF bits.
- **The passphrase goes to stderr, and `--out` is required** (§9). `me seal … > payload.uf2` must not be able to write the twelve words into the file they decrypt.
- **Records are canonical and lowercase** (§6.4): no interior whitespace, no `-`, no uppercase.
- **Public-section records must reassemble and decode as a CARD SET** (§6.3), not per record.
- Constants: `MAGIC = b"MNEMBLOB"`, `VERSION = 0x01`, `HEADER_LEN = 52`, `SALT_LEN = 16`, `IV_LEN = 12`, `TAG_LEN = 16`, `iterations ∈ [100_000, 2_000_000]` default **300_000**, `pub_len`/`ct_len` each `≤ 8191`, `record_count ∈ [1, 24]` across both sections.
- Multi-byte integers are **big-endian**.
- **MSRV is 1.85.0** (`.github/workflows/*.yml`), but a local toolchain may be
  much newer — 1.97.0-nightly at the time of writing. A newer API or language
  feature compiles locally and fails CI. `div_ceil` (1.73) is safe; check
  anything newer before using it.
- Secrets wiped with `zeroize` on every path.
- `cargo fmt` and `cargo clippy -p mnemonic-engrave -- -D warnings` before each commit.

---

## File Structure

| File | Responsibility |
| --- | --- |
| `src/seal/mod.rs` | Public API (`seal`, `Payload`, `Sealed`, `SealError`), the `pub(crate)` deterministic seam, the six canonical vectors |
| `src/seal/wire.rs` | `Header`, encode/decode, every §6.2 bound, both shapes |
| `src/seal/crypto.rs` | `derive_key`, `seal_bytes`, `open_bytes` |
| `src/seal/passphrase.rs` | 12-word BIP-39 generation, §8.1 normalisation |
| `src/seal/record.rs` | Per-record validation, classification, card grouping, set decode |
| `src/seal/container.rs` | LF section encoding |
| `src/seal/pubhash.rs` | The §6.6 fixed public-data hash |
| `src/seal/uf2.rs` | `data`-family UF2 emission |
| `src/main.rs` (modify) | `Command::Seal`, `Command::Hash` |
| `src/lib.rs` (modify) | `pub mod seal;` |
| `src/validate.rs` (modify) | Extract `first_noncanonical` for reuse |
| `tests/seal_cli.rs` | End-to-end CLI behaviour |

---

### Task 1: Dependencies and the wire header

**Files:** modify `crates/me-cli/Cargo.toml`; create `src/seal/{mod.rs,wire.rs}`; modify `src/lib.rs`

**Interfaces:**
- Produces: `wire::{Header, WireError, HEADER_LEN, SALT_LEN, IV_LEN, TAG_LEN, MIN_ITERATIONS, MAX_ITERATIONS, MAX_SECTION_LEN, REGION_LEN}`; `Header::encode() -> [u8; 52]`; `Header::decode(&[u8]) -> Result<Header, WireError>`; `Header::is_sealed(&self) -> bool`.

- [ ] **Step 1: Bump md-codec and add dependencies**

In `crates/me-cli/Cargo.toml`, change `md-codec = "0.40"` to `md-codec = "0.42"` and add:

```toml
aes-gcm = "0.10"
pbkdf2 = { version = "0.12", default-features = false, features = ["hmac"] }
sha2 = "0.10"
bip39 = "2.2"
rand = "0.9"
ms-codec = "0.7"
```

**The md-codec bump is required, not optional.** §6.3's decode requirement cannot be met on 0.40: the vector records carry md1 wire version 9 and 0.40 expects 4 (`wire-format version mismatch: got 9, expected 4`). The existing converter never noticed because `validate()` runs only the version-agnostic BCH layer.

Run: `cargo test -p mnemonic-engrave`
Expected: **all existing tests still pass** (verified 2026-08-07: 103 tests, 0 failures under 0.42). If anything fails, stop — a fixture needs regenerating and that is a separate decision.

- [ ] **Step 2: Wire the module, then write the failing test**

**Do the wiring first.** An undeclared `.rs` file is not compiled, so `cargo test` would report `0 passed` and exit 0 — a green RED step, which is a false PASS in the TDD gate itself.

Create `crates/me-cli/src/seal/mod.rs`:

```rust
//! `me seal` — encrypt a constellation payload for delivery to SeedHammer II
//! flash. See design/SPEC_encrypted_payload_delivery.md.

pub mod wire;
```

Add `pub mod seal;` to `crates/me-cli/src/lib.rs` after line 8.

Now create `crates/me-cli/src/seal/wire.rs` with only this test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sealed() -> Header {
        Header {
            iterations: 100_000,
            salt: [0xbe; SALT_LEN],
            iv: [0xc0; IV_LEN],
            pub_len: 396,
            ct_len: 75,
        }
    }
    fn unsealed() -> Header {
        Header { iterations: 0, salt: [0; SALT_LEN], iv: [0; IV_LEN], pub_len: 396, ct_len: 0 }
    }

    #[test]
    fn both_shapes_round_trip() {
        for h in [sealed(), unsealed()] {
            let b = h.encode();
            assert_eq!(b.len(), HEADER_LEN);
            assert_eq!(&b[..8], b"MNEMBLOB");
            assert_eq!(Header::decode(&b).unwrap(), h);
        }
        assert!(sealed().is_sealed());
        assert!(!unsealed().is_sealed());
    }

    #[test]
    fn sealed_shape_sets_algorithm_ids_and_unsealed_zeroes_them() {
        assert_eq!(sealed().encode()[9..11], [0x01, 0x01]);
        assert_eq!(unsealed().encode()[9..11], [0x00, 0x00]);
    }

    /// Covers every §6.2 header bound. `reserved`, and the SEALED-shape
    /// `kdf_id`/`aead_id` checks, are exercised here and nowhere else —
    /// mutation-proved: deleting all three checks left the other wire tests
    /// green, because `rejects_nonzero_crypto_fields_when_unsealed` only visits
    /// offsets 9/10 in the UNSEALED branch.
    #[test]
    fn rejects_bad_magic_version_reserved_kdf_and_aead() {
        let cases: &[(usize, u8, &str)] = &[
            (0, b'X', "magic"), (8, 0x02, "version"), (11, 0x01, "reserved"),
            (9, 0x02, "kdf_id"), (10, 0x02, "aead_id"),
        ];
        for &(off, val, label) in cases {
            let mut b = sealed().encode();
            b[off] = val;
            assert!(Header::decode(&b).is_err(), "{label} = {val:#x} must be refused");
        }
        // Pin the variants so a mutation collapsing them into one is caught.
        let mut b = sealed().encode(); b[11] = 0x01;
        assert!(matches!(Header::decode(&b), Err(WireError::ReservedNotZero(1))));
        let mut b = sealed().encode(); b[9] = 0x02;
        assert!(matches!(Header::decode(&b), Err(WireError::UnknownKdf(2))));
        let mut b = sealed().encode(); b[10] = 0x02;
        assert!(matches!(Header::decode(&b), Err(WireError::UnknownAead(2))));
    }

    #[test]
    fn rejects_a_short_buffer() {
        assert!(matches!(Header::decode(&[0u8; 51]), Err(WireError::TooShort(51))));
    }

    #[test]
    fn rejects_out_of_range_iterations_when_sealed() {
        for bad in [0u32, 99_999, 2_000_001, u32::MAX] {
            let mut h = sealed();
            h.iterations = bad;
            assert!(
                matches!(Header::decode(&h.encode()), Err(WireError::Iterations(_))),
                "iterations {bad} must be refused"
            );
        }
    }

    #[test]
    fn rejects_out_of_range_lengths() {
        for bad in [8192u32, 0xFFFF_FFF0, u32::MAX] {
            let mut h = sealed();
            h.ct_len = bad;
            assert!(matches!(Header::decode(&h.encode()), Err(WireError::CtLen(_))), "ct_len {bad}");
            let mut h = sealed();
            h.pub_len = bad;
            assert!(matches!(Header::decode(&h.encode()), Err(WireError::PubLen(_))), "pub_len {bad}");
        }
    }

    #[test]
    fn rejects_an_empty_payload() {
        let mut h = unsealed();
        h.pub_len = 0;
        assert!(matches!(Header::decode(&h.encode()), Err(WireError::Empty)));
    }

    /// §6.2: with ct_len == 0 the crypto fields MUST be zero. Junk there would
    /// let an attacker stage a downgrade a later version might honour.
    #[test]
    fn rejects_nonzero_crypto_fields_when_unsealed() {
        let base = unsealed().encode();
        for (off, val, label) in [
            (9usize, 0x01u8, "kdf_id"),
            (10, 0x01, "aead_id"),
            (15, 0x01, "iterations"),
            (16, 0x01, "salt"),
            (32, 0x01, "iv"),
        ] {
            let mut b = base;
            b[off] = val;
            assert!(
                matches!(Header::decode(&b), Err(WireError::UnsealedFieldNotZero(_))),
                "{label} must be zero when ct_len == 0"
            );
        }
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::wire`
Expected: FAIL — `cannot find type Header in this scope`.

- [ ] **Step 4: Implement**

Prepend to `crates/me-cli/src/seal/wire.rs`:

```rust
//! The §6 wire header: 52 bytes, big-endian, and — together with the public
//! section — the AEAD's Associated Data.
//!
//! Parsed BEFORE authentication (it carries the salt and iteration count), so
//! it is hostile input by construction and every field is bound-checked here.

pub const MAGIC: [u8; 8] = *b"MNEMBLOB";
pub const VERSION: u8 = 0x01;
pub const KDF_PBKDF2_SHA256: u8 = 0x01;
pub const AEAD_AES256GCM: u8 = 0x01;

pub const HEADER_LEN: usize = 52;
pub const SALT_LEN: usize = 16;
pub const IV_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

pub const MIN_ITERATIONS: u32 = 100_000;
pub const MAX_ITERATIONS: u32 = 2_000_000;
/// One below the device's 8 KiB scan buffer: `gui/scan.go` flags overflow when
/// the buffer is exactly full.
pub const MAX_SECTION_LEN: u32 = 8191;
pub const REGION_LEN: u64 = 65_536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// 0 when nothing is encrypted.
    pub iterations: u32,
    pub salt: [u8; SALT_LEN],
    pub iv: [u8; IV_LEN],
    pub pub_len: u32,
    /// 0 when nothing is encrypted — and then there is no tag either.
    pub ct_len: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WireError {
    TooShort(usize),
    BadMagic,
    UnknownVersion(u8),
    UnknownKdf(u8),
    UnknownAead(u8),
    ReservedNotZero(u8),
    Iterations(u32),
    PubLen(u32),
    CtLen(u32),
    Empty,
    UnsealedFieldNotZero(&'static str),
    TooLarge(u64),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::TooShort(n) => write!(f, "blob too short: {n} bytes, need at least 52"),
            WireError::BadMagic => write!(f, "not a sealed payload (bad magic)"),
            WireError::UnknownVersion(v) => write!(f, "unsupported format version {v}"),
            WireError::UnknownKdf(k) => write!(f, "unsupported kdf id {k}"),
            WireError::UnknownAead(a) => write!(f, "unsupported aead id {a}"),
            WireError::ReservedNotZero(b) => write!(f, "reserved byte must be 0, got {b}"),
            WireError::Iterations(n) => write!(
                f, "iteration count {n} out of range [{MIN_ITERATIONS}, {MAX_ITERATIONS}]"),
            WireError::PubLen(n) => write!(f, "public section length {n} exceeds {MAX_SECTION_LEN}"),
            WireError::CtLen(n) => write!(f, "ciphertext length {n} exceeds {MAX_SECTION_LEN}"),
            WireError::Empty => write!(f, "payload is empty (pub_len and ct_len are both 0)"),
            WireError::UnsealedFieldNotZero(fld) => write!(
                f, "{fld} must be zero when nothing is encrypted"),
            WireError::TooLarge(n) => write!(f, "blob is {n} bytes; the region is {REGION_LEN}"),
        }
    }
}
impl std::error::Error for WireError {}

impl Header {
    pub fn is_sealed(&self) -> bool {
        self.ct_len > 0
    }

    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let s = self.is_sealed();
        let mut out = [0u8; HEADER_LEN];
        out[..8].copy_from_slice(&MAGIC);
        out[8] = VERSION;
        out[9] = if s { KDF_PBKDF2_SHA256 } else { 0 };
        out[10] = if s { AEAD_AES256GCM } else { 0 };
        out[11] = 0; // reserved
        out[12..16].copy_from_slice(&self.iterations.to_be_bytes());
        out[16..32].copy_from_slice(&self.salt);
        out[32..44].copy_from_slice(&self.iv);
        out[44..48].copy_from_slice(&self.pub_len.to_be_bytes());
        out[48..52].copy_from_slice(&self.ct_len.to_be_bytes());
        out
    }

    /// Parse and bound-check. Every check runs BEFORE any KDF work: the firmware
    /// has no active watchdog, so an unbounded iteration count is a hang.
    pub fn decode(buf: &[u8]) -> Result<Self, WireError> {
        if buf.len() < HEADER_LEN {
            return Err(WireError::TooShort(buf.len()));
        }
        if buf[..8] != MAGIC {
            return Err(WireError::BadMagic);
        }
        if buf[8] != VERSION {
            return Err(WireError::UnknownVersion(buf[8]));
        }
        if buf[11] != 0 {
            return Err(WireError::ReservedNotZero(buf[11]));
        }

        let iterations = u32::from_be_bytes(buf[12..16].try_into().unwrap());
        let pub_len = u32::from_be_bytes(buf[44..48].try_into().unwrap());
        let ct_len = u32::from_be_bytes(buf[48..52].try_into().unwrap());

        if pub_len > MAX_SECTION_LEN {
            return Err(WireError::PubLen(pub_len));
        }
        if ct_len > MAX_SECTION_LEN {
            return Err(WireError::CtLen(ct_len));
        }
        if pub_len == 0 && ct_len == 0 {
            return Err(WireError::Empty);
        }

        if ct_len > 0 {
            if buf[9] != KDF_PBKDF2_SHA256 {
                return Err(WireError::UnknownKdf(buf[9]));
            }
            if buf[10] != AEAD_AES256GCM {
                return Err(WireError::UnknownAead(buf[10]));
            }
            if !(MIN_ITERATIONS..=MAX_ITERATIONS).contains(&iterations) {
                return Err(WireError::Iterations(iterations));
            }
        } else {
            // §6.2's anti-downgrade-staging rule.
            if buf[9] != 0 {
                return Err(WireError::UnsealedFieldNotZero("kdf_id"));
            }
            if buf[10] != 0 {
                return Err(WireError::UnsealedFieldNotZero("aead_id"));
            }
            if iterations != 0 {
                return Err(WireError::UnsealedFieldNotZero("iterations"));
            }
            if buf[16..32].iter().any(|&b| b != 0) {
                return Err(WireError::UnsealedFieldNotZero("salt"));
            }
            if buf[32..44].iter().any(|&b| b != 0) {
                return Err(WireError::UnsealedFieldNotZero("iv"));
            }
        }

        // u64 deliberately: 32-bit arithmetic wraps for lengths near 2^32 and
        // would pass a <= 65536 test. The section caps above already protect a
        // conforming implementation; this must not be relied on alone.
        let tag = if ct_len > 0 { TAG_LEN as u64 } else { 0 };
        let total = HEADER_LEN as u64 + pub_len as u64 + ct_len as u64 + tag;
        if total > REGION_LEN {
            return Err(WireError::TooLarge(total));
        }

        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&buf[16..32]);
        let mut iv = [0u8; IV_LEN];
        iv.copy_from_slice(&buf[32..44]);
        Ok(Header { iterations, salt, iv, pub_len, ct_len })
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib seal::wire`
Expected: 8 passed.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/Cargo.toml Cargo.lock crates/me-cli/src/seal/ crates/me-cli/src/lib.rs
git commit -m "seal: 52-byte wire header, both shapes, bounds before any KDF work"
```

---

### Task 2: KDF and AEAD

**Files:** create `src/seal/crypto.rs`; modify `src/seal/mod.rs`

**Interfaces:**
- Consumes: `wire::{IV_LEN, SALT_LEN, TAG_LEN}` — those three only; importing more trips `-D warnings`.
- Produces: `crypto::derive_key(&str, &[u8; 16], u32) -> Zeroizing<[u8; 32]>`; `crypto::seal_bytes(&[u8;32], &[u8;12], &[u8], &[u8]) -> Result<Vec<u8>, CryptoError>`; `crypto::open_bytes(...) -> Result<Zeroizing<Vec<u8>>, CryptoError>`.

- [ ] **Step 1: Wire the module, then write the failing test**

Add `pub mod crypto;` to `src/seal/mod.rs` **first**. Then create `src/seal/crypto.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const PASS: &str = "beef beef beef beef beef beef beef beef beef beef beef beef";
    fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }

    /// Vector A's derived key (SPEC §11.4). Both implementations bind to it.
    #[test]
    fn derives_the_pinned_key() {
        let salt: [u8; 16] = [0xbe, 0xef].repeat(8).try_into().unwrap();
        assert_eq!(
            hex(&*derive_key(PASS, &salt, 100_000)),
            "615ad9b781b1ad6105d9dffb135d1bf17ebab286c560f26912ee815836e7ad1e"
        );
    }

    /// Vector B exists solely to catch a hardcoded iteration count.
    #[test]
    fn iteration_count_changes_the_key() {
        let salt: [u8; 16] = [0xbe, 0xef].repeat(8).try_into().unwrap();
        assert_eq!(
            hex(&*derive_key(PASS, &salt, 100_001)),
            "003800ae6cec47cd4b34bb264c6bbb1156d806516ad1ab88391e479d14d8776f"
        );
    }

    #[test]
    fn seal_open_round_trips() {
        let sealed = seal_bytes(&[7u8; 32], &[9u8; 12], b"aad", b"plaintext").unwrap();
        assert_eq!(sealed.len(), 9 + TAG_LEN);
        assert_eq!(&*open_bytes(&[7u8; 32], &[9u8; 12], b"aad", &sealed).unwrap(), b"plaintext");
    }

    #[test]
    fn open_fails_on_tampered_aad() {
        let sealed = seal_bytes(&[7u8; 32], &[9u8; 12], b"aad-one", b"pt").unwrap();
        assert!(open_bytes(&[7u8; 32], &[9u8; 12], b"aad-two", &sealed).is_err());
    }

    #[test]
    fn open_fails_on_flipped_ciphertext_byte() {
        let mut sealed = seal_bytes(&[7u8; 32], &[9u8; 12], b"aad", b"pt").unwrap();
        sealed[0] ^= 0x01;
        assert!(open_bytes(&[7u8; 32], &[9u8; 12], b"aad", &sealed).is_err());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::crypto`
Expected: FAIL — `cannot find function derive_key`.

- [ ] **Step 3: Implement**

Prepend to `src/seal/crypto.rs`:

```rust
//! PBKDF2-HMAC-SHA256 → AES-256-GCM (§7). Both are already linked into the
//! SeedHammer firmware, which is why they were chosen over scrypt/Argon2 —
//! neither of those fits its own standard's recommended memory on an RP2350.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use sha2::Sha256;
use zeroize::Zeroizing;

use super::wire::{IV_LEN, SALT_LEN, TAG_LEN};

#[derive(Debug, PartialEq, Eq)]
pub enum CryptoError {
    /// Tag mismatch or altered ciphertext. Fail closed: no plaintext is ever
    /// returned on this path.
    Authentication,
    TooShort,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::Authentication => write!(
                f, "wrong passphrase, or this payload has been altered"),
            CryptoError::TooShort => write!(f, "sealed payload too short"),
        }
    }
}
impl std::error::Error for CryptoError {}

/// `iterations` always comes from the header — never a constant, or vector B fails.
pub fn derive_key(passphrase: &str, salt: &[u8; SALT_LEN], iterations: u32)
    -> Zeroizing<[u8; 32]>
{
    let mut key = Zeroizing::new([0u8; 32]);
    pbkdf2::pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, iterations, &mut *key);
    key
}

/// Returns `ciphertext || tag`.
pub fn seal_bytes(key: &[u8; 32], iv: &[u8; IV_LEN], aad: &[u8], plaintext: &[u8])
    -> Result<Vec<u8>, CryptoError>
{
    Aes256Gcm::new(key.into())
        .encrypt(Nonce::from_slice(iv), Payload { msg: plaintext, aad })
        .map_err(|_| CryptoError::Authentication)
}

/// Verify then decrypt. `aes-gcm` returns an error without releasing plaintext
/// on tag mismatch, which is what makes it safe to parse the §6.4 container out
/// of the result.
pub fn open_bytes(key: &[u8; 32], iv: &[u8; IV_LEN], aad: &[u8], sealed: &[u8])
    -> Result<Zeroizing<Vec<u8>>, CryptoError>
{
    if sealed.len() < TAG_LEN {
        return Err(CryptoError::TooShort);
    }
    Aes256Gcm::new(key.into())
        .decrypt(Nonce::from_slice(iv), Payload { msg: sealed, aad })
        .map(Zeroizing::new)
        .map_err(|_| CryptoError::Authentication)
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p mnemonic-engrave --lib seal::crypto`
Expected: 5 passed. **If `derives_the_pinned_key` fails, STOP** — the KDF inputs disagree with the spec and every downstream vector is wrong.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/
git commit -m "seal: PBKDF2-HMAC-SHA256 + AES-256-GCM, pinned to vector A's key"
```

---

### Task 3: Passphrase generation

**Files:** create `src/seal/passphrase.rs`; modify `src/seal/mod.rs`

**Interfaces:** produces `passphrase::{generate, normalise, is_valid}`.

- [ ] **Step 1: Wire the module, then write the failing test**

Add `pub mod passphrase;` to `src/seal/mod.rs` first. Then create `src/seal/passphrase.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_twelve_valid_lowercase_words() {
        let p = generate();
        assert_eq!(p.split(' ').count(), 12);
        assert!(is_valid(&p));
        assert_eq!(*p, p.to_lowercase());
        assert!(!p.starts_with(' ') && !p.ends_with(' '));
    }

    /// A frozen RNG here is as fatal as a frozen salt.
    #[test]
    fn two_generations_differ() {
        assert_ne!(*generate(), *generate());
    }

    /// beef x12 is checksum-valid — a 1-in-16 coincidence, and the canonical
    /// vector passphrase.
    #[test]
    fn accepts_the_beef_vector() {
        assert!(is_valid("beef beef beef beef beef beef beef beef beef beef beef beef"));
    }

    /// beef x11 + bacon is a valid-length mnemonic of real words differing in
    /// one position, and checksum-INVALID. A gate that passes it is broken.
    #[test]
    fn rejects_near_miss_and_invalid() {
        assert!(!is_valid("beef beef beef beef beef beef beef beef beef beef beef bacon"));
        assert!(!is_valid(&"abandon ".repeat(12).trim()));
        assert!(!is_valid("not even words"));
    }

    #[test]
    fn normalise_collapses_whitespace_and_case() {
        assert_eq!(*normalise("  BEEF   beef\tbeef  "), "beef beef beef");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::passphrase`
Expected: FAIL — `cannot find function generate`.

- [ ] **Step 3: Implement**

```rust
//! The decryption passphrase: a host-generated 12-word BIP-39 mnemonic, 128
//! bits (§8).
//!
//! GENERATED, never user-supplied. Total strength is passphrase entropy plus
//! the ~20 bits the KDF adds, and a human-chosen passphrase is worth 25–35 bits
//! — one rented GPU, minutes. `age` reached the same conclusion and generates
//! 10 words rather than letting the user pick.
//!
//! Used ONLY as a passphrase: never seed entropy, never derives a wallet.

use bip39::{Language, Mnemonic};
use rand::RngCore;
use zeroize::Zeroizing;

pub fn generate() -> Zeroizing<String> {
    let mut entropy = Zeroizing::new([0u8; 16]);
    rand::rng().fill_bytes(&mut *entropy);
    Zeroizing::new(
        Mnemonic::from_entropy_in(Language::English, &*entropy)
            .expect("16 bytes is always a valid 12-word entropy length")
            .to_string(),
    )
}

/// §8.1: lowercase, single-space separated, no leading or trailing space. Host
/// and device MUST produce byte-identical KDF input.
pub fn normalise(s: &str) -> Zeroizing<String> {
    Zeroizing::new(s.split_whitespace().map(|w| w.to_lowercase())
        .collect::<Vec<_>>().join(" "))
}

/// Checksum-valid English mnemonic? The device runs this before committing to a
/// ~31 s KDF, so a typo costs a second rather than half a minute.
pub fn is_valid(s: &str) -> bool {
    Mnemonic::parse_in(Language::English, &*normalise(s)).is_ok()
}
```

- [ ] **Step 4: Run tests** — `cargo test -p mnemonic-engrave --lib seal::passphrase`, expect 5 passed.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/
git commit -m "seal: generate the 12-word BIP-39 passphrase; never accept one"
```

---

### Task 4: Record validation, classification, and CARD-SET decode

**Files:** modify `src/validate.rs`; create `src/seal/record.rs`; modify `src/seal/mod.rs`

**Interfaces:**
- Consumes: `classify::{classify, Format}`, `validate::first_noncanonical`.
- Produces: `validate::first_noncanonical(&str) -> Option<(usize, char)>`; `record::{RecordKind, RecordError, validate_record, decode_public_set}`.

- [ ] **Step 1: Extract the canonical check, behaviour-preserving**

In `crates/me-cli/src/validate.rs`, add **above `validate`'s doc comment (before line 56)**:

```rust
/// First interior separator in a trimmed constellation string, if any.
/// Canonical = no `-` anywhere and no interior whitespace. Callers must have
/// trimmed, so any remaining whitespace is interior.
///
/// Shared by the NDEF converter (md1 only, historical) and by `seal`, which
/// applies it to md1/mk1/ms1 alike — a sealed record is engraved verbatim just
/// as a converted one is.
pub fn first_noncanonical(s: &str) -> Option<(usize, char)> {
    s.char_indices().find(|(_, c)| c.is_whitespace() || *c == '-')
}
```

Then replace the check at `validate.rs:74-77` with:

```rust
            if let Some((pos, ch)) = first_noncanonical(s) {
                return Err(ValidateError::MdNonCanonical { ch, pos });
            }
```

Run: `cargo test -p mnemonic-engrave --lib`
Expected: all existing tests still pass — this is a pure extraction.

**Use `--lib`, not `--lib validate`.** The latter matches only
`validate::tests::*` (4 tests), none of which reach the `Format::Md`
non-canonical branch this step rewires. The tests that actually cover it —
`refuses_noncanonical_md1_interior_dash` / `_space` / `_newline` and
`noncanonical_md1_error_names_char_and_byte_position` — live in `lib.rs`'s root
`tests` module and would be filtered out, so the step would report green having
exercised nothing it changed.

- [ ] **Step 2: Wire the module, then write the failing test**

Add `pub mod record;` to `src/seal/mod.rs` first. Then create `src/seal/record.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const MD1: [&str; 3] = [
        "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3",
        "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374",
        "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz",
    ];
    const MK1: [&str; 2] = [
        "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g",
        "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8",
    ];
    const MS1: &str = "ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw";

    #[test]
    fn classifies_each_record_kind() {
        assert_eq!(validate_record(MD1[0]).unwrap(), RecordKind::Md);
        assert_eq!(validate_record(MK1[0]).unwrap(), RecordKind::Mk);
        assert_eq!(validate_record(MS1).unwrap(), RecordKind::Ms);
    }

    /// THE round-3 Critical. `mnemonic bundle` prints --group-size 5 by default;
    /// codex32's inputChar has no mapping for 0x20, so the device classifies a
    /// spaced record as unknown. Refuse here — never strip, or the plate carries
    /// separators the BCH checksum never covered.
    #[test]
    fn refuses_space_grouped_and_hyphenated_records() {
        assert!(matches!(validate_record("md1fv9w jpqpqpm6"),
            Err(RecordError::NonCanonical { ch: ' ', .. })));
        assert!(matches!(validate_record("md1fv9w-jpqpqpm6"),
            Err(RecordError::NonCanonical { ch: '-', .. })));
    }

    /// §6.4: uppercase passes the BCH validators, so without this the same
    /// wallet has two spec-legal encodings and therefore two §6.6 hashes.
    #[test]
    fn refuses_uppercase_records() {
        assert!(matches!(validate_record(&MD1[0].to_uppercase()),
            Err(RecordError::NotLowercase(_))));
    }

    #[test]
    fn refuses_corrupt_and_unknown_records() {
        let mut bad = MD1[0].to_string();
        let last = bad.pop().unwrap();
        bad.push(if last == 'q' { 'p' } else { 'q' });
        assert!(validate_record(&bad).is_err());
        assert!(validate_record("xx1qqqq").is_err());
    }

    /// §6.3: DECODE is per CARD SET, not per record. Records are CHUNKS —
    /// verified against the real crates:
    ///   md1 single chunk → "chunk set incomplete: got 1 chunks, expected 3"
    ///   mk1 single chunk → "received 1 chunks, header declares total_chunks = 2"
    /// A per-record decode would reject every legitimate payload.
    #[test]
    fn decodes_a_complete_card_set() {
        let all: Vec<&str> = MD1.iter().chain(MK1.iter()).copied().collect();
        assert!(decode_public_set(&all).is_ok(), "the full md1+mk1 set must decode");
    }

    #[test]
    fn refuses_an_incomplete_card_set() {
        assert!(decode_public_set(&[MD1[0]]).is_err(), "one md1 chunk of three");
        assert!(decode_public_set(&[MK1[0]]).is_err(), "one mk1 chunk of two");
        assert!(decode_public_set(&MD1[..2]).is_err(), "two md1 chunks of three");
    }

    /// The §6.3 smuggling case: arbitrary bytes wrapped in a BCH-valid md1.
    /// `ValidMD` passes it; the decode must not.
    #[test]
    fn refuses_a_bch_valid_but_undecodable_record() {
        const SMUGGLED: &str =
            "md1qqqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0sdmjzeptm5fdk0";
        assert!(validate_record(SMUGGLED).is_ok(), "BCH layer accepts it — that is the point");
        assert!(decode_public_set(&[SMUGGLED]).is_err(), "decode must reject it");
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::record`
Expected: FAIL — `cannot find function validate_record`.

- [ ] **Step 4: Implement**

```rust
//! Per-record validation, and the §6.3 card-set decode for the public section.

use crate::classify::{classify, Format};
use crate::validate::first_noncanonical;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    /// md1 — wallet policy. Public.
    Md,
    /// mk1 — xpub + origin. Public.
    Mk,
    /// ms1 — the seed. Secret.
    Ms,
}

impl RecordKind {
    pub fn is_secret(self) -> bool {
        matches!(self, RecordKind::Ms)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecordError {
    NonCanonical { ch: char, pos: usize },
    NotLowercase(usize),
    Unclassifiable(String),
    Invalid(String),
    UndecodableSet(String),
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordError::NonCanonical { ch, pos } => write!(
                f,
                "non-canonical record: separator {ch:?} at byte {pos} — records must be \
                 unbroken. If this came from `mnemonic bundle`, re-run with --group-size 0: \
                 the default --group-size 5 emits a DISPLAY form the engraver cannot read."
            ),
            RecordError::NotLowercase(pos) => write!(
                f,
                "record has an uppercase character at byte {pos} — records must be lowercase, \
                 or the same wallet has two different public-data hashes (§6.4)"
            ),
            RecordError::Unclassifiable(e) => write!(f, "unrecognised record: {e}"),
            RecordError::Invalid(e) => write!(f, "invalid record: {e}"),
            RecordError::UndecodableSet(e) => write!(
                f,
                "public records do not form a decodable card set: {e} — a BCH-valid string is \
                 not proof of a real wallet card (§6.3)"
            ),
        }
    }
}
impl std::error::Error for RecordError {}

/// Validate one record: canonical, lowercase, correct BCH checksum. Reports what
/// it is. Does NOT decode — see `decode_public_set`.
pub fn validate_record(s: &str) -> Result<RecordKind, RecordError> {
    let s = s.trim();
    if let Some((pos, ch)) = first_noncanonical(s) {
        return Err(RecordError::NonCanonical { ch, pos });
    }
    if let Some(pos) = s.char_indices().find(|(_, c)| c.is_uppercase()).map(|(i, _)| i) {
        return Err(RecordError::NotLowercase(pos));
    }
    let fmt = classify(s).map_err(|e| RecordError::Unclassifiable(e.to_string()))?;
    match fmt {
        Format::Md => md_codec::codex32::unwrap_string(s)
            .map(|_| RecordKind::Md)
            .map_err(|e| RecordError::Invalid(e.to_string())),
        Format::Mk => {
            let d = mk_codec::string_layer::decode_string(s)
                .map_err(|e| RecordError::Invalid(e.to_string()))?;
            if d.corrections_applied != 0 {
                return Err(RecordError::Invalid(format!(
                    "not pristine: required {} BCH correction(s)", d.corrections_applied)));
            }
            Ok(RecordKind::Mk)
        }
        // ms_codec::decode, NOT decode_with_correction — a seed that needed
        // repair must be fixed at source, not engraved.
        Format::Ms => ms_codec::decode(s)
            .map(|_| RecordKind::Ms)
            .map_err(|e| RecordError::Invalid(e.to_string())),
    }
}

/// §6.3: every public record must belong to a card set that REASSEMBLES AND
/// DECODES. Records are chunks, so this is necessarily a whole-set operation —
/// a per-record decode rejects every legitimate payload.
///
/// Groups by HRP, then decodes each group:
///   md1 → `md_codec::reassemble(&set)`
///   mk1 → `mk_codec::decode(&set)`
pub fn decode_public_set(records: &[&str]) -> Result<(), RecordError> {
    let mut md: Vec<&str> = Vec::new();
    let mut mk: Vec<&str> = Vec::new();
    for r in records {
        match validate_record(r)? {
            RecordKind::Md => md.push(r),
            RecordKind::Mk => mk.push(r),
            // Guarded by the caller (§6.3 forbids a secret in the public
            // section); reaching here is a caller bug, not a bad payload.
            RecordKind::Ms => {
                return Err(RecordError::UndecodableSet(
                    "a secret record cannot be in the public section".into()))
            }
        }
    }
    if !md.is_empty() {
        md_codec::reassemble(&md)
            .map_err(|e| RecordError::UndecodableSet(format!("md1: {e}")))?;
    }
    if !mk.is_empty() {
        mk_codec::decode(&mk)
            .map_err(|e| RecordError::UndecodableSet(format!("mk1: {e}")))?;
    }
    Ok(())
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p mnemonic-engrave --lib seal::record && cargo test -p mnemonic-engrave --lib validate`
Expected: all pass, including the pre-existing `validate` tests.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/validate.rs crates/me-cli/src/seal/
git commit -m "seal: record validation plus the per-card-set decode §6.3 requires"
```

---

### Task 5: The §6.6 fixed public-data hash

**Files:** create `src/seal/pubhash.rs`; modify `src/seal/mod.rs`

**Interfaces:** produces `pubhash::{public_data_hash, format_hash}`.

- [ ] **Step 1: Wire the module, then write the failing test**

Add `pub mod pubhash;` to `src/seal/mod.rs` first. Then create `src/seal/pubhash.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn public() -> Vec<&'static str> {
        vec![
            "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g",
            "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8",
            "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3",
            "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374",
            "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz",
        ]
    }
    fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }

    /// §11.4 vectors D and E, asserted as LITERALS — not merely as differing.
    /// An agreement-only assertion is satisfied by any deterministic function of
    /// any subset of these bytes, because D and E share their public section.
    #[test]
    fn matches_the_pinned_literals() {
        assert_eq!(hex(&public_data_hash(&public(), true)),
                   "a26ed22bb747dfd0236706ad14c19679", "vector D (sealed)");
        assert_eq!(hex(&public_data_hash(&public(), false)),
                   "70f3e35aacf747dbc40f837691aa61e0", "vector E (unsealed)");
    }

    /// THE downgrade detector. An earlier draft required these to AGREE, which
    /// is exactly the blindness a ciphertext-strip needs.
    #[test]
    fn sealed_and_unsealed_differ() {
        assert_ne!(public_data_hash(&public(), true), public_data_hash(&public(), false));
    }

    /// Every byte must matter — this is what kills subset and off-by-one
    /// mutants, which the D-vs-E inequality cannot.
    #[test]
    fn every_byte_of_the_section_affects_the_hash() {
        let base = public_data_hash(&public(), false);
        // §11.4 requires the SECTION's first and last byte, not a record index.
        // Mutating record[0] and record[4] and popping each one's LAST char
        // never varies the section's true first byte, so a hash over
        // `input[1..]` would survive.
        for label in ["first byte of the section", "last byte of the section"] {
            let mut recs: Vec<String> = public().iter().map(|s| s.to_string()).collect();
            if label.starts_with("first") {
                let r = &mut recs[0];
                let c = r.remove(0);
                r.insert(0, if c == 'm' { 'n' } else { 'm' });
            } else {
                let r = recs.last_mut().unwrap();
                let c = r.pop().unwrap();
                r.push(if c == 'q' { 'p' } else { 'q' });
            }
            let refs: Vec<&str> = recs.iter().map(|s| s.as_str()).collect();
            assert_ne!(public_data_hash(&refs, false), base, "{label} must change the hash");
        }
    }

    /// `public_record_count` is bound in, so a removed record is visible.
    #[test]
    fn removing_a_record_changes_the_hash() {
        let p = public();
        assert_ne!(public_data_hash(&p[..4], false), public_data_hash(&p, false));
    }

    #[test]
    fn formats_in_groups_of_four() {
        assert_eq!(format_hash(&public_data_hash(&public(), false)),
                   "70f3 e35a acf7 47db c40f 8376 91aa 61e0");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::pubhash`
Expected: FAIL — `cannot find function public_data_hash`.

- [ ] **Step 3: Implement**

```rust
//! §6.6 — the fixed public-data hash.
//!
//! When a payload has no encrypted section there is no key, so nothing
//! authenticates it. What stands in place of a tag is this: a hash the operator
//! compares against a value they recorded themselves.
//!
//! COMPUTED, NEVER STORED. There is deliberately no hash field on the wire — a
//! hash carried inside the payload is rewritten by whoever rewrites the records,
//! and the device would display a value matching the tampered data perfectly.
//!
//! 128 bits, not 64. The attacker grinds a MATCH, not a preimage, on fields not
//! bound to their key — origin paths, parent fingerprints, record order (which
//! also enables SHA-256 midstate reuse). A candidate costs one to two SHA-256
//! compressions, not a key derivation, so 2^64 is $60k–$250k of rented GPU.

use sha2::{Digest, Sha256};

const LABEL: &[u8] = b"MNEMBLOB/pub/v1";

/// `SHA-256(LABEL ‖ 0x00 ‖ sealed ‖ public_record_count ‖ input)[..16]`
///
/// `sealed` is what makes a downgrade visible. `public_record_count` is the
/// count of records in the PUBLIC section — **not** §6.4's `1..24` cap, which
/// counts both sections; vector D is 5 public of 6 total and the two produce
/// different digests.
pub fn public_data_hash(records: &[&str], sealed: bool) -> [u8; 16] {
    let mut h = Sha256::new();
    h.update(LABEL);
    h.update([0x00]);
    h.update([if sealed { 0x01 } else { 0x00 }]);
    h.update([records.len() as u8]);
    h.update(records.join("\n").as_bytes());
    let d = h.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&d[..16]);
    out
}

/// `a26e d22b b747 dfd0 2367 06ad 14c1 9679` — grouped so a human can compare it.
pub fn format_hash(h: &[u8; 16]) -> String {
    let hex: String = h.iter().map(|b| format!("{b:02x}")).collect();
    (0..8).map(|i| &hex[i * 4..i * 4 + 4]).collect::<Vec<_>>().join(" ")
}
```

- [ ] **Step 4: Run tests** — expect 5 passed. **If `matches_the_pinned_literals` fails, STOP**: the Go port binds to these bytes.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/
git commit -m "seal: the §6.6 fixed public-data hash, domain-separated by shape"
```

---

### Task 6: Section container

**Files:** create `src/seal/container.rs`; modify `src/seal/mod.rs`

**Interfaces:** produces `container::{encode_section, MAX_RECORDS, MAX_RECORD_LEN, ContainerError}`.

- [ ] **Step 1: Wire the module, then write the failing test**

Add `pub mod container;` to `src/seal/mod.rs` first. Then create `src/seal/container.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
    const B: &str = "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374";

    #[test]
    fn joins_with_lf_and_no_trailing_lf() {
        let out = encode_section(&[A.into(), B.into()]).unwrap();
        assert_eq!(*out, format!("{A}\n{B}"));
        assert!(!out.ends_with('\n'));
    }

    /// Validate-one-form-emit-another is the defect shape behind the round-3
    /// Critical. These must be byte-identical.
    #[test]
    fn surrounding_whitespace_does_not_change_the_encoding() {
        assert_eq!(*encode_section(&[A.into(), B.into()]).unwrap(),
                   *encode_section(&[format!("  {A}  "), format!("\t{B}\n")]).unwrap());
    }

    #[test]
    fn refuses_bad_record_counts() {
        assert!(matches!(encode_section(&[]), Err(ContainerError::RecordCount(0))));
        let many: Vec<String> = std::iter::repeat(A.to_string()).take(25).collect();
        assert!(matches!(encode_section(&many), Err(ContainerError::RecordCount(25))));
        let ok: Vec<String> = std::iter::repeat(A.to_string()).take(24).collect();
        assert!(encode_section(&ok).is_ok(), "24 is legal — a 2-of-3 bundle is 15 records");
    }

    #[test]
    fn refuses_embedded_separators_and_bad_lengths() {
        assert!(encode_section(&["".into()]).is_err());
        assert!(encode_section(&[format!("{A}\n{A}")]).is_err());
        assert!(encode_section(&[format!("{A}\r")]).is_err());
        assert!(matches!(encode_section(&[format!("md1{}", "q".repeat(600))]),
            Err(ContainerError::RecordTooLong { .. })));
    }
}
```

- [ ] **Step 2: Run test to verify it fails** — expect `cannot find function encode_section`.

- [ ] **Step 3: Implement**

```rust
//! §6.4 — the LF section encoding. Used identically for the public and the
//! encrypted section; the `1..24` cap is over the TOTAL across both, enforced by
//! the caller.

use zeroize::Zeroizing;

/// Bounded by `bundleReviewFlow`'s paged list, not by `ChoiceScreen`. An earlier
/// draft capped this at 7 from the wrong widget, which would have rejected every
/// multisig wallet — 2-of-2 is 10 records and 2-of-3 is 15.
pub const MAX_RECORDS: usize = 24;
pub const MAX_RECORD_LEN: usize = 512;

#[derive(Debug, PartialEq, Eq)]
pub enum ContainerError {
    RecordCount(usize),
    RecordTooLong { index: usize, len: usize },
    EmbeddedSeparator { index: usize, ch: char },
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerError::RecordCount(n) => write!(
                f, "{n} records; must be 1..={MAX_RECORDS} (2-of-3 multisig is 15)"),
            ContainerError::RecordTooLong { index, len } => write!(
                f, "record {index} is {len} bytes; must be 1..={MAX_RECORD_LEN}"),
            ContainerError::EmbeddedSeparator { index, ch } => write!(
                f, "record {index} contains {ch:?}, which is the record separator"),
        }
    }
}
impl std::error::Error for ContainerError {}

/// Trim ONCE, then validate and encode the SAME trimmed form. Validating one
/// string and emitting another is how a trailing space survives to the device,
/// where `codex32.inputChar` has no mapping for `0x20`.
pub fn encode_section(records: &[String]) -> Result<Zeroizing<String>, ContainerError> {
    if records.is_empty() || records.len() > MAX_RECORDS {
        return Err(ContainerError::RecordCount(records.len()));
    }
    // §6.4: "No CR. A 0x0D anywhere is a malformed bundle. CRLF is rejected,
    // not tolerated." `\r` is `char::is_whitespace`, so trimming FIRST would
    // silently normalise a trailing CR away instead of refusing it. Scan the
    // UNTRIMMED records before trimming.
    for (i, r) in records.iter().enumerate() {
        if let Some(pos) = r.find('\r') {
            return Err(ContainerError::EmbeddedSeparator {
                index: i, ch: r[pos..].chars().next().unwrap() });
        }
    }
    let trimmed: Vec<&str> = records.iter().map(|r| r.trim()).collect();
    for (i, r) in trimmed.iter().enumerate() {
        if r.is_empty() || r.len() > MAX_RECORD_LEN {
            return Err(ContainerError::RecordTooLong { index: i, len: r.len() });
        }
        if let Some(pos) = r.find(['\n', '\r']) {
            return Err(ContainerError::EmbeddedSeparator {
                index: i, ch: r[pos..].chars().next().unwrap() });
        }
    }
    Ok(Zeroizing::new(trimmed.join("\n")))
}
```

- [ ] **Step 4: Run tests** — expect 4 passed.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/
git commit -m "seal: LF section container, canonical and bounded"
```

---

### Task 7: `seal()` and the six canonical vectors

**Files:** modify `src/seal/mod.rs`

**Interfaces:** produces `seal::{Payload, Sealed, SealError, seal}`; `pub(crate) seal_deterministic`.

- [ ] **Step 1: Write the failing test**

Add to `src/seal/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const PASS: &str = "beef beef beef beef beef beef beef beef beef beef beef beef";

    fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
    fn sha(b: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        hex(&Sha256::digest(b))
    }
    fn bacon24() -> String { std::iter::repeat("bacon").take(24).collect::<Vec<_>>().join(" ") }

    /// The six canonical records of the bip84 bundle for `bacon`×24.
    /// Regenerate with --group-size 0; the default (5) emits a DISPLAY form the
    /// engraver rejects. Canonical lengths: 75, 111, 80, 67, 67, 67.
    fn bip84() -> Vec<String> {
        ["ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw",
         "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g",
         "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8",
         "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3",
         "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374",
         "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz"]
        .iter().map(|s| s.to_string()).collect()
    }
    fn salt(hexpair: [u8; 2]) -> [u8; 16] { hexpair.repeat(8).try_into().unwrap() }
    fn iv(hexpair: [u8; 2]) -> [u8; 12] { hexpair.repeat(6).try_into().unwrap() }

    #[test]
    fn vector_a_bacon24_fully_encrypted() {
        let b = seal_deterministic(
            Payload { public: vec![], secret: vec![bacon24()] },
            100_000, salt([0xbe, 0xef]), iv([0xba, 0xc0]), PASS).unwrap();
        assert_eq!(b.len(), 211);
        assert_eq!(sha(&b), "6707c20e7967e80e4cd4cb6dbe05e681d56c722320aa8213886c05a31e94def0");
    }

    /// Identical to A except iterations. The ONLY test that catches a hardcoded
    /// count: the altered-header negative mismatches on AAD regardless.
    #[test]
    fn vector_b_differs_only_in_iterations() {
        let b = seal_deterministic(
            Payload { public: vec![], secret: vec![bacon24()] },
            100_001, salt([0xbe, 0xef]), iv([0xba, 0xc0]), PASS).unwrap();
        assert_eq!(sha(&b), "25fc2eaf950c9455497dc18eea6a93f5a54463a471cd15a4f8f327d13c7fea4c");
    }

    #[test]
    fn vector_c_full_bundle_encrypted() {
        let b = seal_deterministic(
            Payload { public: vec![], secret: bip84() },
            100_000, salt([0xbe, 0xad]), iv([0xca, 0xfe]), PASS).unwrap();
        assert_eq!(b.len(), 540);
        assert_eq!(sha(&b), "272f45e8ee30c95fdb1804ca54a9ec4b1d8c1358967d88c76312c0f725973ffc");
    }

    /// MIXED: 5 public cards in the AAD, ms1 encrypted.
    #[test]
    fn vector_d_mixed() {
        let all = bip84();
        let b = seal_deterministic(
            Payload { public: all[1..].to_vec(), secret: vec![all[0].clone()] },
            100_000, salt([0xd0, 0x0d]), iv([0xf0, 0x0d]), PASS).unwrap();
        assert_eq!(b.len(), 539);
        assert_eq!(sha(&b), "6332e2d674322b2af656677cb550754b1ec7691f3df14895a807297712cdcd6a");
    }

    /// PUBLIC-ONLY: no key, no tag, no passphrase.
    #[test]
    fn vector_e_public_only() {
        let all = bip84();
        let b = seal_public_only(all[1..].to_vec()).unwrap();
        assert_eq!(b.len(), 448);
        assert_eq!(sha(&b), "39b21ef010540d16967bba954bac6e94a888b2811b65df2e829402dc68d1c132");
        // §6.2: the crypto fields must be zero.
        assert_eq!(&b[9..11], &[0, 0]);
        assert!(b[12..44].iter().all(|&x| x == 0), "iterations, salt and iv must be zero");
        assert_eq!(&b[48..52], &[0, 0, 0, 0], "ct_len must be zero");
    }

    /// THREE secret records. Without this a singular implementation of the
    /// session flow passes A–E and every negative.
    #[test]
    fn vector_f_two_of_three_multisig() {
        let recs = two_of_three();
        assert_eq!(recs.len(), 15);
        assert_eq!(recs.iter().filter(|r| r.starts_with("ms1")).count(), 3);
        let b = seal_deterministic(
            Payload { public: vec![], secret: recs },
            100_000, salt([0xf0, 0x0d]), iv([0xbe, 0xef]), PASS).unwrap();
        assert_eq!(b.len(), 1421);
        assert_eq!(sha(&b), "97e059ac91596da711a70197b20a7fec1edbe7992eba6c51751ef062596f1cb6");
    }

    /// §11.1 "Round-trip seal/open" and §11.4 "each vector round-trips to its
    /// exact records". Pinning blob sha256s alone proves the bytes are STABLE,
    /// not that they are PARSEABLE — without this the Go port is the first
    /// consumer to discover a malformed blob, and vector B's whole purpose
    /// (catching a hardcoded iteration count ON DECRYPT) is only half-realised.
    fn open_vector(blob: &[u8], expect: &[String]) {
        let h = wire::Header::decode(blob).expect("header must parse");
        let split = HEADER_LEN + h.pub_len as usize;
        let key = crypto::derive_key(&passphrase::normalise(PASS), &h.salt, h.iterations);
        let pt = crypto::open_bytes(&key, &h.iv, &blob[..split], &blob[split..])
            .expect("vector must decrypt");
        assert_eq!(std::str::from_utf8(&pt).unwrap().split('\n').collect::<Vec<_>>(),
                   expect.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    }

    #[test]
    fn every_encrypted_vector_round_trips() {
        let all = bip84();
        open_vector(&seal_deterministic(
            Payload { public: vec![], secret: vec![bacon24()] },
            100_000, salt([0xbe, 0xef]), iv([0xba, 0xc0]), PASS).unwrap(),
            &[bacon24()]);
        open_vector(&seal_deterministic(
            Payload { public: vec![], secret: vec![bacon24()] },
            100_001, salt([0xbe, 0xef]), iv([0xba, 0xc0]), PASS).unwrap(),
            &[bacon24()]);
        open_vector(&seal_deterministic(
            Payload { public: vec![], secret: all.clone() },
            100_000, salt([0xbe, 0xad]), iv([0xca, 0xfe]), PASS).unwrap(), &all);
        open_vector(&seal_deterministic(
            Payload { public: all[1..].to_vec(), secret: vec![all[0].clone()] },
            100_000, salt([0xd0, 0x0d]), iv([0xf0, 0x0d]), PASS).unwrap(),
            &[all[0].clone()]);
        open_vector(&seal_deterministic(
            Payload { public: vec![], secret: two_of_three() },
            100_000, salt([0xf0, 0x0d]), iv([0xbe, 0xef]), PASS).unwrap(),
            &two_of_three());
    }

    /// §11.4: "any byte of D's public section flipped → AEAD tag mismatch —
    /// this is what proves the AAD covers the cleartext section."
    ///
    /// Task 2's `open_fails_on_tampered_aad` does NOT prove this: it swaps
    /// `b"aad-one"` for `b"aad-two"`, which only shows the `Payload{aad}` field
    /// is wired up. An implementation setting `aad = header` alone would pass it.
    #[test]
    fn flipping_a_public_section_byte_fails_the_tag() {
        let all = bip84();
        let blob = seal_deterministic(
            Payload { public: all[1..].to_vec(), secret: vec![all[0].clone()] },
            100_000, salt([0xd0, 0x0d]), iv([0xf0, 0x0d]), PASS).unwrap();
        let h = wire::Header::decode(&blob).unwrap();
        let split = HEADER_LEN + h.pub_len as usize;
        // First and last byte of the public section.
        for off in [HEADER_LEN, split - 1] {
            let mut bad = blob.clone();
            bad[off] ^= 0x01;
            let key = crypto::derive_key(&passphrase::normalise(PASS), &h.salt, h.iterations);
            assert!(crypto::open_bytes(&key, &h.iv, &bad[..split], &bad[split..]).is_err(),
                "flipping public-section byte {off} must fail the tag");
        }
    }

    /// §11.4: `iterations` altered 100000 → 100002 on vector A. **Not 50000** —
    /// that is rejected by §6.2's floor before any tag work, so it proves
    /// nothing about the AAD.
    #[test]
    fn altering_iterations_in_the_header_fails_the_tag() {
        let blob = seal_deterministic(
            Payload { public: vec![], secret: vec![bacon24()] },
            100_000, salt([0xbe, 0xef]), iv([0xba, 0xc0]), PASS).unwrap();
        let mut bad = blob.clone();
        bad[12..16].copy_from_slice(&100_002u32.to_be_bytes());
        let h = wire::Header::decode(&bad).expect("100002 is inside §6.2's range");
        let key = crypto::derive_key(&passphrase::normalise(PASS), &h.salt, h.iterations);
        assert!(crypto::open_bytes(&key, &h.iv, &bad[..HEADER_LEN], &bad[HEADER_LEN..]).is_err());
    }

    /// §6.4's 1..24 cap is over the TOTAL across both sections — 20 public plus
    /// 10 secret is legal per-section and illegal combined.
    #[test]
    fn refuses_more_than_24_records_across_both_sections() {
        let all = bip84();
        let public: Vec<String> = std::iter::repeat(all[3].clone()).take(20).collect();
        let secret: Vec<String> = std::iter::repeat(all[0].clone()).take(10).collect();
        assert!(seal(Payload { public, secret }, 300_000).is_err());
    }

    /// Nothing else catches a frozen salt: the round-trip test and every fixed-salt
    /// vector pass under one.
    #[test]
    fn two_seals_of_the_same_payload_differ_everywhere() {
        let p = || Payload { public: vec![], secret: vec![bacon24()] };
        let a = seal(p(), 300_000).unwrap();
        let b = seal(p(), 300_000).unwrap();
        assert_ne!(a.blob, b.blob);
        assert_ne!(a.blob[16..32], b.blob[16..32], "salt must be fresh");
        assert_ne!(a.blob[32..44], b.blob[32..44], "iv must be fresh");
        // Option<Zeroizing<String>> — as_deref(), not `*`, which does not compile.
        assert_ne!(a.passphrase.as_deref(), b.passphrase.as_deref(),
            "passphrase must be fresh");
    }

    /// Two vectors sharing a (key, iv) pair would be GCM nonce reuse in our own
    /// test data — the mistake caught in an earlier draft of vector C.
    #[test]
    fn no_two_vectors_share_a_key_iv_pair() {
        use crate::seal::crypto::derive_key;
        let pairs = [
            (hex(&*derive_key(PASS, &salt([0xbe, 0xef]), 100_000)), "bac0".repeat(6)),
            (hex(&*derive_key(PASS, &salt([0xbe, 0xef]), 100_001)), "bac0".repeat(6)),
            (hex(&*derive_key(PASS, &salt([0xbe, 0xad]), 100_000)), "cafe".repeat(6)),
            (hex(&*derive_key(PASS, &salt([0xd0, 0x0d]), 100_000)), "f00d".repeat(6)),
            (hex(&*derive_key(PASS, &salt([0xf0, 0x0d]), 100_000)), "beef".repeat(6)),
        ];
        assert_eq!(pairs.iter().collect::<std::collections::HashSet<_>>().len(), pairs.len());
    }

    #[test]
    fn refuses_a_secret_in_the_public_section() {
        let all = bip84();
        assert!(matches!(
            seal(Payload { public: vec![all[0].clone()], secret: vec![] }, 300_000),
            Err(SealError::SecretInPublic(_))));
    }

    #[test]
    fn refuses_an_undecodable_public_set() {
        let all = bip84();
        // One md1 chunk of three: BCH-valid, but the set does not decode.
        assert!(seal(Payload { public: vec![all[3].clone()], secret: vec![] }, 300_000).is_err());
    }

    /// §11.4: the checksum gate runs BEFORE the KDF. On device that is 1 s
    /// versus 31 s.
    #[test]
    fn refuses_an_invalid_passphrase_without_running_the_kdf() {
        let start = std::time::Instant::now();
        let r = seal_deterministic(
            Payload { public: vec![], secret: vec![bacon24()] },
            2_000_000, salt([0xbe, 0xef]), iv([0xba, 0xc0]),
            &"abandon ".repeat(12));
        assert!(matches!(r, Err(SealError::Passphrase(_))));
        assert!(start.elapsed() < std::time::Duration::from_millis(200),
            "2M rounds cannot finish that fast — the gate must precede the KDF");
    }

    #[test]
    fn refuses_out_of_range_iterations() {
        for bad in [0u32, 99_999, 2_000_001, u32::MAX] {
            assert!(matches!(
                seal_deterministic(Payload { public: vec![], secret: vec![bacon24()] },
                    bad, salt([0xbe, 0xef]), iv([0xba, 0xc0]), PASS),
                Err(SealError::Iterations(_))), "iterations {bad}");
        }
    }

    /// The 15 records of a 2-of-3 wsh-sortedmulti bundle. Regenerate with:
    ///   mnemonic bundle --network mainnet --template wsh-sortedmulti --threshold 2 \
    ///     --group-size 0 --slot "@0.phrase=<bacon x24>" \
    ///     --slot "@1.phrase=<abandon x23 art>" --slot "@2.phrase=<zoo x23 vote>"
    /// Lengths: 75,75,75,111,93,111,93,111,93,85,85,85,85,85,77
    fn two_of_three() -> Vec<String> {
        let v: Vec<String> = [
         "ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw",
         "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w",
         "ms10entrsqrllllllllllllllllllllllllllllllllllllllllllllllllll7ydtcvhdp9ycqe",
         "mk1qpykrepqqspjtpuhfqjc096gykrewjy6dgjcqpcy3zepaggqseet8ky6z2jxm56yh04m5mqslrmueekdmecm0js2h978k03jfvkwz2rxj8r8",
         "mk1qpykrepp6mxahd48msp0tzgj5s9yxznw0r8jupvyxg2vqhhfmravstpsh0pqr0wwaztl9hwerqcp6rxxuqr0jj3uv5",
         "mk1qpykrcpqqspjtpuhfqjc096gykrewjz5xmtjgpcy3zepuga9dymxtm85extpj6vr0eywa3kau0877xtnwy88y2ecc844qldvxxrkfugtp7wl",
         "mk1qpykrcppqlhmpglkpcp56awj4af8qzg4denyzkevx0kzvznstvlxz92phfs5c8dl3yyaxz7hnxpfvse7frygq7c803",
         "mk1qpykrmpqqspjtpuhfqjc096gykrewjpyfsn85pcy3zepaeyst4nt7h2s5dq9fnj6px73rv7a8ycdj3kmgjws23caq3uzztkytr874p4w3r0w",
         "mk1qpykrmppsn7nsjy07cppzrz7mpfxzntx5q8ldmw6ef4xcxmphwnuksya67stktwpuchaewtgnmz6wxyljvaruuh3w5",
         "md1fe4dazspq3m67zzqqvzrs3pstucnf4ztqz4pk6ujgjycfn6zhs79nmzdp9frd6dzth6asfu2za4mwgfkg6",
         "md1fe4dazsdxcy8c7lxwdnw7wxmu5z4e03aadnwmk6nacqh43yf2gzjrpfh83newqkzry9xq4h30dczfwrfjs",
         "md1fe4dazss9a6wcltyzcv9mcgqmmnhgjled6ktm85extpj6vr0eywa3kau0877xtnwy88yq4meswh2tk9vlw",
         "md1fe4dazsavuvr66sqlhmpglkpcp56awj4af8qzg4denyzkevx0kzvznstvlxz92phfs5cqpgq2768n9u8sa",
         "md1fe4daz3rklcjzwnpwhaw4pg6q2n895zdazxea6wfsm9rdk3yaq4r36prcyxz06wzg3lmqdses95045udnl",
         "md1fe4daz3gzzyx9akzjv9xkdgq07mka4jn2dsdkrwa8edqfm4aqhvkure30mjus2yql0jeypuv4u",
        ].iter().map(|s| s.to_string()).collect();
        assert_eq!(v.iter().map(|r| r.len()).collect::<Vec<_>>(),
            vec![75, 75, 75, 111, 93, 111, 93, 111, 93, 85, 85, 85, 85, 85, 77],
            "vector F records are not canonical — did you use --group-size 0?");
        assert_eq!(v.iter().filter(|r| r.starts_with("ms1")).count(), 3);
        v
    }
}
```



- [ ] **Step 2: Run test to verify it fails** — expect `cannot find type Payload`.

- [ ] **Step 3: Implement**

```rust
use rand::RngCore;
use zeroize::Zeroizing;

use crypto::CryptoError;
use wire::{Header, HEADER_LEN, IV_LEN, MAX_SECTION_LEN, MAX_ITERATIONS, MIN_ITERATIONS, SALT_LEN};

/// What is being sealed. `public` rides in the clear (authenticated via the
/// AAD); `secret` is encrypted.
#[derive(Clone)]
pub struct Payload {
    pub public: Vec<String>,
    pub secret: Vec<String>,
}

pub struct Sealed {
    pub blob: Vec<u8>,
    /// `None` when nothing was encrypted — §6.2 forbids crypto fields then, and
    /// a passphrase protecting nothing is worse than none (§9).
    pub passphrase: Option<Zeroizing<String>>,
}

#[derive(Debug)]
pub enum SealError {
    Record(record::RecordError),
    Container(container::ContainerError),
    Passphrase(String),
    Crypto(CryptoError),
    Iterations(u32),
    SecretInPublic(usize),
    Empty,
    TooLarge(usize),
}

impl std::fmt::Display for SealError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SealError::Record(e) => write!(f, "{e}"),
            SealError::Container(e) => write!(f, "{e}"),
            SealError::Passphrase(e) => write!(f, "{e}"),
            SealError::Crypto(e) => write!(f, "{e}"),
            SealError::Iterations(n) => write!(
                f, "iteration count {n} out of range [{MIN_ITERATIONS}, {MAX_ITERATIONS}]"),
            SealError::SecretInPublic(i) => write!(
                f,
                "record {i} is secret material and cannot ride in the public section — \
                 it would be engraved and readable in the clear (§6.3)"
            ),
            SealError::Empty => write!(f, "payload is empty"),
            SealError::TooLarge(n) => write!(
                f, "section is {n} bytes; the cap is {MAX_SECTION_LEN}"),
        }
    }
}
impl std::error::Error for SealError {}

/// Seal a payload. Salt, IV and passphrase are ALWAYS freshly generated — there
/// is deliberately no public seam to supply them. One fresh salt per call means
/// one key per message, which is what makes AES-GCM's nonce requirement
/// structurally unbreakable rather than a procedural promise.
pub fn seal(payload: Payload, iterations: u32) -> Result<Sealed, SealError> {
    // Range-check even on the public-only path, which ignores the value: silently
    // accepting `--iterations 5` teaches the operator the flag is advisory.
    if !(MIN_ITERATIONS..=MAX_ITERATIONS).contains(&iterations) {
        return Err(SealError::Iterations(iterations));
    }
    if payload.secret.is_empty() {
        return Ok(Sealed { blob: seal_public_only(payload.public)?, passphrase: None });
    }
    let passphrase = passphrase::generate();
    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    rand::rng().fill_bytes(&mut salt);
    rand::rng().fill_bytes(&mut iv);
    let blob = seal_deterministic(payload, iterations, salt, iv, &passphrase)?;
    Ok(Sealed { blob, passphrase: Some(passphrase) })
}

/// Validate the public section: no secrets, and it must decode as a card set.
fn check_public(public: &[String]) -> Result<(), SealError> {
    if public.is_empty() {
        return Ok(());
    }
    for (i, r) in public.iter().enumerate() {
        if record::validate_record(r).map_err(SealError::Record)?.is_secret() {
            return Err(SealError::SecretInPublic(i));
        }
    }
    let refs: Vec<&str> = public.iter().map(|s| s.trim()).collect();
    record::decode_public_set(&refs).map_err(SealError::Record)
}

/// §6.2's unencrypted shape: no key, no tag, all crypto fields zero.
pub fn seal_public_only(public: Vec<String>) -> Result<Vec<u8>, SealError> {
    check_public(&public)?;
    let pubsec = container::encode_section(&public).map_err(SealError::Container)?;
    let bytes = pubsec.as_bytes();
    if bytes.is_empty() {
        return Err(SealError::Empty);
    }
    if bytes.len() > MAX_SECTION_LEN as usize {
        return Err(SealError::TooLarge(bytes.len()));
    }
    let header = Header {
        iterations: 0, salt: [0; SALT_LEN], iv: [0; IV_LEN],
        pub_len: bytes.len() as u32, ct_len: 0,
    };
    let mut blob = Vec::with_capacity(HEADER_LEN + bytes.len());
    blob.extend_from_slice(&header.encode());
    blob.extend_from_slice(bytes);
    Ok(blob)
}

/// Deterministic seam for the canonical vectors ONLY.
///
/// `pub(crate)` and never re-exported. A public version destroys the
/// one-key-one-message property the moment a caller reuses a salt, and there is
/// no legitimate reason for a caller to choose one.
pub(crate) fn seal_deterministic(
    payload: Payload,
    iterations: u32,
    salt: [u8; SALT_LEN],
    iv: [u8; IV_LEN],
    passphrase: &str,
) -> Result<Vec<u8>, SealError> {
    if !(MIN_ITERATIONS..=MAX_ITERATIONS).contains(&iterations) {
        return Err(SealError::Iterations(iterations));
    }
    if !passphrase::is_valid(passphrase) {
        return Err(SealError::Passphrase(
            "passphrase is not a checksum-valid BIP-39 mnemonic".into()));
    }
    check_public(&payload.public)?;
    for r in &payload.secret {
        record_or_mnemonic(r)?;
    }

    let pubsec: Zeroizing<String> = if payload.public.is_empty() {
        Zeroizing::new(String::new())
    } else {
        container::encode_section(&payload.public).map_err(SealError::Container)?
    };
    let secsec = container::encode_section(&payload.secret).map_err(SealError::Container)?;

    let pb = pubsec.as_bytes();
    let sb = secsec.as_bytes();
    for n in [pb.len(), sb.len()] {
        if n > MAX_SECTION_LEN as usize {
            return Err(SealError::TooLarge(n));
        }
    }
    if payload.public.len() + payload.secret.len() > container::MAX_RECORDS {
        return Err(SealError::Container(container::ContainerError::RecordCount(
            payload.public.len() + payload.secret.len())));
    }

    let header = Header {
        iterations, salt, iv,
        pub_len: pb.len() as u32, ct_len: sb.len() as u32,
    };
    // §6.1a: AAD = header ‖ public section.
    let mut aad = Vec::with_capacity(HEADER_LEN + pb.len());
    aad.extend_from_slice(&header.encode());
    aad.extend_from_slice(pb);

    let key = crypto::derive_key(&passphrase::normalise(passphrase), &salt, iterations);
    let sealed = crypto::seal_bytes(&key, &iv, &aad, sb).map_err(SealError::Crypto)?;

    let mut blob = aad;
    blob.extend_from_slice(&sealed);
    Ok(blob)
}

/// A secret-section record is a constellation record OR a BIP-39 mnemonic.
fn record_or_mnemonic(s: &str) -> Result<(), SealError> {
    // Keep the RecordError. It carries the "re-run with --group-size 0"
    // guidance, which is the remedy for the round-3 Critical — and the SECRET
    // section is the DEFAULT path, i.e. exactly where an operator pasting
    // `mnemonic bundle` output lands. Collapsing it into a generic message
    // loses the one sentence that tells them what to do.
    let record_err = match record::validate_record(s) {
        Ok(_) => return Ok(()),
        Err(e) => e,
    };
    if passphrase::is_valid(s) {
        return Ok(());
    }
    Err(SealError::Record(record_err))
}
```

- [ ] **Step 4: Run tests** — `cargo test -p mnemonic-engrave --lib seal`, expect **52 passed** (8 wire + 5 crypto + 5 passphrase + 7 record + 5 pubhash + 4 container + 16 mod + 2 uf2). **If any vector hash mismatches, STOP and reconcile against the spec** — the Go port binds to these bytes.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/
git commit -m "seal: seal() plus all six canonical vectors and the freshness guard"
```

---

### Task 8: UF2 emission

**Files:** create `src/seal/uf2.rs`; modify `src/seal/mod.rs`

- [ ] **Step 1: Wire the module, then write the failing test**

Add `pub mod uf2;` to `src/seal/mod.rs` first. Then create `src/seal/uf2.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    fn field(b: &[u8], off: usize) -> u32 { u32::from_le_bytes(b[off..off+4].try_into().unwrap()) }

    #[test]
    fn every_block_conforms_not_just_the_first() {
        // 600 bytes = 3 blocks, the last short. §9.1 requires payloadSize 256 on
        // EVERY block; writing chunk.len() for the final block would pass a
        // first-block-only check.
        let uf2 = to_uf2(&vec![0xABu8; 600]);
        assert_eq!(uf2.len(), 3 * 512);
        for (i, b) in uf2.chunks(512).enumerate() {
            assert_eq!(field(b, 0), 0x0A32_4655, "block {i} magicStart0");
            assert_eq!(field(b, 4), 0x9E5D_5157, "block {i} magicStart1");
            assert_eq!(field(b, 8), 0x0000_2000, "block {i} flags");
            assert_eq!(field(b, 12), TARGET_ADDR + i as u32 * 256, "block {i} addr");
            assert_eq!(field(b, 16), 256, "block {i} payloadSize must be 256");
            assert_eq!(field(b, 20), i as u32, "block {i} blockNo");
            assert_eq!(field(b, 24), 3, "block {i} numBlocks");
            assert_eq!(field(b, 28), FAMILY_DATA, "block {i} familyID (data, NOT rp2350_arm_s)");
            assert_eq!(field(b, 508), 0x0AB1_6F30, "block {i} magicEnd");
        }
        // The pinned vector sha256s assume 0x00 padding.
        assert!(uf2[2 * 512 + 32 + 88..2 * 512 + 32 + 256].iter().all(|&b| b == 0));
    }

    #[test]
    fn single_block_for_a_short_blob() {
        assert_eq!(to_uf2(&vec![0xABu8; 211]).len(), 512);
    }
}
```

- [ ] **Step 2: Run test to verify it fails.**

- [ ] **Step 3: Implement**

```rust
//! `data`-family UF2 emission (§9.1).
//!
//! Written by the RP2350 bootrom in BOOTSEL mode, not by the running firmware.
//! Verified on real hardware 2026-08-06: a data-family UF2 at 0x10E00000 lands
//! byte-exact and leaves the signed image untouched.

/// Normative and not configurable. §5 derives it so the blob clears the signed
/// image and stays inside physical flash; past 0x11000000 a write wraps to
/// 0x10000000 and destroys the firmware.
pub const TARGET_ADDR: u32 = 0x10E0_0000;

/// `data`. NOT 0xe48bff59 (`rp2350_arm_s`), the bootable-image family the TinyGo
/// target uses — correct for firmware, wrong for a blob.
pub const FAMILY_DATA: u32 = 0xE48B_FF58;

const MAGIC_START0: u32 = 0x0A32_4655;
const MAGIC_START1: u32 = 0x9E5D_5157;
const MAGIC_END: u32 = 0x0AB1_6F30;
const FLAG_FAMILY_ID_PRESENT: u32 = 0x0000_2000;
const PAYLOAD: usize = 256;

pub fn to_uf2(blob: &[u8]) -> Vec<u8> {
    debug_assert!(!blob.is_empty(), "a blob is always >= 52 bytes");
    let num_blocks = blob.len().div_ceil(PAYLOAD) as u32;
    let mut out = Vec::with_capacity(num_blocks as usize * 512);
    for (i, chunk) in blob.chunks(PAYLOAD).enumerate() {
        let mut b = [0u8; 512];
        b[0..4].copy_from_slice(&MAGIC_START0.to_le_bytes());
        b[4..8].copy_from_slice(&MAGIC_START1.to_le_bytes());
        b[8..12].copy_from_slice(&FLAG_FAMILY_ID_PRESENT.to_le_bytes());
        b[12..16].copy_from_slice(&(TARGET_ADDR + i as u32 * PAYLOAD as u32).to_le_bytes());
        // Always 256, even for a short final chunk: the bootrom requires it, and
        // the device bounds every read by pub_len/ct_len so padding is unseen.
        b[16..20].copy_from_slice(&(PAYLOAD as u32).to_le_bytes());
        b[20..24].copy_from_slice(&(i as u32).to_le_bytes());
        b[24..28].copy_from_slice(&num_blocks.to_le_bytes());
        b[28..32].copy_from_slice(&FAMILY_DATA.to_le_bytes());
        b[32..32 + chunk.len()].copy_from_slice(chunk);
        b[508..512].copy_from_slice(&MAGIC_END.to_le_bytes());
        out.extend_from_slice(&b);
    }
    out
}
```

- [ ] **Step 4: Run tests** — expect 2 passed.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/
git commit -m "seal: data-family UF2 emission at 0x10E00000"
```

---

### Task 9: `me seal` and `me hash`

**Files:** modify `src/main.rs`; create `tests/seal_cli.rs`

The real enum is `Command` (singular, `main.rs:39`), and `run()` returns `i32` with **no `match`** — dispatch is a series of early returns (`main.rs:74-82`). `?` is a compile error in `run()`. Mirror `run_bundle_cli` (`main.rs:164`).

- [ ] **Step 1: Write the failing test**

Create `crates/me-cli/tests/seal_cli.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

const MD1: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
const MD1B: &str = "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374";
const MD1C: &str = "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz";
const MS1: &str = "ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw";

fn me() -> Command { Command::cargo_bin("me").unwrap() }

/// Find the generated passphrase on stderr.
///
/// **Do NOT match on "a line with 12 whitespace-separated tokens".** Two lines
/// of `me seal`'s own prose have exactly 12 tokens — the passphrase header
/// (`passphrase — write this down and store it APART from the machine:`) and
/// `RECORD THIS WHOLE LINE. The device shows the same value; if it`. A
/// token-count heuristic returns the header, which made the §2.3 containment
/// assertion below VACUOUS: it degenerated to `!uf2.contains("passphrase")`,
/// and a mutation copying the real twelve words into the UF2's padding left the
/// test GREEN.
fn passphrase_line(err: &str) -> Option<&str> {
    err.lines().find(|l| {
        let w: Vec<&str> = l.split_whitespace().collect();
        w.len() == 12 && w.iter().all(|t| t.chars().all(|c| c.is_ascii_lowercase()))
    })
}

#[test]
fn seals_and_prints_the_passphrase_to_stderr_only() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    let a = me().args(["seal", MS1, "--seal-secret", "--out", out.to_str().unwrap()])
        .assert().success();
    let err = String::from_utf8(a.get_output().stderr.clone()).unwrap();
    let words = passphrase_line(&err).expect("the 12-word passphrase must reach stderr");
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(bytes.len() % 512, 0);
    // §2.3: the passphrase must never land beside the ciphertext it opens.
    // Assert on the LONGEST word: a 3-letter BIP-39 word ("act", "air") has a
    // ~1-in-30,000 chance of appearing by chance in ~500 random ciphertext
    // bytes — a flake nobody would diagnose.
    let longest = words.split_whitespace().max_by_key(|w| w.len()).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains(longest),
        "no passphrase word may appear in the UF2");
}

/// §8 / §2.2a: the prohibition is load-bearing. Assert the flag is ABSENT —
/// `.failure()` alone would also pass if someone added `--passphrase` with
/// validation that happened to reject the value.
#[test]
fn there_is_no_passphrase_flag() {
    me().args(["seal", MD1, "--passphrase", "hunter2"]).assert()
        .failure().stderr(predicate::str::contains("unexpected argument"));
    me().args(["seal", "--help"]).assert()
        .success().stdout(predicate::str::contains("--passphrase").not());
}

#[test]
fn there_is_no_addr_flag() {
    me().args(["seal", MD1, "--addr", "0x10000000"]).assert()
        .failure().stderr(predicate::str::contains("unexpected argument"));
    me().args(["seal", "--help"]).assert()
        .success().stdout(predicate::str::contains("--addr").not());
}

#[test]
fn refuses_ms1_without_the_opt_in_flag() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    me().args(["seal", MS1, "--out", out.to_str().unwrap()])
        .assert().failure().stderr(predicate::str::contains("--seal-secret"));
    assert!(!out.exists(), "nothing may be written on the refusal path");
}

/// A public-only payload prompts for nothing and prints no passphrase (§9).
#[test]
fn public_only_payload_prints_no_passphrase() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    let a = me().args(["seal", "--plaintext", MD1, "--plaintext", MD1B,
                       "--plaintext", MD1C, "--out", out.to_str().unwrap()])
        .assert().success();
    let err = String::from_utf8(a.get_output().stderr.clone()).unwrap();
    assert!(passphrase_line(&err).is_none(),
        "no passphrase may be printed when nothing is encrypted");
    let b = std::fs::read(&out).unwrap();
    assert_eq!(&b[48..52], &[0, 0, 0, 0], "ct_len must be zero");
}

#[test]
fn refuses_a_secret_in_the_public_section() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    me().args(["seal", "--plaintext", MS1, "--out", out.to_str().unwrap()])
        .assert().failure();
    assert!(!out.exists());
}

#[test]
fn refuses_space_grouped_input_with_an_actionable_message() {
    me().args(["seal", "md1fv9w jpqpqpm6", "--out", "/dev/null"])
        .assert().failure().stderr(predicate::str::contains("--group-size 0"));
}

#[test]
fn refuses_out_of_range_iterations() {
    for bad in ["5", "3000000000"] {
        me().args(["seal", MS1, "--seal-secret", "--out", "/dev/null", "--iterations", bad])
            .assert().failure();
    }
}

#[test]
fn me_hash_reproduces_both_shapes() {
    let mk1 = "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g";
    let mk2 = "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8";
    me().args(["hash", "--unsealed", mk1, mk2, MD1, MD1B, MD1C])
        .assert().success()
        .stdout(predicate::str::contains("70f3 e35a acf7 47db c40f 8376 91aa 61e0"));
    me().args(["hash", "--sealed", mk1, mk2, MD1, MD1B, MD1C])
        .assert().success()
        .stdout(predicate::str::contains("a26e d22b b747 dfd0 2367 06ad 14c1 9679"));
}
```

Add `tempfile = "3"` to `[dev-dependencies]`.

- [ ] **Step 2: Run test to verify it fails** — `me` has no `seal` subcommand.

- [ ] **Step 3: Implement**

Add to `enum Command` in `src/main.rs`:

```rust
    /// Encrypt a payload for delivery to SeedHammer II flash.
    ///
    /// The passphrase is GENERATED and printed to STDERR — write it down and
    /// store it apart from the machine. There is deliberately no way to supply
    /// your own: total strength is the passphrase plus about 20 bits from the
    /// KDF, and a memorable passphrase does not survive an offline attack on a
    /// stolen machine.
    Seal {
        /// Records to ENCRYPT. Must be canonical: if they came from
        /// `mnemonic bundle`, use --group-size 0.
        payload: Vec<String>,

        /// Records to carry in the CLEAR. Authenticated via the AAD when
        /// something is also encrypted; unauthenticated otherwise. Never an
        /// ms1 or a BIP-39 mnemonic.
        #[arg(long = "plaintext")]
        plaintext: Vec<String>,

        /// Write the UF2 here. Created 0600. REQUIRED — never stdout, because
        /// the passphrase shares that stream.
        #[arg(long, required = true)]
        out: PathBuf,

        /// Required to encrypt an ms1 (a seed). Sealing a seed must never be
        /// accidental.
        ///
        /// **NOTE: this flag is a deliberate plan-level addition and is NOT in
        /// the spec.** §9's synopsis omits it and §12 item 6 records `ms1` as
        /// ADMITTED with no opt-in, so `me seal <ms1> --out x.uf2` — the spec's
        /// own documented invocation — exits `EXIT_REFUSED` here. That is safer
        /// than the spec, not looser, but it is a divergence: file a spec
        /// amendment to §9 and §12 item 6 rather than leaving the two artefacts
        /// disagreeing.
        #[arg(long)]
        seal_secret: bool,

        /// PBKDF2 iterations. 300,000 = 30.9 s on device, from the measured
        /// 9,715 iters/sec (§7.1, measured 2026-08-07 on real RP2350).
        #[arg(long, default_value_t = 300_000)]
        iterations: u32,
    },

    /// Re-derive the §6.6 public-data hash from your own cards.
    ///
    /// No passphrase, no seal operation, no original file — so the expected
    /// value can be regenerated months later and compared against what the
    /// device displays.
    Hash {
        /// The public records, in order.
        records: Vec<String>,
        /// The payload was sealed (carries an encrypted section).
        #[arg(long, conflicts_with = "unsealed")]
        sealed: bool,
        /// The payload carries no encrypted section.
        #[arg(long)]
        unsealed: bool,
    },
```

Add early returns in `run()`, after the existing `Command::Bundle` block:

```rust
    if let Some(Command::Seal { payload, plaintext, out, seal_secret, iterations }) = &cli.command {
        return run_seal_cli(payload, plaintext, out, *seal_secret, *iterations);
    }
    if let Some(Command::Hash { records, sealed, unsealed }) = &cli.command {
        if *sealed == *unsealed {
            eprintln!("me: pass exactly one of --sealed or --unsealed");
            return EXIT_USAGE;
        }
        return run_hash_cli(records, *sealed);
    }
```

Add the handlers alongside `run_bundle_cli`:

```rust
// `out: &Path`, not `&PathBuf` — clippy::ptr_arg is warn-by-default and the
// plan's own `-D warnings` gate would reject it. Call sites deref-coerce.
fn run_seal_cli(
    payload: &[String], plaintext: &[String], out: &std::path::Path,
    seal_secret: bool, iterations: u32,
) -> i32 {
    use mnemonic_engrave::classify::{classify, Format};
    use mnemonic_engrave::seal::{self, pubhash, Payload};

    // Global Constraint: secrets wiped on every path. argv already exposes these
    // via /proc/$PID/cmdline (inherent to §9's synopsis, filed separately), so
    // this is defence in depth on the heap copy we control.
    let secret: Vec<Zeroizing<String>> =
        payload.iter().map(|s| Zeroizing::new(s.trim().to_string())).collect();
    let secret: Vec<String> = secret.iter().map(|s| (**s).clone()).collect();
    let public: Vec<String> = plaintext.iter().map(|s| s.trim().to_string()).collect();
    if secret.is_empty() && public.is_empty() {
        eprintln!("me: nothing to seal");
        return EXIT_USAGE;
    }

    // §9: ms1 needs the explicit opt-in. Checked on classification, not on
    // anything the caller asserts.
    if !seal_secret && secret.iter().any(|r| matches!(classify(r), Ok(Format::Ms))) {
        eprintln!(
            "me: refusing to seal ms1 without --seal-secret.\n    \
             ms1 is seed entropy. Sealing it puts an offline-attackable ciphertext of your \
             seed into the machine's flash, defended only by the generated passphrase.\n    \
             Re-run with --seal-secret if that is what you intend."
        );
        return EXIT_REFUSED;
    }

    let sealed = match seal::seal(Payload { public: public.clone(), secret }, iterations) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("me: {e}");
            return match e {
                seal::SealError::Iterations(_) => EXIT_USAGE,
                _ => EXIT_INVALID,
            };
        }
    };

    let uf2 = seal::uf2::to_uf2(&sealed.blob);
    if let Err(e) = write_private(out, &uf2) {
        eprintln!("me: cannot write {}: {e}", out.display());
        return EXIT_USAGE;
    }

    // STDERR, always (§2.3).
    eprintln!("me: wrote {} bytes to {}", uf2.len(), out.display());
    if !public.is_empty() {
        let refs: Vec<&str> = public.iter().map(|s| s.as_str()).collect();
        let h = pubhash::public_data_hash(&refs, sealed.passphrase.is_some());
        eprintln!();
        eprintln!("public data hash ({} records, {}):",
            public.len(), if sealed.passphrase.is_some() { "SEALED" } else { "UNSEALED" });
        eprintln!("    {}", pubhash::format_hash(&h));
        eprintln!("RECORD THIS WHOLE LINE. The device shows the same value; if it");
        eprintln!("differs, the payload has been altered or its encryption removed.");
    }
    if let Some(p) = &sealed.passphrase {
        eprintln!();
        eprintln!("passphrase — write this down and store it APART from the machine:");
        eprintln!();
        eprintln!("    {}", &**p);
    }
    eprintln!();
    eprintln!("load:  picotool load --verify {}   (machine in BOOTSEL)", out.display());
    eprintln!("wipe:  picotool erase -r 0x10E00000 0x10E10000");
    EXIT_OK
}

fn run_hash_cli(records: &[String], sealed: bool) -> i32 {
    use mnemonic_engrave::seal::{pubhash, record};
    if records.is_empty() {
        eprintln!("me: no records given");
        return EXIT_USAGE;
    }
    let trimmed: Vec<String> = records.iter().map(|s| s.trim().to_string()).collect();
    for (i, r) in trimmed.iter().enumerate() {
        match record::validate_record(r) {
            Err(e) => { eprintln!("me: record {i}: {e}"); return EXIT_INVALID; }
            // §6.3 forbids a secret in the public section, so hashing one would
            // print a confident value for a payload no device could ever hold.
            Ok(k) if k.is_secret() => {
                eprintln!("me: record {i} is secret material; the public-data hash \
                           covers public records only");
                return EXIT_INVALID;
            }
            Ok(_) => {}
        }
    }
    let refs: Vec<&str> = trimmed.iter().map(|s| s.as_str()).collect();
    // Same card-set decode `me seal --plaintext` applies, so `me hash` cannot
    // bless a record list that `me seal` would refuse.
    if let Err(e) = record::decode_public_set(&refs) {
        eprintln!("me: {e}");
        return EXIT_INVALID;
    }
    println!("{}", pubhash::format_hash(&pubhash::public_data_hash(&refs, sealed)));
    EXIT_OK
}
```

- [ ] **Step 4: Run tests** — `cargo test -p mnemonic-engrave --test seal_cli`, expect 9 passed.

- [ ] **Step 5: Run the whole suite**

Run: `cargo test -p mnemonic-engrave && cargo clippy -p mnemonic-engrave -- -D warnings`
Expected: everything green. Confirm the pre-existing `convert`/`bundle` tests still pass — Tasks 1 and 4 touched shared code.

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add crates/me-cli/src/main.rs crates/me-cli/tests/ crates/me-cli/Cargo.toml Cargo.lock
git commit -m "seal: me seal and me hash subcommands"
```

---

## Mutation testing (required before Plan A is done)

Per §11.3 and project standard, a green suite proves little. **Every mutant must name the test that fails.** A mutant with no killer is a gap, not a pass.

Procedure, both rules non-negotiable: **copy the file first** and restore from the copy, never `git checkout` (it has reverted real uncommitted work); and **assert the substitution matched** before running — a silently-failing `sed` reads exactly like a surviving mutation.

| Mutant | Killed by |
| --- | --- |
| `derive_key` ignores `iterations` | `vector_b_differs_only_in_iterations` |
| `seal` reuses a fixed salt | `two_seals_of_the_same_payload_differ_everywhere` |
| `sealed` byte dropped from the hash input | `pubhash::sealed_and_unsealed_differ` **and** the literals |
| hash computed over a subset of the section | `pubhash::matches_the_pinned_literals` + `every_byte_of_the_section_affects_the_hash` |
| `public_record_count` dropped | `pubhash::matches_the_pinned_literals` (NOT `removing_a_record_changes_the_hash` — LF-joined records are already injective, so that passes under the mutant) |
| `encode_section` appends a trailing LF | `joins_with_lf_and_no_trailing_lf` |
| `validate_record` strips whitespace instead of refusing | `refuses_space_grouped_and_hyphenated_records` |
| uppercase check removed | `refuses_uppercase_records` |
| per-record decode instead of per-set | `decodes_a_complete_card_set` (would reject a legitimate set) |
| decode check removed entirely | `refuses_a_bch_valid_but_undecodable_record` |
| `Header::decode` skips a length bound | `rejects_out_of_range_lengths` |
| `reserved` / `kdf_id` / `aead_id` checks deleted (sealed shape) | `rejects_bad_magic_version_reserved_kdf_and_aead` — mutation-proved that nothing else catches these |
| `aad = header` only, dropping the public section | `flipping_a_public_section_byte_fails_the_tag` (the pinned D/E sha256s catch it too, but nothing else tests the §6.1a property itself) |
| a vector emits an unparseable blob | `every_encrypted_vector_round_trips` — sha256 pins prove the bytes are STABLE, not PARSEABLE |
| CR normalised away instead of refused | `refuses_embedded_separators_and_bad_lengths` |
| `--group-size 0` guidance lost on the secret path | `refuses_space_grouped_input_with_an_actionable_message` |
| unsealed-shape zero checks removed | `rejects_nonzero_crypto_fields_when_unsealed` |
| `to_uf2` emits family `0xE48BFF59` or pads `0xFF` | `every_block_conforms_not_just_the_first` |
| `open_bytes` returns plaintext without checking the tag | `open_fails_on_flipped_ciphertext_byte` |
| passphrase generated for a public-only payload | `public_only_payload_prints_no_passphrase` |
| `ms1` opt-in ignored | `refuses_ms1_without_the_opt_in_flag` |
| secret admitted to the public section | `refuses_a_secret_in_the_public_section` |
| only the first secret record handled | **not covered here** — this is a §10.2.2 device behaviour; Plan B must carry it, using vector F |

Record results in `design/agent-reports/`.

---

## What Plan A does NOT cover

- **All firmware work** — XIP read, header parse on device, the §10.2.1 allow-list, the card-set decode, the `ms1`-first session and its wipe rule, the two-phase idle timer, the §10.2.3 warning, the plate list. That is **Plan B**, which binds to the vectors this plan emits and may never lead them (Rust-primary rule). **Vector F's offer-order property is a Plan B assertion**, not a Plan A one.
- **§12 item 3** — MSD drag-and-drop, untested. `picotool load` is the documented path.
- **The RP2350A-vs-B confirmation** of the 9,715 iters/sec figure — folds into Plan B.
- **`me bundle` integration.** `bundle.rs` returns `BundleError::RefusedSecret` on any `ms1` line. Reconciling that with §12 item 6's admission is deliberate follow-up work: the refusal must be lifted **only** on the sealed path, never on the plaintext NDEF one.

---

## Self-review notes

**Spec coverage.** §6.1a AAD → Task 7. §6.2 bounds and both shapes → Task 1. §6.3 section placement + card-set decode → Task 4. §6.4 container + lowercase → Tasks 4, 6. §6.6 hash → Task 5. §7 crypto → Task 2. §7.2 one-key-one-message → Task 7's `pub(crate)` seam + freshness test. §7.3 host-side randomness → Tasks 3, 7. §8 passphrase → Task 3. §9 CLI → Task 9. §11.1 Rust tests → distributed, vectors in Task 7.

**Deliberate gaps.** The `(key, iv)` uniqueness test enumerates the five encrypted vectors explicitly rather than deriving the set — a derived set silently passes when someone forgets to register a new vector. Vector F's *offer-order* property is device behaviour and belongs to Plan B; Plan A only pins its bytes.
