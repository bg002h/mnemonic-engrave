# R5 mechanical check: mnemonic-gui design fold 5

**Reviewer:** sonnet (independent context), 2026-09-25
**Artifact:** `gui-followups` @ `220c265` (confirmed: `git rev-parse HEAD` in
`/scratch/code/shibboleth/gui-worktrees/followups/design/` = `220c265821070179c348131840d9e9365c62e3da`),
matching the fold map. Not touched: no commit, push, or branch edit made.
**Binaries:** release, via `install.sh --no-gui --no-man --root /scratch/code/shibboleth/review-gui-r5-scratch`
— `mnemonic 0.104.0`, `md 0.20.3`, `ms 0.19.1`, `mk 0.13.0`. sha256 of all four installed
binaries verified byte-equal to `measured_with.json`.
**Scope:** the fold-5 response map's claims only (mechanical re-check, not a fresh audit).
**Verdict:** **GREEN (0C/0I)** — every claim checked holds exactly as written; no new defect found.

## 1. The lookalike predicate (NI8) — HOLDS

Called `plan.looks_like_channel` directly (imported from `plan.py`, no wrapper) with the exact
values named in the brief:

| value | normalized | refused? | expected |
|---|---|---|---|
| `" @env:OTHER"` | `@env:other` | True | refuse |
| `"@ENV:X"` | `@env:x` | True | refuse |
| `"​@env:X"` (zero-width) | `@env:x` | True | refuse |
| `"＠env:X"` (full-width `@`) | `@env:x` | True | refuse |
| `"-"` | `-` | True | refuse |
| `"- "` | `-` | True | refuse |
| `"--"` | `--` | False | not refuse |
| `"-leading"` | `-leading` | False | not refuse |
| `"pass@env:X"` | `pass@env:x` | False | not refuse |
| `"correct horse battery staple"` (normal passphrase) | unchanged | False | not refuse |
| `""` (empty) | `""` | False | not refuse |
| `"пароль"` (Cyrillic) | unchanged | False | not refuse |
| `"パスワード"` (Japanese) | unchanged | False | not refuse |

All 13 match the claimed behaviour exactly — every "refused" case is refused, every
"not refused" case (including both non-ASCII passphrases and the empty value) is not.
No over-refusal of a real passphrase.

## 2. The CR/LF refusal (NI9) — HOLDS

Called `plan.plan()` end-to-end (not just the predicate) with a real `channel_table.json` entry,
on both `linux` (private-channel path) and `macos` (interim/argv path):

| value | platform | result |
|---|---|---|
| `"pass\n"` | linux, macos | refused `value-ends-in-newline` |
| `"pass\r"` | linux, macos | refused `value-ends-in-newline` |
| `"pass\r\n"` | linux, macos | refused `value-ends-in-newline` |
| `"pa\nss"` (interior LF) | linux, macos | **not refused** |
| `"pa\rss"` (interior CR) | linux, macos | **not refused** |
| `"plainpass"` | linux, macos | not refused |

A value whose last byte is CR or LF is refused on both the private and interim paths, i.e. "every
path" as claimed. A value with an interior newline is not auto-refused by this rule — consistent
with the design (DESIGN_secret_channels_and_new_forms.md lines 89/169: interior newlines are a
byte-fidelity corpus case, not a refusal condition).

**One shared value rule, confirmed by grep, not by description:**
```
plan.py:42:RULES = {
run_bytes.py:22:RULES = list(_RULES.items())
run_bytes.py:21:from plan import RULES as _RULES
```
`RULES` is defined exactly once, in `plan.py`; `run_bytes.py` imports it. No second
implementation of the rule functions exists anywhere in the directory (grepped for the
`endswith("\r\n")` pattern — one hit, in `plan.py`).

## 3. Script re-run — HOLDS, all five, against the release binaries

| script | fold-5 claim | this run |
|---|---|---|
| `run_plans.py` | FAILURES: none | `FAILURES: none` (exit 0, 11.5s) |
| `test_plan.py` | 0 failures (3717 NI8 legs) | `NI8 channel-lookalike legs: 3717` / `0 failures` (exit 0) |
| `mutations.py` | 20/20 killed by assertion; control survived | `20/20 mutations killed by assertion; control survived`, `OK CONTROL: no-op (must SURVIVE): nothing failed` (exit 0, 61s) |
| `regen_check.py --plans` | GREEN, incl. schema-coverage leg | `regen_check: GREEN` (exit 0, 42.6s; ran the full pipeline including the cargo schema-mirror enumeration) |
| `check_design_tables.py` | stale blocks: none | `stale blocks: none \| test_plan.py: 0 failures` (exit 0) |

All five numbers match the fold-5 map byte-for-byte, run fresh against the actual pinned release
binaries (not read from a cached log).

## 4. Nm15 — HOLDS: marking a schema flag secret with no table entry turns regen red

Two independent reproductions:

**(a) Direct cache edit.** Appended a synthetic, unlisted line to a scratch copy of
`secret_sources.txt` (present in neither `channel_table.json` nor `missing_sources.txt`) and ran
`regen_check.py --data-from <scratch> --no-enumerate`:
```
RED  coverage: secret source 'mnemonic totally-fake-cmd --totally-fake-secret-flag' has no channel-table entry and no §A2b row
regen_check: 1 red
```

**(b) Full `demo_t4_coverage.sh`**, run end to end against the release binaries (cargo available
via `/nix/var/nix/profiles/default/bin`), which does the real schema-mirror edit (`ms derive
--account` flipped to `secret: true` in a scratch copy of `src/schema/ms.rs`, then re-enumerated
via `enumerate_sources.rs`):
```
=== (1) mirror change: ms derive --account marked secret
RED  coverage: the schema mirror's secret sources differ from secret_sources.txt; in the mirror only: ['ms derive --account']
regen_check: 1 red

=== (2) cache-only change: a source appended to secret_sources.txt, no table entry
RED  coverage: secret source 'ms derive --account' has no channel-table entry and no §A2b row
regen_check: 2 red

=== (3) R4 N4: a missing cache file is a named RED line, not a traceback
RED  derived: reinterpret.json missing from <path>
regen_check: 1 red
```
All three legs of the fold-5 claims table row ("mirror change red, cache edit red, missing file
red by name") reproduced exactly.

## 5. The loosened check — HOLDS

**Crash still turns it red.** Built a wrapper `BIN_DIR` where `mnemonic addresses` always exits
139 (simulated crash) and every other invocation passes through to the real release binary.
Running `run_plans.py` against it:
```
FAILURES: ['addresses phrase+passphrase']
```
exit 1 — a crash is not silently absorbed by the loosened "accept any refusal" rule; it still
turns the gate red.

**No plan that should run is silently refused: admitted-plan count compared against fold 4.**
`run_plans.py`'s own `expect=` field is informational only (confirmed by reading the code: a
shape whose linux plan comes back without `bindings` returns early, before any of the `bad[]`
equality checks run — so a top-level shape refusal is structurally excluded from `FAILURES`
either way, refusal-code check or not). That is exactly why the brief asks for an external count
comparison rather than trusting `FAILURES` for this question. Extracted fold 4's committed
`plans.json` (`git show 462c648:design/measurements/secret-channels/plans.json`) and fold 5's
current `plans.json`, and counted shapes whose Linux plan has `bindings` (admitted) vs. not
(refused):

| | total shapes | admitted | refused | refused shapes |
|---|---|---|---|---|
| fold 4 (`462c648`) | 29 | 27 | 2 | `ms derive phrase+passphrase`, `ms derive hex+passphrase` |
| fold 5 (`220c265`) | 29 | 27 | 2 | `ms derive phrase+passphrase`, `ms derive hex+passphrase` |

Identical count and identical set of refused shapes — no shape that ran under fold 4 is silently
refused under fold 5's loosened check.

## Findings

None. 0 Critical / 0 Important / 0 Minor / 0 Nit.

## GREEN (0C/0I)
