//! The displayed digest — SPEC_systemwide_payloads `[digest-shown]` (§12.4).
//!
//! EPD §6.6's construction verbatim, with ONE change: the domain-separation
//! label. That label exists to stop cross-context collisions, and §4 requires
//! the two containers be distinguishable — reusing `MNEMBLOB/pub/v1` would make
//! two containers the spec deliberately separates produce identical digests over
//! identical public sections.
//!
//! COMPUTED, NEVER STORED, for EPD §6.6's reason: a hash carried inside the
//! payload is rewritten by whoever rewrites the records, and the device would
//! display a value matching the tampered data perfectly.

use sha2::{Digest, Sha256};
use std::fmt::Write as _;

const LABEL: &[u8] = b"MNEMSYSW/pub/v1";

/// `SHA-256(LABEL ‖ 0x00 ‖ sealed ‖ public_record_count ‖ input)[..16]`
///
/// `sealed` is bound in so a downgrade is visible: stripping the ciphertext
/// changes the digest rather than preserving it.
pub fn public_data_hash(records: &[&str], sealed: bool) -> [u8; 16] {
    let mut h = Sha256::new();
    h.update(LABEL);
    h.update([0x00]);
    h.update([u8::from(sealed)]);
    h.update([records.len() as u8]);
    h.update(records.join("\n").as_bytes());
    let d = h.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&d[..16]);
    out
}

/// Grouped in fours so a human can compare it without losing their place.
pub fn format_hash(h: &[u8; 16]) -> String {
    let hex: String = h.iter().fold(String::new(), |mut acc, b| {
        let _ = write!(acc, "{b:02x}");
        acc
    });
    (0..8)
        .map(|i| &hex[i * 4..i * 4 + 4])
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seal::pubhash as blob;

    fn recs() -> Vec<&'static str> {
        vec![
            "text:48656c6c6f",
            "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3",
        ]
    }

    /// THE reason this file exists rather than a call into `seal::pubhash`.
    /// If the labels ever converge, two containers §4 separates would digest
    /// identically over identical sections.
    #[test]
    fn differs_from_the_sealed_payload_digest_over_the_same_records() {
        assert_ne!(
            public_data_hash(&recs(), false),
            blob::public_data_hash(&recs(), false),
            "the systemwide and MNEMBLOB labels must not collide"
        );
    }

    /// The downgrade detector. Requiring these to AGREE is exactly the blindness
    /// a ciphertext-strip needs.
    #[test]
    fn sealed_and_unsealed_differ() {
        assert_ne!(
            public_data_hash(&recs(), true),
            public_data_hash(&recs(), false)
        );
    }

    /// `public_record_count` is bound in, so a REMOVED record is visible and not
    /// merely a changed one.
    ///
    /// The first version of this test compared `recs()[..1]` with `recs()`,
    /// whose JOINED content also differs — so it passed with the count byte
    /// deleted. Found by mutation, not by reading. The case the count byte
    /// actually resolves is two sets that join IDENTICALLY: `["ab","cd"]` and
    /// `["ab\ncd"]` are both `"ab\ncd"` on the wire, and only the count tells
    /// them apart. Without it, one record containing a newline is
    /// indistinguishable from two records.
    #[test]
    fn record_count_distinguishes_sets_with_identical_joined_content() {
        let two = ["ab", "cd"];
        let one = ["ab\ncd"];
        assert_eq!(
            two.join("\n"),
            one.join("\n"),
            "the sets must join identically"
        );
        assert_ne!(
            public_data_hash(&two, false),
            public_data_hash(&one, false),
            "the record count must distinguish them"
        );
    }

    #[test]
    fn removing_a_record_changes_the_digest() {
        assert_ne!(
            public_data_hash(&recs()[..1], false),
            public_data_hash(&recs(), false)
        );
    }

    /// Every byte of the section matters — kills subset and off-by-one mutants
    /// that an inequality between two whole sections cannot.
    #[test]
    fn every_byte_of_the_section_affects_the_digest() {
        let base = public_data_hash(&recs(), false);
        for which in [0usize, 1] {
            let mut r: Vec<String> = recs().iter().map(|s| (*s).to_owned()).collect();
            // Flip the FIRST byte of record 0 and the LAST of record 1, which
            // between them are the section's true first and last bytes.
            if which == 0 {
                let c = r[0].remove(0);
                r[0].insert(0, if c == 't' { 'u' } else { 't' });
            } else {
                let last = r.last_mut().unwrap();
                let c = last.pop().unwrap();
                last.push(if c == '3' { '4' } else { '3' });
            }
            let refs: Vec<&str> = r.iter().map(|s| s.as_str()).collect();
            assert_ne!(
                public_data_hash(&refs, false),
                base,
                "byte {which} must matter"
            );
        }
    }

    #[test]
    fn formats_in_groups_of_four() {
        let f = format_hash(&public_data_hash(&recs(), false));
        assert_eq!(f.len(), 39, "32 hex digits + 7 spaces");
        assert_eq!(f.split(' ').count(), 8);
        assert!(f.split(' ').all(|g| g.len() == 4));
    }
}
