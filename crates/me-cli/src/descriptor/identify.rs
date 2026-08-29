//! **§5.4 — both paths parse host-side first.**
//!
//! The block prints on EVERY successful whole-input parse, BEFORE whatever
//! follows, in TWO tiers. It is what makes §5.1's no-fallback rule usable: the
//! operator can SEE — and CHECK, by one address comparison — that the thing
//! they are about to engrave is the wallet they meant, even when this build's
//! answer is a refusal.
//!
//! # The two tiers, and the one thing that decides between them
//!
//! **TIER** is decided by what does not depend on the flag: a wallet that
//! passes conjuncts 2–8 AND whose shape at least one `--as` path admits gets
//! the FULL block. Since the md1 path's shape set is the descriptor path's plus
//! the three `multi` twins, that predicate is exactly
//! `admit(d, Path::Md1).is_ok()` — one call, not a re-derivation.
//!
//! A wallet NO path admits gets the PARTIAL block: the first three lines plus
//! the watch-only line — no `wallet-id:`, no `address 0:`, no compare prompt.
//! Its class is the underivable, the unspendable, the anyone-can-spend and the
//! unmeasured, and "compare before engraving" is a wrong instruction on every
//! member. A conjunct-8 failure is PARTIAL for a sharper reason: its addresses
//! DO derive, byte-identically to a clean control, so a compare prompt would
//! PASS on an impossible wallet.
//!
//! **FOLLOWER** is decided independently by §5's own logic — the tier picks
//! lines, not outcomes, and any tier may precede any follower.

use super::admit::{self, Path};
use super::cascade::{Multi, Parsed, Script};
use super::derive;
use super::gate;
use super::md1;
use super::refusal;

/// Which `--as` value the invocation named, if any. `None` is the
/// `--as`-omitted path, whose follower is §5.1's choice block.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    Descriptor,
    Md1,
}

/// §5.4's block for `d`, already laid out. `None` is never returned for a
/// parsed descriptor — the caller decides whether to ask.
pub fn block(d: &Parsed, form: Option<Form>) -> String {
    let full = admit::admit(d, Path::Md1).is_ok();
    let mut out: Vec<String> = Vec::new();

    // 1–3: the format, the canonical string, the shape. Both tiers.
    out.push(format!("read as: {}", d.branch.operator_name()));
    out.push(format!("descriptor: {}", d.encode()));
    out.push(shape_line(d));

    if full {
        // 4: `wallet-id:` over the (a′)-materialised policy — the uniform base
        // that makes the id identical under both `--as` values. Emitted only
        // when the wallet HAS an md1 policy form; for an (a)/(a″) shape the
        // honest attempt ERRORS (`AltCountOutOfRange`) and encoding anyway
        // would collapse to a DIFFERENT wallet, whose id would then sit two
        // lines above the compare prompt identifying something else.
        match md1::build(d).and_then(|b| md1::wallet_id(&b)) {
            Ok(id) => out.push(format!("wallet-id: {id}")),
            Err(_) => out.push(
                "wallet-id: none -- this wallet has no md1 policy form; identify it by the \
                 checksum in the descriptor line and by address 0."
                    .to_string(),
            ),
        }
        // 5: `address 0:` and the compare prompt — the executable check.
        //
        // Derived KEY BY KEY (`super::derive`), which is what the device does.
        // The whole-descriptor route this used to take could not express a
        // wallet whose keys want different receive indices and printed a
        // sentence claiming the address did not exist; it does, and the device
        // derives it (IMPL-S1S3-adversarial-review I1).
        match derive::address_0(d) {
            Some(a) => {
                out.push(format!("address 0: {a}"));
                out.push(
                    "compare against your wallet software's first receive address before \
                     engraving."
                        .to_string(),
                );
            }
            // Unreachable in the FULL tier, and that is ASSERTED rather than
            // argued: `every_full_tier_wallet_has_an_address_0` runs the walk
            // over every parsed vector row whose conjuncts 2–8 hold. Kept
            // because a panic on an operator path is never the right answer,
            // and worded to claim nothing about the WALLET — the previous text
            // made three claims about it and all three were false.
            None => out.push(
                "address 0: this build could not derive one. Check the descriptor line \
                 above against your wallet software instead."
                    .to_string(),
            ),
        }
    }

    // 6: the watch-only line, owner-quotable, printed in BOTH tiers — its
    // referent is the wallet DESCRIPTION, which exists whether or not anything
    // is packed.
    out.push(
        "watch-only: public keys only -- this wallet description can SHOW its addresses \
         and balances; it cannot spend. Whoever holds it can watch the wallet -- share it \
         accordingly."
            .to_string(),
    );

    // 7: for `--as md1`, the template and the placeholder-to-fingerprint map,
    // with §5.3(a′)'s annotation whenever materialisation occurred.
    if form == Some(Form::Md1) && full {
        if let Ok(b) = md1::build(d) {
            out.push(format!("template: {}", b.template));
            out.push(format!("keys: {}", slot_map(&b)));
            if b.materialised {
                out.push(MATERIALISED_NOTE.to_string());
            }
        }
    }

    // 8: for a promoted bare key, the full §4.5 announcement. Promotion is
    // ANNOUNCED, not silent — the operator supplied one line and is getting a
    // whole wallet, and this is the last thing they read before the follower.
    if let Some(a) = gate::promotion_announcement(d) {
        out.push(a);
    }

    out.join("\n      ")
}

/// §5.3(a′)'s annotation, verbatim. An unexplained novelty at steel-imminent
/// stakes earns "is this the wrong derivation path?", so the note cites the
/// standards — an authority the operator can check — rather than the device.
///
/// Origin-family-neutral on purpose: it fires for BIP-44/49/84 promoted keys
/// and BIP-48 cosigners alike, and naming BIP-48 here would invite a check that
/// fails for half the inputs it prints on.
pub const MATERIALISED_NOTE: &str =
    "note: your input names no derivation below the key origins; `<0;1>/*` is the \
     standard receive/change continuation below such origins -- the convention your \
     wallet software already uses (in the standards: the BIP-44 family's change level, \
     and BIP-388's canonical tail). Addresses are unchanged by making it explicit.";

/// §5.3(b)'s warning, verbatim. The label is display-only on the device, so
/// this is a warning and not a refusal — and "nothing else is lost" is the
/// sentence that stops it reading like a data-loss report.
/// **The label is the operator's own bytes**, so it is quoted through
/// [`refusal::quote_operator`]: escaped and bounded. This warning sits beside
/// `address 0:` and must never be able to move it.
pub fn label_warning(label: &str) -> String {
    format!(
        "warning: the label \"{}\" is not carried by any record format and will not \
         appear on the device. Nothing else is lost.",
        refusal::quote_operator(label)
    )
}

fn shape_line(d: &Parsed) -> String {
    let script = match d.script {
        Script::P2PKH => "pkh",
        Script::P2WPKH => "wpkh",
        Script::P2SH_P2WPKH => "sh(wpkh)",
        Script::P2TR => "tr",
        Script::P2WSH => "wsh",
        Script::P2SH_P2WSH => "sh(wsh)",
        Script::P2SH => "sh",
    };
    match d.multi {
        Some(m) => format!(
            "script: {script}, {} {} of {} keys",
            m.spelling(),
            d.threshold,
            d.keys.len()
        ),
        None => format!("script: {script}, single-key"),
    }
}

fn slot_map(b: &md1::Built) -> String {
    b.slots
        .iter()
        .map(|(i, fp)| match fp {
            Some(f) => format!("@{i}={f:08x}"),
            // An all-zero fingerprint is "master unknown", not a fingerprint of
            // zero, and printing `00000000` would invite a comparison against a
            // coordinator that will never match.
            None => format!("@{i}=<no master fingerprint>"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The window refusal's TWO variants (§5.1), decided by md1-representability —
/// no refusal may point at a path that refuses in the CURRENT build.
pub fn window_refusal(d: &Parsed) -> String {
    let mut t = String::from(
        "--as descriptor is not available in this build.\n      \
         The QR plate needs device firmware this release does not include.\n      ",
    );
    // A `multi` form never reaches here: §4.7 conjunct 1 refuses it under
    // `--as descriptor` PERMANENTLY, in every build, and the admission refusal
    // precedes the window. The window text's "come back for the QR plate" would
    // be false forever for a shape the descriptor record can never carry.
    debug_assert!(!matches!(d.multi, Some(Multi::Unsorted)));
    let offenders = admit::md1_offenders(d);
    if offenders.is_empty() {
        t.push_str(
            "Available now: --as md1 -- me converts and packs in one step: error-corrected \
             text cards, restored by transcription instead of scanning. Your export file is \
             all you need to come back for the QR plate later; nothing is lost by waiting.",
        );
        return t;
    }
    t.push_str("--as md1 cannot carry this wallet either -- ");
    t.push_str(
        &offenders
            .iter()
            .map(|(i, path)| format!("key `@{i}` uses `{path}`"))
            .collect::<Vec<_>>()
            .join(", and "),
    );
    t.push_str(
        ". No path in this build engraves this file. It loses nothing by waiting: keep it, \
         and it packs the day the device update ships.",
    );
    t
}
