# H5 plan — R0 round 0 FOLD report (opus)

**Artifacts folded:** `design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md`
(engrave master `0c2b13e`, working tree) and `design/SPEC_hashlock_H5_device_polish.md`
(`e03d8e7`). **Tree folded identically:** `/scratch/code/shibboleth/.tmp/h5-gate`
(fork `b9a9a30` + every task). **Reports folded from:**
`hashlock-H5-plan-R0-r0-fidelity.md` (0C/3I/4M/4N),
`-tests.md` (0C/0I/1M), `-journey.md` (0C/2I/5M/2N). Nothing committed; no report
edited; the fork checkout `/scratch/code/shibboleth/seedhammer` verified clean
(`git status --porcelain` empty) and never written to; no sub-agents; no `.jsonl`
read.

**Outcome: every Critical (there were none), every Important (5) and every listed
Minor/Nit folded, except two items declined with reasons in "Declined" below.
All gates re-run GREEN. `scripts/h5-plan-blocks-vs-tree.sh`: 55 blocks checked,
0 FAIL.**

Two findings that the fold's own measurements CHANGED, and that a reviewer should
read first because they were not in any report:

1. **The hook's firmware share is no longer 0 B.** Re-measuring after the fold's
   four copy-string edits gives shipped 1,599,208 B and hook-removed
   1,599,224 B — a share of **−16 B**. Spec §4.1/§6's "asserted 0 bytes" was
   asserting layout luck; both are folded to "no measurable cost", with the pair
   and the reasoning recorded.
2. **Journey M-3's remedy costs more headroom than the report measured**, because
   the report measured it against the pre-fold sentence: 175 drawn / **headroom
   378**, not 397. It still clears the 80 margin by a wide margin, so the fold
   proceeds — but 397 is now wrong wherever it appears as this body's headroom.

---

## Item 1 — fidelity I-1 = journey I-7 unmet: the stored-versus-displayed assertion was a tautology

**Change (tree + plan Task 5 Step 7, spec §4.2).** `cmd/emu/walk_hashlock_phrase.js`
gains `drawnToken(frame, where)`, which matches `/hash([0-9a-f]{8}\.\.[0-9a-f]{8})/`
against the squashed frame and throws if absent. The hardened trial stores the
parsed token in `out.displayed`; after the hold the walk asserts
`short8(after[0]) === displayed` **before** the corpus check, with a
`typeof after[0] !== "string"` guard ahead of both. The reconcile screen's token
assertion also compares against `displayed` rather than the constant. Step 12's
run-(c) row now names the assertion that fires, quoting its real message.

**Evidence — the walk's OWN assertion text, sliced verbatim out of the file by
`/tmp/h5_order_probe.mjs` and replayed for four scenarios:**

```
(a) unmutated          : PASSED all post-hold assertions; out={"storedBeforeHold":null,"stored":"3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12"}
(b) assigned pre-hold  : FAILED -> the path ALREADY holds a hash while the confirm modal is up: the digest is assigned before the hold, so Back after reading it would leave it set (F-485).
(c) stored perturbed   : FAILED -> the stored digest does not abbreviate to the token the confirm modal drew: the screen showed one digest and the policy holds another.
(d) agree, wrong corpus: FAILED -> the STORED digest is not the corpus's hardened digest for this phrase.
```

(d) is the fold's own addition and is what proves the two assertions INDEPENDENT
rather than one restating the other: screen and policy agreeing on a digest the
corpus does not hold fires only the corpus check. Module still loads:
`node -e "import('./cmd/emu/walk_hashlock_phrase.js')…"` → `MODULE PARSES + LOADS: function`.

## Item 2 — fidelity I-2 / journey N-1: the `ok`-guard read only the first assignment

**Change (tree + plan Task 5 Step 6, spec §4.4).** `cmd/emu/needle_test.go` gains
`walkOkAssignments(src)` (`FindAllStringSubmatch`) and `walkOkDriverSupplied(rhs)`;
the branch runs the `plates` check on EVERY right-hand side and takes the
"restates nothing" exit only when every one is a bare boolean. The log line now
says *"assigns `ok` nothing but the constant(s) …, so it restates no assertion"* —
it no longer claims a position it does not measure (journey N-1). New table test
`TestWalkOkGuardReadsEveryAssignment` with five rows.

**RED (the counterexample fidelity I-2 constructed, as `cmd/emu/walk_zzz_probe.js`):**

```
--- FAIL: TestWalkOkContainsNoDriverSuppliedPlateCount (0.00s)
    needle_test.go:638: walk_zzz_probe.js's `ok` contains `plates`, which the CALLER supplies (I-1/F-170):
        out.plates === 3
```

The probe was removed immediately; `ls cmd/emu/walk_*.js | wc -l` = **8**, measured.

**MUTATION of the new guard (first match only):**

```
--- FAIL: TestWalkOkGuardReadsEveryAssignment/the_verdict_is_the_last_assignment (0.00s)
    needle_test.go:611: walkOkDriverSupplied found 0 caller-supplied term(s) [] in ["false"], want 1
    needle_test.go:615: allConst = true over ["false"], want false
```

Measured and RECORDED in the test's own comment: the row *after* it ("an early
offender with a bare verdict after it") **survives** that mutation by construction,
because its offender IS the first match. The plan says so rather than claiming
both rows fail.

**GREEN:**

```
needle_test.go:647: walk_hashlock_phrase.js assigns `ok` nothing but the constant(s) true, so it restates no assertion (H5 §4.4)
needle_test.go:693: 8 walk script(s) checked; no driver-supplied plate count in any `ok`
```

`walk_h0_preimage.js` draws no log line — its `ok` is derived, not constant — and
is still counted as checked. Recorded in the plan.

## Item 3 — fidelity I-3: two production doc comments were stolen

**Change (tree + plan Task 3 Step 1 placement note, Task 5 Step 2 placement note,
new Task 5 Step 3a).** `composerFlowExit` moved above `composerFlow`'s doc comment
in `gui/composer_flow.go`; `composerTextBand` moved above `composerPageLines`' doc
comment in `gui/composer_paged.go`. New gate `gui/composer_doc_comment_test.go`.

**Proof, measured:**

```
$ go doc -u ./gui composerFlow      -> composerFlow is "Build a new policy" (SPEC_wallet_policy_composer.md §7), …
$ go doc -u ./gui composerPageLines -> composerPageLines lays out lines[start:] into the content box …
$ go doc -u ./gui composerFlowExit  -> composerFlowExit is everything one composition must undo, …
$ go doc -u ./gui composerTextBand  -> composerTextBand is the ONE horizontal band composer text wraps inside: …
```

**MUTATION (restore the pre-fold arrangement):**

```
--- FAIL: TestComposerHelpersDidNotStealADocComment (0.00s)
    composer_doc_comment_test.go:78: composerFlow has NO doc comment in composer_flow.go -- a block inserted beneath it with no blank line between takes it, and the record it carried goes with it (r0 fidelity I-3)
    composer_doc_comment_test.go:85: composerFlowExit's doc comment in composer_flow.go opens "composerFlow is \"Build a new policy\" (SPEC_wallet_policy_composer.md §7)," -- it is documenting composerFlowExit with another symbol's text
```

**A NEGATIVE RESULT the fold measured and recorded in three places** (the test's
own comment, the plan's Step 3a, spec §6): deleting the blank line between
`composerFlowExit`'s closing brace and `composerFlow`'s doc comment leaves the test
GREEN — `go/ast` binds a comment group to the declaration on the line AFTER it, so
the defect is the ORDER, not the whitespace. The reviewer's suggested repair
("one blank line before each inserted comment block") would NOT have fixed it, and
saying so is why the fix is a move rather than an insertion.

**Scope check.** An AST scan of every non-test file this stage touches
(`docscan`, `/scratch/code/shibboleth/.tmp/h5-fold-tools`) found exactly these two
`NODOC`+`MISDOC` pairs; the other `MISDOC` hits (`composerCopyHashlockReconcile`,
`composerCopyHashEveryPathPhrase`, `composerCopyHashlockPhraseLead`) are the
package's own house style — doc comments that open with a section reference — and
are pre-existing. That is why the gate is a NAMED list and not a package rule.

## Item 4 — journey I-1: the confirm modal's second sentence

**Change (tree, plan Task 2 Step 4 + Task 6 Step 1 + Task 6 Step 4, spec §1.2).**
Second sentence becomes *"The phrase and method are not on this device."*; the
first stays *"Write down this phrase, the method and this digest now."*
byte-identical (verified: the plan's fragment and the tree's literal both carry it
unchanged, and the checker matches the block).

**Measured by me, on the text as written:**

| body | drawn | headroom | gate |
| --- | --- | --- | --- |
| **shipped by this fold** | **343** | **107** | PASSES |
| the planned three-item form | 347 | 107 | PASSES |
| *"…The phrase and method are not on this device and not on your plates…"* | 361 | 64 | **FAILS** |
| *"…Without the phrase and method, …"* (the spec's already-rejected repair) | 361 | 64 | **FAILS** |

The two rejected variants were built in the tree and run: both trip
`TestConfirmScreensThisBlockTouchesAreDrawnInFull` with
`fits today with only 64 characters to spare, under the 80-character margin`. The
journey report's 343/107 reproduces exactly.

**MUTATION (restore the shipped H2 sentence):**
`TestComposerCopyIsVerbatimFromTheSpec`:
`composerCopyHashlockConfirm (SPEC §H2-4.5) does not match the spec.`, quoting both
bodies in full.

Propagated to: the copy-table row, the fit row (unchanged — same row, new body),
Task 6's H2 §4.5 fold block, the toolkit manual quote in Task 6 Step 4, and
spec §1.2.

## Item 5 — journey I-2: §8h's PLAIN form

**Change (tree, plan Task 1 Step 8, spec new §2.6).**
`composerCopyHashEveryPath` ends *"Back up every preimage separately."*, with a new
doc comment giving the reason. New test
`TestTwoPlateWalletBannerCountsEveryPreimage` beside the phrase-form test, on the
mixed wallet with the phrase path replaced by a second plate.

**Measured:** `the §8h every-path-hashed warning: 133 chars drawn in full, headroom
397 chars (margin 80)` (was 131/397) — logged by
`TestComposerEveryPathHashedWarns`.

**MUTATION (restore the singular):**

```
--- FAIL: TestComposerCopyIsVerbatimFromTheSpec (0.00s)
    composer_copy_test.go:181: composerCopyHashEveryPath (SPEC §8h) does not match the spec.
--- FAIL: TestTwoPlateWalletBannerCountsEveryPreimage (0.00s)
    composer_provenance_test.go:273: §8h's plain form does not count the preimages: …
    composer_provenance_test.go:276: §8h's plain form still names ONE preimage on a two-plate wallet: …
```

Task 6 Step 2 now folds BOTH forms into H2 §4.7 (the blockquote and the paragraph
beneath it that quotes `composerCopyHashEveryPath` with its
`gui/composer_copy.go:169-173` citation).

## Item 6 — journey M-2: the reconcile screen's threshold

**Change (tree, plan Task 2 Step 4 + Task 6, spec §1.1).** *"Before you cut plates,
run ms hashlock with this phrase and method on the host and check the digest
matches. If they differ, do not fund this wallet: build it again."*

The substring **`run ms hashlock with this phrase`** is kept verbatim — it is the
needle of `TestHashlockReconcileScreenIsReachableOnAMixedPolicy` and of the walk —
and both are GREEN.

**Measured:** `the hashlock reconciliation screen (H2 §4.5, H5 §1): 181 chars drawn
in full, headroom 339 chars (margin 80)` (was 186/339).

**MUTATION (restore "Before you fund this wallet"):**
`TestComposerCopyIsVerbatimFromTheSpec`:
`composerCopyHashlockReconcile (SPEC §H2-4.5) does not match the spec.` — with the
whole body on both sides. Recorded in the plan:
`TestHashlockReconcileScreenCarriesTheDigestMethodAndChars` **stays GREEN** under
it, because its three needles are the token, the method/chars line and the mismatch
sentence, none of which move — so the copy table is the gate for this sentence.

## Item 7 — journey M-3: the unlock refusal, and its headroom

**Change (tree, plan Task 4, spec §5).** *"Remove that record -- and any others
like it -- (records count from 0) on the host and seal the payload again."* ASCII
`--`, per the plan's copy rule. A third frame assertion added to
`TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable` with its MUTATION.

**Measured, all five rows of the fit table:**

```
unlock_preimage_test.go:174: a preimage plate at record 1: 174 chars drawn in full, headroom 378 chars (margin 80)
unlock_preimage_test.go:174: a preimage plate at record 0 -- records count from 0: 174 chars drawn in full, headroom 378 chars (margin 80)
unlock_preimage_test.go:174: a codex32 secret in the public section: 162 chars drawn in full, headroom 397 chars (margin 80)
unlock_preimage_test.go:174: the longest noun at a two-digit index: 175 chars drawn in full, headroom 378 chars (margin 80)
unlock_preimage_test.go:174: a record this machine does not read at all: 174 chars drawn in full, headroom 378 chars (margin 80)
```

**The gate the brief set was "fits with >= 80 headroom". Measured 378 at the
longest noun and a two-digit index — it fits, so the change is folded rather than
recorded as documentation-only.** The report's "397 headroom before" was the
headroom of the sentence WITHOUT the clause; the fold's own measurement is 175/378
and that number is what the spec and plan now carry.

**MUTATION (drop the clause only):**

```
--- FAIL: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable (0.10s)
    unlock_preimage_test.go:84: the screen must say there may be more than one; got "…Removethatrecord(recordscountfrom0)onthehostandsealthepayloadagain.SealedPayload"
--- FAIL: TestUnlockNotPermittedBodyNamesTheRecordAndTheKind (0.15s)
    [all four rows] does not carry "Remove that record -- and any others like it -- (records count from 0) on the host and seal the payload again."
```

## Item 8 — journey M-4 / M-5 / N-2

- **M-4 (decision recorded, no copy change).** A paragraph beside the "every … and
  every" sentence in `composerCopyHashEveryPathPhrase` states that the form
  overcounts on an all-phrase wallet and on a re-typed hex digest, that overcounting
  is the safe direction, and that counting exactly would need three variants.
  Mirrored in spec §2.5.
- **M-5 (manual).** Task 6 Step 4 gains the Back-table row
  `| the reconcile screen | the spend-path list | dropped; the hash is already assigned |`
  and the qualified lead sentence ("before the hold"). **Citations re-grepped at
  toolkit `46b40bb`:** heading `:512`, lead `:514-515`, table `:517-524` — the
  journey report's `:514-525` was one row wide, and the plan now carries the
  measured ranges. `:482-483` and `:501-502` re-verified exact.
- **N-2 (manual).** One clause naming `phrase: 28 characters` on `ms hashlock`'s
  **stderr** engraving card as the host counterpart of `chars: 28`. Verified in
  `mnemonic-secret` at `504ff46`: `crates/ms-cli/src/cmd/hashlock.rs:329-331`
  (`phrase_chars` into the JSON) and `:351-352` (the `phrase: {n} characters` line
  on stderr).

## Item 9 — fidelity M-1..M-4, N-1..N-4, tests M-1

- **fidelity M-1** — the "same digest re-typed as 64 hex" row now uses a DISTINCT
  pointer holding the same 32 bytes (`retyped := phrase`), and a new
  `TestReassigningTheSameDigestStaysByPhrase` drives the case through production
  (a payload row carrying the same digest), asserting FIRST that the pointer really
  changed so the test cannot pass for the reason a pointer comparison would.
  **MUTATION (compare pointers, not values):**
  ```
  composer_provenance_test.go:119: composerAnyPathByPhrase = false, want true   [all four positive rows]
  composer_provenance_test.go:163: the digest was derived from a phrase in this composition and re-entered unchanged; the backup burden is the same and the predicate says otherwise
  ```
  The hex PAD itself is not driven — no harness helper exists for 64 hex taps, and
  the payload row is the same arm of `composerHashEdit`; said so in the test.
- **fidelity M-2** — folded the spec, with a measurement as the argument rather
  than a cost estimate. See "Firmware" below.
- **fidelity M-3** — `hashlockOtherPathLine`'s clause now reads *"…unaffected by
  that set's own history…"*; the plan's fragment extended from one line to three so
  the checker covers the corrected sentence.
- **fidelity M-4** — spec §5 gains an **ERRATUM** paragraph: no `composerCopyTable`
  row can exist for `unlockNotPermittedBody`, and the `assertModalBodyFits` row at
  the longest noun and a two-digit index is what stands in its place. Task 4's
  Interfaces paragraph points at it.
- **fidelity N-1** — the walk's prelude now says `hashlock-h5`.
- **fidelity N-2** — the keyboard-probe block now says it was probed during H2 on
  `e1bf137`'s emulator build and NOT re-probed for H5, with the reason it did not
  move. **Verified before rewriting:** the block is byte-identical to `b9a9a30`'s
  (`diff` of the slice against the read-only fork checkout → identical), and
  `e1bf137` is the commit that last touched that file.
- **fidelity N-3** — the positive control's recipe is now stated exactly
  (`println("hook")` inside the stub's `setComposerStateHook`) and re-measured.
- **fidelity N-4** — `composer_hashlock.go`'s HOLD comment drops
  `composer_state.go:239`, matching what Task 2 already did in `composer_copy.go`;
  a new plan fragment covers the corrected lines.
- **tests M-1** — both incomplete RED transcriptions re-captured from my own runs,
  with `-gcflags=-e` added to the commands so Go prints every error instead of
  stopping at ten:
  - **Task 1 Step 4: 20 lines, every one in a TEST file** (captured on
    `.tmp/h5-red`, the gated tree with Step 5's field and helpers and their three
    production uses removed — the exact state Step 4 describes; the plan says so, so
    the capture is reproducible).
  - **Task 2 Step 3: 3 call sites, not 2** —
    `composer_copy_test.go:141:77`, `composer_hashlock_test.go:992:39`,
    `modal_fits_test.go:344:34` (captured on `.tmp/h5-red2`).

## Firmware — five builds, re-measured (the fold changed string literals)

| build | flash | ram |
| --- | --- | --- |
| fork main `b9a9a30` (own `git ls-files` copy, own build) | **1,597,404** | 62,856 |
| H5, folded tree | **1,599,208** | 62,856 |
| hook deleted from the tinygo view | **1,599,224** | 62,856 |
| a SECOND `defer clearComposerStateHook()` | 1,599,304 | 62,856 |
| positive control, `println("hook")` in the stub | 1,599,368 | 62,856 |

- **Stage delta: +1,804 B flash (+0.113 %), +0 B RAM** (was +1,760 B).
- **The hook's share is −16 B, not 0 B.** The hook-LESS image is 16 bytes LARGER.
  Before the fold, on a tree differing only in four operator-facing string
  literals, the same pair measured 1,599,164 / 1,599,164 — an exact 0. Determinism
  checked: the folded tree was built twice (in place and in a fresh copy) and both
  gave 1,599,208.
- **Second-defer cost: +96 B** (was 112). **Positive control: +160 B** (was 224).
- Folded into `gui/composer_state_hook_tinygo.go`, `gui/composer_flow.go` (two
  comments), plan Task 5 Steps 2 and 9, spec §4.1 and §6.
- **This is also the answer to fidelity M-2.** Per-change attribution is below the
  instrument's resolution: an unchanged hook measured 0 B and −16 B on trees
  differing by four string literals. Spec §6 now asks for the STAGE delta against a
  named baseline plus the structural question, measured subtractively with a
  positive control — and says why.

## Declined, with reasons

1. **fidelity M-2's first option — four extra cumulative tinygo builds for a
   per-change delta.** Declined on the measurement above: the noise floor of a
   whole-image build (±16 B, demonstrated) exceeds what four copy-string edits
   could be shown to cost, so per-change rows would report layout noise as cost.
   The spec is folded to the claim that is measurable instead of the plan being
   padded with numbers that are not.
2. **Re-running the two intermediate SHARD runs** (plan Task 1 Step 10, Task 2
   Step 7). Reconstructing a Task-1-only or Tasks-1-2 tree is a revert of four
   later tasks across shared files. The boundary counts are ENUMERATED from
   measured per-file test-function counts instead — 1225 + 6 = **1231**, + 2 =
   **1233** — and the plan states plainly that these are enumerated and not shard
   runs, names the command each term came from, and points at the end-of-tree run
   as the one that gates. That run is measured: **1239, partition verified
   exhaustive**.
3. **fidelity's note that a scrub-only `composerFlowExit` wrapper measures a
   different counterfactual (1,599,196 on the old tree).** Not added as a table
   row: the row the plan measures is the strict one (stub file and both call sites
   deleted), which is the counterfactual spec §4.1's claim is about.

## Gates, all re-run after the fold on `/scratch/code/shibboleth/.tmp/h5-gate`

```
go test -count=1 ./hashlock/ ./codex32/ ./sysw/ ./seal/ ./cmd/emu/
  ok hashlock 0.230s | ok codex32 0.003s | ok sysw 0.041s | ok seal 11.856s | ok cmd/emu 1.052s

gofmt -l .
  gui/transaction.go gui/transaction_golden_test.go gui/transaction_txrecord_test.go mt/mt.go mt/mt_test.go
  (the same five as the pristine b9a9a30 checkout)

go vet ./gui/ ./cmd/emu/
  gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
  gui/transaction_golden_test.go:104:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
  (both pre-existing)

GOOS=js GOARCH=wasm go vet ./cmd/emu/        -> clean, exit 0
./cmd/emu/build.sh                           -> built emu.wasm (10873125 bytes)
node import('./cmd/emu/walk_hashlock_phrase.js') -> MODULE PARSES + LOADS: function

scripts/gui-shard-test.sh ./gui/ 24
      1239 top-level tests
      partition verified exhaustive: 1239 == 1239
  RESULT: ok -- all 1239 tests ran across 24 shards

CGO_ENABLED=0 go test -count=1 -timeout 20m ./...   -> exit 0, 55 ok, 0 FAIL

tinygo x5 (see the Firmware table)

scripts/h5-plan-blocks-vs-tree.sh            -> 55 blocks checked, 0 FAIL
```

`b9a9a30` has **1225** top-level `gui` tests by the shard script's own count; the
plan now adds **14** (was 11), and `cmd/emu` gains **1**
(`grep -c '^func Test' cmd/emu/needle_test.go` = 9 vs 8). Enumeration checks out:
1225 + 6 (`composer_provenance_test.go`) + 2 (`composer_hashlock_test.go`, 18 vs
16) + 3 (`composer_hashlock_geometry_test.go`) + 2 (`composer_state_hook_test.go`)
+ 1 (`composer_doc_comment_test.go`) = **1239**.

## Records written

- Plan: STATUS → `DRAFT — R0 round 0 folded; r1 fold verification pending`, and a
  new `## R0 round 0 folded here` section (finding → change, with the declines).
- Spec: STATUS amended to point at a new `## Plan-round fold` section — the R0
  verdict is unchanged, but five normative items moved and six numbers were
  re-measured. A guard note added at the head of the spec's own
  `## R0 round 0 folded here` so its historical numbers are not read as current.
- New lesson recorded in the spec, beside the r1 one: **a "0" a whole-image build
  produces is not a structural zero until something has moved around it.** §4.1's
  0 B survived a spec round, two fold verifications and a plan build gate, and fell
  to four string literals.

## What a re-review should attack

- Whether *"Before you cut plates"* trades a funds-safety deadline for a cheaper
  one in a way the mismatch sentence does not fully recover.
- Whether *"-- and any others like it --"* reads as an instruction to hunt rather
  than a warning that the index moves, and whether the ASCII `--` inside a sentence
  that also carries a parenthesis is legible on the panel (the fit gate says it
  DRAWS; it does not say it reads well).
- Whether the enumerated boundary counts (1231, 1233) should have been shard runs.
- Whether spec §4.1's weakened claim is now too weak to gate anything, given that
  the measurement it rests on has a ±16 B floor.
- Whether `drawnToken`'s regex could match the reconcile screen's token instead of
  the confirm modal's on a frame that somehow carried both (it takes the first
  match, and only one screen is up at a time — but that is an argument, not a gate).
