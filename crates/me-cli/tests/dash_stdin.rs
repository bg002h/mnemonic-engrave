//! **§6b — `-` means stdin, and it is IMPLEMENTED rather than merely
//! accepted.**
//!
//! §6b's wording is permissive enough that an implementation could take the
//! flag and do nothing with it — and the compliant implementation is then
//! **silently lossy**. Measured before this work:
//!
//! ```text
//! printf 'text:6162\n' | me sysw pack --out b.bin - text:6869
//!   → exit 4, nothing written
//! ```
//!
//! `-` was read as a record and refused as unclassifiable. The dangerous
//! version is not that one: it is the implementation that ACCEPTS `-`, drops
//! it, and packs one record instead of two **at exit 0**, on the artifact that
//! gets cut into metal.
//!
//! **Only `sysw pack` gains anything, and the other four surfaces are asserted
//! UNCHANGED rather than assumed to be.** `sysw pack` is the only surface with
//! a positional RECORD list: `me` and `bundle` already default to stdin with no
//! positional at all, `sysw show`'s positional is a container FILE path, and
//! `sysw wipe` has no positional. So there is nowhere for a `-` to sit on any
//! of them, and each keeps the exact code it has today.
#![cfg(unix)]

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn me() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("me")
}

fn run(args: &[&str], stdin: &str) -> Output {
    let mut c = Command::new(me())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.as_mut().unwrap().write_all(stdin.as_bytes()).unwrap();
    c.wait_with_output().unwrap()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// **THE GATE.** The spliced container is **byte-for-byte** the container the
/// operator would have got by passing both records themselves.
///
/// That is a stronger assertion than a record count: it pins the ORDER too.
/// Appending stdin at the end instead of splicing in place would give the same
/// count, the same `pub_len`, and a different container — and record order is
/// what the operator typed.
#[test]
fn dash_splices_stdin_into_the_record_list_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let spliced = dir.path().join("spliced.bin");
    let explicit = dir.path().join("explicit.bin");
    let one = dir.path().join("one.bin");

    let o = run(
        &[
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            spliced.to_str().unwrap(),
            "-",
            "text:6869",
        ],
        "text:6162\n",
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));

    let o = run(
        &[
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            explicit.to_str().unwrap(),
            "text:6162",
            "text:6869",
        ],
        "",
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));

    assert_eq!(
        std::fs::read(&spliced).unwrap(),
        std::fs::read(&explicit).unwrap(),
        "`-` must splice stdin AT ITS OWN POSITION -- same records, same order, \
         same bytes"
    );

    // And the control that makes the equality mean something: a container with
    // ONE record is a different size, so the test above is not comparing two
    // identically-truncated files.
    let o = run(
        &[
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            one.to_str().unwrap(),
            "text:6869",
        ],
        "",
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_ne!(
        std::fs::read(&one).unwrap().len(),
        std::fs::read(&spliced).unwrap().len(),
        "a one-record container must not be the same size as a two-record one, \
         or the equality above would hold for a silently lossy implementation"
    );
}

/// Reading stdin twice is not possible, so a second `-` is refused rather than
/// silently duplicating or silently emptying the second one.
#[test]
fn a_second_dash_is_refused_rather_than_guessed_at() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let o = run(
        &[
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            out.to_str().unwrap(),
            "-",
            "-",
        ],
        "text:6162\n",
    );
    assert_eq!(o.status.code(), Some(2), "a usage error: {}", err(&o));
    assert!(
        err(&o).contains("stdin can only be read once"),
        "{}",
        err(&o)
    );
    assert!(!out.exists(), "nothing may be written");
}

/// **R7 again.** The operator ASKED for stdin, so nothing arriving there is the
/// failed-upstream signal — not an instruction to pack the rest and exit 0.
///
/// `fish` reports a pipeline's status as the LAST command's, so a failed
/// upstream arrives as nothing at all; splicing zero records would build a
/// container missing exactly what the pipeline was supposed to supply.
#[test]
fn an_empty_stdin_at_a_dash_is_refused_not_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.bin");
    let o = run(
        &[
            "sysw",
            "pack",
            "--no-passphrase",
            "--out",
            out.to_str().unwrap(),
            "-",
            "text:6869",
        ],
        "",
    );
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert!(
        err(&o).contains("no records on stdin"),
        "and it must be R7's refusal, not a generic one: {}",
        err(&o)
    );
    assert!(!out.exists(), "nothing may be written");
}

/// **THE DIFFERENTIAL'S OTHER HALF: unchanged everywhere except the enumerated
/// dash cells.** Each of these four codes was measured on the pre-change tree,
/// and each is justified rather than merely recorded — a table you match
/// against is a table you can update to match a regression.
#[test]
fn the_four_surfaces_a_dash_cannot_occupy_are_unchanged() {
    // `me -` : `me`'s own positional is a subcommand slot, so clap rejects `-`
    // as an unrecognised subcommand. There is no record list to splice into.
    let o = run(&["-"], "md1\n");
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert!(err(&o).contains("unrecognized subcommand"), "{}", err(&o));

    // `bundle -` : `bundle` already defaults to stdin and declares NO
    // positional, so `-` is an unexpected argument and adds nothing.
    let o = run(&["bundle", "-"], "md1\n");
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert!(err(&o).contains("unexpected argument"), "{}", err(&o));

    // `sysw wipe -` : no positional at all; the image is generated, not read.
    let o = run(&["sysw", "wipe", "-"], "");
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert!(err(&o).contains("unexpected argument"), "{}", err(&o));

    // `sysw show -` : its positional is a container FILE PATH, not a record, so
    // `-` is a filename and ENOENT is the honest answer. Making it mean stdin
    // here would be a different feature.
    let o = run(&["sysw", "show", "-"], "");
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert!(err(&o).contains("No such file or directory"), "{}", err(&o));
}
