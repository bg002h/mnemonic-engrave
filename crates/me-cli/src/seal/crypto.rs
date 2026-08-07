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
            CryptoError::Authentication => {
                write!(f, "wrong passphrase, or this payload has been altered")
            }
            CryptoError::TooShort => write!(f, "sealed payload too short"),
        }
    }
}
impl std::error::Error for CryptoError {}

/// `iterations` always comes from the header — never a constant, or vector B fails.
pub fn derive_key(passphrase: &str, salt: &[u8; SALT_LEN], iterations: u32) -> Zeroizing<[u8; 32]> {
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
    Aes256Gcm::new(key.into())
        .encrypt(
            Nonce::from_slice(iv),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CryptoError::Authentication)
}

/// Verify then decrypt. `aes-gcm` returns an error without releasing plaintext
/// on tag mismatch, which is what makes it safe to parse the §6.4 container out
/// of the result.
pub fn open_bytes(
    key: &[u8; 32],
    iv: &[u8; IV_LEN],
    aad: &[u8],
    sealed: &[u8],
) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if sealed.len() < TAG_LEN {
        return Err(CryptoError::TooShort);
    }
    Aes256Gcm::new(key.into())
        .decrypt(Nonce::from_slice(iv), Payload { msg: sealed, aad })
        .map(Zeroizing::new)
        .map_err(|_| CryptoError::Authentication)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASS: &str = "beef beef beef beef beef beef beef beef beef beef beef beef";
    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

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
        assert_eq!(
            &*open_bytes(&[7u8; 32], &[9u8; 12], b"aad", &sealed).unwrap(),
            b"plaintext"
        );
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
