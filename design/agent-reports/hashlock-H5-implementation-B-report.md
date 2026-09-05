# Hashlock H5 implementation report -- implementer B (Task 4)

**Branch:** `h5-b` in worktree `/scratch/code/shibboleth/.tmp/seedhammer-h5-b`, off fork
`main` `b9a9a30`. **Tip after this task:** `c1f0237b83c72daca7c51100e198ee59ae68fe63`.
Files touched: `gui/unlock_kdf.go`, `gui/unlock_preimage_test.go` (only these two;
`git diff --stat` at the tip confirms). Go used throughout:
`/scratch/code/shibboleth/.toolchain/go/bin/go` (`go1.26.7 linux/amd64`), first on
PATH.

Plan: `design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` (engrave master
`5b77367`, STATUS R0 GREEN), Task 4: "The unlock refusal says what to do next"
(spec §5, F-488). Read the gated reference tree
`/scratch/code/shibboleth/.tmp/h5-gate` only to resolve one ambiguity in the
plan text (below); the ambiguity did not stop me from writing my own edits, and
the gated tree was neither copied wholesale nor modified.

## Task 4 -- one commit

**Commit `c1f0237`** (`git commit -s`, both files staged explicitly):
```
unlock: the record refusal says what to do next -- remove that record (records count from 0) on the host and seal again; 0-based stated because the number is now an instruction to delete (hashlock H5, F-488)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
Signed-off-by: Brian Goss <goss.brian@gmail.com>
```
`git show --stat HEAD`: 2 files changed, 66 insertions(+), 4 deletions(-).

### Step 1 -- tests, RED

Added the plan's three `uiContains` assertions to
`TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable`, and updated the
table in `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind`.

**Deviation, recorded (plan-text ambiguity, resolved by reading the gated
tree):** the plan's Interfaces section says "the table's three existing rows
gain the sentence, and a fourth row is added," but the table at `b9a9a30` has
**four** existing rows (record 1, record 0, record 7, record 2), not three, and
the plan gives only one explicit modified-row fragment (record 1) plus the new
row. Which of the four existing rows keeps its original (unmodified) want-list
was not stated. I read `.tmp/h5-gate/gui/unlock_preimage_test.go` (read-only) to
resolve it: exactly one existing row -- "a preimage plate at record 0 -- records
count from 0" -- is left unmodified (its want-list already isolates the 0-based
question and doesn't need the new sentence); the other three existing rows
(record 1, record 7, record 2) each gain the sentence in their want-list, and
the new row ("the longest noun at a two-digit index", Index 13) is inserted
between the record-7 row and the record-2 row, not appended at the end. After
writing this into my own copy of the file, `diff` against the gated tree's
`gui/unlock_preimage_test.go` was empty (byte-identical), confirming the
resolution matches the plan author's intent. This makes "three existing rows"
literal (record 1, 7, 2) and "a fourth row" the new Index-13 row -- consistent
with the mutation table's "all four rows" wording (4 of the resulting 5 rows
check for the sentence).

Run (`go test -count=1 -v -run
'TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable|TestUnlockNotPermittedBodyNamesTheRecordAndTheKind'
./gui/`), captured to `/scratch/code/shibboleth/.tmp/h5b-t4-step1-RED.txt`:

```
--- FAIL: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable (0.11s)
    unlock_preimage_test.go:69: the screen must say what to do next; got "Record1isahashlockpreimage,notaseed.Thispayloadcannotbeunlockedhere.Nothingwasopened.SealedPayload"
    unlock_preimage_test.go:76: the screen must say the index is 0-based; got "Record1isahashlockpreimage,notaseed.Thispayloadcannotbeunlockedhere.Nothingwasopened.SealedPayload"
    unlock_preimage_test.go:84: the screen must say there may be more than one; got "Record1isahashlockpreimage,notaseed.Thispayloadcannotbeunlockedhere.Nothingwasopened.SealedPayload"
--- FAIL: TestUnlockNotPermittedBodyNamesTheRecordAndTheKind (0.17s)
    --- FAIL: .../a_preimage_plate_at_record_1  (does not carry the new sentence)
    --- PASS: .../a_preimage_plate_at_record_0_--_records_count_from_0  (unmodified row -- passes, as expected)
    --- FAIL: .../a_codex32_secret_in_the_public_section  (does not carry the new sentence)
    --- FAIL: .../the_longest_noun_at_a_two-digit_index  (does not carry the new sentence)
    --- FAIL: .../a_record_this_machine_does_not_read_at_all  (does not carry the new sentence)
FAIL
```
Exactly 4 of the 5 table rows fail, matching the plan's mutation-table
description ("all four rows"). RED confirmed.

### Step 2 -- the body

`unlockNotPermittedBody` (`gui/unlock_kdf.go:415-420` in this tree) changed
from the one-sentence body to:
```go
func unlockNotPermittedBody(e *seal.RecordNotPermittedError) string {
	return fmt.Sprintf("Record %d is %s. This payload cannot be unlocked here. "+
		"Nothing was opened. Remove that record -- and any others like it -- "+
		"(records count from 0) on the host and seal the payload again.",
		e.Index, unlockRecordNoun(e))
}
```
plus the plan's doc comment. `diff` against the gated tree's `unlock_kdf.go`
after this edit: empty (byte-identical).

### Step 3 -- GREEN and measured fits

Same run, captured to `/scratch/code/shibboleth/.tmp/h5b-t4-step3-GREEN.txt`:
```
--- PASS: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable (0.12s)
    a preimage plate at record 1: 174 chars drawn in full, headroom 378 chars (margin 80)
    a preimage plate at record 0 -- records count from 0: 174 chars drawn in full, headroom 378 chars (margin 80)
    a codex32 secret in the public section: 162 chars drawn in full, headroom 397 chars (margin 80)
    the longest noun at a two-digit index: 175 chars drawn in full, headroom 378 chars (margin 80)
    a record this machine does not read at all: 174 chars drawn in full, headroom 378 chars (margin 80)
--- PASS: TestUnlockNotPermittedBodyNamesTheRecordAndTheKind (0.18s)
PASS
ok  	seedhammer.com/gui	0.316s
```
Matches the plan's measured numbers exactly (378/397 headroom, 80 margin).

### Mutations -- each applied, re-run, its failure quoted, then reverted

All three re-runs captured to files under `/scratch/code/shibboleth/.tmp/`;
after each, the file was restored from a saved-good copy and `diff` against
`.tmp/h5-gate/gui/unlock_kdf.go` confirmed byte-identical before the next
mutation (and before the final commit).

1. **Drop the whole new sentence** (`h5b-t4-mut1-whole-sentence.txt`):
   ```
   unlock_preimage_test.go:69: the screen must say what to do next; got "...SealedPayload"
   unlock_preimage_test.go:76: the screen must say the index is 0-based; got "...SealedPayload"
   unlock_preimage_test.go:84: the screen must say there may be more than one; got "...SealedPayload"
   --- FAIL: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable
   --- FAIL: TestUnlockNotPermittedBodyNamesTheRecordAndTheKind (4 of 5 rows: does not carry the sentence)
   ```
   Matches plan exactly.

2. **Drop `"(records count from 0)"` only** (`h5b-t4-mut2-records-count.txt`):
   ```
   unlock_preimage_test.go:76: the screen must say the index is 0-based; got "...Removethatrecord--andanyotherslikeit--onthehostandsealthepayloadagain.SealedPayload"
   --- FAIL: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable
   --- FAIL: TestUnlockNotPermittedBodyNamesTheRecordAndTheKind (4 of 5 rows: does not carry the sentence)
   ```
   Matches plan exactly (headroom rose to 397/418 with the clause gone, as expected).

3. **Drop `" -- and any others like it -- "` only** (`h5b-t4-mut3-plural.txt`):
   ```
   unlock_preimage_test.go:84: the screen must say there may be more than one; got "...Removethatrecord(recordscountfrom0)onthehostandsealthepayloadagain.SealedPayload"
   --- FAIL: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable
   --- FAIL: TestUnlockNotPermittedBodyNamesTheRecordAndTheKind (4 of 5 rows: does not carry the sentence)
   ```
   Matches plan exactly.

Final re-check after the last revert (`h5b-t4-final-green-recheck.txt`):
`ok seedhammer.com/gui 0.904s`.

## Runs at the tip (commit `c1f0237`)

**Brief's required run** -- `go test -count=1 -v -run
'Unlock|Preimage|ModalsThisBlock' ./gui/`, captured to
`/scratch/code/shibboleth/.tmp/h5b-t4-brief-run.txt`:
```
49 x --- PASS
0  x --- FAIL
ok  	seedhammer.com/gui	4.156s
```

**Whole gui shard set** -- `scripts/gui-shard-test.sh ./gui/ 24`, captured to
`/scratch/code/shibboleth/.tmp/h5b-t4-whole-gui-shard.txt`:
```
1225 top-level tests; partition verified exhaustive: 1225 == 1225
24 shards, all ok; wall: 27s
RESULT: ok -- all 1225 tests ran across 24 shards
```

`go build ./...`: exit 0, no output. `go vet ./gui/...`: 3 pre-existing
warnings in files this task did not touch (`gui/op/draw_test.go`,
`gui/freetext_sizeproof_golden_test.go`,
`gui/transaction_golden_test.go` -- `testing.ArtifactDir requires go1.26 or
later (file is go1.25)`), unrelated to Task 4.

## Deviations

One, recorded above: the plan's Interfaces-section row count ("three existing
rows") undercounts the table by one against the actual `b9a9a30` tree (four
existing rows), and the plan gives only one of the three modified-row
fragments explicitly. Resolved by reading (not copying) the gated tree, and
verified byte-identical after writing my own edit. No other deviation from the
plan's Task 4 text.

## Not done (out of scope for this task)

Nothing pushed. No commits to `main`/`master`. No sub-agents invoked. No
`.jsonl` read. No phrase or preimage bytes appear in any log file captured
above (the fixture plate string and fixture passphrase are never printed by
these tests; captured logs show only KDF timing and the rendered refusal body,
which by design carries no record bytes).
