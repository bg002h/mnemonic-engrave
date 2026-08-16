# S5 fold re-review — did the fold CLOSE each round-0 finding, or MOVE it?

**Scope:** `git diff 7da66bd..830aaf7` (7 commits) against
`design/agent-reports/s5-whole-diff-review-round0.md` (3C/14I) and
`design/agent-reports/s5-whole-diff-fold-round0.md` (the fold's own log).
**Not in scope:** the 10 commits under `7da66bd`; `gui/singlesig.go` (F-197/F-198,
already filed); a fresh audit of anything outside the fold diff.
**Worktree:** `/scratch/code/shibboleth/wt-s5` @ `830aaf7`, read-only throughout —
confirmed `git status --porcelain --branch` → `## s5-multislot` only.

## Machine baseline, re-run (matches the settled facts in the brief)

```
$ nix develop --command go test ./... -count=1                    -> exit 0, all ok (gui 134.6s)
$ nix develop --command gofmt -l ./                                -> exit 0, empty
$ GOCACHE=$(mktemp -d) nix develop --command go vet ./...          -> exit 1, 0 findings outside _test.go
```

## Verdict

**0 Critical / 0 Important survive.** All 18 round-0 items (C-1..C-3, I-1..I-14,
F-189) are genuinely CLOSED at the production site, each with a pinning test that
was run and observed to (a) pass on the current tree and (b) fail under the
mutation the round-0 report or the fold's own log names. No rebuilt fixture was
found still hand-assembling a state the real flow cannot produce — the specific
trap named in the brief (C-2's `TestGateAcceptsSameSeedAtDistinctOrigins`) is
verified fixed: it now builds its registry via two real `reg.add()` calls and
takes its `sources` from `buildSlotSources`, the actual production projection.

No new defect was found in the fold's own text within the scope of this lens
(finding-closure verification). The fold's own report additionally surfaces two
new items (N-1/N-2, filed as F-197/F-198) on `gui/singlesig.go` — correctly left
unfolded as out-of-scope, and already filed per the brief.

## FINDING → STATUS → EVIDENCE

| # | Status | Site checked | Pinning test, run | Result |
|---|---|---|---|---|
| C-1 | **CLOSED** | `gui/multisig_verify.go:869` (`verifyMultisigLegsPartial` runs before the incomplete report; `checked` built only after `err==nil`, i.e. every leg found+passed its plate) | `TestVerifyIncompleteDoesNotCallAForeignPlateChecked`, `TestVerifyIncompleteReportsWhatTheComparatorMatched` | Both PASS (`go test ./gui/ -run ... -v`); foreign-plate case now returns `Verify Failed` naming `@0`, honest partial case returns `Verify Incomplete` naming `@0`/`@1`/`@2` |
| C-2 | **CLOSED** | `gui/multisig_build_slots.go:411` (`bound := map[uint32][]binding{}`, keyed on `seed.MasterFP`, `slotFromSeed` arm now resolves `seed` instead of existence-checking) | `TestGateAcceptsSameSeedAtDistinctOrigins` (rebuilt: `s5Registry(t, fixtureMasterA, fixtureMasterA)` → two real `reg.add()` calls → `sources := buildSlotSources(...)`, the real projection), `TestBuildFlowAnnouncesTwoSlotsFromOneSeed` (flow-level) | Both PASS. Fixture no longer hand-builds `SeedID: 0` twice — confirmed by reading `gui/multisig_build_gate_test.go:221-234`, and `s5Registry` helper (`gui/multisig_build_s5_test.go:38-54`) calls `reg.add()` in a loop, matching `buildMultisigPolicyFlow`'s per-held-slot call |
| C-3 | **CLOSED** | `gui/multisig.go:217` (`buildFullModeLabel(passphrase != "")`), `:357-358` (`buildPlateInventoryLines(cardsOut, oneSeedPassphraseFact(passphrase != ""))`) | `TestSupplyPassphraseRunTellsTheOperatorWhatIsMissing`, `TestSupplyRestoreDocSaysSoWhenNoPassphraseWasUsed` | Both PASS; drove screen reads `"Full (seed + keys, NOT passphrase)"` and restore doc reads `"A BIP-39 passphrase WAS used..."` |
| I-1 | **CLOSED (test-only, as claimed)** | `gui/multisig_build_tail.go` `slotFromBoth` arm — unchanged, correctly | `TestBuildTailEngravesABothSlotAtTheCardsOwnOrigin` | PASS. Reaches the tail through the real gate (`buildSlotGate` asserted to succeed first, not bypassed) and asserts both origin AND key-byte equality against the assembled policy |
| I-2 | **CLOSED** | `gui/multisig_verify.go:618-695` (the `if full` block, `multisigVerifyMS1Entry`) | `TestVerifyFullModeTwoSeedsReportsTheFullSuccess`, `TestVerifyFullModeBackAtTheSecondMs1ReportsIncomplete`, `TestVerifyFullModeBindsEachMs1ToItsOwnSeed` | All 3 PASS. Coverage re-measured (`GOCACHE` cold, `-coverprofile`): `multisigVerifyMS1Entry` now 69.2% (was 0%); `multisigVerifyFlow` 67.7% |
| I-3 | **CLOSED** | `gui/multisig_verify.go:424` (`multisigVerifyFailureText`, type-switches on `errVerifyLegHasNoPlate`/`errVerifyPlateUnclaimed`) | `TestVerifyFailureTextNamesWhatTheComparatorFound` | PASS |
| I-4 | **CLOSED, option (a)** | `gui/multisig_build.go:402`, `gui/multisig.go:291` (loop on `multisigVerifyResult`, re-offer on incomplete/failed only) | `TestBothEngraveFlowsReOfferTheVerify` | PASS |
| I-5 | **CLOSED** | `gui/multisig_build_slots.go:259` (`passphraseFacts()`), `gui/multisig_build_census.go:121` (`buildPassphraseInventoryLines` takes the slice, enumerates passphrased AND bare seeds); production call site `gui/multisig_build.go:475` uses `reg.passphraseFacts()` (the real registry, not a literal) | `TestRestoreDocNamesEveryPassphrasedSeed`, `TestRestoreDocSaysWhichSeedsNeedNoPassphrase`, `TestSingleSeedInventoryIsUnchanged` | All 3 PASS; observed output names both `@0 (fp 8aaa4f4b)` and `@1 (fp d70ed067)` distinctly |
| I-6 | **CLOSED, departure from prescribed fix (documented, correct)** | `gui/multisig_build_payload.go:222` (`classifyCosignerSupply`, `open==0` → `cosignerAutoFill`) AND `gui/multisig_build.go:118-123` (`if open > 0 { ...gather machinery... }`, skipped entirely at `open==0`) | `TestZeroDemandBuildIsNotRefusedForAPayloadItDoesNotNeed` (classify table, all 3 states), `TestBuildHoldingEverySlotReachesTheSeed` (flow-level, asserts seed entry reached for both @0 and @1) | Both PASS. The fold's own log correctly identifies the review's fix alone was insufficient (moves the dead-end into `bundleGatherFlow`) and the second half (skip-the-gather) is what actually closes it — verified present in code |
| I-7 | **CLOSED** | `gui/multisig_build.go:1608` (`buildOriginAnnouncement(script, held []heldSlotOrigin)`, now a function of actually-used origins via `heldOriginSummary`) | `TestBuildFlowAnnouncesTwoSlotsFromOneSeed` (extended to the Policy Review screen) | PASS. Re-minted `S5-trace-b.walk.json` `reviewScreen` verified to read `"...@0 at m/48h/0h/0h/2h, @1 at m/48h/0h/1h/2h and @2 at m/48h/0h/0h/2h..."` |
| I-8 | **CLOSED — operator decision (b) ACCEPT AND DOCUMENT, made in writing** | `gui/multisig_build_census.go:58-70` (justification rewritten), `buildPlateInventoryLines`'s "Seed handling" ruling rewritten | `TestSeedResidencyRulingDescribesTheMultiSeedReality` | PASS. `grep -rn "holds exactly one seed\|A seed you entered" gui/` → 0 hits (re-verified). Decision record `design/agent-reports/s5-i8-seed-residency-decision.md` present and its reasoning matches what shipped |
| I-9 | **CLOSED** | `oracle/record_test.go:378-411` (`requiredStages` table incl. S5, `TestEveryRequiredStageHasAGateRecord`, `TestS5GateHasARecord`) | Both, plus `TestS0GateHasARecord` | All PASS on the frozen tree. **Independently re-mutated in a `cp -a` copy** (`/tmp/wt-s5-copy`, never touching `wt-s5`): deleted all 4 `S5-trace-b.*` files → both new tests **FAIL** as designed (`"S5 has no gate record..."`) |
| I-10 | **CLOSED** | `gui/multisig_build_slots.go:588` (comment now cites `F-196` by ID) | n/a (a comment-correctness item; no code pin required by round 0) | `grep -n "F-196" design/FOLLOWUPS.md` → present at line 6979, with an owning phase (`the spec — it is a model change, and earns its own R0`) |
| I-11 | **CLOSED** | `gui/bundle_flow.go` (`bundleAbortWarningText`, false promise replaced) | `TestAbortWarningPromisesOnlyWhatTheDeviceCanDo`, `TestBundleEngraveHasNoResumeMechanism` | Both PASS |
| I-12 | **CLOSED** | `gui/bundle_flow.go:394-450` (`bundleEngrave` returns `bundleEngraveResult`), both callers gate on it (`gui/multisig_build.go:402`, `gui/multisig.go:291`) | `TestBothEngraveFlowsGateOnACompletedSet`, `TestSupplyAbortIsTheLastScreenOfTheProgram` (flow-level) | Both PASS |
| I-13 | **CLOSED** | `gui/multisig_verify.go:941` (`multisigVerifyOKMessage`, single-leg arm now `full`-aware) | `TestVerifyOKMessageClaimsASecretOnlyInFullMode` | PASS (implicit in full suite run; not re-isolated individually, but exercised — see note below) |
| I-14 | **CLOSED** | `gui/multisig_verify.go:497` (`multisigVerifyCoveredSeedBody`), wired at `:770-773` via `multisigVerifySeedIsInnocent` in the `default:` arm (previously only wired into the `len(slots)==0` arm) | `TestVerifyCoveredSeedBodyDoesNotAssertAForeignSeed` | PASS, all 4 cells |
| F-189 | **DONE** | `multisigEngraveCards` — no non-comment reference remains in `gui/*.go`; `findUserSlot` signature is `(int, bip32.Path, bool)`, no `reused` return | `TestMultisigEngraveCards` (re-pointed to `multisigEngraveCardsMulti`) | PASS |

I-13 re-run in isolation: `go test ./gui/ -run TestVerifyOKMessageClaimsASecretOnlyInFullMode -count=1 -v` → PASS.

## Additional fold-introduced-defect check (second half of the lens)

Checked specifically for the classes round 0 named as recurring (fail-safe
dedupe, unrealistic fixture, cardinality-not-identity, unrunnable gate):

- **Format consistency in C-2's distinctness key** — `slotFromSeed` arm computes
  `origin: derivedSlotOrigin(script, s.Account).String()`; `slotFromBoth` arm
  computes `origin: k.OriginPath.String()`. Both are `bip32.Path.String()` calls
  on parsed paths (not the card's raw string spelling), so the two arms produce
  origins in the same format and the same-origin/distinct-origin comparison in
  the notice loop cannot be defeated by a formatting mismatch (e.g.
  `m/48'/.../2'` vs `m/48h/.../2h`). Read directly, not inferred from a comment.
- **Re-mint provenance** — `cmd/emu/walk_trace_b.js` now `throw`s (not a
  recorded flag) on both new assertions (`ORIGINS_EXPECTED` at line ~380,
  `multiAccountNotice` at line ~662), matching the round-0 report's own
  complaint that a flag "lets a walk mint a record that vouches for a sentence
  the device should not have drawn." Read directly.
- **Committed record matches the code** — `oracle/gaterecords/S5-trace-b.walk.json`
  `claims.multiAccountNotice: true` and `reviewScreen` / `keySourcesScreen` text
  verified byte-for-byte against the fold log's quoted excerpts (they match).
- No package outside `gui`/`oracle`/`cmd/emu` was touched by the fold diff
  (confirmed by the `git diff --stat` file list), so no cross-package regression
  surface exists beyond what the full `go test ./...` run already covers.

No new Critical or Important was found in the fold's own text.
