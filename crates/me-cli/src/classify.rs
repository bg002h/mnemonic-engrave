//! Classify a constellation string by its bech32 HRP (the text before the
//! first `1` separator), case-insensitively.

/// The three constellation formats this tool understands.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Format {
    /// md1 — wallet descriptor / policy (public).
    Md,
    /// mk1 — xpubs (public).
    Mk,
    /// ms1 — secret entropy (refused; never over RF).
    Ms,
}

/// Why a string could not be classified.
#[derive(Debug, PartialEq, Eq)]
pub enum ClassifyError {
    /// No `1` separator, or the HRP was empty.
    NoSeparator,
    /// HRP present but not one of md/mk/ms.
    UnknownHrp(String),
}

impl std::fmt::Display for ClassifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassifyError::NoSeparator => write!(f, "not a bech32 string (no '1' separator / empty HRP)"),
            ClassifyError::UnknownHrp(h) => write!(f, "unrecognized HRP '{h}' (expected md, mk, or ms)"),
        }
    }
}
impl std::error::Error for ClassifyError {}

/// Determine the format from the HRP. Trims surrounding whitespace and lowercases
/// the HRP before matching (bech32 is case-insensitive).
pub fn classify(s: &str) -> Result<Format, ClassifyError> {
    let s = s.trim();
    let sep = s.find('1').ok_or(ClassifyError::NoSeparator)?;
    if sep == 0 {
        return Err(ClassifyError::NoSeparator);
    }
    match s[..sep].to_ascii_lowercase().as_str() {
        "md" => Ok(Format::Md),
        "mk" => Ok(Format::Mk),
        "ms" => Ok(Format::Ms),
        other => Err(ClassifyError::UnknownHrp(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_each_hrp() {
        assert_eq!(classify("md1yqpqqxqq8xtwhw4xwn4qh"), Ok(Format::Md));
        assert_eq!(classify("mk1qpzry9x8gf2tv"), Ok(Format::Mk));
        assert_eq!(classify("ms10entrsqqqqqqq"), Ok(Format::Ms));
    }

    #[test]
    fn is_case_insensitive_and_trims() {
        assert_eq!(classify("  MD1QQPQ  "), Ok(Format::Md));
    }

    #[test]
    fn rejects_unknown_and_malformed() {
        assert_eq!(classify("xx1qqqq"), Err(ClassifyError::UnknownHrp("xx".into())));
        assert_eq!(classify("noseparator"), Err(ClassifyError::NoSeparator));
        assert_eq!(classify("1leadingsep"), Err(ClassifyError::NoSeparator));
    }
}
