# Encrypted Payload Delivery — Plan A (Rust host, `me seal`) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `me seal` to the `mnemonic-engrave` CLI: it encrypts a constellation payload under a host-generated 12-word BIP-39 passphrase and emits a `data`-family UF2 that the operator loads into SeedHammer II flash at `0x10E00000`.

**Architecture:** A new `seal/` module implementing the `SPEC_encrypted_payload_delivery.md` wire format — 48-byte authenticated header, PBKDF2-HMAC-SHA256 → AES-256-GCM, plaintext being either one bare constellation string or LF-separated bundle records. All randomness is host-side; the device only ever decrypts. This plan produces the **normative** implementation and its test vectors; the Go firmware port (Plan B) is bound to the vectors this plan emits and may not lead it.

**Tech Stack:** Rust 2021, `aes-gcm`, `pbkdf2`, `sha2`, `bip39`, `rand`, `zeroize`, existing `md-codec`/`mk-codec`, new `ms-codec`.

**Spec:** `design/SPEC_encrypted_payload_delivery.md` (R0 GREEN, commit `844bd35`). Section references below (§6.2, §6.4, …) are to that document. **Read §6, §7, §9 and §11 before starting.**

## Global Constraints

- **Rust-primary rule.** This crate is the normative implementation of the wire format. The Go port is downstream and binds to the vectors produced here. Never change wire behaviour to match Go.
- **No caller-supplied salt or IV in any public API** (§7.2). The one-key-one-message property is structural, and a public seam that accepts a salt destroys it. The test-only injection point is `pub(crate)` and unit-tested from inside the module — never exported, never behind a CLI flag.
- **No `--addr` flag** (§9). The target address `0x10E00000` is normative; a wrong address either produces a blob the device never reads or, past `0x11000000`, wraps to `0x10000000` and destroys the signed firmware.
- **The CLI must not accept a user-supplied passphrase** (§8, §2.2a). It is generated, always. This prohibition is load-bearing, not advisory: total strength is passphrase entropy + ~20 KDF bits.
- **Records must be canonical** — no interior whitespace, no `-` (§6.4). This is the R0 round-3 Critical. Refuse; never strip.
- Constants, all normative: `MAGIC = b"MNEMBLOB"`, `VERSION = 0x01`, `HEADER_LEN = 48`, `SALT_LEN = 16`, `IV_LEN = 12`, `TAG_LEN = 16`, `iterations ∈ [100_000, 2_000_000]`, `ct_len ∈ [1, 8191]`, `record_count ∈ [1, 24]`, per-record length `∈ [1, 512]`, region length `65536`.
- All multi-byte integers are **big-endian**.
- Secrets are wiped with `zeroize` on every path.
- `cargo fmt` and `cargo clippy -- -D warnings` must pass before each commit.

---

## File Structure

New module directory `crates/me-cli/src/seal/`. The existing crate is flat, but `bundle.rs` (29 KB) and `preview.rs` (23 KB) show what happens when concerns accumulate in one file; six focused files are easier to review and to hold in context.

| File | Responsibility |
| --- | --- |
| `src/seal/mod.rs` | Public API (`seal`, `Payload`, `Sealed`, `SealError`), the `pub(crate)` deterministic seam, and the canonical vector tests |
| `src/seal/wire.rs` | `Header` type, encode/decode, every §6.2 bound |
| `src/seal/crypto.rs` | `derive_key`, `seal_bytes`, `open_bytes` |
| `src/seal/passphrase.rs` | 12-word BIP-39 generation and §8.1 normalisation |
| `src/seal/container.rs` | Canonical-record checks, payload-kind classification, LF container encode |
| `src/seal/uf2.rs` | `data`-family UF2 block emission |
| `src/main.rs` (modify) | `Seal` subcommand wiring |
| `src/lib.rs` (modify) | `pub mod seal;` |
| `src/validate.rs` (modify) | Extract the canonical check so `seal` reuses it verbatim |
| `tests/seal_cli.rs` | End-to-end CLI behaviour via `assert_cmd` |

---

### Task 1: Dependencies and the wire header

**Files:**
- Modify: `crates/me-cli/Cargo.toml`
- Create: `crates/me-cli/src/seal/wire.rs`
- Create: `crates/me-cli/src/seal/mod.rs`
- Modify: `crates/me-cli/src/lib.rs:9` (add `pub mod seal;`)

**Interfaces:**
- Consumes: nothing.
- Produces: `wire::{Header, PayloadKind, WireError, HEADER_LEN, TAG_LEN, SALT_LEN, IV_LEN, MIN_ITERATIONS, MAX_ITERATIONS, MAX_CT_LEN, REGION_LEN}`; `Header::encode(&self) -> [u8; 48]`; `Header::decode(&[u8]) -> Result<Header, WireError>`.

- [ ] **Step 1: Add dependencies**

In `crates/me-cli/Cargo.toml`, under `[dependencies]`:

```toml
aes-gcm = "0.10"
pbkdf2 = { version = "0.12", default-features = false, features = ["hmac"] }
sha2 = "0.10"
bip39 = "2.2"
rand = "0.9"
ms-codec = "0.7"
```

Run: `cargo build -p mnemonic-engrave`
Expected: builds clean.

- [ ] **Step 2: Wire the module, then write the failing test**

**Do the wiring first.** An undeclared `.rs` file is not compiled, so `cargo test` would report `0 passed` and exit 0 — a green RED step, which is a false PASS in the TDD gate itself.

Create `crates/me-cli/src/seal/mod.rs`:

```rust
//! `me seal` — encrypt a constellation payload for delivery to SeedHammer II
//! flash. See design/SPEC_encrypted_payload_delivery.md.

pub mod wire;
```

Add to `crates/me-cli/src/lib.rs` after line 8 (`pub mod preview;`):

```rust
pub mod seal;
```

Now create `crates/me-cli/src/seal/wire.rs` containing only this test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Header {
        Header {
            kind: PayloadKind::Bip39,
            iterations: 100_000,
            salt: [0xbe; SALT_LEN],
            iv: [0xc0; IV_LEN],
            ct_len: 143,
        }
    }

    #[test]
    fn header_round_trips() {
        let h = sample();
        let bytes = h.encode();
        assert_eq!(bytes.len(), HEADER_LEN);
        assert_eq!(&bytes[..8], b"MNEMBLOB");
        assert_eq!(Header::decode(&bytes).unwrap(), h);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut b = sample().encode();
        b[0] = b'X';
        assert!(matches!(Header::decode(&b), Err(WireError::BadMagic)));
    }

    #[test]
    fn rejects_out_of_range_iterations() {
        for bad in [0u32, 99_999, 2_000_001, u32::MAX] {
            let mut h = sample();
            h.iterations = bad;
            assert!(
                matches!(Header::decode(&h.encode()), Err(WireError::Iterations(_))),
                "iterations {bad} must be refused"
            );
        }
    }

    #[test]
    fn rejects_out_of_range_ct_len() {
        // 8192 is one past the cap: gui/scan.go flags overflow when its 8 KiB
        // buffer is exactly full, so 8191 is the largest classifiable payload.
        for bad in [0u32, 8192, 0xFFFF_FFF0, u32::MAX] {
            let mut h = sample();
            h.ct_len = bad;
            assert!(
                matches!(Header::decode(&h.encode()), Err(WireError::CtLen(_))),
                "ct_len {bad} must be refused"
            );
        }
    }

    #[test]
    fn rejects_unknown_kind_version_kdf_and_aead() {
        for (off, val) in [(11u8, 0x05u8), (8, 0x02), (9, 0x02), (10, 0x02)] {
            let mut b = sample().encode();
            b[off as usize] = val;
            assert!(
                Header::decode(&b).is_err(),
                "byte {off} = {val:#x} must be refused"
            );
        }
        // Pin the specific variants so a mutation that collapses them all into
        // one error is still caught.
        let mut b = sample().encode();
        b[11] = 0x05;
        assert!(matches!(Header::decode(&b), Err(WireError::UnknownKind(5))));
        let mut b = sample().encode();
        b[9] = 0x02;
        assert!(matches!(Header::decode(&b), Err(WireError::UnknownKdf(2))));
        let mut b = sample().encode();
        b[10] = 0x02;
        assert!(matches!(Header::decode(&b), Err(WireError::UnknownAead(2))));
    }

    #[test]
    fn rejects_a_short_buffer() {
        assert!(matches!(
            Header::decode(&[0u8; 47]),
            Err(WireError::TooShort(47))
        ));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::wire`
Expected: FAIL — `cannot find type Header in this scope`.

- [ ] **Step 4: Implement the header**

Prepend to `crates/me-cli/src/seal/wire.rs` (above the test module):

```rust
//! The §6 wire header: 48 bytes, big-endian, authenticated in full as the AEAD
//! AAD. Parsed BEFORE authentication (it carries the salt and iteration count),
//! so it is hostile input by construction and every field is bound-checked here.

pub const MAGIC: [u8; 8] = *b"MNEMBLOB";
pub const VERSION: u8 = 0x01;
pub const KDF_PBKDF2_SHA256: u8 = 0x01;
pub const AEAD_AES256GCM: u8 = 0x01;

pub const HEADER_LEN: usize = 48;
pub const SALT_LEN: usize = 16;
pub const IV_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

pub const MIN_ITERATIONS: u32 = 100_000;
pub const MAX_ITERATIONS: u32 = 2_000_000;
pub const MAX_CT_LEN: u32 = 8191;
pub const REGION_LEN: u64 = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadKind {
    MdMk = 0x01,
    Bip39 = 0x02,
    Ms1 = 0x03,
    Bundle = 0x04,
}

impl PayloadKind {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(PayloadKind::MdMk),
            0x02 => Some(PayloadKind::Bip39),
            0x03 => Some(PayloadKind::Ms1),
            0x04 => Some(PayloadKind::Bundle),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub kind: PayloadKind,
    pub iterations: u32,
    pub salt: [u8; SALT_LEN],
    pub iv: [u8; IV_LEN],
    pub ct_len: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WireError {
    TooShort(usize),
    BadMagic,
    UnknownVersion(u8),
    UnknownKdf(u8),
    UnknownAead(u8),
    UnknownKind(u8),
    Iterations(u32),
    CtLen(u32),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::TooShort(n) => write!(f, "blob too short: {n} bytes, need at least 48"),
            WireError::BadMagic => write!(f, "not a sealed payload (bad magic)"),
            WireError::UnknownVersion(v) => write!(f, "unsupported format version {v}"),
            WireError::UnknownKdf(k) => write!(f, "unsupported kdf id {k}"),
            WireError::UnknownAead(a) => write!(f, "unsupported aead id {a}"),
            WireError::UnknownKind(k) => write!(f, "unsupported payload kind {k}"),
            WireError::Iterations(n) => write!(
                f,
                "iteration count {n} out of range [{MIN_ITERATIONS}, {MAX_ITERATIONS}]"
            ),
            WireError::CtLen(n) => write!(f, "ciphertext length {n} out of range [1, {MAX_CT_LEN}]"),
        }
    }
}
impl std::error::Error for WireError {}

impl Header {
    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let mut out = [0u8; HEADER_LEN];
        out[..8].copy_from_slice(&MAGIC);
        out[8] = VERSION;
        out[9] = KDF_PBKDF2_SHA256;
        out[10] = AEAD_AES256GCM;
        out[11] = self.kind as u8;
        out[12..16].copy_from_slice(&self.iterations.to_be_bytes());
        out[16..32].copy_from_slice(&self.salt);
        out[32..44].copy_from_slice(&self.iv);
        out[44..48].copy_from_slice(&self.ct_len.to_be_bytes());
        out
    }

    /// Parse and bound-check. Every check here runs BEFORE any KDF work: the
    /// firmware has no active watchdog, so an unbounded iteration count is a
    /// hang rather than an error.
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
        if buf[9] != KDF_PBKDF2_SHA256 {
            return Err(WireError::UnknownKdf(buf[9]));
        }
        if buf[10] != AEAD_AES256GCM {
            return Err(WireError::UnknownAead(buf[10]));
        }
        let kind = PayloadKind::from_byte(buf[11]).ok_or(WireError::UnknownKind(buf[11]))?;

        let iterations = u32::from_be_bytes(buf[12..16].try_into().unwrap());
        if !(MIN_ITERATIONS..=MAX_ITERATIONS).contains(&iterations) {
            return Err(WireError::Iterations(iterations));
        }
        let ct_len = u32::from_be_bytes(buf[44..48].try_into().unwrap());
        if ct_len == 0 || ct_len > MAX_CT_LEN {
            return Err(WireError::CtLen(ct_len));
        }
        // Defence in depth, and UNREACHABLE as written: every ct_len large
        // enough to wrap 32-bit `48 + ct_len + 16` is already rejected by the
        // 8191 cap above. It is kept because §6.2's overflow warning is aimed at
        // the GO PORT, whose `int` is 32 bits — which is why the
        // `ct_len = 0xFFFF_FFF0` requirement lives in §11.2 (device) and not
        // §11.1 (host). Do NOT write a Rust test claiming to exercise this
        // branch; no input can reach it.
        let total = HEADER_LEN as u64 + ct_len as u64 + TAG_LEN as u64;
        if total > REGION_LEN {
            return Err(WireError::CtLen(ct_len));
        }

        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&buf[16..32]);
        let mut iv = [0u8; IV_LEN];
        iv.copy_from_slice(&buf[32..44]);
        Ok(Header { kind, iterations, salt, iv, ct_len })
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib seal::wire`
Expected: 6 passed.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/Cargo.toml Cargo.lock crates/me-cli/src/seal/wire.rs crates/me-cli/src/seal/mod.rs crates/me-cli/src/lib.rs
git commit -m "seal: wire header with bound checks before any KDF work"
```

---

### Task 2: KDF and AEAD

**Files:**
- Create: `crates/me-cli/src/seal/crypto.rs`
- Modify: `crates/me-cli/src/seal/mod.rs` (add `pub mod crypto;`)

**Interfaces:**
- Consumes: `wire::{IV_LEN, SALT_LEN}` — those two only; importing the others trips `-D warnings` on `unused_imports`.
- Produces: `crypto::derive_key(passphrase: &str, salt: &[u8; 16], iterations: u32) -> zeroize::Zeroizing<[u8; 32]>`; `crypto::seal_bytes(key, iv, aad, plaintext) -> Result<Vec<u8>, CryptoError>` returning `ciphertext || tag`; `crypto::open_bytes(key, iv, aad, sealed) -> Result<Zeroizing<Vec<u8>>, CryptoError>`.

- [ ] **Step 1: Wire the module, then write the failing test**

Add `pub mod crypto;` to `crates/me-cli/src/seal/mod.rs` **first** — an undeclared file is not compiled and the RED step would falsely report `0 passed`.

Then create `crates/me-cli/src/seal/crypto.rs` with only this test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Canonical vector A inputs (SPEC §11.4). The derived key is pinned here
    // because both implementations bind to it.
    const PASSPHRASE: &str = "beef beef beef beef beef beef beef beef beef beef beef beef";

    #[test]
    fn derives_the_pinned_key() {
        let salt = [0xbeu8, 0xef].repeat(8);
        let salt: [u8; 16] = salt.try_into().unwrap();
        let key = derive_key(PASSPHRASE, &salt, 100_000);
        assert_eq!(
            hex(&*key),
            "615ad9b781b1ad6105d9dffb135d1bf17ebab286c560f26912ee815836e7ad1e"
        );
    }

    #[test]
    fn different_iteration_counts_derive_different_keys() {
        // Vector B exists precisely to catch a hardcoded iteration count.
        let salt = [0xbeu8, 0xef].repeat(8);
        let salt: [u8; 16] = salt.try_into().unwrap();
        let a = derive_key(PASSPHRASE, &salt, 100_000);
        let b = derive_key(PASSPHRASE, &salt, 100_001);
        assert_ne!(*a, *b);
        assert_eq!(
            hex(&*b),
            "003800ae6cec47cd4b34bb264c6bbb1156d806516ad1ab88391e479d14d8776f"
        );
    }

    #[test]
    fn seal_open_round_trips() {
        let key = [7u8; 32];
        let iv = [9u8; 12];
        let aad = b"header-bytes";
        let sealed = seal_bytes(&key, &iv, aad, b"plaintext").unwrap();
        assert_eq!(sealed.len(), 9 + 16);
        assert_eq!(&*open_bytes(&key, &iv, aad, &sealed).unwrap(), b"plaintext");
    }

    #[test]
    fn open_fails_on_tampered_aad() {
        let key = [7u8; 32];
        let iv = [9u8; 12];
        let sealed = seal_bytes(&key, &iv, b"aad-one", b"plaintext").unwrap();
        assert!(open_bytes(&key, &iv, b"aad-two", &sealed).is_err());
    }

    #[test]
    fn open_fails_on_flipped_ciphertext_byte() {
        let key = [7u8; 32];
        let iv = [9u8; 12];
        let mut sealed = seal_bytes(&key, &iv, b"aad", b"plaintext").unwrap();
        sealed[0] ^= 0x01;
        assert!(open_bytes(&key, &iv, b"aad", &sealed).is_err());
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::crypto`
Expected: FAIL — `cannot find function derive_key`.

- [ ] **Step 3: Implement**

Prepend to `crates/me-cli/src/seal/crypto.rs`:

```rust
//! PBKDF2-HMAC-SHA256 → AES-256-GCM (§7). Both are already linked into the
//! SeedHammer firmware, which is why they were chosen over scrypt/Argon2 —
//! neither of those fits its own standard's recommended memory on an RP2350.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use sha2::Sha256;
use zeroize::Zeroizing;

use super::wire::{IV_LEN, SALT_LEN};

#[derive(Debug, PartialEq, Eq)]
pub enum CryptoError {
    /// Tag mismatch, or the ciphertext was altered. Fail closed: no plaintext
    /// is ever returned on this path.
    Authentication,
    /// Ciphertext shorter than a bare tag.
    TooShort,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::Authentication => write!(
                f,
                "wrong passphrase or damaged payload (authentication failed)"
            ),
            CryptoError::TooShort => write!(f, "sealed payload too short"),
        }
    }
}
impl std::error::Error for CryptoError {}

/// Stretch the passphrase into a 32-byte key. `iterations` always comes from
/// the header — never a constant, or vector B fails.
pub fn derive_key(
    passphrase: &str,
    salt: &[u8; SALT_LEN],
    iterations: u32,
) -> Zeroizing<[u8; 32]> {
    let mut key = Zeroizing::new([0u8; 32]);
    pbkdf2::pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, iterations, &mut *key);
    key
}

/// Returns `ciphertext || tag`.
pub fn seal_bytes(
    key: &[u8; 32],
    iv: &[u8; IV_LEN],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new(key.into());
    cipher
        .encrypt(Nonce::from_slice(iv), Payload { msg: plaintext, aad })
        .map_err(|_| CryptoError::Authentication)
}

/// Verify then decrypt. `aes-gcm` returns an error without releasing plaintext
/// on tag mismatch, which is what makes it safe to parse the §6.4 container
/// out of the result.
pub fn open_bytes(
    key: &[u8; 32],
    iv: &[u8; IV_LEN],
    aad: &[u8],
    sealed: &[u8],
) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if sealed.len() < 16 {
        return Err(CryptoError::TooShort);
    }
    let cipher = Aes256Gcm::new(key.into());
    cipher
        .decrypt(Nonce::from_slice(iv), Payload { msg: sealed, aad })
        .map(Zeroizing::new)
        .map_err(|_| CryptoError::Authentication)
}
```


- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib seal::crypto`
Expected: 5 passed. If `derives_the_pinned_key` fails, **stop** — the KDF inputs disagree with the spec and every downstream vector is wrong.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/crypto.rs crates/me-cli/src/seal/mod.rs
git commit -m "seal: PBKDF2-HMAC-SHA256 + AES-256-GCM, pinned to vector A's key"
```

---

### Task 3: Passphrase generation

**Files:**
- Create: `crates/me-cli/src/seal/passphrase.rs`
- Modify: `crates/me-cli/src/seal/mod.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `passphrase::generate() -> Zeroizing<String>` (12 BIP-39 words, space separated, lowercase); `passphrase::normalise(&str) -> Zeroizing<String>`; `passphrase::is_valid(&str) -> bool`.

- [ ] **Step 1: Wire the module, then write the failing test**

Add `pub mod passphrase;` to `crates/me-cli/src/seal/mod.rs` **first** — an undeclared file is not compiled and the RED step would falsely report `0 passed`.

Then create `crates/me-cli/src/seal/passphrase.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_twelve_valid_words() {
        let p = generate();
        assert_eq!(p.split(' ').count(), 12);
        assert!(is_valid(&p), "generated passphrase must have a valid checksum");
        assert_eq!(*p, p.to_lowercase(), "must be lowercase");
        assert!(!p.starts_with(' ') && !p.ends_with(' '));
    }

    #[test]
    fn two_generations_differ() {
        // Freshness at the passphrase level. A frozen RNG here is as fatal as a
        // frozen salt.
        assert_ne!(*generate(), *generate());
    }

    #[test]
    fn accepts_the_beef_vector() {
        // beef x12 is checksum-valid: a 1-in-16 coincidence, verified against
        // the BIP-39 English wordlist. It is the canonical vector passphrase.
        assert!(is_valid("beef beef beef beef beef beef beef beef beef beef beef beef"));
    }

    #[test]
    fn rejects_near_miss_and_invalid() {
        // beef x11 + bacon is a valid-length mnemonic of real words that differs
        // in one position and is checksum-INVALID. A gate that accepts it is broken.
        assert!(!is_valid("beef beef beef beef beef beef beef beef beef beef beef bacon"));
        assert!(!is_valid("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon"));
        assert!(!is_valid("not even words"));
    }

    #[test]
    fn normalise_collapses_whitespace_and_case() {
        let n = normalise("  BEEF   beef\tbeef  ");
        assert_eq!(*n, "beef beef beef");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::passphrase`
Expected: FAIL — `cannot find function generate`.

- [ ] **Step 3: Implement**

Prepend to `crates/me-cli/src/seal/passphrase.rs`:

```rust
//! The decryption passphrase: a host-generated 12-word BIP-39 mnemonic, 128
//! bits (§8).
//!
//! It is GENERATED, never user-supplied. Total strength is passphrase entropy
//! plus the ~20 bits the KDF adds, and a human-chosen passphrase is worth
//! 25–35 bits — which falls to a single rented GPU in minutes. `age` reached
//! the same conclusion and generates 10 words rather than letting the user pick.
//!
//! It is used ONLY as a passphrase. It is never seed entropy and never derives
//! a wallet.

use bip39::{Language, Mnemonic};
use rand::RngCore;
use zeroize::Zeroizing;

/// 12 fresh words from the OS CSPRNG.
pub fn generate() -> Zeroizing<String> {
    let mut entropy = Zeroizing::new([0u8; 16]);
    rand::rng().fill_bytes(&mut *entropy);
    let m = Mnemonic::from_entropy_in(Language::English, &*entropy)
        .expect("16 bytes is always a valid 12-word entropy length");
    Zeroizing::new(m.to_string())
}

/// §8.1: lowercase, single-space separated, no leading or trailing space. Host
/// and device MUST produce byte-identical input to the KDF.
pub fn normalise(s: &str) -> Zeroizing<String> {
    Zeroizing::new(
        s.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

/// Checksum-valid BIP-39 English mnemonic? The device runs this check before
/// committing to a ~30 s KDF, so a typo costs a second rather than half a minute.
pub fn is_valid(s: &str) -> bool {
    Mnemonic::parse_in(Language::English, &*normalise(s)).is_ok()
}
```


- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib seal::passphrase`
Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/passphrase.rs crates/me-cli/src/seal/mod.rs
git commit -m "seal: generate the 12-word BIP-39 passphrase; never accept one"
```

---

### Task 4: Canonical records and payload classification

**Files:**
- Modify: `crates/me-cli/src/validate.rs:63-91` (extract the canonical check)
- Create: `crates/me-cli/src/seal/container.rs`
- Modify: `crates/me-cli/src/seal/mod.rs`

**Interfaces:**
- Consumes: `classify::{classify, Format}`.
- Produces: `validate::first_noncanonical(&str) -> Option<(usize, char)>`; `container::{classify_record, RecordKind, ContainerError}`; `container::validate_record(&str) -> Result<RecordKind, ContainerError>`.

- [ ] **Step 1: Extract the canonical check without changing behaviour**

In `crates/me-cli/src/validate.rs`, add above `validate`'s doc comment (i.e. before line 56 — inserting at line 63 would orphan that doc comment onto the new function):

```rust
/// First interior separator in a trimmed constellation string, if any.
/// Canonical = no `-` anywhere and no interior whitespace. Callers must have
/// trimmed already, so any remaining whitespace is interior.
///
/// Shared by the NDEF converter (md1 only, historical) and by `seal`, which
/// applies it to md1/mk1/ms1 alike — a sealed record is engraved verbatim just
/// as a converted one is, so the same reasoning holds for all three.
pub fn first_noncanonical(s: &str) -> Option<(usize, char)> {
    s.char_indices().find(|(_, c)| c.is_whitespace() || *c == '-')
}
```

Then replace the body of the `Format::Md` arm's canonical check (currently `validate.rs:74-77`) with:

```rust
            if let Some((pos, ch)) = first_noncanonical(s) {
                return Err(ValidateError::MdNonCanonical { ch, pos });
            }
```

Run: `cargo test -p mnemonic-engrave --lib validate`
Expected: all existing tests still pass — this is a pure extraction.

- [ ] **Step 2: Wire the module, then write the failing test**

Add `pub mod container;` to `crates/me-cli/src/seal/mod.rs` **first** — an undeclared file is not compiled and the RED step would falsely report `0 passed`.

Then create `crates/me-cli/src/seal/container.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const MD1: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
    const MK1: &str = "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";
    const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

    #[test]
    fn classifies_each_record_kind() {
        assert_eq!(validate_record(MD1).unwrap(), RecordKind::MdMk);
        assert_eq!(validate_record(MK1).unwrap(), RecordKind::MdMk);
        assert_eq!(validate_record(MS1).unwrap(), RecordKind::Ms1);
    }

    #[test]
    fn refuses_space_grouped_records() {
        // THE round-3 Critical. `mnemonic bundle` prints --group-size 5 by
        // default; codex32's inputChar has no mapping for 0x20, so the device
        // classifies such a record as unknown. Refuse here — never strip, or the
        // plate carries separators the BCH checksum never covered.
        let spaced = "md1yqp qqxqq 8xtwh w4xwn 4qh";
        assert!(matches!(
            validate_record(spaced),
            Err(ContainerError::NonCanonical { ch: ' ', .. })
        ));
    }

    #[test]
    fn refuses_hyphenated_records() {
        assert!(matches!(
            validate_record("md1yqpqq-xqq8xtwhw4xwn4qh"),
            Err(ContainerError::NonCanonical { ch: '-', .. })
        ));
    }

    #[test]
    fn refuses_corrupt_and_unknown_records() {
        let mut bad = MD1.to_string();
        let last = bad.pop().unwrap();
        bad.push(if last == 'q' { 'p' } else { 'q' });
        assert!(validate_record(&bad).is_err());
        assert!(validate_record("xx1qqqq").is_err());
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::container`
Expected: FAIL — `cannot find function validate_record`.

- [ ] **Step 4: Implement**

Prepend to `crates/me-cli/src/seal/container.rs`:

```rust
//! Record validation and the §6.4 bundle container.

use crate::classify::{classify, Format};
use crate::validate::first_noncanonical;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    /// md1 or mk1 — public (an xpub and a wallet policy).
    MdMk,
    /// ms1 — the seed. Secret.
    Ms1,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ContainerError {
    NonCanonical { ch: char, pos: usize },
    Unclassifiable(String),
    Invalid(String),
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerError::NonCanonical { ch, pos } => write!(
                f,
                "non-canonical record: separator {ch:?} at byte {pos} — records must be \
                 unbroken (no interior whitespace, no '-'). If this came from `mnemonic \
                 bundle`, re-run it with --group-size 0: the default --group-size 5 emits a \
                 DISPLAY form the engraver cannot read."
            ),
            ContainerError::Unclassifiable(e) => write!(f, "unrecognised record: {e}"),
            ContainerError::Invalid(e) => write!(f, "invalid record: {e}"),
        }
    }
}
impl std::error::Error for ContainerError {}

/// Validate one record and report what it is. Rejects non-canonical form BEFORE
/// the checksum check, so the operator gets the actionable message.
pub fn validate_record(s: &str) -> Result<RecordKind, ContainerError> {
    let s = s.trim();
    if let Some((pos, ch)) = first_noncanonical(s) {
        return Err(ContainerError::NonCanonical { ch, pos });
    }
    let fmt = classify(s).map_err(|e| ContainerError::Unclassifiable(e.to_string()))?;
    match fmt {
        Format::Md => {
            md_codec::codex32::unwrap_string(s)
                .map(|_| RecordKind::MdMk)
                .map_err(|e| ContainerError::Invalid(e.to_string()))
        }
        Format::Mk => {
            let d = mk_codec::string_layer::decode_string(s)
                .map_err(|e| ContainerError::Invalid(e.to_string()))?;
            if d.corrections_applied != 0 {
                return Err(ContainerError::Invalid(format!(
                    "not pristine: required {} BCH correction(s)",
                    d.corrections_applied
                )));
            }
            Ok(RecordKind::MdMk)
        }
        // ms_codec::decode(s) -> Result<(Tag, Payload)>, re-exported at
        // ms-codec 0.7.0 src/lib.rs:55. Use THIS, never the sibling
        // `decode_with_correction` — that one BCH-repairs silently, and a seed
        // that needed repair must be fixed at the source, not engraved.
        Format::Ms => ms_codec::decode(s)
            .map(|_| RecordKind::Ms1)
            .map_err(|e| ContainerError::Invalid(e.to_string())),
    }
}
```


- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib seal::container && cargo test -p mnemonic-engrave --lib validate`
Expected: all pass, including the pre-existing `validate` tests.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/validate.rs crates/me-cli/src/seal/container.rs crates/me-cli/src/seal/mod.rs
git commit -m "seal: canonical record validation across md1/mk1/ms1"
```

---

### Task 5: The bundle container

**Files:**
- Modify: `crates/me-cli/src/seal/container.rs`

**Interfaces:**
- Consumes: `container::{validate_record, RecordKind, ContainerError}`.
- Produces: `container::encode_bundle(records: &[String]) -> Result<Zeroizing<String>, ContainerError>`; `container::{MAX_RECORDS, MAX_RECORD_LEN}`.

- [ ] **Step 1: Write the failing test**

Append to the `tests` module in `crates/me-cli/src/seal/container.rs`:

```rust
    #[test]
    fn encodes_records_lf_separated_without_trailing_lf() {
        let recs = vec![MD1.to_string(), MK1.to_string()];
        let out = encode_bundle(&recs).unwrap();
        assert_eq!(*out, format!("{MD1}\n{MK1}"));
        assert!(!out.ends_with('\n'), "no trailing LF: the encoding is canonical");
    }

    #[test]
    fn surrounding_whitespace_does_not_change_the_encoding() {
        // Validate-one-form-emit-another is the defect shape that produced the
        // R0 round-3 Critical. These two must be byte-identical.
        let clean = vec![MD1.to_string(), MK1.to_string()];
        let padded = vec![format!("  {MD1}  "), format!("\t{MK1}\n")];
        assert_eq!(*encode_bundle(&clean).unwrap(), *encode_bundle(&padded).unwrap());
    }

    #[test]
    fn refuses_empty_and_oversized_record_lists() {
        assert!(matches!(encode_bundle(&[]), Err(ContainerError::RecordCount(0))));
        let too_many: Vec<String> = std::iter::repeat(MD1.to_string()).take(25).collect();
        assert!(matches!(
            encode_bundle(&too_many),
            Err(ContainerError::RecordCount(25))
        ));
        // 24 is legal — a 2-of-3 multisig bundle is 15 records.
        let ok: Vec<String> = std::iter::repeat(MD1.to_string()).take(24).collect();
        assert!(encode_bundle(&ok).is_ok());
    }

    #[test]
    fn refuses_empty_record_and_embedded_separators() {
        assert!(encode_bundle(&["".to_string()]).is_err());
        assert!(encode_bundle(&[format!("{MD1}\n{MD1}")]).is_err());
        assert!(encode_bundle(&[format!("{MD1}\r")]).is_err());
    }

    #[test]
    fn refuses_oversized_record() {
        let long = format!("md1{}", "q".repeat(600));
        assert!(matches!(
            encode_bundle(&[long]),
            Err(ContainerError::RecordTooLong { .. })
        ));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::container`
Expected: FAIL — `cannot find function encode_bundle`.

- [ ] **Step 3: Implement**

Add to `crates/me-cli/src/seal/container.rs`:

```rust
use zeroize::Zeroizing;

/// Bounded by `bundleReviewFlow`'s paged list, not by `ChoiceScreen`. An earlier
/// draft capped this at 7 from the wrong widget, which would have rejected every
/// multisig wallet — 2-of-2 is 10 records and 2-of-3 is 15.
pub const MAX_RECORDS: usize = 24;
pub const MAX_RECORD_LEN: usize = 512;
```

Add these variants to `ContainerError`:

```rust
    RecordCount(usize),
    RecordTooLong { index: usize, len: usize },
```

and their `Display` arms:

```rust
            ContainerError::RecordCount(n) => write!(
                f,
                "bundle has {n} records; must be 1..={MAX_RECORDS} (2-of-3 multisig is 15)"
            ),
            ContainerError::RecordTooLong { index, len } => write!(
                f,
                "record {index} is {len} bytes; must be 1..={MAX_RECORD_LEN}"
            ),
```

Then:

```rust
/// Join validated records with a single LF and no trailing LF (§6.4). Every
/// constraint is enforced here so a bundle the device will refuse never leaves
/// the host — the operator gets an actionable message instead of "payload
/// unreadable" after a 30-second KDF.
pub fn encode_bundle(records: &[String]) -> Result<Zeroizing<String>, ContainerError> {
    if records.is_empty() || records.len() > MAX_RECORDS {
        return Err(ContainerError::RecordCount(records.len()));
    }
    // Trim ONCE, here, and both validate and encode the trimmed form.
    //
    // `validate_record` trims internally, so validating the raw record and then
    // joining the raw record would validate one string and emit a different
    // one. A trailing space survives to the device, where codex32's inputChar
    // has no mapping for 0x20 and the whole bundle is rejected after the KDF.
    // That is the same defect shape as the round-3 Critical and as the
    // pre-existing A3/F4 finding in validate.rs — validate one form, emit
    // another.
    let trimmed: Vec<&str> = records.iter().map(|r| r.trim()).collect();
    for (i, r) in trimmed.iter().enumerate() {
        if r.is_empty() || r.len() > MAX_RECORD_LEN {
            return Err(ContainerError::RecordTooLong { index: i, len: r.len() });
        }
        // Catches embedded LF/CR before the canonical check gives a vaguer message.
        if let Some(pos) = r.find(['\n', '\r']) {
            return Err(ContainerError::NonCanonical {
                ch: r[pos..].chars().next().unwrap(),
                pos,
            });
        }
        validate_record(r)?;
    }
    Ok(Zeroizing::new(trimmed.join("\n")))
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib seal::container`
Expected: 9 passed (4 from Task 4 + 5 here).

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/container.rs
git commit -m "seal: LF bundle container, canonical and bounded"
```

---

### Task 6: The seal API and the canonical vectors

**Files:**
- Modify: `crates/me-cli/src/seal/mod.rs`

**Interfaces:**
- Consumes: everything from Tasks 1–5.
- Produces: `seal::{Payload, Sealed, SealError}`; `seal::seal(payload: Payload, iterations: u32) -> Result<Sealed, SealError>`; `pub(crate) seal::seal_deterministic(payload, iterations, salt, iv, passphrase) -> Result<Vec<u8>, SealError>`.

- [ ] **Step 1: Write the failing test**

Add to `crates/me-cli/src/seal/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const PASS_A: &str = "beef beef beef beef beef beef beef beef beef beef beef beef";

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
    fn sha256_hex(b: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        hex(&Sha256::digest(b))
    }

    /// SPEC §11.4 vector A. Byte-exact; the Go port binds to this.
    #[test]
    fn vector_a() {
        let pt = std::iter::repeat("bacon").take(24).collect::<Vec<_>>().join(" ");
        let blob = seal_deterministic(
            Payload::Bip39(pt.clone()),
            100_000,
            [0xbe, 0xef].repeat(8).try_into().unwrap(),
            [0xba, 0xc0].repeat(6).try_into().unwrap(),
            PASS_A,
        )
        .unwrap();
        assert_eq!(pt.len(), 143);
        assert_eq!(blob.len(), 207);
        assert_eq!(
            sha256_hex(&blob),
            "53d4991a41994089fbbe35e1c576335d8d6e82904ecd531257397d1780e16bb9"
        );
    }

    /// Vector B — identical to A except `iterations`. MUST succeed. This is the
    /// ONLY test that catches a hardcoded iteration count: the altered-header
    /// negative case mismatches on AAD regardless of what the KDF used.
    #[test]
    fn vector_b() {
        let pt = std::iter::repeat("bacon").take(24).collect::<Vec<_>>().join(" ");
        let blob = seal_deterministic(
            Payload::Bip39(pt),
            100_001,
            [0xbe, 0xef].repeat(8).try_into().unwrap(),
            [0xba, 0xc0].repeat(6).try_into().unwrap(),
            PASS_A,
        )
        .unwrap();
        assert_eq!(
            sha256_hex(&blob),
            "edcba9c5125060a2ae35dc4e99b9d46030e3672409917e4bf12d95d81d15d4fe"
        );
    }

    /// Vector C — a real six-record bip84 bundle, CANONICAL form.
    #[test]
    fn vector_c() {
        let records = vector_c_records();
        let blob = seal_deterministic(
            Payload::Bundle(records),
            100_000,
            [0xbe, 0xad].repeat(8).try_into().unwrap(),
            [0xca, 0xfe].repeat(6).try_into().unwrap(),
            PASS_A,
        )
        .unwrap();
        assert_eq!(blob.len(), 536);
        assert_eq!(
            sha256_hex(&blob),
            "45c31f0096175da31cbc61a2e11a026b6766a2491da5a90291db1b7c829e2536"
        );
    }

    /// Freshness. Nothing else in the suite catches a frozen salt — the
    /// round-trip test and the fixed-salt vectors both pass under one.
    #[test]
    fn two_seals_of_the_same_payload_differ_everywhere() {
        let pt = std::iter::repeat("bacon").take(24).collect::<Vec<_>>().join(" ");
        let a = seal(Payload::Bip39(pt.clone()), 100_000).unwrap();
        let b = seal(Payload::Bip39(pt), 100_000).unwrap();
        assert_ne!(a.blob, b.blob, "ciphertext must differ");
        assert_ne!(a.blob[16..32], b.blob[16..32], "salt must be fresh");
        assert_ne!(a.blob[32..44], b.blob[32..44], "iv must be fresh");
        assert_ne!(*a.passphrase, *b.passphrase, "passphrase must be fresh");
    }

    /// No two shipped vectors may share a (derived key, iv) pair. Two that did
    /// would be GCM nonce reuse in our own test data — which is exactly what
    /// happened to the first draft of vector C.
    #[test]
    fn no_vector_shares_a_key_iv_pair() {
        use crate::seal::crypto::derive_key;
        let beef: [u8; 16] = [0xbe, 0xef].repeat(8).try_into().unwrap();
        let bead: [u8; 16] = [0xbe, 0xad].repeat(8).try_into().unwrap();
        let pairs = [
            (hex(&*derive_key(PASS_A, &beef, 100_000)), "bac0".repeat(6)),
            (hex(&*derive_key(PASS_A, &beef, 100_001)), "bac0".repeat(6)),
            (hex(&*derive_key(PASS_A, &bead, 100_000)), "cafe".repeat(6)),
        ];
        let uniq: std::collections::HashSet<_> = pairs.iter().collect();
        assert_eq!(uniq.len(), pairs.len(), "a (key, iv) pair is reused");
    }

    #[test]
    fn seal_refuses_a_space_grouped_bundle() {
        let mut recs = vector_c_records();
        recs[3] = "md1fv 9wjpq pqpm6 jzzqq".to_string();
        assert!(seal(Payload::Bundle(recs), 100_000).is_err());
    }

    /// §11.1's `payload_kind` classification requirement. Vectors A and C pin
    /// 0x02 and 0x04 only transitively, through their sha256s; nothing pinned
    /// 0x01 or 0x03, and `Payload::MdMk`/`Payload::Ms1` were never constructed
    /// in any test. Swapping the two discriminants left the whole suite green.
    #[test]
    fn payload_kind_byte_matches_the_input_shape() {
        const MD1: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
        const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
        let bacon = std::iter::repeat("bacon").take(24).collect::<Vec<_>>().join(" ");
        let cases = [
            (Payload::MdMk(MD1.into()), 0x01u8),
            (Payload::Bip39(bacon), 0x02),
            (Payload::Ms1(MS1.into()), 0x03),
            (Payload::Bundle(vector_c_records()), 0x04),
        ];
        for (i, (p, want)) in cases.into_iter().enumerate() {
            // Vary salt and IV per case. Four distinct plaintexts under one
            // (key, nonce) pair would be GCM nonce reuse — in the very suite
            // whose §11.1 assertion forbids exactly that. Harmless here (these
            // blobs never leave the function) but the invariant should not have
            // an exception nobody can see the reason for.
            let blob = seal_deterministic(
                p,
                100_000,
                [0xbe ^ i as u8; SALT_LEN],
                [0xba ^ i as u8; IV_LEN],
                PASS_A,
            )
            .unwrap();
            assert_eq!(blob[11], want, "payload_kind byte for case {i}");
        }
    }

    /// §11.4: the passphrase checksum is checked BEFORE the KDF runs. On device
    /// that is the difference between a 1-second rejection and a 30-second one.
    #[test]
    fn refuses_an_invalid_passphrase_without_running_the_kdf() {
        let bacon = std::iter::repeat("bacon").take(24).collect::<Vec<_>>().join(" ");
        let started = std::time::Instant::now();
        let r = seal_deterministic(
            Payload::Bip39(bacon),
            2_000_000, // max iterations: if the KDF ran, this is unmissable
            [0xbe; SALT_LEN],
            [0xba; IV_LEN],
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon",
        );
        assert!(matches!(r, Err(SealError::Passphrase(_))));
        assert!(
            started.elapsed() < std::time::Duration::from_millis(200),
            "the checksum gate must reject before the KDF; 2M rounds cannot finish that fast"
        );
    }

    #[test]
    fn refuses_out_of_range_iterations() {
        let bacon = std::iter::repeat("bacon").take(24).collect::<Vec<_>>().join(" ");
        for bad in [0u32, 99_999, 2_000_001, u32::MAX] {
            assert!(
                matches!(
                    seal_deterministic(
                        Payload::Bip39(bacon.clone()),
                        bad,
                        [0xbe; SALT_LEN],
                        [0xba; IV_LEN],
                        PASS_A
                    ),
                    Err(SealError::Iterations(_))
                ),
                "iterations {bad} must be refused"
            );
        }
        // The boundaries themselves are legal.
        for ok in [100_000u32, 2_000_000] {
            let r = seal_deterministic(
                Payload::Bip39(bacon.clone()),
                ok,
                [0xbe; SALT_LEN],
                [0xba; IV_LEN],
                PASS_A,
            );
            assert!(r.is_ok(), "iterations {ok} must be accepted");
        }
    }

    /// The six canonical records of vector C — a real bip84 bundle for the
    /// `bacon`x24 test seed. Regenerate with:
    ///   mnemonic bundle --network mainnet --template bip84 --group-size 0 \
    ///     --slot "@0.phrase=$(python3 -c "print(' '.join(['bacon']*24))")"
    /// NOTE `--group-size 0`. The default (5) emits a space-grouped DISPLAY
    /// form the engraver rejects outright; that was the R0 round-3 Critical.
    fn vector_c_records() -> Vec<String> {
        let recs: Vec<String> = [
            "ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw",
            "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g",
            "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8",
            "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3",
            "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374",
            "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(
            recs.iter().map(|r| r.len()).collect::<Vec<_>>(),
            vec![75, 111, 80, 67, 67, 67],
            "vector C records are not canonical — did you use --group-size 0?"
        );
        recs
    }
}
```

The six records are inlined in `vector_c_records()` above — no fixture file. The length assertion is the guard: if someone regenerates them with the default `--group-size 5`, the lengths become 89/133/95/80/80/80 and the test fails loudly instead of sealing a blob the device cannot read.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::tests`
Expected: FAIL — `cannot find function seal`.

- [ ] **Step 3: Implement**

Add to `crates/me-cli/src/seal/mod.rs`:

```rust
use rand::RngCore;
use zeroize::Zeroizing;

use crypto::CryptoError;
use wire::{Header, PayloadKind, HEADER_LEN, IV_LEN, SALT_LEN};

/// What is being sealed.
pub enum Payload {
    /// One md1 or mk1 string.
    MdMk(String),
    /// A BIP-39 mnemonic to engrave (not a passphrase).
    Bip39(String),
    /// One ms1 codex32 secret.
    Ms1(String),
    /// Several constellation records, one plate each.
    Bundle(Vec<String>),
}

pub struct Sealed {
    pub blob: Vec<u8>,
    pub passphrase: Zeroizing<String>,
}

#[derive(Debug)]
pub enum SealError {
    Container(container::ContainerError),
    Passphrase(String),
    Payload(String),
    Crypto(CryptoError),
    Iterations(u32),
    Empty,
    TooLarge(usize),
}

impl std::fmt::Display for SealError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SealError::Container(e) => write!(f, "{e}"),
            SealError::Passphrase(e) => write!(f, "{e}"),
            SealError::Payload(e) => write!(f, "{e}"),
            SealError::Crypto(e) => write!(f, "{e}"),
            SealError::Iterations(n) => write!(
                f,
                "iteration count {n} out of range [{}, {}]",
                wire::MIN_ITERATIONS,
                wire::MAX_ITERATIONS
            ),
            SealError::Empty => write!(f, "payload is empty"),
            SealError::TooLarge(n) => write!(
                f,
                "payload is {n} bytes; the ciphertext cap is {} (the device's scan buffer \
                 overflows when exactly full)",
                wire::MAX_CT_LEN
            ),
        }
    }
}
impl std::error::Error for SealError {}

/// Seal a payload. Salt, IV and passphrase are ALWAYS freshly generated — there
/// is deliberately no public seam to supply them. One fresh salt per call means
/// one key per message, which is what makes AES-GCM's nonce-uniqueness
/// requirement structurally unbreakable rather than a procedural promise.
pub fn seal(payload: Payload, iterations: u32) -> Result<Sealed, SealError> {
    let passphrase = passphrase::generate();
    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    rand::rng().fill_bytes(&mut salt);
    rand::rng().fill_bytes(&mut iv);
    let blob = seal_deterministic(payload, iterations, salt, iv, &passphrase)?;
    Ok(Sealed { blob, passphrase })
}

/// Deterministic seam for the canonical vectors ONLY.
///
/// `pub(crate)` and never re-exported. A public version of this destroys the
/// one-key-one-message property the moment a caller reuses a salt, and there is
/// no legitimate reason for a caller to choose one.
pub(crate) fn seal_deterministic(
    payload: Payload,
    iterations: u32,
    salt: [u8; SALT_LEN],
    iv: [u8; IV_LEN],
    passphrase: &str,
) -> Result<Vec<u8>, SealError> {
    if !passphrase::is_valid(passphrase) {
        return Err(SealError::Passphrase(
            "passphrase is not a checksum-valid BIP-39 mnemonic".into(),
        ));
    }
    // §6.2 bounds the iteration count, but `Header::encode` does not enforce it
    // and neither does clap on its own. Without this, `--iterations 5` emits a
    // blob the device provably rejects, and `--iterations 3000000000` burns
    // hours on the laptop before emitting one.
    if !(wire::MIN_ITERATIONS..=wire::MAX_ITERATIONS).contains(&iterations) {
        return Err(SealError::Iterations(iterations));
    }

    let (kind, plaintext) = match payload {
        Payload::MdMk(s) => {
            container::validate_record(&s).map_err(SealError::Container)?;
            (PayloadKind::MdMk, Zeroizing::new(s))
        }
        Payload::Ms1(s) => {
            container::validate_record(&s).map_err(SealError::Container)?;
            (PayloadKind::Ms1, Zeroizing::new(s))
        }
        // §9 and §11.1: "a 12/15/18/21/24-word BIP-39 mnemonic WITH A VALID
        // CHECKSUM → 0x02", and "a mislabelled input is refused at seal time,
        // not emitted." Without this the host emits a well-formed 0x02 blob for
        // any garbage, and the operator discovers it only after loading, typing
        // twelve words, and waiting out the KDF — as "payload unreadable",
        // which §2.2 item 4 has taught them to read as tampering.
        Payload::Bip39(s) => {
            let normalised = passphrase::normalise(&s);
            if bip39::Mnemonic::parse_in(bip39::Language::English, &*normalised).is_err() {
                return Err(SealError::Payload(
                    "not a checksum-valid BIP-39 mnemonic (12/15/18/21/24 words)".into(),
                ));
            }
            (PayloadKind::Bip39, normalised)
        }
        Payload::Bundle(records) => (
            PayloadKind::Bundle,
            container::encode_bundle(&records).map_err(SealError::Container)?,
        ),
    };

    let pt = plaintext.as_bytes();
    if pt.is_empty() {
        return Err(SealError::Empty);
    }
    if pt.len() > wire::MAX_CT_LEN as usize {
        return Err(SealError::TooLarge(pt.len()));
    }

    let header = Header { kind, iterations, salt, iv, ct_len: pt.len() as u32 };
    let aad = header.encode();
    let key = crypto::derive_key(&passphrase::normalise(passphrase), &salt, iterations);
    let sealed = crypto::seal_bytes(&key, &iv, &aad, pt).map_err(SealError::Crypto)?;

    let mut blob = Vec::with_capacity(HEADER_LEN + sealed.len());
    blob.extend_from_slice(&aad);
    blob.extend_from_slice(&sealed);
    Ok(blob)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib seal`
Expected: all pass. **If any vector hash mismatches, stop and reconcile against the spec before continuing** — the Go port binds to these bytes.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/mod.rs
git commit -m "seal: seal() plus the three canonical vectors and the freshness guard"
```

---

### Task 7: UF2 emission

**Files:**
- Create: `crates/me-cli/src/seal/uf2.rs`
- Modify: `crates/me-cli/src/seal/mod.rs`

**Interfaces:**
- Consumes: nothing from earlier tasks (operates on raw bytes).
- Produces: `uf2::to_uf2(blob: &[u8]) -> Vec<u8>`; `uf2::{TARGET_ADDR, FAMILY_DATA}`.

- [ ] **Step 1: Wire the module, then write the failing test**

Add `pub mod uf2;` to `crates/me-cli/src/seal/mod.rs` **first** — an undeclared file is not compiled and the RED step would falsely report `0 passed`.

Then create `crates/me-cli/src/seal/uf2.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn field(block: &[u8], off: usize) -> u32 {
        u32::from_le_bytes(block[off..off + 4].try_into().unwrap())
    }

    #[test]
    fn emits_conforming_blocks() {
        let blob = vec![0xABu8; 207];
        let uf2 = to_uf2(&blob);
        assert_eq!(uf2.len(), 512, "207 bytes fits one 256-byte block payload");
        assert_eq!(field(&uf2, 0), 0x0A32_4655, "magicStart0");
        assert_eq!(field(&uf2, 4), 0x9E5D_5157, "magicStart1");
        assert_eq!(field(&uf2, 8), 0x0000_2000, "familyID present, and nothing else");
        assert_eq!(field(&uf2, 12), TARGET_ADDR);
        assert_eq!(field(&uf2, 16), 256, "payloadSize is always 256");
        assert_eq!(field(&uf2, 20), 0, "blockNo");
        assert_eq!(field(&uf2, 24), 1, "numBlocks");
        assert_eq!(field(&uf2, 28), FAMILY_DATA, "must be data, NOT rp2350_arm_s");
        assert_eq!(field(&uf2, 508), 0x0AB1_6F30, "magicEnd");
    }

    #[test]
    fn pads_the_final_block_with_zeroes() {
        // The pinned vector sha256s assume 0x00 padding; 0xFF would not match.
        let uf2 = to_uf2(&vec![0xABu8; 207]);
        assert!(uf2[32 + 207..32 + 256].iter().all(|&b| b == 0));
    }

    #[test]
    fn every_block_conforms_not_just_the_first() {
        // 600 bytes = 3 blocks, the last one short. §9.1 requires payloadSize
        // 256 on EVERY block; an implementation writing chunk.len() for the
        // final short block would pass a first-block-only check.
        let uf2 = to_uf2(&vec![0xABu8; 600]);
        assert_eq!(uf2.len(), 3 * 512);
        for (i, chunk) in uf2.chunks(512).enumerate() {
            assert_eq!(field(chunk, 0), 0x0A32_4655, "block {i} magicStart0");
            assert_eq!(field(chunk, 4), 0x9E5D_5157, "block {i} magicStart1");
            assert_eq!(field(chunk, 8), 0x0000_2000, "block {i} flags");
            assert_eq!(field(chunk, 12), TARGET_ADDR + (i as u32 * 256), "block {i} addr");
            assert_eq!(field(chunk, 16), 256, "block {i} payloadSize must be 256");
            assert_eq!(field(chunk, 20), i as u32, "block {i} blockNo");
            assert_eq!(field(chunk, 24), 3, "block {i} numBlocks");
            assert_eq!(field(chunk, 28), FAMILY_DATA, "block {i} familyID");
            assert_eq!(field(chunk, 508), 0x0AB1_6F30, "block {i} magicEnd");
        }
        // The final block's 88 unused payload bytes are zero-padded.
        let last = &uf2[2 * 512..];
        assert!(last[32 + 88..32 + 256].iter().all(|&b| b == 0));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --lib seal::uf2`
Expected: FAIL — `cannot find function to_uf2`.

- [ ] **Step 3: Implement**

Prepend to `crates/me-cli/src/seal/uf2.rs`:

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

/// `data`. NOT 0xe48bff59 (`rp2350_arm_s`), which is the bootable-image family
/// the TinyGo target uses — correct for firmware, wrong for a blob.
pub const FAMILY_DATA: u32 = 0xE48B_FF58;

const MAGIC_START0: u32 = 0x0A32_4655;
const MAGIC_START1: u32 = 0x9E5D_5157;
const MAGIC_END: u32 = 0x0AB1_6F30;
const FLAG_FAMILY_ID_PRESENT: u32 = 0x0000_2000;
const PAYLOAD: usize = 256;

pub fn to_uf2(blob: &[u8]) -> Vec<u8> {
    debug_assert!(!blob.is_empty(), "a blob is always >= 65 bytes");
    let num_blocks = blob.len().div_ceil(PAYLOAD) as u32;
    let mut out = Vec::with_capacity(num_blocks as usize * 512);
    for (i, chunk) in blob.chunks(PAYLOAD).enumerate() {
        let mut b = [0u8; 512];
        b[0..4].copy_from_slice(&MAGIC_START0.to_le_bytes());
        b[4..8].copy_from_slice(&MAGIC_START1.to_le_bytes());
        b[8..12].copy_from_slice(&FLAG_FAMILY_ID_PRESENT.to_le_bytes());
        b[12..16].copy_from_slice(&(TARGET_ADDR + (i as u32 * PAYLOAD as u32)).to_le_bytes());
        // Always 256, even for a short final chunk: the bootrom requires it, and
        // the device bounds every read by ct_len so the padding is never seen.
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


- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --lib seal::uf2`
Expected: 3 passed.

- [ ] **Step 5: Verify against the real vector**

Add to the `seal::tests` module and run it:

```rust
    #[test]
    fn vector_a_uf2_matches_the_pinned_hash() {
        let pt = std::iter::repeat("bacon").take(24).collect::<Vec<_>>().join(" ");
        let blob = seal_deterministic(
            Payload::Bip39(pt),
            100_000,
            [0xbe, 0xef].repeat(8).try_into().unwrap(),
            [0xba, 0xc0].repeat(6).try_into().unwrap(),
            PASS_A,
        )
        .unwrap();
        assert_eq!(
            sha256_hex(&uf2::to_uf2(&blob)),
            "c58b684e6d206f599f4a3408e626534af0ce914aa157a93d9e05ab62cc2865fc"
        );
    }
```

Run: `cargo test -p mnemonic-engrave --lib seal`
Expected: PASS. This cross-checks the UF2 writer against a file `picotool` produced.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy -p mnemonic-engrave -- -D warnings
git add crates/me-cli/src/seal/uf2.rs crates/me-cli/src/seal/mod.rs
git commit -m "seal: data-family UF2 emission at 0x10E00000"
```

---

### Task 8: `me seal` CLI

**Files:**
- Modify: `crates/me-cli/src/main.rs`
- Create: `crates/me-cli/tests/seal_cli.rs`

**Interfaces:**
- Consumes: `seal::{seal, Payload, Sealed}`, `seal::uf2::to_uf2`.
- Produces: the `me seal` subcommand.

- [ ] **Step 1: Write the failing test**

Create `crates/me-cli/tests/seal_cli.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

const MD1: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

fn me() -> Command {
    Command::cargo_bin("me").unwrap()
}

#[test]
fn seals_a_single_record_and_prints_the_passphrase_to_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("payload.uf2");
    let assert = me()
        .args(["seal", MD1, "--out", out.to_str().unwrap()])
        .assert()
        .success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.lines().any(|l| l.split_whitespace().count() == 12),
        "the 12-word passphrase must be printed to stderr, got: {stderr}"
    );
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(bytes.len() % 512, 0, "output must be whole UF2 blocks");

    // §2.3: the passphrase must never land beside the ciphertext it opens.
    let words: Vec<&str> = stderr
        .lines()
        .find(|l| l.split_whitespace().count() == 12)
        .unwrap()
        .split_whitespace()
        .collect();
    assert!(
        !String::from_utf8_lossy(&bytes).contains(words[0]),
        "no passphrase word may appear in the UF2 file"
    );
}

#[test]
fn there_is_no_passphrase_flag() {
    // §8 / §2.2a: the prohibition on a user-supplied passphrase is load-bearing,
    // not advisory. Assert the flag is ABSENT — asserting only `.failure()`
    // would also pass if someone added `--passphrase` with validation that
    // happened to reject "hunter2".
    me().args(["seal", MD1, "--passphrase", "hunter2"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
    me().args(["seal", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--passphrase").not());
}

#[test]
fn there_is_no_addr_flag() {
    // §9: a wrong address either produces a blob the device never reads or,
    // past 0x11000000, wraps and destroys the signed firmware. Same reasoning
    // as above — assert absence, not merely failure.
    me().args(["seal", MD1, "--addr", "0x10000000"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
    me().args(["seal", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--addr").not());
}

#[test]
fn refuses_ms1_without_the_opt_in_flag() {
    const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    me().args(["seal", MS1, "--out", out.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--seal-secret"));
    assert!(!out.exists(), "nothing may be written on the refusal path");
}

#[test]
fn seals_ms1_with_the_opt_in_flag() {
    const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    me().args(["seal", MS1, "--out", out.to_str().unwrap(), "--seal-secret"])
        .assert()
        .success();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(bytes[32 + 11], 0x03, "payload_kind must be ms1");
}

#[test]
fn refuses_a_checksum_invalid_bip39_payload() {
    // §11.1: "a mislabelled input is refused at seal time, not emitted."
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    me().args(["seal", "hunter2 please", "--out", out.to_str().unwrap()])
        .assert()
        .failure();
    assert!(!out.exists());
}

#[test]
fn refuses_out_of_range_iterations() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    for bad in ["5", "3000000000"] {
        me().args(["seal", MD1, "--out", out.to_str().unwrap(), "--iterations", bad])
            .assert()
            .failure();
    }
}

#[test]
fn refuses_space_grouped_input_with_an_actionable_message() {
    me().args(["seal", "md1yqp qqxqq 8xtwh w4xwn 4qh"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--group-size 0"));
}

#[test]
fn writes_the_uf2_with_owner_only_permissions() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("p.uf2");
        me().args(["seal", MD1, "--out", out.to_str().unwrap()])
            .assert()
            .success();
        let mode = std::fs::metadata(&out).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "sealed output must be 0600");
    }
}
```

Add to `[dev-dependencies]` in `crates/me-cli/Cargo.toml`:

```toml
tempfile = "3"
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p mnemonic-engrave --test seal_cli`
Expected: FAIL — `me` has no `seal` subcommand.

- [ ] **Step 3: Implement the subcommand**

The real enum is `Command` (singular, `main.rs:39`), and `run()` returns `i32` with **no `match`** — dispatch is a series of early returns (`main.rs:74-82`). `?` is a compile error in `run()`. Mirror `run_bundle_cli` (`main.rs:164`) exactly.

Add to `enum Command` in `crates/me-cli/src/main.rs`:

```rust
    /// Encrypt a payload for delivery to SeedHammer II flash.
    ///
    /// The passphrase is GENERATED and printed to STDERR — write it down and
    /// store it apart from the machine. It is never written to a file, and
    /// there is deliberately no way to supply your own: total strength is the
    /// passphrase plus about 20 bits from the KDF, and a memorable passphrase
    /// does not survive an offline attack on a stolen machine.
    Seal {
        /// One md1/mk1/ms1 string, a BIP-39 mnemonic, or (repeated) the records
        /// of a bundle. Records must be canonical: if they came from
        /// `mnemonic bundle`, use --group-size 0.
        #[arg(required = true)]
        payload: Vec<String>,

        /// Write the UF2 here. Created 0600. REQUIRED — never stdout, because
        /// the passphrase shares that stream.
        #[arg(long, required = true)]
        out: PathBuf,

        /// Required to seal ms1 (a seed). Without it, an ms1 input — standalone
        /// or as a bundle record — is refused. §9: sealing a seed must never be
        /// accidental.
        #[arg(long)]
        seal_secret: bool,

        /// PBKDF2 iterations. The default targets ~30 s on device and MUST be
        /// re-measured on real hardware before release.
        #[arg(long, default_value_t = 450_000)]
        iterations: u32,
    },
```

Add the early return in `run()`, immediately after the existing `Command::Bundle` block (`main.rs:82`):

```rust
    if let Some(Command::Seal { payload, out, seal_secret, iterations }) = &cli.command {
        return run_seal_cli(payload, out, *seal_secret, *iterations);
    }
```

Add the handler alongside `run_bundle_cli`:

```rust
fn run_seal_cli(payload: &[String], out: &PathBuf, seal_secret: bool, iterations: u32) -> i32 {
    use mnemonic_engrave::classify::{classify, Format};
    use mnemonic_engrave::seal::{self, Payload};

    // Move the secrets out of clap's Vec into scrubbed buffers straight away.
    let records: Vec<Zeroizing<String>> = payload
        .iter()
        .map(|s| Zeroizing::new(s.trim().to_string()))
        .collect();

    // §9: ms1 needs the explicit opt-in, whether standalone or inside a bundle.
    // Checked on classification, not on a flag the caller passed us.
    let has_secret = records
        .iter()
        .any(|r| matches!(classify(r), Ok(Format::Ms)));
    if has_secret && !seal_secret {
        eprintln!(
            "me: refusing to seal ms1 without --seal-secret.\n    \
             ms1 is seed entropy. Sealing it puts an offline-attackable ciphertext of your \
             seed into the machine's flash, defended only by the generated passphrase.\n    \
             Re-run with --seal-secret if that is what you intend."
        );
        return EXIT_REFUSED;
    }

    let p = if records.len() > 1 {
        Payload::Bundle(records.iter().map(|r| r.to_string()).collect())
    } else {
        let s = records[0].to_string();
        match classify(&s) {
            Ok(Format::Ms) => Payload::Ms1(s),
            Ok(_) => Payload::MdMk(s),
            // Not a constellation string — try it as a BIP-39 mnemonic to
            // engrave. seal() checks the checksum and refuses if it is not one.
            Err(_) => Payload::Bip39(s),
        }
    };

    let sealed = match seal::seal(p, iterations) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("me: {e}");
            return match e {
                seal::SealError::Container(_)
                | seal::SealError::Payload(_)
                | seal::SealError::Passphrase(_) => EXIT_INVALID,
                _ => EXIT_USAGE,
            };
        }
    };

    let uf2 = seal::uf2::to_uf2(&sealed.blob);
    if let Err(e) = write_private(out, &uf2) {
        eprintln!("me: cannot write {}: {e}", out.display());
        return EXIT_USAGE;
    }

    // STDERR, always. The passphrase must never land in a redirected file
    // beside the ciphertext it opens (§2.3).
    eprintln!("me: wrote {} bytes to {}", uf2.len(), out.display());
    eprintln!();
    eprintln!("passphrase — write this down and store it APART from the machine:");
    eprintln!();
    eprintln!("    {}", &*sealed.passphrase);
    eprintln!();
    eprintln!("load:  picotool load --verify {}   (machine in BOOTSEL)", out.display());
    eprintln!("wipe:  picotool erase -r 0x10E00000 0x10E10000");
    EXIT_OK
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p mnemonic-engrave --test seal_cli`
Expected: 9 passed.

- [ ] **Step 5: Run the whole suite**

Run: `cargo test -p mnemonic-engrave && cargo clippy -p mnemonic-engrave -- -D warnings`
Expected: everything green, no warnings. Confirm the pre-existing `convert`/`bundle` tests still pass — Task 4 touched `validate.rs`.

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add crates/me-cli/src/main.rs crates/me-cli/tests/seal_cli.rs crates/me-cli/Cargo.toml Cargo.lock
git commit -m "seal: me seal subcommand, generated passphrase to stdout only"
```

---

## Mutation testing (required before Plan A is called done)

Per §11.3 and project standard, a green suite proves little. Break the code and confirm a test notices. **Every mutant below must have a test that fails.** A mutant with no killer is a gap in the suite, not a pass.

Procedure, both rules non-negotiable:
1. **Copy the file first** (`cp file.rs file.rs.bak`); restore from the copy, **never** `git checkout` — that has reverted real uncommitted work before.
2. **Assert the substitution actually matched** before running the test. A silently-failing `sed` reads exactly like a surviving mutation and has produced false "0 failures" in this project twice.

| Mutant | Must be killed by |
| --- | --- |
| `derive_key` ignores `iterations`, hardcodes `100_000` | `vector_b` |
| `seal` reuses a fixed salt | `two_seals_of_the_same_payload_differ_everywhere` |
| `encode_bundle` appends a trailing `\n` | `encodes_records_lf_separated_without_trailing_lf` |
| `validate_record` strips whitespace instead of refusing | `refuses_space_grouped_records` |
| `Header::decode` skips the `ct_len` bound | `rejects_out_of_range_ct_len` |
| `Header::decode` drops the `kdf_id` or `aead_id` check | `rejects_unknown_kind_version_kdf_and_aead` |
| `Header::decode` drops the length check | `rejects_a_short_buffer` |
| `seal_deterministic` skips the iterations range check | `refuses_out_of_range_iterations` |
| `seal_deterministic` emits an unvalidated BIP-39 payload | `refuses_a_checksum_invalid_bip39_payload` |
| the `ms1` opt-in is ignored | `refuses_ms1_without_the_opt_in_flag` |
| `payload_kind` discriminants swapped | `payload_kind_byte_matches_the_input_shape` |
| `to_uf2` emits family `0xE48BFF59` | `emits_conforming_blocks` |
| `to_uf2` pads with `0xFF` | `pads_the_final_block_with_zeroes` |
| `open_bytes` returns plaintext without checking the tag | `open_fails_on_flipped_ciphertext_byte` |
| `encode_bundle` accepts 25 records | `refuses_empty_and_oversized_record_lists` |

Record the results in `design/agent-reports/`.

---

## What Plan A does NOT cover

- **All firmware work.** XIP read, header parse on device, the §10.2.1 classifier allow-list, the bundle session and its wipe-on-every-exit rule, the plate list. That is **Plan B**, which binds to the vectors this plan emits and may not lead them (Rust-primary rule).
- **§12 item 1** — the real PBKDF2 iteration count. The 450,000 default is an estimate derived from this project's own SLIP-39 anchor and MUST be measured on hardware before release.
- **§12 item 8** — the session idle-wipe value, which is a firmware concern.
- **§12 item 3** — MSD drag-and-drop, untested. `picotool load` is the documented path.
- **`me bundle` integration.** `bundle.rs` returns `BundleError::RefusedSecret` on any `ms1` line. Reconciling that with §12 item 6's admission is deliberate follow-up work: the refusal must be lifted **only** on the sealed path, never on the plaintext NDEF one.

---

## Self-review notes

**Spec coverage.** §6.2 bounds → Task 1. §6.4 container → Tasks 4–5. §6.5 (structure belongs in the plaintext) → Task 5's placement. §7 crypto → Task 2. §7.2 one-key-one-message → Task 6's `pub(crate)` seam plus the freshness test. §7.3 randomness host-side → Tasks 3 and 6. §8 passphrase → Task 3. §9 host CLI → Tasks 7–8. §11.1 Rust tests → distributed, with §11.4's vectors in Task 6. §11.3 mutation → the section above.

**Known gap, deliberate:** §11.1's "no two shipped vectors share a `(key, iv)` pair" is asserted over the three vectors that exist today (Task 6). If a fourth is ever added, that test must be extended — it enumerates explicitly rather than deriving the set, because a derived set would silently pass when someone forgets to register a vector.
