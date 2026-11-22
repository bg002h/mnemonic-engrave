//! The v2 titles record (S1b §14.4): one record of a v2 payload's encrypted
//! section, `text:` and the lower-case hex of a titles block.

/// The five characters every titles record begins with.
pub const TITLES_PREFIX: &str = "text:";
/// The block's first line.
pub const TITLES_MAGIC: &str = "refugium-plates 2";

/// The ten checks of §14.4, one variant each, in check order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TitlesError {
    /// 1.
    Duplicate,
    /// 2.
    NotLast,
    /// 3.
    NotHex,
    /// 4.
    NoFinalLf,
    /// 5.
    Magic,
    /// 6. `n` counts from 1, the magic line being 1.
    Line(usize),
    /// 7.
    Empty,
    /// 8. `i` counts the seed records from 0.
    NonMs1Secret(usize),
    /// 9.
    Count { titles: usize, secrets: usize },
    /// 10.
    Letter {
        index: usize,
        title: String,
        letter: char,
    },
}

impl std::fmt::Display for TitlesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TitlesError::Duplicate => {
                write!(f, "the encrypted section holds more than one titles record")
            }
            TitlesError::NotLast => write!(
                f,
                "the titles record must be the last record of the encrypted section"
            ),
            TitlesError::NotHex => write!(
                f,
                "the titles record is not text: followed by lower-case hex of UTF-8"
            ),
            TitlesError::NoFinalLf => {
                write!(f, "the titles block must end with a line feed")
            }
            TitlesError::Magic => write!(
                f,
                "the titles block must start with the line {TITLES_MAGIC}"
            ),
            TitlesError::Line(n) => write!(
                f,
                "titles block line {n} is not title SEED <A-P>, title HOT SEED <A-P> or title \
                 KEEP; DO NOT LOAD"
            ),
            TitlesError::Empty => write!(f, "the titles block names no plate"),
            TitlesError::NonMs1Secret(i) => write!(
                f,
                "titles apply only to ms1 entr records; secret record {i} (records count from 0) \
                 is not one"
            ),
            TitlesError::Count { titles, secrets } => write!(
                f,
                "the titles block names {titles} plates but the encrypted section holds \
                 {secrets} seed records"
            ),
            TitlesError::Letter {
                index,
                title,
                letter,
            } => write!(
                f,
                "secret record {index} (records count from 0) is titled {title} but its seed \
                 letter is {letter}"
            ),
        }
    }
}
impl std::error::Error for TitlesError {}

/// The titles record for `titles` (each one of the 33 titles of §14.4):
/// `text:` and the lower-case hex of the block. An empty list is `Empty`; a
/// title outside the 33 is `Line(i + 2)` (the magic line is line 1).
pub fn encode_titles(_titles: &[&str]) -> Result<String, TitlesError> {
    unimplemented!()
}

/// The letter of a seed record (§14.4): `A` plus the top four bits of the first
/// entropy byte of an unshared ms1 entr string. `None` for anything else: a
/// non-ms1, a share, a plain BIP-93 seed, a payload whose prefix is not `0x00`.
pub fn ms1_letter(_ms1: &str) -> Option<char> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const A16: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    const MS1_32: [&str; 8] = [
        "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w",
        "ms10entrsqqg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3ztl5m2c7mg3ls6",
        "ms10entrsqq3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zym4y84zxyuwtkl",
        "ms10entrsqqenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxgyv57h75aydvt",
        "ms10entrsqpzyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygxfk0vzg2zk356",
        "ms10entrsqp2424242424242424242424242424242424242424242424242424c7u8hs6ruhww",
        "ms10entrsqpnxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxvenxv9jwqcdg9hrrgt",
        "ms10entrsqrllllllllllllllllllllllllllllllllllllllllllllllllll7ydtcvhdp9ycqe",
    ];
    const SHARE: &str = "ms13casha320zyxwvutsrqpnmlkjhgfedca2a8d0zehn8a0t";
    const MD1: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
    // A checksum-valid unshared ms1 whose payload is not an entr one (S1b §14.6).
    const BIP93_SEED: &str = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw";

    /// S1b §14.10: `A16`→A, `A`…`G`→A…G, `P`→P; a non-ms1, the N-share string, a
    /// plain BIP-93 seed and a `0x02` payload → `None`.
    #[test]
    fn ms1_letter_is_the_top_nibble_of_the_entropy() {
        assert_eq!(ms1_letter(A16), Some('A'));
        // `A` is 32 bytes of 0x00, `B`...`G` are 0x11...0x66 and `P` is 0xff:
        // the top nibbles are 0, 1 ... 6 and 15.
        assert_eq!(ms1_letter(MS1_32[0]), Some('A'));
        for (i, s) in MS1_32[1..7].iter().enumerate() {
            assert_eq!(ms1_letter(s), Some(char::from(b'B' + i as u8)), "{s}");
        }
        assert_eq!(ms1_letter(MS1_32[7]), Some('P'));
        assert_eq!(ms1_letter(MD1), None, "a non-ms1");
        assert_eq!(
            ms1_letter("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"),
            None,
            "a mnemonic"
        );
        assert_eq!(ms1_letter(SHARE), None, "the N-share string");
        assert_eq!(ms1_letter(BIP93_SEED), None, "a plain BIP-93 seed");
        // A 0x02 payload: ms-codec's Mnem kind.
        let mnem = ms_codec::encode(
            ms_codec::Tag::ENTR,
            &ms_codec::Payload::Mnem {
                language: 0,
                entropy: vec![0x00; 16],
            },
        )
        .unwrap();
        assert_eq!(ms1_letter(&mnem), None, "a 0x02 payload");
    }

    /// S1b §14.7: an empty list and a title outside the 33 are refused; the
    /// record is `text:` and the hex of the block.
    #[test]
    fn encode_titles_refuses_empty_and_unknown_titles() {
        assert_eq!(encode_titles(&[]), Err(TitlesError::Empty));
        assert_eq!(
            encode_titles(&["SEED A", "SEED Q"]),
            Err(TitlesError::Line(3))
        );
        assert_eq!(
            encode_titles(&["SEED A"]).unwrap(),
            "text:726566756769756d2d706c6174657320320a7469746c65205345454420410a"
        );
    }
}
