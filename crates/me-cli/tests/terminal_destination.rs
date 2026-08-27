//! F-253's terminal refusal, pinned through a REAL pty — and pinned by its
//! **exit digit**, not by `!success()`.
//!
//! **Why this file exists at all.** All 12 tests in `world_readable_output.rs`
//! redirect stdout to a file or a pipe, so **not one of them reaches the
//! terminal arm** of `write_block`. That arm is part of P0's moving set and it
//! carries F-259's funds-adjacent behaviour, so a refactor could delete it
//! outright and leave every gate green.
//!
//! **Why the DIGIT and not `!success()` (F-265).** Measured against the
//! unmodified binary: five of `me`'s refusals can be respelled from exit 2 to
//! exit 3 with all 388 tests still passing and the mutated line proven to
//! execute. A gate that only asks "did it fail" cannot see a refusal changing
//! what it means. This one asserts the integer.
//!
//! The technique is `script -qec CMD /dev/null` (util-linux): it runs CMD with
//! a pseudo-terminal on fd 1, and `-e` returns CMD's own exit status.
#![cfg(unix)]

use std::process::Command;

/// Run `me` with the given argument string under a pty, returning
/// `(exit code, combined pty output)`.
///
/// A pty merges stdout and stderr onto the one device, which is exactly the
/// situation the refusal exists for; the refusal's text is therefore in the
/// returned string regardless of which stream it was written to.
fn pty(args: &str) -> (i32, String) {
    let me = assert_cmd::cargo::cargo_bin("me");
    let me = me.display().to_string();
    assert!(
        !me.contains('\'') && !me.contains(' '),
        "the binary path is interpolated into a shell command: {me}"
    );
    let out = Command::new("script")
        .arg("-qec")
        .arg(format!("'{me}' {args}"))
        .arg("/dev/null")
        .output()
        .expect(
            "`script` (util-linux) is required for the pty gates; it is not optional and must \
             not be skipped -- a skipped gate reports ok",
        );
    let code = out
        .status
        .code()
        .expect("script was killed by a signal rather than exiting");
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    (code, s)
}

/// `me sysw pack` with stdout on a terminal refuses, **at exit 2**.
///
/// This is the assertion the P0 move carries with it: it passes before the
/// five functions move into `me`'s lib half and must pass after. Mutating
/// `refuse_write_block`'s `WriteBlock::Terminal` arm from `EXIT_USAGE` to
/// `EXIT_REFUSED` turns it RED — verified in both directions, which is what
/// distinguishes it from the greps, which are green on an untouched tree and
/// therefore cannot fail.
#[test]
fn a_terminal_destination_is_refused_at_exit_2() {
    let (code, out) = pty("sysw pack --no-passphrase text:6869");
    assert_eq!(
        code, 2,
        "the terminal refusal is EXIT_USAGE (2); got {code}. \
         Output was:\n{out}"
    );
    assert!(
        out.contains("TERMINAL") && out.contains("BEARER"),
        "the refusal must say what it refused and why: {out}"
    );
    assert!(
        out.contains("Nothing was written"),
        "the refusal must state that nothing was written: {out}"
    );
}

/// **The positive control, without which a refuse-everything change passes.**
/// A pty on fd 1 plus `--out` is a FILE destination: `me` creates the container
/// itself, owner-only, and exits 0. So the refusal above is keyed on the
/// destination and not on the presence of a terminal anywhere.
#[test]
fn a_pty_with_out_still_packs_at_exit_0() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("payload.bin");
    let p = out.display().to_string();
    assert!(!p.contains('\'') && !p.contains(' '), "tempdir path: {p}");

    let (code, txt) = pty(&format!("sysw pack --no-passphrase --out '{p}' text:6869"));
    assert_eq!(
        code, 0,
        "--out is a file destination, not a terminal: {txt}"
    );
    assert!(
        out.exists() && std::fs::metadata(&out).unwrap().len() > 0,
        "the container must actually have been written: {txt}"
    );
}

// ── F-259: the refusal is right, the stated reason was false ────────────────

/// **F-259.** `me sysw wipe --fill zeros` builds a 65,536-byte all-zeros fill
/// image whose *purpose* is to destroy a stored payload. On a terminal, `me`
/// refused it saying **"this payload is BEARER"** — because `wipe` smuggled
/// its "carries no secret" fact through the `allow_world_readable` parameter,
/// and the terminal arm never consults that parameter.
///
/// **The refusal is right; the reason was false.** Painting 64 KB of binary
/// across a scrollback is worth refusing whatever the secrecy. But the message
/// asserted something untrue about the operator's data, and that costs twice:
/// someone who wipes may believe they exposed a secret, and everyone learns the
/// BEARER label is unreliable.
///
/// **This assertion is the gate, and the type is a convenience.** A probe built
/// `WriteBlock::Terminal(PayloadKind)` in its strongest form, message derived
/// from the carried kind — and then re-wrote the bug by changing one pattern to
/// `WriteBlock::Terminal(_)`: clean build, clean clippy, 391/391 green, and the
/// pty printed "this payload is BEARER" once more. **A type stops a value being
/// CONFUSED for another value. It cannot stop a value being IGNORED.**
#[test]
fn a_wipe_image_is_never_called_bearer() {
    let (code, out) = pty("sysw wipe --fill zeros");
    assert!(
        !out.contains("BEARER") && !out.contains("bearer"),
        "a 65,536-byte zeros fill image carries no secret; saying BEARER about it \
         is F-259. Output was:\n{out}"
    );
    assert_eq!(
        code, 2,
        "the terminal refusal STAYS -- 64 KB of binary across a scrollback is \
         worth refusing whatever the secrecy -- so the digit does not move; only \
         the false claim goes. Got {code}. Output was:\n{out}"
    );
    assert!(
        out.contains("Nothing was written"),
        "the refusal must still state that nothing was written: {out}"
    );
}

/// **The positive control for F-259, in the opposite direction.** A change that
/// simply deleted the word BEARER everywhere would satisfy the test above. The
/// BEARER label must still appear where it is TRUE — on a real container.
#[test]
fn a_real_container_is_still_called_bearer() {
    let (code, out) = pty("sysw pack --no-passphrase text:6869");
    assert_eq!(code, 2, "unchanged: {out}");
    assert!(
        out.contains("BEARER"),
        "deleting the label is not the fix -- it must survive where it is true: {out}"
    );
}
