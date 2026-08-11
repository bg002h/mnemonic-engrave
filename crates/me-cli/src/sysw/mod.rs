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

#[cfg(test)]
mod tests {
    use super::*;

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
