//! Record classes for the systemwide container, and the two encoded ones.
//!
//! SPEC_systemwide_payloads §3.3.1 and §5.3.1. The normative rules live there;
//! this file implements them and restates none.
//!
//! WHY FREE TEXT AND PASSPHRASES ARE HEX-ENCODED, in one paragraph, because it
//! is the least obvious thing here: EPD §6.4 requires every record to be "the
//! canonical, unbroken string — no interior spaces, no hyphens, no grouping of
//! any kind", and uses LF as the record separator on the stated grounds that no
//! constellation string contains a newline. `Hello, World!` has a space,
//! `correct horse battery staple` has three, and Engrave Text's keyboard has a
//! newline key. Both new classes therefore break both clauses. Relaxing §6.4
//! for two classes would weaken it for all of them — records are engraved
//! VERBATIM, so a record carrying uncovered separator characters turns a scratch
//! on the operator's only copy into silently-absorbed damage. So the body is
//! encoded and the record stays canonical.
//!
//! Lowercase hex rather than base64 or base32: EPD §6.6 hashes the section in
//! its canonical LOWERCASE form, and hex is the only common encoding that
//! survives lowercasing unchanged.

use zeroize::Zeroizing;

/// Prefixes are RESERVED. A record beginning with one whose body is not valid
/// lowercase hex is [`Class::Unknown`] and refused — never quietly treated as
/// free text, which would let a malformed record become an engraved plate.
pub const TEXT_PREFIX: &str = "text:";
pub const PASS_PREFIX: &str = "pass:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Mnemonic,
    Codex32Secret,
    Passphrase,
    FreeText,
    Descriptor,
    MdMk,
    Address,
    Unknown,
}

impl Class {
    /// §3.3.1. Extends the shipped predicate (`seal/session.go:17`, which is
    /// `ClassCodex32Secret || ClassMnemonic`) with [`Class::Passphrase`].
    ///
    /// [`Class::FreeText`] is deliberately NOT secret even though an operator
    /// may put anything in it: a class states what the format guarantees, not
    /// what a human might do, and a class claiming secrecy it cannot enforce is
    /// the over-claim F-123 was filed against.
    pub fn is_secret(self) -> bool {
        matches!(
            self,
            Class::Mnemonic | Class::Codex32Secret | Class::Passphrase
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecordError {
    /// A reserved prefix with a body that is not valid lowercase hex.
    BadHex,
    /// Hex decoded, but the bytes are not UTF-8.
    NotUtf8,
    /// Not one of the encoded classes.
    NotEncoded,
}

/// Encode free text as a canonical record.
pub fn encode_text(s: &str) -> String {
    format!("{TEXT_PREFIX}{}", hex_lower(s.as_bytes()))
}

/// Encode a passphrase as a canonical record.
///
/// `Zeroizing` because the intermediate hex is as sensitive as the passphrase:
/// it is a reversible encoding, not a hash.
pub fn encode_pass(s: &str) -> Zeroizing<String> {
    Zeroizing::new(format!("{PASS_PREFIX}{}", hex_lower(s.as_bytes())))
}

/// Decode a `text:` or `pass:` record's body back to its bytes.
pub fn decode_body(record: &str) -> Result<Zeroizing<Vec<u8>>, RecordError> {
    let body = record
        .strip_prefix(TEXT_PREFIX)
        .or_else(|| record.strip_prefix(PASS_PREFIX))
        .ok_or(RecordError::NotEncoded)?;
    unhex_lower(body).ok_or(RecordError::BadHex)
}

/// Decode to a `String`, for the consumers that need text rather than bytes.
pub fn decode_text(record: &str) -> Result<Zeroizing<String>, RecordError> {
    let b = decode_body(record)?;
    let s = std::str::from_utf8(&b).map_err(|_| RecordError::NotUtf8)?;
    Ok(Zeroizing::new(s.to_owned()))
}

/// Indices of [`Class::MdMk`] records that are NOT decode-confirmed (spec §12.6).
///
/// Groups by `(hrp, chunk_set_id)` exactly as `seal::record::decode_public_set`
/// does (`crates/me-cli/src/seal/record.rs:192`) — R1-I2: filter the iteration,
/// never the indices — but REPORTS instead of refusing: an incomplete or
/// non-decoding group marks its members unconfirmed and returns.
///
/// The returned indices are into the CALLER'S slice, whatever else it holds. A
/// caller that pre-filtered to the `MdMk` records and then reported those
/// positions would name the wrong record to the operator, which is R1-I2 said
/// the other way round.
///
/// **Why the grouping is duplicated rather than delegated.** `decode_public_set`
/// answers a different question — "may this whole public section be sealed" — and
/// its `chunk_key` is private to a FROZEN module (spec decision 1). The rule the
/// two share is §12.6's grouping, so
/// [`tests::the_two_walks_agree_wherever_both_have_an_answer`] pins them against
/// each other instead of leaving the copy unwatched.
pub fn mdmk_unconfirmed(records: &[String]) -> Vec<usize> {
    use std::collections::BTreeMap;

    // key: (hrp, chunk_set_id, uniq) -> the ORIGINAL indices in that card set.
    // `uniq` distinguishes non-chunked records, which are each their own card;
    // giving them all one key would let the first one to decode vouch for the
    // rest, which is precisely how smuggled entropy would slip through.
    let mut groups: BTreeMap<(char, Option<u32>, usize), Vec<usize>> = BTreeMap::new();
    let mut out: Vec<usize> = Vec::new();

    for (i, r) in records.iter().enumerate() {
        if super::classify(r) != Class::MdMk {
            continue;
        }
        match crate::seal::record::validate_record(r)
            .ok()
            .and_then(|kind| chunk_key(r, kind))
        {
            Some((hrp, csid)) => {
                let uniq = if csid.is_some() { 0 } else { i + 1 };
                groups.entry((hrp, csid, uniq)).or_default().push(i);
            }
            // Fail CLOSED. A record whose card identity cannot be read cannot be
            // grouped, so nothing could ever confirm it.
            //
            // REACHABLE, and I first wrote that it was not. `validate_record`
            // TRIMS before it validates (`seal/record.rs:118`) while the decoders
            // below are handed the record as given — so ` md1…` classifies
            // `MdMk` and then fails `unwrap_string` with "does not start with HRP
            // md1". `decode_public_set` has the same asymmetry and the same
            // answer, which is why the two walks still agree.
            // See `a_record_whose_card_identity_cannot_be_read_is_unconfirmed`.
            None => out.push(i),
        }
    }

    for ((hrp, csid, _), idxs) in groups {
        let set: Vec<&str> = idxs.iter().map(|&i| records[i].as_str()).collect();
        // The real decoders are the arbiter — semantics-bound, per §12.6. A
        // BCH verifier is not one: that is the whole point of the rule.
        let confirmed = match (hrp, csid) {
            ('d', Some(_)) => md_codec::reassemble(&set).is_ok(),
            ('d', None) => md_codec::decode_md1_string(set[0]).is_ok(),
            ('k', _) => mk_codec::decode(&set).is_ok(),
            _ => false,
        };
        if !confirmed {
            out.extend(idxs);
        }
    }

    out.sort_unstable();
    out
}

/// Card identity: the HRP discriminant plus the 20-bit chunk-set id, or `None`
/// when the record is not chunked (and is therefore its own card).
///
/// DELEGATES to `seal::record::chunk_key`, which the operator made `pub(crate)`
/// on 2026-08-12 for exactly this. It used to be a hand-copy, and a hand-copy of
/// a grouping rule is how the two drift.
///
/// **The `Ms` arm is NOT delegated, and that is the whole reason this wrapper
/// exists.** `seal`'s version is `unreachable!()` there — correct for its
/// caller, which refuses secret records first — but this rule runs over a
/// payload the DEVICE was handed, so an `ms1` reaching it is a disagreement
/// between `classify` and `validate_record`, not an impossibility. Delegating
/// that arm would turn a defensive `None` into a panic on the device.
fn chunk_key(s: &str, kind: crate::seal::record::RecordKind) -> Option<(char, Option<u32>)> {
    use crate::seal::record::RecordKind;
    match kind {
        RecordKind::Ms => None,
        // Every error from `seal`'s version — undecodable string, unrecognised
        // header variant — means "not groupable", which is this rule's
        // fail-closed answer. The mapping is total, so no case is lost.
        _ => crate::seal::record::chunk_key(s, kind).ok(),
    }
}

fn hex_lower(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Strictly lowercase, strictly even-length. Uppercase is rejected rather than
/// accepted-and-lowercased: EPD §6.6 hashes the record as it appears on the
/// wire, so two spellings of one body would be two different digests.
fn unhex_lower(s: &str) -> Option<Zeroizing<Vec<u8>>> {
    // `% 2 != 0` rather than `is_multiple_of` — unstable on CI's Rust.
    #[allow(clippy::manual_is_multiple_of)]
    let odd = s.len() % 2 != 0;
    if odd {
        return None;
    }
    let mut out = Zeroizing::new(Vec::with_capacity(s.len() / 2));
    let b = s.as_bytes();
    for pair in b.chunks(2) {
        let hi = nibble(pair[0])?;
        let lo = nibble(pair[1])?;
        // `|` and `^` are EQUIVALENT here and cargo-mutants reports the swap as
        // missed. It is a true equivalent mutant, not a coverage gap: `hi << 4`
        // occupies bits 4..7 and `lo` bits 0..3, so the operands never share a
        // set bit and both operators yield the same byte for every input. Left
        // as `|` because it states the intent; recorded so the next run does not
        // spend a round rediscovering it.
        out.push(hi << 4 | lo);
    }
    Some(out)
}

fn nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_the_characters_epd_6_4_forbids_raw() {
        // Vector S-G. This is the case the whole encoding exists for: a space,
        // a newline (the record SEPARATOR itself) and a non-ASCII byte.
        let s = "Hello, World!\nsecond line — é";
        let rec = encode_text(s);
        assert!(
            !rec.contains(' '),
            "an encoded record may not contain a space"
        );
        assert!(
            !rec.contains('\n'),
            "an encoded record may not contain the separator"
        );
        assert!(rec.is_ascii(), "an encoded record must be ASCII");
        assert_eq!(&*decode_text(&rec).unwrap(), s);
    }

    #[test]
    fn encoding_is_lowercase_so_it_survives_canonicalisation() {
        // EPD §6.6 hashes the LOWERCASE form. An encoding with uppercase would
        // hash differently before and after canonicalisation.
        let rec = encode_text("\u{00FF}\u{00AB}");
        assert_eq!(rec, "text:c3bfc2ab");
        assert_eq!(rec, rec.to_lowercase());
    }

    #[test]
    fn uppercase_hex_is_refused_not_accepted() {
        assert_eq!(decode_body("text:C3BF"), Err(RecordError::BadHex));
    }

    #[test]
    fn a_reserved_prefix_with_a_bad_body_is_an_error_not_free_text() {
        // The reservation: never quietly treat a malformed record as text.
        for bad in ["text:zz", "text:abc", "pass:!!", "text:"] {
            if bad == "text:" {
                assert_eq!(&**decode_body(bad).unwrap(), b"", "empty body is valid hex");
                continue;
            }
            assert_eq!(decode_body(bad), Err(RecordError::BadHex), "{bad}");
        }
    }

    #[test]
    fn an_unprefixed_record_is_not_encoded() {
        assert_eq!(decode_body("md1qqq"), Err(RecordError::NotEncoded));
    }

    #[test]
    fn passphrase_is_secret_and_free_text_is_not() {
        assert!(Class::Passphrase.is_secret());
        assert!(Class::Mnemonic.is_secret());
        assert!(Class::Codex32Secret.is_secret());
        assert!(
            !Class::FreeText.is_secret(),
            "a class may not claim secrecy it cannot enforce"
        );
        assert!(!Class::MdMk.is_secret());
        assert!(!Class::Descriptor.is_secret());
        assert!(!Class::Address.is_secret());
        assert!(!Class::Unknown.is_secret());
    }

    /// `encode_pass` had NO test until cargo-mutants pointed it out: three of its
    /// mutants — returning an empty string, and returning `"xyzzy"` — all
    /// survived. My own nine hand-written mutants missed it entirely, because I
    /// mutated the code I was thinking about and not the function I had written
    /// and forgotten to exercise. That is the argument for generating mutants
    /// from the AST rather than from the author's memory.
    #[test]
    fn pass_round_trips_and_is_prefixed() {
        let secret = "abandon abandon abandon abandon abandon";
        let rec = encode_pass(secret);
        assert!(
            rec.starts_with(PASS_PREFIX),
            "must carry the reserved prefix"
        );
        assert_ne!(
            &*rec, PASS_PREFIX,
            "an empty body is not an encoding of this"
        );
        assert!(
            !rec.contains(' '),
            "the encoded record may not contain a space"
        );
        assert_eq!(&*decode_text(&rec).unwrap(), secret);
    }

    /// The two encoders must not be interchangeable: a `pass:` record consumed
    /// as free text would engrave a passphrase onto a plate.
    #[test]
    fn text_and_pass_have_different_prefixes_for_the_same_body() {
        let body = "hello";
        assert_ne!(encode_text(body), *encode_pass(body));
        assert_eq!(
            decode_body(&encode_text(body)).unwrap().to_vec(),
            decode_body(&encode_pass(body)).unwrap().to_vec(),
            "same bytes, different prefix"
        );
    }

    #[test]
    fn empty_text_round_trips() {
        assert_eq!(&*decode_text(&encode_text("")).unwrap(), "");
    }

    // ---------------------------------------------------------------------
    // `[mdmk-decode]` — spec §12.6, ruled in as §13 D6.
    //
    // REAL CARDS, not hand-built fixtures: the reassembler is the arbiter of
    // what a card set is, so a fixture invented to make a test pass would only
    // pin my idea of the format. Every string below is lifted verbatim from a
    // shipped vector, and its chunk header was MEASURED, not assumed:
    //
    //   md1fv9wjpq…  chunk_set_id 398802, chunks 0,1,2 of 3   (seal vector D/E)
    //   md1fe4dazs…  chunk_set_id 841149, chunk  0    of 6    (seal vector G)
    //   md1yqpqq…    NOT chunked, decodes on its own          (tests/cli.rs)
    //   mk1qpzg69p…  a complete 2-chunk mk1 set               (tests/prop.rs)
    //   mk1qpykrep…  chunk 0 of a 2-chunk mk1 set             (seal vector G)
    // ---------------------------------------------------------------------

    /// Chunk 0 of 3, chunk_set_id 398802. Also the `MD1` every other sysw
    /// fixture uses — so the whole existing vector set carries an UNCONFIRMED
    /// record, which is worth knowing.
    const MD1_A: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
    const MD1_B: &str = "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374";
    const MD1_C: &str = "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz";
    /// Chunk 0 of 6 of a DIFFERENT set — 841149, not 398802.
    const MD1_OTHER: &str =
        "md1fe4dazspq3m67zzqqvzrs3pstucnf4ztqz4pk6ujgjycfn6zhs79nmzdp9frd6dzth6asfu2za4mwgfkg6";
    /// Not chunked at all: its own card, and it decodes.
    const MD1_SINGLE: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
    const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
    const MK1_B: &str =
        "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";
    /// Chunk 0 of a 2-chunk mk1 set whose second chunk is absent here.
    const MK1_LONE: &str = "mk1qpykrepqqspjtpuhfqjc096gykrewjy6dgjcqpcy3zepaggqseet8ky6z2jxm56yh04m5mqslrmueekdmecm0js2h978k03jfvkwz2rxj8r8";
    const SEED: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon \
                        abandon abandon about";

    fn recs(ss: &[&str]) -> Vec<String> {
        ss.iter().map(|s| (*s).to_string()).collect()
    }

    /// 32 bytes of seed entropy wrapped into a BCH-valid `md1` — the smuggling
    /// case spec §5.3.2 names, built the way a defective sealer would build it.
    ///
    /// This is `[mdmk-decode]`'s DECODE-FAILURE arm, and it is a unit test rather
    /// than a vector because making one needs the encoder: the vectors file
    /// carries only records, and this record has to be constructed.
    ///
    /// `fill` selects WHICH failure. `0xAB` sets the chunked-flag bit with a
    /// wrong wire version, which the two implementations reach by different
    /// routes (both ending at unconfirmed — see `sysw/confirm.go`). `0x01`
    /// leaves the flag clear, so both take the NON-CHUNKED path and the port
    /// tests the same grouping arm this one does.
    fn entropy_smuggled_as_md1(fill: u8) -> String {
        md_codec::codex32::wrap_payload(&[fill; 32], 32 * 8)
            .expect("32 bytes must wrap into a codex32 md1 string")
    }

    #[test]
    fn a_lone_chunk_of_a_declared_set_is_unconfirmed() {
        // The set declares 3 chunks and one is present, so nothing reassembles
        // and §12.6 leaves it unconfirmed. Nothing is REFUSED — that is D6.
        assert_eq!(mdmk_unconfirmed(&recs(&[MD1_A])), vec![0]);
    }

    #[test]
    fn the_complete_set_is_confirmed() {
        assert_eq!(
            mdmk_unconfirmed(&recs(&[MD1_A, MD1_B, MD1_C])),
            Vec::<usize>::new()
        );
    }

    /// **Group by `(hrp, chunk_set_id)`, never by HRP alone.** Grouping by HRP
    /// puts four `md1` records in one group, the header declares 3, and a
    /// COMPLETE set is reported unconfirmed — the R1-I2 shape, and the reason
    /// §12.6 states the key rather than saying "by card".
    #[test]
    fn grouping_is_by_chunk_set_id_not_by_hrp_alone() {
        assert_eq!(
            mdmk_unconfirmed(&recs(&[MD1_A, MD1_B, MD1_C, MD1_OTHER])),
            vec![3],
            "the complete 398802 set must confirm even beside a stray chunk of 841149"
        );
    }

    /// R1-I2 said the other way round: filter the ITERATION, never the indices.
    /// A walk that collected the `MdMk` records first and reported positions in
    /// THAT list would name record 0 here, which is the seed.
    #[test]
    fn the_indices_are_into_the_callers_list_not_a_filtered_one() {
        assert_eq!(
            mdmk_unconfirmed(&recs(&[SEED, encode_text("hi").as_str(), MD1_A])),
            vec![2]
        );
    }

    /// The arm the R0-I1 review named: `ValidMD` is a pure BCH verifier, so seed
    /// entropy wraps into a record that classifies `MdMk` — not secret, no flag.
    /// §12.6 is what closes it: the record does not decode, so it is unconfirmed,
    /// and an unconfirmed record counts as SECRET for flag evaluation.
    #[test]
    fn a_bch_valid_md1_carrying_entropy_does_not_decode_and_is_unconfirmed() {
        let smuggled = entropy_smuggled_as_md1(0xAB);
        assert_eq!(
            super::super::classify(&smuggled),
            Class::MdMk,
            "the premise: BCH validity alone makes this a non-secret class"
        );
        assert!(
            !Class::MdMk.is_secret(),
            "…and the class itself claims nothing"
        );
        assert_eq!(mdmk_unconfirmed(&recs(&[&smuggled])), vec![0]);
    }

    /// The non-chunked arm, and the mutant it kills: a walk that gave every
    /// non-chunked record the SAME group key would decode only the first of them
    /// and call the rest confirmed. Here the first decodes and the second does
    /// not, so collapsing the key silently confirms the smuggled entropy.
    #[test]
    fn two_non_chunked_md1_records_are_two_cards_not_one() {
        // 0x01, not 0xAB: this record must take the NON-CHUNKED path, or it
        // never reaches the grouping and the mutant this test exists for
        // survives. The Go port's mirror of this test found that the hard way.
        let bad = entropy_smuggled_as_md1(0x01);
        assert_eq!(
            mdmk_unconfirmed(&recs(&[MD1_SINGLE, &bad])),
            vec![1],
            "a non-chunked record is its own card; one decoding must not vouch for another"
        );
        assert_eq!(
            mdmk_unconfirmed(&recs(&[&bad, MD1_SINGLE])),
            vec![0],
            "and order must not decide it either"
        );
        assert_eq!(
            mdmk_unconfirmed(&recs(&[MD1_SINGLE])),
            Vec::<usize>::new(),
            "the one that decodes is confirmed on its own"
        );
    }

    #[test]
    fn mk1_sets_are_walked_the_same_way() {
        assert_eq!(
            mdmk_unconfirmed(&recs(&[MK1_A, MK1_B])),
            Vec::<usize>::new()
        );
        assert_eq!(mdmk_unconfirmed(&recs(&[MK1_A])), vec![0]);
        assert_eq!(mdmk_unconfirmed(&recs(&[MK1_LONE])), vec![0]);
        assert_eq!(
            mdmk_unconfirmed(&recs(&[MK1_A, MK1_B, MK1_LONE])),
            vec![2],
            "two mk1 cards, grouped apart by chunk_set_id"
        );
    }

    /// The fail-closed arm, which I had written off as unreachable until I tried
    /// to build an input for it. `validate_record` trims before validating and
    /// the decoders do not, so a record with a leading space classifies `MdMk`
    /// and then has no readable card identity at all. Unconfirmed is the only
    /// honest answer, and it is the one both walks give.
    #[test]
    fn a_record_whose_card_identity_cannot_be_read_is_unconfirmed() {
        let padded = format!(" {MD1_A}");
        assert_eq!(
            super::super::classify(&padded),
            Class::MdMk,
            "the premise: the classifier trims and the decoders do not"
        );
        assert!(
            md_codec::codex32::unwrap_string(&padded).is_err(),
            "…so nothing can read this record's chunk key"
        );
        assert_eq!(mdmk_unconfirmed(&recs(&[&padded])), vec![0]);
        assert!(
            crate::seal::record::decode_public_set(&[&padded]).is_err(),
            "and the frozen walk refuses it, which is the same answer"
        );
    }

    /// §12.6 is about `ClassMDMK` records and nothing else. A mnemonic is
    /// already secret by class and a `text:` record is already not; neither is
    /// this rule's business, and reporting them would double-count.
    #[test]
    fn records_of_other_classes_are_never_reported() {
        assert_eq!(
            mdmk_unconfirmed(&recs(&[SEED, encode_text("hi").as_str(), "ms1notacard"])),
            Vec::<usize>::new()
        );
        assert_eq!(mdmk_unconfirmed(&[]), Vec::<usize>::new());
    }

    /// The duplicated grouping, watched. `decode_public_set` refuses what this
    /// reports, so over a public-only record list the two must agree exactly:
    /// empty here iff `Ok` there. If the copy ever drifts, this fails rather
    /// than the device and the host quietly disagreeing about a card set.
    #[test]
    fn the_two_walks_agree_wherever_both_have_an_answer() {
        let smuggled = entropy_smuggled_as_md1(0xAB);
        let nc = entropy_smuggled_as_md1(0x01);
        let cases: Vec<Vec<&str>> = vec![
            vec![MD1_A],
            vec![MD1_A, MD1_B, MD1_C],
            vec![MD1_A, MD1_B, MD1_C, MD1_OTHER],
            vec![MD1_SINGLE],
            vec![MD1_SINGLE, &smuggled],
            vec![MD1_SINGLE, &nc],
            vec![&nc, MD1_SINGLE],
            vec![MK1_A, MK1_B],
            vec![MK1_A],
            vec![MK1_A, MK1_B, MD1_A, MD1_B, MD1_C],
            vec![MK1_A, MK1_B, MD1_SINGLE],
        ];
        let mut agreed_ok = 0;
        let mut agreed_err = 0;
        for c in cases {
            let sealable = crate::seal::record::decode_public_set(&c).is_ok();
            let confirmed = mdmk_unconfirmed(&recs(&c)).is_empty();
            assert_eq!(
                sealable, confirmed,
                "the two walks disagree about {c:?}: decode_public_set ok={sealable}, \
                 mdmk_unconfirmed empty={confirmed}"
            );
            if sealable {
                agreed_ok += 1;
            } else {
                agreed_err += 1;
            }
        }
        // Agreement on nothing but failures would be agreement by accident.
        assert!(
            agreed_ok >= 4 && agreed_err >= 3,
            "the comparison must exercise both answers: {agreed_ok} ok, {agreed_err} err"
        );
    }
}
