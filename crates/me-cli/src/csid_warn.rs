//! Chunk-set-id mismatch warning (SPEC "The comparison", operator ruling R6
//! "same warning everywhere") — shared by every `me-cli` surface that
//! reassembles a CHUNKED mk1 key card: `me bundle`, `me seal`, `me sysw
//! pack`/`show`. A single-string mk1 card carries no `chunk_set_id`, so this
//! is silent there — nothing to compare (§ "Chunked input only").
//!
//! `me-cli`'s recent bump to `mk-codec 0.5` (0.4.1 -> 0.5.0, a clean,
//! non-breaking bump: 580/580 tests green before this module existed) makes
//! `mk_codec::derive_chunk_set_id` and `mk_codec::bytecode::encode_bytecode`
//! available here for the first time.
//!
//! The warning CONTENT must be byte-identical to mk-cli's `mk decode`
//! warning (`crates/mk-cli/src/cmd/mod.rs::chunk_set_id_mismatch_warning` in
//! `mnemonic-key`) and to md-cli's `seat_chunk_set_id_warnings`
//! (`crates/md-cli/src/seat/input.rs` in `descriptor-mnemonic`) — all three
//! are independent binaries sharing no runtime code, so each computes its
//! own operand and prints its own copy of this exact string (R6). The
//! `wording_pin_matches_the_frozen_r6_text` test below guards this copy
//! against drift with a literal typed independently of the `format!` call
//! it checks — the same interim R6-drift guard md-cli uses, since none of
//! the three crates share the extension corpus as a runtime dependency.

use mk_codec::KeyCard;

/// The frozen R2/R6 mismatch-warning content. `{:05x}`: exactly five
/// lowercase hex digits, zero-padded — the rendering this constellation's
/// `GroupId::Display` prints and `md --seat @i=` accepts elsewhere.
pub fn chunk_set_id_mismatch_warning(declared: u32, derived: u32) -> String {
    format!(
        "warning: this key card's stamped chunk-set id ({declared:05x}) was not derived from \
         its content, which computes {derived:05x}. The card decodes fine, but diagnostics that \
         name plates by id will call it {declared:05x}. To fix it, re-mint: run mk encode again \
         without --chunk-set-id and the id is derived from the key data automatically."
    )
}

/// The chunk-set id declared on the wire header of ONE string of a chunked
/// mk1 set, or `None` for single-string (unchunked) input.
///
/// Any one chunk of an already-successfully-decoded set is sufficient:
/// `mk_codec::decode`'s `reassemble_from_chunks` already proved every chunk
/// agrees on `chunk_set_id` before returning `Ok`, so reading one chunk's
/// header re-reads a value the codec already verified rather than skipping
/// that check.
fn declared_chunk_set_id(one_chunk: &str) -> Option<u32> {
    let decoded = mk_codec::string_layer::decode_string(one_chunk).ok()?;
    let (header, _consumed) =
        mk_codec::string_layer::StringLayerHeader::from_5bit_symbols(decoded.data()).ok()?;
    match header {
        mk_codec::string_layer::StringLayerHeader::Chunked { chunk_set_id, .. } => {
            Some(chunk_set_id)
        }
        // `StringLayerHeader` is `#[non_exhaustive]`; `SingleString` and any
        // future variant alike carry no `chunk_set_id` to compare.
        _ => None,
    }
}

/// SPEC "The comparison": `derive_chunk_set_id(encode_bytecode(decoded_card))`
/// — the canonical RE-ENCODE of the successfully decoded card, not the raw
/// reassembled bytes (a foreign encoder whose bytecode canonicalization
/// drifts stamps an id consistent with its own bytes; only the re-encode
/// route detects that drift).
fn derived_chunk_set_id(card: &KeyCard) -> Option<u32> {
    let bytecode = mk_codec::bytecode::encode_bytecode(card).ok()?;
    Some(mk_codec::derive_chunk_set_id(&bytecode))
}

/// Compare the declared vs. content-derived `chunk_set_id` for a decoded mk1
/// SET. `refs` is the set's chunks in whatever order the caller has (any one
/// carries the declared id post-decode, per `declared_chunk_set_id`'s doc).
/// `None` for single-string input, or if `refs` is empty. `Some((declared,
/// derived))` otherwise, REGARDLESS of whether they agree — callers decide
/// what a match vs. a mismatch means.
pub fn chunk_set_id_comparison(refs: &[&str], card: &KeyCard) -> Option<(u32, u32)> {
    let one_chunk = refs.first()?;
    let declared = declared_chunk_set_id(one_chunk)?;
    let derived = derived_chunk_set_id(card)?;
    Some((declared, derived))
}

/// Emit the R2/R6 stderr warning for a precomputed [`chunk_set_id_comparison`]
/// result. A no-op on a match, or on `None` (single-string input, or a
/// re-encode that could not be computed).
///
/// Independently deletable per call site (the P1 mutation gate this cycle's
/// implementation report measures): each of the three attach points is this
/// one line, seated at that surface's own decode call.
pub fn warn_chunk_set_id_mismatch(comparison: Option<(u32, u32)>) {
    if let Some((declared, derived)) = comparison {
        if declared != derived {
            eprintln!("{}", chunk_set_id_mismatch_warning(declared, derived));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A real 2-chunk mk1 set, chunk_set_id 0x12345 (mk-codec v0.1.json) --
    // `bundle.rs`'s own MK1_A/MK1_B fixture, a LEGACY-pinned card minted
    // before content-derivation existed. Measured (not assumed): declared
    // 0x12345, content-derives 0x83bb2 -- a genuine mismatch.
    const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
    const MK1_B: &str =
        "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";

    // The corpus's CT1 twin (`mnemonic-key`
    // `crates/mk-codec/src/test_vectors/csid_ext_v0.1.json`,
    // `CT1_twin_of_V1_bip48_mainnet_1_stub_with_fp`): same key material as
    // MK1_A/MK1_B, minted WITHOUT a pinned id, so declared == derived ==
    // 0x83bb2. The clean control.
    const CLEAN_A: &str = "mk1qpswajpqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3k25gsrttm4zzk4z4";
    const CLEAN_B: &str =
        "mk1qpswajppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dh520sknslwyt0";

    const MD1_UNCHUNKED: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

    /// R6-drift guard: a literal typed independently of the `format!` call
    /// it checks, matching the extension corpus's frozen `warning_text` for
    /// the pinned `12345`/`ef12f` row verbatim (`csid_ext_v0.1.json`, row
    /// `SEED_pinned_12345_ef12f`).
    #[test]
    fn wording_pin_matches_the_frozen_r6_text() {
        let expected = "warning: this key card's stamped chunk-set id (12345) was not derived \
             from its content, which computes ef12f. The card decodes fine, but diagnostics \
             that name plates by id will call it 12345. To fix it, re-mint: run mk encode \
             again without --chunk-set-id and the id is derived from the key data \
             automatically.";
        assert_eq!(chunk_set_id_mismatch_warning(0x12345, 0xef12f), expected);
    }

    /// Independent literals (declared+derived pair, typed by hand) plus the
    /// remedy sentence, checked as substrings rather than through the
    /// `format!` call under test -- so a wording drift in ANY of the three
    /// segments (stamped-id clause, derived-id clause, remedy) is caught
    /// even if the other two happen to still agree.
    #[test]
    fn wording_pin_independent_fragments() {
        let w = chunk_set_id_mismatch_warning(0x12345, 0xef12f);
        assert!(w.starts_with("warning:"), "{w}");
        assert!(w.contains("(12345)"), "{w}");
        assert!(w.contains("computes ef12f"), "{w}");
        assert!(
            w.contains(
                "run mk encode again without --chunk-set-id and the id is derived from the \
                 key data automatically"
            ),
            "remedy sentence drifted: {w}"
        );
    }

    /// Zero-padding: a five-digit hex value below 0x10000 still renders five
    /// digits (leading zero), matching `{:05x}`'s contract.
    #[test]
    fn wording_pin_zero_pads_below_0x10000() {
        let w = chunk_set_id_mismatch_warning(0x0191c, 0x00042);
        assert!(w.contains("(0191c)"), "{w}");
        assert!(w.contains("computes 00042"), "{w}");
    }

    #[test]
    fn comparison_detects_the_pinned_mismatch() {
        let refs = [MK1_A, MK1_B];
        let card = mk_codec::decode(&refs).expect("decode");
        assert_eq!(
            chunk_set_id_comparison(&refs, &card),
            Some((0x12345, 0x83bb2))
        );
    }

    #[test]
    fn comparison_agrees_on_the_clean_twin() {
        let refs = [CLEAN_A, CLEAN_B];
        let card = mk_codec::decode(&refs).expect("decode");
        let (declared, derived) = chunk_set_id_comparison(&refs, &card).expect("some comparison");
        assert_eq!(declared, derived, "the clean twin must not mismatch");
        assert_eq!(declared, 0x83bb2);
    }

    /// `chunk_set_id_comparison` returns `None` when there is nothing to
    /// compare against -- the degenerate empty-input case, and the same
    /// `None` a genuinely single-string (unchunked) mk1 card produces via
    /// `declared_chunk_set_id`'s non-`Chunked` match arm below.
    ///
    /// A REAL single-string mk1 card is not constructible here to drive that
    /// second path directly: `KeyCard`'s compact xpub form alone is 73 bytes
    /// (`bitcoin_key`'s doc above), which already exceeds a regular codex32
    /// string's usable payload -- confirmed empirically, not assumed: every
    /// fixture in this file and in `bundle.rs`'s own test module (MK1_A/B,
    /// MK1_C/D, the seal pair, the clean twin) is a 2-chunk set, and
    /// `bundle.rs`'s own comment on `Mk1SingleString` says the same ("only
    /// synthetic <=56-byte cards hit this"). The `Chunked`/non-`Chunked`
    /// match in `declared_chunk_set_id` is exhaustive over
    /// `#[non_exhaustive] StringLayerHeader` (compiler-checked), so the
    /// non-`Chunked` arm returning `None` is a type-level guarantee, not
    /// something this test needs to additionally drive with real wire bytes.
    #[test]
    fn comparison_is_none_on_empty_input() {
        let card = mk_codec::decode(&[MK1_A, MK1_B]).expect("decode");
        let empty: [&str; 0] = [];
        assert_eq!(chunk_set_id_comparison(&empty, &card), None);
    }

    /// Sanity: `MD1_UNCHUNKED` is an md1 string, not mk1, so it does not even
    /// decode as a `KeyCard` -- confirms the fixture name means what it says
    /// and is not accidentally reused across the md/mk boundary.
    #[test]
    fn md1_string_does_not_decode_as_mk1() {
        assert!(mk_codec::decode(&[MD1_UNCHUNKED]).is_err());
    }

    #[test]
    fn warn_is_silent_on_a_match_or_none() {
        // No panics, no way to assert silence of eprintln! directly in-proc
        // -- this test exists so the no-op branches are at least exercised
        // (coverage), while the actual stderr assertion lives in the
        // process-level `assert_cmd` integration tests per surface.
        warn_chunk_set_id_mismatch(Some((0x83bb2, 0x83bb2)));
        warn_chunk_set_id_mismatch(None);
    }
}
