// REVIEW SCRATCH HARNESS (fable r0, lens 5): drives Liana v8.0's own
// `LianaDescriptor::from_str` over descriptors read from stdin (JSONL:
// {"name","variant","desc"}), prints one JSON line per input with the
// verbatim error or the inferred policy + first three receive/change
// addresses; `unspendable` rewrites a raw-NUMS tr() with Liana's own
// unspendable xpub; `sign` signs a PSBT with Liana's HotSigner.
use liana::descriptors::{LianaDescriptor, PathInfo};
use liana::miniscript::bitcoin::{self, bip32, hashes::{sha256, Hash}, secp256k1, Network};
use liana::miniscript::descriptor::{self, DescriptorPublicKey};
use liana::miniscript::{Descriptor, ForEachKey};
use liana::signer::HotSigner;
use std::io::{self, BufRead, Read};
use std::str::FromStr;

fn path_json(p: &PathInfo) -> serde_json::Value {
    match p {
        PathInfo::Single(k) => serde_json::json!({"kind":"single","keys":[k.to_string()]}),
        PathInfo::Multi(k, keys) => serde_json::json!({"kind":"multi","k":k,"keys":keys.iter().map(|k| k.to_string()).collect::<Vec<_>>()}),
    }
}

fn network_of(desc: &str) -> Network {
    if desc.contains("tpub") || desc.contains("tprv") { Network::Regtest } else { Network::Bitcoin }
}

fn parse_cmd() {
    let secp = secp256k1::Secp256k1::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.trim().is_empty() { continue; }
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        let name = v["name"].as_str().unwrap_or("").to_string();
        let variant = v["variant"].as_str().unwrap_or("").to_string();
        let desc = v["desc"].as_str().unwrap().to_string();
        let net = network_of(&desc);
        let out = match LianaDescriptor::from_str(&desc) {
            Err(e) => serde_json::json!({"name":name,"variant":variant,"ok":false,"error":e.to_string()}),
            Ok(d) => {
                let pol = d.policy();
                let recs: Vec<serde_json::Value> = pol.recovery_paths().iter().map(|(tl, p)| serde_json::json!({"older":tl,"path":path_json(p)})).collect();
                let mut recv = vec![]; let mut chg = vec![];
                for i in 0..3u32 {
                    recv.push(d.receive_descriptor().derive(bip32::ChildNumber::from_normal_idx(i).unwrap(), &secp).address(net).to_string());
                    chg.push(d.change_descriptor().derive(bip32::ChildNumber::from_normal_idx(i).unwrap(), &secp).address(net).to_string());
                }
                serde_json::json!({"name":name,"variant":variant,"ok":true,
                    "is_taproot": d.is_taproot(),
                    "primary": path_json(pol.primary_path()),
                    "recovery": recs,
                    "receive": recv, "change": chg,
                    "liana_desc": d.to_string(),
                    "receive_desc": d.receive_descriptor().to_string()})
            }
        };
        println!("{}", out);
    }
}

// Liana's own recipe (analysis.rs:398-430): pubkey = BIP-341 NUMS, chaincode =
// sha256 of the concatenation of every leaf xpub's pubkey in tap-tree order.
fn unspendable_cmd() {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s).unwrap();
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let desc = Descriptor::<DescriptorPublicKey>::from_str(line).expect("parse");
        let tr = match &desc { Descriptor::Tr(t) => t, _ => panic!("not tr") };
        let tree = tr.tap_tree().as_ref().expect("tree");
        let mut concat = Vec::new();
        let mut network = None;
        for (_, ms) in tree.iter() {
            ms.for_each_key(|pk| {
                if let DescriptorPublicKey::MultiXPub(x) = pk {
                    concat.extend_from_slice(&x.xkey.public_key.serialize());
                    network.get_or_insert(x.xkey.network);
                } else { panic!("leaf key is not multixpub: {}", pk); }
                true
            });
        }
        let chain_code = bip32::ChainCode::from(sha256::Hash::hash(&concat).as_ref());
        let xkey = bip32::Xpub {
            public_key: secp256k1::PublicKey::from_str("0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0").unwrap(),
            chain_code, depth: 0, parent_fingerprint: [0; 4].into(), child_number: 0.into(),
            network: network.unwrap(),
        };
        let ik = DescriptorPublicKey::MultiXPub(descriptor::DescriptorMultiXKey {
            origin: None, xkey,
            derivation_paths: descriptor::DerivPaths::new(vec![[0.into()][..].into(), [1.into()][..].into()]).unwrap(),
            wildcard: descriptor::Wildcard::Unhardened,
        });
        let new_tr = descriptor::Tr::new(ik, Some(tree.clone())).expect("tr");
        println!("{}", Descriptor::Tr(new_tr));
    }
}

fn sign_cmd(args: &[String]) {
    // sign <network> <mnemonic words...> ; PSBT base64 on stdin
    let net = Network::from_str(&args[0]).unwrap();
    let words = args[1..].join(" ");
    let signer = HotSigner::from_str(net, &words).expect("mnemonic");
    let secp = secp256k1::Secp256k1::new();
    let mut s = String::new();
    io::stdin().read_to_string(&mut s).unwrap();
    let psbt = bitcoin::Psbt::from_str(s.trim()).expect("psbt");
    match signer.sign_psbt(psbt, &secp) {
        Ok(p) => println!("{}", p),
        Err(e) => { eprintln!("sign error: {}", e); std::process::exit(2); }
    }
}

// The GUI's signature accounting (gui/src/app/view/psbt.rs:551 uses
// threshold - sigs_count): print Liana's partial_spend_info for a PSBT.
fn spendinfo_cmd(args: &[String]) {
    let desc = LianaDescriptor::from_str(&args[0]).expect("desc");
    let mut s = String::new();
    io::stdin().read_to_string(&mut s).unwrap();
    let psbt = bitcoin::Psbt::from_str(s.trim()).expect("psbt");
    match desc.partial_spend_info(&psbt) {
        Ok(info) => {
            let p = info.primary_path();
            let recs: Vec<serde_json::Value> = info.recovery_paths().iter().map(|(tl, i)| serde_json::json!({"older":tl,"threshold":i.threshold,"sigs_count":i.sigs_count})).collect();
            println!("{}", serde_json::json!({"primary":{"threshold":p.threshold,"sigs_count":p.sigs_count,"signed":p.signed_pubkeys.iter().map(|(k,v)| (k.to_string(), *v)).collect::<Vec<_>>()},"recovery":recs}));
        }
        Err(e) => { println!("{}", serde_json::json!({"error": e.to_string()})); }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("parse") => parse_cmd(),
        Some("unspendable") => unspendable_cmd(),
        Some("sign") => sign_cmd(&args[2..]),
        Some("spendinfo") => spendinfo_cmd(&args[2..]),
        _ => { eprintln!("usage: fableliana parse|unspendable|sign <network> <words...>"); std::process::exit(1); }
    }
}
