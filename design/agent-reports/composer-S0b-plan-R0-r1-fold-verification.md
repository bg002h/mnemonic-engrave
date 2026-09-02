# composer S0b plan — R0 round 1, FOLD-VERIFICATION lens

**Artifact under review:** the fold commit `6c308b6b8b51597a44ac621a4c962cf1ff6bd5b7`
(pre-fold `b1a3225`) applying round-0's two reports to
`design/IMPLEMENTATION_PLAN_composer_S0b_presets.md`.
**Reviewer:** independent, did not author the plan or the fold.
**Question:** did the fold fix each round-0 finding exactly (or decline it with a
stated reason), and did it introduce a new defect? Not a fresh audit.
**Method:** `cp -r` of the controller's read-only wired scratch
(`/scratch/code/shibboleth/.tmp/plan-build-gate-md`) to
`/scratch/code/shibboleth/.s0b-r1-lens` (`CARGO_TARGET_DIR` on
`/scratch/code/shibboleth/.s0b-r1-lens-target`, toolchain 1.85.0 confirmed from
the copy's own `rust-toolchain.toml`). Every mutation below was reverted and
diffed byte-identical to the pre-mutation file before the next one; no two
mutations were live at once.

## I-1 (defaults are also the device's shape) — VERIFIED

Fold rewrites the "Default parameters" paragraph: the numbers are stated as
both the vectors' parameters and the device's offered default shape; the two
shrunk archetypes get the stated UX justification ("the smallest legal shape...
is the honest starter, a wider tier is one edit away"), not the fixture budget;
a widening-widens-the-fixture sentence is present; S3 Task A10 is named as the
consumer. Matches the controller's decision (b) verbatim in substance. No code
to gate (prose-only).

## I-2 (one vector per archetype, not a wrapper cross-product) — VERIFIED

New paragraph states the wrapper is a parameter of the `PathList`, not an axis
of the archetype; one vector per archetype pins parameter order + lowering;
S3 A10's narrowing is named as scheduled by the controller after S3's own r1,
not made here; no twelve-vector export (confirmed: still exactly six new
`family()` rows in the diff, none added or removed). Matches the decision.

## I-3 (refusal table is a strict subset) — VERIFIED, with live mutation

Six wordings added as five table rows (one row folds k/n-too-large for both `k`
and `n`) plus one more folded under M-3. Preset-name-before-tokens fix and the
k/n-magnitude-vs-`BadThreshold` decline-with-reason are both present.

- **Order dependency, confirmed by mutation.** Reverted the `if
  !PRESET_NAMES.contains(&name)` early-return (folding the check back to
  after-parsing, restoring the pre-fold `unreachable!` construction to a
  graceful fallback so the crate still builds) and re-ran
  `cargo nextest run -p md-cli -E 'binary(/^cli_compose/)'`: exactly
  `preset_unknown_name_wins_over_a_malformed_token` failed (30/31 passed, that
  one FAILED, asserting `md: preset multisig: `2/3` is not <k>of<n>` instead of
  the expected "expected one of ..." line) — everything else stayed green.
  Reverted; diff against the pre-mutation file is empty.
- **Six wordings, machine-checked.** Ran the full 31-test suite unmutated:
  `preset_refuses_a_duplicate_named_parameter`,
  `preset_refuses_a_malformed_kofn_token`,
  `preset_refuses_a_kofn_magnitude_that_does_not_fit_a_small_number` (asserts
  both the k-too-large, n-too-large, AND the `2of15`→`BadThreshold` case in one
  test), `preset_refuses_a_non_numeric_named_value`,
  `preset_refuses_a_missing_or_malformed_sha256`, and
  `preset_decaying_multisig_after_in_the_time_band_names_path_as_the_remedy`
  (M-3) all PASS.
- **BadThreshold decline is a stated reason, not silence.** The design note
  explains the two messages are for two different failure classes (a `u8`
  parse failure vs. a value that fits `u8` but violates the `1<=k<=n<=9` band)
  — matches the fold-brief's "(or state why two messages are right)" option.

## I-4 (fork pin-test edit ownership) — VERIFIED

Fold states S3 Task A10 owns `md/compose_vectors_pin_test.go`'s two hand edits
(26→32 names, 126→156 count), names A10's own three current errors (wrong
Files list, wrong claim that the pin test "moves with" the count, wrong
vendored-file prefix and a missing fifth file), and explicitly does NOT touch
the fork or the S3 plan itself — matches "S0b states this ownership... the S3
side is folded later by the controller."

## M-1..M-4, N-1, N-2 (fidelity) — three VERIFIED, one NOT VERIFIED (M-4)

- **M-1** (older1 locks the primary tier): added to both the grammar-table
  prose and the `--preset` `--help` doc comment (`/// For decaying-multisig,
  older1 locks the FIRST (primary) tier...`). VERIFIED by reading both sites.
- **M-2** (all ten (archetype, wrapper) legacy pairs, including `sh-wsh`):
  **counted directly in the test source** —
  `preset_every_non_plain_archetype_refuses_under_both_legacy_wrappers_spec_4d_shape`
  loops `for wrapper in ["sh", "sh-wsh"]` × 5-element `non_plain` array = 10
  failure assertions, plus 2 `plain-multisig` success checks (one per
  wrapper). Confirmed `sh-wsh` is a real, working `Wrapper::ShWsh` CLI value
  (`"sh-wsh" => Ok(Wrapper::ShWsh)` in `compose.rs:19`; ran it live against the
  built `md` binary and got the expected `sh(wsh(...))` template). **VERIFIED,
  exactly ten.**
- **M-3** (after-in-time-band names `--path` as remedy): implemented via a new
  `need_after_height` closure gated on `md_codec::compose::LOCKTIME_THRESHOLD`,
  used only for `decaying-multisig`'s `after`. Test passes. The wording is a
  close paraphrase of `--path`'s own line (`"reads as a block height"` vs. the
  preset's `"is read as a block height"`, both citing the identical
  `(1..=499999999)` band) — not byte-identical, but the comment only claims
  "mirroring," and the remedy (`--path ... after=<v>t`) is named as required.
  **VERIFIED** (cosmetic wording variance noted as a Nit, not blocking).
- **M-4** (`PRESET_NAMES` cannot drift from the match arms) — **NOT VERIFIED.
  Important.** The fold added
  `every_preset_name_parses_with_some_valid_parameters`, but it iterates a
  **hardcoded** `let valid: [(&str, &str); 6] = [...]` fixture, not
  `PRESET_NAMES` itself, and its guard —
  `assert_eq!(valid.len(), 6, "one entry per PRESET_NAMES member; update this
  fixture if a preset is added")` — compares a fixed-size array's `.len()`
  (`6`, fixed by its own type annotation) to the literal `6`. This is a
  **tautology that cannot fail under any edit** to `PRESET_NAMES` or the
  `match` arms; it is not wired to either.

  **Confirmed live by mutation:** added a 7th entry `"phantom-preset"` to
  `PRESET_NAMES` (`[&str; 6]` → `[&str; 7]`) with **no** matching `match` arm
  (the exact drift direction the original M-4 finding named: *"A name added to
  one and not the other compiles and clippy-passes, and produces an 'expected
  one of…' line advertising a name that does not work"*). Result:
  - `cargo build -p md-cli --all-targets` — clean, no warning (a runtime
    `&str` match has no exhaustiveness check for the compiler to flag).
  - `cargo nextest run -p md-cli -E 'binary(/^cli_compose/)'` — **31/31 PASS,
    0 failed**, including the new M-4 test itself. Nothing catches it.
  - Ran the built binary directly: `md compose --wrapper wsh --preset
    phantom-preset,2of3` → **panics**: `thread 'main' panicked at
    crates/md-cli/src/cmd/compose.rs:387:18: internal error: entered
    unreachable code: name already validated against PRESET_NAMES:
    phantom-preset`, exit code **101**.
  - This is *worse* than the original finding's failure mode (a bad but
    graceful CLI message) — it is an unhandled panic reachable by a plain
    typo-free CLI invocation once the two lists drift in this direction, and
    the added test provides no defense.
  - Reverted; diff against the pre-mutation file is empty.

  Per the brief's severity rule ("a finding folded against the controller's
  decision, a test that cannot fail... = Important"): the controller's decision
  was "PRESET_NAMES derived from the one match or asserted equal to it by a
  test" — the fold did neither; it added a check that makes `PRESET_NAMES`
  authoritative over `match` for one drift direction while leaving the other
  direction not just open but panicking, and the test meant to close it
  contains a self-referential tautology. **Classified Important.**

## N-1 (fidelity, `head:hashed` tag) — VERIFIED

`hashlock_gated`'s family row gets `head:hashed`, with the stated reason (its
head path is one key + a hash, unlocked — none of `head:bare-multi`,
`head:single`, `head:locked`); `SINGULAR_TAGS` gets the same tag with a matching
comment. `cargo nextest run -p md-codec -E 'binary(/^compose_/)'` is 52/52
(unmutated, see Gate below) — `every_tag_appears_in_at_least_two_vectors` is
part of that binary and passes, confirming the singular-tag exemption holds.

## N-2 (fidelity, `--json` schema bump) — VERIFIED as a valid decline

Declined with the stated reason (`SCHEMA` has never moved for an additive
field; S0's own `compose --json` shipped without a bump). The fold-brief
explicitly permitted "a `--json` `schema` bump or a stated reason not to" —
this is the latter, and the reason cites a real precedent (S0's own history),
not an assertion.

## Tests-lens N-1 (Step 2 header wording) — VERIFIED

"Run to verify the tests fail to compile" → "Run to verify the tests fail",
with the R0 tests-lens N-1 citation and the precise mechanism (clap runtime
parse rejection, not `rustc`). Matches.

## Tests-lens N-2 (workspace count parenthetical) — VERIFIED, and see below

The plan now carries a parenthetical noting a scratch missing
`design/display-grouping-vectors.tsv*` shows one extra red, already fixed in
the gate script (`a13feec`). Folded as described. This connects directly to
the reconciliation item below.

---

## Extra item — the plan's Expected 1340 vs. the fold commit's cited 1302

**The plan's `1340 tests run: 1340 passed, 3 skipped` is CORRECT for a real
worktree state and reproduces byte-for-byte. The fold commit's cited `1302
passed, 3 skipped` does not reproduce and should not be trusted.**

Measured directly in the `cp -r` copy of the SAME read-only wired scratch the
controller used:

```
cargo nextest list --workspace --all-features | grep -c '::'   → 1340
cargo nextest run  --workspace --all-features   → Summary [65.172s] 1340 tests run: 1340 passed (1 slow), 3 skipped
```

Zero failures, exact match to the plan's Expected line (the "(1 slow)" is
nextest's own annotation for the one 60s+ property test,
`compose_crosscheck::every_family_entry_passes_the_5b_cross_check`, not a
failure). `-p md-cli` alone (no `--all-features`, matching the plan's own
Step-5 command exactly) is `783 tests run: 783 passed, 1 skipped` — also an
exact match to the plan's stated 783.

Since this is a `cp -r` of the identical scratch the controller's own gate ran
against (not a fresh build from the plan's fragments), a discrepancy here
cannot come from "the copy lacks files the real repo has" (the brief's
hypothesis) — the workspace only has two members (`crates/md-codec`,
`crates/md-cli`, confirmed from the real repo's own `Cargo.toml`), both fully
inside the copied `crates/` tree, and the display-grouping `.tsv` sidecar
(tests-lens N-2's finding) is present in this copy (the gate script's fix,
`a13feec`, copies it). The most likely explanation is that the controller's
"1302 passed, 3 skipped" in the fold commit message was measured against an
earlier or partially-wired state of the scratch, or a transient run, rather
than the final state now sitting read-only at `.tmp/plan-build-gate-md`. This
is not a defect in the plan or the fold — the plan's own Expected line is the
one that holds up under direct, independent re-measurement.

---

## Gate (unmutated, post-revert state)

- `cargo nextest run --locked -p md-codec -E 'binary(/^compose_/)'` → **52
  tests run: 52 passed (1 slow), 0 skipped**.
- `cargo nextest run --locked -p md-cli -E 'binary(/^cli_compose/)'` → **31
  tests run: 31 passed, 0 skipped**.
- `cargo clippy --locked -p md-cli -p md-codec --all-targets -- -D warnings` →
  exit 0, no warnings.
- `cargo fmt --all -- --check` → exit 0, clean.
- `scripts/plan-cite-check.sh` → **25/25** resolved, 0 dangling, 0 ambiguous.
- `scripts/plan-glyph-check.sh` → **104 strings, 0 undrawable**.
- `scripts/plan-table-check.sh` → **25 rows, 0 malformed**.
- `scripts/plan-stepref-check.sh` → **15** prose step references (the
  tolerated class).

All match the fold commit's own cited numbers exactly.

## New-defect sweep

Walked every hunk of `git diff b1a3225..6c308b6b -- design/IMPLEMENTATION_PLAN_composer_S0b_presets.md`
(495 diff lines). Every hunk is attributable to a named finding (I-1..I-4,
M-1..M-4, N-1/N-2 fidelity, N-1/N-2 tests-lens) or is bookkeeping downstream of
one (test-count arithmetic in "What is already machine-verified," Step 5, and
the whole-workspace Expected line; the self-review section's restatement of
M-2's renamed test and N-1's new tag). No orphan hunk found. The one hunk that
does **not** deliver what it is attributed to is M-4's added test — see above;
that is a quality defect in an attributed hunk, not an unattributed one.

## Closing counts

| severity | count | item |
|---|---|---|
| Critical | 0 | — |
| Important | 1 | M-4: `every_preset_name_parses_with_some_valid_parameters` does not actually prevent `PRESET_NAMES`/`match`-arm drift (tautological length assertion against a hardcoded fixture, not `PRESET_NAMES` itself); confirmed live to compile, pass clippy, pass the full 31-test suite, and then **panic** (exit 101, `unreachable!()`) on a real CLI invocation once the two lists drift by one entry — worse than the ungraceful-message failure mode the original finding named |
| Minor | 1 | M-3's time-band wording paraphrases rather than reuses `--path`'s wording verbatim (`"reads as"` vs `"is read as"`) — cosmetic, the remedy is still correctly named |
| Nit | 0 | — |

**Not closed at 0C/0I.** Recommend for the next fold: either iterate the test
directly over `PRESET_NAMES` (a `BTreeMap<&str, &str>` of name→valid-params
keyed the same, with a loop `for name in PRESET_NAMES { params[name]... }`
rather than a hand-typed parallel array), or change the `unreachable!()` arm
back to a graceful `CliError` as a defense-in-depth backstop even if the name
check stays first — either closes the live panic path this round's mutation
found.
