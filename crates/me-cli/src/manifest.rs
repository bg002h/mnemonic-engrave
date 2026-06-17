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
    /// Path to the rendered preview image (Phase B), set by `me bundle --preview`.
    /// `None` (the default / Phase A) omits the field entirely.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
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

impl Manifest {
    /// A human-readable, one-line-per-plate checklist for stderr.
    pub fn checklist(&self) -> String {
        let mut out = format!(
            "me: backup needs {} plates ({} public + ms1 on device):\n",
            self.wallet_plates,
            self.wallet_plates.saturating_sub(1)
        );
        for p in &self.plates {
            let label = match p.kind {
                PlateKind::Md1 => "md1 policy".to_string(),
                PlateKind::Mk1Chunk => {
                    // chunk_index is 0-based; total comes from the matching set.
                    let total = self
                        .sets
                        .iter()
                        .find(|s| Some(&s.chunk_set_id) == p.chunk_set_id.as_ref())
                        .map(|s| s.total)
                        .unwrap_or(0);
                    let idx = p.chunk_index.map(|i| i + 1).unwrap_or(0);
                    format!("mk1 chunk {idx}/{total}")
                }
                PlateKind::Ms1 => "ms1 secret".to_string(),
            };
            let action = match p.kind {
                PlateKind::Ms1 => {
                    "TYPE ON DEVICE (New > Input Seed > CODEX32); never via this tool".to_string()
                }
                _ => "push via NFC & engrave".to_string(),
            };
            out.push_str(&format!(
                "  plate {}/{}  {label}  → {action}\n",
                p.plate, p.of
            ));
        }
        out
    }
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
            preview: None,
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
    fn preview_some_serializes_key_none_omits() {
        let with = PlateEntry {
            plate: 1,
            of: 2,
            kind: PlateKind::Md1,
            string: Some("md1xyz".into()),
            chunk_set_id: None,
            chunk_index: None,
            integrity: Integrity::BchOnly,
            preview: Some("out/plate-1.svg".into()),
        };
        let j = serde_json::to_value(&with).unwrap();
        assert_eq!(j["preview"], "out/plate-1.svg");

        let without = PlateEntry {
            preview: None,
            ..with
        };
        let j = serde_json::to_value(&without).unwrap();
        assert!(
            j.get("preview").is_none(),
            "None preview must omit the field (Phase A golden unaffected)"
        );
    }

    #[test]
    fn serializes_enum_renames() {
        assert_eq!(serde_json::to_value(Kind::Mk1).unwrap(), "mk1");
        assert_eq!(
            serde_json::to_value(PlateKind::Mk1Chunk).unwrap(),
            "mk1-chunk"
        );
        assert_eq!(
            serde_json::to_value(Integrity::SetVerified).unwrap(),
            "set-verified"
        );
        assert_eq!(serde_json::to_value(Integrity::Na).unwrap(), "n/a");
    }

    #[test]
    fn checklist_lists_public_plates_and_ms1_reminder() {
        let m = Manifest {
            tool: "me",
            version: "x.y.z",
            wallet_plates: 3,
            ms1_required: true,
            sets: vec![SetEntry {
                kind: Kind::Mk1,
                chunk_set_id: "0x12345".into(),
                total: 2,
                integrity: Integrity::SetVerified,
            }],
            plates: vec![
                PlateEntry {
                    plate: 1,
                    of: 3,
                    kind: PlateKind::Mk1Chunk,
                    string: Some("mk1a".into()),
                    chunk_set_id: Some("0x12345".into()),
                    chunk_index: Some(0),
                    integrity: Integrity::SetVerified,
                    preview: None,
                },
                PlateEntry {
                    plate: 2,
                    of: 3,
                    kind: PlateKind::Mk1Chunk,
                    string: Some("mk1b".into()),
                    chunk_set_id: Some("0x12345".into()),
                    chunk_index: Some(1),
                    integrity: Integrity::SetVerified,
                    preview: None,
                },
                PlateEntry {
                    plate: 3,
                    of: 3,
                    kind: PlateKind::Ms1,
                    string: None,
                    chunk_set_id: None,
                    chunk_index: None,
                    integrity: Integrity::Na,
                    preview: None,
                },
            ],
        };
        let c = m.checklist();
        assert!(c.contains("3 plates"), "{c}");
        assert!(c.contains("plate 1/3"), "{c}");
        assert!(c.contains("mk1 chunk 1/2"), "{c}");
        assert!(c.contains("plate 3/3"), "{c}");
        assert!(c.contains("TYPE ON DEVICE"), "{c}");
        assert!(c.contains("CODEX32"), "{c}");
    }
}
