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
    (0..8)
        .map(|i| &hex[i * 4..i * 4 + 4])
        .collect::<Vec<_>>()
        .join(" ")
}

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
    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    /// §11.4 vectors D and E, asserted as LITERALS — not merely as differing.
    /// An agreement-only assertion is satisfied by any deterministic function of
    /// any subset of these bytes, because D and E share their public section.
    #[test]
    fn matches_the_pinned_literals() {
        assert_eq!(
            hex(&public_data_hash(&public(), true)),
            "a26ed22bb747dfd0236706ad14c19679",
            "vector D (sealed)"
        );
        assert_eq!(
            hex(&public_data_hash(&public(), false)),
            "70f3e35aacf747dbc40f837691aa61e0",
            "vector E (unsealed)"
        );
    }

    /// THE downgrade detector. An earlier draft required these to AGREE, which
    /// is exactly the blindness a ciphertext-strip needs.
    #[test]
    fn sealed_and_unsealed_differ() {
        assert_ne!(
            public_data_hash(&public(), true),
            public_data_hash(&public(), false)
        );
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
            assert_ne!(
                public_data_hash(&refs, false),
                base,
                "{label} must change the hash"
            );
        }
    }

    /// `public_record_count` is bound in, so a removed record is visible.
    #[test]
    fn removing_a_record_changes_the_hash() {
        let p = public();
        assert_ne!(
            public_data_hash(&p[..4], false),
            public_data_hash(&p, false)
        );
    }

    #[test]
    fn formats_in_groups_of_four() {
        assert_eq!(
            format_hash(&public_data_hash(&public(), false)),
            "70f3 e35a acf7 47db c40f 8376 91aa 61e0"
        );
    }
}
