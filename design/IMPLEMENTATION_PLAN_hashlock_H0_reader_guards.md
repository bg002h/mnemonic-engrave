# Hashlock H0 — Reader Guards Implementation Plan

**IMPLEMENTATION RECORD (2026-09-05):** executed on branches `hashlock-h0` in both repos (ONE opus implementer; report `design/agent-reports/hashlock-H0-implementation-report.md`): engrave 95cd48a merged to master with `--no-ff` at the commit before this line; fork 83fbc17 merged to fork `main`. Emulator walk (Task 3 Step 2, `cmd/emu/walk_h0_preimage.js` on the fork): typed door → "This record is a hashlock preimage, not a seed. It is not engraved as one.", NFC door → "Unknown format"; the same walk on unguarded main 839fa5aa reaches "Confirm Codex32 Secret / id HASH / Unshared secret (S) / 75 chars" at BOTH doors. Post-implementation review (opus, `hashlock-H0-post-impl.md`) 1C/2I/4M/1N: **C-1** the guard sat outside `engraveCodex32`'s loop and the Recover arm reassigns `scan` from interpolated shares — the plan's Step 8 was wrong about "the choke point" being one test before the loop; fixed at the top of the loop with a Recover-arm test. **I-1** a plain BIP-93 33-byte seed beginning 0x03 IS the preimage shape and is refused — the plan's "16..32-byte payload is untouched" was false for 33-byte seeds; wording fixed, collision row `bip93-plain-33-byte-payload-0x03` added (13 rows, sha `bb703f608215bb00ccc677de4a282772016e774dd2d1d0f5c828ea38f5eac78b`, capture 38). **I-2** `me seal` refused with the raw codec error; `RecordError::PreimagePlate` now names the kind on both hosts. Fold verification (sonnet) GREEN. Firmware 1,583,132 / 62,800. Flash: at the operator's word.

**STATUS: R0 GREEN 2026-09-04 (0 Critical / 0 Important open).** Round 0: fidelity (opus, `hashlock-H0-plan-R0-r0-fidelity.md`, 2C/5I/2M/0N) + tests/mutation (sonnet, `hashlock-H0-plan-R0-r0-tests.md`, 0C/1I/3M), one fold (`fdfb040`). Round 1: fold verification (sonnet, `hashlock-H0-plan-R0-r1-fold-verification.md`): 8/8 fixed, 1 new Important (the fold's whole-crate claim; the record-corpus capture), folded (`64a6e0d`). Round 2: fold verification (sonnet, `hashlock-H0-plan-R0-r2-fold-verification.md`): GREEN, one wording observation — the `PreimagePlate` message asserted `id \`hash\`` for a shape whose id may be `entr`; the message now names the kind only (wording; the binary test's assertion is unchanged and was re-run). Lens-closure: fidelity, tests/mutation, fold-verification ×2.
Previous STATUS: DRAFT 2026-09-04 — R0 round 1 FOLDED (sonnet fold verification `hashlock-H0-plan-R0-r1-fold-verification.md`: 8/8 C+I fixed, 1 new Important — the round-0 fold's whole-crate claim did not reproduce because the corpus edit breaks `tests/record_corpus.rs`'s pre-S2 capture, never mentioned; fixed below in Task 1 Step 1b, whole crate re-run with `--no-fail-fast` in a worktree with its own target dir, output in the r1 fold commit's message); round 2 (sonnet, scoped to the capture step and the gate claim) pending.**
Previous STATUS: R0 round 0 folded (fidelity opus 2C/5I/2M/0N; tests sonnet 0C/1I/3M), gate re-run green.
Previous STATUS: DRAFT — build gate green at `b0af794`.
H0 is the prerequisite `SPEC_ms_hashlock.md` §9 places BEFORE ms-cli 0.18.0
(a controller default awaiting the operator: H0 precedes the release rather
than following it as H2). Its ordering is the operator's; its content is
needed under either ordering.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make a kind-`0x03` hashlock PREIMAGE plate string inert on every
reader that would today take it for a seed — the SeedHammer fork's two
classifiers, its two engrave doors, and `me`'s record validator — with shared
vector rows that turn red on whichever side ever regresses, and get the fork
change flashed.

**Architecture:** One new predicate in the fork's `codex32` package,
`IsPreimage(String) bool`, answers *"is this a preimage SINGLE?"*: an
UNSHARED string whose payload is exactly 33 bytes and begins `0x03`. The two
device classifiers (`sysw.isStrictMs1`, `seal.Classify`), the NFC scan
classifier they mirror (`gui/scan.go`), and the engrave choke point every
door reaches (`gui.engraveCodex32`, plus `unlockEngraveCodex32` on the sealed
path) consult it and treat a preimage string as UNKNOWN (no new class). On
the host, `me` at its ms-codec `0.7` pin already refuses the string at the
codec's prefix gate; H0 adds the corpus rows, a pin test that trips on the
`0.8` bump, and a diagnosis that names the kind instead of misreporting the
profile. The shared `codex32_seam_vectors.json` corpus, pinned by sha256 in
both suites, carries the rows.

**Tech Stack:** Rust (`me-cli`, package `mnemonic-engrave`, ms-codec `0.7`
pinned), Go 1.26 at `/scratch/code/shibboleth/.toolchain/go/bin/go` (fork
tests) and the nix flake's Go for firmware, `~/bin/sh/sh2-flash`.

**Spec:** `mnemonic-secret/design/SPEC_ms_hashlock.md` §1 (kind `0x03`, id
`hash`, 75 characters; rule 2: the id/kind check is singles-only), §9 (the
reader table and H0), §12 item 7 ("no engrave path offers it"), §14.

**Baselines (for `scripts/plan-staleness-check.sh`):** mnemonic-engrave
`e06e29d`; seedhammer fork main `839fa5aa`; mnemonic-secret `3592532` (the
H1 plan whose gate produced the vector string).

## Global Constraints

- **Rust-primary rule (CLAUDE.md):** this is a CONVERGENCE fix, exemption (a).
  The Rust primary of both device classifiers is `me`'s `validate_record` →
  `sysw::classify`, which at ms-codec `0.7` refuses every `0x03` string for
  its KIND (`ReservedPrefixViolation { got: 3 }` — the prefix dispatch runs
  before the tag accept set; verified by the fidelity lens, item 3). The Go
  ports accept it because `codex32.New` pins no prefix. Go converges on
  Rust's current answer; nothing is decided in Go. `me` refuses at
  `crates/me-cli/src/seal/record.rs:176` (`ms_codec::decode(s)`); Task 1 pins it.
- **The predicate is singles-only and shape-exact.** Spec §1 rule 2: *"the
  check applies to singles only: a share-set's id is random by construction
  and names no kind."* A K-of-N share's data part is an SSS point (its first
  byte is whatever the polynomial gave it), and a plain BIP-93 secret's data
  part is the seed itself — so "first byte equals 3" alone would make ≈1/256
  of legitimate shares and secrets inert, and ONE such share in a sealed
  payload would refuse the whole payload (fidelity C-2). Hence:
  `IsPreimage` = `ParsePrefix(...).Unshared && len(Seed()) == 33 && Seed()[0] == 0x03`.
  The id is NOT consulted: the kind is the prefix byte (§1); a `0x03` single
  under any other id is a mismatch the host refuses, and not a seed either way
  (tests I-1: an id-keyed predicate would engrave a mistagged real preimage as
  a seed).
- **Minimal narrowing.** ONLY that shape becomes inert. The fork's wider
  acceptance (plain BIP-93 strings at 48/74 characters, shares) is unchanged;
  the corpus now PROVES it with `device_admits: true` rows whose payload
  begins `0x03`. Whether the device should converge fully on the constellation
  profile is a follow-up (Task 4), not this plan.
- **No new class, and what the operator sees.** A preimage string classifies
  `ClassUnknown` in `seal` and in `sysw`, and `gui/scan.go` returns
  `errScanUnknownFormat` for it (the documented mirror of `seal.Classify`,
  `seal/record.go:95-98`, stays true). The two containers then diverge, as they
  do for any unknown record: in the **sysw** container the record is
  per-record inert (stays in the session, offered to no program, counted
  "inert" on the composer door); in the **sealed** container `AdmitSection`
  refuses the encrypted section WHOLE with `ErrRecordNotPermitted`, which
  `gui/unlock_kdf.go`'s `default:` arm renders as **"Payload unreadable."**
  after a successful passphrase. That is the right H0 behaviour — `permitted`
  is an allow-list on purpose (`seal/record.go:225-229`) — and it is reachable
  only by hand-built blobs, since `me` `0.7` cannot pack the record. A
  dedicated arm naming the record is a follow-up owned by H2 (Task 4).
- **Secret-handling defects never gate** (operator ruling 2026-08-27). Every
  vector string here is a fixture, not anyone's secret, and public in two repos.
- **The vector file is shared byte for byte.** Its sha256 after Task 1 is
  `f1f2fa6bbbf27e3697ee496636de49be2f25787deff7b3bc4a2c5e16854e391c`
  (measured on the exact row text below; 12 rows: 2 both / 6 device-only /
  4 neither); both literals are re-pinned to it.
- **Fork commits** signed + DCO, author Brian Goss; branch `hashlock-h0` off
  fork `main`; small PR. **Stage paths explicitly** (no `git add -A`).
- **Flash only via `~/bin/sh/sh2-flash -y`, at the operator's word**, never
  `picotool` by hand. Firmware size is measured before and after.
- **Cite the anchor TEXT, not only the line.** Three of this plan's original
  line citations were off (fidelity I-1); every `Modify` below now quotes the
  text to anchor on, and the line is a hint.
- **Whole-crate numbers are measured with `--no-fail-fast`, in a worktree with
  its OWN `CARGO_TARGET_DIR`.** Without `--no-fail-fast` nextest stops after
  the first failing binary, and the round-0 fold quoted "615/616" for a tree
  that was 610/616 (r1). A target dir shared between worktrees hands a
  whole-crate run unit-test binaries compiled from ANOTHER tree (their
  `testdata/` paths are baked in at compile time), which fails seven vector
  tests for a reason that is not the plan's. And `touch` a file restored from
  a backup, or cargo reuses the mutated build.

---

## File Structure

| File | Change | Responsibility |
| --- | --- | --- |
| engrave `crates/me-cli/testdata/codex32_seam_vectors.json` | Modify (append rows 9-12) | the shared corpus: `preimage-plate-0x03`, `bip93-plain-payload-0x03`, `bip93-share-payload-0x03`, `preimage-shape-entr-id` |
| engrave `crates/me-cli/tests/codex32_seam.rs:25-26` | Modify (fragment) | re-pin `SEAM_VECTORS_SHA256` |
| engrave `crates/me-cli/testdata/record_corpus_pre_s2.json` | Modify (append 4 entries) | S2's invariant-2 capture enumerates the seam corpus, so it grows 33 → 37 with the same rows; class `Unknown`, consult `record-refusal` |
| engrave `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs` | Create | host pin: `validate_record` is `Err`, `sysw::classify` is not `Codex32Secret`; `me sysw pack` names the kind |
| engrave `crates/me-cli/src/seal/record.rs` (after `bip93_outside_the_profile`) | Modify (append fn) | `preimage_plate(&str) -> bool` |
| engrave `crates/me-cli/src/sysw/mod.rs` (`UnknownReason`, `unknown_reason`, tests) | Modify (fragments) | `UnknownReason::PreimagePlate`; the arm BEFORE the profile arm; unit test |
| engrave `crates/me-cli/src/main.rs` (the `U::Bip93OutsideTheProfile` arm) | Modify (fragment) | the Display text for the new reason |
| fork `codex32/mspayload.go` | Modify (const block + append) | `msPrefixPreimage = 0x03`; `IsPreimage(String) bool`; `DecodeMS1` UNCHANGED |
| fork `codex32/mspayload_test.go` | Modify (append) | the six-population table; `DecodeMS1` still refuses |
| fork `sysw/classify.go` (`isStrictMs1`) | Modify (fragment) | `!codex32.IsPreimage` |
| fork `sysw/testdata/codex32_seam_vectors.json` | Replace (vendored copy) | byte-identical to the primary |
| fork `sysw/codex32_seam_test.go:30` | Modify (fragment) | re-pin `seamVectorsSHA256` |
| fork `seal/record.go:212-214` (`Classify`'s `codex32.New` branch) | Modify (fragment) | `!codex32.IsPreimage` before `ClassCodex32Secret` |
| fork `seal/record_test.go` | Modify (row + new test) | branch-order row `ClassUnknown`; `AdmitSection` refuses the section |
| fork `gui/scan.go:89` (`Scan`'s `codex32.New` arm) | Modify (fragment) | the mirror of `seal.Classify` narrows with it |
| fork `gui/codex32_polish.go` (`engraveCodex32`) | Modify (fragment) | the choke point both engrave doors reach: named refusal |
| fork `gui/codex32_polish_test.go` | Modify (append) | the door tests: engrave dispatch refuses; Scan yields no object |
| fork `gui/unlock_session.go` (`unlockEngraveCodex32`) | Modify (fragment) | named refusal on the sealed path, defence in depth |
| fork `gui/unlock_session_test.go` | Modify (append) | the harness twin and the refusal test |
| engrave `design/FOLLOWUPS.md`, `CHANGELOG`s, continuity | Modify | records |

**Gate coverage.** Neither `scripts/plan-build-gate-me.sh` nor
`plan-build-gate-go.sh` recognises these paths, and every Go edit is a
fragment of an existing file, so this plan's gate is the controller
hand-wiring every block below into scratch copies of BOTH repos and running
the named commands before review; the output goes in the fold commit's
message and the reviewer is told what ran. Known blind spot: the gate wires
every fragment at once, so TASK ORDER and line citations are the reviewer's.

**The site enumeration** (fidelity lens, item 1, verified at `839fa5aa`) — every
site that turns a codex32 string into a seed, a display of one, or an engrave:

| site | reachable by a `0x03` string? | guarded by |
| --- | --- | --- |
| `sysw/classify.go` `isStrictMs1` | yes | Task 2 Step 4 |
| `seal/record.go` `seal.Classify` | yes | Task 2 Step 5 |
| `gui/unlock_session.go` `unlockEngraveCodex32` | only via `seal.Classify` | Step 5 + Task 2 Step 7 |
| `gui/gui.go` `newInputFlow` (`syswOffer`), `gui/transaction.go` `txClassName`, `gui/composer_door.go`, `gui/sysw_admit.go` | only via `sysw.Classify` | Step 4 |
| `gui/scan.go` `Scan` (the NFC door) → `gui/gui.go` `engraveObjectFlow` | **yes, directly** | Task 2 Step 8 (Scan arm + choke point) |
| `gui/codex32_polish.go` `validateMStar` (the typed `M*1 STRING` door) → `engraveObjectFlow` | **yes, directly** | Task 2 Step 8 (choke point) |
| `gui/codex32_polish.go` `engraveCodex32` → `backupSeedStringFlow` → `backup.EngraveSeedString` | yes, from both doors | Task 2 Step 8 |
| `gui/ms1_decode.go`, `confirmCodex32Flow`'s `showSecret`, `singlesig_verify.go`, `multisig_verify.go`, `bundle/verify.go` | no — all call `DecodeMS1`, which still returns `errMSBadPrefix` on `0x03` | unchanged |
| `me` `validate_record`, `sysw::classify` | yes | ms-codec 0.7 prefix gate; Task 1 pins and names it |

---

### Task 1: The corpus rows and the host half (mnemonic-engrave)

**Files:**
- Modify: `crates/me-cli/testdata/codex32_seam_vectors.json` (append after row 8, `bip93-bad-checksum`)
- Modify: `crates/me-cli/tests/codex32_seam.rs:25-26` (anchor: `const SEAM_VECTORS_SHA256: &str =`)
- Modify: `crates/me-cli/testdata/record_corpus_pre_s2.json` (append four entries after the `codex32_seam/bip93-bad-checksum` entry)
- Create: `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs`
- Modify: `crates/me-cli/src/seal/record.rs` (append after `pub fn bip93_outside_the_profile`)
- Modify: `crates/me-cli/src/sysw/mod.rs` (anchors: `    Bip93OutsideTheProfile(usize),`; the profile arm in `fn unknown_reason`; the `/// **THE CONTROL, in both directions.**` test)
- Modify: `crates/me-cli/src/main.rs` (anchor: `U::Bip93OutsideTheProfile(len) => format!(`)
- Package name for `-p` is `mnemonic-engrave`.

**Interfaces:**
- Consumes: `mnemonic_engrave::seal::record::{validate_record, RecordError}` (`pub fn validate_record(s: &str) -> Result<RecordKind, RecordError>`; `RecordError::MsTooLong(usize)` at `record.rs:71`), `sysw::classify`, `sysw::record::Class`, `sysw::pack`, `SyswError::Unclassifiable(usize, UnknownReason)`, `ms_codec::Error::ReservedPrefixViolation { got: u8 }` (ms-codec 0.7 `error.rs:62`).
- Produces: rows 9-12 and the sha256 above (Task 2 vendors and re-pins); `seal::record::preimage_plate(&str) -> bool`; `UnknownReason::PreimagePlate`.

- [ ] **Step 1: Append the four rows.** The file ends `    }\n  ]\n}\n`. Replace that tail with `    },\n` + the rows + `  ]\n}\n`, indentation exactly as the existing rows. The four strings were produced on the fork at `839fa5aa`: the first by the H1 plan's gate-wired `ms hashlock`, the other three by `codex32.NewSeed` (the plate is the only real preimage; the rest are the populations the guard must not touch, plus the shape under the wrong id).

```json
    {
      "name": "preimage-plate-0x03",
      "string": "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c",
      "chars": 75,
      "host_admits": false,
      "device_admits": false,
      "source": "SPEC_ms_hashlock section 1, kind 0x03 with id `hash`: a hashlock PREIMAGE plate, produced by the H1 plan's gate-wired `ms hashlock` (mnemonic-secret 3592532, gate run 13); ms-codec 0.7 refuses it with `reserved-prefix byte was 0x03, expected 0x00` at exit 2 (that plan's downgrade row). NOT A SEED: cut as one it exposes a spend secret as a backup, loaded as one it derives keys from a hashlock preimage. H0 (SPEC_ms_hashlock section 9) makes it INERT on both sides -- never Codex32Secret, no class of its own -- and this row is the tripwire: it goes red on the host the day `me` bumps to ms-codec 0.8 without a refusing arm, and on the device if the prefix test is ever removed."
    },
    {
      "name": "bip93-plain-payload-0x03",
      "string": "ms10testsqv0qqqqqqqqqqqqqqqqqqqqqqq8mzk8tjfdnjn5",
      "chars": 48,
      "host_admits": false,
      "device_admits": true,
      "source": "codex32.NewSeed(\"ms\", 0, \"test\", 's', 16 bytes with byte 0 = 0x03): a plain BIP-93 unshared secret whose seed happens to begin 0x03. H0's device guard MUST leave it alone -- the preimage kind is an UNSHARED string with a 33-byte payload whose first byte is 0x03, and this payload is 16 bytes. Measured on the fork at 839fa5aa. Fidelity C-2 (hashlock-H0-plan-R0-r0-fidelity.md)."
    },
    {
      "name": "bip93-share-payload-0x03",
      "string": "ms12testaqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdq7pl8qdc5tsp",
      "chars": 75,
      "host_admits": false,
      "device_admits": true,
      "source": "codex32.NewSeed(\"ms\", 2, \"test\", 'a', 33 bytes with byte 0 = 0x03): a 2-of-N SHARE whose SSS point begins 0x03. A share carries no kind byte (SPEC_ms_hashlock section 1 rule 2: the check is singles-only); H0 must not make it inert, or one such share would refuse a whole sealed payload as unreadable. Measured on the fork at 839fa5aa. Fidelity C-2."
    },
    {
      "name": "preimage-shape-entr-id",
      "string": "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9",
      "chars": 75,
      "host_admits": false,
      "device_admits": false,
      "source": "codex32.NewSeed(\"ms\", 0, \"entr\", 's', 33 bytes with byte 0 = 0x03): the preimage SHAPE (unshared, 33-byte payload, prefix 0x03) under the id `entr`. The kind is the prefix byte, not the id (SPEC_ms_hashlock section 1): ms-codec 0.7 refuses the prefix, 0.8 refuses the id/prefix mismatch (TagKindMismatch), and the device treats it as inert -- never a seed either way. Measured on the fork at 839fa5aa."
    }
```

- [ ] **Step 1b: Extend the pre-S2 record capture — and argue for it.** `tests/record_corpus.rs` is S2's invariant 2 as a gate: it ENUMERATES the corpus (`sysw_vectors.json`, then every `codex32_seam_vectors.json` row in file order, then its literals) and asserts the committed capture `testdata/record_corpus_pre_s2.json` is exactly that enumeration, that every record's class is unchanged, and that the descriptor gate stays shut on each. Four new seam rows therefore red three tests (`the_capture_is_the_whole_corpus`: "is not the enumerated corpus"; the class and gate counts 33 vs 37) until the capture carries them. The file says "a diff to this file IS a change to invariant 2 and has to be argued for": the argument is that these four records are ADDED, not moved — every one classifies `Unknown` on the host today (`host_admits: false` in the seam corpus: ms-codec 0.7 refuses the kind, and the two BIP-93 rows are outside the profile), the descriptor gate refuses each as a record (`record-refusal`), and no record that was placeable changes class. Insert, directly after the entry whose origin is `codex32_seam/bip93-bad-checksum` (keep its trailing `},`), in the file's style:

```json
    {
      "origin": "codex32_seam/preimage-plate-0x03",
      "record": "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c",
      "class": "Unknown",
      "consult": "record-refusal"
    },
    {
      "origin": "codex32_seam/bip93-plain-payload-0x03",
      "record": "ms10testsqv0qqqqqqqqqqqqqqqqqqqqqqq8mzk8tjfdnjn5",
      "class": "Unknown",
      "consult": "record-refusal"
    },
    {
      "origin": "codex32_seam/bip93-share-payload-0x03",
      "record": "ms12testaqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdq7pl8qdc5tsp",
      "class": "Unknown",
      "consult": "record-refusal"
    },
    {
      "origin": "codex32_seam/preimage-shape-entr-id",
      "record": "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9",
      "class": "Unknown",
      "consult": "record-refusal"
    },
```

Run: `cargo nextest run --locked -p mnemonic-engrave --test record_corpus`
Expected: 6/6 PASS (measured), 37 records. (At the 0.8 bump the plate's host class stays `Unknown` only if H1b's refusing arm lands — the same tripwire as Step 5, now with a third witness.)

- [ ] **Step 2: See the seam test fail on the hash.**

Run: `cargo nextest run --locked -p mnemonic-engrave --test codex32_seam`
Expected: FAIL with `testdata/codex32_seam_vectors.json is not the file the fork's copy is pinned to; re-pin BOTH literals`.

- [ ] **Step 3: Measure and re-pin.**

Run: `sha256sum crates/me-cli/testdata/codex32_seam_vectors.json`
Expected: `f1f2fa6bbbf27e3697ee496636de49be2f25787deff7b3bc4a2c5e16854e391c`. If not, the row text differs from Step 1 byte for byte — fix the rows, never pin a different hash.

Replace the two lines under `const SEAM_VECTORS_SHA256: &str =` (`codex32_seam.rs:25-26`):

```rust
const SEAM_VECTORS_SHA256: &str =
    "f1f2fa6bbbf27e3697ee496636de49be2f25787deff7b3bc4a2c5e16854e391c";
```

- [ ] **Step 4: Run it again.**

Run: `cargo nextest run --locked -p mnemonic-engrave --test codex32_seam`
Expected: PASS (measured). All four new rows' host verdicts are `false` already at ms-codec 0.7; 12 rows: 2 both / 6 device-only / 4 neither.

- [ ] **Step 5: The pin test** — green at 0.7 by construction; its job is the 0.8 bump. Create `crates/me-cli/tests/preimage_plate_is_not_a_seed.rs`:

```rust
//! H0 (SPEC_ms_hashlock §9): a kind-0x03 hashlock PREIMAGE plate is never a
//! seed record on the host.
//!
//! At ms-codec 0.7 the codec's own prefix gate refuses the string, so this
//! passes without any change to `validate_record`. When `me` bumps to
//! ms-codec 0.8 (stage H1b) the codec DECODES it as `Payload::Preimage`, and
//! `validate_record`'s `.map(|_| RecordKind::Ms)` would call a preimage a
//! seed. This test is what goes red that day. The seam corpus row
//! `preimage-plate-0x03` pins the same fact through `sysw::classify`.
use mnemonic_engrave::seal::record::{validate_record, RecordError};
use mnemonic_engrave::sysw::record::Class;

/// The H1 plan's downgrade-row string: 75 characters, id `hash`, prefix 0x03.
const PREIMAGE_PLATE: &str =
    "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c";

#[test]
fn a_preimage_plate_is_not_a_seed_record() {
    assert_eq!(PREIMAGE_PLATE.len(), 75);
    // MUTATION: `.map(|_| RecordKind::Ms)` admitting Payload::Preimage after
    // the 0.8 bump -> this arm receives Ok(RecordKind::Ms) and panics.
    match validate_record(PREIMAGE_PLATE) {
        Ok(kind) => panic!("validate_record admitted a 0x03 preimage plate as {kind:?}"),
        // Not `MsTooLong`: 75 characters is inside the engraveable cap, and
        // the refusal must be about the KIND, not the length.
        Err(RecordError::MsTooLong(n)) => panic!("refused as too long ({n}), not as a preimage"),
        Err(_) => {}
    }
    assert_ne!(
        mnemonic_engrave::sysw::classify(PREIMAGE_PLATE),
        Class::Codex32Secret,
        "sysw::classify called a preimage plate a codex32 secret"
    );
}

/// The operator-visible half (fidelity I-3): `me sysw pack` names the kind.
/// The record itself is never echoed.
#[test]
fn sysw_pack_names_a_preimage_plate_and_never_echoes_it() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut c = Command::new(assert_cmd::cargo::cargo_bin("me"))
        .args(["sysw", "pack"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // A broken pipe is the refusal arriving first, not an error (see dash_stdin.rs).
    let _ = c.stdin.take().unwrap().write_all(format!("{PREIMAGE_PLATE}\n").as_bytes());
    let out = c.wait_with_output().unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "sysw pack accepted a preimage plate: {err}");
    assert!(err.contains("hashlock PREIMAGE plate"), "stderr does not name the kind:\n{err}");
    assert!(!err.contains("outside") && !err.contains("re-encode the entropy"), "misdiagnosed as outside the profile:\n{err}");
    assert!(!err.contains(&PREIMAGE_PLATE[10..40]), "the record was echoed:\n{err}");
}
```

- [ ] **Step 6: Run the first test; see the second fail for the right reason.**

Run: `cargo nextest run --locked -p mnemonic-engrave --test preimage_plate_is_not_a_seed`
Expected: `a_preimage_plate_is_not_a_seed_record` PASS; `sysw_pack_names_a_preimage_plate_and_never_echoes_it` FAIL with `stderr does not name the kind` — today `unknown_reason` reaches the profile arm and prints *"is a VALID BIP-93 codex32 string … not a constellation `ms1` record … This one is 75 characters … re-encode the entropy as `ms1`"*: it lists a set containing 75, then gives 75 as the reason, calls a constellation record not one, and tells the operator to re-encode a preimage as seed entropy (fidelity I-3).

- [ ] **Step 7: The diagnosis.** Append to `crates/me-cli/src/seal/record.rs`, directly after `pub fn bip93_outside_the_profile`'s closing brace:

```rust
/// Is `s` a hashlock PREIMAGE plate (SPEC_ms_hashlock §1: kind byte `0x03`,
/// id `hash`, 75 characters) — the one `ms1` string that is inside the
/// profile's lengths and is still not a seed?
///
/// H0 (SPEC_ms_hashlock §9). At ms-codec 0.7 the codec refuses the kind with
/// `ReservedPrefixViolation { got: 3 }`, and this asks for exactly that, so
/// the diagnosis names the kind instead of the profile. At the 0.8 bump the
/// codec DECODES the kind and this arm must be re-pointed at the refusing
/// arm `validate_record` gains then; `preimage_plate_is_not_a_seed.rs` is
/// what goes red if either half is forgotten.
pub fn preimage_plate(s: &str) -> bool {
    let s = s.trim();
    matches!(classify(s), Ok(Format::Ms))
        && matches!(
            ms_codec::decode(s),
            Err(ms_codec::Error::ReservedPrefixViolation { got: 0x03 })
        )
}
```

In `crates/me-cli/src/sysw/mod.rs`, after the `Bip93OutsideTheProfile(usize),` variant (anchor text `    Bip93OutsideTheProfile(usize),`):

```rust
    /// A hashlock PREIMAGE plate (SPEC_ms_hashlock §1, kind `0x03`, id
    /// `hash`): inside the profile's lengths, refused for its KIND. Named so
    /// the operator is not told to "re-encode the entropy as `ms1`" — the
    /// string is a constellation record, just not one this container places
    /// yet (H0, §9). Carries no number: 75 characters is the only shape.
    PreimagePlate,
```

In `fn unknown_reason`, replace the profile arm and the trailing `Unrecognised` (anchor: `    if crate::seal::record::bip93_outside_the_profile(record) {`):

```rust
    // Before the profile arm: a preimage plate is INSIDE the profile's lengths
    // and would otherwise be reported as outside them (H0, SPEC_ms_hashlock §9).
    if crate::seal::record::preimage_plate(record) {
        return UnknownReason::PreimagePlate;
    }
    if crate::seal::record::bip93_outside_the_profile(record) {
        return UnknownReason::Bip93OutsideTheProfile(record.trim().chars().count());
    }
    UnknownReason::Unrecognised
}
```

In `crates/me-cli/src/main.rs`, immediately before the arm `U::Bip93OutsideTheProfile(len) => format!(`:

```rust
                U::PreimagePlate => format!(
                    "record {i} (records count from 0) is a hashlock PREIMAGE plate (kind \
                     0x03), not a seed record; this container cannot place one yet. A \
                     preimage backs a hashlock spend path, not a wallet — keep it with the \
                     policy it unlocks, and do not re-encode it as entropy."
                ),
```

In `sysw/mod.rs`'s test module, immediately before the doc comment `/// **THE CONTROL, in both directions.**`:

```rust
    /// H0 (SPEC_ms_hashlock §9): a preimage plate is refused for its KIND,
    /// named as such, and NOT as "outside the profile" — it is 75 characters,
    /// inside the profile, and the profile arm would claim it if it ran first.
    /// MUTATION: swap the two arms in `unknown_reason` -> `Bip93OutsideTheProfile(75)`.
    #[test]
    fn a_preimage_plate_is_named_not_misdiagnosed() {
        const PLATE: &str = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c";
        assert_eq!(PLATE.chars().count(), 75);
        assert_eq!(
            pack(vec![PLATE.into()], None, ITER),
            Err(SyswError::Unclassifiable(0, UnknownReason::PreimagePlate)),
        );
        // The control: an entr string of the same length is still a seed.
        assert!(matches!(
            classify("ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w"),
            record::Class::Codex32Secret
        ));
    }

```

- [ ] **Step 8: Run, mutate, run.**

Run: `cargo fmt -p mnemonic-engrave && cargo nextest run --locked -p mnemonic-engrave -E 'test(/preimage|codex32_seam|outside_the_profile/)'`
Expected: 5 PASS (measured: the two integration tests, the two existing profile tests, the new unit test). Mutation, measured: swap the two arms in `unknown_reason` → `a_preimage_plate_is_named_not_misdiagnosed` FAILS with `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`. (When reverting a mutation by moving a backup file back, `touch` it — cargo keys on mtime and reused the mutated build once at the gate.) Mutation, measured: insert `if s.starts_with("ms10hash") { return Ok(RecordKind::Ms); }` before `record.rs:176`'s `ms_codec::decode(s)` → `a_preimage_plate_is_not_a_seed_record` FAILS with `validate_record admitted a 0x03 preimage plate as Ms` and the seam test FAILS with `preimage-plate-0x03: host verdict`.

- [ ] **Step 9: Whole crate, then commit.**

Run: `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast && cargo clippy --locked -p mnemonic-engrave --all-targets -- -D warnings && cargo fmt -p mnemonic-engrave -- --check`
Expected: green under CI's toolchain. Measured at the r1 gate in a worktree with its own target dir: 616 tests, every failure a `history_purge` test (`the_harness_records_history_at_all`, `the_emitted_zsh_recipe_actually_purges_the_entry`, `editing_the_file_alone_is_the_trap_the_message_warns_about`) — the three fail identically on untouched `master` (no `/usr/bin/zsh` on this box), so they are not this task's — see the r1 fold commit for the exact summary line; clippy's one `manual implementation of .is_multiple_of()` in `sysw/composer_records.rs:114` is the local nightly's, green in CI at `917d4e3`.

```bash
git add crates/me-cli/testdata/codex32_seam_vectors.json crates/me-cli/testdata/record_corpus_pre_s2.json crates/me-cli/tests/codex32_seam.rs crates/me-cli/tests/preimage_plate_is_not_a_seed.rs crates/me-cli/src/seal/record.rs crates/me-cli/src/sysw/mod.rs crates/me-cli/src/main.rs
git commit -m "seam corpus: preimage-plate rows (the plate, two 0x03-leading populations the guard must not touch, the shape under the wrong id) + host pin + preimage-plate diagnosis (hashlock H0)"
```

---

### Task 2: The device guard (seedhammer fork)

**Files:**
- Modify: `codex32/mspayload.go` (the const block at lines 8-12; append `IsPreimage`)
- Modify: `codex32/mspayload_test.go` (append)
- Modify: `sysw/classify.go` (`isStrictMs1`'s last two lines, `_, err := codex32.New(record)` / `return err == nil`)
- Replace: `sysw/testdata/codex32_seam_vectors.json` (vendored copy of Task 1's file)
- Modify: `sysw/codex32_seam_test.go:30` (anchor: `const seamVectorsSHA256 = `)
- Modify: `seal/record.go:212-214` (anchor: `if _, err := codex32.New(s); err == nil {` … `return ClassCodex32Secret`)
- Modify: `seal/record_test.go` (row in `TestClassifyMirrorsScanBranchOrder` after `{d.Public[2], ClassMDMK}, // md1`; new test)
- Modify: `gui/scan.go:89` (anchor: `} else if s, err := codex32.New(string(buf)); err == nil {`)
- Modify: `gui/codex32_polish.go` (anchor: `func engraveCodex32(ctx *Context, th *Colors, scan codex32.String) bool {` + `	for {`)
- Modify: `gui/codex32_polish_test.go` (add `"errors"` to the import block; append)
- Modify: `gui/unlock_session.go` (`unlockEngraveCodex32`, before `id, _, _ := s.Split()`)
- Modify: `gui/unlock_session_test.go` (append)

**Interfaces:**
- Consumes: `codex32.String` (`Seed() []byte`, `String() string`, `Split()`), `codex32.ParsePrefix(frag string) (Fields, error)` (`Fields.Unshared`), `seal.AdmitSection([][]byte, Section) ([]AdmittedRecord, error)`, `seal.ErrRecordNotPermitted`, `sysw.Classify(string) Class`, `(*scanner).Scan(io.Reader) (any, error)`, `errScanUnknownFormat`, `errScanInProgress`, `engraveObjectFlow`, `runUITouch`, `sessionHarness`.
- Produces: `codex32.IsPreimage(s String) bool`.

Work on branch `hashlock-h0` from fork `main` (`839fa5aa`). Go is
`/scratch/code/shibboleth/.toolchain/go/bin/go`.

- [ ] **Step 1: Vendor the corpus and re-pin; watch the seam test go RED for the right reason.**

```bash
cp ../mnemonic-engrave/crates/me-cli/testdata/codex32_seam_vectors.json sysw/testdata/codex32_seam_vectors.json
sha256sum sysw/testdata/codex32_seam_vectors.json   # f1f2fa6b…391c
```

Replace `sysw/codex32_seam_test.go:30`:

```go
const seamVectorsSHA256 = "f1f2fa6bbbf27e3697ee496636de49be2f25787deff7b3bc4a2c5e16854e391c"
```

Run: `go test -count=1 -run TestCodex32SeamDeviceAdmitsEverythingTheHostDoes ./sysw/`
Expected: FAIL on exactly the two rows the device must refuse and does not yet: `preimage-plate-0x03: device admits = true, want false (Classify = 2)` and `preimage-shape-entr-id: device admits = true, want false (Classify = 2)` (`sysw.Class` has no `String()`; 2 is `ClassCodex32Secret`). The two `0x03`-leading `device_admits: true` rows pass already and must keep passing. This is the spec's reader-table measurement as a failing test.

- [ ] **Step 2: The predicate.** In `codex32/mspayload.go` replace the const block:

```go
const (
	msPrefixEntr     = 0x00 // RESERVED_PREFIX: payload = [0x00][entropy]
	msPrefixMnem     = 0x02 // MNEM_PREFIX:     payload = [0x02][language][entropy]
	msPrefixPreimage = 0x03 // PREIMAGE_PREFIX: payload = [0x03][32-byte hashlock preimage] (SPEC_ms_hashlock §1)
	msMaxLanguage    = 9    // MNEM_LANGUAGE_NAMES indices 0..9
)
```

and append at the end of the file:

```go
// IsPreimage reports whether a New-valid string carries the m-format HASHLOCK
// PREIMAGE kind (SPEC_ms_hashlock §1: payload = [0x03][32 bytes], id `hash`).
//
// H0 (SPEC_ms_hashlock §9): such a string is INERT on this device — never a
// codex32 SECRET and no class of its own — because every path that admits
// ClassCodex32Secret ends at backup.EngraveSeedString, and a hashlock
// preimage is not a seed: engraved as one it exposes a spend secret as a
// backup. DecodeMS1 is deliberately unchanged and still refuses the prefix;
// the device learns to USE a preimage in stage H2, not here.
//
// The question is "is this a preimage SINGLE", not "does some byte equal 3":
// the check is singles-only (§1 rule 2 -- a share's data part is an SSS
// point, and its first byte is whatever the polynomial gave it), and the
// preimage payload is exactly [0x03][32 bytes]. A plain BIP-93 secret whose
// seed begins 0x03 has a 16..32-byte payload and is untouched. The id is NOT
// consulted: the kind is the prefix byte (§1), a 0x03 single under any other
// id is a mismatch the host refuses, and it is not a seed either way.
//
// Reads the prefix fields and one payload byte; nothing new is retained.
func IsPreimage(s String) bool {
	f, err := ParsePrefix(s.String())
	if err != nil || !f.Unshared {
		return false
	}
	d := s.Seed()
	return len(d) == 33 && d[0] == msPrefixPreimage
}
```

- [ ] **Step 3: Its test.** Append to `codex32/mspayload_test.go`:

```go
// H0: the preimage kind is recognised by its shape and prefix byte and by
// nothing else, and DecodeMS1 keeps refusing it (the seed decoder must not
// learn a kind that is not a seed).
func TestIsPreimageReadsThePrefixByteOnly(t *testing.T) {
	const plate = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c"
	s, err := New(plate)
	if err != nil {
		t.Fatalf("New(plate): %v", err)
	}
	if !IsPreimage(s) {
		t.Fatalf("IsPreimage(plate) = false, want true (Seed()[0] = %#x)", s.Seed()[0])
	}
	if id, _, _ := s.Split(); id != "hash" {
		t.Errorf("id = %q, want hash", id)
	}
	// Every population the predicate must NOT touch, and the one it must.
	// MUTATIONS, each measured against exactly one row: dropping `!f.Unshared`
	// calls the share row a preimage; dropping `len(d) == 33` calls the plain
	// 16-byte BIP-93 row one; `d[0] != msPrefixEntr` in place of
	// `== msPrefixPreimage` calls the 33-byte 0x31 row one; keying on the id
	// `hash` instead of the prefix misses the entr-id row. The mnem row is
	// 17 bytes and is refused by the length test alone. Seam-corpus rows
	// where one exists (sysw/testdata/codex32_seam_vectors.json); the 0x31
	// row is codex32.NewSeed("ms", 0, "test", 's', 33 bytes beginning 0x31).
	for _, c := range []struct {
		name, s string
		want    bool
	}{
		{"constellation-entr-128 (prefix 0x00)", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f", false},
		{"mnem-english16 (prefix 0x02)", "ms10entrsqgqqc83yukgh23xkvmp59xf2eldpk4cdrq2y4h82yz", false},
		{"bip93-plain-payload-0x03 (16-byte seed beginning 0x03)", "ms10testsqv0qqqqqqqqqqqqqqqqqqqqqqq8mzk8tjfdnjn5", false},
		{"bip93-share-payload-0x03 (a 2-of-N share beginning 0x03)", "ms12testaqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdq7pl8qdc5tsp", false},
		{"bip93-plain-33-byte-payload-0x31 (unshared, 33 bytes, first byte 0x31)", "ms10testsxy0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5dayejmh0wrfk", false},
		{"preimage-shape-entr-id (unshared, 33 bytes, 0x03, id entr)", "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9", true},
	} {
		e, err := New(c.s)
		if err != nil {
			t.Fatalf("New(%s): %v", c.name, err)
		}
		if got := IsPreimage(e); got != c.want {
			t.Errorf("IsPreimage(%s) = %v, want %v", c.name, got, c.want)
		}
	}
	if _, _, _, err := DecodeMS1(s); err != errMSBadPrefix {
		t.Errorf("DecodeMS1(plate) err = %v, want errMSBadPrefix: the seed decoder must not decode a preimage", err)
	}
}
```

Run: `go test -count=1 -run TestIsPreimageReadsThePrefixByteOnly ./codex32/`
Expected: PASS. Mutations, each measured at the gate and reverted: drop `!f.Unshared` (write `_ = f.Unshared; if err != nil {` so it compiles) → `IsPreimage(bip93-share-payload-0x03 …) = true, want false`; `len(d) > 0` for `len(d) == 33` → `IsPreimage(bip93-plain-payload-0x03 …) = true, want false`; `d[0] != msPrefixEntr` → `IsPreimage(bip93-plain-33-byte-payload-0x31 …) = true, want false`; `id, _, _ := s.Split(); return id == "hash"` → `IsPreimage(preimage-shape-entr-id …) = false, want true`. (The 23-character `ms10entrsqqg5y2z9pzs3gg` that `seal/record_test.go:441` uses for `wipe` is NOT `New`-valid — `codex32: invalid length` — which is why the table uses corpus strings.)

- [ ] **Step 4: `isStrictMs1`.** Replace `sysw/classify.go`'s last two lines of `isStrictMs1` (`_, err := codex32.New(record)` / `return err == nil`):

```go
	c, err := codex32.New(record)
	// H0 (SPEC_ms_hashlock §9): a hashlock preimage plate is BCH-valid and
	// inside the cap, and it is not a seed. Inert here — no class of its own.
	return err == nil && !codex32.IsPreimage(c)
```

Run: `go test -count=1 ./sysw/`
Expected: PASS, the seam test included (12 rows: 2 both / 6 device-only / 4 neither). Mutation, measured: with the `!codex32.IsPreimage(c)` clause removed again, the seam test fails on exactly the Step 1 lines.

- [ ] **Step 5: `seal.Classify`.** Replace `seal/record.go:212-214` (anchor `if _, err := codex32.New(s); err == nil {` … `return ClassCodex32Secret` … `}`; the `ValidMD || ValidMK` branch that follows is untouched):

```go
	if c, err := codex32.New(s); err == nil && !codex32.IsPreimage(c) {
		return ClassCodex32Secret
	}
```

- [ ] **Step 6: seal tests.** In `TestClassifyMirrorsScanBranchOrder` add a row after `{d.Public[2], ClassMDMK}, // md1`:

```go
		{sealPreimagePlate, ClassUnknown}, // H0: a hashlock preimage plate is not a secret and has no class
```

and append to `seal/record_test.go` (`errors` and `strings` are already imported, lines 4-5):

```go
// H0 (SPEC_ms_hashlock §9): the 75-character kind-0x03 preimage plate from the
// shared seam corpus (sysw/testdata/codex32_seam_vectors.json, row
// preimage-plate-0x03). BCH-valid, inside the cap, and NOT a seed.
const sealPreimagePlate = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c"

// An encrypted section carrying a preimage plate is refused whole, exactly as
// one carrying any unknown record is — the payload never reaches the unlock
// screen with an "ms1" it would cut as a seed. (The unlock screen renders this
// as "Payload unreadable."; a named arm is an H2 follow-up.)
func TestAdmitSectionRefusesAPreimagePlateAsUnknown(t *testing.T) {
	_, err := AdmitSection([][]byte{[]byte(sealPreimagePlate)}, SectionEncrypted)
	if !errors.Is(err, ErrRecordNotPermitted) {
		t.Fatalf("AdmitSection(preimage plate, encrypted) err = %v, want ErrRecordNotPermitted", err)
	}
	if !strings.Contains(err.Error(), "unknown") {
		t.Errorf("error %q does not name the class unknown", err)
	}
	// MUTATION: drop `!codex32.IsPreimage(c)` from Classify -> admitted as
	// codex32-secret, err == nil, this test fails on the first check.
}
```

Run: `go test -count=1 ./seal/`
Expected: PASS. Mutation, measured: with Step 5's clause removed, `TestClassifyMirrorsScanBranchOrder` fails with `Classify("ms10hashsqw46h2at4w46h2a") = codex32 secret, want unknown format` and `TestAdmitSectionRefusesAPreimagePlateAsUnknown` with `err = <nil>, want ErrRecordNotPermitted`.

- [ ] **Step 7: The sealed engrave path.** In `gui/unlock_session.go`, `unlockEngraveCodex32`, after the `codex32.New` error check and before `id, _, _ := s.Split()`:

```go
	if codex32.IsPreimage(s) {
		// Unreachable behind seal.Classify's H0 guard, which never admits a
		// preimage plate as ClassCodex32Secret. Named rather than assumed:
		// this is the one call on the sealed path that cuts metal.
		showError(ctx, th, unlockTitle, "This record is a hashlock preimage, not a seed. It is not engraved as one.")
		return
	}
```

Append to `gui/unlock_session_test.go` (the harness is `runUnlockEngraveMnemonic` at `:714`; this is its twin):

```go
// runUnlockEngraveCodex32 is runUnlockEngraveMnemonic's twin for the ms1 arm.
func runUnlockEngraveCodex32(t *testing.T, pf Platform, rec []byte) *sessionHarness {
	t.Helper()
	ctx := NewContext(pf)
	returned := false
	frame, drawer, quit := runUITouch(ctx, func() {
		unlockEngraveCodex32(ctx, &descriptorTheme, rec)
		returned = true
	})
	h := &sessionHarness{t: t, ctx: ctx, done: &returned}
	h.frame, h.drawer = frame, drawer
	t.Cleanup(quit)
	return h
}

// H0 (SPEC_ms_hashlock §9), defence in depth: even if a kind-0x03 preimage
// plate reached the sealed path's cut, it is refused by name and no engrave
// screen is shown. seal.Classify never admits one, so this is the second
// guard, not the first.
func TestUnlockEngraveCodex32RefusesAPreimagePlate(t *testing.T) {
	const plate = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c"
	h := runUnlockEngraveCodex32(t, newPlatform(), []byte(plate))
	// MUTATION: drop the IsPreimage check in unlockEngraveCodex32 -> the flow
	// reaches the EngraveSeed screen for the plate, and this never sees the
	// refusal text.
	h.mustReach("hashlock preimage")
}
```

Run: `go test -count=1 -run 'TestUnlock' ./gui/`
Expected: PASS. Mutation, measured: with the guard removed the new test fails with `never reached "hashlock preimage"; last frame "Insert a blank plate and close the lock. Hold button to start the engraving process. ... Engrave Plate"` — the device, handed a preimage plate through this path, would cut it.

- [ ] **Step 8: The two direct doors (fidelity C-1).** Both the NFC door (`gui/nfc_scan.go` → `gui/scan.go:89`) and the typed `M*1 STRING` door (`inputCodex32Flow` → `validateMStar`, `gui/codex32_polish.go:266`) hand a `codex32.String` to `engraveObjectFlow` (`gui/gui.go:2556`), which calls `engraveCodex32` → `confirmCodex32Flow` ("Confirm Codex32 Secret / Unshared secret (S)") → `backupSeedStringFlow` → `backup.EngraveSeedString`, titled `HASH`. `DecodeMS1` never runs on that path. Two edits:

(a) `gui/scan.go:89` — the NFC classifier is the documented mirror of `seal.Classify` (`seal/record.go:95-98`, "must stay in step"), so it narrows with it. Replace the arm:

```go
	} else if s, err := codex32.New(string(buf)); err == nil && !codex32.IsPreimage(s) {
		// H0 (SPEC_ms_hashlock §9): a hashlock preimage plate is not a seed;
		// seal.Classify mirrors this arm and narrows with it.
		return s, nil
	} else if codex32.ValidMD(string(buf)) || codex32.ValidMK(string(buf)) {
```

A scanned preimage plate then falls through to `errScanUnknownFormat` ("Unknown format" on screen): a refusal, not a named one. The named refusal is at the choke point.

(b) `gui/codex32_polish.go`, `engraveCodex32` — the choke point. Insert before the `for {`:

```go
func engraveCodex32(ctx *Context, th *Colors, scan codex32.String) bool {
	if codex32.IsPreimage(scan) {
		// H0 (SPEC_ms_hashlock §9). Both doors that hand a codex32.String to
		// engraveObjectFlow -- the NFC scan and the typed M*1 STRING -- end
		// here, and this is the call that titles the plate and cuts it. The
		// scan door already refuses upstream; the typed door does not, so the
		// named refusal lives at the choke point.
		showError(ctx, th, "Hashlock preimage", "This record is a hashlock preimage, not a seed. It is not engraved as one.")
		return true
	}
	for {
```

(`return true`: recognised and handled, like a Back at the confirm screen — `engraveObjectFlow`'s `false` means "Unknown format", which would be a lie here.)

Tests: add `"errors"` to `gui/codex32_polish_test.go`'s import block (it has `"strings"` and `"testing"`), then append:

```go
// H0 (SPEC_ms_hashlock §9): a kind-0x03 preimage plate handed to the engrave
// dispatch -- the object both the NFC door and the typed M*1 STRING door
// produce -- is refused by name and never reaches the codex32 confirm screen.
func TestEngraveCodex32RefusesAPreimagePlate(t *testing.T) {
	s, err := codex32.New("ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c")
	if err != nil {
		t.Fatalf("New: %v", err)
	}
	ctx := NewContext(newPlatform())
	returned := false
	frame, drawer, quit := runUITouch(ctx, func() {
		engraveObjectFlow(ctx, &descriptorTheme, s)
		returned = true
	})
	h := &sessionHarness{t: t, ctx: ctx, done: &returned}
	h.frame, h.drawer = frame, drawer
	t.Cleanup(quit)
	// MUTATION: drop the IsPreimage check in engraveCodex32 -> the flow shows
	// "Confirm Codex32 Secret" for the plate and never this text.
	h.mustReach("hashlock preimage")
}

// The NFC door: Scan classifies the plate as no known object, exactly as
// seal.Classify does (the two are documented mirrors), so it never becomes a
// codex32.String for engraveObjectFlow. The typed door has no such gate and
// relies on engraveCodex32's refusal above.
func TestScanDoesNotHandAPreimagePlateToEngrave(t *testing.T) {
	const plate = "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c"
	// Scan reads until io.EOF; a strings.Reader delivers that on its second Read.
	scanAll := func(s string) (any, error) {
		sc := &scanner{}
		r := strings.NewReader(s)
		for {
			obj, err := sc.Scan(r)
			if !errors.Is(err, errScanInProgress) {
				return obj, err
			}
		}
	}
	obj, err := scanAll(plate)
	if !errors.Is(err, errScanUnknownFormat) {
		t.Fatalf("Scan(preimage plate) = %T, %v; want errScanUnknownFormat", obj, err)
	}
	// And the legitimate populations the guard must not touch still scan.
	for _, s := range []string{
		"ms10testsqv0qqqqqqqqqqqqqqqqqqqqqqq8mzk8tjfdnjn5",                            // plain BIP-93, seed begins 0x03
		"ms12testaqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdq7pl8qdc5tsp", // a share beginning 0x03
	} {
		obj, err := scanAll(s)
		if _, ok := obj.(codex32.String); err != nil || !ok {
			t.Errorf("Scan(%.20s...) = %T, %v; want a codex32.String", s, obj, err)
		}
	}
}
```

Run: `go test -count=1 -run 'TestEngraveCodex32|TestScan|TestConfirmCodex32' ./gui/`
Expected: PASS (measured; `TestEngraveCodex32BackoutNotUnknown` and the share tests are unaffected). Mutations, measured: remove (b) → `TestEngraveCodex32RefusesAPreimagePlate` fails with `never reached "hashlock preimage"; last frame "ConfirmCodex32SecretidHASHUnsharedsecret(S)75chars"` — the plate confirmed as a secret titled HASH, one button from the cut; remove (a) → `TestScanDoesNotHandAPreimagePlateToEngrave` fails with `Scan(preimage plate) = codex32.String, <nil>; want errScanUnknownFormat`.

- [ ] **Step 9: Whole surface, then commit.**

Run: `go vet ./codex32/ ./sysw/ ./seal/ && go test -count=1 ./codex32/ ./sysw/ ./seal/ && ../mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`
Expected: all green. (`go vet ./gui/` under 1.26.7 reports two PRE-EXISTING complaints, `testing.ArtifactDir requires go1.26 or later (file is go1.25)` in `freetext_sizeproof_golden_test.go:111` and `transaction_golden_test.go:104`; on `main` today, not this task's.)

```bash
git add codex32/mspayload.go codex32/mspayload_test.go sysw/classify.go sysw/testdata/codex32_seam_vectors.json sysw/codex32_seam_test.go seal/record.go seal/record_test.go gui/scan.go gui/codex32_polish.go gui/codex32_polish_test.go gui/unlock_session.go gui/unlock_session_test.go
git commit -s -m "hashlock H0: a kind-0x03 preimage single is inert -- both classifiers, the scan mirror, and the engrave choke point refuse it; shares and plain BIP-93 secrets untouched"
```

---

### Task 3: Size, acceptance, flash (operator's word required for the flash)

- [ ] **Step 1: Measure the firmware.** From the fork checkout with nix on `PATH`:

Run: `nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`
Expected: flash within a few hundred bytes of `1,582,628` (main `839fa5aa`); RAM `62,800`. Record both in the fold.

- [ ] **Step 2: Acceptance before the flash (fidelity I-4).** The two direct doors need no payload, so they are accepted TODAY: (a) the executable half is Task 2 Step 8's two tests plus Step 7's; (b) the walked half is one emulator session (`cmd/emu` is the shipped `gui` package built for `GOOS=js`; run it the way the S-stage continuity records) typing the 75-character plate at `M*1 STRING` — expect the "Hashlock preimage" error screen and no "Confirm Codex32 Secret" — and presenting it over the emulator's NFC — expect "Unknown format". Record both outcomes, with the frame text, in the continuity entry before the flash. The sysw container half can also be walked with a blob from `cmd/buildpayloadcomposer` (in-tree, no `me`): the record loads, is counted "inert", and is offered by no program. Only the SEALED half waits for H2, because `me seal` refuses to pack the record at 0.7.

- [ ] **Step 3: Merge to fork `main`** after the post-implementation review (Task 4) is GREEN, then flash: `~/bin/sh/sh2-flash -y` **only at the operator's word**, with the SH2 in BOOTSEL. Boot judgement is the operator's (PD negotiation can be slow: a dark screen is not a rejected signature).

---

### Task 4: Records and review

- [ ] `design/FOLLOWUPS.md`, three entries: (1) should the device converge fully on the constellation profile (refuse plain BIP-93 at 48/74 characters and shares of foreign ids)? Owning phase: none — operator decision; the seam corpus's `device_admits: true` rows are the current answer. (2) `me`'s ms-codec 0.8 bump (H1b) MUST add an explicit `Payload::Preimage` refusal arm in `validate_record` AND re-point `seal::record::preimage_plate` at it (at 0.8 `decode` no longer returns `ReservedPrefixViolation` for `0x03`); both Task 1 tests are the tripwire. Owning phase: H1b. (3) `ErrRecordNotPermitted` gets its own arm in `gui/unlock_kdf.go` naming the record index and class, on the argument the neighbouring `ErrTooManyRecords`/`ErrCodex32TooLong` arms already carry — today a payload with a preimage record says only "Payload unreadable." Owning phase: H2.
- [ ] `crates/me-cli/CHANGELOG.md` (unreleased): the four corpus rows, the pin test, the `PreimagePlate` diagnosis. Fork `CHANGELOG` if it keeps one.
- [ ] Post-implementation review (risk set: readers that cut seeds): ONE opus adversarial execution review over the whole diff of both repos, brief `design/agent-briefs/hashlock-H0-post-impl-brief.md`, report `design/agent-reports/hashlock-H0-post-impl.md`; GREEN before the fork merge and the flash.
- [ ] Continuity entry + memory; push engrave via `scripts/push-via-staging.sh master`; fork via its normal PR to `bg002h/seedhammer` main.

---

## Self-review

1. **Spec coverage.** §9 H0 (a): `isStrictMs1` / `seal.Classify` gain the test — Task 2 Steps 4-5 — and §12 item 7 ("no engrave path offers it") is met by guarding what the site enumeration found: the sealed path (Step 7), the NFC mirror (Step 8a) and the choke point both direct doors reach (Step 8b). "With the record-class vector row" — Task 1's rows, read by both suites. "Merged and flashed" — Task 3, now with an acceptance before the flash. §9 H0 (b): `me`'s `validate_record` treats `0x03` as inert "in the same release window as the 0.8 bump" — Task 1's pin test is green at 0.7 and is the tripwire; the refusing arm belongs to the bump (an H1b-owned follow-up, Task 4), because at 0.7 there is no `Payload::Preimage` to match; and the host now NAMES the kind instead of misdiagnosing the profile (Step 7).
2. **Placeholders.** None: the fixture question and the gui harness were settled at the first gate; the door tests and the diagnosis are written out.
3. **Type consistency.** `IsPreimage(s String) bool` is defined once (Task 2 Step 2) and called with a `codex32.String` at five sites (`isStrictMs1`, `seal.Classify`, `Scan`, `engraveCodex32`, `unlockEngraveCodex32`); `ParsePrefix(frag string) (Fields, error)` with `Fields.Unshared bool` (`codex32/polish.go:82,71`); `Seed()` is `func (s String) Seed() []byte` (`codex32/codex32.go:386`); `AdmitSection(records [][]byte, section Section) ([]AdmittedRecord, error)` (`seal/record.go:244`); `validate_record(s: &str) -> Result<RecordKind, RecordError>` (`record.rs:117`); `preimage_plate(s: &str) -> bool` beside `bip93_outside_the_profile` (`record.rs:204`); `pack(vec![...], None, ITER)` returns `Result<_, SyswError>` as the neighbouring test uses it.

**R0 round 1 folded here.** The new Important: the round-0 fold's "whole crate 615/616" was false — nextest without `--no-fail-fast` stopped after the first failing binary, hiding `record_corpus.rs`'s three failures (33 vs 37) that Task 1 Step 1's rows cause. Step 1b now extends the capture with the argument invariant 2 demands; the File Structure table and Step 9's `git add` carry the file; Step 9 and a Global Constraint fix the measurement method (`--no-fail-fast`, own target dir, `touch` after a restore).

**R0 round 0 folded here.** Fidelity: C-1 (two unguarded doors) → Task 2 Step 8 (Scan mirror + choke point + two tests) and the site table; C-2 (any-byte predicate) → the singles-only, shape-exact predicate, two `device_admits: true` corpus rows whose payload begins `0x03`, the six-population test; I-1 (three anchors) → re-cited with anchor text (`codex32_seam.rs:25-26`, `codex32_seam_test.go:30`, `seal/record.go:212-214`) and a Global Constraint; I-2 (the IsPreimage test could not catch its own mutation) → the table, with every mutation measured against one row (and a 33-byte `0x31` row added when the `!= msPrefixEntr` mutation survived the first table); I-3 (`me` misdiagnoses the plate) → `preimage_plate` + `UnknownReason::PreimagePlate` + the Display text + a unit test and a binary test; I-4 (cheaper acceptance) → Task 3 Step 2; I-5 ("Payload unreadable." unrecorded) → Global Constraints and a Task 4 follow-up owned by H2; M-1 (`:177`) → `:176`; M-2 (the two containers diverge) → Global Constraints. Tests lens: I-1 (prefix vs id indistinguishable) → the `preimage-shape-entr-id` row in the corpus and the table, with the id-keyed mutation measured; M-1 (stale citations) → fixed with I-1; M-2/M-3 recorded.
