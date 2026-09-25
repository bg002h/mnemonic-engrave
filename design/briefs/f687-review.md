# Brief — adversarial review of F-687 (ms `e534917`, toolkit `9846a784`)

**One question:** after this change, can any passphrase input in either CLI
derive a wallet other than the one the user's bytes specify, or silently use
an empty or wrong passphrase? Not a fresh audit of either CLI.

## Inputs

- ms: `/scratch/code/shibboleth/ms-worktrees/f687`, branch
  `f687-passphrase-channels`, `git diff origin/master..e534917`.
- toolkit: `/scratch/code/shibboleth/tk-worktrees/f687`, same branch name,
  `git diff origin/master..9846a784`.
- Report (claims to re-run): `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687-impl.md`.
- Ruling: engrave F-687 (`-` = stdin, `@env:VAR` = environment, both CLIs,
  every command; nothing refused; literal argv warns). Operator fact: no
  existing wallets need preserving.

## Settled

- Controller-verified on an ms build: TREZOR via `--passphrase-stdin`, `-`,
  `@env:V`, `@env:V` with a trailing `\n`, and a literal all give fingerprint
  `b4e3f5ed` (BIP-39 vector "abandon ×11 about"); unset VAR errors.
- Newline rule: `-`, `--passphrase-stdin` and `@env:` strip exactly one
  trailing `\n` or `\r\n`; an argv literal is verbatim. That's the chosen
  rule, not a finding.

## Push hardest on

1. **Every passphrase-taking subcommand in both CLIs**: enumerate them from
   `--help`, not from the report, and run `-`, `@env:`, the literal and
   `--passphrase-stdin` with the same passphrase, comparing fingerprints and
   addresses. Include verify-bundle **against a matching bundle** (assert
   `result: ok`) and xpub-search. A subcommand the shared resolver missed is
   Critical.
2. **Byte edges:** `\r` alone, `\n\n`, `\r\n\r\n`, a leading newline, NUL,
   non-UTF-8, and a very long value, through each channel. Do the three
   channels always agree?
3. **stdin sharing:** `-` with another stdin input must refuse; `-` with
   `--passphrase-stdin` refuses (exit 64). Is there any combination where one
   stdin is consumed twice, or read by the wrong input? The two pre-existing
   bugs the implementer fixed (verify-bundle keyless re-read; xpub-search
   `--phrase-stdin --passphrase-stdin`) show this class exists; hunt for more.
4. **Empty passphrase:** empty stdin, and a set-but-empty VAR, both give the
   empty passphrase. Is that ever silent where the user clearly meant to supply
   one? Report it; the operator will rule.
5. **The stderr note:** never contains the passphrase; appears once; stdout
   unchanged. Test with a passphrase that looks like a flag or contains quotes.
6. **Test vectors** (`passphrase_channels.json`, byte-identical in both repos):
   do they pin the behaviour, or could the resolver regress with them green?
   Re-run 3 of the 13 claimed mutations.
7. CHANGELOG and help: accurate against the binaries.

Build per repo with its own `CARGO_TARGET_DIR` under
`/scratch/code/shibboleth/review-f687-scratch/`; pinned clippy for gates if
you re-run them. Delete scratch with `find … -delete`. Don't commit, push or
edit the branches.

## Output

Critical / Important / Minor / Nit, each with a reproduction. End with
`ready to ship: yes` or `ready to ship: no`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f687-review.md`**
and return only a short summary plus that path.
