//! Passphrase generation for the systemwide container.
//!
//! SEPARATE from [`crate::seal::passphrase`] rather than an extension of it,
//! and the separation matters: `seal::passphrase::generate` produces a valid
//! 12-word BIP-39 **mnemonic**, checksum and all, and Sealed Payload is frozen
//! (spec decision 1). This produces `N` words drawn uniformly with **no
//! checksum**, because spec §8b requires exactly that — a checksum at any
//! length is what made 15 of 16 generated passphrases unopenable (R1-C2).
//!
//! Two functions that look alike and must not be merged.
//!
//! Normalisation is NOT duplicated: [`crate::seal::passphrase::normalise`] is
//! the single implementation, because spec §8a requires host and device to
//! produce byte-identical KDF input and a second normaliser would be a second
//! answer to that question.

use rand::Rng;
use zeroize::Zeroizing;

use super::wire::{WORDS_MAX, WORDS_MIN};

#[derive(Debug, PartialEq, Eq)]
pub enum GenError {
    /// `n` outside `[WORDS_MIN, WORDS_MAX]`.
    WordCount(usize),
}

/// `n` words drawn uniformly from the BIP-39 English wordlist, space-joined and
/// already in §8.1 normalised form (the list is lowercase and single spaces are
/// the separator, so no post-processing is needed or wanted).
///
/// **Uniform, and not via `Mnemonic::from_entropy_in`.** That constructor emits
/// a checksummed mnemonic, which only exists at 12/15/18/21/24 words and which
/// the device's word keyboard would then gate on — the exact defect §8b was
/// ruled to remove.
pub fn generate(n: usize) -> Result<Zeroizing<String>, GenError> {
    if !(WORDS_MIN..=WORDS_MAX).contains(&n) {
        return Err(GenError::WordCount(n));
    }
    let list = bip39::Language::English.word_list();
    let mut rng = rand::rng();
    let mut out = Zeroizing::new(String::with_capacity(n * 9));
    for i in 0..n {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(list[rng.random_range(0..list.len())]);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysw::cliff_above;

    #[test]
    fn generates_exactly_n_words_across_the_whole_legal_range() {
        for n in WORDS_MIN..=WORDS_MAX {
            let p = generate(n).unwrap();
            assert_eq!(p.split(' ').count(), n, "n = {n}");
        }
    }

    /// R1-C2, as a test rather than a promise. A generated passphrase must be
    /// enterable at EVERY length, including every `n % 3 == 0` — those are the
    /// ones a checksum gate would reject 15 times in 16.
    #[test]
    fn no_length_produces_a_checksummed_mnemonic_requirement() {
        for n in [3, 6, 9, 12, 15, 18, 21, 24] {
            // Draw several: a single draw passes 1 time in 16 by luck, which is
            // exactly how this defect survived review once already.
            let mut non_mnemonic = 0;
            for _ in 0..32 {
                let p = generate(n).unwrap();
                if bip39::Mnemonic::parse_normalized(&p).is_err() {
                    non_mnemonic += 1;
                }
            }
            assert!(
                non_mnemonic > 0,
                "at n = {n} every draw happened to be a valid mnemonic — either \
                 astronomically unlucky or generation is checksumming"
            );
        }
    }

    #[test]
    fn the_range_is_enforced_at_both_ends() {
        assert_eq!(
            generate(WORDS_MIN - 1),
            Err(GenError::WordCount(WORDS_MIN - 1))
        );
        assert_eq!(
            generate(WORDS_MAX + 1),
            Err(GenError::WordCount(WORDS_MAX + 1))
        );
        assert!(generate(WORDS_MIN).is_ok());
        assert!(generate(WORDS_MAX).is_ok());
    }

    /// Every word must come from the list, or `[cliff]` would score a generated
    /// passphrase below its own threshold.
    #[test]
    fn a_generated_passphrase_of_five_or_more_is_cliff_above() {
        for n in 5..=WORDS_MAX {
            assert!(cliff_above(&generate(n).unwrap()), "n = {n}");
        }
        for n in WORDS_MIN..5 {
            assert!(
                !cliff_above(&generate(n).unwrap()),
                "n = {n} is below by count"
            );
        }
    }

    #[test]
    fn output_is_already_normalised() {
        let p = generate(8).unwrap();
        assert_eq!(&*crate::seal::passphrase::normalise(&p), &*p);
    }

    /// Not a randomness test — a wiring test. Two draws colliding at n = 12
    /// means the RNG is not being advanced.
    #[test]
    fn successive_draws_differ() {
        assert_ne!(*generate(12).unwrap(), *generate(12).unwrap());
    }
}
