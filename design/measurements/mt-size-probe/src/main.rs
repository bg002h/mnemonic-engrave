use bitcoin::consensus::Encodable;
use bitcoin::psbt::Psbt;
use bitcoin::{
    absolute::LockTime, transaction::Version, Amount, OutPoint, ScriptBuf, Sequence, Transaction,
    TxIn, TxOut, Txid, Witness,
};
use miniscript::psbt::PsbtExt;
use miniscript::{Descriptor, DescriptorPublicKey};
use bitcoin::hashes::Hash as _;

const N_IN: usize = 5;

fn dummy_outpoint(i: u32) -> OutPoint {
    let mut b = [0u8; 32];
    b[0] = 0xa1;
    b[31] = i as u8;
    OutPoint { txid: Txid::from_raw_hash(bitcoin::hashes::sha256d::Hash::from_byte_array(b)), vout: i }
}

fn probe(label: &str, desc_str: &str) {
    let (desc, _) = Descriptor::<DescriptorPublicKey>::parse_descriptor(
        &bitcoin::secp256k1::Secp256k1::new(),
        desc_str,
    )
    .expect("parse descriptor");

    // multipath <0;1> -> [receive, change]
    let singles = desc.clone().into_single_descriptors().expect("single descriptors");
    let recv = &singles[0];
    let change = &singles[1];

    // Definite descriptors for 5 receive inputs (indices 0..5) and one change output.
    let ins: Vec<_> = (0..N_IN as u32)
        .map(|i| recv.at_derivation_index(i).expect("derive"))
        .collect();
    let change_def = change.at_derivation_index(0).expect("derive change");

    let spks: Vec<ScriptBuf> = ins.iter().map(|d| d.script_pubkey()).collect();

    // 2 outputs: one external P2TR-ish payment, one change back to the wallet.
    let external = ScriptBuf::from_hex(
        "5120a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90",
    )
    .unwrap();
    let unsigned = Transaction {
        version: Version::TWO,
        lock_time: LockTime::from_height(1_900_000).unwrap(),
        input: (0..N_IN)
            .map(|i| TxIn {
                previous_output: dummy_outpoint(i as u32),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_LOCKTIME_NO_RBF,
                witness: Witness::new(),
            })
            .collect(),
        output: vec![
            TxOut { value: Amount::from_sat(400_000), script_pubkey: external },
            TxOut { value: Amount::from_sat(99_000), script_pubkey: change_def.script_pubkey() },
        ],
    };
    let base_tx_bytes = {
        let mut v = Vec::new();
        unsigned.consensus_encode(&mut v).unwrap();
        v.len()
    };

    let mut psbt = Psbt::from_unsigned_tx(unsigned).expect("psbt");
    for (i, spk) in spks.iter().enumerate() {
        psbt.inputs[i].witness_utxo =
            Some(TxOut { value: Amount::from_sat(100_000), script_pubkey: spk.clone() });
    }
    let bare_len = psbt.serialize().len();

    for (i, d) in ins.iter().enumerate() {
        psbt.update_input_with_descriptor(i, d).expect("update input");
    }
    let after_inputs = psbt.serialize().len();

    psbt.update_output_with_descriptor(1, &change_def).expect("update output");
    let full = psbt.serialize();
    let full_len = full.len();

    // Worst-case satisfied witness weight, per input, from miniscript itself.
    let wu = ins[0].max_weight_to_satisfy().expect("max weight");

    // Signed tx size: base (non-witness) + 2 marker/flag + N * witness bytes.
    let nonwitness = base_tx_bytes; // no witness present yet, so this is the stripped size
    let witness_bytes_per_in = wu.to_wu() as usize; // max_weight_to_satisfy returns witness-unit weight of the witness field
    let signed_total = nonwitness + 2 + witness_bytes_per_in * N_IN;

    println!("=========== {label} ===========");
    println!("descriptor chars                 : {}", desc_str.trim().len());
    println!("unsigned tx (raw, no witness)    : {base_tx_bytes} B");
    println!("PSBT, utxo only (no metadata)    : {bare_len} B");
    println!("PSBT, + input descriptor metadata: {after_inputs} B");
    println!("PSBT, + change output metadata   : {full_len} B   <-- UNSIGNED PSBT mt would produce");
    println!("max witness weight / input       : {} WU", wu.to_wu());
    println!("signed tx, worst-case path       : ~{signed_total} B  <-- SIGNED tx mt would encode");
    println!();
    let cap = 2904usize;
    for (what, n) in [("unsigned PSBT", full_len), ("signed tx", signed_total)] {
        let chunks = (n * 8).div_ceil(363);
        println!(
            "  {what:<14} {n:>6} B -> {chunks:>4} codex32 chunks ({} chars){}",
            chunks * 97,
            if n <= cap { "  FITS in 64 chunks" } else { "  EXCEEDS the 64-chunk ceiling" }
        );
    }
    println!();
}

fn main() {
    for (label, path) in [
        ("PATHOLOGICAL / tr  (taproot, 4 leaves, 11 keys)", "desc-tr.txt"),
        ("PATHOLOGICAL / wsh (or_i nest, 11 keys)", "desc-wsh.txt"),
    ] {
        let s = std::fs::read_to_string(path).expect("read descriptor");
        probe(label, s.trim());
    }
}
