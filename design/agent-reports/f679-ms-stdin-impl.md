# F-679 ms-cli 0.19.1 — admitted argv secrets tripping the "both from stdin" guard: implementation report

Implementer: opus, 2026-09-24. Repo: `/scratch/code/shibboleth/mnemonic-secret`.
Worktree `/scratch/code/shibboleth/ms-worktrees/f679-stdin`, branch `f679-stdin` off
`origin/master` `36be26e`. Not tagged, not pushed, not merged.

Commits:
- `5ebb413` fix(cli): an --allow-argv-secret-admitted value is not a stdin read (F-679)
- `a77c8e4` release(ms-cli): 0.19.1 -- CHANGELOG for F-679

## Root cause (reviewer's I1 diagnosis confirmed)

`argv_guard::substitute` (`crates/ms-cli/src/argv_guard.rs`) replaces every admitted
value with `-` and seeds the material into `ADMITTED`, keyed by channel
(`--phrase`, `--hex`, `--passphrase`, `--hashlock-phrase`, `--ms1`, `<positional>`).
`Source::read_raw` (`parse.rs`) consults `admitted(channel)` before stdin, so the
*readers* were right. The *guards* were not: they decided "reads stdin" from the
literal `-` alone.

- `Source::reads_stdin()` = `in_path.is_none() && is_stdin_arg(arg)` — never looked
  at the side channel, so an admitted positional ms1 counted as stdin.
- `cmd/verify.rs` guard: `ms1_src.reads_stdin() && phrase_arg == Some("-")`.
- `cmd/derive.rs` guard: `entropy_reads_stdin` compared `--hex`/`--phrase` to `"-"`,
  else `ms1_src.reads_stdin()`; refused with `--passphrase-stdin`.

Evidence: the brief's repro, rerun on the fixed binary: argv+argv rc 0, `--in` rc 0,
genuine `verify - --phrase -` rc 1 with the refusal. On the parent, 5 of the 11 new
test rows failed with exactly `cannot read both ms1 and --phrase from stdin`,
`...(one stdin per invocation)`, and `not enough shares: have 1, need 2`.

## Every verb, and how

| verb | affected? | how |
|---|---|---|
| verify | **yes** | argv ms1 + argv phrase, argv phrase + stdin ms1, argv ms1 + stdin phrase all refused |
| derive | **yes** | admitted `<ms1>` / `--hex` / `--phrase` + `--passphrase-stdin` refused |
| combine | **yes, mirror image** | first `-` drained ALL admitted shares; a genuine `-` for stdin was then silently ignored (`combine --allow-argv-secret <s1> -` with s2 on stdin → under-threshold error; with more shares it would silently use fewer) |
| hashlock | no | `pick_source` admits exactly one source; no two-stdin guard exists; `--hashlock-phrase -` is already a usage error |
| encode, split, decode, inspect, repair | no | single secret channel (arg-group / one positional); no stdin-contention logic |

Searched: every `"-"`, `is_stdin_arg`, `reads_stdin`, `admitted(`, and "from stdin"
in `crates/ms-cli/src`. `derive.rs`'s argv-leak advisories compare to `"-"` too; they
are advisories only and correctly silent for admitted values (the guard owns that).

## Fix

- `parse.rs` `Source::reads_stdin()`: adds `&& argv_guard::admitted(self.channel).is_none()`
  — the same side channel `read_raw` consults, so predicate and reader cannot disagree.
- `verify.rs`: phrase contention asks `Source::new(phrase).on("--phrase").reads_stdin()`.
- `derive.rs`: `--hex` / `--phrase` contention asks their `Source` (`.on("--hex")`,
  `.on("--phrase")`); the ms1 arm already used `ms1_src`.
- `combine.rs` `read_shares`: substitution is one `-` per admitted share, so each `-`
  first takes the next admitted share; only a `-` beyond those is the real stdin read
  (read once, as before).

The guard's purpose is kept: a user-typed `-` has no admitted material on its channel,
so it still reads as stdin — including when `--allow-argv-secret` is present but
admitted nothing (tested).

## Tests — `crates/ms-cli/tests/f679_admitted_is_not_stdin.rs` (11 rows)

Every row passes an explicit stdin (empty where nothing should be read).
- verify: argv+argv; argv phrase + stdin ms1 (explicit `-` and omitted); argv ms1 +
  stdin phrase; `--in` + argv phrase; stdin+stdin refused ×4 (with/without override).
- derive: admitted ms1 / `--hex` / `--phrase` + `--passphrase-stdin`, fingerprint must
  EQUAL the private oracle `--in card --passphrase-stdin` (so the passphrase provably
  reached the derivation); `--in` with override present; stdin+stdin refused ×6.
- combine: argv+argv; admitted share + genuine stdin (both orders); `--in` with override.

Existing controls also in play: `freed_stdin.rs` `both_channels_on_stdin_are_still_refused`.

## Mutations (each APPLIED — asserted by exact-count replace + `git diff --stat` — then restored)

Run over `f679_admitted_is_not_stdin`, `freed_stdin`, `cli_combine`, `allow_argv_secret` (34 tests).

| mutation | result |
|---|---|
| M1 verify guard `if false && …` | **killed**: 3 red (`verify_genuine_stdin_twice_is_still_refused`, `freed_stdin::both_channels_on_stdin_are_still_refused`, `freed_stdin::verify_can_read_a_card_and_a_phrase_in_one_invocation`) |
| M2 derive guard `if false && …` | **killed**: 2 red (`derive_genuine_stdin_twice_is_still_refused`, `freed_stdin::both_channels_on_stdin_are_still_refused`) |
| M3 drop the `admitted(...).is_none()` clause from `reads_stdin` | **killed**: 4 red (verify ×3, derive) |
| M4 combine: first admitted `-` also marks stdin consumed | **killed**: 1 red (`combine_admitted_share_plus_genuine_stdin`) |

"The line ran": each refusal string is emitted from exactly one source line
(`verify.rs:82`, `derive.rs:438`, `git grep`-verified) and the unmutated stdin+stdin
rows assert that string, so they pass only if the guard line executed.

## Gates (at `a77c8e4`)

- `cargo nextest run --locked --workspace`: **624 passed, 11 skipped** (skips pre-existing), rc 0.
- clippy 0.1.85 (pinned 1.85.0 toolchain bin prepended) `--workspace --all-targets -D warnings`: clean.
- `cargo fmt --check`: clean.
- `ci/repro/vendor-freshness.sh`: OK (Cargo.lock changed only the ms-cli version line).
- `ms --version` → `ms 0.19.1`.

## Release

`crates/ms-cli/Cargo.toml` + `Cargo.lock` → 0.19.1. CHANGELOG: `## ms-cli [Unreleased]`
became `## ms-cli [0.19.1] — 2026-09-24` with the F-679 entry first; **the section also
carries the already-merged, untagged F-677/F-670 fixes**, so 0.19.1 ships those too.
No flag surface change, so the toolkit manual chapter `43-ms.md` needs no lockstep edit.

## Go port convergence

None needed. `grep` of `seedhammer/` and `mnemonic-engrave/third_party/seedhammer`
`*.go` for `allow-argv-secret`, the two refusal strings and `passphrase-stdin`: zero
hits. The guard is argv/stdin CLI plumbing with no firmware counterpart. Not in scope,
noted only: `me-cli` has its own `--allow-argv-secret`; it was not audited for this class.

Scratch `/scratch/code/shibboleth/f679-ms-scratch/` deleted after the run.
