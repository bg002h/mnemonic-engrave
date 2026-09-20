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
/// `phrase:<hex of "<method>,<phrase>">` — a hashlock PHRASE and its method
/// selector (SPEC_hashlock_H6 §3.1). The `now:` idiom exactly: hex of a UTF-8
/// text, one comma, cut on the FIRST comma so the phrase may contain commas.
///
/// RESERVED like the other three, so a `phrase:` record whose body fails any
/// rule is `Class::Unknown` and refused with its own line rather than treated
/// as free text.
pub const PHRASE_PREFIX: &str = "phrase:";

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

/// The method SELECTOR a `phrase:` record carries.
///
/// The WIRE carries a selector and the PLATE carries the method DEFINITION
/// (H6 §8.6), and the difference is deliberate: a wire record is read by a tool
/// that already knows the parameter set, while a plate is read by a person who
/// may have neither the tool nor this firmware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashlockMethod {
    /// `hardened` — PBKDF2-HMAC-SHA256, the `ms hashlock` default.
    Hardened,
    /// `sha256` — one SHA-256 of the phrase bytes.
    Sha256,
}

impl HashlockMethod {
    /// The selector spelling, which is `ms hashlock --method`'s and the
    /// device's `hashlockMethod.String()`.
    pub fn as_str(self) -> &'static str {
        match self {
            HashlockMethod::Hardened => "hardened",
            HashlockMethod::Sha256 => "sha256",
        }
    }

    /// X from the phrase under this method.
    pub fn preimage(self, phrase: &[u8]) -> zeroize::Zeroizing<[u8; 32]> {
        match self {
            HashlockMethod::Hardened => ms_codec::hashlock::preimage_hardened(phrase),
            HashlockMethod::Sha256 => ms_codec::hashlock::preimage_sha256(phrase),
        }
    }
}

/// A parsed `phrase:` record. `phrase` is SECRET.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhraseRecord {
    /// The method selector the record names.
    pub method: HashlockMethod,
    /// The phrase, verbatim, everything after the FIRST comma.
    pub phrase: String,
}

/// Which hash the SCRIPT commits to, in a `hash:` record
/// (SPEC_hashlock_kinds §6).
///
/// **A LOCAL definition, deliberately** (spec §5: "one local definition per
/// crate, nothing shared across repo boundaries"). `ms-codec` exposes one
/// digest function per kind and each caller maps its own kind onto one; sharing
/// a type across the repo boundary would tie this crate's record grammar to
/// that crate's release schedule.
///
/// NOT the same axis as a `phrase:` record's METHOD, which says how a preimage
/// was derived. They share the token `sha256` and mean different things, and
/// §6 requires that where both could be read, both are named or neither is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RecordHashKind {
    /// `sha256(X)` — the bare form on the wire.
    Sha256,
    /// `hash256(X)` = `sha256(sha256(X))`.
    Hash256,
    /// `ripemd160(X)`, the bare primitive.
    Ripemd160,
    /// `hash160(X)` = `ripemd160(sha256(X))`.
    Hash160,
}

impl RecordHashKind {
    /// **The only place a digest length is written** (spec §5). The hex rule is
    /// `digest_len() * 2`.
    pub const fn digest_len(self) -> usize {
        match self {
            RecordHashKind::Sha256 | RecordHashKind::Hash256 => 32,
            RecordHashKind::Ripemd160 | RecordHashKind::Hash160 => 20,
        }
    }

    /// The lowercase miniscript fragment name, which is the §6 record token.
    pub const fn token(self) -> &'static str {
        match self {
            RecordHashKind::Sha256 => "sha256",
            RecordHashKind::Hash256 => "hash256",
            RecordHashKind::Ripemd160 => "ripemd160",
            RecordHashKind::Hash160 => "hash160",
        }
    }

    /// Parse a §6 token. **Case is rejected, never folded** — no
    /// `to_ascii_lowercase` here, by rule, because that is how two parsers come
    /// to disagree about what a record means.
    pub fn from_token(t: &str) -> Option<Self> {
        match t {
            "sha256" => Some(RecordHashKind::Sha256),
            "hash256" => Some(RecordHashKind::Hash256),
            "ripemd160" => Some(RecordHashKind::Ripemd160),
            "hash160" => Some(RecordHashKind::Hash160),
            _ => None,
        }
    }

    /// This kind's digest of a 32-byte preimage, via `ms-codec`'s one function
    /// per kind. The preimage is 32 bytes for EVERY kind (spec §3 F1); only the
    /// digest width moves.
    pub fn digest_of(self, preimage: &[u8; 32]) -> Vec<u8> {
        use ms_codec::hashlock as h;
        match self {
            RecordHashKind::Sha256 => h::digest_sha256(preimage).to_vec(),
            RecordHashKind::Hash256 => h::digest_hash256(preimage).to_vec(),
            RecordHashKind::Ripemd160 => h::digest_ripemd160(preimage).to_vec(),
            RecordHashKind::Hash160 => h::digest_hash160(preimage).to_vec(),
        }
    }
}

/// A `hash:` record's content: which hash, and the digest at that kind's width.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HashLock {
    kind: RecordHashKind,
    digest: Vec<u8>,
}

impl HashLock {
    /// Build one, refusing a digest that is not this kind's width. Returns
    /// `None` rather than truncating or padding: a wrong-width digest is an
    /// error at the boundary, not something to reshape.
    pub fn new(kind: RecordHashKind, digest: &[u8]) -> Option<Self> {
        (digest.len() == kind.digest_len()).then(|| HashLock {
            kind,
            digest: digest.to_vec(),
        })
    }

    /// Which hash the script commits to.
    pub const fn kind(&self) -> RecordHashKind {
        self.kind
    }

    /// The digest, at its kind's width.
    pub fn digest(&self) -> &[u8] {
        &self.digest
    }
}

/// A parsed record of one of the four classes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposerRecord {
    /// `key:`
    Key(KeyRecord),
    /// `hash:` — which hash the script commits to, and the digest (§6).
    Hash(HashLock),
    /// `now:` — pack seconds and optional height.
    Now {
        /// Unix seconds, 1..=2147483647.
        seconds: u32,
        /// Block height, 1..=499999999, when the packer knew one.
        height: Option<u32>,
    },
    /// `phrase:` — a hashlock phrase and its method.
    Phrase(PhraseRecord),
}

/// Why a prefixed record is `Class::Unknown` (spec §8n has one line per class).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposerRecordError {
    /// `key:` failed; the detail is for logs and tests, the line is fixed.
    Key(&'static str),
    /// A well-formed key record whose extended key is not a MAINNET key (a
    /// `tpub`/`upub`/`vpub` version). Its own variant and its own §8n line,
    /// deliberately: the `Key` line describes a MALFORMED record ("a bare
    /// xpub is not a key record"), and a testnet key is not malformed -- it
    /// is unsupported, because complex-policy derivation is mainnet-only by
    /// construction (SPEC §4f). Refusing it with the malformation body would
    /// be a refusal with the wrong reason, the class the fable review r0
    /// filed twice (L4 M-4, M-5). Three lenses found the silent admission
    /// (L1 M-2, L3 M-2, L4 M-6): the key seated, the consent printed mainnet
    /// addresses for material derived under coin type 1', and the re-minted
    /// card carried the tpub while its label said mainnet.
    KeyNetwork,
    /// `hash:` failed its rule (§6). `Some(kind)` means the token was
    /// understood and the body was not that kind's width or not lowercase hex;
    /// `None` means the token itself was unknown or wrongly cased.
    ///
    /// **It carries the kind, unlike `Phrase`, because a digest is PUBLIC.**
    /// `Phrase` is one variant on purpose — naming which rule a phrase failed
    /// would print a property of a secret. Nothing here is secret, and an
    /// operator told "must be exactly 64 hex characters" about the 64-hex
    /// `ripemd160` value they just pasted learns nothing.
    Hash(Option<RecordHashKind>),
    /// `now:` is not `<seconds>[,<height>]` in range.
    Now,
    /// `phrase:` is not `<method>,<phrase>` with a known method and a phrase
    /// the rule admits. ONE variant, not five: the §8n line is fixed per class,
    /// and a refusal that named WHICH rule the phrase failed would print a
    /// property of a secret the operator did not ask to have described.
    Phrase,
}

impl ComposerRecordError {
    /// The §8n line for record `index` (records count from 0, as every other
    /// `me sysw pack` refusal counts).
    pub fn line(&self, index: usize) -> String {
        match self {
            ComposerRecordError::Key(_) => format!(
                "record {index}: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record"
            ),
            ComposerRecordError::Hash(Some(k)) => format!(
                "record {index}: hash: {} needs exactly {} lowercase hex characters",
                k.token(),
                k.digest_len() * 2
            ),
            ComposerRecordError::Hash(None) => format!(
                "record {index}: hash: unknown hash kind; expected `hash:<hex>` (sha256) or \
                 `hash:<kind>:<hex>` with kind hash256, ripemd160 or hash160, lowercase"
            ),
            ComposerRecordError::KeyNetwork => format!(
                "record {index}: key: a testnet key (tpub) is unsupported; complex-policy derivation is mainnet-only (SPEC §4f)"
            ),
            ComposerRecordError::Now => format!("record {index}: now: must be <seconds>[,<height>] in range"),
            ComposerRecordError::Phrase => format!(
                "record {index}: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256"
            ),
        }
    }

    /// The detail behind a `Key` refusal, for logs and tests.
    pub fn detail(&self) -> &'static str {
        match self {
            ComposerRecordError::Key(d) => d,
            ComposerRecordError::KeyNetwork => {
                "a well-formed key record whose xpub version is not mainnet"
            }
            ComposerRecordError::Hash(Some(_)) => "not that kind's width in lowercase hex",
            ComposerRecordError::Hash(None) => "unknown or wrongly-cased hash kind token",
            ComposerRecordError::Now => "not <seconds>[,<height>] in range",
            ComposerRecordError::Phrase => {
                "not <method>,<phrase> with a known method and an admissible phrase"
            }
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
    // `% 2 != 0`, NOT `is_multiple_of`, AND THE LINT IS ALLOWED ON PURPOSE.
    //
    // A newer clippy suggests `.is_multiple_of(2)`; that method is UNSTABLE
    // (rust-lang/rust#128101) and CI pins Rust 1.85.0, where it is E0658. I
    // took the suggestion, the local gate went green on a 1.97 nightly, and CI
    // refused to compile -- a local tool demanding a change the pinned
    // toolchain rejects. The push ritual caught it and correctly refused to
    // push master, which is the whole reason the staging step exists.
    //
    // `unknown_lints` is allowed alongside it so the pinned 1.85 clippy, which
    // has never heard of this lint, does not warn about the allow itself.
    #[allow(unknown_lints, clippy::manual_is_multiple_of)]
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
pub fn hash_record(lock: &HashLock) -> String {
    // §6's PRODUCER RULE: the bare form for sha256, the explicit form for the
    // other three. "Input is liberal, output is conservative" -- an explicit
    // `hash:sha256:` is accepted on input and never emitted, so every payload
    // that exists today and every new sha256 payload is byte-identical.
    match lock.kind() {
        RecordHashKind::Sha256 => format!("{HASH_PREFIX}{}", hex_lower(lock.digest())),
        k => format!("{HASH_PREFIX}{}:{}", k.token(), hex_lower(lock.digest())),
    }
}

/// `phrase:` + hex of `<method>,<phrase>`. The text is NOT validated here;
/// `parse` is the gate, so a test can build a malformed record on purpose.
///
/// IT RETURNS `Zeroizing<String>` BECAUSE THE PHRASE IS IN IT, and it is the
/// one member of this family that carries a secret: `key_record`,
/// `hash_record` and `now_record` build public data, so a plain `String` is
/// right for them and wrong here (F-493). `ms-codec`'s `qr_text` was folded the
/// same way for the same bytes before the 0.9.0 publish.
///
/// The cost of the fold was nothing, which is why it happened rather than being
/// accepted the way the keyboard's immutable fragment was (F-483): measured at
/// the time of the ruling, this function had NO production call site — eight
/// test uses and nothing else — so no consumer had to change and no convention
/// had to bend.
///
/// The buffer is `Zeroizing` from the first byte and sized up front. Wrapping a
/// finished `format!` would protect only the copy: the `format!` would build an
/// unprotected `String` holding the phrase, and the wrap would guard what was
/// left after the leak. The METHOD is a compile-time constant and carries
/// nothing of the phrase.
pub fn phrase_record(method: HashlockMethod, phrase: &str) -> zeroize::Zeroizing<String> {
    let m = method.as_str();
    let body_len = (m.len() + 1 + phrase.len()) * 2;
    let mut out = zeroize::Zeroizing::new(String::with_capacity(PHRASE_PREFIX.len() + body_len));
    out.push_str(PHRASE_PREFIX);
    for b in m.as_bytes().iter().chain(b",").chain(phrase.as_bytes()) {
        use std::fmt::Write as _;
        write!(&mut *out, "{b:02x}").ok();
    }
    out
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
    if let Some(body) = record.strip_prefix(PHRASE_PREFIX) {
        return Some(parse_phrase(body));
    }
    None
}

/// `phrase:` — hex of `<method>,<phrase>`, CUT ON THE FIRST COMMA.
///
/// Never the last: a phrase may contain commas, and cutting on the last would
/// take everything after the final comma as the phrase and derive a DIFFERENT
/// preimage from the one the packer meant.
///
/// The phrase itself must pass `ms_codec::hashlock::validate_phrase` — the one
/// implementation of SPEC_ms_hashlock §4.3, which `ms hashlock` and the
/// device's `hashlock.ValidatePhrase` also apply — so the record parser and
/// the keyboard cannot disagree about what a phrase is.
fn parse_phrase(body: &str) -> Result<ComposerRecord, ComposerRecordError> {
    use ComposerRecordError::Phrase as P;
    let bytes = unhex_lower(body).ok_or(P)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| P)?;
    let (method_text, phrase) = text.split_once(',').ok_or(P)?;
    let method = match method_text {
        "hardened" => HashlockMethod::Hardened,
        "sha256" => HashlockMethod::Sha256,
        _ => return Err(P),
    };
    ms_codec::hashlock::validate_phrase(phrase.as_bytes()).map_err(|_| P)?;
    Ok(ComposerRecord::Phrase(PhraseRecord {
        method,
        phrase: phrase.to_owned(),
    }))
}

fn parse_hash(body: &str) -> Result<ComposerRecord, ComposerRecordError> {
    // SPEC_hashlock_kinds §6: `hash: [<kind>:] <hex>`. An ABSENT kind means
    // sha256 -- that is what keeps every payload packed before this cycle
    // byte-identical.
    //
    // Split on the LAST colon rather than the first: the body is either
    // `<hex>` or `<kind>:<hex>`, and hex contains no colon, so the tail after
    // a colon is always the digest.
    let (kind, hex_body) = match body.rsplit_once(':') {
        Some((tok, rest)) => (
            // An unknown or wrongly-cased token is a REFUSAL, never a fallback
            // to sha256. Reading `hash:sha512:<hex>` as sha256 is exactly the
            // silent-mis-read §6's fail-closed paragraph rules out.
            RecordHashKind::from_token(tok).ok_or(ComposerRecordError::Hash(None))?,
            rest,
        ),
        None => (RecordHashKind::Sha256, body),
    };
    if hex_body.len() != kind.digest_len() * 2 {
        return Err(ComposerRecordError::Hash(Some(kind)));
    }
    let bytes = unhex_lower(hex_body).ok_or(ComposerRecordError::Hash(Some(kind)))?;
    let lock = HashLock::new(kind, &bytes).ok_or(ComposerRecordError::Hash(Some(kind)))?;
    Ok(ComposerRecord::Hash(lock))
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
    // Each component must be ASCII digits with an optional `'` or `h`. rust-bitcoin's
    // `DerivationPath::from_str` parses indices through `u32::from_str`, which tolerates a
    // leading `+` (`+48'`); that is not BIP-380 key-origin notation, the device refuses it,
    // and the lockstep fixture pins both sides (composer-S2-exec-review-r0 I-1, Rust first).
    if !path_text.split('/').all(|c| {
        let digits = c
            .strip_suffix('\'')
            .or_else(|| c.strip_suffix('h'))
            .unwrap_or(c);
        !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
    }) {
        return Err(K("path component is not digits with an optional ' or h"));
    }
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
    if xpub.network != bitcoin::NetworkKind::Main {
        return Err(ComposerRecordError::KeyNetwork);
    }
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
    // RETIRED ACCEPTANCE (composer fable review r0, 2026-09-20): S1 pinned that a
    // tpub PARSES as a key record; three lenses then found it seating silently
    // into a mainnet-only policy (SPEC §4f, §14). The same record now refuses.
    Case { name: "key-testnet-tpub-refused-s1", record: "key:5b37336335646130612f3438272f31272f30272f32275d747075624446483964677a76657944387a5462505546754c72476d4379644e76786568794e6455584b4a41514e387834615a346a36555a7147666e71467244344e7179615456474b62764557353474737650544b32556f5362434331504a593869434e6977544c3352575a45686551", class: "Unknown", host_line: Some("record 0: key: a testnet key (tpub) is unsupported; complex-policy derivation is mainnet-only (SPEC §4f)") },
    Case { name: "key-depth-2-refused", record: "key:5b37336335646130612f3438272f30275d787075623639784456786235326d37484c693856346242556f7351646a41343771476b5142354b6738454b6867417737386e41615066625a3761765a544862506f58716a7a5743337761766b375a75524e7737325843533343795339587568594a7141764d457245644562366e3254", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-depth-5-refused", record: "key:5b37336335646130612f3438272f30272f30272f32272f305d78707562364767475a4369657850337170683533486a50694c7237476b5846746669635455677661706e4a32583275696737795a763578674538635163445367396f6538597062626f4b43476b68724742565a525973354776585a39366835556b32514845554d735a7250374d764c", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-fingerprint-uppercase", record: "key:5b37334335444130412f3438272f30272f30272f32275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-fingerprint-7-hex", record: "key:5b373363356461302f3438272f30272f30272f32275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-origin-no-path", record: "key:5b37336335646130615d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-origin-longer-than-depth", record: "key:5b37336335646130612f3438272f30272f30272f30272f32275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-origin-unterminated", record: "key:5b37336335646130612f3438272f30272f30272f32277870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-body-not-utf8", record: "key:ff", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-uppercase-H-marker-out-of-scope", record: "key:5b37336335646130612f3438482f30482f30482f32485d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-testnet-tpub-refused", record: "key:5b33663633356136332f3438272f31272f30272f32275d74707562444650745041726a34477a424546486f68656767315861747263314669396f536f78354c7a7553525839316d697751787555724570427870764452736d5a594a4b59466867644b33555374736a43384a4b586655624d696e6a467169454d34754e777a5661436148707973", class: "Unknown", host_line: Some("record 0: key: a testnet key (tpub) is unsupported; complex-policy derivation is mainnet-only (SPEC §4f)") },
    Case { name: "hash-valid", record: "hash:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Hash", host_line: None },
    Case { name: "hash-valid-zeros", record: "hash:0000000000000000000000000000000000000000000000000000000000000000", class: "Hash", host_line: None },
    Case { name: "hash-63-chars", record: "hash:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a", class: "Unknown", host_line: Some("record 0: hash: sha256 needs exactly 64 lowercase hex characters") },
    Case { name: "hash-66-chars", record: "hash:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Unknown", host_line: Some("record 0: hash: sha256 needs exactly 64 lowercase hex characters") },
    Case { name: "hash-uppercase", record: "hash:A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8A8", class: "Unknown", host_line: Some("record 0: hash: sha256 needs exactly 64 lowercase hex characters") },
    Case { name: "hash-hash256-valid", record: "hash:hash256:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Hash", host_line: None },
    Case { name: "hash-ripemd160-valid", record: "hash:ripemd160:5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c", class: "Hash", host_line: None },
    Case { name: "hash-hash160-valid", record: "hash:hash160:5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c", class: "Hash", host_line: None },
    Case { name: "hash-sha256-explicit", record: "hash:sha256:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Hash", host_line: None },
    Case { name: "hash-ripemd160-wrong-width", record: "hash:ripemd160:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Unknown", host_line: Some("record 0: hash: ripemd160 needs exactly 40 lowercase hex characters") },
    Case { name: "hash-hash160-wrong-width", record: "hash:hash160:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Unknown", host_line: Some("record 0: hash: hash160 needs exactly 40 lowercase hex characters") },
    Case { name: "hash-hash256-wrong-width", record: "hash:hash256:5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c", class: "Unknown", host_line: Some("record 0: hash: hash256 needs exactly 64 lowercase hex characters") },
    Case { name: "hash-unknown-token", record: "hash:sha512:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Unknown", host_line: Some("record 0: hash: unknown hash kind; expected `hash:<hex>` (sha256) or `hash:<kind>:<hex>` with kind hash256, ripemd160 or hash160, lowercase") },
    Case { name: "hash-kind-uppercase", record: "hash:RIPEMD160:5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c", class: "Unknown", host_line: Some("record 0: hash: unknown hash kind; expected `hash:<hex>` (sha256) or `hash:<kind>:<hex>` with kind hash256, ripemd160 or hash160, lowercase") },
    Case { name: "hash-kind-mixedcase", record: "hash:Hash160:5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c", class: "Unknown", host_line: Some("record 0: hash: unknown hash kind; expected `hash:<hex>` (sha256) or `hash:<kind>:<hex>` with kind hash256, ripemd160 or hash160, lowercase") },
    Case { name: "hash-ripemd160-short", record: "hash:ripemd160:5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c", class: "Unknown", host_line: Some("record 0: hash: ripemd160 needs exactly 40 lowercase hex characters") },
    Case { name: "hash-hash160-uppercase", record: "hash:hash160:5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C", class: "Unknown", host_line: Some("record 0: hash: hash160 needs exactly 40 lowercase hex characters") },
    Case { name: "hash-hash256-non-hex", record: "hash:hash256:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8ag", class: "Unknown", host_line: Some("record 0: hash: hash256 needs exactly 64 lowercase hex characters") },
    Case { name: "hash-empty", record: "hash:", class: "Unknown", host_line: Some("record 0: hash: sha256 needs exactly 64 lowercase hex characters") },
    Case { name: "hash-31-bytes", record: "hash:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8", class: "Unknown", host_line: Some("record 0: hash: sha256 needs exactly 64 lowercase hex characters") },
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
    // Two rows from the S2 whole-diff review (composer-S2-exec-review-r0 C-1, I-1): the corpus
    // varied a component's SHAPE but never its numeric RANGE or SIGN. An unhardened component of
    // 2^31 is refused (the device's in-band hardening bit would re-read it as 0h); a `+`-signed
    // component is refused on both sides now (the host used to admit it through u32::from_str).
    Case { name: "key-origin-component-unhardened-2^31", record: "key:5b37336335646130612f323134373438333634382f30272f30272f32275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    Case { name: "key-origin-component-plus-sign", record: "key:5b37336335646130612f2b3438272f30272f30272f32275d7870756236446b4641585751326448787132766174727439717941336258595534546f57517743486266355842326d5354657863485a43654b5331565a5963506f4264355838795663625846484a523952385543567074383256583156685232386d43797855464c3472364b467266", class: "Unknown", host_line: Some("record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record") },
    // ---- H6 §3.1: the `phrase:` record, one row per rule (SPEC_hashlock_H6 §11.1).
    // `phrase-space-after-the-comma` is ADMITTED and is the hand-build error §8.2.3's
    // orphan warning exists to catch: a leading 0x20 is printable ASCII, so the record
    // is valid and derives a DIFFERENT preimage from the same words without it.
    Case { name: "phrase-hardened", record: "phrase:68617264656e65642c636f727265637420686f727365206261747465727920737461706c65", class: "Phrase", host_line: None },
    Case { name: "phrase-sha256", record: "phrase:7368613235362c636f727265637420686f727365206261747465727920737461706c65", class: "Phrase", host_line: None },
    Case { name: "phrase-containing-a-comma", record: "phrase:68617264656e65642c6f6e652c2074776f2c207468726565", class: "Phrase", host_line: None },
    Case { name: "phrase-space-after-the-comma", record: "phrase:68617264656e65642c20636f727265637420686f727365206261747465727920737461706c65", class: "Phrase", host_line: None },
    Case { name: "phrase-containing-a-colon", record: "phrase:68617264656e65642c6e6f74653a20756e64657220746865206d6174", class: "Phrase", host_line: None },
    Case { name: "phrase-one-character", record: "phrase:68617264656e65642c78", class: "Phrase", host_line: None },
    Case { name: "phrase-100-characters", record: "phrase:68617264656e65642c30303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030", class: "Phrase", host_line: None },
    Case { name: "phrase-101-characters", record: "phrase:68617264656e65642c3030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-empty", record: "phrase:68617264656e65642c", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    // F-546/F-539: a digest-shaped phrase is an ADVISORY on the typed screen,
    // not a refusal, so the record parser admits it. This row flipped
    // Unknown -> Phrase when me-cli's ms-codec pin caught up with the host.
    Case { name: "phrase-64-hex", record: "phrase:68617264656e65642c61626162616261626162616261626162616261626162616261626162616261626162616261626162616261626162616261626162616261626162616261626162", class: "Phrase", host_line: None },
    Case { name: "phrase-ms1-shaped", record: "phrase:68617264656e65642c6d7331306861736873717734366832617434773436683261743477343668326174347734366832617434773436683261743477343668326174347734366b7a76326e6379363075377a3963", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-ms1-shaped-grouped", record: "phrase:68617264656e65642c6d7331306861736873712077343668326174347734203668326174347734366820326174347734366832612074347734366832617434207734366832617434773420366b7a76326e637936302075377a3963", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-unknown-method", record: "phrase:7363727970742c636f727265637420686f727365206261747465727920737461706c65", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-uppercase-method", record: "phrase:48415244454e45442c636f727265637420686f727365206261747465727920737461706c65", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-no-comma", record: "phrase:68617264656e6564", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-body-not-hex", record: "phrase:zz", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-body-uppercase-hex", record: "phrase:68617264656E65642C78", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-body-odd-length", record: "phrase:686", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-body-not-utf8", record: "phrase:ff", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-body-empty", record: "phrase:", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
    Case { name: "phrase-non-printable-tab", record: "phrase:68617264656e65642c6f6e650974776f", class: "Unknown", host_line: Some("record 0: phrase: must be <method>,<phrase> as lowercase hex, with method hardened or sha256") },
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
