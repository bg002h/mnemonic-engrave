# Hashlock H5 — implementer A report (Tasks 1, 2, 3)

**Branch `h5-a`, worktree `/scratch/code/shibboleth/.tmp/seedhammer-h5-a`, off fork main `b9a9a30`.
Branch tip `6cd4f1331bc335031fe08950c94a1c7b5b78a0e2`. Nothing pushed. Working tree clean.**

Plan: `design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` at engrave master `5b77367`
(STATUS R0 GREEN), spec `design/SPEC_hashlock_H5_device_polish.md` (`e03d8e7`).
Go `1.26.7` at `/scratch/code/shibboleth/.toolchain/go/bin/go`.
Every number below is quoted from a capture under `/scratch/code/shibboleth/.tmp/h5a/`.

| Task | Commit | Files |
| --- | --- | --- |
| 1 — per-digest provenance (§2, F-480) | `7b0868e77ec72407d253187c0440f5c05f634199` | `composer_state.go`, `composer_hash.go`, `composer_shape.go`, `composer_hashlock.go`, `composer_copy.go`, `composer_provenance_test.go` (new), `composer_copy_test.go`, `composer_hashlock_test.go` |
| 2 — the reconcile screen carries its operand (§1, F-487) | `0e2d9ad6868491191b0e950cd79c160d8c220e9c` | `composer_copy.go`, `composer_hashlock.go`, `composer_copy_test.go`, `modal_fits_test.go`, `composer_hashlock_test.go` |
| 3 — the lead inside the page band (§3, F-484) | `6cd4f1331bc335031fe08950c94a1c7b5b78a0e2` | `composer_paged.go`, `passphrase_keyboard.go`, `composer_hashlock.go`, `composer_hashlock_geometry_test.go` (new) |

`git diff --stat b9a9a30 HEAD` touches **exactly the 12 files the brief lists and no others**
(787 insertions, 94 deletions). All three commits carry `Signed-off-by: Brian Goss`,
`Co-Authored-By: Claude Fable 5.1` and the `Claude-Session` trailer.

## Independent machine check of block fidelity

`scripts/h5-plan-blocks-vs-tree.sh <plan> /scratch/code/shibboleth/.tmp/seedhammer-h5-a`
(captured to `.tmp/h5a/blockcheck.txt`):

```
55 blocks checked, 18 FAIL
```

Every one of the 18 failures is at plan line **1477 or later** — Tasks 4, 5 and 6, which belong to
the other implementers and are absent from this branch by design. **All 37 blocks in Tasks 1–3
(plan lines 165–1258) match this tree byte for byte**, whole blocks by `diff` and fragments by
exact substring. The script's own blind-spot tail applies unchanged: it checks TEXT, never a prose
claim, a mutation outcome or whether the tree is green.

---

## Task 1 — per-digest hash provenance (spec §2, F-480)

### RED (Step 4), complete, `-gcflags=-e`

`go test -gcflags=-e -count=1 -run '…' ./gui/` — **20 errors, every one in a TEST file**, the
identical set the plan quotes:

```
# seedhammer.com/gui [seedhammer.com/gui.test]
gui/composer_copy_test.go:165:2: undefined: composerNotePhraseDigest
gui/composer_hashlock_test.go:704:3: undefined: composerNotePhraseDigest
gui/composer_hashlock_test.go:722:6: undefined: composerAnyPathByPhrase
gui/composer_hashlock_test.go:920:6: undefined: composerAnyPathByPhrase
gui/composer_hashlock_test.go:1038:3: undefined: composerNotePhraseDigest
gui/composer_hashlock_test.go:1050:6: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:35:8: st.phraseDigests undefined (type *composerState has no field or method phraseDigests)
gui/composer_provenance_test.go:70:17: st.phraseDigests undefined (type *composerState has no field or method phraseDigests)
gui/composer_provenance_test.go:71:80: st.phraseDigests undefined (type *composerState has no field or method phraseDigests)
gui/composer_provenance_test.go:73:6: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:113:5: undefined: composerNotePhraseDigest
gui/composer_provenance_test.go:118:14: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:143:2: undefined: composerNotePhraseDigest
gui/composer_provenance_test.go:162:6: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:184:2: undefined: composerNotePhraseDigest
gui/composer_provenance_test.go:186:6: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:206:5: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:228:2: undefined: composerNotePhraseDigest
gui/composer_provenance_test.go:231:43: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:268:5: undefined: composerAnyPathByPhrase
FAIL	seedhammer.com/gui [build failed]
```

**Line-number note (not a defect).** The ten `composer_provenance_test.go` lines are identical to
the plan's. The other six sit lower than the plan's quote — `composer_copy_test.go` by 2,
`composer_hashlock_test.go` by 8 (three sites) and 78 (two sites) — because the plan captured its
RED from a copy of the FULLY GATED tree, in which Task 2's own edits are already present
(Task 2 Step 1 grows the reconcile row by 2 lines; Task 2 Step 2 inserts 8 lines into
`TestHashlockPhraseRouteSetsTheCorpusDigest` and ~70 more for the two new tests). At a
Task-1-only boundary those lines do not exist yet. Error **identities and count are identical**.

### GREEN (Step 9)

`go test -count=1 -run '…|TestComposerCopy|TestModals|TestHashlock' ./gui/`
→ `ok  	seedhammer.com/gui	37.051s` (plan expected 38.766s).

### Mutations — all six re-run, each reverted immediately

| # | Mutation | Measured failure at my tip |
| --- | --- | --- |
| 1 | `composerNotePhraseDigest` assigns without the nil check | `--- FAIL: TestComposerPhraseRouteHoldsOnTheZeroValueState` → `panic: assignment to entry in nil map [recovered, repanicked]`, stack through `gui/composer_provenance_test.go:66`. A PANIC, not a failure, exactly as the plan states. |
| 2 | `composerAnyPathByPhrase` returns `len(st.phraseDigests) > 0` | four tests, six lines: `composer_hashlock_test.go:723: the phrase form survived the last hash being cleared` (`TestComposerHashEditDispatchesByRowLabel/none_row_clears_without_the_rule_modal`); `composer_hashlock_test.go:1051: the only phrase-set path was removed and the phrase form is still chosen`; `composer_provenance_test.go:119: composerAnyPathByPhrase = true, want false` on `…/the_phrase_path_was_edited_to_a_payload_row` **and** `…/the_phrase_path_was_removed`; `composer_provenance_test.go:207: no path carries a phrase-derived digest and the predicate still says one does`; `composer_provenance_test.go:210: §8h names a phrase this composition no longer has:` |
| 3 | delete `composerNotePhraseDigest(st, d)` from `hashlockPhraseRoute` | `composer_hashlock_test.go:921: the phrase route did not record that this hash was set by phrase`; `composer_provenance_test.go:71: the anchor's digest is not in the phrase set (0 entries)` |
| 4 | restore `"Back up the phrase and its method, or the preimage plate, separately."` | `composer_copy_test.go:181: composerCopyHashEveryPathPhrase (SPEC §H2-4.7) does not match the spec.` **and** `… composerCopyHashEveryPathFor (SPEC §H2-4.7) …`; `composer_provenance_test.go:237` twice (`does not carry "every phrase and its method"`, `does not carry "every preimage plate"`) and `composer_provenance_test.go:241: §8h's phrase form still offers a CHOICE of backups:` |
| 5 | restore the PLAIN form's `"Back the preimage up separately."` | `composer_copy_test.go:181: composerCopyHashEveryPath (SPEC §8h) does not match the spec.`; `composer_provenance_test.go:273: §8h's plain form does not count the preimages:` and `composer_provenance_test.go:276: §8h's plain form still names ONE preimage on a two-plate wallet:` |
| 6 | `composerAnyPathByPhrase` compares `p.Hash` POINTERS (`for d := range st.phraseDigests { if p.Hash == &d }`) | `composer_provenance_test.go:119: composerAnyPathByPhrase = false, want true` on **all four** positive rows — `one_phrase_path`, `a_mixed_wallet:_one_phrase_path,_one_other`, `two_paths_share_one_phrase_digest`, `the_same_digest_re-typed_as_64_hex_is_still_by_phrase`; and `composer_provenance_test.go:163: the digest was derived from a phrase in this composition and re-entered unchanged; the backup burden is the same and the predicate says otherwise` |

### Whole `gui` package (Step 10)

`scripts/gui-shard-test.sh ./gui/ 24`:

```
    1231 top-level tests
    partition verified exhaustive: 1231 == 1231
```

The plan's predicted boundary figure (1225 + 6) **measured exactly**.
`RESULT: FAIL` on that run, from **one** test in shard 20:

```
--- FAIL: TestEngraveScreenReleasesResumeStateOnReturn (0.03s)
    residency_wiring_test.go:112: INCONCLUSIVE: the job never completed (state 1), so Engrave cannot return in a terminal state
```

This is **F-490**, already filed as a known load-dependent flake with this exact signature and this
exact reproduction ("fails INCONCLUSIVE … when a heavy neighbour shares the shard, and passes 3/3
alone"). Isolated re-run at my tip: `go test -count=3 -run 'TestEngraveScreenReleasesResumeStateOnReturn' ./gui/`
→ `ok  	seedhammer.com/gui	0.003s`. The test is in no file this task touches, and none of Tasks 1–3
goes near the engrave job. **The same shard command came back `RESULT: ok` at both later boundaries**
(Task 2 and Task 3, below), which is the same non-determinism from the other side. Not a blocking
finding; not a new defect.

### Fit number logged by Task 1's copy change

`TestComposerEveryPathHashedWarns` at my tip:
`composer_shape_test.go:238: the §8h every-path-hashed warning: 133 chars drawn in full, headroom 397 chars (margin 80)`
— the plan's Step 8 claim (133/397, was 131/397), re-measured here rather than copied.

---

## Task 2 — the reconcile screen carries its operand (spec §1, F-487)

### RED (Step 3) — byte-identical to the plan, line numbers included

```
# seedhammer.com/gui [seedhammer.com/gui.test]
gui/composer_copy_test.go:141:77: too many arguments in call to composerCopyHashlockReconcile
	have (string, string, number)
	want ()
gui/composer_hashlock_test.go:992:39: too many arguments in call to composerCopyHashlockReconcile
	have (string, string, number)
	want ()
gui/modal_fits_test.go:344:34: too many arguments in call to composerCopyHashlockReconcile
	have (string, string, number)
	want ()
FAIL	seedhammer.com/gui [build failed]
```

Three call sites, as the r0 fold corrected it to.

### GREEN (Step 6) and the measured fits

`go test -count=1 -run 'TestHashlockReconcile|TestHashlockPhraseRouteSetsTheCorpusDigest|TestComposerCopy|TestModals|TestConfirmScreens' ./gui/`
→ `ok  	seedhammer.com/gui	8.643s` (the plan's expected figure, to the millisecond).

```
modal_fits_test.go:352: the hashlock reconciliation screen (H2 §4.5, H5 §1): 181 chars drawn in full, headroom 339 chars (margin 80)
modal_fits_test.go:352: HASH ON EVERY PATH, phrase-route form (H2 §4.7): 165 chars drawn in full, headroom 378 chars (margin 80)
modal_fits_test.go:395: the hashlock confirm modal, longest variant (H2 §4.5): 343 chars drawn in full, headroom 107 chars (margin 80)
```

Spec §1.1's 181/339, §2.5's 165/378 and §1.2's 343/107, **re-measured on this tree**, all three
matching the plan.

### Mutations — all six re-run, each reverted immediately

| # | Mutation | Measured failure at my tip |
| --- | --- | --- |
| 1 | return the old one-sentence reconcile body | `composer_hashlock_test.go:973` three times: `does not carry "hash  3cf5d421..b70a4c12"`, `does not carry "method: hardened   chars: 28"`, `does not carry "If they differ, do not fund this wallet: build it again."`; and `composer_hashlock_test.go:996: the reconcile body does not open with the shared header:` |
| 2 | restore `"Before you fund this wallet, …"` as the FIRST sentence | `composer_copy_test.go:181: composerCopyHashlockReconcile (SPEC §H2-4.5) does not match the spec.` — **and `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars` stayed GREEN**, confirmed in the same run, exactly as the plan predicts: the copy table is this sentence's gate, the flow test is the operand's. |
| 3 | restore the confirm modal's `"They are not on this device and not on your plates."` | `composer_copy_test.go:181: composerCopyHashlockConfirm (SPEC §H2-4.5) does not match the spec.` |
| 4 | drop the mismatch sentence only | `composer_hashlock_test.go:973: the reconcile screen does not carry "If they differ, do not fund this wallet: build it again."`; `composer_copy_test.go:181: composerCopyHashlockReconcile (SPEC §H2-4.5) does not match the spec.` |
| 5 | restore `"Write down this phrase and the method now."` | `TestHashlockPhraseRouteSetsTheCorpusDigest`, BOTH subtests — `composer_hashlock_test.go:383: the confirm modal's write-down line does not name the digest: "hash3cf5d421..b70a4c12method:hardenedchars:28writedownthisphraseandthemethodnow.thephras…"` (hardened anchor) and `"hashb867db87..edbc96cbmethod:sha256chars:28writedownthisphraseandthemethodnow.thephrasea…"` (sha256 anchor); plus `composer_copy_test.go:181: composerCopyHashlockConfirm (SPEC §H2-4.5) does not match the spec.` |
| 6 | `"method: %s chars: %d"` — one space instead of three | `composer_hashlock_test.go:996: the reconcile body does not open with the shared header:` — **and `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars` stayed GREEN**, verified in the same run: `normalizeDrawn` strips whitespace, so no frame assertion in this package can see a spacing change. The header-equality test is the only gate for it. |

### Whole `gui` package (Step 7)

```
    1233 top-level tests
    partition verified exhaustive: 1233 == 1233
=== wall: 23s ===
RESULT: ok -- all 1233 tests ran across 24 shards
```

Predicted 1233, measured 1233, **all 24 shards ok** (F-490 did not flake in this run).

---

## Task 3 — the phrase screen's lead inside the page band (spec §3, F-484)

### The doc-comment placement gate (Step 1)

`composerTextBand` was placed ABOVE `composerPageLines`' own doc comment with a blank line between,
as the plan requires. Verified with the command the plan names:

- `go doc -u ./gui composerPageLines` → `composerPageLines lays out lines[start:] into the content box and returns the ops, HOW MANY were drawn, and each drawn row's TOUCH BAND.`
- `go doc -u ./gui composerTextBand` → `composerTextBand is the ONE horizontal band composer text wraps inside: the panel, less the navigation column at the right edge and the same 8 px …`

Both print their own text; neither comment was merged into the other.

### RED (Step 5) — the pre-fix layout, byte-identical to the plan

```
    composer_hashlock_geometry_test.go:52: the phrase screen's lead is drawn UNDER a navigation button.
          button (427,44)-(480,97) received ink at (431,52)
        The operator cannot read what a button covers, and ExtractText collects the runes anyway -- which is why every text assertion on this screen passed while 152 px of the lead sat inside Back (F-484, W-3).
    composer_hashlock_geometry_test.go:68: band left=8 width=411; lead (440,44) = 2 line(s) of 23 px
--- FAIL: TestHashlockPhraseLeadIsDrawnInsideTheBand (0.00s)
```

The lead measures **440 px against the band's 411**, as the plan records.

### GREEN and the §3.3 branch decision

`go test -count=1 -run 'TestHashlockPhraseLead|TestHashlockPhraseScreen|TestComposerPaged|TestPassphraseKeyboard|TestTextKeyboard|TestComposerPickTouch|TestFreetext' ./gui/`
→ `ok  	seedhammer.com/gui	1.121s` (plan expected 1.117s). Logged:

```
composer_hashlock_geometry_test.go:68:  band left=8 width=411; lead (407,44) = 2 line(s) of 23 px
composer_hashlock_geometry_test.go:96:  scanner sees ink at (435,55)
composer_hashlock_geometry_test.go:145: MaxHeight=209 grid=(340,182) gap=8 -> readout budget 19 px; one line is 19 px
```

**Two lines at 411 px**, so §3.3's fallback copy is NOT used and H2 §4.2 is not folded — the plan's
branch decision, re-measured. §3.2(b)'s budget is **19 px against one line of 19 px: equal, with no
slack**, exactly as the plan warns.

### Mutations — all three re-run, each reverted immediately

| # | Mutation | Measured failure at my tip |
| --- | --- | --- |
| 1 | `hashlockPhraseLead` wraps at `dims.X-2*8` and centres on the panel | the RED above: ink at `(431,52)` inside button `(427,44)-(480,97)`, lead measuring `(440,44)` against the 411 px band |
| 2 | `composerTextBand`'s `right` drops the nav column (`dims.X - bandMargin`) | `composer_hashlock_geometry_test.go:52: the phrase screen's lead is drawn UNDER a navigation button. button (427,44)-(480,97) received ink at (431,52)` **and** W-3's own gate `TestComposerPagedLinesNeverDrawUnderTheNavButtons` at `composer_paged_geometry_test.go:214` on `keyed stub page 0` (ink at `(451,57)`), `keyed stub page 1` (ink at `(429,86)`) and `keyless stub page 0` (ink at `(427,57)`) — confirming the shared function is the one both screens use |
| 3 | restore `content, _ = content.CutBottom(8)` in `hashlockPhraseFlow` (F-481's original defect) | `composer_hashlock_geometry_test.go:145: MaxHeight=201 grid=(340,182) gap=8 -> readout budget 11 px; one line is 19 px` then `composer_hashlock_geometry_test.go:148: the readout budget is 11 px and one line needs 19: PassphraseKeyboard.Layout clamps every rune away, so nothing is masked, nothing is revealed, and the "show" key is a dead control (F-481)`; and `composer_hashlock_test.go:1077: the phrase screen drew 0 asterisks for 10 typed characters; the readout is not drawn (F-481).` |

### Whole `gui` package

```
    1236 top-level tests
    partition verified exhaustive: 1236 == 1236
=== wall: 24s ===
RESULT: ok -- all 1236 tests ran across 24 shards
```

1233 + 3 (`grep -c '^func Test' gui/composer_hashlock_geometry_test.go` = 3, measured).
`gofmt -l gui/` reports only the three files already unformatted at `b9a9a30`
(`transaction.go`, `transaction_golden_test.go`, `transaction_txrecord_test.go`); none of mine.
`gui/composer_hashlock_geometry_test.go` is byte-identical to the gated tree's copy (`diff` clean).

---

## Deviations

**D-1 — a plan gap, fixed minimally: the plain §8h copy-table row.**
Task 1 Step 8 rewrites `composerCopyHashEveryPath`'s body ("Back the preimage up separately." →
"Back up every preimage separately."), but **no block in the plan updates that body's row in
`composerCopyTable()`** — `gui/composer_copy_test.go:60-61`. Step 3 quotes only the *Phrase* and
*For* rows. Left alone, `TestComposerCopyIsVerbatimFromTheSpec` fails at Step 9's GREEN, and Task 1's
own mutation 5 (`restore … → composerCopyHashEveryPath (SPEC §8h) does not match the spec.`) has
nothing to bite. I updated the row's `verbatim` column to match the new body, and nothing else.
Cross-checked against the plan author's gated tree, whose line 61 is byte-identical to what I wrote.
Task 1's mutation 5 then produced the failure the plan predicts, which is the evidence the fix is
the intended one.

**D-2 — a records note, no code effect: Task 1's RED line numbers.** Six of the twenty RED lines sit
2, 8 or 78 lines lower than the plan's quote, because the plan captured its RED on a copy of the
fully gated tree (Tasks 2–6 applied) rather than at a Task-1-only boundary. Documented in full under
Task 1's RED above. Error identities, files and count are identical; nothing was reconstructed.

**No other deviation.** No file outside the brief's twelve was touched, no sub-agent was dispatched,
no `.jsonl` was read, nothing was pushed, no commit was made on `main` or `master`, and no phrase or
preimage byte appears in any capture kept under `.tmp/h5a/` (the only phrase in play is the public
test anchor `correct horse battery staple`, already in the fork's committed test source).

## Not mine, flagged for the controller

- **F-490 (`TestEngraveScreenReleasesResumeStateOnReturn`) flaked once**, in Task 1's shard run, and
  passed at the two later boundaries and 3/3 in isolation. Already filed with the same reproduction;
  owning phase "fork test hygiene, next fork code cycle". No action taken.
- **The fork baseline's `cmd/emu` red** (`TestWalkOkContainsNoDriverSuppliedPlateCount`) is
  untouched here — the plan assigns it to Task 5, not to me. `go build ./...` is clean at my tip; I
  did not run `go test ./...` package-wide, since the brief scopes my whole-package gate to
  `gui-shard-test.sh ./gui/ 24`.
