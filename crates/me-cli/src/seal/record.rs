//! Per-record validation, and the §6.3 card-set decode for the public section.

use crate::classify::{classify, Format};
use crate::validate::first_noncanonical;

/// §10.2.1a — the longest `codex32` secret the SeedHammer II can engrave.
///
/// Derived from `backup.EngraveSeedString`, which builds the seed plate's QR
/// from the UPPERCASED share at error-correction level M and refuses
/// `qrc.Size > 33`. Measured against the real encoder the boundary is exact and
/// sits between 90 and 91:
///
/// ```text
///  63–90  → QR size 33 → cuts
///  91–122 → QR size 37 → refused
/// 123–127 → QR size 41 → refused
/// ```
///
/// **This is a LITERAL here and cannot be re-derived in Rust — there is no QR
/// encoder in this crate.** The authority is the Go test
/// `TestEngraveableLimitIsDerivedFromTheRealQREncoder`
/// (`backup/engraveable_test.go` in the seedhammer fork), which re-runs
/// `qr.Encode(strings.ToUpper(s), qr.M)` against the real encoder and fails if
/// the boundary is no longer 90/91. If that test moves the number, this
/// constant moves with it. Two things can move it independently: the
/// `qrc.Size > 33` cap and the error-correction level — measured, at `qr.Q`
/// instead of `qr.M` the limit drops to 67, which is *below* ordinary
/// `EncodeMS1` output and would both reopen this defect and reject real seeds.
pub const MAX_ENGRAVEABLE_MS1_LEN: usize = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    /// md1 — wallet policy. Public.
    Md,
    /// mk1 — xpub + origin. Public.
    Mk,
    /// ms1 — the seed. Secret.
    Ms,
}

impl RecordKind {
    pub fn is_secret(self) -> bool {
        matches!(self, RecordKind::Ms)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecordError {
    NonCanonical {
        ch: char,
        pos: usize,
    },
    NotLowercase(usize),
    /// A BIP-39 mnemonic whose word separators the device cannot split on.
    ///
    /// A SEPARATE variant for the same reason as `MsTooLong`: this record is not
    /// unreadable, it is intact and spaced in a way the device's parser does not
    /// accept. Rendering it as §6.4's "payload unreadable" would tell an
    /// operator holding a perfectly good backup that it had been tampered with.
    NonCanonicalSpace(usize),
    Unclassifiable(String),
    Invalid(String),
    UndecodableSet(String),
    /// §10.2.1a — a `codex32` secret the engraver's seed plate cannot hold.
    ///
    /// A SEPARATE variant, not an `Invalid`, and that is the requirement rather
    /// than tidiness: §6.4 renders every other record failure to the operator as
    /// "payload unreadable", and this record is not unreadable — it is intact
    /// and too long to cut. Collapsing the two tells an operator with a
    /// perfectly good backup that it has been tampered with.
    MsTooLong(usize),
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordError::NonCanonical { ch, pos } => write!(
                f,
                "non-canonical record: separator {ch:?} at byte {pos} — records must be \
                 unbroken. If this came from `mnemonic bundle`, re-run with --group-size 0: \
                 the default --group-size 5 emits a DISPLAY form the engraver cannot read."
            ),
            RecordError::NotLowercase(pos) => write!(
                f,
                "record has an uppercase character at byte {pos} — records must be lowercase, \
                 or the same wallet has two different public-data hashes (§6.4)"
            ),
            RecordError::NonCanonicalSpace(pos) => write!(
                f,
                "mnemonic has a non-canonical word separator at byte {pos} — words must be \
                 separated by exactly one ASCII space, with none leading or trailing. The \
                 device splits on a single space and would refuse this backup AFTER the ~31 s \
                 key derivation, reporting it as unreadable (§6.4). Re-run with the words \
                 single-spaced."
            ),
            RecordError::Unclassifiable(e) => write!(f, "unrecognised record: {e}"),
            RecordError::Invalid(e) => write!(f, "invalid record: {e}"),
            RecordError::UndecodableSet(e) => write!(
                f,
                "public records do not form a decodable card set: {e} — a BCH-valid string is \
                 not proof of a real wallet card (§6.3)"
            ),
            RecordError::MsTooLong(len) => write!(
                f,
                "this codex32 secret is {len} characters; the machine can engrave at most \
                 {MAX_ENGRAVEABLE_MS1_LEN} (§10.2.1a). The record is INTACT — it is too long \
                 to cut, not unreadable. `me seal` refuses it here so the device does not \
                 refuse it after a ~31 s key derivation."
            ),
        }
    }
}
impl std::error::Error for RecordError {}

/// Validate one record: canonical, lowercase, correct BCH checksum. Reports what
/// it is. Does NOT decode — see `decode_public_set`.
pub fn validate_record(s: &str) -> Result<RecordKind, RecordError> {
    let s = s.trim();
    if let Some((pos, ch)) = first_noncanonical(s) {
        return Err(RecordError::NonCanonical { ch, pos });
    }
    if let Some(pos) = s
        .char_indices()
        .find(|(_, c)| c.is_uppercase())
        .map(|(i, _)| i)
    {
        return Err(RecordError::NotLowercase(pos));
    }
    let fmt = classify(s).map_err(|e| RecordError::Unclassifiable(e.to_string()))?;
    match fmt {
        Format::Md => md_codec::codex32::unwrap_string(s)
            .map(|_| RecordKind::Md)
            .map_err(|e| RecordError::Invalid(e.to_string())),
        Format::Mk => {
            let d = mk_codec::string_layer::decode_string(s)
                .map_err(|e| RecordError::Invalid(e.to_string()))?;
            if d.corrections_applied != 0 {
                return Err(RecordError::Invalid(format!(
                    "not pristine: required {} BCH correction(s)",
                    d.corrections_applied
                )));
            }
            Ok(RecordKind::Mk)
        }
        Format::Mt => Err(RecordError::Invalid(
            "mt1 records belong to the systemwide container (`me sysw pack`), not the \
             frozen sealed payload"
                .into(),
        )),
        Format::Ms => {
            // §10.2.1a, and it MUST run BEFORE `ms_codec::decode`. The crate's
            // own length gate fires first otherwise, and it reports
            // `UnexpectedStringLength` — which renders through `Invalid`, i.e.
            // as §6.4's "payload unreadable". That is precisely the
            // misdiagnosis this rule exists to remove: the operator's backup is
            // intact, it is the plate that cannot hold it.
            //
            // GATED ON THE BIP-93 PARSE, not on the HRP alone, so `ms1` +
            // garbage is still reported as invalid rather than as too long.
            // That mirrors the device, where `Classify` reaches this check only
            // via `codex32.New`. The parse is attempted only once the length has
            // already failed, so the ordinary path allocates nothing extra.
            //
            // `Codex32String` derives `zeroize::ZeroizeOnDrop` and takes the
            // `String` by value, so the copy this makes IS scrubbed — on the
            // success path. On its error path the copy is dropped unscrubbed,
            // but a string that fails the codex32 checksum is not a seed.
            let len = s.chars().count();
            if len > MAX_ENGRAVEABLE_MS1_LEN
                && ms_codec::codex32::Codex32String::from_string(s.to_string()).is_ok()
            {
                return Err(RecordError::MsTooLong(len));
            }
            // ms_codec::decode, NOT decode_with_correction — a seed that needed
            // repair must be fixed at source, not engraved.
            ms_codec::decode(s)
                .map(|_| RecordKind::Ms)
                .map_err(|e| RecordError::Invalid(e.to_string()))
        }
    }
}

/// §6.3: every public record must belong to a card set that REASSEMBLES AND
/// DECODES. Records are chunks, so this is necessarily a whole-set operation —
/// a per-record decode rejects every legitimate payload.
///
/// **Group by `(HRP, chunk_set_id)`, NEVER by HRP alone.** A 2-of-3
/// `wsh-sortedmulti` wallet has THREE separate `mk1` cards and three `md1`
/// cards, each chunked independently. Lumping all six `mk1` records into one
/// group gives `received 6 chunks, header declares total_chunks = 2` — so HRP
/// grouping rejects **every multisig wallet**, which is the shape §6.4's
/// "why not 7" section exists to admit. Vectors D and E carry one card per HRP
/// and F is `pub_len = 0`, so nothing but vector G catches this.
///
/// A record that is NOT chunked is its own card and takes the single-string
/// path; neither path handles both forms (§6.3).
pub fn decode_public_set(records: &[&str]) -> Result<(), RecordError> {
    use std::collections::BTreeMap;
    // key: (hrp, chunk_set_id) — `None` csid means a non-chunked record, which
    // is its own card, so it gets a unique key via its index.
    let mut groups: BTreeMap<(char, Option<u32>, usize), Vec<&str>> = BTreeMap::new();
    for (i, r) in records.iter().enumerate() {
        let kind = validate_record(r)?;
        if kind.is_secret() {
            // Guarded by the caller (§6.3 forbids a secret in the public
            // section); reaching here is a caller bug, not a bad payload.
            return Err(RecordError::UndecodableSet(
                "a secret record cannot be in the public section".into(),
            ));
        }
        let (hrp, csid) = chunk_key(r, kind)?;
        let uniq = if csid.is_some() { 0 } else { i + 1 };
        groups.entry((hrp, csid, uniq)).or_default().push(r);
    }
    for ((hrp, csid, _), set) in groups {
        // Stringify INSIDE each arm: md_codec::Error and mk_codec::Error are
        // different types, so a match unifying them is E0308.
        let res: Result<(), String> = match (hrp, csid) {
            ('d', Some(_)) => md_codec::reassemble(&set)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            ('d', None) => md_codec::decode_md1_string(set[0])
                .map(|_| ())
                .map_err(|e| e.to_string()),
            ('k', _) => mk_codec::decode(&set)
                .map(|_| ())
                .map_err(|e| e.to_string()),
            _ => unreachable!("secret records are refused above"),
        };
        res.map_err(|e| RecordError::UndecodableSet(format!("{hrp}-card: {e}")))?;
    }
    Ok(())
}

/// Card identity: the HRP discriminant plus the 20-bit chunk-set id, or `None`
/// when the record is not chunked (and is therefore its own card).
///
/// **Verified against the real crates on vector G's records** — both paths are
/// public and both distinguish chunked from single-string, which is also what
/// §6.3's non-chunked dispatch needs.
/// `pub(crate)` by operator ruling 2026-08-12, carving a VISIBILITY-ONLY
/// exception into decision 1's freeze on this module. `sysw` needs exactly this
/// grouping and had been forced to keep a second copy of it; two implementations
/// of one rule is the defect shape that already cost this module a silent
/// dropped-secret bug. Nothing about the body, the signature or the emitted
/// bytes changes, so Sealed Payload's behaviour cannot move.
pub(crate) fn chunk_key(s: &str, kind: RecordKind) -> Result<(char, Option<u32>), RecordError> {
    use md_codec::bitstream::BitReader;
    use md_codec::ChunkHeader;
    use mk_codec::string_layer::{decode_string, StringLayerHeader};

    match kind {
        RecordKind::Md => {
            let (bytes, _bits) = md_codec::codex32::unwrap_string(s)
                .map_err(|e| RecordError::Invalid(e.to_string()))?;
            let mut r = BitReader::new(&bytes);
            // A non-chunked md1 fails the chunked-flag read; that is the signal,
            // not an error.
            Ok(('d', ChunkHeader::read(&mut r).ok().map(|h| h.chunk_set_id)))
        }
        RecordKind::Mk => {
            let d = decode_string(s).map_err(|e| RecordError::Invalid(e.to_string()))?;
            let (h, _) = StringLayerHeader::from_5bit_symbols(d.data())
                .map_err(|e| RecordError::Invalid(e.to_string()))?;
            Ok((
                'k',
                match h {
                    StringLayerHeader::Chunked { chunk_set_id, .. } => Some(chunk_set_id),
                    StringLayerHeader::SingleString { .. } => None,
                    // StringLayerHeader is #[non_exhaustive], so this arm is
                    // MANDATORY — it will not compile without it. Fail closed: an
                    // unrecognised header variant on a security path must never be
                    // silently grouped with anything.
                    _ => {
                        return Err(RecordError::UndecodableSet(
                            "unrecognised mk1 string-layer header variant".into(),
                        ))
                    }
                },
            ))
        }
        RecordKind::Ms => unreachable!("secret records are refused by the caller"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD1: [&str; 3] = [
        "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3",
        "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374",
        "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz",
    ];
    const MK1: [&str; 2] = [
        "mk1qpz63tpqqsq3dg4m5wdx5fvqqvzg3vs7mpf0rz2j43zpzpxk0rtjkqkhwreqp6hm7qnp3a8wdvtz6t2k4uxu6ykwxcp9vqugfjyx733cf59g",
        "mk1qpz63tppkeg9pdvqz5744004gvzecsknw6tu25yv3exfhkl6w5zm9e4t24aqdah5585wn3e4xdut8",
    ];
    const MS1: &str = "ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw";

    /// §10.2.1a boundary vectors — REAL BIP-93 codex32 secrets, generated in
    /// the seedhammer fork rather than hand-written:
    ///
    /// ```text
    /// head -c N /dev/zero | go run ./cmd/biptool seed -seedlen N -id entr
    /// ```
    ///
    /// N = 42, 43, 44, 63, 64 bytes → 90, 91, 93, 125, 127 characters.
    /// 92 is NOT constructible: a short code is `9 + ceil(8N/5) + 13` characters,
    /// which steps 90 → 91 → 93. 124 is in the dead zone between codex32's two
    /// length bands (short 48–93, long 125–127) and `Codex32String::from_string`
    /// rejects it outright, so it cannot exercise this rule at all.
    ///
    /// Each test that uses these asserts their length first — a mistyped vector
    /// must fail loudly, not silently stop testing the boundary.
    const MS1_90: &str =
        "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqutd7mdh2lc8h2";
    const MS1_91: &str =
        "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq2uk6ly9a0dmw4";
    const MS1_93: &str =
        "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqmtf88e60hz9eu";
    const MS1_125: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqt042k235w95p5rd";
    const MS1_127: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqmk6rc3gq4c88nvp";

    /// §10.2.1a: a `codex32` secret longer than the seed plate's QR can hold is
    /// refused, and refused with its OWN error — not `Invalid`, which is what
    /// §6.4 renders as "payload unreadable".
    #[test]
    fn refuses_an_ms1_longer_than_the_machine_can_engrave() {
        for (s, want) in [
            (MS1_91, 91usize),
            (MS1_93, 93),
            (MS1_125, 125),
            (MS1_127, 127),
        ] {
            assert_eq!(s.chars().count(), want, "vector is not {want} characters");
            assert_eq!(
                validate_record(s),
                Err(RecordError::MsTooLong(want)),
                "a {want}-character codex32 secret must be refused by §10.2.1a"
            );
        }
    }

    /// 90 characters is the LAST length `backup.EngraveSeedString` can cut, so
    /// §10.2.1a must not fire on it.
    ///
    /// **HONEST GAP, asserted rather than hidden.** This asserts *not
    /// `MsTooLong`*, not `Ok`. `ms-codec`'s accept set is the constellation ms1
    /// v0.1/v0.2 length set — `VALID_STR_LENGTHS` = [50, 56, 62, 69, 75] and
    /// `VALID_MNEM_STR_LENGTHS` = [51, 58, 64, 70, 77] — so it tops out at 77
    /// and no 90-character BIP-93 codex32 is a valid *constellation* record at
    /// all. `me` therefore still refuses this vector, for a separate,
    /// pre-existing reason that has nothing to do with plate geometry. The
    /// device's `codex32.New` has no such narrowing, which is why the Go test
    /// can — and does — assert outright admission at 90.
    #[test]
    fn does_not_fire_at_the_ninety_character_boundary() {
        assert_eq!(MS1_90.chars().count(), 90, "vector is not 90 characters");
        let got = validate_record(MS1_90);
        assert_ne!(
            got,
            Err(RecordError::MsTooLong(90)),
            "90 characters still cuts; §10.2.1a must not fire on it"
        );
        assert!(
            matches!(got, Err(RecordError::Invalid(_))),
            "expected ms-codec's own v0.1 length refusal, got {got:?}"
        );
    }

    /// The rejection MUST be distinguishable from "payload unreadable" (§6.4):
    /// the operator has to learn the record is too long to cut, not that their
    /// payload is corrupt. Length and classification are authenticated
    /// plaintext, so naming them leaks nothing.
    #[test]
    fn the_too_long_message_names_the_length_and_the_cap() {
        let msg = RecordError::MsTooLong(127).to_string();
        assert!(
            msg.contains("127"),
            "message must name the actual length: {msg}"
        );
        assert!(
            msg.contains(&MAX_ENGRAVEABLE_MS1_LEN.to_string()),
            "message must name the cap: {msg}"
        );
        assert!(
            msg.contains("engrave"),
            "message must say it is an engraving limit: {msg}"
        );
    }

    /// `ms1` ONLY — and only when the string really is a codex32 secret.
    /// `ms1` + garbage must still read as invalid, exactly as it does on the
    /// device, where `Classify` reaches the length check only through
    /// `codex32.New`.
    #[test]
    fn does_not_fire_on_a_long_non_codex32_ms1_string() {
        let junk: String = format!("ms1{}", "q".repeat(120));
        assert_eq!(junk.chars().count(), 123);
        assert!(
            !matches!(validate_record(&junk), Err(RecordError::MsTooLong(_))),
            "a string that is not valid codex32 is unreadable, not too long"
        );
    }

    /// Scope: `mdmkText` is deliberately NOT covered (§10.2.1a). md/mk records
    /// chunk, and their plates have variants; a long one is not the same
    /// hazard. Nothing here may start rejecting them by length.
    #[test]
    fn does_not_cover_md_or_mk_records() {
        // MK1[0] is over 90 characters and must still be admitted.
        assert!(MK1[0].chars().count() > MAX_ENGRAVEABLE_MS1_LEN);
        assert_eq!(validate_record(MK1[0]).unwrap(), RecordKind::Mk);
    }

    /// The constant is a LITERAL in Rust and this test says so.
    ///
    /// There is no QR encoder in this crate, so 90 cannot be re-derived here.
    /// The authority is the Go test
    /// `TestEngraveableLimitIsDerivedFromTheRealQREncoder`
    /// (`backup/engraveable_test.go`, seedhammer fork), which re-runs
    /// `qr.Encode(strings.ToUpper(s), qr.M)` and fails if the boundary is no
    /// longer 90/91. Measured there: at `qr.Q` the limit drops to 67.
    #[test]
    fn the_cap_is_a_literal_whose_authority_is_the_go_test() {
        assert_eq!(MAX_ENGRAVEABLE_MS1_LEN, 90);
    }

    #[test]
    fn classifies_each_record_kind() {
        assert_eq!(validate_record(MD1[0]).unwrap(), RecordKind::Md);
        assert_eq!(validate_record(MK1[0]).unwrap(), RecordKind::Mk);
        assert_eq!(validate_record(MS1).unwrap(), RecordKind::Ms);
    }

    /// THE round-3 Critical. `mnemonic bundle` prints --group-size 5 by default;
    /// codex32's inputChar has no mapping for 0x20, so the device classifies a
    /// spaced record as unknown. Refuse here — never strip, or the plate carries
    /// separators the BCH checksum never covered.
    #[test]
    fn refuses_space_grouped_and_hyphenated_records() {
        assert!(matches!(
            validate_record("md1fv9w jpqpqpm6"),
            Err(RecordError::NonCanonical { ch: ' ', .. })
        ));
        assert!(matches!(
            validate_record("md1fv9w-jpqpqpm6"),
            Err(RecordError::NonCanonical { ch: '-', .. })
        ));
    }

    /// §6.4: uppercase passes the BCH validators, so without this the same
    /// wallet has two spec-legal encodings and therefore two §6.6 hashes.
    #[test]
    fn refuses_uppercase_records() {
        assert!(matches!(
            validate_record(&MD1[0].to_uppercase()),
            Err(RecordError::NotLowercase(_))
        ));
    }

    #[test]
    fn refuses_corrupt_and_unknown_records() {
        let mut bad = MD1[0].to_string();
        let last = bad.pop().unwrap();
        bad.push(if last == 'q' { 'p' } else { 'q' });
        assert!(validate_record(&bad).is_err());
        assert!(validate_record("xx1qqqq").is_err());
    }

    /// §6.3: DECODE is per CARD SET, not per record. Records are CHUNKS —
    /// verified against the real crates:
    ///   md1 single chunk → "chunk set incomplete: got 1 chunks, expected 3"
    ///   mk1 single chunk → "received 1 chunks, header declares total_chunks = 2"
    /// A per-record decode would reject every legitimate payload.
    #[test]
    fn decodes_a_complete_card_set() {
        let all: Vec<&str> = MD1.iter().chain(MK1.iter()).copied().collect();
        assert!(
            decode_public_set(&all).is_ok(),
            "the full md1+mk1 set must decode"
        );
    }

    #[test]
    fn refuses_an_incomplete_card_set() {
        assert!(
            decode_public_set(&[MD1[0]]).is_err(),
            "one md1 chunk of three"
        );
        assert!(
            decode_public_set(&[MK1[0]]).is_err(),
            "one mk1 chunk of two"
        );
        assert!(
            decode_public_set(&MD1[..2]).is_err(),
            "two md1 chunks of three"
        );
    }

    /// The pristine-only rule for `mk1`: a string the BCH layer had to CORRECT
    /// must be refused, not silently repaired and engraved.
    ///
    /// This guard was uncovered — the mutation that deletes it survived the
    /// whole suite (Phase 2 Rust-side review). It is load-bearing for
    /// cross-implementation agreement: the device refuses a BCH-correctable
    /// `mk1` too, and a host that quietly accepted one would engrave a card the
    /// machine would not have produced from the same input.
    ///
    /// The test SEARCHES for a correctable corruption rather than hard-coding
    /// one, which makes it its own positive control: if no single-character
    /// change of the fixture is correctable, the search fails loudly instead of
    /// passing vacuously, and the vector — not the guard — is what needs
    /// attention.
    #[test]
    fn refuses_an_mk1_the_bch_layer_had_to_correct() {
        // bech32's charset, minus the four characters it excludes (1, b, i, o).
        const CHARSET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

        let base = MK1[0];
        assert!(
            validate_record(base).is_ok(),
            "control: the pristine fixture must validate before corrupting it"
        );

        let mut found = None;
        'outer: for pos in "mk1".len()..base.len() {
            for &c in CHARSET {
                let mut bytes = base.as_bytes().to_vec();
                if bytes[pos] == c {
                    continue;
                }
                bytes[pos] = c;
                let cand = String::from_utf8(bytes).expect("ascii substitution stays utf8");
                if let Ok(d) = mk_codec::string_layer::decode_string(&cand) {
                    if d.corrections_applied != 0 {
                        found = Some((cand, d.corrections_applied));
                        break 'outer;
                    }
                }
            }
        }

        let (corrupted, n) = found.expect(
            "no single-character corruption of MK1[0] was BCH-correctable -- the search \
             found nothing, so this test proves nothing about the guard",
        );
        assert!(n > 0, "the search must have found a CORRECTED decode");

        match validate_record(&corrupted) {
            Err(RecordError::Invalid(msg)) => assert!(
                msg.contains("not pristine"),
                "refused, but not as a pristineness failure: {msg}"
            ),
            other => panic!(
                "an mk1 needing {n} BCH correction(s) was not refused: {other:?} -- \
                 the host would engrave a card the device rejects"
            ),
        }
    }

    /// The §6.3 smuggling case: arbitrary bytes wrapped in a BCH-valid md1.
    /// `ValidMD` passes it; the decode must not.
    #[test]
    fn refuses_a_bch_valid_but_undecodable_record() {
        const SMUGGLED: &str =
            "md1qqqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0sdmjzeptm5fdk0";
        assert!(
            validate_record(SMUGGLED).is_ok(),
            "BCH layer accepts it — that is the point"
        );
        assert!(
            decode_public_set(&[SMUGGLED]).is_err(),
            "decode must reject it"
        );
    }
}
