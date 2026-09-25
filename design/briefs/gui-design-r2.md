# Brief — R2 re-review: mnemonic-gui design fold 2

**One question:** after fold 2 (`65d807b`, `f1f9ef3`), can the GUI deliver a
secret whose bytes differ from what the CLI would use for the same input, on
any OS, today or after F-687? Did fold 2 fix NI1, NI2 and the R1 Minors without
introducing a defect? GREEN (0C/0I) or not. Not a fresh audit.

## Scope (from the author, and correct)

1. **The terminator model.** Each channel has a measured suffix the GUI
   appends so the CLI's own strip removes it (`\r\n` stdin, `\n`
   `--decrypt-password-file`, none for `@env:`); `env_value_rule` in
   `channel_policy.json` is `verbatim` today; 12 "lenient" cells refuse unclean
   values.
   - Re-run `run_bytes` and the fold's `@env:` endings test. Construct values
     the fold may not have: a value ending in `\r` alone, `\n\n`, `\r\n\r\n`, a
     lone `\n`, an empty value, NUL, and a trailing tab. For each, does the GUI
     deliver the bytes the CLI's own `@env:` would, or refuse? A mismatch at
     exit 0 is Critical.
   - **The coupling to F-687.** The CLI newline rule is changing right now
     (engrave F-687, likely "strip exactly one trailing `\n` or `\r\n`" across
     `-`, `--passphrase-stdin` and `@env:`). When the GUI pins those CLIs, is a
     change to `env_value_rule` and the suffixes enough, and does anything fail
     loudly if someone bumps the pin without updating them? A pin bump that
     silently changes wallets is Critical. Check whether a test ties the
     suffix/rule table to the pinned CLI version.
2. **The interim path and its OS gate.** Off Linux: resolve `-`/`@env:`,
   refuse unmeasured sources, send resolved bytes on argv with
   `--allow-argv-secret`. Does the argv path apply the same `env_value_rule` (so
   macOS/Windows derive the same wallet as Linux for the same `@env:` value)?
   Does the `test_plan.py` workflow-parsing gate actually fail if an OS is
   added to `private_channels_on` without a real-binary CI job? Show it red.
3. **The Copy table:** e.g. `printf '%s\r\n' "$VAR" | …`, measured equal to
   argv-exact in bash, zsh and fish. Re-run it. Does Copy ever print a secret's
   value, as opposed to a variable reference?
4. Spot-check that 3 R1 Minors are fixed, including the refusal tests and the
   A5 regeneration.

Inputs: design and harness at `/scratch/code/shibboleth/gui-worktrees/followups/design/`;
R1 at `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r1.md`;
fold map at `.../gui-design-fold2.md`. Release CLIs via
`sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`.

Scratch under `/scratch/code/shibboleth/review-gui-r2-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branch.

## Output

Per R1 finding: fixed / not fixed. NEW findings Critical / Important / Minor /
Nit with evidence. End with `GREEN (0C/0I)` or `NOT GREEN`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r2.md`**
and return only a short summary plus that path.
