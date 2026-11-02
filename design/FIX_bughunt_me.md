# FIX: me bug hunt 2026-10-31 (small-fix track)

Inputs: grade and rulings `design/agent-reports/grade-bughunt-me.md` (R1-R5
bind), triage `design/agent-reports/bughunt-triage-me.md`, RED tests
`crates/me-cli/tests/bughunt.rs`. Base: branch `bughunt/triage` at the commit
that adds this document. Scope: #4, #2, L5, #3 and #1 (coordinator decision);
L7 is not fixed here.

**#4 is blocked by STOP S1/S3 below. Build #1, #2, #3 and L5 now; build #4
only after the architect answers S1 and S3.**

Toolchain for every command: `export PATH=$HOME/.cargo/bin:$PATH`, then
`cargo +1.85.0 ...`. Never `--release`. Do not touch `firmware/`,
`third_party/` or the submodule.

## 1. Code changes (exact)

Every change below was applied to a scratch worktree at the base commit and
passed the gate in section 5 (the one failing test is S3's). Apply the hunks
as written; `cargo fmt` is already applied to them.

### L5 (R4): `crates/me-cli/src/seal/uf2.rs`

Constant: `seal::wire::REGION_LEN: u64 = 65_536`, defined at
`crates/me-cli/src/seal/wire.rs:22`. Use fix-05's placement with this
constant, not fix-05's own constant.

After the module doc comment (line 5), add:

```rust

use super::wire::REGION_LEN;
```

As the first statement of `pub fn to_uf2(blob: &[u8]) -> Vec<u8>` (line 22),
before the existing `debug_assert!`, add:

```rust
    assert!(
        blob.len() as u64 <= REGION_LEN,
        "seal blob {} bytes exceeds the {REGION_LEN}-byte seal region",
        blob.len()
    );
```

Panic message for a 65537-byte blob: `seal blob 65537 bytes exceeds the
65536-byte seal region`.

### #3 (R3): `crates/me-cli/src/sysw/mod.rs`

Use fix-03b, NOT fix-03 (fix-03 removes passphrase validation; the grade
rates it Important). In `pack_deterministic_with`, immediately before the
final `seal_with(payload, pub_bytes, header, pass, iterations, salt, iv)`
(line 530), after every passphrase check, add:

```rust
    // No secret records: nothing to encrypt, so the unsealed container (the
    // CLI's decide_sealing rule), after the passphrase checks above still ran.
    if payload.secret.is_empty() {
        let mut blob = header.encode().to_vec();
        blob.extend_from_slice(&pub_bytes);
        return bound(blob);
    }
```

### #2 (R2): `crates/me-cli/src/preview.rs`

In `render_plate`, immediately after
`let out_path = dir.join(format!("plate-{idx}.{ext}"));` (line 145) and
before `let mut child = Command::new(sidecar)`, add:

```rust
    match std::fs::remove_file(&out_path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(PreviewError::Spawn(e)),
    }
```

No new variant and no new text: `PreviewError::Spawn` displays as
`cannot run me-preview: <io error>`.

### #1 (R5): `crates/mnemonic-io-lib/src/write.rs`

1. Line 49: `opts.write(true).create(true).truncate(true);` becomes
   `opts.write(true).create(true);`.
2. After the `#[cfg(unix)]` block that calls
   `f.set_permissions(std::fs::Permissions::from_mode(0o600))?;` (line 62)
   and before `f.write_all(bytes)`, add the line `f.set_len(0)?;`. It is not
   under `#[cfg(unix)]`.
3. Doc comment lines 37-40 (the paragraph beginning "`truncate(true)` is
   load-bearing") become:

```rust
/// Truncation is load-bearing and not a stylistic echo of `std::fs::write`:
/// without it a **shrinking** overwrite — a smaller manifest written over a
/// larger one — leaves the tail of the old file in place, which for a JSON
/// artifact means trailing bytes after the closing brace. It is done by
/// `set_len(0)` after `set_permissions`, not by `truncate(true)` at open, so a
/// refused chmod leaves the existing file unchanged.
```

4. Test text only (behaviour unchanged), unit test
   `a_shrinking_overwrite_leaves_no_stale_tail` (lines 125-141): in its doc
   comment, "Dropping `truncate(true)`" becomes "Dropping the `set_len(0)`";
   its assert message "without truncate(true) the tail of the longer write
   survives" becomes "without set_len(0) the tail of the longer write
   survives".

### #4 (R1): `crates/me-cli/src/main.rs` (BLOCKED: S1, S3)

As ruled (fix-04): at the top of the `if let Some(p) = in_path {` branch of
`read_records` (line 3044), before `std::fs::read_to_string`, add:

```rust
        if !argv.is_empty() {
            return Err((
                "pass records on argv OR via --in, not both".to_string(),
                EXIT_USAGE,
            ));
        }
```

Refusal: stderr exactly `me: pass records on argv OR via --in, not both`
followed by a newline (the caller at line 1514 adds `me: `), exit 2
(`EXIT_USAGE`), stdout empty, nothing written. Same text and code as seal's
refusal at main.rs:936-939. If the architect picks S3 option (b), the
placement changes as the ruling says; the text and exit code do not.

## 2. Spec delta (exact)

| # | file:line | old | new |
|---|---|---|---|
| #4 | `design/SPEC_systemwide_payloads.md:908` | the row's text ends `(F-416, amended 2026-08-29)** \|` | append before the closing ` \|`: ` Records on argv together with \`--in\` are refused: \`me: pass records on argv OR via --in, not both\`, exit 2.` (subject to S3) |
| #2 | `design/SPEC_me_preview_hardening.md`, A1, new paragraph after line 40 (`happens to match).`), before line 42 (`Placement:`) | (none) | `` `render_plate` removes any existing `plate-<idx>.<ext>` before it spawns `me-preview`, so the F9 check judges only this run's output; a removal error other than not-found is reported as `cannot run me-preview: <error>`. `` |
| L5 | `design/SPEC_encrypted_payload_delivery.md` §5, new paragraph after line 487 (`... confirmed empirically in §3.1.`), before `## 6.` | (none) | `` `to_uf2` refuses (panics on) a blob longer than the 64 KiB region; the CLI never passes one. `` |
| #1 | `design/SPEC_me_preview_hardening.md:106` | `` `.write(true).create(true).truncate(true)` `` | `` `.write(true).create(true)` `` (derived: R5 drops `truncate(true)`, so this text would otherwise be false) |
| #1 | `design/SPEC_me_preview_hardening.md:108` | `` **`truncate(true)` is load-bearing (R0 I2):** `` | `` Truncation is load-bearing (R0 I2) and happens after the mode is set (`set_len(0)` after `set_permissions`), so a refused chmod leaves the existing file unchanged. `` The rest of the bullet (`` `fs::write` truncates; without it a shrinking overwrite ...``) is kept. |
| #1 | `design/SPEC_me_preview_hardening.md:118-119` | `**0o600 binds on CREATE; an overwrite of a pre-existing file keeps its old mode** (R0 L3)` and the residual that follows | STOP S2 |

The #3 change has no spec delta (R3: SPEC_systemwide_payloads.md:151 already
states the rule).

## 3. RED rows

Command for every row (one row by exact name):

    cargo +1.85.0 nextest run --locked --workspace -E 'test(/(^|::)<row>$/)'

All rows in one run: `-E 'test(/^bughunt_me_/)'`. Every row below was run on
the unmodified base (log of that run: 11 run, 2 passed, 9 failed, 693
skipped; the 2 passes are 06 and 07, the FALSE POSITIVEs, which stay green).

| row | item | RED on base (panic text, `bughunt.rs:line`) | GREEN after |
|---|---|---|---|
| `bughunt_me_01_write_private_keeps_old_file_when_chmod_fails` | #1 | :52 `the write failed (chmod EPERM) yet the old file was destroyed` (left `[]`) | #1 |
| `bughunt_me_02_stale_preview_is_not_this_runs_render` | #2 | :80 `a sidecar that wrote nothing must be refused, got Ok(".../plate-1.svg")` | #2 |
| `bughunt_me_02b_unremovable_stale_preview_is_a_spawn_error` (new) | #2 | :247 `an unremovable stale plate must be a Spawn error, got Err(EmptyOutput { ..., reason: "output is not a regular file" })` | #2 |
| `bughunt_me_03_pack_passphrase_without_secrets_is_well_formed` | #3 | :105 `blob has bytes past the header's declared end (len 77, declared 61)` | #3 |
| `bughunt_me_03b_pack_passphrase_without_secrets_is_the_unsealed_form` (new) | #3 | :271 `no secrets: expected the unsealed form (61 bytes), got 77 bytes` | #3 |
| `bughunt_me_04_pack_in_file_plus_argv_does_not_drop_argv` | #4 | :135 `exit 0 but the argv record was silently dropped: "text:6162"` | #4 |
| `bughunt_me_04b_pack_in_plus_argv_refusal_text_and_exit` (new) | #4 | :296 `refusal`: left `(Some(0), "sealing:  NOT SEALED ...")`, right `(Some(2), "me: pass records on argv OR via --in, not both\n")` | #4 |
| `bughunt_me_05_uf2_oversize_blob_never_addresses_past_flash_end` | L5 | :153 `block 8192 targets 0x11000000, past the end of flash` | L5 |
| `bughunt_me_05b_uf2_cap_is_the_64k_seal_region` (new) | L5 | :321 `a 65537-byte blob was accepted` | L5 |

Why each new row exists (a ruling asserts behaviour the original row does not):
- 02b: R2's "any other error becomes `PreviewError::Spawn`, displayed as
  `cannot run me-preview: ...`", and that the unremovable entry is left alone.
- 03b: R3's "the same unsealed container the `None` path builds" (byte
  equality, not just a well-formed length).
- 04b: R1's exact text, exit 2, empty stdout, nothing written (04 only checks
  that argv is not dropped, which a merge would also satisfy).
- 05b: R4's cap is exactly `REGION_LEN`: 65536 bytes accepted, 65537 panics
  with the ruled message (05 uses 3 MiB, which a 2 MiB cap would also catch).

## 4. Tests that change

- `crates/me-cli/tests/descriptor_as.rs:135`
  `as_with_argv_and_in_together_is_usage`: FAILS under fix-04 as ruled
  (stderr is the "not both" text, not §5.1's `--as packs exactly one
  descriptor per invocation.`). What happens to it is STOP S3.
- `crates/mnemonic-io-lib/src/write.rs` unit test
  `a_shrinking_overwrite_leaves_no_stale_tail`: text only (section 1, #1
  step 4); it passes before and after.
- Must stay green, checked under the fixes: the three library
  passphrase-refusal tests in `tests/sysw_cli.rs` (605-665), the F8
  entry-refusal tests in `tests/cli.rs` (692-724),
  `the_argv_bearer_guard_precedes_the_single_document_check`
  (`descriptor_as.rs:563`).
- No snapshot changes: the workspace tracks no snapshot files (`git ls-files | grep -c '\.snap$'` prints 0).

## 5. Toolkit manual

None. mnemonic-toolkit `origin/master` (f6355e22) has no `me` chapter in
`docs/manual/src/40-cli-reference/` (41-mnemonic, 42-md, 43-ms, 44-mk-cli,
45-mr). Greps run:

    git grep -n -E "sysw pack|me sysw|--in\b|uf2|write_private|me-preview" origin/master -- docs/manual/src/40-cli-reference/
    git grep -n -E "me sysw pack.*--in|--in .*me sysw" origin/master -- docs/manual/src/

The only `me sysw pack ... --in` uses (30-workflows/3B-payload-unlock-refusals.md:47,51;
40-cli-reference/43-ms.md:695) pass no argv records, so the #4 refusal does
not change them; no text "not both" or "exactly one descriptor per
invocation" appears for `me`.

## 6. Build order and gate

Order, one commit each (GREEN for its rows): L5, #3, #2, #1, spec delta for
those four, then #4 with its spec row once S1/S3 are answered. After each
commit run that item's rows by exact name.

Gate (capture each to a file once, then grep):

1. `cargo +1.85.0 nextest run --locked --workspace`
2. `cargo +1.85.0 test --locked --workspace --doc`
3. `cargo +1.85.0 fmt --all --check`
4. `cargo +1.85.0 clippy --locked --all-targets --workspace -- -D warnings`
5. `cargo +1.85.0 nextest run --locked --workspace -E 'test(/^bughunt_me_/)'`

Expected (all five items built, S3 resolved so `as_with_argv_and_in_together_is_usage` passes or is changed as ruled):
- 1: `702 tests run: 702 passed, 2 skipped` (691 base + 7 triage rows + 4
  new rows). Measured with fix-04 as ruled and the test unchanged:
  `702 tests run: 701 passed, 1 failed, 2 skipped`, the failure being
  `as_with_argv_and_in_together_is_usage`.
- 2: exit 0 (no doctests in the workspace; each `test result:` line reads
  `0 passed; 0 failed`).
- 3, 4: exit 0.
- 5: `11 tests run: 11 passed, 693 skipped`.
Before #4 lands: 5 reads `11 tests run: 9 passed, 2 failed` (04, 04b).

## STOP sections

**S1 (#4 order against the argv bearer guard).** fix-04 refuses "not both"
at exit 2 before the argv bearer/secret guard (`read_records`, the argv
branch, R2/G-P3.5) ever runs. The comment at main.rs:1519-1524 states the
pack arm's rule: "a flag-shape usage error must not pre-empt a refusal about
material that is already in the shell's history". So `me sysw pack --in f
tx:<hex>` would exit 2 with "not both" instead of exit 3 with the bearer
refusal and its history advice. Seal (main.rs:934-938) refuses "not both"
first, so fix-04 matches seal. Question: keep fix-04's placement, or run the
argv guard first and refuse "not both" after it? (The lost advice is
secret-handling and so never gates on its own; the order is still a product
point the grade did not weigh.)

**S2 (#1, SPEC_me_preview_hardening.md:118-119).** R5 says "correct
'0o600 binds on CREATE' to cite F-244's explicit `set_permissions`" but gives
no text. Proposed: replace
`**0o600 binds on CREATE; an overwrite of a pre-existing file keeps its old mode** (R0 L3) — acceptable residual: the NDEF/manifest targets are user-named and the preview targets are forced-fresh by A1.`
with
`` `.mode(0o600)` binds only on create, so `write_private` also calls `set_permissions(0o600)` on the open file (F-244): an overwrite of a pre-existing file ends at 0o600, and a refused chmod fails the write before any byte of the old file is touched (R0 L3, superseded). ``
Needs the architect's yes or a replacement text.

**S3 (#4 against SPEC_descriptor_input.md §5.1) — Important against R1 as
written.** §5.1 (lines 905-911) is normative: "Supplying `--as` ... with both
argv and `--in`, is `EXIT_USAGE` (2): *"--as packs exactly one descriptor per
invocation."*", and the `--in` row at SPEC_systemwide_payloads.md:908 says
§5.1 governs when `--as` is present. fix-04 runs inside `read_records`,
before the `--as` check at main.rs:1526-1533, so `me sysw pack
--no-passphrase --as md1 <desc> --in f` changes from exit 2 with §5.1's text
to exit 2 with "not both" (probed on both binaries), and
`as_with_argv_and_in_together_is_usage` fails. Options:
- (a) keep fix-04; amend §5.1's sentence (drop "or with both argv and
  `--in`", or say the "not both" refusal comes first) and change the test's
  expected text to `pass records on argv OR via --in, not both`;
- (b) refuse "not both" only when `--as` is absent (e.g. pass `r#as.is_some()`
  into the check, or move it after the `--as` block), keeping §5.1 and the
  test unchanged; the #4 spec sentence then says "Without `--as`, records on
  argv together with `--in` are refused: ...".
Either needs exact text from the architect.
