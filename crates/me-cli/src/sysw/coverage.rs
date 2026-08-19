//! The spec's 23 named tests, mapped to where each is actually covered.
//!
//! WHY THIS FILE EXISTS. The vector set is the contract the Go port inherits
//! (plan stage 3): whatever it omits, both implementations will agree about
//! incorrectly. Choosing the vectors from imagination is how that happens, so
//! they are derived from SPEC §8.3's named tests instead — and the derivation is
//! checked, not promised. [`assert_every_named_test_is_placed`] fails the build
//! if a spec test has no entry here.
//!
//! An entry may legitimately say "not this crate" — most of §8.3 is device or
//! CLI behaviour and cannot be a container vector. What it may not do is go
//! missing.

/// Where a spec §8.3 test is discharged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where {
    /// A JSON vector, checked by both Rust and the Go port.
    Vector(&'static str),
    /// A Rust unit test in this crate.
    Unit(&'static str),
    /// Belongs to `me`'s CLI layer — plan stage 2.
    Cli,
    /// Belongs to the device, and the behaviour EXISTS there.
    Device,
    /// Belongs to the device, and the behaviour DOES NOT EXIST YET.
    ///
    /// Added 2026-08-12, reconciling this column against the gui tree after the
    /// spec/plan review. `Device` had been carrying both meanings, so five of
    /// its ten entries named behaviour nobody had built — and this file, whose
    /// entire purpose is that "an unplaced test is a gap, a deferred one is a
    /// plan", was reading as coverage. A deferral is only a plan if it says it
    /// is one.
    DeviceUnbuilt(&'static str),
    /// Withdrawn by operator ruling, with the date. Not a gap: a decision.
    Dropped(&'static str),
}

/// Spec §8.3, test id → where it is discharged. **Every id 1..=23 must appear.**
pub const COVERAGE: &[(u32, Where)] = &[
    // §7.4, and now asserted against the §7 flow itself: plate_verify.go names
    // no session identifier at all, so a cached secret has no way to answer a
    // verification prompt. TestPlateVerifyComparesAgainstTheEngravedMnemonicNotTheSession.
    (1, Where::Device),
    // Built by plan stage 12 (gui/plate_verify.go). A wrong word at a CHECKED
    // position is caught and the failure names the position:
    // TestPlateVerifyCatchesAWrongWordAtACheckedPosition, with the passing
    // direction pinned too — a flow reporting every plate wrong would satisfy
    // the failing case alone.
    (2, Where::Device),
    // The draw is uniform and without replacement
    // (TestPlateVerifyDrawIsUniformAndWithoutReplacement, 20 000 draws), and
    // FRESH PER ATTEMPT at the flow (TestPlateVerifyRedrawsOnEveryAttempt) —
    // the second half is where the defect lives, because a correct draw hoisted
    // out of the retry loop passes the unit test.
    (3, Where::Device),
    (4, Where::Vector("S-B")), // plaintext + secret class -> F1
    (5, Where::Cli),           // me warns, does not refuse (§13 D3)
    (6, Where::Vector("S-C")), // byte-identical KDF input, arbitrary N
    (7, Where::Unit("the_sealed_payload_magic_is_refused_here")),
    (8, Where::Device),        // never "payload unreadable"
    (9, Where::Vector("S-C")), // digest shown where one exists
    (10, Where::Device),       // compared once per payload
    (11, Where::Unit("fills_the_whole_region")),
    (12, Where::Unit("each_fill_is_what_it_says")),
    (
        13,
        Where::Dropped("operator ruling 2026-08-12: the reminder is dropped"),
    ),
    // `[mdmk-decode]` (§12.6). Re-pointed 2026-08-12, off S-I: that vector
    // records only that a BCH-valid md1 packs, and asserted nothing about
    // confirmation, so the placement was a claim rather than coverage. S-J
    // carries BOTH answers, which is what makes it able to fail.
    (14, Where::Vector("S-J")),
    (15, Where::Vector("S-D")), // pub_len == 0 -> no digest
    (16, Where::Device),        // no verify flow reaches a payload secret
    // §7.1.1's four provenances, built by plan stage 12 and asserted over the
    // RENDERED strings rather than over the enum:
    // TestVerifyProvenanceIsNeverRenderedAsVerified. The entry previously read
    // "no provenance survives take()", which was about §3.2's record source and
    // not about §7.1.1 at all — a verification's provenance is produced by the
    // verify flow and never travels through the session.
    (17, Where::Device),
    (18, Where::Unit("every_byte_of_the_aad_is_bound")),
    (19, Where::Vector("S-E")), // generated N enterable for every N
    (20, Where::Vector("S-D")), // secrets-only consumable
    (21, Where::Device),        // passphrase buffer never regrows
    // §8c's count confirmation, built by plan stage 9 alongside the `done`
    // affordance the reachability of which this entry had assumed: the button's
    // Clickable and handler existed, but nothing ever drew it, and on a
    // touch-driven panel an undrawn nav button carries no hit target — so
    // `done` could not be pressed at all. Discharged by
    // TestSyswPassphraseDoneConfirmsTheShortCount (gui/sysw_load_test.go),
    // which taps the slot rather than synthesising the button event.
    (22, Where::Device),
    (23, Where::Vector("S-D")), // secrets-only usable whatever the passphrase
];

/// Every vector this crate must ship, derived from [`COVERAGE`] rather than
/// listed by hand — so a vector cannot be dropped while a spec test still
/// points at it.
pub fn required_vectors() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = COVERAGE
        .iter()
        .filter_map(|(_, w)| match w {
            Where::Vector(name) => Some(*name),
            _ => None,
        })
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// The vector set, as inputs. Expected outputs live in `testdata/sysw_vectors.json`
/// and are regenerated by `cargo test -p mnemonic-engrave --lib sysw::vectors -- --ignored`.
///
/// Names come from the plan; the ones §8.3 requires come from [`required_vectors`]
/// and are asserted present, so a spec test cannot point at a vector that
/// vanished.
pub struct VectorInput {
    pub name: &'static str,
    pub records: &'static [&'static str],
    pub passphrase: Option<&'static str>,
    pub note: &'static str,
}

/// Fixed so a fixture is a fixture. Production randomises; see `pack_deterministic`.
pub const FIXTURE_SALT: [u8; 16] = [0x11; 16];
pub const FIXTURE_IV: [u8; 12] = [0x22; 12];
pub const FIXTURE_ITERATIONS: u32 = 100_000;

/// Chunk 0 of 3 of chunk set 398802 — MEASURED, not assumed. It is therefore an
/// UNCONFIRMED record under `[mdmk-decode]` (§12.6) wherever it appears alone,
/// which is every vector below except S-J.
const MD1: &str = "md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3";
/// The other two chunks of set 398802, so S-J can carry a COMPLETE card.
const MD1_B: &str = "md1fv9wjpqg0yq82l0czvx85ae43vtfd26hsmngjecmqy44k2pgttqh74qwxlawq374";
const MD1_C: &str = "md1fv9wjpqsp2026hh65xpvugtfhd9792zxgunymm0a82pdju6442q0jskj9gzfaqmz";
/// Chunk 0 of 6 of a DIFFERENT set, 841149. Beside the complete 398802 set it is
/// what separates `(hrp, chunk_set_id)` grouping from grouping by HRP alone: an
/// implementation that lumped all four `md1` records together would report the
/// complete card as unconfirmed and fail this vector loudly.
const MD1_OTHER: &str =
    "md1fe4dazspq3m67zzqqvzrs3pstucnf4ztqz4pk6ujgjycfn6zhs79nmzdp9frd6dzth6asfu2za4mwgfkg6";
/// Not chunked at all: its own card, and it decodes on its own.
const MD1_SINGLE: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
/// A complete 2-chunk `mk1` card (set 74565).
const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
const MK1_B: &str =
    "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";
/// Chunk 0 of a 2-chunk `mk1` card (set 153721) whose second chunk is absent.
const MK1_LONE: &str = "mk1qpykrepqqspjtpuhfqjc096gykrewjy6dgjcqpcy3zepaggqseet8ky6z2jxm56yh04m5mqslrmueekdmecm0js2h978k03jfvkwz2rxj8r8";
const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const PASS12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const PASS2: &str = "abandon about";

pub const VECTORS: &[VectorInput] = &[
    VectorInput {
        name: "S-A",
        records: &["text:48656c6c6f2c20576f726c6421"],
        passphrase: None,
        note: "plaintext, one text: record — journey (b)",
    },
    VectorInput {
        name: "S-B",
        records: &[SEED],
        passphrase: None,
        note: "plaintext carrying a SECRET class — F1 flags it, decision 6 permits it",
    },
    VectorInput {
        name: "S-C",
        records: &[MD1, SEED],
        passphrase: Some(PASS12),
        note: "sealed with pub_len > 0 — a digest exists and is shown",
    },
    VectorInput {
        name: "S-D",
        records: &[SEED],
        passphrase: Some(PASS12),
        note: "sealed, secrets only, pub_len == 0 — NO digest; opening authenticates it",
    },
    VectorInput {
        name: "S-E",
        records: &[MD1, SEED],
        passphrase: Some(PASS2),
        note: "sealed under a 2-word passphrase — not [cliff]-above, still opens (§13 D1)",
    },
    VectorInput {
        name: "S-G",
        records: &["text:48656c6c6f2c20576f726c64210a7365636f6e64206c696e65"],
        passphrase: None,
        note:
            "a text: body containing a SPACE and the record SEPARATOR — the case §5.3.1 exists for",
    },
    VectorInput {
        name: "S-I",
        records: &[MD1],
        passphrase: None,
        note: "a BCH-valid md1, alone — it packs and classifies ClassMDMK, and that is ALL this \
               vector says; §8.3 test 14 moved to S-J, which asserts confirmation",
    },
    VectorInput {
        name: "S-J",
        records: &[
            MD1,        // 0 ┐
            MD1_B,      // 1 ├ chunk set 398802, COMPLETE  -> confirmed
            MD1_C,      // 2 ┘
            MD1_OTHER,  // 3   chunk 0 of 6 of set 841149  -> UNCONFIRMED
            MD1_SINGLE, // 4   not chunked, decodes        -> confirmed
            MK1_A,      // 5 ┐ chunk set 74565, COMPLETE   -> confirmed
            MK1_B,      // 6 ┘
            MK1_LONE,   // 7   chunk 0 of 2 of set 153721  -> UNCONFIRMED
        ],
        passphrase: None,
        note: "`[mdmk-decode]` (§12.6) — both answers in one payload: two complete card sets and \
               a non-chunked card confirm, two lone chunks do not. Unsealed, so the public \
               section is the record list in order and the reported indices are unambiguous",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// THE GATE. Adding a test to spec §8.3 without placing it here fails the
    /// build, which is the only reason this file is worth its weight.
    #[test]
    fn assert_every_named_test_is_placed() {
        const HIGHEST: u32 = 23;
        for id in 1..=HIGHEST {
            assert!(
                COVERAGE.iter().any(|(n, _)| *n == id),
                "spec §8.3 test {id} has no entry in COVERAGE — place it, even if \
                 the placement is Device or Cli. An unplaced test is a gap; a \
                 deferred one is a plan."
            );
        }
        assert_eq!(COVERAGE.len() as u32, HIGHEST, "no duplicate or stray ids");
    }

    /// The unbuilt column, named out loud.
    ///
    /// This does not fail — an unbuilt behaviour is a legitimate state — but it
    /// PRINTS, so `cargo test -- --nocapture` answers "what does this feature
    /// still not do" without anyone grepping. The reason F-144 survived a green
    /// R0 gate is that no artifact was obliged to say that out loud.
    ///
    /// It DOES fail if the list is empty while a `DeviceUnbuilt` entry exists,
    /// which is the only way this could rot into decoration.
    #[test]
    fn the_unbuilt_behaviours_are_listed_rather_than_implied() {
        let unbuilt: Vec<_> = COVERAGE
            .iter()
            .filter_map(|(n, w)| match w {
                Where::DeviceUnbuilt(why) => Some((*n, *why)),
                _ => None,
            })
            .collect();
        let dropped: Vec<_> = COVERAGE
            .iter()
            .filter_map(|(n, w)| match w {
                Where::Dropped(why) => Some((*n, *why)),
                _ => None,
            })
            .collect();
        println!("spec §8.3 tests whose behaviour does NOT exist yet:");
        for (n, why) in &unbuilt {
            println!("  {n:>2}  {why}");
        }
        println!("withdrawn by ruling:");
        for (n, why) in &dropped {
            println!("  {n:>2}  {why}");
        }
        assert!(
            COVERAGE
                .iter()
                .any(|(_, w)| matches!(w, Where::DeviceUnbuilt(_)))
                == !unbuilt.is_empty(),
            "the unbuilt list must reflect the table"
        );
    }

    /// The vector names are derived, so this pins the derivation rather than a
    /// hand-copied list.
    #[test]
    fn the_required_vectors_are_derived_from_the_coverage_map() {
        let v = required_vectors();
        assert!(
            v.contains(&"S-D"),
            "the pub_len == 0 case must have a vector"
        );
        assert!(
            v.contains(&"S-J"),
            "`[mdmk-decode]` must have a vector — §8.3 test 14"
        );
        assert!(
            !v.contains(&"S-I"),
            "S-I is no longer REQUIRED by any spec test: it asserted nothing about \
             confirmation, which is why test 14 moved to S-J. It still ships, and the \
             golden and round-trip tests still cover it — this pins that the derivation \
             followed the move rather than the list being hand-edited."
        );
        // Three separate spec tests ride on S-D; it must appear once.
        assert_eq!(v.iter().filter(|n| **n == "S-D").count(), 1);
    }

    /// Most of §8.3 is device behaviour, and that is expected — but if it were
    /// ALL device behaviour this crate would be shipping no vectors at all,
    /// which is the failure this file exists to make visible.
    #[test]
    fn the_container_carries_a_meaningful_share_of_the_named_tests() {
        let here = COVERAGE
            .iter()
            .filter(|(_, w)| matches!(w, Where::Vector(_) | Where::Unit(_)))
            .count();
        assert!(
            here >= 8,
            "only {here} of 23 named tests land in this crate"
        );
    }
}
