# composer S4 — W-2 fix report: the pick screen's rows are touch targets

**Implementer:** the same single opus agent, resumed. Brief:
`design/agent-briefs/composer-S4-W2-fix-brief.md`. Defect:
W-2 in `design/S4_journey_walk_2026-09-02.md`, measured and stopped on in
`design/agent-reports/composer-S4-implementation-report.md` (Task 3).

**Outcome: DONE.** The fix shape the controller decided was executed as written
— per-row `Clickable` hit areas on `composerPickScreen`, exactly as
`ChoiceScreen` gives each `Choice` a `click`. The regression test fails on
`60bee002` on the touch path, passes after, and fails again under the mutation.
The emulator confirms it on the machine's own input path.

## Worktree

| repo | path | branch | base |
| --- | --- | --- | --- |
| seedhammer fork | `/scratch/code/shibboleth/wt-composer-s4b` | `composer-s4b` | `main` `60bee00` |

```
$ git -C /scratch/code/shibboleth/wt-composer-s4b log --oneline main..HEAD
2dff0ee gui: the composer's pick-screen rows are touch targets, so the flow can be driven by a hand (S4 walk W-2)
```

Tree clean (`git status --porcelain` empty). Nothing pushed, nothing flashed.
`wt-composer-s4-emu` and `wt-engrave-s4-emu` were not touched — verified: both
still clean, and neither appears in this branch's history. No sub-agent was
dispatched, no `.jsonl` read.

```
 gui/composer_measure_test.go    |   2 +-
 gui/composer_paged.go           | 109 ++++++++++++++++++++---
 gui/composer_paged_test.go      |   4 +-
 gui/composer_pick_touch_test.go | 185 ++++++++++++++++++++++++++++++++++++++++
 gui/composer_stub_test.go       |   4 +-
 5 files changed, 288 insertions(+), 16 deletions(-)
```

## The fix, as built

`composerPickScreen` now declares `var rowHits [composerPickScreenMaxRows]Clickable`
**outside** the frame loop — a `Clickable` carries press state between frames and
the tag a frame registers is the address polled on the next one, so a per-frame
slice would hand the router a pointer that no longer belongs to anything. Per
visible row `j` it polls `rowHits[j].Clicked(ctx)` → `sel = start + j`, and
appends `op.Input(&ctx.B, &rowHits[j]).Clip(bands[b])`.

A tap **selects**; `Button3` still **takes**. `Up`/`Down` still work, `Button2`
still pages. The Clickables are zero-value, so `Clickable.Next`'s repeat arm —
which fires only for `Up`/`Down`/`Left`/`Right` — structurally cannot reach
them: no auto-repeat, no drag, no arrows, as the brief required.

`composerPageLines` gained a third return, the per-row band, because it is the
ONE measure site and a hit area measured anywhere else would be a second answer
to "where is row `i`". `composerReadScreen` discards it — no cursor there, so a
row is not a control, and a hit area would be the present-and-inert affordance
its own icon gate exists to avoid.

### Two decisions inside the decided shape (declared, not hidden)

1. **The band is full-width, not the glyph rectangle.** The brief said "the
   row's drawn rectangle as `composerPageLines` lays it out". Taken as the glyph
   rect, the key-count picker — whose rows are single digits — would get an
   ~8 px-wide target on a device operated by fingertip, i.e. a fix a hand could
   not use. **`ChoiceScreen`'s own rule is padding**: it lays every choice out
   at the widest one's width (`xoff := (maxW-c.Size.X)/2 + buttonPadX`), so a
   short row is not a smaller target than a long one. The band here is the width
   `composerPageLines` already wraps to, which is the same rule one level up.
   Vertically it matches the selection highlight exactly, so what the operator
   sees highlighted is what they can tap. It stops short of the navigation
   column (`dims.X - assets.NavBtnPrimary.Bounds().Size().X`) so no row can
   shadow Back/page/take — `op.Drawer.Hit` returns the *first* registered input
   containing the point, and an overlap would otherwise be settled by traversal
   order rather than intent.
2. **Hit areas are wired for `shown` rows, never `len(bands)`.**
   `composerPageLines` draws a final overflowing row without counting it (its own
   "counted only when it is inside the box" rule). Making that one tappable
   would let the operator take a row the frame cut in half.

## The test — RED on `60bee002` first

`gui/composer_pick_touch_test.go`, on the touch harness (`runUITouch`, `tap`),
driving the real flow `walletPolicyFlow → composerFlow`. Row points are measured
from `composerPageLines`' own layout rule (content starts at `leadingSize+8`;
lead, spacer, then each line advancing by its measured height plus the 6 px gap)
into a **private `op.Buffer`**, so laying out text between frames cannot append
to the buffer the running flow is building. Every lead and row string is
asserted against the live frame **before** it is used as a measurement input, so
a wording change fails an assertion instead of silently shifting the point. No
constant is copied from the JS walks.

The `ChoiceScreen`s on the way in are driven with button clicks on purpose —
they already register per-row hit areas and are covered elsewhere. **Every `tap`
in the test lands on a `composerPickScreen`.**

Run in a throwaway detached worktree at pristine `60bee002` (created, used,
then `git worktree remove`d; `git status --porcelain gui/composer_paged.go` was
empty there, i.e. the file was untouched):

```
$ go test -count=1 -run TestComposerPickScreenRowsAreTouchable ./gui/
--- FAIL: TestComposerPickScreenRowsAreTouchable (0.01s)
    composer_pick_touch_test.go:143: tapping the `3` row did not select it: the threshold picker offers 1..n for the n just taken, and this frame is not 1 2 3.
        Frame: "ThresholdPath1:howmanymustsign?1"
        composerPickScreen's rows are unreachable by touch, which is the only input SeedHammer II has (W-2).
FAIL
FAIL	seedhammer.com/gui	0.008s
EXIT=1
```

**That is W-2 reproduced on the harness**, and the frame is the same one the
emulator produced: `n = 1` after tapping the `3` row.

### GREEN after the fix

```
$ go test -count=1 -v -run TestComposerPickScreenRowsAreTouchable ./gui/
=== RUN   TestComposerPickScreenRowsAreTouchable
--- PASS: TestComposerPickScreenRowsAreTouchable (0.01s)
PASS
ok  	seedhammer.com/gui	0.007s
EXIT=0
```

### Mutation

The `op.Input` wrapper removed and everything else kept — the `Clicked` poll
stays, so this isolates the hit area and nothing else:

```
288:			_ = bands[b] // MUTATION: hit area removed

$ go test -count=1 -run TestComposerPickScreenRowsAreTouchable ./gui/
--- FAIL: TestComposerPickScreenRowsAreTouchable (0.00s)
    composer_pick_touch_test.go:143: tapping the `3` row did not select it: the threshold picker offers 1..n for the n just taken, and this frame is not 1 2 3.
        Frame: "ThresholdPath1:howmanymustsign?1"
FAIL
EXIT=1

$ (restored) go test -count=1 -run TestComposerPickScreenRowsAreTouchable ./gui/
ok  	seedhammer.com/gui	0.008s
EXIT=0
```

The mutated line demonstrably ran: the failure is behavioural (the same frame,
`n = 1`), not a compile error, and `grep -c MUTATION gui/composer_paged.go` is
`0` after the restore.

## Gates

| gate | result |
| --- | --- |
| `gofmt -l gui/` | `transaction.go`, `transaction_golden_test.go`, `transaction_txrecord_test.go` — **identical list at pristine `60bee002`**, so pre-existing. My files clean. |
| `go vet ./gui/` | the two pre-existing `testing.ArtifactDir` findings only (`freetext_sizeproof_golden_test.go:111`, `transaction_golden_test.go:104`) — **identical at `60bee002`** |
| `go test -count=1 -run '^TestComposer' ./gui/` | `ok  seedhammer.com/gui  5.478s` |
| `scripts/gui-shard-test.sh ./gui/ 24` | `RESULT: ok -- all 1187 tests ran across 24 shards`, wall 58s |
| `CGO_ENABLED=0 go test -count=1 ./cmd/emu/` | `ok  seedhammer.com/cmd/emu  1.261s` |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | exit 0 |

Both pre-existing lists were re-measured in the pristine `60bee002` worktree
rather than assumed, so "pre-existing" is a measurement.

The shard count is 1187 against the 1186 recorded for `bc9dd63`: +1, this test.

## The emulator proof — the machine's own input path

`emu.wasm` built from THIS worktree (10,792,051 B), served on a fresh port,
driven by Playwright with `shTap` alone. The three tap points — `(240,179)`,
`(240,150)`, `(240,208)` — were **measured** by a temporary Go test calling the
same `composerPickRowPoint` helper the regression test uses, then that temp file
was deleted (tree verified clean). The same probe was run against a pristine
`60bee002` build as a paired control.

```
                                   60bee002 (before)                                    composer-s4b (after)
door                "Nokeysloaded.Thisbuildsakey-lesstemplate.ScancardsBuild…"          (identical)
n picker            "KeysPath1:howmanykeys?12345"                                       (identical)
after tapping `3`   "ThresholdPath1:howmanymustsign?1"                                  "ThresholdPath1:howmanymustsign?123"
after tapping `2`   "Spendpathsslots:1Path1:1keyAddaspendpathChangethescriptDone"       "Spendpathsslots:3Path1:2-of-3AddaspendpathChangethescriptDone"
after tapping `Done`"Path1:1keyKeysTimelockHashlockRemovepathPath1"                     "Sortedkeys,oryourorder?Sorted(usual)KeepmyorderKeyorder"
```

Before, `Done` opened the **path editor** — row 0, because the cursor never
moved. After, it reaches the key-order question. That is the step that made the
composer a dead end on the machine, and it is now driveable by a hand.

No payload is needed for this proof and none was used (`shSysw("none")`); the
composer payload blob lives on `composer-s4-emu`, which this step did not touch.

## Anything I decided, could not do, or stopped on

1. **The full-width band** (declared above) — the one interpretation inside the
   decided shape, taken because the literal glyph rectangle would leave the
   key-count picker with an unusable target, and because padding to a common
   width is `ChoiceScreen`'s own rule rather than an invention.
2. **Four existing composer test files gained a `_`** for the new return value
   (`composer_measure_test.go`, `composer_paged_test.go` ×2,
   `composer_stub_test.go` ×2). No assertion in any of them changed.
3. **A throwaway detached worktree** at `60bee002` was created for the RED run
   and removed afterwards (`git worktree list` verified). The mutation used a
   `cp` save/restore rather than `git checkout`, so no uncommitted work could be
   discarded.
4. **The other three pick-screen call sites are fixed by construction, not
   separately tested.** `Which hash?` (`composer_hash.go:149`) and `Seat keys`
   (`composer_seat.go:127`) go through the same primitive; the test covers
   `composerCountPick` and the `Spend paths` list, which are the two the S4
   itineraries need. Worth a line in the walk record rather than more tests.
5. **Not done, and not mine:** merging to fork `main`, pushing, flashing. Task 3
   resumes after the fix is on `main`.
