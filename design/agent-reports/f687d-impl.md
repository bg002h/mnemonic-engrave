# F-687d implementation: masked preview in the discarded-input note

Agent-written (Opus 5.5), 2026-09-25. This implements the operator's amendment to the drain ruling, verbatim: *"Agree with masked echo as you suggested"*.

| repo | branch / worktree | commit | base |
|---|---|---|---|
| mnemonic-secret | `f687d-masked-drain`, `/scratch/code/shibboleth/ms-worktrees/f687d` | `dc285d3` | origin/master `b3ea04a` |
| mnemonic-toolkit | `f687d-masked-drain`, `/scratch/code/shibboleth/tk-worktrees/f687d` | `22817c40` | origin/master `f40212e2` |

I created both worktrees fresh from a just-fetched `origin/master`. There are no version bumps, pushes, merges or tags.

## Format

One formatter per repo, `passphrase_input::drain_note(noun, bytes)`. The drain calls it in place of printing the raw text, so every prompt uses it:
- toolkit: passphrase, BIP-38 passphrase, decryption password, and `verify-bundle`'s `Enter ms1:`;
- ms: `Enter passphrase:`.

The header is unchanged. Each discarded line then becomes one line of its own, indented two spaces:

```text
note: discarded 2 line(s) typed after the passphrase (not run, not used):
  abandon … (12 words, 93 chars)
  ls -la
```

| discarded line | shown as |
|---|---|
| empty, or whitespace only | `(blank)` |
| at most 8 characters | the line itself |
| longer | its first 8 characters, then `…`, then `(W word(s), N chars)` |

- Characters are Unicode scalar values, not bytes: Cyrillic `пароль-секрет` is 13 chars and shows as `пароль-с… (1 word, 13 chars)`.
- W counts whitespace-separated words, and N counts the characters of the whole line.
- "word" is singular for 1.
- **Neutralising.** In what is shown, each of these becomes a single `?`:
  - a control character (tab, BEL, CR, DEL, C1);
  - an entire escape sequence: CSI `ESC [ … <@..~>`, OSC `ESC ] … BEL/ESC \`, or any other `ESC x`;
  - an invisible reordering or formatting mark: U+200B–200F, U+202A–202E, U+2066–2069, U+FEFF.

  Examples: `\x1b[2J` → `?`, and an OSC window-title sequence → `?`. No raw ESC byte reaches stderr, and the pty test asserts it.
- **Trade-off:** an emoji joined with ZWJ shows its joiner as `?`. That is cosmetic and on the safe side.

The counts come from the original line, so they stay exact. Only the 8-character head is neutralised.

## Golden (byte-identical across repos)

The golden file is `drain_preview.json`: `crates/mnemonic-toolkit/tests/vectors/` in the toolkit and `crates/ms-cli/vectors/` in ms. `cmp` confirms the two copies are identical. It holds 15 cases, each with `why`, `noun`, `input` and the exact `note`. They cover:
- the 12-word seed plus `ls -la`, and 8 versus 9 characters;
- blank and empty lines;
- CSI alone, CSI at the start of a long line, OSC, tab/BEL, and a bidi override;
- Cyrillic, CJK, and accented text of exactly 8 characters;
- a partial last line;
- the BIP-38 and ms1 nouns.

I wrote the expected notes with an **independent Python implementation** of the rule. The Rust formatter in each repo matches them exactly (`drain_note_matches_the_shared_golden`, a unit test in both crates).

## Tests (real pty)

`pasted_and_typed_ahead_lines_are_drained_and_shown` gains three cases in both CLIs:

| case | stderr shows | also asserted |
|---|---|---|
| a pasted 12-word seed after `TREZOR` | `  abandon … (12 words, 93 chars)` | stderr never contains `about` |
| `\x1b[2J` | `  ?` | no raw ESC byte in stderr |
| Cyrillic text | `  пароль-с… (1 word, 13 chars)` | — |

In every case the fingerprint is still `b4e3f5ed`, nothing is left for the shell, and echo is on afterwards.

The earlier short-line cases (`line2`/`line3`, `ls -la`, `partial`) now expect the two-space indent. The `verify-bundle` ms1 case's second line changed to `echo pwn`: the old test string contained a destructive-command pattern that this machine's command hook blocks when it appears in a shell command line.

## Mutations (whole package suite each; applied once; restored)

| mutation | toolkit red | ms red |
|---|---|---|
| full text printed (no masking) | 2 (golden + pty) | 2 |
| no control-character or escape masking | 2 | 2 |
| bytes counted instead of characters | 2 | 2 |
| CSI not swallowed whole | 2 | 2 |
| no `(blank)` rule | 1 (golden) | 1 |

## Gates

| gate | ms | toolkit |
|---|---|---|
| `cargo nextest run --locked --workspace` | 646/646, 11 skipped | 4085/4085, 20 skipped |
| clippy 0.1.85 (pinned), `--workspace --all-targets -D warnings` | clean | clean |
| `cargo fmt --check` | clean | clean |

Toolkit only:
- Examples golden: regenerated, no diff.
- `docs/manual` `make audit`: OK, 62 transcripts, anchor-check matches baseline.

## Docs

- **CLI manual `41-mnemonic.md`:** the drain paragraph now describes the masked preview, with the seed example.
- **`43-ms.md`:** the `--passphrase` row is updated.
- **CHANGELOG `[Unreleased]` → Changed, in both repos:** a new F-687d entry. The unreleased F-687c entry now says "followed by a masked preview of each line (F-687d)", so the two entries don't contradict each other.
- **`--help`:** unchanged. It does not describe terminal-only behaviour.

The scratch directory is deleted.
