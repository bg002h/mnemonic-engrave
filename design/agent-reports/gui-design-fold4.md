# mnemonic-gui design fold 4: response map

**Result.** R3 (0C/3I/4M/1N) is answered by **changing the shape**, as the controller directed, and not by patching instances. The change is committed as **`462c648`** on mnemonic-gui `gui-followups` (on `42dacaa`), and pushed.
- It is design only: the only code touched is the measurement harness and the Python reference planner under `design/measurements/secret-channels/`.
- It has not been re-reviewed.

**Gate output.** Every script was re-run against the release binaries (mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0).

| script | result |
|---|---|
| `table_build.py` | 85 inputs, all with an OK channel |
| `run_plans.py` | `FAILURES: none` |
| `test_plan.py` | 0 failures |
| `mutations.py` | **16/16 killed by assertion; the no-op control survived** |
| `regen_check.py` | **GREEN** |
| `demo_ni5.sh` against the F-687 builds | **(A) 8 red, (C) GREEN, (B) 5 red** |
| `check_design_tables.py` | stale blocks: none |

**F-687 builds.** The measurements used toolkit `9846a784` and ms `e534917`, built with `cargo build --locked --release` from `git archive`, with the release md and mk alongside them. Both builds report `mnemonic 0.104.0` / `ms 0.19.1`, and their sha256s differ from the releases.

## The shape change (new §A0)

**1. Behaviour is derived, never hand-kept.** `env_value_rule` and `argv_reinterprets` are **removed from `channel_policy.json`**, which now holds decisions only. Every behavioural fact lives in a **derived cache**, produced by the measurement pipeline against the pinned binaries:

| cache | contents |
|---|---|
| `channel_table.json` | channels, terminators, the **per-input** `cli_env_rule`, and `argv_eq_exact` |
| `reinterpret.json` | which argv values each CLI re-reads |
| `measured_with.json` | per CLI: **pinned tag + `--version` + sha256** |

**2. CI regenerates the cache and diffs it** (`regen_check.py`, T4). It:
- checks the installed binaries' **sha256** against `measured_with.json`;
- re-derives every behavioural file in a scratch copy of the repo layout and diffs it; any difference is red;
- with `--plans`, also runs `test_plan.py` and `run_plans.py` on that data.

**3. Every expected answer comes from an independent oracle.** The real-runner legs compare against the **CLI's own `@env:VAR` run**. Where an input has no working CLI `@env:`, they compare against **known bytes**: argv-exact, or, for bytes argv would re-read, a non-lenient `--X-stdin` toggle.
- The NC1 leg **executes** every plan it admits, on both paths, and requires "never `$OTHER`'s wallet". It no longer derives "should refuse" from the row.
- Another hand-kept gate turned up and was removed: `shapes.py`'s `expect: refuse`. Under F-687, `ms derive … + --passphrase` legitimately becomes plannable. A shape that plans must now pass the oracle legs; a refusal is always safe.

## Important

### NI5: shown failing in CI terms, against the F-687 builds

`demo_ni5.sh` → `ni5_demo.out`, which is gated verbatim into §A0.

| scenario | result |
|---|---|
| **(A)** committed cache vs the F-687 binaries | **8 red**: identity for mnemonic and ms (sha256 differs, "same version string: True"); diffs in `channel_table.json` (13 inputs), `reinterpret.json` (ms `['-']` vs measured `['-', '@env:']`), `measured_with.json` and `bytes.json`; T3′ failures; and an NC1 **`$OTHER`'s wallet** on `ms derive`. |
| **(B)** R3's exact scenario: everything re-derived on F-687, but ms's row left at `[-]` | **5 red**. The `reinterpret.json` diff is red. **Independently**, the oracle leg executes the admitted interim plans and finds **`$OTHER`'s wallet** on three `ms derive` shapes. |
| **(C)** the bump done right (complete re-derivation) | **GREEN**: identity, every diff, `test_plan.py` and every oracle leg. So the bump is data-only. |

### NI6: the mutation harness can fail now

`mutations.py` is rewritten:
- **Layout:** each mutation runs in a scratch copy that **keeps the repo layout**: `pinned-upstream.toml`, `.github/`, and the design doc.
- **Kill rule:** a mutation is killed **only by an assertion failure**: `N failures` with N > 0 and no traceback, a changed A5, or `run_plans.py` printing `FAILURES: [...]`. A crash counts as **survived**.
- **Crash-proof legs:** `test_plan.py`'s `refusal()` now turns a crash into an assertion failure. That fixes R3's two KeyError "kills".
- **Control:** a **no-op control must survive**, or the harness exits 1.

Result: **16/16 killed by assertion; control survived.** On its first run the new harness **found a real gap**. Turning off the Nm13 refusal survived, because `-leading` has no whitespace for ms's `=` form to trim. A `-lead  ` value and a pure leg were added. A second gap, a crash in the new Nm13 leg under "interim off", was fixed the same way.

### NI7: a per-input value rule, derived

`run_bytes.py` now derives `cli_env_rule` **per input**. For each candidate rule `f` it tests whether the CLI's own `@env:VAR` equals argv-exact of `f(raw)` over the endings.

| build | per-input rule |
|---|---|
| release | `verbatim` ×53, `null` (no CLI `@env:`) ×31 |
| F-687 | `strip-one-trailing-newline` on exactly the **13 `--passphrase` inputs**; `verbatim` on `--bip38-passphrase` and the rest |

**The pending operator decision** to extend F-687 to `--bip38-passphrase` or `--decrypt-password` is handled either way: it is a re-derivation, with no code change.

**Changes elsewhere:**
- `plan.resolve()` takes the table; an `UNKNOWN` rule refuses, and `test_plan.py` is red while any input is `UNKNOWN`.
- The Copy gate is per input.
- §A3b is corrected.

### Binary identity by content

`measured_with.json` now records the pinned tag and the **sha256** per CLI:
- `test_plan.py` checks the tags against `pinned-upstream.toml`;
- `regen_check.py` checks the sha256s against the installed binaries.

The F-687 builds, which share the release versions, fail this check (demo (A)).

## Minor

| ID | What changed |
|---|---|
| Nm10 | **Hardened; the signed-artifact option was not chosen.** Flipping an OS is a reviewed one-line edit and the decoys are accidents, so signing would add key management for no real threat. Now required: a `push`/`pull_request` trigger (YAML's bare `on:` → `True` is handled); no `if:` or `continue-on-error`; matrix `include` added and **`exclude` removed**; a real shell command line (comments stripped, `VAR=` prefixes allowed) of `cargo test` / `cargo nextest run` naming every target; a **non-empty** `MNEMONIC_BIN` at step, job **or workflow** level. Fixtures: all **10 decoys rejected** (R3's seven plus the original three); **matrix, Linux and workflow-level env accepted** (the false red is fixed). |
| Nm11 | Copy is **disabled** for a typed EnvRef-bound value holding CR or LF, with a tooltip. Measured (`copy_evidence.md`): `mid\nline` and `a\r\nb` through the `read` recipe give the first line's wallet in bash, zsh and fish; the typed stdin row equals argv-exact and stays. |
| Nm12 | `normalise()` recovers slip39 shares over `--passphrase-stdin` (input = pw + `\r\n`), never argv. There is no inherited stdin anywhere. (C) on F-687 ran green with no hang. |
| Nm13 | **Measured, and the controller's suggested remedy was unsafe as written.** On ms 0.19.1, `--passphrase=VALUE` **trims**: `--passphrase=pw\n` gives `pw`'s wallet, while `--passphrase pw\n` keeps the newline. The planner therefore **derives `argv_eq_exact` per input**: exact on 39 inputs, **not** on `ms derive --passphrase` or `ms hashlock --hashlock-phrase`. On the interim path, a leading-dash value uses `--flag=VALUE` only where it is exact; otherwise it refuses `value-starts-with-dash`. All other values stay as separate words. Oracle legs cover `-leading`, `--help` and `-lead  ` on both paths. |

## Nit

**Nit 3.** §A9 T10 now carries the **acceptance criterion**: the change implementing Part A must make T10 pass against the repo's own workflows, and from then on "pending" is a hard failure. §A9 "Where they run" names **one** Linux job, for T2, T2b, T3′, T4, T9, T10 and the shell leg.

## For the controller

- **Next review.** Scope it to the shape:
  - is any behavioural fact still hand-kept, or read back as its own oracle?
  - `regen_check.py`;
  - the oracle definitions in `run_plans.py`;
  - the `argv_eq_exact` refusal;
  - the demo.
- **F-687 finding for the CLI side.** ms's `--passphrase=VALUE` form trims where the separate-word form does not. That may be worth a line in engrave F-687.
- **Scratch.** The F-687 builds are in the scratch directory, which is deleted at the end of this task. Rebuilding them is `git archive` plus `cargo build --locked`, as above.
