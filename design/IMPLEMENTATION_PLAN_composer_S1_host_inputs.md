# Composer Stage 1 Implementation Plan — host inputs: `key:`/`hash:`/`now:` records, `--no-now`, the lockstep fixture, `ms derive --template bip48-p2tr`, the payload-spec fold

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**STATUS: R0 GREEN 2026-09-02 (0 Critical / 0 Important open).** Round 0: fidelity (opus, 0C/6I/10M/3N) and mutation/claims (sonnet, 0C/3I/2M), folded together with the stand-in ruling on the `now:` default at `3919f1f`/`e20eae1`; round 1: fold verification 29 FIXED / 1 PARTIAL / 0 regressions, the partial and one prose slip folded here. Machine-checked in the build-gate scratch copy (toolchain 1.85.0) with Task 2 and the ruled Task 4 hand-wired: 612/612, clippy clean, threaded runner clean; fixture 40 rows, sha256 `eed6b177…464e`; mnemonic-secret Task 5 hand-applied 308/310 (two scratch artefacts). Lenses run: fidelity-to-spec + "what does it make false elsewhere", can-every-test-fail by mutation, fold verification. Implementation may begin: one implementer, UC off, two worktrees (Task 6 is already done).

**Goal:** Give the host everything the composer's device flow consumes from a payload — three new record classes with their body rules and refusal lines, the single-`now:` rule and the auto-appended pack time, a cross-language fixture the Go port is measured against, and an `ms derive` template that emits the device's taproot origin — and fold the payload spec so the normative table says what the code does.

**Architecture:** One new me-cli module, `sysw::composer_records`, owns the three prefixes, their parsers and their §8n lines; the existing classifier, admission and `show` paths gain arms that call it (fragments of existing files, hand-checked). The `now:` auto-append lives in the CLI, never in the library, so `pack_deterministic` stays a pure function of its inputs and the sysw vector fixture stays byte-stable. The lockstep fixture follows the descriptor-seam pattern: a Rust table generates a JSON file whose sha256 is pinned in both repos. `ms derive` gains one enum variant. The payload spec fold is the controller's own task with its own R0.

**Tech Stack:** Rust; `mnemonic-engrave` crate (`crates/me-cli`, binary `me`, CI toolchain 1.85.0 via `RUST_TOOLCHAIN` in `.github/workflows/release.yml`); `bitcoin` 0.32 (`bip32::{Xpub, DerivationPath, Fingerprint, ChildNumber}`, already a dependency); `mnemonic-secret` (`crates/ms-cli`, binary `ms`, 0.16.0); `cargo nextest run --locked` and CI's threaded `cargo test --locked` both; `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`.

**Spec:** `design/SPEC_wallet_policy_composer.md` §6a, §8n, §8r, §10 items 2, 5, 6, §12 item 8. Stages: `design/STAGED_PLAN_wallet_policy_composer.md` (S1). Payload spec under fold: `design/SPEC_systemwide_payloads.md` sections 3.3.1, 3.3.2, 5.3.

**Baselines (for `scripts/plan-staleness-check.sh`):** mnemonic-engrave `b44fb61`; mnemonic-secret `5f37b43` (ms-cli 0.16.0); descriptor-mnemonic `b19dca7b` (Stage 0 in flight on branch `composer-s0`); fork `169073c` (untouched by this stage).

## The `now:` default — RULED 2026-09-02 by a stand-in for the operator (revocable)

Spec §6a and §10 item 2 originally made the pack-time `now:` record the DEFAULT for every `me sysw pack`. Measured in this plan's scratch copy: every payload's public section grew by one record (+25 bytes, new identity digest, six pre-existing tests moved), and a seed-only payload gained a public record that only Wallet Policy admits — and the device shows a digest-compare screen for any payload with a public section. The stand-in ruling (`design/agent-reports/composer-S1-decision-now-default.md`; veto form "use (a)/(b)/(c-wide)"): **(c) narrowed** — `me sysw pack` appends `now:` by default ONLY when the operator's records include at least one `key:` or `hash:` record (the composer-only classes) and no `now:`; a new `--now` forces the append onto any payload; `--no-now` suppresses it; `--now` and `--no-now` conflict; a supplied `now:` always wins and appends nothing. Seeds, descriptors, md1/mk1 cards and `tx:` records do NOT trigger it, so every other program's payload packs byte-identically to today. Spec §6a, §10 item 2 and §7g carry the same words (folded with the S1 R0 round 0); brainstorm §3.12 item 21 records the supersession.

## Global Constraints

- Rust first: the fork's `sysw.Classify`, `syswSession.load` and the vendored fixture are Stage 2's; nothing under the submodule changes here.
- Record bodies are LOWERCASE hex after a reserved prefix, matched BEFORE the sniffers; a prefixed record whose body fails ANY rule is `Class::Unknown` and refused by `pack_with` with its own §8n line (spec §6a; payload spec section 5.3). None of the three classes is secret or bearer.
- `key:` body (UTF-8 text `[fingerprint/path]xpub`): NON-EMPTY origin; fingerprint 8 lowercase hex; a path that parses as BIP-32 notation with `'` or `h` as the hardened marker (uppercase `H` is out of scope on BOTH sides — the fork's `bip32.ParsePathElement` accepts only `h` and `'` too — and is pinned as a refused fixture row); components need not be hardened, but the xpub's own child number must equal the origin's last component and the origin component count must equal the xpub depth, which is 3 or 4; `tpub` is admitted (the network is the xpub's own). `hash:` body: exactly 32 bytes. `now:` body (UTF-8 text `<seconds>[,<height>]`): seconds `1..=2147483647`, height `1..=499999999` when present, digits only, at most 10 and 9 digits respectively.
- Payload-wide: at most ONE valid `now:` record; a second is a host refusal ("Remove one."), enforced where the whole payload is seen (`pack_with`'s split) — not in the per-record classifier.
- `me sysw pack` auto-appends `now:<hex of pack seconds>` as the LAST record ONLY when the operator's records contain no `now:` AND at least one classifies as `Key` or `Hash`, or when `--now` is passed; `--no-now` suppresses that auto-append; `--now` conflicts with `--no-now`; the library never appends anything.
- The §8n lines are the operator-visible refusals; `me sysw pack` emits them on stderr through `sysw_error()` in `main.rs`, prefixed `me: `. Every unclassifiable arm of `sysw_error` OPENS `record N (records count from 0)` — `tests/descriptor_seam.rs`'s `is_record_refusal` recognises the whole refusal surface by exactly that phrase — so the new arms open the same way and then carry the §8n blockquote VERBATIM on the next indented line: the spec's line is a substring of what prints, and the seam invariant stays true. Same shape for the second-`now:` refusal.
- `ms derive --template bip48-p2tr` = `m/48'/coin'/account'/3'`, purpose 48, script type `3'`; the shipped negative test flips to a positive one and is RENAMED, not deleted; its expected xpubs are the two-implementation oracle recorded in this plan (Go btcsuite hdkeychain and an independent Python BIP-32, measured 2026-09-02).
- The payload spec fold is a NORMATIVE artifact with its own R0 history: the controller performs it (Task 6) and runs its own review loop; the implementer does not touch `design/`.
- Build gate before every fold: `scripts/plan-build-gate-me.sh design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md` assembles `crates/me-cli/src/sysw/composer_records.rs` and `crates/me-cli/tests/sysw_composer_*.rs` under the CI toolchain, builds, runs them, clippies. Fragments (`sysw/record.rs`, `sysw/mod.rs`, `main.rs`, `tests/record_corpus.rs`, ms-cli) are hand-wired in the gate's scratch copy by the controller before review, as Stage 0 did.
- Commits: stage paths explicitly (no `git add -A`); one task, one commit, in the repo the task touches.

---

## File Structure

New files (assembled by the build gate):

| file | responsibility |
| --- | --- |
| `crates/me-cli/src/sysw/composer_records.rs` | the three prefixes; `ComposerRecord` and its parser; `ComposerRecordError` and the §8n lines; `now_record`, `key_record`, `hash_record` constructors; `now_indices`; the lockstep `CASES` table and its JSON shape |
| `crates/me-cli/tests/sysw_composer_records.rs` | unit-level tests of the parser and constructors, and the sha256-pinned fixture consumer test |
| `crates/me-cli/tests/sysw_composer_cli.rs` | `me sysw pack` refusals (§8n), `--no-now`, the auto-append, the single-`now:` rule, `me sysw show`'s new lines |

Modified files (fragments; a reviewer's execution pass, hand-wired by the controller at gate time):

| file | change |
| --- | --- |
| `crates/me-cli/src/sysw/mod.rs` | `pub mod composer_records;`; `classify_with` arms for the three prefixes; `UnknownReason::Composer(ComposerRecordError)`; `unknown_reason` arm; `SyswError::SecondNow(usize)`; the single-`now:` check in `split` |
| `crates/me-cli/src/sysw/record.rs` | `Class::{Key, Hash, Now}` (not secret, not bearer) |
| `crates/me-cli/src/main.rs` | `sysw_error` arms (§8n lines); `class_name` arms; `show` rendering for the three classes; `Pack { no_now }` and the auto-append |
| `crates/me-cli/tests/record_corpus.rs` | the one other exhaustive `match` on `Class` gains the three arms |
| `crates/me-cli/testdata/record_class_vectors.json` | GENERATED by the regenerate test, sha256 pinned in the consumer test |
| `crates/me-cli/CHANGELOG.md` | `[Unreleased]` entry |
| `mnemonic-secret/crates/ms-cli/src/cmd/derive.rs` | `Template::Bip48P2tr`; the two stale doc comments rewritten |
| `mnemonic-secret/crates/ms-cli/tests/cli_derive_bip48.rs` | the renamed, now-positive test with oracle constants |
| `mnemonic-secret/CHANGELOG.md`, `mnemonic-secret/design/FOLLOWUPS.md` | `## ms-cli [Unreleased]` entry; `ms-derive-taproot-justifications-stale` closed |
| `design/SPEC_systemwide_payloads.md` | sections 3.3.1, 3.3.2, 5.3 (Task 6, controller, own R0) |

---

### Task 1: The composer_records module — prefixes, parser, §8n lines

**Files:**
- Create: `crates/me-cli/src/sysw/composer_records.rs`
- Modify: `crates/me-cli/src/sysw/mod.rs` (one line after `pub mod coverage;`: `pub mod composer_records;`)
- Test: `crates/me-cli/tests/sysw_composer_records.rs`

**Interfaces:**
- Consumes: `bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub}` (dependency already present).
- Produces: `KEY_PREFIX`, `HASH_PREFIX`, `NOW_PREFIX`; `enum ComposerRecord { Key(KeyRecord), Hash([u8; 32]), Now { seconds: u32, height: Option<u32> } }`; `struct KeyRecord { fingerprint: Fingerprint, origin: DerivationPath, xpub: Xpub, text: String }`; `enum ComposerRecordError { Key(&'static str), Hash, Now }` with `fn line(&self, index: usize) -> String`; `fn parse(record: &str) -> Option<Result<ComposerRecord, ComposerRecordError>>`; `fn key_record(text: &str) -> String`, `fn hash_record(digest: &[u8; 32]) -> String`, `fn now_record(seconds: u32, height: Option<u32>) -> String`; `fn now_indices(records: &[String]) -> Vec<usize>`.

- [ ] **Step 1: Write the failing tests**

Create `crates/me-cli/tests/sysw_composer_records.rs`:

```rust
//! The composer's three payload record classes (SPEC_wallet_policy_composer.md
//! §6a): body rules, refusal lines (§8n), constructors, and the lockstep
//! fixture the Go port is measured against (§12 item 8).

use mnemonic_engrave::sysw::composer_records::{
    hash_record, key_record, now_indices, now_record, parse, ComposerRecord, ComposerRecordError,
    HASH_PREFIX, KEY_PREFIX, NOW_PREFIX,
};

/// The wallet-policy journey's cosigner @0: master 73c5da0a at m/48'/0'/0'/2'.
const KEY0: &str = "[73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const XPUB0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const H: [u8; 32] = [0xa8; 32];

fn hex(b: &[u8]) -> String {
    use std::fmt::Write as _;
    b.iter().fold(String::new(), |mut s, x| {
        let _ = write!(s, "{x:02x}");
        s
    })
}

// ---- constructors round-trip through the parser -----------------------------------

#[test]
fn a_key_record_is_the_prefix_plus_the_hex_of_the_origin_text() {
    let r = key_record(KEY0);
    assert!(r.starts_with(KEY_PREFIX));
    assert_eq!(r, format!("{KEY_PREFIX}{}", hex(KEY0.as_bytes())));
    match parse(&r) {
        Some(Ok(ComposerRecord::Key(k))) => {
            assert_eq!(k.text, KEY0);
            assert_eq!(k.fingerprint.to_string(), "73c5da0a");
            assert_eq!(k.origin.to_string(), "48'/0'/0'/2'");
            assert_eq!(k.xpub.to_string(), XPUB0);
            assert_eq!(k.xpub.depth, 4);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_hash_record_is_the_prefix_plus_64_lowercase_hex() {
    let r = hash_record(&H);
    assert_eq!(r, format!("{HASH_PREFIX}{}", "a8".repeat(32)));
    assert_eq!(parse(&r), Some(Ok(ComposerRecord::Hash(H))));
}

#[test]
fn a_now_record_encodes_seconds_and_an_optional_height() {
    let r = now_record(1_756_684_800, None);
    assert_eq!(r, format!("{NOW_PREFIX}{}", hex(b"1756684800")));
    assert_eq!(parse(&r), Some(Ok(ComposerRecord::Now { seconds: 1_756_684_800, height: None })));
    let r = now_record(1_756_684_800, Some(910_000));
    assert_eq!(r, format!("{NOW_PREFIX}{}", hex(b"1756684800,910000")));
    assert_eq!(parse(&r), Some(Ok(ComposerRecord::Now { seconds: 1_756_684_800, height: Some(910_000) })));
}

#[test]
fn records_without_one_of_the_three_prefixes_are_not_ours() {
    for r in ["text:48656c6c6f", "pass:00", "tx:00", "abandon abandon about", "md1ytpqqxpp3zcpydzk0zdt492xzr7r9qxfc", "", "key", "hash", "now", "Key:00", "KEY:00"] {
        assert_eq!(parse(r), None, "{r}");
    }
}

// ---- §6a body rules: every malformation is Some(Err(..)), never None -------------------

fn key_err(text: &str) -> ComposerRecordError {
    match parse(&key_record(text)) {
        Some(Err(e)) => e,
        other => panic!("{text}: expected a refusal, got {other:?}"),
    }
}

#[test]
fn a_bare_xpub_is_refused_naming_the_origin() {
    let e = key_err(XPUB0);
    assert!(matches!(e, ComposerRecordError::Key(_)), "{e:?}");
    assert!(e.line(3).contains("record 3"), "{}", e.line(3));
    assert!(e.line(3).contains("key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record"), "{}", e.line(3));
}

#[test]
fn key_origin_rules_are_each_enforced() {
    // Origin component count must equal the xpub's depth (4 here): last component 2' matches
    // the child number and the count alone is wrong, so this isolates the count rule.
    assert_eq!(key_err(&format!("[73c5da0a/48'/2']{XPUB0}")).detail(), "origin component count differs from the xpub's depth");
    assert_eq!(key_err(&format!("[73c5da0a/48'/0'/0'/0'/2']{XPUB0}")).detail(), "origin component count differs from the xpub's depth");
    // The xpub's own child number (2') must equal the origin's last component.
    assert!(matches!(key_err(&format!("[73c5da0a/48'/0'/0'/3']{XPUB0}")), ComposerRecordError::Key(_)));
    // Fingerprint must be 8 lowercase hex.
    assert!(matches!(key_err(&format!("[73C5DA0A/48'/0'/0'/2']{XPUB0}")), ComposerRecordError::Key(_)));
    assert!(matches!(key_err(&format!("[73c5da0/48'/0'/0'/2']{XPUB0}")), ComposerRecordError::Key(_)));
    // Path must parse (hardened with ' or h; no stray characters).
    assert!(matches!(key_err(&format!("[73c5da0a/48'/0'/x'/2']{XPUB0}")), ComposerRecordError::Key(_)));
    // Unhardened components are a legal PATH, so they are not refused here; F-166's
    // pathless case is; a mismatch against the xpub is what catches a wrong path.
    assert!(matches!(key_err(&format!("[73c5da0a/48/0/0/2]{XPUB0}")), ComposerRecordError::Key(_)), "2 unhardened != 2' hardened child number");
    // `h` spelling of hardened is accepted.
    assert!(matches!(parse(&key_record(&format!("[73c5da0a/48h/0h/0h/2h]{XPUB0}"))), Some(Ok(ComposerRecord::Key(_)))));
    // Not an xpub at all.
    assert!(matches!(key_err("[73c5da0a/48'/0'/0'/2']notanxpub"), ComposerRecordError::Key(_)));
    // Body not UTF-8, body not hex, body uppercase hex: all refusals of the KEY kind.
    assert!(matches!(parse("key:ff"), Some(Err(ComposerRecordError::Key(_)))));
    assert!(matches!(parse("key:zz"), Some(Err(ComposerRecordError::Key(_)))));
    assert!(matches!(parse(&format!("key:{}", hex(KEY0.as_bytes()).to_uppercase())), Some(Err(ComposerRecordError::Key(_)))));
    // An empty body.
    assert!(matches!(parse("key:"), Some(Err(ComposerRecordError::Key(_)))));
}

#[test]
fn hash_must_be_exactly_64_lowercase_hex() {
    assert_eq!(parse(&format!("hash:{}", "a8".repeat(31))), Some(Err(ComposerRecordError::Hash)));
    assert_eq!(parse(&format!("hash:{}", "a8".repeat(33))), Some(Err(ComposerRecordError::Hash)));
    assert_eq!(parse(&format!("hash:{}", "A8".repeat(32))), Some(Err(ComposerRecordError::Hash)));
    assert_eq!(parse("hash:"), Some(Err(ComposerRecordError::Hash)));
    assert_eq!(parse(&format!("hash:{}g", "a8".repeat(31))), Some(Err(ComposerRecordError::Hash)));
    assert_eq!(ComposerRecordError::Hash.line(0), "record 0: hash: must be exactly 64 hex characters");
}

#[test]
fn now_must_be_seconds_and_optional_height_in_range() {
    let bad = |text: &str| parse(&format!("{NOW_PREFIX}{}", hex(text.as_bytes())));
    for text in [
        "0", "2147483648", "12345678901", "abc", "", ",", "1756684800,", ",910000", "1756684800,0",
        "1756684800,500000000", "1756684800,1000000000", "1756684800,910000,1", " 1756684800", "1756684800 ", "+1756684800", "1756684800.0",
    ] {
        assert_eq!(bad(text), Some(Err(ComposerRecordError::Now)), "{text:?}");
    }
    for (text, want) in [
        ("1", (1, None)),
        ("2147483647", (2_147_483_647, None)),
        ("1756684800,1", (1_756_684_800, Some(1))),
        ("1756684800,499999999", (1_756_684_800, Some(499_999_999))),
    ] {
        assert_eq!(bad(text), Some(Ok(ComposerRecord::Now { seconds: want.0, height: want.1 })), "{text}");
    }
    assert!(matches!(parse("now:zz"), Some(Err(ComposerRecordError::Now))));
    assert!(matches!(parse("now:ff"), Some(Err(ComposerRecordError::Now))), "not UTF-8");
    assert_eq!(ComposerRecordError::Now.line(2), "record 2: now: must be <seconds>[,<height>] in range");
}

#[test]
fn now_indices_counts_only_valid_now_records() {
    let recs = vec![
        "text:48656c6c6f".to_string(),
        now_record(1_756_684_800, None),
        "now:zz".to_string(),
        now_record(1_756_684_801, Some(5)),
    ];
    assert_eq!(now_indices(&recs), vec![1, 3]);
}
```

- [ ] **Step 2: Run to verify the module is missing**

Run: `cargo nextest run --locked -p mnemonic-engrave --test sysw_composer_records 2>&1 | tail -4`
Expected: FAIL to compile, `could not find composer_records in sysw`.

- [ ] **Step 3: Write the module**

Create `crates/me-cli/src/sysw/composer_records.rs`:

```rust
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
    b.iter().fold(String::with_capacity(b.len() * 2), |mut s, x| {
        let _ = write!(s, "{x:02x}");
        s
    })
}

/// Strict: even length, every character in `0-9a-f`. Uppercase is NOT hex here
/// (section 5.3: the section is hashed in its canonical lowercase form).
fn unhex_lower(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 || !s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
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
    let text = std::str::from_utf8(&bytes).map_err(|_| K("body is not UTF-8"))?.to_owned();
    // `[fingerprint/path]xpub`: the origin is REQUIRED (an md1 slot carries a path).
    let rest = text.strip_prefix('[').ok_or(K("no [origin]: a bare xpub"))?;
    let (origin_text, xpub_text) = rest.split_once(']').ok_or(K("unterminated [origin]"))?;
    let (fp_text, path_text) = origin_text.split_once('/').ok_or(K("origin has no path"))?;
    if fp_text.len() != 8 || !fp_text.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err(K("fingerprint is not 8 lowercase hex characters"));
    }
    let fingerprint = Fingerprint::from_str(fp_text).map_err(|_| K("fingerprint does not parse"))?;
    let origin = DerivationPath::from_str(&format!("m/{path_text}")).map_err(|_| K("path does not parse"))?;
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
        return Err(K("the origin's last component is not the xpub's own child number"));
    }
    Ok(ComposerRecord::Key(KeyRecord { fingerprint, origin, xpub, text }))
}
```

Add to `crates/me-cli/src/sysw/mod.rs`, directly after `pub mod coverage;`:

```text
pub mod composer_records;
```

- [ ] **Step 4: Run the tests**

Run: `cargo nextest run --locked -p mnemonic-engrave --test sysw_composer_records 2>&1 | tail -12`
Expected: all PASS. If `DerivationPath::from_str` or `Fingerprint::from_str` behaves differently from the assertions (e.g. `h` spelling, empty-path handling), the `bitcoin` 0.32 crate is the authority for PARSING and the spec is the authority for RULES: fix the code, not the rule.

- [ ] **Step 5: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p mnemonic-engrave --all-targets -- -D warnings 2>&1 | tail -3`
Expected: clean.

```bash
git add crates/me-cli/src/sysw/composer_records.rs crates/me-cli/src/sysw/mod.rs crates/me-cli/tests/sysw_composer_records.rs
git commit -m "me sysw: composer_records -- key:/hash:/now: prefixes, body rules, 8n lines, constructors (composer S1 task 1)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 2: Classes, classifier arms, refusals, the single-`now:` rule

**Files:**
- Modify: `crates/me-cli/src/sysw/record.rs` (`Class` variants), `crates/me-cli/src/sysw/mod.rs` (`classify_with`, `UnknownReason`, `unknown_reason`, `SyswError`, `split`), `crates/me-cli/src/main.rs` (`sysw_error`, `class_name`), `crates/me-cli/tests/record_corpus.rs` (the one other exhaustive match)
- Test: `crates/me-cli/tests/sysw_composer_records.rs` (library-level), plus the unit tests inside `sysw/mod.rs`'s `mod tests`

**Interfaces:**
- Consumes: Task 1.
- Produces: `Class::Key`, `Class::Hash`, `Class::Now`; `UnknownReason::Composer(ComposerRecordError)`; `SyswError::SecondNow(usize)`; `classify`/`classify_with` returning the new classes; `pack_with` refusing a second `now:` with the index of the SECOND one.

- [ ] **Step 1: Write the failing library-level tests**

Add to `crates/me-cli/tests/sysw_composer_records.rs`:

```rust
// ---- the classifier, admission and the single-now rule ----------------------------------

use mnemonic_engrave::sysw::record::Class;
use mnemonic_engrave::sysw::{classify, pack_deterministic, SyswError, UnknownReason};

const ITER: u32 = 1;
const SALT: [u8; mnemonic_engrave::sysw::wire::SALT_LEN] = [7; mnemonic_engrave::sysw::wire::SALT_LEN];
const IV: [u8; mnemonic_engrave::sysw::wire::IV_LEN] = [9; mnemonic_engrave::sysw::wire::IV_LEN];

fn pack(records: Vec<String>) -> Result<Vec<u8>, SyswError> {
    pack_deterministic(records, None, ITER, SALT, IV)
}

#[test]
fn the_three_classes_classify_before_the_sniffers_and_are_not_secret() {
    assert_eq!(classify(&key_record(KEY0)), Class::Key);
    assert_eq!(classify(&hash_record(&H)), Class::Hash);
    assert_eq!(classify(&now_record(1_756_684_800, None)), Class::Now);
    for c in [Class::Key, Class::Hash, Class::Now] {
        assert!(!c.is_secret(), "{c:?}");
        assert!(!c.is_bearer(), "{c:?}");
        assert!(!c.is_argv_forbidden(), "{c:?}");
    }
}

#[test]
fn a_malformed_prefixed_record_is_unknown_and_refused_with_its_8n_line() {
    // A bare xpub as a key: body.
    let bare = key_record(XPUB0);
    assert_eq!(classify(&bare), Class::Unknown);
    match pack(vec!["text:48656c6c6f".into(), bare]) {
        Err(SyswError::Unclassifiable(1, UnknownReason::Composer(e))) => {
            assert!(matches!(e, ComposerRecordError::Key(_)));
            assert!(e.line(1).starts_with("record 1: key: needs [fingerprint/path]xpub"));
        }
        other => panic!("{other:?}"),
    }
    // A 63-character hash.
    let short = format!("hash:{}", "a8".repeat(31)).trim_end_matches('8').to_string();
    assert_eq!(classify(&short), Class::Unknown);
    assert_eq!(
        pack(vec![short]),
        Err(SyswError::Unclassifiable(0, UnknownReason::Composer(ComposerRecordError::Hash)))
    );
    // now: out of range.
    let zero = now_record(0, None);
    assert_eq!(classify(&zero), Class::Unknown);
    assert_eq!(
        pack(vec![zero]),
        Err(SyswError::Unclassifiable(0, UnknownReason::Composer(ComposerRecordError::Now)))
    );
}

#[test]
fn the_payload_holds_at_most_one_now_record() {
    let one = now_record(1_756_684_800, None);
    let two = now_record(1_756_684_801, Some(910_000));
    assert!(pack(vec!["text:48656c6c6f".into(), one.clone()]).is_ok());
    // The SECOND one is named, whatever sits between them.
    assert_eq!(
        pack(vec![one.clone(), "text:48656c6c6f".into(), two.clone()]),
        Err(SyswError::SecondNow(2))
    );
    assert_eq!(pack(vec![one, two]), Err(SyswError::SecondNow(1)));
}

#[test]
fn old_prefixes_and_unprefixed_records_classify_as_before() {
    assert_eq!(classify("text:48656c6c6f"), Class::FreeText);
    assert_eq!(classify("pass:48656c6c6f"), Class::Passphrase);
    assert_eq!(classify("text:zz"), Class::Unknown);
    assert_eq!(classify("not a record"), Class::Unknown);
}

#[test]
fn the_classes_pack_as_public_records_and_read_back() {
    let recs = vec![key_record(KEY0), hash_record(&H), now_record(1_756_684_800, Some(910_000))];
    let blob = pack(recs.clone()).unwrap();
    let opened = mnemonic_engrave::sysw::open(&blob, None).unwrap();
    assert_eq!(opened.public, recs);
    assert!(opened.secret.is_empty());
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo nextest run --locked -p mnemonic-engrave --test sysw_composer_records 2>&1 | tail -8`
Expected: FAIL to compile (`Class::Key`, `UnknownReason::Composer`, `SyswError::SecondNow` do not exist).

- [ ] **Step 3: Add the variants and arms**

In `crates/me-cli/src/sysw/record.rs`, add to `pub enum Class` directly after `Address,`:

```text
    /// `key:` — a cosigner `[fingerprint/path]xpub` for the composer's seating
    /// (SPEC_wallet_policy_composer.md §6a). Not secret, not bearer.
    Key,
    /// `hash:` — a 32-byte sha256 digest for a hashlock. Not secret.
    Hash,
    /// `now:` — the pack time (and optional height), a lower bound the device
    /// echoes. Not secret.
    Now,
```

(`is_secret`, `is_bearer` and `is_argv_forbidden` need no change: they `matches!` on the secret/bearer variants only.)

In `crates/me-cli/src/sysw/mod.rs`:

1. Add to `pub enum UnknownReason` a variant:

```text
    /// One of the composer's three prefixes with a body that fails §6a; the
    /// error carries which class and, for `key:`, the detail.
    Composer(composer_records::ComposerRecordError),
```

2. Add to `pub enum SyswError` a variant:

```text
    /// A second VALID `now:` record, at this index; the payload-wide rule
    /// (§6a) is enforced where the whole payload is seen.
    SecondNow(usize),
```

3. In `classify_with`, directly after the `TEXT_PREFIX` arm and before the BIP-39 sniffer:

```text
    if let Some(parsed) = composer_records::parse(record) {
        return match parsed {
            Ok(composer_records::ComposerRecord::Key(_)) => Class::Key,
            Ok(composer_records::ComposerRecord::Hash(_)) => Class::Hash,
            Ok(composer_records::ComposerRecord::Now { .. }) => Class::Now,
            Err(_) => Class::Unknown,
        };
    }
```

(On the HOST this placement is unobservable: `me` has no free-text fallback — `Class::FreeText` exists only behind `text:` — so no later sniffer can claim a `key:`/`hash:`/`now:` string, and the R0 tests lens measured that moving the arm below the BIP-39 sniffer changes nothing. The rule matters on the DEVICE, whose `Classify` falls back to free text; the lockstep fixture is what pins it there.)

4. In `unknown_reason`, directly after the `TX_PREFIX` block:

```text
    if let Some(Err(e)) = composer_records::parse(record) {
        return UnknownReason::Composer(e);
    }
```

5. In `split`, directly after `admit_check(&records, adm)?;`:

```text
    // §6a: at most ONE `now:` record. The SECOND valid one is named; a
    // malformed one was already refused above as Unknown.
    let nows = composer_records::now_indices(&records);
    if nows.len() > 1 {
        return Err(SyswError::SecondNow(nows[1]));
    }
```

In `crates/me-cli/src/main.rs`:

6. In `sysw_error`, inside the `E::Unclassifiable(i, why)` match, add an arm:

```text
                U::Composer(e) => format!(
                    "record {i} (records count from 0) is a `key:`/`hash:`/`now:` record whose \
                     body fails its rule ({}).\n      {}\n      Build the record with `me sysw \
                     pack`'s helpers: a key record is `key:` + the hex of `[fingerprint/path]xpub` \
                     exactly as `md decompose` prints it; a hash record is `hash:` + the 32-byte \
                     digest as 64 lowercase hex; a now record is `now:` + the hex of \
                     `<seconds>[,<height>]`.",
                    e.detail(),
                    e.line(*i)
                ),
```

and a top-level arm beside `E::TooLarge(n)`:

```text
        E::SecondNow(i) => format!(
            "record {i} (records count from 0) is a second now: record.\n      record {i}: a second \
             now: record; only one is allowed. Remove one."
        ),
```

7. In `class_name`, add arms before `C::Unknown`:

```text
        C::Key => "cosigner key (key:)",
        C::Hash => "sha256 hashlock (hash:)",
        C::Now => "pack time (now:)",
```

8. The repo has exactly TWO exhaustive matches on `Class` (measured: `main.rs`'s `class_name`, above, and `crates/me-cli/tests/record_corpus.rs:67-76`'s name table); the second gains `Class::Key => "Key"`, `Class::Hash => "Hash"`, `Class::Now => "Now"`. `crates/me-cli/src/sysw/expect.rs` has no match on `Class` (it uses `==`/`matches!`) and needs no change. `cargo build --all-targets` names any other site.

- [ ] **Step 4: Run the library tests, the whole me test suite, and the corpus**

Run: `cargo nextest run --locked -p mnemonic-engrave 2>&1 | tail -8`
Expected: all PASS. `record_corpus.rs` pins the classification of every pre-S2 record: none of them starts with one of the new prefixes, so none moves. **If any pre-existing test changes verdict, STOP and record it** — the new prefixes must not reclassify anything.

- [ ] **Step 5: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p mnemonic-engrave --all-targets -- -D warnings 2>&1 | tail -3`
Expected: clean.

```bash
git add crates/me-cli/src/sysw/record.rs crates/me-cli/src/sysw/mod.rs crates/me-cli/src/main.rs crates/me-cli/tests/record_corpus.rs crates/me-cli/tests/sysw_composer_records.rs
git commit -m "me sysw: Class::{Key,Hash,Now}, classifier arms, 8n refusals, the single-now rule (composer S1 task 2)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 3: The lockstep fixture — `record_class_vectors.json`


> **Post-implementation fold (2026-09-02, composer-S1-exec-review-r0 M-2 and composer-S2-plan-R0-r0-tests I-1):** `CASES` gained five rows after the whole-diff review — `now-unicode-digits` (Arabic-Indic digits refuse: `is_ascii_digit`, not `unicode.IsDigit`), `now-leading-zeros-valid` (admitted; `^[0-9]{1,10}$`), `key-body-odd-length`, `now-seconds-eleven-digits` and `now-height-ten-digits` (the §6a digit-COUNT bound, independent of the range bound). The fixture is 45 rows, sha256 `eed6b177d1a3406a69c4a0102635f5d59c6412fa65e106f85b831c4736ac464e`, regenerated by the `regenerate` test; the 40-row table below is the R0-time text and the worktree's `CASES` is authoritative.

**Files:**
- Modify: `crates/me-cli/src/sysw/composer_records.rs` (append the `CASES` table and the JSON row type)
- Test: `crates/me-cli/tests/sysw_composer_records.rs` (the regenerate test, `--ignored`; the sha256-pinned consumer test)
- Generated: `crates/me-cli/testdata/record_class_vectors.json`

**Interfaces:**
- Consumes: Task 1's `parse`, Task 2's `classify`.
- Produces: `pub struct Case { name, record, class, host_line }`, `pub const CASES: &[Case]`, `pub fn fixture_rows() -> Vec<FixtureRow>` (serde-serialisable). Stage 2's Go conformance test vendors the JSON and asserts `sysw.Classify(record) == class` for every row, with the same sha256 pinned (spec §12 item 8: "classifies identically on the host and on the device").

- [ ] **Step 1: Write the failing fixture tests**

Add to `crates/me-cli/tests/sysw_composer_records.rs`:

```rust
// ---- the lockstep fixture (spec §12 item 8) ---------------------------------------------

use mnemonic_engrave::sysw::composer_records::{fixture_rows, FixtureRow, CASES};
use sha2::Digest as _;

/// Pinned IDENTICALLY in the fork's `sysw/composer_records_conformance_test.go`
/// (Stage 2). Changing a row means changing this in both repos — the point.
/// Measured 2026-09-02 by running the regenerate test over CASES in the plan's
/// build-gate scratch copy; the regenerate test prints it again on every run.
const FIXTURE_SHA256: &str = "eed6b177d1a3406a69c4a0102635f5d59c6412fa65e106f85b831c4736ac464e";
const FIXTURE_PATH: &str = "testdata/record_class_vectors.json";

fn fixture_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_PATH)
}

#[test]
fn every_case_classifies_as_its_row_says_and_refuses_with_its_line() {
    for c in CASES {
        let got = classify(c.record);
        assert_eq!(format!("{got:?}"), c.class, "{}: {}", c.name, c.record);
        match parse(c.record) {
            Some(Err(e)) => assert_eq!(Some(e.line(0)), c.host_line.map(str::to_string), "{}", c.name),
            Some(Ok(_)) | None => assert_eq!(c.host_line, None, "{}", c.name),
        }
    }
}

#[test]
fn the_fixture_covers_every_class_and_every_8n_line_at_least_twice() {
    let mut classes = std::collections::BTreeMap::<&str, usize>::new();
    let mut lines = std::collections::BTreeMap::<String, usize>::new();
    for c in CASES {
        *classes.entry(c.class).or_default() += 1;
        if let Some(l) = c.host_line {
            // The line without its index, so "record 0:" does not split the tally.
            *lines.entry(l.trim_start_matches("record 0: ").to_string()).or_default() += 1;
        }
    }
    for cls in ["Key", "Hash", "Now", "Unknown"] {
        assert!(classes.get(cls).copied().unwrap_or(0) >= 2, "class {cls}: {classes:?}");
    }
    for l in [
        "key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record",
        "hash: must be exactly 64 hex characters",
        "now: must be <seconds>[,<height>] in range",
    ] {
        assert!(lines.get(l).copied().unwrap_or(0) >= 2, "line {l:?}: {lines:?}");
    }
    // §12 item 8: "each §6a malformation" — one named row per rule, so a Go port
    // that ignores depth, hard-codes depth 4, reads the fingerprint
    // case-insensitively (Go's hex.DecodeString does) or rejects tpub fails here.
    let names: std::collections::BTreeSet<&str> = CASES.iter().map(|c| c.name).collect();
    for required in [
        "key-journey-cosigner-0", "key-h-spelling", "key-depth-3-valid", "key-testnet-tpub-valid",
        "key-bare-xpub", "key-origin-no-path", "key-origin-shorter-than-depth", "key-origin-longer-than-depth",
        "key-last-component-mismatch", "key-depth-2-refused", "key-depth-5-refused",
        "key-fingerprint-uppercase", "key-fingerprint-7-hex", "key-origin-unterminated",
        "key-body-not-hex", "key-body-uppercase-hex", "key-body-not-utf8", "key-body-empty",
        "key-uppercase-H-marker-out-of-scope",
        "hash-valid", "hash-valid-zeros", "hash-31-bytes", "hash-63-chars", "hash-66-chars", "hash-uppercase", "hash-empty",
        "now-seconds-only", "now-seconds-and-height", "now-min", "now-max-both", "now-zero-seconds",
        "now-seconds-2^31", "now-height-zero", "now-height-at-time-threshold", "now-trailing-comma",
        "now-letters", "now-body-not-hex", "now-body-not-utf8", "now-body-uppercase-hex", "now-empty",
    ] {
        assert!(names.contains(required), "§6a rule without a fixture row: {required}");
    }
}

#[test]
fn the_committed_fixture_is_what_the_table_generates_and_carries_the_pinned_digest() {
    let bytes = std::fs::read(fixture_path()).expect("testdata/record_class_vectors.json exists; run the regenerate test");
    let digest = hex(&sha2::Sha256::digest(&bytes));
    assert_eq!(digest, FIXTURE_SHA256, "the fixture changed: re-pin here AND in the fork");
    let on_disk: Vec<FixtureRow> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(on_disk, fixture_rows(), "the committed file is not what CASES generates; regenerate");
}

/// Regenerate after an intentional change:
/// `cargo test --locked -p mnemonic-engrave --test sysw_composer_records regenerate -- --ignored --nocapture`
/// then paste the printed sha256 into FIXTURE_SHA256 (and, at Stage 2, the fork).
#[test]
#[ignore]
fn regenerate() {
    let rows = fixture_rows();
    let json = serde_json::to_string_pretty(&rows).unwrap() + "\n";
    std::fs::write(fixture_path(), &json).unwrap();
    println!("wrote {} rows to {}", rows.len(), fixture_path().display());
    println!("sha256 {}", hex(&sha2::Sha256::digest(json.as_bytes())));
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo nextest run --locked -p mnemonic-engrave --test sysw_composer_records 2>&1 | tail -6`
Expected: FAIL to compile (`CASES`, `fixture_rows`, `FixtureRow` missing).

- [ ] **Step 3: Write the table and the row type**

Add to `crates/me-cli/src/sysw/composer_records.rs` (append at the end):

```rust
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
```

The hex bodies above were GENERATED (Python `text.encode().hex()`, machine-checked against the plan text before commit), not typed; the first test of Task 3 re-derives every one through `parse`, so a mistyped body fails there, never silently.

- [ ] **Step 4: Run the consumer test, regenerate, pin, run again**

Run: `cargo nextest run --locked -p mnemonic-engrave --test sysw_composer_records 2>&1 | tail -6`
Expected: `every_case_classifies_as_its_row_says_and_refuses_with_its_line` and the coverage test PASS (40 rows; the depth-2/3/5 xpubs and the `tpub` are two-implementation oracles: the fork's Go `hdkeychain` and a Python BIP-32 agree byte for byte, 2026-09-02); `the_committed_fixture_...` FAILS (no file). Then:

```bash
cargo test --locked -p mnemonic-engrave --test sysw_composer_records regenerate -- --ignored --nocapture 2>&1 | grep -E 'wrote|sha256'
```

The printed `sha256` must equal the `FIXTURE_SHA256` already in the test (`eed6b177…464e`, measured when the plan was gated); if it does not, the table or the row type changed — find out why before re-pinning. Run the test file again: all PASS.

- [ ] **Step 5: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p mnemonic-engrave --all-targets -- -D warnings 2>&1 | tail -3`
Expected: clean.

```bash
git add crates/me-cli/src/sysw/composer_records.rs crates/me-cli/tests/sysw_composer_records.rs crates/me-cli/testdata/record_class_vectors.json
git commit -m "me sysw: record_class_vectors.json -- the composer classes' lockstep fixture, 40 rows (one per 6a rule), sha256-pinned (composer S1 task 3)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 4: `me sysw pack --now`/`--no-now`, the auto-appended pack time, `me sysw show`

**Files:**
- Modify: `crates/me-cli/src/main.rs` (`SyswCmd::Pack { now, no_now }`, the auto-append between `read_records` and `pack_with`, `show`'s per-record lines)
- Test: `crates/me-cli/tests/sysw_composer_cli.rs`

**Interfaces:**
- Consumes: Tasks 1-2.
- Produces: the CLI contract of spec §10 item 2 as RULED: default append only for a payload holding a `key:` or `hash:` record; `--now` forces; `--no-now` suppresses.

- [ ] **Step 1: Write the failing CLI tests**

Create `crates/me-cli/tests/sysw_composer_cli.rs`:

```rust
//! `me sysw pack` with the composer's records (SPEC_wallet_policy_composer.md
//! §6a, §8n, §10 item 2): refusal lines, `--no-now`, the auto-appended pack
//! time, the single-`now:` rule, and what `me sysw show` prints back.

use assert_cmd::Command;

const KEY0_TEXT: &str = "[73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const TEXT: &str = "text:48656c6c6f2c20576f726c6421";

fn me() -> Command {
    Command::cargo_bin("me").expect("me binary")
}

fn hex(s: &str) -> String {
    use std::fmt::Write as _;
    s.bytes().fold(String::new(), |mut o, b| {
        let _ = write!(o, "{b:02x}");
        o
    })
}

fn pack_to(dir: &tempfile::TempDir, extra: &[&str], records: &[&str]) -> (std::path::PathBuf, std::process::Output) {
    let out = dir.path().join("payload.bin");
    let mut args: Vec<String> = vec!["sysw".into(), "pack".into(), "--no-passphrase".into(), "--out".into(), out.display().to_string()];
    args.extend(extra.iter().map(|s| s.to_string()));
    args.extend(records.iter().map(|s| s.to_string()));
    let o = me().args(&args).output().unwrap();
    (out, o)
}

fn shown(path: &std::path::Path) -> String {
    let o = me().args(["sysw", "show", path.to_str().unwrap()]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout).unwrap()
}

const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn pack_appends_the_pack_time_when_a_composer_record_is_present_and_says_so() {
    // The RULED default: a payload holding a key: or hash: record gets its bound.
    let dir = tempfile::tempdir().unwrap();
    let hash = format!("hash:{}", "a8".repeat(32));
    let (path, o) = pack_to(&dir, &[], &[TEXT, &hash]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("appended now:"), "{err}");
    assert!(err.contains("--no-now"), "{err}");
    let s = shown(&path);
    assert!(s.contains("public record 2: pack time (now:)"), "{s}");
    // A key: record triggers it too.
    let key = format!("key:{}", hex(KEY0_TEXT));
    let (path, o) = pack_to(&dir, &[], &[&key]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(shown(&path).contains("public record 1: pack time (now:)"));
}

#[test]
fn a_payload_without_a_composer_record_packs_byte_identically_to_before() {
    // Seeds, text, cards: NO bound appended, no note; the six pre-existing pack
    // tests stay untouched for exactly this reason.
    let dir = tempfile::tempdir().unwrap();
    for records in [vec![TEXT], vec![SEED]] {
        let (path, o) = pack_to(&dir, &["--allow-argv-secret"], &records);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        assert!(!String::from_utf8_lossy(&o.stderr).contains("appended now:"), "{records:?}");
        assert!(!shown(&path).contains("now:"), "{records:?}");
    }
}

#[test]
fn now_forces_the_append_onto_any_payload_and_conflicts_with_no_now() {
    let dir = tempfile::tempdir().unwrap();
    let (path, o) = pack_to(&dir, &["--now"], &[TEXT]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(shown(&path).contains("public record 1: pack time (now:)"));
    let (_, o) = pack_to(&dir, &["--now", "--no-now"], &[TEXT]);
    assert!(!o.status.success(), "--now and --no-now must conflict");
}

#[test]
fn no_now_suppresses_the_auto_append_so_a_fixture_is_a_pure_function_of_its_inputs() {
    let dir = tempfile::tempdir().unwrap();
    let hash = format!("hash:{}", "a8".repeat(32));
    let (a, o) = pack_to(&dir, &["--no-now"], &[TEXT, &hash]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!String::from_utf8_lossy(&o.stderr).contains("appended now:"));
    let s = shown(&a);
    assert!(!s.contains("now:"), "{s}");
}

#[test]
fn an_operator_supplied_now_wins_silently_and_nothing_is_appended() {
    let dir = tempfile::tempdir().unwrap();
    let mine = format!("now:{}", hex("1756684800,910000"));
    let hash = format!("hash:{}", "a8".repeat(32));
    let (path, o) = pack_to(&dir, &[], &[TEXT, &hash, &mine]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!String::from_utf8_lossy(&o.stderr).contains("appended now:"));
    let s = shown(&path);
    assert!(s.contains("public record 2: pack time (now:) — 1756684800 (seconds), height 910000"), "{s}");
    assert_eq!(s.matches("pack time (now:)").count(), 1, "{s}");
}

#[test]
fn two_operator_supplied_now_records_are_refused_naming_the_second() {
    let dir = tempfile::tempdir().unwrap();
    let a = format!("now:{}", hex("1756684800"));
    let b = format!("now:{}", hex("1756684801"));
    let (_, o) = pack_to(&dir, &[], &[TEXT, &a, &b]);
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("record 2: a second now: record; only one is allowed. Remove one."), "{err}");
    assert!(err.contains("(records count from 0)"), "the seam's refusal vocabulary: {err}");
}

#[test]
fn a_second_now_is_refused_before_the_passphrase_ceremony() {
    // F-246: an admission failure must never leave the operator holding a
    // freshly printed passphrase for a payload that was then refused. A sealed
    // pack (a secret record, no --no-passphrase) with two now: records must
    // refuse WITHOUT printing "write this down".
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("payload.bin");
    let a = format!("now:{}", hex("1756684800"));
    let b = format!("now:{}", hex("1756684801"));
    let o = me()
        .args(["sysw", "pack", "--allow-argv-secret", "--out", out.to_str().unwrap(), SEED, &a, &b])
        .output()
        .unwrap();
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("a second now: record"), "{err}");
    assert!(!err.contains("write this down"), "the ceremony ran before the refusal: {err}");
}

#[test]
fn malformed_records_are_refused_with_the_8n_lines() {
    let dir = tempfile::tempdir().unwrap();
    let bare = format!("key:{}", hex("xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"));
    let (_, o) = pack_to(&dir, &[], &[TEXT, &bare]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("record 1: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record"));
    let (_, o) = pack_to(&dir, &[], &[&format!("hash:{}", "a8".repeat(31))]);
    assert!(String::from_utf8_lossy(&o.stderr).contains("record 0: hash: must be exactly 64 hex characters"));
    let (_, o) = pack_to(&dir, &[], &[&format!("now:{}", hex("0"))]);
    assert!(String::from_utf8_lossy(&o.stderr).contains("record 0: now: must be <seconds>[,<height>] in range"));
}

#[test]
fn show_prints_each_class_legibly() {
    let dir = tempfile::tempdir().unwrap();
    let key = format!("key:{}", hex(KEY0_TEXT));
    let hash = format!("hash:{}", "a8".repeat(32));
    let now = format!("now:{}", hex("1756684800"));
    let (path, o) = pack_to(&dir, &[], &[&key, &hash, &now]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let s = shown(&path);
    assert!(s.contains(&format!("public record 0: cosigner key (key:) — {KEY0_TEXT}")), "{s}");
    assert!(s.contains("public record 1: sha256 hashlock (hash:) — a8a8a8a8..a8a8a8a8"), "{s}");
    assert!(s.contains("public record 2: pack time (now:) — 1756684800 (seconds): a lower bound on the present the device echoes beside a time lock; never a locktime"), "{s}");
}
```

(`tempfile` is already a dev-dependency of me-cli; if it is not, add `tempfile = "3"` under `[dev-dependencies]` and say so in the commit.)

- [ ] **Step 2: Run to verify the tests fail**

Run: `cargo nextest run --locked -p mnemonic-engrave --test sysw_composer_cli 2>&1 | tail -8`
Expected: `no_now_suppresses...` and `now_forces_...` FAIL (unknown flags); `pack_appends_...` FAILS (nothing is appended); `a_payload_without_a_composer_record_packs_byte_identically_to_before` PASSES already (nothing appends today) — it is the regression guard for the ruled default; the refusal tests PASS already (Task 2) EXCEPT `a_second_now_is_refused_before_the_passphrase_ceremony`, which FAILS by design -- Task 2 refuses at `split`, AFTER the ceremony has printed the passphrase, and Step 3's hoisted pre-check is what moves the refusal in front of it (measured by the implementer, report `composer-S1-implementation-report.md`, deviation 1); `show_prints_each_class_legibly` FAILS (the show lines do not exist).

- [ ] **Step 3: Add the flag, the auto-append, the show lines**

In `crates/me-cli/src/main.rs`, in `SyswCmd::Pack { .. }`, add after `allow_unsigned_inputs: bool,`:

```text
        /// Append the pack time as a trailing `now:` record to THIS payload even
        /// though it holds no `key:`/`hash:` record. By default `pack` appends
        /// `now:<hex of unix seconds>` as the LAST record only when the records
        /// include a composer-only record (`key:` or `hash:`) and no `now:` of
        /// their own, so a Wallet Policy build gets a lower bound on the present
        /// beside a time lock (SPEC_wallet_policy_composer.md §6a) while every
        /// other program's payload packs exactly as before.
        #[arg(long, conflicts_with = "no_now")]
        now: bool,
        /// Never append the pack time, even when a `key:`/`hash:` record is
        /// present. A fixture whose output must be a pure function of its inputs
        /// passes this.
        #[arg(long)]
        no_now: bool,
```

and destructure `now, no_now` in the `SyswCmd::Pack { .. } =>` arm. Then, in TWO places. First, directly AFTER the F-246 block `if let Err(e) = sysw::admit_check(&recs, admission) { ... return EXIT_INVALID; }` (`crates/me-cli/src/main.rs:1563-1566`) and therefore BEFORE `decide_sealing` and the passphrase ceremony (`:1655-1682`):

```text
            // §6a's payload-wide single-`now:` rule, checked HERE as well as in
            // `pack_with`: F-246 hoisted admission ahead of the passphrase ceremony
            // so an operator is never told to write down twelve words that protect
            // no artifact; a second `now:` is an admission failure and takes the
            // same ordering. `split` keeps the library backstop. (A first draft put
            // this beside the auto-append below, AFTER the ceremony; the plan's own
            // test caught it: the passphrase printed before the refusal.)
            {
                let nows = mnemonic_engrave::sysw::composer_records::now_indices(&recs);
                if nows.len() > 1 {
                    eprintln!("me: {}", sysw_error(&mnemonic_engrave::sysw::SyswError::SecondNow(nows[1])));
                    return EXIT_INVALID;
                }
            }
```

Second, directly before `report_strength(passphrase.as_deref(), &recs);`:

```text
            // §6a / §10 item 2 as RULED (composer-S1-decision-now-default): append
            // the pack time ONLY when the operator gave no `now:` AND the payload
            // holds a composer-only record (`key:`/`hash:`) or `--now` says so. An
            // operator-supplied `now:` wins silently; two are refused by pack_with.
            // (`recs` was rebound immutably by the `--as` handling above.)
            let mut recs = recs;
            let composer_record_present = recs.iter().any(|r| {
                matches!(
                    mnemonic_engrave::sysw::classify(r),
                    mnemonic_engrave::sysw::record::Class::Key | mnemonic_engrave::sysw::record::Class::Hash
                )
            });
            if !*no_now
                && (*now || composer_record_present)
                && mnemonic_engrave::sysw::composer_records::now_indices(&recs).is_empty()
            {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                match u32::try_from(secs) {
                    Ok(s) if s >= 1 => {
                        recs.push(mnemonic_engrave::sysw::composer_records::now_record(s, None));
                        eprintln!(
                            "me: appended now:{s} as the last record (the pack time, a lower bound the \
                             device echoes next to a time lock; it is never a locktime). Pass --no-now \
                             to omit it, or supply your own now: record to pin a different bound. \
                             Payloads without a key:/hash: record get none unless --now is passed."
                        );
                    }
                    _ => eprintln!("me: the system clock is outside the now: band; no now: record appended"),
                }
            }
```

In the `show` path: `print_mdmk_confirmation` (`crates/me-cli/src/main.rs:2063`) slices the public section into `records` and calls `print_mt_confirmation(&records); print_descriptor_confirmation(&records);` (`:2088-2089`). Add a third call directly after those two:

```text
    print_composer_confirmation(&records);
```

and the helper beside `print_mt_confirmation`:

```text
/// `key:`/`hash:`/`now:` records, one line each (SPEC_wallet_policy_composer.md
/// §6a). Malformed ones never reach a container -- `pack` refuses them -- so a
/// record that fails to parse here is simply not one of ours.
fn print_composer_confirmation(records: &[String]) {
    use mnemonic_engrave::sysw::composer_records::{parse, ComposerRecord};
    for (i, r) in records.iter().enumerate() {
        let Some(Ok(rec)) = parse(r) else { continue };
        match rec {
            ComposerRecord::Key(k) => println!("public record {i}: cosigner key (key:) — {}", k.text),
            ComposerRecord::Hash(h) => {
                let hx = hex(&h);
                println!("public record {i}: sha256 hashlock (hash:) — {}..{}", &hx[..8], &hx[56..]);
            }
            ComposerRecord::Now { seconds, height } => {
                // `show` cannot tell an auto-appended pack time from an
                // operator-supplied bound, so it names neither provenance.
                let when = match height {
                    Some(h) => format!("{seconds} (seconds), height {h}"),
                    None => format!("{seconds} (seconds)"),
                };
                println!(
                    "public record {i}: pack time (now:) — {when}: a lower bound on the present the \
                     device echoes beside a time lock; never a locktime"
                );
            }
        }
    }
}
```

(`hex` is `main.rs`'s existing helper, defined right after the `Show` arm. A first draft of this plan put the loop INSIDE `print_mt_confirmation`, before its own loop; that function returns early when the payload holds no `mt1` chunk, so the lines never printed — measured in the gate's scratch copy. A helper with its own call site does not depend on a sibling's control flow.)

- [ ] **Step 4: Run the CLI tests and the whole suite**

Run: `cargo nextest run --locked -p mnemonic-engrave 2>&1 | tail -8 && cargo test --locked -p mnemonic-engrave 2>&1 | tail -4`
Expected: all PASS under both runners (CI uses the threaded `cargo test`) with NO pre-existing test changed. Under the plan's first draft (default-on append) six pre-existing tests failed because the appended `now:` record added 25 bytes (`\n` + `now:` + 20 hex) to the public section and moved `pub_len`, the identity and the record count: `descriptor_as::item_1_every_format_packs_one_descriptor_record`, `descriptor_as::item_2_every_format_packs_reads_back_and_derives_the_device_address`, `sysw_cli::a_payload_past_the_old_8191_cap_packs_and_reads_back`, `sysw_cli::an_incomplete_set_still_packs_and_is_readable`, `sysw_cli::a_secrets_only_payload_reports_no_digest`, `sysw_cli::the_descriptor_show_block_leaves_every_other_container_byte_identical` — and a SEVENTH the first draft missed, `sysw_cli::show_reports_exactly_one_descriptor_record_for_each_of_the_four_formats` (same mechanism; found by the R0 tests lens). None of those payloads holds a `key:` or `hash:` record, so under the RULED default they must pass untouched — that is the point of the ruling, and `a_payload_without_a_composer_record_packs_byte_identically_to_before` pins it. In particular `descriptor_as::item_1_every_format_packs_one_descriptor_record` keeps measuring `SPEC_descriptor_input.md`'s "one descriptor, one container, one invocation" on the command an operator types: a descriptor is NOT a composer-only record, so `--as descriptor` packs exactly one public record, as before. If any of the six still fails, the predicate is wrong: stop and record it. Any OTHER pre-existing failure is a finding too.

- [ ] **Step 5: Format, clippy, commit**

Run: `cargo fmt --all && cargo clippy --locked -p mnemonic-engrave --all-targets -- -D warnings 2>&1 | tail -3`
Expected: clean.

```bash
git add crates/me-cli/src/main.rs crates/me-cli/tests/sysw_composer_cli.rs
git commit -m "me sysw pack: --now/--no-now, the pack time appended when a key:/hash: record is present, show lines for key:/hash:/now: (composer S1 task 4)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 5: `ms derive --template bip48-p2tr` (mnemonic-secret)

**Files:**
- Modify: `mnemonic-secret/crates/ms-cli/src/cmd/derive.rs` (`Template` enum, `purpose`, `script_type`, `script_type_label`, the two doc comments), `mnemonic-secret/crates/ms-cli/tests/cli_derive_bip48.rs` (rename + flip the negative test), `mnemonic-secret/CHANGELOG.md`, `mnemonic-secret/design/FOLLOWUPS.md`
- Repo: `/scratch/code/shibboleth/mnemonic-secret` at `5f37b43`, in a worktree (`git worktree add /scratch/code/shibboleth/wt-ms-bip48-p2tr -b bip48-p2tr`)

**Interfaces:**
- Produces: `--template bip48-p2tr` deriving `m/48'/coin'/account'/3'`, never annotated ASSUMED; JSON `account_path` accordingly. (`script_type_label` gets an arm for completeness, but that label prints only for the DEFAULTED bare `bip48` — `mnemonic-secret/crates/ms-cli/src/cmd/derive.rs:504-518` — so it is unobservable for this template; the test asserts the path, not the label.)

**The oracle (measured 2026-09-02, two independent implementations agreeing byte for byte — the fork's Go `btcutil/v2/hdkeychain` run inside the fork module, and a hand-rolled Python BIP-32 over `ecdsa`; seed = BIP-39 of "abandon" ×11 + "about", empty passphrase, master 73c5da0a):**

| path | xpub |
| --- | --- |
| `m/48'/0'/0'/3'` | `xpub6DkFAXWQ2dHxr7LX1ByDVebj6u3C5KSKTVXWkiVKb3tdYfh9t7FhXzvUVSxNSikoVTRb2bGjvYoW8PqYBReMeswi3megtqDwRCeVs3vxMeH` |
| `m/48'/0'/1'/3'` | `xpub6DzhyrnFFYQ1KXnhK7D7U1sD9jf9Cq2E5Ut5HhXdXZFVgEpjz4jNsvEnL1FzP2p4RkMW7MTJC7GWK8CqEWdZsM4XR7Yn8BbbUieRkaTntL2` |
| `m/48'/0'/2'/3'` | `xpub6EGx8sPr9FxPQtmPagzaNqpcvG1JsN9m9tFyimaK4tUdfx3kxmJ76M25uDyZVD1mvrH8H1UcX24dVWLEqa51Li5x39WGpWc2eG2jTZdMzrR` |
| `m/48'/0'/3'/3'` | `xpub6E6Z3Ss5TXJYQKLeD76XTFYJXyVQzT5FBKY3a7evG61SuqJKBVF2EqzMWydzSEbhyj4ESvnBLpdL8Pde5sSUNL9Y9d6mY214mwuvbspUMK5` |

(The shipped `P2WSH_ACCT0` for `m/48'/0'/0'/2'` shares the parent `m/48'/0'/0'`; the two differ only in the last component, which is exactly what the test must be able to see.)

- [ ] **Step 1: Flip the negative test into the positive one, renamed; fix the module doc**

In `mnemonic-secret/crates/ms-cli/tests/cli_derive_bip48.rs`, FIRST replace the module-doc sentences (lines 18-21) "There is no registered Taproot value, so none is offered here; inventing one would put funds at a path no other wallet looks at." with: "BIP-48 registers no Taproot value; `bip48-p2tr` (`3'`) is offered anyway because it is a live CONVENTION (Coldcard `bip48_3`, Liana `p2tr_deriv`) and the SeedHammer II composer's own origin for seed-derived taproot slots — see `derive.rs`'s type doc." (the same false claim the follow-up `ms-derive-taproot-justifications-stale` names at two other sites). THEN replace the whole `an_unregistered_script_type_is_refused` test (its doc comment included) with:

```text
/// BIP-48 registers only 1' and 2', but the constellation's taproot multisig
/// origin is `m/48'/coin'/account'/3'` (composer spec C28; Coldcard's `bip48_3`,
/// Liana's `p2tr_deriv`), so the template exists and derives there. Oracle:
/// two independent BIP-32 implementations (fork Go hdkeychain, Python ecdsa),
/// 2026-09-02, recorded in mnemonic-engrave's
/// IMPLEMENTATION_PLAN_composer_S1_host_inputs.md Task 5.
const P2TR_ACCT0: &str = "xpub6DkFAXWQ2dHxr7LX1ByDVebj6u3C5KSKTVXWkiVKb3tdYfh9t7FhXzvUVSxNSikoVTRb2bGjvYoW8PqYBReMeswi3megtqDwRCeVs3vxMeH";
const P2TR_ACCT1: &str = "xpub6DzhyrnFFYQ1KXnhK7D7U1sD9jf9Cq2E5Ut5HhXdXZFVgEpjz4jNsvEnL1FzP2p4RkMW7MTJC7GWK8CqEWdZsM4XR7Yn8BbbUieRkaTntL2";

#[test]
fn bip48_p2tr_derives_the_composer_taproot_origin() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip48-p2tr"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let s = out(&o);
    assert!(s.contains(MASTER_FP), "{s}");
    assert!(s.contains("m/48'/0'/0'/3'"), "{s}");
    assert!(s.contains(P2TR_ACCT0), "{s}");
    assert!(!s.contains(P2WSH_ACCT0), "3' and 2' must not collapse: {s}");
    assert!(
        !err(&o).contains("ASSUMED"),
        "an explicit script type is a choice, not an assumption"
    );
    let o = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip48-p2tr",
        "--account",
        "1",
    ]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(out(&o).contains(P2TR_ACCT1), "{}", out(&o));
    assert!(out(&o).contains("m/48'/0'/1'/3'"), "{}", out(&o));
}

#[test]
fn bip48_p2tr_json_names_the_path_and_no_assumption() {
    let o = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip48-p2tr",
        "--json",
    ]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["account_path"], "m/48'/0'/0'/3'");
    assert_eq!(v["account_xpub"], P2TR_ACCT0);
    assert_eq!(v["script_type_defaulted"], false);
}
```

(`serde_json = "1"` is a regular dependency of ms-cli, and Cargo puts regular dependencies in scope for integration tests, so `serde_json::Value` compiles here; the existing `json_carries_the_assumption_flag` test does substring matching instead — either style is fine, this one asserts the three keys `mnemonic-secret/crates/ms-cli/src/cmd/derive.rs:485-487` emits.)

- [ ] **Step 2: Run to verify they fail**

Run: `cd /scratch/code/shibboleth/wt-ms-bip48-p2tr && cargo nextest run --locked -p ms-cli --test cli_derive_bip48 2>&1 | tail -6`
Expected: the two new tests FAIL (`bip48-p2tr` is not a valid value); every other test PASSES.

- [ ] **Step 3: Add the variant and rewrite the two stale comments**

In `mnemonic-secret/crates/ms-cli/src/cmd/derive.rs`:

1. Replace the type-level doc paragraph beginning "There is no `bip48-p2tr`: BIP-48 registers no Taproot script type, and inventing one would derive to a path no other wallet looks at." with:

```text
/// `bip48-p2tr` is `m/48'/coin'/account'/3'`. BIP-48 registers only `1'` and
/// `2'` (bips PR #1473 proposing `3'` closed unmerged 2024-05-14), so `3'` is a
/// CONVENTION, not a registration -- but a real one: Coldcard exports `bip48_3`
/// as `m/48h/{coin}h/{acct}h/3h` marked multisig, Liana's corpus consumes a
/// Coldcard export carrying `"p2tr_deriv": "m/48h/1h/0h/3h"`, and
/// mnemonic-toolkit sweeps the path as `bip48-tr-multi-a`. The SeedHammer II
/// composer declares exactly this origin for its seed-derived taproot slots
/// (mnemonic-engrave SPEC_wallet_policy_composer.md C28), so a host-derived
/// `key:` record must be able to match it. This is where permissiveness still
/// STOPS: no bare `bip48` assumes taproot, and no template invents a path no
/// wallet reads.
```

2. Add the variant directly after `Bip48,`:

```text
    /// BIP-48-shaped taproot multisig (script_type 3'): the composer's origin
    /// for seed-derived taproot slots. A convention shared with Coldcard and
    /// Liana, not a BIP-48 registration -- see the type doc.
    #[value(name = "bip48-p2tr")]
    Bip48P2tr,
```

3. In `purpose`, add `Template::Bip48P2tr` to the `=> 48` arm (exhaustive: the compiler enforces it). In `script_type`, add `Template::Bip48P2tr => Some(3),` and in `script_type_label` add `Template::Bip48P2tr => "3' p2tr (taproot multisig; Coldcard/Liana convention)",` — BOTH of those matches end in a wildcard arm (`_ => None`, `_ => ""`), so omitting either compiles and silently derives `m/48'/0'/0'` at depth 3; only the new test's `m/48'/0'/0'/3'` assertion catches it. `script_type_defaulted` is unchanged (only the bare `bip48` defaults).

4. In the `Bg002hTr` variant's doc comment, replace the sentence "BIP-48 registers no taproot script type, so a `tr()` multisig key has no standard 4-level home -- and md requires depth 4 for any multisig script context." with:

```text
    /// BIP-48 registers no taproot script type; `bip48-p2tr` (3') is the
    /// convention the composer and Coldcard/Liana use, and this purpose is the
    /// constellation's own alternative for a layout that can never be mistaken
    /// for BIP-48. (md admits account xpubs at depth 3 OR 4 --
    /// descriptor-mnemonic `parse/keys.rs`, `matches!(depth, 3 | 4)` -- so
    /// depth is not the reason for either template.)
```

- [ ] **Step 4: Run the whole ms suite, format, clippy**

Run: `cargo nextest run --locked -p ms-cli 2>&1 | tail -6 && cargo test --locked 2>&1 | tail -3 && cargo fmt --all --check && cargo clippy --locked --all-targets -- -D warnings 2>&1 | tail -3`
Expected: all PASS; `the_single_sig_template_names_are_unchanged`, `bg002h_templates_derive_the_ruled_path` and `bg002h_wsh_is_not_labelled_as_nested_segwit` still PASS; clean. Measured 2026-09-02: neither `gen_man.rs` nor `gui_schema_emits_spec_v7_json.rs` enumerates template values (the first derives its page set from the subcommand inventory, the second asserts `encode --language`'s choices), so no snapshot changes.

- [ ] **Step 5: Changelog, follow-up, commit**

Add to `mnemonic-secret/CHANGELOG.md`, above `## ms-cli [0.16.0] — 2026-08-15`:

```text
## ms-cli [Unreleased]

- `ms derive --template bip48-p2tr` derives `m/48'/coin'/account'/3'`, the
  SeedHammer II composer's origin for seed-derived taproot multisig slots
  (Coldcard `bip48_3` / Liana `p2tr_deriv` convention; not a BIP-48
  registration). The negative test asserting refusal became a positive one
  pinned to a two-implementation oracle. The `derive.rs` doc comments that
  claimed "no wallet looks at 48'/…/3'" and "md requires depth 4" are
  corrected (follow-up `ms-derive-taproot-justifications-stale`, closed).
```

In `mnemonic-secret/design/FOLLOWUPS.md`, change the `ms-derive-taproot-justifications-stale` entry's status line to `- **Status:** CLOSED 2026-09-xx by composer Stage 1 (bip48-p2tr added; both comments rewritten). **Tier:** was docs + feature.` and, under "What to do", note that `bg002h-tr` KEEPS its purpose as the constellation's own alternative (the composer spec §4f rules `48'/…/3'` for seed-derived slots; nothing in this stage removes `bg002h-tr`).

```bash
git add crates/ms-cli/src/cmd/derive.rs crates/ms-cli/tests/cli_derive_bip48.rs CHANGELOG.md design/FOLLOWUPS.md
git commit -m "ms derive: --template bip48-p2tr (m/48'/coin'/account'/3'), stale taproot justifications rewritten, negative test flipped and renamed (composer S1 task 5)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

---

### Task 6: The payload spec fold — CONTROLLER task, own R0 — DONE 2026-09-02

**Status: executed and closed before this plan's R0.** Fold `12e0659`; its R0 round 0 (opus, 0C/3I/4M/5N, `bb49953`) folded at `44765d7` with F-450 filed at `72ac66d`; round 1 verification 11 FIXED / 1 PARTIAL (`de34664`), the partial closed at `fdf7671`. `SPEC_systemwide_payloads.md` now carries the three class rows, the CREATED Wallet Policy row with all ten cells and three new columns, the three prefixes in 5.3, and the dated departures in 3.3.2's "RULED AT FOLD" paragraph. The steps below are the record of what was done; the implementer skips this task.

**Files:**
- Modify: `design/SPEC_systemwide_payloads.md` sections 3.3.1, 3.3.2, 5.3 (and section 12's normative definitions if they enumerate classes or prefixes — check at fold time with `grep -n 'text:\|ClassFreeText' design/SPEC_systemwide_payloads.md`)
- Reports: `design/agent-reports/payload-spec-composer-fold-R0-r0-correctness.md` (opus), `design/agent-reports/payload-spec-composer-fold-R0-r1-fold-verification.md` (sonnet)

This task is performed by the controller, not the implementer, because the artifact is normative and has its own R0 history. It may run in parallel with Tasks 1-5 (different files) and must be GREEN before the stage exits.

- [ ] **Step 1: Make the three edits**

Section 3.3.1 — add three rows to the class table, after `ClassAddress`:

```text
| `ClassKey` | no | **NEW** (§5.3; composer spec §6a): `key:<hex of "[fingerprint/path]xpub">` |
| `ClassHash` | no | **NEW** (§5.3; composer spec §6a): `hash:<64 lowercase hex>`, the digest itself |
| `ClassNow` | no | **NEW** (§5.3; composer spec §6a): `now:<hex of "<seconds>[,<height>]">`, the pack time |
```

and, after the paragraph on `ClassFreeText`, one paragraph: none of the three is secret — a cosigner xpub, a digest and a timestamp are public by construction; `ClassNow` is a LOWER BOUND on the present that the device echoes beside a time lock and never encodes.

Section 3.3.2 — the table gains three columns `Key | Hash | Now` (blank for every existing row) and the CREATED row, placed after `Engrave Multisig`:

```text
| Wallet Policy | • | • | • | | • | • | | • | • | • |
```

with its reason recorded in the bullet list below the table: the composer (mnemonic-engrave `SPEC_wallet_policy_composer.md` C12) admits a seed so the device can fill slots from it, exactly as Multisig Build does; the flag rules F1/F2 of 3.3.3 therefore fire inside the composer's seed step. Note, in the same list, that `progTransaction` has an admission map in the fork (`gui/sysw_admit.go`) and NO row here — filed for its own owner, not created by this fold.

Section 5.3 — the reserved-prefix list gains `key:`, `hash:`, `now:` with the same rule (lowercase hex body, matched before the sniffers, non-hex body is `ClassUnknown` and refused), and a pointer to the composer spec §6a for the per-class body rules and §8n for the lines.

- [ ] **Step 2: Gates**

Run: `./scripts/spec-structure-check.sh design/SPEC_systemwide_payloads.md` (if the file uses the lettered-subsection structure; otherwise skip and say so), `./scripts/plan-table-check.sh design/SPEC_systemwide_payloads.md`, `./scripts/plan-cite-check.sh design/SPEC_systemwide_payloads.md`, `./scripts/plan-glyph-check.sh design/SPEC_systemwide_payloads.md`.
Expected: both exit 0; the glyph gate prints 2 undrawable glyphs and the cite gate 1 dangling citation (`record.rs:74-108`), ALL PRE-EXISTING at 59e6f12 -- the implementer proved it by running both gates on the base file (identical findings, glyph lines shifted +1 by the N9 rewrap; report deviation 3). Nothing new from this edit is the pass condition.

- [ ] **Step 3: Commit the fold, dispatch R0**

Commit the edit alone. Dispatch an opus correctness/consistency lens on the fold (`git diff` of the spec; "what did this make false elsewhere in the same document"; the three columns against the fork's `gui/sysw_admit.go` map which is the table's transcription) writing the report named above; persist; fold; sonnet verification; close at 0C/0I.

---

### Task 7: Whole-repo gates and the me changelog

**Files:**
- Modify: `crates/me-cli/CHANGELOG.md` (`## [Unreleased]`)

- [ ] **Step 1: Run the repo the way CI does**

Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings 2>&1 | tail -3 && cargo nextest run --locked 2>&1 | tail -4 && cargo test --locked 2>&1 | tail -3`
Expected: clean and green under both runners.

- [ ] **Step 2: The sysw vector fixture is byte-stable**

Run: `git status --short crates/me-cli/testdata/ && cargo test --locked -p mnemonic-engrave --lib sysw::vectors 2>&1 | tail -3`
Expected: only the NEW `record_class_vectors.json` is untracked-then-committed; `sysw_vectors.json` is unchanged (the library appends nothing).

- [ ] **Step 3: The sentences this stage makes false, corrected in the same commit**

Four sites enumerate the reserved prefixes as three and become false when the three composer prefixes land (measured by the R0 fidelity lens, 2026-09-02):

1. `design/SPEC_sh2_sysw_consumption.md:265`, N9 — "`text:`, `pass:` and `tx:` require **lowercase** hex bodies" → "`text:`, `pass:`, `tx:` and, since the composer's Stage 1, `key:`, `hash:` and `now:` require **lowercase** hex bodies" (one clause; that spec's own gates run on it: `plan-glyph-check.sh`, `plan-cite-check.sh`).
2. `crates/me-cli/src/main.rs:2693-2698`, the `U::Unrecognised` message — "not a `text:`/`pass:`/`tx:` record" → "not a `text:`/`pass:`/`tx:`/`key:`/`hash:`/`now:` record".
3. `crates/me-cli/src/main.rs:183-206`, `SyswCmd::Pack`'s `--help` doc — after the `tx:` paragraph add: "`key:<hex of \"[fingerprint/path]xpub\">`, `hash:<64 lowercase hex>` and `now:<hex of \"<seconds>[,<height>]\">` feed the SeedHammer II's Wallet Policy composer: a cosigner key for seating, a sha256 hashlock digest, and the pack time (a lower bound the device echoes beside a time lock; appended for you when a `key:`/`hash:` record is present — see `--now`/`--no-now`)."
4. `crates/me-cli/src/sysw/record.rs:25-31`, the reserved-prefix doc paragraph — add one sentence: "Three more reserved prefixes — `key:`, `hash:`, `now:` — live in `composer_records.rs` with their own body rules; `decode_body` below strips only the three declared here, and every caller guards it with its own prefix test."

Run `cargo nextest run --locked -p mnemonic-engrave 2>&1 | tail -3` again (the message edits touch two tests' expectations at most; fix the expectation only if the assertion was about the old wording).

- [ ] **Step 4: Changelog and commit**

Add under `## [Unreleased]` in `crates/me-cli/CHANGELOG.md`:

```text
- `me sysw pack`: three new record classes for the SeedHammer II composer —
  `key:` (a cosigner `[fingerprint/path]xpub`), `hash:` (a 32-byte sha256
  digest), `now:` (the pack time and optional height). Bodies are lowercase
  hex; a malformed body is refused with its own line, before any passphrase is
  printed. At most one `now:` per payload. When the records include a `key:`
  or `hash:` record and no `now:`, `pack` appends the pack time as a trailing
  `now:`; `--now` forces that onto any payload, `--no-now` suppresses it, and
  a supplied `now:` always wins — payloads without a composer record pack
  byte-identically to before. `me sysw show` prints the three.
  `testdata/record_class_vectors.json` is the lockstep fixture (40 rows, one
  per §6a rule) the device's classifier is measured against (composer spec §12
  item 8).
```

```bash
git add crates/me-cli/CHANGELOG.md design/SPEC_sh2_sysw_consumption.md crates/me-cli/src/main.rs crates/me-cli/src/sysw/record.rs
git commit -m "me: changelog for the composer's record classes; the three-prefix sentences corrected (N9, the Unrecognised message, pack --help, record.rs) (composer S1 task 7)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA"
```

- [ ] **Step 5: Hand off**

The stage is complete when: every task's commit exists (Tasks 1-4 and 7 on a mnemonic-engrave branch; Task 5 on the mnemonic-secret worktree branch); Task 6's payload-spec fold is GREEN under its own R0 (it is); the whole-diff independent review (opus execution review over each repo's diff, persisted to `design/agent-reports/composer-S1-exec-review-r0.md`) returns 0 Critical / 0 Important after folds; `me` and `ms` are released per each repo's release process (`crates/me-cli/CHANGELOG.md` + `.github/workflows/release.yml` here; `design/RELEASE_PROCESS.md` in mnemonic-secret). Stage 2 begins only then, and it vendors `record_class_vectors.json` with its pinned sha256 (vendoring is S2's first act, not S1's exit — the staged plan says the same).

---

## Self-review (run by the plan author before dispatch)

1. **Spec coverage, Stage 1 scope:** §6a body rules → Task 1 parser + tests; §6a "matched before the sniffers" and "none is secret" → Task 2; §8n lines → Task 1 `line()` rendered by Task 2's `sysw_error` arm and Task 4's CLI tests; single-`now:` rule at `pack_with` → Task 2 `split`; auto-append as RULED (composer-only records trigger it; `--now`/`--no-now`) (§10 item 2) → Task 4; §12 item 8 host half → Task 3 fixture, one row per §6a rule (device half: Stage 2 vendors it); §10 item 5 → Task 5; §10 item 6 → Task 6. Device-visible signals (§8r "Keys loaded", "not understood") are Stage 3's.
2. **Placeholder scan:** `FIXTURE_SHA256` carries the digest measured in the plan's scratch copy (the regenerate test re-derives it); the CHANGELOG date in Task 5's follow-up close is written as `2026-09-xx` because the closing date is the commit's — the implementer fills the day. No other TBD.
3. **Type consistency:** `parse(&str) -> Option<Result<ComposerRecord, ComposerRecordError>>`, `ComposerRecordError::line(&self, usize) -> String`, `now_indices(&[String]) -> Vec<usize>`, `Case { name, record, class, host_line }`, `fixture_rows() -> Vec<FixtureRow>`, `SyswError::SecondNow(usize)`, `UnknownReason::Composer(ComposerRecordError)` are used with these shapes in every task.

## What the build gate covers, and does not

`scripts/plan-build-gate-me.sh design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md` assembles `crates/me-cli/src/sysw/composer_records.rs` (Tasks 1 and 3), `crates/me-cli/tests/sysw_composer_records.rs` (Tasks 1-3) and `crates/me-cli/tests/sysw_composer_cli.rs` (Task 4) into a scratch copy of this repo under CI's toolchain, registers the module, builds all targets, runs the composer test binaries and clippies. Because Task 2's variants live in fragments the gate does not assemble, the Task 2 and Task 3 tests that reference `Class::Key`, `UnknownReason::Composer` or `SyswError::SecondNow` will not compile in the bare gate; the controller hand-wires the Task 2 and Task 4 fragments into the scratch copy (as Stage 0 did for its Task 8) and runs the whole `mnemonic-engrave` suite there before any reviewer sees the plan. Re-measured 2026-09-02 with the RULED Task 4 (the `key:`/`hash:` predicate, `--now`, `--no-now`) hand-wired beside Task 2's fragments: `cargo nextest run -p mnemonic-engrave --locked --no-fail-fast` → 612 tests run, 612 passed, 2 skipped — the six tests the default-on draft had moved pass untouched, every composer test passes, and the regenerated fixture's sha256 is the pinned `eed6b177…464e`.

mnemonic-secret's Task 5 is fragments only and was hand-checked the same way in a scratch copy of that repo (2026-09-02, toolchain 1.85.0): with the variant, the three arms and the flipped test applied, `cargo nextest run -p ms-cli` ran 310 tests, 308 passed including both new `bip48_p2tr_*` tests; the two failures are scratch artefacts (`format::conformance::conformance_vectors_pass` reads `design/display-grouping-vectors.tsv`, which the copy of `crates/` alone lacks; `the_display_grouping_conformance_pin_is_untouched` shells out to `sha256sum`, absent from the scratch PATH) and pass in the repo itself. The payload spec fold is prose and has its own gates.
