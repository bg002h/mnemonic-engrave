//! How much data fits on ONE engraved plate — as codex32 text, or as a QR code?
//!
//! Physical constants read out of the SeedHammer fork, not assumed:
//!   plate            85 x 85 mm      third_party/seedhammer/backup/backup.go:99-102
//!   strokeWidth      0.3 mm          cmd/controller/platform_sh2.go:188
//!   qrScale          3               backup/backup.go:108  -> 0.9 mm per module
//!   QR ECC level     M (15%)         backup/backup.go:77   qr.Encode(seed, qr.M)
//!   PLATE_TEXT_BUDGET 300 chars      me-cli/src/lib.rs:48

use qrcode::{EcLevel, QrCode, Version};

const PLATE_MM: f64 = 85.0;
const MARGIN_MM: f64 = 4.0; // per side
const STROKE_MM: f64 = 0.3;
const PLATE_TEXT_BUDGET: usize = 300;

/// Largest byte payload encodable at `ec` whose module count fits `usable_mm`
/// at `module_mm` per module. Found by probing the encoder, not from a table.
fn payload(n: usize, alnum: bool) -> Vec<u8> {
    // Pure BYTE mode must be FORCED. The encoder does optimal mode
    // SEGMENTATION, so any run of digits or uppercase inside the payload gets
    // re-encoded in a denser mode and inflates the apparent binary capacity.
    // Bytes >= 0x80 are outside both the numeric and alphanumeric sets.
    if alnum {
        (0..n).map(|i| b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"[i % 36]).collect()
    } else {
        // Lowercase ASCII: outside QR's alphanumeric set (which is UPPERCASE
        // only), so byte mode is forced, and pure Latin-1 so no ECI header is
        // emitted. High bytes (>=0x80) also force byte mode but cost an ECI
        // segment on this encoder, which is why they measured ~0.5% low.
        (0..n).map(|i| b"abcdefghijklmnopqrstuvwxyz"[i % 26]).collect()
    }
}

/// Largest payload at `ec` whose module count fits `usable_mm`. Binary search:
/// capacity is monotone in n, so ~13 encodes instead of thousands.
fn max_bytes(module_mm: f64, usable_mm: f64, ec: EcLevel, alnum: bool) -> Option<(usize, usize, u8)> {
    let max_modules = (usable_mm / module_mm).floor() as usize;
    let fits = |n: usize| -> Option<(usize, u8)> {
        QrCode::with_error_correction_level(&payload(n, alnum), ec).ok().and_then(|c| {
            let w = c.width();
            if w > max_modules { return None }
            let v = match c.version() { Version::Normal(v) => v as u8, Version::Micro(v) => v as u8 };
            Some((w, v))
        })
    };
    fits(1)?;
    let (mut lo, mut hi) = (1usize, 5000usize);
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        if fits(mid).is_some() { lo = mid } else { hi = mid - 1 }
    }
    let (w, v) = fits(lo)?;
    Some((lo, w, v))
}

fn ecname(e: EcLevel) -> &'static str {
    match e { EcLevel::L => "L  ~7%", EcLevel::M => "M ~15%", EcLevel::Q => "Q ~25%", EcLevel::H => "H ~30%" }
}

fn main() {
    let usable = PLATE_MM - 2.0 * MARGIN_MM;
    println!("ONE PLATE: {PLATE_MM} x {PLATE_MM} mm, {MARGIN_MM} mm margin per side => {usable} mm usable");
    println!("stroke {STROKE_MM} mm\n");

    println!("--- AS ENGRAVED TEXT (codex32) ---");
    // A chunk string measured from real md output: 85 chars unbroken.
    // Theoretical filled chunk: hrp(3) + 80 data + 13 checksum = 96 chars.
    for (label, chars, payload_bits) in [
        ("md today (balanced, 85 ch)", 85usize, 69 * 5 - 37),
        ("regular code, filled",       96,      80 * 5 - 37),
        ("LONG code, filled",         127,     112 * 5 - 37),
    ] {
        let per_plate = PLATE_TEXT_BUDGET / chars;
        println!("  {label:<28} {chars:>3} ch/string, {:>3} B/string | {per_plate} string(s)/plate at the 300-char budget => {:>4} B/plate  (today: 1 string/plate => {:>3} B/plate)",
                 payload_bits / 8, per_plate * (payload_bits / 8), payload_bits / 8);
    }

    println!("\n--- AS AN ENGRAVED QR ---");
    println!("  {:<16} {:>7} {:>8} {:>12} {:>10} {:>14}",
             "scale / ECC", "mod mm", "max mods", "QR version", "BYTE mode", "ALNUM mode");
    for scale in [3u32, 2, 1] {
        let module_mm = STROKE_MM * scale as f64;
        for ec in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H] {
            let b = max_bytes(module_mm, usable, ec, false);
            let a = max_bytes(module_mm, usable, ec, true);
            let vtxt = b.map(|(_, w, v)| format!("v{v} ({w}mod)")).unwrap_or("-".into());
            println!("  scale {scale} {:<10} {module_mm:>6.2} {:>8} {:>12} {:>10} {:>14}",
                     ecname(ec), (usable / module_mm).floor() as usize, vtxt,
                     b.map(|(n, _, _)| format!("{n} B")).unwrap_or("none".into()),
                     a.map(|(n, _, _)| format!("{n} ch")).unwrap_or("none".into()));
        }
    }
    println!("\n  (the fork engraves at scale 3, ECC M — backup.go:108 and :77)");

    // Sanity gate: an UNBOUNDED v40 must match the published QR limits, or the
    // mode being measured is not the mode being claimed.
    for (ec, want, name) in [(EcLevel::L, 2953usize, "L"), (EcLevel::M, 2331, "M"),
                             (EcLevel::Q, 1663, "Q"), (EcLevel::H, 1273, "H")] {
        let got = max_bytes(0.001, 1000.0, ec, false).map(|(n, _, _)| n).unwrap_or(0);
        println!("  gate: v40-{name} byte capacity measured {got}, published {want} -> {}",
                 if got == want { "MATCH" } else { "MISMATCH — byte mode not forced" });
    }

    println!("\n--- PLATES NEEDED FOR A REAL SIGNED TRANSACTION (1-input sweep) ---");
    println!("  {:<28} {:>7} {:>22} {:>24}", "wallet / path", "tx", "QR plates @s3 ECC M", "codex32 plates today");
    for (name, tx) in [("RCW tr KEY-PATH", 162usize), ("RCW tr tier4", 405),
                       ("RCW tr tier1", 535), ("RCW wsh tier1", 742),
                       ("pathological wsh tier1", 852)] {
        let qcap = max_bytes(STROKE_MM * 3.0, usable, EcLevel::M, false).unwrap().0;
        let qplates = tx.div_ceil(qcap);
        let text_plates = (tx * 8).div_ceil(69 * 5 - 37); // md today: 1 balanced chunk per plate
        println!("  {name:<28} {tx:>5} B {qplates:>19} {text_plates:>22}");
    }
}
