//! Per-record validation, and the §6.3 card-set decode for the public section.

use crate::classify::{classify, Format};
use crate::validate::first_noncanonical;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    /// md1 — wallet policy. Public.
    Md,
    /// mk1 — xpub + origin. Public.
    Mk,
    /// ms1 — the seed. Secret.
    Ms,
}

impl RecordKind {
    pub fn is_secret(self) -> bool {
        matches!(self, RecordKind::Ms)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecordError {
    NonCanonical { ch: char, pos: usize },
    NotLowercase(usize),
    Unclassifiable(String),
    Invalid(String),
    UndecodableSet(String),
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordError::NonCanonical { ch, pos } => write!(
                f,
                "non-canonical record: separator {ch:?} at byte {pos} — records must be \
                 unbroken. If this came from `mnemonic bundle`, re-run with --group-size 0: \
                 the default --group-size 5 emits a DISPLAY form the engraver cannot read."
            ),
            RecordError::NotLowercase(pos) => write!(
                f,
                "record has an uppercase character at byte {pos} — records must be lowercase, \
                 or the same wallet has two different public-data hashes (§6.4)"
            ),
            RecordError::Unclassifiable(e) => write!(f, "unrecognised record: {e}"),
            RecordError::Invalid(e) => write!(f, "invalid record: {e}"),
            RecordError::UndecodableSet(e) => write!(
                f,
                "public records do not form a decodable card set: {e} — a BCH-valid string is \
                 not proof of a real wallet card (§6.3)"
            ),
        }
    }
}
impl std::error::Error for RecordError {}

/// Validate one record: canonical, lowercase, correct BCH checksum. Reports what
/// it is. Does NOT decode — see `decode_public_set`.
pub fn validate_record(s: &str) -> Result<RecordKind, RecordError> {
    let s = s.trim();
    if let Some((pos, ch)) = first_noncanonical(s) {
        return Err(RecordError::NonCanonical { ch, pos });
    }
    if let Some(pos) = s
        .char_indices()
        .find(|(_, c)| c.is_uppercase())
        .map(|(i, _)| i)
    {
        return Err(RecordError::NotLowercase(pos));
    }
    let fmt = classify(s).map_err(|e| RecordError::Unclassifiable(e.to_string()))?;
    match fmt {
        Format::Md => md_codec::codex32::unwrap_string(s)
            .map(|_| RecordKind::Md)
            .map_err(|e| RecordError::Invalid(e.to_string())),
        Format::Mk => {
            let d = mk_codec::string_layer::decode_string(s)
                .map_err(|e| RecordError::Invalid(e.to_string()))?;
            if d.corrections_applied != 0 {
                return Err(RecordError::Invalid(format!(
                    "not pristine: required {} BCH correction(s)",
                    d.corrections_applied
                )));
            }
            Ok(RecordKind::Mk)
        }
        // ms_codec::decode, NOT decode_with_correction — a seed that needed
        // repair must be fixed at source, not engraved.
        Format::Ms => ms_codec::decode(s)
            .map(|_| RecordKind::Ms)
            .map_err(|e| RecordError::Invalid(e.to_string())),
    }
}

/// §6.3: every public record must belong to a card set that REASSEMBLES AND
/// DECODES. Records are chunks, so this is necessarily a whole-set operation —
/// a per-record decode rejects every legitimate payload.
///
/// **Group by `(HRP, chunk_set_id)`, NEVER by HRP alone.** A 2-of-3
/// `wsh-sortedmulti` wallet has THREE separate `mk1` cards and three `md1`
/// cards, each chunked independently. Lumping all six `mk1` records into one
/// group gives `received 6 chunks, header declares total_chunks = 2` — so HRP
/// grouping rejects **every multisig wallet**, which is the shape §6.4's
/// "why not 7" section exists to admit. Vectors D and E carry one card per HRP
/// and F is `pub_len = 0`, so nothing but vector G catches this.
///
/// A record that is NOT chunked is its own card and takes the single-string
/// path; neither path handles both forms (§6.3).
pub fn decode_public_set(records: &[&str]) -> Result<(), RecordError> {
    use std::collections::BTreeMap;
    // key: (hrp, chunk_set_id) — `None` csid means a non-chunked record, which
    // is its own card, so it gets a unique key via its index.
    let mut groups: BTreeMap<(char, Option<u32>, usize), Vec<&str>> = BTreeMap::new();
    for (i, r) in records.iter().enumerate() {
        let kind = validate_record(r)?;
        if kind.is_secret() {
            // Guarded by the caller (§6.3 forbids a secret in the public
            // section); reaching here is a caller bug, not a bad payload.
            return Err(RecordError::UndecodableSet(
                "a secret record cannot be in the public section".into(),
            ));
        }
        let (hrp, csid) = chunk_key(r, kind)?;
        let uniq = if csid.is_some() { 0 } else { i + 1 };
        groups.entry((hrp, csid, uniq)).or_default().push(r);
    }
    for ((hrp, csid, _), set) in groups {
        // Stringify INSIDE each arm: md_codec::Error and mk_codec::Error are
        // different types, so a match unifying them is E0308.
        let res: Result<(), String> = match (hrp, csid) {
            ('d', Some(_)) => md_codec::reassemble(&set)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            ('d', None) => md_codec::decode_md1_string(set[0])
                .map(|_| ())
                .map_err(|e| e.to_string()),
            ('k', _) => mk_codec::decode(&set)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            _ => unreachable!("secret records are refused above"),
        };
        res.map_err(|e| RecordError::UndecodableSet(format!("{hrp}-card: {e}")))?;
    }
    Ok(())
}

/// Card identity: the HRP discriminant plus the 20-bit chunk-set id, or `None`
/// when the record is not chunked (and is therefore its own card).
///
/// **Verified against the real crates on vector G's records** — both paths are
/// public and both distinguish chunked from single-string, which is also what
/// §6.3's non-chunked dispatch needs.
fn chunk_key(s: &str, kind: RecordKind) -> Result<(char, Option<u32>), RecordError> {
    use md_codec::bitstream::BitReader;
    use md_codec::ChunkHeader;
    use mk_codec::string_layer::{decode_string, StringLayerHeader};

    match kind {
        RecordKind::Md => {
            let (bytes, _bits) = md_codec::codex32::unwrap_string(s)
                .map_err(|e| RecordError::Invalid(e.to_string()))?;
            let mut r = BitReader::new(&bytes);
            // A non-chunked md1 fails the chunked-flag read; that is the signal,
            // not an error.
            Ok(('d', ChunkHeader::read(&mut r).ok().map(|h| h.chunk_set_id)))
        }
        RecordKind::Mk => {
            let d = decode_string(s).map_err(|e| RecordError::Invalid(e.to_string()))?;
            let (h, _) = StringLayerHeader::from_5bit_symbols(d.data())
                .map_err(|e| RecordError::Invalid(e.to_string()))?;
            Ok((
                'k',
                match h {
                    StringLayerHeader::Chunked { chunk_set_id, .. } => Some(chunk_set_id),
                    StringLayerHeader::SingleString { .. } => None,
                    // StringLayerHeader is #[non_exhaustive], so this arm is
                    // MANDATORY — it will not compile without it. Fail closed: an
                    // unrecognised header variant on a security path must never be
                    // silently grouped with anything.
                    _ => {
                        return Err(RecordError::UndecodableSet(
                            "unrecognised mk1 string-layer header variant".into(),
                        ))
                    }
                },
            ))
        }
        RecordKind::Ms => unreachable!("secret records are refused by the caller"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD1: [&str; 3] = [
        "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3",
        "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374",
        "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz",
    ];
    const MK1: [&str; 2] = [
        "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g",
        "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8",
    ];
    const MS1: &str = "ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw";

    #[test]
    fn classifies_each_record_kind() {
        assert_eq!(validate_record(MD1[0]).unwrap(), RecordKind::Md);
        assert_eq!(validate_record(MK1[0]).unwrap(), RecordKind::Mk);
        assert_eq!(validate_record(MS1).unwrap(), RecordKind::Ms);
    }

    /// THE round-3 Critical. `mnemonic bundle` prints --group-size 5 by default;
    /// codex32's inputChar has no mapping for 0x20, so the device classifies a
    /// spaced record as unknown. Refuse here — never strip, or the plate carries
    /// separators the BCH checksum never covered.
    #[test]
    fn refuses_space_grouped_and_hyphenated_records() {
        assert!(matches!(
            validate_record("md1fv9w jpqpqpm6"),
            Err(RecordError::NonCanonical { ch: ' ', .. })
        ));
        assert!(matches!(
            validate_record("md1fv9w-jpqpqpm6"),
            Err(RecordError::NonCanonical { ch: '-', .. })
        ));
    }

    /// §6.4: uppercase passes the BCH validators, so without this the same
    /// wallet has two spec-legal encodings and therefore two §6.6 hashes.
    #[test]
    fn refuses_uppercase_records() {
        assert!(matches!(
            validate_record(&MD1[0].to_uppercase()),
            Err(RecordError::NotLowercase(_))
        ));
    }

    #[test]
    fn refuses_corrupt_and_unknown_records() {
        let mut bad = MD1[0].to_string();
        let last = bad.pop().unwrap();
        bad.push(if last == 'q' { 'p' } else { 'q' });
        assert!(validate_record(&bad).is_err());
        assert!(validate_record("xx1qqqq").is_err());
    }

    /// §6.3: DECODE is per CARD SET, not per record. Records are CHUNKS —
    /// verified against the real crates:
    ///   md1 single chunk → "chunk set incomplete: got 1 chunks, expected 3"
    ///   mk1 single chunk → "received 1 chunks, header declares total_chunks = 2"
    /// A per-record decode would reject every legitimate payload.
    #[test]
    fn decodes_a_complete_card_set() {
        let all: Vec<&str> = MD1.iter().chain(MK1.iter()).copied().collect();
        assert!(
            decode_public_set(&all).is_ok(),
            "the full md1+mk1 set must decode"
        );
    }

    #[test]
    fn refuses_an_incomplete_card_set() {
        assert!(
            decode_public_set(&[MD1[0]]).is_err(),
            "one md1 chunk of three"
        );
        assert!(
            decode_public_set(&[MK1[0]]).is_err(),
            "one mk1 chunk of two"
        );
        assert!(
            decode_public_set(&MD1[..2]).is_err(),
            "two md1 chunks of three"
        );
    }

    /// The §6.3 smuggling case: arbitrary bytes wrapped in a BCH-valid md1.
    /// `ValidMD` passes it; the decode must not.
    #[test]
    fn refuses_a_bch_valid_but_undecodable_record() {
        const SMUGGLED: &str =
            "md1qqqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0sdmjzeptm5fdk0";
        assert!(
            validate_record(SMUGGLED).is_ok(),
            "BCH layer accepts it — that is the point"
        );
        assert!(
            decode_public_set(&[SMUGGLED]).is_err(),
            "decode must reject it"
        );
    }
}
