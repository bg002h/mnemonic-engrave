# F-679 implementer report — mnemonic-gui v0.62.0 pinned to today's CLIs

**Outcome:** branch `f679-pins` (pushed, tip `123f09a`, 6 commits on `origin/master` `e5e746c`) prepares mnemonic-gui **v0.62.0**, pinned to and exercised against the release binaries of mnemonic 0.104.0 / md 0.20.3 / ms 0.19.0 / mk 0.13.0. Full local suite and CI are green. The x86_64-linux-gnu glibc floor falls from **GLIBC_2.39 to GLIBC_2.18**, measured on the CI build, and a new CI gate now fails any build above 2.34. Nothing is tagged, published or merged.

Worktree: `/scratch/code/shibboleth/gui-worktrees/f679`.

## Commits (`f679-pins`)

| SHA | What |
|---|---|
| `ca4c191` | Pins, mirror lockstep, GUI-managed `--allow-argv-secret`, conditionals, tests, 17 form PNGs |
| `fa93fc3` | Tutorial corpus re-driven against toolkit v0.104.0; J4 step 14 renamed `tut-j4-14-depth2-export` |
| `65a4278` | x86_64-linux-gnu release row moved to `cross` (glibc floor) |
| `0627691` | FOLLOWUPS: 3 new entries |
| `d651d4d` | Release prep: version 0.62.0, CHANGELOG, README status and self-tag. **Not tagged.** |
| `123f09a` | CI `glibc-floor` gate on the linux-gnu rows |

## Pin changes

- `Cargo.toml`: the toolkit dependency tag moves from `mnemonic-toolkit-v0.97.0` to **`v0.104.0`** (the load-bearing pin), and the toolkit's own `[patch.crates-io] miniscript = { git = rust-miniscript, rev = ff4732e5… }` is added. Without that patch, md-codec 0.47.0 (pulled in by the toolkit) **fails to compile**: E0599 on `derive_at_index`, `into_definite` and `SortedMultiA`. A `[patch]` only applies at the build root, so the toolkit's patch does not reach the GUI. Drop the GUI's copy in lockstep with the toolkit's.
- `pinned-upstream.toml`: toolkit v0.104.0, md `v0.11.0→v0.20.3`, ms `v0.16.0→v0.19.0`, mk v0.13.0 (unchanged).
- The schema `pinned_version` strings now read `mnemonic 0.104.0`, `md 0.20.3`, `ms 0.19.0` and `mk 0.13.0`. They were `md 0.11.0`, `ms 0.13.0` and `mk 0.11.0`, all stale.
- README: the four CLI install lines, the self-install tag `mnemonic-gui-v0.62.0` and the Status line.

The binaries came from `install.sh --no-gui --no-man --root f679-scratch/rel`. `--version` reported 0.104.0 / 0.20.3 / 0.19.0 / 0.13.0, and every test ran with `MNEMONIC_BIN`, `MD_BIN`, `MS_BIN` and `MK_BIN` pointing at those binaries.

## Per-surface decisions

The baseline at the old pins, run against the release binaries, had 12 failing tests. A full diff of the mirror against `gui-schema` found the following.

### Behaviour change that needed a real decision: `--allow-argv-secret`

- **Surface:** a new global flag on all 32 `mnemonic` subcommands and on 9 ms verbs. The 8 of those that the GUI mirrors are covered here; the ninth is `hashlock`, which the GUI does not surface.
- **Why it matters:** both CLIs now **refuse secret material on argv** (mnemonic exits 2, ms exits 1). Every secret-bearing GUI run failed: bundle with a phrase slot, `ms decode` of an ms1, the passphrase flows, and `restore --from ms1=`.
- **Decision: deliberately hide it and have the GUI manage it.**
  - It is mirrored in the schema, which keeps the parity gate set-equal.
  - It is never rendered (`is_gui_managed_flag`, applied in app_window, render_emit and the test harness).
  - It is never emitted from form state.
  - The **Run** path (`form::invocation::admit_argv_secret_for_run` / `assemble_argv_for_run`) inserts it straight after the subcommand, and only when the argv carries a secret-masked token or a secret `<node>=<value>` token. The second case exists because `restore --from ms1=…` is a plain Text flag that the mask cannot see.
  - The **Copy command** path never carries the flag. A pasted command lands in shell history, and there the CLI's refusal is correct.
- **Why this follows the GUI's existing secret rules:**
  - The GUI has always passed secrets on argv. The spawn goes through `execve` with no shell, so no shell history.
  - Every such run already goes through the run-confirm modal, and the modal now shows the flag. The tutorial screenshot `tut-j1-01-bundle-single-sig-modal.png` shows this end to end in the real app window.
  - The remaining exposure is `/proc` cmdline to the same UID and root, which is unchanged from before.
- **Better end state, filed:** route secrets over stdin, `--in` or `@env:`. See `argv-secret-via-private-channels`.

### Flags and values exposed (added to the mirror as widgets)

| CLI | Surface | Decision / why |
|---|---|---|
| mnemonic | `export-wallet --allow` (repeatable, 5 rules) | Expose. The same vocabulary as build-descriptor. The help text says only `sigless-branch` is enforced, and that it permits emission only. |
| mnemonic | `export-wallet --count` (default 20) + `--format bitcoin-core-addresses` | Expose. This is the only Core route for sigless wallets. `EXPORT_FORMATS` is shared with `restore --format`, and upstream lists the value on both. |
| mnemonic | `restore --recalibrate-threads` | Expose as a Boolean. Note that it writes `~/.mnemonic/mt.conf`. |
| mnemonic | `--group-size` default 5→0 on bundle, convert and ms-shares split/combine | Mirror the default. The gate caught it. |
| mnemonic, md, ms | `--separator` hyphen/comma **retired** | Drop them from the dropdown (`space` only). The drift gate **cannot see this** because all four CLIs report `text`. Measured refusals: mnemonic exit 64, md exit 2, ms exit 64. **mk 0.13.0 still accepts all three**, so mk is unchanged. A new real-CLI test checks every offered separator against every CLI, and was mutation-proven. |
| md | `--in` on inspect/encode/decode/verify/bytecode/repair | Expose as Path. The md1 positionals are no longer marked required, matching upstream. |
| md | `encode --out` | Expose as Path. The file is created 0600. |
| md | `--experimental` on encode/verify/address | Expose as Boolean. The help states the bearer-access risk of a keyless path. |
| md | `--path` on verify/address | Expose as Text. |
| md | `address --from-mk1` (repeating), `--from-mk1-file`, `--seat` (repeating) | Expose. This is watch-only material. New conditional: `--from-mk1` and `--template` exclude each other. |
| md | `encode --in` | Counts as the template input in the `md_encode` conditional. |
| ms | `--in` on all 8 material verbs | Expose as Path. **This is a private channel**, so a GUI user can keep the secret off argv entirely. It joins `ms encode`'s phrase/hex one-of in the conditional, and `ms repair --ms1` is no longer marked required. |
| ms | `--out` on encode/repair/split | Expose as Path. The CLI writes it 0600. |
| ms | `derive --template` + `bip48-p2tr`, `bg002h-tr`, `bg002h-wsh` | Expose. The choices gate caught it. |
| md | `decode --json` schema tag `md-cli/1→md-cli/2` | No GUI change needed because the GUI parses no md JSON. The i4 test pin was updated (descriptor-mnemonic `f29ecb93`). |

### Deliberately not surfaced (filed as `md-ms-new-subcommands-unsurfaced`)

md `compose`, `shape-key`, `descriptor` and `decompose`, and ms `hashlock`. The mirror is a declared subset, and each of these is a new form; `hashlock` also takes a secret phrase. The parity gate walks only mirrored subcommands, so they are invisible to it.

### Secret-handling per new item

- Every new Path flag (`--in`, `--out`, `--from-mk1-file`) is `secret: false`. That matches the GUI's `--secret-file` precedent: the path is not the secret.
- No new Text flag carries secret material: `--from-mk1` is xpub cards, `--seat` is a chunk-set id, `--path` is a derivation path.
- `--allow-argv-secret` is covered above.
- **Pre-existing gap found and measured, not fixed:** `restore --from ms1=<card>` survives `redact_for_persistence`, so it reaches the autosave file. It also gets no confirm modal and is unmasked in the Preview. This is filed as `restore-from-secret-node-unmasked-and-persisted`. Under the 2026-08-27 ruling it is logged and does not gate.

## Snapshots — every change explained, none silently regenerated

- **Forms (`GUI_SNAPSHOTS=1`):** 17 PNGs failed. They are exactly the 17 forms whose rows changed: md × 7, ms × 8, export-wallet and restore. The 15 mnemonic forms that only gained the hidden flag did not change, which confirms the flag is hidden. Only those 17 were regenerated (GL llvmpipe). CI's lavapipe `snapshots` job passed at 0.6.
- **Tutorial (`GUI_TUTORIAL_SNAPSHOTS=1`, release mnemonic):**
  - **J4 step 14 used to teach a depth-2 taptree refusal. Toolkit 0.104.0 exports it at exit 0**, because the pinned rust-miniscript carries #953. The step is now a `run_step` renamed `tut-j4-14-depth2-export`. The rename is deliberate: the manual's include of the old stem fails loudly rather than teaching a refusal that no longer happens.
  - All other transcript diffs come from two upstream changes: (a) bundle cards are unbroken because the group-size default is now 0; (b) restore descriptors now carry xpubs whose header matches the origin (`xpub661My…` depth 0 → `xpub6Bem…`, toolkit v0.102.0). Addresses are unchanged; the first recv address is byte-identical.
  - PNG deltas: the pinned-version label, the new export-wallet rows, and the modal now showing the flag.
  - Census: 50/50 shots and 33/33 runs.

## Gates actually run

| Gate | Command | Result |
|---|---|---|
| Full suite | `cargo nextest run --locked --no-fail-fast`, `*_BIN` = release binaries | **702 passed, 6 skipped.** The 6 are 3 proptest finders, 2 tutorial harness tests (run separately below), and `manual_anchor_coverage` (needs the built manual). Baseline at the old pins was 674 passed / 12 failed. |
| New tests | `tests/f679_argv_secret_admission.rs` | 12 tests: 8 logic, 4 real-CLI (ms encode via the secret widget, ms `--in`/`--out` to a 0600 file, md decode `--in`, every offered separator accepted). Mutation-proven: removing the insert failed 5 tests; dropping the node-value classifier failed 2; re-adding `hyphen` to md failed the separator test. Plus 4 new conditional cells. |
| Schema-mirror gate | part of the suite (`schema_mirror`, `*_defaults_drift`, `xpub_search_schema_mirror`) against the release binaries | green |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings`; `--no-default-features -- -D warnings` | clean |
| MSRV | `rustup run 1.88.0 cargo check --locked` | clean |
| fmt | `cargo fmt --check` | `origin/master` **already has drift in 82 files**. The branch adds no new drift hunks, compared per file against a pristine `origin/master` worktree. One exception: `tests/tutorial/manifest.rs` counts 13 hunks against 12, because the step-14 edit follows that file's hand-compact house style. |
| Headless GUI | `gui_tutorial_snapshots --include-ignored` (drives the real `MnemonicGuiApp`: Run button, confirm modal, and spawns release `mnemonic`) | 12 passed. This is the end-to-end mnemonic flow. |
| Per-CLI e2e | `ui_harness_i4_realcli` (widget-driven → assembler → release binary): mnemonic decode-address, md decode, ms decode (through the admission), mk decode; plus the f679 real-CLI cells above | all pass |
| Forms snapshot | `GUI_SNAPSHOTS=1 WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1 cargo test --test gui_form_snapshots` | green |
| **CI** | pushed `f679-pins`, and the same SHA to `ci/f679-pins` (build/schema-mirror trigger on `ci/**`; no workflow_dispatch exists). Ref deleted afterwards. | Tip `123f09a`: **build run 36100903163** — every job passed (clippy, headless, msrv, snapshots, tutorial-snapshots, 7 target builds); `release` skipped, as it only runs on tags. **schema-mirror run 36100903176** — schema-mirror gate passed. Earlier tip `d651d4d`: build 36100214287 and schema-mirror 36100214288, all passed. |

## Glibc finding (brief item 5)

- **Cause:** the `x86_64-unknown-linux-gnu` row built natively on `ubuntu-latest` (24.04, glibc 2.39). glibc symbol versions are fixed by the build host at link time. Measured on the v0.61.0 asset with `objdump -T`, the maximum is **GLIBC_2.39** (`dlopen@2.34`, `stat64@2.33`, …). The aarch64-gnu row already used `cross` and measures GLIBC_2.18.
- **Fix:** a one-line matrix change (`cross: true` on the x86_64-gnu row), plus a `glibc-floor` step on both gnu rows. The step runs `readelf --dyn-syms`, prints the maximum GLIBC version, and fails above 2.34 (RHEL 9).
  - Checked locally on the v0.61.0 assets with the exact pipeline: aarch64 2.18 passes; x86_64 2.39 fails.
- **Proven on CI** (run 36100903163 logs): `glibc floor (x86_64-unknown-linux-gnu): GLIBC_2.18`, `glibc floor (aarch64-unknown-linux-gnu): GLIBC_2.18`. Debian 12, Ubuntu 22.04 and RHEL 9 are all covered.
- **Not verified:** a cross-built x86_64 binary actually *starting* on an old distro. It only links libc, libm, libgcc_s and ld-linux; winit/wgpu load their libraries with dlopen, the same as the aarch64 asset.

## Concerns for the controller (toolkit side and beyond)

1. **The GUI manual must change.** `mnemonic-toolkit/docs/manual-gui/tutorial/50-j4-taproot-twin.md` §"Depth-2 refusal" says the tree refuses with exit 2, and includes `tut-j4-14-depth2-refusal.*`. Those files no longer exist, so that manual build will fail until the section is rewritten around `tut-j4-14-depth2-export` (exit 0). Step 15's "the fix … is a depth-1 tree" is no longer a fix either. The manual also documents v0.57.0.
2. **The installer text is stale once it pins 0.62.0.** `install.sh --help` says "on Linux x86_64 the GUI needs glibc >= 2.39". With v0.62.0 that becomes ≥ 2.18, and the installer's source-build fallback for old-glibc hosts can be relaxed.
3. **Toolkit CHANGELOG record defect.** At tag `mnemonic-toolkit-v0.104.0`, the argv-secret refusal, the group-size and separator changes, and `export-wallet --allow`/`--count` all sit under **[Unreleased]**, above the `[0.104.0]` heading. The tagged code and the release binary do ship them.
4. **ms help text is stale.** `ms encode|split --help` still says `Separator: space|hyphen|comma` while ms 0.19.0 refuses hyphen.
5. **README install line for md** has no `--features cli-compiler`, which the installer uses. This was already the case; the schema-mirror CI also builds without it and passed, because `compile` is not mirrored.
6. **GUI CHANGELOG has no 0.60.0 or 0.61.0 entries**, although both are tagged. Not backfilled.
7. **README Status line** says `v0.62.0 (2026-09-24)` as prepared. Adjust the date if it is tagged later.

## Scratch

`/scratch/code/shibboleth/f679-scratch/` (release binaries, logs, a pristine `origin/master` worktree used for the fmt baseline) was removed with `find … -delete` after this report was written, and its git worktree registration was pruned.
