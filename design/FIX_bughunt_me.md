# FIX: me bug hunt 2026-10-31 (small-fix track)

Inputs: grade and rulings `design/agent-reports/grade-bughunt-me.md` (R1-R5
bind), triage `design/agent-reports/bughunt-triage-me.md`, RED tests
`crates/me-cli/tests/bughunt.rs`. Base: branch `bughunt/triage` at the commit
that adds this document. Scope: #4, #2, L5, #3 and #1 (coordinator decision);
L7 is not fixed here.

Toolchain for every command: `export PATH=$HOME/.cargo/bin:$PATH`, then
`cargo +1.85.0 ...`. Never `--release`. Do not touch `firmware/`,
`third_party/` or the submodule.

## 1. Code changes (exact)

Every change below was applied to a scratch worktree at the base commit and
passed the gate in section 5. Apply the hunks
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
   and before `f.write_all(bytes)`, add the three lines
   `if f.metadata()?.is_file() {` / `    f.set_len(0)?;` / `}` (four-space
   indent inside the function). They are not under `#[cfg(unix)]`. A pipe,
   FIFO or tty has no length to truncate (`ftruncate` returns EINVAL), and
   `truncate(true)` ignored them too.
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

### #4 (R1): `crates/me-cli/src/main.rs`

1. `read_records` signature (line 3039-3043). After `allow_argv_secret: bool,`
   add the parameter `as_present: bool,`.
2. Line 3044: `if let Some(p) = in_path {` becomes
   `if let Some(p) = in_path.filter(|_| argv.is_empty()) {`.
3. Inside `if !argv.is_empty() {`, immediately after the closing `}` of the
   `for (i, r) in argv.iter().enumerate()` guard loop (base line 3156) and
   before the comment `// §6b — \`-\` MEANS STDIN, ...`, insert:

```rust
        // R1 (bug hunt #4): argv AND --in. Checked AFTER the guard above, for
        // the pack arm's rule: a flag-shape usage error must not pre-empt a
        // refusal about material already in the shell's history.
        if in_path.is_some() {
            if !as_present {
                return Err((
                    "pass records on argv OR via --in, not both".to_string(),
                    EXIT_USAGE,
                ));
            }
            // With `--as`, the caller's §5.1 shape check refuses this
            // invocation in its own words. Read neither --in nor stdin.
            return Ok(Records {
                records: argv.to_vec(),
                document: argv.join("\n"),
            });
        }
```

4. Caller, line 1511. The call becomes
   `read_records(records, r#in.as_ref(), *allow_argv_secret, r#as.is_some())`.
   Then run `cargo +1.85.0 fmt --all`, which wraps it as
   `let (recs, document) =\n                match read_records(...) {` with the
   arms indented one level.
5. Unit tests in `mod tests`, lines 3618 and 3638. Append `, false` to each
   call, giving `read_records(std::slice::from_ref(&rec), None, false, false)`
   and `read_records(&["text:6869".to_string()], None, false, false)`.

The full diff is saved at `/scratch/cache/bughunt/me/arch-s1s3.patch`. fmt,
clippy (`-D warnings`, all targets, workspace) and the suite are all clean;
the measurements are below.

## 2. Spec delta (exact)

| # | file:line | old | new |
|---|---|---|---|
| #4 | `design/SPEC_systemwide_payloads.md:908` | the row's text ends `(F-416, amended 2026-08-29)** \|` | append before the closing ` \|`: ` Without \`--as\`, records on argv together with \`--in\` are refused after the argv bearer/secret guard: \`me: pass records on argv OR via --in, not both\`, exit 2.` |
| #2 | `design/SPEC_me_preview_hardening.md`, A1, new paragraph after line 40 (`happens to match).`), before line 42 (`Placement:`) | (none) | `` `render_plate` removes any existing `plate-<idx>.<ext>` before it spawns `me-preview`, so the F9 check judges only this run's output; a removal error other than not-found is reported as `cannot run me-preview: <error>`. `` |
| L5 | `design/SPEC_encrypted_payload_delivery.md` §5, new paragraph after line 487 (`... confirmed empirically in §3.1.`), before `## 6.` | (none) | `` `to_uf2` refuses (panics on) a blob longer than the 64 KiB region; the CLI never passes one. `` |
| #1 | `design/SPEC_me_preview_hardening.md:106` | `` `.write(true).create(true).truncate(true)` `` | `` `.write(true).create(true)` `` (derived: R5 drops `truncate(true)`, so this text would otherwise be false) |
| #1 | `design/SPEC_me_preview_hardening.md:108` | `` **`truncate(true)` is load-bearing (R0 I2):** `` | `` Truncation is load-bearing (R0 I2) and happens after the mode is set (`set_len(0)` after `set_permissions`, on a regular file only; a pipe or FIFO is written without truncation), so a refused chmod leaves the existing file unchanged. `` The rest of the bullet (`` `fs::write` truncates; without it a shrinking overwrite ...``) is kept. |
| #1 | `design/SPEC_me_preview_hardening.md:118-120` | `**0o600 binds on CREATE; an overwrite of a pre-existing file keeps its old mode** (R0 L3)` and the residual that follows | `` `.mode(0o600)` binds only on create, so on Unix `write_private` also calls `set_permissions(0o600)` on the open file (F-244): an overwrite of a pre-existing file ends at 0o600, and a refused chmod fails the write before any byte of the old file is touched (R0 L3, superseded). `` |

The #3 change has no spec delta (R3: SPEC_systemwide_payloads.md:151 already
states the rule).

## 3. RED rows

Command for every row (one row by exact name):

    cargo +1.85.0 nextest run --locked --workspace -E 'test(/(^|::)<row>$/)'

All rows in one run: `-E 'test(/^bughunt_me_/)'`. Every row below was run on
the unmodified base (log of that run: 13 run, 2 passed, 11 failed, 693
skipped; the 2 passes are 06 and 07, the FALSE POSITIVEs, which stay green).

| row | item | RED on base (panic text, `bughunt.rs:line`) | GREEN after |
|---|---|---|---|
| `bughunt_me_01_write_private_keeps_old_file_when_chmod_fails` | #1 | :52 `the write failed (chmod EPERM) yet the old file was destroyed` (left `[]`) | #1 |
| `bughunt_me_01b_pack_out_to_a_fifo_still_writes` (new) | #1 | PASSES on base (regression guard, not a RED row); with #1 step 2 as first written it fails at the exit assert, stderr `me: <ff>: Invalid argument (os error 22)` | #1 (stays green) |
| `bughunt_me_02_stale_preview_is_not_this_runs_render` | #2 | :80 `a sidecar that wrote nothing must be refused, got Ok(".../plate-1.svg")` | #2 |
| `bughunt_me_02b_unremovable_stale_preview_is_a_spawn_error` (new) | #2 | :247 `an unremovable stale plate must be a Spawn error, got Err(EmptyOutput { ..., reason: "output is not a regular file" })` | #2 |
| `bughunt_me_03_pack_passphrase_without_secrets_is_well_formed` | #3 | :105 `blob has bytes past the header's declared end (len 77, declared 61)` | #3 |
| `bughunt_me_03b_pack_passphrase_without_secrets_is_the_unsealed_form` (new) | #3 | :271 `no secrets: expected the unsealed form (61 bytes), got 77 bytes` | #3 |
| `bughunt_me_04_pack_in_file_plus_argv_does_not_drop_argv` | #4 | :135 `exit 0 but the argv record was silently dropped: "text:6162"` | #4 |
| `bughunt_me_04b_pack_in_plus_argv_refusal_text_and_exit` (new) | #4 | :296 `refusal`: left `(Some(0), "sealing:  NOT SEALED ...")`, right `(Some(2), "me: pass records on argv OR via --in, not both\n")` | #4 |
| `bughunt_me_04c_argv_bearer_guard_precedes_not_both` (new) | #4 | :352 `left: Some(0) right: Some(3)` (argv dropped, exit 0) | #4 |
| `bughunt_me_04d_as_argv_bearer_guard_precedes_single_document_with_in` (new) | #4 | :372 `left: Some(2) right: Some(3)`, stderr `me: --as packs exactly one descriptor per invocation.` | #4 |
| `bughunt_me_05_uf2_oversize_blob_never_addresses_past_flash_end` | L5 | :153 `block 8192 targets 0x11000000, past the end of flash` | L5 |
| `bughunt_me_05b_uf2_cap_is_the_64k_seal_region` (new) | L5 | :321 `a 65537-byte blob was accepted` | L5 |

Why each new row exists (a ruling asserts behaviour the original row does not):
- 01b: R5 must not regress `--out` to a pipe or FIFO. Body, at the end of
  `crates/me-cli/tests/bughunt.rs`: mkfifo `ff` in a tempdir; a reader thread
  `std::fs::read(ff)`; run `me sysw pack --no-passphrase --out <ff> text:6869`;
  assert exit 0 ("pack to a FIFO must succeed: <stderr>") and that the FIFO
  bytes equal a `--out reg.bin` run's bytes.
- 02b: R2's "any other error becomes `PreviewError::Spawn`, displayed as
  `cannot run me-preview: ...`", and that the unremovable entry is left alone.
- 03b: R3's "the same unsealed container the `None` path builds" (byte
  equality, not just a well-formed length).
- 04b: R1's exact text, exit 2, empty stdout, nothing written (04 only checks
  that argv is not dropped, which a merge would also satisfy).
- 05b: R4's cap is exactly `REGION_LEN`: 65536 bytes accepted, 65537 panics
  with the ruled message (05 uses 3 MiB, which a 2 MiB cap would also catch).

## 4. Tests that change

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
those four, then #4 with its spec row. After each
commit run that item's rows by exact name.

Gate (capture each to a file once, then grep):

1. `cargo +1.85.0 nextest run --locked --workspace`
2. `cargo +1.85.0 test --locked --workspace --doc`
3. `cargo +1.85.0 fmt --all --check`
4. `cargo +1.85.0 clippy --locked --all-targets --workspace -- -D warnings`
5. `cargo +1.85.0 nextest run --locked --workspace -E 'test(/^bughunt_me_/)'`

Expected (all five items built, S3 ruled (b), so `as_with_argv_and_in_together_is_usage` passes unchanged):
- 1: `705 tests run: 705 passed, 2 skipped` (691 base + 7 triage rows + 7
  new rows).
- 2: exit 0 (no doctests in the workspace; each `test result:` line reads
  `0 passed; 0 failed`).
- 3, 4: exit 0.
- 5: `14 tests run: 14 passed, 693 skipped`.
Before #4 lands: 5 reads `14 tests run: 10 passed, 4 failed` (04, 04b, 04c, 04d;
derived: the run of 13 plus 01b, which passes without #4).

## STOP sections

**S1:** Ruled: design/agent-reports/architect-bughunt-me.md

**S2:** Ruled: design/agent-reports/architect-bughunt-me.md

**S3:** Ruled: design/agent-reports/architect-bughunt-me.md
