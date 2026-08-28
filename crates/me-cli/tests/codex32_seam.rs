//! **The host/device codex32 seam, as a gate.**
//!
//! `host-admits ⇒ device-admits` used to live only in prose, in
//! `seal/record.rs` (*"The device's `codex32.New` has no such narrowing"*). A
//! comment cannot fail a build, and the direction it protects is not
//! symmetric: the host being NARROWER is safe, and a host that admitted what
//! the device refuses would pack a record into a payload the device cannot
//! read — an engraved backup that will not load.
//!
//! This half asserts the HOST column of
//! `testdata/codex32_seam_vectors.json` against `sysw::classify`, and the
//! implication against the file's device column. The fork's
//! `sysw/codex32_seam_test.go` reads a byte-identical copy of the same file and
//! asserts the DEVICE column against `sysw.Classify`. Neither implementation is
//! ever compared to the other — both are compared to the file, which is why the
//! file has to be the same file. [`SEAM_VECTORS_SHA256`] is what makes that
//! structural: the fork's test pins the same literal, so the two copies cannot
//! drift without one of the two suites going red.

use sha2::Digest as _;

/// The sha256 of `testdata/codex32_seam_vectors.json`, pinned IDENTICALLY in
/// the fork's `sysw/codex32_seam_test.go`. Changing a row means changing this
/// in both repos, which is the point — see the file's own header.
const SEAM_VECTORS_SHA256: &str =
    "3d53ef88a474f02c15aa60a839f4a31071598a26c853463122a847515926eb6a";

const PATH: &str = "testdata/codex32_seam_vectors.json";

#[test]
fn the_host_never_admits_what_the_device_would_refuse() {
    let raw = std::fs::read(PATH).unwrap_or_else(|e| panic!("{PATH}: {e}"));
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(&raw)),
        SEAM_VECTORS_SHA256,
        "{PATH} is not the file the fork's copy is pinned to; re-pin BOTH literals"
    );
    let doc: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let rows = doc["vectors"].as_array().unwrap();
    let (mut both, mut device_only, mut neither) = (0, 0, 0);
    for r in rows {
        let (name, s) = (r["name"].as_str().unwrap(), r["string"].as_str().unwrap());
        let (host, device) = (
            r["host_admits"].as_bool().unwrap(),
            r["device_admits"].as_bool().unwrap(),
        );
        // A mistyped vector must fail loudly, not quietly stop testing.
        assert_eq!(
            s.chars().count() as u64,
            r["chars"].as_u64().unwrap(),
            "{name}"
        );
        // THE SAFE DIRECTION. This is the assertion the file exists for.
        assert!(
            host <= device,
            "{name}: the HOST admits what the DEVICE refuses"
        );
        assert_eq!(
            mnemonic_engrave::sysw::classify(s)
                == mnemonic_engrave::sysw::record::Class::Codex32Secret,
            host,
            "{name}: host verdict"
        );
        match (host, device) {
            (true, true) => both += 1,
            (false, true) => device_only += 1,
            (false, false) => neither += 1,
            (true, false) => unreachable!("asserted above"),
        }
    }
    // All three shapes, or the set goes vacuous: with no yes/yes row a mutant
    // that refuses everything passes, with no no/no row one that admits
    // everything passes, and with no device-only row the seam is untested.
    assert!(
        both > 0 && device_only > 0 && neither > 0,
        "{} rows: {both} both / {device_only} device-only / {neither} neither",
        rows.len()
    );
}
