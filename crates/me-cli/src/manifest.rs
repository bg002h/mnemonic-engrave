//! The `me bundle` output model: a manifest of the plates a wallet backup needs,
//! plus a human-readable checklist. Pure data + serde; no I/O.

use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Md1,
    Mk1,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum PlateKind {
    #[serde(rename = "md1")]
    Md1,
    #[serde(rename = "mk1-chunk")]
    Mk1Chunk,
    #[serde(rename = "ms1")]
    Ms1,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum Integrity {
    #[serde(rename = "set-verified")]
    SetVerified,
    #[serde(rename = "bch-only")]
    BchOnly,
    #[serde(rename = "n/a")]
    Na,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SetEntry {
    pub kind: Kind,
    pub chunk_set_id: String,
    pub total: u8,
    pub integrity: Integrity,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PlateEntry {
    pub plate: usize,
    pub of: usize,
    pub kind: PlateKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_set_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_index: Option<u8>,
    pub integrity: Integrity,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Manifest {
    pub tool: &'static str,
    pub version: &'static str,
    pub wallet_plates: usize,
    pub ms1_required: bool,
    pub sets: Vec<SetEntry>,
    pub plates: Vec<PlateEntry>,
}

/// Render a 20-bit chunk_set_id as the canonical `0x%05x` string.
pub fn fmt_chunk_set_id(id: u32) -> String {
    format!("0x{id:05x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_unchunked_md1_plate_without_chunk_fields() {
        let p = PlateEntry {
            plate: 1,
            of: 2,
            kind: PlateKind::Md1,
            string: Some("md1xyz".into()),
            chunk_set_id: None,
            chunk_index: None,
            integrity: Integrity::BchOnly,
        };
        let j = serde_json::to_value(&p).unwrap();
        assert_eq!(j["kind"], "md1");
        assert_eq!(j["integrity"], "bch-only");
        assert!(
            j.get("chunk_set_id").is_none(),
            "unchunked md1 must omit chunk_set_id"
        );
        assert!(j.get("chunk_index").is_none());
    }

    #[test]
    fn serializes_enum_renames() {
        assert_eq!(serde_json::to_value(Kind::Mk1).unwrap(), "mk1");
        assert_eq!(serde_json::to_value(PlateKind::Mk1Chunk).unwrap(), "mk1-chunk");
        assert_eq!(serde_json::to_value(Integrity::SetVerified).unwrap(), "set-verified");
        assert_eq!(serde_json::to_value(Integrity::Na).unwrap(), "n/a");
    }
}
