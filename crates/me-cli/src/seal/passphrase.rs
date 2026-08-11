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
/// Builds the result directly into ONE `Zeroizing` buffer.
///
/// The obvious spelling — `split_whitespace().map(to_lowercase).collect::<Vec<_>>().join(" ")` —
/// returns a `Zeroizing<String>` while leaving three unscrubbed allocations
/// behind it: a `String` per lowercased word, the `Vec` collecting them, and the
/// `join`'s own buffer. All hold the passphrase, and the wrapper reaches none of
/// them. Measured at 3 leaked blocks per call by the Phase 2 Rust-side review.
///
/// The length is counted before allocating so the buffer never grows: a `String`
/// that reallocates mid-build orphans the partially-written copy, which is the
/// same defect one level down. Lowercasing can lengthen a character (`İ` becomes
/// two), so the count cannot be taken from `s.len()`.
pub fn normalise(s: &str) -> Zeroizing<String> {
    let mut n = 0usize;
    for (i, w) in s.split_whitespace().enumerate() {
        if i > 0 {
            n += 1; // the single ASCII separator
        }
        for c in w.chars().flat_map(char::to_lowercase) {
            n += c.len_utf8();
        }
    }

    let mut out = Zeroizing::new(String::with_capacity(n));
    for (i, w) in s.split_whitespace().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        for c in w.chars().flat_map(char::to_lowercase) {
            out.push(c);
        }
    }
    debug_assert_eq!(out.len(), n, "normalise: pre-count disagreed with the build");
    out
}

/// Checksum-valid English mnemonic? The device runs this before committing to a
/// ~31 s KDF, so a typo costs a second rather than half a minute.
pub fn is_valid(s: &str) -> bool {
    Mnemonic::parse_in(Language::English, &*normalise(s)).is_ok()
}

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
        assert!(is_valid(
            "beef beef beef beef beef beef beef beef beef beef beef beef"
        ));
    }

    /// beef x11 + bacon is a valid-length mnemonic of real words differing in
    /// one position, and checksum-INVALID. A gate that passes it is broken.
    #[test]
    fn rejects_near_miss_and_invalid() {
        assert!(!is_valid(
            "beef beef beef beef beef beef beef beef beef beef beef bacon"
        ));
        assert!(!is_valid("abandon ".repeat(12).trim()));
        assert!(!is_valid("not even words"));
    }

    /// The buffer must be sized exactly, so it never grows mid-build and never
    /// orphans a partially-written copy of the passphrase.
    ///
    /// `capacity == len` is the observable form of that: `with_capacity(n)`
    /// allocates exactly `n`, so if the pre-count were short the push would have
    /// reallocated and capacity would exceed len.
    #[test]
    fn normalise_allocates_exactly_once_and_exactly_enough() {
        for input in [
            "abandon abandon about",
            "  ABANDON\tabandon\u{00a0}about  ",
            "İ İ",           // lowercasing lengthens this one
            "ÄÖÜ  ßß\tàé",   // multi-byte, mixed case, runs
        ] {
            let out = normalise(input);
            assert_eq!(
                out.capacity(),
                out.len(),
                "normalise({input:?}) reallocated: capacity {} != len {} -- a grown \
                 String orphans the partial copy it was holding",
                out.capacity(),
                out.len()
            );
        }
    }

    #[test]
    fn normalise_collapses_whitespace_and_case() {
        assert_eq!(*normalise("  BEEF   beef\tbeef  "), "beef beef beef");
    }
}
