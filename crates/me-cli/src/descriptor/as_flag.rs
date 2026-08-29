//! **§5.1 — `--as` is required, and the invocation it appears in is
//! SINGLE-DOCUMENT.**
//!
//! `--in`'s shipped contract is newline-separated records, which can never
//! carry a multi-line BlueWallet file or pretty-printed JSON. So `--as` changes
//! the input contract of the invocation it appears in: exactly one descriptor,
//! read WHOLE.
//!
//! The order every `--as` invocation follows is §5.4's, and it is the order the
//! operator reads:
//!
//! 1. the host-side parse — nothing ships bytes the host has not understood;
//! 2. §5.4's identification block, in its tier;
//! 3. §5.3(b)'s label warning, where it applies;
//! 4. the FOLLOWER — a §4.7 admission refusal, a §5.3 refusal, §5.1's window
//!    refusal, or the pack.
//!
//! **The admission refusal PRECEDES the window refusal.** A wallet no path
//! admits has a PERMANENT status and possibly a funds-urgent one:
//! `sortedmulti(0,…)` must hear "treat those funds as at risk now", never
//! "nothing is lost by waiting".

use super::admit::{self, Path};
use super::cascade;
use super::gate;
use super::identify::{self, Form};
use super::md1;
use super::refusal::{Refusal, Row};

/// What the invocation does after the block has printed.
pub enum Decision {
    /// The md1 strings to pack, one record each — `Class::MdMk`.
    Pack(Vec<String>),
    /// One or more §6 rows, at `EXIT_REFUSED`. More than one is §6's
    /// both-rows-fire case: a descriptor mixing an (a)-shaped and an
    /// (a″)-shaped key matches both §5.3 rows, both are true, and both name the
    /// same remedy.
    Refused(Vec<Refusal>),
}

/// Everything one `--as` invocation prints and decides.
pub struct Run {
    /// stderr lines, in order, before the decision is acted on. The caller
    /// prefixes each with `me: `.
    pub notes: Vec<String>,
    pub decision: Decision,
}

/// §5.1's single-document contract, as a usage check over the invocation shape.
///
/// This is the `--as`-PRESENT half of §5.1 that P1 could not implement, having
/// no flag to hang it on.
pub fn single_document_error(argv_records: usize, has_in: bool) -> Option<&'static str> {
    if argv_records > 1 || (argv_records >= 1 && has_in) {
        return Some("--as packs exactly one descriptor per invocation.");
    }
    None
}

/// Run one `--as` invocation over the whole document.
pub fn run(form: Form, document: &str) -> Run {
    let doc = cascade::normalise(document);
    let d = match cascade::cascade(&doc) {
        Ok(d) => d,
        // The cascade failed. `--as` declared the input single-document, so
        // §5.1's shape gate has nothing left to decide and §6's five-step cause
        // selection reports over the whole input directly.
        Err(errs) => {
            return Run {
                notes: Vec::new(),
                decision: Decision::Refused(vec![gate::select_cause(&doc, &errs)]),
            }
        }
    };

    let mut notes = vec![identify::block(&d, Some(form))];
    let decision = match form {
        Form::Md1 => md1_follower(&d),
        Form::Descriptor => descriptor_follower(&d),
    };

    // §5.3(b)'s label warning follows the block — and fires EXACTLY on the
    // paths that pack (controller ruling, IMPL-S1S3-adversarial-review M3).
    //
    // Its own text is a statement about what was just packed: *"is not carried
    // by any record format and will not appear on the device. Nothing else is
    // lost."* On a refusal nothing was carried anywhere, so the sentence
    // describes an event that did not happen — and "nothing else is lost" is
    // actively wrong next to a refusal, where the whole wallet was lost. The
    // implementation previously printed it on the `--as descriptor` window
    // refusal too, and never on the `--as`-omitted choice block: three paths,
    // three different answers. One rule now: it follows a PACK.
    if matches!(decision, Decision::Pack(_)) {
        if let Some(label) = d.title.as_deref().filter(|l| !l.is_empty()) {
            notes.push(identify::label_warning(label));
        }
    }
    Run { notes, decision }
}

fn md1_follower(d: &cascade::Parsed) -> Decision {
    if let Err(r) = admit::admit(d, Path::Md1) {
        return Decision::Refused(vec![r]);
    }
    let rs = admit::md1_refusals(
        d,
        &gate::remedy_fixed_index(d),
        &gate::remedy_no_wildcard(d),
    );
    if !rs.is_empty() {
        return Decision::Refused(rs);
    }
    match md1::build(d).and_then(|b| md1::strings(&b)) {
        Ok(s) => Decision::Pack(s),
        // Unreachable from an admitted, representable descriptor: every conjunct
        // the codec enforces has already run host-side, conjunct 8 included.
        // Reported as the closed-set row rather than panicking, because a panic
        // on an operator path is never the right answer to a surprise.
        Err(e) => Decision::Refused(vec![Refusal::new(
            Row::UseSiteOutOfSet,
            format!("this wallet could not be encoded as md1 text cards: {e}."),
        )]),
    }
}

fn descriptor_follower(d: &cascade::Parsed) -> Decision {
    // §4.7 FIRST, in every build — conjunct 1's `multi` refusal included, which
    // is PERMANENT and must never be dressed as a wait.
    if let Err(r) = admit::admit(d, Path::Descriptor) {
        return Decision::Refused(vec![r]);
    }
    // The wallet the descriptor path WOULD carry in a full build. When
    // `DESCRIPTOR_PATH_SHIPPED` becomes true (S2, F-418), this is where §5.2's
    // canonical `Descriptor` record is packed instead.
    Decision::Refused(vec![Refusal::new(
        Row::WindowNotInBuild,
        identify::window_refusal(d),
    )])
}
