//! **Per-key receive-address-0 derivation, for §5.4's `address 0:` line.**
//!
//! [`super::md1::derivation_twin`] derives through `md_codec`, which takes ONE
//! `(chain, index)` pair for the whole descriptor. That is exact for every
//! wallet whose keys want the same receive index — which is every wallet the
//! vector corpus carries — and it is the path the adversarial review measured
//! clean against the device over 71 inputs × 3 flag states.
//!
//! It cannot express a wallet whose keys want DIFFERENT indices, because md1's
//! use-site path always ends in exactly one wildcard level and that level takes
//! the single global index. `wsh(sortedmulti(2, K1/<2;3>, K2/<0;1>/*))` wants
//! `K1/2` and `K2/0/0`: no `(chain, index)` produces both. The twin returned
//! `None` there and the block printed a sentence claiming the address did not
//! exist — it does, and the device derives it
//! (`bc1qv70wqy0t9vp4ftlku3yz845x53yqkgm5xlus47m3zq8xzzy503hscqluvy`,
//! IMPL-S1S3-adversarial-review I1, reconfirmed here through the fork's own
//! `address.Receive`).
//!
//! This module derives **each key at its own receive path**, which is what the
//! device does and what removes the limitation rather than describing it.
//!
//! | use-site | this key's receive-address-0 path |
//! | --- | --- |
//! | absent | `/0/0` — §5.3(a′)'s materialised `<0;1>/*`, receive, index 0 |
//! | `/*` | `/0` |
//! | `/i/*` | `/i/0` |
//! | `<i;i+1>` | `/i` |
//! | `<i;i+1>/*` | `/i/0` |
//!
//! # Why a second derivation is safe here
//!
//! Two implementations of one answer is the F-212 divergence class this cycle
//! exists to guard against, so this one is **held to the first**:
//! `the_two_derivations_agree_wherever_both_can_derive` runs both over every
//! vector row and every constructed mixture and requires equality. `md_codec`
//! stays the authority for what is PACKED — nothing here ever reaches a card.

use bitcoin::bip32::{ChainCode, ChildNumber, Fingerprint, Xpub};
use bitcoin::opcodes::all::OP_CHECKMULTISIG;
use bitcoin::script::Builder;
use bitcoin::secp256k1::{PublicKey, Secp256k1};
use bitcoin::{Address, NetworkKind, ScriptBuf};

use super::cascade::{Derivation, Key, Multi, Parsed, Script};

/// The derivation `derive` applies to a key that carries none of its own:
/// BIP-388's `<0;1>/*`. Stated ONCE, so that everything asking "is this the
/// same key" applies the same default.
const DEFAULT_CHILDREN: [Derivation; 2] = [
    Derivation::Range { start: 0, end: 1 },
    Derivation::Wildcard { hardened: false },
];

/// One element as `derive` resolves it for `change`: the child id, and whether
/// this is the trailing wildcard (whose id is the address index, equal for
/// equal indices).
///
/// `None` for an element derivation refuses, so a key that cannot derive is
/// never reported as deriving the same thing as another.
///
/// HARDENED IS ABSENT ON PURPOSE, mirroring `derive`: `/0h/*` derives the
/// unhardened child and `/*h` the plain wildcard, so two spellings differing
/// only in hardening ARE one key.
fn resolve_child(d: &Derivation, change: bool) -> Option<(u32, bool)> {
    match d {
        Derivation::Child { index, .. } => Some((*index, false)),
        Derivation::Range { start, end } => {
            if *end != start + 1 {
                return None;
            }
            Some((if change { *end } else { *start }, false))
        }
        Derivation::Wildcard { .. } => Some((0, true)),
    }
}

/// Whether two key expressions put the SAME public key at every address index,
/// on at least one chain.
///
/// THE QUESTION CONJUNCT 8(b) ASKS, answered here rather than in `admit`,
/// because the answer depends on the normalisations in this module. The check
/// compared `a.children == b.children` and admitted four spellings of one key
/// (F-530 review C-2); comparing `receive_path` alone then fixed that but left
/// the Go port and this half disagreeing on a key that collides only on CHANGE
/// (fold review NEW-1). Change addresses hold funds too, so EITHER chain is the
/// rule, and it is now the same sentence on both sides -- `address.DerivesSameKey`
/// in the fork is this function.
pub(crate) fn derives_same_key(a: &Key, b: &Key) -> bool {
    if a.identity() != b.identity() {
        return false;
    }
    let ac: &[Derivation] = if a.children.is_empty() {
        &DEFAULT_CHILDREN
    } else {
        &a.children
    };
    let bc: &[Derivation] = if b.children.is_empty() {
        &DEFAULT_CHILDREN
    } else {
        &b.children
    };
    if ac.len() != bc.len() {
        return false;
    }
    [false, true].iter().any(|&change| {
        ac.iter().zip(bc.iter()).all(|(x, y)| {
            match (resolve_child(x, change), resolve_child(y, change)) {
                (Some(px), Some(py)) => px == py,
                _ => false,
            }
        })
    })
}

/// One key's path to ITS receive address 0, or `None` for a use-site outside
/// §4.7 conjunct 7's closed set (unreachable after admission).
///
/// `pub(crate)` so that conjunct 8(b) compares use sites by their MEANING
/// rather than by the spelling of `children`. That check read
/// `a.children == b.children`, and an absent path is the device's `<0;1>/*` --
/// stated four lines below, in this function, in the same crate -- so
/// `wsh(sortedmulti(2,A,A/<0;1>/*,B))` was admitted as two distinct keys while
/// deriving one key twice. Sharing THIS function rather than restating the
/// normalisation is the point: a second copy is what drifted (F-530 review C-2).
pub(crate) fn receive_path(k: &Key) -> Option<Vec<u32>> {
    use Derivation::*;
    let plain = Wildcard { hardened: false };
    Some(match k.children.as_slice() {
        // §5.3(a′): an absent path IS the device's `<0;1>/*`, made explicit.
        [] => vec![0, 0],
        [w] if *w == plain => vec![0],
        [Range { start, .. }, w] if *w == plain => vec![*start, 0],
        [Child {
            index,
            hardened: false,
        }, w]
            if *w == plain =>
        {
            vec![*index, 0]
        }
        // A multipath group with NO trailing wildcard is ONE address per chain,
        // so receive address 0 is the receive alternative itself.
        [Range { start, .. }] => vec![*start],
        _ => return None,
    })
}

/// Receive address 0 for `d`, derived key by key. `None` when the descriptor
/// carries a use-site conjunct 7 refuses, or a key that will not derive.
pub fn address_0(d: &Parsed) -> Option<String> {
    let secp = Secp256k1::verification_only();
    let net = match d.keys.first()?.network {
        super::cascade::Network::Testnet => NetworkKind::Test,
        super::cascade::Network::Mainnet => NetworkKind::Main,
    };

    let mut pubkeys: Vec<PublicKey> = Vec::with_capacity(d.keys.len());
    for k in &d.keys {
        let path: Vec<ChildNumber> = receive_path(k)?
            .into_iter()
            // Hardened public derivation is impossible (BIP-32) and conjunct 7
            // refuses it upstream; `Normal` is the only shape that reaches here.
            .map(|i| ChildNumber::Normal { index: i })
            .collect();
        let xpub = Xpub {
            network: net,
            // The four metadata fields take no part in CKDpub — only the chain
            // code and the public key do — so they carry placeholders rather
            // than a rebuilt depth that would imply a claim.
            depth: 0,
            parent_fingerprint: Fingerprint::default(),
            child_number: ChildNumber::Normal { index: 0 },
            public_key: PublicKey::from_slice(&k.key_data).ok()?,
            chain_code: ChainCode::from(k.chain_code),
        };
        pubkeys.push(xpub.derive_pub(&secp, &path).ok()?.public_key);
    }

    let hrp = match net {
        NetworkKind::Main => bitcoin::Network::Bitcoin,
        NetworkKind::Test => bitcoin::Network::Testnet,
    };
    let addr = match (d.script, d.multi) {
        (Script::P2PKH, None) => Address::p2pkh(bitcoin::PublicKey::new(pubkeys[0]), net),
        (Script::P2WPKH, None) => Address::p2wpkh(&bitcoin::CompressedPublicKey(pubkeys[0]), hrp),
        (Script::P2SH_P2WPKH, None) => {
            Address::p2shwpkh(&bitcoin::CompressedPublicKey(pubkeys[0]), net)
        }
        (Script::P2TR, None) => {
            let (xonly, _) = pubkeys[0].x_only_public_key();
            Address::p2tr(&secp, xonly, None, hrp)
        }
        (Script::P2WSH, Some(m)) => Address::p2wsh(&multisig(d, &pubkeys, m)?, hrp),
        (Script::P2SH_P2WSH, Some(m)) => {
            Address::p2sh(&multisig(d, &pubkeys, m)?.to_p2wsh(), net).ok()?
        }
        (Script::P2SH, Some(m)) => Address::p2sh(&multisig(d, &pubkeys, m)?, net).ok()?,
        // Every remaining pair is refused by conjunct 1 before this runs.
        _ => return None,
    };
    Some(addr.to_string())
}

/// The bare multisig redeemScript: `OP_k <pk>… OP_n OP_CHECKMULTISIG`.
///
/// **`sortedmulti` sorts the DERIVED keys**, lexicographically over their
/// 33-byte compressed serialisations, at the use site — BIP-67's rule, and the
/// reason `multi` and `sortedmulti` are not synonyms. `multi` keeps the
/// operator's order.
fn multisig(d: &Parsed, pubkeys: &[PublicKey], m: Multi) -> Option<ScriptBuf> {
    let mut ser: Vec<[u8; 33]> = pubkeys.iter().map(|p| p.serialize()).collect();
    if m == Multi::Sorted {
        ser.sort_unstable();
    }
    let k = i64::from(u32::try_from(d.threshold).ok()?);
    let mut b = Builder::new().push_int(k);
    for p in &ser {
        b = b.push_slice(p);
    }
    Some(
        b.push_int(ser.len() as i64)
            .push_opcode(OP_CHECKMULTISIG)
            .into_script(),
    )
}
