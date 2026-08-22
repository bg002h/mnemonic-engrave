//! What is the LARGEST QR the plate can physically engrave, and how big a raw
//! (non-codex32) transaction fits at the strongest and weakest Reed-Solomon
//! levels?
//!
//! Physical constants read from the fork, not assumed:
//!   plate        85 x 85 mm   backup/backup.go:99-102
//!   outerMargin  3 mm         backup/backup.go:45
//!   strokeWidth  0.3 mm       cmd/controller/platform_sh2.go:188
//! The engraved line is 0.3 mm wide, so 0.3 mm is the smallest feature that can
//! exist — a module cannot be smaller than one stroke.

use qrcode::{EcLevel, QrCode, Version};

const PLATE_MM: f64 = 85.0;
const OUTER_MARGIN_MM: f64 = 3.0;
const STROKE_MM: f64 = 0.3;
/// QR spec: 4 modules of quiet zone on every side of every symbol.
const QUIET_MODULES: usize = 4;

fn ec_name(e: EcLevel) -> &'static str {
    match e { EcLevel::L => "L ~7%", EcLevel::M => "M ~15%", EcLevel::Q => "Q ~25%", EcLevel::H => "H ~30%" }
}

/// Byte-mode capacity of an exact version at an exact EC level, measured by
/// binary search against the encoder. Lowercase ASCII forces pure byte mode
/// with no ECI segment (validated against published v40 limits elsewhere).
fn capacity(v: u8, ec: EcLevel) -> usize {
    let fits = |n: usize| {
        let data: Vec<u8> = (0..n).map(|i| b"abcdefghijklmnopqrstuvwxyz"[i % 26]).collect();
        QrCode::with_version(&data, Version::Normal(v as i16), ec).is_ok()
    };
    if !fits(1) { return 0 }
    let (mut lo, mut hi) = (1usize, 3000usize);
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        if fits(mid) { lo = mid } else { hi = mid - 1 }
    }
    lo
}

fn modules(v: u8) -> usize { 4 * v as usize + 17 }

fn main() {
    let usable = PLATE_MM - 2.0 * OUTER_MARGIN_MM;
    println!("PLATE {PLATE_MM} x {PLATE_MM} mm, outerMargin {OUTER_MARGIN_MM} mm/side => {usable} mm usable");
    println!("stroke {STROKE_MM} mm = the smallest feature that can be engraved => module floor\n");

    println!("=== 1. WHAT BINDS: the QR standard, or the plate? ===");
    let max_modules_on_plate = (usable / STROKE_MM).floor() as usize;
    let v40 = modules(40);
    println!("  plate can hold           {max_modules_on_plate} modules across at 1 stroke/module");
    println!("  largest QR that exists   {v40} modules (version 40)");
    println!("  v40 at 1 stroke/module   {:.1} mm square", v40 as f64 * STROKE_MM);
    println!("  => the {} binds; linear headroom {:.0}%",
             if v40 <= max_modules_on_plate { "QR STANDARD" } else { "PLATE" },
             (max_modules_on_plate as f64 / v40 as f64 - 1.0) * 100.0);
    // Largest module size at which a v40 still fits — spend the headroom on robustness.
    // Include the 4-module quiet zone on each side, as sections 3 and 4 do.
    let biggest_module = usable / (v40 + 2 * QUIET_MODULES) as f64;
    println!("  a v40 still fits at      {biggest_module:.3} mm/module ({:.2} strokes), quiet zone included",
             biggest_module / STROKE_MM);

    println!("\n=== 2. ONE v40 SYMBOL — raw transaction bytes, no codex32 ===");
    for ec in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H] {
        println!("  v40 {:<7} {:>5} B", ec_name(ec), capacity(40, ec));
    }

    println!("\n=== 3. TILING: many symbols per plate (quiet zone included) ===");
    println!("  {:<10} {:>8} {:>10} {:>9} {:>12} {:>12}", "module mm", "best ver", "footprint", "grid", "L total", "H total");
    for &module_mm in &[STROKE_MM, 0.45, 0.6, 0.9] {
        let mut best: Option<(u8, usize, usize, usize)> = None; // ver, grid, Ltot, Htot
        for v in 1..=40u8 {
            let footprint_mm = (modules(v) + 2 * QUIET_MODULES) as f64 * module_mm;
            let per_axis = (usable / footprint_mm).floor() as usize;
            if per_axis == 0 { continue }
            let n = per_axis * per_axis;
            let ltot = n * capacity(v, EcLevel::L);
            if best.map_or(true, |(_, _, bl, _)| ltot > bl) {
                best = Some((v, per_axis, ltot, n * capacity(v, EcLevel::H)));
            }
        }
        if let Some((v, per_axis, ltot, htot)) = best {
            let fp = (modules(v) + 2 * QUIET_MODULES) as f64 * module_mm;
            println!("  {module_mm:<10.2} {:>8} {fp:>8.1}mm {:>9} {ltot:>10} B {htot:>10} B",
                     format!("v{v}"), format!("{per_axis}x{per_axis}"));
        }
    }
    println!("\n  NOTE: tiling beyond 16 symbols exceeds QR Structured Append's limit and would");
    println!("  need a scheme of mt's own. Counts above are unconstrained.");

    println!("\n=== 4. HOW BIG A TRANSACTION FITS — one plate, raw bytes ===");
    for &module_mm in &[STROKE_MM, 0.6, 0.9] {
        // best single symbol at this module size
        let mut bv = 0u8;
        for v in 1..=40u8 {
            if (modules(v) + 2 * QUIET_MODULES) as f64 * module_mm <= usable { bv = v }
        }
        if bv == 0 { continue }
        println!("  {module_mm:.2} mm/module -> single v{bv}: L {:>5} B   H {:>5} B",
                 capacity(bv, EcLevel::L), capacity(bv, EcLevel::H));
    }

    println!("\n=== 5. INPUTS PER PLATE, raw signed transaction, no codex32 ===");
    println!("  (1-input size and marginal cost per input measured by envelope.rs)");
    // best standard-compliant layout at the 0.3 mm stroke floor: 2x2 of v26,
    // which is 4 symbols and so within Structured Append's 16-symbol limit.
    let tiled_l = 4 * capacity(26, EcLevel::L);
    let tiled_h = 4 * capacity(26, EcLevel::H);
    let solo_l = capacity(40, EcLevel::L);
    let solo_h = capacity(40, EcLevel::H);
    println!("  budgets: one v40 = {solo_l} B (L) / {solo_h} B (H);  2x2 v26 = {tiled_l} B (L) / {tiled_h} B (H)\n");
    println!("  {:<26} {:>10} {:>10} {:>10} {:>10}", "wallet / spend path", "v40 L", "v40 H", "2x2 L", "2x2 H");
    for (name, one, marg) in [
        ("RCW tr key-path", 162usize, 107usize),
        ("RCW tr tier4", 405, 350),
        ("RCW tr tier1", 535, 480),
        ("RCW wsh tier1", 742, 691),
        ("pathological wsh tier1", 852, 796),
        ("3-of-5 wsh", 491, 433),
        ("2-of-3 wsh", 349, 293),
    ] {
        let n = |budget: usize| -> String {
            if budget < one { return "0".into() }
            format!("{}", 1 + (budget - one) / marg)
        };
        println!("  {name:<26} {:>10} {:>10} {:>10} {:>10}",
                 n(solo_l), n(solo_h), n(tiled_l), n(tiled_h));
    }
}
