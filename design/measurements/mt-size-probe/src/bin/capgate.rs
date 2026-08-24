//! THE MODE-SEGMENTATION GATE, applied to select2.rs's OWN cap().
//!
//! F-234 records why this exists: a QR encoder does optimal mode segmentation
//! and will silently re-encode part of a payload in a denser mode. An all-0x41
//! payload measured *alphanumeric* capacity while claiming byte; a high-byte
//! payload paid an ECI header; a mixed payload read 6.6% low. Every one produced
//! a plausible number. Only asserting measured v40 capacity against the
//! published limits caught them.
//!
//! qrmodes.rs runs this for ITS probe and PRINTS ok. select2.rs -- which
//! produced RESULTS_ecc_selection_2026-08-24.txt, the table the spec cites --
//! has its own cap() and never ran it. This binary closes that, and EXITS
//! NON-ZERO on mismatch, so it is a gate rather than a report.
use qrcode::{EcLevel, QrCode, Version};

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

fn main() -> Result<(), String> {
    // Published ISO/IEC 18004 v40 capacities.
    let expect = [
        (EcLevel::L, "L", 4296usize, 2953usize),
        (EcLevel::M, "M", 3391, 2331),
        (EcLevel::Q, "Q", 2420, 1663),
        (EcLevel::H, "H", 1852, 1273),
    ];
    let mut bad = 0;
    println!("mode-segmentation gate -- select2.rs's cap() vs published v40 limits");
    for (ec, name, alnum_want, byte_want) in expect {
        let a = cap(40, ec, true);
        let b = cap(40, ec, false);
        let ok_a = a == alnum_want;
        let ok_b = b == byte_want;
        if !ok_a || !ok_b { bad += 1 }
        println!("  v40-{name}: alnum {a}/{alnum_want} {}  | byte {b}/{byte_want} {}",
                 if ok_a { "OK" } else { "*** MISMATCH ***" },
                 if ok_b { "OK" } else { "*** MISMATCH ***" });
    }
    if bad > 0 {
        return Err(format!("{bad} EC level(s) mismatched -- the capacity function is \
                            measuring a different mode than it claims, and every number \
                            derived from it is suspect"));
    }
    println!("\ngate PASSES: cap() measures the mode it claims, at every EC level.");
    Ok(())
}
