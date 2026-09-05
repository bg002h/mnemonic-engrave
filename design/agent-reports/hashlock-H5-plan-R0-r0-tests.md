# Hashlock H5 plan — R0 round 0, tests/mutation review (sonnet)

**Scope.** Independent tests/mutation reviewer for
`design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` (engrave master
`0c2b13e`), spec `design/SPEC_hashlock_H5_device_polish.md` R0 GREEN `e03d8e7`.
ONE QUESTION: can every test the plan adds actually FAIL on the defect it
names, does every RED and every declared `MUTATION:` reproduce when run from
the plan's own text, and which mutations of the new guards survive.

**Method.** Read the whole plan (2,698 lines) and the plan-author's report
verbatim. Diffed the fork's read-only pristine checkout
(`/scratch/code/shibboleth/seedhammer`, main `b9a9a30`, never modified) against
the gated tree (`/scratch/code/shibboleth/.tmp/h5-gate`, never modified) to
identify the exact per-task hunks in every file the plan touches, including
the two files three tasks share (`gui/composer_hashlock.go`,
`gui/composer_copy.go`). Worked exclusively in my own copy
(`/scratch/code/shibboleth/.tmp/h5-tests`); after every RED/mutation the file
was restored from the gate tree and the restoration verified
(`diff -rq h5-gate h5-tests` returns clean before the final run). Go
1.26.7 at `/scratch/code/shibboleth/.toolchain/go/bin/go`. No sub-agents, no
`.jsonl` read, nothing committed anywhere.

## RED quotes (all six task boundaries)

**Task 1** (revert `composerNotePhraseDigest`/`composerAnyPathByPhrase`/
`phraseDigests`, `composerHashByPhraseSync` and both call sites, and the §8h
sentence change; keep Tasks 2/3's hunks in the shared files):

```
gui/composer_copy_test.go:167:2: undefined: composerNotePhraseDigest
gui/composer_hashlock_test.go:712:3: undefined: composerNotePhraseDigest
gui/composer_hashlock_test.go:730:6: undefined: composerAnyPathByPhrase
gui/composer_hashlock_test.go:928:6: undefined: composerAnyPathByPhrase
gui/composer_hashlock_test.go:1116:3: undefined: composerNotePhraseDigest
gui/composer_hashlock_test.go:1128:6: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:35:8: st.phraseDigests undefined (type *composerState has no field or method phraseDigests)
gui/composer_provenance_test.go:70:17: st.phraseDigests undefined (type *composerState has no field or method phraseDigests)
gui/composer_provenance_test.go:71:80: st.phraseDigests undefined (type *composerState has no field or method phraseDigests)
gui/composer_provenance_test.go:73:6: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:73:6: too many errors
FAIL	seedhammer.com/gui [build failed]
```
Same defect class as the plan's quote (build fails on the removed
identifiers); see **Finding M-1** below on the quote itself.

**Task 2** (revert `composerCopyHashlockReconcile`'s signature/body and the
confirm write-down sentence; keep Task 1's hunks):

```
gui/composer_copy_test.go:141:77: too many arguments in call to composerCopyHashlockReconcile
gui/composer_hashlock_test.go:992:39: too many arguments in call to composerCopyHashlockReconcile
gui/modal_fits_test.go:344:34: too many arguments in call to composerCopyHashlockReconcile
FAIL	seedhammer.com/gui [build failed]
```
Same defect class as quoted; see Finding M-1.

**Task 3.** The plan states plainly that "the pre-fix state IS the first
mutation, so RED and the mutation table are the same evidence" — verified true
by inspection (the geometry test file is created only after `composerTextBand`
and `hashlockPhraseLead` already exist as functions; there is no
build-failure RED for this task). Its RED is mutation 1 below.

**Task 4** (revert `unlockNotPermittedBody` to the pre-H5 one-sentence form):

```
--- FAIL: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable (0.13s)
    unlock_preimage_test.go:69: the screen must say what to do next; got "Record1isahashlockpreimage,notaseed.Thispayloadcannotbeunlockedhere.Nothingwasopened.SealedPayload"
    unlock_preimage_test.go:76: the screen must say the index is 0-based; got "Record1isahashlockpreimage,notaseed.Thispayloadcannotbeunlockedhere.Nothingwasopened.SealedPayload"
--- FAIL: TestUnlockNotPermittedBodyNamesTheRecordAndTheKind (0.18s)
    [4 of 5 table rows] does not carry "Remove that record (records count from 0) on the host and seal the payload again."
```
Exact match, including the nuance that the 5th row ("a preimage plate at
record 0") correctly stays PASS since its own `want` list is
`["Record 0", "hashlock preimage"]` and never asserts the new sentence — this
is not a gap, it is that row's purpose (naming record 0 itself).

**Task 5, pre-existing RED** (run directly on the **read-only pristine**
`b9a9a30` checkout, no file touched):

```
$ cd /scratch/code/shibboleth/seedhammer && CGO_ENABLED=0 go test -count=1 ./cmd/emu/
--- FAIL: TestWalkOkContainsNoDriverSuppliedPlateCount (0.02s)
    needle_test.go:525: INCONCLUSIVE: walk_h0_preimage.js has no `ok:` property this test can read...
    needle_test.go:525: INCONCLUSIVE: walk_hashlock_phrase.js has no `ok:` property this test can read...
    needle_test.go:563: 6 walk script(s) checked; no driver-supplied plate count in any `ok`
FAIL	seedhammer.com/cmd/emu	4.235s
```
Exact match to the plan's claim that this predates H5 (timing differs, 4.2s
vs. the plan's 1.1s — machine variance, not a discrepancy).

After the fix (final tree), the guard reports exactly what the plan claims:
```
needle_test.go:554: walk_hashlock_phrase.js sets `ok` to true after its last assertion, so it restates nothing (H5 §4.4)
needle_test.go:606: 8 walk script(s) checked; no driver-supplied plate count in any `ok`
--- PASS: TestWalkOkContainsNoDriverSuppliedPlateCount (0.00s)
```

## Mutation table

Every declared `MUTATION:` in the plan was applied (in my own copy, restored
after each), run, and its printed failure compared verbatim against the
plan's quote. All 20 declared mutations reproduced exactly; none survived
(none stayed GREEN when it should have failed). Four extra mutations of my
own (marked **own**) were also run.

| # | Mutation | Caught by | Quoted assertion (my run) |
| --- | --- | --- | --- |
| T1-1 | `composerNotePhraseDigest` assigns without the nil check | `TestComposerPhraseRouteHoldsOnTheZeroValueState` | `panic: assignment to entry in nil map [recovered, repanicked]` at `composer_provenance_test.go:66` through `holdConfirm` — exact match |
| T1-2 | `composerAnyPathByPhrase` returns `len(st.phraseDigests) > 0` | `TestComposerHashEditDispatchesByRowLabel/none_row_clears...`, `TestRemovePathThenAHexHashDrawsThePlainBanner`, `TestComposerAnyPathByPhraseIsPerDigest/the_phrase_path_was_edited_to_a_payload_row`+`.../the_phrase_path_was_removed`, `TestComposerHashEditToAPayloadRowDropsThePhraseForm` | all 4/6 failures reproduced verbatim; `TestComposerMixedWalletBannerNamesEveryPhraseAndEveryPlate` correctly stays PASS (both paths in that fixture keep intact phrase-set hashes, so `len(set)>0` still agrees with the real predicate) |
| T1-3 | delete `composerNotePhraseDigest(st, d)` from `hashlockPhraseRoute` | `TestHashlockReconcileScreenIsReachableOnAMixedPolicy`, `TestComposerPhraseRouteHoldsOnTheZeroValueState` | both FAIL, exact match |
| T1-4 | restore "Back up the phrase and its method, or the preimage plate, separately." | `TestComposerCopyIsVerbatimFromTheSpec` (both §8h rows), `TestComposerMixedWalletBannerNamesEveryPhraseAndEveryPlate` | `does not carry "every phrase and its method"`, `does not carry "every preimage plate"`, `still offers a CHOICE of backups` — exact match |
| T2-1 | return the old one-sentence reconcile body | `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars`, `TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal` | `does not carry "hash  3cf5d421..b70a4c12"`, `...method: hardened   chars: 28`, `...If they differ...`, `does not open with the shared header` — exact match |
| T2-2 | drop the mismatch sentence only | `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars`, `TestComposerCopyIsVerbatimFromTheSpec` | `does not carry "If they differ, do not fund this wallet: build it again."`; `composerCopyHashlockReconcile (SPEC §H2-4.5) does not match the spec.` — exact match |
| T2-3 | restore "Write down this phrase and the method now." | `TestHashlockPhraseRouteSetsTheCorpusDigest` (both cases), `TestComposerCopyIsVerbatimFromTheSpec` | `the confirm modal's write-down line does not name the digest`, `composerCopyHashlockConfirm (SPEC §H2-4.5) does not match the spec.` — exact match |
| T2-4 | `"method: %s chars: %d"` (one space) | `TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal` | FAILS as claimed; **and** `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars` independently confirmed to **stay PASS** under this mutation (verified, not assumed) — `normalizeDrawn` strips whitespace, matching the plan's own false-PASS-awareness note |
| T3-1 | `hashlockPhraseLead` wraps at `dims.X-2*8`, centres on panel | `TestHashlockPhraseLeadIsDrawnInsideTheBand` | `button (427,44)-(480,97) received ink at (431,52)`, `lead (440,44) = 2 line(s)` — exact match, same pixel values |
| T3-2 | `composerTextBand`'s `right` drops the nav column | `TestHashlockPhraseLeadIsDrawnInsideTheBand`, `TestComposerPagedLinesNeverDrawUnderTheNavButtons` | full verbose output confirms exactly the three named subtests: `keyed stub page 0`, `keyed stub page 1`, `keyless stub page 0` — exact match |
| T3-3 | restore `content, _ = content.CutBottom(8)` in `hashlockPhraseFlow` | `TestHashlockPhraseScreenKeepsTheReadoutBudget`, `TestHashlockPhraseScreenDrawsTheMaskedReadout` | `MaxHeight=201 grid=(340,182) gap=8 -> readout budget 11 px; one line is 19 px`; `the phrase screen drew 0 asterisks for 10 typed characters` — exact match |
| T4-1 | drop the whole new sentence | `TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable`, `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind` | both fail as claimed (see RED quote above, identical text) |
| T4-2 | drop `"(records count from 0)"` only | both of the above | `got "...Removethatrecordonthehostandsealthepayloadagain.SealedPayload"` and all 4 rows `does not carry "Remove that record (records count from 0)..."` — exact match |
| T5-1 | delete `clearComposerStateHook()` from `composerFlowExit` | `TestComposerStateHookIsInstalledOnlyWhileAFlowRuns` | `the hook survived the composition it was installed for: []` — exact match |
| T5-2 | delete `setComposerStateHook(st)` from `composerFlow` | same test | `the hook is not installed while composerFlow is running` — exact match |
| T5-3 | hook hands out `st`'s own pointers (`out[i] = p.Hash`) | `TestComposerStateHookReportsEachPathAndHandsOutCopies` | `writing through the hook's pointer changed the POLICY: ff0102...1e1f, want 0001...1e1f` — exact match |
| T5-4 | hook skips nil-hash paths instead of leaving a hole | same test | `the hook reports 1 entries for a 2-path composition` — exact match |
| T5-5 | `composer_state_hook_tinygo.go` exports anything | `TestBuildTaggedHooksAreAbsentFromTheFirmwareImage` | `composer_state_hook_tinygo.go exports ComposerPathHashesOnDevice -- that file IS the firmware...` — exact match |
| T5-6 | `ComposerPathHashes` named in another `gui` file | same test | `composer_flow.go uses ComposerPathHashes in code but is not composer_state_hook.go` — exact match |
| **own** | swap `m.String()`/`len(phrase)` at the reconcile call site (Task 2's third named mutation, not previously exercised) | `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars` | `does not carry "method: hardened   chars: 28"` — reproduces as the plan's comment predicts |
| **own** | reproduce the author's *self-reported* false PASS: revert `TestComposerStateHookReportsEachPathAndHandsOutCopies`'s `want := d` snapshot to compare directly against `d`, **with** the T5-3 pointer-copy mutation applied | (none — false PASS) | **PASS** — confirms the author's report is accurate: the first-draft test shape really was blind to this mutation, and the shipped `want := d` snapshot really does fix it (T5-3 above, run against the shipped test, is RED) |
| **own** | stress-test the *new* `ok`-assignment-form guard itself: a synthetic `walk_zzz_test_probe.js` with `out.ok = state.plates > 0;` (temp file, removed immediately after, tree verified clean via `diff -rq` against the gate tree) | `TestWalkOkContainsNoDriverSuppliedPlateCount` | `walk_zzz_test_probe.js's \`ok\` contains \`plates\`, which the CALLER supplies (I-1/F-170)` — the new assignment path preserves the guard's original I-1/F-170 protection; it is not widened to a blind exemption |

**All 20 declared mutations plus 4 of my own reproduced exactly as claimed or
as their own logic predicts. Zero survived.**

## False-PASS hunting

1. **Does any test recompute its expectation with the code under test?** No.
   Every `composerCopyTable()` row's `want` string is a hand-typed literal
   (checked `composer_copy_test.go`'s reconcile/confirm/§8h rows and
   `TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal`'s `head` variable,
   which is built from `"hash  " + tok + ...` literals, never by calling
   `composerCopyHashlockReconcile`/`Confirm`). One **already-fixed** instance
   of this class was found and independently reproduced: see the "own"
   mutation row above (`want := d` vs. comparing directly against `d`).
2. **Does the geometry test measure ink or only layout arithmetic?**
   Confirmed real rasterization. `inkUnderNavOps`
   (`gui/composer_paged_geometry_test.go:74`) draws into a real `rgb565`
   framebuffer via `op.Drawer.Draw` and samples actual pixels against a
   zero-value "blank" reference — it is not rectangle-overlap arithmetic.
3. **Do the fit rows drive the renderer production uses?** Yes.
   `errorScreenBody` (`gui/modal_fits_test.go:113`) calls `showError` directly
   — the exact function `hashlockPhraseRoute`'s Task 2 call site and
   `unlockNotPermittedBody`'s caller both go through in production. Confirmed
   by reading `slip39_polish.go:36`'s `showError` and its two callers.

## Machine-checked whole-tree state (final GREEN tree)

- `diff -rq h5-gate h5-tests` returns clean before this run (no leftover
  mutation from the above).
- 24-shard `gui` run: **1236 top-level tests, `partition verified exhaustive:
  1236 == 1236`, 24/24 shards ok, 38s wall** (plan: 31s; timing only).
- `CGO_ENABLED=0 go test -timeout 20m ./...` (CI's exact command): **55
  packages `ok` (counted, not eyeballed: `grep -c '^ok'`), 0 `FAIL` lines.**
- `go test ./gui/ -list '.*' | grep -cE '^(Test|Example|Fuzz)'` on the
  **pristine, unmodified** `b9a9a30` checkout: **1225** — matches the plan's
  cited baseline exactly.
- `node -e "import('./walk_hashlock_phrase.js')..."` → `MODULE PARSES + LOADS
  function` — matches the author's report.
- `GOOS=js GOARCH=wasm go vet ./cmd/emu/` → clean (exit 0).
- `TestComposerCopyTableCoversEveryBody` PASSES and the `declared != 53`
  literal is confirmed unmoved (`grep -n "declared != 53"
  gui/composer_copy_test.go`).
- Fit-gate numbers independently reproduced, exact: 186/339 (reconcile),
  165/378 (§8h phrase form), 347/107 (confirm modal) for Task 2; 152/397,
  140/418, 153/397 for Task 4's rows; §3.2(b)'s unmutated `MaxHeight=209
  grid=(340,182) gap=8 -> readout budget 19 px; one line is 19 px` (zero
  slack, as claimed).

## Findings

**M-1 (Minor).** The plan's "Measured:" RED quotes for Task 1 Step 4 and Task
2 Step 3 are **incomplete transcriptions** of what the literal command
produces against the tree state the plan itself describes at that point.
Task 1's quote shows only `composer_provenance_test.go` errors (6 lines);
re-running the exact command against a tree with Task 1's own Steps 1–3
already applied (as the plan's own step order requires) also surfaces
`composer_copy_test.go:167` and three `composer_hashlock_test.go` lines before
Go's 10-error cap is hit — all from files the SAME task edits in the SAME
step sequence. Task 2's quote (2 lines) similarly omits a third,
`composer_copy_test.go:141:77`, from its own Step 1 row edit. In both cases
the omission does not change the substantive claim — the build still fails
for the reason given (the reverted identifier/signature) — so this does not
block. It means the plan's quoted evidence is not a verbatim capture of a
run against its own described tree state; a reviewer should not assume a
"Measured:" block enumerates every line a rerun produces. Reproduction:
diff-derived per-task reverts are recorded in this review's own working
notes; the two commands are exactly those printed at each task's "Step N:
RED" heading.

No Critical or Important findings. Every RED reproduces the claimed defect
class; every declared and self-constructed mutation reproduces or behaves as
its own stated logic predicts; no false PASS was found beyond the one the
plan's own author already caught and fixed (independently re-confirmed
above); the new `ok`-assignment guard was stress-tested against its stated
purpose (I-1/F-170) and holds. Task 6 carries no executable content (all
blocks unheaded, confirmed by re-reading the plan) and is outside this
review's remit.

## Closing counts

- RED steps reproduced: **5 of 5** applicable (Task 3 has none separate from
  its mutation 1, as the plan itself states and this review confirms).
- Declared `MUTATION:` entries: **20 of 20** reproduced, **0 survived**.
- Own mutations run: **4** (arg-swap, false-PASS re-reproduction, ok-guard
  stress test, plus the ones already listed above as part of the declared
  set where the brief named them) — **0 survived** in the sense of passing
  when they should fail; the one deliberately-reverted false-PASS (T5-3
  without the `want` snapshot) reproduces the **already-fixed** defect and is
  not present in the shipped test.
- False-PASS hunt: **1 pattern found**, already caught and fixed by the plan
  author, independently re-verified here; **0 new** false PASSes found.
- Whole-tree gate: **GREEN** (1236/1236 gui tests across 24 shards; 55/55
  packages via `go test ./...`; 0 FAIL anywhere).
- Blocking findings: **0 Critical, 0 Important.** One Minor (M-1, quoted-RED
  completeness) recorded above.
