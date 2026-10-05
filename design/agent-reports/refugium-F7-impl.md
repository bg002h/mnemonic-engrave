# Refugium F7: implementation report

The plan was `mnemonic-engrave/design/IMPLEMENTATION_PLAN_fork_refugium_F5_F7.md`. This report covers §4 only, with the binding pieces §2 D1, D3 and D4.

- **Worktree:** `/home/claude/f7`
- **Branch:** `claude/project-thread-59q2a6-f7`
- **Base:** fork main `be00ef8`
- **Date:** 2026-10-05
- **Status:** committed locally only. Nothing was pushed and no PR was opened.

## Commits

| SHA | Subject |
|---|---|
| `4348274` | gui: characterization tests for the start screen's debug-command arm |
| `6bb9985` | cmd/emu: count delivered NFC records; add the Refugium NFC walk |
| `944bfe2` | gui, cmd: the Refugium build profile; debug commands and NFC out of it (§4.1, §4.2) |
| `0a0d367` | gui: no BIP-39 passphrase in the Refugium build, loudly (§4.3) |
| `11061f2` | backup, gui: no QR of an ms1 string in the Refugium build (§4.4) |
| `c05df0e` | gui, ci: Refugium tests; tagged suite, ELF marker check (§4.5) |

Each intermediate commit was compiled from its own index in a scratch tree:

- the default build, its tests, and `go vet` of gui, backup and cmd/emu;
- `-tags refugium` for gui, including its tests;
- `GOOS=js` vet of both emulator builds;
- the `gofmt` baseline.

The suites below ran on the final tree. The TinyGo ELFs were built from the final production code; only tests, scripts and CI changed after they were built.

## Re-validation of §4 against be00ef8 (done before coding)

Every §4 citation held at be00ef8:

- **Debug commands:**
  - The arm is `gui/gui.go` StartScreen.Flow, handling the `FOREVERLAURA!` and `lock-boot` cases.
  - The `lock-boot: %v` log format sits beside the switch.
- **OTP writer (`cmd/controller/platform_sh2.go`):**
  - `LockBoot`, `writeOTPValues` and the white-label constants, including `https://seedhammer.com/doc/?d=SHII`, live here.
  - A grep confirmed this file is the only caller of `otp.WriteWhiteLabel*`, `AddBootKey` and `EnableSecureBoot`.
- **Proof triggers:**
  - The five `ftProofTrigger*` constants and the `ftProofs` table are in `freetext_proof.go`.
  - `ppPassProofTrigger` and the `ppPassProofKeep*` help strings are in `passphrase_passproof.go`.
  - Preview entries are in `preview.go`.
- **NFC call sites:**
  - Seven `startScanner(ctx, ctx.Platform.NFCReader())` sites.
  - Three `Features().Has(FeatureNFC)` reads.
  - The unkeyed scan offers named in R0 round 2 M-3.
- **Passphrase prompts:** the eight "Add a BIP-39 passphrase?" `ChoiceScreen.Choose` sites, plus SLIP-39's own prompt.
- **ms1 QR producers:** `backupSeedStringFlow`, `unlockEngraveCodex32`, the single-string branch of `validateMdmkStrings`, and Engrave Text's QR step.
- **TinyGo measurements:**
  - `writeOTPValues`, `AddBootKey` and `EnableSecureBoot` have no symbols in the default ELF (they are inlined).
  - `FOREVERLAURA!` occurs 0 times.
  - Every §4.5 marker is present (measured in the table below).

Nothing in §4 was false or impossible as written. Where I chose an implementation the plan left open, or departed from its wording, the choice is listed under **Deviations**.

## What was built (file:line)

### §4.1 Profile, debug commands, OTP writer, proof triggers

- **Profile constants** (`gui/profile_default.go`, `gui/profile_refugium.go:22,27`):
  - `refugiumProfile`: false in the default build, true in Refugium.
  - `proofTriggersEnabled`: true in the default build, false in Refugium.
- **Debug commands:** `StartScreen.Flow` now calls `handleDebugCommand`.
  - `gui/debugcmd_default.go:29` holds the original logic, which is characterization-tested.
  - `gui/debugcmd_refugium.go:27` recognises no command. It logs and stays on the start screen, and the command text is not logged.
- **QA flow:** `qaEngraveFlow` is gated by `if !refugiumProfile` at `gui/gui.go:2172`. It does not link into the Refugium ELF (see the marker table).
- **OTP writer:**
  - `LockBoot`, `writeOTPValues` and the white-label constants moved to `cmd/controller/lockboot_default.go:29`.
  - `cmd/controller/lockboot_refugium.go:13` returns an error and links no OTP write.
- **Proof triggers:**
  - The triggers, the `ftProofs` table and `ppPassProofKeep*` moved to `gui/prooftriggers_default.go`.
  - `gui/prooftriggers_refugium.go` has an empty table and false/"" stubs.
  - `ftProofForTrigger` and `ppPassProofOffer` also check `proofTriggersEnabled`.
  - The preview entries moved to `gui/preview_proofs_default.go` (`!tinygo && !refugium`).
- **Comments:** `seal/record.go` and `seal/record_test.go` comments were repointed to the new files.

### §4.2 NFC off (D3), two layers

**Platform layer**

- **Controller, default build:** `cmd/controller/nfc_default.go:21,26` has the original wiring. The `nfcDev` poller adapter moved here from `platform_sh2.go`, its only user.
- **Controller, Refugium build** (`cmd/controller/nfc_refugium.go`):
  - `initNFC` (`:26`) calls `Device.Close()`, which writes `regOpCtrl=0` and turns the field off on a warm reboot. A failure is recorded as `nfcFault`.
  - `NFCReader` (`:34`) returns an untyped nil.
  - `NFCFault()` is at `:41`.
  - There is no `FeatureNFC`, and `nfcDev` is an empty struct.
- **Emulator:**
  - `cmd/emu/platform_nfc_default.go` has the original wiring.
  - `cmd/emu/platform_nfc_refugium.go`: `Features` returns 0, and `NFCReader` returns nil unless `shNFC.forceReader()` was called (the "refugium-attached" walk arm).
  - `cmd/emu/nfc.go:88,131,191` adds `deliveredCount`, which counts dequeues in Read; this landed in commit `6bb9985`.

**gui layer** (`gui/nfc_scan.go`)

- `startScanner` (`:61-65`) treats any reader as nil under the profile.
- `nfcReader()` (`:161`) and `nfcAvailable()` (`:170`) are the only places gui asks the platform. `TestNFCIsAskedForInOnePlace` enforces this.
- `scanOffered()` is at `:188` and `nextChunkLine()` at `:198`.

**NFC fault**

- `gui/nfc_fault.go`: an optional `nfcFaulter` interface (Platform is unchanged) and `nfcFaultScreen`, which shows "NFC could not be turned off. Power off." and takes no input.
- It is checked first in `uiFlow` (`gui/gui.go:2100-2105`).

**Verify flows**

Both refuse up front with "This build reads no cards over NFC, so it cannot read these plates back. Verify on another build.":

- `gui/singlesig_verify.go:106` refuses before the seed is retyped and writes no verify bit.
- `gui/multisig_verify.go:777` refuses after its two guards and before the gather.

#### Per-flow NFC list (§4.2): every place a tag could be read

All seven reader sites now use `ctx.nfcReader()`, which returns nil under the profile; `startScanner` nils it again.

| Site | Flow | Refugium |
|---|---|---|
| `gui/gui.go:2265` | StartScreen.Flow (every scanned object, debug commands) | no reader |
| `gui/derive_xpub.go:343` | scanSeedFlow (seed "SCAN" row) | no reader; the row is not offered |
| `gui/bundle_flow.go:225` | bundleGatherFlowResume (bundle and verify readback gather) | no reader; verify refuses before reaching it |
| `gui/verify_address.go:82` | scanAddressFlow | no reader; the Scan/Type choice is skipped |
| `gui/md1_gather.go:111` | md1GatherFlow (single-card chunks) | no reader; the line says "This build takes chunks from the payload only." |
| `gui/mk1_inspect.go:204` | mk1GatherFlow | same |
| `gui/transaction.go:750` | transactionGatherFlow | no reader; `hasReader` is false |

Above the reader sites, the emulator walk (below) presents a tag on every screen from the boot offer to the Restore Doc and asserts zero deliveries.

#### Scan-offer list (§4.2)

**Keyed on `nfcAvailable()`** (previously `Features().Has(FeatureNFC)`):

- `gui/derive_xpub.go:287`: the seed picker's SCAN row.
- `gui/bundle_flow.go:195`: the gather's `hasReader`, which drives the "scan…" lines.
- `gui/transaction.go:316`: the transaction gather's `hasReader`.

**Never keyed, now on `scanOffered()`:**

- `gui/verify_address.go:23`: the "Scan / Type" choice is skipped, and the keyboard opens.
- `gui/composer_door.go:132`: "Scan cards" is dropped. "From payload" and "Build a new policy" stay.
- `gui/sysw_session.go:251`: `syswChoose` with the scan alternative takes the payload without drawing a one-row picker.
- `gui/md1_gather.go:142` and `gui/mk1_inspect.go:232`: "Scan the next chunk." becomes `chunksFromPayloadOnly`.

### §4.3 Passphrases off (D4), loudly

`gui/passphrase_off.go`:

- `askBIP39Passphrase` shows `passphraseOffNotice` ("This build takes no BIP-39 passphrase. The seed is used without one.") under the question's title. It answers `(0, true)`, which means Skip.
- `slip39PassphraseRefusal` is defined here.

#### Passphrase call-site semantics

| Site | Prompt | Default | Refugium | How the result is read |
|---|---|---|---|---|
| `gui/gui.go:2883` | backupWalletFlow | asks | notice → Skip | `ok && sel==1` adds; otherwise none |
| `gui/derive_xpub.go:427` | deriveXpubFlow | asks | notice → Skip | same |
| `gui/singlesig.go:121` | singleSigInputs | asks | notice → Skip | same (`got && sel==1`) |
| `gui/singlesig_verify.go:126` | singleSigVerifyFlow | asks | unreachable (verify refuses at `:106`) | same |
| `gui/multisig.go:145` | supplyMultisigPolicyFlow | asks | notice → Skip | same |
| `gui/multisig_verify.go:953` | multisigVerifyFlow `legStepPassphrase` | asks | unreachable (refuses at `:777`) | `!ok` steps back to the seed |
| `gui/multisig_build_slots.go:890` | seedPassphraseStep | asks | notice → Skip | `!ok` re-asks; never false under the profile |
| `gui/bip85.go:325` | bip85DeriveFlow | asks | notice → Skip | `!ok` steps back |

`TestEveryBIP39PassphrasePromptIsAsked` pins these eight sites at the source and runs in both builds.

Other passphrase entry points:

- **Missed prompts:** `passphraseFlowTitled` (`gui/gui.go:935`) and `syswPassphraseFlowTitled` (`gui/sysw_source.go:102`) return `("", false)` under the profile without drawing. At every caller, false means no passphrase was given.
  - slip39 reads false as "abort". `seedPassphraseStep` reads `!ok` from its own question as "ask again".
  - Neither reading is reachable under the profile, because the question before them is a notice or a refusal there.
- **SLIP-39:** `gui/slip39_polish.go:293`. The question is still asked, because skipping it would silently recover a different seed. A "yes" shows "This build cannot take a SLIP-39 passphrase. Recover these shares on another build." and ends the recovery.
- **Carousel:**
  - `programHidden` (`gui/gui.go:2643`) hides the passphrase program.
  - The prev/next loops skip it (`gui/gui.go:2305-2328`).
  - `layoutMainPager` (`:2607`) draws one dot fewer.
  - The `engravePassphrase` dispatch is gated too.
- **Payload refusal:** a payload with a `pass:` record is refused whole at `gui/sysw_load.go:168`, after Open and before any session is created.
  - The message names the record by position, never by content: "Record N of this payload is a BIP-39 passphrase (pass:)…".

### §4.4 No QR of an ms1 string

`gui/ms1_qr_gate.go`:

- `isMS1String` strips whitespace, '-' and ',', then compares the first three characters to "ms1", case-insensitive.
- `noMS1QR` applies it under the profile.
- `engraveSeedStringPlate` holds the notices.

#### ms1-QR producers gated

| Producer | File:line | Refugium |
|---|---|---|
| Codex32 engrave (`backupSeedStringFlow`) | `gui/gui.go:2925` | `backup.EngraveSeedStringTextOnly` for ms1 |
| Sealed-unlock codex32 plate | `gui/unlock_session.go:206` | same |
| Bundle ms1 card variants (`validateMdmkStrings` single-string branch) | `gui/gui.go:2732` | only TEXT ONLY (no TEXT+QR / QR ONLY) |
| Engrave Text QR step | `gui/freetext_flow.go:530` | only "No QR", with the ms1 lead |
| Engrave Text, QR chosen before an ms1 text was typed | `gui/freetext_flow.go:1586` | QR forced off, with a notice |
| Engrave Text plate builder (last sink) | `gui/freetext_flow.go:1458` | `useQR=false` for ms1 |

`backup/backup.go:183` adds `EngraveSeedStringTextOnly`:

- It runs the same encode and size checks, then `engraveSeedString(params, plate, nil)`.
- `engraveSeedString` now handles a nil QR.
- New goldens are `backup/testdata/codex32-{0,1}-text.bin`. The existing codex32 goldens are unchanged, and `go test ./backup` passes.
- `TestSeedStringPlatesGoThroughTheMS1Gate` pins `backup.EngraveSeedString` to the gate.

### §4.5 Gates

**Source level, every build** (`gui/refugium_build_test.go`, 5 tests):

- `TestRefugiumFileSetsCarryNoForbiddenLiteralOrCall`:
  - It runs `go list` over gui (`refugium` and `tinygo,rp,refugium`) and over `cmd/controller` (`tinygo,rp,refugium`).
  - It uses an AST check: `BasicLit` for literals and `CallExpr` for calls.
  - Positive control: the default sets (`""` and `tinygo,rp`) must contain each literal and call. The one exception is `WriteBootKey`, which is in the forbidden list but not called even in the default build.
- `TestRefugiumPairsAreWellFormed`: every `_refugium.go` file has a `_default.go` twin in the paired constraint. There are 6 pairs.
- `TestNFCIsAskedForInOnePlace`
- `TestEveryBIP39PassphrasePromptIsAsked`
- `TestSeedStringPlatesGoThroughTheMS1Gate`

**Behaviour, tagged** (`gui/refugium_profile_test.go`, 20 tests). They were mutation-checked: with `refugiumProfile` set to false under the tag, 19 of 20 fail. `TestRefugiumDebugCommandsAreInert` passes either way, because it tests the tagged file directly. The tests:

- profile on;
- `startScanner` reads nothing from a live counting reader;
- `nfcReader`/`nfcAvailable`/`scanOffered` never consult the platform;
- a start-screen lock-boot tag over a reader-equipped platform: 0 reads, 0 NFCReader calls, LockBoot never called, Flow does not return;
- the debug arm is inert;
- each scan offer: seed entry, verify-address, composer door, syswChoose, and the chunk-gather line;
- verify refuses and the verify record stays zero;
- the passphrase notice answers Skip for both Button3 and Button1;
- passphrase entry returns nothing;
- SLIP-39 with a passphrase stops;
- the carousel skips only the passphrase program;
- the `chain-pass` payload is refused without showing the secret;
- the ms1 seed-string plate equals the text-only plate and differs from the QR plate;
- an ms1 card offers TEXT ONLY;
- the Engrave Text QR step for ms1;
- the NFC fault screen survives eight buttons.

**Tagged-suite runner** (`scripts/refugium-tagged-test.sh`):

- It enumerates with `go test -list` under the tag, partitions into anchored shards, and asserts the union is exhaustive.
- It runs with `-v` and accepts only three outcomes:
  - top-level PASS;
  - a SKIP whose first log line is `refugium profile: …`;
  - one of 3 named environment skips, which skip in every build.
- PASS + skips must equal the listed count.
- It asserts the tag took effect: `TestRefugiumProfileIsOn` must be listed.

**ELF check** (`scripts/refugium-elf-check.sh`):

- Pure python3 ELF32 parsing. It scans allocated sections only, plus defined symbols in allocated sections.
- It has a reverse control: the profile's own notices must be absent in the default ELF and present in the Refugium ELF.

**CI** (`.github/workflows/test.yml`):

- The tests job gains `GOOS=js GOARCH=wasm go vet -tags refugium ./cmd/emu/` and `./scripts/refugium-tagged-test.sh`.
- The TinyGo job:
  - builds `default.elf` (keeping the existing `-size full -print-stacks` report) and `refugium.elf`;
  - runs the marker check;
  - uploads both ELFs (`actions/upload-artifact@v4`).
- Every step calls a script that was run locally. The CI jobs themselves have not run, because nothing was pushed.

## ELF marker table

Both ELFs were built with TinyGo 0.41.1 (release tarball, not nix): `tinygo build -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks [-tags refugium] ./cmd/controller`, with the final production code.

Allocated bytes scanned: default 1,666,816; Refugium 1,597,376. The ELF files are 14,438,308 and 13,959,448 bytes.

| Marker | Kind | default | refugium |
|---|---|---|---|
| `lock-boot: %v` | data literal | 1 | 0 |
| `seedhammer.com/doc/?d=SHII` | data literal | 1 | 0 |
| `PASSPROOF` | data literal | 2 | 0 |
| `TEXTPROOF` | data literal | 1 | 0 |
| `CONSTPROOF` | data literal | 1 | 0 |
| `BOTHPROOF` | data literal | 1 | 0 |
| `SIZEPROOF` | data literal | 2 | 0 |
| `(*seedhammer.com/nfc/poller.Poller).Read` | symbol | 1 | 0 |
| `seedhammer.com/gui.qaEngraveFlow` | symbol | 1 | 0 |
| `seedhammer.com/driver/otp.writeECC` | symbol | 1 | 0 |
| `seedhammer.com/driver/otp.writeOrRow` | symbol | 1 | 0 |
| profile: `NFC could not be turned off` | reverse control | 0 | 1 |
| profile: `This build takes no BIP-39 passphrase` | reverse control | 0 | 2 |
| profile: `This build reads no cards over NFC` | reverse control | 0 | 1 |

`RESULT: ok`. The script also fails correctly when given two default ELFs.

Running against the plan's own base `default.elf` (built from be00ef8) in place of the new default ELF gives the same verdict. That shows the default build's markers are unchanged by F7.

**st25r3916 `Device` methods.**

- Default ELF: Close, Read, commandAndWait, configureProtocol, enable, `handleInterrupt$bound`, interruptStatus, resetInterruptMask, waitForInterrupt, write, writeReg, writeRegs.
- Refugium ELF: none of them is a symbol.
- `Close` and `writeReg` are inlined into `(*main.Platform).initNFC` (0x64 bytes). Its disassembly contains the I2C `Tx` interface call and `fmt.Errorf`, i.e. the `regOpCtrl=0` write and its error wrap.

Before `nfcDev` moved to `nfc_default.go`, `Device.Read` and `write` still linked into the Refugium ELF through the unused adapter type. Moving the adapter removed them.

`NFCFault` has no symbol of its own. TinyGo devirtualised the single-implementation assertion. The fault screen's text is linked into the Refugium ELF only, as the reverse-control row shows.

## Emulator walks (`scripts/emu-refugium-walk.sh`, Playwright 1.56.1, headless Chromium)

The default walk ran FIRST, before any profile code. It was the positive control, recorded at commit `6bb9985`:

- `ok:true`, presented=24, delivered=23.
- Variants: card 1 offered `TEXT+QR / TEXT ONLY / QR ONLY`; cards 2 and 3 offered `TEXT ONLY`.

After the profile code was in place:

| Build | ok | presented | delivered | Card 1 (ms1) variants |
|---|---|---|---|---|
| default (re-run) | true | 24 | 23 | TEXT+QR / TEXT ONLY / QR ONLY (unchanged) |
| refugium | true | 22 | 0 | TEXT ONLY |
| refugium-attached (reader forced on) | true | 22 | 0 | TEXT ONLY |

Per-screen counters, as presented/delivered:

| Screen | default | refugium | refugium-attached |
|---|---|---|---|
| start screen | 2/2 | 2/0 | 2/0 |
| seed source | 2/2 | 2/0 | 2/0 |
| wallet type | 5/2 | 5/0 | 5/0 |
| passphrase | 6/2 | 6/0 | 6/0 |
| engrave mode | 7/2 | 7/0 | 7/0 |
| census | 9/2 | 9/0 | 9/0 |
| verify offer | 19/2 | 19/0 | 19/0 |
| verify gather (default) / verify refused (Refugium) | 23/23 | 20/0 | 20/0 |
| restore doc | 24/23 | 22/0 | 22/0 |

What each walk showed on screen:

- **Default:** the picker showed "Where from? TYPE IT / SCAN" and the passphrase step showed "Add a BIP-39 passphrase?".
- **Refugium:** the walk went straight to "Choose number of words" and showed the passphrase notice.

In the default build, records presented while no scanner runs stay queued, so the verify gather drains them. That is why the default delivered count jumps from 2 to 23 at the gather.

## Test commands and counts

All Go commands ran with `GOTOOLCHAIN=go1.26.7`, from `/home/claude/f7`. Each suite's output was captured once to a file and then grepped.

| Command | Result |
|---|---|
| `/home/claude/mnemonic-engrave/scripts/gui-shard-test.sh ./gui 24` (default, final tree) | **1411 tests**, partition exhaustive, all 24 shards ok, wall 102 s |
| `scripts/refugium-tagged-test.sh 24` (= `GOFLAGS=-tags=refugium`, anchored shards, `-v`) | **1333 listed: 1237 PASS, 93 SKIP (refugium profile), 3 SKIP (environment), 0 FAIL**, wall 60 s |
| `CGO_ENABLED=0 go test -timeout 20m $(go list ./... \| grep -v '^seedhammer.com/gui$')` | 55 packages ok, rc=0 |
| `go test ./cmd/emu/ ./backup/` | ok; includes the 4 delivered-count host tests and the 3 text-only seed-string tests |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` and the same with `-tags refugium` | clean |
| `CGO_ENABLED=0 go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/ ./cmd/emu/` (and `-tags oraclelive,refugium ./gui/`) | compiles |
| `./scripts/test-32bit.sh` | rc=0 |
| `scripts/refugium-elf-check.sh f7-default.elf f7-refugium.elf` | RESULT ok (table above) |
| `scripts/emu-refugium-walk.sh {default,refugium,refugium-attached}` | all `ok:true` (table above) |
| `gofmt -l .` | exactly the five-file baseline (gui/transaction.go, gui/transaction_golden_test.go, gui/transaction_txrecord_test.go, mt/mt.go, mt/mt_test.go) |

The default count is 1411, up from be00ef8's suite. It adds:

- 3 debug-command characterization tests;
- 5 source-level gate tests;
- the moved and refactored tests, which keep their count.

The 20 tagged behaviour tests run only under the tag.

## Tagged-suite exclusions

Tests are excluded in two ways.

### `//go:build !refugium` on the file (98 tests)

These are proof-trigger tests, plus the default-arm characterizations. The things they drive do not exist in the Refugium build.

| File | Tests |
|---|---|
| `freetext_proof_test.go` | 30 |
| `freetext_proof_sizes_test.go` | 9 |
| `freetext_sizeproof_test.go` | 27 |
| `freetext_sizeproof_table_test.go` | 4 |
| `freetext_sizeproof_golden_test.go` | 2 |
| `passphrase_passproof_test.go` | 21 |
| `freetext_proof_gear_test.go` | 2 (moved unchanged from `freetext_settings_test.go`/`freetext_speed_test.go`; they reach the gear through `ftProofTriggerConst`) |
| `debugcmd_characterization_test.go` | 3 |

The `-update` golden flag moved to the untagged `golden_flag_test.go`, because the chain-plate and transaction goldens use it in both builds.

### `skipUnderRefugium(t, <reason>)` (93 tests)

The helper is in `gui/profile_skip_test.go`. It skips only when the profile is on, and the message starts with `refugium profile:`. The default suite runs all 93 tests unchanged.

#### Carousel: the passphrase program is hidden, so program positions shift (§4.3) — 16
TestBip85DeriveProgramNavigable (bip85_program_test.go), TestEngraveBundleProgramNavigable (bundle_program_test.go), TestEngraveMultisigProgramNavigable (multisig_program_test.go), TestEngraveSingleSigProgramNavigable (singlesig_program_test.go), TestEngraveTextProgramNavigableByTouch (text_program_test.go), TestEngraveTextProgramSelectable (text_program_test.go), TestEngraveXpubProgramNavigable (derive_xpub_program_test.go), TestPassphraseProgramReachable (gui_test.go), TestStartScreenPagerTouchReachesEveryProgram (start_screen_touch_test.go), TestStartScreenPagerTouchable (start_screen_touch_test.go), TestStartupProbesWithoutReadingTheRegion (unlock_program_test.go), TestUnlockPayloadEntrySelectable (unlock_program_test.go), TestUnlockPayloadInvisibleWithoutAPayload (unlock_program_test.go), TestUnlockPayloadVisibleWithAPayload (unlock_program_test.go), TestWalletPolicyProgramIsNavigableAndOpens (wallet_policy_test.go), TestWipeZeroesEveryPinnedBufferAtRunLevel (wipe_inventory_audit_test.go)

#### Passphrase: the prompt is a notice, entry returns nothing, a pass: payload is refused (§4.3) — 24
TestBip85BackStepsBackAndLosesNothing (bip85_test.go), TestBothEngraveFlowsDriveTheRetryLoop (multisig_engrave_tail_walk_test.go), TestBuildAbortIsTheLastScreenOfTheProgram (multisig_engrave_tail_walk_test.go), TestBuildFlowScrubsEverySeedOnEveryExit (multisig_build_scrub_test.go), TestBuildRefusesDuplicateOnAPayloadSourcedSeed (multisig_build_payloadseed_test.go), TestBuildTakesTheSelfSeedFromThePayload (multisig_build_payloadseed_test.go), TestBuildWalkTypedSeed (multisig_build_walk_test.go), TestChainMdMkFromTheEmulatorsOwnPayloadToFourPlates (chain_class_walk_test.go), TestChainMnemonicFromAMePackedPayloadToASeedPlate (chain_class_walk_test.go), TestChainPassphraseFromAMePackedPayloadToAPasswordPlate (chain_class_walk_test.go), TestComposerConsentDoesNotClaimLianaImportsASameSeedWallet (composer_f671_test.go), TestDecliningTheSeamPassphraseOfferReachesTheKeyboard (sysw_cells_test.go), TestFableSpecBackOnThePassphraseKeyboardIsADecline (composer_fable_r0_flow_test.go), TestFableSpecBackOnThePassphraseQuestionUnRegistersTheSeed (composer_fable_r0_flow_test.go), TestPassphraseFlow (gui_test.go), TestPassphrasePlateOfferReachableFromTheOrchestrator (s6b_passphrase_plate_test.go), TestRestoreDocReflectsARealCutPassphrasePlate (s6b_restore_doc_test.go), TestSeedEntryScreensNameTheirSlot (multisig_build_scrub_test.go), TestSingleSigAbortIsTheLastScreenOfTheProgram (singlesig_truth_test.go), TestSingleSigBareRunDoesNotCryWolf (singlesig_truth_test.go), TestSingleSigPassphraseRunTellsTheOperatorWhatIsMissing (singlesig_truth_test.go), TestSingleSigShowsThePlateCensusBeforeTheEngrave (singlesig_truth_test.go), TestSupplyPassphraseRunTellsTheOperatorWhatIsMissing (multisig_supply_passphrase_test.go), TestTheSeamPassphraseComesFromThePayloadWithoutTyping (sysw_cells_test.go)

#### NFC: no tag is read and no scan row or offer is drawn, so row indices shift (§4.2) — 23
TestComposerDoorOffersFromPayloadOnlyWhenThePayloadHasOne (composer_door_test.go), TestComposerKeyPathChoiceIsPlacedBeforeTheChunks (composer_unspendable_test.go), TestF437CardDoorsDoNotPromiseTyping (payload_door_label_test.go), TestF440BundleIncompleteModalDismissesOnBack (modal_back_test.go), TestF76ACorruptedChunkInThePayloadIsStillRefused (payload_door_walk_test.go), TestF76BundleCountsACompleteMd1CardFromThePayload (payload_door_walk_test.go), TestF76BundleCountsACompleteMk1CardFromThePayload (payload_door_walk_test.go), TestF76CompletePayloadNeverSeesTheIncompleteRefusal (payload_door_walk_test.go), TestF76IncompletePayloadGetsTheRepackAdvice (payload_door_walk_test.go), TestF76IncompletePayloadNamesBothRoutesOnAnNFCMachine (payload_door_walk_test.go), TestF76WalletPolicyCountsACompleteMd1CardFromThePayload (payload_door_walk_test.go), TestMultisigTakesItsFirstCardFromThePayload (sysw_cells_test.go), TestNFCScannerDoesNotSpinAtEOF (nfc_scan_test.go), TestNFCScannerStillDeliversATag (nfc_scan_test.go), TestRunSealedPayloadReentryAfterWipe (run_reentry_test.go), TestStopScannerAbandonsAReaderThatWillNotStop (nfc_scan_abandon_test.go), TestStopScannerJoinTimeoutArmAbandons (nfc_scan_abandon_test.go), TestSyswSeedPickerOffersScanWithoutAPayload (sysw_source_test.go), TestSyswSeedScanAcceptsAMnemonicFromATag (sysw_source_test.go), TestSyswSeedScanDeclineDoesNotAcceptTheSeed (sysw_source_test.go), TestSyswSeedScanRefusesANonSeedTag (sysw_source_test.go), TestWalkWalletPolicyFromAPackedDescriptorRecordToTheDescriptorScreen (wallet_policy_descriptor_walk_test.go), TestWalkWalletPolicyRendersARecordWithLeadingWhitespace (wallet_policy_descriptor_walk_test.go)

#### Verify: the readback is refused (§4.2) — 27
TestEveryFlowsRestoreDocumentSaysWhatItCheckedAndWhatItHolds (singlesig_truth_test.go), TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts (verify_status_ms1_clause_test.go), TestMultisigVerifyRecordsWhatItObserved (singlesig_truth_test.go), TestSingleSigVerifyCSIDNoteOnFailureLive (csid_warning_test.go), TestSingleSigVerifyCSIDNoteSilentOnCleanTwinLive (csid_warning_test.go), TestSingleSigVerifyFailedCopyConditionsOnPassphrase (singlesig_verify_failure_copy_test.go), TestSingleSigVerifyRecordsWhatItObserved (singlesig_truth_test.go), TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine (s6b_p9_failure_states_test.go), TestSupplyDuplicateSlotVerifiesItsOwnOutput (multisig_supply_dupslot_test.go), TestSupplyEngraveVerifiesItsOwnOutput (multisig_supply_multislot_test.go), TestVerifyBackAtPassphraseDoesNotSkipIt (multisig_verify_back_test.go), TestVerifyBackAtPassphraseKeepsTheSeed (multisig_verify_back_test.go), TestVerifyBuildShapeChecksEveryEngravedPlate (multisig_verify_flow_test.go), TestVerifyFullModeBackAtTheSecondMs1ReportsIncomplete (multisig_verify_report_test.go), TestVerifyFullModeBindsEachMs1ToItsOwnSeed (multisig_verify_report_test.go), TestVerifyFullModeTwoSeedsReportsTheFullSuccess (multisig_verify_report_test.go), TestVerifyIncompleteDoesNotCallAForeignPlateChecked (multisig_verify_report_test.go), TestVerifyIncompleteReportsWhatTheComparatorMatched (multisig_verify_report_test.go), TestVerifyOneSlotRunChecksTheONEPlateItEngraved (multisig_verify_flow_test.go), TestVerifyPassLineNamesCosignersOnlyWhereThereAreSome (singlesig_truth_test.go), TestVerifyRefusesAPartialReadbackOfAThreePlateBuild (multisig_supply_multislot_test.go), TestVerifyRefusesPlatesFromADifferentPolicy (multisig_verify_policy_test.go), TestVerifyReoffersOnAnUnaccountableReadback (multisig_verify_refusal_test.go), TestVerifyReportsIncompleteAfterAMidLoopRefusal (multisig_verify_policy_test.go), TestVerifyRetriesAfterACorrectableFirstSeed (multisig_verify_report_test.go), TestVerifyStillFailsWhenTheENGRAVEDPlateIsWrong (multisig_verify_flow_test.go), TestVerifyStillPassesItsOwnPolicy (multisig_verify_policy_test.go)

#### SLIP-39: a passphrase ends the recovery (§4.3) — 3
TestRecoverSLIP39MultiGroup (slip39_polish_test.go), TestRecoverSLIP39Passphrase (slip39_polish_test.go), TestSLIP39RecoveredSeedIsolatedFromBIP39Passphrase (slip39_polish_test.go)

I triaged each of these failures from its first failure message before adding the skip. Each one is the profile doing what §4.2/§4.3 says:

- landing on the passphrase notice;
- landing on the verify refusal;
- a row index or carousel position shifted by one;
- a tag never read.

None of the 93 exposed an unintended regression.

### Environment skips (3, every build)

These skip in every build and are named in the script:

- `TestFableTwoKeylessPathsAgreeWithTheHostOracle` (needs `md` on PATH)
- `TestCaptureTransactionJourney` (needs TX_JOURNEY_OUT)
- `TestIdleTimerUnderSH2ShapedEventLoop` (needs SH2_REALCLOCK)

## Deviations and why

1. **Unkeyed scan offers key on `scanOffered()`, not `nfcAvailable()`.**
   - The plan says these move "onto nfcAvailable()". On a reader-less platform that would change the default build: gui's testPlatform has no reader unless a test sets one, and these screens drew their offer regardless. That would break D1's "default unchanged".
   - `scanOffered()` is false under the profile and true otherwise. The Refugium behaviour is identical: no offer anywhere.
2. **Both verify flows refuse up front** under the profile, before the seed is retyped.
   - There is no NFC readback, and §7.4 forbids taking the readback from the payload.
   - singlesig writes no verify bit, so the restore document states the weakest true thing. multisig returns `verifyRefused`.
3. **SLIP-39 keeps its passphrase question**, unlike the BIP-39 sites, which became notices. A "yes" refuses the recovery.
   - Skipping the question would silently recover a different valid seed.
4. **`syswChoose` with the scan alternative auto-takes the payload** under the profile, without drawing a one-row picker (the §13 D9 rule). Back on the following screen still leaves.
5. **`nfcDev` moved into `cmd/controller/nfc_default.go`**, with an empty struct twin in `nfc_refugium.go`. This was not in the plan. Without it, `Device.Read` and `write` still linked into the Refugium ELF through the unused adapter type; after the move, no `st25r3916.Device` method has a symbol there.
6. **ELF check additions.** The check has a reverse control (the profile notices) that the plan did not ask for, so that a default ELF missing markers for some other reason cannot pass as the Refugium build.
7. **Tagged runner skip handling.** The runner accepts profile-named skips and 3 named environment skips, in addition to PASS. The plan's "PASS count must reach the expected number" is implemented as PASS + profile skips + environment skips == listed. Any other SKIP fails.
8. **Coarse skips: `TestRefugiumPairsAreWellFormed` and subtests.**
   - The skip helper skips whole tests. Several tests fail only in one subtest under the tag, so their other subtests do not run tagged:
     - `TestBuildFlowScrubsEverySeedOnEveryExit`
     - `TestRunSealedPayloadReentryAfterWipe`
     - `TestF437CardDoorsDoNotPromiseTyping`
     - `TestMultisigTakesItsFirstCardFromThePayload`
     - `TestEveryFlowsRestoreDocumentSaysWhatItCheckedAndWhatItHolds`
     - `TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts`
     - `TestWipeZeroesEveryPinnedBufferAtRunLevel`
     - `TestMultisigVerifyRecordsWhatItObserved`
     - `TestSingleSigVerifyFailedCopyConditionsOnPassphrase`
     - `TestComposerDoorOffersFromPayloadOnlyWhenThePayloadHasOne`
     - `TestBothEngraveFlowsDriveTheRetryLoop`
   - Separately, `TestRefugiumPairsAreWellFormed`'s minimum pair count is 6, the number this phase adds. The earlier draft had 7.

## Open questions

1. **Composer door with one row.** With no consumable policy in the payload, the Refugium composer door draws one row, "Build a new policy", under its explanatory lead. `TestComposerKeyPathChoiceIsPlacedBeforeTheChunks` trips on this.
   - I kept the row, because the lead carries information. D9's "no one-row picker" rule could argue for skipping straight to the template.
   - This needs Brian's call.
2. **Single-card md1/mk1 gathers are a dead end.** Under the profile they say "This build takes chunks from the payload only." Their loop only ever receives chunks from scans, so after the first chunk the only way out is Back.
   - Should these flows refuse up front, like verify does, rather than open a gather that cannot advance?
3. **The NFC fault path is untested on hardware.** It is unit-tested through an optional-interface test platform. On the device, `NFCFault` was devirtualised: it has no symbol, but the fault text is linked. A device test needs a forced I2C failure.
4. **What the refugium-attached arm proves.** It shows gui reads nothing even when the platform hands out a reader. But `ctx.nfcReader()` short-circuits before `Platform.NFCReader()`, so the walk does not exercise `startScanner`'s own nil-ing. That second gate is pinned by `TestRefugiumStartScannerReadsNothing`, a unit test.
5. **CI is unverified.** The new steps have not run on GitHub:
   - The nix-devshell TinyGo honouring `-tags refugium` is assumed. I built with the 0.41.1 release tarball, not nix.
   - `actions/upload-artifact@v4` is a new action dependency.
6. **Mandatory post-implementation review is still owed.** The policy in CLAUDE.md calls for an independent adversarial execution review over the whole diff. I am the implementing agent, so I did not do it and did not substitute a self-review for it. The caller should dispatch it.

---

## Fold of the execution review (`refugium-F7-exec-review.md`, 0C/2I/7M/5N)

### Commits (on top of `c05df0e`, local only, not pushed)

| SHA | Contents |
|---|---|
| `9295013` | gui: I-1, M-1, M-2, M-4, M-6, N-1..N-3 (production code plus `refugium_profile_test.go`) |
| `0ada371` | controller: M-3 |
| `a45d45b` | emu: M-5 |
| `c85294c` | gui tests: I-2 (profile-aware drivers, subtest-level skips) |
| `e70a941` | merge of `origin/main` (`d156a3e`, F5) |

### Per finding

- **I-1.** The free-text gate (`containsMS1String` / `noMS1QRText`, `gui/ms1_qr_gate.go`) now looks for `ms1` followed by 16 or more bech32 characters anywhere in the text. Before matching, it lowercases the text and strips whitespace (`unicode.IsSpace`), `-` and `,`. All three `freetext_flow.go` sites use it. The bundle and codex32 gates are unchanged.
  - Test: `TestRefugiumFreeTextSinkDropsTheQRForAnEmbeddedMS1` drives `ftBuildPlate` directly and captures the plate through `freetextPlateHook`.
  - Inputs that must lose the QR: the bare vector, "Share A: …", "(…)", "1. …", a leading quote, a leading space, upper case, dashed and space-grouped.
  - Positive controls that keep the QR: "HELLO WORLD", "see the ms1 card" and "ms1 plates are text only".
  - NBSP and thin space are checked through the predicate only, because the plate font has no NBSP glyph and `ftBuildPlate` panics with "unsupported rune".
  - **Unverified side observation:** a payload text record containing NBSP may reach that same panic in Engrave Text in either build. This is pre-existing, and I did not chase it.
- **I-2.** The test drivers now follow the build profile:
  - `ppQuestion` is the passphrase question in the default build and the notice under the profile. Button3 means Skip on the question and acknowledge on the notice, and both take the no-passphrase branch.
  - `carouselRights(p)` and `shownTitles(titles)` (`gui/profile_skip_test.go`) navigate the carousel by program.
  - `addCarouselSteps` skips the hidden program's title.

  These now **run under the tag**:
  - wipe residency (both vectors);
  - sealed re-entry after wipe (all 5 subtests);
  - scrub-on-exit (gate FAIL, EXPERIMENTAL and ctx.Done; only "Back at the passphrase prompt" skips);
  - both payload chains (seed plate, md1/mk1 four plates);
  - build-from-payload-seed and its duplicate refusal;
  - `TestBothEngraveFlowsDriveTheRetryLoop`, both supply and build;
  - single-sig abort and census;
  - Bip85 Back;
  - the Fable Back-on-question test;
  - BareRunDoesNotCryWolf;
  - BuildAbort;
  - BuildWalkTypedSeed;
  - `TestMK1GatherFlowBackNoReader`;
  - 14 of the 15 carousel tests.

  I also found the remaining whole-test skips by a probe that disabled the skip and ran each skipped test alone (85 tests, 24-way). Where only some subtests broke, the skip moved down into those subtests:
  - `EveryFlowsRestoreDocument` (single-sig and multisig-supply skip; multisig-build runs);
  - `MultisigVerifyRecordsWhatItObserved` (comparator-disagreed skips; refused-before-read runs);
  - `MultisigTakesItsFirstCardFromThePayload` (supplied policy skips; built policy runs).
- **M-1.** On the notice, Back and ctx.Done return `(0,false)`, and the acknowledgement returns `(0,true)` (`passphraseOffNoticeFlow`).
  - I checked the 8 call sites. `!ok` does exactly what the question's own Back does in the default build, and no site sets a passphrase on `!ok`:
    - bip85, singlesig and multisig verify step back to the seed or script step;
    - `multisig_build_slots` discards the seed it just registered and returns false;
    - derive_xpub, gui.go's backup, multisig supply and singlesig verify (`ok && sel == 1`) continue with no passphrase.
  - Tests:
    - `TestRefugiumPassphrasePromptIsANotice` (both buttons);
    - `TestRefugiumPassphraseNoticeEndedByDoneIsNotOK`;
    - `TestRefugiumSeedPassphraseStepKeepsItsBack` (Back registers 0 seeds; acknowledge registers 1).
- **M-2.** Under the profile, an md1/mk1 single-card gather that the payload does not complete shows one `showError` screen and returns false. The screen reads "Captured N of M. This build takes the remaining chunks from the payload only; pack the full set." There is no Back-only gather loop. The default line is back to "Scan the next chunk.". Test: `TestRefugiumIncompleteChunkGathersRefuse` (md1 and mk1).
- **M-3.** `p.initNFC(st25r3916.New(mi2c, NFC_INT))` now runs immediately after `newMultiplexI2C`, before USB-PD and the rest of `Init`.
- **M-4.** `syswFirstPassphraseRecord` refuses any record starting with `sysw.PassPrefix`, whatever its body. Test: `TestRefugiumAnyPassPrefixedRecordIsFound`.
- **M-5.** I fixed the comments and added a measurement:
  - The emulator counts `NFCReader()` asks, exposed as `shNFC.readerAsks`.
  - The walk records `readerAsks` and asserts 0 on both Refugium arms and at least 1 on default.
  - The comments now say the forced arm proves gui never *asks*. `startScanner`'s nil-ing is pinned by `TestRefugiumStartScannerReadsNothing`, not by the walk.
  - Host test: `TestAsksCountsReaderRequests`.
- **M-6.**
  - `TestRefugiumFreeTextForcedOffQRIsSaid` walks Engrave Text to its last QR check and asserts the forced-off notice.
  - `unlockCodex32Plan` is split out of `unlockEngraveCodex32`. `TestRefugiumUnlockCodex32PlateIsTextOnly` asserts that the sealed-unlock codex32 plate has no QR. No seed-derived hook ships in the firmware.
- **Nits.**
  - N-1 and N-2: the two wrong comments are fixed.
  - N-3: `refugium_build_test.go` now says what the literal scan cannot see (concatenation, method values); the ELF check covers those.
  - **Q1:** I kept the one-row composer door.
  - **N-5 ("Verify now" offer): left as is.** It is not trivially hideable. The offer and its Skip/refusal outcome feed the verify record and the restore document's status line in three flows (`singlesig.go:284`, `multisig.go:341`, `multisig_build.go:554`). Removing it would change those documents' lines, not just one screen. The refusal stays informative and records nothing.

### Merge

`git merge origin/main` (`d156a3e`) had two conflicts:
- `backup/backup.go`: I kept both F7's `EngraveSeedStringTextOnly` and F5's `EngraveSeedStringSeedQR`.
- `gui/preview.go`: I kept F5's `ms1seedqr` entry. The proof entries stay in `preview_proofs_default.go`.

One more fix went into the merge commit. F5's `preview_ms1seedqr_test.go` called `proofParams()`, which lives in a `!refugium` file, so the tagged package did not compile. The test now calls `newPlatform().EngraverParams()`, which is the same value.

F5 adds no flow caller of the SeedQR layout; it is used only by plateview's preview.

### Results (post-merge tree `e70a941`; each run once, captured to a file)

| Command | Result |
|---|---|
| `gui-shard-test.sh ./gui 24` (default) | **1412 tests**, all 24 shards ok, wall 91 s (1411 + F5's `TestPreviewMS1SeedQR`) |
| `scripts/refugium-tagged-test.sh 24` | **1340 listed: 1276 PASS, 61 SKIP (profile), 3 SKIP (environment), 0 FAIL** (was 1237 / 93 / 3 of 1333) |
| `CGO_ENABLED=0 go test $(go list ./... \| grep -v gui$)` | 56 packages ok |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` (with and without `-tags refugium`) | clean |
| oraclelive compile (`./oracle ./gui ./sysw ./cmd/emu`; `oraclelive,refugium ./gui`) | rc=0 |
| `./scripts/test-32bit.sh` | rc=0 |
| `gofmt -l .` | the same five-file baseline |
| TinyGo `pico-plus2` builds | default 14,444,860 B; refugium 13,950,256 B |
| `scripts/refugium-elf-check.sh` | RESULT ok: 11 markers default-only, 3 profile notices refugium-only, no st25r3916 Device method in the Refugium ELF |
| emu walk default | ok:true, presented 24, delivered 23, **readerAsks 2** |
| emu walk refugium | ok:true, presented 22, delivered 0, **readerAsks 0** |
| emu walk refugium-attached | ok:true, presented 22, delivered 0, **readerAsks 0** |

### Remaining tagged skips (61 whole tests + 5 subtests), by reason

Each one drives a behaviour the profile removes on purpose: the reader, the scan rows, the verify readback, the passphrase entry, or SLIP-39 passphrase recovery. The single carousel skip pins the hidden program's own position.
- **Carousel** (1): `TestPassphraseProgramReachable` (gui_test.go)
- **NFC** (22): `TestComposerDoorOffersFromPayloadOnlyWhenThePayloadHasOne` (composer_door_test.go), `TestComposerKeyPathChoiceIsPlacedBeforeTheChunks` (composer_unspendable_test.go), `TestF440BundleIncompleteModalDismissesOnBack` (modal_back_test.go), `TestStopScannerAbandonsAReaderThatWillNotStop` (nfc_scan_abandon_test.go), `TestStopScannerJoinTimeoutArmAbandons` (nfc_scan_abandon_test.go), `TestNFCScannerDoesNotSpinAtEOF` (nfc_scan_test.go), `TestNFCScannerStillDeliversATag` (nfc_scan_test.go), `TestF437CardDoorsDoNotPromiseTyping` (payload_door_label_test.go), `TestF76WalletPolicyCountsACompleteMd1CardFromThePayload` (payload_door_walk_test.go), `TestF76BundleCountsACompleteMd1CardFromThePayload` (payload_door_walk_test.go), `TestF76BundleCountsACompleteMk1CardFromThePayload` (payload_door_walk_test.go), `TestF76IncompletePayloadGetsTheRepackAdvice` (payload_door_walk_test.go), `TestF76IncompletePayloadNamesBothRoutesOnAnNFCMachine` (payload_door_walk_test.go), `TestF76CompletePayloadNeverSeesTheIncompleteRefusal` (payload_door_walk_test.go), `TestF76ACorruptedChunkInThePayloadIsStillRefused` (payload_door_walk_test.go), `TestMultisigTakesItsFirstCardFromThePayload/supplied policy` (sysw_cells_test.go), `TestSyswSeedPickerOffersScanWithoutAPayload` (sysw_source_test.go), `TestSyswSeedScanAcceptsAMnemonicFromATag` (sysw_source_test.go), `TestSyswSeedScanDeclineDoesNotAcceptTheSeed` (sysw_source_test.go), `TestSyswSeedScanRefusesANonSeedTag` (sysw_source_test.go), `TestWalkWalletPolicyFromAPackedDescriptorRecordToTheDescriptorScreen` (wallet_policy_descriptor_walk_test.go), `TestWalkWalletPolicyRendersARecordWithLeadingWhitespace` (wallet_policy_descriptor_walk_test.go)
- **Verify** (28): `TestSingleSigVerifyCSIDNoteOnFailureLive` (csid_warning_test.go), `TestSingleSigVerifyCSIDNoteSilentOnCleanTwinLive` (csid_warning_test.go), `TestSupplyDuplicateSlotVerifiesItsOwnOutput` (multisig_supply_dupslot_test.go), `TestVerifyRefusesAPartialReadbackOfAThreePlateBuild` (multisig_supply_multislot_test.go), `TestSupplyEngraveVerifiesItsOwnOutput` (multisig_supply_multislot_test.go), `TestVerifyBackAtPassphraseDoesNotSkipIt` (multisig_verify_back_test.go), `TestVerifyBackAtPassphraseKeepsTheSeed` (multisig_verify_back_test.go), `TestVerifyOneSlotRunChecksTheONEPlateItEngraved` (multisig_verify_flow_test.go), `TestVerifyStillFailsWhenTheENGRAVEDPlateIsWrong` (multisig_verify_flow_test.go), `TestVerifyBuildShapeChecksEveryEngravedPlate` (multisig_verify_flow_test.go), `TestVerifyRefusesPlatesFromADifferentPolicy` (multisig_verify_policy_test.go), `TestVerifyStillPassesItsOwnPolicy` (multisig_verify_policy_test.go), `TestVerifyReportsIncompleteAfterAMidLoopRefusal` (multisig_verify_policy_test.go), `TestVerifyReoffersOnAnUnaccountableReadback` (multisig_verify_refusal_test.go), `TestVerifyIncompleteDoesNotCallAForeignPlateChecked` (multisig_verify_report_test.go), `TestVerifyIncompleteReportsWhatTheComparatorMatched` (multisig_verify_report_test.go), `TestVerifyRetriesAfterACorrectableFirstSeed` (multisig_verify_report_test.go), `TestVerifyFullModeTwoSeedsReportsTheFullSuccess` (multisig_verify_report_test.go), `TestVerifyFullModeBackAtTheSecondMs1ReportsIncomplete` (multisig_verify_report_test.go), `TestVerifyFullModeBindsEachMs1ToItsOwnSeed` (multisig_verify_report_test.go), `TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine` (s6b_p9_failure_states_test.go), `TestEveryFlowsRestoreDocumentSaysWhatItCheckedAndWhatItHolds/single-sig` (singlesig_truth_test.go), `TestEveryFlowsRestoreDocumentSaysWhatItCheckedAndWhatItHolds/multisig-supply` (singlesig_truth_test.go), `TestVerifyPassLineNamesCosignersOnlyWhereThereAreSome` (singlesig_truth_test.go), `TestSingleSigVerifyRecordsWhatItObserved` (singlesig_truth_test.go), `TestMultisigVerifyRecordsWhatItObserved/comparator-disagreed` (singlesig_truth_test.go), `TestSingleSigVerifyFailedCopyConditionsOnPassphrase` (singlesig_verify_failure_copy_test.go), `TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts` (verify_status_ms1_clause_test.go)
- **Passphrase** (12): `TestChainPassphraseFromAMePackedPayloadToAPasswordPlate` (chain_class_walk_test.go), `TestComposerConsentDoesNotClaimLianaImportsASameSeedWallet` (composer_f671_test.go), `TestFableSpecBackOnThePassphraseKeyboardIsADecline` (composer_fable_r0_flow_test.go), `TestPassphraseFlow` (gui_test.go), `TestBuildFlowScrubsEverySeedOnEveryExit/Back at the passphrase prompt` (multisig_build_scrub_test.go), `TestSeedEntryScreensNameTheirSlot` (multisig_build_scrub_test.go), `TestSupplyPassphraseRunTellsTheOperatorWhatIsMissing` (multisig_supply_passphrase_test.go), `TestPassphrasePlateOfferReachableFromTheOrchestrator` (s6b_passphrase_plate_test.go), `TestRestoreDocReflectsARealCutPassphrasePlate` (s6b_restore_doc_test.go), `TestSingleSigPassphraseRunTellsTheOperatorWhatIsMissing` (singlesig_truth_test.go), `TestTheSeamPassphraseComesFromThePayloadWithoutTyping` (sysw_cells_test.go), `TestDecliningTheSeamPassphraseOfferReachesTheKeyboard` (sysw_cells_test.go)
- **SLIP39** (3): `TestRecoverSLIP39Passphrase` (slip39_polish_test.go), `TestRecoverSLIP39MultiGroup` (slip39_polish_test.go), `TestSLIP39RecoveredSeedIsolatedFromBIP39Passphrase` (slip39_polish_test.go)

### Fold of the re-check (`refugium-F7-fold-recheck.md`, 0C/1I/1M/2N)

| SHA | Contents |
|---|---|
| `d8218df` | m-1 (the free-text ms1 gate drops every rune that is not a letter or digit), n-1 (comment) |
| `b9271ad` | I-2r (payload-door, descriptor and composer walks run under the tag), n-2 (end-to-end scrub exit) |

- **I-2r.** Two new driver helpers in `gui/profile_skip_test.go`:
  - `takePayloadOffer` waits for the payload offer and presses FROM PAYLOAD in the default build. Under the profile it does nothing, because `syswChoose` takes the payload without drawing the offer.
  - `composerDoorRow(t, ctx, label)` picks a composer-door row by its label. It reads the row list from `composerDoorRows`, which I extracted from `composerDoorFlow` (a pure refactor: the same rows in the same order).

  These now **run under the tag**:
  - the six F76 door tests, including `TestF76ACorruptedChunkInThePayloadIsStillRefused`;
  - F440;
  - both Descriptor walks;
  - `TestComposerKeyPathChoiceIsPlacedBeforeTheChunks`;
  - `TestComposerConsentDoesNotClaimLianaImportsASameSeedWallet` (its driver now taps the notice's forward target where the question had a Skip row);
  - `TestComposerDoorOffersFromPayloadOnlyWhenThePayloadHasOne`, which now asserts that "Scan cards" is offered exactly where a scan is, so under the profile it must be absent.

  Only `TestF76IncompletePayloadNamesBothRoutesOnAnNFCMachine` still steps aside; its subject is the NFC arm.
- **m-1.** `containsMS1String` now drops every rune that is neither a letter nor a digit before it matches.
  - The sink test gains `.`, `/`, `:`, `_`, `|` and `;` groupings, plus a punctuation-heavy positive control: "ms1: a share. keep it apart / never photograph it". All the earlier controls keep their QR.
  - One known limit is documented in the predicate's comment. The re-check's "label per line" example (`Line 1: ms10testsxxxx\nLine 2: …`) is still not caught. Once punctuation is dropped, the word "Line" sits inside the data run, and its `i` is not a bech32 character, so the run ends at 11 characters. The re-check's own proposed fix would not catch this case either. I left it out of the test rather than assert a gate the predicate does not provide.
- **n-1.** A comment now explains why `return !ctx.Done` is defensive.
- **n-2.** Done. The new subtest "Back on the passphrase step" in `TestBuildFlowScrubsEverySeedOnEveryExit` presses Back on the question in the default build and on the notice under the profile. It then backs out of the flow and asserts the seed was scrubbed. It runs in both builds.
  - Mutation check: I left `discardLast`'s word-zeroing out, and the subtest failed ("seed 0 word 11 is still 3"). The mutant is reverted.

**Results (tree `b9271ad`; each run once, captured to a file)**

| Command | Result |
|---|---|
| `scripts/refugium-tagged-test.sh 24` | **1340 listed: 1288 PASS, 49 SKIP (profile), 3 SKIP (environment), 0 FAIL** (was 1276 / 61 / 3) |
| `gui-shard-test.sh ./gui 24` (default) | **1412 tests**, all 24 shards ok |
| `go test ./backup`, with and without `-tags refugium` | ok, ok |

**Remaining tagged skips after the re-check fold** (49 whole tests + 5 subtests):

- **Carousel** (1): `TestPassphraseProgramReachable`
- **NFC** (11): `TestStopScannerAbandonsAReaderThatWillNotStop`, `TestStopScannerJoinTimeoutArmAbandons`, `TestNFCScannerDoesNotSpinAtEOF`, `TestNFCScannerStillDeliversATag`, `TestF437CardDoorsDoNotPromiseTyping`, `TestF76IncompletePayloadNamesBothRoutesOnAnNFCMachine`, `TestMultisigTakesItsFirstCardFromThePayload/supplied policy`, `TestSyswSeedPickerOffersScanWithoutAPayload`, `TestSyswSeedScanAcceptsAMnemonicFromATag`, `TestSyswSeedScanDeclineDoesNotAcceptTheSeed`, `TestSyswSeedScanRefusesANonSeedTag`
- **Verify** (28): `TestSingleSigVerifyCSIDNoteOnFailureLive`, `TestSingleSigVerifyCSIDNoteSilentOnCleanTwinLive`, `TestSupplyDuplicateSlotVerifiesItsOwnOutput`, `TestVerifyRefusesAPartialReadbackOfAThreePlateBuild`, `TestSupplyEngraveVerifiesItsOwnOutput`, `TestVerifyBackAtPassphraseDoesNotSkipIt`, `TestVerifyBackAtPassphraseKeepsTheSeed`, `TestVerifyOneSlotRunChecksTheONEPlateItEngraved`, `TestVerifyStillFailsWhenTheENGRAVEDPlateIsWrong`, `TestVerifyBuildShapeChecksEveryEngravedPlate`, `TestVerifyRefusesPlatesFromADifferentPolicy`, `TestVerifyStillPassesItsOwnPolicy`, `TestVerifyReportsIncompleteAfterAMidLoopRefusal`, `TestVerifyReoffersOnAnUnaccountableReadback`, `TestVerifyIncompleteDoesNotCallAForeignPlateChecked`, `TestVerifyIncompleteReportsWhatTheComparatorMatched`, `TestVerifyRetriesAfterACorrectableFirstSeed`, `TestVerifyFullModeTwoSeedsReportsTheFullSuccess`, `TestVerifyFullModeBackAtTheSecondMs1ReportsIncomplete`, `TestVerifyFullModeBindsEachMs1ToItsOwnSeed`, `TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine`, `TestEveryFlowsRestoreDocumentSaysWhatItCheckedAndWhatItHolds/single-sig`, `TestEveryFlowsRestoreDocumentSaysWhatItCheckedAndWhatItHolds/multisig-supply`, `TestVerifyPassLineNamesCosignersOnlyWhereThereAreSome`, `TestSingleSigVerifyRecordsWhatItObserved`, `TestMultisigVerifyRecordsWhatItObserved/comparator-disagreed`, `TestSingleSigVerifyFailedCopyConditionsOnPassphrase`, `TestMS1ClauseIsCountFreeAcrossSeedAndLegCounts`
- **Passphrase** (11): `TestChainPassphraseFromAMePackedPayloadToAPasswordPlate`, `TestFableSpecBackOnThePassphraseKeyboardIsADecline`, `TestPassphraseFlow`, `TestBuildFlowScrubsEverySeedOnEveryExit/Back at the passphrase prompt`, `TestSeedEntryScreensNameTheirSlot`, `TestSupplyPassphraseRunTellsTheOperatorWhatIsMissing`, `TestPassphrasePlateOfferReachableFromTheOrchestrator`, `TestRestoreDocReflectsARealCutPassphrasePlate`, `TestSingleSigPassphraseRunTellsTheOperatorWhatIsMissing`, `TestTheSeamPassphraseComesFromThePayloadWithoutTyping`, `TestDecliningTheSeamPassphraseOfferReachesTheKeyboard`
- **SLIP39** (3): `TestRecoverSLIP39Passphrase`, `TestRecoverSLIP39MultiGroup`, `TestSLIP39RecoveredSeedIsolatedFromBIP39Passphrase`

(54 sites)

### Engrave Text has no QR under the profile (coordinator decision; closes m-1)

Commit `e84770a`.

Under the Refugium profile, Engrave Text now cuts no QR for any text. Before this commit it dropped the QR only when a predicate recognised an ms1 string, and a predicate like that can always be dodged by formatting.

**Behaviour under the profile:**
- The QR step offers one answer, "No QR", with the lead "This build engraves text without a QR.". This holds even when a prior opt-in is carried in.
- `ftBuildPlate` never cuts a QR, even when its caller asks for one.
- The forced-off notice after the text step is reworded to "This build engraves text without a QR, so the plate carries no QR.". It is now defensive only, because the QR step can no longer produce an opt-in.

**Code:**
- `noFreeTextQR()` (= `refugiumProfile`) replaces the free-text predicate.
- `containsMS1String` and `noMS1QRText` are removed.
- `isMS1String` / `noMS1QR` still gate the bundle and codex32 producers. They and their tests are unchanged.

**Tests (red first):**
- Three tagged tests check that no text ever gets a QR under the profile:
  - `TestRefugiumFreeTextQRStepOffersNoQR` checks the QR step.
  - `TestRefugiumFreeTextSinkNeverCutsAQR` checks the sink.
  - `TestRefugiumFreeTextWalkEngravesNoQR` walks end to end to a confirm screen that reads "QR: no".
- Their text list covers:
  - every ms1 disguise from I-1 and m-1, including the per-line-label example `Line 1: ms10testsxxxx\nLine 2: …`;
  - plain text ("HELLO WORLD", "see the ms1 card").
- The default-build controls are in the new `!refugium` file `gui/freetext_qr_build_test.go`, as `TestFreeTextQRStaysOnInTheDefaultBuild`. The default build still cuts the requested QR for the same texts, ms1 included, and its QR step still offers "Add QR".
- Default Engrave Text tests whose subject is the QR now skip under the tag. They use a sixth named reason, `refugiumSkipFreeTextQR`, and the runner's header comment now says "six".
  - Whole tests: `TestFTRefusalOffersTheQRRatherThanDroppingIt`, `TestFTQREncodesTheTextOnly`, `TestFTBuildPlateEncodesOnce`, `TestFTQRChoiceLabelsBindToMeaning`, `TestFTPlateIsWhatWasApproved`.
  - Subtests: `TestFTBuiltPlateIsTheFittedComposition/qr=true` (the loop is now subtests) and `TestFTConfirmCarriesTheSafetyCopy/with a QR`.
- These keep running under the tag:
  - `TestFTBackPreservesEveryValue` runs, with "No QR" as the value Back must keep.
  - `TestEngraveTextProgramSelectable` runs, waiting for the profile's own QR lead.

**Results** (tree `e84770a`; each run once, output captured to a file):

| Command | Result |
|---|---|
| `scripts/refugium-tagged-test.sh 24` | **1340 listed: 1283 PASS, 54 SKIP (profile), 3 SKIP (environment), 0 FAIL** |
| `gui-shard-test.sh ./gui 24` (default) | **1413 tests**, all ok (+1: `TestFreeTextQRStaysOnInTheDefaultBuild`) |
| `go test ./backup`, with and without `-tags refugium` | ok, ok |
| `gofmt -l .` | the same five-file baseline |

**Remaining tagged skips:** 54 whole tests + 7 subtests. Compared with the re-check list above:
- **Added** (`FreeTextQR`, 5 whole tests + 2 subtests): the tests named in the Tests list above.
- **Unchanged:** Carousel 1, NFC 11, Verify 28, Passphrase 11, SLIP39 3.

The TinyGo ELFs and the emulator walks were not re-run for this change; it was not asked.
