//! Per-string (single-chunk) validation of public constellation strings.
//! Confirms HRP + BCH checksum so a corrupted string is never engraved.

use crate::classify::Format;

/// A validation failure, carrying the underlying codec error for a useful message.
#[derive(Debug)]
pub enum ValidateError {
    /// md1 string failed `md_codec` per-string checks.
    Md(md_codec::Error),
    /// mk1 string failed `mk_codec` per-string checks.
    Mk(mk_codec::Error),
    /// mk1 string was not pristine — `decode_string` had to BCH-correct N
    /// symbol(s). We refuse non-pristine input rather than engrave a string
    /// that needed repair (the converter engraves the input verbatim).
    MkCorrected(usize),
}

impl std::fmt::Display for ValidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidateError::Md(e) => write!(f, "invalid md1 string: {e}"),
            ValidateError::Mk(e) => write!(f, "invalid mk1 string: {e}"),
            ValidateError::MkCorrected(n) => write!(
                f,
                "mk1 string is not pristine: it required {n} BCH correction(s) — fix the input \
                 rather than engrave a string that needed repair"
            ),
        }
    }
}
impl std::error::Error for ValidateError {}

/// Validate one md1/mk1 string at the per-string BCH level, requiring PRISTINE
/// input. md1 (`unwrap_string`) is a pure verify — any corruption is rejected.
/// mk1 (`decode_string`) BCH error-CORRECTS up to 4 symbols, so we additionally
/// reject any string that needed correction (`corrections_applied != 0`): the
/// converter engraves the input verbatim, so a non-pristine input means the user
/// should fix their source, not engrave a string that required repair.
/// `Format::Ms` must never reach this function (it is refused before validation).
pub fn validate(fmt: Format, s: &str) -> Result<(), ValidateError> {
    match fmt {
        Format::Md => md_codec::codex32::unwrap_string(s)
            .map(|_| ())
            .map_err(ValidateError::Md),
        Format::Mk => {
            let decoded = mk_codec::string_layer::decode_string(s).map_err(ValidateError::Mk)?;
            if decoded.corrections_applied != 0 {
                return Err(ValidateError::MkCorrected(decoded.corrections_applied));
            }
            Ok(())
        }
        Format::Ms => panic!("validate() called on ms1 — must be refused before validation"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known-good vectors, current as of md-codec 0.36 / mk-codec 0.4. To refresh:
    //   md1: regenerate via the `md` CLI or `md_codec::encode_md1_string(...)`; the
    //        canonical v0.30 example is asserted at
    //        descriptor-mnemonic/crates/md-cli/tests/smoke.rs:21. NOTE: test_vectors.rs
    //        holds TEMPLATES, not strings — you must encode to get an md1 literal.
    //   mk1: copy a string from mnemonic-key/crates/mk-codec/src/test_vectors/v0.1.json
    const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
    const MK1_VALID: &str =
        "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";

    #[test]
    fn accepts_valid_md1() {
        assert!(validate(Format::Md, MD1_VALID).is_ok());
    }

    #[test]
    fn accepts_valid_mk1() {
        assert!(validate(Format::Mk, MK1_VALID).is_ok());
    }

    #[test]
    fn rejects_corrupted_md1() {
        // Flip one data character; the BCH checksum must reject it.
        let mut bad = MD1_VALID.to_string();
        let last = bad.pop().unwrap();
        bad.push(if last == 'q' { 'p' } else { 'q' });
        assert!(validate(Format::Md, &bad).is_err());
    }

    #[test]
    fn rejects_corrupted_mk1() {
        // A single flipped symbol is BCH-correctable, so decode_string returns
        // Ok with corrections_applied=1 — which validate() rejects as non-pristine.
        let mut bad = MK1_VALID.to_string();
        let last = bad.pop().unwrap();
        bad.push(if last == 'q' { 'p' } else { 'q' });
        assert!(matches!(validate(Format::Mk, &bad), Err(ValidateError::MkCorrected(_))));
    }
}
