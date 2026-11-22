//! `mr1` records in the systemwide container: the string rule (S1 §14b), the
//! chunk-set rule (§14c) and the kit rule (§14d).
//!
//! Rust is primary; the fork's Go port converges on this module and on
//! `design/vectors/sysw_mr_v1_vectors.json`.

use super::record::Class;

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
    pub fn name(self) -> &'static str {
        unimplemented!()
    }
}

/// An mr1 string that passed §14b: a single-string card (`chunk` is `None`) or
/// one chunk of a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MrString {
    pub chunk: Option<Chunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub csid: u32,
    pub count: u8,
    pub index: u8,
    pub slice: Vec<u8>,
}

/// §14b candidate: the trimmed record begins `mr1` in any case.
pub fn is_candidate(_record: &str) -> bool {
    unimplemented!()
}

/// §14b: the first failed check is the reason.
pub fn check_string(_record: &str) -> Result<MrString, MrReason> {
    unimplemented!()
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
        unimplemented!()
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
        unimplemented!()
    }
}

/// What a sentence names. Record numbers are positions in the list as given,
/// from 0.
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

/// §14c over every `ClassMr` chunk of the payload.
pub fn check_sets(_records: &[String]) -> Result<(), Refusal> {
    unimplemented!()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub kind: CardKind,
    pub strings: usize,
    pub begins: String,
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
/// are not decode-confirmed (`record::mdmk_unconfirmed_why`).
pub fn kit(_records: &[String], _unconfirmed: &[usize]) -> Kit {
    unimplemented!()
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
