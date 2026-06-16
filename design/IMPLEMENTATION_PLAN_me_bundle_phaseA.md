# `me bundle` Phase A Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a pure-Rust `me bundle` subcommand that takes the public constellation strings of a wallet backup, proves each md1/mk1 chunk set is complete and consistent, and emits a JSON manifest + a guided per-plate checklist.

**Architecture:** New `bundle.rs` (orchestration: read → classify → per-string pristine-validate → group by `chunk_set_id` → per-group set-integrity) + `manifest.rs` (serde types + checklist renderer), wired as an **optional** clap subcommand in `main.rs`. The existing single-string→NDEF converter is untouched (runs when no subcommand is given). Reuses `classify`/`validate` verbatim.

**Tech Stack:** Rust, clap (derive), `serde`/`serde_json`, `md-codec 0.36`, `mk-codec 0.4`, `zeroize`. Tests with `assert_cmd`/`predicates`.

**Spec:** `design/SPEC_me_bundle_phaseA.md` (GREEN, R0→R1). **This plan must pass the plan R0 architect gate before any code.**

---

## File Structure

- **Create** `crates/me-cli/src/manifest.rs` — `Manifest`, `SetEntry`, `PlateEntry`, `Kind`, `PlateKind`, `Integrity` (all `serde::Serialize`); `Manifest::checklist(&self) -> String`. One responsibility: the output model + its rendering.
- **Create** `crates/me-cli/src/bundle.rs` — `BundleError` (+ `exit_code()`), the per-string→group→integrity pipeline, and `run_bundle(input: &str) -> Result<Manifest, BundleError>` (pure, no I/O). One responsibility: orchestration logic.
- **Modify** `crates/me-cli/src/lib.rs` — `pub mod bundle; pub mod manifest;`.
- **Modify** `crates/me-cli/src/main.rs` — add an optional `bundle` clap subcommand; wire stdin/`--in` input and stdout/`--manifest` output; map `BundleError` → exit codes; print checklist to stderr. The existing flags stay on the top-level parser as the no-subcommand fallback.
- **Modify** `crates/me-cli/Cargo.toml` — add `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`; bump `version` to `0.2.0`.
- **Modify** `crates/me-cli/tests/cli.rs` — add `me bundle` CLI integration tests.
- **Reuse unchanged:** `classify.rs`, `validate.rs`.

---

### Task 1: Cargo deps + version bump

**Files:** Modify `crates/me-cli/Cargo.toml`

- [ ] **Step 1: Add deps and bump version**

In `[package]` set `version = "0.2.0"`. In `[dependencies]` add:
```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Verify it builds**

Run: `cargo build -p mnemonic-engrave`
Expected: compiles (no code uses the new deps yet — that's fine).

- [ ] **Step 3: Commit**
```bash
git add crates/me-cli/Cargo.toml && git commit -m "build(me): add serde/serde_json, bump to 0.2.0 for bundle"
```

---

### Task 2: Manifest types + JSON serialization

**Files:** Create `crates/me-cli/src/manifest.rs`; Modify `crates/me-cli/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/me-cli/src/manifest.rs` with this test module at the bottom:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_unchunked_md1_plate_without_chunk_fields() {
        let p = PlateEntry {
            plate: 1, of: 2, kind: PlateKind::Md1,
            string: Some("md1xyz".into()),
            chunk_set_id: None, chunk_index: None,
            integrity: Integrity::BchOnly,
        };
        let j = serde_json::to_value(&p).unwrap();
        assert_eq!(j["kind"], "md1");
        assert_eq!(j["integrity"], "bch-only");
        assert!(j.get("chunk_set_id").is_none(), "unchunked md1 must omit chunk_set_id");
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
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p mnemonic-engrave manifest`
Expected: FAIL — `manifest` module not declared / types missing.

- [ ] **Step 3: Implement the types**

Prepend to `crates/me-cli/src/manifest.rs`:
```rust
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
```

Add to `crates/me-cli/src/lib.rs` (after the existing `pub mod` lines): `pub mod bundle;` and `pub mod manifest;`. (Task 3 creates `bundle.rs`; if compiling Task 2 alone, add only `pub mod manifest;` here and add `pub mod bundle;` in Task 3.)

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p mnemonic-engrave manifest`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**
```bash
git add crates/me-cli/src/manifest.rs crates/me-cli/src/lib.rs
git commit -m "feat(me): manifest model + serde serialization for bundle"
```

---

### Task 3: Checklist renderer

**Files:** Modify `crates/me-cli/src/manifest.rs`

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `manifest.rs`:
```rust
#[test]
fn checklist_lists_public_plates_and_ms1_reminder() {
    let m = Manifest {
        tool: "me", version: "x.y.z", wallet_plates: 3, ms1_required: true,
        sets: vec![SetEntry { kind: Kind::Mk1, chunk_set_id: "0x12345".into(), total: 2, integrity: Integrity::SetVerified }],
        plates: vec![
            PlateEntry { plate: 1, of: 3, kind: PlateKind::Mk1Chunk, string: Some("mk1a".into()), chunk_set_id: Some("0x12345".into()), chunk_index: Some(0), integrity: Integrity::SetVerified },
            PlateEntry { plate: 2, of: 3, kind: PlateKind::Mk1Chunk, string: Some("mk1b".into()), chunk_set_id: Some("0x12345".into()), chunk_index: Some(1), integrity: Integrity::SetVerified },
            PlateEntry { plate: 3, of: 3, kind: PlateKind::Ms1, string: None, chunk_set_id: None, chunk_index: None, integrity: Integrity::Na },
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
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p mnemonic-engrave checklist`
Expected: FAIL — no method `checklist`.

- [ ] **Step 3: Implement**

Add to `manifest.rs` (after the structs):
```rust
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
            out.push_str(&format!("  plate {}/{}  {label}  → {action}\n", p.plate, p.of));
        }
        out
    }
}
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p mnemonic-engrave manifest`
Expected: PASS.

- [ ] **Step 5: Commit**
```bash
git add crates/me-cli/src/manifest.rs
git commit -m "feat(me): bundle checklist renderer"
```

---

### Task 4: BundleError + exit-code mapping

**Files:** Create/extend `crates/me-cli/src/bundle.rs`; Modify `lib.rs` (`pub mod bundle;` if not already)

- [ ] **Step 1: Write the failing test**

Create `crates/me-cli/src/bundle.rs`:
```rust
//! `me bundle` orchestration: validate a wallet backup's public strings, prove
//! each chunk set is complete/consistent, and build a Manifest. Refuses ms1.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_match_spec() {
        assert_eq!(BundleError::Empty.exit_code(), 2);
        assert_eq!(BundleError::RefusedSecret.exit_code(), 3);
        assert_eq!(BundleError::Mk1SingleString("mk1x".into()).exit_code(), 4);
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p mnemonic-engrave bundle`
Expected: FAIL — `BundleError` undefined.

- [ ] **Step 3: Implement the error type**

Prepend to `bundle.rs` (above the test module):
```rust
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
```

Ensure `crates/me-cli/src/lib.rs` has `pub mod bundle;`.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p mnemonic-engrave bundle`
Expected: PASS.

- [ ] **Step 5: Commit**
```bash
git add crates/me-cli/src/bundle.rs crates/me-cli/src/lib.rs
git commit -m "feat(me): BundleError + exit-code mapping"
```

---

### Task 5: Classify + ms1 refusal + pristine validation + header/grouping

**Files:** Modify `crates/me-cli/src/bundle.rs`

This task builds the internal classification & grouping helpers. We model each input string as a `Parsed` record carrying its HRP, the verbatim string, and (for chunked) its `chunk_set_id`/`chunk_index`.

- [ ] **Step 1: Write the failing tests**

Add to `bundle.rs` `tests`:
```rust
const MD1_UNCHUNKED: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
// A real 2-chunk mk1 set, chunk_set_id 74565 = 0x12345 (mk-codec v0.1.json).
const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
const MK1_B: &str = "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

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
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p mnemonic-engrave bundle`
Expected: FAIL — `parse_line` / `Parsed` undefined.

- [ ] **Step 3: Implement `Parsed` + `parse_line`**

Add to `bundle.rs` (above tests):
```rust
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
```

> **Implementer note (plan-R0-verified APIs):** `StringLayerHeader::Chunked { version, chunk_set_id, total_chunks, chunk_index }` and `is_chunked()` are public (`mk-codec/src/string_layer/header.rs`). `md_codec::bitstream::BitReader::with_bit_limit(&[u8], usize)`, `md_codec::chunk::ChunkHeader::read(&mut BitReader) -> Result<ChunkHeader, md_codec::Error>` with `pub chunk_set_id/count/index`, and the `md_codec::Error::{ChunkHeaderChunkedFlagMissing, WireVersionMismatch{got}}` variants are public. If a variant name differs at the pinned version, adjust the match arms (do not change behavior).

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p mnemonic-engrave bundle`
Expected: PASS (4 new tests + the exit-code test).

- [ ] **Step 5: Commit**
```bash
git add crates/me-cli/src/bundle.rs
git commit -m "feat(me): classify + ms1 refusal + pristine validation + chunk header extraction"
```

---

### Task 6: Grouping + per-group set-integrity + manifest assembly (`run_bundle`)

**Files:** Modify `crates/me-cli/src/bundle.rs`

- [ ] **Step 1: Write the failing tests**

Add to `bundle.rs` `tests`:
```rust
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
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p mnemonic-engrave bundle`
Expected: FAIL — `run_bundle` undefined.

- [ ] **Step 3: Implement `run_bundle`**

Add to `bundle.rs`:
```rust
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
```

> **Note on `mk_codec::decode` total vs header total:** the spec uses the header's `total_chunks` for the integrity bracket, but `reassemble`/`decode` already enforce `chunks.len() == total_chunks` internally — so a dropped chunk fails `decode` (→ `SetIncompleteMk`) before we ever build a `SetEntry`. We report `total = chunks.len()` only for the verified (complete) set, which by then equals the header total. This is correct because the `SetEntry` is built only after `decode`/`reassemble` succeeds.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p mnemonic-engrave bundle`
Expected: PASS (all bundle tests).

- [ ] **Step 5: Commit**
```bash
git add crates/me-cli/src/bundle.rs
git commit -m "feat(me): run_bundle — grouping, set-integrity, manifest assembly"
```

---

### Task 7: Additional integrity coverage (duplicate, foreign set, cross-chunk-hash, md1 chunked)

**Files:** Modify `crates/me-cli/src/bundle.rs`

- [ ] **Step 1: Write the failing tests**

Add to `bundle.rs` `tests`:
```rust
#[test]
fn duplicate_mk1_chunk_index_fails() {
    // MK1_A twice (same chunk_index 0) → reassembly dup detection.
    let input = lines(&[MK1_A, MK1_A]);
    assert!(matches!(run_bundle(&input), Err(BundleError::SetIncompleteMk(..))));
}

#[test]
fn cross_chunk_hash_mismatch_fails() {
    // MK1_A (set 0x12345, index 0) + MK1_B2 (a DIFFERENT card's index-1 chunk
    // re-encoded at chunk_set_id 0x12345). Each chunk is individually pristine
    // but the cross-chunk hash disagrees. See plan note for vector construction.
    let mk1_b2 = foreign_index1_same_setid();
    let input = lines(&[MK1_A, &mk1_b2]);
    assert!(matches!(run_bundle(&input), Err(BundleError::SetIncompleteMk(..))));
}

#[test]
fn md1_chunked_set_verifies_and_drop_fails() {
    let chunks = chunked_md1_vector(); // ≥2 md1 strings of one set
    assert!(chunks.len() >= 2, "need a multi-chunk md1 vector");
    let ok = lines(&chunks.iter().map(String::as_str).collect::<Vec<_>>());
    let m = run_bundle(&ok).unwrap();
    assert!(m.sets.iter().any(|s| s.kind == Kind::Md1 && s.integrity == Integrity::SetVerified));
    // Drop the last chunk → incomplete.
    let partial = lines(&chunks[..chunks.len() - 1].iter().map(String::as_str).collect::<Vec<_>>());
    assert!(matches!(run_bundle(&partial), Err(BundleError::SetIncompleteMd(..))));
}
```

Add these test-vector helpers inside the `tests` module:
```rust
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
                .map(|i| PathComponent { hardened: true, value: c * 100 + i + 1 })
                .collect(),
        })
        .collect();
    let d = Descriptor {
        n: 6,
        path_decl: PathDecl { n: 6, paths: PathDeclPaths::Divergent(paths) },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys { k: 2, indices: (0..6).collect() },
            }]),
        },
        tlv: TlvSection::new_empty(),
    };
    md_codec::chunk::split(&d).expect("split multi-chunk descriptor into md1 chunks")
}
```

> **Implementer note:** `foreign_index1_same_setid` uses the SECOND multi-chunk fixture from `mk-codec/src/test_vectors/v0.1.json` (strings shown above) — both chunks are individually pristine; re-encoding that card at `chunk_set_id = 0x12345` (via `mk_codec::encode_with_chunk_set_id`) and pairing its index-1 chunk with `MK1_A` (index 0, a *different* card at the same id) makes `mk_codec::decode` reject the spliced set (`CrossChunkHashMismatch`) — confirmed by the plan-R0 gate. `chunked_md1_vector` is now fully concrete (md-codec public types + `md_codec::chunk::split`, mirroring md-codec's own `tests/bch_adversarial.rs`); the `md_codec::{origin_path,tag,tlv,tree,use_site_path}` modules and `Descriptor` are all public. No fixture-generation step or `_fixture` indirection remains.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p mnemonic-engrave bundle`
Expected: FAIL (missing helper(s) / behavior).

- [ ] **Step 3: Implement the md1 fixture helper**

No production code changes here — `chunked_md1_vector` and `foreign_index1_same_setid` are both concrete and self-contained (above), and the foreign/dup/drop/md1-chunked tests exercise `run_bundle`'s existing grouping + `mk_codec::decode`/`md_codec::chunk::reassemble` integrity logic. Just add the two helpers + the tests.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p mnemonic-engrave bundle`
Expected: PASS.

- [ ] **Step 5: Commit**
```bash
git add crates/me-cli/src/bundle.rs
git commit -m "test(me): bundle integrity coverage (dup, foreign-hash, md1 chunked)"
```

---

### Task 8: CLI wiring — optional `bundle` subcommand

**Files:** Modify `crates/me-cli/src/main.rs`

- [ ] **Step 1: Write the failing tests**

Add to `crates/me-cli/tests/cli.rs`:
```rust
const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
const MK1_B: &str = "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";

#[test]
fn bundle_emits_manifest_json_on_stdout() {
    let assert = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n"))
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON on stdout");
    assert_eq!(v["wallet_plates"], 4);
    assert_eq!(v["sets"][0]["chunk_set_id"], "0x12345");
    // checklist must NOT be on stdout
    assert!(!stdout.contains("TYPE ON DEVICE"));
}

#[test]
fn bundle_checklist_on_stderr() {
    let assert = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n"))
        .assert()
        .success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("TYPE ON DEVICE"), "{stderr}");
}

#[test]
fn bundle_ms1_refused_exit_3() {
    Command::cargo_bin("me").unwrap().arg("bundle")
        .write_stdin(MS1)
        .assert().code(3).stderr(predicates::str::contains("CODEX32"));
}

#[test]
fn bundle_dropped_chunk_exit_4_no_stdout() {
    let assert = Command::cargo_bin("me").unwrap().arg("bundle")
        .write_stdin(MK1_A) // total=2, only 1
        .assert().code(4);
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.trim().is_empty(), "no manifest on failure: {stdout}");
}

#[test]
fn existing_converter_still_works_without_subcommand() {
    Command::cargo_bin("me").unwrap().arg("--hex")
        .write_stdin(MD1_VALID).assert().success();
}
```
(`MD1_VALID` and `MS1` already exist as consts in `cli.rs`.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p mnemonic-engrave --test cli`
Expected: FAIL — no `bundle` subcommand.

- [ ] **Step 3: Implement the subcommand**

In `main.rs`, add a subcommand enum and an optional field on `Cli`, keeping the existing flags:
```rust
use clap::Subcommand;

#[derive(Subcommand)]
enum Command {
    /// Validate a wallet backup's public strings and emit a plate manifest + checklist.
    Bundle {
        /// Read newline-separated public strings from this file instead of stdin.
        #[arg(long, value_name = "FILE")]
        r#in: Option<PathBuf>,
        /// Write the manifest JSON to this file instead of stdout.
        #[arg(long, value_name = "FILE")]
        manifest: Option<PathBuf>,
    },
}
```
Add to `struct Cli` (alongside the existing flags):
```rust
    #[command(subcommand)]
    command: Option<Command>,
```
In `run()`, branch at the top BEFORE the existing single-string logic:
```rust
    let cli = Cli::parse();
    if let Some(Command::Bundle { r#in, manifest }) = &cli.command {
        return run_bundle_cli(r#in.as_ref(), manifest.as_ref());
    }
    // ... existing single-string conversion unchanged ...
```
Add the bundle CLI driver:
```rust
fn run_bundle_cli(in_path: Option<&PathBuf>, manifest_path: Option<&PathBuf>) -> i32 {
    use zeroize::Zeroizing;
    let mut input = Zeroizing::new(String::new());
    if let Some(path) = in_path {
        match std::fs::read_to_string(path) {
            Ok(s) => *input = s,
            Err(e) => { eprintln!("me: cannot read {}: {e}", path.display()); return EXIT_USAGE; }
        }
    } else if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("me: cannot read stdin: {e}");
        return EXIT_USAGE;
    }

    let manifest = match mnemonic_engrave::bundle::run_bundle(&input) {
        Ok(m) => m,
        Err(e) => { eprintln!("me: {e}"); return e.exit_code(); }
    };

    let json = match serde_json::to_string_pretty(&manifest) {
        Ok(j) => j,
        Err(e) => { eprintln!("me: cannot serialize manifest: {e}"); return EXIT_USAGE; }
    };
    if let Some(path) = manifest_path {
        if let Err(e) = std::fs::write(path, json.as_bytes()) {
            eprintln!("me: cannot write {}: {e}", path.display());
            return EXIT_USAGE;
        }
        eprintln!("me: wrote manifest to {}", path.display());
    } else {
        println!("{json}");
    }
    eprint!("{}", manifest.checklist());
    EXIT_OK
}
```
Ensure `mnemonic_engrave::bundle` is reachable (it is, via `lib.rs`).

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p mnemonic-engrave --test cli`
Expected: PASS (5 new + existing).

- [ ] **Step 5: Commit**
```bash
git add crates/me-cli/src/main.rs crates/me-cli/tests/cli.rs
git commit -m "feat(me): wire optional bundle subcommand (stdin/--in -> manifest/--manifest + checklist)"
```

---

### Task 9: Manifest golden test + clippy/fmt sweep

**Files:** Modify `crates/me-cli/tests/cli.rs` (or a new `tests/bundle_golden.rs`)

- [ ] **Step 1: Write the golden test**

Add a test that builds the manifest for a fixed input and compares a version-normalized JSON:
```rust
#[test]
fn bundle_manifest_golden() {
    let assert = Command::cargo_bin("me").unwrap().arg("bundle")
        .write_stdin(format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n"))
        .assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Normalize the version so a routine bump doesn't break the golden (spec m-4).
    v["version"] = serde_json::Value::String("x.y.z".into());
    let golden = include_str!("vectors/bundle-md1-mk1.json");
    let expected: serde_json::Value = serde_json::from_str(golden).unwrap();
    assert_eq!(v, expected);
}
```
Generate `crates/me-cli/tests/vectors/bundle-md1-mk1.json` once from the actual output (with `version` set to `"x.y.z"`), inspect it for correctness against SPEC §6, and commit it.

- [ ] **Step 2: Run to verify it fails, then passes after adding the vector**

Run: `cargo test -p mnemonic-engrave --test cli bundle_manifest_golden`
Expected: FAIL (missing vector) → add vector → PASS.

- [ ] **Step 3: Full gate**

Run: `cargo test -p mnemonic-engrave` then `cargo clippy -p mnemonic-engrave --all-targets -- -D warnings` then `cargo fmt -p mnemonic-engrave -- --check`
Expected: all green/clean. Fix any clippy/fmt issues.

- [ ] **Step 4: Commit**
```bash
git add crates/me-cli/tests/cli.rs crates/me-cli/tests/vectors/bundle-md1-mk1.json
git commit -m "test(me): bundle manifest golden (version-normalized)"
```

---

### Task 10: CHANGELOG + FOLLOWUPS bookkeeping

**Files:** `crates/me-cli/CHANGELOG.md` (create if absent); `design/FOLLOWUPS.md`

- [ ] **Step 1: CHANGELOG**

Add a `0.2.0` entry: "Added `me bundle`: validates a wallet backup's public md1/mk1 strings, proves chunk-set integrity (catches dropped/reordered/duplicate/foreign chunks), emits a JSON manifest + guided plate checklist. Refuses ms1."

- [ ] **Step 2: FOLLOWUPS**

In `design/FOLLOWUPS.md`: mark `me-bundle-preview-layer` Phase-A as done; open `me-bundle-preview-sidecar` (Phase B) carrying `DESIGN_me_bundle_preview.md` §B (R0 findings I-3/I-4/m-5 + the upstream-v1.4.2 pin).

- [ ] **Step 3: Commit**
```bash
git add crates/me-cli/CHANGELOG.md design/FOLLOWUPS.md
git commit -m "docs(me): CHANGELOG 0.2.0 + FOLLOWUPS (bundle Phase A done, Phase B opened)"
```

---

## Self-Review (run before dispatching execution)

**1. Spec coverage** (SPEC_me_bundle_phaseA.md → task):
- §3 CLI surface (optional subcommand, stdin/`--in`, stdout/`--manifest`, exit 0/2/3/4) → Task 8.
- §4 pipeline (classify, ms1 early-refuse, pristine validate, group, integrity) → Tasks 5, 6.
- §4 mk1 SingleString → exit 4 (I-1) → Task 5 (`Mk1SingleString`).
- §4 md1 4-way dispatch incl. WireVersionMismatch → exit 4 (I-2) → Task 5.
- §5 error mapping → Task 4 (`BundleError` Display) + Task 6.
- §6 manifest schema (sets/plates, integrity tristate, unchunked md1 omits chunk fields & sets[]) → Tasks 2, 6.
- §7 checklist → Task 3.
- §8 edge cases (empty/all-ms1/dup/incomplete/corrupted) → Tasks 5, 6, 8.
- §10 tests #1–#12 → Tasks 5–9 (golden #12 → Task 9; cross-hash #7 → Task 7; md1 chunked #8 → Task 7).
- §11 version bump + CHANGELOG + FOLLOWUPS → Tasks 1, 10.

**2. Placeholder scan:** None. Both Task-7 test-vector helpers are now concrete and hermetic — `foreign_index1_same_setid` uses literal `v0.1.json` strings + `mk_codec::encode_with_chunk_set_id`; `chunked_md1_vector` builds md-codec's public `multi_chunk_descriptor` + `md_codec::chunk::split` (plan-R0 pinned both paths). No placeholders.

**3. Type consistency:** `Parsed` variants (`Md1Single`/`Md1Chunk`/`Mk1Chunk`) used consistently in Tasks 5–6. `BundleError` variants (Task 4) referenced consistently in Tasks 5/6/8. `Integrity`/`Kind`/`PlateKind` (Task 2) used in Tasks 3/6. `run_bundle`/`parse_line` signatures stable across tasks. `fmt_chunk_set_id` defined Task 2, used Task 6.

**Known item for plan-R0:** confirm `md_codec::Error::WireVersionMismatch` and `ChunkHeaderChunkedFlagMissing` variant spellings; confirm `mk_codec::decode`/`md_codec::chunk::reassemble` take `&[&str]`; pin the md1 multi-chunk vector generation. (All asserted CONFIRMED in the spec R0 gate, but the plan-R0 gate re-verifies against the pinned crates.)

---

## Execution Handoff

Plan complete and saved to `design/IMPLEMENTATION_PLAN_me_bundle_phaseA.md`. **Before execution, this plan must pass the plan R0 architect gate (0C/0I).** After GREEN, recommended execution: **subagent-driven-development** (fresh subagent per task + two-stage review), in an isolated worktree.
