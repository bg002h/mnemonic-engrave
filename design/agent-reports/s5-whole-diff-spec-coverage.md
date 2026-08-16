# S5 whole-diff review — SPEC COVERAGE lens

Repo under review: `/scratch/code/shibboleth/wt-s5` (frozen worktree, commit `7da66bd`).
Diff: `git diff main..s5-multislot` (10 commits, 57 files, +8873/-607).
Governing docs: `design/SPEC_multisig_build_repair.md` (§0.1a, §4.1, §4.1a, §4.2, §4.3,
§4.4, §4.5, §4.6, §7 rulings), `design/IMPLEMENTATION_PLAN_multisig_build_repair.md`
(S5 section, lines 1137–1300, and §0.1a/§0.1b at lines 76–146).

Method: enumerated every normative clause scoped to S5 from the spec and plan, then
located the satisfying code/test by reading it (not by name/comment), and ran the
relevant tests myself. All `go test` invocations below were run under
`nix develop --command`.

**Reused from the dispatch brief, not re-derived:** `go test ./... -count=1` (cold
GOCACHE) exit 0 / 51 ok / 0 FAIL; `gofmt -l ./` clean; `go vet ./...` (cold GOCACHE)
exit 1 with 40 test-only findings (established baseline); `./scripts/oracle-live.sh`
PASS 7/7; `./cmd/emu/build.sh` OK. F-189..F-195 already filed, not re-reported.

---

## CLAUSE -> EVIDENCE table

| Clause | Requirement (paraphrased) | Evidence (file:line) | Verified how |
| --- | --- | --- | --- |
| §4.1 multi-slot self | operator declares ≥1 held slot; new-seed or reused-account per slot | `gui/multisig_build.go:876` `SelfSlots []int`; `multisigSelfSlotPickFlow` `gui/multisig_build.go:798-846` (ascending, multi-round picker) | Read; `TestMultiSlotSelfAssembles` passes |
| §4.1 `OriginDivergent`/`OriginShared` | used correctly per whether origins agree | `gui/multisig_build.go:1300-1307` `commonOrigin`→`OriginShared`/`OriginDivergent` | Read; `TestAssembleBuildPolicyStaysSharedWhenOriginsAgree` passes |
| §4.1 card origin not discarded | `cosignerFromCard` carries `card.Path` | `gui/multisig_build.go:1166-1191` | Read; `TestCosignerCardOriginIsHonoured` passes |
| §4.1 duplicate-key check, FINAL slot set, exact cc‖pk | `duplicateSlotPair` runs over `all` after assembly fill, before origin-mode assignment and before review | `gui/multisig_build.go:1024-1031` (comparison), `:1266-1270` (call site, before `commonOrigin`) | Read; `gui/multisig_build_dupkey_test.go` |
| §4.1 same seed ≥2 slots, distinct origins legitimate | `buildSlotGate` notice branch | `gui/multisig_build_slots.go:436-465` | Read; `TestGateAcceptsSameSeedAtDistinctOrigins` (referenced, exists) |
| §4.1 passphrase per seed | asked inside `buildSeedForSlot`, one call per held slot | `gui/multisig_build.go:542-573` | Read |
| §4.1a item 1 — leg at declared origin, never shared | `buildEngraveTail` switch on `s.Kind`: `slotFromSeed`→`derivedSlotOrigin`, `slotFromBoth`→card path | `gui/multisig_build_tail.go:83-96` | Read; `TestLegDerivedAtHeldSlotOrigin` passes |
| §4.1a item 2 — one mk1 per held slot | `buildEngraveTail` loop appends one leg/mk1 per held slot, no de-dup on mk1 | `gui/multisig_build_tail.go:96-121` | Read; `TestOneMk1PerHeldSlot` passes |
| §4.1a item 3 — every master's ms1 engraved in full mode | ms1 dedup keyed on **ms1 string**, not SeedID (documents & fixes the C1 dedupe defect) | `gui/multisig_build_tail.go:70-121` | Read; `TestFullModeEngravesMs1ForEveryMaster` passes, asserts actual decoded entropy per master (catches "captured one mnemonic" mutation) |
| §4.1a item 4 / §4.5 — byte comparison extends to mk1s and ms1s | Walk census compared byte-for-byte to oracle-derived expectation, unconditionally | `oracle/expect_test.go:113` `TestEveryGateRecordCensusMatchesItsCommittedExpectation` (no toolchain needed, runs in plain `go test ./...`) | Ran: part of the already-green `go test ./...`; confirmed it's in the unconditional (non-`oraclelive`) build tag by reading the file header |
| §0.1a — sh(wsh)/sh/wsh script-type origin, template-aware from S5 | `derivedSlotOrigin` / `multisigScriptTypeComponent` | `gui/multisig_build_slots.go:96-118` | Read |
| §4.2 seed lifetime, scrub on every exit, mutation-checked | `seedRegistry`, `defer reg.scrub()` at flow entry, subtests per exit class | `gui/multisig_build.go:190-192`; `gui/multisig_build_scrub_test.go:108` `TestBuildFlowScrubsEverySeedOnEveryExit` | Ran targeted: `go test ./gui/ -run TestBuildFlowScrubsEverySeedOnEveryExit` — PASS (S4-owned, re-verified still wired after S5's registry changes) |
| §4.3 model: `payloadKey`/`derived`/`both` per slot | `slotSource`, `slotSourceKind` | `gui/multisig_build_slots.go:22-90` | Read |
| §4.3 mechanism: derivation not fingerprint | `buildSlotGate` calls `findUserSlot` | `gui/multisig_build_slots.go:369-427` (comment + code) | Read |
| §4.3 M-B binding 1 — `both` slot: card origin authoritative | `buildEngraveTail`/`assembleBuildPolicy` both read the card's `Path`, never `account` | `gui/multisig_build_tail.go:88-93`; `gui/multisig_build.go:1236-1244` (fills from `cosigners[gi]` for `both` slots) | Read |
| §4.3 M-B binding 2 — `derived` slot: account is the BIP-48 component | `derivedSlotOrigin(script, account)` | `gui/multisig_build_slots.go:96-104` | Read |
| §4.3 gate fires on `both` only, never inferred | `case slotFromCard: continue` | `gui/multisig_build_slots.go:381-383` | Read; `TestGateIgnoresUnassignedCosigners` (referenced, exists) |
| §4.3 outcome table row 1/2 (proceed / FAIL LOUDLY naming slot) | `findUserSlot(...)` miss → `errBuildSeedKeyMismatch{Slot}` | `gui/multisig_build_slots.go:406-410` | Read; `TestGateStillFiresAfterOriginsDiverge` PROCEED+FAIL pair, ran: `go test ./gui/ -run TestGateStillFiresAfterOriginsDiverge -v` — both subtests PASS |
| §4.3 row 3 (fingerprint contradicts derivation) | checked after derivation match, named refusal | `gui/multisig_build_slots.go:412-419` | Read |
| §4.3 row 4 (duplicate final slot) | delegated to `duplicateSlotPair` over final assembled set — not re-decided in the gate | `gui/multisig_build_slots.go:441-448` (comment states this explicitly) | Read; cross-checked against `duplicateSlotPair` above |
| §4.3 row 5 (one seed, distinct origins → notice) | `buildSlotGate` notice loop | `gui/multisig_build_slots.go:436-465` | Read |
| §4.3 row 6 (`payloadKey`, no seed match → normal) | `case slotFromCard: continue` (same as gate-fires row) | `gui/multisig_build_slots.go:381-383` | Read |
| §4.4 nested segwit nameable (S3-owned, not S5, but load-bearing for S5's `sh(wsh)` default) | `scriptName` three call sites | `gui/bundle.go:315`, `gui/md1_inspect.go:87`, `gui/multisig_restore.go:80` | Read — all three present and consistent |
| §4.5 emulator walk, byte comparison, per-stage gate | `cmd/emu/walk_trace_b.js` (689 lines) + `oracle/gaterecords/S5-trace-b.*` | `oracle/gaterecords/S5-trace-b.walk.json`: `"ok":true`, `plateCount:17`, 2 ms1 + 7 mk1 (2+3+2) + 8 md1 | Read the committed walk/record/expect JSON directly; confirmed the census-match test above is what enforces it in `go test ./...` |
| §4.6 tier-1 budget for NEW S5 tests | new S5 tests use `synctest`, no real-time waits | `gui/multisig_build_s5_test.go`, `gui/multisig_build_s5_flow_test.go` | Ran: `go test ./gui/ -run '<8 named S5 tests>' -v` — total 0.247s. The pre-existing whole-`./gui/` 56s is the already-acknowledged, explicitly-deferred (§4.6) baseline, not an S5 regression — ran `time go test ./gui/ -count=1`: 56.1s, consistent with the spec's own 54.3s baseline measurement, not materially worse |
| S5 plan test 6 — depth-0 card named refusal | `errBuildEmptyOrigin` | `gui/multisig_build.go:1032-1040`, refusal wired at `:262-267` | Read; `TestDepthZeroCosignerCardIsNamedRefusal` — ran, PASS |
| S5 plan test 7 — re-run mints byte-identical plates | `TestReRunMintsByteIdenticalPlates` | `gui/multisig_build_s5_test.go:471-514` | Ran — PASS; compares both `assembleBuildPolicy` outputs and `buildEngraveTail` card sets across two runs of the same inputs |
| Abort text: DESTROY only for a set carrying a seed | `bundleAbortWarningText(p, secret)` | `gui/bundle_flow.go:485-497`; predicate `bundleSetCarriesASecret` at `:450-452` | Read |
| Passphrase omission stated in both mode label and restore doc | `buildPassphraseInventoryLines`; census label `"Full (seed + keys, NOT passphrase)"` | `gui/multisig_build_census.go:83-142` | Read |
| Review screen shows per-slot keys (not just fp) | `buildSlotKeyStrings`, passed into `buildReviewFlow` | `gui/multisig_build.go:279-289` | Read |
| EXPERIMENTAL warning rewritten (independent-source comparison, fingerprint disclaimer, names external-coordinator backstop) | `multisigBuildExperimentalWarningBody` | `gui/multisig_build.go:668-676` | Read |
| Engrave order contract: all-ms1s → all-mk1s → md1 | `multisigEngraveCardsMulti` | `gui/multisig_engrave.go:47-68` | Read; matches `oracle.ArtifactKindsFor` per its own comment |
| F-188 supply-path plate-per-matched-slot + dedupe of byte-identical plates, announced before first cut | `supplyEngraveTail`, `multisigSlotsShareAKey`, census insert in `gui/multisig.go` | `gui/multisig_supply_tail.go:114-160`; `gui/multisig.go:185-260` | Read (already-filed F-188 territory per brief, cross-checked mechanism only) |
| Verify: obligation carries slot set AND engraved md1 | `multisigVerifyFlow(ctx, th, full, expectedSlots, engravedMd1)`, both refused if empty | `gui/multisig_verify.go:426-444` | Read; both call sites pass real `engraveMd1`/`suppliedMd1` (`gui/multisig_build.go:399`, `gui/multisig.go:297`) |
| Verify: policy-identity check (Critical #3 fix) — exact chunk equality before anything else | `slices.Equal(readbackMd1, engravedMd1)` | `gui/multisig_verify.go:474-479` | Read |
| Verify: `expectedSlots ∩ allUserSlots(seed)` | `verifyFreshSlots` | `gui/multisig_verify.go:271-281` | Read |
| Verify: per-leg bijection via mk1 xpub, not origin path (Trace B has two same-path legs across masters) | `verifyMultisigLegs`/`verifyClaimPlate` | `gui/multisig_verify.go:284-326` | Read |
| Oracle: `built-policy-full`/`built-policy-watch` ExpectKind + `ArtifactKindsFor` | `oracle/expect.go:76-158` | Read |
| Oracle pin bump `ms-cli-v0.16.0` + S0-trace-a re-anchor | `oracle/pins.json:16` | Read; matches commit `7910e00`..`7da66bd` history |
| S5.0 ordering: oracle rows land and close green BEFORE engrave-tail device code | Commit order `f0006b7`→...→`7da66bd` includes an S0 pin-bump commit ahead of tail commits per the plan's own ordering rule | `git log --oneline main..s5-multislot` | Ran `git log`; oracle/pins-touching commit precedes the S5(A+B) tail commit in the series (both present, order plausible from messages — not independently re-verified commit-by-commit, low stakes given the gate itself is green) |

---

## Findings

### Important — the picker cannot express a genuinely mixed self-slot source, and the code's own claim that this gap is tracked does not check out

**File:** `gui/multisig_build_slots.go:509-518` (comment), mechanism at
`gui/multisig_build.go:80-92` and `gui/multisig_build_slots.go:517-538`
(`buildSlotSources`).

**Failure scenario:** An operator holds two slots of one BIP-48 masters worth of
keys, e.g. `@0` (whose key is *also* printed on a payload cosigner card — they
want the §4.3 cross-check) and `@1` (freshly derived, no card exists for it — a
completely ordinary, spec-legal shape per §4.1's "another account index of a seed
already supplied"). `multisigSelfSlotPickFlow` collects the held set `{0,1}`
correctly, but `buildSelfSourceFlow` (`gui/multisig_build.go:83`) asks **one**
Yes/No question — "Are your @0 and @1 keys on cards?" — and `p.SelfFromCard`
applies that single answer to **every** held slot via `buildSlotSources`
(`gui/multisig_build.go:463-475`, `gui/multisig_build_slots.go:592-605`: the
`slotFromBoth` vs `slotFromSeed` branch is chosen once, for the whole held set,
not per slot).
Concretely: answering "YES" when only `@0` has a card forces `@1` into
`slotFromBoth` too; with no card supplied for `@1`, `open = p.N` requires the
payload to already carry a card at every non-held slot **and** every held slot,
so the flow either dead-ends on an under-supply refusal before the operator can
even try, or (if a card of someone else's happens to fill that gather slot)
`buildSlotGate`'s `bothSlotKey`/`findUserSlot` comparison against the wrong
card fails loudly with `errBuildSeedKeyMismatch`. Answering "NO" instead derives
**both** slots purely from the seed and skips the cross-check for `@0` even
though the operator has a card that could have caught a corrupted derivation.
Neither answer reaches the mixed configuration the underlying model
(`slotSource` is genuinely per-slot) and the gate already support — §4.3's own
words are "Every slot @0..@{n-1} carries exactly one source, **chosen by the
operator**".

The code comments (`gui/multisig_build_slots.go:513-518`) acknowledge this
exact gap explicitly: *"a genuinely MIXED build... is not expressible through
the screens... Making it per-slot... is a change to the model this block does
not own; **filed rather than smuggled in**."* I grepped
`design/FOLLOWUPS.md` (the project's only follow-up ledger, per
`mnemonic-engrave/CLAUDE.md`) for `SelfFromCard`, `per-slot`, `mixed build`, and
`buildSelfSourceFlow`, and read the six most-recently-filed entries (F-189
through F-195, `design/FOLLOWUPS.md:6848-6955`) verbatim. None of them name
this gap. The "filed" claim does not check out against the project's own
tracking file as it exists on disk right now.

**Why Important, not Critical:** every reachable path from this gap either
refuses loudly (a card-mismatch or under-supply refusal) or falls back to a
strictly-safe all-derived configuration — there is no silent wrong result and no
funds-loss path. It is a genuine "can a user do the thing" gap against §4.3's
per-slot language, compounded by an unverified process claim in the code that
would otherwise have made it discoverable.

**How verified:** read `gui/multisig_build.go:60-92`, `gui/multisig_build_slots.go:481-538` in full; ran
`grep -n "SelfFromCard|per-slot|mixed build|buildSelfSourceFlow" design/FOLLOWUPS.md`
(no hits) and `grep -n "^### F-19" design/FOLLOWUPS.md` then read F-189..F-195
verbatim (`design/FOLLOWUPS.md:6848-6955`) to confirm none matches.

---

## Reverse-direction check (unauthorised normative behaviour)

Read every production (non-`_test.go`) file in the diff's stat list for
behaviour changes not traceable to a spec clause or an explicit operator ruling
(F-188, §0.1a, §0.1b). Found none beyond the one gap above. Specifically checked
and cleared: `gui/multisig_match.go` (pure extraction, `findUserSlot` behaviour
byte-for-byte preserved, `allUserSlots` is new but additive), `gui/singlesig.go`
(mechanical signature change, adds a title param to `bundleEngrave` for the new
DESTROY-wording call site, no behaviour change to single-sig), `cmd/buildpayloadcards/main.go`
(new scratch tool, explicitly "not part of the firmware", public BIP-39 vectors
only, not shipped). `gui/multisig.go`'s F-188 rewrite and `gui/multisig_supply_tail.go`
are the already-filed F-188 ruling, not unauthorised.

## Not re-reported (already filed / already machine-verified per brief)

F-189, F-191, F-192, F-193, F-194, F-195 — encountered `multisigEngraveCards`
(`gui/multisig_engrave.go:25`, explicitly documented as F-189's retained,
no-production-caller shape) and did not re-file it.
