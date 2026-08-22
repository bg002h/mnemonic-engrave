//! How many inputs can a SIGNED transaction carry before it overflows the
//! codex32 container (64 chunks / 2,904 bytes)? Sweeps every wallet until it
//! breaks, and reports the marginal cost per input.

use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::consensus::Encodable;
use bitcoin::hashes::{sha256, Hash as _};
use bitcoin::psbt::Psbt;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{
    absolute::LockTime, transaction::Version, Amount, Network, OutPoint, ScriptBuf, Sequence,
    Transaction, TxIn, TxOut, Txid, Witness,
};
use miniscript::psbt::PsbtExt;
use miniscript::{Descriptor, DescriptorPublicKey};
use std::str::FromStr;

const RCW: &str = "/scratch/code/shibboleth/mnemonic-engrave/design/journeys/inputs-rcw";
const PATH_DIR: &str = "/scratch/code/shibboleth/mnemonic-engrave/design/journeys/inputs-pathological";
const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
const CAP_CHUNKS: usize = 64;

struct Masters(Vec<Xpriv>);
impl bitcoin::psbt::GetKey for Masters {
    type Error = bitcoin::psbt::GetKeyError;
    fn get_key<C: bitcoin::secp256k1::Signing>(
        &self, req: bitcoin::psbt::KeyRequest, secp: &Secp256k1<C>,
    ) -> Result<Option<bitcoin::PrivateKey>, Self::Error> {
        self.0.iter().find_map(|x| x.get_key(req.clone(), secp).transpose()).transpose()
    }
}

fn outpoint(i: u32) -> OutPoint {
    let mut b = [0u8; 32];
    b[0] = 0xe1; b[31] = i as u8;
    OutPoint { txid: Txid::from_raw_hash(bitcoin::hashes::sha256d::Hash::from_byte_array(b)), vout: i }
}

fn rcw_master(n: usize) -> Xpriv {
    let w = std::fs::read_to_string(format!("{RCW}/seeds/key-{n}.seed")).unwrap();
    Xpriv::new_master(Network::Bitcoin, &bip39::Mnemonic::parse(w.trim()).unwrap().to_seed("")).unwrap()
}
fn path_master(letter: char) -> Xpriv {
    let seed = match letter {
        'A' => "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        'B' => "legal winner thank year wave sausage worth useful legal winner thank yellow",
        _   => "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
    };
    Xpriv::new_master(Network::Bitcoin, &bip39::Mnemonic::parse(seed).unwrap().to_seed("")).unwrap()
}
fn keyexpr(m: &Xpriv, path: &str, branch: u32) -> String {
    let secp = Secp256k1::new();
    let child = m.derive_priv(&secp, &DerivationPath::from_str(path).unwrap()).unwrap();
    format!("[{}/{}]{}/{}/*", m.fingerprint(&secp), path, child, branch)
}

struct Case {
    name: &'static str,
    /// (branch) -> descriptor string
    desc: Box<dyn Fn(u32) -> String>,
    masters: Vec<Xpriv>,
    preimages: Vec<[u8; 32]>,
    locktime: LockTime,
    sequence: Sequence,
}

fn signed_len(c: &Case, n_in: usize, n_out: usize) -> Option<(usize, Vec<usize>)> {
    let secp = Secp256k1::new();
    let (recv, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &(c.desc)(0)).ok()?;
    let (change, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &(c.desc)(1)).ok()?;
    let ins: Vec<_> = (0..n_in as u32).map(|i| recv.at_derivation_index(i).unwrap()).collect();
    let change_def = change.at_derivation_index(0).unwrap();
    let external = ScriptBuf::from_hex(
        "5120a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90").unwrap();
    let unsigned = Transaction {
        version: Version::TWO,
        lock_time: c.locktime,
        input: (0..n_in).map(|i| TxIn {
            previous_output: outpoint(i as u32), script_sig: ScriptBuf::new(),
            sequence: c.sequence, witness: Witness::new() }).collect(),
        output: {
            let sp = (n_in as u64) * 100_000 - 1_000;
            let first = if n_out == 2 { sp / 2 } else { sp };
            let mut o = vec![TxOut { value: Amount::from_sat(first), script_pubkey: external }];
            if n_out == 2 { o.push(TxOut { value: Amount::from_sat(sp - first),
                                           script_pubkey: change_def.script_pubkey() }); }
            o
        },
    };
    let mut psbt = Psbt::from_unsigned_tx(unsigned).ok()?;
    for (i, d) in ins.iter().enumerate() {
        psbt.inputs[i].witness_utxo =
            Some(TxOut { value: Amount::from_sat(100_000), script_pubkey: d.script_pubkey() });
        psbt.update_input_with_descriptor(i, d).ok()?;
        for p in &c.preimages {
            psbt.inputs[i].sha256_preimages.insert(sha256::Hash::hash(p), p.to_vec());
        }
    }
    if n_out == 2 { psbt.update_output_with_descriptor(1, &change_def).ok()?; }
    psbt.sign(&Masters(c.masters.clone()), &secp).ok();
    psbt.finalize_mut(&secp).ok()?;
    let tx = psbt.extract_tx().ok()?;
    let mut raw = Vec::new();
    tx.consensus_encode(&mut raw).unwrap();
    let shape = tx.input[0].witness.iter().map(|e| e.len()).collect();
    Some((raw.len(), shape))
}

fn chunks(n: usize) -> usize { (n * 8).div_ceil(363) }

/// The BARE-PSBT envelope, for comparison: prevouts + amounts + spk only, no
/// descriptor metadata. Depends on the scriptPubKey length, not the policy.
fn bare_sweep(name: &str, spk_len: usize) {
    // 1 input, 2 outputs, then marginal per input, both from the real serialiser.
    let build = |n_in: usize| {
        let spk = ScriptBuf::from(vec![0u8; spk_len]);
        let tx = Transaction {
            version: Version::TWO, lock_time: LockTime::ZERO,
            input: (0..n_in).map(|i| TxIn { previous_output: outpoint(i as u32),
                script_sig: ScriptBuf::new(), sequence: Sequence(0xFFFF_FFFE),
                witness: Witness::new() }).collect(),
            output: vec![
                TxOut { value: Amount::from_sat(1000), script_pubkey: spk.clone() },
                TxOut { value: Amount::from_sat(1000), script_pubkey: spk.clone() }],
        };
        let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();
        for i in 0..n_in {
            psbt.inputs[i].witness_utxo =
                Some(TxOut { value: Amount::from_sat(100_000), script_pubkey: spk.clone() });
        }
        psbt.serialize().len()
    };
    let one = build(1);
    let marginal = build(2) - one;
    let mut best = 1;
    while chunks(build(best + 1)) <= CAP_CHUNKS && best < 400 { best += 1 }
    let (at, over, nover) = (build(best), build(best + 1), best + 1);
    let (c1, cat, cov) = (chunks(one), chunks(at), chunks(over));
    println!("{name:<26} 1in {one:>4}B({c1:>2}ch) | +{marginal:>3}B/input | MAX {best:>3} inputs = {at:>4}B({cat:>2}ch) | {nover} inputs = {over:>4}B({cov:>3}ch) OVER");
}

fn sweep(c: &Case) {
    // A recovery spend is a SWEEP: one output, no change. Report that, and the
    // 2-output shape for ordinary spending.
    for n_out in [1usize, 2] {
        let (one, shape) = match signed_len(c, 1, n_out) { Some(x) => x, None => {
            println!("{:<26} FINALIZE FAILED", c.name); return } };
        let two = match signed_len(c, 2, n_out) { Some((n, _)) => n, None => return };
        let marginal = two - one;
        let mut best = 1usize;
        while best < 400 {
            match signed_len(c, best + 1, n_out) {
                Some((n, _)) if chunks(n) <= CAP_CHUNKS => best += 1,
                _ => break,
            }
        }
        let at = signed_len(c, best, n_out).unwrap().0;
        let kind = if n_out == 1 { "sweep " } else { "2-out " };
        // One plate per string today (F-225). ~21 min/plate measured.
        let plates = chunks(at);
        let hours = (plates as f64 * 21.0) / 60.0;
        if n_out == 1 {
            println!("{:<26} {kind} 1in {one:>4}B({:>2}ch) | +{marginal:>3}B/in | MAX {best:>2} in = {at:>4}B \
= {:>2}ch = {plates:>2} plates = {hours:>4.1} h   witness{:?}",
                c.name, chunks(one), chunks(at), shape);
        } else {
            println!("{:<26} {kind} 1in {one:>4}B({:>2}ch) | +{marginal:>3}B/in | MAX {best:>2} in = {at:>4}B \
= {:>2}ch = {plates:>2} plates = {hours:>4.1} h",
                c.name, chunks(one), chunks(at));
        }
    }
}

fn main() {
    let secp = Secp256k1::new();
    let mut cases: Vec<Case> = Vec::new();

    // ---- ordinary wallets ----
    let m: Vec<Xpriv> = (0..5).map(rcw_master).collect();
    let mk = |ms: Vec<Xpriv>, path: &'static str, f: fn(&[String]) -> String| {
        let ms2 = ms.clone();
        (Box::new(move |b: u32| {
            let keys: Vec<String> = ms2.iter().map(|x| keyexpr(x, path, b)).collect();
            f(&keys)
        }) as Box<dyn Fn(u32) -> String>, ms)
    };
    let plain = |name, d: Box<dyn Fn(u32) -> String>, ms| Case {
        name, desc: d, masters: ms, preimages: vec![],
        locktime: LockTime::ZERO, sequence: Sequence(0xFFFF_FFFE) };

    let (d, ms) = mk(m[..1].to_vec(), "84'/0'/0'", |k| format!("wpkh({})", k[0]));
    cases.push(plain("single-sig wpkh", d, ms));
    let (d, ms) = mk(m[..1].to_vec(), "86'/0'/0'", |k| format!("tr({})", k[0]));
    cases.push(plain("single-sig tr keypath", d, ms));
    let (d, ms) = mk(m[..3].to_vec(), "48'/0'/0'/2'", |k| format!("wsh(multi(2,{},{},{}))", k[0], k[1], k[2]));
    cases.push(plain("2-of-3 wsh", d, ms));
    let (d, ms) = mk(m[..3].to_vec(), "48'/0'/0'/2'", |k| format!("tr({NUMS},multi_a(2,{},{},{}))", k[0], k[1], k[2]));
    cases.push(plain("2-of-3 tr", d, ms));
    let (d, ms) = mk(m[..5].to_vec(), "48'/0'/0'/2'", |k| format!("wsh(multi(3,{},{},{},{},{}))", k[0], k[1], k[2], k[3], k[4]));
    cases.push(plain("3-of-5 wsh", d, ms));
    let (d, ms) = mk(m[..5].to_vec(), "48'/0'/0'/2'", |k| format!("tr({NUMS},multi_a(3,{},{},{},{},{}))", k[0], k[1], k[2], k[3], k[4]));
    cases.push(plain("3-of-5 tr", d, ms));

    // ---- RCW, real fixture, real preimages ----
    let rcw_all: Vec<Xpriv> = (0..7).map(rcw_master).collect();
    let pre = |t: usize| -> [u8; 32] {
        let h = std::fs::read_to_string(format!("{RCW}/preimages/preimage-{t}.hex")).unwrap();
        let mut o = [0u8; 32];
        for (i, b) in o.iter_mut().enumerate() {
            *b = u8::from_str_radix(&h.trim()[i * 2..i * 2 + 2], 16).unwrap() }
        o
    };
    for (form, file) in [("tr", "policy-tr.txt"), ("wsh", "policy-wsh.txt")] {
        let kd = if form == "tr" { "keys-tr" } else { "keys-wsh" };
        let base = std::fs::read_to_string(format!("{RCW}/{file}")).unwrap().trim().to_string();
        let mut origins = Vec::new();
        for n in 0..7 {
            let meta = std::fs::read_to_string(format!("{RCW}/{kd}/key-{n}.xpub")).unwrap();
            let o = meta.lines().find_map(|l| l.split_once("origin [")
                .map(|(_, r)| r.trim_end_matches(']').to_string())).unwrap();
            origins.push(o);
        }
        let b2 = base.clone(); let og = origins.clone(); let ms = rcw_all.clone();
        let build = move |branch: u32| {
            let mut out = b2.clone();
            for (n, origin) in og.iter().enumerate() {
                let (_, path) = origin.split_once('/').unwrap();
                let child = ms[n].derive_priv(&Secp256k1::new(),
                    &DerivationPath::from_str(path).unwrap()).unwrap();
                out = out.replace(&format!("@{n}/{path}"), &format!("[{origin}]{child}"));
            }
            out.replace("<0;1>", &branch.to_string())
        };
        let bb = build.clone();
        cases.push(Case { name: Box::leak(format!("RCW {form} tier1 (worst)").into_boxed_str()),
            desc: Box::new(bb), masters: rcw_all.clone(), preimages: vec![pre(0)],
            locktime: LockTime::ZERO, sequence: Sequence(0xFFFF_FFFE) });
        let bb = build.clone();
        let mut no5 = rcw_all.clone(); no5.remove(5);
        cases.push(Case { name: Box::leak(format!("RCW {form} tier4 (keyed)").into_boxed_str()),
            desc: Box::new(bb), masters: no5, preimages: vec![pre(2)],
            locktime: LockTime::from_height(1_400_000).unwrap(), sequence: Sequence(0xFFFF_FFFE) });
    }

    // ---- pathological, tier 1 ----
    let pmap: [(usize, char, u32); 11] = [(0,'A',0),(1,'A',1),(2,'A',2),(3,'A',3),(4,'B',0),
        (5,'B',1),(6,'B',2),(7,'B',3),(8,'C',0),(9,'C',1),(10,'C',2)];
    let pstand = [0x42u8; 32];
    for (form, file) in [("tr", "wallet-policy-tr.txt"), ("wsh", "wallet-policy.txt")] {
        let base = std::fs::read_to_string(format!("{PATH_DIR}/{file}")).unwrap().trim().to_string();
        let build = move |branch: u32| {
            let mut out = base.clone();
            for (n, letter, acct) in pmap {
                let m = path_master(letter);
                let path = format!("48'/0'/{acct}'/2'");
                let child = m.derive_priv(&Secp256k1::new(),
                    &DerivationPath::from_str(&path).unwrap()).unwrap();
                out = out.replace(&format!("@{n}/{path}"),
                    &format!("[{}/{path}]{child}", m.fingerprint(&Secp256k1::new())));
            }
            out = out.replace("a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad",
                              &sha256::Hash::hash(&pstand).to_string());
            out.replace("<0;1>", &branch.to_string())
        };
        cases.push(Case { name: Box::leak(format!("pathological {form} tier1").into_boxed_str()),
            desc: Box::new(build), masters: ['A','B','C'].iter().map(|c| path_master(*c)).collect(),
            preimages: vec![pstand],
            locktime: LockTime::from_height(1_900_000).unwrap(), sequence: Sequence(0xFFFF_FFFE) });
    }

    // ---- KEY-PATH variants: same taptree, but a REAL internal key instead of
    // the NUMS pin, so the wallet can be spent with a single Schnorr signature
    // and no script reveal at all. TEST ONLY internal key.
    let ikey = Xpriv::new_master(Network::Bitcoin, &[0x08u8; 32]).unwrap();
    {
        let base = std::fs::read_to_string(format!("{RCW}/policy-tr.txt")).unwrap().trim().to_string();
        let mut origins = Vec::new();
        for n in 0..7 {
            let meta = std::fs::read_to_string(format!("{RCW}/keys-tr/key-{n}.xpub")).unwrap();
            origins.push(meta.lines().find_map(|l| l.split_once("origin [")
                .map(|(_, r)| r.trim_end_matches(']').to_string())).unwrap());
        }
        let ms = rcw_all.clone(); let og = origins.clone();
        let build = move |branch: u32| {
            let mut out = base.clone();
            for (n, origin) in og.iter().enumerate() {
                let (_, path) = origin.split_once('/').unwrap();
                let child = ms[n].derive_priv(&Secp256k1::new(),
                    &DerivationPath::from_str(path).unwrap()).unwrap();
                out = out.replace(&format!("@{n}/{path}"), &format!("[{origin}]{child}"));
            }
            out = out.replace(NUMS, &keyexpr(&ikey, "86'/0'/0'", branch));
            out.replace("<0;1>", &branch.to_string())
        };
        let mut with_ikey = rcw_all.clone(); with_ikey.push(ikey);
        cases.push(Case { name: "RCW tr KEY-PATH", desc: Box::new(build),
            masters: with_ikey, preimages: vec![],
            locktime: LockTime::ZERO, sequence: Sequence(0xFFFF_FFFE) });
    }
    {
        let base = std::fs::read_to_string(format!("{PATH_DIR}/wallet-policy-tr.txt")).unwrap().trim().to_string();
        let build = move |branch: u32| {
            let mut out = base.clone();
            for (n, letter, acct) in pmap {
                let m = path_master(letter);
                let path = format!("48'/0'/{acct}'/2'");
                let child = m.derive_priv(&Secp256k1::new(),
                    &DerivationPath::from_str(&path).unwrap()).unwrap();
                out = out.replace(&format!("@{n}/{path}"),
                    &format!("[{}/{path}]{child}", m.fingerprint(&Secp256k1::new())));
            }
            out = out.replace("a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad",
                              &sha256::Hash::hash(&pstand).to_string());
            out = out.replace(NUMS, &keyexpr(&ikey, "86'/0'/0'", branch));
            out.replace("<0;1>", &branch.to_string())
        };
        let mut ms: Vec<Xpriv> = ['A','B','C'].iter().map(|c| path_master(*c)).collect();
        ms.push(ikey);
        cases.push(Case { name: "pathological tr KEY-PATH", desc: Box::new(build),
            masters: ms, preimages: vec![],
            locktime: LockTime::ZERO, sequence: Sequence(0xFFFF_FFFE) });
    }

    let _ = secp;
    println!("SIGNED TRANSACTIONS ONLY — container = {CAP_CHUNKS} chunks / 2904 B; one plate per string today (F-225), ~21 min/plate\n");
    for c in &cases { sweep(c); }
    println!("\n\nPLATE COST OF A SWEEP (1 output) — one plate per string today, ~21 min/plate\n");
    println!("{:<26} {:>18} {:>18} {:>18}   {:>10}", "wallet / spend path", "1 input", "2 inputs", "5 inputs", "max inputs");
    for c in &cases {
        let cell = |n_in: usize| -> String {
            match signed_len(c, n_in, 1) {
                Some((b, _)) => { let ch = chunks(b);
                    if ch > CAP_CHUNKS { format!("{b}B OVER") }
                    else { format!("{b}B {ch}pl {:.1}h", (ch as f64 * 21.0) / 60.0) } }
                None => "-".into(),
            }
        };
        let mut best = 1usize;
        while best < 400 {
            match signed_len(c, best + 1, 1) {
                Some((n, _)) if chunks(n) <= CAP_CHUNKS => best += 1,
                _ => break,
            }
        }
        println!("{:<26} {:>18} {:>18} {:>18}   {:>10}", c.name, cell(1), cell(2), cell(5), best);
    }

    println!("\nBARE PSBT for comparison — skeleton only, descriptor metadata re-derived from md1.");
    println!("Depends on scriptPubKey length, NOT on the policy, so these two lines cover every wallet above.");
    bare_sweep("bare PSBT (wpkh, 22B spk)", 22);
    bare_sweep("bare PSBT (tr/wsh, 34B spk)", 34);
}
