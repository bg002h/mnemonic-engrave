# Mutation testing — Plan A (`me seal` / `me hash`), encrypted payload delivery

**Run:** 2026-08-07, branch `feat/encrypted-payload-hostA`, worktree
`/scratch/code/shibboleth/me-wt-seal`, HEAD `ae722e8` (Task 9 committed).
**Required by:** SPEC §11.3 and the "Mutation testing (required before Plan A is
done)" table in `design/IMPLEMENTATION_PLAN_encrypted_payload_hostA.md`.

**Baseline before mutating:** `cargo test -p mnemonic-engrave` →
116 + 1 + 30 + 1 + 3 + 1 + 6 + 11 + 0 = **169 passed, 0 failed**.

## Procedure — both non-negotiable rules honoured

- **File copy, never `git checkout`.** Each mutant copies the target to
  `<file>.mutbak`, edits in place, then restores by copying the backup back and
  deleting it. `git checkout` is never invoked. Verified afterwards:
  `git status --short` and `git diff --stat` are both **empty**, and no
  `*.mutbak` file remains.
- **The substitution is asserted before the test runs.** The driver counts
  occurrences of the exact `old` string and aborts the mutant as
  `HARNESS-ERROR` unless the count is exactly **1**. No mutant reported a
  harness error, so every "SURVIVED" below is a real survival and not a
  silently-failing edit.
- **Stale-binary trap.** Every write and restore is followed by `os.utime(path)`.
  After the whole run the sources were `touch`ed again and the suite re-run: a
  `Compiling mnemonic-engrave v0.4.0` line appeared and the suite returned
  **169 passed, 0 failed**, so the restore is real and nothing was measured
  against a stale artifact.

A mutant counts as KILLED when its **named** killer test exits non-zero. The
result line cargo printed is recorded verbatim; nothing here is hand-counted.

## Results — 31 mutants applied

| # | Mutant | Named killer | Result | cargo's line |
| --- | --- | --- | --- | --- |
| M01 | `derive_key` ignores `iterations` (hardcodes 100_000) | `vector_b_differs_only_in_iterations` | **KILLED** | FAILED. 0 passed; 1 failed |
| M02 | `seal()` reuses a fixed (all-zero) salt | `two_seals_of_the_same_payload_differ_everywhere` | **KILLED** | FAILED. 0 passed; 1 failed |
| M03 | `sealed` byte dropped from the hash input | `pubhash::sealed_and_unsealed_differ` | **KILLED** | FAILED. 0 passed; 1 failed |
| M03b | (same mutant) | `pubhash::matches_the_pinned_literals` | **KILLED** | FAILED. 0 passed; 1 failed |
| M04 | hash over a subset of the section (`input[1..]`) | `pubhash::matches_the_pinned_literals` | **KILLED** | FAILED. 0 passed; 1 failed |
| M04b | (same mutant) | `every_byte_of_the_section_affects_the_hash` | **KILLED** | FAILED. 0 passed; 1 failed |
| M05 | `public_record_count` dropped from the hash input | `pubhash::matches_the_pinned_literals` | **KILLED** | FAILED. 0 passed; 1 failed |
| M05b | (same mutant — **plan-predicted control**) | `removing_a_record_changes_the_hash` | *SURVIVED (expected)* | ok. 1 passed; 0 failed |
| M06 | `encode_section` appends a trailing LF | `joins_with_lf_and_no_trailing_lf` | **KILLED** | FAILED. 0 passed; 1 failed |
| M07 | `validate_record` strips separators instead of refusing | `refuses_space_grouped_and_hyphenated_records` | **KILLED** | FAILED. 0 passed; 1 failed |
| M08 | uppercase check removed from `validate_record` | `refuses_uppercase_records` | **KILLED** | FAILED. 0 passed; 1 failed |
| M09 | per-record decode instead of per-card-set | `decodes_a_complete_card_set` | **KILLED** | FAILED. 0 passed; 1 failed |
| M10 | decode check removed entirely | `refuses_a_bch_valid_but_undecodable_record` | **KILLED** | FAILED. 0 passed; 1 failed |
| M11 | `Header::decode` skips the `ct_len` bound | `rejects_out_of_range_lengths` | **KILLED** | FAILED. 0 passed; 1 failed |
| M12 | `reserved` check deleted | `rejects_bad_magic_version_reserved_kdf_and_aead` | **KILLED** | FAILED. 0 passed; 1 failed |
| M12b | sealed-shape `kdf_id`/`aead_id` checks deleted | `rejects_bad_magic_version_reserved_kdf_and_aead` | **KILLED** | FAILED. 0 passed; 1 failed |
| M13 | `aad = header` only — public section left out of the AAD | `flipping_a_public_section_byte_fails_the_tag` | **SURVIVED — see below** | ok. 1 passed; 0 failed |
| M14 | a vector emits an unparseable blob (`pub_len` written as 0) | `every_encrypted_vector_round_trips` | **KILLED** | FAILED. 0 passed; 1 failed |
| M15 | CR normalised away instead of refused | `refuses_embedded_separators_and_bad_lengths` | **KILLED** | FAILED. 0 passed; 1 failed |
| M16 | `--group-size 0` guidance lost on the secret path | `refuses_space_grouped_input_with_an_actionable_message` | **KILLED** | FAILED. 0 passed; 1 failed |
| M17 | unsealed-shape zero checks removed | `rejects_nonzero_crypto_fields_when_unsealed` | **KILLED** | FAILED. 0 passed; 1 failed |
| M18 | grouping by HRP instead of `(HRP, chunk_set_id)` | `vector_g_multisig_public_section_spans_four_cards` | **KILLED** | FAILED. 0 passed; 1 failed |
| M18b | (same mutant — **plan-predicted control**) | `record::decodes_a_complete_card_set` | *SURVIVED (expected)* | ok. 1 passed; 0 failed |
| M19 | the combined 1..24 cap deleted | `refuses_more_than_24_records_across_both_sections` | **KILLED** | FAILED. 0 passed; 1 failed |
| M20 | uppercase BIP-39 mnemonic emitted (`record_or_mnemonic` guard deleted) | `refuses_an_uppercase_bip39_mnemonic` | **KILLED** | FAILED. 0 passed; 1 failed |
| M21 | `--iterations` check moved below the public-only early return | `refuses_out_of_range_iterations_on_the_public_only_path` | **KILLED** | FAILED. 0 passed; 1 failed |
| M22 | CR trimmed at the CLI before the container sees it | `refuses_a_record_carrying_a_cr` | **KILLED** | FAILED. 0 passed; 1 failed |
| M23 | printed hash computed over untrimmed argv | `printed_hash_matches_me_hash_regardless_of_surrounding_whitespace` | **KILLED** | FAILED. 0 passed; 1 failed |
| M24 | `to_uf2` emits family `0xE48BFF59` | `every_block_conforms_not_just_the_first` | **SURVIVED — REAL GAP** | ok. 1 passed; 0 failed |
| M24b | `to_uf2` pads with `0xFF` | `every_block_conforms_not_just_the_first` | **KILLED** | FAILED. 0 passed; 1 failed |
| M25 | `open_bytes` returns plaintext without checking the tag | `open_fails_on_flipped_ciphertext_byte` | **KILLED** | FAILED. 0 passed; 1 failed |
| M26 | passphrase generated for a public-only payload | `public_only_payload_prints_no_passphrase` | **KILLED** | FAILED. 0 passed; 1 failed |
| M27 | `ms1` opt-in ignored | `refuses_ms1_without_the_opt_in_flag` | **KILLED** | FAILED. 0 passed; 1 failed |
| M28 | secret admitted to the public section | `seal::tests::refuses_a_secret_in_the_public_section` | **KILLED** | FAILED. 0 passed; 1 failed |
| M28b | (same mutant, CLI test of the same name) | `seal_cli::refuses_a_secret_in_the_public_section` | *SURVIVED — weak test, see below* | ok. 1 passed; 0 failed |

**Score against the plan's table: 27 of 29 distinct mutant/killer pairs killed by
the named test.** Two survivals (M05b, M18b) are controls the plan itself
predicted. Three findings follow.

---

## Finding 1 — **M24 escapes the ENTIRE suite.** The UF2 family-ID assertion is self-referential.

`every_block_conforms_not_just_the_first` asserts

```rust
assert_eq!(field(b, 28), FAMILY_DATA, "block {i} familyID (data, NOT rp2350_arm_s)");
```

`FAMILY_DATA` is in scope via `use super::*` — it is **the very constant under
test**. Changing `pub const FAMILY_DATA` from `0xE48B_FF58` to `0xE48B_FF59`
changes both sides of the comparison, so the assertion holds under the mutant.
Measured, not reasoned: the whole-suite run under M24 exits **0** with **no
failing test**.

Every other field in that test is pinned to a hex literal
(`0x0A32_4655`, `0x9E5D_5157`, `0x0000_2000`, `256`, `0x0AB1_6F30`) — the family
ID is the one exception, and it is the field §9.1 calls out by name and the one
the plan's own doc comment warns about ("NOT `0xe48bff59` (`rp2350_arm_s`), the
bootable-image family the TinyGo target uses").

This is the only mutant in the run that no test anywhere catches. Note the other
half of the same table row — `0xFF` padding, M24b — **is** killed, so the row
reads green if you only run it once.

**Impact.** A wrong family ID is not cosmetic: `0xE48BFF59` is the *bootable
image* family. `picotool load` of a UF2 claiming that family targets the signed
firmware region rather than a data region, which is exactly the failure mode §5
and §9's "no `--addr` flag" analysis exist to prevent.

**Suggested fix** (NOT applied — the plan is normative and this is a plan-level
defect, not a transcription slip): pin the literal, e.g. add to that test

```rust
assert_eq!(FAMILY_DATA, 0xE48B_FF58, "the data family ID is normative (§9.1)");
assert_eq!(TARGET_ADDR, 0x10E0_0000, "the target address is normative (§5)");
```

`TARGET_ADDR` has the identical shape — `assert_eq!(field(b, 12), TARGET_ADDR +
i as u32 * 256)` — and was not separately mutated here, but it is
self-referential by the same argument and is a *destructive* constant (§9: past
`0x11000000` a write wraps to `0x10000000` and destroys the firmware).

---

## Finding 2 — M13's named killer is the wrong test. The mutant is caught, but not by what the table says.

The plan's row reads:

> | `aad = header` only, dropping the public section | `flipping_a_public_section_byte_fails_the_tag` (the pinned D/E sha256s catch it too, but nothing else tests the §6.1a property itself) |

`flipping_a_public_section_byte_fails_the_tag` **cannot** see this mutant, and
the reason is structural rather than incidental: the test builds the AAD itself
in the test body —

```rust
crypto::open_bytes(&key, &h.iv, &bad[..split], &bad[split..])
```

— so it exercises `open_bytes`, never `seal_deterministic`'s AAD construction. A
seal-side-only mutation makes the open-side AAD *disagree* with the seal-side
one, the tag mismatches, `is_err()` is satisfied, and the test goes green over
the defect.

The mutant is nonetheless **caught by the suite**. Whole-suite run under M13,
failing tests measured:

```
seal::tests::vector_d_mixed
seal::tests::vector_g_multisig_public_section_spans_four_cards
seal::tests::every_encrypted_vector_round_trips
```

i.e. the pinned blob sha256s for the two mixed-shape vectors, plus the
round-trip. So §6.1a **is** defended, by the vectors the plan's own parenthetical
already credits. The table's primary attribution is simply wrong, and correcting
it matters because the parenthetical implies the sha pins are the redundant
backstop when in fact they are the only line.

**Suggested fix** (NOT applied): reword the row to name the D/G vectors and the
round-trip as the killers, or strengthen the test so it seals under one AAD and
opens under the AAD the *implementation* produced.

---

## Finding 3 — the CLI `refuses_a_secret_in_the_public_section` is satisfied by the wrong error.

Two tests share this name. `seal::tests::refuses_a_secret_in_the_public_section`
matches the specific variant (`Err(SealError::SecretInPublic(_))`) and **kills**
M28. The CLI test of the same name in `tests/seal_cli.rs` asserts only
`.failure()` and `!out.exists()`, and under M28 the payload still fails — one
step later, in `decode_public_set`'s own secret guard, with
`UndecodableSet("a secret record cannot be in the public section")`. So the CLI
test goes green over a deleted `check_public` guard.

That is not an escape (the lib test holds the line) but it is a weak assertion of
exactly the shape the plan spends several comments warning about, and the shared
name makes the mutation table's single row ambiguous about which test is meant.
Adding `.stderr(predicate::str::contains("cannot ride in the public section"))`
would fix it.

---

## Not covered here

The plan's final row — *"only the first secret record handled"* — is marked
**not covered here**: it is a §10.2.2 device behaviour and belongs to Plan B,
which will use vector F. No attempt was made to mutate it in this crate.

## Reproduction

Drivers used for this run (scratch, not committed):
a table-driven Python harness that copies → asserts single match → mutates →
runs the named killer → restores from the copy, and a second pass that re-runs
the whole suite for each surviving mutant to characterise the gap.
