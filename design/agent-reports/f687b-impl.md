# F-687b implementation: passphrase rulings 1–3, F-689, F-691

Agent-written (Opus 5.5), 2026-09-25. Implements the operator's rulings of 2026-09-25 ("1) yes 2) yes 3) yes") on top of merged F-687: ms master `d1ab447`, toolkit master `9da32e2f`.

| repo | branch / worktree | commit |
|---|---|---|
| mnemonic-secret | `f687b-passphrase-rulings`, `/scratch/code/shibboleth/ms-worktrees/f687b` | `3a031cf` |
| mnemonic-toolkit | `f687b-passphrase-rulings`, `/scratch/code/shibboleth/tk-worktrees/f687b` | `7f9c400b` |

The worktrees did not exist yet; I created them from a freshly fetched `origin/master`. There are no version bumps, pushes, merges or tags.

## Secret flags enumerated from `--help`

I walked `--help` recursively on both binaries and matched `pass|password|secret|pin|key`.

**Password- or passphrase-like, and now under the rule (ruling 1):**

| CLI | flag | subcommands |
|---|---|---|
| toolkit | `--passphrase` | the 12 from F-687 |
| toolkit | `--bip38-passphrase` | `convert` |
| toolkit | `--decrypt-password` | `import-wallet`, `electrum-decrypt` |
| ms | `--passphrase` | `derive` |

**Seen and deliberately left alone:**

| flag | why it is out of scope |
|---|---|
| `--decrypt-password-file` | Takes a path. It keeps its existing rule: one trailing `\n` stripped, `\r\n` not. |
| ms `hashlock --hashlock-phrase` | Already refuses `-` with a pointer to `--hashlock-phrase-stdin`, so it fails loud. The project's own terminology keeps it distinct from a passphrase. |
| `import-wallet --bsms-encryption-token` | Takes a path or `-`, so `-` is already stdin. |
| `--secret` (`silent-payment`, `nostr`) | Key material, not a password. |
| `--final-key`, `--key`, `--recovery-key`, `--taproot-internal-key`, `--pubkey`, `--reveal-secret` | Not passwords. |

## What was built

**One resolver, parameterised.** The toolkit's `passphrase_input.rs` gains `SecretFlag` records (`PASSPHRASE`, `BIP38_PASSPHRASE`, `DECRYPT_PASSWORD`), each holding the flag, its stdin twin, a noun and a prompt. The existing functions are now thin wrappers over `*_for(&SecretFlag, …)`, so there is still one implementation.
- `convert --bip38-passphrase` goes through `reads_stdin`, `read_stdin_secret` and `resolve_env_for`, and gets the flag's own argv note.
- `electrum-decrypt` and `import-wallet --decrypt-password` go through `resolve_for` and `emit_argv_note_for`.
- The argv guard now treats `-` on both flags as a channel (`sentinel: true`), and its refusal names all three private forms.
- One stdin per invocation, which now also covers:
  - `--bip38-passphrase -` beside `--from <node>=-` or a stdin `--passphrase`;
  - `--decrypt-password -` beside `--ciphertext -`;
  - `--decrypt-password -` beside `--blob -` or `--bsms-encryption-token -`;
  - a `--decrypt-password-file` path that is stdin.

**Ruling 2: empty value.** When `-`, a `*-stdin` flag or `@env:VAR` produces an empty value, one stderr line is printed and the command proceeds. It is never refused. The text is byte-identical in both CLIs:

```text
warning: --passphrase from stdin is empty; proceeding with the EMPTY passphrase
warning: --passphrase from environment variable PP is empty; proceeding with the EMPTY passphrase
```

The other flags get their own name and noun, for example `--bip38-passphrase … EMPTY BIP-38 passphrase` and `--decrypt-password … EMPTY decryption password`. A literal `""` is not warned; it gets only the argv note. The warning is printed in the single place each value is read (`read_stdin_secret` and `resolve_env_for`), so every path, including the `@env:` pre-passes, warns exactly once.

**Ruling 3: terminal prompt.** When the value is read from stdin and stdin is a terminal:

1. The prompt goes to stderr: `Enter passphrase: ` (or `Enter BIP-38 passphrase: ` / `Enter decryption password: `).
2. Echo is switched off with `termios` `ECHO` off and `ECHONL` on, so the Enter key still moves the cursor. The settings are restored by a drop guard, including on error returns. If the terminal refuses the change, the prompt adds `(input will be visible)`.
3. **One line** is read, byte by byte, so Enter finishes the input. Before, a terminal needed Ctrl-D.

On a pipe or file there is no prompt and the bytes are unchanged. Unit tests never count as a terminal (`cfg(test)`).

What this does not do:
- A Ctrl-C at the prompt kills the process before the drop guard runs, which can leave echo off in that terminal. `stty echo` restores it. This is the same limitation `rpassword` has.
- On non-Unix builds there is no echo control, so the input is visible and the prompt says so.

**F-689: `verify-bundle --ms1 -` reads stdin.** This is the root cause, not a guess: the `-` went through the display-separator strip, became `""` (the watch-only sentinel), and a matching bundle reported `result: mismatch`. The ms1 is now substituted before that strip. Empty stdin is refused, because it would otherwise silently mean watch-only. Only one `--ms1 -` is allowed, and not beside a stdin passphrase or a stdin slot.

Measured against a matching bundle:
- installed 0.103.2 binary: `--ms1 <argv>` gives `result: ok`; `--ms1 -` gives `result: mismatch`
- branch: both give `result: ok`

**Other ms1/secret inputs checked for a literal `-` (all fail closed; none silently derives a wallet):**

| input | behaviour of `-` |
|---|---|
| `xpub-search --ms1 -` | literal: `error: ms1 string length 1 …`. `--ms1-stdin` is the channel. |
| `silent-payment --secret -` | argv-refused. With the override it is literal and errors (`expected a seed-bearing secret`). `--secret-stdin` exists. |
| `nostr --secret -` | literal, errors (`invalid bech32 nostr key`). `--secret-stdin` exists. |
| `import-wallet --ms1 -` and `--slot @N.phrase=-` | literal per the guard's measured notes; not a valid ms1 or phrase, so it errors. `@env:` is the channel. |
| `inspect --ms1 -`, `repair --ms1 -`, `ms-shares --share -`, `--digits -` | already read stdin |

Candidate follow-ups:
- Make `-` a channel on `xpub-search --ms1`, `import-wallet --ms1` and `import-wallet --slot`.
- The argv guard still does not refuse a literal `--ms1` on `verify-bundle`: its channel table maps it to `None`, the pre-F-689 "no channel exists" answer. Refusing it now would force the override on multisig verification, which needs more than one `--ms1`, so I left it and flag it.

**F-691: `--passphrase "- "` now agrees in both CLIs.**

| CLI | before | after |
|---|---|---|
| toolkit | clap refuses (exit 64) | unchanged |
| ms | literal `"- "` admitted | usage error (exit 64) |

The cause in ms was that its argv guard's flag-shape test trimmed the value. On `--passphrase` it now reads the value exactly as the parser sees it. Agreement in both CLIs:
- `--passphrase=- ` is the literal `3ca82432`.
- A leading space (`" -"`) is the literal `e20c1882`.
- Only the exact `-` is stdin.

A vector row pins the exit-64 case.

## Vectors

`passphrase_channels.json` grows from 37 to 42 cases and stays byte-identical in both repos (`cmp` passes).
- New fields: `empty_warnings` on every case, a top-level `empty_warning` template, and a `prompt`.
- The harness checks that no piped case prints a prompt.
- New rows:
  - a lone LF on stdin, a CRLF via `-`, and a lone LF via `@env:`: each gives `73c5da0a` and 1 warning
  - a literal `""`: `73c5da0a`, 1 note, 0 warnings
  - the F-691 `"- "` row: exit 64
- Every fingerprint uses the BIP-39 vector "abandon ×11 about".

## Tests (tests written first per rule; each rule mutation-checked)

- **Toolkit** (`cli_f687_passphrase_channels.rs`):
  - `bip38_passphrase_takes_every_form`: the BIP-38 test vector `6PRVWUbkzz…ZoGg` via the stdin flag, `-`, `=-`, `@env:` with an LF, and a literal. It also covers notes, the empty warning, and the two one-stdin refusals.
  - `decrypt_password_takes_every_form`: `electrum-decrypt` gives `hello world` via every form, and warns on an empty `@env:`. `import-wallet` decrypts the BIE1 fixture via `-` and `@env:`. The one-stdin refusals are covered too.
  - `verify_bundle_ms1_dash_reads_stdin`: `result: ok`; a wrong ms1 on stdin still mismatches; empty stdin is refused; a second stdin reader and a second `--ms1 -` are refused.
  - `a_terminal_gets_a_prompt_and_no_echo`: runs on a real pty (`posix_openpt`). The prompt appears; `b4e3f5ed` comes back for `-` and `--passphrase-stdin`; the terminal never displays `TREZOR`; the BIP-38 prompt is checked too.
  - The argv-guard unit test is extended.
- **ms:**
  - `a_terminal_gets_a_prompt_and_no_echo` (pty)
  - `a_dash_space_argument_is_a_usage_error_under_the_override`
  - vectors

Mutations: each was applied exactly once, run against the whole package suite, then restored.

| mutation | red |
|---|---|
| R1a toolkit: `--bip38-passphrase -` literal | 2 |
| R1b bip38 `@env:` bypasses the resolver | 1 |
| R1c guard: `--decrypt-password -` treated as material | 2 |
| R2 no empty warning (toolkit / ms) | 3 / 1 |
| R3a no prompt (toolkit / ms) | 1 / 1 (20 s prompt timeout) |
| R3b echo left on (toolkit / ms) | 1 / 1 |
| R3c terminal treated as a pipe | 1 |
| F689a `--ms1 -` not substituted | 1 |
| F689b `--ms1 -` not counted as a stdin reader | 1 |
| F691 flag-shape test trims again (ms) | 2 |

## Gates

| gate | ms | toolkit |
|---|---|---|
| `cargo nextest run --locked --workspace` | 642/642, 11 skipped | 4078/4078, 20 skipped |
| clippy 0.1.85 (pinned), `--workspace --all-targets -D warnings` | clean | clean |
| `cargo fmt --check` | clean | clean |

Toolkit only:
- Examples golden: regenerated, no diff.
- `docs/manual` `make audit`: OK, 62 transcripts, anchor-check matches baseline. MD/MS/MK came from a scratch `install.sh` root.
  - The first run failed markdownlint MD038 on the trailing space inside the `` `Enter passphrase: ` `` code spans. I reworded them without the space and re-ran; the result above is the second run.

## Docs

- Help on `--bip38-passphrase` (convert) and `--decrypt-password` (import-wallet, electrum-decrypt) now describes `-` and `@env:`.
- CLI manual `41-mnemonic.md`:
  - the private-channels table now covers both flags;
  - "How `--passphrase` is read" names the other flags and adds the empty-warning and terminal-prompt paragraphs and the exit-64 rule;
  - the flag rows for the other flags are updated.
- `43-ms.md`: the `--passphrase` row is updated.
- CHANGELOG `[Unreleased]`:
  - toolkit: Changed (BREAKING: the two flags, the empty warning, the prompt) and Fixed (F-689);
  - ms: Changed (the empty warning, the prompt) and Fixed (F-691).

The scratch directory is deleted.
