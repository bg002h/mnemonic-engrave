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
    let tx = match psbt.extract_tx() { Ok(t) => t, Err(_) => { println!("{form} extract failed"); return } };
    let mut raw = Vec::new();
    tx.consensus_encode(&mut raw).unwrap();
    let shape: Vec<usize> = tx.input[0].witness.iter().map(|e| e.len()).collect();
    let ch = |n: usize| (n * 8).div_ceil(CHUNK_PAYLOAD_BITS);
    let mk = |n: usize| if ch(n) <= 64 { "fits" } else { "OVER" };
    println!(
        "{form:<4} {n_in}in/{n_out}out {:<20} | bare {bare:>4}B({:>2}ch,{}) | full-unsigned {unsigned_len:>5}B({:>3}ch,{}) | SIGNED TX {:>5}B({:>3}ch,{}) vsize {:<5} wit{:?}",
        sc.name, ch(bare), mk(bare), ch(unsigned_len), mk(unsigned_len),
        raw.len(), ch(raw.len()), mk(raw.len()), tx.vsize(), shape);
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
