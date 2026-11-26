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
//! Since the constellation exit-code spec (`SPEC_constellation_exit_codes.md`,
//! 2026-11-26) the digits are the constellation's: the world-readable
//! destination is REFUSED (8, a destination change or `--allow-world-readable`
//! waives it), a missing `--in` file and a failed write are IO (32), and stdin
//! that is not text is an INVALID input (2). The rows pin those digits.
//!
//! **The tests checked that a refusal HAPPENED, never which kind**, which is
//! why the mutation survived. `!success()` cannot discharge this; only the
//! integer can.
//!
//! **Site 1 is pinned in `terminal_destination.rs`, not here**, because it
//! needs a real pty: none of the 12 `world_readable_output.rs` tests reaches
//! the terminal arm, and a pipe or a file is not a terminal. The other four are
//! reachable with ordinary process plumbing and live here.
//!
//! **Each assertion pins the digit AND a distinguishing phrase.** Pinning the
//! digit alone would let a DIFFERENT refusal with the same digit satisfy the
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
const INVALID: i32 = 2;
const REFUSED: i32 = 8;
const IO: i32 = 32;
const USAGE: i32 = 64;

/// **Site 2 — `refuse_write_block`'s WorldReadable arm.**
/// stdout is a regular file whose mode grants group/other read.
#[test]
fn a_world_readable_stdout_is_refused_eight() {
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
        Some(REFUSED),
        "a world-readable stdout is REFUSED -- a destination change or \
         --allow-world-readable waives it: {}",
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
fn an_unreadable_in_file_is_io_thirty_two() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nope.txt");

    let o = Command::new(me())
        .args(["sysw", "pack", "--no-passphrase", "--in"])
        .arg(&missing)
        .output()
        .unwrap();

    assert_eq!(
        o.status.code(),
        Some(IO),
        "a missing --in file is something needed from the system: {}",
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
fn stdin_that_is_not_text_is_invalid_two() {
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
        Some(INVALID),
        "stdin that is not text is an invalid input: {}",
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
/// pipe. Here, a parent directory that does not exist. A failed write is IO,
/// exit 32; this row pins that digit.
#[test]
fn a_failed_write_is_io_thirty_two() {
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
        Some(IO),
        "a write that failed is the environment, not a policy refusal: {}",
        err_of(&o)
    );
    assert!(
        err_of(&o).contains(&format!(
            "me: cannot write the output: {}: No such file or directory (os error 2). Nothing was written.",
            out.display()
        )),
        "and it must be THIS refusal: {}",
        err_of(&o)
    );
    assert!(!out.exists(), "nothing may be left behind");
}

/// **THE CONTROL.** The same command with nothing wrong with it exits 0, and a
/// genuine USAGE error exits 64 -- so a build in which everything exited one
/// digit could not satisfy the rows above.
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

    // A policy refusal: a `tx:` record on argv. REFUSED, 8.
    let o = Command::new(me())
        .args(["sysw", "pack", "--no-passphrase", "tx:0100"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(REFUSED), "{}", err_of(&o));

    // A usage error: a setting out of range, judged from the command line.
    let o = Command::new(me())
        .args(["sysw", "pack", "--no-passphrase", "--iterations", "5"])
        .arg("text:6869")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(USAGE), "{}", err_of(&o));
}

/// R11: the root `--help` ends with the spec's legend, verbatim, and so does
/// `-h`. The text is `mnemonic_io_lib::exit::HELP_LEGEND`, which its own unit
/// test pins to the spec.
#[test]
fn the_root_help_ends_with_the_exit_status_legend() {
    for flag in ["--help", "-h"] {
        let o = Command::new(me()).arg(flag).output().unwrap();
        assert_eq!(o.status.code(), Some(0), "{flag}");
        let out = String::from_utf8(o.stdout).unwrap();
        assert!(
            out.trim_end()
                .ends_with(mnemonic_engrave::io::exit::HELP_LEGEND),
            "{flag} does not end with the legend:\n{out}"
        );
        assert!(out.contains("Exit status: 0 = done"), "{flag}");
    }
}
