//! What is the LARGEST transaction that fits ONE plate?
//!
//! Uses the 2026-08-24 objective (select2.rs): ECC floor M, symbols <= 16,
//! minimise plates -> symbols -> maximise ECC, and the PACKED legend.
//!
//! The answer is bounded by plate 1's usable height, because plate 1 alone
//! carries the legend. That is what makes the one-plate ceiling lower than a
//! bare-plate capacity figure would suggest.
use qrcode::{EcLevel, QrCode, Version};

const USABLE_MM: f64 = 79.0;
const QUIET: usize = 4;
const LINE_PITCH_MM: f64 = 85.0 / 20.0;
const LEGEND_COLS: f64 = 44.0;
const LEGEND_CHARS_FIRST: f64 = 153.0 + 14.0;
fn legend_lines(c: f64) -> f64 { (c / LEGEND_COLS).ceil() }
fn modules(v: u8) -> usize { 4 * v as usize + 17 }

fn cap(v: u8, ec: EcLevel) -> usize {
    let fits = |n: usize| {
        let d: Vec<u8> = (0..n).map(|i| b"abcdefghijklmnopqrstuvwxyz"[i % 26]).collect();
        QrCode::with_version(&d, Version::Normal(v as i16), ec).is_ok()
    };
    if !fits(1) { return 0 }
    let (mut lo, mut hi) = (1usize, 5000usize);
    while lo < hi { let m = (lo + hi + 1) / 2; if fits(m) { lo = m } else { hi = m - 1 } }
    lo
}

fn main() {
    let legend_mm = legend_lines(LEGEND_CHARS_FIRST) * LINE_PITCH_MM;
    println!("packed legend on plate 1: {:.0} lines = {:.1} mm, leaving {:.1} mm for the QR\n",
             legend_lines(LEGEND_CHARS_FIRST), legend_mm, USABLE_MM - legend_mm);
    println!("{:<10} {:<9} {:>8} {:>8}   {}", "module", "largest v", "symbols", "MAX B", "at ECC (floor M)");
    for mm in [0.90_f64, 0.60, 0.45, 0.30] {
        let h = USABLE_MM - legend_mm;
        let mut best: Option<(u8, EcLevel, usize, usize)> = None;
        for v in 1..=40u8 {
            let fp = (modules(v) + 2 * QUIET) as f64 * mm;
            let across = (USABLE_MM / fp).floor() as usize;
            let rows = (h / fp).floor() as usize;
            let tiles = across * rows;
            if tiles == 0 { continue }
            let tiles = tiles.min(16);           // Structured Append cap
            for ec in [EcLevel::M, EcLevel::Q, EcLevel::H] {   // floor is M
                let c = cap(v, ec);
                if c == 0 { continue }
                let total = c * tiles;
                // objective: max bytes, then FEWEST symbols, then strongest ECC
                let key = (total, std::cmp::Reverse(tiles), ec as u8);
                if best.map_or(true, |(bv, bec, bt, bb)| key > (bb, std::cmp::Reverse(bt), bec as u8) || (bv, bec) == (bv, bec) && false) {
                    if best.map_or(true, |(_, _, _, bb)| total > bb) { best = Some((v, ec, tiles, total)); }
                }
            }
        }
        match best {
            Some((v, ec, t, b)) => println!("{:<10.2} v{:<8} {:>8} {:>8}   {:?}", mm, v, t, b, ec),
            None => println!("{:<10.2} {:<9} {:>8} {:>8}", mm, "-", "-", "-"),
        }
    }
}
