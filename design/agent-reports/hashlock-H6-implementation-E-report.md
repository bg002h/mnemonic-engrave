# H6 implementer E — Tasks 8a, 8b, 9, 10, 11 (the fork `gui` surface)

**Branch:** `h6-e` off `hashlock-h6` at `aeb1a070` (Task 7, implementer D).
**Tip:** `bdb669638cc20f441940eb075c15b1b6a6430f46`.
**Worktree:** `/scratch/code/shibboleth/.tmp/seedhammer-h6-e`.
**Nothing pushed. No `main`/`master` commit. Four commits, one per task boundary
(8a and 8b in one, as the plan requires).**

| task | commit | boundary gate |
| --- | --- | --- |
| 8a + 8b | `56b5de8375de1f03596886d183aa0e8af23d4b99` | `gui-shard-test.sh ./gui/ 24` — **1250** tests, 24/24 shards ok, wall 24 s |
| 9 | `54aea54aa2412d4a61a52b17b119bf539cf6beab` | **1271** tests, 24/24 ok, wall 25 s |
| 10 | `d3c75a909645ba2b36f2932f858b411cc0f5d472` | **1279** tests, 24/24 ok, wall 25 s |
| 11 | `bdb669638cc20f441940eb075c15b1b6a6430f46` | **1287** tests, 24/24 ok, wall 36 s |

Every gate ran `scripts/gui-shard-test.sh ./gui/ 24`, which asserts its partition
exhaustive before running. Raw output:
`/scratch/code/shibboleth/.tmp/h6e-gate-{8b,9,10,11}.txt`.

1250 / 1271 / 1279 are the plan's own numbers to the test. **1287 is 1286 + 1**,
and the one is a test this implementer added — see Deviation 3.

27 files changed, 4,536 insertions, 30 deletions against `aeb1a070`.

---

## Gates run beyond the four boundaries

- `go build ./...` — clean at every task boundary.
- **Every non-`gui` package**, at the final tip:
  `go test $(go list ./... | grep -v /gui$ | grep -v third_party)` — **exit 0**.
- `gofmt -l .` — **adds no file**. See Deviation 1 for what the baseline actually is.
- **Firmware, the plan's nix recipe**, measured at both ends on this box:

  | tree | flash | RAM |
  | --- | --- | --- |
  | `aeb1a070` (branch base, after Task 7) | 1,600,272 | 63,248 |
  | `bdb6696` (this tip, after 8a–11) | **1,643,580** | **63,272** |
  | delta | **+43,308 B** | **+24 B** |

  The plan's `composerPreimagePlates` comment claims `sort.Slice` measures
  1,643,580 B of flash against 1,644,120 for a hand-rolled insertion sort. The
  first figure reproduces **exactly** on this tree.

---

## RED evidence, per task

Every task was driven test-first: the tests were written and run against the
un-implemented tree, the failure captured, then the production code written.

**8a** — `/scratch/code/shibboleth/.tmp/h6e-red-8a.txt`:
```
gui/composer_hashlock_held_test.go:27:8: st.hashlockHeld undefined (type *composerState has no field or method hashlockHeld)
gui/composer_hashlock_held_test.go:42:2: undefined: composerHoldHashlockMaterial
gui/composer_hashlock_held_test.go:42:38: undefined: hashlockMaterial
gui/composer_hashlock_held_test.go:46:15: undefined: hashlockFromPhrase
FAIL	seedhammer.com/gui [build failed]
```

**8a Step 5** (the falsified device body) — the table row was changed first, so
the RED is the body still saying the false thing
(`/scratch/code/shibboleth/.tmp/h6e-red-8a-step5.txt`):
```
composer_copy_test.go:181: composerCopyHashlockConfirm (SPEC §H2-4.5) does not match the spec.
 got:  "... Write down this phrase, the method and this digest now. The phrase and method are not on this device. ..."
 want: "... Write down this phrase, the method and this digest now. This composition holds them until it ends. ..."
```

**8b** — `/scratch/code/shibboleth/.tmp/h6e-red-8b.txt`:
```
gui/composer_hash_test.go:94:31: too many arguments in call to composerHashRows
	have (*syswSession, nil)
	want (*syswSession)
gui/composer_hash_test.go:180:16: rows.preimages undefined (type composerHashRowSet has no field or method preimages)
gui/composer_hash_test.go:185:12: rows.preimageRow undefined (type composerHashRowSet has no field or method preimageRow)
```

**9** — `/scratch/code/shibboleth/.tmp/h6e-red-9.txt`:
```
vet: gui/composer_census_test.go:42:70: too many arguments in call to composerCensusLines
	have (engrave.Params, []bundleCard, nil)
	want (engrave.Params, []bundleCard)
gui/composer_preimage_plate_test.go:73:20: undefined: composerPreimagePlates
gui/composer_preimage_plate_test.go:74:23: undefined: hashlockDigestHex
```

**10** — `/scratch/code/shibboleth/.tmp/h6e-red-10.txt`:
```
gui/composer_hashlock_plates_test.go:50:14: undefined: composerDoorHasPreimage
gui/composer_hashlock_plates_test.go:82:35: assignment mismatch: 4 variables but composerDoorCounts returns 3 values
gui/composer_hashlock_plates_test.go:112:10: undefined: hashlockPlatesRecords
```

**11** — `/scratch/code/shibboleth/.tmp/h6e-red-11.txt`:
```
gui/sysw_session_test.go:62:13: undefined: syswWarnMS1Shaped
gui/sysw_session_test.go:165:6: undefined: syswNoticeHashlockPhrase
gui/composer_copy_test.go:214:48: undefined: composerCopyHashlockLooksLikeMS1
```

---

## Every body measured

All measured through `assertModalBodyFits` on the renderer production uses, at
`sh2DisplaySize`, against `modalBodyMargin = 80`. **Every figure the plan
carried is confirmed to the character, and none was taken from the plan.**

| body | renderer | drawn | headroom | plan says |
| --- | --- | --- | --- | --- |
| §H2-4.5 confirm, rewritten (8a Step 5) | `confirmWarningBody` + `composerConfirmBody` | 342 | 107 | 342 / 107 ✓ |
| §5.1 payload preimage-record confirm | same | 274 | 186 | 274 / 186 ✓ |
| §8.5 QR warning | same | 126 | 378 | 126 / 378 ✓ |
| §9 ms1-shaped warning | same | 204 | 302 | 204 / 302 ✓ |
| §8.4a no preimage plate was cut | `errorScreenBody` | 75 | 476 | 75 / 476 ✓ |
| §8.4b a preimage plate was cut | `errorScreenBody` | 86 | 476 | 86 / 476 ✓ |
| §10.1 held form | `errorScreenBody` | 185 | 360 | 185 / 360 ✓ |
| §10.1 held-phrase form | `errorScreenBody` | 186 | 360 | 186 / 360 ✓ |
| §8.3 stand-alone notice | `errorScreenBody` | 107 | 455 | 107 / 455 ✓ |
| §5.3 preimage-plate refusal | `errorScreenBody` | 99 | 455 | 99 / 455 ✓ |
| §5.2 flow's own abort | `errorScreenBody` | 80 | 476 | 80 / 476 ✓ |
| §5.2 empty-payload refusal | `errorScreenBody` | 46 | 513 | 46 / 513 ✓ |
| §8.8 Password-program notice | `errorScreenBody` | 165 | 397 | 165 / 397 ✓ |
| §9 warning on `errorScreenBody` | `errorScreenBody` | 184 | 378 | 184 / 378 ✓ |

Raw: `/scratch/code/shibboleth/.tmp/h6e-measure-9.txt` and the `-v` runs of
`TestModalsThisBlockTouchesAreDrawnInFull` /
`TestConfirmScreensThisBlockTouchesAreDrawnInFull`.

---

## Mutations — every one executed, its failure quoted, and reverted

Raw tails: `/scratch/code/shibboleth/.tmp/h6e-mut-{8a,8b,9,10,11,M8}.txt`.
Each was applied to a snapshot, run, and the snapshot restored; the scoped suite
was re-run green after every revert.

### Task 8a — three

1. **assign without the nil check** →
   `panic: assignment to entry in nil map [recovered, repanicked]`
2. **remove the scrub from `composerFlowExit`** →
   `composer_hashlock_held_test.go:83: the phrase survived the flow-exit defer: "correct horse battery staple"`
   `composer_hashlock_held_test.go:87: the preimage survived the flow-exit defer: abcdef00…`
   `composer_hashlock_held_test.go:142: composerFlowExit does not call composerScrubHashlockHeld`
3. **wipe without writing the value back** →
   `composer_hashlock_held_test.go:87: the preimage survived the flow-exit defer: abcdef00…`
   `composer_hashlock_held_test.go:90: the phrase slice was not released: [0 0 0 …]`
   — the phrase-equality assertion still passes, exactly as the plan predicts,
   which is why both are asserted.

### Task 8b — six

1. **band order swapped** → `composer_hash_test.go:187: band starts 2/1/3/4/5 with n=1`
2. **derive at row-build time** → `TestWhichHashDoesNotDeriveAtRowBuildTime` and all
   three `TestWhichHashRowsCarryTheTwoNewBands` sub-rows fail
3. **`(in payload)` dropped** →
   `composer_hash_test.go:228: the hash: row whose digest the payload also carries is not annotated: "hash 1  b867db87..edbc96cb"`
4. **a payload phrase through `hashlockPhraseRoute`** →
   `TestComposerHashEditDispatchesTheTwoNewBands/the_phrase-record_row_derives_and_never_picks_a_method` fails
5. **the shipped three-arm `taking` restored** →
   `composer_hashlock_test.go:1181: never reached "32-byte value"; last frame "hashb867db87..edbc96cbfromapreimagerecord…"`
   (and the same at `:1213` for the phrase-record band) — the 32-byte rule never states
6. **the HOLD removed from both payload routes** →
   `composer_hashlock_test.go:1194: the preimage was not held (0 entries)`
   `composer_hashlock_test.go:1238: the derived material was not held (0 entries)`

### Task 9 — eleven

1. **the two phrase rows offered with no phrase held** →
   `composer_preimage_plate_test.go:140: a payload preimage record offers [preimage string phrase + method phrase + method + QR do not cut this preimage]; the two phrase forms have no phrase to engrave`
2. **the declined plate dropped silently** →
   `composer_preimage_plate_test.go:304: the census does not carry §8.3's row "preimage f20a0890..94165eb9: declined, will not be cut"`
   and `:867: the census does not NAME the declined plate.`
3. **the census drawn through `confirmReviewScreen`** →
   `composer_preimage_plate_test.go:1126: composerEngraveStep does not draw its census through composerReadScreen, which is the only paged confirm that wraps inside composerTextBand`
4. **the accepted plates added as `bundleCard`s** →
   `composer_preimage_plate_test.go:1073: composerEngraveStep never returned.` (see Deviation 4)
5. **the preimage loop moved after `bundleEngrave`** → three tests fail:
   `TestComposerCutsPreimagePlatesBeforeThePolicySet`,
   `TestComposerAbortAfterAPreimagePlateWasCutDrawsSection84b`,
   `TestComposerCompletedRunDrawsNeitherAbortArm`
6. **§8.4b fired unconditionally** →
   `composer_preimage_plate_test.go:1080: an abort arm was drawn on a COMPLETED run.`
   `Frame: "APREIMAGEPLATEWASCUTandnopolicyplatewas.Storeordestroyitnow;donotleaveitwiththeblanks.WalletPolicy"`
7. **§8.4a fired unconditionally** → the same test, `Frame:
   "NOPREIMAGEPLATEWASCUT.Thephrasedieswiththiscomposition.Donotfundthiswallet.WalletPolicy"`
8. **§8.4a attached to `bundleAbortWarningText`** →
   `composer_preimage_plate_test.go:804: §8.4a never drew.` — the arm is unreachable,
   which is the property `bundleAbortWarningText` is UNCHANGED for
9. **§10.1's future tense restored** →
   `composer_preimage_plate_test.go:554: §10.1's body does not state the possibility in the present tense: "…This run cuts a plate for each one…"`
10. **the mark suppressed** → `TestComposerPreimageMarkTitleMarksMd1AndMk1ButNeverMs1` fails
11. **an em dash in `composerCopyAbortNoPreimage`** →
    `modal_fits_test.go:360: the modal drew only 5004 ink pixels (floor 6000) -- this frame is near blank`
    — **the spec's own predicted number, exactly** — plus
    `composer_copy_test.go:248: composerCopyAbortNoPreimage carries the non-ASCII or control rune '—'; device strings are ASCII only`

### Task 10 — seven

1. **derive per pick without the once guard** →
   `composer_hashlock_plates_test.go:182: the second pick took 11.644901ms: the KDF ran again`
2. **the locator's `hash` row omitted for a phrase record** →
   `composer_hashlock_plates_test.go:212: locator = [], want exactly ["hash  b867db87..edbc96cb"]`
3. **the locator built one statement early, before `hashlockPlatesDerive`** →
   `composer_hashlock_plates_test.go:354: the plate's locator = ["hash  00000000..00000000"], want a "hash  f22cc3f5..81f6e837" row. A locator built before the derive reads hash  00000000..00000000 and names nothing: a bearer phrase in plain text with no digest, no path and no payload position`
4. **§8.4a reused in this flow** → `TestHashlockPlatesAbortDrawsNeitherSection84Arm` fails
5. **the route offered when the payload holds neither class** →
   `TestComposerDoorOffersHashlockPlatesOnlyForAPayloadThatHasOne/{key_records_only,a_hash:_record_but_no_material}` fail
6. **derive at LIST time** →
   `composer_hashlock_plates_test.go:115: one hardened derivation 10.242043ms; the list 30.817384ms`
   `:122: row 0 is "phrase 1  3cf5d421..b70a4c12", want "phrase record 1 (derive to see the digest)"`
7. **a `composerState` reference in the file** →
   `composer_hashlock_plates_test.go:462: composer_hashlock_plates.go references composerState; decision 4 scopes this route to ctx.sysw alone`

### Task 11 — four, plus the added gate's own

1. **`codex32.IsPreimage` instead of `hashlock.IsMS1Shaped`** →
   `passphrase_flow_test.go:1397: §9's warning did not fire at the passphrase entry screen`
   and `TestSyswWarnMS1ShapedFiresOnceAndIsReArmedByAnEdit` fails with it. The
   standalone `TestHashlockMS1WarningFiresOnAGroupedPlate` stays green, exactly as
   R0 round 0's tests lens recorded.
2. **refuse instead of warning** → `TestPassphraseMS1WarningWarnsAndStillCuts` and
   `TestSyswWarnMS1ShapedFiresOnceAndIsReArmedByAnEdit` fail
3. **the notice helper stops drawing** →
   `sysw_session_test.go:224: the notice drew = false, want true.`
4. **the notice drawn when a `pass:` record is also present** →
   `TestSyswNoticeHashlockPhraseFiresOnItsCondition/both_classes_present` fails
5. **(added) the CALL SITE deleted from `engravePassphraseFlowFrom`** →
   `sysw_session_test.go:190: engravePassphraseFlowFrom draws no §8.8 notice: the operator whose payload holds a hashlock phrase still meets the ordinary keyboard with no offer, no mention and no reason`

### The ADDENDUM's mutation, M8 — twice

At the Task 8b tip, after the oracle extension:
```
sysw_admit_oracle_test.go:244: composer_hash.go:composerPayloadPreimages names ClassPreimage, which §3.3.2 REFUSES to program 7 (H6 §5.1 band 2 …)
sysw_admit_oracle_test.go:244: composer_hash.go:composerPayloadPhrases names ClassPhrase, which §3.3.2 REFUSES to program 7 (H6 §5.1 band 3 …)
```
At the Task 10 tip, once the switch form was matched too — **six sites**:
```
composer_door.go:composerDoorCounts names ClassPreimage, which §3.3.2 REFUSES to program 7
composer_door.go:composerDoorCounts names ClassPhrase, which §3.3.2 REFUSES to program 7
composer_hash.go:composerPayloadPreimages names ClassPreimage, which §3.3.2 REFUSES to program 7
composer_hash.go:composerPayloadPhrases names ClassPhrase, which §3.3.2 REFUSES to program 7
composer_hashlock_plates.go:hashlockPlatesRecords names ClassPreimage, which §3.3.2 REFUSES to program 7
composer_hashlock_plates.go:hashlockPlatesRecords names ClassPhrase, which §3.3.2 REFUSES to program 7
```

---

## Deviations

### 1. The `gofmt` baseline is FIVE files, not the three the plan's constraint names

MEASURED on a pristine `git archive` of fork main `fb0dd04` and again on
`hashlock-h6`:

```
gui/transaction.go
gui/transaction_golden_test.go
gui/transaction_txrecord_test.go
mt/mt.go
mt/mt_test.go
```

The plan's Global Constraint 1 says three and forbids a fourth. The true
baseline is five, unchanged since `fb0dd04`. **H6 adds no sixth** — the
constraint's actual intent holds, but the number in the plan is wrong and any
implementer checking `-l | wc -l` against 3 would report a false red. **Not
fixed here** (it is a plan text change, not code); recorded for Task 13.

### 2. THE ADDENDUM'S PREMISE WAS FALSE, and its remedy would have broken the oracle

The addendum's finding is real and was reproduced: **D's `progWalletPolicy` row
for `ClassPreimage`/`ClassPhrase` had no gate.** Deleting both cells at the
branch base left the package green, and no test in `gui/` named either class.

But the prescribed remedy does not apply. **Neither Task 8b nor Task 10 adds a
`syswOffer(...)` or `.take(...)` site.** Both read `s.records` and filter on
`r.class` — the shape the SHIPPED `composerPayloadDigests` (`ClassHash`) and
`composerBoundFrom` (`ClassNow`) already used, and which
`sysw_admit_oracle_test.go`'s matcher (`syswOffer` prefix; `take`/`takeAll`/
`cardSet` selectors) cannot see. Registering entries in `syswConsumers` that the
matcher never finds would have fired the oracle's own tail arm —
`INCONCLUSIVE: only %d consumption sites found for %d mapped entries — a mapped
site has vanished and this test now guards less than it claims`.

So the minimal correct thing was to give the oracle the shape it was missing:

- **Task 8b** added a second matcher for `r.class == sysw.ClassX` /
  `!=` comparisons (`syswFilteredClass`), registered the **four** sites it finds
  — two of them SHIPPED and never reconciled against §3.3.2 at all — and added
  `ClassPreimage`/`ClassPhrase` to `classNames`.
- **Task 10** was caught by that extension automatically:
  `NEW consumption site composer_hashlock_plates.go:hashlockPlatesStub (class ClassMDMK) is not in syswConsumers`.
  Registering it exposed the remaining hole — a site keeping TWO classes writes a
  `switch r.class`, not two comparisons — so a third matcher
  (`syswSwitchedClasses`) was added and the door's own counts registered with it.
- **`ClassUnknown` is exempted from the reconciliation**, by construction rather
  than convenience: it is the inert class, refused to every program,
  `TestUnknownIsRefusedEverywhere` pins that whole row on its own, and
  `composerDoorCounts` names it to COUNT what will NOT be used. Without the
  exemption the oracle reported the door as an admission violation for saying so.

Net: **22 consumption sites reconciled against §3.3.2**, up from 19 before H6,
and M8 reds at six of them. The addendum's stated goal — prove the admission row
has a gate — is met; the route to it is different from the one prescribed, for
the reason above.

### 3. ONE TEST ADDED — §8.8's call site had no gate

`TestSyswNoticeHashlockPhraseFiresOnItsCondition` drives
`syswNoticeHashlockPhrase` **directly**, so deleting the call from
`engravePassphraseFlowFrom` leaves it green: §8.8 ships as an inert function and
today's silence is unchanged. The composer's join guard cannot catch it either —
`syswNoticeHashlockPhrase` HAS a production caller, so the guard is satisfied by
the very line at issue. This is the "plans list components and omit the call that
joins them" class, and it is the one Task 11 was most exposed to, because §8.8's
whole content is *the absence of a screen*.

`TestPasswordProgramActuallyDrawsTheHashlockPhraseNotice` asserts the call on the
AST (the same instrument `TestComposerScrubIsInTheExistingDefer` uses, and for
the same reason: there is nothing at runtime to distinguish "no notice drew"
from "there is no notice"). **This is the entire difference between the plan's
1286 and this tip's 1287.**

### 4. Task 9's `bundleCard` mutation reds at a different test than the plan predicts

The plan says adding the accepted plates as `bundleCard`s makes
"`bundlePlatePlan`'s count and the 'all of it exists' claim change". It does not.
`TestComposerPreimagePlateIsNotABundleCard` is a **purity** unit test on
`buildPlateCensusLines(params, cards)` and cannot see the flow appending a card
to its own local slice. The mutation IS caught — by
`TestComposerCompletedRunDrawsNeitherAbortArm`, `composerEngraveStep never
returned`, because the extra card adds an engrave screen the harness does not
tap through. The property is gated; the plan's attribution of which test gates it
is wrong.

### 5. `composer_state.go` — the new declarations placed AFTER `composerNotePhraseDigest`

The plan's Step 2/3 blocks are `mode=fragment` and carry no placement. The gated
scratch tree inserted them **between `composerNotePhraseDigest`'s doc comment and
its `func` line**, which detaches a shipped doc comment from its function and
attaches four paragraphs about `phraseDigests` to `hashlockProvenance`. Placed
after the function instead. The block text itself is byte-identical to the plan's.

---

## What was NOT done, and is not mine

- **Task 12** (the emulator walk) and **Task 13** (records) belong to group F.
- The plan's Global Constraint 2 predicts `go vet ./engrave/` grows from 2 to 6
  `testing.ArtifactDir` diagnostics. Confirmed as a pre-existing baseline red per
  the controller's brief; not touched.
- No `ms`/`me` (Rust) work: Tasks 8a–11 are fork-only, and no Cargo command was
  run.

---

## Method note

The plan wired Tasks 8b–12 in the build-gate agent's scratch tree
(`/scratch/code/shibboleth/.tmp/h6-gate`) but inlines only a subset of the
resulting files as `mode=fragment` blocks — `gui/composer_preimage_plate.go`
(496 lines), `gui/composer_hashlock_plates.go` (285) and their three test files
(2,088 lines between them) appear in the plan as four fragments and a handful of
prose paragraphs. Per the brief's licence to READ that tree to resolve a block
whose context is unclear, it was used as the reference for the parts the plan
specifies but does not inline; the tree was never modified, and nothing was taken
without being split back to its own task, gated at that task's boundary, and
mutated here. Every number in this report comes from a run at a commit on `h6-e`,
captured under `/scratch/code/shibboleth/.tmp/` and quoted from the capture.
