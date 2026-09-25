# mnemonic-gui design fold 1: response map

**Result.** Every R0 finding (1C/3I/7M/2N) is folded into `design/DESIGN_secret_channels_and_new_forms.md` on mnemonic-gui `gui-followups`, commit **`2d244d2`** (on `ec31aed`, pushed). So are Q1–Q4 and the operator's final F-687 ruling. The fold is design only: the new code is the measurement harness and a Python reference model of the planner under `design/measurements/secret-channels/`, and no GUI code changed. It has not been re-reviewed.

**Gate output** (release binaries mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0):

| gate | result |
|---|---|
| `table_build.py` | 85 inputs, each with at least one measured-OK channel |
| `run_plans.py` | FAILURES: none |
| `check_design_tables.py` | stale blocks: none (the document carries every generated table verbatim) |
| full-pipeline re-run from scratch | every artifact byte-identical, temp paths aside |

`run_plans.py` covers 29 shapes. 27 equal their argv baseline and 2 refuse as expected. Of the swap pairs, 28 asymmetric ones differ from the baseline and 4 symmetric ones do not. All 56 resolved-`@env:` runs equal the baseline.

## Critical

| ID | What changed |
|---|---|
| **C1** | New §A3a, rewritten after the operator's F-687 ruling, which superseded the controller's earlier "refuse" instruction. A secret field holding a channel spelling means that channel; the GUI never sends those characters as the secret:<br>• **`@env:VAR`**: the GUI reads `VAR` from its own environment at Run time and sends the bytes over the planned channel.<br>• **`-`**: refused, because the GUI has no stdin of its own to forward.<br>• **Unset or empty `VAR`, or an `@env:MNEMONIC_GUI_*` name**: refused.<br>Other points:<br>• **Pin independence.** Resolution is unconditional, so the F-687 pin bump is only an edit to `channel_table.json` (the 15 WRONG cells become OK), not new code (§A3b).<br>• **Measured.** `c1_evidence.sh` shows the intended fingerprint `ca2c62d2`. The two naive readings give `fee4dc32` (typed text sent as the bytes) and `66d564d1` (literal `-`). GUI-side resolution gives `ca2c62d2`. `run_plans.py` then typed every source of every runnable shape as `@env:X`; all **56/56 equal the baseline**.<br>• **Provenance.** Preview and the confirm dialog show each secret's source, e.g. "value of $MY_PW".<br>• **Retirements listed for the implementing change:** the five help strings (`src/schema/mnemonic.rs:600, 757, 1109, 4033, 4103`, all verified); a re-pin of kittest Cell 8; two f679 tests; `is_private_channel_value` and `masked_token_is_private_channel`; and phase 1a's sentinel test, which becomes a C1 test. |

## Important

| ID | What changed |
|---|---|
| **I1** | The rule (§A4.3) is now `plan.py`. It is **authoritative**, and the design states it in prose. **A5 is generated** from it by `run_plans.py`, never hand-written.<br>• **Measurement.** Every multi-secret plan the rule produces is run through a real runner against the argv baseline, with effect lines: fingerprint, address, xpub, verdict, digest.<br>• **Group channel.** `ms combine` is modelled as a group source with a new `StdinMulti` channel (all shares on one `-`). It is measured in `measure_groups.py` and planned in A5.<br>• **Resolved mismatch.** R0's A4/A5 disagreements go away: the rule's step 2 gives stdin to the first `--X-stdin` toggle, which matches fold 0's A5 intent, and A5 is now regenerated from the rule. |
| **I2** | Added **T3′**: each runnable shape goes `plan()` → real runner, and its output must equal the argv + `--allow-argv-secret` baseline. A per-pair swap leg shows the check can fail.<br>• **How a swapping runner fails it.** It produces exactly the swapped invocation, which is measured to differ from the baseline on all **28 asymmetric pairs**.<br>• **Output-invisible swaps.** 4 pairs are invisible by the math (slip39 twice, seed-xor, ms-shares; combining shares is order-independent). They are guarded by T1 (plan level) and a new **T7** runner echo test: a helper child reports, per binding, the bytes it received on each env var, stdin or fd, and each must equal its own source's sentinel. T7 is portable and needs no CLIs. |
| **I3** | Took **Q3: refuse**.<br>• **fd channel:** Linux only. macOS stays refused until a macOS real-binary job measures `/dev/fd`; Windows is refused. **No temp files anywhere.** The three fd shapes refuse `fd-not-on-platform` on macOS/Windows; this is in the generated A5.<br>• **env/stdin on macOS/Windows:** the argv admission is deleted per OS only once that OS's real-binary job (T2, T3′, T7) is green. Until then that OS keeps today's path.<br>• The unrunnable Windows T5 mutation is removed. §A9 says where each test runs. |

## Minor and Nit

| ID | What changed |
|---|---|
| M1 | §A4.1 defines "secret source" as a union of six kinds. `enumerate_sources.rs` applies that definition to the mirror, giving 124 sources in `secret_sources.txt`. T4 regenerates it in Rust. |
| M2 | All five listed inputs measured (`run3.py`): `convert --from bip38=`, `slip39`/`ms-shares split --from entropy=`, `xpub-search passphrase-of-xpub --ms1`, `ms hashlock <ms1>`. Beyond R0's list, the whole schema was diffed against the table (§A2b): 42 sources have no entry. **32** are inputs the CLI rejects itself (e.g. `restore --from xprv=`, `export-wallet` secret slots). **10** are unmeasured: verify-bundle template `--from` (needs a ≥5-byte wallet-id fixture; a 4-byte one is refused, exit 4) and `--slot @N.wif=`. These refuse until measured, owned by the implementation cycle. |
| M3 | When a plan is refused, Copy is disabled and its tooltip shows the refusal. It never synthesises a spelling. |
| M4 | §A6 pipe mechanics: `pipe2(O_CLOEXEC)`; the whole payload is written and the write end closed **before spawn**; payloads over 4096 B are refused (`payload-too-large`); `command-fds` handles `dup2(n,n)`; a leaked write end is caught by T7's timeout. `run_plans.py` implements write-then-close, and every fd plan equals its baseline. |
| M5 | Step 1 (forced stdin) now comes first regardless of source order. T1 gains a permutation leg, and T5 names "remove step 1 → `silent-payment` reversed refuses". |
| M6 | ms hashlock:<br>• **Source exclusivity:** a conditional where the first filled source wins.<br>• **`--kind`:** starts at `(choose)`, and **Run is disabled until a kind is chosen**. An explicit `all kinds — lookup only` option omits the flag and shows a banner. Tests for both.<br>• **No-trim:** a real-binary test, **T9**, added to A9.<br>• **Q2:** build after Part A. |
| M7 | Checked: `BUILD_DESCRIPTOR_FLAGS` has no secret flag, so tree mode has no argv secret source. The Copy comment-line rule applies to the pipeline too, and `--spec -` is modelled as pre-bound stdin. |
| N1 | `table.py` now prints the actual exit code in WRONG cells. The verify-bundle WRONG cells are exit 4. |
| N2 | Resolved by I3: there are no Windows temp files, so there is no ACL claim and no startup-sweep race. |

## Q1–Q4 (R0 recommendations adopted)

| Q | Decision |
|---|---|
| Q1 | Keep the confirm dialog, reworded to "sends these secrets privately to"; it shows provenance. |
| Q2 | Build hashlock after Part A. |
| Q3 | Refuse the fd shapes off Linux; no temp files. |
| Q4 | verify-bundle measured against a **matching** bundle: `--ms1` + `--slot @0.phrase=` + `--mk1`/`--md1` as flags, from `bundle --json`, baseline `result: ok`. `--ms1`, `--passphrase` and four slot subkeys are valid rows. |

## New facts for the controller

1. **More F-687 evidence.** Against a matching bundle, `verify-bundle --ms1 -` and `--passphrase -` give exit 4 "mismatch" (a false verdict) on today's binaries. It could be added to engrave F-687. That entry was not edited.
2. **Next review scope.** It should be the fold, not a fresh audit: C1's resolve-in-GUI semantics, the rule in `plan.py` against its prose, and whether T3′, T7 and T4 together close the swap and drift classes. Everything numeric is reproducible with `BIN_DIR=… ` and the §A1 commands.
