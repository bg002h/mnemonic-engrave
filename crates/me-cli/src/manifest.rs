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
    /// Origin fingerprint of the mk1 card this plate belongs to, lowercase hex.
    ///
    /// `None` for non-card plates, and for privacy-preserving cards, which omit
    /// the fingerprint by design (`mk encode --privacy-preserving`). Stored as
    /// `String` rather than the `bitcoin` type deliberately: `me-cli` has no
    /// `bitcoin` dependency, and it needs none — the value is only ever
    /// rendered, and `.to_string()` needs no type name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_fingerprint: Option<String>,
    /// Origin derivation path of the mk1 card this plate belongs to, hardened
    /// components rendered with `'` to match `mk decode`'s output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_path: Option<String>,
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
    /// Hash-fragment kinds found in the decoded policy, deduplicated (F-557).
    ///
    /// The checklist's count is a COMPLETENESS CLAIM, and for a hashlock wallet
    /// the plate it omits is the one that opens the hashed path. Empty for a
    /// policy with no hashlock, so the note costs nothing where it does not
    /// apply.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hashlock_kinds: Vec<&'static str>,
    /// Key slots the policy declares (`Descriptor.n`), 0 when unknown (F-580).
    ///
    /// The seed-side number this tool CAN know. It cannot know how many of
    /// those seeds the operator holds, so it must not state a plate total that
    /// silently assumes one.
    pub key_slots: usize,
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
        // F-580: this said "backup needs {N} plates ({N-1} public + ms1 on
        // device)", where the ms1 half was a HARDCODED ONE regardless of the
        // policy. Measured: 1-, 3-, 5- and 7-cosigner policies all printed
        // "backup needs 2 plates". An operator who gathers exactly that many
        // for a 7-seed wallet is six seed backups short, and reads a count
        // presented as the requirement AS the requirement.
        //
        // `me bundle` cannot know how many of a policy's seeds are the
        // operator's -- a 2-of-3 where they hold one needs one ms1, a 7-of-7
        // they hold alone needs seven. So it no longer states a total that
        // folds them in. It states what it knows, and names what it does not.
        let public = self.wallet_plates.saturating_sub(1);
        let mut out = format!(
            "me: backup needs {public} public plate{}, plus one ms1 plate per seed you hold:\n",
            if public == 1 { "" } else { "s" }
        );
        if self.key_slots > 1 {
            out.push_str(&format!(
                "me: NOTE — this policy declares {} key slots. `me bundle` cannot know how many \
                 of those seeds are yours, so ms1 plates are NOT counted above: hold all {} and \
                 you need {} of them.\n",
                self.key_slots, self.key_slots, self.key_slots
            ));
        }
        // F-557: the count above is complete about the POLICY and silent about
        // the WALLET. A hashlock path needs its preimage -- a phrase written
        // down, or a preimage plate cut by `ms hashlock`/the device -- and that
        // is not among these plates and is not counted here. An operator who
        // gathers exactly this many plates for a keyless hashlock wallet has
        // everything except the thing that opens it.
        if !self.hashlock_kinds.is_empty() {
            out.push_str(&format!(
                "me: NOTE — this policy has a hashlock path ({}). Its preimage is NOT one of \
                 these plates and is not counted above: keep the phrase or the preimage plate \
                 apart, or the hashed path cannot be spent.\n",
                self.hashlock_kinds.join(", ")
            ));
        }
        // Which origins are shared by MORE THAN ONE CARD.
        //
        // Scanned per CARD (by chunk-set id), never per PLATE: every chunk of a
        // single card trivially carries that card's origin, so a per-plate scan
        // would call all 30 pathological plates a collision and suffix every one
        // of them — noise that hides the real case it exists to flag.
        let mut origin_sets: std::collections::HashMap<
            (Option<&str>, Option<&str>),
            std::collections::HashSet<&str>,
        > = std::collections::HashMap::new();
        for p in &self.plates {
            if !matches!(p.kind, PlateKind::Mk1Chunk) {
                continue;
            }
            if let Some(sid) = p.chunk_set_id.as_deref() {
                origin_sets
                    .entry((p.card_fingerprint.as_deref(), p.card_path.as_deref()))
                    .or_default()
                    .insert(sid);
            }
        }

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

                    // Name the card. An operator cutting 34 plates could not
                    // otherwise tell whose key is on the one in front of them.
                    let ident = match (p.card_fingerprint.as_deref(), p.card_path.as_deref()) {
                        (Some(f), Some(path)) => format!("[{f}/{path}]"),
                        // Privacy-preserving cards omit the fingerprint by
                        // design. Never fabricate one.
                        (None, Some(path)) => format!("[path {path}, no fingerprint]"),
                        (Some(f), None) => format!("[{f}, no path]"),
                        (None, None) => "[unidentified]".to_string(),
                    };

                    // Two DIFFERENT cards sharing an origin would otherwise
                    // render identically; the set id keeps them apart.
                    let ambiguous = origin_sets
                        .get(&(p.card_fingerprint.as_deref(), p.card_path.as_deref()))
                        .is_some_and(|ids| ids.len() > 1);
                    let suffix = if ambiguous {
                        format!(" set {}", p.chunk_set_id.as_deref().unwrap_or("?"))
                    } else {
                        String::new()
                    };

                    format!("mk1 {ident} chunk {idx}/{total}{suffix}")
                }
                PlateKind::Ms1 => "ms1 secret".to_string(),
            };
            let action = match p.kind {
                PlateKind::Ms1 => {
                    "TYPE ON DEVICE (New > Input Seed > CODEX32); never via this tool".to_string()
                }
                _ => "push via NFC & engrave".to_string(),
            };
            // F-580, second half: the ms1 entry is a REMINDER, not a plate in
            // this bundle, and numbering it "2/2" told a seven-seed operator
            // that one more plate completed the set. Public plates keep their
            // x/y -- that set really is closed and countable -- and the ms1
            // line carries no numbering at all.
            match p.kind {
                PlateKind::Ms1 => {
                    out.push_str(&format!("  (per seed)  {label}  → {action}\n"));
                }
                _ => out.push_str(&format!(
                    "  plate {}/{}  {label}  → {action}\n",
                    p.plate,
                    self.wallet_plates.saturating_sub(1)
                )),
            }
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
            card_fingerprint: None,
            card_path: None,
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
            card_fingerprint: None,
            card_path: None,
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
            hashlock_kinds: Vec::new(),
            wallet_plates: 3,
            key_slots: 3,
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
                    card_fingerprint: Some("aabbccdd".into()),
                    card_path: Some("48'/0'/0'/2'".into()),
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
                    card_fingerprint: Some("aabbccdd".into()),
                    card_path: Some("48'/0'/0'/2'".into()),
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
                    card_fingerprint: None,
                    card_path: None,
                    preview: None,
                },
            ],
        };
        let c = m.checklist();
        // F-580 moved these numbers, and the movement IS the fix. The three
        // plate entries are two public plates plus one ms1 reminder; the old
        // assertions read "3 plates" and "plate 3/3", which is precisely the
        // claim that told a seven-seed operator one more plate closed the set.
        // The test's purpose is unchanged: public plates are listed and
        // numbered, and the ms1 reminder is present and unmistakable.
        assert!(c.contains("2 public plates"), "{c}");
        assert!(c.contains("plate 1/2"), "{c}");
        assert!(c.contains("mk1 [aabbccdd/48'/0'/0'/2'] chunk 1/2"), "{c}");
        assert!(c.contains("(per seed)  ms1 secret"), "{c}");
        assert!(
            !c.contains("plate 3/3"),
            "the ms1 reminder is numbered again:\n{c}"
        );
        assert!(c.contains("TYPE ON DEVICE"), "{c}");
        assert!(c.contains("CODEX32"), "{c}");
    }

    /// F-557: the checklist's count is a COMPLETENESS CLAIM, and for a hashlock
    /// wallet the plate it omits is the only one that opens the hashed path.
    ///
    /// An operator who gathers exactly "8 plates" for a keyless hashlock wallet
    /// has everything except the thing that spends it. A count presented as the
    /// backup's requirement is trusted as one.
    ///
    /// MUTATION: drop the `hashlock_kinds` block from `checklist` -> the first
    /// row fails. MUTATION: make the note unconditional -> the second fails,
    /// because a wallet with no hashlock must not be told to keep a preimage it
    /// does not have.
    #[test]
    fn the_checklist_says_the_preimage_is_not_among_the_plates() {
        let m = Manifest {
            tool: "me",
            version: "x.y.z",
            hashlock_kinds: vec!["ripemd160"],
            wallet_plates: 3,
            key_slots: 3,
            ms1_required: true,
            sets: Vec::new(),
            plates: Vec::new(),
        };
        let c = m.checklist();
        assert!(
            c.contains("hashlock path") && c.contains("ripemd160"),
            "a hashlock policy's checklist does not mention the hashlock:\n{c}"
        );
        assert!(
            c.contains("not counted above") || c.contains("NOT one of"),
            "the note does not say the preimage is outside the count:\n{c}"
        );

        let plain = Manifest {
            hashlock_kinds: Vec::new(),
            ..m
        };
        let c = plain.checklist();
        assert!(
            !c.contains("hashlock path"),
            "a policy with NO hashlock is told to keep a preimage it does not have:\n{c}"
        );
        // The count itself is unchanged in both cases.
        assert!(c.contains("backup needs"), "{c}");
    }
}
