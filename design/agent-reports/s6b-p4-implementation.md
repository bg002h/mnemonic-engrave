# S6b P4 implementation report — the restore document (spec §6/§6.1)

**Worktree:** `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`.
**Commit:** `639e1b2` — "S6b P4 (6/6a/6.1): the restore document conditions on CUT, not offered".
**Gates:** 6, 6a, 6.1 (spec `SPEC_s6b_pre_flash_cycle.md` §6/§6.1).

## What changed, and where

**The defect.** `buildPassphraseInventoryLines` (`gui/multisig_build_census.go`)
printed, unconditionally when a passphrase was used, *"nothing this device
engraves carries a passphrase"*. S6b P3's passphrase-plate offer
(`engraveSingleSigFlow` → `singleSigPassphrasePlateOffer`) can falsify that on
the exact run where it fires, minutes before the document is shown.

**Mechanism (gui/passphrase_flow.go, gui/singlesig.go).** Added
`passphrasePlateResult` (`passphrasePlateNotCut` / `passphrasePlateCut`),
following the `bundleEngraveResult` idiom already beside it.
`engravePassphraseFlowPreloaded` and `singleSigPassphrasePlateOffer` now
return this instead of `void`. Every exit before
`NewEngraveScreen(...).Engrave(...)` returns `true` — declined offer, Back at
any step, `ctx.Done` — collapses to `passphrasePlateNotCut`. Only the
accepted-and-engraved plate returns `passphrasePlateCut`. `engraveSingleSigFlow`
captures the result in `plateResult` and passes `plateResult ==
passphrasePlateCut` into `buildPlateInventoryLines`.

**Document text (gui/multisig_build_census.go).**
`buildPassphraseInventoryLines(seeds []seedPassphraseFact, passphrasePlateCut
bool)` and `buildPlateInventoryLines(..., passphrasePlateCut bool)` gained the
new parameter. When `passphrasePlateCut` is true, the passphrased arm's two
lines become:

> *"A BIP-39 passphrase WAS used. It is not on these plates: this device also
> cut a separate passphrase plate, engraved this run."*
> *"That passphrase plate is not one of the plates listed above and is not
> counted in this backup. Keep it somewhere separate, and make sure whoever
> needs this backup can also get it."*

This does both gate 6 (retracts the false "nothing carries a passphrase"
claim) and gate 6.1 (says the plate is not counted, and repeats R1's "keep it
somewhere separate" verbatim) in one place, since spec §6.1 frames both as one
instruction applied twice. When `passphrasePlateCut` is false, the shipped two
lines render byte-identical to what shipped before S6b (pinned by
`TestRestoreDocUnchangedWhenPlateNotCut`).

**Callers.** `gui/multisig.go` and `gui/multisig_build.go` pass `false`
unconditionally (both commented `R-B, a later phase`) — neither multisig path
has a passphrase-plate offer to have cut one from. **No `bundleCard` is added
anywhere** for the passphrase plate, so `bundlePlatePlan`/`len(plan)` and the
"If any of them is missing" line are structurally unaffected — confirmed by
diff (`git status --short` touches only the 10 `gui/*.go` files listed below,
no card-construction sites) and by test, not merely by assertion.

**Files touched:** `gui/passphrase_flow.go`, `gui/singlesig.go`,
`gui/multisig_build_census.go`, `gui/multisig.go`, `gui/multisig_build.go`,
plus 4 pre-existing test files whose calls needed the new bool argument
(`gui/multisig_build_perseed_passphrase_test.go`,
`gui/multisig_build_prose_test.go`, `gui/multisig_supply_passphrase_test.go`,
`gui/singlesig_truth_test.go`), and one new test file,
`gui/s6b_restore_doc_test.go`.

## TDD, per gate

**RED (compile failure), captured before any production edit landed** — with
the test-file call sites already updated to the new 4-arg signature but the
production code still at the old 3-arg one:

```
vet: gui/multisig_build_perseed_passphrase_test.go:134:81: too many arguments in call to buildPlateInventoryLines
	have ([]bundleCard, []seedPassphraseFact, seedCapacity, bool)
	want ([]bundleCard, []seedPassphraseFact, seedCapacity)
```

(Reproduced live via `git stash` on just the 5 production files, confirming
the RED state against pre-P4 code, then `git stash pop` before continuing.)

**GREEN, 6 new tests in `gui/s6b_restore_doc_test.go`:**

| test | gate | what it proves |
| --- | --- | --- |
| `TestRestoreDocSaysPassphrasePlateWasCutSeparately` | 6 | cut=true drops the false claim, names a separate plate |
| `TestRestoreDocUnchangedWhenPlateNotCut` | 6a | cut=false renders the shipped two lines byte-for-byte (covers declined AND aborted at once, since both collapse to the same bool) |
| `TestPassphrasePlateNotCountedInBackupSet` | 6.1 | the census line and per-card block are unaffected by cut; the passphrase plate is named only after "If any of them is missing", on a line containing "not counted in this backup" and "somewhere separate" |
| `TestSingleSigPassphrasePlateOfferNotCutStates` | 6a wiring | `singleSigPassphrasePlateOffer` itself returns `passphrasePlateNotCut` for no-passphrase and for a declined offer (direct calls, cheap) |
| `TestEngravePassphraseFlowPreloadedAbortReturnsNotCut` | 6a wiring | backing out of the preloaded flow after accepting the offer (Back at the entry step) also returns `passphrasePlateNotCut` |
| `TestRestoreDocReflectsARealCutPassphrasePlate` | 6/6.1 end-to-end | drives the REAL `engraveSingleSigFlow` through an actual passphrase-plate engrave (not a hand-built call) and reads the final restore-document screen |

All 6 pass:

```
=== RUN   TestRestoreDocSaysPassphrasePlateWasCutSeparately
--- PASS: TestRestoreDocSaysPassphrasePlateWasCutSeparately (0.00s)
=== RUN   TestRestoreDocUnchangedWhenPlateNotCut
--- PASS: TestRestoreDocUnchangedWhenPlateNotCut (0.00s)
=== RUN   TestPassphrasePlateNotCountedInBackupSet
--- PASS: TestPassphrasePlateNotCountedInBackupSet (0.00s)
=== RUN   TestSingleSigPassphrasePlateOfferNotCutStates
    --- PASS: TestSingleSigPassphrasePlateOfferNotCutStates/no_passphrase (0.00s)
    --- PASS: TestSingleSigPassphrasePlateOfferNotCutStates/offer_declined (0.00s)
=== RUN   TestEngravePassphraseFlowPreloadedAbortReturnsNotCut
--- PASS: TestEngravePassphraseFlowPreloadedAbortReturnsNotCut (0.00s)
=== RUN   TestRestoreDocReflectsARealCutPassphrasePlate
    watch-only mode cut 5 base plate(s)
    restore doc: ... "This backup is 5 plates: mk1 key: 2 plates ... md1
    descriptor: 3 plates ... If any of them is missing, this backup is
    incomplete. ... A BIP-39 passphrase WAS used. It is not on these plates:
    this device also cut a separate passphrase plate, engraved this run.
    That passphrase plate is not one of the plates listed above and is not
    counted in this backup. Keep it somewhere separate, and make sure
    whoever needs this backup can also get it. ..."
--- PASS: TestRestoreDocReflectsARealCutPassphrasePlate (26-28s across repeated runs)
```

**Cost tradeoff, disclosed:** `TestRestoreDocReflectsARealCutPassphrasePlate`
drives a real 6-plate engrave (5 watch-only base plates + 1 passphrase plate,
via the raster harness). It reuses
`TestPassphrasePlateOfferReachableFromTheOrchestrator`'s exact walk up to the
offer (already paid for in the P3 test, ~24.9s) and adds one more real engrave
plus a few screens, measured at 26–28s across repeated runs — roughly 1–3s
more than the walk already in the suite. I chose to pay this rather than rely
only on unit-level `buildPassphraseInventoryLines` assertions, because the
untested risk was specifically whether `engraveSingleSigFlow`'s own 3-line
call site correctly threads the real "cut" bool through — the exact "a
swapped argument compiles, renders, and looks entirely healthy" shape this
codebase's own `T7c` test (`gui/singlesig_truth_test.go`) was written to
guard against for a different bool. I judged this worth the ~26s given gate
6's funds-adjacent nature (a false safety claim on the one artifact meant to
outlive everyone); a cheaper alternative would have been to trust code
reading alone for that one call site.

**Regression pins, GREEN throughout (not red-then-green):** existing tests
that build documents on the multisig paths — `TestSingleSeedInventoryIsUnchanged`,
`TestRestoreDocNamesEveryPassphrasedSeed`,
`TestRestoreDocSaysWhichSeedsNeedNoPassphrase`,
`TestSupplyRestoreDocSaysSoWhenNoPassphraseWasUsed`,
`TestSingleSigBareRunDoesNotCryWolf`,
`TestSeedResidencyRulingDescribesTheMultiSeedReality` — all still print the
pre-S6b text, because those callers always pass `false`. Also extended the
existing glyph-safety sweep in `TestSeedHandlingRulingIsKeyedOnCapacityAndOnThePlates`
(`gui/singlesig_truth_test.go`) with a new `plateCut ∈ {false, true}`
dimension, so the new prose is covered by that invariant too.

## Full suite — run once, blocking, stdout/stderr separated

```
export PATH="/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH"
go build ./...                 # clean, no output
go vet ./gui/                  # only: gui/freetext_sizeproof_golden_test.go:111:13:
                                #   testing.ArtifactDir requires go1.26 or later (file is go1.25)
                                #   -- pre-existing, not mine (per plan)
go test ./... -count=1 > stdout.log 2> stderr.log   # blocking, waited for completion
```

`stderr.log` was **empty**. `stdout.log`: every package reports `ok`, **zero**
`FAIL`, zero panics. Full per-package list checked; the relevant line:

```
ok  	seedhammer.com/gui	496.782s
```

496.782s against Go's 600s per-package default (~83%). This is higher than
the ~430–450s baseline the brief quoted for P1–P3's state, but still green
with real margin. The increase is consistent with this phase's ~26–28s new
E2E test plus normal run-to-run variance; I did not chase down the exact
delta further since the result is unambiguously green and the margin (over
100s) is not tight.

`go vet ./...` (whole repo, for completeness) shows the same 2 `gui/`
failures plus others in `bspline`, `engrave`, `backup` — all the same
pre-existing go1.26 `t.ArtifactDir()` class, in files I never touched. The
`backup` package specifically shows 2 (`backup_test.go`, `freetext_test.go`),
matching the plan's "two in gui, two in backup" note exactly.

## No golden moved

`git status --short` after the full suite run touches only the 10 `gui/*.go`
files in this commit (9 modified + `gui/s6b_restore_doc_test.go` new).
`backup/testdata/` and `gui/testdata/` show zero changes. P4's own gate table
expected none.

## Grep sweep for the sentences touched (per the brief's standing instruction)

- **`"nothing this device engraves carries a passphrase"`** — exactly one
  production emission site, `gui/multisig_build_census.go` (the
  `passphrasePlateCut == false` arm). No second rendering anywhere in the
  fork (`grep -rn --include="*.go" .` from the repo root).
- **`"A BIP-39 passphrase WAS used"`** — two production sites, both inside
  the same function I edited (the cut/not-cut arms of one `if`) — intentional,
  not a duplication defect.
- **`"Keep it somewhere separate" / "somewhere separate"`** — three
  production occurrences, all in `gui/multisig_build_census.go`: the
  not-cut arm (unchanged), the cut arm (new, per spec §6.1's instruction to
  repeat it verbatim), and `buildFullModeLabel`'s doc comment (prose, not a
  render site). No occurrence outside this file.
- **`"If any of them is missing, this backup is incomplete."`** — exactly one
  production emission site, `gui/multisig_build_census.go:78`
  (`buildPlateInventoryLines`), shared by all three restore-document flows.
  Every other hit across the tree is a *comment* quoting the sentence for
  context (`gui/singlesig.go`, `gui/multisig_build.go`,
  `gui/multisig_engrave_tail_walk_test.go`,
  `gui/multisig_verify_report_test.go`, `gui/singlesig_truth_test.go`), not a
  second rendering.

No second site found for any of the four. `gui/multisig_restore.go:97`'s
comment referencing "buildPlateInventoryLines' passphrase arm" is generic
prose describing the mechanism, not a quote of the specific sentence, and
needed no update.

## Spec ambiguity / what the spec left open

Spec §8 explicitly lists *"The exact wording of ... §6's conditional
clause"* as **not settled** by the spec — the wording choice above (the two
replacement lines) was made in this implementation, not dictated by the spec.
I read "the document says so and says where it is" (§6) as: names the plate's
existence and states it is a plate *separate* from the ones enumerated above
(the device cannot know a physical storage location, so "where it is" reads
as "not among these," not a literal address) — combined with §6.1's own
"repeats the separation instruction" requirement into one two-line block
rather than three, on the grounds that §6.1 itself frames both jobs as "one
instruction applied twice." Flagging this reading explicitly since it is a
judgment call the spec deferred rather than a defect found.

## Prohibitions honored

- Condition is **CUT**, never "offered" — verified by
  `TestEngravePassphraseFlowPreloadedAbortReturnsNotCut` and
  `TestSingleSigPassphrasePlateOfferNotCutStates` (declined case), both
  distinct from a completed cut.
- **No `bundleCard`** added for the passphrase plate anywhere (`git diff |
  grep bundleCard{` shows no new construction site).
- **`len(plan)` unchanged** — `TestPassphrasePlateNotCountedInBackupSet`
  compares the census line byte-for-byte between cut=true/false, and the E2E
  test confirms the real document names exactly the base-plate count with no
  +1.

## Nothing I could not do

Nothing in scope was blocked. One pre-existing, unrelated `gofmt` quirk was
noticed in `gui/singlesig.go` (a doc comment a few lines above the code I
touched renders oddly under `gofmt -d`, apparently a pre-existing Unicode
artifact in a comment, not something `gofmt -w` would rewrite in my diff) —
confirmed present at HEAD before this phase's changes, so left alone as
out of scope, same posture as the pre-existing `go vet` `t.ArtifactDir()`
failures.
