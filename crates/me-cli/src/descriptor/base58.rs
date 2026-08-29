//! Base58check, as `hdkeychain.NewKeyFromString` reads it.
//!
//! Hand-rolled rather than pulled in, because the plan forbids a new
//! dependency and this is the whole of what an extended key needs: a base58
//! alphabet, a big-endian radix conversion, and a four-byte double-SHA256
//! trailer. `sha2` is already a dependency of this crate.

use sha2::Digest as _;

const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Decode base58 (no checksum). `None` on any character outside the alphabet.
fn decode(s: &str) -> Option<Vec<u8>> {
    // Reject non-ASCII up front: the index table below is byte-oriented, and a
    // multi-byte char would otherwise be split into bytes that are not in the
    // alphabet anyway — but saying so is cheaper than reasoning about it.
    if !s.is_ascii() {
        return None;
    }
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    for c in s.bytes() {
        let v = ALPHABET.iter().position(|a| *a == c)? as u32;
        // out = out * 58 + v, big-endian.
        let mut carry = v;
        for b in out.iter_mut().rev() {
            let x = u32::from(*b) * 58 + carry;
            *b = (x & 0xff) as u8;
            carry = x >> 8;
        }
        while carry > 0 {
            out.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    // Leading '1's are leading zero bytes.
    let zeros = s.bytes().take_while(|c| *c == b'1').count();
    let mut res = vec![0u8; zeros];
    res.extend_from_slice(&out);
    Some(res)
}

fn encode(bytes: &[u8]) -> String {
    let mut digits: Vec<u8> = Vec::with_capacity(bytes.len() * 138 / 100 + 1);
    for b in bytes {
        let mut carry = u32::from(*b);
        for d in digits.iter_mut() {
            let x = u32::from(*d) * 256 + carry;
            *d = (x % 58) as u8;
            carry = x / 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    let mut out = String::with_capacity(digits.len() + bytes.len());
    for _ in bytes.iter().take_while(|b| **b == 0) {
        out.push('1');
    }
    for d in digits.iter().rev() {
        out.push(ALPHABET[*d as usize] as char);
    }
    if out.is_empty() {
        out.push('1');
    }
    out
}

fn checksum4(payload: &[u8]) -> [u8; 4] {
    let first = sha2::Sha256::digest(payload);
    let second = sha2::Sha256::digest(first);
    let mut c = [0u8; 4];
    c.copy_from_slice(&second[..4]);
    c
}

/// Decode base58check and strip the four-byte trailer. `None` if the string is
/// not base58, is shorter than the trailer, or the trailer does not match —
/// the same three refusals `base58.Decode` + the `DoubleHashB` compare make.
pub fn decode_check(s: &str) -> Option<Vec<u8>> {
    let raw = decode(s)?;
    if raw.len() < 4 {
        return None;
    }
    let (payload, trailer) = raw.split_at(raw.len() - 4);
    if checksum4(payload) != trailer {
        return None;
    }
    Some(payload.to_vec())
}

/// Encode a payload with its four-byte double-SHA256 trailer.
pub fn encode_check(payload: &[u8]) -> String {
    let mut v = payload.to_vec();
    v.extend_from_slice(&checksum4(payload));
    encode(&v)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `dc567276` fixture cosigner key, from the shared vector file. A
    /// round trip through both halves is the only claim that matters here.
    const XPUB: &str = "xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan";

    #[test]
    fn an_extended_key_round_trips_and_is_78_bytes() {
        let p = decode_check(XPUB).expect("valid base58check");
        assert_eq!(p.len(), 78, "an extended key payload is 78 bytes");
        assert_eq!(encode_check(&p), XPUB);
    }

    #[test]
    fn a_flipped_character_fails_the_trailer() {
        let mut bad = XPUB.to_string();
        bad.pop();
        bad.push('m');
        assert!(decode_check(&bad).is_none(), "checksum must catch it");
    }

    #[test]
    fn characters_outside_the_alphabet_are_refused() {
        // `0`, `O`, `I` and `l` are the four base58 exclusions.
        for c in ['0', 'O', 'I', 'l'] {
            let mut bad = XPUB.to_string();
            bad.pop();
            bad.push(c);
            assert!(decode_check(&bad).is_none(), "{c} is not base58");
        }
    }

    #[test]
    fn leading_zero_bytes_survive_both_directions() {
        let payload = [0u8, 0, 1, 2, 3];
        let s = encode_check(&payload);
        assert!(
            s.starts_with("11"),
            "two leading zero bytes are two '1's: {s}"
        );
        assert_eq!(decode_check(&s).unwrap(), payload);
    }

    #[test]
    fn the_empty_payload_encodes_and_decodes() {
        let s = encode_check(&[]);
        assert_eq!(decode_check(&s).unwrap(), Vec::<u8>::new());
    }
}
