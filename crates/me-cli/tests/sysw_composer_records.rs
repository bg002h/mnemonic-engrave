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
fn hash_must_be_exactly_64_lowercase_hex() {
    assert_eq!(
        parse(&format!("hash:{}", "a8".repeat(31))),
        Some(Err(ComposerRecordError::Hash))
    );
    assert_eq!(
        parse(&format!("hash:{}", "a8".repeat(33))),
        Some(Err(ComposerRecordError::Hash))
    );
    assert_eq!(
        parse(&format!("hash:{}", "A8".repeat(32))),
        Some(Err(ComposerRecordError::Hash))
    );
    assert_eq!(parse("hash:"), Some(Err(ComposerRecordError::Hash)));
    assert_eq!(
        parse(&format!("hash:{}g", "a8".repeat(31))),
        Some(Err(ComposerRecordError::Hash))
    );
    assert_eq!(
        ComposerRecordError::Hash.line(0),
        "record 0: hash: must be exactly 64 hex characters"
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
