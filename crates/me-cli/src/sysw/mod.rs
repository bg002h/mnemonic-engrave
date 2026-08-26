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

pub mod coverage;
pub mod identity;
pub mod mt;
pub mod overwrite;
pub mod passphrase;
pub mod pubhash;
pub mod record;
pub mod tx;
pub mod vectors;
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
    ///
    /// Carries WHY, because [`classify`] returns `Unknown` for two unrelated
    /// situations and the operator's next move differs completely between them.
    /// A single message naming only one of them sent the reader looking at the
    /// wrong thing — found writing the Load Payload journey, where a `pass:`
    /// record with a plain-text body was refused with a sentence about
    /// descriptors and addresses.
    Unclassifiable(usize, UnknownReason),
    /// Sections exceed what the region can hold.
    TooLarge(usize),
    Crypto,
    /// A sealed payload was handed no passphrase, or vice versa.
    PassphraseMismatch,
    /// The passphrase normalises to nothing. Not a strength judgement — the
    /// DEVICE reads an empty passphrase as "none supplied", so such a payload
    /// could never be opened on the machine it is for.
    EmptyPassphrase,
    /// A token the device's keyboard cannot produce. Same class as
    /// [`SyswError::EmptyPassphrase`]: not about strength, about whether the
    /// resulting payload can ever be opened on the machine it is for. Carries
    /// the offending token so the operator is told WHICH word to change.
    NotEnterableOnDevice(String),
    /// Normalised passphrase longer than `[passphrase-bounds]` (§12.5) allows.
    PassphraseTooLong(usize),
    NotUtf8,
}

/// Why [`classify`] could not place a record.
///
/// Carries NO operator data, and that is load-bearing rather than tidy: the
/// record that most often lands here is a `pass:` one, whose body IS the
/// passphrase. An error message is the last place it may appear — stderr is
/// logged, scrolled back and pasted into bug reports. Only the reserved prefix
/// is named, and that is a compile-time constant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnknownReason {
    /// A reserved prefix (`text:` / `pass:` / `tx:`) whose body is not
    /// lowercase hex. The prefixes are RESERVED, so this is refused rather
    /// than demoted to free text — see [`record`]'s module docs for why the
    /// bodies are encoded.
    NonHexBody(&'static str),
    /// A `tx:` body that IS hex but does not parse as one serialized Bitcoin
    /// transaction. Carries the structural reason — which names no operator
    /// data, only a shape.
    NotATransaction(tx::TxError),
    /// A `tx:` body that parses as a transaction but has an input carrying
    /// NEITHER a scriptSig NOR a witness — it is unsigned, or its signatures
    /// were stripped. **The txid is unchanged by stripping**, so this is the
    /// only signal there is, and a plate cut from such a body can never be
    /// broadcast.
    ///
    /// Carries the failing INPUT indices. They name no operator data — an
    /// input's position in a serialization the operator already holds — and a
    /// refusal that says only "an input is unsigned" gives them nothing to
    /// look at. [`Admission::allow_unsigned_inputs`] overrides this one arm.
    UnsignedInputs(Vec<usize>),
    /// No reserved prefix, not a BIP-39 mnemonic, and not a constellation
    /// string. This is the case the descriptor/address gap belongs to.
    Unrecognised,
}

/// Which reason applies, decided where the record is still in hand.
fn unknown_reason(record: &str) -> UnknownReason {
    if record.starts_with(record::TX_PREFIX) {
        return match record::decode_body(record) {
            Err(_) => UnknownReason::NonHexBody(record::TX_PREFIX),
            Ok(b) => match tx::parse(&b) {
                Err(e) => UnknownReason::NotATransaction(e),
                // It parsed, so the refusal was the signature predicate.
                Ok(t) if !t.every_input_signed => {
                    UnknownReason::UnsignedInputs(t.unsigned_inputs)
                }
                // classify refused it, so neither arm can be reached here; keep
                // a total answer anyway rather than panic on a future skew.
                Ok(_) => UnknownReason::Unrecognised,
            },
        };
    }
    for prefix in [record::PASS_PREFIX, record::TEXT_PREFIX] {
        if record.starts_with(prefix) {
            return UnknownReason::NonHexBody(prefix);
        }
    }
    UnknownReason::Unrecognised
}

/// What the packer will admit that strict classification would not.
///
/// **One arm, deliberately.** `--allow-unsigned-inputs` (`FORWARD_PLAN` §2.1)
/// exists because the signature predicate has honest false positives — a P2A
/// anchor-spend input carries neither a scriptSig nor a witness and is
/// perfectly valid — and a check with no escape hatch becomes a reason to stop
/// using the tool. It is NOT a general "admit anything" switch: every other
/// requirement the `tx:` prefix carries still refuses.
///
/// **It deliberately does not reach the `mt1` chunk class.** Nothing in the
/// chunk path refuses (ruling 2026-08-25b), so there is no refusal to
/// override; and the DEVICE recomputes confirmation itself, so a host flag
/// that made an unsigned set report "confirmed" would only make the two
/// disagree. `sysw::mt::set_confirmed` therefore ignores this type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Admission {
    /// Admit a `tx:` record whose transaction parses but has at least one
    /// input carrying neither a scriptSig nor a witness.
    pub allow_unsigned_inputs: bool,
}

/// Which section a record belongs in.
///
/// **Descriptor and Address are deliberately absent**, and this is a known
/// limitation rather than an oversight: classifying them needs a descriptor
/// parser and an address decoder, neither of which is a dependency of this
/// crate. An unclassifiable record is REFUSED at pack time with its index, so
/// the failure is a named error at creation rather than a mis-filed secret.
pub fn classify(record: &str) -> record::Class {
    classify_with(record, Admission::default())
}

/// [`classify`] under a stated [`Admission`]. The strict default is what every
/// reader uses; only `me sysw pack` with an explicit flag passes anything else.
pub fn classify_with(record: &str, adm: Admission) -> record::Class {
    use record::Class;
    if record.starts_with(record::TX_PREFIX) {
        // Reserved, like text:/pass: -- and admission requires the body to
        // PARSE as a transaction, so the prefix cannot smuggle arbitrary
        // bytes into a non-secret class.
        return match record::decode_body(record) {
            // A `tx:` record must parse AND carry a signature on every input.
            // The signature check is not fastidiousness: a witness-stripped
            // transaction has the SAME TXID as the honest one it came from, so
            // it passes every identifier comparison an operator can make — and
            // a plate cut from it can never be broadcast. See `tx::TxSummary`.
            Ok(b) => match tx::parse(&b) {
                Ok(t) if t.every_input_signed || adm.allow_unsigned_inputs => Class::Tx,
                _ => Class::Unknown,
            },
            _ => Class::Unknown,
        };
    }
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
    // Before validate_record: mt1's HRP is unknown to seal's validator, and
    // the strict check is self-contained.
    if mt::valid_mt(record) {
        return Class::Mt;
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
    pack_with(records, passphrase, iterations, Admission::default())
}

/// [`pack`] under a stated [`Admission`] — the `--allow-unsigned-inputs` seam.
pub fn pack_with(
    records: Vec<String>,
    passphrase: Option<&str>,
    iterations: u32,
    adm: Admission,
) -> Result<Vec<u8>, SyswError> {
    let mut salt = [0u8; wire::SALT_LEN];
    let mut iv = [0u8; wire::IV_LEN];
    rand::RngCore::fill_bytes(&mut rand::rng(), &mut salt);
    rand::RngCore::fill_bytes(&mut rand::rng(), &mut iv);
    pack_deterministic_with(records, passphrase, iterations, salt, iv, adm)
}

/// [`pack`] with the randomness supplied, so a fixture can be a fixture.
///
/// **This is the only implementation**, and that is deliberate. An earlier
/// version had `pack` and `pack_deterministic` each assemble the blob, and the
/// unsealed secret-class move was added to one and not the other — so
/// `pack_deterministic` SILENTLY DROPPED secret records. The vector round-trip
/// caught it (S-B recovered 0 records of 1). Two implementations of one rule is
/// the defect shape this whole cycle has been about; there is now one.
pub fn pack_deterministic(
    records: Vec<String>,
    passphrase: Option<&str>,
    iterations: u32,
    salt: [u8; wire::SALT_LEN],
    iv: [u8; wire::IV_LEN],
) -> Result<Vec<u8>, SyswError> {
    pack_deterministic_with(records, passphrase, iterations, salt, iv, Admission::default())
}

/// [`pack_deterministic`] under a stated [`Admission`]. **Still the only
/// implementation** — the doc comment above is about this function.
pub fn pack_deterministic_with(
    records: Vec<String>,
    passphrase: Option<&str>,
    iterations: u32,
    salt: [u8; wire::SALT_LEN],
    iv: [u8; wire::IV_LEN],
    adm: Admission,
) -> Result<Vec<u8>, SyswError> {
    let (mut payload, mut pub_bytes, mut header) = split(records, adm)?;

    // UNSEALED carries secret classes in the cleartext section — decision 6
    // permits it and F1 flags it at load. Only the sealed path encrypts them.
    if passphrase.is_none() && !payload.secret.is_empty() {
        payload
            .public
            .extend(payload.secret.drain(..).map(|z| (*z).clone()));
        pub_bytes = payload.public.join("\n").into_bytes();
        header.pub_len = pub_bytes.len() as u32;
    }

    let Some(pass) = passphrase else {
        let mut blob = header.encode().to_vec();
        blob.extend_from_slice(&pub_bytes);
        return bound(blob);
    };

    // Checked BEFORE the KDF, not just in `bound`, for two reasons: there is no
    // point running PBKDF2 for a container we are about to refuse, and the
    // operator has already been told to write a generated passphrase down. A
    // refusal that arrives after that ceremony teaches them the note is
    // worthless.
    if !(wire::MIN_ITERATIONS..=wire::MAX_ITERATIONS).contains(&iterations) {
        return Err(SyswError::Wire(wire::WireError::Iterations(iterations)));
    }

    // NOT a strength rule — those warn and proceed (spec §13 D3), and this must
    // not be mistaken for one. It is an UNOPENABLE-ARTIFACT rule. Rust models
    // "no passphrase" as `None`, so `Some("")` is a real passphrase and this
    // host will happily seal with it and open it again. The device models the
    // absent passphrase as the empty string (`open.go`), so on the machine an
    // empty passphrase reads as "none supplied" and the payload can NEVER be
    // opened — there is no keystroke that expresses it. `--passphrase-ask` plus
    // a bare Enter reaches this, and normalisation collapses whitespace-only to
    // the same place.
    let normalised = crate::seal::passphrase::normalise(pass);
    if normalised.is_empty() {
        return Err(SyswError::EmptyPassphrase);
    }

    // `[passphrase-bounds]` (§12.5), which was DECLARED on both sides and
    // enforced on neither until now — the constant, a const assertion and an
    // arithmetic test were its only references.
    if normalised.len() > wire::PASSPHRASE_MAX {
        return Err(SyswError::PassphraseTooLong(normalised.len()));
    }

    // NARROWED by operator ruling 2026-08-12: every token must be a BIP-39
    // English word. §12.5's character range is 0x20–0x7E, and decision 8
    // deliberately allowed an ASCII passphrase — but the DEVICE offers only a
    // word keyboard, so an ASCII passphrase seals a payload that cannot be typed
    // back in. Given a cheap-and-narrowing choice against building the keyboard,
    // the operator chose to narrow the host.
    //
    // This is deliberately NOT `cliff_above`: that is a word COUNT rule (five or
    // more) and this is only the wordlist half. Two words remain legal and stay
    // below the cliff, which is what decision 8 restored and F2 exists to warn
    // about.
    for tok in normalised.split_whitespace() {
        if bip39::Language::English.find_word(tok).is_none() {
            return Err(SyswError::NotEnterableOnDevice(tok.to_string()));
        }
    }
    seal_with(payload, pub_bytes, header, pass, iterations, salt, iv)
}

/// Admission, and **the only place it is decided** (F-246).
///
/// Split out of [`split`] so a caller can run it BEFORE anything expensive or
/// irreversible-looking happens — specifically before `me sysw pack` generates
/// a passphrase and tells the operator to write it down and store it apart from
/// the machine. Refusing after that ceremony hands them material to record
/// off-machine that protects nothing, directly above an error saying the run
/// failed.
///
/// It borrows, so hoisting it costs no clone of records that may be secret.
/// [`split`] calls it first and then partitions, which is why `split`'s loop no
/// longer matches `Unknown` — one rule, one implementation, per this module's
/// own note about `pack`/`pack_deterministic` drifting apart.
pub fn admit_check(records: &[String], adm: Admission) -> Result<(), SyswError> {
    for (i, r) in records.iter().enumerate() {
        if matches!(classify_with(r, adm), record::Class::Unknown) {
            return Err(SyswError::Unclassifiable(i, unknown_reason(r)));
        }
    }
    Ok(())
}

fn split(
    records: Vec<String>,
    adm: Admission,
) -> Result<(Payload, Vec<u8>, wire::Header), SyswError> {
    // Admission first, so the partition below is total: every record is either
    // secret or public by the time it runs.
    admit_check(&records, adm)?;
    let mut payload = Payload::default();
    for r in records {
        if classify_with(&r, adm).is_secret() {
            payload.secret.push(Zeroizing::new(r));
        } else {
            payload.public.push(r);
        }
    }
    let pub_bytes = payload.public.join("\n").into_bytes();
    let header = wire::Header {
        pub_len: pub_bytes.len() as u32,
        ..Default::default()
    };
    Ok((payload, pub_bytes, header))
}

fn seal_with(
    payload: Payload,
    pub_bytes: Vec<u8>,
    mut header: wire::Header,
    pass: &str,
    iterations: u32,
    salt: [u8; wire::SALT_LEN],
    iv: [u8; wire::IV_LEN],
) -> Result<Vec<u8>, SyswError> {
    let secret: Vec<&str> = payload.secret.iter().map(|s| s.as_str()).collect();
    let plaintext = Zeroizing::new(secret.join("\n").into_bytes());
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

/// The last gate every emitted container passes — both the sealed and unsealed
/// paths end here.
///
/// **The writer must refuse everything the reader refuses.** The pre-flash
/// review found it did not: `--iterations 5` and an over-long section both
/// produced a container that `Header::parse` rejects, at exit 0, and
/// `--region` would then have written that to flash. For a SEALED payload that
/// is a seed backup nobody can ever open.
///
/// The fix is deliberately not a second copy of the reader's bounds — two
/// copies of one rule is exactly how these drifted apart, and it is the defect
/// shape this whole module has been fighting. Instead: **run the reader.** Any
/// future divergence fails here rather than on steel.
fn bound(blob: Vec<u8>) -> Result<Vec<u8>, SyswError> {
    if blob.len() > wire::REGION_LEN {
        return Err(SyswError::TooLarge(blob.len()));
    }
    wire::Header::parse(&blob).map_err(SyswError::Wire)?;
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

    /// `bound`'s SIZE edge. Added because cargo-mutants showed `>` was
    /// indistinguishable from `>=` and `==`: nothing exercised a blob at exactly
    /// the region size.
    ///
    /// The fixture was `vec![0u8; N]` until `bound` also began parsing its own
    /// output (pre-flash review C1). All-zeros has no MAGIC, so it now fails for
    /// the wrong reason and would no longer pin the boundary. A REAL container
    /// padded to length keeps the original mutant dead: `Header::parse` reads
    /// only the header, so trailing bytes are legal, and the size check still
    /// runs first.
    #[test]
    fn a_blob_exactly_filling_the_region_is_legal_and_one_byte_more_is_not() {
        let real = pack(vec!["text:6869".into()], None, wire::MIN_ITERATIONS).unwrap();
        let pad_to = |n: usize| {
            let mut v = real.clone();
            v.resize(n, 0xFF);
            v
        };
        assert!(
            bound(pad_to(wire::REGION_LEN)).is_ok(),
            "exactly REGION_LEN fits"
        );
        assert_eq!(
            bound(pad_to(wire::REGION_LEN + 1)).unwrap_err(),
            SyswError::TooLarge(wire::REGION_LEN + 1)
        );
    }

    /// C1, as a property rather than two cases: whatever `pack` emits, the
    /// reader accepts. This is the invariant `bound` now enforces by running the
    /// reader, and it is what makes a writer/reader drift impossible to ship.
    #[test]
    fn pack_never_emits_what_the_reader_would_refuse() {
        for iters in [0, 1, 5, wire::MIN_ITERATIONS - 1, wire::MAX_ITERATIONS + 1] {
            assert!(
                pack(vec!["text:6869".into()], Some(PASS), iters).is_err(),
                "sealed pack accepted out-of-range iterations {iters}"
            );
        }
        // SIZED FROM THE CAP, never hard-coded. This used to build a literal
        // 30 records, which was comfortably past 8191 and stopped testing
        // anything the moment the cap was raised to 32,734: the section it
        // produced became legal, and `pack` was right to accept it. A count
        // derived from the constant cannot go quietly vacuous that way.
        const REC_LEN: usize = "text:".len() + 800; // 400 hex pairs
        let n = wire::MAX_SECTION_LEN / (REC_LEN + 1) + 2; // + the LF between records
        let section_len = n * REC_LEN + (n - 1);
        assert!(
            section_len > wire::MAX_SECTION_LEN,
            "the fixture must exceed the cap to test anything: {section_len} vs {}",
            wire::MAX_SECTION_LEN
        );
        let huge: Vec<String> = (0..n)
            .map(|_| format!("text:{}", "61".repeat(400)))
            .collect();
        assert!(
            pack(huge, None, wire::MIN_ITERATIONS).is_err(),
            "pack accepted a section its own parser rejects"
        );
    }

    /// `open`'s truncation check. The `<` -> `>` mutant survived because nothing
    /// handed `open` a blob shorter than its header declares.
    #[test]
    fn open_refuses_a_blob_shorter_than_its_header_declares() {
        let blob = pack(vec![md1()], None, ITER).unwrap();
        let short = &blob[..blob.len() - 1];
        assert_eq!(
            open(short, None).unwrap_err(),
            SyswError::Wire(wire::WireError::TooShort(short.len()))
        );
        assert!(
            open(&blob, None).is_ok(),
            "the untruncated blob still opens"
        );
    }

    #[test]
    fn an_unclassifiable_record_is_refused_with_its_index() {
        assert_eq!(
            pack(vec![md1(), "not a record".into()], None, ITER),
            Err(SyswError::Unclassifiable(1, UnknownReason::Unrecognised))
        );
    }

    /// The two `Unknown` cases must not collapse to one message. A reserved
    /// prefix with a plain-text body is the mistake everyone makes first, and
    /// its remedy — hex-encode the body — has nothing to do with the
    /// descriptor/address gap the other case is about.
    #[test]
    fn a_reserved_prefix_with_a_plain_body_reports_the_body_not_the_gap() {
        for (prefix, record) in [
            (record::PASS_PREFIX, "pass:correct horse battery staple"),
            (record::TEXT_PREFIX, "text:SEEDHAMMER II DEMO PAYLOAD"),
        ] {
            assert_eq!(
                pack(vec![record.into()], None, ITER),
                Err(SyswError::Unclassifiable(
                    0,
                    UnknownReason::NonHexBody(prefix)
                )),
                "{record}"
            );
        }
    }

    /// Uppercase hex is not lowercase hex, and §5.3.1 means the lowercase one:
    /// EPD §6.6 hashes the section lowercased, so accepting uppercase would let
    /// two spellings of one record produce two digests.
    #[test]
    fn an_uppercase_hex_body_is_still_a_non_hex_body() {
        assert_eq!(
            pack(vec!["text:5345454448414D4D4552".into()], None, ITER),
            Err(SyswError::Unclassifiable(
                0,
                UnknownReason::NonHexBody(record::TEXT_PREFIX)
            ))
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
