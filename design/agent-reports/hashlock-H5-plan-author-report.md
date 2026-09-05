# Hashlock H5 — plan author report (build gate RUN)

**Deliverables.** `design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` (2,698
lines, 51 headed code blocks) and `scripts/h5-plan-blocks-vs-tree.sh`. Gate tree
left in place at `/scratch/code/shibboleth/.tmp/h5-gate` (a `git ls-files` copy of
fork main `b9a9a30`). Nothing committed. No sub-agents. No `.jsonl` read. No
phrase or preimage bytes in any log kept.

Spec read at `d36ede5`, re-read at `44b1690`, `d206a2e` and finally **`e03d8e7`**
(R0 GREEN), which is the commit the plan header cites. Every number the plan
carries was measured on the gate tree by me; where it agrees with the spec that is
two independent runs producing the same value, not a copy.

---

## 1. What was built, task by task, with the evidence

Wired **task by task in the order written**, with a build and test run at every
boundary and the whole `gui` package sharded after Tasks 1, 2 and 5.

### Task 1 — §2 per-digest provenance

RED (measured):

```
gui/composer_provenance_test.go:35:8: st.phraseDigests undefined (type *composerState has no field or method phraseDigests)
gui/composer_provenance_test.go:73:6: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:108:5: undefined: composerNotePhraseDigest
FAIL	seedhammer.com/gui [build failed]
```

GREEN: `ok  	seedhammer.com/gui	38.162s`.

Mutations, all RED:

| mutation | measured |
| --- | --- |
| `composerNotePhraseDigest` without the nil check | `panic: assignment to entry in nil map [recovered, repanicked]` at `composer_provenance_test.go:66` through `holdConfirm` — a panic, not a failure |
| `composerAnyPathByPhrase` returns `len(st.phraseDigests) > 0` | 4 tests / 6 assertions: `the phrase form survived the last hash being cleared`; `the only phrase-set path was removed and the phrase form is still chosen`; `composerAnyPathByPhrase = true, want false` (×2 subtests); `no path carries a phrase-derived digest and the predicate still says one does`; `§8h names a phrase this composition no longer has` |
| delete `composerNotePhraseDigest(st, d)` from `hashlockPhraseRoute` | `the phrase route did not record that this hash was set by phrase`; `the anchor's digest is not in the phrase set (0 entries)` |
| restore `"…the phrase and its method, or the preimage plate…"` | `TestComposerCopyIsVerbatimFromTheSpec` on both §8h rows; `does not carry "every phrase and its method"`, `does not carry "every preimage plate"`, `still offers a CHOICE of backups` |

Boundary: **1229 top-level tests, `partition verified exhaustive: 1229 == 1229`,
24/24 shards ok, 22 s.** (`b9a9a30` = 1225 by the same count.)

### Task 2 — §1 the reconcile screen

RED: `too many arguments in call to composerCopyHashlockReconcile / have (string, string, number) / want ()` at `composer_hashlock_test.go:992` and `modal_fits_test.go:344`; `FAIL	seedhammer.com/gui [build failed]`.

GREEN: `ok  	seedhammer.com/gui	8.827s`, with the fit gates logging

```
the hashlock reconciliation screen (H2 §4.5, H5 §1): 186 chars drawn in full, headroom 339 chars (margin 80)
HASH ON EVERY PATH, phrase-route form (H2 §4.7): 165 chars drawn in full, headroom 378 chars (margin 80)
the hashlock confirm modal, longest variant (H2 §4.5): 347 chars drawn in full, headroom 107 chars (margin 80)
```

Mutations, all RED:

| mutation | measured |
| --- | --- |
| the old one-sentence reconcile body | `does not carry "hash  3cf5d421..b70a4c12"`, `…"method: hardened   chars: 28"`, `…"If they differ…"`; and the header test |
| drop the mismatch sentence | `does not carry "If they differ, do not fund this wallet: build it again."`; `composerCopyHashlockReconcile (SPEC §H2-4.5) does not match the spec.` |
| restore `"Write down this phrase and the method now."` | both `TestHashlockPhraseRouteSetsTheCorpusDigest` cases: `the confirm modal's write-down line does not name the digest`; plus the verbatim gate |
| `"method: %s chars: %d"` (one space) | header test RED; **the frame test stays GREEN** — `normalizeDrawn` strips whitespace, so NO frame assertion in this package can see a spacing change. That is why the header-equality test exists separately. |

Boundary: **1231 tests, exhaustive, 24/24, 32 s.**

### Task 3 — §3 the lead in the band

**Which §3.3 branch: NEITHER.** Measured: `band left=8 width=411; lead (407,44) = 2
line(s) of 23 px` — two lines in the narrower band, so the fallback copy is not
used and H2 §4.2 is not folded.

Mutations, all RED:

| mutation | measured |
| --- | --- |
| restore the panel-wide lead (the pre-fix state = RED) | `button (427,44)-(480,97) received ink at (431,52)`, lead measuring `(440,44)` against the band's 411 |
| `composerTextBand` drops the nav column | the lead test **and** `TestComposerPagedLinesNeverDrawUnderTheNavButtons` on `keyed stub` p0/p1 and `keyless stub` p0 |
| restore `content.CutBottom(8)` (F-481) | `MaxHeight=201 grid=(340,182) gap=8 -> readout budget 11 px; one line is 19 px`; and `the phrase screen drew 0 asterisks for 10 typed characters` |

Unmutated §3.2(b): `MaxHeight=209 grid=(340,182) gap=8 -> readout budget 19 px;
one line is 19 px`. **The budget equals one line exactly — zero slack.** Worth a
reviewer's attention: the narrower band did not change it (the lead is 2 lines at
both 464 and 411 px), but any future pixel added above the keyboard turns it red.

Boundary: `ok  	seedhammer.com/gui	1.110s` over the geometry, keyboard, paged and freetext tests.

### Task 4 — §5 the refusal's next step

RED: `the screen must say what to do next`, `the screen must say the index is
0-based`, and all four table rows `does not carry "Remove that record (records
count from 0) on the host and seal the payload again."`

GREEN with the fits:

```
a preimage plate at record 1: 152 chars drawn in full, headroom 397 chars (margin 80)
the longest noun at a two-digit index: 153 chars drawn in full, headroom 397 chars (margin 80)
```

397 at the longest noun and a two-digit index — spec §5's number, reproduced.
Both mutations RED (drop the sentence; drop `(records count from 0)` alone).

### Task 5 — §4 the seam, the glue, the walk

Seam mutations, all RED:

| mutation | measured |
| --- | --- |
| drop the clear from `composerFlowExit` | `the hook survived the composition it was installed for: []` |
| drop `setComposerStateHook(st)` | `the hook is not installed while composerFlow is running` |
| hand out `st`'s own pointers | `writing through the hook's pointer changed the POLICY: ff01…, want 0001…` |
| skip paths with no hash | `the hook reports 1 entries for a 2-path composition` |
| the tinygo stub exports something | `composer_state_hook_tinygo.go exports ComposerPathHashesOnDevice -- that file IS the firmware` |
| `ComposerPathHashes` named in another gui file | `composer_flow.go uses ComposerPathHashes in code but is not composer_state_hook.go` |

The walk parses and loads as an ES module (`node -e "import(...)"` →
`MODULE PARSES + LOADS`). `GOOS=js GOARCH=wasm go vet ./cmd/emu/` clean;
`./cmd/emu/build.sh` → `built emu.wasm (10873113 bytes)`.

Boundary: **1236 tests, `partition verified exhaustive: 1236 == 1236`, 24/24, 31 s**;
`CGO_ENABLED=0 go test -timeout 20m ./...` → **55 packages ok, exit 0**.

### Firmware size

| build | flash | ram |
| --- | --- | --- |
| fork main `b9a9a30` | 1,597,404 | 62,856 |
| H5 (this tree) | **1,599,164** | **62,856** |
| H5 with the hook deleted from the tinygo view | 1,599,164 | 62,856 |
| H5 with a SECOND `defer clearComposerStateHook()` — **not used** | 1,599,276 | 62,856 |
| positive control: the stub given one `println` | 1,599,388 | 62,856 |

**The hook's share is 0 B flash / 0 B RAM**, and the zero is believable because
the positive control moves the image by +224 B. Whole stage: **+1,760 B flash
(+0.11%), +0 B RAM.**

### Checker

`scripts/h5-plan-blocks-vs-tree.sh` → **51 blocks checked, 0 FAIL**; 23 unheaded
blocks named as its blind spot. Written as a thin wrapper over
`scripts/h2-plan-blocks-vs-tree.sh`, which already takes (plan, tree) arguments
and whose parser is generic — a 153-line copy would be a second thing to fix.

---

## 2. Things the spec asked for that I could NOT implement as written

Four, each with its measurement. None of them blocks the plan; all four are folded
into it explicitly rather than silently.

### 2.1 The hook's 0 bytes required a shape the spec does not describe (measured)

Spec §4.1: *"the tinygo twin is empty, so the delta attributable to this hook is
asserted 0 bytes."* Written the obvious way —

```go
setComposerStateHook(st)
defer clearComposerStateHook()
```

— beside the seed scrub's existing `defer st.reg.scrub()`, it measured **1,599,276
B, i.e. +112 B**, not 0. TinyGo elides the empty stub's CALL but not the defer
record around it. Folding both into the ONE deferred call the flow already had
(`composerFlowExit`, which runs the scrub and the clear) measured **1,599,164 B —
byte-identical to the no-hook build**. The spec's assertion holds, but only for a
shape the spec does not name; both numbers are recorded in the plan (Task 5 Step 9)
and in `composer_state_hook_tinygo.go`, where the next reader will look.

### 2.2 `go test ./...` is RED at the plan's own baseline (pre-existing)

`TestWalkOkContainsNoDriverSuppliedPlateCount` (`cmd/emu/needle_test.go`) FAILS on
the **pristine** fork checkout at `b9a9a30` under CI's exact command
(`CGO_ENABLED=0 go test ./...`, `.github/workflows/test.yml:75`):

```
--- FAIL: TestWalkOkContainsNoDriverSuppliedPlateCount (0.00s)
    needle_test.go:525: INCONCLUSIVE: walk_h0_preimage.js has no `ok:` property this test can read …
    needle_test.go:525: INCONCLUSIVE: walk_hashlock_phrase.js has no `ok:` property this test can read …
FAIL	seedhammer.com/cmd/emu	1.094s
```

Its `okExprRe` reads only the object-literal form; both walks ASSIGN, and have
since `45f3d4c` (H0) and `e1bf137` (H2). **Not introduced by H5.** It has to be
fixed here anyway, because spec §4.4 changes this walk's `ok` to exactly the shape
the guard cannot read — so Task 5 Step 6 teaches the guard the assignment form and
treats a bare boolean RHS as the *strongest* form of the property. After the fix
the guard reports `8 walk script(s) checked` where it reported 6, i.e. it had been
silently skipping the two walks it names in its own doc comment.

### 2.3 Spec §5's journey M-5 has no target that exists

*"the manual's unlock section says so."* There is no unlock section. Toolkit
`docs/manual/src/40-cli-reference/` holds only `41-mnemonic.md`, `42-md.md`,
`43-ms.md`, `44-mk-cli.md`, and a grep of the whole `docs/manual/src/` tree at
`46b40bb` for `Nothing was opened`, `cannot be unlocked here` and `not a seed`
returns nothing. The plan files it (Task 6 Step 3, owning phase *the `me`/sysw
manual chapter*) rather than inventing a chapter or hiding a sentence about
re-sealing inside the `ms hashlock` chapter.

### 2.4 Spec §5's "copy-table row updated" has no row to update

`unlockNotPermittedBody` is not composer copy, so it has no `composerCopyTable()`
row and the AST scan does not reach it; `gui/s6b_p7_modal_fit_sweep_test.go` covers
a *different* unlock body. Its fit gate is the `assertModalBodyFits` call inside its
own table (`gui/unlock_preimage_test.go:139`), and that is where the plan adds the
longest-noun / two-digit-index row. Stated plainly in Task 4's Interfaces rather
than left to look like an omission.

---

## 3. Other findings a reviewer should have

- **A false PASS in one of my own tests, caught by running the mutation.**
  `TestComposerStateHookReportsEachPathAndHandsOutCopies` first compared the policy
  against `d` — the very variable `st.list.Paths[1].Hash` points at — so the
  "hands out `st`'s own pointers" mutation was GREEN. Fixed with a `want := d`
  snapshot; the mutation is now RED and the reason is a comment in the test.
- **A whitespace change is invisible to every frame assertion in `gui`.**
  `normalizeDrawn` strips whitespace before comparing, so `"method: %s chars: %d"`
  (one space) passes the reconcile flow test. Only the header-equality test sees
  it. Worth knowing before someone deletes that test as redundant.
- **The readout budget has zero slack** (19 px available, 19 px needed). Recorded
  in Task 3, but a reviewer may want to decide whether that is a gate or a trap.
- **Five spec/fork citations did not resolve at `b9a9a30`** and are corrected in
  the plan (measured, not guessed): `hashlockFirst8Last8` is at
  `gui/composer_hashlock.go:131` (not `:133`); the reconcile `showError` at `:82`
  (not `:81`); `TestHashlockReconcileScreenIsReachableOnAMixedPolicy` at
  `gui/composer_hashlock_test.go:882` with its needle at `:909` (the spec's `:909`
  is the needle, not the func); the "line budget, not a character budget" text at
  `gui/modal_fits_test.go:30-32` (spec says `:33-35`); the sweep's
  `unlock_kdf.go:448` row label is stale — that body is at `:502`.
- **H2 spec §4.5's reuse block is drifted and NOT fixed here.** Its lines
  `:267-271` still carry the pre-drop-order wording while the shipped body carries
  the two-sentence form §4.5's own drop order prescribes. Outside H5's five
  follow-ups; the plan files it rather than folding it, so `git diff` on the
  records commit is H5's change and nothing else.
- **`gofmt -l`** reports `gui/transaction.go`, `gui/transaction_golden_test.go`,
  `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go` — identical set
  on the pristine baseline. **`go vet ./gui/`** reports the two pre-existing
  `testing.ArtifactDir requires go1.26` findings. Both verified against
  `b9a9a30`, not assumed.

---

## 4. Not covered by the gate

The three emulator walk runs (spec §4.5) — the walk was written, parsed and
loaded, and `cmd/emu` builds, but no browser drove it; the plan writes both
mutation recipes as exact one-line edits for the controller. The toolkit
`make lint` of Task 6. Every prose claim in Task 6, whose files are outside the
fork tree and therefore outside the checker.
