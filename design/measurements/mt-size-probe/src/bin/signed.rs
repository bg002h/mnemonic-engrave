//! Produce a REAL, fully-satisfied 5-in/2-out transaction from the pathological
//! wallet (tr and wsh forms) and measure it. No estimates: every number below is
//! the length of bytes rust-bitcoin actually serialised.
//!
//! The fixture's sha256 literal has no committed preimage, so the hashlock tiers
//! are measured against a SIZE-EQUIVALENT descriptor: the same policy with the
//! 32-byte hash literal swapped for sha256(known 32-byte preimage). Script size
//! and witness size are byte-identical either way.

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

const PREIMAGE: [u8; 32] = [0x42u8; 32];

// (placeholder index, master letter, account index) — from the fixture's own
// key-NN.xpub comment headers.
const MAP: [(u32, char, u32); 11] = [
    (0, 'A', 0), (1, 'A', 1), (2, 'A', 2), (3, 'A', 3),
    (4, 'B', 0), (5, 'B', 1), (6, 'B', 2), (7, 'B', 3),
    (8, 'C', 0), (9, 'C', 1), (10, 'C', 2),
];

fn seed_of(letter: char) -> &'static str {
    match letter {
        'A' => "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        'B' => "legal winner thank year wave sausage worth useful legal winner thank yellow",
        'C' => "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
        _ => unreachable!(),
    }
}

fn dummy_outpoint(i: u32) -> OutPoint {
    let mut b = [0u8; 32];
    b[0] = 0xa1;
    b[31] = i as u8;
    OutPoint { txid: Txid::from_raw_hash(bitcoin::hashes::sha256d::Hash::from_byte_array(b)), vout: i }
}

/// Build the descriptor string with real xprvs substituted for @N, and the
/// hashlock re-pointed at a preimage we hold.
fn build(policy: &str) -> String {
    let secp = Secp256k1::new();
    let mut out = policy.trim().to_string();

    for (n, letter, acct) in MAP {
        let mn = bip39::Mnemonic::parse(seed_of(letter)).expect("mnemonic");
        let master = Xpriv::new_master(Network::Bitcoin, &mn.to_seed("")).expect("master");
        let fp = master.fingerprint(&secp);
        let path = DerivationPath::from_str(&format!("48'/0'/{acct}'/2'")).unwrap();
        let child = master.derive_priv(&secp, &path).expect("derive");

        // Self-check: the xpub we derive must equal the one committed in the fixture.
        let want = std::fs::read_to_string(format!(
            "/scratch/code/shibboleth/mnemonic-engrave/design/journeys/inputs-pathological/keys/key-{n:02}.xpub"
        ))
        .expect("fixture xpub");
        let want_xpub = want.lines().find(|l| l.starts_with("xpub")).unwrap().trim();
        let got_xpub = bitcoin::bip32::Xpub::from_priv(&secp, &child).to_string();
        assert_eq!(got_xpub, want_xpub, "@{n} derivation disagrees with the fixture");
        assert!(want.contains(&format!("[{fp}/48'/0'/{acct}'/2']")), "@{n} fingerprint/origin mismatch");

        let needle = format!("@{n}/48'/0'/{acct}'/2'");
        let repl = format!("[{fp}/48'/0'/{acct}'/2']{child}");
        assert!(out.contains(&needle), "placeholder {needle} not found");
        out = out.replace(&needle, &repl);
    }
    assert!(!out.contains('@'), "unsubstituted placeholder remains");

    let h = sha256::Hash::hash(&PREIMAGE);
    let old = "a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad";
    assert!(out.contains(old), "hash literal not found");
    assert_eq!(old.len(), h.to_string().len(), "hash swap must be size-neutral");
    out.replace(old, &h.to_string())
}

/// `GetKey` over the three master xprvs. `Xpriv` implements neither `Ord` nor
/// `Hash` in bitcoin 0.32.101, so the crate's `BTreeSet`/`HashSet` impls are
/// unusable here; this is the same first-match-wins logic over a Vec.
struct Masters(Vec<Xpriv>);

impl bitcoin::psbt::GetKey for Masters {
    type Error = bitcoin::psbt::GetKeyError;
    fn get_key<C: bitcoin::secp256k1::Signing>(
        &self,
        req: bitcoin::psbt::KeyRequest,
        secp: &Secp256k1<C>,
    ) -> Result<Option<bitcoin::PrivateKey>, Self::Error> {
        self.0
            .iter()
            .find_map(|x| x.get_key(req.clone(), secp).transpose())
            .transpose()
    }
}

struct Scenario {
    name: &'static str,
    locktime: LockTime,
    sequence: Sequence,
}

fn run(label: &str, policy: &str, sc: &Scenario, n_in: usize, n_out: usize) {
    let secp = Secp256k1::new();
    let desc_str = build(policy);
    let (desc, _keymap) =
        Descriptor::<DescriptorPublicKey>::parse_descriptor(&secp, &desc_str).expect("parse");
    let masters = Masters(
        ['A', 'B', 'C']
            .iter()
            .map(|l| {
                let mn = bip39::Mnemonic::parse(seed_of(*l)).unwrap();
                Xpriv::new_master(Network::Bitcoin, &mn.to_seed("")).unwrap()
            })
            .collect(),
    );

    let singles = desc.into_single_descriptors().expect("singles");
    let (recv, change) = (&singles[0], &singles[1]);
    let ins: Vec<_> = (0..n_in as u32).map(|i| recv.at_derivation_index(i).unwrap()).collect();
    let change_def = change.at_derivation_index(0).unwrap();

    let external = ScriptBuf::from_hex(
        "5120a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90",
    )
    .unwrap();
    let unsigned = Transaction {
        version: Version::TWO,
        lock_time: sc.locktime,
        input: (0..n_in)
            .map(|i| TxIn {
                previous_output: dummy_outpoint(i as u32),
                script_sig: ScriptBuf::new(),
                sequence: sc.sequence,
                witness: Witness::new(),
            })
            .collect(),
        output: {
            let spendable = (n_in as u64) * 100_000 - 1_000; // 1000 sat fee
            let first = if n_out == 2 { spendable / 2 } else { spendable };
            let mut o = vec![TxOut { value: Amount::from_sat(first), script_pubkey: external }];
            if n_out == 2 {
                o.push(TxOut {
                    value: Amount::from_sat(spendable - first),
                    script_pubkey: change_def.script_pubkey(),
                });
            }
            o
        },
    };
    let stripped = { let mut v = Vec::new(); unsigned.consensus_encode(&mut v).unwrap(); v.len() };

    let mut psbt = Psbt::from_unsigned_tx(unsigned).expect("psbt");
    for (i, d) in ins.iter().enumerate() {
        psbt.inputs[i].witness_utxo =
            Some(TxOut { value: Amount::from_sat(100_000), script_pubkey: d.script_pubkey() });
    }
    let bare_len = psbt.serialize().len();
    for (i, d) in ins.iter().enumerate() {
        psbt.update_input_with_descriptor(i, d).expect("update input");
        psbt.inputs[i].sha256_preimages.insert(sha256::Hash::hash(&PREIMAGE), PREIMAGE.to_vec());
    }
    if n_out == 2 {
        psbt.update_output_with_descriptor(1, &change_def).expect("update output");
    }
    let unsigned_psbt_len = psbt.serialize().len();

    let signed_keys = psbt.sign(&masters, &secp).unwrap_or_else(|(ok, err)| {
        panic!("sign: ok={ok:?} err={err:?}")
    });
    let n_sigs = signed_keys.len();
    let signed_psbt_len = psbt.serialize().len();

    if let Err(e) = psbt.finalize_mut(&secp) {
        println!("{label:<4} {n_in}in/{n_out}out {:<24} FINALIZE FAILED after {n_sigs} sigs: {}", sc.name,
            e.iter().map(|x| format!("{x:?}")).next().unwrap_or_default());
        return;
    }
    let tx = match psbt.extract_tx() { Ok(t) => t, Err(e) => { println!("{label} {n_in}in/{n_out}out extract FAILED"); let _ = e; return; } };
    let mut raw = Vec::new();
    tx.consensus_encode(&mut raw).unwrap();

    let witness_bytes: usize = tx.input.iter().map(|i| i.witness.size()).sum();
    let ch = |n: usize| (n * 8).div_ceil(363);
    let mark = |n: usize| if ch(n) <= 64 { "fits" } else { "OVER" };
    println!(
        "{label:<4} {n_in}in/{n_out}out {:<24} | bare {bare_len:>5}B({:>3}ch,{}) | full-unsigned {unsigned_psbt_len:>6}B({:>3}ch,{}) | SIGNED TX {:>5}B({:>3}ch,{}) vsize {}",
        sc.name,
        ch(bare_len), mark(bare_len),
        ch(unsigned_psbt_len), mark(unsigned_psbt_len),
        raw.len(), ch(raw.len()), mark(raw.len()),
        tx.vsize()
    );
    let _ = (stripped, signed_psbt_len, witness_bytes, n_sigs);
}

fn main() {
    let dir = "/scratch/code/shibboleth/mnemonic-engrave/design/journeys/inputs-pathological";
    let tr = std::fs::read_to_string(format!("{dir}/wallet-policy-tr.txt")).unwrap();
    let wsh = std::fs::read_to_string(format!("{dir}/wallet-policy.txt")).unwrap();

    let scenarios = [
        Scenario {
            name: "tier1 (3-of-3 + hash)",
            locktime: LockTime::from_height(1_900_000).unwrap(),
            sequence: Sequence(0xFFFF_FFFE),
        },
        Scenario {
            name: "tier4 (1-of-3, cheapest)",
            locktime: LockTime::ZERO,
            sequence: Sequence(0x0040_F0DA),
        },
    ];
    for sc in &scenarios {
        for (n_in, n_out) in [(1usize, 1usize), (1, 2), (2, 2), (5, 2), (10, 2)] {
            run("tr", &tr, sc, n_in, n_out);
            run("wsh", &wsh, sc, n_in, n_out);
        }
        println!();
    }
}
