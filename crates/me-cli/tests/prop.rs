//! Property tests (proptest, STABLE toolchain, CI-covered) for the funds-safety
//! invariants — the load-bearing half of Cycle C (F18). Each property drives the
//! shared checkers in `tests/support/invariants.rs`, which are ALSO the bodies of
//! the cargo-fuzz targets (`fuzz/`), so the CI layer and the deep-fuzz layer can
//! never drift. Insurance, not a fix: D6-7 found no reachable panic — these pin
//! that arbitrary input keeps the invariants TRUE.
//!
//! Case budget is bounded (`ProptestConfig::with_cases(256)`) so the CI suite
//! stays fast; random inputs almost never form BCH-valid md1/mk1 (fast
//! classify/validate rejection) and ms-refusal short-circuits before validation,
//! so per-case cost is trivial. Tune via that one knob.

#[allow(dead_code)]
#[path = "support/invariants.rs"]
mod invariants;

use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Known-VALID public constellation strings (lifted verbatim from the crate's own
// unit/bundle tests) so the P5 strategy actually produces Ok manifests and the
// property is non-vacuous. MD1_VALID is a proven unchunked single (bundle test
// `parses_unchunked_md1_as_bch_only`); MK1_A/MK1_B are the proven-complete 2-chunk
// mk1 set (chunk_set_id 0x12345) from `bundle::tests`.
// ---------------------------------------------------------------------------
const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
const MK1_B: &str =
    "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";

/// bech32 data charset (no `1`, so it never introduces a spurious separator).
const BECH32: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Genuinely arbitrary UTF-8, INCLUDING newlines, control chars, and multibyte
/// scalars. `(?s)` makes `.` match `\n` too (R0 L3: a plain `.*` never emits
/// newlines and would under-test the multi-line / control-byte domain).
fn arb_text() -> impl Strategy<Value = String> {
    proptest::string::string_regex("(?s).{0,300}").unwrap()
}

/// A single line (no embedded newline) of arbitrary content — a building block
/// for the multi-line strategies.
fn arb_line() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[^\n]{0,80}").unwrap()
}

/// A bech32-ish token with a chosen (possibly whitespace/case-perturbed) HRP —
/// biases generation toward the classify/validate code paths that a purely random
/// string almost never reaches.
fn biased_line() -> impl Strategy<Value = String> {
    let hrp = prop_oneof![
        Just("md"),
        Just("mk"),
        Just("ms"),
        Just("MS"),
        Just("Ms"),
        Just("xx"),
        Just(""),
    ];
    let tail = proptest::string::string_regex(&format!("[{BECH32}]{{0,120}}")).unwrap();
    (hrp, tail, any::<bool>(), any::<bool>()).prop_map(|(hrp, tail, pad_l, pad_r)| {
        let mut s = format!("{hrp}1{tail}");
        if pad_l {
            s = format!("  {s}");
        }
        if pad_r {
            s = format!("{s}  ");
        }
        s
    })
}

/// A line guaranteed to classify as `ms` — the leading token of its own line
/// (mirrors P2's "first token" precision; R0 N-c), across case and whitespace
/// padding variants.
fn ms_line() -> impl Strategy<Value = String> {
    let hrp = prop_oneof![Just("ms"), Just("MS"), Just("Ms"), Just("mS")];
    let tail = proptest::string::string_regex(&format!("[{BECH32}]{{0,120}}")).unwrap();
    (hrp, tail, any::<bool>(), any::<bool>()).prop_map(|(hrp, tail, pad_l, pad_r)| {
        let mut s = format!("{hrp}1{tail}");
        if pad_l {
            s = format!("   {s}");
        }
        if pad_r {
            s = format!("{s}   ");
        }
        s
    })
}

/// An arbitrary MULTI-LINE input (R0 L3: the vec-of-lines join guarantees the
/// `\n`s that `run_bundle` is about; a bare `.*` never would).
fn multiline() -> impl Strategy<Value = String> {
    prop::collection::vec(prop_oneof![arb_line(), biased_line()], 0..8)
        .prop_map(|lines| lines.join("\n"))
}

/// A multi-line input that places an `ms1…` line (its own leading token) at an
/// arbitrary position among other lines.
fn multiline_with_ms() -> impl Strategy<Value = String> {
    (
        prop::collection::vec(prop_oneof![arb_line(), biased_line()], 0..6),
        ms_line(),
        0usize..7,
    )
        .prop_map(|(mut lines, ms, pos)| {
            let pos = pos.min(lines.len());
            lines.insert(pos, ms);
            lines.join("\n")
        })
}

/// A COMPLETE, valid public bundle that `run_bundle` accepts (Ok), with random
/// surrounding whitespace + interspersed blank lines (all trimmed away) so the
/// emitted plate strings must trace back to a *trimmed* line — this is what makes
/// P5 non-vacuous and gives it teeth against a fabricated/re-serialized string.
fn valid_bundle() -> impl Strategy<Value = String> {
    let scenario = prop_oneof![
        Just(vec![MD1_VALID]),
        Just(vec![MK1_A, MK1_B]),
        Just(vec![MD1_VALID, MK1_A, MK1_B]),
    ];
    (scenario, any::<u32>()).prop_map(|(lines, bits)| {
        let mut out = String::new();
        for (i, line) in lines.iter().enumerate() {
            if (bits >> (i * 3)) & 1 == 1 {
                out.push('\n'); // interspersed blank line (trimmed away)
            }
            let left = ((bits >> (i * 3 + 1)) & 3) as usize;
            let right = ((bits >> (i * 3 + 2)) & 1) as usize;
            out.push_str(&" ".repeat(left));
            out.push_str(line);
            out.push_str(&" ".repeat(right));
            out.push('\n');
        }
        out
    })
}

/// NDEF text domain: arbitrary UTF-8 PLUS a byte-length sweep across the 249/250
/// boundary (ASCII 1-byte and a 3-byte multibyte scalar), so BOTH the Ok
/// round-trip branch and the `TooLong` branch are reliably exercised — for
/// single-byte and multibyte inputs alike (R0 I1: bound is on BYTES, not chars).
fn ndef_text() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_text(),
        (0usize..=260).prop_map(|n| "a".repeat(n)),
        (0usize..=90).prop_map(|n| "\u{20AC}".repeat(n)), // '€' = 3 bytes; ×83 = 249 bytes
    ]
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// P1 — `convert` never panics on arbitrary (incl. newline/control/multibyte)
    /// input, nor on charset-biased input.
    #[test]
    fn p1_convert_never_panics(s in prop_oneof![arb_text(), biased_line()]) {
        invariants::assert_convert_no_panic(&s);
    }

    /// P2 — any `ms`-first-token input is refused by `convert` with RefusedSecret.
    #[test]
    fn p2_convert_refuses_ms(s in ms_line()) {
        invariants::assert_convert_ms_refused(&s);
    }

    /// P3 — `run_bundle` never panics on arbitrary multi-line (or arbitrary) input.
    #[test]
    fn p3_run_bundle_never_panics(s in prop_oneof![multiline(), arb_text()]) {
        invariants::assert_run_bundle_no_panic(&s);
    }

    /// P4 — any input containing an `ms` line (any position) → RefusedSecret.
    #[test]
    fn p4_run_bundle_refuses_ms_line(s in multiline_with_ms()) {
        invariants::assert_bundle_ms_line_refused(&s);
    }

    /// P5 — no substitution: every emitted plate string is a verbatim trimmed
    /// input line. Exercised over Ok-producing valid bundles AND arbitrary
    /// multi-line inputs (the checker is vacuous when `run_bundle` errs).
    #[test]
    fn p5_manifest_strings_trace_to_input(
        s in prop_oneof![valid_bundle(), multiline()]
    ) {
        invariants::assert_manifest_strings_trace(&s);
    }

    /// P6 — NDEF round-trip total on ≤249 bytes / `TooLong` at ≥250 bytes,
    /// charset-agnostic, keyed on BYTE length.
    #[test]
    fn p6_ndef_round_trip(t in ndef_text()) {
        invariants::assert_ndef_roundtrip(&t);
    }
}
