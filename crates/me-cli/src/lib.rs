//! `mnemonic-engrave` (`me`) — converts public constellation strings (md1/mk1)
//! into NFC NDEF payloads for SeedHammer II. Refuses the secret ms1.

pub mod bundle;
pub mod classify;
pub mod manifest;
pub mod ndef;
pub mod validate;

use classify::{ClassifyError, Format};

/// Failure modes of the end-to-end conversion.
#[derive(Debug)]
pub enum ConvertError {
    /// Could not classify the HRP.
    Classify(ClassifyError),
    /// `ms1` was supplied — refused: secret material must never go over RF.
    RefusedSecret,
    /// The md1/mk1 string failed validation.
    Validate(validate::ValidateError),
    /// NDEF encoding failed (e.g. string too long for one record).
    Ndef(ndef::NdefError),
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertError::Classify(e) => write!(f, "{e}"),
            ConvertError::RefusedSecret => write!(
                f,
                "refusing to emit ms1 over NFC: ms1 is secret seed entropy and must never be \
                 transmitted by radio (RF eavesdropping risk). Enter it by hand on the device: \
                 New > Input Seed > CODEX32."
            ),
            ConvertError::Validate(e) => write!(f, "{e}"),
            ConvertError::Ndef(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for ConvertError {}

/// Conservative single-plate text budget. SeedHammer's 85x85mm text layout
/// wraps ~35 chars/line over ~20 usable lines; with a QR present, far less.
/// This is an advisory pre-check — the firmware still backstops with ErrTooLarge.
const PLATE_TEXT_BUDGET: usize = 300;

/// Reports whether a string is long enough to risk overflowing one plate.
pub fn exceeds_plate_budget(s: &str) -> bool {
    s.trim().len() > PLATE_TEXT_BUDGET
}

/// Validate one public constellation string and return its TLV-wrapped NDEF
/// bytes. Refuses `ms1`. The input is trimmed; the verbatim trimmed string is
/// what gets encoded (and later engraved).
pub fn convert(input: &str) -> Result<Vec<u8>, ConvertError> {
    let s = input.trim();
    let fmt = classify::classify(s).map_err(ConvertError::Classify)?;
    if fmt == Format::Ms {
        return Err(ConvertError::RefusedSecret);
    }
    validate::validate(fmt, s).map_err(ConvertError::Validate)?;
    ndef::encode_text_tlv(s).map_err(ConvertError::Ndef)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

    #[test]
    fn converts_md1_to_ndef() {
        let bytes = convert(MD1_VALID).unwrap();
        assert_eq!(ndef::decode_text_tlv(&bytes).as_deref(), Some(MD1_VALID));
    }

    #[test]
    fn refuses_ms1() {
        let err = convert("ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f").unwrap_err();
        assert!(matches!(err, ConvertError::RefusedSecret));
    }

    #[test]
    fn rejects_unknown_hrp() {
        assert!(matches!(convert("xx1qqqq"), Err(ConvertError::Classify(_))));
    }

    #[test]
    fn flags_plate_overflow_risk() {
        // A string longer than the ~ one-plate text budget should be flagged.
        assert!(super::exceeds_plate_budget(&"q".repeat(400)));
        assert!(!super::exceeds_plate_budget("md1yqpqqxqq8xtwhw4xwn4qh"));
    }
}
