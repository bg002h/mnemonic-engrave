# R3 re-review: mnemonic-gui design fold 3 (`42dacaa`)

**Reviewer:** opus (independent context), 2026-09-25
**Artifact:** `gui-followups` @ `42dacaa`, run in a scratch clone. The branch was not touched. Scratch is deleted.
**Binaries:**
- **Release:** `install.sh --no-gui --no-man` installed mnemonic 0.104.0, md 0.20.3, ms 0.19.1 and mk 0.13.0.
- **F-687:** built from `git archive` of toolkit `9846a784` and ms `e534917` (`cargo build --locked`, own target dirs). **Both report the unchanged versions `mnemonic 0.104.0` / `ms 0.19.1`.** md and mk are the release builds.

**Scope:** fold 3 only.
**Verdict:** **NOT GREEN: 0 Critical, 3 Important, 4 Minor, 1 Nit**

## Reproduced (settled)

| check | result |
|---|---|
| `measure_reinterpret.py` (release) | mnemonic 0.104.0 `[-, @env:]` (42/53); ms 0.19.1 `[-]` (14). `reinterpret.json` is **byte-identical** to the committed copy. |
| `run_plans.py` (release) | `FAILURES: none`; `plans.json` and `t3.md` are identical |
| `test_plan.py` | 0 failures; 1260 C1 legs; 236 NC1 legs |
| `c1_evidence.sh` | byte-identical to `c1_evidence.out` |
| `copy_evidence.py` | 0 of 33; typed stdin row 0 of 11; share group == argv in bash, zsh and fish |
| `check_design_tables.py` | `stale blocks: none`, exit 0. With one T8 `value_cases` payload tampered: `plans_pure.json (T8 parity file) differs`, **exit 1** |
| `mutations.py` | prints 15/15 killed, but **the kills are vacuous** (**NI6**) |

## R2 findings: status

| R2 | status | evidence |
|---|---|---|
| **NC1** | **Fixed** for 0.104.0 / 0.19.1 and, updated as data, for the F-687 branches. The refusal is one code path, driven by the row. See below. The row's staleness is **not gated** (**NI5**). |
| **NI3** | **Fixed** | R2's mutation plus three of mine are all red in `test_plan.py` **and** `run_plans.py` (below). This was measured with the pin file reachable; see NI6 for why that matters. |
| **NI4** | **Fixed** | Copy's recipe never changes the bytes of a value it can carry, measured on 19 values × 3 shells. A value with an interior newline is a gap (**Nm11**). |
| Nm8 | **Partly fixed** | The three decoys and a one-target job are rejected. Seven other false greens remain (**Nm10**). `linux: pending` is recorded only in §A6 prose (**Nit 3**). |
| Nm9 | Fixed | share group == argv in all three shells, and typed == argv (re-run) |
| Nit 1 | Fixed | `env-empty after rule`, `nul-in-value` on every OS, with a mutation |
| Nit 2 | Fixed | `measured_with.json`, `env_value_rule.measured_with`, row `version` |
| Pin check | **Works as a version-label check** | See the Pin-bump section below. It cannot see a stale measurement under an updated label (NI5). |

### NC1: measured

**What the row would become.** `measure_reinterpret.py` against the F-687 builds:
- mnemonic 0.104.0: `[-, @env:]`, with 54 inputs for `-` and 54 for `@env:`;
- ms 0.19.1: `[-, @env:]`, with 15 inputs for `-` and 1 for `@env:`.

That matches the design's prediction ("ms gains `@env:`").

**Updated as data, no second path.** I did a full bump simulation in a second clone:
- re-ran `run_all`/`run2`/`run3`, `measure_groups`, `run_bytes`, `table_build` and `measure_reinterpret`;
- set `env_value_rule` to `strip-one-trailing-newline`;
- copied the row from `reinterpret.json`;
- ran `gen_plans` and `run_plans`.

The result: **118 NC1 records, 0 interim plans admitted.** The single check at `plan.py` handles both CLIs from the row alone.

**The brief's values** (`restore phrase+passphrase` and `ms derive ms1+passphrase`; OTHER = the fixture passphrase; OTHER's wallet = `45fbfbe6`):

| `$MY_PW` | release: CLI `@env:MY_PW` / Linux / interim | F-687 + updated row: CLI / Linux / interim |
|---|---|---|
| `@env:OTHER` | a1c20c4f / a1c20c4f / **refused** | a1c20c4f / a1c20c4f / **refused** (ms derive too) |
| `-` | 66d564d1 / 66d564d1 / refused | 66d564d1 / 66d564d1 / refused |
| `@env:` | 445a7a91 / 445a7a91 / refused | 445a7a91 / 445a7a91 / refused |
| ` @env:OTHER` | 8d31199c / 8d31199c / 8d31199c | 8d31199c ×3 |
| `-\n` | d1b2c83b / d1b2c83b / exit 64 (clap) | 66d564d1 / 66d564d1 / refused (target `-` after the rule) |
| `@env:OTHER\n` | 6a365759 / 6a365759 / refused | a1c20c4f / a1c20c4f / refused |

- F-687's "`@env:` holding `@env:X` or `-` is literal" is confirmed: the CLI's own `@env:MY_PW` never gives OTHER's wallet.
- On ms derive with the release binaries, the CLI's `@env:` is literal (§A2 WRONG cell), and the GUI matches argv-exact as designed.

**The trim-mismatch hunt.** For every one of the 84 single-input rows, I put these on argv against both binary sets. A spelling counts as re-read when the output equals the secret's own output.
- `@env:` variants: `@env:R`, ` @env:R`, `\t@env:R`, `@env:R `, `@env:R\n`, `@env:R\r\n`, `@ENV:R`, `\n@env:R`, `@env: R`.
- `-` variants: `-`, ` -`, `- `, `-\n`, `-\r\n`, `\t-`, `\n-`, `--`.

Only the exact `@env:R` and `-` were ever re-read. Counts:
- release: `@env:` 53, `-` 56;
- F-687: `@env:` 55, `-` 69.

**0 re-read variants escape `v == "-" or v.startswith("@env:")`.** There is no Critical.

### NI3: mutations on the interim path

The pin file was reachable for these runs; see NI6.

| mutation (interim branch of `plan()`) | test_plan | run_plans |
|---|---|---|
| R2's: returns `sources` (typed text) | **red** | **red** (26 shapes) |
| mine: strips trailing `\n` from resolved values (the rule bypassed on interim only) | **red** | **red** (21 shapes) |
| mine: swaps the resolved values of sources 0 and 1 | **red** | **red** |
| mine: applies strip-one-newline itself under a `verbatim` policy | **red** | **red** (21 shapes) |

### NI4: Copy, with my own values

My added values:
- `endslash\`, `x\`, `\\`, `a\tb` (a literal backslash-t), `trail `: in bash, zsh and fish, each **equals argv-exact**. The typed stdin row equals it too.
- `-leading` and `--help`: argv-exact **errors** (clap, exit 64). The recipe and the stdin row both give the wallet of the literal value, and they agree (`11b9d5c7`, `9d299016`). So Copy does not change the bytes.
- `mid\nline`: see **Nm11**.

**Copy never changes the bytes of a single-line value.**

### Pin-bump check

In the scratch clone, I changed `pinned-upstream.toml`:
- `ms-cli-v0.20.0` gives 3 failures: `measured_with`, `env_value_rule measured with ms`, and `argv_reinterprets ms`;
- `mnemonic-toolkit-v0.105.0` gives 3 failures, of the same kinds;
- `mk-cli-v0.14.0` gives 1 failure (`measured_with`).

Reverting gives 0 failures, and `check_design_tables.py` is clean. The check works as a version-label comparison. See NI5 for what it cannot see.

---

## Important

### NI5. A stale `argv_reinterprets` row passes every gate; at the F-687 bump that is R2 NC1 again, on ms

**Mechanism.** Only one thing ties the row to the binaries: `test_plan.py` checks `row == reinterpret.json` and `version == pin`. Both files are hand-committed. Nothing else checks the row:
- `measure_reinterpret.py` is not in any CI job. §A9 lists T2, T2b, T3′, T9 and T6 for `schema-mirror`, and T4 regenerates `channel_table.json`, not `reinterpret.json`.
- `run_plans.py`'s NC1 leg does not execute the interim invocation. It derives `interim_should_refuse` **from the same policy row** (`run_plans.py`, the `sp = POLICY["argv_reinterprets"]…` line), so it is circular.

**Measured.** I used the bump clone (F-687 binaries, the rule and table regenerated). There I set ms's row and `reinterpret.json` back to today's `["-"]`, with versions unchanged, as a bump that skipped step 3 or hand-edited it would leave them.
- `test_plan.py`: no pin, row or NC1 failure;
- `run_plans.py`'s failure list is **unchanged**, and `ms derive ms1+passphrase` is not in it. Its NC1 records read `('@env:OTHER', 'planned', interim_should_refuse=False)`.
- The interim run itself, `MY_PW='@env:OTHER'`: CLI `@env:MY_PW` = `a1c20c4f`, Linux = `a1c20c4f`, **interim = `45fbfbe6`**, which is OTHER's wallet, at exit 0.

**Aggravating: the version label cannot tell these binaries apart.** The F-687 branches report `mnemonic 0.104.0` and `ms 0.19.1`, the same as the releases. The pin check is satisfied by either behaviour.

**Why this is Important.** §A3b says "a pin bump cannot land on stale measurements". That holds for the labels, not for the measurement. And the bump that is coming changes exactly this row.

**Remedy (either one):**
- Make the NC1 real-runner leg an **oracle**. Where the interim plan is admitted, execute it and require it to equal argv-exact of the target, and never OTHER's wallet. That is the same shape as the NI3 leg.
- Or regenerate `reinterpret.json` in `schema-mirror` against the pinned binaries and diff it, T4-style.

### NI6. `mutations.py` reports every mutation killed, including a no-op, because fold 3's pin check makes `test_plan.py` crash in the temp copy

**Mechanism.**
- `mutations.py` copies `*.py`, `*.json` and `fixtures/` into `tempfile.mkdtemp()`.
- Fold 3's pin check opens `HERE/../../../pinned-upstream.toml`, which does not exist there. `FileNotFoundError` is raised, so exit is 1, and that is counted as "RED".

**Measured.**
- An unmutated `test_plan.py` in such a copy gives `FileNotFoundError: …/../../../pinned-upstream.toml`, rc=1.
- `mutations.py` with a no-op mutation added (a comment edit) prints `RED CONTROL no-op` and **16/16 mutations killed**.
- I then put `pinned-upstream.toml` at the right relative place:
  - the no-op **survives** (`GREEN (mutation survived)`);
  - all 15 real mutations are still red. Two of them, "unset allowed" and "no-table-entry off", are red via a `KeyError` crash, not an assertion.

**So the fold's "15/15 killed" is true, but only by coincidence.** T5 as shipped cannot fail. Any future mutation would be reported killed whether or not a test catches it. Under this repo's rule, a gate that cannot fail blocks.

**Remedy:**
- Copy the tree with its relative layout, or pass the pin path in.
- Add a no-op control that must survive.
- Distinguish an assertion failure from a crash.

### NI7. F-687 makes the `@env:` rule per-flag, so the documented bump ("a data edit, no planner code changes", the single `env_value_rule`) cannot be completed as data

**Measured.** I did the full bump against the F-687 builds, with `env_value_rule = strip-one-trailing-newline`. `run_plans.py`'s NI1 leg is red with `eq_argv_exact: True, eq_cli_env: False` on:
- `convert wif+bip38-passphrase`: `--bip38-passphrase` and `--from wif=` `@env:`;
- `convert bip38+bip38-passphrase`;
- `import-wallet 2 cosigner ms1`: the `--ms1`/cosigner `@env:` cells.

On those flags the F-687 CLI's own `@env:` is still **verbatim**. Directly: `--bip38-passphrase @env:X` with `X=$'pw\n'` gives `…WuC3ms…`, while argv `pw` gives `…W6yN8y…`, on both builds. On `--passphrase` it strips. That matches the brief's statement of F-687 ("on `--passphrase`").

**Consequence.**
- With one global rule, the GUI's target for those cells would differ from what the CLI's own `@env:` uses.
- The gate catches it: it is red, not a silent wrong wallet. But §A3b's claim, restated in fold 3 as "the F-687 behaviour as implemented", is false.
- The rule has to become per-input (or per-flag) data. That changes `env_value_rule(raw)`'s signature and every caller: planner code, T8 parity and the Copy gate. It has to be designed before the bump, not discovered at it.

**Remedy:**
- Key `env_value_rule` by channel-table input, or by flag.
- Measure it per cell as part of `run_bytes.py`: the CLI's own `@env:` versus argv-exact, over the endings.
- Correct §A3b.

## Minor

### Nm10. T10 still has false greens

`os_gate.gate(..., "macos", targets)` **accepts** each of these jobs (my fixtures):
- `continue-on-error: true` on the job;
- a step `run: echo cargo test --test …t2 --test …t3prime --test …t7`;
- `cargo build --test …` (built, never run);
- a `run: |` block whose cargo line is a **shell comment**;
- `on: workflow_dispatch` only (never runs on PRs);
- a matrix whose `exclude:` removes `macos-latest`;
- `MNEMONIC_BIN: ""`.

It **rejects** the three committed decoys and a one-target job, as claimed. It also falsely rejects a workflow-level `env: MNEMONIC_BIN`, a false red.

An accidental form like `continue-on-error` or a commented line could flip an OS with no real-binary tests behind it. It is Minor because flipping an OS is a reviewed human edit. For the Rust T10: require `cargo test`/`nextest run` as a command token, reject `continue-on-error`, honour `exclude`, require a PR/push trigger, and require a non-empty `MNEMONIC_BIN`.

### Nm11. The typed-`EnvRef` Copy recipe reads one line; a typed value with an interior newline gives a different wallet at exit 0

`mid\nline`:
- argv-exact `74db797b`;
- the recipe in bash, zsh and fish: `bdb4bfbc`, which is the wallet of `mid`;
- the `--passphrase-stdin` row: `74db797b`, correct.

The GUI holds the typed value, so it can tell. For a typed value containing CR or LF, Copy should use the stdin spelling or be disabled with a tooltip. Otherwise §A7's "a pasted command behaves like Linux's Run" does not hold for it. It is Minor because an interior newline in a passphrase field is unusual.

### Nm12. `run_plans.py`'s slip39 oracle puts the NC1 content on argv; at the F-687 bump the harness hangs, then false-reds or passes vacuously

**Mechanism.**
- Fold 3's NC1 leg sets `pw5 = content`, which is `-` or `@env:OTHER`.
- `normalise()` then runs `mnemonic slip39 combine --allow-argv-secret --passphrase <pw5>` with **inherited stdin**.

**Under F-687:**
- `--passphrase -` reads stdin. With stdin a live pipe, `run_plans.py` **hung**: I observed the process blocked on that exact argv for more than 10 minutes. With `< /dev/null` the leg reports a false `linux_eq_cli_env: False` on `slip39 split phrase+passphrase`.
- `--passphrase @env:OTHER` is resolved against the harness's own environment, where `OTHER` is unset. Both sides error, so `same()` compares two empty outputs, and that leg passes **vacuously**.

**Remedy:** recover the shares by a private channel (stdin with `input=`), and pass `stdin=DEVNULL` everywhere.

### Nm13. The interim path gives the CLI a leading-dash value as a separate argv word

This is outside fold 3's scope, but found on the brief's path. A target like `-\n` or `-leading` on the interim path makes clap exit 64 ("unexpected argument"). It fails safe, with no wallet, but it is a CLI error the GUI does not anticipate or explain. Use `--flag=value` on the interim path, or refuse the value with a message.

## Nit

**Nit 3.** "`linux: pending` → hard in the implementing change" is stated once, in §A6 prose. It is not in §A9's T10 bullet or in any acceptance list. There is no plan yet, so put it where the plan will inherit it. §A9's "Where they run" also puts T10 in two jobs, which is harmless but should be one.

---

**NOT GREEN (0C / 3I: NI5, NI6, NI7)**
