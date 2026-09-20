# Composer fable review r0 — Lens 4: device flow, adversarial

Reviewer: fable-tier, independent. Tip reviewed: SeedHammer fork `main`
`f5b068faf3049ccf97603dfbaa3709f12893df97` (detached worktree `fable-flow`,
removed at the end). Date: 2026-09-19. Method: the Go test harness as the
scriptable device; every claim below was measured by a walk or unit test whose
RED/GREEN is recorded in "What I ran". Read-only on every repo; nothing
committed.

## Verdict

**0 C / 2 I / 6 M / 0 N.** Through the taps the SH2's buttons allow I could
not make the review, consent or census state one policy while the plate
planner was handed another: the keyed chunks, the template and the minted mk1
cards that reached the engraver after a key -> seed -> Back re-seat carried
exactly what the mapping review showed, and the consent's chunks are the ones
cut. What I did find is two moments where the device **silently substitutes a
different policy for the one the operator confirmed** and then shows the
substitute truthfully: a hold-confirmed "Keep my order" is reversed to
sortedmulti by a routine Back + Done + forward (I-1), and Back inside the
passphrase entry seats the seed **without** its passphrase (I-2). Both are
DEFAULTs at a moment where the wrong outcome is worse than silence. Six Minors
follow the same "picker proposes a setting" and wrong-body-refusal classes.

## Findings

### I-1 — A hold-confirmed "Keep my order" is reversed by Back + Done + forward; the wallet becomes sortedmulti with only a generic banner

**Inputs (taps).** `Build a new policy` -> `Segwit (wsh)` -> `Build my own
paths` -> `Add a spend path` -> Keys -> 3 -> 2 -> `Done` -> "Sorted keys, or
your order?" -> `Keep my order` -> §8b `UNSORTED KEYS (EXPERIMENTAL)`, **held
to confirm** -> Template screen -> `Back` (to re-read the list) -> `Done` ->
"Sorted keys, or your order?" is asked again -> forward button (the control
that has advanced every other screen), without moving the cursor.

**Observed** (`TestFableSpecKeyOrderSurvivesBackFromTheStubScreen`, RED at
tip): the picker reopened on row 0, `Sorted (usual)`; §8b did **not** re-fire;
the Template screen re-drew with `The shape changed, so this id changed` (true,
unspecific); the consent showed no `UNSORTED (EXPERIMENTAL)` line; the chunks
the consent was rendered from — the set handed to the engrave step — decode
with `Branches[0].Sorted == true`:
`md1f6frtqq9qjtvyyy5jmpprjjtvyy49gqpsgwzyxqqzqw55nlnzfdw3`.

**Expected and why.** §7b: "Back preserves everything ('going back should lose
nothing')"; §8b fires "once per key set where sorted was legal and declined" —
the operator declined it and the decision was reversed without a decline
being withdrawn. Journey C-1 (fixed for the script picker, F-527) is this
class: "a picker that opens on row zero proposes a setting". A `multi()`
wallet and a `sortedmulti()` wallet over the same keys are different wallets
unless the keys happen to be in lexicographic order; an operator matching an
existing `multi` descriptor gets other addresses.

**Where.** `gui/composer_shape.go:300-323` (`composerKeyOrderStep`): the
`ChoiceScreen` at 304-308 sets no `Initial`, and the answer is written at
314/321 on every pass. The only signal is `composerCopyIdChanged` on the stub
screen, which names no cause; the consent's mark is absence-only
(`gui/composer_consent.go:137-139`).

**Class.** DEFAULT. Worse than silence: yes — the operator confirmed the
opposite under a hold. **Severity: Important.** (Not Critical: the consent and
the plate agree; the direction of the flip is toward the usual form.)

**Proof the test is not vacuous.** F1 (open the picker on the setting in
force, `Initial: 1` when `!Sorted`) turns it GREEN: §8b re-fires, no banner,
consent carries `UNSORTED`, decoded `Sorted=false`.

### I-2 — Back inside the passphrase entry seats the seed WITHOUT its passphrase

**Inputs (taps).** Payload with two `key:` records (the fixtures). wsh,
1-of-2, `Done`, Sorted, Template -> `Seat keys` @0 -> `Type a seed` -> 12
words (the abandon vector) -> "Add a BIP-39 passphrase?" -> `Add passphrase`
-> the keyboard (title `Passphrase seed 1`) -> type -> **Back** (to correct
something).

**Observed** (`TestFableSpecBackOnThePassphraseKeyboardIsADecline`, RED at
tip): the device drew the seat prompt
`Slot @0, Path 1 key 1 of 2: choose a key` with a new row `seed 1 (any
slots)`; seating it and `K2` at @1 gave a mapping review of
`@0: 73c5da0a m/48'/0'/0'/2'  @1: 73c5da0a m/48'/0'/1'/2'` — the **bare**
seed's fingerprint. With the passphrase the operator had started typing (`x`)
the master fingerprint is `1d39a522`: a different wallet, different keys,
different addresses. The same happens for Back on the "Add a BIP-39
passphrase?" question itself.

**Expected and why.** "Back is a decline everywhere on this device"
(`gui/composer_digitpad.go:57-60`), and §7b's rule. Back one screen should
return to the passphrase question, or drop the seed; it must not register the
seed as a source with the passphrase silently omitted. The engrave-mode label
then reads plain `Full (seed + keys)` (no passphrase registered), so nothing
downstream names the missing factor.

**Where.** `gui/composer_sources.go:258-270`:
`if sel, ok := pp.Choose(ctx, th); ok && sel == 1 { if pass, ok :=
syswPassphraseFlowTitled(...); ok { bind } }` — both `!ok` legs fall through
to `return composerSource{...}, true` at 274. The registry keeps the seed
(`st.reg.add` at 253 runs before the question).

**Class.** DEFAULT at a Back edge. Worse than silence: yes. **Severity:
Important.** Out of lens: `gui/multisig_build.go:775-787` (`buildSeedForSlot`)
has the identical shape, so Multisig Build shares it.

**Proof.** F2 (loop: Back on the keyboard re-asks the question; Back on the
question declines the seed) turns the test GREEN.

### M-1 — The Keys editor reopens on n = 1, k = 1, so "look and leave" rewrites a 2-of-3 to one key

Taps: `Path 1: 2-of-3` -> `Keys` -> forward -> forward. Observed
(`TestFableSpecKeysEditorShowsTheKeySetInForce`, RED): the menu now reads
`Path 1: 1 key`. `composerCountPick` (`gui/composer_shape.go:235-248`) always
opens on `min`; `composerKeysEdit` (268, 272) passes no current value.
Journey C-1's class again. Minor rather than Important because the row, the
stub banner and the consent all report the new shape, and §8j holds the door
when a slot is seated. Class DEFAULT. F4 (open both pickers on the value in
force) turns it GREEN.

### M-2 — "Full (seed + keys)" is asked for a seed that seats nothing, and cuts no seed

Taps: type a seed at @0, `Skip` the passphrase, then seat `K1` and `K2` from
the records instead; form A. Observed
(`TestFableSpecEngraveModeIsAskedOnlyForSeedDerivedSlots`, RED): `Engrave
Mode — What to engrave? Full (seed + keys) / Watch-only (keys)` is asked;
choosing Full, the census reads `This engraves 1 plate. md1 policy: 1 plate`
— no ms1. §7f scopes the question to "seed-derived slots".
`gui/composer_flow.go:407` gates it on `st.reg.count() > 0`, the registry of
every seed ever typed in the flow, while `composerSecretCards` (550-579) cuts
only seated seeds. The census corrects the promise, so not worse than
silence; a question whose answer changes nothing. Class DEFAULT. F5 turns it
GREEN.

### M-3 — One seed typed at two prompts is cut twice, byte-identical

`TestFableSpecOneSeedTypedTwiceIsCutOnce` (RED): the same mnemonic
registered twice (two `Type a seed` answers, as an operator filling two slots
does), seated at @0 and @1, derives accounts 0 and 1 correctly
(`composerSeedAccountFor` keys on the master fingerprint,
`gui/composer_sources.go:291-305`) but `composerSecretCards` dedups by
`seedID` (`gui/composer_flow.go:552-561`) and plans **two** ms1 plates with
identical strings, labelled `ms1 secret share 1` and `2`. §7f: "A seed that
filled several slots is cut ONCE." Two bearer plates of one secret, and a
census that counts two shares. Positive control in the same test: one
registration at two slots is cut once. F6 (dedup by `MasterFP`) turns it
GREEN.

### M-4 — 2009-01-01 and 2009-01-02 are refused with the 2038 ceiling body

`TestFableSpecDateBelowTheFloorInsideTwoThousandNineNamesTheFloor` (RED):
`composerDateBandEcho("20090102")` returns `This build writes dates up to
2038-01-19. For a later time, use a block height instead.` §6b/§8t: every date
before 2009-01-03 is refused with the floor line. `gui/composer_lock.go:327`
tests `y < 2009`, so the two calendar days inside 2009 but below the floor
fall to the ceiling arm at 334. A refusal with the wrong reason; no plate.
Class REFUSAL (wrong body). F3 (compare the Unix time to
`composerDateFloorUnix`) turns it GREEN.

### M-5 — A key-less path cleared to empty is refused as a lock-only path

Reachable via `Path N` -> `Hash lock` -> `No hash lock` on a key-less path
(`gui/composer_hash.go:755-757`; the §8j probe asks first, then discards).
The row reads `Path 2: empty` (`gui/composer_state.go:415-421`) and `Done`
refuses with `A path with only a time lock means anyone can spend after it.
Add a key or a hash.` (`gui/composer_shape.go:43-44` maps
`ErrComposeLockOnlyPath`, which `md.ValidatePathList` returns for a path with
neither keys nor hash). `composerAddPath` (377-384) already removes such a
path when it is CREATED empty, for exactly this reason; the edit route does
not. Spec §4e row 2 maps "neither keys nor hash" to that body, so this is
spec copy as much as code. Documented by
`TestFableEmptiedKeylessPathIsRefusedWithTheLockOnlyBody` (PASS; mutation M4
RED). Class REFUSAL (wrong-true body).

### M-6 — A testnet `tpub` in a `key:` record is admitted as a mainnet key, silently, on host and device

`TestFableTestnetXpubKeyRecord`: `sysw.ParseKeyRecord` accepts
`[73c5da0a/48'/1'/0'/2']tpub...` (`sysw/composer_records.go:395-407` checks
depth and child, never the version) and `Classify` returns `ClassKey`, so the
door counts it and seating offers it. The host packs it too: `me sysw pack
--no-passphrase --in tpub.rec --out tpub.bin` wrote 349 bytes without a word.
Downstream: the keyed md1 carries only the 65 key bytes, so the consent
prints mainnet `bc1q` addresses for material derived under coin `1'`; the
mk1 card the composer mints carries the composer's `Network: "mainnet"` label
(`gui/composer_cards.go:42`) but `mk.Encode` serialises the tpub's version
bytes (`mk/encode.go:167-171`), and `mk.Decode` of that card reports
`Network="testnet"`. §4f: "complex-policy derivation is mainnet-only by
construction". Class DEFAULT. Severity Minor: the key is a real key at the
declared origin, so nothing is unspendable, but a testnet account is mixed
into a mainnet policy with no notice on either side. Cross-lens (host records
are lens 1/2).

## Divergence table

| sequence (taps) | screen promise | what reached the planner | class | worse than silence? | severity |
| --- | --- | --- | --- | --- | --- |
| wsh 2-of-3, Keep my order (hold), Template, Back, Done, forward | "Keep my order", §8b confirmed | sortedmulti chunks `md1f6frtqq9...`; §8b silent; banner "shape changed"; consent has no UNSORTED mark | DEFAULT | yes | **I-1** |
| Type a seed, Add passphrase, type, Back | Back is a decline | bare seed as `seed 1` source; seated: fp `73c5da0a` (passphrased: `1d39a522`) | DEFAULT | yes | **I-2** |
| Path 1 (2-of-3) -> Keys -> forward, forward | a picker shows the setting | `Path 1: 1 key` (row, banner and consent say so; §8j when seated) | DEFAULT | borderline | M-1 |
| seed typed then unused; form A; Full | "Full (seed + keys)" | 1 plate, md1 policy; census says so | DEFAULT | no | M-2 |
| same words at two prompts, seated @0/@1, Full | "cut ONCE" | two identical ms1 plates, "share 1", "share 2" | DEFAULT | marginal | M-3 |
| date pad `20090102` | §8t floor | refused with the 2038 ceiling body | REFUSAL, wrong body | no plate | M-4 |
| key-less path -> Hash lock -> No hash lock -> Done | row "Path 2: empty" | refused: "only a time lock…" | REFUSAL, wrong-true body | no plate | M-5 |
| `key:` record with a tpub (host packs it) | "Keys loaded: 1"; §4f mainnet-only | seated; bc1q addresses from a coin-1' key; minted mk1 decodes `Network=testnet` | DEFAULT | no | M-6 |
| seat K1@0, Back at @1, Type a seed, seed@0, K2@1, form B watch-only, cut all | mapping review `@0: 73c5da0a m/48'/0'/0'/2' @1: 73c5da0a m/48'/0'/1'/2'`, §8g on page 2 | md1 template: 2 slots, both fp 73c5da0a, origins 0h/2h and 1h/2h, no xpub; mk1 cards: xpubA@0h, xpubB@1h, both stubs; consent chunks: xpubA, xpubB | — | — | verified correct |
| key-less path, EXPERIMENTAL door: tap; then Back | hold-only; Back returns to the shape | tap leaves the door up; Back: list `slots: 0`, no path | WARNING | — | verified correct |
| `key:` record with CRLF / LF / leading or trailing space / upper-case hex / `KEY:` | inert | all `ClassUnknown` | REFUSAL (inert) | — | verified correct |
| pads: blocks 0 / 00012 / 65535 / 65536; days 0 / 036 / 388 / 389; height 0 / 499999999 / 500000000; date 20270230 / 19851105 / 20090103 / 20380119 | §6b bands | 0 -> ceiling body (F-526 M-6, known); 00012 -> "12 blocks"; 65535 ok; 65536 refused; 036 -> "36 days = 6075 units"; 388 -> 65475 units; 389 refused; 499999999 ok; 500000000 refused; "does not exist"; floor; ok; ok. Every edge operand passes `md.Lock.Check`. `older(0x400000)` (zero units) is unreachable: days >= 1 gives >= 169 units, and `md/compose.go:111-113` refuses 0 | DEFAULT / REFUSAL | — | measured |
| consent: CONTINUE on page 1 of 3 | checkmark withheld until the last page | withheld (existing gate; mutation M6 RED) | REFUSAL | — | verified |
| reset / power loss mid-flow | — | `composerState` is a local of `composerFlow` (`gui/composer_flow.go:50`); no flash write in `gui/composer_*.go`; scrubbed by the flow-exit defer; nothing is resumed; a set cut halfway has no on-device record and the restore document is shown only on `done` (486-489) | DOCUMENTATION | — | read |
| Back at a "Preimage plate" pick | — | means "do not cut this preimage" for that digest (`gui/composer_preimage_plate.go:156-190`); the census names it | DOCUMENTATION | — | read |

## State-audit table — Back / Cancel edges

| edge | guard it should pass through | does it | stale state left |
| --- | --- | --- | --- |
| wrapper picker, first pass, Back | none (opening screen) | leaves the composer; composition dropped (F-519, open) | none: `composerFlowExit` scrubs |
| "Start from?" Back | — | wrapper picker (`composer_flow.go:209-213`); second pass opens on the unapplied pick (preselect review M-1, accepted) | none |
| path list Back | §8j if a preset/wrapper is then picked | preset screen with the list intact; a change there goes through `composerShapeGuard` + `composerApplyShapeEdit` (219-226) | none (`TestComposerBackLegPresetAsksBeforeDiscardingSeats`, existing) |
| Keys editor Back (n or k) | snapshot | `before` restored (`composer_shape.go:427-432`) | none |
| Lock editor Back (kind/unit/pad/echo) | — | lock unchanged (`composer_lock.go:189-267`); a below-bound refusal also leaves the OLD lock (F-525 shape) | none |
| Hash editor Back (Which hash / kind / pad / phrase) | — | unchanged; pad draft and kind kept (`composer_hash.go:729-754`) | `hashlockHeld` keeps orphaned material by design; listed at Done, never cut |
| Add path Back at any step | — | path truncated (`composer_shape.go:341-384`); verified (`TestFableKeylessDoorIsHoldOnlyAndBackReturnsToTheShape`, M7 RED) | none |
| Done -> Key order Back | — | path list; `Sorted` holds the last pass's value | **forward reopens on row 0: I-1** |
| stub screen Back | — | path list (`composer_flow.go:139-140`); `shown`/`seen` recorded on both legs only when drawn (134-137) | none; the banner compares id AND advertised origins against memory (F-520 fix; `TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit`, existing) |
| seat prompt Back at slot i > 0 | release | releases i-1 and re-offers its source (`composer_seat.go:128-137`); verified in the planner walk (M2 RED) | none |
| seat prompt Back at slot 0 | — | leaves seating to the path list; after a resume, higher seats are kept | none |
| seed source picker / words Back | — | nothing registered (`composer_sources.go:244-246`) | none |
| passphrase question Back; passphrase keyboard Back | decline | **treated as Skip: I-2** | seed registered; a seed abandoned here still triggers the Full/Watch-only question (M-2) |
| shortfall "Back to the paths" | — | path list, partial seats kept (`composer_seat.go:191-197`) | none |
| mapping review Back | — | releases the highest seated slot and re-asks it (`composer_flow.go:359-366`); §8g/§8k recomputed on every entry (`composer_review.go:183-208`) | none |
| keyed stub screen Back | — | path list, not the mapping review (`composer_flow.go:151-153`) | none (documentation) |
| consent Back; §8l Back | — | path list (159-161); seats kept | none |
| form pick Back; engrave-mode Back | — | path list (373-376, 408-411) | none |
| preimage pick Back | — | declines that plate, continues (`composer_preimage_plate.go:170-190`) | none; census lists it |
| census Back | — | path list (445-448); preimage decisions are re-asked on the next pass | none |
| bundleEngrave Back | set-level abort | "Bundle Incomplete", Aborted -> path list; a preimage plate already cut stays cut and `composerAbortPreimageCut` says so (`composer_flow.go:483-485`) | none |

No Back edge bypasses §8j: every shape edit is routed through
`composerShapeGuard`/`composerApplyShapeEdit` or discards unconditionally
(`composerMoveUp`), including the W-6/W-7 Back leg. The stale state the
audit found is the seed registry (I-2, M-2) and the key-order answer (I-1).

## Self-check versus census (hunt item)

`composerSelfCheck` (`gui/composer_selfcheck.go:65-216`) runs on the
`consent` chunks — `keyed` when every slot is seated, else `template`
(`composer_flow.go:155-159`) — and compares them to `st.list` and
`st.assigned`. The census (`composerCensusLines`) is derived through
`bundlePlatePlan` from the same `template`/`keyed` slices plus cards minted
from `st.assigned` (`composer_cards.go:59-77`). Form B cuts `template`, which
is composed from the same declared origins as `keyed` by two `ComposeWith`
calls with identical inputs (`composer_flow.go:280-318`); I found no operator
input that makes the checked set and the cut set differ, and the planner walk
confirms the cut cards carry the reviewed assignments. What the self-check
does not cover is stated in its header: the key bytes, which the consent's
addresses cover and which the walk confirmed slot by slot.

## What I ran

Toolchain: Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go`, `CGO_ENABLED=0`,
`TMPDIR=/scratch/code/shibboleth/.tmp`. Worktree
`/scratch/code/shibboleth/.tmp/fable-flow` at the tip, removed afterwards
(`git worktree list | grep -c fable-flow` = 0). Artifacts (test file, run
logs, mutation driver, tpub payload) are in
`/scratch/code/shibboleth/.tmp/fable-flow-out/` and the test file is Appendix A.

**New test file:** `gui/fable_flow_walk_test.go` (916 lines, gofmt clean;
`go vet ./gui/` reports only two pre-existing go1.25/1.26 `ArtifactDir` notes
in unrelated goldens).

`go test ./gui/ -run '^TestFable' -v -count=1` — 12 tests ran, 6 FAIL / 6
PASS, 3.6 s (log `fable-r2.txt`):

| test | at tip | what it drives |
| --- | --- | --- |
| TestFableSpecKeyOrderSurvivesBackFromTheStubScreen | FAIL (I-1) | keyless wsh 2-of-3; §8b hold; Back; Done; forward; consent chunks captured through `composerSelfCheckFaultHook` as a read seam |
| TestFableSpecBackOnThePassphraseKeyboardIsADecline | FAIL (I-2) | seat @0 by a typed seed, Add passphrase, Back; then seats the offered bare seed and reads the mapping review |
| TestFableSpecDateBelowTheFloorInsideTwoThousandNineNamesTheFloor | FAIL (M-4) | `composerDateBandEcho` at 20081231 / 20090101 / 20090102 / 20380120, and both floor and ceiling accepted |
| TestFableEmptiedKeylessPathIsRefusedWithTheLockOnlyBody | PASS (documents M-5) | row text and §8m body for an empty path |
| TestFableSpecKeysEditorShowsTheKeySetInForce | FAIL (M-1) | Path 1 -> Keys -> forward, forward |
| TestFableSpecEngraveModeIsAskedOnlyForSeedDerivedSlots | FAIL (M-2) | typed seed unused; form A; Full; census |
| TestFableSpecOneSeedTypedTwiceIsCutOnce | FAIL (M-3) | two registrations, two accounts, `composerSecretCards`; positive control passes |
| TestFableKeylessDoorIsHoldOnlyAndBackReturnsToTheShape | PASS | tap on §8a leaves it up; Back removes the path |
| TestFableDigitPadBands | PASS | the 15-row band table above; edge operands through `md.Lock.Check` |
| TestFableTestnetXpubKeyRecord | PASS (documents M-6) | tpub record parse/classify; card encode/decode |
| TestFableRecordSpellingsAreInert | PASS | six spellings -> `ClassUnknown` |
| TestFableKeyThenSeedThenBackReachesThePlannerAsShown | PASS | key -> Back -> seed re-seat; form B watch-only; 3 plates cut through `NewEngraveScreen` on the raster harness; engraved strings captured by `engravedAwarePlatform` and decoded (template, 2 cards, consent chunks) |

**Mutations** (driver `mutate.py`; each applied to one production file, the
named test run with `-run '^Name$' -v`, `=== RUN` confirmed, then `git
checkout --` restored; `git status --porcelain` afterwards showed only the
untracked test file):

| # | mutation (file, edit) | test | result |
| --- | --- | --- | --- |
| M1 | `gui/composer_cards.go`: `a := st.assigned[slot]` -> `st.assigned[(int(slot)+1)%len(st.assigned)]` | planner walk | RED: `card 0: path m/48h/0h/1h/2h ... want m/48h/0h/0h/2h` (and card 1) |
| M2 | `gui/composer_seat.go` `composerReleaseSeat`: `used = false` -> `used = true` | planner walk | RED: `K1 was not released back onto the list` |
| M3 | `sysw/composer_records.go` `unhexLower`: also accept `A-F` | RecordSpellings | RED: `upper-case hex: classified as a key` |
| M4 | `gui/composer_state.go`: `"empty"` -> `"hash only"` | EmptiedKeylessPath | RED: `row reads "Path 2: hash only"` |
| M5 | `gui/composer_lock.go` blocks band: `65535` -> `65536` | DigitPadBands | RED: `65536 blocks accepted` |
| M6 | `gui/composer_paged.go`: `if seenEnd && contBtn.Clicked` -> `if contBtn.Clicked` | TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage (existing) | RED: `a 64-line consent was confirmed from its FIRST page` |
| M7 | `gui/composer_shape.go` `composerAddPath`: drop the truncation on §8a decline | KeylessDoor | RED: `Back at the EXPERIMENTAL door left a path behind: "...Path1:empty..."` |
| F1 | `composerKeyOrderStep`: `Initial: 1` when `!Sorted` | KeyOrder | GREEN (`§8b re-fired=true, stub banner=false, UNSORTED mark=true, Sorted=false`) |
| F2 | `composerSeedSource`: loop; keyboard Back re-asks, question Back declines | Passphrase | GREEN |
| F3 | `composerDateBandEcho`: compare `time.Date(...).Unix()` to the floor | Date | GREEN |
| F4 | `composerCountPick` takes `initial`; `composerKeysEdit` passes n, k in force | KeysEditor | GREEN |
| F5 | `composerEngraveStep`: ask only when a seated slot is seed-derived | EngraveMode | GREEN |
| F6 | `composerSecretCards`: dedup by `MasterFP` | SeedTwice | GREEN |

**Existing gates run with -v** (all PASS): `TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage`,
`TestComposerPickScreenNeverReturnsARowItDidNotDraw`,
`TestComposerMeasureSection13Numbers`,
`TestComposerPagedLinesNeverDrawUnderTheNavButtons`,
`TestComposerPagedGeometryProbeCanSeeInk`.

**Host:** `ms 0.19.0`: `ms derive --template bip48-p2wsh --account 0
--network testnet --phrase "<abandon x11 about>" --allow-argv-secret` ->
`tpubDFH9dgzveyD8zTbP...` (111 chars); `me 0.10.0`: `me sysw pack
--no-passphrase --in tpub.rec --out tpub.bin` -> packed, 349 bytes, appended
`now:`; no refusal.

Prior findings re-checked and NOT re-filed: F-518, F-519, F-521..F-526 (open,
listed at `design/FOLLOWUPS.md:17372-17500`), F-520 (closed; the banner's
id+origins comparison held in every walk here), F-527 (partly closed; I-1 and
M-1 are new sites of its class). `grep` of FOLLOWUPS.md for key-order / Keep
my order, passphrase Back, Watch-only/`reg.count`, `2009-01-0`, tpub, "cut
once": none of I-1, I-2, M-1..M-6 is filed.

## What I could not verify

- No hardware. List selection was driven with synthetic Up/Down button
  events, which the SH2 lacks; the row-tap primitive is covered by the
  existing W-2 touch test, not by my walks.
- The `hash:` -> `phrase:` source switch and the KDF-backed phrase route were
  not walked (lens 3's instrument). By reading: every `composerHashEdit` arm
  writes `p.Hash` directly (`gui/composer_hash.go:662,752,756`), provenance is
  per digest, and orphaned held material is listed at Done and never cut
  (`composer_preimage_plate.go:76-125`).
- The `now:`-bounded date pad was not re-walked; its below-bound modal ejects
  the operator (F-525, open).
- The 32-slot / 8-path / 9-per-path ceilings were read
  (`composer_shape.go:252-268, 328-333, 496-503`), not walked; §8m line 5 is
  covered by `TestComposerSection8mRefusalsAllDrawThroughTheRealPath`.
- Full mode with a seed-derived slot was not driven to the engraver (the
  planner walk took Watch-only); `composerSecretCards` was unit-tested.
- Power loss mid-cut: no instrument; conclusion is by reading only.
- The whole gui shard set was not re-run: no shared harness helper was
  touched (my file adds only `fable*` helpers).

## Out of lens

- Multisig Build's `buildSeedForSlot` (`gui/multisig_build.go:775-787`) has
  I-2's exact shape; a fix should land on both or the two programs will
  disagree about what Back means at the passphrase.
- Host `me sysw pack` admits a `tpub` in a `key:` record (M-6's other half;
  lens 1/2).
- `bundleMs1ReminderText` ("Also hand-engrave your ms1 share(s)",
  `gui/bundle_flow.go:796-798`) is shown after every composer set with no
  cardMS1 — including a set whose keys all came from `key:` records, where no
  ms1 share exists. Not measured on screen in my walks.
- During this run the mnemonic-engrave working tree showed `demo/sh2/
  CLI_SPINE.md` and `demo/sh2/src/index.html` modified; not by me (I wrote
  only this report there).

## Appendix A — `gui/fable_flow_walk_test.go` (verbatim, as run)

```go
package gui

// Lens-4 (device flow, adversarial) walk tests written for the composer fable
// review r0. They script the SH2's real buttons through the Go harness and
// assert on what the plate planner was handed, not on screen text alone.
//
// A test whose name says "Spec" asserts the SPEC's promise; where it is RED at
// the tip it IS the counterexample. Tests that document a refusal or a
// correct path are mutation-checked in the report.

import (
	"encoding/binary"
	"encoding/hex"
	"strings"
	"testing"
	"testing/synctest"
	"time"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
	"seedhammer.com/md"
	"seedhammer.com/mk"
	"seedhammer.com/sysw"
)

// fableHold holds Button3 past confirmDelay: the only route through a
// ConfirmWarningScreen.
func fableHold(ctx *Context, frame func() (string, bool)) {
	press(&ctx.Router, Button3)
	frame()
	time.Sleep(confirmDelay)
	frame()
}

// fableCaptureConsent installs composerSelfCheckFaultHook as a READ seam and
// returns the chunks the consent was rendered from -- the set that reaches
// composerEngraveStep as `consent` (keyed when every slot is seated, the
// template otherwise). The hook returns its input unchanged.
func fableCaptureConsent(t *testing.T) *[]string {
	t.Helper()
	captured := new([]string)
	prev := composerSelfCheckFaultHook
	composerSelfCheckFaultHook = func(c []string) []string {
		*captured = append([]string(nil), c...)
		return c
	}
	t.Cleanup(func() { composerSelfCheckFaultHook = prev })
	return captured
}

// fableStartWsh walks composerFlow's opening pair: Segwit (wsh), then the
// blank row of "Start from?".
func fableStartWsh(t *testing.T, ctx *Context, frame func() (string, bool)) {
	t.Helper()
	if got, ok := pumpUntil(frame, "Which script?", 24); !ok {
		t.Fatalf("no wrapper picker.\nLast frame: %q", got)
	}
	click(&ctx.Router, Down) // Taproot -> Segwit (wsh)
	click(&ctx.Router, Button3)
	if got, ok := pumpUntil(frame, "Start from?", 24); !ok {
		t.Fatalf("no preset screen.\nLast frame: %q", got)
	}
	click(&ctx.Router, Button3) // row 0 = Build my own paths
	if got, ok := pumpUntil(frame, "Add a spend path", 24); !ok {
		t.Fatalf("no path list.\nLast frame: %q", got)
	}
}

// fableAddKeyedPath takes "Add a spend path" (rowsDown rows below the top of
// the list) and answers Keys, n, k.
func fableAddKeyedPath(t *testing.T, ctx *Context, frame func() (string, bool), rowsDown, n, k int) {
	t.Helper()
	if got, ok := pumpUntil(frame, "Add a spend path", 24); !ok {
		t.Fatalf("no path list.\nLast frame: %q", got)
	}
	for range rowsDown {
		click(&ctx.Router, Down)
	}
	click(&ctx.Router, Button3)
	pumpUntil(frame, "What can spend on this path?", 24)
	click(&ctx.Router, Button3) // Keys
	pumpUntil(frame, "how many keys?", 24)
	for range n - 1 {
		click(&ctx.Router, Down)
	}
	click(&ctx.Router, Button3)
	pumpUntil(frame, "how many must sign?", 24)
	for range k - 1 {
		click(&ctx.Router, Down)
	}
	click(&ctx.Router, Button3)
}

// fableDone takes the Done row of a list holding `paths` paths: the rows are
// the paths, "Add a spend path", "Change the script", "Done".
func fableDone(t *testing.T, ctx *Context, frame func() (string, bool), paths int) {
	t.Helper()
	if got, ok := pumpUntil(frame, "Add a spend path", 24); !ok {
		t.Fatalf("no path list.\nLast frame: %q", got)
	}
	for range paths + 2 {
		click(&ctx.Router, Down)
	}
	click(&ctx.Router, Button3)
}

// fableTypeSeed answers the word-count picker with 12 and types the abandon
// vector.
func fableTypeSeed(t *testing.T, ctx *Context, frame func() (string, bool)) {
	t.Helper()
	if got, ok := pumpUntil(frame, "Choose number of words", 48); !ok {
		t.Fatalf("no word-count picker.\nLast frame: %q", got)
	}
	click(&ctx.Router, Button3) // 12 words
	frame()
	typeWords(&ctx.Router, frame, fixtureMasterA)
}

// ─── 1. Key order: "Keep my order" is reversed by Back + Done + forward ──────

// TestFableSpecKeyOrderSurvivesBackFromTheStubScreen: §7b "Back preserves
// everything" and §8b "fires once per key set where sorted was legal and
// declined". The operator declines sorted with a hold-to-confirm, reaches the
// stub screen, steps Back to re-read the list, takes Done again and leaves the
// re-asked "Key order" screen by the forward button -- the control that has
// advanced every other screen. The policy the consent renders must still be
// the unsorted one they confirmed.
func TestFableSpecKeyOrderSurvivesBackFromTheStubScreen(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		captured := fableCaptureConsent(t)
		frame, quit := runUI(ctx, func() { composerFlow(ctx, &descriptorTheme) })
		defer quit()

		fableStartWsh(t, ctx, frame)
		fableAddKeyedPath(t, ctx, frame, 0, 3, 2)
		fableDone(t, ctx, frame, 1)
		if got, ok := pumpUntil(frame, "Sorted keys, or your order?", 24); !ok {
			t.Fatalf("no key-order question.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down) // Sorted (usual) -> Keep my order
		click(&ctx.Router, Button3)
		if got, ok := pumpUntil(frame, "UNSORTED KEYS (EXPERIMENTAL)", 24); !ok {
			t.Fatalf("§8b never fired on the decline.\nLast frame: %q", got)
		}
		fableHold(ctx, frame)
		if got, ok := pumpUntil(frame, "mk1 stub (template)", 32); !ok {
			t.Fatalf("no stub screen.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button1) // Back, to the path list
		fableDone(t, ctx, frame, 1)
		if got, ok := pumpUntil(frame, "Sorted keys, or your order?", 24); !ok {
			t.Fatalf("Done did not re-ask key order.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3) // forward, on whatever row the screen opened on

		_, experimentalAgain := pumpUntil(frame, "UNSORTED KEYS (EXPERIMENTAL)", 6)
		if experimentalAgain {
			fableHold(ctx, frame)
		}
		got, ok := pumpUntil(frame, "mk1 stub (template)", 32)
		if !ok {
			t.Fatalf("no second stub screen.\nLast frame: %q", got)
		}
		bannerFired := uiContains(got, "The shape changed")
		composerPageToEnd(t, ctx, frame)
		if _, ok := pumpUntil(frame, "Seat keys into this template?", 12); ok {
			click(&ctx.Router, Button3) // Engrave a key-less template
		}
		if got, ok = pumpUntil(frame, "Script:", 32); !ok {
			t.Fatalf("no consent.\nLast frame: %q", got)
		}
		if len(*captured) == 0 {
			t.Fatal("the consent hook saw no chunks")
		}
		shape, err := md.PolicyShapeChunks(*captured)
		if err != nil || len(shape.Branches) != 1 {
			t.Fatalf("consent chunks do not describe one branch: %v", err)
		}
		t.Logf("second pass: §8b re-fired=%v, stub banner=%v, consent has UNSORTED mark=%v, decoded Sorted=%v",
			experimentalAgain, bannerFired, uiContains(got, "UNSORTED"), shape.Branches[0].Sorted)
		if shape.Branches[0].Sorted {
			t.Errorf("the operator hold-confirmed 'Keep my order'; after Back from the stub "+
				"screen and Done again, the forward button on 'Key order' silently made the "+
				"policy sortedmulti (§8b re-fired=%v, only signal: stub banner %q=%v). "+
				"Consent chunks: %q", experimentalAgain, "The shape changed", bannerFired, *captured)
		}
	})
}

// ─── 2. Back inside the passphrase entry seats the seed WITHOUT it ───────────

// TestFableSpecBackOnThePassphraseKeyboardIsADecline: the digit pad's own rule
// ("Back is a decline everywhere on this device") and §7b's "going back should
// lose nothing". The operator chose "Add passphrase", started typing, and
// pressed Back to correct something. Expected: one screen back, the
// passphrase question. Measured: what the flow does instead, and the
// fingerprint that then reaches the mapping review.
func TestFableSpecBackOnThePassphraseKeyboardIsADecline(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		ctx.sysw = composerSessionWith([]string{composerTestKeyRecord, composerTestKeyRecord2}, nil)
		frame, quit := runUI(ctx, func() { composerFlow(ctx, &descriptorTheme) })
		defer quit()

		fableStartWsh(t, ctx, frame)
		fableAddKeyedPath(t, ctx, frame, 0, 2, 1)
		fableDone(t, ctx, frame, 1)
		pumpUntil(frame, "Sorted keys, or your order?", 24)
		click(&ctx.Router, Button3) // Sorted
		if got, ok := pumpUntil(frame, "mk1 stub (template)", 32); !ok {
			t.Fatalf("no stub screen.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)
		if got, ok := pumpUntil(frame, "Slot @0", 24); !ok {
			t.Fatalf("no seat prompt.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down, Down) // K1, K2 -> Type a seed
		click(&ctx.Router, Button3)
		fableTypeSeed(t, ctx, frame)
		if got, ok := pumpUntil(frame, "Add a BIP-39 passphrase?", 48); !ok {
			t.Fatalf("no passphrase question.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down) // Skip -> Add passphrase
		click(&ctx.Router, Button3)
		// The keyboard: same title as the question, no "Add a BIP-39" lead.
		var got string
		onKeyboard := false
		for i := 0; i < 8; i++ {
			c, ok := frame()
			if !ok {
				break
			}
			got = c
			if uiContains(c, "Passphrase seed 1") && !uiContains(c, "Add a BIP-39 passphrase?") {
				onKeyboard = true
				break
			}
		}
		if !onKeyboard {
			t.Fatalf("the passphrase keyboard never drew.\nLast frame: %q", got)
		}
		runes(&ctx.Router, "x")
		frame()
		click(&ctx.Router, Button1) // Back, mid-passphrase

		got, ok := pumpUntil(frame, "Add a BIP-39 passphrase?", 6)
		if ok {
			return // spec behaviour: one screen back
		}
		t.Errorf("Back on the passphrase keyboard did not return to the passphrase question; "+
			"the device drew: %q", got)

		// Measure what it did instead.
		got, ok = pumpUntil(frame, "choose a key", 8)
		if !ok {
			t.Fatalf("after Back, neither the passphrase question nor the seat prompt drew.\nLast frame: %q", got)
		}
		if !uiContains(got, "seed 1") {
			t.Fatalf("the seat prompt after Back offers no seed row.\nFrame: %q", got)
		}
		click(&ctx.Router, Down, Down) // K1, K2 -> seed 1 (any slots)
		click(&ctx.Router, Button3)
		if got, ok = pumpUntil(frame, "Slot @1", 24); !ok {
			t.Fatalf("slot @1 never asked.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down) // K1 -> K2 (K1 would be the same key as seed@0)
		click(&ctx.Router, Button3)
		if got, ok = pumpUntil(frame, "Key mapping", 24); !ok {
			t.Fatalf("no mapping review.\nLast frame: %q", got)
		}
		m, err := bip39.ParseMnemonic(fixtureMasterA)
		if err != nil {
			t.Fatal(err)
		}
		_, bareFP, _ := deriveAccountXpub(m, "", &chaincfg.MainNetParams, nil)
		_, ppFP, _ := deriveAccountXpub(m, "x", &chaincfg.MainNetParams, nil)
		var bare, pp [4]byte
		binary.BigEndian.PutUint32(bare[:], bareFP)
		binary.BigEndian.PutUint32(pp[:], ppFP)
		t.Logf("mapping review after Back mid-passphrase: %q", got)
		t.Logf("bare-seed fingerprint %x (what the review shows); with passphrase 'x' it would be %x", bare, pp)
		if !uiContains(got, "@0: "+hex.EncodeToString(bare[:])) {
			t.Errorf("expected the mapping review to seat the BARE seed at @0 (%x); frame %q", bare, got)
		}
	})
}

// ─── 3. The date pad: 2009-01-01 and 2009-01-02 get the CEILING body ─────────

// TestFableSpecDateBelowTheFloorInsideTwoThousandNineNamesTheFloor: §6b "the
// entry refuses every date before 2009-01-03 with §8t". The refusal body for a
// date below the floor must be §8t's, not the 2038 ceiling's.
func TestFableSpecDateBelowTheFloorInsideTwoThousandNineNamesTheFloor(t *testing.T) {
	for _, tc := range []struct {
		digits string
		want   string
	}{
		{"20081231", composerCopyDateFloor()},
		{"20090101", composerCopyDateFloor()},
		{"20090102", composerCopyDateFloor()},
		{"20380120", composerCopyDateCeiling()},
	} {
		line, valid := composerDateBandEcho(tc.digits)
		if valid {
			t.Errorf("%s was accepted", tc.digits)
			continue
		}
		if line != tc.want {
			t.Errorf("%s refused with %q, want %q", tc.digits, line, tc.want)
		}
	}
	if line, valid := composerDateBandEcho("20090103"); !valid {
		t.Errorf("20090103 (the floor itself) refused with %q", line)
	}
	if line, valid := composerDateBandEcho("20380119"); !valid {
		t.Errorf("20380119 (the ceiling itself) refused with %q", line)
	}
}

// ─── 4. A key-less path cleared to empty is refused as a lock-only path ──────

// TestFableEmptiedKeylessPathIsRefusedWithTheLockOnlyBody documents the body
// an operator meets after "Hash lock -> No hash lock" on a key-less path: the
// row says "empty", the refusal talks about "only a time lock".
func TestFableEmptiedKeylessPathIsRefusedWithTheLockOnlyBody(t *testing.T) {
	list := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 1, Sorted: true}},
		{}, // key-less path whose hash was cleared
	}}
	if got := composerPathLine(list.Paths[1], 1); got != "Path 2: empty" {
		t.Fatalf("row reads %q", got)
	}
	_, err := md.ValidatePathList(list)
	if err == nil {
		t.Fatal("an empty path validated")
	}
	body, ok := composerRefusalBody(err)
	if !ok {
		t.Fatalf("no §8m body for %v", err)
	}
	t.Logf("row %q is refused with %q", composerPathLine(list.Paths[1], 1), body)
	if body != composerCopyRefuseLockOnly() {
		t.Errorf("body changed: %q", body)
	}
}

// ─── 5. The Keys editor opens on n = 1, k = 1 ────────────────────────────────

// TestFableSpecKeysEditorShowsTheKeySetInForce is journey C-1's class on the
// count pickers: opening "Keys" on a 2-of-3 path and leaving by the forward
// button must not rewrite the path.
func TestFableSpecKeysEditorShowsTheKeySetInForce(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, quit := runUI(ctx, func() { composerFlow(ctx, &descriptorTheme) })
		defer quit()

		fableStartWsh(t, ctx, frame)
		fableAddKeyedPath(t, ctx, frame, 0, 3, 2)
		if got, ok := pumpUntil(frame, "Path 1: 2-of-3", 24); !ok {
			t.Fatalf("no path row.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3) // row 0 = Path 1
		if got, ok := pumpUntil(frame, "Remove path", 24); !ok {
			t.Fatalf("no path menu.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3) // Keys
		pumpUntil(frame, "how many keys?", 24)
		click(&ctx.Router, Button3) // forward, on the row the picker opened on
		pumpUntil(frame, "how many must sign?", 24)
		click(&ctx.Router, Button3) // forward again
		got, ok := pumpUntil(frame, "Remove path", 24)
		if !ok {
			t.Fatalf("the path menu never came back.\nLast frame: %q", got)
		}
		if !uiContains(got, "Path 1: 2-of-3") {
			t.Errorf("opening Keys on a 2-of-3 path and leaving by the forward button twice "+
				"rewrote it; the menu now reads: %q", got)
		}
	})
}

// ─── 6. A typed seed that seats nothing still asks Full vs Watch-only ────────

// TestFableSpecEngraveModeIsAskedOnlyForSeedDerivedSlots: §7f "For
// seed-derived slots: Full (seed + keys) or Watch-only (keys)". A seed typed
// and then not used for any slot must not raise the question; if it does,
// "Full" cuts no seed plate.
func TestFableSpecEngraveModeIsAskedOnlyForSeedDerivedSlots(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		ctx.sysw = composerSessionWith([]string{composerTestKeyRecord, composerTestKeyRecord2}, nil)
		frame, quit := runUI(ctx, func() { composerFlow(ctx, &descriptorTheme) })
		defer quit()

		fableStartWsh(t, ctx, frame)
		fableAddKeyedPath(t, ctx, frame, 0, 2, 1)
		fableDone(t, ctx, frame, 1)
		pumpUntil(frame, "Sorted keys, or your order?", 24)
		click(&ctx.Router, Button3)
		pumpUntil(frame, "mk1 stub (template)", 32)
		composerPageToEnd(t, ctx, frame)
		pumpUntil(frame, "Slot @0", 24)
		click(&ctx.Router, Down, Down) // Type a seed
		click(&ctx.Router, Button3)
		fableTypeSeed(t, ctx, frame)
		pumpUntil(frame, "Add a BIP-39 passphrase?", 48)
		click(&ctx.Router, Button3) // Skip
		// Back at @0 with the seed on offer; seat the two key records instead.
		if got, ok := pumpUntil(frame, "Slot @0", 24); !ok || !uiContains(got, "seed 1") {
			t.Fatalf("no seat prompt with the seed row.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3) // K1
		pumpUntil(frame, "Slot @1", 24)
		click(&ctx.Router, Button3) // K2 (K1 is used)
		if got, ok := pumpUntil(frame, "Key mapping", 24); !ok {
			t.Fatalf("no mapping review.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)
		if got, ok := pumpUntil(frame, "mk1 stub (policy)", 32); !ok {
			t.Fatalf("no keyed stub screen.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)
		if got, ok := pumpUntil(frame, "Script:", 32); !ok {
			t.Fatalf("no consent.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)
		if got, ok := pumpUntil(frame, "Nothing outside this device", 32); !ok {
			t.Fatalf("no §8l.\nLast frame: %q", got)
		}
		fableHold(ctx, frame)
		if got, ok := pumpUntil(frame, "Which form?", 24); !ok {
			t.Fatalf("no form pick.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3) // The policy itself
		got, asked := pumpUntil(frame, "What to engrave?", 6)
		if asked {
			t.Errorf("Full vs Watch-only was asked though no slot is seed-derived: %q", got)
			click(&ctx.Router, Button3) // Full (seed + keys)
		}
		if got, ok := pumpUntil(frame, "Plates To Cut", 32); !ok {
			t.Fatalf("no census.\nLast frame: %q", got)
		} else {
			t.Logf("census after choosing Full with no seed-derived slot: %q", got)
			if uiContains(got, "ms1") {
				t.Errorf("a seed plate was planned for a seed that seats nothing: %q", got)
			}
		}
	})
}

// ─── 7. One seed registered twice is cut twice ───────────────────────────────

// TestFableSpecOneSeedTypedTwiceIsCutOnce: §7f "A seed that filled several
// slots is cut ONCE". Typing the same words at two "Type a seed" prompts
// registers two ids for one secret.
func TestFableSpecOneSeedTypedTwiceIsCutOnce(t *testing.T) {
	st := &composerState{reg: &seedRegistry{}, list: md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 2, Sorted: true}},
	}}}
	for i := 0; i < 2; i++ {
		id, err := st.reg.add("seed", composerTestMnemonic(t), "", &chaincfg.MainNetParams)
		if err != nil {
			t.Fatal(err)
		}
		seed, _ := st.reg.at(id)
		var fp [4]byte
		binary.BigEndian.PutUint32(fp[:], seed.MasterFP)
		st.sources = append(st.sources, composerSource{
			kind: composerSourceSeed, label: "seed", fingerprint: fp, fpPresent: true, seedID: id,
		})
	}
	st.assigned = make([]composerAssignment, 2)
	for i := range st.assigned {
		a, err := composerSeedDerive(st, uint8(i), i)
		if err != nil {
			t.Fatal(err)
		}
		st.assigned[i] = a
	}
	if st.assigned[0].account != 0 || st.assigned[1].account != 1 {
		t.Fatalf("accounts %d/%d: the two registrations were not seen as one master",
			st.assigned[0].account, st.assigned[1].account)
	}
	if st.assigned[0].xpub == st.assigned[1].xpub {
		t.Fatal("the two slots hold one key; the mapping review would refuse this")
	}
	cards, err := composerSecretCards(st)
	if err != nil {
		t.Fatal(err)
	}
	if len(cards) != 1 {
		same := len(cards) == 2 && cards[0].strings[0] == cards[1].strings[0]
		t.Errorf("one seed at two slots planned %d ms1 plates (byte-identical: %v); §7f says a seed is cut ONCE",
			len(cards), same)
	}
	// Positive control: ONE registration at two slots is cut once.
	st.assigned[1].src = 0
	st.sources = st.sources[:1]
	cards, err = composerSecretCards(st)
	if err != nil {
		t.Fatal(err)
	}
	if len(cards) != 1 {
		t.Errorf("control: one registration at two slots planned %d ms1 plates", len(cards))
	}
}

// ─── 10. The key-less EXPERIMENTAL door: a tap does not pass it, Back removes the path ───

// TestFableKeylessDoorIsHoldOnlyAndBackReturnsToTheShape: §8 header, "every
// confirm-to-proceed screen is dismissed only by a tap on CONTINUE, and Back
// returns to the shape". A plain click on Button3 must leave the door up; Back
// must return to the path list with the half-made path gone.
func TestFableKeylessDoorIsHoldOnlyAndBackReturnsToTheShape(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, quit := runUI(ctx, func() { composerFlow(ctx, &descriptorTheme) })
		defer quit()

		fableStartWsh(t, ctx, frame)
		click(&ctx.Router, Button3) // Add a spend path
		pumpUntil(frame, "What can spend on this path?", 24)
		click(&ctx.Router, Down) // Keys -> A hash, no keys
		click(&ctx.Router, Button3)
		got, ok := pumpUntil(frame, "KEY-LESS PATH (EXPERIMENTAL)", 24)
		if !ok {
			t.Fatalf("§8a never drew.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3) // a TAP, not a hold
		for i := 0; i < 4; i++ {
			got, _ = frame()
		}
		if !uiContains(got, "KEY-LESS PATH (EXPERIMENTAL)") {
			t.Errorf("a plain tap dismissed the EXPERIMENTAL door: %q", got)
		}
		click(&ctx.Router, Button1) // Back
		if got, ok = pumpUntil(frame, "Add a spend path", 24); !ok {
			t.Fatalf("Back did not return to the shape.\nLast frame: %q", got)
		}
		if uiContains(got, "Path 1:") {
			t.Errorf("Back at the EXPERIMENTAL door left a path behind: %q", got)
		}
		if !uiContains(got, "slots: 0") {
			t.Errorf("the list after Back does not read slots: 0: %q", got)
		}
	})
}

// ─── 11. Digit-pad bands: what each edge entry echoes ────────────────────────

// TestFableDigitPadBands records what the four pads make of their edges, and
// pins the two relative ceilings and the height ceiling.
func TestFableDigitPadBands(t *testing.T) {
	type row struct {
		pad, digits string
		echo        func(string) (string, bool)
	}
	rows := []row{
		{"blocks", "0", composerBlocksBandEcho},
		{"blocks", "00012", composerBlocksBandEcho},
		{"blocks", "65535", composerBlocksBandEcho},
		{"blocks", "65536", composerBlocksBandEcho},
		{"days", "0", composerDaysBandEcho},
		{"days", "036", composerDaysBandEcho},
		{"days", "388", composerDaysBandEcho},
		{"days", "389", composerDaysBandEcho},
		{"height", "0", composerHeightBandEcho},
		{"height", "499999999", composerHeightBandEcho},
		{"height", "500000000", composerHeightBandEcho},
		{"date", "20270230", composerDateBandEcho},
		{"date", "19851105", composerDateBandEcho},
		{"date", "20090103", composerDateBandEcho},
		{"date", "20380119", composerDateBandEcho},
	}
	for _, r := range rows {
		line, valid := r.echo(r.digits)
		t.Logf("%-6s %-9s valid=%-5v %q", r.pad, r.digits, valid, line)
	}
	if _, valid := composerBlocksBandEcho("65535"); !valid {
		t.Error("65535 blocks refused")
	}
	if _, valid := composerBlocksBandEcho("65536"); valid {
		t.Error("65536 blocks accepted")
	}
	if _, valid := composerDaysBandEcho("389"); valid {
		t.Error("389 days accepted")
	}
	if _, valid := composerHeightBandEcho("500000000"); valid {
		t.Error("height 500000000 accepted")
	}
	// The encoded operands at the edges pass md.Lock.Check.
	for _, l := range []md.Lock{
		{Kind: md.LockOlderBlocks, Value: 65535},
		{Kind: md.LockOlderUnits, Value: composerDaysToUnits(388)},
		{Kind: md.LockAfterHeight, Value: 499_999_999},
		{Kind: md.LockAfterTime, Value: composerDateFloorUnix},
		{Kind: md.LockAfterTime, Value: composerDateCeilingUnix},
	} {
		if err := l.Check(); err != nil {
			t.Errorf("%v/%d: %v", l.Kind, l.Value, err)
		}
	}
	if u := composerDaysToUnits(388); u > 65535 {
		t.Errorf("388 days = %d units, over the wire's 65535", u)
	}
}

// ─── 12. A testnet xpub in a key: record ─────────────────────────────────────

// TestFableTestnetXpubKeyRecord measures what the device does with a key:
// record whose xpub is a tpub: is it classified as a key, and what does the
// card minted from it say.
func TestFableTestnetXpubKeyRecord(t *testing.T) {
	m, err := bip39.ParseMnemonic(fixtureMasterA)
	if err != nil {
		t.Fatal(err)
	}
	const h = 0x80000000
	tpub, fp, err := deriveAccountXpub(m, "", &chaincfg.TestNet3Params, []uint32{48 | h, 1 | h, 0 | h, 2 | h})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.HasPrefix(tpub, "tpub") {
		t.Fatalf("derived %s, not a tpub", tpub[:8])
	}
	var fpb [4]byte
	binary.BigEndian.PutUint32(fpb[:], fp)
	rec := composerRecord("key:", "[73c5da0a/48'/1'/0'/2']"+tpub)
	kr, err := sysw.ParseKeyRecord(rec)
	t.Logf("ParseKeyRecord(tpub record): err=%v; Classify=%v", err, sysw.Classify(rec))
	if err != nil {
		return
	}
	card, err := mk.Encode(mk.Card{Network: "mainnet", Path: "m/48'/1'/0'/2'",
		Fingerprint: "73c5da0a", Xpub: kr.Xpub, Stubs: [][4]byte{{1, 2, 3, 4}}})
	if err != nil {
		t.Logf("mk.Encode(card with tpub, Network mainnet): %v", err)
		return
	}
	back, err := mk.Decode(card)
	t.Logf("re-decoded card: Network=%q Xpub prefix=%s err=%v", back.Network, back.Xpub[:4], err)
}

// ─── 8. Malformed record spellings are inert ─────────────────────────────────

// TestFableRecordSpellingsAreInert: a key: record with CRLF, a leading or
// trailing space, upper-case hex, or an upper-case prefix must not classify
// as a key (§6a: malformed records are inert on the device).
func TestFableRecordSpellingsAreInert(t *testing.T) {
	if got := sysw.Classify(composerTestKeyRecord); got != sysw.ClassKey {
		t.Fatalf("the well-formed fixture classifies as %v", got)
	}
	body := strings.TrimPrefix(composerTestKeyRecord, "key:")
	for _, tc := range []struct{ name, rec string }{
		{"CRLF", composerTestKeyRecord + "\r\n"},
		{"LF", composerTestKeyRecord + "\n"},
		{"leading space", " " + composerTestKeyRecord},
		{"trailing space", composerTestKeyRecord + " "},
		{"upper-case hex", "key:" + strings.ToUpper(body)},
		{"upper-case prefix", "KEY:" + body},
	} {
		got := sysw.Classify(tc.rec)
		t.Logf("%-17s -> %v", tc.name, got)
		if got == sysw.ClassKey {
			t.Errorf("%s: classified as a key", tc.name)
		}
	}
}

// ─── 9. Key -> seed -> Back: what reaches the planner ────────────────────────

// fableDecodeCards groups a run of mk1 chunk lines into cards the way
// composerCardSources does: a growing window that decodes is one card.
func fableDecodeCards(t *testing.T, lines []string) []mk.Card {
	t.Helper()
	var out []mk.Card
	for start := 0; start < len(lines); {
		decoded := false
		for end := start + 1; end <= len(lines); end++ {
			c, err := mk.Decode(lines[start:end])
			if err == nil {
				out = append(out, c)
				start = end
				decoded = true
				break
			}
		}
		if !decoded {
			t.Fatalf("mk1 lines from %d never decode: %q", start, lines[start:])
		}
	}
	return out
}

// TestFableKeyThenSeedThenBackReachesThePlannerAsShown seats @0 from a key:
// record, steps Back from @1 (releasing @0), re-seats @0 from a typed seed,
// seats @1 from the other record, takes form B watch-only, and cuts every
// plate. It asserts on the strings the engraver was handed: the md1 template
// and the two mk1 cards, decoded, must carry exactly what the mapping review
// showed.
func TestFableKeyThenSeedThenBackReachesThePlannerAsShown(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		e := newEngraver()
		p := newEngravedAwarePlatform()
		p.display = sh2DisplaySize
		p.engraver = e
		ctx := NewContext(p)
		ctx.sysw = composerSessionWith([]string{composerTestKeyRecord, composerTestKeyRecord2}, nil)
		captured := fableCaptureConsent(t)
		frame, _, _, quit := runUITouchRaster(ctx, func() { composerFlow(ctx, &descriptorTheme) })
		defer quit()

		fableStartWsh(t, ctx, frame)
		fableAddKeyedPath(t, ctx, frame, 0, 2, 1)
		fableDone(t, ctx, frame, 1)
		pumpUntil(frame, "Sorted keys, or your order?", 24)
		click(&ctx.Router, Button3)
		if got, ok := pumpUntil(frame, "mk1 stub (template)", 32); !ok {
			t.Fatalf("no stub screen.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)

		// @0 = K1.
		if got, ok := pumpUntil(frame, "Slot @0", 24); !ok {
			t.Fatalf("no seat prompt.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3)
		// At @1, Back: releases @0.
		if got, ok := pumpUntil(frame, "Slot @1", 24); !ok {
			t.Fatalf("no @1 prompt.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button1)
		got, ok := pumpUntil(frame, "Slot @0", 24)
		if !ok {
			t.Fatalf("Back at @1 did not land on @0.\nLast frame: %q", got)
		}
		if !uiContains(got, "48h/0h/0h/2h") {
			t.Fatalf("K1 was not released back onto the list.\nFrame: %q", got)
		}
		// @0 = a typed seed.
		click(&ctx.Router, Down, Down) // K1, K2 -> Type a seed
		click(&ctx.Router, Button3)
		fableTypeSeed(t, ctx, frame)
		pumpUntil(frame, "Add a BIP-39 passphrase?", 48)
		click(&ctx.Router, Button3) // Skip
		if got, ok = pumpUntil(frame, "Slot @0", 24); !ok || !uiContains(got, "seed 1") {
			t.Fatalf("no seat prompt with the seed row.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down, Down) // K1, K2 -> seed 1 (any slots)
		click(&ctx.Router, Button3)
		// @1 = K2.
		if got, ok = pumpUntil(frame, "Slot @1", 24); !ok {
			t.Fatalf("no @1 prompt.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down) // K1 -> K2
		click(&ctx.Router, Button3)
		if got, ok = pumpUntil(frame, "Key mapping", 24); !ok {
			t.Fatalf("no mapping review.\nLast frame: %q", got)
		}
		for _, want := range []string{"@0: 73c5da0a m/48'/0'/0'/2'", "@1: 73c5da0a m/48'/0'/1'/2'"} {
			if !uiContains(got, want) {
				t.Errorf("mapping review lacks %q: %q", want, got)
			}
		}
		t.Logf("mapping review page 1: %q", got)
		// §8g: the seed at @0 and the record at @1 share one master inside one
		// 1-of-2 path -- one person can satisfy it. The body is on a later page.
		if got, ok = composerPageUntil(t, ctx, frame, "SAME SEED, SAME PATH", 6); !ok {
			t.Errorf("§8g never drew on the mapping review for two slots of one master in one path.\nLast page: %q", got)
		} else {
			t.Logf("§8g page: %q", got)
		}
		composerPageToEnd(t, ctx, frame)
		if got, ok = pumpUntil(frame, "mk1 stub (policy)", 32); !ok {
			t.Fatalf("no keyed stub screen.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)
		if got, ok = pumpUntil(frame, "Script:", 32); !ok {
			t.Fatalf("no consent.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)
		if got, ok = pumpUntil(frame, "Nothing outside this device", 32); !ok {
			t.Fatalf("no §8l.\nLast frame: %q", got)
		}
		fableHold(ctx, frame)
		if got, ok = pumpUntil(frame, "Which form?", 24); !ok {
			t.Fatalf("no form pick.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down) // The policy itself -> Template plus key cards
		click(&ctx.Router, Button3)
		if got, ok = pumpUntil(frame, "What to engrave?", 24); !ok {
			t.Fatalf("no engrave-mode pick.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down) // Full -> Watch-only
		click(&ctx.Router, Button3)
		if got, ok = pumpUntil(frame, "Plates To Cut", 32); !ok {
			t.Fatalf("no census.\nLast frame: %q", got)
		}
		for _, want := range []string{"md1 template", "mk1 key @0", "mk1 key @1"} {
			if !uiContains(got, want) {
				t.Errorf("census lacks %q: %q", want, got)
			}
		}
		if uiContains(got, "ms1") {
			t.Errorf("watch-only census plans a seed plate: %q", got)
		}
		composerPageToEnd(t, ctx, frame)

		plates := 0
		for {
			if _, ok := pumpUntil(frame, "Choose engraving", 96); !ok {
				break
			}
			click(&ctx.Router, Button3)
			frame()
			engraveOnePlate(t, ctx, frame, e)
			plates++
			if plates > 12 {
				t.Fatal("the engrave loop did not terminate")
			}
		}
		if plates == 0 {
			t.Fatal("no plate was cut")
		}
		t.Logf("%d plates cut, %d engraved texts recorded", plates, len(p.engraved))

		var md1Lines, mk1Lines []string
		for _, text := range p.engraved {
			for _, l := range strings.Split(text, "\n") {
				switch {
				case strings.HasPrefix(l, "md1"):
					md1Lines = append(md1Lines, l)
				case strings.HasPrefix(l, "mk1"):
					mk1Lines = append(mk1Lines, l)
				default:
					t.Errorf("an engraved line is neither md1 nor mk1: %q", l)
				}
			}
		}
		// The template on steel.
		_, keys, err := md.ExpandWalletPolicyChunks(md1Lines)
		if err != nil {
			t.Fatalf("the engraved md1 does not expand: %v (%q)", err, md1Lines)
		}
		if len(keys) != 2 {
			t.Fatalf("engraved template has %d slots", len(keys))
		}
		for i, wantPath := range []string{"m/48h/0h/0h/2h", "m/48h/0h/1h/2h"} {
			k := keys[i]
			if k.XpubPresent {
				t.Errorf("template slot @%d carries an xpub", i)
			}
			if !k.FingerprintPresent || hex.EncodeToString(k.Fingerprint[:]) != "73c5da0a" {
				t.Errorf("template slot @%d fingerprint %x present=%v", i, k.Fingerprint, k.FingerprintPresent)
			}
			if k.OriginPath.String() != wantPath {
				t.Errorf("template slot @%d origin %s, want %s", i, k.OriginPath, wantPath)
			}
		}
		// The cards on steel.
		cards := fableDecodeCards(t, mk1Lines)
		if len(cards) != 2 {
			t.Fatalf("%d cards engraved, want 2", len(cards))
		}
		tstub, _ := md.FormAwareStubChunks(md1Lines)
		kstub, _ := md.FormAwareStubChunks(*captured)
		// mk.Decode renders hardening as `h`; the wire carries no notation.
		for i, want := range []struct{ path, xpub string }{
			{"m/48h/0h/0h/2h", composerTestXpubA},
			{"m/48h/0h/1h/2h", composerTestXpubB},
		} {
			c := cards[i]
			if c.Path != want.path || c.Xpub != want.xpub || c.Fingerprint != "73c5da0a" {
				t.Errorf("card %d: path %s fp %s xpub %s; want %s 73c5da0a %s",
					i, c.Path, c.Fingerprint, c.Xpub, want.path, want.xpub)
			}
			hasT, hasK := false, false
			for _, s := range c.Stubs {
				hasT = hasT || s == tstub
				hasK = hasK || s == kstub
			}
			if !hasT || !hasK {
				t.Errorf("card %d stubs %x lack template %x / policy %x", i, c.Stubs, tstub, kstub)
			}
		}
		// The consent chunks (the keyed policy) hold the same two keys.
		_, ckeys, err := md.ExpandWalletPolicyChunks(*captured)
		if err != nil || len(ckeys) != 2 {
			t.Fatalf("consent chunks: %v, %d keys", err, len(ckeys))
		}
		for i, xpub := range []string{composerTestXpubA, composerTestXpubB} {
			cc, pk, _, err := decodeXpubBytes(xpub)
			if err != nil {
				t.Fatal(err)
			}
			var b [65]byte
			copy(b[0:32], cc[:])
			copy(b[32:65], pk[:])
			if !ckeys[i].XpubPresent || ckeys[i].Xpub != b {
				t.Errorf("consent slot @%d does not carry %s", i, xpub[:12])
			}
		}
	})
}
```

## Appendix B — mutation driver `mutate.py` (verbatim, as run)

```python
#!/usr/bin/env python3
"""Apply one mutation at a time to the fable-flow worktree, run one test, restore."""
import os, subprocess, sys, json

WT = "/scratch/code/shibboleth/.tmp/fable-flow"
ENV = dict(os.environ, PATH="/scratch/code/shibboleth/.toolchain/go/bin:" + os.environ["PATH"],
           TMPDIR="/scratch/code/shibboleth/.tmp", CGO_ENABLED="0")
OUT = "/scratch/code/shibboleth/.tmp/fable-flow-out"

M = []  # (name, file, old, new, test, expect)

# ── break-the-code mutations: the test must go RED ──
M.append(("M1 card minted from the NEXT slot's assignment", "gui/composer_cards.go",
"\ta := st.assigned[slot]\n\tif a.src < 0 {",
"\ta := st.assigned[(int(slot)+1)%len(st.assigned)]\n\tif a.src < 0 {",
"TestFableKeyThenSeedThenBackReachesThePlannerAsShown", "FAIL"))
M.append(("M2 Back never releases the source", "gui/composer_seat.go",
"\tif src < len(st.sources) && st.sources[src].kind != composerSourceSeed {\n\t\tst.sources[src].used = false\n\t}",
"\tif src < len(st.sources) && st.sources[src].kind != composerSourceSeed {\n\t\tst.sources[src].used = true\n\t}",
"TestFableKeyThenSeedThenBackReachesThePlannerAsShown", "FAIL"))
M.append(("M3 unhexLower folds upper-case hex", "sysw/composer_records.go",
"\t\tif !(c >= '0' && c <= '9' || c >= 'a' && c <= 'f') {",
"\t\tif !(c >= '0' && c <= '9' || c >= 'a' && c <= 'f' || c >= 'A' && c <= 'F') {",
"TestFableRecordSpellingsAreInert", "FAIL"))
M.append(("M4 empty path row reads hash only", "gui/composer_state.go",
"\t\tbody = \"empty\"",
"\t\tbody = \"hash only\"",
"TestFableEmptiedKeylessPathIsRefusedWithTheLockOnlyBody", "FAIL"))
M.append(("M5 blocks band widened to 65536", "gui/composer_lock.go",
"\tif err != nil || n < 1 || n > 65535 {\n\t\treturn composerCopyRelativeCeiling(), false\n\t}\n\treturn composerCopyLockEchoBlocks(uint32(n)), true",
"\tif err != nil || n < 1 || n > 65536 {\n\t\treturn composerCopyRelativeCeiling(), false\n\t}\n\treturn composerCopyLockEchoBlocks(uint32(n)), true",
"TestFableDigitPadBands", "FAIL"))
M.append(("M6 read screen accepts CONTINUE before the last page", "gui/composer_paged.go",
"\t\tif seenEnd && contBtn.Clicked(ctx) {",
"\t\tif contBtn.Clicked(ctx) {",
"TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage", "FAIL"))
M.append(("M7 Back at the key-less door leaves the path", "gui/composer_shape.go",
"\tif !composerConfirmScreen(ctx, th, \"EXPERIMENTAL\",\n\t\tcomposerConfirmBody(composerCopyKeylessPath())) {\n\t\tst.list.Paths = st.list.Paths[:idx]\n\t\treturn\n\t}",
"\tif !composerConfirmScreen(ctx, th, \"EXPERIMENTAL\",\n\t\tcomposerConfirmBody(composerCopyKeylessPath())) {\n\t\treturn\n\t}",
"TestFableKeylessDoorIsHoldOnlyAndBackReturnsToTheShape", "FAIL"))

# ── fix-direction edits: a RED-at-tip test must go GREEN ──
M.append(("F1 key-order picker opens on the setting in force", "gui/composer_shape.go",
"\tcs := &ChoiceScreen{\n\t\tTitle:   \"Key order\",\n\t\tLead:    \"Sorted keys, or your order?\",\n\t\tChoices: []string{\"Sorted (usual)\", \"Keep my order\"},\n\t}",
"\tinitial := 0\n\tif !st.list.Paths[0].Keys.Sorted {\n\t\tinitial = 1\n\t}\n\tcs := &ChoiceScreen{\n\t\tTitle:   \"Key order\",\n\t\tLead:    \"Sorted keys, or your order?\",\n\t\tChoices: []string{\"Sorted (usual)\", \"Keep my order\"},\n\t\tInitial: initial,\n\t}",
"TestFableSpecKeyOrderSurvivesBackFromTheStubScreen", "PASS"))
M.append(("F2 Back on the passphrase keyboard re-asks the question", "gui/composer_sources.go",
"\tif sel, ok := pp.Choose(ctx, th); ok && sel == 1 {\n\t\tif pass, ok := syswPassphraseFlowTitled(ctx, th, \"Passphrase \"+label); ok {\n\t\t\tif err := st.reg.bindPassphrase(seedID, pass, &chaincfg.MainNetParams); err != nil {\n\t\t\t\tshowError(ctx, th, \"Seed\", \"Couldn't apply that passphrase.\")\n\t\t\t\treturn composerSource{}, false\n\t\t\t}\n\t\t}\n\t}",
"\tfor {\n\t\tsel, ok := pp.Choose(ctx, th)\n\t\tif !ok {\n\t\t\treturn composerSource{}, false\n\t\t}\n\t\tif sel == 0 {\n\t\t\tbreak\n\t\t}\n\t\tpass, ok := syswPassphraseFlowTitled(ctx, th, \"Passphrase \"+label)\n\t\tif !ok {\n\t\t\tcontinue\n\t\t}\n\t\tif err := st.reg.bindPassphrase(seedID, pass, &chaincfg.MainNetParams); err != nil {\n\t\t\tshowError(ctx, th, \"Seed\", \"Couldn't apply that passphrase.\")\n\t\t\treturn composerSource{}, false\n\t\t}\n\t\tbreak\n\t}",
"TestFableSpecBackOnThePassphraseKeyboardIsADecline", "PASS"))
M.append(("F3 date floor compared as a time, not a year", "gui/composer_lock.go",
"\t\tif y < 2009 {\n\t\t\treturn composerCopyDateFloor(), false\n\t\t}",
"\t\tif time.Date(y, time.Month(m), d, 0, 0, 0, 0, time.UTC).Unix() < int64(composerDateFloorUnix) {\n\t\t\treturn composerCopyDateFloor(), false\n\t\t}",
"TestFableSpecDateBelowTheFloorInsideTwoThousandNineNamesTheFloor", "PASS"))
M.append(("F4 count pickers open on the value in force", "gui/composer_shape.go",
[("func composerCountPick(ctx *Context, th *Colors, title, lead string, min, max int) (int, bool) {",
  "func composerCountPick(ctx *Context, th *Colors, title, lead string, min, max, initial int) (int, bool) {"),
 ("\tsel, ok := composerPickScreen(ctx, th, title, lead, rows)\n\tif !ok {\n\t\treturn 0, false\n\t}\n\treturn min + sel, true",
  "\tsel, ok := composerPickScreenFrom(ctx, th, title, lead, rows, initial-min)\n\tif !ok {\n\t\treturn 0, false\n\t}\n\treturn min + sel, true"),
 ("\tn, ok := composerCountPick(ctx, th, \"Keys\", fmt.Sprintf(\"Path %d: how many keys?\", idx+1), min, max)",
  "\tcurN, curK := min, 1\n\tif ks := st.list.Paths[idx].Keys; ks != nil {\n\t\tcurN, curK = int(ks.N), int(ks.K)\n\t}\n\tn, ok := composerCountPick(ctx, th, \"Keys\", fmt.Sprintf(\"Path %d: how many keys?\", idx+1), min, max, curN)"),
 ("\tk, ok := composerCountPick(ctx, th, \"Threshold\", fmt.Sprintf(\"Path %d: how many must sign?\", idx+1), 1, n)",
  "\tk, ok := composerCountPick(ctx, th, \"Threshold\", fmt.Sprintf(\"Path %d: how many must sign?\", idx+1), 1, n, curK)")],
None, "TestFableSpecKeysEditorShowsTheKeySetInForce", "PASS"))
M.append(("F5 engrave mode asked only when a seed-derived slot exists", "gui/composer_flow.go",
"\tif st.reg.count() > 0 {\n\t\tfull, ok := composerEngraveModePick(ctx, th, st)",
"\tseedSeated := false\n\tfor _, a := range st.assigned {\n\t\tif a.src >= 0 && a.src < len(st.sources) && st.sources[a.src].kind == composerSourceSeed {\n\t\t\tseedSeated = true\n\t\t}\n\t}\n\tif seedSeated {\n\t\tfull, ok := composerEngraveModePick(ctx, th, st)",
"TestFableSpecEngraveModeIsAskedOnlyForSeedDerivedSlots", "PASS"))
M.append(("F6 secret plates deduplicated by master, not by registration", "gui/composer_flow.go",
"\tseen := map[int]bool{}\n\tvar out []bundleCard\n\tfor _, a := range st.assigned {\n\t\tif a.src < 0 || a.src >= len(st.sources) {\n\t\t\tcontinue\n\t\t}\n\t\tsrc := st.sources[a.src]\n\t\tif src.kind != composerSourceSeed || seen[src.seedID] {\n\t\t\tcontinue\n\t\t}\n\t\tseen[src.seedID] = true\n\t\tseed, ok := st.reg.at(src.seedID)\n\t\tif !ok {\n\t\t\tcontinue\n\t\t}",
"\tseen := map[uint32]bool{}\n\tvar out []bundleCard\n\tfor _, a := range st.assigned {\n\t\tif a.src < 0 || a.src >= len(st.sources) {\n\t\t\tcontinue\n\t\t}\n\t\tsrc := st.sources[a.src]\n\t\tif src.kind != composerSourceSeed {\n\t\t\tcontinue\n\t\t}\n\t\tseed, ok := st.reg.at(src.seedID)\n\t\tif !ok || seen[seed.MasterFP] {\n\t\t\tcontinue\n\t\t}\n\t\tseen[seed.MasterFP] = true",
"TestFableSpecOneSeedTypedTwiceIsCutOnce", "PASS"))

only = set(sys.argv[1:])
results = []
for entry in M:
    name, path, old, new, test, expect = entry
    tag = name.split()[0]
    if only and tag not in only:
        continue
    full = os.path.join(WT, path)
    src = open(full).read()
    pairs = old if isinstance(old, list) else [(old, new)]
    ok = True
    for o, n in pairs:
        if src.count(o) != 1:
            results.append((name, "MUTATION-NOT-APPLIED (%d matches)" % src.count(o), ""))
            ok = False
            break
        src = src.replace(o, n)
    if not ok:
        continue
    open(full, "w").write(src)
    try:
        log = os.path.join(OUT, "mut-%s.txt" % tag)
        with open(log, "w") as f:
            r = subprocess.run(["go", "test", "./gui/", "-run", "^%s$" % test, "-v", "-count=1"],
                               cwd=WT, env=ENV, stdout=f, stderr=subprocess.STDOUT)
        text = open(log).read()
        ran = ("=== RUN   " + test) in text
        verdict = "PASS" if ("--- PASS: " + test) in text else ("FAIL" if ("--- FAIL: " + test) in text else "BUILD-ERROR/NOT-RUN")
        first = ""
        for line in text.splitlines():
            if "fable_flow_walk_test.go:" in line or "composer_gates_test.go:" in line or line.startswith("#") or ".go:" in line and "error" in line.lower():
                first = line.strip()[:200]
                break
        results.append((name, verdict + (" (ran)" if ran else " (NOT RUN)") + " expect " + expect, first))
    finally:
        subprocess.run(["git", "checkout", "--", path], cwd=WT, check=True)

print(json.dumps(results, indent=1))
clean = subprocess.run(["git", "status", "--porcelain"], cwd=WT, capture_output=True, text=True).stdout
print("git status after restore:\n" + clean)
```
