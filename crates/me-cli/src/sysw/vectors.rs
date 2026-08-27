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
use std::fmt::Write as _;

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
    /// `[mdmk-decode]` (§12.6): the indices of the PUBLIC-section records that
    /// are not decode-confirmed.
    ///
    /// **Indices are into the public section, not into `records`.** That is the
    /// one list both implementations reconstruct identically from `blob` — a
    /// sealed payload's `records` field is the packing order, which the port
    /// never sees. It costs nothing: `Class::MdMk` is not a secret class, so
    /// every `md1`/`mk1` record is in the public section in both variants.
    #[serde(default)]
    pub mdmk_unconfirmed: Vec<usize>,
}

fn hex(b: &[u8]) -> String {
    b.iter().fold(String::new(), |mut acc, x| {
        let _ = write!(acc, "{x:02x}");
        acc
    })
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
            let public_records: Vec<String> = if h.pub_len > 0 {
                std::str::from_utf8(public)
                    .expect("public section is utf-8")
                    .split('\n')
                    .map(str::to_owned)
                    .collect()
            } else {
                Vec::new()
            };
            let digest = if h.pub_len > 0 {
                let refs: Vec<&str> = public_records.iter().map(String::as_str).collect();
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
                mdmk_unconfirmed: super::record::mdmk_unconfirmed(&public_records),
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

    /// I5, pre-flash conformance: the artifact actually written to flash is a
    /// 65536-byte region — container at offset 0, `0xFF` to the end. Every
    /// vector is a bare blob, so nothing pinned that both sides TRIM to the
    /// header's declared total before hashing. They agree today; measured. This
    /// is what keeps them agreeing.
    ///
    /// As a property over the whole set rather than one new fixture: a 128 KB
    /// hex blob in the JSON would pin one payload, this pins all of them.
    #[test]
    fn padding_a_vector_to_a_full_region_changes_no_identity_and_no_digest() {
        for v in load() {
            let blob: Vec<u8> = (0..v.blob.len() / 2)
                .map(|i| u8::from_str_radix(&v.blob[i * 2..i * 2 + 2], 16).unwrap())
                .collect();
            let mut region = blob.clone();
            region.resize(wire::REGION_LEN, 0xFF);

            assert_eq!(
                hex(&identity::identity(&blob)),
                v.identity,
                "{}: the bare blob must still match its recorded identity",
                v.name
            );
            let h = wire::Header::parse(&region).expect("a padded region still parses");
            assert_eq!(
                hex(&identity::identity(&region[..h.total_len()])),
                v.identity,
                "{}: padding to a full region must not move the identity — if this \
                 fails, the device and the host disagree about every flashed payload",
                v.name
            );
        }
    }

    /// `[mdmk-decode]` (§12.6), pinned from the BLOB rather than from the
    /// `records` field — because the blob is all the Go port gets, and a field
    /// checked only against the generator would pin nothing the port can reach.
    ///
    /// It also guards the guard. A set whose every vector recorded an empty list
    /// would be passed by an implementation that answers "confirmed" to
    /// everything, and a set where every `md1` were unconfirmed would be passed
    /// by one that answers "unconfirmed" to everything. So the assertions below
    /// demand BOTH answers, and demand them inside ONE payload: only there does
    /// the `(hrp, chunk_set_id)` grouping have to be right.
    #[test]
    fn the_decode_check_is_recomputed_from_the_blob_and_carries_both_answers() {
        let vs = load();
        let mut confirmed_beside_unconfirmed = 0;
        for v in &vs {
            let blob: Vec<u8> = (0..v.blob.len() / 2)
                .map(|i| u8::from_str_radix(&v.blob[i * 2..i * 2 + 2], 16).unwrap())
                .collect();
            let h = wire::Header::parse(&blob).unwrap();
            let public: Vec<String> = if h.pub_len > 0 {
                std::str::from_utf8(&blob[wire::HEADER_LEN..wire::HEADER_LEN + h.pub_len as usize])
                    .unwrap()
                    .split('\n')
                    .map(str::to_owned)
                    .collect()
            } else {
                Vec::new()
            };
            assert_eq!(
                crate::sysw::record::mdmk_unconfirmed(&public),
                v.mdmk_unconfirmed,
                "{}: the recorded unconfirmed set is not what the public section yields",
                v.name
            );
            let mdmk = public
                .iter()
                .filter(|r| crate::sysw::classify(r) == crate::sysw::record::Class::MdMk)
                .count();
            if mdmk > v.mdmk_unconfirmed.len() && !v.mdmk_unconfirmed.is_empty() {
                confirmed_beside_unconfirmed += 1;
            }
        }
        assert!(
            confirmed_beside_unconfirmed >= 1,
            "no vector holds a confirmed card BESIDE an unconfirmed one, so nothing here \
             can fail an implementation that answers the same way for every record"
        );
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
