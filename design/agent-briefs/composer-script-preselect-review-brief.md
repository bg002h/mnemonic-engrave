You are the INDEPENDENT adversarial execution reviewer (opus tier) for a Critical
fix in the SeedHammer fork: the composer's script picker now opens on the script
in force instead of always on row 0.

Repo `/scratch/code/shibboleth/seedhammer`, branch `composer-script-preselect`,
tip `6728c22`, base `main` at the merge commit below it. The whole diff is
`git diff main..6728c22`. The finding it fixes is journey C-1 in
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-policy-journey.md`.

ONE QUESTION: over the whole diff, can you construct a screen or a sequence
where this change makes a picker open on, or return, a row it should not — and
does the new test actually fail on the defect it names?

Read-only on the fork; commit nothing; no sub-agents; never read any `.jsonl`.
Work in your OWN scratch copy:
`rm -rf /scratch/code/shibboleth/.tmp/preselect-review && git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/preselect-review 6728c22`.
Go is `/scratch/code/shibboleth/.toolchain/go/bin/go`, first on PATH; set
`TMPDIR=/scratch/code/shibboleth/.tmp`. Revert every mutation; remove the
worktree when done.

## Already settled — do not re-derive
- `gui` is 1294/1294 across 24 shards, partition verified exhaustive
  (`/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`).
- `go vet` is clean beyond the known TinyGo go-directive gap (10 occurrences).
- Firmware cost measured: 1,644,588 -> 1,645,052 flash, RAM unchanged.
- Two mutations already run and RED: preselect forced to row 0; `Choose`
  ignoring `Initial` (which also reds `TestBackPreservesEnteredValues`).
- Secret-handling never gates.

## Construct counterexamples — an assessment without one is Minor at most
1. **The 85 call sites.** `ChoiceScreen` gained `Initial int`, claimed safe
   because its zero value is row 0. Enumerate the call sites and find one where
   that claim is false — a composite literal that would now behave differently,
   a struct copied after `Choose`, a screen value reused across two DIFFERENT
   choice lists where `seeded` then suppresses the second list's intended
   opening row. The `seeded` guard is the newest thing here and the least
   exercised; attack it specifically.
2. **The six migrated pokes.** `cs.choice = X` became `cs.Initial = X` at
   `freetext_flow.go` 540/774/827/898/917 and `passphrase_flow.go` 406. For each,
   prove the two are equivalent AT THAT CALL SITE or show where they diverge.
   The old poke set the live selection; the new field is seeded inside `Choose`.
   Is there a site where something reads `choice` between the assignment and the
   `Choose` call, or where `Choose` is never reached?
3. **The sized/unsized QR screen.** `ftQRChoiceFlow` deliberately does NOT carry
   the prior opt-in when `sized` — its comment says so. Confirm the migration
   preserved that asymmetry and did not make the sized branch inherit an
   `Initial` from anywhere.
4. **Does the fix actually fix it.** Drive the composer and confirm the reported
   sequence — open `Change the script`, confirm without moving — now leaves the
   wrapper alone, on BOTH the path-list row and the Back leg. Then confirm a
   deliberate change still works and still discards seats and sources.
5. **The two edited tests.** `TestComposerChangeTheScriptRowRewrapsAndDiscards`
   and `TestComposerBackLegWrapperChangeAsksBeforeDiscardingSeats` each gained a
   `click(Up)`. Confirm each still fails if its own guarantee is broken — mutate
   the guarantee, not the navigation — and that the added Up did not turn either
   into a test that passes for a new reason.

## Severity
Critical: a picker that returns a row the operator did not choose; a
reintroduction of the C-1 defect on any path; a test that cannot fail on what it
names. Important: a behaviour change at a call site the diff did not intend, an
unsound assumption, a missing case. Minor/Nit: wording, records.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-script-preselect-review.md`
(create; must not exist): findings as `### C-n / I-n / M-n / N-n — title` with the
command, verbatim output and the line violated; a table of every `ChoiceScreen`
call site you judged non-trivial with its verdict; the mutation table; closing
counts and GREEN / NOT GREEN. Return a two-line summary plus the path.
