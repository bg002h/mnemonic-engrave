//! The overwrite artefact — SPEC_systemwide_payloads §5.5.
//!
//! It is a RAW REGION IMAGE, not a container: exactly `REGION_LEN` bytes of
//! fill, no magic, no header, no records. Deliberately unparseable — a reader
//! finding it sees no magic and reports "no payload", which is the correct
//! answer.
//!
//! It is not a zero-LENGTH payload, and that distinction is the whole point: a
//! zero-length payload rewrites the header and leaves every byte of the old body
//! sitting in flash, which looks like erasure and is not.

use rand::RngCore;
use zeroize::Zeroizing;

use super::wire::REGION_LEN;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fill {
    /// The default. Where the goal is that nothing be inferable from the
    /// residue, random is the only fill that achieves it.
    #[default]
    Random,
    Zeros,
    /// NOTE: `0xFF` is the ERASED state of NOR flash, so a region written to
    /// ones is indistinguishable from one erased and never written. Deniability
    /// if that is wanted; a loss of evidence if the operator wanted to show they
    /// acted deliberately. The three fills are not interchangeable.
    Ones,
}

pub fn region_image(fill: Fill) -> Zeroizing<Vec<u8>> {
    let mut out = Zeroizing::new(vec![0u8; REGION_LEN]);
    match fill {
        Fill::Zeros => {}
        Fill::Ones => out.iter_mut().for_each(|b| *b = 0xFF),
        Fill::Random => rand::rng().fill_bytes(&mut out[..]),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysw::wire::MAGIC;

    /// Spec test 11: it FILLS the region. A zero-length payload must fail this,
    /// which is the defect the requirement exists to prevent.
    #[test]
    fn fills_the_whole_region() {
        for f in [Fill::Random, Fill::Zeros, Fill::Ones] {
            assert_eq!(region_image(f).len(), REGION_LEN, "{f:?}");
        }
    }

    /// Spec test 12: each fill produces the region it claims, and ones is
    /// byte-identical to an erased region.
    #[test]
    fn each_fill_is_what_it_says() {
        assert!(region_image(Fill::Zeros).iter().all(|&b| b == 0x00));
        assert!(region_image(Fill::Ones).iter().all(|&b| b == 0xFF));
        assert_eq!(
            &region_image(Fill::Ones)[..],
            &vec![0xFFu8; REGION_LEN][..],
            "ones must equal an erased region"
        );
        let r = region_image(Fill::Random);
        assert!(!r.iter().all(|&b| b == r[0]), "random must not be constant");
    }

    /// Unparseable by design: no magic, so a reader reports "no payload".
    #[test]
    fn carries_no_magic() {
        for f in [Fill::Random, Fill::Zeros, Fill::Ones] {
            assert_ne!(&region_image(f)[..8], &MAGIC[..], "{f:?}");
        }
    }

    #[test]
    fn random_is_the_default() {
        assert_eq!(Fill::default(), Fill::Random);
    }
}
