//! **Wallet descriptors as `me` input** — `SPEC_descriptor_input.md`, S1.
//!
//! `me` reads the four formats `nonstandard.OutputDescriptor` reads, in the
//! same order ([`cascade`]), narrows them to a profile stated as eight explicit
//! conjuncts ([`admit`]), decides whether an `--as`-omitted invocation is
//! looking at a descriptor at all ([`gate`]), and refuses with one message per
//! cause ([`refusal`]) instead of the device's single five-word message for
//! eleven distinct causes.
//!
//! **The direction that matters is asymmetric.** A host that admits what the
//! device refuses packs a payload the device cannot read — an engraved plate
//! for a wallet that will not load. Everything here is held to that by
//! `crates/me-cli/testdata/descriptor_seam_vectors.json`, a file the fork
//! carries byte-identically and asserts the device column of; neither
//! implementation is ever compared to the other, both are compared to the file.
//!
//! # The `format` column, and why "matched" means "succeeded" (F-1)
//!
//! §7 defines `format` as *"the branch of §4's cascade that `me` MATCHED"*,
//! while §6 separately ranks *"the branch the input most RESEMBLES"*. P0 read
//! the first STRICTLY — the branch that SUCCEEDED, per §4.1's *"first branch
//! that succeeds wins"* — and wrote that reading into the vector file's own
//! `_comment`. **P1 CONFIRMS it, in code and by assertion**, and the confirming
//! consequence is structural rather than a preference:
//!
//! * §4.2's four narrowings are stated as things *"`me` refuses"* about a
//!   BlueWallet FILE, so they make branch 1 FAIL. All five `narrowed-4.2` rows
//!   therefore carry `format: "none"`, which is what the file says.
//! * §4.5's refused promotions make branch 4 fail, so the eight refused rows
//!   carry `none` too.
//! * A row whose cascade SUCCEEDED and whose refusal comes from §4.7 keeps its
//!   branch name — which is exactly the `narrowed-4.7` rows' `bip380`.
//!
//! Under the other reading those thirteen rows would carry `bluewallet` /
//! `promoted-key`, and the column would stop distinguishing a PARSER refusal
//! from a PROFILE one. `tests/descriptor_seam.rs` asserts the column on every
//! row, so the reading is now pinned by a suite rather than by a comment.
//!
//! # What is NOT here yet
//!
//! `--as` itself (the flag, the md1 build path, §5.4's identification block,
//! §6's per-row text tests) is P2. This module is the input contract and the
//! gate, and it is complete for both.

pub mod admit;
pub mod base58;
pub mod cascade;
pub mod checksum;
pub mod gate;
pub mod refusal;
pub mod secp;

pub use admit::{format_of, host_admits, Path};
pub use gate::{
    choice_block, consult, gate_opens, Outcome, DESCRIPTOR_PATH_SHIPPED, MD1_PATH_SHIPPED,
};
pub use refusal::{Refusal, Row};
