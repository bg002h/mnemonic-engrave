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
    /// mt1 — one chunk of a signed Bitcoin transaction (public; bearer).
    Mt,
}

/// Why a string could not be classified.
#[derive(Debug, PartialEq, Eq)]
pub enum ClassifyError {
    /// No `1` separator, or the HRP was empty.
    NoSeparator,
    /// HRP present but not one of md/mk/ms/mt.
    ///
    /// **Carries no input text** (bug hunt 2026-11-25 E1). The "HRP" is
    /// everything before the first `1` of the input, with no length or charset
    /// bound, so a mistyped ms1 (`msl0…`) or a BIP-39 mnemonic followed by any
    /// `1` used to be printed to stderr in full. It names the HRP only when it
    /// is one of [`KNOWN_PUBLIC_HRPS`], a fixed list of public constellation
    /// prefixes, and then only as that list's own `&'static str`.
    UnknownHrp(Option<&'static str>),
}

/// Public constellation HRPs `me` does not take, which an "unrecognized HRP"
/// message may name because they are not secret and naming them helps
/// (`me seal` on an mr1 card says `'mr'`, as SPEC_systemwide_payloads §14d
/// rules). Nothing else from the input is ever shown.
pub const KNOWN_PUBLIC_HRPS: &[&str] = &["mr"];

impl std::fmt::Display for ClassifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassifyError::NoSeparator => {
                write!(f, "not a bech32 string (no '1' separator / empty HRP)")
            }
            ClassifyError::UnknownHrp(Some(h)) => {
                write!(f, "unrecognized HRP '{h}' (expected md, mk, ms, or mt)")
            }
            // The input is NOT echoed: the text before its first `1` may be a
            // secret (a mistyped ms1, a seed phrase, a private key).
            ClassifyError::UnknownHrp(None) => write!(
                f,
                "unrecognized HRP (expected md, mk, ms, or mt; the text before the \
                 first '1' is not shown, because it may be secret)"
            ),
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
        "mt" => Ok(Format::Mt),
        other => Err(ClassifyError::UnknownHrp(
            KNOWN_PUBLIC_HRPS.iter().copied().find(|k| *k == other),
        )),
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
        assert_eq!(classify("xx1qqqq"), Err(ClassifyError::UnknownHrp(None)));
        assert_eq!(
            classify("MR1qqqq"),
            Err(ClassifyError::UnknownHrp(Some("mr")))
        );
        assert_eq!(classify("noseparator"), Err(ClassifyError::NoSeparator));
        assert_eq!(classify("1leadingsep"), Err(ClassifyError::NoSeparator));
    }

    /// Bug hunt 2026-11-25 E1: the message never carries input text. The
    /// canary sits where the HRP is taken from -- everything before the first
    /// `1` -- in each shape the hunt reproduced.
    #[test]
    fn bh1125_e1_unknown_hrp_display_carries_no_input() {
        const CANARY: &str = "canarysecretbody";
        for input in [
            format!("msl0{CANARY}q5f042\nmd1yqpqqxqq8xtwhw4xwn4qh"),
            format!("abandon {CANARY} about plate 1"),
            format!("seed: {CANARY} (12 words)\nmd1qq"),
            format!("e3b0c44298fc{CANARY}1c149a"),
            format!("{CANARY}1"),
        ] {
            let e = classify(&input).unwrap_err();
            let shown = format!("{e} {e:?}");
            assert!(!shown.contains(CANARY), "echoed the input: {shown}");
            assert!(!shown.contains("abandon"), "echoed the input: {shown}");
            assert!(!shown.contains("e3b0"), "echoed the input: {shown}");
        }
    }
}
