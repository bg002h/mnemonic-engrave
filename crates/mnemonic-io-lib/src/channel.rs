//! **`--in` / `--out` / `-`, and where a container's bytes are going.**
//!
//! Classification ONLY (N-I1). This module answers *where is stdout pointing*
//! and nothing else: it does not decide whether that is acceptable — that is
//! [`super::exit`] — and it does not announce anything, which is the binary's.
//!
//! **The `--out` overwrite rule is NOT gated here, and saying so is
//! load-bearing.** `0o600` binds on CREATE, so an existing world-readable
//! target keeps its old mode; the fix is `write_private` tightening the OPEN
//! file, and `write_private` stays in `me`. Gating that rule on `destination`
//! would gate nothing at all, because `destination` never touches a path.

/// The refusal F-244 asks for, with the override it names.
/// Where a container's bytes are going — F-253.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    /// `--out`: `me` creates the file itself, owner-only.
    File,
    /// A pipe or a redirect. The ruled pipeline lives here.
    Stream,
    /// A terminal. **Not a destination for a bearer container.**
    Terminal,
}

/// Decide where stdout is pointing, as a pure function of the two facts that
/// matter — so it is testable without a pty, which no dev-dependency here can
/// give us. `emit` supplies `std::io::IsTerminal`.
///
/// A pty gate exists too, and it is not redundant with this:
/// `tests/terminal_destination.rs` drives a REAL terminal through util-linux
/// `script` and pins the refusal's exit DIGIT, because none of the 12
/// `world_readable_output.rs` tests reaches the terminal arm at all.
pub fn destination(out_given: bool, stdout_is_tty: bool) -> Destination {
    if out_given {
        Destination::File
    } else if stdout_is_tty {
        Destination::Terminal
    } else {
        Destination::Stream
    }
}
