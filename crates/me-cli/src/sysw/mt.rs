//! `mt1` records in the systemwide container — classification and the
//! `[mt-decode]` confirmation walk.
//!
//! An `mt1` string carries a chunk of an **already-signed Bitcoin
//! transaction** (SPEC_mt_v0_1). It is a BEARER instrument — anyone holding
//! the complete set can broadcast — but it is NOT a secret class here: the
//! whole purpose of packing one is to engrave it in cleartext on steel, so
//! flash holds nothing the plate will not. What IS carried over from the
//! md/mk handling is the smuggling concern, and mt's version is worse:
//! reassembly alone confirms nothing, because any complete set of BCH-valid
//! strings "decodes" — there is no semantic decoder the way `md::Reassemble`
//! is one. So confirmation here is:
//!
//!   1. the set is COMPLETE and reassembles (`mt_codec::pipeline::decode`),
//!   2. the bytes PARSE as a serialized Bitcoin transaction ([`super::tx`]),
//!   3. the set's `chunk_set_id` equals the top 20 bits of the parsed
//!      transaction's display txid — the binding §10.13(c) of the mt spec
//!      builds into the format.
//!
//! A wrapped 32-byte seed fails (2); a forged transaction with an unrelated
//! header fails (3) at 1 in 2^20. An unconfirmed record counts as SECRET for
//! flag evaluation, exactly as `[mdmk-decode]` records do.
//!
//! **Strictness relative to `mt-codec`.** Classification uses a strict
//! verifier: exact BCH validity (no error correction) and consistent case.
//! `mt-codec`'s decoder corrects up to t = 4 and lowercases blindly — right
//! for hand-typed recovery, wrong for admitting a payload someone else wrote:
//! a record needing correction would be engraved verbatim, correcting steel
//! into carrying the damage. The device's `codex32.ValidMT` is the same
//! strictness; Rust is primary and the Go port converges on THIS module.

use mt_codec::consts::{CHECKSUM_SYMBOLS, HEADER_SYMBOLS, HRP, REGULAR_CODE_SYMBOLS_MAX};
use mt_codec::string_layer::bch::{bch_verify_regular, ALPHABET};
use mt_codec::string_layer::ChunkHeader;

use super::tx;

/// The shortest data part a structurally meaningful mt1 string can have:
/// the 11-symbol header plus the 13-symbol checksum.
const MIN_DATA_SYMBOLS: usize = HEADER_SYMBOLS + CHECKSUM_SYMBOLS;

/// Strict validity: is this record an `mt1` string the device would admit?
///
/// Trims (matching `seal::record::validate_record`'s asymmetry note — the
/// classifier trims everywhere in this container), refuses mixed case,
/// requires exact BCH validity and a parseable header.
pub fn valid_mt(record: &str) -> bool {
    data_symbols(record).is_some()
}

/// The parsed header of a strictly-valid mt1 record.
pub fn header(record: &str) -> Option<ChunkHeader> {
    let syms = data_symbols(record)?;
    ChunkHeader::from_symbols(&syms).ok()
}

/// Data-part symbols of a strictly-valid mt1 string, checksum included.
/// `None` for anything that is not one.
fn data_symbols(record: &str) -> Option<Vec<u8>> {
    let t = record.trim();
    // Consistent case only, like the device's codex32 engine: a mixed-case
    // string is refused rather than normalised, because a payload is not a
    // keyboard.
    if !(t == t.to_ascii_lowercase() || t == t.to_ascii_uppercase()) {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    let rest = lower.strip_prefix(HRP)?.strip_prefix('1')?;
    if rest.len() < MIN_DATA_SYMBOLS || rest.len() > REGULAR_CODE_SYMBOLS_MAX {
        return None;
    }
    let mut syms = Vec::with_capacity(rest.len());
    for c in rest.bytes() {
        let v = ALPHABET.iter().position(|&a| a == c)?;
        syms.push(v as u8);
    }
    if !bch_verify_regular(HRP, &syms) {
        return None;
    }
    // The header must parse (known version, index < count) or the record is
    // not a chunk of anything.
    ChunkHeader::from_symbols(&syms).ok()?;
    Some(syms)
}

/// WHY a chunk set did not confirm — the operator-facing diagnosis.
///
/// Ruling 2026-08-25 made "loudly" NORMATIVE, and more than the md/mk sibling
/// does: `mdmk_unconfirmed` returns indices and says nothing else. An mt1 set
/// fails for five distinguishable reasons whose remedies are not close — find
/// three more strings, re-encode from the transaction, or throw the payload
/// away — and a message that says only "could not confirm" leaves the operator
/// to guess which.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetProblem {
    /// The header declares `count` chunks and these 0-based indices are
    /// absent. THE FIX IS CHEAP: pack the missing strings. This is the case
    /// the ruling exists for — refusing would cost the operator a signing
    /// ceremony over a set that is merely short.
    Missing { count: usize, missing: Vec<usize> },
    /// Every index is present and the strings still do not reassemble —
    /// duplicates that disagree, or a chunk `mt_codec` reads as ambiguous.
    DoesNotReassemble,
    /// Reassembles, and the bytes are not one serialized transaction. The C3
    /// smuggling channel, and the reason this walk parses at all.
    NotATransaction,
    /// Parses, and the derived txid does not carry the set id every chunk
    /// declares. 1 in 2^20 by accident; deliberate the rest of the time.
    TxidDoesNotBind { txid: String, csid: u32 },
    /// Parses and binds, and an input carries neither a scriptSig nor a
    /// witness. **Nothing else in this walk can see it**: stripping the
    /// witnesses leaves the txid unchanged, so the binding still holds.
    UnsignedInputs { txid: String, inputs: Vec<usize> },
}

/// Every mt1 chunk set in `records`, keyed by `chunk_set_id`, with the record
/// indices that belong to it and — when it did not confirm — why.
///
/// ONE grouping, used by [`mt_unconfirmed`], by `pack`'s stderr report and by
/// `me sysw show`. Three copies of "which sets are there" is how a report and
/// a refusal come to disagree about the same payload.
pub fn set_problems(records: &[String]) -> Vec<(u32, Vec<usize>, Option<SetProblem>)> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (i, r) in records.iter().enumerate() {
        if super::classify(r) != super::record::Class::Mt {
            continue;
        }
        if let Some(h) = header(r) {
            groups.entry(h.chunk_set_id).or_default().push(i);
        }
    }
    groups
        .into_iter()
        .map(|(csid, idxs)| {
            let set: Vec<String> = idxs.iter().map(|&i| records[i].clone()).collect();
            let problem = diagnose(csid, &set);
            (csid, idxs, problem)
        })
        .collect()
}

/// `None` exactly when [`set_confirmed`] is true — asserted by
/// `the_diagnosis_and_the_verdict_are_one_answer`, because two predicates over
/// one condition is the defect shape this module already carries a note about.
fn diagnose(csid: u32, set: &[String]) -> Option<SetProblem> {
    // COMPLETENESS FIRST, from the headers alone. It needs no reassembly,
    // which is the point: it is the one failure whose remedy is cheap, and it
    // must not be reported as whatever `mt_codec` happens to say about a set
    // that is merely short.
    let mut count = 0usize;
    let mut present: Vec<usize> = Vec::with_capacity(set.len());
    for r in set {
        if let Some(h) = header(r) {
            count = count.max(h.count);
            present.push(h.index);
        }
    }
    let missing: Vec<usize> = (0..count).filter(|i| !present.contains(i)).collect();
    if !missing.is_empty() {
        return Some(SetProblem::Missing { count, missing });
    }
    let Ok(decoded) = mt_codec::pipeline::decode(set) else {
        return Some(SetProblem::DoesNotReassemble);
    };
    if decoded.chunks.iter().any(|c| c.corrected != 0) || !decoded.unreadable.is_empty() {
        return Some(SetProblem::DoesNotReassemble);
    }
    let Ok(summary) = tx::parse(&decoded.bytes) else {
        return Some(SetProblem::NotATransaction);
    };
    if !summary.every_input_signed {
        return Some(SetProblem::UnsignedInputs {
            txid: summary.txid_display.clone(),
            inputs: summary.unsigned_inputs,
        });
    }
    if summary.chunk_set_id() != csid {
        return Some(SetProblem::TxidDoesNotBind {
            txid: summary.txid_display,
            csid,
        });
    }
    None
}

/// Indices of [`super::record::Class::Mt`] records that are NOT
/// decode-confirmed — the `[mt-decode]` walk. Mirrors
/// [`super::record::mdmk_unconfirmed`]'s contract exactly:
///
/// - grouped by `chunk_set_id` (every mt1 is chunked; there is no
///   non-chunked form and therefore no `uniq` arm),
/// - indices are into the CALLER'S slice, whatever else it holds,
/// - nothing is refused — the caller flags, per §13 D6's demotion.
pub fn mt_unconfirmed(records: &[String]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    for (i, r) in records.iter().enumerate() {
        // Fail closed. Unreachable today — classify and header run the same
        // strict parse — but kept so a future classifier loosening cannot
        // silently confirm what it cannot read.
        if super::classify(r) == super::record::Class::Mt && header(r).is_none() {
            out.push(i);
        }
    }
    for (_, idxs, problem) in set_problems(records) {
        if problem.is_some() {
            out.extend(idxs);
        }
    }
    out.sort_unstable();
    out
}

/// The three-step confirmation. `csid` is the group key every member carries.
fn set_confirmed(csid: u32, set: &[String]) -> bool {
    let Ok(decoded) = mt_codec::pipeline::decode(set) else {
        return false;
    };
    // Strictly-valid strings never need correction; a decode that used the
    // correction budget means the walk was handed something classify did not
    // admit. Refuse rather than trust it.
    if decoded.chunks.iter().any(|c| c.corrected != 0) || !decoded.unreadable.is_empty() {
        return false;
    }
    let Ok(summary) = tx::parse(&decoded.bytes) else {
        return false;
    };
    // (G-P3.1) EVERY INPUT MUST CARRY A SIGNATURE, and this class needs the
    // check as much as `tx:` does -- more, because a chunk set is the path a
    // transaction takes when it is too large for one record, i.e. the usual
    // one. Stripping the witnesses does not change the txid, so the binding
    // below passes on a stripped set exactly as it does on the honest one:
    // every other check in this function is blind to it.
    if !summary.every_input_signed {
        return false;
    }
    summary.chunk_set_id() == csid
}

/// Decode a confirmed set to `(transaction bytes, summary)` — what the host's
/// `show` prints and the device's review screen displays. `None` exactly when
/// [`mt_unconfirmed`] would report the set.
pub fn decode_confirmed(set: &[String]) -> Option<(Vec<u8>, tx::TxSummary)> {
    let decoded = mt_codec::pipeline::decode(set).ok()?;
    if decoded.chunks.iter().any(|c| c.corrected != 0) || !decoded.unreadable.is_empty() {
        return None;
    }
    let summary = tx::parse(&decoded.bytes).ok()?;
    // (G-P3.1) Mirrors `set_confirmed`; the doc comment above promises these
    // two agree exactly, and an unsigned set must not decode to a candidate.
    if !summary.every_input_signed {
        return None;
    }
    if summary.chunk_set_id() != decoded.chunks.first()?.header.chunk_set_id {
        return None;
    }
    Some((decoded.bytes, summary))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysw::record::Class;

    /// The "even" vector from mt-codec's pinned corpus
    /// (`src/test_vectors/mt1_v1.json`): a real signed 222-byte transaction,
    /// 6 chunks of 37 bytes, chunk_set_id 0x2dcf2. Strings VERBATIM from the
    /// corpus, which `scripts/gen-mt1-vectors.py` produced independently of
    /// mt-codec — so this fixture can falsify the encoder it is checked
    /// against below.
    const EVEN: [&str; 6] = [
        "mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax",
        "mt1p9h8jqq9qqphgdqqqqqqqq0mllllupyqj6vqqqqqqqqzcqpfsw7ph2rt5w54kt768636cls8zxg0najlzunp",
        "mt1p9h8jqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qd8pwskxfl7ju5",
        "mt1p9h8jqq9qqrqfrnq3qzyp77h37cnxzvwutegzmzy5zrrrfvrpykdfsckvk03dcq6rcjtvlsfcglv7zx43yaz",
        "mt1p9h8jqq9qqylgpzqmhcwhuupdvnrc82rncvzzdahpgjsdwgu52jd7vmxsve9x3w5ujeqyssuvddxvwqze4ve",
        "mt1p9h8jqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq5k5y2ye5nv8yf",
    ];

    /// The pinned set, cross-checked against a fresh encode of the vector's
    /// raw bytes so the two can never drift apart silently.
    fn even_set() -> Vec<String> {
        let bytes = crate::sysw::tx::tests::unhex(crate::sysw::tx::tests::EVEN_RAW_HEX);
        let set = mt_codec::pipeline::encode(&bytes, crate::sysw::tx::tests::EVEN_TXID).unwrap();
        assert_eq!(set, EVEN.map(String::from).to_vec(), "encoder diverges from the pinned vector");
        set
    }

    #[test]
    fn a_real_mt1_string_is_strictly_valid_and_classified() {
        for s in &even_set() {
            assert!(valid_mt(s), "{s}");
            assert_eq!(crate::sysw::classify(s), Class::Mt);
        }
        // Uppercase consistently: valid. Mixed: refused.
        let up = even_set()[0].to_ascii_uppercase();
        assert!(valid_mt(&up));
        let mut mixed = even_set()[0].clone();
        mixed.replace_range(0..1, "M");
        assert!(!valid_mt(&mixed), "mixed case must be refused");
    }

    #[test]
    fn one_flipped_character_is_not_corrected_into_validity() {
        let s = &even_set()[0];
        let flipped = format!(
            "{}{}",
            &s[..s.len() - 1],
            if s.ends_with('x') { 'y' } else { 'x' }
        );
        assert!(
            !valid_mt(&flipped),
            "classification must be exact BCH validity, never correction"
        );
        assert_eq!(crate::sysw::classify(&flipped), Class::Unknown);
    }

    #[test]
    fn the_complete_set_is_confirmed_and_partial_sets_are_not() {
        let set = even_set();
        assert_eq!(mt_unconfirmed(&set), Vec::<usize>::new());
        // Any one chunk missing: every remaining member is unconfirmed.
        let partial: Vec<String> = set[..5].to_vec();
        assert_eq!(mt_unconfirmed(&partial), vec![0, 1, 2, 3, 4]);
        // A single chunk alone.
        assert_eq!(mt_unconfirmed(&set[..1]), vec![0]);
    }

    #[test]
    fn indices_are_into_the_callers_list_not_a_filtered_one() {
        let set = even_set();
        let mixed = vec![
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string(),
            set[0].clone(),
        ];
        assert_eq!(mt_unconfirmed(&mixed), vec![1]);
    }

    /// The smuggling arm this module exists for: bytes that reassemble but do
    /// not PARSE. Encode 32 bytes of entropy as a (BCH-valid, complete,
    /// 1-chunk) mt1 set — reassembly succeeds, tx::parse refuses, unconfirmed.
    #[test]
    fn entropy_wrapped_as_a_complete_mt_set_is_unconfirmed() {
        let set = mt_codec::pipeline::encode(&[0xAB; 32], "deadbeef00").unwrap();
        assert_eq!(set.len(), 1, "32 bytes is one chunk");
        assert_eq!(crate::sysw::classify(&set[0]), Class::Mt);
        assert_eq!(mt_unconfirmed(&set), vec![0]);
    }

    /// The forgery arm: a REAL transaction encoded under the WRONG
    /// chunk_set_id. Reassembles, parses — and fails the txid binding.
    #[test]
    fn a_real_tx_under_a_foreign_set_id_is_unconfirmed() {
        let bytes = crate::sysw::tx::tests::unhex(crate::sysw::tx::tests::EVEN_RAW_HEX);
        let forged = mt_codec::pipeline::encode(&bytes, "00000feed").unwrap();
        assert_eq!(forged.len(), 6);
        assert_eq!(mt_unconfirmed(&forged), vec![0, 1, 2, 3, 4, 5]);
    }

    /// Two sets in one payload are grouped apart by chunk_set_id: the complete
    /// one confirms beside the lone stranger.
    #[test]
    fn grouping_is_by_chunk_set_id() {
        let mut records = even_set();
        let stranger = mt_codec::pipeline::encode(&[0x01; 80], "fffff00000").unwrap();
        assert!(stranger.len() >= 2);
        records.push(stranger[0].clone());
        assert_eq!(mt_unconfirmed(&records), vec![6]);
    }

    #[test]
    fn decode_confirmed_returns_the_bytes_and_the_summary() {
        let set = even_set();
        let (bytes, summary) = decode_confirmed(&set).unwrap();
        assert_eq!(
            bytes,
            crate::sysw::tx::tests::unhex(crate::sysw::tx::tests::EVEN_RAW_HEX)
        );
        assert_eq!(summary.txid_display, crate::sysw::tx::tests::EVEN_TXID);
        assert!(decode_confirmed(&set[..5]).is_none());
    }

    #[test]
    fn mt_is_not_a_secret_class() {
        // The plate is cleartext by design; flash holds nothing the steel will
        // not. Bearer-ness is a MESSAGING posture (mt-cli's), not a secrecy
        // class — but an UNCONFIRMED record still reads as secret via flags.
        assert!(!Class::Mt.is_secret());
    }

    /// G-P3.7. The DIAGNOSIS, per set, and it must distinguish the five
    /// reasons a set fails — their remedies are not close (find three more
    /// strings / re-encode from the transaction / throw the payload away) and
    /// a message that says only "could not confirm" leaves the operator to
    /// guess which one they are in.
    #[test]
    fn every_way_a_set_can_fail_is_named_separately() {
        use crate::sysw::tx::tests::{unhex, EVEN_RAW_HEX, EVEN_STRIPPED_HEX};

        // (0) Confirmed: no problem at all.
        let ok = set_problems(&even_set());
        assert_eq!(ok.len(), 1);
        assert_eq!(ok[0].0, 0x2dcf2);
        assert_eq!(ok[0].2, None);

        // (1) MISSING, by index and against the header's own count. Two gone,
        // not adjacent, so "the first missing one" is visibly not the answer.
        let set = even_set();
        let short: Vec<String> = vec![set[0].clone(), set[2].clone(), set[5].clone()];
        let p = set_problems(&short);
        assert_eq!(
            p[0].2,
            Some(SetProblem::Missing { count: 6, missing: vec![1, 3, 4] })
        );

        // (2) NOT A TRANSACTION — the C3 smuggling channel: 32 bytes of
        // entropy as a complete 1-chunk set.
        let smuggled = mt_codec::pipeline::encode(&[0xAB; 32], "deadbeef00").unwrap();
        assert_eq!(set_problems(&smuggled)[0].2, Some(SetProblem::NotATransaction));

        // (3) UNSIGNED — parses, and binds, because stripping the witnesses
        // leaves the txid alone. Nothing else in the walk can see it.
        let bytes = unhex(EVEN_STRIPPED_HEX);
        let txid = tx::parse(&bytes).unwrap().txid_display;
        let stripped = mt_codec::pipeline::encode(&bytes, &txid).unwrap();
        assert_eq!(
            set_problems(&stripped)[0].2,
            Some(SetProblem::UnsignedInputs { txid: txid.clone(), inputs: vec![0] })
        );

        // (4) TXID DOES NOT BIND — a real signed transaction under a foreign
        // set id.
        let forged = mt_codec::pipeline::encode(&unhex(EVEN_RAW_HEX), "00000feed").unwrap();
        assert_eq!(
            set_problems(&forged)[0].2,
            Some(SetProblem::TxidDoesNotBind {
                txid: crate::sysw::tx::tests::EVEN_TXID.to_string(),
                csid: 0x00000,
            })
        );
    }

    /// The diagnosis and the verdict are ONE answer. Two predicates over one
    /// condition is how a report and a refusal come to disagree about the same
    /// payload, and this module already carries a note about that shape.
    #[test]
    fn the_diagnosis_and_the_verdict_are_one_answer() {
        use crate::sysw::tx::tests::{unhex, EVEN_RAW_HEX, EVEN_STRIPPED_HEX};
        let stripped = unhex(EVEN_STRIPPED_HEX);
        let stripped_txid = tx::parse(&stripped).unwrap().txid_display;
        let corpora: Vec<Vec<String>> = vec![
            even_set(),
            even_set()[..3].to_vec(),
            mt_codec::pipeline::encode(&[0xAB; 32], "deadbeef00").unwrap(),
            mt_codec::pipeline::encode(&stripped, &stripped_txid).unwrap(),
            mt_codec::pipeline::encode(&unhex(EVEN_RAW_HEX), "00000feed").unwrap(),
        ];
        for set in corpora {
            let unconfirmed = mt_unconfirmed(&set);
            for (csid, idxs, problem) in set_problems(&set) {
                assert_eq!(
                    problem.is_none(),
                    set_confirmed(csid, &idxs.iter().map(|&i| set[i].clone()).collect::<Vec<_>>()),
                    "diagnosis and set_confirmed disagree about set {csid:05x}"
                );
                // ...and mt_unconfirmed reports exactly the sets with problems.
                for i in &idxs {
                    assert_eq!(unconfirmed.contains(i), problem.is_some());
                }
            }
        }
    }

    /// Two sets in one payload are diagnosed SEPARATELY. A report that folds
    /// them into one line cannot tell the operator which strings to go find.
    #[test]
    fn two_broken_sets_are_two_diagnoses() {
        let mut records = even_set()[..4].to_vec();
        let other = mt_codec::pipeline::encode(&[0x01; 80], "fffff00000").unwrap();
        assert!(other.len() >= 2);
        records.push(other[0].clone());
        let p = set_problems(&records);
        assert_eq!(p.len(), 2, "two chunk_set_ids, two diagnoses");
        let by_id: std::collections::BTreeMap<u32, &Option<SetProblem>> =
            p.iter().map(|(c, _, pr)| (*c, pr)).collect();
        assert_eq!(
            by_id[&0x2dcf2],
            &Some(SetProblem::Missing { count: 6, missing: vec![4, 5] })
        );
        assert!(matches!(by_id[&0xfffff], Some(SetProblem::Missing { .. })));
    }

    /// RED FIRST (G-P3.1). The signature predicate guarded the `tx:` class and
    /// NOT this one -- and the mt1 chunk set is the path a transaction takes
    /// when it is too large for a single record, i.e. the primary one.
    ///
    /// A stripped set reassembles, parses, and binds to its chunk_set_id,
    /// because the txid is UNCHANGED by stripping. Every check this module had
    /// passes it.
    #[test]
    fn a_stripped_transaction_as_a_chunk_set_is_unconfirmed() {
        use crate::sysw::tx::tests::{unhex, EVEN_STRIPPED_HEX};
        let bytes = unhex(EVEN_STRIPPED_HEX);
        let txid = crate::sysw::tx::parse(&bytes).unwrap().txid_display;
        let set = mt_codec::pipeline::encode(&bytes, &txid).unwrap();

        // The premise: every OTHER check passes. It is a real, complete,
        // pristine set that reassembles to bytes that parse and bind.
        for c in &set {
            assert_eq!(crate::sysw::classify(c), Class::Mt, "each chunk is valid");
        }

        assert_eq!(
            mt_unconfirmed(&set),
            (0..set.len()).collect::<Vec<_>>(),
            "a set carrying an UNSIGNED transaction must not confirm -- the txid \
             is identical to the honest transaction's, so nothing else can tell"
        );
        assert!(decode_confirmed(&set).is_none());
    }

}
