# Hashlock H2 — device leg, implementation report

**Plan:** `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` (engrave master `1cb05b8`, STATUS R0 GREEN).
**Spec:** `design/SPEC_hashlock_H2_device.md` (GREEN `55ee7a4`).
**Branch tips:** seedhammer fork `hashlock-h2` = **`e1bf137`** (5 commits off `main` `c4a64fc`); mnemonic-engrave `hashlock-h2` = **`f67b94b`** (2 commits, branched from `master` at `279d731`). Engrave `master` moved to `047dafc` while this ran, adding only `design/agent-reports/push-engrave-279d731.md`, which this branch does not touch — the merge is clean.
**Nothing pushed. No `master`/`main` commit. The merge, the flash, the post-implementation review and H4 are NOT mine.**

Every count below is from a run captured once to a file under `/scratch/code/shibboleth/.tmp/h2-impl/` at the tip reported here, and quoted from that file.

---

## Summary

All six tasks executed test-first, each RED reproduced before the code that turns it green.
**32 mutation runs**, each mutated / observed / reverted, covering every mutation the plan names. The plan's tables hold 29 rows (10 in Task 1, 2 in Task 2, 17 in Task 4 — one of which repeats Task 2's), and three of those rows name two mutations each, so 32 is the row count expanded. Every one failed on the test the plan names; where the plan's own *description* of a failure differed from what was measured, the deviation is recorded below rather than smoothed over.

The strongest single check is not a count. `scripts/h2-plan-blocks-vs-tree.sh` was run against **my worktree** (not the gated tree it defaults to) and reports **26 blocks checked, 0 FAIL** — and independently, all 16 files this stage touches are byte-identical to the gated tree at `/scratch/code/shibboleth/.tmp/h2-gate`, reached by following the plan's steps rather than by copying.

**One thing the post-implementation review should look at first, because I found it and could not close it:** the phrase screen draws **no readout at all** — masked or revealed — and its `show` key is drawn, tappable, flips to `hide`, and reveals nothing (F-481, measured on the emulator). I did not argue it as gating; it is an affordance defect on a screen this stage created, and its severity is a reviewer's call, not mine.

---

## Task 1 — the `hashlock` package and its lockstep gate

**Fork commit `f8f0bc2`** — `hashlock/hashlock.go`, `hashlock/hashlock_test.go`, `hashlock/testdata/hashlock-v0.8.json`, `hashlock/testdata/hashlock-v0.8.provenance.json`.

**Step 1, the vendored corpus.** `sha256sum` of the copied file:

    a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30  hashlock/testdata/hashlock-v0.8.json

which is the plan's pin exactly. See deviation D1 on the source revision. Shape as parsed: `{'kind': 1, 'derivation': 11, 'refusals': 15, 'lengths_by_door': 7, 'lockstep': 4}`.

**Step 3, RED (verbatim):**

    # seedhammer.com/hashlock [seedhammer.com/hashlock.test]
    hashlock/hashlock_test.go:84:8: undefined: PreimageHardened
    hashlock/hashlock_test.go:88:17: undefined: Digest
    hashlock/hashlock_test.go:91:9: undefined: PreimageSHA256
    ...
    FAIL	seedhammer.com/hashlock [build failed]

The plan's Expected line — *"does not compile (`PreimageHardened` … undefined)"* — reproduced.

**Step 5, GREEN: 9 tests, all PASS** (`go vet ./hashlock/` clean):
`TestDerivationRowsLockstep`, `TestCorpusCarriesTheNonFixedPointRows`, `TestRefusalRowsMatchTheHost`, `TestKindRowPreimageDigest`, `TestPhraseMaxCharsIsTheCap`, `TestDeriveHardenedAbandonsWhenProgressSaysStop`, `TestIsMS1ShapedMinLengthBoundary`, `TestIsMS1ShapedTrimsWhatTheStripLoopCannot`, `TestLockstepListIsTheOneWeDrive` — `ok seedhammer.com/hashlock 0.277s`.

**The ten mutations, re-run.**

| Mutation | Measured | Plan said |
| --- | --- | --- |
| `Salt` padded to 16 bytes | **22** assertion failures, `TestDerivationRowsLockstep`; first line `"correct horse battery staple" hardened X: got 81b38099… want c3e97525…` | 22 ✓ |
| `Iterations = 99999` | **22**, same rows | 22 ✓ |
| `seal.NormalisePassphrase` at the top of `PreimageHardened` | **6**, on exactly the two named rows `"  a  b "` and `"Correct Horse Battery Staple"` | "4 failures (X+H each)" — see D6 |
| strip `-`/`,` from the phrase first | **12**, on exactly **4** rows: `correct-horse,battery staple`, `a-b,c`, and both 64-char rows | "FOUR rows fail, not one" ✓ |
| `IsMS1Shaped` via `codex32.New` | refusals **rows 11, 12 and 13** fail, exactly as named (`row 11 rule ms1-shaped: got <nil> want …not a phrase`) — see D8 | rows 11-13 ✓ |
| the cap literal 99 in `ValidatePhrase` | **only** `TestPhraseMaxCharsIsTheCap`: `100 characters must be accepted: hashlock: the phrase is longer than 100 characters` | "ONLY TestPhraseMaxCharsIsTheCap" ✓ |
| `Digest` double-hashes | `kind[0] digest: got 88b8f02ce56abce1d453e0610318130f4d0a13067549e804af1f5186f81a2691 want 9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885` | `88b8f02c… / 9a2db2e2…` ✓ (both prefixes match) |
| `DeriveHardened` ignores `progress`'s return | `returned ok=true after progress abandoned it`; `progress was called 199 times`; `an abandoned derivation returned a non-zero value` | ok=true, 199 ✓ |
| `minMS1Len = 47` | `47 characters must be BELOW the ms1 shape bound` + the grouped row | ✓ |
| `minMS1Len = 49` | `48 characters is the bound and must be ms1-shaped` + the grouped row | ✓ |
| drop `strings.TrimSpace` from `IsMS1Shaped` | **10** failures — `\v`, `\f`, U+0085, U+00A0, U+2003, leading and trailing | 10 ✓ |

Restoration verified after every mutation: the committed `hashlock.go` `diff`s clean against the plan's block, and the suite is green at the tip.

---

## Task 2 — `codex32.DecodeMS1Preimage`

**Fork commit `fa4b701`** — `codex32/mspayload.go`, `codex32/mspayload_test.go`.

**Step 2, RED (verbatim):** `codex32/mspayload_test.go:180:12: undefined: DecodeMS1Preimage` (×5 sites), `FAIL seedhammer.com/codex32 [build failed]`. Plan's Expected — *"does not compile"* — reproduced.

**Step 4, GREEN:** `--- PASS: TestDecodeMS1PreimageIsShapeExact (0.00s)`, `ok seedhammer.com/codex32 0.029s`.

**Both mutations.**

| Mutation | Measured |
| --- | --- |
| `copy(preimage[:], d[:32])` | `preimage = 03ababababababababababababababababababababababababababababababab, want the corpus's preimage_hex abababab…ababab` — the plan's line, verbatim |
| drop the `!f.Unshared` clause | `DecodeMS1Preimage(a 2-of-N share beginning 0x03) err = <nil>, want codex32: not an m-format secret payload` — the plan's line. See D7: the naive edit does not compile |

---

## Task 3 — `Which hash?`, label-keyed

**Fork commit `f283e3a`** — `gui/composer_hash.go`, `gui/composer_hash_test.go`, `gui/composer_copy.go`, `gui/composer_copy_test.go`, `gui/composer_gates_test.go`, `gui/composer_hashlock.go` (the stub), **and `gui/composer_state.go`** (deviation D3).

**Step 1, RED (verbatim):**

    gui/composer_hash_test.go:89:11: undefined: composerHashRows
    gui/composer_hash_test.go:93:37: undefined: composerHashRowPhrase

**Step 5, GREEN:** `go test -run 'TestWhichHashRowsAreLabelKeyed|TestComposerHash|TestComposerCopy|TestComposerLockAndHashEdits' ./gui/` → **9 top-level PASS**, `ok seedhammer.com/gui 0.412s`. Build-gate fix 12 confirmed live: `TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm/hash_lock` PASSes on the moved pump target `Path 1 hash`.

**Whole package at this task:** `1206 top-level tests`, `partition verified exhaustive: 1206 == 1206`, all 24 shards ok, wall 50s. (Baseline `c4a64fc` is therefore **1205**, and this task's one new test makes 1206 — consistent with the 1220 reached in Task 4; see the note under Task 4.)

---

## Task 4 — the phrase route

**Fork commit `978a9de`** — `gui/composer_hashlock.go` (replacing the stub), `gui/composer_hashlock_test.go`, `gui/composer_copy.go`, `gui/composer_copy_test.go`, `gui/modal_fits_test.go`, `gui/composer_state.go`, `gui/composer_shape.go`, `gui/composer_hash.go`.

**Step 1, RED:** `gui/composer_copy_test.go:144:46: undefined: composerCopyHashEveryPathFor` — the copy gate's row for a body Step 3 creates.

**Step 2, RED (verbatim):**

    gui/composer_copy_test.go:144:46: undefined: composerCopyHashEveryPathFor
    gui/composer_hashlock_test.go:613:12: undefined: hashlockRelationLine
    gui/composer_hashlock_test.go:773:13: undefined: hashlockDerivingLead
    gui/composer_hashlock_test.go:821:14: undefined: hashlockDeriveFlow
    gui/composer_hashlock_test.go:821:86: undefined: hashlockHardened
    gui/composer_hashlock_test.go:916:12: undefined: hashlockOtherPathLine   (×4)
    FAIL	seedhammer.com/gui [build failed]

This is deviation **D5**: the plan's Expected line says the package COMPILES and fails at runtime in `tapPassphraseKey`. It does not — six symbols the R0 round 0 fold added to the test file are declared only in Step 3. Still RED, at the same checkpoint; only the plan's account of *how* is stale.

**Step 4, GREEN:** `go vet ./gui/` clean but for the two pre-existing `testing.ArtifactDir` complaints the plan names; `go test -run 'TestHashlock|TestWhichHash|TestComposerHash|TestComposerCopy|TestModals|TestConfirmScreens' ./gui/` → **23 top-level PASS**, `ok seedhammer.com/gui 37.026s`. All **15** new top-level `gui` tests pass, including `TestHashlockDeriveKeepsAwakeUnderTheScreensaver` (0.16s under `synctest`) and `TestConfirmScreensThisBlockTouchesAreDrawnInFull`.

**Step 5, whole package:** `1220 top-level tests`, `partition verified exhaustive: 1220 == 1220`, all 24 shards ok. **This is the plan's own number.** One reconciliation note: the plan calls the new `gui` tests "14" and then enumerates **15** of them; 1205 + 15 = 1220, so the enumeration is right and the word is a typo. `hashlock`'s 9 and `codex32`'s `TestDecodeMS1PreimageIsShapeExact` are separate packages and outside the 1220, as the plan says.

**Task 4's mutation table, re-run in full** — the plan's 17 rows, expanded to the 19 runs they name (row 6 is Task 2's, already run there; rows 17 and 18 each hold two). Each mutated, measured, reverted.

| # | Mutation | Measured failure |
| --- | --- | --- |
| 1 | `seal.NormalisePassphrase` in `hashlockPhraseFlow` | `TestHashlockPhraseRouteDoesNotNormalise`: `"Correct Horse Battery Staple": path hash = …, want 95d4447031cdc4117f…` |
| 2 | the confirm's Back returns `hashlockBackToWhichHash` | `TestHashlockBackContractKeepsThePath`: `never reached "Which method?"` |
| 3 | `composerHashEdit` returns `false` from the phrase route's Back | same test, `never reached "Type a hashlock phrase"` — the plan's *corrected* location, not the path-count assertion |
| 4 | pass `""` for the relation line | `TestHashlockConfirmRelationLine`, **both** cases: `never reached "matches hash 2 in the payload"` and `never reached "no hash: record in the payload has this digest"` |
| 5 | delete the release from `holdConfirm` | `TestHashlockPhraseRouteSetsTheCorpusDigest/sha256_anchor`: `path hash = <nil>, want b867db87…`. See D9 — it fails rather than hanging |
| 6 | drop `!f.Unshared` (Task 2) | as in Task 2 |
| 7 | delete `ctx.KeepAwake()` | `Run exceeded 100000 ticks without terminating -- flow is probably parked (screensaver?). 180 frames drawn, last = "89%About21secondsleft.Deriving"` — the r0 Critical, verbatim |
| 8 | delete `ctx.WakeupAt(time.Now())`, keep `KeepAwake` | `the derivation took 9h57m1s of device time; at a 1s tick floor and 200 frames it should take about 3m20s` — verbatim |
| 9 | delete the hoisted zero-state frame | `only 199 frames drawn; 100,000 iterations in 500-step slices is 201` |
| 10 | `hashlockDerivingLead` returns the estimate unconditionally | `hashlockDerivingLead(0, 100000, 0s) = "About -9223372036 seconds left."` |
| 11 | `return false` for `continue` in the hex arm | `TestHashlockHexRowBackKeepsThePath`: `never reached "Type a hashlock phrase"; last frame "0123456789ABCDEF0of64hexHashlock"` — verbatim |
| 12 | the surgical index-arithmetic reversion | `TestComposerHashEditDispatchesByRowLabel/hex_row_opens_hex_entry_and_does_not_clear`: `never reached "0 of 64 hex"`. **The plan's claim that `TestWhichHashRowsAreLabelKeyed` and `TestHashlockPhraseRouteSetsTheCorpusDigest` stay GREEN under it was re-verified**: all three ran in one selection and only the dispatch subtest failed |
| 13 | swap the phrase/hex appends | `TestWhichHashRowsAreLabelKeyed`: `n=0: indices 1/0/2` |
| 14 | delete the `composerHashByPhraseSync` call | `st.hashByPhrase survived the last hash being cleared` |
| 15 | delete `st.hashByPhrase = true` | `the phrase route did not record that this hash was set by phrase` |
| 16 | delete the post-HOLD reconciliation `showError` | `never reached "run ms hashlock with this phrase"` |
| 17a | `match := 0` | the no-match case: `never reached "no hash: record in the payload has this digest"`, frame carries `matcheshash1inthepayload` |
| 17b | `%d` on `i` rather than `i+1` | the second-record case: `never reached "matches hash 2 in the payload"`, frame carries `matcheshash1inthepayload` |
| 18a | `hashlockOtherPathLine` returns `""` | `never reached "two phrases to back up"`, and `a DIFFERENT hash on another path drew "", want the warning` |
| 18b | drop its `*p.Hash != h` comparison | `an EQUAL hash on another path drew "another path has a different hash: two phrases to back up", want silence` |

---

## Task 5 — the emulator walk and the firmware size

**Fork commit `e1bf137`** — `cmd/emu/walk_hashlock_phrase.js` (297 lines).

### Step 1: the walk, written and RUN

Built `emu.wasm` from `hashlock-h2` (`./cmd/emu/build.sh`, 10,853,903 bytes), served `cmd/emu` on a **fresh port 8791**, driven through playwright. Confirmed the build is the branch's before asserting anything: the `Which hash?` frame carries this stage's own no-payload lead, `"No hash record in the payload. Type a phrase below, or make one with ms hashlock on the host."`

**Result, verbatim, `run()` in 49.4 s — `ok: true`:**

    {
      "typed":    "hashb867db87..edbc96cbmethod:sha256chars:28Writedownthisphraseandthemethodnow.Theyarenotonthisdeviceandnotonyourplates.Withoutboth,thispathcanneverbespent.Onephraseperpolicy.Neverusethisphraseasapassphraseorapasswordanyw",
      "control":  "hashc8043156..253e7389method:sha256chars:27Writedownthisphraseandthemethodnow.Theyarenotonthisdeviceandnotonyourplates.Withoutboth,thispathcanneverbespent.Onephraseperpolicy.Neverusethisphraseasapassphraseorapasswordanyw",
      "mixed":    "hash95d44470..2297a7ffmethod:sha256chars:28Writedownthisphraseandthemethodnow.Theyarenotonthisdeviceandnotonyourplates.Withoutboth,thispathcanneverbespent.Onephraseperpolicy.Neverusethisphraseasapassphraseorapasswordanyw",
      "hardened": "hash3cf5d421..b70a4c12method:hardenedchars:28Writedownthisphraseandthemethodnow.Theyarenotonthisdeviceandnotonyourplates.Withoutboth,thispathcanneverbespent.Onephraseperpolicy.Neverusethisphraseasapassphraseorapasswordan",
      "ok": true,
      "hardenedFirstFrame": "Write down this phrase",
      "reconcile": "Beforeyoufundthiswallet,runmshashlockwiththisphraseandmethodonthehostandcheckthedigestmatches.Hashlock",
      "pathRow": "Spendpathsslots:0Path1:hashonlyAddaspendpathChangethescriptDone"
    }

Read against the corpus, which is the only oracle the walk uses:

- **typed** — `correct horse battery staple`, SHA-256 → `b867db87..edbc96cb` = `derivation[0].sha256_h`. ✓
- **control** — `correct horse battery stapl` (one character short) → `c8043156..253e7389`, `chars: 27`. Not the anchor's digest, so **the walk can fail**. Independently confirmed off-device: `sha256(sha256(b"correct horse battery stapl"))` = `c8043156…253e7389`, so the control is a real derivation of what was actually typed, not a blank.
- **mixed** — `Correct Horse Battery Staple` → `95d44470..2297a7ff` = that row's `sha256_h`, and **not** the lowercase row's. Nothing on the screen path folds case, trims or normalises (spec §2). ✓
- **hardened** — `3cf5d421..b70a4c12` = `derivation[0].hardened_h`, after the countdown; then HOLD assigned it, §4.5's reconciliation screen was reached and dismissed, and the path list came back reading **`Path 1: hash only`**. ✓

The keyboard mapping is not asserted anywhere: it is *proved* by the digests. One mistyped character changes SHA-256 completely, so trial 1 landing on the corpus constant is a 28-press proof that every tap hit the intended key. Getting there took two probing rounds — see D11, and F-481 for what the second round found.

### Step 2: firmware size

Recipe as the plan gives it, `nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`.

**Branch `e1bf137`:**

       code    data     bss |   flash     ram
    1564424   31852   31004 | 1596276   62856

**Baseline `c4a64fc`, measured myself in a detached worktree rather than quoted from the plan:**

       code    data     bss |   flash     ram
    1551336   31796   31004 | 1583132   62800

which reproduces the plan's cited baseline exactly.

**Delta: +13,144 B flash (+0.830%), +56 B RAM (+0.089%).** The plan's own expectation is *"a small delta over `c4a64fc`'s 1,583,132 / 62,800"*, with no numeric ceiling asserted anywhere, and it predicts the number will land above the build gate's pre-fold `1,595,236 / 62,856` because the R0 fold added 78 non-comment lines of production Go. It does, by **+1,040 B flash and +0 B RAM** — the fold's cost, and the RAM figure is unmoved by it. PBKDF2 and SHA-256 were already linked and the keyboard already existed, so nothing here suggests a package was pulled in that should not have been.

---

## Task 6 — records

**Engrave commit `ae288e7`** — `design/FOLLOWUPS.md`, five entries continuing the sequence from F-476:

- **F-477** — the composer spec's §6c line 386 (*"The composer never derives, stores or engraves a preimage this cycle"*) and its §14 out-of-scope row, both falsified by H2, each with replacement wording. Owning phase **H3**. The fork's own header comment is already corrected by Task 3, so code and spec disagree until this is folded.
- **F-478** — §4.5's drop-order destination (the reconciliation line into the phrase-route §8h, unreachable behind `composerEveryPathHashed`), with the plan's exact replacement sentence. Owning phase **H3**.
- **F-479** — §4.5's line list gaining the other-path line, with the exact replacement text and the measured headroom (337 drawn / 107 spare). Owning phase **H3**.
- **F-480** — per-path hash provenance in place of `composerState.hashByPhrase`, declined for H2 with its reason. Owning phase **H3**.
- **F-481** — NEW, and mine: the phrase screen's inert `show` key. Owning phase **H3**. Detail below.

**The fork has no CHANGELOG** (`ls CHANGELOG*` → no such file), as the plan states; the merge commit message is the record, and the merge is the controller's.

**F-475 (the M-6 seam-corpus prose correction) stays filed**, per the plan's own condition: H2 vendors `hashlock-v0.8.json`, a different file, and never touches `codex32_seam_vectors.json`.

---

## F-481 — the finding the walk produced, for the reviewer's attention

On the `Hashlock phrase` screen, measured on the emulator:

- with 20 characters typed and reveal OFF, the frame carries **no `*` at all**;
- tapping `show` flips the label to `hide` and the frame still carries **no characters**.

Mechanism: `hashlockPhraseFlow` cuts a lead band and a counter band out of the content rectangle and hands what is left to `kbd.MaxHeight`; `PassphraseKeyboard.Layout` then binary-searches leading runes off the readout until it fits `MaxHeight - grid - 8`. On this screen that budget is under one line, so every rune is dropped and `shown` is `""`.

This is why the walk's key mapping had to be probed with the `n/100` counter as its only oracle and proved by the digest.

I am not arguing it as gating, and I have not treated it as one: nothing is mis-derived, the counter reports the true length, and the confirm modal shows the digest, the method and the character count before anything is assigned. But the fork's own `passphrase_keyboard.go` makes the case against exactly this shape when it explains why the gear key was **removed** rather than left inert — *"a live-looking control that swallows the press, on the machine where the next thing the operator approves is cut into steel"*. `show` is now that control on this screen. The severity call belongs to the post-implementation review.

---

## Two follow-ups OWNED BY THIS PHASE that the plan does not schedule

The burndown rule is per-phase by ownership, and reconciliation happens on entering a phase. Two open entries name **H2** as their owning phase, and the H2 plan schedules neither:

- **F-474** — `unlock-kdf-names-the-refused-record`: `ErrRecordNotPermitted` renders as "Payload unreadable." with no record index or class (owning phase: **H2**).
- **F-475** — `seam-corpus-33-byte-collision-row-names-the-wrong-0.8-error` (owning phase: **H2**) — though the H2 plan explicitly re-schedules this one to H3 on the ground that H2 does not touch `codex32_seam_vectors.json`, which held.

I did not act on either: neither is in the plan I was given, and re-scheduling a phase-owned item is a controller decision, not an implementer's. Flagging both so the reconciliation is not silently skipped at the gate.

---

## Deviations from the plan

Every one of these is recorded because the plan said something and I did something else, or because a measurement differed from the plan's prediction. None changes what shipped.

**D1 — the corpus was copied from the ms worktree at HEAD `504ff46`, not `cd0a60f`.** The plan and brief name `cd0a60f`. The bytes are identical and that was verified rather than assumed: `git show cd0a60f:crates/ms-codec/tests/vectors/hashlock-v0.8.json | sha256sum` = `a46c197a…11d30`, the same as the vendored file and the same as the plan's pin. The provenance pin's `"commit": "cd0a60f"` is therefore accurate for this file, and no edit was needed.

**D2 — trailer order: `git commit -s` was dropped in favour of writing all three trailers into the message.** `-s` appends `Signed-off-by` **after** the Claude trailers, while every commit on fork `main` carries `Signed-off-by` first, then `Co-Authored-By`, then `Claude-Session`. Adding it to the message and *keeping* `-s` produced a duplicate sign-off. Task 1's commit was amended twice getting this right (`8411a9c` → `ab4ed4f` → `f8f0bc2`); the four later commits were correct on the first try. Every commit carries a DCO sign-off by Brian Goss, authored and committed as Brian Goss.

**D3 — the `composerState.hashByPhrase` field moved from Task 4 Step 3 into Task 3 (a real task-ordering gap in the plan).** Task 3 Step 2's block puts `composerHashByPhraseSync` in `gui/composer_hash.go`, and that function assigns `st.hashByPhrase`; the plan adds the field to `composerState` only in Task 4 Step 3. Task 3 therefore **cannot compile as ordered** — measured: `gui/composer_hash.go:198:5: st.hashByPhrase undefined (type *composerState has no field or method hashByPhrase)`. I applied the plan's own field block (plan line 2809) verbatim in Task 3 instead; only the task it lands in moved, and the reason is in `f283e3a`'s commit message. The build gate could not have caught this: it wired all four tasks into one tree at once.

**D4 — Task 3 Step 5's stub dropped its first line.** The block is unheadered by design and opens with `// gui/composer_hashlock.go, Task 3's transient content`, which is the locator standing in for the `file=` header the other blocks carry. I wrote the file from `package gui` onward. The file is replaced wholesale in Task 4 Step 3, and `go build ./gui/` at Task 3 exits 0.

**D5 — Task 4 Step 2's Expected line does not reproduce** (recorded in full under Task 4 above): the plan says the package compiles and fails at runtime; it fails to compile on six symbols Step 3 declares. Both are RED at the same checkpoint. Worth folding into the plan, because the Expected line was itself written by an R0 fold (r0 tests I-1) to correct an *earlier* wrong prediction, and it is wrong in the other direction now.

**D6 — Task 1's `NormalisePassphrase` mutation fails 6 assertions, not 4.** The scope claim is exactly right — only `"  a  b "` and `"Correct Horse Battery Staple"` fail — but each contributes three lines, not two: hardened X, hardened H, **and** `DeriveHardened != PreimageHardened`, because the one-shot function is mutated and the stepwise driver is not. The plan's "4 failures (X+H each)" predates that third assertion.

**D7 — Task 2's "drop the `!f.Unshared` clause" is not compilable as literally stated.** Removing the clause leaves `f` declared and unused: `codex32/mspayload.go:114:2: declared and not used: f`. I ran it as `_, perr := ParsePrefix(...)`, which is the same behavioural mutation, and it produced the plan's exact failure line.

**D8 — Task 1's `codex32.New` mutation has no code in the plan, so I wrote one.** Form used: parse with `codex32.New` and require an `ms1` prefix on the parsed string. Refusals **rows 11, 12 and 13 fail exactly as the plan names**; my form additionally fails `TestIsMS1ShapedMinLengthBoundary` and `TestIsMS1ShapedTrimsWhatTheStripLoopCannot`, whose synthetic non-checksummed inputs a real parse also rejects. Those two tests arrived in the R0 round 0 fold, after the gate measured this row, so the plan's scope line is right for the tests it was written against.

**D9 — Task 4's `holdConfirm`-release mutation fails rather than hangs.** The plan says *"every test with two or more holds hangs at its second one"*. Measured: `TestHashlockPhraseRouteSetsTheCorpusDigest/sha256_anchor` **fails** with `path hash = <nil>, want b867db87…` — the harness's own frame-pump bound turns the stuck hold into a bounded failure. Same mechanism, caught either way.

**D10 — the walk embeds the mixed-case digest as a constant instead of reading it from the corpus.** The plan says *"read it from the corpus"*. The corpus lives at `hashlock/testdata/`, outside the served `cmd/emu` directory, and `python3 -m http.server` refuses `..` traversal, so a page-side `fetch` cannot reach it. Every expected value in the walk is a constant copied from the corpus with the field it came from named beside it, and the file says in its header that nothing in it recomputes a digest.

**D11 — the walk's key grid was probed by TAPPING, not by reading `shScreen`.** The plan says to map the coordinates *"by probing `shScreen` for the keyboard's page, as `walk_verify.js` did for the ms1 keypad"*. Two things block that route on this screen, and both are recorded in the walk's own header:
  - `window.shTargets()` hit-tests only the **centre column**, and on the 10-key `qwertyuiop` row x=240 falls in the 8 px dead gap between two keys — so that row is absent from its output entirely;
  - there is no readout to read a character back from (F-481).

  So I swept x in 4 px steps across each row with the `n/100` counter as the oracle. The bands come out uniform: **34 px pitch, 26 px live, every row centred on x=239**, i.e. key *j* of an *n*-key row at `239 - 17(n-1) + 34j`. My first attempt used a wrongly-derived 28 px pitch and mistyped 8 of 28 characters — caught immediately, because the walk asserts the corpus digest.

**D12 — the walk's `Deriving` post-condition is a race, not an assertion.** Requiring the countdown screen made the first full run fail after passing all three SHA-256 trials: hardened PBKDF2 in wasm finishes before the next poll, so the confirm modal was already up. The walk now records which of the two frames it saw first (`hardenedFirstFrame`) and asserts only on the modal. Deliberate: demanding "Deriving" would make the walk fail on a device that is merely fast — a timing assertion dressed as a behavioural one — and `TestHashlockDeriveKeepsAwakeUnderTheScreensaver` is what gates that screen, in CI, on a clock the test controls.

---

## Machine checks beyond the plan's own list

- **`scripts/h2-plan-blocks-vs-tree.sh <plan> <my worktree>` → `26 blocks checked, 0 FAIL`.** Run against my tree rather than the gated one. It caught a real transcription slip on the way: Task 4's edit to `composer_copy_test.go` had replaced Task 3's `// 42 SINCE H2 TASK 3 …` comment instead of stacking the `// 53 SINCE H2 TASK 4 …` one under it. Restored, re-run clean.
- **All 16 touched files `diff` byte-identical to the gated tree** at `/scratch/code/shibboleth/.tmp/h2-gate` — reached by following the plan's steps, seeing each RED, not by copying.
- **`gofmt -l` offenders verified pre-existing** by checking out `c4a64fc` and re-running: `gui/transaction.go`, `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go` — the same five, before and after.

---

## Final gate runs at `e1bf137` — verbatim tails

`go test -count=1 ./hashlock/... ./codex32/...`

    ok  	seedhammer.com/hashlock	0.232s
    ok  	seedhammer.com/codex32	0.003s

`scripts/gui-shard-test.sh ./gui/ 24`

        1220 top-level tests
        partition verified exhaustive: 1220 == 1220
      ...
      shard 23: ok    50 tests  ok  	seedhammer.com/gui	24.902s
    === wall: 33s ===
    RESULT: ok -- all 1220 tests ran across 24 shards

`go vet ./hashlock/... ./codex32/... ./gui/... ./cmd/emu/`

    gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file is go1.25)
    gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
    gui/transaction_golden_test.go:104:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)

All three are the pre-existing Go-version complaints; the plan names the latter two for `./gui/`, and `gui/op/draw_test.go` is the same complaint in a subpackage that `./gui/...` reaches and `./gui/` does not. Nothing else. `git status` is clean at the tip, so no mutation survives in the tree.

`gofmt -l .` (minus `third_party`)

    gui/transaction.go
    gui/transaction_golden_test.go
    gui/transaction_txrecord_test.go
    mt/mt.go
    mt/mt_test.go

---

## Not done, by design

The merge to fork `main`, any push, the flash, H4 acceptance with the operator, and the post-implementation review are the controller's and the operator's. The controller re-runs the walk independently; the emulator's HTTP server on port 8791 has been stopped and the baseline build worktree removed.

Secret handling: no phrase and no preimage was written to any log, file, or commit kept by this work. The phrases that appear here are the corpus's own public test vectors, already committed in `hashlock/hashlock_test.go` and `hashlock/testdata/hashlock-v0.8.json`.

---

## Task 7 — F-474 burned down in-phase, F-475 reconciled

Added by the controller after this report's first version flagged that two open follow-ups named **H2** as their owning phase while the H2 plan scheduled neither. Both are now settled, and **no follow-up in the file names H2 any more** — the phase reconciles clean.

**Fork commit `17b3979`** — `seal/record.go`, `seal/record_not_permitted_test.go` (new), `seal/record_test.go`, `gui/unlock_kdf.go`, `gui/unlock_preimage_test.go` (new).
**Engrave commit `141788d`** — `design/FOLLOWUPS.md`.

### What was wrong

`seal.ErrRecordNotPermitted` already named the record index and the classification **in its message**, and that was never reachable: `gui/unlock_kdf.go` matches with `errors.Is` and cannot take a message apart, so every allow-list refusal fell through to the `default:` arm — **"Payload unreadable."** — after a *successful* authentication and a ~31 s derivation, on a payload that is intact. §2.2 item 4 has taught the operator to read "unreadable" as *someone replaced my payload*, so the screen sent them chasing a compromise that did not happen. `ErrTooManyRecords` and `ErrCodex32TooLong` already have named arms for exactly this reason, and §6.4 requires the machine to distinguish it.

### The change

`seal` gained `RecordNotPermittedError{Index, Class, Section, Preimage}`, returned by `AdmitSection`'s allow-list arm and `Unwrap`ing to `ErrRecordNotPermitted` so every existing `errors.Is` call site is untouched — the type is **additive**.

Two decisions worth naming, because both could have gone the other way:

- **It carries no record bytes.** Index, class and section are authenticated plaintext, so naming them leaks nothing — the argument §6.4 already won for the record count — while the record itself may be seed material. Asserted, not just intended: the seal test checks the rendered message does not contain the plate, and the gui test checks the same of the body.
- **`Preimage` is a field, not a `Classification`.** H0 considered a class for the kind and rejected one; a preimage plate stays `ClassUnknown` and inert on every classifier, and the reason it is refused here is that it is not on the allow-list, not that it is special. The flag only lets the screen say *which* unknown it was. `isPreimageRecord` runs only on the refusal path, which returns immediately, so the happy path pays nothing for it.

`gui/unlock_kdf.go` gained a named arm and two functions: `unlockNotPermittedBody` (`Record N is <kind>. This payload cannot be unlocked here. Nothing was opened.`) and `unlockRecordNoun`, which names a record for an *operator* rather than for a log — `seal.Classification.String()` says "unknown format", which is true of a preimage plate and useless to someone holding one. For a preimage the noun is H0's own reader words, **"a hashlock preimage, not a seed"**, and "not a seed" is the half that stops the operator re-cutting it as one. "Nothing was opened." is true and load-bearing: `AdmitSection` wipes every record it copied and returns none.

### RED, before the arm existed

    --- FAIL: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable (0.15s)
        unlock_preimage_test.go:46: never reached "hashlock preimage"; last frame "Payloadunreadable.SealedPayload"

The seal half was RED first as a compile failure (`undefined: RecordNotPermittedError`, three sites).

### A pre-existing test caught a defect in the first draft

The first `Error()` said `record 1 classifies as a hashlock preimage plate` — the kind **instead of** the class. `TestAdmitSectionRefusesAPreimagePlateAsUnknown` (H0's own) failed:

    record_test.go:470: error "...record 0 classifies as a hashlock preimage plate, which the encrypted section does not permit" does not name the class unknown

That test exists to pin that H0 keeps the plate `ClassUnknown`, and the new message had erased the fact it holds. `Error()` now names both — `unknown format (a hashlock preimage plate)` — and that test's own stale parenthetical ("the unlock screen renders this as *Payload unreadable.*; a named arm is an H2 follow-up") was updated in the same commit, because this **is** that arm.

### Six mutations, each run once and reverted

| Mutation | Measured failure |
| --- | --- |
| `Index: 0` in place of `Index: i` | `record 0, want 1 (records count from 0…)` **and** `record 0, want 2` — both rows, because they sit at different indices |
| `isPreimageRecord` always false | `the refusal does not report the record as a hashlock preimage` |
| drop `Unwrap()` | all three seal tests fail, incl. `the typed error no longer matches ErrRecordNotPermitted -- every existing caller is broken` |
| delete the `errors.As` arm from `unlockSealedFlow` | the RED again: `never reached "hashlock preimage"; last frame "Payloadunreadable.SealedPayload"` |
| hardcode `Record 1` in the body | the record-0, record-7 and record-2 rows fail (`does not carry "Record 0"`, `carries "Record 1" and must not`) — the flow test alone could **not** see this, which is why the body is table-tested |
| ignore the `Preimage` flag | both preimage rows report `not a format this machine reads` where `a hashlock preimage` is wanted |

The fit gate ran on all four body rows: **85 characters drawn in full, headroom 476 (margin 80)** for the longest.

### Records

`design/FOLLOWUPS.md`: **F-474 CLOSED** by fork `17b3979`, with what closed it and what gates it; **F-475's owning phase moved H2 → H3** with the reason in place — the H2 plan re-scheduled it on the condition that H2 does not touch `codex32_seam_vectors.json`, and it did not (H2 vendors `hashlock-v0.8.json`, a different file). Its original scheduling paragraph is kept rather than overwritten, since that argument is unchanged.

**The CHANGELOG line lives in the fork commit message, because the fork has no CHANGELOG file** — verified rather than assumed: `find -iname CHANGELOG*` returns nothing, and `git log --all --diff-filter=A -- 'CHANGELOG*'` shows the fork has never had one on any branch. The H2 plan says so too and names the commit message as the record. Creating a repo-wide CHANGELOG on a feature branch to hold one line is a decision for the operator, not this task; the line is in `17b3979`'s message under a `CHANGELOG` heading, ready to move.

### Final gate runs at fork `17b3979` — verbatim tails

`go test -count=1 ./hashlock/... ./codex32/... ./seal/... ./sysw/...`

    ok  	seedhammer.com/hashlock	0.230s
    ok  	seedhammer.com/codex32	0.003s
    ok  	seedhammer.com/seal	11.824s
    ok  	seedhammer.com/sysw	0.037s

`scripts/gui-shard-test.sh ./gui/ 24`

        1222 top-level tests
        partition verified exhaustive: 1222 == 1222
    === wall: 30s ===
    RESULT: ok -- all 1222 tests ran across 24 shards

**1222 = 1220 + 2**, the two new `gui` tests (`TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable`, `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind`); `seal`'s three new tests are a separate package and outside the count.

`go vet ./hashlock/... ./codex32/... ./seal/... ./sysw/... ./gui/... ./cmd/emu/`

    gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file is go1.25)
    gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
    gui/transaction_golden_test.go:104:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)

`gofmt -l .` (minus `third_party`)

    gui/transaction.go
    gui/transaction_golden_test.go
    gui/transaction_txrecord_test.go
    mt/mt.go
    mt/mt_test.go

Both the same as before Task 7, and both verified pre-existing at `c4a64fc`. `git status` clean at the tip, so no mutation survives.

### Not re-run, and why

The emulator walk and the firmware size were **not** re-measured for Task 7. The walk drives the composer's phrase route and never enters the unlock flow, so nothing it asserts is reachable from this diff. The size delta is not claimed for `17b3979`; if the controller wants a number for the merge, it should be re-measured at the merge tip, since this commit adds a struct, a method and two functions to code that is linked either way.
