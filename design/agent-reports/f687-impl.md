# F-687 implementation report — `--passphrase -` = stdin, `--passphrase @env:VAR` = environment

Agent-written (Opus 5.5), 2026-09-25. Brief: `design/briefs/f687-passphrase-note.md`, plus three controller messages received mid-task: verify-bundle must assert `result: ok`; one newline rule across the three forms; operator: "Nobody has ever made a wallet before using this software".

## Outcome

Both CLIs now read `--passphrase` through a single resolver per repo, on every subcommand that takes the flag. `-` reads stdin, byte-for-byte the same as `--passphrase-stdin`. `@env:VAR` reads the environment. A literal value keeps working and prints exactly one stderr note. Nothing is refused for using `-` or `@env:`. Two pre-existing toolkit defects were found along the way and fixed, because the new rule could not hold without fixing them (see "Found and fixed").

| repo | branch | commit | base |
|---|---|---|---|
| mnemonic-secret | `f687-passphrase-channels` (worktree `/scratch/code/shibboleth/ms-worktrees/f687`) | `e534917` | `4b02906` (origin/master) |
| mnemonic-toolkit | `f687-passphrase-channels` (worktree `/scratch/code/shibboleth/tk-worktrees/f687`) | `9846a784` | `b72dbdfa` (origin/master) |

Nothing is pushed, merged, tagged or version-bumped. The worktrees were checked out on branch `f687-note`, left from the earlier brief, which had no commits. I created `f687-passphrase-channels` at the same base. `f687-note` still exists, unused.

## Per-command table (measured)

Reference: "abandon ×11 about" with `TREZOR` via `--passphrase-stdin`. Each cell compares that form's output with the reference: `same` or `DIFF`, then the exit code, then `nN` = the number of stderr lines naming a passphrase on argv. "none" = no passphrase, which is a control proving the passphrase changes the output. verify-bundle ran against a matching `bundle --json`. slip39 split is compared by what its shares combine to.

**Before** (toolkit 0.104.0 at `b72dbdfa`, ms 0.19.1 at `4b02906`):

| command | `-` (no override) | `-` + `--allow-argv-secret` | `@env:PP` | literal + override | none |
|---|---|---|---|---|---|
| mnemonic addresses | refused(2) | DIFF(0) n1 | same(0) n0 | same(0) n1 | diff |
| mnemonic restore | refused(2) | DIFF(0) n1 | same(0) n0 | same(0) n1 | diff |
| mnemonic derive-child | refused(2) | DIFF(0) n1 | same(0) n0 | same(0) n1 | diff |
| mnemonic bundle | refused(2) | DIFF(0) n1 | same(0) n0 | same(0) n1 | diff |
| mnemonic convert | refused(2) | DIFF(0) n1 | same(0) n0 | same(0) n1 | diff |
| mnemonic silent-payment | refused(2) | DIFF(0) n1 | **DIFF(0) n1** | same(0) n1 | diff |
| mnemonic slip39 split | refused(2) | DIFF(0) n1 | same(0) **n1** | same(0) n1 | diff |
| mnemonic slip39 combine | refused(2) | DIFF(0) n1 | same(0) **n1** | same(0) n1 | diff |
| mnemonic verify-bundle | refused(2) | **DIFF(4)** n1 | same(0) n0 | same(0) n1 | diff |
| mnemonic xpub-search path-of-xpub | refused(2) | DIFF(4) n1 | same(0) n0 | same(0) n1 | diff |
| mnemonic xpub-search account-of-descriptor | refused(2) | DIFF(4) n1 | same(0) n0 | same(0) n1 | diff |
| mnemonic xpub-search passphrase-of-xpub | refused(2) | DIFF(4) n1 | same(0) n0 | same(0) n1 | diff |
| ms derive (`--in card.ms1`) | **DIFF(0)** n1 | DIFF(0) n1 | refused(1) | same(0) n1 | diff |

**After** (both branches): every command, in every column, reads `same(0)`, with `n0` for `-`, `-`+override and `@env:`, `n1` for the literal, and `diff` for none. On verify-bundle, `-` produces `result: ok`, and the test asserts that line.

verify-bundle keyless-template completion (wsh-sortedmulti 2-of-2, operator seed with TREZOR, BIP-87 origins, `--expect-wallet-id`):

| form | before (0.104.0) | after |
|---|---|---|
| `--passphrase-stdin` | **rc 4 (false NO MATCH)**, 1 spurious note | rc 0 OK, 0 notes |
| `--passphrase -` | rc 2 (argv refusal) | rc 0 OK, 0 notes |
| `@env:PP` | rc 0, **1 spurious note** | rc 0, 0 notes |
| literal + override | rc 0, **2 notes** | rc 0, 1 note |
| none (control) | rc 4 | rc 4 |

## Byte rule (measured, both CLIs, master fingerprints)

**Before** (stdin flag / `-` / `@env:PP` / argv literal; mnemonic ↔ ms):

| bytes | `--passphrase-stdin` | `--passphrase -` | `@env:PP` | argv literal |
|---|---|---|---|---|
| `TREZOR` | b4e3f5ed | rc2 / 66d564d1 | b4e3f5ed / rc1 | b4e3f5ed |
| `TREZOR\n` | b4e3f5ed | rc2 / 66d564d1 | **48efb44f** / rc1 | 48efb44f |
| `TREZOR\r\n` | b4e3f5ed | rc2 / 66d564d1 | **45494db3** / rc1 | 45494db3 |
| `TREZOR\n\n` | 48efb44f | rc2 / 66d564d1 | d1f56c4d / rc1 | d1f56c4d |
| `TRE\nZOR` | c6278769 | rc2 / 66d564d1 | c6278769 / rc1 | c6278769 |
| ` TREZOR ` | 9a2a7b73 | rc2 / 66d564d1 | 9a2a7b73 / rc1 | 9a2a7b73 |

**After** (the two CLIs agree cell for cell):

| bytes | `--passphrase-stdin` | `--passphrase -` | `@env:PP` | argv literal |
|---|---|---|---|---|
| `TREZOR` | b4e3f5ed | b4e3f5ed | b4e3f5ed | b4e3f5ed |
| `TREZOR\n` | b4e3f5ed | b4e3f5ed | b4e3f5ed | 48efb44f |
| `TREZOR\r\n` | b4e3f5ed | b4e3f5ed | b4e3f5ed | 45494db3 |
| `TREZOR\n\n` | 48efb44f | 48efb44f | 48efb44f | d1f56c4d |
| `TRE\nZOR` | c6278769 | c6278769 | c6278769 | c6278769 |
| ` TREZOR ` | 9a2a7b73 | 9a2a7b73 | 9a2a7b73 | 9a2a7b73 |
| `-` | 66d564d1 | 66d564d1 | 66d564d1 | (73c5da0a: `-` now means stdin, which was empty) |

**The chosen rule:** all three private forms strip exactly one trailing `\n` or `\r\n` and keep every other byte. An argv literal stays verbatim. The only wallet-changing part is `@env:` on the toolkit, where a value ending in a newline changed (e.g. 48efb44f → b4e3f5ed). The operator ruled that no existing wallets need preserving. The CHANGELOG still records it as a behaviour change.

## Edge rules as implemented (identical in both repos)

- **`-`** means stdin, byte for byte the same as `--passphrase-stdin`: both call the same reader. Matching is on the exact value `-`, so ` -` is a literal.
- **`@env:VAR`** is a whole-value prefix; `x@env:VAR` is a literal. `VAR` must match `[A-Z_][A-Z0-9_]*`, or the result is an error. An unset `VAR` is an error naming it: toolkit `--passphrase: env-var VAR referenced by sentinel is not set`, exit 1; ms has the same text, exit 1. A set-but-empty `VAR` gives the empty passphrase, the same as omitting `--passphrase`, with no note. The value then gets the one-newline rule.
- **Literal:** the value is used verbatim and exactly one stderr line is printed, byte-identical in both CLIs: `warning: secret material on argv (--passphrase) — read it privately with --passphrase - or --passphrase-stdin (stdin), or --passphrase @env:VAR (environment variable)`. The line never contains the value, and stdout is unchanged (tested). There is no note for `-`, `@env:` or `--passphrase-stdin`. `--passphrase ""` is literal and gets the note.
- **Both stdin forms together:** `--passphrase - --passphrase-stdin` is refused by clap's `conflicts_with` (exit 64) on every site. The resolvers refuse it too, as a backstop.
- **One stdin per invocation:** every existing single-stdin guard now asks the resolver ("does the passphrase read stdin?") instead of reading `passphrase_stdin` directly. The error names the spelling the user typed.
- **ms `--allow-argv-secret`:** the guard rewrites an admitted value to a `-` placeholder. The resolver takes the admitted side channel and classifies that placeholder as argv, not stdin, so it is not counted as a stdin reader (tested). The guard no longer treats `--passphrase @env:VAR` (or `=@env:`) as material, but only on `--passphrase`: `--hex @env:X` is still refused.
- **Toolkit argv guard:** `--passphrase` gets `sentinel: true`, so `-` is a channel. `--bip38-passphrase -` is still a value (unit-tested).
- **Literal `-`:** a caller who wanted the literal passphrase `-`, or one beginning `@env:`, must now pipe it (vector: stdin `-` → 66d564d1). Both CHANGELOGs list this under "Changed" as a BREAKING behaviour change.

## Found and fixed (toolkit, pre-existing)

1. **verify-bundle keyless-template path, false NO MATCH.** The stdin passphrase was substituted into the args up front. The shared `resolve_template_completion_seed` then saw `passphrase_stdin` still true, read the drained stdin again and derived with the empty passphrase: exit 4 on a matching bundle. It also printed the argv warning for stdin and `@env:` input. Fixed with `PassphraseArg::Resolved`.
2. **xpub-search had no single-stdin guard.** `--phrase-stdin --passphrase-stdin` searched with the empty passphrase and gave a false "no match" (exit 4). It is now refused through `refuse_second_stdin`, which also covers `--ms1-stdin` and `--descriptor-from <node>=-`.
3. **Second copies of the `@env:` rule.** convert, derive-child, bundle and verify-bundle each resolved `@env:` in their own pre-pass. The first version of this change missed them: the vector test caught the newline case failing on convert. They now call `passphrase_input::resolve_env`. slip39's pre-pass no longer resolves the passphrase at all, which also removes its spurious note for `@env:`.

## Tests

- **Vectors:** `passphrase_channels.json`, 27 cases, byte-identical in both repos (`cmp` passes). Toolkit copy: `crates/mnemonic-toolkit/tests/vectors/`; ms copy: `crates/ms-cli/vectors/`. The file states the rule, the note text and, per case, the arguments, stdin, env, the override, the expected fingerprint or error, and the note count. It covers every edge rule above.
- **ms** `tests/f687_passphrase_channels.rs` (5 tests):
  - all vectors
  - the literal changes stderr only, by exactly one line
  - `-` and the flag beside an ms1 on stdin are refused
  - the override placeholder is not stdin
  - the guard admits `@env:` in both spellings, and still refuses `--hex @env:`

  Plus 4 unit tests in `passphrase_input`.
- **Toolkit** `tests/cli_f687_passphrase_channels.rs` (5 tests):
  - all vectors, through `convert`
  - all 12 subcommands: `-`, `-` plus LF, `@env:`, `@env:` plus LF, and the literal each equal the stdin reference and differ from none, with the right note count. An unset `@env:` errors naming the variable, and `-` with the flag is refused. verify-bundle asserts `result: ok`.
  - a second stdin reader is refused on all 12, for both spellings
  - template completion takes every form (`OK`, exit 0), with the none control giving exit 4
  - xpub-search two-stdin refusal

  Plus 6 unit tests in `passphrase_input`, and the argv-guard unit test was rewritten (`a_dash_passphrase_is_a_channel`).
- **Updated existing tests:** `cli_slip39_advisories.rs` (2 asserts moved to the new note text) and `lint_zeroize_discipline.rs`. The lint gains a row for `src/passphrase_input.rs`. Three xpub-search rows now take evidence `let passphrase: Zeroizing<String>`, so the partition floor of 40 still holds.

## Mutations (each applied only after asserting the edit landed exactly once; file restored afterwards)

| id | mutation | result |
|---|---|---|
| S1 | ms: `-` literal again | 4 tests red; probe shows the mutated line ran: `--passphrase -` + TREZOR → `66d564d1` |
| S2 | ms: override placeholder read as stdin | 6 red |
| S3 | ms: `@env:` keeps the newline | 2 red |
| S4 | ms: guard refuses `@env:` again | 2 red |
| S5 | ms: derive two-stdin guard ignores `-` | 1 red |
| T1 | toolkit: `-` literal again | 9 red; probe shows `fingerprint: 66d564d1` plus the argv note |
| T2 | toolkit: `@env:` keeps the newline | 3 red |
| T3 | toolkit: note also for `@env:` | 4 red |
| T4 | toolkit: template path back to as-written (the old defect) | 1 red |
| T5 | toolkit: two-stdin helper disabled | 2 red |
| T6 | toolkit: argv guard `sentinel: false` | 5 red |
| T7 | toolkit: convert `@env:` pre-pass bypasses the resolver | 2 red |
| T8 | toolkit: bundle substitution ignores `-` | 1 red |

## Gates

| gate | ms | toolkit |
|---|---|---|
| `cargo nextest run --locked --workspace` (never `--release`) | 637/637 passed, 11 skipped | 4069/4069 passed, 20 skipped |
| clippy 0.1.85 (pinned 1.85.0 bin first on PATH), `--workspace --all-targets -D warnings` | clean | clean |
| `cargo fmt --all -- --check` (pinned) | clean | clean |

Toolkit only:
- Examples golden: regenerated with `EXAMPLES_BIN_DIR=target/debug bash .examples-build/gen.sh`, and `git diff .examples-build/Examples.md` is empty.
- `docs/manual` `make audit`: passes. `MNEMONIC_BIN` was the branch build; `MD_BIN`/`MS_BIN`/`MK_BIN` came from `scripts/install.sh --no-gui --no-man --root /scratch/code/shibboleth/f687-scratch/install`. Result: lint OK, 62 transcripts pass, 9 anchor-check danglers matching baseline.

## Docs

- Help text (clap doc comments) updated on every `--passphrase` in both CLIs.
- CLI manual `41-mnemonic.md`: a new section, "How `--passphrase` is read (every subcommand)", covering the table, the one-newline rule, one stdin, and no literal `-`. The private-channels table, the three advisory-text tables and rows, and five `--passphrase` flag rows are updated.
- CLI manual `43-ms.md`: the `--passphrase` row is updated.
- CHANGELOG `[Unreleased]`:
  - ms: Changed (BREAKING behaviour).
  - toolkit: Changed (BREAKING behaviour, including the `@env:` newline) and Fixed (the template path and xpub-search).

## Seedhammer fork (Go)

Confirmed: the fork has no argv `--passphrase`. The Go CLIs under `cmd/` declare no passphrase flag, and the BIP-39 passphrase is entered on the device in `gui/` (checked at fork `0287e3a`). The fork is not edited.

## Not done, flagged for the controller

- **`--bip38-passphrase -` and `--decrypt-password -` are still literal.** Same shape as this defect, but F-687 names `--passphrase`. The toolkit resolver would take a flag name with little change.
- **`verify-bundle --ms1 -` (F-689) is left for F-689.** The resolver is passphrase-specific, so the fix was not trivial here.
- **Shared with `--passphrase-stdin`, unchanged:**
  - On a TTY, `--passphrase -` waits for input with no prompt.
  - Empty stdin (`< /dev/null`) gives the empty passphrase silently.
- **Pre-existing, outside passphrase scope:**
  - `slip39 split --from phrase=@env:X` still prints the `--from` argv warning.
  - In convert and derive-child, an `@env:`-resolved `--from` value of exactly `-` would be treated as stdin (the resolution happens before the stdin check).

## Reproduction scripts

The scripts lived under `/scratch/code/shibboleth/f687-scratch/`, which the brief says to delete, so they were deleted with it:
- `measure.py`: the per-command table
- `bytes.py`: the byte table
- `tmpl.sh`: the template path
- `mutate.py` with `muts_*.json`: the mutations

The committed tests reproduce every row.
