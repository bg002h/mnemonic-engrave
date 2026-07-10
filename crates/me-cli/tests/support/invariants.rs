//! Shared funds-safety invariant checkers, used by BOTH the proptest layer
//! (CI-covered, `tests/prop.rs`) and the cargo-fuzz layer (`fuzz/`). Included into
//! each consumer via `#[path = "…/invariants.rs"] mod invariants;`, so the two
//! layers assert IDENTICAL properties and can never drift.
//!
//! Every checker rides ONLY the already-`pub` API of `mnemonic-engrave`
//! (`convert`, `bundle::run_bundle`, `ConvertError`, `BundleError`,
//! `Manifest.plates`, `PlateEntry.string`, `classify`, `ndef`). It adds ZERO new
//! public surface to the PUBLISHED crate — no `pub fn check_*` on the library.
//!
//! The library is referenced by its EXTERNAL name `mnemonic_engrave::…` (never
//! `crate::`), so this one file compiles identically in the in-tree integration
//! test crate (which links the lib as `mnemonic_engrave`) and in the separate
//! fuzz crate (whose `mnemonic-engrave = { path = ".." }` dep imports the same
//! name). Insurance, not a fix: D6-7 found no reachable panic — these pin that
//! arbitrary input keeps the invariants TRUE.
//!
//! `#[allow(dead_code)]` at each include site: an individual fuzz target uses only
//! a subset of these checkers, so the rest are (correctly) unused in that binary.

use mnemonic_engrave::bundle::{run_bundle, BundleError};
use mnemonic_engrave::classify::{classify, Format};
use mnemonic_engrave::ndef::{decode_text_tlv, encode_text_tlv, NdefError};
use mnemonic_engrave::{convert, ConvertError};

/// P1 — `convert` returns Ok/Err on any input, never panics. (A panic surfaces as
/// a test/fuzz failure at the call site.)
pub fn assert_convert_no_panic(s: &str) {
    let _ = convert(s);
}

/// P2 — any input whose (trimmed) HRP classifies as `ms` is refused by `convert`
/// with EXACTLY `RefusedSecret` — never Ok, never a different error that would let
/// it proceed. Mirrors `convert`'s own `classify` gate as an independent oracle,
/// so removing the refusal reds this. Vacuous (no-op) on non-`ms` inputs.
pub fn assert_convert_ms_refused(s: &str) {
    if classify(s) == Ok(Format::Ms) {
        assert!(
            matches!(convert(s), Err(ConvertError::RefusedSecret)),
            "convert must refuse an ms-HRP input with RefusedSecret: {s:?}"
        );
    }
}

/// P3 — `run_bundle` returns Ok/Err on any input, never panics.
pub fn assert_run_bundle_no_panic(s: &str) {
    let _ = run_bundle(s);
}

/// P4 — if ANY non-empty trimmed line classifies as `ms`, `run_bundle` refuses the
/// whole run with `RefusedSecret` (regardless of position / the other lines).
/// Mirrors `run_bundle`'s pre-scan (`input.lines().map(trim).filter(non-empty)`)
/// as an independent oracle. Vacuous on inputs with no `ms` line.
pub fn assert_bundle_ms_line_refused(s: &str) {
    let has_ms_line = s
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .any(|l| classify(l) == Ok(Format::Ms));
    if has_ms_line {
        assert!(
            matches!(run_bundle(s), Err(BundleError::RefusedSecret)),
            "run_bundle must refuse an input containing an ms line with RefusedSecret: {s:?}"
        );
    }
}

/// P5 — no substitution (the funds-relevant property): when `run_bundle` is Ok,
/// every emitted `Some` plate string is a VERBATIM trimmed input line — never
/// fabricated, re-serialized, or reassembled. Uses `.lines()` (CRLF-aware, mirrors
/// `run_bundle`'s own line pipeline), NOT `split('\n')`. Vacuous when `run_bundle`
/// errs (no manifest to check).
pub fn assert_manifest_strings_trace(s: &str) {
    if let Ok(manifest) = run_bundle(s) {
        for plate in &manifest.plates {
            if let Some(emitted) = &plate.string {
                assert!(
                    s.lines().map(str::trim).any(|line| line == emitted),
                    "emitted plate string is not a verbatim trimmed input line: {emitted:?}"
                );
            }
        }
    }
}

/// P6 — NDEF round-trip is TOTAL on the ≤249-BYTE domain and the encoder is
/// charset-agnostic (raw `str` byte copy); the bound is on UTF-8 BYTE length, not
/// char count. Result-aware: `encode` is Ok iff `t.len() <= 249` bytes and then
/// `decode` recovers `t` byte-exact; `t.len() >= 250` bytes → `Err(TooLong)`.
pub fn assert_ndef_roundtrip(t: &str) {
    match encode_text_tlv(t) {
        Ok(bytes) => assert_eq!(
            decode_text_tlv(&bytes).as_deref(),
            Some(t),
            "≤249-byte text must round-trip byte-exact"
        ),
        Err(NdefError::TooLong(_)) => assert!(
            t.len() >= 250,
            "encode returned TooLong below the 250-byte boundary (t = {} bytes)",
            t.len()
        ),
    }
}
