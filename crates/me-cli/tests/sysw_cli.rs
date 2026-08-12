//! `me sysw` — the plan's stage-2 green criteria, as tests.

use assert_cmd::Command;
use predicates::prelude::*;

const MD1: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const TEXT: &str = "text:48656c6c6f2c20576f726c6421";

fn me() -> Command {
    Command::cargo_bin("me").unwrap()
}

/// The digest line, located by its LABEL rather than by shape.
///
/// `seal_cli.rs` records why that matters: a token-count heuristic there matched
/// a line of prose instead of the value, and the assertion built on it went
/// vacuous without failing.
fn digest_line(err: &str) -> Option<&str> {
    err.lines().find_map(|l| l.strip_prefix("digest:   "))
}

#[test]
fn wipe_emits_exactly_one_region() {
    for fill in ["random", "zeros", "ones"] {
        let out = me()
            .args(["sysw", "wipe", "--fill", fill])
            .assert()
            .success();
        assert_eq!(
            out.get_output().stdout.len(),
            65_536,
            "--fill {fill} must fill the region"
        );
    }
}

#[test]
fn wipe_rejects_an_unknown_fill_rather_than_defaulting() {
    me().args(["sysw", "wipe", "--fill", "sideways"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown --fill"));
}

/// Ones is the erased state, so the operator is told — it is the one fill whose
/// result is indistinguishable from "never written".
#[test]
fn wipe_with_ones_says_what_ones_means() {
    me().args(["sysw", "wipe", "--fill", "ones"])
        .assert()
        .success()
        .stderr(predicate::str::contains("ERASED state"));
}

/// The blob goes to stdout and the digest to stderr, so `me sysw pack > f.bin`
/// still shows the operator the number they must compare on the machine.
#[test]
fn pack_separates_the_blob_from_the_digest() {
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .assert()
        .success();
    let o = out.get_output();
    assert!(!o.stdout.is_empty(), "the blob goes to stdout");
    assert_eq!(
        &o.stdout[..8],
        b"MNEMSYSW",
        "and it is a systemwide container"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        digest_line(&err).is_some(),
        "the digest goes to stderr:\n{err}"
    );
}

/// The comparison loop: what `pack` prints must be what `show` prints, or the
/// operator has nothing to compare against.
#[test]
fn show_prints_the_same_digest_pack_did() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("p.bin");
    let packed = me()
        .args(["sysw", "pack", "--no-passphrase", TEXT, "--out"])
        .arg(&f)
        .assert()
        .success();
    let perr = String::from_utf8_lossy(&packed.get_output().stderr).into_owned();
    let shown = me().args(["sysw", "show"]).arg(&f).assert().success();
    let serr = String::from_utf8_lossy(&shown.get_output().stderr).into_owned();
    let a = digest_line(&perr).expect("pack prints a digest");
    let b = digest_line(&serr).expect("show prints a digest");
    assert_eq!(
        a, b,
        "pack and show must agree, or the comparison is worthless"
    );
    assert_eq!(a.split(' ').count(), 8, "grouped in fours for a human");
}

/// Spec §13 D3: `me` WARNS and proceeds. It once refused; a test asserting a
/// non-zero exit here would be re-imposing the demoted rule.
#[test]
fn a_weak_passphrase_over_a_secret_warns_and_still_succeeds() {
    me().args(["sysw", "pack", "--passphrase-words", "2", SEED])
        .assert()
        .success()
        .stderr(predicate::str::contains("WARNING"))
        .stderr(predicate::str::contains("BELOW the threshold"));
}

#[test]
fn allow_weak_is_accepted_and_says_it_is_ignored() {
    me().args(["sysw", "pack", "--allow-weak", "--no-passphrase", TEXT])
        .assert()
        .success()
        .stderr(predicate::str::contains("accepted and ignored"));
}

/// The default is to GENERATE, not to leave a payload unprotected by omission.
#[test]
fn omitting_every_passphrase_flag_generates_one() {
    me().args(["sysw", "pack", MD1])
        .assert()
        .success()
        .stderr(predicate::str::contains("write this down"))
        .stderr(predicate::str::contains("12 words"));
}

#[test]
fn the_passphrase_modes_are_mutually_exclusive() {
    me().args([
        "sysw",
        "pack",
        "--no-passphrase",
        "--passphrase-words",
        "8",
        MD1,
    ])
    .assert()
    .failure();
    me().args(["sysw", "pack", "--passphrase-ask", "--no-passphrase", MD1])
        .assert()
        .failure();
}

/// Spec §5.2: structural failures must NEVER say "payload unreadable" — that
/// phrase teaches the operator to read a wrong file as tampering.
#[test]
fn a_wrong_file_is_not_reported_as_tampering() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("junk.bin");
    std::fs::write(&f, b"MNEMBLOB....................").unwrap();
    me().args(["sysw", "show"])
        .arg(&f)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a systemwide container"))
        .stderr(predicate::str::contains("payload unreadable").not());
}

/// A secrets-only payload has no public section, so there is no digest to
/// compare — and `show` says so rather than printing a constant every such
/// payload shares (R0-C2).
#[test]
fn a_secrets_only_payload_reports_no_digest() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("s.bin");
    me().args(["sysw", "pack", "--passphrase-words", "12", SEED, "--out"])
        .arg(&f)
        .assert()
        .success();
    me().args(["sysw", "show"])
        .arg(&f)
        .assert()
        .success()
        .stdout(predicate::str::contains("pub_len:  0"))
        .stderr(predicate::str::contains("no public section"));
}

#[test]
fn records_can_come_from_a_file_instead_of_argv() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("recs.txt");
    std::fs::write(&f, format!("{MD1}\n{TEXT}\n")).unwrap();
    me().args(["sysw", "pack", "--no-passphrase", "--in"])
        .arg(&f)
        .assert()
        .success();
}

#[test]
fn an_unplaceable_record_is_named_with_its_index() {
    me().args(["sysw", "pack", "--no-passphrase", MD1, "not a record"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("record 1"));
}
