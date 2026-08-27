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

impl Payload {
    /// Zero the secret records in place.
    ///
    /// Separate from `Drop` so it can be **tested directly**: a test that tried
    /// to read the buffer after a drop would be reading freed memory, and a
    /// test that cannot be written soundly tends not to be written at all.
    pub fn zeroize_secret(&mut self) {
        use zeroize::Zeroize;
        for r in &mut self.secret {
            r.zeroize();
        }
        self.secret.zeroize();
    }
}

/// `Payload.secret` holds the operator's seed as plain `String`s, and a plain
/// `Vec<String>` is freed in the clear — which undid part of F-102's fix one
/// line after it was made. Found by the Phase 2 Rust-side review with a probing
/// allocator.
///
/// KNOWN RESIDUAL, stated rather than implied: this reaches the CURRENT
/// buffers. Any reallocation during construction — a `push` past capacity, a
/// `String` grown while being built — orphans an earlier array that nothing can
/// reach. That is the same unfunnelled-growth class the device side records
/// against `bip39.Parse`, and closing it needs the records to be built into
/// pre-sized `Zeroizing` buffers rather than accumulated.
impl Drop for Payload {
    fn drop(&mut self) {
        self.zeroize_secret();
    }
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
pub fn seal(mut payload: Payload, iterations: u32) -> Result<Sealed, SealError> {
    // Range-check even on the public-only path, which ignores the value: silently
    // accepting `--iterations 5` teaches the operator the flag is advisory.
    if !(MIN_ITERATIONS..=MAX_ITERATIONS).contains(&iterations) {
        return Err(SealError::Iterations(iterations));
    }
    if payload.secret.is_empty() {
        // `take` rather than a move: `Payload` implements `Drop` so its secret
        // records are scrubbed, and a type with `Drop` cannot be moved out of
        // field-wise. Taking the PUBLIC section costs nothing (it is not secret
        // and the payload is dead after this) and leaves the drop glue intact.
        // Cloning here — the compiler's suggestion — would copy the public
        // records for no reason.
        return Ok(Sealed {
            blob: seal_public_only(std::mem::take(&mut payload.public))?,
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
        // An over-length `ms1` must be reported as a SECRET in the public
        // section, not as a length problem. §10.2.1a's rule lives inside
        // `validate_record`, so its error would otherwise propagate before
        // `is_secret()` is ever evaluated, and the operator would be told the
        // record is too long while saying nothing about their being one
        // keystroke from publishing a seed.
        //
        // Matching on the ERROR rather than pre-empting the call is what keeps
        // `validate_record` the classifier: canonical form, case and the real
        // parse are all still checked. An earlier fold pre-empted it with
        // `classify(r) == Format::Ms`, which fires on a bare two-character HRP
        // match -- so `ms1`-prefixed garbage, a space-interrupted string and an
        // uppercase-but-valid one were all relabelled "secret material",
        // replacing three accurate diagnoses to fix one. Same defect class as
        // the one being fixed.
        match record::validate_record(r) {
            Ok(k) if k.is_secret() => return Err(SealError::SecretInPublic(i)),
            Err(record::RecordError::MsTooLong(_)) => return Err(SealError::SecretInPublic(i)),
            Err(e) => return Err(SealError::Record(e)),
            Ok(_) => {}
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
    // The WHITESPACE half of the argument the uppercase guard above makes.
    // `normalise` does two things — it lowercases AND it collapses whitespace
    // via `split_whitespace`, which treats runs, TAB, NBSP and every other
    // Unicode space as a separator. Only the case half was acted on, so a
    // mnemonic separated by a double space or a TAB still validates here and is
    // emitted VERBATIM by `encode_section`. The device splits on a single ASCII
    // space only (`bip39/bip39.go:289`: `bytes.SplitSeq(buf, []byte(" "))`), so
    // it refuses the backup after the ~31 s KDF and shows §6.4's "payload
    // unreadable" — which §2.2 item 4 has taught the operator to read as
    // tampering. §6.4 requires that a bundle the device will reject never leave
    // the host, so this belongs on this side.
    if let Some(pos) = non_canonical_separator(s) {
        return Err(SealError::Record(record::RecordError::NonCanonicalSpace(
            pos,
        )));
    }
    if passphrase::is_valid(s) {
        return Ok(());
    }
    Err(SealError::Record(record_err))
}

/// Byte offset of the first separator the device's parser could not split on,
/// or `None` if every word is separated by exactly one ASCII space with none
/// leading or trailing.
///
/// Deliberately NOT `s != *normalise(s)`: that conflates case with spacing and
/// would report an uppercase mnemonic under a spacing error, losing the guard
/// above's position and its message.
fn non_canonical_separator(s: &str) -> Option<usize> {
    if let Some((i, _)) = s
        .char_indices()
        .find(|(_, c)| c.is_whitespace() && *c != ' ')
    {
        return Some(i);
    }
    if s.starts_with(' ') {
        return Some(0);
    }
    if s.ends_with(' ') {
        return Some(s.len() - 1);
    }
    s.find("  ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    const PASS: &str = "beef beef beef beef beef beef beef beef beef beef beef beef";

    /// A checksum-valid 12-word English mnemonic, canonical form.
    const MNEMONIC: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon \
         abandon about";

    /// C1 from the Phase 2 Rust-side review.
    ///
    /// `passphrase::is_valid` normalises with `split_whitespace` before parsing,
    /// which collapses runs AND accepts any Unicode whitespace. `encode_section`
    /// then emits the record AS SUPPLIED. The device splits on a single ASCII
    /// space only (`bip39/bip39.go:289`, `bytes.SplitSeq(buf, []byte(" "))`), so
    /// every one of these seals cleanly and is then refused on the machine after
    /// a ~31 s KDF — surfaced as §6.4 "payload unreadable", which §2.2 item 4
    /// has taught the operator to read as TAMPERING.
    ///
    /// This is the whitespace half of the argument the uppercase guard above
    /// already makes; only the case half was acted on.
    #[test]
    fn refuses_a_mnemonic_whose_whitespace_the_device_cannot_split() {
        for (name, sep) in [
            ("double space", "  "),
            ("tab", "\t"),
            ("no-break space", "\u{00a0}"),
            ("vertical tab", "\u{000b}"),
            ("ideographic space", "\u{3000}"),
            ("newline", "\n"),
        ] {
            let m = MNEMONIC.replacen(' ', sep, 1);
            assert!(
                record_or_mnemonic(&m).is_err(),
                "{name}: sealed a mnemonic the device cannot parse"
            );
        }
        // Leading and trailing space survive `split_whitespace` too.
        assert!(
            record_or_mnemonic(&format!(" {MNEMONIC}")).is_err(),
            "leading space"
        );
        assert!(
            record_or_mnemonic(&format!("{MNEMONIC} ")).is_err(),
            "trailing space"
        );
    }

    /// The secret records must not be freed in the clear.
    ///
    /// Asserts on `zeroize_secret` rather than after a drop, because reading a
    /// buffer after its owner is dropped is reading freed memory — the test
    /// would be unsound whatever it printed.
    #[test]
    fn zeroizes_the_secret_records() {
        let mut p = Payload {
            public: vec!["md1public".to_string()],
            secret: vec![MNEMONIC.to_string(), "ms1secret".to_string()],
        };
        // Positive control: the secret really is present first, so a test that
        // passed because the vector was empty all along cannot masquerade.
        assert!(
            p.secret.iter().any(|r| r.contains("abandon")),
            "control: the payload must hold the secret before it is scrubbed"
        );
        p.zeroize_secret();
        assert!(
            p.secret.is_empty(),
            "secret records survive zeroize_secret: {:?}",
            p.secret.len()
        );
        // The public section is untouched — scrubbing must not eat the data the
        // device needs.
        assert_eq!(p.public, vec!["md1public".to_string()]);
    }

    /// The positive control for the test above: without it, a guard that
    /// refused EVERY mnemonic would pass and would break the feature outright.
    #[test]
    fn still_accepts_a_canonical_mnemonic() {
        assert!(
            record_or_mnemonic(MNEMONIC).is_ok(),
            "canonical single-space mnemonic must still seal"
        );
    }

    fn hex(b: &[u8]) -> String {
        b.iter().fold(String::new(), |mut acc, x| {
            let _ = write!(acc, "{x:02x}");
            acc
        })
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

    // ---------------------------------------------------------------------
    // The canonical-vector exporter (Plan B Phase A, Task 1).
    //
    // It lives HERE, inside `mod tests`, and not under `tests/`: an
    // integration test is a separate crate and sees only `pub` items, while
    // `seal_deterministic` is deliberately `pub(crate)` (see its doc comment)
    // and every fixture below — `PASS`, `bacon24`, `bip84`, `two_of_three`,
    // `salt`, `iv` — is private to this module. Making the seam `pub` to
    // satisfy an exporter would destroy the one-key-one-message property.
    //
    // Reusing the same fixtures the vector tests use is the point: the INPUTS
    // cannot drift from the emitted vectors either.
    // ---------------------------------------------------------------------

    /// One emitted vector. Field order here is the field order in the JSON —
    /// `serde_json` writes a derived struct in declaration order.
    #[derive(serde::Serialize)]
    struct EmittedVector {
        name: String,
        /// `null` for the unsealed shape: there is no key, so there is no
        /// passphrase.
        passphrase: Option<String>,
        iterations: u32,
        salt_hex: String,
        iv_hex: String,
        public: Vec<String>,
        secret: Vec<String>,
        /// REQUIRED, and taken from the `Header` the exporter built from the
        /// encoded sections — never re-read from `header_hex`/`blob_hex`. The
        /// Go parser's binding test asserts against these, and a value
        /// recovered from the bytes the parser just read cannot fail.
        pub_len: u32,
        ct_len: u32,
        blob_hex: String,
        blob_sha256: String,
        header_hex: String,
        derived_key_hex: Option<String>,
        tag_hex: Option<String>,
        /// `null` when `pub_len == 0` — §10.2 step 3 displays nothing then.
        pubhash_sealed: Option<String>,
        pubhash_unsealed: Option<String>,
    }

    #[derive(serde::Serialize)]
    struct EmittedVectors {
        note: String,
        spec: String,
        vectors: Vec<EmittedVector>,
    }

    /// The inputs to one vector. `passphrase: None` selects the unsealed shape,
    /// which is emitted via `seal_public_only`.
    struct VectorSpec {
        name: &'static str,
        passphrase: Option<&'static str>,
        iterations: u32,
        salt: [u8; SALT_LEN],
        iv: [u8; IV_LEN],
        public: Vec<String>,
        secret: Vec<String>,
    }

    const VECTORS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/seal_vectors.json");
    const EMIT_CMD: &str =
        "ME_EMIT_VECTORS=1 cargo test -p mnemonic-engrave --lib seal::tests::emit_vectors";

    fn build_vectors() -> EmittedVectors {
        let all = bip84();
        let tot = two_of_three();
        let g_public: Vec<String> = tot
            .iter()
            .filter(|r| !r.starts_with("ms1"))
            .cloned()
            .collect();
        let g_secret: Vec<String> = tot
            .iter()
            .filter(|r| r.starts_with("ms1"))
            .cloned()
            .collect();

        let specs: Vec<VectorSpec> = vec![
            VectorSpec {
                name: "A",
                passphrase: Some(PASS),
                iterations: 100_000,
                salt: salt([0xbe, 0xef]),
                iv: iv([0xba, 0xc0]),
                public: vec![],
                secret: vec![bacon24()],
            },
            VectorSpec {
                name: "B",
                passphrase: Some(PASS),
                iterations: 100_001,
                salt: salt([0xbe, 0xef]),
                iv: iv([0xba, 0xc0]),
                public: vec![],
                secret: vec![bacon24()],
            },
            VectorSpec {
                name: "C",
                passphrase: Some(PASS),
                iterations: 100_000,
                salt: salt([0xbe, 0xad]),
                iv: iv([0xca, 0xfe]),
                public: vec![],
                secret: all.clone(),
            },
            VectorSpec {
                name: "D",
                passphrase: Some(PASS),
                iterations: 100_000,
                salt: salt([0xd0, 0x0d]),
                iv: iv([0xf0, 0x0d]),
                public: all[1..].to_vec(),
                secret: vec![all[0].clone()],
            },
            // E is the UNSEALED shape and is emitted via `seal_public_only`,
            // not `seal_deterministic`: no passphrase, no key, no tag, and
            // §6.2 requires iterations/salt/iv to be zero.
            VectorSpec {
                name: "E",
                passphrase: None,
                iterations: 0,
                salt: [0u8; SALT_LEN],
                iv: [0u8; IV_LEN],
                public: all[1..].to_vec(),
                secret: vec![],
            },
            VectorSpec {
                name: "F",
                passphrase: Some(PASS),
                iterations: 100_000,
                salt: salt([0xf0, 0x0d]),
                iv: iv([0xbe, 0xef]),
                public: vec![],
                secret: tot.clone(),
            },
            VectorSpec {
                name: "G",
                passphrase: Some(PASS),
                iterations: 100_000,
                salt: salt([0xab, 0xcd]),
                iv: iv([0x12, 0x34]),
                public: g_public,
                secret: g_secret,
            },
        ];

        let mut vectors = Vec::with_capacity(specs.len());
        for VectorSpec {
            name,
            passphrase: pass,
            iterations,
            salt: s,
            iv: i,
            public,
            secret,
        } in specs
        {
            let blob = if secret.is_empty() {
                seal_public_only(public.clone()).expect("vector must seal")
            } else {
                seal_deterministic(
                    Payload {
                        public: public.clone(),
                        secret: secret.clone(),
                    },
                    iterations,
                    s,
                    i,
                    pass.expect("a sealed vector needs a passphrase"),
                )
                .expect("vector must seal")
            };

            // Build the header from the ENCODED SECTIONS, independently of the
            // blob. `pub_len`/`ct_len` must be a claim about what was sealed,
            // not a re-read of the bytes a parser is about to be tested on.
            let pubsec = if public.is_empty() {
                Zeroizing::new(String::new())
            } else {
                container::encode_section(&public).expect("public section must encode")
            };
            let secsec = if secret.is_empty() {
                Zeroizing::new(String::new())
            } else {
                container::encode_section(&secret).expect("secret section must encode")
            };
            let header = Header {
                iterations,
                salt: s,
                iv: i,
                pub_len: pubsec.len() as u32,
                ct_len: secsec.len() as u32,
            };

            let (pubhash_sealed, pubhash_unsealed) = if public.is_empty() {
                (None, None)
            } else {
                let refs: Vec<&str> = public.iter().map(|r| r.as_str()).collect();
                (
                    Some(hex(&pubhash::public_data_hash(&refs, true))),
                    Some(hex(&pubhash::public_data_hash(&refs, false))),
                )
            };

            vectors.push(EmittedVector {
                name: name.to_string(),
                passphrase: pass.map(|p| p.to_string()),
                iterations,
                salt_hex: hex(&s),
                iv_hex: hex(&i),
                public,
                secret,
                pub_len: header.pub_len,
                ct_len: header.ct_len,
                blob_hex: hex(&blob),
                blob_sha256: sha(&blob),
                header_hex: hex(&header.encode()),
                derived_key_hex: pass.map(|p| {
                    hex(&*crypto::derive_key(
                        &passphrase::normalise(p),
                        &s,
                        iterations,
                    ))
                }),
                tag_hex: if header.ct_len > 0 {
                    Some(hex(&blob[blob.len() - wire::TAG_LEN..]))
                } else {
                    None
                },
                pubhash_sealed,
                pubhash_unsealed,
            });
        }

        EmittedVectors {
            note: format!(
                "Generated by `{EMIT_CMD}`. Normative source: crates/me-cli/src/seal/. \
                 Do not hand-edit."
            ),
            spec: "SPEC_encrypted_payload_delivery.md".to_string(),
            vectors,
        }
    }

    /// Emits `testdata/seal_vectors.json` under `ME_EMIT_VECTORS=1`; otherwise
    /// asserts the committed file still matches, so drift in EITHER direction
    /// fails the suite.
    #[test]
    fn emit_vectors() {
        let json = serde_json::to_string_pretty(&build_vectors()).unwrap() + "\n";
        if std::env::var("ME_EMIT_VECTORS").as_deref() == Ok("1") {
            let path = std::path::Path::new(VECTORS_PATH);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, &json).unwrap();
            return;
        }
        let on_disk = std::fs::read_to_string(VECTORS_PATH)
            .unwrap_or_else(|e| panic!("{VECTORS_PATH}: {e} — regenerate with `{EMIT_CMD}`"));
        assert!(
            on_disk == json,
            "{VECTORS_PATH} is stale ({} bytes on disk, {} generated). \
             It is generated, never hand-edited: regenerate with `{EMIT_CMD}`.",
            on_disk.len(),
            json.len()
        );
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

        // SPEC §11.4 vector G publishes a §6.6 hash and the Go port (Plan B)
        // binds to it. `pubhash`'s own literals cover only D and E, whose public
        // sections hold one card per HRP — G is the only published value over a
        // MULTI-CARD public section, i.e. the one that moves if record ordering
        // or the count byte is ever disturbed by the grouping logic.
        // Value taken from the spec table, not from this implementation's output.
        let g_refs: Vec<&str> = public.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            pubhash::format_hash(&pubhash::public_data_hash(&g_refs, true)),
            "be11 7b56 9cc4 cd6e b47d 32b6 fd32 ccb8",
            "vector G's §6.6 hash (SPEC §11.4)"
        );

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

    /// An over-length `ms1` in the PUBLIC section must be reported as a secret,
    /// not as a length problem.
    ///
    /// Both refuse, so nothing leaks either way — this pins WHICH REASON the
    /// operator is told, and the suppressed one is the one that matters. §10.2.1a's
    /// length rule lives inside `validate_record`, so its error propagates through
    /// `check_public`'s `?` before `is_secret()` is ever evaluated. Measured on the
    /// real binary before the fix:
    ///
    ///   "this codex32 secret is 91 characters; the machine can engrave at most 90"
    ///
    /// which says nothing about the operator being one keystroke from publishing a
    /// seed in the clear. After:
    ///
    ///   "record 0 is secret material and cannot ride in the public section"
    ///
    /// Mutation this pins: delete the `classify(r) == Format::Ms` guard from
    /// `check_public` — the assertion then sees `SealError::Record` instead.
    #[test]
    fn an_overlong_ms1_in_public_is_reported_as_a_secret_not_as_too_long() {
        // 91 characters: one past §10.2.1a's engraveable limit, and a secret.
        const MS1_91: &str =
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq2uk6ly9a0dmw4";
        assert_eq!(MS1_91.len(), 91, "fixture is not 91 characters");

        match seal(
            Payload {
                public: vec![MS1_91.to_string()],
                secret: vec![],
            },
            300_000,
        ) {
            Err(SealError::SecretInPublic(0)) => {}
            Err(other) => panic!(
                "over-length ms1 in --plaintext reported as {other:?}; the operator needs \
                 to hear that it is a SEED about to ship in the clear, not that it is too \
                 long"
            ),
            Ok(_) => panic!("an ms1 in the public section must be refused"),
        }
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
