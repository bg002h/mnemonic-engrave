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
            // A1/F1: NEVER interpolate the raw input string (`s`) here — a
            // mangled-HRP ms1 (`msx1…`) would print its intact secret body to
            // stderr. Mirror ConvertError: show only the underlying error `e`,
            // whose own text is metadata-only (HRP prefix / single offending
            // char + position / bit counts — verified against the codec sources).
            BundleError::Classify(_, e) => write!(f, "cannot classify input: {e}"),
            BundleError::Validate(_, e) => write!(f, "invalid input string: {e}"),
            BundleError::Mk1SingleString(_) => {
                write!(f, "mk1 SingleString header: unsupported for bundle (no chunk_set_id)")
            }
            BundleError::Md1WireVersion(_) => write!(f, "unsupported md1 wire version"),
            BundleError::Md1HeaderRead(_, e) => write!(f, "cannot read md1 chunk header: {e}"),
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
    Md1Chunk {
        s: String,
        chunk_set_id: u32,
        total: u8,
        index: u8,
    },
    /// One chunk of an mk1 key card.
    Mk1Chunk {
        s: String,
        chunk_set_id: u32,
        total: u8,
        index: u8,
    },
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
            let (hdr, _) = mk_codec::string_layer::header::StringLayerHeader::from_5bit_symbols(
                decoded.data(),
            )
            .map_err(|e| BundleError::Validate(s.to_string(), ValidateError::Mk(e)))?;
            use mk_codec::string_layer::header::StringLayerHeader as H;
            // `StringLayerHeader` is #[non_exhaustive] (mk-codec) — an external
            // crate MUST include a wildcard arm or this fails to compile (E0004).
            // The `_` arm covers SingleString and any future non-chunked variant:
            // none has a chunk_set_id to group by, so all are unsupported here.
            match hdr {
                H::Chunked {
                    chunk_set_id,
                    total_chunks,
                    chunk_index,
                    ..
                } => Ok(Parsed::Mk1Chunk {
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
    let parsed: Vec<Parsed> = raw
        .iter()
        .map(|l| parse_line(l))
        .collect::<Result<_, _>>()?;

    // Partition: unchunked md1 (each its own bch-only plate), chunked md1 groups,
    // mk1 groups — keyed by chunk_set_id. BTreeMap keeps a deterministic order.
    let mut md1_singles: Vec<String> = Vec::new();
    let mut md1_groups: BTreeMap<u32, Vec<(u8, String)>> = BTreeMap::new();
    let mut mk1_groups: BTreeMap<u32, Vec<(u8, String)>> = BTreeMap::new();
    for p in parsed {
        match p {
            Parsed::Md1Single { s } => md1_singles.push(s),
            Parsed::Md1Chunk {
                s,
                chunk_set_id,
                index,
                ..
            } => {
                md1_groups.entry(chunk_set_id).or_default().push((index, s));
            }
            Parsed::Mk1Chunk {
                s,
                chunk_set_id,
                index,
                ..
            } => {
                mk1_groups.entry(chunk_set_id).or_default().push((index, s));
            }
        }
    }

    let mut sets: Vec<SetEntry> = Vec::new();
    let mut plates: Vec<PlateEntry> = Vec::new();

    // 1) Unchunked md1 policy plates (bch-only).
    for s in &md1_singles {
        plates.push(PlateEntry {
            plate: 0,
            of: 0,
            kind: PlateKind::Md1,
            string: Some(s.clone()),
            chunk_set_id: None,
            chunk_index: None,
            integrity: Integrity::BchOnly,
            preview: None,
        });
    }

    // 2) Chunked md1 sets.
    for (id, mut chunks) in md1_groups {
        chunks.sort_by_key(|(i, _)| *i);
        let refs: Vec<&str> = chunks.iter().map(|(_, s)| s.as_str()).collect();
        md_codec::chunk::reassemble(&refs)
            .map_err(|e| BundleError::SetIncompleteMd(fmt_chunk_set_id(id), e))?;
        let total = chunks.len() as u8;
        sets.push(SetEntry {
            kind: Kind::Md1,
            chunk_set_id: fmt_chunk_set_id(id),
            total,
            integrity: Integrity::SetVerified,
        });
        for (idx, s) in &chunks {
            plates.push(PlateEntry {
                plate: 0,
                of: 0,
                kind: PlateKind::Md1,
                string: Some(s.clone()),
                chunk_set_id: Some(fmt_chunk_set_id(id)),
                chunk_index: Some(*idx),
                integrity: Integrity::SetVerified,
                preview: None,
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
        sets.push(SetEntry {
            kind: Kind::Mk1,
            chunk_set_id: fmt_chunk_set_id(id),
            total,
            integrity: Integrity::SetVerified,
        });
        for (idx, s) in &chunks {
            plates.push(PlateEntry {
                plate: 0,
                of: 0,
                kind: PlateKind::Mk1Chunk,
                string: Some(s.clone()),
                chunk_set_id: Some(fmt_chunk_set_id(id)),
                chunk_index: Some(*idx),
                integrity: Integrity::SetVerified,
                preview: None,
            });
        }
    }

    // 4) Trailing ms1 reminder.
    plates.push(PlateEntry {
        plate: 0,
        of: 0,
        kind: PlateKind::Ms1,
        string: None,
        chunk_set_id: None,
        chunk_index: None,
        integrity: Integrity::Na,
        preview: None,
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
    const MK1_B: &str =
        "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";
    // A second complete 2-chunk mk1 set (different chunk_set_id than MK1_A/MK1_B).
    const MK1_C: &str = "mk1qpydzkpqqsqupllwqr02m0h0qvzg3vs7zqsrqq4g4z52329g4z52329g4z52329g4z52329g4z52329g4z52329g4qpy6m8lr3sdrxkguwax";
    const MK1_D: &str =
        "mk1qpydzkppfdkdzdssxt9fh54wh8vsp2jdghv74kq2e9prxaxy2xnj2ng8vm68nf54c0vrdlfrgjzpd";
    const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

    // B8 (F1): every BundleError arm that carries the raw INPUT string must
    // redact it in Display (bounded metadata only), mirroring ConvertError. We
    // inject a marker where the raw input would go and assert Display never
    // echoes it. `SetIncompleteMk`/`SetIncompleteMd` are intentionally excluded:
    // their first field is a tool-derived `fmt_chunk_set_id(id)` (bounded hex
    // metadata), never the raw input, and Display legitimately shows it.
    #[test]
    fn no_bundle_error_display_leaks_the_input_body() {
        const CANARY: &str = "CANARY_SECRET_BODY";
        let variants: Vec<BundleError> = vec![
            BundleError::Classify(CANARY.into(), ClassifyError::UnknownHrp("zz".into())),
            BundleError::Validate(CANARY.into(), ValidateError::MkCorrected(2)),
            BundleError::Mk1SingleString(CANARY.into()),
            BundleError::Md1WireVersion(CANARY.into()),
            BundleError::Md1HeaderRead(
                CANARY.into(),
                md_codec::Error::ChunkHeaderChunkedFlagMissing,
            ),
        ];
        for e in &variants {
            let shown = format!("{e}");
            assert!(
                !shown.contains(CANARY),
                "BundleError Display leaked the input body: {shown:?}"
            );
        }
    }

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
            Parsed::Mk1Chunk {
                chunk_set_id,
                total,
                ..
            } => {
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

    fn lines(v: &[&str]) -> String {
        v.join("\n")
    }

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

    // Spec §10 #2 — one unchunked md1 + TWO distinct mk1 sets (different chunk_set_id).
    #[test]
    fn multi_set_two_distinct_mk1_cards() {
        let input = lines(&[MD1_UNCHUNKED, MK1_A, MK1_B, MK1_C, MK1_D]);
        let m = run_bundle(&input).unwrap();
        // Two mk1 sets (both set-verified), the unchunked md1 is bch-only (not a set).
        assert_eq!(m.sets.len(), 2, "expected 2 mk1 sets, got {:?}", m.sets);
        assert!(m.sets.iter().all(|s| s.kind == Kind::Mk1));
        assert!(m.sets.iter().all(|s| s.integrity == Integrity::SetVerified));
        // Distinct chunk_set_ids, one of them MK1_A/B's 0x12345.
        assert!(m.sets.iter().any(|s| s.chunk_set_id == "0x12345"));
        assert_ne!(m.sets[0].chunk_set_id, m.sets[1].chunk_set_id);
        // 1 md1 + 2 + 2 mk1 chunks + 1 ms1 reminder = 6 plates.
        assert_eq!(m.wallet_plates, 6);
        assert_eq!(m.plates.len(), 6);
        assert!(matches!(m.plates.last().unwrap().kind, PlateKind::Ms1));
    }

    // Spec §10 #6 — two mk1 chunks with MISMATCHED chunk_set_id presented together:
    // each forms an incomplete 1-of-2 group → exit 4 (distinct from the same-id
    // cross_chunk_hash case in test #7).
    #[test]
    fn foreign_mismatched_set_ids_fail() {
        // MK1_A is index 0/2 of set 0x12345; MK1_C is index 0/2 of a different set.
        let input = lines(&[MK1_A, MK1_C]);
        assert!(matches!(
            run_bundle(&input),
            Err(BundleError::SetIncompleteMk(..))
        ));
    }

    // Spec §10 #9 — pristine policy through the full run_bundle pipeline (not just parse_line).
    #[test]
    fn run_bundle_rejects_corrupted_mk1() {
        let mut bad = MK1_B.to_string();
        let last = bad.pop().unwrap();
        bad.push(if last == 'q' { 'p' } else { 'q' });
        assert!(matches!(
            run_bundle(&bad),
            Err(BundleError::Validate(
                _,
                crate::validate::ValidateError::MkCorrected(_)
            ))
        ));
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
        assert!(matches!(
            run_bundle(&input),
            Err(BundleError::SetIncompleteMk(..))
        ));
    }

    #[test]
    fn empty_input_is_usage_error() {
        assert!(matches!(run_bundle("   \n  \n"), Err(BundleError::Empty)));
    }

    #[test]
    fn ms1_anywhere_refuses_early() {
        let input = lines(&[MK1_A, MS1, MK1_B]);
        assert!(matches!(
            run_bundle(&input),
            Err(BundleError::RefusedSecret)
        ));
    }

    #[test]
    fn duplicate_mk1_chunk_index_fails() {
        // MK1_A twice (same chunk_index 0) → reassembly dup detection.
        let input = lines(&[MK1_A, MK1_A]);
        assert!(matches!(
            run_bundle(&input),
            Err(BundleError::SetIncompleteMk(..))
        ));
    }

    #[test]
    fn cross_chunk_hash_mismatch_fails() {
        // MK1_A (set 0x12345, index 0) + MK1_B2 (a DIFFERENT card's index-1 chunk
        // re-encoded at chunk_set_id 0x12345). Each chunk is individually pristine
        // but the cross-chunk hash disagrees. See plan note for vector construction.
        let mk1_b2 = foreign_index1_same_setid();
        let input = lines(&[MK1_A, &mk1_b2]);
        assert!(matches!(
            run_bundle(&input),
            Err(BundleError::SetIncompleteMk(..))
        ));
    }

    #[test]
    fn md1_chunked_set_verifies_and_drop_fails() {
        let chunks = chunked_md1_vector(); // ≥2 md1 strings of one set
        assert!(chunks.len() >= 2, "need a multi-chunk md1 vector");
        let ok = lines(&chunks.iter().map(String::as_str).collect::<Vec<_>>());
        let m = run_bundle(&ok).unwrap();
        assert!(m
            .sets
            .iter()
            .any(|s| s.kind == Kind::Md1 && s.integrity == Integrity::SetVerified));
        // Drop the last chunk → incomplete.
        let partial = lines(
            &chunks[..chunks.len() - 1]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        );
        assert!(matches!(
            run_bundle(&partial),
            Err(BundleError::SetIncompleteMd(..))
        ));
    }

    // A 2-chunk mk1 set with chunk_set_id 0x12345 where index-1 chunk is from a
    // DIFFERENT KeyCard re-encoded at the same chunk_set_id → CrossChunkHashMismatch.
    // Built via the public encode API (mirrors mk-codec's pipeline perturbation tests).
    fn foreign_index1_same_setid() -> String {
        // Decode the genuine other-card single string into a KeyCard, re-encode it at
        // 0x12345, and take its index-1 chunk. The other card must itself be ≥2 chunks.
        // Implementer: source a second multi-chunk mk1 set from mk-codec v0.1.json (the
        // second "strings" fixture) and return its index-1 string, then rewrite its
        // chunk_set_id symbols to 0x12345 via mk_codec::encode_with_chunk_set_id on the
        // decoded KeyCard. Concretely:
        let other = [
            "mk1qpydzkpqqsqupllwqr02m0h0qvzg3vs7zqsrqq4g4z52329g4z52329g4z52329g4z52329g4z52329g4z52329g4qpy6m8lr3sdrxkguwax",
            "mk1qpydzkppfdkdzdssxt9fh54wh8vsp2jdghv74kq2e9prxaxy2xnj2ng8vm68nf54c0vrdlfrgjzpd",
        ];
        let card = mk_codec::decode(&other).expect("decode other card");
        let re = mk_codec::encode_with_chunk_set_id(&card, 0x12345).expect("re-encode");
        re.into_iter().nth(1).expect("index-1 chunk")
    }

    // A multi-chunk md1 set, built from md-codec's OWN public multi-chunk descriptor
    // (verbatim from md-codec tests/bch_adversarial.rs::multi_chunk_descriptor — fully
    // public types) and `split`. Hermetic, deterministic; `split` yields ≥4 md1 chunks.
    fn chunked_md1_vector() -> Vec<String> {
        use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
        use md_codec::tag::Tag;
        use md_codec::tlv::TlvSection;
        use md_codec::tree::{Body, Node};
        use md_codec::use_site_path::UseSitePath;
        use md_codec::Descriptor;

        let paths = (0..6u32)
            .map(|c| OriginPath {
                components: (0..15u32)
                    .map(|i| PathComponent {
                        hardened: true,
                        value: c * 100 + i + 1,
                    })
                    .collect(),
            })
            .collect();
        let d = Descriptor {
            n: 6,
            path_decl: PathDecl {
                n: 6,
                paths: PathDeclPaths::Divergent(paths),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Wsh,
                body: Body::Children(vec![Node {
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: (0..6).collect(),
                    },
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        md_codec::chunk::split(&d).expect("split multi-chunk descriptor into md1 chunks")
    }
}
