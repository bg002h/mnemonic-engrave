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
    // A BARE `hash:` body is the sha256 case (§6), so the refusal names
    // sha256's width -- it said "must be exactly 64 hex characters" under every
    // kind before, which told a `ripemd160` record given 64 hex that its 64-hex
    // value should be 64 hex.
    assert!(String::from_utf8_lossy(&o.stderr)
        .contains("record 0: hash: sha256 needs exactly 64 lowercase hex characters"));
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

/// The advice printed beside a rejected `hash:` record must not tell the
/// operator to do the one thing that loses funds.
///
/// **This was a live hazard created by SPEC_hashlock_kinds** (phase 1 journey
/// walk, F-C1). `me sysw pack` refuses `hash:hash256:<64 hex>` with *"hash: must
/// be exactly 64 hex characters"*, and the build advice said a hash record is
/// *"`hash:` + the 32-byte digest as 64 lowercase hex"* — follow that literally
/// and you delete the `hash256:` tag. The untagged record is then ACCEPTED, as
/// **sha256**, committing the payload to a different digest than the wallet.
///
/// `ms hashlock`'s card already warns against exactly this and quotes this
/// error string, so before the fix the two tools contradicted each other and
/// the dangerous one was the one on screen at the moment the operator was stuck.
#[test]
fn the_rejected_hash_record_advice_does_not_tell_you_to_strip_the_kind_tag() {
    // PHASE 3 CLOSED THIS HAZARD AT ITS ROOT: `hash:hash256:<64 hex>` is now
    // ACCEPTED (SPEC_hashlock_kinds §6), so it no longer reaches this advice at
    // all. The advice still matters for a record that IS malformed, and it must
    // still not push the operator toward deleting a tag — so the trigger is a
    // wrong-width tagged record instead.
    let out = me()
        .args([
            "sysw",
            "pack",
            // ripemd160 given sha256's width
            "hash:ripemd160:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488",
        ])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "a wrong-width tagged record is still refused"
    );
    let se = String::from_utf8_lossy(&out.stderr);

    assert!(
        se.contains("DO NOT DELETE IT"),
        "the advice must warn against stripping the tag, because the refusal \
         above it reads as an instruction to:\n{se}"
    );
    assert!(
        se.contains("hash:<kind>:"),
        "the advice must name the tagged form as legitimate, or the operator \
         concludes their record is malformed:\n{se}"
    );
    assert!(
        se.contains("read as sha256"),
        "the advice must say WHAT stripping it does; 'do not' without a reason \
         loses to a refusal that looks like an instruction:\n{se}"
    );
}

/// F-549: the "stripping succeeds silently" clause belongs to hash256 alone.
///
/// It was stated of every tagged record and is FALSE for the 20-byte kinds:
/// stripping `ripemd160:` leaves 40 hex, which this very tool refuses. Only
/// hash256 shares sha256's width and so strips to a legal record committing the
/// payload to a different digest.
///
/// Over-warning is not harmless — it teaches a false rule about the 40-hex
/// kinds and undermines a correct refusal the operator meets if they try it.
///
/// MUTATION: make the clause unconditional again -> the ripemd160 row fails.
#[test]
fn the_silent_strip_warning_is_scoped_to_the_kind_it_is_true_of() {
    let dir = tempfile::tempdir().unwrap();
    for (record, silent) in [
        (
            "hash:hash256:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488",
            true,
        ),
        (
            "hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b",
            false,
        ),
    ] {
        let (_, o) = pack_to(&dir, &[], &[record]);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let err = String::from_utf8_lossy(&o.stderr).to_string();
        assert_eq!(
            err.contains("stripping THAT tag succeeds"),
            silent,
            "{record}: the silent-strip clause fired={}, want {silent}\n{err}",
            !silent
        );
        if !silent {
            assert!(
                err.contains("which this tool refuses"),
                "a 40-hex kind must be told that stripping is REFUSED, not silent:\n{err}"
            );
        }
        // Both keep the part that is true of every tagged record.
        assert!(
            err.contains("Do not strip the tag"),
            "{record}: the instruction itself is gone:\n{err}"
        );
    }
}

/// F-552: `me sysw show` prints the FULL digest of a public hash record.
///
/// It printed only `09e7bb50..bcc2946b`, and `show` is the only payload reader
/// — so an operator whose policy card is the thing they lost could confirm a
/// candidate and still not retype it into `md compose`. The value sat in the
/// container in the clear the whole time, so this publishes nothing new.
///
/// The elided line STAYS: it is what makes a side-by-side comparison with the
/// device's own screen possible.
///
/// MUTATION: delete the full-digest println -> the second assertion fails.
#[test]
fn show_prints_the_full_digest_of_a_public_hash_record() {
    let dir = tempfile::tempdir().unwrap();
    const D: &str = "09e7bb5051d89788fb4e4b374126721dbcc2946b";
    let (path, o) = pack_to(&dir, &[], &[&format!("hash:ripemd160:{D}")]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let s = shown(&path);
    assert!(
        s.contains("09e7bb50..bcc2946b"),
        "the elided form is what matches the device screen; it must stay:\n{s}"
    );
    assert!(
        s.contains(&format!("ripemd160:{D}")),
        "the full digest is not printed, so a lost policy card cannot be retyped:\n{s}"
    );
    // A PHRASE record is SECRET and must still show neither its phrase nor a
    // digest. It rides WITH the public record and through --in: a secret record
    // needs a seal (so no --no-passphrase), and the argv guard refuses a phrase
    // record on the command line -- both of those are the guards working.
    let recs = dir.path().join("recs.txt");
    std::fs::write(
        &recs,
        format!(
            "hash:ripemd160:{D}\nphrase:{}\n",
            hex("hardened,correct horse battery staple")
        ),
    )
    .unwrap();
    let out2 = dir.path().join("phrase.bin");
    let o = me()
        .args([
            "sysw",
            "pack",
            "--in",
            recs.to_str().unwrap(),
            "--out",
            out2.to_str().unwrap(),
            "--passphrase-stdin",
        ])
        .write_stdin("a-test-passphrase\n")
        .output()
        .unwrap();
    if o.status.success() {
        let s = shown(&out2);
        assert!(
            !s.contains("correct horse"),
            "show printed a SECRET phrase record's content:\n{s}"
        );
    }
}
