use mnemonic_engrave::convert;
use std::io::Write;
use std::process::{Command, Stdio};

/// `true` iff `ME_REQUIRE_GO=1` — CI sets this so a missing Go toolchain is a
/// HARD FAILURE (the differential oracle must not silently no-op) rather than a
/// skip (F3). Locally-unset behavior is unchanged (skip note + pass).
fn go_required() -> bool {
    std::env::var("ME_REQUIRE_GO").map(|v| v == "1").unwrap_or(false)
}

/// Cross-language anchor (spec §9): NDEF emitted by the converter, parsed by
/// SeedHammer's own Go `nfc/ndef` reader, must round-trip to the exact input.
/// Auto-skips when the Go toolchain is unavailable (e.g. CI without Go) so the
/// suite stays green everywhere UNLESS `ME_REQUIRE_GO=1`, in which case a missing
/// `go` fails hard; runs for real wherever `go` is on PATH.
#[test]
fn rust_ndef_parses_in_seedhammer_go_reader() {
    if Command::new("go").arg("version").output().is_err() {
        assert!(
            !go_required(),
            "ME_REQUIRE_GO=1 but `go` is not on PATH: the cross-language NDEF oracle \
             cannot run (install Go + init the seedhammer submodule, or unset ME_REQUIRE_GO)"
        );
        eprintln!("skipping cross-language round-trip: `go` is not on PATH");
        return;
    }

    // The Go harness lives at <repo>/firmware/ndef-roundtrip. cargo runs this
    // test with CWD = the crate directory, so resolve the harness relative to
    // CARGO_MANIFEST_DIR (= crates/me-cli) rather than assuming the repo root.
    let harness = concat!(env!("CARGO_MANIFEST_DIR"), "/../../firmware/ndef-roundtrip");

    let input = "md1yqpqqxqq8xtwhw4xwn4qh";
    let ndef = convert(input).unwrap();

    let mut child = Command::new("go")
        .args(["run", "."])
        .current_dir(harness)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn go harness");
    child.stdin.take().unwrap().write_all(&ndef).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "go harness failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8(out.stdout).unwrap(), input);
}
