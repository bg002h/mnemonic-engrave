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
