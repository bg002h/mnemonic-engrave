//! The §6 wire header: 52 bytes, big-endian, and — together with the public
//! section — the AEAD's Associated Data.
//!
//! Parsed BEFORE authentication (it carries the salt and iteration count), so
//! it is hostile input by construction and every field is bound-checked here.

pub const MAGIC: [u8; 8] = *b"MNEMBLOB";
pub const VERSION: u8 = 0x01;
/// MNEMBLOB v2 (S1b §14.2): offset 11 is `pw_form`, not `reserved`.
pub const VERSION_V2: u8 = 0x02;
/// `pw_form` values 6 to 12 are the n-word password form (§14.5); 0 is §8's
/// twelve generated words.
pub const PW_WORDS_MIN: u8 = 6;
pub const PW_WORDS_MAX: u8 = 12;
pub const KDF_PBKDF2_SHA256: u8 = 0x01;
pub const AEAD_AES256GCM: u8 = 0x01;

pub const HEADER_LEN: usize = 52;
pub const SALT_LEN: usize = 16;
pub const IV_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

pub const MIN_ITERATIONS: u32 = 100_000;
pub const MAX_ITERATIONS: u32 = 2_000_000;
/// The floor for a `pw_form` 6 to 12 payload (S1b §14.2, RULED 2026-11-22).
pub const MIN_ITERATIONS_NWORD: u32 = 600_000;
/// One below the device's 8 KiB scan buffer: `gui/scan.go` flags overflow when
/// the buffer is exactly full.
pub const MAX_SECTION_LEN: u32 = 8191;
pub const REGION_LEN: u64 = 65_536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// `0x01` or `0x02` (S1b §14.2).
    pub version: u8,
    /// Offset 11. Always 0 for a v1 header (it encodes as the reserved byte);
    /// for v2, 0 or 6 to 12.
    pub pw_form: u8,
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
    ReservedNotZero(u8),
    Iterations(u32),
    /// v2 only: offset 11 is neither 0 nor 6 to 12.
    PwForm(u8),
    /// v2 only: `pw_form` 6 to 12 with `iterations` below 600,000.
    PwFormIterations(u32),
    PubLen(u32),
    CtLen(u32),
    Empty,
    UnsealedFieldNotZero(&'static str),
    TooLarge(u64),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::TooShort(n) => write!(f, "blob too short: {n} bytes, need at least 52"),
            WireError::BadMagic => write!(f, "not a sealed payload (bad magic)"),
            WireError::UnknownVersion(v) => write!(f, "unsupported format version {v}"),
            WireError::UnknownKdf(k) => write!(f, "unsupported kdf id {k}"),
            WireError::UnknownAead(a) => write!(f, "unsupported aead id {a}"),
            WireError::ReservedNotZero(b) => write!(f, "reserved byte must be 0, got {b}"),
            WireError::Iterations(n) => write!(
                f,
                "iteration count {n} out of range [{MIN_ITERATIONS}, {MAX_ITERATIONS}]"
            ),
            WireError::PwForm(b) => {
                write!(f, "password form byte must be 0 or 6 to 12, got {b}")
            }
            WireError::PwFormIterations(n) => write!(
                f,
                "iteration count {n} out of range [{MIN_ITERATIONS_NWORD}, {MAX_ITERATIONS}] \
                 for a 6 to 12 word password"
            ),
            WireError::PubLen(n) => {
                write!(f, "public section length {n} exceeds {MAX_SECTION_LEN}")
            }
            WireError::CtLen(n) => write!(f, "ciphertext length {n} exceeds {MAX_SECTION_LEN}"),
            WireError::Empty => write!(f, "payload is empty (pub_len and ct_len are both 0)"),
            WireError::UnsealedFieldNotZero(fld) => {
                write!(f, "{fld} must be zero when nothing is encrypted")
            }
            WireError::TooLarge(n) => write!(f, "blob is {n} bytes; the region is {REGION_LEN}"),
        }
    }
}
impl std::error::Error for WireError {}

impl Header {
    pub fn is_sealed(&self) -> bool {
        self.ct_len > 0
    }

    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let s = self.is_sealed();
        let mut out = [0u8; HEADER_LEN];
        out[..8].copy_from_slice(&MAGIC);
        out[8] = self.version;
        out[9] = if s { KDF_PBKDF2_SHA256 } else { 0 };
        out[10] = if s { AEAD_AES256GCM } else { 0 };
        out[11] = self.pw_form; // v1: the reserved byte, always 0
        out[12..16].copy_from_slice(&self.iterations.to_be_bytes());
        out[16..32].copy_from_slice(&self.salt);
        out[32..44].copy_from_slice(&self.iv);
        out[44..48].copy_from_slice(&self.pub_len.to_be_bytes());
        out[48..52].copy_from_slice(&self.ct_len.to_be_bytes());
        out
    }

    /// Parse and bound-check. Every check runs BEFORE any KDF work: the firmware
    /// has no active watchdog, so an unbounded iteration count is a hang.
    pub fn decode(buf: &[u8]) -> Result<Self, WireError> {
        if buf.len() < HEADER_LEN {
            return Err(WireError::TooShort(buf.len()));
        }
        if buf[..8] != MAGIC {
            return Err(WireError::BadMagic);
        }
        if buf[8] != VERSION {
            return Err(WireError::UnknownVersion(buf[8]));
        }
        if buf[11] != 0 {
            return Err(WireError::ReservedNotZero(buf[11]));
        }

        let iterations = u32::from_be_bytes(buf[12..16].try_into().unwrap());
        let pub_len = u32::from_be_bytes(buf[44..48].try_into().unwrap());
        let ct_len = u32::from_be_bytes(buf[48..52].try_into().unwrap());

        if pub_len > MAX_SECTION_LEN {
            return Err(WireError::PubLen(pub_len));
        }
        if ct_len > MAX_SECTION_LEN {
            return Err(WireError::CtLen(ct_len));
        }
        if pub_len == 0 && ct_len == 0 {
            return Err(WireError::Empty);
        }

        if ct_len > 0 {
            if buf[9] != KDF_PBKDF2_SHA256 {
                return Err(WireError::UnknownKdf(buf[9]));
            }
            if buf[10] != AEAD_AES256GCM {
                return Err(WireError::UnknownAead(buf[10]));
            }
            if !(MIN_ITERATIONS..=MAX_ITERATIONS).contains(&iterations) {
                return Err(WireError::Iterations(iterations));
            }
        } else {
            // §6.2's anti-downgrade-staging rule.
            if buf[9] != 0 {
                return Err(WireError::UnsealedFieldNotZero("kdf_id"));
            }
            if buf[10] != 0 {
                return Err(WireError::UnsealedFieldNotZero("aead_id"));
            }
            if iterations != 0 {
                return Err(WireError::UnsealedFieldNotZero("iterations"));
            }
            if buf[16..32].iter().any(|&b| b != 0) {
                return Err(WireError::UnsealedFieldNotZero("salt"));
            }
            if buf[32..44].iter().any(|&b| b != 0) {
                return Err(WireError::UnsealedFieldNotZero("iv"));
            }
        }

        // u64 deliberately: 32-bit arithmetic wraps for lengths near 2^32 and
        // would pass a <= 65536 test. The section caps above already protect a
        // conforming implementation; this must not be relied on alone.
        let tag = if ct_len > 0 { TAG_LEN as u64 } else { 0 };
        let total = HEADER_LEN as u64 + pub_len as u64 + ct_len as u64 + tag;
        if total > REGION_LEN {
            return Err(WireError::TooLarge(total));
        }

        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&buf[16..32]);
        let mut iv = [0u8; IV_LEN];
        iv.copy_from_slice(&buf[32..44]);
        Ok(Header {
            version: VERSION,
            pw_form: 0,
            iterations,
            salt,
            iv,
            pub_len,
            ct_len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sealed() -> Header {
        Header {
            version: VERSION,
            pw_form: 0,
            iterations: 100_000,
            salt: [0xbe; SALT_LEN],
            iv: [0xc0; IV_LEN],
            pub_len: 396,
            ct_len: 75,
        }
    }
    fn unsealed() -> Header {
        Header {
            version: VERSION,
            pw_form: 0,
            iterations: 0,
            salt: [0; SALT_LEN],
            iv: [0; IV_LEN],
            pub_len: 396,
            ct_len: 0,
        }
    }

    #[test]
    fn both_shapes_round_trip() {
        for h in [sealed(), unsealed()] {
            let b = h.encode();
            assert_eq!(b.len(), HEADER_LEN);
            assert_eq!(&b[..8], b"MNEMBLOB");
            assert_eq!(Header::decode(&b).unwrap(), h);
        }
        assert!(sealed().is_sealed());
        assert!(!unsealed().is_sealed());
    }

    #[test]
    fn sealed_shape_sets_algorithm_ids_and_unsealed_zeroes_them() {
        assert_eq!(sealed().encode()[9..11], [0x01, 0x01]);
        assert_eq!(unsealed().encode()[9..11], [0x00, 0x00]);
    }

    /// Covers every §6.2 header bound. `reserved`, and the SEALED-shape
    /// `kdf_id`/`aead_id` checks, are exercised here and nowhere else —
    /// mutation-proved: deleting all three checks left the other wire tests
    /// green, because `rejects_nonzero_crypto_fields_when_unsealed` only visits
    /// offsets 9/10 in the UNSEALED branch.
    #[test]
    fn rejects_bad_magic_version_reserved_kdf_and_aead() {
        let cases: &[(usize, u8, &str)] = &[
            (0, b'X', "magic"),
            (8, 0x03, "version"),
            (11, 0x01, "reserved"),
            (9, 0x02, "kdf_id"),
            (10, 0x02, "aead_id"),
        ];
        for &(off, val, label) in cases {
            let mut b = sealed().encode();
            b[off] = val;
            assert!(
                Header::decode(&b).is_err(),
                "{label} = {val:#x} must be refused"
            );
        }
        // Pin the variants so a mutation collapsing them into one is caught.
        let mut b = sealed().encode();
        b[11] = 0x01;
        assert!(matches!(
            Header::decode(&b),
            Err(WireError::ReservedNotZero(1))
        ));
        let mut b = sealed().encode();
        b[9] = 0x02;
        assert!(matches!(Header::decode(&b), Err(WireError::UnknownKdf(2))));
        let mut b = sealed().encode();
        b[10] = 0x02;
        assert!(matches!(Header::decode(&b), Err(WireError::UnknownAead(2))));
    }

    #[test]
    fn rejects_a_short_buffer() {
        assert!(matches!(
            Header::decode(&[0u8; 51]),
            Err(WireError::TooShort(51))
        ));
    }

    #[test]
    fn rejects_out_of_range_iterations_when_sealed() {
        for bad in [0u32, 99_999, 2_000_001, u32::MAX] {
            let mut h = sealed();
            h.iterations = bad;
            assert!(
                matches!(Header::decode(&h.encode()), Err(WireError::Iterations(_))),
                "iterations {bad} must be refused"
            );
        }
    }

    #[test]
    fn rejects_out_of_range_lengths() {
        for bad in [8192u32, 0xFFFF_FFF0, u32::MAX] {
            let mut h = sealed();
            h.ct_len = bad;
            assert!(
                matches!(Header::decode(&h.encode()), Err(WireError::CtLen(_))),
                "ct_len {bad}"
            );
            let mut h = sealed();
            h.pub_len = bad;
            assert!(
                matches!(Header::decode(&h.encode()), Err(WireError::PubLen(_))),
                "pub_len {bad}"
            );
        }
    }

    #[test]
    fn rejects_an_empty_payload() {
        let mut h = unsealed();
        h.pub_len = 0;
        assert!(matches!(Header::decode(&h.encode()), Err(WireError::Empty)));
    }

    /// §6.2: with ct_len == 0 the crypto fields MUST be zero. Junk there would
    /// let an attacker stage a downgrade a later version might honour.
    #[test]
    fn rejects_nonzero_crypto_fields_when_unsealed() {
        let base = unsealed().encode();
        for (off, val, label) in [
            (9usize, 0x01u8, "kdf_id"),
            (10, 0x01, "aead_id"),
            (15, 0x01, "iterations"),
            (16, 0x01, "salt"),
            (32, 0x01, "iv"),
        ] {
            let mut b = base;
            b[off] = val;
            assert!(
                matches!(Header::decode(&b), Err(WireError::UnsealedFieldNotZero(_))),
                "{label} must be zero when ct_len == 0"
            );
        }
    }

    // ── MNEMBLOB v2 (S1b §14.2, §14.10) ─────────────────────────────────────

    /// V2-A's header, from S1b §14.9.
    const V2_A_HEADER: &str = "4d4e454d424c4f4202010106000927c0feedfeedfeedfeedfeedfeedfeedfeedfacefacefacefacefaceface00000000000000f2";

    fn unhex(h: &str) -> Vec<u8> {
        (0..h.len() / 2)
            .map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap())
            .collect()
    }

    fn v2(pw_form: u8, iterations: u32) -> Header {
        Header {
            version: VERSION_V2,
            pw_form,
            iterations,
            salt: [0xbe; SALT_LEN],
            iv: [0xc0; IV_LEN],
            pub_len: 0,
            ct_len: 242,
        }
    }

    /// S1b §14.10: `pw_form` 0, 6 and 12 headers encode then decode equal;
    /// V2-A's header hex decodes to version 2, pw_form 6, 600,000 iterations,
    /// pub_len 0, ct_len 242.
    #[test]
    fn v2_header_round_trips_each_password_form() {
        for (pw, iters) in [(0u8, 100_000u32), (6, 600_000), (12, 600_000)] {
            let h = v2(pw, iters);
            let b = h.encode();
            assert_eq!(b[8], 0x02);
            assert_eq!(b[11], pw);
            assert_eq!(Header::decode(&b).unwrap(), h, "pw_form {pw}");
        }
        let a = Header::decode(&unhex(V2_A_HEADER)).unwrap();
        assert_eq!((a.version, a.pw_form), (2, 6));
        assert_eq!((a.iterations, a.pub_len, a.ct_len), (600_000, 0, 242));
    }

    /// S1b §14.10: each offset-11 edit of V2-A's header gives `PwForm(b)` and its
    /// text.
    #[test]
    fn v2_refuses_pw_form_outside_0_and_6_to_12() {
        for b in [0x01u8, 0x05, 0x0d, 0x80, 0xff] {
            let mut h = unhex(V2_A_HEADER);
            h[11] = b;
            match Header::decode(&h) {
                Err(e @ WireError::PwForm(got)) => {
                    assert_eq!(got, b);
                    assert_eq!(
                        e.to_string(),
                        format!("password form byte must be 0 or 6 to 12, got {b}")
                    );
                }
                other => panic!("pw_form {b:#x}: {other:?}"),
            }
        }
        for b in [0x00u8, 0x06, 0x0c] {
            let mut h = unhex(V2_A_HEADER);
            h[11] = b;
            h[12..16].copy_from_slice(&600_000u32.to_be_bytes());
            assert!(Header::decode(&h).is_ok(), "pw_form {b} is a form");
        }
    }

    /// S1b §14.10: a v2 public-only header (vector E's, version set to 2) with
    /// offset 11 = 6 is `UnsealedFieldNotZero("pw_form")`.
    #[test]
    fn v2_public_only_refuses_nonzero_pw_form() {
        let mut h = unsealed();
        h.version = VERSION_V2;
        assert!(
            Header::decode(&h.encode()).is_ok(),
            "v2 public-only, pw_form 0"
        );
        let mut b = h.encode();
        b[11] = 0x06;
        match Header::decode(&b) {
            Err(e @ WireError::UnsealedFieldNotZero("pw_form")) => {
                assert_eq!(
                    e.to_string(),
                    "pw_form must be zero when nothing is encrypted"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    /// S1b §14.10: a v1 header carrying 6 at offset 11 stays refused.
    #[test]
    fn v1_still_refuses_a_nonzero_reserved_byte() {
        let mut b = sealed().encode();
        b[11] = 0x06;
        match Header::decode(&b) {
            Err(e @ WireError::ReservedNotZero(6)) => {
                assert_eq!(e.to_string(), "reserved byte must be 0, got 6");
            }
            other => panic!("{other:?}"),
        }
        // And an unknown version, 0x00 and 0x03.. alike.
        for v in [0x00u8, 0x03, 0xff] {
            let mut b = sealed().encode();
            b[8] = v;
            assert!(matches!(Header::decode(&b), Err(WireError::UnknownVersion(x)) if x == v));
        }
    }
}
