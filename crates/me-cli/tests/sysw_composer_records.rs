//! The composer's three payload record classes (SPEC_wallet_policy_composer.md
//! §6a): body rules, refusal lines (§8n), constructors, and the lockstep
//! fixture the Go port is measured against (§12 item 8).

use mnemonic_engrave::sysw::composer_records::{
    hash_record, key_record, now_indices, now_record, parse, phrase_record, ComposerRecord,
    ComposerRecordError, HashLock, HashlockMethod, RecordHashKind, HASH_PREFIX, KEY_PREFIX,
    NOW_PREFIX,
};

/// The wallet-policy journey's cosigner @0: master 73c5da0a at m/48'/0'/0'/2'.
const KEY0: &str = "[73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const XPUB0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const H: [u8; 32] = [0xa8; 32];

/// `H` as the sha256 lock it has always meant. Existing rows keep asserting the
/// BARE record, which §6's producer rule says stays byte-identical.
fn hl() -> HashLock {
    HashLock::new(RecordHashKind::Sha256, &H[..]).expect("32 bytes is sha256's width")
}

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
    let r = hash_record(&hl());
    assert_eq!(r, format!("{HASH_PREFIX}{}", "a8".repeat(32)));
    assert_eq!(parse(&r), Some(Ok(ComposerRecord::Hash(hl()))));
}

#[test]
fn a_now_record_encodes_seconds_and_an_optional_height() {
    let r = now_record(1_756_684_800, None);
    assert_eq!(r, format!("{NOW_PREFIX}{}", hex(b"1756684800")));
    assert_eq!(
        parse(&r),
        Some(Ok(ComposerRecord::Now {
            seconds: 1_756_684_800,
            height: None
        }))
    );
    let r = now_record(1_756_684_800, Some(910_000));
    assert_eq!(r, format!("{NOW_PREFIX}{}", hex(b"1756684800,910000")));
    assert_eq!(
        parse(&r),
        Some(Ok(ComposerRecord::Now {
            seconds: 1_756_684_800,
            height: Some(910_000)
        }))
    );
}

#[test]
fn records_without_one_of_the_three_prefixes_are_not_ours() {
    for r in [
        "text:48656c6c6f",
        "pass:00",
        "tx:00",
        "abandon abandon about",
        "md1ytpqqxpp3zcpydzk0zdt492xzr7r9qxfc",
        "",
        "key",
        "hash",
        "now",
        "Key:00",
        "KEY:00",
    ] {
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
    assert!(
        e.line(3).contains(
            "key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record"
        ),
        "{}",
        e.line(3)
    );
}

#[test]
fn key_origin_rules_are_each_enforced() {
    // Origin component count must equal the xpub's depth (4 here): last component 2' matches
    // the child number and the count alone is wrong, so this isolates the count rule.
    assert_eq!(
        key_err(&format!("[73c5da0a/48'/2']{XPUB0}")).detail(),
        "origin component count differs from the xpub's depth"
    );
    assert_eq!(
        key_err(&format!("[73c5da0a/48'/0'/0'/0'/2']{XPUB0}")).detail(),
        "origin component count differs from the xpub's depth"
    );
    // The xpub's own child number (2') must equal the origin's last component.
    assert!(matches!(
        key_err(&format!("[73c5da0a/48'/0'/0'/3']{XPUB0}")),
        ComposerRecordError::Key(_)
    ));
    // Fingerprint must be 8 lowercase hex.
    assert!(matches!(
        key_err(&format!("[73C5DA0A/48'/0'/0'/2']{XPUB0}")),
        ComposerRecordError::Key(_)
    ));
    assert!(matches!(
        key_err(&format!("[73c5da0/48'/0'/0'/2']{XPUB0}")),
        ComposerRecordError::Key(_)
    ));
    // Path must parse (hardened with ' or h; no stray characters).
    assert!(matches!(
        key_err(&format!("[73c5da0a/48'/0'/x'/2']{XPUB0}")),
        ComposerRecordError::Key(_)
    ));
    // Unhardened components are a legal PATH, so they are not refused here; F-166's
    // pathless case is; a mismatch against the xpub is what catches a wrong path.
    assert!(
        matches!(
            key_err(&format!("[73c5da0a/48/0/0/2]{XPUB0}")),
            ComposerRecordError::Key(_)
        ),
        "2 unhardened != 2' hardened child number"
    );
    // A `+`-signed component is refused (rust-bitcoin would have admitted it; the device does
    // not; composer-S2-exec-review-r0 I-1), and so is an unhardened component of 2^31 (C-1).
    assert_eq!(
        key_err(&format!("[73c5da0a/+48'/0'/0'/2']{XPUB0}")).detail(),
        "path component is not digits with an optional ' or h"
    );
    assert!(matches!(
        key_err(&format!("[73c5da0a/2147483648/0'/0'/2']{XPUB0}")),
        ComposerRecordError::Key(_)
    ));
    // `h` spelling of hardened is accepted.
    assert!(matches!(
        parse(&key_record(&format!("[73c5da0a/48h/0h/0h/2h]{XPUB0}"))),
        Some(Ok(ComposerRecord::Key(_)))
    ));
    // Not an xpub at all.
    assert!(matches!(
        key_err("[73c5da0a/48'/0'/0'/2']notanxpub"),
        ComposerRecordError::Key(_)
    ));
    // Body not UTF-8, body not hex, body uppercase hex: all refusals of the KEY kind.
    assert!(matches!(
        parse("key:ff"),
        Some(Err(ComposerRecordError::Key(_)))
    ));
    assert!(matches!(
        parse("key:zz"),
        Some(Err(ComposerRecordError::Key(_)))
    ));
    assert!(matches!(
        parse(&format!("key:{}", hex(KEY0.as_bytes()).to_uppercase())),
        Some(Err(ComposerRecordError::Key(_)))
    ));
    // An empty body.
    assert!(matches!(
        parse("key:"),
        Some(Err(ComposerRecordError::Key(_)))
    ));
}

#[test]
fn a_descriptor_form_key_names_the_suffix_not_the_xpub() {
    // Copied out of a descriptor rather than `md decompose --emit keys`: refused (the rule wants
    // the account key alone), and the detail must say so rather than "not an extended public key".
    let e = key_err(&format!("[73c5da0a/48'/0'/0'/2']{XPUB0}/<0;1>/*"));
    assert_eq!(
        e.detail(),
        "the key carries a derivation suffix; give the account xpub alone, as `md decompose --emit keys` prints it"
    );
}

#[test]
fn hash_must_be_exactly_64_lowercase_hex() {
    assert_eq!(
        parse(&format!("hash:{}", "a8".repeat(31))),
        Some(Err(ComposerRecordError::Hash(Some(RecordHashKind::Sha256))))
    );
    assert_eq!(
        parse(&format!("hash:{}", "a8".repeat(33))),
        Some(Err(ComposerRecordError::Hash(Some(RecordHashKind::Sha256))))
    );
    assert_eq!(
        parse(&format!("hash:{}", "A8".repeat(32))),
        Some(Err(ComposerRecordError::Hash(Some(RecordHashKind::Sha256))))
    );
    assert_eq!(
        parse("hash:"),
        Some(Err(ComposerRecordError::Hash(Some(RecordHashKind::Sha256))))
    );
    assert_eq!(
        parse(&format!("hash:{}g", "a8".repeat(31))),
        Some(Err(ComposerRecordError::Hash(Some(RecordHashKind::Sha256))))
    );
    // The refusal now names the KIND'S OWN width. It said "must be exactly 64
    // hex characters" under every kind, so a `ripemd160` record given 64 hex
    // was told its 64-hex value should be 64 hex (SPEC_hashlock_kinds §6).
    assert_eq!(
        ComposerRecordError::Hash(Some(RecordHashKind::Sha256)).line(0),
        "record 0: hash: sha256 needs exactly 64 lowercase hex characters"
    );
    assert_eq!(
        ComposerRecordError::Hash(Some(RecordHashKind::Ripemd160)).line(0),
        "record 0: hash: ripemd160 needs exactly 40 lowercase hex characters"
    );
    // An unrecognised token is its own refusal: it must NOT fall back to
    // sha256's width, because falling back at all is how `hash:sha512:<hex>`
    // could be read AS sha256 (§6's fail-closed paragraph).
    assert!(
        ComposerRecordError::Hash(None)
            .line(0)
            .contains("unknown hash kind"),
        "{}",
        ComposerRecordError::Hash(None).line(0)
    );
}

#[test]
fn now_must_be_seconds_and_optional_height_in_range() {
    let bad = |text: &str| parse(&format!("{NOW_PREFIX}{}", hex(text.as_bytes())));
    for text in [
        "0",
        "2147483648",
        "12345678901",
        "abc",
        "",
        ",",
        "1756684800,",
        ",910000",
        "1756684800,0",
        "1756684800,500000000",
        "1756684800,1000000000",
        "1756684800,910000,1",
        " 1756684800",
        "1756684800 ",
        "+1756684800",
        "1756684800.0",
    ] {
        assert_eq!(bad(text), Some(Err(ComposerRecordError::Now)), "{text:?}");
    }
    for (text, want) in [
        ("1", (1, None)),
        ("2147483647", (2_147_483_647, None)),
        ("1756684800,1", (1_756_684_800, Some(1))),
        ("1756684800,499999999", (1_756_684_800, Some(499_999_999))),
    ] {
        assert_eq!(
            bad(text),
            Some(Ok(ComposerRecord::Now {
                seconds: want.0,
                height: want.1
            })),
            "{text}"
        );
    }
    assert!(matches!(
        parse("now:zz"),
        Some(Err(ComposerRecordError::Now))
    ));
    assert!(
        matches!(parse("now:ff"), Some(Err(ComposerRecordError::Now))),
        "not UTF-8"
    );
    assert_eq!(
        ComposerRecordError::Now.line(2),
        "record 2: now: must be <seconds>[,<height>] in range"
    );
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

// ---- the classifier, admission and the single-now rule ----------------------------------

use mnemonic_engrave::sysw::record::Class;
use mnemonic_engrave::sysw::{classify, pack_deterministic, SyswError, UnknownReason};

const ITER: u32 = 1;
const SALT: [u8; mnemonic_engrave::sysw::wire::SALT_LEN] =
    [7; mnemonic_engrave::sysw::wire::SALT_LEN];
const IV: [u8; mnemonic_engrave::sysw::wire::IV_LEN] = [9; mnemonic_engrave::sysw::wire::IV_LEN];

fn pack(records: Vec<String>) -> Result<Vec<u8>, SyswError> {
    pack_deterministic(records, None, ITER, SALT, IV)
}

#[test]
fn the_three_classes_classify_before_the_sniffers_and_are_not_secret() {
    assert_eq!(classify(&key_record(KEY0)), Class::Key);
    assert_eq!(classify(&hash_record(&hl())), Class::Hash);
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
            assert!(e
                .line(1)
                .starts_with("record 1: key: needs [fingerprint/path]xpub"));
        }
        other => panic!("{other:?}"),
    }
    // A 63-character hash.
    let short = format!("hash:{}", "a8".repeat(31))
        .trim_end_matches('8')
        .to_string();
    assert_eq!(classify(&short), Class::Unknown);
    assert_eq!(
        pack(vec![short]),
        Err(SyswError::Unclassifiable(
            0,
            UnknownReason::Composer(ComposerRecordError::Hash(Some(RecordHashKind::Sha256)))
        ))
    );
    // now: out of range.
    let zero = now_record(0, None);
    assert_eq!(classify(&zero), Class::Unknown);
    assert_eq!(
        pack(vec![zero]),
        Err(SyswError::Unclassifiable(
            0,
            UnknownReason::Composer(ComposerRecordError::Now)
        ))
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
    let recs = vec![
        key_record(KEY0),
        hash_record(&hl()),
        now_record(1_756_684_800, Some(910_000)),
    ];
    let blob = pack(recs.clone()).unwrap();
    let opened = mnemonic_engrave::sysw::open(&blob, None).unwrap();
    assert_eq!(opened.public, recs);
    assert!(opened.secret.is_empty());
}

// ---- the lockstep fixture (spec §12 item 8) ---------------------------------------------

use mnemonic_engrave::sysw::composer_records::{fixture_rows, FixtureRow, CASES};
use sha2::Digest as _;

/// Pinned IDENTICALLY in the fork's `sysw/composer_records_conformance_test.go`
/// (Stage 2). Changing a row means changing this in both repos — the point.
/// Measured 2026-09-02 by running the regenerate test over CASES in the plan's
/// build-gate scratch copy; the regenerate test prints it again on every run.
const FIXTURE_SHA256: &str = "c5f721455b75513b4217c200cf49b4b0eed471fbfcb704121c3cebbd09066770";
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
            Some(Err(e)) => assert_eq!(
                Some(e.line(0)),
                c.host_line.map(str::to_string),
                "{}",
                c.name
            ),
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
            *lines
                .entry(l.trim_start_matches("record 0: ").to_string())
                .or_default() += 1;
        }
    }
    for cls in ["Key", "Hash", "Now", "Unknown"] {
        assert!(
            classes.get(cls).copied().unwrap_or(0) >= 2,
            "class {cls}: {classes:?}"
        );
    }
    for l in [
        "key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record",
        // One line PER KIND now (§6): the refusal names the kind's own width,
        // so a `ripemd160` record given 64 hex is no longer told its 64-hex
        // value should be 64 hex.
        "hash: sha256 needs exactly 64 lowercase hex characters",
        "hash: ripemd160 needs exactly 40 lowercase hex characters",
        // ...and an unrecognised token is its OWN refusal rather than a
        // fallback to sha256, which is what §6's fail-closed paragraph rules
        // out: reading `hash:sha512:<hex>` AS sha256 composes a wallet nobody
        // can spend, silently.
        "hash: unknown hash kind; expected `hash:<hex>` (sha256) or `hash:<kind>:<hex>` \
         with kind hash256, ripemd160 or hash160, lowercase",
        "now: must be <seconds>[,<height>] in range",
    ] {
        assert!(
            lines.get(l).copied().unwrap_or(0) >= 2,
            "line {l:?}: {lines:?}"
        );
    }
    // §12 item 8: "each §6a malformation" — one named row per rule, so a Go port
    // that ignores depth, hard-codes depth 4, reads the fingerprint
    // case-insensitively (Go's hex.DecodeString does) or rejects tpub fails here.
    let names: std::collections::BTreeSet<&str> = CASES.iter().map(|c| c.name).collect();
    for required in [
        "key-journey-cosigner-0",
        "key-h-spelling",
        "key-depth-3-valid",
        "key-testnet-tpub-refused-s1",
        "key-testnet-tpub-refused",
        "key-bare-xpub",
        "key-origin-no-path",
        "key-origin-shorter-than-depth",
        "key-origin-longer-than-depth",
        "key-last-component-mismatch",
        "key-depth-2-refused",
        "key-depth-5-refused",
        "key-fingerprint-uppercase",
        "key-fingerprint-7-hex",
        "key-origin-unterminated",
        "key-body-not-hex",
        "key-body-uppercase-hex",
        "key-body-not-utf8",
        "key-body-empty",
        "key-uppercase-H-marker-out-of-scope",
        "hash-valid",
        "hash-valid-zeros",
        "hash-31-bytes",
        "hash-63-chars",
        "hash-66-chars",
        "hash-uppercase",
        "hash-empty",
        "now-seconds-only",
        "now-seconds-and-height",
        "now-min",
        "now-max-both",
        "now-zero-seconds",
        "now-seconds-2^31",
        "now-height-zero",
        "now-height-at-time-threshold",
        "now-trailing-comma",
        "now-letters",
        "now-body-not-hex",
        "now-body-not-utf8",
        "now-body-uppercase-hex",
        "now-empty",
    ] {
        assert!(
            names.contains(required),
            "§6a rule without a fixture row: {required}"
        );
    }
}

#[test]
fn the_committed_fixture_is_what_the_table_generates_and_carries_the_pinned_digest() {
    let bytes = std::fs::read(fixture_path())
        .expect("testdata/record_class_vectors.json exists; run the regenerate test");
    let digest = hex(&sha2::Sha256::digest(&bytes));
    assert_eq!(
        digest, FIXTURE_SHA256,
        "the fixture changed: re-pin here AND in the fork"
    );
    let on_disk: Vec<FixtureRow> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        on_disk,
        fixture_rows(),
        "the committed file is not what CASES generates; regenerate"
    );
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

/// F-493: `phrase_record` is the one constructor in this family that carries a
/// SECRET, and its type says so.
///
/// The binding below is the guard, and it is a compile-time one on purpose: if
/// the return type goes back to `String` this file stops building, which is a
/// louder failure than an assertion and cannot be satisfied by accident. Its
/// three siblings (`key_record`, `hash_record`, `now_record`) build public data
/// and keep their plain `String` — the point is the asymmetry, not a house
/// style.
///
/// The capacity assertion pins that the buffer is sized up front and never
/// grows: `Zeroizing` protects the buffer it holds, so a reallocation part-way
/// through would leave the abandoned one unwiped and the wrapper none the wiser.
///
/// WHAT IT DOES NOT PIN, measured rather than assumed: wrapping a finished
/// `format!` instead of building into the buffer PASSES this test — `format!`
/// happened to return a `String` already sized to its content, so capacity
/// equals length either way. That version is worse (the phrase exists in an
/// unprotected `String` before the wrap), and nothing here can tell. The
/// no-intermediate property is argued from the four lines of the function, not
/// tested, and this paragraph is here so nobody later reads the assertion as
/// covering it.
///
/// MUTATION that DOES red it: reserve a capacity larger than the body (e.g.
/// `+ 8`) -> `the buffer grew past what was reserved` fires on the mismatch.
#[test]
fn a_phrase_record_is_wiped_on_drop_and_never_reallocates() {
    let phrase = "correct horse battery staple";
    let rec: zeroize::Zeroizing<String> = phrase_record(HashlockMethod::Hardened, phrase);

    assert_eq!(
        &*rec, "phrase:68617264656e65642c636f727265637420686f727365206261747465727920737461706c65",
        "the record is not the corpus row `phrase-hardened`"
    );
    assert_eq!(
        rec.capacity(),
        rec.len(),
        "the buffer grew past what was reserved, so an unwiped copy was abandoned on the way"
    );

    // The property has to hold at the awkward widths too, not just this one.
    for p in ["", "x", &"0".repeat(100), "a phrase, with a comma"] {
        for m in [HashlockMethod::Hardened, HashlockMethod::Sha256] {
            let r = phrase_record(m, p);
            assert_eq!(
                r.capacity(),
                r.len(),
                "method {m:?}, phrase of {} chars",
                p.len()
            );
        }
    }
}

// ─── SPEC_hashlock_kinds §6: `hash: [<kind>:] <hex>` ────────────────────────

/// §6's producer rule: **bare for sha256, explicit for the other three.** Every
/// payload that exists today stays byte-identical, which is the whole reason
/// the bare form was kept rather than tagging all four.
#[test]
fn the_producer_rule_is_bare_for_sha256_and_tagged_for_the_rest() {
    let d32 = [0xa8u8; 32];
    let d20 = [0x5cu8; 20];

    assert_eq!(
        hash_record(&HashLock::new(RecordHashKind::Sha256, &d32[..]).unwrap()),
        format!("hash:{}", hex(&d32)),
        "sha256 must stay BARE -- a tagged sha256 record would change every \
         payload that exists"
    );
    for (kind, body) in [
        (RecordHashKind::Hash256, &d32[..]),
        (RecordHashKind::Ripemd160, &d20[..]),
        (RecordHashKind::Hash160, &d20[..]),
    ] {
        assert_eq!(
            hash_record(&HashLock::new(kind, body).unwrap()),
            format!("hash:{}:{}", kind.token(), hex(body)),
        );
    }
}

/// §6: "Input is liberal, output is conservative." `hash:sha256:<64hex>` is
/// ACCEPTED and never emitted.
#[test]
fn an_explicit_sha256_record_is_accepted_and_never_emitted() {
    let d32 = [0xa8u8; 32];
    let explicit = format!("hash:sha256:{}", hex(&d32));
    let parsed = parse(&explicit).expect("recognised").expect("admitted");
    let ComposerRecord::Hash(hl) = parsed else {
        panic!("not a hash record: {parsed:?}")
    };
    assert_eq!(hl.kind(), RecordHashKind::Sha256);
    assert_eq!(hl.digest(), &d32[..]);
    assert_eq!(
        hash_record(&hl),
        format!("hash:{}", hex(&d32)),
        "accepted on input, but the emitted form is the bare one"
    );
}

/// §6: the hex body is `digest_len() * 2`, so a 20-byte kind takes 40 and a
/// 64-hex `ripemd160` is a refusal rather than a truncation.
#[test]
fn each_kind_takes_its_own_hex_width() {
    let h64 = hex(&[0xa8u8; 32]);
    let h40 = hex(&[0x5cu8; 20]);
    for (kind, ok_hex, wrong_hex) in [
        ("sha256", &h64, &h40),
        ("hash256", &h64, &h40),
        ("ripemd160", &h40, &h64),
        ("hash160", &h40, &h64),
    ] {
        let good = if kind == "sha256" {
            format!("hash:{ok_hex}")
        } else {
            format!("hash:{kind}:{ok_hex}")
        };
        assert!(
            matches!(parse(&good), Some(Ok(ComposerRecord::Hash(_)))),
            "{kind}: its own width must be admitted: {good}"
        );
        let bad = format!("hash:{kind}:{wrong_hex}");
        assert!(
            matches!(parse(&bad), Some(Err(ComposerRecordError::Hash(_)))),
            "{kind}: the other width must be REFUSED, not truncated: {bad}"
        );
    }
}

/// §6: "Case is rejected, never folded." Stated as a rule rather than left to
/// "obviously lowercase", which is how two parsers come to disagree.
#[test]
fn an_uppercase_kind_token_is_refused_not_folded() {
    let h40 = hex(&[0x5cu8; 20]);
    for tok in ["RIPEMD160", "Ripemd160", "riPEMD160"] {
        let r = format!("hash:{tok}:{h40}");
        assert!(
            matches!(parse(&r), Some(Err(ComposerRecordError::Hash(_)))),
            "{tok} must be refused, not folded: {r}"
        );
    }
}

/// An unknown token is refused as a hash record, NOT silently read as a bare
/// digest. Reading `hash:sha512:<hex>` as sha256 is the failure mode §6's
/// fail-closed paragraph exists to rule out.
#[test]
fn an_unknown_kind_token_is_refused() {
    let h64 = hex(&[0xa8u8; 32]);
    assert!(matches!(
        parse(&format!("hash:sha512:{h64}")),
        Some(Err(ComposerRecordError::Hash(_)))
    ));
}

/// `HashLock::new` REFUSES a digest that is not its kind's width, and that
/// guard is tested directly rather than through `parse`.
///
/// It was redundant with `parse_hash` and deletable with every test still green
/// (R0 round 1, M11) — but it is a `pub` constructor, so the Go port will
/// reimplement it, and a guard no test exercises is a guard the port can drop
/// without anything noticing. A 20-byte digest inside a `Sha256` lock would
/// reach `hash_record` and emit a 40-hex record claiming to be sha256.
#[test]
fn hash_lock_new_refuses_a_digest_of_the_wrong_width() {
    let d32 = [0xa8u8; 32];
    let d20 = [0x5cu8; 20];
    for (kind, right, wrong) in [
        (RecordHashKind::Sha256, &d32[..], &d20[..]),
        (RecordHashKind::Hash256, &d32[..], &d20[..]),
        (RecordHashKind::Ripemd160, &d20[..], &d32[..]),
        (RecordHashKind::Hash160, &d20[..], &d32[..]),
    ] {
        assert!(
            HashLock::new(kind, right).is_some(),
            "{kind:?}: its own width must be admitted"
        );
        assert!(
            HashLock::new(kind, wrong).is_none(),
            "{kind:?}: the other width must be refused, not reshaped"
        );
        // ...and nothing in between, either.
        assert!(HashLock::new(kind, &[]).is_none(), "{kind:?}: empty");
        assert!(
            HashLock::new(kind, &d32[..31]).is_none() || kind.digest_len() == 31,
            "{kind:?}: 31 bytes"
        );
    }
}
