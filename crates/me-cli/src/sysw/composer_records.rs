//! The composer's three payload record classes — `key:`, `hash:`, `now:` —
//! per `SPEC_wallet_policy_composer.md` §6a (mnemonic-engrave), following
//! `SPEC_systemwide_payloads.md` section 5.3: a RESERVED prefix, a lowercase-hex
//! body, matched before the sniffers, and a prefixed record whose body fails any
//! rule is `Class::Unknown` and refused with its own line (§8n).
//!
//! None of the three is secret or bearer. `key:` carries a cosigner's
//! `[fingerprint/path]xpub` (BIP-380 key-origin notation, the key form
//! `md decompose` prints); `hash:` a 32-byte sha256 digest for a hashlock;
//! `now:` the PACK time and optional height — a LOWER BOUND on the present that
//! the device (which has no clock) echoes and never encodes (C24).
//!
//! What a `key:` record's origin PROVES: the xpub's depth and its last child
//! number are checked against the declared path; the fingerprint, the account
//! and every interior component are declarations nothing here can verify
//! (F-217). The mapping review on the device says so beside each slot.
//!
//! The hex helpers are local twins of `record.rs`'s private ones on purpose:
//! this module's rules are the composer spec's and are ported to the Go
//! classifier as ONE unit; sharing a private helper across the two prefix
//! families would couple that port to `record.rs`'s history.

use std::str::FromStr;

use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};

/// `key:<hex of "[fingerprint/path]xpub">`.
pub const KEY_PREFIX: &str = "key:";
/// `hash:<64 lowercase hex>` — the 32-byte digest itself.
pub const HASH_PREFIX: &str = "hash:";
/// `now:<hex of "<seconds>[,<height>]">`.
pub const NOW_PREFIX: &str = "now:";

/// BIP-65: absolute locktimes below this are heights; `now:`'s height band.
const MAX_HEIGHT: u32 = 499_999_999;
/// BIP-379: the largest absolute locktime miniscript admits; `now:`'s seconds band.
const MAX_SECONDS: u32 = 2_147_483_647;

/// One cosigner key for seating.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRecord {
    /// The DECLARED master fingerprint (unverifiable from the xpub).
    pub fingerprint: Fingerprint,
    /// The DECLARED origin, component count == xpub depth.
    pub origin: DerivationPath,
    /// The extended public key, depth 3 or 4.
    pub xpub: Xpub,
    /// The decoded body, verbatim (`[fingerprint/path]xpub`).
    pub text: String,
}

/// A parsed record of one of the three classes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposerRecord {
    /// `key:`
    Key(KeyRecord),
    /// `hash:` — the digest.
    Hash([u8; 32]),
    /// `now:` — pack seconds and optional height.
    Now {
        /// Unix seconds, 1..=2147483647.
        seconds: u32,
        /// Block height, 1..=499999999, when the packer knew one.
        height: Option<u32>,
    },
}

/// Why a prefixed record is `Class::Unknown` (spec §8n has one line per class).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposerRecordError {
    /// `key:` failed; the detail is for logs and tests, the line is fixed.
    Key(&'static str),
    /// `hash:` is not exactly 64 lowercase hex characters.
    Hash,
    /// `now:` is not `<seconds>[,<height>]` in range.
    Now,
}

impl ComposerRecordError {
    /// The §8n line for record `index` (records count from 0, as every other
    /// `me sysw pack` refusal counts).
    pub fn line(&self, index: usize) -> String {
        match self {
            ComposerRecordError::Key(_) => format!(
                "record {index}: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record"
            ),
            ComposerRecordError::Hash => format!("record {index}: hash: must be exactly 64 hex characters"),
            ComposerRecordError::Now => format!("record {index}: now: must be <seconds>[,<height>] in range"),
        }
    }

    /// The detail behind a `Key` refusal, for logs and tests.
    pub fn detail(&self) -> &'static str {
        match self {
            ComposerRecordError::Key(d) => d,
            ComposerRecordError::Hash => "not exactly 64 lowercase hex characters",
            ComposerRecordError::Now => "not <seconds>[,<height>] in range",
        }
    }
}

fn hex_lower(b: &[u8]) -> String {
    use std::fmt::Write as _;
    b.iter()
        .fold(String::with_capacity(b.len() * 2), |mut s, x| {
            let _ = write!(s, "{x:02x}");
            s
        })
}

/// Strict: even length, every character in `0-9a-f`. Uppercase is NOT hex here
/// (section 5.3: the section is hashed in its canonical lowercase form).
fn unhex_lower(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for pair in s.as_bytes().chunks(2) {
        let hi = (pair[0] as char).to_digit(16)? as u8;
        let lo = (pair[1] as char).to_digit(16)? as u8;
        out.push((hi << 4) | lo);
    }
    Some(out)
}

/// `key:` + hex of the origin text. The text is NOT validated here; `parse`
/// is the gate, so a test can build a malformed record on purpose.
pub fn key_record(text: &str) -> String {
    format!("{KEY_PREFIX}{}", hex_lower(text.as_bytes()))
}

/// `hash:` + the digest as 64 lowercase hex.
pub fn hash_record(digest: &[u8; 32]) -> String {
    format!("{HASH_PREFIX}{}", hex_lower(digest))
}

/// `now:` + hex of `<seconds>[,<height>]`.
pub fn now_record(seconds: u32, height: Option<u32>) -> String {
    let text = match height {
        Some(h) => format!("{seconds},{h}"),
        None => seconds.to_string(),
    };
    format!("{NOW_PREFIX}{}", hex_lower(text.as_bytes()))
}

/// Indices of the VALID `now:` records in `records` (a malformed one is
/// `Unknown` and refused elsewhere; it is not a second `now:`).
pub fn now_indices(records: &[String]) -> Vec<usize> {
    records
        .iter()
        .enumerate()
        .filter(|(_, r)| matches!(parse(r), Some(Ok(ComposerRecord::Now { .. }))))
        .map(|(i, _)| i)
        .collect()
}

/// `None`: not one of the three prefixes (case-sensitive, like `text:`).
/// `Some(Ok)`: a valid record. `Some(Err)`: prefixed but malformed — the
/// caller classifies it `Unknown` and refuses with `err.line(index)`.
pub fn parse(record: &str) -> Option<Result<ComposerRecord, ComposerRecordError>> {
    if let Some(body) = record.strip_prefix(KEY_PREFIX) {
        return Some(parse_key(body));
    }
    if let Some(body) = record.strip_prefix(HASH_PREFIX) {
        return Some(parse_hash(body));
    }
    if let Some(body) = record.strip_prefix(NOW_PREFIX) {
        return Some(parse_now(body));
    }
    None
}

fn parse_hash(body: &str) -> Result<ComposerRecord, ComposerRecordError> {
    if body.len() != 64 {
        return Err(ComposerRecordError::Hash);
    }
    let bytes = unhex_lower(body).ok_or(ComposerRecordError::Hash)?;
    let mut h = [0u8; 32];
    h.copy_from_slice(&bytes);
    Ok(ComposerRecord::Hash(h))
}

fn parse_now(body: &str) -> Result<ComposerRecord, ComposerRecordError> {
    let bytes = unhex_lower(body).ok_or(ComposerRecordError::Now)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| ComposerRecordError::Now)?;
    let (secs, height) = match text.split_once(',') {
        Some((s, h)) => (s, Some(h)),
        None => (text, None),
    };
    let seconds = digits_in_range(secs, 10, 1, MAX_SECONDS).ok_or(ComposerRecordError::Now)?;
    let height = match height {
        Some(h) => Some(digits_in_range(h, 9, 1, MAX_HEIGHT).ok_or(ComposerRecordError::Now)?),
        None => None,
    };
    Ok(ComposerRecord::Now { seconds, height })
}

/// `^[0-9]{1,max_digits}$` and `lo..=hi`, with no sign, whitespace or point.
fn digits_in_range(s: &str, max_digits: usize, lo: u32, hi: u32) -> Option<u32> {
    if s.is_empty() || s.len() > max_digits || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let v: u64 = s.parse().ok()?;
    if v < u64::from(lo) || v > u64::from(hi) {
        return None;
    }
    Some(v as u32)
}

fn parse_key(body: &str) -> Result<ComposerRecord, ComposerRecordError> {
    use ComposerRecordError::Key as K;
    let bytes = unhex_lower(body).ok_or(K("body is not lowercase hex"))?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| K("body is not UTF-8"))?
        .to_owned();
    // `[fingerprint/path]xpub`: the origin is REQUIRED (an md1 slot carries a path).
    let rest = text
        .strip_prefix('[')
        .ok_or(K("no [origin]: a bare xpub"))?;
    let (origin_text, xpub_text) = rest.split_once(']').ok_or(K("unterminated [origin]"))?;
    let (fp_text, path_text) = origin_text.split_once('/').ok_or(K("origin has no path"))?;
    if fp_text.len() != 8
        || !fp_text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(K("fingerprint is not 8 lowercase hex characters"));
    }
    let fingerprint =
        Fingerprint::from_str(fp_text).map_err(|_| K("fingerprint does not parse"))?;
    let origin = DerivationPath::from_str(&format!("m/{path_text}"))
        .map_err(|_| K("path does not parse"))?;
    if origin.is_empty() {
        return Err(K("origin has no path components"));
    }
    if xpub_text.contains('/') {
        // The likeliest mis-paste: a key copied out of a descriptor, derivation suffix and
        // all. It IS an extended public key with an origin, so "not an extended public key"
        // would name the wrong problem (composer-S1-exec-review-r0 M-3). Class and §8n line
        // are unchanged; only the detail says what actually went wrong.
        return Err(K("the key carries a derivation suffix; give the account xpub alone, as `md decompose --emit keys` prints it"));
    }
    let xpub = Xpub::from_str(xpub_text).map_err(|_| K("not an extended public key"))?;
    if !matches!(xpub.depth, 3 | 4) {
        return Err(K("xpub depth is not 3 or 4"));
    }
    if origin.len() != usize::from(xpub.depth) {
        return Err(K("origin component count differs from the xpub's depth"));
    }
    let last: ChildNumber = *origin.as_ref().last().expect("non-empty");
    if last != xpub.child_number {
        return Err(K(
            "the origin's last component is not the xpub's own child number",
        ));
    }
    Ok(ComposerRecord::Key(KeyRecord {
        fingerprint,
        origin,
        xpub,
        text,
    }))
}

/// One lockstep case (spec §12 item 8): the record, the class Rust assigns, and
/// the §8n line Rust prints when it refuses (index 0: each case is packed alone).
/// The Go port asserts the same class for the same record and leaves refused
/// records inert; Stage 2 vendors the generated JSON with the same sha256.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Case {
    /// Stable id.
    pub name: &'static str,
    /// The record, verbatim.
    pub record: &'static str,
    /// `Debug` name of the `Class`: "Key", "Hash", "Now" or "Unknown".
    pub class: &'static str,
    /// The refusal line at index 0, or `None` for an admitted record.
    pub host_line: Option<&'static str>,
}

/// The journey's cosigner @0, `[73c5da0a/48'/0'/0'/2']xpub…`, as a key: record.
const KEY0_RECORD: &str = "key:5b37336335646130612f3438272f30272f30272f32275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266";
/// The same xpub with its origin at account 3 (component count 4 == depth, last component mismatch 3' vs 2').
const KEY_LAST_MISMATCH: &str = "key:5b37336335646130612f3438272f30272f30272f33275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266";
/// Two origin components for a depth-4 xpub, last component 2' == the xpub's child number, so ONLY the component-count rule fires.
const KEY_SHORT_ORIGIN: &str = "key:5b37336335646130612f3438272f32275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266";
/// The bare xpub (no `[origin]`).
const KEY_BARE: &str = "key:7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266";

pub const CASES: &[Case] = &[
    Case { name: "key-journey-cosigner-0", record: KEY0_RECORD, class: "Key", host_line: None },
    Case { name: "key-h-spelling", record: "key:5b37336335646130612f3438682f30682f30682f32685d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Key", host_line: None },
    Case { name: "key-bare-xpub", record: KEY_BARE, class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-origin-shorter-than-depth", record: KEY_SHORT_ORIGIN, class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-last-component-mismatch", record: KEY_LAST_MISMATCH, class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-body-not-hex", record: "key:zz", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-body-uppercase-hex", record: "key:5B", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-body-empty", record: "key:", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    // ---- every §6a rule has a row of its own (the coverage test below lists them by name)
    Case { name: "key-depth-3-valid", record: "key:5b37336335646130612f3438272f30272f30275d7870756236434b5a7455614b3159487051626736434c6147526d734d4b4c514231694b7a73766d7874794844365837677a4c71434232564e5a596431584378726363516e453868684478745962523153616b6b76697379324a3443635478576565476a6d6b6173436f4e5339765a6d", class: "Key", host_line: None },
    Case { name: "key-testnet-tpub-valid", record: "key:5b37336335646130612f3438272f31272f30272f32275d747075624446483964677a76657944387a5462505546754c72476d4379644e76786568794e6455584b4a41514e387834615a346a36555a7147666e71467244344e7179615456474b62764557353474737650544b32556f5362434331504a593869434e6977544c3352575a45686551", class: "Key", host_line: None },
    Case { name: "key-depth-2-refused", record: "key:5b37336335646130612f3438272f30275d787075623639784456786235326d37484c693856346242556f7351646a41343771476b5142354b6738454b6867417737386e41615066625a3761765a544862506f58716a7a5743337761766b375a75524e7737325843533343795339587568594a7141764d457245644562366e3254", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-depth-5-refused", record: "key:5b37336335646130612f3438272f30272f30272f32272f305d78707562364767475a4369657850337170683533486a50694c7237476b5846746669635455677661706e4a32583275696737795a763578674538635163445367396f6538597062626f4b43476b68724742565a525973354776585a39366835556b32514845554d735a7250374d764c", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-fingerprint-uppercase", record: "key:5b37334335444130412f3438272f30272f30272f32275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-fingerprint-7-hex", record: "key:5b373363356461302f3438272f30272f30272f32275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-origin-no-path", record: "key:5b37336335646130615d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-origin-longer-than-depth", record: "key:5b37336335646130612f3438272f30272f30272f30272f32275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-origin-unterminated", record: "key:5b37336335646130612f3438272f30272f30272f32277870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-body-not-utf8", record: "key:ff", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-uppercase-H-marker-out-of-scope", record: "key:5b37336335646130612f3438482f30482f30482f32485d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "hash-valid", record: "hash:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Hash", host_line: None },
    Case { name: "hash-valid-zeros", record: "hash:0000000000000000000000000000000000000000000000000000000000000000", class: "Hash", host_line: None },
    Case { name: "hash-63-chars", record: "hash:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a", class: "Unknown", host_line: Some("record 0: hash: must be exactly 64 hex characters") },
    Case { name: "hash-66-chars", record: "hash:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Unknown", host_line: Some("record 0: hash: must be exactly 64 hex characters") },
    Case { name: "hash-uppercase", record: "hash:A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8", class: "Unknown", host_line: Some("record 0: hash: must be exactly 64 hex characters") },
    Case { name: "hash-empty", record: "hash:", class: "Unknown", host_line: Some("record 0: hash: must be exactly 64 hex characters") },
    Case { name: "hash-31-bytes", record: "hash:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Unknown", host_line: Some("record 0: hash: must be exactly 64 hex characters") },
    Case { name: "now-seconds-only", record: "now:31373536363834383030", class: "Now", host_line: None },
    Case { name: "now-seconds-and-height", record: "now:313735363638343830302c393130303030", class: "Now", host_line: None },
    Case { name: "now-min", record: "now:31", class: "Now", host_line: None },
    Case { name: "now-max-both", record: "now:323134373438333634372c343939393939393939", class: "Now", host_line: None },
    Case { name: "now-zero-seconds", record: "now:30", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-seconds-2^31", record: "now:32313437343833363438", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-height-zero", record: "now:313735363638343830302c30", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-height-at-time-threshold", record: "now:313735363638343830302c353030303030303030", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-trailing-comma", record: "now:313735363638343830302c", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-letters", record: "now:616263", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-body-not-hex", record: "now:zz", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-body-not-utf8", record: "now:ff", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-empty", record: "now:", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-body-uppercase-hex", record: "now:313735363638343830302C393130303030", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    // Three rows added at the S1 whole-diff review (composer-S1-exec-review-r0 M-2): the
    // divergences a reasonable Go port would produce. Arabic-Indic digits are digits to
    // `unicode.IsDigit` and not to `is_ascii_digit`; leading zeros are ADMITTED (the rule is
    // ^[0-9]{1,10}$, and a base-0 strconv would read them as octal); odd-length hex is refused.
    Case { name: "now-unicode-digits", record: "now:d9a1d9a7d9a5d9a6d9a6d9a8d9a4d9a8d9a0d9a0", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-leading-zeros-valid", record: "now:30303031373536383030", class: "Now", host_line: None },
    Case { name: "key-body-odd-length", record: "key:5b3", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    // Two rows added at the S2 plan's tests lens (composer-S2-plan-R0-r0-tests I-1): the §6a
    // digit-COUNT bound is independent of the range bound, so an in-range value padded past
    // the count (11 digits of seconds, 10 of height) must refuse even though it parses in range.
    Case { name: "now-seconds-eleven-digits", record: "now:3031373536363834383030", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
    Case { name: "now-height-ten-digits", record: "now:313735363638343830302c30343939393939393939", class: "Unknown", host_line: Some("record 0: now: must be <seconds>[,<height>] in range") },
];

/// One JSON row of `testdata/record_class_vectors.json`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FixtureRow {
    pub name: String,
    pub record: String,
    pub class: String,
    pub host_line: Option<String>,
}

/// The rows the fixture file holds, derived from [`CASES`] — never edited by hand.
pub fn fixture_rows() -> Vec<FixtureRow> {
    CASES
        .iter()
        .map(|c| FixtureRow {
            name: c.name.to_string(),
            record: c.record.to_string(),
            class: c.class.to_string(),
            host_line: c.host_line.map(str::to_string),
        })
        .collect()
}
