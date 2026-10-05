# Refugium F7: independent adversarial execution review

- **Reviewer:** independent execution-review subagent (Claude Opus 5.5), not the implementer
- **Date:** 2026-10-05
- **Diff:** `be00ef8..c05df0e` (branch `claude/project-thread-59q2a6-f7`, 6 commits, 116 files)
- **Inputs:** plan §2 (D1, D3, D4) and §4; R0 reports (rounds 1–3); the implementer's report; UI spec §4.4, §4.6.2 and §14 item 8
- **Counts:** **0 Critical, 2 Important, 7 Minor, 5 Nit**

Verdict: the profile does what D1, D3 and D4 require on every path I could find. I found no path in the Refugium build where NFC is read, a BIP-39 passphrase is taken, or a bundle or codex32 ms1 plate gets a QR. Two Important items should be fixed before merge:

- **I-1:** the Engrave Text ms1 predicate only matches a prefix.
- **I-2:** the tagged suite has no coverage of the Refugium build's wipe and scrub flows.

## What I ran (reproduced independently, outputs captured once to files)

All runs used GOTOOLCHAIN=go1.26.7 on a 4-core machine (not the 24-core box), in a scratch worktree at c05df0e.

| Run | Result |
|---|---|
| `gui-shard-test.sh ./gui 4` (default) | 1411 tests, partition exhaustive, all shards ok |
| `go test -list` default, base vs head | Head adds exactly 8 tests (3 characterization, 5 gate). None removed. |
| `scripts/refugium-tagged-test.sh 4` | 1333 listed: 1237 PASS, 93 profile SKIP, 3 env SKIP, 0 FAIL. RESULT ok. |
| TinyGo 0.41.1, both ELFs, `refugium-elf-check.sh` | RESULT ok. All 11 markers are 1/0 and all 3 reverse controls are 0/1. Every st25r3916 `Device` method is gone from the Refugium ELF. |
| `emu-refugium-walk.sh` refugium / refugium-attached / default | All ok:true. Presented/delivered: 22/0, 22/0 and 24/23. |
| Moved tests (`freetext_proof_gear_test.go`) | Byte-identical to their originals |

### Mutation tests (all in a separate worktree, since removed)

| Mutation | Expected | Got |
|---|---|---|
| `"lock-boot: %v"` literal put into `debugcmd_refugium.go` | source gate fails | **killed**, by both `TestRefugiumFileSetsCarryNoForbiddenLiteralOrCall` and `TestRefugiumPairsAreWellFormed` |
| `otp.AddBootKey(nil)` call added to `lockboot_refugium.go` | source gate fails | **killed**, by both tests |
| `verify_address.go` calls `ctx.Platform.NFCReader()` directly | one-place test fails | **killed**, by `TestNFCIsAskedForInOnePlace` |
| `startScanner`'s profile nil-ing removed | tagged test fails | **killed**, by `TestRefugiumStartScannerReadsNothing` |
| bip85 site reverted to `ppChoice.Choose` | prompt pin fails | **killed**, by `TestEveryBIP39PassphrasePromptIsAsked` |
| `validateMdmkStrings` ms1 gate disabled | tagged test fails | **killed**, by `TestRefugiumMS1CardOffersTextOnly` |
| `qaEngraveFlow` dispatch un-gated, Refugium ELF rebuilt | ELF check fails | **killed**: `qaEngraveFlow` is PRESENT in the Refugium ELF, RESULT FAIL, rc=1 |
| ELF check given two default ELFs | fails | **killed** (rc=1) |
| `"lock-"+"boot: %v"` (constant concatenation) | source gate | **survives** (N-3). The ELF check still catches it, because Go folds the constant. |

## Critical

None.

## Important

### I-1. The ms1-QR predicate only checks a prefix, so Engrave Text still cuts a QR containing an ms1 secret

**Evidence:**

- `gui/ms1_qr_gate.go:25-38`: `isMS1String` looks only at the first three non-separator characters.
- `gui/freetext_flow.go:530` and `:1458` gate on it.

I demonstrated this under `-tags refugium` with a scratch test, since deleted. Each of these returns `noMS1QR(...) == false`, so `ftBuildPlate` keeps `useQR`:

- `"Share A: ms10tests…"`
- `"(ms10tests…)"`
- `"1. ms10tests…"`
- `"\"ms10tests…"`
- `" ms10tests…"`

`qrFor(CompositionText(blocks), useQR)` (`backup/fit.go:283`) then encodes the whole text, ms1 string included.

The UI spec (§4.4 "never a QR of the ms1 string"; §4.6.2 "An ms1 plate's QR is the seed in one photo") is about the secret being machine-readable. A label in front of it changes nothing.

- **Plan conformance:** the plan's own wording ("the string's HRP is ms … after separators are stripped") produced this behaviour. So this is a plan-level gap the implementation inherited, not an implementer error.
- **Bundle and codex32 producers are not affected.** Their strings are verbatim codec strings, so for them the prefix test is exactly "the HRP is ms".

**Fix:** for the free-text gate (`ftQRChoiceFlow`, `ftBuildPlate` and the `engraveTextFlowFrom` notice), apply a containment predicate:

- Use `noMS1QR` if, after stripping separators, the text contains `ms1` followed by bech32 characters anywhere, case-insensitive.
- Keep the "stricter than `IsMS1Shaped`" philosophy: a threshold of 16 or more bech32 characters is enough. Erring wide only costs a QR on text that merely mentions "ms1…".
- Add a tagged test with the five cases above. It should call `ftBuildPlate` directly, i.e. at the last sink, not only the choice step (see M-6).

### I-2. The Refugium build's wipe, scrub and payload-chain flows have no tagged coverage: the skips are mechanical, not semantic

**Evidence:** the 93 `skipUnderRefugium` skips include the suite's seed-safety tests:

- `TestWipeZeroesEveryPinnedBufferAtRunLevel` (`wipe_inventory_audit_test.go:201`, filed under "carousel")
- `TestRunSealedPayloadReentryAfterWipe` (`run_reentry_test.go:494`, filed under "NFC")
- `TestBuildFlowScrubsEverySeedOnEveryExit` (`multisig_build_scrub_test.go:109`)
- `TestBuildTakesTheSelfSeedFromThePayload`
- `TestChainMnemonicFromAMePackedPayloadToASeedPlate`
- `TestChainMdMkFromTheEmulatorsOwnPayloadToFourPlates`
- `TestBothEngraveFlowsDriveTheRetryLoop`
- `TestSingleSigAbortIsTheLastScreenOfTheProgram`
- `TestSingleSigShowsThePlateCensusBeforeTheEngrave`

I removed the skips from 11 of these and ran them under the tag. **Every failure was the test's own driver:**

- `"driver stalled at step "s1: right-tap from BIP-39 Password""`, because the carousel step count is off by one.
- `"the passphrase prompt was not reached; got "This build takes no BIP-39 passphrase…""`.
- `"the menu entry never appeared"`, also a carousel position.
- `"the readback never reached the gatherer's tally"` (verify; that skip is legitimate).

None showed a product regression, so the implementer's triage holds. The cost is that the Refugium build is the one that will hold seeds, and the tagged suite now runs none of these:

- the run-level wipe residency assertions;
- sealed re-entry after a wipe;
- scrubbing on the gate-FAIL, EXPERIMENTAL-warning and `ctx.Done` exits;
- the payload → seed-plate chain;
- the build-from-payload-seed walk.

Those scrub exits come *after* the passphrase step. They exist in the Refugium build, and the notice path changes what precedes them.

`TestBothEngraveFlowsDriveTheRetryLoop/supply` actually **passes** under the tag, yet the whole test is skipped (deviation 8's coarse skips).

A future Refugium-only change that broke scrubbing on any of these paths would pass CI.

**Fix:**

- Make the shared drivers profile-aware instead of skipping:
  - `driveWipeResidency`/reentry scripts should navigate the carousel to a program **by enum**, stepping over `programHidden`.
  - Passphrase waits should race prompt|notice and press Confirm on either, as `walk_refugium_nfc.js` already does with `raceFor`.
  - Scrub subtests whose exit is the passphrase question's Back should skip at **subtest** level only.
- Keep `skipUnderRefugium` for tests whose subject is the removed behaviour itself: NFC delivery, the passphrase keyboard, verify readback.
- At minimum, port these before merge:
  - `TestWipeZeroesEveryPinnedBufferAtRunLevel`
  - `TestRunSealedPayloadReentryAfterWipe`
  - `TestBuildFlowScrubsEverySeedOnEveryExit`, the three post-passphrase exits
  - `TestChainMnemonicFromAMePackedPayloadToASeedPlate`

## Minor

### M-1. Back or a wipe on the passphrase notice reads as "Skip", which drops each site's Back semantics

**Where:** `gui/passphrase_off.go:18-24`. The tagged test pins this as intended.

`showModal` (`slip39_polish.go:23`) cannot tell Back from OK, and `askBIP39Passphrase` returns `(0, true)` after any dismissal, including when `ctx.Done` ends the modal. At the reachable sites Back used to mean:

- `seedPassphraseStep` (`multisig_build_slots.go:890-894`): "not this seed", which un-registers the seed.
- `bip85` (`:325-328`): return to the seed.
- `singleSigInputs` (`singlesig.go:121-124`): return to the script.

Under the profile, Back moves forward with the seed registered.

Impact:

- **No secret is exposed and no passphrase is taken.** Later screens still allow backing out, so this is a navigation regression, not a safety one.
- **On `ctx.Done`,** returning `ok=true` is also what lets `seedPassphraseStep` return true during an unwind. It is harmless today, because every downstream loop checks Done.

**Fix:** use a notice whose Button1/Back returns `(0, false)` and acknowledge returns `(0, true)`, and return `(0, !ctx.Done)`. At all 8 sites `!ok` is benign: either "no passphrase" or "step back".

### M-2. Q2: single-card md1/mk1 gathers are Back-only dead ends; refuse up front instead

**Where:** `md1_gather.go:95-150`, `mk1_inspect.go:195-240`.

**How it is reached:** under the profile, a typed md1/mk1 chunk goes `newInputFlow` → `validateMStar` (`codex32_polish.go:286-293`) → `mdmkFlow` → "Inspect …" → gather.

**What happens:** after `syswPrimeCard`, an incomplete set opens a live loop whose only source is a reader that does not exist. The screen reads "This build takes chunks from the payload only.", so the operator is told, not deceived. Still, the plan says "No flow may wait on a reader that does not exist", and this one waits.

**Fix:** under `!ctx.nfcAvailable()`, if the set is incomplete after the payload prime, show a one-shot error and return false. Suggested text: "Captured k of N. This build takes the remaining chunks from the payload only; pack the full set." This mirrors the verify refusal and needs one tagged test.

### M-3. The field-off write is the last step of `Init`, so any earlier `Init` failure leaves the field however the previous image left it

**Where:** `cmd/controller/platform_sh2.go:304`.

`initNFC` runs after USB-PD, engraver, LCD and touch setup. Any `return nil, err` before it (`:240-292`) ends `main` (`main.go:30-33`) without the `regOpCtrl=0` write and without the fault screen. That is exactly the warm-reboot case D3 exists for.

Nothing reads the chip in this build, so no tag is read. But "off for the whole power cycle" does not hold on those paths.

**Fix:** call `p.initNFC(st25r3916.New(mi2c, NFC_INT))` immediately after `newMultiplexI2C(dataI2C)`, so the NFC write is the second I2C operation at boot. `p` would need to be constructed first, or the fault returned and stored.

### M-4. A malformed `pass:` record is loaded, not refused

**Where:** `gui/sysw_load.go:312-322`.

`syswFirstPassphraseRecord` uses `sysw.Classify`. A record with the `pass:` prefix and an undecodable body classifies as `ClassUnknown` (`sysw/record.go:127-131`), so the payload loads, and a malformed passphrase is still secret material.

**Fix:** refuse on `strings.HasPrefix(r, sysw.PassPrefix)`. Plan §4.3 says "a payload holding a `pass:` record".

### M-5. The "refugium-attached" walk does not exercise what its comments say

**Where:**

- `cmd/emu/platform_nfc_refugium.go:20-26` and `walk_refugium_nfc.js` (header) both say `startScanner` is "the only gate" between the forced reader and a read.
- In fact `ctx.nfcReader()` (`gui/nfc_scan.go:161-166`) short-circuits under the profile, so `Platform.NFCReader()` is never called. The forced reader is never handed to gui.

The implementer flags this honestly as Q4, and the arm still proves that gui never asks. **Fix:** correct the two comments, or add a check that `forceReader()` took effect (for example a JS-visible count of `NFCReader()` calls, asserted to be 0).

### M-6. Tagged producer coverage is thinner than §4.4 asks

**Missing:** §4.4 wants "a tagged test [that] walks each producer and asserts no QR whose content is an ms1 string is emitted". Two producers have no behavioural test today:

- **Engrave Text.** `TestRefugiumFreeTextMS1HasNoQR` covers only the choice step. Not covered:
  - the `ftBuildPlate` sink (`freetext_flow.go:1458`);
  - the forced-off notice (`:1586`).
- **Sealed-unlock codex32 plate** (`unlockEngraveCodex32`). It is covered only by the source pin `TestSeedStringPlatesGoThroughTheMS1Gate`.

**Fix:** add direct tests, for example:

- `ftBuildPlate(…, useQR=true, …)` on an ms1 text yields `plate.QR == nil` (this pairs with I-1);
- an unlock-codex32 walk compares the cut plate with `EngraveSeedStringTextOnly`.

### M-7. CI steps are plausible but unverified; observe the first run

**Will work:**

- `ubuntu-latest` has python3, and both scripts are mode 100755.
- `nix develop` pins TinyGo 0.41.1 (`flake.nix:45`), the same version used locally.

**Unverified:** the nix TinyGo links against nixpkgs-25.11's Go, not 1.26.7. Different inlining could remove a *default-ELF* marker (`otp.writeECC`/`writeOrRow` are the likely ones). That fails loudly through the positive control, never vacuously.

**Cost:** the tagged runner adds a second full gui run to the `tests` job, about 1–2 minutes on 4 cores.

**Can it pass vacuously? No:**

- `go test -list` failing aborts the runner under `pipefail`;
- an empty list exits 2;
- a missing `TestRefugiumProfileIsOn` exits 2;
- PASS + accepted skips must equal the listed count.

**Remaining gap:** the env-skip allowlist accepts those three names whatever their skip message says. That is Nit-level.

## Nits

- **N-1.** `gui/profile_refugium.go:19` cites `gui/refugium_split_test.go`, which does not exist. The file is `refugium_build_test.go`.
- **N-2.** `gui/sysw_load.go:160-166` says "a secret this build cannot use is never held in RAM". `sysw.Open` has already decrypted it into Go strings that cannot be wiped. Reword to "never enters a session".
- **N-3.** The source literal gate is AST `BasicLit`-only, so `"lock-"+"boot"` passes (measured). The call gate is name-only, so a method value `f := otp.AddBootKey; f()` passes too. The ELF check catches folded constants; say this in the test's comment so nobody relies on the source gate alone.
- **N-4 (Q1).** The one-row composer door: keep the row. The lead carries the reason there is only one way forward, and D9's no-one-row rule is about a picker that hides nothing. This is Brian's call, but I see no defect.
- **N-5.** Under the profile, "Verify the engraved plates? Verify now / Skip" is still offered, and "Verify now" always refuses. The refusal is informative and records nothing, which is correct. A lead such as "This build cannot read plates back" would save a tap. Optional.

## Judgement on the implementer's deviations

| # | Deviation | Verdict |
|---|---|---|
| 1 | `scanOffered()` instead of `nfcAvailable()` | **Correct.** Keying those sites on `FeatureNFC` would have changed the default build on reader-less platforms. I verified that `scanOffered` is a constant true in the default build, that every gated site reduces to the old code there, and that the default suite (1411) passes with an unchanged test list (+8 new). |
| 2 | Verify refuses up front | **Correct.** No seed is retyped, singlesig writes no bit, and multisig returns `verifyRefused`, which breaks the offer loops (`singlesig.go:285-298`, `multisig.go:343-354`). The restore document states the weakest true line. |
| 3 | SLIP-39 keeps its question | **Correct**, and better than the plan's literal reading: skipping it would recover a different valid seed. |
| 4 | `syswChoose` auto-takes the payload | **Acceptable.** The only caller with `syswAltScan` is `wallet_policy.go:76` plus `sysw_session.go:369`; these are public cards, Back on the next screen leaves, and the D9 rule applies. |
| 5 | `nfcDev` moved to `nfc_default.go` | **Good catch.** I confirmed no `st25r3916.Device` method has a symbol in the Refugium ELF. |
| 6 | ELF reverse control | **Good.** It killed the two-default-ELFs mutation. |
| 7 | Runner skip semantics | **Sound.** The skip must be the test's first log line and must be named; anything else fails. |
| 8 | Coarse whole-test skips | **Not acceptable as is.** See I-2. |

### Default build unchanged except the characterization-tested refactor (item 5)

`handleDebugCommand`'s three arms match the old inline switch, and `debugPassThrough` falls through to `return {scan: cnt}` exactly as before. `programHidden` is false in the default build, so these are equivalent:

- the carousel loops break after one step;
- `layoutMainPager`'s dot count and offsets are identical;
- the `engravePassphrase` dispatch is unconditional.

The controller and emulator default twins are the moved code, unchanged. `backup.engraveSeedString` with a non-nil QR is unchanged: the codex32 goldens pass.

### NFC hunt (item 1)

| What I checked | Finding |
|---|---|
| `startScanner` call sites | All 7 go through `ctx.nfcReader()` and `startScanner`'s own nil-ing |
| Platform asks | `NFCReader()` and `FeatureNFC` appear only in `nfc_scan.go:165,174`, pinned by a test |
| `NFCReader`/`Features` implementations | Only the two controller twins and the two emulator twins exist. There is no other platform. |
| Controller NFC reach in the Refugium build | It constructs no `nfcDev`, configures no interrupt (`New` is passive; `Configure`/`SetInterrupt` live in `enable`, which is not linked), and makes one `writeReg` |
| Fault path | Checked first in `uiFlow` (`gui.go:2100`). The screen has no clickables. `ctx.Done` is set only by a wipe, which needs `wipe.armed()`, which is never true before any flow, so the screen holds. |
| Hidden routes | No poller or tag-emulation path remains, and the sealed-payload classes (`unlock_session.go:68-70`) carry no passphrase |

### Passphrase hunt (item 2)

| Route | Finding |
|---|---|
| BIP-39 prompts | 8 sites, all via `askBIP39Passphrase`, pinned by a test |
| Keyboard entry functions | `passphraseFlowTitled` and `syswPassphraseFlowTitled` refuse under the profile |
| Passphrase engrave program | Hidden, and its dispatch is gated |
| `engravePassphraseFlowFrom` | Reachable only from an NFC scan (dead) and the hidden program |
| `engravePassphraseFlowPreloaded` | Needs a non-empty passphrase |
| `pass:` payloads | Refused at load (with the M-4 gap) |

No route remains to take or apply a BIP-39 passphrase. "Skip" never reads as cancel in a way that loses a flow; the reverse problem (Back reads as Skip) is M-1.

### ms1-QR hunt (item 3)

The producers I reviewed:

- `qr.Encode` call sites: `gui.go:746` (descriptor), `:852` (SeedQR of words, allowed), `:2740` (gated)
- `backup.EngraveSeedString` (gated, pinned)
- `fit.go` (free text, gated except I-1)
- `passphrase.go` (program hidden)
- `hashlock` (phrase validator refuses ms1-shaped input)
- `txqr` (transaction bytes)

`biptool` is not in the firmware import graph.
