//! Does the storable amount depend on the character set? Yes. QR has four
//! encoding modes with different bit costs:
//!
//!   numeric       0-9                              10 bits per 3 chars = 3.33
//!   alphanumeric  0-9 A-Z(upper) space $%*+-./:    11 bits per 2 chars = 5.50
//!   byte          any octet (Latin-1 / binary)                          8.00
//!   kanji         Shift-JIS                                            13.00
//!
//! So the question for `mt` is: to store N bytes of a signed transaction, which
//! textual representation costs least inside a QR? Capacities are MEASURED from
//! the encoder; expansion ratios are exact per each encoding's definition.

use qrcode::{EcLevel, QrCode, Version};

/// Measured max characters of a given class that fit an exact version+ECC.
fn cap(v: u8, ec: EcLevel, class: Class) -> usize {
    let fits = |n: usize| {
        let data: Vec<u8> = match class {
            // UPPERCASE LETTERS ONLY. Mixing in digits makes the optimizer split
            // the payload into numeric and alphanumeric SEGMENTS and pay a mode
            // + count header per switch, which measured 6.6% low. Letters are in
            // the alphanumeric set but not the numeric one, so this stays one
            // segment.
            Class::Alnum => (0..n).map(|i| b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"[i % 26]).collect(),
            // lowercase -> outside the alphanumeric set, forces BYTE, no ECI
            Class::Byte => (0..n).map(|i| b"abcdefghijklmnopqrstuvwxyz"[i % 26]).collect(),
            Class::Numeric => (0..n).map(|i| b"0123456789"[i % 10]).collect(),
        };
        QrCode::with_version(&data, Version::Normal(v as i16), ec).is_ok()
    };
    if !fits(1) { return 0 }
    let (mut lo, mut hi) = (1usize, 9000usize);
    while lo < hi { let m = (lo + hi + 1) / 2; if fits(m) { lo = m } else { hi = m - 1 } }
    lo
}

#[derive(Clone, Copy)]
enum Class { Numeric, Alnum, Byte }

struct Repr {
    name: &'static str,
    class: Class,
    /// encoded character count for n source bytes
    chars: fn(usize) -> usize,
    note: &'static str,
}

fn main() {
    let reprs = [
        Repr { name: "raw binary", class: Class::Byte, chars: |n| n,
               note: "1 octet per byte" },
        Repr { name: "base45 (RFC 9285)", class: Class::Alnum, chars: |n| n / 2 * 3 + (n % 2) * 2,
               note: "designed FOR qr alphanumeric" },
        Repr { name: "base32 / bech32 UPPER", class: Class::Alnum, chars: |n| (n * 8).div_ceil(5),
               note: "codex32 uppercased; 5 data bits per 5.5-bit char" },
        Repr { name: "base64", class: Class::Byte, chars: |n| n.div_ceil(3) * 4,
               note: "mixed case -> byte mode, no alnum discount" },
        Repr { name: "base58", class: Class::Byte, chars: |n| (n as f64 * 1.365658_f64).ceil() as usize,
               note: "mixed case -> byte mode" },
        Repr { name: "hex UPPER", class: Class::Alnum, chars: |n| n * 2,
               note: "2 chars/byte but alnum-discounted" },
        Repr { name: "decimal digits", class: Class::Numeric, chars: |n| (n as f64 * 2.408_f64).ceil() as usize,
               note: "densest MODE, worst expansion" },
    ];

    // Gate every mode against the published v40 limits before reporting ratios.
    // A search bound that silently caps a row looks exactly like a measurement.
    for (ec, n, a, b, name) in [(EcLevel::L, 7089usize, 4296usize, 2953usize, "L"),
                                (EcLevel::H, 3057, 1852, 1273, "H")] {
        let (gn, ga, gb) = (cap(40, ec, Class::Numeric), cap(40, ec, Class::Alnum), cap(40, ec, Class::Byte));
        println!("gate v40-{name}: numeric {gn}/{n} {} | alnum {ga}/{a} {} | byte {gb}/{b} {}",
                 if gn == n {"OK"} else {"MISMATCH"},
                 if ga == a {"OK"} else {"MISMATCH"},
                 if gb == b {"OK"} else {"MISMATCH"});
    }

    for ec in [EcLevel::L, EcLevel::H] {
        let ecn = match ec { EcLevel::L => "L ~7%", _ => "H ~30%" };
        let c_alnum = cap(40, ec, Class::Alnum);
        let c_byte = cap(40, ec, Class::Byte);
        let c_num = cap(40, ec, Class::Numeric);
        println!("\n=== ONE v40 SYMBOL, ECC {ecn} ===");
        println!("  raw capacity by mode: numeric {c_num} ch | alphanumeric {c_alnum} ch | byte {c_byte} ch");
        println!("  {:<24} {:>10} {:>12} {:>9}   {}", "representation", "SOURCE B", "vs binary", "chars", "note");
        let baseline = c_byte;
        for r in &reprs {
            let capacity = match r.class { Class::Alnum => c_alnum, Class::Byte => c_byte, Class::Numeric => c_num };
            // largest n whose encoded length fits
            let mut lo = 0usize;
            let mut hi = 6000usize;
            while lo < hi { let m = (lo + hi + 1) / 2; if (r.chars)(m) <= capacity { lo = m } else { hi = m - 1 } }
            println!("  {:<24} {lo:>8} B {:>11.0}% {:>9}   {}",
                     r.name, (lo as f64 / baseline as f64) * 100.0, (r.chars)(lo), r.note);
        }
    }
}
