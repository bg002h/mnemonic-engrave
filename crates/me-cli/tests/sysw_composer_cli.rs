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

fn pack_to(
    dir: &tempfile::TempDir,
    extra: &[&str],
    records: &[&str],
) -> (std::path::PathBuf, std::process::Output) {
    let out = dir.path().join("payload.bin");
    let mut args: Vec<String> = vec![
        "sysw".into(),
        "pack".into(),
        "--no-passphrase".into(),
        "--out".into(),
        out.display().to_string(),
    ];
    args.extend(extra.iter().map(|s| s.to_string()));
    args.extend(records.iter().map(|s| s.to_string()));
    let o = me().args(&args).output().unwrap();
    (out, o)
}

fn shown(path: &std::path::Path) -> String {
    let o = me()
        .args(["sysw", "show", path.to_str().unwrap()])
        .output()
        .unwrap();
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
fn a_payload_without_a_composer_record_gains_no_pack_time_record() {
    // Seeds, text, cards: NO bound appended, no note; the six pre-existing pack
    // tests stay untouched for exactly this reason.
    let dir = tempfile::tempdir().unwrap();
    for records in [vec![TEXT], vec![SEED]] {
        let (path, o) = pack_to(&dir, &["--allow-argv-secret"], &records);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        assert!(
            !String::from_utf8_lossy(&o.stderr).contains("appended now:"),
            "{records:?}"
        );
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
    assert!(
        s.contains("public record 2: pack time (now:) — 1756684800 (seconds), height 910000"),
        "{s}"
    );
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
    assert!(
        err.contains("record 2: a second now: record; only one is allowed. Remove one."),
        "{err}"
    );
    assert!(
        err.contains("(records count from 0)"),
        "the seam's refusal vocabulary: {err}"
    );
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
        .args([
            "sysw",
            "pack",
            "--allow-argv-secret",
            "--out",
            out.to_str().unwrap(),
            SEED,
            &a,
            &b,
        ])
        .output()
        .unwrap();
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("a second now: record"), "{err}");
    assert!(
        !err.contains("write this down"),
        "the ceremony ran before the refusal: {err}"
    );
}

#[test]
fn malformed_records_are_refused_with_the_8n_lines() {
    let dir = tempfile::tempdir().unwrap();
    let bare = format!("key:{}", hex("xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"));
    let (_, o) = pack_to(&dir, &[], &[TEXT, &bare]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("record 1: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record"));
    let (_, o) = pack_to(&dir, &[], &[&format!("hash:{}", "a8".repeat(31))]);
    assert!(String::from_utf8_lossy(&o.stderr)
        .contains("record 0: hash: must be exactly 64 hex characters"));
    let (_, o) = pack_to(&dir, &[], &[&format!("now:{}", hex("0"))]);
    assert!(String::from_utf8_lossy(&o.stderr)
        .contains("record 0: now: must be <seconds>[,<height>] in range"));
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
    assert!(
        s.contains(&format!(
            "public record 0: cosigner key (key:) — {KEY0_TEXT}"
        )),
        "{s}"
    );
    assert!(
        s.contains("public record 1: sha256 hashlock (hash:) — a8a8a8a8..a8a8a8a8"),
        "{s}"
    );
    assert!(s.contains("public record 2: pack time (now:) — 1756684800 (seconds): a lower bound on the present the device echoes beside a time lock; never a locktime"), "{s}");
}
