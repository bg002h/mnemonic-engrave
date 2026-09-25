# Brief — F-687: stderr note when `--passphrase` is `-` or `@env:…` taken literally

Read F-687 in `/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md`,
including the operator ruling:

> "Don't refuse --passphrase - or env but print message to stderr how to
> accomplish same goal more securely. Sometimes this software will run on a
> secure offline computer."

## What to build

In **mnemonic-secret** and **mnemonic-toolkit** (Rust-primary; these are the
primary repos):

- Wherever a passphrase flag's value is **used as the literal passphrase**
  while being `-` or starting with `@env:`, keep that behaviour exactly (the
  same derived wallet, the same exit code), and print one note on **stderr**:
  - that the value was used as the literal passphrase (quote it: `"-"`, or
    `"@env:NAME"`), not read from stdin or the environment;
  - how to do it more securely: `--passphrase-stdin`, or whatever private
    channel that command really accepts.
- **Only where the value really is literal.** The measured table (mnemonic-gui
  branch `gui-followups`, `design/DESIGN_secret_channels_and_new_forms.md` and
  `design/measurements/secret-channels/`) says `--passphrase -` is literal on
  every toolkit subcommand measured, and on `ms derive`; `@env:` is literal on
  `silent-payment` and `ms derive`, but a real channel elsewhere. Re-measure
  against your builds. A note printed where the value was actually honoured as
  a channel would be a false statement.
- Never print the passphrase itself beyond the literal token the user typed.
- stdout is unchanged (scripts parse it).

## Tests first

For each affected command:
- the note appears on stderr for `-` and, where literal, for `@env:X`;
- stdout and exit code are byte-identical with and without the note path;
- the derived fingerprint still equals the literal-passphrase wallet;
- no note for an ordinary passphrase, for `--passphrase-stdin`, or where
  `@env:` is a real channel.

Mutation-check: remove the note and prove a test goes red.

Use the BIP-39 test vector "abandon ×11 about" (no funds). Measured on
ms 0.19.1: `--passphrase -` gives fingerprint `66d564d1`, passphrase `TREZOR`
gives `b4e3f5ed`.

## Where

- ms: worktree `/scratch/code/shibboleth/ms-worktrees/f687`, branch `f687-note`
  off `origin/master`.
- toolkit: worktree `/scratch/code/shibboleth/tk-worktrees/f687`, branch
  `f687-note` off `origin/master`.

CHANGELOG entries under `[Unreleased]`; no version bumps. Don't push default
branches, merge or tag. The seedhammer fork's Go ports take passphrases on the
device, not argv; confirm that and say so in one line, but don't edit the fork.

## Gates

Per repo: `cargo nextest run --locked --workspace` (never `--release`), pinned
clippy `-D warnings` (pinned toolchain bin first on PATH), fmt; toolkit also
its help/Examples goldens and `docs/manual` `make audit`, with `*_BIN` pointing
at a scratch install from `scripts/install.sh --no-gui --no-man --root <scratch>`.
Scratch under `/scratch/code/shibboleth/f687-scratch/`; delete with `find … -delete`.

## Deliverable

Commits ending with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
**As your final action, write your report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687-impl.md`**,
including the exact note text per command and the measured table you acted on.
Return a short summary plus the path.
