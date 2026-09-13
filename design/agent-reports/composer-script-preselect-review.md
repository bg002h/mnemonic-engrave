# Independent adversarial execution review — composer script preselect

Artifact: `/scratch/code/shibboleth/seedhammer`, branch `composer-script-preselect`,
tip `6728c22a554caedba850204680c9afe15df152e8`. Whole diff `git diff main..6728c22`
(7 files, +144 / -13).

Question asked: over the whole diff, can a screen or sequence be constructed where
this change makes a picker open on, or return, a row it should not — and does the
new test actually fail on the defect it names?

Method: read-only worktree at `/scratch/code/shibboleth/.tmp/preselect-review`,
Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go`, `TMPDIR=/scratch/code/shibboleth/.tmp`.
Twelve mutations, four instrumented probes, three full-suite runs
(`gui-shard-test.sh ./gui/ 24`), and two scratch journey tests driving the real
flow. Every mutation reverted; the tree was verified pristine at `6728c22` and
green (`1294 == 1294`) as the last act before the worktree was removed.

**Answer to the question: no such sequence exists in the shipped code.** The
preselect returns a row the operator did not choose on no path I could construct,
and the new test fails on the defect it names (MU-3 below, independent of the two
mutations already settled). One Important remains: the property the fix newly
depends on and newly promises the operator — that the highlighted row is *labelled*
with the script in force — is asserted by nothing, and a mutation of it is green
across all 1294 tests.

---

## Findings

### I-1 — the preselect's promise rests on a hand-maintained parallel table that no test binds

`gui/composer_shape.go:128-135` (at the tip):

```go
	choices := []string{"Taproot (tr)", "Segwit (wsh)", "Nested (sh-wsh)", "Legacy (sh)"}
	wrappers := []md.ComposeWrapper{md.ComposeTr, md.ComposeWsh, md.ComposeShWsh, md.ComposeSh}
	initial := 0
	for i, w := range wrappers {
		if w == current {
			initial = i
			break
		}
	}
```

The diff's stated value is in its own doc comment (`gui/composer_shape.go:118-124`
and `gui/gui.go:1922-1924`): *"Preselecting what is in force makes ✓ a no-op and
turns the screen into somewhere the setting can be READ."* The operator reads it by
**label**. `initial` is computed from `wrappers`; what is drawn under the highlight
comes from `choices`. Nothing anywhere asserts `choices[i]` names `wrappers[i]`.

**Counterexample, MU-11 — swap the LABELS of rows 1 and 2, leave `wrappers` alone:**

```
$ perl -0pi -e 's/...Taproot \(tr\)", "Segwit \(wsh\)", "Nested \(sh-wsh\)".../"Taproot (tr)", "Nested (sh-wsh)", "Segwit (wsh)", "Legacy (sh)"/'
$ scripts/gui-shard-test.sh ./gui/ 24
    partition verified exhaustive: 1294 == 1294
RESULT: ok -- all 1294 tests ran across 24 shards
```

Green. A wsh policy now opens the picker with the highlight on a row that reads
**"Nested (sh-wsh)"**, and the whole suite says nothing. That is journey C-1's harm
restated one layer up: the operator opens the screen to read the script in force and
reads the wrong one, with nothing downstream to report it (the path list is
byte-identical under every wrapper and Review never names the script — the diff's own
finding).

**The index↔wrapper half IS covered, which is what makes the gap precise.**
MU-12 — swap `wrappers[1]`/`wrappers[2]` and leave the labels alone:

```
$ scripts/gui-shard-test.sh ./gui/ 24
RESULT: FAIL
--- FAIL: TestComposerBackAtTheMappingReviewKeepsTheSeatedKeys (0.01s)
--- FAIL: TestComposerBackAtThePathListKeepsTheComposition (0.01s)
--- FAIL: TestComposerBackLegWrapperChangeAsksBeforeDiscardingSeats (0.03s)
--- FAIL: TestComposerChangeTheScriptRowRewrapsAndDiscards (0.00s)
--- FAIL: TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit (0.03s)
--- FAIL: TestComposerNoPayloadWalkEngravesAKeylessTemplate (0.01s)
--- FAIL: TestComposerPickScreenRowsAreTouchable (0.00s)
--- FAIL: TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen (0.01s)
```

Eight tests. So every test in the package binds *row index* to *wrapper*; not one
binds *row index* to *the text the operator reads*.

**Second face of the same table.** `initial := 0` with a `break`-on-match loop is a
silent fallback: any `md.ComposeWrapper` value absent from the four-element table
resolves to row 0 = Taproot, and confirming without moving then rewraps the policy to
tr — journey C-1 exactly, reinstated. `md` has four values today
(`md/compose.go:31-36`, `ComposeTr ComposeWrapper = iota`) and all four are listed, so
this is latent rather than live; but `md.ComposeWrapper.ScriptType()` already carries
a `default: return 2` arm (`md/compose.go:41-50`), so `md` itself is not structured to
make a fifth wrapper fail loudly here.

**Why Important and not Minor.** The alignment predates the diff, and the labels are
correct today — this is not a live wrong-result. What the diff adds is (a) a new
dependency on the table, since `Initial` is now derived from `wrappers`, and (b) a new
*promise to the operator* that the highlighted row states the current setting. The
diff adds no test for the property it now sells. Note also that the new test is named
`TestComposerScriptPickerShowsTheScriptInForce` while what it asserts is that the
wrapper is *unchanged*; it reads no label, so it cannot fail on "shows".

Not prescribing the remedy, but it is small: a table test that, for each of the four
wrappers, drives the picker and asserts the *labelled* row under the highlight — or
folding `choices`/`wrappers` into one slice of pairs so the two cannot drift.

---

### M-1 — on the Back leg's second pass the picker opens on the unapplied pick, not the script in force

`gui/composer_shape.go:115-118` states the contract absolutely: *"It opens on the
script CURRENTLY IN FORCE, not on row 0 (journey C-1)."* At the path-list row that is
exactly true — `composerShapeFlow` passes `st.list.Wrapper` live
(`gui/composer_shape.go:449`). At `composerStartStep` it is not: the caller passes the
running local `w` (`gui/composer_flow.go:169`), which after a pick that was never
applied is the operator's last answer rather than the wrapper in force.

**Reproduced** with a scratch test built from `TestComposerBackLegWrapperChange...`,
wsh policy with two seats, on the Back leg:

```
pass 1: picker opens on wsh (row 1). Down -> sh-wsh (row 2), ✓.
        "Start from?" draws. Back, WITHOUT applying anything.
pass 2: picker draws again. ✓ without moving.

    PASS 2 OPENED ON THE UNAPPLIED PICK (sh-wsh): a no-op confirm proposed a rewrap.
    Frame: "EDITINGTHESHAPECLEARSTHEKEYSSlotnumberschangewiththeshape.
            Everykeyyouseatedwillbecleared.Continue?Holdbuttontoconfirm.Edittheshape"
```

**Not the C-1 defect and not silent**, which is why this is Minor: §8j fires before
anything is accepted (`composerStartStep`'s signature comparison), and §4e then
refuses the legacy wrapper outright — the next frame was
`"Legacywrappersholdoneplainmultisigonly.Usewshortr.Spendpaths"`. The pick was also
deliberate one screen earlier, and `gui/composer_shape.go:123-124` deliberately
endorses this ("stepping Back to this screen now shows what was chosen instead of
forgetting it"). The finding is that the comment three lines above says *currently in
force* without qualification, and one of the function's two callers does not satisfy
it. Records, not behaviour.

---

### M-2 — four of the six migrated pokes are covered by nothing

MU-8 — make `Choose` ignore `Initial` entirely (`if false {` over the seed block),
i.e. revert all six migrations at once — full suite:

```
$ scripts/gui-shard-test.sh ./gui/ 24     # RESULT: FAIL, 3 shards
--- FAIL: TestBackPreservesEnteredValues (0.03s)               <- ppQRChoiceFlow
--- FAIL: TestComposerScriptPickerShowsTheScriptInForce (0.01s) <- composerWrapperPick
--- FAIL: TestFTBackPreservesEveryValue (0.04s)                 <- ftQRChoiceFlow
--- FAIL: TestQRStepStillOffersBothAnswersOffTheLadder (0.04s)  <- ftQRChoiceFlow
--- FAIL: TestSizeProofQRStepReturnsFalseOverAStaleOptIn (0.00s)<- ftQRChoiceFlow
```

Five tests, reaching three of the seven `Initial` sites. `ftSpeedChoiceFlow`
(`freetext_flow.go:774`), `ftPassChoiceFlow` (`:827`), `ftFaceChoiceFlow` (`:898`) and
`ftSizeChoiceFlow` (`:917`) have **no test that fails when their preselect is dropped**.

This is a carried-forward gap, not a regression: the old `cs.choice = i` at those four
sites was equally untested, and I proved the migration behaviour-preserving there by
other means (see the call-site table and PROBE-A). Recording it because the diff moved
those four lines and is the natural place to notice.

---

### M-3 — the sized-QR asymmetry is now held by the clamp, so its guard test cannot fail on the branch

Brief item 3 asked whether the migration preserved `ftQRChoiceFlow`'s deliberate
refusal to carry a prior opt-in when `sized`. **It did** — the sized branch sets
`Choices = []string{"No QR"}` and never touches `Initial` (`freetext_flow.go:529-542`),
and `cs` is constructed fresh on every call, so nothing can leak in. But the guard test
can no longer fail on that branch being broken.

MU-10 — hoist `if prior { cs.Initial = 1 }` out of the `else`, so the sized branch
inherits the opt-in:

```
$ go test ./gui/ -run 'TestSizeProofQRStepReturnsFalseOverAStaleOptIn|TestQRStepStillOffersBothAnswersOffTheLadder' -count=1
ok  	seedhammer.com/gui	0.052s
```

Green — because `Choose`'s clamp absorbs it: `Initial = 1` over a one-row list is out
of range and falls to 0. Confirmed directly with a scratch unit test of that exact
shape:

```
$ go test ./gui/ -run TestReviewClampAbsorbsAnOutOfRangeInitial -v
    opened on choice=0 with Initial=1 over 1 rows
--- PASS
```

So `freetext_sizeproof_test.go:784` ("the unavailable state opens on index 0 with a
prior opt-in behind it") now passes via the clamp whatever the branch does. Harmless
today; it stops being harmless the day the sized branch grows a second row, at which
point the leak becomes reachable with no test watching. Worth a line in
`ftQRChoiceFlow`'s comment at minimum.

---

### N-1 — the cancel-path change is inert at both call sites

`composerWrapperPick` now returns `current, false` on Back instead of
`md.ComposeTr, false`. Both callers discard the value on `!ok`
(`composer_flow.go:169-171` assigns to a local and returns immediately;
`composer_shape.go:450-452` returns from the closure). No behaviour change either way;
the new form is the safer one. No action.

### N-2 — the new test asserts half of what the fix guarantees

`TestComposerScriptPickerShowsTheScriptInForce` checks only `st.list.Wrapper`. The
no-op confirm also has to keep the seats and the sources, which is the other half of
C-1's harm. It does — measured, by extending the test in scratch:

```
after the no-op confirm: wrapper=1 assigned=[{src:0 ...} {src:-1 ...} {src:-1 ...} {src:-1 ...}]
                         sources=[{kind:0 used:true ...}]
```

Two extra lines in the shipped test would pin it.

---

## `ChoiceScreen` call sites — the enumeration and the verdicts

The enumeration is closed, which is what makes the table meaningful:

```
$ grep -rn "ChoiceScreen{" --include=*.go . | grep -v _test.go | wc -l        -> 85
$ grep -rn "ChoiceScreen{" --include=*.go . | grep -v "&ChoiceScreen{"        -> (none)
$ grep -rn "\*ChoiceScreen" --include=*.go . | grep -v _test.go               -> comments only
```

All 85 are `&ChoiceScreen{...}` locals. **No `ChoiceScreen` is stored in a struct
field, a package-level var, a slice, or passed to or returned from any function** — so
no screen value outlives the function that built it, and the "struct copied after
`Choose`" counterexample the brief asked for cannot be constructed: there are no
`ChoiceScreen` values, only pointers.

| Group | Sites | What it is | Verdict |
| --- | --- | --- | --- |
| **A — sets `Initial`** (7) | `composer_shape.go:141`, `freetext_flow.go:540/774/827/898/917`, `passphrase_flow.go:406` | fresh `&ChoiceScreen{}` per call; exactly one statement (`hookPPWidget`, nil in production) between the write and `Choose`; `Choose` unconditionally reached | **PASS** — `seeded` is false on a fresh value, so `choice = Initial` applies exactly once and is identical to the old poke |
| **B — reused across ≥2 `Choose` calls, `Initial` unset** (10) | `bip85.go:158`, `bundle_flow.go:627`, `derive_xpub.go:231`, `derive_xpub.go:581`, `gui.go:2765`, `gui.go:2915`, `multisig_verify.go:921`, `plate_verify.go:201`, `transaction.go:1095`, `unlock_platelist.go:247` | one screen, `Choose` in a loop; `Choices` fixed for the life of the value in every one | **PASS** — `seeded` preserves the carried `choice`, which is the pre-diff behaviour exactly. Without the guard these are the screens that break (`TestBackPreservesEnteredValues` et al.) |
| **C — fresh per call, `Initial` unset** (68) | remainder | zero value is row 0 | **PASS by construction** — byte-identical to pre-diff |

**The `seeded` guard was attacked specifically, as the brief asked, and machine-checked
rather than argued.** I instrumented `Choose` with four panicking probes and ran the
whole package:

```go
if !s.seeded { ...; s.probeInitial = s.Initial; s.probeLen = len(s.Choices)
    PROBE-A: panic if s.Initial != 0 && out of range
} else {
    PROBE-B: panic if s.Initial != s.probeInitial      // Initial changed after seeding
    PROBE-C: panic if len(s.Choices) != s.probeLen     // Choices resized on a reused screen
    PROBE-D: panic if s.choice out of range            // carried choice now unmappable
}
```

```
$ scripts/gui-shard-test.sh ./gui/ 24
    partition verified exhaustive: 1294 == 1294
RESULT: ok -- all 1294 tests ran across 24 shards
$ grep -iE "PROBE-|panic" probe.log      -> (no output)
```

None fired. Over the package's whole test surface there is no screen whose `Initial`
is out of range, none whose `Initial` changes between `Choose` calls (so `seeded`
never suppresses an intended opening row — the brief's specific counterexample does
not exist), none whose `Choices` is resized under a live `choice`, and no carried
`choice` that its caller cannot map. Combined with the static enumeration above —
where the only seven `Initial` writers all construct fresh — this closes group A and
group B together.

Range safety at the four `slices.Index` sites is structural, not incidental:
`ftSpeedOptions`, `ftPassOptions`, `ftFaceOptions` and `ftSizeOptions` build their
label slice and their value slice with the same appends on every branch, so an index
into the value slice is always an index into `Choices`.

**Nothing reads `choice` between the assignment and `Choose`:**
`grep -rn "\.choice\b"` over the tree returns no production write to a
`ChoiceScreen.choice` outside `gui.go` itself (the `plates[i].choice` hits are
`hashlockPlate`, a different type), and the only production read is inside `Choose`
and `Draw`. The six migrated sites pass `cs` to `hookPPWidget` before `Choose`; that
seam is `nil` in production and in tests only records the pointer, and every test that
reads `cs.choice` does so after frames have been pumped, i.e. after `Choose` seeded.

---

## Mutation table

| # | Mutation | Expected | Result |
| --- | --- | --- | --- |
| MU-1 | `composer_shape.go:454` — drop `st.list.Wrapper = w` | rewrap half RED | **RED** — `TestComposerChangeTheScriptRowRewrapsAndDiscards`: *"the wrapper is 1, want ComposeTr -- the row did not apply the change"* |
| MU-2 | `composerApplyShapeEdit` never discards | discard half RED | **RED** — both edited tests; backleg: *"a seat survived a wrapper change on the Back leg"* (frame showed `Slot@0:73c5da0a`) |
| MU-3 | remove the **added `click(Up)`** from `composer_gates_test.go` | RED if the preselect really moved the opening row | **RED** — *"the wrapper is 1, want ComposeTr"*. The Up is load-bearing; the test did not become a pass-for-a-new-reason |
| MU-4 | remove the added `click(Up)` from `composer_backleg_test.go` | RED | **RED** — *"the wrapper changed on the Back leg without asking §8j"* fires because nothing changed: the Back-leg no-op is confirmed from the other side |
| MU-5 | `composerWrapperPick` ignores `sel`, always returns `current` | new test GREEN, edited tests RED | **as expected** — proves the new test alone does not prove the picker still applies a deliberate change; the pair does |
| MU-6 | `composerStartStep` skips `composerShapeGuard` | backleg RED on §8j | **RED** |
| MU-7a | `initial = 9` forced, clamp intact | no panic, degrades to row 0 | **RED on the assertion, no panic** — *"the wrapper is 0, want ComposeWsh"* |
| MU-7b | `initial = 9`, clamp removed from `Choose` | panic | **PANIC** — `index out of range [9] with length 4` at `wrappers[sel]`. The clamp is real and load-bearing |
| MU-8 | `Choose` ignores `Initial` (all six migrations reverted at once) | RED | **RED, 5 tests** — see M-2 for the coverage map |
| MU-10 | sized QR branch inherits `Initial` | expected RED | **GREEN** — absorbed by the clamp; see M-3 |
| MU-11 | swap `choices[1]`/`choices[2]` labels only | expected RED | **GREEN across 1294** — see I-1 |
| MU-12 | swap `wrappers[1]`/`wrappers[2]` only | RED | **RED, 8 tests** — the index↔wrapper binding is well covered |
| PROBE-A/B/C/D | assertions inside `Choose`, full suite | never fire | **never fired**, 1294/1294 |

Journey drives (scratch tests against the real flow, both removed):

- **Path-list row, no-op confirm** — wrapper stays `1` (wsh), `assigned[0].src` stays
  `0`, the source stays `used:true`. The C-1 sequence is a no-op in full, not just for
  the wrapper.
- **Back leg, no-op confirm** — nothing is asked, the path list returns, and the stub
  screen still shows both seats:
  `"Slot@0:73c5da0a m/48h/0h/0h/2h  Slot@1:73c5da0a m/48h/0h/1h/2h  Slot@2 expects a key at m/48h/0h/2h/2h"`
  — script-type `2h` throughout, i.e. still wsh. (Under MU-2's tr rewrap the same frame
  read `m/48h/0h/0h/3h`.)
- **Back leg, deliberate change** — the shipped `TestComposerBackLegWrapperChange...`
  covers it and MU-1/MU-2/MU-5 all red it.

Housekeeping checks: `gofmt -l .` returns exactly the five-file pristine baseline
(`gui/transaction.go`, `gui/transaction_golden_test.go`,
`gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`) — the diff adds no
offender. `go build ./...` clean.

---

## Counts

**0 Critical / 1 Important / 3 Minor / 2 Nit — NOT GREEN.**

The Critical fix itself is sound: I could not construct a screen or a sequence in the
shipped code where the preselect opens on, or returns, a row the operator did not
choose, and the new test fails on the defect it names by a mutation independent of the
two already settled. The Important is that the fix's central claim to the operator —
*the highlighted row is the script in force* — is carried by an unasserted parallel
table whose label half no test in the package can see break.
