use std::collections::BTreeSet;
use std::str::FromStr;

use bitcoin::hashes::sha256;
use bitcoin::{absolute, relative};
use miniscript::descriptor::{DescriptorPublicKey, DefiniteDescriptorKey};
use miniscript::miniscript::satisfy::{Preimage32, Satisfier};
use miniscript::plan::Assets;
use miniscript::iter::TreeLike;
use miniscript::{Descriptor, Terminal, ToPublicKey};

const DESC: &str = include_str!("/scratch/code/shibboleth/mnemonic-toolkit/.examples-build/degrade2.desc");

// A satisfier that hands back a fixed worst-case *standard* (low-S) ECDSA
// signature for any key in `keys`, plus the sha256 preimage / timelocks.
struct Sat {
    keys: BTreeSet<bitcoin::PublicKey>,
    sha: bool,
    after: Option<absolute::LockTime>,
    older: Option<relative::LockTime>,
}

fn worst_case_sig() -> bitcoin::ecdsa::Signature {
    // Worst-case *standard* (low-S, BIP-146) ECDSA signature:
    //   r = 0xff 00..00 01  -> high bit set, DER pads a 0x00 => 33-byte INTEGER
    //   s = 0x7f 00..00 01  -> high bit clear and < n/2      => 32-byte INTEGER
    // DER = 30 <len> 02 21 00<r32> 02 20 <s32>  = 71 bytes, + 1 sighash byte = 72.
    let mut compact = [0u8; 64];
    compact[0] = 0xff;
    compact[31] = 0x01;
    compact[32] = 0x7f;
    compact[63] = 0x01;
    let sig = secp256k1::ecdsa::Signature::from_compact(&compact).expect("valid sig scalars");
    let s = bitcoin::ecdsa::Signature {
        signature: sig,
        sighash_type: bitcoin::sighash::EcdsaSighashType::All,
    };
    assert_eq!(s.serialize().len(), 72, "expected worst-case low-S sig to serialize to 72 bytes");
    s
}

impl Satisfier<DefiniteDescriptorKey> for Sat {
    fn lookup_ecdsa_sig(&self, pk: &DefiniteDescriptorKey) -> Option<bitcoin::ecdsa::Signature> {
        if self.keys.contains(&pk.to_public_key()) {
            Some(worst_case_sig())
        } else {
            None
        }
    }
    fn lookup_sha256(&self, _: &sha256::Hash) -> Option<Preimage32> {
        if self.sha {
            Some([0xabu8; 32])
        } else {
            None
        }
    }
    fn check_after(&self, l: absolute::LockTime) -> bool {
        match self.after {
            Some(a) => a.is_implied_by(l) || l.is_implied_by(a),
            None => false,
        }
    }
    fn check_older(&self, s: relative::LockTime) -> bool {
        match self.older {
            Some(o) => s.is_implied_by(o),
            None => false,
        }
    }
}

fn count_static_ops(script: &bitcoin::Script) -> (usize, usize) {
    // Bitcoin Core: `if (opcode > OP_16 && ++nOpCount > MAX_OPS_PER_SCRIPT)`
    // counted for EVERY opcode encountered, executed or not.
    let mut ops = 0usize;
    let mut instrs = 0usize;
    for ins in script.instruction_indices_minimal() {
        let (_, ins) = ins.expect("script parses");
        instrs += 1;
        if let bitcoin::script::Instruction::Op(op) = ins {
            if op.to_u8() > bitcoin::opcodes::all::OP_PUSHNUM_16.to_u8() {
                ops += 1;
            }
        }
    }
    (ops, instrs)
}

fn main() {
    let raw = DESC.trim();
    println!("== descriptor ==");
    println!("chars (incl. checksum): {}", raw.len());

    let desc = Descriptor::<DescriptorPublicKey>::from_str(raw).expect("parses");
    println!("parsed OK; desc_type = {:?}", desc.desc_type());
    match desc.sanity_check() {
        Ok(()) => println!("Descriptor::sanity_check() = Ok"),
        Err(e) => println!("Descriptor::sanity_check() = Err({e})"),
    }
    println!("max_weight_to_satisfy() = {:?}", desc.max_weight_to_satisfy());

    let singles = desc.clone().into_single_descriptors().expect("multipath split");
    println!("into_single_descriptors() -> {} descriptors", singles.len());

    let d0 = singles[0].clone().at_derivation_index(0).expect("derive");
    let script = d0.explicit_script().expect("wsh has explicit script");
    let spk = d0.script_pubkey();
    println!();
    println!("== witnessScript (receive branch, index 0) ==");
    println!("scriptPubkey       : {}", spk.to_hex_string());
    println!("witnessScript bytes: {}", script.len());
    let (static_ops, instrs) = count_static_ops(&script);
    println!("script instructions: {instrs}");
    println!("static opcodes >OP_16 (measured by walking the script): {static_ops}");

    // inner miniscript
    let ms = match d0.clone() {
        Descriptor::Wsh(w) => w.into_inner(),
        _ => panic!("not wsh"),
    };
    println!();
    println!("== rust-miniscript ExtData (Segwitv0) ==");
    println!("ms.script_size()                       = {}", ms.script_size());
    println!("ms.ext.pk_cost                         = {}", ms.ext.pk_cost);
    println!("ms.ext.static_ops                      = {}", ms.ext.static_ops);
    println!("ms.ext.tree_height                     = {}", ms.ext.tree_height);
    let sd = ms.ext.sat_data.expect("satisfiable");
    println!("sat_data.max_exec_op_count             = {}", sd.max_exec_op_count);
    println!("=> sat_op_count (static + exec)        = {}", ms.ext.static_ops + sd.max_exec_op_count);
    println!("sat_data.max_witness_stack_count       = {}", sd.max_witness_stack_count);
    println!("sat_data.max_witness_stack_size        = {}", sd.max_witness_stack_size);
    println!("sat_data.max_exec_stack_count          = {}", sd.max_exec_stack_count);
    println!("ms.max_satisfaction_witness_elements() = {:?}", ms.max_satisfaction_witness_elements());
    println!("ms.max_satisfaction_size()             = {:?}", ms.max_satisfaction_size());
    println!("d0.max_weight_to_satisfy()             = {:?}", d0.max_weight_to_satisfy());
    println!("ms.sanity_check() (analyzable)         = {:?}", ms.sanity_check());

    // limits
    println!();
    println!("== limits (miniscript::miniscript::limits) ==");
    println!("MAX_OPS_PER_SCRIPT             = {}", miniscript::miniscript::limits::MAX_OPS_PER_SCRIPT);
    println!("MAX_SCRIPT_SIZE                = {}", miniscript::miniscript::limits::MAX_SCRIPT_SIZE);
    println!("MAX_STANDARD_P2WSH_SCRIPT_SIZE = {}", miniscript::miniscript::limits::MAX_STANDARD_P2WSH_SCRIPT_SIZE);
    println!("MAX_STANDARD_P2WSH_STACK_ITEMS = {}", miniscript::miniscript::limits::MAX_STANDARD_P2WSH_STACK_ITEMS);
    println!("MAX_SCRIPT_ELEMENT_SIZE        = {}", miniscript::miniscript::limits::MAX_SCRIPT_ELEMENT_SIZE);
    println!("MAX_STACK_SIZE                 = {}", miniscript::miniscript::limits::MAX_STACK_SIZE);

    // enumerate multi() nodes in tier order
    let mut multis: Vec<(usize, Vec<DefiniteDescriptorKey>)> = vec![];
    for node in ms.pre_order_iter() {
        if let Terminal::Multi(ref thresh) = node.node {
            multis.push((thresh.k(), thresh.iter().cloned().collect()));
        }
    }
    println!();
    println!("== multi() nodes in pre-order ==");
    for (i, (k, keys)) in multis.iter().enumerate() {
        println!("tier {}: multi({}, n={})", i + 1, k, keys.len());
    }

    let sha = sha256::Hash::from_str("a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad").unwrap();

    struct TierSpec {
        name: &'static str,
        idx: usize,
        sha: bool,
        after: Option<absolute::LockTime>,
        older: Option<relative::LockTime>,
    }
    let tiers = vec![
        TierSpec { name: "T1 3-of-3 + sha256, after HEIGHT 1000000", idx: 0, sha: true,
                   after: Some(absolute::LockTime::from_height(1_000_000).unwrap()), older: None },
        TierSpec { name: "T2 2-of-3 + sha256, after TIME 1893456000", idx: 1, sha: true,
                   after: Some(absolute::LockTime::from_time(1_893_456_000).unwrap()), older: None },
        TierSpec { name: "T3 2-of-2, older BLOCKS 65535", idx: 2, sha: false, after: None,
                   older: Some(relative::LockTime::from_height(65535)) },
        TierSpec { name: "T4 1-of-3, older TIME 4255898 (bip68 time flag)", idx: 3, sha: false, after: None,
                   older: Some(relative::LockTime::from_512_second_intervals(65535)) },
    ];

    println!();
    println!("== per-path measured satisfaction (real witness built by Descriptor::get_satisfaction) ==");
    for t in &tiers {
        let (k, keys) = &multis[t.idx];
        // give exactly k signing keys of that multi
        let signing: BTreeSet<bitcoin::PublicKey> =
            keys.iter().take(*k).map(|pk| pk.to_public_key()).collect();
        let sat = Sat { keys: signing, sha: t.sha, after: t.after, older: t.older };
        match d0.get_satisfaction(&sat) {
            Ok((wit, ss)) => {
                let total: usize = wit.iter().map(|w| w.len() + bitcoin::VarInt(w.len() as u64).size()).sum::<usize>()
                    + bitcoin::VarInt(wit.len() as u64).size();
                let non_script: Vec<usize> = wit[..wit.len() - 1].iter().map(|w| w.len()).collect();
                let maxitem = non_script.iter().copied().max().unwrap_or(0);
                let ops = static_ops + keys.len();
                println!();
                println!("-- {}", t.name);
                println!("   scriptSig len                : {}", ss.len());
                println!("   witness stack items (total)  : {}", wit.len());
                println!("   ... excluding witnessScript  : {}", wit.len() - 1);
                println!("   item sizes (excl. script)    : {:?}", non_script);
                println!("   max item size (excl. script) : {}", maxitem);
                println!("   last item (witnessScript) len: {}", wit[wit.len() - 1].len());
                println!("   total witness bytes (w/ varints, excl. per-input marker): {}", total);
                println!("   executed op count            : {} (= {} static + {} multisig keys)", ops, static_ops, keys.len());
            }
            Err(e) => println!("-- {}\n   get_satisfaction FAILED: {e}", t.name),
        }
    }

    // --- extra checks ---
    println!();
    println!("== sigops (rust-bitcoin Script::count_sigops, BIP141 accurate counting) ==");
    println!("witnessScript.count_sigops()        = {}", script.count_sigops());
    println!("witnessScript.count_sigops_legacy() = {}", script.count_sigops_legacy());

    println!();
    println!("== script size stability across derivation index / multipath branch ==");
    for (bi, sd) in singles.iter().enumerate() {
        for idx in [0u32, 1, 1000, 2147483647] {
            let dd = sd.clone().at_derivation_index(idx).unwrap();
            let sc = dd.explicit_script().unwrap();
            let (o, _) = count_static_ops(&sc);
            print!("branch{bi} idx{idx}: {} bytes / {} static ops   ", sc.len(), o);
        }
        println!();
    }

    println!();
    println!("== witness item bytes (MINIMALIF / standardness inspection), T1 ==");
    {
        let (k, keys) = &multis[0];
        let signing: BTreeSet<bitcoin::PublicKey> =
            keys.iter().take(*k).map(|pk| pk.to_public_key()).collect();
        let sat = Sat { keys: signing, sha: true,
                        after: Some(absolute::LockTime::from_height(1_000_000).unwrap()), older: None };
        let (wit, _) = d0.get_satisfaction(&sat).unwrap();
        for (i, w) in wit.iter().enumerate() {
            if i == wit.len() - 1 {
                println!("  [{i}] witnessScript, {} bytes", w.len());
            } else {
                println!("  [{i}] {} bytes = {}", w.len(), hex::DisplayHex::as_hex(w.as_slice()));
            }
        }
    }

    println!();
    println!("== interpreter run-check: does each witness actually satisfy the script? ==");
    for t in &tiers {
        let (k, keys) = &multis[t.idx];
        let signing: BTreeSet<bitcoin::PublicKey> =
            keys.iter().take(*k).map(|pk| pk.to_public_key()).collect();
        let sat = Sat { keys: signing, sha: t.sha, after: t.after, older: t.older };
        let (wit, ss) = d0.get_satisfaction(&sat).unwrap();
        let witness = bitcoin::Witness::from_slice(&wit);
        let seq = match t.older {
            Some(o) => bitcoin::Sequence::from_consensus(match o {
                relative::LockTime::Blocks(h) => u32::from(h.value()),
                relative::LockTime::Time(m) => (1u32 << 22) | u32::from(m.value()),
            }),
            None => bitcoin::Sequence::ENABLE_LOCKTIME_NO_RBF,
        };
        let lt = t.after.unwrap_or(absolute::LockTime::ZERO);
        let interp = miniscript::interpreter::Interpreter::from_txdata(
            &spk, ss.as_script(), &witness, seq, lt).expect("interpreter builds");
        let mut n = 0;
        let mut err = None;
        for res in interp.iter_assume_sigs() {
            match res { Ok(_) => n += 1, Err(e) => { err = Some(format!("{e}")); break } }
        }
        println!("-- {}\n   satisfied conditions = {n}, error = {:?}", t.name, err);
    }

    println!();
    println!("== interpreter run-check on a hash-substituted twin (proves T1/T2 branch structure) ==");
    {
        use bitcoin::hashes::Hash as _;
        let pre = [0xabu8; 32];
        let h = sha256::Hash::hash(&pre);
        let twin_str = raw.split('#').next().unwrap()
            .replace("a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad", &format!("{h}"));
        let twin = Descriptor::<DescriptorPublicKey>::from_str(&twin_str).unwrap();
        let t0 = twin.into_single_descriptors().unwrap()[0].clone().at_derivation_index(0).unwrap();
        let tspk = t0.script_pubkey();
        let tsc = t0.explicit_script().unwrap();
        println!("twin witnessScript bytes = {} (same length as the real one: {})", tsc.len(), tsc.len() == script.len());
        let tms = match t0.clone() { Descriptor::Wsh(w) => w.into_inner(), _ => unreachable!() };
        let mut tmultis: Vec<(usize, Vec<DefiniteDescriptorKey>)> = vec![];
        for node in tms.pre_order_iter() {
            if let Terminal::Multi(ref th) = node.node { tmultis.push((th.k(), th.iter().cloned().collect())); }
        }
        for t in &tiers[..2] {
            let (k, keys) = &tmultis[t.idx];
            let signing: BTreeSet<bitcoin::PublicKey> = keys.iter().take(*k).map(|pk| pk.to_public_key()).collect();
            let sat = Sat { keys: signing, sha: true, after: t.after, older: t.older };
            let (wit, ss) = t0.get_satisfaction(&sat).unwrap();
            let witness = bitcoin::Witness::from_slice(&wit);
            let lt = t.after.unwrap();
            let interp = miniscript::interpreter::Interpreter::from_txdata(
                &tspk, ss.as_script(), &witness, bitcoin::Sequence::ENABLE_LOCKTIME_NO_RBF, lt).unwrap();
            let mut n = 0; let mut err = None;
            for res in interp.iter_assume_sigs() {
                match res { Ok(_) => n += 1, Err(e) => { err = Some(format!("{e}")); break } }
            }
            println!("-- {}\n   satisfied conditions = {n}, error = {:?}", t.name, err);
        }
    }

    println!();
    println!("== wsh Plan::witness_size() vs the REAL witness (does it include the witnessScript?) ==");
    println!("(this is what mnemonic-toolkit `compare-cost` reads: cost/enumerate.rs:267)");
    for t in &tiers {
        let (k, keys) = &multis[t.idx];
        let mut assets = Assets::new();
        for pk in keys.iter().take(*k) {
            let fp = pk.master_fingerprint();
            let path = pk.full_derivation_paths().into_iter().next().unwrap();
            assets.keys.insert(((fp, path), miniscript::plan::CanSign::default()));
        }
        if t.sha { let mut ss = BTreeSet::new(); ss.insert(sha); assets.sha256_preimages = ss; }
        if let Some(a) = t.after { assets = assets.after(a); }
        if let Some(o) = t.older { assets = assets.older(o); }
        let plan_ws = d0.clone().into_plan(&assets).ok().map(|p| p.witness_size()).unwrap();

        let signing: BTreeSet<bitcoin::PublicKey> = keys.iter().take(*k).map(|pk| pk.to_public_key()).collect();
        let sat = Sat { keys: signing, sha: t.sha, after: t.after, older: t.older };
        let (wit, _) = d0.get_satisfaction(&sat).unwrap();
        let real: usize = wit.iter().map(|w| w.len() + bitcoin::VarInt(w.len() as u64).size()).sum::<usize>()
            + bitcoin::VarInt(wit.len() as u64).size();
        let vb = |w: usize| (164 + w + 3) / 4;
        println!("-- {}", t.name);
        println!("   Plan::witness_size() = {plan_ws:>4}  -> compare-cost reports {} vB", vb(plan_ws));
        println!("   real witness bytes   = {real:>4}  -> true cost           {} vB   (delta {} bytes = witnessScript {} + varint {})",
                 vb(real), real - plan_ws, script.len(), real - plan_ws - script.len());
    }

    println!();
    println!("== tr(NUMS, single-leaf) twin: does Plan::witness_size() include the tapleaf script? ==");
    {
        const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
        let inner = raw.trim().split('#').next().unwrap();
        let inner = inner.strip_prefix("wsh(").unwrap().strip_suffix(")").unwrap();
        let tr_str = format!("tr({},{})", NUMS, inner.replace("multi(", "multi_a("));
        let trd = Descriptor::<DescriptorPublicKey>::from_str(&tr_str).expect("tr parses");
        let t0 = trd.into_single_descriptors().unwrap()[0].clone().at_derivation_index(0).unwrap();
        let tms = match t0.clone() { Descriptor::Tr(ref tr) => tr.clone(), _ => unreachable!() };
        let leaf_len: usize = tms.spend_info().leaves().map(|l| l.script().len()).sum();
        println!("tapleaf script bytes = {leaf_len}");
        let tinner = match t0.clone() { Descriptor::Tr(tr) => tr, _ => unreachable!() };
        let mut tmultis: Vec<(usize, Vec<DefiniteDescriptorKey>)> = vec![];
        for leaf in tinner.spend_info().leaves() {
            for node in leaf.miniscript().pre_order_iter() {
                if let Terminal::MultiA(ref th) = node.node { tmultis.push((th.k(), th.iter().cloned().collect())); }
            }
        }
        let vb = |w: usize| (164 + w + 3) / 4;
        for t in &tiers {
            let (k, keys) = &tmultis[t.idx];
            let mut assets = Assets::new();
            for pk in keys.iter().take(*k) {
                let fp = pk.master_fingerprint();
                let path = pk.full_derivation_paths().into_iter().next().unwrap();
                assets.keys.insert(((fp, path), miniscript::plan::CanSign::default()));
            }
            if t.sha { let mut ss = BTreeSet::new(); ss.insert(sha); assets.sha256_preimages = ss; }
            if let Some(a) = t.after { assets = assets.after(a); }
            if let Some(o) = t.older { assets = assets.older(o); }
            match t0.clone().into_plan(&assets) {
                Ok(p) => {
                    let has_script = p.witness_template().iter().any(|ph| matches!(ph, miniscript::miniscript::satisfy::Placeholder::TapScript(_)));
                    let has_cb = p.witness_template().iter().any(|ph| matches!(ph, miniscript::miniscript::satisfy::Placeholder::TapControlBlock(_)));
                    println!("-- {}\n   tr witness_size() = {} -> {} vB | template has TapScript={} TapControlBlock={}",
                             t.name, p.witness_size(), vb(p.witness_size()), has_script, has_cb);
                }
                Err(_) => println!("-- {}\n   NO tr PLAN", t.name),
            }
        }
    }

    println!();
    println!("== scaling probe: how many MORE tiers of this shape fit before a limit binds? ==");
    {
        use secp256k1::{Secp256k1, SecretKey, PublicKey};
        let secp = Secp256k1::new();
        let mut pks: Vec<String> = vec![];
        for i in 1u32..200 {
            let mut sk = [0u8; 32];
            sk[28..].copy_from_slice(&i.to_be_bytes());
            let sk = SecretKey::from_slice(&sk).unwrap();
            pks.push(format!("{}", PublicKey::from_secret_key(&secp, &sk)));
        }
        // tier shape: and_v(v:older(N), multi(2, pk, pk))  -- 2 keys, cheapest realistic tier
        for tiers_n in 1..=60usize {
            let mut inner = String::new();
            let mut close = 0;
            for t in 0..tiers_n {
                let a = &pks[(2 * t) % pks.len()];
                let b = &pks[(2 * t + 1) % pks.len()];
                let leaf = format!("and_v(v:older({}),multi(2,{},{}))", 100 + t, a, b);
                if t == tiers_n - 1 {
                    inner.push_str(&leaf);
                } else {
                    inner.push_str(&format!("or_i({},", leaf));
                    close += 1;
                }
            }
            for _ in 0..close { inner.push(')'); }
            let ds = format!("wsh({})", inner);
            match Descriptor::<DescriptorPublicKey>::from_str(&ds) {
                Ok(d) => {
                    let sc = d.clone().at_derivation_index(0).map(|x| x.explicit_script().unwrap().len());
                    let m = match d.clone() { Descriptor::Wsh(w) => Some(w.into_inner()), _ => None }.unwrap();
                    let ops = m.ext.static_ops + m.ext.sat_data.unwrap().max_exec_op_count;
                    let items = m.max_satisfaction_witness_elements().unwrap();
                    let sane = d.sanity_check().err().map(|e| format!("{e}"));
                    if tiers_n % 5 == 0 || sane.is_some() {
                        println!("  tiers={tiers_n:>2}: script={:?} ops={ops} wit_items={items} sanity_err={:?}",
                                 sc.map(|x| x), sane);
                    }
                    if sane.is_some() { break }
                }
                Err(e) => { println!("  tiers={tiers_n:>2}: PARSE REFUSED: {e}"); break }
            }
        }
    }

    // Plans, for cross-check
    println!();
    println!("== per-path Plan (rust-miniscript planning module) ==");
    for t in &tiers {
        let (k, keys) = &multis[t.idx];
        let mut assets = Assets::new();
        for pk in keys.iter().take(*k) {
            let fp = pk.master_fingerprint();
            let path = pk.full_derivation_paths().into_iter().next().unwrap();
            assets.keys.insert(((fp, path), miniscript::plan::CanSign::default()));
        }
        if t.sha {
            let mut s = BTreeSet::new();
            s.insert(sha);
            assets.sha256_preimages = s;
        }
        if let Some(a) = t.after {
            assets = assets.after(a);
        }
        if let Some(o) = t.older {
            assets = assets.older(o);
        }
        match d0.clone().into_plan(&assets) {
            Ok(p) => println!("-- {}\n   template items = {}, witness_size() = {}, satisfaction_weight() = {}, abs_lock = {:?}, rel_lock = {:?}",
                              t.name, p.witness_template().len(), p.witness_size(), p.satisfaction_weight(), p.absolute_timelock, p.relative_timelock),
            Err(_) => println!("-- {}\n   NO PLAN with those assets", t.name),
        }
    }
}
