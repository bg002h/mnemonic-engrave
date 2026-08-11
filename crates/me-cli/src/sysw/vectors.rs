//! The vector fixtures — the contract the Go port is checked against.
//!
//! Plan stage 3: the Go implementation reads the SAME file. The two
//! implementations never compare to each other, only to this, so a vector both
//! sides are missing is a defect neither will ever notice. That is why the set
//! is derived from spec §8.3 through [`super::coverage::COVERAGE`] rather than
//! chosen, and why [`tests::every_required_vector_exists`] fails the build if a
//! spec test points at a vector that is not here.
//!
//! Regenerate after an intentional format change:
//!
//! ```sh
//! cargo test -p mnemonic-engrave --lib sysw::vectors::regenerate -- --ignored --nocapture
//! ```
//!
//! Regeneration is `--ignored` on purpose. A fixture that rewrites itself on
//! every run cannot fail: it would assert that today's code agrees with today's
//! code, which is exactly the tautology a golden file exists to avoid.

use serde::{Deserialize, Serialize};

use super::coverage::{FIXTURE_ITERATIONS, FIXTURE_IV, FIXTURE_SALT, VECTORS};
use super::{identity, pack_deterministic, pubhash, wire};

pub const PATH: &str = "testdata/sysw_vectors.json";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Vector {
    pub name: String,
    pub note: String,
    pub records: Vec<String>,
    pub passphrase: Option<String>,
    /// Whole blob, lowercase hex.
    pub blob: String,
    pub pub_len: u32,
    pub ct_len: u32,
    pub sealed: bool,
    /// `[digest-shown]` — **absent when `pub_len == 0`**, which is EPD §6.6's
    /// own rule and the case R0-C2 was raised about. A vector with a digest here
    /// for a secrets-only payload would encode the bug.
    pub digest: Option<String>,
    /// `[identity]`, always present.
    pub identity: String,
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Build every vector from the current implementation.
pub fn generate() -> Vec<Vector> {
    VECTORS
        .iter()
        .map(|v| {
            let records: Vec<String> = v.records.iter().map(|s| (*s).to_string()).collect();
            let blob = pack_deterministic(
                records.clone(),
                v.passphrase,
                FIXTURE_ITERATIONS,
                FIXTURE_SALT,
                FIXTURE_IV,
            )
            .expect("fixture inputs must pack");
            let h = wire::Header::parse(&blob).expect("a packed blob must parse");
            let pub_end = wire::HEADER_LEN + h.pub_len as usize;
            let public = &blob[wire::HEADER_LEN..pub_end];
            let digest = if h.pub_len > 0 {
                let s = std::str::from_utf8(public).expect("public section is utf-8");
                let refs: Vec<&str> = s.split('\n').collect();
                Some(hex(&pubhash::public_data_hash(&refs, h.sealed())))
            } else {
                None
            };
            Vector {
                name: v.name.to_string(),
                note: v.note.to_string(),
                records,
                passphrase: v.passphrase.map(str::to_string),
                blob: hex(&blob),
                pub_len: h.pub_len,
                ct_len: h.ct_len,
                sealed: h.sealed(),
                digest,
                identity: hex(&identity::identity(&blob)),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysw::{coverage, open};
    use std::path::PathBuf;

    fn path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PATH)
    }

    fn load() -> Vec<Vector> {
        let raw = std::fs::read_to_string(path())
            .unwrap_or_else(|e| panic!("{}: {e} — regenerate with --ignored", path().display()));
        serde_json::from_str(&raw).expect("vectors are valid json")
    }

    /// Not a test. `--ignored` so it never runs in CI: a fixture that rewrites
    /// itself asserts only that today's code agrees with today's code.
    #[test]
    #[ignore = "regenerates the fixture; run deliberately"]
    fn regenerate() {
        let v = generate();
        std::fs::write(path(), serde_json::to_string_pretty(&v).unwrap() + "\n").unwrap();
        println!("wrote {} vectors to {}", v.len(), path().display());
    }

    /// THE golden check: the implementation still produces the recorded bytes.
    #[test]
    fn the_implementation_still_matches_the_recorded_vectors() {
        assert_eq!(
            generate(),
            load(),
            "the container's output changed — if deliberate, regenerate"
        );
    }

    /// A spec §8.3 test may not point at a vector that does not exist.
    #[test]
    fn every_required_vector_exists() {
        let have: Vec<String> = load().into_iter().map(|v| v.name).collect();
        for want in coverage::required_vectors() {
            assert!(
                have.iter().any(|h| h == want),
                "spec §8.3 requires vector {want}, which is not in {PATH}"
            );
        }
    }

    /// Every vector must round-trip through `open`, or it is a fixture of
    /// something that cannot be read back.
    #[test]
    fn every_vector_opens() {
        for v in load() {
            let blob: Vec<u8> = (0..v.blob.len() / 2)
                .map(|i| u8::from_str_radix(&v.blob[i * 2..i * 2 + 2], 16).unwrap())
                .collect();
            let p = open(&blob, v.passphrase.as_deref())
                .unwrap_or_else(|e| panic!("{} does not open: {e:?}", v.name));
            let n = p.public.len() + p.secret.len();
            assert_eq!(n, v.records.len(), "{} lost or gained a record", v.name);
        }
    }

    /// R0-C2, as a property of the fixture set rather than of one case: a
    /// digest exists exactly when `pub_len > 0`.
    #[test]
    fn a_digest_is_recorded_exactly_when_a_public_section_exists() {
        let vs = load();
        for v in &vs {
            assert_eq!(
                v.digest.is_some(),
                v.pub_len > 0,
                "{}: digest presence must track pub_len > 0",
                v.name
            );
        }
        assert!(
            vs.iter().any(|v| v.pub_len == 0 && v.sealed),
            "the set must CONTAIN a secrets-only sealed payload, or this proves nothing"
        );
        assert!(
            vs.iter().any(|v| v.pub_len > 0),
            "and one with a public section"
        );
    }
}
