# Hashlock H1b — `me` bumps to ms-codec 0.8 Implementation Plan
**IMPLEMENTATION RECORD (2026-09-04, git's clock):** executed on branch `hashlock-h1b` (ONE opus implementer; report `design/agent-reports/hashlock-H1b-implementation-report.md`): commits 51f25c9 (Tasks 1+2), 2bb3f3b (Task 3), 6f4edf8 (records: me 0.8.1 unreleased, F-473 closed, F-454 advanced, F-475 filed), 278a0e4 (report); merged to master with `--no-ff` at 8c83e4e. Controller gate at 278a0e4 (own target dir, `--no-fail-fast`): fmt clean; nextest 619 run / 616 passed / 3 failed (box-local `history_purge` trio) / 2 skipped; clippy only the pre-existing `is_multiple_of` nightly lint; `Cargo.lock` moved ms-codec only; both verbs exit 4 on the plate. Post-implementation review (opus, `hashlock-H1b-post-impl.md`, 0C/0I/3M/2N, GREEN): 19 kind-space families through both verbs (6 place, 13 refuse), 6 killing mutations, 4 survivors wording-only. Minors folded at 4d5b6b7: **M-1** the CHANGELOG and `preimage_plate`'s doc claimed every wrong-X `0x03` single is named a plate; the codec reaches `PreimageLengthMismatch` only when the string length sits in the profile's length sets (X in {16, 17, 20, 21, 24, 25, 28, 29, 32, 33}), so X = 18 is `Bip93OutsideTheProfile(53)` -- claim narrowed, behaviour unchanged; **M-3** two assertions added (a `0x03` 2-of-N share is not a plate; the UPPERCASE plate is); **N-1** `:136` -> `:137`; **N-2** the widened profile sentence now says a `hash` id is refused for its kind; **M-2** (argv guard does not cover a plate; secret-handling) filed as F-476. Deviations D-1/D-2 and observations O-1/O-2 accepted by the reviewer. Release of me 0.8.1: fable decision `decision-me-0.8.1-release-and-flash-rule.md`.


**STATUS: R0 GREEN 2026-09-05 (0 Critical / 0 Important open).** Round 0: fidelity (opus, `hashlock-H1b-plan-R0-r0-fidelity.md`, 0C/2I/6M/2N) + tests/mutation (sonnet, `hashlock-H1b-plan-R0-r0-tests.md`, 0C/4I/2M), one fold (`b7ced42`, gate re-run green). Round 1: fold verification (sonnet, `hashlock-H1b-plan-R0-r1-fold-verification.md`): GREEN — all six Importants reproduce as FIXED from the plan's text (five build/test stages, four mutations, both host verbs); 2 Minors + 1 Nit (an anchor quote, an unstated comment text, a blank line) folded here as wording. Lens-closure: fidelity, tests/mutation, fold-verification. Baseline for staleness: engrave `0f5ce23`.
Previous STATUS: R0 round 0 folded (`b7ced42`), r1 pending.
Previous STATUS: build gate green at `e672194`, R0 dispatched.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `me`'s ms-codec pin from `0.7` to `0.8` (now on crates.io) so
`me` reads the released wire, while a kind-`0x03` PREIMAGE plate stays refused
by NAME on both host verbs — on the codec's SUCCESS path now, not by the
accident of an old pin — and an id/kind mismatch is diagnosed as what it is.

**Architecture:** `validate_record` matches the decoded payload: `Payload::Preimage`
→ `RecordError::PreimagePlate` (never `RecordKind::Ms`); every other payload →
`RecordKind::Ms`; the error path keeps naming the kind for a codec that refuses
the prefix, so the diagnosis is pin-independent. `preimage_plate` answers for
both codec behaviours and names EVERY kind-0x03 single the way the device does — by
shape (unshared, 33-byte payload, first byte 0x03) or by the codec's own kind
errors — with the L24 mismatch as the one deliberate exception. `unknown_reason` gains a `TagKindMismatch` arm ahead of
the profile arm. Follow-up F-473 closes; H0's three tripwire tests are the RED
step and stay as the guard. `me` becomes 0.8.1 (CHANGELOG; F-454 is due here too), release not in scope.

**Tech Stack:** Rust (`me-cli`, package `mnemonic-engrave`), ms-codec 0.8.0 from
crates.io (published 2026-09-05T03:28:43Z, tag `ms-codec-v0.8.0` = ms `cd0a60f`).

**Spec:** `mnemonic-secret/design/SPEC_ms_hashlock.md` §3 (0.8.0 API: `Payload::Preimage`,
`#[non_exhaustive]` `Payload`), §9 (H0 (b): `validate_record` treats `0x03` as
inert in the same window as the bump), §1 rule 2 (`TagKindMismatch`, ruling L24).
Follow-up: `design/FOLLOWUPS.md` F-473.

**Baselines (for `scripts/plan-staleness-check.sh`):** mnemonic-engrave `0f5ce23`;
mnemonic-secret `cd0a60f` (the API cited below).

## Global Constraints

- **Rust-primary rule:** nothing here is decided in Go; the fork is untouched.
  H2 (the fork's `0x03` arm) carries its own provenance pin to ms `cd0a60f`.
- **The pin is `"0.8"`, not `=0.8.0`.** `me` is an application, not a lockstep
  sibling; `Cargo.lock` pins the exact version (`ms-codec 0.8.0`) and CI runs
  `--locked`. (ms-cli's `=0.8.0` is its own rule, spec §10.)
- **No `vendor/` tree in this repo** (unlike ms and the toolkit): the lockfile
  update is the whole dependency change. `cargo update -p ms-codec` only —
  never a bare `cargo update`.
- **Source-breaking surface, checked by the compiler:** `Payload` is
  `#[non_exhaustive]` (every match on it carries a wildcard arm — Task 2's does,
  and that arm REFUSES: a payload kind a future ms-codec minor adds must not be
  placed as a seed until `me` has decided what it is; the compiler cannot warn,
  so the wildcard is the guard — fidelity I-2);
  `InspectKind` is not, but `me` never matches on it (grep at the baseline:
  `PayloadKind::`/`Payload::` in `me` refer to `me`'s own `sysw` types, not the
  codec's). If the bump surfaces any other exhaustive match, the compiler names
  it and the implementer adds the arm — recorded as a deviation.
- **A preimage plate is refused on BOTH host verbs** (`me seal`, `me sysw pack`)
  with the same words as H0 (`RecordError::PreimagePlate` / `UnknownReason::PreimagePlate`);
  the RED step is H0's three tripwires going red at the bump, exactly as F-473
  predicts, and GREEN is them passing again for the right reason.
- **Secret-handling defects never gate** (operator ruling 2026-08-27).
- **Stage paths explicitly**; commit per task; trailers as this repo uses.

---

## File Structure

| File | Change | Responsibility |
| --- | --- | --- |
| `crates/me-cli/Cargo.toml:53` | Modify | `ms-codec = "0.8"` |
| `Cargo.lock` | Modify (`cargo update -p ms-codec`) | ms-codec 0.7.0 → 0.8.0 and its new transitive deps (pbkdf2, hmac, sha2 …) |
| `crates/me-cli/src/seal/record.rs` (`validate_record`'s `Format::Ms` arm; `preimage_plate`) | Modify (fragments) | the success-path refusal; the pin-independent predicate |
| `crates/me-cli/src/sysw/mod.rs` (`UnknownReason`, `unknown_reason`, tests) | Modify (fragments) | `TagKindMismatch` diagnosis |
| `crates/me-cli/src/main.rs` (the `U::PreimagePlate` Display arm's neighbour) | Modify (fragment) | its Display text |
| `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs` | Modify (append) | the bump-specific witness: a `Payload::Preimage` DECODES and is still refused |
| `crates/me-cli/Cargo.toml:3`, `crates/me-cli/CHANGELOG.md` | Modify | `0.8.1`, the entry |
| `design/FOLLOWUPS.md` (F-473) | Modify | closed with the commit SHA |

**Gate coverage.** No extractor script fits (every edit is a fragment of an
existing file); the controller hand-wires the fragments into a worktree with
its OWN `CARGO_TARGET_DIR`, runs the RED step (bump only), then GREEN and the
mutations, and quotes the output in the fold/plan commit. Whole-crate runs use
`--no-fail-fast` (the three `history_purge` failures are box-local).

---

### Task 1: The bump — and watch H0's tripwires go RED

**Files:**
- Modify: `crates/me-cli/Cargo.toml:53` (`ms-codec = "0.7"` → `"0.8"`)
- Modify: `Cargo.lock` via `cargo update -p ms-codec`

- [ ] **Step 1: Bump and lock.**

```toml
ms-codec = "0.8"
```

Run: `cargo update -p ms-codec && grep -A1 'name = "ms-codec"' Cargo.lock`
Expected: `version = "0.8.0"` — one package moves (`Updating ms-codec v0.7.0 -> v0.8.0`);
`pbkdf2`, `hmac` and `sha2` are ALREADY in the lockfile through other
dependencies, so no new crates enter (fidelity M-1; measured).

- [ ] **Step 2: Build — the compiler's list is the deviation list.**

Run: `cargo build --locked -p mnemonic-engrave`
Expected: builds. (If any `match` in `me` on a codec enum fails to compile, the
compiler names the site; add the arm minimally and record it.)

- [ ] **Step 3: The RED step — H0's tripwires.**

Run: `cargo nextest run --locked -p mnemonic-engrave -E 'test(/preimage/)'`
Expected (measured at the gate; the tests lens re-ran it): at the bare bump the
codec DECODES the plate and `.map(|_| RecordKind::Ms)` calls it a seed, so
every witness of H0's refusal fails by ADMISSION, not by the profile arm:
`a_preimage_plate_is_not_a_seed_record` panics `validate_record admitted a 0x03
preimage plate as Ms`; `seal_names_a_preimage_plate_and_never_echoes_it` and
`sysw_pack_names_a_preimage_plate_and_never_echoes_it` fail because both verbs
now ACCEPT the plate (no refusal, no named kind); the sysw unit test
`a_preimage_plate_is_named_not_misdiagnosed` gets `Ok(..)` where it expects the
named refusal; the seam test fails `preimage-plate-0x03: host verdict`; and
`record_corpus::every_corpus_record_classifies_as_it_did_before_s2` fails
because the capture says `Unknown` and the host now says `Codex32Secret` (tests
I-4: SIX failures beyond `history_purge` at the bare bump — the filtered run
`-E 'test(/preimage/) | test(/the_host_never_admits/)'` shows five, the whole
crate six). Quote them in the commit. **Do not commit yet** — a
red tree is not a commit; Task 2 lands with it.

---

### Task 2: The refusal on the success path, and the predicate

**Files:**
- Modify: `crates/me-cli/src/seal/record.rs` — `validate_record`'s `Format::Ms` arm (anchor: `ms_codec::decode(s).map(|_| RecordKind::Ms).map_err(|e| {`) and `pub fn preimage_plate`
- Modify: `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs` (append)

**Interfaces:**
- Consumes: `ms_codec::decode(s: &str) -> Result<(Tag, Payload), ms_codec::Error>`; `ms_codec::Payload::Preimage(Zeroizing<[u8; 32]>)` (`#[non_exhaustive]` enum); `ms_codec::Error::{ReservedPrefixViolation { got }, TagKindMismatch { tag, prefix }}`; `RecordError::PreimagePlate` (H0).
- Produces: nothing new; `preimage_plate(&str) -> bool` keeps its signature.

- [ ] **Step 1: Replace the decode arm.** In `validate_record`, replace the block that starts `ms_codec::decode(s).map(|_| RecordKind::Ms).map_err(|e| {` and ends at its closing `})` (the whole `map`/`map_err` expression, keeping the two comment lines above it) with:

```rust
            // ms_codec::decode, NOT decode_with_correction — a seed that needed
            // repair must be fixed at source, not engraved.
            //
            // H1b (SPEC_ms_hashlock §9, F-473): at ms-codec 0.8 the codec DECODES a
            // kind-0x03 string as `Payload::Preimage`. It is not a seed record and
            // `RecordKind::Ms` is never answered for it — the arm is on the SUCCESS
            // path. The error path keeps naming the kind for a codec that still
            // refuses the prefix (0.7 behaviour), so the diagnosis survives either
            // pin; `me seal` and `me sysw pack` both land here.
            match ms_codec::decode(s) {
                Ok((_, ms_codec::Payload::Preimage(_))) => Err(RecordError::PreimagePlate),
                Ok((_, ms_codec::Payload::Entr(_) | ms_codec::Payload::Mnem { .. })) => {
                    Ok(RecordKind::Ms)
                }
                // `Payload` is #[non_exhaustive]: a kind a future ms-codec minor
                // adds arrives HERE, silently, and must not be placed as a seed
                // until `me` has decided what it is. Refuse; the compiler cannot
                // warn (R0 r0 fidelity I-2).
                Ok(_) => Err(RecordError::Invalid(
                    "an ms1 payload kind this me does not know; refusing to place it as a seed"
                        .to_string(),
                )),
                Err(ms_codec::Error::TagKindMismatch { .. }) => Err(RecordError::TagKindMismatch),
                Err(e) => {
                    if preimage_plate(s) {
                        Err(RecordError::PreimagePlate)
                    } else {
                        Err(RecordError::Invalid(e.to_string()))
                    }
                }
            }
```

- [ ] **Step 2: Re-point the predicate — by SHAPE, plus the codec's own kind errors.** Replace the whole of `pub fn preimage_plate` (from its doc comment's first line, which at `0f5ce23` reads `/// Is `s` a string of the hashlock PREIMAGE KIND (SPEC_ms_hashlock §1: kind`, to the function's closing brace; the function name is the anchor — r1 N-1) with the two functions below. The device's `codex32.IsPreimage` (H0) is shape-exact — unshared, 33-byte payload, first byte `0x03` — and the host must NAME the same population whatever the id says (fidelity I-1: at 0.8 a `0x03` single under an unknown id decodes to `UnknownTag`, and one with a wrong X length to `PreimageLengthMismatch`; neither is `Ok(Preimage)`, so a predicate keyed on the decode result alone regresses to "outside the profile"). The L24 mismatch is the one deliberate exception and gets its own predicate:

```rust
/// Is `s` an `ms`-HRP single whose 4-character id and kind byte disagree
/// (SPEC_ms_hashlock §1 rule 2, ruling L24)? Refused, never read by either
/// field; diagnosed by name on both host verbs (H1b). The HRP gate comes
/// first, as in its two siblings, so a non-`ms` string never reaches the codec.
pub fn id_kind_mismatch(s: &str) -> bool {
    let s = s.trim();
    matches!(classify(s), Ok(Format::Ms))
        && matches!(
            ms_codec::decode(s),
            Err(ms_codec::Error::TagKindMismatch { .. })
        )
}

/// Is `s` a hashlock PREIMAGE plate — an `ms`-HRP, codex32-valid, UNSHARED
/// single whose kind byte is `0x03` (SPEC_ms_hashlock §1)? A well-formed plate
/// is 75 characters with id `hash`; a `0x03` single under any other id, or with
/// a wrong X length, is named the same way, because the KIND is the prefix byte
/// (the device's `codex32.IsPreimage` tests the same shape). Pin-independent
/// (H1b, F-473): 0.7 refused the kind with `ReservedPrefixViolation { got: 3 }`,
/// 0.8 decodes it or names its length; both answer `true` here.
pub fn preimage_plate(s: &str) -> bool {
    let s = s.trim();
    if !matches!(classify(s), Ok(Format::Ms)) {
        return false;
    }
    // An id/kind MISMATCH is diagnosed separately (ruling L24), never as a plate.
    if id_kind_mismatch(s) {
        return false;
    }
    // A kind-0x03 single whose X is not 32 bytes: the codec names the kind
    // itself (`PreimageLengthMismatch`), and so does this (tests I-3).
    if matches!(
        ms_codec::decode(s),
        Err(ms_codec::Error::PreimageLengthMismatch { .. })
    ) {
        return true;
    }
    // The KIND is the prefix byte on an UNSHARED single with a 33-byte payload —
    // the same shape the device's `codex32.IsPreimage` tests (H0) — so a
    // malformed or mistagged 0x03 single is named the same way on both sides,
    // and a share or a plain 16..32-byte BIP-93 secret whose first byte happens
    // to be 0x03 is not. BIP-93 layout: `ms1` + threshold char + 4-char id +
    // share index; `0` and `s` mean unshared.
    let b = s.as_bytes();
    let unshared = b.get(3).is_some_and(|c| *c == b'0')
        && b.get(8).is_some_and(|c| c.eq_ignore_ascii_case(&b's'));
    unshared
        && match ms_codec::codex32::Codex32String::from_string(s.to_string()) {
            Ok(c) => {
                let d = c.parts().data();
                d.len() == 33 && d[0] == 0x03
            }
            Err(_) => false,
        }
}
```

(`Codex32String::from_string` and `Parts::data` are the codec's public surface — `me` already calls the former at `record.rs:187`; `parts()` borrows, so bind `c` first. Step 1's `TagKindMismatch` arm and this predicate agree by construction: the mismatch is refused there and excluded here.)

Add the `RecordError::TagKindMismatch` variant (fidelity M-3 — `me seal` names the mismatch with the same words as `sysw pack`), after `    PreimagePlate,` in `RecordError`:

```rust
    /// An ms1 single whose 4-character id and kind byte disagree
    /// (SPEC_ms_hashlock §1 rule 2, ruling L24): refused, never read by either
    /// field. `me seal` and `me sysw pack` both name it (R0 r0 fidelity M-3).
    TagKindMismatch,
```

and its Display arm immediately before `RecordError::PreimagePlate => write!(`:

```rust
            RecordError::TagKindMismatch => write!(
                f,
                "this ms1 string's 4-character id and kind byte disagree; it is refused rather \
                 than read by either field (SPEC_ms_hashlock §1 rule 2). A damaged or forged \
                 plate — re-encode it from the source rather than editing the string."
            ),
```

Two comments and one operator message become false at 0.8 and are fixed here (fidelity M-5): `bip93_outside_the_profile`'s doc line `/// must be `entr` — so plain BIP-93 secrets (48 and 74 characters) and BIP-93` → `/// must be `entr` (a seed) or, since ms-codec 0.8, `hash` (a hashlock preimage\n/// plate) — so plain BIP-93 secrets (48 and 74 characters) and BIP-93`; the `Bip93OutsideTheProfile` variant doc's `/// the 4-character id `entr`), so plain BIP-93 secrets at 48 and 74` → `/// the 4-character id `entr` for a seed or `hash` for a hashlock preimage\n/// plate), so plain BIP-93 secrets at 48 and 74`; and in `main.rs`'s `U::Bip93OutsideTheProfile` text, `be `entr`. This one is {len} characters.` → `be `entr` (a seed) or `hash` (a hashlock preimage plate). This one is {len} \\\n                     characters.` (keep the `\` line continuation; `cargo fmt` does not reflow string literals).

- [ ] **Step 3: The bump-specific witness.** Append to `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs`:

```rust
/// H1b (F-473): the codec now DECODES the plate — prove the refusal is on the
/// success path, not an accident of the pin. MUTATION: replace the
/// `Ok((_, Payload::Preimage(_)))` arm with `Ok(_) => Ok(RecordKind::Ms)` (i.e.
/// delete the arm) -> this fails on `decoded`, and
/// `a_preimage_plate_is_not_a_seed_record` fails with "admitted ... as Ms".
#[test]
fn the_codec_decodes_the_plate_and_me_still_refuses_it() {
    let decoded = ms_codec::decode(PREIMAGE_PLATE);
    assert!(
        matches!(decoded, Ok((_, ms_codec::Payload::Preimage(_)))),
        "ms-codec did not decode the plate as Payload::Preimage: {decoded:?}"
    );
    assert!(matches!(
        validate_record(PREIMAGE_PLATE),
        Err(RecordError::PreimagePlate)
    ));
}
```

(The integration test names `ms_codec` directly: a bin crate's `[dependencies]`
are visible to its integration tests — verified at the gate, no
`[dev-dependencies]` entry needed. Its name carries no "preimage", so run it by
its own filter: `-E 'test(/the_codec_decodes/)'`. The message names no version —
`env!("CARGO_PKG_VERSION")` here would print `mnemonic-engrave`'s, fidelity M-4.)

- [ ] **Step 4: GREEN, then the mutations.**

Run: `cargo nextest run --locked -p mnemonic-engrave -E 'test(/preimage/) | test(/the_host_never_admits/) | test(/the_codec_decodes/)'`
Expected: 6 PASS (the four `preimage` tests, the seam test, the new witness) —
measured. Then add three shape assertions to the sysw unit test
`a_preimage_plate_is_named_not_misdiagnosed` (fidelity I-1, tests I-3), before
its control line, and replace that test's recorded mutation comment (fidelity
M-5: at 0.8 the swap yields `Unrecognised`, not `Bip93OutsideTheProfile(75)`,
because the plate now decodes — the test still fails either way) with:

```rust
    /// MUTATION (measured at ms-codec 0.7): swap the two arms in `unknown_reason`
    /// -> `Bip93OutsideTheProfile(75)`. At 0.8 the plate DECODES, so
    /// `bip93_outside_the_profile` is false for it and the swap yields
    /// `Unrecognised` instead; the test still fails either way.
```

then the assertions:

```rust
        // R0 r0 fidelity I-1: the shape names a 0x03 single whatever its id,
        // exactly as the device does -- the seam corpus's `test`-id 33-byte row
        // is a plate here; its 48-char sibling (a 16-byte payload) is not.
        assert_eq!(
            pack(
                vec!["ms10testsqvrsu9guyv4rzwplgex4gkmzd9c8wl593jfe4gdg47mtm3xt6tv7qh3pm4xrfdlvvp".into()],
                None,
                ITER
            ),
            Err(SyswError::Unclassifiable(0, UnknownReason::PreimagePlate)),
        );
        assert!(!matches!(
            pack(vec!["ms10testsqv0qqqqqqqqqqqqqqqqqqqqqqq8mzk8tjfdnjn5".into()], None, ITER),
            Err(SyswError::Unclassifiable(0, UnknownReason::PreimagePlate))
        ));
        // R0 r0 tests I-3: a kind-0x03 single whose X is not 32 bytes (id hash,
        // 16-byte X, 50 characters) is refused by the codec as
        // PreimageLengthMismatch and is named a preimage plate here too.
        assert_eq!(
            pack(vec!["ms10hashsqw46h2at4w46h2at4w46h2at4w4ssrnvvaudn2k4d".into()], None, ITER),
            Err(SyswError::Unclassifiable(0, UnknownReason::PreimagePlate)),
        );
```

then `cargo fmt -p mnemonic-engrave` (the long literals wrap). Measured on the
wired tree: `me sysw pack` and `me seal` both name the 50-character malformed
plate at exit 4.

Mutations, each reverted (`touch` after restoring), measured: (a) delete the
`Ok((_, Payload::Preimage(_)))` arm → all five `preimage`/seam tests FAIL
(`admitted … as Ms`; `preimage-plate-0x03: host verdict`; the two binary tests;
the sysw unit test) and the witness FAILS on its second assertion — six in all
(tests lens: `6 failed, 0 passed`); (b) drop the shape clause (`d.len() == 33 &&
d[0] == 0x03` → `false`) from `preimage_plate` → `sysw_pack_names_a_preimage_plate_
and_never_echoes_it` FAILS `stderr does not name the kind` and
`a_preimage_plate_is_named_not_misdiagnosed` FAILS; `seal_names_…` stays green
because `validate_record`'s success-path arm already refuses it.

- [ ] **Step 5: Whole crate, commit (Tasks 1+2 together).**

Run: `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast && cargo clippy --locked -p mnemonic-engrave --all-targets -- -D warnings && cargo fmt -p mnemonic-engrave -- --check`
Expected: green but for the three box-local `history_purge` tests — measured
after Task 3 as well: 619 run, 616 passed, 3 failed (`history_purge` ×3); fmt
clean; clippy's one `manual implementation of .is_multiple_of()` in
`sysw/composer_records.rs:114` is the local nightly's, green in CI.

```bash
git add crates/me-cli/Cargo.toml Cargo.lock crates/me-cli/src/seal/record.rs crates/me-cli/tests/preimage_plate_is_not_a_seed.rs
git commit -m "me: ms-codec 0.8 -- a decoded Payload::Preimage is refused as a preimage plate on both host verbs; preimage_plate pin-independent (hashlock H1b, F-473)"
```

---

### Task 3: Name an id/kind mismatch

**Files:**
- Modify: `crates/me-cli/src/sysw/mod.rs` (`UnknownReason` after `PreimagePlate,`; `unknown_reason` before the `preimage_plate` arm; the tests module after `a_preimage_plate_is_named_not_misdiagnosed`)
- Modify: `crates/me-cli/src/main.rs` (before `U::PreimagePlate => format!(`)

At 0.8 the seam row `preimage-shape-entr-id` (`ms10entrsqv0…`, kind byte `0x03`
under id `entr`) decodes to `Err(TagKindMismatch { .. })` (ruling L24); without
this arm `me sysw pack` diagnoses it as "outside the profile" (75 characters,
listed as inside) — the I-3 misdiagnosis shape again, one row over.

- [ ] **Step 1: RED.** Append to the `sysw` tests module, after `a_preimage_plate_is_named_not_misdiagnosed`:

```rust
    /// H1b: an id/kind MISMATCH (SPEC_ms_hashlock §1 rule 2, ruling L24) is named
    /// as such, not as "outside the profile". The string is the seam corpus's
    /// `preimage-shape-entr-id` row: kind byte 0x03 under the id `entr`.
    /// MUTATION: remove the TagKindMismatch arm -> `Bip93OutsideTheProfile(75)`.
    #[test]
    fn an_id_kind_mismatch_is_named_not_misdiagnosed() {
        const MISMATCH: &str =
            "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9";
        assert_eq!(MISMATCH.chars().count(), 75);
        assert_eq!(
            pack(vec![MISMATCH.into()], None, ITER),
            Err(SyswError::Unclassifiable(0, UnknownReason::TagKindMismatch)),
        );
    }
```

Run: `cargo nextest run --locked -p mnemonic-engrave -E 'test(/an_id_kind_mismatch/)'`
Expected: does not compile (`UnknownReason::TagKindMismatch` undefined) — the
RED is a compile error; after Step 2's variant exists but before its arm, FAIL
with `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`. Run `cargo fmt
-p mnemonic-engrave` after every block in this task; the literal above is
wrapped so `cargo fmt --check` stays clean (tests I-2: the one-line form was 109
columns).

- [ ] **Step 2: The variant, the arm, the words.** After `    PreimagePlate,` in `UnknownReason`:

```rust
    /// An ms1 string whose 4-character id and kind byte disagree (SPEC_ms_hashlock
    /// §1 rule 2, `TagKindMismatch`, ruling L24 — refused, never read by either
    /// field). Damaged or forged: re-encode from the source rather than editing.
    TagKindMismatch,
```

In `unknown_reason`, before `if crate::seal::record::preimage_plate(record) {`:

```rust
    // Before the preimage and profile arms: a mismatch is inside the profile's
    // lengths and is neither a plate nor outside the profile (H1b, ruling L24).
    // The HRP gate lives in the helper, as in its two siblings (fidelity N-1).
    if crate::seal::record::id_kind_mismatch(record) {
        return UnknownReason::TagKindMismatch;
    }
```

In `main.rs`, before `U::PreimagePlate => format!(`:

```rust
                U::TagKindMismatch => format!(
                    "record {i} (records count from 0) is an ms1 string whose 4-character \
                     id and kind byte disagree; it is refused rather than read by either \
                     field (SPEC_ms_hashlock §1 rule 2). A damaged or forged plate — \
                     re-encode it from the source rather than editing the string."
                ),
```

Run: `cargo nextest run --locked -p mnemonic-engrave -E 'test(/misdiagnosed/)'`
Expected: 2 PASS (measured, with the helper form). Mutation, measured: remove the
arm → the new test FAILS with `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`;
the preimage test stays green (its arm is separate). Tests-lens mutation (iii),
the arm moved AFTER the profile arm, is caught by the same test.

- [ ] **Step 3: Commit.**

```bash
git add crates/me-cli/src/sysw/mod.rs crates/me-cli/src/main.rs
git commit -m "me sysw pack: name an id/kind mismatch (TagKindMismatch) instead of 'outside the profile' (hashlock H1b, ruling L24)"
```

---

### Task 4: Records

- [ ] `crates/me-cli/Cargo.toml` `version = "0.8.1"`; `Cargo.lock` follows on the next build. `crates/me-cli/CHANGELOG.md`: KEEP the single `## [Unreleased]` section (the file's style is `## [x.y.z] - YYYY-MM-DD` only at release time — fidelity N-2) and add the H1b items to it beside the two entries already there (H0's `### Added`, the `+`-signed `key:` path tightening's `### Changed`): the ms-codec `0.8` pin; the success-path `Payload::Preimage` refusal and the fail-closed wildcard; `preimage_plate` by shape; the `TagKindMismatch` diagnosis on both verbs. Rewrite H0's sentence *"At the pinned ms-codec `0.7` the codec's prefix gate already refuses the string; `tests/preimage_plate_is_not_a_seed.rs` is the tripwire that goes red at the `0.8` bump if the refusing arm is forgotten (follow-up F-473)"* in the past tense: *"H0 shipped against ms-codec 0.7, where the codec's prefix gate refused the string; H1b moved the refusal onto the codec's success path and the tripwire test now asserts the named variant."* (fidelity M-2).
- [ ] `design/FOLLOWUPS.md`: **F-473 closed** with the Task 2 commit SHA (`preimage_plate` pin-independent; the success-path arm). **F-454** (`me-0.8.1-owed-plus-sign-path-tightening-unreleased`, owning phase "cut me 0.8.1 with the next host change") is DUE here: this plan is the next host change and bumps the version; advance it to "released with 0.8.1" once the tag exists, or close it if the release cuts (fidelity M-2). File the seam-corpus prose correction the fidelity lens found (M-6: the `bip93-plain-33-byte-payload-0x03` row's `source` says the host refuses it "at 0.8 as a TagKindMismatch"; at 0.8 it is `UnknownTag { got: "test" }`) with owning phase **H2**, because editing that file re-pins `SEAM_VECTORS_SHA256` in BOTH repos and H2 vendors the corpus anyway.
- [ ] Post-implementation review (risk set: `me` packs what the device engraves): ONE opus adversarial execution review over the whole diff; brief `design/agent-briefs/hashlock-H1b-post-impl-brief.md`, report `design/agent-reports/hashlock-H1b-post-impl.md`; GREEN before merge.
- [ ] Continuity + memory; push engrave via `scripts/push-via-staging.sh master`. The `me` 0.8.1 RELEASE (tag, `release.yml` assemble + sign) is NOT this plan's — it is the operator's or a fable decision, recorded when taken.

---

## Self-review

1. **Spec coverage.** §9 H0 (b) "treats kind 0x03 as inert in the same release window as the 0.8 bump" → Task 2 Step 1 (the success-path arm) and Step 2 (the predicate), guarded by H0's tripwires (Task 1 Step 3 RED → Task 2 Step 4 GREEN) and the new witness. §1 rule 2 / L24 → Task 3. §3's `#[non_exhaustive]` `Payload` → the wildcard `Ok(_)` arm. F-473 both halves → Tasks 2 and 4.
2. **Placeholders.** None; Step 3's aside about the test crate's access to `ms_codec` is settled at the gate (integration tests of a bin crate see its `[dependencies]`).

**R0 round 0 folded here.** Fidelity: I-1 → the shape-exact `preimage_plate` (+ the codec's `PreimageLengthMismatch`; the L24 mismatch excluded via `id_kind_mismatch`), with the two corpus rows asserted in the sysw unit test; I-2 → positive `Entr | Mnem` arms and a REFUSING wildcard; M-1 → Step 1's Expected (no new crates); M-2 → Task 4 (one `[Unreleased]`, the H0 sentence in the past tense, F-454 due); M-3 → `RecordError::TagKindMismatch` on both verbs; M-4 → the witness message; M-5 → the mutation comment, both doc comments, the operator text; M-6 → filed with H2; N-1 → the HRP gate inside `id_kind_mismatch`; N-2 → the heading style. Tests: I-1 → the RED paragraph states the real mechanism (admission, six failures); I-2 → the wrapped literal and a fmt step; I-3 → the `PreimageLengthMismatch` clause and its assertion; I-4 → `record_corpus` named among the RED failures; M-1/M-2 recorded.

3. **Type consistency.** `ms_codec::decode(&str) -> Result<(Tag, Payload)>` (ms `crates/ms-codec/src/decode.rs:46` at `cd0a60f`); `Payload::Preimage(Zeroizing<[u8; 32]>)` (`payload.rs:46`); `Error::TagKindMismatch { tag: [u8; 4], prefix: u8 }` (`error.rs:67`); `Error::ReservedPrefixViolation { got: u8 }` (`error.rs:76`); `RecordError::PreimagePlate` (`record.rs:81`); `UnknownReason::PreimagePlate` (`sysw/mod.rs:158`); `pack(vec![...], None, ITER) -> Result<_, SyswError>` as the neighbouring test uses it.
