//! Exact UR multi-part overhead, and what redundancy costs.
//!
//! Structure read from the fork, not from the BCR spec: a part is a 5-element
//! CBOR array (`cbor:",toarray"`, fountain.go:73-80) of
//!   SeqNum:u32, SeqLen:int, MessageLen:int, Checksum:u32, Data:[]byte
//! deterministically encoded (CoreDetEncOptions, fountain.go:62), then
//! bytewords-encoded (2 chars/byte + an 8-char CRC32) and prefixed
//! `ur:<type>/<seq>-<len>/` (ur.go:117-123).
//!
//! Fragment payload is n = ceil(messageLen / seqLen), the SAME for every part
//! (fountain.go:32).

use qrcode::{EcLevel, QrCode, Version};

const USABLE_MM: f64 = 79.0;
const QUIET: usize = 4;
const LINE_PITCH_MM: f64 = 85.0 / 20.0;
const LEGEND_LINES_FIRST: f64 = 6.0;

/// Bytes a deterministic CBOR unsigned integer occupies (head only).
fn cbor_uint(v: u64) -> usize {
    match v { 0..=23 => 1, 24..=255 => 2, 256..=65535 => 3, 65536..=4294967295 => 5, _ => 9 }
}
/// Bytes a CBOR byte-string head occupies for `len` bytes of content.
fn cbor_bstr_head(len: usize) -> usize { cbor_uint(len as u64) }

/// Characters one multi-part UR fragment occupies, uppercased.
fn ur_fragment_chars(ur_type: &str, msg_len: usize, seq_len: usize, seq_num: usize) -> usize {
    let n = msg_len.div_ceil(seq_len);                 // payload per part
    let cbor = 1                                        // array(5) head
        + cbor_uint(seq_num as u64)                     // SeqNum
        + cbor_uint(seq_len as u64)                     // SeqLen
        + cbor_uint(msg_len as u64)                     // MessageLen
        + 5                                             // Checksum, a u32 that is
                                                        // effectively always >= 65536
        + cbor_bstr_head(n) + n;                        // Data
    let bytewords = cbor * 2 + 8;                       // 2 ch/byte + CRC32 as 8 ch
    let prefix = 3 + ur_type.len() + 1                  // "ur:" + type + "/"
        + seq_num.to_string().len() + 1 + seq_len.to_string().len() + 1; // "N-M/"
    bytewords + prefix
}

fn cap_alnum(v: u8, ec: EcLevel) -> usize {
    let fits = |n: usize| {
        let d: Vec<u8> = (0..n).map(|i| b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"[i % 26]).collect();
        QrCode::with_version(&d, Version::Normal(v as i16), ec).is_ok()
    };
    if !fits(1) { return 0 }
    let (mut lo, mut hi) = (1usize, 5000usize);
    while lo < hi { let m = (lo + hi + 1) / 2; if fits(m) { lo = m } else { hi = m - 1 } }
    lo
}
fn modules(v: u8) -> usize { 4 * v as usize + 17 }

fn main() {
    println!("=== PER-FRAGMENT OVERHEAD (ur:bytes), exact ===");
    println!("  {:<10} {:>8} {:>9} {:>11} {:>10} {:>12}",
             "msg bytes", "seqLen", "payload/pt", "CBOR bytes", "overhead", "chars/part");
    for &msg in &[162usize, 535, 1097, 2455] {
        for &sl in &[1usize, 2, 4] {
            if sl == 1 {
                // single-part: no fountain wrapper at all (ur.go:118-119)
                let chars = 3 + 5 + 1 + (msg * 2 + 8);
                println!("  {msg:<10} {:>8} {:>9} {:>11} {:>10} {:>12}",
                         "1 (none)", msg, "-", "0 (raw)", chars);
                continue;
            }
            let n = msg.div_ceil(sl);
            let cbor = 1 + cbor_uint(1) + cbor_uint(sl as u64) + cbor_uint(msg as u64)
                     + 5 + cbor_bstr_head(n) + n;
            let chars = ur_fragment_chars("bytes", msg, sl, 1);
            println!("  {msg:<10} {sl:>8} {n:>9} {cbor:>11} {:>10} {chars:>12}", cbor - n);
        }
    }

    println!("\n=== THE REDUNDANCY QUESTION ===");
    println!("  fountain.go:242 — parts 1..seqLen are SINGLETONS (one pure fragment each).");
    println!("  Only seqNum > seqLen produces mixed parts. So emitting exactly seqLen");
    println!("  parts gives ZERO redundancy: lose one and the message is unrecoverable.");
    println!("  Decoder::Progress() estimates seqLen * 1.75 parts for RANDOM reception.\n");
    println!("  {:<28} {:>10} {:>12} {:>14}", "policy", "parts", "survives", "cost vs bare");
    for (label, extra) in [("exactly seqLen (singletons)", 0usize), ("seqLen + 1", 1), ("seqLen + 2", 2)] {
        println!("  {label:<28} {:>10} {:>12} {:>14}",
                 format!("seqLen+{extra}"),
                 if extra == 0 { "no loss".to_string() } else { format!("any {extra} lost") },
                 if extra == 0 { "-".to_string() } else { format!("+{extra} symbols") });
    }

    println!("\n=== PLATES, legend reserved, 0.60mm, ECC H, with and without redundancy ===");
    println!("  {:<22} {:>8} {:>10} {:>12} {:>14}", "artifact", "bytes", "seqLen", "plates(bare)", "plates(+1 redun)");
    for (label, msg) in [("RCW tr key-path", 162usize), ("RCW tr tier1 1in", 535),
                         ("9-of-11 signed 1in", 1097), ("RCW tr tier1 5in", 2455)] {
        // pick the largest version that fits beside the legend at 0.60mm
        let mut best = None;
        for v in 1..=40u8 {
            let fp = (modules(v) + 2 * QUIET) as f64 * 0.60;
            if fp <= USABLE_MM - LEGEND_LINES_FIRST * LINE_PITCH_MM { best = Some(v) }
        }
        let v = best.unwrap();
        let c = cap_alnum(v, EcLevel::H);
        // find the smallest seqLen whose fragment fits one symbol
        let mut sl = 1usize;
        while sl < 200 && ur_fragment_chars("bytes", msg, sl.max(2), 1) > c { sl += 1 }
        let parts = sl.max(1);
        let across = (USABLE_MM / ((modules(v) + 2 * QUIET) as f64 * 0.60)).floor().max(1.0) as usize;
        let per_first = across * (((USABLE_MM - LEGEND_LINES_FIRST * LINE_PITCH_MM)
                        / ((modules(v) + 2 * QUIET) as f64 * 0.60)).floor().max(0.0) as usize);
        let per_rest = across * ((USABLE_MM / ((modules(v) + 2 * QUIET) as f64 * 0.60)).floor().max(1.0) as usize);
        let plates = |p: usize| if p <= per_first { 1 } else { 1 + (p - per_first).div_ceil(per_rest.max(1)) };
        println!("  {label:<22} {msg:>8} {parts:>10} {:>12} {:>14}",
                 plates(parts), plates(parts + 1));
    }
    println!("\n  (v{} ECC H holds {} alnum chars beside the legend at 0.60mm)",
             { let mut b=1u8; for v in 1..=40u8 { if (modules(v)+2*QUIET) as f64*0.60 <= USABLE_MM - LEGEND_LINES_FIRST*LINE_PITCH_MM { b=v } } b },
             { let mut b=1u8; for v in 1..=40u8 { if (modules(v)+2*QUIET) as f64*0.60 <= USABLE_MM - LEGEND_LINES_FIRST*LINE_PITCH_MM { b=v } } cap_alnum(b, EcLevel::H) });
}
