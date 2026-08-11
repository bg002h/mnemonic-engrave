//! Wire format constants — SPEC_systemwide_payloads §4, §4.1, §12.5.
//!
//! The header layout is EPD §6's, byte for byte, so the two containers stay
//! structurally comparable and one decoder's bounds reasoning transfers to the
//! other. Only the magic differs.

/// §4.1. Eight bytes, matching `MNEMBLOB`'s width so both containers present a
/// same-width discriminator at offset 0.
pub const MAGIC: [u8; 8] = *b"MNEMSYSW";

/// §4. Fixed and normative, for the same reason `seal::PayloadAddr` is: any
/// other value produces a blob the device never looks at. A full megabyte below
/// the Sealed Payload region at `0x10E00000`, so an overrun in either direction
/// hits unprogrammed flash rather than the other feature's data.
pub const REGION_ADDR: u32 = 0x10D0_0000;

/// 64 KiB — 16 × 4 KiB sectors.
pub const REGION_LEN: usize = 65_536;

/// §12.5. Over the NORMALISED string, host and device.
///
/// 215 = `bip39::LongestWord` (8) × 24 words + 23 separators. NOT
/// `passphrase::MaxLen`, which is 100 and is by its own comment "a plate-capacity
/// limit chosen for legibility" — a fact about steel, not about entry.
pub const PASSPHRASE_MAX: usize = 215;

/// §12.5. Generated-mode word count.
pub const WORDS_MIN: usize = 2;
pub const WORDS_MAX: usize = 24;
pub const WORDS_DEFAULT: usize = 12;

/// `passphrase::MaxLen` is 100 and MUST NOT bound this path — applying it would
/// make every long generated passphrase unenterable, which is the
/// host-seals-what-the-device-refuses shape that cost this cycle three separate
/// defects. A const assertion, so it fails the BUILD rather than a test.
const _: () = assert!(PASSPHRASE_MAX > 100);
const _: () = assert!(REGION_LEN.is_multiple_of(4096));
const _: () = assert!((REGION_ADDR as usize + REGION_LEN) <= 0x10E0_0000);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_is_distinguishable_from_the_sealed_payload_container() {
        assert_eq!(MAGIC.len(), 8);
        assert_ne!(&MAGIC[..], b"MNEMBLOB");
    }

    /// §4's clearance argument, as arithmetic rather than prose.
    #[test]
    fn the_region_clears_everything_it_must() {
        let end = REGION_ADDR as usize + REGION_LEN;
        assert!(
            REGION_ADDR as usize > 0x1013_6000,
            "above the sector picotool touches"
        );
        assert!(
            end <= 0x10E0_0000,
            "a full megabyte below the Sealed Payload region"
        );
        assert!(
            end < 0x10FF_F000,
            "below the top sector --abs-block could clobber"
        );
        assert!(
            end < 0x1100_0000,
            "inside physical flash; past this a write wraps and kills the firmware"
        );
        assert_eq!(REGION_LEN % 4096, 0, "whole 4 KiB sectors");
        assert_eq!(REGION_ADDR as usize % 4096, 0, "sector aligned");
    }

    /// The measured maximum, so a 24-word passphrase is enterable. Getting this
    /// from `passphrase::MaxLen` (100) would make every long generated
    /// passphrase unopenable — the same host-seals-what-the-device-refuses shape
    /// that cost this cycle three separate defects.
    #[test]
    fn passphrase_max_admits_the_longest_generated_passphrase() {
        let longest = bip39::Language::English
            .word_list()
            .iter()
            .map(|w| w.len())
            .max()
            .unwrap();
        assert_eq!(longest, 8, "BIP-39 English longest word");
        assert_eq!(PASSPHRASE_MAX, longest * WORDS_MAX + (WORDS_MAX - 1));
    }
}
