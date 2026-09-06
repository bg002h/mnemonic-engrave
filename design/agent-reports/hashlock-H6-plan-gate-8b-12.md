# H6 plan build gate — Tasks 8b-12 (the fork's `gui` surface)

**Agent:** build gate (opus). **Artifact:** `design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md`
at engrave master `4b9a4dd8`. **Brief:** `design/agent-briefs/hashlock-H6-plan-gate-8b-12-brief.md`.
**Tree:** the plan author's own fork scratch tree `/scratch/code/shibboleth/.tmp/h6-gate`
(seedhammer fork, baseline `fb0dd04`), continued from its Task-8a state and left
in place. Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go/bin/go`.

**VERDICT: GATE GREEN WITH FIXES (17).** All five tasks are wired, built,
tested and mutated; 40 mutations executed and every one went red with the
evidence quoted below; the emulator walk RAN, four times, in a browser. Seventeen
things the plan said were wrong, unimplementable, or absent, listed in §6 with
what was changed and why. Nothing remains RED.

---

## 1. What was done, per task

Order was 8b → 9 → 10 → 11 → 12, one at a time, each with its RED quoted before
the implementation, its GREEN after, every declared `MUTATION:` executed with
its failure quoted and reverted, then its own tests, then the whole `gui` package
through `scripts/gui-shard-test.sh ./gui/ 24`.

| task | tests added | package total after | shard result |
| --- | --- | --- | --- |
| 8b | 6 | 1250 | 24 of 24 ok |
| 9 | 15 | 1271 | 24 of 24 ok |
| 10 | 7 | 1278 | 24 of 24 ok |
| 11 | 6 | 1285 | 24 of 24 ok |
| 12 | walk arm | 1285 | 24 of 24 ok |

Every shard run reports `partition verified exhaustive`, so no test was silently
dropped. Final sweep: `go test ./engrave/ ./backup/ ./codex32/ ./sysw/
./hashlock/ ./cmd/emu/` — **all ok**.

---

## 2. RED and GREEN, per task

### Task 8b — `Which hash?` gains two bands (§5.1)

**RED** (compile-level, the signature the new bands force):

```
vet: gui/composer_hash_test.go:176:32: too many arguments in call to composerHashRows
	have (*syswSession, *composerState)
	want (*syswSession)
```

**GREEN:** `TestWhichHashRowsCarryTheTwoNewBands` (0/1/2 of each class),
`TestWhichHashAnnotatesAHashRowThePayloadAlsoCarries`,
`TestWhichHashRowsDrawOnOneLine`, `TestComposerHashEditDispatchesTheTwoNewBands`
(4 sub-tests, driven BY TOUCH), `TestWhichHashDoesNotDeriveAtRowBuildTime`,
`TestWhichHashPhraseRowShowsTheDigestOnceDerived`,
`TestWhichHashRuleFiresForTheTwoNewBands`, `TestWhichHashPageHoldsFiveRows` —
`ok seedhammer.com/gui 2.038s`.

### Task 9 — the Done review (§5.3, §5.4, §8.3, §8.4, §10.1, §10.3)

**RED** (the census's signature change, which is §5.3 item 2 / r0 fidelity N-3):

```
vet: gui/composer_census_test.go:42:68: not enough arguments in call to composerCensusLines
	have (engrave.Params, []bundleCard)
	want (engrave.Params, []bundleCard, []hashlockPlate)
```

**GREEN:** fifteen tests in `gui/composer_preimage_plate_test.go`, including two
that drive `composerEngraveStep` through a COMPLETED cut on the test engraver
(`TestComposerAbortAfterAPreimagePlateWasCutDrawsSection84b`,
`TestComposerCompletedRunDrawsNeitherAbortArm`). `ok seedhammer.com/gui 3.766s`.

### Task 10 — the Hashlock plates flow (§5.2, §6.3)

**RED:** `composerDoorHasPreimage`, `composerHashlockPlatesFlow`,
`hashlockPlatesRecords` undefined; and after the door's fourth count landed,

```
vet: gui/composer_door_test.go:64:24: assignment mismatch: 3 variables but composerDoorCounts returns 4 values
```

**GREEN:** seven tests in `gui/composer_hashlock_plates_test.go`. `ok 0.072s`.

### Task 11 — §9's warning and §8.8's notice

**RED:** `syswWarnMS1Shaped`, `syswNoticeHashlockPhrase`,
`composerCopyHashlockLooksLikeMS1`, `composerCopyHashlockPhraseNotPassphrase`
undefined.

**GREEN:** `TestH6MS1ShapedFixtureIsWhatItClaims`,
`TestSyswWarnMS1ShapedFiresOnceAndIsReArmedByAnEdit`,
`TestHashlockMS1WarningFiresOnAGroupedPlate`,
`TestSyswNoticeHashlockPhraseFiresOnItsCondition` (4 rows),
`TestFTMS1WarningWarnsAndStillCuts`, `TestPassphraseMS1WarningWarnsAndStillCuts`,
`TestPassphraseMS1WarningIsSilentForAnOrdinaryPassphrase`.

### Task 12 — the walk (§11.7)

**RUN, in a browser, against the real emulator** — `cmd/emu` built with
`./cmd/emu/build.sh` (`built emu.wasm (11010867 bytes)`) and served on a local
port; the walk imported and `run()` awaited.

| run | tree | result |
| --- | --- | --- |
| (a) | unmutated | **`ok: true`**, 60.3 s |
| (b) | census row's digest dropped | **RED**: *"the census row carries the token the confirm modal drew: the screen does not carry \"3cf5d421..b70a4c12\""* — screen reads `path1preimagestring` |
| (c) | census row's digest perturbed one nibble | **RED**: same assertion — screen reads `path13df5d421..b70a4c12` |
| (d) | unmutated, after both reverts | **`ok: true`**, 60.3 s |

`git checkout -- <file> && git diff --quiet` is not available (the gate trees are
`git ls-files | tar` copies with no `.git`), so the walk file's integrity between
runs is proven by **sha256 `990e67812a253061a18fe75df7ed4e83f982c4bcaf6b11b74878e40c8d6629c9`**,
verified identical before and after each mutation — a stricter instrument for the
same claim.

The arm's own returned frames, verbatim from run (d):

```
pick:   Preimageplatehash3cf5d421..b70a4c12path1phrase:28charactersmethod:hardened
        preimagestringphrase+methodphrase+method+QRdonotcutthispreimage
census: PlatesToCutPlus1preimageplate(s),cutfirstandNOTpartofthisbackup:
        path13cf5d421..b70a4c12preimagestringKeepeachpreimageplateapartfrom
        thepolicyplatesandfromtheothers.
```

Both the §5.3 pick screen and the §8.3 census carry `3cf5d421..b70a4c12`, the
token the confirm modal painted — and the pick screen carries `phrase: 28
characters` and NOT one character of the phrase.

`GOOS=js GOARCH=wasm go vet ./cmd/emu/` — exit 0. `go test ./cmd/emu/` — ok.
`needle_test.go` needed no change: `out.ok` is still `= true` set after the last
assertion, the strongest shape `TestWalkOkContainsNoDriverSuppliedPlateCount`
accepts.

---

## 3. Every mutation, with its failure

**Task 8b (6 of 6 red)**

| # | mutation | failure |
| --- | --- | --- |
| 8b-1 | append the phrase-record band before the preimage band | `band starts 2/1/3/4/5 with n=1`; and the dispatch test lands on the wrong route's confirm |
| 8b-2 | derive at row-build time | `one hardened derivation 10.360674ms; composerHashRows 10.600239ms` + `the underived row does not say so: "phrase 1  3cf5d421..b70a4c12"` |
| 8b-3 | drop the `(in payload)` annotation | `the hash: row whose digest the payload also carries is not annotated: "hash 1  b867db87..edbc96cb"` |
| 8b-4 | route a payload phrase through `hashlockPhraseRoute` | `never reached "Write down this phrase"; last frame "...Hashlockphrase"` — the PHRASE KEYBOARD drew |
| 8b-5 | restore the shipped three-arm `taking` | `never reached "32-byte value"` on BOTH new bands |
| 8b-6 | drop the HOLD from the payload routes | `the derived material was not held (0 entries)` |

**Task 9 (11 of 11 red)**

| # | mutation | failure |
| --- | --- | --- |
| 9-1 | offer the two phrase rows with no phrase held | `a payload preimage record offers [preimage string phrase + method phrase + method + QR do not cut this preimage]` |
| 9-2 | drop the declined plate silently | `the census does not carry §8.3's row "preimage f20a0890..94165eb9: declined, will not be cut"`; and the flow test's `the census does not NAME the declined plate` |
| 9-3 | draw the census through `confirmReviewScreen` | `composerEngraveStep draws through confirmReviewScreen, which wraps at dims.X-2*8 and centres on the whole panel: every census row over 374 px has its right edge under a navigation button (W-3)` |
| 9-4 | add the accepted plates as `bundleCard`s | `the shipped plate count changed when a preimage plate was accepted` |
| 9-5 | move the preimage loop AFTER `bundleEngrave` | `the first engrave screen never drew` (the run reaches `Choose engraving` first) |
| 9-6 | fire §8.4b unconditionally | `an abort arm was drawn on a COMPLETED run` |
| 9-7 | restore §10.1's future tense | `composerCopyHashEveryPathHeld (SPEC §H6-10.1) does not match the spec`; and `§10.1's body promises a cut the operator may still decline: "this run cuts"` |
| 9-8 | suppress the mark on `cardMK1` | `bundlePlateMark(0) = "", want "PREIMAGE REQUIRED"` |
| 9-9 | an em dash in `composerCopyAbortNoPreimage` | `the modal drew only 5004 ink pixels (floor 6000) -- this frame is near blank`; and `carries the non-ASCII or control rune '—'` |
| 9-10 | attach §8.4a to `bundleAbortWarningText` | `§8.4a never drew` — the text is unreachable on that path, which is the plan's own claim, now measured |
| 9-11 | fire §8.4a unconditionally | `the preimage plate's engrave screen never drew` |

Mutation 9-9's ink count is **5,004 against a 6,000 floor — the number the plan
predicted, exactly.**

The plan's first Step-8 mutation ("put the form/QR/decline controls on
`confirmReviewScreen`") is **ARGUED, not executed**, and that is the plan's own
point: the screen is read-only, all three inputs are already bound, and its body
lines are `widget.Labelw` ops, so there is no control to bind. The runnable half
of that row (offer a QR row with no phrase held) IS 9-1.

**Task 10 (5 of 5 red)**

| # | mutation | failure |
| --- | --- | --- |
| 10-1 | derive per pick without the once guard | `the second pick took 11.678186ms: the KDF ran again` (against 111 ns unmutated) |
| 10-2 | omit the locator's `hash` row | `locator = [], want exactly ["hash  b867db87..edbc96cb"]` |
| 10-3 | reuse §8.4a as this flow's abort | `this flow's abort borrows §8.4's wording "dies with this composition", which is false here: the flow builds no composition and the material stays in the payload, in flash` |
| 10-4 | offer the route unconditionally | `composerDoorHasPreimage = true, want false`; `Hashlock plates offered = true, want false` |
| 10-5 | derive at LIST time | `the list took 31.36247ms against a 12.9785ms derivation`; every row reads `phrase 1  3cf5d421..b70a4c12` |

**Task 11 (4 of 4 red)**

| # | mutation | failure |
| --- | --- | --- |
| 11-1 | use `codex32.IsPreimage` instead of `IsMS1Shaped` | `§9's warning did not fire at the entry screen` in BOTH programs |
| 11-2 | refuse instead of warning | `accepting §9's warning did not advance the program`; `the entry step did not accept the string (done=false ok=false)` |
| 11-3 | drop the §8.8 notice | `the notice drew = false, want true` |
| 11-4 | draw the notice when a `pass:` record is present | `the notice drew = true, want false` |

**Task 12 (2 of 2 red)** — see §2, runs (b) and (c).

Mutation transcripts are in `/scratch/code/shibboleth/.tmp/h6g/mut-*.txt`.

---

## 4. Measured bodies and geometry

**Every carried number the spec supplied for these five tasks was CONFIRMED to
the character.** The spec's own measurements were right.

### Modal bodies (`assertModalBodyFits`, `sh2DisplaySize`)

| body | renderer | drawn | headroom | plan said |
| --- | --- | --- | --- | --- |
| §8.4a, no preimage plate was cut | `errorScreenBody` | 75 | 476 | 75 / 476 ✔ |
| §8.4b, a preimage plate was cut | `errorScreenBody` | 86 | 476 | 86 / 476 ✔ |
| §10.1, held form | `errorScreenBody` | 185 | 360 | 185 / 360 ✔ |
| §10.1, held-phrase form | `errorScreenBody` | 186 | 360 | 186 / 360 ✔ |
| §8.3, stand-alone unused-preimage notice | `errorScreenBody` | 107 | 455 | 107 / 455 ✔ |
| §8.8, Password-program notice | `errorScreenBody` | 165 | 397 | 165 / 397 ✔ |
| §9, ms1-shaped warning | `errorScreenBody` | 184 | 378 | 184 / 378 ✔ |
| §9, ms1-shaped warning | `confirmWarningBody` + `composerConfirmBody` | 204 | 302 | 204 / 302 ✔ |
| §8.5, QR warning | `confirmWarningBody` + `composerConfirmBody` | 126 | 378 | 126 / 378 ✔ |
| §5.1, payload preimage-record confirm (longest) | `confirmWarningBody` + `composerConfirmBody` | 274 | 186 | *new* |
| §5.3, preimage-plate refusal | `errorScreenBody` | 99 | 455 | *new* |
| §5.2, Hashlock plates abort | `errorScreenBody` | 80 | 476 | *new* |
| §5.2, empty-payload refusal | `errorScreenBody` | 46 | 513 | *new* |

Margin is 80 characters throughout; the tightest new body has 186.

### `Which hash?` rows, in `composerPageLines`' own 411 px band

| row | chars | px | lines |
| --- | --- | --- | --- |
| `hash 10  b867db87..edbc96cb` | 27 | 236 | 1 |
| `hash 10  b867db87..edbc96cb  (in payload)` | 41 | 343 | 1 |
| `preimage 10  b867db87..edbc96cb` | 31 | 276 | 1 |
| `phrase record 10 (derive to see the digest)` | 43 | 336 | 1 |
| `phrase 10  b867db87..edbc96cb` | 29 | 253 | 1 |
| `Type a hashlock phrase` | 22 | 189 | 1 |
| `Type 64 hex` | 11 | 95 | 1 |
| `No hash lock` | 12 | 100 | 1 |

The rejected wording `hash 10  b867db87..edbc96cb  (preimage in payload)` is
**50 characters, 348 px, TWO lines** — the plan's own figure, confirmed.

### §5.3 step (A)'s pick rows

| row | chars | px | plan said |
| --- | --- | --- | --- |
| `preimage string` | 15 | 128 | 128 ✔ |
| `phrase + method` | 15 | 138 | 138 ✔ |
| `phrase + method + QR` | 20 | 180 | 180 ✔ |
| `do not cut this preimage` | 24 | 196 | 196 ✔ |

### §8.3's census rows

Panel 480 px; nav column at **427 px**; `confirmReviewScreen` wraps at **464 px**;
a panel-centred row is under the column above **374 px**; `composerPageLines`'
band is **411 px**. The shipped completeness line measures **459 px** at the 464
wrap. All four of the plan's geometry figures confirmed.

At the 464 px wrap, on the gate's fixture: `Plus 1 preimage plate(s)…` 419 px,
`path 2  <digest>  phrase, sha256, QR` 385 px, `preimage <digest>: declined…`
434 px, `preimage <digest>: not on any path…` 442 px, `Keep each preimage plate
apart…` 441 px. All over 374; none draws under a button in the 411 px band.

### Firmware size (`nix develop -c tinygo build -size short … ./cmd/controller`)

| tree | flash | ram |
| --- | --- | --- |
| Tasks 4-8a (the author's number) | 1,600,944 | 63,248 |
| Tasks 4-12, wired | **1,643,580** | **63,272** |
| delta, 8b-12 | **+42,636 B** | **+24 B** |
| delta, whole stage vs pristine `fb0dd04` | **+44,372 B** | **+416 B** |

Two probes, both measured:

- **The plate layout is a small part of it.** Stubbing `backup.EngraveHashlock`
  out of `composerHashlockPlateFor` — production's only reference to Task 6's
  whole layout — measures **1,638,972 B**, so making it reachable costs 4,608 B.
  The corollary matters for the author's number too: Tasks 4-8a wired a plate
  layout, a constant-time QR encoder and an ms1 preimage encoder that NOTHING
  production called, so TinyGo dropped them and the +1,736 B was not the codec
  half's real cost either.
- **`sort.Slice` is not the expensive one.** Replacing it in
  `composerPreimagePlates` with a hand-rolled insertion sort measured
  **1,644,120 B** — **540 B WORSE**. The reflect machinery is already resident.
  The measurement is recorded at that line so it is not repeated.

---

## 5. The shard-18 diagnosis

**Reproduced, twice-over, and it is CORRECT — Task 8a's own finding, not a
regression and not F-490.**

At the author's Task-8a tree, `scripts/gui-shard-test.sh ./gui/ 24` reports
1242 tests, partition exhaustive, 23 of 24 shards ok, and shard 18:

```
--- FAIL: TestComposerEveryScreenFunctionHasAProductionCaller (0.03s)
    composer_join_test.go:97: exempt: composerDescriptorCeilingChars -- the same measurement, called by TestComposerMeasureSection13Numbers
    composer_join_test.go:104: these composer functions have no production caller, so the screens they
        draw cannot be reached by any operator: [composerHoldHashlockMaterial]
```

Findings:

1. **It is deterministic, not a flake.** It reproduced on the first run here and
   names one function with a stable message. `TestEngraveScreenReleasesResumeStateOnReturn`
   — F-490's known load flake — **passed in the same run**, so F-490 is not the
   cause and the two are unrelated.
2. **It is not a regression from Tasks 4-8a.** Tasks 4-7 touch `engrave/`,
   `codex32/`, `backup/` and `sysw/`, none of which the join guard reads; 8a
   inserted exactly one function, and that function is the one named.
3. **It CLOSED the moment Task 8b landed.** 8b gives
   `composerHoldHashlockMaterial` two production callers —
   `hashlockPayloadRoute`/`hashlockPreimageRecordRoute` and the HOLD in
   `hashlockPhraseRoute` — and the same 24-shard command over the 8b tree reports
   **1250 tests, exhaustive, all 24 shards ok**. The plan's ruling stands
   verbatim: 8a and 8b land in ONE commit, and the exemption table is not the
   instrument.

---

## 6. Fix table — every plan block or claim the gate changed

Seventeen. Each names the block, what failed, what changed and why.

| # | where | what was wrong | what changed |
| --- | --- | --- | --- |
| F1 | 8b Interfaces | `hashlockPayloadRoute(… rec sysw.Record)` — **there is no exported `sysw.Record`**; the block below it already said `sysw.PhraseRecord`. | Signature corrected to `(ctx, th, st, idx int, rec sysw.PhraseRecord, payload [][32]byte)`. The payload digests are added because `hashlockRelationLine` needs them for the confirm modal's `matches hash <i>` line, which the block's `...` elided. |
| F2 | 8b Step 3 block | The block was a SKETCH: `hashlockDeriveFlow(…, rec.Phrase, rec.Method)` passes a `string` and a `sysw.HashlockMethod` where the function takes `[]byte` and a `gui.hashlockMethod`; `composerCopyHashlockConfirm(ctx, th, st, idx, h, …)` is not that function's signature at all (it takes five strings/ints and RETURNS a body); `hashlock.Digest` needs a pointer. **It does not compile in any form.** | Replaced with the wired function, plus `hashlockPreimageRecordRoute` beside it (the prose already required it), plus `hashlockMethodOf` for the wire→screen method mapping. |
| F3 | 8b, unstated | A payload PREIMAGE record has no phrase and no method, so `composerCopyHashlockConfirm` — which the Interfaces name for both carriers — draws `method: hardened   chars: 0` on the screen that gates funds. | New body `composerCopyHashlockPreimageConfirm`, measured 274 / 186, with a `composerCopyTable` row so §12 item 5's four gates reach it. |
| F4 | 8b, unstated | Band 3's row form depends on what THIS composition has derived, and `composerHashRows(s *syswSession)` cannot see it. | `composerHashRows` gains `st *composerState` (nil-safe); `hashlockDerivedDigest` answers "has this record been derived here" out of Task 8a's `hashlockHeld` rather than a second, index-keyed map. |
| F5 | 8b Step 2 | *"When a `hash:` digest and a preimage **or `phrase:` record**… carry the SAME digest"* — an underived `phrase:` record's digest is not knowable without the KDF that Step 3 forbids at row-build time. | The annotation is exact for preimage records and becomes true for a `phrase:` record once derived (the row set is rebuilt every pass of `composerHashEdit`'s loop). Stated in the plan. |
| F6 | 8b Step 1 | *"23 px per row"* is the LABEL height, not the row PITCH (29 px), and the plan carried no page-capacity number. **Measured: the first page holds FIVE rows**, so a payload with a `hash:`, a preimage and a `phrase:` record puts `No hash lock` on page 2. | Measured, stated, and pinned by `TestWhichHashPageHoldsFiveRows` (which also asserts the row is reachable by paging). Filed as a follow-up. |
| F7 | 9 Interfaces | `composerAbortNoPreimage(ctx, th, st)` — `st` is unused by either arm; `composerBuildHashlockPlate(st, h, choice) backup.Hashlock` cannot build a plate (it needs the pre-formatted locator, and it can fail). | `composerAbortNoPreimage(ctx, th) bool`; `composerBuildHashlockPlate(p hashlockPlate, locator []string) (backup.Hashlock, error)`. |
| F8 | 9, unstated | **§8.6's method line and `ms_codec::hashlock::qr_text`'s Go twin existed only as CONSTANTS IN `backup/hashlock_test.go`.** No task wires them, so a plate built from production has nothing to put in `Method` or `QRText`. | `hashlock.MethodLine(hardened bool)` and `hashlock.QRText(hardened bool, phrase string)`, built from the package's own `Iterations`/`Salt`/`PreimageLen` so a constant change cannot drift the plate, and pinned against the Rust primary's literal. The hardened line is **73 characters** — §6.5's own pin. |
| F9 | 9 Step 3 block | The loop returns `composerAbortNoPreimage` unconditionally on a failed engrave, so §8.4b — *"at least one preimage plate cut, then the run ends"*, the window the ORDER creates — can never fire; and it never calls `bundleEngrave`'s result back into §8.4b either. | Replaced with the wired loop: a `cut` counter chooses the arm, and `!done && cut > 0` after `bundleEngrave` fires §8.4b. Both arms are now reachable and both are driven by a test. |
| F10 | 9 Step 2 | *"Drawn through `composerPageLines`' 411 px band"* — but `composerEngraveStep` called `confirmReviewScreen`, which wraps at 464 px and centres on the whole panel. | `composerEngraveStep` now calls `composerReadScreen` (identical contract, band-correct). Gated by an AST assertion, because no text or raster assertion on the live screen can tell the two apart. |
| F11 | 9 Step 2 | The five census-row widths (405, 423, 441, 447, 448 px) are **digest-dependent** and not reproducible as literals. | Replaced with the digest-independent property — every row is over the 374 px threshold, none draws under a button in the 411 px band — plus the fixture's measured values. The census also joins `composerPagedScreens`, so the shipped W-3 gate covers it. |
| F12 | 9 Step 6 | §10.1's fourth arm is specified as *"at least one of those came from a phrase typed here"*, but its body says the composition *"holds the phrase and method **for each one**"* — false on a mixed policy where one path's material is a payload preimage record with no phrase; and it excludes a payload-delivered phrase the device DOES hold, whose backup burden is identical. | Predicate is `composerEveryHeldPathHasAPhrase`: every hashed path's material carries a phrase. Both readings are tested (`TestComposerHashEveryPathArmsAreInThePresentTenseOfWhatIsHeld` has a mixed row and a partial row). |
| F13 | 9 Step 1, unstated | The masked lead is drawn as a per-page header THROUGH `composerPageLines`, so each line it spends is a row lost. A four-line lead leaves **three** of the four rows on page 1, with `do not cut this preimage` — the UNDO row — on page 2. | The lead is two lines; `TestComposerPreimagePlatePickDrawsAllFourRows` pins all four onto page 1. |
| F14 | 9/10, unstated | `composerPreimagePlates` ranging `hashlockHeld` would list the same composition's plates in a **different order on every frame** (Go randomises map iteration) — on the screen whose job is to be read against the bench. | Deterministic order: paths first in path order (deduplicated by digest), then unused digests in digest order. Pinned over 32 attempts. |
| F15 | 10 Files/Interfaces | The task names `gui/composer_door.go` and not `gui/wallet_policy.go`, its ONE caller — a fourth route nothing dispatches is the F-437 defect one level in. `composerDoorHasPreimage(ctx)` also disagrees with its sibling `composerDoorHasConsumablePolicy(s)`. | `wallet_policy.go` added to Files and wired (the route is a loop member, so Back lands on the door); the predicate takes the session. `hashlockPlateLocator` is wired in Task 9, where the composer-native plate needs the same rows. |
| F16 | 11 Files | **Three of the four test paths do not exist**: no `gui/freetext_test.go` (it is `freetext_flow_test.go`, one of nine), no `gui/passphrase_test.go` (it is `passphrase_flow_test.go`), and no `gui/sysw_session_test.go` at all. | Corrected; `gui/sysw_session_test.go` created. |
| F17 | 11 Step 2 block | The `syswOfferAlt` citation carries an inline `// <-- no screen is drawn` annotation that is not in the source, so it could never be a byte-exact fragment. | Annotation moved out of the block; the block is now the verbatim source and is checked. |

**Two deliberate non-changes**, recorded rather than fixed:

- **§8.3's `plate(s)`** is the spelling this package's own house rule refuses
  (`composerSlotWord` renders "slot @3" or "slots @3 and @4" precisely so a
  refusal never reads "slots @3"). Kept **verbatim**: changing spec copy inside
  a build gate puts the shipped string and the document it is diffed against out
  of step. Filed as a Task-13 follow-up.
- **Four bodies the device now draws have no §8 blockquote** (the payload
  preimage-record confirm, the masked pick lead, the plate refusal, and the
  Hashlock plates flow's three screens). They are quoted strings in
  `composerCopyTable`, which §11 admits — but the spec is no longer the source
  they are diffed against. Filed as a Task-13 follow-up.

---

## 7. What the gate could NOT reach

Stated because a gate that hides its blind spot is worse than no gate.

- **The plan's first Step-8 mutation is argued, not executed** — see §3. It is
  unimplementable by construction, which is the plan's claim; the gate did not
  manufacture a proxy for it.
- **`git diff` between walk runs** is unavailable in a `.git`-less scratch tree;
  sha256 was used instead (§2).
- **No hardware.** Every screen number is `sh2DisplaySize` in the harness or the
  wasm emulator; nothing was cut on steel and no plate was read back by eye.
- **`gofmt`, `go vet` and the three baseline reds are unchanged.** `gofmt -l`
  over `gui/ hashlock/ backup/ codex32/ sysw/ engrave/ cmd/emu/` still prints
  exactly `gui/transaction.go`, `gui/transaction_golden_test.go`,
  `gui/transaction_txrecord_test.go` and no fourth; `go vet ./gui/` exits 1 with
  the same two pre-existing `testing.ArtifactDir requires go1.26` diagnostics.
  H6 adds no new class of either.
- **`scripts/plan-glyph-check.sh`** reports the same 4 pre-existing undrawable
  operator strings (lines 1196, 2469, 2493, 3705 — all Task 1-8a text or the
  author's elided `-run` expression); the gate introduced none.
  `scripts/plan-table-check.sh` reports the same 1 malformed row, the author's
  8a boundary-gate row, whose pipes sit inside a code span (the checker's own
  named blind spot).

---

## 8. Checker output

`./scripts/h6-plan-blocks-vs-tree.sh` from the engrave checkout, over the three
gate trees:

```
86 blocks checked, 0 FAIL

NOT COVERED by this script:
  * 6 fenced blocks carry no file= header (bash recipes, illustrative
    snippets); nothing here runs or checks them:
      …:686  …:1269  …:1492  …:1648  …:2036  …:2479
```

The six are five bash/recipe blocks in Tasks 1-4 and one captured `go test`
failure tail in Task 8a — **none is in Tasks 8b-12**. The plan's `## Build gate`
claimed "65 blocks checked"; the author's own script reported 70 at the time, so
that figure was already stale before this round. It now reads 86.

The script's header, which named the 8b-12 gap as its one H6-specific blind
spot, is updated to record that the gap is closed.

---

## 9. Verdict

**GATE GREEN WITH FIXES (17).**

- Tasks 8b, 9, 10, 11 and 12 are wired, built, tested and mutated in the author's
  own fork scratch tree, left in place at
  `/scratch/code/shibboleth/.tmp/h6-gate`.
- 40 mutations executed; **40 red**, each with its failure quoted above.
- Whole `gui`: **1285 top-level tests, partition verified exhaustive, all 24
  shards ok.** Fork sweep: `engrave backup codex32 sysw hashlock cmd/emu` all ok.
- The emulator walk **RAN** four times with the two required mutations red and
  the file byte-identical throughout.
- Every code block in Tasks 8b-12 carries a `file=`/`mode=` header and is
  byte-for-byte the compiled text; **86 blocks checked, 0 FAIL**.
- Firmware measured: **1,643,580 B flash / 63,272 B RAM**, +42,636 B over the
  author's tree.
- Shard 18 diagnosed: correct, deterministic, Task 8a's own finding, closed by
  Task 8b.

Nothing remains RED. The seventeen fixes are folded into the plan; four items
are filed as follow-ups (the two copy items above, the five-row page capacity,
and the firmware delta).
