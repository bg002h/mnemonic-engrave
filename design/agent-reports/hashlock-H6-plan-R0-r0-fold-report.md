# H6 plan — R0 round 0, the FOLD

**Author:** opus, fold role, brief
`design/agent-briefs/hashlock-H6-plan-R0-r0-fold-brief.md`.
**Artifacts:** `design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md`
(engrave master `e6d84d9c`, working tree) and
`design/SPEC_hashlock_H6_preimage_plates.md` (`5bb46948`).
**Trees:** `/scratch/code/shibboleth/.tmp/h6-ms`, `-me`, `-gate` — edited in
place, identically to the plan's blocks, as the brief directs.
**Folded:** `hashlock-H6-spec-plan-round-verification.md` (GREEN, nothing to
fold), `-fidelity.md` (0C/8I/4M/2N), `-journey.md` (1C/5I/4M/2N), `-tests.md`
(0C/0I/2M).

**Result: 1 Critical, 13 Importants and 8 Minors/Nits folded; 3 Minors recorded
as decisions with reasons; 1 prescribed remedy declined on measurement; 3 defects
found by the fold itself and fixed.** Nothing committed. No sub-agents. No
`.jsonl` read. No phrase or preimage bytes in any log.

---

## 1. The Critical — journey C-1, the phrase-form locator

**The defect, reproduced.** Every call of `hashlockPlatesLocator` in the suite
built its session from `composerTestPreimageRecord`, which arrives
`derived: true` (`hashlockPlatesRecords` decodes X at list time), so the carrier
the rule is written for was never exercised. And `hashlockPlateLocator`
unconditionally appends the row, so "NON-EMPTY" is true by construction — a
locator built BEFORE the derive is equally non-empty and reads
`hash  00000000..00000000`.

**Folded as three changes**, all in the tree and in the plan (Task 9 Step 8 for
the seam, Task 10 Step 6 for the rows):

1. `gui/composer_preimage_plate.go` gains `composerHashlockPlateBuiltHook`, a
   nil-in-production seam of `freetextEngraveHook`'s shape, called from
   `composerHashlockPlateFor` — because a locator is ENGRAVED and never drawn,
   so no screen assertion can reach it.
2. a phrase row in `TestHashlockPlatesLocatorAlwaysCarriesTheHashRow`, using a
   HARDENED phrase and the derived state `hashlockPlatesDerive` leaves behind.
3. `TestHashlockPlatesFlowLocatorCarriesTheDerivedDigest`, which drives
   `composerHashlockPlatesFlow` on a phrase-only payload through the list, the
   derive, the form pick and the plate build, and compares the plate's locator
   against a digest THE TEST derives from `hashlock.PreimageSHA256`.

**RED, mutation 1 — omit the `hash` row when the record is a phrase:**
```
--- FAIL: TestHashlockPlatesLocatorAlwaysCarriesTheHashRow/a_phrase_record,_derived
    composer_hashlock_plates_test.go:248: locator = [], want a "hash  e7e68d52..476e1407" row:
    a phrase-form plate with no locator at all is the worst artifact this stage can cut
--- FAIL: TestHashlockPlatesFlowLocatorCarriesTheDerivedDigest
    composer_hashlock_plates_test.go:354: the plate's locator = [], want a
    "hash  f22cc3f5..81f6e837" row
```
**RED, mutation 2 — build the locator one statement before `hashlockPlatesDerive`:**
```
--- FAIL: TestHashlockPlatesFlowLocatorCarriesTheDerivedDigest
    the plate's locator = ["hash  00000000..00000000"], want a "hash  f22cc3f5..81f6e837" row
```
**GREEN, unmutated:** `ok  seedhammer.com/gui  0.077s`. Both files restored and
`diff`-verified identical after each mutation.

Spec §5.2 item 4 and §11.5 now state the property as the DIGEST rather than the
presence of a row, name the instrument, and name the mutation that fires.

## 2. The Importants

| # | change | evidence |
| --- | --- | --- |
| fid I-1 | group table rewritten from the measured file set: **E = 8a→8b→9→10→11, one implementer**, after Tasks 6 and 7; the three shared files added to every owning task's Files list | measured: Tasks 9/10/11 all write `gui/composer_copy.go`, `composer_copy_test.go`, `modal_fits_test.go` (H6 rows at `modal_fits_test.go:352-361`); Task 10's flow uses **six** symbols declared in Task 9's `composer_preimage_plate.go` (`hashlockPlate`, `hashlockPlateChoice`, `hashlockPlateDecline`, `composerPreimagePlateRows`, `hashlockPlateLocator`, `composerHashlockPlateFor`); `composer_hash.go` carries 7 `sysw.Class*`/`PhraseRecord` refs (Task 7) and `composer_preimage_plate.go` 10 `backup.` refs (Task 6) |
| fid I-2 | the ms-corpus re-vendor moves out of Task 7 into a new **Task 5b** (package `hashlock`), in group C beside Tasks 4 and 5 | **boundary re-run.** With `hashlock/` reverted to `fb0dd04`: `go test ./backup/` → *"hashlock_test.go:462: the corpus carries 0 qr_text rows; H6 §11.2 pins seven"*, i.e. Task 6's own gate fails in the declared order. In the new order: `go test ./hashlock/` ok, `go test ./backup/` ok |
| fid I-3 | `hashlock/hashlock.go` and `hashlock/methodline_h6_test.go` get File Structure rows and Task 5b's Files list; Task 9's "WHICH NO TASK WIRED" becomes "which are now Task 5b's" | `hashlock/` imports only stdlib + `seedhammer.com/seal`, so 5b needs only Task 1 |
| fid I-4 | `hashlock/methodline_h6_test.go` rewritten to read the seven vendored `qr_text` rows (`QRText` against `row.qr_text` and `row.bytes`, `MethodLine` against the row's own second line), plus a both-methods non-vacuity sweep and the 73-character geometry pin | **RED on the exact event the pin exists for**: reorder `salt=`/`iterations=` in `MethodLine` (same 73 chars) → *"row anchor-hardened: MethodLine(true) = \"…salt=ms-hashlock-v1 iterations=100000…\", want the corpus line \"…iterations=100000 salt=ms-hashlock-v1…\""* on every hardened row. Under the old literal-based test that mutation left `go test ./hashlock/` GREEN |
| fid I-5 | `TestConstantTimeQRBudgetBoundsEveryPayload` covers **{29, 33, 37, 41, 45, 49, 53}**; `h6DimForBytes` gains the four versions below 79 bytes; `TestECCLThresholdsAreWhatTheBudgetAssumes` sweeps 1..240; `h6ShapesFor` loses its 79-byte floor | GREEN, measured: `dim 29 v3 observed max 347 / budget 391 / headroom 44`, `33: 478/547/69`, `37: 617/684/67`, `41: 787/843/56`, `45: 918/1013/95`, `49: 1134/1199/65`, `53: 1348/1399/51`. **RED**: lower the SHIPPED dim-29 entry to 340 → *"dim 29, payload 9: ConstantQR: too many dims 29 QR modules for constant time engraving n: 342 waste: 8"*. No shipped entry moved |
| fid I-6 = jrn M-1 | §8.2.4 becomes *"the device needs **this payload's passphrase** before it can reach it"* | new test `the_sealed_transit_note_does_not_point_the_wrong_way`; **RED** on restoring "the passphrase above": *"§8.2.4 points ABOVE at a passphrase that is printed BELOW it"*. Verified on a real binary: the sealing line one line above says *"opens only with the passphrase below"* |
| fid I-7 = jrn I-5 | plan Task 9 Step 1 and spec §5.3 step (A) drop the "backing out of the STEP" clause; both now state the census's Button1 as the only exit and its cost | verified in code: `composerPickScreen` → `(0,false)`, `composerPreimagePlatePick` maps `!ok` to `hashlockPlateDecline`, and neither it nor `composerPreimagePlateStep` has a failure return (`composer_preimage_plate.go:182-191`) |
| fid I-8 | Task 1 Step 6 becomes `RELEASE_PROCESS.md`'s checklist with commands: the SemVer-forcing corpus SHA (item 1), CHANGELOG (2), MIGRATION.md (5), the CI gate (3), `ci/repro/vendor-freshness.sh`, the R0 gate (4), `cargo publish --dry-run` (7), the tag (8), cross-repo notification to BOTH siblings (6) | item numbers re-grepped in `mnemonic-secret/design/RELEASE_PROCESS.md` |
| jrn I-1 + M-3 | new Task 3 **Step 8b**: the `Pack` doc comment gains `phrase:<hex of "<method>,<phrase>">` with the first-comma rule; `U::Composer`'s header and build list gain the fifth prefix and its recipe; `U::Unrecognised`'s enumeration gains the word | measured before: the refusal called a `phrase:` record *"a `key:`/`hash:`/`now:` record"* and offered recipes for those three. After: the header names four prefixes and the list carries the phrase recipe |
| jrn I-2 | `composerCopyHashlockConfirm`'s middle sentence rewritten (plan Task 8a Step 5, spec §0 item 5, §8.9 row, Task 13 Step 1) | **342 drawn / headroom 107** through `assertModalBodyFits`, one character shorter than the 343/107 it replaces; `TestComposerCopyIsVerbatimFromTheSpec`'s table row updated with it |
| jrn I-3 = fid M-4 | §8.2.2 goes silent when a carrier-SHAPED record is present, via a new `hashlock_carrier_shaped` diagnostic | new test `the_no_op_warning_is_silent_when_a_carrier_shaped_record_is_present` over four shapes; **RED** on restoring the unconditional test: *"§8.2.2 fired above the refusal for the wrong-id plate, and the two contradict each other"*. A payload with no carrier shape still draws §8.2.2 (asserted in the same test) |
| jrn I-4 | §12 item 8's cost model replaced by measurement, in spec §12/§11.4 and plan Task 13 Step 4 / Task 6 Step 7b, with a new logged-and-bounded test | MEASURED by `engrave.TimePlan` at `sh2.Params()`: worst-case plate **43m31s**, without the QR **14m39s**, string form **12m14s**, the v9 QR alone **32m12s**, one 6 mm character **7s** |

## 3. Minors and Nits taken

- **fid M-1** — the `dim > 53` comment now says ECC-L caps at **230** bytes at v9
  (192 is v8's), with the 36-byte headroom; spec §7.2 item 3 matches. Measured
  from the encoder via the extended threshold sweep: first byte count reaching
  dim 53 is 193, dim 57 is 231.
- **fid M-2** — §8.2.3's payload-wide note hoisted above the per-carrier loop.
  New test `the_no_hash_record_note_prints_once_per_payload`; **RED** on moving
  it back: *"the payload-wide note printed 2 times"*.
- **fid M-3** — `ms-cli`'s `is_ms1_shaped`, `MIN_MS1_LEN` and `BECH32_CHARSET`
  DELETED; the four unit rows now drive `looks_like_ms1`, plus a fifth for the
  uppercase spelling. **The comment's claim was not merely unasserted, it was
  false**: writing the equality assertion failed —
  *"the crate-local shape test and the codec's disagree on
  \"MS10ENTRSQQQQQQQQQQQQQQQQQQQQQQQQQQQQCJ9SXRAQ34V7F\""* — because the local
  copy stripped display separators without case-folding, i.e. it answered `false`
  for the one spelling `looks_like_ms1`'s doc comment says is caught "here and
  only here". Production was never affected (production calls the delegating
  function). **RED** for the new row: drop `to_ascii_lowercase` from the codec →
  *"assertion failed: looks_like_ms1(\"MS10ENTRSQ…\")"*.
- **fid N-1 = jrn N-1** — Task 12 says FOUR runs.
- **fid N-2** — dissolved: Task 7 no longer touches `hashlock/`.
- **tests M-1** — Task 3's inline gate count was Task 2's; re-measured on the
  folded tree: **633 run, 630 passed, 3 failed (`history_purge`), 2 skipped**.
- **tests M-2** — Task 3's test table carried three names that do not exist;
  every row now carries the shipped name, and the three new tests are rows.
- **tests, attribution** — Task 11's §9 mutation is caught by
  `TestSyswWarnMS1ShapedFiresOnceAndIsReArmedByAnEdit`. VERIFIED:
  `TestHashlockMS1WarningFiresOnAGroupedPlate` calls `hashlock.IsMS1Shaped`
  directly (`sysw_session_test.go:127`) and never reaches `syswWarnMS1Shaped`.
- Counts corrected while re-grepping: Task 6's tests are **10**
  (`grep -c '^func Test'`), not the "Seven" its own step said against a nine-row
  table; `sysw_pack_preimage.rs` is **12** tests, not nine.

## 4. Recorded rather than fixed, with reasons

- **jrn M-4** (decline-all reaches §8.4a's end state and draws no arm) — VERIFIED
  structurally at `gui/composer_flow.go:422-441`: declining every plate leaves
  `accepted` empty, `cut` stays 0, `bundleEngrave` completes. A refusal would be
  wrong by §5.3 item 5 and a third confirm arm is new behaviour, so it is
  recorded in spec §13 and the plan's follow-up table — which is the finding's
  own second option — together with the measured mitigation (the census
  withholds Button3 until its last page).
- **jrn M-2** (`--expect` has no `preimage` kind) — MEASURED: *"me: unknown
  --expect kind \"preimage\"; want one or more of descriptor, cosigner,
  transaction, mnemonic, secret"*, exit 2; `--expect secret` is not satisfied by
  a plate. Filed to a later `me` cycle, as the finding proposes.
- **jrn N-2** (the plates flow returns to an unmarked list) — filed, spec §13.

## 5. Declined, with a reason

- **jrn I-2's suggested wording** (*"This composition holds the phrase and method
  until it ends, and can cut a plate for them at Done…"*). The finding's premise
  — *"the current body is well inside the margin, so there is room"* — is FALSE,
  measured: the body has **107** characters of headroom against
  `modalBodyMargin = 80`, i.e. 27 characters of room. The suggested wording
  measures **364 drawn / headroom 64** and
  `TestConfirmScreensThisBlockTouchesAreDrawnInFull` refuses it: *"fits today
  with only 64 characters to spare, under the 80-character margin"*. The
  substance (the composition holds the material) is folded in a sentence one
  character SHORTER than the false one it replaces.
- **jrn I-3's predicate was widened by one case** rather than declined: the L24
  id/kind mismatch is diagnosed as neither a plate nor a phrase, so
  `preimage_plate(r) || r.starts_with(PHRASE_PREFIX)` alone would have left the
  `entr` mistag printing §8.2.2 above its own refusal. Measured on all four
  shapes.

## 6. Found by the fold itself, outside the three reports

- **`cargo fmt --check` was in no boundary gate and was RED in both Rust trees** —
  five files in `me` (`main.rs`, `sysw/composer_records.rs`, `sysw/mod.rs`,
  `sysw/record.rs`, `tests/sysw_pack_preimage.rs`) and two hunks in `ms`
  (`tests/hashlock_qr_text.rs`), every one H6's own code. Both trees are clean
  now; the plan's Global Constraints and the Rust gate rows name the check. This
  matters beyond hygiene: `RELEASE_PROCESS.md` item 3 requires it at the release
  Task 1 Step 6 performs, so the stage would have reached its one irreversible
  action with a gate nobody had run. Three plan blocks were re-synced to the
  reformatted text and the checker re-run.
- **a zero-byte `crates/ms-codec/tests/hashlock_vectors.rs`** in the `ms` tree
  that exists in no ms revision (`git ls-tree 504ff46 crates/ms-codec/tests/`
  lists `hashlock_derivation.rs`, `hashlock_kind.rs`, `hashlock_repro.rs` only).
  Deleted; it was the third `cargo fmt` complaint.
- **clippy cannot be proved in these trees** — both repos pin `1.85.0` in
  `rust-toolchain.toml` and this box resolves `rustc 1.98.0`, whose clippy warns
  about `manual_is_multiple_of`, `manual div_ceil` and elided lifetimes in files
  H6 never touches, with rewrites 1.85 cannot compile. Task 1 Step 6 says so
  instead of claiming a green it cannot have.

## 7. Gates, re-run over the folded trees

```
ms    cargo fmt --check                              clean
ms    cargo nextest run --locked                     562 tests run: 562 passed, 11 skipped
me    cargo fmt --check                              clean
me    cargo nextest run --locked -p mnemonic-engrave --no-fail-fast
                                                     633 tests run: 630 passed, 3 failed,
                                                     2 skipped  (history_purge x3, the
                                                     box-local /usr/bin/zsh baseline)
fork  go test ./engrave/ ./backup/ ./codex32/ ./sysw/ ./hashlock/ ./cmd/emu/    all ok
fork  scripts/gui-shard-test.sh ./gui/ 24            1286 top-level tests, partition
                                                     verified exhaustive: 1286 == 1286,
                                                     RESULT: ok -- 24 of 24 shards, wall 27s
fork  gofmt -l gui/ sysw/ backup/ engrave/ codex32/ hashlock/ cmd/emu/
                                                     the three baseline files, no fourth
fork  go vet ./gui/                                  the two baseline ArtifactDir diagnostics
fork  GOOS=js GOARCH=wasm go vet ./cmd/emu/          exit 0
fork  ./cmd/emu/build.sh                             exit 0, built emu.wasm (11011084 bytes)
fork  go build ./...                                 ok
fork  nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 \
        -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
                                                     1611456 code / 32124 data / 31148 bss
                                                     flash 1643580, ram 63272
                                                     -- byte-identical to the build gate's
                                                     wired figure, so the fold costs nothing
engrave scripts/h6-plan-blocks-vs-tree.sh            97 blocks checked, 0 FAIL
```

**The browser walk was NOT re-run**, and that is stated rather than papered over:
no browser is available to this agent. `cmd/emu/walk_hashlock_phrase.js` is
byte-unchanged (sha256
`990e67812a253061a18fe75df7ed4e83f982c4bcaf6b11b74878e40c8d6629c9`, the literal
Task 12 records), and the only device copy this round edits is the HOLD confirm
modal's middle sentence, which the walk does not assert — it waits for
`Write down this phrase` and asserts `method: `, `chars: ` and
`One phrase per policy`, all unchanged (grepped). `go test ./cmd/emu/`, the wasm
vet and `build.sh` all pass over the folded tree.

## 8. Discipline

- Every code change was made in the tree and in the plan block, and
  `scripts/h6-plan-blocks-vs-tree.sh` (97/0) is what proves they agree — run
  after every edit, not only at the end.
- Every citation this fold ADDED was re-grepped after the edits that shifted
  lines: `backup/hashlock_test.go:447`, `gui/composer_preimage_plate.go:290,292`
  and `:140-144`, `gui/composer_copy.go:712/722/731/739/767/785` (wired) and
  `:426`/`:437` (at `fb0dd04`), `gui/composer_copy_test.go:203,209`,
  `gui/modal_fits_test.go:358-361`, `gui/composer_flow.go:422-441`,
  `hashlock/hashlock_test.go:13`, `RELEASE_PROCESS.md` items 1-8.
- Superseded phrasing swept: no live "Group F/G", "disjoint from every other
  group", "nine tests", "Four shipped records", "192 bytes", "the passphrase
  above", "run it three times", "WHICH NO TASK WIRED" or `is_ms1_shaped` remains
  outside a deliberate quotation of what was replaced.
- Plan STATUS is now **DRAFT — R0 round 0 folded; r1 fold verification
  pending**; the spec's STATUS is untouched, as the brief directs, and its
  `## Plan-round fold` section gained a subsection listing the nine spec changes.
- Every device copy string this round touches is ASCII (checked
  programmatically and by `TestComposerCopyIsDrawable`). No phrase or preimage
  bytes were printed to any log. Nothing was committed; the reports and the repo
  checkouts were not edited.
