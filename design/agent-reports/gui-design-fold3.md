# mnemonic-gui design fold 3: response map

**Result.** Every R2 finding (1C/2I/2M/2N) is folded into `design/DESIGN_secret_channels_and_new_forms.md` on mnemonic-gui `gui-followups`, and so is the controller's pin-bump check. Commit **`42dacaa`** (on `f1f9ef3`), pushed. It is design only: the code that changed is the measurement harness and the Python reference planner under `design/measurements/secret-channels/`. Not yet re-reviewed.

**Gate output.** Every measurement script was re-run against mnemonic 0.104.0, md 0.20.3, ms 0.19.1 and mk 0.13.0.

| script | result |
|---|---|
| `table_build.py` | 85 inputs, every one with an OK channel |
| `measure_reinterpret.py` | mnemonic 0.104.0 `[-, @env:]`; ms 0.19.1 `[-]` |
| `run_plans.py` | FAILURES: none |
| `copy_evidence.py` | 0 of 33 recipe mismatches |
| `test_plan.py` | 0 failures |
| `mutations.py` | 15/15 killed |
| `check_design_tables.py` | stale blocks: none |

A tampered T8 payload turns the check red: `plans_pure.json (T8 parity file) differs`, exit 1. The pipeline's artifacts are byte-identical to the pre-run snapshot, apart from temp paths and the intended additions.

## Critical

### NC1: a resolved value the CLI would re-read

**The measurement.** `measure_reinterpret.py`, a new script, checks every table input: is a value of `@env:VAR` or `-` resolved **again** when it arrives on argv?
- mnemonic 0.104.0 re-reads both `-` and `@env:`, on 42 and 53 inputs respectively.
- ms 0.19.1 re-reads `-`, on 14 inputs.

**The rule.** It is one policy row, `argv_reinterprets`, keyed by CLI and version. `test_plan.py` asserts the row equals `reinterpret.json`. After F-687, both CLIs are expected to list both spellings, and the bump updates the row as data.

**The refusal.** The interim argv path refuses `value-is-a-channel-spelling`, naming the field and saying why ("`<cli> <version>` reads this on the command line as a channel, not as the secret"). The private path carries such values as bytes, and was measured never to re-read them. §A6's "by construction" sentence is corrected.

**The reviewer's case, `MY_PW='@env:OTHER'`** (`c1_evidence.sh`):

| run | fingerprint |
|---|---|
| the CLI's own `@env:MY_PW` | `a1c20c4f` |
| GUI Linux plan | `a1c20c4f` |
| unguarded interim (the defect) | `ca2c62d2`, which is `$OTHER`'s wallet |
| GUI interim, fold 3 | refused |

**Tests.**
- `run_plans.py` NC1 leg, every shape, with `@env:OTHER` and `-`:
  - Linux equals the CLI's own `@env:` on every cell that has an OK EnvRef;
  - Linux is **never** `$OTHER`'s wallet (0 across all shapes);
  - interim refuses wherever the policy says.
- `test_plan.py`: 236 NC1 interim legs.
- The mutation "NC1 refusal off" is killed.

## Important

### NI3: the interim path is tested with real bytes

**Pure leg** (`test_plan.py`): every source of every shape on macOS and Windows, typed as `@env:USER_SECRET`, under both `env_value_rule` values. The interim plan must carry `env_value_rule(raw)`, never the typed text.

**Real-runner leg** (`run_plans.py`): the interim plan is **executed on Linux by forcing the OS**, with endings `""`, `\n`, `\r\n` and trailing spaces. It is compared against argv-exact of a target computed **independently of `plan.py`**. All shapes are equal (e.g. restore 8/8).

**The reviewer's mutation** (the interim path sends the typed text) is in `mutations.py`. It turns **both** `test_plan.py` and `run_plans.py` red.

**T8's parity file now pins values.** `plans_pure.json` gains `value_cases`: per shape and OS, the typed case and each source typed as `@env:` with a trailing `\n`, with the exact payload bytes of every binding. `check_design_tables.py` fails if the file differs from a regeneration, shown with a tampered payload.

### NI4: the Copy typed-secret recipe

The recipe is now `IFS= read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1` in bash and zsh, and `read -s -x --delimiter \n MNEMONIC_GUI_S1` in fish.

`copy_evidence.py` runs the printed recipes in all three shells with 11 values: the reviewer's `"  pad  "`, a leading tab, a trailing tab, `" a  b "`, a backslash, an interior tab, quotes, `*`, `$HOME`, `%s%d`, and a plain value.
- **New recipe:** 0 of 33 mismatches against argv-exact.
- **Fold-2 recipe, reproduced:** bash and zsh are wrong on `"  pad  "`, `"\tpad"`, `"pad\t"` and `" a  b "`.
- **The typed stdin row** (`--passphrase-stdin`, value + Enter): 0 of 11 mismatches.
- `-` and `@env:…` are excluded, with a stated reason: C1 handles them before any typed recipe, and argv-exact would itself re-read them.

The table is in §A7 and gated verbatim. T6 gains the shell leg.

## Minor

| ID | What changed |
|---|---|
| Nm8 | `os_gate.py` parses the YAML and requires a job that has no `if:`, runs on the OS (literal or `${{ matrix.* }}`, resolved with `include`), invokes **every** target in `real_binary_test_targets` through cargo, and has `MNEMONIC_BIN` in the step or job `env:`. Pinned against `fixtures/ci/`: the comment-only, `if: false` and missing-target decoys are rejected, and the genuine matrix and Linux jobs are accepted. **CI wiring:** the implementing change adds `secret_channels_t10` to the Linux `schema-mirror` job. The repo's workflows cannot pass until those targets exist, so the prototype **reports** `linux: pending`; the Rust T10 is hard from the implementing change on. That part stays pending. |
| Nm9 | Share-group Copy rows. Typed: `ms combine -- -` with a paste-each-line comment. `$VAR`: `printf '%s\n' "$S1" "$S2" \| ms combine -- -`. Measured equal to argv in bash, zsh and fish, and the typed form too. |

## Nit

| Nit | What changed |
|---|---|
| 1 | `C1-env-empty` is now judged on the **target** after `env_value_rule`: a variable holding only `\n` refuses under the F-687 rule, and has a test. `nul-in-value` refuses on **every** OS, with a test and a mutation. |
| 2 | Versions are recorded: `measured_with.json` from `table_build.py`, `measured_with` on `env_value_rule`, and `version` on every `argv_reinterprets` row. |

## Controller item: the pin-bump check

**The check (T11, in `test_plan.py`).** It compares `pinned-upstream.toml` (parsed with tomllib) against:
- `measured_with.json`;
- every CLI-behaviour policy row's version;
- `reinterpret.json`.

Any difference is red, so a pin bump forces a re-measure. §A3b's bump procedure now lists it, the F-687 behaviour as implemented, and the `argv_reinterprets` update.

## For the controller

- **Next review:** scope it to fold 3:
  - NC1's policy row and refusal;
  - the interim real-runner leg;
  - the Copy shell table;
  - T10's parser;
  - the pin check.
- **One pending item:** T10 against the repo's real workflows. It can only go hard in the implementing change, which creates the targets it looks for.
