//! Bug-hunt rows (2026-11-25 hunt at 80dfaeb). Each asserts the CORRECT
//! behaviour; a red row is a confirmed bug.
#![cfg(unix)]

use assert_cmd::Command;

fn me() -> Command {
    Command::cargo_bin("me").unwrap()
}

/// The public test-vector ms1 body from `seal/record.rs`, with the separator
/// `1` mistyped as `l`, so `classify` sees no `ms` HRP.
const MANGLED_MS1: &str =
    "msl0entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw";
const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon \
                        abandon abandon abandon about";
const MD1: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

/// Distinctive pieces of the secrets above; none may reach stderr or stdout.
const SECRET_PIECES: &[&str] = &["entrsqqg5y2z9", "q5f042qmrw90mw", "abandon", "msl0"];

fn assert_no_secret(label: &str, out: &std::process::Output) {
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    for p in SECRET_PIECES {
        assert!(
            !all.contains(p),
            "{label}: output echoes secret text {p:?}:\n{all}"
        );
    }
}

/// E1: the "unrecognized HRP" message interpolated everything before the
/// first `1` of the input, so a mistyped ms1 or a BIP-39 mnemonic followed by
/// any `1` was printed to stderr in full. The converter, `me bundle` and
/// `me seal` must refuse without echoing any of it.
#[test]
fn bh1125_e1_unknown_hrp_never_echoes_the_input() {
    let dir = tempfile::tempdir().unwrap();
    let cases: Vec<(&str, String)> = vec![
        ("mangled-ms1", format!("{MANGLED_MS1}\n{MD1}\n")),
        (
            "mangled-ms1-plate-1",
            format!("{MANGLED_MS1} plate 1\n{MD1}\n"),
        ),
        ("mnemonic", format!("{MNEMONIC}\n{MD1}\n")),
        ("seed-note", format!("seed: {MNEMONIC} (12 words)\n{MD1}\n")),
    ];
    for (label, text) in &cases {
        let path = dir.path().join(format!("{label}.txt"));
        std::fs::write(&path, text).unwrap();

        let out = me().arg("--in").arg(&path).arg("--hex").output().unwrap();
        assert!(!out.status.success(), "{label}: converter accepted it");
        assert_no_secret(&format!("me --in ({label})"), &out);

        let out = me().args(["bundle", "--in"]).arg(&path).output().unwrap();
        assert!(!out.status.success(), "{label}: bundle accepted it");
        assert_no_secret(&format!("me bundle ({label})"), &out);
    }

    // `me seal`: a record whose HRP is unknown (a hex private key here, whose
    // first `1` sits after 12 hex digits) is refused without its prefix.
    let out_path = dir.path().join("x.uf2");
    let out = me()
        .args([
            "seal",
            "--plaintext",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "--out",
        ])
        .arg(&out_path)
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("unrecognized HRP"), "{err}");
    assert!(!err.contains("e3b0c442"), "me seal echoed the key: {err}");
}
