//! The systemwide payload container — SPEC_systemwide_payloads.
//!
//! Separate from [`crate::seal`] on purpose, and the separation is the design:
//! the operator froze Sealed Payload (spec decision 1), so widening the format
//! it depends on would unfreeze it through the back door. Two containers, two
//! magics, two flash regions a megabyte apart, no shared state.
//!
//! What IS shared is deliberate and narrow: `seal::crypto` for PBKDF2 and
//! AES-256-GCM, and `seal::passphrase::normalise`, because spec §8a requires
//! host and device to produce byte-identical KDF input and a second
//! normalisation would be a second answer to that question.

pub mod identity;
pub mod overwrite;
pub mod pubhash;
pub mod record;
pub mod wire;

use zeroize::Zeroizing;

/// `[cliff]` — SPEC_systemwide_payloads §12.1.
///
/// Five or more whitespace-separated tokens, every one a BIP-39 English
/// wordlist entry. A pure function of the normalised string, so host and device
/// agree with no shared state and nothing attacker-controlled.
///
/// **A SPEED BUMP, NOT A STRENGTH MEASURE.** `abandon` five times is five
/// wordlist tokens, zero entropy, and above it. That is deliberate: these
/// programs are the lower-assurance branch.
///
/// It lives here rather than in `seal` because `seal`'s passphrase rules are
/// frozen and this is a different container's rule.
pub fn cliff_above(normalised: &str) -> bool {
    let mut n = 0usize;
    for tok in normalised.split_whitespace() {
        if bip39::Language::English.find_word(tok).is_none() {
            return false;
        }
        n += 1;
    }
    n >= 5
}

/// A payload's records, split the way the container stores them.
#[derive(Debug, Default)]
pub struct Payload {
    /// Cleartext on the wire, whatever their class. In an UNSEALED payload this
    /// is everything; in a sealed one it is the non-secret records.
    pub public: Vec<String>,
    /// Encrypted. Empty in an unsealed payload.
    pub secret: Vec<Zeroizing<String>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SyswError {
    Wire(wire::WireError),
    /// A record `classify` cannot place. Fails CLOSED: better refused at
    /// creation with a name than mis-filed into the wrong section.
    Unclassifiable(usize),
    /// Sections exceed what the region can hold.
    TooLarge(usize),
    Crypto,
    /// A sealed payload was handed no passphrase, or vice versa.
    PassphraseMismatch,
    NotUtf8,
}

/// Which section a record belongs in.
///
/// **Descriptor and Address are deliberately absent**, and this is a known
/// limitation rather than an oversight: classifying them needs a descriptor
/// parser and an address decoder, neither of which is a dependency of this
/// crate. An unclassifiable record is REFUSED at pack time with its index, so
/// the failure is a named error at creation rather than a mis-filed secret.
pub fn classify(record: &str) -> record::Class {
    use record::Class;
    if record.starts_with(record::PASS_PREFIX) {
        return if record::decode_body(record).is_ok() {
            Class::Passphrase
        } else {
            Class::Unknown
        };
    }
    if record.starts_with(record::TEXT_PREFIX) {
        return if record::decode_body(record).is_ok() {
            Class::FreeText
        } else {
            Class::Unknown
        };
    }
    if bip39::Mnemonic::parse_normalized(record).is_ok() {
        return Class::Mnemonic;
    }
    match crate::seal::record::validate_record(record) {
        Ok(crate::seal::record::RecordKind::Ms) => Class::Codex32Secret,
        Ok(_) => Class::MdMk,
        Err(_) => Class::Unknown,
    }
}

/// Build a container. `passphrase` `None` produces the unsealed variant.
///
/// The AAD is `header ‖ public section` — EPD §6.1a, and spec §5.4 states it for
/// this container explicitly rather than by reference. Binding only the
/// ciphertext's own framing would let an attacker swap a public record for one
/// encoding THEIR xpub, with the tag still verifying, and the operator would
/// engrave a steel backup of a wallet they do not control.
pub fn pack(
    records: Vec<String>,
    passphrase: Option<&str>,
    iterations: u32,
) -> Result<Vec<u8>, SyswError> {
    let mut payload = Payload::default();
    for (i, r) in records.into_iter().enumerate() {
        match classify(&r) {
            record::Class::Unknown => return Err(SyswError::Unclassifiable(i)),
            c if c.is_secret() && passphrase.is_some() => payload.secret.push(Zeroizing::new(r)),
            _ => payload.public.push(r),
        }
    }
    let pub_bytes = payload.public.join("\n").into_bytes();
    let mut header = wire::Header {
        pub_len: pub_bytes.len() as u32,
        ..Default::default()
    };

    let Some(pass) = passphrase else {
        // No `secret.is_empty()` check here: it cannot be non-empty. The match
        // above only fills `secret` when a passphrase exists, so an unsealed
        // pack necessarily has everything in `public` — including secret
        // CLASSES, which decision 6 permits and F1 flags at load. An earlier
        // draft had a refusal here; it was dead code enforcing a rule §13 had
        // already demoted.
        let mut blob = header.encode().to_vec();
        blob.extend_from_slice(&pub_bytes);
        return bound(blob);
    };

    let secret: Vec<&str> = payload.secret.iter().map(|s| s.as_str()).collect();
    let plaintext = Zeroizing::new(secret.join("\n").into_bytes());
    let mut salt = [0u8; wire::SALT_LEN];
    let mut iv = [0u8; wire::IV_LEN];
    rand::RngCore::fill_bytes(&mut rand::rng(), &mut salt);
    rand::RngCore::fill_bytes(&mut rand::rng(), &mut iv);
    header.iterations = iterations;
    header.salt = salt;
    header.iv = iv;
    header.ct_len = plaintext.len() as u32;

    // AAD is taken from the ASSEMBLED bytes, never re-encoded, so what is bound
    // is exactly what a reader will parse.
    let mut blob = header.encode().to_vec();
    blob.extend_from_slice(&pub_bytes);
    let aad = blob.clone();
    let normalised = crate::seal::passphrase::normalise(pass);
    let key = crate::seal::crypto::derive_key(&normalised, &salt, iterations);
    let sealed = crate::seal::crypto::seal_bytes(&key, &iv, &aad, &plaintext)
        .map_err(|_| SyswError::Crypto)?;
    blob.extend_from_slice(&sealed);
    bound(blob)
}

fn bound(blob: Vec<u8>) -> Result<Vec<u8>, SyswError> {
    if blob.len() > wire::REGION_LEN {
        return Err(SyswError::TooLarge(blob.len()));
    }
    Ok(blob)
}

/// Parse and, if sealed, decrypt.
pub fn open(blob: &[u8], passphrase: Option<&str>) -> Result<Payload, SyswError> {
    let h = wire::Header::parse(blob).map_err(SyswError::Wire)?;
    if blob.len() < h.total_len() {
        return Err(SyswError::Wire(wire::WireError::TooShort(blob.len())));
    }
    let pub_end = wire::HEADER_LEN + h.pub_len as usize;
    let public_bytes = &blob[wire::HEADER_LEN..pub_end];
    let public = split_records(public_bytes)?;

    if !h.sealed() {
        return Ok(Payload {
            public,
            secret: Vec::new(),
        });
    }
    let Some(pass) = passphrase else {
        return Err(SyswError::PassphraseMismatch);
    };
    let aad = &blob[..pub_end];
    let ct = &blob[pub_end..h.total_len()];
    let normalised = crate::seal::passphrase::normalise(pass);
    let key = crate::seal::crypto::derive_key(&normalised, &h.salt, h.iterations);
    let pt =
        crate::seal::crypto::open_bytes(&key, &h.iv, aad, ct).map_err(|_| SyswError::Crypto)?;
    let secret = split_records(&pt)?
        .into_iter()
        .map(Zeroizing::new)
        .collect();
    Ok(Payload { public, secret })
}

fn split_records(b: &[u8]) -> Result<Vec<String>, SyswError> {
    let s = std::str::from_utf8(b).map_err(|_| SyswError::NotUtf8)?;
    if s.is_empty() {
        return Ok(Vec::new());
    }
    Ok(s.split('\n').map(str::to_owned).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASS: &str = "abandon abandon abandon abandon abandon abandon";
    const ITER: u32 = wire::MIN_ITERATIONS;

    fn md1() -> String {
        "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3".into()
    }

    #[test]
    fn unsealed_round_trips_and_has_no_ciphertext() {
        let recs = vec![record::encode_text("Hello, World!"), md1()];
        let blob = pack(recs.clone(), None, ITER).unwrap();
        let h = wire::Header::parse(&blob).unwrap();
        assert!(!h.sealed(), "no passphrase means no ciphertext");
        assert_eq!(h.ct_len, 0);
        let p = open(&blob, None).unwrap();
        assert_eq!(p.public, recs);
        assert!(p.secret.is_empty());
    }

    #[test]
    fn sealed_round_trips_with_the_secret_encrypted() {
        let secret = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let recs = vec![md1(), secret.to_string()];
        let blob = pack(recs, Some(PASS), ITER).unwrap();
        let h = wire::Header::parse(&blob).unwrap();
        assert!(h.sealed());
        assert!(h.pub_len > 0, "the md1 stays public");
        assert!(
            !blob.windows(secret.len()).any(|w| w == secret.as_bytes()),
            "the secret must not appear in the blob in the clear"
        );
        let p = open(&blob, Some(PASS)).unwrap();
        assert_eq!(p.public, vec![md1()]);
        assert_eq!(p.secret.len(), 1);
        assert_eq!(&**p.secret[0], secret);
    }

    /// The case that cost this cycle three R0 findings: pub_len == 0, so there
    /// is no digest to show and opening is what authenticates it.
    #[test]
    fn a_secrets_only_sealed_payload_has_no_public_section_and_still_opens() {
        let secret = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let blob = pack(vec![secret.to_string()], Some(PASS), ITER).unwrap();
        let h = wire::Header::parse(&blob).unwrap();
        assert_eq!(h.pub_len, 0, "nothing public");
        assert!(h.sealed());
        let p = open(&blob, Some(PASS)).unwrap();
        assert_eq!(&**p.secret[0], secret);
    }

    #[test]
    fn a_wrong_passphrase_fails_and_a_missing_one_is_named() {
        let secret = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let blob = pack(vec![secret.to_string()], Some(PASS), ITER).unwrap();
        assert_eq!(
            open(&blob, Some("wrong words here friend now")).unwrap_err(),
            SyswError::Crypto
        );
        assert_eq!(
            open(&blob, None).unwrap_err(),
            SyswError::PassphraseMismatch
        );
    }

    /// **The AAD binding, exhaustively.** Every byte of `header ‖ public
    /// section` must be covered, or an attacker swaps a public record for one
    /// encoding THEIR xpub and the tag still verifies.
    ///
    /// Run at the crypto layer rather than through `open`, deliberately: `open`
    /// derives a key per call, so a byte-by-byte sweep would pay a 100,000-round
    /// KDF for each of ~90 positions. Here the key is derived ONCE and every
    /// position is checked. `open_wires_the_same_aad_in` below is what ties this
    /// to the real entry point.
    #[test]
    fn every_byte_of_the_aad_is_bound() {
        let secret = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let blob = pack(vec![md1(), secret.to_string()], Some(PASS), ITER).unwrap();
        let h = wire::Header::parse(&blob).unwrap();
        let pub_end = wire::HEADER_LEN + h.pub_len as usize;
        let key = crate::seal::crypto::derive_key(
            &crate::seal::passphrase::normalise(PASS),
            &h.salt,
            h.iterations,
        );
        let ct = &blob[pub_end..h.total_len()];

        assert!(
            crate::seal::crypto::open_bytes(&key, &h.iv, &blob[..pub_end], ct).is_ok(),
            "the unmutated AAD must open"
        );
        for i in 0..pub_end {
            let mut aad = blob[..pub_end].to_vec();
            aad[i] ^= 0x01;
            assert!(
                crate::seal::crypto::open_bytes(&key, &h.iv, &aad, ct).is_err(),
                "byte {i} of header-plus-public-section is not bound into the AAD"
            );
        }
    }

    /// Ties the exhaustive crypto-layer check above to the real entry point:
    /// altering a public record must make `open` fail, and the alteration is to
    /// another VALID record so the payload is not merely refused structurally
    /// before the AEAD ever runs.
    #[test]
    fn open_wires_the_same_aad_in() {
        let secret = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let a = record::encode_text("aaaa");
        let b = record::encode_text("bbbb");
        assert_eq!(a.len(), b.len(), "same length, so only the CONTENT differs");
        let blob = pack(vec![a.clone(), secret.to_string()], Some(PASS), ITER).unwrap();
        let at = blob
            .windows(a.len())
            .position(|w| w == a.as_bytes())
            .unwrap();
        let mut tampered = blob.clone();
        tampered[at..at + b.len()].copy_from_slice(b.as_bytes());
        assert_eq!(
            open(&tampered, Some(PASS)).unwrap_err(),
            SyswError::Crypto,
            "a valid-for-valid public swap must fail the tag"
        );
    }

    #[test]
    fn an_unclassifiable_record_is_refused_with_its_index() {
        assert_eq!(
            pack(vec![md1(), "not a record".into()], None, ITER),
            Err(SyswError::Unclassifiable(1))
        );
    }

    #[test]
    fn the_sealed_payload_magic_is_refused_here() {
        let mut blob = pack(vec![md1()], None, ITER).unwrap();
        blob[..8].copy_from_slice(b"MNEMBLOB");
        assert_eq!(
            open(&blob, None).unwrap_err(),
            SyswError::Wire(wire::WireError::BadMagic)
        );
    }

    /// A secret with NO passphrase goes in the cleartext section, and that is
    /// correct: spec decision 6 permits the plaintext variant to carry secret
    /// classes, flagged by F1 at load rather than refused at creation.
    ///
    /// The first version of this test asserted a REFUSAL and failed. My test was
    /// wrong, not the code — and wrong in a specific direction worth recording:
    /// it re-imposed exactly the blocking security machinery §13 demoted. Test
    /// expectations drift back toward the stricter rule unless the demotion is
    /// asserted, so this asserts it.
    #[test]
    fn a_secret_without_a_passphrase_is_carried_in_the_clear_not_refused() {
        let secret = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let blob = pack(vec![secret.to_string()], None, ITER).unwrap();
        let h = wire::Header::parse(&blob).unwrap();
        assert!(!h.sealed());
        assert!(h.pub_len > 0, "it is carried, in the public section");
        let p = open(&blob, None).unwrap();
        assert_eq!(p.public, vec![secret.to_string()]);
        assert_eq!(
            classify(&p.public[0]),
            record::Class::Mnemonic,
            "still classifies as secret, so F1 has something to flag"
        );
    }

    #[test]
    fn five_wordlist_tokens_are_above_even_when_degenerate() {
        // The documented consequence, asserted so nobody "fixes" it later.
        assert!(cliff_above("abandon abandon abandon abandon abandon"));
    }

    #[test]
    fn four_are_below() {
        assert!(!cliff_above("abandon abandon abandon abandon"));
    }

    /// Measured against the real wordlist: `correct` and `horse` are entries,
    /// `battery` and `staple` are NOT. The spec used this phrase for months as
    /// its illustration of a four-word passphrase; it is 2-of-4.
    #[test]
    fn the_famous_phrase_is_below_because_two_of_its_words_are_not_bip39() {
        assert!(bip39::Language::English.find_word("correct").is_some());
        assert!(bip39::Language::English.find_word("horse").is_some());
        assert!(bip39::Language::English.find_word("battery").is_none());
        assert!(bip39::Language::English.find_word("staple").is_none());
        assert!(!cliff_above("correct horse battery staple"));
        // Even padded past five tokens, a non-wordlist token keeps it below.
        assert!(!cliff_above("correct horse battery staple abandon abandon"));
    }

    #[test]
    fn every_user_entered_non_bip39_password_is_below() {
        for p in [
            "Tr0ub4dor&3",
            "hunter2",
            "a b c d e",
            "abandon abandon abandon abandon zzzz",
        ] {
            assert!(!cliff_above(p), "{p}");
        }
    }

    #[test]
    fn empty_is_below() {
        assert!(!cliff_above(""));
    }
}
