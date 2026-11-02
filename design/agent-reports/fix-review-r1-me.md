**⚠** r1: 0C/1I/1M/0N

# FIX review r1: design/FIX_bughunt_me.md (mnemonic-engrave, bughunt/triage @ bffa4f0b)

Reviewer: Opus, effort medium, round 1. Fold accounting: r1, so 0 findings from a
prior round's fix commit (the doc's fold commits 2983586c and bffa4f0b are
controller transcription of architect texts).

## Findings (appended as found)

- M1 (Minor) FIX §3 line 171: "log of that run: 11 run, 2 passed, 9 failed, 693 skipped"
  is stale; the `bughunt_me_` filter on base runs 13 (2 passed: 06, 07; 11 failed;
  693 skipped). Fix: replace with "13 run, 2 passed, 11 failed, 693 skipped". Run:
  review-me-r1-red.log `13 tests run: 2 passed, 11 failed, 693 skipped`.

- **I1 (Important) FIX §1 #1 step 2 (FIX_bughunt_me.md:84-87), with §2 row #1
  (spec delta for SPEC_me_preview_hardening.md:108), §3 and §6.** Followed
  literally, the unconditional `f.set_len(0)?;` regresses `me sysw pack --out`
  to any non-regular file the holder owns: `ftruncate` on a pipe/FIFO/tty is
  EINVAL. Base (`truncate(true)` = O_TRUNC, ignored on a FIFO) works; the doc's
  text fails closed with no file written.
  Scenario (run on the scratch tree with §1 applied exactly):
  `me sysw pack --out /dev/stdout text:6869 | wc -c` -> rc 2,
  `me: /dev/stdout: Invalid argument (os error 22)`, 0 bytes (installed
  `me 0.12.0` and base: rc 0, 61 bytes). Same with `--out <mkfifo path>`.
  One unusual condition only (a non-regular `--out`, which `--help` invites by
  steering holders away from `>`), so the likelihood test does not apply.
  Fix text (run):
  1. §1 #1 step 2: replace "add the line `f.set_len(0)?;`. It is not under
     `#[cfg(unix)]`." with "add the three lines
     `if f.metadata()?.is_file() {` / `    f.set_len(0)?;` / `}` (four-space
     indent inside the function). They are not under `#[cfg(unix)]`. A pipe,
     FIFO or tty has no length to truncate (`ftruncate` returns EINVAL), and
     `truncate(true)` ignored them too."
  2. §2 row #1 new text: "Truncation is load-bearing (R0 I2) and happens after
     the mode is set (`set_len(0)` after `set_permissions`, on a regular file
     only; a pipe or FIFO is written without truncation), so a refused chmod
     leaves the existing file unchanged."
  3. §3: add row `bughunt_me_01b_pack_out_to_a_fifo_still_writes` (new, item
     #1) at the end of `crates/me-cli/tests/bughunt.rs`: mkfifo `ff` in a
     tempdir; a reader thread `std::fs::read(ff)`; run
     `me sysw pack --no-passphrase --out <ff> text:6869`; assert exit 0
     ("pack to a FIFO must succeed: <stderr>") and that the FIFO bytes equal a
     `--out reg.bin` run's bytes. It PASSES on base (a regression guard, not a
     RED row); with step 2 as first written it fails at the exit assert,
     stderr `me: <ff>: Invalid argument (os error 22)`. "Why" line: "01b: R5
     must not regress `--out` to a pipe or FIFO."
  4. §6 expected: 1 reads `705 tests run: 705 passed, 2 skipped` (… + 7 new
     rows); 5 reads `14 tests run: 14 passed, 693 skipped`; before #4 lands
     5 reads `14 tests run: 10 passed, 4 failed` (derived: the run of 13 plus 01b, which passes without #4).
  Run: doc text -> `1 test run: 0 passed, 1 failed` (panic bughunt.rs:400,
  exit assert); base write.rs -> `1 passed`; fix -> `1 passed`. Full gate with
  every §1 item + fix: g1 `705 tests run: 705 passed, 2 skipped`, fmt/clippy
  exit 0, g5 `14 tests run: 14 passed, 693 skipped`. Pipe and FIFO probes:
  rc 0, 61 bytes, identical to the regular-file output (mode 600).

## Case tables (doc quoted; all run on the scratch tree)

L5 (uf2.rs, "assert! blob.len() as u64 <= REGION_LEN", REGION_LEN = 65_536 in
seal/wire.rs:22, TARGET_ADDR 0x10E0_0000):
| input | decided | observed |
|---|---|---|
| 65536-byte blob | accepted, 256 blocks | 05b: `uf2.len() == 131072` passes |
| 65537-byte blob | panic "seal blob 65537 bytes exceeds the 65536-byte seal region" | 05b passes (exact message) |
| 3 MiB blob | panic (never addresses past 0x11000000) | 05 passes |
0x10E00000 + 0x10000 = 0x10E10000, inside flash.

#3 (sysw/mod.rs, passphrase with no secret records -> "the same unsealed
container the `None` path builds"):
| records | passphrase | decided | observed |
|---|---|---|---|
| public only | Some | unsealed, 61 bytes, == None path | 03, 03b pass |
| any secret | Some | sealed (unchanged) | S-C/S-D/S-E vectors unchanged |
| public only | None | unsealed (unchanged) | suite green |
The CLI never passes a passphrase without secrets (`decide_sealing`), so #3
reaches only library callers.

#2 (preview.rs, stale `plate-{idx}.{svg|png}` removed before spawn; other
remove error -> `PreviewError::Spawn`):
| state | decided | observed |
|---|---|---|
| stale plate, sidecar writes nothing | F9 refusal, not Ok(stale) | 02 passes |
| stale plate unremovable | Spawn error, entry left alone | 02b passes |
| foreign plate-* in CLI out dir | F8 exit 2 before render | cli.rs dirty_preview_dir_refused_exit_2 passes |
The removal touches only names the F8 scan already refuses in the CLI, so the
CLI never deletes a holder file.

#1 (write.rs): chmod EPERM -> old file intact (01 passes); shrinking overwrite
-> no tail (unit test passes); new file -> 0600; FIFO/pipe -> see I1.

#4 (main.rs, S1 guard first exit 3, then "not both" exit 2 only without
`--as`; r.txt holds `text:6869`, stdin /dev/null):
| argv | --in | --as | rc | stderr | written |
|---|---|---|---|---|---|
| text | - | - | 0 | - | p.bin |
| - | r.txt | - | 0 | - | p.bin |
| text | r.txt | - | 2 | me: pass records on argv OR via --in, not both | nothing |
| tx:00 | r.txt | - | 3 | bearer refusal | nothing |
| `-` | r.txt | - | 2 | not both | nothing |
| text | r.txt | md1 | 2 | --as packs exactly one descriptor per invocation. | nothing |
| tx:00 | r.txt | md1 | 3 | bearer refusal | nothing |
| `-` | r.txt | md1 | 2 | §5.1 message | nothing |
| text | nonexist | - | 2 | not both | nothing |
| text | nonexist | md1 | 2 | §5.1 message | nothing |
All as ruled (S1, S3(b)); `as_with_argv_and_in_together_is_usage` and
`the_argv_bearer_guard_precedes_the_single_document_check` pass.

## Run what you print

- Base: `test(/^bughunt_me_/)` `13 tests run: 2 passed, 11 failed, 693 skipped`;
  every panic line/text matches the §3 table (see M1 for the stale header).
- After L5, #3, #2, #1: `13 tests run: 9 passed, 4 failed` (04, 04b, 04c, 04d), as §6.
- All five items: g1 `704 tests run: 704 passed, 2 skipped`; g2 exit 0, two
  `0 passed; 0 failed` lines; g3, g4 exit 0 (clippy `suspicious_open_options`
  does not fire on `write(true).create(true)`); g5 `13 tests run: 13 passed,
  693 skipped`. All match §6 (before I1's row is added).
- main.rs diff equals the architect patch byte for byte after `cargo fmt`.
- §2 anchors: SPEC_systemwide_payloads.md:908, SPEC_me_preview_hardening.md:40,
  42, 106, 108, 117-120, SPEC_encrypted_payload_delivery.md:487/489 all exist
  and are unique.
- §4: sysw_cli.rs 608/631/657, cli.rs 697, descriptor_as.rs:563 exist and pass;
  `git ls-files | grep -c '\.snap$'` prints 0.
- §5: mnemonic-toolkit origin/master f6355e22; both greps run; the only
  `me sysw pack ... --in` uses are 3B-payload-unlock-refusals.md:47,51 and
  43-ms.md:695, none with argv records. "None" holds. No manual uses
  `--out /dev/stdout` or a FIFO.

## Create-input table

| printed command (source) | flags vs clap `me sysw pack` | result |
|---|---|---|
| `me sysw pack --in records.txt --out p.bin` (bearer refusal, main.rs:646, unchanged) | --in, --out exist | runs (rc 0 with a records file) |
| `me sysw pack --allow-argv-secret` (main.rs:649, unchanged) | exists | flag accepted |
| "not both" refusal | prints no command | - |
| manual 3B:47/51, 43-ms:695 (`--pack-preimage --passphrase-words 4/--no-passphrase --in records.txt --out payload.bin`) | all flags exist; no argv records | unaffected by #4 |

## Design concern (ungraded)

- `write_private` fchmods any holder-owned `--out` target to 0600, so
  `--out /dev/tty` changes the holder's terminal mode (base too; not in scope).

Fold accounting: r1; 0 of 0C/1I/1M/0N came from a prior round's fix commit
(2983586c and bffa4f0b were controller transcription of architect texts).

0 / 1 / 1 / 0
