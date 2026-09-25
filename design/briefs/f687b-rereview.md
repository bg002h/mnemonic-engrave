# Brief — re-review of F-687b fold 1

**One question:** did fold 1 (ms `3a031cf..9d744e5`, toolkit `7f9c400b..98423e68`,
branch `f687b-passphrase-rulings`) fix the review's findings, and is the new
signal handler correct? Not a fresh audit.

## Inputs

- Worktrees `/scratch/code/shibboleth/ms-worktrees/f687b` and `/scratch/code/shibboleth/tk-worktrees/f687b`.
- Review: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687b-review.md`.
- Fold report: `.../design/agent-reports/f687b-fold1.md`.

## Check, with evidence

1. **The signal handler** (new: restores the terminal mode on
   SIGINT/TERM/HUP/QUIT while echo is off, then re-raises with the default
   action):
   - Read it. Does it use only async-signal-safe calls? Is the saved terminal
     state written before the handler is installed and never torn (e.g. a
     static `termios` set once)? Are the handlers uninstalled (or made inert)
     after echo is restored, so a later signal doesn't "restore" a stale mode?
   - Is it installed only on the terminal-prompt path, and a no-op otherwise?
     A signal inherited as ignored stays ignored (the report claims this).
   - Under a real pty: Ctrl-C mid-typing gives exit by SIGINT with echo on;
     SIGTERM from another process does the same; a successful run leaves echo on;
     and a pipe (no terminal) involves no handler at all.
   - Say what Ctrl-Z does (the report says echo stays off while stopped) and
     whether that is acceptable.
2. **M1–M5, N1, N2:** spot-check each against the binaries.
3. **The new `verify-bundle --ms1 -` terminal prompt:** `result: ok`, the ms1
   never shown, echo restored.
4. Re-run 3 of the 13 mutations; the pty tests leave no orphaned processes (ps before/after).
5. Gates: nextest both, pinned clippy, fmt; the vector files still byte-identical.

Own `CARGO_TARGET_DIR` under `/scratch/code/shibboleth/review-f687b3-scratch/`;
delete with `find … -delete`. Don't commit, push or edit the branches.

## Output

Per finding: fixed / not fixed. NEW findings Critical / Important / Minor /
Nit. End with `ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687b-rereview.md`**
and return only a short summary plus that path.
