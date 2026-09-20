# Fold verification — fold A's fold (descriptor-mnemonic), composer fable review r0

**Verdict: VERIFIED, 7/7.** All seven findings (I-1, M-1, M-2, M-3, N-1, N-2,
N-3) were fixed exactly as the review stated — no neighbour, no paraphrase.
Every new assertion the fold added can fail: three prescribed mutations, each
applied to a fresh worktree at the fold commit and reverted, produced RED at
the exact test the finding names. A fourth mutation, run to double-check the
fold's own M-2 doc-comment correction, reproduced its claimed panic line
exactly. No new defect found.

Mechanical lens (sonnet), read-only. Repo
`/scratch/code/shibboleth/descriptor-mnemonic`, range `4de55155..dd7cdffc`.
Own worktree `/scratch/code/shibboleth/.tmp/verify-rust` (detached at
`dd7cdffc`), own `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/verify-rust-target`,
`TMPDIR=/scratch/code/shibboleth/.tmp`, pinned toolchain
(`rustc 1.85.0`, `cargo 1.85.0`) first on PATH. Nothing committed; no
sub-agents; no `.jsonl` read. `git status --porcelain` empty and
`git diff dd7cdffc --stat` empty at the end of every mutation and at the end
of the run. Did not re-run the full 1356-test suite, fmt or clippy (already
settled per the brief).

## Finding → verdict table

| # | Finding (review) | Fold's fix, exact location | Verdict | Evidence |
| --- | --- | --- | --- | --- |
| I-1 | mk1 cards WALLET-CONFIRMED at 0.16.2 now warn, undocumented | `CHANGELOG.md` `[0.17.0]` entry, new "Migration" paragraph | VERIFIED | Content matches the review's "Suggested content" clause-for-clause: decode + SPEND-EQUAL exit 0, policy shape id unaffected, only composed-wallet-id-stubbed mk1 cards need re-minting, fail-safe direction stated. Warning text quoted matches `crates/md-cli/src/seat/disposition.rs:139-141` verbatim except "before trusting." → "before trust" (grammatical fit into the CHANGELOG's own sentence, not a substance change — checked against source string). |
| M-1 | cap pre-empts `TooManySlots`/`LegacyWrapperShape`, remedy fails | `crates/md-codec/src/compose/mod.rs:549-574` — the `TooManyKeylessPaths` check block moved to run *after* the `TooManySlots` and `LegacyWrapperShape` checks | VERIFIED | Diff confirmed: block relocated exactly as review prescribed; new comment cites M-1 and both prior precedence rules. Mutation 1 (below): RED. |
| M-2 | mutation doc names wrong failing assertions | `crates/md-cli/tests/cli_compose_encode_gate.rs:99-105` doc comment rewritten to "assertions 2 and 4 fail ... assertion 3 SURVIVES" | VERIFIED | Mutation 4 (below) reproduces the panic at the exact assertion the corrected comment identifies. |
| M-3 | fill grows a card, can cross the 64-chunk cap, undocumented | `CHANGELOG.md`, second new paragraph ("One more consequence, at the wire's edge") | VERIFIED | Chunk-count numbers (`n=6: 11→12`, `n=9: 17→18`, `n=11: 20→21`, `n=3` unchanged) transcribed byte-for-byte from the review's measured table — no new unmeasured number introduced. |
| N-1 | vector's precedence prose has no driving case | `crates/md-codec/tests/vectors/compose_refusal_keyless_cap.json` — 4 new cases (`precedence_keyless_under_tr`, `precedence_no_keyed_path`, `precedence_too_many_slots`, `precedence_legacy_wrapper_shape`) + `crates/md-codec/tests/compose_keyless_cap.rs` loop reads `error.kind` to pick the expected variant | VERIFIED | Baseline run: all 4 new cases pass through the loop (`every_case_in_the_conformance_vector_behaves_as_recorded` 10/10 tests green, no unreadable field). Mutation 2 (below): RED when the loop is forced to ignore `kind`. |
| N-2 | `refused_when` looser than the code | `…compose_refusal_keyless_cap.json:34` reworded to add the `hash != null` clause and the `LockOnlyPath` carve-out | VERIFIED | Text now matches `validate()`'s actual classification at `compose/mod.rs:525-530` (`hash.is_none()` → `LockOnlyPath` fires before a path can ever reach `keyless.push`). |
| N-3 | premise `OR` would pass with both fingerprints at `@0` | `crates/md-cli/tests/i2_partial_template_completion.rs:118-127` — loop now pairs `SLOTS[i]` with `@{i}`, no `OR` | VERIFIED | Mutation 3 (below): RED when the pairing is deliberately broken (checked against the wrong slot). |

**7 VERIFIED / 0 NOT VERIFIED.**

## Mutations run, each on the fold tip, each reverted

All four applied and reverted in `/scratch/code/shibboleth/.tmp/verify-rust`;
`git diff dd7cdffc` was empty before and after every one.

### Mutation 1 — swap the two new precedence checks back above the cap (M-1)

Moved the `TooManyKeylessPaths` block in `compose/mod.rs` back to its
pre-fold position (before `TooManySlots`/`LegacyWrapperShape`), i.e. the exact
reverse of the fold's M-1 hunk.

```
$ cargo nextest run -p md-codec --test compose_keyless_cap
FAIL a_legacy_wrapper_with_two_keyless_paths_still_reports_the_legacy_rule
  assertion `left == right` failed
    left: TooManyKeylessPaths { first: 1, second: 2 }
   right: LegacyWrapperShape
FAIL a_policy_over_the_slot_cap_still_reports_the_slot_cap
  panicked: TooManyKeylessPaths { first: 4, second: 5 }
FAIL every_case_in_the_conformance_vector_behaves_as_recorded
  assertion `left == right` failed: precedence_too_many_slots
    left: TooManyKeylessPaths { first: 4, second: 5 }
   right: TooManySlots { got: 36, max: 32 }
Summary: 10 tests run: 7 passed, 3 failed
```

Reverted with `git checkout -- crates/md-codec/src/compose/mod.rs`; diff
against `dd7cdffc` empty afterward.

### Mutation 2 — make the vector loop ignore `kind` (N-1)

In `compose_keyless_cap.rs`, replaced `match want["kind"].as_str().expect("kind")`
with `match "TooManyKeylessPaths"` (loop always assumes the old single error
shape, i.e. `kind` is read from the JSON but never consulted).

```
$ cargo nextest run -p md-codec --test compose_keyless_cap -- every_case_in_the_conformance_vector_behaves_as_recorded
FAIL every_case_in_the_conformance_vector_behaves_as_recorded
  panicked at crates/md-codec/tests/compose_keyless_cap.rs:310:70:
  first
Summary: 1 test run: 0 passed, 1 failed
```

The panic is `.expect("first")` on `precedence_keyless_under_tr`'s `error`
object, which has no `first` field — the mutated loop cannot even read that
case, let alone check it. Reverted with `git checkout --`; diff empty
afterward.

### Mutation 3 — break the pairing in the i2 premise (N-3)

In `i2_partial_template_completion.rs`, changed the loop to check
`SLOTS[i]`'s fingerprint against `@{1-i}` (the *other* slot) instead of
`@{i}`.

```
$ cargo nextest run -p md-cli --test i2_partial_template_completion -- the_fixture_header_still_records_this_mint
FAIL the_fixture_header_still_records_this_mint
  panicked at crates/md-cli/tests/i2_partial_template_completion.rs:125:9:
  the fixture no longer declares 73c5da0a at @1
Summary: 1 test run: 0 passed, 1 failed
```

Reverted with `git checkout --`; diff empty afterward.

### Mutation 4 — extra check: does M-2's own corrected doc-comment hold up?

Not one of the brief's three prescribed mutations, but the natural way to
verify M-2 (a fix to what a mutation NOTE claims, not to test logic) is to
run the mutation the note describes and check the panic lands where the
corrected comment now says it does. Deleted the `TooManyKeylessPaths` arm
entirely from `compose/mod.rs` (the exact mutation M-2's doc comment
describes) and ran the test it documents:

```
$ cargo nextest run -p md-cli --test cli_compose_encode_gate -- compose_refuses_to_emit_a_template_encode_would_reject
FAIL compose_refuses_to_emit_a_template_encode_would_reject
  panicked at crates/md-cli/tests/cli_compose_encode_gate.rs:135:5:
  the refusal does not name the offending paths:
  ...
  template parse error: miniscript parse failed even with --experimental: Miniscript is malleable ...
Summary: 1 test run: 0 passed, 1 failed
```

Line 135 is the `err.contains("paths 2 and 3")` assertion — the fold's
corrected comment calls this "assertion 2" (of the four content assertions
following the exit/stdout checks) and states it fails while "assertion 3"
(the `"malleable"` check) would survive, since the generic parser message
does contain that word — visible directly in the captured stderr above. Both
claims check out against what actually ran. Reverted with `git checkout --`;
diff empty afterward.

## Anything new

- **No new defect** matching the brief's two named shapes (a vector case the
  loop cannot read; a CHANGELOG number the review never measured). The four
  new vector cases are all read correctly at baseline (10/10 green before any
  mutation), and both new CHANGELOG numeric claims (the chunk counts) are
  verbatim from the review's own measured table.
- **One cosmetic-only observation, not filed as a finding**: the vector
  file's top-level `"error_fields"` documentation block still lists only
  `first`/`second` and does not mention `path`, `got`, `max` or the new
  `kind` discriminant, even though four cases now use them. This does not
  block anything — the test loop reads every field correctly (confirmed
  above) — so it is a doc-completeness nit on the schema's own self-description,
  not a functional gap; leaving it unfiled per scope (not one of the seven
  findings, and not a new defect the fold introduced that anything fails on).
- Baseline (unmodified fold tip) targeted-test run, for completeness:
  `compose_keyless_cap` 10/10, `i2_partial_template_completion` +
  `cli_compose_encode_gate` 9/9 — 19/19 total, all green, both before and
  after the four mutation/revert cycles.

## Cleanup

`git status --porcelain` and `git diff dd7cdffc` both empty immediately
before removal. Worktree removed with `git worktree remove
/scratch/code/shibboleth/.tmp/verify-rust`, its target dir
(`/scratch/code/shibboleth/.tmp/verify-rust-target`) deleted, and
`git worktree prune` run. No files outside the worktree were touched.
