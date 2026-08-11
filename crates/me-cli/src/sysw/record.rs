//! Record classes for the systemwide container, and the two encoded ones.
//!
//! SPEC_systemwide_payloads §3.3.1 and §5.3.1. The normative rules live there;
//! this file implements them and restates none.
//!
//! WHY FREE TEXT AND PASSPHRASES ARE HEX-ENCODED, in one paragraph, because it
//! is the least obvious thing here: EPD §6.4 requires every record to be "the
//! canonical, unbroken string — no interior spaces, no hyphens, no grouping of
//! any kind", and uses LF as the record separator on the stated grounds that no
//! constellation string contains a newline. `Hello, World!` has a space,
//! `correct horse battery staple` has three, and Engrave Text's keyboard has a
//! newline key. Both new classes therefore break both clauses. Relaxing §6.4
//! for two classes would weaken it for all of them — records are engraved
//! VERBATIM, so a record carrying uncovered separator characters turns a scratch
//! on the operator's only copy into silently-absorbed damage. So the body is
//! encoded and the record stays canonical.
//!
//! Lowercase hex rather than base64 or base32: EPD §6.6 hashes the section in
//! its canonical LOWERCASE form, and hex is the only common encoding that
//! survives lowercasing unchanged.

use zeroize::Zeroizing;

/// Prefixes are RESERVED. A record beginning with one whose body is not valid
/// lowercase hex is [`Class::Unknown`] and refused — never quietly treated as
/// free text, which would let a malformed record become an engraved plate.
pub const TEXT_PREFIX: &str = "text:";
pub const PASS_PREFIX: &str = "pass:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Mnemonic,
    Codex32Secret,
    Passphrase,
    FreeText,
    Descriptor,
    MdMk,
    Address,
    Unknown,
}

impl Class {
    /// §3.3.1. Extends the shipped predicate (`seal/session.go:17`, which is
    /// `ClassCodex32Secret || ClassMnemonic`) with [`Class::Passphrase`].
    ///
    /// [`Class::FreeText`] is deliberately NOT secret even though an operator
    /// may put anything in it: a class states what the format guarantees, not
    /// what a human might do, and a class claiming secrecy it cannot enforce is
    /// the over-claim F-123 was filed against.
    pub fn is_secret(self) -> bool {
        matches!(
            self,
            Class::Mnemonic | Class::Codex32Secret | Class::Passphrase
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecordError {
    /// A reserved prefix with a body that is not valid lowercase hex.
    BadHex,
    /// Hex decoded, but the bytes are not UTF-8.
    NotUtf8,
    /// Not one of the encoded classes.
    NotEncoded,
}

/// Encode free text as a canonical record.
pub fn encode_text(s: &str) -> String {
    format!("{TEXT_PREFIX}{}", hex_lower(s.as_bytes()))
}

/// Encode a passphrase as a canonical record.
///
/// `Zeroizing` because the intermediate hex is as sensitive as the passphrase:
/// it is a reversible encoding, not a hash.
pub fn encode_pass(s: &str) -> Zeroizing<String> {
    Zeroizing::new(format!("{PASS_PREFIX}{}", hex_lower(s.as_bytes())))
}

/// Decode a `text:` or `pass:` record's body back to its bytes.
pub fn decode_body(record: &str) -> Result<Zeroizing<Vec<u8>>, RecordError> {
    let body = record
        .strip_prefix(TEXT_PREFIX)
        .or_else(|| record.strip_prefix(PASS_PREFIX))
        .ok_or(RecordError::NotEncoded)?;
    unhex_lower(body).ok_or(RecordError::BadHex)
}

/// Decode to a `String`, for the consumers that need text rather than bytes.
pub fn decode_text(record: &str) -> Result<Zeroizing<String>, RecordError> {
    let b = decode_body(record)?;
    let s = std::str::from_utf8(&b).map_err(|_| RecordError::NotUtf8)?;
    Ok(Zeroizing::new(s.to_owned()))
}

fn hex_lower(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Strictly lowercase, strictly even-length. Uppercase is rejected rather than
/// accepted-and-lowercased: EPD §6.6 hashes the record as it appears on the
/// wire, so two spellings of one body would be two different digests.
fn unhex_lower(s: &str) -> Option<Zeroizing<Vec<u8>>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Zeroizing::new(Vec::with_capacity(s.len() / 2));
    let b = s.as_bytes();
    for pair in b.chunks(2) {
        let hi = nibble(pair[0])?;
        let lo = nibble(pair[1])?;
        // `|` and `^` are EQUIVALENT here and cargo-mutants reports the swap as
        // missed. It is a true equivalent mutant, not a coverage gap: `hi << 4`
        // occupies bits 4..7 and `lo` bits 0..3, so the operands never share a
        // set bit and both operators yield the same byte for every input. Left
        // as `|` because it states the intent; recorded so the next run does not
        // spend a round rediscovering it.
        out.push(hi << 4 | lo);
    }
    Some(out)
}

fn nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_the_characters_epd_6_4_forbids_raw() {
        // Vector S-G. This is the case the whole encoding exists for: a space,
        // a newline (the record SEPARATOR itself) and a non-ASCII byte.
        let s = "Hello, World!\nsecond line — é";
        let rec = encode_text(s);
        assert!(
            !rec.contains(' '),
            "an encoded record may not contain a space"
        );
        assert!(
            !rec.contains('\n'),
            "an encoded record may not contain the separator"
        );
        assert!(rec.is_ascii(), "an encoded record must be ASCII");
        assert_eq!(&*decode_text(&rec).unwrap(), s);
    }

    #[test]
    fn encoding_is_lowercase_so_it_survives_canonicalisation() {
        // EPD §6.6 hashes the LOWERCASE form. An encoding with uppercase would
        // hash differently before and after canonicalisation.
        let rec = encode_text("\u{00FF}\u{00AB}");
        assert_eq!(rec, "text:c3bfc2ab");
        assert_eq!(rec, rec.to_lowercase());
    }

    #[test]
    fn uppercase_hex_is_refused_not_accepted() {
        assert_eq!(decode_body("text:C3BF"), Err(RecordError::BadHex));
    }

    #[test]
    fn a_reserved_prefix_with_a_bad_body_is_an_error_not_free_text() {
        // The reservation: never quietly treat a malformed record as text.
        for bad in ["text:zz", "text:abc", "pass:!!", "text:"] {
            if bad == "text:" {
                assert_eq!(&**decode_body(bad).unwrap(), b"", "empty body is valid hex");
                continue;
            }
            assert_eq!(decode_body(bad), Err(RecordError::BadHex), "{bad}");
        }
    }

    #[test]
    fn an_unprefixed_record_is_not_encoded() {
        assert_eq!(decode_body("md1qqq"), Err(RecordError::NotEncoded));
    }

    #[test]
    fn passphrase_is_secret_and_free_text_is_not() {
        assert!(Class::Passphrase.is_secret());
        assert!(Class::Mnemonic.is_secret());
        assert!(Class::Codex32Secret.is_secret());
        assert!(
            !Class::FreeText.is_secret(),
            "a class may not claim secrecy it cannot enforce"
        );
        assert!(!Class::MdMk.is_secret());
        assert!(!Class::Descriptor.is_secret());
        assert!(!Class::Address.is_secret());
        assert!(!Class::Unknown.is_secret());
    }

    /// `encode_pass` had NO test until cargo-mutants pointed it out: three of its
    /// mutants — returning an empty string, and returning `"xyzzy"` — all
    /// survived. My own nine hand-written mutants missed it entirely, because I
    /// mutated the code I was thinking about and not the function I had written
    /// and forgotten to exercise. That is the argument for generating mutants
    /// from the AST rather than from the author's memory.
    #[test]
    fn pass_round_trips_and_is_prefixed() {
        let secret = "abandon abandon abandon abandon abandon";
        let rec = encode_pass(secret);
        assert!(rec.starts_with(PASS_PREFIX), "must carry the reserved prefix");
        assert_ne!(&*rec, PASS_PREFIX, "an empty body is not an encoding of this");
        assert!(!rec.contains(' '), "the encoded record may not contain a space");
        assert_eq!(&*decode_text(&rec).unwrap(), secret);
    }

    /// The two encoders must not be interchangeable: a `pass:` record consumed
    /// as free text would engrave a passphrase onto a plate.
    #[test]
    fn text_and_pass_have_different_prefixes_for_the_same_body() {
        let body = "hello";
        assert_ne!(encode_text(body), *encode_pass(body));
        assert_eq!(
            decode_body(&encode_text(body)).unwrap().to_vec(),
            decode_body(&encode_pass(body)).unwrap().to_vec(),
            "same bytes, different prefix"
        );
    }

    #[test]
    fn empty_text_round_trips() {
        assert_eq!(&*decode_text(&encode_text("")).unwrap(), "");
    }
}
