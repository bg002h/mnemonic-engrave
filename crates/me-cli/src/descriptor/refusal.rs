//! **§6 — what the operator SEES.**
//!
//! The device's parser has exactly one message for eleven distinct causes
//! (§4.1). `me` has one per cause, and this module is the cause taxonomy:
//! [`Row`] is §6's table as a closed enum, and [`Refusal`] pairs a row with the
//! text that row prints for THIS input.
//!
//! **The slug vocabulary is fixed on disk**, in
//! `testdata/descriptor_seam_vectors.json`'s `refusal_rows` map (36 entries,
//! one per §6 data row). `tests/descriptor_seam.rs` asserts that [`Row::ALL`]'s
//! slugs are exactly that set, so the gate rows' `refusal_row` field and P2.4's
//! per-row text tests cannot drift apart by inventing two names for one
//! refusal (PLAN-r4's NEW-M6).
//!
//! **Two rules bind every text here**, from `SPEC_constellation_cli_uniformity`
//! and the walk's W5 verdict:
//!
//! * the remedy is EXECUTABLE — where a row says "supply the descriptor", it
//!   prints the descriptor with the operator's own key and origin substituted
//!   in, never a placeholder;
//! * the text leads with the verdict and contains NO internal identifiers — no
//!   phase labels, no F-numbers, no `§` references inside the quoted span.
//!   Those live in the doc comment beside the constructor, which is where a
//!   reader who needs them is looking anyway.

use super::cascade::{Branch, Derivation, Key, KeyVersion, Multi, Network, Parsed, Script};

/// One row of §6's table.
///
/// All 35 are named, and each has a test in `tests/descriptor_refusals.rs` that
/// reaches it and asserts its TEXT: the vocabulary and the row tests are one
/// set, and a half-vocabulary is what PLAN-r4's NEW-M6 predicted would drift.
///
/// **S2 SUBTRACTED one.** `WindowNotInBuild` said "`--as descriptor` is not
/// available in this build"; with the descriptor path shipped no build state
/// refuses, so §5.1's window has no trigger left and the row retired with it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    Unparseable,
    EmptyFile,
    WhitespaceOnly,
    AsOmitted,
    JsonInnerMalformed,
    BlueWalletNoName,
    BlueWalletNoFormat,
    BlueWalletZeroCosigners,
    BlueWalletPolicyCount,
    BlueWalletNoOrigin,
    BlueWalletBadFingerprint,
    MultiUnderDescriptor,
    Miniscript,
    Md1FixedIndex,
    ThresholdExceedsKeys,
    ThresholdBelowOne,
    KeyCountExceeded,
    KeyIdentity,
    KeyIdentityDuplicate,
    MixedNetwork,
    UnsupportedKeyVersion,
    TaprootMultisig,
    PromotionPathNotInferable,
    PromotionAccountNotZero,
    PromotionFingerprintNoPath,
    PromotionMultisigCosignerKey,
    PromotionTestnetKey,
    BitcoinAddress,
    MultiInSingleKeyScript,
    KeyInScriptSlot,
    UseSiteHardened,
    UseSiteNonConsecutive,
    UseSiteOutOfSet,
    Md1NoWildcard,
    MultiRecordDescriptor,
}

impl Row {
    pub const ALL: &'static [Row] = &[
        Row::Unparseable,
        Row::EmptyFile,
        Row::WhitespaceOnly,
        Row::AsOmitted,
        Row::JsonInnerMalformed,
        Row::BlueWalletNoName,
        Row::BlueWalletNoFormat,
        Row::BlueWalletZeroCosigners,
        Row::BlueWalletPolicyCount,
        Row::BlueWalletNoOrigin,
        Row::BlueWalletBadFingerprint,
        Row::MultiUnderDescriptor,
        Row::Miniscript,
        Row::Md1FixedIndex,
        Row::ThresholdExceedsKeys,
        Row::ThresholdBelowOne,
        Row::KeyCountExceeded,
        Row::KeyIdentity,
        Row::KeyIdentityDuplicate,
        Row::MixedNetwork,
        Row::UnsupportedKeyVersion,
        Row::TaprootMultisig,
        Row::PromotionPathNotInferable,
        Row::PromotionAccountNotZero,
        Row::PromotionFingerprintNoPath,
        Row::PromotionMultisigCosignerKey,
        Row::PromotionTestnetKey,
        Row::BitcoinAddress,
        Row::MultiInSingleKeyScript,
        Row::KeyInScriptSlot,
        Row::UseSiteHardened,
        Row::UseSiteNonConsecutive,
        Row::UseSiteOutOfSet,
        Row::Md1NoWildcard,
        Row::MultiRecordDescriptor,
    ];

    /// The slug the shared vector file names this row by.
    pub fn slug(self) -> &'static str {
        match self {
            Row::Unparseable => "unparseable",
            Row::EmptyFile => "empty-file",
            Row::WhitespaceOnly => "whitespace-only",
            Row::AsOmitted => "as-omitted",
            Row::JsonInnerMalformed => "json-inner-malformed",
            Row::BlueWalletNoName => "bluewallet-no-name",
            Row::BlueWalletNoFormat => "bluewallet-no-format",
            Row::BlueWalletZeroCosigners => "bluewallet-zero-cosigners",
            Row::BlueWalletPolicyCount => "bluewallet-policy-count",
            Row::BlueWalletNoOrigin => "bluewallet-no-origin",
            Row::BlueWalletBadFingerprint => "bluewallet-bad-fingerprint",
            Row::MultiUnderDescriptor => "multi-under-descriptor",
            Row::Miniscript => "miniscript",
            Row::Md1FixedIndex => "md1-fixed-index",
            Row::ThresholdExceedsKeys => "threshold-exceeds-keys",
            Row::ThresholdBelowOne => "threshold-below-one",
            Row::KeyCountExceeded => "key-count-exceeded",
            Row::KeyIdentity => "key-identity",
            Row::KeyIdentityDuplicate => "key-identity-duplicate",
            Row::MixedNetwork => "mixed-network",
            Row::UnsupportedKeyVersion => "unsupported-key-version",
            Row::TaprootMultisig => "taproot-multisig",
            Row::PromotionPathNotInferable => "promotion-path-not-inferable",
            Row::PromotionAccountNotZero => "promotion-account-not-zero",
            Row::PromotionFingerprintNoPath => "promotion-fingerprint-no-path",
            Row::PromotionMultisigCosignerKey => "promotion-multisig-cosigner-key",
            Row::PromotionTestnetKey => "promotion-testnet-key",
            Row::BitcoinAddress => "bitcoin-address",
            Row::MultiInSingleKeyScript => "multi-in-single-key-script",
            Row::KeyInScriptSlot => "key-in-script-slot",
            Row::UseSiteHardened => "use-site-hardened",
            Row::UseSiteNonConsecutive => "use-site-non-consecutive",
            Row::UseSiteOutOfSet => "use-site-out-of-set",
            Row::Md1NoWildcard => "md1-no-wildcard",
            Row::MultiRecordDescriptor => "multi-record-descriptor",
        }
    }
}

/// A refusal: which §6 row fired, and the text that row prints for this input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub row: Row,
    pub text: String,
}

impl Refusal {
    pub fn new(row: Row, text: impl Into<String>) -> Self {
        Self {
            row,
            text: text.into(),
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Shared substitution helpers
// ───────────────────────────────────────────────────────────────────────────

/// Elide the middle of a long base58 string so a refusal stays readable while
/// still showing enough for "is that my key?" to be answerable.
pub fn short_key(s: &str) -> String {
    if s.chars().count() <= 24 {
        return s.to_string();
    }
    let head: String = s.chars().take(14).collect();
    let tail: String = s
        .chars()
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{head}…{tail}")
}

/// **Operator-supplied bytes, rendered safe for a terminal and BOUNDED.**
///
/// Every refusal that quotes the operator's own file back at them runs the
/// fragment through this. It escapes four classes and bounds the result at
/// [`QUOTE_MAX`] CHARACTERS:
///
/// 1. **C0/C1 controls and DEL** — an ESC sequence next to `address 0:` could
///    scroll away the very line the operator is being told to compare (M2).
/// 2. **The two DELIMITERS this is interpolated into**, `"` and `` ` ``. A
///    label of `ok" -- nothing is wrong with this wallet. "` closed the quote
///    and continued in `me`'s own voice; a use-site fragment containing a
///    backtick left an apparently unterminated pair on the line. Escaping both
///    at the source is what makes ONE helper safe for BOTH call sites
///    (fold-1 re-review M-A(a) and N-a).
/// 3. **The escape character itself**, `\`, or a literal `\x1b` in a label
///    would be indistinguishable from an escape this function emitted.
/// 4. **Invisible and directional formatting** — Unicode `Cf` and the private
///    use areas. `char::is_control()` covers `Cc` ONLY, so U+202E and friends
///    passed through raw: `a\u{202E}KCATTA` renders as `aATTACK` in a
///    bidi-aware terminal, and an unterminated override reorders the rest of
///    the line (M-A(b)).
///
/// **What is deliberately NOT escaped:** ordinary non-ASCII. A wallet named
/// `Grüße — Konto Nº1 ✓ 日本語` must come back recognisable, because the
/// operator's job here is to recognise their own label. Escaping by CLASS
/// rather than by an ASCII allowlist is what buys that.
///
/// This is output INTEGRITY, not secret handling: nothing here decides what is
/// packed, and the bytes are public wallet-export text.
pub fn quote_operator(s: &str) -> String {
    let mut out = String::new();
    let mut width = 0usize;
    for c in s.chars() {
        if width >= QUOTE_MAX {
            out.push('…');
            break;
        }
        let piece = match c {
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            '"' => "\\\"".to_string(),
            '`' => "\\`".to_string(),
            '\\' => "\\\\".to_string(),
            c if (c.is_control() || is_invisible_or_directional(c)) && (c as u32) < 0x80 => {
                format!("\\x{:02x}", c as u32)
            }
            c if c.is_control() || is_invisible_or_directional(c) => {
                format!("\\u{{{:04x}}}", c as u32)
            }
            c => c.to_string(),
        };
        width += piece.chars().count();
        out.push_str(&piece);
    }
    out
}

/// The bound on a quoted fragment, in CHARACTERS.
///
/// Not columns: `width` accumulates `chars().count()`, so 48 CJK glyphs occupy
/// roughly 96 terminal columns (fold-1 re-review N-b — the doc said columns and
/// the code counted chars). Still far short of a screen, and still enough to
/// recognise your own file.
pub const QUOTE_MAX: usize = 48;

/// Unicode `Cf` (format) plus the private use areas — the characters that are
/// invisible or that reorder what follows them, and which `char::is_control()`
/// does not reach.
///
/// Hard-coded because `std` exposes no general-category table and this does not
/// warrant a dependency. The list is the `Cf` ranges plus PUA; each line names
/// what it is, so a reader can check it rather than trust it. Erring wide is
/// safe here — a false positive escapes a character instead of hiding it.
fn is_invisible_or_directional(c: char) -> bool {
    matches!(c as u32,
        0x00AD                          // SOFT HYPHEN
        | 0x0600..=0x0605 | 0x061C | 0x06DD | 0x070F | 0x08E2  // Arabic format
        | 0x180E                        // MONGOLIAN VOWEL SEPARATOR
        | 0x200B..=0x200F               // ZWSP, ZWNJ, ZWJ, LRM, RLM
        | 0x202A..=0x202E               // bidi embedding / override
        | 0x2060..=0x2064               // word joiner, invisible operators
        | 0x2066..=0x206F               // bidi ISOLATES, deprecated formats
        | 0xFEFF                        // ZWNBSP / BOM
        | 0xFFF9..=0xFFFB               // interlinear annotation
        | 0x110BD | 0x110CD             // Kaithi number signs
        | 0x13430..=0x1343F             // Egyptian hieroglyph format
        | 0x1BCA0..=0x1BCA3             // Shorthand format
        | 0x1D173..=0x1D17A             // Musical beam/slur/phrase
        | 0xE0001 | 0xE0020..=0xE007F   // language tag, TAG characters
        | 0xE000..=0xF8FF               // private use area
        | 0xF0000..=0xFFFFD             // supplementary PUA-A
        | 0x100000..=0x10FFFD           // supplementary PUA-B
    )
}

/// `[fp/path]` as the input spelled it, or the empty string for a key that
/// declared no origin.
pub fn origin_prefix(k: &Key) -> String {
    if k.fingerprint == 0 && !k.origin_explicit {
        return String::new();
    }
    format!(
        "[{:08x}{}]",
        k.fingerprint,
        super::cascade::path_encode(&k.origin)
    )
}

/// The key expression as the operator wrote it, with only the base58 payload
/// elided: `[4bbaa801/86h/0h/0h]xpub6C9j4wAxxkW…coGnx`.
///
/// The origin block is kept VERBATIM and the use-site tail is dropped, because
/// every §6 row that names a key names the offending path separately in the
/// same sentence. Built from `as_supplied` rather than from the parsed fields
/// so the operator sees their own spelling — a first draft prepended
/// [`origin_prefix`] to the whole of `as_supplied` and printed the origin
/// twice, caught by reading the emitted text rather than by any assertion.
pub fn key_display(k: &Key) -> String {
    let s = k.as_supplied.as_str();
    let (origin, rest) = match s.strip_prefix('[').and_then(|r| r.split_once(']')) {
        Some((o, rest)) => (format!("[{o}]"), rest),
        None => (String::new(), s),
    };
    let key = rest.split('/').next().unwrap_or(rest);
    format!("{origin}{}", short_key(key))
}

/// How a refusal names a key slot: `@N` plus enough of the key to recognise it.
fn key_name(i: usize, k: &Key) -> String {
    format!("@{i} ({})", key_display(k))
}

fn children_string(children: &[Derivation]) -> String {
    if children.is_empty() {
        return "no use-site path".to_string();
    }
    children.iter().map(|d| d.encode()).collect()
}

/// **§6's `multi`-class device-clause rule (spec amendment 2026-08-29).**
///
/// `bip380.Parse`'s script switch has a `sortedmulti` case and NO `multi` case,
/// so the device refuses every `multi` form at PARSE and none of them ever
/// reaches address derivation. Six §6 rows carry a sentence about what the
/// device does with the descriptor, and for a `multi` input each of those six
/// says this instead.
///
/// The rows: the single-key wrapper, `tr(…(…))`, the mixed network, the
/// hardened use-site, the non-consecutive multipath and the key count. Five of
/// them printed a MEASURABLY FALSE claim before the amendment — IMPL-P1's
/// review M1 constructed all six by rewriting `sortedmulti(` to `multi(` and
/// ran them.
const MULTI_AT_PARSE: &str =
    "The device's parser refuses `multi` outright, so this file never reaches address derivation there.";

/// Which device sentence a row prints, given the descriptor's multi form.
/// `None` (single-key) and `Some(Sorted)` both take the measured sentence.
fn device_clause(m: Option<Multi>, sorted: &str) -> String {
    match m {
        Some(Multi::Unsorted) => MULTI_AT_PARSE.to_string(),
        _ => sorted.to_string(),
    }
}

/// The replacement clause with its first letter lowered, for the rows that
/// splice it mid-sentence.
fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

// ───────────────────────────────────────────────────────────────────────────
// §4.2 — the BlueWallet rows
// ───────────────────────────────────────────────────────────────────────────

/// §6 row 6. **The enumeration is substituted, not fixed.** §6 spells it "it
/// has `Policy`, `Derivation` and `Format` headers and `N` cosigner lines", and
/// the gate row that reaches it (`gate/deadbeef-fronts-an-xpub`) is a ONE-LINE
/// file with none of those headers — so a fixed enumeration would be false
/// about the operator's own file, which is the defect §6 exists to remove.
pub fn bluewallet_no_name(headers: &[&'static str], cosigners: usize) -> Refusal {
    let what = if headers.is_empty() {
        format!(
            "it has {cosigners} cosigner line{}",
            if cosigners == 1 { "" } else { "s" }
        )
    } else {
        format!(
            "it has {} header{} and {cosigners} cosigner line{}",
            headers
                .iter()
                .map(|h| format!("`{h}`"))
                .collect::<Vec<_>>()
                .join(", "),
            if headers.len() == 1 { "" } else { "s" },
            if cosigners == 1 { "" } else { "s" }
        )
    };
    Refusal::new(
        Row::BlueWalletNoName,
        format!(
            "this is a BlueWallet setup file -- {what} -- but no `Name:` header, \
             and the device requires one. Add a line `Name: <anything>`."
        ),
    )
}

/// §6 row 7. §4.2 defect 1: without `Format:` the device's `Script` stays
/// Unknown and its own `Encode()` PANICS.
pub fn bluewallet_no_format() -> Refusal {
    Refusal::new(
        Row::BlueWalletNoFormat,
        "this BlueWallet setup file has no `Format:` header, so the script type is \
         undefined. Add `Format: P2WSH` (or `P2SH`, or `P2WSH-P2SH`).",
    )
}

/// §6 row 8 — F-419, written from the walk.
pub fn bluewallet_zero_cosigners() -> Refusal {
    Refusal::new(
        Row::BlueWalletZeroCosigners,
        "this BlueWallet file has headers but no cosigner lines \
         (`<8-hex-fingerprint>: <xpub>`). There is no wallet here to pack -- was the \
         export truncated? Re-export from the coordinator.",
    )
}

/// §6 row 10.
pub fn bluewallet_policy_count(policy: &str, declared: usize, found: usize) -> Refusal {
    Refusal::new(
        Row::BlueWalletPolicyCount,
        format!(
            "`Policy: {policy}` declares {declared} cosigners; the file has {found}. \
             Cosigner lines are `<8-hex-fingerprint>: <xpub>`."
        ),
    )
}

/// §6 row 11 — R0's C1. Stated over the KEYS, so it catches a file with no
/// `Derivation:` header at all as well as one where the header follows the
/// cosigner lines.
pub fn bluewallet_no_origin(fingerprint: u32, after_keys: bool) -> Refusal {
    let where_ = if after_keys {
        "the `Derivation:` header appears after the cosigner lines"
    } else {
        "the `Derivation:` header is missing"
    };
    Refusal::new(
        Row::BlueWalletNoOrigin,
        format!(
            "cosigner `{fingerprint:08x}` has no derivation path -- {where_}. The \
             descriptor this file produces cannot be re-read by the device. Put \
             `Derivation: <path>` above the first cosigner line."
        ),
    )
}

/// §6 row 12 — §4.2 defect 4. The device PANICS on fewer than 4 bytes, so this
/// file must never reach it.
pub fn bluewallet_bad_fingerprint(line: &str) -> Refusal {
    Refusal::new(
        Row::BlueWalletBadFingerprint,
        format!(
            "cosigner line `{}` -- a master fingerprint is exactly 8 hex characters \
             (4 bytes).",
            elide_line(line)
        ),
    )
}

fn elide_line(line: &str) -> String {
    match line.split_once(": ") {
        Some((k, v)) => format!("{k}: {}", short_key(v)),
        None => short_key(line),
    }
}

// ───────────────────────────────────────────────────────────────────────────
// §4.1/§4.3/§4.4 — cascade failures
// ───────────────────────────────────────────────────────────────────────────

/// §6 row 1. `most` is the branch the input most RESEMBLES, by §6's five-step
/// rule; `None` is step 5, where the "looks most like" sentence is dropped
/// rather than guessed at.
pub fn unparseable(most: Option<(Branch, String)>) -> Refusal {
    let mut t = String::from(
        "this is not a wallet descriptor in any of the four forms `me` reads: a \
         BlueWallet `Key: value` setup file, a plain BIP-380 descriptor, a \
         `{\"label\":…,\"descriptor\":…}` JSON export, or a single extended key.",
    );
    if let Some((b, why)) = most {
        t.push_str(&format!(
            " It looks most like {}, which failed because: {why}.",
            b.operator_name()
        ));
    }
    Refusal::new(Row::Unparseable, t)
}

/// §6 row 5. The wrapper is named, then the inner error, then where the problem
/// is — the one format with a useful error message today.
pub fn json_inner_malformed(label: &str, inner: &str) -> Refusal {
    Refusal::new(
        Row::JsonInnerMalformed,
        format!(
            "the `{{label, descriptor}}` JSON parsed, and its `descriptor` field did \
             not: {inner}. The label was \"{label}\". The problem is in the descriptor \
             string, not the JSON."
        ),
    )
}

/// §6 row 14. `md encode` accepts miniscript TEMPLATES — a different tool and
/// input form, which is why the remedy names it and does not point at `me`.
pub fn miniscript(fragment: &str) -> Refusal {
    Refusal::new(
        Row::Miniscript,
        format!(
            "`me` reads the descriptor family the device reads: single-sig and \
             `sortedmulti`, optionally under `sh`. This descriptor uses miniscript \
             fragments (`{fragment}`), which neither path handles in this release. \
             `md encode` accepts miniscript TEMPLATES -- a different tool and input \
             form."
        ),
    )
}

/// §6 row 22. **The remedy names the PER-VERSION target** (R0 r2's NEW-I3 — one
/// template cannot serve five): four of the five are TESTNET keys, and an
/// `xpub` remedy would name a mainnet wallet the operator does not hold.
/// The per-version descriptor §6's SLIP-132 row tells the operator to run.
/// Extracted for the same reason as [`remedy_full_origin`].
///
/// **The script is the VERSION's, never a guess**: `ypub` is BIP-49
/// (`sh(wpkh)`), `vpub` is testnet BIP-84 (`wpkh`), and `Upub`/`Vpub` are
/// testnet MULTISIG accounts with no single-key form at all. One template
/// cannot serve five, and four of the five are testnet — an `xpub` remedy would
/// name a mainnet wallet the operator does not hold.
pub fn remedy_for_version(v: KeyVersion, converted: &str, origin: Option<&str>) -> String {
    match v {
        KeyVersion::Ypub => match origin {
            // With an origin the operator's own fingerprint and path carry over.
            Some(o) => format!("sh(wpkh({o}{converted}/<0;1>/*))"),
            // Handing back a bare converted key would PROMOTE to a different
            // wallet (`pkh(…)`, measured), so the bare remedy is the
            // origin-less descriptor spelling, which the device admits.
            None => format!("sh(wpkh({converted}/<0;1>/*))"),
        },
        KeyVersion::Upub => match origin {
            Some(o) => format!("sh(wpkh({o}{converted}/<0;1>/*))"),
            None => format!("sh(wpkh({converted}/<0;1>/*))"),
        },
        KeyVersion::Vpub => match origin {
            Some(o) => format!("wpkh({o}{converted}/<0;1>/*)"),
            None => format!("wpkh({converted}/<0;1>/*)"),
        },
        // `Upub`/`Vpub` are testnet MULTISIG accounts: no single-key remedy
        // exists, so the remedy is the full multisig spelling.
        KeyVersion::UpubCap => format!(
            "sh(wsh(sortedmulti(<k>,{}{converted}/<0;1>/*,<the other cosigners>)))",
            origin.unwrap_or("")
        ),
        KeyVersion::VpubCap => format!(
            "wsh(sortedmulti(<k>,{}{converted}/<0;1>/*,<the other cosigners>))",
            origin.unwrap_or("")
        ),
        _ => converted.to_string(),
    }
}

pub fn unsupported_key_version(v: KeyVersion, converted: &str, origin: Option<&str>) -> Refusal {
    let remedy = remedy_for_version(v, converted, origin);
    let target = match v {
        KeyVersion::Ypub => "xpub",
        _ => "tpub",
    };
    Refusal::new(
        Row::UnsupportedKeyVersion,
        format!(
            "`me` admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`. This \
             key is `{}`, whose equivalent is `{target}`: {remedy}",
            v.spelling()
        ),
    )
}

// ───────────────────────────────────────────────────────────────────────────
// §4.5 — the promotion rows
// ───────────────────────────────────────────────────────────────────────────

/// §6 row 24. The path is quoted back and the three that qualify are listed,
/// then the descriptor is printed with the operator's own key substituted in.
pub fn promotion_path_not_inferable(k: &Key, suggested: Option<&str>) -> Refusal {
    let tail = match suggested {
        Some(d) => format!(
            "{}, which is not inferable. Supply the descriptor instead: {d}",
            purpose_gloss(&k.origin)
        ),
        // M2: no script is named, and the refusal says WHY rather than guessing
        // one. An unrecognised purpose step carries no script `me` can infer,
        // and the key's own version byte is not authority enough to invent one.
        None => ", whose first step is not one `me` recognises -- it names no script, \
             and `me` will not guess which wallet you meant. Supply the descriptor \
             your wallet software shows, with this key and this origin in it."
            .to_string(),
    };
    Refusal::new(
        Row::PromotionPathNotInferable,
        format!(
            "`{}` is a single extended key. `me` can infer a whole wallet from one \
             only when its origin is `m/44h/0h/0h` (-> `pkh`), `m/84h/0h/0h` \
             (-> `wpkh`) or `m/49h/0h/0h` (-> `sh(wpkh)`). This one is `{}`{tail}",
            key_display(k),
            super::cascade::path_string(&k.origin)
        ),
    )
}

/// §6 row 25 — §4.5's measured live near-miss.
pub fn promotion_account_not_zero(k: &Key, suggested: Option<&str>) -> Refusal {
    // The purpose step is one of the inferable three by construction -- this
    // error only fires after the purpose matched -- so the remedy is always
    // nameable here and the `None` arm is defensive, not a second M2 case.
    let tail = match suggested {
        Some(d) => format!("Supply the descriptor: {d}"),
        None => "Supply the descriptor your wallet software shows.".to_string(),
    };
    Refusal::new(
        Row::PromotionAccountNotZero,
        format!(
            "`{}` is a single extended key, and this one is `{}`. Only account 0 is \
             inferable. {tail}",
            key_display(k),
            super::cascade::path_string(&k.origin)
        ),
    )
}

/// §6 row 26.
/// The full-origin key §6's fingerprint-no-path row tells the operator to run.
/// Extracted so the §6h gate can feed it back through the cascade rather than
/// scraping it out of a sentence.
pub fn remedy_full_origin(supplied: &str) -> String {
    let fp = supplied
        .strip_prefix('[')
        .and_then(|s| s.split_once(']'))
        .map(|(f, _)| f.to_string())
        .unwrap_or_default();
    let key = supplied
        .split_once(']')
        .map(|(_, k)| k.to_string())
        .unwrap_or_else(|| supplied.to_string());
    format!("[{fp}/84h/0h/0h]{key}")
}

pub fn promotion_fingerprint_no_path(supplied: &str) -> Refusal {
    let fp = supplied
        .strip_prefix('[')
        .and_then(|s| s.split_once(']'))
        .map(|(f, _)| f.to_string())
        .unwrap_or_default();
    let key = supplied
        .split_once(']')
        .map(|(_, k)| k.to_string())
        .unwrap_or_else(|| supplied.to_string());
    Refusal::new(
        Row::PromotionFingerprintNoPath,
        format!(
            "`[{fp}]{}` gives a fingerprint with no derivation path, so there is \
             nothing to match a script against. Either give the full origin -- \
             `{}` -- or drop the brackets entirely, in which case the key's version \
             byte decides.",
            short_key(&key),
            remedy_full_origin(supplied)
        ),
    )
}

/// §6 row 27 — forms per `Script.DerivationPath()`.
pub fn promotion_multisig_cosigner_key(v: KeyVersion) -> Refusal {
    let (path, form) = match v {
        KeyVersion::ZpubCap => ("m/48h/0h/0h/2h", "wsh(sortedmulti(…))"),
        _ => ("m/48h/0h/0h/1h", "sh(wsh(sortedmulti(…)))"),
    };
    Refusal::new(
        Row::PromotionMultisigCosignerKey,
        format!(
            "a `{}` declares a MULTISIG account (`{path}`). A multisig cosigner key is \
             not a wallet -- supply the full descriptor (`{form}`), or a BlueWallet \
             setup file listing every cosigner.",
            v.spelling()
        ),
    )
}

/// §6 row 28 — §4.5's ruling, and the reason is stated rather than implied.
pub fn promotion_testnet_key() -> Refusal {
    Refusal::new(
        Row::PromotionTestnetKey,
        "this is a testnet key. Its version byte would map to the MAINNET path \
         `m/44h/0h/0h`, which `me` will not assume. Supply the descriptor with its \
         real origin.",
    )
}

// ───────────────────────────────────────────────────────────────────────────
// §4.7 — the admission rows
// ───────────────────────────────────────────────────────────────────────────

/// §6 row 30. **The remedy differs by multi form** (R0 r6's NEW-M4, r7's
/// NEW-I2): all three single-key `multi` twins are device REFUSE at PARSE, so
/// the device-measurement parenthetical does not transpose, and the remedy must
/// name the working flag rather than the invocation that just refused.
pub fn multi_in_single_key_script(m: Multi) -> Refusal {
    let t = match m {
        Multi::Sorted => "a multisig policy cannot live inside a single-key script. The \
             device's parser accepts this spelling and then cannot derive any address \
             from it (measured: `address: multisig script: … unsupported descriptor`). \
             The forms the device derives are `wsh(sortedmulti(…))`, \
             `sh(wsh(sortedmulti(…)))` and `sh(sortedmulti(…))`."
            .to_string(),
        Multi::Unsorted => {
            "a multisig policy cannot live inside a single-key script on EITHER path. \
             Change the wrapper -- `wsh(multi(…))`, `sh(multi(…))` or \
             `sh(wsh(multi(…)))` -- and use `--as md1`, which carries those forms."
                .to_string()
        }
    };
    Refusal::new(Row::MultiInSingleKeyScript, t)
}

/// §6 row 31 — both measured `Supported=false` with no derivable address.
pub fn key_in_script_slot() -> Refusal {
    Refusal::new(
        Row::KeyInScriptSlot,
        "`wsh`/`sh` of a single key is not a wallet form the device can derive \
         addresses for (measured: `Supported=false`, `address: singlesig script: … \
         unsupported descriptor`). A single-key wallet is `pkh(…)`, `wpkh(…)`, \
         `sh(wpkh(…))` or `tr(…)`.",
    )
}

/// §6 row 23.
pub fn taproot_multisig(m: Multi) -> Refusal {
    Refusal::new(
        Row::TaprootMultisig,
        format!(
            "taproot multisig is `multi_a`/`sortedmulti_a` (BIP-387); `tr({}(…))` is not \
             a valid descriptor {}. Check the export.",
            m.spelling(),
            match m {
                Multi::Sorted => "even though the device's parser accepts it".to_string(),
                Multi::Unsorted => format!(
                    "-- and {}",
                    lower_first(MULTI_AT_PARSE.trim_end_matches('.'))
                ),
            }
        ),
    )
}

/// §6 row 13 — conjunct 1's PERMANENT refusal under `--as descriptor`, in every
/// build.
pub fn multi_under_descriptor() -> Refusal {
    Refusal::new(
        Row::MultiUnderDescriptor,
        "the device's descriptor parser accepts `sortedmulti` and not `multi`. This \
         wallet can still be engraved: `--as md1` encodes `multi` policies (for \
         use-site paths md1 can represent -- otherwise no path carries it, and the \
         refusal says so). (`sortedmulti` differs from `multi` only in key ordering \
         at spend time -- it is not a synonym, so `me` will not rewrite it for you.)",
    )
}

/// §6 row 16 — conjunct 2, the unsatisfiable half.
pub fn threshold_exceeds_keys(k: i64, n: usize) -> Refusal {
    Refusal::new(
        Row::ThresholdExceedsKeys,
        format!(
            "threshold {k} of {n} keys can never be satisfied -- no combination of \
             signatures reaches {k}. Funds sent to this wallet would be unspendable. \
             Nothing was packed."
        ),
    )
}

/// §6 row 17 — conjunct 2, the spendable-by-anyone half. The device derives a
/// real address for `k = 0` and even `k = −1`, so this refusal is the host's
/// alone, and it is the one refusal in §6 that tells the operator to act NOW.
pub fn threshold_below_one(k: i64) -> Refusal {
    Refusal::new(
        Row::ThresholdBelowOne,
        format!(
            "threshold {k} means NO signature is required: anyone who can see this \
             script can spend from it. This is almost certainly not the wallet you \
             meant -- and if it already holds funds, treat them as at risk now. \
             Nothing was packed."
        ),
    )
}

/// §6 row 18 — conjunct 3, with the bound corrected per R0 r2's NEW-I2: the
/// 520-byte limit binds only where the multi's own output script IS the
/// redeemScript.
pub fn key_count_exceeded(n: usize, form: &str, m: Option<Multi>) -> Refusal {
    Refusal::new(
        Row::KeyCountExceeded,
        format!(
            "`sh({}(…))` carries at most 15 keys -- there the multi's output \
             script IS the redeemScript, one 520-byte script element (BIP-383). \
             `wsh(…)` and `sh(wsh(…))` carry at most 20 (`OP_CHECKMULTISIG`); their \
             redeemScript is 34 bytes and the 520-byte limit never binds. This \
             descriptor has {n} keys under `{form}`. {}",
            // §6's transposition rule: the form name substitutes. The BOUND
            // does not -- it is the redeemScript's, not the ordering's (§4.7
            // conjunct 3), which is why one sentence serves both forms.
            m.map(|m| m.spelling()).unwrap_or("sortedmulti"),
            device_clause(
                m,
                "The device would accept it and derive addresses whose coins cannot be spent."
            )
        ),
    )
}

/// §6 row 21 — conjunct 5. The device accepts the descriptor and then cannot
/// derive any address from it.
pub fn mixed_network(i: usize, k: &Key, first: &Key, m: Option<Multi>) -> Refusal {
    let name = |k: &Key| {
        if k.network == Network::Testnet {
            "`tpub` (testnet)"
        } else {
            "`xpub` (mainnet)"
        }
    };
    Refusal::new(
        Row::MixedNetwork,
        format!(
            "key {i} is {} while key 0 is {}. {} All keys must share one network.",
            name(k),
            name(first),
            device_clause(
                m,
                "The device accepts this descriptor and then cannot derive any address \
                 from it."
            )
        ),
    )
}

/// §6 row 19 — conjunct 8's origin contradiction. One origin identifies exactly
/// one key, so such a description matches no wallet at all.
///
/// **The cause clause is stated over ENTRIES, not over BlueWallet LINES** (spec
/// amendment 2026-08-29, from IMPL-P1's review M5). This row cannot fire for a
/// BlueWallet file: every key there shares the single `Derivation:` header, so
/// two keys carry the same `(fingerprint, origin)` iff they carry the same
/// header key — which the device's own `seenKeys` map catches FIRST as
/// `inconsistent header value`, failing branch 1 into §6 row 1. It IS reachable
/// from a BIP-380 descriptor and from the JSON wrapper, where two key
/// expressions may carry one origin block and different keys.
pub fn key_identity(i: usize, j: usize, origin: &str) -> Refusal {
    Refusal::new(
        Row::KeyIdentity,
        format!(
            "this wallet description contradicts itself: keys {i} and {j} both claim \
             origin `{origin}` but name different keys -- one origin identifies \
             exactly one key, so no wallet matches this description. Check the export: \
             one of the two entries carries the wrong key, and a copied-and-edited \
             cosigner is the usual cause."
        ),
    )
}

/// §6 row 20 — conjunct 8's duplicate slot, split from row 19 per PLAN-r3's I3
/// because "no wallet matches" is FALSE for a duplicate.
///
/// The stakes clause is PLAN-r4's NEW-M4, folded here rather than left for the
/// operator to infer: one key seated twice lets its holder produce two of the
/// required signatures, so a 2-of-3 is really 1-of-2 for that holder.
pub fn key_identity_duplicate(i: usize, j: usize) -> Refusal {
    Refusal::new(
        Row::KeyIdentityDuplicate,
        format!(
            "keys {i} and {j} are the same key at the same derivation -- a threshold \
             that needs the same key twice is not the multisig this file describes, \
             and it lets one holder produce two of the required signatures. Remove \
             the duplicate line, or supply the missing cosigner's key."
        ),
    )
}

/// §6 row 32 — conjunct 7. Hardened derivation from an xpub is impossible; the
/// device silently derives the UNhardened child and displays addresses for a
/// wallet that cannot exist.
pub fn use_site_hardened(m: Option<Multi>) -> Refusal {
    Refusal::new(
        Row::UseSiteHardened,
        format!(
            "a hardened use-site step cannot be derived from an xpub (BIP-32). {}",
            match m {
                Some(Multi::Unsorted) =>
                    format!("{MULTI_AT_PARSE} This is refused on both `--as` paths regardless."),
                _ => "The device would silently derive the UNhardened child and display \
                      addresses for a wallet that cannot exist, so this is refused on \
                      both `--as` paths."
                    .to_string(),
            }
        ),
    )
}

/// §6 row 33 — conjunct 7. The device parses it, and then errors on every
/// address.
pub fn use_site_non_consecutive(m: Option<Multi>) -> Refusal {
    Refusal::new(
        Row::UseSiteNonConsecutive,
        format!(
            "the device derives only `<i;i+1>` pairs (receive; change). {}",
            device_clause(
                m,
                "It accepts this descriptor and then errors on every address."
            )
        ),
    )
}

/// §6 row 34 — conjunct 7's closed set. "accepts" not "packs": admission is
/// build-independent, and which flag packs which member is §5.3's business.
pub fn use_site_out_of_set(got: &[Derivation]) -> Refusal {
    Refusal::new(
        Row::UseSiteOutOfSet,
        format!(
            "use-site paths `me` ACCEPTS: absent, `/*`, `/i/*`, `<i;i+1>`, \
             `<i;i+1>/*`. This one is `{}`, outside the set the device is measured to \
             handle.",
            children_string(got)
        ),
    )
}

// ───────────────────────────────────────────────────────────────────────────
// §5.3 — md1 representability
// ───────────────────────────────────────────────────────────────────────────

/// §6 row 15 — §5.3(a). `remedy` is the window substitution: in a build with
/// no `--as descriptor` path, "use `--as descriptor`" would point the operator
/// at a flag that refuses.
/// **Every offending slot is named**, per §5.3's per-key quantifier: a
/// descriptor may mix admitted members, and a refusal naming only the first
/// would send the operator back for a second refusal after one edit.
pub fn md1_fixed_index(slots: &[usize], keys: &[Key], remedy: &str) -> Refusal {
    Refusal::new(
        Row::Md1FixedIndex,
        format!(
            "md1 cannot carry this wallet as written: {}, a single fixed chain index, \
             which has no md1 form -- encoding it would silently produce a DIFFERENT \
             wallet. {remedy}",
            offending_keys(slots, keys)
        ),
    )
}

/// §6 row 35 — §5.3(a″). Encoding it would silently produce the `<0;1>/*`
/// wallet, which derives DIFFERENT addresses.
pub fn md1_no_wildcard(slots: &[usize], keys: &[Key], remedy: &str) -> Refusal {
    Refusal::new(
        Row::Md1NoWildcard,
        format!(
            "md1 cannot carry this wallet as written: {} with no trailing wildcard, \
             which has no md1 form -- encoding it would silently produce the `<0;1>/*` \
             wallet, which derives DIFFERENT addresses. {remedy}",
            offending_keys(slots, keys)
        ),
    )
}

/// `key @0 ([fp/path]xpub…) uses `/0/*`` — one clause per offender.
fn offending_keys(slots: &[usize], keys: &[Key]) -> String {
    slots
        .iter()
        .map(|i| {
            format!(
                "key {} uses `{}`",
                key_name(*i, &keys[*i]),
                children_string(&keys[*i].children)
            )
        })
        .collect::<Vec<_>>()
        .join(", and ")
}

/// §6 row 29. The reasoning — why an address is not a thing to engrave — is
/// §10's, and it stays OUTSIDE the quote per the walk-W5 rule.
pub fn bitcoin_address() -> Refusal {
    Refusal::new(
        Row::BitcoinAddress,
        "that is a bitcoin address, not a descriptor. No program on the device \
         consumes an address record.",
    )
}

// ───────────────────────────────────────────────────────────────────────────
// §5.1 — the multi-record split
// ───────────────────────────────────────────────────────────────────────────

/// §6 row 36. Applies ONLY when the whole input does not parse as one
/// descriptor: naming `--as` here would send the operator to a whole-file read
/// that refuses with a message false about the file.
pub fn multi_record_descriptor(index: usize) -> Refusal {
    Refusal::new(
        Row::MultiRecordDescriptor,
        format!(
            "record {index} is a wallet descriptor. A descriptor is packed ALONE: run \
             `me sysw pack --as <descriptor|md1>` with just the descriptor -- one \
             container cannot yet carry a descriptor plus other records. The other \
             records pack without `--as`, as usual."
        ),
    )
}

/// The descriptor `me` suggests for a bare key its promotion refused — the
/// executable half of §6's promotion rows.
pub fn suggested_descriptor_for(path: &[u32], k: &Key) -> Option<String> {
    let script = purpose_script(path)?;
    // §6h: the remedy must be EXECUTABLE. It carries the operator's own key IN
    // FULL — [`key_display`]'s elision is for text that NAMES a key, never for
    // text they are told to run, and an elided key in a "supply the descriptor"
    // line is a placeholder wearing their fingerprint.
    let key = format!(
        "{}{}/<0;1>/*",
        origin_prefix(k),
        k.as_supplied
            .rsplit(']')
            .next()
            .unwrap_or(&k.as_supplied)
            .split('/')
            .next()
            .unwrap_or_default()
    );
    Some(match script {
        Script::P2PKH => format!("pkh({key})"),
        Script::P2WPKH => format!("wpkh({key})"),
        Script::P2SH_P2WPKH => format!("sh(wpkh({key}))"),
        Script::P2TR => format!("tr({key})"),
        Script::P2WSH => format!("wsh(sortedmulti(<k>,{key},<the other cosigners>))"),
        Script::P2SH_P2WSH => format!("sh(wsh(sortedmulti(<k>,{key},<the other cosigners>)))"),
        Script::P2SH => format!("sh(sortedmulti(<k>,{key},<the other cosigners>))"),
    })
}

/// **The script a purpose step implies, or `None` when `me` will not guess.**
///
/// The mapping is HARDENED-ONLY and the set is closed. The previous `_ =>
/// P2PKH` fallback swallowed everything else — including an UNHARDENED purpose,
/// a real coordinator export bug — and handed the operator a LEGACY P2PKH
/// remedy for a native-segwit key (measured on
/// `promotion/11-origin-unhardened-refused`: `[4bbaa801/84/0/0]zpub…` got
/// `pkh(…)`). The remedy was executable and admitted, so §6h was satisfied and
/// the one test that looks at these remedies passed.
///
/// Refusing to guess is the constellation's own precedent (F-417's md1-narrow
/// ruling: refuse and name, rather than widen to fit a use site). The cost is
/// one sentence; the alternative is a harder restore from a plate cut to a
/// script nobody chose.
fn purpose_script(path: &[u32]) -> Option<Script> {
    let first = path.first()?;
    // Unhardened purposes fall out here rather than through `wrapping_sub`:
    // `m/84/0/0` is NOT `m/84h/0h/0h`, and treating them as one is the guess.
    if *first < super::cascade::HARDENED {
        return None;
    }
    Some(match first - super::cascade::HARDENED {
        44 => Script::P2PKH,
        49 => Script::P2SH_P2WPKH,
        84 => Script::P2WPKH,
        86 => Script::P2TR,
        45 => Script::P2SH,
        48 => {
            if path.len() == 4 && path[3] == super::cascade::HARDENED + 1 {
                Script::P2SH_P2WSH
            } else {
                Script::P2WSH
            }
        }
        _ => return None,
    })
}

/// §6's gloss on a recognised-but-not-inferable purpose — the spec's own
/// *"— taproot single-sig"* on the `m/86'/0'/0'` row, generalised to the other
/// two purposes that reach this refusal.
fn purpose_gloss(path: &[u32]) -> &'static str {
    match purpose_script(path) {
        Some(Script::P2TR) => " -- taproot single-sig",
        Some(Script::P2WSH) | Some(Script::P2SH_P2WSH) => " -- a multisig cosigner account",
        Some(Script::P2SH) => " -- a legacy multisig account",
        _ => "",
    }
}

/// A one-line rendering of a parsed descriptor's shape, for a refusal that
/// needs to name the form the operator wrote.
pub fn form_of(d: &Parsed) -> String {
    match (d.script, d.multi) {
        (s, None) => s.descriptor_form().to_string(),
        (s, Some(m)) => format!("{}({}(…))", s.descriptor_form(), m.spelling()),
    }
}
