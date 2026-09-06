# Hashlock H6 — Preimage Plates Implementation Plan (three repos)

**STATUS: DRAFT, awaiting R0 round 0.** Written by the plan author (opus) from
`design/SPEC_hashlock_H6_preimage_plates.md` (R0 GREEN at engrave `a67a3924`).
The build gate below was run BY THE AUTHOR at every task boundary in three
scratch trees; `## Build gate` records what was wired and, explicitly, **what
was specified and NOT wired** — Tasks 8b through 12 are the fork's `gui`
surface, and their blocks deliberately carry no `file=` header so the checker
cannot report a coverage it does not have. Author report:
`design/agent-reports/hashlock-H6-plan-author-report.md`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** put a hashlock preimage on steel — either as the ms1 kind-`0x03`
string `ms hashlock` prints, or as the phrase and its method in plain text — on
a plate layout of its own that says `NOT A SEED`, and give the operator the two
routes that reach it: a composition that derived the preimage here, and a
payload that carries one.

**What it reverses.** Ruling L7 scoped the device to the digest alone: *"It
never stores, shows, engraves or sources a preimage."* H6 lifts three of those
four verbs and leaves the fourth — sourcing a preimage plate back into a seed
flow — refused. Four shipped records become false and are rewritten by Task 13.

**Architecture.** Three repositories, in the order the Rust-primary rule forces:

1. **mnemonic-secret** owns the phrase RULE and the QR TEXT. The rule moves out
   of the `ms-cli` binary and into `ms-codec`, because `me sysw pack` must apply
   it byte for byte and `me` depends on the codec, not on the CLI. This is a
   published-crate change and it GATES everything downstream (Task 1).
2. **mnemonic-engrave** owns the two record classes, the `phrase:` wire form,
   the `hash`-id narrowing and `--pack-preimage` (Tasks 2-3). The corpus it
   generates is what the fork's lockstep test reads.
3. **the seedhammer fork** ports the codec half (Tasks 4-7) and builds the
   device half (Tasks 8-12).

**Tech Stack:** Rust (workspace `cargo nextest run --locked`; `cargo-nextest`
0.9.140), Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go/bin/go`, the
fork's `gui` touch harness, TinyGo via `nix develop -c tinygo build …` for size,
`cmd/emu` (GOOS=js) for the walk, `scripts/gui-shard-test.sh` (engrave) for the
whole `gui` package.

**Spec:** `design/SPEC_hashlock_H6_preimage_plates.md` (R0 GREEN at engrave
`a67a3924`: three lenses, one fold, one verification). Parents:
`design/SPEC_hashlock_H2_device.md`, `design/SPEC_hashlock_H5_device_polish.md`,
`SPEC_wallet_policy_composer.md`, `SPEC_ms_hashlock` (mnemonic-secret).

**Baselines (for `scripts/plan-staleness-check.sh`):** seedhammer fork main
`fb0dd04`; mnemonic-engrave `75f00b56`; mnemonic-secret `504ff46`. Every
`file:line` below was re-grepped at those revisions.

---

## Global Constraints

- **THE RUST-PRIMARY RULE ORDERS THE WHOLE STAGE, and Task 1 is a RELEASE.**
  `me-cli/Cargo.toml` depends on `ms-codec = "0.8"` from crates.io, not on a
  path. Task 2's `phrase:` parser calls `ms_codec::hashlock::validate_phrase`,
  which does not exist in 0.8 — so **Task 2 cannot build until Task 1 is
  released and `me-cli`'s dependency is bumped.** The author's gate stood a
  `[patch.crates-io]` in front of that so the rest of the stage could be built
  and run; the patch is a SCRATCH DEVICE and must not be committed. Task 1
  Step 6 is the release, and Task 2 Step 0 is the bump.
- **THREE BASELINE REDS, all pre-existing at `fb0dd04` / `75f00b56`, none
  introduced here, and every one of them MEASURED on the pristine checkouts:**
  1. `gofmt -l` reports `gui/transaction.go`, `gui/transaction_golden_test.go`
     and `gui/transaction_txrecord_test.go` on the pristine fork. H6 touches
     none of the three and may not leave a fourth.
  2. `go vet ./engrave/` **EXITS 1** on the pristine fork, printing
     `engrave/engrave_test.go:177:50: testing.ArtifactDir requires go1.26 or
     later (file is go1.25)` and the same at `:253`, because `go.mod` says
     `go 1.25.10` while the toolchain is 1.26.7 and the project's own floor is
     `go1.26`. **H6 grows the count from 2 to 6**: the two new golden tests call
     `t.ArtifactDir()` exactly as their four shipped siblings do, and matching
     the convention was preferred to inventing a second one for two call sites.
     The fix is a `go.mod` directive bump, filed as a follow-up rather than
     slipped into this stage — it is a toolchain change with a CI blast radius,
     and it is not H6's to make.
  3. `cargo nextest run --locked -p mnemonic-engrave` fails three
     `history_purge` tests on this box (`the_harness_records_history_at_all`,
     `editing_the_file_alone_is_the_trap_the_message_warns_about`,
     `the_emitted_zsh_recipe_actually_purges_the_entry`). Box-local; 618 of 621
     pass. **A fourth failure is a finding.**
- **`ConstantQR`'s raise falsifies TWO SHIPPED TESTS the spec does not name, and
  both are rewritten here rather than deleted** (author finding, §14 carries
  neither): `TestConstantQRLargeVersionsFailClosed`
  (`engrave/engrave_test.go:544-568`) asserts that dim 41 is REFUSED, and
  `TestPassphraseQRTooLong` (`backup/passphrase_test.go:775-791`) uses a
  200-character passphrase — 194+ bytes, dim 53 — as its "beyond ConstantQR's
  reach" case. Both properties survive; only the version each is asserted at
  moves, to v10 (dim 57). A third record, the comment on
  `TestPassphraseQRFitsSupportedVersion`'s boundary table (*"dim 41 — which is
  deliberately unsupported"*), is rewritten with it.
- **The fit gates are the arbiter of every copy change, and no character budget
  is asserted.** `assertModalBodyFits` (`gui/modal_fits_test.go:202`) renders the
  body and measures headroom against `modalBodyMargin = 80` (`:52`); headroom is
  a LINE budget, not a character budget.
- **EVERY DEVICE BODY IS ASCII, and it is a mechanism rather than house style.**
  `font/bitmap/bitmap.go:33` sets `indexLen = unicode.MaxASCII` and `glyphFor`
  rejects `int(r) >= indexLen`; the consequence is not a dropped character but a
  BLANK FRAME (5,004 ink pixels against a 6,000 floor) that `ExtractText` still
  reports as present. HOST lines are stderr, carry no panel budget, and are
  exempt — the shipped refusals already carry em dashes.
- **Secret-handling defects never gate** (operator ruling 2026-08-27). F-483
  (the typed phrase in an unwipeable Go string) is open and stays open.
- **Rust-primary (CLAUDE.md):** the fork's `codex32`, `sysw` and `hashlock`
  packages are DOWNSTREAM. Every normative decision in this stage lands in
  mnemonic-secret or mnemonic-engrave first, with vectors, and is ported after.
- **Fork commits** `git commit -s` (DCO), author Brian Goss, branch
  `hashlock-h6` off fork `main` `fb0dd04`; stage paths explicitly, never
  `git add -A`.
- **Flash only via `~/bin/sh/sh2-flash -y` at the operator's word**; never
  picotool by hand. **The SH2 has no camera**, so §12 item 8's QR scan is a
  phone and a test plate, never a device read-back.

---

## File Structure

| File | Change | Task | Responsibility |
| --- | --- | --- | --- |
| ms `crates/ms-codec/src/hashlock.rs` | Modify | 1 | `validate_phrase`, `PhraseRefusal`, `looks_like_ms1`, `HASHLOCK_PHRASE_MAX_CHARS`, `qr_text` |
| ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json` | Modify | 1 | seven `qr_text` rows; `format` bumped |
| ms `crates/ms-codec/tests/hashlock_qr_text.rs` | Create | 1 | §11.2's rows and their mutations |
| ms `crates/ms-cli/src/hashlock_phrase.rs` | Modify | 1 | `validate_phrase` DELEGATES; message rendering stays |
| ms `crates/ms-cli/src/argv_guard.rs` | Modify | 1 | `looks_like_ms1` DELEGATES |
| ms `crates/ms-codec/Cargo.toml`, `CHANGELOG` | Modify | 1 | the 0.9.0 release Task 2 depends on |
| me `crates/me-cli/Cargo.toml` | Modify | 2 | `ms-codec = "0.9"` |
| me `crates/me-cli/src/sysw/composer_records.rs` | Modify | 2 | `PHRASE_PREFIX`, `HashlockMethod`, `PhraseRecord`, `phrase_record`, `parse_phrase`, 21 `CASES` rows |
| me `crates/me-cli/src/sysw/record.rs` | Modify | 2 | `Class::Preimage`, `Class::Phrase`, `is_secret`, `is_bearer` |
| me `crates/me-cli/src/sysw/mod.rs` | Modify | 2,3 | the two classify arms; `Admission.pack_preimage`; `admit_check`'s second rule; `SyswError::PreimageNotAdmitted`; the shipped unit test |
| me `crates/me-cli/src/seal/record.rs` | Modify | 2 | `preimage_plate_admissible` (`preimage_plate` UNCHANGED) |
| me `crates/me-cli/testdata/record_class_vectors.json` | Regenerate | 2 | 47 → 68 rows |
| me `crates/me-cli/testdata/record_corpus_pre_s2.json` | Modify (one row) | 2 | `preimage-plate-0x03` `Unknown` → `Preimage` |
| me `crates/me-cli/tests/sysw_composer_records.rs` | Modify | 2 | `FIXTURE_SHA256` re-pinned |
| me `crates/me-cli/tests/record_corpus.rs` | Modify | 2 | the class map and the moved-row note |
| me `crates/me-cli/src/main.rs` | Modify | 2,3 | `class_name`'s two arms (2); `--pack-preimage`, §8.1.1, §8.1.2, the four warnings, the `phrase:` confirmation arm, `argv_secret_guard`'s new arm (3) |
| me `crates/me-cli/tests/sysw_pack_preimage.rs` | Create | 3 | §11.1's admission, sealing, id-partition and warning rows — nine tests |
| fork `engrave/engrave.go` | Modify | 4 | the v6-v9 alignment table, the `dim > 53` bound, four `constantTimeQRModules` arms, `engraveModule`'s `case 2` |
| fork `engrave/h6_qr_test.go` | Create | 4 | §11.3's six rows |
| fork `engrave/engrave_test.go` | Modify | 4 | the two falsified records |
| fork `engrave/testdata/h6-qr-v{6,7,8,9}-scale2.bin` | Create | 4 | §7.2 item 6's goldens |
| fork `codex32/msencode.go` | Modify | 5 | `EncodeMS1Preimage` |
| fork `codex32/mspayload.go` | Modify | 5 | `IsPreimagePlate` (`IsPreimage` UNCHANGED) |
| fork `codex32/msencode_preimage_test.go` | Create | 5 | §11.1's four device rows |
| fork `backup/hashlock.go` | Create | 6 | `Hashlock`, `HashlockForm`, the layout, `EngraveHashlock` |
| fork `backup/hashlock_test.go` | Create | 6 | §11.4's rows, plus the `qr_text` corpus lockstep |
| fork `backup/passphrase_test.go` | Modify | 6 | the falsified `TestPassphraseQRTooLong` |
| fork `backup/testdata/hashlock-*.bin` | Create | 6 | four plate goldens |
| fork `sysw/record.go` | Modify | 7 | `ClassPreimage`, `ClassPhrase`, `IsSecret` |
| fork `sysw/composer_records.go` | Modify | 7 | `PhrasePrefix`, `HashlockMethod`, `PhraseRecord`, `ParsePhraseRecord`, `PhraseRecordString` |
| fork `sysw/classify.go` | Modify | 7 | `isPreimagePlateRecord`, answered BEFORE `isStrictMs1` |
| fork `sysw/composer_records_test.go` | Modify | 7 | the `Phrase` class and the 68-row count |
| fork `sysw/testdata/record_class_vectors.json`, `.provenance.json` | Re-vendor | 7 | 68 rows, new sha, new pin |
| fork `hashlock/testdata/hashlock-v0.8.json`, `.provenance.json`, `hashlock/hashlock_test.go` | Re-vendor + re-pin | 7 | Task 1's seven `qr_text` rows; `corpusSHA256` moves |
| fork `gui/sysw_admit.go` | Modify | 7 | both classes at `progWalletPolicy` and no other row |
| fork `gui/composer_state.go` | Modify | 8a | `hashlockHeld`, `hashlockMaterial`, `hashlockProvenance`, the hold and the scrub |
| fork `gui/composer_flow.go` | Modify | 8a,9 | the scrub in the EXISTING defer; the engrave step's pick, census, cut order and abort arms |
| fork `gui/composer_hashlock_held_test.go` | Create | 8a | §2.2's rows |
| fork `gui/composer_hash.go` | Modify | 8b | the two new row bands, the `(in payload)` annotation, `taking` |
| fork `gui/composer_hashlock.go` | Modify | 8b | the derive-only sibling of `hashlockPhraseRoute` |
| fork `gui/composer_census.go` | Modify | 9 | `composerCensusLines`' third parameter and §8.3's block |
| fork `gui/composer_copy.go` | Modify | 9 | §8.3, §8.4a, §8.4b, §8.5, §10.1's third arm |
| fork `gui/composer_door.go` | Modify | 10 | the fourth route, its predicate, the lead's counts |
| fork `gui/composer_hashlock_plates.go` | Create | 10 | the Hashlock plates flow |
| fork `gui/freetext_flow.go`, `gui/passphrase_flow.go` | Modify | 11 | §9's warning at OK |
| fork `gui/sysw_session.go` | Modify | 11 | §8.8's Password-program notice |
| fork `gui/modal_fits_test.go` | Modify | 9,11 | one row per new DEVICE body |
| fork `cmd/emu/walk_hashlock_phrase.js` | Modify | 12 | §11.7's H6 arm |
| engrave `design/SPEC_hashlock_H2_device.md`, `SPEC_wallet_policy_composer.md`, `design/FOLLOWUPS.md` | Modify | 13 | the four falsified records; F-132's plate half |
| toolkit `docs/manual/…` | Modify | 13 | `--pack-preimage` and the plate forms |

**Parallel groups, by DISJOINT file lists.** The controller assigns implementers
A/B/C/D as it did for H5.

| group | tasks | shares files with |
| --- | --- | --- |
| **A** | 1 | nothing (its own repo) |
| **B** | 2 → 3 | each other (`sysw/mod.rs`); blocked on A |
| **C** | 4, 5 in parallel, then 6 | 6 needs 4's `ConstantQR` raise at runtime, not at compile time; 4 and 5 are disjoint |
| **D** | 7 | needs B's corpus and C's `IsPreimagePlate`; touches `gui/sysw_admit.go`, which no other task touches |
| **E** | 8a → 8b → 9 | ALL THREE TOUCH `gui/composer_flow.go` and `gui/composer_hash.go`. **One implementer, sequentially.** |
| **F** | 10 | `gui/composer_door.go` + a new file; disjoint from E once E has landed `hashlockHeld` |
| **G** | 11 | `gui/freetext_flow.go`, `gui/passphrase_flow.go`, `gui/sysw_session.go` — disjoint from every other group |
| **H** | 12, then 13 | 12 after E and F; 13 last, and it is records only |

**Gate coverage — what the checker script does and does not prove.** Every
fenced block below that carries file content opens with

    ```go   file=fork/engrave/engrave.go            mode=fragment
    ```rust file=ms/crates/ms-codec/src/hashlock.rs mode=fragment

`mode=whole` means the block IS the file; `mode=fragment` means it must appear
VERBATIM inside it, indentation included. **The path is rooted at a REPO name** —
`fork`, `ms` or `me` — because H6 spans three trees;
`scripts/h6-plan-blocks-vs-tree.sh` builds a symlink farm and hands it to the
shared engine, which is otherwise unchanged. A block with NO header is a
command, an illustration, or a Task 8b-12 body the author specified without
wiring. The script prints its own blind spots on every run; read its tail.

---

### Task 1: The phrase rule and the QR text move into `ms-codec` (spec §8.6, §11.2)

**Repo:** mnemonic-secret, at `504ff46`.

**Files:**
- Modify: `crates/ms-codec/src/hashlock.rs`, `crates/ms-codec/tests/vectors/hashlock-v0.8.json`, `crates/ms-cli/src/hashlock_phrase.rs`, `crates/ms-cli/src/argv_guard.rs`, `crates/ms-codec/Cargo.toml`
- Create: `crates/ms-codec/tests/hashlock_qr_text.rs`

**Interfaces:**
- Produces: `ms_codec::hashlock::{validate_phrase, PhraseRefusal, looks_like_ms1, qr_text, HASHLOCK_PHRASE_MAX_CHARS}`.
- Consumes: `HASHLOCK_SALT`, `HASHLOCK_ITERATIONS`, `HASHLOCK_DKLEN` (`crates/ms-codec/src/hashlock.rs:27,30,32`) — read, never re-spelled.
- Unchanged: `ms-cli`'s `PhraseRefusal` and its `message()`; every refusal sentence an operator sees.

**Why this task is FIRST, and why it is a RELEASE.** The spec says a `phrase:`
record's phrase must pass *"`ms-cli`'s `validate_phrase` … byte for byte"*.
Measured: `validate_phrase` and `looks_like_ms1` are `ms-cli`'s, and
`me-cli/Cargo.toml:53` depends on `ms-codec = "0.8"` and on nothing of `ms-cli`.
So the spec as written asks `me` to call a function it cannot reach, and the
only two ways to satisfy it are a THIRD copy of the rule or a move into the
codec. **A third copy is the defect the rule exists to prevent** — its whole
point is that no two readers of a phrase disagree about what one is — so the
rule moves, `ms-cli` delegates, and there is still exactly one implementation.
`qr_text` lands beside it for the same reason: the fork's plate builder needs
something to assert against that is not a literal it transcribed itself.

- [ ] **Step 1: `validate_phrase`, `looks_like_ms1` and `qr_text` in the codec.**
Appended to `crates/ms-codec/src/hashlock.rs`, after `digest`:

```rust file=ms/crates/ms-codec/src/hashlock.rs mode=fragment
/// The phrase cap. Its own constant on each side, lockstep-pinned; NOT the
/// device's plate-legibility `passphrase.MaxLen`.
pub const HASHLOCK_PHRASE_MAX_CHARS: usize = 100;

/// The shortest string `looks_like_ms1` will call ms1-shaped.
const MIN_MS1_LEN: usize = 48;

const BECH32_CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Why a phrase was refused. One variant per rule, in the order the rule
/// checks them; the CALLER renders the sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseRefusal {
    /// No bytes at all.
    Empty,
    /// A byte outside `0x20..=0x7E`, with the byte and its position.
    NotPrintableAscii {
        /// The offending byte.
        byte: u8,
        /// Its zero-based position.
        at: usize,
    },
    /// An ms1 string — a preimage plate, not a phrase.
    Ms1Shaped,
    /// Over `HASHLOCK_PHRASE_MAX_CHARS`.
    TooLong {
        /// The length that was measured.
        chars: usize,
    },
    /// Exactly 64 hex characters — a preimage in hex, not a phrase.
    Hex64,
}

/// `looks_like_ms1` over the NORMALISED token: trimmed, ASCII-lowercased,
/// display separators (whitespace, `-`, `,`) stripped, then at least 48
/// characters, an `ms1` prefix and only bech32 characters.
///
/// NO CHECKSUM, deliberately. A GROUPED plate is what `ms hashlock`'s
/// engraving card prints and therefore what an operator retypes, and a
/// checksum test would answer false for it — so the guard would miss the one
/// spelling it exists to catch.
pub fn looks_like_ms1(raw: &str) -> bool {
    let t: String = raw
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != ',')
        .collect();
    t.len() >= MIN_MS1_LEN
        && t.starts_with("ms1")
        && t[3..].chars().all(|c| BECH32_CHARSET.contains(c))
}

/// The rule. ORDER MATTERS and is the spec's: empty, printable ASCII,
/// ms1-shape (BEFORE the cap, so a grouped plate string gets the `--in`
/// remedy and not "too long"), the cap, 64-hex.
///
/// It changes nothing: no trim, no case fold, no normalisation. The shape test
/// works on a copy.
pub fn validate_phrase(bytes: &[u8]) -> core::result::Result<(), PhraseRefusal> {
    if bytes.is_empty() {
        return Err(PhraseRefusal::Empty);
    }
    if let Some((at, &byte)) = bytes
        .iter()
        .enumerate()
        .find(|(_, b)| !(0x20..=0x7e).contains(*b))
    {
        return Err(PhraseRefusal::NotPrintableAscii { byte, at });
    }
    // All bytes are printable ASCII now, so this is a &str.
    let s = core::str::from_utf8(bytes).expect("printable ASCII is UTF-8");
    if looks_like_ms1(s) {
        return Err(PhraseRefusal::Ms1Shaped);
    }
    if s.len() > HASHLOCK_PHRASE_MAX_CHARS {
        return Err(PhraseRefusal::TooLong { chars: s.len() });
    }
    if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(PhraseRefusal::Hex64);
    }
    Ok(())
}
```

**One deliberate difference from the text it replaces, and it is a
convergence.** `ms-cli`'s 64-hex test was `s.len() == 64 && hex::decode(s).is_ok()`,
which pulled the `hex` crate into a codec that does not otherwise need it;
`b.is_ascii_hexdigit()` is the same predicate over the same 64-byte window (the
`hex` crate accepts exactly `[0-9a-fA-F]` pairs and the length is already
pinned at 64). The device's `hashlock.isHex` already spells it this way.

- [ ] **Step 2: the QR text, in the same file.** It is `qr_text` and not a
`format!` at a call site because §8.6 pins the bytes and §11.2's corpus rows
have to compare against something:

```rust file=ms/crates/ms-codec/src/hashlock.rs mode=fragment
/// The QR text a hashlock PHRASE plate carries (SPEC_hashlock_H6 §8.6), byte
/// for byte: three labelled lines, LF-separated, NO trailing newline, the
/// phrase LAST.
///
/// The phrase is last so a reader knows where it ends: it may itself contain
/// `:` and spaces, and everything after `phrase: ` on the final line is the
/// phrase, verbatim, with real `0x20` spaces.
///
/// The method line names the ALGORITHM in full — not the `--method` selector —
/// so a reader with the plate and no tool can reproduce the derivation. Its
/// parameters are read from `HASHLOCK_SALT`, `HASHLOCK_ITERATIONS` and
/// `HASHLOCK_DKLEN` and never from a literal, so a parameter change cannot
/// leave the plate lying.
///
/// `hashlock v1` is the VERSION TAG of this TEXT, not of the derivation. A
/// future parameter set gets `hashlock v2`.
pub fn qr_text(hardened: bool, phrase: &str) -> String {
    let method = if hardened {
        format!(
            "method: pbkdf2-hmac-sha256 iterations={HASHLOCK_ITERATIONS} salt={} dklen={HASHLOCK_DKLEN}",
            core::str::from_utf8(HASHLOCK_SALT).expect("the salt is ASCII"),
        )
    } else {
        "method: sha256".to_string()
    };
    format!("hashlock v1\n{method}\nphrase: {phrase}")
}
```

- [ ] **Step 3: `ms-cli` delegates, and keeps its message rendering.**

```rust file=ms/crates/ms-cli/src/hashlock_phrase.rs mode=fragment
pub fn validate_phrase(bytes: &[u8]) -> std::result::Result<(), PhraseRefusal> {
    ms_codec::hashlock::validate_phrase(bytes).map_err(|r| match r {
        ms_codec::hashlock::PhraseRefusal::Empty => PhraseRefusal::Empty,
        ms_codec::hashlock::PhraseRefusal::NotPrintableAscii { byte, at } => {
            PhraseRefusal::NotPrintableAscii { byte, at }
        }
        ms_codec::hashlock::PhraseRefusal::Ms1Shaped => PhraseRefusal::Ms1Shaped,
        ms_codec::hashlock::PhraseRefusal::TooLong { chars } => PhraseRefusal::TooLong { chars },
        ms_codec::hashlock::PhraseRefusal::Hex64 => PhraseRefusal::Hex64,
    })
}
```

```rust file=ms/crates/ms-cli/src/argv_guard.rs mode=fragment
pub(crate) fn looks_like_ms1(raw: &str) -> bool {
    // DELEGATED to ms_codec::hashlock::looks_like_ms1 since H6, so the argv
    // guard, the phrase rule and `me sysw pack`'s `phrase:` record all read one
    // predicate. `is_ms1_shaped` below is kept as the crate-local spelling the
    // unit tests drive and is asserted equal to the codec's.
    ms_codec::hashlock::looks_like_ms1(raw)
}
```

- [ ] **Step 4: the seven corpus rows.** Added to
`crates/ms-codec/tests/vectors/hashlock-v0.8.json` as a new top-level
`qr_text` array, with `format` bumped to
`"ms hashlock corpus v0.9 (SPEC_ms_hashlock §8; qr_text rows added by SPEC_hashlock_H6 §8.6/§11.2)"`.
Each row carries `name`, `method`, `phrase`, `qr_text`, `bytes`, `note`.
MEASURED byte counts, which are what §7.1's version table is built on:

| row | method | bytes | ECC-L |
| --- | --- | --- | --- |
| `anchor-hardened` | hardened | 122 | v6, 41 modules |
| `anchor-sha256` | sha256 | 63 | v4, 33 |
| `max-phrase-hardened` | hardened | **194** | **v9, 53** |
| `max-phrase-sha256` | sha256 | 135 | v7, 45 |
| `phrase-with-colon` | hardened | 119 | v6 |
| `phrase-trailing-space` | hardened | 102 | v5 |
| `phrase-with-comma` | hardened | 109 | v6 |

- [ ] **Step 5: the test.** Create `crates/ms-codec/tests/hashlock_qr_text.rs`
with three tests — `qr_text_matches_every_corpus_row`,
`parameters_come_from_the_constants`, `the_worst_case_is_194_bytes`. The whole
file is in the gated tree; the load-bearing assertions are:

```rust file=ms/crates/ms-codec/tests/hashlock_qr_text.rs mode=fragment
        let got = qr_text(hardened, &row.phrase);
        assert_eq!(got, row.qr_text, "row {}", row.name);
        assert_eq!(got.len(), row.bytes, "row {}: byte count", row.name);
        assert!(
            !got.ends_with('\n'),
            "row {}: the text has a trailing newline",
            row.name
        );
        assert_eq!(
            got.lines().count(),
            3,
            "row {}: the text is three LF-separated lines",
            row.name
        );
        let last = got.lines().next_back().unwrap();
        assert_eq!(
            last.strip_prefix("phrase: ").unwrap(),
            row.phrase,
            "row {}: the phrase is the LAST line, verbatim",
            row.name
        );
```

```rust file=ms/crates/ms-codec/tests/hashlock_qr_text.rs mode=fragment
    assert_eq!(
        method.len(),
        73,
        "the hardened method line is 73 characters; H6 §6.5 pins the plate's \
         worst case on it and a 79th character puts the phrase plate over budget"
    );
```

**MUTATIONS, all four executed:** emit a trailing newline → every row fails;
put `phrase:` before `method:` → every row fails; change `HASHLOCK_ITERATIONS`
without the corpus → `qr_text_matches_every_corpus_row` fails; drop a parameter
from the method line → the 73-character assertion fails.

- [ ] **Step 6: RELEASE `ms-codec` 0.9.0.** Bump `crates/ms-codec/Cargo.toml`,
write the CHANGELOG entry (the phrase rule moved in, `qr_text` added, no
behaviour change to any existing verb), re-vendor, publish. **Task 2 is blocked
until this lands**, and the fork's `hashlock/testdata/hashlock-v0.8.provenance.json`
gains the new commit and sha at Task 7.

**Boundary gate (RUN):** `cargo nextest run --locked` in the ms workspace —
**562 tests run: 562 passed, 11 skipped.**

---

### Task 2: The `phrase:` record and the two classes (spec §3.1, §3.4, §4.2, §4.3)

**Repo:** mnemonic-engrave, at `75f00b56`. **Blocked on Task 1's release.**

**Files:**
- Modify: `crates/me-cli/Cargo.toml`, `crates/me-cli/src/sysw/composer_records.rs`, `crates/me-cli/src/sysw/record.rs`, `crates/me-cli/src/sysw/mod.rs`, `crates/me-cli/src/seal/record.rs`, `crates/me-cli/src/main.rs` (one match arm), `crates/me-cli/tests/sysw_composer_records.rs`, `crates/me-cli/tests/record_corpus.rs`, `crates/me-cli/testdata/record_corpus_pre_s2.json`
- Regenerate: `crates/me-cli/testdata/record_class_vectors.json`

**Interfaces:**
- Produces: `PHRASE_PREFIX`, `HashlockMethod`, `PhraseRecord`, `phrase_record(method, phrase)`, `ComposerRecord::Phrase`, `ComposerRecordError::Phrase`; `Class::Preimage`, `Class::Phrase`; `seal::record::preimage_plate_admissible`.
- Consumes: `ms_codec::hashlock::{validate_phrase, preimage_hardened, preimage_sha256, digest}`.
- Unchanged, and each is normative: `seal::record::preimage_plate` (the DIAGNOSTIC predicate behind the refusal message), `decide_sealing` (`crates/me-cli/src/main.rs:2353-2410`), `unknown_reason`'s `entr` arm.

- [ ] **Step 0: bump the dependency.** `crates/me-cli/Cargo.toml:53`,
`ms-codec = "0.8"` → `"0.9"`. Nothing else in this task compiles without it.

- [ ] **Step 1: the wire form.** `PHRASE_PREFIX` beside the other three, and
the method selector:
```rust file=me/crates/me-cli/src/sysw/composer_records.rs mode=fragment
/// `phrase:<hex of "<method>,<phrase>">` — a hashlock PHRASE and its method
/// selector (SPEC_hashlock_H6 §3.1). The `now:` idiom exactly: hex of a UTF-8
/// text, one comma, cut on the FIRST comma so the phrase may contain commas.
///
/// RESERVED like the other three, so a `phrase:` record whose body fails any
/// rule is `Class::Unknown` and refused with its own line rather than treated
/// as free text.
pub const PHRASE_PREFIX: &str = "phrase:";
```

```rust file=me/crates/me-cli/src/sysw/composer_records.rs mode=fragment
/// The method SELECTOR a `phrase:` record carries.
///
/// The WIRE carries a selector and the PLATE carries the method DEFINITION
/// (H6 §8.6), and the difference is deliberate: a wire record is read by a tool
/// that already knows the parameter set, while a plate is read by a person who
/// may have neither the tool nor this firmware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashlockMethod {
    /// `hardened` — PBKDF2-HMAC-SHA256, the `ms hashlock` default.
    Hardened,
    /// `sha256` — one SHA-256 of the phrase bytes.
    Sha256,
}

impl HashlockMethod {
    /// The selector spelling, which is `ms hashlock --method`'s and the
    /// device's `hashlockMethod.String()`.
    pub fn as_str(self) -> &'static str {
        match self {
            HashlockMethod::Hardened => "hardened",
            HashlockMethod::Sha256 => "sha256",
        }
    }

    /// X from the phrase under this method.
    pub fn preimage(self, phrase: &[u8]) -> zeroize::Zeroizing<[u8; 32]> {
        match self {
            HashlockMethod::Hardened => ms_codec::hashlock::preimage_hardened(phrase),
            HashlockMethod::Sha256 => ms_codec::hashlock::preimage_sha256(phrase),
        }
    }
}

/// A parsed `phrase:` record. `phrase` is SECRET.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhraseRecord {
    /// The method selector the record names.
    pub method: HashlockMethod,
    /// The phrase, verbatim, everything after the FIRST comma.
    pub phrase: String,
}
```

- [ ] **Step 2: the parser.** CUT ON THE FIRST COMMA, and the phrase rule is
called rather than re-spelled:
```rust file=me/crates/me-cli/src/sysw/composer_records.rs mode=fragment
/// `phrase:` — hex of `<method>,<phrase>`, CUT ON THE FIRST COMMA.
///
/// Never the last: a phrase may contain commas, and cutting on the last would
/// take everything after the final comma as the phrase and derive a DIFFERENT
/// preimage from the one the packer meant.
///
/// The phrase itself must pass `ms_codec::hashlock::validate_phrase` — the one
/// implementation of SPEC_ms_hashlock §4.3, which `ms hashlock` and the
/// device's `hashlock.ValidatePhrase` also apply — so the record parser and
/// the keyboard cannot disagree about what a phrase is.
fn parse_phrase(body: &str) -> Result<ComposerRecord, ComposerRecordError> {
    use ComposerRecordError::Phrase as P;
    let bytes = unhex_lower(body).ok_or(P)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| P)?;
    let (method_text, phrase) = text.split_once(',').ok_or(P)?;
    let method = match method_text {
        "hardened" => HashlockMethod::Hardened,
        "sha256" => HashlockMethod::Sha256,
        _ => return Err(P),
    };
    ms_codec::hashlock::validate_phrase(phrase.as_bytes()).map_err(|_| P)?;
    Ok(ComposerRecord::Phrase(PhraseRecord {
        method,
        phrase: phrase.to_owned(),
    }))
}
```

and its builder, used by the tests and by nothing in production this stage —
**§13 records that a `phrase:` record has no CLI producer, which is what §3.3's
orphan warning exists to cover:**
```rust file=me/crates/me-cli/src/sysw/composer_records.rs mode=fragment
/// `phrase:` + hex of `<method>,<phrase>`. The text is NOT validated here;
/// `parse` is the gate, so a test can build a malformed record on purpose.
/// The returned string is SECRET.
pub fn phrase_record(method: HashlockMethod, phrase: &str) -> String {
    format!(
        "{PHRASE_PREFIX}{}",
        hex_lower(format!("{},{phrase}", method.as_str()).as_bytes())
    )
}
```

- [ ] **Step 3: the two classes.** `Class::Preimage` and `Class::Phrase`, both
SECRET and both BEARER:
```rust file=me/crates/me-cli/src/sysw/record.rs mode=fragment
    /// A hashlock PREIMAGE plate: the `ms1` kind-`0x03` string under the id
    /// `hash` (SPEC_ms_hashlock §1 rule 2). SECRET and BEARER.
    Preimage,
    /// `phrase:` — a hashlock phrase and its method selector (H6 §3.1).
    /// SECRET and BEARER.
    Phrase,
```
```rust file=me/crates/me-cli/src/sysw/record.rs mode=fragment
    /// H6: a hashlock preimage and a hashlock phrase are BOTH — key material
    /// AND, for a key-less hashlock path, everything spending needs. They are
    /// secret and bearer, and the argv gate follows from either.
    pub fn is_bearer(self) -> bool {
        matches!(self, Class::Mt | Class::Tx | Class::Preimage | Class::Phrase)
    }
```

- [ ] **Step 4: `preimage_plate_admissible`, and `preimage_plate` UNCHANGED.**

**TWO NARROWINGS, and the second one is a defect the SHIPPED test caught.** The
spec's §4.3 gives one — the id `hash`. The author's first implementation stopped
there and `cargo nextest` reddened
`sysw::tests::a_preimage_plate_is_named_not_misdiagnosed`, whose R0-r0-tests-I-3
row is *a kind-0x03 single under the id `hash` whose X is 16 bytes* (50
characters, the codec's `PreimageLengthMismatch`). Admitting it would put a
string `DecodeMS1Preimage` refuses into a flow that engraves. The device's
`codex32.IsPreimage` already required `len(d) == 33 && d[0] == 0x03`, so **Go was
right and the Rust needed the second conjunct** — and the third, case
sensitivity, came from the same test's UPPERCASE row.
```rust file=me/crates/me-cli/src/seal/record.rs mode=fragment
/// `preimage_plate` PLUS the id `hash` — the ADMISSION predicate (H6 §4.3,
/// ruling A7), and the Rust half of the device's `codex32.IsPreimagePlate`.
///
/// `preimage_plate` above stays the DIAGNOSTIC predicate behind the refusal
/// message, so a mistagged plate is still NAMED a preimage plate when it is
/// refused. This narrower one decides what `--pack-preimage` lets INTO a
/// payload the device will engrave from, and the id is the whole difference:
/// H0 could afford the wide kind-byte rule because a false positive was a
/// REFUSAL, while here a false positive routes a string into a flow that cuts
/// it onto steel under a band reading NOT A SEED. A plain BIP-93 33-byte
/// secret beginning `0x03` — roughly 1 in 256 of them — must not arrive.
pub fn preimage_plate_admissible(s: &str) -> bool {
    let s = s.trim();
    if !preimage_plate(s) {
        return false;
    }
    // BIP-93 layout: `ms1` + threshold char + 4-char id + share index.
    //
    // CASE-SENSITIVE, and that is the device's answer rather than a choice made
    // here: `codex32.IsPreimagePlate` reads the id out of `String.Split()` and
    // compares it to the literal `"hash"`, so the UPPERCASE spelling of a plate
    // -- the QR-alphanumeric form, which `preimage_plate` DOES name as a plate --
    // is refused on both sides. Section 5.3 hashes a record in its canonical
    // lowercase form, so an uppercase one is not the record it looks like.
    if s.as_bytes().get(4..8) != Some(b"hash".as_slice()) {
        return false;
    }
    // AND the payload must be a WELL-FORMED preimage: exactly 33 bytes
    // beginning 0x03. `preimage_plate` deliberately answers `true` for a
    // kind-0x03 single whose X is the wrong length (the codec's
    // `PreimageLengthMismatch`) so the DIAGNOSTIC names it, and the shipped
    // test `a_preimage_plate_is_named_not_misdiagnosed` carries such a row
    // under the id `hash`. Admitting it would put a 50-character string that
    // `DecodeMS1Preimage` refuses into a flow that engraves, so the ADMISSION
    // predicate asks the narrower question -- and this is the same shape the
    // device's `codex32.IsPreimage` already tests (`len(d) == 33 && d[0] ==
    // 0x03`), so the two sides agree by construction rather than by review.
    match ms_codec::codex32::Codex32String::from_string(s.to_string()) {
        Ok(c) => {
            let d = c.parts().data();
            d.len() == 33 && d[0] == 0x03
        }
        Err(_) => false,
    }
}
```

- [ ] **Step 5: the two classify arms, and classification is UNCONDITIONAL.**
```rust file=me/crates/me-cli/src/sysw/mod.rs mode=fragment
    // H6 §3.2 item 1: CLASSIFICATION IS UNCONDITIONAL. A preimage plate is a
    // preimage plate whatever the admission, exactly as a `key:` record is a
    // key record — the flag is not a parameter of what a record IS. That is
    // what lets `decide_sealing` stay byte-unchanged on the strict classifier
    // and still seal a payload holding one, and what keeps ONE truth per
    // corpus row for the device's lockstep test.
    //
    // The predicate is `preimage_plate_admissible`, not `preimage_plate`: the
    // wider one stays the DIAGNOSTIC behind the refusal message, so a kind-0x03
    // single under an id outside {entr, hash} is still NAMED a preimage plate
    // when it is refused and is still not admitted.
    if crate::seal::record::preimage_plate_admissible(record) {
        return Class::Preimage;
    }
```

The `phrase:` arm rides the existing `composer_records::parse` match:

```rust file=me/crates/me-cli/src/sysw/mod.rs mode=fragment
            Ok(composer_records::ComposerRecord::Phrase(_)) => Class::Phrase,
```

- [ ] **Step 6: 21 `CASES` rows** appended to
`crates/me-cli/src/sysw/composer_records.rs:306`'s table, then
`cargo test --locked -p mnemonic-engrave --test sysw_composer_records regenerate -- --ignored --nocapture`,
then the new sha into `FIXTURE_SHA256`. RUN:

```
wrote 68 rows to .../crates/me-cli/testdata/record_class_vectors.json
sha256 3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf
```

The rows, and each is a §11.1 bullet made executable. The header is normative
about which one is the funds-relevant row:

```rust file=me/crates/me-cli/src/sysw/composer_records.rs mode=fragment
    // ---- H6 §3.1: the `phrase:` record, one row per rule (SPEC_hashlock_H6 §11.1).
    // `phrase-space-after-the-comma` is ADMITTED and is the hand-build error §8.2.3's
    // orphan warning exists to catch: a leading 0x20 is printable ASCII, so the record
    // is valid and derives a DIFFERENT preimage from the same words without it.
```

| name | class | pins |
| --- | --- | --- |
| `phrase-hardened`, `phrase-sha256` | Phrase | both selectors |
| `phrase-containing-a-comma` | Phrase | **cut on the FIRST comma** |
| `phrase-space-after-the-comma` | Phrase | the hand-build error §8.2.3 covers |
| `phrase-containing-a-colon` | Phrase | `:` is an ordinary phrase byte |
| `phrase-one-character`, `phrase-100-characters` | Phrase | the cap's low and high ends |
| `phrase-101-characters`, `phrase-empty` | Unknown | the cap and the empty rule |
| `phrase-64-hex` | Unknown | a pasted preimage is not a phrase |
| `phrase-ms1-shaped`, `phrase-ms1-shaped-grouped` | Unknown | the shape test, INCLUDING the grouped spelling a checksum test would miss |
| `phrase-unknown-method`, `phrase-uppercase-method` | Unknown | the selector is exactly `hardened` or `sha256` |
| `phrase-no-comma` | Unknown | one comma is required |
| `phrase-body-not-hex`, `-uppercase-hex`, `-odd-length`, `-not-utf8`, `-empty` | Unknown | the reserved-prefix hex rule |
| `phrase-non-printable-tab` | Unknown | printable ASCII only |

**MUTATION: cut on the LAST comma** → `phrase-containing-a-comma` classifies
`Phrase` but `ComposerRecord::Phrase.phrase` is `" three"` instead of
`"one, two, three"`, and `sysw_composer_records`'s value assertions red.
**MUTATION: accept an unknown method** → `phrase-unknown-method` and
`phrase-uppercase-method` classify `Phrase`.
**MUTATION: gate the classifier on admission** → the corpus row says `Unknown`
while the device's `Classify` says `ClassPhrase`, and Task 7's vendored lockstep
reds. **This is the row that justifies §3.2's whole design.**

- [ ] **Step 7: the one row H6 moves in `record_corpus_pre_s2.json`**, and its
note. `codex32_seam/preimage-plate-0x03` goes `Unknown` → `Preimage`; its
sibling `codex32_seam/preimage-shape-entr-id` STAYS `Unknown`, and that pair is
§4.3's id narrowing stated as data.

```rust file=me/crates/me-cli/tests/record_corpus.rs mode=fragment
/// **H6 MOVED EXACTLY ONE ROW, deliberately, and it is recorded here rather
/// than absorbed.** `codex32_seam/preimage-plate-0x03` was `Unknown` under H0,
/// whose whole design was that a hashlock preimage plate is INERT on this
/// device -- no class of its own -- and it is `Preimage` from H6, which gives
/// the shape a class so it can be admitted at one program and cut onto a plate
/// of its own. Its sibling `codex32_seam/preimage-shape-entr-id` STAYS
/// `Unknown`, and that pair is the id narrowing of H6 §4.3 stated as data: the
/// kind byte alone is not enough to reach the flow that engraves.
```

**Boundary gate (RUN):** `cargo nextest run --locked -p mnemonic-engrave
--no-fail-fast` — **621 tests run: 618 passed, 3 failed** (the box-local
`history_purge` trio named in Global Constraints), 2 skipped.

---

### Task 3: `--pack-preimage`, the refusals and the four warnings (spec §3.2, §3.3, §8.1, §8.2)

**Repo:** mnemonic-engrave. **Blocked on Task 2** (shares `sysw/mod.rs` and
`main.rs`).

**Files:**
- Modify: `crates/me-cli/src/sysw/mod.rs` (`Admission`, `admit_check`, `SyswError`), `crates/me-cli/src/main.rs`
- Create: `crates/me-cli/tests/sysw_pack_preimage.rs`

**Interfaces:**
- Produces: `Admission.pack_preimage`; `SyswError::PreimageNotAdmitted(usize, Class)`; `report_preimage_admission`, `report_sealed_preimage`, `preimage_digest_of`.
- **`decide_sealing` is BYTE-UNCHANGED**, and that is the point of §3.2: classification is unconditional, so the strict classifier it already calls sees both classes, `is_secret()` is true for each, and the payload seals by default with no second call site to keep in step. The declined alternative — threading `admission` into `decide_sealing` — adds exactly the second site the shipped `--expect` comment at `main.rs:1488-1494` records as *"a false refusal carrying a false message, on the funds path, inside the feature added to prevent exactly that."*

- [ ] **Step 1: the `Admission` field.**
```rust file=me/crates/me-cli/src/sysw/mod.rs mode=fragment
    /// Admit a hashlock PREIMAGE into this payload: an `ms1` kind-`0x03` plate
    /// string, or a `phrase:` record.
    ///
    /// It gates ADMISSION, never CLASSIFICATION (H6 §3.2). Both are BEARER
    /// material — whoever holds the preimage can spend any key-less hashlock
    /// path it unlocks — so admission is explicit, exactly as `--seal-secret`
    /// makes encrypting seed material explicit. It is NOT `--seal-secret` and
    /// the two do not substitute.
    pub pack_preimage: bool,
```

- [ ] **Step 2: `admit_check`'s SECOND refusal rule.** Beside the `Unknown`
one, not inside it:
```rust file=me/crates/me-cli/src/sysw/mod.rs mode=fragment
pub fn admit_check(records: &[String], adm: Admission) -> Result<(), SyswError> {
    for (i, r) in records.iter().enumerate() {
        let class = classify_with(r, adm);
        if matches!(class, record::Class::Unknown) {
            return Err(SyswError::Unclassifiable(i, unknown_reason(r)));
        }
        // H6 §3.2 item 2. A SECOND refusal rule, beside the `Unknown` one and
        // deliberately NOT a `Class::Unknown` / `UnknownReason`: the record is
        // not unclassifiable — it is perfectly well understood and simply not
        // asked for, and a reason enum that said otherwise would make the
        // refusal message lie about what `me` found.
        if !adm.pack_preimage
            && matches!(class, record::Class::Preimage | record::Class::Phrase)
        {
            return Err(SyswError::PreimageNotAdmitted(i, class));
        }
    }
    Ok(())
}
```

and its error variant:
```rust file=me/crates/me-cli/src/sysw/mod.rs mode=fragment
    /// A record that classifies `Preimage` or `Phrase` in a payload that did
    /// not pass `--pack-preimage` (H6 §3.2 item 2, §8.1.1).
    ///
    /// NOT an `Unclassifiable`: the record is perfectly well understood. The
    /// class rides along so the message can name which carrier it was.
    PreimageNotAdmitted(usize, record::Class),
```

- [ ] **Step 3: the flag.** `crates/me-cli/src/main.rs`, in the `pack`
subcommand's argument block, immediately after `allow_unsigned_inputs`:
```rust file=me/crates/me-cli/src/main.rs mode=fragment
        /// Admit a hashlock PREIMAGE into this payload: an ms1 kind-0x03 plate
        /// string, or a `phrase:` record carrying a hashlock phrase and its
        /// method.
        ///
        /// Both are BEARER material -- whoever holds the preimage can spend any
        /// key-less hashlock path it unlocks -- so admission is explicit,
        /// exactly as `--seal-secret` makes encrypting seed material explicit.
        /// It is NOT `--seal-secret` and the two do not substitute.
        ///
        /// It gates ADMISSION, never CLASSIFICATION: `me` calls a preimage a
        /// preimage either way, which is why the payload still SEALS by default
        /// and why the device's vendored class corpus carries one answer per
        /// row.
        #[arg(long)]
        pack_preimage: bool,
```

- [ ] **Step 4: §8.1.1, the refusal.** Its final sentence is CONDITIONAL on the
carrier being one `--pack-preimage` can actually admit — for any other id the
advice is affirmatively false and the operator would see three refusals, none of
which says the one thing that is true.
```rust file=me/crates/me-cli/src/main.rs mode=fragment
        // H6 §8.1.1. A well-formed preimage plate (id `hash`) or a `phrase:`
        // record in a payload that did not ask for one.
        //
        // THE FINAL SENTENCE IS EMITTED ONLY FOR THE PLATE-SHAPED CARRIERS THIS
        // FLAG ACTUALLY ADMITS. For any other id the advice would be
        // affirmatively false -- `--pack-preimage` refuses a wrong-id record on
        // the next run (§4.3), and the operator is then sent to `ms hashlock`,
        // which refuses it a third time in a message written for a different
        // audience. Three refusals, none of them saying the one thing that is
        // true and actionable; a wrong-id record gets §8.1.2 straight away,
        // from the no-flag path too, and sees ONE refusal.
        E::PreimageNotAdmitted(i, class) => {
            use mnemonic_engrave::sysw::record::Class as C;
            let what = match class {
                C::Phrase => "a hashlock PHRASE record (phrase:)",
                _ => "a hashlock PREIMAGE plate (kind 0x03)",
            };
            format!(
                "record {i} (records count from 0) is {what}, not a seed record; this \
                 payload did not ask for one. A preimage backs a hashlock spend path, not \
                 a wallet — keep it with the policy it unlocks, and do not re-encode it as \
                 entropy. Re-run with --pack-preimage if that is what you intend."
            )
        }
```

- [ ] **Step 5: §8.1.2, the EXISTING body rewritten.** Not a second arm: `entr`
never reaches `U::PreimagePlate` (the mismatch is diagnosed first, at
`sysw/mod.rs:207`), and id `hash` is now a CLASS, so the only case this arm
covers is an id outside `{entr, hash}` — and the body now says so. §9's own rule
(*"two near-identical bodies is how one of them goes stale"*) is the reason.
```rust file=me/crates/me-cli/src/main.rs mode=fragment
                // H6 §8.1.2 -- THE EXISTING BODY REWRITTEN, not a second arm.
                // Measured: `entr` never reaches here (id_kind_mismatch is
                // diagnosed first, above), and a kind-0x03 single under the id
                // `hash` is now a CLASS and reaches PreimageNotAdmitted
                // instead. So the only case this arm actually covers is an id
                // outside {entr, hash}, and the body now says so.
                //
                // THE LAST SENTENCE IS THE POINT: §4.3 exists for exactly one
                // case, and the operator most likely to hit this refusal is the
                // one holding a 33-byte seed backup that happens to begin 0x03.
                U::PreimagePlate => format!(
                    "record {i} (records count from 0) is a kind-0x03 preimage payload \
                     whose 4-character id is not `hash`. A preimage plate is kind 0x03 \
                     under the id `hash` (SPEC_ms_hashlock rule 2), and --pack-preimage \
                     admits only that. Re-encode it with `ms hashlock` rather than editing \
                     the string. If this string is a 33-byte seed backup that happens to \
                     begin 0x03, it is not a preimage: roughly 1 in 256 of them look like \
                     this."
                ),
```

- [ ] **Step 6: warnings 1-3, BEFORE the passphrase ceremony.** All three are
payload-wide, so none belongs in the per-record `admit_check`; all three print
in the pack arm beside the existing `--expect` and unsigned-override reporting,
which `main.rs:1474-1494` already orders that way and says why (F-246).
```rust file=me/crates/me-cli/src/main.rs mode=fragment
/// H6 §3.3 / §8.2 — the payload-wide warnings `--pack-preimage` prints, on
/// stderr, BEFORE the passphrase ceremony (warnings 1-3).
///
/// Warning 4, the sealed-transit note, is printed by [`report_sealed_preimage`]
/// AFTER the sealing line, because its own wording ("the device needs the
/// passphrase above") refers to it.
///
/// Host lines are stderr, carry no panel budget, and are exempt from the
/// device's ASCII rule; the shipped refusals already carry em dashes.
fn report_preimage_admission(records: &[String]) {
    use mnemonic_engrave::sysw::composer_records::{parse, ComposerRecord};
    use mnemonic_engrave::sysw::record::Class as C;
    use mnemonic_engrave::sysw::classify;

    let carriers: Vec<(usize, C)> = records
        .iter()
        .enumerate()
        .filter_map(|(i, r)| match classify(r) {
            c @ (C::Preimage | C::Phrase) => Some((i, c)),
            _ => None,
        })
        .collect();

    // §8.2.2 — the flag with nothing to admit. A WARNING, never a refusal: the
    // flag loosens admission, and loosening it over nothing costs nothing.
    if carriers.is_empty() {
        eprintln!(
            "me: --pack-preimage was passed and this payload holds no preimage plate and no \
             `phrase:` record. Nothing was admitted that would otherwise have been refused."
        );
        return;
    }

    // §8.2.1 — transit, always, when a preimage or `phrase:` record is admitted.
    eprintln!(
        "me: WARNING — this payload carries a hashlock PREIMAGE. Anyone who holds the tag \
         can read it, and for a key-less hashlock path the preimage alone spends the coins. \
         Treat this payload as bearer material until it is on the machine and erased."
    );

    // §8.2.3 — the orphan check, over BOTH carriers.
    //
    // The `phrase:` half is the one that matters: a preimage record is produced
    // by `ms hashlock --out` and is correct by construction, while a `phrase:`
    // record has no CLI producer at all, so the operator hand-builds it as hex.
    // Two hand-build errors pass every rule the parser applies and are caught by
    // nothing else -- a SPACE AFTER THE COMMA (printable ASCII, so
    // `hardened, my phrase` is admitted and derives a different preimage from
    // `my phrase`), and the WRONG METHOD SELECTOR. Both are one PBKDF2 run away
    // here -- milliseconds on the host against ~10 s on the device, after a
    // pick, at a screen with no copy for a digest that matches nothing.
    let hashes: Vec<[u8; 32]> = records
        .iter()
        .filter_map(|r| match parse(r) {
            Some(Ok(ComposerRecord::Hash(h))) => Some(h),
            _ => None,
        })
        .collect();
    for (i, class) in carriers {
        let Some(digest) = preimage_digest_of(&records[i], class) else {
            continue;
        };
        if hashes.contains(&digest) {
            continue;
        }
        let hx = hex(&digest);
        let (first8, last8) = (&hx[..8], &hx[56..]);
        // INCOMPLETE versus CONTRADICTORY, and only the second is a WARNING.
        // `ms hashlock --out X.txt` writes only the ms1 string and prints
        // `hash:` to stdout, so the minimal correct journey packs a payload with
        // no `hash:` record at all -- and a WARNING on every single run is how a
        // warning stops being read.
        if hashes.is_empty() {
            eprintln!(
                "me: note — this payload holds no `hash:` record, so nothing here says which \
                 policy the preimage unlocks. The Hashlock plates flow will print the digest \
                 alone."
            );
            continue;
        }
        match class {
            C::Phrase => eprintln!(
                "me: WARNING — record {i} (records count from 0) is a hashlock phrase whose \
                 digest {first8}..{last8} matches no `hash:` record in this payload. Check \
                 the method selector and the text after the first comma — a space after the \
                 comma is part of the phrase and derives a different preimage."
            ),
            _ => eprintln!(
                "me: WARNING — record {i} (records count from 0) is a preimage whose digest \
                 {first8}..{last8} matches no `hash:` record in this payload. Nothing here \
                 tells the device which policy it unlocks, and the Hashlock plates flow will \
                 print the digest alone."
            ),
        }
    }
}
```

The digest an orphan check compares is DECODED for a plate and DERIVED for a
`phrase:` record, and deriving is what makes the phrase half possible at all:
```rust file=me/crates/me-cli/src/main.rs mode=fragment
/// H = sha256(X) for an admitted carrier: decoded for a plate, DERIVED for a
/// `phrase:` record.
///
/// Deriving is what makes §8.2.3's phrase half possible at all, and it is the
/// reason the check lives on the host: PBKDF2 at 100,000 iterations is
/// milliseconds here and about ten seconds on the SH2.
fn preimage_digest_of(
    record: &str,
    class: mnemonic_engrave::sysw::record::Class,
) -> Option<[u8; 32]> {
    use mnemonic_engrave::sysw::composer_records::{parse, ComposerRecord};
    use mnemonic_engrave::sysw::record::Class as C;
    match class {
        C::Phrase => match parse(record) {
            Some(Ok(ComposerRecord::Phrase(p))) => {
                let x = p.method.preimage(p.phrase.as_bytes());
                Some(ms_codec::hashlock::digest(&x))
            }
            _ => None,
        },
        C::Preimage => match ms_codec::decode(record.trim()) {
            Ok((_, ms_codec::Payload::Preimage(x))) => Some(ms_codec::hashlock::digest(&x)),
            _ => None,
        },
        _ => None,
    }
}
```

- [ ] **Step 7: warning 4, AFTER the sealing line**, because its own wording
("the device needs the passphrase above") refers to it. §3.4's correction is
carried in the body: the F-474 arm belongs to `me seal`'s **Sealed Payload**, a
different container, and a sealed `sysw` payload's preimage IS reachable after
unlocking (`sysw/open.go:36-73` runs no admission; `gui/sysw_session.go:79-110`
appends both sections).
```rust file=me/crates/me-cli/src/main.rs mode=fragment
/// H6 §8.2.4 — the sealed-transit note, printed AFTER the sealing line.
///
/// Each clause is true of the container it names. `me seal` — the frozen Sealed
/// Payload container — refuses a preimage plate outright, at pack time on the
/// host and in `AdmitSection` on the device. THIS container does not: `sysw.Open`
/// runs no admission and the session appends both sections, so a sealed `sysw`
/// payload's preimage IS reachable after unlocking.
fn report_sealed_preimage(records: &[String]) {
    use mnemonic_engrave::sysw::classify;
    use mnemonic_engrave::sysw::record::Class as C;
    if !records
        .iter()
        .any(|r| matches!(classify(r), C::Preimage | C::Phrase))
    {
        return;
    }
    eprintln!(
        "me: this payload is SEALED and holds a hashlock preimage, so the device needs the \
         passphrase above before it can reach it. `me seal` — the Sealed Payload container — \
         refuses a preimage plate outright; this one does not."
    );
}
```

its call site, guarded so the note follows a line that exists:

```rust file=me/crates/me-cli/src/main.rs mode=fragment
            if sealing {
                report_sealed_preimage(&recs);
            }
```

- [ ] **Step 8: the two operator-facing surfaces the new classes reach.**
`class_name` gains two arms, and `print_composer_confirmation` gains one that
names the class and the method and NEVER the phrase, the derived preimage or
its digest — a confirmation line goes to a terminal whose scrollback outlives
the run.

```rust file=me/crates/me-cli/src/main.rs mode=fragment
        C::Preimage => "hashlock preimage plate",
        C::Phrase => "hashlock phrase (phrase:)",
```

```rust file=me/crates/me-cli/src/main.rs mode=fragment
            ComposerRecord::Phrase(_) => {
                // The record is SECRET and BEARER. `show` names the class and
                // the method and NEVER the phrase, the derived preimage or its
                // digest: a confirmation line is printed to a terminal whose
                // scrollback outlives the run.
                println!("secret record {i}: hashlock phrase (phrase:) — not shown");
            }
```

- [ ] **Step 9: the shipped unit test H6 changes.**
`sysw::tests::a_preimage_plate_is_named_not_misdiagnosed` moves from
`Unclassifiable(0, PreimagePlate)` to `PreimageNotAdmitted(0, Preimage)` for the
WELL-FORMED plate, keeps `Unclassifiable` for the malformed, mistagged and
uppercase rows, and gains two H6 rows:

```rust file=me/crates/me-cli/src/sysw/mod.rs mode=fragment
        // ... and WITH the flag it is admitted.
        assert!(pack_with(
            vec![PLATE.into()],
            None,
            ITER,
            Admission {
                pack_preimage: true,
                ..Default::default()
            }
        )
        .is_ok());
```

```rust file=me/crates/me-cli/src/sysw/mod.rs mode=fragment
        // H6 §4.3, the funds-relevant row: --pack-preimage admits the id `hash`
        // and NOTHING else. A kind-0x03 single under any other id stays
        // unclassifiable WITH the flag, so a plain BIP-93 33-byte secret
        // beginning 0x03 -- roughly 1 in 256 of them -- never reaches a flow
        // that engraves.
        //
        // MUTATION: drop the id test from `preimage_plate_admissible` -> this
        // becomes Ok(_) and the mistagged record is packed.
```

- [ ] **Step 10: THE ARGV SURFACE MOVED, and this is the plan's third finding.**
`Class::Preimage` and `Class::Phrase` are BEARER, so `is_argv_forbidden()` is
true and `argv_secret_guard` (`crates/me-cli/src/main.rs:566`) refuses either on
the command line BEFORE the parser runs. Two consequences, and the spec states
neither:

1. **Every invocation must use the private channel.** `me sysw pack
   --pack-preimage --no-passphrase <ms1>` — §12 item 3's own acceptance
   invocation — is now REFUSED. `--in records.txt` is the route, and it is the
   one the guard's own message already offers. **§12 item 3 and the toolkit
   manual must say so** (Task 13).
2. **The guard's MESSAGE was false for the new classes**, and a refusal that
   names the wrong material is a defect in what the tool claims to have found.
   It said *"BEARER material -- a signed transaction, or the mt1 set carrying
   one. Anyone who can read it can broadcast it"* for a hashlock preimage:
```rust file=me/crates/me-cli/src/main.rs mode=fragment
            // H6: the two hashlock carriers are BOTH secret AND bearer, so they
            // reach this guard and the transaction wording would be FALSE for
            // them -- a refusal that names the wrong material is a defect in
            // what the tool claims to have found, not a nicety.
            use mnemonic_engrave::sysw::record::Class as GC;
            let what = match class {
                GC::Preimage | GC::Phrase => {
                    "a HASHLOCK PREIMAGE -- the plate string, or the phrase a `phrase:` \
                     record carries. For a key-less hashlock path it alone spends the coins"
                }
                _ if class.is_bearer() => {
                    "BEARER material -- a signed transaction, or the mt1 set carrying one. \
                     Anyone who can read it can broadcast it"
                }
                _ => "SECRET key material. It can spend everything derived from it, forever",
            };
```

- [ ] **Step 11: `crates/me-cli/tests/sysw_pack_preimage.rs`** — nine tests, all
WIRED AND RUN. Every invocation goes through `--in`, and that is H6's own
consequence rather than test hygiene:
```rust file=me/crates/me-cli/tests/sysw_pack_preimage.rs mode=fragment
/// Every invocation goes through `--in`, NEVER argv, and that is H6's own
/// consequence rather than test hygiene: `Class::Preimage` and `Class::Phrase`
/// are BEARER, so `argv_secret_guard` refuses either on the command line before
/// the parser runs. **§12 item 3's acceptance path has to use a file too**, and
/// so does every operator.
fn run_with(flags: &[&str], records: &[&str]) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    let recs = dir.path().join("records.txt");
    std::fs::write(&recs, records.join("\n") + "\n").unwrap();
    let out = dir.path().join("payload.bin");
    let mut a: Vec<String> = vec![
        "sysw".into(),
        "pack".into(),
        "--in".into(),
        recs.display().to_string(),
        "--out".into(),
        out.display().to_string(),
    ];
    a.extend(flags.iter().map(|s| s.to_string()));
    let o = me().args(&a).output().unwrap();
    std::mem::forget(dir);
    o
}
```


| test | asserts | MUTATION that must red it |
| --- | --- | --- |
| `no_flag_refuses_both_carriers_by_index` | a plate and a `phrase:` record each refuse with §8.1.1 at their index | drop `admit_check`'s new rule → both are admitted |
| `the_flag_admits_both` | `--pack-preimage` packs both at exit 0 | — |
| `the_flag_seals_by_default_and_names_the_class` | `--pack-preimage` alone prints `sealing:  SEALED` naming `hashlock preimage plate` | make the classes non-secret → `NOT SEALED`, and the payload ships bearer material in cleartext |
| `classification_is_unconditional` | `classify` answers `Preimage`/`Phrase` under `Admission::default()` AND under `pack_preimage` | gate the classifier on admission → the corpus row and the device's `Classify` disagree; Task 7's lockstep reds |
| `the_id_partition` | id `hash` admissible; id outside `{entr,hash}` → §8.1.2 with and without the flag; id `entr` → the SHIPPED `TagKindMismatch` text, unchanged | use `entr` as the "any other id" case → the row expects the wrong text |
| `warning_order_against_the_ceremony` | warnings 1-3 appear BEFORE the generated-passphrase block on stderr; warning 4 after the sealing line | move warning 1 after the ceremony → the F-246 ordering row reds |
| `the_orphan_warning_covers_both_carriers` | a `phrase:` record whose derived digest matches no `hash:` warns; one that matches does not | drop the phrase arm → the hand-built rows print nothing |
| `a_space_after_the_comma_is_a_different_phrase` | `hardened, my phrase` and `hardened,my phrase` derive DIFFERENT digests and the first warns | — this row IS the hand-build error |
| `no_hash_record_at_all_is_a_note_not_a_warning` | a payload with no `hash:` record draws `me: note —`, never `WARNING —` | make it a WARNING → §12 item 3's own acceptance path fires one every run, which is how a warning stops being read |
| `the_flag_over_nothing_is_a_warning` | `--pack-preimage` with no carrier: exit 0 and §8.2.2 | refuse instead → a loosening flag refuses over nothing |

**MUTATIONS, all five EXECUTED and quoted:**

| mutation | measured failure |
| --- | --- |
| drop `Preimage`/`Phrase` from `Class::is_secret` | `the_warnings_print_in_the_f246_order`: **`the passphrase ceremony did not run`** — i.e. NOT SEALED, and the payload ships bearer material in cleartext. **This is the funds-relevant row, and it is why §3.2 gates admission and not classification.** |
| drop `admit_check`'s second rule | `no_flag_refuses_both_carriers_by_index`: `carrier 0 packed without the flag` |
| drop the phrase arm from `preimage_digest_of` | `a_space_after_the_comma_derives_a_different_preimage_and_warns`: `the space-after-the-comma row did not warn` |
| make the no-`hash:` case a `WARNING` | `no_hash_record_at_all_is_a_note_not_a_warning`: `the incomplete case did not draw the NOTE` |
| drop the id test from `preimage_plate_admissible` | `the_three_ids_each_get_their_own_refusal`: `not §8.1.2: me: record 0 … is a hashlock PREIMAGE plate (kind 0x03) … Re-run with --pack-preimage` — the mistagged record became a CLASS and got the wrong refusal |

**Boundary gate (RUN, after Steps 1-11):** `cargo nextest run --locked -p
mnemonic-engrave --no-fail-fast` — **621 tests run: 618 passed, 3 failed**
(`history_purge`), 2 skipped.

---

### Task 4: the constant-time QR encoder to v9, and its scale-2 arm (spec §7)

**Repo:** the fork, branch `hashlock-h6` off `fb0dd04`.

**Files:**
- Modify: `engrave/engrave.go`, `engrave/engrave_test.go`
- Create: `engrave/h6_qr_test.go`, `engrave/testdata/h6-qr-v{6,7,8,9}-scale2.bin`

**Interfaces:**
- Produces: `qrAlignCentres`; `bitmapForQRStatic` for 41/45/49/53; `ConstantQR` bound `dim > 53`; `constantTimeQRModules` arms for 41/45/49/53; `engraveModule` `case 2`.
- Consumes: nothing new.
- Unchanged: `centerOf`, `constantTimeStartEnd`, `findPath`, `ConstantQRCmd.Engrave`'s loop, and `bitmapForQRStatic`'s `default: panic` — which is what keeps an unhandled version from being drawn as if it had no alignment patterns at all.

**§7.4: THE TWO HALVES ARE ONE DELIVERABLE.** With the raise alone,
`ConstantQR` returns a command for a 53-module code and `Engrave` panics on the
only scale the plate fits at; with the arm alone, `ConstantQR` refuses the code
before `Engrave` is reached. Every test row below is written so that removing
either half reds.

- [ ] **Step 1: the alignment table.** v6 is free — its single ring sits at
`(dim-9, dim-9)`, which is the formula the case arm already uses, so adding
`41` to it is the whole change. v7-v9 take SIX rings and are tabulated.
```go file=fork/engrave/engrave.go mode=fragment
	var alignMarkers []bezier.Point
	switch dim {
	case 21:
		// No marker.
	case 25, 29, 33, 37, 41:
		// Single marker. v5 (37) and v6 (41) each still take exactly one
		// alignment pattern, at the same (dim-9, dim-9) offset as v2-v4.
		//.
		alignMarkers = append(alignMarkers, bezier.Pt(dim-9, dim-9))
	case 45, 49, 53:
		// v7-v9 take SIX alignment patterns: the version's three alignment
		// coordinates crossed with themselves, less the three combinations the
		// position markers already occupy. The centres are tabulated rather
		// than computed because none of them follow the (dim-9) rule above,
		// and TestAlignmentTableMatchesTheEncoder derives this table from the
		// encoder's own bitmap on every run so it cannot be transcribed wrong.
		//
		// fillMarker takes a TOP-LEFT, so each entry is (centre - 2).
		for _, c := range qrAlignCentres[dim] {
			alignMarkers = append(alignMarkers, bezier.Pt(c.X-2, c.Y-2))
		}
	default:
		panic("unsupported qr code version")
	}
	return posMarkers, alignMarkers
}
```
```go file=fork/engrave/engrave.go mode=fragment
// qrAlignCentres are the alignment-pattern CENTRES of the versions whose
// patterns do not sit alone at (dim-9, dim-9). Keyed by dimension:
// v7 (45), v8 (49), v9 (53).
var qrAlignCentres = map[int][]bezier.Point{
	45: {{X: 22, Y: 6}, {X: 6, Y: 22}, {X: 22, Y: 22}, {X: 38, Y: 22}, {X: 22, Y: 38}, {X: 38, Y: 38}},
	49: {{X: 24, Y: 6}, {X: 6, Y: 24}, {X: 24, Y: 24}, {X: 42, Y: 24}, {X: 24, Y: 42}, {X: 42, Y: 42}},
	53: {{X: 26, Y: 6}, {X: 6, Y: 26}, {X: 26, Y: 26}, {X: 46, Y: 26}, {X: 26, Y: 46}, {X: 46, Y: 46}},
}
```

**MEASURED, by testing the 5x5 ring shape at every candidate centre of the
encoder's own bitmap** — the derivation is kept as a test so the table cannot be
transcribed wrong:

```
v2 dim=25 rings=1 (18,18)          v6 dim=41 rings=1 (34,34)
v3 dim=29 rings=1 (22,22)          v7 dim=45 rings=6 (22,6) (6,22) (22,22) (38,22) (22,38) (38,38)
v4 dim=33 rings=1 (26,26)          v8 dim=49 rings=6 (24,6) (6,24) (24,24) (42,24) (24,42) (42,42)
v5 dim=37 rings=1 (30,30)          v9 dim=53 rings=6 (26,6) (6,26) (26,26) (46,26) (26,46) (46,46)
```

Every row of §7.2 reproduced exactly.

- [ ] **Step 2: the bound.**
```go file=fork/engrave/engrave.go mode=fragment
	dim := qrc.Size
	if dim > 53 {
		// The bound is v9 (dim 53), which is what the H6 hashlock phrase plate
		// needs: ECC-L caps at 192 bytes at the last full step below the
		// 194-byte worst case (hashlock v1 + the 73-character hardened method
		// line + a 100-character phrase, SPEC_hashlock_H6 8.6/7.1).
		// bitmapForQRStatic tabulates 21/25/29/33/37/41/45/49/53 only, and
		// constantTimeQRModules has an arm for each, so rejecting here is what
		// keeps a larger version from reaching either default and panicking or
		// silently refusing. Raise all three together or not at all.
		return nil, fmt.Errorf("engrave: constant QR size too large: %d", dim)
	}
```

- [ ] **Step 3: the four `constantTimeQRModules` arms — THE FUZZED BUDGET.**

**The number cannot be invented, because it is content-dependent**, and a budget
between the observed min and max makes the plate cuttable for some phrases and
refused for others AT THE SAME QR VERSION — an operator-visible,
content-dependent failure on the funds path, which is the exact class
`ConstantQR` exists to remove. `nmod` is BOTH the budget `ConstantQR` checks and
the loop count `Engrave` runs, so a wrong entry is not a near miss.

The campaign the plan author ran, on the protocol the v5 entry records
(`engrave/engrave.go:363-371`): §8.6-shaped payloads only — a random printable-ASCII
phrase at a length that reaches the version under one of the two method lines —
32 minutes per dimension on 24 cores, with the last-improvement sample recorded
so convergence is a measurement rather than a hope.

| dim | v | samples | observed max | max/dim^2 | buffer | ENTRY | `findPath` errors | converged (quiet) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 41 | 6 | 14,494,291 | **823** | 0.4896 | +20 | **843** | 0 | 16.4 min |
| 45 | 7 | 11,065,492 | **960** | 0.4741 | +53 | **1013** | 0 | 2.8 min |
| 49 | 8 | 10,138,994 | **1179** | 0.4910 | +20 | **1199** | 0 | 4.1 min |
| 53 | 9 | 7,759,282 | **1379** | 0.4909 | +20 | **1399** | 0 | 22.6 min |

**The campaign CLEARS the spec's stated floor at every dimension**, which is the
checkable gate §7.2 item 4 asks for -- *"a plan-time entry that comes in BELOW
the floor above is proof of an under-converged run"*. The fold's 216,000-payload
floor was 813 / 945 / 1161 / 1369; this run found 41: 823-813=+10, 45: 960-945=+15, 49: 1179-1161=+18, 53: 1379-1369=+10. **43,458,059
payloads in total, and ZERO `findPath` failures** -- no *"QR modules spaced too
far for constant time engraving"* at any raised version. That is evidence the
path-finder scales, not proof, and §11.3 keeps it as a row.

The four arms, as measured:

```go file=fork/engrave/engrave.go mode=fragment
	case 41:
		// v6 (H6 SPEC_hashlock_H6 §7.2 item 4). Derived by fuzzing on the
		// protocol the v5 entry records, over §8.6-SHAPED PAYLOADS ONLY -- a
		// random printable-ASCII phrase at a length that reaches this version
		// under one of the two method lines, which is the only content this
		// plate ever carries: 14,494,291 executions in 32 min on 24
		// cores, observed max 823, converged with 16.4 min of quiet
		// (the last improvement was at sample 7,013,580). Ratio
		// max/dims^2 = 0.4896; findPath failed on 0 of them.
		//
		// The asymmetry justifies the margin, in the v5 entry's own words: an
		// underestimate produces content-dependent engrave-time failures on a
		// permanent plate, an overestimate costs ~2% more engraving time.
		return 823 + 20
	case 45:
		// v7 (H6 SPEC_hashlock_H6 §7.2 item 4). Derived by fuzzing on the
		// protocol the v5 entry records, over §8.6-SHAPED PAYLOADS ONLY -- a
		// random printable-ASCII phrase at a length that reaches this version
		// under one of the two method lines, which is the only content this
		// plate ever carries: 11,065,492 executions in 32 min on 24
		// cores, observed max 960, converged with 2.8 min of quiet
		// (the last improvement was at sample 10,124,362). Ratio
		// max/dims^2 = 0.4741; findPath failed on 0 of them.
		// ITS RATIO FALLS BELOW ITS NEIGHBOURS (0.4741 against a
		// 0.4905 mean of the other three), which is exactly the signal the
		// v5 entry used to justify a buffer of 20 rather than the historical 5.
		// Trend-implied max at that ratio is ~993, so the buffer is widened to
		// 53 rather than left at 20.
		//
		// The asymmetry justifies the margin, in the v5 entry's own words: an
		// underestimate produces content-dependent engrave-time failures on a
		// permanent plate, an overestimate costs ~2% more engraving time.
		return 960 + 53
	case 49:
		// v8 (H6 SPEC_hashlock_H6 §7.2 item 4). Derived by fuzzing on the
		// protocol the v5 entry records, over §8.6-SHAPED PAYLOADS ONLY -- a
		// random printable-ASCII phrase at a length that reaches this version
		// under one of the two method lines, which is the only content this
		// plate ever carries: 10,138,994 executions in 32 min on 24
		// cores, observed max 1179, converged with 4.1 min of quiet
		// (the last improvement was at sample 8,820,790). Ratio
		// max/dims^2 = 0.4910; findPath failed on 0 of them.
		//
		// The asymmetry justifies the margin, in the v5 entry's own words: an
		// underestimate produces content-dependent engrave-time failures on a
		// permanent plate, an overestimate costs ~2% more engraving time.
		return 1179 + 20
	case 53:
		// v9 (H6 SPEC_hashlock_H6 §7.2 item 4). Derived by fuzzing on the
		// protocol the v5 entry records, over §8.6-SHAPED PAYLOADS ONLY -- a
		// random printable-ASCII phrase at a length that reaches this version
		// under one of the two method lines, which is the only content this
		// plate ever carries: 7,759,282 executions in 32 min on 24
		// cores, observed max 1379, converged with 22.6 min of quiet
		// (the last improvement was at sample 2,213,470). Ratio
		// max/dims^2 = 0.4909; findPath failed on 0 of them.
		//
		// The asymmetry justifies the margin, in the v5 entry's own words: an
		// underestimate produces content-dependent engrave-time failures on a
		// permanent plate, an overestimate costs ~2% more engraving time.
		return 1379 + 20
```

**§11.3's BUDGET MUTATION CANNOT FIRE FROM AN IN-SUITE SAMPLE, and this plan
adds the row that can** (author finding, MEASURED). §11.3 says *"set any of the
four `constantTimeQRModules` arms to the observed maximum MINUS ONE
→ that version's row reds, which is also the check that the entry was derived
rather than guessed."* Run it: with the arms at 822 / 959 / 1178 / 1378 the
fuzzed row **still PASSES**, because its 400 payloads per dimension observe
792 / 910 / 1143 / 1322 — and reaching the true maximum took between 7 and 14
MILLION payloads. A suite cannot spend 32 minutes a dimension, so no in-suite
sample can carry that mutation, and a plan that relied on it would ship a budget
nothing guards.

So the entries get a PIN, which is a different test rather than a weaker one:
the fuzzed row proves the entry BOUNDS fresh content; the pin proves it is the
number the campaign produced. Changing an entry now means changing the table,
which means saying where the new number came from.

```go file=fork/engrave/h6_qr_test.go mode=fragment
// TestConstantTimeQRBudgetEntriesAreTheFuzzedOnes PINS the four entries to the
// numbers the plan-time campaign produced, and it exists because
// TestConstantTimeQRBudgetBoundsEveryPayload CANNOT CATCH A ONE-OFF CHANGE.
//
// MEASURED: with a budget set to each campaign maximum MINUS ONE
// (822 / 959 / 1178 / 1378), the fuzzed row above still PASSES -- its 400
// payloads per dimension observed 792 / 910 / 1143 / 1322, well under the
// altered budgets, and reaching the true maximum took 7 to 14 million payloads.
// §11.3's stated mutation ("set any of the four arms to the observed maximum
// MINUS ONE -> that version's row reds") is therefore not achievable by any
// in-suite sample, and a plan that relied on it would ship a budget nothing
// guarded.
//
// So the guard is a PIN. It is not a weaker test than the fuzzed one, it is a
// different one: the fuzzed row proves the entry BOUNDS fresh content, and this
// row proves the entry is the one that was DERIVED rather than a number someone
// adjusted. Changing an entry now means changing this table, which means saying
// where the new number came from.
//
// MUTATION: any change to any of the four arms -> the entry assertion names the
// dimension, both values, and the campaign that produced the original.
func TestConstantTimeQRBudgetEntriesAreTheFuzzedOnes(t *testing.T) {
	for _, tc := range []struct {
		dim, observed, buffer int
		samples               string
	}{
		{41, 823, 20, "14,494,291 in 32 min, converged with 16.4 min of quiet"},
		{45, 960, 53, "11,065,492 in 32 min, quiet only 2.8 min -- the UNDER-CONVERGED entry, hence the wider buffer"},
		{49, 1179, 20, "10,138,994 in 32 min, quiet 4.1 min"},
		{53, 1379, 20, "7,759,282 in 32 min, converged with 22.6 min of quiet"},
	} {
		want := tc.observed + tc.buffer
		if got := constantTimeQRModules(tc.dim); got != want {
			t.Errorf("constantTimeQRModules(%d) = %d, want %d (%d observed + %d buffer). "+
				"The original was derived by fuzzing: %s. A different number needs its own campaign.",
				tc.dim, got, want, tc.observed, tc.buffer, tc.samples)
		}
	}
	// And the shipped entries are untouched.
	for dim, want := range map[int]int{21: 171, 25: 266, 29: 391, 33: 547, 37: 684} {
		if got := constantTimeQRModules(dim); got != want {
			t.Errorf("constantTimeQRModules(%d) = %d, want the SHIPPED %d: H6 raises the "+
				"ceiling and moves no existing entry", dim, got, want)
		}
	}
}
```

**MUTATION, executed:** `return 1379 + 20` → `return 1378` gives
`constantTimeQRModules(53) = 1378, want 1399 (1379 observed + 20 buffer). The
original was derived by fuzzing: 7,759,282 in 32 min, converged with 22.6 min of
quiet. A different number needs its own campaign.`

- [ ] **Step 4: `engraveModule`'s `case 2`.** It is NOT "copy case 3 with
smaller numbers". `centerOf` is `(p*scale+1)*sw + sw/2`, which puts the centre of
module `p` at `p*scale*sw + 1.5sw` — the middle of a 3-stroke cell but 1.5
strokes into a 2-stroke one, half a stroke off-centre, exactly as it is for
scale 4, whose arm compensates the same way.
```go file=fork/engrave/engrave.go mode=fragment
	case 2:
		// ASYMMETRIC, and not "case 3 with smaller numbers". centerOf puts the
		// centre of module p at p*scale*sw + 1.5*sw, which is the middle of a
		// 3-stroke cell but 1.5 strokes into a 2-stroke one -- half a stroke
		// off-centre, exactly as it is for case 4, whose arm compensates the
		// same way. The cell is [p*2sw, p*2sw+2sw], so relative to `center` the
		// painted extent must be [-1.5sw, +0.5sw] and the PATH must run
		// [-sw, 0] on both axes: a closed square on the corners (-sw,-sw),
		// (0,-sw), (0,0) and (-sw,0), with `center` ON the (0,0) corner.
		//
		// FIVE commands, the same constant as case 3, and that is the property
		// this arm has to carry: ConstantQRCmd.Engrave pads every move to
		// maxDur and runs `for range nmod`, so a per-module command count that
		// is constant in the CONTENT is what keeps the toolpath
		// content-independent. An arm whose command count varied by module
		// would re-open the timing leak ConstantQR exists to close.
		return yield(Line(center.Add(bezier.Pt(-sw, -sw)))) &&
			yield(Line(center.Add(bezier.Pt(0, -sw)))) &&
			yield(Line(center)) &&
			yield(Line(center.Add(bezier.Pt(-sw, 0)))) &&
			yield(Line(center.Add(bezier.Pt(-sw, -sw))))
```

**MEASURED at the production stroke (`internal/sh2.Params`: StrokeWidth 1920,
Millimeter 6400), with scale 3 as the control:**

```
scale=2 p={0 0} cmds=5 ink x=[0,3840] want [0,3840] | y=[0,3840] want [0,3840]
scale=2 p={1 0} cmds=5 ink x=[3840,7680] want [3840,7680] | y=[0,3840] want [0,3840]
scale=2 p={5 7} cmds=5 ink x=[19200,23040] want [19200,23040] | y=[26880,30720] want [26880,30720]
scale=3 p={5 7} cmds=5 ink x=[28800,34560] want [28800,34560] | y=[40320,46080] want [40320,46080]
```

3,840 units = **0.6 mm**, the cell exactly, at every probed position, with no
overlap into a neighbour — and **five commands, the same constant as case 3**,
which is the property that keeps the toolpath content-independent.

- [ ] **Step 5: the tests.** Create `engrave/h6_qr_test.go`. Six rows, and §11.3's
own distinction between the two QR rows is kept in the names and the headers:

| test | what it proves |
| --- | --- |
| `TestQRAlignmentTableMatchesTheEncoder` | the table is DERIVED from the bitmap for v2..v9 and compared |
| `TestConstantQRAcceptsThroughV9AndRefusesV10` | 21..53 accepted, 57 refused; the `default: panic` still stands behind it |
| `TestECCLThresholdsAreWhatTheBudgetAssumes` | the byte→dim table the fuzzer samples from, pinned against the encoder for 79..240 bytes |
| `TestConstantTimeQRBudgetBoundsEveryPayload` | the entry BOUNDS fresh content — 400 fuzzed §8.6 payloads per version, `findPath` clean |
| `TestConstantTimeQRBudgetEntriesAreTheFuzzedOnes` | **the entry is the one that was DERIVED** — see the finding below |
| `TestConstantQRMoveCountIsAFunctionOfDimAlone` | **THE REGRESSION GUARD, labelled as one** |
| `TestEngraveModuleScale2` + `TestConstantQREngraveAtScale2Completes` | five commands, exact extents, and both halves end to end |
| `TestH6ConstantQRGoldens` | one golden per newly admitted version, at scale 2 |

The budget row's header states outright what the guard row cannot do, because
the first draft's test could not earn the constant-time argument:
```go file=fork/engrave/h6_qr_test.go mode=fragment
// TestConstantQRMoveCountIsAFunctionOfDimAlone IS A REGRESSION GUARD, NOT THE
// BUDGET PROOF, and it is labelled as one because it cannot substitute.
//
// Engrave loops `for range nmod` and pads every move to maxDur via DelayMove,
// so the emitted count is a function of dim alone FOR ANY VALUE OF nmod,
// correct or not -- this row passes on a budget of 0, on 700, and on a correct
// one. What it does prove is the padding: two different payloads of the same
// version emit the same commands in the same count.
//
// MUTATION: make the move list depend on a module's colour (skip the
// engraveModule call when the module is already engraved) -> the two payloads
// differ in command count.
```

and the scale-2 test uses the PRODUCTION stroke rather than the package's own,
for a stated reason that is not cosmetic:
```go file=fork/engrave/h6_qr_test.go mode=fragment
	// The PRODUCTION stroke (internal/sh2.Params: StrokeWidth 1920, Millimeter
	// 6400), not this package's own `strokeWidth = mm/3 = 2133`. centerOf adds
	// sw/2 and engraveModule's extent is grown by sw/2 on each side, so an ODD
	// stroke loses a unit to integer division and the exact-extent assertion
	// below would be off by one for a reason that has nothing to do with the
	// arm. 2*1920 = 3840 units = 0.6mm, the free-text plate's module pitch.
	const sw = 1920
```

- [ ] **Step 6: the two shipped records H6 falsifies.** Neither is in the spec's
§14 and both were found by running the suite.

`TestConstantQRLargeVersionsFailClosed` asserted that **dim 41 is refused**. The
fail-closed property is unchanged; the version it is asserted at moves to v10:
```go file=fork/engrave/engrave_test.go mode=fragment
// H6 MOVED THE CEILING, and this test moved with it. Until H6 the bound was v5
// (dim 37) and this test asserted that v6 (dim 41) was refused, because spec O6
// resolved the passphrase plate to v5 only. H6 §7 raises the bound to v9
// (dim 53) for the hashlock phrase plate's 194-byte worst case and gives
// bitmapForQRStatic and constantTimeQRModules an entry for each of
// 41/45/49/53, so 41 is now ADMITTED and TestConstantQRAcceptsThroughV9AndRefusesV10
// asserts it. The fail-closed property is unchanged; only the version it is
// asserted at moved, to v10 (dim 57), which is the first version with no
// alignment-centre row and no fuzzed budget.
//
// An earlier version of this test called t.Skipf when ConstantQR errored,
// which was ALWAYS, so it asserted nothing beyond "did not panic" while its
// comment claimed to check for silent truncation. Assert the rejection.
func TestConstantQRLargeVersionsFailClosed(t *testing.T) {
	long := strings.Repeat("Xy7#", 60) // 240 bytes -> v10, dim 57
	c, err := qr.Encode(long, qr.L)
	if err != nil {
		t.Fatal(err)
	}
	if c.Size != 57 {
		t.Fatalf("expected dim 57, got %d", c.Size)
	}
	if _, err := ConstantQR(c); err == nil {
		t.Fatal("ConstantQR accepted a dim-57 code; the guard is meant to reject it")
	}
}
```

and `TestPassphraseQRFitsSupportedVersion`'s boundary comment, which called dim
41 *"deliberately unsupported"*:
```go file=fork/engrave/engrave_test.go mode=fragment
	// Boundary: v5-L holds 106 bytes, so the 100-char cap has 6 chars of
	// headroom and 107 is where dim 41 begins. Dim 41 is ADMITTED since H6 §7
	// (it is the hashlock phrase plate's smallest version); what this row pins
	// is that a 100-character PASSPHRASE still never reaches it, so the
	// passphrase plate's own QR stays at v5 and its scale-3 goldens do not move.
```

**Boundary gate (RUN):** `go test ./engrave/` after generating the four goldens
with `-update`. **The goldens depend on the budget** — `Engrave` loops
`for range nmod` — so they are generated AFTER Step 3's final values, never
before.

---

### Task 5: `codex32.EncodeMS1Preimage` and `IsPreimagePlate` (spec §3.5, §4.3)

**Repo:** the fork. **Disjoint from Task 4**; runs in parallel with it.

**Files:**
- Modify: `codex32/msencode.go`, `codex32/mspayload.go`
- Create: `codex32/msencode_preimage_test.go`

**Interfaces:**
- Produces: `codex32.EncodeMS1Preimage(x [32]byte) (string, error)`; `codex32.IsPreimagePlate(s String) bool`.
- Consumes: `NewSeed`, `msPrefixPreimage`, `String.Split` (`codex32/codex32.go:394-401`), `DecodeMS1Preimage`.
- **`IsPreimage` is UNCHANGED**: it is H0's inertness rule and still governs every other call site (the six live ones are inventoried in spec §14).

**This is a PORT, not new codec design.** `ms_codec::encode` already exists,
already carries the `tag != payload.kind().single_tag()` refusal
(`crates/ms-codec/src/encode.rs:24-31`), and is already the encoder
`ms hashlock` prints the plate with. The Go half is the gap.

- [ ] **Step 1: the encoder.** The id is a FIXED LITERAL with no parameter, and
the doc comment says why:
```go file=fork/codex32/msencode.go mode=fragment
// EncodeMS1Preimage encodes a hashlock preimage X as the ms1 kind-0x03 plate
// string: NewSeed("ms", 0, "hash", 's', [0x03‖X]). It is the Go port of
// ms_codec::encode(Tag::HASH, &Payload::Preimage(x)) (SPEC_ms_hashlock §1
// rule 2) and is DOWNSTREAM of it — the Rust side decides the wire form.
//
// The id is the FIXED literal "hash" and there is no parameter for it. That is
// the whole point: NewSeed will mint a kind-0x03 payload under id "entr" quite
// happily, and the Rust encoder refuses that shape outright with
// Error::TagKindMismatch, so the id must not be reachable from a caller here
// either. A mistagged plate satisfies the wide IsPreimage, fails
// IsPreimagePlate, and is therefore an operator's only backup of a spend
// secret, on steel, that no tool will read.
//
// The returned string is SECRET (it embeds the preimage); the caller scrubs.
// DecodeMS1Preimage(New(EncodeMS1Preimage(x))) == x.
func EncodeMS1Preimage(x [32]byte) (string, error) {
	payload := make([]byte, 0, 1+len(x))
	payload = append(payload, msPrefixPreimage)
	payload = append(payload, x[:]...)
	s, err := NewSeed("ms", 0, "hash", 's', payload)
	if err != nil {
		return "", err
	}
	return s.String(), nil
}
```

**The hazard is real rather than hypothetical, and it is MEASURED.** Running the
mutation that passes `"entr"` to `NewSeed` produces, from the corpus's own X:

```
EncodeMS1Preimage = ms10entrsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kp9wv63u5a0u7q,
             want the corpus ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c
```

`NewSeed` mints that string without complaint; it satisfies the wide
`IsPreimage` and FAILS `IsPreimagePlate`, so §4.3's admission predicate refuses
it on the way back in — an operator's only backup of a spend secret, on steel,
that no tool will read. Hence the fixed id, and hence the test asserts on the
OUTPUT.

- [ ] **Step 2: the admission predicate.**
```go file=fork/codex32/mspayload.go mode=fragment
// IsPreimagePlate is IsPreimage PLUS the id `hash` (SPEC_ms_hashlock §1 rule 2,
// ruling L14). H0's kind-byte rule is unchanged and still governs INERTNESS
// everywhere else; this narrower predicate governs ADMISSION to a flow that
// ENGRAVES.
//
// The two predicates differ because H6 inverts the consequence of a false
// positive. Under H0 a false positive was a REFUSAL — "a refusal costs a
// re-encode; a wrong cut exposes a spend secret", which is why IsPreimage does
// not consult the id. On the H6 admission path a false positive routes a string
// INTO a flow that engraves it under a band reading NOT A SEED, so a plain
// BIP-93 33-byte secret beginning 0x03 — roughly 1 in 256 of them — must not
// arrive. Hence the id.
func IsPreimagePlate(s String) bool {
	if !IsPreimage(s) {
		return false
	}
	id, _, _ := s.Split()
	return id == "hash"
}
```

- [ ] **Step 3: the tests**, `codex32/msencode_preimage_test.go`. Four:
`TestEncodeMS1PreimageMatchesTheCorpusKindRow` (lockstep against the vendored
`hashlock-v0.8.json` `kind` row, byte for byte, 75 characters, id `hash`),
`TestEncodeMS1PreimageRoundTrips`, `TestEncodeMS1PreimageCanNeverEmitIDEntr`
(asserted on `Split()`'s id of the OUTPUT over 256 random X, so a future
refactor that reintroduces an id parameter reds), and
`TestIsPreimagePlateNarrowsByID`.

That last one carries the row the HOST's `a_preimage_plate_is_named_not_misdiagnosed`
also carries, and the two agree by construction rather than by review:
```go file=fork/codex32/msencode_preimage_test.go mode=fragment
	// THE UPPERCASE PLATE -- the QR-alphanumeric spelling -- is a preimage by
	// the wide predicate and is NOT admissible. Split() reports the id as it is
	// written, and section 5.3 hashes a record in its canonical LOWERCASE form,
	// so an uppercase string is not the record it looks like. The host's
	// `preimage_plate_admissible` compares the id bytes case-SENSITIVELY for
	// exactly this reason, and `a_preimage_plate_is_named_not_misdiagnosed`
	// pins the same row over there.
	//
	// MUTATION: lowercase the id before comparing (or use EqualFold) -> this
	// row admits the uppercase plate and the two sides disagree.
	up, err := New(strings.ToUpper(c.Kind[0].MS1))
	if err != nil {
		t.Fatalf("New(the UPPERCASE plate): %v", err)
	}
	if !IsPreimage(up) {
		t.Error("the uppercase plate is not a preimage by the WIDE predicate; the two no longer part here")
	}
	if IsPreimagePlate(up) {
		t.Error("IsPreimagePlate admitted the UPPERCASE spelling of a plate")
	}
```

**MUTATIONS, all executed (tails in `## Build gate`):** `NewSeed("entr", …)` →
the corpus row and the id row both red; drop the `msPrefixPreimage` byte → the
string is 73 characters and the round trip fails; drop the id test from
`IsPreimagePlate` → `IsPreimagePlate(ms10entrsq…) = true: a kind-0x03 payload
under id "entr" was admitted`.

**Boundary gate (RUN):** `go test ./codex32/` — **ok**.

---

### Task 6: the dedicated plate layout (spec §6)

**Repo:** the fork. **After Task 4** — the worst-case plate's QR is a v9 code at
scale 2, which neither exists nor draws until Task 4 has landed.

**Files:**
- Create: `backup/hashlock.go`, `backup/hashlock_test.go`, `backup/testdata/hashlock-{string-6mm,phrase-noqr,phrase-qr-v9,phrase-space-legend}.bin`
- Modify: `backup/passphrase_test.go` (the second falsified shipped test)

**Interfaces:**
- Produces: `backup.Hashlock`, `backup.HashlockForm` (`HashlockString`, `HashlockPhrase`), `backup.EngraveHashlock`, `backup.HashlockQRCode`, `backup.ErrHashlockTooLarge`, `backup.ErrHashlockQRForm`.
- Consumes: `SpaceMark`, `passphraseGlyphs`, `passphraseLegend`, `CharsPerLine`, `FontSizes`, `engrave.ConstantQR`, `engrave.NewPassphraseStringer`.
- **NEVER `validateMdmkStrings`** (`gui/gui.go:2626-2648`), whose single-string arm offers `TEXT + QR` / `QR ONLY` and QR-encodes THAT STRING — which decision 1 forbids.

- [ ] **Step 1: the type and the constants.** `QRText` is a field rather than
something `backup` composes, for the same reason `Locator` is: `backup` takes no
dependency on `md` or `hashlock`, exactly as `Passphrase` takes none. The caller
(Task 9 / Task 10) passes `ms_codec::hashlock::qr_text`'s Go twin.
```go file=fork/backup/hashlock.go mode=fragment
// HashlockForm selects the band text and the body of a hashlock plate (H6 §6.2).
type HashlockForm int

const (
	// HashlockString cuts the ms1 kind-0x03 plate string, text only.
	HashlockString HashlockForm = iota
	// HashlockPhrase cuts the hashlock phrase and its method, optionally with
	// a QR of §8.6's text.
	HashlockPhrase
)

// Hashlock is a hashlock preimage plate: a DEDICATED layout, never a free-text
// plate and never validateMdmkStrings' single-string arm, whose TEXT + QR /
// QR ONLY options would QR-encode the ms1 string itself (decision 1 forbids
// it).
//
// It is not a seed plate and it is not a bundleCard: it does not travel with
// the policy set, and the footer band says NOT A SEED on both forms.
type Hashlock struct {
	// Form selects the band text and the body (§6.2).
	Form HashlockForm
	// MS1 is the kind-0x03 plate string, for HashlockString. Engraved
	// VERBATIM: lowercase and UNGROUPED, which is what the goldens pin. The
	// fork's ms1 SEED plate does the opposite (EngraveSeedString upper-cases
	// and groups in tens) and so does `ms hashlock`'s engraving card, so
	// "verbatim" is a decision here and not an oversight -- grouped in tens the
	// 75 characters take 8 rows at 6.0mm instead of 4 and the plate no longer
	// fits at that rung.
	MS1 string
	// Phrase and Method are for HashlockPhrase. Phrase is engraved VERBATIM,
	// with every space rendered as SpaceMark; Method is §8.6's line.
	Phrase string
	Method string
	// Locator rows, pre-formatted by the caller (§6.3). backup takes no
	// dependency on md or hashlock, exactly as Passphrase takes none.
	Locator []string
	// QR is opt-in and legal only on HashlockPhrase (§6.4). It encodes §8.6's
	// text, which the caller supplies as QRText -- NEVER the ms1 string.
	QR     bool
	QRText string
	Font   *vector.Face
}
```
```go file=fork/backup/hashlock.go mode=fragment
	hashlockTitleString = "HASHLOCK PREIMAGE"
	hashlockTitlePhrase = "HASHLOCK PHRASE"
	hashlockFooter      = "NOT A SEED"

	// hashlockQRScale is 2, NOT the passphrase plate's 3. MEASURED: at 53
	// modules scale 3 is 47.70mm and does not fit at any rung (3.0mm gives
	// 79.70mm against a 65mm budget); scale 2 is 31.80mm. The pitch matches the
	// free-text plate's (freeTextQRScale = 2, "0.6mm modules against the 0.9mm
	// every other plate uses") but not its code path: this QR is a spend secret
	// and goes through engrave.ConstantQR, never engrave.QR.
	hashlockQRScale = 2
	// hashlockQREnvelope is the reserved module count -- v9, the §8.6 worst
	// case (a 100-character phrase under the hardened method line is 194 bytes
	// and encodes at ECC-L to 53 modules). The size is VARIABLE with the phrase
	// length, so the layout reserves the worst case and centres the actual code
	// inside it, exactly as passphraseQREnvelope does.
	hashlockQREnvelope = 53
	// hashlockQRGap separates the text block from the QR, in millimetres.
	hashlockQRGap = 2
)
```

- [ ] **Step 2: the body, and the wrap rule.** The method line is the FIRST body
row of the phrase form, the locator follows it, a blank separates each block,
and the SECRET is last so a reader knows where it ends.

**A SPEC INCONSISTENCY, resolved and recorded** (author finding): §6.2 and §6.3
put the method line first and the locator after it, while §6.5's worst-case
listing reads *"header … a blank, the … method line, a blank, and a
100-character phrase"* — locator FIRST. The NORMATIVE sections win. The row
count is identical either way (3 + 1 + 2 + 1 + 3 = 10 at 3.0 mm), so no
measurement in §6.5 moves.
```go file=fork/backup/hashlock.go mode=fragment
// hashlockWrap splits s into rows of at most n CHARACTERS. It never breaks on
// word boundaries, and that is forced rather than chosen: a 100-character
// phrase and a 75-character ms1 string are SINGLE TOKENS, so a word wrapper has
// nothing to break them on and would either fail or overflow. The visible
// consequence is that an 8-hex stub splits mid-token at the narrow rungs, which
// is accepted.
func hashlockWrap(s string, n int) []string {
	if n <= 0 {
		return nil
	}
	if s == "" {
		return []string{""}
	}
	var out []string
	for len(s) > n {
		out = append(out, s[:n])
		s = s[n:]
	}
	return append(out, s)
}
```
```go file=fork/backup/hashlock.go mode=fragment
// hashlockSegments is the body in LOGICAL rows, before wrapping (§6.2, §6.3).
//
// The method line is the FIRST body row of the phrase form, directly beneath
// the title band; the locator follows it; a blank separates each block from the
// next; the secret is LAST, so a reader knows where it ends.
func hashlockSegments(plate Hashlock) []hashlockRow {
	var segs []hashlockRow
	if plate.Form == HashlockPhrase && plate.Method != "" {
		segs = append(segs, hashlockRow{text: plate.Method})
		segs = append(segs, hashlockRow{text: ""})
	}
	for _, l := range plate.Locator {
		segs = append(segs, hashlockRow{text: l})
	}
	if len(segs) > 0 {
		segs = append(segs, hashlockRow{text: ""})
	}
	switch plate.Form {
	case HashlockPhrase:
		segs = append(segs, hashlockRow{text: passphraseGlyphs(plate.Phrase), secret: true})
	default:
		segs = append(segs, hashlockRow{text: plate.MS1, secret: true})
	}
	return segs
}
```

- [ ] **Step 3: the layout and the fit.** The vertical budget is 65 mm — the
plate less the two `innerMargin` bands, which hold the title and the footer —
and the group is centred inside it exactly as `passphraseLayoutFor` centres its
own.
```go file=fork/backup/hashlock.go mode=fragment
func hashlockLayoutFor(params engrave.Params, plate Hashlock, fontMM float32, qrDim int) hashlockLayout {
	plateDims := image.Point{X: params.F(plateSize), Y: params.F(plateSize)}
	l := hashlockLayout{
		fontMM:  fontMM,
		em:      params.F(fontMM),
		qrDim:   qrDim,
		smallEm: params.F(plateSmallFontSize),
		topY:    params.F(outerMargin),
		bottomY: params.F(plateSize - innerMargin),
		// The vertical budget for the centred group: the plate less the two
		// innerMargin bands, which hold the title and the footer.
		budget: params.F(plateSize) - 2*params.I(innerMargin),
	}
	switch plate.Form {
	case HashlockPhrase:
		l.topLines = []string{hashlockTitlePhrase}
	default:
		l.topLines = []string{hashlockTitleString}
	}
	if plate.Form == HashlockPhrase && strings.ContainsRune(plate.Phrase, ' ') {
		// The SAME legend the passphrase plate draws, with the same reason:
		// one space and two look identical on steel, and "Correct Horse" and
		// "correct horse" derive different preimages.
		l.bottomLines = append(l.bottomLines, passphraseLegend)
	}
	l.bottomLines = append(l.bottomLines, hashlockFooter)

	perLine := CharsPerLine(params, plate.Font, fontMM)
	for _, seg := range hashlockSegments(plate) {
		for _, r := range hashlockWrap(seg.text, perLine) {
			l.rows = append(l.rows, hashlockRow{text: r, secret: seg.secret})
		}
	}
	l.blockH = len(l.rows) * l.em

	gap := 0
	if qrDim > 0 {
		l.envSize = hashlockQREnvelope * params.StrokeWidth * hashlockQRScale
		l.qrSize = qrDim * params.StrokeWidth * hashlockQRScale
		gap = params.I(hashlockQRGap)
	}
	l.total = l.blockH + gap + l.envSize

	// Centre body, gap and QR envelope as one group on the plate. innerMargin
	// is symmetric, so centring on the plate is centring in the usable area.
	l.textX = params.I(outerMargin)
	l.textY = (plateDims.Y - l.total) / 2
	if qrDim > 0 {
		l.envX = (plateDims.X - l.envSize) / 2
		l.envY = l.textY + l.blockH + gap
		l.qrX = l.envX + (l.envSize-l.qrSize)/2
		l.qrY = l.envY + (l.envSize-l.qrSize)/2
	}
	return l
}
```

- [ ] **Step 4: the auto-fit and the REFUSAL.** It refuses at the bottom rung
rather than drawing what it cannot lay out, the way `EngraveText`, `EngraveSeed`
and `EngraveSeedString` do, and names the measured ceiling — because the
alternative lands on the device, and there is no camera to read a plate back.
```go file=fork/backup/hashlock.go mode=fragment
func EngraveHashlock(params engrave.Params, plate Hashlock) (engrave.Engraving, error) {
	if plate.QR && plate.Form != HashlockPhrase {
		return nil, ErrHashlockQRForm
	}
	var qrc *engrave.ConstantQRCmd
	qrDim := 0
	if plate.QR {
		code, err := HashlockQRCode(plate)
		if err != nil {
			return nil, err
		}
		// ConstantQR, never engrave.QR: the latter engraves in a
		// content-dependent pattern and would leak the phrase through timing.
		qrc, err = engrave.ConstantQR(code)
		if err != nil {
			return nil, err
		}
		qrDim = qrc.Size
	}
	l, ok := hashlockFit(params, plate, qrDim)
	if !ok {
		return nil, fmt.Errorf("%w: %d rows at %.1fmm need %d units against a budget of %d",
			ErrHashlockTooLarge, len(l.rows), l.fontMM, l.total, l.budget)
	}
	return engraveHashlock(params, plate, l, qrc), nil
}

// hashlockFit walks FontSizes largest-first and returns the layout at the first
// rung the whole composition fits. When none fits it returns the BOTTOM rung's
// layout and false, so the refusal can name the measured ceiling.
func hashlockFit(params engrave.Params, plate Hashlock, qrDim int) (hashlockLayout, bool) {
	var l hashlockLayout
	for _, size := range FontSizes {
		l = hashlockLayoutFor(params, plate, size, qrDim)
		if l.fits() {
			return l, true
		}
	}
	return l, false
}
```

- [ ] **Step 5: the engraver.** The SECRET rows go through the constant-time
passphrase stringer, exactly as the passphrase plate's own body does; the method
line and the locator carry a digest, a stub and a parameter set — none of which
is secret — and go through the ordinary `engrave.String` the passphrase plate's
metadata bands already use. The bands are engraved VERBATIM, never through
`TitleString`, which upper-cases and truncates at `MaxTitleLen`.
```go file=fork/backup/hashlock.go mode=fragment
func engraveHashlock(params engrave.Params, plate Hashlock, l hashlockLayout, qrc *engrave.ConstantQRCmd) engrave.Engraving {
	// NewPassphraseStringer, never NewConstantStringer: the shared alphabet is
	// 36 characters and panics on lowercase, and both a phrase and a bech32
	// ms1 string are lowercase.
	constant := engrave.NewPassphraseStringer(plate.Font, params, l.em)
	plateX := params.F(plateSize)
	band := func(t engrave.Transform, y int, lines []string) {
		for i, line := range lines {
			s := engrave.String(plate.Font, l.smallEm, line)
			w, _ := s.Measure()
			t.Offset((plateX-w)/2, y+i*l.smallEm)
			s.Engrave(t.Yield)
		}
	}
	return func(yield func(engrave.Command) bool) {
		t := engrave.NewTransform(yield)
		// The bands are engraved VERBATIM, never through TitleString, which
		// upper-cases and truncates at MaxTitleLen -- the three literals are
		// already inside that cap and already upper-case, and running them
		// through it would let a future longer literal be silently cut.
		band(t, l.topY, l.topLines)

		y := l.textY
		for _, r := range l.rows {
			if r.text != "" {
				off := t.Offset(l.textX, y)
				if r.secret {
					constant.String(off.Yield, r.text)
				} else {
					engrave.String(plate.Font, l.em, r.text).Engrave(off.Yield)
				}
			}
			y += l.em
		}

		if qrc != nil {
			qrCmd := qrc.Engrave(params.StepperConfig, params.StrokeWidth, hashlockQRScale)
			t.Offset(l.qrX, l.qrY)
			qrCmd(t.Yield)
		}

		band(t, l.bottomY, l.bottomLines)
	}
}
```

- [ ] **Step 6: §6.5's geometry, REPRODUCED.** Run against `prodParams`
(`Millimeter = 6400`, `StrokeWidth = 1920` — `internal/sh2.Params`),
`constant.Font`, plate 85 mm:

```
rung | chars/line(79mm) | lines/plate | advance mm
6.0 | 19 | 13 | 4.0000
5.0 | 23 | 15 | 3.3333
4.4 | 26 | 17 | 2.9333
3.8 | 31 | 20 | 2.5333
3.4 | 34 | 23 | 2.2666
3.0 | 39 | 26 | 2.0000
budget = 416000 units = 65.00 mm
QR env scale2 = 203520 units = 31.80 mm ; scale3 = 47.70 mm
method line chars = 73

phrase+QR(scale2)    3.4mm rows=11 text=37.40mm total= 71.20mm fits=false
phrase+QR(scale2)    3.0mm rows=10 text=30.00mm total= 63.80mm fits=true
phrase, no QR        5.0mm rows=16 text=80.00mm total= 80.00mm fits=false
phrase, no QR        4.4mm rows=13 text=57.20mm total= 57.20mm fits=true
string, no QR        6.0mm rows=10 text=60.00mm total= 60.00mm fits=true
```

**Every cell of §6.5's two tables reproduces exactly**, including the corrected
5.0 mm advance (3.3333 mm, not the withdrawn 3.435) and the 1.20 mm of spare at
the one rung the worst case fits at.

- [ ] **Step 7: the tests**, `backup/hashlock_test.go`. Seven, and one of them
carries a SPEC CORRECTION:

| test | pins |
| --- | --- |
| `TestHashlockWorstCaseFitsAtExactlyOneRung` | §6.5's fit gate on the real layout: fits at 3.0 mm, at no larger rung, with `budget - total == P.F(1.2)` |
| `TestHashlockRefusesWhatItCannotLayOut` | one row over budget → `ErrHashlockTooLarge` naming `budget of 416000` |
| `TestHashlockStringFormIsLowercaseAndUngrouped` | §6.1: 6.0 mm, 4 rows, the 75 characters rejoin to the plate string VERBATIM |
| `TestHashlockBodyWrapsOnCharacters` | §6.5's wrap rule: every row but the last is a FULL line |
| `TestHashlockBandsAndBodyRows` | the three band literals inside `MaxTitleLen`; the body inside the inner margins; each band's ENGRAVING ink no taller than the lines it declares |
| `TestHashlockQREncodesTheTextNotTheString` | §6.4: the modules equal `qr_text`'s, differ from the ms1 string's, and differ from the SpaceMark-substituted phrase's — while the TEXT block does carry the mark |
| `TestHashlockRefusesAQROnTheStringForm` | decision 1, enforced rather than documented |
| `TestHashlockQRTextMatchesTheMSCorpus` | the two method-line literals against the VENDORED `qr_text` rows |
| `TestHashlockGoldens` | four plates |

The corpus lockstep is what stops this package's literals from being a
transcription — the defect the corpus exists to prevent:

```go file=fork/backup/hashlock_test.go mode=fragment
// TestHashlockQRTextMatchesTheMSCorpus is the lockstep that keeps this
// package's two method-line literals from being a transcription.
//
// §8.6's text is decided in mnemonic-secret (`ms_codec::hashlock::qr_text`) and
// pinned by the `qr_text` rows of the vendored corpus, so the plate builder
// asserts against those rows rather than against itself. Without this the
// literals above would be exactly the "a literal it transcribed itself" the
// corpus exists to prevent, and a parameter change on the Rust side would move
// the plate's QR and nothing here would notice.
//
// MUTATION: change one character of h6HardenedMethodLine -> the hardened rows
// fail with the corpus text beside the built one.
// MUTATION: build the text as "phrase: %s\nmethod: %s" -> every row fails.
```


**§11.4's method-line MUTATION AS WRITTEN CANNOT FAIL, and this plan replaces
it** (author finding, MEASURED). §11.4 says *"add one character to the method
line → the worst case no longer fits"*. At 39 characters per line a 73-character
line wraps to 2 rows anywhere from 40 to 78 characters, so 74, 75, 76, 77 and 78
ALL still fit — all five run, all five pass. §6.5 consequence 3's own number is
the real threshold and is what the test carries:
```go file=fork/backup/hashlock_test.go mode=fragment
// MUTATION: grow the method line to 79 characters -> it wraps to 3 rows at
// 3.0mm instead of 2, the body is 11 rows, and EngraveHashlock refuses with
// `11 rows at 3.0mm need 427520 units against a budget of 416000` (66.80mm
// against 65.00mm).
//
// SEVENTY-NINE, NOT SEVENTY-FOUR, and the spec's own §11.4 says "add one
// character", which CANNOT FAIL: 39 characters per line at 3.0mm means the
// line wraps to 2 rows anywhere from 40 to 78 characters, so 74, 75, ... 78 all
// still fit -- MEASURED, all four pass. The threshold is §6.5 consequence 3's
// own number ("a 79th character would make it 3"), and this row is written to
// it so the mutation is one that fires.
// MUTATION: set hashlockQRScale to 3 -> the envelope is 47.70mm, 3.0mm needs
// 510080 units (79.70mm), and every rung reds.
```

Executed at 79 characters:
`11 rows at 3.0mm need 427520 units against a budget of 416000` (66.80 mm
against 65.00 mm).

The band test measures ENGRAVING knots only, and says why a naive version could
never fail:
```go file=fork/backup/hashlock_test.go mode=fragment
		// ENGRAVING knots only. A travel move crosses both bands on its way to
		// the body on every plate, so counting moves would report the whole
		// band as inked and this assertion could never fail.
		if !k.Engrave {
			continue
		}
		p := k.Ctrl
```

- [ ] **Step 8: the second falsified shipped test.** `TestPassphraseQRTooLong`
used a 200-character passphrase as its "beyond ConstantQR's reach" case; 200
bytes at ECC-L is dim 53, which Task 4 ADMITS.
```go file=fork/backup/passphrase_test.go mode=fragment
// H6 MOVED THE CEILING AND THIS TEST MOVED WITH IT. Until H6 ConstantQR
// refused anything over v5 (dim 37) and a 200-character passphrase was over it;
// H6 §7 raised the bound to v9 (dim 53) for the hashlock phrase plate, and 200
// bytes at ECC-L is dim 53, which is now ADMITTED. The property under test is
// unchanged -- an error, never a panic, never a fall back to engrave.QR -- so
// the length moved to 240, which is dim 57 and the first version with no
// alignment-centre row and no fuzzed budget.
```

**Boundary gate (RUN):** `go test ./backup/` after `-update` for the four
goldens.

---

### Task 7: the two classes, the `phrase:` port and admission (spec §3.1, §4.1, §4.2)

**Repo:** the fork. **After Task 2** (its corpus) **and Task 5** (its
predicate). Touches `gui/sysw_admit.go`, which no other task touches.

**Files:**
- Modify: `sysw/record.go`, `sysw/composer_records.go`, `sysw/classify.go`, `sysw/composer_records_test.go`, `gui/sysw_admit.go`
- Re-vendor: `sysw/testdata/record_class_vectors.json`, `sysw/testdata/record_class_vectors.provenance.json`
- Re-pin: `hashlock/testdata/hashlock-v0.8.provenance.json` (Task 1's release)

**Interfaces:**
- Produces: `sysw.ClassPreimage`, `sysw.ClassPhrase`, `sysw.PhrasePrefix`, `sysw.HashlockMethod`, `sysw.PhraseRecord`, `sysw.ParsePhraseRecord`, `sysw.PhraseRecordString`, `isPreimagePlateRecord`.
- Consumes: `hashlock.ValidatePhrase`, `codex32.IsPreimagePlate`.
- **`isStrictMs1` is UNCHANGED.** Its final line — `return err == nil && !codex32.IsPreimage(c)` — is H0's inertness and still keeps a preimage out of every seed class. The new class is answered BEFORE it, so the two rules never overlap.

- [ ] **Step 1: the classes.**
```go file=fork/sysw/record.go mode=fragment
	// ClassPreimage is a hashlock preimage PLATE: the ms1 kind-0x03 string
	// under the id `hash` (SPEC_ms_hashlock §1 rule 2). ClassPhrase is a
	// `phrase:` record carrying a hashlock phrase and its method.
	//
	// BOTH ARE SECRET and both are BEARER: whoever holds one can spend any
	// key-less hashlock path it unlocks. IsSecret answers true for each, which
	// is what makes an unsealed payload holding one raise F1 at load and what
	// makes `me sysw pack` seal by default.
	//
	// A hashlock phrase is NOT a BIP-39 passphrase (ruling L2): interchanging
	// them opens a different wallet, so ClassPhrase is admitted at
	// progWalletPolicy alone and progPassword's row stays {ClassPassphrase}.
	ClassPreimage
	ClassPhrase
)
```
```go file=fork/sysw/record.go mode=fragment
func (c Class) IsSecret() bool {
	return c == ClassMnemonic || c == ClassCodex32Secret || c == ClassPassphrase ||
		c == ClassPreimage || c == ClassPhrase
}
```

- [ ] **Step 2: the `phrase:` port.** Rust decided it; this is the port, and it
calls `hashlock.ValidatePhrase` rather than re-spelling the rule, so the record
parser and the keyboard cannot disagree about what a phrase is.
```go file=fork/sysw/composer_records.go mode=fragment
// ParsePhraseRecord: hex of "<method>,<phrase>", cut on the FIRST comma.
//
// The order of the checks is the host's and is not decorative: hex, UTF-8,
// a comma, the method selector, then the phrase rule of SPEC_ms_hashlock §4.3
// in ITS host order (empty, printable ASCII, ms1-shaped, the 100-character cap,
// 64 hex). hashlock.ValidatePhrase is that rule and is called here rather than
// re-spelled, so the record parser and the keyboard cannot disagree about what
// a phrase is.
//
// CUT ON THE FIRST COMMA, never the last: a phrase may contain commas, and
// cutting on the last would take everything after the final comma as the phrase
// and derive a different preimage from the one the host packed.
func ParsePhraseRecord(record string) (PhraseRecord, error) {
	body, ok := strings.CutPrefix(record, PhrasePrefix)
	if !ok {
		return PhraseRecord{}, ErrPhraseRecord
	}
	b, ok := unhexLower(body)
	if !ok || !utf8.Valid(b) {
		return PhraseRecord{}, ErrPhraseRecord
	}
	methodText, phrase, hasComma := strings.Cut(string(b), ",")
	if !hasComma {
		return PhraseRecord{}, ErrPhraseRecord
	}
	var method HashlockMethod
	switch methodText {
	case "hardened":
		method = HashlockHardened
	case "sha256":
		method = HashlockSHA256
	default:
		return PhraseRecord{}, ErrPhraseRecord
	}
	if err := hashlock.ValidatePhrase([]byte(phrase)); err != nil {
		return PhraseRecord{}, ErrPhraseRecord
	}
	return PhraseRecord{Method: method, Phrase: phrase}, nil
}
```

- [ ] **Step 3: the classifier.** BEFORE `isStrictMs1`:
```go file=fork/sysw/classify.go mode=fragment
// isPreimagePlateRecord is the H6 ADMISSION shape: an engraveable ms1 string
// that is a preimage plate -- kind 0x03, 33 bytes, unshared AND under the id
// `hash` (codex32.IsPreimagePlate).
//
// THE ID IS THE WHOLE DIFFERENCE from H0's predicate, and it is here because
// H6 INVERTS the consequence of a false positive. Under H0 a false positive was
// a REFUSAL, so the wide kind-byte rule was the safe direction: "a refusal costs
// a re-encode; a wrong cut exposes a spend secret". On this path a false
// positive routes a string INTO a flow that ENGRAVES it under a band reading
// NOT A SEED, so a plain BIP-93 33-byte secret beginning 0x03 -- roughly 1 in
// 256 of them -- must not arrive. A kind-0x03 single under any other id stays
// ClassUnknown and inert, and the host names it in its refusal.
func isPreimagePlateRecord(record string) bool {
	if len(record) > MaxEngraveableMs1Len {
		return false
	}
	if !strings.HasPrefix(strings.ToLower(record), "ms1") {
		return false
	}
	c, err := codex32.New(record)
	return err == nil && codex32.IsPreimagePlate(c)
}
```

- [ ] **Step 4: admission, at ONE program.**
```go file=fork/gui/sysw_admit.go mode=fragment
		// H6: a hashlock PREIMAGE plate and a `phrase:` record, admitted at
		// this program and AT NO OTHER ROW, exactly as Key, Hash and Now are.
		//
		// A HASHLOCK PHRASE IS NOT A BIP-39 PASSPHRASE and is never admitted at
		// progPassword. The terminology ruling L2 exists because interchanging
		// them opens a different wallet, so progPassword's row stays
		// {ClassPassphrase} and the operator who opens the program whose NAME
		// matches what they are holding gets §8.8's notice instead of a
		// keyboard that would quietly accept the wrong secret.
		sysw.ClassPreimage: true,
		sysw.ClassPhrase:   true,
```

- [ ] **Step 5: re-vendor and re-pin.** Copy the regenerated
`record_class_vectors.json` from Task 2, set the provenance pin's `commit`,
`file_commit`, `sha256` (`3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf`)
and `vectors` (68), and teach the lockstep test the fourth class:

```go file=fork/sysw/composer_records_test.go mode=fragment
	// H6: the fourth composer prefix. ClassPreimage is NOT here -- a preimage
	// plate carries no prefix and is placed by classifyConstellation, so it has
	// no CASES row in this corpus and is pinned by the seam corpus instead.
	"Phrase": ClassPhrase,
```

and add `Phrase` to the non-vacuity sweep, so a corpus that lost the rows would
red rather than pass vacuously:

```go file=fork/sysw/composer_records_test.go mode=fragment
	for _, cls := range []string{"Key", "Hash", "Now", "Phrase", "Unknown"} {
```

- [ ] **Step 6: re-vendor the ms corpus too.** Task 1 changed
`crates/ms-codec/tests/vectors/hashlock-v0.8.json` (the seven `qr_text` rows), so
the fork's vendored copy at `hashlock/testdata/hashlock-v0.8.json` and its
provenance pin are stale the moment Task 1 lands. Copy it, set
`hashlock/hashlock_test.go`'s `corpusSHA256` to
`4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4`, and record the
ms release commit in `hashlock/testdata/hashlock-v0.8.provenance.json`. Task 6's
`TestHashlockQRTextMatchesTheMSCorpus` is what CONSUMES those rows, so this step
is what makes that lockstep mean anything.

**Boundary gate (RUN):** `go test ./sysw/ ./hashlock/` — **ok**. That run is the
cross-language conformance result this task exists for: **68 of 68 rows,
including all 21 `phrase:` rows, classify identically on the host and on the
device.**

---

### Task 8a: retention — `hashlockHeld`, and the scrub in the EXISTING defer (spec §2.2)

**Repo:** the fork. **Group E, first of three, ONE implementer** — 8a, 8b and 9
all touch `gui/composer_flow.go`, and 8b and 9 both touch `gui/composer_hash.go`.

**Files:**
- Modify: `gui/composer_state.go`, `gui/composer_flow.go`
- Create: `gui/composer_hashlock_held_test.go`

**Interfaces:**
- Produces: `composerState.hashlockHeld`, `hashlockMaterial`, `hashlockProvenance` (`hashlockFromPhrase` | `hashlockFromPayload`), `composerHoldHashlockMaterial`, `composerScrubHashlockHeld`.
- Consumes: `wipeBytes` (`gui/slip39_polish.go:344`), `hashlockMethod` (`gui/composer_hashlock.go:29-34`).
- Unchanged: `phraseDigests` and `composerNotePhraseDigest` — the new map is keyed the same way and allocates the same way, and H5 §2's C16 reasoning carries over unchanged.

- [ ] **Step 1: the field, and the three rules its comment makes normative.**
```go file=fork/gui/composer_state.go mode=fragment
	// hashlockHeld is the material THIS COMPOSITION may cut onto a preimage
	// plate, keyed by digest exactly as phraseDigests is (H6 §2.2). H5 §2's
	// C16 reasoning applies unchanged: "Remove path" splices the slice, so an
	// index is not an identity.
	//
	// H6 REVERSES THREE OF RULING L7's FOUR VERBS. L7 scoped the device to the
	// digest alone -- "It never stores, shows, engraves or sources a preimage"
	// -- and H2 implemented it literally: hashlockPhraseRoute derived X on the
	// stack and dropped it when the function returned. H6 lifts store, show and
	// engrave, and leaves the fourth (SOURCE: reading a preimage plate back
	// into a seed flow) refused.
	//
	// ONE INSERTION SITE, composerHoldHashlockMaterial, which ALLOCATES when
	// the map is nil. composerState is built at its one production site
	// (composerFlow) as a struct literal setting two fields, and the same way
	// in every test, so this map arrives NIL and an assignment into a nil map
	// panics -- in the GUI goroutine, at the moment the operator holds to
	// confirm a hash that gates funds. That is composerNotePhraseDigest's rule
	// applied to the second map, and it is normative for the same demonstrated
	// reason.
	//
	// NOTHING DELETES. A digest no path carries is not removed; it is REPORTED
	// at Done ("not on any path, will not be cut"). The scrub is the flow-exit
	// defer, composerFlowExit, and it goes IN that defer rather than beside it:
	// a second defer costs 96 B of firmware flash because TinyGo removes the
	// empty stub's CALL and not the defer bookkeeping around it.
	hashlockHeld map[[32]byte]hashlockMaterial
```

- [ ] **Step 2: the value, the provenance and the ONE insertion site.**
Provenance is a RECORDED FACT rather than a runtime test, and §5.1 is what makes
that possible: the two carriers run different code paths, so §10's copy reads
which path ran.
```go file=fork/gui/composer_state.go mode=fragment
// hashlockProvenance records WHICH PATH RAN, not a runtime test: a phrase
// typed on this device takes hashlockPhraseRoute, and a payload preimage or
// `phrase:` record takes §5.1's derive-only sibling. The §10 copy that has to
// tell the two apart therefore reads a recorded fact rather than re-deriving
// one.
type hashlockProvenance int

const (
	// hashlockFromPhrase: typed here, on this device, in this composition.
	hashlockFromPhrase hashlockProvenance = iota
	// hashlockFromPayload: delivered in the loaded payload.
	hashlockFromPayload
)

// hashlockMaterial is what one held digest may be cut as (H6 §2.2, §5.3).
//
// Phrase and Method are EMPTY for a payload-delivered preimage record: that
// carrier holds X and H and no phrase at all, which is why §5.3's pick step
// offers the phrase forms only when Phrase is non-empty.
type hashlockMaterial struct {
	phrase     []byte
	method     hashlockMethod
	preimage   [32]byte
	provenance hashlockProvenance
}

// composerHoldHashlockMaterial is the ONE insertion site, and it ALLOCATES.
//
// MUTATION-RELEVANT: assigning straight into st.hashlockHeld panics on the
// zero-value state composerFlow builds, which is every production run's first
// HOLD.
func composerHoldHashlockMaterial(st *composerState, h [32]byte, m hashlockMaterial) {
	if st.hashlockHeld == nil {
		st.hashlockHeld = make(map[[32]byte]hashlockMaterial)
	}
	st.hashlockHeld[h] = m
}
```

- [ ] **Step 3: the scrub, and it goes IN the existing defer.**
```go file=fork/gui/composer_state.go mode=fragment
// composerScrubHashlockHeld wipes every held phrase and zeroes every preimage.
//
// It is called from composerFlowExit -- the EXISTING defer -- so every exit
// (a Back, a refusal, a ctx.Done unwind, a panic) is covered by construction.
//
// The map ENTRIES are left in place with their secrets zeroed rather than
// deleted: a value type cannot be wiped through a map, so each entry is read
// out, wiped, and written back. F-483 already records that the typed phrase
// also lives in kbd.Fragment, an immutable Go string, before anything here
// stores it; that is non-gating by the 2026-08-27 ruling and is a follow-up.
func composerScrubHashlockHeld(st *composerState) {
	for h, m := range st.hashlockHeld {
		wipeBytes(m.phrase)
		m.phrase = nil
		m.preimage = [32]byte{}
		st.hashlockHeld[h] = m
	}
}
```
```go file=fork/gui/composer_flow.go mode=fragment
func composerFlowExit(st *composerState) {
	st.reg.scrub()
	// H6 §2.2 item 3. IN this defer, not beside it: a second `defer` costs 96 B
	// of flash for the reason composerFlow's own comment records.
	composerScrubHashlockHeld(st)
	clearComposerStateHook()
}
```

- [ ] **Step 4: the tests.** Three, in `gui/composer_hashlock_held_test.go`, and
the helper is the whole design: it builds `composerState` EXACTLY as
`composerFlow` does and asserts the map arrived nil, because a helper that
pre-allocated would hide the defect on the machine.
```go file=fork/gui/composer_hashlock_held_test.go mode=fragment
func composerH6ZeroState(t *testing.T, paths int) *composerState {
	t.Helper()
	st := &composerState{reg: &seedRegistry{}, bound: composerBoundFrom(nil)}
	if st.hashlockHeld != nil {
		t.Fatal("this helper exists to reproduce the NIL map composerFlow leaves; it is not nil")
	}
	st.list = md.PathList{Wrapper: md.ComposeWsh, Paths: make([]md.SpendPath, paths)}
	return st
}
```

The scrub test is driven through `composerFlowExit` — the FUNCTION
`composerFlow` defers — so a scrub that existed but was never wired in would
still red; and it asserts BOTH the phrase and the preimage, because a `[]byte`'s
backing store is shared while a `[32]byte` in a map is a value:
```go file=fork/gui/composer_hashlock_held_test.go mode=fragment
// MUTATION: remove the composerScrubHashlockHeld call from composerFlowExit ->
// the phrase is still readable after exit and both assertions below fail.
// MUTATION: range over the map and wipe `m` without writing it back -> the
// PREIMAGE assertion fails (the byte array is a value in the map), while the
// phrase one still passes because a []byte's backing store is shared -- which
// is why both are asserted.
```

and the third asserts on the AST, because there is nothing at runtime to
observe: two defers and one defer behave identically and differ only in 96 bytes
of flash.

**MUTATIONS, all three executed:**
- assign without the nil check → `panic: assignment to entry in nil map`
- remove the scrub from `composerFlowExit` →
  `the phrase survived the flow-exit defer: "correct horse battery staple"` and
  `the preimage survived the flow-exit defer: abcdef00…`
- wipe without writing the value back → the PREIMAGE assertion fails while the
  phrase one still passes, which is why both exist

**Boundary gate (RUN):** `go test -run 'TestComposerHoldsHashlockMaterial|TestComposerFlowExitScrubs|TestComposerScrubIsInThe' ./gui/` — **ok**.

**BUT THE WHOLE-PACKAGE GATE REDS, and that is a finding rather than a defect —
8a MAY NOT LAND ALONE.** `scripts/gui-shard-test.sh ./gui/ 24` over the wired
tree reports, from shard 18:

```
--- FAIL: TestComposerEveryScreenFunctionHasAProductionCaller (0.03s)
    composer_join_test.go:104: these composer functions have no production caller, so the screens they
        draw cannot be reached by any operator: [composerHoldHashlockMaterial]
```

That guard is `gui/composer_join_test.go`, and it is the gate the composer cycle
added after two R0 lenses found **fourteen production functions at once with a
green suite, because every test called them directly** — the "plans list
components and omit the call that joins them" class. `composerHoldHashlockMaterial`
is called by Task 8b's `hashlockPayloadRoute` and by Task 8b's HOLD, so:

> **Tasks 8a and 8b land in ONE commit.** Splitting them leaves an inserted
> production function with no production caller, and the fork's own join guard
> reds — correctly. The exemption table at `gui/composer_join_test.go:87-90`
> exists for functions whose consumer is DEFERRED with a follow-up number
> (`composerDescriptorPlateFits` → F-457), and it is not the right instrument
> here: 8b's consumer is in this stage, not a future one. **Adding a name to
> that table to make 8a green alone would be quieting the one gate that catches
> this defect class.**

---

### Task 8b: `Which hash?` gains two bands, and the derive-only path (spec §5.1)

**Repo:** the fork. **Group E, after 8a.** **SPECIFIED, NOT WIRED** by the plan
author — the blocks below carry no `file=` header and the checker does not check
them; see `## Build gate`.

**Files:**
- Modify: `gui/composer_hash.go`, `gui/composer_hashlock.go`
- Modify (tests): `gui/composer_hash_test.go`, `gui/composer_hashlock_test.go`

**Interfaces:**
- Produces: `composerHashPreimageRow(i int, d [32]byte) string`, `composerHashPhraseRow(i int, d *[32]byte) string`, `composerHashInPayloadRow(i int, d [32]byte) string`, `hashlockPayloadRoute(ctx, th, st, idx int, rec sysw.Record) hashlockOutcome`.
- Consumes: `composerHashRows` (`gui/composer_hash.go:157-175`), `composerHashEdit`'s label-keyed switch (`:184-224`), `hashlockDeriveFlow`, `composerCopyHashlockConfirm`, `hashlockRelationLine`, `hashlockOtherPathLine`, `composerHoldHashlockMaterial`.
- **`hashlockPhraseRoute` is UNCHANGED** and a payload phrase never enters it.

- [ ] **Step 1: the six bands, in order.** `composerHashRows` builds the row set
once and records each named band's index; `composerHashEdit`'s switch dispatches
on those NAMES and its `default` PANICS rather than assigns (H2 §5, r2 C-4).

1. the payload's `hash:` digests — `hash <i>  <first8>..<last8>`
2. **the payload's preimage records** — `preimage <i>  <first8>..<last8>`
3. **the payload's `phrase:` records** — `phrase record <i> (derive to see the digest)`
   before derivation, `phrase <i>  <first8>..<last8>` after
4. `Type a hashlock phrase`  5. `Type 64 hex`  6. `No hash lock`

MEASURED at `sh2DisplaySize`: every row draws on ONE line in
`composerPageLines`' 411 px band (23 px per row), `preimage 10  b867db87..edbc96cb`
and `phrase record 10 (derive to see the digest)` included.
`composerPickScreenMaxRows` (`gui/composer_paged.go:243`) is checked against the
longest row set.

- [ ] **Step 2: the `(in payload)` annotation.** When a `hash:` digest and a
preimage or `phrase:` record in the same payload carry the SAME digest, bands 1
and 2 draw identical digest text, adjacent, differing by one word, with
different consequences — band 1 assigns the digest and holds NO material, so no
plate is offered at Done; band 2 cuts a plate. Band 1's row becomes
`hash <i>  <first8>..<last8>  (in payload)`.

MEASURED at `sh2DisplaySize`, longest two-digit form: **41 characters, 343 px,
ONE line.** The alternative wording `(preimage in payload)` is 50 characters and
**348 px over two lines**, which a picker band cannot spend.

- [ ] **Step 3: DERIVATION IS LAZY, and the payload path is a DIFFERENT
FUNCTION.** A `phrase:` row derives on PICK, behind the existing `Deriving`
countdown (H2 §4.4), and the result is entered into `hashlockHeld` with
`provenance = hashlockFromPayload`.

`hashlockPhraseRoute` (`gui/composer_hashlock.go:43-86`) does BOTH of the things
a payload phrase must not do: it calls `hashlockMethodPick` (`:53`) and it ends
on `composerCopyHashlockReconcile` (`:83`). The method is the RECORD's, never a
pick — so J4-1's mistake cannot occur — and the reconcile screen's instruction
("run `ms hashlock` with this phrase") is a no-op for a phrase the host already
has. So the payload path is a DERIVE-ONLY sibling:

```
func hashlockPayloadRoute(ctx *Context, th *Colors, st *composerState, idx int, rec sysw.PhraseRecord) hashlockOutcome {
        // no phrase screen, no method pick, no reconcile screen
        x, ok := hashlockDeriveFlow(ctx, th, rec.Phrase, rec.Method)   // the countdown
        if !ok { return hashlockBackToWhichHash }
        h := hashlock.Digest(&x)
        if !composerCopyHashlockConfirm(ctx, th, st, idx, h, ...) { return hashlockBackToWhichHash }
        st.list.Paths[idx].Hash = &h
        composerHoldHashlockMaterial(st, h, hashlockMaterial{
                phrase: []byte(rec.Phrase), method: rec.Method,
                preimage: x, provenance: hashlockFromPayload,
        })
        return hashlockAssigned
}
```

A preimage RECORD takes the same shape without the KDF: `DecodeMS1Preimage`
gives X directly, so there is no countdown, `phrase` stays nil, and the pick
step will offer only `preimage string` for it.

**Why this matters beyond convenience:** it is what makes §10.2 true BY
CONSTRUCTION. `composerCopyHashlockReconcile` has exactly ONE call site in the
whole tree, inside `hashlockPhraseRoute`, so a payload phrase can never reach
it — no runtime guard, and no test, because the guard the first draft proposed
would have been dead code.

- [ ] **Step 4: `taking`** (`gui/composer_hash.go:194`), which fires the §8i rule
modal, extends to the two new bands.

- [ ] **Step 5: the tests** (§11.5). By LABEL, with 0, 1 and 2 records of each
new class; every row does what its label says; `Type 64 hex` never clears the
lock (H2's C-4 regression test).
**MUTATION: reintroduce an index-keyed `default` that assigns** → the
displaced-row rows fail.
**MUTATION: derive at row-build time** → a timing assertion on the screen's
first frame fails.
**MUTATION: drop the `(in payload)` annotation** → the both-present row shows
two adjacent rows with identical digest text and different consequences.
**MUTATION: route a payload phrase through `hashlockPhraseRoute`** → the method
PICK appears for a record that names its own method, and the reconcile
screen — whose instruction is a no-op here — is drawn.

**Boundary gate:** `scripts/gui-shard-test.sh ./gui/ 24`.

---

### Task 9: the Done review — a PICK step, the census, cut order and both abort arms (spec §5.3, §5.4, §8.3, §8.4, §10.1, §10.3)

**Repo:** the fork. **Group E, after 8b.** **SPECIFIED, NOT WIRED.**

**Files:**
- Modify: `gui/composer_flow.go` (`composerEngraveStep`, `:335-394`), `gui/composer_census.go`, `gui/composer_copy.go`, `gui/composer_copy_test.go`, `gui/modal_fits_test.go`
- Create: `gui/composer_preimage_plate.go`, `gui/composer_preimage_plate_test.go`

**Interfaces:**
- Produces: `composerPreimagePlatePick(ctx, th, st, h) hashlockPlateChoice`, `composerPreimagePlates(st) []hashlockPlate`, `composerAbortNoPreimage`, `composerAbortPreimageCut`, `composerCopyHashEveryPathHeld`, `composerCopyHashEveryPathHeldPhrase`, `composerBuildHashlockPlate(st, h, choice) backup.Hashlock`.
- **`composerCensusLines`'s SIGNATURE CHANGES** (§5.3 item 2 / r0 fidelity N-3): today `composerCensusLines(params engrave.Params, cards []bundleCard)`, which can see neither step (A)'s decisions nor `hashlockHeld`. It gains the ACCEPTED-PLATE LIST as a third parameter — a VALUE computed by step (A) — rather than reading state, so the census reports a decision instead of recomputing one.
- **`bundleAbortWarningText` is UNCHANGED** (`gui/bundle_flow.go:780-792`), and that is normative: it is reachable only from `bundleAbortWarning` at `:625` and `:640`, both INSIDE `bundleEngrave`, and §5.4 cuts every accepted preimage plate BEFORE calling it — so a clause added there could never fire. It also could not be plumbed: the shipped comment at `:610-616` prohibits the variadic tail by name.
- **`buildPlateCensusLines`' count and its "a set is only a backup when all of it exists" claim (`gui/multisig_build_census.go:63-73`) stay BYTE-UNCHANGED**, because a preimage plate is not a `bundleCard` and does not enter `plan` — S6b's passphrase-plate precedent (`:88-93`: entering `plan` *"would tell a reader it travels WITH the set"*).

- [ ] **Step 1: the PICK step (A).** The per-plate decisions do NOT live on
`confirmReviewScreen` and the first draft's claim that they did was
unimplementable: that screen (`gui/multisig_build.go:1895`) is paged and
READ-ONLY, every input is already bound (`backBtn = Button1`,
`contBtn = Button3 + Center`, `pageBtn = Button2`), its body lines are
`widget.Labelw` ops and NOT `Clickable`s, and it is SHARED with the Policy
Review. W-2 measured exactly this failure once already: **205 taps that moved
nothing**, on a screen whose rows were not targets.

So step (A) is one `composerPickScreen` (`gui/composer_paged.go:278`) per held
digest, before the census. That primitive already has what this needs and nothing
else does: a tap on a row selects it, Button3 takes the highlighted row, Button1
declines, Button2 pages, and `composerPickScreenMaxRows = 24` bounds the hit
areas. Rows, MEASURED at `sh2DisplaySize` in the 411 px band, each ONE line:

| row | chars | px |
| --- | --- | --- |
| `preimage string` | 15 | 128 |
| `phrase + method` | 15 | 138 |
| `phrase + method + QR` | 20 | 180 |
| `do not cut this preimage` | 24 | 196 |

The QR rows are offered ONLY when the device holds the phrase
(`len(m.phrase) > 0`, decision 1), and taking one fires §8.5's warning.
**Back contract:** Button1 is `do not cut` for the highlighted plate, matching
`composerPickScreen`'s shipped decline arm; backing out of the STEP returns to
the engrave step's entry and thence round `composerFlow`'s loop with the state
intact, which is §2.2 item 4 unchanged.

**A phrase is MASKED and NOT revealed here** (§5.3 item 7). The pick screen and
the census print `phrase: <n> characters` and the method, and show no characters
at all. The first draft's "the affordance the operator already met on the phrase
keyboard … revealed only while the toggle is held" is wrong twice: the shipped
affordance is `{label: "show", action: ppReveal}`
(`gui/passphrase_keyboard.go:141`), a KEY ON A KEYBOARD GRID, and it LATCHES
(`k.revealed = !k.revealed`, `:221`). Neither a pick screen nor a paged confirm
has a key grid to put it on, and **no new control is invented for a screen with
no spare input.** This applies to BOTH provenances: a payload-delivered phrase
is a secret the operator never typed, may not own, and did not ask to see, in
whatever room the machine lives in.

- [ ] **Step 2: the census (B).** `confirmReviewScreen(ctx, th, "Plates To Cut",
composerCensusLines(...))` (`gui/composer_flow.go:389-390`), REPORTING the
choices already made and staying read-only. `composerCensusLines`
(`gui/composer_census.go:86`) appends §8.3's block: a heading, one row per
plate, and the apart-storage line.

**Drawn through `composerPageLines`' 411 px band, NOT `confirmReviewScreen`'s
own wrap.** MEASURED at `sh2DisplaySize`: `confirmReviewScreen` wraps at
`dims.X - 2*8 = 464` px and centres on the whole panel, while the navigation
column starts at 427 px, so any row wider than **374 px** has its right edge
under a button. Every new row is over it — 405, 423, 441, 447, 448 px — and so
is the SHIPPED completeness line at **459 px**, which is why this is a
PRE-EXISTING property rather than something H6 invents. H6 still may not add five
more rows to it: that is the W-3 class the composer's paged screens were rebuilt
to remove.

§8.3's rows, verbatim:

```
Plus {n} preimage plate(s), cut first and NOT part of this backup:
path {n}  {first8}..{last8}  {phrase, hardened, QR | phrase, sha256 | preimage string}
preimage {first8}..{last8}: not on any path, will not be cut
preimage {first8}..{last8}: declined, will not be cut
Keep each preimage plate apart from the policy plates and from the others.
```

and the stand-alone notice form, for a review whose only entry is an unused
preimage, MEASURED **107 drawn / headroom 455**:

```
One preimage this composition holds is on no path of this policy. It will not be cut. Go back and set a path's hash to it, or leave it.
```

**A retained preimage no current path carries is LISTED, never cut**, for BOTH
provenances, and **nothing is deleted from `hashlockHeld` to achieve it**
(§2.2 item 2). **A declined plate is dropped from the plan and NAMED**;
declining does NOT abort the run, because a preimage plate is not part of the
policy set and `bundleEngrave`'s set-level "a partial bundle can't be used"
reasoning (`gui/bundle_flow.go:625-631`) does not reach it. **No cap** on the
number of plates.

- [ ] **Step 3: cut order — preimage plates FIRST.**

```
for _, pl := range plates {           // §6, one Plate per accepted preimage
        if !NewEngraveScreen(ctx, pl).Engrave(ctx, &engraveTheme) {
                return composerAbortNoPreimage(ctx, th, st)   // §8.4a
        }
}
return bundleEngrave(ctx, th, "Wallet Policy", cards, markTitle, "") == bundleEngraveDone
```

`NewEngraveScreen` is `gui/gui.go:3296`; the plate is built by Task 6's
`backup.EngraveHashlock` and `toPlate` (`gui/gui.go:3620`), on the
`ppBuildPlate` pattern (`gui/passphrase_flow.go:570-588`). Ordering removes the
window in which the md1 plates exist and the preimage does not. The ms1 secret
cards keep their place at the head of `cards` (`gui/composer_flow.go:381`), so
the whole run is secrets-then-policy.

- [ ] **Step 4: BOTH abort arms, on the COMPOSER's own screens.** The order
creates TWO distinct windows, not one:

| window | what the operator holds | arm |
| --- | --- | --- |
| a preimage plate's own `Engrave` returns false, none cut yet | no plate of any kind; the phrase held only by this composition | §8.4a |
| at least one preimage plate cut, then the run ends | a BEARER plate on the bench and no usable policy set | §8.4b |

The second is the one this ORDER creates, and its danger is specific:
`bundleEngrave`'s set-level copy tells the operator a partial bundle cannot be
used, whose natural response is to run the composition again; §5.3 sets no cap,
so the second run cuts a SECOND bearer plate for the same secret, one of which
the operator has no record of, while §8.3's own line tells them to store them
apart from each other.

```
NO PREIMAGE PLATE WAS CUT. The phrase dies with this composition. Do not fund this wallet.
```
MEASURED on `errorScreenBody`: **75 drawn / headroom 476.**

```
A PREIMAGE PLATE WAS CUT and no policy plate was. Store or destroy it now; do not leave it with the blanks.
```
MEASURED on `errorScreenBody`: **86 drawn / headroom 476.**

**It says "dies with this composition", not "is now gone"**, and that is a
correction rather than a style: an abort inside `bundleEngrave` returns
`bundleEngraveAborted`, `composerEngraveStep` returns false, and `composerFlow`
loops back with the state intact (`gui/composer_flow.go:47-131`) — the phrase is
still held. Saying it is gone would be false on the screen whose job is to stop
a funding decision.

- [ ] **Step 5: §8.5's QR warning**, confirm-to-proceed on the model of
`ftWarnQR` (`gui/freetext_flow.go:1214-1216`), which already warns for strictly
less dangerous content:

```
The QR makes the phrase readable by any camera. A photograph of the plate is a copy of the phrase, and the phrase spends this path.
```
MEASURED through `confirmWarningBody` wrapped in `composerConfirmBody`:
**126 drawn / headroom 378.**

- [ ] **Step 6: §10.1's third §8h arm.** `composerCopyHashEveryPathFor`
(`gui/composer_copy.go:527-532`) gains a third arm, chosen when EVERY hashed
path's digest has material in `hashlockHeld` this run will cut, and a fourth
when at least one of those came from a phrase typed here. **Both are in the
PRESENT TENSE of what is HELD, not the future tense of what will be cut** — the
banner's one call site (`gui/composer_shape.go:442-443`) runs at
`composerFlow`'s line 75, while the accept/decline happens at line 125, so an
operator who read "this run cuts a plate for each one" and then declined one
would be left with a backup instruction that was false.

```
HASH ON EVERY PATH
Every way to spend this wallet needs the preimage of a hash. This composition holds the preimage for each one and can cut a plate for it at Done. Store those plates apart from these, and apart from each other.
```
```
HASH ON EVERY PATH
Every way to spend this wallet needs a hashlock preimage. This composition holds the phrase and method for each one and can cut a plate at Done. Store those plates apart from these, and apart from each other.
```
MEASURED on `errorScreenBody`: **185 drawn / headroom 360** and **186 / 360.**

**Where the load actually falls is NOT here.** The call site is guarded by
`composerEveryPathHashed` (`gui/composer_state.go:257-259`), which is false the
moment ONE path is keyed — i.e. on the ordinary mixed hashlock wallet. **On a
mixed policy it is §8.3's census block at Done that tells the operator a
preimage plate is being cut**, for every composition, and that is why §8.3 is
not optional.

- [ ] **Step 7: `PREIMAGE REQUIRED` on the md1 AND mk1 plates** (§10.3).
`composerEngraveStep` passes a marking to `bundleEngrave`
(`gui/composer_flow.go:393`, today `"", ""`): `markTitle = "PREIMAGE REQUIRED"`
when any path carries a hash, `""` otherwise; the footer stays empty. This is
`singleSigPlateMark`'s mechanism unchanged (`gui/singlesig.go:365-374`, whose
`"PASSWORD REQUIRED"` is also 17 characters against `MaxTitleLen = 18`).

`bundlePlateMark` (`gui/bundle_flow.go:573-578`) excludes exactly one kind —
`cardMS1` — and the template form's card list is `cardMD1` **plus** the minted
`cardMK1` key cards (`gui/composer_cards.go:69-74`), so a run-level `markTitle`
stamps every mk1 key card too. **That is taken deliberately**: it is
`singleSigPlateMark`'s own precedent, and it is what F-132 asks for — a cosigner
holding a key card is exactly the year-later reader who otherwise has nothing
telling them a preimage exists. The alternative, marking per card kind, would
suppress the message on the only artifact that travels.

- [ ] **Step 8: the tests** (§11.5), and three of them are written against
failures the first draft's versions could not detect:

| test | MUTATION that must red it |
| --- | --- |
| the review is two steps; the pick screen offers the four rows BY LABEL; QR rows only when the phrase is held | put the form/QR/decline controls on `confirmReviewScreen` → **there is no control left to bind and the test cannot drive them, which is the point**; offer a QR row with no phrase held → the row assertion fails |
| the census is READ-ONLY and REPORTS the choice | drop the declined plate silently → the census row assertion fails |
| the census block is drawn through `composerPageLines`' band | draw it through `confirmReviewScreen`'s own wrap → `inkUnderNavOps` finds the 405-448 px rows under the column |
| the plate is not a `bundleCard` | add it as a card → `bundlePlatePlan`'s count and the "all of it exists" claim change |
| cut order | move the loop after `bundleEngrave` → an order assertion on the engrave hook fails |
| BOTH abort arms; NEITHER on a completed run | fire either unconditionally → the success row fails; attach either to `bundleAbortWarningText` → the arm is unreachable and the §8.4b test fails |
| §10.1's third form fires on its predicate and **does not claim a plate was cut** | restore the future-tense wording → the DECLINE row asserts a body that is false about what the run did |
| `PREIMAGE REQUIRED` marks md1 **and the mk1 key cards**, never `cardMS1` | mark unconditionally → the unhashed row fails; **suppress on `cardMK1` → the key-card row fails**. Without the mk1 assertion every assertion in this row passes whether or not the key cards are marked, and the row would prove nothing about §10.3's sentence |

- [ ] **Step 9: every new DEVICE body gets a `modal_fits_test.go` row**, measured
ALONE as it is drawn, and **the ASCII assertion is mechanical** (`r <=
unicode.MaxASCII` over the table), not by inspection.
**MUTATION: put a `§` or an em dash in any body** → `assertModalBodyFits`
reports a near-blank frame (5,004 ink pixels against a 6,000 floor) rather than
a truncation, and the ASCII assertion names the character. **This mutation must
be in the suite**: the spec's first draft carried non-ASCII in five bodies and
claimed a measurement for one that the harness cannot produce.

**Boundary gate:** `scripts/gui-shard-test.sh ./gui/ 24`; firmware size.

---

### Task 10: the Hashlock plates flow, under the Wallet Policy door (spec §5.2, §6.3)

**Repo:** the fork. **Group F, after Tasks 6 and 7.** **SPECIFIED, NOT WIRED.**

**Files:**
- Create: `gui/composer_hashlock_plates.go`, `gui/composer_hashlock_plates_test.go`
- Modify: `gui/composer_door.go`

**Interfaces:**
- Produces: `composerHashlockPlatesFlow(ctx, th)`, `composerDoorHasPreimage(ctx) bool`, `hashlockPlateLocator(...) []string`.
- Consumes: `ctx.sysw` only. **It builds no composition and reads nothing from `composerState`** (decision 4: it cuts "without a composition").

- [ ] **Step 1: the fourth route, CONDITIONAL.** After "Scan cards", "From
payload" and "Build a new policy" (`composerDoorFlow`,
`gui/composer_door.go:98-119`), gated by a predicate of the same shape as
`composerDoorHasConsumablePolicy` (`:86-90`): offered only when the loaded
payload holds at least one `ClassPreimage` or `ClassPhrase` record. **A door row
that names a route it cannot take is the F-437 defect the door exists to
remove.**

- [ ] **Step 2: the door's LEAD names these records.** `composerDoorCounts`
(`gui/composer_door.go:37-52`) counts `ClassKey`, `ClassMnemonic`/
`ClassCodex32Secret` and `ClassUnknown` and nothing else, so `composerDoorLines`'
`default` arm draws `composerCopyNoKeys()` — a lead saying there are no keys,
above a door offering the Hashlock plates route **for exactly that payload**. It
gains a preimage/phrase count so the lead and the routes agree. This is the
screen that tells the operator the tap worked, and §12 item 3's payload is the
one that reads wrongest.

- [ ] **Step 3: the flow.** List the payload's preimage and phrase records; the
operator picks; per record it offers form and QR (Task 9's pick rows); then it
cuts.

**A `phrase:` record DERIVES ON PICK and the result lives for the flow —
NORMATIVE, in four parts:**
1. **The LIST draws without deriving.** A phrase row reads
   `phrase record <i> (derive to see the digest)`. Deriving to draw the list is
   the *"three records would be a 30 s stall before a list could be drawn"*
   §5.1 rejects, and this flow has MORE phrase records than the composer
   typically does, not fewer.
2. **Picking one derives ONCE, behind the `Deriving` countdown** — never
   silently, because a 10 s stall with no screen is a hang to the operator.
3. **The result lives in a LOCAL of the flow**, keyed by record index, scrubbed
   on return. NOT `composerState`: §2.3's "holds nothing" stays true, and a
   redraw, a page or a Back must not re-run the KDF.
4. **The locator's `hash` row is printed from that result**, so it is never
   blank. **A phrase-form plate with no locator at all — a bearer phrase in
   plain text with no digest, no path and no payload position — is the worst
   artifact this stage can cut**, and it is what a literal reading of the first
   draft produced.

- [ ] **Step 4: the locator (§6.3), pre-formatted by this flow.** Body rows, in
order, immediately after the method line (phrase form) or the title (string
form):

```
path <n>                            (composer-native only)
hash  <first8>..<last8>             (always)
mk1 stub (policy): <8 hex>          (when every slot is seated)
mk1 stub (template): <8 hex>        (otherwise, and when no policy id exists)
matches hash <i> in the payload     (Hashlock plates flow, when one matches)
```

The stub labels are `gui/composer_stub.go:54,67`'s literals, so the plate and
the screen the operator copied into their notebook use the same words. In THIS
flow the `mk1 stub` is printed **only when it can be computed from an md1 record
in the same payload** — no payload record carries an id, so an md1 is the only
source — and the field is omitted otherwise.

MEASURED against the 32-character cap at 3.0 mm: the rows are 6, 24, 27, 29 and
30 characters and **every one of them fits a band**. They are body rows anyway,
because **a band holds at most TWO LINES and this stage has already spent both**
(`backup/passphrase.go:250-253`: *"a band offers innerMargin 10 - outerMargin 3
= 7mm, and three 3mm lines need 9mm and run off the plate edge"*).

- [ ] **Step 5: AN ABORT IN THIS FLOW DRAWS NEITHER §8.4 ARM, and must not
borrow one.** §8.4's arms say the phrase *"dies with this composition"* and speak
about a run that also cuts policy plates. **Neither is true here**: this flow
builds no composition, and the material it cuts stays in the payload, in flash.
Telling the operator a secret is gone when it is still in flash is false in the
dangerous direction. Reusing the arm is the obvious implementation, which is why
it is refused by name.

- [ ] **Step 6: the tests.**
**MUTATION: derive per frame** → the once-per-pick assertion fails.
**MUTATION: omit the locator's `hash` row for a phrase record** → the non-empty
assertion fails, **which is the row standing between the operator and a bearer
plate with no locator at all**.
**MUTATION: reuse §8.4a here** → an assertion that this flow's abort draws no
"dies with this composition" body fails, because the material is still in flash.
**MUTATION: offer the route when the payload holds neither class** → the door
predicate row fails.

**Boundary gate:** `scripts/gui-shard-test.sh ./gui/ 24`.

---

### Task 11: the free-text and passphrase warning, and the Password-program notice (spec §9, §8.8)

**Repo:** the fork. **Group G — disjoint from every other group.**
**SPECIFIED, NOT WIRED.**

**Files:**
- Modify: `gui/freetext_flow.go`, `gui/passphrase_flow.go`, `gui/sysw_session.go`, `gui/modal_fits_test.go`
- Modify (tests): `gui/freetext_test.go`, `gui/passphrase_test.go`, `gui/sysw_session_test.go`

- [ ] **Step 1: §9's warning. NEVER A REFUSAL** — both programs cut as typed.
`engraveTextFlow` and `engravePassphraseFlow` gain a confirm-to-proceed at OK on
the TEXT-ENTRY screen, earlier than the confirm summary, which is already paged
(`ftConfirmFlow`, `gui/freetext_flow.go:1362-1402`; `ppConfirmFlow`,
`gui/passphrase_flow.go:497`). It fires once per composition, re-armed by an
edit.

**The predicate is `hashlock.IsMS1Shaped`** (`hashlock/hashlock.go:122-148`),
the host's `looks_like_ms1` ported byte for byte — **no checksum**, and that is
H2 §2 rule 3's own argument: a GROUPED plate is what `ms hashlock`'s card prints
and therefore what an operator retypes, and `codex32.IsPreimage` would answer
false for it.

```
This looks like an ms1 string. A seed plate comes from a payload; a marked hashlock plate comes from the Wallet Policy program, from a phrase typed there or a preimage packed on the host. Continue here to cut it as plain text.
```
MEASURED: **204 drawn / headroom 302** through `confirmWarningBody` wrapped in
`composerConfirmBody`, and **184 / 378** on `errorScreenBody`.

**The passphrase program shows the SAME body.** The string is about to become a
BIP-39 passphrase rather than a plate, but the sentence that matters — what it
looks like, and where the marked plate comes from — is identical, and **two
near-identical bodies is how one of them goes stale**.

- [ ] **Step 2: §8.8's Password-program notice.** The refusal at `progPassword`
is correct, structural, and **completely invisible**:

```go
// gui/sysw_session.go:270-278
func syswOfferAlt(ctx *Context, th *Colors, want sysw.Class, title, lead, alt string) (string, bool) {
	if ctx.sysw == nil || !ctx.sysw.has(want) {
		return "", false          // <-- no screen is drawn
	}
```

So the operator packs their hashlock phrase, taps, opens the program whose NAME
matches the thing they are holding, and gets the ordinary passphrase keyboard —
no offer, no mention, no reason. **The obvious next move is the harmful one**:
re-pack the phrase as a `pass:` record so it "works", which is the substitution
ruling L2 exists to prevent and whose stated stake is a different wallet.

Note the asymmetry this leaves without §8.8: §9 gives the passphrase program a
warning for an operator who TYPES an ms1 string at it — the rarer mistake, with
no funds consequence — while the operator whose payload literally CONTAINS a
hashlock phrase gets nothing. Silence is today's behaviour, so the test is
whether the wrong outcome is worse than saying nothing; here **saying nothing is
what routes the operator around the guard**, so it is worse.

One modal, drawn at `progPassword` when the loaded payload holds a `ClassPhrase`
record **and no `ClassPassphrase` record**:

```
This payload holds a HASHLOCK PHRASE, not a BIP-39 passphrase. They are not interchangeable: using one as the other opens a different wallet. A hashlock phrase is used in the Wallet Policy program.
```
MEASURED on `errorScreenBody`: **165 drawn / headroom 397.**

- [ ] **Step 3: the tests.**
**MUTATION: use `codex32.IsPreimage` instead of `IsMS1Shaped`** → the GROUPED
row fails.
**MUTATION: refuse instead of warning** → the still-cut assertion fails.
**MUTATION: drop the notice** → the operator sees the ordinary keyboard with no
explanation, which is today's behaviour and the finding.
**MUTATION: draw the notice when the payload also holds a `pass:` record** →
the both-present row fails.

**Boundary gate:** `scripts/gui-shard-test.sh ./gui/ 24`.

---

### Task 12: the walk (spec §11.7)

**Repo:** the fork. **After Tasks 8b-10.** **SPECIFIED, NOT WIRED.**

**Files:** Modify `cmd/emu/walk_hashlock_phrase.js`, `cmd/emu/needle_test.go` (if the arm changes `ok`'s shape).

- [ ] **Step 1: the H6 arm.** Type the anchor phrase, HOLD, reach the census,
accept a preimage plate, and assert **the census row carries the same
`first8..last8` the confirm modal carried**.

**H5 §4.1's doctrine BINDS:** *"a walk may READ state only to assert that what
the screen shows equals what is stored; it never drives through a hook."* So the
walk asserts the SCREEN and Task 6's goldens assert the plate, and **no third
hook carrying a preimage is added**.

- [ ] **Step 2: run it three times** — unmutated, and twice against a mutation —
with `git checkout -- <file> && git diff --quiet` proven between runs, as H5's
Step 12 did. The two mutations: (a) drop the census row's digest, (b) perturb
the stored digest. Both must FAIL the walk.

**Boundary gate:** `GOOS=js GOARCH=wasm go vet ./cmd/emu/`; `./cmd/emu/build.sh`;
`go test ./cmd/emu/`.

---

### Task 13: records — the four falsified statements, the follow-ups, the manual

**Repo:** mnemonic-engrave + mnemonic-toolkit. **Last, and records only.**

- [ ] **Step 1: the four shipped records H6 makes false** (spec §0), each
rewritten rather than deleted, so a reader sees a decision:
1. `gui/composer_hash.go:27-28` — *"THE COMPOSER DERIVES A PREIMAGE IN RAM FOR
   ONE SCREEN (H2) AND NEVER STORES, SHOWS OR ENGRAVES IT."*
2. `gui/composer_hashlock.go:17-20` — *"The preimage lives on the stack here and
   is dropped when this function returns (L7, L15)."*
3. `SPEC_wallet_policy_composer.md` §6c and its §14 row.
4. `crates/ms-cli/src/cmd/hashlock.rs:352` — the engraving card's *"write the
   method line next to your phrase; **it is on no plate**"*. §6.2 and §8.6 put
   the method line ON the phrase-form plate. It errs SAFE (it asks for a copy the
   operator no longer strictly needs), and it is still a record this stage
   falsifies in a repo the spec treats as primary.

`codex32/mspayload.go:63-93`'s `IsPreimage` header keeps its second half — still
true, and still the reason the plate this stage cuts is not a seed plate — and
loses its *"the device learns to USE a preimage in stage H2, not here"*
sentence.

- [ ] **Step 2: FOLLOWUPS.** Close the plate half of **F-132**
(`design/FOLLOWUPS.md:4298`). File, with owning phases:
- **a CLI producer for `phrase:` records** (`ms hashlock … --emit-phrase-record`
  or `me sysw record phrase --method …`). §3.1 gives the wire form a Rust
  function and §3.2 a flag; **no verb emits one**, so the operator hand-builds
  hex every time, and §3.3's orphan warning is a mitigation rather than a fix.
  **A wire form with no writer is worth its own follow-up.**
- **a reconcile-style screen for a PAYLOAD phrase** (§10.2).
- **cross-run awareness of preimage plates already cut** (§5.3 item 6, §8.4b).
- **the `composerNotePhraseDigest` doc comment cites `gui/composer_flow.go:34`
  for the construction site, which is stale** — the literal is at `:48` (spec
  §2.2 item 1 names it as a nit for this stage's fold).
- **F-483** stays open and is re-stated: the typed phrase lives in
  `kbd.Fragment`, an immutable Go string, before H6 stores anything.
- **THE THREE BASELINE REDS** of Global Constraints, each with its reproduction:
  the `gofmt` trio, the `go vet` go-directive skew, and the box-local
  `history_purge` trio.

- [ ] **Step 3: the toolkit manual** — `--pack-preimage`, the two plate forms,
§8.6's text, and the fact that `ms hashlock` does not yet PARSE it.

- [ ] **Step 4: §12's acceptance, transcribed into the acceptance doc**, with
**item 8 flagged as a GATE rather than an assumption**: before the QR toggle
ships, one test plate is cut at the WORST case — a 100-character hardened
phrase, v9, 53 modules, scale 2, 0.6 mm modules — and scanned with a phone.
**53 modules at scale 2 has no precedent in this tree on either axis**, and
§11.3's gates are all bytes and toolpath. If it does not scan, **the QR toggle
does not ship and the phrase form is text-only**; the plate is still complete,
because §6.4 makes the TEXT authoritative. Cut it with the single-character
test-plate pattern (~2 s a try) rather than a full plate (~21 min).

---

## Build gate

**Three scratch trees, left in place**, each a `git ls-files | tar` copy of its
repository at the baseline revision:

| tree | repo | baseline |
| --- | --- | --- |
| `/scratch/code/shibboleth/.tmp/h6-gate` | seedhammer fork | `fb0dd04` |
| `/scratch/code/shibboleth/.tmp/h6-ms` | mnemonic-secret | `504ff46` |
| `/scratch/code/shibboleth/.tmp/h6-me` | mnemonic-engrave | `75f00b56` |

`scripts/h6-plan-blocks-vs-tree.sh` compares every block carrying a `file=`
header against those trees, through a symlink farm rooted at `fork`, `ms` and
`me`. **65 blocks checked, 0 FAIL.**

### WHAT WAS WIRED AND RUN, AND WHAT WAS NOT

**A gate that hides its blind spot is worse than no gate**, so this is stated
before the results rather than after them.

**WIRED, BUILT AND RUN by the plan author** — every block in these tasks is
byte-for-byte the text that was compiled and tested, and every `MUTATION:` in
them was executed and its failure quoted:

- **Task 1** (mnemonic-secret): the phrase rule and `qr_text` in `ms-codec`,
  `ms-cli`'s two delegations, the seven corpus rows, the three tests.
- **Task 2** (engrave Rust): the `phrase:` record, the two classes,
  `preimage_plate_admissible`, the 21 `CASES` rows, the regenerated 68-row
  corpus, the moved seam row.
- **Task 3** (engrave Rust): the flag, `Admission.pack_preimage`, `admit_check`'s
  second rule, `SyswError::PreimageNotAdmitted`, §8.1.1, §8.1.2, all four
  warnings, `preimage_digest_of`, the two operator-facing surfaces, and the
  shipped unit test's three new rows. **Step 10's integration file was NOT
  written**; its rows are specified in the table.
- **Task 4** (fork `engrave/`): the alignment table, the bound, the four budget
  arms, the scale-2 arm, the seven tests, the two falsified shipped records.
- **Task 5** (fork `codex32/`): both functions and all four tests.
- **Task 6** (fork `backup/`): the whole plate layout and all eight tests, plus
  the second falsified shipped test.
- **Task 7** (fork `sysw/`, `gui/sysw_admit.go`): both classes, the port, the
  classifier arm, the admission row, the re-vendored corpus and the lockstep.
- **Task 8a** (fork `gui/`): the retention field, the provenance, the hold, the
  scrub and its three tests.

**SPECIFIED AND NOT WIRED** — Tasks **8b, 9, 10, 11 and 12**.
Their blocks carry no `file=` header and the checker does not check them. They
are the fork's `gui` surface: two new screens, a new flow, a changed
`composerCensusLines` signature, five new copy bodies and a walk arm. **Every
measured number quoted in them is the R0-GREEN spec's own measurement, carried
across, not the author's** — the author measured nothing on those screens.
Treat those five tasks as a specification an implementer must build and gate,
not as gated text; the first thing each of them owes is its own RED.

### Per-task boundary gates, in order, as run

| # | gate | result |
| --- | --- | --- |
| 1 | ms: `cargo nextest run --locked` | **562 run, 562 passed, 11 skipped** |
| 2 | me: `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` | **621 run, 618 passed, 3 failed** (baseline `history_purge`), 2 skipped |
| 3 | me: the same, after the flag, the warnings and the nine integration tests | **630 run, 627 passed, 3 failed** (baseline `history_purge`), 2 skipped |
| 4 | fork: `go test ./engrave/` (with the four goldens generated AFTER the budgets) | ok |
| 5 | fork: `go test ./codex32/` | ok |
| 6 | fork: `go test ./backup/` | ok |
| 7 | fork: `go test ./sysw/` | ok — **68 of 68 corpus rows classify identically on the host and the device, including all 21 `phrase:` rows** |
| 8a | fork: `go test -run 'TestComposerHolds…\|TestComposerFlowExitScrubs\|TestComposerScrubIsInThe' ./gui/` | ok |
| — | fork, final sweep: `go test ./engrave/ ./backup/ ./codex32/ ./sysw/ ./hashlock/` | **all ok** |

**The three baseline reds, each MEASURED on the pristine checkouts:**
1. `gofmt -l gui/ sysw/ backup/ engrave/ codex32/` on the pristine fork prints
   `gui/transaction.go`, `gui/transaction_golden_test.go`,
   `gui/transaction_txrecord_test.go`. H6 adds no fourth: every file this plan
   touches is `gofmt`-clean.
2. `go vet ./engrave/` **exits 1** on the pristine fork with two
   `testing.ArtifactDir requires go1.26 or later (file is go1.25)` diagnostics;
   over the wired tree `go vet ./engrave/ ./backup/ ./codex32/ ./sysw/` exits 1
   with six, the four extra being the shipped `backup` goldens and H6's own two.
   Same diagnostic, same cause, no new class.
3. The three `history_purge` failures are `/usr/bin/zsh` missing on this box, and
   the test says so in its own words: *"there is no way to run it without zsh.
   This is deliberately a FAILURE and not a skip -- a skipped gate prints ok and
   exit 0."*

### Whole-package gates on the fork

- `GOOS=js GOARCH=wasm go vet ./cmd/emu/` — **exit 0**.
- `./cmd/emu/build.sh` — **exit 0**, `built emu.wasm (10882452 bytes)`.
- `scripts/gui-shard-test.sh ./gui/ 24` — **1242 top-level tests, partition
  verified exhaustive (1242 == 1242), 23 of 24 shards ok, shard 18 FAIL**, run
  twice with the same single failure:
  `TestComposerEveryScreenFunctionHasAProductionCaller` names
  `composerHoldHashlockMaterial`. **That red is CORRECT and is Task 8a's own
  finding** — see the note there: 8a and 8b land in one commit, and the
  exemption table is not the instrument.

### Firmware size

Measured with `nix develop -c tinygo build -size short -o /dev/null -target
pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`:

| tree | flash | ram |
| --- | --- | --- |
| pristine fork `fb0dd04` | **1,599,208 B** | 62,856 B |
| the wired tree (Tasks 4, 5, 6, 7, 8a; final budgets) | **1,600,944 B** | 63,248 B |
| delta | **+1,736 B** | **+392 B** |

That is the codec-and-plate half only. Tasks 8b-12 add two screens, a flow and
five copy bodies, so the stage's total will be larger; §11.6 already says *"the
delta is not expected to be small"*.

---

## What this plan found against the R0-GREEN spec

Nine things the spec does not say, every one of them measured. None is a
Critical against the spec's design; six are things an implementer following it
literally would have got wrong.

1. **`me` cannot call the phrase rule the spec names.** §3.1 requires a
   `phrase:` body to pass *"`ms-cli`'s `validate_phrase` … byte for byte"*, and
   `me-cli` depends on `ms-codec`, not on `ms-cli`. A literal reading produces a
   THIRD copy of the rule. **Task 1 moves it into the codec** and makes the
   stage's first deliverable a published release. *(Blocks Task 2; the plan
   author stood a `[patch.crates-io]` in front of it to build the rest.)*
2. **§11.4's method-line mutation cannot fail.** *"Add one character to the
   method line → the worst case no longer fits"* — measured, 74, 75, 76, 77 and
   78 characters all still fit, because 39 characters per line wraps a
   73-character line to 2 rows anywhere below 79. §6.5 consequence 3's own
   number (*"a 79th character"*) is the threshold, and Task 6 carries it.
3. **`preimage_plate_admissible` needs TWO more conjuncts than §4.3 gives it**,
   and the shipped `a_preimage_plate_is_named_not_misdiagnosed` found both: a
   kind-`0x03` single under the id `hash` whose X is 16 bytes, and the UPPERCASE
   spelling of a plate. The device's `codex32.IsPreimage` already required
   `len(d) == 33 && d[0] == 0x03`, so **Go was right and Rust needed narrowing** —
   which is the Rust-primary rule's own "whenever a defect is found in a Go port
   we MUST check the Rust" running in reverse.
4. **The raise falsifies two shipped tests §14 does not list.**
   `TestConstantQRLargeVersionsFailClosed` asserts dim 41 is REFUSED;
   `TestPassphraseQRTooLong` uses a 200-character passphrase (dim 53) as its
   out-of-reach case. Both are rewritten at v10 in Tasks 4 and 6.
5. **`composerHoldHashlockMaterial` trips the fork's own join guard**, so §2.2's
   retention cannot land as a commit of its own. Recorded on Task 8a.
6. **§6.2/§6.3 and §6.5 disagree about body-row ORDER** — method line first
   versus locator first. The normative sections win; the row count is identical
   either way, so no measurement moves. Recorded on Task 6 Step 2.
7. **Making the classes BEARER moves the argv surface**, and the spec does not
   say so. `me sysw pack --pack-preimage --no-passphrase <ms1>` — §12 item 3's
   own acceptance invocation — is refused by `argv_secret_guard` before the
   parser runs, and the guard's message named *"a signed transaction, or the
   mt1 set carrying one"* for a hashlock preimage. **The refusal is correct and
   its wording was not**; Task 3 Step 10 fixes the wording and Task 13 fixes the
   acceptance path.
8. **§11.3's budget mutation cannot fire from any in-suite sample.** With the
   four arms at the campaign maximum MINUS ONE, the fuzzed row still passes —
   its 400 payloads per dimension see 792 / 910 / 1143 / 1322, and the true
   maxima took 7 to 14 million. Task 4 adds
   `TestConstantTimeQRBudgetEntriesAreTheFuzzedOnes`, a PIN, which is the row
   that actually carries "derived rather than guessed".
9. **The `preimage-plate-0x03` seam row moves class**, from `Unknown` to
   `Preimage`, in a corpus whose test is named *"not one of these records may
   change class"*. Its `entr`-id sibling stays `Unknown`, and that PAIR is §4.3
   stated as data. Recorded in Task 2 Step 7 rather than absorbed.

## Follow-ups this plan files

Owning phases, per the constellation's per-phase burndown rule:

| item | owning phase |
| --- | --- |
| a CLI producer for `phrase:` records — a wire form with no writer | a later `ms`/`me` cycle |
| §12 item 3's acceptance invocation must use `--in`; the argv guard refuses a preimage on the command line | **Task 13** (records) |
| a reconcile-style screen for a PAYLOAD phrase (§10.2) | a later device cycle |
| cross-run awareness of preimage plates already cut (§5.3 item 6, §8.4b) | a later device cycle |
| `composerNotePhraseDigest`'s doc comment cites `composer_flow.go:34`; the literal is at `:48` | **Task 8a** (fix inline) |
| `go.mod` says `go 1.25.10` while the toolchain is 1.26.7 and the project floor is `go1.26`, so `go vet` exits 1 on any package using `t.ArtifactDir()` | fork toolchain hygiene |
| F-483 — the typed phrase in an unwipeable Go string | open, non-gating (2026-08-27) |
