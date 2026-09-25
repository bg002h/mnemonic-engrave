# R4 review: mnemonic-gui design fold 4 (`462c648`): the derived-data shape

**Reviewer:** opus (independent context), 2026-09-25
**Artifact:** `gui-followups` @ `462c648`, run in scratch clones under `/scratch/code/shibboleth/review-gui-r4-scratch/` (now deleted). The branch was not touched.
**Binaries:**
- **Release:** `install.sh --no-gui --no-man` installed mnemonic 0.104.0, md 0.20.3, ms 0.19.1 and mk 0.13.0. Their sha256s equal `measured_with.json`.
- **F-687 master:** ms `d1ab447` and toolkit `9da32e2f`, built from `git archive` with `cargo build --locked --release` in their own target dirs. They report `ms 0.19.1` / `mnemonic 0.104.0`, with sha256 `1d13f0f8…` / `88cb9fbf…`. md and mk are the release builds.
  - These are **not** the builds fold 4 measured (`e534917` / `9846a784`). F-687 fold 1 lies between them.

**Scope:** fold 4's shape (§A0) and its R3 fixes.
**Verdict:** **NOT GREEN: 0 Critical, 2 Important, 3 Minor, 3 Nit**

## Answer to the one question

**Can a behavioural fact go stale without a gate going red? Yes, in one class.**

Fold 4 closed the class R3 named. Every JSON fact the planner reads is now derived, and the diff plus the oracles catch every hand edit, skipped step and partial bump I tried (below).

What is still hand-kept is the **probe set** the derivation and the oracles run: which endings and which spellings get tried. A CLI behaviour that differs only outside those sets:
- re-derives **byte-identically**;
- passes `regen_check.py --plans` **GREEN**;
- gives a different wallet at exit 0. In one case that wallet is `$OTHER`'s.

I showed both cases with wrapped binaries through a full, correct re-derivation (NI8, NI9).

The real release and F-687 master binaries behave as the design assumes on every probe I added. So the design delivers the user's exact bytes **today and after the F-687 bump**. The gap is that the next bump that changes those behaviours would not be seen.

## Reproduced (settled)

| check | result |
|---|---|
| `regen_check.py --plans`, release | **GREEN**, 18.5 s |
| `demo_ni5.sh`, **F-687 master** builds | (A) 8 red; (B) 5 red, on the diff **and** on the oracle (`$OTHER`'s wallet on 3 `ms derive` shapes); (C) **GREEN**. Same shape as fold 4's run on the older builds. |
| `mutations.py` (release `BIN_DIR`) | 16/16 killed by assertion; control survived |
| `test_plan.py` | 0 failures; 236 NC1 legs |
| `check_design_tables.py` | `stale blocks: none` |
| `copy_evidence.py` (release) | `copy_evidence.md` regenerated **byte-identical**: 0 of 33 recipe mismatches, 0 of 11 typed-stdin mismatches |

**F-687 master versus the builds fold 4 measured.** A full re-derivation on master changes 13 inputs:
- `cli_env_rule` becomes `strip-one-trailing-newline` on the 13 `--passphrase` inputs, including `ms derive` and `silent-payment`, which were `null`;
- `--bip38-passphrase` stays `verbatim`;
- **`ms derive --passphrase` `argv_eq_exact` becomes `True`**. F-687 fold 1 M2 stopped ms trimming the `=` form, so fold 4's "not exact on ms `--passphrase`" is a pre-fold-1 fact. `ms hashlock --hashlock-phrase` stays `False`.

The derivation absorbs this with no code change, which is the point of the shape. See Nit N6 for the prose.

## R3 findings: status

| R3 | status | evidence |
|---|---|---|
| **NI5** | **Fixed** | Demo (B) on F-687 master: the stale ms row is red on the `reinterpret.json` diff, and independently red on the NC1 oracle (`$OTHER`'s wallet ×3). My extra partial bumps are all red (below). |
| **NI6** | **Fixed** | The control survives. My two crash probes (an import-time crash and a crash inside `plan()`) are each reported `SURVIVED … CRASH`: 16/18, control survived. |
| **NI7** | **Fixed** | The rule is per input and derived. It is spot-checked by hand below on both binary sets. My derivation mutation DM3 (report `verbatim` whatever matched) is killed by the oracle alone. |
| Nm10 | **Partly fixed** | R3's seven decoys are rejected. **Four new decoys are accepted** (Nm14). |
| Nm11 | Fixed | Copy is disabled for multi-line typed values. `copy_evidence.md` reproduced. |
| Nm12 | Fixed | Demo (C) on master ran through `slip39 split` with no hang (63 s total), and the NC1 `-` legs pass. |
| Nm13 | Fixed | `argv_eq_exact` is derived per input. DM1 (forced `True`) is killed by the dash oracle leg on `ms derive`. On master the flip to `True` is picked up as data. |
| Nit 3 | Fixed | §A9 T10 carries the acceptance criterion. "Where they run" names one job. |

## Check 1: behavioural facts the planner and runner read

| fact | where | derived by `regen_check`? | independent oracle? |
|---|---|---|---|
| channels per input (kind, flag) | `channel_table.json` | yes | T3′ baseline leg (argv) |
| terminator per channel | `channel_table.json` | yes | T3′ NI1 legs (the CLI's own `@env:`, argv-exact) |
| `cli_env_rule` per input | `channel_table.json` | yes, **over 7 fixed endings** | NI1 legs, **over 5 fixed endings** → **NI9** |
| `argv_eq_exact` per input | `channel_table.json` | yes | Nm13 dash leg (21 legs have no oracle, Nit N5) |
| re-read spellings per CLI | `reinterpret.json` | yes, **over the 2 exact tokens `-` and `@env:X`** | NC1 leg, **over the same 2 tokens** → **NI8** |
| what counts as a re-read value | `plan.py` predicate `v == "-" or v.startswith("@env:")` | **no: hand-kept code** | none beyond the 2 tokens → **NI8** |
| what each rule name does | `plan.env_value_rule` **and** a second copy in `run_bytes.RULES` | no (two hand copies) | NI1 legs, on the 5 endings only → part of NI9 |
| binary identity | `measured_with.json` | yes (sha256) | the installed binary |
| schema secret sources, and §A2b's "CLI rejects" verdicts | `secret_sources.txt`, `missing_sources.md` | **no** (§A9 says T4 does it; the prototype does not) | none → **Nm15**. Fails safe: `no-table-entry`. |
| env-name rule, "the CLI's own rule" | `channel_policy.json` | no | not needed: the GUI resolves the name itself, so a mismatch can only over-refuse (Nit N7) |
| `--` before positionals makes a leading-dash positional literal | `run_plans.baseline_argv` (the runner shape) | no | the dash leg runs value-form sources only. Standard clap semantics; not raised. |

## Check 2: breaking `regen_check` (T4)

| break | result |
|---|---|
| B1: hand edit, `mnemonic restore --passphrase` `cli_env_rule` from `verbatim` to strip | **red** ×2: the diff, and T3′ (`restore phrase+passphrase`, `restore ms1+passphrase`) |
| B1b: hand edit with no oracle shape, `ms hashlock --hashlock-phrase` `argv_eq_exact` from `False` to `True` | **red**, on the diff |
| B2: `ms` plus one appended byte (same `--version`, same behaviour) | **red**: `identity … (same version string: True)`, and the `measured_with.json` diff |
| B3: `reinterpret.json` deleted | **red, by traceback** (`FileNotFoundError`) and not a named failure (Nit N4) |
| B4: a new line in `secret_sources.txt` with no table entry | **GREEN**, which is **Nm15** |

## Check 3: the "after" case (F-687 master)

| data given to `regen_check --plans` (`BIN_DIR` = master) | result |
|---|---|
| complete re-derivation (demo C) | **GREEN** |
| demo (B): complete, with ms's re-read row left at `[-]` | **5 red** (diff, T3′, 3× NC1 `$OTHER`) |
| P1: only `measured_with.json` refreshed (the "make identity green" shortcut) | **5 red**: `channel_table` (13 inputs), `reinterpret`, `bytes`, T3′ (17 shapes), NC1 `$OTHER` on `ms derive` |
| P2: `run_bytes.py` skipped, `table_build.py` re-run on the release `bytes.json` | **cannot be produced**: `table_build.py` raises `KeyError ('mnemonic addresses --passphrase', 'DashValue', None)`. A skipped step cannot yield a cache. |
| P4: complete, with one cell hand-reverted (`ms derive --passphrase` `argv_eq_exact` back to `False`, the safe direction) | **red**, on the diff |

## Check 4: per-input rules, 6 cells by hand

Method: output hash per run. `sep` = `--flag VALUE`; `eq` = `--flag=VALUE`; `env` = `--flag @env:X`.

| cell | release: derived / hand | F-687 master: derived / hand |
|---|---|---|
| `mnemonic restore --passphrase` | verbatim, eq True / `env pw\n` = `sep pw\n`; `eq "pw  "` = `sep "pw  "` ✓ | strip-one, eq True / `env pw\n` = `sep pw`; `env pw\n\n` = `sep pw\n`; `env \npw` = `sep \npw`; `env pw\r\n` = `sep pw` ✓ |
| **`ms derive --passphrase`** | null, eq **False** / `env` is literal (the same hash for every value); `eq "pw  "` = `sep pw` (**trims**) ✓ | strip-one, eq **True** / strip-one as above; `eq "pw  "` = `sep "pw  "` ✓ |
| **`ms hashlock --hashlock-phrase`** | null, eq False / `env` literal; `eq "pw  "` = `sep pw` (trims) ✓ | unchanged ✓ |
| `mnemonic convert --bip38-passphrase` | verbatim, eq True ✓ | verbatim ✓ (not extended; the operator decision is pending) |
| `mnemonic silent-payment --passphrase` | null (`env` literal), eq True ✓ | strip-one ✓ |
| `mnemonic import-wallet --ms1` | verbatim, eq True (derived) | verbatim (derived) |

For `import-wallet --ms1` I compared the derived rows; I did not re-run it by hand.

The same runs also covered spellings outside the derivation's probes: `" @env:X"` is literal on every cell and both builds, and `"- "` exits 64 on the toolkit and is a literal passphrase on ms master (`9e82509d` ≠ `sep pw`), which is F-691.

## Check 5: `mutations.py`

- **The control survives, and a crash is not a kill.** Measured: two added crash mutations are reported `SURVIVED … CRASH`.
- **My mutations of the new derivation code** were applied, then the cache was **regenerated with the mutated code**, so the diff is green by construction and only the oracles can kill:

| mutation | binaries | cache changed? | result |
|---|---|---|---|
| DM1 `run_bytes.py`: `argv_eq_exact = True` always | release | `channel_table.json` | **killed**: T3′ `ms derive ms1+passphrase` (dash leg), no traceback |
| DM2 `measure_reinterpret.py`: `@env:` never recorded | release | `reinterpret.json` | **killed**: T3′ plus **46 NC1 `$OTHER`'s-wallet** legs |
| DM3 `run_bytes.py`: `cli_env_rule = "verbatim"` whenever a rule matched | F-687 master | `channel_table.json` | **killed**: T3′, every passphrase shape (NI1 oracle) |

## Check 6: T10 decoys beyond the ten

All four are **accepted** for macOS by `os_gate.gate` (Nm14):
1. `cargo test --test …t2 --test …t3prime --test …t7 || true`: the failure is masked.
2. `cargo test --no-run --test …`: built, never run.
3. A job with `needs:` on a job that has `if: false`: skipped at runtime.
4. `cargo test --test … -- no_such_test_name`: 0 tests run, exit 0.

---

## Important

### NI8. The re-read predicate and its measurement cover only the exact tokens `-` and `@env:X`: a CLI that also re-reads a padded spelling passes every gate and gives `$OTHER`'s wallet on the interim path

**Mechanism.** Three things are limited to the same two tokens:
- `measure_reinterpret.py` probes only `@env:REINT_VAR` and `-`;
- the planner's interim refusal is the hand-kept predicate `v == "-" or v.startswith("@env:")`;
- run_plans' NC1 leg executes only the contents `@env:OTHER` and `-`.

A CLI that also treats a padded spelling as the channel is therefore invisible to the measurement and to the oracle. That covers ` @env:X`, `-\n`, `- ` and similar. It is the R2 NC1 class again, one level out: a hand-kept fact about CLI behaviour, checked only against itself.

**Measured.** W2 is a wrapper around the F-687 master `ms` that trims a `--passphrase` value before recognising a channel spelling. I then did the bump correctly:
- the full §A1 derivation on W2 gives `channel_table.json`, `reinterpret.json` and `groups.json` **byte-identical** to the real master derivation;
- `regen_check.py --data-from <that> --plans` gives **GREEN**.

Then `ms derive ms1+passphrase` and `ms derive phrase+passphrase` on the interim path, with `USER_SECRET=" @env:OTHER"`:
- **planned** (admitted), exit 0, `master_fingerprint: 45fbfbe6`, which is **`$OTHER`'s wallet**;
- `@env:OTHER` unpadded: refused `value-is-a-channel-spelling`;
- control, real master: the same input gives `8d31199c`, the literal, and not `$OTHER`'s wallet.

**Why it is live, not hypothetical.** F-691 is open, with no owning phase: ms and the toolkit already **disagree** on `"- "` (the toolkit exits 64; ms takes it literally, as I re-measured on master). If F-691 is resolved in the "padded spelling is the channel" direction, this happens with no red.

**Why Important and not Critical.** No pinned or pending binary does this. I measured the release and master on `" @env:X"` and `"- "`.

**Remedy (either one):**
- Derive the predicate. `measure_reinterpret.py` probes a variant corpus: R3's hunt, i.e. leading and trailing space, tab, `\n`, `\r\n`, case, `--`. It records every re-read spelling, and the planner refuses a value matching any of them. NC1 executes the same corpus as contents.
- Or make the interim refusal deliberately wider than any plausible CLI: refuse when `v.strip()` equals `-` or `v.strip().casefold()` starts with `@env:`, with NC1 still executing the variant corpus.

### NI9. `cli_env_rule` is inferred from single endings, so "strip exactly one" cannot be told apart from "strip every trailing newline"

**Mechanism.**
- `run_bytes.py` derives the rule over the endings `"", \n, \r\n, \r, "  ", " \n", \nX`;
- run_plans' NI1 oracle uses `"", \n, \r\n, \nX, "  "`.

No ending has two newlines. Yet the design specifies the two-newline case: `test_plan.py` asserts `("pw\n\n", strip-one) → "pw\n"`, a fact nothing measures. The rule's behaviour is also written twice by hand, in `plan.env_value_rule` and `run_bytes.RULES`, and the two copies are compared only on those same endings.

**Measured.** W1 is a wrapper around the F-687 master `mnemonic` whose `--passphrase @env:` strips **every** trailing newline. It is identical to master on at most one newline. The full derivation on W1 gives `channel_table`, `reinterpret`, `groups` and **`bytes.json` byte-identical** to the real master derivation, and `regen_check --plans` gives **GREEN**.

Then `restore phrase+passphrase` on Linux with `USER_SECRET = <fixture>\n\n`:
- the GUI target is `'hunter2-passphrase\n'` over `--passphrase-stdin`, which gives fingerprint `5d5b0e1a`;
- the CLI's own `@env:USER_SECRET` gives `45fbfbe6`;
- that is a **different wallet, both at exit 0**.

Control, real master: `5d5b0e1a` on both. Master strips exactly one (hand check above: `env pw\n\n` = `sep pw\n`).

**Why Important.** It is the question asked: a behavioural fact, "exactly one", that goes stale with every gate green. The fix is cheap.

**Remedy:**
- Use one ending corpus for both scripts, covering every trailing sequence over `{\r, \n}` up to length 3 (`\n\n`, `\r\n\r\n`, `\n\r\n`, `\r\r\n`, …) plus a leading `\n` and a leading space.
- Make `run_bytes.py` import the rule functions from `plan.py`, so there is one copy.

## Minor

**Nm14. T10 still accepts four accidental decoys** (check 6).
- It is the same class as R3 Nm10, and Minor for the same reason: flipping an OS is a reviewed edit.
- For the Rust T10, reject:
  - `||` or `;` after the cargo command;
  - `--no-run`;
  - a trailing test-name filter after `--`;
  - a job whose `needs:` chain reaches an `if:` job.

**Nm15. The prototype T4 has no schema-coverage leg.**
- §A9 T4 says "the Rust source enumeration must equal `secret_sources.txt`, and every source must have an entry or be in §A2b". `regen_check.py` does neither: B4 is GREEN.
- §A2b's 42 "CLI rejects / unmeasured" verdicts are a one-time `probe_missing.py` measurement that nothing re-derives.
- It fails safe. An uncovered source refuses `no-table-entry` on every OS, so it is a coverage and UX gap, not wrong bytes.
- Put the leg in the prototype, or mark it "Rust only, not prototyped" in §A9.

**Nm16. Fold 4's F-687 facts describe superseded builds.** §A3b, the fold map and `ni5_demo.out` cite ms `e534917` / toolkit `9846a784`, and "`--flag=VALUE` not exact on ms `--passphrase`". On the master commits (`d1ab447` / `9da32e2f`), `ms derive --passphrase` `argv_eq_exact` is **True** (F-687 fold 1 M2). The fold map's suggested CLI-side F-687 note ("ms's `=` form trims") is already fixed upstream. No gate is affected, since the data re-derives. Re-point §A3b at the master SHAs.

## Nit

- **N4.** `regen_check.py` crashes on a missing derived file (B3): it is red, but by traceback, before step 3, and without naming the file. Report it as a `RED derived: <file> missing` line.
- **N5.** 21 Nm13 dash legs and 15 NC1 legs have no oracle. The dash legs then assert nothing: `ms verify phrase+ms1` and the three `xpub-search … ms1+passphrase` shapes. They should at least assert "fails closed, or equals the `=`-form argv run".
- **N6.** The `pw\n\n` resolution leg in `test_plan.py` states behaviour that no measurement covers. It becomes legitimate once NI9's corpus includes it.
- **N7.** `channel_policy.json` documents `env_name_rule` as "the CLI's own rule", which is a behavioural claim in the decisions file. It is harmless, because the GUI resolves names itself. Call it the GUI's rule.

---

**NOT GREEN (0C / 2I: NI8, NI9)**
