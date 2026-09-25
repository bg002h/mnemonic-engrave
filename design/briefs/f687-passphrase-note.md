# Brief — F-687: `--passphrase -` is stdin and `--passphrase @env:VAR` is the environment, everywhere

> **This brief replaces an earlier version** (keep literal behaviour + warn),
> which the operator stopped. Read F-687 in
> `/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md` for the
> full ruling. Confirmed with the operator ("Yes exactly"):

- **`--passphrase -`** reads the passphrase from **stdin**, in **both** CLIs
  (mnemonic-toolkit and mnemonic-secret), on **every** command that takes a
  passphrase.
- **`--passphrase @env:VAR`** reads it from the environment variable `VAR`,
  likewise everywhere.
- **Nothing is refused** for using these forms. A passphrase given literally on
  argv (`--passphrase hunter2`) keeps working exactly as today and prints one
  **stderr** note naming the safer forms (`--passphrase -` /
  `--passphrase-stdin`, `--passphrase @env:VAR`). Never echo the passphrase in
  the note.

## What exists today (measured, reproduce it)

mnemonic-gui branch `gui-followups`, `design/DESIGN_secret_channels_and_new_forms.md`
and `design/measurements/secret-channels/` hold a measured table: `--passphrase -`
is taken **literally** on every toolkit subcommand measured and on `ms derive`;
`@env:` is literal on `silent-payment` and `ms derive` but already a real
channel on other toolkit commands. Controller-measured on ms 0.19.1 with the
BIP-39 test vector "abandon ×11 about" (no funds): passphrase `TREZOR` via
`--passphrase-stdin` gives fingerprint `b4e3f5ed`; `--passphrase -` gives
`66d564d1` (the literal "-") even with `TREZOR` on stdin.

**Reuse, don't duplicate:** the toolkit already implements `@env:` on some
commands and `-`-as-stdin for other inputs; ms has `--passphrase-stdin`. Route
every passphrase flag through one shared resolver per repo. A second copy of the
rule is how the inconsistency arose.

## Edge rules to specify and test

- **One stdin per invocation.** `--passphrase -` alongside another input
  genuinely on stdin must be refused as a two-stdin conflict. That's the
  existing guard (see ms 0.19.1's fix for admitted argv values), not a refusal
  of the passphrase form.
- `@env:VAR` with `VAR` unset must be an error naming `VAR`. Treating it as an
  empty passphrase is a different wallet. With `VAR` set but empty, say what
  happens and test it.
- Trailing newline from stdin: match what `--passphrase-stdin` does today,
  byte for byte. A passphrase differing by a `\n` is a different wallet. State
  the rule.
- `--passphrase -` together with `--passphrase-stdin`: define it (probably an
  error) and test it.
- What happens to a caller who deliberately wanted the literal `-`: the
  operator says nobody does. Note the behaviour change in the CHANGELOG under
  "Changed" as a breaking behaviour change.

## Tests first (per repo, per affected command)

- `--passphrase -` with `TREZOR` on stdin gives the same fingerprint as
  `--passphrase-stdin` with `TREZOR` (`b4e3f5ed` on the vector above).
- `@env:VAR` with `VAR=TREZOR` gives the same; with `VAR` unset it errors.
- a literal argv passphrase gives the same wallet as before, plus exactly one
  stderr note, with stdout unchanged;
- no note for `-`, `@env:` or `--passphrase-stdin`.
- Mutation-check the resolver (e.g. make `-` literal again) and prove the tests
  go red and that the line ran.

Also add **test vectors** (this is normative input behaviour; Rust-primary):
vector files or golden tests other implementations can follow.

## Where

- ms: worktree `/scratch/code/shibboleth/ms-worktrees/f687`, branch
  `f687-passphrase-channels`, off `origin/master` (the worktree already exists,
  clean, at `4b02906`).
- toolkit: worktree `/scratch/code/shibboleth/tk-worktrees/f687`, branch
  `f687-passphrase-channels`, off `origin/master` (exists, clean, at `b72dbdfa`).

CHANGELOG under `[Unreleased]`; no version bumps. Don't push default branches,
merge or tag. The seedhammer fork's Go ports take passphrases on the device,
not argv; confirm in one line, don't edit the fork. Update help text and the
CLI manual wherever they describe `--passphrase`.

## Gates

Per repo: `cargo nextest run --locked --workspace` (never `--release`), pinned
clippy `-D warnings` (pinned toolchain bin first on PATH), fmt; toolkit also its
help/Examples goldens and `docs/manual` `make audit` with `*_BIN` pointed at a
scratch install (`scripts/install.sh --no-gui --no-man --root <scratch>`).
Scratch under `/scratch/code/shibboleth/f687-scratch/`; delete with `find … -delete`.

## Deliverable

Commits ending with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
**As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687-impl.md`**:
the per-command before/after table (measured), the edge rules as implemented,
tests, mutations, gates. Return a short summary plus the path.
