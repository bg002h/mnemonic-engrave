# mnemonic-gui design fold 5: response map

**Result.** R4 (0C/2I/3M/3N) is answered by making the GUI **stop depending on CLI behaviour outside the probe set**, rather than probing more. Commit **`220c265`** on mnemonic-gui `gui-followups` (on `462c648`), pushed.
- **Design only.** The only code changed is the measurement harness and the Python reference planner under `design/measurements/secret-channels/`.
- **Review status:** not re-reviewed.

**Gate output.** Every script was re-run: against the release binaries, and for the demos against the **F-687 master** builds (toolkit `9da32e2f`, ms `d1ab447`, built `--locked --release`, sha256 `88cb9fbf…`/`1d13f0f8…`, matching R4).

| script | result |
|---|---|
| `run_plans.py` | FAILURES: none |
| `test_plan.py` | 0 failures (3717 NI8 legs) |
| `mutations.py` | **20/20 killed by assertion; control survived** |
| `regen_check.py --plans` | **GREEN**, including the new schema-coverage leg |
| `demo_ni5.sh` (master) | (A) 6 red, (C) GREEN, (B) 1 red |
| `demo_r4_wrappers.sh` | W2 GREEN (refused regardless), W1 red (`UNKNOWN`) |
| `demo_t4_coverage.sh` | mirror change red, cache edit red, missing file red by name |
| `check_design_tables.py` | stale blocks: none |

## Important

### NI8: a broad lookalike refusal

**The decision.** It lives in `channel_policy.json` (`channel_lookalike`) and is independent of any CLI. A resolved **or typed** secret is normalized:
1. NFKC;
2. remove Unicode categories Cc and Cf (controls, NUL, zero-width);
3. strip Unicode whitespace;
4. case-fold.

If the result equals `-` or starts with `@env`, the value is refused on **every path**, private and interim, on every OS. The refusal code is `value-looks-like-a-channel`, and the message names the field.
- **Refused:** `" @env:X"`, `"@ENV:X"`, `"​@env:X"`, `"＠env:X"`, `"- "`, `" -"`, `"@envelope"`.
- **Not refused:** `"--"`, `"-leading"`, `"pass@env:X"`.

It subsumes fold 3's measured `value-is-a-channel-spelling` check, which is removed. The planner no longer reads `reinterpret.json`, so its safety holds whatever F-691 decides about `"- "`.

**R4's padded wrapper case.** W2 is a wrapper around master ms that trims `--passphrase` before recognising a channel.
- Through a full, correct re-derivation and `regen_check --plans` it is **GREEN**. The measurement now *records* its 13 re-read spellings, and the planner refuses all of them anyway.
- The planner refuses `" @env:OTHER"` on Linux and macOS alike (`demo_r4_wrappers.sh`).
- The `run_plans.py` NC1 leg executes the whole corpus as the variable's content: 0 `$OTHER` wallets, everything refused.

**The measurement is kept, for the oracle only.** `measure_reinterpret.py` probes the 21-spelling corpus (`corpus.CHANNEL_VARIANTS`). `test_plan.py` asserts that every spelling a pinned CLI was measured to re-read is inside the predicate, so a CLI that starts re-reading something new and odd goes red.

### NI9: the refusal is the robust option, and it is the one chosen

**The decision.** It lives in `channel_policy.json` (`refuse_trailing_cr_lf`): a target whose last byte is CR or LF is refused on **every** path, `value-ends-in-newline`. Delivery then never depends on how many trailing newlines a CLI strips; strip-one, strip-all and verbatim give the same bytes for everything the GUI sends. R4's W1 case (`<pw>\n\n`) is refused.

**Kept as detection:**
- **One corpus.** `corpus.py` is shared by `run_bytes.py`, `run_plans.py` and `measure_reinterpret.py`. It has 17 suffixes (`\n`, `\r\n`, `\r`, `\n\n`, `\r\n\r\n`, `\n\r\n`, `\r\r\n`, `\n\r`, `\r\r`, `\n\n\n`, spaces, space+`\n`, spaces+`\r\n`, tab+`\n`, a tab, an interior newline) and 4 prefixes (`\n`, `\r\n`, space, tab).
- **One rule implementation.** `plan.RULES` is the only copy; `run_bytes.py` imports it.

**Shown.** W1 (master mnemonic with strip-all `@env:`), fully re-derived, is measured **`UNKNOWN`** on the 12 affected inputs and `regen_check` goes **red**. The `pw\n\n` resolution leg (R4 N6) is now backed by a measured ending. The mutation "the one strip-one rule becomes strip-all" is killed.

## Minor

| ID | What changed |
|---|---|
| Nm14 | `os_gate.py` rejects: `\|\|`, `\|`, `;` or `&` on the line (a trailing `&& …` is allowed); `--no-run` and filter/skip options; bare test-name filters, before `--` or as non-flag words after it; and a `needs:` chain reaching an `if:` job or a missing job. Fixtures: **14 decoys rejected**; **4 genuine accepted**, including `-- --nocapture && echo done`. |
| Nm15 | **In the prototype.** `regen_check.py` re-enumerates the GUI's secret sources from the schema mirror (`enumerate_sources.rs` built against the repo) and diffs them with `secret_sources.txt`. It re-derives and diffs the §A2b verdicts (`probe_missing.py`), and requires every source to have a table entry or a §A2b row. `demo_t4_coverage.sh` shows: a mirror change (ms `derive --account` marked secret) **red**; a cache-only addition **red**. |
| Nm16 | §A3b now cites F-687 **master** (ms `d1ab447`, toolkit `9da32e2f`). There, **`ms derive --passphrase=VALUE` is exact** (`argv_eq_exact` becomes `True`), `hashlock` stays not exact, and the fold-4 builds are named as earlier. `demo_ni5.sh` runs on master. The fold-4 map's suggested CLI note ("ms's `=` form trims") is **withdrawn**: it is fixed upstream. |

## Nit

| Nit | What changed |
|---|---|
| N4 | A missing cache file is a named red line (`RED derived: reinterpret.json missing from …`), not a traceback. Shown in the coverage demo. |
| N5 | A Nm13 dash leg with no oracle now asserts **fails closed, or equals the `--flag=VALUE` argv run**. |
| N6 | The `pw\n\n` leg is backed by the corpus's `\n\n` ending. |
| N7 | `env_name_rule` is documented as the **GUI's** rule. |

## A consequence worth noting

Since the planner's safety no longer reads the derived data, a stale cache can no longer produce `$OTHER`'s wallet at runtime. In fold 4 it did (demo A/B). Staleness is now caught only by identity and the diff (demo A 6 red, B 1 red), and it is **harmless** until caught.

One `run_plans.py` check was loosened, deliberately: "only expected refusal codes". On master, some corpus values make an input's only exact channel stdin, so the plan refuses `two-stdin`. A planner refusal is always safe, so any refusal is accepted; crashes are still red.

## For the controller

- **Suggested next-review scope:**
  - whether the two decisions (§A0.1) are broad enough and no broader than wanted;
  - the prototype T4 coverage leg;
  - T10's parser.
- **Scratch.** The F-687 master builds were in scratch, which is deleted. Rebuilding is `git archive` + `cargo build --locked --release`.
