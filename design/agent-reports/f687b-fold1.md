# F-687b fold 1: response to `f687b-review.md` (0C/0I/5M/3N)

Agent-written (Opus 5.5), 2026-09-25.

| repo | branch | fold commit | on top of |
|---|---|---|---|
| mnemonic-secret | `f687b-passphrase-rulings` | `9d744e5` | `3a031cf` |
| mnemonic-toolkit | `f687b-passphrase-rulings` | `98423e68` | `7f9c400b` |

Everything the controller asked for is folded. N3 is reported below, not implemented. There are no pushes, merges, tags or version bumps.

## Response to each item

**M1: `verify-bundle --ms1 -` could reach the watch-only `""`.** Emptiness is now judged on `strip_display_separators(read)`. Stdin of `-\n`, `---` or ` - - ` is refused with `--ms1 -: stdin was empty or only separators …`, and the watch-only notice never appears. Test: `fold1_stdin_gaps_are_refused`.

**M2: `import-wallet --blob /dev/stdin`.** The blob side of the stdin guard now also asks `path_is_stdin(blob)`. With `--decrypt-password -` or `--decrypt-password-stdin` it is refused as two stdin readers. Test: same function as M1.

**M3: the `--ms1 -` guard.** It now lists `--bundle-json /dev/stdin` and `--descriptor-file /dev/stdin`. Test: same function.

**Ctrl-C.** While `EchoOff` is live (both crates):
1. Handlers for SIGINT, SIGTERM, SIGHUP and SIGQUIT restore the saved termios.
2. Each handler then resets its signal to `SIG_DFL` and re-raises it, so the process exits by the signal: the conventional status, 130 in a shell for Ctrl-C.

How it is safe:
- The handler calls only async-signal-safe functions (`tcsetattr`, `signal`, `raise`). It reads the saved mode from a static that is written before an atomic flag is set.
- Handlers are armed before echo is switched off.
- Drop restores the mode first, then disarms and puts the previous actions back.
- A signal the process inherited as ignored (`nohup`, a background job) is left ignored.

Ctrl-Z is not handled; the terminal stays `-echo` while the job is stopped, as the review measured.

New pty test `ctrl_c_and_ctrl_d_at_the_prompt_leave_a_working_terminal`, in both crates:
- It types `TRE^C` and asserts the child died of SIGINT with no stdout, and that the terminal's mode, read back, has ECHO on.
- It types `^D` at an empty prompt and asserts `73c5da0a` and that the warning starts on its own line (N1).

**M4: echo restored.** The pty harness reads the terminal mode back through its own handle on the slave after the child is gone, and every pty test asserts ECHO is on. The reviewer's mutation (`EchoOff::drop` never restores) now fails:
- toolkit: 3 tests red;
- ms: 2 tests red.

**M5: harness hygiene.**
- On a prompt timeout the harness now calls `child.kill()` and `wait()` before panicking.
- The pty is now the child's controlling terminal (`setsid` + `TIOCSCTTY` in `pre_exec`), so a typed ^C really sends SIGINT.
- After the full runs and all mutation runs, including the 20 s timeout mutants, `pgrep` found no leftover `mnemonic` or `ms` processes.

**N1.** On the terminal path, if the input did not end in a newline (Ctrl-D), one `\n` is written to stderr before anything else. The warning then reads `Enter passphrase: ⏎warning: --passphrase from stdin is empty; …`.

**N2.** For `--passphrase`, ms's flag-shape refusal now shows the value as typed: `was given "- "`. Other flags keep the trimmed value.

**N3: leftover pasted lines (report only, not done).** Draining them is cheap: one `libc::tcflush(0, libc::TCIFLUSH)` after the line is read on the terminal path, inside `EchoOff`'s lifetime.
- Benefit: the rest of a multi-line paste would not be run by the shell afterwards. That matters when someone pastes a seed phrase by mistake, because it could otherwise land in shell history as a command.
- Risk: it also throws away anything the user typed ahead on purpose. That is a behaviour change.

I left it for a decision.

**Follow-up: `verify-bundle --ms1 -` on a terminal.** This was a small step with the same code, so it is done. I split the toolkit reader into `read_stdin_raw(&SecretFlag, stdin)` (prompt, echo off, one line, the Ctrl-D newline, one trailing newline stripped) and `read_stdin_secret`, which adds the empty warning. `--ms1 -` uses `read_stdin_raw(&MS1, …)` with prompt `Enter ms1: `, then applies its own refuse-on-empty rule. It does not get the passphrase empty-warning. Pty test `verify_bundle_ms1_dash_prompts_on_a_terminal` checks:
- the result is `result: ok`;
- the terminal does not display the ms1;
- echo is on afterwards.

## Mutations

Each mutation was applied exactly once to the working tree, run against the whole package suite, then restored.

| mutation | toolkit red | ms red |
|---|---|---|
| `EchoOff::drop` never restores (reviewer's) | 3 | 2 |
| no signal handler installed | 1 | 1 |
| handler does not restore the mode | 1 | 1 |
| no newline after Ctrl-D (N1) | 1 | 1 |
| M1: emptiness judged before the strip | 1 | — |
| M2: blob path not counted as a reader | 1 | — |
| M3: descriptor-file dropped from the `--ms1` guard | 1 | — |
| `Enter ms1: ` prompt lost | 1 (by timeout, child killed) | — |
| N2: message shows the trimmed value | — | 1 |

## Gates

| gate | ms | toolkit |
|---|---|---|
| `cargo nextest run --locked --workspace` | 643/643, 11 skipped | 4081/4081, 20 skipped |
| clippy 0.1.85 (pinned), `--workspace --all-targets -D warnings` | clean | clean |
| `cargo fmt --check` | clean | clean |

Toolkit only:
- Examples golden: regenerated, no diff.
- `docs/manual` `make audit`: OK, 62 transcripts, anchor-check matches baseline. MD/MS/MK came from a scratch `install.sh` root.

## Docs

- Toolkit `--ms1` help on `verify-bundle` now describes `-`.
- `41-mnemonic.md`: the terminal paragraph covers Ctrl-C and the other signals, and `--ms1 -`; the `verify-bundle --ms1` flag row is updated.
- CHANGELOG `[Unreleased]` in both repos covers the signal handling and the Ctrl-D newline. The toolkit entry also covers the M1 separators-only refusal, the M2 and M3 guards and the ms1 prompt; the ms entry adds N2.

The scratch directory is deleted.
