//! Payload identity — SPEC_systemwide_payloads `[identity]` (§12.3).
//!
//! This is NOT the displayed digest, and the distinction cost an R0 round.
//!
//! `[digest-shown]` is EPD §6.6's public-data hash, which by that section's own
//! rule is displayed only when `pub_len > 0`: the digest of an empty record set
//! is a constant. A secrets-only sealed payload therefore has no such value at
//! all. An earlier draft made it the session's identity anyway, which would have
//! given EVERY secrets-only payload one shared identity — so a payload swapped
//! between two consumptions would inherit the previous one's `[compared]` flag
//! and be treated as authenticated on a comparison the operator never made.
//!
//! So identity is a separate hash, over the REGION BYTES: it always exists, it
//! is content-derived, and it covers the ciphertext, which the §6.6 digest
//! deliberately does not.
//!
//! 32 bytes, not 16. This value is never read by a human — it is an equality
//! key — so there is no reason to truncate it, and truncating an equality key is
//! how collisions become someone else's problem.

use sha2::{Digest, Sha256};

const LABEL: &[u8] = b"MNEMSYSW/id/v1";

/// `SHA-256(LABEL ‖ 0x00 ‖ region)`
///
/// `region` is the payload's bytes as read, bounded by the header's declared
/// total. That bound is attacker-controlled, which is safe here **because the
/// bound is inside the hash**: a different declared length hashes differently,
/// so no two regions collide and no region yields two identities.
pub fn identity(region: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(LABEL);
    h.update([0x00]);
    h.update(region);
    h.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysw::pubhash;

    #[test]
    fn is_not_the_displayed_digest() {
        // Different label, different length, different input. If these ever
        // agreed, [identity] would inherit [digest-shown]'s pub_len == 0 hole.
        let region = b"MNEMSYSW....";
        assert_ne!(
            &identity(region)[..16],
            &pubhash::public_data_hash(&["x"], true)[..]
        );
    }

    #[test]
    fn exists_for_an_empty_region() {
        // The whole point: a secrets-only payload has no public section and so
        // no displayed digest, but it still has an identity.
        let _ = identity(&[]);
    }

    /// The bypass this construction exists to close: two DIFFERENT payloads must
    /// never share an identity, or the second inherits the first's `[compared]`.
    #[test]
    fn different_regions_have_different_identities() {
        assert_ne!(identity(b"payload one"), identity(b"payload two"));
    }

    /// The declared-length bound is inside the hash, so a region truncated to a
    /// different declared total is a different identity rather than the same one.
    #[test]
    fn a_different_bound_is_a_different_identity() {
        let full = b"MNEMSYSW\x01\x02\x03\x04\x05\x06";
        assert_ne!(identity(full), identity(&full[..full.len() - 1]));
    }

    #[test]
    fn is_stable() {
        assert_eq!(identity(b"abc"), identity(b"abc"));
    }
}
