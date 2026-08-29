//! **Is this 33-byte string a point on secp256k1?** — and nothing else.
//!
//! ## Why this exists at all
//!
//! `bip380.ParseKey` calls `hdkeychain.NewKeyFromString`, which does not stop
//! at the base58check trailer: for a public key it runs `btcec.ParsePubKey`,
//! which *decompresses* the point and fails if `x³ + 7` has no square root
//! (`btcutil/v2@v2.0.0/hdkeychain/extendedkey.go:724–731`, read at
//! implementation time). A host that checked only the trailer and the `0x02`/
//! `0x03` prefix would therefore ADMIT an extended key the device REFUSES —
//! `SPEC_descriptor_input.md` §7's forbidden direction, an engraved plate for a
//! wallet that will not load.
//!
//! The plan forbids a new dependency, and `me` has no `bitcoin`/`secp256k1`
//! crate. So the one predicate that gap needs is implemented here, and only
//! that one: **no** point arithmetic, no signatures, no secrets. Nothing in
//! this file touches key material that is not already a public extended key
//! the operator handed us on a public channel.
//!
//! ## The predicate
//!
//! A compressed point is `0x02`/`0x03 || x`. It is on the curve iff `x < p`
//! and `x³ + 7` is a quadratic residue mod `p` — Euler's criterion,
//! `v^((p−1)/2) ≡ 1`. The parity byte then selects which root, which is
//! irrelevant to validity.
//!
//! `p = 2^256 − 2^32 − 977`, so reduction of a 512-bit product needs no
//! division: `2^256 ≡ 2^32 + 977 (mod p)`, and folding the high half through
//! that constant twice always lands within one or two subtractions of `p`.

/// Little-endian 64-bit limbs.
type U256 = [u64; 4];

/// `p = 2^256 − 2^32 − 977`.
const P: U256 = [0xFFFF_FFFE_FFFF_FC2F, u64::MAX, u64::MAX, u64::MAX];
/// `2^256 mod p = 2^32 + 977`.
const C: u64 = 0x0001_0000_03D1;
const ONE: U256 = [1, 0, 0, 0];
const SEVEN: U256 = [7, 0, 0, 0];

fn cmp(a: &U256, b: &U256) -> std::cmp::Ordering {
    for i in (0..4).rev() {
        match a[i].cmp(&b[i]) {
            std::cmp::Ordering::Equal => {}
            o => return o,
        }
    }
    std::cmp::Ordering::Equal
}

fn is_zero(a: &U256) -> bool {
    a.iter().all(|l| *l == 0)
}

/// `a − b`, wrapping. Only called where `a >= b`.
fn sub(a: &U256, b: &U256) -> U256 {
    let mut out = [0u64; 4];
    let mut borrow = 0u64;
    for i in 0..4 {
        let (d, b1) = a[i].overflowing_sub(b[i]);
        let (d, b2) = d.overflowing_sub(borrow);
        out[i] = d;
        borrow = u64::from(b1) + u64::from(b2);
    }
    out
}

fn add_mod(a: &U256, b: &U256) -> U256 {
    let mut out = [0u64; 4];
    let mut carry = 0u128;
    for i in 0..4 {
        let s = u128::from(a[i]) + u128::from(b[i]) + carry;
        out[i] = s as u64;
        carry = s >> 64;
    }
    // A carry out of the top means the true value is `out + 2^256`, and
    // `2^256 ≡ C`, so folding it in is one small add.
    if carry != 0 {
        let mut c = C as u128;
        for limb in out.iter_mut() {
            let s = u128::from(*limb) + c;
            *limb = s as u64;
            c = s >> 64;
            if c == 0 {
                break;
            }
        }
    }
    if cmp(&out, &P) != std::cmp::Ordering::Less {
        out = sub(&out, &P);
    }
    out
}

/// Schoolbook 256×256 → 512.
fn mul_wide(a: &U256, b: &U256) -> [u64; 8] {
    let mut z = [0u64; 8];
    for (i, ai) in a.iter().enumerate() {
        let mut carry = 0u128;
        for (j, bj) in b.iter().enumerate() {
            let t = u128::from(*ai) * u128::from(*bj) + u128::from(z[i + j]) + carry;
            z[i + j] = t as u64;
            carry = t >> 64;
        }
        let mut k = i + 4;
        while carry != 0 {
            let t = u128::from(z[k]) + carry;
            z[k] = t as u64;
            carry = t >> 64;
            k += 1;
        }
    }
    z
}

/// Reduce a 512-bit value mod `p` by folding the high half through
/// `2^256 ≡ 2^32 + 977`.
fn reduce(mut z: [u64; 8]) -> U256 {
    // Two folds are enough: the first drops 512 bits to ~289, the second to
    // ~257. The loop is written as a loop anyway so the bound is a fact of the
    // code rather than a comment.
    while z[4..].iter().any(|l| *l != 0) {
        let hi = [z[4], z[5], z[6], z[7]];
        let mut acc = [z[0], z[1], z[2], z[3], 0u64, 0, 0, 0];
        // acc += hi * C
        let mut carry = 0u128;
        for (i, h) in hi.iter().enumerate() {
            let t = u128::from(*h) * u128::from(C) + u128::from(acc[i]) + carry;
            acc[i] = t as u64;
            carry = t >> 64;
        }
        let mut k = 4;
        while carry != 0 {
            let t = u128::from(acc[k]) + carry;
            acc[k] = t as u64;
            carry = t >> 64;
            k += 1;
        }
        z = acc;
    }
    let mut out = [z[0], z[1], z[2], z[3]];
    while cmp(&out, &P) != std::cmp::Ordering::Less {
        out = sub(&out, &P);
    }
    out
}

fn mul_mod(a: &U256, b: &U256) -> U256 {
    reduce(mul_wide(a, b))
}

/// `(p − 1) / 2`, derived rather than transcribed.
fn p_minus_1_over_2() -> U256 {
    let m = sub(&P, &ONE);
    let mut out = [0u64; 4];
    for i in 0..4 {
        let hi = if i == 3 { 0 } else { m[i + 1] << 63 };
        out[i] = (m[i] >> 1) | hi;
    }
    out
}

fn pow_mod(base: &U256, exp: &U256) -> U256 {
    let mut result = ONE;
    let mut b = *base;
    for limb in exp.iter() {
        for bit in 0..64 {
            if (limb >> bit) & 1 == 1 {
                result = mul_mod(&result, &b);
            }
            b = mul_mod(&b, &b);
        }
    }
    result
}

fn from_be(bytes: &[u8]) -> U256 {
    let mut out = [0u64; 4];
    for (i, chunk) in bytes.chunks(8).enumerate() {
        let mut w = [0u8; 8];
        w.copy_from_slice(chunk);
        out[3 - i] = u64::from_be_bytes(w);
    }
    out
}

/// Whether `key` is a valid COMPRESSED secp256k1 public key — the predicate
/// `btcec.ParsePubKey` applies inside `hdkeychain.NewKeyFromString`.
///
/// An uncompressed or hybrid encoding is refused: an extended key's key-data
/// field is 33 bytes, so nothing else fits, and a `0x00` prefix is the PRIVATE
/// form, which `me` never accepts on a public channel.
pub fn is_valid_compressed_pubkey(key: &[u8]) -> bool {
    if key.len() != 33 || (key[0] != 0x02 && key[0] != 0x03) {
        return false;
    }
    let x = from_be(&key[1..33]);
    if cmp(&x, &P) != std::cmp::Ordering::Less {
        return false;
    }
    // v = x³ + 7
    let v = add_mod(&mul_mod(&mul_mod(&x, &x), &x), &SEVEN);
    if is_zero(&v) {
        return false;
    }
    pow_mod(&v, &p_minus_1_over_2()) == ONE
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The generator's x-coordinate — the one point every implementation agrees
    /// about.
    const G_X: [u8; 32] = [
        0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87, 0x0B,
        0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B, 0x16, 0xF8,
        0x17, 0x98,
    ];

    fn key(prefix: u8, x: [u8; 32]) -> Vec<u8> {
        let mut v = vec![prefix];
        v.extend_from_slice(&x);
        v
    }

    #[test]
    fn the_generator_is_on_the_curve_in_both_parities() {
        assert!(is_valid_compressed_pubkey(&key(0x02, G_X)));
        assert!(is_valid_compressed_pubkey(&key(0x03, G_X)));
    }

    fn small_x(v: u8) -> [u8; 32] {
        let mut x = [0u8; 32];
        x[31] = v;
        x
    }

    /// **The off-curve half, and the values are MEASURED rather than assumed.**
    ///
    /// The first draft of this test asserted `x = 1` was off the curve, on the
    /// reasoning that `1³ + 7 = 8` is a non-residue. It is not: `p ≡ 7 (mod 8)`,
    /// so 2 is a residue and 8 with it, and `x = 1` is a real point. The test
    /// failed and the code was right — which is the whole reason to write the
    /// negative case with values taken from an independent computation instead
    /// of from an argument.
    #[test]
    fn the_off_curve_x_values_are_refused_and_the_on_curve_ones_are_not() {
        // `x³ + 7` is a non-residue at these, computed independently.
        for v in [0u8, 5, 7, 9, 10, 11] {
            assert!(
                !is_valid_compressed_pubkey(&key(0x02, small_x(v))),
                "x = {v} is NOT on the curve"
            );
        }
        for v in [1u8, 2, 3, 4, 6, 8] {
            assert!(
                is_valid_compressed_pubkey(&key(0x02, small_x(v))),
                "x = {v} IS on the curve"
            );
        }
    }

    #[test]
    fn x_at_or_above_the_field_prime_is_refused() {
        let all_ff = [0xffu8; 32];
        assert!(!is_valid_compressed_pubkey(&key(0x02, all_ff)));
    }

    #[test]
    fn only_the_two_compressed_prefixes_are_accepted() {
        for p in [0x00u8, 0x01, 0x04, 0x05, 0x06, 0x07, 0xff] {
            assert!(!is_valid_compressed_pubkey(&key(p, G_X)), "prefix {p:#04x}");
        }
        assert!(!is_valid_compressed_pubkey(&[]), "empty");
        assert!(!is_valid_compressed_pubkey(&[0x02; 32]), "short");
    }

    /// The arithmetic itself, against values a reader can check by hand.
    #[test]
    fn the_field_arithmetic_agrees_with_small_cases() {
        // p − 1 + 1 == 0
        assert!(is_zero(&add_mod(&sub(&P, &ONE), &ONE)));
        // 2 * (p−1)/2 == p − 1
        let half = p_minus_1_over_2();
        assert_eq!(add_mod(&half, &half), sub(&P, &ONE));
        // Fermat: 2^(p−1) == 1, i.e. (2^((p−1)/2))² == 1
        let t = pow_mod(&[2, 0, 0, 0], &half);
        assert_eq!(mul_mod(&t, &t), ONE);
        // A 512-bit product reduces: (p−1)² ≡ 1
        let pm1 = sub(&P, &ONE);
        assert_eq!(mul_mod(&pm1, &pm1), ONE);
    }
}
