//! **§4.1–§4.6 — the input contract.**
//!
//! `nonstandard.OutputDescriptor` (`nonstandard/parse.go:36`) tries four things
//! in a fixed order: **1 BlueWallet → 2 plain BIP-380 → 3 `{label, descriptor}`
//! JSON → 4 promoted bare key**. `me` reproduces that **admission** order
//! exactly and does **not** reproduce the device's diagnostic loss: it keeps
//! every branch's error and §6's cause-selection rule picks the one to print.
//!
//! Three places where `me` is deliberately NARROWER than the device, each
//! licensed by §7's invariant (the host may be narrower, never wider):
//!
//! * §4.2's four BlueWallet refusals — no `Format:`, zero cosigners, an
//!   origin-less cosigner key, a fingerprint that is not exactly 8 hex
//!   characters (the last PANICS the device's parser).
//! * §4.5's ruling that a bare `tpub` is not promoted at all.
//! * a key whose 33 key-data bytes are not a compressed point (see
//!   [`super::secp`]) or are the PRIVATE `0x00` form.
//!
//! And one place `me` is deliberately WIDER, safe only because what gets packed
//! is the canonical re-encoding and never the operator's bytes: §4.6's
//! whitespace normalisation.

use super::{base58, checksum, secp};

// ───────────────────────────────────────────────────────────────────────────
// §4.6 — whitespace
// ───────────────────────────────────────────────────────────────────────────

/// Normalise CRLF to LF and trim leading/trailing ASCII whitespace from the
/// WHOLE input, before the cascade runs (§4.6).
///
/// This does not violate §7's invariant for a mechanical reason rather than a
/// judgement call: the record `me` packs is the canonical re-encoded string,
/// never the operator's file, and a `sysw` record cannot contain a newline by
/// construction. The device never sees the whitespace the host absorbed.
pub fn normalise(input: &str) -> String {
    input
        .replace("\r\n", "\n")
        .trim_matches(|c: char| c.is_ascii_whitespace())
        .to_string()
}

// ───────────────────────────────────────────────────────────────────────────
// Keys
// ───────────────────────────────────────────────────────────────────────────

/// The hardening offset, `hdkeychain.HardenedKeyStart`.
pub const HARDENED: u32 = 0x8000_0000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Network {
    Mainnet,
    Testnet,
}

/// Extended-key version bytes.
///
/// The first five are the ones `ParseExtendedKey`'s classification switch
/// admits (`bip380/bip380.go:428–466`) and therefore the ONLY five `me` admits
/// (§4.3, NORMATIVE). The rest are the SLIP-132 spellings §6's remedy row must
/// be able to NAME — `ypub` is declared in the device's constants and has no
/// case in the switch, so it is refused there even with a full explicit origin.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyVersion {
    Xpub,
    Tpub,
    Zpub,
    YpubCap,
    ZpubCap,
    Ypub,
    Upub,
    Vpub,
    UpubCap,
    VpubCap,
}

impl KeyVersion {
    fn from_bytes(v: u32) -> Option<Self> {
        Some(match v {
            0x0488_b21e => Self::Xpub,
            0x0435_87cf => Self::Tpub,
            0x04b2_4746 => Self::Zpub,
            0x0295_b43f => Self::YpubCap,
            0x02aa_7ed3 => Self::ZpubCap,
            0x049d_7cb2 => Self::Ypub,
            0x044a_5262 => Self::Upub,
            0x045f_1cf6 => Self::Vpub,
            0x0242_89ef => Self::UpubCap,
            0x0257_5483 => Self::VpubCap,
            _ => return None,
        })
    }

    /// §4.3's five-member admitted set.
    pub fn admitted(self) -> bool {
        matches!(
            self,
            Self::Xpub | Self::Tpub | Self::Zpub | Self::YpubCap | Self::ZpubCap
        )
    }

    /// The prefix an operator sees, for §6's per-version remedy.
    pub fn spelling(self) -> &'static str {
        match self {
            Self::Xpub => "xpub",
            Self::Tpub => "tpub",
            Self::Zpub => "zpub",
            Self::YpubCap => "Ypub",
            Self::ZpubCap => "Zpub",
            Self::Ypub => "ypub",
            Self::Upub => "upub",
            Self::Vpub => "vpub",
            Self::UpubCap => "Upub",
            Self::VpubCap => "Vpub",
        }
    }

    /// The network after `ParseExtendedKey`'s `SetNet` normalisation: the
    /// SLIP-132 mainnet spellings all become `xpub`, `tpub` becomes testnet.
    pub fn network(self) -> Network {
        match self {
            Self::Tpub | Self::Upub | Self::Vpub | Self::UpubCap | Self::VpubCap => {
                Network::Testnet
            }
            _ => Network::Mainnet,
        }
    }

    /// §4.5's SLIP-132 fallback: the script a key with NO explicit origin
    /// implies. Only defined for the admitted five, because the others never
    /// get past [`parse_extended_key`].
    fn implied_script(self) -> Option<Script> {
        Some(match self {
            Self::Xpub | Self::Tpub => Script::P2PKH,
            Self::Zpub => Script::P2WPKH,
            Self::YpubCap => Script::P2SH_P2WSH,
            Self::ZpubCap => Script::P2WSH,
            _ => return None,
        })
    }
}

/// `bip380.Script`. Named as the device names them, including the two the
/// spelling makes easy to confuse (`Ypub` is `P2SH_P2WSH`, not `P2SH_P2WPKH`).
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Script {
    P2SH,
    P2SH_P2WSH,
    P2SH_P2WPKH,
    P2PKH,
    P2WSH,
    P2WPKH,
    P2TR,
}

impl Script {
    /// `Script.DerivationPath()` (`bip380/bip380.go:122`) — read from the
    /// function, not from BIP-44/49/84 lore.
    pub fn derivation_path(self) -> Vec<u32> {
        let h = |n: u32| HARDENED + n;
        match self {
            Self::P2WPKH => vec![h(84), h(0), h(0)],
            Self::P2PKH => vec![h(44), h(0), h(0)],
            Self::P2SH_P2WPKH => vec![h(49), h(0), h(0)],
            Self::P2TR => vec![h(86), h(0), h(0)],
            Self::P2SH => vec![h(45)],
            Self::P2SH_P2WSH => vec![h(48), h(0), h(0), h(1)],
            Self::P2WSH => vec![h(48), h(0), h(0), h(2)],
        }
    }

    /// The descriptor spelling, for §6's substituted remedies.
    pub fn descriptor_form(self) -> &'static str {
        match self {
            Self::P2SH => "sh",
            Self::P2SH_P2WSH => "sh(wsh(…))",
            Self::P2SH_P2WPKH => "sh(wpkh(…))",
            Self::P2PKH => "pkh",
            Self::P2WSH => "wsh",
            Self::P2WPKH => "wpkh",
            Self::P2TR => "tr",
        }
    }
}

/// One element of a use-site (children) path.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Derivation {
    Child { index: u32, hardened: bool },
    Wildcard { hardened: bool },
    Range { start: u32, end: u32 },
}

impl Derivation {
    /// `Derivation.Encode()` — `/i`, `/*`, `/<a;b>`, with a trailing `h` when
    /// hardened.
    pub fn encode(self) -> String {
        let mut s = String::from("/");
        match self {
            Self::Child { index, hardened } => {
                s.push_str(&index.to_string());
                if hardened {
                    s.push('h');
                }
            }
            Self::Wildcard { hardened } => {
                s.push('*');
                if hardened {
                    s.push('h');
                }
            }
            Self::Range { start, end } => {
                s.push('<');
                s.push_str(&start.to_string());
                s.push(';');
                s.push_str(&end.to_string());
                s.push('>');
            }
        }
        s
    }
}

/// A parsed key expression: `[fingerprint/origin]xpub/children`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    /// The operator's own bytes for this key, verbatim. §4.5's announcement
    /// exists to answer "is that my key?", and printing only the normalised
    /// form makes that check fail on a correct result (R0's I5).
    pub as_supplied: String,
    /// `0` means "master unknown"; `Descriptor.encode` then omits the `[…]`
    /// block entirely, which is why conjunct 6 does not bind such a key.
    pub fingerprint: u32,
    /// The EFFECTIVE derivation path — explicit if the input gave one, else the
    /// script's implied path (branch 2) or the SLIP-132 fallback (branch 4).
    pub origin: Vec<u32>,
    /// Whether the `[…]` origin block was actually present in the input.
    pub origin_explicit: bool,
    pub children: Vec<Derivation>,
    pub version: KeyVersion,
    pub network: Network,
    pub parent_fingerprint: u32,
    pub chain_code: [u8; 32],
    pub key_data: [u8; 33],
}

impl Key {
    /// The key's material identity — what makes two slots "the same key"
    /// (conjunct 8). Deliberately NOT the base58 string: two spellings of one
    /// key differ in version and depth bytes and are still one key.
    pub fn identity(&self) -> ([u8; 33], [u8; 32]) {
        (self.key_data, self.chain_code)
    }

    /// `Key.String()` — `ExtendedKey()` with the version normalised to the
    /// network's public id, `depth` rebuilt from `len(DerivationPath)` and
    /// `childNum` from its last element. This is why the canonical string can
    /// carry a base58 payload the operator has never seen.
    pub fn canonical_string(&self) -> String {
        let version: u32 = match self.network {
            Network::Mainnet => 0x0488_b21e,
            Network::Testnet => 0x0435_87cf,
        };
        let depth = self.origin.len() as u8;
        let child = self.origin.last().copied().unwrap_or(0);
        let mut b = Vec::with_capacity(78);
        b.extend_from_slice(&version.to_be_bytes());
        b.push(depth);
        b.extend_from_slice(&self.parent_fingerprint.to_be_bytes());
        b.extend_from_slice(&child.to_be_bytes());
        b.extend_from_slice(&self.chain_code);
        b.extend_from_slice(&self.key_data);
        base58::encode_check(&b)
    }
}

/// `m/…` rendering of a path, as `bip32.Path.String()` writes it (`h` for
/// hardened, never `'`).
pub fn path_string(path: &[u32]) -> String {
    let mut s = String::from("m");
    s.push_str(&path_encode(path));
    s
}

/// `bip32.Path.Encode()` — the `[fp<here>]` half, with no leading `m`.
pub fn path_encode(path: &[u32]) -> String {
    let mut s = String::new();
    for e in path {
        s.push('/');
        let hard = *e >= HARDENED;
        s.push_str(&(if hard { e - HARDENED } else { *e }).to_string());
        if hard {
            s.push('h');
        }
    }
    s
}

// ───────────────────────────────────────────────────────────────────────────
// The parsed descriptor
// ───────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Multi {
    /// `sortedmulti` — the only multi form the device's parser reads.
    Sorted,
    /// `multi` — refused by the device, carried natively by md1. `me` parses it
    /// because §6 must name it and §4.7 conjunct 1 admits it on the md1 path.
    Unsorted,
}

impl Multi {
    pub fn spelling(self) -> &'static str {
        match self {
            Self::Sorted => "sortedmulti",
            Self::Unsorted => "multi",
        }
    }
}

/// Which branch of §4's cascade produced this descriptor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Branch {
    BlueWallet,
    Bip380,
    Json,
    PromotedKey,
}

impl Branch {
    /// The value §7's `format` column carries. **Only a branch that SUCCEEDED
    /// has one** — see the module-level note on F-1 in `mod.rs`.
    pub fn format(self) -> &'static str {
        match self {
            Self::BlueWallet => "bluewallet",
            Self::Bip380 => "bip380",
            Self::Json => "json",
            Self::PromotedKey => "promoted-key",
        }
    }

    /// How §6's four-forms text names this branch.
    pub fn operator_name(self) -> &'static str {
        match self {
            Self::BlueWallet => "a BlueWallet `Key: value` setup file",
            Self::Bip380 => "a plain BIP-380 descriptor",
            Self::Json => "a `{\"label\":…,\"descriptor\":…}` JSON export",
            Self::PromotedKey => "a single extended key",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Parsed {
    pub branch: Branch,
    pub title: Option<String>,
    pub script: Script,
    /// `None` is `bip380.Singlesig`.
    pub multi: Option<Multi>,
    /// Parsed with `strconv.Atoi`'s permissiveness — a sign is accepted here
    /// and refused by conjunct 2, which is where the refusal TEXT lives.
    pub threshold: i64,
    pub keys: Vec<Key>,
    /// Set on branch 4: this wallet was inferred from one key, and §4.5 makes
    /// the inference ANNOUNCED rather than silent.
    pub promoted: bool,
}

impl Parsed {
    /// `Descriptor.Encode()` — the canonical re-encoded string WITH its BIP-380
    /// checksum. This is what §5.2 packs and what §4.5's announcement prints.
    pub fn encode(&self) -> String {
        let body = self.encode_no_checksum();
        match checksum::compute(&body) {
            Some(c) => format!("{body}#{c}"),
            // Unreachable: every byte the encoder emits is in the checksum
            // alphabet. Returning the body is still better than a panic on a
            // path that prints to an operator.
            None => body,
        }
    }

    fn encode_no_checksum(&self) -> String {
        let mut s = String::new();
        let mut parens = 1;
        if matches!(self.script, Script::P2SH_P2WSH | Script::P2SH_P2WPKH) {
            s.push_str("sh(");
            parens += 1;
        }
        s.push_str(match self.script {
            Script::P2SH => "sh",
            Script::P2PKH => "pkh",
            Script::P2WSH | Script::P2SH_P2WSH => "wsh",
            Script::P2WPKH | Script::P2SH_P2WPKH => "wpkh",
            Script::P2TR => "tr",
        });
        s.push('(');
        if let Some(m) = self.multi {
            s.push_str(m.spelling());
            s.push('(');
            s.push_str(&self.threshold.to_string());
            s.push(',');
            parens += 1;
        }
        for (i, k) in self.keys.iter().enumerate() {
            if k.fingerprint != 0 {
                s.push('[');
                s.push_str(&format!("{:08x}", k.fingerprint));
                s.push_str(&path_encode(&k.origin));
                s.push(']');
            }
            s.push_str(&k.canonical_string());
            for d in &k.children {
                s.push_str(&d.encode());
            }
            if i + 1 < self.keys.len() {
                s.push(',');
            }
        }
        for _ in 0..parens {
            s.push(')');
        }
        s
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Errors — §6's cause taxonomy, per branch
// ───────────────────────────────────────────────────────────────────────────

/// Branch 1's refusals. The first five are the Go parser's own line-level
/// errors; the rest are §4.2's NORMATIVE narrowings, which make `me`'s branch 1
/// FAIL where the device's succeeds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlueWalletError {
    InvalidHeaderLine(String),
    InconsistentHeader(String),
    InvalidPolicy(String),
    InvalidDerivation(String),
    UnknownFormat(String),
    InvalidCosignerKey(String),
    /// Not 8 hex characters. §4.2 defect 4 — the device PANICS below 4 bytes.
    BadFingerprint(String),
    /// `bw.Title == ""` — `OutputDescriptor`'s own admission gate
    /// (`parse.go:37`).
    NoName {
        headers: Vec<&'static str>,
        cosigners: usize,
    },
    /// F-419.
    ZeroCosigners,
    PolicyCount {
        declared: usize,
        found: usize,
        policy: String,
    },
    NoFormat,
    /// Any cosigner key would carry an empty origin path — R0's C1, stated over
    /// the KEYS and not over line order.
    NoOrigin {
        fingerprint: u32,
        after_keys: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bip380Error {
    InvalidChecksum,
    MissingOpenParen,
    MissingCloseParen,
    UnknownScriptType(String),
    InvalidWrappedScriptType(String),
    InvalidThreshold(String),
    Key(KeyError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyError {
    MissingCloseBracket,
    /// `[4bbaa801]xpub…` — 8 characters is too short for `originAndPath[8]=='/'`.
    FingerprintNoPath,
    InvalidFingerprint,
    InvalidOriginPath(String),
    InvalidChildrenPath(String),
    /// Not base58check, or not 78 bytes.
    NotAnExtendedKey,
    /// Parsed as an envelope, but the version is outside §4.3's five. Carries
    /// what §6's remedy needs: the version as supplied, the SAME key material
    /// re-serialised under the per-version target (`ypub` → `xpub`, the four
    /// testnet spellings → `tpub`), and the operator's own origin block if the
    /// input gave one — because "supply the descriptor" must print a descriptor
    /// they can run.
    UnsupportedVersion {
        version: KeyVersion,
        converted: String,
        origin: Option<String>,
    },
    /// A version byte no SLIP-132 spelling claims.
    UnknownVersion(u32),
    /// The 33 key-data bytes are not a compressed point on secp256k1, or are
    /// the private `0x00` form.
    NotAPublicPoint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonError {
    /// The document is not JSON at all, or is not a shape `json.Unmarshal`
    /// accepts into `struct{Label, Descriptor string}`.
    NotClaimed,
    /// The wrapper parsed and its `descriptor` field did not — the one format
    /// with a useful error message today (§4.1).
    Inner {
        label: String,
        inner: Box<Bip380Error>,
    },
}

#[derive(Clone, Debug)]
pub enum PromotionError {
    /// `bip380.ParseKey` refused the whole file.
    Key(KeyError),
    /// Parsed, but the path matches none of the three promotable scripts.
    PathNotInferable(Box<Key>),
    /// Parsed, purpose and coin type qualify, the ACCOUNT does not.
    AccountNotZero(Box<Key>),
    /// A `Zpub`/`Ypub`, whose version implies a multisig COSIGNER account.
    MultisigCosignerKey(KeyVersion),
    /// §4.5's ruling: `me` refuses `tpub` promotion entirely.
    TestnetKey,
}

/// Every branch's error, retained. §4.1: `me` reproduces the admission order
/// and does NOT reproduce the diagnostic loss.
#[derive(Clone, Debug, Default)]
pub struct Errors {
    pub bluewallet: Option<BlueWalletError>,
    pub bip380: Option<Bip380Error>,
    pub json: Option<JsonError>,
    pub promotion: Option<PromotionError>,
}

/// What the cascade produced: a descriptor, or every branch's reason.
pub type Outcome = Result<Parsed, Box<Errors>>;

// ───────────────────────────────────────────────────────────────────────────
// §4.1 — the cascade
// ───────────────────────────────────────────────────────────────────────────

/// Run §4's four branches in the device's order over an ALREADY-NORMALISED
/// input (see [`normalise`]).
///
/// **First branch that succeeds wins**, exactly as `OutputDescriptor` returns
/// immediately. The one divergence is the JSON branch's early return: the
/// device returns branch 3's failure without trying branch 4, and `me` keeps
/// going — which changes nothing about ADMISSION (a JSON document is never a
/// bare key) and lets §6 report a better cause.
pub fn cascade(input: &str) -> Outcome {
    let mut errs = Errors::default();
    match parse_bluewallet(input) {
        Ok(d) => return Ok(d),
        Err(e) => errs.bluewallet = Some(e),
    }
    match parse_bip380(input) {
        Ok(d) => return Ok(d),
        Err(e) => errs.bip380 = Some(e),
    }
    match parse_json(input) {
        Ok(d) => return Ok(d),
        Err(e) => errs.json = Some(e),
    }
    match promote(input) {
        Ok(d) => return Ok(d),
        Err(e) => errs.promotion = Some(e),
    }
    Err(Box::new(errs))
}

// ───────────────────────────────────────────────────────────────────────────
// §4.2 — branch 1, BlueWallet
// ───────────────────────────────────────────────────────────────────────────

/// The four headers `parseBlueWalletDescriptor` recognises. §5.1's gate test
/// T2 asks about the same four, and shares this constant so the two cannot
/// drift.
pub const BW_HEADERS: [&str; 4] = ["Name", "Policy", "Derivation", "Format"];

/// A `Key: value` line's key, if the line splits on the two-character
/// separator `": "`. Exposed because §5.1's gate test T2 asks the same
/// question of a line and must not ask it differently.
pub fn header_key(line: &str) -> Option<&str> {
    line.split_once(": ").map(|(k, _)| k)
}

/// Whether a string is exactly 8 hexadecimal characters — what `me` requires of
/// a BlueWallet cosigner fingerprint (§4.2 defect 4) and what
/// `bip380.ParseKey` already requires of an inline origin.
pub fn is_8_hex(s: &str) -> bool {
    s.len() == 8 && s.bytes().all(|c| c.is_ascii_hexdigit())
}

fn parse_bluewallet(input: &str) -> Result<Parsed, BlueWalletError> {
    use BlueWalletError as E;

    let mut title: Option<String> = None;
    let mut threshold: i64 = 0;
    let mut nkeys: usize = 0;
    let mut policy_seen: Option<String> = None;
    let mut path: Vec<u32> = Vec::new();
    let mut script: Option<Script> = None;
    let mut keys: Vec<Key> = Vec::new();
    let mut seen: std::collections::BTreeMap<String, String> = Default::default();
    // R0's C1: the rule is stated over the KEYS, not over line order. This
    // records, per origin-less key, whether a `Derivation:` had been seen WHEN
    // the cosigner line was read — which catches BOTH the after-the-keys
    // ordering and a file with no `Derivation:` header at all. Which of the two
    // it was is decided at the end, against whether the header EVER appeared:
    // `seen later` is the ordering defect, `never seen` is the missing one, and
    // the two want different sentences from §6.
    let mut origin_missing: Vec<(u32, bool)> = Vec::new();
    let mut derivation_seen = false;

    for line in input.split('\n') {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once(": ") else {
            return Err(E::InvalidHeaderLine(line.to_string()));
        };
        if let Some(old) = seen.get(key) {
            if old != val {
                return Err(E::InconsistentHeader(key.to_string()));
            }
            continue;
        }
        seen.insert(key.to_string(), val.to_string());
        match key {
            "Name" => title = Some(val.to_string()),
            "Policy" => {
                let (t, n) = parse_policy(val).ok_or_else(|| E::InvalidPolicy(val.to_string()))?;
                threshold = t;
                nkeys = n;
                policy_seen = Some(val.to_string());
            }
            "Derivation" => {
                path = parse_path_m(val).map_err(|_| E::InvalidDerivation(val.to_string()))?;
                derivation_seen = true;
            }
            "Format" => {
                script = Some(match val {
                    "P2WSH" => Script::P2WSH,
                    "P2SH" => Script::P2SH,
                    "P2WSH-P2SH" => Script::P2SH_P2WSH,
                    _ => return Err(E::UnknownFormat(val.to_string())),
                });
            }
            fp_hex => {
                // §4.2 defect 4, and this check comes FIRST because the device
                // PANICS on a short fingerprint: this file must never reach it.
                if !is_8_hex(fp_hex) {
                    return Err(E::BadFingerprint(line.to_string()));
                }
                let ext =
                    parse_extended_key(val).map_err(|_| E::InvalidCosignerKey(val.to_string()))?;
                let fp = u32::from_str_radix(fp_hex, 16)
                    .map_err(|_| E::BadFingerprint(line.to_string()))?;
                if path.is_empty() {
                    origin_missing.push((fp, derivation_seen));
                }
                keys.push(Key {
                    as_supplied: val.to_string(),
                    fingerprint: fp,
                    origin: path.clone(),
                    origin_explicit: true,
                    children: Vec::new(),
                    version: ext.version,
                    network: ext.version.network(),
                    parent_fingerprint: ext.parent_fingerprint,
                    chain_code: ext.chain_code,
                    key_data: ext.key_data,
                });
            }
        }
    }

    // The ADMISSION GATE at the call site (`parse.go:37`), lifted ahead of the
    // key-count check on purpose. The device's own order puts the count check
    // first, so `deadbeef: xpub…` fails there — but that file carries no
    // `Policy:` line at all, and §6's count row would then print
    // "`Policy: 0 of 0` declares 0 cosigners", which is FALSE about the
    // operator's file. §7's `gate/deadbeef-fronts-an-xpub` row pins the
    // outcome as the no-`Name:` row, and the no-`Name:` row is also the true
    // one. See design/agent-reports/IMPL-P1-report.md, finding F-2.
    if title.as_deref().unwrap_or("").is_empty() {
        return Err(E::NoName {
            headers: BW_HEADERS
                .iter()
                .copied()
                .filter(|h| *h != "Name" && seen.contains_key(*h))
                .collect(),
            cosigners: keys.len(),
        });
    }
    // F-419, before the count check: telling an operator their `Policy` count
    // is wrong when the export carries no cosigner lines at all describes the
    // symptom rather than the truncation.
    if keys.is_empty() {
        return Err(E::ZeroCosigners);
    }
    if nkeys != keys.len() {
        return Err(E::PolicyCount {
            declared: nkeys,
            found: keys.len(),
            policy: policy_seen.unwrap_or_default(),
        });
    }
    let Some(script) = script else {
        // §4.2 defect 1 — without it `Script` stays Unknown and the device's
        // own `Encode()` PANICS.
        return Err(E::NoFormat);
    };
    if let Some((fingerprint, seen_when_read)) = origin_missing.first().copied() {
        return Err(E::NoOrigin {
            fingerprint,
            after_keys: derivation_seen && !seen_when_read,
        });
    }

    Ok(Parsed {
        branch: Branch::BlueWallet,
        title,
        script,
        // `parse.go:80` — unconditionally. BlueWallet cannot produce a
        // single-sig descriptor here.
        multi: Some(Multi::Sorted),
        threshold,
        keys,
        promoted: false,
    })
}

/// `fmt.Sscanf(val, "%d of %d", …)`. Sscanf stops at the first mismatch and
/// ignores trailing input, which is reproduced: `"2 of 3 keys"` scans.
fn parse_policy(val: &str) -> Option<(i64, usize)> {
    let mut it = val.split_whitespace();
    let t: i64 = it.next()?.parse().ok()?;
    if it.next()? != "of" {
        return None;
    }
    let n: i64 = it.next()?.parse().ok()?;
    if n < 0 {
        return None;
    }
    Some((t, n as usize))
}

// ───────────────────────────────────────────────────────────────────────────
// §4.3 — branch 2, plain BIP-380
// ───────────────────────────────────────────────────────────────────────────

/// `Parse`'s inner `parseFunc`: cut `f(` … `)` off the front and the back, or
/// say which paren is missing. Advances `rest` only on success, exactly as the
/// Go closure leaves `desc` untouched when it returns an error — which is what
/// makes `sh(wpkh(KEY))` stay single-sig.
fn parse_func<'a>(rest: &mut &'a str) -> Result<String, Bip380Error> {
    let s: &'a str = rest;
    match s.find('(') {
        Some(i) => {
            if !s.ends_with(')') {
                return Err(Bip380Error::MissingCloseParen);
            }
            let f = s[..i].to_string();
            *rest = &s[i + 1..s.len() - 1];
            Ok(f)
        }
        None => Err(Bip380Error::MissingOpenParen),
    }
}

fn parse_bip380(input: &str) -> Result<Parsed, Bip380Error> {
    use Bip380Error as E;

    let (body, sum) = match input.split_once('#') {
        Some((b, c)) => (b, Some(c)),
        None => (input, None),
    };
    if let Some(c) = sum {
        if !checksum::verify(body, c) {
            return Err(E::InvalidChecksum);
        }
    }

    let mut rest = body;
    let outer = parse_func(&mut rest)?;
    let mut script = match outer.as_str() {
        "wsh" => Script::P2WSH,
        "pkh" => Script::P2PKH,
        "sh" => Script::P2SH,
        "wpkh" => Script::P2WPKH,
        "tr" => Script::P2TR,
        _ => return Err(E::UnknownScriptType(outer)),
    };

    let mut multi: Option<Multi> = None;
    // Mirrors the Go control flow exactly, including the part that matters
    // most: when the SECOND `parseFunc` fails after a wrapper, the multi switch
    // is SKIPPED and the descriptor stays single-sig — that is `sh(wpkh(KEY))`.
    if let Ok(mut inner) = parse_func(&mut rest) {
        let mut ok = true;
        if inner == "wpkh" || inner == "wsh" {
            if script != Script::P2SH {
                return Err(E::InvalidWrappedScriptType(inner));
            }
            script = if inner == "wpkh" {
                Script::P2SH_P2WPKH
            } else {
                Script::P2SH_P2WSH
            };
            match parse_func(&mut rest) {
                Ok(next) => inner = next,
                Err(_) => ok = false,
            }
        }
        if ok {
            multi = Some(match inner.as_str() {
                "sortedmulti" => Multi::Sorted,
                // `me`'s one widening of the device's grammar, and it is
                // required: §6 must NAME `multi` in its refusal, and §4.7
                // conjunct 1 carries it on the md1 path.
                "multi" => Multi::Unsorted,
                _ => return Err(E::UnknownScriptType(inner)),
            });
        }
    }

    let mut threshold: i64 = 1;
    let key_strs: Vec<&str> = match multi {
        None => vec![rest],
        Some(_) => {
            let args: Vec<&str> = rest.split(',').collect();
            threshold = parse_atoi(args[0]).ok_or_else(|| E::InvalidThreshold(rest.to_string()))?;
            args[1..].to_vec()
        }
    };

    let implied = script.derivation_path();
    let mut keys = Vec::with_capacity(key_strs.len());
    for k in key_strs {
        keys.push(parse_key(Some(&implied), k).map_err(E::Key)?);
    }

    Ok(Parsed {
        branch: Branch::Bip380,
        title: None,
        script,
        multi,
        threshold,
        keys,
        promoted: false,
    })
}

/// `strconv.Atoi`: an optional sign then decimal digits, nothing else, no
/// surrounding space.
fn parse_atoi(s: &str) -> Option<i64> {
    let (sign, digits) = match s.strip_prefix('-') {
        Some(d) => (-1i64, d),
        None => (1i64, s.strip_prefix('+').unwrap_or(s)),
    };
    if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse::<i64>().ok().map(|v| sign * v)
}

// ───────────────────────────────────────────────────────────────────────────
// §4.4 — branch 3, `{label, descriptor}` JSON
// ───────────────────────────────────────────────────────────────────────────

fn parse_json(input: &str) -> Result<Parsed, JsonError> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(input) else {
        return Err(JsonError::NotClaimed);
    };
    // `json.Unmarshal` into a struct succeeds for an object and for `null`, and
    // fails for every other top-level shape.
    let obj = match &v {
        serde_json::Value::Object(o) => Some(o),
        serde_json::Value::Null => None,
        _ => return Err(JsonError::NotClaimed),
    };
    // §4.4, NORMATIVE: field matching is case-INSENSITIVE, because Go's
    // `encoding/json` is — exact match first, then the first case-insensitive
    // one. A host that required lowercase would refuse a file the device takes.
    let field = |name: &str| -> Result<Option<&serde_json::Value>, JsonError> {
        let Some(o) = obj else { return Ok(None) };
        if let Some(v) = o.get(name) {
            return Ok(Some(v));
        }
        for (k, v) in o.iter() {
            if k.eq_ignore_ascii_case(name) {
                return Ok(Some(v));
            }
        }
        Ok(None)
    };
    let as_string = |v: Option<&serde_json::Value>| -> Result<String, JsonError> {
        match v {
            None | Some(serde_json::Value::Null) => Ok(String::new()),
            Some(serde_json::Value::String(s)) => Ok(s.clone()),
            // A non-string in either field makes `json.Unmarshal` itself fail,
            // so the branch is not claimed at all.
            Some(_) => Err(JsonError::NotClaimed),
        }
    };
    let label = as_string(field("label")?)?;
    let descriptor = as_string(field("descriptor")?)?;

    match parse_bip380(&descriptor) {
        Ok(mut d) => {
            d.branch = Branch::Json;
            d.title = Some(label);
            Ok(d)
        }
        Err(e) => Err(JsonError::Inner {
            label,
            inner: Box::new(e),
        }),
    }
}

// ───────────────────────────────────────────────────────────────────────────
// §4.5 — branch 4, the promoted bare key
// ───────────────────────────────────────────────────────────────────────────

/// The three paths that qualify, read from `Script.DerivationPath()`. Three
/// hardened components, coin type 0, account 0. Nothing else.
const PROMOTABLE: [Script; 3] = [Script::P2PKH, Script::P2WPKH, Script::P2SH_P2WPKH];

fn promote(input: &str) -> Result<Parsed, PromotionError> {
    let key = parse_key(None, input).map_err(PromotionError::Key)?;

    // §4.5, NORMATIVE and a RULING rather than a transcription: a testnet key
    // whose only claim to being a wallet is a version byte that maps to a
    // MAINNET derivation path is an inference the host declines to make. The
    // device stays wider; §7's invariant permits the host to be narrower.
    if key.version == KeyVersion::Tpub {
        return Err(PromotionError::TestnetKey);
    }

    for s in PROMOTABLE {
        if s.derivation_path() == key.origin {
            return Ok(Parsed {
                branch: Branch::PromotedKey,
                title: None,
                script: s,
                multi: None,
                threshold: 1,
                keys: vec![key],
                promoted: true,
            });
        }
    }

    // A `Zpub`/`Ypub` with no explicit origin: its version declares a MULTISIG
    // cosigner account, and §6 has a row that says so.
    if !key.origin_explicit && matches!(key.version, KeyVersion::ZpubCap | KeyVersion::YpubCap) {
        return Err(PromotionError::MultisigCosignerKey(key.version));
    }
    if is_account_shaped(&key.origin) {
        return Err(PromotionError::AccountNotZero(Box::new(key)));
    }
    Err(PromotionError::PathNotInferable(Box::new(key)))
}

/// `purpose'/0'/account'` with a promotable purpose and a NON-zero account —
/// §4.5's measured live near-miss, which deserves its own §6 row rather than
/// the generic "matches no script" one.
fn is_account_shaped(path: &[u32]) -> bool {
    if path.len() != 3 || path.iter().any(|e| *e < HARDENED) {
        return false;
    }
    let purpose = path[0] - HARDENED;
    let coin = path[1] - HARDENED;
    let account = path[2] - HARDENED;
    matches!(purpose, 44 | 49 | 84) && coin == 0 && account != 0
}

// ───────────────────────────────────────────────────────────────────────────
// Key expressions
// ───────────────────────────────────────────────────────────────────────────

struct ExtendedKey {
    version: KeyVersion,
    parent_fingerprint: u32,
    chain_code: [u8; 32],
    key_data: [u8; 33],
}

/// `bip380.ParseExtendedKey` — base58check, 78 bytes, and a version in §4.3's
/// admitted five.
fn parse_extended_key(s: &str) -> Result<ExtendedKey, KeyError> {
    let raw = base58::decode_check(s).ok_or(KeyError::NotAnExtendedKey)?;
    if raw.len() != 78 {
        return Err(KeyError::NotAnExtendedKey);
    }
    let version = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
    let mut key_data = [0u8; 33];
    key_data.copy_from_slice(&raw[45..78]);
    // `hdkeychain.NewKeyFromString` decompresses the point; a host that skipped
    // this would ADMIT a key the device REFUSES.
    if !secp::is_valid_compressed_pubkey(&key_data) {
        return Err(KeyError::NotAPublicPoint);
    }
    let Some(v) = KeyVersion::from_bytes(version) else {
        return Err(KeyError::UnknownVersion(version));
    };
    if !v.admitted() {
        // The per-version target, computed from the operator's OWN bytes: only
        // the four version bytes change, so the remedy names the wallet they
        // hold rather than a different one.
        let target: u32 = match v.network() {
            Network::Mainnet => 0x0488_b21e,
            Network::Testnet => 0x0435_87cf,
        };
        let mut re = raw.clone();
        re[..4].copy_from_slice(&target.to_be_bytes());
        return Err(KeyError::UnsupportedVersion {
            version: v,
            converted: base58::encode_check(&re),
            origin: None,
        });
    }
    let mut chain_code = [0u8; 32];
    chain_code.copy_from_slice(&raw[13..45]);
    Ok(ExtendedKey {
        version: v,
        parent_fingerprint: u32::from_be_bytes([raw[5], raw[6], raw[7], raw[8]]),
        chain_code,
        key_data,
    })
}

/// `bip380.ParseKey(impliedPath, enc)` — `[fingerprint/path]key/children`.
fn parse_key(implied: Option<&[u32]>, enc: &str) -> Result<Key, KeyError> {
    let as_supplied = enc.to_string();
    let mut k = enc;
    let mut fingerprint = 0u32;
    let mut origin: Option<Vec<u32>> = implied.map(|p| p.to_vec());
    let mut origin_explicit = false;

    if let Some(stripped) = k.strip_prefix('[') {
        let end = stripped.find(']').ok_or(KeyError::MissingCloseBracket)?;
        let origin_and_path = &stripped[..end];
        k = &stripped[end + 1..];
        if origin_and_path.len() < 9 || origin_and_path.as_bytes()[8] != b'/' {
            // `[4bbaa801]xpub…`: a fingerprint with nothing to match a script
            // against. §6 gives this its own row.
            return Err(KeyError::FingerprintNoPath);
        }
        if !is_8_hex(&origin_and_path[..8]) {
            return Err(KeyError::InvalidFingerprint);
        }
        fingerprint = u32::from_str_radix(&origin_and_path[..8], 16)
            .map_err(|_| KeyError::InvalidFingerprint)?;
        let p = parse_path(&origin_and_path[9..])
            .map_err(|_| KeyError::InvalidOriginPath(origin_and_path.to_string()))?;
        origin = Some(p);
        origin_explicit = true;
    }

    let mut children = Vec::new();
    if let Some(i) = k.find('/') {
        let tail = &k[i + 1..];
        k = &k[..i];
        children =
            parse_children(tail).map_err(|_| KeyError::InvalidChildrenPath(tail.to_string()))?;
    }

    let ext = parse_extended_key(k).map_err(|e| match e {
        // Enrich with the origin the input supplied, which only this frame has.
        KeyError::UnsupportedVersion {
            version, converted, ..
        } if origin_explicit => KeyError::UnsupportedVersion {
            version,
            converted,
            origin: Some(format!(
                "[{:08x}{}]",
                fingerprint,
                path_encode(origin.as_deref().unwrap_or(&[]))
            )),
        },
        other => other,
    })?;
    // The SLIP-132 fallback fires only where there is neither an implied nor an
    // explicit path — i.e. branch 4 alone, because branch 2 always supplies the
    // script's own path.
    let origin = match origin {
        Some(p) => p,
        // Unreachable: `parse_extended_key` already refused every version
        // without an implied script, so this arm exists only to stay total.
        None => match ext.version.implied_script() {
            Some(s) => s.derivation_path(),
            None => return Err(KeyError::NotAnExtendedKey),
        },
    };

    Ok(Key {
        as_supplied,
        fingerprint,
        origin,
        origin_explicit,
        children,
        version: ext.version,
        network: ext.version.network(),
        parent_fingerprint: ext.parent_fingerprint,
        chain_code: ext.chain_code,
        key_data: ext.key_data,
    })
}

/// `bip32.ParsePathElement`.
fn parse_path_element(p: &str) -> Result<u32, ()> {
    let (body, offset) = match p.strip_suffix('h').or_else(|| p.strip_suffix('\'')) {
        Some(b) => (b, HARDENED),
        None => (p, 0),
    };
    let idx: i64 = body.parse().map_err(|_| ())?;
    let iu32 = idx as u32;
    if i64::from(iu32) != idx || iu32.checked_add(offset).is_none() {
        return Err(());
    }
    Ok(iu32 + offset)
}

/// The components of `bip32.ParsePath("m/…")` AFTER the `m`. Key expressions
/// arrive already stripped, because `ParseKey` builds `"m/" + originAndPath[9:]`.
fn parse_path(p: &str) -> Result<Vec<u32>, ()> {
    p.split('/').map(parse_path_element).collect()
}

/// `bip32.ParsePath` proper — it REQUIRES the leading `m`, which is why a
/// BlueWallet `Derivation:` header must go through this one and not
/// [`parse_path`]. `Derivation: m` is a legal empty path, and then §4.2's
/// origin rule refuses the file.
fn parse_path_m(p: &str) -> Result<Vec<u32>, ()> {
    let mut it = p.split('/');
    if it.next() != Some("m") {
        return Err(());
    }
    it.map(parse_path_element).collect()
}

/// `bip380.parsePath` — the use-site (children) grammar.
fn parse_children(path: &str) -> Result<Vec<Derivation>, ()> {
    let mut out = Vec::new();
    for p in path.split('/') {
        let d = if p == "*" {
            Derivation::Wildcard { hardened: false }
        } else if p == "*'" || p == "*h" {
            Derivation::Wildcard { hardened: true }
        } else if p.len() > 2 && p.starts_with('<') && p.ends_with('>') {
            let inner = &p[1..p.len() - 1];
            // Cut on the FIRST `;`, so a three-element group and a reversed
            // pair are both parse REFUSALS, never admitted shapes.
            let (a, b) = inner.split_once(';').ok_or(())?;
            let start = parse_path_element(a)?;
            let end = parse_path_element(b)?;
            if start > end || start >= HARDENED || end >= HARDENED {
                return Err(());
            }
            Derivation::Range { start, end }
        } else {
            let e = parse_path_element(p)?;
            if e >= HARDENED {
                Derivation::Child {
                    index: e - HARDENED,
                    hardened: true,
                }
            } else {
                Derivation::Child {
                    index: e,
                    hardened: false,
                }
            }
        };
        out.push(d);
    }
    Ok(out)
}

// ───────────────────────────────────────────────────────────────────────────
// §6's cause selection — WHICH branch's error gets printed
// ───────────────────────────────────────────────────────────────────────────

/// Whether a token's leading segment before any `/` is a 78-byte base58check
/// payload — an extended-key ENVELOPE of any version, with or without a
/// use-site tail.
///
/// §6 step 4 and §5.1's gate test T3 are the same test, and share this function
/// so they cannot drift: a shape test, not a parse-success test. Every branch-4
/// row §6 must reach is a `ParseKey` FAILURE, so a success test could never
/// select them.
pub fn looks_like_an_extended_key(token: &str) -> bool {
    let head = token.split('/').next().unwrap_or(token);
    base58::decode_check(head).is_some_and(|p| p.len() == 78)
}

/// **Is the whole input a bitcoin ADDRESS?** §6 gives an address its own row —
/// *"that is a bitcoin address, not a descriptor"* — and step 5's generic
/// four-forms text would bury the one fact the operator needs.
///
/// The test is narrow on purpose, and it is a SHAPE test rather than a checksum
/// one: a mistyped address is still an address and earns the same sentence.
///
/// * a single token whose human-readable part is `bc`, `tb` or `bcrt` and whose
///   body is bech32/bech32m charset — segwit v0 and v1, both cases; or
/// * a single token that base58check-decodes to exactly 21 bytes under one of
///   the four version bytes bitcoin uses for P2PKH and P2SH.
///
/// Nothing in §4's four formats can match either: an extended key's payload is
/// 78 bytes, and no constellation record's HRP is one of the three.
pub fn is_bitcoin_address(input: &str) -> bool {
    let t = input.trim();
    if t.is_empty() || t.split_whitespace().nth(1).is_some() {
        return false;
    }
    // bech32 / bech32m — `bc1…`, `tb1…`, `bcrt1…`, either case, never mixed.
    let lower = t.to_ascii_lowercase();
    if t.chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        || t.chars().all(|c| !c.is_ascii_uppercase())
    {
        for hrp in ["bc", "tb", "bcrt"] {
            if let Some(body) = lower.strip_prefix(hrp).and_then(|r| r.strip_prefix('1')) {
                const CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
                if body.len() >= 6 && body.chars().all(|c| CHARSET.contains(c)) {
                    return true;
                }
            }
        }
    }
    // base58check P2PKH / P2SH, mainnet and testnet.
    matches!(base58::decode_check(t), Some(p)
        if p.len() == 21 && matches!(p[0], 0x00 | 0x05 | 0x6f | 0xc4))
}

/// Which branch §6 reports for an input no branch admitted, by the fixed
/// five-step rule. Whole-INPUT scope, deliberately unlike §5.1's per-LINE gate.
pub fn most_resembled(input: &str) -> Option<Branch> {
    // 1. parses as JSON
    if serde_json::from_str::<serde_json::Value>(input).is_ok() {
        return Some(Branch::Json);
    }
    // 2. first non-comment line contains `": "`
    if let Some(l) = input
        .split('\n')
        .find(|l| !l.is_empty() && !l.starts_with('#'))
    {
        if l.contains(": ") {
            return Some(Branch::BlueWallet);
        }
    }
    // 3. contains `(`
    if input.contains('(') {
        return Some(Branch::Bip380);
    }
    // 4. LOOKS like an extended key
    let trimmed = input.trim_start();
    if trimmed.starts_with('[') {
        return Some(Branch::PromotedKey);
    }
    if input.split_whitespace().nth(1).is_none() && looks_like_an_extended_key(input.trim()) {
        return Some(Branch::PromotedKey);
    }
    // 5. none of the above
    None
}

#[cfg(test)]
#[path = "cascade_tests.rs"]
mod tests;
