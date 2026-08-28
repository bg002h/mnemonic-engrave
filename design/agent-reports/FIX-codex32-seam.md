# FIX — the codex32 seam: a refusal that named the wrong cause, and an invariant that was only prose

**Date:** 2026-08-28
**Branches:** `fix/codex32-seam` in `mnemonic-engrave` (off `master` `fdc67c8`) and in
`seedhammer` (off `main` `57315eb`). **Neither pushed.** `upstream/` untouched.
**Scope:** the two items owed by the architect's ruling on the `ms1` profile. Nothing
else. No validation set widened; `G-P3.10` and `G-P3.14` untouched.

The ruling is taken as settled and was not re-litigated: constellation `ms1` is a
deliberate two-gate profile over BIP-93 (length ∈ `VALID_STR_LENGTHS` ∪
`VALID_MNEM_STR_LENGTHS`, then the id must be `entr`); the Rust codec and the Go fork
are both correct about different things; the fork's `ms10faux…`/`ms10leets…` fixtures
are BIP-93 conformance vectors. Nothing was owed to either codec or to any fixture.

---

## Item 1 — the refusal now names the profile, not the classifier

### The two messages, run, verbatim

Input (BIP-93 test vector 1, a valid 128-bit codex32 secret, fed via `--in` so nothing
reaches argv):

```
me sysw pack --in bip93.txt --no-passphrase --out p.bin
```

**Before** — exit 4:

```
me: record 0 (records count from 0) is not a form this container can place: not a
BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`
record. Descriptors and addresses are not yet classifiable here — see sysw::classify
```

**After** — exit 4:

```
me: record 0 (records count from 0) is a VALID BIP-93 codex32 string — the checksum is
good — but not a constellation `ms1` record, so this container cannot place it.
      `ms1` is a two-gate PROFILE over BIP-93: the whole string must be [50, 56, 62,
69, 75] characters (entropy) or [51, 58, 64, 70, 77] (mnemonic), and the 4-character id
must be `entr`. This one is 48 characters.
      Plain BIP-93 secrets are 48 or 74 characters and BIP-93 SHARES carry their own
id, so neither is a constellation record — re-encode the entropy as `ms1` rather than
editing the string.
```

The old message was measured by `git stash`-ing the change, rebuilding, and running the
same command against the same file — not quoted from source.

The two length sets are read from `ms_codec::consts::VALID_STR_LENGTHS` and
`VALID_MNEM_STR_LENGTHS` at format time, not typed as literals, so they cannot go stale
against the codec.

### How it is decided

A new `pub fn bip93_outside_the_profile` in `crates/me-cli/src/seal/record.rs`, beside
the `MsTooLong`/`Invalid` split it reuses:

- HRP is `ms` (`classify(s) == Ok(Format::Ms)`), **and**
- `ms_codec::codex32::Codex32String::from_string(s)` succeeds, **and**
- `ms_codec::decode(s)` fails.

`sysw::unknown_reason` consults it last and returns a new
`UnknownReason::Bip93OutsideTheProfile(usize)`; `main.rs::sysw_error` renders it.

### No body is echoed

The record is a seed. The message names only the character count and the two length
sets — shape, the same class of fact `RecordError::MsTooLong` already names.

Asserted, not assumed:
`sysw_cli.rs::a_valid_bip93_string_is_told_it_is_bip93_and_not_a_constellation_ms1`
checks that stderr contains neither the whole record **nor any 12-character window of
it** (37 windows for the 48-character vector). It passes.

### The admission set is unchanged

Three independent measurements:

1. **The diff deletes nothing.** `git diff --numstat` over the whole change:
   `19/0`, `89/0`, `95/0`, `75/0`, `35/0` — 313 insertions, **0 deletions**. No line of
   `classify`, `classify_with` or `validate_record` was touched. Only a new enum
   variant, a new predicate, a new render arm and tests were added.
2. **Every previously-passing test still passes, and the count moved by exactly the
   number of new tests.** 433 → 440 (7 added: 2 in `seal/record.rs`, 2 in
   `sysw/mod.rs`, 1 in `tests/codex32_seam.rs`, 2 in `tests/sysw_cli.rs`).
3. **The verdicts are pinned per string.** The item-2 vector file pins
   `sysw::classify(s) == Class::Codex32Secret` for all 8 rows, and
   `sysw::tests::the_profile_arm_is_gated_on_a_real_bip93_parse` asserts that both
   constellation `ms1` records still classify as `Codex32Secret` **and still pack**.
   The exit code for the BIP-93 string is 4 before and after.

---

## Item 2 — the seam is a gate

### The shared file

**Primary:** `crates/me-cli/testdata/codex32_seam_vectors.json` (mnemonic-engrave)
**Vendored, byte-identical:** `sysw/testdata/codex32_seam_vectors.json` (seedhammer)
**sha256:** `3d53ef88a474f02c15aa60a839f4a31071598a26c853463122a847515926eb6a`
**Rows:** 8

| row | chars | host | device | provenance |
| --- | --- | --- | --- | --- |
| `bip93-secret-128` | 48 | no | **yes** | BIP-93 test vector 1 (`TestBIPVector1`) |
| `bip93-secret-256` | 74 | no | **yes** | BIP-93 test vector 4 (`TestBIPVector4`) |
| `bip93-share` | 48 | no | **yes** | BIP-93 test vector 3, share `s` (`TestBIPVector3`) |
| `constellation-entr-128` | 50 | yes | yes | the repo's canonical `ms1` fixture; `inspect` = tag `entr`, 16 payload bytes |
| `constellation-entr-256` | 75 | yes | yes | the bip84 `bacon`×24 bundle's `ms1`; `inspect` = tag `entr`, 32 payload bytes |
| `entr-id-but-off-profile-length-90` | 90 | no | **yes** | `MS1_90`, generated in the fork by `biptool seed -seedlen 42 -id entr` |
| `past-the-engraveable-cap-91` | 91 | no | no | `MS1_91`, same generator at `-seedlen 43` |
| `bip93-bad-checksum` | 48 | no | no | BIP-93's own bad-checksum vector (`TestBIPBadChecksums`) |

`host` = Rust `mnemonic_engrave::sysw::classify(s) == Class::Codex32Secret`.
`device` = Go `sysw.Classify(s) == ClassCodex32Secret`. Both are the *same* layer in the
two languages, so the columns answer one question.

Four divergence rows, not two. The `entr`-id 90-character row is the one that matters
most: **its id IS `entr` and it still fails**, so a diagnosis blaming only the tag would
be wrong about it. It is also why the item-1 message names the length sets rather than
the id alone.

The row set deliberately keeps all three shapes — yes/yes, no/yes, no/no — and both
tests assert that it still does, because a set with no yes/yes row is passed by a mutant
that refuses everything and a set with no no/no row by one that admits everything.

Every `chars` field was machine-checked against `len(string)` (one was wrong when
written by hand — 47 for a 48-character string — and the check caught it).

### The two tests

- `mnemonic-engrave/crates/me-cli/tests/codex32_seam.rs` (79 lines: a 49-line `fn`, the rest doc comment) — asserts
  the sha256 pin, the `chars` self-check, **the invariant `host <= device`**, the host
  column against `sysw::classify`, and the shape coverage.
- `seedhammer/sysw/codex32_seam_test.go` (85 lines: a 54-line `func`, the rest comment) — the same, with the
  **device** column against `sysw.Classify`.

Neither implementation is compared to the other; both are compared to the file. That
only means something if it is the same file, so **both tests pin the identical sha256
literal**. Editing one copy without the other reds that copy's suite (demonstrated
below).

### Results

```
$ cargo nextest run --locked            # mnemonic-engrave, whole suite
     Summary [  33.651s] 440 tests run: 440 passed, 1 skipped

$ go test ./sysw/ -run TestCodex32Seam -v   (seedhammer)
=== RUN   TestCodex32SeamDeviceAdmitsEverythingTheHostDoes
--- PASS: TestCodex32SeamDeviceAdmitsEverythingTheHostDoes (0.00s)
ok  	seedhammer.com/sysw	0.005s
```

---

## Mutation results — every check was tried against a break

### Item 1

| mutant | effect | outcome |
| --- | --- | --- |
| **M1** — delete the `Bip93OutsideTheProfile` branch from `unknown_reason` | the arm can never be reached | **RED.** `sysw::tests::a_valid_bip93_codex32_names_the_profile_not_the_classifier` FAILED, and `a_valid_bip93_string_is_told_it_is_bip93_and_not_a_constellation_ms1` FAILED with `"BIP-93" missing from: me: record 0 … not an md1/mk1/ms1/mt1 string …` — i.e. the old text came back and the test saw it. The control `a_record_of_no_class_at_all_still_names_the_classifier` stayed **ok**, so the two cases really are distinguished and not merely both matched. |
| **M2** — delete the `Codex32String::from_string(...).is_ok()` clause from `bip93_outside_the_profile` | a bad-checksum `ms1` becomes "outside the profile" | **RED**, two tests: `bip93_outside_the_profile_separates_bip93_from_the_constellation` (`not codex32 at all: ms10fauxs…`) and `the_profile_arm_is_gated_on_a_real_bip93_parse`. |

Both mutants were reverted and the four tests re-run green.

### Item 2 — flips in both directions, and back

Each flip edits the row, recomputes the sha256 and **re-pins both literals**, so the
failure is the verdict assertion and not the hash pin.

| flip | Rust | Go |
| --- | --- | --- |
| `bip93-secret-128` → `host=true, device=false` (the forbidden shape) | **RED** — `bip93-secret-128: the HOST admits what the DEVICE refuses` | **RED** — same line, plus `device admits = true, want false (Classify = 2)` |
| `constellation-entr-128` → `host=false` | **RED** — `constellation-entr-128: host verdict` | ok (device column unchanged) |
| `past-the-engraveable-cap-91` → `device=true` | ok (host column unchanged) | **RED** — `past-the-engraveable-cap-91: device admits = false, want true (Classify = 0)` |
| edit the fork's copy only, **no** re-pin | n/a | **RED** on the pin — `hashes to 8f326532…, not the pinned 3d53ef88… — the vendored copy and the primary have drifted` |

The last three exist because the first flip alone does not prove much: the invariant
assertion fires before the column assertions, so it could mask a vacuous column check.
Flips 2 and 3 fire the *column* assertions on their own, one in each language.

**Restored:** the file hashes back to `3d53ef88a474f02c15aa60a839f4a31071598a26c853463122a847515926eb6a`
in both repos, both literals carry that value, and both tests pass.

---

## Verification, verbatim

```
mnemonic-engrave $ cargo nextest run --locked
     Summary [  33.651s] 440 tests run: 440 passed, 1 skipped
   nextest_rc=0
mnemonic-engrave $ cargo clippy --locked --all-targets -- -D warnings   -> rc 0
mnemonic-engrave $ cargo fmt --check                                    -> rc 0

seedhammer $ /nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go test ./sysw/ ./codex32/
ok  	seedhammer.com/sysw	0.036s
ok  	seedhammer.com/codex32	0.002s
seedhammer $ …/go vet ./sysw/    -> rc 0
seedhammer $ …/go build ./...    -> rc 0
seedhammer $ …/gofmt -l sysw/    -> no output
```

Baseline before the change was **433 passed, 1 skipped** on `master` `fdc67c8`. (The
brief cited 430 against `59dd1e4`; `master` had moved two commits ahead — `ba8915f`
and `fdc67c8` — before this work started. Measured, not assumed.) The 1 skipped test is
the pre-existing `ms_remedy_runs::the_advised_pipeline_runs_and_writes_a_payload`, which
skips unless `MS_P2_BIN` is set.

No exit code was read through a pipe: every gate above was captured to a file with its
`$status` appended, then grepped.

### Pre-existing, reported and NOT fixed here

`go vet ./...` on the fork exits **1**, in four files this change does not touch:

```
backup/backup_test.go:393:48: testing.ArtifactDir requires go1.26 or later (file is go1.25)
backup/freetext_test.go:240:48: …
gui/freetext_sizeproof_golden_test.go:111:13: …
gui/transaction_golden_test.go:104:13: …
vet_all_rc=1
```

`go.mod` pins `go 1.25.10` while the tree uses a 1.26 API, so 1.25 cannot build `./gui/`
at all and the toolchain used throughout was
`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`. The working tree is
otherwise clean (only the two new files are untracked), so this is entirely pre-existing.
`go vet ./sysw/` — the package this change touches — is rc 0.

---

## Scope of the negative claims

- *"The diff deletes nothing"* — `git diff --numstat` over all five modified files in
  `mnemonic-engrave`, plus `git status --short` in `seedhammer` showing only two
  untracked additions. Whole change, not a sample.
- *"No body is echoed"* — the whole record and all 37 twelve-character windows of the
  48-character vector, against the full stderr of the real binary. It does **not** claim
  anything about other records or other commands.
- *"`md1` never reaches the new arm"* — measured over the three `MD1` fixtures in
  `seal/record.rs` only. The mechanism found is that `md`/`mk` checksum under different
  BCH constants, so `Codex32String::from_string` rejects a real `md1` outright; the test
  asserts that rejection first, so the day it stops being true the test names the HRP
  gate as the only thing left. It is **not** a claim that no non-`ms` string can ever
  satisfy the other two clauses — I could not construct one with the tools in this
  crate (`from_unchecksummed_string` does not round-trip through `from_string` for any
  HRP, `ms` included), and the HRP gate is kept as the cheap defence rather than removed
  on the strength of a failure to construct.

---

## Follow-up filed

**F-401** (`design/FOLLOWUPS.md`, owning phase: ownerless residue) — `me sysw pack`
cannot tell an over-cap `ms1` from an off-profile one. `me seal` distinguishes
`MsTooLong` from `Invalid`, but `sysw::classify` sees only `Err(_)`, so the new arm
answers `Bip93OutsideTheProfile` at 91 characters exactly as it does at 48. Both
messages are true — 91 is in neither profile length set — so it is a lost distinction,
not a false statement. The two `me seal` messages in F-401's table were run, not quoted
from source. The vector already exists as the `past-the-engraveable-cap-91` row, so the
fix is a row, not a new fixture.

---

## Files

**mnemonic-engrave** (branch `fix/codex32-seam`)
- `crates/me-cli/src/seal/record.rs` — `bip93_outside_the_profile` + 2 tests
- `crates/me-cli/src/sysw/mod.rs` — `UnknownReason::Bip93OutsideTheProfile`, the
  `unknown_reason` arm, + 2 tests
- `crates/me-cli/src/main.rs` — the rendered message
- `crates/me-cli/tests/sysw_cli.rs` — the message test + its control
- `crates/me-cli/testdata/codex32_seam_vectors.json` — **new**, the shared vectors
- `crates/me-cli/tests/codex32_seam.rs` — **new**, the host half of the gate
- `design/FOLLOWUPS.md` — F-401

**seedhammer** (branch `fix/codex32-seam`)
- `sysw/testdata/codex32_seam_vectors.json` — **new**, byte-identical vendored copy
- `sysw/codex32_seam_test.go` — **new**, the device half of the gate
