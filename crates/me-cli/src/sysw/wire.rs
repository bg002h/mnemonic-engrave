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

pub const VERSION: u8 = 0x01;
pub const KDF_PBKDF2_SHA256: u8 = 0x01;
pub const AEAD_AES256GCM: u8 = 0x01;
pub const HEADER_LEN: usize = 52;
pub const SALT_LEN: usize = 16;
pub const IV_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

/// EPD §6's cap, inherited unchanged: 8191 rather than 8192 because the
/// device's scan buffer signals overflow when it is exactly FULL.
pub const MAX_SECTION_LEN: usize = 8191;
pub const MIN_ITERATIONS: u32 = 100_000;
pub const MAX_ITERATIONS: u32 = 2_000_000;

/// `passphrase::MaxLen` is 100 and MUST NOT bound this path — applying it would
/// make every long generated passphrase unenterable, which is the
/// host-seals-what-the-device-refuses shape that cost this cycle three separate
/// defects. A const assertion, so it fails the BUILD rather than a test.
const _: () = assert!(PASSPHRASE_MAX > 100);
const _: () = assert!(REGION_LEN.is_multiple_of(4096));
const _: () = assert!((REGION_ADDR as usize + REGION_LEN) <= 0x10E0_0000);

/// The 52-byte header. EPD §6's layout byte for byte — only the magic differs —
/// so one decoder's bounds reasoning transfers to the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Header {
    /// 0 when nothing is encrypted.
    pub iterations: u32,
    pub salt: [u8; SALT_LEN],
    pub iv: [u8; IV_LEN],
    pub pub_len: u32,
    /// 0 when nothing is encrypted — and then there is no tag either.
    pub ct_len: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WireError {
    TooShort(usize),
    BadMagic,
    UnknownVersion(u8),
    UnknownKdf(u8),
    UnknownAead(u8),
    SectionTooLong,
    Iterations(u32),
}

impl Header {
    pub fn sealed(&self) -> bool {
        self.ct_len > 0
    }

    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let s = self.sealed();
        let mut out = [0u8; HEADER_LEN];
        out[..8].copy_from_slice(&MAGIC);
        out[8] = VERSION;
        out[9] = if s { KDF_PBKDF2_SHA256 } else { 0 };
        out[10] = if s { AEAD_AES256GCM } else { 0 };
        out[11] = 0; // reserved
        out[12..16].copy_from_slice(&self.iterations.to_be_bytes());
        out[16..32].copy_from_slice(&self.salt);
        out[32..44].copy_from_slice(&self.iv);
        out[44..48].copy_from_slice(&self.pub_len.to_be_bytes());
        out[48..52].copy_from_slice(&self.ct_len.to_be_bytes());
        out
    }

    /// Parse and bound-check. **Every check runs before any KDF work** — the
    /// firmware has no active watchdog, so an unbounded iteration count is a
    /// hang, and both section lengths are attacker-controlled until proven
    /// otherwise. Nothing below may be used for arithmetic before its bound.
    pub fn parse(buf: &[u8]) -> Result<Self, WireError> {
        if buf.len() < HEADER_LEN {
            return Err(WireError::TooShort(buf.len()));
        }
        if buf[..8] != MAGIC {
            return Err(WireError::BadMagic);
        }
        if buf[8] != VERSION {
            return Err(WireError::UnknownVersion(buf[8]));
        }
        let mut salt = [0u8; SALT_LEN];
        let mut iv = [0u8; IV_LEN];
        salt.copy_from_slice(&buf[16..32]);
        iv.copy_from_slice(&buf[32..44]);
        let h = Header {
            iterations: u32::from_be_bytes(buf[12..16].try_into().unwrap()),
            salt,
            iv,
            pub_len: u32::from_be_bytes(buf[44..48].try_into().unwrap()),
            ct_len: u32::from_be_bytes(buf[48..52].try_into().unwrap()),
        };
        if h.pub_len as usize > MAX_SECTION_LEN || h.ct_len as usize > MAX_SECTION_LEN {
            return Err(WireError::SectionTooLong);
        }
        if h.sealed() {
            if buf[9] != KDF_PBKDF2_SHA256 {
                return Err(WireError::UnknownKdf(buf[9]));
            }
            if buf[10] != AEAD_AES256GCM {
                return Err(WireError::UnknownAead(buf[10]));
            }
            if !(MIN_ITERATIONS..=MAX_ITERATIONS).contains(&h.iterations) {
                return Err(WireError::Iterations(h.iterations));
            }
        }
        Ok(h)
    }

    /// Total on-wire length, tag included. Safe only AFTER [`Header::parse`],
    /// which is what proves both lengths are bounded and the sum cannot wrap.
    pub fn total_len(&self) -> usize {
        HEADER_LEN
            + self.pub_len as usize
            + self.ct_len as usize
            + if self.sealed() { TAG_LEN } else { 0 }
    }
}

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
