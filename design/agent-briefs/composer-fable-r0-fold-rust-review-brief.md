# Whole-diff review — fold A (descriptor-mnemonic, the Rust primary), composer fable review r0

Independent reviewer (opus). You did not write the diff, the brief, or the
report. Repo `/scratch/code/shibboleth/descriptor-mnemonic`, branch
`fable-r0-fold`, base `922778ad0400957840da26081846fc761deaca41`, tip
`4de55155f7bb5dda6896f733331f9ccd97275490`. YOUR RANGE IS `git diff 922778ad..4de55155`
(two commits: `3d22b714` the key-less cap, `4de55155` the fingerprint fill).
Read-only; commit nothing; no sub-agents; never read `.jsonl`. Work in your own
worktree: `rm -rf /scratch/code/shibboleth/.tmp/review-rust && git -C /scratch/code/shibboleth/descriptor-mnemonic worktree add --detach /scratch/code/shibboleth/.tmp/review-rust 4de55155f7bb5dda6896f733331f9ccd97275490`;
own `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/review-rust-target`;
`export TMPDIR=/scratch/code/shibboleth/.tmp`; never build under `/tmp`. Clippy
under the PINNED toolchain: put `$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin`
first on PATH (a bare `cargo clippy` resolves 0.1.98 and reports pre-existing
noise). Revert every mutation; remove the worktree when done.

What was asked of the implementer: `design/agent-briefs/composer-fable-r0-fold-rust-brief.md`
(mnemonic-engrave). What the implementer reports:
`design/agent-reports/composer-fable-r0-fold-rust-implementation.md`. The
findings themselves: lens 1 C-1 in `composer-fable-r0-funds-safety.md`, lens 3
I-2 in `composer-fable-r0-steel-restore.md` (same directory).

## Already settled — do not re-derive
- The rule "more than one key-less path ⇒ refuse, any locks, any positions" is
  MEASURED (controller: 22 lists; implementer: 859 lists, 0 counterexamples)
  and follows from `or_i`'s non-malleability needing one `safe` arm. Do not
  re-argue the rule; check that the code implements THAT rule and nothing
  narrower or wider.
- The implementer reports nextest 1354/1354, fmt clean, pinned clippy clean;
  the controller is re-running that gate in parallel. Do not spend your budget
  re-running the whole suite; run what you need to construct counterexamples.

## ONE QUESTION
Over this diff, can you construct an input for which either change gives a
WRONG answer — a path list with at most one key-less path that the new
`validate()` refuses, or one with two that it admits; a seating where a
PRESENT fingerprint is overwritten, or an ABSENT one is filled with the wrong
value, or the completed card's wallet id differs from the fully-seated mint of
the same keys — or for which a test in this diff passes while the property it
names is false? Also: the fingerprint fill changes the md1 the host emits for a
completed partial template (5 of 34 fixtures move their id, addresses
unchanged, per the report) — is every consumer of that id (seat, verify,
identity, stubs, the fork's expectations) still consistent, and does the
CHANGELOG say what an operator with an already-completed 0.16.x card should
expect?

Five deviations are declared in the report's §"Deviations" — judge each.

## Report — your FINAL action
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-fable-r0-fold-rust-review.md`:
verdict line with counts C/I/M/N; each finding as a counterexample (inputs,
observed, expected, file:line); the deviations, each ACCEPTED or a finding;
what you ran with its numbers; what you could not verify. Return only the
verdict line, the counts, and the path.
