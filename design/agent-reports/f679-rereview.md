# F-679 re-review — mnemonic-gui `f679-pins` folds 1 (`69e3adb`) and 2 (`5d9709c`)

**Question:** did folds 1 and 2 fix each finding of `f679-review.md` (0C/2I/3M/2N),
and did they introduce a new defect? **Answer: yes, and no new defect found.**

## Binaries used

- `mnemonic` 0.104.0, `md` 0.20.3, `mk` 0.13.0, `ms` 0.19.0 — installed via
  `install.sh --no-gui --no-man --root <scratch>` (the installer still pins ms
  0.19.0, confirmed, not used for the fix checks).
- `ms` 0.19.1 — `gh release download ms-cli-v0.19.1` asset
  `ms-0.19.1-x86_64-linux-musl.tar.gz`; sha256
  `92a45a92d8e8995cacff838c38f8930219b757f1752572eff4cfdbe839b2eb55` matches
  `SHA256SUMS.x86_64` exactly. `ms --version` → `ms 0.19.1`.

## Per-finding result

| ID | Status | Evidence |
|---|---|---|
| **I1** | **Fixed** | `real_ms_verify_phrase_and_positional_ms1` (fold 2's test): exit 0 against ms 0.19.1; re-run against ms 0.19.0 with `MS_BIN` swapped reproduces I1's exact error verbatim (`error: cannot read both ms1 and --phrase from stdin`, exit 1). |
| **I2 / "always `--`"** | **Fixed** | See below — 13-probe re-check, all real-CLI, all pass. |
| **M1** | **Fixed, survives defeat attempts** | See below. |
| **M2** | **Fixed** | `md_address` marks `positional:phrases` Required with `--from-mk1` alone, and stops once the positional is also filled — checked directly against the conditional function. |
| **N1** | **Fixed** | Repo's own `private_channel_sentinels_in_secret_fields_need_no_opt_in` passes; mutation-confirmed (below). |
| **N2** | Record-only, no code change (as fold 1 states) — not a gate item. |
| **M3** | Stays filed, per the 2026-08-27 secret-handling severity ruling — not re-derived. |

## I2 / "always `--`" — 13-probe re-check (brief asked for ≥ 8, all four CLIs)

New probe file `tests/f679b_rereview_probe.rs` (untracked scratch test,
deleted after the round; worktree confirmed clean via `git status --porcelain`
before and after). Real-CLI cases, all pass:

1. `mnemonic decode-address` valid address (baseline).
2. `mnemonic decode-address` **positional starting with `-`** (`-not-an-address`):
   protected — CLI's own error ("not a valid Bitcoin address: base58 error"),
   not clap's usage error. A same-token negative control **without** `--`
   (bypassing the GUI assembler) gets `error: unexpected argument '-n' found` —
   the exact class of bug the marker prevents.
3. `md decode` with an **empty positional**: no `--` emitted (0 positionals),
   exit non-zero on empty stdin (`Stdio::null()`), no hang.
4. `md address --from-mk1` (I2's original repro), re-verified: exit 0, correct
   address `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`.
5. `ms decode <ms1>` **secret positional, WITHOUT `--allow-argv-secret`**
   (Copy path): refused (exit 1, ms's argv-secret guard message) — not silently run.
6. `ms decode <ms1>` **secret positional, WITH the flag** (Run path): exit 0;
   flag confirmed before `--`, `--` immediately before the ms1 token.
7. `ms verify --phrase <P> <ms1>` (Run, two secrets): flag at argv index 2
   (right after the subcommand), before `--`.
8. `mk decode <mk1-chunk-A> <mk1-chunk-B>` baseline: exit 0.
9. `mk decode` positional starting with `-` (`-notamk1`): protected, mk's own
   error ("invalid HRP: notamk"), not a clap usage error.
10–13. `Copy == Run-minus-flag` asserted programmatically for every case above
   (all derive from one `assemble_argv_with_secret_mask` call — confirmed
   structurally too: the fold's `app_window.rs` diff only touches the
   Required-marker block at the positional-render site; the
   Copy/Preview/confirm-modal/Run composition at `app_window.rs:895-1010`
   (`assemble_argv_with_secret_mask` → `preview`/`argv_windows`/`argv_posix`,
   confirm gated by `should_confirm_run`) is byte-for-byte untouched by
   either fold).

**Is `--allow-argv-secret` ever emitted after `--`?** No, in every case
checked (asserted `position(flag) < position(marker)` — holds because the
flag is inserted right after the subcommand token, which always precedes the
positionals block where `--` lives).

**Mutation:** removed the `if !positionals.is_empty() { argv.push(END_OF_OPTIONS...) }`
block. Turned exactly the expected 5 tests red — the two pre-existing repo
tests (`positionals_follow_an_end_of_options_marker`,
`real_md_address_from_mk1_with_the_policy_positional`, the latter failing with
md's *exact* original I2 error) plus 3 of the new probes. Restored; full suite
re-confirmed green afterward.

## M1 — defeat attempt: fill several sources, clear one, reorder

`ms_derive`'s order is `["positional:ms1", "--in", "--phrase", "--hex"]`
(precedence over a FIXED order, not fill sequence). Tried directly against
`ms_derive`/`assemble_argv`, plus one real-binary run:

- **A** all four filled → positional wins, both flag sources Disabled and
  suppressed from argv.
- **B** clear the positional → `--in` (next in order) wins, `--phrase`
  disabled.
- **C** also clear `--in` → `--phrase` wins, `--hex` disabled; ran for real
  (`ms derive --phrase <ABANDON>`, `--hex` confirmed absent from argv): exit 0.
- **D** reorder — same final state as C but with `--hex` inserted into the
  state's internal `Vec` before `--phrase`: winner unchanged (rules out
  insertion-order / iteration-order dependence, since the function keys off
  the fixed `order` array, not state's Vec order).
- **E** refill the positional after B/C: positional wins again — no
  hysteresis; the function is pure in `state`.

None of these produced two simultaneously-enabled sources or a stuck winner.

**Mutation:** changed `order.iter().skip(winner + 1)` to `skip(winner)` (also
disabling the winner itself). Turned red: my new
`ms1_derive_precedence_survives_clearing_and_reordering` +
`real_ms1_derive_precedence_phrase_wins_when_positional_and_in_absent`, **and**
the repo's own committed cells `cell_fold1_ms_derive_first_source_wins` and
`cell_fold1_ms_positional_disables_in` in `tests/conditional_visibility.rs`.
Restored; `src/form/conditional.rs` back to its committed state
(`git status --porcelain` empty).

## N1 — mutation

Changed `is_private_channel_value` to drop the `@env:` check. Turned red:
`private_channel_sentinels_in_secret_fields_need_no_opt_in`
(`@env:SEED` wrongly got the opt-in). Restored.

## Tutorial and snapshots

- `GUI_SNAPSHOTS=1 WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1 UPDATE_SNAPSHOTS=1
  cargo test --test gui_form_snapshots`: adapter confirmed
  `device_type: Cpu` (llvmpipe/GL), all 61 forms pass at the 0.6 threshold.
  Byte-for-byte `cmp` of every regenerated `.new.png` against its committed
  PNG: **0 diffs across all 61** — a superset of fold 1's claimed 7
  (ms repair/combine/inspect/decode/verify/derive, md address), all confirmed
  unchanged. `.new.png` artifacts deleted afterward (gitignored; worktree was
  never dirtied).
- Tutorial harness `GUI_TUTORIAL_SNAPSHOTS=1 cargo test --test
  gui_tutorial_snapshots -- --include-ignored`, release mnemonic 0.104.0:
  **12 passed**, matches both fold reports.

## Suite, clippy, MSRV, fmt

- `cargo nextest run --locked --no-fail-fast`, all four `*_BIN` set (MS_BIN =
  ms 0.19.1): **723 passed, 0 failed, 6 skipped** — same 6 skips as prior
  rounds (3 proptest finders, 2 tutorial-harness-run-separately,
  `manual_anchor_coverage`); 723 = fold 2's 710 + my 13 new probe tests.
- `cargo clippy --locked --all-targets -- -D warnings`: clean.
- `cargo clippy --locked --no-default-features -- -D warnings`: clean.
- `rustup run 1.88.0 cargo check --locked`: clean (MSRV 1.88.0).
- fmt on touched files: `rustfmt --check` at `123f09a` vs `5d9709c` per
  touched file gives the **same hunk count** in every file except
  `render_emit.rs` (9 → 8, i.e. fewer) — no new fmt violations introduced by
  either fold, consistent with `origin/master`'s pre-existing 82-file
  baseline (settled fact, not re-derived) and with both fold reports' "no new
  hunks" claim.

## New findings

**None.** No Critical, Important, Minor, or Nit found beyond what was already
filed (M3, N2).

## Cleanup

- Probe file `tests/f679b_rereview_probe.rs` deleted; `gui-worktrees/f679`
  confirmed clean (`git status --porcelain` empty) before finishing.
- `/scratch/code/shibboleth/review-f679b-scratch/` deleted with `find … -delete`.
- Nothing committed, pushed, tagged, or edited on the `f679-pins` branch.

## ready to tag: yes
