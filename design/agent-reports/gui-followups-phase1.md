# mnemonic-gui follow-ups, phase 1: implementation report

**Result.** Both phase-1a fixes are done: test-first, every site mutation-checked, and every gate green, including CI on the branch. The phase-1b design document is written and committed. It is unreviewed, and there is no code for it.

- **Branch:** `gui-followups` in `/scratch/code/shibboleth/gui-worktrees/followups`, off `origin/master` `9f569e1`, pushed to `origin/gui-followups`. Not tagged, not merged.

| commit | what |
|---|---|
| `5956da2` | F-685: CHANGELOG 0.60.0 + 0.61.0 reconstructed; `## Unreleased` opened |
| `e3a55a4` | `restore-from-secret-node-unmasked-and-persisted`: fix + `tests/restore_from_secret_node.rs` |
| `ec31aed` | design doc + measurement harness (`design/measurements/secret-channels/`) |

## Phase 1a

### F-685 (CHANGELOG)

Both entries were reconstructed from the tag messages and `git log mnemonic-gui-v0.59.0..v0.61.0`:

- **0.60.0:** `939d170` (#37, the BIP-322 security re-pin to toolkit v0.91.0) and `2b0e54e` (README).
- **0.61.0:** `82fc3f8` (#38, currency re-pin to v0.97.0 plus tutorial corpus).

Each entry says it was reconstructed. It states only what the commits and tags show: pin sites, the `canonicity_drift` `@N/**` fixtures, and the uniform-label tutorial regen with transcripts unchanged. The engrave-side F-685 entry is still OPEN, for the controller to close.

### `restore --from <secret-node>=…`

- **The change.** There is one classifier, `secrets::text_value_is_secret_node_token`. A value counts as secret when its node is in `SECRET_NODE_TYPES_ARGV` and its value is not empty, `-` or `@env:…`. It replaces F-679's private `invocation::is_secret_node_value_token`, so there is still one list and one predicate. It now drives:
  1. the argv mask (`emit_one`'s Text arm), which feeds Preview and the confirm body;
  2. `should_confirm_run`;
  3. `redact_for_persistence`;
  4. the Text widget: `.password(..)` plus the reveal eye, with an **explicit TextEdit id**.
- **Why the explicit id.** The field flips to masked mid-typing (at `ms1=m`) and the eye appears in front of it. With an auto id, the field is re-keyed and loses focus. The mutation check below shows this happens.
- **Wider than `restore`.** The predicate is value-based, so it also covers the other non-secret Text `--from` flags: `addresses`, `verify-bundle` and `word-card`. `restore` was the reported case.
- **Tests first.** `tests/restore_from_secret_node.rs` has 12 tests. **6 failed on `9f569e1`**, each for the right reason; the typing test got as far as the PasswordInput assert. For each site there is a positive test and an `xpub=` negative control, plus an all-`SECRET_NODE_TYPES_ARGV` sweep and an `ms1=-` sentinel control.
- **Mutation checks.** Each mutation was applied to the fixed tree, confirmed applied, and made the listed tests fail:

| mutation | failing tests |
|---|---|
| site 1 mask forced to `false` | 2 |
| site 2 confirm clause disabled | 1 |
| site 3 redaction clause disabled | 1 |
| site 4 `.password(false…)` | 2 |
| explicit id removed | typing test fails with `Text("ms1=m")`: focus lost |
| node check forced `true` | all 4 `xpub=` over-masking controls |
| sentinel check removed | the `ms1=-` control |

- **Not done:** paste-warn for this Text field is unchanged. The brief listed four sites and paste-warn was not one of them.

### Gates

| gate | result |
|---|---|
| `cargo nextest run --locked`, all four `*_BIN` = release binaries | **722 passed, 6 skipped** (was 710 + 12 new) |
| clippy `-D warnings`, `--all-targets` and `--no-default-features` | clean |
| `build --no-default-features` | clean |
| MSRV `rustup run 1.88.0 cargo check --locked` | clean |
| form snapshots (`GUI_SNAPSHOTS=1`, GL llvmpipe) | 61 rendered, **no PNG changed** |
| tutorial harness `--include-ignored` | 12 passed, no PNG changed |
| fmt | no new drift hunks in the touched files (the counts match base); the new test file is rustfmt-clean |
| **CI** `ci/gui-followups` @ `e3a55a4` | build run 36137949688, every job success: clippy, headless, msrv, snapshots, tutorial-snapshots, 7 targets; release skipped. schema-mirror 36137949686 success |

CI on the final tip `ec31aed` (docs only) is in the note at the end.

## Phase 1b: the design (`design/DESIGN_secret_channels_and_new_forms.md`)

### Part A: private channels

The per-subcommand, per-flag table is **measured**, not read off `--help`. It covers 72 input rows plus 14 multi-secret combinations. The harness is committed and a re-run reproduced it exactly.

**How each row is measured.** The channel form is compared with the argv + `--allow-argv-secret` baseline, and a second run with a **different secret must change the output**. Without that dependence check, three dangerous results would look like passes:

- **`--passphrase -` is silently the literal passphrase `-`**, with exit 0 and a different wallet. This holds on every `mnemonic` subcommand whose passphrase row is valid, and on `ms derive`.
- **`--passphrase @env:VAR` is literal on `silent-payment` and `ms derive`.**
- So a generic "secret → `-`" rule would produce wrong wallets. The design requires a committed per-input channel table of measured-OK cells only. A missing cell refuses the run; it never falls back to argv.

**Other findings:**

- `ms` has no `@env:` at all.
- `import-wallet --ms1` and `--slot` accept only `@env:`.
- **0.104.0 does not refuse** argv secrets on `import-wallet --ms1`, `seed-xor combine --share`, `slip39 combine --share` or `ms-shares combine --share`. They run with a warning.
- stdin strips exactly one trailing `\r?\n`, and `@env:` is verbatim.
- A `/dev/fd/3` pipe works as `ms --in`, so on Unix a second secret for ms never has to touch disk.

**The design proposes:**

- A `RunPlan`: argv with no secret bytes, plus stdin, env and fd bindings.
- **Assignment:** env for every source that supports it, one stdin, then pipe fd, else refuse.
- Env scrubbing of inherited `MNEMONIC_GUI_S*` variables.
- Deleting the GUI-managed `--allow-argv-secret` admission.
- A per-shape multi-secret table. `ms derive --phrase|--hex` with `--passphrase` has no second channel and is refused.
- A temp file on Windows only: a private directory, removed on child exit, swept at startup. The runner is synchronous, so "cancel" means the GUI was killed.
- What Preview, confirm and Copy show. Copy gets comment lines, so a pasted `-` does not look like a hang.
- The refusal list.
- **Testing:** a pure plan property test with distinct sentinels per source; real-binary equivalence **with the dependence leg**; swap tests (measured: swapping two env-bound multisig slots changes the output); a table↔measurement drift gate; named mutation checks.

**Unmeasured, and refused by the design until measured:**

- all `verify-bundle` rows (my fixture's output did not depend on the secret);
- the bundle `ms1`/`xprv`/`wif` slot subkeys;
- `addresses --from ms1=`/`seedqr=`;
- `import-wallet --decrypt-password`.

### Part B: the five forms

There is one table per form, built from each binary's `gui-schema`, `--help`, and runs of the binary for every conditional claim. **All five fit the existing `FlagKind`s; no new widget kind is needed.** What each needs:

- **md forms:** conditional functions. `md descriptor` is the largest, with three input modes.
- **`ms hashlock`:** a secret `--hashlock-phrase`, a secret `--hex`, and a secret ms1 positional. Its only private channel is `--hashlock-phrase-stdin`. The phrase is byte-verbatim (measured: `"  pad  "` hashes differently from `"pad"`, and the same over stdin as on argv), so the widget must not trim.
- **`--kind`:** gets a no-default sentinel, so the GUI never silently materialises `sha256` (F-553).
- **md fields B2–B4:** a pasted xprv descriptor should be masked and redacted. md refuses it, but only after the GUI has shown and persisted it.

Four open questions (Q1–Q4) are listed for the reviewer.

## Items for the controller

1. **Toolkit follow-ups to file** (Rust-primary, `mnemonic-toolkit/design/FOLLOWUPS.md`):
   - `--passphrase -` and `--passphrase @env:` taken as literal values (exit 0, a different wallet). The same applies to ms `derive`.
   - The four argv-secret non-refusals listed above.

   Neither is filed.
2. Close the engrave F-685 entry. The GUI FOLLOWUPS entry for `restore-from-secret-node-unmasked-and-persisted` is marked RESOLVED-on-branch in `e3a55a4`.
3. The design is unreviewed. Its load-bearing claims are machine-reproducible via `design/measurements/secret-channels/` with `BIN_DIR` set to the four release binaries.

The scratch directory `/scratch/code/shibboleth/gui-followups-scratch/` held the binaries, logs and throwaway crates, and was removed with `find … -delete`.

**CI on the final tip `ec31aed`:** build run 36139704304, every job success (release skipped); schema-mirror 36139704339 success. The `ci/gui-followups` ref was deleted afterwards; `origin/gui-followups` remains.
