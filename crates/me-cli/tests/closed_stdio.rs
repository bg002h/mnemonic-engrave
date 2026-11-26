//! A closed stdout is a failed write, not success (exit-code survey §8,
//! 2026-11-26).
//!
//! std's runtime puts a read-write `/dev/null` on a closed fd 1 before `main`,
//! so the output vanished at exit 0. `me` detects that at startup and reports
//! the write the way it reports a read-only stdout (EBADF, exit 32: IO).
#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const MD1: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
const EBADF: &str = "me: cannot write the output: Bad file descriptor (os error 9)";

fn me() -> PathBuf {
    assert_cmd::cargo::cargo_bin("me")
}

struct Run {
    rc: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Run {
    fn err(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
    fn dump(&self) -> String {
        format!("rc {:?}, stderr:\n{}", self.rc, self.err())
    }
}

/// Run `sh -c <script>` in `dir`, `$ME` the binary, stdin `/dev/null`.
fn sh(dir: &Path, script: &str) -> Run {
    let o = Command::new("sh")
        .arg("-c")
        .arg(script)
        .current_dir(dir)
        .env("ME", me())
        .env_remove("ME_PREVIEW_BIN")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    Run {
        rc: o.status.code(),
        stdout: o.stdout,
        stderr: o.stderr,
    }
}

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("m.txt"), format!("{MD1}\n")).unwrap();
    dir
}

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn closed_stdout_hex_exits_32() {
    let d = fixture();
    let r = sh(d.path(), "exec \"$ME\" --in m.txt --hex >&-");
    assert_eq!(r.rc, Some(32), "{}", r.dump());
    assert_eq!(r.err(), format!("{EBADF}\n"), "{}", r.dump());
}

#[test]
fn closed_stdout_help_and_version_exit_32() {
    let d = fixture();
    for flag in ["--help", "--version", "bundle --help"] {
        let r = sh(d.path(), &format!("exec \"$ME\" {flag} >&-"));
        assert_eq!(r.rc, Some(32), "{flag}: {}", r.dump());
        assert_eq!(r.err(), format!("{EBADF}\n"), "{flag}: {}", r.dump());
    }
}

#[test]
fn closed_stdout_with_out_file_succeeds_and_nothing_goes_astray() {
    let d = fixture();
    let want = sh(d.path(), "exec \"$ME\" --in m.txt --out ref.ndef");
    assert_eq!(want.rc, Some(0), "{}", want.dump());
    let r = sh(d.path(), "exec \"$ME\" --in m.txt --out o.ndef >&-");
    assert_eq!(r.rc, Some(0), "{}", r.dump());
    let a = fs::read(d.path().join("o.ndef")).unwrap();
    let b = fs::read(d.path().join("ref.ndef")).unwrap();
    assert_eq!(a, b, "the --out file holds exactly the output");
    assert_eq!(names(d.path()), ["m.txt", "o.ndef", "ref.ndef"]);
}

#[test]
fn closed_stdout_out_dev_stdout_fails() {
    let d = fixture();
    let r = sh(d.path(), "exec \"$ME\" --in m.txt --out /dev/stdout >&-");
    assert_ne!(r.rc, Some(0), "{}", r.dump());
    assert!(!r.err().contains("panicked"), "{}", r.dump());
}

#[test]
fn closed_stdin_and_stderr_do_not_panic() {
    let d = fixture();
    // Closed stdin: nothing to read is empty input, INVALID (2).
    let r = sh(d.path(), "exec \"$ME\" --hex <&-");
    assert_eq!(r.rc, Some(2), "{}", r.dump());
    // Closed stderr: the conversion still succeeds on stdout.
    let r = sh(d.path(), "exec \"$ME\" --in m.txt --hex 2>&-");
    assert_eq!(r.rc, Some(0), "{}", r.dump());
    assert!(!r.stdout.is_empty());
}

#[test]
fn stdout_to_dev_null_still_succeeds() {
    // A shell's `>/dev/null` (O_WRONLY) is a real destination, not a closed fd.
    let d = fixture();
    let r = sh(d.path(), "exec \"$ME\" --in m.txt --hex >/dev/null");
    assert_eq!(r.rc, Some(0), "{}", r.dump());
    assert!(r.stderr.is_empty(), "{}", r.dump());
}
