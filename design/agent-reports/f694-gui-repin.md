# F-694 part 1 — mnemonic-gui re-pin to toolkit 0.105.1 / ms 0.20.1 (report)

Agent: Opus 5.5, 2026-09-25. Brief: `design/briefs/f694-gui-repin.md`.
Worktree `/scratch/code/shibboleth/gui-worktrees/repin063`, branch `repin-063` off
origin/master `3ca1d60`; pushed as `repin-063` and `ci/repin-063` (CI only builds `ci/**`).

## Verdict: STOPPED at the brief's stop condition — the bump is NOT data-only

The pin move and the cache re-derivation are done, and every data gate is green:
`regen_check.py --plans` GREEN, `check_design_tables.py` GREEN. But **the re-derived
data turns three Rust tests red and makes one Python mutation survive**. Fixing either
needs test-code changes, which is the brief's stop condition. Nothing is worked around.
**v0.63.0 was NOT prepared** (no version bump, no CHANGELOG entry), because the gates it
depends on are red. Nothing merged, nothing tagged.

## Commits on `repin-063`

- `45e6153` pin: toolkit v0.104.0 -> v0.105.1, ms v0.19.1 -> v0.20.1.
- `245a43d` measurements: re-derive the secret-channel caches (+ DESIGN generated blocks,
  CI installer SHA).
- `f209244` tutorial snapshots: re-baseline 50 PNGs. Every diff has the same bounding box,
  (299,73)-(339,92): the action-bar label `Pinned: mnemonic 0.104.0` -> `0.105.1`. I
  inspected the crops; nothing else moved. Locally the non-ignored suite skipped these
  (`GUI_TUTORIAL_SNAPSHOTS=1` gates them), and CI caught them.

## Step 2 — pins and flag surface (decisions)

- `pinned-upstream.toml` (mnemonic, ms tags + provenance comment), `Cargo.toml`'s toolkit
  tag (+ `Cargo.lock`: only the toolkit git rev moves), README install block,
  `pinned_version` in `src/schema/{mnemonic,ms}.rs`. md 0.20.3 / mk 0.13.0 are unchanged.
- The toolkit's `[patch.crates-io]` miniscript rev is unchanged at v0.105.1, so the GUI's
  copied patch stays. The `secret_taxonomy`/`secrets` API the GUI imports is unchanged
  between the tags (`git diff --stat` empty on lib.rs and secret_taxonomy.rs).
- **Flag surface.** I diffed `gui-schema` from the old release binaries (downloaded; their
  sha256 match the old `measured_with.json`) against the new ones:
  - mnemonic: byte-identical.
  - ms: `--separator` on encode, split and hashlock is now `dropdown ["space"]`
    (was `text`, null). The mirror was already `Dropdown(["space"])`, so only its comments
    change. The choices drift gate now covers it: adding `"hyphen"` to `SEPARATORS` turns
    `md_ms_mk_choices_and_defaults_match_pinned_gui_schema` red on all three subcommands.
    Reverted.
- **CI decision.** `schema-mirror.yml`'s `INSTALLER_SHA` moved from `c39ea361` (which pins
  0.104.0/0.19.1) to toolkit `1402d547471bcf69adc7ba291012aeca0a5f6ee4`. That commit pins
  0.105.1/0.20.1 and is byte-identical to the install.sh I used. Without this change,
  T4's identity check in CI would compare old binaries against the new cache.

## Step 3 — the channel-table diff

The §A1 pipeline was re-run in order against the release binaries:
mnemonic `e6925454…`, ms `f594b26d…`, md `c5b9a26e…`, mk `f0785f5e…`.

- **All 17 former WRONG cells are OK.** They were 15 rows: 13 `--passphrase -`, and 2 rows
  with both `-` and `@env:` wrong (`silent-payment`, `ms derive`), plus
  `--bip38-passphrase -` and `verify-bundle --ms1 -`. `table.md` now has 0 WRONG.
  Also: the `-`/`@env:` cells of `electrum-decrypt` and `import-wallet --decrypt-password`
  went from fails-closed to OK.
- `channel_table.json`: 17 of 85 inputs change.
  - `cli_env_rule` went from verbatim 53 / None 32 to verbatim 41 / None 28 /
    **strip-one-trailing-newline 16**. The 16 are the 13 `--passphrase` inputs,
    `--bip38-passphrase`, and both `--decrypt-password`, so the operator's extension of
    F-687 is measured.
  - EnvRef on those inputs now has terminator `None` (strip-one; this is modelled by
    `cli_env_rule`, and the CR/LF refusal makes it harmless).
  - `ms derive --passphrase` `argv_eq_exact` goes False -> **True**.
- `reinterpret.json`: ms re-reads `-` **and `@env:`** (was `-`). mnemonic is unchanged.
- §A5 plans change on shapes that now have a CLI `@env:`/`-` for the passphrase. For
  example, `silent-payment secret+passphrase` Linux: `--secret` stdin, `--passphrase` env
  (was secret over a pipe fd, passphrase over stdin; Windows no longer refuses
  fd-not-on-platform).
- `run_plans.py`: `FAILURES: none`. `test_plan.py`: 0 failures. `c1_evidence.sh`,
  `combos.sh`, `refusals.py`, `copy_evidence.py` and `probe_missing.py` were re-run.
  `missing_sources.md` and `copy_evidence.md` are unchanged. `demo_t4_coverage.sh` output
  is byte-identical.
- DESIGN doc: I refreshed its generated blocks. I also corrected four prose lines that
  quoted 0.104.0/0.19.1 numbers: the "Measured against" line, §A1's re-read line, the
  "17 WRONG cells" paragraph and §A3a's re-read line.
- **Not re-run:** `demo_ni5.sh` and `demo_r4_wrappers.sh`. They need `F687_BIN_DIR` (the
  F-687 *master* builds `9da32e2f`/`d1ab447`) and document the fold-4/5 argument against
  those builds. Their embedded outputs are historical evidence, not pinned data, and
  `check_design_tables.py` still finds them verbatim.
- Stale label (not fixed): the `c1_evidence.sh` line "= the literal passphrase '-' on
  argv" now prints `73c5da0a` (no passphrase). Argv `-` is now stdin, and the script
  feeds that run no stdin. The label, not the data, is out of date.

## What needs code — the stop

1. **`mutations.py`: 19/20. `Nm13: leading-dash refusal off` SURVIVES.** The Nm13 leg in
   `test_plan.py` only walks `shapes.SHAPES`. It reached the `value-starts-with-dash`
   refusal only through `ms derive --passphrase`, whose `argv_eq_exact` was False. F-687
   fold 1 made it True, as §A3b predicted, and no other shape input is False now. The one
   remaining False input, `ms hashlock --hashlock-phrase`, is single-source and in no
   shape. The refusal is still reachable in the product (hashlock phrase `-lead` on
   macOS/Windows), but the leg is now vacuous. **Design defect:** a pure test leg drew its
   reach from measured data, so a bump can silently empty it. A fix could walk every
   value-form table key, or use a synthetic `argv_eq_exact: False` row.
2. **`secret_channels_t6::t6_every_shape_through_the_real_form_plans_as_the_pure_planner`**
   fails on `silent-payment secret+passphrase linux`.
   - The pure planner gives passphrase EnvRef + secret StdinToggle. The form gives
     passphrase StdinToggle + secret **FileFlag (pipe fd)**.
   - Cause: `shapes.py` lists the sources `[secret, passphrase]`, but the form's argv order
     is `[passphrase, secret]`. §A4.3's rule ("first source in **argv order** with a
     `--X-stdin` toggle takes stdin") is order-dependent.
   - Before the bump the passphrase had only one channel, so order could not matter.
   - Permuting sources in `plan.py` shows 4 shapes are now order-sensitive: silent-payment,
     and xpub-search path-of-xpub / passphrase-of-xpub / account-of-descriptor
     phrase+passphrase. The M5 permutation leg checks only plan-vs-refuse, so it did not
     flag them.
   - The GUI's plan follows the rule as written and delivers exact bytes. But on
     silent-payment it spends a Linux-only pipe fd where an env channel was free.
   - Decision needed: fix `shapes.py` order to argv order (test data), or make the rule
     order-independent (planner code).
3. **`t6_copy_env_provenance_spells_the_users_own_variable_or_printf`** and
   **`t6_shell_leg_printed_recipes_equal_argv_exact_in_bash_zsh_fish`** hard-code the
   pre-bump plan: `--passphrase` over stdin, a `printf … |` recipe, and
   `read -rs MNEMONIC_GUI_S1`. The new data lets `--passphrase` use the user's own
   `@env:MY_PW` / `@env:PHR`. That is the better outcome, but the tests assert the old
   one. These are expected answers kept by hand, the class §A0 item 3 set out to remove.
   They need rewriting.
4. As a consequence, the Rust mutation harness (`scripts/secret-channels-mutations.py`)
   cannot run meaningfully. Its no-op CONTROL is "killed" by the three red t6 tests
   (`control DID NOT SURVIVE`), so every kill would be false until item 2/3 are fixed.

## Step 4 — the operator's CLI rulings and the GUI

No change needed; this was verified by measurement and by reading the code, not by a new
test.

- The runner spawns with stdin `Stdio::null()` or `Stdio::piped()` (`src/runner.rs:194-199`),
  never a TTY. So the terminal prompt and the paste drain (both TTY-only) cannot trigger.
- Empty values: a typed empty secret widget is omitted from argv (`invocation.rs:398/457`),
  and an empty `@env:` target is refused `C1-env-empty` before spawn. So the GUI never
  sends an empty value.
- If one did arrive, the CLI prints one line on stderr and exits 0, and the GUI shows
  stderr verbatim. Measured on 0.105.1: `warning: --passphrase from stdin is empty;
  proceeding with the EMPTY passphrase` for both `-` with null stdin and empty piped
  `--passphrase-stdin`, and `…from environment variable E is empty…` for `@env:`.
  `hunter2\r\n` on stdin gives `ca2c62d2`, which is correct.

## Gates (against the pinned release binaries)

| gate | result |
|---|---|
| `cargo nextest run --locked --workspace` (no `--release`) | **770/773; 3 FAIL** (the t6 tests above); log `gui-repin-scratch/logs/nextest1.log` |
| form snapshots / T10 (in the suite) | pass; no form PNG changed |
| tutorial harness (`GUI_TUTORIAL_SNAPSHOTS=1 … --include-ignored`) | 50 PNGs re-baselined for the version label only (`f209244`); then 12/12 pass |
| schema_mirror 21/21, schema_mirror_defaults_drift 3/3 | pass |
| clippy `--all-targets -D warnings` / `--no-default-features -D warnings` | pass / pass |
| MSRV 1.88.0 `cargo check --locked` | pass |
| `regen_check.py --plans` | **GREEN** |
| `check_design_tables.py` | GREEN (stale blocks: none; test_plan 0 failures) |
| `mutations.py` (Python T5) | **19/20, Nm13 survives** |
| Rust mutation harness | **not meaningful**: control killed by the red baseline |
| CI @ `245a43d` | schema-mirror run 36201824665: **failure**, only in `cargo-test-full-suite` (the 3 t6 tests). Its secret-channels-real-binary-tests (T2/T3'/T7), os-gate (T10), regen-check and design-gate steps all passed. build run 36201824669: tutorial-snapshots **failure** (the version label, since fixed); everything else success |
| CI @ `f209244` | build run 36202731848: **all jobs success**. schema-mirror run 36202731778: **failure**, only `cargo-test-full-suite` (the same 3 t6 tests) |

`cargo fmt --check` shows diffs in files this branch does not touch (`src/app_window.rs`,
among others). They are pre-existing on master and are not a brief gate.

## Cleanup

Scratch lives at `/scratch/code/shibboleth/gui-repin-scratch/`: binaries, target dirs, logs
and the `before/` snapshot of the old caches. I left it in place for the fix cycle. Delete
it with `find /scratch/code/shibboleth/gui-repin-scratch -delete`. The worktree and the
two remote branches remain.
