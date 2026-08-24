//! select2 -- the 2026-08-24 objective. `select.rs` IS KEPT UNCHANGED so the
//! 2026-08-22 results file stays reproducible; this is a second measurement, not
//! an edit of the first.
//!
//! WHAT CHANGED, and why (SPEC_engrave_transaction.md 4.5, 4.5a, 4.5b):
//!
//!   CONSTRAINT  ECC >= M       operator ruling. A configuration below the floor
//!                             is DISCARDED, not ranked -- same status as "does
//!                             not fit the plate".
//!   CONSTRAINT  symbols <= 16  QR Structured Append's 4-bit count-1 field.
//!
//!     1. minimise plates
//!     2. minimise SYMBOL COUNT   <- was ranked 3rd, now 2nd
//!     3. maximise ECC            <- was ranked 2nd, now 3rd
//!     4. TIE-BREAK: maximise MODULE SIZE   <- was ABSENT: ties fell to loop
//!                                             order, which ascends from
//!                                             0.30mm, so they broke toward the
//!                                             SMALLEST, least legible symbol
//!     5. then minimise QR version          <- also absent
//!
//! Symbols outrank ECC because QR Structured Append has NO CROSS-SYMBOL
//! REDUNDANCY: lose one symbol and the whole message is lost. Each extra symbol
//! is an independent fatal point. ECC keeps a FLOOR rather than a rank because
//! it is the only thing that survives DISTRIBUTED damage.
//!
//! And the legend reservation is COMPUTED from the field set and the face
//! instead of a hard-coded 6 lines -- see LEGEND_* below.

use qrcode::{EcLevel, QrCode, Version};

const USABLE_MM: f64 = 79.0;
const QUIET: usize = 4;
/// The plate must hold the QR *and* its legend. Measured in legend.rs: the
/// minimal legend is 5 fields / 136 chars / 6 lines, and the fork's own budget
/// comment implies a 4.25 mm line pitch (85 mm / 20 lines). Plate 1 carries the
/// full legend; later plates carry only "PLATE n OF m", one line.
const LINE_PITCH_MM: f64 = 85.0 / 20.0;
/// PACKED, not one field per line. The five 5-section fields total 153
/// characters; `PLATE n OF m` is the sixth and is NORMATIVE (4.4), which the
/// first version of 4.5a omitted. font/sh gives 44 columns at the 3.0mm face,
/// which gui/freetext_proof.go:24 calls "the smallest rung and the hardest
/// legibility case" -- so 3.0mm is the floor and this is the honest column count.
const LEGEND_COLS: f64 = 44.0;
const LEGEND_CHARS_FIRST: f64 = 153.0 + 14.0;   // five fields + "PLATE 1 OF m"
const LEGEND_CHARS_REST: f64 = 14.0;            // "PLATE n OF m" alone
fn legend_lines(chars: f64) -> f64 { (chars / LEGEND_COLS).ceil() }
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

fn cbor_uint(v: u64) -> usize {
    match v { 0..=23 => 1, 24..=255 => 2, 256..=65535 => 3, 65536..=4294967295 => 5, _ => 9 }
}

/// Characters ONE multi-part UR fragment occupies, uppercased. Structure read
/// from the fork: a 5-element CBOR array (fountain.go:73-80) of SeqNum, SeqLen,
/// MessageLen, Checksum, Data, deterministically encoded, then bytewords
/// (2 ch/byte + 8-char CRC32) behind `ur:<type>/<n>-<m>/` (ur.go:117-123).
/// A single-part UR skips the fountain wrapper entirely (ur.go:118).
fn ur_chars_per_fragment(msg: usize, seq_len: usize) -> usize {
    if seq_len == 1 { return 3 + 5 + 1 + msg * 2 + 8 }   // "ur:bytes/" + bytewords
    let n = msg.div_ceil(seq_len);
    let cbor = 1 + cbor_uint(seq_len as u64) + cbor_uint(seq_len as u64)
             + cbor_uint(msg as u64) + 5 + cbor_uint(n as u64) + n;
    cbor * 2 + 8 + 3 + 5 + 1 + seq_len.to_string().len() * 2 + 2
}

/// Smallest fragment count whose parts each fit `cap` characters. None if even
/// a very fine split cannot fit — the symbol is simply too small.
fn ur_fragments(msg: usize, cap: usize) -> Option<usize> {
    (1..=512usize).find(|&sl| ur_chars_per_fragment(msg, sl) <= cap)
}
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
            // Reserve legend height. Symbols tile the width freely, but the
            // vertical budget is shared with the text.
            let h_first = USABLE_MM - legend_lines(LEGEND_CHARS_FIRST) * LINE_PITCH_MM;
            let h_rest  = USABLE_MM - legend_lines(LEGEND_CHARS_REST)  * LINE_PITCH_MM;
            let across = (USABLE_MM / fp).floor() as usize;
            let rows_first = (h_first / fp).floor() as usize;
            let rows_rest  = (h_rest  / fp).floor() as usize;
            if across == 0 || rows_rest == 0 { continue }
            let first_cap = across * rows_first;   // may be 0: QR too tall to share with the legend
            let per_plate = across * rows_rest;
            for ec in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H] {
                let c = if alnum { caps.alnum[v as usize][ec_rank(ec) as usize] }
                        else { caps.byte[v as usize][ec_rank(ec) as usize] };
                if c == 0 { continue }
                let symbols = units.div_ceil(c);
                // plate 1 holds `first_cap`; the rest hold `per_plate` each
                let plates = if symbols <= first_cap { 1 }
                             else if first_cap == 0 {
                                 // legend cannot share a plate with this symbol
                                 // size: it needs a plate of its own
                                 1 + symbols.div_ceil(per_plate)
                             } else { 1 + (symbols - first_cap).div_ceil(per_plate) };
                // CONSTRAINTS first -- discarded, never ranked.
                if ec_rank(ec) < ec_rank(EcLevel::M) { continue }   // ECC floor
                if symbols > 16 { continue }                        // Structured Append
                let key = |plates: usize, symbols: usize, ec: EcLevel, mm: f64, v: u8| (
                    plates,
                    symbols,
                    std::cmp::Reverse(ec_rank(ec)),
                    std::cmp::Reverse((mm * 100.0).round() as i64),  // max module
                    v,                                              // min version
                );
                let better = match &best {
                    None => true,
                    Some(b) => key(plates, symbols, ec, mm, v)
                             < key(b.plates, b.symbols, b.ec, b.module_mm, b.ver),
                };
                if better { best = Some(Pick { plates, symbols, ec, ver: v, module_mm: mm, per_plate }); }
            }
        }
    }
    best
}

/// UR selection: a fragment must fit a WHOLE symbol, so symbol count comes from
/// splitting, not from dividing a flat character total.
fn best_ur(msg: usize, min_module_mm: f64, caps: &Caps) -> Option<Pick> {
    let mut best: Option<Pick> = None;
    for &mm in MODULES_MM.iter().filter(|m| **m >= min_module_mm - 1e-9) {
        for v in 1..=40u8 {
            let fp = (modules(v) + 2 * QUIET) as f64 * mm;
            let h_first = USABLE_MM - legend_lines(LEGEND_CHARS_FIRST) * LINE_PITCH_MM;
            let h_rest = USABLE_MM - legend_lines(LEGEND_CHARS_REST) * LINE_PITCH_MM;
            let across = (USABLE_MM / fp).floor() as usize;
            let rows_first = (h_first / fp).floor() as usize;
            let rows_rest = (h_rest / fp).floor() as usize;
            if across == 0 || rows_rest == 0 { continue }
            let first_cap = across * rows_first;
            let per_plate = across * rows_rest;
            for ec in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H] {
                let c = caps.alnum[v as usize][ec_rank(ec) as usize];
                if c == 0 { continue }
                let Some(symbols) = ur_fragments(msg, c) else { continue };
                let plates = if symbols <= first_cap { 1 }
                             else if first_cap == 0 { 1 + symbols.div_ceil(per_plate) }
                             else { 1 + (symbols - first_cap).div_ceil(per_plate) };
                let better = match &best {
                    None => true,
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

    let a = best(raw, false, min_mm, caps);
    let b = best_ur(raw, min_mm, caps);
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
        println!("  rule: fewest plates, then FEWEST SYMBOLS, then strongest ECC. Floor: ECC >= M. Cap: 16 symbols.");
        println!("  legend is PACKED and COMPUTED: {:.0} chars / {:.0} cols = {:.0} lines = {:.1}mm on plate 1\n",
                 LEGEND_CHARS_FIRST, LEGEND_COLS,
                 legend_lines(LEGEND_CHARS_FIRST),
                 legend_lines(LEGEND_CHARS_FIRST) * LINE_PITCH_MM);
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
            // THE PATHOLOGICAL WALLET (11 keys, 3 masters), wsh, tier 1 --
            // the most expensive spend path the constellation describes, and
            // absent from every earlier version of this table. Sizes from
            // RESULTS_2026-08-22.txt, signed transactions.
            ("PATH wsh t1, 1in/1out", 852),
            ("PATH wsh t1, 1in/2out", 893),
            ("PATH wsh t1, 2in/2out", 1692),
            ("PATH wsh t1, 5in/2out", 4080),
            ("PATH wsh t1, 10in/2out", 8067),
        ] { row(l, n, min_mm, &caps); }

        if (min_mm - 0.60).abs() < 1e-9 {
            println!("\n  --- WHAT GOES IN THE QR: three candidate payloads ---");
            for (label, raw) in [("RCW tr tier4 1in", 465usize),
                                 ("RCW tr tier1 1in", 595),
                                 ("RCW wsh tier1 1in", 802),
                                 ("RCW tr tier1 5in", 2769),
                                 ("RCW wsh tier1 5in", 3809)] {
                println!("  {label}  (PSBT {raw} B, {} chunks)", raw.div_ceil(40));
                for (form, qr_bytes, eff) in qr_payload_forms(raw) {
                    let p = best(qr_bytes, false, min_mm, &caps);
                    match p {
                        Some(k) => println!("      {form:<16} {qr_bytes:>5} B in QR  eff {:>5.1}%  -> {} pl, {} qr, v{} ECC {:?}",
                                            eff*100.0, k.plates, k.symbols, k.ver, k.ec),
                        None => println!("      {form:<16} {qr_bytes:>5} B in QR  eff {:>5.1}%  -> DOES NOT FIT", eff*100.0),
                    }
                }
            }
        }

        // ---- R0 T-3: what the COMPLIANT envelope costs, in PLATES ----------
        // BCR-2020-005 forbids `ur:bytes` outside testing and the BCR-2020-006
        // registry has no raw-transaction type, so the compliant payload is a
        // fully finalized PSBT (extract -> the same raw signed transaction).
        // Sizes measured by psbtfinal.rs, MIN form: UTXO records kept so the
        // standard extract_tx() fee check passes, output maps cleared because
        // no extractor or broadcaster reads them. Pairs are (raw, psbt-min) for
        // the SAME artifact, so the two rows differ only by the wrapper.
        println!("\n  --- same artifacts as a finalized PSBT (compliant envelope) ---");
        for (l, raw, psbt) in [
            ("RCW tr tier3, 1in/1out", 333usize, 391usize),
            ("RCW tr tier4, 1in/1out", 405, 465),
            ("RCW tr tier1, 1in/1out", 535, 595),
            ("RCW wsh tier3, 1in/1out", 566, 626),
            ("RCW wsh tier1, 1in/1out", 742, 802),
            ("RCW tr tier1, 5in/2out", 2498, 2769),
            ("RCW wsh tier1, 5in/2out", 3538, 3809),
        ] {
            row(&format!("{l} [raw]"), raw, min_mm, &caps);
            row(&format!("{l} [PSBT]"), psbt, min_mm, &caps);
        }
    }
}

// ─── What actually goes INTO the QR? (operator question, 2026-08-23) ─────────
//
// Three candidate payloads for `mt qr`, measured against each other. The
// question is whether carrying the codex32 STRING into the QR -- which would
// give one artifact definition for both verbs, and BCH t=4 on top of QR's
// Reed-Solomon -- costs an acceptable number of plates.
//
// Per mt1 chunk, from md-codec's real geometry:
//   header      37 bits   (version, chunk_set_id, count, index)
//   payload    320 bits   (SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64*5, = 40 bytes)
//   checksum    13 symbols = 65 bits  (REGULAR_CHECKSUM_SYMBOLS)
//   hrp+sep     "mt1" + "1" = 4 chars
fn qr_payload_forms(raw: usize) -> Vec<(&'static str, usize, f64)> {
    let chunks = raw.div_ceil(40);

    // (1) codex32 STRING in QR alphanumeric mode.
    let data_syms = (37 + 320usize).div_ceil(5);        // 72 symbols
    let chars_per_chunk = data_syms + 13 + 4;           // + checksum + hrp/sep
    let total_chars = chunks * chars_per_chunk;
    // alnum packs 2 chars per 11 bits
    let alnum_bytes = (total_chars.div_ceil(2) * 11).div_ceil(8);

    // (2) header+payload as BYTES, no BCH. 37 bits rounds to 5 bytes/chunk.
    let bin_bytes = raw + chunks * 5;

    // (3) same as (2) but base45'd: 2 bytes -> 3 chars, alnum.
    let b45_chars = bin_bytes.div_ceil(2) * 3;
    let b45_bytes = (b45_chars.div_ceil(2) * 11).div_ceil(8);

    // (4) bech32 UPPERCASE: 5 data bits per character, alnum-packed at 11 bits
    //     per 2 chars. No SPACE in its charset, so unlike base45 it satisfies
    //     EPD 6.4's canonical-record rule, and lowercasing is lossless so it
    //     survives EPD 6.6's lowercase hashing.
    let b32_chars = (bin_bytes * 8).div_ceil(5);
    let b32_bytes = (b32_chars.div_ceil(2) * 11).div_ceil(8);

    vec![
        ("codex32 string", alnum_bytes, raw as f64 / alnum_bytes as f64),
        ("bytes + base45", b45_bytes,   raw as f64 / b45_bytes as f64),
        ("bytes + bech32U", b32_bytes,  raw as f64 / b32_bytes as f64),
        ("bytes, binary",  bin_bytes,   raw as f64 / bin_bytes as f64),
    ]
}
