# F-679 fold 1 — mnemonic-gui `f679-pins` @ `69e3adb`

**Outcome:** one commit, `69e3adb`, on top of `123f09a`. It fixes I2, M1, M2 and N1 from `f679-review.md`, and is pushed to `f679-pins`. CI is green: build run **36103784622**, all 12 jobs passed and `release` was skipped as it only runs on tags; schema-mirror run **36103784533** passed. Nothing is tagged or merged. I1 (ms verify) was left alone, as instructed; it is waiting on the ms 0.19.1 pin bump. M3 was also left, because it is already filed.

## I2 — `--` before positionals, always

**Decision: always emit `--`, not only after a multi-value option.**

- **Where:** `assemble_argv_with_secret_mask` (`src/form/invocation.rs`, const `END_OF_OPTIONS`). When at least one positional token is emitted, `--` goes in front of the positionals. When there are no positionals, nothing is added.
- **What was measured to justify it:** I ran every mirrored subcommand that takes positionals against the release binaries, once with `--` and once without. That is 20 subcommands:
  - md: inspect, encode, decode, verify, bytecode, compile, address, repair
  - ms: inspect, decode, verify, derive, combine
  - mk: decode, inspect, verify, repair, address, derive
  - mnemonic: decode-address
- **Result:** the exit code was the same every time and the combined stdout+stderr was **byte-identical**.
  - The mk address/derive probes first used multisig cards and exited 64 in both forms. I re-ran them with single-sig cards and they exited 0 in both forms.
  - ms's argv guard still refuses a secret positional placed after `--` (the Copy path).
  - `--allow-argv-secret` placed before `--` is honoured (the Run path, `ms decode`, exit 0).
- **Correction to the fold commit message:** it says "all 21 mirrored subcommands"; the correct count is **20**. I probed md address twice, and counted it twice.
- **Why unconditional:** a rule keyed on "a multi-value option precedes the positionals" needs one more hand-maintained list of which options take several values, and that list can drift. `--` also protects a positional value that starts with `-`.
- **Consistency of what the user sees and what runs:** Copy, Preview, the confirm modal and Run all derive from this one argv. Run only adds `--allow-argv-secret` after the subcommand, which comes before `--`. The new test asserts copy == run for md address.
- **Test:** `real_md_address_from_mk1_with_the_policy_positional` runs the GUI-assembled argv against release md 0.20.3. It uses two `--from-mk1` rows plus the keyless md1 `md1yq802gggqpsqwgtua24e7ssf3` and expects exit 0 with address `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`. A structural cell, `positionals_follow_an_end_of_options_marker`, checks that the marker appears exactly once, sits immediately before the first positional, is absent when there are no positionals, and that a secret positional keeps its mask bit.
- **Mutation:** removing the `--` push makes both tests fail. The real-CLI test fails with md's exact error: exit 1, "seating refused: `md1yq802…` is an md1 policy-card string, not an mk1 key card".

## M1 — ms input-source exclusivity

- **What was added:** new conditionals in `src/form/conditional.rs`:
  - `ms_ms1_or_in` for inspect, decode and verify
  - `ms_derive`
  - `ms_repair`
  - `ms_combine`
- **How they work:** the first source that has a value wins, and every later **flag** source is Disabled, which also suppresses it from argv. Positionals cannot be disabled, so they come first in the order and a filled positional always wins.
- **Input orders:**
  - derive: `[MS1]` → `--in` → `--phrase` → `--hex`. clap refuses `--in` together with `--phrase`, with `--hex`, and with the positional; I measured each.
  - repair: `--ms1` → `--in`. When neither has a value, both are marked Required.
  - combine: `shares` → `--in`. When neither has a value, both are marked Required.
- **Tests:** 4 new cells in `tests/conditional_visibility.rs`.
  - `ui_harness_i2_conditional`: the census of subcommands with conditionals goes from 17 to 23, with ms going from 1 to 7.
  - `CASES` gains a seed for each new subcommand, so the render-versus-conditional check now covers them.

## M2 — md address Required marker

- When `--from-mk1` or `--from-mk1-file` is set without a policy phrase, `md_address` now marks `positional:phrases` Required.
- Positionals now honour a conditional Required on the reserved key `positional:<name>`. The check is `render_emit::positional_required`, shared by app_window's positional loop, the `.gui` emit, and the harness mirror in `tests/ui_harness/mod.rs`.
- `gui-render --form ms combine` now shows `shares … (required, secret)` and `--in (required)`.
- New cell: `cell_fold1_md_address_mk1_only_marks_phrases_required`.

## Nits

- **N1 — fixed (it was cheap).** A secret-masked token no longer earns `--allow-argv-secret` when it only *names* a private channel. That covers:
  - the whole token being `-`, `@env:…` or empty;
  - a slot token `@N.<subkey>=<sentinel>`;
  - a composite `<argv-secret-node>=<sentinel>`.

  A secret value that merely contains `=` is not split, so it still gets the opt-in. The test `private_channel_sentinels_in_secret_fields_need_no_opt_in` covers `phrase=@env:SEED` (no flag), `phrase=-` (no flag), a real phrase (flag), and a passphrase `a=-` (flag).
- **N2 — record only; no code change.** Both counts are true:
  - My implementation report said "failed 2" because it counted two test binaries: the f679 binary plus `bundle_restore_independent_oracle`, whose restore leg also depends on the classifier.
  - The reviewer's "1 of 12" counted the f679 binary alone.

## Gates run on `69e3adb`

| Gate | Result |
|---|---|
| `cargo nextest run --locked` against release mnemonic 0.104.0, md 0.20.3, ms 0.19.0, mk 0.13.0 (re-installed with `install.sh --no-gui --no-man`) | **709 passed, 6 skipped**. Same 6 as before: 3 proptest finders, 2 tutorial harness tests run separately, `manual_anchor_coverage`. |
| clippy `--all-targets -D warnings`; `--no-default-features -D warnings` | clean |
| MSRV `rustup run 1.88.0 cargo check --locked` | clean |
| `GUI_SNAPSHOTS=1` form snapshots (GL llvmpipe) | green. **No PNG changed.** I deleted and regenerated the 7 affected forms (ms repair/combine/inspect/decode/verify/derive, md address) and each came out byte-identical to the committed one. Their blank fixture triggers no Disabled state, and apparently no visible marker change at that size. |
| Tutorial harness `--include-ignored` against release mnemonic | 12 passed |
| rustfmt on touched files | no new hunks, compared per file against `HEAD` |
| CI | build 36103784622, all jobs passed; schema-mirror 36103784533, passed. The `ci/f679-pins` trigger ref was deleted afterwards. |

## Concerns

- **I1 is still open.** `ms verify --phrase <P> <ms1>` fails at ms 0.19.0. Please send the 0.19.1 pin bump: `pinned-upstream.toml` [ms].tag, `src/schema/ms.rs` pinned_version, and the README install line. I will add a real-CLI verify-with-phrase cell with it.
- **CHANGELOG:** the 0.62.0 entry gained two bullets for this fold.
- **Scratch:** `/scratch/code/shibboleth/f679-scratch/` was removed with `find … -delete`.
