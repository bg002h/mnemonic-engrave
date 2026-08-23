//! OPERATOR RULE (2026-08-22): "The QR RS density should be the highest that
//! minimizes plate count."
//!
//! Plate count is the real cost -- one plate per string today, ~21 min each --
//! so minimise plates FIRST, then spend every leftover byte on error
//! correction. Never trade a plate for redundancy; never leave redundancy
//! unbought.
//!
//! Search space per artifact: module size x QR version x ECC level x k*k tiling
//! on one 79 mm plate. Returns min plates, then max ECC among those.

use qrcode::{EcLevel, QrCode, Version};

const USABLE_MM: f64 = 79.0;
const QUIET: usize = 4;
/// 0.30 mm = one engraved stroke, the theoretical floor and OPTICALLY
/// UNVALIDATED (F-234). 0.60 mm = two strokes, the conservative floor.
const MODULES_MM: [f64; 4] = [0.30, 0.45, 0.60, 0.90];

fn cap(v: u8, ec: EcLevel, alnum: bool) -> usize {
    let fits = |n: usize| {
        let d: Vec<u8> = if alnum {
            (0..n).map(|i| b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"[i % 26]).collect()
        } else {
            (0..n).map(|i| b"abcdefghijklmnopqrstuvwxyz"[i % 26]).collect()
        };
        QrCode::with_version(&d, Version::Normal(v as i16), ec).is_ok()
    };
    if !fits(1) { return 0 }
    let (mut lo, mut hi) = (1usize, 5000usize);
    while lo < hi { let m = (lo + hi + 1) / 2; if fits(m) { lo = m } else { hi = m - 1 } }
    lo
}
fn modules(v: u8) -> usize { 4 * v as usize + 17 }
fn ec_rank(e: EcLevel) -> u8 { match e { EcLevel::L => 0, EcLevel::M => 1, EcLevel::Q => 2, EcLevel::H => 3 } }
fn ec_name(e: EcLevel) -> &'static str { match e { EcLevel::L => "L", EcLevel::M => "M", EcLevel::Q => "Q", EcLevel::H => "H" } }

/// Capacity for every (version, ECC, mode), computed ONCE. Recomputing it
/// inside the search loop made the 0.30 mm sweep take minutes.
struct Caps { byte: [[usize; 4]; 41], alnum: [[usize; 4]; 41] }
fn build_caps() -> Caps {
    let mut c = Caps { byte: [[0; 4]; 41], alnum: [[0; 4]; 41] };
    for v in 1..=40usize {
        for (i, ec) in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H].into_iter().enumerate() {
            c.byte[v][i] = cap(v as u8, ec, false);
            c.alnum[v][i] = cap(v as u8, ec, true);
        }
    }
    c
}

struct Pick { plates: usize, symbols: usize, ec: EcLevel, ver: u8, module_mm: f64, per_plate: usize }

/// `units` is bytes for byte mode, characters for alphanumeric mode.
fn best(units: usize, alnum: bool, min_module_mm: f64, caps: &Caps) -> Option<Pick> {
    let mut best: Option<Pick> = None;
    for &mm in MODULES_MM.iter().filter(|m| **m >= min_module_mm - 1e-9) {
        for v in 1..=40u8 {
            let fp = (modules(v) + 2 * QUIET) as f64 * mm;
            let k = (USABLE_MM / fp).floor() as usize;
            if k == 0 { continue }
            let per_plate = k * k;
            for ec in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H] {
                let c = if alnum { caps.alnum[v as usize][ec_rank(ec) as usize] }
                        else { caps.byte[v as usize][ec_rank(ec) as usize] };
                if c == 0 { continue }
                let symbols = units.div_ceil(c);
                let plates = symbols.div_ceil(per_plate);
                let better = match &best {
                    None => true,
                    // RULE: fewest plates wins; ties go to the strongest ECC;
                    // then to fewer symbols (fewer things to scan).
                    Some(b) => (plates, std::cmp::Reverse(ec_rank(ec)), symbols)
                             < (b.plates, std::cmp::Reverse(ec_rank(b.ec)), b.symbols),
                };
                if better { best = Some(Pick { plates, symbols, ec, ver: v, module_mm: mm, per_plate }); }
            }
        }
    }
    best
}

fn row(label: &str, raw: usize, min_mm: f64, caps: &Caps) {
    // UR: bytewords minimal is exactly 2 chars/byte + an 8-char CRC32
    // (bc/bytewords/bytewords.go:17-31), and uppercased `ur:bytes/N-M/...` is
    // fully QR-alphanumeric (`:` and `/` are both in the alnum set).
    let ur_chars = raw * 2 + 8 + 16; // + room for the ur:bytes/N-M/ prefix
    let a = best(raw, false, min_mm, caps);
    let b = best(ur_chars, true, min_mm, caps);
    let f = |p: &Option<Pick>| match p {
        Some(p) => format!("{} pl, {} qr, v{} ECC {} @{:.2}mm{}",
                           p.plates, p.symbols, p.ver, ec_name(p.ec), p.module_mm,
                           if p.per_plate > 1 { format!(" ({} up)", p.per_plate) } else { String::new() }),
        None => "-".into(),
    };
    println!("  {label:<26} {raw:>5} B | RAW  {:<38} | UR  {}", f(&a), f(&b));
}

fn main() {
    let caps = build_caps();
    for (min_mm, title) in [(0.60_f64, "CONSERVATIVE — 0.60 mm modules (2 strokes), optically plausible"),
                            (0.30_f64, "AGGRESSIVE — 0.30 mm allowed (1 stroke, UNVALIDATED, F-234)")] {
        println!("\n=== {title} ===");
        println!("  rule: fewest plates first, then the STRONGEST ECC that still fits that plate count\n");
        for (l, n) in [
            ("single-sig tr, 1in sweep", 162usize),
            ("RCW tr key-path, 1in", 162),
            ("RCW tr tier4, 1in", 405),
            ("3-of-5 wsh signed, 1in", 488),
            ("3-of-5 tr signed, 1in", 501),
            ("RCW tr tier1, 1in", 535),
            ("RCW wsh tier1, 1in", 742),
            ("9-of-11 tr signed, 1in", 1097),
            ("9-of-11 wsh signed, 1in", 1130),
            ("RCW tr tier1, 5in", 2455),
            ("9-of-11 tr PSBT, 2in/2out", 4962),
        ] { row(l, n, min_mm, &caps); }
    }
}
