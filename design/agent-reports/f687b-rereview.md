# F-687b re-review: fold 1 (ms `3a031cf..9d744e5`, toolkit `7f9c400b..98423e68`)

Agent-written (Sonnet 5), 2026-09-25. Brief: `design/briefs/f687b-rereview.md`.

**The question:** did fold 1 fix the review's findings, and is the new signal
handler correct? **Answer: yes to both.** Every finding from `f687b-review.md`
is fixed and verified independently against the built binaries and under a
real pty; the signal handler is async-signal-safe, correctly scoped, and its
new tests are live (confirmed by re-running 3 of the 13 mutations against
freshly built `git archive` copies of the fold). No new Critical, Important,
Minor or Nit findings.

## Method

Both worktrees are clean at the stated fold SHAs (`git status --short` empty,
`HEAD` matches). Built with an owned `CARGO_TARGET_DIR` under
`/scratch/code/shibboleth/review-f687b3-scratch/` (deleted at the end via
`find … -delete`, per the brief). Mutation testing ran in `git archive HEAD`
copies so the branches were never edited; nothing was committed, pushed or
merged.

**Papercut hit and worked around:** `rustup run 1.85.0 cargo clippy` silently
ran the system clippy 0.1.98 (4 spurious `div_ceil`/lifetime-elision errors on
`ms-codec`, a lint set that doesn't exist in 0.1.85) instead of the pinned
1.85.0 clippy — this is the already-logged
`local-clippy-is-not-the-pinned-clippy` papercut. Fixed by prepending
`~/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin` to `PATH` directly;
re-run was clean. No new papercuts-log entry needed (already recorded).

## 1. The signal handler

Read both crates' `passphrase_input.rs` (byte-identical `EchoOff` /
`echo_signal` modules). Findings, each checked against source AND behavior:

- **Async-signal-safe only.** `restore_and_reraise` calls `libc::tcsetattr`,
  `libc::signal`, `libc::raise` and an `AtomicBool::swap` — nothing else. All
  POSIX async-signal-safe.
- **Saved state never torn.** `SAVED` is a `static mut MaybeUninit<termios>`,
  written once per `arm()` call (one prompt at a time, documented safety
  precondition) *before* `ACTIVE.store(true, SeqCst)`; the handler only reads
  it after `ACTIVE.swap(false, SeqCst)` returns `true`, so the write
  happens-before the read under `SeqCst`.
- **Uninstalled after echo restored, not left dangling.** `Drop for EchoOff`
  restores termios first, then calls `echo_signal::disarm`, which puts the
  *previous* `sigaction` back and clears `ACTIVE`. A later signal after drop
  gets the pre-existing disposition, not a stale "restore".
- **Installed only on the terminal-prompt path.** `EchoOff::new()` is called
  only inside `if stdin_is_terminal() { … }` in both crates (`read_stdin_raw`
  / `read_stdin_passphrase`). Confirmed behaviorally: a pipe run shows no
  `Enter …` prompt and exits normally — no handler is ever armed.
- **Inherited-ignored signals stay ignored — verified experimentally, not
  just read.** Spawned `ms derive` under a pty with `SIGINT` pre-set to
  `SIG_IGN` (survives `execv`, unlike an installed handler). Sent `SIGINT`
  directly to the process: it stayed alive (`os.kill(pid, 0)` succeeded after
  0.5s), and after completing normally the terminal still restored (`ECHO
  ON`). Matches `arm()`'s `if old[i].sa_sigaction == SIG_IGN { continue; }`.

**Real-pty behavior, verified independently of the implementer's own pty test
harness** (`python pty.fork`, `ms derive --in <card> --passphrase -`):

| case | result |
|---|---|
| Ctrl-C mid-typing (`TRE^C`) | died by `SIGINT`, ECHO ON after |
| SIGTERM from another process | died by `SIGTERM`, ECHO ON after (**not** covered by the implementer's own test, which only exercises Ctrl-C — confirms the handler generalizes to the other 3 armed signals as claimed) |
| successful run (`TREZOR⏎`) | exit 0, fingerprint `b4e3f5ed`, ECHO ON after |
| pipe (no terminal) | exit 0, no prompt on stderr, correct output — no handler involved |
| Ctrl-D at empty prompt | `Enter passphrase: \r\nwarning: --passphrase from stdin is empty…`, fingerprint `73c5da0a` — N1 confirmed: warning starts on its own line |

**Ctrl-Z:** could not be reproduced as a literal job-control stop in this
sandbox — verified independently that `SIGTSTP`, sent either via the tty's
`VSUSP` byte or directly with `kill()`, does not suspend *any* session-leader
child here (reproduced the same non-stop with a plain `setsid()`'d
`sleep`/`cat`, no ms/mnemonic involved), while a directly-sent `SIGSTOP` does
stop a non-session-leader child — this is an environment/sandbox property,
not a property of the reviewed code. Falling back to source: `SIGNALS =
[SIGINT, SIGTERM, SIGHUP, SIGQUIT]` deliberately excludes `SIGTSTP`, so Ctrl-Z
is unhandled by construction and the terminal is left `-echo` while stopped —
exactly what the original review measured and explicitly accepted as
non-gating ("not needed to ship"). No new finding here.

## 2. M1–M5, N1, N2: spot-checked against the binaries — all fixed

| finding | status | evidence |
|---|---|---|
| **M1** watch-only `""` reachable from `verify-bundle --ms1 -` | **fixed** | `printf -- '-\n'`, `'---'`, `' - - \n'` on stdin each give `error: --ms1 -: stdin was empty or only separators…` (exit 1); the watch-only notice never appears. Source: emptiness now judged on `strip_display_separators(&read)` (`verify_bundle.rs:367`). |
| **M2** `import-wallet --blob /dev/stdin` + `--decrypt-password -`/`-stdin` not refused | **fixed** | `error: --blob=- and --decrypt-password[-stdin] cannot both read from stdin` for both spellings. Source: `blob_p.as_os_str() == "-" \|\| path_is_stdin(blob_p)` (`import_wallet.rs`). |
| **M3** `verify-bundle --ms1 -` + `--descriptor-file /dev/stdin` skips the guard | **fixed** | `error: --ms1 - and --descriptor-file /dev/stdin all read stdin; only one input can come from stdin per invocation`. Source: two new `path_is_stdin` entries added to `refuse_second_stdin`'s call in `verify_bundle.rs`. |
| **M4** no test pins "echo restored" | **fixed** | Every pty test in both crates now asserts `ECHO` is on afterward, read back through the harness's own slave handle. Verified live (see §3). |
| **M5** pty harness leaks child on prompt timeout | **fixed** | Source shows `child.kill(); child.wait();` before the panic in both crates' `run_on_a_terminal`. Ran both crates' full pty test files (ms 11/11, toolkit 15/15, all green) and diffed `ps`/`pgrep` before and after: zero orphaned `ms`/`mnemonic` processes. |
| **N1** Ctrl-D warning lands on the prompt line | **fixed** | Confirmed above: `Enter passphrase: \r\nwarning: …` — warning on its own line. |
| **N2** ms's flag-shape message shows the trimmed value | **fixed** | `--passphrase "- "` under the override now errors `was given "- "` (as typed), not `"-"`. Confirmed by source diff and the passing `a_dash_space_argument_is_a_usage_error_under_the_override` test. |
| **N3** leftover pasted lines | **not fixed — reported only, by design.** Fold explicitly left this for an operator decision (draining vs. not draining ahead-typed input is a behavior trade-off); this matches the fold report and is not gating (it was a Nit in the original review). |

## 3. The new `verify-bundle --ms1 -` terminal prompt

Independently built a fresh single-sig bip84 bundle, then ran
`verify-bundle --ms1 -` under a real pty (not the implementer's test, a
separate script), typing the ms1 at the prompt:

- exit 0, `result: ok` (plus every intermediate row `ok`).
- The ms1 body was never present anywhere in what the pty displayed
  (`ms1[5:].encode() in full` → `False`).
- `ECHO` back on afterward.

Matches the fold's `verify_bundle_ms1_dash_prompts_on_a_terminal` test exactly.

## 4. Mutations: 3 of 13 re-run, all match the fold's claimed counts

Run against `git archive` copies of the fold commit (not the live worktree,
so the base builds running concurrently were never disturbed), full test file
per run, mutation applied once and never persisted:

| mutation | fold's claimed red | measured red |
|---|---|---|
| `EchoOff::drop` never restores (toolkit, reviewer's original) | 3 | **3**: `a_terminal_gets_a_prompt_and_no_echo`, `ctrl_c_and_ctrl_d_at_the_prompt_leave_a_working_terminal`, `verify_bundle_ms1_dash_prompts_on_a_terminal` |
| no signal handler installed (toolkit) | 1 | **1**: `ctrl_c_and_ctrl_d_at_the_prompt_leave_a_working_terminal` (`Ctrl-C left echo OFF`) |
| M1: emptiness judged before the strip (toolkit) | 1 | **1**: `fold1_stdin_gaps_are_refused` |

All three reproduce exactly as claimed. Combined with the independently-run
full pty suites (§2, M5), this confirms the fold's tests are live, not
decorative.

## 5. Gates

| gate | ms | toolkit |
|---|---|---|
| `cargo nextest run --locked --workspace` | **643/643 passed, 11 skipped** (matches fold report) | **4081/4081 passed, 20 skipped** (matches fold report) |
| clippy 0.1.85 (pinned, toolchain bin on `PATH`), `--workspace --all-targets -D warnings` | **clean** | **clean** |
| `cargo fmt --check` | **clean** | **clean** |
| `crates/ms-cli/vectors/passphrase_channels.json` vs. `crates/mnemonic-toolkit/tests/vectors/passphrase_channels.json` | `cmp` **byte-identical**, 619 lines | same file |

Both worktrees confirmed unmodified and clean at their fold SHAs throughout
(`git status --short` empty before and after). No pushes, merges or edits to
either branch. Scratch build directory deleted.

## Findings

**Fixed:** M1, M2, M3, M4, M5, N1, N2 — all verified independently against
built binaries and/or under a real pty, not just re-read from the fold
report.

**Not fixed (by design, non-gating):** N3 — reported only, left for an
operator decision, as the fold states.

**NEW findings:** 0 Critical, 0 Important, 0 Minor, 0 Nit.

ready to ship: yes
