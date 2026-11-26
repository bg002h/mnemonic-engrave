//! A non-UTF-8 argument is a usage refusal (exit 64), not a panic (exit 101),
//! and the refusal never echoes the argument (exit-code survey §8, 2026-11-26).
#![cfg(unix)]

use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn me() -> PathBuf {
    assert_cmd::cargo::cargo_bin("me")
}

#[test]
fn non_utf8_argument_is_usage_not_panic_and_not_echoed() {
    let d = tempfile::tempdir().unwrap();
    // Bytes \xff\xfe plus a recognisable marker, passed as one argv token.
    let mut tok = b"SECRETMARK".to_vec();
    tok.extend_from_slice(&[0xff, 0xfe]);
    for args in [
        vec![b"--hex".to_vec(), tok.clone()],
        vec![b"--in".to_vec(), tok.clone()],
        vec![b"bundle".to_vec(), tok.clone()],
    ] {
        let o = Command::new(me())
            .args(args.iter().map(|a| std::ffi::OsStr::from_bytes(a)))
            .current_dir(d.path())
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&o.stderr).into_owned();
        assert_eq!(o.status.code(), Some(64), "stderr:\n{err}");
        assert!(!err.contains("panicked"), "{err}");
        assert!(!err.contains("SECRETMARK"), "argument echoed: {err}");
        assert!(!o.stderr.windows(2).any(|w| w == [0xff, 0xfe]), "{err}");
        assert!(o.stdout.is_empty());
    }
}

#[test]
fn argv_secret_guard_still_runs_before_the_utf8_refusal() {
    // An ms1 token on argv is refused by the guard (exit 8) even when another
    // argument is not UTF-8.
    let d = tempfile::tempdir().unwrap();
    let ms1 = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let o = Command::new(me())
        .arg(ms1)
        .arg(std::ffi::OsStr::from_bytes(&[0xff, 0xfe]))
        .current_dir(d.path())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr).into_owned();
    assert!(!err.contains("panicked"), "{err}");
    assert_eq!(o.status.code(), Some(8), "stderr:\n{err}");
    // The GUARD's refusal, not some other 8 (review M6), and it echoes
    // neither the secret nor the raw bytes.
    assert!(
        err.contains("Refused BEFORE the command line was parsed"),
        "{err}"
    );
    assert!(!err.contains(ms1), "the secret reached stderr:\n{err}");
    assert!(
        !o.stderr.windows(2).any(|w| w == [0xff, 0xfe]),
        "the raw bytes reached stderr"
    );
}
