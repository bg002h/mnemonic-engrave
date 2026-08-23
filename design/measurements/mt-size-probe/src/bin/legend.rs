//! Does the §5 plate legend fit, and does it fit BESIDE the QR the §4 rule picks?
//!
//! Two separate questions, and the second is the one that bites:
//!   (a) character count vs PLATE_TEXT_BUDGET = 300 (me-cli/src/lib.rs:48)
//!   (b) PHYSICAL AREA left over once the ECC-maximising QR is placed
//!
//! Layout constants from the fork: plate 85x85 mm, outerMargin 3 mm => 79 mm
//! usable (backup.go:45,99-102); plateFontSize 4.1 (backup.go:89). The budget
//! comment states the geometry it assumes: "~35 chars/line over ~20 usable
//! lines; with a QR present, far less."

const USABLE_MM: f64 = 79.0;
const BUDGET_CHARS: usize = 300;
const CHARS_PER_LINE: f64 = 35.0;   // over the full 85 mm width, per lib.rs:46
const LINES_FULL_PLATE: f64 = 20.0; // ditto
const QUIET: usize = 4;

fn modules(v: u8) -> usize { 4 * v as usize + 17 }

/// A concrete legend for a 1-input / 1-output sweep, built from the real fields
/// §5 lists. Values are real lengths: a txid is 64 hex chars, an outpoint is
/// txid:vout, a bech32m p2tr address is 62.
fn legend(n_inputs: usize) -> Vec<(&'static str, String)> {
    let txid = "9a3f21c0d4e5b6a7f8091a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f";
    let addr = "bc1p8rrz3ts8u4dm2fu7ax3hlwywy3esads3dz2ykrwrwvcjrcqz5q3s6n0vcl";
    let blockhash = "0000000000000000000021c5fe68b74cfc766960ea8ed93b2bcda93cf7845fa8";
    let mut v = vec![
        ("bearer warning", "BEARER - ANYONE HOLDING THIS PLATE CAN SPEND IT".into()),
        ("source wallet",  "FROM WALLET fa568be0".into()),
        ("txid",           format!("TXID {txid}")),
        ("destination",    format!("TO {addr}  0.00399 BTC")),
        ("locktime",       "LOCKED TO BLOCK 1383520 ~FALL 2034".into()),
        ("fee",            "FEE 12 SAT/VB SET 2026-08-22".into()),
        ("mtp bound",      "INPUTS EXISTED NOT BEFORE 2026-08-22T23:22Z".into()),
        ("symbol index",   "PLATE 1 OF 1".into()),
    ];
    for i in 0..n_inputs {
        v.push(("input outpoint", format!("IN{i} {txid}:{i}")));
        v.push(("block anchor",   format!("   BLK 963650 {blockhash}")));
    }
    v
}

/// MINIMAL legend: only what a human needs BEFORE the QR is decoded. Every
/// field derivable from the decoded transaction is dropped — engraving it is
/// pure duplication, and if the QR is unreadable the duplicate is useless
/// anyway because you still have no transaction.
fn minimal_legend() -> Vec<(&'static str, String)> {
    vec![
        ("bearer warning", "BEARER - ANYONE HOLDING THIS CAN SPEND IT".into()),
        ("source wallet",  "FROM WALLET fa568be0".into()),
        ("locktime",       "LOCKED TO BLOCK 1383520 ~FALL 2034".into()),
        ("destination",    "TO bc1p8rrz...s6n0vcl  0.00399 BTC".into()),
        ("symbol index",   "PLATE 1 OF 1".into()),
    ]
}

fn main() {
    println!("PLATE_TEXT_BUDGET = {BUDGET_CHARS} chars ({CHARS_PER_LINE:.0} chars/line x {LINES_FULL_PLATE:.0} lines, TEXT-ONLY plate)\n");

    for n in [1usize, 2, 5] {
        let l = legend(n);
        let chars: usize = l.iter().map(|(_, s)| s.len()).sum();
        let lines: f64 = l.iter().map(|(_, s)| (s.len() as f64 / CHARS_PER_LINE).ceil()).sum();
        println!("  FULL legend, {n}-input sweep: {:>2} fields, {chars:>4} ch, {lines:>2.0} lines -> {}",
                 l.len(), if chars <= BUDGET_CHARS { "fits budget" } else { "OVER BUDGET" });
    }

    let m = minimal_legend();
    let mchars: usize = m.iter().map(|(_, s)| s.len()).sum();
    let mlines: f64 = m.iter().map(|(_, s)| (s.len() as f64 / CHARS_PER_LINE).ceil()).sum();
    println!("\n  MINIMAL legend (any input count): {} fields, {mchars} ch, {mlines:.0} lines -> {}",
             m.len(), if mchars <= BUDGET_CHARS { "fits budget" } else { "OVER BUDGET" });
    for (k, s) in &m { println!("     {:<16} {:>3} ch  {}", k, s.len(), s); }

    println!("\n=== ROOM BESIDE THE QR — the binding constraint ===");
    println!("  {:<28} {:>8} {:>11} {:>7} {:>7}  {:>12} {:>12}",
             "config @0.60mm", "QR mm", "strip mm", "lines", "chars", "FULL(1in)", "MINIMAL");
    let full1: usize = legend(1).iter().map(|(_, s)| s.len()).sum();
    for (label, ver) in [("RCW tr key-path  v13 ECC H", 13u8), ("RCW tr tier4     v22 ECC H", 22),
                         ("3-of-5 signed    v24 ECC H", 24), ("RCW tr tier1     v25 ECC H", 25),
                         ("RCW wsh tier1    v26 ECC Q", 26)] {
        let qr_mm = (modules(ver) + 2 * QUIET) as f64 * 0.60;
        let strip = USABLE_MM - qr_mm;
        let lines = (strip / (85.0 / LINES_FULL_PLATE)).floor().max(0.0);
        let avail = (lines * CHARS_PER_LINE) as usize;
        println!("  {label:<28} {qr_mm:>6.1}mm {strip:>9.1}mm {lines:>7.0} {avail:>7}  {:>12} {:>12}",
                 if avail >= full1 { "fits" } else { "NO" },
                 if avail >= mchars { "fits" } else { "NO" });
    }

    println!("\n=== what ECC each config could afford if the legend needs {mlines:.0} lines ({:.1}mm) ===", mlines * (85.0 / LINES_FULL_PLATE));
    let need_mm = mlines * (85.0 / LINES_FULL_PLATE);
    for ver in [13u8, 18, 20, 22, 24, 25, 26] {
        let qr_mm = (modules(ver) + 2 * QUIET) as f64 * 0.60;
        println!("  v{ver:<3} {qr_mm:>6.1}mm  + legend {need_mm:.1}mm = {:>5.1}mm  {}",
                 qr_mm + need_mm, if qr_mm + need_mm <= USABLE_MM { "fits one plate" } else { "NEEDS A SECOND PLATE" });
    }
}
