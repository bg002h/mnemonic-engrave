#![no_main]
//! cargo-fuzz target for the single-string converter: `convert` never panics on
//! arbitrary bytes, and any input whose first token has HRP `ms` is refused with
//! `RefusedSecret`.
//!
//! Deep/local coverage only — this crate is INTENTIONALLY NOT built in CI
//! (nightly-only; adds toolchain + flakiness for an insurance item). That residual
//! is acceptable because every invariant this target checks is ALSO exercised by
//! the CI-covered proptest layer (`crates/me-cli/tests/prop.rs`) through the SAME
//! shared checkers below — only this ~10-line wrapper is un-CI'd, so it cannot
//! silently rot a real invariant (R0 L4). Run:
//!   cargo +nightly fuzz run fuzz_convert -- -runs=100000

use libfuzzer_sys::fuzz_target;

// Shared invariant checkers — the identical file driven by the proptest layer, so
// the two layers can never drift. `#[allow(dead_code)]`: this target uses only the
// convert checkers; the run_bundle / ndef checkers are (correctly) unused here.
#[allow(dead_code)]
#[path = "../../tests/support/invariants.rs"]
mod invariants;

fuzz_target!(|data: &[u8]| {
    let s = String::from_utf8_lossy(data);
    invariants::assert_convert_no_panic(&s);
    invariants::assert_convert_ms_refused(&s);
});
