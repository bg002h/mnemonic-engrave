# mnemonic-gui design fold 2: response map

**Result.** Every R1 finding (0C/2I/7M/4N) is folded into `design/DESIGN_secret_channels_and_new_forms.md` on mnemonic-gui `gui-followups`: commit **`65d807b`** (on `2d244d2`), plus `f1f9ef3`, which removes an empty stderr file committed by mistake. Both are pushed.
- Design only: no GUI code. The code that did change is the measurement harness and the Python reference planner in `design/measurements/secret-channels/`.
- Not yet re-reviewed.

**Gate output.** Every measurement script was re-run against mnemonic 0.104.0, md 0.20.3, ms 0.19.1 and mk 0.13.0.

| script | result |
|---|---|
| `table_build.py` | 85 inputs, every one with an OK channel |
| `run_bytes.py` | 146 channel cells; 0 give both sides OK with different output |
| `run_plans.py` | FAILURES: none |
| `test_plan.py` | 0 failures (includes 1260 C1 legs) |
| `mutations.py` | 12/12 planner mutations killed |
| `check_design_tables.py` | stale blocks: none, test_plan 0 failures |
| same check after an un-regenerated `plan.py` edit (step 2 disabled) | `stale blocks: ['A5_PLANS']`, exit 1 (then restored) |
| everything else | byte-identical to the pre-run snapshot, apart from temp paths and the intended changes |

## Important

### NI1: byte-exact delivery (new §A3c)

**The target bytes.** They are `env_value_rule(raw)` for a GUI-resolved `@env:VAR`, and the typed text otherwise.
- `env_value_rule` is **one entry** in the new `channel_policy.json`.
- It is `verbatim` today. That is measured: the CLI's own `@env:` equals argv-exact on all 53 EnvRef cells, for all seven endings.
- It tracks the CLIs' F-687 newline ruling by editing that one entry.

**How each channel stays exact.** Each channel cell in `channel_table.json` now carries a measured `terminator`, derived by the new `run_bytes.py`.
- **The sweep.** It covers every OK channel of every input × 7 endings: none, `\n`, `\r\n`, `\r`, trailing spaces, space+`\n`, interior `\nX`. Each run is compared against argv carrying the exact same bytes.
- **The measured terminators:**
  - stdin channels: `\r\n` (the GUI sends target + `\r\n`; the CLI strips one `\r?\n`);
  - `--decrypt-password-file`: `\n`;
  - EnvRef: none.
- **Lenient cells** (12, recorded as `null`) trim whitespace where argv would not. The planner uses them only for clean values, and otherwise refuses `value-not-byte-exact`.
- **What fold 1 got wrong.** With fold 1's no-terminator delivery, **14 passphrase stdin toggles** give a different wallet or output at exit 0/4. That is NI1, now in the data.
- **With the terminators,** every non-lenient cell equals argv-exact on all seven endings.

**The T3′-style leg the controller asked for** (in `run_plans.py`): every source of every runnable shape is typed as `@env:USER_SECRET`, with endings `""`, `\n`, `\r\n`, `\nX` and trailing spaces. Each run is compared against argv-exact **and** against the CLI's own `@env:USER_SECRET` wherever its EnvRef is OK.
- **All equal.** The per-shape counts are in §A9, e.g. restore 10/10 and 10/10.
- **Refused as designed:** 12 non-clean ms1 values on xpub-search's lenient `--ms1-stdin`.

**After the pin bump,** the change is data only: edit `env_value_rule`, then `run_bytes.py` re-derives the terminators (EnvRef included).

### NI2: the interim path (§A6)

**Which Run path each OS gets is data:** `private_channels_on` (today `["linux"]`).

**Every other OS runs the interim path.** `plan.py` defines it in three steps:
1. C1 resolution, on every OS;
2. a `no-table-entry` refusal for any unmeasured source;
3. the target bytes on argv, with `--allow-argv-secret`.

So nothing typed as `-` or `@env:`, and no unmeasured source, reaches argv literally on any OS. `test_plan.py` checks all of this on macOS and Windows: the C1 legs, `no-table-entry`, and "every binding is `Argv`".

**Flipping an OS is gated.** `test_plan.py`'s **OS gate (T10)** parses `.github/workflows/*.yml`. It fails unless every OS in `private_channels_on` has a job on that OS with `MNEMONIC_BIN` set. Linux passes through `schema-mirror`.

**The generated A5** now has columns for macOS and Windows (interim), plus "macOS/Windows once their flag flips", where the three fd shapes refuse because `fd_channel_on` is still Linux-only. T8 pins `plans_pure.json` on every OS.

## Minor

| ID | What changed |
|---|---|
| Nm1 | `guard_passthrough` refuses `@env:MNEMONIC_GUI_*` in **every** field. It has a test leg and a mutation. |
| Nm2 | `test_plan.py` legs for `C1-dash`, `-reserved-name` (two spellings), `-bad-name` (two), `-env-unset` and `-env-empty`: every source × every shape × 3 OSes = 1260. `mutations.py` kills each of those checks, and 12/12 mutations die. |
| Nm3 | Split into `gen_plans.py` (pure, no CLIs) and `run_plans.py` (measured). `check_design_tables.py` regenerates §A5 from `plan.py` on every run and runs `test_plan.py`. A planner edit that is not regenerated is red (shown above). |
| Nm4 | **T9** is in A9: real `ms`, `"  pad  "` ≠ `"pad"`, and the planned run equals argv-exact for `"  pad  "`, `"pad\n"` and `"pad\r\n"`. It is backed by the `run_bytes.py` cell. |
| Nm5 | The retirement list now has **25** help strings on secret sources, each with `file:line` and flag: 19 in `mnemonic.rs` and 6 in `ms.rs`. That is fold 1's five, all 13 of R1's, and seven more: `mnemonic.rs` `:1766`, `:2261`, `:2368`, `:2392`, `:2493`, `:2543`, `:2647`. Help on public inputs keeps its `-`. |
| Nm6 | The design says **17 WRONG cells in 15 rows**: 13 `--passphrase -`, 2 `--passphrase @env:`, `--bip38-passphrase -`, and `verify-bundle --ms1 -` (F-689). The bump procedure expects only F-687-covered cells to flip. |
| Nm7 | A Copy table covering every binding kind × provenance. For a stdin-bound value from `$MY_PW` the spelling is `printf '%s\r\n' "$MY_PW" \| …`, **measured equal to argv-exact in bash, zsh and fish** with a value ending in `\n` (fingerprint `762fff19`). An EnvRef-bound `$VAR` is spelled with the user's own `@env:VAR`. A **Copy gate** in `test_plan.py` goes red if `env_value_rule` changes while an input lacks an exact EnvRef cell. |

## Nit

| Nit | What changed |
|---|---|
| Prefix | Aligned: the reserved prefix is `MNEMONIC_GUI_` everywhere; planner names are `MNEMONIC_GUI_S<i>`. |
| Env-name rule | Adopted the CLI's `[A-Z_][A-Z0-9_]*` (`C1-bad-name`). |
| `payload-too-large` | Modelled in `plan.py`, with a test and a mutation. |
| T7 | The helper parses its own argv for `@env:NAME`, `/dev/fd/N` and `-`/`--X-stdin`. |

## Also fixed during the fold

1. **Kittest Cell 8.** `MNEMONIC_MS1_0` is not under the reserved prefix, so Cell 8 is re-pinned to GUI-side **resolution**. A new cell pins the reserved-prefix refusal.
2. **`combos.sh`** left two fixture temp files behind. It now cleans them up with a trap.

## For the controller

Scope the next review to fold 2:
- the terminator model in §A3c against `run_bytes.py`;
- the interim path and the OS gate;
- the Copy table.

Every number is reproducible with the §A1 commands and `BIN_DIR` set.
