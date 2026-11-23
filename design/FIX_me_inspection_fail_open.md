# FIX: `region_image` refuses a short container, and `me sysw show` refuses a public section that is not UTF-8 (bh-me-I7, bh-me-I8)

- Source: bughunt 2026-11-19 keys **bh-me-I7** and **bh-me-I8**, both graded **Important**
  (C/agent-reports/bughunt-1119/grade-me.md:70-71; `design/agent-reports/bug-hunt-2026-11-19.md` §I7, §I8,
  lines 90-100). Architect table row #51, **small** track, one Opus review round. Not class B. No T-ruling is
  named for this row; P7 (library-only reach, recommended I) covers I7's grade, not its fix shape.
- Base: master `e09d538dcd0b552cdc58fe7ff228e2eca5f7f64e`. The graded head was `ff5ecfea`; `region_image` moved from
  sysw/mod.rs:694 to :739 and the `show` arm from main.rs:2011 to :2017 (MNEMBLOB v2 and io-lib safe-write
  landed in between). The code of both is unchanged. Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22.
- One new holder-visible text (§4, **T-UTF8**), **PROPOSED (architect confirms)**. No new flag, type, variant or
  command. §7 lists the points this document decides that the bug report and rulings do not literally state.
- **Files the build changes** (for build ordering against other me rows): `crates/me-cli/src/sysw/mod.rs`,
  `crates/me-cli/src/main.rs`, `crates/me-cli/tests/sysw_cli.rs`, `crates/me-cli/CHANGELOG.md`,
  `design/SPEC_systemwide_payloads.md`. This document and the gate script
  `scripts/fix-me-inspection-fail-open-gate.sh` are already committed on this branch.
- The toolkit manual (`mnemonic-toolkit` master `docs/manual/src/40-cli-reference/`) does not mention `me sysw show`
  or `region_image` (`git grep -n -E "sysw show|region_image" origin/master -- docs/manual` prints nothing), so there
  is no lockstep text.

## 1. The defects (measured on master e09d538d)

### 1.1 I7: `sysw::region_image` turns a short container into a full 64 KiB image

`crates/me-cli/src/sysw/mod.rs:739-747` checks only `blob.len() > REGION_LEN` and `Header::parse`. `open` at
:750-754 also refuses `blob.len() < h.total_len()` with `SyswError::Wire(WireError::TooShort(blob.len()))`.
So a container missing its last public byte (unsealed) or its last tag byte (sealed) gets `Ok` and a 65,536-byte
image whose missing bytes are `0xFF` padding; `open` on that image fails. The doc comment (:736-738) says the
function refuses "one the reader would not parse". The only CLI caller (`me sysw pack --region`, main.rs:1934)
passes its own freshly packed blob, so no `me` command reaches this; the library is the reach (P7).

### 1.2 I8: `me sysw show` exits 0 with an identity for a public section that is not UTF-8

The `Show` arm (main.rs:2017-2060) prints the header, refuses truncation (:2043-2052), then prints `identity:`,
calls `print_digest` (:2095) and `print_records` (:2333). Both printers drop a non-UTF-8 public section with a
silent `let Ok(s) = std::str::from_utf8(section) else { return; };` (:2119-2121 and :2342-2344). The command
prints a 64-digit identity, no digest, no records, empty stderr, and exits 0. `open` refuses the same section with
`SyswError::NotUtf8` (sysw/mod.rs:757, :782).

## 2. Fix shape

- **I7.** In `region_image`, after `Header::parse` and before the allocation, refuse
  `blob.len() < h.total_len()` with `SyswError::Wire(wire::WireError::TooShort(blob.len()))`, the error `open`
  returns for the same input. A blob longer than `total_len` up to `REGION_LEN` stays accepted (a padded image
  round-trips; the existing test `region_image_pads_with_erased_flash_and_refuses_what_bound_refuses` pins it).
  The doc comment's "Refuses" sentence is replaced by: "Refuses what [`bound`] refuses (a blob larger than the
  region, and one `Header::parse` would not parse) and also one shorter than its header declares (the `TooShort`
  `open` returns)." The CHANGELOG entry (§6) records the behaviour change.
- **I8.** In the `Show` arm, after the truncation refusal and before the `identity:` line, decode the public
  section (empty when `pub_len == 0`) `blob[HEADER_LEN..HEADER_LEN + pub_len]` with `std::str::from_utf8`. On `Err(e)`,
  print T-UTF8 to stderr with `{o}` = `HEADER_LEN + e.valid_up_to()` and return `EXIT_INVALID` (4). On `Ok(s)`,
  pass `s` to `print_records` in place of `(&blob, &h)` (its `pub_len == 0` early return becomes `s.is_empty()`;
  its silent `from_utf8` arm is deleted). `print_digest` keeps its signature because `pack` calls it on its own
  blob; its silent arm stays and gets the comment "unreachable from `show`, which refuses a non-UTF-8 public
  section first (FIX_me_inspection_fail_open); `pack` builds the section from `String`s".
- Order in `show` after this FIX: read error (exit 2) → header parse (exit 4) → header lines on stdout → truncation
  (exit 4) → **UTF-8 (exit 4)** → `identity:` → digest → records → exit 0.

## 3. Spec delta (`design/SPEC_systemwide_payloads.md`)

- **§5.6**, after the paragraph that begins "**`me sysw show` exists so the operator can re-derive the digest**"
  and ends "…they must have transcribed correctly the first time.", insert one paragraph:
  `**AMENDED (FIX_me_inspection_fail_open, bughunt-1119 I8):** \`show\` refuses, at exit 4, a container whose
  public section is not UTF-8, after the truncation check and before any identity or digest is printed. It used to
  print an identity, skip the digest and the record list in silence, and exit 0. The refusal names the file offset
  where UTF-8 decoding first fails. A sealed container is checked the same way: its public section is read
  without a passphrase. A container with no public section has nothing to check.`

## 4. Printed texts and exit codes

| id | stream | exit | text | status |
|---|---|---|---|---|
| T-UTF8 | stderr, one line | 4 (`EXIT_INVALID`) | ``me: this file's public section is not UTF-8 text (where UTF-8 decoding first fails, at file offset {o}); it is damaged, or it was not written by `me sysw pack`. No identity or digest is shown, because either would be a number for records that cannot be read.`` | **PROPOSED (architect confirms)** |

`{o}` is a decimal byte offset from the start of the file. The text avoids "payload unreadable" (spec §5.2 and the
comment at main.rs:2028-2030) and does not claim tampering. Every other `show` text and exit code is unchanged.
The `sealed:`, `pub_len:`, `ct_len:` stdout lines still print before the refusal, as they do before the truncation
refusal today. I7 adds no text: no CLI path reaches the new refusal, and `--region`'s existing `Err(e)` arm
(main.rs:1945-1948) would print `sysw_error(&e)` unchanged.

## 5. RED rows (run on master e09d538d, signet-irrelevant: no keys involved)

R1-R2 are library rows; the author ran them from a scratch integration test (not committed; copy at
`/scratch/cache/fix-me-insp-logs/zz_red_me_inspection_fail_open.rs`) with
`cargo nextest run --locked --build-jobs 2 -j 2 -p mnemonic-engrave --test zz_red_me_inspection_fail_open --no-fail-fast`.
R3-R8 are CLI rows and run with `bash scripts/fix-me-inspection-fail-open-gate.sh --cli`. Containers:
`plain` = `me sysw pack --no-passphrase text:6162` (61 bytes, public section at offsets 52-60);
`sealed` = `me sysw pack --passphrase-words 2 --in` a file holding `text:6162` and `pass:6869` (86 bytes, public
section at 52-60); `secrets-only` = the same with `pass:6869` only (77 bytes, `pub_len: 0`).

| row | input | expected after the FIX | measured on master |
|---|---|---|---|
| R1 | `region_image(&blob[..60])`, `blob` = `pack(["text:6869"], None, MIN_ITERATIONS)` (61 bytes) | `Err(Wire(TooShort(60)))` | `Ok`, image length 65536 — RED |
| R2 | `region_image(&blob[..n-1])`, `blob` = `pack(["text:6162","pass:6869"], Some(PASS), MIN_ITERATIONS)`, `n` = 86 = `total_len` (last tag byte missing) | `Err(Wire(TooShort(85)))` | `Ok`, image length 65536 — RED |
| R3 | `me sysw show` on `plain` with offset 60 set to `0xff` | exit 4; T-UTF8 with `{o}` = 60; no `identity:`, no `digest:`, no `public record` line | exit 0; `sealed:   false`, `pub_len:  9`, `ct_len:   0`, `identity: edcbba71…c77a`; stderr empty — RED |
| R4 | `me sysw show` on `plain` with offset 52 set to `0x80` | exit 4; T-UTF8 with `{o}` = 52 | exit 0; `identity: 6323bacf…6ffdc` — RED |
| R5 | `me sysw show` on `sealed` with offset 60 set to `0xff` | exit 4; T-UTF8 with `{o}` = 60 | exit 0; `sealed:   true`, `identity: b33a6d76…4207` (this run's container) — RED |
| R6 | `me sysw show` on `plain` (control) | exit 0; `identity:` line; `digest:   1ad5 a49b 17a1 2dbb 7ba1 093b e9cf 09bb` | same — passes (control) |
| R7 | `me sysw show` on `secrets-only` (control) | exit 0; `digest:   none — this payload has no public section` | same — passes (control) |
| R8 | `me sysw show` on R5's file cut by one byte (85 bytes) | exit 4 with the existing truncation sentence ("…is 85 bytes but its header declares 86; it is truncated…"), no T-UTF8 | same — passes (order control) |

Gate `--cli` output on master (`GATE_LOG_DIR=/scratch/cache/fix-me-insp-logs/gate`): nine `FAIL` lines for R3, R4,
R5 (exit 0 want 4, missing T-UTF8, printed an identity), `ok` for R6, R7, R8, then `gate --cli: FAIL`.

## 6. Tests

New (each carries the row it covers in its doc comment; names are the gate's):

- `crates/me-cli/src/sysw/mod.rs` `mod tests`:
  - `region_image_refuses_a_blob_missing_its_last_public_byte` (R1): `assert_eq!` on the `Err` exactly as R1.
  - `region_image_refuses_a_sealed_blob_missing_its_last_tag_byte` (R2): asserts the blob is sealed and
    `total_len() == blob.len()` first, then the `Err` exactly as R2 (with the blob's own length minus one).
- `crates/me-cli/tests/sysw_cli.rs`:
  - `show_refuses_a_public_section_that_is_not_utf8` (R3): exit code 4; stderr contains T-UTF8 with `{o}` = 60 as a
    whole line; stdout has no line starting `identity:` or `public record`; `digest_line(stderr)` is `None`.
  - `show_names_the_first_bad_byte_of_the_public_section` (R4): exit 4; the T-UTF8 line with `{o}` = 52.
  - `show_refuses_a_sealed_public_section_that_is_not_utf8` (R5): the container is built in-process with
    `mnemonic_engrave::sysw::pack(vec!["text:6162".into(), "pass:6869".into()], Some(PASS), MIN_ITERATIONS)`
    (`PASS` = six `abandon`; the `--passphrase-words` CLI path prints a passphrase the test does not need) and
    written to a temp file, `pub_len == 9` asserted, offset 60 set to `0xff`; exit 4; T-UTF8 with `{o}` = 60; no
    `identity:`.
  - `show_reports_truncation_before_an_unreadable_public_section` (R8): R5's bytes minus the last; exit 4; stderr
    contains "it is truncated" and not "is not UTF-8 text".

Unchanged and must still pass (controls the gate names): `region_image_pads_with_erased_flash_and_refuses_what_bound_refuses`
(complete and padded inputs), `show_prints_the_same_digest_pack_did` (R6's shape),
`a_secrets_only_payload_reports_no_digest` (R7's shape), `show_on_a_truncated_container_fails_cleanly_without_printing_a_digest`.
No existing test changes: none packs or shows a non-UTF-8 public section, and none passes a short blob to
`region_image`.

CHANGELOG (`crates/me-cli/CHANGELOG.md`, `## [Unreleased]` → `### Fixed`, appended after the existing bullets):
`- **\`me sysw show\` refuses a public section that is not UTF-8, and \`sysw::region_image\` refuses a short
  container** (bughunt-1119 bh-me-I8 and bh-me-I7, both Important). \`show\` printed an identity, skipped the digest
  and the records in silence, and exited 0; it now exits 4 and names the file offset where UTF-8 decoding first fails, and
  prints no identity or digest. \`region_image\` padded a container shorter than its header declares into a full
  region image the reader then refused; it now returns the \`TooShort\` error \`open\` returns. Exit codes
  elsewhere are unchanged.` (**PROPOSED**, same status as T-UTF8.)

Gate: `bash scripts/fix-me-inspection-fail-open-gate.sh` (A: CLI rows R3-R8; B: the whole nextest suite once,
each named test PASS exactly once, no FAIL line; C: doc-tests, `cargo fmt --check`, both clippy runs on 1.85.0). Its
header states what it does not cover.

## 7. Points this document decides that the bug report and rulings do not literally state (PROPOSED, architect confirms)

1. **T-UTF8 wording and its file offset** (§4). The report fixes the exit (4) and "a format error"; the sentence is
   new. It is modelled on the truncation sentence at main.rs:2044-2050.
2. **Header lines still print before the refusal.** The report says "before presenting the successful inspection";
   this document reads that as before `identity:`, which is where the existing truncation refusal sits. Moving all
   checks above the header lines would change the truncation path too, which this row does not own.
3. **`region_image` gains the length check only**, not the reader's UTF-8 check on the public section. The report's
   fix and the grader (grade-me.md:70, "the fix is one length check") stop there. A complete container whose public
   section is not UTF-8 still gets an image from `region_image` (and passes `bound`); that residue is filed as a
   Minor (author report, FOLLOWUPS line `sysw-region-image-reader-parity`) rather than widened into this row.
