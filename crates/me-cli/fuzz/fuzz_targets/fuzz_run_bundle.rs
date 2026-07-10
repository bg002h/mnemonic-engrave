#![no_main]
//! cargo-fuzz target for the multi-line bundle orchestrator: `run_bundle` never
//! panics on arbitrary bytes; any input containing an `ms` line is refused with
//! `RefusedSecret`; and on any Ok manifest every emitted plate string is a
//! verbatim trimmed input line (P5 — the funds-relevant no-substitution property).
//!
//! Same CI residual as `fuzz_convert`: nightly-only, NOT CI-built, but every
//! invariant is also covered by the CI proptest layer via the SAME shared checkers
//! (R0 L4). Run:
//!   cargo +nightly fuzz run fuzz_run_bundle -- -runs=100000

use libfuzzer_sys::fuzz_target;

// Shared invariant checkers (see fuzz_convert.rs). `#[allow(dead_code)]`: this
// target uses only the run_bundle checkers; the convert / ndef checkers are unused.
#[allow(dead_code)]
#[path = "../../tests/support/invariants.rs"]
mod invariants;

fuzz_target!(|data: &[u8]| {
    let s = String::from_utf8_lossy(data);
    invariants::assert_run_bundle_no_panic(&s);
    invariants::assert_bundle_ms_line_refused(&s);
    invariants::assert_manifest_strings_trace(&s);
});
