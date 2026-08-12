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

/// The plan's stage-2 green line said `me sysw pack … | wc -c` is 65536. It was
/// not: `pack` emitted the container and only `wipe` emitted a region, so the
/// one artifact you can actually write to `0x10D00000` had no command. `--region`
/// is that command.
#[test]
fn region_pads_the_container_to_a_flashable_image() {
    let out = me()
        .args(["sysw", "pack", "--no-passphrase", "--region", TEXT])
        .assert()
        .success();
    let o = &out.get_output().stdout;
    assert_eq!(o.len(), 65_536, "--region emits exactly one region");
    assert_eq!(&o[..8], b"MNEMSYSW", "the container is at offset 0");
    assert!(
        o[600..].iter().all(|&b| b == 0xFF),
        "the tail is 0xFF — the ERASED state, so the image is what the sector \
         looks like with only the container written. Zeros would be a WRITE, \
         and would wear the flash for nothing."
    );
}

/// The padding must not change what the operator compares on screen, or the
/// region image and the container would be two different payloads.
#[test]
fn region_and_container_have_the_same_digest_and_identity() {
    let dir = tempfile::tempdir().unwrap();
    let (c, r) = (dir.path().join("c.bin"), dir.path().join("r.bin"));
    for (f, extra) in [(&c, None), (&r, Some("--region"))] {
        let mut cmd = me();
        cmd.args(["sysw", "pack", "--no-passphrase"]);
        if let Some(e) = extra {
            cmd.arg(e);
        }
        cmd.arg(TEXT).arg("--out").arg(f).assert().success();
    }
    let show = |f: &std::path::Path| {
        let a = me().args(["sysw", "show"]).arg(f).assert().success();
        let o = a.get_output();
        (
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
        )
    };
    let (co, ce) = show(&c);
    let (ro, re) = show(&r);
    assert_eq!(
        digest_line(&ce),
        digest_line(&re),
        "padding must not move the digest"
    );
    let id = |s: &str| {
        s.lines()
            .find_map(|l| l.strip_prefix("identity: "))
            .map(str::to_string)
    };
    assert_eq!(id(&co), id(&ro), "nor the identity");
    assert!(id(&co).is_some(), "and one was actually printed");
}

/// `--region` describes where the bytes go, not what is in them; refusing to
/// combine it with a sealed payload would make the flashable form the ONE form
/// a secret cannot take.
#[test]
fn region_works_for_a_sealed_payload_too() {
    let out = me()
        .args([
            "sysw",
            "pack",
            "--region",
            "--passphrase-words",
            "12",
            SEED,
        ])
        .assert()
        .success();
    assert_eq!(out.get_output().stdout.len(), 65_536);
}

/// Pre-flash fable review, C1. `pack` enforced neither bound its own parser
/// enforces, so it emitted containers `show` refuses — exit 0 on write, exit 4
/// on read. With `--region` that becomes a flash-ready image of an unreadable
/// payload, and for a SEALED one that is a seed backup nobody can ever open.
///
/// The rule these pin: **the writer must refuse everything the reader refuses.**
/// A writer looser than its reader is how permanent media gets a payload that
/// was never openable.
#[test]
fn pack_refuses_iterations_its_own_parser_would_reject() {
    for n in ["5", "99999", "2000001"] {
        me().args([
            "sysw",
            "pack",
            "--iterations",
            n,
            "--passphrase-words",
            "12",
            SEED,
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("iterations"));
    }
}

#[test]
fn pack_accepts_the_iteration_bounds_themselves() {
    for n in ["100000", "2000000"] {
        me().args([
            "sysw",
            "pack",
            "--iterations",
            n,
            "--passphrase-words",
            "12",
            SEED,
        ])
        .assert()
        .success();
    }
}

#[test]
fn pack_refuses_a_section_too_long_for_its_own_parser() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("big.txt");
    let recs: Vec<String> = (0..30).map(|_| format!("text:{}", "61".repeat(400))).collect();
    std::fs::write(&f, recs.join("\n")).unwrap();
    me().args(["sysw", "pack", "--no-passphrase", "--in"])
        .arg(&f)
        .assert()
        .failure()
        .stderr(predicate::str::contains("too long"));
}

/// The property behind both, stated once: anything `pack` writes, `show` reads.
#[test]
fn everything_pack_emits_is_readable_by_show() {
    let dir = tempfile::tempdir().unwrap();
    for (i, args) in [
        vec!["--no-passphrase", TEXT],
        vec!["--no-passphrase", MD1],
        vec!["--passphrase-words", "12", SEED],
        vec!["--iterations", "100000", "--passphrase-words", "2", SEED],
    ]
    .iter()
    .enumerate()
    {
        let f = dir.path().join(format!("p{i}.bin"));
        let mut c = me();
        c.args(["sysw", "pack"]);
        c.args(args.iter());
        c.arg("--out").arg(&f).assert().success();
        me().args(["sysw", "show"])
            .arg(&f)
            .assert()
            .success()
            .stdout(predicate::str::contains("identity:"));
    }
}

/// Pre-flash fable review, I2. `show` printed a plausible identity and THEN
/// panicked (exit 101) on a container whose header declares more than the file
/// holds. A panic after a plausible-looking line is the worst shape: the
/// operator has already read a number that means nothing.
#[test]
fn show_on_a_truncated_container_fails_cleanly_without_printing_a_digest() {
    let dir = tempfile::tempdir().unwrap();
    let (full, trunc) = (dir.path().join("f.bin"), dir.path().join("t.bin"));
    me().args(["sysw", "pack", "--no-passphrase", TEXT, "--out"])
        .arg(&full)
        .assert()
        .success();
    let all = std::fs::read(&full).unwrap();
    std::fs::write(&trunc, &all[..60.min(all.len())]).unwrap();
    let out = me().args(["sysw", "show"]).arg(&trunc).assert().failure();
    let o = out.get_output();
    assert_ne!(o.status.code(), Some(101), "must not panic");
    assert!(
        digest_line(&String::from_utf8_lossy(&o.stderr)).is_none(),
        "and must not print a digest it cannot compute"
    );
}

#[test]
fn an_unplaceable_record_is_named_with_its_index() {
    me().args(["sysw", "pack", "--no-passphrase", MD1, "not a record"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("record 1"));
}
