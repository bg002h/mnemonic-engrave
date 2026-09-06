# F-503 Implementation Plan — the preimage KIND under the id `hash` is inert at every length

**STATUS: GATED, R0 SKIPPED by the operator (2026-09-06: "Approve, but skip the R0 round"); ready for ONE implementer.** Written by the controller from a scratch implementation in two gate worktrees; every code block below is the verbatim text that was built and tested there (gate results in §5), so the implementer's job is transcription with the RED/GREEN/mutation steps run again on real branches.

> **For agentic workers:** execute the tasks in order (Rust first — the Rust-primary rule — then the fork as a CONVERGENCE port, then records). Steps use checkbox syntax.

**Goal:** a kind-`0x03` ms1 single under the id `hash` whose X is not 32 bytes is refused BY NAME on the host and is `ClassUnknown` on the device — never a seed on either side — while every other id keeps H0's BIP-93-wide device rule.

**Spec:** engrave `design/FOLLOWUPS.md` F-503; `mnemonic-secret/design/SPEC_ms_hashlock.md` §1 (the kind byte), §9 (H0's inertness), §12 item 7; the operator's two rulings of 2026-09-06 recorded in `design/CONTINUITY_composer_2026-09-01.md`:
1. **Device rule:** the 0x03 KIND under the id `hash`, at any length, is inert (`ClassUnknown`). Not "reserve the whole id" — a kind-0x00 payload under `hash` stays a device-side seed (host: `TagKindMismatch`). Not "kind 0x03 at any length under any id" — that would flip the pinned seam row `bip93-plain-payload-0x03` (a 16-byte 0x03 payload under `test`, device-admits TRUE by H0's design).
2. **me release:** one `me` release AFTER this lands (0.9.0: H6's host half + this). Not part of this plan; its own brief.

**Why the host side changes too (Rust-first, and a wording defect):** `me` already refuses the F-503 string (`ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqmv3lqlgkn6s5c`, 50 characters, exit 4) — but its refusal says *"whose 4-character id is not `hash`"*, which is false (the id IS `hash`; the X is 16 bytes), and ends with the 1-in-256 collision sentence, which is true only of a 33-byte payload. The `UnknownReason::PreimagePlate` reason carried no data (an earlier review's M-1 chose that because "no number would be true of every case"); the fix carries the diagnosis PER RECORD so the text says only what is true of the record in hand.

**Baselines:** engrave master `db37ec79`; fork main `3cadffa8`; ms master `a994a99` (ms-codec 0.9.0 published). Gate worktrees the blocks were extracted from: `/scratch/code/shibboleth/me-worktrees/f503-gate` (detached at db37ec79) and `/scratch/code/shibboleth/.tmp/seedhammer-f503-gate` (detached at 3cadffa8) — READ-ONLY for the implementer; the checker `scripts/h6-plan-blocks-vs-tree.sh <this plan> <fork tree> <any ms checkout> <me tree>` proves the blocks against them (§5).

## Global constraints
- Cargo env: `PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/<own dir>`; `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast`; `cargo fmt --all -- --check` before every commit; clippy under the box's rustc 1.98 warns ONLY on the pre-existing `manual_is_multiple_of` in `composer_records.rs` (known; not yours).
- Go: `/scratch/code/shibboleth/.toolchain/go/bin/go` first on PATH; whole-gui counts only via `scripts/gui-shard-test.sh ./gui/ 24` from the fork worktree.
- The three `history_purge` tests fail on this box for want of `/usr/bin/zsh` (F-500); the me suite's expected figure is **634 run: 631 passed, 3 failed, 2 skipped**.
- Commit per task, `git commit -F <file>` (`-s` on the fork), trailers `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>` and `Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA`; stage paths explicitly; nothing pushed; no `master`/`main` commits.
- The seam corpus is SHARED BYTE FOR BYTE (its own header says how): edit it in me, re-pin `SEAM_VECTORS_SHA256` (Rust) AND `seamVectorsSHA256` (Go) to the same value, copy the file to the fork. The value for this change is `f53a17dc9d1ea5a4c0ff913786e82a5f19de004f4719536bb6d957bccdef1ee8`.

## File structure
| repo | file | change | task |
| --- | --- | --- | --- |
| me | `crates/me-cli/src/sysw/mod.rs` | `UnknownReason::PreimagePlate` becomes a struct variant carrying `id_is_hash`/`x_len`; the naming arm computes both; six test patterns follow | 1 |
| me | `crates/me-cli/src/main.rs` | the `U::PreimagePlate` arm renders three truthful bodies | 1 |
| me | `crates/me-cli/tests/sysw_pack_preimage.rs` | new test `a_hash_id_plate_with_a_short_x_is_refused_for_its_length_not_its_id` | 1 |
| me | `crates/me-cli/testdata/codex32_seam_vectors.json` | four rows appended (three F-503 shapes, one control) | 1 |
| me | `crates/me-cli/tests/codex32_seam.rs` | sha re-pin | 1 |
| me | `crates/me-cli/testdata/record_corpus_pre_s2.json` | four capture entries, after the last `codex32_seam/` entry (precedent: be72e75a, 8018873c, 4d00fbbf) | 1 |
| me | `crates/me-cli/CHANGELOG.md` | `[Unreleased]` entry | 1 |
| fork | `codex32/mspayload.go` | `IsPreimageKind`; the H0 comment's "untouched" sentence | 2 |
| fork | `codex32/mspayload_test.go` | new test `TestIsPreimageKindIsTheKindAtEveryLength` | 2 |
| fork | `sysw/classify.go` | `isStrictMs1` gains the conjunct; `isHashIdPreimageKind` | 2 |
| fork | `sysw/testdata/codex32_seam_vectors.json` | vendored copy (byte-identical) | 2 |
| fork | `sysw/codex32_seam_test.go` | sha re-pin | 2 |
| ms | `design/SPEC_ms_hashlock.md` | §9 and §12 item 7 state the precise device rule | 3 |
| engrave | `design/FOLLOWUPS.md`, `design/CONTINUITY_composer_2026-09-01.md` | F-503 closed with SHAs; continuity | 3 |

Branches: me `f503` off engrave master; fork `f503` off main; ms `f503-records` off master. Worktrees: `/scratch/code/shibboleth/me-worktrees/f503`, `/scratch/code/shibboleth/.tmp/seedhammer-f503`, `/scratch/code/shibboleth/ms-worktrees/f503-records`.

---

### Task 1: `me` — the reason carries the diagnosis; the seam rows; the capture

**Files:** Modify `crates/me-cli/src/sysw/mod.rs`, `crates/me-cli/src/main.rs`, `crates/me-cli/tests/sysw_pack_preimage.rs`, `crates/me-cli/testdata/codex32_seam_vectors.json`, `crates/me-cli/tests/codex32_seam.rs`, `crates/me-cli/testdata/record_corpus_pre_s2.json`, `crates/me-cli/CHANGELOG.md`.

**Interfaces — Produces:** `mnemonic_engrave::sysw::UnknownReason::PreimagePlate { id_is_hash: bool, x_len: Option<usize> }` (was a unit variant). Consumers inside the crate only (main.rs, the sysw tests).

- [ ] **Step 1: the failing test (RED).** Append to `crates/me-cli/tests/sysw_pack_preimage.rs`:

```rust file=me/crates/me-cli/tests/sysw_pack_preimage.rs mode=fragment
/// F-503: the refusal names the conjunct that failed. The F-503 string is the
/// kind 0x03 under the id `hash` with an X of 16 bytes -- `ms10hashsqv...` at
/// 50 characters -- and before F-503 the body said its "4-character id is not
/// `hash`", which was false, and ended with the 1-in-256 collision sentence,
/// which is true only of a 33-byte payload. Now it says what is wrong (the X
/// is 16 bytes, not 32) and what to do (re-encode with `ms hashlock`), and it
/// says neither false thing. WITH and WITHOUT the flag, one refusal.
///
/// MUTATION: make `x_len` always `None` in the naming arm -> the body falls
/// back to the wrong-id sentence, and this fails on the first assertion.
#[test]
fn a_hash_id_plate_with_a_short_x_is_refused_for_its_length_not_its_id() {
    const HASH_16: &str = "ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqmv3lqlgkn6s5c";
    for extra in [
        vec!["--no-passphrase"],
        vec!["--no-passphrase", "--pack-preimage"],
    ] {
        let o = run_with(&extra, &[HASH_16]);
        assert!(!o.status.success(), "the short-X plate packed: {extra:?}");
        let e = stderr(&o);
        assert!(
            e.contains("under the id `hash` whose X is 16 bytes, not 32"),
            "F-503: the length is not named: {e}"
        );
        assert!(
            !e.contains("4-character id is not `hash`"),
            "F-503: the id IS `hash`, and the body still says it is not: {e}"
        );
        assert!(
            !e.contains("roughly 1 in 256"),
            "the collision sentence is true only of a 33-byte payload: {e}"
        );
        assert!(
            e.contains("re-encode it with `ms hashlock`"),
            "the remedy is missing: {e}"
        );
        assert_eq!(
            e.matches("records count from 0").count(),
            1,
            "one refusal, not several: {e}"
        );
    }
}
```

Run: `cargo nextest run --locked -p mnemonic-engrave -E 'test(a_hash_id_plate_with_a_short_x)'`
Expected: FAIL — `F-503: the length is not named: me: record 0 (records count from 0) is a kind-0x03 preimage payload whose 4-character id is not \`hash\`. ...` (measured in the gate with the arm reverted: 2 tests run, 0 passed, 2 failed — the unit test in Step 3 reds too).

- [ ] **Step 2: the variant carries the diagnosis.** In `crates/me-cli/src/sysw/mod.rs`, replace the `PreimagePlate` doc comment and unit variant with:

```rust file=me/crates/me-cli/src/sysw/mod.rs mode=fragment
    /// A string of the hashlock PREIMAGE kind (SPEC_ms_hashlock §1, kind
    /// `0x03`): inside the profile's lengths, refused for its KIND. Named so
    /// the operator is not told to "re-encode the entropy as `ms1`" — the
    /// string is a constellation record, just not one this container places
    /// yet (H0, §9).
    ///
    /// Carries WHICH conjunct of the admission shape failed, so the refusal
    /// can say what to fix (F-503: the text used to claim the id was not
    /// `hash` for a string whose id WAS `hash` and whose X was 16 bytes).
    /// `id_is_hash` is the id read from bytes 4..8 of the trimmed record;
    /// `x_len` is `Some(n)` when the codec refused the payload as
    /// `PreimageLengthMismatch { got: n }` (an X of `n` bytes, not 32) and
    /// `None` when the payload is a well-formed 33-byte kind under a wrong id.
    /// Both false-shaped (`id_is_hash: true, x_len: None`) cannot occur: that
    /// string is admissible and never reaches here.
    PreimagePlate {
        id_is_hash: bool,
        x_len: Option<usize>,
    },
```

and replace the naming arm with:

```rust file=me/crates/me-cli/src/sysw/mod.rs mode=fragment
    if crate::seal::record::preimage_plate(record) {
        let trimmed = record.trim();
        let id_is_hash = trimmed.as_bytes().get(4..8) == Some(b"hash".as_slice());
        let x_len = match ms_codec::decode(trimmed) {
            Err(ms_codec::Error::PreimageLengthMismatch { got }) => Some(got),
            _ => None,
        };
        return UnknownReason::PreimagePlate { id_is_hash, x_len };
    }
```

Then make the six test sites in the same file compile: three `assert!(matches!(...))` sites use the pattern `UnknownReason::PreimagePlate { .. }`; three `assert_eq!` sites (the 33-byte 0x03 payload under `test`, twice, and the UPPERCASE plate) need the concrete value `UnknownReason::PreimagePlate { id_is_hash: false, x_len: None }` — `assert_eq!` compares VALUES, so `{ .. }` is `E0797` there. Run `cargo fmt --all` after; the formatter breaks those onto four lines.

- [ ] **Step 3: the I-3 site asserts the diagnosis.** The existing `a_preimage_plate_is_named_not_misdiagnosed` test's I-3 row (the 16-byte X under `hash`) becomes:

```rust file=me/crates/me-cli/src/sysw/mod.rs mode=fragment
// R0 r0 tests I-3: a kind-0x03 single whose X is not 32 bytes (id hash,
        // 16-byte X, 50 characters) is refused by the codec as
        // PreimageLengthMismatch and is named a preimage plate here too.
        assert_eq!(
            pack(
                vec!["ms10hashsqw46h2at4w46h2at4w46h2at4w4ssrnvvaudn2k4d".into()],
                None,
                ITER
            ),
            Err(SyswError::Unclassifiable(
                0,
                UnknownReason::PreimagePlate {
                    id_is_hash: true,
                    x_len: Some(16)
                }
            )),
        );
```

- [ ] **Step 4: the arm renders three truthful bodies.** In `crates/me-cli/src/main.rs`, replace the `U::PreimagePlate` arm (and the H6 §8.1.2 comment above it) with:

```rust file=me/crates/me-cli/src/main.rs mode=fragment
                // H6 §8.1.2, and F-503: this arm covers THREE shapes, not
                // one. A kind-0x03 single under the id `hash` with a
                // well-formed 33-byte payload is a CLASS and reaches
                // PreimageNotAdmitted instead, so what lands here is (a) an id
                // outside {entr, hash} with a well-formed payload -- the §4.3
                // collision case, and the ONLY one the 1-in-256 sentence is
                // true of; (b) the id `hash` with an X that is not 32 bytes --
                // a damaged or hand-built plate, which the text used to
                // misdiagnose as a wrong id (F-503); (c) a wrong id AND a wrong
                // length. The reason carries the diagnosis; the text says only
                // what is true of the record in hand.
                U::PreimagePlate { id_is_hash, x_len } => match (id_is_hash, x_len) {
                    (true, Some(n)) => format!(
                        "record {i} (records count from 0) is a kind-0x03 preimage payload \
                         under the id `hash` whose X is {n} bytes, not 32. A preimage plate \
                         is kind 0x03 followed by exactly 32 bytes (SPEC_ms_hashlock §1), \
                         and --pack-preimage admits only that. This string is a damaged or \
                         hand-built plate: re-encode it with `ms hashlock` from the phrase or \
                         the 32-byte preimage rather than editing the string."
                    ),
                    (false, Some(n)) => format!(
                        "record {i} (records count from 0) is a kind-0x03 preimage payload \
                         whose 4-character id is not `hash` and whose X is {n} bytes, not 32. \
                         A preimage plate is kind 0x03 under the id `hash` followed by exactly \
                         32 bytes (SPEC_ms_hashlock §1 rule 2), and --pack-preimage admits \
                         only that. Re-encode it with `ms hashlock` rather than editing the \
                         string."
                    ),
                    (_, None) => format!(
                        "record {i} (records count from 0) is a kind-0x03 preimage payload \
                         whose 4-character id is not `hash`. A preimage plate is kind 0x03 \
                         under the id `hash` (SPEC_ms_hashlock rule 2), and --pack-preimage \
                         admits only that. Re-encode it with `ms hashlock` rather than editing \
                         the string. If this string is a 33-byte seed backup that happens to \
                         begin 0x03, it is not a preimage: roughly 1 in 256 of them look like \
                         this."
                    ),
                },
```

Run: `cargo nextest run --locked -p mnemonic-engrave -E 'test(a_hash_id_plate_with_a_short_x) | test(a_preimage_plate_is_named)'`
Expected: PASS (2 tests run, 2 passed).

- [ ] **Step 5: MUTATION (run once, revert).** In the naming arm change `Err(ms_codec::Error::PreimageLengthMismatch { got }) => Some(got),` to `Err(ms_codec::Error::PreimageLengthMismatch { got: _ }) => None,` → both tests of Step 4 FAIL (the integration test on its first assertion, quoted above). Revert.

- [ ] **Step 6: the four seam rows.** Append to `crates/me-cli/testdata/codex32_seam_vectors.json` as the LAST four elements of `vectors` (the file is `json.dumps(indent=2)` style; keep it byte-exact):

```json file=me/crates/me-cli/testdata/codex32_seam_vectors.json mode=fragment
    {
      "name": "hash-kind03-16-byte-x",
      "string": "ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqmv3lqlgkn6s5c",
      "chars": 50,
      "host_admits": false,
      "device_admits": false,
      "source": "codex32.NewSeed(\"ms\", 0, \"hash\", 's', 17 bytes: 0x03 then 16 zero bytes): the preimage KIND under the preimage id `hash` with an X of 16 bytes, not 32 -- the string F-503 was filed on. The host names it (`PreimageLengthMismatch` -> a kind-0x03 payload under `hash` whose X is 16 bytes); before F-503 the device called it ClassCodex32Secret, a seed class, because H0's inertness tested only the 33-byte width. F-503 (operator ruling 2026-09-06): under the id `hash`, the kind 0x03 is inert at EVERY length. Every other id keeps H0's BIP-93-wide rule -- see bip93-plain-payload-0x03."
    },
    {
      "name": "hash-kind03-31-byte-x",
      "string": "ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq0n4lhy39fprwh",
      "chars": 74,
      "host_admits": false,
      "device_admits": false,
      "source": "codex32.NewSeed(\"ms\", 0, \"hash\", 's', 32 bytes: 0x03 then 31 zero bytes): one byte short of a plate, at the 74-character width of a plain BIP-93 256-bit secret. Refused on both sides (F-503)."
    },
    {
      "name": "hash-kind03-33-byte-x",
      "string": "ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsdgdmr756qg7w",
      "chars": 77,
      "host_admits": false,
      "device_admits": false,
      "source": "codex32.NewSeed(\"ms\", 0, \"hash\", 's', 34 bytes: 0x03 then 33 zero bytes): one byte past a plate. Refused on both sides (F-503)."
    },
    {
      "name": "hash-id-kind00-16-byte",
      "string": "ms10hashsqqqqqqqqqqqqqqqqqqqqqqqqqqqq0mck2082pkd87",
      "chars": 50,
      "host_admits": false,
      "device_admits": true,
      "source": "codex32.NewSeed(\"ms\", 0, \"hash\", 's', 17 bytes: 0x00 then 16 zero bytes): the entr KIND under the preimage id `hash`. The host refuses it as an id/kind mismatch (TagKindMismatch, ruling L24); the device still admits it as a BIP-93 seed. THE CONTROL ROW for F-503: the operator chose to make only the 0x03 kind inert under `hash`, not to reserve the whole id, and this row is what keeps that choice from silently widening."
    }
```

Re-pin the Rust literal:

```rust file=me/crates/me-cli/tests/codex32_seam.rs mode=fragment
const SEAM_VECTORS_SHA256: &str =
    "f53a17dc9d1ea5a4c0ff913786e82a5f19de004f4719536bb6d957bccdef1ee8";
```

Run: `cargo nextest run --locked -p mnemonic-engrave -E 'test(the_host_never_admits)'` → PASS (host column: all four `Unknown`, `host_admits` false). MUTATION: put the OLD sha (`2c2fbb3f…`) back → FAIL `testdata/codex32_seam_vectors.json is not the file the fork's copy is pinned to; re-pin BOTH literals`. Revert.

- [ ] **Step 7: the capture.** The record-corpus capture enumerates the seam file position by position, so it now fails (`the_capture_is_the_whole_corpus`: left 38, right 42). Insert these four entries in `crates/me-cli/testdata/record_corpus_pre_s2.json` immediately after the `codex32_seam/bip93-plain-33-byte-payload-0x03` entry:

```json file=me/crates/me-cli/testdata/record_corpus_pre_s2.json mode=fragment
    {
      "origin": "codex32_seam/hash-kind03-16-byte-x",
      "record": "ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqmv3lqlgkn6s5c",
      "class": "Unknown",
      "consult": "record-refusal"
    },
    {
      "origin": "codex32_seam/hash-kind03-31-byte-x",
      "record": "ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq0n4lhy39fprwh",
      "class": "Unknown",
      "consult": "record-refusal"
    },
    {
      "origin": "codex32_seam/hash-kind03-33-byte-x",
      "record": "ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsdgdmr756qg7w",
      "class": "Unknown",
      "consult": "record-refusal"
    },
    {
      "origin": "codex32_seam/hash-id-kind00-16-byte",
      "record": "ms10hashsqqqqqqqqqqqqqqqqqqqqqqqqqqqq0mck2082pkd87",
      "class": "Unknown",
      "consult": "record-refusal"
    },
```

- [ ] **Step 8: CHANGELOG.** Under `## [Unreleased]` / `### Changed`, first item:

```md file=me/crates/me-cli/CHANGELOG.md mode=fragment
- `me sysw pack --pack-preimage` names WHICH conjunct of the plate shape a
  kind-0x03 record failed: a record under the id `hash` whose X is not 32 bytes
  is now refused as "under the id `hash` whose X is N bytes, not 32", where it
  used to be told its id was not `hash` and handed the 1-in-256 collision
  sentence that is true only of a 33-byte payload (F-503, the host half). The
  same records are refused; only the text changed. Four seam corpus rows pin
  the F-503 shapes on both sides (`codex32_seam_vectors.json`, re-pinned).
```

- [ ] **Step 9: boundary gate.** `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` → **634 run: 631 passed, 3 failed (the zsh trio), 2 skipped**; `cargo fmt --all -- --check` clean; `cargo clippy --locked --all-targets -p mnemonic-engrave` → only the pre-existing `manual_is_multiple_of` warning. Commit: `F-503 (me): the preimage-plate refusal names the conjunct that failed; four seam rows pin the hash-id kind at every length`.

---

### Task 2: the fork — `IsPreimageKind`, the `isStrictMs1` conjunct, the vendored corpus (CONVERGENCE with Task 1)

**Files:** Modify `codex32/mspayload.go`, `codex32/mspayload_test.go`, `sysw/classify.go`, `sysw/codex32_seam_test.go`; replace `sysw/testdata/codex32_seam_vectors.json` with a byte-identical copy of Task 1's file.

**Interfaces — Produces:** `codex32.IsPreimageKind(s String) bool`. **Consumes:** `codex32.IsPreimage`, `String.Split`, `ParsePrefix` (unchanged).

- [ ] **Step 1: vendor the corpus and re-pin (RED).** `cp <me worktree>/crates/me-cli/testdata/codex32_seam_vectors.json sysw/testdata/codex32_seam_vectors.json`; `sha256sum` it → `f53a17dc9d1ea5a4c0ff913786e82a5f19de004f4719536bb6d957bccdef1ee8`; set

```go file=fork/sysw/codex32_seam_test.go mode=fragment
const seamVectorsSHA256 = "f53a17dc9d1ea5a4c0ff913786e82a5f19de004f4719536bb6d957bccdef1ee8"
```

Run: `go test ./sysw/ -run TestCodex32Seam -count=1`
Expected: FAIL, exactly three rows —
```
codex32_seam_test.go:66: hash-kind03-16-byte-x: device admits = true, want false (Classify = 2)
codex32_seam_test.go:66: hash-kind03-31-byte-x: device admits = true, want false (Classify = 2)
codex32_seam_test.go:66: hash-kind03-33-byte-x: device admits = true, want false (Classify = 2)
```
(the control row `hash-id-kind00-16-byte` passes: device admits = true, want true).

- [ ] **Step 2: the predicate and its test.** In `codex32/mspayload.go`, insert before `// DecodeMS1Preimage decodes`:

```go file=fork/codex32/mspayload.go mode=fragment
// IsPreimageKind is the KIND alone: an unshared string whose first payload
// byte is the preimage prefix 0x03, at ANY payload length. It is wider than
// IsPreimage (which also requires the 33-byte plate width) and narrower than
// nothing else: a share is never of any kind, because a share's bytes are SSS
// points and its first byte says nothing about the secret's kind.
//
// F-503 (operator ruling 2026-09-06). ms-codec's dispatch_payload reads the
// first payload byte as the kind at every width, so a 0x03 payload of the
// wrong length is `PreimageLengthMismatch` -- refused, never a seed. This
// device deliberately stays BIP-93-wide for every id but one: under the
// preimage id `hash`, sysw.isStrictMs1 treats this kind as inert at every
// length, so a damaged or hand-built plate (a 17-byte payload under `hash`,
// 50 characters) is ClassUnknown instead of a seed. Under any OTHER id the
// 16-, 20-, 24-, 28- and 32-byte seeds beginning 0x03 stay seeds, as the seam
// corpus row bip93-plain-payload-0x03 pins; the id is what keeps H0's
// argument above intact.
//
// Reads the prefix fields and one payload byte; nothing new is retained.
func IsPreimageKind(s String) bool {
	f, err := ParsePrefix(s.String())
	if err != nil || !f.Unshared {
		return false
	}
	d := s.Seed()
	return len(d) > 0 && d[0] == msPrefixPreimage
}
```

and change the H0 comment sentence above `IsPreimage` to read:

```go file=fork/codex32/mspayload.go mode=fragment
// IS REFUSED. Roughly 1 in 256 of 33-byte seeds. The 16-, 20-, 24-, 28- and
// 32-byte seeds are untouched under every id but `hash` (F-503: see
// IsPreimageKind), and so is every share.
```

Append to `codex32/mspayload_test.go`:

```go file=fork/codex32/mspayload_test.go mode=fragment
// TestIsPreimageKindIsTheKindAtEveryLength is F-503: the KIND predicate
// answers for the first payload byte alone, at 17, 32 and 34 bytes as at the
// plate's 33, under any id -- and never for a share, whose first byte is an
// SSS point and not a kind. The strings are the seam corpus rows (F-503's
// four) and H0's, generated by NewSeed and pinned there.
//
// MUTATION: `len(d) == 33 &&` back into IsPreimageKind -> the 17-, 32- and
// 34-byte rows answer false, and this fails.
func TestIsPreimageKindIsTheKindAtEveryLength(t *testing.T) {
	for _, tc := range []struct {
		name string
		s    string
		want bool
	}{
		{"plate, 33 bytes under hash", "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c", true},
		{"17 bytes under hash (F-503)", "ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqmv3lqlgkn6s5c", true},
		{"32 bytes under hash", "ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq0n4lhy39fprwh", true},
		{"34 bytes under hash", "ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsdgdmr756qg7w", true},
		{"17 bytes under test (H0 keeps it a seed)", "ms10testsqv0qqqqqqqqqqqqqqqqqqqqqqq8mzk8tjfdnjn5", true},
		{"33 bytes under entr", "ms10entrsqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5gz69g08wwtz9", true},
		{"kind 0x00 under hash (the control)", "ms10hashsqqqqqqqqqqqqqqqqqqqqqqqqqqqq0mck2082pkd87", false},
		{"a 2-of-N share beginning 0x03", "ms12testaqv0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdq7pl8qdc5tsp", false},
	} {
		c, err := New(tc.s)
		if err != nil {
			t.Fatalf("%s: %v", tc.name, err)
		}
		if got := IsPreimageKind(c); got != tc.want {
			t.Errorf("%s: IsPreimageKind = %v, want %v", tc.name, got, tc.want)
		}
	}
}
```

Run: `go test ./codex32/ -run TestIsPreimageKind -count=1` → PASS. MUTATION: `len(d) == 33 &&` in `IsPreimageKind` (NOT in `IsPreimage` — the two lines are identical text; edit by function, and diff before restoring) → FAIL on the 17-, 32- and 34-byte rows and the 17-byte `test` row. Revert.

- [ ] **Step 3: the conjunct (GREEN).** In `sysw/classify.go`, replace `isStrictMs1`'s tail and add the helper:

```go file=fork/sysw/classify.go mode=fragment
	c, err := codex32.New(record)
	// H0 (SPEC_ms_hashlock §9): a hashlock preimage plate is BCH-valid and
	// inside the cap, and it is not a seed. Inert here — no class of its own.
	// F-503: and under the preimage id `hash`, the 0x03 KIND is inert at
	// EVERY length, not only the plate's 33 bytes -- a damaged or hand-built
	// plate is ClassUnknown, as it is on the host (PreimageLengthMismatch),
	// instead of a seed the host would never have packed. Every other id keeps
	// the BIP-93-wide rule (seam row bip93-plain-payload-0x03).
	return err == nil && !codex32.IsPreimage(c) && !isHashIdPreimageKind(c)
}

// isHashIdPreimageKind is F-503's conjunct: the preimage KIND (first payload
// byte 0x03, any length, unshared) under the preimage id `hash`. The id is
// read the way codex32.IsPreimagePlate reads it -- case-sensitively, from
// String.Split -- so the UPPERCASE spelling is not this shape either; it is
// refused elsewhere (H6 §4.3) and never reaches a seed class.
func isHashIdPreimageKind(c codex32.String) bool {
	if !codex32.IsPreimageKind(c) {
		return false
	}
	id, _, _ := c.Split()
	return id == "hash"
}
```

Run: `go test ./codex32/ ./sysw/ -count=1` → both `ok`. MUTATION: drop `&& !isHashIdPreimageKind(c)` → Step 1's three-row FAIL returns. Revert.

- [ ] **Step 4: boundary gate.** `gofmt -l codex32/ sysw/` empty; `go test $(go list ./... | grep -v /gui$) -count=1` all ok; `scripts/gui-shard-test.sh ./gui/ 24` → **RESULT: ok -- all 1289 tests ran across 24 shards** (no gui test changes; `TestScanDoesNotHandAPreimagePlateToEngrave` in `gui/codex32_polish_test.go` is the one that catches a widened `IsPreimage` — it stays green only because `IsPreimage` is untouched). Commit (`-s`): `sysw: the preimage KIND under the id hash is inert at every length; codex32.IsPreimageKind (F-503)`.

---

### Task 3: records

- [ ] **Step 1: ms spec.** In `mnemonic-secret/design/SPEC_ms_hashlock.md` (branch `f503-records`): where §9 says the fork's `isStrictMs1` "makes a `0x03` string INERT (never `ClassCodex32Secret`, no new class…)", append one sentence: *"F-503 (2026-09-06): the device tests the 33-byte plate SHAPE under any id, and — under the preimage id `hash` only — the 0x03 KIND at every length; a 0x03 payload of another length under another id stays a BIP-93 seed on the device (seam row `bip93-plain-payload-0x03`), which the host never packs."* In §12 item 7, after "`sysw.Classify` is not `ClassCodex32Secret`", add *"(at every length under the id `hash`, F-503)"*. Commit with the trailers.
- [ ] **Step 2: F-503 closure** in engrave `design/FOLLOWUPS.md`: append to the entry *"CLOSED 2026-09-06: me <sha>, fork <sha>, ms <sha>; rule = kind 0x03 under `hash` at every length (operator ruling), control row `hash-id-kind00-16-byte`."* (on the me branch, last commit).
- [ ] **Step 3: report** to `design/agent-reports/f503-implementation-report.md` — per-task SHAs, the RED quotes, the boundary numbers, every mutation with its failing line, every deviation.

## 4. Ship (controller)
Post-impl check (sonnet): each new test reds under its named mutation on the real branches; the seam file is byte-identical in both repos and both pins equal its sha256; the capture matches the enumerated corpus. Then: engrave master ← `f503` (merge), `scripts/push-via-staging.sh master`; fork main ← `f503` (merge, signoff), plain push, `~/bin/sh/sh2-flash -b`; ms master ← `f503-records`, ci/staging ritual. Then the me 0.9.0 release (own brief).

## 5. Gate evidence (controller, 2026-09-06, gate worktrees)
- me: `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` → 634 run / 631 passed / 3 failed (zsh trio) / 2 skipped; fmt clean; clippy only `manual_is_multiple_of` (pre-existing). Mutation "x_len always None" → 2 FAIL (the unit test and the integration test), restored → PASS. Old seam sha → `re-pin BOTH literals` FAIL, restored → PASS.
- fork: `go test ./codex32/ ./sysw/` ok; gofmt clean; gui 1289/24 shards ok. Conjunct dropped → the three F-503 rows FAIL (quoted in Task 2 Step 1); `IsPreimageKind` at 33 bytes → 4 rows FAIL; restored → ok. One controller slip recorded so nobody repeats it: a `sed` restore matched `IsPreimage`'s identical line and widened it; the gui set caught it at `TestScanDoesNotHandAPreimagePlateToEngrave`.
- checker: `scripts/h6-plan-blocks-vs-tree.sh design/IMPLEMENTATION_PLAN_hashlock_F503_hash_kind_inert.md /scratch/code/shibboleth/.tmp/seedhammer-f503-gate /scratch/code/shibboleth/mnemonic-secret /scratch/code/shibboleth/me-worktrees/f503-gate` → see the plan commit message.
