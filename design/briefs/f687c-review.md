# Brief — review of F-687c: drain and print input left over after a terminal prompt

**One question:** after a terminal prompt, can leftover input ever (a) reach the
shell, (b) be used as part of the secret, (c) be dropped without being
printed (beyond the stated microsecond edge), or (d) leave the terminal broken?
And is anything changed on the non-terminal path?

Operator ruling (verbatim): *"Yes to the paste question but print what was
dropped on stderr."*

## Inputs

- ms `/scratch/code/shibboleth/ms-worktrees/f687c` @ `350728f`, toolkit
  `/scratch/code/shibboleth/tk-worktrees/f687c` @ `8a578f38`, branch
  `f687c-drain-paste`; diff against `origin/master`.
- Report: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687c-impl.md`.

## Check under a real pty (your own harness, not the implementer's)

1. Paste `TREZOR\nline2\nline3\n`: fingerprint `b4e3f5ed` (BIP-39 vector
   "abandon ×11 about"); line2/line3 on stderr under the note; after exit,
   nothing is read by the parent shell side; echo on.
2. A slow paste: send `TREZOR\n`, then `line2\n` 50 ms later, then `line3\n`
   300 ms later. Which are printed, which reach the shell? The 0.1 s window is
   the design; say exactly what happens past it.
3. Partial last line, Ctrl-C mid-drain (exit by SIGINT, echo on), nothing
   pending (no note), a pipe (no note, bytes unchanged).
4. Every prompt: passphrase, BIP-38, decryption password, verify-bundle `Enter ms1:`.
5. The two surviving mutations (flush removed; prompt mode not restored): agree
   or disagree that they're unobservable, with evidence.
6. Gates: nextest both, pinned clippy, fmt. Run the pty tests 20 times in a
   loop for flakiness.

Own `CARGO_TARGET_DIR` under `/scratch/code/shibboleth/review-f687c-scratch/`;
delete with `find … -delete`. Don't commit, push or edit the branches.

## Output

Critical / Important / Minor / Nit, each with a reproduction. End with
`ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687c-review.md`**
and return only a short summary plus that path.
