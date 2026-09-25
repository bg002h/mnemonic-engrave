# R1 re-review: mnemonic-gui design fold 1 (`2d244d2`)

**Reviewer:** opus (independent context), 2026-09-25
**Artifact:** `gui-followups` @ `2d244d2`, `design/DESIGN_secret_channels_and_new_forms.md` + `design/measurements/secret-channels/`
**Binaries:** fresh `install.sh --no-gui --no-man` into scratch: mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0 (versions printed)
**Scope:** the fold and the F-687 ruling only. This is not a fresh audit.
**Verdict:** **NOT GREEN: 0 Critical, 2 Important, 7 Minor, 4 Nit**

## Reproduced (settled; later rounds need not re-derive these)

- **`run_plans.py`:** `FAILURES: none`. `plans.md` is byte-identical to the committed copy.
- **C1 `@env:` runs:** 56/56 equal the baseline.
- **Swap pairs:** 32 in total. The 28 asymmetric pairs all differ from the baseline; the 4 symmetric pairs do not.
- **`c1_evidence.sh`:** byte-identical to the committed `c1_evidence.out`. The fingerprints are `ca2c62d2` (intended), `fee4dc32` (typed text sent as the bytes), `66d564d1` (literal `-`), and `ca2c62d2` again for the design's resolution.
- **`check_design_tables.py`:** `stale blocks: none`.
- **Deliberately swapping runner (T3′).** I wrapped `build()` so that the runner writes source *i*'s value into source *j*'s channel. `plan()` was untouched. Against the argv baseline:
  - pair (0,1): **22 caught**, 4 invisible (exactly the 4 symmetric share sets);
  - pair (1,2): 3 caught;
  - pair (0,2): 3 caught;
  - **28/28 asymmetric pairs caught**, as the fold claims.

  27 of the 28 are caught because the swapped run exits non-zero; only `bundle wsh-multi 2 slots` swaps to exit 0 with different output.
- **T7 prototype.** A helper stands in for the CLI and echoes, per channel named in argv (`@env:NAME`, `/dev/fd/N`, stdin), the bytes it received. The unmutated runner gives 0/27 red. The swapping runner gives 26/27 red, and **all 4 symmetric shapes go red**. The only green shape is `ms combine`, which has one source and so has nothing to swap. So T7 does catch the output-invisible swaps.
- **M5 permutation leg.** Across 29 shapes × 3 platforms, no permutation changes plan-vs-refuse (0 mismatches). `silent-payment` with its sources reversed still plans `--passphrase-stdin` plus the fd.
- **I1 mutations.** Two edits to `plan.py`, each followed by re-running `run_plans.py`:
  - step 2 disabled;
  - `ENV_PREFIX` renamed.

  Both turn `check_design_tables.py` red (`stale blocks: ['A5_PLANS']`). `run_plans.py` itself stays exit 0, because the plans still run correctly.
- **The 10 unmeasured §A2b sources** refuse `no-table-entry` in `plan.py` (checked `verify-bundle --from phrase=` and `--slot @N.wif=`).
- **F-687 double resolution (check 2).** The in-progress toolkit F-687 tree (`tk-worktrees/f687`, uncommitted, `passphrase_input.rs`) resolves `@env:VAR` once, verbatim, and does not re-classify the resolved value. `plan.py resolve()` is also single-pass. **Nothing double-resolves.** The claim that "the pin bump is a table edit" holds for code; see Nm6 for the prose.

## R0 findings: status

| R0 | status | evidence / residue |
|---|---|---|
| C1 | **Fixed on the planner path**, not on the interim path | 56/56 resolved runs = baseline; `-` refused in every source (run_plans C1 leg). The interim macOS/Windows admission path is undefined: see **NI2**. |
| I1 | **Fixed** | A5 is generated from `plan.py`, and a mutation goes red once the tables are regenerated. The check itself does not regenerate: Nm3. |
| I2 | **Fixed** | Swapping runner: 28/28 caught by T3′, 4/4 symmetric caught by the T7 prototype. |
| I3 | **Partly fixed** | The Windows temp file is gone, fd is Linux-only, and the fd shapes refuse off Linux in the generated A5. The per-OS "delete admission only after a green real-binary job" rule is **prose only**; nothing in `plan.py`, the tests or CI enforces it. Folded into **NI2**. |
| M1 | Fixed | §A4.1; 124 sources. |
| M2 | Fixed | 16 new rows plus §A2b. |
| M3 | Fixed | §A7: Copy is disabled, and its tooltip shows the refusal. |
| M4 | Fixed | §A6 pipe mechanics. `payload-too-large` is not modelled in `plan.py` (Nit). |
| M5 | Fixed | 0 permutation mismatches. |
| M6 | Mostly fixed | Exclusivity and the `(choose)` kind are in place. **T9 is not in A9** (Nm4). |
| M7 | Fixed | §A4.5. |
| N1 | Fixed | The exit code is printed. |
| N2 | Fixed (moot) | There are no temp files. |

---

## Important

### NI1. A resolved `@env:VAR` whose value ends in `\n` or `\r\n` gives a different wallet from the CLI's own `@env:VAR`, at exit 0

**The mechanism.** §A3a promises that the resolved bytes "go through the channel the plan assigns, exactly like a typed value". But the planner sends a passphrase over `--passphrase-stdin` (step 2), and stdin strips one trailing `\r?\n` (§A3.5). The CLI's `@env:` reads the variable verbatim, both on 0.104.0 and in the F-687 working tree. So for such a value, the GUI's Run and the CLI given the same spelling derive different wallets. The fold's 56 `@env:` runs used values with no trailing newline, so they cannot see this.

**Measured.** `restore phrase+passphrase` and `silent-payment secret+passphrase`, typed `@env:MY_PW`, resolved by `plan.py`, run through `run_plans.build()`:

| `MY_PW` | plan | GUI result | argv with the exact bytes | CLI `--passphrase @env:MY_PW` |
|---|---|---|---|---|
| `hunter2\n` | StdinToggle | restore `ca2c62d2` | `762fff19` | `762fff19` |
| `hunter2\r\n` | StdinToggle | restore `ca2c62d2` | `2ee01640` | `2ee01640` |
| `hunter2` | StdinToggle | restore `ca2c62d2` | `ca2c62d2` | `ca2c62d2` |
| `hunter2\n` | StdinToggle | silent-payment `sp1qqgzt…` | `sp1qqdpq…` | (literal today) |

**Consequences.**
- A user who restores with the GUI and later, with the CLI, runs `--passphrase @env:MY_PW` gets a different wallet, with no error on either side.
- Copy (A7) makes the gap direct: it prints `@env:MNEMONIC_GUI_S1` with `# export MNEMONIC_GUI_S1="$MY_PW"` only for env-bound sources. The Run path used stdin, which strips; the Copy path would use env, which does not.

**Reach.** It needs an environment value that ends in a newline. `$(…)` and `read` strip newlines, so this is rare, but it is a wrong wallet at exit 0.

**Remedy.**
- In step 0, a value ending in `\n` must not take a stdin channel. Use `EnvRef` or fd, and refuse if neither is available, naming the reason. Alternatively refuse such values outright.
- Add a T1/T3′ leg with a trailing-`\n` value, plus a T5 mutation.
- Fix §A3.5's "writes a value's exact bytes" so that it states the stdin strip applies to the value.

### NI2. The interim admission path (macOS/Windows until their job is green; Linux before §A4.6) is undefined under C1, and contradicts A5 and T8

**The contradiction.**
- §A6 says env and stdin on macOS and Windows are used "**after** its real-binary job is green; until then, today's admission path".
- §A5 and T8 say the macOS and Windows plans are "same as Linux", and that the Rust `plan()` must produce `plans.json` on every platform.
- Today's admission path (`src/form/invocation.rs:80-117`, `:230-249`) **passes `-` and `@env:` through to argv**: `masked_token_is_private_channel` exempts them from the opt-in.

**What an implementer will do.** The design retires `is_private_channel_value`, and rewrites the help strings globally to "`@env:VAR` (read by the GUI)". It never says that `resolve()` runs before the interim path. That leaves two natural implementations:
- **"Keep today's path."** Typed `@env:MY_PW` in `silent-payment --passphrase` is taken literally (a wrong wallet, measured by `c1_evidence.sh`). `--passphrase -` gives the literal-`-` wallet `66d564d1`. This is exactly R0 C1, on two of three OSes, with help text that now promises otherwise.
- **"Keep the admission, with `is_private_channel_value` gone."** `-` is masked, earns `--allow-argv-secret`, and is again the literal `-`.

**Enforcement (check 5).** "Delete the admission per OS once that OS's job is green" has no mechanism. It is not in `plan.py`, not in any T-leg, and not in CI. The macOS and Windows jobs are listed only as follow-ups (§A10) with no owning date.

The same gap covers brief item 3. On the interim path, the 10 unmeasured §A2b sources go to argv rather than refusing. That is exposure only, and the right wallet, *if* C1 resolution runs first; it is undefined otherwise.

**Remedy.**
- State that `resolve()` (C1) runs on **every** OS before **either** path, so the interim path is "resolved bytes on argv + `--allow-argv-secret`", which is the baseline.
- Make the per-OS switch a single named constant, for example `PRIVATE_CHANNELS_ON: &[Os]`, that `plan()` reads, and have `plans.json` and T8 reflect it. A macOS or Windows job flips the constant in the same change that adds the job.
- Keep a C1 test on the interim path.

---

## Minor

- **Nm1. `@env:MNEMONIC_GUI_*` in a pass-through (non-secret) field is not refused.** R0's C1 remedy asked for this; the fold refuses the name only in secret sources.
  - The CLI does resolve `@env:` on non-secret inputs. Measured: `addresses --from xpub=@env:MNEMONIC_GUI_S0` gives the same address as the literal xpub.
  - In a multi-source run, such a field reads another source's secret from the child's environment. Measured: `bundle --slot @0.phrase=@env:MNEMONIC_GUI_S0 --slot @1.xpub=@env:MNEMONIC_GUI_S0` (S0 = the phrase) gives `base58check decode`, exit 1. That fails closed and echoes nothing.
  - The GUI-side resolver cannot cross-read, because it reads the GUI's own environment and the planner sets variables only on the child.
  - Refuse the prefix in every Text field, or document why failing closed is enough.
- **Nm2. The C1 refusals for reserved, unset and empty names have no test leg.**
  - I removed the reserved-name and empty checks from `plan.py`. `run_plans.py` and `check_design_tables.py` both stay green.
  - The empty case matters: the CLI takes an empty `@env:` as the empty passphrase (measured `73c5da0a` "passphrase: none", exit 0).
  - Add a T1 C1 leg for each of `C1-reserved-name`, `C1-env-unset` and `C1-env-empty`, and a T5 mutation.
- **Nm3. `check_design_tables.py` does not regenerate A5 from `plan.py`.** I mutated `plan.py` and ran only the check: `stale blocks: none`, exit 0. The plan.py → plans.json link holds only when `run_plans.py` is re-run, and that needs the binaries. T8 pins Rust to `plans.json`, not to `plan.py`. Split the pure planning half out of `run_plans.py` so the check can regenerate §A5 without CLIs.
- **Nm4. T9 is claimed as "added to A9" (line 558), but A9 has no T9.** The fold map says the same. `grep -n T9` finds only line 558.
- **Nm5. The C1 retirement list names five help strings, but many more teach `-` in a secret field, which C1 now refuses.**
  - `mnemonic.rs`: `:1346`, `:1871`, `:1960`, `:2062`, `:2154`, `:2204`, `:2316`.
  - `ms.rs`: `:85`, `:249`, `:309`, `:466`, `:479`, `:570`.

  The refusal fails closed, but the help will contradict it.
- **Nm6. §A3a/§A3b say "the 15 WRONG cells" flip to OK after the bump.**
  - A2 has **17** WRONG cells in 15 rows.
  - In the F-687 working tree, `convert --bip38-passphrase` is still taken literally (`convert.rs:868-872` clones the argv value). `verify-bundle --ms1` resolves only `@env:` (`verify_bundle.rs:64-65`), so `--ms1 -` stays WRONG unless F-687 widens.

  The bump procedure is data-driven, so the planner stays safe. The stated expectation will mislead the reviewer of the bump diff.
- **Nm7. Copy is specified only for an env-bound resolved value** (`# export MNEMONIC_GUI_S1="$MY_PW"`). The usual passphrase plan is stdin-bound, and for a stdin binding whose provenance is `$MY_PW`, A7 gives no spelling. Specify one, and keep it consistent with NI1.

## Nit

- `plan.py` refuses the prefix `MNEMONIC_GUI_S`, while the prose says `MNEMONIC_GUI_…`. Align them.
- The GUI accepts `@env:` names the CLI rejects (lowercase, a leading digit). That is harmless, but the GUI and CLI rules diverge. Consider the CLI's POSIX-name rule.
- `payload-too-large` (§A8) is not modelled in `plan.py`.
- T7's helper should resolve channels **from argv** (as my prototype did), not from the plan's binding list, so that it also catches an argv/env name mismatch introduced by the runner.

---

## Answers to the brief's questions

1. **C1.**
   - Can a typed `@env:` or `-` still reach a CLI literally? On the Linux planner path, no, in any secret source. On the interim path, it is undefined (NI2).
   - Is the `MNEMONIC_GUI_*` refusal enough to stop one source reading another's secret? In secret fields, yes. It is not enough in pass-through fields (Nm1), where the result fails closed.
   - What does Copy print? `@env:MNEMONIC_GUI_Sn` plus an `export` comment when the source is env-bound; nothing is specified when it is stdin-bound (Nm7).
2. **Ruling, and double resolution.** No double resolution in code. The "only `channel_table.json` changes" claim holds; the "15 cells" expectation does not (Nm6).
3. **I1.** `plan.py` is authoritative, and a mutation goes red after regeneration, but the check does not regenerate (Nm3). The unmeasured sources refuse in `plan.py`; they fall back to argv on the interim path (NI2).
4. **I2.** Confirmed. T3′ catches 28/28 asymmetric swaps, and T7 catches the 4 symmetric ones (prototype).
5. **I3.** The per-OS rule is prose only (NI2).
6. **Minors.** M3, M5 and M6 were spot-checked: M3 and M5 are fixed, and M6 is fixed except for T9 (Nm4).

Scratch (`/scratch/code/shibboleth/review-gui-r1-scratch/`) was deleted with `find … -delete`. No commits, no pushes, no edits to the branch.

**NOT GREEN (0C / 2I)**
