//! `me bundle` orchestration: validate a wallet backup's public strings, prove
//! each chunk set is complete/consistent, and build a Manifest. Refuses ms1.

use crate::classify::{self, ClassifyError, Format};
use crate::manifest::{
    fmt_chunk_set_id, Integrity, Kind, Manifest, PlateEntry, PlateKind, SetEntry,
};
use crate::validate::{self, ValidateError};

/// Why a bundle could not be produced. `exit_code()` maps to the CLI contract
/// (2 usage, 3 ms1 refused, 4 invalid/integrity), consistent with the converter.
#[derive(Debug)]
pub enum BundleError {
    /// No input strings at all.
    Empty,
    /// An `ms1` line was present — refused before any further processing.
    RefusedSecret,
    /// A line could not be classified by HRP.
    Classify(String, ClassifyError),
    /// A line failed per-string pristine validation.
    Validate(String, ValidateError),
    /// An mk1 string carries a `SingleString` header (no chunk_set_id) —
    /// unsupported for bundle (only synthetic ≤56-byte cards hit this).
    Mk1SingleString(String),
    /// An md1 string has an unsupported wire version.
    Md1WireVersion(String),
    /// An md1 chunk header could not be read for another reason.
    Md1HeaderRead(String, md_codec::Error),
    /// An mk1 chunk set failed reassembly/integrity.
    SetIncompleteMk(String, mk_codec::Error),
    /// An md1 chunk set failed reassembly/integrity.
    SetIncompleteMd(String, md_codec::Error),
}

impl BundleError {
    pub fn exit_code(&self) -> i32 {
        match self {
            BundleError::Empty => 2,
            BundleError::RefusedSecret => 3,
            _ => 4,
        }
    }
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::Empty => write!(f, "no input strings (expected newline-separated md1/mk1)"),
            BundleError::RefusedSecret => write!(
                f,
                "refusing to process ms1 over this tool: ms1 is secret seed entropy — \
                 enter it by hand on the device (New > Input Seed > CODEX32), never via NFC/this tool"
            ),
            BundleError::Classify(s, e) => write!(f, "cannot classify '{s}': {e}"),
            BundleError::Validate(s, e) => write!(f, "invalid string '{s}': {e}"),
            BundleError::Mk1SingleString(_) => {
                write!(f, "mk1 SingleString header: unsupported for bundle (no chunk_set_id)")
            }
            BundleError::Md1WireVersion(_) => write!(f, "unsupported md1 wire version"),
            BundleError::Md1HeaderRead(s, e) => write!(f, "cannot read md1 chunk header for '{s}': {e}"),
            BundleError::SetIncompleteMk(id, e) => {
                write!(f, "mk1 set {id} is incomplete/inconsistent: {e}")
            }
            BundleError::SetIncompleteMd(id, e) => {
                write!(f, "md1 set {id} is incomplete/inconsistent: {e}")
            }
        }
    }
}
impl std::error::Error for BundleError {}

/// One classified, pristine-validated input string, with chunk metadata extracted.
#[derive(Debug)]
pub enum Parsed {
    /// Unchunked single md1 — its own bch-only plate, no set.
    Md1Single { s: String },
    /// One chunk of a (possibly size-1) chunked md1 set.
    Md1Chunk { s: String, chunk_set_id: u32, total: u8, index: u8 },
    /// One chunk of an mk1 key card.
    Mk1Chunk { s: String, chunk_set_id: u32, total: u8, index: u8 },
}

/// Classify, refuse ms1, pristine-validate, and extract chunk metadata for one line.
pub fn parse_line(s: &str) -> Result<Parsed, BundleError> {
    let s = s.trim();
    let fmt = classify::classify(s).map_err(|e| BundleError::Classify(s.to_string(), e))?;
    if fmt == Format::Ms {
        return Err(BundleError::RefusedSecret);
    }
    // Per-string PRISTINE validation BEFORE any reassembly (reuses the converter).
    validate::validate(fmt, s).map_err(|e| BundleError::Validate(s.to_string(), e))?;

    match fmt {
        Format::Mk => {
            let decoded = mk_codec::string_layer::decode_string(s)
                .map_err(|e| BundleError::Validate(s.to_string(), ValidateError::Mk(e)))?;
            let (hdr, _) =
                mk_codec::string_layer::header::StringLayerHeader::from_5bit_symbols(decoded.data())
                    .map_err(|e| BundleError::Validate(s.to_string(), ValidateError::Mk(e)))?;
            use mk_codec::string_layer::header::StringLayerHeader as H;
            // `StringLayerHeader` is #[non_exhaustive] (mk-codec) — an external
            // crate MUST include a wildcard arm or this fails to compile (E0004).
            // The `_` arm covers SingleString and any future non-chunked variant:
            // none has a chunk_set_id to group by, so all are unsupported here.
            match hdr {
                H::Chunked { chunk_set_id, total_chunks, chunk_index, .. } => Ok(Parsed::Mk1Chunk {
                    s: s.to_string(),
                    chunk_set_id,
                    total: total_chunks,
                    index: chunk_index,
                }),
                _ => Err(BundleError::Mk1SingleString(s.to_string())),
            }
        }
        Format::Md => {
            let (bytes, bit_count) = md_codec::codex32::unwrap_string(s)
                .map_err(|e| BundleError::Validate(s.to_string(), ValidateError::Md(e)))?;
            // DEVIATION (md-codec 0.36): `ChunkHeader::read` reads its 4-bit
            // version from the *top* 4 bits of the first 5-bit symbol, whereas a
            // single (non-chunked) `Header` packs version in the *low* 4 bits with
            // divergent_paths in bit 4 — so on a pristine single md1 string
            // `ChunkHeader::read` spuriously returns `WireVersionMismatch{got:2}`
            // instead of `ChunkHeaderChunkedFlagMissing` (see SPEC §2.5 doc note in
            // md-codec chunk.rs). The canonical chunked/single discriminator md-codec
            // itself uses is bit 0 of the first 5-bit symbol (`symbols.first() & 0x01`,
            // chunk.rs `decode_with_corrections`). So check that flag first; only call
            // `ChunkHeader::read` when the chunked flag is set. Behavior is unchanged.
            let mut probe = md_codec::bitstream::BitReader::with_bit_limit(&bytes, bit_count);
            let chunked_flag = probe
                .read_bits(5)
                .map(|sym| sym & 0x01 != 0)
                .unwrap_or(false);
            if !chunked_flag {
                return Ok(Parsed::Md1Single { s: s.to_string() });
            }
            let mut r = md_codec::bitstream::BitReader::with_bit_limit(&bytes, bit_count);
            match md_codec::chunk::ChunkHeader::read(&mut r) {
                Ok(h) => Ok(Parsed::Md1Chunk {
                    s: s.to_string(),
                    chunk_set_id: h.chunk_set_id,
                    total: h.count,
                    index: h.index,
                }),
                Err(md_codec::Error::ChunkHeaderChunkedFlagMissing) => {
                    Ok(Parsed::Md1Single { s: s.to_string() })
                }
                Err(md_codec::Error::WireVersionMismatch { .. }) => {
                    Err(BundleError::Md1WireVersion(s.to_string()))
                }
                Err(e) => Err(BundleError::Md1HeaderRead(s.to_string(), e)),
            }
        }
        Format::Ms => unreachable!("ms1 refused above"),
    }
}

use std::collections::BTreeMap;

/// Validate the public strings of one or more wallet backups and build a manifest.
/// Pure: no I/O. Refuses ms1. See `design/SPEC_me_bundle_phaseA.md`.
pub fn run_bundle(input: &str) -> Result<Manifest, BundleError> {
    let raw: Vec<&str> = input
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if raw.is_empty() {
        return Err(BundleError::Empty);
    }
    // Refuse ms1 BEFORE validating ANY line (spec §4.2 / m-1): a classify-only
    // pre-scan, so no line's content is BCH-validated if an ms1 is present.
    for line in &raw {
        if classify::classify(line) == Ok(Format::Ms) {
            return Err(BundleError::RefusedSecret);
        }
    }
    let parsed: Vec<Parsed> = raw.iter().map(|l| parse_line(l)).collect::<Result<_, _>>()?;

    // Partition: unchunked md1 (each its own bch-only plate), chunked md1 groups,
    // mk1 groups — keyed by chunk_set_id. BTreeMap keeps a deterministic order.
    let mut md1_singles: Vec<String> = Vec::new();
    let mut md1_groups: BTreeMap<u32, Vec<(u8, String)>> = BTreeMap::new();
    let mut mk1_groups: BTreeMap<u32, Vec<(u8, String)>> = BTreeMap::new();
    for p in parsed {
        match p {
            Parsed::Md1Single { s } => md1_singles.push(s),
            Parsed::Md1Chunk { s, chunk_set_id, index, .. } => {
                md1_groups.entry(chunk_set_id).or_default().push((index, s));
            }
            Parsed::Mk1Chunk { s, chunk_set_id, index, .. } => {
                mk1_groups.entry(chunk_set_id).or_default().push((index, s));
            }
        }
    }

    let mut sets: Vec<SetEntry> = Vec::new();
    let mut plates: Vec<PlateEntry> = Vec::new();

    // 1) Unchunked md1 policy plates (bch-only).
    for s in &md1_singles {
        plates.push(PlateEntry {
            plate: 0, of: 0, kind: PlateKind::Md1,
            string: Some(s.clone()),
            chunk_set_id: None, chunk_index: None,
            integrity: Integrity::BchOnly,
        });
    }

    // 2) Chunked md1 sets.
    for (id, mut chunks) in md1_groups {
        chunks.sort_by_key(|(i, _)| *i);
        let refs: Vec<&str> = chunks.iter().map(|(_, s)| s.as_str()).collect();
        md_codec::chunk::reassemble(&refs)
            .map_err(|e| BundleError::SetIncompleteMd(fmt_chunk_set_id(id), e))?;
        let total = chunks.len() as u8;
        sets.push(SetEntry { kind: Kind::Md1, chunk_set_id: fmt_chunk_set_id(id), total, integrity: Integrity::SetVerified });
        for (idx, s) in &chunks {
            plates.push(PlateEntry {
                plate: 0, of: 0, kind: PlateKind::Md1,
                string: Some(s.clone()),
                chunk_set_id: Some(fmt_chunk_set_id(id)), chunk_index: Some(*idx),
                integrity: Integrity::SetVerified,
            });
        }
    }

    // 3) mk1 key-card sets.
    for (id, mut chunks) in mk1_groups {
        chunks.sort_by_key(|(i, _)| *i);
        let refs: Vec<&str> = chunks.iter().map(|(_, s)| s.as_str()).collect();
        mk_codec::decode(&refs)
            .map_err(|e| BundleError::SetIncompleteMk(fmt_chunk_set_id(id), e))?;
        let total = chunks.len() as u8;
        sets.push(SetEntry { kind: Kind::Mk1, chunk_set_id: fmt_chunk_set_id(id), total, integrity: Integrity::SetVerified });
        for (idx, s) in &chunks {
            plates.push(PlateEntry {
                plate: 0, of: 0, kind: PlateKind::Mk1Chunk,
                string: Some(s.clone()),
                chunk_set_id: Some(fmt_chunk_set_id(id)), chunk_index: Some(*idx),
                integrity: Integrity::SetVerified,
            });
        }
    }

    // 4) Trailing ms1 reminder.
    plates.push(PlateEntry {
        plate: 0, of: 0, kind: PlateKind::Ms1,
        string: None, chunk_set_id: None, chunk_index: None,
        integrity: Integrity::Na,
    });

    // Renumber plate/of now that the full ordered set is known.
    let total_plates = plates.len();
    for (i, p) in plates.iter_mut().enumerate() {
        p.plate = i + 1;
        p.of = total_plates;
    }

    Ok(Manifest {
        tool: "me",
        version: env!("CARGO_PKG_VERSION"),
        wallet_plates: total_plates,
        ms1_required: true,
        sets,
        plates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD1_UNCHUNKED: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
    // A real 2-chunk mk1 set, chunk_set_id 74565 = 0x12345 (mk-codec v0.1.json).
    const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
    const MK1_B: &str = "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";
    const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

    #[test]
    fn exit_codes_match_spec() {
        assert_eq!(BundleError::Empty.exit_code(), 2);
        assert_eq!(BundleError::RefusedSecret.exit_code(), 3);
        assert_eq!(BundleError::Mk1SingleString("mk1x".into()).exit_code(), 4);
    }

    #[test]
    fn parses_unchunked_md1_as_bch_only() {
        let p = parse_line(MD1_UNCHUNKED).unwrap();
        assert!(matches!(p, Parsed::Md1Single { .. }));
    }

    #[test]
    fn parses_mk1_chunk_with_set_id() {
        let p = parse_line(MK1_A).unwrap();
        match p {
            Parsed::Mk1Chunk { chunk_set_id, total, .. } => {
                assert_eq!(chunk_set_id, 0x12345);
                assert_eq!(total, 2);
            }
            _ => panic!("expected Mk1Chunk"),
        }
    }

    #[test]
    fn refuses_ms1_line() {
        assert!(matches!(parse_line(MS1), Err(BundleError::RefusedSecret)));
    }

    #[test]
    fn rejects_corrupted_mk1() {
        let mut bad = MK1_B.to_string();
        let last = bad.pop().unwrap();
        bad.push(if last == 'q' { 'p' } else { 'q' });
        assert!(matches!(parse_line(&bad), Err(BundleError::Validate(..))));
    }

    fn lines(v: &[&str]) -> String { v.join("\n") }

    #[test]
    fn happy_path_md1_plus_2chunk_mk1() {
        let input = lines(&[MD1_UNCHUNKED, MK1_A, MK1_B]);
        let m = run_bundle(&input).unwrap();
        // 1 md1 (bch-only) + 2 mk1 chunks + 1 ms1 reminder = 4 plates.
        assert_eq!(m.wallet_plates, 4);
        assert_eq!(m.plates.len(), 4);
        // Unchunked md1 is bch-only and NOT in sets[].
        assert!(m.sets.iter().all(|s| s.kind != Kind::Md1));
        assert_eq!(m.sets.len(), 1); // just the mk1 set
        assert_eq!(m.sets[0].chunk_set_id, "0x12345");
        assert_eq!(m.sets[0].total, 2);
        // ms1 reminder is last.
        assert!(matches!(m.plates.last().unwrap().kind, PlateKind::Ms1));
        assert!(m.ms1_required);
    }

    #[test]
    fn reordered_mk1_chunks_still_verify() {
        let input = lines(&[MK1_B, MK1_A]); // reversed
        let m = run_bundle(&input).unwrap();
        assert_eq!(m.sets[0].integrity, Integrity::SetVerified);
    }

    #[test]
    fn dropped_mk1_chunk_fails() {
        let input = lines(&[MK1_A]); // total=2, only 1 supplied
        assert!(matches!(run_bundle(&input), Err(BundleError::SetIncompleteMk(..))));
    }

    #[test]
    fn empty_input_is_usage_error() {
        assert!(matches!(run_bundle("   \n  \n"), Err(BundleError::Empty)));
    }

    #[test]
    fn ms1_anywhere_refuses_early() {
        let input = lines(&[MK1_A, MS1, MK1_B]);
        assert!(matches!(run_bundle(&input), Err(BundleError::RefusedSecret)));
    }
}
