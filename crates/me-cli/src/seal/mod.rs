//! `me seal` — encrypt a constellation payload for delivery to SeedHammer II
//! flash. See design/SPEC_encrypted_payload_delivery.md.

pub mod container;
pub mod crypto;
pub mod passphrase;
pub mod pubhash;
pub mod record;
pub mod uf2;
pub mod wire;

use rand::RngCore;
use zeroize::Zeroizing;

use crypto::CryptoError;
use wire::{Header, HEADER_LEN, IV_LEN, MAX_ITERATIONS, MAX_SECTION_LEN, MIN_ITERATIONS, SALT_LEN};

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
                f,
                "iteration count {n} out of range [{MIN_ITERATIONS}, {MAX_ITERATIONS}]"
            ),
            SealError::SecretInPublic(i) => write!(
                f,
                "record {i} is secret material and cannot ride in the public section — \
                 it would be engraved and readable in the clear (§6.3)"
            ),
            SealError::Empty => write!(f, "payload is empty"),
            SealError::TooLarge(n) => {
                write!(f, "section is {n} bytes; the cap is {MAX_SECTION_LEN}")
            }
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
        return Ok(Sealed {
            blob: seal_public_only(payload.public)?,
            passphrase: None,
        });
    }
    let passphrase = passphrase::generate();
    let mut salt = [0u8; SALT_LEN];
    let mut iv = [0u8; IV_LEN];
    rand::rng().fill_bytes(&mut salt);
    rand::rng().fill_bytes(&mut iv);
    let blob = seal_deterministic(payload, iterations, salt, iv, &passphrase)?;
    Ok(Sealed {
        blob,
        passphrase: Some(passphrase),
    })
}

/// Validate the public section: no secrets, and it must decode as a card set.
fn check_public(public: &[String]) -> Result<(), SealError> {
    if public.is_empty() {
        return Ok(());
    }
    for (i, r) in public.iter().enumerate() {
        // A BIP-39 mnemonic is not a constellation record, so `validate_record`
        // reports it as non-canonical and suggests `--group-size 0` — which
        // misdiagnoses it. Check for a mnemonic first and say the real reason.
        if passphrase::is_valid(r) {
            return Err(SealError::SecretInPublic(i));
        }
        if record::validate_record(r)
            .map_err(SealError::Record)?
            .is_secret()
        {
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
        iterations: 0,
        salt: [0; SALT_LEN],
        iv: [0; IV_LEN],
        pub_len: bytes.len() as u32,
        ct_len: 0,
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
            "passphrase is not a checksum-valid BIP-39 mnemonic".into(),
        ));
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
        return Err(SealError::Container(
            container::ContainerError::RecordCount(payload.public.len() + payload.secret.len()),
        ));
    }

    let header = Header {
        iterations,
        salt,
        iv,
        pub_len: pb.len() as u32,
        ct_len: sb.len() as u32,
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
    // §6.4's all-lowercase rule binds BOTH sections, and §9 says `me seal` must
    // "refuse rather than emit". `passphrase::is_valid` lowercases via
    // `normalise` before parsing, so without this check an UPPERCASE mnemonic
    // validates and is then emitted verbatim by `encode_section` — the device's
    // case-sensitive parse rejects it and the operator gets "payload
    // unreadable" after a ~31 s KDF.
    if let Some(pos) = s
        .char_indices()
        .find(|(_, c)| c.is_uppercase())
        .map(|(i, _)| i)
    {
        return Err(SealError::Record(record::RecordError::NotLowercase(pos)));
    }
    if passphrase::is_valid(s) {
        return Ok(());
    }
    Err(SealError::Record(record_err))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASS: &str = "beef beef beef beef beef beef beef beef beef beef beef beef";

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
    fn sha(b: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        hex(&Sha256::digest(b))
    }
    fn bacon24() -> String {
        std::iter::repeat_n("bacon", 24)
            .collect::<Vec<_>>()
            .join(" ")
    }

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
    fn salt(hexpair: [u8; 2]) -> [u8; 16] {
        hexpair.repeat(8).try_into().unwrap()
    }
    fn iv(hexpair: [u8; 2]) -> [u8; 12] {
        hexpair.repeat(6).try_into().unwrap()
    }

    #[test]
    fn vector_a_bacon24_fully_encrypted() {
        let b = seal_deterministic(
            Payload {
                public: vec![],
                secret: vec![bacon24()],
            },
            100_000,
            salt([0xbe, 0xef]),
            iv([0xba, 0xc0]),
            PASS,
        )
        .unwrap();
        assert_eq!(b.len(), 211);
        assert_eq!(
            sha(&b),
            "6707c20e7967e80e4cd4cb6dbe05e681d56c722320aa8213886c05a31e94def0"
        );
    }

    /// Identical to A except iterations. The ONLY test that catches a hardcoded
    /// count: the altered-header negative mismatches on AAD regardless.
    #[test]
    fn vector_b_differs_only_in_iterations() {
        let b = seal_deterministic(
            Payload {
                public: vec![],
                secret: vec![bacon24()],
            },
            100_001,
            salt([0xbe, 0xef]),
            iv([0xba, 0xc0]),
            PASS,
        )
        .unwrap();
        assert_eq!(
            sha(&b),
            "25fc2eaf950c9455497dc18eea6a93f5a54463a471cd15a4f8f327d13c7fea4c"
        );
    }

    #[test]
    fn vector_c_full_bundle_encrypted() {
        let b = seal_deterministic(
            Payload {
                public: vec![],
                secret: bip84(),
            },
            100_000,
            salt([0xbe, 0xad]),
            iv([0xca, 0xfe]),
            PASS,
        )
        .unwrap();
        assert_eq!(b.len(), 540);
        assert_eq!(
            sha(&b),
            "272f45e8ee30c95fdb1804ca54a9ec4b1d8c1358967d88c76312c0f725973ffc"
        );
    }

    /// MIXED: 5 public cards in the AAD, ms1 encrypted.
    #[test]
    fn vector_d_mixed() {
        let all = bip84();
        let b = seal_deterministic(
            Payload {
                public: all[1..].to_vec(),
                secret: vec![all[0].clone()],
            },
            100_000,
            salt([0xd0, 0x0d]),
            iv([0xf0, 0x0d]),
            PASS,
        )
        .unwrap();
        assert_eq!(b.len(), 539);
        assert_eq!(
            sha(&b),
            "6332e2d674322b2af656677cb550754b1ec7691f3df14895a807297712cdcd6a"
        );
    }

    /// PUBLIC-ONLY: no key, no tag, no passphrase.
    #[test]
    fn vector_e_public_only() {
        let all = bip84();
        let b = seal_public_only(all[1..].to_vec()).unwrap();
        assert_eq!(b.len(), 448);
        assert_eq!(
            sha(&b),
            "39b21ef010540d16967bba954bac6e94a888b2811b65df2e829402dc68d1c132"
        );
        // §6.2: the crypto fields must be zero.
        assert_eq!(&b[9..11], &[0, 0]);
        assert!(
            b[12..44].iter().all(|&x| x == 0),
            "iterations, salt and iv must be zero"
        );
        assert_eq!(&b[48..52], &[0, 0, 0, 0], "ct_len must be zero");
    }

    /// A public section spanning FOUR CARDS: one `md1` card chunked six ways
    /// (csid 841149) plus three `mk1` cards of two chunks each (153720, 153721,
    /// 153723) — one per cosigner. Those csids are measured on BOTH sides and
    /// agree: `mk.ParseHeader` (device) and `StringLayerHeader::from_5bit_symbols`
    /// (host) return the same values, and the records reproduce byte-identically
    /// across `mnemonic bundle` runs. The only vector that catches an
    /// implementation grouping by HRP instead of by `(HRP, chunk_set_id)`;
    /// D and E carry one card per HRP, F is `pub_len = 0`.
    ///
    /// The structural rule, verified rather than assumed: **one `mk1` card per
    /// cosigner, one `md1` card chunked as the policy requires.** An earlier
    /// draft said "six cards", generalising "three cosigners" onto both halves.
    #[test]
    fn vector_g_multisig_public_section_spans_four_cards() {
        let recs = two_of_three();
        let public: Vec<String> = recs
            .iter()
            .filter(|r| !r.starts_with("ms1"))
            .cloned()
            .collect();
        let secret: Vec<String> = recs
            .iter()
            .filter(|r| r.starts_with("ms1"))
            .cloned()
            .collect();
        assert_eq!((public.len(), secret.len()), (12, 3));
        let b = seal_deterministic(
            Payload { public, secret },
            100_000,
            salt([0xab, 0xcd]),
            iv([0x12, 0x34]),
            PASS,
        )
        .unwrap();
        assert_eq!(b.len(), 1420);
        assert_eq!(
            sha(&b),
            "483fb482ac7aef0da3fec638de183f8f3bfb35e1b6c0ec4f5b274ec0409908f1"
        );
    }

    /// THREE secret records. Without this a singular implementation of the
    /// session flow passes A–E and every negative.
    #[test]
    fn vector_f_two_of_three_multisig() {
        let recs = two_of_three();
        assert_eq!(recs.len(), 15);
        assert_eq!(recs.iter().filter(|r| r.starts_with("ms1")).count(), 3);
        let b = seal_deterministic(
            Payload {
                public: vec![],
                secret: recs,
            },
            100_000,
            salt([0xf0, 0x0d]),
            iv([0xbe, 0xef]),
            PASS,
        )
        .unwrap();
        assert_eq!(b.len(), 1421);
        assert_eq!(
            sha(&b),
            "97e059ac91596da711a70197b20a7fec1edbe7992eba6c51751ef062596f1cb6"
        );
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
        assert_eq!(
            std::str::from_utf8(&pt)
                .unwrap()
                .split('\n')
                .collect::<Vec<_>>(),
            expect.iter().map(|s| s.as_str()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn every_encrypted_vector_round_trips() {
        let all = bip84();
        open_vector(
            &seal_deterministic(
                Payload {
                    public: vec![],
                    secret: vec![bacon24()],
                },
                100_000,
                salt([0xbe, 0xef]),
                iv([0xba, 0xc0]),
                PASS,
            )
            .unwrap(),
            &[bacon24()],
        );
        open_vector(
            &seal_deterministic(
                Payload {
                    public: vec![],
                    secret: vec![bacon24()],
                },
                100_001,
                salt([0xbe, 0xef]),
                iv([0xba, 0xc0]),
                PASS,
            )
            .unwrap(),
            &[bacon24()],
        );
        open_vector(
            &seal_deterministic(
                Payload {
                    public: vec![],
                    secret: all.clone(),
                },
                100_000,
                salt([0xbe, 0xad]),
                iv([0xca, 0xfe]),
                PASS,
            )
            .unwrap(),
            &all,
        );
        open_vector(
            &seal_deterministic(
                Payload {
                    public: all[1..].to_vec(),
                    secret: vec![all[0].clone()],
                },
                100_000,
                salt([0xd0, 0x0d]),
                iv([0xf0, 0x0d]),
                PASS,
            )
            .unwrap(),
            &[all[0].clone()],
        );
        open_vector(
            &seal_deterministic(
                Payload {
                    public: vec![],
                    secret: two_of_three(),
                },
                100_000,
                salt([0xf0, 0x0d]),
                iv([0xbe, 0xef]),
                PASS,
            )
            .unwrap(),
            &two_of_three(),
        );
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
            Payload {
                public: all[1..].to_vec(),
                secret: vec![all[0].clone()],
            },
            100_000,
            salt([0xd0, 0x0d]),
            iv([0xf0, 0x0d]),
            PASS,
        )
        .unwrap();
        let h = wire::Header::decode(&blob).unwrap();
        let split = HEADER_LEN + h.pub_len as usize;
        // First and last byte of the public section.
        for off in [HEADER_LEN, split - 1] {
            let mut bad = blob.clone();
            bad[off] ^= 0x01;
            let key = crypto::derive_key(&passphrase::normalise(PASS), &h.salt, h.iterations);
            assert!(
                crypto::open_bytes(&key, &h.iv, &bad[..split], &bad[split..]).is_err(),
                "flipping public-section byte {off} must fail the tag"
            );
        }
    }

    /// §11.4: `iterations` altered 100000 → 100002 on vector A. **Not 50000** —
    /// that is rejected by §6.2's floor before any tag work, so it proves
    /// nothing about the AAD.
    #[test]
    fn altering_iterations_in_the_header_fails_the_tag() {
        let blob = seal_deterministic(
            Payload {
                public: vec![],
                secret: vec![bacon24()],
            },
            100_000,
            salt([0xbe, 0xef]),
            iv([0xba, 0xc0]),
            PASS,
        )
        .unwrap();
        let mut bad = blob.clone();
        bad[12..16].copy_from_slice(&100_002u32.to_be_bytes());
        let h = wire::Header::decode(&bad).expect("100002 is inside §6.2's range");
        let key = crypto::derive_key(&passphrase::normalise(PASS), &h.salt, h.iterations);
        assert!(crypto::open_bytes(&key, &h.iv, &bad[..HEADER_LEN], &bad[HEADER_LEN..]).is_err());
    }

    /// §6.4's 1..24 cap is over the TOTAL across both sections — 20 public plus
    /// 10 secret is legal per-section and illegal combined.
    /// §6.4's 1..24 cap is over the TOTAL across both sections.
    ///
    /// **The public set must actually DECODE**, or this test passes for the
    /// wrong reason: an earlier version used 20 copies of one md1 chunk, which
    /// dies in `check_public` with `chunk set incomplete: got 20 chunks,
    /// expected 3` long before the cap is reached — and deleting the combined-cap
    /// check entirely left all 52 tests green. Match the specific error, not
    /// merely `is_err()`.
    #[test]
    fn refuses_more_than_24_records_across_both_sections() {
        let all = bip84();
        let public = all[1..].to_vec(); // 5, decodes
        let secret: Vec<String> = std::iter::repeat_n(all[0].clone(), 20).collect();
        assert!(matches!(
            seal(Payload { public, secret }, 300_000),
            Err(SealError::Container(
                container::ContainerError::RecordCount(25)
            ))
        ));
    }

    /// Nothing else catches a frozen salt: the round-trip test and every fixed-salt
    /// vector pass under one.
    #[test]
    fn two_seals_of_the_same_payload_differ_everywhere() {
        let p = || Payload {
            public: vec![],
            secret: vec![bacon24()],
        };
        let a = seal(p(), 300_000).unwrap();
        let b = seal(p(), 300_000).unwrap();
        assert_ne!(a.blob, b.blob);
        assert_ne!(a.blob[16..32], b.blob[16..32], "salt must be fresh");
        assert_ne!(a.blob[32..44], b.blob[32..44], "iv must be fresh");
        // Option<Zeroizing<String>> — as_deref(), not `*`, which does not compile.
        assert_ne!(
            a.passphrase.as_deref(),
            b.passphrase.as_deref(),
            "passphrase must be fresh"
        );
    }

    /// Two vectors sharing a (key, iv) pair would be GCM nonce reuse in our own
    /// test data — the mistake caught in an earlier draft of vector C.
    #[test]
    fn no_two_vectors_share_a_key_iv_pair() {
        use crate::seal::crypto::derive_key;
        let pairs = [
            (
                hex(&*derive_key(PASS, &salt([0xbe, 0xef]), 100_000)),
                "bac0".repeat(6),
            ),
            (
                hex(&*derive_key(PASS, &salt([0xbe, 0xef]), 100_001)),
                "bac0".repeat(6),
            ),
            (
                hex(&*derive_key(PASS, &salt([0xbe, 0xad]), 100_000)),
                "cafe".repeat(6),
            ),
            (
                hex(&*derive_key(PASS, &salt([0xd0, 0x0d]), 100_000)),
                "f00d".repeat(6),
            ),
            (
                hex(&*derive_key(PASS, &salt([0xf0, 0x0d]), 100_000)),
                "beef".repeat(6),
            ),
        ];
        assert_eq!(
            pairs.iter().collect::<std::collections::HashSet<_>>().len(),
            pairs.len()
        );
    }

    #[test]
    fn refuses_a_secret_in_the_public_section() {
        let all = bip84();
        assert!(matches!(
            seal(
                Payload {
                    public: vec![all[0].clone()],
                    secret: vec![]
                },
                300_000
            ),
            Err(SealError::SecretInPublic(_))
        ));
    }

    #[test]
    fn refuses_an_undecodable_public_set() {
        let all = bip84();
        // One md1 chunk of three: BCH-valid, but the set does not decode.
        assert!(seal(
            Payload {
                public: vec![all[3].clone()],
                secret: vec![]
            },
            300_000
        )
        .is_err());
    }

    /// §11.4: the checksum gate runs BEFORE the KDF. On device that is 1 s
    /// versus 31 s.
    #[test]
    fn refuses_an_invalid_passphrase_without_running_the_kdf() {
        let start = std::time::Instant::now();
        let r = seal_deterministic(
            Payload {
                public: vec![],
                secret: vec![bacon24()],
            },
            2_000_000,
            salt([0xbe, 0xef]),
            iv([0xba, 0xc0]),
            &"abandon ".repeat(12),
        );
        assert!(matches!(r, Err(SealError::Passphrase(_))));
        assert!(
            start.elapsed() < std::time::Duration::from_millis(200),
            "2M rounds cannot finish that fast — the gate must precede the KDF"
        );
    }

    /// Kills the mutation that deletes `record_or_mnemonic`'s lowercase check.
    /// An earlier draft listed the killer as a manual `me seal "BACON …"`
    /// invocation rather than a test — and deleting the guard left all 166 tests
    /// green. Every mutation row must name a real test function.
    #[test]
    fn refuses_an_uppercase_bip39_mnemonic() {
        let upper = bacon24().to_uppercase();
        assert!(matches!(
            seal_deterministic(
                Payload {
                    public: vec![],
                    secret: vec![upper]
                },
                100_000,
                salt([0xbe, 0xef]),
                iv([0xba, 0xc0]),
                PASS
            ),
            Err(SealError::Record(record::RecordError::NotLowercase(_)))
        ));
    }

    /// The guard sits above `seal()`'s public-only early return, so it binds
    /// that path too. Without this case, moving the check below the early
    /// return leaves the suite green — both other iteration tests take the
    /// secret path.
    #[test]
    fn refuses_out_of_range_iterations_on_the_public_only_path() {
        let all = bip84();
        assert!(matches!(
            seal(
                Payload {
                    public: all[1..].to_vec(),
                    secret: vec![]
                },
                5
            ),
            Err(SealError::Iterations(5))
        ));
    }

    #[test]
    fn refuses_out_of_range_iterations() {
        for bad in [0u32, 99_999, 2_000_001, u32::MAX] {
            assert!(
                matches!(
                    seal_deterministic(
                        Payload {
                            public: vec![],
                            secret: vec![bacon24()]
                        },
                        bad,
                        salt([0xbe, 0xef]),
                        iv([0xba, 0xc0]),
                        PASS
                    ),
                    Err(SealError::Iterations(_))
                ),
                "iterations {bad}"
            );
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
        assert_eq!(
            v.iter().map(|r| r.len()).collect::<Vec<_>>(),
            vec![75, 75, 75, 111, 93, 111, 93, 111, 93, 85, 85, 85, 85, 85, 77],
            "vector F records are not canonical — did you use --group-size 0?"
        );
        assert_eq!(v.iter().filter(|r| r.starts_with("ms1")).count(), 3);
        v
    }
}
