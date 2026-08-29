//! The BIP-380 descriptor checksum, ported from the fork's `bip380/checksum.go`
//! (read at implementation time, `d402f18`).
//!
//! `me` needs both halves: `verify` because §4.3 refuses a descriptor whose
//! `#checksum` does not validate, and [`compute`] because §5.2's canonical
//! record is `Descriptor::encode()` WITH its checksum, and §4.5's promotion
//! announcement prints that canonical string.

const ALPHABET: &[u8] =
    b"0123456789()[],'/*abcdefgh@:$%{}IJKLMNOPQRSTUVWXYZ&+-.;<=>?!^_|~ijklmnopqrstuvwxyzABCDEFGH`#\"\\ ";
const CHECKSUM_ALPHABET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
const GENERATOR: [u64; 5] = [
    0x00f5_dee5_1989,
    0x00a9_fdca_3312,
    0x001b_ab10_e32d,
    0x0037_06b1_677a,
    0x0064_4d62_6ffd,
];

/// Expand a descriptor body to checksum symbols. `None` if any character is
/// outside the input alphabet.
fn expand(s: &str) -> Option<Vec<u8>> {
    let mut groups: Vec<u8> = Vec::with_capacity(3);
    let mut syms: Vec<u8> = Vec::with_capacity(s.len() * 4 / 3 + 1);
    for c in s.bytes() {
        let idx = ALPHABET.iter().position(|a| *a == c)? as u8;
        syms.push(idx & 31);
        groups.push(idx >> 5);
        if groups.len() == 3 {
            syms.push(groups[0] * 9 + groups[1] * 3 + groups[2]);
            groups.clear();
        }
    }
    match groups.len() {
        1 => syms.push(groups[0]),
        2 => syms.push(groups[0] * 3 + groups[1]),
        _ => {}
    }
    Some(syms)
}

fn polymod(syms: &[u8]) -> u64 {
    let mut chk: u64 = 1;
    for v in syms {
        let top = chk >> 35;
        chk = ((chk & 0x0007_ffff_ffff) << 5) ^ u64::from(*v);
        for (i, g) in GENERATOR.iter().enumerate() {
            if (top >> i) & 1 != 0 {
                chk ^= g;
            }
        }
    }
    chk
}

/// Whether `c` is the valid checksum for descriptor body `s`.
pub fn verify(s: &str, c: &str) -> bool {
    if c.len() != 8 {
        return false;
    }
    let Some(mut syms) = expand(s) else {
        return false;
    };
    for ch in c.bytes() {
        match CHECKSUM_ALPHABET.iter().position(|a| *a == ch) {
            Some(i) => syms.push(i as u8),
            None => return false,
        }
    }
    polymod(&syms) == 1
}

/// The checksum of `s`. `None` if `s` contains characters outside the alphabet.
pub fn compute(s: &str) -> Option<String> {
    let mut syms = expand(s)?;
    syms.extend_from_slice(&[0; 8]);
    let sum = polymod(&syms) ^ 1;
    let mut out = String::with_capacity(8);
    for i in 0..8u32 {
        out.push(CHECKSUM_ALPHABET[((sum >> (5 * (7 - i))) & 31) as usize] as char);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fork's own `nonstandard/parse_test.go` JSON fixture carries this
    /// descriptor and this checksum — so the port is pinned to a value the
    /// device produced, not to one this file computed.
    const BODY: &str = "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/0/*,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/0/*,[c5d87297/48h/0h/0h/2h]xpub6DjrnfAyuonMaboEb3ZQZzhQ2ZEgaKV2r64BFmqymZqJqviLTe1JzMr2X2RfQF892RH7MyYUbcy77R7pPu1P71xoj8cDUMNhAMGYzKR4noZ/0/*))";

    #[test]
    fn compute_matches_the_forks_own_fixture() {
        assert_eq!(compute(BODY).as_deref(), Some("hfwurrvt"));
        assert!(verify(BODY, "hfwurrvt"));
    }

    #[test]
    fn a_wrong_checksum_is_rejected() {
        assert!(!verify(BODY, "00000000"));
        assert!(!verify(BODY, "hfwurrvu"));
    }

    #[test]
    fn a_checksum_of_the_wrong_length_is_rejected() {
        // §4.3's doubled-checksum row: `Parse` cuts at the FIRST `#`, so the
        // remainder is 17 characters and can never validate.
        assert!(!verify(BODY, "hfwurrvt#hfwurrvt"));
        assert!(!verify(BODY, "hfwurrv"));
    }

    #[test]
    fn a_character_outside_the_alphabet_has_no_checksum() {
        assert!(compute("wpkh(\u{e9})").is_none());
        assert!(!verify("wpkh(\u{e9})", "qqqqqqqq"));
    }

    #[test]
    fn every_computed_checksum_verifies() {
        for s in ["wpkh(xpub)", "", "sh(wsh(sortedmulti(2,a,b)))", "tr(K)"] {
            let c = compute(s).unwrap();
            assert!(verify(s, &c), "{s} -> {c}");
        }
    }
}
