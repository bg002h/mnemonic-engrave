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
    // N-1 (post-impl review): assert the EXACT variant, not merely "some Err".
    // `Err(_) => {}` would accept the wrong refusal as readily as the right
    // one — at the 0.8 bump the distinguishing error becomes a different
    // variant, and a `PreimageLengthMismatch` on the wrong string would pass.
    // Not `MsTooLong`: 75 characters is inside the engraveable cap, and the
    // refusal must be about the KIND, not the length.
    match validate_record(PREIMAGE_PLATE) {
        Ok(kind) => panic!("validate_record admitted a 0x03 preimage plate as {kind:?}"),
        Err(RecordError::PreimagePlate) => {}
        Err(other) => panic!("refused as {other:?}, not as a preimage plate"),
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

/// I-2 (post-impl review): the SECOND host verb. `me seal` reaches the same
/// `validate_record`, and before this it stopped at the raw codec string
/// ("invalid record: reserved-prefix byte was 0x03, expected 0x00") — no kind
/// name, no guidance, and in a multi-record seal no way to tell which record.
/// The named diagnosis must exist on both verbs, not just `me sysw pack`.
#[test]
fn seal_names_a_preimage_plate_and_never_echoes_it() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    let mut c = Command::new(assert_cmd::cargo::cargo_bin("me"))
        .args(["seal", "--seal-secret", "--out", out.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = c
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{PREIMAGE_PLATE}\n").as_bytes());
    let out = c.wait_with_output().unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "me seal accepted a preimage plate: {err}"
    );
    // MUTATION: drop the `preimage_plate` arm from `validate_record` -> this
    // reports the raw codec string and both of the next two asserts fire.
    assert!(
        err.contains("hashlock PREIMAGE plate"),
        "stderr does not name the kind:\n{err}"
    );
    assert!(
        !err.contains("reserved-prefix byte"),
        "the raw codec error is still what the operator sees:\n{err}"
    );
    assert!(
        !err.contains(&PREIMAGE_PLATE[10..40]),
        "the record was echoed:\n{err}"
    );
}

/// H1b (F-473): the codec now DECODES the plate — prove the refusal is on the
/// success path, not an accident of the pin. MUTATION: replace the
/// `Ok((_, Payload::Preimage(_)))` arm with `Ok(_) => Ok(RecordKind::Ms)` (i.e.
/// delete the arm) -> this fails on its SECOND assertion (`decoded` still
/// succeeds — the codec is unchanged; measured), and
/// `a_preimage_plate_is_not_a_seed_record` fails with "admitted ... as Ms".
#[test]
fn the_codec_decodes_the_plate_and_me_still_refuses_it() {
    let decoded = ms_codec::decode(PREIMAGE_PLATE);
    assert!(
        matches!(decoded, Ok((_, ms_codec::Payload::Preimage(_)))),
        "ms-codec did not decode the plate as Payload::Preimage: {decoded:?}"
    );
    assert!(matches!(
        validate_record(PREIMAGE_PLATE),
        Err(RecordError::PreimagePlate)
    ));
}

/// F-489: `me seal` names WHICH record it refused, the way `me sysw pack` does
/// ("record N (records count from 0)"), and which section it sat in. Two secret
/// records: an entr-32 seed at 0 and the plate at 1 -- the index must be 1.
/// MUTATION: wrap the secret loop's error as `SealError::Record(e)` again (no
/// index) -> the first assertion fails.
#[test]
fn seal_names_the_refused_record_index_like_sysw_pack() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    const ENTR32: &str =
        "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w";
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    let mut c = Command::new(assert_cmd::cargo::cargo_bin("me"))
        .args(["seal", "--seal-secret", "--out", out.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = c
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{ENTR32}\n{PREIMAGE_PLATE}\n").as_bytes());
    let out = c.wait_with_output().unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "me seal accepted a payload holding a plate: {err}"
    );
    assert!(
        err.contains("record 1 (records count from 0)"),
        "stderr does not name the refused record's index the way sysw pack does:\n{err}"
    );
    assert!(
        err.contains("secret section"),
        "stderr does not name the section:\n{err}"
    );
    assert!(
        err.contains("hashlock PREIMAGE plate"),
        "stderr does not name the kind:\n{err}"
    );
    assert!(
        !err.contains(&PREIMAGE_PLATE[10..40]),
        "the record was echoed:\n{err}"
    );
}

/// F-489, the PUBLIC section: the same locator on `check_public`'s per-record
/// loop. A valid md1 at 0 and the same string with one character changed at 1
/// (its checksum no longer holds) -- the index must be 1 and the section
/// "public". MUTATION: return `SealError::Record(e)` from `check_public`'s
/// per-record arm again -> the first assertion fails.
#[test]
fn seal_names_the_refused_public_record_index_too() {
    use std::process::Command;
    const MD1: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
    let mut corrupted = MD1.to_string();
    // Flip the last character to another bech32 character so the checksum fails.
    let last = corrupted.pop().unwrap();
    corrupted.push(if last == 'q' { 'p' } else { 'q' });
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("p.uf2");
    let o = Command::new(assert_cmd::cargo::cargo_bin("me"))
        .args([
            "seal",
            "--plaintext",
            MD1,
            "--plaintext",
            &corrupted,
            "--out",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        !o.status.success(),
        "me seal accepted a corrupted public record: {err}"
    );
    assert!(
        err.contains("record 1 (records count from 0) in the public section"),
        "stderr does not name the refused PUBLIC record's index:\n{err}"
    );
    assert!(
        !err.contains("record 0 (records count from 0)"),
        "the wrong record was named:\n{err}"
    );
}
