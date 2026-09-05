//! H0 (SPEC_ms_hashlock §9): a kind-0x03 hashlock PREIMAGE plate is never a
//! seed record on the host.
//!
//! At ms-codec 0.7 the codec's own prefix gate refuses the string, so this
//! passes without any change to `validate_record`. When `me` bumps to
//! ms-codec 0.8 (stage H1b) the codec DECODES it as `Payload::Preimage`, and
//! `validate_record`'s `.map(|_| RecordKind::Ms)` would call a preimage a
//! seed. This test is what goes red that day. The seam corpus row
//! `preimage-plate-0x03` pins the same fact through `sysw::classify`.
use mnemonic_engrave::seal::record::{validate_record, RecordError};
use mnemonic_engrave::sysw::record::Class;

/// The H1 plan's downgrade-row string: 75 characters, id `hash`, prefix 0x03.
const PREIMAGE_PLATE: &str =
    "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c";

#[test]
fn a_preimage_plate_is_not_a_seed_record() {
    assert_eq!(PREIMAGE_PLATE.len(), 75);
    // MUTATION: `.map(|_| RecordKind::Ms)` admitting Payload::Preimage after
    // the 0.8 bump -> this arm receives Ok(RecordKind::Ms) and panics.
    match validate_record(PREIMAGE_PLATE) {
        Ok(kind) => panic!("validate_record admitted a 0x03 preimage plate as {kind:?}"),
        // Not `MsTooLong`: 75 characters is inside the engraveable cap, and
        // the refusal must be about the KIND, not the length.
        Err(RecordError::MsTooLong(n)) => panic!("refused as too long ({n}), not as a preimage"),
        Err(_) => {}
    }
    assert_ne!(
        mnemonic_engrave::sysw::classify(PREIMAGE_PLATE),
        Class::Codex32Secret,
        "sysw::classify called a preimage plate a codex32 secret"
    );
}

/// The operator-visible half (fidelity I-3): `me sysw pack` names the kind.
/// The record itself is never echoed.
#[test]
fn sysw_pack_names_a_preimage_plate_and_never_echoes_it() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut c = Command::new(assert_cmd::cargo::cargo_bin("me"))
        .args(["sysw", "pack"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // A broken pipe is the refusal arriving first, not an error (see dash_stdin.rs).
    let _ = c
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{PREIMAGE_PLATE}\n").as_bytes());
    let out = c.wait_with_output().unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "sysw pack accepted a preimage plate: {err}"
    );
    assert!(
        err.contains("hashlock PREIMAGE plate"),
        "stderr does not name the kind:\n{err}"
    );
    assert!(
        !err.contains("outside") && !err.contains("re-encode the entropy"),
        "misdiagnosed as outside the profile:\n{err}"
    );
    assert!(
        !err.contains(&PREIMAGE_PLATE[10..40]),
        "the record was echoed:\n{err}"
    );
}
