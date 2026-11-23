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

/// The 33 titles of §14.4: `SEED A` to `SEED P`, `HOT SEED A` to `HOT SEED P`
/// and `KEEP; DO NOT LOAD`.
const KEEP_TITLE: &str = "KEEP; DO NOT LOAD";

/// Whether `t` is one of the 33 titles.
fn is_title(t: &str) -> bool {
    t == KEEP_TITLE || title_letter(t).is_some()
}

/// The letter of a lettered title (`SEED <L>` or `HOT SEED <L>`, `L` in `A..=P`).
fn title_letter(t: &str) -> Option<char> {
    let l = t
        .strip_prefix("SEED ")
        .or_else(|| t.strip_prefix("HOT SEED "))?;
    let mut cs = l.chars();
    match (cs.next(), cs.next()) {
        (Some(c @ 'A'..='P'), None) => Some(c),
        _ => None,
    }
}

fn hex(b: &[u8]) -> String {
    use std::fmt::Write as _;
    b.iter().fold(String::new(), |mut a, x| {
        let _ = write!(a, "{x:02x}");
        a
    })
}

/// The titles record for `titles` (each one of the 33 titles of §14.4):
/// `text:` and the lower-case hex of the block. An empty list is `Empty`; a
/// title outside the 33 is `Line(i + 2)` (the magic line is line 1).
pub fn encode_titles(titles: &[&str]) -> Result<String, TitlesError> {
    if titles.is_empty() {
        return Err(TitlesError::Empty);
    }
    let mut block = format!("{TITLES_MAGIC}\n");
    for (i, t) in titles.iter().enumerate() {
        if !is_title(t) {
            return Err(TitlesError::Line(i + 2));
        }
        block.push_str("title ");
        block.push_str(t);
        block.push('\n');
    }
    Ok(format!("{TITLES_PREFIX}{}", hex(block.as_bytes())))
}

/// The letter of a seed record (§14.4): `A` plus the top four bits of the first
/// entropy byte of an unshared ms1 entr string. `None` for anything else: a
/// non-ms1, a share, a plain BIP-93 seed, a payload whose prefix is not `0x00`.
pub fn ms1_letter(ms1: &str) -> Option<char> {
    use zeroize::{Zeroize, Zeroizing};
    // `ms_codec::decode` is the constellation profile: an unshared string, the
    // `entr` id, and the 0x00-prefixed entropy of 16, 20, 24, 28 or 32 bytes
    // (`Payload::Entr`); a share, a 0x02 (mnem) payload and a preimage are other
    // answers. The entropy is a seed: it is wiped before this returns.
    match ms_codec::decode(ms1) {
        Ok((_, ms_codec::Payload::Entr(e))) => {
            let e = Zeroizing::new(e);
            e.first().map(|b| char::from(b'A' + (b >> 4)))
        }
        Ok((_, ms_codec::Payload::Mnem { mut entropy, .. })) => {
            entropy.zeroize();
            None
        }
        _ => None,
    }
}

/// Whether a record of an encrypted section is a seed record (§14.4): a
/// `codex32` secret `ms1` or a BIP-39 mnemonic. A titles record and an
/// `md1`/`mk1` record are not.
fn is_seed_record(r: &str) -> bool {
    matches!(
        super::record::validate_record(r),
        Ok(super::record::RecordKind::Ms)
    ) || super::passphrase::is_valid(r)
}

/// §14.4's ten checks over the records of a v2 encrypted section (the titles
/// record included, wherever it is), in the table's order. `Ok(None)` when the
/// section holds no titles record (legal: STOP-1); `Ok(Some(titles))` with the
/// title strings otherwise.
pub fn check_titles(records: &[String]) -> Result<Option<Vec<String>>, TitlesError> {
    let at: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| r.starts_with(TITLES_PREFIX))
        .map(|(i, _)| i)
        .collect();
    let Some(&first) = at.first() else {
        return Ok(None);
    };
    // 1.
    if at.len() > 1 {
        return Err(TitlesError::Duplicate);
    }
    // 2.
    if first + 1 != records.len() {
        return Err(TitlesError::NotLast);
    }
    // 3. An even number, at least 2, of lower-case hex digits that are UTF-8.
    let digits = &records[first][TITLES_PREFIX.len()..];
    if digits.len() < 2
        || digits.len() % 2 != 0
        || !digits
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(TitlesError::NotHex);
    }
    let bytes: Vec<u8> = (0..digits.len() / 2)
        .map(|i| u8::from_str_radix(&digits[2 * i..2 * i + 2], 16).expect("hex digits"))
        .collect();
    let block = String::from_utf8(bytes).map_err(|_| TitlesError::NotHex)?;
    // 4.
    let Some(body) = block.strip_suffix('\n') else {
        return Err(TitlesError::NoFinalLf);
    };
    let lines: Vec<&str> = body.split('\n').collect();
    // 5.
    if lines[0] != TITLES_MAGIC {
        return Err(TitlesError::Magic);
    }
    // 6. `n` counts from 1, the magic line being 1.
    let mut titles: Vec<String> = Vec::new();
    for (k, line) in lines.iter().enumerate().skip(1) {
        match line.strip_prefix("title ") {
            Some(t) if is_title(t) => titles.push(t.to_string()),
            _ => return Err(TitlesError::Line(k + 1)),
        }
    }
    // 7.
    if titles.is_empty() {
        return Err(TitlesError::Empty);
    }
    let seeds: Vec<&String> = records[..first]
        .iter()
        .filter(|r| is_seed_record(r))
        .collect();
    // 8.
    let mut letters: Vec<char> = Vec::with_capacity(seeds.len());
    for (i, r) in seeds.iter().enumerate() {
        match ms1_letter(r) {
            Some(l) => letters.push(l),
            None => return Err(TitlesError::NonMs1Secret(i)),
        }
    }
    // 9.
    if titles.len() != seeds.len() {
        return Err(TitlesError::Count {
            titles: titles.len(),
            secrets: seeds.len(),
        });
    }
    // 10. `KEEP; DO NOT LOAD` is not checked.
    for (i, (t, l)) in titles.iter().zip(&letters).enumerate() {
        if let Some(tl) = title_letter(t) {
            if tl != *l {
                return Err(TitlesError::Letter {
                    index: i,
                    title: t.clone(),
                    letter: *l,
                });
            }
        }
    }
    Ok(Some(titles))
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
