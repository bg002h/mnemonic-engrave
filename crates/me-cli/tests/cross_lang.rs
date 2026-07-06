use mnemonic_engrave::ndef::encode_text_tlv;
use mnemonic_engrave::convert;
use std::io::Write;
use std::process::{Command, Stdio};

/// `true` iff `ME_REQUIRE_GO=1` — CI sets this so a missing Go toolchain is a
/// HARD FAILURE (the differential oracle must not silently no-op, F3) rather than
/// a skip. Locally-unset behavior is unchanged (skip note + pass).
fn go_required() -> bool {
    std::env::var("ME_REQUIRE_GO").map(|v| v == "1").unwrap_or(false)
}

/// Decode `ndef` through SeedHammer's OWN Go `nfc/ndef` reader
/// (`firmware/ndef-roundtrip`, hermetic against the pinned submodule after A4)
/// and return the recovered text-record body.
fn oracle_decode(ndef: &[u8]) -> String {
    // cargo runs tests with CWD = the crate dir; resolve the harness relative to
    // CARGO_MANIFEST_DIR (= crates/me-cli), not an assumed repo root.
    let harness = concat!(env!("CARGO_MANIFEST_DIR"), "/../../firmware/ndef-roundtrip");
    let mut child = Command::new("go")
        .args(["run", "."])
        .current_dir(harness)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn go harness");
    child.stdin.take().unwrap().write_all(ndef).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "go harness failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("harness stdout is UTF-8")
}

/// A length-`len` synthetic ASCII text with per-position variety, so the oracle
/// round-trip catches truncation/reordering (not just length). NDEF Text records
/// carry arbitrary UTF-8, so these bypass codec validation by construction.
fn synthetic(len: usize) -> String {
    const A: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    (0..len).map(|i| A[i % A.len()] as char).collect()
}

/// Cross-language anchor (spec §9 / B2): everything `me` emits — every
/// convert-level golden AND NDEF-layer synthetic texts at the interesting
/// lengths — must round-trip through SeedHammer's real Go reader to the exact
/// input, positionally. No golden is asserted ONLY via me's own
/// `decode_text_tlv`. Auto-skips when `go` is unavailable UNLESS
/// `ME_REQUIRE_GO=1` (then a missing `go` fails hard).
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

    // (1) Every convert-level golden (B1): encode via convert(), decode via the
    // independent oracle.
    const GOLDEN_INPUTS: &[&str] = &[
        "md1yqpqqxqq8xtwhw4xwn4qh",
        "md15kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd9uguh8nmgfllzz",
        "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x",
        "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf",
    ];
    for input in GOLDEN_INPUTS {
        let ndef = convert(input).unwrap_or_else(|e| panic!("convert {input:?}: {e}"));
        let decoded = oracle_decode(&ndef);
        assert_eq!(&decoded, input, "convert-golden round-trip mismatch");
    }

    // (2) NDEF-layer synthetic texts at the interesting lengths (bypass codec
    // validation by construction; positional check).
    for &len in &[1usize, 63, 64, 96, 111, 248, 249] {
        let text = synthetic(len);
        let ndef = encode_text_tlv(&text).expect("synthetic text must encode");
        let decoded = oracle_decode(&ndef);
        assert_eq!(decoded, text, "synthetic len={len} round-trip mismatch");
    }
}
