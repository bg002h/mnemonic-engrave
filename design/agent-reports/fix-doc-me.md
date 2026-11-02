**⚠ 1 Important: R1's fix-04, as ruled, overrides normative SPEC_descriptor_input.md §5.1 and fails an existing test (STOP S3).**

# FIX-doc author report: me bug hunt

FIX doc: `design/FIX_bughunt_me.md` (commit 9a172a38, with RED rows 02b, 03b,
04b, 05b appended to `crates/me-cli/tests/bughunt.rs`). No production code is
committed.

## What was run

- RED on the unmodified base, `-E 'test(/^bughunt_me_/)'`: 11 run, 2 passed
  (06 and 07), 9 failed. Each new row fails for its stated reason (the panic
  texts are quoted in FIX section 3).
- All five fixes applied to a scratch probe worktree at the base. Results:
  - nextest: 702 run, 701 passed, 1 failed, 2 skipped;
  - doctests: rc 0; fmt: rc 0; clippy `-D warnings`: rc 0;
  - bughunt filter: 11 run, 11 passed.
  The one failure is `descriptor_as::as_with_argv_and_in_together_is_usage` (S3).
- CLI probe of both binaries: with `--as md1 <desc> --in f`, the base prints
  §5.1's `--as packs exactly one descriptor per invocation.`; the probe
  prints `pass records on argv OR via --in, not both`. Both exit 2.
- Toolkit manual greps on origin/master f6355e22: no rows change.

## Findings

- **I (S3):** fix-04 sits in `read_records`, which runs before the `--as`
  check. It therefore pre-empts §5.1's normative text for `--as` together
  with argv and `--in`, and the `--in` row at SPEC_systemwide_payloads.md:908
  says §5.1 governs when `--as` is present. The grade did not consider this.
  This is a design question for the architect: (a) amend §5.1 and the test,
  or (b) skip the refusal when `--as` is given.
- **STOP S1:** fix-04 also runs before the argv bearer guard. That
  contradicts the pack arm's stated rule at main.rs:1519-1524 ("a flag-shape
  usage error must not pre-empt a refusal about material that is already in
  the shell's history"), though it matches seal's order. The consequence (lost
  history advice) is secret-handling, so it does not gate on its own; the
  order is undecided.
- **STOP S2:** R5 asks for the "0o600 binds on CREATE" correction but gives no
  text. The FIX doc proposes a wording.

## Judgement calls (noted, not STOPs)

- The #2 spec sentence goes in as a new A1 paragraph after line 40 and before
  "Placement:".
- SPEC_me_preview_hardening.md:106 `.truncate(true)` is dropped from the
  OpenOptions text. R5 forces this; otherwise the spec states the old options.
- The write.rs unit-test doc and assert text are updated (text only).
- Build order: L5, #3, #2, #1, then #4 after S1 and S3. #4 is the only blocked item.

Scratch: `/scratch/cache/bughunt/me/fixdoc/` (probe worktree, logs). It is
safe to delete.

Counts: Critical 0, Important 1, Minor 0, Nit 0; STOPs 3 (S1, S2, S3).
