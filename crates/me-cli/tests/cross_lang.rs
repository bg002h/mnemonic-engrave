use mnemonic_engrave::convert;
use std::io::Write;
use std::process::{Command, Stdio};

/// Cross-language anchor (spec §9): Rust-emitted NDEF parsed by SeedHammer's Go
/// reader must round-trip. Ignored by default; run with:
///   cargo test -p mnemonic-engrave --test cross_lang -- --ignored
#[test]
#[ignore]
fn rust_ndef_parses_in_seedhammer_go_reader() {
    let input = "md1yqpqqxqq8xtwhw4xwn4qh";
    let ndef = convert(input).unwrap();

    let mut child = Command::new("go")
        .args(["run", "."])
        .current_dir("firmware/ndef-roundtrip")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("go must be installed and firmware/ndef-roundtrip present");
    child.stdin.take().unwrap().write_all(&ndef).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "go harness failed: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8(out.stdout).unwrap(), input);
}
