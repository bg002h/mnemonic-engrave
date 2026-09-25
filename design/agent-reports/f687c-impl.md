# F-687c implementation: the terminal prompt drains a paste or type-ahead and shows it

Agent-written (Opus 5.5), 2026-09-25. It implements the operator's ruling on N3, verbatim: *"Yes to the paste question but print what was dropped on stderr"*.

| repo | branch / worktree | commit | base |
|---|---|---|---|
| mnemonic-secret | `f687c-drain-paste`, `/scratch/code/shibboleth/ms-worktrees/f687c` | `350728f` | origin/master `f1a1792` |
| mnemonic-toolkit | `f687c-drain-paste`, `/scratch/code/shibboleth/tk-worktrees/f687c` | `8a578f38` | origin/master `30afb101` |

The worktrees were created fresh from a just-fetched `origin/master`. There are no version bumps, pushes, merges or tags.

## What it does

`drain_pending_input(noun)` lives in each CLI's `passphrase_input.rs`, next to `EchoOff`. It is called only on the **terminal path** of the shared prompt reader:
- toolkit `read_stdin_raw`, which covers every prompt: `Enter passphrase:`, `Enter BIP-38 passphrase:`, `Enter decryption password:` and `verify-bundle`'s `Enter ms1:`;
- ms `read_stdin_passphrase`.

It runs after the prompted line is read and while `EchoOff` is still alive, so echo is off and the signal handlers are armed.

1. Save the prompt's current terminal mode, then switch to non-canonical input with `VMIN = 0` and `VTIME = 1`. **`ISIG` stays on.**
2. Read with `read(0)` until a read returns 0, meaning 0.1 s passed with no new input. This catches:
   - the rest of a multi-line paste, including pieces still arriving;
   - a partial last line with no newline, which a canonical read or `FIONREAD` would not report;
   - type-ahead after Enter.

   The read is capped at 1 MiB.
3. Flush with `tcflush(0, TCIFLUSH)` so nothing is left for the shell. This is a backstop to the read loop.
4. Put the prompt's mode back. `EchoOff`'s drop then restores the original mode as before.
5. If anything was read, print it on stderr:

   ```text
   note: discarded 2 line(s) typed after the passphrase (not run, not used):
   line2
   line3
   ```

   The noun follows the prompt: `BIP-38 passphrase`, `decryption password`, `ms1`. If nothing was pending, nothing is printed. The text is the same in both CLIs.

**Pipes and files are unchanged.** The drain is never called there, and no byte beyond what the command consumes is read. A test checks this.

**Rust buffering is not an issue.** Canonical-mode `read(2)` returns at most one line, so after the prompted line the rest of a paste is still in the kernel queue, where the fd-level drain sees it. It is not in `Stdin`'s internal buffer.

**Signals.** A Ctrl-C during the drain is still an interrupt, because `ISIG` stays on. The handler armed by `EchoOff` restores the **original** mode (echo on, canonical) and re-raises. The same holds for SIGTERM, SIGHUP and SIGQUIT.

**Cost.** Each terminal prompt adds 0.1 s of waiting after Enter. There is no cost on pipes.

**Known edge.** Input typed in the few microseconds between the last read timing out and `tcflush` is discarded without being printed. That is the only way the flush can drop something the note does not show.

## Tests (real pty, both CLIs)

The pty harness now:
- reports `left_for_shell`: after the child exits, it switches its own handle on the slave to non-blocking, non-canonical mode and reads everything still queued, which is exactly what the shell would read next;
- accepts several writes, where an empty write means a 40 ms pause.

`pasted_and_typed_ahead_lines_are_drained_and_shown` covers these cases:

| case | what the test types | fingerprint | stderr note | left for the shell | echo after |
|---|---|---|---|---|---|
| paste | `TREZOR\nline2\nline3\n` in one write | b4e3f5ed | `2 line(s)`, then `line2\nline3` | nothing | on |
| type-ahead | `TREZOR\n`, then `ls -la\n` | b4e3f5ed | `1 line(s)`, then `ls -la` | nothing | on |
| partial line | `TREZOR\npartial` | b4e3f5ed | `1 line(s)`, then `partial` | nothing | on |
| nothing pending | `TREZOR\n` | b4e3f5ed | no note | nothing | on |
| pipe (control) | `TREZOR\nline2\n` piped | the whole stream is the passphrase, as before | no note | n/a | n/a |

In the paste case the terminal also never displays `line2`, because echo is off.

Toolkit only:
- The BIP-38 prompt: `1 line(s) typed after the BIP-38 passphrase`.
- `verify_bundle_ms1_prompt_drains_a_paste`: pasting `<ms1>\nrm -rf ~\n` gives `result: ok`, a note naming `rm -rf ~`, and nothing left for the shell.

`ctrl_c_during_the_drain_leaves_a_working_terminal` (both CLIs) types `TREZOR\n`, pauses 40 ms (the drain listens for 100 ms), then types `^C`. It asserts death by SIGINT, no stdout, and echo on afterwards.

**Stability:** the terminal, drain and Ctrl-C tests ran 20× in each crate, and the mid-drain Ctrl-C test another 30× in each crate. There were no failures, and all the pty tests also passed in the full parallel suite runs. No stray processes were left.

## Mutations (whole package suite each; applied exactly once; restored)

| mutation | toolkit | ms |
|---|---|---|
| no drain | 2 red | 1 red |
| drain but no print | 2 red | 1 red |
| `ISIG` off during the drain | 1 red (`ctrl_c_during_the_drain…`) | 1 red |
| no `tcflush` | survives | survives |
| prompt mode not restored after the drain | survives | survives |

Why the last two survive:
- **No `tcflush`:** the read loop already empties the queue, so the flush is a deliberate backstop with no observable effect when timing is normal.
- **Prompt mode not restored:** `EchoOff`'s drop restores the original mode immediately afterwards, so leaving the non-canonical prompt mode has no observable effect either.

In both cases the terminal ends in the right state. These two survivors do not hide a defect.

## Gates

| gate | ms | toolkit |
|---|---|---|
| `cargo nextest run --locked --workspace` | 645/645, 11 skipped | 4084/4084, 20 skipped |
| clippy 0.1.85 (pinned), `--workspace --all-targets -D warnings` | clean | clean |
| `cargo fmt --check` | clean | clean |

Toolkit only:
- Examples golden: regenerated, no diff.
- `docs/manual` `make audit`: OK, 62 transcripts, anchor-check matches baseline. MD/MS/MK came from a scratch `install.sh` root.

## Docs

- CLI manual `41-mnemonic.md`: the terminal paragraph now describes the drain, with the sample note.
- CLI manual `43-ms.md`: the `--passphrase` row mentions it.
- CHANGELOG `[Unreleased]` → Changed, in both repos. I merged the new entry under the existing `### Changed` heading rather than adding a duplicate.
- `--help` is not changed. It does not describe any terminal-only behaviour, neither the F-687b prompt and echo-off nor this. The manual is the single place that covers it.

The scratch directory is deleted.
