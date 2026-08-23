//! psbtfinal.rs -- what the COMPLIANT envelope costs.
//!
//! Derived from rcw.rs: same fixture, same wallets, same forced spending paths.
//! The only difference is what it measures. Answers the R0 round-0 T-3 question
//! -- if `ur:bytes` is off-label per BCR-2020-005 and the registry has no raw
//! transaction type, how many bytes does moving to `ur:psbt` actually cost?
//!
//! Reports per artifact: raw signed tx, finalized PSBT as BIP-174 leaves it,
//! and finalized PSBT with UTXO records stripped -- each with the overhead over
//! raw. Asserts the lean form re-parses and extracts to a byte-identical
//! transaction, so the cheap option is proven rather than assumed.

//! RCW sizing. Three parts:
//!   1. Can the fixture's committed preimages satisfy its own hashlocks?
//!   2. Does stock rust-miniscript accept the RCW as written (keyless tier 4)?
//!   3. Sizing, with tier 4 keyed = after AND hashlock AND signature.

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

/// The REAL 32-byte witness preimage for a tier, read from the fixture's
/// `preimage-N.hex`. The policy commits to sha256 of this value.
fn preimage(tier: usize) -> [u8; 32] {
    let hex = std::fs::read_to_string(format!("{RCW}/preimages/preimage-{tier}.hex"))
        .expect("preimage hex file");
    let hex = hex.trim();
    assert_eq!(hex.len(), 64, "preimage-{tier} is not 32 bytes");
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).expect("hex");
    }
    // And it must be sha256 of the committed passphrase, with no trailing newline.
    let phrase = std::fs::read(format!("{RCW}/preimages/preimage-{tier}.txt")).expect("phrase");
    assert_eq!(sha256::Hash::hash(&phrase).to_byte_array(), out,
               "preimage-{tier}.hex is not sha256(preimage-{tier}.txt)");
    out
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

fn dummy_outpoint(i: u32) -> OutPoint {
    let mut b = [0u8; 32];
    b[0] = 0xc3; b[31] = i as u8;
    OutPoint { txid: Txid::from_raw_hash(bitcoin::hashes::sha256d::Hash::from_byte_array(b)), vout: i }
}

struct Built { desc: String, masters: Vec<Xpriv> }

/// `keyed_t4`: rewrite tier 4 as after AND hashlock AND signature, adding a 7th key.
fn build(form: &str, branch: Option<u32>) -> Built {
    let secp = Secp256k1::new();
    let (keys_dir, policy_file) = if form == "tr" {
        ("keys-tr", "policy-tr.txt")
    } else {
        ("keys-wsh", "policy-wsh.txt")
    };
    let mut out = std::fs::read_to_string(format!("{RCW}/{policy_file}")).unwrap().trim().to_string();
    let mut masters = Vec::new();
    let mut _key_path = String::new();

    for n in 0..7u32 {
        let meta = std::fs::read_to_string(format!("{RCW}/{keys_dir}/key-{n}.xpub")).unwrap();
        let origin = meta.lines()
            .find_map(|l| l.split_once("origin [").map(|(_, r)| r.trim_end_matches(']').to_string()))
            .expect("origin line");
        let (fp_str, path_str) = origin.split_once('/').unwrap();
        _key_path = path_str.to_string();
        let want_xpub = meta.lines().find(|l| l.starts_with("xpub")).unwrap().trim();

        let words = std::fs::read_to_string(format!("{RCW}/seeds/key-{n}.seed")).unwrap();
        let mn = bip39::Mnemonic::parse(words.trim()).expect("mnemonic");
        let master = Xpriv::new_master(Network::Bitcoin, &mn.to_seed("")).unwrap();
        assert_eq!(master.fingerprint(&secp).to_string(), fp_str, "@{n} fingerprint");
        let child = master.derive_priv(&secp, &DerivationPath::from_str(path_str).unwrap()).unwrap();
        assert_eq!(bitcoin::bip32::Xpub::from_priv(&secp, &child).to_string(), want_xpub,
                   "@{n} derived xpub disagrees with the fixture");
        masters.push(master);

        let needle = format!("@{n}/{path_str}");
        assert!(out.contains(&needle), "placeholder {needle} not found");
        out = out.replace(&needle, &format!("[{origin}]{child}"));
    }
    assert!(!out.contains('@'), "unsubstituted placeholder remains");

    if let Some(b) = branch {
        out = out.replace("<0;1>", &b.to_string());
        assert!(!out.contains("<0;1>"), "multipath marker survived");
    }
    Built { desc: out, masters }
}

struct Scenario {
    name: &'static str,
    locktime: LockTime,
    sequence: Sequence,
    preimages: &'static [usize],
    /// key indices withheld, to force the finalizer onto the intended branch
    withhold: &'static [usize],
}

fn run(form: &str, sc: &Scenario, n_in: usize, n_out: usize) {
    let secp = Secp256k1::new();
    let b0 = build(form, Some(0));
    let b1 = build(form, Some(1));
    let masters = Masters(
        b0.masters.iter().enumerate()
            .filter(|(i, _)| !sc.withhold.contains(i))
            .map(|(_, m)| *m).collect(),
    );
    let (recv, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &b0.desc).unwrap();
    let (change, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &b1.desc).unwrap();
    let ins: Vec<_> = (0..n_in as u32).map(|i| recv.at_derivation_index(i).unwrap()).collect();
    let change_def = change.at_derivation_index(0).unwrap();

    let external = ScriptBuf::from_hex(
        "5120a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90").unwrap();
    let unsigned = Transaction {
        version: Version::TWO,
        lock_time: sc.locktime,
        input: (0..n_in).map(|i| TxIn {
            previous_output: dummy_outpoint(i as u32),
            script_sig: ScriptBuf::new(),
            sequence: sc.sequence,
            witness: Witness::new(),
        }).collect(),
        output: {
            let spendable = (n_in as u64) * 100_000 - 1_000;
            let first = if n_out == 2 { spendable / 2 } else { spendable };
            let mut o = vec![TxOut { value: Amount::from_sat(first), script_pubkey: external }];
            if n_out == 2 {
                o.push(TxOut { value: Amount::from_sat(spendable - first),
                               script_pubkey: change_def.script_pubkey() });
            }
            o
        },
    };

    let mut psbt = Psbt::from_unsigned_tx(unsigned).unwrap();
    for (i, d) in ins.iter().enumerate() {
        psbt.inputs[i].witness_utxo =
            Some(TxOut { value: Amount::from_sat(100_000), script_pubkey: d.script_pubkey() });
    }
    let bare = psbt.serialize().len();
    for (i, d) in ins.iter().enumerate() {
        psbt.update_input_with_descriptor(i, d).unwrap();
        for &t in sc.preimages {
            let p = preimage(t);
            psbt.inputs[i].sha256_preimages.insert(sha256::Hash::hash(&p), p.to_vec());
        }
    }
    if n_out == 2 { psbt.update_output_with_descriptor(1, &change_def).unwrap(); }
    let unsigned_len = psbt.serialize().len();

    psbt.sign(&masters, &secp).ok();
    if let Err(e) = psbt.finalize_mut(&secp) {
        println!("{form:<4} {n_in}in/{n_out}out {:<20} FINALIZE FAILED: {}", sc.name,
                 e.iter().map(|x| format!("{x:?}")).next().unwrap_or_default());
        return;
    }
    // ---- THE MEASUREMENT THIS BINARY EXISTS FOR -------------------------
    // The spec's engraved payload is a raw signed transaction. BCR-2020-005
    // forbids `ur:bytes` outside testing, and the BCR-2020-006 registry has no
    // type for a raw transaction -- `psbt` is the only transaction-shaped one.
    // So the compliant envelope is a FULLY FINALIZED PSBT, from which the raw
    // signed transaction is extracted mechanically. This measures what that
    // wrapper costs, in the two forms it can take.

    // (a) as BIP-174's finalizer leaves it: final scriptSig/scriptWitness per
    //     input, with the UTXO record RETAINED (BIP-174 keeps it).
    let psbt_bytes = psbt.serialize();
    let fin_full = psbt_bytes.len();

    // (b) lean: UTXO records stripped. An Extractor needs only the unsigned
    //     transaction plus each input's finalized unlocking script, so this is
    //     still a valid PSBT and still extracts -- asserted below, not assumed.
    // (c) MINIMAL COMPLIANT: keep each input's UTXO record, so the standard
    //     `extract_tx()` fee check still passes, but clear the OUTPUT maps.
    //     Output descriptor metadata (change derivations, taproot leaf scripts)
    //     is what an *updater* put there for a *signer*; an extractor and a
    //     broadcaster need none of it, and BIP-174's finalizer only strips
    //     INPUT fields, so it survives into the engraved payload unless we say
    //     otherwise. On 2-output artifacts this is the whole blowup.
    let mut min = psbt.clone();
    for o in min.outputs.iter_mut() { *o = Default::default(); }
    let psbt_min_bytes = min.serialize();
    let fin_min = psbt_min_bytes.len();
    let min_reparsed = Psbt::deserialize(&psbt_min_bytes).expect("min PSBT re-parses");
    let min_ok = min_reparsed.clone().extract_tx().is_ok();
    let mut min_raw = Vec::new();
    min_reparsed.extract_tx_unchecked_fee_rate().consensus_encode(&mut min_raw).unwrap();

    let mut lean = psbt.clone();
    for i in lean.inputs.iter_mut() { i.witness_utxo = None; i.non_witness_utxo = None; }
    let fin_lean = lean.serialize().len();

    let tx = match psbt.extract_tx() { Ok(t) => t, Err(_) => { println!("{form} extract failed"); return } };
    let mut raw = Vec::new();
    tx.consensus_encode(&mut raw).unwrap();

    // Does the lean form survive a round trip through ordinary tooling?
    // MEASURED, not assumed -- and the first version of this probe assumed
    // wrong. rust-bitcoin's `extract_tx()` runs a FEE CHECK, which needs each
    // input's value, so a PSBT with its UTXO records stripped fails with
    // MissingInputValue. The bytes are all there; the safe API refuses to hand
    // them over. Only the explicitly-unchecked entry point extracts it.
    let reparsed = Psbt::deserialize(&lean.serialize()).expect("lean PSBT re-parses");
    let lean_checked_ok = reparsed.clone().extract_tx().is_ok();
    let lean_tx = reparsed.extract_tx_unchecked_fee_rate();
    let mut lean_raw = Vec::new();
    lean_tx.consensus_encode(&mut lean_raw).unwrap();
    assert_eq!(lean_raw, raw, "lean PSBT extracted a DIFFERENT transaction");
    assert_eq!(min_raw, raw, "minimal PSBT extracted a DIFFERENT transaction");

    // And the full form, which is what BIP-174's finalizer actually leaves.
    let full_reparsed = Psbt::deserialize(&psbt_bytes).expect("full PSBT re-parses");
    let full_ok = full_reparsed.extract_tx().is_ok();

    let shape: Vec<usize> = tx.input[0].witness.iter().map(|e| e.len()).collect();
    let ch = |n: usize| (n * 8).div_ceil(363);
    let mk = |n: usize| if ch(n) <= 64 { "fits" } else { "OVER" };
    let _ = (bare, unsigned_len, mk(bare), tx.vsize());
    let ovh_full = fin_full as i64 - raw.len() as i64;
    let ovh_lean = fin_lean as i64 - raw.len() as i64;
    let ovh_min = fin_min as i64 - raw.len() as i64;
    println!(
        "{form:<4} {n_in}in/{n_out}out {:<20} | RAW {:>5}B | full {fin_full:>5}B(+{ovh_full:>4}) ex:{} | MIN {fin_min:>5}B(+{ovh_min:>3},{:>5.1}%) ex:{} | lean {fin_lean:>5}B(+{ovh_lean:>4}) ex:{} | all extract==raw",
        sc.name, raw.len(),
        if full_ok { "y" } else { "n" },
        100.0 * ovh_min as f64 / raw.len() as f64,
        if min_ok { "y" } else { "n" },
        if lean_checked_ok { "y" } else { "n" });
    let _ = shape;
}

fn main() {
    let secp = Secp256k1::new();

    println!("== 1. do the fixture's preimages satisfy its hashlocks? ==");
    for tier in 0..3usize {
        let phrase = std::fs::read(format!("{RCW}/preimages/preimage-{tier}.txt")).unwrap();
        let pre = preimage(tier);
        let literal = sha256::Hash::hash(&pre);
        let in_policy = std::fs::read_to_string(format!("{RCW}/policy-tr.txt"))
            .unwrap()
            .contains(&format!("sha256({literal})"));
        println!(
            "   tier {}: phrase {:>2} B -> preimage 32 B: {} | literal in policy: {}",
            tier + 1, phrase.len(), pre.len() == 32, in_policy
        );
    }
    let b = build("tr", Some(0));
    let (d, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &b.desc).unwrap();
    if let Descriptor::Tr(ref tr) = d.at_derivation_index(0).unwrap() {
        for leaf in tr.leaves() {
            let sc = leaf.compute_script();
            let head: String = sc.to_asm_string().split_whitespace().take(6).collect::<Vec<_>>().join(" ");
            println!("   tr leaf depth {} ({:>3} B script): {head} ...", leaf.depth(), sc.len());
        }
    }

    println!("\n== 2. does stock rust-miniscript 13.1 accept the STORED RCW (tier 4 keyed)? ==");
    for form in ["tr", "wsh"] {
        let b = build(form, Some(0));
        match Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &b.desc) {
            Ok(_) => println!("   {form:<4} ACCEPTED"),
            Err(e) => println!("   {form:<4} REFUSED: {e}"),
        }
    }

    println!("\n== 3. sizing, STORED fixture: tier 4 = after AND hashlock AND signature, 7 keys ==");
    let scenarios = [
        Scenario { name: "tier1 3of3+hash", locktime: LockTime::ZERO,
                   sequence: Sequence(0xFFFF_FFFE), preimages: &[0], withhold: &[] },
        Scenario { name: "tier3 pk+after", locktime: LockTime::from_height(1_200_000).unwrap(),
                   sequence: Sequence(0xFFFF_FFFE), preimages: &[], withhold: &[] },
        Scenario { name: "tier4 keyed", locktime: LockTime::from_height(1_400_000).unwrap(),
                   sequence: Sequence(0xFFFF_FFFE), preimages: &[2], withhold: &[5] },
    ];
    for sc in &scenarios {
        for (n_in, n_out) in [(1usize, 1usize), (1, 2), (5, 2)] {
            run("tr", sc, n_in, n_out);
            run("wsh", sc, n_in, n_out);
        }
        println!();
    }
}
