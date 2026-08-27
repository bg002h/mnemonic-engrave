//! `me sysw pack --expect <kinds>` — **state what the container must hold, and
//! be refused if it does not.**
//!
//! §6g exists because a backup can be silently incomplete. `mk encode` refuses,
//! the operator's pipeline carries on, `me sysw pack` builds a container from
//! the `md1` records alone at exit 0, and a plate is cut from a wallet that
//! cannot be restored. `--expect descriptor,cosigner` is the assertion that
//! turns that into a refusal.
//!
//! ## The vocabulary, and what resolves each
//!
//! | kind | resolved by |
//! | --- | --- |
//! | `descriptor` | **HRP `'d'`** (`md1`) — *not* by `Class` |
//! | `cosigner` | **HRP `'k'`** (`mk1`) — *not* by `Class` |
//! | `transaction` | `Class::Mt` ∪ `Class::Tx`, under the caller's `Admission` |
//! | `mnemonic` | `Class::Mnemonic` |
//! | `secret` | `Class::Codex32Secret` |
//!
//! **`descriptor` and `cosigner` must not resolve through `Class`.** `me`'s
//! `Class` has a single `MdMk` variant covering both, so a `Class`-keyed
//! `--expect descriptor,cosigner` cannot tell a descriptor card from a cosigner
//! card — and it is exactly the funds case above that would slip through.
//!
//! **`address` is NOT in the vocabulary, deliberately.** `Class::Address` and
//! `Class::Descriptor` are never produced by `classify` — `me sysw pack`
//! refuses an address record outright. A kind that can never be satisfied is
//! worse than an absent one: it turns a gate into a permanent refusal.
//! `passphrase` is out for the same reason.
//!
//! **`FreeText` and `Unknown` are deliberately unnameable.** `--expect` states
//! what must be PRESENT, and neither can be required of a stream.
//!
//! ## Admission is a parameter, and omitting it creates a FALSE REFUSAL
//!
//! Built without it, `me sysw pack --allow-unsigned-inputs --expect transaction`
//! refuses at exit 4 saying *no record of that kind is in the stream* — for a
//! record the **same invocation packs at exit 0 without `--expect`**. A false
//! refusal carrying a false message, on the funds path, inside the feature
//! added to prevent exactly that. So the kind test takes the `Admission` flags.
//!
//! ## Completeness needs THREE walks, not one
//!
//! Presence is not enough: a half-transmitted set is present and useless.
//!
//! 1. the HRP walk above, for presence and for which card kind;
//! 2. [`super::record::mdmk_unconfirmed`], for `md1`/`mk1` set completeness —
//!    but it **discards the HRP**, so its indices are mapped back through
//!    [`super::record::card_hrp`] to learn which kind is broken;
//! 3. [`super::mt::mt_unconfirmed`], because walk 2 is **blind to `mt1`
//!    entirely** — it filters on `Class::MdMk`, and an `mt1` chunk is
//!    `Class::Mt`. Measured on three of six even chunks:
//!    `mdmk_unconfirmed` returns `[]` — *"nothing wrong here"* — while
//!    `mt_unconfirmed` returns `[0, 1, 2]`.
//!
//! Without walk 3, `--expect transaction` passes a half-transmitted
//! transaction as complete: §6g's own failure mode surviving inside §6g's own
//! remedy.

use super::record::{card_hrp, mdmk_unconfirmed, Class};
use super::Admission;

/// A kind a container may be required to hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Descriptor,
    Cosigner,
    Transaction,
    Mnemonic,
    Secret,
}

/// Every kind, in the order `--expect`'s help lists them.
pub const ALL: [Kind; 5] = [
    Kind::Descriptor,
    Kind::Cosigner,
    Kind::Transaction,
    Kind::Mnemonic,
    Kind::Secret,
];

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Descriptor => "descriptor",
            Kind::Cosigner => "cosigner",
            Kind::Transaction => "transaction",
            Kind::Mnemonic => "mnemonic",
            Kind::Secret => "secret",
        }
    }

    /// What an operator is looking for when they name this kind.
    pub fn describes(self) -> &'static str {
        match self {
            Kind::Descriptor => "an md1 descriptor card",
            Kind::Cosigner => "an mk1 cosigner card",
            Kind::Transaction => "a transaction (an mt1 set or a `tx:` record)",
            Kind::Mnemonic => "a BIP-39 mnemonic",
            Kind::Secret => "an ms1 codex32 secret",
        }
    }

    /// Does `record` satisfy this kind, under `adm`?
    fn matches(self, record: &str, adm: Admission) -> bool {
        match self {
            // NOT through Class -- it cannot tell 'd' from 'k'.
            Kind::Descriptor => card_hrp(record) == Some('d'),
            Kind::Cosigner => card_hrp(record) == Some('k'),
            // The union is deliberate: a transaction reaches the device either
            // as an mt1 SET of text plates or as a single `tx:` record for the
            // QR path, and an operator asking for "a transaction" means either.
            Kind::Transaction => {
                matches!(super::classify_with(record, adm), Class::Mt | Class::Tx)
            }
            Kind::Mnemonic => super::classify_with(record, adm) == Class::Mnemonic,
            Kind::Secret => super::classify_with(record, adm) == Class::Codex32Secret,
        }
    }
}

/// Why a stated expectation was not met.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unmet {
    /// Nothing in the stream is of this kind.
    Absent(Kind),
    /// Records of this kind are present, but the set does not reassemble —
    /// so what is there cannot be restored from.
    Incomplete { kind: Kind, indices: Vec<usize> },
}

/// Parse `--expect`'s comma-separated value.
///
/// Errors carry the full vocabulary, because the two most likely wrong guesses
/// — `address` and `passphrase` — are absent on purpose and an operator who
/// tries them deserves the reason rather than a bare "unknown".
pub fn parse_kinds(spec: &str) -> Result<Vec<Kind>, String> {
    let mut out = Vec::new();
    for raw in spec.split(',') {
        let word = raw.trim().to_ascii_lowercase();
        if word.is_empty() {
            continue;
        }
        let k = ALL.iter().copied().find(|k| k.name() == word);
        match k {
            Some(k) => {
                if !out.contains(&k) {
                    out.push(k);
                }
            }
            None => {
                let vocab: Vec<&str> = ALL.iter().map(|k| k.name()).collect();
                let extra = match word.as_str() {
                    "address" | "descriptors" | "addresses" => {
                        "\n      `address` is deliberately absent: `me sysw pack` cannot \
                         classify an address record at all, so requiring one would be a \
                         refusal that nothing could ever satisfy."
                    }
                    "passphrase" | "pass" => {
                        "\n      `passphrase` is deliberately absent: it cannot be \
                         satisfied on the flag path, and a kind that cannot be satisfied \
                         turns a gate into a permanent refusal."
                    }
                    "text" | "freetext" | "free-text" | "unknown" => {
                        "\n      free text is deliberately unnameable: --expect states what \
                         must be PRESENT, and free text cannot be required of a stream."
                    }
                    _ => "",
                };
                return Err(format!(
                    "unknown --expect kind {word:?}; want one or more of {}{extra}",
                    vocab.join(", ")
                ));
            }
        }
    }
    if out.is_empty() {
        return Err("--expect needs at least one kind; it cannot be empty".to_string());
    }
    Ok(out)
}

/// Check `records` against every stated `kind`. An empty result means every
/// expectation was met.
pub fn check(records: &[String], kinds: &[Kind], adm: Admission) -> Vec<Unmet> {
    // Walk 2 and walk 3, computed once each rather than per kind.
    let mdmk_bad = mdmk_unconfirmed(records);
    let mt_bad = super::mt::mt_unconfirmed(records);

    let mut out = Vec::new();
    for &kind in kinds {
        let present: Vec<usize> = records
            .iter()
            .enumerate()
            .filter(|(_, r)| kind.matches(r, adm))
            .map(|(i, _)| i)
            .collect();
        if present.is_empty() {
            out.push(Unmet::Absent(kind));
            continue;
        }
        let broken: Vec<usize> = match kind {
            // Walk 2 discards the HRP, so its indices are mapped back through
            // the discriminant to learn WHICH card kind failed to reassemble.
            Kind::Descriptor | Kind::Cosigner => {
                let want = if kind == Kind::Descriptor { 'd' } else { 'k' };
                mdmk_bad
                    .iter()
                    .copied()
                    .filter(|&i| card_hrp(&records[i]) == Some(want))
                    .collect()
            }
            // Walk 3. `mt_unconfirmed` only ever names Class::Mt records and
            // set-level problems among them, so a `tx:`-only stream leaves it
            // empty and is correctly treated as complete.
            Kind::Transaction => mt_bad.clone(),
            // Neither is chunked: a mnemonic and an ms1 are each their own
            // whole, so presence IS completeness and inventing a check here
            // would be a gate that cannot fail.
            Kind::Mnemonic | Kind::Secret => Vec::new(),
        };
        if !broken.is_empty() {
            out.push(Unmet::Incomplete {
                kind,
                indices: broken,
            });
        }
    }
    out
}

/// The refusal an [`Unmet`] deserves, wording included.
///
/// Returned rather than printed: this module decides, and the binary announces.
pub fn describe(u: &Unmet) -> String {
    match u {
        Unmet::Absent(k) => format!(
            "--expect {} was not met: NO record of that kind is in the stream.\n      \
             Looking for {}.\n      \
             Nothing was written -- a container built without it would flash and \
             engrave, and the gap would only show when someone tried to restore.",
            k.name(),
            k.describes()
        ),
        Unmet::Incomplete { kind, indices } => format!(
            "--expect {} was not met: records of that kind ARE present, but the set \
             does not reassemble.\n      \
             Unconfirmed at record {} (records count from 0).\n      \
             A partial set is not a backup: it passes every checksum it carries and \
             still cannot be restored from. Nothing was written.",
            kind.name(),
            indices
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD1_A: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
    const MT_EVEN: [&str; 6] = [
        "mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax",
        "mt1p9h8jqq9qqphgdqqqqqqqq0mllllupyqj6vqqqqqqqqzcqpfsw7ph2rt5w54kt768636cls8zxg0najlzunp",
        "mt1p9h8jqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qd8pwskxfl7ju5",
        "mt1p9h8jqq9qqrqfrnq3qzyp77h37cnxzvwutegzmzy5zrrrfvrpykdfsckvk03dcq6rcjtvlsfcglv7zx43yaz",
        "mt1p9h8jqq9qqylgpzqmhcwhuupdvnrc82rncvzzdahpgjsdwgu52jd7vmxsve9x3w5ujeqyssuvddxvwqze4ve",
        "mt1p9h8jqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq5k5y2ye5nv8yf",
    ];

    fn recs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    /// **WHY WALK 3 EXISTS, measured rather than argued.**
    ///
    /// `mdmk_unconfirmed` filters on `Class::MdMk`, and an `mt1` chunk is
    /// `Class::Mt`, so on half a transaction it answers *"nothing wrong here"*.
    /// An implementation that used it alone — which an earlier draft of the
    /// plan proposed, on the ground that it *"already groups by `(hrp,
    /// chunk_set_id)`"* — would ship an `--expect transaction` that passes a
    /// half-transmitted transaction as complete.
    ///
    /// This is pinned as an assertion rather than left in a comment so that a
    /// future change to either walk has to come past it.
    #[test]
    fn the_mdmk_walk_is_blind_to_mt1_and_that_is_why_there_are_three() {
        let half = recs(&MT_EVEN[..3]);
        assert_eq!(
            mdmk_unconfirmed(&half),
            Vec::<usize>::new(),
            "the md/mk walk sees NOTHING WRONG with half a transaction"
        );
        assert_eq!(
            super::super::mt::mt_unconfirmed(&half),
            vec![0, 1, 2],
            "and the mt walk sees all three chunks unconfirmed"
        );
    }

    /// **WHY `descriptor` AND `cosigner` DO NOT GO THROUGH `Class`.**
    /// Both card kinds collapse to the single `MdMk` variant, so a
    /// `Class`-keyed test cannot tell them apart — and `--expect
    /// descriptor,cosigner` would then be satisfied by the descriptor cards
    /// alone, which is the funds case §6g exists for.
    #[test]
    fn the_card_kinds_are_indistinguishable_by_class_and_distinct_by_hrp() {
        const MK1_A: &str = "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g";
        assert_eq!(super::super::classify(MD1_A), Class::MdMk);
        assert_eq!(
            super::super::classify(MK1_A),
            Class::MdMk,
            "one variant, two card kinds -- this is the trap"
        );
        assert_eq!(card_hrp(MD1_A), Some('d'));
        assert_eq!(card_hrp(MK1_A), Some('k'), "the HRP is what separates them");
    }

    /// The kinds that are deliberately unnameable stay unnameable, and the two
    /// likely wrong guesses each get their own reason.
    #[test]
    fn the_vocabulary_is_exactly_five_kinds() {
        assert_eq!(ALL.len(), 5);
        for bad in ["address", "passphrase", "text", "freetext", "unknown", ""] {
            assert!(
                parse_kinds(bad).is_err(),
                "{bad:?} must not be nameable -- a kind that cannot be satisfied \
                 turns a gate into a permanent refusal"
            );
        }
        assert_eq!(
            parse_kinds(" Descriptor , COSIGNER ,descriptor").unwrap(),
            vec![Kind::Descriptor, Kind::Cosigner],
            "whitespace and case are forgiven, and a repeat is not two checks"
        );
    }
}
