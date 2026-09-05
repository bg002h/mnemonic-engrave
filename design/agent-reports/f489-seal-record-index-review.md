# F-489 review — `me seal` names the refused record's index

Reviewer: sonnet (independent, mechanical/verification tier). Branch
`f489-seal-record-index`. **Reviewed tip: `768fb12f`** (moved during review from
the originally-assigned `379fd528`, code commit `2abc4a1a`, base master
`7777a236`). Worktree: `/scratch/code/shibboleth/me-worktrees/f489-review`
(detached, mutated and reverted in place, never committed).

Question: does the change name the right record in every refusal path `me seal`
has, without changing WHAT is refused, and does every test the diff adds or
edits fail on the defect it names?

## Commits in scope

```
768fb12f seal: test the PUBLIC section's record locator too (F-489)
379fd528 records: CHANGELOG [Unreleased] -- ...; F-489 CLOSED by 2abc4a1a
2abc4a1a seal: name the refused record's index and section ... SealError::RecordAt
```

## Checks executed

**(1) Full diff read.** `git diff 7777a236..768fb12f` read in full (three
commits: code + secret-section test in `2abc4a1a`; CHANGELOG/FOLLOWUPS in
`379fd528`; public-section test added in `768fb12f`).

**(2) Secret-loop CLI test RED under mutation.** Reverted
`seal_deterministic`'s secret loop from the `record_or_mnemonic(r).map_err(...)`
wrapping back to plain `record_or_mnemonic(r)?`, ran
`seal_names_the_refused_record_index_like_sysw_pack`:

```
FAIL preimage_plate_is_not_a_seed seal_names_the_refused_record_index_like_sysw_pack
panicked at .../preimage_plate_is_not_a_seed.rs:174:5:
stderr does not name the refused record's index the way sysw pack does:
me: this record is a hashlock PREIMAGE plate (kind 0x03), not a seed record; ...
```
Reverted; confirmed clean (`git diff --stat` empty) before rebuilding.

**(3) Public section path.** Built `me` (`cargo build -p mnemonic-engrave`),
read `me seal --help` for flag names, then ran a real invocation with a valid
`md1` at index 0 and an invalid one at index 1:

```
$ me seal --plaintext "md1yqpqqxqq8xtwhw4xwn4qh" --plaintext "md1NOTVALID" --out /tmp/f489_pub_test.uf2
me: record 1 (records count from 0) in the public section: record has an uppercase character at byte 3 — records must be lowercase, or the same wallet has two different public-data hashes (§6.4)
EXIT=4
```
Confirms the public per-record loop (`check_public`) correctly names index 1,
section "public", exit `EXIT_INVALID` (4).

At the originally-assigned tip `379fd528` **no test covered this path** — the
only `RecordAt` tests (unit `refuses_an_uppercase_bip39_mnemonic` and the CLI
test) both exercised the **secret** section only. That was a genuine Important
finding (new code path, no test) at that tip. **Commit `768fb12f`, added after
this review began, closes it**: `seal_names_the_refused_public_record_index_too`
(a valid `md1` at public index 0, a checksum-corrupted copy at index 1 via
`--plaintext`) — verified:
- Unmutated: `PASS` (0.004s).
- Mutated (public arm reverted to `Err(e) => return Err(SealError::Record(e))`):
  ```
  FAIL preimage_plate_is_not_a_seed seal_names_the_refused_public_record_index_too
  panicked at .../preimage_plate_is_not_a_seed.rs:224:5:
  stderr does not name the refused PUBLIC record's index:
  me: invalid record: codex32 decode error: BCH checksum verification failed
  ```
Reverted; `git diff --stat` empty afterward. **The gap is closed as of
`768fb12f`; not counted as an open finding below.**

**(4) All `SealError::Record(` producers/consumers.**
```
seal/mod.rs:91   Display arm for Record (unchanged)
seal/mod.rs:196  decode_public_set(&refs).map_err(SealError::Record)   -- whole-SET decode, kept unindexed by design (no single record to blame)
seal/mod.rs:248  match arm inside seal_deterministic's secret loop (the new wrapping)
seal/mod.rs:319,333,340  record_or_mnemonic's three error returns -- ALL are SealError::Record; every one is now caught and re-tagged RecordAt by the caller's map_err (no case falls through as bare Record from the secret path)
```
Only other-crate/consumer hit: `main.rs:952-953`:
```rust
seal::SealError::Iterations(_) => EXIT_USAGE,
_ => EXIT_INVALID,
```
a catch-all — `RecordAt` and the old `Record` both fall into `_`. Verified live
in check (3): the public-section refusal exited 4 (`EXIT_INVALID`). No other
`crates/` file pattern-matches on `SealError::Record` specifically, so no
consumer silently stopped matching. **No behaviour change found: same records
refused, for the same reasons, same exit codes — only the message gained a
locator.**

**(5) Existing seal tests, whole crate, fmt, clippy.**
- `--test preimage_plate_is_not_a_seed` (all 6): PASS.
- `--lib -E 'test(/seal/)'` (80 tests): PASS.
- Whole crate `--no-fail-fast`, captured once:
  ```
  621 tests run: 618 passed, 3 failed, 2 skipped
  FAIL history_purge::editing_the_file_alone_is_the_trap_the_message_warns_about
  FAIL history_purge::the_harness_records_history_at_all
  FAIL history_purge::the_emitted_zsh_recipe_actually_purges_the_entry
  ```
  All three fail with `/usr/bin/zsh is required` — the known box-local trio
  (no zsh on this machine), not related to this diff.
- `cargo fmt -p mnemonic-engrave --check`: exit 0, clean.
- `cargo clippy -p mnemonic-engrave --all-targets --locked`: one warning,
  `manual_is_multiple_of` in `sysw/composer_records.rs:114` — the known
  pre-existing nightly lint, not touched by this diff.

**(6) CHANGELOG and FOLLOWUPS closure header.** Both read at tip `768fb12f`.
CHANGELOG `[Unreleased]` entry: states `me seal` now names the refused record
by position and section, quoting the secret-section form as its example,
"Behaviour unchanged: the same records are refused for the same reasons; only
the sentence gained a locator" — true per (4)/(5) above; it illustrates with
one section but does not claim the fix is secret-only, so not misleading.
FOLLOWUPS.md F-489 header: `CLOSED 2026-09-05 by 2abc4a1a (SealError::RecordAt;
seal_names_the_refused_record_index_like_sysw_pack)` — the cited commit and
test both exist and do what's claimed; it does not name `768fb12f`'s
public-section test, which landed after the header was written. Not false, but
worth a note: the header was written naming only the secret-side proof at a
moment when the public per-record path (touched by the same code commit) had
no test at all — see finding I-1 below, now resolved.

## Findings

### I-1 — public-section `RecordAt` path shipped untested at `379fd528`; closed by `768fb12f` (no longer open)
At the tip this review was assigned (`379fd528`), `check_public`'s per-record
match arm (`crates/me-cli/src/seal/mod.rs`, the arm now at line ~185) had been
changed to emit `SealError::RecordAt { section: "public", .. }`, but neither the
unit tests nor the CLI test suite exercised it — both existing `RecordAt`
assertions were secret-section only. Per this review's severity rubric ("a new
path with no test = Important"), this was Important. **Commit `768fb12f`
(landed mid-review) adds `seal_names_the_refused_public_record_index_too`,
verified above: PASS unmutated, RED under the matching mutation
(`Err(e) => return Err(SealError::Record(e))` restored) with the exact stderr
the mutation comment predicts.** Treated as resolved; not counted in the
closing tally.

### M-1 — stale comment now describes the wrong error variant
`crates/me-cli/src/seal/mod.rs:1202-1203`, on
`an_overlong_ms1_in_public_is_reported_as_a_secret_not_as_too_long`:
> "Mutation this pins: delete the `classify(r) == Format::Ms` guard from
> `check_public` — the assertion then sees `SealError::Record` instead."

This comment predates the diff and was not touched by it, but the diff changed
the code it describes. Verified by performing exactly the mutation it names
(deleting the `Err(record::RecordError::MsTooLong(_)) => ...` arm): the test
still fails correctly, but the panic message is
```
over-length ms1 in --plaintext reported as RecordAt { section: "public", index: 0, source: MsTooLong(91) }; ...
```
— `RecordAt`, not the bare `SealError::Record` the comment claims. The test's
own catch-all (`Err(other) => panic!(...)`) still catches the mutation, so
there is no test-efficacy defect — this is wording only. Reverted after
verification (`git diff --stat` empty).

## Closing counts

- Important: 0 open (1 found during review at `379fd528`, resolved by `768fb12f`)
- Minor: 1 (M-1, stale comment)
- Nit: 0

**GREEN** at tip `768fb12f`. No behaviour change beyond the added locator;
every test the diff adds fails on the defect it names (both the secret- and
public-section CLI tests, verified RED under their stated mutations); exit
codes and all pre-existing refusals unchanged; fmt clean; clippy shows only the
pre-existing lint; the only whole-crate test failures are the known box-local
`history_purge` trio (missing zsh), unrelated to this change.
