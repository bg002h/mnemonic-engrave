//! **The private-channel EXAMPLES `me` prints when it refuses argv material.**
//!
//! They live in the LIB rather than beside the refusal in `main.rs` for one
//! reason: §6h requires that remedy text name channels that exist and pipelines
//! that RUN, and the only way to prove that is to RUN the exact bytes the binary
//! emits. `main.rs` is a binary target, so an integration test cannot reach a
//! constant declared there — and a test that ran its own copy of the string
//! would prove only that the copy works. See
//! `crates/me-cli/tests/ms_remedy_runs.rs`.

/// The private-channel example shown to an operator who put BEARER material on
/// argv.
pub const BEARER_PRIVATE_CHANNEL_EXAMPLE: &str =
    "    mt encode --qr --in tx.hex | me sysw pack --out p.bin";

/// The private-channel example shown to an operator who put SECRET key material
/// on argv.
///
/// **§6h's standing instruction fired here: this is now the `--in` form, and it
/// was not before.** The retracted comment beside it read *"`ms encode --in`
/// DOES NOT EXIST (exit 64) … `--phrase -` … is verified to pipe into pack"*.
/// Both halves were false by the time P2 ran:
///
/// * `ms encode --in FILE` exists as of P2 and means a PHRASE;
/// * the `--phrase -` pipeline, RUN VERBATIM, exited **4** with
///   `me: record 0 (records count from 0) is not a form this container can
///   place` and wrote no payload, because `ms encode`'s stdout was grouped by
///   default and `me sysw pack` cannot classify a grouped `ms1`. The same
///   pipeline with `--group-size 0` exited 0 and wrote a 102-byte payload at
///   0600. Nothing verified the claim: `crates/me-cli/tests/` held 14 `.rs`
///   files with 33 `Command::new` sites and **none** naming an `ms` binary.
///   F-301.
///
/// P2 made `ms encode`'s stdout the canonical ungrouped `ms1`, so the pipeline
/// below now runs. It is a `pub const` so a test can RUN the exact bytes the
/// binary emits rather than a copy of them.
///
/// **THIS BRANCH IS CURRENTLY UNREACHABLE, and saying so is the point.**
/// Measured 2026-08-27 over eleven argv shapes (a BIP-39 phrase, an `ms1` in
/// three spellings, `pass:`, `text:`, three `tx:` forms, `md1`, `mt1`): the
/// pre-parser `argv_secret_guard` refuses every input for which
/// `class.is_argv_forbidden()` holds, at exit 3, before `read_records` runs —
/// so the `else` arm above can only be selected by an input the pre-parser
/// already rejected. The reachable half of this refusal is the `by_prefix` arm
/// (a `tx:` prefix the classifier does not recognise), which always takes the
/// BEARER example. The text is corrected regardless, because a dead branch that
/// is also WRONG is one refactor away from being live and wrong. F-362.
pub const SECRET_PRIVATE_CHANNEL_EXAMPLE: &str =
    "    ms encode --in seed.txt | me sysw pack --out p.bin";
