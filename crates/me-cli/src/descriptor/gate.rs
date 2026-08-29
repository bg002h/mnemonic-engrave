//! **§5.1 — the whole-input discriminator and the descriptor-shape gate.**
//!
//! When `--as` is absent and record classification fails, `me` consults this
//! gate, and re-reads the whole input through §4's cascade ONLY when the gate
//! opens — when the input is DESCRIPTOR-SHAPED.
//!
//! **The gate keeps two promises at once**, and an implementation is conformant
//! exactly when both hold:
//!
//! 1. **No record-shaped input ever hears descriptor vocabulary or a changed
//!    exit code.** Well-formed records of the six shapes the shipped classifier
//!    admits, and mistyped attempts at them, keep the SHIPPED refusal
//!    unchanged: exit 4, record vocabulary.
//! 2. **Every admitted descriptor spelling reaches the descriptor surfaces.**
//!    §4's four formats, all fifteen §4.5 rows, all five of conjunct 7's
//!    use-site spellings, and descriptor content buried behind other records.
//!
//! **The precision is §7's 37 `gate`-tagged vector rows, which are NORMATIVE:
//! where any reading of the shape tests below disagrees with a gate row, the
//! row is the answer.** `tests/descriptor_seam.rs` runs every one of them
//! against the real `--as`-omitted invocation.
//!
//! The four shape tests are §5.1's non-normative implementation guidance, T1–T3
//! applied to EVERY line and T4 to the WHOLE input — the per-line scope is what
//! lets a buried descriptor open the gate.

use super::admit::{self, Path};
use super::cascade::{
    self, Bip380Error, BlueWalletError, Branch, Errors, JsonError, KeyError, Multi, Parsed,
    PromotionError,
};
use super::refusal::{self, Refusal};

// ───────────────────────────────────────────────────────────────────────────
// What THIS BUILD carries
// ───────────────────────────────────────────────────────────────────────────

/// Whether `--as descriptor` has shipped. **False for the whole S3 release**:
/// §5.2's record needs a `sysw.Classify` descriptor arm the device does not
/// have, so a `Descriptor` record packed today would be `ClassUnknown` on the
/// machine. F-418 parks it with S2.
pub const DESCRIPTOR_PATH_SHIPPED: bool = false;

/// Whether `--as md1` has shipped. True: it is what S3 IS.
///
/// **It is true from P1 while the FLAG itself lands in P2.1**, which is a real
/// intra-branch window and is named here rather than left to be discovered: on
/// this commit the choice block offers `--as md1` and `me` does not yet accept
/// the flag. The alternative — computing carriage from the current tree — makes
/// §7's seven `as-decides` gate rows unsatisfiable in P1, which the plan's own
/// P1 gate forbids. See design/agent-reports/IMPL-P1-report.md, finding F-3.
pub const MD1_PATH_SHIPPED: bool = true;

// ───────────────────────────────────────────────────────────────────────────
// The outcome
// ───────────────────────────────────────────────────────────────────────────

/// What the `--as`-omitted invocation does with this input — §7's `outcome`
/// column, as a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The gate stayed CLOSED. The shipped record-classification refusal
    /// stands, unchanged, in record vocabulary.
    RecordRefusal,
    /// §5.1's choice block: the input IS a descriptor and at least one `--as`
    /// value carries it in this build.
    ///
    /// It carries no payload: §4.5's promotion announcement is the LAST element
    /// of §5.4's identification block, which prints on every successful
    /// whole-input parse and therefore precedes this follower too.
    AsDecides,
    /// A §6 row.
    DescriptorRefusal(Refusal),
    /// §6's multi-record row — the whole input is not one descriptor, but one
    /// of its records is.
    MultiRecord(Refusal),
}

impl Outcome {
    /// §7's `outcome` value.
    pub fn class(&self) -> &'static str {
        match self {
            Self::RecordRefusal => "record-refusal",
            Self::AsDecides => "as-decides",
            Self::DescriptorRefusal(_) => "descriptor-refusal",
            Self::MultiRecord(_) => "multi-record",
        }
    }

    /// §7's `gate_open` value. The equivalence with the outcome class is the
    /// two invariants' two halves, which is why it is derived here rather than
    /// tracked separately and allowed to disagree.
    pub fn gate_open(&self) -> bool {
        !matches!(self, Self::RecordRefusal)
    }

    /// §7's `refusal_row` value, `None` where the outcome names no §6 row.
    pub fn refusal_row(&self) -> Option<&'static str> {
        self.refusal().map(|r| r.row.slug())
    }

    pub fn refusal(&self) -> Option<&Refusal> {
        match self {
            Self::DescriptorRefusal(r) | Self::MultiRecord(r) => Some(r),
            _ => None,
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// The shape gate
// ───────────────────────────────────────────────────────────────────────────

/// T1 — a line whose FIRST TOKEN is an identifier immediately followed by `(`.
///
/// A script expression, not any parenthesis: `text: my wallet (2 of 3)` stays a
/// record. The identifier is `[A-Za-z][A-Za-z0-9_]*`; the underscore is in so a
/// bare miniscript fragment (`or_d(…)`) reaches §6's miniscript row rather than
/// the record refusal, while the wrapper-prefixed `v:pkh(…)` fails on every
/// reading, its identifier being followed by `:`.
fn t1_script_expression(line: &str) -> bool {
    let tok = line.split_whitespace().next().unwrap_or("");
    let mut chars = tok.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    for c in chars {
        if c == '(' {
            return true;
        }
        if !(c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
    }
    false
}

/// T2 — a line whose `": "` key is a BlueWallet header or an 8-hex
/// fingerprint. A bare `": "` is NOT enough, or `seed:`/`text:`/`pass:`-style
/// records would hear descriptor vocabulary.
fn t2_bluewallet_line(line: &str) -> bool {
    match cascade::header_key(line) {
        Some(k) => cascade::BW_HEADERS.contains(&k) || cascade::is_8_hex(k),
        None => false,
    }
}

/// T3 — a line that is a SINGLE TOKEN beginning with `[`, or whose leading
/// segment before any `/` is a 78-byte base58check payload.
///
/// That second disjunct covers the origin-annotated key AND the keyed-no-origin
/// spellings like `xpub…/<0;1>/*`, which is all fifteen §4.5 rows. The
/// single-token requirement is what keeps `text: <a real xpub>` a record.
fn t3_key_shaped(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() || t.split_whitespace().nth(1).is_some() {
        return false;
    }
    t.starts_with('[') || cascade::looks_like_an_extended_key(t)
}

/// T4 — a WHOLE input that is JSON with a descriptor field. Whole-input, not
/// per-line, because a pretty-printed export is one document across many lines.
fn t4_json_document(document: &str) -> bool {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(document) else {
        return false;
    };
    let Some(o) = v.as_object() else {
        return false;
    };
    o.keys().any(|k| k.eq_ignore_ascii_case("descriptor"))
}

/// Whether §5.1's gate opens for this input.
pub fn gate_opens(document: &str) -> bool {
    if t4_json_document(document) {
        return true;
    }
    document
        .split('\n')
        .any(|l| t1_script_expression(l) || t2_bluewallet_line(l) || t3_key_shaped(l))
}

// ───────────────────────────────────────────────────────────────────────────
// The discriminator
// ───────────────────────────────────────────────────────────────────────────

/// §5.1's whole-input discriminator, run AFTER record classification has
/// failed.
///
/// `document` is the invocation's whole input; `records` is the same input as
/// the shipped record stream, which is what §6's multi-record row indexes into.
pub fn consult(document: &str, records: &[String]) -> Outcome {
    let doc = cascade::normalise(document);
    if !gate_opens(&doc) {
        return Outcome::RecordRefusal;
    }
    match cascade::cascade(&doc) {
        // The whole input IS one descriptor.
        Ok(d) => carriage(&d),
        Err(errs) => {
            // A descriptor among OTHER records. Naming `--as` here would send
            // the operator to a whole-file read that refuses with a message
            // false about the file.
            for (i, r) in records.iter().enumerate() {
                if cascade::cascade(&cascade::normalise(r)).is_ok() {
                    return Outcome::MultiRecord(refusal::multi_record_descriptor(i));
                }
            }
            // Neither parse succeeded: §6's cause selection, over the WHOLE
            // input. A refused promotion buried among records lands here, and
            // §6 states that outcome explicitly.
            Outcome::DescriptorRefusal(select_cause(&doc, &errs))
        }
    }
}

/// §5.4's carriage rule. The choice block fires when at least one `--as` value
/// carries this input IN THIS BUILD; when neither does, the input's own refusal
/// fires directly — never a two-option menu whose options both refuse.
fn carriage(d: &Parsed) -> Outcome {
    let descriptor_carries = DESCRIPTOR_PATH_SHIPPED && admit::admit(d, Path::Descriptor).is_ok();
    let md1_admits = admit::admit(d, Path::Md1);
    let representable = admit::md1_representable(d, &remedy_fixed_index(d), &remedy_no_wildcard(d));
    let md1_carries = MD1_PATH_SHIPPED && md1_admits.is_ok() && representable.is_ok();

    if descriptor_carries || md1_carries {
        return Outcome::AsDecides;
    }
    // §4.7's admission refusal PRECEDES anything about a flag or a build: a
    // wallet no path admits has a PERMANENT status, and possibly a funds-urgent
    // one. The md1 path's admission is the wider of the two, so its refusal is
    // the one that is true of the input rather than of a flag.
    if let Err(r) = md1_admits {
        return Outcome::DescriptorRefusal(r);
    }
    if let Err(r) = representable {
        return Outcome::DescriptorRefusal(r);
    }
    // Admitted and representable, and still carried by nothing — reachable only
    // in a build where the md1 path has not shipped either.
    Outcome::DescriptorRefusal(Refusal::new(
        refusal::Row::WindowNotInBuild,
        "no `me` path in this build engraves this wallet. It loses nothing by \
         waiting: keep your export file, and it packs the day the update ships.",
    ))
}

/// §5.3's WINDOW SUBSTITUTION, both rows, stated once.
///
/// Three texts, and which one fires is decided by two facts about the input and
/// the build — never by a preference:
///
/// * a `multi` form is carried by NO path in ANY build (§4.7 conjunct 1 refuses
///   it under `--as descriptor` permanently, and md1 cannot represent this
///   use-site), so it gets §6's own neither-path replacement. The window
///   substitution is EXEMPT here: "wait for the update" would be false forever,
///   and a refusal that routes nowhere has nothing to replace;
/// * with the descriptor path SHIPPED, §6's own remedy — it names the flag that
///   carries this exact shape;
/// * without it, §5.3's stock replacement, verbatim. It DESCRIBES the path's
///   future availability without ROUTING to it, which is what the rule
///   distinguishes.
fn window_remedy(d: &Parsed, path: &str, sorted_alternative: &str) -> String {
    if matches!(d.multi, Some(Multi::Unsorted)) {
        return format!(
            "This is a `multi` policy, which only `--as md1` carries -- and md1 cannot \
             represent `{path}`. No `me` path engraves this file as written, in any \
             build. Re-export with `<0;1>/*` -- carried in every build. {sorted_alternative}"
        );
    }
    if DESCRIPTOR_PATH_SHIPPED {
        return format!("Use `--as descriptor`, which carries `{path}` exactly.");
    }
    "The scannable-plate path is not in this build -- keep the export file; it packs \
     when the device update ships."
        .to_string()
}

/// The offending path §5.3's remedy names — the FIRST offender's, matching the
/// row text, which names every offender but takes one remedy.
fn first_offender_path(d: &Parsed, want_fixed: bool) -> String {
    use cascade::Derivation::*;
    d.keys
        .iter()
        .find(|k| {
            if want_fixed {
                matches!(k.children.as_slice(), [Child { .. }, Wildcard { .. }])
            } else {
                matches!(k.children.as_slice(), [Range { .. }])
            }
        })
        .map(|k| k.children.iter().map(|c| c.encode()).collect::<String>())
        .unwrap_or_default()
}

/// §5.3(a) — the `/i/*` row's remedy.
pub fn remedy_fixed_index(d: &Parsed) -> String {
    window_remedy(
        d,
        &first_offender_path(d, true),
        "(Re-exporting as a `sortedmulti` policy keeps the fixed index but is a \
         DIFFERENT policy -- `me` will not rewrite it -- and needs the scannable-plate \
         path.)",
    )
}

/// §5.3(a″) — the `<i;i+1>`-without-wildcard row's remedy.
pub fn remedy_no_wildcard(d: &Parsed) -> String {
    window_remedy(
        d,
        &first_offender_path(d, false),
        "(Re-exporting as a `sortedmulti` policy keeps the multipath but is a DIFFERENT \
         policy -- `me` will not rewrite it -- and needs the scannable-plate path.)",
    )
}

/// §4.5, NORMATIVE: promotion is **announced, not silent**. The operator
/// supplied one line and is getting a whole wallet.
///
/// It prints BOTH forms (R0's I5): the canonical re-encoding rebuilds an xpub's
/// depth and child-number bytes from the invented origin path, so for any key
/// whose true depth is not 3 the inferred descriptor contains a base58 string
/// the operator has never seen — and the one check the announcement exists for
/// is "is that my key?".
pub fn promotion_announcement(d: &Parsed) -> Option<String> {
    if !d.promoted {
        return None;
    }
    let k = d.keys.first()?;
    Some(format!(
        "this is a single extended key, and `me` inferred a whole wallet from it:\n      \
         key as supplied: {}\n      \
         inferred wallet: {}\n      \
         The key's serialisation was normalised (version and depth bytes rebuilt from \
         the origin path); the key material itself is unchanged.",
        k.as_supplied,
        d.encode()
    ))
}

// ───────────────────────────────────────────────────────────────────────────
// §6's cause selection
// ───────────────────────────────────────────────────────────────────────────

/// Which §6 row fires for an input no branch admitted.
///
/// The five-step rule ranks CASCADE failures only; it reads the WHOLE input,
/// deliberately unlike §5.1's per-line gate, because the refusal describes the
/// file the operator gave. Where the branch's own error names a §6 row, that
/// row fires; where it does not, §6 row 1's four-forms text carries the
/// branch's real reason — which is the diagnostic the device destroys.
pub fn select_cause(document: &str, errs: &Errors) -> Refusal {
    // §6's address row, ahead of the five-step rule: an address matches none of
    // the four branches, so the rule would report step 5's generic four-forms
    // text and bury the one fact the operator needs — that no program on the
    // device consumes an address record at all.
    if cascade::is_bitcoin_address(document) {
        return refusal::bitcoin_address();
    }
    match cascade::most_resembled(document) {
        Some(Branch::Json) => match &errs.json {
            Some(JsonError::Inner { label, inner }) => {
                refusal::json_inner_malformed(label, &describe_bip380(inner))
            }
            _ => refusal::unparseable(Some((
                Branch::Json,
                "the document is not a `{label, descriptor}` object".to_string(),
            ))),
        },
        Some(Branch::BlueWallet) => match &errs.bluewallet {
            Some(e) => bluewallet_row(e),
            None => refusal::unparseable(None),
        },
        Some(Branch::Bip380) => match &errs.bip380 {
            Some(e) => bip380_row(e),
            None => refusal::unparseable(None),
        },
        Some(Branch::PromotedKey) => match &errs.promotion {
            Some(e) => promotion_row(document, e),
            None => refusal::unparseable(None),
        },
        None => refusal::unparseable(None),
    }
}

fn bluewallet_row(e: &BlueWalletError) -> Refusal {
    use BlueWalletError as E;
    match e {
        E::NoName { headers, cosigners } => refusal::bluewallet_no_name(headers, *cosigners),
        E::ZeroCosigners => refusal::bluewallet_zero_cosigners(),
        E::PolicyCount {
            declared,
            found,
            policy,
        } => refusal::bluewallet_policy_count(policy, *declared, *found),
        E::NoFormat => refusal::bluewallet_no_format(),
        E::NoOrigin {
            fingerprint,
            after_keys,
        } => refusal::bluewallet_no_origin(*fingerprint, *after_keys),
        E::BadFingerprint(line) => refusal::bluewallet_bad_fingerprint(line),
        other => refusal::unparseable(Some((Branch::BlueWallet, describe_bluewallet(other)))),
    }
}

fn bip380_row(e: &Bip380Error) -> Refusal {
    match e {
        Bip380Error::Key(KeyError::UnsupportedVersion {
            version,
            converted,
            origin,
        }) => refusal::unsupported_key_version(*version, converted, origin.as_deref()),
        Bip380Error::UnknownScriptType(name) if is_miniscript_fragment(name) => {
            refusal::miniscript(name)
        }
        other => refusal::unparseable(Some((Branch::Bip380, describe_bip380(other)))),
    }
}

fn promotion_row(document: &str, e: &PromotionError) -> Refusal {
    use PromotionError as E;
    match e {
        E::Key(KeyError::FingerprintNoPath) => refusal::promotion_fingerprint_no_path(document),
        E::Key(KeyError::UnsupportedVersion {
            version,
            converted,
            origin,
        }) => refusal::unsupported_key_version(*version, converted, origin.as_deref()),
        E::Key(other) => refusal::unparseable(Some((Branch::PromotedKey, describe_key(other)))),
        E::PathNotInferable(k) => refusal::promotion_path_not_inferable(
            k,
            refusal::suggested_descriptor_for(&k.origin, k).as_deref(),
        ),
        E::AccountNotZero(k) => refusal::promotion_account_not_zero(
            k,
            refusal::suggested_descriptor_for(&k.origin, k).as_deref(),
        ),
        E::MultisigCosignerKey(v) => refusal::promotion_multisig_cosigner_key(*v),
        E::TestnetKey => refusal::promotion_testnet_key(),
    }
}

/// The miniscript fragment identifiers, so a miniscript descriptor reaches §6's
/// own row rather than the generic four-forms text. Everything here is a
/// fragment name the device's `bip380.Parse` will report as an unknown script
/// type.
fn is_miniscript_fragment(name: &str) -> bool {
    const FRAGMENTS: &[&str] = &[
        "pk",
        "pkh",
        "pk_k",
        "pk_h",
        "older",
        "after",
        "sha256",
        "hash256",
        "ripemd160",
        "hash160",
        "andor",
        "and_v",
        "and_b",
        "and_n",
        "or_b",
        "or_c",
        "or_d",
        "or_i",
        "thresh",
        "multi_a",
        "sortedmulti_a",
    ];
    FRAGMENTS.contains(&name)
}

// ───────────────────────────────────────────────────────────────────────────
// Branch error text — the reason the device DISCARDS
// ───────────────────────────────────────────────────────────────────────────

fn describe_bluewallet(e: &BlueWalletError) -> String {
    use BlueWalletError as E;
    match e {
        E::InvalidHeaderLine(l) => {
            format!("a line is not `Key: value`: `{}`", refusal::short_key(l))
        }
        E::InconsistentHeader(k) => format!("the `{k}` header appears twice with different values"),
        E::InvalidPolicy(v) => format!("the `Policy:` header is not `k of n`: `{v}`"),
        E::InvalidDerivation(v) => format!("the `Derivation:` header is not a path: `{v}`"),
        E::UnknownFormat(v) => {
            format!("`Format: {v}` is not one of `P2WSH`, `P2SH`, `P2WSH-P2SH`")
        }
        E::InvalidCosignerKey(v) => {
            format!(
                "a cosigner line's value is not an extended key: `{}`",
                refusal::short_key(v)
            )
        }
        // The six rows above are the ones with no §6 row of their own; the rest
        // never reach this function.
        _ => "the BlueWallet file was refused".to_string(),
    }
}

fn describe_bip380(e: &Bip380Error) -> String {
    match e {
        Bip380Error::InvalidChecksum => "invalid checksum".to_string(),
        Bip380Error::MissingOpenParen => "script: missing `(`".to_string(),
        Bip380Error::MissingCloseParen => "script: missing `)`".to_string(),
        Bip380Error::UnknownScriptType(s) => format!("unknown script type: `{s}`"),
        Bip380Error::InvalidWrappedScriptType(s) => format!("invalid wrapped script type: `{s}`"),
        Bip380Error::InvalidThreshold(_) => "invalid multikey threshold".to_string(),
        Bip380Error::Key(k) => describe_key(k),
    }
}

fn describe_key(e: &KeyError) -> String {
    match e {
        KeyError::MissingCloseBracket => {
            "an origin block opens with `[` and never closes".to_string()
        }
        KeyError::FingerprintNoPath => {
            "the origin block gives a fingerprint with no derivation path".to_string()
        }
        KeyError::InvalidFingerprint => {
            "the origin block's fingerprint is not 8 hex characters".to_string()
        }
        KeyError::InvalidOriginPath(p) => format!(
            "the origin path is not a path: `{}`",
            refusal::quote_operator(p)
        ),
        // N1 (IMPL-P1's review): the offending tail can be the EMPTY string --
        // `xpub…/` -- and the general sentence then printed empty backticks:
        // "the use-site path is not a path: ``." Correct row, correct exit code,
        // and a sentence that reads as a bug.
        KeyError::InvalidChildrenPath(p) if p.is_empty() => {
            "the key ends in `/` with no use-site path after it".to_string()
        }
        KeyError::InvalidChildrenPath(p) => format!(
            "the use-site path is not a path: `{}`",
            refusal::quote_operator(p)
        ),
        KeyError::NotAnExtendedKey => "it is not an extended key".to_string(),
        KeyError::UnsupportedVersion { version, .. } => {
            format!(
                "`{}` is not a version the device admits",
                version.spelling()
            )
        }
        KeyError::UnknownVersion(v) => format!("the version bytes `{v:08x}` name no known key"),
        KeyError::NotAPublicPoint => "the key bytes are not a public key on the curve".to_string(),
    }
}

// ───────────────────────────────────────────────────────────────────────────
// §5.1's choice block
// ───────────────────────────────────────────────────────────────────────────

/// §5.1's `--as`-omitted text, at `EXIT_USAGE`. **It states the CHOICE**,
/// because an operator holding a wallet export does not know which they want.
///
/// A value the BUILD does not carry is marked inline, so the block never offers
/// a build-dead flag unmarked. A value this particular INPUT does not carry is
/// deliberately left unmarked: the operator who picks it gets that path's own
/// refusal, which names the working flag.
pub fn choice_block() -> String {
    let descriptor_head = if DESCRIPTOR_PATH_SHIPPED {
        "      --as descriptor"
    } else {
        "      --as descriptor (not available in this build)"
    };
    let md1_head = if MD1_PATH_SHIPPED {
        "      --as md1          "
    } else {
        "      --as md1 (not available in this build)"
    };
    format!(
        "this input is a wallet descriptor, and `--as` decides how it is packed.\n\
{descriptor_head}\n\
\x20                       the SCANNABLE plate. The device engraves the wallet\n\
\x20                       as a QR that any phone or wallet app can read -- no\n\
\x20                       special tooling to restore, ever. Packed in CANONICAL\n\
\x20                       form (SLIP-132 versions become xpub, ' becomes h,\n\
\x20                       checksum recomputed). The engraver itself cannot\n\
\x20                       read the QR back (it has no camera).\n\
{md1_head}the HAND-COPYABLE plate. me converts the descriptor\n\
\x20                       and packs error-corrected md1 text cards in ONE step\n\
\x20                       (no md invocation needed). Restored by transcription;\n\
\x20                       each string survives up to 4 MIS-STRUCK characters\n\
\x20                       (substitutions -- a missing or extra strike is not\n\
\x20                       correctable), so it can even be hand-stamped. Carries\n\
\x20                       policies --as descriptor cannot. Restoring needs an\n\
\x20                       md1 decoder (an open spec; the tooling today is this\n\
\x20                       project's).\n\
\x20   They are not interchangeable -- `me sysw pack --help` has the comparison."
    )
}
