# mnemonic-gui phase 2: implementation report (secret channels + five forms)

**Outcome:** Parts A and B are both implemented on `gui-followups` (220c265 → `ee26ef1`, 9 commits, pushed), and CI is green on the final head: build `36174422248`, schema-mirror `36174422278`. Test counts: 768/768 plus the env-gated suites, and 29/29 mutations killed. There is one deviation from the frozen spec: `ms hashlock --separator` offers `space` only. There is also one inconsistency inside the spec (T9's newline legs vs fold 5). Nothing was tagged or merged.

Worktree: `/scratch/code/shibboleth/gui-worktrees/followups`, branch `gui-followups`.

## Commits

| sha | what |
|---|---|
| d35033a | Part A: planner, runner, Preview/Copy/confirm, retirements |
| 0210c49 | tutorial re-pin (deliberate, J1/J2) |
| 3722dc4 | T1, T2/T2b, T3′, T6, T7, T8, T10 in Rust |
| 849dbfe | CI: the Linux secret-channel job |
| d841305 | T5 mutation harness (`scripts/secret-channels-mutations.py`) |
| fd9f1bc | T1 parity fix for an older Unicode database in CI (see CI) |
| 58eb261 | Part B: the five forms |
| 32909c8 | Part B tests + T9 |
| ee26ef1 | T5 extended to Part B |

## Part A, per design section

- **§A0/§A1 data.** `src/form/channels/data.rs` pulls in `channel_table.json`, `channel_policy.json` and `reinterpret.json` with `include_str!`, so each file exists once, as the CI-diffed cache. The design says "generated from the JSON files" (§A4.2). This is that, with no generated `.rs` copies.
- **§A0.1 decisions.**
  - `lookalike.rs` implements the lookalike check: NFKC, drop Cc/Cf, strip, then casefold. The Cf table and the 297 casefold-vs-lower exceptions were generated from Python's Unicode 16.0 tables.
  - The CR/LF refusal is in the planner.
  - Both apply on every path and every OS.
- **§A3a C1.** `resolve()` mirrors `plan.resolve`:
  - `-` refuses.
  - `@env:VAR` is resolved from the GUI's own environment under the input's measured `cli_env_rule`.
  - The reserved, bad-name, unset, empty-after-rule and UNKNOWN cases each refuse.
  - The pass-through guard covers every non-source token.
  - Provenance ("typed" or `$VAR`) is carried per value.
- **§A4.1 sources.** The assembler records each source's site and channel-table key: secret Text, secret slot, argv-secret composite, node-named Text (node-only, "whatever `<v>` is"), and secret positional or group.
- **§A4.2–§A4.4 planner and runner.**
  - `plan_sources` mirrors `plan.py` branch for branch, including the interim path, Nm13 `--flag=VALUE`, and the fd-not-on-platform fallback.
  - `materialize` turns a plan into argv, env, stdin and fds.
  - `runner::run_plan` scrubs inherited `MNEMONIC_GUI_*`, sets the plan's env, and writes stdin. Pipes are `std::io::pipe` (O_CLOEXEC) with the write end closed before spawn, mapped with `command-fds`.
  - `form::channels::plan(schema, sub, state, user_env, os)` is the §A4.2 signature.
- **§A4.5.** The `*-stdin` toggles stay disabled. Tree mode's `--spec -` is a pre-bound stdin: a plan that also needs stdin refuses `two-stdin`.
- **§A6.** The interim path runs off `private_channels_on`, carrying the resolved bytes plus `--allow-argv-secret`. The confirm dialog stays. Its first sentence is "sends these secrets privately to" on the private path; the interim path keeps the old sentence.
- **§A7 Copy.** Generated from the private plan on every OS (`Policy::for_copy`). It never carries a secret or `--allow-argv-secret`.
  - It uses the §A7 recipes: typed EnvRef `IFS= read -rs` (with the fish form), `$VAR` → the user's own `@env:`, a stdin `printf '%s\r\n'` pipeline, `--in <FILE>`, and the StdinMulti recipes.
  - It is disabled for a refused plan (the refusal is the tooltip), for a multi-line typed EnvRef value, and on Windows for a stdin binding.
- **Preview.** Shows the planned argv plus one `<source> ← <channel> (provenance)` line per binding. A refused plan disables Run and names the refusal.
- **Retirements (§A3a).**
  - All 25 help strings are rewritten. They now also state that `-`/`@env…` values are refused, per the "documentation only, stated in the field help" line.
  - Removed: `admit_argv_secret_for_run`, `assemble_argv_for_run`, `is_private_channel_value`, `masked_token_is_private_channel`.
  - `text_value_is_secret_node_token` is split into the node-only `text_value_is_secret_source` plus C1.
  - Kittest Cell 8 is re-pinned to GUI-side resolution, with a new reserved-name cell.
  - The f679 private-channel legs are dropped. The admission cells moved to the interim path, and the real-CLI cells now run through the planner.
  - The phase-1a sentinel test is now a C1 test.
- **Tutorial re-pin (deliberate, 0210c49).**
  - The J1/J2 form, modal and run PNGs show the planner reference instead of `••••`, the binding lines, and the new dialog text.
  - The toolkit's "secret material on argv" warning is gone from 7 transcripts.
  - Three convert-fingerprint steps now expect no stderr; their transcripts are removed and the stems regenerated.
  - The harness's "must render ••••" check accepts the private-channel Preview; the no-plaintext guards are unchanged.

## Part B

- **Mirrored in the schema:**
  - md `compose`, `shape-key`, `descriptor`, `decompose`, each with a conditional fn per the tables.
  - ms `hashlock`, with the phrase, `--hex` and `<ms1>` hand-marked secret. The phrase goes byte-verbatim over `--hashlock-phrase-stdin`.
  - hashlock source exclusivity: the first filled wins, in the order plate → phrase → hex → in → random.
  - `--random` makes `--out` required; `--method` and `--emit-record` are phrase-only.
  - `--kind` starts at "(choose)", which emits nothing and disables Run until a kind is chosen. "all kinds — lookup only" omits the flag and shows the banner. There is also a `--json` notice.
- **B2–B4 `*`.** A pasted xprv in shape-key `--descriptor`, descriptor `--key` or the decompose positional is masked in the widget and every display, including the plan's Preview, and never persisted. The tree form's `is_xprv_like` is applied per token.
- **Small refinement.** `md descriptor --emit` greys the whole dropdown outside mode C instead of disabling only the `md1` option. Its only other choice is "(none)", and this also keeps a stale `md1` off argv. It avoids adding a `DisableOptions` producer that the I2 harness pins as absent.
- **Snapshots.** Five new form PNGs (md-compose, md-decompose, md-descriptor, md-shape-key, ms-hashlock). No existing form PNG changed. The census goes 61 → 66 in the suite and in CI.
- **Census updates (deliberate):**
  - conditional subcommands: 23 → 28
  - secret partition: 42 value-bearing / 25 toggles / 6 secret positionals
  - the frozen secret-positional literal
  - f679 declaring count: 40 → 41
- **`secret_sources.txt`** was re-derived from the mirror: +3 hashlock sources, all with table entries.
- **Not done here:** the five manual pages (§B6). The design makes them a toolkit-repo paired PR.

## Deviation and spec inconsistency

1. **`ms hashlock --separator` offers `space` only (deviation).** The §B5 table lists `space, hyphen, comma`, taken from ms's `--help`. The pinned ms 0.19.1 refuses both extra values: `error: invalid value 'hyphen' … separator "hyphen" is no longer offered: ms emits whitespace grouping only` (exit 64). I used ms.rs's existing `SEPARATORS` (`["space"]`) and extended `real_every_offered_separator_is_accepted` to hashlock and md descriptor. A hand mutation that offers hyphen turns that test red.
2. **T9 vs fold 5 (inconsistency, handled by the design's own rule).** §A9 T9 says "planned run == the oracle for `pad\n` and `pad\r\n`". Fold 5 (§A0.1, NI9) refuses any target ending in CR or LF on every path, and §A9 T3′ says "a refusal is always safe". T9 therefore asserts `value-ends-in-newline` for those two values, and runs and oracle-checks `"  pad  "` and `"pad"`, which differ as they should.

Other judgment calls:
- **T8 order.** T8 compares against `plans_pure.json` at the pure-planner level, in the prototype shapes' argv order. The GUI's flag order differs for bundle, verify-bundle and silent-payment; that renumbers only `MNEMONIC_GUI_S<i>`, not the channel kinds. T6 separately pins every shape through a real `FormState` against the pure planner (same channel and payload per source, env names following the GUI's own index).
- **Non-UTF-8 variables.** A non-UTF-8 environment variable reads as unset (`C1-env-unset`).

## Tests (Rust, `tests/secret_channels_*.rs`, `tests/part_b_forms.rs`)

Shapes, corpus and single-input cases are read from `shapes.py`, `corpus.py` and `cases3.py` through python3, so there is one copy of each. A missing python3 or `MNEMONIC_BIN` is a failure, never a skip.

- **T1** (pure): the plan property with distinct sentinels.
  - 1260 C1 legs, 3717 NI8 legs.
  - Both rules and the per-input rule; UNKNOWN; the guard; no-table-entry; the interim switch.
  - M5 permutations, lenient, the payload bound, NI3, NI9, Nm13, env-empty, NUL, the pin identity, the Copy gate.
  - The lookalike predicate against the Python reference over every code point.
- **T8** (pure): Rust == `plans_pure.json` (bindings and payload bytes) on all shapes × OS, and `describe()` == §A5.
- **T7:** the real runner against an argv-parsing echo helper. Checks per-binding bytes, a 30 s leak timeout, the env scrub, and the interim argv.
- **T2/T2b:** every table cell through the GUI's materialize + runner == the argv baseline, with dependence; the 21-variant corpus == argv-exact; ms combine group cells.
- **T3′:** the port of `run_plans.py`. Legs: baseline, swap, NI1, interim, NC1, Nm13. Its counts match the Python run (1451 refused endings).
- **T6:** every shape through a real form; the §A7 spellings and disabled states; a **shell leg** (the GUI's own Copy text run in bash, zsh and fish == argv-exact for 11 values, across the typed stdin, printf and `read` recipes); the window's Preview, provenance and confirm dialog; a refusal disables Run.
- **T9:** as above.
- **T10:** `os_gate.py` against the repo's workflows is a hard failure (red before 849dbfe, green after), and the gate is pinned against 14 decoys and 4 genuine fixtures.
- **T4:** `regen_check.py --plans` is GREEN locally (51 s) and in CI; `check_design_tables.py` and `test_plan.py` report 0 failures.

**T5 mutations** (`design/measurements/secret-channels/rust_mutations.out`): 29/29 killed by assertion, each with the mutated line proven to have run, and the no-op control survived. Covered:
- the planner's assignment (6)
- the lookalike predicate (5)
- CR/LF (3, including strip-one → strip-all)
- C1 (3)
- delivery (7): two secrets swapped, typed text sent instead of resolved bytes on each path, terminator dropped, stdin from the wrong binding, env scrub dropped, write end left open, the admission restored
- Part B (4)

## Gates

- Full nextest against the pinned release binaries (installed with `install.sh`; sha256 == `measured_with.json`), never `--release`: **768 passed**, 6 skipped. The skips are the env-gated suites, run separately below.
- clippy `-D warnings`, default and `--no-default-features`: clean.
- MSRV: `rustup run 1.88.0 cargo check --locked` is clean.
- Tutorial harness (`GUI_TUTORIAL_SNAPSHOTS=1 --include-ignored`): 12/12. Form snapshots (`GUI_SNAPSHOTS=1`): 2/2.
- Measurement scripts: `regen_check.py --plans` GREEN, `check_design_tables.py` no stale blocks, `test_plan.py` 0 failures.
- New files are rustfmt'd. The baseline isn't rustfmt-clean, so existing files were not reformatted.

**CI** (pushed to `ci/gui-followups`, which I deleted afterwards):

| run | head | result |
|---|---|---|
| 36167043149 build / 36167043173 schema-mirror | 849dbfe | build green; schema-mirror red, see below |
| 36169729548 build / 36169729603 schema-mirror | fd9f1bc | green |
| 36174422248 build / 36174422278 schema-mirror | ee26ef1 (final) | green |

The one red run was T1's exhaustive parity leg: CI's python has an older Unicode database, so Rust refused three extra patterns at U+1CCDA/U+1CCE3/U+1CCEB. That is the safe direction. The leg now forbids any under-refusal and allows extra Rust refusals only on code points the reference reports unassigned.

CI's Linux job now installs the prebuilt pinned binaries (toolkit installer at `c39ea361`, sha-checked) and runs T2, T3′ and T7, then T10, T4 and the design gate.

## Scratch

`/scratch/code/shibboleth/gui-impl2-scratch/` was deleted with `find … -delete`.
