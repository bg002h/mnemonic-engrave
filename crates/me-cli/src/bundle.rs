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
    RefusedSecret {
        /// True when the ms1 is a hashlock PREIMAGE plate (`hash`, 0x03), not
        /// seed entropy. Metadata only -- read from `ms_codec::inspect`, which
        /// never yields the secret body (F-586).
        preimage: bool,
    },
    /// An `mt1` line was present. A signed transaction is not part of a wallet
    /// backup bundle; it travels via `me sysw pack` (and the device's Engrave
    /// Transaction program), where its chunk set is decode-confirmed.
    RefusedTransaction,
    /// A line could not be classified by HRP.
    Classify(String, ClassifyError),
    /// A line failed per-string pristine validation.
    Validate(String, ValidateError),
    /// An mk1 string carries a `SingleString` header (no chunk_set_id) —
    /// unsupported for bundle (only synthetic ≤56-byte cards hit this).
    Mk1SingleString(String),
    /// An md1 string has a wire version this build does not read. The `u8` is
    /// the version the header carries, so the refusal can NAME it (SPEC
    /// §8b/§6a: an older `me` meeting a newer plate must say which).
    Md1WireVersion(String, u8),
    /// An unchunked md1 plate passed BCH validation but did not decode. The
    /// plate count is a completeness claim, and it cannot be computed from a
    /// plate that does not decode (F-635), so the bundle is refused.
    Md1Undecodable(String, md_codec::Error),
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
            BundleError::RefusedSecret { .. } => 3,
            _ => 4,
        }
    }
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::Empty => write!(f, "no input strings (expected newline-separated md1/mk1)"),
            // F-586: this said "ms1 is secret seed entropy … New > Input Seed >
            // CODEX32" for EVERY ms1, including a hashlock PREIMAGE plate. It
            // is not seed entropy, CODEX32 seed entry is the wrong screen, and
            // `ms inspect` names the kind exactly ("tag: hash, prefix_byte:
            // 0x03"). A refusal that misidentifies what it refused sends the
            // operator to a screen that cannot accept what they hold.
            //
            // The refusal itself is correct either way and stays: both kinds
            // are secret, and neither belongs on this tool.
            BundleError::RefusedSecret { preimage: false } => write!(
                f,
                "refusing to process ms1 over this tool: ms1 is secret seed entropy — \
                 enter it by hand on the device (New > Input Seed > CODEX32), never via NFC/this tool"
            ),
            BundleError::RefusedSecret { preimage: true } => write!(
                f,
                "refusing to process ms1 over this tool: this is a hashlock PREIMAGE plate \
                 (`ms inspect` calls it `tag: hash`), not seed entropy — it is secret and \
                 BEARER, so whoever reads it can spend the hashed path. It is not entered \
                 through CODEX32 seed entry. Keep it apart from the policy plates; \
                 `me bundle` never carries it"
            ),
            // A1/F1: NEVER interpolate the raw input string (`s`) here — a
            // mangled-HRP ms1 (`msx1…`) would print its intact secret body to
            // stderr. Mirror ConvertError: show only the underlying error `e`.
            // Codec error text is metadata-only EXCEPT mk-codec's InvalidHrp
            // (carries an input substring), which ValidateError's own Display
            // redacts before it reaches here.
            BundleError::RefusedTransaction => write!(
                f,
                "mt1 is a signed TRANSACTION, not part of a wallet backup bundle — pack it \
                 with `me sysw pack` for the device's Engrave Transaction program"
            ),
            BundleError::Classify(_, e) => write!(f, "cannot classify input: {e}"),
            BundleError::Validate(_, e) => write!(f, "invalid input string: {e}"),
            BundleError::Mk1SingleString(_) => {
                write!(f, "mk1 SingleString header: unsupported for bundle (no chunk_set_id)")
            }
            // The accepted set is md-codec's, rendered through its own
            // Display -- a second spelling of "4, 8" here would drift the day
            // the codec accepts 12.
            BundleError::Md1WireVersion(_, got) => write!(
                f,
                "unsupported md1 wire version: {} -- this build of me cannot read \
                 this plate, so it states no plate count for it",
                md_codec::Error::WireVersionMismatch { got: *got }
            ),
            BundleError::Md1Undecodable(_, e) => write!(
                f,
                "md1 plate does not decode ({e}) -- it passes its checksum, but the \
                 plate count is computed from what the plate encodes, so no count \
                 can be stated for it"
            ),
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
        // Metadata only: `inspect` reports the prefix byte's kind and the body
        // never leaves this expression. On any error we fall back to the seed
        // wording, which is the fail-safe direction -- it tells the operator to
        // keep it off this tool either way.
        let preimage = ms_codec::inspect::inspect(s)
            .map(|r| r.kind == ms_codec::inspect::InspectKind::Preimage)
            .unwrap_or(false);
        return Err(BundleError::RefusedSecret { preimage });
    }
    if fmt == Format::Mt {
        return Err(BundleError::RefusedTransaction);
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
                Err(md_codec::Error::WireVersionMismatch { got }) => {
                    Err(BundleError::Md1WireVersion(s.to_string(), got))
                }
                Err(e) => Err(BundleError::Md1HeaderRead(s.to_string(), e)),
            }
        }
        Format::Mt => unreachable!("mt1 is refused above"),
        Format::Ms => unreachable!("ms1 refused above"),
    }
}

use std::collections::BTreeMap;

/// Validate the public strings of one or more wallet backups and build a manifest.
/// Pure: no I/O. Refuses ms1. See `design/SPEC_me_bundle_phaseA.md`.
/// Every hash-fragment kind in a decoded policy, by token.
///
/// F-557: `me bundle`'s checklist says "backup needs N plates", and an operator
/// reads a count presented as a requirement as the requirement. For a keyless
/// hashlock wallet the plate NOT in that count is the only one that opens the
/// hashed path — so the count was complete about the policy and silent about
/// the wallet.
fn descriptor_hash_kinds(d: &md_codec::Descriptor) -> Vec<&'static str> {
    use md_codec::tag::Tag;
    use md_codec::tree::{Body, Node};
    fn walk(n: &Node, out: &mut Vec<&'static str>) {
        match n.tag {
            Tag::Sha256 => out.push("sha256"),
            Tag::Hash256 => out.push("hash256"),
            Tag::Ripemd160 => out.push("ripemd160"),
            Tag::Hash160 => out.push("hash160"),
            _ => {}
        }
        // EVERY body that can hold children, not just Children -- a hashlock
        // under a thresh or a multi-family node is still a hashlock, and a walk
        // that saw one shape would be a completeness claim with a hole in it,
        // which is the defect this whole function exists to close.
        //
        // THIS MATCH IS EXHAUSTIVE ON PURPOSE -- NEVER ADD A WILDCARD ARM.
        // It had `_ => {}`, and that arm silently swallowed `Body::Tr`, whose
        // `tree` is where a taproot descriptor keeps its ENTIRE taptree. The
        // result (F-578, journey walk 2026-09-16): the same wallet warned in
        // `wsh` form and said nothing in `tr` form, with `hashlock_kinds`
        // absent from the manifest for all four kinds -- and taproot is the
        // reference wallet's primary form. A wildcard here cannot be
        // distinguished from a deliberate decision, so the compiler must be
        // the thing that forces one when a Body variant is added.
        match &n.body {
            Body::Children(kids) => kids.iter().for_each(|k| walk(k, out)),
            Body::Variable { children, .. } => children.iter().for_each(|k| walk(k, out)),
            Body::Tr { tree, .. } => {
                if let Some(t) = tree {
                    walk(t, out);
                }
            }
            // Leaves and key-only bodies: no child Node can hang off these, so
            // there is nothing further to walk. Named individually rather than
            // wildcarded, per the note above.
            Body::MultiKeys { .. }
            | Body::KeyArg { .. }
            | Body::Hash256Body(_)
            | Body::Hash160Body(_)
            | Body::Timelock(_)
            | Body::Empty => {}
        }
    }
    let mut out = Vec::new();
    walk(&d.tree, &mut out);
    out
}

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
            // F-586: same kind lookup as `parse_line`. Metadata only; the body
            // never leaves this expression.
            let preimage = ms_codec::inspect::inspect(line)
                .map(|r| r.kind == ms_codec::inspect::InspectKind::Preimage)
                .unwrap_or(false);
            return Err(BundleError::RefusedSecret { preimage });
        }
    }
    let parsed: Vec<Parsed> = raw
        .iter()
        .map(|l| parse_line(l))
        .collect::<Result<_, _>>()?;

    // Partition: unchunked md1 (each its own bch-only plate), chunked md1 groups,
    // mk1 groups — keyed by chunk_set_id.
    //
    // F-608: the map keeps the grouping; `*_order` keeps the ORDER, which is
    // first appearance in the input. Iterating the BTreeMap instead sorted sets
    // by chunk_set_id — a hash-derived number with no relation to anything the
    // operator typed — so an operator who fed three cosigner cards in the order
    // they hold them got plates back in a third order, with nothing on the page
    // saying so. Measured: input sets 0x38ea8, 0xf10a9, 0x5ac6c came back
    // 0x38ea8, 0x5ac6c, 0xf10a9. Input order is the only ordering the operator
    // can check a stack of plates against without decoding them.
    let mut md1_singles: Vec<String> = Vec::new();
    let mut md1_groups: BTreeMap<u32, Vec<(u8, String)>> = BTreeMap::new();
    let mut mk1_groups: BTreeMap<u32, Vec<(u8, String)>> = BTreeMap::new();
    let mut md1_order: Vec<u32> = Vec::new();
    let mut mk1_order: Vec<u32> = Vec::new();
    for p in parsed {
        match p {
            Parsed::Md1Single { s } => md1_singles.push(s),
            Parsed::Md1Chunk {
                s,
                chunk_set_id,
                index,
                ..
            } => {
                if !md1_groups.contains_key(&chunk_set_id) {
                    md1_order.push(chunk_set_id);
                }
                md1_groups.entry(chunk_set_id).or_default().push((index, s));
            }
            Parsed::Mk1Chunk {
                s,
                chunk_set_id,
                index,
                ..
            } => {
                if !mk1_groups.contains_key(&chunk_set_id) {
                    mk1_order.push(chunk_set_id);
                }
                mk1_groups.entry(chunk_set_id).or_default().push((index, s));
            }
        }
    }

    let mut hashlock_kinds: Vec<&'static str> = Vec::new();
    // F-580: the number of KEY SLOTS the policy declares. This is the only
    // seed-side number `me bundle` can honestly know -- it cannot know how many
    // of those seeds are the operator's, which is exactly why the old checklist
    // was wrong to state a total that folded in a single hardcoded ms1 plate.
    let mut key_slots: usize = 0;
    // F-602: true when a decoded md1 is a TEMPLATE -- no `Pubkeys` TLV, so the
    // plate carries the policy and none of the cosigner keys. Restoring from
    // that plate alone is impossible; you need each cosigner's xpub (an mk1
    // card) or every seed. `me bundle` counted neither and said "backup needs
    // 2 plates".
    let mut keyless_template = false;
    let mut sets: Vec<SetEntry> = Vec::new();
    let mut plates: Vec<PlateEntry> = Vec::new();

    // 1) Unchunked md1 policy plates (bch-only).
    for s in &md1_singles {
        // Decoded for the SAME reason as the chunked path below; a one-plate
        // wallet can carry a hashlock too, and a check that covered only the
        // chunked shape would be a completeness claim with a hole in it.
        //
        // F-635 (SPEC_liana_unspendable_internal_key §8b): this was
        // `if let Ok(d) = …`, so a plate that did not decode was SKIPPED while
        // still being pushed below -- `key_slots`, `keyless_template` and
        // `hashlock_kinds` kept their zero values and the checklist stated a
        // count computed from nothing. Measured on me 0.10.0: a version-8 2-key
        // TEMPLATE plate printed "backup needs 1 public plate" with no template
        // note. The chunked path below already refuses via `?`; this is now
        // the same rule for the unchunked shape.
        let d = md_codec::decode::decode_md1_string(s).map_err(|e| match e {
            md_codec::Error::WireVersionMismatch { got } => {
                BundleError::Md1WireVersion(s.clone(), got)
            }
            e => BundleError::Md1Undecodable(s.clone(), e),
        })?;
        hashlock_kinds.extend(descriptor_hash_kinds(&d));
        key_slots = key_slots.max(d.n as usize);
        keyless_template |= !d.is_wallet_policy();
        plates.push(PlateEntry {
            plate: 0,
            of: 0,
            kind: PlateKind::Md1,
            string: Some(s.clone()),
            chunk_set_id: None,
            chunk_index: None,
            integrity: Integrity::BchOnly,
            card_fingerprint: None,
            card_path: None,
            preview: None,
        });
    }

    // 2) Chunked md1 sets.
    for id in md1_order {
        let mut chunks = md1_groups
            .remove(&id)
            .expect("md1_order only holds ids inserted into md1_groups");
        chunks.sort_by_key(|(i, _)| *i);
        let refs: Vec<&str> = chunks.iter().map(|(_, s)| s.as_str()).collect();
        let d = md_codec::chunk::reassemble(&refs)
            .map_err(|e| BundleError::SetIncompleteMd(fmt_chunk_set_id(id), e))?;
        // F-557: the checklist makes a COMPLETENESS claim ("backup needs N
        // plates"), and for a hashlock wallet the plate it does not count is
        // the one that opens the hashed path. The reassemble above already
        // hands us the tree, so noticing costs nothing.
        hashlock_kinds.extend(descriptor_hash_kinds(&d));
        key_slots = key_slots.max(d.n as usize);
        keyless_template |= !d.is_wallet_policy();
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
                card_fingerprint: None,
                card_path: None,
                preview: None,
            });
        }
    }

    // 3) mk1 key-card sets.
    for id in mk1_order {
        let mut chunks = mk1_groups
            .remove(&id)
            .expect("mk1_order only holds ids inserted into mk1_groups");
        chunks.sort_by_key(|(i, _)| *i);
        let refs: Vec<&str> = chunks.iter().map(|(_, s)| s.as_str()).collect();
        // The decoded card used to be DISCARDED here — decoded purely to prove
        // set integrity, then dropped. Its origin is what lets the engrave
        // checklist say WHOSE key a plate carries, which an operator cutting 34
        // plates otherwise cannot tell. Since F-608 the plate ORDER is the input
        // order too, but the origin line stays load-bearing: two cards can share
        // a path, and the operator's input list is not on the plate.
        let card = mk_codec::decode(&refs)
            .map_err(|e| BundleError::SetIncompleteMk(fmt_chunk_set_id(id), e))?;

        // R2/R6 (`design/agent-reports/impl-me-cli-csid-warning.md`): the
        // card just reassembled cleanly, so recompute-and-warn on a
        // stamped/derived chunk_set_id mismatch. The mutation gate for THIS
        // surface is deleting this one line.
        crate::csid_warn::warn_chunk_set_id_mismatch(crate::csid_warn::chunk_set_id_comparison(
            &refs, &card,
        ));

        // Rendered to `String` right here, deliberately: `me-cli` has no
        // `bitcoin` dependency and needs none, because these values are only
        // ever displayed and `.to_string()` requires no type name.
        let card_fingerprint = card.origin_fingerprint.map(|f| f.to_string());
        let card_path = {
            let p = card.origin_path.to_string();
            // A card MAY carry an empty origin path; render that as absent
            // rather than as an empty bracket.
            if p.is_empty() {
                None
            } else {
                Some(p)
            }
        };

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
                card_fingerprint: card_fingerprint.clone(),
                card_path: card_path.clone(),
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
        card_fingerprint: None,
        card_path: None,
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
        hashlock_kinds: {
            hashlock_kinds.sort_unstable();
            hashlock_kinds.dedup();
            hashlock_kinds
        },
        wallet_plates: total_plates,
        key_slots,
        keyless_template,
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
            BundleError::Md1WireVersion(CANARY.into(), 12),
            BundleError::Md1Undecodable(
                CANARY.into(),
                md_codec::Error::BitStreamTruncated {
                    requested: 14,
                    available: 1,
                },
            ),
            BundleError::Md1HeaderRead(
                CANARY.into(),
                md_codec::Error::ChunkHeaderChunkedFlagMissing,
            ),
            // Codec PASS-THROUGH surface (exec-review L1/L2): mk-codec's
            // InvalidHrp carries an input substring — on the no-`1`-separator
            // branch the ENTIRE lowercased input. Unreachable via
            // classify-routed flow, but Display must redact it regardless.
            BundleError::Validate(
                CANARY.into(),
                ValidateError::Mk(mk_codec::Error::InvalidHrp(CANARY.into())),
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
        // Both kinds refuse with the same exit code -- the kind changes the
        // GUIDANCE, never the verdict (F-586).
        assert_eq!(
            BundleError::RefusedSecret { preimage: false }.exit_code(),
            3
        );
        assert_eq!(BundleError::RefusedSecret { preimage: true }.exit_code(), 3);
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
        assert!(matches!(
            parse_line(MS1),
            Err(BundleError::RefusedSecret { .. })
        ));
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
            Err(BundleError::RefusedSecret { .. })
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

    /// F-586: `me bundle` refused EVERY ms1 with "ms1 is secret seed entropy —
    /// enter it by hand on the device (New > Input Seed > CODEX32)", including
    /// a hashlock PREIMAGE plate. It is not seed entropy, CODEX32 seed entry is
    /// the wrong screen, and `ms inspect` names the kind exactly
    /// ("tag: hash, prefix_byte: 0x03"). A refusal that misidentifies what it
    /// refused sends the operator to a screen that cannot accept what they
    /// hold.
    ///
    /// The refusal itself is correct for both and does not move: both kinds are
    /// secret, neither belongs on this tool, and both exit 3.
    ///
    /// The two ms1 strings below are throwaway test values -- a preimage of
    /// `sha256("y"*32)` and the all-but-one-zero entropy card -- generated for
    /// this test and used by no wallet. A committed preimage is bearer access
    /// to whatever uses it, so these must never be reused.
    ///
    /// MUTATION: collapse the two arms back into one message -> the preimage
    /// assertions fail. MUTATION: report every ms1 as a preimage -> the seed
    /// control fails.
    #[test]
    fn a_preimage_plate_is_not_refused_as_seed_entropy() {
        const PREIMAGE_MS1: &str =
            "ms10hashsqwruttf3qhwlesecux6mnh2ddtlm06n6ldqm3e72schwypgux9u4s3kqd63fe8r49f";
        const SEED_MS1: &str =
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzr3hcpkas7yqr8";

        let pre = run_bundle(PREIMAGE_MS1).expect_err("a preimage ms1 must still be refused");
        let msg = pre.to_string();
        assert!(
            msg.contains("PREIMAGE plate"),
            "a preimage plate is not named as one:\n{msg}"
        );
        // The message DOES name CODEX32 -- to say the plate does not go there,
        // because that is where the old text sent it and where an operator
        // who read it once will go. What must not survive is the INSTRUCTION.
        assert!(
            !msg.contains("New > Input Seed > CODEX32"),
            "a preimage plate is still sent to CODEX32 seed entry, which cannot accept it:\n{msg}"
        );
        assert!(
            msg.contains("not entered through CODEX32"),
            "the preimage message does not correct the screen the old text named:\n{msg}"
        );
        assert_eq!(pre.exit_code(), 3, "the refusal verdict must not change");

        // The control: a real seed card keeps the original wording, or the fix
        // is just a reworded blanket message.
        let seed = run_bundle(SEED_MS1).expect_err("a seed ms1 must still be refused");
        let smsg = seed.to_string();
        assert!(
            smsg.contains("secret seed entropy") && smsg.contains("CODEX32"),
            "a seed card lost its (correct) guidance:\n{smsg}"
        );
        assert_eq!(seed.exit_code(), 3);

        // Neither message may carry the secret body.
        for m in [&msg, &smsg] {
            assert!(
                !m.contains("hashsqwrutt") && !m.contains("entrsqqqq"),
                "a refusal echoed the ms1 body:\n{m}"
            );
        }
    }

    /// F-602: a key-less md1 is a TEMPLATE -- it carries the policy and NONE of
    /// the cosigner keys -- and `me bundle` said "backup needs 2 plates" for
    /// one. Cut those two and the wallet is gone: restoring needs every
    /// cosigner's xpub (an mk1 card each) or every seed, and nothing told the
    /// operator to cut any of them.
    ///
    /// Given the SAME policy with the keys embedded, the tool said 32 plates
    /// for the 9-slot case -- so the first number was never a floor. It was
    /// stated as the answer.
    ///
    /// The discriminator is `Descriptor::is_wallet_policy()`: true when the
    /// `Pubkeys` TLV is present and non-empty.
    ///
    /// MUTATION: drop the `keyless_template` note from `checklist` -> the
    /// first assertion fails. MUTATION: set `keyless_template` unconditionally
    /// true -> the keyed control fails, which is the assertion that keeps this
    /// from passing on a note that always fires.
    #[test]
    fn a_keyless_template_says_the_cosigner_cards_are_missing() {
        // wsh(sortedmulti(2,@0,@1,@2)) with NO keys: one short md1.
        let keyless = "md1yzfdsssj5qqcy8pzrqrxahye32v7pju";
        // The SAME policy with three real xpubs embedded: a 6-chunk set.
        let keyed = [
            "md1fp8z0zspqsgqpsgwzyxzhs783l9ptp6y899r55wq66atyyv2fyvhqwawq8m4p9dsedhun4k2czf3z",
            "md1fp8z0zs0pnfykxj458upmne64hl86j79cw06h2xjv7es8u2f2vhk9zv73pl3xdpcrdj5crvh54t9l",
            "md1fp8z0zskjdwd4x2n6sy3kkfjh6lq3vcqq7lfnxv5qcs2tqtp2ygzuw0jfjgeqyec3mgkcmm7mck6a",
            "md1fp8z0zsuey3qgfkmwl3qp89hp8xtkm8a0h6ch67jl5xqfcd85g0se056yvr7pgkgfpt56l90dkcmu",
            "md1fp8z0z3rwujsepmc9h3mxzajul0gw9f5jcyrvxnq90svhxshepch75hcktucrmwq7znrjdx2rjtfy",
            "md1fp8z0z3vnhgymmc0924tpdkh3prfu7f47g4ntt488aal33v7xgtm7nzyqq7dn6da2eu9nf",
        ]
        .join("\n");

        let t = run_bundle(keyless).expect("key-less template should bundle");
        assert!(
            t.keyless_template,
            "a key-less md1 is not flagged as a template"
        );
        assert!(
            t.checklist()
                .contains("carries the policy and NONE of the cosigner keys"),
            "the checklist does not say the cosigner cards are missing:\n{}",
            t.checklist()
        );

        // The control: the same policy WITH keys must not carry the note, or
        // the note is unconditional and says nothing.
        let k = run_bundle(&keyed).expect("keyed policy should bundle");
        assert!(
            !k.keyless_template,
            "a wallet-policy md1 with embedded xpubs is wrongly flagged key-less"
        );
        assert!(
            !k.checklist().contains("NONE of the cosigner keys"),
            "a keyed policy is told its cosigner keys are missing:\n{}",
            k.checklist()
        );
    }

    /// F-580 (Critical): the checklist said `backup needs N plates (N-1
    /// public, ms1 on device)`, and the ms1 half was a HARDCODED ONE. Measured on
    /// the real binary before the fix: 1-, 3-, 5- and 7-cosigner policies ALL
    /// printed "backup needs 2 plates". An operator backing up a seven-seed
    /// wallet reads a count presented as the requirement and is six seed
    /// backups short, discovering it at the device with steel already cut.
    ///
    /// `me bundle` cannot know how many of a policy's seeds are the operator's,
    /// so the fix is not a better number -- it is to stop stating a total that
    /// folds in a number it cannot know, and to name what it does know
    /// (`Descriptor.n`, the declared key slots).
    ///
    /// MUTATION: restore the "{N} plates ({N-1} public + ms1 on device)" line
    /// -> the first two assertions fail. MUTATION: drop `key_slots` from the
    /// manifest (leave it 0) -> the seven-slot assertion fails. MUTATION:
    /// number the ms1 line "x/y" again -> the last assertion fails.
    #[test]
    fn the_checklist_never_states_a_seed_count_it_cannot_know() {
        let one_key = "md1yq802gggqpsg5rh7m5mygtq5w9";
        let seven_key = "md1yx802gggqpsgvzvpfewnjf3lqcza99pc";

        let m1 = run_bundle(one_key).expect("1-key policy should bundle");
        let m7 = run_bundle(seven_key).expect("7-key policy should bundle");

        // The number the tool CAN know, and the one whose absence made every
        // policy look identical.
        assert_eq!(m1.key_slots, 1, "one key slot");
        assert_eq!(m7.key_slots, 7, "seven key slots");

        let c1 = m1.checklist();
        let c7 = m7.checklist();

        // No total that silently assumes exactly one seed.
        for (n, c) in [(1, &c1), (7, &c7)] {
            assert!(
                !c.contains("ms1 on device):"),
                "{n}-key checklist still states a total folding in one ms1:\n{c}"
            );
            assert!(
                c.contains("per seed you hold"),
                "{n}-key checklist does not say the ms1 count is per seed:\n{c}"
            );
        }

        // A multi-slot policy names the count; a single-slot one has nothing
        // to warn about, which is what keeps the note from becoming noise.
        assert!(
            c7.contains("declares 7 key slots"),
            "the 7-key checklist does not name its slot count:\n{c7}"
        );
        assert!(
            !c1.contains("key slots"),
            "a single-key policy should not carry the multi-seed note:\n{c1}"
        );

        // The ms1 line is a reminder, not a numbered plate in this bundle:
        // "plate 2/2" told a seven-seed operator that one more plate closed
        // the set.
        assert!(
            c7.contains("(per seed)  ms1 secret"),
            "the ms1 reminder is still numbered like a plate:\n{c7}"
        );
    }

    /// F-578: `descriptor_hash_kinds` had NO test at all. The only thing that
    /// mentioned `hashlock_kinds` built the vector BY HAND and asserted on
    /// `checklist()` -- so it tested the renderer, and the detector behind it
    /// could return an empty vector forever without a gate failing.
    ///
    /// It did exactly that for every taproot wallet: the walk matched
    /// `Body::Children` and `Body::Variable` with a `_ => {}` arm, and a
    /// taproot descriptor keeps its whole taptree in `Body::Tr`. The same
    /// wallet warned in `wsh` form and said nothing in `tr` form -- and tr is
    /// the reference wallet's primary form.
    ///
    /// This runs the REAL path: an md1 string through `run_bundle`.
    ///
    /// MUTATION: delete the `Body::Tr` arm -> `tr` row returns `[]` and the
    /// first assert fails. MUTATION: make the walk unconditional -> the
    /// no-hashlock control fails, because a wallet with no hashlock must not
    /// claim one.
    #[test]
    fn the_hashlock_detector_sees_a_taproot_taptree_and_not_only_wsh() {
        // Same policy shape in both wrappers; only the wrapper differs.
        let tr_ripemd160 = "md1yq80tgggqps8fnvqkhz9jzkuntq3qpvf72n7avxt6us5c0ky528a6dm2ut7x25";
        let wsh_sha256 =
            "md1yq802gggqpsfxdwj6ugkg2mjdvzyq938e20m4se0tjznp7ceq0xymvpztpchjgdy3qfg6svfkr8rsj0mc";
        let tr_no_hashlock = "md1yq80tgggqps89q34sy5q79h4yrk";

        let tr = run_bundle(tr_ripemd160).expect("tr hashlock policy should bundle");
        assert_eq!(
            tr.hashlock_kinds,
            vec!["ripemd160"],
            "a taproot taptree hashlock is invisible to the detector, so the \
             completeness note never fires for the wallet form that needs it"
        );

        let wsh = run_bundle(wsh_sha256).expect("wsh hashlock policy should bundle");
        assert_eq!(wsh.hashlock_kinds, vec!["sha256"]);

        // The control that makes the assertion above falsifiable: a walk that
        // reported a hashlock unconditionally would pass both rows and be
        // caught only here.
        let plain = run_bundle(tr_no_hashlock).expect("plain tr policy should bundle");
        assert!(
            plain.hashlock_kinds.is_empty(),
            "a policy with no hashlock claims one: {:?}",
            plain.hashlock_kinds
        );

        // And the operator-visible half: the note must actually reach the
        // checklist for the tr form, which is the thing F-557 promised.
        assert!(
            tr.checklist().contains("hashlock path") && tr.checklist().contains("ripemd160"),
            "the checklist omits the hashlock note for a taproot wallet:\n{}",
            tr.checklist()
        );
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

    // ---- B6 (F17): md1 chunk-discriminator drift guards -------------------
    //
    // parse_line's md1 branch discriminates single vs chunked via bit 0 of the
    // first 5-bit symbol (`symbols.first() & 0x01`), then reads a ChunkHeader.
    // These fixtures pin every REACHABLE/OBSERVABLE parse_line arm plus the funds
    // -relevant single-chunk-of-multichunk case; a direct codec-level test pins
    // the ChunkHeaderChunkedFlagMissing arm (unreachable via parse_line, so a
    // parse_line fixture for it would be vacuous — see plan-R0 I1).

    /// A BCH-valid md1 whose first 5-bit symbol has the chunked flag SET (bit 0)
    /// but a WRONG version nibble (0, not `WF_REDESIGN_VERSION`=4). parse_line's
    /// probe therefore treats it as chunked and `ChunkHeader::read` rejects the
    /// version → `BundleError::Md1WireVersion`. Built via the bumped codec's own
    /// `wrap_payload` (hermetic; first symbol 0b00001).
    fn md1_wireversion_fixture() -> String {
        let mut payload = vec![0u8; 13];
        payload[0] = 0x08; // top-5 payload bits = 00001 → version 0, chunked 1
        md_codec::codex32::wrap_payload(&payload, 100).expect("wrap_payload")
    }

    #[test]
    fn parse_line_md1_chunk_extracts_metadata() {
        let chunks = chunked_md1_vector();
        assert!(chunks.len() >= 2, "need a multi-chunk md1 vector");
        match parse_line(&chunks[0]).unwrap() {
            Parsed::Md1Chunk { total, index, .. } => {
                assert!(total >= 2, "multi-chunk total");
                assert_eq!(index, 0, "first chunk is index 0");
            }
            other => panic!("expected Md1Chunk, got {other:?}"),
        }
    }

    #[test]
    fn single_chunk_of_multichunk_md1_is_incomplete_not_lone_plate() {
        // The funds-relevant case: one chunk of a known multi-chunk md1 must NOT
        // be accepted as a lone Md1Single plate — parse_line sees a chunk, and
        // run_bundle refuses the incomplete set.
        let chunks = chunked_md1_vector();
        assert!(chunks.len() >= 2);
        assert!(
            matches!(parse_line(&chunks[0]).unwrap(), Parsed::Md1Chunk { .. }),
            "a chunk of a multi-chunk set must not parse as Md1Single"
        );
        assert!(matches!(
            run_bundle(&chunks[0]),
            Err(BundleError::SetIncompleteMd(..))
        ));
    }

    #[test]
    fn parse_line_md1_wrong_wire_version_rejected() {
        let s = md1_wireversion_fixture();
        assert!(matches!(
            parse_line(&s),
            Err(BundleError::Md1WireVersion(_, 0))
        ));
    }

    // Direct codec-level drift guard: pin the md-codec chunk discriminator the
    // bundle probe relies on, independent of parse_line. (a) v4+chunked=1 parses;
    // (b) v4+flag-clear → ChunkHeaderChunkedFlagMissing (the arm parse_line never
    // reaches); (c) wrong-version+flag-clear → WireVersionMismatch FIRST (version
    // check precedes the flag check); (d) the chunked flag is bit 0 of the first
    // 5-bit symbol — the exact bit `parse_line` masks with `& 0x01`.
    #[test]
    fn md_codec_chunk_discriminator_behaviour_is_pinned() {
        use md_codec::bitstream::{BitReader, BitWriter};
        use md_codec::chunk::ChunkHeader;
        use md_codec::Header;

        // Full 37-bit chunk header wire: [version:4][chunked:1][set_id:20][count-1:6][index:6].
        fn wire(version: u8, chunked: u64) -> Vec<u8> {
            let mut w = BitWriter::new();
            w.write_bits(u64::from(version & 0b1111), 4);
            w.write_bits(chunked, 1);
            w.write_bits(0, 20);
            w.write_bits(0, 6);
            w.write_bits(0, 6);
            assert_eq!(w.bit_len(), 37);
            w.into_bytes()
        }

        // (a) version=4, chunked=1 → Ok.
        let ok = wire(Header::WF_REDESIGN_VERSION, 1);
        let h = ChunkHeader::read(&mut BitReader::new(&ok)).expect("v4 + chunked=1 parses");
        assert_eq!(h.version, Header::WF_REDESIGN_VERSION);

        // (b) version=4, chunked=0 → ChunkHeaderChunkedFlagMissing.
        let miss = wire(Header::WF_REDESIGN_VERSION, 0);
        assert!(matches!(
            ChunkHeader::read(&mut BitReader::new(&miss)),
            Err(md_codec::Error::ChunkHeaderChunkedFlagMissing)
        ));

        // (c) version!=4, chunked=0 → WireVersionMismatch (version precedes flag).
        let wrongver = wire(0, 0);
        assert!(matches!(
            ChunkHeader::read(&mut BitReader::new(&wrongver)),
            Err(md_codec::Error::WireVersionMismatch { got: 0 })
        ));

        // (d) chunked flag = bit 0 of the first 5-bit symbol (version=4 → 0b0100x):
        //     chunked=1 → 0b01001=9 (bit0 set); chunked=0 → 0b01000=8 (bit0 clear).
        assert_eq!(BitReader::new(&ok).read_bits(5).unwrap() & 0x01, 1);
        assert_eq!(BitReader::new(&miss).read_bits(5).unwrap() & 0x01, 0);
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
