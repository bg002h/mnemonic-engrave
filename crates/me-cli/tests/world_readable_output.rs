//! F-244 — a bearer container must not land world-readable.
//!
//! Found by the Goal 1 journey walk at the operator's own words: *"I didn't
//! realize `>` creates a world readable file"*. `me sysw pack` wrote its
//! container with `std::fs::write`, so an UNSEALED payload holding a BIP-39
//! mnemonic landed at mode 0644 — readable by every user on the machine.
//!
//! **Two destinations, two mechanisms.** `--out` is a path `me` opens, so it can
//! be created and tightened. `>` is the shell's file, which `me` never names —
//! but it can `fstat` its own stdout and see the mode.
//!
//! **The last two tests are the NEAR MISSES**, and they are the point. Every
//! guard added during the `mt` cycle broke on the input that merely *resembles*
//! the one the finding named; a finding hands you a hostile X and never the
//! legitimate near-X. A pipe and an owner-only file must both pass.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

const TEXT: &str = "text:6869";

fn me_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("me")
}

fn mode_of(p: &std::path::Path) -> u32 {
    fs::metadata(p).unwrap().permissions().mode() & 0o777
}

/// `--out` on a path that does not exist yet must create it owner-only.
#[test]
fn out_creates_the_container_owner_only() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("fresh.bin");

    let st = Command::new(me_bin())
        .args(["sysw", "pack", "--no-passphrase", "--out"])
        .arg(&out)
        .arg(TEXT)
        .status()
        .unwrap();

    assert!(st.success(), "pack should succeed");
    assert_eq!(
        mode_of(&out),
        0o600,
        "a bearer container must not be group- or world-readable"
    );
}

/// `0o600` binds on CREATE. Creating carefully is therefore NOT enough: an
/// existing world-readable target keeps its mode, and that is the case an
/// operator re-running a command actually hits.
///
/// `write_private` moved to `mnemonic-io-lib`'s `write` module in P1 row 6 and
/// carries a unit test of the same shape. **This one stays**, because it is the
/// only one that measures it end to end: it runs the real binary, so it also
/// pins that `--out` still routes through that function at all.
#[test]
fn out_tightens_a_preexisting_world_readable_target() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("stale.bin");
    fs::write(&out, b"old").unwrap();
    fs::set_permissions(&out, fs::Permissions::from_mode(0o644)).unwrap();

    let st = Command::new(me_bin())
        .args(["sysw", "pack", "--no-passphrase", "--out"])
        .arg(&out)
        .arg(TEXT)
        .status()
        .unwrap();

    assert!(st.success(), "pack should succeed");
    assert_eq!(
        mode_of(&out),
        0o600,
        "overwriting a 0644 target must tighten it, not inherit it"
    );
}

/// The case the operator hit: `me sysw pack … > payload.bin`. `me` never sees
/// that path, but it can see the mode of its own stdout.
#[test]
fn refuses_a_world_readable_stdout_redirect() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("redirected.bin");
    let f = fs::File::create(&out).unwrap();
    fs::set_permissions(&out, fs::Permissions::from_mode(0o644)).unwrap();

    let res = Command::new(me_bin())
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .stdout(Stdio::from(f))
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        !res.status.success(),
        "a world-readable stdout must be refused"
    );
    let err = String::from_utf8_lossy(&res.stderr);
    assert!(
        err.contains("world-readable") || err.contains("readable by other users"),
        "the refusal must say WHY; got: {err}"
    );
    assert!(
        err.contains("--allow-world-readable"),
        "the refusal must name the override; got: {err}"
    );
}

/// The override exists so the refusal is a guard rather than a wall.
#[test]
fn the_override_permits_a_world_readable_stdout_redirect() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("permitted.bin");
    let f = fs::File::create(&out).unwrap();
    fs::set_permissions(&out, fs::Permissions::from_mode(0o644)).unwrap();

    let st = Command::new(me_bin())
        .args([
            "sysw",
            "pack",
            "--no-passphrase",
            "--allow-world-readable",
            TEXT,
        ])
        .stdout(Stdio::from(f))
        .status()
        .unwrap();

    assert!(st.success(), "the override must permit the write");
    assert!(
        fs::metadata(&out).unwrap().len() > 0,
        "the container must actually be written"
    );
}

// ---------------------------------------------------------------------------
// NEAR MISSES. Both MUST pass. A guard that catches these is worse than none.
// ---------------------------------------------------------------------------

/// `me sysw pack … | picotool` has no file mode at all. `S_ISFIFO`, not
/// `S_ISREG` — the check must not fire, or every pipeline in the constellation
/// breaks.
#[test]
fn does_not_refuse_a_pipe() {
    let res = Command::new(me_bin())
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        res.status.success(),
        "a pipe is not a world-readable file; stderr: {}",
        String::from_utf8_lossy(&res.stderr)
    );
    assert!(!res.stdout.is_empty(), "the container must reach the pipe");
}

/// An operator with `umask 077`, or one who tightened the file first, is already
/// safe and must not be refused.
#[test]
fn does_not_refuse_an_owner_only_stdout_redirect() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("private.bin");
    let f = fs::File::create(&out).unwrap();
    fs::set_permissions(&out, fs::Permissions::from_mode(0o600)).unwrap();

    let res = Command::new(me_bin())
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .stdout(Stdio::from(f))
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        res.status.success(),
        "an owner-only file is exactly what we want; stderr: {}",
        String::from_utf8_lossy(&res.stderr)
    );
    assert!(
        fs::metadata(&out).unwrap().len() > 0,
        "the container must actually be written"
    );
}

/// NEAR MISS I INTRODUCED WHILE FIXING F-244, caught by asking what else the
/// guard would now catch rather than by a failing test.
///
/// `emit` is shared, so gating it gated `sysw wipe` too — and a wipe image is
/// 65,536 bytes of fill (`random`/`zeros`/`ones`) with **nothing in it**. It is
/// the opposite of bearer: its whole purpose is to destroy a payload. Refusing
/// it buys no safety and costs the operator a working command.
///
/// This is the fifth instance of the pattern the `mt` cycle recorded: *every
/// guard added in response to a finding broke on the input that merely resembles
/// the one the finding named.*
#[test]
fn does_not_refuse_a_wipe_image_which_carries_no_secret() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("wipe.bin");
    let f = fs::File::create(&out).unwrap();
    fs::set_permissions(&out, fs::Permissions::from_mode(0o644)).unwrap();

    let res = Command::new(me_bin())
        .args(["sysw", "wipe", "--fill", "ones"])
        .stdout(Stdio::from(f))
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        res.status.success(),
        "a wipe image holds no secret; refusing it is over-firing. stderr: {}",
        String::from_utf8_lossy(&res.stderr)
    );
    assert_eq!(
        fs::metadata(&out).unwrap().len(),
        65_536,
        "the region image must still be written in full"
    );
}

// ---------------------------------------------------------------------------
// The NDEF converter. `me`'s own note above `write_private`'s import (main.rs)
// already rules these bytes worth protecting: "NDEF and manifest artifacts
// embed or depict md1/mk1 material, so on a multi-user host their at-rest
// copies must not be world- or group-readable." `--out` honoured that; the
// stdout modes did not. (The function moved to the shared crate in P1 row 6;
// the REASON stayed here, because naming md1/mk1 is `me`'s job and not the
// crate's.)
// ---------------------------------------------------------------------------

const MD1: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";

fn md1_file(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let p = dir.path().join("card.txt");
    fs::write(&p, MD1).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
    p
}

#[test]
fn converter_refuses_a_world_readable_stdout_redirect() {
    let dir = tempfile::tempdir().unwrap();
    let src = md1_file(&dir);
    let out = dir.path().join("ndef.bin");
    let f = fs::File::create(&out).unwrap();
    fs::set_permissions(&out, fs::Permissions::from_mode(0o644)).unwrap();

    let res = Command::new(me_bin())
        .arg("--in")
        .arg(&src)
        .arg("--stdout")
        .stdout(Stdio::from(f))
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        !res.status.success(),
        "--stdout to a world-readable file must be refused"
    );
    let err = String::from_utf8_lossy(&res.stderr);
    assert!(
        err.contains("--allow-world-readable"),
        "the refusal must name the override; got: {err}"
    );
}

/// `--hex` is the same bytes in a different coat. A guard that catches raw and
/// not hex teaches the operator to reach for hex.
#[test]
fn converter_refuses_a_world_readable_hex_redirect() {
    let dir = tempfile::tempdir().unwrap();
    let src = md1_file(&dir);
    let out = dir.path().join("ndef.hex");
    let f = fs::File::create(&out).unwrap();
    fs::set_permissions(&out, fs::Permissions::from_mode(0o644)).unwrap();

    let res = Command::new(me_bin())
        .arg("--in")
        .arg(&src)
        .arg("--hex")
        .stdout(Stdio::from(f))
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(!res.status.success(), "--hex must be gated like --stdout");
}

/// NEAR MISS. `me --in card.txt --hex | xxd` must keep working.
#[test]
fn converter_does_not_refuse_a_pipe() {
    let dir = tempfile::tempdir().unwrap();
    let src = md1_file(&dir);

    let res = Command::new(me_bin())
        .arg("--in")
        .arg(&src)
        .arg("--hex")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        res.status.success(),
        "a pipe must not be refused; stderr: {}",
        String::from_utf8_lossy(&res.stderr)
    );
    assert!(!res.stdout.is_empty(), "the NDEF bytes must reach the pipe");
}

// ---------------------------------------------------------------------------
// R0 round 0, finding I3. The first fix keyed on `is_file()`, and the spec said
// "a pipe/FIFO has no file mode" -- citing a measurement as proof. MEASURED
// FALSE: a NAMED fifo carries a mode (0666 from mkfifo) and a third party
// reading it really does receive the bytes. Only the ANONYMOUS pipe is 0600.
// ---------------------------------------------------------------------------

/// Open a FIFO `O_RDWR` — on Linux that does not block waiting for a reader,
/// which opening write-only would.
#[cfg(unix)]
fn open_fifo_rdwr(p: &std::path::Path) -> fs::File {
    fs::OpenOptions::new().read(true).write(true).open(p).unwrap()
}

#[test]
fn refuses_a_world_readable_named_fifo() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("leak");
    std::process::Command::new("mkfifo")
        .arg(&p)
        .status()
        .unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o666)).unwrap();

    let res = Command::new(me_bin())
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .stdout(Stdio::from(open_fifo_rdwr(&p)))
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        !res.status.success(),
        "a 0666 named FIFO is readable by others and really leaks; it must be refused"
    );
}

/// NEAR MISS, and the sharpest one yet: `/dev/null` is mode **0666**. A guard
/// that looks only at permission bits refuses `me … > /dev/null`, which is one
/// of the most ordinary things anyone does with a CLI. Character devices persist
/// nothing, so they are exempt.
#[test]
fn does_not_refuse_dev_null() {
    let f = fs::OpenOptions::new().write(true).open("/dev/null").unwrap();
    let res = Command::new(me_bin())
        .args(["sysw", "pack", "--no-passphrase", TEXT])
        .stdout(Stdio::from(f))
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    assert!(
        res.status.success(),
        "/dev/null is 0666 but persists nothing; stderr: {}",
        String::from_utf8_lossy(&res.stderr)
    );
}
