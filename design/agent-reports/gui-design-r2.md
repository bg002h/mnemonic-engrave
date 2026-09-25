# R2 re-review: mnemonic-gui design fold 2 (`65d807b`, `f1f9ef3`)

**Reviewer:** opus (independent context), 2026-09-25
**Artifact:** `gui-followups` @ `f1f9ef3`: `design/DESIGN_secret_channels_and_new_forms.md` and `design/measurements/secret-channels/`. I ran everything in a local clone; the branch was not touched.
**Binaries:** a fresh `install.sh --no-gui --no-man` into scratch installed mnemonic 0.104.0, md 0.20.3, ms 0.19.1 and mk 0.13.0.
**Scope:** fold 2 only, not a fresh audit.
**Verdict:** **NOT GREEN: 1 Critical, 2 Important, 2 Minor, 2 Nit**

## Reproduced (settled)

| check | result |
|---|---|
| `run_bytes.py` | `bytes.json`, `bytes.md` and `bytes.txt` are **byte-identical** to the committed copies (146 cells) |
| `table_build.py` | 85 inputs; 0 with no OK channel |
| `run_plans.py` | `FAILURES: none`; `plans.json` and `t3.md` are identical |
| `test_plan.py` | 1260 C1 legs, 0 failures |
| `mutations.py` | 12/12 killed |
| `check_design_tables.py` | `stale blocks: none` |
| `c1_evidence.sh` | byte-identical to `c1_evidence.out`, including the Copy lines for bash, zsh and fish |

### Terminator model under the brief's extra endings

I ran `run_bytes.py`'s harness again with these endings on all 146 cells: `""`, `\n\n`, `\r\n\r\n`, `\t`, `\r\r\n`, `\n\r`, `\r`, and ` \t\n`. For each cell I then checked the terminator recorded in `channel_table.json`.
- **Every recorded terminator still gives argv-exact output on every new ending.** There were 0 mismatches.
- Lenient cells still match on clean values.
- `EnvRef` with terminator `""` equals argv-exact on all the new endings.

### Constructed values through the planner's own runner

I typed `@env:MY_PW` into the passphrase source, resolved it with `plan.py`, and ran the result with `run_plans.build()` on Linux and with the interim argv invocation for macOS. I compared both against argv-exact and against the CLI's own `@env:MY_PW`. Shapes: restore, bundle, silent-payment and ms derive.

| `$MY_PW` | Linux plan | interim (macOS/Win) | argv-exact | CLI `@env:MY_PW` |
|---|---|---|---|---|
| `hunter2\r` | 4b82c850 | 4b82c850 | 4b82c850 | 4b82c850 |
| `hunter2\n\n` | 76c20d5f | 76c20d5f | 76c20d5f | 76c20d5f |
| `hunter2\r\n\r\n` | 1ddb1664 | 1ddb1664 | 1ddb1664 | 1ddb1664 |
| `\n` alone | d493bc20 | d493bc20 | d493bc20 | d493bc20 |
| `\r\n` alone | c93901c0 | c93901c0 | c93901c0 | c93901c0 |
| empty | refuse `C1-env-empty` | refuse | 73c5da0a | 73c5da0a |
| `hunter2\t` | 3fd82710 | 3fd82710 | 3fd82710 | 3fd82710 |
| NUL | not reachable via `@env:` (the OS environment cannot hold NUL) | — | — | — |
| **`@env:OTHER`** (OTHER=hunter2) | **a1c20c4f** | **ca2c62d2** | ca2c62d2 | **a1c20c4f** |
| `-` | 66d564d1 | 66d564d1 | 66d564d1 | 66d564d1 |

The fingerprints are for restore. bundle gives the same pattern with `mk1` output. On silent-payment and ms derive, the CLI's `@env:` is literal (these are §A2's WRONG cells), and the GUI matches argv-exact on every row.

Every ordinary ending is exact on both paths, and **the interim path applies `env_value_rule`**, so macOS/Windows and Linux agree. The one exception is the `@env:OTHER` row (**NC1**).

### Pin-bump coupling

- **Does a rule/binary disagreement go red?** Yes. I flipped `env_value_rule` to `strip-one-trailing-newline` against today's verbatim binaries, and `run_plans.py`'s NI1 leg went red on 26 shapes (exit 1). The mismatch is symmetric, so a pin bump to an F-687 CLI without editing the rule also goes red, provided T3′ runs in `schema-mirror` against the pinned binaries, as §A9 says it will.
- **What happens if the table is regenerated but the rule is not edited?** T4 and T2b stay green: the `EnvRef` terminator becomes `\n`. The T3′ NI1 leg is then the only thing that fails.
- **Versions are not recorded.** No test ties the table to a CLI version, and `channel_table.json` and `bytes.json` carry no binary version (only `measured_in`). The tie is "T3′/T2b/T4 run against the pinned binaries in CI". That is adequate, but it exists only once T3′ is implemented in CI (Nit 2).
- **The Copy gate** goes red under the flipped rule (for ms and `xpub-search --ms1`), as designed.

## R1 findings: status

| R1 | status | evidence |
|---|---|---|
| NI1 | **Fixed** | terminators hold on 15 distinct endings, measured; NI1 leg re-run green; rule mismatch goes red (above) |
| NI2 | **Partly fixed** | the interim path is defined and the per-OS switch is data. But the interim *resolution* is untested (**NI3**), and argv double-resolves sentinel-shaped values (**NC1**). The OS gate goes red as claimed (below), with holes (**Nm8**). |
| Nm1 | Fixed | `guard_passthrough` has a test leg and a mutation |
| Nm2 | Fixed | 1260 legs; `mutations.py` kills each C1 check (12/12) |
| Nm3 | Fixed | I disabled step 2 in `plan.py` without regenerating: `check_design_tables.py` gives `stale blocks: ['A5_PLANS']`, exit 1 |
| Nm4 | Fixed | T9 is in §A9 (line 492) |
| Nm5 | Fixed (not re-counted) | 25 strings listed |
| Nm6 | Fixed | "17 WRONG cells in 15 rows" |
| Nm7 | Fixed for the stdin/`$VAR` row | `printf '%s\r\n' "$MY_PW" \| …` equals argv-exact in bash, zsh and fish for `\r`, `a\\nb`, `%d%s`, edge spaces, a leading tab, `\n\n`, `-`, a lone `\n` and `\r\n\r\n` |
| Nits | Fixed | prefix, name rule, `payload-too-large`, T7 parses argv |

**The OS gate shown red.** Adding `macos` to `private_channels_on` gives `FAIL os gate: macos has a real-binary CI job`, exit 1.

**Does Copy print a secret?** No. Per §A7 every row prints a reference (`@env:NAME`, `"$MY_PW"`, `<FILE>`) or a prompt comment, never a value.

---

## Critical

### NC1. The interim argv path double-resolves a resolved value that is itself a CLI sentinel, giving a different wallet from Linux and from the CLI's own `@env:` at exit 0

**Mechanism.**
- C1 resolves `@env:MY_PW` to the target bytes.
- The interim path (§A6 step 3) puts those bytes on argv.
- The CLI treats argv values beginning with `@env:` (`env_sentinel.rs:60`) and, on stdin-reading cells, `-` as channel spellings, and **resolves them again**.

§A6's claim that "the interim invocation *is* the argv baseline, so T3′ holds by construction" assumes argv is byte-transparent. For these values it is not.

**Measured.** `restore phrase+passphrase`, typed `@env:MY_PW`, with `MY_PW='@env:OTHER'` and `OTHER=hunter2`:
- Linux plan: `a1c20c4f`;
- CLI's own `--passphrase @env:MY_PW`: `a1c20c4f`, because the CLI resolves once;
- **interim (macOS/Windows): `ca2c62d2`**, which is `$OTHER`'s wallet;
- `bundle slot+passphrase` gives the same split: `mk1qph72j…` on Linux and via the CLI, `mk1qpdkj2…` on the interim path.

**Reach.** It needs a variable whose content starts with `@env:`. That is contrived, but it is a different wallet at exit 0, on two of three OSes, with no warning.
- After F-687, `-` means stdin on every command. A resolved value of exactly `-` then joins this class: the argv `-` reads the (empty) stdin.

**Remedy.**
- On the interim path, refuse (`value-is-a-channel-spelling`) any resolved or typed target that equals `-` or begins with `@env:`. Typed values already cannot reach this, because C1 intercepts them.
- Add a `test_plan.py` leg that pins the refusal, and a mutation for it.
- Correct §A6's "by construction" sentence.

## Important

### NI3. Nothing tests that the interim path sends the *resolved* bytes; the R0-C1 regression on macOS/Windows survives every gate

**Mutation.** I made `plan()`'s interim branch return the caller's unresolved `sources` instead of `res`, so the typed `@env:MY_PW` goes to argv literally. That is R0 C1 again.
- `check_design_tables.py`: `stale blocks: none`, `test_plan.py: 0 failures`, **exit 0**.

**Why nothing catches it.**
- **The pure tests don't look at values.** `test_plan.py` checks only that interim bindings are `kind == "Argv"` and that the C1 *refusals* fire. It never checks the values the interim invocation carries.
- **T8 can't see it.** `plans_pure.json` holds bindings with no values, so it cannot catch a Rust interim path that passes typed text through.
- **The real-runner tests never run the interim path.**
  - T3′'s NI1 leg runs only the Linux planner.
  - T7 is specified as "the first leg of any macOS or Windows job", and no such job exists.
  - §A9 T5 assigns "restore the admission" to T3′/T7, but neither of them executes the interim plan.

R1's NI2 remedy asked to "keep a C1 test on the interim path"; the fold kept only the refusal half.

**Remedy.**
- Add a pure leg: for every shape on macOS and Windows, with each source typed as `@env:VAR`, the interim invocation's argv must carry `env_value_rule(raw)` and never the typed text.
- Add the matching mutation to `mutations.py`.
- Have T7 (or T3′) also execute the interim plan on Linux, by forcing the platform, so that the runner half is covered before any macOS job exists.

### NI4. Copy's `EnvRef`/typed recipe `read -rs VAR` trims edge whitespace in bash and zsh, so the pasted command derives a different wallet from Run

**Where.** §A7, row `EnvRef` | typed: `# read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1`. The line was carried over from fold 1, and R1 missed it.

**Cause.** In bash and zsh, `read` without `IFS=` strips leading and trailing IFS whitespace.

**Measured.** The user types the value and presses Enter. Compared against argv-exact:

| passphrase | argv-exact | bash | zsh | fish (`read -sx`) |
|---|---|---|---|---|
| `"  pad  "` | `bc403662` | **`8728f37c`** | **`8728f37c`** | `bc403662` |
| `"\tpad"` | `6e09a416` | **`8728f37c`** | **`8728f37c`** | `6e09a416` |

A wrong wallet at exit 0 contradicts §A7's "a pasted command behaves like Linux's Run". It is the very edge-whitespace case that T9 exists to pin.

**Remedy.**
- Change the recipe to `IFS= read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1`.
- Add a T6 leg that runs the printed recipe with edge-whitespace values in bash, zsh and fish.

## Minor

### Nm8. The OS gate (T10) has false greens and a false red, and today nothing runs it in CI

The gate is `"MNEMONIC_BIN" in block` with `RUNNER[os] in runs-on`. I ran it with `macos` added to `private_channels_on`:

| decoy workflow | expected | actual |
|---|---|---|
| macOS job with `cargo build` only and `MNEMONIC_BIN` in a comment | red | **0 failures** |
| macOS job with `if: false` | red | **0 failures** |
| a genuine `runs-on: ${{ matrix.os }}` job with `macos-latest` in the matrix | green | **red** |

- **The stated criterion is not checked.** §A6 says the job "must run T2, T3′ and T7". The gate never looks for those tests. Linux passes today through `schema-mirror`, which runs none of them.
- **CI never runs the gate.** No workflow calls `test_plan.py` or `check_design_tables.py` (grep of `.github/`).

**Remedy.** In the Rust T10, key the gate on a job step that invokes the named T2/T3′/T7 test targets with `MNEMONIC_BIN` in its `env:`; reject `if:` guards; and resolve matrix `runs-on`.

### Nm9. Copy has no stated spelling for `StdinMulti` (`ms combine` share group) with `$VAR` provenance

The group payload is newline-joined with no terminator (`run_plans.build`), so the `printf '%s<T>'` row does not apply. Specify it, or disable Copy for it.

## Nit

1. **`C1-env-empty` and NUL are checked only at edges.**
   - `C1-env-empty` tests the *raw* value. Under `strip-one-trailing-newline`, `$MY_PW = "\n"` resolves to an empty target that is not refused. It is harmless, because both sides give "no passphrase", but it is inconsistent with the refusal's stated reason.
   - A typed value holding NUL is delivered on Linux: a stdin toggle gives `bcb5b513`. On the interim path it fails at spawn, because argv cannot carry NUL. Add a `nul-in-value` refusal on every OS, for a clear message.
2. **No binary version is recorded.** `channel_table.json` and `bytes.json` do not record the binaries they were measured with. Record them, so that T4's "at the pinned binaries" can assert them.

---

Scratch (`/scratch/code/shibboleth/review-gui-r2-scratch/`) was deleted with `find … -delete`. No commits, no pushes, no edits to the branch (`git status` is clean at `f1f9ef3`).

**NOT GREEN (1C / 2I)**
