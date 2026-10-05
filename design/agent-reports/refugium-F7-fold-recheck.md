# Refugium F7: independent re-check of the execution-review fold

- **Reviewer:** independent fold re-check subagent (Claude Opus 5.5). I am not the implementer and not the execution reviewer.
- **Date:** 2026-10-05
- **Diff:** `c05df0e..e70a941`: fold commits 9295013, 0ada371, a45d45b and c85294c, plus merge e70a941 of `origin/main` d156a3e (F5)
- **Scope:** proportional re-review. It covers (1) whether each finding of `refugium-F7-exec-review.md` (0C/2I/7M/5N) is fixed, and (2) whether the fold or the merge introduced a new defect. It does not re-audit settled parts.
- **Counts:** **0 Critical, 1 Important, 1 Minor, 2 Nit**. One pre-existing defect, not caused by F7, goes to FOLLOWUPS.

**Verdict: does not close yet.** One Important item remains: a residue of I-2. Eleven tests are still skipped as whole tests, but what they test is behaviour the Refugium build *keeps*. They fail only because their driver waits for a screen that the profile auto-takes or removes. One of them is a funds-safety test on the Refugium build's only input path. Every other finding is fixed, and the merge is clean.

## What I ran (each run once, output captured to a file)

All runs used GOTOOLCHAIN=go1.26.7 on this box (4 cores), in scratch worktrees at e70a941 (`/home/claude/f7-recheck`, `/home/claude/f7-recheck2`), which I have since removed.

| Run | Result |
|---|---|
| `GOFLAGS=-tags=refugium gui-shard-test.sh ./gui 24` | 1340 tests, partition exhaustive, all 24 shards ok, wall 76 s, rc=0 |
| `scripts/refugium-tagged-test.sh 4` | 1276 passed, 61 profile skips, 3 environment skips, of 1340. RESULT ok. This matches the implementer's numbers. |
| `go test ./backup ./sysw ./codex32 ./bip32` (default), plus `./backup` with `-tags refugium` | all ok (F5's ms1-SeedQR goldens included) |
| Skip probe: `skipUnderRefugium` made a no-op, then each of the 61 whole-skipped tests run alone under the tag (60 s cap each) | all 61 fail. Their first failure lines are classified under I-2r below. |
| Driver-only patch to 5 F76 tests (wait for the "Cards from where?" offer only when `!refugiumProfile`) | 4 of 5 **PASS** under the tag, the corrupted-chunk funds-safety test among them. The fifth failed only because my patch did not match its differently worded wait, so it was not exercised. |
| NBSP reproduction (both builds) | confirmed. See the FOLLOWUPS section. |

I relied on the implementer's logs for the emulator walks and the ELF check (`fold-walk-*.log`: readerAsks is 2 on default, 0 on refugium and 0 on refugium-attached; `fold-elfcheck.log`: RESULT ok). I did not rebuild them.

### Mutation tests (in the scratch worktree; each mutant was reverted, and the unmutated tagged `^TestRefugium` set was green first)

| Mutant | Result |
|---|---|
| I-1: `noMS1QRText` uses the old prefix predicate `isMS1String` | **killed**, by `TestRefugiumFreeTextSinkDropsTheQRForAnEmbeddedMS1` and `TestRefugiumFreeTextForcedOffQRIsSaid` |
| I-1: `ftBuildPlate` sink gate disabled | **killed**, by `TestRefugiumFreeTextSinkDropsTheQRForAnEmbeddedMS1` |
| I-1: text-step forced-off gate disabled (`freetext_flow.go:1586`) | **killed**, by `TestRefugiumFreeTextForcedOffQRIsSaid` |
| I-1: QR choice-step gate disabled (`:530`) | **killed**, by `TestRefugiumFreeTextMS1HasNoQR` |
| I-1: `ms1MinDataChars` set to 64 | **killed**, by all three free-text tests |
| I-1: no lowercasing | **killed** (sink test) |
| I-1: no separator stripping | **killed** (sink test) |
| I-1: threshold `n >= 0` (the false-positive direction) | **killed** (sink test's positive controls) |
| M-4: back to `sysw.Classify(r) == ClassPassphrase` | **killed**, by `TestRefugiumAnyPassPrefixedRecordIsFound` |
| M-1: Back on the notice returns true | **killed**, by `TestRefugiumPassphrasePromptIsANotice` and `TestRefugiumSeedPassphraseStepKeepsItsBack` |
| M-1: `return !ctx.Done` changed to `return true` in the dismissed branch | **survives**. This is an equivalent mutant (Nit n-1). |
| M-2: md1 refusal removed | **killed**, by `TestRefugiumIncompleteChunkGathersRefuse` |
| M-2: mk1 refusal removed | **killed**, by `TestRefugiumIncompleteChunkGathersRefuse` |

## Finding by finding

| Finding | Status | Evidence |
|---|---|---|
| **I-1** (ms1 prefix predicate) | **Fixed**, with a Minor residual (m-1) | `gui/ms1_qr_gate.go:51-75` `containsMS1String` (anywhere in the text, any case; strips Unicode space, `-` and `,`; then `ms1` plus at least 16 bech32 characters). All three free-text sites use `noMS1QRText` (`freetext_flow.go:530, 1458, 1586`). The bundle and codex32 producers keep the exact prefix predicate (`gui.go:2721`, `ms1_qr_gate.go:100`), which is correct for verbatim codec strings. All five review cases, plus upper case, dashed and space-grouped forms, are tested at the `ftBuildPlate` sink (`refugium_profile_test.go:618-661`), with three positive controls. |
| **I-2** (coarse whole-test skips) | **Partly fixed.** See **I-2r**. | The minimum list was ported: wipe residency, sealed re-entry, the scrub exits, both payload chains, build-from-payload-seed, retry loop, census and abort all run under the tag. Profile-aware drivers live in `profile_skip_test.go:36-73`. |
| **M-1** (Back on the notice) | **Fixed** | `passphrase_off.go:31-59`. Back returns `(0,false)`, acknowledge returns `(0,true)`, and Done returns false. I re-read all 8 call sites: `bip85.go:325`, `derive_xpub.go:427`, `gui.go:2872`, `multisig.go:145`, `multisig_build_slots.go:890` (`discardLast` zeroes the mnemonic, `:844-855`), `multisig_verify.go:953`, `singlesig.go:121`, `singlesig_verify.go:126`. On `!ok`, each either steps back or continues with no passphrase. None binds one. |
| **M-2** (Back-only gather) | **Fixed** | `md1_gather.go:109-114` and `mk1_inspect.go:202-207`. The check sits after `syswPrimeCard` and `complete()`, so a complete payload still proceeds. One `showError`, then return false. The default line is back to "Scan the next chunk.". |
| **M-3** (field-off write last) | **Fixed** | `cmd/controller/platform_sh2.go:243-263`. `initNFC` is now the first device operation after `dataI2C.Configure` and `newMultiplexI2C` (which does no bus I/O, `:641-647`). Nothing runs before `Init`: `run()` calls it first, and the only package `init` sets a hook (`debug_sh2.go:12`). The default `initNFC` does no I/O (`nfc_default.go:21-24`; `feats` is only ever OR-ed, `:291`), so moving it changes nothing in the default build. `st25r3916.Device.Close` is the one `regOpCtrl=0` write (`driver/st25r3916:297-302`). Unavoidable residue: if `dataI2C.Configure` itself fails, no write happens. |
| **M-4** (malformed `pass:`) | **Fixed** | `sysw_load.go:319-329` uses `strings.HasPrefix(r, sysw.PassPrefix)`. That is a strict superset of the old check: `Classify` uses the same case-sensitive prefix (`sysw/record.go:117-122`), so nothing that was refused before is admitted now. Mutation killed. |
| **M-5** (emu walk comments) | **Fixed** | `cmd/emu/platform_nfc_refugium.go:20-31`, `nfc.go:100-179` (`askCount`), and the walk asserts readerAsks (0/0 on the Refugium arms, 2 on default, per the logs). |
| **M-6** (producer coverage) | **Fixed** | `TestRefugiumFreeTextForcedOffQRIsSaid` (`:665`) and `TestRefugiumUnlockCodex32PlateIsTextOnly` (`:681`, byte-compare against `EngraveSeedStringTextOnly`), via the `unlockCodex32Plan` split (`unlock_session.go:349-361`). That split is behaviour-preserving. |
| **M-7** (CI) | **n/a**. Observe the first CI run. | |
| **N-1, N-2, N-3** | **Fixed** | `profile_refugium.go:19`; `sysw_load.go:164-167`; `refugium_build_test.go` comment |
| **N-4, N-5** | **Kept, as the reviewer allowed** | N-5's rationale (the verify record and the restore-document lines) is sound. |

## Merge (e70a941)

- **`backup/backup.go`.** The diff against main (`d156a3e`) contains **only** F7's hunks:
  - `EngraveSeedStringTextOnly`;
  - the nil-`qrc` guards in `engraveSeedString` (`:324-326`, `:351-355`).

  F5's `EngraveSeedStringSeedQR` and its helpers are byte-identical to main. F5 does not route through `engraveSeedString` with a nil QR.

  `go test ./backup` (F5's `ms1_seedqr_test.go` and vectors included) passes in both builds.
- **`gui/preview.go`.** F5's `ms1seedqr` entry is kept, and the proof entries stay in `preview_proofs_default.go`.
  - `ms1SeedQRPreview` is a SeedQR of the **words**, not of the ms1 string, which the spec allows.
  - It is reachable only through plateview's preview builder, not through a flow.
- **`gui/preview_ms1seedqr_test.go`.** `proofParams()` is replaced by `newPlatform().EngraverParams()`. This is **equivalent**: `proofParams` is exactly that expression (`freetext_proof_test.go:28-30`). It was needed because `proofParams` lives in a `!refugium` file.
- No new defect found in the merge.

## Critical

None.

## Important

### I-2r. Eleven whole-test Refugium skips test behaviour the build keeps; their drivers, not the product, are what fail

**Where:** `gui/payload_door_walk_test.go:139,169,192,294,359,391`; `wallet_policy_descriptor_walk_test.go:129,202`; `modal_back_test.go:87`; `composer_unspendable_test.go:237`; `composer_f671_test.go:196`.

The fold's skip list (`refugium-F7-impl.md`, "Remaining tagged skips") says every remaining skip "drives a behaviour the profile removes on purpose". The original I-2 fix criterion was to keep `skipUnderRefugium` only for tests "whose subject is the removed behaviour itself". I disabled the skip and ran each test alone under the tag. Their first failures fall into two groups.

**Group 1: the payload card or descriptor offer, which the profile auto-takes** (deviation 4: `syswChoose` takes the payload). The failure reads "…offer never drew.". The subject of each test is payload assembly, refusal or rendering. That is the Refugium build's **only** input route.

- `TestF76ACorruptedChunkInThePayloadIsStillRefused`. **Funds safety:** "the gatherer is the only thing standing between a corrupted chunk and a plate". Its walk part (3) is the only check that priming from the payload goes through `offer()` and the BCH checksum at the door.
- `TestF76CompletePayloadNeverSeesTheIncompleteRefusal`
- `TestF76IncompletePayloadGetsTheRepackAdvice`
- `TestF76BundleCountsACompleteMd1CardFromThePayload`
- `TestF76BundleCountsACompleteMk1CardFromThePayload`
- `TestF76WalletPolicyCountsACompleteMd1CardFromThePayload`
- `TestF440BundleIncompleteModalDismissesOnBack`
- `TestWalkWalletPolicyFromAPackedDescriptorRecordToTheDescriptorScreen`. Its subject: a classified Descriptor record reaches `DescriptorScreen` without the encode panic.
- `TestWalkWalletPolicyRendersARecordWithLeadingWhitespace`

**Group 2: the composer door, driven by row index.** The door has one row under the profile.

- `TestComposerKeyPathChoiceIsPlacedBeforeTheChunks` ("drew 1 touch targets; cannot select #1"). Its subject is the ordering of the unspendable-key step.
- `TestComposerConsentDoesNotClaimLianaImportsASameSeedWallet` (F-671 consent copy, "row 0 asked for, the frame has 0 tappable rows"). It is filed under the *passphrase* reason, but nothing in its failure involves a passphrase.

**Demonstrated:** with only a driver change (wait for and click the offer when `!refugiumProfile`), four of the five F76 tests I patched **pass** under the tag:

- corrupted-chunk refused;
- complete payload proceeds;
- incomplete payload gets repack advice;
- bundle md1 card counted.

So these are not hidden product regressions. They are the same blind spot I-2 described: a Refugium-only change to the auto-take path (for example, one that appended payload records to the gatherer without going through `offer()`) would ship with the corrupted-chunk door test silent. The tests that run under the tag (the md1/mk1 chain walks, `TestRefugiumIncompleteChunkGathersRefuse`) never feed a corrupted chunk through the door.

**Fix:**
- In the F76, F440 and descriptor walks, make the offer step conditional, either with `if !refugiumProfile { pumpUntil(offer); click }` or with a shared `takePayloadOffer(frame, title)` helper that is a no-op under the profile.
- Select composer-door rows by label, not by index.
- Keep the skip only on `TestF76IncompletePayloadNamesBothRoutesOnAnNFCMachine` (its subject is the NFC arm) and, at subtest level, on the "Scan cards" assertion of `TestComposerDoorOffersFromPayloadOnlyWhenThePayloadHasOne`.
- At minimum, port `TestF76ACorruptedChunkInThePayloadIsStillRefused` and the two other F76 door tests before merge.

**The other 50 whole-test skips are legitimate,** and I verified each by its first failure line:

- **Verify:** every one stops at "This build reads no cards over NFC…".
- **Passphrase:** the keyboard or payload passphrase is not offered, or "Payload Digest" never appears (pass: refused).
- **SLIP-39:** the three tests time out on the refusal path.
- **NFC scanner and seed-scan:** no reader and no SCAN row.
- **Carousel:** the hidden program itself.
- **Fable keyboard-Back:** legitimate.

I also spot-checked the 5 subtest skips, and the reasons hold. The scrub subtest "Back at the passphrase prompt" drives the keyboard, as its comment says (`multisig_build_scrub_test.go:115-117`).

## Minor

### m-1. The free-text ms1 predicate strips only whitespace, `-` and `,`, so an ms1 string grouped with other punctuation the keyboard offers still gets a QR

**Where:** `gui/ms1_qr_gate.go:54`. Measured under `-tags refugium` (scratch probe, since deleted):

- `noMS1QRText` is **false** for:
  - `ms10.test.sxxx.…` (also with `/`, `:` or `_` as the group separator);
  - a label per line that splits the string before 16 data characters (`"Line 1: ms10testsxxxx\nLine 2: …"`).
- It is **true** for:
  - upper case;
  - text split across lines or paragraphs (`\n\n` is stripped);
  - `ms\n10…`;
  - one character per space;
  - mixed `-`, `,` and space grouping.

The text keyboard types all of `. / : _ | ;` (`passphrase_keyboard.go:21-23`), so the gaps can be reached from typed text, not only from a payload.

This is the same accident class I-1 targeted (formatting around a share, not deliberate obfuscation), but a rarer form than a label in front of the share.

**Fix:** before the match, drop every rune that is not a letter or a digit, not just the three separators. False positives grow only marginally, and that is the direction the gate's philosophy accepts. Add `.`, `/` and `:` grouped cases to the sink test.

A ZWSP or NBSP *inside* the string also defeats or is handled by the predicate. That case is moot, because the plate panics first (FOLLOWUPS below).

## Nits

- **n-1.** `passphrase_off.go:52`: in `return !ctx.Done`, the negation is an equivalent mutant (`return true` survives). In this GUI, Done flips only in `ctx.Frame`, so after `Layout` reports a dismissal Done is still false. A wipe on the notice exits through the loop condition, `:58`. `TestRefugiumPassphraseNoticeEndedByDoneIsNotOK` pre-sets Done, so it tests only the loop guard. This is harmless, but either drop the expression or say in its comment that it is defensive.
- **n-2.** M-1 creates a new Refugium exit with a seed live: Back on the per-seed notice leads to `discardLast`. It has unit coverage (`TestRefugiumSeedPassphraseStepKeepsItsBack` checks that 0 seeds remain, and `discardLast` zeroes the words), but no end-to-end `assertScrubbed` walk like the other exits in `TestBuildFlowScrubsEverySeedOnEveryExit`. Adding a tagged subtest would be cheap.

## Implementer's side note: NBSP in a payload text record (CONFIRMED, PRE-EXISTING, goes to FOLLOWUPS)

**Reproduction** (scratch test, run in both builds, since deleted):

- `ftBuildPlate(params, &ftPlanSH, "hello world", "", "", false, 0, 0)` panics with `unsupported rune` (`engrave/engrave.go:1411/1569/1674`). It does the same for U+200B and for `é`.
- The full Engrave Text walk with the field pre-filled to `"hello world"` (the payload route pre-fills `kbd.Fragment` the same way, `freetext_flow.go:1505-1520`) does this:
  1. it passes the text step, Title, Footer **and the Confirm screen** (`ftEvaluate` → `FitBlocks` does not reject the rune);
  2. it then **panics** at the engrave step's `ftBuildPlate` → `EngraveFitted`.
- Same result with and without `-tags refugium`.

**How it is reached:**
- Only through a `text:` payload record. The keyboard types ASCII only, and `sysw.DecodeBody` accepts any bytes.
- The code is present at `be00ef8`, the pre-F7 base (`syswOffer(ctx, th, sysw.ClassFreeText, …)` / `DecodeBody`, `freetext_flow.go:1506-1507` there).
- So it is **not reachable only through F7 code**, and it does not belong to this PR.

**Impact:**
- The firmware panics after the operator has approved the plate, which is the "mid-flow with a plate clamped in the machine" class that `backup/fit.go:373` says the fit must catch as an error.
- No secret is exposed, and no QR is cut.

**Suggested FOLLOWUPS entry:**
- Refuse (or map) runes the plate font lacks at the fit or admission stage: `AdmissibleBlocks`/`FitBlocks` should return an error. Alternatively, refuse at `engraveTextFlowFrom` when the record enters.
- Add a tagged and default test with an NBSP payload record.
