//! **F-265 — five of `me`'s refusals could swap exit 2 for exit 3 with all 388
//! tests green.**
//!
//! Proven against the UNMODIFIED baseline, with each mutated line shown to
//! execute by watching the binary's exit code change:
//!
//! | site | mutation | suite |
//! | --- | --- | --- |
//! | `refuse_write_block`, Terminal arm | 2 → 3 | green, missed |
//! | `refuse_write_block`, WorldReadable arm | 2 → 3 | green, missed |
//! | `read_records`, `--in` error | 2 → 3 | green, missed |
//! | `read_records`, stdin error | 2 → 3 | green, missed |
//! | `emit`, write failure | 2 → 3 | green, missed |
//!
//! **Exit 2 is a usage error; exit 3 is a policy refusal.** They mean different
//! things to a script — *fix your command line* versus *this tool will never do
//! that* — and five of `me`'s refusals could swap one for the other undetected.
//!
//! **The tests checked that a refusal HAPPENED, never which kind**, which is
//! why the mutation survived. `!success()` cannot discharge this; only the
//! integer can. It is not a defect P0 introduced — the control proves it is the
//! state of the shipped binary — but P0 moves exactly these functions, and a
//! refactor over an untested distinction is how the distinction quietly dies.
//!
//! **Site 1 is pinned in `terminal_destination.rs`, not here**, because it
//! needs a real pty: none of the 12 `world_readable_output.rs` tests reaches
//! the terminal arm, and a pipe or a file is not a terminal. The other four are
//! reachable with ordinary process plumbing and live here.
//!
//! **Each assertion pins the digit AND a distinguishing phrase.** Pinning the
//! digit alone would let a DIFFERENT refusal that also exits 2 satisfy the
//! test — the same false-pass shape one level up.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Output, Stdio};

fn me() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("me")
}

fn err_of(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// The exit-code vocabulary, named so the assertions read as intent.
const USAGE: i32 = 2;

/// **Site 2 — `refuse_write_block`'s WorldReadable arm.**
/// stdout is a regular file whose mode grants group/other read.
#[test]
fn a_world_readable_stdout_is_usage_two() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("wr.bin");
    let f = fs::File::create(&out).unwrap();
    fs::set_permissions(&out, fs::Permissions::from_mode(0o644)).unwrap();

    let o = Command::new(me())
        .args(["sysw", "pack", "--no-passphrase", "text:6869"])
        .stdout(Stdio::from(f))
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert_eq!(
        o.status.code(),
        Some(USAGE),
        "a world-readable stdout is a USAGE error -- fix the redirect -- not a \
         policy refusal: {}",
        err_of(&o)
    );
    assert!(
        err_of(&o).contains("mode 0644"),
        "and it must be THIS refusal, quoting the mode it measured: {}",
        err_of(&o)
    );
}

/// **Site 3 — `read_records`'s `--in` error.** The file is not there.
#[test]
fn an_unreadable_in_file_is_usage_two() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nope.txt");

    let o = Command::new(me())
        .args(["sysw", "pack", "--no-passphrase", "--in"])
        .arg(&missing)
        .output()
        .unwrap();

    assert_eq!(
        o.status.code(),
        Some(USAGE),
        "a missing --in file is the operator's command line, not a policy \
         refusal: {}",
        err_of(&o)
    );
    assert!(
        err_of(&o).contains("No such file or directory"),
        "and it must be THIS refusal: {}",
        err_of(&o)
    );
}

/// **Site 4 — `read_records`'s stdin error.** `read_to_string` fails on bytes
/// that are not UTF-8, which is what a binary file piped in by mistake looks
/// like.
#[test]
fn unreadable_stdin_is_usage_two() {
    use std::io::Write;
    let mut child = Command::new(me())
        .args(["sysw", "pack", "--no-passphrase"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(&[0xff, 0xfe, 0xff])
        .unwrap();
    let o = child.wait_with_output().unwrap();

    assert_eq!(
        o.status.code(),
        Some(USAGE),
        "stdin that is not text is a usage error: {}",
        err_of(&o)
    );
    assert!(
        err_of(&o).contains("stdin:") && err_of(&o).contains("UTF-8"),
        "and it must be THIS refusal: {}",
        err_of(&o)
    );
}

/// **Site 5 — `emit`'s write failure.** A write that failed is the
/// environment, not the artifact: a read-only directory, a full disk, a closed
/// pipe. Here, a parent directory that does not exist.
#[test]
fn a_failed_write_is_usage_two() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("no-such-dir").join("p.bin");

    let o = Command::new(me())
        .args(["sysw", "pack", "--no-passphrase", "--out"])
        .arg(&out)
        .arg("text:6869")
        .output()
        .unwrap();

    assert_eq!(
        o.status.code(),
        Some(USAGE),
        "a write that failed is the environment, not a policy refusal: {}",
        err_of(&o)
    );
    assert!(
        err_of(&o).contains("No such file or directory"),
        "and it must be THIS refusal: {}",
        err_of(&o)
    );
    assert!(!out.exists(), "nothing may be left behind");
}

/// **THE CONTROL.** Every assertion above is `== 2`, so a build in which
/// everything exited 2 would satisfy all four. This pins that the same command,
/// with nothing wrong with it, exits 0 — and that a genuine POLICY refusal
/// still exits 3, so the two codes really are distinguishable in this suite.
#[test]
fn the_vocabulary_still_has_more_than_one_code_in_it() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("ok.bin");
    let o = Command::new(me())
        .args(["sysw", "pack", "--no-passphrase", "--out"])
        .arg(&out)
        .arg("text:6869")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "control: {}", err_of(&o));

    // A policy refusal: a `tx:` record on argv. Exit 3, and it must NOT be 2.
    let o = Command::new(me())
        .args(["sysw", "pack", "--no-passphrase", "tx:0100"])
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(3),
        "a policy refusal is 3 -- if this is 2, the distinction the four tests \
         above pin has collapsed and they are all passing for nothing: {}",
        err_of(&o)
    );
}
