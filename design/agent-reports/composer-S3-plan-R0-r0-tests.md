# Composer S3 plan — R0 round 0, TESTS lens

Reviewer: independent (did not author the plan). Method: mutation testing in a
disposable scratch copy (`/scratch/code/shibboleth/.s3-tests-lens`, git-init'd
for clean mutate/revert/diff cycles), against
`design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` (master). One question:
can every test in the plan actually FAIL when the code it guards is wrong?

Baseline reproduced: `go test -run '^TestComposer' ./gui/` → **ok, 118
sub-tests** (counted via `grep -cE -- '--- (PASS|FAIL)'` on `-v` output, not
hand-counted). Every mutation below was applied, run, observed, then reverted
and confirmed clean via `git diff`.

## Headline: two structural defects explain most of the mutation table

**H-1. Part B has no caller.** `composerSeatFlow`, `composerMappingReview`,
`composerFormPick`, `composerConsentFlow`, `composerSecretFormPick`,
`composerEngraveModePick`, `composerApplyShapeEdit`/`composerDiscardAssignments`,
`composerCensusRefusal`/`composerCensusLines`, `composerMintCards` are declared,
compile, and (mostly) have direct unit or harness tests **called on them in
isolation** — but grepping all non-test `.go` files for each name (excluding
its own `func` declaration) returns **zero production call sites**. Nothing in
`composer_flow.go` (Part A's, the only flow file that exists in the wired tree)
or anywhere else reaches them. Plan line 4437 states: *"Part B replaces
`gui/composer_flow.go` wholesale (the gate's ``Replace `gui/composer_flow.go`
`` anchor)"* — but `grep -n '^Replace \`gui/composer_flow.go\`'` over the
**entire 7638-line document returns nothing**. No task ever writes that
replacement. Consequence: every Part B screen is permanently unreachable by
any test this plan could produce, which is why mutation categories 6–8 below
read almost uniformly "NONE."

**H-2. Two tasks promise a test file and never write it.** Task B7's
`gui/composer_engrave.go` names `Test: gui/composer_engrave_test.go` (line
6964); Task B8's `gui/composer_cards.go` names `Test:
gui/composer_cards_test.go` (line 7133), with Task B8's own prose specifying
three cases ("existing stubs preserved IN ORDER," "a card round-trips through
`mk.Decode`," "a partially seated composition's cards carry the TEMPLATE stub
only"). `grep -c` for each filename over the whole plan returns **1** — the
"Files:" line only. No ```go block exists for either. Confirmed absent from
the wired tree: `ls gui/composer_cards_test.go gui/composer_engrave_test.go`
→ no such file, for both. Card minting (the `mk.Decode` round-trip) and the
engrave-form choice ship with **zero tests of any kind**, not even the plan's
own promised coverage.

**H-3. Task B10 (§12 item 6, "the seating vector leg") is prose, not code.**
Its section (plan lines 7394–7413) is five numbered bullets with no ```go
block at all. `gui/composer_seating_vectors_test.go` does not exist in the
wired tree. This was to be the ONE test proving seated cards round-trip
through the SHIPPED `seatKeyCards` consumer to reproduce a real keyed policy's
addresses — the closest thing to funds-safety proof for seating — and it does
not exist.

## Mutation table

`CAUGHT` = a named test failed. `NONE` = whole suite (`go test -run
'^TestComposer' -v ./gui/`) stayed green. Root cause abbreviated where it's
H-1/H-2/H-3 above.

### 1. Door (§7a/§8r) — `composer_door.go`

| # | mutation | plan line(s) | result |
| - | - | - | - |
| 1a | key-state line always "Keys loaded: 0" | composer_door.go:72 | **CAUGHT** — `TestComposerDoorLinesCoverEveryKeyState/keys_only` |
| 1b | hide "From payload" when payload holds an md1 (drop `\|\| s.has(sysw.ClassMDMK)`) | composer_door.go:90 | **NONE — Critical.** No test ever loads a ClassMDMK-holding session into `composerDoorHasConsumablePolicy`; the two existing cases are "key records only" and "a descriptor" only |
| 1c | route "Build a new policy" to `composerRouteScan` instead of `composerRouteBuild` | composer_door.go:117 | **NONE — Critical.** No test in the whole `gui/` package (not just `TestComposer*`) ever clicks "Build a new policy." Every click-driven walk that passes the door does `Down; Button3` → "From payload." `TestComposerNoPayloadWalkReachesAKeylessTemplateThatDecodes` (composer_flow_test.go:174), despite its name and doc comment claiming it "chooses Build, composes a shape, ... consents, and engraves," stops at `pumpUntil(frame, "Build a new policy", 16)` and never clicks. The "decodes" half is proven separately, by calling `md.Compose`/`md.ExpandWalletPolicyChunks` directly — bypassing the GUI entirely |

### 2. Shape (§7b/§4e/§8m) — `composer_shape.go`, `composer_state.go`

| # | mutation | plan line(s) | result |
| - | - | - | - |
| 2a | picker bound allows a 33rd slot (`free := ... + 1`) | composer_state.go:203 | **CAUGHT** — `TestComposerPickerBoundsNeverOfferAnIllegalValue` |
| 2b | accept a lock-only path (disable the Done-branch's `md.ValidatePathList` check entirely) | composer_shape.go:298-302 | **NONE — Critical.** `composerShapeFlow`, `composerAddPath`, `composerPathEdit`, `composerKeysEdit` have **zero** test callers (`grep -rn "composerShapeFlow("` etc. over `*_test.go` → nothing). Disabling the ENTIRE §4e refusal gate at once (this also covers 2c: keyless-under-tr, and legacy-wrapper/no-keyed-path) leaves the whole suite green |
| 2c | accept a keyless path under `tr` (same disabled gate as 2b) | composer_shape.go:298-302 | **NONE** — same root cause as 2b, confirmed by the same single mutation |
| 2d | make Back drop the path list | composer_shape.go (whole file) | **NONE (by construction)** — `composerShapeFlow`'s Back-preserving loop is never invoked by any test, so no mutation to it can be caught; not independently re-verified beyond the 0-caller fact, to avoid re-deriving 2b/2c's proof |
| 2e | make the §8a EXPERIMENTAL confirm dismissible with no action (auto-accept a keyless path) | composer_shape.go:222-229 | **NONE — Critical.** `composerAddPath` (where the confirm gate actually lives) has zero test callers. `TestComposerExperimentalConfirmsDrawInFullAndFireOnCondition`'s own doc comment claims "declining it leaves the path list unchanged" but the test body never declines anything — it calls `composerConfirmScreen` directly with a hardcoded body and never touches `composerAddPath` |

### 3. Lock entry (§6b/§8c/§8o/§8t/§8u) — `composer_lock.go`

| # | mutation | plan line(s) | result |
| - | - | - | - |
| 3a-g | 65536 blocks / 389 units / 2008-12-31 / 2027-02-31 / below-pack-bound date / missing lock flag / dropped "cannot tell the time" line — tested as ONE mutation: disable `composerLockAccept`'s two gates (`l.Check()` and `composerLockBelowBound`) entirely | composer_lock.go:148-160 | **NONE — Critical.** `composerLockAccept`/`composerLockEdit` have zero test callers. `TestComposerLockCheckRefusesEverySection4cBoundary` and `TestComposerDateEntryRefusesImpossibleAndPre2009Dates` are real and thorough, but they call `md.Lock.Check()` (the S2 **codec**, out of this plan's scope) and pure parsing helpers directly — never the composer's own UI-to-check wiring. Disabling that wiring wholesale leaves the whole suite green |

The pure functions underneath (`composerParseDateDigits`, `composerDateToUnix`,
`composerDaysToUnits`, `composerLockEcho*`, `composerLockBelowBound`) ARE well
tested individually — the gap is specifically the UI acceptance wiring, not
the arithmetic.

### 4. Hash entry (§6c/§8i) — `composer_hash.go`

| # | mutation | plan line(s) | result |
| - | - | - | - |
| 4a | accept 63 hex characters | composer_hash.go (composerHexEntry, `valid := len(frag) == 64`) | **Not cleanly constructible as an acceptance**: `hex.DecodeString` on an odd-length fragment fails independently, so loosening `valid` to allow 63 doesn't produce a spendable/acceptable hashlock through this path — the safety net is accidental, not because the bound is tested. `composerHexEntry` itself has zero test callers (**Important**, same family as §3/§5/§7-§8) |
| 4b | show a 64-hex row unelided | composer_hash.go:40 (`composerHashRow`) | **CAUGHT** — `TestComposerHashRowIsShortEnoughToDraw` |

### 5. Stub screen (§7c/§8s) — `composer_stub.go`

| # | mutation | plan line(s) | result |
| - | - | - | - |
| 5a | skip the §8s re-show after an edit (`composerFlow`'s `edited = true` → `false`) | composer_flow.go:56 | **NONE — Critical.** `composerFlow` has zero test callers (subsumed by H-1's own file, but this is Part A code, so it's independent of the Part B wiring gap). `TestComposerStubScreenSaysTheIdChangedAfterAnEdit` passes `edited` as a literal argument to `composerStubLines` directly; it never exercises `composerFlow`'s own signal-setting |
| 5b | show "expects a key at" for an already-seated slot | composer_stub.go:84 | **NONE — Critical.** `grep -rn "composerStubLines("` over every `*_test.go` shows **every single call passes `nil` for `keyedChunks`** (5 call sites, 5 nils). The seated-slot branch (`k.FingerprintPresent`) is never taken by any test; disabling it entirely (always "expects a key at") leaves the whole suite green |
| 5c | fixed page-capacity constant too large instead of measured | composer_paged.go (`composerPageLines`) | **Not applicable as stated** — verified good: there is no separate capacity constant to swap in. `composerPageLines` measures per-line height from real glyph metrics (`widget.Labelw`) every frame. `TestComposerStubScreenIsPagedAtItsMeasuredCapacity` drives full-length paging to the end and asserts it doesn't stall — this is a genuine positive |

### 6. Seating (§7d/§8g/§8j/§8k/§8p/§8v) — `composer_sources.go`, `composer_seat.go`, `composer_review.go`, `composer_discard.go`

| # | mutation | plan line(s) | result |
| - | - | - | - |
| 6a | seat the same card twice | composer_seat.go (composerSeatFlow) | **NONE (H-1)** — `composerSeatFlow` has zero callers anywhere, test or production |
| 6b | keep assignments after a wrapper change | composer_discard.go:54-65 (composerApplyShapeEdit) | **NONE (H-1-adjacent).** `composerApplyShapeEdit` has zero test callers AND zero production callers — confirmed by grepping `composer_shape.go`/`composer_flow.go` for `composerDiscardAssignments\|composerApplyShapeEdit\|composerShapeSignature`: no hits. The discard RULE (`composerShapeSignature`, `composerDiscardAssignments`) is well unit-tested in isolation, but nothing in production ever invokes the combination |
| 6c | refuse nothing on two slots sharing an origin with one fingerprint | composer_review.go:45-68 (logic) vs :207 (screen wiring) | **Split result.** The pure logic `composerInvariantViolation` is well tested and CAUGHT the mutation (`TestComposerInvariantRefusesTwoSlotsAtOneOriginWithOneFingerprint/same_origin,_only_one_has_a_fingerprint`). But the real screen, `composerMappingReview`, which is what actually gates progression — **NONE — Critical** when the mutation is applied at the call site instead (`if false && composerInvariantViolation(st)`): whole suite green. `composerMappingReview` has zero test callers |
| 6d | count SOURCES instead of assignable SLOTS in the §8p shortfall | composer_seat.go:138-148 (composerShortfall) vs composer_seat.go:26 (composerAssignableSlots) | **Split result.** `composerAssignableSlots` (the shared helper) is well tested and CAUGHT a direct mutation of its own body. But `composerShortfall` (the real screen) has zero test callers — swapping its call site from `composerAssignableSlots(st)` to `len(st.sources)` **NONE — Critical**, whole suite green |
| 6e | drop the C29 same-seed-same-path warning | composer_review.go:179-181 (composerMappingLines) | **NONE — Critical.** `composerMappingLines` DOES have a direct test caller (`TestComposerMappingLinesPrintOriginsVerbatimAndSayWhatIsNotChecked`), unlike most of this section — but that test never checks for the C29 body's presence. Deleting the C29 loop from `composerMappingLines` leaves the whole suite green, despite `composerSharedSeedInPath`/`composerSharedSeedBody` themselves being well tested in isolation |
| 6f | number the tr internal-key slot last instead of first | composer_sources.go (composerSlotOrder) | **CAUGHT** — `TestComposerSlotOrderAgreesWithTheCodec`, which explicitly constructs both internal-key-first and internal-key-second `PathList` orderings and checks agreement with `md.Compose`'s real slot indices. Genuinely good test |

### 7. Consent (§7e/§8q/§8l) — `composer_selfcheck.go`, `composer_flow.go`

| # | mutation | plan line(s) | result |
| - | - | - | - |
| 7a | skip the self-check (`composerSelfCheck` → always `nil`) | composer_selfcheck.go:65 | **CAUGHT** — 4 sub-tests of `TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput` plus `TestComposerConsentRefusesThroughTheHookAndSaysSection8q`. This is the best-tested surface in the plan |
| 7b | compare against UI state instead of the decoded md1; fault-injection test must fire | composer_selfcheck.go:26,168-171 | **Verified CORRECT and CAUGHT.** `TestComposerConsentRefusesThroughTheHookAndSaysSection8q` drives the real `composerConsentFlow` through the harness with `composerSelfCheckFaultHook` swapping in a genuinely different wallet's chunks, and asserts the refusal reaches the drawn frame — a real, working fault-injection test, not a unit call |
| 7c | render lock kind from UI state instead of decoded payload | composer_consent.go:68 (composerConsentLines) | **Not constructible as stated — verified good.** `composerConsentLines(chunks []string)` takes no `composerState`/UI-state parameter at all; there is no channel for UI state to leak in structurally |
| 7d | drop §8l entirely | composer_flow.go:69-72 | **NONE — Critical.** The comment at composer_flow.go:68 calls this confirm "unskippable, immediately before the first thing that cuts." Deleting the whole block leaves the whole suite green — `composerFlow` has zero test callers |

Note: `composerConsentFlow` itself, despite being genuinely well harness-tested
(7a/7b above), **also has zero production callers** (see H-1) — it is tested
in isolation but structurally unreachable from `walletPolicyFlow`.

### 8. Engrave (§7f) — `composer_engrave.go`, `composer_cards.go`, `composer_census.go`

| # | mutation | plan line(s) | result |
| - | - | - | - |
| 8a | offer form A (words) for a keyless composition | composer_engrave.go:41-63 (composerFormsFor) | **NONE (H-1 + H-2).** No test file exists for this code at all (H-2); the function also has zero production callers |
| 8b | mint cards with the template stub before seating is complete | composer_cards.go | **NONE (H-2).** `composer_cards_test.go` does not exist |
| 8c | count plates without card chunks | composer_census.go (reuses `buildPlateCensusLines`) | **NONE (H-1).** `composerCensusLines` has zero callers anywhere, test or production |
| 8d | cut a multi-slot seed's card twice | composer_cards.go | **NONE (H-2)** — no test file |
| 8e | skip the ceiling refusal | composer_census.go:83-90 (composerCensusRefusal) | **NONE — worse than untested.** `composerCensusRefusal`/`composerDescriptorPlateFits` are never called from ANY production code path (`grep` for both names over non-test `.go`, excluding their own declarations, returns nothing but their own bodies). The ceiling-refusal machinery is correctly implemented (a real render-and-reject search, matching `qrCeilingBytes`'s pattern) but is wired to nothing — an oversized concrete descriptor has no refusal to skip because nothing ever calls the refusal |

### 9. Copy gates — `composer_copy.go` / `composer_copy_test.go`

- 39 `composerCopy*` functions declared, all 39 present in `composerCopyTable`, verified by running `TestComposerCopyTableCoversEveryBody` (PASS) and cross-checking the AST-extracted function list against the plan's own table (exact match, both 39).
- `TestComposerCopyIsVerbatimFromTheSpec` and `TestComposerCopyIsDrawable` both PASS against the wired tree (re-run, not assumed).
- **A 1-character mutation to any of the 39 body strings is caught by construction** (`TestComposerCopyIsVerbatimFromTheSpec` does exact-string comparison over the whole table) — verified for one sample (`composerCopyOwnWallet`, one char changed → immediate FAIL, reverted).
- §12 item 5's FOURTH gate (fires-on-condition) is where the real gaps are — not in this file, but in the callers, per the mutation table above. Of the 39, the ones with confirmed fires-on-condition coverage via a REAL caller-level test: §8a/§8b (shape confirms, though 8a's own confirm-bypass escapes at the `composerAddPath` gating level — see 2e), all six §8r door lines (`TestComposerDoorLinesCoverEveryKeyState`), §8c echoes and §8o/§8t/§8u (pure-function level only, per §3 above), §8i (`TestComposerHashRuleIsStatedAtEntry`), §8h (`TestComposerEveryPathHashedWarns`), §8v (`TestComposerInvariantRefusesTwoSlotsAtOneOriginWithOneFingerprint`, logic-level only per 6c), §8g (`TestComposerC29WarningFiresInsideOnePathAndNotAcross`, logic-level only per 6e), §8m refusals (logic-level only, `TestComposerShapeRefusalsActuallyRefuse`, calls `md.ValidatePathList` directly rather than through `composerShapeFlow`), §8p (`TestComposerAssignableSlotsCountsSeatsNotSources`, logic-level only per 6d), §8q/§8l (`composerConsentFlow`, genuinely screen-level — see §7), §8s id-changed (logic-level only per 5a), §8f/§8d/§8k — present in tables, not independently re-verified for fires-on-condition beyond the grep-based call-count pass.
- None of the 39 bodies is completely unreferenced outside the copy table itself — the coverage-table mechanism (gates 1–3 of §12 item 5) works as designed. Gate 4 (fires-on-condition) is where roughly half the table's rows have only a pure-logic-level test rather than a screen-level one, consistent with the pattern above.

## Expected-line audit

Every checkable `Expected:` line I could run against the wired tree came back
true: `TestComposerCopyTableCoversEveryBody` (39, PASS), the §13
measurement test's output (re-run: `stub_screen lines=42 per_frame=7 pages=6`,
`pick_list lines=36 per_frame=7 pages=6`, `consent lines=15 per_frame=7
pages=3`, `descriptor_plate ceiling_chars=596 c10_688_fits=false` — all
produced by the actual command, not transcribed), and the plan's own §12
item 5 gate counts. `Expected: FAIL to build` lines (Task A1 Step 2 and
equivalents elsewhere) are process instructions for a tree that doesn't yet
have the code — not falsifiable against the already-wired tree and not
treated as findings.

## §13 item 1 measurement task (Task C1)

Re-ran `go test -run '^TestComposerMeasureSection13Numbers$' -v ./gui/` in
the scratch: produces real `SPEC13` lines (quoted above), confirming the
mechanism works. The task correctly **describes** producing the numbers (its
own Step 2 says "paste them verbatim... do not write a number this command
did not print") rather than asserting fabricated numbers as already-measured
fact — this is the correct posture for an unexecuted plan step, not a defect.
Checked `design/SPEC_wallet_policy_composer.md:929-931`: still says "the same
kind of plan-time render measurement" (unfolded), consistent with
implementation not having started. No finding here.

## Harness-coverage table

`Y` = a `synctest.Test`+`runUI`/`runUITouchRaster`+`click`/`pumpUntil` test
drives the real screen function. `logic-only` = the screen function itself is
never called by any test; only pure helpers underneath are.

| screen / function | harness-driven test | notes |
| - | - | - |
| `composerDoorFlow` | Y | draws + labels only; never clicks past the door (C-2 above) |
| `composerShapeFlow`/`composerAddPath`/`composerPathEdit`/`composerKeysEdit` | **NONE** | zero test callers of any kind |
| `composerWrapperPick` | **NONE** | zero test callers |
| `composerLockEdit`/`composerLockAccept` | **NONE** | zero test callers; pure helpers below are well tested |
| `composerHashEdit`/`composerHexEntry` | **NONE** | zero test callers |
| `composerDigitEntry` | Y | 3 tests incl. Back-behavior; genuinely solid |
| `composerStubFlow` | Y (partial) | first-frame draw + label only; never with `keyedChunks` set, never navigates pages via clicks |
| `composerPickScreen` (generic picker) | Y | tested directly with synthetic rows, not via `composerSeatFlow` |
| `composerSeatFlow` | **NONE** | zero callers anywhere (H-1) |
| `composerMappingReview` | **NONE** | zero callers anywhere (H-1) |
| `composerConsentFlow` | Y | best-tested screen in the plan — but zero PRODUCTION callers (H-1) |
| `composerFormPick`/`composerSecretFormPick`/`composerEngraveModePick` | **NONE** | zero callers anywhere (H-1) |
| `composerFlow` (Part A, the only flow file that exists) | **NONE** | zero test callers |
| card minting (`composerMintCard(s)`) | **NONE** | no test file exists (H-2) |
| census (`composerCensusLines`/`Refusal`) | **NONE** | zero callers anywhere, including production (8e) |

## Findings

**C-1 (Critical).** Part B — seating, mapping review, consent flow, engrave
form pick, the discard rule, the census — has no caller into the program.
Plan line 4437 promises a `Replace \`gui/composer_flow.go\`` task that is
never written. Every Part B screen is structurally unreachable by any test
this plan specifies. Subsumes mutations 6a, 6b, 7d(shares cause though Part
A), 8a, 8c.

**C-2 (Critical).** Tasks B7 and B8 each name a test file in their own
"Files:" header (`composer_engrave_test.go`, `composer_cards_test.go`) that
is never written — confirmed by grep (each filename appears exactly once,
in its own task header) and by absence from the wired tree. Card minting's
`mk.Decode` round-trip and the engrave-form choice ship with zero tests,
below even the plan's own stated bar. Subsumes 8a (test-file half), 8b, 8d.

**C-3 (Critical).** Task B10 (§12 item 6, the seating-vector leg) is five
prose bullets with no code; `composer_seating_vectors_test.go` does not
exist. The one test meant to prove seated cards round-trip through the
shipped `seatKeyCards` consumer to the keyed policy's real addresses is
absent.

**C-4 (Critical).** Door: hiding "From payload" for a payload holding an
md1/mk1 (removing the `ClassMDMK` branch of `composerDoorHasConsumablePolicy`)
is a false PASS. Mutation 1b.

**C-5 (Critical).** Door: no test anywhere clicks "Build a new policy";
retargeting its route to `composerRouteScan` is a false PASS across the whole
`gui/` package, not just `TestComposer*`. The test named as if it walks this
path (`TestComposerNoPayloadWalkReachesAKeylessTemplateThatDecodes`) stops at
the door. Mutation 1c.

**C-6 (Critical).** Shape: `composerShapeFlow`'s entire §4e refusal gate
(covering lock-only paths, keyless-under-tr, legacy-wrapper shapes, and
no-keyed-path shapes at once) can be disabled with the whole suite green.
Mutations 2b, 2c.

**C-7 (Critical).** Shape: the §8a EXPERIMENTAL keyless-path confirm can be
bypassed with no operator action inside `composerAddPath`; the test that
claims to cover this ("declining it leaves the path list unchanged") never
exercises decline at all. Mutation 2e.

**C-8 (Critical).** Lock: `composerLockAccept`'s bound checks can be disabled
wholesale (accepting 65536 blocks, any date, any impossible date) with the
whole suite green; only the underlying codec method and pure parsers are
tested directly. Mutations 3a-3g.

**C-9 (Critical).** Stub: the §8s re-show-after-edit signal
(`composerFlow`'s `edited` variable) can be inverted with the whole suite
green. Mutation 5a.

**C-10 (Critical).** Stub: every test call to `composerStubLines` passes
`nil` for `keyedChunks`; the seated-slot labelling branch is completely
unexercised, and disabling it (always "expects a key at" even when seated)
is a false PASS. Mutation 5b.

**C-11 (Critical).** Seating: `composerMappingReview` (the real screen)
can have its §4f invariant check disabled at the call site with the whole
suite green, even though the underlying `composerInvariantViolation` logic
is itself well tested. This is the funds-safety property the code's own
comment names ("a misassignment does not fail, it derives a different
wallet's address and shows it to the operator as proof"). Mutation 6c
(screen-wiring layer).

**C-12 (Critical).** Seating: `composerShortfall` (the real §8p screen) can
have its count swapped from seats to sources at the call site with the whole
suite green, even though the shared `composerAssignableSlots` helper is
itself well tested. Mutation 6d (screen-wiring layer).

**C-13 (Critical).** Seating: the C29 same-seed-same-path warning can be
deleted from `composerMappingLines`' actual output with the whole suite
green, despite that function having a direct test caller — the caller just
never checks for it. Mutation 6e.

**C-14 (Critical).** Consent: §8l, commented "unskippable, immediately
before the first thing that cuts," can be deleted outright from
`composerFlow` with the whole suite green. Mutation 7d.

**C-15 (Critical).** Engrave: the descriptor-plate ceiling refusal
(`composerCensusRefusal`) is not called from any production code path at
all — worse than untested, it is unreachable even by an operator who tries
to trigger it, because the concrete-descriptor engrave path never calls it.
Mutation 8e.

**I-1 (Important).** `composerHexEntry`'s exact-64-character bound has no
test of any kind; the brief's specific "accept 63 hex" mutation happens to
be caught by an unrelated safety net (`hex.DecodeString` on odd length),
which is not the same as the bound being tested. Mutation 4a.

## Verified correct (no finding)

- Shape picker bound (33rd slot), hash-row elision (64 hex unelided),
  invariant-violation pure logic, slot-order-vs-codec agreement (internal
  key), assignable-slots seats-not-sources logic — all CAUGHT cleanly.
  Mutations 2a, 4b, 6c(logic), 6f, 6d(logic).
- The self-check / consent-flow fault-injection test
  (`TestComposerConsentRefusesThroughTheHookAndSaysSection8q`) is a real,
  working, screen-level fault-injection test — the best-tested surface in
  the plan. Mutations 7a, 7b.
- `composerConsentLines` takes no UI-state parameter at all, so 7c's
  mutation has no channel to exist through — verified by signature, not
  argued from absence of a test.
- Paging capacity is measured from real glyph metrics every frame, not a
  swappable constant; full-length paging proven not to stall. Mutation 5c.
- All 39 `composerCopy*` bodies present, verbatim-correct, and 1-character
  changes are caught by construction (exact-string comparison table).
- §13 item 1's measurement task correctly defers to a real command rather
  than asserting fabricated numbers; independently re-run with matching
  live output.

## Closing counts

- Mutations attempted: 27 (all brief-listed items across categories 1–8,
  plus 3 additional wiring-layer variants on 6c/6d to separate logic
  coverage from screen coverage).
- CAUGHT: 8 (1a, 2a, 4b, 6f, 6c-logic, 6d-logic, 7a, 7b).
- NONE / false PASS: 19.
- Findings: **15 Critical, 1 Important, 0 Minor recorded separately** (Minor/Nit
  items folded into the "verified correct" section above where they were
  positive, per "do not pad").
- Two of the fifteen Criticals (C-2, C-3) are missing-test-file defects
  discovered by direct enumeration, not by mutation (nothing to mutate when
  no test exists).

This plan does not close at 0 Critical / 0 Important under the tests lens.
