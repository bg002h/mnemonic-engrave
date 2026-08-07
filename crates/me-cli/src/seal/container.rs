//! §6.4 — the LF section encoding. Used identically for the public and the
//! encrypted section; the `1..24` cap is over the TOTAL across both, enforced by
//! the caller.

use zeroize::Zeroizing;

/// Bounded by `bundleReviewFlow`'s paged list, not by `ChoiceScreen`. An earlier
/// draft capped this at 7 from the wrong widget, which would have rejected every
/// multisig wallet — 2-of-2 is 10 records and 2-of-3 is 15.
pub const MAX_RECORDS: usize = 24;
pub const MAX_RECORD_LEN: usize = 512;

#[derive(Debug, PartialEq, Eq)]
pub enum ContainerError {
    RecordCount(usize),
    RecordTooLong { index: usize, len: usize },
    EmbeddedSeparator { index: usize, ch: char },
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerError::RecordCount(n) => write!(
                f,
                "{n} records; must be 1..={MAX_RECORDS} (2-of-3 multisig is 15)"
            ),
            ContainerError::RecordTooLong { index, len } => write!(
                f,
                "record {index} is {len} bytes; must be 1..={MAX_RECORD_LEN}"
            ),
            ContainerError::EmbeddedSeparator { index, ch } => write!(
                f,
                "record {index} contains {ch:?}, which is the record separator"
            ),
        }
    }
}
impl std::error::Error for ContainerError {}

/// Trim ONCE, then validate and encode the SAME trimmed form. Validating one
/// string and emitting another is how a trailing space survives to the device,
/// where `codex32.inputChar` has no mapping for `0x20`.
pub fn encode_section(records: &[String]) -> Result<Zeroizing<String>, ContainerError> {
    if records.is_empty() || records.len() > MAX_RECORDS {
        return Err(ContainerError::RecordCount(records.len()));
    }
    // §6.4: "No CR. A 0x0D anywhere is a malformed bundle. CRLF is rejected,
    // not tolerated." `\r` is `char::is_whitespace`, so trimming FIRST would
    // silently normalise a trailing CR away instead of refusing it. Scan the
    // UNTRIMMED records before trimming.
    for (i, r) in records.iter().enumerate() {
        if let Some(pos) = r.find('\r') {
            return Err(ContainerError::EmbeddedSeparator {
                index: i,
                ch: r[pos..].chars().next().unwrap(),
            });
        }
    }
    let trimmed: Vec<&str> = records.iter().map(|r| r.trim()).collect();
    for (i, r) in trimmed.iter().enumerate() {
        if r.is_empty() || r.len() > MAX_RECORD_LEN {
            return Err(ContainerError::RecordTooLong {
                index: i,
                len: r.len(),
            });
        }
        if let Some(pos) = r.find(['\n', '\r']) {
            return Err(ContainerError::EmbeddedSeparator {
                index: i,
                ch: r[pos..].chars().next().unwrap(),
            });
        }
    }
    Ok(Zeroizing::new(trimmed.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
    const B: &str = "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374";

    #[test]
    fn joins_with_lf_and_no_trailing_lf() {
        let out = encode_section(&[A.into(), B.into()]).unwrap();
        assert_eq!(*out, format!("{A}\n{B}"));
        assert!(!out.ends_with('\n'));
    }

    /// Validate-one-form-emit-another is the defect shape behind the round-3
    /// Critical. These must be byte-identical.
    #[test]
    fn surrounding_whitespace_does_not_change_the_encoding() {
        assert_eq!(
            *encode_section(&[A.into(), B.into()]).unwrap(),
            *encode_section(&[format!("  {A}  "), format!("\t{B}\n")]).unwrap()
        );
    }

    #[test]
    fn refuses_bad_record_counts() {
        assert!(matches!(
            encode_section(&[]),
            Err(ContainerError::RecordCount(0))
        ));
        let many: Vec<String> = std::iter::repeat_n(A.to_string(), 25).collect();
        assert!(matches!(
            encode_section(&many),
            Err(ContainerError::RecordCount(25))
        ));
        let ok: Vec<String> = std::iter::repeat_n(A.to_string(), 24).collect();
        assert!(
            encode_section(&ok).is_ok(),
            "24 is legal — a 2-of-3 bundle is 15 records"
        );
    }

    #[test]
    fn refuses_embedded_separators_and_bad_lengths() {
        assert!(encode_section(&["".into()]).is_err());
        assert!(encode_section(&[format!("{A}\n{A}")]).is_err());
        assert!(encode_section(&[format!("{A}\r")]).is_err());
        assert!(matches!(
            encode_section(&[format!("md1{}", "q".repeat(600))]),
            Err(ContainerError::RecordTooLong { .. })
        ));
    }
}
