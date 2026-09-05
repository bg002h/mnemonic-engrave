# hashlock H2 post-implementation fold — R2 fold-verification (sonnet, independent)

**Scope.** Verify the SECOND hashlock H2 post-implementation fold: fork commit
`a1fd139` over `26fd1dd` (branch `hashlock-h2`), and engrave records commit
`e879123` over `67f9fe9` (branch `hashlock-h2`). Own detached worktree:
`/scratch/code/shibboleth/.tmp/h2-fold-verify-2` (removed at the end of this
run). Read-only on both branch worktrees (`.tmp/seedhammer-hashlock-h2`,
`me-worktrees/hashlock-h2`) — nothing was modified there. No sub-agents. No
`.jsonl` read.

**Verdict: GREEN.** Every claim in the fork commit message and the engrave
records reproduced exactly when executed. Both named mutations fail as
predicted and revert clean. Both end-to-end scenarios (three-hashlock other-path
line; Remove-then-hex-hash reaching Done) reproduce on the real harness, not
just via direct function calls. All eight refute-pass CONFIRMED/PARTIAL
verdicts map to a commit or a filed follow-up; none unaddressed. No new defect
found.

---

## 1. `git diff 26fd1dd..a1fd139 --stat`, read whole

```
 gui/composer_copy.go          |  7 ++++--
 gui/composer_copy_test.go     |  4 ++--
 gui/composer_hashlock_test.go | 53 +++++++++++++++++++++++++++++++++++++++++--
 gui/composer_shape.go         |  3 +++
 4 files changed, 61 insertions(+), 6 deletions(-)
```

Read in full. Two production changes, exactly as the fold commit message
states:

1. `gui/composer_copy.go`: `composerCopyHashlockOtherPath()` now returns
   `"another path has a different hash: back up every phrase"` (count-free),
   replacing the hard-coded `"...two phrases to back up"`.
2. `gui/composer_shape.go`: `composerPathEdit`'s Remove arm (case 3) now calls
   `composerHashByPhraseSync(st)` after splicing the path out.

`gui/composer_copy_test.go` and part of `gui/composer_hashlock_test.go` update
existing assertions to the new string. The rest of `composer_hashlock_test.go`
adds: (a) a three-other-differing-hashes row plus a no-digit assertion inside
`TestHashlockOtherPathLineIsSilentOnAnEqualHash`; (b) a new test,
`TestRemovePathReSyncsHashByPhrase`.

## 2. Mutation 1 — restore the hard-coded count

Edited `composerCopyHashlockOtherPath()` back to
`"another path has a different hash: two phrases to back up"`:

```
composer_hashlock_test.go:958: the other-path line carries a count: "another path has a different hash: two phrases to back up"
--- FAIL: TestHashlockOtherPathLineIsSilentOnAnEqualHash
```

Exactly the no-count assertion, exactly as the brief predicted. Reverted;
`git diff --stat` empty afterward.

### Three-hashlock shape on the real harness

Built a scratch test (`gui/zz_verify_e2e_test.go`, not committed, deleted
before the whole-suite gate run) driving a REAL 4-path composition through
`runComposerHashEdit` — paths 0/1/2 pre-set to three DIFFERENT digests, path 3
taking the anchor phrase through the phrase route (typed keyboard, method pick,
brainwallet warning, hold-confirm) — and read the drawn confirm-modal body:

```
CONFIRM MODAL BODY: "hashb867db87..edbc96cbmethod:sha256chars:28anotherpathhasadifferenthash:backupeveryphraseWritedownthisphraseandthemethodnow...."
```

`anotherpathhasadifferenthash:backupeveryphrase` — the count-free line,
reached through the real UI (typed phrase, real method pick, real hold), not
merely via a direct `hashlockOtherPathLine` call. Matches
`composerCopyHashlockOtherPath()` byte-for-byte (module's own `uiContains`
convention drops inter-word spaces).

## 3. Mutation 2 — delete the resync call

Removed `composerHashByPhraseSync(st)` from `composerPathEdit`'s Remove arm:

```
composer_hashlock_test.go:1038: the only phrase-set hash was removed and st.hashByPhrase is still true
--- FAIL: TestRemovePathReSyncsHashByPhrase
```

Exactly as predicted. Reverted; `git diff --stat` empty afterward.

### Interruption M-1 scenario, end to end

Constructed the scenario named in the brief on the real harness (same scratch
file). Two structural notes on how it had to be built, both forced by
constraints elsewhere in the codebase, not by anything in this fold:

- `md.ValidatePathList` refuses any composition with **zero** keyed paths
  ("Every wallet needs at least one path with a key"), so a key-less-only
  policy (the shape the lens's own *pinned* test used) can never itself reach
  `composerShapeFlow`'s Done branch. A mixed policy — `composerTwoPathList()`
  (2-of-3 keyed + 1-of-1 keyed) — was used instead, with path 0 also given a
  hash.
- `composerEveryPathHashed` (the guard on whether §8h fires at all) requires
  **every** path to carry a hash. `composerHashByPhraseSync` only clears the
  flag when **no** path carries any hash at all (its own documented,
  intentionally over-sticky design — filed as a separate follow-up, not this
  fold's scope). So the kept path had to end up hashed too, by hex, for §8h to
  fire and be observable at all; it could not carry any hash at the exact
  moment of removal (that is the precondition the fix's clearing logic reads).

Sequence actually driven, all through the real UI:

1. Two-path policy (`composerTwoPathList()`); path 0 hashed by phrase through
   the real phrase route → `st.hashByPhrase == true`, confirmed.
2. Path 0 removed via `composerPathEdit`'s real Remove arm (touch harness,
   `Down,Down,Down` → Remove path → Button3). Remaining path (path 1) carries
   no hash of any kind at this instant.
   ```
   hashByPhrase == false   -- confirmed (the fix's clearing precondition, met)
   ```
3. The kept path (now index 0) hashed via the REAL `Type 64 hex` route
   (typed through the router); a brand-new path appended and ALSO hex-hashed
   via the same real route (matching the brief's "add a path hashed by
   `Type 64 hex`" literally). `hashByPhrase` stayed `false` through both.
4. `composerShapeFlow` driven to Done (`Down×4` → Button3, on the real
   pick-screen row order: path 1, path 2, "Add a spend path", "Change the
   script", "Done").

```
Section 8h AT DONE DREW: "HASHONEVERYPATHEverywaytospendthiswalletneedsthepreimageofahash.Itisnotonthisdeviceandnotontheseplates.Backthepreimageupseparately.Spendpaths"
```

The PLAIN preimage form (`composerCopyHashEveryPath()` — "Back the preimage up
separately"), NOT the phrase form (`composerCopyHashEveryPathPhrase()` —
"phrase and its method"). Re-applied the mutation (deleted the resync call)
against this same scratch test and confirmed it fails at the exact point the
flag should have cleared:
```
hashByPhrase still true after removing the only phrase-hashed path, with no hash of any kind left in the composition -- M-1 NOT fixed
```
Reverted; `git diff --stat` empty afterward. Scratch test file deleted before
the whole-suite gate run below; final `git status --porcelain` on the worktree
is empty.

## 4. Copy-table and modal-fit gates

```
--- PASS: TestComposerCopyTableCoversEveryBody
```

The brief names `TestModalsThisBlockTouchesAreDrawnInFull` for the modal-fit
gate; that test exists and passes, but its 8 subtests do not include the
confirm-modal string that changed. The gate that actually covers
`composerCopyHashlockOtherPath()` in a fits-check is a **different**,
correctly-scoped test in the same file:

```
=== RUN   TestConfirmScreensThisBlockTouchesAreDrawnInFull/the_hashlock_confirm_modal,_longest_variant_(H2_§4.5)
    modal_fits_test.go:393: the hashlock confirm modal, longest variant (H2 §4.5): 336 chars drawn in full, headroom 107 chars (margin 80)
--- PASS
```

Ran both; both pass. **Minor, not blocking:** the brief cites the wrong test
name for this specific check (`TestModalsThisBlockTouchesAreDrawnInFull` vs.
`TestConfirmScreensThisBlockTouchesAreDrawnInFull`) — worth a one-line
correction if this brief is reused, same class as the R1 report's recorded
gofmt-scope-wording note.

## 5. Whole gates at `a1fd139`

Four packages named in the fold commit message:
```
ok  	seedhammer.com/hashlock	0.233s
ok  	seedhammer.com/codex32	0.002s
ok  	seedhammer.com/seal	11.851s
ok  	seedhammer.com/sysw	0.038s
```

`gui-shard-test.sh ./gui/ 24`:
```
=== enumerating tests in ./gui/ ===
    1225 top-level tests
    partition verified exhaustive: 1225 == 1225
=== running 24 shards in parallel (timeout 20m each) ===
  [all 24 shards: ok]
=== wall: 27s ===
RESULT: ok -- all 1225 tests ran across 24 shards
```
Matches the brief's expected 1225 exactly, and the fold commit's own claim
("gui 1225 / 24 shards ok").

`gofmt -l .` (whole tree):
```
gui/transaction.go
gui/transaction_golden_test.go
gui/transaction_txrecord_test.go
mt/mt.go
mt/mt_test.go
```
The three `gui/transaction*.go` files match the brief's "only the pre-existing
transaction*.go"; the two `mt/*.go` files are the SAME pre-existing,
already-recorded Minor from the R1 report ("gofmt scope wording... a whole-tree
`gofmt -l .` also names `mt/mt.go` and `mt/mt_test.go` — pre-existing... not in
a touched package"). Unchanged since R1; not new; not blocking.

`go vet` scoped to the touched packages (hashlock, codex32, seal, sysw, gui —
same scoping the R1 report used):
```
gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file is go1.25)
gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
gui/transaction_golden_test.go:104:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
```
Exactly the same three pre-existing warnings the R1 report recorded at
`26fd1dd`/`c4a64fc`. Nothing introduced by this fold.

## 6. Refute reconciliation

Eight refute reports on file (two skeptics × four findings). For each, the
CONFIRMED/PARTIAL verdict and what addresses it:

| lens finding | skeptic 0 verdict | skeptic 1 verdict | addressed by |
| --- | --- | --- | --- |
| geometry C-1 (phrase screen draws no readout) | CONFIRMED | CONFIRMED | `26fd1dd` (F-481) — verified GREEN in `hashlock-H2-post-impl-r1-fold-verification.md` |
| geometry I-1 (`MaxHeight` overflow, 8px) | CONFIRMED (Important) | CONFIRMED | `26fd1dd` (F-481, same mechanism) |
| interruption I-1 (phrase screen draws no readout, same defect via a different lens) | CONFIRMED (Important) | CONFIRMED | `26fd1dd` (F-481) |
| host-device-e2e I-1 ("two phrases" hard-coded count) | **PARTIAL** (defect confirmed; severity disputed to Minor — the per-path backup instruction is unconditional and unaffected) | CONFIRMED (Important, not downgraded) | `a1fd139` — count-free wording, verified in §2/§3 above, both via mutation and via a real three-hashlock harness drive |

All eight verdicts map to one of the two fold commits; none is unaddressed.
The one severity split (PARTIAL vs. CONFIRMED-Important on host-device-e2e
I-1) does not change the reconciliation outcome — both skeptics agree the
underlying defect is real and reproduces, and the fold fixed the wording
regardless of which severity reading is preferred.

`interruption M-1` (Remove path never re-synced `hashByPhrase`) is a **Minor**
finding from the interruption lens report itself (not one of the eight
CONFIRMED/PARTIAL refute-pass targets — refute passes in this cycle ran only
against Critical/Important claims, and M-1 is Minor). It is fixed at `a1fd139`
regardless, verified in §3 above via mutation and the real end-to-end drive.

The `walk-control` and `records-claims` lenses each closed 0 Critical / 0
Important on their own (confirmed by reading their reports' closing-counts
sections directly) — no refute pass was run against them for that reason,
consistent with house practice.

## 7. Records at `e879123`

```
$ git -C me-worktrees/hashlock-h2 diff 67f9fe9..e879123 --stat
 design/FOLLOWUPS.md                                | 40 ++++++++++++++++++++++
 .../hashlock-H2-implementation-report.md           | 17 ++++++++-
 2 files changed, 56 insertions(+), 1 deletion(-)
```

**F-484..F-489 headers.** All six present, each traced to a specific lens
finding: F-484 (geometry lens M-1, lead-in-Back-margin), F-485 (walk-control
lens M-1/M-2/N-1/N-2), F-486 (host-device-e2e lens M-1), F-487
(host-device-e2e lens M-2), F-488 (host-device-e2e lens M-3), F-489
(host-device-e2e lens N-1) — matched by content, not just by number.

**Citations checked against `17b3979` (the pre-fold baseline all five lenses
ran against), read directly with `git show`:**

- F-484: `gui/composer_hashlock.go:166-168` — the exact `dims.X-2*8` /
  `widget.Labelw` wrap call; `:183` — the exact `op.Layer(kbdOp, leadOp, cntOp,
  nav, titleOp, ...)` line. `gui/composer_paged.go:62-90` — the exact W-3
  narrower-band comment block (`bandMargin`, `bandLeft`, `bandRight`). All
  three verbatim matches.
- F-485 / walk-control N-1: `cmd/emu/walk_hashlock_phrase.js:232` — the exact
  `chooseRow(0, "32-byte value", "Type a hashlock phrase")` call.
  `gui/composer_hash.go:158-174` (`composerHashRows`) and `:212-214` (the
  `taking := ...` predicate) — both verbatim matches.
- F-483's added sentence ("the phrase route holds a secret with §10.2.4's
  idle wipe timer disarmed") traces to the interruption lens's own M-3
  ("the route holds a secret with §10.2.4's timer disarmed") — confirmed
  present in the lens report.

Sample, not exhaustive (F-486 through F-489's citations were not re-derived
line-by-line — the content match against their source lens findings was
checked instead), but every citation actually checked resolved exactly.

**The implementation report's `331 lines` claim** (`cmd/emu/walk_hashlock_phrase.js`,
corrected from a stale "297"):
```
$ git show 17b3979:cmd/emu/walk_hashlock_phrase.js | wc -l
331
$ git show e1bf137:cmd/emu/walk_hashlock_phrase.js | wc -l    # the commit the report cites
331
```
Confirmed exactly; the file was never touched between `e1bf137` and `17b3979`
(`git diff e1bf137..17b3979 -- cmd/emu/walk_hashlock_phrase.js` is empty), so
"297" was a stale miscounted number from the original report, correctly fixed
here.

**The two fold addenda's gui test counts**, both re-derived independently
rather than trusted:
```
$ git worktree add --detach <scratch> 26fd1dd && go test ./gui/ -list '.*' | grep -cE '^(Test|Example|Fuzz)'
1224
```
(worktree removed after) — matches the `26fd1dd` addendum's "gui 1224 tests /
24 shards ok" exactly. The `a1fd139` addendum's "gui 1225 / 24 shards ok" is
the same figure already confirmed live in §5 above.

**Records-lens self-check:** the addendum's "records lens: 0C/0I, one Nit
(stale self-review line numbers in the plan)" matches the records-claims lens
report's own closing counts (`Critical: 0`, `Important: 0`, `Nit: 1` —
"three stale file:line citations in the plan's Self-review section") read
directly.

## Closing counts

**0 Critical / 0 Important / 0 new Minor / 1 recorded Minor (test-name citation
in this brief, `TestModalsThisBlockTouchesAreDrawnInFull` vs. the actually-
scoped `TestConfirmScreensThisBlockTouchesAreDrawnInFull` — informational, not
blocking, same class as the R1 report's recorded gofmt-scope note).**

All eight refute-pass CONFIRMED/PARTIAL verdicts reconciled to a commit
(`26fd1dd` or `a1fd139`); the Minor interruption M-1 finding reconciled to
`a1fd139`; six new follow-ups (F-484..F-489) filed and their headers/citations
spot-checked true; F-483's addendum confirmed; the implementation report's
line-count correction (297→331) and both fold addenda's gui-test counts
(1224, 1225) independently re-derived and confirmed exact.

## GREEN

This closes the post-implementation loop for hashlock H2. The branch merges.
