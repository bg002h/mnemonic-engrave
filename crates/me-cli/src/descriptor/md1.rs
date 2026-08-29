//! **§5.3 — the `--as md1` build path, in process.**
//!
//! `me` builds the [`md_codec::Descriptor`] itself. It does not shell out to
//! `md` and it does not depend on `md-cli`: §2.6(b) established that `md-cli` is
//! bin-only, and a CLI's stdout is a channel that has already caused a
//! cross-tool defect in this constellation.
//!
//! # What this module is held to
//!
//! The Go device has its own builder for the same wallets
//! (`md.EncodeMultisig`, fork `md/encode_multisig.go`), and §7's `wallet_id`
//! column is the F-212 cross-language gate between them. Four decisions here
//! are therefore not free — they are what the Go route does, read from its
//! source rather than inferred:
//!
//! | decision | here | `encode_multisig.go` |
//! | --- | --- | --- |
//! | pubkeys TLV | every key, idx-ascending | `pubPresent: true`, all `n` (`:150`) |
//! | fingerprints TLV | only keys whose fingerprint is non-zero | `if c.FpPresent` (`:157`) |
//! | key order | input order fixes `@0..@n-1` | the ordering contract (`:16-24`) |
//! | use-site | §5.3(a′)'s materialised `<0;1>/*` | hard-coded `<0;1>/*` (`:167`) |
//!
//! The origin declaration is the one place the two differ and CANNOT diverge on
//! anything observable: Go always writes `OriginDivergent`, this writes
//! `Shared` when every key's origin is the same path. `compute_wallet_policy_id`
//! resolves per-`@N` origins through `expand_per_at_n` before hashing, and a
//! shared path resolves to the identical per-key set, so the id is the same
//! either way — asserted, not assumed, by the `wallet_id` rows of
//! `tests/descriptor_seam.rs`.
//!
//! # Conjunct 8 refuses BEFORE anything here runs
//!
//! The PUBLISHED `md-codec` 0.42.0 that `me` links predates F-217/F-218's
//! encode-time validators, so an impossible wallet would encode clean. The
//! caller runs [`super::admit::admit`] with [`super::admit::Path::Md1`] first,
//! which is where conjunct 8 lives, and this module is never reached for one.
//! F-424 owns the codec bump.

use md_codec::encode::Descriptor as MdDescriptor;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::{Alternative, UseSitePath};

use super::cascade::{Derivation, Key, Multi, Network, Parsed, Script, HARDENED};

/// Why a descriptor is being built.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Purpose {
    /// The card set the operator engraves. Every key's use-site path must be
    /// md1-representable — the caller has already run
    /// [`super::admit::md1_representable`], so an (a)/(a″) shape reaching here
    /// is a caller bug, not an operator input.
    Encode,
    /// A DERIVATION TWIN, for §5.4's `address 0:` line only. Never encoded,
    /// never packed, never hashed into an identity — see [`derivation_twin`].
    Derive,
}

/// A built md1 descriptor and the facts §5.4's block prints about it.
pub struct Built {
    pub descriptor: MdDescriptor,
    /// `descriptor_to_template`'s rendering — the template WITH §5.3(a′)'s
    /// materialised `<0;1>/*`, which is what §5.4 requires the block to show.
    pub template: String,
    /// `@N` → the fingerprint the input declared, `None` for "master unknown".
    pub slots: Vec<(usize, Option<u32>)>,
    /// Whether §5.3(a′) materialised a default into at least one key — the
    /// annotation line fires exactly when this is true.
    pub materialised: bool,
}

/// What went wrong. Kept separate from §6's [`super::refusal::Row`] because
/// every variant here is either unreachable after admission or a codec-internal
/// limit, and neither is an operator-language refusal.
#[derive(Debug)]
pub enum BuildError {
    /// The codec refused the descriptor. Carries its own message.
    Codec(md_codec::Error),
    /// `descriptor_to_template` could not render the tree.
    Render(md_codec::RenderError),
    /// A shape [`Purpose::Encode`] cannot carry. Unreachable from `main.rs`,
    /// which checks representability first.
    Unrepresentable(usize),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Codec(e) => write!(f, "{e}"),
            Self::Render(e) => write!(f, "{e}"),
            Self::Unrepresentable(i) => {
                write!(f, "key @{i}'s use-site path has no md1 form")
            }
        }
    }
}

impl From<md_codec::Error> for BuildError {
    fn from(e: md_codec::Error) -> Self {
        Self::Codec(e)
    }
}

// ───────────────────────────────────────────────────────────────────────────
// §5.3(a′) — the materialisation, stated once
// ───────────────────────────────────────────────────────────────────────────

/// The `<0;1>/*` the device defaults an empty children list to
/// (`address/address.go:188–202`).
fn device_default() -> UseSitePath {
    UseSitePath::standard_multipath()
}

/// One key's use-site path in md1 form, for `purpose`.
///
/// The five members of §4.7 conjunct 7's closed set, and what each becomes:
///
/// | input | `Encode` | `Derive` (twin) |
/// | --- | --- | --- |
/// | absent | `<0;1>/*` — §5.3(a′) MATERIALISED | `<0;1>/*` |
/// | `/*` | `/*` | `/*` |
/// | `<i;i+1>/*` | `<i;i+1>/*` | `<i;i+1>/*` |
/// | `/i/*` | REFUSED — §5.3(a) | `<i;i+1>/*`, chain 0 |
/// | `<i;i+1>` | REFUSED — §5.3(a″) | `/*`, index `i` |
///
/// The two `Derive` rows are the whole reason the twin exists, and each is an
/// EQUALITY rather than an approximation: `/i/*` at address 0 is `key/i/0`,
/// which is `<i;i+1>/*` chain 0 index 0; `<i;i+1>` at address 0 is `key/i`,
/// which is `/*` at index `i`. Neither twin is ever encoded — encoding either
/// is exactly the silent wallet change §5.3 refuses.
fn use_site_for(k: &Key, i: usize, purpose: Purpose) -> Result<(UseSitePath, u32), BuildError> {
    use Derivation::*;
    let plain = Wildcard { hardened: false };
    match k.children.as_slice() {
        // (a′): the absent path IS the device default, made explicit.
        [] => Ok((device_default(), 0)),
        [w] if *w == plain => Ok((
            UseSitePath {
                multipath: None,
                wildcard_hardened: false,
            },
            0,
        )),
        [Range { start, end }, w] if *w == plain => Ok((range_path(*start, *end), 0)),
        // (a): a single fixed chain index.
        [Child {
            index,
            hardened: false,
        }, w]
            if *w == plain =>
        {
            match purpose {
                Purpose::Encode => Err(BuildError::Unrepresentable(i)),
                Purpose::Derive => Ok((range_path(*index, index + 1), 0)),
            }
        }
        // (a″): a multipath group with no trailing wildcard.
        [Range { start, .. }] => match purpose {
            Purpose::Encode => Err(BuildError::Unrepresentable(i)),
            Purpose::Derive => Ok((
                UseSitePath {
                    multipath: None,
                    wildcard_hardened: false,
                },
                *start,
            )),
        },
        // Unreachable after conjunct 7, which is the closed set this matches.
        _ => Err(BuildError::Unrepresentable(i)),
    }
}

fn range_path(start: u32, end: u32) -> UseSitePath {
    UseSitePath {
        multipath: Some(vec![
            Alternative {
                hardened: false,
                value: start,
            },
            Alternative {
                hardened: false,
                value: end,
            },
        ]),
        wildcard_hardened: false,
    }
}

// ───────────────────────────────────────────────────────────────────────────
// The build
// ───────────────────────────────────────────────────────────────────────────

/// Build the md1 descriptor `--as md1` packs.
pub fn build(d: &Parsed) -> Result<Built, BuildError> {
    build_for(d, Purpose::Encode).map(|(b, _)| b)
}

/// Build the DERIVATION TWIN and the wildcard index §5.4's `address 0:` line
/// derives at. `None` for the index means no single index serves every key —
/// see [`use_site_for`]'s table; it takes a descriptor mixing a
/// `<i;i+1>`-without-wildcard key at `i ≠ 0` with a key of another shape.
pub fn derivation_twin(d: &Parsed) -> Result<(Built, Option<u32>), BuildError> {
    build_for(d, Purpose::Derive)
}

fn build_for(d: &Parsed, purpose: Purpose) -> Result<(Built, Option<u32>), BuildError> {
    let n = d.keys.len();
    let mut paths: Vec<UseSitePath> = Vec::with_capacity(n);
    // The index every key agrees on for "receive address 0". A wildcard-bearing
    // key wants 0; an (a″) twin wants its own `start`. `None` once two keys
    // disagree — the caller then prints no address rather than a wrong one.
    let mut index0: Option<u32> = Some(0);
    let mut wants: Vec<u32> = Vec::with_capacity(n);
    let mut materialised = false;
    for (i, k) in d.keys.iter().enumerate() {
        if k.children.is_empty() {
            materialised = true;
        }
        let (p, want) = use_site_for(k, i, purpose)?;
        paths.push(p);
        wants.push(want);
    }
    if let Some(first) = wants.first() {
        if wants.iter().any(|w| w != first) {
            index0 = None;
        } else {
            index0 = Some(*first);
        }
    }

    // Baseline + sparse per-`@N` overrides. md1 carries per-key divergence
    // natively (`TLV_USE_SITE_PATH_OVERRIDES`), which is what makes §5.3's
    // per-key quantifier encodable rather than only statable.
    let baseline = paths[0].clone();
    let overrides: Vec<(u8, UseSitePath)> = paths
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, p)| **p != baseline)
        .map(|(i, p)| (i as u8, p.clone()))
        .collect();

    let path_decl = path_decl_for(&d.keys);
    let tree = tree_for(d)?;

    let mut tlv = TlvSection::new_empty();
    if !overrides.is_empty() {
        tlv.use_site_path_overrides = Some(overrides);
    }
    let fps: Vec<(u8, [u8; 4])> = d
        .keys
        .iter()
        .enumerate()
        .filter(|(_, k)| k.fingerprint != 0)
        .map(|(i, k)| (i as u8, k.fingerprint.to_be_bytes()))
        .collect();
    if !fps.is_empty() {
        tlv.fingerprints = Some(fps);
    }
    tlv.pubkeys = Some(
        d.keys
            .iter()
            .enumerate()
            .map(|(i, k)| {
                let mut b = [0u8; 65];
                b[..32].copy_from_slice(&k.chain_code);
                b[32..].copy_from_slice(&k.key_data);
                (i as u8, b)
            })
            .collect(),
    );

    let descriptor = MdDescriptor {
        n: n as u8,
        path_decl,
        use_site_path: baseline,
        tree,
        tlv,
    };
    let template = md_codec::descriptor_to_template(&descriptor).map_err(BuildError::Render)?;
    let slots = d
        .keys
        .iter()
        .enumerate()
        .map(|(i, k)| (i, (k.fingerprint != 0).then_some(k.fingerprint)))
        .collect();
    Ok((
        Built {
            descriptor,
            template,
            slots,
            materialised,
        },
        index0,
    ))
}

/// `Shared` when every key declares the same origin, `Divergent` otherwise.
/// Both resolve to the identical per-`@N` origin set, which is what
/// `compute_wallet_policy_id` hashes — so this choice is a card-size decision,
/// never an identity one.
fn path_decl_for(keys: &[Key]) -> PathDecl {
    let paths: Vec<OriginPath> = keys.iter().map(|k| origin_path(&k.origin)).collect();
    let shared = paths.iter().all(|p| *p == paths[0]);
    PathDecl {
        n: keys.len() as u8,
        paths: if shared {
            PathDeclPaths::Shared(paths.into_iter().next().unwrap_or(OriginPath {
                components: Vec::new(),
            }))
        } else {
            PathDeclPaths::Divergent(paths)
        },
    }
}

fn origin_path(origin: &[u32]) -> OriginPath {
    OriginPath {
        components: origin
            .iter()
            .map(|e| PathComponent {
                hardened: *e >= HARDENED,
                value: e & !HARDENED,
            })
            .collect(),
    }
}

/// The operator AST for §4.7's shapes. `multi` and `sortedmulti` are separate
/// tags on the wire, which is what lets md1 carry the form the device's own
/// parser refuses.
fn tree_for(d: &Parsed) -> Result<Node, BuildError> {
    let inner = match d.multi {
        Some(m) => Node {
            tag: match m {
                Multi::Sorted => Tag::SortedMulti,
                Multi::Unsorted => Tag::Multi,
            },
            body: Body::MultiKeys {
                k: d.threshold as u8,
                indices: (0..d.keys.len() as u8).collect(),
            },
        },
        None => match d.script {
            Script::P2PKH => key_arg(Tag::Pkh),
            Script::P2WPKH | Script::P2SH_P2WPKH => key_arg(Tag::Wpkh),
            Script::P2TR => Node {
                tag: Tag::Tr,
                body: Body::Tr {
                    is_nums: false,
                    key_index: 0,
                    tree: None,
                },
            },
            // Unreachable after conjunct 1: a key in a `wsh`/`sh` script slot is
            // refused, and every other single-key script is covered above.
            _ => return Err(BuildError::Unrepresentable(0)),
        },
    };
    Ok(match d.script {
        Script::P2WSH => wrap(Tag::Wsh, inner),
        Script::P2SH_P2WSH => wrap(Tag::Sh, wrap(Tag::Wsh, inner)),
        Script::P2SH | Script::P2SH_P2WPKH => wrap(Tag::Sh, inner),
        _ => inner,
    })
}

fn key_arg(tag: Tag) -> Node {
    Node {
        tag,
        body: Body::KeyArg { index: 0 },
    }
}

fn wrap(tag: Tag, inner: Node) -> Node {
    Node {
        tag,
        body: Body::Children(vec![inner]),
    }
}

// ───────────────────────────────────────────────────────────────────────────
// What the block and the pack need out of a built descriptor
// ───────────────────────────────────────────────────────────────────────────

/// The md1 strings `--as md1` packs, one record each. `Class::MdMk` is what
/// `sysw::classify` calls them — no new class, and the three programs that
/// admit a descriptor record admit these already.
pub fn strings(b: &Built) -> Result<Vec<String>, BuildError> {
    Ok(md_codec::split(&b.descriptor)?)
}

/// §5.4's `wallet-id:` — the WalletPolicyId over the (a′)-materialised policy,
/// as 32 lowercase hex characters.
pub fn wallet_id(b: &Built) -> Result<String, BuildError> {
    let id = md_codec::compute_wallet_policy_id(&b.descriptor)?;
    Ok(id
        .as_bytes()
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect::<String>())
}

/// The network every key shares — conjunct 5 has already refused a mixture.
pub fn network(d: &Parsed) -> bitcoin::Network {
    match d.keys.first().map(|k| k.network) {
        Some(Network::Testnet) => bitcoin::Network::Testnet,
        _ => bitcoin::Network::Bitcoin,
    }
}

/// The address at `(chain, index)`, as the operator sees it.
pub fn address(
    b: &Built,
    chain: u32,
    index: u32,
    net: bitcoin::Network,
) -> Result<String, BuildError> {
    Ok(b.descriptor
        .derive_address(chain, index, net)?
        .assume_checked()
        .to_string())
}
