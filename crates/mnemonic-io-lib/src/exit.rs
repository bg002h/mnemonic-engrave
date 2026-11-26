//! **The constellation's exit codes, and the refusal DECISIONS.**
//!
//! ## The numbers (`SPEC_constellation_exit_codes.md` §3)
//!
//! This module used to publish no exit integer at all, on the ground that "a
//! constant is a mapping" and the binaries agreed on none. The exit-code spec
//! is that mapping (its decision 4), so the integers now live here, once:
//!
//! | code | name | set when |
//! | --- | --- | --- |
//! | 0 | [`OK`] | everything requested was done and every check passed |
//! | 1 | [`NO`] | a well-formed input compared against something and came out no |
//! | 2 | [`INVALID`] | an input is wrong for this verb whatever the flags |
//! | 4 | [`UNVERIFIED`] | the tool checked part of the result and could not confirm it |
//! | 8 | [`REFUSED`] | the tool declined under a safety or policy rule a flag, network, channel or destination would waive; or a network mismatch |
//! | 16 | [`REPAIRED`] | a checksum correction was applied and the corrected value used |
//! | 32 | [`IO`] | something needed from the system failed or is missing |
//! | 64 | [`USAGE`] | never combines: the command line alone is wrong; nothing ran |
//! | 101 | [`INTERNAL`] | never combines: a bug, or a failure no class covers |
//! | 130 | [`STOPPED`] | the operator stopped it (a prompt answer, or Ctrl-C) |
//!
//! Codes 1 to 63 are sums of the six bits, and [`Bits`] is the type that sums
//! them. 64, 101 and 130 are single codes and are not bits: [`add`] adds bits
//! to a code and leaves a single code unchanged, and [`Bits::from_code`] reads
//! a code back into bits (`None` for a single code).
//!
//! **What stays each binary's own is the CLASSIFICATION**: which of its
//! failures is which class. The spec's ordered tests decide that per binary,
//! and nothing here names a binary's error type.
//!
//! ## Decision, not announcement
//!
//! `write_block` decides; the caller emits. Every `refuse_*` stays in the
//! binary, because a library six binaries share cannot write to stdio
//! unconditionally — it cannot be tested without capturing process stdio and a
//! caller cannot redirect it, which is doubly wrong in a module whose whole
//! purpose is controlling what reaches stdout. Which bit a refusal sets is the
//! caller's classification too.

/// 0: everything requested was done and every check passed.
pub const OK: u8 = 0;
/// Bit 1: a well-formed input was compared against something and came out no.
pub const NO: u8 = 1;
/// Bit 2: an input is wrong for this verb whatever the flags (bad checksum,
/// wrong kind, empty input, a malformed value the tool reads).
pub const INVALID: u8 = 2;
/// Bit 4: the tool checked part of the result and could not confirm it.
pub const UNVERIFIED: u8 = 4;
/// Bit 8: refused under a safety or policy rule that a flag, network, channel
/// or different destination would waive; also a network mismatch.
pub const REFUSED: u8 = 8;
/// Bit 16: a checksum correction was applied and the corrected value used.
pub const REPAIRED: u8 = 16;
/// Bit 32: something needed from the system failed or is missing.
pub const IO: u8 = 32;
/// 64, alone: the command line alone is wrong. Nothing ran.
pub const USAGE: u8 = 64;
/// 101, alone: a bug, or any failure no class covers (Rust's panic code).
pub const INTERNAL: u8 = 101;
/// 130, alone: the operator stopped it, at a prompt or with Ctrl-C.
pub const STOPPED: u8 = 130;

/// The root `--help` legend every binary ends with (spec R11), verbatim.
pub const HELP_LEGEND: &str =
    "Exit status: 0 = done, every check passed. 64 = bad command line, nothing \
ran. 101 = internal error. 130 = stopped by the operator. Other codes from \
128 up = killed by a signal. Otherwise the sum (1 to 63) of: 1 a check said no; \
2 an input is invalid; 4 verify the result before trusting it; 8 refused \
(network mismatch, secret on the command line, unsafe output); 16 a correction \
was applied; 32 a file, device or helper failed.";

/// A set of the six combinable bits (1, 2, 4, 8, 16, 32). Its exit code is
/// their sum, so every combination is a unique code from 0 to 63.
///
/// ```
/// use mnemonic_io_lib::exit::{self, Bits};
/// let rc = Bits::REPAIRED | Bits::UNVERIFIED;
/// assert!(rc.contains(Bits::REPAIRED));
/// assert_eq!(rc.to_exit_code(), 20);
/// assert_eq!(Bits::OK.to_exit_code(), exit::OK);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Bits(u8);

impl Bits {
    /// No bit set: exit 0.
    pub const OK: Bits = Bits(0);
    /// [`NO`] (1).
    pub const NO: Bits = Bits(NO);
    /// [`INVALID`] (2).
    pub const INVALID: Bits = Bits(INVALID);
    /// [`UNVERIFIED`] (4).
    pub const UNVERIFIED: Bits = Bits(UNVERIFIED);
    /// [`REFUSED`] (8).
    pub const REFUSED: Bits = Bits(REFUSED);
    /// [`REPAIRED`] (16).
    pub const REPAIRED: Bits = Bits(REPAIRED);
    /// [`IO`] (32).
    pub const IO: Bits = Bits(IO);

    /// True when every bit of `other` is set here.
    pub const fn contains(self, other: Bits) -> bool {
        self.0 & other.0 == other.0
    }

    /// True when no bit is set (exit 0).
    pub const fn is_ok(self) -> bool {
        self.0 == 0
    }

    /// The process exit status: the sum of the bits, 0 to 63.
    pub const fn to_exit_code(self) -> u8 {
        self.0
    }

    /// The bits of an exit code, or `None` for a code that is not a sum of
    /// bits (64 and above: [`USAGE`], [`INTERNAL`], [`STOPPED`], a signal).
    pub const fn from_code(code: u8) -> Option<Bits> {
        if code <= 63 {
            Some(Bits(code))
        } else {
            None
        }
    }
}

/// `code` with `bits` added (R1, R8): a code that is already a sum of bits
/// (0 to 63) gains them; a single code (64 and above: [`USAGE`], [`INTERNAL`],
/// [`STOPPED`], a signal) is returned unchanged, because those never combine.
/// So an IO failure after a usage error stays 64, never 96.
pub const fn add(code: u8, bits: Bits) -> u8 {
    if code > 63 {
        code
    } else {
        code | bits.0
    }
}

impl std::ops::BitOr for Bits {
    type Output = Bits;
    fn bitor(self, rhs: Bits) -> Bits {
        Bits(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Bits {
    fn bitor_assign(&mut self, rhs: Bits) {
        self.0 |= rhs.0;
    }
}

use super::channel::{destination, Destination};
use super::observation::PayloadKind;

/// Why a write cannot proceed — **the single decision**, so the early check and
/// `emit`'s cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteBlock {
    /// Nothing in the way.
    None,
    /// stdout is a terminal (F-253), **carrying what the payload is** so the
    /// refusal can say something true about it rather than assert the gate's
    /// name. F-259: this arm used to be a bare `Terminal`, and the only fact
    /// that could have corrected its message was riding a `bool` it never
    /// consulted.
    Terminal(PayloadKind),
    /// stdout is a file whose mode grants group/other read (F-252), carrying
    /// the mode so the refusal can quote what it measured.
    WorldReadable(u32),
}

/// **F-246 binds every gate, including ones added later.** The rule is that no
/// line describing a container may print until every gate that can abort the
/// write has run — so both gates are decided here, once, and consulted at the
/// top of `pack` as well as inside `emit`.
///
/// Terminal is checked FIRST because the mode check structurally cannot see it:
/// a TTY is a character device, and those are exempt there (that exemption is
/// load-bearing for `/dev/null`, mode 0666).
pub fn write_block(
    out_given: bool,
    kind: PayloadKind,
    allow_world_readable: bool,
    stdout_is_tty: bool,
    world_readable_mode: Option<u32>,
) -> WriteBlock {
    match destination(out_given, stdout_is_tty) {
        // `--out`: `me` creates the file 0600 itself, so neither gate applies.
        Destination::File => WriteBlock::None,
        // `--allow-world-readable` does NOT override this. It says "this file's
        // permissions are my problem"; it is not a request to paint a payload
        // across a scrollback, and the message offers a file route.
        //
        // **The KIND rides along rather than deciding here (F-259).** A fill
        // image is refused too — 64 KB of binary in a scrollback is worth
        // refusing whatever the secrecy — but the refusal has to be able to say
        // WHICH it is refusing, and before this the fact simply was not here.
        Destination::Terminal => WriteBlock::Terminal(kind),
        // The mode gate asks about EXPOSURE, so it is the one gate the kind
        // genuinely settles: there is nothing to expose in a fill image, and
        // `wipe` used to buy this outcome by passing `true` in the FLAG's seat.
        Destination::Stream => match world_readable_mode {
            Some(mode) if !allow_world_readable && kind.exposure_matters() => {
                WriteBlock::WorldReadable(mode)
            }
            _ => WriteBlock::None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `add` sums onto a bit code and never onto a single code; `from_code`
    /// is its inverse on 0..=63 and `None` above.
    #[test]
    fn add_and_from_code() {
        assert_eq!(add(OK, Bits::IO), 32);
        assert_eq!(add(REPAIRED | UNVERIFIED, Bits::IO), 52);
        assert_eq!(add(INVALID, Bits::INVALID), 2);
        for single in [USAGE, INTERNAL, STOPPED, 137, 255] {
            assert_eq!(add(single, Bits::IO), single);
            assert_eq!(Bits::from_code(single), None);
        }
        for c in 0..=63u8 {
            let b = Bits::from_code(c).unwrap();
            assert_eq!(b.to_exit_code(), c);
            assert_eq!(add(c, Bits::OK), c);
        }
        assert_eq!(Bits::from_code(0), Some(Bits::OK));
        assert_eq!(Bits::from_code(20), Some(Bits::REPAIRED | Bits::UNVERIFIED));
    }

    /// The numbers are the spec's (§3), pinned one by one: a renumber here is a
    /// constellation-wide change, never a local one.
    #[test]
    fn the_numbers_are_the_spec_s() {
        assert_eq!(
            [OK, NO, INVALID, UNVERIFIED, REFUSED, REPAIRED, IO, USAGE, INTERNAL, STOPPED],
            [0, 1, 2, 4, 8, 16, 32, 64, 101, 130]
        );
    }

    #[test]
    fn every_sum_of_the_six_bits_is_unique_and_below_64() {
        let all = [
            Bits::NO,
            Bits::INVALID,
            Bits::UNVERIFIED,
            Bits::REFUSED,
            Bits::REPAIRED,
            Bits::IO,
        ];
        let mut seen = std::collections::HashSet::new();
        for mask in 0u8..64 {
            let mut b = Bits::OK;
            for (i, bit) in all.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    b |= *bit;
                }
            }
            let code = b.to_exit_code();
            assert!(code < USAGE, "{code}");
            assert!(seen.insert(code), "{code} twice");
            for (i, bit) in all.iter().enumerate() {
                assert_eq!(b.contains(*bit), mask & (1 << i) != 0);
            }
        }
        assert_eq!(seen.len(), 64);
    }

    #[test]
    fn or_and_contains() {
        let b = Bits::REPAIRED | Bits::INVALID;
        assert_eq!(b.to_exit_code(), 18);
        assert!(b.contains(Bits::REPAIRED) && b.contains(Bits::INVALID));
        assert!(!b.contains(Bits::UNVERIFIED));
        assert!(b.contains(Bits::OK));
        assert!(Bits::OK.is_ok() && Bits::default().is_ok() && !b.is_ok());
    }

    /// R11's legend, word for word.
    #[test]
    fn the_legend_is_the_spec_s_text() {
        assert_eq!(
            HELP_LEGEND,
            "Exit status: 0 = done, every check passed. 64 = bad command line, nothing ran. \
             101 = internal error. 130 = stopped by the operator. Other codes from 128 up = \
             killed by a signal. Otherwise the sum (1 to 63) of: 1 a check said no; 2 an input \
             is invalid; 4 verify the result before trusting it; 8 refused (network mismatch, \
             secret on the command line, unsafe output); 16 a correction was applied; 32 a \
             file, device or helper failed."
        );
    }
}
