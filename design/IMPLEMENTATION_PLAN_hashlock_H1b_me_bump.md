# Hashlock H1b — `me` bumps to ms-codec 0.8 Implementation Plan

**STATUS: DRAFT 2026-09-05 — BUILD GATE GREEN (controller hand-wire in `me-worktrees/h1b-gate`, own target dir; output in this file's commit message); R0 not yet dispatched.**

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `me`'s ms-codec pin from `0.7` to `0.8` (now on crates.io) so
`me` reads the released wire, while a kind-`0x03` PREIMAGE plate stays refused
by NAME on both host verbs — on the codec's SUCCESS path now, not by the
accident of an old pin — and an id/kind mismatch is diagnosed as what it is.

**Architecture:** `validate_record` matches the decoded payload: `Payload::Preimage`
→ `RecordError::PreimagePlate` (never `RecordKind::Ms`); every other payload →
`RecordKind::Ms`; the error path keeps naming the kind for a codec that refuses
the prefix, so the diagnosis is pin-independent. `preimage_plate` answers for
both codec behaviours. `unknown_reason` gains a `TagKindMismatch` arm ahead of
the profile arm. Follow-up F-473 closes; H0's three tripwire tests are the RED
step and stay as the guard. `me` becomes 0.8.1 (CHANGELOG), release not in scope.

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
  `#[non_exhaustive]` (every match on it carries a wildcard arm — Task 2's does);
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
Expected: `version = "0.8.0"`; the lockfile also gains `pbkdf2`, `hmac`, `sha2`
(and their deps) as ms-codec 0.8.0 requires them.

- [ ] **Step 2: Build — the compiler's list is the deviation list.**

Run: `cargo build --locked -p mnemonic-engrave`
Expected: builds. (If any `match` in `me` on a codec enum fails to compile, the
compiler names the site; add the arm minimally and record it.)

- [ ] **Step 3: The RED step — H0's tripwires.**

Run: `cargo nextest run --locked -p mnemonic-engrave -E 'test(/preimage/)'`
Expected (measured at the gate): FAIL ×4 of the `preimage` tests —
`a_preimage_plate_is_not_a_seed_record` panics with
`validate_record admitted a 0x03 preimage plate as Ms` (the codec now DECODES it
and `.map(|_| RecordKind::Ms)` calls it a seed); the sysw unit test
`a_preimage_plate_is_named_not_misdiagnosed`,
`sysw_pack_names_a_preimage_plate_and_never_echoes_it` and
`seal_names_a_preimage_plate_and_never_echoes_it` fail (`preimage_plate` asked for
`ReservedPrefixViolation { got: 3 }` BY NAME, which 0.8 no longer returns, so
`me sysw pack` falls to the profile arm and `me seal` admits the plate). Also
run the seam test:
`cargo nextest run --locked -p mnemonic-engrave --test codex32_seam` — its
`preimage-plate-0x03` row FAILS on `host verdict` (the host now admits the plate
as `Codex32Secret`). Five failures in one filtered run
(`-E 'test(/preimage/) | test(/the_host_never_admits/)'`: 0 passed, 5 failed);
quote them in the commit. **Do not commit yet** — a
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
                Ok(_) => Ok(RecordKind::Ms),
                Err(e) => {
                    if preimage_plate(s) {
                        Err(RecordError::PreimagePlate)
                    } else {
                        Err(RecordError::Invalid(e.to_string()))
                    }
                }
            }
```

- [ ] **Step 2: Re-point the predicate.** Replace the body of `pub fn preimage_plate` (anchor: `Err(ms_codec::Error::ReservedPrefixViolation { got: 0x03 })`):

```rust
/// Is `s` a hashlock PREIMAGE plate — an `ms`-HRP, codex32-valid string whose
/// kind byte is `0x03` (SPEC_ms_hashlock §1; a well-formed plate is 75
/// characters with id `hash`; a malformed `0x03` string is named the same way)?
///
/// Pin-independent (H1b, F-473): ms-codec 0.8 DECODES the kind as
/// `Payload::Preimage`; 0.7 refused it with `ReservedPrefixViolation { got: 3 }`.
/// Both answer `true` here, so the diagnosis in `unknown_reason` and
/// `validate_record` does not depend on which codec is pinned. An id/kind
/// MISMATCH (`TagKindMismatch`, ruling L24) is NOT a preimage plate: it is
/// diagnosed separately.
pub fn preimage_plate(s: &str) -> bool {
    let s = s.trim();
    matches!(classify(s), Ok(Format::Ms))
        && matches!(
            ms_codec::decode(s),
            Ok((_, ms_codec::Payload::Preimage(_)))
                | Err(ms_codec::Error::ReservedPrefixViolation { got: 0x03 })
        )
}
```

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
        "ms-codec {} did not decode the plate as Payload::Preimage: {decoded:?}",
        env!("CARGO_PKG_VERSION")
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
its own filter: `-E 'test(/the_codec_decodes/)'`.)

- [ ] **Step 4: GREEN, then the mutations.**

Run: `cargo nextest run --locked -p mnemonic-engrave -E 'test(/preimage/) | test(/the_host_never_admits/) | test(/the_codec_decodes/)'`
Expected: 6 PASS (the four `preimage` tests, the seam test, the new witness) —
measured. Mutations, each reverted (`touch` after restoring), measured: (a) delete
the `Ok((_, Payload::Preimage(_)))` arm → all five `preimage`/seam tests FAIL
(`admitted … as Ms`; `preimage-plate-0x03: host verdict`; the two binary tests;
the sysw unit test) and the witness FAILS on its second assertion
(`matches!(validate_record(PREIMAGE_PLATE), Err(RecordError::PreimagePlate))`);
(b) drop the `Ok((_, Payload::Preimage(_)))` alternative from `preimage_plate` →
`sysw_pack_names_a_preimage_plate_and_never_echoes_it` FAILS `stderr does not name
the kind` and `a_preimage_plate_is_named_not_misdiagnosed` FAILS (the profile arm
claims the plate again); `seal_names_…` stays green because `validate_record`'s
success-path arm already refuses it — the predicate matters to `sysw pack`'s
diagnosis, the arm to both verbs.

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
        const MISMATCH: &str = "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9";
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
with `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`.

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
    if matches!(
        ms_codec::decode(record.trim()),
        Err(ms_codec::Error::TagKindMismatch { .. })
    ) {
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
Expected: 2 PASS (measured). Mutation, measured: remove the arm → the new test
FAILS with `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`; the
preimage test stays green (its arm is separate).

- [ ] **Step 3: Commit.**

```bash
git add crates/me-cli/src/sysw/mod.rs crates/me-cli/src/main.rs
git commit -m "me sysw pack: name an id/kind mismatch (TagKindMismatch) instead of 'outside the profile' (hashlock H1b, ruling L24)"
```

---

### Task 4: Records

- [ ] `crates/me-cli/Cargo.toml` `version = "0.8.1"`; `Cargo.lock` follows (`cargo update -p mnemonic-engrave --offline` or the build's own rewrite); `crates/me-cli/CHANGELOG.md` gets a `## [0.8.1] — unreleased` section: the ms-codec 0.8 pin; the success-path refusal; the mismatch diagnosis; "H0 shipped in 0.8.x unreleased" folded in as the record shows.
- [ ] `design/FOLLOWUPS.md` F-473: **closed** with the Task 2 commit SHA; note that `preimage_plate` is pin-independent.
- [ ] Post-implementation review (risk set: a reader that cuts seeds — `me` packs what the device engraves): ONE opus adversarial execution review over the whole diff; brief `design/agent-briefs/hashlock-H1b-post-impl-brief.md`, report `design/agent-reports/hashlock-H1b-post-impl.md`; GREEN before merge.
- [ ] Continuity + memory; push engrave via `scripts/push-via-staging.sh master`. The `me` 0.8.1 RELEASE (tag, `release.yml` assemble + sign) is NOT this plan's — it is the operator's or a fable decision, recorded when taken.

---

## Self-review

1. **Spec coverage.** §9 H0 (b) "treats kind 0x03 as inert in the same release window as the 0.8 bump" → Task 2 Step 1 (the success-path arm) and Step 2 (the predicate), guarded by H0's tripwires (Task 1 Step 3 RED → Task 2 Step 4 GREEN) and the new witness. §1 rule 2 / L24 → Task 3. §3's `#[non_exhaustive]` `Payload` → the wildcard `Ok(_)` arm. F-473 both halves → Tasks 2 and 4.
2. **Placeholders.** None; Step 3's aside about the test crate's access to `ms_codec` is settled at the gate (integration tests of a bin crate see its `[dependencies]`).
3. **Type consistency.** `ms_codec::decode(&str) -> Result<(Tag, Payload)>` (ms `crates/ms-codec/src/decode.rs:46` at `cd0a60f`); `Payload::Preimage(Zeroizing<[u8; 32]>)` (`payload.rs:46`); `Error::TagKindMismatch { tag: [u8; 4], prefix: u8 }` (`error.rs:67`); `Error::ReservedPrefixViolation { got: u8 }` (`error.rs:76`); `RecordError::PreimagePlate` (`record.rs:81`); `UnknownReason::PreimagePlate` (`sysw/mod.rs:158`); `pack(vec![...], None, ITER) -> Result<_, SyswError>` as the neighbouring test uses it.
