//! `mr1` records in the systemwide container: the string rule (S1 §14b), the
//! chunk-set rule (§14c) and the kit rule (§14d).
//!
//! Rust is primary; the fork's Go port converges on this module and on
//! `design/vectors/sysw_mr_v1_vectors.json`.
//!
//! The container implements the string and framing rules itself and takes no
//! `mr-codec` dependency: they are a BCH residue (md-codec's public polymod,
//! as mr itself runs it), a 37-bit header and one SHA-256 (§14a, F-699). It
//! does **not** decode a reassembled card (§14c, STOP-1).

use sha2::{Digest, Sha256};

use super::record::{mdmk_cards, Class};

/// The bech32 alphabet, in value order.
const ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// mr1's target residue (mnemonic-refugium `mr-codec` `MR_REGULAR_CONST`).
const MR_REGULAR_CONST: u128 = 0x085d21404aee15059;

/// Domain prefix of the chunk-set id hash (§14c rule 6).
const CHUNKS_DOMAIN: &[u8] = b"mr1/chunks";

/// A slice carries at most this many bytes before a card needs another chunk
/// (§14c rule 5: count = max(2, ceil(len / 45))).
const SLICE_TARGET: usize = 45;

/// The §14b reasons, in check order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MrReason {
    Case,
    Character,
    Length,
    Checksum,
    ReservedSymbol,
    ChunkShort,
    ChunkPadding,
}

impl MrReason {
    /// The reason's spec name.
    pub fn name(self) -> &'static str {
        match self {
            MrReason::Case => "MR_CASE",
            MrReason::Character => "MR_CHARACTER",
            MrReason::Length => "MR_LENGTH",
            MrReason::Checksum => "MR_CHECKSUM",
            MrReason::ReservedSymbol => "MR_RESERVED_SYMBOL",
            MrReason::ChunkShort => "MR_CHUNK_SHORT",
            MrReason::ChunkPadding => "MR_CHUNK_PADDING",
        }
    }
}

/// An mr1 string that passed §14b: a single-string card (`chunk` is `None`) or
/// one chunk of a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MrString {
    pub chunk: Option<Chunk>,
}

/// A chunk's 37-bit header and its slice (§14b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub csid: u32,
    /// The card's chunk count (the header carries count - 1).
    pub count: u8,
    pub index: u8,
    pub slice: Vec<u8>,
}

/// §14b candidate: the trimmed record begins `mr1` in any case.
pub fn is_candidate(record: &str) -> bool {
    record
        .trim()
        .as_bytes()
        .get(..3)
        .is_some_and(|p| p.eq_ignore_ascii_case(b"mr1"))
}

/// §14b: the checks run in the spec's order and the first failure is the
/// reason. `record` must be a candidate ([`is_candidate`]); anything else is
/// `Err(Character)`, the answer for a record whose first character after the
/// prefix test does not belong.
pub fn check_string(record: &str) -> Result<MrString, MrReason> {
    let t = record.trim();
    if !is_candidate(t) {
        return Err(MrReason::Character);
    }
    // 1. MR_CASE: any ASCII capital, in the whole record.
    if t.bytes().any(|b| b.is_ascii_uppercase()) {
        return Err(MrReason::Case);
    }
    // 2. MR_CHARACTER: every character after `mr1` is in the alphabet.
    let mut syms: Vec<u8> = Vec::with_capacity(t.len());
    for c in t[3..].chars() {
        match u8::try_from(c)
            .ok()
            .and_then(|b| ALPHABET.iter().position(|&a| a == b))
        {
            Some(v) => syms.push(v as u8),
            None => return Err(MrReason::Character),
        }
    }
    // 3. MR_LENGTH: L = the characters after `mr1`.
    let l = syms.len();
    if !(14..=93).contains(&l) {
        return Err(MrReason::Length);
    }
    // 4. MR_CHECKSUM: exact validity, no correction.
    let mut input = md_codec::bch::hrp_expand("mr");
    input.extend_from_slice(&syms);
    if md_codec::bch::polymod_run(&input) != MR_REGULAR_CONST {
        return Err(MrReason::Checksum);
    }
    // 5. MR_RESERVED_SYMBOL: card version 7.
    let first = syms[0];
    if matches!(ALPHABET[first as usize], b'u' | b'a' | b'7') {
        return Err(MrReason::ReservedSymbol);
    }
    // A first symbol that is not `l` is a single-string card.
    if ALPHABET[first as usize] != b'l' {
        return Ok(MrString { chunk: None });
    }
    // 6. MR_CHUNK_SHORT: fewer than 8 symbols before the 13-symbol checksum.
    let data = &syms[..l - 13];
    if data.len() < 8 {
        return Err(MrReason::ChunkShort);
    }
    // The bits, MSB first.
    let bits: Vec<u8> = data
        .iter()
        .flat_map(|&v| (0..5).rev().map(move |k| (v >> k) & 1))
        .collect();
    let take = |from: usize, n: usize| -> u32 {
        bits[from..from + n]
            .iter()
            .fold(0u32, |a, &b| (a << 1) | u32::from(b))
    };
    // 7. MR_CHUNK_PADDING: after the 37-bit header, whole bytes then 0 to 4
    //    zero bits.
    let rest = bits.len() - 37;
    let (nbytes, pad) = (rest / 8, rest % 8);
    if pad > 4 || bits[37 + nbytes * 8..].iter().any(|&b| b != 0) {
        return Err(MrReason::ChunkPadding);
    }
    let slice = (0..nbytes).map(|k| take(37 + k * 8, 8) as u8).collect();
    Ok(MrString {
        chunk: Some(Chunk {
            csid: take(5, 20),
            count: (take(25, 6) + 1) as u8,
            index: take(31, 6) as u8,
            slice,
        }),
    })
}

/// Every reason of §14b to §14d, by its spec name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Mr(MrReason),
    SetCount,
    SetIndex,
    SetDuplicate,
    SetMissing,
    SetSlices,
    SetId,
    CardSplit,
    KitOtherRecord,
    KitUnconfirmedMdmk,
}

impl Reason {
    pub fn name(self) -> &'static str {
        match self {
            Reason::Mr(r) => r.name(),
            Reason::SetCount => "SET_COUNT",
            Reason::SetIndex => "SET_INDEX",
            Reason::SetDuplicate => "SET_DUPLICATE",
            Reason::SetMissing => "SET_MISSING",
            Reason::SetSlices => "SET_SLICES",
            Reason::SetId => "SET_ID",
            Reason::CardSplit => "CARD_SPLIT",
            Reason::KitOtherRecord => "KIT_OTHER_RECORD",
            Reason::KitUnconfirmedMdmk => "KIT_UNCONFIRMED_MDMK",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardKind {
    Md1,
    Mk1,
    Mr1,
}

impl CardKind {
    pub fn name(self) -> &'static str {
        match self {
            CardKind::Md1 => "md1",
            CardKind::Mk1 => "mk1",
            CardKind::Mr1 => "mr1",
        }
    }
}

/// What a sentence names. Record numbers are positions in the list as given,
/// from 0; `begins` is the first 8 characters of a set's or card's first record
/// in payload order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Detail {
    /// A single record (the §14b reasons in a kit, `KIT_UNCONFIRMED_MDMK`).
    Record(usize),
    Count {
        begins: String,
        record: usize,
        count: u8,
        first_count: u8,
    },
    Index {
        begins: String,
        record: usize,
        index: u8,
        count: u8,
    },
    Duplicate {
        begins: String,
        index: u8,
        records: [usize; 2],
    },
    Missing {
        begins: String,
        index: u8,
        count: u8,
    },
    /// `SET_SLICES` and `SET_ID`.
    Begins(String),
    Split {
        kind: CardKind,
        begins: String,
        record: usize,
    },
    Other {
        record: usize,
        class: Class,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub reason: Reason,
    pub detail: Detail,
}

/// The records in payload order: the public section, then the secret section,
/// each in the order given (§14h). Entries are positions in `records`.
fn payload_order(records: &[String]) -> Vec<usize> {
    let secret: Vec<bool> = records
        .iter()
        .map(|r| super::classify(r).is_secret())
        .collect();
    let mut order: Vec<usize> = (0..records.len()).filter(|&i| !secret[i]).collect();
    order.extend((0..records.len()).filter(|&i| secret[i]));
    order
}

/// The first 8 characters of a record, as the spec's `begins` reads.
fn begins(record: &str) -> String {
    record.trim().chars().take(8).collect()
}

/// One chunk of the payload, with where it sits.
struct Placed {
    /// Position in the list as given.
    given: usize,
    /// Position in payload order.
    pos: usize,
    chunk: Chunk,
}

/// §14c over every `ClassMr` chunk of the payload; the first set that breaks a
/// rule is refused with that rule. A candidate that fails §14b is not a
/// `ClassMr` chunk and is not read here (the kit rule and `admit_check` refuse
/// it first).
pub fn check_sets(records: &[String]) -> Result<(), Refusal> {
    sets(records).map(|_| ())
}

/// The sets of a payload, each as the placed chunks in payload order, or the
/// first refusal.
fn sets(records: &[String]) -> Result<Vec<Vec<Placed>>, Refusal> {
    let order = payload_order(records);
    let mut sets: Vec<Vec<Placed>> = Vec::new();
    for (pos, &given) in order.iter().enumerate() {
        let Ok(MrString { chunk: Some(chunk) }) = check_string(&records[given]) else {
            continue;
        };
        // The LATEST set with this chunk-set id; it is joined only while it
        // holds fewer chunks than its first chunk's count.
        let joins = sets
            .iter()
            .rposition(|s| s[0].chunk.csid == chunk.csid)
            .filter(|&k| sets[k].len() < usize::from(sets[k][0].chunk.count));
        let placed = Placed { given, pos, chunk };
        match joins {
            Some(k) => sets[k].push(placed),
            None => sets.push(vec![placed]),
        }
    }
    for set in &sets {
        check_set(records, &order, set)?;
    }
    Ok(sets)
}

fn check_set(records: &[String], order: &[usize], set: &[Placed]) -> Result<(), Refusal> {
    let first = &set[0];
    let b = begins(&records[first.given]);
    let count = first.chunk.count;
    let refuse = |reason, detail| Err(Refusal { reason, detail });
    // 1. SET_COUNT
    if let Some(p) = set.iter().find(|p| p.chunk.count != count) {
        return refuse(
            Reason::SetCount,
            Detail::Count {
                begins: b,
                record: p.given,
                count: p.chunk.count,
                first_count: count,
            },
        );
    }
    // 2. SET_INDEX
    if let Some(p) = set.iter().find(|p| p.chunk.index >= count) {
        return refuse(
            Reason::SetIndex,
            Detail::Index {
                begins: b,
                record: p.given,
                index: p.chunk.index,
                count,
            },
        );
    }
    // 3. SET_DUPLICATE: the first repeated index in payload order and the
    //    first two records that carry it.
    let mut seen: Vec<Option<usize>> = vec![None; usize::from(count)];
    for p in set {
        match seen[usize::from(p.chunk.index)] {
            Some(earlier) => {
                return refuse(
                    Reason::SetDuplicate,
                    Detail::Duplicate {
                        begins: b,
                        index: p.chunk.index,
                        records: [earlier, p.given],
                    },
                )
            }
            None => seen[usize::from(p.chunk.index)] = Some(p.given),
        }
    }
    // 4. SET_MISSING: the lowest absent index.
    if let Some(i) = seen.iter().position(Option::is_none) {
        return refuse(
            Reason::SetMissing,
            Detail::Missing {
                begins: b,
                index: i as u8,
                count,
            },
        );
    }
    // The slices in index order.
    let mut by_index: Vec<&Placed> = set.iter().collect();
    by_index.sort_by_key(|p| p.chunk.index);
    let slices: Vec<&[u8]> = by_index.iter().map(|p| p.chunk.slice.as_slice()).collect();
    let len: usize = slices.iter().map(|s| s.len()).sum();
    let n = usize::from(count);
    // 5. SET_SLICES
    let per = len.div_ceil(n);
    let slices_ok = len >= 2
        && n == len.div_ceil(SLICE_TARGET).max(2)
        && slices[..n - 1].iter().all(|s| s.len() == per);
    if !slices_ok {
        return refuse(Reason::SetSlices, Detail::Begins(b));
    }
    // 6. SET_ID
    let mut h = Sha256::new();
    h.update(CHUNKS_DOMAIN);
    for s in &slices {
        h.update(s);
    }
    let d = h.finalize();
    let top20 = (u32::from(d[0]) << 12) | (u32::from(d[1]) << 4) | (u32::from(d[2]) >> 4);
    if top20 != first.chunk.csid {
        return refuse(Reason::SetId, Detail::Begins(b));
    }
    // 7. CARD_SPLIT: a record that is not in the set between its first and
    //    last record.
    let members: Vec<usize> = set.iter().map(|p| p.pos).collect();
    let (lo, hi) = (
        *members.iter().min().unwrap(),
        *members.iter().max().unwrap(),
    );
    if let Some(pos) = (lo + 1..hi).find(|p| !members.contains(p)) {
        return refuse(
            Reason::CardSplit,
            Detail::Split {
                kind: CardKind::Mr1,
                begins: b,
                record: order[pos],
            },
        );
    }
    Ok(())
}

/// One card of an eligible kit (§14d).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub kind: CardKind,
    pub strings: usize,
    /// The first 8 characters of the index-0 string.
    pub begins: String,
    /// The last 4 characters of the highest-index string.
    pub ends: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kit {
    /// The payload holds no mr1 candidate: it offers no kit.
    None,
    Eligible(Vec<Card>),
    Refused(Refusal),
}

/// §14d. `unconfirmed` holds the as-given indices of the md1/mk1 records that
/// are not decode-confirmed (`record::mdmk_unconfirmed_why`); the caller passes
/// them so that walk runs once per invocation.
pub fn kit(records: &[String], unconfirmed: &[usize]) -> Kit {
    if !records.iter().any(|r| is_candidate(r)) {
        return Kit::None;
    }
    let order = payload_order(records);
    let refuse = |reason, detail| Kit::Refused(Refusal { reason, detail });
    // 1. the §14b reasons.
    for &i in &order {
        if is_candidate(&records[i]) {
            if let Err(r) = check_string(&records[i]) {
                return refuse(Reason::Mr(r), Detail::Record(i));
            }
        }
    }
    // 2. KIT_OTHER_RECORD.
    for &i in &order {
        let class = super::classify(&records[i]);
        if !matches!(class, Class::MdMk | Class::Mr) {
            return refuse(Reason::KitOtherRecord, Detail::Other { record: i, class });
        }
    }
    // 3. the §14c reasons.
    let mr_sets = match sets(records) {
        Ok(s) => s,
        Err(r) => return Kit::Refused(r),
    };
    // 4. KIT_UNCONFIRMED_MDMK.
    for &i in &order {
        if super::classify(&records[i]) == Class::MdMk && unconfirmed.contains(&i) {
            return refuse(Reason::KitUnconfirmedMdmk, Detail::Record(i));
        }
    }
    // 5. CARD_SPLIT for an md1 or mk1 card.
    let pos_of = |given: usize| order.iter().position(|&g| g == given).unwrap();
    let md_cards = mdmk_cards(records);
    for card in &md_cards {
        let members: Vec<usize> = card.members.iter().map(|&(i, _)| pos_of(i)).collect();
        let (lo, hi) = (
            *members.iter().min().unwrap(),
            *members.iter().max().unwrap(),
        );
        if let Some(pos) = (lo + 1..hi).find(|p| !members.contains(p)) {
            let first = order[members.iter().copied().min().unwrap()];
            return refuse(
                Reason::CardSplit,
                Detail::Split {
                    kind: if card.hrp == 'd' {
                        CardKind::Md1
                    } else {
                        CardKind::Mk1
                    },
                    begins: begins(&records[first]),
                    record: order[pos],
                },
            );
        }
    }
    // Eligible: the cards, in the order of their first record.
    let mut cards: Vec<(usize, Card)> = Vec::new();
    for set in &mr_sets {
        let mut by_index: Vec<&Placed> = set.iter().collect();
        by_index.sort_by_key(|p| p.chunk.index);
        cards.push((
            set.iter().map(|p| p.pos).min().unwrap(),
            card_of(
                CardKind::Mr1,
                records,
                by_index.iter().map(|p| p.given).collect(),
            ),
        ));
    }
    for (i, r) in records.iter().enumerate() {
        // An mr1 single-string card.
        if let Ok(MrString { chunk: None }) = check_string(r) {
            if is_candidate(r) {
                cards.push((pos_of(i), card_of(CardKind::Mr1, records, vec![i])));
            }
        }
    }
    for card in &md_cards {
        let mut by_index = card.members.clone();
        by_index.sort_by_key(|&(_, idx)| idx);
        cards.push((
            card.members.iter().map(|&(i, _)| pos_of(i)).min().unwrap(),
            card_of(
                if card.hrp == 'd' {
                    CardKind::Md1
                } else {
                    CardKind::Mk1
                },
                records,
                by_index.iter().map(|&(i, _)| i).collect(),
            ),
        ));
    }
    cards.sort_by_key(|&(pos, _)| pos);
    Kit::Eligible(cards.into_iter().map(|(_, c)| c).collect())
}

/// A card from its strings' positions in index order.
fn card_of(kind: CardKind, records: &[String], strings: Vec<usize>) -> Card {
    let last = records[*strings.last().unwrap()].trim();
    Card {
        kind,
        strings: strings.len(),
        begins: begins(&records[strings[0]]),
        ends: last.chars().skip(last.chars().count() - 4).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysw::record::{mdmk_unconfirmed_why, Class};
    use serde_json::Value;

    fn vectors() -> Value {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../design/vectors/sysw_mr_v1_vectors.json");
        serde_json::from_slice(&std::fs::read(path).expect("vector file")).expect("vector json")
    }

    fn strings(v: &Value) -> Vec<String> {
        v["records"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.as_str().unwrap().to_string())
            .collect()
    }

    fn class_of(c: Class) -> &'static str {
        match c {
            Class::Mr => "Mr",
            Class::Unknown => "Unknown",
            _ => "other",
        }
    }

    /// The vector `detail` object a refusal must equal.
    fn detail_json(d: &Detail) -> Value {
        use serde_json::json;
        match d {
            Detail::Record(r) => json!({ "record": r }),
            Detail::Count {
                begins,
                record,
                count,
                first_count,
            } => {
                json!({"begins": begins, "record": record, "count": count, "first_count": first_count})
            }
            Detail::Index {
                begins,
                record,
                index,
                count,
            } => json!({"begins": begins, "record": record, "index": index, "count": count}),
            Detail::Duplicate {
                begins,
                index,
                records,
            } => json!({"begins": begins, "index": index, "records": records}),
            Detail::Missing {
                begins,
                index,
                count,
            } => json!({"begins": begins, "index": index, "count": count}),
            Detail::Begins(b) => json!({ "begins": b }),
            Detail::Split {
                kind,
                begins,
                record,
            } => json!({"kind": kind.name(), "begins": begins, "record": record}),
            Detail::Other { record, class } => {
                json!({"record": record, "class": crate::sysw::record::Class::name(*class)})
            }
        }
    }

    /// S1 test 24: every `string_vectors` row classifies as its `class`; an
    /// `Unknown` row reports its `reason`.
    #[test]
    fn sysw_mr_string_vectors() {
        let d = vectors();
        let rows = d["string_vectors"].as_array().unwrap();
        assert_eq!(rows.len(), 13);
        for v in rows {
            let name = v["name"].as_str().unwrap();
            let record = v["record"].as_str().unwrap();
            let class = crate::sysw::classify(record);
            assert_eq!(class_of(class), v["class"].as_str().unwrap(), "{name}");
            match check_string(record) {
                Ok(_) => assert!(v["reason"].is_null(), "{name}"),
                Err(r) => assert_eq!(Some(r.name()), v["reason"].as_str(), "{name}"),
            }
        }
    }

    /// S1 test 25: every `set_vectors` row gets its verdict, reason and detail.
    #[test]
    fn sysw_mr_set_vectors() {
        let d = vectors();
        let rows = d["set_vectors"].as_array().unwrap();
        assert_eq!(rows.len(), 12);
        for v in rows {
            let name = v["name"].as_str().unwrap();
            match check_sets(&strings(v)) {
                Ok(()) => {
                    assert_eq!(v["pack"], "ok", "{name}");
                    assert!(v["reason"].is_null(), "{name}");
                }
                Err(r) => {
                    assert_eq!(v["pack"], "refused", "{name}");
                    assert_eq!(Some(r.reason.name()), v["reason"].as_str(), "{name}");
                    assert_eq!(detail_json(&r.detail), v["detail"], "{name}");
                }
            }
        }
        // Order inside a card is free; the container does not decode a card.
        for n in ["complete-reversed", "framing-only"] {
            let v = rows.iter().find(|v| v["name"] == n).unwrap();
            assert_eq!(v["pack"], "ok");
        }
    }

    /// S1 test 26: every `kit_vectors` row gets its verdict, reason, detail and
    /// cards, in order.
    #[test]
    fn sysw_mr_kit_vectors() {
        let d = vectors();
        let rows = d["kit_vectors"].as_array().unwrap();
        assert_eq!(rows.len(), 10);
        for v in rows {
            let name = v["name"].as_str().unwrap();
            let recs = strings(v);
            let unconf: Vec<usize> = mdmk_unconfirmed_why(&recs)
                .iter()
                .map(|&(i, _)| i)
                .collect();
            match kit(&recs, &unconf) {
                Kit::None => assert_eq!(v["kit"], "none", "{name}"),
                Kit::Eligible(cards) => {
                    assert_eq!(v["kit"], "eligible", "{name}");
                    let want = v["cards"].as_array().unwrap();
                    assert_eq!(cards.len(), want.len(), "{name}");
                    for (c, w) in cards.iter().zip(want) {
                        assert_eq!(c.kind.name(), w["kind"].as_str().unwrap(), "{name}");
                        assert_eq!(c.strings as u64, w["strings"].as_u64().unwrap(), "{name}");
                        assert_eq!(c.begins, w["begins"].as_str().unwrap(), "{name}");
                        assert_eq!(c.ends, w["ends"].as_str().unwrap(), "{name}");
                    }
                }
                Kit::Refused(r) => {
                    assert_eq!(v["kit"], "refused", "{name}");
                    assert_eq!(Some(r.reason.name()), v["reason"].as_str(), "{name}");
                    assert_eq!(detail_json(&r.detail), v["detail"], "{name}");
                }
            }
        }
        let two_of_three = rows.iter().find(|v| v["name"] == "kit-2of3").unwrap();
        assert_eq!(two_of_three["cards"].as_array().unwrap().len(), 6);
    }
}
