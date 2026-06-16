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
