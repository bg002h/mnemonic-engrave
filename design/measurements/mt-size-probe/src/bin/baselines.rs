//! Baseline wallet sizes — single-sig, 2-of-3 and 3-of-5 — measured the same way
//! as the complex wallets, so the comparison is apples to apples: real xprvs,
//! real signatures, real finalize, real serialised bytes.

use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::consensus::Encodable;
use bitcoin::hashes::Hash as _;
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
/// The constellation's NUMS pin — script-path only, no key-path spend.
const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

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
    b[0] = 0xb2; b[31] = i as u8;
    OutPoint { txid: Txid::from_raw_hash(bitcoin::hashes::sha256d::Hash::from_byte_array(b)), vout: i }
}

/// The n'th master xprv, from the RCW's committed test seeds.
fn master(n: usize) -> Xpriv {
    let words = std::fs::read_to_string(format!("{RCW}/seeds/key-{n}.seed")).expect("seed");
    let mn = bip39::Mnemonic::parse(words.trim()).expect("mnemonic");
    Xpriv::new_master(Network::Bitcoin, &mn.to_seed("")).unwrap()
}

/// `[fp/path]xprv/<branch>/*` for master n.
fn keyexpr(n: usize, path: &str, branch: u32) -> String {
    let secp = Secp256k1::new();
    let m = master(n);
    let child = m.derive_priv(&secp, &DerivationPath::from_str(path).unwrap()).unwrap();
    format!("[{}/{}]{}/{}/*", m.fingerprint(&secp), path, child, branch)
}

struct Wallet { name: &'static str, path: &'static str, n_keys: usize, build: fn(&[String]) -> String }

fn desc_of(w: &Wallet, branch: u32) -> String {
    let keys: Vec<String> = (0..w.n_keys).map(|i| keyexpr(i, w.path, branch)).collect();
    (w.build)(&keys)
}

fn run(w: &Wallet, n_in: usize, n_out: usize) {
    let secp = Secp256k1::new();
    let masters = Masters((0..w.n_keys).map(master).collect());
    let (recv, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &desc_of(w, 0)).unwrap();
    let (change, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &desc_of(w, 1)).unwrap();
    let ins: Vec<_> = (0..n_in as u32).map(|i| recv.at_derivation_index(i).unwrap()).collect();
    let change_def = change.at_derivation_index(0).unwrap();

    let external = ScriptBuf::from_hex(
        "5120a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90").unwrap();
    let unsigned = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: (0..n_in).map(|i| TxIn {
            previous_output: dummy_outpoint(i as u32),
            script_sig: ScriptBuf::new(),
            sequence: Sequence(0xFFFF_FFFE),
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
    for (i, d) in ins.iter().enumerate() { psbt.update_input_with_descriptor(i, d).unwrap(); }
    if n_out == 2 { psbt.update_output_with_descriptor(1, &change_def).unwrap(); }
    let unsigned_len = psbt.serialize().len();

    psbt.sign(&masters, &secp).ok();
    if let Err(e) = psbt.finalize_mut(&secp) {
        println!("{:<22} {n_in}in/{n_out}out FINALIZE FAILED: {}", w.name,
                 e.iter().map(|x| format!("{x:?}")).next().unwrap_or_default());
        return;
    }
    let tx = psbt.extract_tx().expect("extract");
    let mut raw = Vec::new();
    tx.consensus_encode(&mut raw).unwrap();
    let ch = |n: usize| (n * 8).div_ceil(363);
    let mk = |n: usize| if ch(n) <= 64 { "fits" } else { "OVER" };
    println!(
        "{:<22} {n_in}in/{n_out}out | bare {bare:>4}B({:>2}ch) | full-unsigned {unsigned_len:>5}B({:>3}ch,{}) | SIGNED TX {:>5}B({:>3}ch,{}) vsize {}",
        w.name, ch(bare), ch(unsigned_len), mk(unsigned_len),
        raw.len(), ch(raw.len()), mk(raw.len()), tx.vsize());
}

fn main() {
    let wallets = [
        Wallet { name: "single-sig wpkh", path: "84'/0'/0'", n_keys: 1,
                 build: |k| format!("wpkh({})", k[0]) },
        Wallet { name: "single-sig tr (keypath)", path: "86'/0'/0'", n_keys: 1,
                 build: |k| format!("tr({})", k[0]) },
        Wallet { name: "2-of-3 wsh", path: "48'/0'/0'/2'", n_keys: 3,
                 build: |k| format!("wsh(multi(2,{},{},{}))", k[0], k[1], k[2]) },
        Wallet { name: "2-of-3 tr (1 leaf)", path: "48'/0'/0'/2'", n_keys: 3,
                 build: |k| format!("tr({NUMS},multi_a(2,{},{},{}))", k[0], k[1], k[2]) },
        Wallet { name: "3-of-5 wsh", path: "48'/0'/0'/2'", n_keys: 5,
                 build: |k| format!("wsh(multi(3,{},{},{},{},{}))", k[0], k[1], k[2], k[3], k[4]) },
        Wallet { name: "3-of-5 tr (1 leaf)", path: "48'/0'/0'/2'", n_keys: 5,
                 build: |k| format!("tr({NUMS},multi_a(3,{},{},{},{},{}))", k[0], k[1], k[2], k[3], k[4]) },
    ];
    for (n_in, n_out) in [(1usize, 1usize), (1, 2), (5, 2)] {
        for w in &wallets { run(w, n_in, n_out); }
        println!();
    }
}
