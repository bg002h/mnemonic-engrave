//! How many QR codes, and how many plates, does an unsigned PSBT take for
//! 9-of-11 and 3-of-5 multisig, in both wrappers?
//!
//! Plate/QR constants read from the fork: 85x85 mm, outerMargin 3 mm
//! (backup.go:45) => 79 mm usable; strokeWidth 0.3 mm is the smallest feature
//! that can be engraved, so it is the module floor. Quiet zone 4 modules/side.

use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::psbt::Psbt;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{
    absolute::LockTime, transaction::Version, Amount, Network, OutPoint, ScriptBuf, Sequence,
    Transaction, TxIn, TxOut, Txid, Witness,
};
use bitcoin::hashes::Hash as _;
use bitcoin::consensus::Encodable;
use miniscript::psbt::PsbtExt;
use miniscript::{Descriptor, DescriptorPublicKey};
use qrcode::{EcLevel, QrCode, Version as QrVersion};
use std::str::FromStr;

/// Bits of PAYLOAD each md1 chunk carries, read from the REAL chunker rather
/// than derived from codex32's theoretical long-form capacity.
///
/// `md-codec` sizes chunks by `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64 * 5 = 320`
/// (`md-codec/src/chunk.rs:224`), applied as
/// `payload_bytes.len() * 8 / 320` rounded up (`chunk.rs:253-254`). That is a
/// flat **40 payload bytes per chunk**, and the 64-chunk cap is therefore
/// **2,560 B**, not the 2,904 B a filled-capacity model predicts.
///
/// THIS CONSTANT WAS 363 AND THAT WAS WRONG. 363 = 80 symbols x 5 bits - 37
/// header bits, i.e. what a chunk COULD carry if the chunker filled to
/// codex32's long-form maximum. It does not: it balances at a 320-bit budget.
/// The old model ran ~13% light on every chunk count in every results file.
const CHUNK_PAYLOAD_BITS: usize = 320;


const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
const USABLE_MM: f64 = 79.0;
const STROKE_MM: f64 = 0.3;
const QUIET: usize = 4;

fn master(i: usize) -> Xpriv {
    let mut seed = [0u8; 32];
    seed[31] = (i + 1) as u8;
    Xpriv::new_master(Network::Bitcoin, &seed).unwrap()
}
fn keyexpr(i: usize, branch: u32) -> String {
    let secp = Secp256k1::new();
    let m = master(i);
    let path = "48'/0'/0'/2'";
    let c = m.derive_priv(&secp, &DerivationPath::from_str(path).unwrap()).unwrap();
    format!("[{}/{}]{}/{}/*", m.fingerprint(&secp), path, c, branch)
}
fn outpoint(i: u32) -> OutPoint {
    let mut b = [0u8; 32]; b[0] = 0x77; b[31] = i as u8;
    OutPoint { txid: Txid::from_raw_hash(bitcoin::hashes::sha256d::Hash::from_byte_array(b)), vout: i }
}

/// byte-mode capacity of an exact version+ECC, measured (lowercase forces byte
/// mode with no ECI; gated against published v40 limits in qrmodes.rs)
fn cap(v: u8, ec: EcLevel) -> usize {
    let fits = |n: usize| {
        let d: Vec<u8> = (0..n).map(|i| b"abcdefghijklmnopqrstuvwxyz"[i % 26]).collect();
        QrCode::with_version(&d, QrVersion::Normal(v as i16), ec).is_ok()
    };
    if !fits(1) { return 0 }
    let (mut lo, mut hi) = (1usize, 3000usize);
    while lo < hi { let m = (lo + hi + 1) / 2; if fits(m) { lo = m } else { hi = m - 1 } }
    lo
}
/// ALPHANUMERIC-mode capacity (uppercase letters force one alnum segment; a
/// digits+letters payload would be split into segments and measure ~6.6% low).
fn cap_alnum(v: u8, ec: EcLevel) -> usize {
    let fits = |n: usize| {
        let d: Vec<u8> = (0..n).map(|i| b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"[i % 26]).collect();
        QrCode::with_version(&d, QrVersion::Normal(v as i16), ec).is_ok()
    };
    if !fits(1) { return 0 }
    let (mut lo, mut hi) = (1usize, 5000usize);
    while lo < hi { let m = (lo + hi + 1) / 2; if fits(m) { lo = m } else { hi = m - 1 } }
    lo
}

/// Characters a codex32 chunk-set costs for `n` payload bytes.
/// NOT 363. See CHUNK_PAYLOAD_BITS below: the real chunker sizes by 320
/// payload bits; the engraved string is hrp(3) + 80 + 13 checksum = 96 chars.
/// Capped at 64 chunks by the wire format.
fn codex32_chars(n: usize) -> Option<usize> {
    let chunks = (n * 8).div_ceil(CHUNK_PAYLOAD_BITS);
    if chunks > 64 { return None }
    Some(chunks * 96)
}

fn modules(v: u8) -> usize { 4 * v as usize + 17 }
/// how many version-v symbols tile one plate at module_mm
fn per_plate(v: u8, module_mm: f64) -> usize {
    let fp = (modules(v) + 2 * QUIET) as f64 * module_mm;
    let k = (USABLE_MM / fp).floor() as usize;
    k * k
}

struct Masters(Vec<Xpriv>);
impl bitcoin::psbt::GetKey for Masters {
    type Error = bitcoin::psbt::GetKeyError;
    fn get_key<C: bitcoin::secp256k1::Signing>(
        &self, req: bitcoin::psbt::KeyRequest, secp: &Secp256k1<C>,
    ) -> Result<Option<bitcoin::PrivateKey>, Self::Error> {
        self.0.iter().find_map(|x| x.get_key(req.clone(), secp).transpose()).transpose()
    }
}

/// Build, SIGN, finalize and extract. Returns the serialised transaction length.
fn signed_bytes(desc_of: &dyn Fn(u32) -> String, n_keys: usize, n_in: usize, n_out: usize) -> Option<usize> {
    let secp = Secp256k1::new();
    let (recv, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &desc_of(0)).ok()?;
    let (change, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &desc_of(1)).ok()?;
    let ins: Vec<_> = (0..n_in as u32).map(|i| recv.at_derivation_index(i).unwrap()).collect();
    let change_def = change.at_derivation_index(0).unwrap();
    let external = ScriptBuf::from_hex(
        "5120a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90").unwrap();
    let tx = Transaction {
        version: Version::TWO, lock_time: LockTime::ZERO,
        input: (0..n_in).map(|i| TxIn { previous_output: outpoint(i as u32),
            script_sig: ScriptBuf::new(), sequence: Sequence(0xFFFF_FFFE),
            witness: Witness::new() }).collect(),
        output: {
            let sp = (n_in as u64) * 100_000 - 1_000;
            let first = if n_out == 2 { sp / 2 } else { sp };
            let mut o = vec![TxOut { value: Amount::from_sat(first), script_pubkey: external }];
            if n_out == 2 { o.push(TxOut { value: Amount::from_sat(sp - first),
                                           script_pubkey: change_def.script_pubkey() }); }
            o
        },
    };
    let mut psbt = Psbt::from_unsigned_tx(tx).ok()?;
    for (i, d) in ins.iter().enumerate() {
        psbt.inputs[i].witness_utxo =
            Some(TxOut { value: Amount::from_sat(100_000), script_pubkey: d.script_pubkey() });
        psbt.update_input_with_descriptor(i, d).ok()?;
    }
    if n_out == 2 { psbt.update_output_with_descriptor(1, &change_def).ok()?; }
    psbt.sign(&Masters((0..n_keys).map(master).collect()), &secp).ok();
    psbt.finalize_mut(&secp).ok()?;
    let t = psbt.extract_tx().ok()?;
    let mut raw = Vec::new();
    t.consensus_encode(&mut raw).ok()?;
    Some(raw.len())
}

fn psbt_bytes(desc_of: &dyn Fn(u32) -> String, n_in: usize, n_out: usize) -> Option<usize> {
    let secp = Secp256k1::new();
    let (recv, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &desc_of(0)).ok()?;
    let (change, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &desc_of(1)).ok()?;
    let ins: Vec<_> = (0..n_in as u32).map(|i| recv.at_derivation_index(i).unwrap()).collect();
    let change_def = change.at_derivation_index(0).unwrap();
    let external = ScriptBuf::from_hex(
        "5120a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90").unwrap();
    let tx = Transaction {
        version: Version::TWO, lock_time: LockTime::ZERO,
        input: (0..n_in).map(|i| TxIn { previous_output: outpoint(i as u32),
            script_sig: ScriptBuf::new(), sequence: Sequence(0xFFFF_FFFE),
            witness: Witness::new() }).collect(),
        output: {
            let sp = (n_in as u64) * 100_000 - 1_000;
            let first = if n_out == 2 { sp / 2 } else { sp };
            let mut o = vec![TxOut { value: Amount::from_sat(first), script_pubkey: external }];
            if n_out == 2 { o.push(TxOut { value: Amount::from_sat(sp - first),
                                           script_pubkey: change_def.script_pubkey() }); }
            o
        },
    };
    let mut psbt = Psbt::from_unsigned_tx(tx).ok()?;
    for (i, d) in ins.iter().enumerate() {
        psbt.inputs[i].witness_utxo =
            Some(TxOut { value: Amount::from_sat(100_000), script_pubkey: d.script_pubkey() });
        psbt.update_input_with_descriptor(i, d).ok()?;
    }
    if n_out == 2 { psbt.update_output_with_descriptor(1, &change_def).ok()?; }
    Some(psbt.serialize().len())
}

fn main() {
    // configs: (label, version, ecc, module mm)
    let v40l = cap(40, EcLevel::L); let v40h = cap(40, EcLevel::H);
    let v26l = cap(26, EcLevel::L); let v26h = cap(26, EcLevel::H);
    let v15m = cap(15, EcLevel::M);
    let cfgs: [(&str, usize, usize); 5] = [
        ("fork today  v15 @0.90mm ECC M", v15m, per_plate(15, 0.90)),
        ("safe        v26 @0.60mm ECC L", v26l, per_plate(26, 0.60)),
        ("safe+robust v26 @0.60mm ECC H", v26h, per_plate(26, 0.60)),
        ("dense       v26 @0.30mm ECC L", v26l, per_plate(26, 0.30)),
        ("max single  v40 @0.30mm ECC L", v40l, per_plate(40, 0.30)),
    ];
    println!("QR configs (capacity per symbol, symbols that tile one 79 mm plate):");
    for (n, c, p) in cfgs { println!("  {n}   {c:>5} B/symbol   {p} symbol(s)/plate   => {:>5} B/plate", c * p); }
    println!("  (v40 ECC H = {v40h} B/symbol, v26 ECC H = {v26h} B/symbol)\n");

    let wallets: Vec<(&str, Box<dyn Fn(u32) -> String>)> = vec![
        ("3-of-5  wsh", Box::new(|b| format!("wsh(multi(3,{}))",
            (0..5).map(|i| keyexpr(i, b)).collect::<Vec<_>>().join(",")))),
        ("3-of-5  tr ", Box::new(|b| format!("tr({NUMS},multi_a(3,{}))",
            (0..5).map(|i| keyexpr(i, b)).collect::<Vec<_>>().join(",")))),
        ("9-of-11 wsh", Box::new(|b| format!("wsh(multi(9,{}))",
            (0..11).map(|i| keyexpr(i, b)).collect::<Vec<_>>().join(",")))),
        ("9-of-11 tr ", Box::new(|b| format!("tr({NUMS},multi_a(9,{}))",
            (0..11).map(|i| keyexpr(i, b)).collect::<Vec<_>>().join(",")))),
    ];

    println!("=== SIGNED TRANSACTION vs UNSIGNED PSBT, same wallets, 1 in / 1 out (sweep) ===");
    println!("  {:<12} {:>10} {:>12} {:>9}   {:>18} {:>18}",
             "wallet", "PSBT", "SIGNED TX", "ratio", "PSBT: v26 qr/pl", "SIGNED: v26 qr/pl");
    for (name, nk, f) in [("3-of-5  wsh", 5usize, &wallets[0].1), ("3-of-5  tr ", 5, &wallets[1].1),
                          ("9-of-11 wsh", 11, &wallets[2].1), ("9-of-11 tr ", 11, &wallets[3].1)] {
        let pb = psbt_bytes(f.as_ref(), 1, 1);
        let sb = signed_bytes(f.as_ref(), nk, 1, 1);
        match (pb, sb) {
            (Some(pb), Some(sb)) => {
                let c = cap(26, EcLevel::L);
                println!("  {name:<12} {pb:>8} B {sb:>10} B {:>8.0}%   {:>18} {:>18}",
                         (sb as f64 / pb as f64) * 100.0,
                         format!("{} qr / {} pl", pb.div_ceil(c), pb.div_ceil(c)),
                         format!("{} qr / {} pl", sb.div_ceil(c), sb.div_ceil(c)));
            }
            (Some(pb), None) => println!("  {name:<12} {pb:>8} B   SIGN/FINALIZE FAILED", ),
            _ => println!("  {name:<12}  FAILED"),
        }
    }
    println!();

    // What codex32 would cost, for the same PSBTs, in the same QR.
    println!("=== THE SAME PSBT, WRAPPED IN CODEX32 FIRST (what this probe does NOT do) ===");
    println!("  raw bytes go in QR BYTE mode; a codex32 string is bech32, and UPPERCASED it");
    println!("  goes in ALPHANUMERIC mode. v26 alnum {} ch, v40 alnum {} ch (ECC L).",
             cap_alnum(26, EcLevel::L), cap_alnum(40, EcLevel::L));
    println!("  {:<12} {:>8} {:>10} {:>12} {:>14} {:>14}",
             "wallet(1in/1out)", "PSBT", "cx32 chars", "cx32 chunks", "raw: v26 qr", "cx32: v26 qr");
    for (name, f) in &wallets {
        if let Some(b) = psbt_bytes(f.as_ref(), 1, 1) {
            let raw_qr = b.div_ceil(cap(26, EcLevel::L));
            match codex32_chars(b) {
                Some(ch) => println!("  {name:<12} {b:>6} B {ch:>10} {:>12} {raw_qr:>14} {:>14}",
                                     (b * 8).div_ceil(CHUNK_PAYLOAD_BITS), ch.div_ceil(cap_alnum(26, EcLevel::L))),
                None => println!("  {name:<12} {b:>6} B {:>10} {:>12} {raw_qr:>14} {:>14}",
                                 "-", "OVER 64", "-"),
            }
        }
    }
    println!();

    for (n_in, n_out) in [(1usize, 1usize), (1, 2), (2, 2)] {
        println!("=== UNSIGNED PSBT, {n_in} in / {n_out} out ===");
        println!("  {:<12} {:>8}   {}", "wallet", "PSBT", 
                 cfgs.iter().map(|(n, _, _)| format!("{:>16}", n.split_whitespace().next().unwrap()))
                     .collect::<Vec<_>>().join(""));
        for (name, f) in &wallets {
            match psbt_bytes(f.as_ref(), n_in, n_out) {
                Some(b) => {
                    let cells: String = cfgs.iter().map(|(_, c, p)| {
                        let sym = b.div_ceil(*c);
                        let plates = sym.div_ceil(*p);
                        format!("{:>16}", format!("{sym} qr / {plates} pl"))
                    }).collect();
                    println!("  {name:<12} {b:>6} B   {cells}");
                }
                None => println!("  {name:<12}  PARSE/BUILD FAILED"),
            }
        }
        println!();
    }
}
