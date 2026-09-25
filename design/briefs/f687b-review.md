# Brief — adversarial review of F-687b (ms `3a031cf`, toolkit `7f9c400b`)

**One question:** can any secret input these branches touch derive a wallet
other than the one the user's bytes specify, silently use an empty or wrong
value, leak the typed secret to the screen, or leave the terminal broken?
Not a fresh audit.

## Inputs

- ms `/scratch/code/shibboleth/ms-worktrees/f687b`, toolkit
  `/scratch/code/shibboleth/tk-worktrees/f687b`, branch
  `f687b-passphrase-rulings`; diff each against `origin/master`.
- Report (claims to re-run): `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687b-impl.md`.
- Operator rulings (engrave F-687): (1) same `-`/`@env:` rule on
  `--bip38-passphrase` and `--decrypt-password`; (2) an empty value from
  stdin/`@env:` warns and proceeds; (3) "Enter passphrase:" on stderr when
  stdin is a terminal. Plus F-689 (`verify-bundle --ms1 -`) and F-691 (exact
  `-` only).

## Settled

- F-687 (the `--passphrase` resolver) is reviewed and on master. Only
  re-check it where these branches touched it.

## Push hardest on

1. **Rulings 1–2 by effect:** for `--bip38-passphrase` (convert) and
   `--decrypt-password` (import-wallet, electrum-decrypt), give the same secret
   via `-`, `@env:`, the `-stdin` flag where one exists, and the literal, and
   compare outputs (a decrypted key or wallet). Need a BIP-38 test vector
   (the BIP-38 spec has published ones) and an Electrum-encrypted fixture;
   build them if the repo lacks them. Any mismatch is Critical.
2. **Ruling 3, the terminal:** under a real pty (e.g. python `pty`/`script`),
   check the prompt appears, input doesn't echo, echo is restored after, and
   what happens on Ctrl-C (the report says echo can stay off) and on EOF at the
   prompt. Is the echo-off leak fixable cheaply (restore on SIGINT)? Report.
   No prompt when stdin is a pipe or file, and the bytes are unchanged.
3. **F-689:** `verify-bundle --ms1 -` against a matching bundle gives
   `result: ok`, and empty stdin is refused. The watch-only marker `""` path
   must be unreachable from `-`.
4. **One stdin per invocation**, across the new flags combined with existing
   stdin readers, including stdin-by-path (`/dev/stdin`).
5. **Empty warning:** exact text, once, stderr only; never for a literal `""`
   beyond the argv note.
6. **Vectors:** 42 cases byte-identical across repos; re-run 3 of the 13
   claimed mutations and add one of your own.

Build with your own `CARGO_TARGET_DIR` under
`/scratch/code/shibboleth/review-f687b2-scratch/`; delete with `find … -delete`.
Don't commit, push or edit the branches.

## Output

Critical / Important / Minor / Nit, each with a reproduction. End with
`ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687b-review.md`**
and return only a short summary plus that path.
