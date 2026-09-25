# Brief — R0 review: mnemonic-gui DESIGN_secret_channels_and_new_forms

**One question:** if this design is implemented as written, can any secret
reach a flag the user didn't fill, reach a wallet other than the one intended,
or silently fall back to a channel the design says it refuses? Second: is it
complete enough to implement without guessing? Not a review of existing GUI code.

## Inputs

- The design: `/scratch/code/shibboleth/gui-worktrees/followups/design/DESIGN_secret_channels_and_new_forms.md`
  (branch `gui-followups` at `ec31aed`), with its measurement scripts in
  `design/measurements/secret-channels/`.
- The author's report: `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-followups-phase1.md`.
- The follow-ups it answers: `argv-secret-via-private-channels` and
  `md-ms-new-subcommands-unsurfaced` in `/scratch/code/shibboleth/mnemonic-gui/FOLLOWUPS.md`.
- Release CLIs: `sh /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh --no-gui --no-man --root <scratch>`
  (mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0).

## Settled — do not re-derive

- Controller-reproduced: on ms 0.19.1, `--passphrase -` is the literal
  passphrase `-` (fingerprint `66d564d1`, exit 0) even with the real passphrase
  on stdin. Filed as engrave F-687 for a CLI-side fix; the design must be
  correct against today's CLIs regardless.
- Secret-handling severity rule (operator 2026-08-27): exposure of a secret is
  never Critical/Important. **A secret reaching the wrong flag or the wrong
  wallet is a correctness/funds defect and gates normally.**

## Where to push hardest

1. **The channel table is the design's foundation.** Re-run the measurement
   scripts and at least 10 rows by hand, including every `-` and `@env:` row.
   A row marked OK that actually changes the result (the `--passphrase -` class)
   is Critical. Are there inputs where "accepted, exit 0" hides "used a
   different value"? Does the design check the *effect* (a fingerprint or
   address), not just the exit code?
2. **Two or more secrets in one run:** one stdin per invocation. Is the
   assignment rule deterministic, and can two secrets ever swap? What happens
   with three?
3. **`/dev/fd/3` for `ms --in`:** Unix-only. What does the design do on
   Windows and macOS? Is the pipe written fully before the child reads, and
   what if the child exits early or never reads (a deadlock, or a SIGPIPE)?
4. **Refusal instead of fallback:** find any path where an unmeasured input
   would still end up on argv.
5. **Shown vs run:** do Preview, Copy command and the confirm dialog tell the
   truth about which channel carries each secret?
6. **The five forms:** check each table against the release binary's
   `gui-schema`/`--help`. `ms hashlock`'s phrase is used byte-for-byte (no
   trim) and `--kind` needs a "not specified" option; are those enforced by
   the design, or just noted?
7. Answer the design's open questions Q1–Q4 with a recommendation each.
8. Could the implementation phase's acceptance tests actually fail? Name the
   test that would catch a swapped-secret regression.

Scratch under `/scratch/code/shibboleth/review-gui-r0-scratch/`; delete with
`find … -delete`. Don't commit, push or edit the branch.

## Output

Critical / Important / Minor / Nit, each with evidence or a constructed
counterexample; recommendations on Q1–Q4. End with `GREEN (0C/0I)` or
`NOT GREEN`.

**As your final action, write the report to
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/gui-design-r0.md`**
and return only a short summary plus that path.
