# F-687c review: drain and print input left over after a terminal prompt

Agent-written (Sonnet 5), 2026-09-25. Independent review of the implementer's
report (`design/agent-reports/f687c-impl.md`) against ms `f687c-drain-paste`
@ `350728f` and toolkit `f687c-drain-paste` @ `8a578f38`, diffed against
`origin/master`. All checks below were run with my own harness (a Python
`pty`-based script, not the implementer's Rust one) against binaries built
from the two worktrees in an isolated `CARGO_TARGET_DIR` under
`/scratch/code/shibboleth/review-f687c-scratch/` (deleted with `find … -delete`
at the end of the session). Neither worktree was edited, committed or pushed;
`git status --short` was clean in both at the end.

## The one question, answered directly

- **(a) Can leftover input reach the shell?** Only past the ~0.1 s window
  (measured cutoff between a 99 ms and a 105 ms gap — see "Timing
  characterization" below), and only exactly what arrives after that window.
  Everything within the window is drained; `left_for_shell` was empty in
  every within-window case tested.
- **(b) Can it be used as part of the secret?** No, in every scenario tested
  (paste, type-ahead, partial last line, BIP-38, decryption password,
  verify-bundle `ms1`) the derived result matched the passphrase/secret
  *before* the extra lines, never the concatenation.
- **(c) Can it be dropped without being printed, beyond the stated
  microsecond edge?** No new case found. Every non-empty drain in every test
  — including a 4-line paste with 80 ms gaps between lines (a sliding
  window, not a fixed budget) — was printed with the exact byte content and
  line count. See Minor 2 below for a precision point on the *documented*
  microsecond edge's failure mode, not a new gap.
- **(d) Can it leave the terminal broken?** No. `ECHO`, `ISIG` and `ICANON`
  were verified restored (all "on") after every scenario, including Ctrl-C
  mid-drain.
- **Is the non-terminal path changed?** No. A pipe carrying
  `TREZOR\nline2\n` to `ms derive --passphrase -` derives with the *whole*
  stream minus one trailing newline as the passphrase (fingerprint is NOT
  `b4e3f5ed`), and prints no note. Confirmed byte-for-byte unchanged.

## Harness

`os.openpty()`; the child's fd 0 and fd 2 are `dup2`'d onto the pty slave
(controlling terminal via `TIOCSCTTY`), fd 1 is a plain pipe so stdout is
captured cleanly and separately from the terminal stream. A **second,
independent fd is opened on the same slave device path before the fork** and
kept alive across it (never inherited/dup'd into the child); after the child
exits, that fd is switched to non-blocking non-canonical mode and drained —
this is exactly what a real shell sitting on that terminal would read next
(`left_for_shell`). `ECHO`/`ISIG`/`ICANON` are read from that fd's `termios`
after exit.

## Checks run

1. **Paste** `TREZOR\nline2\nline3\n` (ms): exit 0, fingerprint `b4e3f5ed`,
   stderr contains exactly
   `note: discarded 2 line(s) typed after the passphrase (not run, not used):\r\nline2\r\nline3\r\n`,
   `left_for_shell` empty, echo restored on. Confirmed identically on
   toolkit's `--passphrase` prompt.
2. **Slow paste**: `TREZOR\n`, then `line2\n` +50 ms, then `line3\n` +300 ms
   later. Result: note says `1 line(s)` (`line2` only); `line3` is **not**
   printed, **not** flushed, and **is** left for the shell
   (`left_for_shell == b"line3\n"`), appearing on the terminal as ordinary
   echoed type-ahead (echo is back on by the time it arrives, since the
   child has already exited and `EchoOff`'s drop ran well before 300 ms
   elapsed). This is exactly the documented design: past the window, the
   drain has already returned control and the terminal is back to normal —
   the pasted line behaves like input typed after any other command
   finishes, not a special leak.
   **Timing characterization** (`ms`, gap between `line2` and `line3`):
   | gap | captured (note) | left for shell |
   |---|---|---|
   | 50–99 ms | yes (`2 line(s)`) | empty |
   | 105–200 ms | no (`1 line(s)`, just `line2`) | `line3\n` |

   The cutoff is sharp and matches the documented `VTIME=1` (0.1 s).
   **Sliding-window check**: 4 lines, 80 ms apart (320 ms total, each gap
   under the 100 ms threshold) were *all* captured in one drain
   (`discarded 4 line(s)`) — confirms the window resets on every byte
   received rather than being a fixed total budget, as the doc comment
   claims ("the read ends 0.1 s **after input stops**").
3. **Partial last line** `TREZOR\npartial` (no trailing `\n`): note says
   `1 line(s)`, text `partial`, `left_for_shell` empty. **Ctrl-C mid-drain**:
   `TREZOR\n`, sleep 40 ms, `\x03`: process killed by `SIGINT`, no stdout,
   `ECHO`/`ISIG`/`ICANON` all restored on/on/on afterward. **Nothing
   pending**: `TREZOR\n` alone: no note, nothing left for shell, echo on.
   **A pipe**: no note, bytes unchanged (see above).
4. **Every prompt**, all via the real binaries on a pty:
   - ms `Enter passphrase: ` — verified (case 1).
   - toolkit `Enter passphrase: ` — verified (case 1, toolkit).
   - toolkit `Enter BIP-38 passphrase: ` (`convert --bip38-passphrase -`,
     typed `TestingOneTwoThree\nstray\n`): output
     `bip38: 6PRVWUbkzzsbcVac2qwfssoUJAN1Xhrg6bNk8J7Nzm5H7kxEbn2Nh2ZoGg`, note
     names "BIP-38 passphrase", `stray` shown and dropped, nothing left for
     shell.
   - toolkit `Enter decryption password: ` (`electrum-decrypt
     --decrypt-password -`, typed `test-password\nstray2\n`): decrypts to
     `hello world`, note names "decryption password", nothing left for
     shell.
   - toolkit `Enter ms1: ` (`verify-bundle --ms1 -`, typed
     `<ms1>\nrm -rf ~\n`): `result: ok`, note names "ms1" and shows
     `rm -rf ~` (not run), nothing left for shell.
5. **Two surviving mutations** — my read: **partially agree**. Both are
   correctly unobservable in every test I ran (see Minor 2 for the
   distinction in *why*).
6. **Gates**, own `CARGO_TARGET_DIR`:
   - `cargo nextest run --locked --workspace`: ms `645/645 passed, 11
     skipped`; toolkit `4084/4084 passed, 20 skipped`. Matches the
     implementer's report.
   - Pinned clippy `0.1.85` (`~/.rustup/toolchains/1.85.0-…/bin` on PATH,
     confirmed via `cargo clippy --version`), `--workspace --all-targets -D
     warnings`: clean on both.
   - `cargo fmt --check`: clean on both.
   - **20x flakiness loop**: the drain/paste/Ctrl-C pty tests (2 tests on ms,
     3 on toolkit incl. `verify_bundle_ms1_prompt_drains_a_paste`), run 20
     times each via `cargo nextest run -E 'test(...)'`. **0 failures** across
     40 (ms) and 60 (toolkit) individual test executions.

## Findings

**Critical: none. Important: none.**

### Minor 1 — the drain visibly echoes a blank line per pending newline (undocumented)

Reproduction: paste `TREZOR\nline2\nline3\n` to either CLI's `--passphrase -`
on a real pty. The terminal stream is
`Enter passphrase: \r\n\r\n\r\nnote: discarded 2 line(s)...` — one `\r\n` for
Enter's own echo, plus **one extra blank `\r\n` per newline byte drained**
(confirmed to scale: 1 pending line → 1 extra blank line; 3 pending lines →
3 extra blank lines). Cause: `drain_pending_input`'s raw-mode `termios` copy
(`raw = prompt_mode; raw.c_lflag &= !ICANON;`) keeps `ECHONL` set (inherited
from `EchoOff`, which sets it so Enter is visible with `ECHO` off), and the
kernel's `n_tty` line discipline echoes `\n` whenever `ECHONL` is set,
independent of `ICANON`. The pasted/typed-ahead *characters* are never shown
(`ECHO` is off) — only a blank line flashes per newline, immediately before
the note prints the same count in text. Not a security issue (it reveals no
more than the note's own line count, and reveals it only moments earlier);
not blocking. It is a real, reproducible visual artifact that isn't
mentioned in either CHANGELOG entry or the manual's terminal paragraph, so
readers may be confused by blank lines appearing at the prompt. Worth a doc
note or a follow-up to clear `ECHONL` in `raw` before draining, at leisure.

### Minor 2 — the two survivors are unobservable for different reasons; "no tcflush" deserves a sharper caveat

The report calls both mutations "no observable effect." I agree fully for
**"prompt mode not restored"**: this is unobservable *structurally*, not
just untested — the armed signal handler (`restore_and_reraise`) restores
the *original* globally-saved `termios` (the static `SAVED`, set once by
`EchoOff::new()`) unconditionally on `SIGINT`/`SIGTERM`/`SIGHUP`/`SIGQUIT`,
never the local `prompt_mode` variable that `drain_pending_input` would
(or wouldn't) restore. A signal in the gap between drain-return and
`EchoOff::drop()` is handled identically either way, and no code path
re-enters a terminal read using `prompt_mode` afterward (the single-stdin
guard forecloses a second terminal prompt in the same run). This is a
provably-inert mutation, confirmed also empirically (identical output,
`echo_after`/`icanon_after` both true, under normal and Ctrl-C-mid-drain
timing on a mutated build).

For **"no tcflush,"** I'd put it more precisely: it is unobservable *in
testing* (confirmed — a mutated build behaved identically to the original at
every gap from 0–200 ms, including the 99↔105 ms boundary), but it is not
inert in the exact scenario the design doc already calls out as a known
edge. The report's own docs say bytes arriving between the read loop's final
0-byte timeout and the `tcflush` call are "discarded without being printed."
That characterization is only true *because* `tcflush` is there: without it,
that same race-window data is not discarded — it is left queued, and
becomes ordinary canonical-mode input for the shell once the prompt's mode
is restored (i.e. it converts a *silently-dropped* byte into a
*silently-forwarded-to-the-shell* byte). Both are inside the documented
microsecond edge and neither is reachable by any black-box test I could
construct, so this changes no assessment of the **shipped** code (which
does call `tcflush`) — but conflating the two survivors' reasons risks the
wrong takeaway if `tcflush` is ever "cleaned up" as redundant later, since
removing it would quietly change that edge's failure mode from swallow to
leak. Not blocking; worth a one-line doc clarification if the code is
touched again.

## ready to ship: yes
