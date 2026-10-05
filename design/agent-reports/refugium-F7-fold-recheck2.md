# Refugium F7: independent re-check of the fold, round 2

- **Reviewer:** independent fold re-check subagent, round 2 (Claude Opus 5.5). I am not the implementer, the execution reviewer, or the round-1 re-checker.
- **Date:** 2026-10-05
- **Diff:** `e70a941..e84770a`, which is the fold commits d8218df, b9271ad and e84770a on `claude/project-thread-59q2a6-f7`.
- **Scope:**
  1. whether each round-1 finding (I-2r, m-1, n-1, n-2) is fixed;
  2. whether the fold introduced a new defect, in particular in:
     - the `composerDoorRows` extraction;
     - `takePayloadOffer`;
     - the controller's decision to switch off Engrave Text's QR entirely under the profile (e84770a);
  3. whether the 5 new whole-test skips and 2 subtest skips (`refugiumSkipFreeTextQR`) are legitimate.

  On request, I also re-ran the ELF check and the three emulator walks on HEAD.
- **Counts:** **0 Critical, 0 Important, 1 Minor, 3 Nit**

**Verdict: CLOSES (0C/0I).**

- Every round-1 finding is fixed. m-1 is fixed by superseding the predicate rather than widening it.
- Turning the free-text QR off is complete on every device path, it is mutation-tight at both the QR step and the plate builder, and it leaves the default build unchanged.
- The ELF check and all three walks pass on HEAD.
- The Minor and the Nits can be handled at merge or in FOLLOWUPS.

## What I ran (each run once, output captured to a file under `scratchpad/rc2/`)

Toolchain: GOTOOLCHAIN=go1.26.7; TinyGo 0.41.1 (LLVM 20.1.1); PLAYWRIGHT_BROWSERS_PATH=/opt/pw-browsers. I did the builds and walks from `/home/claude/f7` at e84770a, which they did not modify. I did the mutations in the scratch worktree `/home/claude/f7-recheck2`, which I have since removed.

| Run | Result |
|---|---|
| `tinygo build -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks [-tags refugium] ./cmd/controller` | both rc=0. default.elf is 14,602,636 B and refugium.elf is 14,027,456 B. |
| `scripts/refugium-elf-check.sh default.elf refugium.elf` | **RESULT: ok.** 11 markers are present in default and absent in refugium; the 3 profile notices are the other way round. No st25r3916 Device method is in the Refugium ELF. The table is identical to the implementer's `fold-elfcheck.log`. |
| Extra ELF probe (whole-file grep) | "This build engraves text without a QR" occurs 0 times in default and once in refugium. The old "This text is an ms1 secret string" occurs 0 times in both. So the new branch is DCE'd from the default device build. |
| `scripts/emu-refugium-walk.sh default` | **ok:true**: presented 24, delivered 23, **readerAsks 2**. Variant 1 still offers TEXT+QR / TEXT ONLY / QR ONLY. |
| `scripts/emu-refugium-walk.sh refugium` | **ok:true**: presented 22, delivered 0, **readerAsks 0**. Every variant is TEXT ONLY, and the walk reaches "verify refused". |
| `scripts/emu-refugium-walk.sh refugium-attached` | **ok:true**: presented 22, delivered 0, **readerAsks 0** |
| Targeted baseline, `-tags refugium`, `-run '^(TestRefugiumFreeText\|TestF76\|TestF440\|TestWalkWalletPolicy\|TestComposerKeyPathChoice\|TestComposerConsentDoesNotClaim\|TestComposerDoorOffersFromPayload\|TestFT\|TestEngraveTextProgramSelectable\|TestBuildFlowScrubsEverySeedOnEveryExit\|TestFreeTextQRStaysOn)'` | ok. 63 RUN lines, no FAIL. Skips are exactly the 5 new FreeTextQR whole tests, the 2 FreeTextQR subtests, `TestF76IncompletePayloadNamesBothRoutesOnAnNFCMachine` and the pre-existing scrub subtest "Back at the passphrase prompt". |
| The same set, default build | ok, 61 RUN lines, no SKIP and no FAIL |
| `go test -race`, both builds, on the walks that call `composerDoorRow`/`takePayloadOffer` while the UI goroutine runs (F76 wallet policy, F440, both descriptor walks, key-path choice, F-671 consent, scrub) | ok, with 0 DATA RACE in both builds |
| `go vet ./gui`, with and without the tag; `gofmt -l gui/` | nothing new. Only the pre-existing `testing.ArtifactDir` go1.25 vet note and the known gofmt baseline. |
| Stale-reference grep (`containsMS1String`, `noMS1QRText`, `ftMS1NoQRNotice`, `ftQRLeadMS1`, the two removed test names) | none, anywhere in the repo |

### Mutation tests

Each mutant was applied by an exact single-occurrence replace, run, then reverted with `git checkout`. The unmutated set was green first.

| # | Mutant | Build | Result |
|---|---|---|---|
| 1 | **QR step:** `if noFreeTextQR()` changed to `if false` (`freetext_flow.go:530`) | refugium | **killed** by `TestRefugiumFreeTextQRStepOffersNoQR`, `TestRefugiumFreeTextWalkEngravesNoQR` and `TestEngraveTextProgramSelectable` |
| 2 | **QR step:** profile branch offers `{"No QR","Add QR"}` (`:535`) | refugium | **killed** by `TestRefugiumFreeTextQRStepOffersNoQR` and `TestRefugiumFreeTextWalkEngravesNoQR` |
| 3 | **Plate builder sink:** `if noFreeTextQR() { useQR = false }` changed to `if false` (`freetext_flow.go:1458`) | refugium | **killed** by `TestRefugiumFreeTextSinkNeverCutsAQR` |
| 4 | `noFreeTextQR` returns `false` (`ms1_qr_gate.go:47-49`) | refugium | **killed** by all four Refugium free-text tests |
| 5 | `noFreeTextQR` returns `true` (default-build control) | default | **killed** by `TestFreeTextQRStaysOnInTheDefaultBuild` (all three texts) and by 8 default `TestFT*` QR tests |
| 6 | Text-step defensive arm disabled (`freetext_flow.go:1586`) | refugium | **survives**. This is an equivalent mutant; see Nit n-1. |
| 7 | `syswChoose` under the profile returns `false`, so the payload is not taken (`sysw_session.go:251-252`) | refugium | **killed** by all 9 ported payload-door walks, `TestF76ACorruptedChunkInThePayloadIsStillRefused` among them |
| 8 | `syswChoose` draws the offer under the profile, so `takePayloadOffer`'s no-op would be wrong | refugium | **killed** by the same 9 |
| 9 | `syswPrimeCard` bypasses `offer()` for md1 under the profile | refugium | **killed** by `TestF76PrimingOnlyEverAddsToTheIdentifiedSet` and `TestF76InspectDescriptorCompletesFromThePayload` |
| 10 | `syswPrimeCard` primes nothing under the profile | refugium | **killed** by `TestF76InspectDescriptorCompletesFromThePayload` and `TestF76InspectKeyCompletesFromThePayload` |
| 11 | `discardLast` stops zeroing the mnemonic (n-2) | refugium | **killed** by the new subtest "Back on the passphrase step" (and by "ctx.Done unwind") |

## Finding by finding

| Round-1 finding | Status | Evidence |
|---|---|---|
| **I-2r**: 11 whole-test skips of kept behaviour | **Fixed** | See the detail after this table. |
| **m-1**: free-text gate punctuation | **Fixed, by supersession** | See the detail after this table. |
| **n-1**: `return !ctx.Done` | **Fixed** | The comment at `passphrase_off.go:52-53` now says the expression is defensive and that Done flips only in `ctx.Frame`. |
| **n-2**: no end-to-end scrub for Back on the notice | **Fixed** | The new subtest "Back on the passphrase step" (`multisig_build_scrub_test.go:168-203`) runs in both builds and passes. Mutant 11 shows that it is what detects a non-zeroing `discardLast` under the tag. |

**I-2r in detail.**

- All 10 skips that the round-1 re-check called illegitimate are removed. They run and pass under the tag:
  - the six F76 door tests apart from the NFC-arm one, including `TestF76ACorruptedChunkInThePayloadIsStillRefused` at `payload_door_walk_test.go:369`;
  - F440 (`modal_back_test.go:86`);
  - both Descriptor walks (`wallet_policy_descriptor_walk_test.go:128,207`);
  - `TestComposerKeyPathChoiceIsPlacedBeforeTheChunks` (`composer_unspendable_test.go:236`);
  - `TestComposerConsentDoesNotClaimLianaImportsASameSeedWallet` (`composer_f671_test.go:200`).
- `TestComposerDoorOffersFromPayloadOnlyWhenThePayloadHasOne` now runs too. It asserts that "Scan cards" is offered exactly where `!refugiumProfile` (`composer_door_test.go:107-110`), which is an improvement on the subtest-level skip that round 1 suggested.
- Only `TestF76IncompletePayloadNamesBothRoutesOnAnNFCMachine` still steps aside. Its subject is the NFC arm, so that skip is legitimate.
- Mutants 7 and 8 show the ported walks are not vacuous under the profile, including the corrupted-chunk funds-safety test. If the payload is not taken, no partial set forms and "Dropped an incomplete card" never draws. If the offer is drawn, nothing presses it.

**m-1 in detail.**

- d8218df widened the predicate.
- e84770a then removed the free-text predicate entirely: `containsMS1String` and `noMS1QRText` are deleted, and `noFreeTextQR()` is `refugiumProfile` (`ms1_qr_gate.go:47-49`).
- All three free-text sites now key on it: the QR step `:530`, the sink `:1458`, and the defensive text-step arm `:1586`.
- The round-1 per-line-label case that no predicate caught is now in `refugiumFreeTexts` (`refugium_profile_test.go:634`), and it gets no QR.
- The bundle and codex32 producers keep `isMS1String` and `noMS1QR`, unchanged.

## New-defect review of the fold

- **`composerDoorRows` extraction** (`composer_door.go:127-161`). This is a pure move. A diff against e70a941 shows only the lines that moved. `composerDoorLines` is pure, so computing the rows after the lead changes nothing. The rows are in the same order, under the same conditions: `scanOffered`, consumable policy, unconditional Build, preimage. The ChoiceScreen and the `routes[sel]` mapping are unchanged. The default build's door is unchanged.
- **`composerDoorRow`** (`profile_skip_test.go:83-95`). It derives the index from the same function the flow draws from, so it cannot disagree with the drawn door. It does not on its own assert the row order, but the order was never the subject of these walks. `TestComposerDoorOffersFromPayloadOnlyWhenThePayloadHasOne` still pins the rows' presence. It is called while the UI goroutine runs, and `-race` is clean in both builds.
- **`takePayloadOffer`** (`profile_skip_test.go:97-110`):
  - In the default build it behaves exactly like the code it replaced: wait for the needle, fail the test if it never draws, then press Button3 for FROM PAYLOAD at row 0.
  - Under the profile it is a no-op. That is correct only because `syswChoose` returns true without drawing for `syswAltScan` when `!scanOffered()` (`sysw_session.go:250-253`).
  - Mutants 7 and 8 show that a test cannot pass in either direction without exercising the auto-take.
  - Every call site passes the right needle, and the Descriptor walk keeps its "FROM PAYLOAD" lead assertion in the default build (`wallet_policy_descriptor_walk_test.go:156-171`).
- **Free-text QR off: is any QR path left under the profile?** I traced every write of `useQR` in `freetext_flow.go` and `freetext_proof.go`:
  - `ftQRChoiceFlow` returns `sel == 1`, and its one-choice screen cannot yield that.
  - `ftRefuse` (`:1088`) only clears the flag.
  - `ftProofLoader` (`freetext_proof.go:766`) only clears the flag, and the proof triggers are `!refugium` in any case.
  - `ftBuildPlate` forces `useQR` false at the last sink before `ftFitAt` → `EngraveFitted`.
  - The only other caller of `ftFitAt` with a QR flag is `fittedPreviewAt` (`preview.go:176`). That file is `//go:build !tinygo`, the host plateview tool, so it is not a device path.
  - **No path remains.** The device ELF probe, the walk's TEXT ONLY variants and mutants 1 to 4 agree.
- **Default build unchanged.**
  - `TestFreeTextQRStaysOnInTheDefaultBuild` (`freetext_qr_build_test.go:15-41`) passes and detects mutant 5.
  - The default `TestFT*` set runs with no skips.
  - The default ELF does not contain the new lead.
  - The default walk is identical to before (readerAsks 2, TEXT+QR offered).
- **Are the new skips legitimate?** I read each skipped test.
  - `TestFTRefusalOffersTheQRRatherThanDroppingIt`, `TestFTQREncodesTheTextOnly`, `TestFTBuildPlateEncodesOnce` and `TestFTQRChoiceLabelsBindToMeaning` have the QR or the two-row QR screen as their subject. **Legitimate.**
  - The `TestFTBuiltPlateIsTheFittedComposition/qr=true` subtest compares a QR-on `Fit` against `ftBuildPlate`, which by design drops the QR under the profile. **Legitimate.** The `qr=false` arm runs.
  - The `TestFTConfirmCarriesTheSafetyCopy/with a QR` subtest checks the QR-specific camera warning. **Legitimate.** The "without" arm runs.
  - `TestFTPlateIsWhatWasApproved` is legitimate as written, because it asserts a QR and needs a paged confirm screen produced by the QR's budget. Its subject, however, is broader than the QR. See Minor m-1.
  - The QR-adjacent tests that still run under the tag are `TestFTBackPreservesEveryValue` (with "No QR" as the kept value), `TestEngraveTextProgramSelectable` (with the profile's lead) and `TestFTNoQRMeansNoCode`. The last walks end to end, engraves, and asserts `QR == nil`.

## Critical

None.

## Important

None.

## Minor

### m-1. `TestFTPlateIsWhatWasApproved`'s invariant is not covered end to end under the profile

**Where:** `gui/freetext_flow_test.go:449`. The skip reason is `refugiumSkipFreeTextQR`.

**The gap:**
- The test's subject is spec 5's "the plate is what was approved". It checks that the multi-row composition paged on the confirm screen has the same lines, size, title and footer as the `Fitted` that `EngraveFreeText` is handed.
- Only its QR assertions and its paging precondition depend on the QR.
- With it skipped, the Refugium build has no end-to-end check of approval against the engraved plate for a composition longer than one row. `TestFTNoQRMeansNoCode` engraves "hi" and checks only the QR, title and footer.

**Why it is low risk:**
- The confirm fit (`ftEvaluate`) and the engrave fit (`ftBuildPlate`) share `ftFitAt`.
- The only Refugium divergence between them is the forced `useQR = false` at the sink. With the QR step forced to "No QR", that is a no-op on this path, and `TestFTBuiltPlateIsTheFittedComposition/qr=false` and `TestRefugiumFreeTextSinkNeverCutsAQR` both cover it.
- This is the same class as I-2r (a whole-test skip of kept behaviour), but it is not funds-safety.

**Fix (cheap):** split the test the way the fold split `TestFTBuiltPlateIsTheFittedComposition`. Under the profile, run it with `ftPastQR(h, false)`, fit with `useQR=false`, use a text long enough to page without a QR (or drop the `len(pages) < 2` precondition under the profile), and skip only the two QR assertions.

## Nits

- **n-1.** The defensive text-step arm in `gui/freetext_flow.go:1584-1589` is now unreachable. Under the profile, no path delivers `useQR == true` to `ftStepText`, so mutant 6 survives as an equivalent mutant, and `ftNoQRNotice` (`ms1_qr_gate.go:66-69`) is dead copy in the Refugium build. Its comment says it is defensive, which is acceptable. The sink at `:1458` is the real guarantee, and it is tested. Either keep the arm as written or delete it together with the notice. Do not count it as covered.
- **n-2.** The comment on `TestRefugiumFreeTextWalkEngravesNoQR` (`gui/refugium_profile_test.go:671-674`) overclaims what the walk does:
  - The comment says the per-line-label share "reaches the confirm screen as 'QR: no', and the plate built from it carries none".
  - The walk types only `"Line 1: ms10testsxxxx"`, which is not the share.
  - It stops at Confirm without engraving: the `ftRun` from `startFT` is discarded (`h, _ :=`).
  - The engraved-plate claim actually holds by `TestFTNoQRMeansNoCode` (under the tag) and by the sink test.

  **Fix:** reword the comment, or add `ftOK(h); h.step()` and assert `r.gotPlate && r.got.QR == nil`.
- **n-3.** The plan has not caught up with the controller's stricter decision. `mnemonic-engrave/design/IMPLEMENTATION_PLAN_fork_refugium_F5_F7.md:252-262` still says "the free-text gate uses the same predicate … ms1-shaped free text gets 'No QR' forced", whereas Engrave Text under the profile now offers no QR for any text. The decision is recorded only in the implementer's notes (`refugium-F7-impl.md`, last section). At merge, record it as a §4.4 deviation in the plan, or in the PR body or FOLLOWUPS, so the next reader of the plan does not re-add a predicate.

## Carried forward (unchanged, not this fold's)

- The NBSP-in-payload panic (round 1's FOLLOWUPS item) is pre-existing and out of scope. It is still owed to FOLLOWUPS.
- M-7 (observe the first CI run) is still open.
