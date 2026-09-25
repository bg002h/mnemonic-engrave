# F-687b adversarial review: ms `3a031cf`, toolkit `7f9c400b`

Agent-written (Opus 5.5), 2026-09-25. Brief: `design/briefs/f687b-review.md`.

**The question:** can any secret input these branches touch derive a wallet other than the one the user's bytes specify, silently use an empty or wrong value, leak the typed secret to the screen, or leave the terminal broken?

**Answer: no, on every path that produces a result.** Every output I compared matched a published vector across every channel. The empty value is always announced. The prompt hides input, and echo comes back on every exit I measured except a signal. Separately, the F-689 guarantee is incomplete (the watch-only `""` can still be reached from `--ms1 -`), but it fails closed. There are three test and hygiene gaps.

**Counts:** 0 Critical, 0 Important, 5 Minor, 3 Nit.

## Method

- **Binaries:** built from both worktrees with my own `CARGO_TARGET_DIR`. Mutations ran in `git archive HEAD` copies, so the branches were never edited. Both worktrees are clean at the SHAs above, and the scratch directory is deleted.
- **Channels:** every by-effect run fed the SAME secret through `-`, `=-`, the `*-stdin` flag, `@env:` (with and without a trailing LF), a literal under `--allow-argv-secret`, and, where one exists, `--decrypt-password-file`.
- **Terminal:** all terminal checks ran on a real pty (python `pty.fork`, which gives a controlling terminal, so Ctrl-C really sends SIGINT). I also ran them under interactive bash, zsh, fish and `/bin/sh`.

## 1. Rulings 1–2 by effect: all match

**BIP-38 (`convert`), against the BIP-38 spec's published vectors:**

| vector | direction | channels | result |
|---|---|---|---|
| 1 (`6PRVWUbk…ZoGg`, `TestingOneTwoThree`) | bip38→wif | stdin flag, `-` (LF), `=-` (no LF), `-` (CRLF), `@env:`, `@env:`+LF, literal, `--passphrase -` fallback, `--passphrase @env:` fallback | all `5KN7MzqK…Qi5CVR` |
| 3 (the passphrase with U+0000) | bip38→wif | `-`, `--bip38-passphrase-stdin` | `5Jajm8eQ…MSZ4` |
| 3 | wif→bip38 | `-`, `--bip38-passphrase-stdin` | `6PRW5o9F…pcDQn` |
| compressed (`6PYNKZ1E…`) | bip38→wif | `@env:`, `-` | `L44B5gGE…GVpP` |

Failure cases:
- A wrong passphrase, an empty `-`, an empty `@env:`, `@env:` holding `-` (literal, as specified) and a literal `""` all fail closed with the address-hash error.
- The empty cases also print their one warning.

**`electrum-decrypt`:** the field vector gives `hello world` via the stdin flag, `-`, `=-`, `@env:`, `@env:`+LF, a literal, a file, and `--decrypt-password-file /dev/stdin`. An empty value warns once, then fails with "wrong password".

**`import-wallet` with the BIE1 fixture:** stdout was compared by `cmp` across the stdin flag, `-`, `@env:`, a literal and `--decrypt-password-file`. All five are **byte-identical** (335 B, fingerprint `5436d724`). A wrong or empty password fails closed.

## 2. Ruling 3: the terminal, on a real pty

Results, the same in `mnemonic` and `ms` wherever both were tested:

| input at the prompt | outcome | ECHO while waiting | ECHO after |
|---|---|---|---|
| `TREZOR⏎` (`--passphrase -` and `--passphrase-stdin`) | `b4e3f5ed`; the terminal never displays `TREZOR` | off | **on** |
| BIP-38 `-`, `electrum-decrypt -` / `-stdin` | the correct key / `hello world`, each with its own prompt | off | on |
| `⏎` only | `73c5da0a`, plus the empty warning | off | on |
| Ctrl-D at an empty prompt | `73c5da0a`, plus the empty warning (ruling 2) | off | on |
| `TREZOR^D^D` (the old habit) | `b4e3f5ed` (correct) | off | on |
| Ctrl-C | killed by SIGINT | off | **off** |
| Ctrl-Z | stopped | off | off |
| paste of `TREZOR⏎secondline⏎` | `b4e3f5ed`; `secondline` is left in the tty queue | off | on |

With a pipe or file on stdin there is no prompt, and the bytes are unchanged. I checked all 42 vector cases, and the stdin/`-`/CRLF rows by hand.

### Is the Ctrl-C echo-off leak real for a user?

I measured the pty state after Ctrl-C in a real interactive shell by running `stty -a` at the next prompt:

| shell | after Ctrl-C | control: after a plain `stty -echo` |
|---|---|---|
| bash | echo | -echo |
| bash `--noediting` | echo | -echo |
| zsh `-f` | echo | echo |
| fish | echo | echo |
| `/bin/sh` | echo | -echo |

So bash and sh restore the tty specifically when a foreground job dies of a signal, while zsh and fish always restore it. **Behind a shell, the Ctrl-C leak never reaches the user.** It survives only under a non-shell parent (a script runner, an IDE task, python `subprocess`) or with Ctrl-Z.

**Cheap fix:** while `EchoOff` is live, install a `sigaction` handler for SIGINT, SIGTERM, SIGHUP and SIGQUIT. It would `tcsetattr(0, TCSANOW, &saved)` (async-signal-safe) from a static, reset the signal to `SIG_DFL` and re-`raise`. That is about 25 lines per crate, and both crates already depend on `libc`. It is not needed to ship.

## 3. F-689

Measured:
- `--ms1 -` and `--ms1=-` against a matching single-sig bundle: `result`/all rows **ok**.
- The same with a grouped (hyphenated) ms1 on stdin: ok.
- A multisig bundle: the correct ms1 on stdin gives `result: ok`, and the other cosigner's ms1 gives `mismatch`.
- Empty stdin, whitespace-only stdin and `\n\n` are refused.

**But the watch-only `""` IS reachable from `-`** (see M1).

## 4. One stdin per invocation

Refused correctly:
- **ms:** `derive -` / omitted / `--in /dev/stdin` / `--in /proc/self/fd/0`, each with `--passphrase -` or `-stdin`.
- **`electrum-decrypt`:** `--ciphertext -` with `-` or with `--decrypt-password-file /dev/stdin`.
- **`import-wallet`:** `--blob -` with `--decrypt-password-file /dev/stdin`.
- **`verify-bundle`:** `--ms1 -` twice, or beside `--passphrase -`.
- **`convert`:** `--bip38-passphrase -` beside `--from wif=-` or beside a stdin `--passphrase` (these are the implementer's tests, and they pass).

Not refused: two gaps, M2 and M3. Both fail closed.

## 5. The empty warning

Toolkit, `--passphrase` × {`@env:` empty, `-` with `\n`, `--passphrase-stdin` empty, literal `""`}, across `convert`, `bundle`, `verify-bundle`, `addresses`, `derive-child`, `silent-payment`, `restore` and `xpub-search passphrase-of-xpub`:
- Every private-channel case gives **exactly 1** warning and 0 argv notes.
- Every literal gives 0 warnings and 1 note.
- The `@env:` pre-passes do not double-warn.
- The warning goes to stderr only: stdout, including `--json`, is clean.

ms `derive`, with `-`, `=-`, `-stdin` and `@env:` empty: 1 warning each.

The ms and toolkit warning lines are **byte-identical** (`cmp`).

## 6. Vectors and mutations

`passphrase_channels.json` is `cmp`-identical across the two repos, with 42 cases (5 carry `empty_warnings: 1`).

My baselines match the report exactly: toolkit 4078/4078 (20 skipped), ms 642/642 (11 skipped).

Re-run mutations, each against the whole package, asserted to apply exactly once, then restored:

| mutation | claimed red | measured red |
|---|---|---|
| R1a `--bip38-passphrase -` literal | 2 | 2 (`bip38_passphrase_takes_every_form`, `a_terminal_gets_a_prompt_and_no_echo`) |
| F689a `--ms1 -` not substituted | 1 | 1 (`verify_bundle_ms1_dash_reads_stdin`) |
| F691 ms flag-shape test trims again | 2 | 2 (`a_dash_space_argument_is_a_usage_error_under_the_override`, `every_vector_case_holds`) |
| **NEW:** `EchoOff::drop` never restores (toolkit) | — | **0: survives** |
| **NEW:** the same in ms | — | **0: survives** |

The new mutation is semantically live. A mutant binary on a pty, after a normal successful exit, leaves **ECHO off**, and bash keeps that state after a normal exit (see the control column above). See M4.

## Findings

### Minor

**M1. The watch-only `""` is still reachable from `verify-bundle --ms1 -`.**
- **Cause:** the empty check runs on `trim()`, BEFORE the display-separator strip. A stdin made only of separators passes the check and then strips to `""`.
- **Reproduction** (single-sig or multisig bundle): `printf -- '-\n' | mnemonic verify-bundle … --ms1 - …` (also `---`, ` - - `) prints `notice: cosigner[0] marked watch-only via empty --ms1 sentinel` and then `result: mismatch`, exit 4.
- **Why Minor:** it fails closed in every configuration I tried (phrase slots, single-sig and 2-of-2). The watch-only verify with xpub slots already ignores the ms1. Still, the brief's "unreachable from `-`" and the CHANGELOG's "Empty stdin is refused" are not fully true.
- **Fix:** run the emptiness test on `strip_display_separators(read)`, and add a row with stdin `-\n`.

**M2. `import-wallet --blob /dev/stdin` with `--decrypt-password -` or `--decrypt-password-stdin` is not refused as a second stdin reader.**
- **Cause:** the guard checks `blob == "-"` only, not `path_is_stdin(blob)`.
- **Behaviour:** the blob drains stdin, so the password reads empty and the user sees `warning: … is empty` then "wrong password". That is misleading, but it fails closed.
- **History:** the stdin-flag half predates this branch; the `-` spelling is new.
- **Fix:** add `path_is_stdin` to the blob side of the hoisted guard.

**M3. `verify-bundle --ms1 -` with `--descriptor-file /dev/stdin` (or `--bundle-json /dev/stdin`) skips the one-stdin guard.**
- **Cause:** the new F-689 `refuse_second_stdin` omits the two path-is-stdin readers that the F-687 guard just above it lists.
- **Measured:**
  - With `--descriptor-file /dev/stdin`, the ms1 read drains stdin and the command then fails on the descriptor ("no keys…").
  - `--bundle-json` is saved by clap (`--ms1` conflicts with it).
- It fails closed. **Fix:** add the two entries to that call.

**M4. No test pins "echo restored after the read".**
- **Evidence:** the `EchoOff::drop` no-op mutation survives both suites, and the mutant really leaves the terminal at `-echo` after a *successful* run. Under bash, which does not restore after a normal exit, that would break the user's terminal.
- **Fix:** after `wait_with_output`, both pty tests should `tcgetattr` the master and assert `ECHO` is set, which is one assertion each. The Ctrl-C leak and its cheap fix are reported in §2; they are not gating.

**M5. The pty test harness leaks its child on a prompt timeout.**
- **Cause:** `rx.recv_timeout(…).expect(…)` panics without killing the child, which then blocks forever on the pty.
- **Evidence:** at review time three such processes from the implementer's mutation runs (09:34–09:36) were still alive: two `mnemonic convert … --passphrase -` and one `ms derive --in /tmp/.tmp… --passphrase -`. I killed them.
- **Fix:** `child.kill()` before the panic, in both crates.

### Nit

**N1.** After Ctrl-D at an empty prompt, the warning lands on the prompt's line (`Enter passphrase: warning: …`), because ECHONL echoes only a real NL. On the terminal path, emit `\n` when the line did not end in one.

**N2.** ms's flag-shape refusal message prints the *trimmed* value. For `--passphrase "- "` it reads `was given "-"`, while the user typed `"- "`. The exit code 64 and the behaviour are right.

**N3.** The terminal path reads one line. For a multi-line paste, the remaining lines stay in the tty queue, so the shell reads and runs them after exit; on a real shell they would also land in shell history. The same is true of `rpassword`. It is a documentation note at most.

### Follow-up candidates (not gating)

- **Secret handling (non-gating class, operator ruling 2026-08-27):** on a terminal, `verify-bundle --ms1 -` has no prompt and no echo-off. The typed ms1 (seed material) is echoed, and the command appears to hang until Ctrl-D.
- **Terminal UX:** with stderr redirected (`2>/dev/null`), the prompt is invisible and a terminal user sees a silent wait.

## Settled items I did not re-derive

- F-687's `--passphrase` resolver: re-checked only where these branches wrap it (`resolve_for`, `read_stdin_secret`, `resolve_env_for`).
- F-691 agreement between the CLIs, measured:
  - `--passphrase=- ` gives `3ca82432` and `" -"` gives `e20c1882`; `=-\t` gives `ba32f96f` and `=-\n` gives `d1b2c83b`, identical in both CLIs.
  - The spaced `"- "` form is exit 64 in both.

ready to ship: yes
